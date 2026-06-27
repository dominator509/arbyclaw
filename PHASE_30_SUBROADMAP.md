# PHASE_30_SUBROADMAP.md

## Phase

Phase 30 - Connector Scenario Aggregate Gate

## Status

Implemented for local deterministic aggregate connector scenario validation only.

## Goal

Compose the existing local market-data, fee, and CEX/DEX connector lifecycle CLI probes into one stronger gate that verifies current connector boundary fixtures remain deterministic, fail closed, and free of live/external side effects.

## Completed Tasks

- Created `PHASE_30_SUBROADMAP.md`.
- Added `scripts/validate_connector_scenario_gate.py`.
- The aggregate gate now runs sixteen existing connector-adjacent CLIs covering market-data provider preflight, reconnect planning, market-data quality assessment, paid-provider evaluation, deterministic historical persistence, market-data audit/state recovery, fee verification, fee audit/state recovery, CEX governance and request/balance transcript validation, DEX request/response/transaction/protocol-risk validation, and CEX/DEX connector lifecycle audit/state recovery.
- The gate fails closed if any nested command reports live network use, WebSocket connection opening, credential loading, live provider calls, external submission, RPC calls, signing, broadcasts, live execution, or production readiness.
- Added the aggregate gate to CI after the existing connector lifecycle audit CLI.
- Updated structure validation for Phase 30 files.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, RPC, market-data provider, fee-provider, or DEX/router calls.
- No credential loading, wallet custody, signing, withdrawals, bridges, broadcasts, or external adapter submission.
- No production deployment or production-readiness approval.
- No claim that local deterministic connector fixtures are external sandbox/live evidence.

## Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met for local aggregate connector scenario validation only. Real REST/WebSocket exchange adapters, provider-backed market-data and fee validation, external exchange sandbox/live lifecycle calibration, live DEX/RPC simulation and router validation, custody/signing validation, production deployment-host connector validation, and production readiness remain unclaimed.
