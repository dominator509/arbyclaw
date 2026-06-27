# PHASE_21_SUBROADMAP.md

## Phase

Phase 21 - Paper Balance Ledgering

## Status

Implemented for local deterministic paper balance ledgering. This phase does not enable live trading, real balances, external venue calls, wallet custody, signing, withdrawals, bridges, broadcasts, or production readiness.

## Goal

Close the paper balance ledger coding gap by adding typed local-only simulated balances, quote-notional reservations, deterministic fill settlement, insufficient-balance denial, missing-reservation denial, and SQLite checkpoint persistence for the paper execution boundary.

## Scope

In scope:

- Paper-only balance records.
- Paper-only ledger entries.
- Initial paper balances.
- Quote-notional reservation before modeled fills are recorded.
- Settlement of modeled paper fills with net paper P&L.
- Fail-closed insufficient available balance checks.
- Fail-closed missing reservation checks.
- Adapter helper that returns a report only after ledger reservation and settlement succeed.
- State-store checkpoint helper for the latest paper balance ledger.
- Unit tests for success, insufficient balance, missing reservation, and SQLite WAL persistence.
- Roadmap, architecture, handoff, README, security, gap tracker, and manifest reconciliation.

Out of scope:

- Real balance reads.
- Real account mutation.
- Live exchange/RPC calls.
- Signing, withdrawals, bridges, broadcasts, or wallet custody.
- Phase 23 covers local supplied-depth consumption, partial fills, latency, and queue-position modeling; exchange-specific matching, calibration, adverse selection, and sandbox/live discrepancy analysis remain deferred.
- Production deployment or production-readiness approval.

## Subsystem Boundaries

- `arb-core::paper` owns paper-only simulated balances, reservations, settlement, and ledger checkpointing.
- `arb-core::policy` remains the approval boundary before any paper report is produced.
- `arb-core::state` persists only non-secret paper ledger checkpoints.
- `arb-agent` may report ledger availability but must not read real balances or call external venues.

## Implementation Sequence

1. Reconcile governance files and active roadmap state.
2. Run required baseline validation before patching.
3. Add paper balance ledger records, entries, version marker, and checkpoint key.
4. Add reserve and settle methods with fail-closed balance checks.
5. Add `PaperExecutionAdapter::submit_with_ledger`.
6. Add paper ledger checkpoint persistence.
7. Export paper ledger types through `arb-core`.
8. Update CLI status text.
9. Update structure validator and governance docs.
10. Run required validation again.

## Required Validation

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Acceptance Criteria

- Paper ledger balances cannot be negative or non-finite.
- Paper execution with ledger fails closed if available quote balance is insufficient.
- Paper settlement fails closed if quote notional was not reserved.
- Successful paper execution reserves notional, settles modeled fill, clears reservation, and applies net paper P&L.
- Paper ledger checkpoints persist and restore through the SQLite WAL state store.
- No live trading, real balance mutation, signing, broadcasts, withdrawals, bridges, external calls, or secrets are introduced.

## Rollback Plan

1. Remove paper ledger records, methods, exports, checkpoint helper, and tests from `crates/arb-core/src/paper.rs`.
2. Remove paper ledger exports from `crates/arb-core/src/lib.rs`.
3. Remove paper ledger status text from `crates/arb-agent/src/main.rs`.
4. Remove Phase 21 from `scripts/validate_structure.py`.
5. Revert roadmap, architecture, handoff, README, security, gap tracker, and manifest updates.
6. Re-run required validation.

## Deferred Work

- Depth-aware fill simulation.
- Partial fills.
- Latency and queue-position modeling.
- Exchange-specific matching behavior.
- Production audit durability validation for paper ledger mutation audit records.
- Production runtime replay and deployment-host validation.
