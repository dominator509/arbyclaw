# Phase 95 - CEX Live Adapter Boundary Review Gate

## Goal

Replace the loose CEX live-adapter "not implemented" dead end with a typed local review boundary that validates local prerequisites and keeps live exchange adapter implementation blocked until sandbox/live evidence exists.

## Completed Tasks

- Added `CexLiveAdapterBoundaryReviewRequest`, `CexLiveAdapterBoundaryReviewReport`, and status typing.
- Added `review_cex_live_adapter_boundary()` with fail-closed side-effect validation and explicit blocker codes.
- Added focused Rust tests for blocked local prerequisite review and side-effect rejection.
- Added `arb-agent validate-cex-live-adapter-boundary`.
- Wired the new command into `scripts/validate_connector_scenario_gate.py`, raising the connector aggregate to 20 components.

## Non-Goals

- No REST calls.
- No WebSocket connections.
- No credential loading.
- No live exchange orders.
- No cancel submission.
- No account reads.
- No live execution or production-readiness claims.

## Validation

- `cargo test -p arb-core cex_live_adapter_boundary -- --nocapture`
- `cargo run -p arb-agent -- validate-cex-live-adapter-boundary`
- `python scripts/validate_connector_scenario_gate.py --json`
- `python scripts/validate_structure.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Exit Criteria

- The CEX live-adapter boundary is typed, locally validated, included in the connector aggregate gate, and remains blocked until real sandbox/live adapter evidence is available.
