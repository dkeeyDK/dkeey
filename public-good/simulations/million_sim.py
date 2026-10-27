"""DKEEY 1,000,000-USER SIMULATION (agent-based, numpy). EVERY NUMBER BELOW IS AN ASSUMPTION unless it says "contract".
The ladder follows the v4 contract rules: bins, price steps, buy at the current bin, sell top-down from the current bin
at each bin's own price minus 0.5% spread, ONE GLOBAL daily sell cap (first come first served). Prices are in units (1.0 = ladder start).
Run: python3 -I million_sim.py [N]     (default N = 1,000,000)"""
import sys, numpy as np
N = int(sys.argv[1]) if len(sys.argv) > 1 else 1_000_000
rng = np.random.default_rng(7)

# ---- contract constants / assumptions ----
BINS = 127; SUPPLY = 100_000_000            # contract: supply/bin_size <= 127, CAP_LADDER = 100M
BIN = SUPPLY / BINS
SPREAD = 0.005                              # contract SELL_SPREAD_BPS
STEP = 0.02                                 # ASSUMPTION: agents set 2% per bin (contract max 5%)
PRICE0 = 1.0
price = PRICE0 * (1 + STEP) ** np.arange(BINS)
bounds = np.arange(BINS + 1) * BIN                       # DK sold at each bin boundary
cumval = np.concatenate([[0.0], np.cumsum(price * BIN)])  # value paid for first k bins
def C(x):  # total value of the first x DK sold along the ladder (piecewise linear)
    return np.interp(x, bounds, cumval)
def spot(S):  # current price of the bin being filled
    return price[np.minimum((S // BIN).astype(int) if isinstance(S, np.ndarray) else int(S // BIN), BINS - 1)]

def line(s=""): print(s)
def gini(x):
    x = np.sort(np.asarray(x, float)); n = len(x)
    if n == 0 or x.sum() == 0: return 0.0
    return float((2 * np.arange(1, n + 1) - n - 1).dot(x) / (n * x.sum()))

# =========================================================
line(f"N = {N:,} users | ladder supply {SUPPLY:,} DK in {BINS} bins | step {STEP:.0%} | max price = {price[-1]:.1f}x start")
# ---------------- S1: presale fairness and sybil ----------------
line("\nS1) PRESALE: 30M DK, fair water-filling, 10K DK cap per WALLET (contract). Honest wallets request lognormal (median 60 DK).")
PRE = 30_000_000
honest = np.minimum(rng.lognormal(np.log(60), 1.4, N), 10_000)
def waterfill(req, supply):
    lo, hi = 0.0, req.max()
    for _ in range(60):
        mid = (lo + hi) / 2
        if np.minimum(req, mid).sum() > supply: hi = mid
        else: lo = mid
    return np.minimum(req, lo), lo
for whales, wallets_each in ((0, 0), (200, 1000), (1000, 1000), (1000, 5000)):
    req = honest if whales == 0 else np.concatenate([honest, np.full(whales * wallets_each, 10_000.0)])
    alloc, L = waterfill(req, PRE)
    sy = alloc[N:].sum() if whales else 0.0
    ha = alloc[:N]
    srt = np.sort(ha)[::-1]
    line(f"   whales={whales:>5} x wallets={wallets_each:>5}: level {L:8.1f} DK/wallet | honest total {ha.sum()/1e6:5.2f}M | sybil share {sy/PRE:6.1%} | honest top-1% share {srt[:N//100].sum()/ha.sum():5.1%}")
line("   Reading: the 10K cap is per wallet, so many wallets bypass it; the sybil share rises with the number of wallets, not with money alone.")

# ---------------- ladder market engine ----------------
def run_market(days, run_day=None, run_share=0.0, daily_cap=None, design="fifo", attacker=False, seed=1):
    r = np.random.default_rng(seed)
    daily_cap = daily_cap if daily_cap is not None else 2_000_000 * N / 1_000_000   # ASSUMPTION: cap = 2% of ladder supply per day at N=1M
    want = np.minimum(r.lognormal(np.log(60), 1.4, N), 50_000)       # DK each user would like to hold
    h = np.zeros(N); basis = np.zeros(N)                              # holdings (DK) and value paid
    S = 0.0                                                            # DK sold along the ladder
    m = 0.0; log = []
    runners = None; stuck_hist = []; run_hold0 = 0.0
    exit_attempt_day1 = None
    for d in range(days):
        m = 0.9 * m + r.normal(0, 0.1)                                 # sentiment
        # --- buys ---
        pb = 0.15 if d < 10 else max(0.0, 0.01 * (1 + 3 * m))          # launch: each user buys within ~10 days, later 1%/day
        if runners is not None: pb = 0.0
        buyers = np.where((r.random(N) < pb) & (h < want))[0]
        if len(buyers):
            amt = np.minimum(want[buyers] - h[buyers], want[buyers] * r.uniform(0.2, 1.0, len(buyers)))
            order = r.permutation(len(buyers)); buyers, amt = buyers[order], amt[order]
            cum = np.cumsum(amt); before = cum - amt
            fill = np.clip(np.minimum(amt, SUPPLY - S - before), 0, None)
            cost = C(S + before + fill) - C(S + before)
            h[buyers] += fill; basis[buyers] += cost
            S += fill.sum()
        # --- sell attempts ---
        if run_day is not None and d == run_day:
            runners = r.random(N) < run_share
            run_hold0 = float(h[runners].sum())
        ps = max(0.0, 0.01 * (1 - 3 * m)) if d >= 10 else 0.0
        sell_mask = (r.random(N) < ps) & (h > 0)
        if runners is not None: sell_mask |= runners & (h > 0)
        sellers = np.where(sell_mask)[0]
        att_sold = 0.0
        if attacker and d >= (run_day or 0):
            att_sold = min(daily_cap if design == "fifo" else 0.5 * daily_cap, S)   # fifo: attacker fills the cap first; pro-rata: attacker holds 50% so gets 50%
        sold_today = 0.0
        if len(sellers) or att_sold:
            amt = np.where(runners[sellers], h[sellers], h[sellers] * r.uniform(0.3, 1.0, len(sellers))) if runners is not None and len(sellers) else (h[sellers] * r.uniform(0.3, 1.0, len(sellers)))
            cap_left = max(daily_cap - att_sold, 0.0)
            if design == "prorata":
                tot = amt.sum(); f = min(1.0, cap_left / tot) if tot > 0 else 0
                fill = amt * f; sold = fill.sum()
                avg = (C(S) - C(S - sold)) / sold * (1 - SPREAD) if sold > 0 else 0
                proceeds = fill * avg
            else:
                order = r.permutation(len(sellers)); sellers, amt = sellers[order], amt[order]
                cum = np.cumsum(amt); before = cum - amt
                fill = np.clip(np.minimum(amt, cap_left - before), 0, None)
                top = S - att_sold
                proceeds = (C(top - before) - C(top - before - fill)) * (1 - SPREAD)
                sold = fill.sum()
            h_before = h[sellers].copy()
            frac = np.divide(fill, h_before, out=np.zeros_like(fill), where=h_before > 0)
            basis[sellers] -= basis[sellers] * frac
            h[sellers] -= fill
            S -= sold; sold_today = sold          # attacker's own sale is bought back the same day (round trip), so only honest sales move the supply
            if exit_attempt_day1 is None and run_day is not None and d == run_day:
                exit_attempt_day1 = (len(sellers), int((fill > 0).sum()), amt.sum(), sold)
        if runners is not None:
            left = (h[runners] > 1e-9).sum(); stuck_hist.append((d - run_day, int(left), float(h[runners].sum())))
        log.append((d, S, float(price[min(int(S // BIN), BINS - 1)]), sold_today))
    return dict(h=h, basis=basis, S=S, log=log, runners=runners, stuck=stuck_hist, day1=exit_attempt_day1, want=want, hold0=run_hold0, cap=daily_cap)

# ---------------- S2: calm market ----------------
line(f"\nS2) CALM MARKET, 120 days (daily sell cap {2_000_000*N/1e6:,.0f} DK = 2% of ladder supply at N=1M, ONE cap for everyone):")
res = run_market(120)
lg = res["log"]
line(f"   DK sold on the ladder at end: {res['S']/1e6:6.1f}M of 100M | price now {lg[-1][2]:.2f}x | holders {int((res['h']>0).sum()):,}")
line(f"   daily cap used on average {np.mean([x[3] for x in lg[10:]])/res['cap']:5.1%}; busiest day {max(x[3] for x in lg)/res['cap']:5.1%}")
line("   Price path (day: price): " + ", ".join(f"{x[0]}: {x[2]:.2f}" for x in lg[::20]))
hh = res["h"] > 0; avgp = res["basis"][hh] / res["h"][hh]
line(f"   holders whose average buy price is below today's price (paper profit): {np.mean(avgp < lg[-1][2]):5.1%}; median holding {np.median(res['h'][hh]):.0f} DK; top 1% hold {np.sort(res['h'])[::-1][:N//100].sum()/res['h'].sum():5.1%} of DK")

# ---------------- S3: bank run, global FIFO vs pro-rata ----------------
line("\nS3) BANK RUN on day 40: a share of all holders tries to sell EVERYTHING; others stop buying.")
for share in (0.10, 0.30, 0.60):
    for design in ("fifo", "prorata"):
        res = run_market(100, run_day=40, run_share=share, design=design, seed=3)
        st = res["stuck"]; r0 = res["runners"]; tot0 = res["hold0"] or 1
        n_att, n_ok, att_amt, sold1 = res["day1"] or (0, 0, 0, 0)
        def left_at(k):
            return next((s[2] / tot0 for s in st if s[0] == k), 0.0)
        line(f"   {share:4.0%} run, {design:7s}: day-1 want {att_amt/1e6:6.1f}M DK, cap lets {sold1/1e6:4.1f}M out | still holding after 1/7/30/59 days: {left_at(1):5.1%} {left_at(7):5.1%} {left_at(30):5.1%} {left_at(59):5.1%} | price now {res['log'][-1][2]:.2f}x")

# ---------------- S4: cap grief ----------------
line("\nS4) EXIT DENIAL: one attacker sells the whole daily cap first every day, then buys it back (round trip).")
cap_value = (2_000_000 * N / 1e6) * price[40]
line(f"   cost per day of blocking everyone else (FIFO global cap) = spread on the cap's value = {SPREAD:.1%} x {cap_value/1e6:.2f}M units = {SPREAD*cap_value:,.0f} units/day")
res0 = run_market(100, run_day=40, run_share=0.3, design="fifo", seed=3)
res1 = run_market(100, run_day=40, run_share=0.3, design="fifo", attacker=True, seed=3)
res2 = run_market(100, run_day=40, run_share=0.3, design="prorata", attacker=True, seed=3)
def left(res, k):
    st = res["stuck"]; tot0 = res["hold0"] or 1
    return next((s[2] / tot0 for s in st if s[0] == k), 0.0)
line(f"   30% run, still holding after 7 / 30 days:  no attacker {left(res0,7):5.1%} {left(res0,30):5.1%} | attacker + FIFO cap {left(res1,7):5.1%} {left(res1,30):5.1%} | attacker + pro-rata cap (attacker holds 50% of supply) {left(res2,7):5.1%} {left(res2,30):5.1%}")
line("   Reading: with one global first-come cap, a very cheap attacker can freeze the exit. A cap shared in proportion to holdings forces the attacker to hold a large share of the supply.")

# ---------------- S5: theft ----------------
line("\nS5) THEFT: each day a share of wallets is compromised; thief tries to drain the wallet through the ladder (cap per wallet per round r, flag chance per round p).")
for theta in (0.0001, 0.001):
    n_wal = int(N * theta)
    for cap, p in ((1.0, 0.3), (0.2, 0.3), (0.05, 0.3), (0.05, 0.6)):
        rr = np.random.default_rng(5); leaked = []
        for _ in range(4000):
            bal = 1.0; out = 0.0
            for rd in range(48):
                o = bal * cap; bal -= o; out += o
                if rr.random() < p: break
            leaked.append(out)
        line(f"   {theta:.2%} wallets/day ({n_wal:>5,} wallets): exit cap {cap:4.0%}/round, flag {p:3.0%}/round -> {np.mean(leaked):5.1%} of each stolen balance leaves = about {n_wal*120*np.mean(leaked):,.0f} units/day (assumed 120 units per wallet)")
line("   (Model from blur_sim part C; the flag is only useful when the per-round exit cap is small.)")

# ---------------- S6: load, blur set, false flags, treasury ----------------
line("\nS6) LOAD, BLUR, FALSE FLAGS, TREASURY (assumed activity: 0.3 transactions per user per day, peak x50)")
tx_day = N * 0.3; avg_tps = tx_day / 86400
line(f"   {tx_day:,.0f} tx/day = {avg_tps:,.1f} tx/s average, {avg_tps*50:,.0f} tx/s at peak; trades per 60 s round at average: {avg_tps*60:,.0f} (blur needs >= 20)")
for fp in (0.0001, 0.001):
    line(f"   false flag {fp:.2%}/wallet/day, lasts 1 day -> {N*fp:,.0f} honest wallets blocked on any day")
line("   Treasury: presale raise = 30M DK at about 1.27 units; reserve 75%; withdrawal cap 0.5%/week (v4). Agent bill per month needs:")
for u in (0.0005, 0.001, 0.005):
    raised = 30e6 * 1.27 * u; reserve = raised * 0.75; wk = reserve * 0.005
    line(f"   unit = ${u}: raise ${raised:,.0f}, reserve ${reserve:,.0f}, weekly cap ${wk:,.0f} = ${wk*4.33:,.0f}/month   (AI bill $25 / $97 / $378 per month)")
line("\nNOT MODELED: real prices and SOL/USD, wallet behaviour beyond simple random rules, outside DEX trades, MEV, the AI committee's decisions, network failures, legal events.")
