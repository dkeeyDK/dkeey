// DKEEY Treasury modulu — TASLAK (derlenmedi, denetlenmedi). Mantik treasury_ref.py ile ayni.
// Rezerv hesabi presale_program.rs'deki PDA ["reserve", sale]'dir. Cekim yalniz sabit ops PDA'sina gider.
// Epoch basina cekim tavani: Normal %0,5, Crisis %1 (epoch basindaki bakiyeden). Kullanilmayan tavan birikmez.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

declare_id!("11111111111111111111111111111111"); // Playground kendi ID'sini yazar

pub const NORMAL_BPS: u64 = 50;
pub const CRISIS_BPS: u64 = 100;

#[program]
pub mod dkeey_treasury {
    use super::*;

    pub fn init(ctx: Context<Init>, sale: Pubkey, start: i64, epoch_secs: i64, authority: Pubkey) -> Result<()> {
        require!(epoch_secs > 0, E::BadParams);
        let t = &mut ctx.accounts.treasury;
        t.sale = sale; t.authority = authority; t.start = start; t.epoch_secs = epoch_secs;
        t.epoch = u64::MAX; t.snapshot = 0; t.withdrawn = 0; t.crisis = false;
        t.reserve_bump = ctx.bumps.reserve;
        Ok(())
    }

    // Yalniz yetkili imza (Ag Cekirdegi taahhut hatti) modu degistirir.
    pub fn set_mode(ctx: Context<SetMode>, crisis: bool) -> Result<()> {
        ctx.accounts.treasury.crisis = crisis;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
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
        invoke_signed(
            &system_instruction::transfer(&ctx.accounts.reserve.key(), &ctx.accounts.ops.key(), amount),
            &[ctx.accounts.reserve.to_account_info(), ctx.accounts.ops.to_account_info(), ctx.accounts.system_program.to_account_info()],
            &[seeds],
        )?;
        t.withdrawn += amount;
        Ok(())
    }
}

#[account]
pub struct Treasury { pub sale: Pubkey, pub authority: Pubkey, pub start: i64, pub epoch_secs: i64, pub epoch: u64, pub snapshot: u64, pub withdrawn: u64, pub crisis: bool, pub reserve_bump: u8 }

#[derive(Accounts)]
#[instruction(sale: Pubkey)]
pub struct Init<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1, seeds = [b"treasury", sale.as_ref()], bump)]
    pub treasury: Account<'info, Treasury>,
    #[account(seeds = [b"reserve", sale.as_ref()], bump)] pub reserve: SystemAccount<'info>,
    #[account(mut)] pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SetMode<'info> {
    #[account(mut, has_one = authority)] pub treasury: Account<'info, Treasury>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = authority)] pub treasury: Account<'info, Treasury>,
    #[account(mut, seeds = [b"reserve", treasury.sale.as_ref()], bump = treasury.reserve_bump)] pub reserve: SystemAccount<'info>,
    #[account(mut, seeds = [b"ops", treasury.sale.as_ref()], bump)] pub ops: SystemAccount<'info>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum E { #[msg("bad params")] BadParams, #[msg("epoch cap exceeded")] OverCap, #[msg("not enough balance")] NoBalance, #[msg("time went back")] TimeBack }
