# Architecture: what is done, what is left, in order (2026-10-26)

## Done (design or code, none audited)
v4 contract compiled in Playground (not run); key split, pause, 24 h lock, committee gate; NFT math tests; privacy page and demo (live); research experiments (resharing, math, handover, auto-pay, common mode); docs for keys, upgrade/migration, privacy tracks, continuity, committee seats.

## Left, in order
1. **Run v4 tests.** Path A: GitHub Actions on a local validator (free, no SOL; `ci/` scaffold). Path B: devnet (needs 3.63 more SOL). Fix errors from the first real run. Then add the 24 h lock test with a clock jump (bankrun).
2. **Vendor pool contract** (`pool_init`, `set_vendor`, `pay_vendor`) per AUTOPAY_DESIGN; tests.
3. **Committee on chain.** Pick an existing audited multisig/timelock program (verify before use), or keep the v4 committee key as a multisig PDA. Build rule seats R1–R3 (COMMITTEE_SEATS), twice in two languages, compare on random inputs. Public seat rotation rule.
4. **Core/module split, migration function, managed upgrade** (CORE_MODULE_MIGRATION) with the multisig phases.
5. **Agent runtime.** Pre-filter against message floods, priority budget pools, two hosting providers, rule seats on a third place.
6. **Continuity.** GitHub organization with several owners, multi-year domain, exit script and on-chain memo (CONTINUITY_PLAN).
7. **Token v5.** Token-2022 confidential mint: a small devnet test (one mint, two accounts, one confidential transfer); check Playground/CI versions. DK utility binding (entry fee, NFT fee, vote lock, work payments); review the SOL bond.
8. **NFT.** Metaplex Core standard (allowlist royalties, burn delegate), market program, burn controller PDA; devnet tests incl. blocked direct wallet transfers; pre-mint only after the program IDs exist.
9. **Research (parallel, not blocking).** Track B milestones M2–M4; Winternitz cold vault review; unread zkML/TEE sources; resharing only in a bound-respecting shape if a shared key is needed.
10. **Before mainnet.** Outside audit; legal review; DK_DECIMALS final; whale splitting; real yearly cost and treasury gap; family similarity test; rework of the post-quantum commitment layer; then multisig, K1 retirement, managed upgrade phases.

## Added 2026-10-27 (decisions from sybil-neutral design)
- v5 presale: remove per-wallet 10K cap; price curve / clearing-price sale.
- Exit cap: share the daily cap pro-rata by holdings (optionally x age factor, capped).
- Rewards: holdings x time or fees paid; never per wallet.
- Large buys: force round mode (batch, single price) against sandwiching.
- Gizli havuz (DK_BAG_SPEC): ZK cost measurement prototype after v4 tests.
