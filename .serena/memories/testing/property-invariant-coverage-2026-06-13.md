# Phase 15 local property-invariant coverage (2026-06-13)

- Added `proptest = "1.5"` as an `arb-core` dev-dependency.
- Added local property tests in `crates/arb-core/src/opportunity.rs` covering:
  - candidate truncation + descending-profit ordering under varying `max_candidates`
  - depth/inventory liquidity caps never exceeding caller-supplied order-book or inventory bounds
  - stale quotes always failing closed with `OPPORTUNITY_QUOTE_STALE`
- This advances GAP-0066 and Phase 15 local validation coverage without introducing external fuzz engines, live network calls, credentials, signing, broadcasts, or production claims.
- Updated `PRODUCTION_GAP_TRACKER.md`, `ROADMAP.md`, and `ARCHITECTURE.md` to acknowledge local `proptest` invariant coverage while keeping external fuzzing/load/security/runtime blockers open.
- Validation after patch:
  - `rtk python3 scripts/validate_structure.py` passed
  - `rtk cargo fmt --check` passed
  - `rtk cargo check --workspace` passed
  - `rtk cargo test --workspace` passed (`478 passed`)
  - `rtk cargo clippy --workspace --all-targets -- -D warnings` passed