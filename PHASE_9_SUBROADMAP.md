# PHASE_9_SUBROADMAP.md

## Phase

Phase 9 — Opportunity Engine

## Governance Status

- Created before Phase 9 implementation work began.
- Authoritative parent roadmap: `ROADMAP.md`.
- Required predecessors confirmed complete for ChatGPT Project Mode scope:
  - Phase 0 — Governance Initialization
  - Phase 1 — Rust Workspace Scaffold
  - Phase 2 — Config, Secrets, and Mode Gates
  - Phase 3 — Policy Engine and Trust Contract
  - Phase 4 — Audit Journal and State Store Boundary
  - Phase 5 — Market Data Core and Fee Models
  - Phase 6 — Simulated/Paper Connectors
  - Phase 7 — CEX Connector Framework
  - Phase 8 — DEX/Web3 Connector Framework

## Objective

Implement a deterministic, fee-aware opportunity engine boundary that can discover and rank simulated arbitrage candidates from already-normalized market data without placing orders, generating execution intents, signing transactions, broadcasting transactions, withdrawing funds, bridging assets, or calling real exchange/RPC endpoints.

## Strict Scope

In scope:

- Opportunity model types.
- Route-kind classification for CEX/CEX, DEX/DEX, CEX/DEX, and triangular model boundaries.
- Deterministic cross-venue top-of-book discovery from supplied `NormalizedQuote` values.
- Fee-aware edge calculation using supplied `FeeSchedule` values.
- Freshness checks using existing market-data freshness semantics.
- Deterministic scoring and ranking.
- Validation and fail-closed errors.
- Documentation and gap-tracker updates.

Out of scope:

- Live trading.
- Execution planning.
- Order placement.
- Wallet signing.
- Transaction construction.
- Transaction broadcast.
- Withdrawals.
- Bridges.
- Real CEX API calls.
- Real DEX/RPC calls.
- Strategy optimization, ML, or autonomous capital allocation.

## Subsystem Boundaries

Phase 9 may depend on:

- `market_data` for normalized quotes, pairs, venues, and freshness checks.
- `fees` for deterministic fee estimates and fee-adjusted edges.
- `policy` venue classifications only.

Phase 9 must not depend on:

- Secret material.
- Live connector credentials.
- Signing/custody code.
- Network clients.
- Execution adapters.
- Bridge or withdrawal flows.

## Implementation Plan

1. Add `crates/arb-core/src/opportunity.rs`.
2. Export opportunity-engine types through `crates/arb-core/src/lib.rs`.
3. Add `OpportunityDiscoveryConfig`, `OpportunityDiscoveryRequest`, `OpportunityCandidate`, route/leg/score types, and deterministic validation errors.
4. Add `DeterministicOpportunityEngine` that ranks cross-venue opportunities from supplied quotes and fees only.
5. Keep triangular arbitrage as a typed route boundary while deeper triangular path discovery remains deferred.
6. Update CLI status output to advertise the framework boundary only.
7. Update structure validation to require Phase 9 files.
8. Update `ROADMAP.md`, `PRODUCTION_GAP_TRACKER.md`, `ARCHITECTURE.md`, `HANDOFF_CONTEXT.md`, `STRUCTURE_MANIFEST.md`, `README.md`, and `SECURITY.md`.

## Validation Plan

Run in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Completion Criteria

Phase 9 is complete for ChatGPT Project Mode when:

- `PHASE_9_SUBROADMAP.md` exists.
- `crates/arb-core/src/opportunity.rs` exists.
- Opportunity candidates are deterministic data records, not execution instructions.
- Opportunity discovery consumes supplied normalized quotes and fee schedules only.
- Stale or future-dated market data fails closed.
- Fee-aware ranking is deterministic.
- No live connector, RPC, signer, bridge, withdrawal, or broadcast path is added.
- Governance files and gap tracker are updated.
- Structure validator passes.

## Deferred Work

- Full triangular route path search.
- Inventory-aware sizing.
- Depth-aware order-book slippage modeling beyond top-of-book quantity limits.
- Cross-venue transfer latency and settlement risk models.
- Execution intent generation in Phase 10.
- Live execution adapters in later phases.
- Production/runtime validation.

## Rollback Plan

1. Remove `crates/arb-core/src/opportunity.rs`.
2. Remove opportunity exports from `crates/arb-core/src/lib.rs`.
3. Revert CLI status text in `crates/arb-agent/src/main.rs`.
4. Remove Phase 9 requirements from `scripts/validate_structure.py`.
5. Revert `ROADMAP.md`, `PRODUCTION_GAP_TRACKER.md`, `ARCHITECTURE.md`, `HANDOFF_CONTEXT.md`, `STRUCTURE_MANIFEST.md`, `README.md`, and `SECURITY.md` to Phase 8 state.
6. Run `python3 scripts/validate_structure.py`.
