# Phase 105 - Web3 Connector Aggregate Coverage Gate

## Scope

- Require existing local Web3 provider nonce reconciliation and sandbox/live discrepancy calibration CLIs in the connector scenario aggregate gate.
- Preserve local-only validation over sanitized nonce snapshots and reference-only discrepancy observations.
- Fail closed if the aggregate sees external calls, RPC calls, credential loading, signer material, signing, broadcasts, live execution, or production-readiness claims.
- Do not add live RPC adapters, provider calls, signing, broadcasts, bridges, credentials, or deployment behavior.

## Implementation

- Added `web3_provider_nonce_reconciliation` to `scripts/validate_connector_scenario_gate.py`.
- Added `web3_sandbox_live_discrepancy_calibration` to `scripts/validate_connector_scenario_gate.py`.
- Added aggregate assertions for nonce readiness, provider snapshot readiness, pending nonce uniqueness, sandbox/live observation references, sample-size readiness, and deviation-limit checks.
- Added generic dangerous-key detection for the existing `rpc-called` and `external-call-performed` CLI fields.
- Raised the connector scenario aggregate to 25 local components.

## Validation

Required local validation for this phase:

```text
cargo run -p arb-agent -- validate-web3-provider-nonce-reconciliation
cargo run -p arb-agent -- validate-web3-sandbox-live-discrepancy-calibration
python scripts/validate_connector_scenario_gate.py --json
python scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Remaining Blockers

- This phase is local-only. Live RPC adapters, real provider-backed nonce retrieval, production nonce/confirmation management, external sandbox/live calibration evidence, custody-backed signing, broadcasts, bridges, deployment-host validation, and production readiness remain incomplete.
