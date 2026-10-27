# Idea (user, 2026-10-27): blurred pairing and core-completed transfers

User's idea: each DK has a hidden "twin"; who is paired with whom is not known in advance. When a buy, sell or transfer happens, the core matches it in the pool and completes it. If a security condition is open (flag), the match is not completed. Aimed at the case where DK is also traded on other exchanges/DEXs.

## Closest real mechanism (to be verified before any design)
- Token-2022 "transfer hook": the token program calls a program of ours on every transfer; the program can reject the transfer. This is a transfer gated by our rules. Fits the idea of "the core completes the move".
- Blurred pairing: commitments (and later zero-knowledge proofs) so an outsider cannot see which twin belongs to which holder. Not designed.
- Fits the v5 plan (move DK to Token-2022).

## Open questions
- Do outside DEXs accept a token with a transfer hook? Support differs per DEX and per version; must be checked one by one (not verified).
- Censorship risk: a core that can refuse transfers is a gatekeeper. Needs hard limits: flags only by written rules, expire in at most 72 h, never block exits to the user's own wallet, public log.
- Single point of failure if the hook program or the core keys break: tokens could freeze. The v4 pause design (72 h max, 24 h cooldown) is the model.
- Does it add security beyond a second factor? Not shown. It may deter selling stolen tokens through DEXs (needs a test).
- Per-unit twins are impossible for a fungible token (1B units); the twin is per account or per transfer.

## Next step (when the user agrees)
Short research note: transfer hook mechanics on Solana, which DEXs accept it, a small devnet test (one mint with a hook that rejects when a flag is set), and a rule list for flags.

## Simulation result (research/blur_sim.py, assumptions only, 2026-10-27)
- Commitments alone hide the pairing (observer guesses at random level). This is true by construction, not a finding.
- What leaks is metadata. If amounts are public or the settlement keeps order, an observer links 100% of pairs. Hidden amounts plus a batch settled in random order gets close to random guessing when a round has 20 or more trades (5.4% vs 5.0% at k=20; 1.1% vs 1.0% at k=100); with 2 to 5 trades per round the blur is weak. Fixed-size lots help but do not replace hidden amounts.
- Flag gate on a stolen wallet: it only works together with a per-round exit cap. With a 5% cap per round and a 30% chance per round that the flag is raised, about 15% of the balance leaves; with no cap, 100% leaves, whatever the flag speed.
- False flags cost honest users: at 0.5% false flags per round lasting 12 rounds a wallet is blocked about 5.7% of the time. Flags must be rare, short and rule based.
- Not modeled: the core sees the whole mapping (blur is only against outsiders); other chain data that clusters wallets; trades on outside DEXs.
