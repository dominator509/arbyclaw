# Phase 64 Subroadmap - Observability Runtime Preconditions

## Goal

Strengthen the deterministic observability operations boundary so local observability runtime validation records the non-secret preconditions that any future exporter, alert, or daemon-hosted observability runtime must preserve before real observability integration can be wired.

## Implemented in this phase

- Added future-runtime precondition flags to `ObservabilityOperationsPolicy`.
- Required audit/state preflight, exporter kill-switch controls, alert authorization, rate-limit/backpressure controls, retry/backoff controls, and non-secret telemetry controls before operations reviews can report ready for local review.
- Added matching durable fields to `ObservabilityOperationsReviewReport`.
- Persisted the precondition fields in operations-review audit metadata and SQLite checkpoints.
- Surfaced the precondition fields in `arb-agent validate-observability-runtime`.
- Updated `scripts/validate_operator_surface_gate.py` so the operator-surface aggregate gate enforces the new observability precondition fields.
- Wired the same preconditions into runtime-smoke observability operations records.

## Deferred work

- Real daemon-hosted observability runtime.
- Real metrics exporter sessions.
- Real log shipping.
- Real alert delivery.
- Real daemon-wide tracing or panic-hook installation under deployment orchestration.
- Deployment-host retention/rotation execution.
- Incident drills and production observability validation.

## Safety notes

This phase adds local deterministic validation only. It performs no telemetry export, outbound alert delivery, public exposure, service-manager action, live trading, adapter submission, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, secret loading, or production-readiness claim.
