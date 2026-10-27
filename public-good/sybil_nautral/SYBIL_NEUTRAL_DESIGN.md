# Sybil-neutral design principle (draft v0.1, 2026-10-27)

## Research basis (what the literature says)
- arXiv 2605.18990: any rule using only wallet balance is plutocratic under free wallet splitting; linear is split-neutral; concave only a constant factor; age = friction; only proof-of-personhood breaks it.
- arXiv 2407.14485 ("On sibyl-proof mechanisms"): in the single-item quasi-linear setting, the only non-wasteful, symmetric, incentive-compatible, sibyl-proof mechanism is a second-price auction with symmetric tie-breaking (highest bidder wins). Meaning: without identity, "fair to many small people" and "sibyl-proof" cannot both hold; money decides.
- Gitcoin "cost of forgery": cap the reward a fake identity can get below its forging cost; graded rights from graded evidence.
- Proof-of-personhood options (air3 comparison): device attestation (very low friction, medium resistance, depends on Apple/Google), phone/SIM (low), KYC (high but identity), biometrics (high, special data), social graph (medium), reusable ZK uniqueness credentials (needs an issuer; e.g. ePrint 2026/1725 Proof-of-Uniqueness, 65 s proof, issuer-trusted, preprint).

## Principle
No identity, no user task. Make every flow either (1) linear in money (holdings, volume, fees) so splitting gives nothing, or (2) a function of GLOBAL state (price curve, total sold), never of wallet count.

## Where it applies in DK
| Item | Old | Sybil-neutral rule |
|---|---|---|
| Presale 10K cap per wallet | bypassed by splitting; hurts only honest big buyers | no per-wallet cap; sell on the price curve / clearing price (simulation: attacker share = money share) |
| Daily exit cap | one global first-come cap (exit denial) | shared pro-rata by holdings (optionally x age factor) |
| Rewards (250M) | undefined per-wallet | pay by holdings x time or fees paid, never per wallet |
| Cover/blur | n/a | agent-made, not user-dependent |

## Simulation result (research/sybil_neutral_sim.py)
Per-wallet cap: attacker with 20% of money and 100K wallets takes 36% (and rises with wallets); money-proportional or clearing-price: attacker share stays 20% for any wallet count. Honest median small buyer: 62% of ask without attackers under the cap rule, 16% under proportional rules; the cap rule is kinder only while nobody splits.

## DECIDED 2026-10-27
Device attestation CLOSED (developer accounts need identity, Apple/Google dependency, trusted verifier needed on-chain, medium resistance, two-tier users, needs mobile app). Presale per-wallet cap REMOVED in favour of price curve / clearing price; exit cap pro-rata by holdings; rewards by holdings x time or fees paid.

## (Closed) Optional graded rights from evidence
A wallet can OPT IN to extra evidence (e.g. device attestation) to raise a bounded perk (never a core right). Cost: dependency on Apple/Google, medium resistance. Decide later; core stays identity-free.

## Honesty
Not new in theory (false-name-proof / split-neutral design, uniform-price auctions, bonding curves). Our contribution is applying it consistently to an admin-minimized exchange (exit cap, presale, rewards). Wording: "to our knowledge this combination is new".
