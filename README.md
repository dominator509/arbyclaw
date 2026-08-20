# ArbyClaw

ArbyClaw is a local-first Rust arbitrage research, simulation, policy, audit, and validation system. The current repository is intentionally fail-closed: it can model and validate paper/local execution paths, but it does **not** provide production live-trading infrastructure.

## What exists today

- Rust workspace with `arb-core` and `arb-agent`.
- Typed configuration and deny-by-default policy enforcement.
- Secret-reference boundaries and redaction checks.
- Append-only audit primitives and SQLite WAL state/checkpoint recovery.
- Deterministic paper market data, fee, execution, balance, replay, and backtest paths.
- Local CEX and DEX/Web3 framework models, fixtures, transcript parsing, and safety reviews.
- Opportunity discovery/ranking, draft-only planning, and local execution-adapter boundaries.
- Local-only communications, dashboard, and observability validation surfaces.
- Release-artifact, container/deployment, hardening, and handoff validation scripts.
- Crash/restart and fail-closed regression coverage.

## What does not exist yet

The following are not production capabilities of this repository and must not be inferred from local readiness models, transcripts, or validators:

- live exchange REST/WebSocket adapters;
- live DEX/RPC providers;
- wallet custody or production signer implementation;
- transaction signing or broadcast;
- withdrawals or bridges;
- production persistent dashboard hosting;
- real outbound communications delivery;
- production observability exporters/log shipping/alert delivery;
- production service deployment or live-funds approval.

See [`CAPABILITIES.md`](CAPABILITIES.md) for the canonical capability-state matrix and [`PRODUCTION_GAP_TRACKER.md`](PRODUCTION_GAP_TRACKER.md) for closure conditions.

## Build and validate

```bash
python3 scripts/validate_repository_hygiene.py
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace --locked
python3 scripts/validate_test_collection.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/validate_release_artifact.py
python3 scripts/validate_agentic_handoff_candidate_gate.py --json
```

`validate_release_artifact.py` performs the locked release build and smoke-runs the copied production artifact. The top handoff gate composes the current local hardening surface without granting production approval.

## Evidence semantics

ArbyClaw does not use numeric production-readiness percentages. A capability may be modeled, locally implemented, locally integrated, externally validated, or production approved. Those states are not interchangeable.

A test, transcript, review record, mock, fixture, or local preflight is evidence only for the thing it actually executes. Simulated evidence must never be represented as a completed external audit or production validation.

## Repository map

- `crates/arb-core/` — domain primitives, policy, persistence, local models, and validation-supporting logic.
- `crates/arb-agent/` — CLI/runtime entrypoint and validation runners.
- `scripts/` — CI/local validation orchestration.
- `hardening/` — concise external validation and production-readiness checklists.
- `deployment/` — packaging/deployment examples and validation assets.
- `handoff/` — bounded future-agent/human continuation guidance.
- `docs/ai/` — compact architecture and API context for coding agents.
- `docs/history/` — historical governance summary; detailed phase history remains in Git.

## Safety invariant

No AI agent, local validator, or model output is authorized to bypass policy, load production credentials, sign transactions, broadcast transactions, move funds, expose a public service, or declare production readiness without explicit human authorization plus applicable external evidence.
