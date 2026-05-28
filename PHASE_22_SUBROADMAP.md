# PHASE_22_SUBROADMAP.md

## Phase

Phase 22 - Crash/Restart Durability Validation

## Status

Implemented for local process-level SQLite WAL crash/restart validation. This phase does not enable live trading, real balances, external venue calls, wallet custody, signing, withdrawals, bridges, broadcasts, or production deployment.

## Goal

Close the local crash/restart durability validation coding gap by proving committed SQLite WAL checkpoints survive abrupt child-process termination and can be reopened with integrity checks across runtime checkpoint stages.

## Scope

In scope:

- Process-level integration test harness.
- Child-process writes to `SqliteWalStateStore`.
- Abrupt child termination after start checkpoint.
- Abrupt child termination after planner checkpoint.
- Abrupt child termination after adapter checkpoint.
- Parent-process reopen and `PRAGMA integrity_check` verification.
- Parent-process verification of expected checkpoint presence or absence after each crash point.
- Governance, roadmap, gap tracker, handoff, README, security, and manifest reconciliation.

Out of scope:

- OS power-loss simulation.
- Disk-full simulation.
- Filesystem permission fault injection.
- Production service deployment.
- Live trading or external adapter submission.
- Real exchange/RPC calls.
- Signing, withdrawals, bridges, broadcasts, or wallet custody.

## Subsystem Boundaries

- `arb-core::state` remains the durable checkpoint boundary.
- Phase 22 tests exercise the existing `SqliteWalStateStore` and runtime checkpoint keys without adding live behavior.
- Runtime lifecycle behavior remains local and fail-closed.
- Production-host validation claims remain limited to the local process-level harness actually executed by Cargo.

## Implementation Sequence

1. Reconcile governance files and active roadmap state.
2. Run required baseline validation before patching.
3. Add a process-level SQLite WAL crash/restart integration test.
4. Add Phase 22 to the structure validator.
5. Update roadmap, architecture, handoff, README, security, gap tracker, and manifest.
6. Run required validation again.

## Required Validation

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Acceptance Criteria

- Crash/restart test launches child test processes.
- Child process exits abruptly after deterministic checkpoint stages.
- Parent process reopens the SQLite WAL database after each child exit.
- Parent process runs integrity check after each child exit.
- Parent process verifies committed checkpoints survived and not-yet-written checkpoints are absent.
- No live trading, real balances, external calls, signing, broadcasts, withdrawals, bridges, custody, secrets, public exposure, or production deployment are introduced.

## Rollback Plan

1. Remove `crates/arb-core/tests/sqlite_wal_crash_restart.rs`.
2. Remove Phase 22 from `scripts/validate_structure.py`.
3. Revert roadmap, architecture, handoff, README, security, gap tracker, and manifest updates.
4. Re-run required validation.

## Deferred Work

- Disk-full validation.
- Filesystem permission fault validation.
- Audit journal crash/concurrency validation.
- Long-running daemon restart validation.
- Deployment-host crash/restart validation outside local Cargo tests.
