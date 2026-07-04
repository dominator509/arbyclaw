# Phase 61 Subroadmap - Execution Adapter Submission Preconditions

## Goal

Strengthen the deterministic execution-adapter boundary so every durable adapter run records the non-secret preconditions that any future external submission path must preserve before live adapters can be wired.

## Implemented in this phase

- Added future-submission precondition flags to `ExecutionAdapterConfig` for kill-switch, audit/state preflight, and idempotency protection.
- Required those preconditions in adapter config validation even while external submission remains disabled.
- Added matching durable fields to `ExecutionAdapterRunRecord`.
- Required adapter run validation to fail closed if any future-submission precondition is absent.
- Added the precondition fields to execution-adapter audit metadata and `arb-agent validate-execution-adapter-audit` output.
- Updated `scripts/validate_execution_path_gate.py` so the aggregate execution-path gate enforces the new adapter precondition fields.
- Bumped `EXECUTION_ADAPTER_FRAMEWORK_VERSION` for the updated serialized adapter-run shape.

## Deferred work

- Live adapter submission.
- Real exchange/RPC adapter execution.
- Sandbox/live adapter reconciliation.
- Custody-backed signing or wallet interaction.
- Service-manager restart execution and deployment-host runtime validation.

## Safety notes

This phase adds local deterministic validation only. It performs no live trading, adapter submission, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, secret loading, or production-readiness claim.
