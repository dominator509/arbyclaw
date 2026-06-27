# Expanded local DEX/Web3 protocol-risk governance review

- Extended `DexProtocolRiskReviewRequest` with local-only governance/scope fields: `chain_allowlisted`, `pair_allowlisted`, `token_contract_reviewed`, `token_decimals_verified`, `jurisdiction_reviewed`, and `incident_reputation_reviewed`.
- Extended `DexProtocolRiskReviewReport` with `asset_scope_passed`, `token_hygiene_passed`, and `governance_review_passed`.
- Review now blocks on `chain-not-allowlisted`, `pair-not-allowlisted`, `token-contract-not-reviewed`, `token-decimals-not-verified`, `jurisdiction-not-reviewed`, and `incident-reputation-not-reviewed` in addition to prior router/spender/gas/MEV/terms checks.
- Updated `arb-agent validate-dex-protocol-risk-review` ready-path assertions and blocked-path blocker count from 10 to 16.
- Reconciled Phase 37 / roadmap / architecture / tracker wording so GAP-0051 now reads as an external validation gap with stronger local deterministic coverage already present.
- Validation on 2026-06-13: targeted protocol-risk tests and CLI gate passed, followed by full structure/fmt/check/test/clippy pass.