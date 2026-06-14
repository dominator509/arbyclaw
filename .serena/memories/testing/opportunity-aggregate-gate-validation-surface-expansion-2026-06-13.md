# Opportunity aggregate gate expansion for validation-run/property/fuzz coverage (2026-06-13)

- Expanded `scripts/validate_opportunity_scenario_gate.py` from 10 to 13 components.
- Added fresh-temp-workspace execution and aggregate assertions for:
  - `arb-agent validate-local-validation-run --workspace <fresh-dir>`
  - `arb-agent validate-local-property-checks --workspace <fresh-dir>`
  - `arb-agent validate-local-fuzz-corpus --workspace <fresh-dir>`
- The gate now validates:
  - validation-run status `planned-only` with planned test/fixture/fuzz/backtest counts present and checkpoint recovery true
  - property-check executed == passed, failed/missing-fixture/empty-fuzz/nonlocal-backtest counts all zero, checkpoint recovery true
  - fuzz-corpus replay status `ready-for-local-review`, nonzero corpus/seed/target counts, unique seed count matching total seeds, checkpoint recovery true
- Existing fresh-workspace validation-corpus and paper-backtest probes remain in place, as do unsafe-flag denials for external calls/data downloads/fuzzer/network/live execution/adapter submission/signing/broadcast/production ready.
- Updated `ROADMAP.md`, `PHASE_29_SUBROADMAP.md`, and `PRODUCTION_GAP_TRACKER.md` so Phase 29 / GAP-0066 wording now reflects 13 aggregate components and explicit validation-run/property-check/fuzz-corpus inclusion.
- Validation passed after the change: `rtk python3 -m py_compile scripts/validate_opportunity_scenario_gate.py`, `rtk python3 scripts/validate_opportunity_scenario_gate.py --json` (13 components), `rtk python3 scripts/validate_structure.py`, `rtk cargo fmt --check`, `rtk cargo check --workspace`, `rtk cargo test --workspace` (495 passed), and `rtk cargo clippy --workspace --all-targets -- -D warnings`.
- This strengthens local deterministic testing/backtest coverage only; external fuzz engines, broader external/deployment corpora, sandbox/live calibration, and production runtime validation remain open blockers.