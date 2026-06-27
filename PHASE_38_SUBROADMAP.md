# PHASE_38_SUBROADMAP.md

## Phase

Phase 38 - Local Signer Runtime Isolation Review

## Status

Implemented for local deterministic signer runtime isolation review only.

## Goal

Validate caller-supplied local signer-runtime isolation metadata so signer access remains behind typed policy, destination, secret-scope, audit, and state-checkpoint boundaries without loading signer material, decrypting plaintext, signing, broadcasting, making RPC calls, or claiming live custody readiness.

## Completed Tasks

- Created `PHASE_38_SUBROADMAP.md`.
- Added `SignerRuntimeIsolationReviewRequest`, `SignerRuntimeIsolationReviewReport`, and `SignerRuntimeIsolationReviewStatus`.
- Added deterministic fail-closed local review logic for LLM signer access denial, direct signing-call denial, plaintext key exposure denial, policy-gate requirements, destination allowlist requirements, secret-scope review requirements, audit-before-signing requirements, and state-checkpoint requirements.
- Added side-effect denial fields proving local review does not load signer material, decrypt plaintext, sign, broadcast, call RPC, or mark production ready.
- Added Rust tests for ready local metadata, blocked local metadata, and side-effect denial.
- Added `arb-agent validate-signer-runtime-isolation`.
- Wired the CLI into CI after signer boundary audit validation.

## Explicit Non-Goals

- No live trading.
- No credential loading, signer material loading, plaintext decryption, transaction construction, nonce management, signing, withdrawals, bridges, broadcasts, RPC calls, exchange calls, or external execution.
- No wallet custody backend, hardware wallet integration, OS keyring integration, production key derivation, live signer provider, or production-readiness approval.
- No claim that local signer runtime isolation review is live custody, signer, RPC, wallet, or broadcast validation.

## Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-signer-runtime-isolation
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local signer runtime isolation review only. Real custody-backed signing, runtime key loading, OS keyring or hardware wallet integration, nonce handling, transaction construction, RPC simulation, broadcasts, deployment signer isolation, and external custody validation remain unclaimed.
