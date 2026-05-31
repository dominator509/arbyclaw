# ROADMAP.md

## Project

ArbyClaw

## Current Roadmap Position

- Active phase: Phase 26 - Audit Crash, Concurrency, Filesystem, Disk-Full, and Stale-Lock Validation implemented for local audit journal durability probes; next required work is deployment-host/runtime validation and external sandbox/live evidence beyond current Rust/CI gates
- Active sub-roadmap: `PHASE_26_SUBROADMAP.md`
- Current production readiness: 96%
- Current implementation status: Minimal Rust workspace, typed config, reference-only secret boundary, mode-gate validation, deny-by-default policy engine, append-only audit journal primitives with local lock/sync append behavior, crash/concurrency/filesystem/simulated-disk-full validation probes, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning, state-store trait boundary with SQLite WAL-backed checkpoint store, local integrity/checkpoint/reopen/backup-restore/multi-handle durability validation, process-level crash/restart recovery tests, and local runtime state-permission fail-closed validation, normalized market-data models, fee models, freshness classification, provider trait boundaries, deterministic paper connectors with local paper-report checkpoint persistence, local paper balance ledgering, realistic local paper fill modeling, venue matching profiles, adverse-selection modeling, reference-only calibration records, paper ledger replay validation, local historical-fixture paper backtest corpus execution, and direct local audit journal records for paper reports plus reserve/settlement ledger mutations, CEX connector framework types/traits, DEX/Web3 connector framework types/traits, deterministic opportunity-engine types/traits, draft-only execution-planner types/traits with local plan-draft checkpoint persistence, execution-adapter boundary records/traits with local run checkpoint persistence, local fail-closed runtime lifecycle wiring for audit/state/adapter sequencing, local concurrent runtime lifecycle access checks, local graceful-shutdown audit/state checkpointing, local runtime audit/SQLite backup-restore validation, local runtime restart recovery summaries with CLI-visible operator-review dispositions and incomplete-checkpoint fail-closed coverage, local deployment-like runtime smoke validation and CLI runner without service-manager actions, communications/CLI command and notification boundaries, embedded-dashboard local render boundaries, observability/runbook local record boundaries, deterministic testing/fuzzing/backtesting plan boundaries, deterministic packaging/deployment plan boundaries with repeatable local example-container validation and static example systemd-unit validation, deterministic external-hardening evidence/checklist boundaries, and deterministic agentic handoff package boundaries exist; local and GitHub Actions evidence covers current structure, formatting, workspace compilation, tests, clippy, locked release build, dependency audit, SBOM generation, local-SARIF SAST, example image scan, static example systemd-unit checks, secret-pattern scan, and hardening evidence indexing. Live trading, production container/systemd/ARM deployment validation, real validation runner execution beyond Cargo tests, real fuzzing engines, real external backtest corpus execution, real observability runtime, real dashboard hosting, outbound messaging integrations, external adapter submission, exchange-specific live connectors, DEX RPC adapters, wallet signer, transaction broadcasts, custody backend, deployment-host durability validation, physical disk-full and retention/rotation execution evidence, service-manager restart execution evidence, external sandbox/live calibration evidence, and production execution logic are not implemented.
- Current risk posture: High, because live-funds architecture is policy-, audit-, market-data-, paper-simulation-, paper-ledger-, paper-realism/replay/backtest/audit-, CEX-framework-, DEX/Web3-framework-, opportunity-engine-, planner-, adapter-, runtime-lifecycle-, communication-, dashboard-, observability-, and validation-boundary-gated but lacks custody isolation, exchange-specific live connectors, live DEX/RPC adapters, signer boundary, live adapter submission, real communications adapters, real dashboard hosting/authentication, real metrics/exporter/alert runtime, property/fuzz/backtest execution beyond current Cargo tests, real package/deployment validation, broad external hardening validation, future-agent execution validation, deployment-host database and audit validation, physical disk-full and retention/rotation evidence, external sandbox/live fill calibration evidence, and external validation.

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
| 1 | Rust Workspace Scaffold | Scaffold complete; current workspace Rust/CI validation covered | +2% realized / +1% deferred | Minimal workspace, CI skeleton, safety docs, and structure validator created. Future changes must rerun Cargo validation. |
| 2 | Config, Secrets, and Mode Gates | Implemented; current workspace Rust/CI validation covered | +5% realized / +1% deferred | Typed config, environment secret references, keystore interface boundary, and live mode-gate validation added. |
| 3 | Policy Engine and Trust Contract | Implemented; current workspace Rust/CI validation covered | +7% realized / +3% deferred | Deny-by-default policy checks, intent model, trust-contract denials, and CLI policy initialization added. |
| 4 | Audit Journal and State Store | Implemented; current workspace Rust/CI validation covered; deployment-host durability validation deferred | +4% realized / +2% deferred | Append-only hash-chained JSONL audit primitives, redaction checks, state-store trait, in-memory store, SQLite WAL checkpoint store, Phase 26 local audit crash/concurrency/filesystem/simulated-disk-full probes, side-effect-free retention planning, and stale-lock restart recheck planning added; physical disk-full, retention/rotation execution, service-manager restart execution, and deployment-host validation remain future work. |
| 5 | Market Data Core | Implemented; current workspace Rust/CI validation covered; live provider validation deferred | +5% realized / +2% deferred | Normalized quotes/order books, freshness windows, fee models, and provider traits added; no live network providers. |
| 6 | Simulated/Paper Connectors | Implemented; current workspace Rust/CI validation covered; paper-model limitations deferred | +6% realized / +1% deferred | Deterministic in-memory paper market data, static fee provider, policy-gated paper execution adapter, and local paper-report state checkpoint helper added; no live venues or balances. |
| 7 | CEX Connector Framework | Implemented as framework boundary; current workspace Rust/CI validation covered; live exchange validation deferred | +6% realized / +2% deferred | CEX venue profiles, capability registry, order request model, policy gate, and connector traits added; no exchange-specific adapters or live orders. |
| 8 | DEX/Web3 Connector Framework | Implemented as framework boundary; current workspace Rust/CI validation covered; live RPC, signing, and broadcast validation deferred | +8% realized / +0% deferred in ChatGPT | Chain/router/token profiles, router capabilities, swap quote models, local transaction simulation boundary, policy gate, and connector traits added; no live RPC, signing, bridges, or broadcasts. |
| 9 | Opportunity Engine | Implemented as deterministic discovery/ranking boundary; current workspace Rust/CI validation covered; advanced route validation deferred | +8% realized / +0% deferred in ChatGPT | Cross-venue top-of-book discovery, CEX/CEX, DEX/DEX, CEX/DEX, triangular model boundary, freshness checks, and fee-aware scoring added; no execution intents or order placement. |
| 10 | Execution Planner | Implemented as draft-only model boundary; current workspace Rust/CI validation covered; adapter integration deferred | +7% realized / +0% deferred in ChatGPT | Deterministic plan drafts, per-leg intent generation, policy preflight outcomes, sequencing, failure-mode boundaries, and local plan-draft state checkpoint helper added; no adapter submission or live execution. |
| 11 | Execution Adapters | Implemented as deterministic boundary framework; current workspace Rust/CI validation covered; live submission deferred | +7% realized / +0% deferred in ChatGPT | Consumes planner drafts, revalidates policy, models attempts/fills/reconciliation, and blocks all external submission. |
| 12 | Communications and CLI | Implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real outbound integrations deferred | +6% realized / +0% deferred in ChatGPT | Typed local command parsing/routing, notification models, redaction checks, and local dispatch records added; no platform tokens or outbound network delivery. |
| 13 | Embedded Dashboard | Implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real hosting deferred | +3% realized / +0% deferred in ChatGPT | Local snapshot/panel/render records, fail-closed server binding, secret redaction, and live-control denial added; no web server or public exposure. |
| 14 | Observability and Runbooks | Implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real observability runtime deferred | +5% realized / +0% deferred in ChatGPT | Local health, structured-log, metric, and runbook records added; metrics endpoints and outbound alerts denied. |
| 15 | Testing, Fuzzing, and Backtesting | Implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real fuzz/backtest execution deferred | +4% realized / +2% deferred | Validation harness config, test case metadata, fixture records, fuzz corpus definitions, backtest scenario definitions, and local plan records added; no external fuzzer invocation or live network tests. |
| 16 | Packaging and Deployment | Implemented as deterministic model/docs boundary; current release-build, example-container, and example systemd-unit static/syntax validation gates covered; production container/systemd/ARM validation deferred | +2% realized / +0% deferred in ChatGPT | Package/deployment plan records, release gates, rollback steps, Docker/systemd/ARM docs, repeatable local example-container validation script, and static plus optional syntax example systemd-unit validator; no production build/install/deploy claim. |
| 17 | External Production Hardening | Implemented as deterministic evidence/checklist boundary; real external validation deferred | +0% in ChatGPT | Evidence records, release blockers, and hardening checklists added; no pen test, cloud deployment, live exchange validation, or load test executed. |
| 18 | Agentic Handoff Package | Implemented as deterministic model/docs boundary; external agent execution not performed | +0% direct | Codex/Cursor/Jules/Claude/human handoff package records, prompts, and checklists added; no external agents executed. |
| 19 | Runtime Lifecycle Wiring | Implemented as local deterministic fail-closed lifecycle boundary; production durability/runtime validation deferred | +2% realized / +0% deferred in ChatGPT | Runtime lifecycle records append audit events, persist planner state before adapter evaluation, evaluate deterministic adapter boundary, persist adapter run state, reject live scope without external submission, validate concurrent local audit/SQLite lifecycle access, fail closed on simulated state permission failure, record local graceful-shutdown audit/state checkpoints without service actions, validate local audit/SQLite backup-restore copies without deployment actions, produce local restart recovery summaries with CLI-visible operator-review dispositions and without service resume, fail closed on incomplete recovery checkpoints, and now provide a local deployment-like smoke harness that combines those checks with audit durability probes without service-manager actions. |
| 20 | SQLite WAL Durability Validation | Implemented as local deterministic state-store validation boundary; external production-host validation deferred | +1% realized / +0% deferred in ChatGPT | Validates WAL mode, synchronous FULL, integrity check, WAL checkpoint truncate, primary reopen, checkpointed backup/restore, and multi-handle visibility with non-secret probes. |
| 21 | Paper Balance Ledgering | Implemented as local deterministic paper balance boundary; paper realism/audit/runtime validation deferred | +1% realized / +0% deferred in ChatGPT | Adds simulated paper balances, quote-notional reservation, fill settlement with net P&L, insufficient-balance denial, missing-reservation denial, and SQLite ledger checkpointing. |
| 22 | Crash/Restart Durability Validation | Implemented as local process-level SQLite WAL recovery validation; deployment-host validation deferred | +1% realized / +0% deferred in ChatGPT | Spawns child processes that write runtime checkpoints and exit abruptly, then reopens the WAL database and verifies integrity plus expected checkpoint survival. |
| 23 | Realistic Paper Fills | Implemented as local deterministic order-book depth and partial-fill modeling; external calibration deferred | +1% realized / +0% deferred in ChatGPT | Consumes supplied order-book depth, models latency, queue-position, slippage, full/partial/unfilled outcomes, and ledger-safe unfilled notional release without external submission. |
| 24 | Paper Replay, Calibration, Backtest, and Runtime Validation Boundaries | Implemented as local deterministic venue-realism, replay, and historical-fixture backtest boundary; external sandbox/live and deployment-host validation deferred | +1% realized / +0% deferred in ChatGPT | Adds local exchange matching profiles, adverse-selection penalties, reference-only calibration records, paper ledger replay validation, local backtest corpus execution, and runtime validation records without external calls. |
| 25 | Paper Audit Journal Integration | Implemented as local deterministic audit journal integration; production audit durability validation deferred | +1% realized / +0% deferred in ChatGPT | Adds append-only audit records for paper execution reports and paper reserve/settlement ledger mutations, with local journal reopen/replay tests and no external calls. |
| 26 | Audit Crash, Concurrency, Filesystem, Disk-Full, and Stale-Lock Validation | Implemented as local deterministic audit durability validation; deployment-host evidence deferred | +1% realized / +0% deferred in ChatGPT | Adds lock/sync append behavior plus local probes for append replay, truncated crash-like replay rejection, tamper rejection, concurrent appends, invalid filesystem failure, simulated disk-full fail-closed behavior, retention/rotation planning without deletion, stale-lock restart recheck planning without service actions, and a local deployment-like runtime smoke harness plus CLI runner without service-manager actions. |

Potential total inside ChatGPT Project Mode: approximately 75-96% of code/documentation readiness, but not full production readiness because live infrastructure, external exchange credentials, real deployment, penetration testing, deployment-host audit validation, external sandbox/live calibration evidence, deployment-host durability validation, and live trading verification are environment-limited.

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

Scaffold complete; current workspace Rust validation now has local and GitHub Actions evidence. Future changes must rerun validation for their exact workspace state.

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

The following commands now have current local and GitHub Actions evidence for the present workspace state:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Known Limitations

- No compiled binary has been produced in ChatGPT Project Mode.
- Hosted CI now runs for pushed commits on `dominator509/arbyclaw`.
- No runtime execution validation was performed.
- No trading behavior exists.

### Exit Criteria

Met for scaffold creation. Full Phase 1 build validation remains tracked as environment-limited work in `PRODUCTION_GAP_TRACKER.md`.

## Phase 2 — Config, Secrets, and Mode Gates

### Status

Implemented in ChatGPT Project Mode; current workspace Rust/CI validation evidence exists and must be refreshed after changes.

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

The following commands have current local and GitHub Actions evidence for the present workspace state:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode scaffold/config implementation with current workspace Rust/CI validation evidence. Future changes still require rerunning validation before protected-branch or downstream reliance.

## Phase 3 — Policy Engine and Trust Contract

### Status

Implemented in ChatGPT Project Mode; current workspace Rust/CI validation evidence exists and must be refreshed after changes.

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

The following commands have current local and GitHub Actions evidence for the present workspace state:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode policy implementation with current workspace Rust/CI validation evidence. Property tests, audit integration, and execution-adapter integration remain required.

## Phase 4 — Audit Journal and State Store

### Status

Implemented in ChatGPT Project Mode with current local/CI Rust validation evidence; SQLite WAL checkpoint persistence, local state-store durability validation, Phase 26 local audit crash/concurrency/filesystem/simulated-disk-full probes, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning are implemented, while deployment-host audit validation, physical disk-full behavior, retention/rotation execution, service-manager restart execution behavior, and external production-host validation remain deferred.

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
- Local lock/sync append behavior
- Local crash-like truncation rejection
- Local concurrent append replay validation
- Local invalid-filesystem fail-closed validation
- Local simulated disk-full append failure classification and state-preservation validation
- Local retention/rotation planning that never deletes or mutates files
- Local stale-lock restart recheck planning that never deletes lock files, inspects live processes, starts services, or mutates deployment state

### State Boundary Implemented

- `StateCheckpoint` model
- `StateStore` trait
- `InMemoryStateStore` for tests and early local wiring only
- `SqliteWalStateStore` for local SQLite WAL-backed checkpoint persistence
- Secret-like checkpoint content rejection

### Validation Completed

- Governance files reread and reconciled.
- Required Phase 4 files created.
- Structure validator passed with `python3 scripts/validate_structure.py`.
- Secret-assignment static scan passed using `scripts/validate_structure.py`.

### Validation Deferred

The following commands have current local and GitHub Actions evidence for the present workspace state:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Additional Phase 4 validations covered locally as of Phase 26:

- audit append/reopen test execution
- audit tamper-detection test execution
- audit redaction test execution
- crash-like truncated replay rejection
- concurrent append replay validation
- invalid-filesystem failure validation

Additional Phase 4 production validations deferred:

- deployment-host audit crash/recovery testing
- physical deployment-host disk-full behavior testing
- deployment-host retention/rotation execution validation
- deployment-host service-manager restart execution validation
- SQLite WAL migration, deployment file-locking, backup/restore under deployment load, and filesystem-permission validation

### Exit Criteria

Met for ChatGPT Project Mode audit/state boundary implementation with current workspace Rust/CI validation evidence and local Phase 26 audit durability probes. Deployment-host audit/state validation remains required before any live execution path may depend on audit persistence.

## Phase 5 — Market Data Core

### Status

Implemented in ChatGPT Project Mode as model and trait boundaries with current workspace Rust/CI validation evidence; live provider validation remains deferred.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode market-data/fee-model boundary implementation with current workspace Rust/CI validation evidence. Live provider validation remains required before market data may be used for production decisions.

## Phase 6 — Simulated/Paper Connectors

### Status

Implemented in ChatGPT Project Mode as deterministic in-memory paper connector boundaries with current workspace Rust/CI validation evidence; local paper-balance ledgering, realistic local fill simulation, venue matching profiles, adverse-selection modeling, reference-only calibration records, paper ledger replay validation, and local historical-fixture backtest execution now exist, while external sandbox/live calibration evidence and deployment-host runtime validation remain deferred.

### Goal

Enable deterministic strategy testing without live funds.

### Completed Tasks

- Created `PHASE_6_SUBROADMAP.md`.
- Added `arb-core::paper` with `PaperMarketDataProvider`, `PaperFeeProvider`, and `PaperExecutionAdapter`.
- Implemented in-memory paper order-book lookup through the existing `MarketDataProvider` trait.
- Implemented static paper fee lookup through the existing `FeeProvider` trait.
- Implemented policy-gated paper execution reports for paper-scoped intents only.
- Added a typed local `StateStore` checkpoint helper for the latest paper execution report, including SQLite WAL persistence/reopen coverage.
- Added local paper balance ledgering in Phase 21 with quote-notional reservation, deterministic settlement, insufficient-balance denial, missing-reservation denial, and SQLite checkpoint persistence.
- Added realistic local fill simulation in Phase 23 with supplied order-book depth walking, partial/unfilled outcomes, latency, queue-position, average price, slippage, and ledger-safe unfilled notional release.
- Added Phase 24 local paper venue-realism, replay, and backtest validation records for exchange matching profiles, adverse selection, reference-only calibration records, ledger replay, local historical-fixture backtest corpus execution, and runtime validation records that preserve production blockers.
- Exported paper connector primitives through `arb-core`.
- Updated `arb-agent` status output to report paper connector boundary availability.
- Updated structure validation to require Phase 6 files.

### Deferred Tasks

- Cargo format/check/test/clippy validation.
- Position tracking beyond quote-balance ledgering.
- External sandbox/live discrepancy calibration evidence and exchange account validation.
- Audit journal integration for paper execution events.
- Production runtime validation beyond local replay/backtest records and lifecycle checkpointing.
- Broader scenario fixture library, property/fuzz runner integration, and external backtest corpora.
- Live CEX/DEX connector implementation.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode deterministic paper connector boundary implementation with current workspace Rust/CI validation evidence. Local exchange matching profiles, adverse-selection modeling, calibration records, paper replay validation, and local backtest execution are implemented; external sandbox/live calibration evidence and production-host runtime validation remain required before Phase 6 can be treated as tested production-like runtime behavior.

## Phase 7 — CEX Connector Framework

### Status

Implemented as typed framework boundary; current workspace Rust/CI validation covered and live exchange validation deferred.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode CEX framework boundary implementation with current workspace Rust/CI validation evidence. Exchange-specific adapter work, sandbox testing, credential-scope checks, rate-limit validation, and terms/jurisdiction review remain required before any live CEX use.

## Phase 8 — DEX/Web3 Connector Framework

### Status

Implemented as typed framework boundary; current workspace Rust validation covered; live RPC validation, signer validation, transaction simulation integration, and broadcast validation deferred.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode DEX/Web3 framework boundary implementation with current workspace Rust/CI validation evidence. RPC/simulation adapters, signer/custody work, transaction broadcast controls, protocol review, and live/on-chain validation remain required before any DEX/Web3 use with real funds.

## Phase 9 — Opportunity Engine

### Status

Implemented as deterministic discovery/ranking boundary; current workspace Rust/CI validation evidence exists and must be refreshed after changes.

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
- Keep Rust validation current after each workspace change.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode opportunity-engine model/ranking boundary implementation with current workspace Rust/CI validation evidence. Advanced route modeling, planner integration, execution-adapter integration, live connector validation, and production runtime validation remain required before any use with real funds.

## Phase 10 — Execution Planner

### Status

Implemented as draft-only execution-planner model boundary; current workspace Rust/CI validation evidence exists and must be refreshed after changes.

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
- Added a typed local `StateStore` checkpoint helper for the latest execution-plan draft, including SQLite WAL persistence/reopen coverage.
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
- Production runtime validation beyond local fail-closed planner checkpointing.
- Execution adapter handoff in Phase 11.
- Partial-fill, timeout, cancellation, and hedge sequencing validation.
- Keep Rust validation current after each workspace change.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode execution-planner model boundary implementation with current workspace Rust/CI validation evidence. Local runtime lifecycle wiring now checkpoints plans before deterministic adapter-boundary evaluation. Signer/custody integration, live connector validation, restart replay, and production runtime validation remain required before any use with real funds.

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
- `persist_execution_adapter_run_checkpoint`

### Completed

- Added adapter-boundary request/config validation.
- Added deterministic run records for plan-level adapter evaluation.
- Added per-intent attempt records.
- Added deterministic paper fill records without external submission.
- Added reconciliation records for modeled fills and blocked paths.
- Revalidated every intent through `PolicyEngine` at adapter boundary.
- Preserved `external_submission_enabled = false` and per-attempt `submitted_to_external_adapter = false`.
- Rejected live scope and external adapter submission fail-closed.
- Added a typed local `StateStore` checkpoint helper for the latest execution-adapter run.
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
- Production-validated durable audit/state runtime integration.
- Balance mutation against real venues or wallets.

### Validation

Passed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode execution-adapter model/trait boundary implementation with current workspace Rust/CI validation evidence. Local runtime lifecycle wiring now persists adapter-run checkpoints after deterministic adapter-boundary evaluation. Exchange-specific live connectors, signer/custody integration, live submission controls, restart replay, and production runtime validation remain required before any use with real funds.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode communications and CLI model/trait boundary implementation with current workspace Rust/CI validation evidence. Real messaging adapters, authentication, platform-token storage, notification audit/state integration, and production runtime operator UX validation remain required before production use.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode embedded-dashboard model/trait boundary implementation with current workspace Rust/CI validation evidence. Real dashboard hosting, authentication/session design, CSRF protection, secure headers, rate limiting, audit/state integration, UX validation, and penetration testing remain required before production dashboard use.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode observability and runbook model/trait boundary implementation with current workspace Rust/CI validation evidence. Real tracing/logging subscriber integration, Prometheus/OpenTelemetry exporters, authenticated metrics endpoint design, alert routing, log retention/rotation policies, audit/state integration, incident drills, and production runtime validation remain required before production observability use.

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

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for ChatGPT Project Mode testing, fuzzing, fixture, and backtesting boundary implementation with current workspace Rust/CI validation evidence. Phase 24 now executes local paper backtest corpora over caller-supplied fixtures, but property-test execution, fuzzing engine execution, broader fixture corpus expansion, CI-scale replay/backtest execution, load testing, penetration testing, and production runtime validation remain required before production claims.

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
- Added `scripts/validate_container_example.py` for repeatable local example image build, Trivy image scan, critical-vulnerability enforcement, and container CLI smoke checks.
- Exported packaging/deployment types from `arb-core`.
- Surfaced the packaging/deployment boundary version in `arb-agent` status output.
- Updated structure validator for Phase 16 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Production container image building or deployment validation.
- systemd service installation or startup.
- ARM cross-build execution.
- Public dashboard, metrics, command, or control exposure.
- Real deployment, cloud provisioning, or production rollout.
- Real credentials, secrets, or credential-bearing artifacts.
- Release signing, SBOM review, dependency-audit review, production image-scan review, load testing, penetration testing, rollback drills, or incident drills.

### Validation

Executed successfully in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Current workspace validation, locked release build, dependency-audit gate, SBOM-generation gate, local-SARIF SAST gate, example-only container image build/scan gate, secret-pattern scan, and hardening evidence indexing now pass locally or in GitHub Actions where applicable:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
cargo audit
CycloneDX SBOM generation
CodeQL local-SARIF generation
example container build and Trivy image scan
python3 scripts/validate_container_example.py
Gitleaks secret-pattern scan
```

Still required externally:

```bash
production container build validation in an approved runtime
systemd unit validation on Linux
ARM target build validation
rollback drill validation
```

### Exit Criteria

Met for ChatGPT Project Mode packaging and deployment boundary implementation only. Current Rust/CI/release-build/example-image/static-example-systemd evidence does not prove production deployment readiness; production container, deployment-host systemd, ARM, runtime, rollback, load, penetration, and production deployment validation remain required before production claims.

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
candidate-commit CI evidence refresh
SBOM review
dependency-audit review
production container build and image-scan review
GitHub code scanning upload processing or accepted deferral review
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
- Rust/Cargo validation has current local and GitHub Actions evidence for the present workspace state.
- CI, release build, dependency audit, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing now run in GitHub Actions. Staging deployment, load testing, penetration testing, rollback drills, incident drills, exchange/RPC sandbox validation, custody review, compliance review, and production readiness review remain external.

### Acceptance Criteria

Met for deterministic handoff package records and documentation only. This phase adds no production readiness and does not approve live funds, public exposure, production deployment, or autonomous live execution.

### Next Required Action

Keep Rust/CI validation current and continue production-hardening evidence review plus external validation closure before any production, live-funds, public-service, or production-readiness claim.

## Phase 19 — Runtime Lifecycle Wiring

### Status

Implemented for local deterministic lifecycle scope.

### Goal

Wire planner drafts and execution-adapter boundary records through fail-closed local audit/state lifecycle preconditions without enabling live trading, signing, withdrawals, bridges, broadcasts, real exchange/RPC calls, wallet custody, public exposure, or secrets.

### Completed Tasks

- Created `PHASE_19_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/runtime.rs`.
- Added `RuntimeLifecycleRequest`, `RuntimeLifecycleRecord`, `RuntimeLifecycleStatus`, `RuntimeLifecycleError`, and `run_local_runtime_lifecycle`.
- Added `persist_execution_adapter_run_checkpoint` plus adapter state checkpoint constants.
- Enforced audit append and plan state checkpoint before deterministic adapter evaluation.
- Enforced adapter run checkpoint and audit append after deterministic adapter evaluation.
- Rejected live-scope lifecycle requests before audit/state mutation.
- Added in-memory and SQLite WAL-backed lifecycle tests.
- Added local graceful-shutdown audit/state checkpoint records and a SQLite reopen test without stopping services or changing deployment behavior.
- Added a local concurrent runtime lifecycle access test over shared audit journal and SQLite WAL state paths without service startup or deployment changes.
- Added a local state permission-denial test proving lifecycle execution stops before adapter evaluation when state checkpoint persistence fails.
- Added local runtime audit/SQLite backup-restore validation records and a copy/reopen test without service startup, deployment changes, or embedded artifact contents.
- Added local runtime restart recovery validation records and a replay/reopen test without service resume, deployment changes, or embedded artifact contents.
- Added typed local restart recovery dispositions for ready-for-local-review and needs-operator-review, plus a test for missing graceful-shutdown checkpoint review classification.
- Surfaced restart recovery dispositions in `arb-agent` status text as local operator-review labels.
- Added a local incomplete-recovery test proving restart recovery fails closed when audit replay exists but required SQLite lifecycle checkpoints are missing.
- Exported runtime lifecycle types from `arb-core`.
- Surfaced runtime lifecycle status in `arb-agent`.
- Updated structure validator for Phase 19 files.

### Explicit Non-Goals

- No live trading.
- No external adapter submission.
- No real exchange/RPC calls.
- No signing, withdrawals, bridges, or broadcasts.
- No wallet custody or encrypted keystore implementation.
- No production deployment or production-readiness approval.

### Deferred Tasks

- Production durability validation for SQLite WAL under crash, restart, locking, filesystem permission, backup/restore, and concurrent access scenarios.
- Long-running daemon orchestration and deployment-host graceful shutdown execution.
- Real observability runtime integration.
- Real dashboard hosting integration.
- Real outbound communications integration.
- Live/sandbox exchange and RPC validation.
- Custody, signer, encrypted keystore, and external adapter submission phases.
- Penetration, load, rollback, incident-drill, deployment, systemd, ARM, and production-readiness validation.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic runtime lifecycle wiring, local graceful-shutdown audit/state checkpointing, local audit/SQLite backup-restore copy validation, local restart recovery summaries with operator-review dispositions, and incomplete-recovery fail-closed checks only. No production readiness, live-funds readiness, external adapter submission, public exposure readiness, deployment readiness, deployment-load backup/restore approval, service-manager shutdown/restart execution, or live exchange/RPC validation is claimed.

## Phase 20 — SQLite WAL Durability Validation

### Status

Implemented for local deterministic SQLite WAL durability validation.

### Goal

Validate the SQLite WAL state-store boundary for non-secret checkpoint durability across local integrity checks, WAL checkpoint flushing, primary reopen, checkpointed backup/restore, and multi-handle visibility without enabling live trading, signing, withdrawals, bridges, broadcasts, real exchange/RPC calls, wallet custody, public exposure, or secrets.

### Completed Tasks

- Created `PHASE_20_SUBROADMAP.md` before implementation.
- Added `SQLITE_WAL_DURABILITY_VERSION`.
- Added `SqliteWalDurabilityReport`.
- Added `journal_mode`, `synchronous_mode`, `integrity_check`, `wal_checkpoint_truncate`, and `validate_durability` methods to `SqliteWalStateStore`.
- Enforced WAL mode and synchronous FULL checks in the durability validation path.
- Enforced `PRAGMA integrity_check` returning `ok`.
- Enforced `PRAGMA wal_checkpoint(TRUNCATE)` completing without busy pages.
- Added non-secret probe checkpoint writes.
- Added primary database reopen validation.
- Added checkpointed main-database backup copy and backup reopen/read validation.
- Added multi-handle visibility validation.
- Added fail-closed backup path validation for empty, identical, secret-like, and pre-existing backup paths.
- Added Rust tests covering successful durability validation and existing-backup rejection.
- Exported the durability report and version from `arb-core`.
- Surfaced SQLite WAL durability status in `arb-agent`.
- Updated structure validator for Phase 20 files.

### Explicit Non-Goals

- No live trading.
- No external adapter submission.
- No real exchange/RPC calls.
- No signing, withdrawals, bridges, or broadcasts.
- No wallet custody or encrypted keystore implementation.
- No production deployment or production-readiness approval.

### Deferred Tasks

- External production-host crash/restart validation.
- Filesystem permission and physical disk-full validation.
- Long-running runtime lifecycle load validation.
- Deployment-host audit journal durability validation beyond Phase 26 local crash/concurrency/filesystem probes.
- Deployment-like validation with retained non-secret evidence references.
- Container/systemd/ARM deployment validation.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic SQLite WAL durability validation only. External production-host durability validation, deployment readiness, live-funds readiness, public exposure readiness, and live exchange/RPC validation are not claimed.

## Phase 21 — Paper Balance Ledgering

### Status

Implemented for local deterministic paper balance ledgering.

### Goal

Add paper-only simulated balances and ledger entries so modeled paper fills reserve quote notional, fail closed on insufficient balances, settle net paper P&L deterministically, and persist ledger state through the local `StateStore` boundary.

### Completed Tasks

- Created `PHASE_21_SUBROADMAP.md` before implementation.
- Added `PAPER_BALANCE_LEDGER_VERSION` and `PAPER_BALANCE_LEDGER_CHECKPOINT_KEY`.
- Added `PaperAssetBalance`, `PaperBalanceLedger`, `PaperLedgerEntry`, `PaperLedgerEntryKind`, and `PaperLedgeredExecution`.
- Added quote-notional reservation for paper intents.
- Added deterministic settlement for filled paper reports.
- Added insufficient available balance and missing reservation fail-closed errors.
- Added `PaperExecutionAdapter::submit_with_ledger`.
- Added `persist_paper_balance_ledger_checkpoint`.
- Added SQLite WAL persistence/reopen coverage for the paper ledger.
- Exported paper ledger types from `arb-core`.
- Surfaced paper ledger status in `arb-agent`.
- Updated structure validator for Phase 21 files.

### Explicit Non-Goals

- No live trading.
- No real balance reads or real account mutation.
- No external adapter submission.
- No real exchange/RPC calls.
- No signing, withdrawals, bridges, or broadcasts.
- No wallet custody or encrypted keystore implementation.
- No production deployment or production-readiness approval.

### Deferred Tasks

- Depth-aware fill simulation.
- Partial fills.
- Latency and queue-position modeling.
- Exchange-specific matching behavior.
- Production audit durability validation for paper ledger mutation audit records.
- Production runtime replay and deployment-host validation.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic paper balance ledgering only. Real balances, live trading, production deployment, public exposure readiness, and live exchange/RPC validation are not claimed.

## Phase 22 — Crash/Restart Durability Validation

### Status

Implemented for local process-level SQLite WAL crash/restart validation.

### Goal

Validate that committed non-secret runtime checkpoints survive abrupt child-process termination and can be recovered by a fresh parent process through the SQLite WAL state-store boundary.

### Completed Tasks

- Created `PHASE_22_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/tests/sqlite_wal_crash_restart.rs`.
- Added child-process crash/restart harness using the current Cargo test binary.
- Added crash-after-start checkpoint coverage.
- Added crash-after-plan checkpoint coverage.
- Added crash-after-adapter checkpoint coverage.
- Added parent-process SQLite reopen verification after each child exit.
- Added parent-process SQLite integrity-check verification after each child exit.
- Added expected checkpoint presence/absence checks for each crash stage.
- Updated structure validator for Phase 22 files.

### Explicit Non-Goals

- No live trading.
- No real balance reads or real account mutation.
- No external adapter submission.
- No real exchange/RPC calls.
- No signing, withdrawals, bridges, or broadcasts.
- No wallet custody or encrypted keystore implementation.
- No production deployment or production-readiness approval.

### Deferred Tasks

- Physical deployment-host disk-full evidence.
- Filesystem permission fault validation.
- Deployment-host audit journal validation beyond Phase 26 local crash/concurrency/filesystem probes.
- Long-running daemon restart validation.
- Deployment-host crash/restart validation outside local Cargo tests.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local process-level SQLite WAL crash/restart validation only. Deployment-host durability validation, live trading, production deployment, public exposure readiness, and live exchange/RPC validation are not claimed.

## Phase 23 - Realistic Paper Fills

### Status

Implemented for local deterministic realistic paper fill modeling.

### Goal

Replace full-notional paper-fill assumptions with caller-supplied order-book depth consumption, partial/unfilled outcomes, deterministic latency, queue-position haircuts, slippage reporting, and ledger-safe unfilled notional release.

### Completed Tasks

- Created `PHASE_23_SUBROADMAP.md` before implementation.
- Added realistic paper fill model records in `crates/arb-core/src/paper.rs`.
- Added local depth walking over supplied normalized order-book snapshots.
- Added buy-base and sell-base side modeling.
- Added full, partial, and unfilled paper fill statuses.
- Added average fill price, worst fill price, consumed levels, slippage, latency, and queue-position reporting.
- Wired realistic fill reports into paper ledger settlement so unfilled reserved notional is released safely.
- Added focused tests for full depth fills, partial ledger settlement, and fail-closed unfilled outcomes when partial fills are disabled.
- Exported Phase 23 fill model types from `arb-core`.
- Updated structure validator for Phase 23 files.

### Explicit Non-Goals

- No live trading.
- No real balance reads or real account mutation.
- No external adapter submission.
- No real exchange/RPC calls.
- No signing, withdrawals, bridges, or broadcasts.
- No wallet custody or encrypted keystore implementation.
- No exchange-specific matching engine or live venue calibration.
- No production deployment or production-readiness approval.

### Deferred Tasks

- External sandbox/live discrepancy evidence and exchange account validation.
- Broader queue-position calibration against venue data.
- Direct append-only audit journal integration for paper reports.
- Deployment-host production runtime validation.
- Broader fixture corpora, CI-scale replay, and production runtime validation.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic realistic paper fill modeling only. Live trading, production deployment, public exposure readiness, live exchange/RPC validation, external exchange-specific calibration evidence, and production readiness are not claimed.

## Phase 24 - Paper Replay, Calibration, Backtest, and Runtime Validation Boundaries

### Status

Implemented for local deterministic paper validation scope.

### Goal

Close the local coding gaps around paper venue realism and validation by adding exchange-specific matching profiles, adverse-selection modeling, reference-only calibration records, paper ledger replay validation, local historical-fixture backtest corpus execution, and runtime validation records that preserve external production blockers.

### Completed Tasks

- Created `PHASE_24_SUBROADMAP.md` before implementation.
- Added local `PaperExchangeMatchingProfile` records for venue tick size, quantity step, min/max notional, order-type support, partial-fill support, and queue-position behavior.
- Added `PaperVenueRealismRequest` and related execution records that apply exchange matching, adverse-selection penalties, and optional calibration records through the existing paper adapter and ledger.
- Added reference-only sandbox/live calibration records that can store sanitized evidence locators without copying artifact contents or secret material into the repo.
- Added paper ledger replay validation that reconstructs balances from ledger entries and detects mutation mismatches or open reservations.
- Added local paper backtest corpus execution over caller-supplied historical fixtures with no data downloads, live network use, or external execution.
- Added paper runtime validation records that distinguish local replay/backtest coverage from missing production-host evidence.
- Exported Phase 24 paper validation types from `arb-core`.
- Surfaced Phase 24 status in `arb-agent`.
- Updated structure validation for Phase 24 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, or RPC calls.
- No real balance reads or real account mutation.
- No signing, withdrawals, bridges, broadcasts, wallet custody, or external adapter submission.
- No production deployment or production-readiness approval.
- No claim that local calibration records are externally observed sandbox/live evidence unless a future operator supplies non-secret external evidence references.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic paper venue-realism, replay, and fixture-backtest validation only. External sandbox/live calibration evidence, production-host runtime validation, deployment validation, live exchange/RPC validation, custody/signing validation, and production readiness are not claimed.

## Phase 25 - Paper Audit Journal Integration

### Status

Implemented for local deterministic paper audit scope.

### Goal

Wire paper execution reports and local paper balance-ledger mutations into the existing append-only audit journal with local replay tests, while preserving no-live-execution and no-external-call boundaries.

### Completed Tasks

- Created `PHASE_25_SUBROADMAP.md`.
- Added `PAPER_AUDIT_INTEGRATION_VERSION`.
- Added audit append helpers for paper execution reports and paper reserve/settlement ledger entries.
- Added audited ledgered execution helpers for realistic paper fills and venue-realistic paper fills.
- Reopened the local audit journal after paper audit appends to verify hash-chain replay.
- Added Rust tests covering paper report/ledger mutation audit records and journal replay.
- Exported Phase 25 paper audit integration types and helpers from `arb-core`.
- Surfaced Phase 25 status in `arb-agent`.
- Updated structure validation for Phase 25 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, or RPC calls.
- No real balance reads or real account mutation.
- No signing, withdrawals, bridges, broadcasts, wallet custody, or external adapter submission.
- No production deployment or production-readiness approval.
- No audit journal crash, concurrency, filesystem permission, disk-full, rotation, retention, or deployment-host validation claim.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic paper execution report and ledger mutation audit journal wiring only. Audit journal production durability, live connector audit-before-action enforcement, production-host runtime validation, deployment validation, live exchange/RPC validation, custody/signing validation, and production readiness are not claimed.

## Phase 26 - Audit Crash, Concurrency, Filesystem, Disk-Full, and Stale-Lock Validation

### Status

Implemented for local deterministic audit durability validation scope.

### Goal

Fill the local coding gap for append-only audit journal crash, concurrency, and filesystem validation while preserving no-live-execution and no-external-call boundaries.

### Completed Tasks

- Created `PHASE_26_SUBROADMAP.md`.
- Added `AUDIT_DURABILITY_VALIDATION_VERSION`.
- Updated `AppendOnlyAuditJournal::append_event` to acquire a local lock before replay, sequence/hash calculation, and append.
- Updated audit append persistence to flush and `sync_all` the JSONL file.
- Added `AuditDurabilityValidationReport` and `validate_audit_journal_durability`.
- Added local probes for append/reopen replay, crash-like truncated record rejection, tamper/hash-chain rejection, concurrent append replay, invalid filesystem failure, and simulated disk-full append failure.
- Added Rust tests for durability validation, direct partial JSONL replay rejection, permission/disk-failure state preservation, and existing-workspace fail-closed behavior.
- Added side-effect-free retention/rotation planning models and tests that mark rotate, retain, and expired decisions without deleting logs.
- Added side-effect-free stale-lock restart recheck planning models and tests that mark stale/fresh lock observations without deleting lock files, inspecting live processes, starting services, or mutating deployment state.
- Exported Phase 26 audit validation types from `arb-core`.
- Surfaced Phase 26 status in `arb-agent`.
- Updated structure validation for Phase 26 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, or RPC calls.
- No real balance reads or real account mutation.
- No signing, withdrawals, bridges, broadcasts, wallet custody, or external adapter submission.
- No production deployment or production-readiness approval.
- No claim that local validation proves every deployment filesystem, physical disk-full condition, retention/rotation policy, container runtime, service-manager restart execution, or remote storage layer.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic audit append/replay, crash-like truncation rejection, tamper rejection, concurrent append replay, invalid filesystem fail-closed validation, simulated disk-full fail-closed validation, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning only. Deployment-host audit validation, physical disk-full evidence, retention/rotation execution validation, service-manager restart execution validation, live exchange/RPC validation, custody/signing validation, and production readiness are not claimed.
