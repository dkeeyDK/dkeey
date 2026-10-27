# DKEEY public-good bundle

Open-source parts of DKEEY, an experimental Solana exchange operated by a committee of AI agents under public rules. **Not launched. Not audited. Nothing here is financial or legal advice.**

## Why publish this
Show a concrete, checkable example of AI automation used with limits: agents follow public rules, propose changes, and nothing ships without an approval vote (threshold signatures plus a time lock). Everything below can be reused and checked.

## Contents
- `sybil_neutral/` - allocation and exit-cap rules that do not reward splitting money across wallets (design note + simulation).
- `committee_threshold/` - toy checks of Pedersen commitments, Shamir 6-of-8 sharing, threshold PRF and nullifiers. A teaching toy, not production cryptography.
- `simulations/` - agent-based simulations (1M users, price-ladder vs AMM, treasury waterfall).
- `docs/` - novelty check (this is a combination of known pieces, not a new invention), treasury design, grant notes.

## Run
`python3 <file>.py` (Python 3, needs numpy for the simulations (pip install numpy); bond_toy.py needs only the standard library). Result files `*_sonuc.txt` are the recorded outputs.

## Limits
Simulations use assumptions stated in each file. Real behaviour will differ. Read `docs/NOVELTY_CHECK_2026-10-27.md` before citing any claim of novelty.

License: Apache-2.0.
