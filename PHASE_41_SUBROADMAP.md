# PHASE_41_SUBROADMAP.md

## Phase

Phase 41 - Local Web3 Nonce Reservation

## Status

Implemented for local deterministic Web3 nonce reservation only.

## Goal

Add a typed local nonce reservation boundary before pre-sign safety review, proving caller-supplied nonce metadata can reject missing, stale, duplicated, or already reserved nonces and persist the accepted local reservation to audit and SQLite WAL state without calling RPC, loading signer material, signing, broadcasting, executing live transactions, or claiming production readiness.

## Completed Tasks

- Created `PHASE_41_SUBROADMAP.md`.
- Added `Web3NonceReservationRequest`, `Web3NonceReservationReport`, and `Web3NonceReservationStatus`.
- Added deterministic local review checks for chain/venue/account metadata, requested nonce presence, stale nonce denial, duplicate in-flight nonce denial, already-reserved nonce denial, TTL, timestamp, and side-effect denial.
- Added append-only audit and SQLite WAL checkpoint helpers for local Web3 nonce reservation reports.
- Added Rust tests for ready local reservation, stale/duplicate blocking, side-effect denial, audit replay, and SQLite checkpoint reopen.
- Added `arb-agent validate-web3-nonce-reservation`.
- Wired the CLI into CI before Web3 pre-sign safety validation.

## Explicit Non-Goals

- No live trading.
- No credential loading, signer material loading, plaintext decryption, transaction signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, external submission, or production execution.
- No provider-backed nonce retrieval, custody provider, hardware wallet, OS keyring integration, live signer provider, live RPC adapter, or production-readiness approval.
- No claim that local nonce reservation is live chain nonce management, custody, signer, RPC, wallet, transaction simulation, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-web3-nonce-reservation
cargo test -p arb-core web3_nonce -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local Web3 nonce reservation only. Provider-backed nonce retrieval, mempool-aware replacement/expiration, custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, real transaction construction, broadcasts, deployment signer isolation, and external custody/RPC validation remain unclaimed.
