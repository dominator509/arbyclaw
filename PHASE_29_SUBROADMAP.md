# PHASE_29_SUBROADMAP.md

## Phase

Phase 29 - Opportunity Scenario Aggregate Gate

## Status

Implemented for local deterministic aggregate opportunity scenario-corpus validation only.

## Goal

Compose the existing local opportunity replay, quote-load, provider-ingestion,
historical-fixture, planner-handoff, and trace-recovery CLI probes into one
stronger gate that verifies the local opportunity scenario corpus remains
deterministic, broad enough to cover the current built-in fixture families, and
free of live/external side effects.

## Scope

- Add `scripts/validate_opportunity_scenario_gate.py`.
- Run the existing local-only opportunity CLIs as one aggregate gate.
- Fail closed on external calls, external data downloads, adapter submission,
  signing, broadcasts, live execution, or production-readiness claims.
- Verify replay iterations pass, quote-load backpressure is exercised,
  historical fixtures pass, planner handoff trace counts match, and trace
  recovery reports no missing checkpoints.
- Wire the aggregate gate into CI.
- Keep external/deployment scenario-corpus and sandbox/live calibration blockers
  open unless real external evidence exists.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, RPC, market-data provider, or backtest data calls.
- No adapter submission, signing, withdrawals, bridges, broadcasts, or wallet custody.
- No production deployment or production-readiness approval.
- No claim that local synthetic/recorded fixtures are external sandbox/live evidence.

## Validation

Required after this phase:

```bash
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local aggregate opportunity scenario-corpus validation only when the
script passes locally, CI includes the gate, structure validation includes the
new phase and script, and governance files record that broader external
scenario-corpus execution, live/provider-backed market-data validation,
sandbox/live calibration, and production runtime validation remain open.
