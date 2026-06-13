# PHASE_28_SUBROADMAP.md

## Phase

Phase 28 - Deployment Runtime Aggregate Gate

## Status

Implemented for local deterministic aggregate deployment-runtime validation only.

## Goal

Compose the existing local deployment/runtime probes into one stronger gate that
verifies runtime smoke, audit durability, audit retention execution, graceful
shutdown, backup/restore, backup/restore load, restart recovery, incomplete
recovery fail-closed behavior, supervised restart, permission-denial fail-closed
behavior, blocked state/audit preflights, filesystem and retention preflights,
communications, dashboard, observability, and panic-hook runtime boundaries
preserve safety invariants together.

## Scope

- Add `scripts/validate_deployment_runtime_gate.py`.
- Run the existing non-secret deployment-host runtime helper with all local-only
  runtime components enabled against fresh `target/` workspaces.
- Fail closed if any nested report claims service-manager action, external
  calls, live execution, secret loading, production readiness, public exposure,
  telemetry export, outbound alert/network delivery, or production-path mutation.
- Wire the aggregate gate into CI.
- Keep all production/deployment-host blockers open unless real external
  evidence exists.

## Explicit Non-Goals

- No live trading.
- No signing, withdrawals, bridges, broadcasts, or wallet custody.
- No real exchange, sandbox, RPC, dashboard-public, messaging, exporter, or
  alert-provider calls.
- No service installation, systemd reload, enable, start, stop, or restart.
- No deployment-state mutation outside the local `target/` validation workspace.
- No production readiness or live-funds readiness claim.

## Validation

Required after this phase:

```bash
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for the local aggregate deployment-runtime gate only when the script passes
locally, CI includes the gate, structure validation includes the new phase and
script, and governance files record that deployment-host/service-manager,
physical disk-full, real rollback/incident execution, external sandbox/live, and
production readiness evidence remain open.
