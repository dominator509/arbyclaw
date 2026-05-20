# PHASE_5_SUBROADMAP.md

## Phase 5 — Market Data Core

## Objectives

1. Add normalized market-data primitives without live network connectors.
2. Add top-of-book, order-book, and freshness models that future strategies can consume deterministically.
3. Add fee-model primitives that calculate fee-adjusted edge without placing trades.
4. Add provider trait boundaries for future read-only connectors.
5. Preserve all existing policy, audit, secret, and mode-gate boundaries.
6. Keep live execution, wallet signing, exchange credentials, DEX transactions, and withdrawals out of scope.

## Deliverables

- `crates/arb-core/src/market_data.rs`
- `crates/arb-core/src/fees.rs`
- Public exports through `crates/arb-core/src/lib.rs`
- CLI status text acknowledging market-data boundary availability
- Structure validator updates
- Roadmap, architecture, and gap-tracker updates

## Subsystem Boundaries

### In Scope

- Market pair normalization
- Price-level validation
- Top-of-book quote validation
- Order-book snapshot validation
- Market-data freshness classification
- Market-data provider trait boundary
- Fee schedule validation
- Fee estimate calculation
- Fee-adjusted edge calculation
- Fee provider trait boundary

### Out of Scope

- Live CEX REST/WebSocket calls
- Live DEX RPC calls
- Paid data-provider integrations
- Exchange API credential loading
- Wallet signing
- Order placement
- Swap submission
- Balance mutation
- Opportunity ranking
- Execution planning
- Real runtime market-data ingestion

## Dependencies

Completed prerequisites:

- Phase 0 governance files
- Phase 1 Rust workspace scaffold
- Phase 2 typed config and secret-reference boundary
- Phase 3 deny-by-default policy engine
- Phase 4 append-only audit and state-store boundary

External/deferred prerequisites:

- Rust/Cargo validation environment
- Future connector API review
- Future exchange/provider terms and rate-limit review
- Future paid market-data provider account setup where applicable

## Implementation Sequence

1. Reconcile governance files and current code boundaries.
2. Create this Phase 5 sub-roadmap before implementation.
3. Add `market_data.rs` with normalized pair, quote, order-book, freshness, and provider trait types.
4. Add `fees.rs` with fee schedule, estimate, fee-adjusted edge, and provider trait types.
5. Export new primitives from `arb-core`.
6. Update `arb-agent` status text without adding runtime network behavior.
7. Update structure validator.
8. Update governance docs and production gap tracker.
9. Run available validation.

## Validation Sequence

Executable inside ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Deferred to Rust-enabled local/CI environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Additional future validation required:

- property tests for market-pair normalization
- property tests for fee arithmetic
- fuzz tests for malformed order books
- stale/future timestamp tests
- no-secret-in-market-data tests
- provider trait integration tests
- live exchange rate-limit and data-quality validation

## Rollback Strategy

1. Remove `crates/arb-core/src/market_data.rs`.
2. Remove `crates/arb-core/src/fees.rs`.
3. Remove new exports from `crates/arb-core/src/lib.rs`.
4. Revert `arb-agent` status text.
5. Revert `scripts/validate_structure.py` Phase 5 entries.
6. Revert roadmap, architecture, and gap-tracker updates to Phase 4 state.

No secrets, wallets, exchange accounts, runtime state, network integrations, or infrastructure are introduced in this phase.

## Drift-Prevention Constraints

- Do not add live connectors in Phase 5.
- Do not add trading execution paths in Phase 5.
- Do not add wallet signing or transaction building in Phase 5.
- Do not add API keys, tokens, or provider credentials.
- Do not claim live market-data validation occurred.
- Do not claim production readiness beyond local boundary construction.
- Keep market-data code read-only and side-effect free.

## Environment Limitations

- Rust/Cargo validation is unavailable in the current ChatGPT Project Mode environment.
- Real exchange, DEX, and paid-provider data cannot be validated here.
- Real latency, WebSocket, REST, rate-limit, and data-quality behavior cannot be validated here.
- ARM-device runtime behavior cannot be validated here.

## Expected Unresolved Gaps

- Rust/Cargo validation deferred.
- No live market-data providers implemented.
- No rate-limit handling.
- No WebSocket reconnect logic.
- No paid market-data provider integration.
- No historical data persistence.
- No market-data quality scoring.
- No strategy/opportunity engine consuming these models yet.

## Expected Future Continuation Tasks

1. Run Rust validation externally.
2. Add deterministic paper/simulated market-data providers in Phase 6.
3. Add read-only CEX connector traits and sandbox connectors in Phase 7.
4. Add DEX/Web3 quote connector boundaries in Phase 8.
5. Feed validated quotes into the opportunity engine in Phase 9.
6. Add market-data metrics and health checks in Phase 14.
