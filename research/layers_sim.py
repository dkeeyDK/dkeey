"""LAYERS SIM (model, ASSUMPTIONS). Three cheap fixes for the weak points found in blur_sim.py.
 1) Cover trades: pad a round with indistinguishable dummy trades so a real trade hides among more.
 2) Delay-and-cancel exits: an exit waits D rounds; a flag cancels everything pending.
 3) Split view: the core is cut into seats and each seat sees only a part of the trades.
Run: python3 -I layers_sim.py"""
import random, math
random.seed(41)

print("1) COVER TRADES. Chance an observer links a real seller to its buyer = 1 / (real + dummies not recognised)")
print("   real trades per round = 3; cost = dummy trades paid for per round")
print("   dummies | observer can spot  0%   50%   90% of dummies")
for d in (0, 5, 17, 97):
    row = []
    for phi in (0.0, 0.5, 0.9):
        eff = 3 + d * (1 - phi)
        row.append(1 / eff)
    print(f"   {d:>7} |                 " + "".join(f"{x:6.1%}" for x in row))
print("   Caveat: dummies funded by the core are easy to spot on chain; they must come from real users (volunteers, self-trades) to be worth anything.")

print("\n2) DELAY-AND-CANCEL. Stolen wallet; cap = 20% of the balance per round (requests), flag chance per round p.")
print("   An exit request matures D rounds later; a flag cancels all pending and blocks the rest.")
def theft(D, p, cap=0.2, rounds=60, n=20000):
    tot = 0.0
    for _ in range(n):
        bal = 1.0; pend = []; out = 0.0; flagged = False
        for r in range(rounds):
            if random.random() < p: flagged = True
            if flagged: break
            req = bal * cap; bal -= req; pend.append((r + D, req))
            for m, a in list(pend):
                if m <= r: out += a; pend.remove((m, a))
        tot += out
    return tot / n
print("   delay D \\ flag chance   " + "".join(f"{p:>8.0%}" for p in (0.1, 0.3, 0.6)))
for D in (0, 2, 6, 12):
    print(f"   {D:>3} rounds           " + "".join(f"{theft(D,p):8.1%}" for p in (0.1, 0.3, 0.6)))
print("   Price: every honest large exit also waits D rounds (the owner can cancel a stolen request, too).")

print("\n3) SPLIT VIEW. The core is n=8 seats. Share of trades whose pairing a compromised group can see.")
print("   compromised seats | each seat sees only its 1/8 | full mapping needs 6-of-8 shared (secret sharing)")
for c in range(0, 9):
    part = c / 8
    ss = 1.0 if c >= 6 else 0.0
    print(f"   {c:>17} | {part:27.0%} | {ss:.0%}")
print("   Secret-sharing protects better but needs multi-party computation to MATCH trades; the split view needs no new cryptography.")
