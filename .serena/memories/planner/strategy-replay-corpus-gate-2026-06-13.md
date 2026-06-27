# Strategy replay corpus gate (2026-06-13)

- `crates/arb-core/src/planner.rs` now exposes `validate_strategy_profile_replay_corpus(...)`.
- The validator replays `phase27_local_opportunity_historical_fixture_corpus()` through accepted and rejected `StrategyProfile` inputs using `DeterministicExecutionPlanner::plan_with_strategy_profile(...)`.
- Expected local invariant: accepted profile stays `draft-ready` with zero rejected intents; rejected profile yields `policy-denied-draft` for every discovered candidate and rejects every intent.
- Side-effect invariant remains fail-closed: no adapter submission, external calls, live execution, signing/broadcast, or production-ready flags.
- `crates/arb-agent/src/main.rs` now exposes `arb-agent validate-strategy-replay-corpus`, and `scripts/validate_opportunity_scenario_gate.py` includes it in the aggregate local opportunity gate.
- Docs reconciled in `ARCHITECTURE.md` and `PRODUCTION_GAP_TRACKER.md`; GAP-0028 now treats local replay/corpus validation as existing while keeping profitability tuning, config migration expansion, external calibration, and production validation open.
- Full RTK validation after the change passed on 2026-06-13: structure manifest generation, py_compile for validation scripts, structure validation, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (468 passed), and `cargo clippy --workspace --all-targets -- -D warnings`.