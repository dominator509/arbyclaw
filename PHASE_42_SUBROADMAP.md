# PHASE_42_SUBROADMAP.md

## Phase

Phase 42 - Local Web3 Unsigned Payload Review

## Status

Implemented for local deterministic unsigned payload metadata review only.

## Goal

Add a typed local unsigned payload review boundary between nonce reservation and pre-sign safety review, proving a reviewed payload hash/label, router/spender labels, gas cap metadata, and nonce reservation can be checked and persisted without embedding raw calldata, constructing a live transaction, calling RPC, loading signer material, signing, broadcasting, executing live transactions, or claiming production readiness.

## Completed Tasks

- Created `PHASE_42_SUBROADMAP.md`.
- Added `Web3UnsignedPayloadReviewRequest`, `Web3UnsignedPayloadReviewReport`, and `Web3UnsignedPayloadReviewStatus`.
- Added deterministic local review checks for simulation request validity, nonce reservation readiness, payload hash/label matching, router/spender label coherence, gas cap metadata, raw-calldata denial, and side-effect denial.
- Added append-only audit and SQLite WAL checkpoint helpers for local Web3 unsigned payload review reports.
- Added Rust tests for ready local review, nonce/payload/router/gas blocking, side-effect denial, audit replay, and SQLite checkpoint reopen.
- Added `arb-agent validate-web3-unsigned-payload-review`.
- Wired the CLI into CI between nonce reservation and pre-sign safety validation.

## Explicit Non-Goals

- No live trading.
- No raw calldata construction, credential loading, signer material loading, plaintext decryption, transaction signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, external submission, or production execution.
- No provider-backed nonce retrieval, custody provider, hardware wallet, OS keyring integration, live signer provider, live RPC adapter, or production-readiness approval.
- No claim that local unsigned payload review is live transaction construction, live transaction simulation, custody, signer, RPC, wallet, nonce, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-web3-unsigned-payload-review
cargo test -p arb-core web3_unsigned -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local unsigned payload metadata review only. Real transaction construction, raw calldata generation, provider-backed nonce retrieval, mempool-aware replacement/expiration, custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, broadcasts, deployment signer isolation, and external custody/RPC validation remain unclaimed.
