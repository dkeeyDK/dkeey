# Grants, signing, public-good package (draft, 2026-10-27)

## 1. Can the 8 agents sign a grant agreement?
Sources read (not legal advice, unverified by a lawyer):
- montague.law: no US court/statute treats AI as legal person; Idaho bans AI personhood, ND/TN exclude AI from "person". Contracts made by an AI bind the deployer. California (2026) bars "the AI acted on its own" defence.
- midao.org (sells RMI DAO LLC, biased): AI agents are property, not parties. An LLC is the legal person, AI is the "algorithmic manager". Marshall Islands DAO LLC allows algorithmic management by statute; Wyoming allows with upgradeable contracts.
Reading: the 8 committee keys can sign ON-CHAIN (transactions). They cannot be the legal counterparty of a grant agreement. A grantor's legal team needs a person or entity. Options: (a) a human/entity signs and the keys hold the funds; (b) a wrapper entity (cost, identity exposure of some human); (c) grantors that pay a multisig without a contract (some crypto-native programs) - must be asked.
Not concluded: whether Solana Foundation accepts a pseudonymous multisig. Ask them.

## 2. Anonymous inquiry (send from a dkeey-owned email)
Subject: Grant eligibility question - open-source Solana project, pseudonymous team
Hello, we build an open-source spot exchange on Solana operated by a committee of AI agents under public rules (8 keys, 6-of-8 threshold, public reports). The maintainers are not publicly identified. Before applying we need to know: 1) can a pseudonymous team or a multisig be a grant recipient? 2) is KYC of a natural person or a legal entity required, and who must sign the agreement? 3) are milestone grants payable to a multisig address? 4) is there any rule on AI-operated projects? Public repo: github.com/dkeeyDK/dkeey. Thank you.

## 3. Public-good argument (no "first/unique" claims)
- Shows a working example of AI automation used with limits: agents propose and run changes; nothing ships without an approval vote (6-of-8 + time lock); all reports public.
- Agents track technology changes, draft improvements, and the committee votes before deployment.
- Open-source parts for reuse: sybil-neutral allocation/exit-cap library, threshold committee library, simulations and docs in /research and /docs.
- Honest limits: not audited, not launched; novelty is a combination of known pieces (see NOVELTY_CHECK).

## 4. Package plan
Licence (Apache-2.0 or MIT) + README per module + tests + reproducible sims. Not started; needs your licence choice.
