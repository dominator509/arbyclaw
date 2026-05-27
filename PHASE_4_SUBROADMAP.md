# PHASE_4_SUBROADMAP.md

## Phase

Phase 4 — Audit Journal and State Store

## Objectives

Create the first durable accountability boundary for the arbitrage agent without adding live trading, exchange connectors, wallet signing, or production deployment behavior.

Phase 4 must ensure future execution paths have a deterministic place to record:

- runtime lifecycle events
- config loading events
- policy decisions
- execution intents
- execution planning events
- future connector submissions and outcomes
- reconciliation results
- security alerts

## Deliverables

1. `crates/arb-core/src/audit.rs`
   - typed audit events
   - typed audit metadata values
   - redaction validation
   - secret-like metadata rejection
   - append-only JSONL journal
   - hash-chained audit records
   - file-open replay validation
   - deterministic append API
   - unit tests drafted for append/reopen and redaction rejection

2. `crates/arb-core/src/state.rs`
   - state-store trait boundary
   - checkpoint model
   - in-memory implementation for tests and early local scaffolding only
   - explicit non-production persistence warning

3. `crates/arb-core/src/lib.rs`
   - audit and state module exports

4. `crates/arb-core/Cargo.toml`
   - dependency update for JSON serialization and hash chaining

5. `crates/arb-agent/src/main.rs`
   - status messaging updated to show audit/state boundary availability without writing runtime state yet

6. Governance updates
   - `ARCHITECTURE.md`
   - `ROADMAP.md`
   - `PRODUCTION_GAP_TRACKER.md`
   - `README.md`
   - `SECURITY.md`

## Subsystem Boundaries

### In Scope

- Append-only local audit journal primitives
- Hash-chain integrity checks for local JSONL records
- Secret redaction checks before audit append
- State-store abstraction
- In-memory test state store
- SQLite WAL checkpoint state store

### Out of Scope

- SQLCipher or encrypted database implementation
- production retention policy
- production durability claims
- log shipping
- OpenTelemetry integration
- market data persistence
- order execution persistence
- exchange connectors
- DEX/Web3 connectors
- wallet signing
- live trading

## Dependencies

- Phase 0 governance exists.
- Phase 1 Rust workspace scaffold exists.
- Phase 2 typed config and secret-reference boundaries exist.
- Phase 3 policy engine exists.
- Rust/Cargo validation remains environment-limited in ChatGPT Project Mode.

## Implementation Sequence

1. Reconcile governance files.
2. Create this Phase 4 sub-roadmap before implementation.
3. Add isolated `audit` module.
4. Add isolated `state` module.
5. Export modules from `arb-core`.
6. Update CLI status only; do not start writing journals automatically.
7. Update structure validator.
8. Update governance and gap tracker.
9. Run available Python structure validation.
10. Record all Rust/Cargo validation as deferred.

## Validation Sequence

### Executable in ChatGPT Project Mode

- `python3 scripts/validate_structure.py`
- static required-file verification
- static secret-assignment scan

### Deferred to Rust-Enabled Environment

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- append/reopen audit tests
- tamper-detection tests
- redaction tests
- crash/recovery tests
- filesystem permission tests
- concurrent append tests

## Rollback Strategy

Rollback Phase 4 by:

1. Removing `PHASE_4_SUBROADMAP.md`.
2. Removing `crates/arb-core/src/audit.rs`.
3. Removing `crates/arb-core/src/state.rs`.
4. Removing audit/state exports from `crates/arb-core/src/lib.rs`.
5. Reverting dependency additions in `crates/arb-core/Cargo.toml`.
6. Reverting CLI status text.
7. Reverting governance and gap-tracker changes to Phase 3 state.

No secrets, wallets, exchange accounts, infrastructure, or live trading state should exist as a result of Phase 4.

## Drift-Prevention Constraints

- Do not add live trading.
- Do not add exchange connectors.
- Do not add DEX/Web3 connectors.
- Do not add wallet signing.
- Do not create secret storage beyond redaction checks.
- Do not claim database production durability beyond implemented local SQLite WAL checkpoint persistence.
- Do not claim Rust validation until Cargo commands actually run.
- Do not permit audit events to contain raw secrets.

## Environment Limitations

Current local and GitHub Actions Rust/Cargo validation exists for this workspace. Future state-store changes must rerun structure, format, compile, test, and clippy validation for the exact changed state.

## Expected Unresolved Gaps

- Rust validation deferred.
- SQLite WAL state store production durability not externally validated.
- audit durability not crash-tested.
- audit file permissions not validated.
- concurrent append safety not validated.
- audit integration with policy/execution not mandatory yet.
- external log shipping not implemented.

## Expected Future Continuation Tasks

- Run Cargo validation externally.
- Continue wiring SQLite WAL-backed state store into runtime lifecycle checkpoints beyond the current local paper-report checkpoint helper.
- Add audit integration to execution planner and adapters.
- Add audit retention and compaction strategy.
- Add tamper-evident export verification tool.
- Add operator-readable audit inspection CLI.
- Add runtime health checks and observability integration.

## Completion Update

Status: Implemented in ChatGPT Project Mode as an audit/state boundary patch.

### Completed Tasks

- Created `crates/arb-core/src/audit.rs`.
- Created `crates/arb-core/src/state.rs`.
- Exported audit and state types from `crates/arb-core/src/lib.rs`.
- Added `serde_json`, `sha2`, and `rusqlite` dependencies for JSONL serialization, hash chaining, and SQLite WAL checkpoint persistence.
- Added `SqliteWalStateStore` for non-secret local checkpoint persistence.
- Updated `arb-agent` status text without starting runtime journal writes.
- Updated `scripts/validate_structure.py`.
- Updated governance and gap tracker.

### Validated Tasks

- `python3 scripts/validate_structure.py` passed.
- Mandatory Phase 4 file presence passed.
- Static secret-assignment scan passed.

### Deferred Tasks

- Rust formatting, compilation, tests, and clippy.
- Audit append/reopen unit test execution.
- Audit tamper-detection test execution.
- Audit redaction test execution.
- Crash/recovery tests.
- Concurrent append tests.
- Filesystem permission tests.
- SQLite WAL crash/recovery, migration, file-locking, backup/restore, and filesystem-permission validation.

### Environment-Limited Tasks

- Cargo validation requires Rust/Cargo outside ChatGPT Project Mode.
- Filesystem durability and concurrent writer behavior require local/CI runtime testing.
- Database persistence requires future dependency selection and migration validation.

### New Discovered Gaps

- Phase 4 Rust validation not executed.
- SQLite WAL state store is implemented for local non-secret checkpoints, but production durability validation is incomplete.
- Audit durability/concurrency/filesystem validation missing.
- Audit not yet mandatory in execution paths.

### Production Readiness Recalculation

Current production readiness: 20%.

### Risk Posture Recalculation

Risk remains High because live trading, wallet custody, execution adapters, connector integrations, production durability validation, and external runtime validations are still missing.

### Next Continuation Path

Create `PHASE_5_SUBROADMAP.md`, then implement market-data core models, freshness logic, fee models, and provider trait boundaries without adding live exchange credentials or live execution.
