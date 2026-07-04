# PHASE_76_SUBROADMAP.md

## Phase 76 - Local Deployment Graceful-Shutdown Transcript Gate

### Goal

Add a typed local deployment-host graceful-shutdown transcript validator so deployment-runtime evidence can account for service-lifecycle context, shutdown request references, stop/quiesce observation references, graceful-shutdown checkpoint references, audit replay after shutdown, SQLite reopen after shutdown, restart recovery after shutdown, post-shutdown runtime smoke evidence, and reviewer/operator approval without stopping services, calling service managers, mutating deployment paths, loading secrets, submitting adapters, performing live execution, or claiming production readiness.

### Completed Tasks

- Added `RuntimeDeploymentGracefulShutdownTranscript`, `RuntimeDeploymentGracefulShutdownTranscriptReport`, and `RuntimeDeploymentGracefulShutdownTranscriptStatus`.
- Added `validate_deployment_graceful_shutdown_transcript` with ready/blocked status and non-secret blocker codes.
- Required deployment-host, service-lifecycle, shutdown-request, service-stopped, graceful-shutdown checkpoint, audit replay, SQLite reopen, restart recovery, post-shutdown smoke, operator, and reviewer evidence references.
- Rejected validator-performed service-manager actions, deployment path mutation, secret loading, external submission, live execution, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-deployment-graceful-shutdown-transcript`.
- Added the new transcript to `scripts/validate_deployment_runtime_gate.py`, `scripts/validate_deployment_evidence_bundle.py`, and `scripts/validate_deployment_evidence_checklist.py`.
- Added focused local Rust tests for ready, missing-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No service stop/start/restart execution.
- No service-manager actions.
- No deployment path mutation.
- No secret loading.
- No external calls, adapter submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_graceful_shutdown_transcript -- --nocapture
cargo run -p arb-agent -- validate-deployment-graceful-shutdown-transcript
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deployment graceful-shutdown transcript validation only. Actual service-manager-controlled graceful shutdown, deployment-host stop/start/restart execution, deployment-host recovery behavior, production audit/SQLite recovery, and production readiness remain unclaimed.
