# Strategy profitability tuning gate (2026-06-13)

- `crates/arb-core/src/planner.rs` now exposes `validate_strategy_profitability_tuning(...)`.
- The validator derives a deterministic low/median/high threshold sweep from observed local replay intent net profit across `phase27_local_opportunity_historical_fixture_corpus()`, then checks monotonic draft-ready vs policy-denied behavior as thresholds rise.
- Expected local invariants: monotonic acceptance decreases, monotonic rejection increases, the highest derived threshold denies every discovered candidate, and all side-effect flags remain false.
- `crates/arb-agent/src/main.rs` now exposes `arb-agent validate-strategy-profitability-tuning`, and `scripts/validate_opportunity_scenario_gate.py` includes it in the aggregate local opportunity gate.
- Docs reconciled in `ARCHITECTURE.md`, `ROADMAP.md`, and `PRODUCTION_GAP_TRACKER.md`; GAP-0028 now treats local profitability tuning as implemented while keeping external calibration and production/runtime validation open.
- `HANDOFF_CONTEXT.md` still has an encoding issue that prevents safe `apply_patch` edits in-session.
- Full RTK validation after the change passed on 2026-06-13: structure manifest generation, py_compile for validation scripts, structure validation, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (473 passed), and `cargo clippy --workspace --all-targets -- -D warnings`.