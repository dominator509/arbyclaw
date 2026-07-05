# Phase 125 - Hardening Core Opportunity Scenario Aggregate Gate

## Scope

Promote the existing local opportunity-scenario aggregate validator into the hardening-core aggregate gate so local hardening evidence requires opportunity replay, validation corpus, property/fuzz corpus replay, paper backtest, planner handoff, and trace-recovery controls before hardening can pass.

## Implemented Local Work

- Added `scripts/validate_opportunity_scenario_gate.py --json` as a required `opportunity_scenario_gate` component in `scripts/validate_hardening_core_gate.py`.
- Added aggregate assertions for the 14-component opportunity-scenario report, full component pass status, replay-latency review enforcement, validation-coverage review enforcement, candidate coverage, no unsafe side-effect flags, no external calls, no external data downloads, no adapter submission, no external fuzzer invocation, no live network use, no signing or broadcast, no live execution, and no production-readiness flag.

## Explicit Non-Scope

- No live/provider-backed market data calls.
- No external data downloads, external fuzz engine execution, adapter submission, signing, broadcasts, live execution, or production-readiness claim.

## Remaining Production Blockers

- Broader external/deployment scenario-corpus execution.
- External sandbox/live calibration evidence.
- Live/provider-backed market-data validation.
- External fuzzing-engine and broader production backtest execution.
- Production runtime validation.
