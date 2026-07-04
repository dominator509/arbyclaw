# PHASE_74_SUBROADMAP.md

## Phase 74 - Local Hosted Dashboard Runtime Readiness Review Gate

### Goal

Promote existing local hosted-dashboard security, request-preflight, one-shot loopback request, and session-validation evidence into an explicit local hosted-dashboard runtime readiness review so the operator-surface aggregate gate can account for authentication, authorization, CSRF, secure-header, rejection-accounting, loopback serving, and unresolved external hosting evidence without starting a persistent server, exposing public bindings, enabling live controls, loading secrets, submitting adapters, executing live actions, or claiming production readiness.

### Completed Tasks

- Added `DashboardHostedRuntimeReadinessReviewRequest`, `DashboardHostedRuntimeReadinessReviewReport`, and `DashboardHostedRuntimeReadinessReviewStatus`.
- Added `review_dashboard_hosted_runtime_readiness` to compose hosted-security, hosted-request preflight, and hosted-session reports.
- Required accepted-request, unauthenticated-rejection, CSRF-rejection, rate-limit-rejection, loopback-serving, secure-header, and remaining-external-evidence checks.
- Rejected persistent server startup, public network exposure, live controls, and production-readiness claims.
- Surfaced the review through `arb-agent validate-dashboard-runtime`.
- Added the new readiness assertions to `scripts/validate_operator_surface_gate.py` and `scripts/validate_deployment_host_runtime.py`.
- Added focused local Rust tests for ready, missing-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No persistent dashboard server.
- No public network exposure.
- No real browser session hosting.
- No production authentication/session/authorization implementation.
- No CSRF-token issuing from a daemon.
- No live controls, adapter submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core hosted_dashboard_runtime_readiness -- --nocapture
cargo run -p arb-agent -- validate-dashboard-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local hosted-dashboard runtime readiness review only. Persistent daemon hosting, browser authentication/session handling, CSRF token serving, secure-header serving from a live server, public-exposure validation, penetration testing, deployment-host orchestration, and production readiness remain unclaimed.
