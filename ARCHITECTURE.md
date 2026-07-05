# ARCHITECTURE.md

## Project Name

ArbyClaw

## Architecture Status

- Status: Phase 103 market-data live provider boundary gate is implemented after Phase 102 structure manifest consistency gate and Phase 101 CI handoff aggregate container scan preparation, Phase 100 handoff candidate full local surface gate enforcement, and earlier local connector/runtime validation phases. The current architecture includes a typed local live market-data provider boundary review, typed local DEX/Web3 and CEX live-adapter boundary reviews; typed local production-runtime preflight accounting; direct deployment evidence bundle/checklist enforcement for lifecycle plan, deployment-host runtime plan, retention preflight, service-manager rehearsal, rollback/incident drill plans, deployment disk-full, retention, permission-denial, audit/SQLite, backup/restore, graceful-shutdown, SQLite schema migration, rollback, incident-response, failure-capture, response-drill, config redaction, log redaction, static hardening/config smoke, and observability metrics runtime wrapper evidence; deployment-host wrapper and aggregate deployment-runtime enforcement for static deployment hardening/config smoke validation through the real `arb-agent --config` path; deployment-host wrapper and aggregate deployment-runtime enforcement for typed local observability metrics runtime validation; typed local observability metrics runtime, dashboard loopback runtime, communications outbox, deployment log/config redaction, SQLite WAL schema migration, runtime config reload, fee schedule reconciliation, market-data bad-data/provider reconciliation/live-provider boundary accounting, runtime load profile, deployment transcript/rehearsal, packaging, hardening, and handoff validation boundaries. The packaging/deployment aggregate gate directly requires the production-intent container validator, including Docker validation completion, hardened read-only/no-network smoke, dropped capabilities, no-new-privileges, and service-installation non-claims propagated through hardening and handoff aggregates. The handoff-candidate aggregate gate composes execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening-core, deployment-evidence checklist, and local handoff audit validation, and CI now pulls the Dockerized Trivy scanner image before running that strict aggregate on fresh runners. The structure validator now enforces required-file byte/hash consistency against the generated structure manifest so required governance, source, script, workflow, and phase files cannot silently drift from `STRUCTURE_MANIFEST.md`. Current workspace Rust/CI validation evidence exists, while real production load testing, daemon-hosted observability runtime/exporter/alert operation, real persistent dashboard hosting, real outbound communications delivery, live exchange/RPC adapter implementation, live REST/WebSocket market-data providers, provider-backed market-data session/latency/rate-limit/outage/bad-data validation, real provider-backed nonce retrieval, provider-backed validation, external fee/account-tier/gas/withdrawal validation, broader external/deployment scenario-corpus validation, production image publishing, service installation, actual deployment-host config reload/start/stop/restart validation, actual deployment-host backup/restore execution under service lifecycle and load, actual deployment-host graceful-shutdown execution, deployment-host audit validation under service lifecycle, deployment-host schema migration execution under service lifecycle, deployment-host config loading under service lifecycle, deployment-host log/audit redaction under service lifecycle, real deployment-host runtime-write permission-denial execution, physical disk-full evidence, deployment-host retention/rotation execution evidence, operator-controlled service-manager lifecycle execution evidence, actual rollback execution evidence, actual incident-response execution evidence, daemon failure-capture execution evidence, external sandbox/live calibration evidence, deployment-host durability validation, external agent execution, and real production hardening validation remain deferred.
- Production readiness: Not approved; local validation coverage has expanded, but there is still no live-funds approval, deployment approval, production audit durability approval, or externally executed production validation.
- Phase 87 update: Deployment-host runtime reporting now composes `arb-agent validate-observability-metrics-runtime` through `--run-observability-metrics-runtime`, and the aggregate deployment-runtime gate now requires that wrapper report. The aggregate reports 36 local runtime/deployment components and 23 nested runtime components while preserving no service actions, no external calls, no live execution, no secret loading, no telemetry export, no outbound alert delivery, no public exposure, and no production-readiness claim.
- Phase 88 update: The deployment evidence bundle now includes `deployment-host-observability-metrics-runtime` as a direct bounded local component, reports 20 bundle components when validated, and refreshes only a scoped workspace under `target/deployment-evidence-bundle` while preserving no service actions, no external calls, no live execution, no secret loading, no telemetry export, no outbound alert delivery, no public exposure, and no production-readiness claim.
- Phase 89 update: The deployment evidence checklist now fails closed if required bundle components are missing, currently requiring `deployment-host-observability-metrics-runtime`, and reports required/missing required component names without embedding artifact contents or claiming production readiness.
- Phase 90 update: The deployment-host runtime wrapper now composes static deployment hardening/config smoke validation through `--run-deployment-static-hardening`, and the aggregate deployment-runtime gate now requires it. The aggregate reports 37 local runtime/deployment components and 24 nested runtime components while preserving no service actions, no external calls, no live execution, no secret loading, no network listeners, no public exposure, and no production-readiness claim.
- Phase 91 update: The deployment evidence bundle now includes `deployment-host-static-hardening-config-smoke` as a direct bounded local component, raising the bundle to 21 components, and the deployment evidence checklist now requires both that component and `deployment-host-observability-metrics-runtime` before reporting zero missing required components.
- Phase 92 update: The deployment evidence bundle now includes `deployment-host-config-redaction` and `deployment-host-log-redaction` as direct bounded local components, raising the bundle to 23 components, and the deployment evidence checklist now requires four bundle components before reporting zero missing required components.
- Phase 93 update: The deployment evidence bundle now includes `deployment-disk-full-transcript`, `deployment-retention-transcript`, and `deployment-permission-transcript` as direct bounded local components, raising the bundle to 26 components, and the deployment evidence checklist now requires seven bundle components before reporting zero missing required components.
- Phase 94 update: The typed local production-runtime preflight now reports and blocks on deployment-host backup/restore, graceful shutdown, audit/SQLite recovery, SQLite schema migration, daemon failure-capture, and concurrent lifecycle execution evidence categories, and the deployment-runtime aggregate gate requires those local evidence flags to remain unavailable until real deployment-host execution evidence exists.
- Phase 95 update: The CEX framework now exposes `CexLiveAdapterBoundaryReviewRequest` / `CexLiveAdapterBoundaryReviewReport` plus `arb-agent validate-cex-live-adapter-boundary`, and the connector scenario aggregate gate includes the CEX live-adapter boundary while preserving no REST calls, WebSocket connections, credential loading, external submission, live execution, or production-readiness claim.
- Phase 96 update: The DEX/Web3 framework now exposes `DexLiveAdapterBoundaryReviewRequest` / `DexLiveAdapterBoundaryReviewReport` plus `arb-agent validate-dex-live-adapter-boundary`, and the connector scenario aggregate gate now includes 21 local components with DEX/Web3 live-adapter boundary enforcement while preserving no HTTP/RPC calls, credential loading, signing, broadcasts, bridges, external submission, live execution, or production-readiness claim.
- Phase 97 update: The deployment evidence checklist now requires existing local bundle components for deployment audit/SQLite recovery, backup/restore, graceful shutdown, SQLite schema migration, rollback execution, incident-response execution, failure capture, and response-drill rehearsal, in addition to the prior required components, while preserving reference-only, non-mutating behavior.
- Phase 98 update: The deployment evidence checklist now also requires existing local systemd lifecycle plan, deployment-host runtime plan, deployment-host retention preflight, rollback-drill plan, incident-response drill plan, and service-manager lifecycle rehearsal components before reporting zero missing required components.
- Phase 99 update: The packaging/deployment aggregate gate now requires `scripts/validate_production_container.py --json` as a direct component and validates Docker completion, hardened read-only/no-network smoke, dropped capabilities, no-new-privileges, and explicit non-claims. The hardening-core and handoff-candidate aggregates now propagate `service_installed: false` so service installation cannot be implied by nested packaging evidence.
- Phase 100 update: The handoff-candidate aggregate gate now requires execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening-core, deployment-evidence checklist, and local handoff audit validation before reporting a candidate pass, and it carries the nested software-surface external evidence summary without enabling external execution or readiness claims.
- Phase 101 update: GitHub Actions now pulls `aquasec/trivy:latest` immediately before the strict handoff-candidate aggregate so `scripts/validate_production_container.py` can keep using Dockerized Trivy scan containers with `--pull never` without depending on a fresh runner's image cache.
- Phase 102 update: `scripts/validate_structure.py` now parses `STRUCTURE_MANIFEST.md` and fails closed when required files are absent from the manifest or their byte count/SHA-256 digest is stale. `scripts/generate_structure_manifest.py` now emits a current generated-inventory context instead of the stale Phase 55 paragraph, and the manifest has been refreshed to include current required phase files through Phase 102.
- Phase 103 update: The market-data framework now exposes `MarketDataLiveProviderBoundaryReviewRequest` / `MarketDataLiveProviderBoundaryReviewReport` plus `arb-agent validate-market-data-live-provider-boundary`, and the connector scenario aggregate gate now includes 22 local components with market-data live-provider boundary enforcement while preserving no live provider calls, WebSocket connections, credential loading, external submission, live execution, or production-readiness claim.
- Phase 86 update: Local observability metrics runtime validation now composes a bounded multi-scrape loopback `/metrics` listener, authenticated local scrape checks, metric-line consistency checks, clean shutdown, audit replay, SQLite checkpoint recovery, and operator-surface aggregate enforcement. This remains local-only and does not start a daemon-hosted persistent metrics endpoint, expose public networks, export telemetry, ship logs, deliver alerts, perform service-manager actions, make external calls, execute live trades, sign, broadcast, or claim production readiness.
- Phase 81 update: Local SQLite WAL schema migration validation now composes fresh legacy fixture creation, actual `SqliteWalStateStore` migration, checkpoint preservation, future-version fail-closed rejection, deployment-host wrapper reporting, and aggregate deployment-runtime enforcement. This remains local-only and does not perform deployment-host schema migration execution, service-manager reloads, service start/stop/restart, deployment-host mutation, secret loading, external submission, live execution, signing, broadcasts, or production readiness.
- Implementation status: Minimal Rust workspace, typed config, typed local strategy profile constraints, local persistent destination allowlist records with ownership-evidence reference reviews, redacted secret-reference abstractions, local fail-closed signer request records, signer secret-scope reviews, signer runtime isolation reviews, signer authorization envelope reviews with audit/state checkpoints, local Web3 nonce reservation reviews with audit/state checkpoints, local Web3 unsigned payload reviews with audit/state checkpoints, local Web3 pre-sign safety reviews with audit/state checkpoints, and local Web3 unsigned transaction construction records with audit/state checkpoints, local Web3 provider nonce reconciliation records with audit/state checkpoints, local Web3 raw transaction serialization review records with audit/state checkpoints, local Web3 broadcast adapter control review records with audit/state checkpoints, local Web3 sandbox/live discrepancy calibration records with audit/state checkpoints, CLI config loading, isolated policy engine, append-only audit journal primitives with local lock/sync appends, audit durability validation probes including simulated disk-full fail-closed behavior, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning, state-store trait boundary with SQLite WAL checkpoint persistence, local schema v1 migration and future-version rejection, local durability validation, process-level crash/restart recovery tests, and local runtime state-permission fail-closed validation, normalized market-data models, freshness classification, local market-data provider preflight records, fee models, local fee-schedule verification records, provider trait boundaries, local provider-to-opportunity ingestion for non-REST/non-WebSocket market-data and fee providers, deterministic paper connectors with local paper balance ledgering, realistic local fill simulation, venue matching profiles, adverse-selection modeling, reference-only calibration records, paper replay validation, local historical-fixture backtest execution, and local paper report/ledger-mutation audit journal records, CEX connector framework types/traits with a deterministic local CEX adapter, local Binance/Coinbase/Kraken-shaped fixture matching validation, mocked Binance/Coinbase/Kraken order-book transcript parsing, local Binance/Coinbase/Kraken balance snapshot transcript parsing, local rate-limit validation, local credential/API-scope review, and local validation audit/state checkpoints, DEX/Web3 framework types/traits with a deterministic local DEX adapter, local validation audit/state checkpoints, local EVM receipt/Solana signature-status transaction lifecycle transcript parsing with nonce/confirmation accounting, and local protocol risk review for chain/pair scope allowlists, router/spender contract hygiene, gas/slippage caps, MEV controls, token metadata/contract/decimals, and terms/jurisdiction/incident review, deterministic opportunity-engine types/traits with local replay/false-positive reports, local replay latency/throughput review enforcement, local strategy replay/profitability-tuning validation over the historical opportunity corpus, draft-only execution-planner types/traits with local plan-draft audit/checkpoint persistence and per-intent policy-outcome audit records, a local execution-path aggregate gate over planner handoff, strategy constraints, planner audit, policy/destination audit, adapter audit, signer controls, and Web3 non-broadcast controls, execution-adapter boundary records/traits with local adapter-run checkpoint persistence, local fail-closed runtime lifecycle wiring for audit/state/adapter sequencing, local concurrent runtime lifecycle access checks, local graceful-shutdown audit/state checkpointing, local runtime audit/SQLite backup-restore validation, local runtime restart recovery summaries with recovered connector-lifecycle and opportunity-trace summary accounting, CLI-visible operator-review dispositions, and incomplete-checkpoint fail-closed coverage, local deployment-like runtime smoke validation with concurrent lifecycle reporting, blocked audit/state preflight checks, runtime load-profile review enforcement, communications command/review/platform-ingress/envelope/channel-adapter/channel-session/platform-adapter/notification, embedded-dashboard render/security/preflight/request/session flows, observability collection/operations/export/alert-route/endpoint/bind/scrape/tracing/failure-capture flows, and a local operator-surface aggregate gate over communications, dashboard, observability, deployment-host wrapper reporting, and runtime-smoke integration, communications/CLI command, notification, and local authenticated channel-adapter boundaries with local audit/state checkpoints plus caller-supplied local notification rate-limit/outage gating, embedded-dashboard local render and hosted-security review boundaries with local audit/state checkpoints, observability/runbook local collection, operations-review, export dry-run, local alert-route dispatch through the deterministic communications notification boundary, endpoint preflight, loopback-bind validation, authenticated scrape preflight, one-shot loopback metrics endpoint validation, scoped panic-hook, and runtime failure-capture boundaries with local audit/state checkpoints plus a repeatable local observability-runtime CLI gate, testing/fuzzing/backtesting validation-plan boundaries with local validation-runner, property-check, and validation-corpus CLI audit/state/reopen checks, packaging/deployment plan boundaries with repeatable local example-container validation, static deployment hardening validation, static example systemd-unit validation, manual non-mutating systemd lifecycle plan/inspect evidence tooling, combined deployment-host runtime report tooling with non-mutating audit/state filesystem preflight, audit retention active/archive path preflight reporting, and structured runtime load-profile review enforcement, the aggregate deployment-runtime gate composing 35 local probes including thirteen sanitized runtime/deployment transcript/rehearsal validators plus local runtime config reload, local SQLite WAL schema migration validation, local deployment config redaction validation, and local deployment log redaction validation, sanitized service-manager lifecycle, service-manager lifecycle rehearsal, deployment disk-full, deployment retention, deployment permission, deployment backup/restore, deployment graceful-shutdown, deployment audit/SQLite, deployment SQLite schema migration, rollback execution, incident-response execution, deployment failure-capture, and deployment response drill rehearsal validators, non-mutating rollback-drill evidence tooling, non-mutating incident-response drill evidence tooling, non-mutating deployment evidence bundle indexing with rollback/incident/failure-capture/backup-restore/graceful-shutdown/audit-SQLite/schema-migration transcript components, and non-mutating deployment evidence checklist validation, external hardening evidence/checklist boundaries, deterministic agentic handoff package boundaries, and a local handoff-candidate aggregate gate over handoff audit, execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening core, and deployment-evidence checklist exist; current local and GitHub Actions validation covers structure, formatting, workspace compilation, tests, clippy, locked release build, dependency audit, dependency license policy validation, SBOM generation, local-SARIF SAST evidence, example image scan, production-intent container build/scan/hardened-smoke validation, static deployment hardening/config-loading/redaction checks, static example systemd-unit checks, ARM cross-target check validation, secret-pattern scan, deployment evidence checklist artifact generation, hardening evidence indexing, the local operator-surface aggregate gate, the local execution-path aggregate gate, the local opportunity-scenario aggregate gate, the local connector-scenario aggregate gate, and the strict local handoff-candidate aggregate gate. Production container publishing, service installation, deployment-host systemd lifecycle execution validation, rollback execution validation, incident-response execution validation, ARM target-class runtime validation, runtime deployment, production load testing, live/provider-backed market-data and fee validation, external fuzzing engine, larger external/deployment opportunity scenario-corpus execution, external backtest corpus execution, daemon-hosted production observability runtime/exporters/alerting, daemon-wide/deployment-host panic hooks, real dashboard hosting, outbound messaging integrations, external adapter submission, real exchange-specific live adapters, live RPC adapters, custody-backed wallet signer, transaction broadcasts, bridges, external address ownership validation, deployment-host backup/restore execution, deployment-host graceful-shutdown execution, deployment-host audit validation, physical disk-full and retention/rotation execution evidence, service-manager restart execution evidence, deployment-host durability validation, deployment-host schema migration execution, external sandbox/live fill calibration evidence, and live trading adapters remain unimplemented or unvalidated.
- Safety posture: Design-time controls, Phase 2 mode gates, Phase 3 policy checks, Phase 4 redacted audit/hash-chain primitives, local audit lock/sync/replay/disk-full-failure validation, side-effect-free retention planning, side-effect-free stale-lock restart recheck planning, and SQLite WAL checkpoint persistence with local schema v1 migration/future-version rejection, Phase 5 read-only market-data/fee boundaries, Phase 6 paper-only simulation boundaries with local paper-report, paper-ledger checkpoint, and paper audit journal persistence, Phase 7 CEX framework live-order denial plus local validation audit/state persistence, Phase 8 DEX/Web3 framework live-swap/RPC/broadcast denial plus local validation audit/state persistence and local signer-request rejection records, Phase 9 opportunity discovery, Phase 10 draft-only execution planning with policy preflight outcomes plus local plan-draft and policy-outcome audit journaling, Phase 11 execution-adapter boundary records with policy revalidation, Phase 12 command/notification boundaries with outbound-network denial plus local audit/state persistence and future-delivery preconditions, Phase 13 dashboard render records with server/public-exposure/live-control denial plus local audit/state persistence and future-hosting preconditions, Phase 14 observability records with metrics-endpoint/outbound-alert denial plus local audit/state persistence, Phase 15 validation plans and local property checks with external-fuzzer/live-network/live-execution denial, Phase 16 packaging records with build/deployment/public-exposure/production-claim denial plus non-mutating systemd lifecycle inspection tooling, Phase 17 hardening records with external-action/production-claim/live-funds/public-exposure denial, Phase 18 handoff records with external-agent-execution/external-validation-claim/production-claim/live-funds/public-exposure denial, Phase 19 runtime lifecycle records with audit-before-adapter and state-before-adapter checks, Phase 20 SQLite WAL durability checks with non-secret probes, Phase 21 paper balance ledger checks, Phase 22 process-level crash/restart recovery checks, Phase 23 realistic paper fill checks, Phase 24 paper replay/calibration/backtest validation checks, Phase 25 local paper audit append/replay checks, and Phase 26 local audit durability probes exist; custody-backed signer implementation, live connector integrations, live execution controls, deployment-host audit validation, physical disk-full and retention/rotation execution evidence, operator-controlled service-manager lifecycle execution evidence, external sandbox/live fill calibration evidence, deployment-host durability validation, deployment-host schema migration execution, external adapter submission, real communication delivery, real dashboard hosting/authentication, real metrics/exporter/alert runtime, external property/fuzz execution, production container/systemd/ARM deployment validation, broader Phase 17 production hardening review, future-agent execution validation, and external validation are not yet implemented or externally validated.
- Current mode: Config/policy/audit/market-data/paper/CEX-framework/DEX-Web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-CLI/dashboard/observability/testing/packaging/hardening/handoff-ready greenfield construction with live trading disabled until later custody, exchange-specific live connector, DEX RPC adapter, custody-backed signer, live adapter submission, production execution hardening, real communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, packaging/deployment validation, production hardening review, and external validation phases are complete.

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

Phase 19 implementation status:

- `arb-core::runtime` defines local `RuntimeLifecycleRequest`, `RuntimeLifecycleRecord`, lifecycle status/error types, and `run_local_runtime_lifecycle`.
- The lifecycle appends an audit event, persists the execution-plan draft, appends a plan-checkpoint audit event, evaluates the deterministic execution-adapter boundary, persists the adapter run, and appends an adapter-complete audit event.
- Local tests cover concurrent lifecycle access through shared audit journal and SQLite WAL state paths without service startup.
- Local tests cover simulated state permission failure and verify the lifecycle stops before adapter evaluation.
- The runtime boundary also defines a local graceful-shutdown checkpoint model that appends shutdown audit records and persists a non-secret state checkpoint without stopping services.
- The runtime boundary also defines local backup-restore validation reports that copy non-secret audit/SQLite artifacts after WAL checkpoints, reopen the copies, verify restored planner and adapter checkpoints, and expose both one-shot and concurrent-load CLI gates without touching production files.
- The runtime boundary also defines a local restart recovery validation report and standalone CLI that replay audit records, reopen SQLite checkpoints, carry compact recovered connector-lifecycle and opportunity-trace summaries alongside planner/adapter/graceful-shutdown checkpoint outcomes, classify local recovery as ready-for-local-review or needs-operator-review without resuming services, surface those labels through CLI/report status for local operator review, and fail closed when required checkpoints are missing. A local process-supervised restart CLI writes lifecycle checkpoints in a child process and validates restart recovery from the parent without service-manager actions.
- The runtime boundary also defines a local deployment-like smoke validation report, a repeated-iteration load/latency aggregate report, and CLI runners that run lifecycle, graceful-shutdown, backup/restore, permission-denial fail-closed validation, incomplete-recovery missing-checkpoint fail-closed validation, restart recovery with opportunity-trace summary accounting, local communications command-route, remote-command review, platform command-ingress, remote-command envelope, channel-adapter validation, channel-session validation, platform-adapter review, and notification-dispatch checkpoint recovery, local dashboard render, hosted-security review, hosted-request preflight, bounded one-shot hosted-request checkpoint recovery, and hosted-session validation summary recovery for accepted/unauthenticated/CSRF/rate-limit request accounting, local observability collection, operations review, export dry-run, alert-route dispatch, endpoint preflight, loopback-bind validation, metrics scrape preflight, bounded one-shot metrics endpoint validation, scoped tracing subscriber capture, local failure-capture checkpoint recovery, standalone local runtime panic-hook failure-capture validation, audit durability probes, and blocked-state preflight fail-closed checks together without starting services, public dashboard servers, long-lived metrics endpoints, exporters, alert delivery, outbound-network notification delivery, service-manager actions, deployment-state inspection, external adapter submission, live execution, or production-readiness claims.
- Live-scope lifecycle requests are rejected before audit/state mutation.
- Lifecycle records preserve `external_submission_performed = false` and `live_execution_performed = false`.

Boundaries:

- Does not sign transactions
- Does not bypass policy
- Does not directly mutate secrets
- Does not submit external adapter requests or live orders

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
- Does not itself persist audit events; runtime lifecycle and audit modules own durable audit writes
- Does not replace connector-specific validation

Destination allowlist implementation status:

- `arb-core::destination` defines local approved-destination entries, destination allowlist snapshots, append-only audit records, and SQLite WAL checkpoint helpers.
- Destination entries reject LLM-generated approval sources, require enabled entries to reference ownership evidence, and validate chain/label/fingerprint/operator approval metadata without storing secrets.
- Local ownership-evidence review reports audit and checkpoint reference presence only; they do not verify chain ownership, load signers, sign challenges, or approve production readiness.
- `arb-agent validate-withdrawal-policy-boundary --workspace <fresh-dir>` verifies local config, strategy-profile, policy-engine, destination-allowlist, signer-reference, and audit/state fail-closed withdrawal denial without enabling withdrawals, signing, wallet custody, RPC calls, or production-readiness claims.
- `arb-agent validate-destination-boundary-audit --workspace <fresh-dir>` verifies local destination allowlist and ownership-reference review audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without address ownership proof, signer loading, challenge signing, RPC calls, transfers, withdrawals, or production-readiness claims.
- `PolicyContext` carries local approved destination entries, and `PolicyEngine` rejects `DestinationPolicy::ApprovedAddress` unless the chain/label matches an enabled allowlist entry.
- The boundary does not prove address ownership, administer a production address book, sign transactions, withdraw funds, bridge assets, or call wallet/RPC services.

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

- `arb-core::secrets` now defines reference-only secret identifiers, non-cloneable redacted `SecretMaterial` with local clear/drop byte clearing, a `SecretProvider` trait, an environment-variable provider skeleton, a local keystore alias loader using versioned XChaCha20-Poly1305 authenticated encryption with alias-bound associated data, temporary master-key/plaintext clearing after use, authenticated-ciphertext tamper rejection tests, metadata-only local keystore entry preflight reports, and non-mutating local rotation plan records with audit/SQLite checkpoint helpers.
- `arb-agent validate-secret-boundary-audit --workspace <fresh-dir>` verifies local secret-rotation plan audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without secret material loading, plaintext decryption, keystore entry writes, external credential revocation, or production-readiness claims.
- `arb-core::signer` adds local signer secret-scope review records that require an approved keystore alias, strategy id, and chain before future signer work can proceed, with audit/SQLite checkpoint helpers and explicit no-key-load/no-plaintext-decrypt/no-sign/no-broadcast/no-RPC side-effect fields.
- The local keystore, rotation-plan, and signer-scope boundaries are not production custody: OS keyring behavior, production key derivation policy, runtime signer-scoped key use, actual secret generation/keystore writes/revocation, deployment filesystem validation, panic-path review, and external custody review remain future work.
- `arb-core::signer` now defines a local fail-closed signer request boundary with policy-decision matching, append-only audit records, SQLite WAL checkpoints, and explicit no-key-load/no-sign/no-broadcast/no-RPC side-effect fields.
- `arb-agent validate-signer-boundary-audit --workspace <fresh-dir>` verifies local signer request and signer secret-scope audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without key loading, plaintext decryption, signing, broadcasts, RPC calls, custody, or production-readiness claims.
- The signer boundary never loads wallet material, signs payloads, broadcasts transactions, calls RPC endpoints, or claims production readiness.

Boundaries:

- LLM subsystem cannot access raw secrets
- Messaging subsystem cannot access raw secrets
- Strategy subsystem cannot access raw secrets
- Dashboard cannot display raw secrets
- Local signer request records are evidence of fail-closed request handling only; they are not custody or signing implementation.

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

Implementation status:

- `arb-core::policy` defines deterministic execution intents, policy decisions, trust-contract denials, and non-secret `PolicyDecisionRecord` summaries.
- Local policy decision records can be appended to the hash-chained audit journal and persisted through the typed SQLite WAL/state checkpoint boundary.
- `arb-agent validate-policy-decision-audit --workspace <fresh-dir>` records approved and denied local policy decisions, verifies audit replay plus SQLite checkpoint recovery, proves invalid side-effectful policy-decision audit records fail closed without advancing the journal, and proves state-write failure is surfaced without external submission, secrets, or live execution.
- Policy decision recording explicitly reports no external submission and no secret material recording.
- Live connector submission, signer/custody enforcement, production runtime wiring, and external policy validation remain future work.

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

Implementation status:

- `arb-core::strategy` defines `StrategyProfile`, typed capital/risk/opportunity/execution/venue/alert parameter groups, `StrategyPolicyConstraintReport`, and `STRATEGY_PROFILE_VERSION`.
- Strategy profiles validate mode, capital allocation, risk ceilings, opportunity thresholds, execution-shape toggles, venue/chain/router/asset allowlists, and destination labels.
- Local strategy constraint reports reject candidate intents outside the profile without executing, signing, broadcasting, calling live networks, or replacing the policy engine.
- `DeterministicExecutionPlanner::plan_with_strategy_profile` composes draft-only planning, policy preflight, and typed strategy constraint checks before any adapter boundary; rejected strategy constraints leave the draft in a denied state.
- `validate_strategy_profile_replay_corpus` replays the local historical opportunity fixture corpus through accepted and rejected strategy profiles, proving draft-ready vs policy-denied outcomes without adapter submission, signing, broadcasting, or live network calls.
- `validate_strategy_profitability_tuning` derives a deterministic low/median/high profitability-threshold sweep from local replay intent net profit and proves monotonic draft-ready vs policy-denied behavior across the historical corpus without adapter submission, signing, broadcasting, or live network calls.
- `migrate_config_toml_to_current` validates current configs and upgrades known local legacy `[markets]` and `[notifications]` aliases plus legacy venue allowlist field names under `[venues]` into the current non-secret schema without loading secret material or enabling live execution.
- Live-armed strategy scope, withdrawals, bridges, and flashloans are denied in this boundary.
- External calibration remains future work.

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

- `arb-core::market_data` defines normalized market pairs, price levels, top-of-book quotes, order-book snapshots, freshness classification, market-data requests, provider capabilities, local provider preflight records, local reconnect/backoff plan validation records, local quality-assessment records, local historical-persistence batch records, local paid-provider evaluation records, audit/state checkpoint helpers for preflight, reconnect, quality-assessment, historical persistence, and paid-provider evaluation reports, and a read-only `MarketDataProvider` trait.
- `arb-agent validate-market-data-boundary-audit --workspace <fresh-dir>` verifies local clean/degraded provider-preflight and ready/blocked reconnect-plan audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without REST calls, WebSocket connections, provider credentials, downloaded market data, or production-readiness claims.
- `arb-agent validate-market-data-quality-assessment` verifies local acceptable/degraded/blocked market-data quality scoring over normalized quotes and optional order books for freshness, spread, depth, and capture-latency thresholds without provider accounts, REST calls, WebSocket connections, or production-readiness claims.
- `arb-agent validate-market-data-history-persistence --workspace <fresh-dir>` verifies local normalized quote/order-book history batches persist through SQLite WAL checkpoints, replay through audit summaries, retain the latest records per kind under deterministic truncation, and fail closed on invalid audit/state writes without provider accounts, REST calls, WebSocket connections, or production-readiness claims.
- `arb-agent validate-paid-market-data-provider-evaluation` verifies local ready/blocked paid-provider comparison dossiers for coverage, latency, rate-limit, cost, failure-behavior, and governance metadata without provider accounts, billing setup, API keys, REST calls, WebSocket connections, or production-readiness claims.
- `arb-core::fees` defines fee schedules, fee estimates, fee-adjusted edge calculation, liquidity-role classification, reference-only fee verification records, local fee schedule reconciliation review records, audit/state checkpoint helpers for fee verification reports, and a read-only `FeeProvider` trait.
- `arb-agent validate-fee-boundary-audit --workspace <fresh-dir>` verifies local current/blocked fee-verification audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without provider API calls, chain RPC calls, credentials, account queries, or production-readiness claims.
- These models are local deterministic primitives only; they do not open sockets, call exchanges, call chain RPCs, load credentials, or execute trades.

Boundaries:

- Must track data provenance
- Must never treat stale data as executable
- Must not place orders, sign transactions, mutate balances, or load trading secrets
- Live provider validation, provider-side rate-limit reconciliation, live WebSocket reconnect behavior, and paid-data integrations are deferred


### 6A. Paper Connector Subsystem

Responsibilities:

- Provide deterministic in-memory market-data fixtures
- Provide static paper fee schedules
- Produce policy-gated paper execution reports
- Track local simulated paper balances, notional reservations, and modeled fill settlement
- Support future opportunity-engine and backtesting phases without live funds

Phase 6 implementation status:

- `arb-core::paper` defines `PaperMarketDataProvider`, `PaperFeeProvider`, `PaperExecutionAdapter`, and paper execution reports.
- Paper market data implements the Phase 5 `MarketDataProvider` trait using caller-supplied normalized order-book snapshots.
- Paper fees implement the Phase 5 `FeeProvider` trait using static schedules.
- Paper execution requires `PolicyEngine` approval and rejects non-paper scopes before producing a report.
- `persist_paper_execution_report_checkpoint` stores the latest deterministic paper report through the typed local `StateStore` boundary and can use `SqliteWalStateStore` for non-secret local checkpoint persistence.
- `PaperBalanceLedger` tracks caller-supplied simulated balances, reserves quote notional for paper intents, settles filled, partial, or unfilled reports with net paper P&L, releases unfilled reserved notional, and fails closed on insufficient balances or missing reservations.
- `PaperFillSimulationRequest` and `PaperFillSimulationReport` consume caller-supplied local order-book depth, model buy/sell side fills, partial fills, unfilled outcomes, average fill price, slippage, latency, queue-position haircuts, and consumed levels without network calls.
- `PaperExchangeMatchingProfile`, `PaperVenueRealismRequest`, adverse-selection records, and calibration records model local venue-specific tick/step/min-notional behavior, P&L haircuts, and reference-only sandbox/live discrepancy data without calling venues or embedding evidence contents.
- `PaperAuditReplayValidationReport` reconstructs paper ledger balances from local ledger entries and detects replay mismatches.
- `append_paper_execution_intent_audit`, `append_paper_execution_report_audit`, `append_paper_ledger_entry_audit`, and audited ledgered execution helpers append paper intent, report, and reserve/settlement mutation records to the local append-only audit journal and reopen it for hash-chain replay checks.
- `ledger_execution_adapter_run_paper_fills` replays adapter reconciliation records against modeled fills, settles local deterministic execution-adapter modeled fills into the paper ledger, appends paper intent, report, and reserve/settlement audit records, persists the final ledger checkpoint, rejects duplicate modeled-fill settlement after SQLite checkpoint reopen before ledger/audit mutation, and refuses live/external-submission records.
- `PaperBacktestCorpus` execution runs local historical-fixture paper steps through the existing paper adapter and ledger without data downloads, live networks, or external execution.
- `PaperRuntimeValidationReport` distinguishes local replay/backtest validation from missing production-host evidence and never marks production readiness.
- `persist_paper_balance_ledger_checkpoint` stores the latest deterministic paper ledger through the typed local `StateStore` boundary and can use `SqliteWalStateStore` for non-secret local checkpoint persistence.

Boundaries:

- Does not call live exchanges, DEXes, RPC endpoints, or paid data providers
- Does not load secrets
- Does not sign transactions
- Does not mutate real balances
- Local exchange matching, adverse selection, reference-only calibration records, paper replay, and fixture backtesting are modeled deterministically, but external sandbox/live evidence is still missing
- Must call policy before paper execution reports are produced
- Provides local paper audit-before-action coverage for audited paper execution paths, but does not replace broader future mandatory audit-before-action journal integration across live connectors, external sandbox/live realism checks, or external production-host durability validation

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
- Local CEX validation records can be appended to the audit journal and persisted as SQLite WAL checkpoints for policy-approved framework validation outcomes.
- Local/mock CEX lifecycle responses can be reconciled through deterministic status-transition, fill-quantity/price/fee, cancelled-after-partial remainder accounting, lifecycle audit/state checkpoint, and duplicate-client-order-id checks without exchange calls, credentials, external submission, live cancellation, or live execution.
- Local CEX balance snapshot transcripts can normalize caller-supplied Binance account, Coinbase accounts, and Kraken balance payloads without exchange calls, credentials, account-state queries, balance mutation, or live execution.
- `arb-agent validate-connector-lifecycle-audit --workspace <fresh-dir>` verifies local/mock CEX lifecycle and local DEX/Web3 quote/simulation lifecycle audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without exchange calls, RPC calls, credentials, signing, broadcasts, external submission, live execution, or production-readiness claims.
- `LocalDeterministicCexAdapter` serves caller-supplied local quote/fee fixtures through read-only traits and validates paper orders through policy, returning only `LocallyValidated`.
- Live CEX order requests are explicitly denied in Phase 7.

Boundaries:

- Does not call exchange REST APIs
- Does not open WebSocket streams
- Does not load credentials
- Does not read live balances or query account state
- Does not submit or cancel live orders
- Does not mutate balances
- Local deterministic adapter does not prove exchange-specific REST, WebSocket, sandbox, fee-tier, rate-limit, cancel, or reconciliation behavior
- Does not validate terms, jurisdiction, rate limits, or fees externally
- Future live adapters must integrate real response lifecycle audit/state transitions, sandbox/live fill and cancel reconciliation, provider-backed rate-limit control, external credential/account validation, and deployment restart idempotency before any production use

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
- Local DEX/Web3 validation records can be appended to the audit journal and persisted as SQLite WAL checkpoints for policy-approved framework validation outcomes.
- Local DEX/Web3 quote/simulation lifecycle records can reconcile deterministic quote and local transaction-simulation responses, output shortfall, gas usage, duplicate intent ids, audit replay, and SQLite WAL checkpoint recovery without RPC calls, signer material, signing, broadcasts, bridges, live execution, or production-readiness claims.
- Local DEX/Web3 protocol risk review records check caller-supplied chain/pair scope allowlists, router/spender contract hygiene, allowance denial, approval revocation planning, gas/slippage caps, MEV controls, token metadata/contract/decimals review, and terms/jurisdiction/incident review without RPC calls, signer material, signing, broadcasts, bridges, live execution, or production-readiness claims.
- `arb-agent validate-connector-lifecycle-audit --workspace <fresh-dir>` covers this local DEX/Web3 lifecycle path together with the local/mock CEX lifecycle path as one repeatable no-live connector lifecycle gate.
- `LocalDeterministicDexAdapter` serves caller-supplied quote/fee/simulation fixtures, validates paper swap quotes through policy, and returns only non-broadcastable local simulation responses.
- Live DEX swaps, live RPC transaction simulation, signing, transaction broadcast, and bridges are explicitly unavailable in Phase 8.

Boundaries:

- Does not call chain RPC endpoints
- Does not call DEX/router/aggregator APIs
- Does not load wallet keys or signer secrets
- Does not build arbitrary contract calls from LLM output
- Does not sign transactions
- Does not broadcast transactions
- Does not execute bridges or withdrawals
- Local deterministic adapter and local protocol risk review do not prove live RPC, router/aggregator, gas, nonce, confirmation, approval/spender, MEV, or protocol-specific behavior against real venues or chains
- Local protocol risk review checks caller-supplied metadata for chain/pair scope, router/spender contract hygiene, token metadata/contract/decimals, slippage, MEV risk, gas, and terms/jurisdiction/incident review, but does not validate router contracts, token metadata, jurisdiction, incident history, slippage, MEV risk, gas, or protocol terms externally
- Future live adapters must integrate RPC/simulation/signing/broadcast lifecycle audit/state transitions, signer policy, spender/approval hygiene, nonce/confirmation tracking, and external protocol review before any production use

### 7. Opportunity Engine

Responsibilities:

- Detect candidate arbitrage opportunities from already-normalized market data
- Estimate net profit after supplied trading/network fees and top-of-book spread
- Rank opportunities deterministically
- Produce deterministic opportunity records
- Preserve model boundaries for CEX/CEX, DEX/DEX, CEX/DEX, and triangular route classes
- Forward only validated candidates to the future execution-planning layer

Phase 9 implementation status:

- `arb-core::opportunity` defines `OpportunityDiscoveryConfig`, `OpportunityDiscoveryRequest`, `OpportunityCandidate`, `OpportunityLeg`, `OpportunityScore`, `OpportunityRouteKind`, local replay corpus/report records, the Phase 27 local regression corpus builder, `OpportunityEngine`, and `DeterministicOpportunityEngine`.
- The deterministic engine consumes supplied `NormalizedQuote` and `FeeSchedule` values only.
- Market-data freshness failures are fail-closed.
- Ranking is deterministic and fee-aware.
- Same-venue triangular arbitrage search is implemented over supplied local quotes and fee schedules only.
- Local opportunity replay reports can check expected route presence, no-candidate false-positive scenarios, and forbidden route kinds against supplied non-secret records.
- Local historical opportunity fixture reports aggregate deterministic replay windows without downloading market data, calling exchanges/RPC endpoints, submitting orders, or mutating balances.
- Local opportunity discovery ranking collapses duplicate candidates by stable candidate id before planner handoff, and local opportunity planner handoff reports append one local audit record and persist one SQLite WAL state checkpoint for each deduplicated replay candidate before converting it into a draft-only plan, then fail closed on missing traces, missing recovered trace checkpoints after audit/state reopen, planner failures, adapter-submission flags, external-call flags, or live-execution flags.
- The built-in local regression corpus exercises cross-venue, no-candidate, triangular, depth/inventory, transfer-risk, DEX/DEX, CEX/DEX, candidate-truncation, and stale-data fail-closed scenarios without live data or execution.
- `arb-agent validate-opportunity-replay` runs the built-in local corpus and fails closed on failed scenarios or forbidden side-effect flags; `arb-agent validate-opportunity-historical-fixtures` runs the local historical fixture corpus and fails closed on failed windows or forbidden side-effect flags; `arb-agent validate-opportunity-planner-handoff` converts replay candidates into draft-only plans and fails closed on planner failures or forbidden side-effect flags; `arb-agent validate-opportunity-trace-recovery` reopens local audit/state traces and fails closed on missing recovered trace evidence or forbidden side-effect flags; `arb-agent validate-local-validation-run --workspace <fresh-dir>` validates the local deterministic validation-runner audit/state/reopen path; `arb-agent validate-local-property-checks --workspace <fresh-dir>` validates local plan invariants; `arb-agent validate-local-fuzz-corpus --workspace <fresh-dir>` replays deterministic local fuzz seed metadata with audit/state recovery while denying external fuzzer invocation; `arb-agent validate-local-validation-corpus --workspace <fresh-dir>` aggregates multiple deterministic plans through local validation/property-check boundaries; `arb-agent validate-local-paper-backtest-corpus --workspace <fresh-dir>` executes a local paper backtest corpus with filled, partial, and unfilled modeled outcomes, sanitized audit records, SQLite checkpoint recovery, and no live network or external execution; CI runs these commands as hard local gates.

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
- `append_execution_plan_draft_audit` writes one deterministic plan-draft audit event plus one redacted policy-decision audit event per generated intent, `persist_execution_plan_draft_checkpoint` stores the latest deterministic plan draft through the typed local `StateStore` boundary and can use `SqliteWalStateStore` for non-secret local checkpoint persistence, and `arb-agent validate-execution-planner-audit --workspace <fresh-dir>` verifies local planner audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without adapter submission, signing, broadcasts, withdrawals, bridges, or live execution.
- `adapter_submission_enabled` is always false.

Boundaries:

- Does not submit to adapters
- Does not place orders
- Does not sign transactions
- Does not broadcast transactions
- Does not withdraw or bridge funds
- Does not call CEX APIs
- Does not call DEX/router/RPC APIs
- Does not provide mandatory fail-closed live audit/state lifecycle orchestration yet
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
- `ExecutionPlanDraft` validation rejects duplicate draft intent ids and duplicate policy-outcome intent ids before adapter handoff.
- Each `ExecutionAdapterAttempt` records `policy_revalidated = true`; adapter run validation rejects records without adapter-boundary policy revalidation evidence, and local tests cover kill-switch denial at adapter time without modeled fills or submission.
- `ExecutionAdapterRunRecord` validation rejects duplicate attempt sequences, attempt intent ids, fill ids, fill intent ids, reconciliation ids, reconciliation intent ids, and unknown fill/reconciliation intent references before audit or checkpoint persistence.
- `append_execution_adapter_run_audit` writes sanitized deterministic adapter-run metadata to the local append-only audit journal, `persist_execution_adapter_run_checkpoint` stores the latest deterministic adapter run through the typed local `StateStore` boundary using `SqliteWalStateStore` for non-secret local checkpoint persistence when configured, `arb-agent validate-execution-adapter-audit --workspace <fresh-dir>` verifies adapter-run and recovery-plan audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without external submission, and `ledger_execution_adapter_run_paper_fills` can locally replay reconciliations and settle modeled paper fills into the paper ledger with audit/state recovery plus duplicate-settlement rejection after checkpoint reopen.
- `ExecutionAdapterRecoveryPlan` records local no-op, cancel-remainder, and hedge-exposure follow-up plans for full-fill, no-fill, and partial-fill adapter outcomes; `append_execution_adapter_recovery_plan_audit` and `persist_execution_adapter_recovery_plan_checkpoint` persist sanitized local recovery metadata without submitting cancels, hedges, orders, swaps, transactions, broadcasts, withdrawals, or bridges.

Boundaries:

- Cannot operate outside selected mode
- Cannot execute blocked intents
- Cannot override policy denial
- Does not call CEX APIs
- Does not call DEX/router/RPC APIs
- Does not sign or broadcast transactions
- Does not withdraw or bridge funds
- Does not submit external orders or transactions
- Does not provide production-validated durable audit/state persistence yet

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
- Audit appends now acquire a local lock before replay/sequence/hash calculation and call `sync_all` after writing.
- `validate_audit_journal_durability` creates local non-secret journals to validate append/reopen replay, crash-like truncated replay rejection, tamper rejection, concurrent append replay, and invalid filesystem failure behavior.
- `arb-core::state` defines a state-store trait and checkpoint model with an in-memory non-production implementation for tests and early wiring only.

Boundaries:

- Must avoid secret leakage
- Must be efficient and non-blocking where possible
- Must support durable flush semantics for live-fund operations
- Phase 4 JSONL audit is not SQLite WAL storage, externally crash-tested deployment durability, physical disk-full/deployment-host retention/rotation execution validated, or log shipping
- State persistence has local SQLite WAL integrity/checkpoint/reopen/backup-restore/multi-handle validation, but is not production-ready until crash/restart, filesystem failure, and deployment-host validation are executed with non-secret evidence references

### 11. Communications Subsystem

Responsibilities:

- Local CLI command parsing and routing boundaries
- Operator notification models
- Non-secret channel profile models
- Secret-safe message validation, redaction, and truncation
- Local dispatch records for future notifications
- Future Telegram, Discord, Matrix, email, Slack, PagerDuty, Signal, iMessage, webhook, or SMS adapters after explicit validation

Phase 12 implementation status:

- `arb-core::communications` defines `CommunicationBoundaryConfig`, `NotificationChannelProfile`, `NotificationChannelSafetyState`, `OperatorCommand`, `OperatorCommandAuthorizationStatus`, `OperatorCommandRouter`, `DeterministicOperatorCommandRouter`, `OperatorNotification`, `NotificationPublisher`, `DeterministicNotificationBoundary`, `NotificationDispatchRecord`, `ChannelAdapterValidationRequest`, and `ChannelAdapterValidationReport`.
- The deterministic command router accepts local status/help/config/safety/roadmap/plan-only commands as typed boundaries.
- The command router requires local operator command-source authorization, rejects disabled local CLI routing, and rejects direct remote/scheduled/dashboard command routing. Separate local remote-command security review, mocked platform command-ingress, and envelope validation records model authentication, token-reference/raw-material controls, platform signature/identity verification, platform authorization, channel permission, replay protection, command allowlisting, command-injection marker detection, freshness, provider state, and unsafe-command denial without enabling remote execution, platform calls, or outbound delivery.
- Local authenticated channel-adapter validation records connect ready remote envelopes to local notification dispatch records with replay, provider-rate-limit, provider-outage, no-delivery, no-network, and audit/state recovery checks, without platform tokens or real delivery.
- Local channel-session validation summaries aggregate accepted, unauthenticated, replayed, and provider-unavailable adapter outcomes so local runtime validation can prove those denial paths without outbound delivery.
- Local mocked platform command-ingress validation records prove token-reference metadata, raw-token-material denial, platform-signature verification, identity authorization, channel permission, replay nonce, freshness, provider-state, command-injection, and side-effect controls before a ready local ingress record is converted into remote envelope validation input.
- Local platform-adapter control review records model non-secret token-reference metadata, raw-token-material denial, platform identity verification/authorization, channel permission, command-injection blocking, token revocation, provider rate-limit, and provider outage controls without storing tokens, calling platform APIs, or delivering messages.
- `arb-agent validate-communications-runtime --workspace <fresh-dir>` replays and recovers local route, remote-review, platform-ingress, remote-envelope, channel-adapter, channel-session, platform-adapter review, and notification records through the append-only audit journal and SQLite WAL state store without outbound delivery.
- The deterministic command router rejects live execution, withdrawals, bridges, signing, and broadcast command requests.
- The deterministic notification boundary creates local dispatch records only and preserves `outbound_network_used = false`.
- Caller-supplied local notification channel safety observations can block dispatch records for rate limits or outages before any future delivery adapter would be considered.
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

- `arb-core::dashboard` defines `DashboardBoundaryConfig`, `DashboardServerBinding`, `DashboardAccessContext`, `DashboardAccessAuthorizationStatus`, `DashboardSnapshot`, `DashboardPanel`, `DashboardPanelItem`, `DashboardRenderer`, `DeterministicDashboardRenderer`, `DashboardRenderRecord`, hosted-dashboard security review records, hosted-dashboard request preflight records, a bounded one-shot authenticated loopback hosted-request validation path that serves sanitized rendered-dashboard body content with byte/digest accounting, and local hosted-session validation summaries for accepted, unauthenticated, CSRF-rejected, and rate-limited request accounting.
- The deterministic renderer creates local in-process render records only.
- The deterministic renderer requires local access authorization and rejects hosted/browser session sources until real authentication is implemented.
- The dashboard boundary also defines local hosted-security review records that require authentication, authorization, CSRF protection, CSRF token rotation/scoping, secure headers, clickjacking protection, rate limits, loopback-only defaults, and side-effect denial before future hosted work can be locally reviewed.
- Local hosted-request preflight records account for loopback-only binding, browser-session source, hosted authentication/authorization, CSRF enforcement for state-changing methods, secure response headers, clickjacking/header coverage, and rate-limit windows without starting a server or binding sockets.
- Local hosted-session validation records summarize multiple local hosted-request validations so `validate-dashboard-runtime` can prove one accepted authenticated loopback request plus unauthenticated, CSRF-invalid, and rate-limited rejections through audit replay and SQLite WAL checkpoint recovery without starting a persistent server or exposing public routes.
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

- `arb-core::observability` defines `ObservabilityBoundaryConfig`, `ObservabilityEndpointBinding`, `ObservabilityAccessContext`, `ObservabilityAccessAuthorizationStatus`, `HealthStatus`, `ComponentHealthStatus`, `StructuredLogEvent`, `MetricSample`, `Runbook`, `ObservabilitySnapshot`, `ObservabilityCollector`, `DeterministicObservabilityCollector`, `ObservabilityRecord`, local export/alert dry-run records, local endpoint/exporter preflight records, local ephemeral loopback bind validation records, local authenticated metrics scrape preflight records, local one-shot loopback metrics endpoint validation records, local bounded multi-scrape metrics runtime probe records, local scoped tracing subscriber validation records, sandbox-only observability log retention/rotation execution records, local scoped panic-hook capture records, and local runtime failure-capture records.
- The deterministic collector creates local in-process observability records only.
- The deterministic collector requires local collection authorization and rejects metrics endpoint/exporter/alert delivery session sources until real authentication is implemented.
- The observability boundary also defines local operations review records for retention, redaction, alert-route references, incident runbook references, and loopback/authenticated endpoint policy, plus deterministic non-network Prometheus-style metric rendering and alert-route dry-run accounting while preserving no-export/no-alert side-effect denial.
- Alert-route dry-run decisions can be bridged into the deterministic local communications notification boundary through `ObservabilityAlertRouteDispatchRequest`/`ObservabilityAlertRouteDispatchReport`, with append-only audit and SQLite WAL checkpoint helpers; accepted reports require local notification dispatch records and preserve no outbound alert delivery, no outbound network usage, no telemetry export, no live execution, and no production-readiness claims.
- Local endpoint/exporter preflight records account for loopback binding, authentication, authorization, transport protection, telemetry redaction, alert-route references, exporter backpressure/fail-closed controls, endpoint-start denial, public-exposure denial, telemetry-export denial, and outbound-alert denial without exporting telemetry. Local loopback bind validation records can open and immediately close an ephemeral numeric-loopback listener. Local metrics scrape preflight records validate authenticated loopback `GET /metrics` behavior in-process against rendered metric lines, and local one-shot endpoint validation briefly serves exactly one authenticated loopback socket scrape before closing the listener.
- The boundary rejects public network exposure, non-loopback bind hosts, outbound alert delivery, telemetry export, exporter sessions, and secret observability.
- Observability collection/export/failure records preserve `public_network_exposed = false` and `outbound_alerts_sent = false`; the one-shot endpoint validation separately records `local_metrics_endpoint_started = true` and `network_request_served = true` for the bounded loopback probe only.
- Secret-like health/log/metric/runbook/failure-capture text is redacted before local records are produced.

Default future approach:

- `tracing` for structured logs after explicit runtime integration scope
- optional OpenTelemetry exporter after authentication, redaction, and exporter review
- optional Prometheus-compatible metrics endpoint after loopback/authentication/rate-limit validation
- local runbooks from day one

Boundaries:

- No long-lived or public metrics endpoint startup in Phase 14
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

- `arb-core::testing` defines `ValidationHarnessConfig`, `ValidationTestCase`, `ValidationFixtureRecord`, `FuzzSeedRecord`, `FuzzCorpusDefinition`, `BacktestDatasetDefinition`, `BacktestScenarioDefinition`, `ValidationPlan`, `ValidationHarness`, `DeterministicValidationHarness`, `ValidationRunRecord`, `LocalPropertyCheckReport`, `LocalFuzzCorpusReplayReport`, `LocalValidationCorpusReport`, and local validation-run/property-check/fuzz-corpus-replay/validation-corpus audit/state checkpoint helpers; `arb-agent validate-local-validation-run --workspace <fresh-dir>`, `arb-agent validate-local-property-checks --workspace <fresh-dir>`, `arb-agent validate-local-fuzz-corpus --workspace <fresh-dir>`, `arb-agent validate-local-validation-corpus --workspace <fresh-dir>`, and `arb-agent validate-local-paper-backtest-corpus --workspace <fresh-dir>` execute deterministic local runner boundaries, enforce local validation-corpus breadth requirements for plans/test cases/fixtures/fuzz corpora/backtest scenarios, and verify audit/state reopen recovery without external tooling, while local `proptest` coverage exercises opportunity-engine invariants under `cargo test`.
- The deterministic harness validates plans and returns local records only.
- The boundary rejects external fuzzer invocation, live network tests, live execution tests, credential-bearing fixtures, live order submission, signing, and transaction broadcasts.
- Validation records preserve `external_fuzzer_invoked = false`, `live_network_used = false`, `live_execution_submitted = false`, and `signing_or_broadcast_performed = false`.
- Secret-like operator labels are redacted before validation records are produced.
- Local validation run, property-check, and validation-corpus records can be appended to the audit journal, recovered from SQLite WAL checkpoints, and exercised through local CLI runners without launching external fuzzers or live tooling.

Default future approach:

- `cargo test` for unit/integration tests after each workspace change
- property tests after explicit dependency and corpus design
- fuzzing with reviewed local harnesses after explicit future scope
- deterministic fixture replay and backtesting against curated local corpora
- CI gating after actual runner validation beyond the current local deterministic CLI audit/state/reopen and validation-corpus checks

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
- The deterministic planner validates local package/deployment plans and returns records only; local rollback-validation records can be audited and checkpointed without executing rollback steps.
- The boundary rejects public network exposure, embedded secret material, live trading deployment, build claims, deployment claims, and production deployment claims.
- Package records preserve `build_performed = false`, `deployment_performed = false`, `public_network_exposed = false`, `live_trading_enabled = false`, `secret_material_embedded = false`, and `production_deployment_claimed = false`.
- Example container, production-intent container, unsigned release artifact, systemd, ARM, and deployment notes exist as local/CI validation artifacts only. Current CI builds and scans the example container image, packages and verifies an unsigned locked release binary with SHA-256 manifest and unsigned provenance record, statically validates the example systemd unit, runs `systemd-analyze verify` syntax checks for the example unit against a temporary fake root, statically validates the ARM build-profile target/command/no-claim notes, and now defines production-container image-scan/hardened-smoke and ARM cross-target check jobs; `scripts/validate_release_artifact.py` can repeat the local unsigned release-artifact package/provenance/integrity/smoke gate with bounded build/smoke/metadata helper commands, `scripts/validate_container_example.py` can repeat the local example Docker/Trivy smoke gate with bounded Docker command timeouts and fail-closed unavailable-Docker reporting, `scripts/validate_production_container.py` can repeat the production-intent Docker/Trivy/help smoke gate and hardened read-only/no-network help smoke where Docker is healthy and fail closed with explicit non-claims when Docker is unavailable or unresponsive, `scripts/validate_systemd_example.py` can repeat static or optional syntax example-unit checks with bounded `systemd-analyze` execution, `scripts/validate_deployment_static_hardening.py` can repeat static deployment hardening and optional config/status smoke with bounded smoke execution, `scripts/validate_arm_build_profiles.py` can repeat ARM profile validation without installing targets or cross-building, `scripts/validate_arm_cross_check.py` can run `cargo check --workspace --target aarch64-unknown-linux-gnu --locked` through a host compiler when present or a bounded Docker fallback when it is not, `scripts/validate_systemd_lifecycle.py` can produce a manual lifecycle plan or bounded read-only deployment-host `systemctl show` inspection, `scripts/validate_deployment_host_runtime.py` can compose lifecycle evidence with a bounded lifecycle helper call, the local runtime-smoke CLI, local audit durability CLI, local sandbox audit-retention execution CLI, local graceful-shutdown checkpoint/reopen CLI, local backup/restore copy/reopen CLI, local blocked-state preflight CLI, non-mutating audit/state filesystem preflight reporting, non-mutating audit retention active/archive path preflight reporting, and local observability-runtime CLI reporting when explicitly requested, `scripts/validate_rollback_drill.py` can validate a non-mutating rollback-drill evidence plan, `scripts/validate_incident_response_drill.py` can validate a non-mutating incident-response drill evidence plan, `scripts/validate_deployment_evidence_bundle.py` can summarize the non-mutating helper outputs, including the deployment-host retention preflight component, into a compact bounded local evidence index, and `scripts/validate_deployment_evidence_checklist.py` can map remaining production evidence categories to sanitized locator references or explicit missing statuses through a bounded bundle call. These checks do not prove artifact signing, attestation upload, release publishing, image registry publishing, service lifecycle execution, ARM binary execution, ARM target-class runtime behavior, runtime deployment, deployment-host retention execution, observability exporter/alert operation, rollback readiness, incident-response readiness, physical disk-full behavior, or production readiness.

Default future approach:

- release builds through local or CI validation before release review
- container builds after approved local or CI runtime validation
- systemd service validation on Linux targets
- ARM target-class runtime validation on actual target hardware or verified emulator
- rollback drills before unattended operation

Boundaries:

- No image push, service install, or production deployment execution in Phase 16
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

The local strategy profile module now supports typed parameters such as:

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
- No production encrypted-at-rest secret manager or custody backend beyond local alias loading and metadata preflight
- No custody-backed wallet signer beyond local fail-closed signer request records
- No exchange-specific live CEX adapters
- No live DEX/Web3 RPC adapters, signer, transaction simulation integrations, or broadcasts; local DEX/Web3 request plans, response transcript parsers, transaction lifecycle transcript parsers, and protocol risk reviews now exist only as side-effect-free metadata/fixture parsing
- No live execution adapter submissions
- SQLite WAL state store exists for local checkpoints, is wired into the local runtime lifecycle, adapter recovery-plan checkpoint recovery, graceful-shutdown checkpoint, local backup-restore validation, local restart recovery summary boundaries, and local deployment-smoke blocked-state preflight checks, and has local integrity/checkpoint/reopen/backup-restore/multi-handle plus process-level crash/restart and concurrent lifecycle access validation, but deployment-host crash/restart/filesystem validation is missing
- Audit journal has local paper intent/report/ledger mutation append/replay wiring, local crash/concurrency/filesystem/simulated-disk-full validation probes, side-effect-free retention planning, local sandbox-only retention/rotation execution, and side-effect-free stale-lock restart recheck planning, but deployment-host audit validation, physical disk-full behavior, deployment-host retention/rotation execution, and service-manager restart execution evidence are missing
- CEX framework has local named exchange fixture matching, local Binance/Coinbase/Kraken REST/WebSocket market-data request plans, mocked order-book transcript parsing, local balance snapshot transcript parsing, local rate-limit validation, local credential/API-scope review, and local governance review for fee/rate-limit/terms/jurisdiction/API-capability/incident metadata only and is not connected to live REST/WebSocket APIs, account queries, or sandboxes
- DEX/Web3 framework has local Uniswap V3 quoter, 0x quote, Jupiter quote, and EVM simulation request plans, local response transcript parsing, local transaction lifecycle transcript parsing, and local protocol risk review only and is not connected to real RPC, router, aggregator, signer, simulation, or broadcast adapters
- Opportunity engine has current workspace Rust validation evidence and now models local caller-supplied order-book depth, paper inventory caps, transfer-risk penalties, same-venue triangular path discovery, duplicate-candidate collapse by stable candidate id, local replay/false-positive reports, a built-in local regression corpus, local historical fixture replay aggregation, local replay CLI validation commands, local candidate audit/state traces, local candidate trace restart/reopen recovery validation, and replay-candidate planner handoff validation wired into CI, but broader external/deployment scenario-corpus validation and external sandbox/live calibration remain missing
- Execution planner and execution-adapter framework have current workspace Rust validation evidence, local audit/state lifecycle wiring, and local adapter recovery-plan restart/smoke checkpoint recovery, and paper execution has local replay/backtest/audit-journal wiring, but production runtime validation and live/sandbox adapter integration remain missing
- No runtime deployment validation
- No production container, systemd, ARM, rollback-drill, or incident-response drill validation
- CI/CD execution validation exists for structure, Rust validation, locked release build, dependency audit, dependency license policy validation, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing only
- No external security review
- No live exchange API keys
- No production deployment environment

## Phase 18 Agentic Handoff Boundary

The agentic handoff subsystem provides deterministic package records, continuation prompts, governance checklists, external validation checklists, future-agent instructions, and local handoff-review audit/state checkpoints only. `arb-agent validate-agentic-handoff-audit --workspace <fresh-dir>` appends a sanitized handoff review to the local audit journal, persists the review through SQLite WAL state, reopens both stores, and verifies fail-closed invalid-audit and state-write behavior. It does not execute external agents, call coding-agent APIs, deploy infrastructure, approve production readiness, approve public exposure, approve live funds, or store credentials. Handoff records may reference current local/CI validation evidence, but they preserve unresolved gaps and live-funds blockers so future agents cannot silently erase deferred validation work.
