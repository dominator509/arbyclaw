# PHASE_46_SUBROADMAP.md

## Phase 46 - Local Web3 Raw Transaction Serialization Review

Goal: add deterministic local raw transaction serialization readiness review without serializing raw transaction bytes, embedding raw calldata, loading signer material, signing, broadcasting, calling RPC, or claiming production readiness.

## Scope

- Model local raw transaction serialization review records that depend on provider nonce reconciliation.
- Require sanitized transaction type, chain id, fee field, and access-list/account-meta references.
- Deny raw transaction bytes, raw calldata, raw transaction serialization, broadcast permission, RPC calls, signer material loading, signing, broadcasting, live execution, and production-readiness claims.
- Persist a ready review checkpoint into SQLite WAL state and append the ready report to the append-only audit journal.
- Add a local CLI validation command and CI gate for audit/state recovery.

## Out Of Scope

- Real raw transaction serialization.
- Raw calldata construction, wallet custody, key material, plaintext secrets, signing, broadcasts, bridges, real RPC calls, or live exchange calls.
- Production readiness, live-funds approval, or external provider validation claims.

## Validation

- `cargo run -p arb-agent -- validate-web3-raw-transaction-serialization-review`
- `cargo test -p arb-core web3_raw_transaction_serialization`
- Workspace structure, format, check, test, and clippy gates remain required before claiming the phase locally complete.

## Remaining Work

- Real serializer implementation remains blocked until custody-backed signing boundaries, provider-backed nonce validation, transaction simulation, operator approvals, broadcast controls, and deployment-host validation are implemented.
- Custody-backed signer implementation, broadcast adapters, sandbox/live discrepancy calibration, and production runtime validation remain future roadmap work.
