# Phase 7 CEX Governance Gate

- Added local CLI `arb-agent validate-cex-governance-review`.
- Gate composes two local-only review surfaces:
  - `validate_cex_credential_scope_review` over sanitized `SecretRef` metadata and permissions
  - `validate_cex_rate_limit` over caller-supplied local rate-limit observations
- Credential-scope review now also models local governance metadata flags for:
  - fee schedule review
  - rate-limit documentation review
  - terms-of-service review
  - jurisdiction review
  - API capability review
  - incident/reputation review
- Missing governance metadata blocks the report fail-closed but does not make the input invalid.
- CI aggregate wiring now includes the CEX governance gate in `scripts/validate_connector_scenario_gate.py` and `.github/workflows/ci.yml`.
- Repo-local Obsidian project note path was not present at `C:\Users\domin\Documents\Obsidian\Projects\arbyclaw` during this pass, so Serena memory remains the active project-memory surface for this environment.
