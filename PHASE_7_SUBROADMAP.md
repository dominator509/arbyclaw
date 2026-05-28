# PHASE_7_SUBROADMAP.md

## Phase 7 — CEX Connector Framework

## Objectives

Create a centralized-exchange connector framework that future live exchange adapters can implement without broad rewrites. This phase defines typed venue profiles, capability declarations, CEX order request models, framework-level order validation, policy-gated paper/sandbox order validation, and connector traits.

Phase 7 deliberately does **not** add exchange-specific REST/WebSocket implementations, credentials, account access, live order submission, withdrawal behavior, or balance mutation.

## Deliverables

- `crates/arb-core/src/cex.rs`
- CEX venue profile model
- CEX capability model
- CEX connector registry
- CEX order request model
- CEX order side/type/time-in-force enums
- CEX policy-gate validator
- CEX connector trait boundaries
- Public exports from `arb-core`
- CLI status text update
- Structure validator update
- Roadmap, architecture, security, README, and gap-tracker updates

## Subsystem Boundaries

### In Scope

- CEX framework types
- Read-only connector identity traits
- Trading connector trait boundary
- Capability validation
- Policy-gated paper/sandbox order validation
- Live-order denial for Phase 7

### Out of Scope

- Real exchange REST clients
- Real exchange WebSocket clients
- Real API credentials
- Real balance reads
- Real order placement
- Real order cancellation
- Real fill streams
- Real withdrawal/transfer APIs
- DEX/Web3 behavior
- Wallet signing
- Runtime daemon orchestration

## Dependencies

- Phase 1 Rust workspace scaffold
- Phase 2 typed config and secret-reference boundary
- Phase 3 policy engine
- Phase 4 audit/state boundaries
- Phase 5 market-data and fee model boundaries
- Phase 6 paper connector scaffolding

## Implementation Sequence

1. Reconcile governance files.
2. Create `PHASE_7_SUBROADMAP.md` before code changes.
3. Add isolated `cex` module.
4. Export CEX framework types from `arb-core`.
5. Update CLI status output.
6. Update structure validation.
7. Update governance and safety docs.
8. Run available validation.
9. Record environment-limited validations and new gaps.

## Validation Sequence

Executable inside ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Future connector validation:

- REST market-data integration tests
- WebSocket reconnect tests
- rate-limit tests
- sandbox order tests where available
- fee-schedule verification
- jurisdiction/terms review
- credential-scope verification

## Rollback Strategy

Rollback Phase 7 by:

1. Removing `PHASE_7_SUBROADMAP.md`.
2. Removing `crates/arb-core/src/cex.rs`.
3. Removing CEX exports from `crates/arb-core/src/lib.rs`.
4. Reverting `arb-agent` status text.
5. Reverting `scripts/validate_structure.py`.
6. Reverting governance docs to Phase 6 state.

No secrets, exchange accounts, runtime state, network integrations, or infrastructure are introduced in this phase.

## Drift-Prevention Constraints

- Do not add exchange-specific clients in Phase 7.
- Do not load credentials.
- Do not connect to live networks.
- Do not submit orders.
- Do not mutate balances.
- Do not weaken policy checks for convenience.
- Do not mark CEX integrations production-ready.
- Keep all live behavior denied until later execution, audit, custody, and external validation phases.

## Environment Limitations

Rust/Cargo, exchange sandboxes, network calls, live API credentials, CI execution, rate-limit tests, and exchange terms/jurisdiction review are unavailable in ChatGPT Project Mode.

## Expected Unresolved Gaps

- Rust validation remains deferred.
- No real CEX REST/WebSocket connector exists.
- No sandbox exchange validation exists.
- No credential-scope validation exists.
- No exchange-specific fee/rate-limit verification exists.
- CEX framework is not integrated with runtime orchestration.
- CEX framework is not integrated with durable audit/state writes.

## Expected Future Continuation Tasks

- Implement read-only REST/WebSocket adapters per exchange.
- Add sandbox adapters where supported by exchanges.
- Add authenticated balance read boundaries.
- Add connector-specific rate-limit controllers.
- Add audit-before-action integration for every order request.
- Add order lifecycle state machine.
- Add external exchange terms/jurisdiction review checklist.
- Add eventual live order adapters only after policy, audit, state, secrets, and signer/custody phases are validated.
