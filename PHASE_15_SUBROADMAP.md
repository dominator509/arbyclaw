# PHASE_15_SUBROADMAP.md

## Phase

Phase 15 — Testing, Fuzzing, and Backtesting

## Governance Status

Created before Phase 15 implementation. This file is authoritative for Phase 15 scope until the phase is completed and the roadmap advances.

## Preconditions Reconciled

- `ARCHITECTURE.md` reviewed.
- `ROADMAP.md` reviewed.
- `PHASE_14_SUBROADMAP.md` reviewed as the latest completed sub-roadmap.
- `AGENTS.md` reviewed.
- `PRODUCTION_GAP_TRACKER.md` reviewed.
- `scripts/validate_structure.py` passed before Phase 15 code changes.
- Phases 0 through 14 are documented complete for ChatGPT Project Mode scope.

## Goal

Add deterministic testing, fuzzing, fixture, and backtesting boundary models that describe how validation work is planned, constrained, and recorded without invoking external fuzzers, live networks, real exchange/RPC calls, credentials, signing, broadcasts, or live execution.

## Scope

Phase 15 may add:

- deterministic validation harness configuration
- test case definitions
- fixture metadata records
- fuzz corpus and seed metadata models
- backtest dataset and scenario metadata models
- validation plan and local run record models
- fail-closed checks for live network usage, external fuzzer invocation, live execution, credentials, withdrawals, bridges, signing, and broadcasts
- local-only deterministic validation harness trait and implementation
- exports through `arb-core`
- CLI/status text declaring validation boundary availability
- structure validator updates
- governance documentation updates
- gap tracker updates

## Explicit Non-Goals

Phase 15 must not add:

- live trading
- order placement
- adapter submission
- live exchange calls
- live DEX/RPC calls
- wallet signing
- transaction broadcasts
- withdrawals
- bridges
- real credentials or secret examples
- external fuzzer process invocation
- CI fuzzing jobs that assume unavailable tooling
- real backtest market-data downloads
- live network test runners
- production deployment packaging
- dashboard hosting
- outbound alert delivery
- policy bypass

## Subsystem Boundaries

Phase 15 belongs to the validation subsystem and must remain downstream of existing boundaries:

- `arb-core::config`
- `arb-core::policy`
- `arb-core::audit`
- `arb-core::market_data`
- `arb-core::fees`
- `arb-core::paper`
- `arb-core::cex`
- `arb-core::dex`
- `arb-core::opportunity`
- `arb-core::planner`
- `arb-core::execution_adapter`
- `arb-core::communications`
- `arb-core::dashboard`
- `arb-core::observability`

The validation subsystem may model test coverage and fixtures, but it must not become an execution path or a hidden live adapter.

## Implementation Plan

1. Add `crates/arb-core/src/testing.rs`.
2. Define `TESTING_BACKTESTING_VERSION`.
3. Define validation harness config with fail-closed live/external toggles.
4. Define test suite, test case, fixture, fuzz corpus, and backtest scenario models.
5. Define validation plan and run request/record models.
6. Add deterministic harness trait and local-only implementation.
7. Add validation helpers for IDs, names, duplicate records, counts, secret-like text, and live-scope denial.
8. Export Phase 15 types from `arb-core`.
9. Surface Phase 15 status in `arb-agent`.
10. Update `scripts/validate_structure.py` to require Phase 15 files and module.
11. Update governance documentation.
12. Run available validation commands.

## Validation Plan

Run in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempt but do not claim success unless available:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Completion Criteria

- `PHASE_15_SUBROADMAP.md` exists before code implementation.
- Phase 15 module exists and is exported.
- Validation harness is deterministic and local-only.
- Fuzzing and backtesting are represented as model/fixture boundaries only.
- Unsafe toggles fail closed.
- No live network, external fuzzer process, real exchange/RPC, signing, broadcast, bridge, withdrawal, or adapter-submission path exists.
- Governance docs and gap tracker reflect Phase 15 status and remaining risks.
- Structure validator passes.

## Rollback Plan

1. Remove `crates/arb-core/src/testing.rs`.
2. Remove testing exports from `crates/arb-core/src/lib.rs`.
3. Revert `arb-agent` Phase 15 status output.
4. Remove Phase 15 requirements from `scripts/validate_structure.py`.
5. Revert `ROADMAP.md`, `ARCHITECTURE.md`, `README.md`, `SECURITY.md`, `AGENTS.md`, `HANDOFF_CONTEXT.md`, `STRUCTURE_MANIFEST.md`, and `PRODUCTION_GAP_TRACKER.md` to the Phase 14 state.
6. Run `python3 scripts/validate_structure.py`.

## Final Status

Completed for ChatGPT Project Mode scope as a deterministic testing, fuzzing, fixture, and backtesting boundary. External Rust validation, real property-test runners, fuzzing engines, fixture corpus expansion, market replay validation, load tests, penetration tests, and production deployment tests remain deferred.
