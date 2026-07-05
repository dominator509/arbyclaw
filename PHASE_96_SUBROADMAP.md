# Phase 96 - DEX/Web3 Live Adapter Boundary Review Gate

## Scope

- Convert the remaining DEX/Web3 live RPC/signing/broadcast adapter gap into a typed local boundary review.
- Preserve the existing prohibition on real RPC calls, HTTP calls, signing, broadcasts, bridges, external submission, credential loading, live execution, and production-readiness claims.
- Wire the review into the connector scenario aggregate gate.

## Implementation

- Added `DexLiveAdapterBoundaryReviewRequest`, `DexLiveAdapterBoundaryReviewReport`, and status typing.
- Added `review_dex_live_adapter_boundary()` with fail-closed side-effect validation and explicit blocker codes.
- Added unit coverage for blocked external-evidence state and side-effect rejection.
- Added `arb-agent validate-dex-live-adapter-boundary`.
- Added `dex_live_adapter_boundary` to `scripts/validate_connector_scenario_gate.py`, raising the connector aggregate gate to 21 local components.

## Validation

Required local validation for this phase:

```text
cargo test -p arb-core dex_live_adapter_boundary -- --nocapture
cargo run -p arb-agent -- validate-dex-live-adapter-boundary
python scripts/validate_connector_scenario_gate.py --json
python scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Remaining Blockers

- Real DEX/RPC adapter implementation remains absent.
- Real provider-backed quote/simulation/nonce evidence remains absent.
- Custody-backed signer evidence remains absent.
- Broadcast permission/control evidence remains absent.
- Production runtime/deployment validation remains absent.

## Completion Note

The DEX/Web3 live-adapter boundary is typed, locally validated, included in the connector aggregate gate, and remains blocked until real testnet/sandbox/live adapter evidence is available.
