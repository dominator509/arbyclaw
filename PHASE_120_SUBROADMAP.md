# Phase 120 - Hardening Core Withdrawal Policy Gate

## Scope

Promote the existing local withdrawal policy boundary validator into the hardening-core aggregate gate so local hardening evidence requires fail-closed withdrawal denial across config, strategy, trust-contract, destination allowlist, signing-boundary, audit, and state controls.

## Implemented Local Work

- Added `validate-withdrawal-policy-boundary --workspace <fresh-dir>` as a required component in `scripts/validate_hardening_core_gate.py`.
- Added aggregate assertions for config guard, strategy flag guard, strategy intent guard, trust-contract guard, destination allowlist guard, signing-boundary guard, audit/state fail-closed behavior, replayed audit record, SQLite checkpoint recovery, and no unsafe external submission, secret recording, or production-readiness flags.

## Explicit Non-Scope

- No withdrawal execution.
- No signer/custody implementation.
- No credential loading or destination ownership validation.
- No external exchange, wallet, RPC, or provider calls.
- No signing, broadcasts, live execution, deployment mutation, or production-readiness claim.

## Remaining Production Blockers

- Per-period withdrawal limit policy.
- Operator confirmation workflow.
- Signer-scoped execution with custody review.
- Destination ownership validation.
- Sandbox/testnet withdrawal evidence under explicit external approval.
