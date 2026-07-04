# Phase 67 - Deployment SQLite Schema Migration Transcript Gate

## Goal

Add a local, non-mutating validator for deployment-host SQLite schema migration evidence so operators can record sanitized execution references for state schema migration review without the validator touching deployment paths or services.

## Scope

- Validate non-secret references for deployment-host evidence, service lifecycle context, pre-migration backup, migration execution, schema-version transition, SQLite integrity/checkpoint reopen, audit replay after migration, rollback reference, runtime quiesce/degrade evidence, and reviewer/operator approval.
- Expose the validator through `arb-agent validate-deployment-sqlite-schema-migration-transcript`.
- Include the validator in deployment runtime/evidence aggregate scripts.
- Preserve production blockers until real operator-controlled deployment-host migration execution evidence exists.

## Non-Goals

- No live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, or secrets.
- No actual deployment-host migration execution.
- No service-manager actions.
- No deployment path mutation.
- No external submission or production-readiness claim.

## Validation

- `cargo test -p arb-core deployment_sqlite_schema_migration -- --nocapture`
- `cargo run -p arb-agent -- validate-deployment-sqlite-schema-migration-transcript`
- `python scripts/validate_deployment_runtime_gate.py --json`
- Full workspace gates before commit/push.
