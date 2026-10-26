// DRAFT v3: gercek SPL token (1B sabit arz), adil on satis (10K DK/cuzdan, esit dagitim), gercek kutu havuzu (alim, geri satis, gunluk tavan, tur+orantili dolum), komite oylamasi.
// Rust 'math' modulu birim testlerden gecti (rustc --test). Anchor ile DERLENMEDI; ilk derlemede hata cikabilir.
// Not compiled yet in Playground, not tested on a network, not audited. Do not deploy with real funds.
// DKEEY ön satış emaneti — DEVNET DENEME SÜRÜMÜ (denetlenmedi, yalnızca test)
// Solana Playground (Anchor 0.29+) içinde derlenir. Para birimi: lamport (SOL).
// Mantık presale_ref.py ile aynı: haftalık devir (rollover) tavanı, kısmi dolum, eşik, çekilebilir iade.
// Test için hafta süresi parametredir (ör. 60 sn).
// Treasury (rezerv çekim tavanı) bu programın içindedir: rezerv PDA'sını yalnız bu program imzalayabilir.
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{self, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer as SplTransfer};
use anchor_spl::token::spl_token::instruction::AuthorityType;

pub const NORMAL_BPS: u64 = 50;
pub const CRISIS_BPS: u64 = 100;

declare_id!("11111111111111111111111111111111"); // Playground derlerken kendi ID'sini yazar

#[program]
pub mod dkeey_presale {
    use super::*;

    // rates = FIYAT: 1 DK basina lamport (hafta 1, 2, 3). Istek lamport olarak emanete alinir, dagitim hafta kapaninca hesaplanir.
    pub fn initialize(ctx: Context<Initialize>, id: u64, start: i64, week_secs: i64, cap: u64, min: u64, rates: [u64; 3]) -> Result<()> {
        require!(week_secs > 0 && cap > 0 && min > 0 && min <= cap, E::BadParams);
        require!(rates.iter().all(|r| *r > 0 && *r <= 1_000_000_000_000), E::BadParams);
        let s = &mut ctx.accounts.sale;
        s.id = id; s.bump = ctx.bumps.sale; s.authority = ctx.accounts.payer.key();
        s.start = start; s.week_secs = week_secs; s.cap = cap; s.min = min; s.rates = rates;
        s.raised = 0; s.finalized = false; s.success = false; s.released = false; s.finalized_at = 0;
        s.level = [u64::MAX; 3]; s.alloc = [0; 3]; s.settled = [false; 3];
        let mut d = ctx.accounts.demand.load_init()?;
        d.minr = [[u64::MAX; 128]; 3];
        Ok(())
    }

    // Istek: cuzdan basina toplam en fazla 10.000 DK (haftalar toplami). Para emanette; fazlasi hafta sonunda iade edilir.
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &ctx.accounts.sale;
        require!(!s.finalized, E::Closed);
        require!(now >= s.start && now < s.start + 3 * s.week_secs, E::OutsideWindow);
        let wi = (((now - s.start) / s.week_secs) as usize).min(2);
        require!(!s.settled[wi], E::Closed);
        let c = &mut ctx.accounts.contrib;
        let mut used: u128 = 0; // DK karsiligi x SCALE
        for i in 0..3 { used += (c.req[i] as u128) * math::SCALE / s.rates[i] as u128; }
        let capq = (MAX_DK_PER_WALLET as u128) * math::SCALE;
        require!(used < capq, E::NoRoom);
        let take = (amount as u128).min((capq - used) * s.rates[wi] as u128 / math::SCALE) as u64;
        require!(take > 0, E::NoRoom);
        let max_req = MAX_DK_PER_WALLET * s.rates[wi];
        let old = c.req[wi];
        let new = old.checked_add(take).ok_or(E::BadParams)?;
        require!(new <= max_req, E::NoRoom);
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.user.to_account_info(), to: ctx.accounts.sale.to_account_info() }), take)?;
        let mut d = ctx.accounts.demand.load_mut()?;
        if old > 0 {
            let b = math::bucket_of(max_req, old).ok_or(E::BadParams)?;
            d.cnt[wi][b] -= 1; d.sum[wi][b] -= old;
        }
        let b = math::bucket_of(max_req, new).ok_or(E::BadParams)?;
        d.cnt[wi][b] += 1; d.sum[wi][b] += new;
        if new < d.minr[wi][b] { d.minr[wi][b] = new; }
        c.req[wi] = new;
        Ok(())
    }

    // Hafta kapaninca herkes cagirabilir: o haftanin oda'si (rollover dahil) adil dagitilir. Sirayla (once hafta 0).
    pub fn settle_week(ctx: Context<SettleWeek>, week: u8) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &mut ctx.accounts.sale;
        let w = week as usize;
        require!(w < 3 && !s.settled[w], E::BadParams);
        require!(w == 0 || s.settled[w - 1], E::TooEarly);
        require!(now >= s.start + (w as i64 + 1) * s.week_secs, E::TooEarly);
        let cum = ((w as u128 + 1) * s.cap as u128 / 3) as u64;
        let room = cum.saturating_sub(s.raised).min(s.cap - s.raised);
        let max_req = MAX_DK_PER_WALLET * s.rates[w];
        let d = ctx.accounts.demand.load()?;
        let (level, alloc) = math::fair_level(room, max_req, &d.cnt[w], &d.sum[w], &d.minr[w]);
        s.level[w] = level; s.alloc[w] = alloc; s.raised += alloc; s.settled[w] = true;
        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &mut ctx.accounts.sale;
        require!(!s.finalized, E::Closed);
        require!(s.settled[2], E::TooEarly);
        s.finalized = true;
        s.success = s.raised >= s.min;
        s.finalized_at = now;
        Ok(())
    }

    // Basarisizsa tum istek; basariliysa her haftanin dolmayan fazlasi iade edilir. Birden cok kez cagrilabilir (yalniz borc kadar).
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let s = &ctx.accounts.sale;
        let c = &mut ctx.accounts.contrib;
        require!(s.finalized, E::NoRefund);
        let mut req_total: u128 = 0; let mut alloc_total: u128 = 0;
        for i in 0..3 { req_total += c.req[i] as u128; alloc_total += c.req[i].min(s.level[i]) as u128; }
        let back = if s.success { req_total - alloc_total } else { req_total };
        let due = back.saturating_sub(c.paid_back as u128) as u64;
        require!(due > 0, E::NoRefund);
        c.paid_back += due;
        **s.to_account_info().try_borrow_mut_lamports()? -= due;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += due;
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

    // Token hakki: finalize anindan itibaren 8 hafta dogrusal acilim; GERCEK DK, on satis kasasindan kullanicinin token hesabina.
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let s = &ctx.accounts.sale;
        let c = &mut ctx.accounts.contrib;
        require!(s.finalized && s.success, E::NoClaim);
        let mut total: u128 = 0;
        for i in 0..3 { total += (c.req[i].min(s.level[i]) / s.rates[i]) as u128; }
        let span = (8 * s.week_secs) as i128;
        let el = ((now - s.finalized_at) as i128).max(0).min(span);
        let vested = (total * (el as u128) / (span as u128)) as u64;
        let x = vested.saturating_sub(c.claimed);
        require!(x > 0, E::NoClaim);
        c.claimed += x;
        let sale_key = ctx.accounts.sale.key();
        spl_out(&ctx.accounts.token_program, &ctx.accounts.vault, &ctx.accounts.user_dk, &ctx.accounts.dkauth, sale_key, ctx.accounts.tok.auth_bump, x)?;
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
        c.year_dist = [0; 5]; c.gov_mode = false;
        Ok(())
    }

    pub fn update_cfg(ctx: Context<UpdateCfg>, p: CfgParams) -> Result<()> {
        require!(!ctx.accounts.cfg.gov_mode, E::GovOn);
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

    // ---- 2) Kutu merdiveni (GERCEK havuz): alim ve satim, gercek DK + lamport. Fiyat gecmisi hist[] ile geri satis birebir ters iner. ----
    pub fn init_ladder(ctx: Context<InitLadder>, price: u64, bin_size: u64, supply: u64, daily_cap: u64, round_len: i64) -> Result<()> {
        require!(price > 0 && bin_size > 0 && supply >= bin_size && supply <= CAP_LADDER, E::BadParams);
        require!(supply / bin_size <= 127 && daily_cap > 0, E::BadParams);
        require!(round_len == 0 || round_len >= 60, E::BadParams);
        let l = &mut ctx.accounts.ladder;
        l.cfg = ctx.accounts.cfg.key(); l.hist = [0; 128]; l.hist[0] = price; l.idx = 0;
        l.bin_size = bin_size; l.bin_sold = 0; l.step_bps = 0; l.supply = supply; l.sold_total = 0; l.last_step_ts = 0;
        l.daily_cap = daily_cap; l.day = 0; l.day_sold = 0; l.round_len = round_len; l.round_id = 0; l.round_open = false;
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

    // Anlik alim (round_len == 0 modu). En fazla MAX_BINS_TX kutu; odeme havuza (liq), DK kasadan kullaniciya.
    pub fn buy_ladder(ctx: Context<BuyLadder>, dk: u64, max_cost: u64) -> Result<()> {
        let l = &mut ctx.accounts.ladder;
        require!(l.round_len == 0 && !l.round_open, E::NotActive);
        let want = dk.min(l.supply - l.sold_total);
        require!(want > 0, E::NoRoom);
        let (idx, bs, bsold, step) = (l.idx, l.bin_size, l.bin_sold, l.step_bps);
        let (cost, nidx, nsold, filled) = math::ladder_buy(&mut l.hist, idx, bs, bsold, step, want, MAX_BINS_TX).ok_or(E::BadParams)?;
        require!(filled > 0 && cost <= max_cost, E::Slippage);
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.user.to_account_info(), to: ctx.accounts.liquidity.to_account_info() }), cost)?;
        l.idx = nidx; l.bin_sold = nsold; l.sold_total += filled;
        let sale = ctx.accounts.cfg.sale;
        spl_out(&ctx.accounts.token_program, &ctx.accounts.vault, &ctx.accounts.user_dk, &ctx.accounts.dkauth, sale, ctx.accounts.tok.auth_bump, filled)?;
        Ok(())
    }

    // Geri satis: her kutu kendi alim fiyatindan (spread dusulur), gunluk satis tavani, piyasa ustunde asla.
    pub fn sell_ladder(ctx: Context<SellLadder>, dk: u64, min_out: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let l = &mut ctx.accounts.ladder;
        require!(!l.round_open, E::NotActive);
        let day = now / DAY;
        if l.day != day { l.day = day; l.day_sold = 0; }
        let want = dk.min(l.daily_cap.saturating_sub(l.day_sold));
        require!(want > 0, E::ExitCap);
        let (idx, bs, bsold) = (l.idx, l.bin_size, l.bin_sold);
        let (out, nidx, nsold, filled) = math::ladder_sell(&l.hist, idx, bs, bsold, want, SELL_SPREAD_BPS, MAX_BINS_TX).ok_or(E::BadParams)?;
        require!(filled > 0 && out >= min_out, E::Slippage);
        let avail = ctx.accounts.liquidity.to_account_info().lamports().saturating_sub(FLOOR);
        require!(out <= avail, E::NoBalance);
        l.idx = nidx; l.bin_sold = nsold; l.sold_total -= filled; l.day_sold += filled;
        spl_in(&ctx.accounts.token_program, &ctx.accounts.user_dk, &ctx.accounts.vault, &ctx.accounts.user, filled)?;
        let sale = ctx.accounts.cfg.sale;
        let seeds: &[&[u8]] = &[b"liq", sale.as_ref(), &[ctx.bumps.liquidity]];
        let signer = &[seeds];
        transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.liquidity.to_account_info(), to: ctx.accounts.user.to_account_info() }, signer), out)?;
        Ok(())
    }

    // ---- Tur modu (round_len > 0): talep > kapasite ise herkes ayni oranda dolar ----
    pub fn open_round(ctx: Context<OpenRound>, id: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let lk = ctx.accounts.ladder.key();
        let l = &mut ctx.accounts.ladder;
        require!(l.round_len > 0 && !l.round_open && id == l.round_id + 1, E::NotActive);
        require!(l.sold_total < l.supply, E::NoRoom);
        let r = &mut ctx.accounts.round;
        r.ladder = lk; r.id = id; r.start = now;
        r.price = l.hist[l.idx as usize]; r.demand = 0; r.filled = 0; r.closed = false;
        r.cap = (l.bin_size - l.bin_sold).min(l.supply - l.sold_total);
        l.round_id = id; l.round_open = true;
        Ok(())
    }

    pub fn place_order(ctx: Context<PlaceOrder>, dk: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let l = &ctx.accounts.ladder;
        let r = &mut ctx.accounts.round;
        require!(l.round_open && !r.closed && now < r.start + l.round_len, E::Closed);
        require!(dk > 0, E::BadParams);
        let pay = (dk as u128).checked_mul(r.price as u128).ok_or(E::BadParams)?;
        let pay = u64::try_from(pay).map_err(|_| E::BadParams)?;
        require!(pay >= MIN_ORDER, E::BadParams);
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(),
            Transfer { from: ctx.accounts.user.to_account_info(), to: ctx.accounts.escrow.to_account_info() }), pay)?;
        let o = &mut ctx.accounts.order;
        if o.dk == 0 { o.owner = ctx.accounts.user.key(); o.round = r.key(); }
        o.dk = o.dk.checked_add(dk).ok_or(E::BadParams)?;
        require!(o.dk.checked_mul(r.price).is_some(), E::BadParams);
        r.demand = r.demand.checked_add(dk).ok_or(E::BadParams)?;
        Ok(())
    }

    // Tur bitince herkes kapatabilir: dolum = min(talep, kapasite). Kutu dolduysa fiyat adim kadar artar.
    pub fn close_round(ctx: Context<CloseRound>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let l = &mut ctx.accounts.ladder;
        let r = &mut ctx.accounts.round;
        require!(l.round_open && !r.closed && now >= r.start + l.round_len, E::TooSoon);
        let filled = r.demand.min(r.cap);
        r.filled = filled; r.closed = true;
        l.round_open = false;
        l.bin_sold += filled; l.sold_total += filled;
        if l.bin_sold == l.bin_size && (l.idx as usize) + 1 < 128 {
            let i = l.idx as usize;
            l.hist[i + 1] = math::next_price(l.hist[i], l.step_bps);
            l.idx += 1; l.bin_sold = 0;
        }
        Ok(())
    }

    // Emir sahibi: dolum payini alir (gercek DK), odemeyi havuza yollar, kullanilmayan kismi geri alir; emir hesabi kapanir.
    pub fn settle_order(ctx: Context<SettleOrder>) -> Result<()> {
        let r = &ctx.accounts.round;
        require!(r.closed, E::TooEarly);
        let o = &ctx.accounts.order;
        let fill = math::pro_rata(o.dk, r.filled, r.demand);
        let cost = fill.checked_mul(r.price).ok_or(E::BadParams)?;
        let paid = o.dk.checked_mul(r.price).ok_or(E::BadParams)?;
        let back = paid - cost;
        let lk = ctx.accounts.ladder.key();
        let seeds: &[&[u8]] = &[b"oesc", lk.as_ref(), &[ctx.bumps.escrow]];
        let signer = &[seeds];
        if cost > 0 {
            transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
                Transfer { from: ctx.accounts.escrow.to_account_info(), to: ctx.accounts.liquidity.to_account_info() }, signer), cost)?;
        }
        if back > 0 {
            transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
                Transfer { from: ctx.accounts.escrow.to_account_info(), to: ctx.accounts.user.to_account_info() }, signer), back)?;
        }
        if fill > 0 {
            let sale = ctx.accounts.cfg.sale;
            spl_out(&ctx.accounts.token_program, &ctx.accounts.vault, &ctx.accounts.user_dk, &ctx.accounts.dkauth, sale, ctx.accounts.tok.auth_bump, fill)?;
        }
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

    // ---- 4) Staking tavani (GERCEK DK): yillik odul = stake x tavan; ajan bonusu tavanin ustune (en fazla +%50), ajan basina sinirli ----
    pub fn stake_dk(ctx: Context<StakeDk>, amount: u64) -> Result<()> {
        require!(amount > 0, E::BadParams);
        let now = Clock::get()?.unix_timestamp;
        spl_in(&ctx.accounts.token_program, &ctx.accounts.user_dk, &ctx.accounts.vstake, &ctx.accounts.owner, amount)?;
        let c = &ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        if s.last_ts == 0 {
            s.owner = ctx.accounts.owner.key(); s.cfg = ctx.accounts.cfg.key(); s.last_ts = now;
        }
        accrue(s, c, now);
        s.amount += amount;
        Ok(())
    }

    pub fn unstake_dk(ctx: Context<UnstakeDk>, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let c = &ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        require!(amount > 0 && amount <= s.amount, E::BadParams);
        accrue(s, c, now);
        s.amount -= amount;
        let sale = ctx.accounts.cfg.sale;
        spl_out(&ctx.accounts.token_program, &ctx.accounts.vstake, &ctx.accounts.user_dk, &ctx.accounts.dkauth, sale, ctx.accounts.tok.auth_bump, amount)?;
        Ok(())
    }

    // Odeme yalniz o yilin butcesinden (75M, 62.5M, 50M, 37.5M, 25M DK) ve odul kasasindaki bakiyeden; butce bitince odeme yok (ilk gelen alir).
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let c = &mut ctx.accounts.cfg;
        let s = &mut ctx.accounts.stake;
        accrue(s, c, now);
        let idx = math::year_index(now, c.start).ok_or(E::Budget)?;
        let avail = math::YEAR_BUDGET[idx].saturating_sub(c.year_dist[idx]).min(ctx.accounts.vreward.amount);
        let pay = s.accrued.min(avail);
        require!(pay > 0, E::Budget);
        s.accrued -= pay; c.year_dist[idx] += pay;
        let sale = c.sale;
        spl_out(&ctx.accounts.token_program, &ctx.accounts.vreward, &ctx.accounts.user_dk, &ctx.accounts.dkauth, sale, ctx.accounts.tok.auth_bump, pay)?;
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

    // ================= GERCEK TOKEN: 1 milyar DK sabit arz, mint yetkisi kalici kapatilir =================
    // DK_DECIMALS = 0 (taslak: fiyatlar tam sayi lamport/DK). Ana agda ondalik secimi yeniden degerlendirilecek.
    pub fn init_token(ctx: Context<InitToken>) -> Result<()> {
        let t = &mut ctx.accounts.tok;
        t.sale = ctx.accounts.sale.key(); t.authority = ctx.accounts.authority.key();
        t.mint = ctx.accounts.mint.key(); t.auth_bump = ctx.bumps.dkauth; t.funded = [0; 3];
        let sale = ctx.accounts.sale.key();
        let seeds: &[&[u8]] = &[b"dkauth", sale.as_ref(), &[ctx.bumps.dkauth]];
        let signer = &[seeds];
        token::mint_to(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),
            MintTo { mint: ctx.accounts.mint.to_account_info(), to: ctx.accounts.master.to_account_info(), authority: ctx.accounts.dkauth.to_account_info() }, signer), TOTAL_SUPPLY)?;
        token::set_authority(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),
            SetAuthority { current_authority: ctx.accounts.dkauth.to_account_info(), account_or_mint: ctx.accounts.mint.to_account_info() }, signer),
            AuthorityType::MintTokens, None)?;
        Ok(())
    }

    // Kasalar (hepsinin sahibi dkauth PDA'si): 1 on satis, 2 merdiven, 3 odul, 4 stake, 5 oy kilidi. Her tur icin bir kez cagrilir.
    pub fn init_vault(_ctx: Context<InitVault>, kind: u8) -> Result<()> {
        require!((1..=5).contains(&kind), E::BadParams);
        Ok(())
    }

    // Ana kasadan hedef kasaya; her hedef icin SERT ust sinir (30M on satis, 100M merdiven, 250M staking odulu).
    pub fn fund_presale(ctx: Context<FundVault>, amount: u64) -> Result<()> { do_fund(ctx, 0, VK_PRESALE, CAP_PRESALE, amount) }
    pub fn fund_ladder(ctx: Context<FundVault>, amount: u64) -> Result<()> { do_fund(ctx, 1, VK_LADDER, CAP_LADDER, amount) }
    pub fn fund_reward(ctx: Context<FundVault>, amount: u64) -> Result<()> { do_fund(ctx, 2, VK_REWARD, CAP_REWARD, amount) }

    // ================= KOMITE: oy gucu = kilitli DK x sure; sahip basina tavan; oneri -> oy -> uygulama =================
    pub fn init_gov(ctx: Context<InitGov>, vote_secs: i64, quorum_bps: u64, cap_bps: u64) -> Result<()> {
        require!((60..=30 * 86_400).contains(&vote_secs), E::OutOfBounds);
        require!((500..=5_000).contains(&quorum_bps) && (100..=1_000).contains(&cap_bps), E::OutOfBounds);
        let g = &mut ctx.accounts.gov;
        g.cfg = ctx.accounts.cfg.key(); g.total_weight = 0; g.next_id = 1;
        g.vote_secs = vote_secs; g.quorum_bps = quorum_bps; g.cap_bps = cap_bps;
        Ok(())
    }

    // Tek yonlu: bundan sonra update_cfg kapanir, ayarlar yalniz kabul edilmis oneriyle degisir.
    pub fn enable_gov(ctx: Context<EnableGov>) -> Result<()> {
        ctx.accounts.cfg.gov_mode = true;
        Ok(())
    }

    pub fn lock_votes(ctx: Context<LockVotes>, amount: u64, secs: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(amount > 0 && (30 * 86_400..=VOTE_MAX_LOCK).contains(&secs), E::OutOfBounds);
        spl_in(&ctx.accounts.token_program, &ctx.accounts.user_dk, &ctx.accounts.vvote, &ctx.accounts.owner, amount)?;
        let w = math::vote_weight(amount, secs);
        let v = &mut ctx.accounts.lock;
        v.owner = ctx.accounts.owner.key(); v.cfg = ctx.accounts.cfg.key(); v.amount = amount;
        v.created = now; v.lock_until = now + secs; v.weight = w;
        ctx.accounts.gov.total_weight += w;
        Ok(())
    }

    pub fn unlock_votes(ctx: Context<UnlockVotes>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let v = &ctx.accounts.lock;
        require!(now >= v.lock_until, E::Locked);
        let (amount, w) = (v.amount, v.weight);
        ctx.accounts.gov.total_weight = ctx.accounts.gov.total_weight.saturating_sub(w);
        let sale = ctx.accounts.cfg.sale;
        spl_out(&ctx.accounts.token_program, &ctx.accounts.vvote, &ctx.accounts.user_dk, &ctx.accounts.dkauth, sale, ctx.accounts.tok.auth_bump, amount)?;
        Ok(())
    }

    // Oneri: yeni ayar seti. Onerici toplam agirligin en az %1'ine sahip ve kilidi oylama sonuna kadar surmeli.
    pub fn propose(ctx: Context<Propose>, p: CfgParams) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(ctx.accounts.cfg.gov_mode, E::NotActive);
        check_params(&p)?;
        let g = &mut ctx.accounts.gov;
        let v = &ctx.accounts.lock;
        require!(v.weight * 10_000 >= g.total_weight * PROPOSE_MIN_BPS as u128 && g.total_weight > 0, E::NoWeight);
        require!(v.lock_until >= now + g.vote_secs, E::Locked);
        let pr = &mut ctx.accounts.proposal;
        pr.cfg = ctx.accounts.cfg.key(); pr.id = g.next_id; pr.p = p;
        pr.start = now; pr.end = now + g.vote_secs; pr.yes = 0; pr.no = 0; pr.snap_total = g.total_weight; pr.executed = false;
        g.next_id += 1;
        Ok(())
    }

    // Oy: kilit oneriden ONCE olusturulmus ve oylama sonuna kadar kilitli olmali (token tasiyip iki kez oy vermek olmaz).
    pub fn vote(ctx: Context<Vote>, approve: bool) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let pr = &mut ctx.accounts.proposal;
        let v = &ctx.accounts.lock;
        require!(now < pr.end && !pr.executed, E::VoteClosed);
        require!(v.created <= pr.start && v.lock_until >= pr.end, E::Locked);
        let w = math::eff_weight(v.weight, pr.snap_total, ctx.accounts.gov.cap_bps);
        require!(w > 0, E::NoWeight);
        if approve { pr.yes += w; } else { pr.no += w; }
        ctx.accounts.rec.voted = true;
        Ok(())
    }

    pub fn execute(ctx: Context<Execute>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let pr = &mut ctx.accounts.proposal;
        require!(ctx.accounts.cfg.gov_mode, E::NotActive);
        require!(now >= pr.end && !pr.executed, E::VoteOpen);
        require!(math::passes(pr.yes, pr.no, pr.snap_total, ctx.accounts.gov.quorum_bps), E::Rejected);
        check_params(&pr.p)?;
        apply_params(&mut ctx.accounts.cfg, &pr.p);
        pr.executed = true;
        Ok(())
    }

}

#[account]
pub struct Sale { pub id: u64, pub bump: u8, pub start: i64, pub week_secs: i64, pub cap: u64, pub min: u64, pub rates: [u64; 3], pub raised: u64, pub finalized: bool, pub success: bool, pub released: bool, pub finalized_at: i64, pub authority: Pubkey,
    pub level: [u64; 3], pub alloc: [u64; 3], pub settled: [bool; 3] }
#[account]
pub struct Contrib { pub req: [u64; 3], pub paid_back: u64, pub claimed: u64 }

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = SALE_SPACE, seeds = [b"sale", id.to_le_bytes().as_ref()], bump)]
    pub sale: Box<Account<'info, Sale>>,
    #[account(init, payer = payer, space = 8 + std::mem::size_of::<Demand>(), seeds = [b"demand", sale.key().as_ref()], bump)]
    pub demand: AccountLoader<'info, Demand>,
    #[account(mut)] pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(mut, seeds = [b"demand", sale.key().as_ref()], bump)] pub demand: AccountLoader<'info, Demand>,
    #[account(init_if_needed, payer = user, space = 8 + 24 + 8 + 8, seeds = [b"c", sale.key().as_ref(), user.key().as_ref()], bump)]
    pub contrib: Account<'info, Contrib>,
    #[account(mut)] pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
}
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(mut, seeds = [b"c", sale.key().as_ref(), user.key().as_ref()], bump)] pub contrib: Account<'info, Contrib>,
    #[account(mut)] pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Release<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(mut, seeds = [b"reserve", sale.key().as_ref()], bump)] pub reserve: SystemAccount<'info>,
    #[account(mut, seeds = [b"liq", sale.key().as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    #[account(mut)] pub caller: Signer<'info>,
}
#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(mut, seeds = [b"c", sale.key().as_ref(), user.key().as_ref()], bump)] pub contrib: Account<'info, Contrib>,
    #[account(seeds = [b"tok", sale.key().as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", sale.key().as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vault", sale.key().as_ref(), VK_PRESALE.to_le_bytes().as_ref()], bump)] pub vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = user)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(mut)] pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Treasury { pub sale: Pubkey, pub authority: Pubkey, pub start: i64, pub epoch_secs: i64, pub epoch: u64, pub snapshot: u64, pub withdrawn: u64, pub crisis: bool, pub reserve_bump: u8 }

#[derive(Accounts)]
pub struct TrInit<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
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
  #[msg("no weight")] NoWeight, #[msg("too soon")] TooSoon, #[msg("no fee")] NoFee,
  #[msg("vote closed")] VoteClosed, #[msg("vote still open")] VoteOpen, #[msg("proposal rejected")] Rejected, #[msg("governance is on")] GovOn }

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
    pub stake_cap_bps: u64, pub agent_stake_cap: u64, pub lpv_bump: u8, pub lfee_bump: u8, pub year_dist: [u64; 5], pub gov_mode: bool }
#[account]
pub struct Agent { pub cfg: Pubkey, pub owner: Pubkey, pub bond: u64, pub daily_limit: u64, pub status: u8, pub exit_at: i64,
    pub strikes: u8, pub score_bps: u64, pub slashed: u64 }
#[account]
pub struct Ladder { pub cfg: Pubkey, pub hist: [u64; 128], pub idx: u64, pub bin_size: u64, pub bin_sold: u64, pub step_bps: u64,
    pub supply: u64, pub sold_total: u64, pub last_step_ts: i64, pub daily_cap: u64, pub day: i64, pub day_sold: u64,
    pub round_len: i64, pub round_id: u64, pub round_open: bool }
#[account]
pub struct LpPool { pub cfg: Pubkey, pub total_shares: u64, pub total_weight: u128, pub acc: u128 }
#[account]
pub struct LpPos { pub owner: Pubkey, pub shares: u64, pub init_shares: u64, pub withdrawn: u64, pub weight: u128, pub mult: u64,
    pub lock_until: i64, pub debt: u128, pub owed: u64 }
#[account]
pub struct Stake { pub owner: Pubkey, pub cfg: Pubkey, pub amount: u64, pub last_ts: i64, pub accrued: u64, pub bonus_bps: u64 }

#[derive(Accounts)]
pub struct InitCfg<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(init, payer = authority, space = 8 + 32 + 32 + 8*11 + 1 + 1 + 40 + 8, seeds = [b"cfg", sale.key().as_ref()], bump)]
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
    #[account(init, payer = authority, space = LADDER_SPACE, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(mut)] pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SetStep<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct BuyLadder<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", cfg.sale.as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_LADDER.to_le_bytes().as_ref()], bump)] pub vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = user)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"liq", cfg.sale.as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    #[account(mut)] pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
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
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_STAKE.to_le_bytes().as_ref()], bump)] pub vstake: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = owner)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(mut)] pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UnstakeDk<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"stake", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub stake: Account<'info, Stake>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", cfg.sale.as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_STAKE.to_le_bytes().as_ref()], bump)] pub vstake: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = owner)] pub user_dk: Box<Account<'info, TokenAccount>>,
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)] pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"stake", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub stake: Account<'info, Stake>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", cfg.sale.as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_REWARD.to_le_bytes().as_ref()], bump)] pub vreward: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = owner)] pub user_dk: Box<Account<'info, TokenAccount>>,
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct SyncBonus<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"stake", cfg.key().as_ref(), stake.owner.as_ref()], bump)] pub stake: Account<'info, Stake>,
    pub agent: Option<Account<'info, Agent>>,
}

// ================= v3: gercek token, adil on satis, gercek havuz, komite =================
pub const MAX_DK_PER_WALLET: u64 = 10_000;
pub const DK_DECIMALS: u8 = 0;
pub const TOTAL_SUPPLY: u64 = 1_000_000_000;
pub const CAP_PRESALE: u64 = 30_000_000;
pub const CAP_LADDER: u64 = 100_000_000;
pub const CAP_REWARD: u64 = 250_000_000;
pub const SELL_SPREAD_BPS: u64 = 50;
pub const DAY: i64 = 86_400;
pub const MIN_ORDER: u64 = 1_000_000;
pub const PROPOSE_MIN_BPS: u64 = 100;
pub const VOTE_MAX_LOCK: i64 = 4 * 365 * 86_400;
pub const VK_PRESALE: u8 = 1;
pub const VK_LADDER: u8 = 2;
pub const VK_REWARD: u8 = 3;
pub const VK_STAKE: u8 = 4;
pub const VK_VOTE: u8 = 5;
pub const SALE_SPACE: usize = 175;
pub const LADDER_SPACE: usize = 8 + 32 + 1024 + 96 + 1;

// Gercek SPL transferleri. Kasalarin sahibi dkauth PDA'sidir; cikis dkauth imzasiyla, giris kullanici imzasiyla.
fn spl_out<'info>(tp: &Program<'info, Token>, from: &Account<'info, TokenAccount>, to: &Account<'info, TokenAccount>,
    auth: &SystemAccount<'info>, sale: Pubkey, bump: u8, amount: u64) -> Result<()> {
    let seeds: &[&[u8]] = &[b"dkauth", sale.as_ref(), &[bump]];
    let signer = &[seeds];
    token::transfer(CpiContext::new_with_signer(tp.to_account_info(),
        SplTransfer { from: from.to_account_info(), to: to.to_account_info(), authority: auth.to_account_info() }, signer), amount)
}
fn spl_in<'info>(tp: &Program<'info, Token>, from: &Account<'info, TokenAccount>, to: &Account<'info, TokenAccount>,
    owner: &Signer<'info>, amount: u64) -> Result<()> {
    token::transfer(CpiContext::new(tp.to_account_info(),
        SplTransfer { from: from.to_account_info(), to: to.to_account_info(), authority: owner.to_account_info() }), amount)
}
fn do_fund(ctx: Context<FundVault>, idx: usize, kind: u8, cap: u64, amount: u64) -> Result<()> {
    require!(amount > 0, E::BadParams);
    let sale = ctx.accounts.tok.sale;
    let expect = Pubkey::find_program_address(&[b"vault", sale.as_ref(), &[kind]], ctx.program_id).0;
    require_keys_eq!(ctx.accounts.dest.key(), expect, E::BadParams);
    let t = &mut ctx.accounts.tok;
    let nf = t.funded[idx].checked_add(amount).ok_or(E::BadParams)?;
    require!(nf <= cap, E::OutOfBounds);
    t.funded[idx] = nf;
    let bump = t.auth_bump;
    spl_out(&ctx.accounts.token_program, &ctx.accounts.master, &ctx.accounts.dest, &ctx.accounts.dkauth, sale, bump, amount)
}

#[account(zero_copy)]
pub struct Demand { pub sum: [[u64; 128]; 3], pub minr: [[u64; 128]; 3], pub cnt: [[u32; 128]; 3] }
#[account]
pub struct TokState { pub sale: Pubkey, pub authority: Pubkey, pub mint: Pubkey, pub auth_bump: u8, pub funded: [u64; 3] }
#[account]
pub struct Round { pub ladder: Pubkey, pub id: u64, pub price: u64, pub start: i64, pub demand: u64, pub filled: u64, pub cap: u64, pub closed: bool }
#[account]
pub struct Order { pub owner: Pubkey, pub round: Pubkey, pub dk: u64 }
#[account]
pub struct Gov { pub cfg: Pubkey, pub total_weight: u128, pub next_id: u64, pub vote_secs: i64, pub quorum_bps: u64, pub cap_bps: u64 }
#[account]
pub struct VoteLock { pub owner: Pubkey, pub cfg: Pubkey, pub amount: u64, pub created: i64, pub lock_until: i64, pub weight: u128 }
#[account]
pub struct Proposal { pub cfg: Pubkey, pub id: u64, pub p: CfgParams, pub start: i64, pub end: i64, pub yes: u128, pub no: u128, pub snap_total: u128, pub executed: bool }
#[account]
pub struct VoteRec { pub voted: bool }

#[derive(Accounts)]
pub struct SettleWeek<'info> {
    #[account(mut, seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(seeds = [b"demand", sale.key().as_ref()], bump)] pub demand: AccountLoader<'info, Demand>,
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitToken<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(init, payer = authority, space = 8 + 32 + 32 + 32 + 1 + 24, seeds = [b"tok", sale.key().as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", sale.key().as_ref()], bump)] pub dkauth: SystemAccount<'info>,
    #[account(init, payer = authority, seeds = [b"dkmint", sale.key().as_ref()], bump, mint::decimals = DK_DECIMALS, mint::authority = dkauth)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(init, payer = authority, seeds = [b"vmaster", sale.key().as_ref()], bump, token::mint = mint, token::authority = dkauth)]
    pub master: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = sale.authority)] pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
#[instruction(kind: u8)]
pub struct InitVault<'info> {
    #[account(seeds = [b"sale", sale.id.to_le_bytes().as_ref()], bump = sale.bump)] pub sale: Box<Account<'info, Sale>>,
    #[account(seeds = [b"tok", sale.key().as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", sale.key().as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(address = tok.mint)] pub mint: Box<Account<'info, Mint>>,
    #[account(init, payer = authority, seeds = [b"vault", sale.key().as_ref(), kind.to_le_bytes().as_ref()], bump, token::mint = mint, token::authority = dkauth)]
    pub vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = sale.authority)] pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
pub struct FundVault<'info> {
    #[account(mut, has_one = authority)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", tok.sale.as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vmaster", tok.sale.as_ref()], bump)] pub master: Box<Account<'info, TokenAccount>>,
    #[account(mut)] pub dest: Box<Account<'info, TokenAccount>>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct OpenRound<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(init, payer = caller, space = 8 + 32 + 48 + 8 + 1, seeds = [b"round", ladder.key().as_ref(), id.to_le_bytes().as_ref()], bump)] pub round: Account<'info, Round>,
    #[account(mut)] pub caller: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(mut, seeds = [b"round", ladder.key().as_ref(), ladder.round_id.to_le_bytes().as_ref()], bump)] pub round: Account<'info, Round>,
    #[account(init_if_needed, payer = user, space = 8 + 32 + 32 + 8, seeds = [b"ord", round.key().as_ref(), user.key().as_ref()], bump)] pub order: Account<'info, Order>,
    #[account(mut, seeds = [b"oesc", ladder.key().as_ref()], bump)] pub escrow: SystemAccount<'info>,
    #[account(mut)] pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct CloseRound<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(mut, seeds = [b"round", ladder.key().as_ref(), ladder.round_id.to_le_bytes().as_ref()], bump)] pub round: Account<'info, Round>,
    pub caller: Signer<'info>,
}
#[derive(Accounts)]
pub struct SettleOrder<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(seeds = [b"round", ladder.key().as_ref(), round.id.to_le_bytes().as_ref()], bump)] pub round: Account<'info, Round>,
    #[account(mut, close = user, seeds = [b"ord", round.key().as_ref(), user.key().as_ref()], bump)] pub order: Account<'info, Order>,
    #[account(mut, seeds = [b"oesc", ladder.key().as_ref()], bump)] pub escrow: SystemAccount<'info>,
    #[account(mut, seeds = [b"liq", cfg.sale.as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", cfg.sale.as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_LADDER.to_le_bytes().as_ref()], bump)] pub vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = user)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(mut)] pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SellLadder<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"ladder", cfg.key().as_ref()], bump)] pub ladder: Box<Account<'info, Ladder>>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_LADDER.to_le_bytes().as_ref()], bump)] pub vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = user)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"liq", cfg.sale.as_ref()], bump)] pub liquidity: SystemAccount<'info>,
    #[account(mut)] pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitGov<'info> {
    #[account(has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(init, payer = authority, space = 8 + 32 + 16 + 8 * 4, seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    #[account(mut)] pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct EnableGov<'info> {
    #[account(mut, has_one = authority)] pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct LockVotes<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_VOTE.to_le_bytes().as_ref()], bump)] pub vvote: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = owner)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = owner, space = 8 + 32 + 32 + 8 + 8 + 8 + 16, seeds = [b"vl", cfg.key().as_ref(), owner.key().as_ref()], bump)] pub lock: Account<'info, VoteLock>,
    #[account(mut)] pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UnlockVotes<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    #[account(seeds = [b"tok", cfg.sale.as_ref()], bump)] pub tok: Account<'info, TokState>,
    #[account(seeds = [b"dkauth", cfg.sale.as_ref()], bump = tok.auth_bump)] pub dkauth: SystemAccount<'info>,
    #[account(mut, seeds = [b"vault", cfg.sale.as_ref(), VK_VOTE.to_le_bytes().as_ref()], bump)] pub vvote: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = tok.mint, token::authority = owner)] pub user_dk: Box<Account<'info, TokenAccount>>,
    #[account(mut, close = owner, seeds = [b"vl", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub lock: Account<'info, VoteLock>,
    #[account(mut)] pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct Propose<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(mut, seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    #[account(seeds = [b"vl", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub lock: Account<'info, VoteLock>,
    #[account(init, payer = owner, space = 8 + 32 + 8 + 80 + 16 + 48 + 1, seeds = [b"prop", cfg.key().as_ref(), gov.next_id.to_le_bytes().as_ref()], bump)] pub proposal: Account<'info, Proposal>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Vote<'info> {
    pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    #[account(mut, seeds = [b"prop", cfg.key().as_ref(), proposal.id.to_le_bytes().as_ref()], bump)] pub proposal: Account<'info, Proposal>,
    #[account(seeds = [b"vl", cfg.key().as_ref(), owner.key().as_ref()], bump, has_one = owner)] pub lock: Account<'info, VoteLock>,
    #[account(init, payer = owner, space = 8 + 1, seeds = [b"vr", proposal.key().as_ref(), owner.key().as_ref()], bump)] pub rec: Account<'info, VoteRec>,
    #[account(mut)] pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(mut)] pub cfg: Account<'info, Cfg>,
    #[account(seeds = [b"gov", cfg.key().as_ref()], bump)] pub gov: Account<'info, Gov>,
    #[account(mut, seeds = [b"prop", cfg.key().as_ref(), proposal.id.to_le_bytes().as_ref()], bump)] pub proposal: Account<'info, Proposal>,
    pub caller: Signer<'info>,
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

    // ---- Kutu merdiveni (fiyat gecmisi hist[i] = i. kutunun fiyati; geri satis tam ayni fiyatlardan geri iner) ----
    pub fn next_price(p: u64, step_bps: u64) -> u64 {
        if step_bps == 0 { return p; }
        let np = ((p as u128) * (10_000 + step_bps) as u128 / 10_000) as u64;
        np.max(p + 1)
    }
    // Alim: en fazla max_bins kutu. Donus: (maliyet, yeni_idx, yeni_bin_sold, doldurulan_dk)
    pub fn ladder_buy(hist: &mut [u64], idx: u64, bin_size: u64, bin_sold: u64, step_bps: u64, dk: u64, max_bins: u64)
        -> Option<(u64, u64, u64, u64)> {
        if bin_size == 0 || bin_sold >= bin_size || (idx as usize) >= hist.len() || hist[idx as usize] == 0 { return None; }
        let (mut i, mut sold, mut rem) = (idx, bin_sold, dk);
        let (mut cost, mut filled, mut bins) = (0u128, 0u64, 0u64);
        while rem > 0 && bins < max_bins {
            let p = hist[i as usize];
            let take = rem.min(bin_size - sold);
            cost = cost.checked_add((take as u128).checked_mul(p as u128)?)?;
            filled = filled.checked_add(take)?; sold += take; rem -= take;
            if sold == bin_size {
                if (i as usize) + 1 >= hist.len() { break; }
                hist[i as usize + 1] = next_price(p, step_bps);
                i += 1; sold = 0; bins += 1;
            }
        }
        Some((u64::try_from(cost).ok()?, i, sold, filled))
    }
    // Geri satis: asla piyasa fiyatinin ustunde degil (her kutu kendi alim fiyatindan, spread dusulur). Donus: (gelir, yeni_idx, yeni_bin_sold, satilan_dk)
    pub fn ladder_sell(hist: &[u64], idx: u64, bin_size: u64, bin_sold: u64, dk: u64, spread_bps: u64, max_bins: u64)
        -> Option<(u64, u64, u64, u64)> {
        if bin_size == 0 || spread_bps > 10_000 || (idx as usize) >= hist.len() { return None; }
        let (mut i, mut sold, mut rem) = (idx, bin_sold, dk);
        let (mut out, mut filled, mut bins) = (0u128, 0u64, 0u64);
        while rem > 0 {
            if sold == 0 {
                if i == 0 || bins >= max_bins { break; }
                i -= 1; sold = bin_size; bins += 1; continue;
            }
            let take = rem.min(sold);
            let v = (take as u128).checked_mul(hist[i as usize] as u128)?.checked_mul((10_000 - spread_bps) as u128)? / 10_000;
            out = out.checked_add(v)?;
            sold -= take; rem -= take; filled += take;
        }
        Some((u64::try_from(out).ok()?, i, sold, filled))
    }
    // Tur (orantili dolum): talep > kapasite ise herkes ayni oranda dolar (asagi yuvarlanir, toplam <= filled).
    pub fn pro_rata(dk: u64, filled: u64, demand: u64) -> u64 {
        if demand == 0 { 0 } else { ((dk as u128) * filled as u128 / demand as u128) as u64 }
    }

    // ---- On satis: adil (esit) dagitim. Istek buyuklugune gore 128 kova (ust sinir = max*(k+1)^2/16384). ----
    pub const BUCKETS: usize = 128;
    pub fn bucket_bound(max_req: u64, k: usize) -> u64 {
        ((max_req as u128) * ((k as u128 + 1) * (k as u128 + 1)) / 16_384) as u64
    }
    pub fn bucket_of(max_req: u64, req: u64) -> Option<usize> {
        if req == 0 || req > max_req { return None; }
        for k in 0..BUCKETS { if req <= bucket_bound(max_req, k) { return Some(k); } }
        None
    }
    // Su-doldurma: herkes min(istek, seviye) alir. Once kova sinirinda en yuksek uygun seviye bulunur (toplam TAM bilinir),
    // sonra artan oda, ustte kalan isteklere esit eklenir; ek, ustteki kovalarin (guvenli) en kucuk istegini asmaz,
    // boylece toplam yine TAM bilinir. minr[k] yalniz asagi guncellenir (eski kucuk deger de guvenlidir).
    // Donus: (seviye, dagitilan). Talep <= room ise (u64::MAX, talep).
    pub fn fair_level(room: u64, max_req: u64, cnt: &[u32; 128], sum: &[u64; 128], minr: &[u64; 128]) -> (u64, u64) {
        let total: u128 = sum.iter().map(|x| *x as u128).sum();
        if total <= room as u128 { return (u64::MAX, total as u64); }
        let n_total: u128 = cnt.iter().map(|x| *x as u128).sum();
        let (mut below, mut n_below) = (0u128, 0u128);
        let mut best: (u128, u128, i32) = (0, 0, -1); // (seviye, toplam, kova)
        for k in 0..BUCKETS {
            below += sum[k] as u128; n_below += cnt[k] as u128;
            let l = bucket_bound(max_req, k) as u128;
            let f = below + l * (n_total - n_below);
            if f <= room as u128 { best = (l, f, k as i32); } else { break; }
        }
        let (l, f, kb) = best;
        let n_above: u128 = cnt.iter().enumerate().filter(|(j, _)| (*j as i32) > kb).map(|(_, c)| *c as u128).sum();
        if n_above == 0 { return (l as u64, f as u64); }
        let min_above: u128 = (0..BUCKETS).filter(|j| (*j as i32) > kb && cnt[*j] > 0).map(|j| minr[j] as u128).min().unwrap_or(l);
        let extra = ((room as u128 - f) / n_above).min(min_above.saturating_sub(l));
        ((l + extra) as u64, (f + extra * n_above) as u64)
    }

    // ---- Komite oylamasi ----
    pub fn vote_weight(amount: u64, secs: i64) -> u128 { (amount as u128) * (secs.max(0) as u128) / 86_400 }
    pub fn eff_weight(w: u128, snap_total: u128, cap_bps: u64) -> u128 { w.min(snap_total * cap_bps as u128 / 10_000) }
    pub fn passes(yes: u128, no: u128, snap_total: u128, quorum_bps: u64) -> bool {
        yes > no && yes * 10_000 >= snap_total * quorum_bps as u128
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
        fn lcg(s: &mut u64) -> u64 { *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); *s >> 33 }
        #[test] fn ladder() {
            let mut h = [0u64; 128]; h[0] = 1000;
            // 5 kutu bir islemde
            let r = ladder_buy(&mut h, 0, 1_000_000, 0, 330, 10_000_000, 5).unwrap();
            assert_eq!(r.3, 5_000_000); assert_eq!(r.1, 5); assert_eq!(r.2, 0);
            assert!(h[5] > 1000 && h[5] < 1200);
            assert!(r.0 > 5_000_000_000 && r.0 < 6_000_000_000);
            // kutu icinde sabit fiyat
            let mut h = [0u64; 128]; h[0] = 1000;
            let r = ladder_buy(&mut h, 0, 1_000_000, 0, 330, 1_000, 5).unwrap();
            assert_eq!((r.0, r.1, r.2), (1_000_000, 0, 1000));
            // 50 kutu sonunda ~5 kat
            let mut h = [0u64; 128]; h[0] = 1000; let (mut i, mut s) = (0u64, 0u64);
            for _ in 0..50 { let r = ladder_buy(&mut h, i, 1_000_000, s, 330, 1_000_000, 5).unwrap(); i = r.1; s = r.2; }
            assert!(h[i as usize] > 4500 && h[i as usize] < 5500, "{}", h[i as usize]);
            // adim 0: fiyat sabit
            let mut h = [0u64; 128]; h[0] = 1000;
            assert_eq!(ladder_buy(&mut h, 0, 10, 0, 0, 10, 5).unwrap().1, 1);
            assert_eq!(h[1], 1000);
            // gecersiz
            let mut h = [0u64; 128];
            assert!(ladder_buy(&mut h, 0, 10, 0, 10, 5, 5).is_none());
        }
        #[test] fn ladder_sell_roundtrip() {
            let mut h = [0u64; 128]; h[0] = 1000;
            // 3.5 kutu al, hepsini geri sat: gelir <= odenen (spread kadar eksik), asla fazla degil
            let b = ladder_buy(&mut h, 0, 1_000_000, 0, 330, 3_500_000, 8).unwrap();
            let s = ladder_sell(&h, b.1, 1_000_000, b.2, 3_500_000, 50, 8).unwrap();
            assert_eq!(s.3, 3_500_000); assert_eq!((s.1, s.2), (0, 0));
            assert!(s.0 <= b.0, "{} {}", s.0, b.0);
            assert!(s.0 >= b.0 * 99 / 100, "spread cok yuksek");
            // spread 0: tam esit (fiyat gecmisi birebir geri iner)
            let s0 = ladder_sell(&h, b.1, 1_000_000, b.2, 3_500_000, 0, 8).unwrap();
            assert_eq!(s0.0, b.0);
            // kismi satis: kutu icinde
            let p = ladder_sell(&h, b.1, 1_000_000, b.2, 100, 0, 8).unwrap();
            assert_eq!((p.1, p.2, p.3), (b.1, b.2 - 100, 100));
            // sadece baslangicta: satilacak bir sey yok
            let z = ladder_sell(&h, 0, 1_000_000, 0, 10, 50, 8).unwrap();
            assert_eq!((z.0, z.3), (0, 0));
            // max_bins siniri
            let m = ladder_sell(&h, b.1, 1_000_000, b.2, 3_500_000, 0, 1).unwrap();
            assert!(m.3 < 3_500_000);
            // simulasyon: rastgele al-sat, kasa asla negatif, sold_total == idx*bin+sold
            let mut h = [0u64; 128]; h[0] = 1000; let (mut i, mut so) = (0u64, 0u64);
            let (mut paid, mut got) = (0u128, 0u128); let mut seed = 7u64; let mut tot: u64 = 0;
            for _ in 0..4000 {
                let amt = lcg(&mut seed) % 2_500_000 + 1;
                if lcg(&mut seed) % 2 == 0 {
                    if let Some(r) = ladder_buy(&mut h, i, 1_000_000, so, 330, amt, 5) { if r.3 > 0 { paid += r.0 as u128; i = r.1; so = r.2; tot += r.3; } }
                } else if let Some(r) = ladder_sell(&h, i, 1_000_000, so, amt.min(tot), 50, 5) { got += r.0 as u128; i = r.1; so = r.2; tot -= r.3; }
                assert_eq!(tot, i * 1_000_000 + so);
                assert!(got <= paid, "havuz acik verdi");
                if i >= 120 { break; }
            }
        }
        #[test] fn prorata() {
            // talep 3x kapasite: herkes 1/3
            assert_eq!(pro_rata(300, 1000, 3000), 100);
            assert_eq!(pro_rata(7, 1000, 3000), 2);
            let mut seed = 3u64;
            for _ in 0..200 {
                let n = (lcg(&mut seed) % 40 + 1) as usize;
                let o: Vec<u64> = (0..n).map(|_| lcg(&mut seed) % 100_000 + 1).collect();
                let d: u64 = o.iter().sum(); let cap = lcg(&mut seed) % d + 1;
                let f: u64 = o.iter().map(|x| pro_rata(*x, cap, d)).sum();
                assert!(f <= cap && cap - f < n as u64 + 1);
            }
            assert_eq!(pro_rata(5, 10, 0), 0);
        }
        #[test] fn fair() {
            let max_req = 10_000u64 * 1000; // 10K DK x 1000 lamport
            // bucket
            assert_eq!(bucket_of(max_req, max_req), Some(127));
            assert_eq!(bucket_of(max_req, 1), Some(0));
            assert_eq!(bucket_of(max_req, 0), None);
            assert_eq!(bucket_of(max_req, max_req + 1), None);
            for k in 0..BUCKETS { assert_eq!(bucket_of(max_req, bucket_bound(max_req, k)), Some(k)); }
            // rastgele: tam esitlik ve eksiksiz hesap
            let mut seed = 99u64; let mut worst_gap = 0f64;
            for round in 0..300 {
                let n = (lcg(&mut seed) % 300 + 1) as usize;
                let mut reqs: Vec<u64> = Vec::new();
                for _ in 0..n {
                    let r = match lcg(&mut seed) % 4 { 0 => max_req, 1 => max_req / 2, _ => lcg(&mut seed) % max_req + 1 };
                    reqs.push(r.max(1));
                }
                let mut cnt = [0u32; 128]; let mut sum = [0u64; 128]; let mut minr = [u64::MAX; 128];
                for r in &reqs { let b = bucket_of(max_req, *r).unwrap(); cnt[b] += 1; sum[b] += *r; minr[b] = minr[b].min(*r); }
                let total: u64 = reqs.iter().sum();
                let room = if round % 3 == 0 { total } else { (total as u128 * (lcg(&mut seed) % 95 + 1) as u128 / 100) as u64 };
                let (lv, al) = fair_level(room, max_req, &cnt, &sum, &minr);
                let real: u64 = reqs.iter().map(|r| (*r).min(lv)).sum();
                assert_eq!(real, al, "toplam tam bilinmeli");
                assert!(al <= room);
                // kesin su-doldurma seviyesi
                let mut lo = 0u64; let mut hi = max_req;
                while lo < hi { let mid = (lo + hi + 1) / 2; let f: u128 = reqs.iter().map(|r| (*r).min(mid) as u128).sum(); if f <= room as u128 { lo = mid } else { hi = mid - 1 } }
                if total > room {
                    let exact: u64 = reqs.iter().map(|r| (*r).min(lo)).sum();
                    assert!(lv <= lo || lv == u64::MAX);
                    if n >= 30 { worst_gap = worst_gap.max((exact - al) as f64 / room.max(1) as f64); }
                }
                // en buyuk cuzdan en kucukten (esit seviye) fazla alamaz
                if let (Some(mx), true) = (reqs.iter().max(), lv != u64::MAX) { assert!((*mx).min(lv) <= lv); }
            }
            assert!(worst_gap < 0.06, "kova kaba kaldi: {}", worst_gap);
            // balina testi: 1 balina (max) + 200 kucuk (50 DK), oda = %20 balina+kucuk
            let small = 50u64 * 1000;
            let mut cnt = [0u32; 128]; let mut sum = [0u64; 128]; let mut minr = [u64::MAX; 128];
            let bw = bucket_of(max_req, max_req).unwrap(); cnt[bw] += 1; sum[bw] += max_req; minr[bw] = max_req;
            let bs = bucket_of(max_req, small).unwrap(); cnt[bs] += 200; sum[bs] += 200 * small; minr[bs] = small;
            let room = 12_000_000u64; // talep 20M (balina 10M + 200x50K), oda 12M
            let (lv, al) = fair_level(room, max_req, &cnt, &sum, &minr);
            assert!(al <= room && lv >= small, "kucukler tam dolmali: lv={}", lv);
            let whale = max_req.min(lv);
            assert!(whale <= 2_000_000 && al == whale + 200 * small, "balina yalniz artan odayi alir: {}", whale);
            // az talep: hepsi tam
            assert_eq!(fair_level(u64::MAX, max_req, &cnt, &sum, &minr).0, u64::MAX);
            // cok kalabalik: seviye 0
            let mut cnt = [0u32; 128]; let mut sum = [0u64; 128]; let mut minr = [u64::MAX; 128]; cnt[0] = 1000; sum[0] = 1000 * 9000; minr[0] = 9000;
            let r = fair_level(100, max_req, &cnt, &sum, &minr); assert_eq!(r, (0, 0)); // 100 lamport / 1000 cuzdan = 0 (tam bolunmez)
            let r = fair_level(5000, max_req, &cnt, &sum, &minr); assert_eq!(r, (5, 5000)); // 5 lamport kisi basi
        }
        #[test] fn gov() {
            let w = vote_weight(1_000_000, 365 * 86_400); assert_eq!(w, 365_000_000);
            assert_eq!(vote_weight(10, -5), 0);
            assert_eq!(eff_weight(500, 1000, 500), 50);      // %5 tavan
            assert_eq!(eff_weight(10, 1000, 500), 10);
            assert!(passes(60, 40, 1000, 500));              // yes>no ve kenar yok
            assert!(!passes(40, 40, 1000, 500));
            assert!(!passes(30, 10, 1000, 500));             // yes>no ama yetersiz katilim (30 < 50)
            assert!(passes(50, 0, 1000, 500));
            // 100 cuzdan bolunmus balina: her biri tavan altinda -> toplam agirlik korunur (bilinen sinir: kume tespiti zincir disi)
            let split: u128 = (0..100).map(|_| eff_weight(30, 1000, 500)).sum(); assert_eq!(split, 3000);
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
