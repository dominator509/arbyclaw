# PHASE_45_SUBROADMAP.md

## Phase 45 - Local Web3 Provider Nonce Reconciliation

Goal: add deterministic local provider nonce reconciliation for caller-supplied nonce snapshot metadata without performing RPC calls, signing, broadcasting, custody access, or live execution.

## Scope

- Model local provider nonce reconciliation records that compare unsigned transaction construction nonce metadata against a caller-supplied provider next nonce and pending nonce set.
- Require sanitized provider snapshot references, provider next nonce presence, pending nonce uniqueness, construction nonce match, construction nonce absence from pending nonces, and bounded snapshot age.
- Persist a ready reconciliation checkpoint into SQLite WAL state and append the ready report to the append-only audit journal.
- Add a local CLI validation command and CI gate for replaying audit/state recovery.

## Out Of Scope

- Real RPC/provider nonce retrieval.
- Wallet custody, key material, plaintext secrets, signing, raw transaction serialization, broadcasts, bridges, or live exchange calls.
- Production readiness, live-funds approval, or external provider validation claims.

## Validation

- `cargo run -p arb-agent -- validate-web3-provider-nonce-reconciliation`
- `cargo test -p arb-core web3_provider_nonce`
- Workspace structure, format, check, test, and clippy gates remain required before claiming the phase locally complete.

## Remaining Work

- Provider-backed nonce retrieval and live provider validation remain blocked until real RPC/provider adapters, custody separation, operator approvals, and deployment-host validation are implemented.
- Custody-backed signer implementation, raw transaction serialization review, broadcast adapters, sandbox/live discrepancy calibration, and production runtime validation remain future roadmap work.
