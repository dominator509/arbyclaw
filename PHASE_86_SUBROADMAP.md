## Phase 86 - Local Observability Metrics Runtime Gate

### Goal

Add bounded loopback-only observability metrics runtime validation so observability evidence moves beyond one-shot metrics endpoint validation to a multi-scrape local listener lifecycle with audit replay, SQLite checkpoint recovery, response consistency checks, clean shutdown, public exposure denial, telemetry-export denial, outbound-alert denial, and operator-surface aggregate enforcement without daemon hosting, exporter sessions, alert delivery, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Added `ObservabilityMetricsRuntimeProbe`, `ObservabilityMetricsRuntimeProbeReport`, and `ObservabilityMetricsRuntimeProbeStatus`.
- Added `validate_observability_metrics_runtime_probe` to serve multiple authenticated loopback `/metrics` scrapes on one bounded listener and verify response status, metric-line consistency, listener startup, and shutdown.
- Added `append_observability_metrics_runtime_probe_audit` and `persist_observability_metrics_runtime_probe_checkpoint`.
- Added `arb-agent validate-observability-metrics-runtime --workspace <fresh-dir>`.
- Added focused core and agent tests for audit replay and SQLite checkpoint recovery.
- Added `observability_metrics_runtime_cli` to `scripts/validate_operator_surface_gate.py`, raising the aggregate to 10 local operator-surface components.

### Explicit Non-Goals

- No public metrics endpoint exposure.
- No persistent daemon-hosted observability endpoint.
- No Prometheus/OpenTelemetry exporter session.
- No log shipping or outbound alert delivery.
- No deployment-host service-manager action.
- No live controls, external submission, live execution, signing, broadcasts, exchange calls, RPC calls, wallet custody, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core observability_metrics_runtime -- --nocapture
cargo test -p arb-agent observability_metrics_runtime -- --nocapture
cargo run -p arb-agent -- validate-observability-metrics-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for bounded local loopback observability metrics runtime validation only. Real daemon-hosted metrics endpoint operation, exporter sessions, log shipping, outbound alert delivery, deployment-host orchestration, incident drills, external AppSec review, and production readiness remain unclaimed.
