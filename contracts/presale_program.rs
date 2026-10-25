// DRAFT: not compiled, not tested on a network, not audited. Do not deploy with real funds.
// DKEEY ön satış emaneti — DEVNET DENEME SÜRÜMÜ (denetlenmedi, yalnızca test)
// Solana Playground (Anchor 0.29+) içinde derlenir. Para birimi: lamport (SOL).
// Mantık presale_ref.py ile aynı: haftalık devir (rollover) tavanı, kısmi dolum, eşik, çekilebilir iade.
// Test için hafta süresi parametredir (ör. 60 sn).
// Treasury (rezerv çekim tavanı) bu programın içindedir: rezerv PDA'sını yalnız bu program imzalayabilir.
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

pub const NORMAL_BPS: u64 = 50;
pub const CRISIS_BPS: u64 = 100;

declare_id!("11111111111111111111111111111111"); // Playground derlerken kendi ID'sini yazar

#[program]
pub mod dkeey_presale {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, id: u64, start: i64, week_secs: i64, cap: u64, min: u64, rates: [u64; 3]) -> Result<()> {
        require!(week_secs > 0 && cap > 0 && min > 0 && min <= cap, E::BadParams);
        let s = &mut ctx.accounts.sale;
        s.id = id; s.bump = ctx.bumps.sale; s.authority = ctx.accounts.payer.key();
        s.start = start; s.week_secs = week_secs; s.cap = cap; s.min = min; s.rates = rates;
        s.raised = 0; s.finalized = false; s.success = false; s.released = false; s.finalized_at = 0;
        Ok(())
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &mut ctx.accounts.sale;
        require!(!s.finalized, E::Closed);
        require!(now >= s.start && now < s.start + 3 * s.week_secs, E::OutsideWindow);
        require!(s.raised < s.cap, E::Closed);
        let w = (((now - s.start) / s.week_secs) as u64).min(2);
        let cum = ((w as u128 + 1) * s.cap as u128 / 3) as u64;
        let room = cum.saturating_sub(s.raised).min(s.cap - s.raised);
        let take = amount.min(room);
        require!(take > 0, E::NoRoom);
        // yalnızca 'take' çekilir; fazlası hiç alınmaz (aynı çağrıda iade ile eşdeğer)
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.user.to_account_info(), to: s.to_account_info() }), take)?;
        s.raised += take;
        let c = &mut ctx.accounts.contrib;
        c.amount += take;
        c.tokens += (take as u128 * s.rates[w as usize] as u128) as u64;
        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &mut ctx.accounts.sale;
        require!(!s.finalized, E::Closed);
        require!(now >= s.start + 3 * s.week_secs || s.raised >= s.cap, E::TooEarly);
        s.finalized = true;
        s.success = s.raised >= s.min;
        s.finalized_at = now;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let s = &ctx.accounts.sale;
        let c = &mut ctx.accounts.contrib;
        require!(s.finalized && !s.success, E::NoRefund);
        require!(!c.refunded && c.amount > 0, E::NoRefund);
        c.refunded = true;
        **s.to_account_info().try_borrow_mut_lamports()? -= c.amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += c.amount;
        Ok(())
    }

    // Basari sonrasi, izinsiz, bir kez: %75 rezerv hesabi, %25 likidite ve tek seferlik gider hesabi (42.8 A2).
    // Hedefler bu programin PDA'laridir; cekim kurali (epoch basina %0,5) tr_withdraw'da.
    pub fn release(ctx: Context<Release>) -> Result<()> {
        let s = &mut ctx.accounts.sale;
        require!(s.finalized && s.success && !s.released, E::NoRelease);
        let total = s.raised;
        let reserve = ((total as u128) * 75 / 100) as u64;
        let liquidity = total - reserve;
        **s.to_account_info().try_borrow_mut_lamports()? -= total;
        **ctx.accounts.reserve.to_account_info().try_borrow_mut_lamports()? += reserve;
        **ctx.accounts.liquidity.to_account_info().try_borrow_mut_lamports()? += liquidity;
        s.released = true;
        Ok(())
    }

    // Token hakki: finalize anindan itibaren 8 hafta dogrusal acilim. Hesap bazli kayit; gercek token programi baglaninca CPI eklenecek.
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &ctx.accounts.sale;
        let c = &mut ctx.accounts.contrib;
        require!(s.finalized && s.success, E::NoClaim);
        let span = (8 * s.week_secs) as i128;
        let el = ((now - s.finalized_at) as i128).max(0).min(span);
        let vested = ((c.tokens as u128) * (el as u128) / (span as u128)) as u64;
        let x = vested.saturating_sub(c.claimed);
        require!(x > 0, E::NoClaim);
        c.claimed += x;
        Ok(())
    }
    // ---- Treasury: rezervden epoch basina cekim tavani; hedef yalniz sabit ops PDA'si ----
    // Yalniz Sale.authority imzasi; yalniz release sonrasi; bir kez.
    pub fn tr_init(ctx: Context<TrInit>, epoch_secs: i64) -> Result<()> {
        require!(ctx.accounts.sale.released, E::NoRelease);
        require!(epoch_secs > 0, E::BadParams);
        let t = &mut ctx.accounts.treasury;
        t.sale = ctx.accounts.sale.key(); t.authority = ctx.accounts.sale.authority;
        t.start = Clock::get()?.unix_timestamp; t.epoch_secs = epoch_secs;
        t.epoch = u64::MAX; t.snapshot = 0; t.withdrawn = 0; t.crisis = false;
        t.reserve_bump = ctx.bumps.reserve;
        Ok(())
    }

    pub fn tr_set_mode(ctx: Context<TrSetMode>, crisis: bool) -> Result<()> {
        ctx.accounts.treasury.crisis = crisis;
        Ok(())
    }

    pub fn tr_withdraw(ctx: Context<TrWithdraw>, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let t = &mut ctx.accounts.treasury;
        require!(amount > 0 && now >= t.start, E::BadParams);
        let e = ((now - t.start) / t.epoch_secs) as u64;
        let bal = ctx.accounts.reserve.to_account_info().lamports();
        if t.epoch == u64::MAX || e > t.epoch { t.epoch = e; t.snapshot = bal; t.withdrawn = 0; }
        require!(e == t.epoch, E::TimeBack);
        let bps = if t.crisis { CRISIS_BPS } else { NORMAL_BPS };
        let cap = ((t.snapshot as u128) * bps as u128 / 10_000) as u64;
        require!(t.withdrawn.checked_add(amount).ok_or(E::BadParams)? <= cap, E::OverCap);
        require!(amount <= bal, E::NoBalance);
        let sale = t.sale;
        let seeds: &[&[u8]] = &[b"reserve", sale.as_ref(), &[t.reserve_bump]];
        let signer = &[seeds];
        transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.reserve.to_account_info(), to: ctx.accounts.ops.to_account_info() }, signer), amount)?;
        t.withdrawn += amount;
        Ok(())
    }
}

#[account]
pub struct Sale { pub id: u64, pub bump: u8, pub start: i64, pub week_secs: i64, pub cap: u64, pub min: u64, pub rates: [u64; 3], pub raised: u64, pub finalized: bool, pub success: bool, pub released: bool, pub finalized_at: i64, pub authority: Pubkey }
#[account]
pub struct Contrib { pub amount: u64, pub tokens: u64, pub refunded: bool, pub claimed: u64 }

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 8 + 1 + 8*4 + 24 + 8 + 2 + 1 + 8 + 32, seeds = [b"sale", id.to_le_bytes().as_ref()], bump)]
    pub sale: Account<'info, Sale>,
    #[account(mut)] pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
    #[account(init_if_needed, payer = user, space = 8 + 8 + 8 + 1 + 8, seeds = [b"c", sale.key().as_ref(), user.key().as_ref()], bump)]
    pub contrib: Account<'info, Contrib>,
    #[account(mut)] pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
}
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
    #[account(mut, seeds = [b"c", sale.key().as_ref(), user.key().as_ref()], bump)] pub contrib: Account<'info, Contrib>,
    #[account(mut)] pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Release<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
    #[account(mut, seeds = [b"reserve", sale.key().as_ref()], bump)] pub reserve: SystemAccount<'info>,
    #[account(mut, seeds = [b"liq", sale.key().as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    #[account(mut)] pub caller: Signer<'info>,
}
#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
    #[account(mut, seeds = [b"c", sale.key().as_ref(), user.key().as_ref()], bump)] pub contrib: Account<'info, Contrib>,
    #[account(mut)] pub user: Signer<'info>,
}

#[account]
pub struct Treasury { pub sale: Pubkey, pub authority: Pubkey, pub start: i64, pub epoch_secs: i64, pub epoch: u64, pub snapshot: u64, pub withdrawn: u64, pub crisis: bool, pub reserve_bump: u8 }

#[derive(Accounts)]
pub struct TrInit<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
    #[account(init, payer = authority, space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1, seeds = [b"treasury", sale.key().as_ref()], bump)]
    pub treasury: Account<'info, Treasury>,
    #[account(seeds = [b"reserve", sale.key().as_ref()], bump)] pub reserve: SystemAccount<'info>,
    #[account(mut, address = sale.authority)] pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct TrSetMode<'info> {
    #[account(mut, has_one = authority)] pub treasury: Account<'info, Treasury>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct TrWithdraw<'info> {
    #[account(mut, has_one = authority)] pub treasury: Account<'info, Treasury>,
    #[account(mut, seeds = [b"reserve", treasury.sale.as_ref()], bump = treasury.reserve_bump)] pub reserve: SystemAccount<'info>,
    #[account(mut, seeds = [b"ops", treasury.sale.as_ref()], bump)] pub ops: SystemAccount<'info>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum E { #[msg("bad params")] BadParams, #[msg("closed")] Closed, #[msg("outside window")] OutsideWindow,
  #[msg("no room this week")] NoRoom, #[msg("too early")] TooEarly, #[msg("nothing to refund")] NoRefund,
  #[msg("cannot release")] NoRelease, #[msg("nothing to claim")] NoClaim,
  #[msg("epoch cap exceeded")] OverCap, #[msg("not enough balance")] NoBalance, #[msg("time went back")] TimeBack }
