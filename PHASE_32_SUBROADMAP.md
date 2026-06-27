# PHASE_32_SUBROADMAP.md

## Phase

Phase 32 - Local DEX/Web3 Request Plans

## Status

Implemented for local deterministic DEX/Web3 request-plan validation only.

## Goal

Add typed local request plans for future DEX/router/RPC quote and simulation adapter shapes, and validate those plans through existing local quote/simulation request boundaries without performing HTTP calls, RPC calls, signing, broadcasts, bridges, credential loading, or live execution.

## Completed Tasks

- Created `PHASE_32_SUBROADMAP.md`.
- Added `DexRequestPlanKind` and `DexRequestPlan`.
- Added local request-plan constructors for Uniswap V3 quoter `eth_call`, 0x swap quote HTTP, Jupiter quote HTTP, and EVM transaction simulation `eth_call`.
- Added fail-closed request-plan validation for HTTP/RPC shape mismatch, malformed local path metadata, invalid venue kind, invalid pair metadata, and side-effect flags.
- Added conversion from quote-capable plans into existing `DexSwapQuoteRequest` records.
- Added conversion from simulation-capable plans into existing `Web3TransactionSimulationRequest` records.
- Added Rust tests for request-plan counts, quote/simulation conversion, side-effect denial, and wrong-capability conversion denial.
- Added `arb-agent validate-dex-request-plans`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Updated structure validation for Phase 32 files.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, wallet custody, signing, withdrawals, bridges, broadcasts, or external execution.
- No raw transaction construction for broadcast.
- No production deployment or production-readiness approval.
- No claim that local request plans are live adapter implementations.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-request-plans
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local DEX/Web3 request-plan modeling only. Live RPC clients, router/aggregator integrations, transaction simulation providers, nonce handling, custody-backed signing, broadcasts, bridges, sandbox/live provider validation, deployment-host connector validation, and production readiness remain unclaimed.
