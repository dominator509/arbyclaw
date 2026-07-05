## Phase 81 - Local SQLite WAL Schema Migration Gate

### Goal

Add a local SQLite WAL schema migration validation gate so deployment/runtime evidence can account for real `SqliteWalStateStore` legacy-schema migration, checkpoint preservation, and future-version fail-closed behavior without touching deployment hosts, service managers, secrets, external systems, live execution, or production readiness.

### Completed Tasks

- Added `SqliteWalSchemaMigrationStatus` and `SqliteWalSchemaMigrationValidationReport`.
- Added `validate_sqlite_wal_schema_migration` over a fresh local SQLite fixture path.
- Exercised the actual `SqliteWalStateStore` migration path from a legacy v0 checkpoint table to the supported schema version.
- Verified the legacy non-secret checkpoint remains readable after migration.
- Verified an intentionally future-versioned SQLite fixture is rejected fail-closed.
- Surfaced the gate through `arb-agent validate-sqlite-wal-schema-migration --workspace <fresh-dir>`.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-sqlite-schema-migration`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`.
- Added focused local Rust tests for ready local migration and stale-path rejection.

### Explicit Non-Goals

- No deployment-host schema migration execution.
- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core sqlite_wal_schema_migration -- --nocapture
cargo test -p arb-agent sqlite_wal_schema_migration -- --nocapture
cargo run -p arb-agent -- validate-sqlite-wal-schema-migration --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-sqlite-schema-migration --sqlite-schema-migration-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local SQLite WAL schema migration fixture validation only. Deployment-host schema migration execution under service lifecycle, deployment permissions, backup/restore load, restart recovery, physical disk constraints, and production readiness remain unclaimed.
