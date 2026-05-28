# PHASE_3_SUBROADMAP.md

## Phase 3 — Policy Engine and Trust Contract

## Objectives

Implement the first deny-by-default policy subsystem that every future execution adapter must call before orders, swaps, transfers, withdrawals, or signing operations. The subsystem must encode the project trust contract as deterministic Rust types and checks while preserving strict subsystem isolation.

## Deliverables

- `crates/arb-core/src/policy.rs`
- Public policy exports from `crates/arb-core/src/lib.rs`
- CLI initialization message showing policy availability after config load
- Policy tests covering approval and denial paths
- Updated `ROADMAP.md`
- Updated `PRODUCTION_GAP_TRACKER.md`
- Updated `ARCHITECTURE.md`
- Updated structure validator

## Subsystem Boundaries

### In Scope

- Deterministic execution-intent model
- Execution scope classification: observe, paper, live
- Venue, asset, chain, risk, freshness, destination, audit, signing, withdrawal, bridge, and kill-switch checks
- Explicit deny-by-default live runtime gate
- Trust-contract violation codes
- Unit tests that can run once Rust/Cargo is available

### Out of Scope

- Live order submission
- Wallet signing
- Secret loading beyond existing references
- Audit journal persistence
- Market-data ingestion
- CEX/DEX connectors
- Opportunity detection
- Transaction simulation
- External allowlist persistence
- Runtime deployment

## Dependencies

- Phase 0 governance files
- Phase 1 Rust workspace scaffold
- Phase 2 typed config and secret-reference boundary

## Implementation Sequence

1. Reconcile governance files and current roadmap position.
2. Create this Phase 3 sub-roadmap before code changes.
3. Add isolated `policy.rs` module in `arb-core`.
4. Export policy types from `arb-core`.
5. Add CLI config-load policy initialization message.
6. Update static structure validator.
7. Update governance and gap tracker.
8. Run available validation.
9. Record deferred Rust/Cargo validation.

## Validation Sequence

### Executable In ChatGPT Project Mode

- `python3 scripts/validate_structure.py`
- Static file-existence validation
- Static secret-assignment scan
- Manual governance reconciliation

### Deferred Outside ChatGPT Project Mode

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Property tests for policy invariants
- Fuzzing of generated execution intents
- Mutation tests for denial-path resilience
- Runtime integration with future audit and execution subsystems

## Rollback Strategy

1. Remove `crates/arb-core/src/policy.rs`.
2. Remove policy exports from `crates/arb-core/src/lib.rs`.
3. Revert CLI policy initialization message in `crates/arb-agent/src/main.rs`.
4. Remove `PHASE_3_SUBROADMAP.md` from structure validation.
5. Revert governance documents to Phase 2 state.

No secrets, infrastructure, exchange accounts, wallet keys, deployed services, or runtime state are introduced in this phase.

## Drift-Prevention Constraints

- Do not implement execution adapters in Phase 3.
- Do not implement signing in Phase 3.
- Do not load secrets in policy code.
- Do not make live runtime available by default.
- Do not claim Cargo validation unless actually run.
- Do not weaken Phase 2 mode gates.
- Do not permit withdrawals.
- Do not permit LLM-generated destinations.

## Environment Limitations

Current workspace Rust/Cargo validation has local and GitHub Actions evidence for formatting, compilation, unit tests, and clippy. Fuzzing and property tests remain future validation work.

## Expected Unresolved Gaps

- Rust/Cargo validation deferred.
- Policy engine not integrated with audit journal yet.
- No persistent destination allowlist exists yet.
- No market-data provenance subsystem exists yet.
- No execution adapter calls policy yet.
- No signer boundary exists yet.
- No property/fuzz test framework exists yet.

## Expected Future Continuation Tasks

- Add audit journal in Phase 4.
- Make every future execution intent produce an audit event before and after policy evaluation.
- Add policy property tests in Phase 15.
- Add connector-specific policy adapters once CEX/DEX connector frameworks exist.
- Add persistent destination allowlists before any real transfer or withdrawal capability.
- Keep live runtime unavailable until audit, custody, connectors, simulation, rollback, and external validation complete.

## Phase 3 Completion Status

Implemented in ChatGPT Project Mode with Python structure validation only. Rust/Cargo validation is deferred and tracked in `PRODUCTION_GAP_TRACKER.md`.
