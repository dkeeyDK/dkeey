// DRAFT v2: yeni mekanizmalar (ajan teminati, kutu merdiveni, kilitli likidite, staking tavani) EKLENDI.
// Not compiled yet in Playground, not tested on a network, not audited. Do not deploy with real funds.
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

    // ================= Yeni mekanizmalar (TASLAK: derlenmedi, agda test edilmedi, denetlenmedi) =================
    // Komite karari = Cfg.authority imzasi; ayarlar sozlesmedeki SERT sinirlar (HARD_*) icinde kalir.
    // Token ve ucret muhasebesi hesap bazlidir (gercek SPL token programi baglanmadi); para birimi lamport.

    pub fn init_cfg(ctx: Context<InitCfg>, p: CfgParams) -> Result<()> {
        check_params(&p)?;
        let c = &mut ctx.accounts.cfg;
        c.sale = ctx.accounts.sale.key(); c.authority = ctx.accounts.authority.key();
        c.start = Clock::get()?.unix_timestamp;
        apply_params(c, &p);
        c.lpv_bump = ctx.bumps.lpv; c.lfee_bump = ctx.bumps.lfee;
        c.year_dist = [0; 5];
        Ok(())
    }

    pub fn update_cfg(ctx: Context<UpdateCfg>, p: CfgParams) -> Result<()> {
        check_params(&p)?;
        apply_params(&mut ctx.accounts.cfg, &p);
        Ok(())
    }

    // ---- 1) Ajan teminati: bir cuzdan = bir ajan (PDA tek) ----
    pub fn register_agent(ctx: Context<RegisterAgent>, daily_limit: u64) -> Result<()> {
        require!(daily_limit > 0, E::BadParams);
        let c = &ctx.accounts.cfg;
        let bond = math::required_bond(daily_limit, c.bond_bps, c.min_bond).ok_or(E::BadParams)?;
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.owner.to_account_info(), to: ctx.accounts.agent.to_account_info() }), bond)?;
        let a = &mut ctx.accounts.agent;
        a.cfg = ctx.accounts.cfg.key(); a.owner = ctx.accounts.owner.key();
        a.bond = bond; a.daily_limit = daily_limit; a.status = 0; a.exit_at = 0;
        a.strikes = 0; a.score_bps = 0; a.slashed = 0;
        Ok(())
    }

    pub fn request_exit(ctx: Context<AgentOwner>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let a = &mut ctx.accounts.agent;
        require!(a.status == 0, E::NotActive);
        a.status = 1; a.exit_at = now + ctx.accounts.cfg.exit_wait;
        Ok(())
    }

    // Bekleme bitince kalan teminat sahibine iade edilir (ihlal varsa slash zaten dusmustur).
    pub fn withdraw_bond(ctx: Context<AgentOwner>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let a = &mut ctx.accounts.agent;
        require!(a.status == 1 && now >= a.exit_at, E::Locked);
        let b = a.bond;
        require!(b > 0, E::NoBalance);
        a.bond = 0; a.status = 2;
        **a.to_account_info().try_borrow_mut_lamports()? -= b;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += b;
        Ok(())
    }

    // severity: 0 uyari (yalniz ilk hata), 1 kucuk (%10), 2 ciddi (%100). Kesilen para havuza (liq) gider.
    pub fn slash(ctx: Context<Slash>, severity: u8) -> Result<()> {
        let a = &mut ctx.accounts.agent;
        let (amt, strikes) = math::slash_amount(a.bond, severity, a.strikes).ok_or(E::BadParams)?;
        let amt = amt.min(a.bond);
        if amt > 0 {
            a.bond -= amt; a.slashed += amt;
            **a.to_account_info().try_borrow_mut_lamports()? -= amt;
            **ctx.accounts.liquidity.to_account_info().try_borrow_mut_lamports()? += amt;
        }
        a.strikes = strikes;
        if severity == 2 { a.status = 2; }
        Ok(())
    }

    // Dogrulanmis is skoru (bps, en fazla 5000 = %50 bonus). Ajan is yapmazsa 0 kalir.
    pub fn set_score(ctx: Context<SetScore>, score_bps: u64) -> Result<()> {
        require!(score_bps <= HARD_BONUS_MAX, E::OutOfBounds);
        ctx.accounts.agent.score_bps = score_bps;
        Ok(())
    }

    // ---- 2) Kutu merdiveni: kutu icinde sabit fiyat, kutu dolunca adim kadar artis ----
    pub fn init_ladder(ctx: Context<InitLadder>, price: u64, bin_size: u64, supply: u64) -> Result<()> {
        require!(price > 0 && bin_size > 0 && supply >= bin_size, E::BadParams);
        let l = &mut ctx.accounts.ladder;
        l.cfg = ctx.accounts.cfg.key(); l.price = price; l.bin_size = bin_size; l.bin_sold = 0;
        l.step_bps = 0; l.bin_index = 0; l.supply = supply; l.sold_total = 0; l.last_step_ts = 0;
        Ok(())
    }

    // Adim ajan(lar) tarafindan ayarlanir; her cagrida en fazla 200 bps degisir, en fazla %5, en az 60 sn arayla.
    pub fn set_step(ctx: Context<SetStep>, step_bps: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let l = &mut ctx.accounts.ladder;
        require!(step_bps <= STEP_MAX, E::OutOfBounds);
        let d = if step_bps > l.step_bps { step_bps - l.step_bps } else { l.step_bps - step_bps };
        require!(d <= STEP_DELTA_MAX, E::OutOfBounds);
        require!(l.last_step_ts == 0 || now - l.last_step_ts >= 60, E::TooSoon);
        l.step_bps = step_bps; l.last_step_ts = now;
        Ok(())
    }

    // Alim: en fazla MAX_BINS_TX kutu doldurur (kismi dolum olabilir). Odeme havuza (liq) gider.
    pub fn buy_ladder(ctx: Context<BuyLadder>, dk: u64, max_cost: u64) -> Result<()> {
        let l = &mut ctx.accounts.ladder;
        let remaining = l.supply - l.sold_total;
        let want = dk.min(remaining);
        require!(want > 0, E::NoRoom);
        let (cost, p, sold, filled, bins) = math::ladder_buy(l.price, l.bin_size, l.bin_sold, l.step_bps, want, MAX_BINS_TX)
            .ok_or(E::BadParams)?;
        require!(filled > 0 && cost <= max_cost, E::Slippage);
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.user.to_account_info(), to: ctx.accounts.liquidity.to_account_info() }), cost)?;
        l.price = p; l.bin_sold = sold; l.bin_index += bins; l.sold_total += filled;
        let h = &mut ctx.accounts.holder;
        h.dk += filled;
        Ok(())
    }

    // ---- 3) Kilitli likidite: NAV'a gore pay, 1 yil kilit, sonra kademeli cikis, komisyonlu ucret paylasimi ----
    pub fn init_lp(ctx: Context<InitLp>) -> Result<()> {
        let p = &mut ctx.accounts.lpool;
        p.cfg = ctx.accounts.cfg.key(); p.total_shares = 0; p.total_weight = 0; p.acc = 0;
        Ok(())
    }

    // Bir cuzdan = bir pozisyon (ikinci yatirim ayni cuzdandan yapilamaz).
    pub fn deposit_lp(ctx: Context<DepositLp>, amount: u64) -> Result<()> {
        require!(amount >= MIN_LP, E::BadParams);
        let now = Clock::get()?.unix_timestamp;
        let c = &ctx.accounts.cfg;
        let nav = ctx.accounts.lpv.to_account_info().lamports();
        let pool = &mut ctx.accounts.lpool;
        let shares = math::shares_for(amount, pool.total_shares, nav).ok_or(E::BadParams)?;
        require!(shares > 0, E::NoShares);
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.owner.to_account_info(), to: ctx.accounts.lpv.to_account_info() }), amount)?;
        let mult = math::mult_bps(now, c.start, c.early_secs, c.early_mult);
        let weight = (shares as u128) * mult as u128 / 10_000;
        let pos = &mut ctx.accounts.pos;
        pos.owner = ctx.accounts.owner.key(); pos.shares = shares; pos.init_shares = shares; pos.withdrawn = 0;
        pos.weight = weight; pos.mult = mult; pos.lock_until = now + c.lp_lock;
        pos.debt = weight.saturating_mul(pool.acc) / math::SCALE; pos.owed = 0;
        pool.total_shares += shares; pool.total_weight += weight;
        Ok(())
    }

    // Ucret girisi (ornegin islem ucreti): komisyon rezerve, kalan LP'lere agirliga gore.
    pub fn add_fees(ctx: Context<AddFees>, amount: u64) -> Result<()> {
        require!(amount >= MIN_FEE, E::NoFee);
        let pool = &mut ctx.accounts.lpool;
        require!(pool.total_weight > 0, E::NoWeight);
        let (fee, rest) = math::split_commission(amount, ctx.accounts.cfg.commission_bps);
        if fee > 0 {
            transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
                Transfer { from: ctx.accounts.payer.to_account_info(), to: ctx.accounts.reserve.to_account_info() }), fee)?;
        }
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.payer.to_account_info(), to: ctx.accounts.lfee.to_account_info() }), rest)?;
        pool.acc += (rest as u128) * math::SCALE / pool.total_weight;
        Ok(())
    }

    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        let pool = &ctx.accounts.lpool;
        let pos = &mut ctx.accounts.pos;
        let total = (pos.owed as u128 + math::pending_fee(pos.weight, pool.acc, pos.debt) as u128) as u64;
        let avail = ctx.accounts.lfee.to_account_info().lamports().saturating_sub(FLOOR);
        let pay = total.min(avail);
        require!(pay > 0, E::NoFee);
        if pay <= pos.owed { pos.owed -= pay; } else { pos.debt += (pay - pos.owed) as u128; pos.owed = 0; }
        let cfgk = ctx.accounts.cfg.key();
        let seeds: &[&[u8]] = &[b"lfee", cfgk.as_ref(), &[ctx.accounts.cfg.lfee_bump]];
        let signer = &[seeds];
        transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.lfee.to_account_info(), to: ctx.accounts.owner.to_account_info() }, signer), pay)?;
        Ok(())
    }

    // Kilit bittikten sonra herkes cagirabilir: erken donem carpani biter, agirlik paya esitlenir.
    pub fn end_bonus(ctx: Context<EndBonus>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let pool = &mut ctx.accounts.lpool;
        let pos = &mut ctx.accounts.pos;
        require!(now >= pos.lock_until && pos.mult > 10_000, E::TooSoon);
        pos.owed += math::pending_fee(pos.weight, pool.acc, pos.debt);
        let new_w = pos.shares as u128;
        pool.total_weight = pool.total_weight - pos.weight + new_w;
        pos.weight = new_w; pos.mult = 10_000;
        pos.debt = new_w.saturating_mul(pool.acc) / math::SCALE;
        Ok(())
    }

    pub fn withdraw_lp(ctx: Context<WithdrawLp>, burn: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let c = &ctx.accounts.cfg;
        let pool = &mut ctx.accounts.lpool;
        let pos = &mut ctx.accounts.pos;
        require!(burn > 0 && burn <= pos.shares, E::BadParams);
        let allowed = math::allowed_exit(pos.init_shares, c.lp_exit_bps, pos.lock_until, now);
        require!(pos.withdrawn.checked_add(burn).ok_or(E::BadParams)? <= allowed, E::ExitCap);
        let nav = ctx.accounts.lpv.to_account_info().lamports();
        let out = math::value_for(burn, pool.total_shares, nav).ok_or(E::BadParams)?;
        let left = nav - out;
        require!(left == 0 || left >= FLOOR, E::Floor);
        // bekleyen ucreti sakla, agirligi orantili dusur
        pos.owed += math::pending_fee(pos.weight, pool.acc, pos.debt);
        let wburn = pos.weight * burn as u128 / pos.shares as u128;
        pos.weight -= wburn; pool.total_weight -= wburn;
        pool.total_shares -= burn; pos.shares -= burn; pos.withdrawn += burn;
        pos.debt = pos.weight.saturating_mul(pool.acc) / math::SCALE;
        let cfgk = ctx.accounts.cfg.key();
        let seeds: &[&[u8]] = &[b"lpv", cfgk.as_ref(), &[c.lpv_bump]];
        let signer = &[seeds];
        transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.lpv.to_account_info(), to: ctx.accounts.owner.to_account_info() }, signer), out)?;
        Ok(())
    }

    // ---- 4) Staking tavani: yillik odul = stake x tavan; ajan bonusu tavanin ustune (en fazla +%50), ajan basina sinirli ----
    pub fn stake_dk(ctx: Context<StakeDk>, amount: u64) -> Result<()> {
        require!(amount > 0, E::BadParams);
        let now = Clock::get()?.unix_timestamp;
        let c = &ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        if s.last_ts == 0 {
            s.owner = ctx.accounts.owner.key(); s.cfg = ctx.accounts.cfg.key(); s.last_ts = now;
        }
        accrue(s, c, now);
        s.amount += amount;
        Ok(())
    }

    pub fn unstake_dk(ctx: Context<StakeOwner>, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let c = &ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        require!(amount > 0 && amount <= s.amount, E::BadParams);
        accrue(s, c, now);
        s.amount -= amount;
        Ok(())
    }

    // Odeme yalniz o yilin butcesinden (75M, 62.5M, 50M, 37.5M, 25M DK); butce bitince odeme yok (ilk gelen alir).
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let c = &mut ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        accrue(s, c, now);
        let idx = math::year_index(now, c.start).ok_or(E::Budget)?;
        let avail = math::YEAR_BUDGET[idx].saturating_sub(c.year_dist[idx]);
        let pay = s.accrued.min(avail);
        require!(pay > 0, E::Budget);
        s.accrued -= pay; c.year_dist[idx] += pay;
        Ok(())
    }

    // Herkes cagirabilir: bonus, sahibin aktif ajaninin skoruna esitlenir (ajan yoksa/kapaliysa 0).
    pub fn sync_bonus(ctx: Context<SyncBonus>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let c = &ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        accrue(s, c, now);
        let mut b = 0u64;
        if let Some(a) = &ctx.accounts.agent {
            if a.cfg == s.cfg && a.owner == s.owner && a.status == 0 { b = a.score_bps.min(HARD_BONUS_MAX); }
        }
        s.bonus_bps = b;
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
  #[msg("epoch cap exceeded")] OverCap, #[msg("not enough balance")] NoBalance, #[msg("time went back")] TimeBack,
  #[msg("slippage or no fill")] Slippage, #[msg("locked")] Locked, #[msg("exit cap")] ExitCap, #[msg("no shares")] NoShares,
  #[msg("floor")] Floor, #[msg("budget exhausted")] Budget, #[msg("not active")] NotActive, #[msg("out of bounds")] OutOfBounds,
  #[msg("no weight")] NoWeight, #[msg("too soon")] TooSoon, #[msg("no fee")] NoFee }

// ================= Yeni mekanizmalar: sabitler, hesaplar, baglamlar (TASLAK) =================
pub const MAX_BINS_TX: u64 = 5;
pub const STEP_MAX: u64 = 500;        // kutu adimi en fazla %5
pub const STEP_DELTA_MAX: u64 = 200;  // tek guncellemede en fazla 200 bps degisim
pub const HARD_BONUS_MAX: u64 = 5000; // ajan bonusu en fazla %50
pub const FLOOR: u64 = 1_000_000;     // sistem hesaplarinda birakilacak taban (kira muafiyeti)
pub const MIN_LP: u64 = 1_000_000;
pub const MIN_FEE: u64 = 2_000_000;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CfgParams {
    pub min_bond: u64, pub bond_bps: u64, pub exit_wait: i64, pub lp_lock: i64, pub lp_exit_bps: u64,
    pub early_secs: i64, pub early_mult: u64, pub commission_bps: u64, pub stake_cap_bps: u64, pub agent_stake_cap: u64,
}

// Sert sinirlar: komite bu araliklarin disina cikamaz.
fn check_params(p: &CfgParams) -> Result<()> {
    require!(p.min_bond > 0 && p.agent_stake_cap > 0, E::OutOfBounds);
    require!((100..=1_500).contains(&p.bond_bps), E::OutOfBounds);
    require!((7 * 86_400..=14 * 86_400).contains(&p.exit_wait), E::OutOfBounds);
    require!((90 * 86_400..=730 * 86_400).contains(&p.lp_lock), E::OutOfBounds);
    require!((100..=500).contains(&p.lp_exit_bps), E::OutOfBounds);
    require!((0..=365 * 86_400).contains(&p.early_secs), E::OutOfBounds);
    require!((10_000..=20_000).contains(&p.early_mult), E::OutOfBounds);
    require!(p.commission_bps <= 2_000, E::OutOfBounds);
    require!((500..=4_000).contains(&p.stake_cap_bps), E::OutOfBounds);
    Ok(())
}
fn apply_params(c: &mut Cfg, p: &CfgParams) {
    c.min_bond = p.min_bond; c.bond_bps = p.bond_bps; c.exit_wait = p.exit_wait; c.lp_lock = p.lp_lock;
    c.lp_exit_bps = p.lp_exit_bps; c.early_secs = p.early_secs; c.early_mult = p.early_mult;
    c.commission_bps = p.commission_bps; c.stake_cap_bps = p.stake_cap_bps; c.agent_stake_cap = p.agent_stake_cap;
}
fn accrue(s: &mut Stake, c: &Cfg, now: i64) {
    let add = math::stake_accrue(s.amount, c.agent_stake_cap, c.stake_cap_bps, s.bonus_bps, now - s.last_ts);
    s.accrued = s.accrued.saturating_add(add);
    s.last_ts = now;
}

#[account]
pub struct Cfg { pub sale: Pubkey, pub authority: Pubkey, pub start: i64, pub min_bond: u64, pub bond_bps: u64, pub exit_wait: i64,
    pub lp_lock: i64, pub lp_exit_bps: u64, pub early_secs: i64, pub early_mult: u64, pub commission_bps: u64,
    pub stake_cap_bps: u64, pub agent_stake_cap: u64, pub lpv_bump: u8, pub lfee_bump: u8, pub year_dist: [u64; 5] }
#[account]
pub struct Agent { pub cfg: Pubkey, pub owner: Pubkey, pub bond: u64, pub daily_limit: u64, pub status: u8, pub exit_at: i64,
    pub strikes: u8, pub score_bps: u64, pub slashed: u64 }
#[account]
pub struct Ladder { pub cfg: Pubkey, pub price: u64, pub bin_size: u64, pub bin_sold: u64, pub step_bps: u64, pub bin_index: u64,
    pub supply: u64, pub sold_total: u64, pub last_step_ts: i64 }
#[account]
pub struct Holder { pub dk: u64 }
#[account]
pub struct LpPool { pub cfg: Pubkey, pub total_shares: u64, pub total_weight: u128, pub acc: u128 }
#[account]
pub struct LpPos { pub owner: Pubkey, pub shares: u64, pub init_shares: u64, pub withdrawn: u64, pub weight: u128, pub mult: u64,
    pub lock_until: i64, pub debt: u128, pub owed: u64 }
#[account]
pub struct Stake { pub owner: Pubkey, pub cfg: Pubkey, pub amount: u64, pub last_ts: i64, pub accrued: u64, pub bonus_bps: u64 }

#[derive(Accounts)]
pub struct InitCfg<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Account<'info, Sale>,
    #[account(init, payer = authority, space = 8 + 32 + 32 + 8*11 + 1 + 1 + 40, seeds = [b"cfg", sale.key().as_ref()], bump)]
    pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"lpv", cfg.key().as_ref()], bump)] pub lpv: SystemAccount<'info>,
    #[account(seeds = [b"lfee", cfg.key().as_ref()], bump)] pub lfee: SystemAccount<'info>,
    #[account(mut, address = sale.authority)] pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UpdateCfg<'info> {
    #[account(mut, has_one = authority)] pub cfg: Account<'info, Cfg>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(init, payer = owner, space = 8 + 32 + 32 + 8 + 8 + 1 + 8 + 1 + 8 + 8, seeds = [b"agent", cfg.key().as_ref(), owner.key().as_ref()], bump)]
    pub agent: Account<'info, Agent>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct AgentOwner<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"agent", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub agent: Account<'info, Agent>,
    #[account(mut)] pub owner: Signer<'info>,
}
#[derive(Accounts)]
pub struct Slash<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"agent", cfg.key().as_ref(), agent.owner.as_ref()], bump)] pub agent: Account<'info, Agent>,
    #[account(mut, seeds = [b"liq", cfg.sale.as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct SetScore<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"agent", cfg.key().as_ref(), agent.owner.as_ref()], bump)] pub agent: Account<'info, Agent>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitLadder<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(init, payer = authority, space = 8 + 32 + 8*8, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Account<'info, Ladder>,
    #[account(mut)] pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SetStep<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Account<'info, Ladder>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct BuyLadder<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Account<'info, Ladder>,
    #[account(init_if_needed, payer = user, space = 8 + 8, seeds = [b"h", cfg.key().as_ref(), user.key().as_ref()], bump)] pub holder: Account<'info, Holder>,
    #[account(mut, seeds = [b"liq", cfg.sale.as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    #[account(mut)] pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitLp<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(init, payer = authority, space = 8 + 32 + 8 + 16 + 16, seeds = [b"lpool", cfg.key().as_ref()], bump)] pub lpool: Account<'info, LpPool>,
    #[account(mut)] pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct DepositLp<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"lpool", cfg.key().as_ref()], bump)] pub lpool: Account<'info, LpPool>,
    #[account(mut, seeds = [b"lpv", cfg.key().as_ref()], bump = cfg.lpv_bump)] pub lpv: SystemAccount<'info>,
    #[account(init, payer = owner, space = 8 + 32 + 8 + 8 + 8 + 16 + 8 + 8 + 16 + 8, seeds = [b"lp", cfg.key().as_ref(), owner.key().as_ref()], bump)] pub pos: Account<'info, LpPos>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct AddFees<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"lpool", cfg.key().as_ref()], bump)] pub lpool: Account<'info, LpPool>,
    #[account(mut, seeds = [b"lfee", cfg.key().as_ref()], bump = cfg.lfee_bump)] pub lfee: SystemAccount<'info>,
    #[account(mut, seeds = [b"reserve", cfg.sale.as_ref()], bump)] pub reserve: SystemAccount<'info>,
    #[account(mut)] pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct ClaimFees<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"lpool", cfg.key().as_ref()], bump)] pub lpool: Account<'info, LpPool>,
    #[account(mut, seeds = [b"lp", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub pos: Account<'info, LpPos>,
    #[account(mut, seeds = [b"lfee", cfg.key().as_ref()], bump = cfg.lfee_bump)] pub lfee: SystemAccount<'info>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct EndBonus<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"lpool", cfg.key().as_ref()], bump)] pub lpool: Account<'info, LpPool>,
    #[account(mut, seeds = [b"lp", cfg.key().as_ref(), pos.owner.as_ref()], bump)] pub pos: Account<'info, LpPos>,
}
#[derive(Accounts)]
pub struct WithdrawLp<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"lpool", cfg.key().as_ref()], bump)] pub lpool: Account<'info, LpPool>,
    #[account(mut, seeds = [b"lp", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub pos: Account<'info, LpPos>,
    #[account(mut, seeds = [b"lpv", cfg.key().as_ref()], bump = cfg.lpv_bump)] pub lpv: SystemAccount<'info>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeDk<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(init_if_needed, payer = owner, space = 8 + 32 + 32 + 8 + 8 + 8 + 8, seeds = [b"stake", cfg.key().as_ref(), owner.key().as_ref()], bump)] pub stake: Account<'info, Stake>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct StakeOwner<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"stake", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub stake: Account<'info, Stake>,
    pub owner: Signer<'info>,
}
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)] pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"stake", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub stake: Account<'info, Stake>,
    pub owner: Signer<'info>,
}
#[derive(Accounts)]
pub struct SyncBonus<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"stake", cfg.key().as_ref(), stake.owner.as_ref()], bump)] pub stake: Account<'info, Stake>,
    pub agent: Option<Account<'info, Agent>>,
}

// ---- Saf hesap fonksiyonlari (zincir ve hesap bagimsiz; birim testleri asagida) ----
pub mod math {
    pub const MONTH: i64 = 30 * 86_400;
    pub const YEAR: i64 = 365 * 86_400;
    pub const SCALE: u128 = 1_000_000_000_000;

    // Ajan teminati: gunluk islem limitinin bps'i, alt sinir min.
    pub fn required_bond(daily_limit: u64, bps: u64, min: u64) -> Option<u64> {
        let v = (daily_limit as u128).checked_mul(bps as u128)? / 10_000;
        let v = u64::try_from(v).ok()?;
        Some(v.max(min))
    }

    // Kesinti: severity 0 uyari (yalniz ilk hata, strikes==0), 1 kucuk (%10), 2 ciddi (%100).
    // Donus: (kesilen, yeni strikes)
    pub fn slash_amount(bond: u64, severity: u8, strikes: u8) -> Option<(u64, u8)> {
        match severity {
            0 => if strikes == 0 { Some((0, 1)) } else { slash_amount(bond, 1, strikes) },
            1 => Some((((bond as u128) * 1_000 / 10_000) as u64, strikes.saturating_add(1))),
            2 => Some((bond, strikes.saturating_add(1))),
            _ => None,
        }
    }

    // Kutu merdiveni alimi. Bir islemde en fazla max_bins kutu doldurulur (kismi dolum olabilir).
    // Donus: (maliyet, yeni_fiyat, yeni_bin_sold, doldurulan_dk, doldurulan_kutu)
    pub fn ladder_buy(price: u64, bin_size: u64, bin_sold: u64, step_bps: u64, dk: u64, max_bins: u64)
        -> Option<(u64, u64, u64, u64, u64)> {
        if bin_size == 0 || bin_sold >= bin_size || price == 0 { return None; }
        let (mut p, mut sold, mut rem) = (price, bin_sold, dk);
        let (mut cost, mut filled, mut bins) = (0u128, 0u64, 0u64);
        while rem > 0 && bins < max_bins {
            let room = bin_size - sold;
            let take = rem.min(room);
            cost = cost.checked_add((take as u128).checked_mul(p as u128)?)?;
            filled = filled.checked_add(take)?;
            sold += take; rem -= take;
            if sold == bin_size {
                let np = ((p as u128) * (10_000 + step_bps) as u128 / 10_000) as u64;
                p = if step_bps > 0 { np.max(p + 1) } else { p };
                sold = 0; bins += 1;
            }
        }
        Some((u64::try_from(cost).ok()?, p, sold, filled, bins))
    }

    // Kilitli likidite: NAV'a gore pay. nav = havuzdaki ana para.
    pub fn shares_for(amount: u64, total_shares: u64, nav: u64) -> Option<u64> {
        if total_shares == 0 || nav == 0 { return Some(amount); }
        u64::try_from((amount as u128).checked_mul(total_shares as u128)? / nav as u128).ok()
    }
    pub fn value_for(shares: u64, total_shares: u64, nav: u64) -> Option<u64> {
        if total_shares == 0 { return None; }
        u64::try_from((shares as u128).checked_mul(nav as u128)? / total_shares as u128).ok()
    }
    // Erken donem ucret carpani (bps): ilk early_secs boyunca early_mult, sonraki early_secs boyunca 10000'e dogrusal iner.
    pub fn mult_bps(now: i64, start: i64, early_secs: i64, early_mult: u64) -> u64 {
        if early_secs <= 0 || early_mult <= 10_000 { return 10_000; }
        let t = now - start;
        if t <= early_secs { early_mult }
        else if t >= 2 * early_secs { 10_000 }
        else {
            let extra = (early_mult - 10_000) as i128;
            let left = (2 * early_secs - t) as i128;
            (10_000 + extra * left / early_secs as i128) as u64
        }
    }
    // Komisyon: (komisyon, kalan)
    pub fn split_commission(amount: u64, bps: u64) -> (u64, u64) {
        let fee = ((amount as u128) * bps as u128 / 10_000) as u64;
        (fee, amount - fee)
    }
    // Kilit bittikten sonra ayda init_shares * exit_bps kadar cikis hakki birikir (ilk ay kilit bitince acilir).
    pub fn allowed_exit(init_shares: u64, exit_bps: u64, lock_until: i64, now: i64) -> u64 {
        if now < lock_until { return 0; }
        let months = ((now - lock_until) / MONTH + 1) as u128;
        let v = (init_shares as u128) * exit_bps as u128 * months / 10_000;
        v.min(init_shares as u128) as u64
    }
    pub fn pending_fee(weight: u128, acc: u128, debt: u128) -> u64 {
        let gross = weight.saturating_mul(acc) / SCALE;
        gross.saturating_sub(debt) as u64
    }

    // Staking: tavan oraninda yillik odul; ajan bonusu yalniz agent_cap'e kadar olan kisma, tavan oraninin bonus_bps'i kadar eklenir.
    pub fn stake_accrue(amount: u64, agent_cap: u64, cap_bps: u64, bonus_bps: u64, dt: i64) -> u64 {
        if dt <= 0 { return 0; }
        let base = (amount as u128) * cap_bps as u128 * dt as u128 / (YEAR as u128 * 10_000);
        let elig = amount.min(agent_cap) as u128;
        let bonus = elig * cap_bps as u128 * bonus_bps as u128 * dt as u128 / (YEAR as u128 * 10_000 * 10_000);
        (base + bonus).min(u64::MAX as u128) as u64
    }
    pub const YEAR_BUDGET: [u64; 5] = [75_000_000, 62_500_000, 50_000_000, 37_500_000, 25_000_000];
    pub fn year_index(now: i64, start: i64) -> Option<usize> {
        if now < start { return None; }
        let y = ((now - start) / YEAR) as usize;
        if y < 5 { Some(y) } else { None }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test] fn bond() {
            assert_eq!(required_bond(1_000_000, 500, 50_000), Some(50_000));
            assert_eq!(required_bond(100_000_000, 500, 50_000), Some(5_000_000));
            assert_eq!(required_bond(u64::MAX, 10_000, 1), Some(u64::MAX));
        }
        #[test] fn slash() {
            assert_eq!(slash_amount(1000, 0, 0), Some((0, 1)));      // ilk kucuk teknik hata: uyari
            assert_eq!(slash_amount(1000, 0, 1), Some((100, 2)));    // tekrar: %10
            assert_eq!(slash_amount(1000, 1, 0), Some((100, 1)));
            assert_eq!(slash_amount(1000, 2, 0), Some((1000, 1)));
            assert_eq!(slash_amount(1000, 9, 0), None);
        }
        #[test] fn ladder() {
            // 50 kutu, kutu 1M DK, fiyat 1000, adim %3.3: tek islemde en fazla 5 kutu
            let (cost, p, sold, filled, bins) = ladder_buy(1000, 1_000_000, 0, 330, 10_000_000, 5).unwrap();
            assert_eq!(filled, 5_000_000); assert_eq!(bins, 5); assert_eq!(sold, 0);
            assert!(p > 1000 && p < 1200);
            assert!(cost > 5_000_000_000 && cost < 6_000_000_000);
            // kutu icinde fiyat sabit
            let r = ladder_buy(1000, 1_000_000, 0, 330, 1_000, 5).unwrap();
            assert_eq!(r.0, 1_000_000); assert_eq!(r.1, 1000); assert_eq!(r.2, 1000);
            // 50 kutu sonunda fiyat ~5 kat
            let mut p = 1000u64; let mut s = 0u64;
            for _ in 0..50 { let r = ladder_buy(p, 1_000_000, s, 330, 1_000_000, 5).unwrap(); p = r.1; s = r.2; }
            assert!(p > 4500 && p < 5500, "{}", p);
            // adim 0: fiyat sabit
            assert_eq!(ladder_buy(1000, 10, 0, 0, 10, 5).unwrap().1, 1000);
            // gecersiz
            assert!(ladder_buy(0, 10, 0, 10, 5, 5).is_none());
        }
        #[test] fn lp() {
            assert_eq!(shares_for(100, 0, 0), Some(100));
            assert_eq!(shares_for(150, 100, 150), Some(100)); // NAV 1.5 -> ayni para daha az pay
            assert_eq!(value_for(100, 200, 300), Some(150));
            assert_eq!(split_commission(1000, 700), (70, 930));
            assert_eq!(mult_bps(0, 0, 100, 15_000), 15_000);
            assert_eq!(mult_bps(100, 0, 100, 15_000), 15_000);
            assert_eq!(mult_bps(150, 0, 100, 15_000), 12_500);
            assert_eq!(mult_bps(200, 0, 100, 15_000), 10_000);
            assert_eq!(mult_bps(999, 0, 100, 15_000), 10_000);
            assert_eq!(allowed_exit(1000, 200, 1000, 999), 0);          // kilit suruyor
            assert_eq!(allowed_exit(1000, 200, 1000, 1000), 20);        // ilk ay %2
            assert_eq!(allowed_exit(1000, 200, 1000, 1000 + MONTH), 40);
            assert_eq!(allowed_exit(1000, 200, 1000, 1000 + 100 * MONTH), 1000); // tavan
            assert_eq!(pending_fee(100, 2 * SCALE, 50), 150);
        }
        #[test] fn stake() {
            // 1M stake, %20 tavan, 1 yil: 200K
            assert_eq!(stake_accrue(1_000_000, 100_000, 2000, 0, YEAR), 200_000);
            // ajan bonusu %50: yalniz 100K'ya kadar kisma 20%*50% = ek 10K
            assert_eq!(stake_accrue(1_000_000, 100_000, 2000, 5000, YEAR), 210_000);
            // tum stake uygunsa %30
            assert_eq!(stake_accrue(100_000, 100_000, 2000, 5000, YEAR), 30_000);
            assert_eq!(stake_accrue(5, 5, 2000, 5000, -1), 0);
            assert_eq!(year_index(0, 0), Some(0));
            assert_eq!(year_index(YEAR, 0), Some(1));
            assert_eq!(year_index(5 * YEAR, 0), None);
            assert_eq!(year_index(-1, 0), None);
        }
    }
}
