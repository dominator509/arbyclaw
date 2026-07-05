## Phase 79 - Local Fee Schedule Reconciliation Review Gate

### Goal

Add a typed local fee schedule reconciliation review so connector evidence can account for current fee-review readiness, unverified-schedule rejection, maker/taker tier rejection, network/gas fee rejection, withdrawal-fee rejection, stale-review rejection, and unresolved external fee evidence without provider API calls, RPC calls, credential loading, account queries, signing, broadcasts, live execution, or production-readiness approval.

### Completed Tasks

- Added `FeeScheduleReconciliationReviewRequest`, `FeeScheduleReconciliationReviewReport`, and `FeeScheduleReconciliationReviewStatus`.
- Added `review_fee_schedule_reconciliation` over existing local current and blocked fee verification reports.
- Required current fee-review readiness, unverified-schedule rejection, maker/taker rejection, network/gas fee rejection, withdrawal-fee rejection, stale-review rejection, and remaining-external-evidence fields before the review reports ready for local review.
- Rejected live provider calls, credential loading, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-fee-schedule-reconciliation`.
- Added the new CLI to `scripts/validate_connector_scenario_gate.py`, raising the connector scenario aggregate to 19 local components.
- Added focused local Rust tests for ready, missing external-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No provider API calls or account queries.
- No exchange credentials or wallet credentials.
- No RPC/gas provider calls.
- No external fee schedule retrieval.
- No adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core fee_schedule_reconciliation -- --nocapture
cargo test -p arb-agent fee_schedule_reconciliation -- --nocapture
cargo run -p arb-agent -- validate-fee-schedule-reconciliation
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local fee schedule reconciliation review only. Real account-tier reconciliation, provider/API fee validation, gas/RPC fee validation, withdrawal-cost verification, external fee schedule review, and production readiness remain unclaimed.
