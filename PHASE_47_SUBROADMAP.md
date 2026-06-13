# PHASE_47_SUBROADMAP.md

## Phase 47 - Local Web3 Broadcast Adapter Control Review

Goal: add deterministic local broadcast adapter control review without granting broadcast permission, submitting transactions, calling RPC, loading signer material, signing, or claiming production readiness.

## Scope

- Model local broadcast adapter control review records that depend on the raw transaction serialization review.
- Require sanitized adapter, operator approval, and audit/state preflight references.
- Require kill-switch, rate-limit, and replay/idempotency control metadata before a future broadcast adapter can be externally reviewed.
- Deny broadcast permission, raw transaction bytes, raw serialization, RPC calls, signer material loading, signing, broadcasting, live execution, and production-readiness claims.
- Persist a ready review checkpoint into SQLite WAL state and append the ready report to the append-only audit journal.
- Add a local CLI validation command and CI gate for audit/state recovery.

## Out Of Scope

- Real broadcast adapter implementation.
- Transaction submission, raw transaction serialization, wallet custody, key material, plaintext secrets, signing, bridges, real RPC calls, or live exchange calls.
- Production readiness, live-funds approval, or external provider validation claims.

## Validation

- `cargo run -p arb-agent -- validate-web3-broadcast-adapter-control-review`
- `cargo test -p arb-core web3_broadcast_adapter_control`
- Workspace structure, format, check, test, and clippy gates remain required before claiming the phase locally complete.

## Remaining Work

- Real broadcast adapter implementation remains blocked until custody-backed signing, provider-backed nonce validation, transaction simulation, raw serialization, operator approvals, deployment-host audit/state validation, and external sandbox/testnet validation are implemented.
- Sandbox/live discrepancy calibration and production runtime validation remain future roadmap work.
