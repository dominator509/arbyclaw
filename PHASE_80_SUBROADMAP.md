## Phase 80 - Local Runtime Config Reload Validation Gate

### Goal

Add local runtime config reload validation so deployment/runtime evidence can account for safe reload parsing, non-live mode enforcement, local allowlist change detection, deployment-host wrapper reporting, and aggregate deployment-runtime enforcement without reloading service managers, starting services, loading secrets, submitting adapters, performing live execution, or claiming production readiness.

### Completed Tasks

- Added `RuntimeConfigReloadValidationRequest`, `RuntimeConfigReloadValidationReport`, and `RuntimeConfigReloadStatus`.
- Added `validate_runtime_config_reload` over two already parsed `AgentConfig` values.
- Required safe initial and reloaded modes, detected local config changes, CEX allowlist change detection, asset allowlist change detection, and no side-effect flags before the report is ready for local review.
- Surfaced the gate through `arb-agent validate-runtime-config-reload --workspace <fresh-dir>`.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-runtime-config-reload`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`, raising the aggregate to 32 local runtime/deployment components.
- Added focused local Rust tests for ready, unchanged-config blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core runtime_config_reload -- --nocapture
cargo test -p arb-agent runtime_config_reload -- --nocapture
cargo run -p arb-agent -- validate-runtime-config-reload --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-runtime-config-reload --runtime-config-reload-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local runtime config reload validation only. Service-manager-controlled reload, deployment-host config reload, daemon uptime soak, start/stop/restart validation, production filesystem behavior, real observability smoke, and production readiness remain unclaimed.
