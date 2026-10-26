# Contracts (draft)

Status: **drafts**. Not audited. Never compiled with Anchor outside Solana Playground; not tested on a test network. Do not use with real funds.

- `presale_program_v3.rs`: current draft. Real SPL token (fixed 1B supply, mint authority revoked), fair presale (10K DK per wallet, equal allocation), ladder pool (buy, sell-back, daily sell cap, rounds with pro-rata fill), agent bond, locked liquidity, staking cap, committee voting.
- `presale_program_v2.rs`, `presale_program.rs`: earlier drafts (kept for history).
- `treasury_program.rs`: reserve withdrawal limits per epoch (0.5% normal, 1% crisis).
- `anchor.test.v3.ts`: test plan for v3 (partial, not run). `anchor.test.ts`: tests of the first draft.

Only the pure `math` module has unit tests (run with `rustc --test`). See `../KNOWN_LIMITATIONS.md`.
