# PHASE_23_SUBROADMAP.md

## Phase

Phase 23 - Realistic Paper Fills

## Status

Implemented for local deterministic paper simulation scope.

## Goal

Replace the prior full-notional paper-fill assumption with deterministic local fill realism that consumes caller-supplied order-book depth, models partial and unfilled outcomes, applies latency and queue-position assumptions, and settles modeled results through the existing paper balance ledger without live exchange calls or real balance mutation.

## Scope

Phase 23 may add:

- local paper fill model configuration
- caller-supplied order-book depth walking
- buy/sell side selection for consuming asks or bids
- deterministic average fill price and slippage reporting
- partial-fill and unfilled statuses
- latency and queue-position fields
- ledger settlement that releases unfilled reserved quote notional
- unit tests for full, partial, unfilled, and ledgered outcomes
- governance and structure validation updates

## Explicit Non-Goals

Phase 23 must not implement:

- live trading
- live CEX orders
- live DEX swaps
- real exchange or RPC calls
- wallet signing
- broadcasts
- withdrawals
- bridges
- real balance reads or mutation
- exchange-specific matching engines
- external adapter submission
- secrets or credential handling
- production deployment or production-readiness approval

## Implementation Tasks

1. Add realistic paper fill model records to `crates/arb-core/src/paper.rs`.
2. Add local order-book depth walking for supplied normalized snapshots.
3. Add deterministic partial-fill and unfilled outcomes.
4. Wire realistic reports into paper balance ledger settlement.
5. Export the fill model records from `arb-core`.
6. Add focused Rust tests.
7. Update structure validation and governance docs.
8. Regenerate `STRUCTURE_MANIFEST.md`.
9. Run the required validation sequence.

## Validation

Required after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met when realistic paper fills are represented by typed deterministic local records, supplied order-book depth is consumed without network calls, partial/unfilled outcomes are ledger-safe, tests pass, docs and gap tracker are updated, and all live execution boundaries remain denied.

## Rollback Plan

1. Remove Phase 23 fill-model types and methods from `crates/arb-core/src/paper.rs`.
2. Remove Phase 23 exports from `crates/arb-core/src/lib.rs`.
3. Remove Phase 23 from `scripts/validate_structure.py`.
4. Revert roadmap, architecture, handoff, README, security, manifest, and gap tracker updates to Phase 22 state.
5. Re-run the required validation sequence.

## Deferred Work

- Exchange-specific matching behavior.
- Real sandbox venue validation.
- Queue-position calibration against venue data.
- Historical replay and backtesting corpus execution.
- Paper audit/replay validation.
- Production runtime validation and deployment-host validation.

Phase 24 later added local deterministic implementations for exchange matching profiles, adverse-selection penalties, reference-only calibration records, paper replay validation, local historical-fixture backtest execution, and runtime validation records. Real sandbox/live evidence, venue-data calibration, and deployment-host validation remain deferred.
