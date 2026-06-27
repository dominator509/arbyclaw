# PHASE_36_SUBROADMAP.md

## Phase

Phase 36 - Local DEX/Web3 Transaction Lifecycle Transcript Parsing

## Status

Implemented for local deterministic DEX/Web3 transaction lifecycle transcript parsing only.

## Goal

Parse caller-supplied local EVM transaction receipt and Solana signature-status JSON into normalized local Web3 transaction lifecycle records with nonce and confirmation accounting, without performing RPC calls, loading credentials, loading signer material, signing, broadcasting, bridging, submitting transactions, or claiming live adapter readiness.

## Completed Tasks

- Created `PHASE_36_SUBROADMAP.md`.
- Added `Web3TransactionLifecycleTranscript`, `Web3TransactionLifecycleTranscriptFormat`, `Web3TransactionLifecycleRecord`, and `Web3TransactionLifecycleStatus`.
- Added local parsing for EVM transaction receipt/status payloads and Solana signature-status payloads.
- Added fail-closed validation for malformed metadata, malformed JSON, missing transaction identifiers, side-effect flags, live RPC response flags, signer material loading, signing, broadcast, bridge, live execution, production-readiness claims, and confirmed statuses without local confirmation evidence.
- Added Rust tests for successful EVM/Solana local lifecycle parsing, nonce tracking, confirmation accounting, side-effect denial, and missing-confirmation denial.
- Added `arb-agent validate-dex-transaction-lifecycle-transcripts`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Added aggregate connector scenario assertions for transaction lifecycle transcript counts, parsed record counts, confirmed/reverted/failed status counts, and nonce tracking counts.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, signer, wallet, bridge, or backtest data calls.
- No credential loading, signer material loading, transaction construction, account query, adapter submission, signing, withdrawals, bridges, broadcasts, live order submission, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local transaction lifecycle records are live RPC, testnet, mainnet, signer, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-transaction-lifecycle-transcripts
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local DEX/Web3 transaction lifecycle transcript parsing only. Live RPC adapters, custody-backed signing, transaction construction, nonce management against live chains, broadcast controls, testnet/mainnet simulation replay, external confirmation reconciliation, deployment restart recovery, and production readiness remain unclaimed.
