## Phase 90 - Deployment Host Static Hardening Config Smoke Runtime Gate

### Goal

Wire static deployment hardening/config-loading smoke validation into the deployment-host runtime wrapper and aggregate deployment-runtime gate so deployment-facing runtime evidence includes the committed example config loading through the real `arb-agent --config` path, while preserving local-only execution, no service-manager actions, no external calls, no live execution, and no production-readiness claims.

### Completed Tasks

- Add `--run-deployment-static-hardening` to `scripts/validate_deployment_host_runtime.py`.
- Run `scripts/validate_deployment_static_hardening.py --run-config-smoke --json` through the deployment-host runtime wrapper.
- Surface config smoke pass/fail, safe mode, live-execution denial, service-action denial, network-listener denial, and secret-loading denial fields in the wrapper report.
- Require the new wrapper component in `scripts/validate_deployment_runtime_gate.py`.
- Update governance docs and gap tracker after validation.

### Explicit Non-Goals

- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
- No production config mutation or deployment-host path mutation.
- No public network exposure.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_host_runtime.py --run-deployment-static-hardening --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met when the deployment-host runtime wrapper and aggregate deployment-runtime gate require static deployment hardening/config smoke validation and fail closed if config loading, safe-mode, live-execution denial, service-action denial, network-listener denial, or secret-loading denial evidence is missing.
