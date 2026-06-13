# PHASE_44_SUBROADMAP.md

## Phase

Phase 44 - Local Web3 Unsigned Transaction Construction

## Status

Implemented for local deterministic unsigned transaction metadata construction only.

## Goal

Add a typed local unsigned transaction construction boundary after broadcast-readiness review, proving sanitized payload, selector, encoded-argument digest, nonce, and gas metadata can be assembled into a review-only unsigned transaction reference while still denying raw calldata embedding, raw transaction serialization, signing, broadcasting, RPC calls, live execution, and production-readiness claims.

## Completed Tasks

- Created `PHASE_44_SUBROADMAP.md`.
- Added `Web3UnsignedTransactionConstructionRequest`, `Web3UnsignedTransactionConstructionReport`, and `Web3UnsignedTransactionConstructionStatus`.
- Added deterministic local checks for broadcast-readiness prerequisite coherence, sanitized payload reference, sanitized target/selector/digest metadata, nonce presence, gas metadata, raw-calldata denial, raw-transaction serialization denial, broadcast-permission denial, and side-effect denial.
- Added append-only audit and SQLite WAL checkpoint helpers for local Web3 unsigned transaction construction reports.
- Added Rust tests for ready local construction metadata, blocked metadata, side-effect denial, audit replay, and SQLite checkpoint reopen.
- Added `arb-agent validate-web3-unsigned-transaction-construction`.
- Wired the CLI into CI after broadcast-readiness validation.

## Explicit Non-Goals

- No live trading.
- No raw calldata embedding, raw transaction serialization, transaction signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, external submission, signer material loading, plaintext decryption, live adapter calls, or production execution.
- No provider-backed nonce retrieval, custody provider, hardware wallet, OS keyring integration, live signer provider, live RPC adapter, broadcast adapter, or production-readiness approval.
- No claim that local unsigned transaction construction creates a signable/broadcastable production transaction.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-web3-unsigned-transaction-construction
cargo test -p arb-core web3_unsigned_transaction_construction -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local unsigned transaction metadata construction only. Provider-backed nonce retrieval, raw calldata generation review, real transaction serialization, custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, real RPC simulation, broadcasts, broadcast adapter implementation, deployment signer isolation, and external custody/RPC/broadcast validation remain unclaimed.
