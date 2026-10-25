# Contracts (draft)

Status: **drafts**. Not audited. Never compiled outside Solana Playground; not tested on a test network. Do not use with real funds.

- `presale_program.rs`: presale escrow and treasury (original).
- `presale_program_v2.rs`: original plus agent bond, price ladder, locked liquidity, staking cap. Only the pure math module has unit tests.
- `treasury_program.rs`: reserve withdrawal limits per epoch (0.5% normal, 1% crisis).
- `anchor.test.ts`: test plan for the Anchor runner; not run yet.

See `../KNOWN_LIMITATIONS.md`.
