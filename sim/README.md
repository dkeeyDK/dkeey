# Simulations and reference models

Run each file with `python3 file.py`. Results are models built on assumptions (for example, demand), not predictions.

- `presale_ref.py`, `treasury_ref.py`: reference logic for the contracts in `../contracts`.
- `social_ref.py`: allowed-message templates for the social agent, with tests.
- `aml_ref.py`, `mev_sim.py`, `committee_corr_sim.py`, `jobmarket_sim.py`: safety and incentive checks.
- `sim42.py`, `sim4216.py` and their `*_sonuc.txt` outputs: governance and revenue/sell-pressure simulations.


## v3 additions
- `presale_v3_ref.py` + `presale_v3_ref_sonuc.txt`: Python reference of the fair presale allocation (same bucket algorithm as the contract); checks conservation of funds, the 10K DK per wallet cap and whale share.
- `math_v3_tests.rs`: the contract's pure math module with its unit tests (`rustc --edition 2021 --test math_v3_tests.rs && ./math_v3_tests`).
