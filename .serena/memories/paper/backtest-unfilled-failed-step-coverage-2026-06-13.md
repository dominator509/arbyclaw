# Paper backtest unfilled-step coverage (2026-06-13)

- Added `paper_backtest_corpus_tracks_unfilled_failed_steps_locally` in `crates/arb-core/src/paper.rs`.
- The test executes a one-step local historical-fixture backtest corpus with `fill_request(40.0, 80, false)` so the realistic fill model returns `Unfilled` under the existing partial-fill-denied policy.
- Assertions cover run-level and scenario-level counts (`filled_steps == 0`, `partially_filled_steps == 0`, `unfilled_steps == 1`), zero net profit, replay closure, and final balance restoration to the starting 1000 USDC with no residual reservation.
- `PRODUCTION_GAP_TRACKER.md` GAP-0009 was reconciled so `failed-trade simulation tests` is no longer listed as future-only validation, because local deterministic failed/unfilled paper coverage now exists in unit tests and the local paper backtest CLI/report path already records filled/partial/unfilled outcomes.
- Full validation after the change passed: `rtk python3 scripts/validate_structure.py`, `rtk cargo fmt --check`, `rtk cargo check --workspace`, `rtk cargo test --workspace` (495 passed), and `rtk cargo clippy --workspace --all-targets -- -D warnings`.