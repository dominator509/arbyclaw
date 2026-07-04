## Phase 78 - Local Market-Data Bad-Data Rejection Review Gate

### Goal

Add a typed local market-data bad-data rejection review so connector evidence can account for stale-data rejection, excessive-spread rejection, insufficient-depth rejection, capture-latency rejection, acceptable baseline evidence, and unresolved external provider evidence without live REST/WebSocket calls, provider credentials, WebSocket connections, adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Completed Tasks

- Added `MarketDataBadDataRejectionReviewRequest`, `MarketDataBadDataRejectionReviewReport`, and `MarketDataBadDataRejectionReviewStatus`.
- Added `review_market_data_bad_data_rejection` over existing local quality assessment evidence.
- Required acceptable baseline quality, stale-data rejection, excessive-spread rejection, insufficient-depth rejection, capture-latency rejection, bad-data fixture floor, and remaining-external-evidence fields before the review reports ready for local review.
- Rejected live network use, WebSocket opening, credential loading, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-market-data-bad-data-rejection`.
- Added the new CLI to `scripts/validate_connector_scenario_gate.py`, raising the connector scenario aggregate to 18 local components.
- Added focused local Rust tests for ready, missing fixture-floor, and fail-closed side-effect cases.

### Explicit Non-Goals

- No live REST/WebSocket provider implementation.
- No provider credentials or account queries.
- No external latency or data-quality measurement.
- No real historical dataset download.
- No adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core market_data_bad_data_rejection -- --nocapture
cargo run -p arb-agent -- validate-market-data-bad-data-rejection
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local market-data bad-data rejection review only. Live REST/WebSocket providers, provider-backed bad-data rejection validation, external latency/data-quality evidence, real historical dataset validation, sandbox/read-only provider validation, and production readiness remain unclaimed.
