# Architecture

ArbyClaw is a local-first Rust arbitrage research and validation system with explicit safety boundaries around external execution. This document describes stable architecture, not chronological implementation history.

## Workspace

```text
Cargo.toml
├── crates/arb-core   # domain logic, policy, persistence, local models
└── crates/arb-agent  # CLI/runtime entrypoint and validation runners
```

Python under `scripts/` orchestrates CI/local validation. It is not the business-logic runtime.

## Core domains

`arb-core` currently exposes the following domain modules:

- `config` — typed configuration, schema migration, runtime mode gates.
- `secrets` — secret references, local secret lifecycle reviews, redaction-safe material wrappers.
- `state` — in-memory and SQLite WAL checkpoint/state storage.
- `policy` — deny-by-default intent evaluation and trust-contract restrictions.
- `audit` — append-only audit journal, replay, durability/retention models and transcript validation.
- `destination` — destination allowlist and ownership-evidence-reference controls.
- `market_data` — normalized quotes/order books, freshness, provider boundaries and local validation.
- `fees` — fee schedules, estimates, reconciliation and provider-boundary reviews.
- `paper` — deterministic paper providers, fills, balance ledgering, replay/backtest support.
- `cex` — CEX profiles, capabilities, request/transcript models, deterministic local adapter and live-boundary reviews.
- `dex` — DEX/Web3 profiles, request/simulation/receipt/nonce models, deterministic local adapter and non-broadcast reviews.
- `opportunity` — deterministic opportunity discovery, ranking, replay and false-positive review.
- `strategy` — strategy/risk/venue parameters and policy-constraint checks.
- `planner` — draft-only execution plans and per-intent policy preflight.
- `execution_adapter` — local execution-attempt/recovery boundary; external submission remains blocked.
- `runtime` — local lifecycle sequencing, recovery, graceful shutdown and preflight validation.
- `communications` — local command routing, notification/outbox and provider readiness/preflight models.
- `dashboard` — local render/auth/session/loopback and persistent-host readiness models.
- `observability` — health/log/metric/runbook, loopback metrics and provider readiness/preflight models.
- `testing` — validation-plan, corpus, property/fuzz/backtest metadata and local execution support.
- `packaging` — release/package/deployment plan and evidence models.
- `hardening` — external evidence/checklist boundaries.
- `handoff` — bounded agent/human handoff records.

## Runtime and data flow

The intended local flow is:

```text
configuration
   ↓
market/fee inputs (local providers or fixtures)
   ↓
opportunity discovery/ranking
   ↓
strategy constraints
   ↓
draft execution planner
   ↓
policy + destination + signer/non-broadcast preflight
   ↓
local execution-adapter evaluation
   ↓
audit journal + SQLite WAL checkpoints
```

A successful local path does not imply external submission. Live external adapters are separate capabilities and remain blocked unless explicitly implemented and approved.

## Trust boundaries

### Policy boundary

Funds-moving or live-scope intents must fail closed unless all applicable configuration, policy, destination, signer, and execution prerequisites are satisfied. No LLM output may bypass policy.

### Secret boundary

Repository configuration stores references, not production secret values. Secret-like material must be redacted from logs, audit records, dashboard/communications output, crash paths, prompts and generated evidence.

### Signing boundary

The current signer subsystem is a fail-closed local boundary. Production custody-backed signing is not implemented. Signing material must never be directly exposed to an LLM, generic command router, dashboard control, or validation transcript.

### External-call boundary

CEX, DEX/RPC, communications and observability provider modules contain local models and readiness/preflight logic. These models are not proof that real providers were contacted.

### Production-approval boundary

Software can report evidence state; it cannot grant production or live-funds approval. `CAPABILITIES.md` defines the allowed state vocabulary.

## Persistence

SQLite WAL is the current persistent state implementation. Audit data uses append-only journal primitives. Crash/restart and checkpoint recovery are first-class validation concerns.

The current SQLite store is not an encrypted SQLCipher database. Any future at-rest encryption claim must correspond to an actual implementation and test evidence.

## Validation architecture

Validation has four layers:

1. **Rust unit/integration tests** — semantic behavior and failure boundaries.
2. **Leaf CLI validators** — execute concrete local runtime/domain scenarios and emit structured output.
3. **Aggregate gates** — combine distinct leaf results by domain.
4. **CI** — clean-checkout build/test/artifact/security execution.

The top handoff gate must not directly rerun aggregate suites already owned by `hardening-core`. A wrapper is justified only when it adds a distinct assertion, environment, or artifact.

`validate_repository_hygiene.py` checks the tracked Git tree for generated/cache/mock evidence that must not become source of truth. `validate_structure.py` checks current required files, anti-drift invariants, workspace membership, AI context completeness and secret-pattern hygiene. No generated structure hash manifest is canonical.

## Known structural debt

The current design has accumulated oversized Rust files, especially `arb-agent/src/main.rs` and several core domain modules. The remediation roadmap calls for mechanical decomposition into internal modules while preserving command/API behavior.

The crate root also re-exports a very broad symbol surface. Future refactoring should prefer domain-qualified imports so ownership is obvious.

These are maintainability defects, not justification for a ground-up rewrite.

## Non-capabilities

Unless `CAPABILITIES.md` explicitly states otherwise, the architecture does not currently provide:

- real live exchange REST/WebSocket connectivity;
- real DEX/RPC connectivity;
- production custody-backed signing;
- transaction broadcasts;
- withdrawals or bridges;
- production persistent dashboard hosting;
- real outbound messaging delivery;
- production telemetry export/log shipping/alert delivery;
- production service installation/deployment;
- live-funds approval.

## Source-of-truth order

For implementation questions, use this order:

1. source code and executable tests;
2. `CAPABILITIES.md`;
3. this architecture document and `docs/ai/ARCHITECTURE_MAP.md`;
4. `docs/ai/API_CONTRACTS.md`;
5. `PRODUCTION_GAP_TRACKER.md` and `ROADMAP.md`;
6. handoff/tool memories as non-canonical navigation aids.

When documents and executable behavior disagree, treat the discrepancy as drift and fix the documentation or implementation before making a stronger capability claim.
