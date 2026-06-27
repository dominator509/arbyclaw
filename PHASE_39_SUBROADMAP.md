# PHASE_39_SUBROADMAP.md

## Phase

Phase 39 - Local Signer Authorization Envelope

## Status

Implemented for local deterministic signer authorization envelope review only.

## Goal

Require a reference-only pre-signing envelope before any future constrained signer can be introduced, proving local policy/destination approval, signer secret-scope review, runtime isolation review, transaction simulation reference, nonce-plan reference, pre-sign audit reference, and pre-sign state checkpoint reference exist without loading signer material, decrypting plaintext, signing, broadcasting, calling RPC, or claiming custody readiness.

## Completed Tasks

- Created `PHASE_39_SUBROADMAP.md`.
- Added `SignerAuthorizationEnvelopeRequest`, `SignerAuthorizationEnvelopeReport`, and `SignerAuthorizationEnvelopeStatus`.
- Added deterministic local envelope validation for signer request readiness, secret-scope readiness, runtime isolation readiness, transaction simulation reference, nonce-plan reference, audit reference, and state checkpoint reference.
- Added side-effect denial fields proving local envelope creation does not load signer material, decrypt plaintext, sign, broadcast, call RPC, or mark production ready.
- Added append-only audit and SQLite WAL checkpoint helpers for local signer authorization envelopes.
- Added Rust tests for ready local envelopes, blocked preconditions, side-effect denial, audit replay, and SQLite checkpoint reopen.
- Added `arb-agent validate-signer-authorization-envelope`.
- Wired the CLI into CI after signer runtime isolation validation.

## Explicit Non-Goals

- No live trading.
- No credential loading, signer material loading, plaintext decryption, transaction signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, external submission, or production execution.
- No custody provider, hardware wallet, OS keyring integration, live signer provider, or production-readiness approval.
- No claim that local signer authorization envelopes are live custody, signer, RPC, wallet, transaction simulation, nonce, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-signer-authorization-envelope
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local signer authorization envelope review only. Real custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, real transaction construction, live nonce handling, external transaction simulation, RPC calls, broadcasts, deployment signer isolation, and external custody validation remain unclaimed.
