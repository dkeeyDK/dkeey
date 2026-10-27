"""BLURRED-PAIRING SIM (model; all numbers ASSUMPTIONS, not measured on a network).
Idea: the core matches buyers and sellers per round and publishes only hash commitments; a flagged wallet's match is not completed.
Questions:
 A) Does the commitment alone hide who is matched with whom?  (by construction yes; shown)
 B) What still leaks? amounts, order/timing, batch size.  Observer tries to link each seller to its buyer.
 C) Flag gate: how much of a stolen wallet can leave before the flag takes effect?
 D) Cost to honest users: how often a legit wallet is blocked.
Run: python3 -I blur_sim.py"""
import hashlib, os, random
random.seed(31)

def H(*parts):
    h = hashlib.sha256()
    for p in parts: h.update(str(p).encode()); h.update(b"|")
    return h.hexdigest()

# ---------- A) commitments ----------
def part_a(k=20, rounds=3000):
    ok = 0
    for r in range(rounds):
        sellers = list(range(k)); buyers = list(range(k)); random.shuffle(buyers)   # true pairing: seller i -> buyers[i]
        coms = [H(r, os.urandom(8).hex(), sellers[i], buyers[i]) for i in range(k)]  # what the chain shows
        # observer sees only coms; best it can do without nonces is guess a random buyer for seller 0
        guess = random.randrange(k)
        ok += (guess == buyers[0])
    return ok / rounds

# ---------- B) metadata leakage ----------
def one_round(k, mode):
    """mode: (amounts_visible, granularity, settlement)  settlement: 'fifo' keeps order, 'shuffled' = batch settled in random order"""
    vis, gran, sett = mode
    amts = [max(gran, round(random.lognormvariate(6.5, 1.2) / gran) * gran) if gran else random.lognormvariate(6.5, 1.2) for _ in range(k)]
    perm = list(range(k)); random.shuffle(perm)                  # seller i is matched to buyer perm[i]
    seller_view = [(i, amts[i]) for i in range(k)]               # seller side events (id, amount)
    buyer_events = [(perm[i], amts[i]) for i in range(k)]        # buyer side events
    if sett == "fifo":
        order = sorted(range(k), key=lambda i: i)                # buyer event i settles in the same order as the seller event i
        buyer_seq = [buyer_events[i] for i in order]
    else:
        buyer_seq = buyer_events[:]; random.shuffle(buyer_seq)
    # observer: link each seller to a buyer
    guesses = {}
    used = set()
    for pos, (sid, a) in enumerate(seller_view):
        cand = []
        for bpos, (bid, ba) in enumerate(buyer_seq):
            if bid in used: continue
            if vis and ba != a: continue
            cand.append((bpos, bid))
        if not vis and sett == "fifo":
            cand = [c for c in cand if c[0] == pos] or cand      # order leaks the pairing
        if not cand: cand = [(0, b) for b in range(k) if b not in used]
        bid = random.choice(cand)[1]; guesses[sid] = bid; used.add(bid)
    return sum(1 for i in range(k) if guesses[i] == perm[i]) / k

def part_b():
    modes = {
        "amounts public, exact units, in order": (True, 0, "fifo"),
        "amounts public, round to 100, in order": (True, 100, "fifo"),
        "amounts public, fixed lots of 1000, shuffled": (True, 1000, "shuffled"),
        "amounts hidden, in order (timing leaks)": (False, 0, "fifo"),
        "amounts hidden, batch shuffled": (False, 0, "shuffled"),
    }
    print("B) Observer's share of correctly linked pairs (random guess = 1/k)")
    print("   k per round ->            " + "".join(f"{k:>8}" for k in (2, 5, 20, 100)))
    for name, m in modes.items():
        row = []
        for k in (2, 5, 20, 100):
            n = 400 if k < 100 else 120
            row.append(sum(one_round(k, m) for _ in range(n)) / n)
        print(f"   {name:46s}" + "".join(f"{x:8.1%}" for x in row))
    print("   random-guess baseline:            " + "".join(f"{1/k:8.1%}" for k in (2, 5, 20, 100)))

# ---------- C) flag gate on a stolen wallet ----------
def part_c():
    print("\nC) Stolen wallet: share of its balance that leaves before the flag works")
    print("   exit cap per round (share of balance) vs. chance per round that the flag is raised")
    caps = (0.05, 0.20, 1.00); ps = (0.1, 0.3, 0.6, 0.9)
    print("   cap \\ flag chance   " + "".join(f"{p:>8.0%}" for p in ps))
    for c in caps:
        row = []
        for p in ps:
            tot = 0; n = 20000
            for _ in range(n):
                bal = 1.0; out = 0.0
                for r in range(48):
                    out_r = bal * c; bal -= out_r; out += out_r
                    if random.random() < p: break           # flag raised: matches stop
                tot += out
            row.append(tot / n)
        print(f"   {c:>6.0%}              " + "".join(f"{x:8.1%}" for x in row))

# ---------- D) cost to honest users ----------
def part_d():
    print("\nD) Honest wallets blocked by false flags (assumed false-flag rate per wallet per round, flag lasts L rounds)")
    for fp in (0.001, 0.005, 0.02):
        for L in (3, 12):
            print(f"   false-flag rate {fp:.1%}/round, lasts {L:>2} rounds -> share of time a wallet is blocked: {fp*L/(1+fp*L):.2%}")

if __name__ == "__main__":
    print(f"A) Share of correct guesses from commitments alone (k=20, random = {1/20:.1%}): {part_a():.1%}\n")
    part_b(); part_c(); part_d()
    print("\nNOT MODELED: a compromised or colluding core sees the whole mapping (blur hides only from outsiders);")
    print("wallet-level clustering by other chain data; DEX-side trades (outside our pool); real timing noise.")
