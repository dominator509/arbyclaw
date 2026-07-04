# Phase 68 - Local Service-Manager Lifecycle Rehearsal Gate

## Goal

Add an executable local-only service-manager lifecycle rehearsal validator so the runtime gate can prove ordered start, smoke, graceful shutdown, stop, restart, and recovery evidence semantics without touching real services.

## Scope

- Validate sanitized lifecycle event order.
- Require operator-controlled event references and successful outcomes.
- Require audit replay, SQLite recovery, graceful-shutdown checkpoint, restart recovery, concurrent lifecycle, operator approval, and reviewer approval references.
- Expose `arb-agent validate-service-manager-lifecycle-rehearsal`.
- Include the validator in local deployment runtime and evidence bundle gates.

## Non-Goals

- No `systemctl` calls or real service-manager actions.
- No deployment path mutation.
- No secret loading.
- No external submission, live execution, or production-readiness claim.

## Validation

- `cargo test -p arb-core service_manager_lifecycle_rehearsal -- --nocapture`
- `cargo run -p arb-agent -- validate-service-manager-lifecycle-rehearsal`
- `python scripts/validate_deployment_runtime_gate.py --json`
- Full workspace gates before commit/push.
