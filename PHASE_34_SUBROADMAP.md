# PHASE_34_SUBROADMAP.md

## Phase

Phase 34 - Local CEX Order Lifecycle Transcript Parsing

## Status

Implemented for local deterministic CEX order lifecycle transcript parsing only.

## Goal

Parse caller-supplied local Binance-, Coinbase-, and Kraken-shaped order lifecycle JSON transcripts into existing local `CexOrderLifecycleResponse` records, then reconcile filled and cancelled-after-partial lifecycles through the existing CEX lifecycle audit/state gate without performing REST calls, opening WebSockets, loading credentials, submitting orders, cancelling orders, or claiming live adapter readiness.

## Completed Tasks

- Created `PHASE_34_SUBROADMAP.md`.
- Added `CexOrderLifecycleTranscript` and `CexOrderLifecycleTranscriptFormat`.
- Added fail-closed transcript validation for malformed metadata, malformed JSON, side-effect flags, validation-record venue/pair mismatch, and unknown statuses.
- Added local parsing for Binance execution-report, Coinbase order-event, and Kraken order-status payload shapes.
- Wired `arb-agent validate-connector-lifecycle-audit` to parse local CEX lifecycle transcripts before reconciliation.
- Added Rust tests for successful exchange-shaped lifecycle transcript parsing, side-effect denial, and validation-record mismatch denial.
- Added aggregate connector scenario assertion for the parsed CEX lifecycle transcript count.
- Added local cancelled-after-partial lifecycle transcript reconciliation with audit replay, SQLite checkpoint recovery, and aggregate gate assertions.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, wallet custody, signing, withdrawals, bridges, broadcasts, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local lifecycle transcripts are live exchange responses.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-connector-lifecycle-audit --workspace <fresh-dir>
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local CEX order lifecycle transcript parsing and local cancelled-after-partial reconciliation only. Live REST/WebSocket clients, credentialed account calls, sandbox/live exchange responses, production idempotency, rate-limit reconciliation, cancel/reconciliation adapters, deployment-host connector validation, and production readiness remain unclaimed.
