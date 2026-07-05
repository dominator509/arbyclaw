# Phase 126 - Hardening Core Connector Scenario Aggregate Gate

## Scope

Promote the existing local connector-scenario aggregate validator into the hardening-core aggregate gate so local hardening evidence requires market-data, fee, CEX, DEX/Web3, live-provider boundary, nonce, and sandbox/live discrepancy controls before hardening can pass.

## Implemented Local Work

- Added `scripts/validate_connector_scenario_gate.py --json` as a required `connector_scenario_gate` component in `scripts/validate_hardening_core_gate.py`.
- Added aggregate assertions for the 25-component connector-scenario report, full component pass status, audit replay coverage, required market-data/fee/Web3 review enforcement, no unsafe side-effect flags, no live network use, no credential loading, no WebSocket opening, no live provider calls, no external submission, no RPC calls, no signing or broadcast, no live execution, and no production-readiness flag.

## Explicit Non-Scope

- No live REST/WebSocket exchange adapters.
- No live provider calls, credentials, RPC calls, signing, broadcasts, external submission, live execution, or production-readiness claim.

## Remaining Production Blockers

- Live REST/WebSocket exchange adapters.
- Provider-backed market-data session, latency, rate-limit/outage, and bad-data validation.
- Provider-backed fee, account-tier, gas/RPC, and withdrawal-cost validation.
- External exchange sandbox/live order lifecycle calibration.
- Live DEX/RPC simulation and router validation without broadcasts.
- Production deployment-host connector validation.
