# Pairs, fees and treasury funding (draft v0.1, 2026-10-27)

## Decisions by the user (2026-10-27)
- Launch/test with DK/SOL only. Treasury funding through other pairs is a separate, important topic.
- New pairs are voted by the committee; freezable assets (issuer can freeze, e.g. fiat stablecoins) need a stricter rule.
- Wallet support should be as wide as possible (Wallet Standard).

## What sets the fee on a pair (our share = total fee x protocol share)
1. Risk class of the pair (volatility, depth, manipulation risk, freeze risk) -> base total fee, within hard bounds (proposal: 10-100 bps).
2. Competition: a fee far above the deepest competing pool sends volume away; the committee may not set it more than a fixed margin above the best outside pool.
3. Cost floor: protocol share must at least cover running cost (formula below).
4. Protocol share of the fee: committee-set inside hard bounds (proposal: 15-30% of the fee); the rest goes to liquidity providers.
5. Change speed: at most +-10 bps per month; bounds and share changes need 8/8 plus the 24 h lock (same tier as caps).
6. Listing a pair: proposer posts a refundable bond (anti-spam); refunded if the pair passes a volume/depth threshold.
Already fixed earlier (to be recomputed): LP-locked-liquidity commission 7% of LP fees; ladder sell spread 0.5%; NFT auction/secondary commission 2.5% -> treasury; draft simulated fees 15 bps LP + 5 bps protocol.

## Break-even volume per month = monthly cost / (total fee x protocol share)
| monthly cost | 0.03% to us | 0.05% to us | 0.10% to us |
|---|---|---|---|
| $25 | $83K | $50K | $25K |
| $97 | $323K | $194K | $97K |
| $378 | $1.26M | $756K | $378K |
(Our share of volume. Real volume is unknown; all numbers are assumptions.)

## Treasury funding sources (not only pairs)
presale/clearing price proceeds; protocol share of pair fees; ladder spread; NFT auction and secondary commissions; LP-locked-liquidity commission. Early months depend on the first two; pair fees need volume nobody can promise.

## Rules for freezable assets (proposal)
Listed only if (a) committee 8/8, (b) position size capped, (c) automatic delisting if the issuer freezes any DKEEY-controlled account, (d) never used as the only reserve.
