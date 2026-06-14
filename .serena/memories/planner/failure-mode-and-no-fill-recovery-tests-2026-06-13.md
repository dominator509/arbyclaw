# Planner failure-mode and no-fill recovery coverage (2026-06-13)

- Added `planner_assigns_route_specific_failure_modes_to_draft_steps` in `crates/arb-core/src/planner.rs`.
- The test proves deterministic CEX drafts keep `CancelUnfilledRemainder` coverage on prepare steps, deterministic DEX drafts add `DoNotSignOrBroadcast` coverage on prepare steps, and draft plans still terminate with `ManualReviewRequired` without adapter submission.
- Added `dex_candidate()` planner test helper by mutating the existing deterministic candidate into a DEX/DEX route with DEX venue kinds and corresponding quote ids.
- Added `adapter_recovery_plan_models_no_fill_cancel_without_hedge` in `crates/arb-core/src/execution_adapter.rs`.
- That test uses the existing kill-switch-denied adapter path to prove no-fill runs produce cancel-only recovery steps, `no_fill_count == plan.intents.len()`, `hedge_exposure_steps == 0`, and no external submission/live execution/production-ready flags.
- `PRODUCTION_GAP_TRACKER.md` now records this validation attempt and updates GAP-0013/GAP-0056/GAP-0058 phrasing so local failure-mode/cancellation/no-fill recovery coverage is treated as existing local evidence rather than future-only work.
- Validation after the change passed on 2026-06-13: `cargo fmt --check`, `cargo test --workspace` (465 passed), `cargo clippy --workspace --all-targets -- -D warnings`, and `python3 scripts/validate_structure.py`.