## Phase 87 - Deployment Observability Metrics Runtime Wrapper Gate

### Goal

Wire the bounded local observability metrics runtime validator into the deployment-host runtime wrapper and aggregate deployment-runtime gate so deployment-facing local evidence accounts for the same multi-scrape metrics runtime controls as the operator-surface gate, without daemon hosting, telemetry export, alert delivery, service-manager actions, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Added `--run-observability-metrics-runtime` and `--observability-metrics-workspace` to `scripts/validate_deployment_host_runtime.py`.
- Added deployment-host wrapper execution and JSON/text reporting for `arb-agent validate-observability-metrics-runtime`.
- Added explicit deployment-runtime aggregate enforcement for the observability metrics runtime wrapper report.
- Raised `scripts/validate_deployment_runtime_gate.py` to 36 total local runtime/deployment components and 23 nested runtime components.

### Explicit Non-Goals

- No daemon-hosted persistent metrics endpoint.
- No public network exposure.
- No telemetry export, log shipping, or outbound alert delivery.
- No deployment-host mutation or service-manager action.
- No external calls, live execution, signing, broadcasts, exchange calls, RPC calls, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_host_runtime.py --run-observability-metrics-runtime --observability-metrics-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for deployment-host wrapper and aggregate-gate coverage of the bounded local observability metrics runtime only. Real daemon-hosted metrics endpoint operation, exporter sessions, log shipping, outbound alert delivery, deployment-host service orchestration, incident drills, external AppSec review, and production readiness remain unclaimed.
