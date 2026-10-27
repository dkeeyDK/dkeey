# Blur layers: ways to strengthen the idea (proposal, 2026-10-27)

Weak points found in `research/blur_sim.py`: few trades per round, the core sees the whole mapping, a flag works only with an exit limit. Candidate layers below; numbers from `research/layers_sim.py` (assumptions; sections 1 and 3 are formulas, only section 2 is a Monte Carlo run).

| Layer | What it does | Weak point it targets | Evidence | Cost / risk |
|---|---|---|---|---|
| L0 Token-2022 + hidden amounts | amounts hidden from outsiders | amount leak | decided earlier (v5) | rent, 48-bit limit |
| L1 Gate (transfer hook) | our rules run on every transfer | stolen tokens moving | design only | DEX acceptance unknown, freeze risk |
| L2 Batch settlement, min round size | trades settle together in random order | order/timing leak | blur_sim: random-level at 20+ trades | slower trades |
| L3 Cover trades | dummy trades enlarge the crowd | rounds with few trades | formula: 3 real + 17 dummies -> 5% link chance if dummies are indistinguishable | dummies from the core are spotted; need real volunteers; fee cost |
| L4 Delay-and-cancel exits | exit waits D rounds; flag or owner cancels | theft before a flag works | Monte Carlo: at 30% flag chance, D=6 -> 3.9% leaves (D=0 -> 31.7%) | honest large exits wait too |
| L5 Split view | each core seat sees only part of the trades | core sees everything | formula: one bad seat sees 1/8 | no new crypto; partial exposure remains |
| L6 Secret-shared matching (MPC) | no seat sees the mapping unless 6 of 8 collude | same | formula only | needs multi-party computation; heavy; research |
| L7 Stealth settlement addresses | one-time receiving address per trade | address clustering | not simulated, not verified on Solana | research |

## Proposed order
1. L2 and L4 first: cheap, fit the existing round mode and the 24 h exit lock idea, address the two largest numbers above.
2. L5 next (no new cryptography).
3. L3 only if real volunteers can be found; core-funded dummies do not help.
4. L1 together with v5 (Token-2022), after the DEX acceptance check.
5. L6 and L7 are research, with stop rules like Track B.

Not covered: a core fully compromised at once, outside DEX trades, other chain data that links wallets.

## Correction (user, 2026-10-27): instant, see-then-blur again
The user's intent: the core sees the pairing only at the moment of the trade, then it becomes blurred again; the user never waits. Consequences:
- Batch settlement (L2) and delayed exits (L4) are NOT the default. At most: amounts above a per-round cap wait; below the cap everything is instant.
- The matching mechanism is re-randomization inside the same transaction (research/rerandomize_demo.py shows the math for ElGamal): the balance keeps its value, its ciphertext changes, no permanent pairing record on chain.
- The gate checks are precomputed state (flag, cap), so they cost no waiting.
- Not solved: the core cannot prove it forgot (off-chain memory and logs); the signer address stays public. Mitigations: the user prepares the commitment and the core only verifies; split view (each seat sees part of the trade).
- Open decision for the user: accept waiting only above the cap (recommended) or fully instant with higher theft exposure (model: 5% cap, 30% flag chance per round -> about 15% of a stolen balance leaves).
