# ROADMAP.md

## Project

Fully Autonomous Crypto Arbitrage Agent

## Current Roadmap Position

- Active phase: Phase 18 — Agentic Handoff Package implemented; next required work is external Rust validation and production hardening evidence generation
- Active sub-roadmap: `PHASE_18_SUBROADMAP.md`
- Current production readiness: 87%
- Current implementation status: Minimal Rust workspace, typed config, reference-only secret boundary, mode-gate validation, deny-by-default policy engine, append-only audit journal primitives, state-store trait boundary, normalized market-data models, fee models, freshness classification, provider trait boundaries, deterministic paper connectors, CEX connector framework types/traits, DEX/Web3 connector framework types/traits, deterministic opportunity-engine types/traits, draft-only execution-planner types/traits, execution-adapter boundary records/traits, communications/CLI command and notification boundaries, embedded-dashboard local render boundaries, observability/runbook local record boundaries, deterministic testing/fuzzing/backtesting plan boundaries, deterministic packaging/deployment plan boundaries, deterministic external-hardening evidence/checklist boundaries, and deterministic agentic handoff package boundaries exist; live trading, real build/container/systemd/ARM deployment validation, real validation runner execution, real fuzzing engines, real backtest corpus execution, real observability runtime, real dashboard hosting, outbound messaging integrations, external adapter submission, exchange-specific live connectors, DEX RPC adapters, wallet signer, transaction broadcasts, custody backend, durable SQLite state, and production execution logic are not implemented.
- Current risk posture: High, because live-funds architecture is policy-, audit-, market-data-, paper-simulation-, CEX-framework-, DEX/Web3-framework-, opportunity-engine-, planner-, adapter-, communication-, dashboard-, observability-, and validation-boundary-gated but lacks custody isolation, exchange-specific live connectors, live DEX/RPC adapters, signer boundary, live adapter submission, real communications adapters, real dashboard hosting/authentication, real metrics/exporter/alert runtime, actual Rust/property/fuzz/backtest execution, real package/deployment validation, executed external hardening evidence, future-agent execution validation, durable database validation, and external validation.

## Roadmap Governance Rules

1. Every task begins with reconciliation against governance files.
2. Every phase must have a `PHASE_X_SUBROADMAP.md` before implementation begins.
3. Every phase completion must update `ROADMAP.md`, active `PHASE_X_SUBROADMAP.md`, and `PRODUCTION_GAP_TRACKER.md`.
4. Small, reversible patches are mandatory.
5. Live trading functionality must remain gated behind policy, tests, simulation, audit, and explicit mode controls.
6. No production validation may be claimed unless actually performed.
7. Secrets must never be committed or stored in Markdown.
8. The LLM layer must never directly sign transactions or bypass policy.

## Phase Summary

| Phase | Name | Status | Production Readiness Contribution | Notes |
|---|---|---:|---:|---|
| 0 | Governance Initialization | Complete | 2% | Mandatory governance files established. |
| 1 | Rust Workspace Scaffold | Scaffold complete; Rust validation deferred | +2% realized / +1% deferred | Minimal workspace, CI skeleton, safety docs, and structure validator created. Cargo validation requires Rust toolchain. |
| 2 | Config, Secrets, and Mode Gates | Implemented; Rust validation deferred | +5% realized / +1% deferred | Typed config, environment secret references, keystore interface boundary, and live mode-gate validation added. |
| 3 | Policy Engine and Trust Contract | Implemented; Rust validation deferred | +7% realized / +3% deferred | Deny-by-default policy checks, intent model, trust-contract denials, and CLI policy initialization added. |
| 4 | Audit Journal and State Store | Implemented; Rust validation deferred; SQLite WAL deferred | +4% realized / +2% deferred | Append-only hash-chained JSONL audit primitives, redaction checks, and state-store trait added; SQLite WAL persistence remains future work. |
| 5 | Market Data Core | Implemented; Rust validation and live provider validation deferred | +5% realized / +2% deferred | Normalized quotes/order books, freshness windows, fee models, and provider traits added; no live network providers. |
| 6 | Simulated/Paper Connectors | Implemented; Rust validation and paper-model limitations deferred | +6% realized / +1% deferred | Deterministic in-memory paper market data, static fee provider, and policy-gated paper execution adapter added; no live venues or balances. |
| 7 | CEX Connector Framework | Implemented as framework boundary; Rust validation and live exchange validation deferred | +6% realized / +2% deferred | CEX venue profiles, capability registry, order request model, policy gate, and connector traits added; no exchange-specific adapters or live orders. |
| 8 | DEX/Web3 Connector Framework | Implemented as framework boundary; Rust validation, live RPC, signing, and broadcast validation deferred | +8% realized / +0% deferred in ChatGPT | Chain/router/token profiles, router capabilities, swap quote models, local transaction simulation boundary, policy gate, and connector traits added; no live RPC, signing, bridges, or broadcasts. |
| 9 | Opportunity Engine | Implemented as deterministic discovery/ranking boundary; Rust validation and advanced route validation deferred | +8% realized / +0% deferred in ChatGPT | Cross-venue top-of-book discovery, CEX/CEX, DEX/DEX, CEX/DEX, triangular model boundary, freshness checks, and fee-aware scoring added; no execution intents or order placement. |
| 10 | Execution Planner | Implemented as draft-only model boundary; Rust validation and adapter integration deferred | +7% realized / +0% deferred in ChatGPT | Deterministic plan drafts, per-leg intent generation, policy preflight outcomes, sequencing, and failure-mode boundaries added; no adapter submission or live execution. |
| 11 | Execution Adapters | Implemented as deterministic boundary framework; Rust validation and live submission deferred | +7% realized / +0% deferred in ChatGPT | Consumes planner drafts, revalidates policy, models attempts/fills/reconciliation, and blocks all external submission. |
| 12 | Communications and CLI | Implemented as deterministic model/trait boundary; Rust validation and real outbound integrations deferred | +6% realized / +0% deferred in ChatGPT | Typed local command parsing/routing, notification models, redaction checks, and local dispatch records added; no platform tokens or outbound network delivery. |
| 13 | Embedded Dashboard | Implemented as deterministic model/trait boundary; Rust validation and real hosting deferred | +3% realized / +0% deferred in ChatGPT | Local snapshot/panel/render records, fail-closed server binding, secret redaction, and live-control denial added; no web server or public exposure. |
| 14 | Observability and Runbooks | Implemented as deterministic model/trait boundary; Rust validation and real observability runtime deferred | +5% realized / +0% deferred in ChatGPT | Local health, structured-log, metric, and runbook records added; metrics endpoints and outbound alerts denied. |
| 15 | Testing, Fuzzing, and Backtesting | Implemented as deterministic model/trait boundary; Rust validation and real fuzz/backtest execution deferred | +4% realized / +2% deferred | Validation harness config, test case metadata, fixture records, fuzz corpus definitions, backtest scenario definitions, and local plan records added; no external fuzzer invocation or live network tests. |
| 16 | Packaging and Deployment | Implemented as deterministic model/docs boundary; Rust/container/systemd/ARM validation deferred | +2% realized / +0% deferred in ChatGPT | Package/deployment plan records, release gates, rollback steps, Docker/systemd/ARM docs; no build/install/deploy claim. |
| 17 | External Production Hardening | Implemented as deterministic evidence/checklist boundary; real external validation deferred | +0% in ChatGPT | Evidence records, release blockers, and hardening checklists added; no pen test, cloud deployment, live exchange validation, or load test executed. |
| 18 | Agentic Handoff Package | Implemented as deterministic model/docs boundary; external agent execution not performed | +0% direct | Codex/Cursor/Jules/Claude/human handoff package records, prompts, and checklists added; no external agents executed. |

Potential total inside ChatGPT Project Mode: approximately 75–87% of code/documentation readiness, but not full production readiness because live infrastructure, external exchange credentials, real deployment, penetration testing, runtime validation, and live trading verification are environment-limited.

## Phase 0 — Governance Initialization

### Status

Complete.

### Completed Tasks

- Created `ARCHITECTURE.md`
- Created `ROADMAP.md`
- Created `AGENTS.md`
- Created `PHASE_0_SUBROADMAP.md`
- Created `PRODUCTION_GAP_TRACKER.md`
- Established Rust-first, lightweight single-binary architecture direction
- Established strict no-secrets-in-Markdown rule
- Established trust contract and policy-first live-funds boundary
- Established flexible but deterministic roadmap continuation model

### Deferred Tasks

- No code scaffold yet
- No runtime validations
- No exchange integrations
- No DEX integrations
- No secret manager implementation
- No production environment

### Exit Criteria

Met for governance initialization only.

## Phase 1 — Rust Workspace Scaffold

### Status

Scaffold complete; Rust toolchain validation deferred because Rust/Cargo is unavailable in the ChatGPT execution environment.

### Goal

Create the smallest safe Rust repository scaffold that future phases can extend without broad rewrites.

### Completed Deliverables

- `PHASE_1_SUBROADMAP.md`
- Root `Cargo.toml` workspace
- `crates/arb-core` library crate
- `crates/arb-agent` binary crate
- `rust-toolchain.toml`
- `rustfmt.toml`
- `.github/workflows/ci.yml`
- `.gitignore`
- `.env.example` with no secrets
- `README.md`
- `SECURITY.md`
- `scripts/validate_structure.py`

### Dependencies

- Phase 0 complete
- `PHASE_1_SUBROADMAP.md` created before Phase 1 implementation

### Validation Completed

- Governance files reread and reconciled.
- Required Phase 1 files created.
- Workspace members statically verified by `scripts/validate_structure.py`.
- Secret-assignment static scan passed using `scripts/validate_structure.py`.

### Validation Deferred

The following commands were not run because Rust/Cargo is unavailable in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Known Limitations

- No compiled binary has been produced in ChatGPT Project Mode.
- Hosted CI has not run.
- No runtime execution validation was performed.
- No trading behavior exists.

### Exit Criteria

Met for scaffold creation. Full Phase 1 build validation remains tracked as environment-limited work in `PRODUCTION_GAP_TRACKER.md`.

## Phase 2 — Config, Secrets, and Mode Gates

### Status

Implemented in ChatGPT Project Mode; Rust toolchain validation deferred because Rust/Cargo is unavailable in the execution environment.

### Goal

Add typed non-secret configuration, secret-reference boundaries, and deterministic mode gates without implementing live trading, wallet signing, or exchange connectors.

### Completed Deliverables

- `PHASE_2_SUBROADMAP.md`
- `crates/arb-core/src/config.rs`
- `crates/arb-core/src/secrets.rs`
- Updated `crates/arb-core/src/lib.rs`
- Updated `crates/arb-core/Cargo.toml` with `serde` and `toml` dependencies
- Updated `crates/arb-agent/src/main.rs` with `--config <path>` loading
- `config.example.toml` with no secrets
- Updated `.env.example` with empty reference names only
- Updated `scripts/validate_structure.py`

### Validation Completed

- Governance files reread and reconciled.
- Required Phase 2 files created.
- Structure validator passed with `python3 scripts/validate_structure.py`.
- Secret-assignment static scan passed using `scripts/validate_structure.py`.

### Validation Deferred

The following commands require a Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode scaffold/config implementation only. External Rust validation remains required before merge to a protected branch or any downstream phase relying on compiled behavior.

## Phase 3 — Policy Engine and Trust Contract

### Status

Implemented in ChatGPT Project Mode; Rust toolchain validation deferred because Rust/Cargo is unavailable in the execution environment.

### Goal

Implement deterministic deny-by-default policy enforcement before any live execution features exist.

### Completed Deliverables

- `PHASE_3_SUBROADMAP.md`
- `crates/arb-core/src/policy.rs`
- Policy exports from `crates/arb-core/src/lib.rs`
- CLI policy initialization message after config load
- Updated `scripts/validate_structure.py`
- Policy unit tests drafted for approval and denial paths

### Policy Coverage Implemented

- Runtime mode checks
- Observe/paper/live scope checks
- Live runtime unavailable by default
- Kill-switch denial
- Withdrawal denial
- Bridge-route denial
- Audit-redaction requirement for executable intents
- Venue allowlist check
- Asset allowlist check
- Chain allowlist check
- Risk-limit checks
- Profit-after-fee check
- Market-data freshness check
- Unknown-destination denial
- LLM-generated-destination denial
- Signing secret-reference check

### Validation Completed

- Governance files reread and reconciled.
- Required Phase 3 files created.
- Structure validator passed with `python3 scripts/validate_structure.py`.
- Secret-assignment static scan passed using `scripts/validate_structure.py`.

### Validation Deferred

The following commands require a Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode policy implementation only. External Rust validation, property tests, audit integration, and execution-adapter integration remain required.

## Phase 4 — Audit Journal and State Store

### Status

Implemented in ChatGPT Project Mode; Rust toolchain validation, SQLite WAL persistence, crash testing, concurrent append testing, and filesystem permission validation deferred.

### Goal

Create durable, efficient, redacted, append-only audit and state tracking.

### Completed Deliverables

- `PHASE_4_SUBROADMAP.md`
- `crates/arb-core/src/audit.rs`
- `crates/arb-core/src/state.rs`
- Audit and state exports from `crates/arb-core/src/lib.rs`
- Updated `crates/arb-core/Cargo.toml` with `serde_json` and `sha2` dependencies
- CLI status messaging updated to reflect audit/state boundary availability
- Updated `scripts/validate_structure.py`
- Audit unit tests drafted for append/reopen and redaction rejection
- State unit tests drafted for checkpoint round-trip and secret-like rejection

### Audit Coverage Implemented

- Typed audit event categories
- Typed audit metadata values
- Explicit redaction marker
- Secret-like metadata rejection
- Append-only JSONL record append
- Hash-chained records with genesis marker
- Replay-time sequence validation
- Replay-time previous-hash validation
- Replay-time format-version validation
- Replay-time event redaction validation
- Replay-time record-hash validation

### State Boundary Implemented

- `StateCheckpoint` model
- `StateStore` trait
- `InMemoryStateStore` for tests and early local wiring only
- Secret-like checkpoint content rejection

### Validation Completed

- Governance files reread and reconciled.
- Required Phase 4 files created.
- Structure validator passed with `python3 scripts/validate_structure.py`.
- Secret-assignment static scan passed using `scripts/validate_structure.py`.

### Validation Deferred

The following commands require a Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Additional Phase 4 validations deferred:

- audit append/reopen test execution
- audit tamper-detection test execution
- audit redaction test execution
- crash/recovery testing
- concurrent append testing
- filesystem permission testing
- durable SQLite WAL implementation and migration validation

### Exit Criteria

Met for ChatGPT Project Mode audit/state boundary implementation only. External Rust validation and durable database implementation remain required before any live execution path may depend on audit persistence.

## Phase 5 — Market Data Core

### Status

Implemented in ChatGPT Project Mode as model and trait boundaries; Rust/Cargo validation and live provider validation are deferred.

### Goal

Implement normalized quote, order book, fee, and freshness models.

### Completed Tasks

- Created `PHASE_5_SUBROADMAP.md`.
- Added `arb-core::market_data` with normalized market pairs, price levels, top-of-book quotes, order-book snapshots, freshness classification, market-data requests, provider capabilities, and `MarketDataProvider` trait.
- Added `arb-core::fees` with liquidity roles, fee schedules, fee estimates, fee-adjusted edge calculation, and `FeeProvider` trait.
- Exported market-data and fee primitives through `arb-core`.
- Updated `arb-agent` status output to report market-data boundary availability without starting network providers.
- Updated structure validation to require Phase 5 files.

### Deferred Tasks

- Cargo format/check/test/clippy validation.
- Live REST/WebSocket provider implementations.
- Paid data-provider integration.
- Exchange-specific fee schedule validation.
- Market-data latency, reconnect, rate-limit, and quality validation.
- Opportunity-engine consumption of market-data models.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Deferred until Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode market-data/fee-model boundary implementation only. External Rust validation and live provider validation remain required before market data may be used for production decisions.

## Phase 6 — Simulated/Paper Connectors

### Status

Implemented in ChatGPT Project Mode as deterministic in-memory paper connector boundaries; Rust/Cargo validation, paper-balance ledgering, audit integration, and realistic fill simulation are deferred.

### Goal

Enable deterministic strategy testing without live funds.

### Completed Tasks

- Created `PHASE_6_SUBROADMAP.md`.
- Added `arb-core::paper` with `PaperMarketDataProvider`, `PaperFeeProvider`, and `PaperExecutionAdapter`.
- Implemented in-memory paper order-book lookup through the existing `MarketDataProvider` trait.
- Implemented static paper fee lookup through the existing `FeeProvider` trait.
- Implemented policy-gated paper execution reports for paper-scoped intents only.
- Exported paper connector primitives through `arb-core`.
- Updated `arb-agent` status output to report paper connector boundary availability.
- Updated structure validation to require Phase 6 files.

### Deferred Tasks

- Cargo format/check/test/clippy validation.
- Paper balance ledger and position tracking.
- Realistic order-book depth, partial-fill, latency, and slippage simulation.
- Audit journal integration for paper execution events.
- Scenario fixture library and backtesting harness.
- Live CEX/DEX connector implementation.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Deferred until Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode deterministic paper connector boundary implementation only. External Rust validation and richer paper-simulation validation remain required before Phase 6 can be treated as tested runtime behavior.

## Phase 7 — CEX Connector Framework

### Status

Implemented as typed framework boundary; Rust validation and live exchange validation deferred.

### Goal

Define exchange connector traits, venue capability profiles, order request models, and policy-gated CEX validation without adding exchange-specific live adapters.

### Completed Tasks

- Created `PHASE_7_SUBROADMAP.md`.
- Added `arb-core::cex` with CEX venue profiles, connector capabilities, registry, order request models, order side/type/time-in-force enums, and order status boundary.
- Added `CexPolicyGate` to validate paper/sandbox CEX order requests through `PolicyEngine` while blocking live CEX orders in Phase 7.
- Added `CexConnectorIdentity`, `CexReadOnlyConnector`, and `CexTradingConnector` trait boundaries for future adapters.
- Exported CEX framework primitives through `arb-core`.
- Updated `arb-agent` status output to report CEX framework availability.
- Updated structure validation to require Phase 7 files.

### Deferred Tasks

- Cargo format/check/test/clippy validation.
- Exchange-specific REST and WebSocket adapters.
- Authenticated balance reads.
- Live order submission and cancellation.
- Sandbox exchange integration tests.
- Rate-limit and fee-schedule verification.
- Jurisdiction and exchange terms-of-service review.
- Audit/state integration for future CEX order lifecycle events.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Deferred until Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode CEX framework boundary implementation only. External Rust validation, exchange-specific adapter work, sandbox testing, credential-scope checks, rate-limit validation, and terms/jurisdiction review remain required before any live CEX use.

## Phase 8 — DEX/Web3 Connector Framework

### Status

Implemented as typed framework boundary; Rust validation, live RPC validation, signer validation, transaction simulation integration, and broadcast validation deferred.

### Goal

Implement chain, router, token, swap quote, and transaction-simulation boundaries before any signing.

### Completed Tasks

- Created `PHASE_8_SUBROADMAP.md`.
- Added `arb-core::dex` with Web3 chain profiles, token profiles, DEX/router profiles, router capabilities, connector registry, swap quote request/response models, transaction simulation request/response models, simulation status enum, and connector trait boundaries.
- Added `DexPolicyGate` to validate paper/simulation-scoped DEX swap quote requests through `PolicyEngine` while blocking live DEX swaps in Phase 8.
- Added local transaction simulation request validation that performs no RPC, signing, or broadcast and returns non-broadcastable local validation responses only.
- Exported DEX/Web3 framework primitives through `arb-core`.
- Updated `arb-agent` status output to report DEX/Web3 framework availability.
- Updated structure validation to require Phase 8 files.

### Deferred Tasks

- Cargo format/check/test/clippy validation.
- Real chain RPC adapters.
- Router/aggregator quote adapters.
- Testnet/mainnet transaction simulation integrations.
- Wallet signer boundary and custody implementation.
- Transaction construction, signing, and broadcast adapters.
- Spender approval hygiene, allowance management, and nonce management.
- Bridge and cross-chain route support after elevated risk review.
- Protocol, token, gas, slippage, MEV, terms, and jurisdiction review.
- Audit/state integration for future on-chain lifecycle events.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Deferred until Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode DEX/Web3 framework boundary implementation only. External Rust validation, RPC/simulation adapters, signer/custody work, transaction broadcast controls, protocol review, and live/on-chain validation remain required before any DEX/Web3 use with real funds.

## Phase 9 — Opportunity Engine

### Status

Implemented as deterministic discovery/ranking boundary; Rust toolchain validation deferred because Rust/Cargo is unavailable in the ChatGPT execution environment.

### Goal

Implement fee-aware arbitrage discovery and ranking.

### Completed Tasks

- Created `PHASE_9_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/opportunity.rs`.
- Added `OpportunityDiscoveryConfig`, `OpportunityDiscoveryRequest`, `OpportunityCandidate`, `OpportunityLeg`, `OpportunityScore`, `OpportunityRouteKind`, and validation errors.
- Added `DeterministicOpportunityEngine` for supplied normalized quotes and fee schedules only.
- Added cross-venue top-of-book discovery across CEX/CEX, DEX/DEX, and CEX/DEX venue-kind boundaries.
- Added triangular route-kind model boundary while full triangular path search remains deferred.
- Added fail-closed market-data freshness checks.
- Added fee-aware edge calculation and deterministic ranking.
- Exported opportunity-engine types from `arb-core`.
- Updated CLI status output and structure validation.

### Explicit Non-Goals

- No execution intents.
- No order placement.
- No signing.
- No withdrawals.
- No bridges.
- No broadcasts.
- No real CEX API calls.
- No real DEX/RPC calls.
- No live trading.

### Deferred Tasks

- Full triangular route path search.
- Inventory-aware sizing.
- Depth-aware slippage beyond top-of-book quantity limits.
- Cross-venue transfer latency and settlement-risk modeling.
- Durable planner/audit/state lifecycle integration.
- External Rust validation.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Deferred until Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode opportunity-engine model/ranking boundary implementation only. External Rust validation, advanced route modeling, planner integration, execution-adapter integration, live connector validation, and production runtime validation remain required before any use with real funds.

## Phase 10 — Execution Planner

### Status

Implemented as draft-only execution-planner model boundary; Rust toolchain validation deferred because Rust/Cargo is unavailable in the ChatGPT execution environment.

### Goal

Generate deterministic execution-plan drafts and per-leg execution intents from validated opportunities without submitting anything to adapters.

### Completed Tasks

- Created `PHASE_10_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/planner.rs`.
- Added `ExecutionPlannerConfig`, `ExecutionPlannerRequest`, `ExecutionPlanDraft`, `ExecutionPlanStep`, `ExecutionPlanFailureMode`, `PlannerPolicyOutcome`, and validation errors.
- Added `ExecutionPlanner` trait and `DeterministicExecutionPlanner`.
- Generated one draft `ExecutionIntent` per validated opportunity leg.
- Rejected live planner scope fail-closed.
- Evaluated each generated draft intent through `PolicyEngine` and captured redacted approval/denial outcomes.
- Modeled deterministic sequencing and failure boundaries without adapter submission.
- Exported planner types from `arb-core`.
- Updated CLI status output and structure validation.

### Explicit Non-Goals

- No live trading.
- No adapter submission.
- No order placement.
- No signing.
- No withdrawals.
- No bridges.
- No broadcasts.
- No real CEX API calls.
- No real DEX/RPC calls.
- No autonomous execution loop.

### Deferred Tasks

- Durable audit records for plan creation and policy outcomes.
- Durable state lifecycle/checkpointing for plan drafts.
- Execution adapter handoff in Phase 11.
- Partial-fill, timeout, cancellation, and hedge sequencing validation.
- External Rust validation.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Deferred until Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode execution-planner model boundary implementation only. External Rust validation, durable audit/state integration, execution-adapter integration, signer/custody integration, live connector validation, and production runtime validation remain required before any use with real funds.

## Phase 11 — Execution Adapters

### Goal

Implement execution-adapter model and trait boundaries that consume planner drafts, revalidate policy, model status/fill/reconciliation records, and preserve fail-closed external-submission controls.

### Status

Implemented in ChatGPT Project Mode as a deterministic framework boundary only.

### Added

- `PHASE_11_SUBROADMAP.md`
- `crates/arb-core/src/execution_adapter.rs`
- `ExecutionAdapterConfig`
- `ExecutionAdapterRequest`
- `ExecutionAdapterRunRecord`
- `ExecutionAdapterAttempt`
- `ExecutionFillRecord`
- `ExecutionReconciliationRecord`
- `ExecutionAdapter` trait
- `DeterministicExecutionAdapterBoundary`
- `EXECUTION_ADAPTER_FRAMEWORK_VERSION`

### Completed

- Added adapter-boundary request/config validation.
- Added deterministic run records for plan-level adapter evaluation.
- Added per-intent attempt records.
- Added deterministic paper fill records without external submission.
- Added reconciliation records for modeled fills and blocked paths.
- Revalidated every intent through `PolicyEngine` at adapter boundary.
- Preserved `external_submission_enabled = false` and per-attempt `submitted_to_external_adapter = false`.
- Rejected live scope and external adapter submission fail-closed.
- Surfaced framework status in `arb-agent`.
- Updated structure validator for Phase 11 files.

### Explicitly Not Implemented

- Live CEX orders.
- Live DEX swaps.
- Real exchange adapters.
- RPC calls.
- Signing.
- Transaction broadcasts.
- Withdrawals.
- Bridges.
- Secrets or custody.
- Durable audit/state runtime integration.
- Balance mutation against real venues or wallets.

### Validation

Passed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempted but blocked because Cargo is unavailable in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode execution-adapter model/trait boundary implementation only. External Rust validation, durable audit/state integration, exchange-specific live connectors, signer/custody integration, live submission controls, and production runtime validation remain required before any use with real funds.

## Phase 12 — Communications and CLI

### Goal

Provide operator controls and alerts through CLI plus configured messaging channels while preserving fail-closed execution and no-outbound-network boundaries.

### Status

Implemented in ChatGPT Project Mode as a deterministic communications and CLI model/trait boundary only.

### Added

- `PHASE_12_SUBROADMAP.md`
- `crates/arb-core/src/communications.rs`
- `COMMUNICATIONS_CLI_VERSION`
- `CommunicationBoundaryConfig`
- `NotificationChannelProfile`
- `OperatorCommand`
- `OperatorCommandRouter` trait
- `DeterministicOperatorCommandRouter`
- `OperatorNotification`
- `NotificationPublisher` trait
- `DeterministicNotificationBoundary`
- `NotificationDispatchRecord`

### Completed

- Added typed non-secret channel profiles.
- Added local CLI command parsing boundary.
- Added deterministic operator-command routing records.
- Rejected live execution, withdrawal, bridge, signing, and broadcast commands.
- Added typed notification payload and dispatch records.
- Added deterministic local notification publisher boundary.
- Preserved `execution_enabled = false` for all routed commands.
- Preserved `outbound_network_used = false` for all command routes, dispatches, and channel records.
- Added secret-like text detection and redaction/truncation helpers for operator-facing messages.
- Surfaced communications/CLI status in `arb-agent`.
- Updated structure validator for Phase 12 files.

### Explicitly Not Implemented

- Telegram, Discord, Slack, Matrix, email, PagerDuty, Signal, iMessage, webhook, SMS, or other real delivery adapters.
- Messaging platform tokens or credentials.
- Outbound HTTP, SMTP, WebSocket, bot, or platform API calls.
- Live trading commands.
- Withdrawals.
- Bridges.
- Signing.
- Transaction broadcasts.
- Real exchange/RPC calls.
- Policy bypass.

### Validation

Passed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempted but blocked because Cargo is unavailable in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode communications and CLI model/trait boundary implementation only. External Rust validation, real messaging adapters, authentication, platform-token storage, notification audit/state integration, and production runtime operator UX validation remain required before production use.

## Phase 13 — Embedded Dashboard

### Status

Implemented for ChatGPT Project Mode as a deterministic embedded-dashboard model/trait boundary.

### Goal

Provide optional lightweight local dashboard boundaries without starting a web server or exposing a network surface.

### Completed Tasks

- Created `PHASE_13_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/dashboard.rs`.
- Added dashboard boundary version marker.
- Added dashboard config and loopback-only server-binding model.
- Added snapshot, panel, item, severity, render-request, and render-record models.
- Added `DashboardRenderer` trait and `DeterministicDashboardRenderer`.
- Added fail-closed rejection for HTTP server startup, public exposure, non-loopback bind hosts, live controls, and secret rendering.
- Added secret-like display redaction for local render records.
- Exported dashboard types from `arb-core`.
- Surfaced the dashboard boundary version in `arb-agent` status output.
- Updated structure validator for Phase 13 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Real dashboard hosting.
- HTTP server startup.
- Public web exposure.
- Authentication/session handling.
- WebSocket, SSE, polling, or browser delivery.
- Live execution controls.
- Remote command execution.
- Signing, withdrawals, bridges, broadcasts, or adapter submission.
- Real exchange/RPC calls.
- Policy bypass.

### Validation

Passed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempted but blocked because Cargo is unavailable in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode embedded-dashboard model/trait boundary implementation only. External Rust validation, real dashboard hosting, authentication/session design, CSRF protection, secure headers, rate limiting, audit/state integration, UX validation, and penetration testing remain required before production dashboard use.

## Phase 14 — Observability and Runbooks

### Status

Implemented for ChatGPT Project Mode as a deterministic observability and runbook model/trait boundary.

### Goal

Add local-only health, structured-log, metric, and runbook record boundaries without starting telemetry services, exposing metrics endpoints, sending alerts, or collecting secrets.

### Completed Tasks

- Created `PHASE_14_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/observability.rs`.
- Added observability/runbook boundary version marker.
- Added `ObservabilityBoundaryConfig` and fail-closed metrics endpoint binding model.
- Added health status and component status models.
- Added structured log event and field models.
- Added deterministic metric sample and label models using integer micro-units.
- Added operator runbook and runbook step models.
- Added observability snapshot, collection request, and collection record models.
- Added `ObservabilityCollector` trait and `DeterministicObservabilityCollector`.
- Added fail-closed rejection for metrics endpoint startup, public exposure, non-loopback bind hosts, outbound alert delivery, and secret observability.
- Added secret-like text redaction before local observability records are returned.
- Exported observability types from `arb-core`.
- Surfaced the observability/runbook boundary version in `arb-agent` status output.
- Updated structure validator for Phase 14 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Real metrics endpoint hosting.
- HTTP server startup.
- Public metrics or telemetry exposure.
- OpenTelemetry exporters.
- Prometheus scraping endpoint.
- Log shipping or SIEM integrations.
- PagerDuty, Slack, email, webhook, SMS, or other outbound alert delivery.
- Panic hooks that exfiltrate process state.
- Secret telemetry, credential logging, wallet-key logging, provider-token logging, signing, withdrawals, bridges, broadcasts, or adapter submission.
- Real exchange/RPC calls.
- Policy bypass.

### Validation

Passed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempted but blocked because Cargo is unavailable in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode observability and runbook model/trait boundary implementation only. External Rust validation, real tracing/logging subscriber integration, Prometheus/OpenTelemetry exporters, authenticated metrics endpoint design, alert routing, log retention/rotation policies, audit/state integration, incident drills, and production runtime validation remain required before production observability use.

## Phase 15 — Testing, Fuzzing, and Backtesting

### Status

Implemented for ChatGPT Project Mode as a deterministic testing, fuzzing, fixture, and backtesting model/trait boundary.

### Goal

Strengthen deterministic validation planning through typed test, fixture, fuzz corpus, and backtest scenario records without invoking external fuzzers, live networks, credentials, signing, broadcasts, or live execution.

### Completed Tasks

- Created `PHASE_15_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/testing.rs`.
- Added testing/backtesting boundary version marker.
- Added `ValidationHarnessConfig` with fail-closed live-network, external-fuzzer, live-execution, and secret-fixture toggles.
- Added suite kind, expected outcome, fixture, fuzz-target, and execution-mode enums.
- Added `ValidationTestCase`, `ValidationFixtureRecord`, `FuzzSeedRecord`, `FuzzCorpusDefinition`, `BacktestDatasetDefinition`, and `BacktestScenarioDefinition`.
- Added `ValidationPlan`, `ValidationRunRequest`, and `ValidationRunRecord`.
- Added `ValidationHarness` trait and `DeterministicValidationHarness`.
- Added duplicate-ID, count-limit, digest, asset, secret-like text, and live-scope validation helpers.
- Ensured validation run records preserve external side-effect denial flags.
- Exported testing/backtesting types from `arb-core`.
- Surfaced the testing/backtesting boundary version in `arb-agent` status output.
- Updated structure validator for Phase 15 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Live network testing.
- External fuzzer process invocation.
- Real property-test runner integration.
- Real fuzzing engine integration.
- Real backtest market-data downloads.
- Live CEX/DEX/RPC calls.
- Order placement, swaps, withdrawals, bridges, signing, or transaction broadcasts.
- Real credentials, secrets, or secret-bearing fixtures.
- Deployment/load/penetration test execution.

### Validation

Executed successfully in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Deferred because Rust/Cargo is unavailable in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode testing, fuzzing, fixture, and backtesting boundary implementation only. External Rust validation, property-test execution, fuzzing engine execution, fixture corpus expansion, replay/backtest execution, load testing, penetration testing, CI validation, and production runtime validation remain required before production claims.

## Phase 16 — Packaging and Deployment

### Status

Implemented for ChatGPT Project Mode as deterministic packaging and deployment model/documentation boundaries.

### Goal

Package for local, VPS, and ARM-capable environments without claiming builds, service installs, public exposure, production deployment, or live-readiness.

### Completed Tasks

- Created `PHASE_16_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/packaging.rs`.
- Added packaging/deployment boundary version marker.
- Added `PackagingBoundaryConfig` with fail-closed public-exposure, live-trading, embedded-secret, and production-claim toggles.
- Added deployment environment, artifact kind, network exposure, runtime configuration, and service-hardening models.
- Added `PackageTargetPlan`, `ReleaseGate`, `RollbackStep`, `DeploymentPackagePlan`, `DeploymentPackageRequest`, and `DeploymentPackageRecord`.
- Added `PackagingDeploymentPlanner` trait and `DeterministicPackagingDeploymentPlanner`.
- Ensured records preserve no-build, no-deployment, no-public-exposure, no-live-trading, no-secret-embedding, and no-production-claim flags.
- Added example-only deployment docs for container, systemd, ARM, and deployment validation notes.
- Exported packaging/deployment types from `arb-core`.
- Surfaced the packaging/deployment boundary version in `arb-agent` status output.
- Updated structure validator for Phase 16 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Container image building.
- systemd service installation or startup.
- ARM cross-build execution.
- Public dashboard, metrics, command, or control exposure.
- Real deployment, cloud provisioning, or production rollout.
- Real credentials, secrets, or credential-bearing artifacts.
- Release signing, SBOM generation, dependency audit, load testing, penetration testing, rollback drills, or incident drills.

### Validation

Executed successfully in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Attempted but blocked because Rust/Cargo is unavailable in this environment:

```bash
cargo fmt --check
```

Still required externally:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
container build validation in an approved runtime
systemd unit validation on Linux
ARM target build validation
rollback drill validation
```

### Exit Criteria

Met for ChatGPT Project Mode packaging and deployment boundary implementation only. External Rust, container, systemd, ARM, CI, runtime, rollback, load, penetration, and production deployment validation remain required before production claims.

## Phase 17 — External Production Hardening

### Status

Implemented as deterministic evidence/checklist boundary in ChatGPT Project Mode. Real external hardening execution remains deferred.

### Goal

Make environment-limited production validation explicit, evidence-based, and fail-closed without claiming work that ChatGPT cannot run.

### Completed Deliverables

- `PHASE_17_SUBROADMAP.md`
- `crates/arb-core/src/hardening.rs`
- `ExternalHardeningBoundaryConfig`
- `HardeningEvidenceRecord`
- `ProductionHardeningPlan`
- `ExternalHardeningReviewRecord`
- `ExternalHardeningReviewer` trait
- `DeterministicExternalHardeningReviewer`
- External validation runbook, production readiness checklist, and incident-response drill template
- Structure validator updated for Phase 17 files
- CLI status text updated

### Validation Completed

- `python3 scripts/validate_structure.py`
- `python3 -m py_compile scripts/validate_structure.py`

### Validation Deferred

The following remain external and were not run in ChatGPT Project Mode:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
dependency audit
SBOM generation and review
container build and image scan
systemd hardening validation
ARM target validation
staging deployment validation
load and soak tests
penetration test
rollback drill
incident-response drill
live exchange/RPC sandbox validation
production readiness review
```

### Exit Criteria

Met for ChatGPT Project Mode evidence-boundary implementation only. No production readiness, live-funds readiness, public exposure readiness, cloud deployment, penetration test, load test, live exchange validation, or release validation is claimed.

## Phase 18 — Agentic Handoff Package

### Status

Implemented for ChatGPT Project Mode.

### Goal

Create final handoff instructions, prompts, and checklists for external coding agents and human maintainers while preserving all unresolved gaps and live-funds blockers.

### Completed Tasks

- Created `PHASE_18_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/handoff.rs` deterministic handoff package model/trait boundary.
- Added conservative handoff package construction with authoritative files, completed phases, unresolved gaps, live-funds blockers, and non-executing artifacts.
- Added local review records that explicitly preserve `external_agents_executed = false`, `external_validation_claimed = false`, `production_ready = false`, `live_funds_approved = false`, `public_exposure_approved = false`, and `secret_material_recorded = false`.
- Added `handoff/AGENTIC_HANDOFF_PACKAGE.md`.
- Added `handoff/FUTURE_AGENT_PROMPTS.md`.
- Added `handoff/EXTERNAL_VALIDATION_CHECKLIST.md`.
- Exported handoff types from `arb-core`.
- Surfaced Phase 18 status in `arb-agent`.
- Updated structure validator for Phase 18 files.

### Deferred Tasks

- External agents were not executed.
- Rust/Cargo validation remains external because the ChatGPT environment lacks Rust tooling.
- CI, release build, dependency audit, SBOM, image scan, staging deployment, load testing, penetration testing, rollback drills, incident drills, exchange/RPC sandbox validation, custody review, compliance review, and production readiness review remain external.

### Acceptance Criteria

Met for deterministic handoff package records and documentation only. This phase adds no production readiness and does not approve live funds, public exposure, production deployment, or autonomous live execution.

### Next Required Action

Run external Rust validation and production-hardening evidence generation in a capable environment before any production, live-funds, public-service, or external-validation claim.

