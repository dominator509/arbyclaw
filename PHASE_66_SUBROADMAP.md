# Phase 66 - SQLite WAL State Schema Migration Guard

## Goal

Add a local versioned schema guard to the SQLite WAL state store so current local checkpoint databases record the supported schema version, legacy v0 local checkpoint tables migrate to v1, and unknown future schema versions fail closed before reads or writes continue.

## Completed Tasks

- Added `SQLITE_WAL_STATE_SCHEMA_VERSION`.
- Added `PRAGMA user_version` based schema-version reads.
- Added local schema migration during `SqliteWalStateStore::open`.
- Preserved legacy v0 `state_checkpoints` rows during migration to schema v1.
- Failed closed when a database reports a schema version newer than this binary supports.
- Added schema-version reporting to `SqliteWalDurabilityReport`.
- Surfaced schema migration coverage in the `arb-agent` status output.
- Added local Rust tests for v0 migration, future-version rejection, and durability report schema-version evidence.

## Explicit Non-Goals

- No production database migration.
- No deployment-host filesystem mutation beyond local test/temp SQLite files.
- No live trading, live exchange/RPC calls, signing, withdrawals, bridges, broadcasts, or adapter submission.
- No secret loading or wallet custody.
- No production deployment or production-readiness approval.

## Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core state::tests::sqlite_wal -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local SQLite WAL state schema-version migration and fail-closed compatibility checks only. Deployment-host schema migration, deployment-host audit/SQLite recovery execution, production database migration operations, service-manager-controlled runtime validation, and production readiness remain unclaimed.
