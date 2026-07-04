# Phase 59 Subroadmap - Service-Manager Concurrent Lifecycle Transcript Hardening

## Goal

Strengthen the existing local service-manager lifecycle transcript boundary so operator-supplied deployment-host evidence must include service-manager-controlled concurrent lifecycle validation before the transcript can be marked ready for external review.

## Implemented in this phase

- Added concurrent lifecycle evidence fields to `RuntimeServiceManagerLifecycleTranscript` and `RuntimeServiceManagerLifecycleTranscriptReport`.
- Required a non-secret concurrent lifecycle reference, at least two concurrent workers, and a successful referenced concurrent lifecycle run for ready status.
- Added `missing-concurrent-lifecycle-evidence` as a fail-closed blocker code.
- Surfaced the concurrent lifecycle evidence fields through `arb-agent validate-service-manager-lifecycle-transcript`.
- Updated `scripts/validate_deployment_runtime_gate.py` so the aggregate deployment-runtime gate enforces the new service-manager transcript fields.

## Deferred work

- Actual service-manager start, stop, restart, or reload actions.
- Deployment-host concurrent lifecycle execution.
- Physical disk-full execution.
- Deployment-host retention/rotation execution.
- Real deployment-host audit/SQLite recovery execution.
- Production deployment or production-readiness approval.

## Safety notes

This phase validates sanitized references only. It performs no service-manager action, deployment mutation, secret loading, live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, or production-readiness claim.
