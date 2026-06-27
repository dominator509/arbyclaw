# PHASE_35_SUBROADMAP.md

## Phase

Phase 35 - Local CEX Balance Snapshot Transcript Parsing

## Status

Implemented for local deterministic CEX balance snapshot transcript parsing only.

## Goal

Parse caller-supplied local Binance-, Coinbase-, and Kraken-shaped balance snapshot JSON into normalized local CEX balance records without performing REST calls, opening WebSockets, loading credentials, querying account state, mutating balances, submitting or cancelling orders, or claiming live adapter readiness.

## Completed Tasks

- Created `PHASE_35_SUBROADMAP.md`.
- Added `CexBalanceSnapshotTranscript`, `CexBalanceSnapshotTranscriptFormat`, `CexAssetBalanceSnapshot`, and `CexBalanceSnapshotRecord`.
- Added fail-closed transcript validation for malformed metadata, malformed JSON, duplicate assets, invalid balances, side-effect flags, credential loading, account-state query flags, and production-readiness claims.
- Added local parsing for Binance account balances, Coinbase accounts, and Kraken balance payload shapes.
- Added `arb-agent validate-cex-balance-snapshots`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Added aggregate connector scenario assertions for parsed balance snapshot counts and asset counts.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No account state queries, credential loading, balance mutation, adapter submission, signing, withdrawals, bridges, broadcasts, live order submission, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local balance snapshots are live exchange account reads.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-cex-balance-snapshots
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local CEX balance snapshot transcript parsing only. Authenticated live balance reads, credentialed account calls, sandbox/live exchange account validation, balance reconciliation against real venues, live REST/WebSocket adapters, and production readiness remain unclaimed.
