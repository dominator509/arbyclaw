# ARCHITECTURE.md

## Project Name

Fully Autonomous Crypto Arbitrage Agent

## Architecture Status

- Status: Phase 18 agentic handoff package boundary implemented; current workspace Rust/CI validation evidence exists, while external agent execution and real production hardening validation remain deferred.
- Production readiness: 87% unchanged; Phase 18 adds handoff governance but no externally executed validation or production approval.
- Implementation status: Minimal Rust workspace, typed config, redacted secret-reference abstractions, CLI config loading, isolated policy engine, append-only audit journal primitives, state-store trait boundary, normalized market-data models, freshness classification, fee models, provider trait boundaries, deterministic paper connectors, CEX connector framework types/traits, DEX/Web3 framework types/traits, deterministic opportunity-engine types/traits, draft-only execution-planner types/traits, execution-adapter boundary records/traits, communications/CLI command and notification boundaries, embedded-dashboard local render boundaries, observability/runbook local record boundaries, testing/fuzzing/backtesting validation-plan boundaries, packaging/deployment plan boundaries, external hardening evidence/checklist boundaries, and agentic handoff package boundaries exist; current local and GitHub Actions validation covers structure, formatting, workspace compilation, tests, clippy, locked release build, dependency audit, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing. Production container validation, service installation, ARM cross-build validation, runtime deployment, validation runner execution, external fuzzing engine, backtest corpus execution, observability runtime, real dashboard hosting, outbound messaging integrations, external adapter submission, exchange-specific live adapters, live RPC adapters, wallet signer, transaction broadcasts, bridges, and live trading adapters remain unimplemented or unvalidated.
- Safety posture: Design-time controls, Phase 2 mode gates, Phase 3 policy checks, Phase 4 redacted audit/hash-chain primitives and SQLite WAL checkpoint persistence, Phase 5 read-only market-data/fee boundaries, Phase 6 paper-only simulation boundaries with local paper-report checkpoint persistence, Phase 7 CEX framework live-order denial, Phase 8 DEX/Web3 framework live-swap/RPC/broadcast denial, Phase 9 opportunity discovery, Phase 10 draft-only execution planning with policy preflight outcomes, Phase 11 execution-adapter boundary records with policy revalidation, Phase 12 command/notification boundaries with outbound-network denial, Phase 13 dashboard render records with server/public-exposure/live-control denial, Phase 14 observability records with metrics-endpoint/outbound-alert denial, Phase 15 validation plans with external-fuzzer/live-network/live-execution denial, Phase 16 packaging records with build/deployment/public-exposure/production-claim denial, Phase 17 hardening records with external-action/production-claim/live-funds/public-exposure denial, and Phase 18 handoff records with external-agent-execution/external-validation-claim/production-claim/live-funds/public-exposure denial exist; signer boundary, live connector integrations, live execution controls, production durability validation, external adapter submission, real communication delivery, real dashboard hosting/authentication, real metrics/exporter/alert runtime, actual property/fuzz/backtest execution, production container/systemd/ARM deployment validation, broader Phase 17 production hardening review, future-agent execution validation, and external validation are not yet implemented or externally validated.
- Current mode: Config/policy/audit/market-data/paper/CEX-framework/DEX-Web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/communications-CLI/dashboard/observability/testing/packaging/hardening/handoff-ready greenfield construction with live trading disabled until later custody, exchange-specific live connector, DEX RPC adapter, signer, live adapter submission, execution hardening, real communications adapters, dashboard hosting hardening, observability runtime hardening, validation runner execution, packaging/deployment validation, production hardening review, and external validation phases are complete.

## Mission

Build an ultra-lightweight, Rust-native, single-binary crypto arbitrage assistant/agent that can discover, evaluate, and execute crypto arbitrage opportunities across reputable centralized exchanges, decentralized exchanges, aggregators, chains, and market-data providers while enforcing deterministic safety, custody, policy, and audit controls.

The system is designed for a single primary operator first, with clean seams for future SaaS or invite-only multi-user expansion.

## Non-Negotiable Truths

1. Crypto arbitrage is not risk-free.
2. No software architecture can guarantee profit.
3. No autonomous agent may be trusted with live funds until the policy engine, key custody, execution sandboxing, transaction simulation, rollback paths, and live safety checks are implemented and externally validated.
4. The agent must never store secrets in Markdown files.
5. The agent must never send funds to unknown, unapproved, or LLM-generated destinations.
6. The LLM layer must never be able to directly sign transactions, mutate wallet policy, bypass risk controls, or disable audit logging.
7. Every live-funds action must be produced as a deterministic execution intent, checked by policy, journaled, simulated where possible, and only then executed by a constrained signer.

## Target Runtime Shape

The first-class runtime target is a lightweight Rust single binary capable of operating as:

- CLI application
- background daemon/service
- VPS service
- local machine process
- ARM-capable edge process where feasible
- optional embedded web dashboard host
- communication-channel connected assistant

The system should avoid heavyweight infrastructure for the default local/single-user path.

## Preferred Stack

### Core Language

- Rust

Rationale:

- Memory safety without a garbage collector
- High performance
- Strong type system
- Good fit for async network services and low-footprint daemons
- Good fit for deterministic policy enforcement

### Async Runtime

- Tokio

Rationale:

- Mature Rust async ecosystem
- Strong support for network clients, timers, channels, task orchestration, and graceful shutdown

### Local Storage

Default:

- SQLite with WAL mode for local persistence, audit metadata, opportunities, configuration state, and execution history

Optional later:

- SQLCipher or application-layer envelope encryption for sensitive persisted values
- PostgreSQL for future server/multi-user mode
- TimescaleDB or ClickHouse for high-volume historical market-data analytics if needed

### Internal Message Passing

Default:

- In-process Tokio channels

Optional later:

- NATS for distributed agent components
- Redis Streams only if external queueing becomes necessary
- Kafka out of scope for the lightweight default unless scale requirements change

### Configuration

Default files:

- `config.toml` for non-secret runtime configuration
- `.env` for local development secret references only
- encrypted local keystore for exchange API keys, wallet keys, and provider tokens

Forbidden:

- API keys in `.md` files
- API keys in committed `.toml` files
- seed phrases in any plaintext repository file
- wallet private keys in logs, prompts, chat transcripts, Markdown, or telemetry

## Source-Informed Exchange Seed Strategy

Initial connector priority should be source-informed and periodically refreshed. As of Phase 0 source review, public exchange-ranking sources show Binance, Coinbase Exchange, Kraken, OKX, and Upbit among major exchanges by volume/trust/ranking, with availability varying by jurisdiction. This list is not a compliance approval and must be revalidated before live use.

Phase 1+ candidate CEX connector tiers:

### CEX Tier 0: Framework Before Specific Exchanges

- Exchange trait definitions
- Market-data trait definitions
- Order-book normalization
- Fee model abstraction
- Balance model abstraction
- Execution intent model
- Sandbox/paper/live mode gate
- Per-exchange capability registry (Phase 7 framework boundary implemented)

### CEX Tier 1: High-Liquidity/Reputable Candidates Requiring Validation

- Binance / Binance.US where legally available
- Coinbase Exchange / Advanced Trade
- Kraken
- OKX where legally available
- Upbit where legally available
- Bybit where legally available

### CEX Tier 2: Additional Candidates After Risk Review

- KuCoin
- Gate.io
- Bitget
- MEXC
- Gemini
- Bitstamp

Each exchange must be individually enabled only after:

- API capability review
- jurisdiction review
- withdrawal policy review
- fee model validation
- order-type support validation
- rate-limit validation
- sandbox or paper-trading validation where available
- incident and reputation review
- terms-of-service review

## DEX and Web3 Candidate Strategy

DEX support is required, but must be implemented behind strict chain, router, token, and contract allowlists.

Candidate connector classes:

- EVM router connectors
- EVM aggregator connectors
- Solana DEX/aggregator connectors
- Stable-swap pool connectors
- Cross-chain bridge/route connectors only after elevated risk review

Candidate venues/providers requiring validation:

- Uniswap-style pools
- Curve-style pools
- Balancer-style pools
- PancakeSwap-style pools
- 0x-style aggregators
- 1inch-style aggregators
- Jupiter-style Solana routing

All Web3 transaction execution must require:

- chain allowlist
- router allowlist
- token allowlist
- spender allowlist
- transaction simulation where available
- slippage cap
- gas cap
- MEV/sandwich-risk checks where available
- allowance hygiene
- nonce management
- destination-address policy check
- deterministic signer boundary

## Primary Subsystems

### 1. Runtime and CLI Subsystem

Responsibilities:

- Start/stop daemon
- Load config
- Validate config
- Select mode
- Manage lifecycle
- Expose CLI commands
- Trigger graceful shutdown

Boundaries:

- Does not sign transactions
- Does not bypass policy
- Does not directly mutate secrets

### 2. Configuration Subsystem

Responsibilities:

- Parse `config.toml`
- Validate strategy profiles
- Validate mode gates
- Validate venue allowlists
- Validate risk limits
- Validate communication-channel configuration

### 3. Policy and Trust Contract Subsystem

Responsibilities:

- Represent deterministic execution intents before adapters act
- Enforce deny-by-default approvals
- Reject withdrawals, bridge routes, unknown destinations, and LLM-generated destinations in Phase 3
- Enforce mode, venue, asset, chain, risk, freshness, audit, signing, and kill-switch checks
- Keep live runtime unavailable by default until later execution, audit, custody, and validation phases explicitly enable it

Boundaries:

- Does not submit orders
- Does not sign transactions
- Does not load secrets
- Does not persist audit events yet
- Does not replace connector-specific validation

Phase 2 implementation status:

- `arb-core::config` now contains typed TOML models for runtime, risk, venues, secrets, communication, and audit settings.
- `arb-agent --config <path>` can load and validate a non-secret config file.
- `live-armed` mode requires an explicit live execution switch, exact operator acknowledgement, enabled kill switch, non-disabled secret backend, exchange secret reference, wallet signer reference, and withdrawals disabled.

Boundaries:

- No secret values in committed config
- Secrets referenced by key IDs, environment names, or encrypted keystore aliases
- Phase 2 mode gates are not the full Phase 3 policy engine

### 3. Secrets and Custody Subsystem

Responsibilities:

- Manage encrypted local secrets
- Load exchange API credentials
- Load wallet signer material only into constrained signing context
- Prevent accidental log/prompt exposure
- Support future external KMS/HSM providers

Phase 2 implementation status:

- `arb-core::secrets` now defines reference-only secret identifiers, redacted `SecretMaterial`, a `SecretProvider` trait, and an environment-variable provider skeleton.
- The encrypted keystore backend is not implemented yet; only the typed boundary exists.

Boundaries:

- LLM subsystem cannot access raw secrets
- Messaging subsystem cannot access raw secrets
- Strategy subsystem cannot access raw secrets
- Dashboard cannot display raw secrets

### 4. Policy and Trust Contract Subsystem

Responsibilities:

- Enforce non-bypassable runtime rules
- Validate every execution intent
- Enforce kill switch
- Enforce max exposure
- Enforce venue allowlists
- Enforce token/pair allowlists
- Enforce destination allowlists
- Enforce profit thresholds
- Enforce fee/slippage/gas caps
- Enforce drawdown limits
- Enforce mode restrictions
- Block withdrawals unless explicitly enabled by a signed local policy profile

Boundaries:

- Must be deterministic
- Must not depend on LLM judgment for safety-critical decisions
- Must deny by default

### 5. Strategy and Command Library Subsystem

Responsibilities:

- Define arbitrage strategy profiles
- Define behavior parameters
- Define capital allocation policies
- Define compounding behavior
- Define minimum ROI requirements
- Define latency/fee/risk preferences
- Define approval thresholds

Boundaries:

- Produces candidate intents only
- Cannot execute directly
- Cannot sign
- Cannot disable policy

### 6. Market Data Subsystem

Responsibilities:

- Collect quotes
- Collect order books
- Collect liquidity depth
- Collect fee schedules
- Collect gas estimates
- Normalize market symbols
- Maintain freshness windows
- Flag stale data

Phase 5 implementation status:

- `arb-core::market_data` defines normalized market pairs, price levels, top-of-book quotes, order-book snapshots, freshness classification, market-data requests, provider capabilities, and a read-only `MarketDataProvider` trait.
- `arb-core::fees` defines fee schedules, fee estimates, fee-adjusted edge calculation, liquidity-role classification, and a read-only `FeeProvider` trait.
- These models are local deterministic primitives only; they do not open sockets, call exchanges, call chain RPCs, load credentials, or execute trades.

Boundaries:

- Must track data provenance
- Must never treat stale data as executable
- Must not place orders, sign transactions, mutate balances, or load trading secrets
- Live provider validation, rate-limit handling, WebSocket reconnect behavior, and paid-data integrations are deferred


### 6A. Paper Connector Subsystem

Responsibilities:

- Provide deterministic in-memory market-data fixtures
- Provide static paper fee schedules
- Produce policy-gated paper execution reports
- Support future opportunity-engine and backtesting phases without live funds

Phase 6 implementation status:

- `arb-core::paper` defines `PaperMarketDataProvider`, `PaperFeeProvider`, `PaperExecutionAdapter`, and paper execution reports.
- Paper market data implements the Phase 5 `MarketDataProvider` trait using caller-supplied normalized order-book snapshots.
- Paper fees implement the Phase 5 `FeeProvider` trait using static schedules.
- Paper execution requires `PolicyEngine` approval and rejects non-paper scopes before producing a report.
- `persist_paper_execution_report_checkpoint` stores the latest deterministic paper report through the typed local `StateStore` boundary and can use `SqliteWalStateStore` for non-secret local checkpoint persistence.

Boundaries:

- Does not call live exchanges, DEXes, RPC endpoints, or paid data providers
- Does not load secrets
- Does not sign transactions
- Does not mutate real balances
- Does not model production-grade depth, partial fills, latency, or settlement yet
- Must call policy before paper execution reports are produced
- Does not replace future mandatory audit-before-action, fail-closed state writes, balance reconciliation, or production durability validation

### 6B. CEX Connector Framework Subsystem

Responsibilities:

- Define centralized-exchange venue profiles
- Define connector capabilities
- Register known CEX profiles without secrets
- Normalize CEX order requests before adapter work
- Validate order request shape, venue kind, order type, time-in-force, and capability support
- Convert CEX order requests into policy `ExecutionIntent` records
- Provide read-only and trading connector trait boundaries for future exchange-specific adapters

Phase 7 implementation status:

- `arb-core::cex` defines `CexVenueProfile`, `CexConnectorCapabilities`, `CexConnectorRegistry`, `CexOrderRequest`, CEX order enums, `CexPolicyGate`, and connector traits.
- `CexPolicyGate` validates paper/sandbox-scoped CEX order requests through the Phase 3 `PolicyEngine`.
- Live CEX order requests are explicitly denied in Phase 7.

Boundaries:

- Does not call exchange REST APIs
- Does not open WebSocket streams
- Does not load credentials
- Does not read live balances
- Does not submit or cancel live orders
- Does not mutate balances
- Does not validate terms, jurisdiction, rate limits, or fees externally
- Future live adapters must integrate audit-before-action, state checkpointing, rate-limit control, and credential-scope validation before any production use

### 6C. DEX/Web3 Connector Framework Subsystem

Responsibilities:

- Define Web3 chain profiles without RPC URLs or provider tokens
- Define token metadata profiles without secret material
- Define DEX/router profiles and capability declarations
- Register chain/router/token profiles without live adapters
- Normalize DEX swap quote requests and responses before adapter work
- Define local transaction simulation request and response boundaries without raw calldata, signing, or broadcast
- Convert DEX swap quote requests into policy `ExecutionIntent` records
- Provide quote and simulation connector trait boundaries for future DEX/RPC adapters

Phase 8 implementation status:

- `arb-core::dex` defines `Web3ChainProfile`, `DexTokenProfile`, `DexRouterProfile`, `DexRouterCapabilities`, `DexConnectorRegistry`, `DexSwapQuoteRequest`, `DexSwapQuoteResponse`, `Web3TransactionSimulationRequest`, `Web3TransactionSimulationResponse`, `DexPolicyGate`, and connector traits.
- `DexPolicyGate` validates paper/simulation-scoped DEX swap quote requests through the Phase 3 `PolicyEngine`.
- Live DEX swaps, live RPC transaction simulation, signing, transaction broadcast, and bridges are explicitly unavailable in Phase 8.

Boundaries:

- Does not call chain RPC endpoints
- Does not call DEX/router/aggregator APIs
- Does not load wallet keys or signer secrets
- Does not build arbitrary contract calls from LLM output
- Does not sign transactions
- Does not broadcast transactions
- Does not execute bridges or withdrawals
- Does not validate router contracts, token metadata, slippage, MEV risk, gas, or protocol terms externally
- Future live adapters must integrate audit-before-action, state checkpointing, simulation, signer policy, spender/approval hygiene, and external protocol review before any production use

### 7. Opportunity Engine

Responsibilities:

- Detect candidate arbitrage opportunities from already-normalized market data
- Estimate net profit after supplied trading/network fees and top-of-book spread
- Rank opportunities deterministically
- Produce deterministic opportunity records
- Preserve model boundaries for CEX/CEX, DEX/DEX, CEX/DEX, and triangular route classes
- Forward only validated candidates to the future execution-planning layer

Phase 9 implementation status:

- `arb-core::opportunity` defines `OpportunityDiscoveryConfig`, `OpportunityDiscoveryRequest`, `OpportunityCandidate`, `OpportunityLeg`, `OpportunityScore`, `OpportunityRouteKind`, `OpportunityEngine`, and `DeterministicOpportunityEngine`.
- The deterministic engine consumes supplied `NormalizedQuote` and `FeeSchedule` values only.
- Market-data freshness failures are fail-closed.
- Ranking is deterministic and fee-aware.
- Triangular arbitrage has a typed route boundary, but full triangular path search is deferred.

Boundaries:

- Does not execute trades
- Does not produce execution intents
- Does not access secrets
- Does not call CEX APIs
- Does not call DEX/router/RPC APIs
- Does not sign or broadcast transactions
- Does not withdraw or bridge funds
- Does not bypass policy

### 8. Execution Planner

Responsibilities:

- Convert validated opportunities into draft execution intents
- Select draft order/swap intent kinds
- Plan deterministic per-leg sequencing
- Plan fail-safe behavior and reconciliation boundaries
- Capture policy preflight approval/denial outcomes
- Estimate draft execution risk metadata

Phase 10 implementation status:

- `arb-core::planner` defines `ExecutionPlannerConfig`, `ExecutionPlannerRequest`, `ExecutionPlanDraft`, `ExecutionPlanStep`, `ExecutionPlanFailureMode`, `PlannerPolicyOutcome`, `ExecutionPlanner`, and `DeterministicExecutionPlanner`.
- The deterministic planner converts each validated `OpportunityCandidate` leg into one draft `ExecutionIntent`.
- Live planner scope is rejected fail-closed.
- Every generated draft intent is evaluated through `PolicyEngine` and stored as a redacted policy outcome.
- `adapter_submission_enabled` is always false.

Boundaries:

- Does not submit to adapters
- Does not place orders
- Does not sign transactions
- Does not broadcast transactions
- Does not withdraw or bridge funds
- Does not call CEX APIs
- Does not call DEX/router/RPC APIs
- Does not provide durable audit/state persistence yet
- Must write to audit journal before any future live execution adapter receives a plan

### 9. Execution Adapter Subsystem

Future responsibilities:

- Submit exchange orders after explicit future enablement
- Submit DEX transactions after explicit future enablement
- Manage nonces after signer/RPC boundaries exist
- Manage confirmations after RPC/provider integrations exist
- Track fills
- Reconcile balances
- Report failures

Phase 11 implementation status:

- `arb-core::execution_adapter` defines `ExecutionAdapterConfig`, `ExecutionAdapterRequest`, `ExecutionAdapterRunRecord`, `ExecutionAdapterAttempt`, `ExecutionFillRecord`, `ExecutionReconciliationRecord`, `ExecutionAdapter`, and `DeterministicExecutionAdapterBoundary`.
- The deterministic adapter boundary consumes `ExecutionPlanDraft` records and revalidates every intent through `PolicyEngine`.
- Paper fills and reconciliation records are deterministic model records only.
- `external_submission_enabled` and every per-attempt `submitted_to_external_adapter` flag remain false.
- Live scope and external adapter submission are rejected fail-closed.

Boundaries:

- Cannot operate outside selected mode
- Cannot execute blocked intents
- Cannot override policy denial
- Does not call CEX APIs
- Does not call DEX/router/RPC APIs
- Does not sign or broadcast transactions
- Does not withdraw or bridge funds
- Does not submit external orders or transactions
- Does not provide durable audit/state persistence yet

### 10. Audit Journal Subsystem

Responsibilities:

- Append-only event trail
- Config snapshot hashes
- Strategy profile hashes
- Opportunity records
- Intent records
- Policy decision records
- Execution records
- Balance reconciliation records
- Failure records
- Operator override records

Phase 4 implementation status:

- `arb-core::audit` defines typed audit events, redacted metadata values, validation errors, append-only JSONL records, and a hash-chained local audit journal.
- Existing records are replayed on open and rejected if sequence, previous-hash, format-version, redaction, or record-hash checks fail.
- `arb-core::state` defines a state-store trait and checkpoint model with an in-memory non-production implementation for tests and early wiring only.

Boundaries:

- Must avoid secret leakage
- Must be efficient and non-blocking where possible
- Must support durable flush semantics for live-fund operations
- Phase 4 JSONL audit is not yet SQLite WAL storage, externally crash-tested durability, concurrent append validation, or log shipping
- State persistence is not production-ready until the SQLite WAL-backed implementation is crash/concurrency/filesystem validated and wired into runtime lifecycle paths

### 11. Communications Subsystem

Responsibilities:

- Local CLI command parsing and routing boundaries
- Operator notification models
- Non-secret channel profile models
- Secret-safe message validation, redaction, and truncation
- Local dispatch records for future notifications
- Future Telegram, Discord, Matrix, email, Slack, PagerDuty, Signal, iMessage, webhook, or SMS adapters after explicit validation

Phase 12 implementation status:

- `arb-core::communications` defines `CommunicationBoundaryConfig`, `NotificationChannelProfile`, `OperatorCommand`, `OperatorCommandRouter`, `DeterministicOperatorCommandRouter`, `OperatorNotification`, `NotificationPublisher`, `DeterministicNotificationBoundary`, and `NotificationDispatchRecord`.
- The deterministic command router accepts local status/help/config/safety/roadmap/plan-only commands as typed boundaries.
- The deterministic command router rejects live execution, withdrawals, bridges, signing, and broadcast command requests.
- The deterministic notification boundary creates local dispatch records only and preserves `outbound_network_used = false`.
- Secret-like command and notification text is rejected or redacted before local dispatch records are produced.

Boundaries:

- No raw secrets
- No arbitrary command execution
- No policy bypass
- No live execution commands
- No withdrawals, bridges, signing, or broadcasts
- No real messaging platform tokens
- No outbound HTTP, SMTP, WebSocket, bot, webhook, or platform API calls
- Commands must map to typed, allowlisted command handlers

### 12. LLM Assistant Subsystem

Responsibilities:

- Explain opportunities
- Summarize state
- Suggest strategy-profile changes
- Draft operator-readable reports
- Assist with configuration reasoning

Boundaries:

- Cannot access private keys
- Cannot sign
- Cannot directly execute trades
- Cannot mutate policy without deterministic typed configuration update flow
- Cannot decide that an unknown destination is safe
- Cannot override kill switch

### 13. Optional Embedded Dashboard

Responsibilities:

- Local dashboard snapshot models
- Read-only runtime status by default
- Opportunity, planner, execution-adapter, communications, audit/state, and gap views
- Deterministic local render records
- Secret-safe display redaction
- Explicit mode and safety indicators
- Future lightweight local web interface after explicit hosting scope

Phase 13 implementation status:

- `arb-core::dashboard` defines `DashboardBoundaryConfig`, `DashboardServerBinding`, `DashboardSnapshot`, `DashboardPanel`, `DashboardPanelItem`, `DashboardRenderer`, `DeterministicDashboardRenderer`, and `DashboardRenderRecord`.
- The deterministic renderer creates local in-process render records only.
- The boundary rejects HTTP server startup, public network exposure, non-loopback bind hosts, live controls, and secret rendering.
- Render records preserve `server_started = false`, `public_network_exposed = false`, and `live_controls_enabled = false`.
- Secret-like dashboard text is redacted before render records are produced.

Boundaries:

- No server startup in Phase 13
- No public web exposure
- Localhost-only model assumptions for future hosting
- Auth required before any future network exposure
- No secret display
- No direct raw command injection
- No live execution controls
- No withdrawals, bridges, signing, broadcasts, or adapter submission
- No policy bypass

### 14. Observability Subsystem

Responsibilities:

- Structured logs
- Metric sample models
- Health checks
- Runtime state reporting
- Panic/failure capture models
- Alert-routing boundaries
- Operator runbooks

Phase 14 implementation status:

- `arb-core::observability` defines `ObservabilityBoundaryConfig`, `ObservabilityEndpointBinding`, `HealthStatus`, `ComponentHealthStatus`, `StructuredLogEvent`, `MetricSample`, `Runbook`, `ObservabilitySnapshot`, `ObservabilityCollector`, `DeterministicObservabilityCollector`, and `ObservabilityRecord`.
- The deterministic collector creates local in-process observability records only.
- The boundary rejects metrics endpoint startup, public network exposure, non-loopback bind hosts, outbound alert delivery, and secret observability.
- Observability records preserve `metrics_endpoint_started = false`, `public_network_exposed = false`, and `outbound_alerts_sent = false`.
- Secret-like health/log/metric/runbook text is redacted before collection records are produced.

Default future approach:

- `tracing` for structured logs after explicit runtime integration scope
- optional OpenTelemetry exporter after authentication, redaction, and exporter review
- optional Prometheus-compatible metrics endpoint after loopback/authentication/rate-limit validation
- local runbooks from day one

Boundaries:

- No metrics endpoint startup in Phase 14
- No public telemetry exposure
- No log shipping, SIEM delivery, OpenTelemetry exporter, Prometheus endpoint, or outbound alert delivery
- No secret telemetry
- No raw exchange keys, wallet keys, provider tokens, mnemonics, or authorization headers in health/log/metric/runbook records
- No live execution controls
- No withdrawals, bridges, signing, broadcasts, or adapter submission
- No policy bypass


### 15. Testing, Fuzzing, and Backtesting Subsystem

Responsibilities:

- Deterministic validation planning
- Test-case metadata boundaries
- Fixture metadata boundaries
- Fuzz corpus and seed metadata boundaries
- Backtest dataset and scenario metadata boundaries
- Local validation run records
- Validation safety gates for live-network, external-fuzzer, live-execution, signing, broadcast, and secret-fixture denial

Phase 15 implementation status:

- `arb-core::testing` defines `ValidationHarnessConfig`, `ValidationTestCase`, `ValidationFixtureRecord`, `FuzzSeedRecord`, `FuzzCorpusDefinition`, `BacktestDatasetDefinition`, `BacktestScenarioDefinition`, `ValidationPlan`, `ValidationHarness`, `DeterministicValidationHarness`, and `ValidationRunRecord`.
- The deterministic harness validates plans and returns local records only.
- The boundary rejects external fuzzer invocation, live network tests, live execution tests, credential-bearing fixtures, live order submission, signing, and transaction broadcasts.
- Validation records preserve `external_fuzzer_invoked = false`, `live_network_used = false`, `live_execution_submitted = false`, and `signing_or_broadcast_performed = false`.
- Secret-like operator labels are redacted before validation records are produced.

Default future approach:

- `cargo test` for unit/integration tests after each workspace change
- property tests after explicit dependency and corpus design
- fuzzing with reviewed local harnesses after explicit future scope
- deterministic fixture replay and backtesting against curated local corpora
- CI gating after actual runner validation

Boundaries:

- No external fuzzer process invocation in Phase 15
- No live network tests
- No live exchange/RPC calls
- No live orders, swaps, withdrawals, bridges, signing, or broadcasts
- No secret-bearing fixtures
- No policy bypass
- No production validation claim until commands and scenarios actually run


### 16. Packaging and Deployment Subsystem

Responsibilities:

- Deterministic packaging and deployment planning
- Package target metadata for binary, container, systemd, ARM, deployment-document, and CI release-gate artifacts
- Service hardening metadata
- Release-gate and rollback-step records
- Fail-closed denial of public exposure, live trading, embedded secrets, build claims, deployment claims, and production claims

Phase 16 implementation status:

- `arb-core::packaging` defines `PackagingBoundaryConfig`, `PackageTargetPlan`, `ServiceHardeningProfile`, `ReleaseGate`, `RollbackStep`, `DeploymentPackagePlan`, `DeploymentPackageRequest`, `DeploymentPackageRecord`, `PackagingDeploymentPlanner`, and `DeterministicPackagingDeploymentPlanner`.
- The deterministic planner validates local package/deployment plans and returns records only.
- The boundary rejects public network exposure, embedded secret material, live trading deployment, build claims, deployment claims, and production deployment claims.
- Package records preserve `build_performed = false`, `deployment_performed = false`, `public_network_exposed = false`, `live_trading_enabled = false`, `secret_material_embedded = false`, and `production_deployment_claimed = false`.
- Example container, systemd, ARM, and deployment notes exist as templates only. Current CI builds and scans the example container image, but that does not prove production container, service, ARM, runtime deployment, or rollback readiness.

Default future approach:

- release builds through local or CI validation before release review
- container builds after approved local or CI runtime validation
- systemd service validation on Linux targets
- ARM cross-build validation on actual target class or verified emulator
- rollback drills before unattended operation

Boundaries:

- No production container build execution in Phase 16
- No service installation or daemon start
- No public dashboard, metrics, command, or control exposure
- No embedded secrets in artifacts
- No live trading enablement
- No production deployment claim until external validation actually runs

## Operating Modes

### Mode 0: Observe

- Collects data only
- No paper trades
- No live trades
- No wallet/API signing

### Mode 1: Simulate

- Uses static/sample data
- Tests opportunity logic
- No external execution

### Mode 2: Paper

- Uses live or recorded market data
- Simulates decisions and fills
- No live orders

### Mode 3: Shadow

- Tracks real opportunities against live accounts in read-only mode
- No live execution
- Balance read-only only

### Mode 4: Assisted Live

- Creates live execution intents
- Requires human approval before submission
- Strongly preferred first live-funds mode

### Mode 5: Guarded Live

- Autonomous live execution under small-capital, strict policy limits
- No withdrawals by default
- Requires passing production-readiness checklist

### Mode 6: Autonomous Live

- Fully autonomous execution under signed policy profiles
- Requires external validation, security review, and live burn-in
- Still cannot bypass policy, allowlists, audit, kill switch, or signer constraints

## Trust Contract

The system must enforce the following pact at the code level:

1. Funds may only move through approved venues, chains, contracts, and wallet addresses.
2. Funds may never be sent to a destination created, guessed, suggested, or inferred by an LLM.
3. Private keys and API secrets may never be logged, displayed, committed, or exposed to prompts.
4. Every execution intent must be checked by policy before execution.
5. Every policy decision must be journaled.
6. Every live execution must be reconcilable after completion or failure.
7. Unknown state must halt or degrade to a safer mode.
8. Stale data must not be executable.
9. Policy denial must be final unless the operator explicitly changes signed/local policy config.
10. Kill switch must take priority over all strategies.
11. Withdrawal capability must be disabled by default and must require explicit strategy-profile capability, address allowlist, per-period limit, and operator confirmation flow until externally validated.
12. The agent must prefer missing an opportunity over violating policy.

## Baseline Risk Controls

Initial mandatory controls:

- global kill switch
- mode gate
- max capital at risk per opportunity
- max daily realized loss
- max daily drawdown
- max venue exposure
- max chain exposure
- max token exposure
- max single transaction value
- max slippage
- max gas fee
- max stale quote age
- minimum net profit after all fees
- minimum confidence score
- minimum liquidity depth
- exchange allowlist
- chain allowlist
- token allowlist
- contract/router allowlist
- address allowlist
- no unknown withdrawal destinations
- no unconstrained approvals
- approval revocation/expiry strategy
- transfer cooldowns
- balance reconciliation after execution
- circuit breaker on repeated failures
- rate-limit aware execution
- durable audit before live execution

## Strategy Profile Parameter Library: Initial Design

Strategy profiles should support typed parameters such as:

- `mode`
- `capital.base_asset`
- `capital.max_total_deployed`
- `capital.max_per_opportunity`
- `capital.reserve_minimum`
- `capital.compound_enabled`
- `capital.compound_rate`
- `risk.max_daily_loss`
- `risk.max_daily_drawdown_pct`
- `risk.max_open_exposure_pct`
- `risk.max_venue_exposure_pct`
- `risk.max_chain_exposure_pct`
- `risk.max_token_exposure_pct`
- `risk.max_single_tx_value`
- `risk.consecutive_failure_limit`
- `opportunity.min_net_profit_abs`
- `opportunity.min_net_profit_pct`
- `opportunity.min_roi_after_fees_pct`
- `opportunity.max_quote_age_ms`
- `opportunity.min_liquidity_depth`
- `opportunity.min_confidence_score`
- `execution.max_slippage_bps`
- `execution.max_gas_native`
- `execution.max_gas_usd`
- `execution.order_timeout_ms`
- `execution.cancel_on_partial_fill`
- `execution.allow_market_orders`
- `execution.allow_limit_orders`
- `execution.allow_ioc`
- `execution.allow_fok`
- `execution.allow_flashloans`
- `execution.allow_bridges`
- `execution.allow_withdrawals`
- `execution.allowed_destination_addresses`
- `venues.allowed_exchanges`
- `venues.allowed_chains`
- `venues.allowed_routers`
- `venues.allowed_tokens`
- `alerts.notify_on_opportunity`
- `alerts.notify_on_execution`
- `alerts.notify_on_policy_denial`
- `alerts.notify_on_loss`
- `alerts.notify_on_kill_switch`

## Security Architecture Principles

- Deny by default
- Least privilege
- No plaintext secrets in repository
- No secrets in Markdown
- No LLM access to secrets
- No LLM access to signer
- Explicit allowlists
- Typed command handling
- Deterministic policy checks
- Immutable audit trail intent
- Safe mode degradation
- Crash-safe state handling
- Rollback-safe patches
- Dependency minimization
- Supply-chain audit before production

## Compliance and Legal Posture

Current declared requirement:

- Internal audit only
- No known external compliance framework selected

Architecture requirement:

- Terms-of-service and jurisdiction reviews are mandatory before live exchange integrations.
- Tax, legal, regulatory, and accounting review remain external human tasks.
- Production releases must not imply compliance certification unless completed externally.

## Rollback Strategy

Every phase must preserve rollback safety by:

- isolating subsystem patches
- avoiding broad rewrites
- keeping deterministic migrations
- documenting config changes
- preserving mode defaults that cannot accidentally go live
- adding tests before enabling live behaviors
- maintaining gap tracker entries for unvalidated production tasks

## Current Hard Blockers to Production

- Rust/Cargo validation must be rerun for every changed workspace state and does not imply production readiness
- No encrypted secret manager or custody backend
- No wallet signer boundary
- No exchange-specific live CEX adapters
- No live DEX/Web3 RPC adapters, signer, transaction simulation integrations, or broadcasts
- No live execution adapter submissions
- SQLite WAL state store exists for local checkpoints, but production durability validation and runtime lifecycle wiring are missing
- Audit journal is not crash/concurrency/filesystem validated
- CEX framework is not connected to real REST/WebSocket APIs or sandboxes
- DEX/Web3 framework is not connected to real RPC, router, aggregator, signer, simulation, or broadcast adapters
- Opportunity engine has current workspace Rust validation evidence but does not yet model inventory, transfer latency, full triangular path discovery, or depth-aware slippage
- Execution planner and execution-adapter framework have current workspace Rust validation evidence but do not provide durable audit/state lifecycle integration
- No runtime deployment validation
- No container, systemd, ARM, or rollback-drill validation
- CI/CD execution validation exists for structure, Rust validation, locked release build, dependency audit, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing only
- No external security review
- No live exchange API keys
- No production deployment environment

## Phase 18 Agentic Handoff Boundary

The agentic handoff subsystem provides deterministic package records, continuation prompts, governance checklists, external validation checklists, and future-agent instructions only. It does not execute external agents, call coding-agent APIs, deploy infrastructure, approve production readiness, approve public exposure, approve live funds, or store credentials. Handoff records may reference current local/CI validation evidence, but they preserve unresolved gaps and live-funds blockers so future agents cannot silently erase deferred validation work.
