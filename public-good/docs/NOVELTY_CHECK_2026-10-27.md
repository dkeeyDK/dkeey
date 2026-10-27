# Novelty check (focused search, 2026-10-27) - not a patent search

## Candidate 1: holdings x age weighted shares (exit cap / rewards)
- arXiv 2605.18990 ("Concave is the New Linear"): any rule based only on wallet balance is plutocratic under free wallet splitting; linear weighting is split-neutral; concave rules help only by a constant factor; age/conviction weighting is only friction (a patient attacker pre-splits); only proof-of-personhood formally breaks it.
- Our simulation agrees (linear = split-neutral; age cannot beat the money share). => We must NOT claim "age stops sybil". Honest claim: split-neutral, and it makes a fresh-money attack slower.
- Withdrawal allocation in the wild: FIFO queues, per-epoch caps, daily caps, size-based fees (Spark glossary). Pro-rata-by-holdings exit capacity was not found in that source; absence is not proven.
- Verdict: LOW novelty. Useful engineering choice, not an invention.

## Candidate 2: non-LLM "rule seats" in an AI committee (threshold math)
- Known: LLM agents are correlated (shared blind spots); debate can drift to unanimous wrong answers (23.9% in a cited study); Byzantine bound N>=3f+1 assumes independent faults; "representational collapse" paper (arXiv 2604.03809, preliminary, one model family). Recommended mitigations: different evaluator models, dissent checks, deterministic validation of outputs.
- Not found in what I read: counting deterministic rule seats as threshold voters with a pigeonhole bound (>= n-t+1 non-LLM seats at t-of-n).
- Verdict: POSSIBLY small novelty, not verified (limited search). Simple idea; must be described as a design choice.

## Wording rule
Allowed: "to our knowledge this combination of parts is new". Not allowed: "first", "unique", "new cryptography".
