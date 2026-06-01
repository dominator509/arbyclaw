# PHASE_27_SUBROADMAP.md

## Phase

Phase 27 - Opportunity Depth, Inventory, and Transfer-Risk Modeling

## Status

Implemented for local deterministic opportunity-engine realism. Production, sandbox, and live-market validation remain deferred.

## Goal

Reduce the remaining local opportunity-engine gap by modeling caller-supplied order-book depth, paper inventory caps, transfer-latency risk, and same-venue triangular path discovery inside opportunity discovery without adding live trading, real exchange calls, real RPC calls, signing, broadcasts, withdrawals, bridges, custody, or secrets.

## Scope

In scope:

- Optional order-book inputs for depth-aware candidate sizing.
- Weighted average buy/sell prices from caller-supplied local order books.
- Local paper inventory caps for buy-side quote availability and sell-side base availability.
- Optional transfer-risk profiles with sanitized evidence labels.
- Deterministic score penalties for transfer latency/settlement risk.
- Same-venue triangular path discovery over caller-supplied local quotes and fee schedules.
- Candidate records that expose liquidity and transfer-risk modeling details.
- Rust tests for depth/inventory sizing, transfer-risk scoring, and triangular discovery.

Out of scope:

- Live market-data connections.
- Exchange REST/WebSocket clients.
- Chain RPC calls.
- Wallet signing.
- Transaction broadcasts.
- Withdrawals or bridges.
- Real balance reads.
- Real transfer execution.
- Production readiness or profit claims.
- External sandbox/live calibration evidence.

## Dependencies

- Phase 5 market-data and fee models.
- Phase 9 opportunity engine.
- Phase 21 paper balance ledgering.
- Phase 23 realistic local paper fills.
- Phase 24 paper replay/calibration/backtest boundaries.
- Current local and CI Rust validation gates.

## Implementation Tasks

1. Add optional order-book, inventory-limit, and transfer-risk inputs to opportunity discovery requests.
2. Add candidate liquidity and transfer-risk records.
3. Walk local order-book levels to compute executable quantity and average prices.
4. Cap candidate size using supplied local paper inventory limits.
5. Apply transfer-risk score penalties from sanitized local profiles.
6. Preserve fail-closed validation for stale/future order books and invalid local inputs.
7. Add same-venue triangular path search from local quote and fee inputs only.
8. Export new Phase 27 opportunity types.
9. Surface the refined opportunity-engine scope in CLI status.
10. Update governance docs, structure validation, and production gap tracking.
11. Run the standard validation sequence.

## Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local deterministic modeling when:

- Phase 27 sub-roadmap exists.
- Opportunity discovery remains non-executing and local-input-only.
- Candidate sizing can use supplied order-book depth.
- Candidate sizing can be capped by supplied paper inventory.
- Transfer-risk score penalties can be attached from sanitized local profiles.
- Same-venue triangular paths can be discovered from supplied local quotes and fee schedules.
- Tests cover depth/inventory caps, transfer-risk penalties, and triangular discovery.
- Standard validation passes.

## Deferred Work

- Broader triangular route scenario replay and false-positive validation remain future work.
- Inventory-aware opportunity discovery must still be validated against broader scenario corpora.
- External sandbox/live calibration evidence remains missing.
- Production-host runtime validation remains missing.
- Live connectors, custody, signing, broadcasts, bridges, and withdrawals remain unavailable.

## Rollback Plan

1. Revert the Phase 27 opportunity-engine changes.
2. Remove Phase 27 exports and CLI status text.
3. Remove this sub-roadmap from structure validation.
4. Revert roadmap/gap tracker documentation to Phase 26.
5. Re-run the standard validation sequence.
