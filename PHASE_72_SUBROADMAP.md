# PHASE_72_SUBROADMAP.md

## Phase 72 - Local Market-Data Provider Latency Review Gate

### Goal

Promote existing local market-data provider preflight, reconnect/backoff, quality-assessment, and paid-provider dossier evidence into an explicit local latency/backpressure review so the connector aggregate gate can account for provider receive latency, capture latency, reconnect delay, sample floor, and unresolved external evidence without opening REST/WebSocket sessions, loading credentials, measuring live providers, submitting adapters, signing, broadcasting, executing live orders, or claiming production readiness.

### Completed Tasks

- Added `MarketDataProviderLatencyReviewRequest`, `MarketDataProviderLatencyReviewReport`, and `MarketDataProviderLatencyReviewStatus`.
- Added `review_market_data_provider_latency` with local provider latency, capture latency, reconnect-delay, quality-score, sample-floor, and remaining-external-evidence checks.
- Rejected live network use, WebSocket opening, credential loading, and production-readiness claims.
- Surfaced the review through `arb-agent validate-market-data-provider-preflight`.
- Added `scripts/validate_connector_scenario_gate.py` assertions for the provider latency review.
- Added focused Rust tests for ready, blocked-budget, and fail-closed side-effect cases.

### Explicit Non-Goals

- No live REST/WebSocket provider implementation.
- No provider-backed latency measurement.
- No provider account, credential, or API-key handling.
- No exchange/RPC calls.
- No adapter submission, signing, broadcast, bridge, withdrawal, or live execution.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core market_data_provider_latency_review -- --nocapture
cargo run -p arb-agent -- validate-market-data-provider-preflight
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local market-data provider latency/backpressure review only. Live REST/WebSocket providers, provider-backed reconnect/rate-limit/outage reconciliation, external latency measurement, paid-provider integration, real market-data quality evidence, deployment-host resource profiling, broader external validation, and production readiness remain unclaimed.
