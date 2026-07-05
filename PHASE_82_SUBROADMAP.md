## Phase 82 - Local Deployment Config Redaction Gate

### Goal

Add local deployment config loading and audit-redaction validation so deployment/runtime evidence can account for safe config parsing, audit redaction enforcement, unsafe secret-like metadata rejection, redacted audit append/replay, deployment-host wrapper reporting, and aggregate deployment-runtime enforcement without starting services, mutating deployment hosts, loading secrets, calling external systems, performing live execution, or claiming production readiness.

### Completed Tasks

- Added `arb-agent validate-deployment-config-redaction --workspace <fresh-dir>`.
- Added a local non-secret deployment config fixture that loads through the existing config parser with paper mode, kill switch enabled, withdrawals disabled, live execution disabled, disabled secret backends, and `audit.redact_secrets = true`.
- Exercised the actual append-only audit journal validation path by rejecting unsafe secret-like metadata.
- Appended a redacted configuration audit event and verified audit replay after reopen.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-deployment-config-redaction`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`, raising the aggregate to 34 local runtime/deployment components.
- Added focused local Rust tests for the CLI validation path.

### Explicit Non-Goals

- No deployment-host config loading under a service manager.
- No deployment-host log/audit redaction test under a deployed service.
- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-agent deployment_config_redaction -- --nocapture
cargo run -p arb-agent -- validate-deployment-config-redaction --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-deployment-config-redaction --deployment-config-redaction-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deployment config loading and audit-redaction fixture validation only. Deployment-host config loading under service orchestration, deployed log/audit redaction, startup/shutdown/restart, service hardening, production filesystem behavior, and production readiness remain unclaimed.
