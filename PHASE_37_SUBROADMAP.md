# PHASE_37_SUBROADMAP.md

## Phase

Phase 37 - Local DEX/Web3 Protocol Risk Review

## Status

Implemented for local deterministic DEX/Web3 protocol risk review only.

## Goal

Evaluate caller-supplied local DEX/Web3 protocol metadata for chain/pair scope allowlists, router/spender contract hygiene, gas and slippage limits, MEV controls, token metadata and decimals review, protocol terms review, and jurisdiction/incident review without performing RPC calls, loading credentials, loading signer material, signing, broadcasting, bridging, submitting transactions, or claiming live adapter readiness.

## Completed Tasks

- Created `PHASE_37_SUBROADMAP.md`.
- Added `DexProtocolRiskReviewRequest`, `DexProtocolRiskReviewReport`, and `DexProtocolRiskReviewStatus`.
- Added deterministic local review logic for chain/pair allowlisting, router and spender allowlisting, unlimited allowance denial, approval revocation planning, gas/slippage caps, MEV risk limits, public-mempool mitigation review, token metadata review, token contract review, token decimals verification, protocol terms review, and jurisdiction/incident review.
- Added fail-closed validation for malformed metadata, invalid numeric limits, side-effect flags, live RPC flags, signer material loading, signing, broadcast, bridge, live execution, and production-readiness claims.
- Added Rust tests for ready local metadata, blocked local metadata, and side-effect denial.
- Added `arb-agent validate-dex-protocol-risk-review`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Added aggregate connector scenario assertions for ready/blocked review counts, blocker count, and ready-path asset-scope, contract hygiene, token hygiene, gas-slippage, MEV, governance, and terms controls.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, signer, wallet, bridge, or backtest data calls.
- No credential loading, signer material loading, allowance submission, approval transaction construction, account query, adapter submission, signing, withdrawals, bridges, broadcasts, live order submission, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local protocol risk reviews are live contract, RPC, spender, MEV, testnet, mainnet, signer, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-protocol-risk-review
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local DEX/Web3 protocol risk review only. Live RPC adapters, custody-backed signing, transaction construction, real spender/allowance checks, external chain/router/token/jurisdiction/incident verification, live gas estimation, external MEV validation, protocol contract review, testnet/mainnet validation, broadcast controls, deployment restart recovery, and production readiness remain unclaimed.
