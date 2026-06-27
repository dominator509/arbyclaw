# PHASE_48_SUBROADMAP.md

## Phase 48 - Local Web3 Sandbox/Live Discrepancy Calibration

Goal: add deterministic local sandbox/live discrepancy calibration over caller-supplied, non-secret observation references and tolerance metadata without making external calls or claiming external validation.

## Scope

- Model local sandbox/live discrepancy calibration records that depend on broadcast adapter control review.
- Require sanitized sandbox and live observation references.
- Require minimum sandbox/live sample counts and bounded price, latency, and fee discrepancy metadata.
- Deny external calls, credential loading, RPC calls, signer material loading, signing, broadcasting, live execution, and production-readiness claims.
- Persist a ready calibration checkpoint into SQLite WAL state and append the ready report to the append-only audit journal.
- Add a local CLI validation command and CI gate for audit/state recovery.

## Out Of Scope

- Real sandbox or live exchange/RPC calls.
- Credentialed account access, transaction submission, raw transaction serialization, wallet custody, key material, plaintext secrets, signing, broadcasts, or bridges.
- Production readiness, live-funds approval, or external sandbox/live evidence claims.

## Validation

- `cargo run -p arb-agent -- validate-web3-sandbox-live-discrepancy-calibration`
- `cargo test -p arb-core web3_sandbox_live_discrepancy`
- Workspace structure, format, check, test, and clippy gates remain required before claiming the phase locally complete.

## Remaining Work

- Actual external sandbox/live calibration remains blocked until approved credentials-free evidence references, sandbox/testnet execution harnesses, live/RPC adapters, custody-backed signing, deployment-host validation, and operator approvals exist.
- Production runtime validation remains future roadmap work.
