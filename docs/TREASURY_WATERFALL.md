# Treasury waterfall (draft v0.1, 2026-10-27)

## User decisions (2026-10-27)
- Pair fee share is set by the competitive picture (our cost base differs from competitors, so we may undercut): rule = protocol share chosen so that we are not above the best outside pool, never below the cost floor.
- Treasury first: everything flows to the treasury ("tank") until we are secured to a maximum target. When the tank is full, the surplus flows to the places that need it (a pooled structure, "investment pool / fund"-like). Important.

## Design
1. TANK (security reserve): target = 24 months of running cost + audit/legal reserve A. All inflows go here first.
2. OVERFLOW POOL ("needs pool"): receives only what exceeds the tank target. Spent on needs (liquidity depth, audits, development, security) by committee vote; each need has a per-period cap; outflow to a vendor pool uses fixed dollar limits (see AUTOPAY_DESIGN).
3. Refill rule: if the tank falls under the target (e.g. below 90%), overflow stops and inflows refill the tank first.
4. Withdrawal cap (0.5%/week, v4) applies to the tank; the tank-to-needs flow needs its own rule (it is not an ordinary withdrawal).
5. Wording: the needs pool pays for protocol needs only. It is NOT a fund that pays holders and must not be called an "investment fund" in public text (securities risk). Internal name: "needs pool".

## Simulation (research/treasury_waterfall_sim.py, ALL INPUTS ASSUMED)
- Tank target about 52K USD (24 months at 97 USD + 50K audit/legal reserve assumption).
- NFT proceeds dominate early (62-79% of 24-month income in the assumed scenarios); pair fees are 1-17%.
- Base case tank fill time by NFT sell-through: 0% -> month 59, 10% -> month 23, 25% -> month 7, 50% -> month 4, 100% -> month 2. Treasury security therefore depends on NFT demand and presale, not on pair fees.
- Pair-fee revenue only matters at high volume; NFT revenue ends after month 12.

## Open
Audit price quotes, real NFT demand, tank-to-needs flow rule, who may propose needs, spending caps per need.
