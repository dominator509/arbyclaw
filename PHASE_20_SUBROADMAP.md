# PHASE_20_SUBROADMAP.md

## Phase

Phase 20 - SQLite WAL Durability Validation

## Status

Implemented for local deterministic SQLite WAL durability validation. External production-host validation remains deferred until it is run on an approved runtime host with non-secret evidence references.

## Goal

Close the local coding gap for SQLite WAL durability by validating integrity checks, WAL checkpoint flushing, primary reopen persistence, backup/restore readability, and multi-handle checkpoint visibility without enabling live trading, signing, withdrawals, bridges, broadcasts, real exchange/RPC calls, wallet custody, public exposure, or secrets.

## Scope

In scope:

- `SqliteWalStateStore` durability validation method.
- Non-secret `SqliteWalDurabilityReport`.
- SQLite journal-mode and synchronous-mode verification.
- `PRAGMA integrity_check` enforcement.
- `PRAGMA wal_checkpoint(TRUNCATE)` enforcement.
- Primary database reopen read validation.
- Checkpointed database copy and backup/restore read validation.
- Multi-handle local checkpoint visibility validation.
- Unit tests for success and fail-closed backup path behavior.
- Roadmap, architecture, handoff, security, README, manifest, and gap tracker reconciliation.

Out of scope:

- Live trading.
- External adapter submission.
- Real exchange/RPC calls.
- Signing, withdrawals, bridges, broadcasts, or wallet custody.
- Encrypted keystore implementation.
- Production deployment.
- Production-host crash injection or filesystem failure injection.
- Production readiness, live-funds readiness, or deployment approval.

## Subsystem Boundaries

- `arb-core::state` owns SQLite WAL local durability validation.
- Runtime lifecycle code may use the state boundary, but this phase does not change live execution behavior.
- `arb-agent` may report that the local validation boundary exists; it must not run live integrations or start network services.
- Documentation must distinguish local code/test validation from external production-host validation.

## Implementation Sequence

1. Reconcile governance files and active roadmap state.
2. Run the required baseline validation before patching.
3. Add `SqliteWalDurabilityReport` and `SQLITE_WAL_DURABILITY_VERSION`.
4. Add journal-mode, synchronous-mode, integrity-check, WAL checkpoint truncate, and full durability validation methods to `SqliteWalStateStore`.
5. Add deterministic tests for durability success and fail-closed backup path handling.
6. Export the durability report/version through `arb-core`.
7. Update CLI status text to reflect the local durability validation boundary.
8. Update roadmap, architecture, handoff, security, README, gap tracker, and structure manifest.
9. Run required validation again.

## Required Validation

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Acceptance Criteria

- `SqliteWalStateStore::validate_durability` writes only non-secret probes.
- Validation fails closed if the backup path is empty, equal to primary, secret-like, or already exists.
- Validation confirms WAL mode and synchronous FULL mode.
- Validation fails closed unless SQLite integrity check returns `ok`.
- Validation fails closed if WAL checkpoint truncate reports busy pages.
- Validation proves primary reopen and backup/restore read the probe checkpoint.
- Validation proves two local handles can observe each other's checkpoint writes.
- Rust tests pass with the new durability coverage.
- The production gap tracker no longer claims the local SQLite durability validation code is missing.
- External production-host validation remains tracked separately and open until actually performed.

## Rollback Plan

1. Remove the durability report, version constant, and validation methods from `crates/arb-core/src/state.rs`.
2. Remove durability exports from `crates/arb-core/src/lib.rs`.
3. Remove CLI status text for SQLite durability validation.
4. Remove Phase 20 from `scripts/validate_structure.py`.
5. Revert roadmap, architecture, handoff, README, security, manifest, and gap tracker Phase 20 updates.
6. Re-run the required validation sequence.

## Deferred Work

- Production-host crash/restart validation.
- Filesystem permission and disk-full validation.
- Long-running runtime lifecycle load validation.
- Audit journal crash/concurrency durability validation.
- Deployment-like validation with retained non-secret evidence references.
- Container/systemd/ARM deployment validation.
