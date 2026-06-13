# PHASE_40_SUBROADMAP.md

## Phase

Phase 40 - Local Web3 Pre-Sign Safety Review

## Status

Implemented for local deterministic Web3 pre-sign safety review only.

## Goal

Require a typed local pre-sign Web3 safety review before any future signer or live RPC adapter can use transaction metadata, proving local simulation success, gas-fee caps, minimum-output checks, nonce metadata, lifecycle coherence, audit persistence, and SQLite checkpoint recovery without calling RPC, loading signer material, signing, broadcasting, executing live transactions, or claiming production readiness.

## Completed Tasks

- Created `PHASE_40_SUBROADMAP.md`.
- Added `Web3PreSignSafetyReviewRequest`, `Web3PreSignSafetyReviewReport`, and `Web3PreSignSafetyReviewStatus`.
- Added deterministic local review checks for simulation request/response coherence, gas fee cap, minimum output, nonce readiness, lifecycle coherence, and side-effect denial.
- Added append-only audit and SQLite WAL checkpoint helpers for local Web3 pre-sign safety reviews.
- Added Rust tests for ready local review, blocked simulation/nonce/gas/output paths, side-effect denial, audit replay, and SQLite checkpoint reopen.
- Added `arb-agent validate-web3-pre-sign-safety`.
- Wired the CLI into CI after signer authorization envelope validation.

## Explicit Non-Goals

- No live trading.
- No credential loading, signer material loading, plaintext decryption, transaction signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, external submission, or production execution.
- No custody provider, hardware wallet, OS keyring integration, live signer provider, live RPC adapter, live nonce source, or production-readiness approval.
- No claim that local pre-sign safety review is live transaction simulation, custody, signer, RPC, wallet, nonce, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-web3-pre-sign-safety
cargo test -p arb-core web3_pre_sign -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local Web3 pre-sign safety review only. Real RPC simulation, provider-backed nonce retrieval, custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, real transaction construction, broadcasts, deployment signer isolation, and external custody/RPC validation remain unclaimed.
