# PHASE_6_SUBROADMAP.md

## Phase 6 — Simulated/Paper Connectors

## Objectives

1. Add deterministic paper-only connector primitives that can be used by future strategy and opportunity-engine phases.
2. Provide an in-memory market-data provider that implements the existing `MarketDataProvider` trait without network access.
3. Provide a static paper fee provider that implements the existing `FeeProvider` trait without paid provider credentials.
4. Provide a policy-gated paper execution adapter that produces deterministic paper execution reports only.
5. Preserve all live-trading, wallet, withdrawal, signer, exchange, DEX, and chain execution prohibitions.

## Deliverables

- `crates/arb-core/src/paper.rs`
- `PaperMarketDataProvider`
- `PaperFeeProvider`
- `PaperExecutionAdapter`
- `PaperExecutionReport`
- `PaperConnectorError`
- Phase 6 exports from `arb-core`
- CLI status update noting paper connectors are available
- Structure validator update
- Governance updates in `ROADMAP.md` and `PRODUCTION_GAP_TRACKER.md`

## Subsystem Boundaries

### In Scope

- In-memory deterministic paper market data
- Static fee schedules
- Policy-gated paper intent submission
- Paper execution reports
- Unit-testable deterministic scaffolds

### Out of Scope

- Live CEX connectors
- Live DEX connectors
- Wallet signing
- Withdrawals
- Bridge routing
- Chain RPC calls
- WebSocket/REST market-data networking
- Real balances or settlement
- Real order placement
- SQLite persistence
- Production telemetry

## Dependencies

- Phase 2 config and mode gates
- Phase 3 policy engine
- Phase 4 audit/state boundaries
- Phase 5 market-data and fee models

## Implementation Sequence

1. Reconcile existing governance files and current roadmap position.
2. Create this Phase 6 sub-roadmap before code changes.
3. Add `paper.rs` as an isolated `arb-core` module.
4. Implement the paper market-data provider using already-normalized order books.
5. Implement the static paper fee provider using Phase 5 fee schedules.
6. Implement paper execution report generation behind `PolicyEngine` approval.
7. Export Phase 6 primitives from `lib.rs`.
8. Update CLI status output.
9. Update structure validation.
10. Update roadmap and gap tracker.
11. Run available validation.

## Validation Sequence

Executable in ChatGPT Project Mode:

- `python3 scripts/validate_structure.py`
- static file-existence validation
- secret-assignment scan

Current workspace validation now passes locally and in GitHub Actions:

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

Future external validation:

- deterministic paper provider tests
- policy-denial path tests
- paper execution report consistency tests
- opportunity-engine integration tests
- audit integration tests once execution paths begin writing journals

## Rollback Strategy

1. Remove `crates/arb-core/src/paper.rs`.
2. Remove paper exports from `crates/arb-core/src/lib.rs`.
3. Revert CLI status text.
4. Revert validator required-file list.
5. Revert this sub-roadmap and governance updates.

No secrets, external accounts, runtime state, wallets, infrastructure, or network integrations are created in this phase.

## Drift-Prevention Constraints

- No live network code.
- No secret loading.
- No exchange-specific API implementation.
- No signer or wallet control.
- No withdrawal path.
- No bridge-route support.
- Paper execution must call `PolicyEngine` before producing any filled report.
- Paper execution must reject non-paper scopes.

## Environment Limitations

- Current workspace Rust/Cargo validation has local and CI evidence for the present state.
- No live exchange sandbox credentials are available.
- No actual network calls can be validated as production-ready.
- No runtime persistence, concurrency, or crash-recovery validation is performed.

## Expected Unresolved Gaps

- Rust validation remains deferred.
- Paper connectors are not production execution connectors.
- Phase 23 models local supplied-depth slippage, partial fills, latency, queue position, and unfilled notional release. Phase 24 adds local venue matching profiles, adverse-selection modeling, reference-only calibration records, paper replay validation, and local historical-fixture backtest execution; external sandbox/live calibration evidence and real settlement remain deferred.
- Audit/state integration remains deferred.
- Live connectors remain unimplemented.

## Expected Future Continuation Tasks

- Add opportunity detection using paper market data.
- Paper balance ledgering was added in Phase 21 for local simulated balances, quote-notional reservation, deterministic settlement, insufficient-balance denial, missing-reservation denial, and SQLite checkpoint persistence. Phase 23 added local supplied-depth fill simulation with partial fills and latency, and Phase 24 added local venue realism, replay validation, and local historical-fixture scenario execution. Future work still needs external sandbox/live calibration evidence and production-host validation.
- Add audit journaling for every paper execution event.
- Add deterministic scenario fixtures.
- Add CEX sandbox connectors after validation.
- Add DEX quote-only connectors before signer work.

## Phase 6 Completion Criteria

- Paper connector module exists.
- Paper market-data provider implements the market-data trait.
- Paper fee provider implements the fee trait.
- Paper execution adapter requires policy approval and paper scope.
- Structure validation passes.
- Gap tracker records deferred Rust validation and future paper-model limitations.
