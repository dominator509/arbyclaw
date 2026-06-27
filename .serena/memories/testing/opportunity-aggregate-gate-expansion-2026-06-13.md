# Opportunity aggregate gate expansion (2026-06-13)

- Extended `scripts/validate_opportunity_scenario_gate.py` to include `validate-local-validation-corpus --workspace <fresh-dir>` and `validate-local-paper-backtest-corpus --workspace <fresh-dir>`.
- Added dangerous-flag enforcement for `external-fuzzer-invoked`, `live-network-used`, and `live-execution-submitted`, closing a safety gap where the aggregate script had not been checking those outputs.
- Added assertions for local validation-corpus accepted-plan/property-check recovery and local paper-backtest filled/partial/unfilled replay coverage.
- The aggregate gate now passes with `component_count = 10` and all unsafe side-effect flags false.
- Updated `ROADMAP.md`, `PHASE_29_SUBROADMAP.md`, and `PRODUCTION_GAP_TRACKER.md` so Phase 29 and GAP-0054/GAP-0066 describe the stronger aggregate local scenario/backtest coverage.
- Validation after the change passed: `rtk python3 scripts/validate_opportunity_scenario_gate.py --json`, `rtk python3 -m py_compile scripts/validate_opportunity_scenario_gate.py`, `rtk python3 scripts/validate_structure.py`, `rtk cargo fmt --check`, `rtk cargo check --workspace`, `rtk cargo test --workspace` (495 passed), and `rtk cargo clippy --workspace --all-targets -- -D warnings`.