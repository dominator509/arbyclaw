# PHASE_73_SUBROADMAP.md

## Phase 73 - Local Validation Coverage Review Gate

### Goal

Promote existing local validation-run, property-check, fuzz-corpus replay, validation-corpus, and paper-backtest evidence into an explicit local coverage review so the opportunity aggregate gate can account for local validation breadth and unresolved external validation evidence without invoking external fuzzers, downloading external data, opening live network providers, submitting adapters, signing, broadcasting, executing live orders, or claiming production readiness.

### Completed Tasks

- Added `LocalValidationCoverageReviewRequest`, `LocalValidationCoverageReviewReport`, and `LocalValidationCoverageReviewStatus`.
- Added `review_local_validation_coverage` with local validation-plan, property-check, fuzz-target, validation-corpus, paper-backtest, and remaining-external-evidence checks.
- Rejected live network use, external fuzzer invocation, live execution submission, signing/broadcast, and production-readiness claims.
- Surfaced the review through `arb-agent validate-local-validation-coverage-review`.
- Added aggregate opportunity scenario gate assertions to `scripts/validate_opportunity_scenario_gate.py`.
- Added focused local Rust tests for ready, missing-breadth, and fail-closed side-effect cases.

### Explicit Non-Goals

- No external fuzzing-engine execution.
- No external data download.
- No provider-backed market-data or exchange/RPC calls.
- No adapter submission, signing, broadcast, bridge, withdrawal, or live execution.
- No production load, security, or external backtest execution.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core local_validation_coverage_review -- --nocapture
cargo run -p arb-agent -- validate-local-validation-coverage-review
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local validation coverage review only. External fuzz/property execution, broader external/deployment replay corpora, provider-backed market-data validation, production load/security validation, external backtest execution, deployment-host validation, and production readiness remain unclaimed.
