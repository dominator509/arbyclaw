# ArbyClaw Safety

## Forbidden Without Explicit Future Authorization

- Live trading.
- Transaction signing.
- Withdrawals.
- Bridges.
- Broadcasts.
- Real RPC calls.
- Real exchange calls.
- Wallet custody.
- Secrets or secret material.
- Real outbound production communications, hosted dashboard exposure, telemetry export, or alert delivery.

## Accepted Direction

- Local-only typed boundaries.
- Deterministic fixtures and replay.
- Fail-closed validation.
- Append-only audit and SQLite WAL state recovery.
- Reference-only evidence records when needed, never embedded secrets or artifact contents.

## Review Bias

- Preserve production blockers until real authoritative validation exists.
- Do not claim production readiness from local gates.
