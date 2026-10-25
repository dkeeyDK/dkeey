# Known limitations (honest list)

Status: design and test stage. Nothing here is live. Not audited. Not financial or legal advice.

## Contract (contracts/presale_program_v2.rs)
- Never compiled with Anchor outside Solana Playground; only the pure math module has unit tests.
- Accounting only: no real SPL token mint and no real trading pool yet.
- Presale per-wallet cap (10K DK) and equal distribution are not in `contribute` yet.
- Price ladder has no sell-back, no daily sell cap and no pro-rata fill.
- "Committee" is currently a single authority key, not a real vote.
- Staking budget is first-come within each year.
- Tiny transfers to empty system PDAs can fail rent checks; slash can fail if the liquidity PDA is empty and the amount is tiny.

## Design
- No complete whale protection exists; limits are tiers (wallet cap, one wallet = one agent, bond, on-chain clustering). Evasion reduces their effect.
- Protections apply only to our own pool; other venues are out of reach. Strategy: be the deepest pool.
- Yearly cost numbers are general estimates, not verified quotes.
- Early period: reserve (about 20.8K USD) is below the minimum yearly cost estimate (75K USD).
- Commission (7%) is enough at large volume, not at small volume.
- Legal status of this kind of system is unclear; a monitoring agent only reports, it is not a lawyer.
- Security: admin-minimized, not "no admin". Experimental. No price target, no promise.
