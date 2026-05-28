# PHASE_24_SUBROADMAP.md

## Phase

Phase 24 - Paper Replay, Calibration, Backtest, and Runtime Validation Boundaries

## Status

Implemented for local deterministic paper validation scope. This phase does not enable live trading, real balances, external venue calls, wallet custody, signing, withdrawals, bridges, broadcasts, or production deployment.

## Goal

Close the remaining local coding gaps from Phase 23 by adding exchange-specific paper matching profiles, adverse-selection modeling, sandbox/live discrepancy calibration records, paper ledger replay validation, local historical-fixture backtest corpus execution, and runtime validation records that preserve external production blockers honestly.

## Scope

In scope:

- Exchange-specific paper matching profiles for tick size, quantity step, min/max notional, partial-fill support, and queue behavior.
- Deterministic adverse-selection penalty records.
- Reference-only sandbox/live discrepancy calibration records.
- Ledger replay validation for paper balances and entries.
- Local-only paper backtest corpus execution over caller-supplied fixtures.
- Runtime validation records that distinguish local replay/backtest evidence from missing production-host validation.
- Focused Rust tests for the above boundaries.
- Roadmap, architecture, handoff, README, security, gap tracker, structure validator, and manifest reconciliation.

Out of scope:

- Live trading.
- Real exchange, sandbox, or RPC calls.
- Real balance reads or account mutation.
- Wallet custody, signing, withdrawals, bridges, broadcasts, or external adapter submission.
- External data downloads.
- Production deployment or production-readiness approval.
- Treating sandbox/live calibration records as external evidence unless a future operator supplies non-secret evidence references.

## Subsystem Boundaries

- `arb-core::paper` owns local paper realism, replay, backtest, and runtime-validation records.
- `arb-core::policy` remains the approval boundary before any paper execution report is produced.
- `arb-core::state` remains the non-secret checkpoint boundary for paper reports and ledgers.
- Documentation and gap tracking must continue to separate local deterministic validation from external production-host validation.

## Implementation Sequence

1. Reconcile governance files and active roadmap state.
2. Run the required baseline validation before patching.
3. Add Phase 24 paper realism/replay/backtest/runtime validation records.
4. Wire venue-realism paper execution through the existing paper adapter and ledger.
5. Add paper ledger replay validation.
6. Add local-only backtest corpus execution over supplied fixtures.
7. Add runtime validation records that keep production blockers visible.
8. Export new types through `arb-core`.
9. Update CLI status text and structure validator.
10. Update governance docs and regenerate the structure manifest.
11. Run required validation again.

## Required Validation

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Acceptance Criteria

- Venue matching profiles alter local paper fills deterministically without external calls.
- Adverse-selection and calibration penalties are represented as explicit non-secret records and adjust paper P&L only inside paper reports.
- Paper ledger replay validation detects mutation inconsistency and validates final balances.
- Local backtest corpus execution runs through the paper adapter and ledger without live network use or external execution.
- Runtime validation records can show local replay/backtest coverage while keeping production-host validation and readiness unclaimed.
- All new behavior is covered by Rust tests and preserves deny-by-default live-funds boundaries.

## Rollback Plan

1. Remove Phase 24 types, methods, exports, and tests from `crates/arb-core/src/paper.rs`.
2. Remove Phase 24 exports from `crates/arb-core/src/lib.rs`.
3. Remove Phase 24 CLI status text.
4. Remove Phase 24 from `scripts/validate_structure.py`.
5. Revert roadmap, architecture, handoff, README, security, gap tracker, and manifest updates.
6. Re-run the required validation sequence.

## Deferred Work

- Real exchange sandbox/live calibration evidence.
- Deployment-host runtime validation.
- Production service soak testing.
- Audit journal crash/concurrency validation.
- Live/sandbox exchange and RPC validation.
- Custody, signer, encrypted keystore, and external adapter submission phases.
- Penetration, load, rollback, incident-drill, deployment, systemd, ARM, and production-readiness validation.
