# Phase 104 - Fee Live Provider Boundary Gate

## Scope

- Replace the loose external fee/account-tier/gas/withdrawal blocker with a typed local fee live-provider boundary review.
- Compose existing local fee schedule verification and reconciliation evidence into one provider-backed fee validation gate.
- Require the new boundary in the connector scenario aggregate gate.
- Preserve no live provider calls, RPC calls, credential loading, signing, broadcasts, withdrawals, live execution, or production-readiness claims.

## Implementation

- Added `FeeLiveProviderBoundaryReviewRequest`, `FeeLiveProviderBoundaryReviewReport`, and `FeeLiveProviderBoundaryReviewStatus`.
- Added `review_fee_live_provider_boundary()` with explicit blockers for missing provider-backed maker/taker fee, account-tier, gas/RPC/network fee, and withdrawal-cost evidence.
- Added focused unit tests for blocked-pending-provider fee validation and side-effect fail-closed behavior.
- Added `arb-agent validate-fee-live-provider-boundary`.
- Wired `fee_live_provider_boundary` into `scripts/validate_connector_scenario_gate.py`, raising the aggregate to 23 local components.

## Validation

Required local validation for this phase:

```text
cargo test -p arb-core fee_live_provider_boundary -- --nocapture
cargo test -p arb-agent fee_live_provider_boundary -- --nocapture
cargo run -p arb-agent -- validate-fee-live-provider-boundary
python scripts/validate_connector_scenario_gate.py --json
python scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Remaining Blockers

- This phase is local-only. Real provider/API fee validation, external account-tier confirmation, gas/RPC/network fee validation, withdrawal-cost validation, credentials, exchange/RPC calls, deployment-host validation, and production readiness remain incomplete.
