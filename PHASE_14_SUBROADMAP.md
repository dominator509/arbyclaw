# PHASE_14_SUBROADMAP.md

## Phase 14 — Observability and Runbooks

## Governance Status

Created before Phase 14 implementation, after rereading and reconciling:

1. `ARCHITECTURE.md`
2. `ROADMAP.md`
3. `PHASE_13_SUBROADMAP.md`
4. `AGENTS.md`
5. `PRODUCTION_GAP_TRACKER.md`
6. `HANDOFF_CONTEXT.md`
7. `STRUCTURE_MANIFEST.md`

## Baseline Reconciliation

The authoritative baseline for Phase 14 is the Phase 13 embedded-dashboard snapshot.

Completed phases confirmed:

- Phase 0 — Governance Initialization
- Phase 1 — Rust Workspace Scaffold
- Phase 2 — Config, Secrets, and Mode Gates
- Phase 3 — Policy Engine and Trust Contract
- Phase 4 — Audit Journal and State Store Boundary
- Phase 5 — Market Data Core and Fee Models
- Phase 6 — Simulated/Paper Connectors
- Phase 7 — CEX Connector Framework
- Phase 8 — DEX/Web3 Connector Framework
- Phase 9 — Opportunity Engine
- Phase 10 — Execution Planner
- Phase 11 — Execution Adapters
- Phase 12 — Communications and CLI
- Phase 13 — Embedded Dashboard

Current production readiness entering Phase 14: 76% governance estimate only.

## Phase Goal

Add deterministic observability and runbook model/trait boundaries, including local operations review records for retention and alert-route controls, endpoint/exporter preflight records for loopback/auth/transport/redaction/alert-route/backpressure accounting, ephemeral numeric-loopback bind validation records, authenticated metrics scrape preflight records, and one-shot loopback metrics endpoint validation, without starting long-lived telemetry services, exposing public metrics endpoints, sending alerts, exporting telemetry, or capturing secrets.

## In Scope

- Local-only observability configuration models.
- Metrics endpoint binding model that is fail-closed by default.
- Health component status models.
- Structured log event models with redaction.
- Metric sample models with deterministic labels and integer micro-unit values.
- Operator runbook models.
- Observability snapshot and collection request/record models.
- Deterministic local collector trait implementation.
- Local audit-journal and SQLite WAL checkpoint helpers for sanitized collection outcomes.
- Local non-network metrics/export and alert-route dry-run records.
- Local alert-route dispatch review records wired through the deterministic communications notification boundary, with audit/state checkpointing and no outbound delivery.
- Local ephemeral numeric-loopback bind validation that opens and closes a listener without serving requests.
- Local authenticated metrics scrape preflight over rendered metric lines plus one-shot authenticated loopback socket scrape validation.
- Secret-like text detection and redaction before records are returned.
- CLI status text indicating observability boundary availability.
- Structure validator update for Phase 14 files.
- Governance documentation updates.
- Gap tracker updates for deferred Rust validation and real observability runtime work.

## Explicitly Out of Scope

- Live trading.
- Signing.
- Withdrawals.
- Bridges.
- Broadcasts.
- Real CEX calls.
- Real DEX/RPC calls.
- Public metrics endpoints.
- HTTP server startup.
- OpenTelemetry exporters.
- Prometheus scraping endpoint.
- Log shipping.
- SIEM integrations.
- PagerDuty, Slack, email, webhook, SMS, or other outbound alert delivery.
- Panic hooks that exfiltrate process state.
- Secret capture, secret logging, credential telemetry, wallet-key telemetry, or provider-token telemetry.
- Runtime process supervision.
- Production incident automation.

## Dependencies and Preconditions

- Phase 13 completed.
- `PHASE_14_SUBROADMAP.md` exists before code changes.
- Structure validator passes before implementation.
- Existing no-secret scanning remains enabled.
- Rust validation remains environment-limited in ChatGPT Project Mode.

## Subsystem Boundary

Phase 14 may add only:

- `arb-core::observability`
- `ObservabilityBoundaryConfig`
- `ObservabilityEndpointBinding`
- `HealthStatus`
- `ComponentHealthStatus`
- `StructuredLogEvent`
- `MetricSample`
- `Runbook`
- `ObservabilitySnapshot`
- `ObservabilityCollectionRequest`
- `ObservabilityRecord`
- `ObservabilityExportDryRunRequest`
- `ObservabilityExportDryRunReport`
- `ObservabilityCollector`
- `DeterministicObservabilityCollector`

Phase 14 must not mutate planner, execution adapter, communications, dashboard, policy, secrets, or connector behavior except for safe exports/status text.

## Implementation Sequence

1. Re-run baseline structure validator.
2. Create this sub-roadmap.
3. Add `crates/arb-core/src/observability.rs` as a local-only model/trait module.
4. Export observability types from `arb-core`.
5. Update `arb-agent` status output.
6. Update `scripts/validate_structure.py` to require Phase 14 files.
7. Update `ARCHITECTURE.md`, `ROADMAP.md`, `README.md`, `SECURITY.md`, `AGENTS.md`, `HANDOFF_CONTEXT.md`, and `PRODUCTION_GAP_TRACKER.md`.
8. Regenerate `STRUCTURE_MANIFEST.md`.
9. Run available validation.
10. Package a commit-ready ZIP.

## Validation Plan

Run in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempt but do not claim as passed unless available:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Phase 14 is complete for ChatGPT Project Mode when:

- `PHASE_14_SUBROADMAP.md` exists.
- `crates/arb-core/src/observability.rs` exists.
- Observability types are exported from `arb-core`.
- `arb-agent` reports the observability boundary without starting endpoints.
- Metrics endpoint exposure is denied in the model boundary.
- Outbound alert delivery is denied in the model boundary.
- Observability collection outcomes can be locally journaled and checkpointed, then recovered after audit/SQLite reopen.
- Secret-like observability text is redacted before records are returned.
- Structure validator passes.
- Gap tracker records deferred Rust validation and real observability runtime work.

## Rollback Plan

To rollback Phase 14:

1. Remove `crates/arb-core/src/observability.rs`.
2. Remove observability exports from `crates/arb-core/src/lib.rs`.
3. Remove observability status text from `crates/arb-agent/src/main.rs`.
4. Remove Phase 14 requirements from `scripts/validate_structure.py`.
5. Revert governance docs to Phase 13 state.
6. Re-run `python3 scripts/validate_structure.py`.

## Completion Status

Completed for ChatGPT Project Mode after Phase 14 implementation and validation, including local sanitized collection, operations review, sandbox-only observability log retention/rotation execution, export dry-run, endpoint/exporter preflight, ephemeral loopback bind validation, authenticated metrics scrape preflight, one-shot loopback metrics endpoint validation, scoped local tracing subscriber capture, scoped panic-hook, runtime failure-capture audit/state boundaries, and a repeatable local `validate-observability-runtime` CLI gate.

## Deferred Work

- Keep Rust/Cargo validation current after future changes.
- Daemon-wide/deployment-host tracing/logging subscriber installation beyond the scoped local capture.
- Prometheus/OpenTelemetry exporter session implementation beyond local dry-run rendering.
- Daemon-hosted metrics endpoint authentication, long-running scrape behavior, and deployment-host endpoint serving validation.
- Real alert delivery through authenticated communications adapters beyond the local deterministic notification-boundary bridge.
- Deployment-host log retention/rotation execution and privacy policy validation beyond sandbox-only local execution.
- Deployment-host/runtime panic-hook integration and failure-capture hardening under real service orchestration beyond the local panic-hook helpers.
- Production runbook exercises and incident drills.
