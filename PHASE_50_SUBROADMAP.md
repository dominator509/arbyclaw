# Phase 50 Subroadmap - Local Service-Manager Lifecycle Transcript Validation

## Scope

Add a typed local validator for sanitized operator-owned service-manager lifecycle transcript metadata. The validator checks that lifecycle evidence includes start, runtime-smoke, graceful-shutdown, stop, restart, and recovery references without performing service actions.

## Implemented

- Added service-manager lifecycle transcript types and status enums in the runtime core.
- Added `validate_service_manager_lifecycle_transcript` for local-only transcript validation.
- Added `arb-agent validate-service-manager-lifecycle-transcript` with ready and blocked fixture paths.
- Added CI coverage through the Rust validation workflow.
- Added unit tests for complete evidence, missing restart/recovery evidence, and validator service-action denial.

## Deferred

- Actual deployment-host service start, stop, restart, and graceful-shutdown execution.
- Real deployment-host audit/SQLite recovery collection.
- Production service-manager orchestration under load.
- Operator-owned external evidence capture and independent review.

## Safety

This phase performs no service installation, service start/stop/restart, systemd daemon reload, production-path mutation, live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, or secret handling.
