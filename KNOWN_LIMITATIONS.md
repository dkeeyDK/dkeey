# Known limitations (honest list)

Status: design and test stage. Nothing here is live. Not audited. Not financial or legal advice.

## Contract (contracts/presale_program_v3.rs)
- Never compiled with Anchor outside Solana Playground. Only the pure `math` module has unit tests (9 tests pass). The first Anchor build will likely show errors to fix.
- On-chain tests (`contracts/anchor.test.v3.ts`) cover the token and presale flow only, and have not been run. Pool, rounds and voting tests are still to be written.
- Token uses 0 decimals in this draft (prices are whole lamports per DK). The decimal choice must be revisited before any mainnet use.
- Presale fairness: allocation is exact per bucket level; a small remainder can stay unallocated and rolls to the next week. Splitting one person across many wallets is not stopped by this rule alone (see design limits).
- Committee: parameter changes go through lock-weighted voting with a per-wallet cap, but operators (slash, score, step) are still a single authority key. Agent wallets (5% allocation, no voting) are not blocked from locking votes on-chain. Voting needs enough distinct lockers to reach quorum (bootstrap).
- Ladder: sell-back pays from the liquidity account, which also holds presale proceeds. Rounds need someone to call open/close (anyone can).
- Small transfers to empty system PDAs can fail rent checks; slash can fail if the liquidity PDA is empty and the amount is tiny.
- Staking budget is first-come within each year and limited by the reward vault balance.

## Design
- No complete whale protection exists; limits are tiers (wallet cap, equal allocation, one wallet = one agent, bond, on-chain clustering). Evasion reduces their effect.
- Protections apply only to our own pool; other venues are out of reach. Strategy: be the deepest pool.
- Yearly cost numbers are general estimates, not verified quotes.
- Early period: reserve (about 20.8K USD) is below the minimum yearly cost estimate (75K USD).
- Commission (7%) is enough at large volume, not at small volume.
- Legal status of this kind of system is unclear; a monitoring agent only reports, it is not a lawyer.
- Security: admin-minimized, not "no admin". Experimental. No price target, no promise.
