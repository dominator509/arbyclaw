# PHASE_43_SUBROADMAP.md

## Phase

Phase 43 - Local Web3 Broadcast Readiness Review

## Status

Implemented for local deterministic broadcast-readiness review only.

## Goal

Add a typed local broadcast-readiness review boundary after unsigned payload and pre-sign safety review, proving prerequisite metadata, signer authorization reference, live adapter reference, and operator approval reference can be checked and persisted while still denying actual broadcast permission. This phase does not call RPC, load signer material, sign, broadcast, execute live transactions, or claim production readiness.

## Completed Tasks

- Created `PHASE_43_SUBROADMAP.md`.
- Added `Web3BroadcastReadinessRequest`, `Web3BroadcastReadinessReport`, and `Web3BroadcastReadinessStatus`.
- Added deterministic local review checks for unsigned-payload readiness, pre-sign safety readiness, prerequisite coherence, non-secret signer authorization reference, non-secret live adapter reference, non-secret operator approval reference, broadcast-permission denial, and side-effect denial.
- Added append-only audit and SQLite WAL checkpoint helpers for local Web3 broadcast-readiness review reports.
- Added Rust tests for ready external review, blocked prerequisite/reference review, side-effect denial, audit replay, and SQLite checkpoint reopen.
- Added `arb-agent validate-web3-broadcast-readiness`.
- Wired the CLI into CI after pre-sign safety validation.

## Explicit Non-Goals

- No live trading.
- No transaction signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, external submission, signer material loading, plaintext decryption, live adapter calls, or production execution.
- No provider-backed nonce retrieval, custody provider, hardware wallet, OS keyring integration, live signer provider, live RPC adapter, broadcast adapter, or production-readiness approval.
- No claim that local broadcast-readiness review authorizes, signs, submits, or broadcasts a transaction.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-web3-broadcast-readiness
cargo test -p arb-core web3_broadcast_readiness -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local broadcast-readiness metadata review only. Real transaction construction, provider-backed nonce retrieval, mempool-aware replacement/expiration, custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, real RPC simulation, broadcasts, broadcast adapter implementation, deployment signer isolation, and external custody/RPC/broadcast validation remain unclaimed.
