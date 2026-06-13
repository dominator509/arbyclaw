# PHASE_31_SUBROADMAP.md

## Phase

Phase 31 - Local CEX Market-Data Request Plans

## Status

Implemented for local deterministic exchange-specific request-plan validation only.

## Goal

Add typed, local-only Binance/Coinbase/Kraken market-data request plans for REST depth/book and WebSocket depth/book subscription shapes, and validate those plans against caller-supplied local transcripts without performing network calls.

## Completed Tasks

- Created `PHASE_31_SUBROADMAP.md`.
- Added `CexMarketDataRequestKind` and `CexMarketDataRequestPlan`.
- Added Binance depth REST/WebSocket request-plan constructors.
- Added Coinbase product-book REST/WebSocket request-plan constructors.
- Added Kraken depth REST/WebSocket request-plan constructors.
- Added request-plan validation that fails closed on malformed REST/WebSocket shapes or side-effect flags.
- Added request-plan transcript parsing that requires format, venue, and pair agreement before normalizing a supplied local transcript.
- Added Rust tests for exchange-specific REST/WebSocket shapes, local transcript parsing, side-effect denial, and plan/transcript mismatch denial.
- Added `arb-agent validate-cex-market-data-request-plans`.
- Wired the CLI into CI and the connector scenario aggregate gate.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, or account calls.
- No credential loading, API signing, wallet custody, withdrawals, bridges, broadcasts, or external adapter submission.
- No production deployment or production-readiness approval.
- No claim that request plans are live adapter implementations.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-cex-market-data-request-plans
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local exchange-specific CEX market-data request-plan modeling only. Live REST/WebSocket clients, credentialed account calls, sandbox/live provider validation, order submission/cancel adapters, deployment-host connector validation, and production readiness remain unclaimed.
