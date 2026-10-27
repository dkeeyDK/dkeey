# DK Bond (Bağ) Specification - draft v0.1 (2026-10-27)

Status: DESIGN DRAFT. Not audited. No claim of novelty beyond "to our knowledge this combination is new". No claim of quantum safety (the "quantum bond" is only a metaphor).

## 1. Decision (pool or plain number)
Hybrid, decided by Claude (user delegated):
- **Open DK** stays an ordinary SPL token: plain number, unique mint, DEX-compatible. Nothing about the bond is required to hold or trade it.
- **Shielded pool** is OPTIONAL. The bond lives only inside the pool. A user deposits open DK, trades blurred inside, withdraws back. Works even if outside DEXes reject confidential amounts.

## 2. Goals
- G1 Unlinkability: an observer cannot tell which deposit/sell in a round is paired with which buy/withdraw beyond the anonymity set.
- G2 Correctness: no inflation. Sum of committed values is conserved (range proofs needed).
- G3 Threshold privacy: any 5 of 8 seats learn nothing about pairings; 6 of 8 are needed to evaluate a pairing tag.
- G4 No single point of failure; no founder key.
- G5 Instant: no user waits. Small, indistinguishable cover trades fill small rounds.

## 3. Building blocks (all standard, none invented here)
| Need | Primitive |
|---|---|
| Hide amount and owner | Pedersen commitment in a Merkle tree of commitments |
| Look different each time | Re-randomization C' = C * h^dr (same value) |
| Prove validity without revealing which | Zero-knowledge membership + range proof (Groth16 or similar; Solana alt_bn128 verification must be checked) |
| Stop double spend | Nullifier per unit |
| Nobody sees the key | Shamir 6-of-8 secret sharing + threshold PRF (tag = H(x)^k, partials combined by Lagrange) |
| Pairing at trade time | Committee evaluates tag for the round, shuffles, settles, then outputs are re-randomized |

## 4. Flow (per round)
1. Users deposit open DK -> commitments appended to the tree.
2. Orders enter the round (real + agent cover trades, amounts hidden, cover size small).
3. 6 seats evaluate threshold tags; pairing is visible only to the combined computation for this round.
4. Settlement outputs are new commitments (re-randomized); old ones nullified.
5. Withdraw: ZK proof of ownership + nullifier -> open DK.

## 5. What the toy check showed (research/bond_toy.py, 13/13 pass)
Math behaves as claimed: re-randomization keeps the value and changes the look; commitments add up (no inflation); 6 shares rebuild a key, 5 shares give no information; 6 partial tags equal the real tag, 5 do not; double spend is caught.
NOT shown: zero-knowledge proofs, real group sizes, network security, Solana cost. The toy uses a 256-bit group and no ZK.

## 6. Simulation results that bound the promise (research/blur_levels_sim.py)
- Hidden amounts + cover: ~30 candidates per side from 100 users on (link probability ~3%). Visible amounts: amount matching defeats blur at any crowd size.
- Cover trades distinguishable (90% accuracy) at 10K users: link probability rises 3% -> 25%.
- Repeat trading leaks: 10 trades -> ~40% chance at least one is linked (at 5% per trade).
- Compromised core (6 of 8 needed): 0.002% at 10% independent seat compromise; ~1.4% if 5% chance of a shared LLM blind spot (rule seats limit this).
- Entry and exit moments of the pool are visible; small pools weaken this.

## 7. Promise wording (allowed)
"The pairing of a single trade is blurred inside the pool; strength shown as a level (low/medium/high) that depends on crowd size."
NOT allowed: "fully anonymous", "untraceable", "quantum-safe", "first/unique in the world".

## 8. Open items
1. Devnet/local-validator test: Token-2022 confidential transfer + transfer hook with DEXes (only needed if we also want confidential amounts on the open token).
2. Solana verification cost of ZK proofs (compute units, proof size).
3. Cover-trade indistinguishability (timing, fee, wallet patterns).
4. Review by an applied cryptographer before any real funds.
5. Legal review: mixer-like classification; patent screening.
6. Pool bootstrap: weak at low usage; show the level honestly.

## 9. Research findings (2026-10-27) - feasibility on Solana
Sources: Chainstack Solana ZK proofs guide; Solana Compass (BN254/BLS12-381/ZK ElGamal activation); Raydium Token-2022 support page; Solana Program Token-2022 status page.
- Groth16 verification on Solana: about 170K-500K compute units, proof 256-388 bytes, tx size limit 1232 bytes (larger in tx v1). Fits one instruction with a raised CU limit.
- BN254 (alt_bn128) pairing checks are live on mainnet (about 100-bit security, not 128). BLS12-381 also live.
- ZK ElGamal Proof Program (used by Token-2022 confidential transfers) was disabled in June 2025 after a flaw, re-enabled after audit in Agave 4.0. Lesson: this class of code has had real bugs.
- Raydium blocks confidential transfer AND transfer hook for permissionless CLMM pools. => the "confidential open DK on outside DEXes" path is closed. The shielded-pool design (open DK stays a plain SPL token) is the compatible path.
- Cautions: Groth16 needs a trusted setup; a 1-of-1 setup leaves a trapdoor with whoever ran it => needs a multi-party ceremony (8 seats + public contributors; safe if at least one participant is honest). Tooling seen is unaudited. Verifier program must be made immutable. Public inputs must be bound to on-chain state. Circuit correctness needs its own audit.
- Simulation (research/pool_cost_sim.py): ~86K proofs/day (mostly cover trades), ~0.3% of chain capacity, ~0.43 SOL/day in base fees at 350K CU/proof; blur level stays HIGH from 10K users on. All values are estimates; CU limit per block, Poseidon cost and proving time were not measured.
