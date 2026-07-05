# Phase 129 - Communications Provider Adapter Readiness Gate

## Objective

Advance the communications production roadmap by wiring a typed local-only provider adapter readiness boundary after provider submission preflight. The boundary must represent the non-secret adapter plan shape needed before future real outbound delivery work while preserving the existing prohibition on provider calls, token loading, message delivery, outbound network use, live execution, signing, broadcasts, and production-readiness claims.

## Scope

- Add core request/report/status types for communications provider adapter readiness.
- Validate that the local submission preflight prerequisite is coherent.
- Require sanitized local-only provider adapter plan controls: request plan, endpoint/auth references, payload template, idempotency, rate-limit budget, retry/backoff, outage circuit breaker, response transcript requirement, and delivery receipt requirement.
- Keep real provider validation evidence explicitly missing and blocked.
- Add `arb-agent validate-communications-provider-adapter-readiness --workspace <fresh-dir>`.
- Require the new CLI in `scripts/validate_operator_surface_gate.py` and the hardening-core aggregate expected count.

## Non-Goals

- No live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, real provider calls, wallet custody, secrets, token loading, message delivery, public network exposure, or production-readiness claims.
- No deployment-host service-manager execution or production communications enablement.

## Validation

- `cargo test -p arb-core communications::tests::communication_provider_adapter_readiness`
- `cargo run -p arb-agent -- validate-communications-provider-adapter-readiness --workspace <fresh-dir>`
- `python3 scripts/validate_operator_surface_gate.py`
- `python3 scripts/validate_hardening_core_gate.py`
- `python3 scripts/validate_structure.py`
- Full workspace format/check/test/clippy gates before claiming the phase is commit-ready.
