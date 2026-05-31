# PHASE_19_SUBROADMAP.md

## Phase

Phase 19 - Runtime Lifecycle Wiring

## Status

Implemented for local deterministic lifecycle scope. Current workspace Rust/Cargo validation must be refreshed after changes.

## Goal

Wire planner drafts and execution-adapter boundary records through fail-closed local audit/state lifecycle preconditions without enabling live trading, signing, withdrawals, bridges, broadcasts, real exchange/RPC calls, wallet custody, public exposure, or secrets.

## Scope

In scope:

- Local runtime lifecycle request and record models.
- Fail-closed audit-before-adapter lifecycle events.
- Fail-closed plan state checkpoint before adapter evaluation.
- Deterministic adapter-boundary evaluation after audit/state preconditions pass.
- Adapter run checkpoint persistence after deterministic adapter evaluation.
- Local graceful-shutdown audit/state checkpoint records without stopping services.
- Local runtime audit/SQLite backup-restore validation records without deployment actions.
- Local runtime restart recovery validation records without service resume or deployment actions.
- Local restart recovery dispositions for ready-for-local-review versus needs-operator-review.
- CLI status text for local restart recovery dispositions without loading secrets or resuming services.
- Tests proving in-memory and SQLite WAL-backed checkpoint persistence.
- Tests proving concurrent local runtime lifecycle access over shared audit and SQLite WAL paths.
- Tests proving local state permission failure stops the lifecycle before adapter evaluation.
- Tests proving graceful-shutdown audit/state checkpoints reopen locally.
- Tests proving copied local audit and SQLite state artifacts reopen with runtime checkpoints.
- Tests proving local restart recovery replays audit and reopens SQLite runtime checkpoints.
- Tests proving local restart recovery marks missing graceful-shutdown checkpoints as needs-operator-review.
- Tests proving CLI status labels expose the restart recovery dispositions as local operator-review states.
- Tests proving local restart recovery fails closed when audit exists but required SQLite checkpoints are missing.
- Tests proving live-scope lifecycle requests are rejected before audit/state mutation.

Out of scope:

- Live trading.
- External adapter submission.
- Real CEX orders.
- Real DEX swaps.
- Real exchange/RPC calls.
- Signing, withdrawals, bridges, or broadcasts.
- Wallet custody or encrypted keystore implementation.
- Public dashboard, metrics endpoint, or outbound communications runtime.
- Production deployment or production-readiness approval.

## Deliverables

- `crates/arb-core/src/runtime.rs`
- Runtime lifecycle exports from `arb-core`
- Execution-adapter run checkpoint helper
- Structure validator update
- CLI status update
- Governance and gap tracker updates

## Runtime Lifecycle Contract

The local lifecycle must:

1. Validate the lifecycle request and reject live scope.
2. Append a runtime-start audit event.
3. Persist the plan draft through `StateStore`.
4. Append a plan-checkpoint audit event.
5. Evaluate the deterministic execution-adapter boundary.
6. Persist the adapter run through `StateStore`.
7. Append an adapter-complete audit event.
8. Preserve `external_submission_performed = false`.
9. Preserve `live_execution_performed = false`.
10. Record local graceful-shutdown checkpoints without service-manager actions.
11. Validate local audit/state backup-restore copies without storing paths, contents, secrets, or production-readiness claims in the report.
12. Validate local restart recovery summaries without service resume, deployment actions, or production-readiness claims.
13. Classify locally coherent restart recovery as ready-for-local-review or needs-operator-review.
14. Surface those dispositions in CLI status text as local operator-review states only.
15. Fail closed when restart recovery sees incomplete local lifecycle checkpoint state.

Any audit, state, planner, adapter, or lifecycle validation failure must stop the lifecycle before subsequent steps.

## Validation

Required after implementation:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Deferred Work

- Production durability validation for SQLite WAL under crash, restart, locking, deployment-host filesystem permission, backup/restore, and deployment-host concurrent access scenarios.
- Long-running daemon orchestration and deployment-host graceful shutdown execution.
- Real observability runtime integration.
- Real dashboard hosting integration.
- Real outbound communications integration.
- Live/sandbox exchange and RPC validation.
- Custody, signer, encrypted keystore, and external adapter submission phases.
- Penetration, load, rollback, incident-drill, deployment, systemd, ARM, and production-readiness validation.

## Rollback Plan

1. Remove `crates/arb-core/src/runtime.rs`.
2. Remove runtime exports from `crates/arb-core/src/lib.rs`.
3. Remove execution-adapter checkpoint helper exports and helper.
4. Remove Phase 19 validator requirement.
5. Revert CLI and governance docs to Phase 18 runtime-lifecycle-gap wording.
6. Run the required validation sequence.
