## Phase 77 - Local Market-Data Provider Reconciliation Review Gate

### Goal

Add a typed local market-data provider rate-limit/outage reconciliation review so connector evidence can account for degraded provider preflight, retry-after/backoff handling, outage retry exhaustion, stale-data blocking, latency blocking, and unresolved external provider evidence without live REST/WebSocket calls, provider credentials, WebSocket connections, adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Completed Tasks

- Added `MarketDataProviderReconciliationReviewRequest`, `MarketDataProviderReconciliationReviewReport`, and `MarketDataProviderReconciliationReviewStatus`.
- Added `review_market_data_provider_reconciliation` over existing local latency/backpressure review, degraded provider preflight, ready rate-limit reconnect plan, and blocked outage reconnect plan.
- Required local rate-limit, outage, stale-data, latency, degraded-sample, retry-after/backoff, outage-exhaustion, and remaining-external-evidence fields before the review reports ready for local review.
- Rejected live network use, WebSocket opening, credential loading, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-market-data-provider-reconciliation`.
- Added the new CLI to `scripts/validate_connector_scenario_gate.py`, raising the connector scenario aggregate to 17 local components.
- Added focused local Rust tests for ready, missing-outage-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No live REST/WebSocket provider implementation.
- No provider credentials or account queries.
- No provider-backed reconnect loops.
- No external latency measurement.
- No adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core market_data_provider_reconciliation_review -- --nocapture
cargo run -p arb-agent -- validate-market-data-provider-reconciliation
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local market-data provider rate-limit/outage reconciliation review only. Live REST/WebSocket providers, provider-backed reconnect/rate-limit/outage validation, external latency/data-quality evidence, deployment-host resource profiling, sandbox/read-only provider validation, and production readiness remain unclaimed.
