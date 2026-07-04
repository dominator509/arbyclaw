# PHASE_71_SUBROADMAP.md

## Phase 71 - Local Opportunity Replay Latency Review Gate

### Goal

Promote existing local opportunity replay load evidence into an explicit local latency and throughput review so the opportunity scenario aggregate gate can account for ranking/replay latency budgets and scenario/candidate breadth without downloading market data, calling exchanges/RPCs, invoking external fuzzers, submitting adapters, signing, broadcasting, executing live orders, or claiming production readiness.

### Completed Tasks

- Added `OpportunityReplayLatencyReviewRequest`, `OpportunityReplayLatencyReviewReport`, and `OpportunityReplayLatencyReviewStatus`.
- Added `review_opportunity_replay_latency` with local latency budget and scenario/candidate throughput checks.
- Rejected external calls, external data downloads, live execution, and production-readiness claims.
- Surfaced the review through `arb-agent validate-opportunity-replay --iterations <n>`.
- Added `scripts/validate_opportunity_scenario_gate.py` assertions for the replay latency review.
- Added focused Rust tests for ready, blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No external fuzzer execution.
- No live/provider market-data download.
- No exchange/RPC calls.
- No adapter submission, signing, broadcast, bridge, withdrawal, or live execution.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core opportunity_replay_latency -- --nocapture
cargo run -p arb-agent -- validate-opportunity-replay --iterations 2
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local opportunity replay latency and throughput review only. Broader external/deployment scenario-corpus execution, external fuzz engines, live/provider-backed market-data validation, deployment-host resource profiling, production load tests, penetration tests, production backtest evidence, and production readiness remain unclaimed.
