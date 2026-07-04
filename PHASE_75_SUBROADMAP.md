# PHASE_75_SUBROADMAP.md

## Phase 75 - Local Deployment Backup/Restore Transcript Gate

### Goal

Add a typed local deployment-host backup/restore transcript validator so deployment-runtime evidence can account for service-lifecycle context, backup artifact references, restore execution references, deployment-load references, audit replay/hash-chain continuity, SQLite restore checks, runtime checkpoint restoration, post-restore smoke evidence, rollback references, and reviewer/operator approval without executing backup/restore actions, mutating deployment paths, calling service managers, loading secrets, submitting adapters, performing live execution, or claiming production readiness.

### Completed Tasks

- Added `RuntimeDeploymentBackupRestoreTranscript`, `RuntimeDeploymentBackupRestoreTranscriptReport`, and `RuntimeDeploymentBackupRestoreTranscriptStatus`.
- Added `validate_deployment_backup_restore_transcript` with ready/blocked status and non-secret blocker codes.
- Required deployment-host, service-lifecycle, backup-artifact, restore-execution, deployment-load, audit-restore, SQLite-restore, runtime-checkpoint-restore, post-restore-smoke, rollback, runbook, operator, and reviewer evidence references.
- Rejected validator-performed backup/restore execution, service-manager actions, deployment path mutation, secret loading, external submission, live execution, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-deployment-backup-restore-transcript`.
- Added the new transcript to `scripts/validate_deployment_runtime_gate.py`, `scripts/validate_deployment_evidence_bundle.py`, and `scripts/validate_deployment_evidence_checklist.py`.
- Added focused local Rust tests for ready, missing-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No real backup execution.
- No real restore execution.
- No service-manager actions.
- No deployment path mutation.
- No secret loading.
- No external calls, adapter submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_backup_restore_transcript -- --nocapture
cargo run -p arb-agent -- validate-deployment-backup-restore-transcript
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

Met for local deployment backup/restore transcript validation only. Actual deployment-host backup execution, restore execution, service-manager-controlled recovery, deployment-load restore validation, production audit/SQLite recovery, rollback execution, and production readiness remain unclaimed.
