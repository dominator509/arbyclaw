# ROADMAP.md

## Project

ArbyClaw

## Current Roadmap Position

- Active phase: Phase 100 - Handoff Candidate Full Local Surface Gate implemented to require the handoff-candidate aggregate to compose execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening-core, deployment-evidence checklist, and local handoff audit validation while preserving no external agent execution, adapter submission, exchange/RPC calls, signing, broadcasts, image push, service installation, public exposure, secret loading, live execution, or production-readiness claims; next required work remains live REST/WebSocket providers, live DEX/RPC adapters, provider-backed reconnect/rate-limit/outage validation, provider-backed bad-data rejection validation, external fee/account-tier/gas/withdrawal validation, external latency measurement, external fuzz/property execution, broader external/deployment replay corpora, production load/security/backtest validation, real daemon-hosted observability/exporter/alert validation, real persistent dashboard hosting/auth/session/authorization validation, real outbound communications adapters, real platform authentication/authorization, real external sandbox/live calibration, custody-backed signer implementation, operator-controlled deployment-host/service-manager execution, real deployment-host config reload/start/stop/restart validation, real deployment-host backup/restore execution under service lifecycle and load, real deployment-host permission execution during runtime writes, physical disk-full execution, deployment-host retention/rotation execution, actual rollback execution, actual incident-response execution, real daemon failure-capture execution, real deployment-host audit/SQLite recovery execution, real deployment-host SQLite schema migration execution under service lifecycle, deployment-host config loading under service lifecycle, deployment-host log/audit redaction under service lifecycle, production image publishing, service deployment, and external evidence beyond current Rust/CI/local gates
- Active sub-roadmap: `PHASE_100_SUBROADMAP.md`
- Runtime-smoke composition update: local deployment-like runtime smoke now recovers observability operations review, export dry-run, alert-route dispatch, endpoint preflight, loopback bind, metrics scrape preflight, one-shot metrics endpoint validation, and scoped tracing capture alongside lifecycle, communications, dashboard, validation-runner, and paper-ledger recovery, without telemetry export, outbound alerts, public exposure, service-manager action, live execution, or production-readiness claims.
- Current production readiness: 96%
- Current implementation status: Minimal Rust workspace, typed config, reference-only secret boundary, policy/audit/state/runtime lifecycle gates, deterministic local market-data/fee/paper/CEX/DEX/opportunity/planner/adapter boundaries, local communications/dashboard/observability/testing/packaging/handoff gates, and aggregate local validation scripts exist. The connector aggregate gate now composes 21 local connector/provider components, including typed local CEX and DEX/Web3 live-adapter boundary reviews that validate local REST/WebSocket/RPC request-plan, lifecycle transcript, balance transcript, credential-scope, rate-limit, exchange-matching, unsigned payload, nonce, and non-broadcast prerequisites while blocking on sandbox/live evidence. The current deployment-host runtime wrapper and Phase 28/90 aggregate deployment-runtime gate compose 37 local runtime/deployment probes, including 24 nested runtime/helper probes and thirteen sanitized runtime/deployment transcript validators, with static deployment hardening/config smoke validation, runtime config reload, SQLite WAL schema migration, deployment config/log redaction, observability metrics runtime, runtime load-profile review, and production-runtime preflight enforcement while failing on unsafe side-effect flags. The packaging aggregate gate directly requires the production-intent container validator, including Docker validation completion, hardened read-only/no-network smoke, dropped capabilities, no-new-privileges, and explicit service-installation non-claims. The handoff-candidate aggregate gate now composes execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening-core, deployment-evidence checklist, and local handoff audit validation in one local candidate surface. Local and GitHub Actions evidence covers current structure, formatting, workspace compilation, tests, clippy, locked release build, aggregate packaging/deployment gate validation, dependency audit, dependency license policy validation, SBOM generation, local-SARIF SAST, example image scan, static example systemd-unit checks, secret-pattern scan, deployment evidence checklist artifact generation, and hardening evidence indexing. Live trading, production container publishing/service deployment, production systemd/ARM deployment validation, external validation runner execution beyond local deterministic gates, real fuzzing engines, broader external/deployment opportunity scenario-corpus execution, real external backtest corpus execution, real observability runtime, real dashboard hosting, outbound messaging integrations, external adapter submission, real exchange-specific live connectors, DEX RPC adapters, wallet signer, transaction broadcasts, custody backend, deployment-host durability validation, real deployment-host permission execution, physical disk-full and retention/rotation execution evidence, operator-controlled service-manager lifecycle execution evidence, deployment-host schema migration execution under service lifecycle, actual rollback execution evidence, incident-response execution evidence, daemon failure-capture execution evidence, external sandbox/live calibration evidence, and production execution logic are not implemented.
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
| 2 | Config, Secrets, and Mode Gates | Implemented; current workspace Rust/CI validation covered | +5% realized / +1% deferred | Typed config, environment secret references, local authenticated keystore loading for test/local entries, and live mode-gate validation added. |
| 3 | Policy Engine and Trust Contract | Implemented; current workspace Rust/CI validation covered | +7% realized / +3% deferred | Deny-by-default policy checks, intent model, trust-contract denials, local policy-decision audit/checkpoint validation, and CLI policy initialization added. |
| 4 | Audit Journal and State Store | Implemented; current workspace Rust/CI validation covered; deployment-host durability validation deferred | +4% realized / +2% deferred | Append-only hash-chained JSONL audit primitives, redaction checks, state-store trait, in-memory store, SQLite WAL checkpoint store, Phase 26 local audit crash/concurrency/filesystem/simulated-disk-full probes, side-effect-free retention planning, and stale-lock restart recheck planning added; physical disk-full, retention/rotation execution, service-manager restart execution, and deployment-host validation remain future work. |
| 5 | Market Data Core | Implemented; current workspace Rust/CI validation covered; live provider validation deferred | +5% realized / +2% deferred | Normalized quotes/order books, freshness windows, fee models, provider traits, local provider preflight, local reconnect/backoff plan validation, local quality assessment, local historical persistence, and local paid-provider evaluation dossiers added; no live network providers. |
| 6 | Simulated/Paper Connectors | Implemented; current workspace Rust/CI validation covered; paper-model limitations deferred | +6% realized / +1% deferred | Deterministic in-memory paper market data, static fee provider, policy-gated paper execution adapter, and local paper-report state checkpoint helper added; no live venues or balances. |
| 7 | CEX Connector Framework | Implemented as framework boundary; current workspace Rust/CI validation covered; live exchange validation deferred | +6% realized / +2% deferred | CEX venue profiles, capability registry, order request model, policy gate, connector traits, deterministic local adapter, local Binance/Coinbase/Kraken-shaped fixture matching, mocked order-book transcript parsing, local balance snapshot transcript parsing, local rate-limit validation, local credential/API-scope review, and local governance review for fee/rate-limit/terms/jurisdiction/API-capability/incident metadata added; no real REST/WebSocket, sandbox, balance, order, cancel, or live adapters. |
| 8 | DEX/Web3 Connector Framework | Implemented as framework boundary; current workspace Rust/CI validation covered; live RPC, signing, and broadcast validation deferred | +8% realized / +0% deferred in ChatGPT | Chain/router/token profiles, router capabilities, swap quote models, local transaction simulation boundary, policy gate, and connector traits added; no live RPC, signing, bridges, or broadcasts. |
| 9 | Opportunity Engine | Implemented as deterministic discovery/ranking boundary; current workspace Rust/CI validation covered; advanced route validation deferred | +8% realized / +0% deferred in ChatGPT | Cross-venue top-of-book discovery, CEX/CEX, DEX/DEX, CEX/DEX, triangular model boundary, freshness checks, and fee-aware scoring added; no execution intents or order placement. |
| 10 | Execution Planner | Implemented as draft-only model boundary with local execution-path aggregate gate coverage; adapter integration deferred | +7% realized / +0% deferred in ChatGPT | Deterministic plan drafts, per-leg intent generation, policy preflight outcomes, sequencing, failure-mode boundaries, local plan-draft audit/checkpoint persistence, per-intent policy-outcome audit records, opportunity planner handoff validation, strategy-constrained planner validation, and the local execution-path aggregate gate added; no adapter submission or live execution. |
| 11 | Execution Adapters | Implemented as deterministic boundary framework with local execution-path aggregate gate coverage; live submission deferred | +7% realized / +0% deferred in ChatGPT | Consumes planner drafts, revalidates policy, models attempts/fills/reconciliation, records local partial/no-fill recovery plans, blocks all external submission, durably records future-submission kill-switch/audit-state/idempotency preconditions, and participates in a local aggregate gate over planner, adapter, destination, signer, and Web3 non-broadcast controls. |
| 12 | Communications and CLI | Implemented as deterministic model/trait boundary with local operator-surface aggregate gate coverage; current workspace Rust/CI validation covered; real outbound integrations deferred | +6% realized / +0% deferred in ChatGPT | Typed local command parsing/routing, remote-command security review, mocked platform command-ingress validation, remote-command envelope validation, notification models, redaction checks, local dispatch records, channel-session summaries, platform-adapter control reviews, and the local operator-surface aggregate gate added; no platform tokens or outbound network delivery. |
| 13 | Embedded Dashboard | Implemented as deterministic model/trait boundary with local operator-surface aggregate gate coverage; current workspace Rust/CI validation covered; real hosting deferred | +3% realized / +0% deferred in ChatGPT | Local snapshot/panel/render records, hosted-security review, hosted-request preflight, one-shot authenticated loopback hosted-request validation, hosted-session summaries, fail-closed server binding, secret redaction, live-control denial, and the local operator-surface aggregate gate added; no persistent web server or public exposure. |
| 14 | Observability and Runbooks | Implemented as deterministic model/trait boundary with local operator-surface aggregate gate coverage; current workspace Rust/CI validation covered; real observability runtime deferred | +5% realized / +0% deferred in ChatGPT | Local health, structured-log, metric, runbook, scoped tracing/panic capture, alert-route dispatch, metrics scrape preflight, one-shot loopback metrics endpoint validation, bounded multi-scrape loopback metrics runtime validation, and the local operator-surface aggregate gate added; no daemon-hosted metrics runtime, telemetry export, or outbound alerts. |
| 15 | Testing, Fuzzing, and Backtesting | Implemented as deterministic model/trait boundary with local corpus breadth enforcement; current workspace Rust/CI validation covered; real fuzz/backtest execution deferred | +4% realized / +2% deferred | Validation harness config, test case metadata, fixture records, fuzz corpus definitions, backtest scenario definitions, local plan records, validation-corpus breadth requirements, and local `proptest` opportunity invariants added; no external fuzzer invocation or live network tests. |
| 16 | Packaging and Deployment | Implemented as deterministic model/docs boundary; current release-build, unsigned release-artifact packaging path, example-container, production-intent container path with hardened local smoke, aggregate packaging/deployment gate validation, example systemd-unit static/syntax validation, static ARM build-profile validation, ARM cross-target check path with bounded Docker fallback when the host compiler is unavailable, and bounded manual systemd lifecycle plan/inspect tooling covered where tools are available; production deployment/systemd/ARM runtime validation deferred | +2% realized / +0% deferred in ChatGPT | Package/deployment plan records, release gates, rollback steps, Docker/systemd/ARM docs, unsigned release-artifact package script and CI artifact upload definition, repeatable local example-container validation script, production-intent container build/scan/hardened-smoke script and CI job definition, a Phase 16 aggregate packaging/deployment gate that composes release-artifact, systemd example, static hardening, ARM profile, and ARM cross-target validation while preserving bounded host-or-Docker toolchain fallback accounting, static plus optional syntax example systemd-unit validator, static ARM target/command/no-claim validator, ARM `cargo check --target aarch64-unknown-linux-gnu --locked` script and CI job definition with host-or-Docker fallback behavior, and non-mutating bounded manual lifecycle inspection helper; no signing, publishing, image push, service install, deploy, ARM binary execution, or readiness claim. |
| 17 | External Production Hardening | Implemented as deterministic evidence/checklist boundary with a local hardening-core aggregate gate; real external validation deferred | +0% in ChatGPT | Evidence records, release blockers, hardening checklists, and a local hardening-core aggregate gate over packaging/deployment, dependency-license policy, and secret/policy boundary audits added; no pen test, cloud deployment, live exchange validation, or load test executed. |
| 18 | Agentic Handoff Package | Implemented as deterministic model/docs plus local audit/state boundary with a local handoff-candidate aggregate gate; external agent execution not performed | +0% direct | Codex/Cursor/Jules/Claude/human handoff package records, prompts, checklists, `validate-agentic-handoff-audit` local replay gate, and `validate_agentic_handoff_candidate_gate.py` over handoff audit, hardening core, and deployment-evidence checklist added; no external agents executed. |
| 19 | Runtime Lifecycle Wiring | Implemented as local deterministic fail-closed lifecycle boundary; production durability/runtime validation deferred | +2% realized / +0% deferred in ChatGPT | Runtime lifecycle records append audit events, persist planner state before adapter evaluation, evaluate deterministic adapter boundary, persist adapter run state, persist adapter recovery-plan state, reject live scope without external submission, validate concurrent local audit/SQLite lifecycle access, fail closed on simulated state permission failure, record local graceful-shutdown audit/state checkpoints without service actions, validate local audit/SQLite backup-restore copies without deployment actions, produce local restart recovery summaries with CLI-visible operator-review dispositions plus recovered local connector-lifecycle and opportunity-trace summaries alongside planner/adapter/recovery-plan checkpoint recovery without service resume, fail closed on incomplete recovery checkpoints, and now provide a local deployment-like smoke harness that combines those checks plus concurrent lifecycle reporting, blocked audit/state preflight checks, and audit durability probes without service-manager actions. |
| 20 | SQLite WAL Durability Validation | Implemented as local deterministic state-store validation boundary; external production-host validation deferred | +1% realized / +0% deferred in ChatGPT | Validates WAL mode, synchronous FULL, integrity check, WAL checkpoint truncate, primary reopen, checkpointed backup/restore, and multi-handle visibility with non-secret probes. |
| 21 | Paper Balance Ledgering | Implemented as local deterministic paper balance boundary; paper realism/audit/runtime validation deferred | +1% realized / +0% deferred in ChatGPT | Adds simulated paper balances, quote-notional reservation, fill settlement with net P&L, insufficient-balance denial, missing-reservation denial, and SQLite ledger checkpointing. |
| 22 | Crash/Restart Durability Validation | Implemented as local process-level SQLite WAL recovery validation; deployment-host validation deferred | +1% realized / +0% deferred in ChatGPT | Spawns child processes that write runtime checkpoints and exit abruptly, then reopens the WAL database and verifies integrity plus expected checkpoint survival. |
| 23 | Realistic Paper Fills | Implemented as local deterministic order-book depth and partial-fill modeling; external calibration deferred | +1% realized / +0% deferred in ChatGPT | Consumes supplied order-book depth, models latency, queue-position, slippage, full/partial/unfilled outcomes, and ledger-safe unfilled notional release without external submission. |
| 24 | Paper Replay, Calibration, Backtest, and Runtime Validation Boundaries | Implemented as local deterministic venue-realism, replay, and historical-fixture backtest boundary; external sandbox/live and deployment-host validation deferred | +1% realized / +0% deferred in ChatGPT | Adds local exchange matching profiles, adverse-selection penalties, reference-only calibration records, paper ledger replay validation, local backtest corpus execution, and runtime validation records without external calls. |
| 25 | Paper Audit Journal Integration | Implemented as local deterministic audit journal integration; production audit durability validation deferred | +1% realized / +0% deferred in ChatGPT | Adds append-only audit records for paper execution reports and paper reserve/settlement ledger mutations, with local journal reopen/replay tests and no external calls. |
| 26 | Audit Crash, Concurrency, Filesystem, Disk-Full, and Stale-Lock Validation | Implemented as local deterministic audit durability validation with manual deployment-host lifecycle/runtime/rollback/incident tooling; deployment-host execution evidence deferred | +1% realized / +0% deferred in ChatGPT | Adds lock/sync append behavior plus local probes for append replay, truncated crash-like replay rejection, tamper rejection, concurrent appends, invalid filesystem failure, simulated disk-full fail-closed behavior, retention/rotation planning without deletion, stale-lock restart recheck planning without service actions, a local deployment-like runtime smoke harness plus CLI runner without service-manager actions, standalone local graceful-shutdown checkpoint/reopen, backup/restore copy/reopen, backup/restore concurrent-load, runtime permission-denial, incomplete-recovery missing-checkpoint fail-closed, panic-hook failure-capture, restart-recovery replay/reopen, and process-supervised restart CLI gates, a bounded non-mutating systemd lifecycle plan/inspect helper, static deployment hardening validation, a bounded combined deployment-host runtime report wrapper with audit/state filesystem and audit retention active/archive preflight reporting, a non-mutating rollback-drill evidence wrapper, a non-mutating incident-response drill evidence wrapper, a compact bounded non-mutating deployment evidence bundle index, a bounded non-mutating external evidence checklist, and CI artifact/summary wiring for that checklist. |
| 27 | Opportunity Depth, Inventory, Transfer-Risk, and Replay Modeling | Implemented as local deterministic opportunity realism; external calibration and larger corpus validation deferred | +0% direct | Adds optional caller-supplied order-book depth walking, paper inventory caps, transfer-risk profile penalties, same-venue triangular path search, local replay/false-positive reports, built-in local regression corpus coverage for route classification, truncation, stale-data fail-closed cases, and duplicate-candidate collapse by stable candidate id, plus local replay, candidate audit/state trace, and replay-candidate planner handoff CLI/CI gates, and candidate liquidity/transfer-risk records without live data calls, real transfers, signing, broadcasts, or execution. |
| 28 | Deployment Runtime Aggregate Gate | Implemented as local deterministic aggregate runtime/deployment validation; deployment-host execution evidence deferred | +0% direct | Adds `scripts/validate_deployment_runtime_gate.py`, which now composes 37 local runtime/deployment probes by combining the deployment-host runtime helper with thirteen sanitized runtime/deployment transcript/rehearsal validators, explicitly enforces the embedded production-runtime preflight contract, local static deployment hardening/config smoke validation, local runtime load-profile review, local runtime config reload validation, local SQLite WAL schema migration validation, local deployment config redaction validation, local deployment log redaction validation, and deployment-host wrapper reporting for bounded local observability metrics runtime validation, and fails closed if any nested report or transcript summary claims service-manager action, external calls, live execution, secret loading, network listener startup, public exposure, telemetry export, outbound alert/network delivery, production-path mutation, deployment-path mutation, or production readiness. |
| 29 | Opportunity Scenario Aggregate Gate | Implemented as local deterministic aggregate opportunity scenario-corpus validation; external/deployment corpus evidence deferred | +0% direct | Adds `scripts/validate_opportunity_scenario_gate.py`, which now composes fourteen local opportunity/testing CLIs across replay, quote-load, provider-ingestion, historical fixtures, planner handoff, strategy replay, profitability tuning, validation-run, property-check, fuzz-corpus replay, validation-corpus, paper-backtest, local validation coverage review, and trace recovery, enforces local opportunity replay latency/throughput and validation coverage review fields, runs workspace-backed validation-run/property-check/fuzz-corpus/validation-corpus/paper-backtest probes against fresh local temp directories, and fails closed if external calls, external data downloads, external fuzzing, live network use, adapter submission, signing/broadcast, live execution, or production readiness are reported. |
| 30 | Connector Scenario Aggregate Gate | Implemented as local deterministic aggregate connector scenario validation; live exchange/RPC and external sandbox evidence deferred | +0% direct | Adds `scripts/validate_connector_scenario_gate.py`, which now composes nineteen local market-data, fee, CEX request-plan, CEX balance-snapshot, DEX request-plan, DEX response-transcript, DEX transaction-lifecycle transcript, DEX protocol-risk review, and CEX/DEX lifecycle CLIs, including market-data quality assessment, bad-data rejection review, fee schedule reconciliation review, paid-provider evaluation, historical persistence, and provider rate-limit/outage reconciliation review, and fails closed if live network use, credential loading, account queries, WebSocket opening, provider calls, external submission, RPC calls, signing/broadcast, live execution, or production readiness are reported. |
| 31 | Local CEX Market-Data Request Plans | Implemented as local deterministic exchange-specific request-plan validation; live REST/WebSocket clients deferred | +0% direct | Adds typed Binance/Coinbase/Kraken REST and WebSocket market-data request plans plus `arb-agent validate-cex-market-data-request-plans`, all without network calls, credentials, external submission, signing/broadcast, live execution, or readiness claims. |
| 32 | Local DEX/Web3 Request Plans | Implemented as local deterministic router/RPC request-plan validation; live RPC/router/simulation adapters deferred | +0% direct | Adds typed Uniswap V3 quoter, 0x quote, Jupiter quote, and EVM simulation request plans plus `arb-agent validate-dex-request-plans`, all without HTTP/RPC calls, credentials, signing, broadcasts, bridges, live execution, or readiness claims. |
| 33 | Local DEX/Web3 Response Transcript Parsing | Implemented as local deterministic DEX/router/RPC response parsing; live RPC/router/simulation adapters deferred | +0% direct | Adds `DexResponseTranscript` and `arb-agent validate-dex-response-transcripts` for local Uniswap V3 quoter, 0x quote, Jupiter quote, and EVM simulation payload parsing without HTTP/RPC calls, credentials, signing, broadcasts, bridges, live execution, or readiness claims. |
| 34 | Local CEX Order Lifecycle Transcript Parsing | Implemented as local deterministic exchange-shaped lifecycle response parsing; live REST/WebSocket/order/cancel adapters deferred | +0% direct | Adds `CexOrderLifecycleTranscript` for local Binance execution-report, Coinbase order-event, and Kraken order-status payload parsing into filled and cancelled-after-partial lifecycle reconciliation without REST calls, WebSockets, credentials, submissions, cancellations, live execution, or readiness claims. |
| 35 | Local CEX Balance Snapshot Transcript Parsing | Implemented as local deterministic exchange-shaped balance snapshot parsing; live authenticated balance reads deferred | +0% direct | Adds `CexBalanceSnapshotTranscript` for local Binance account, Coinbase accounts, and Kraken balance payload parsing into normalized local balance records without REST calls, WebSockets, credentials, account queries, balance mutation, live execution, or readiness claims. |
| 36 | Local DEX/Web3 Transaction Lifecycle Transcript Parsing | Implemented as local deterministic EVM/Solana transaction lifecycle parsing; live RPC, signing, and broadcast adapters deferred | +0% direct | Adds `Web3TransactionLifecycleTranscript` for local EVM receipt and Solana signature-status parsing into normalized lifecycle records with nonce and confirmation accounting, without RPC calls, signer material, signing, broadcast, bridge, live execution, or readiness claims. |
| 37 | Local DEX/Web3 Protocol Risk Review | Implemented as local deterministic protocol-risk metadata review; live contract/RPC/router/token/jurisdiction/incident validation deferred | +0% direct | Adds `DexProtocolRiskReviewRequest` and `DexProtocolRiskReviewReport` for local chain/pair scope allowlists, router/spender contract hygiene, allowance, gas/slippage, MEV, token metadata/contract/decimals, and terms/jurisdiction/incident checks without RPC calls, signer material, signing, broadcast, bridge, live execution, or readiness claims. |
| 38 | Local Signer Runtime Isolation Review | Implemented as local deterministic signer-isolation metadata review; custody-backed signing deferred | +0% direct | Adds `SignerRuntimeIsolationReviewRequest` and `SignerRuntimeIsolationReviewReport` for local no-LLM-signer-access, no-plaintext-key-exposure, policy/destination/secret-scope/audit/state precondition checks without key loading, plaintext decrypt, signing, broadcast, RPC calls, live execution, or readiness claims. |
| 39 | Local Signer Authorization Envelope | Implemented as local deterministic pre-signing reference-envelope review; custody-backed signing deferred | +0% direct | Adds `SignerAuthorizationEnvelopeRequest` and `SignerAuthorizationEnvelopeReport` for local policy/destination, signer-scope, runtime-isolation, transaction-simulation-reference, nonce-plan-reference, audit-reference, and state-checkpoint-reference checks without key loading, plaintext decrypt, signing, broadcast, RPC calls, live execution, or readiness claims. |
| 40 | Local Web3 Pre-Sign Safety Review | Implemented as local deterministic pre-sign simulation/nonce/lifecycle safety review; live RPC/signing/broadcast deferred | +0% direct | Adds `Web3PreSignSafetyReviewRequest` and `Web3PreSignSafetyReviewReport` for local simulation request/response coherence, gas cap, minimum output, nonce readiness, lifecycle coherence, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 41 | Local Web3 Nonce Reservation | Implemented as local deterministic nonce reservation; provider-backed nonce retrieval deferred | +0% direct | Adds `Web3NonceReservationRequest` and `Web3NonceReservationReport` for local nonce presence, stale nonce denial, duplicate in-flight nonce denial, already-reserved nonce denial, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 42 | Local Web3 Unsigned Payload Review | Implemented as local deterministic unsigned payload metadata review; real transaction construction deferred | +0% direct | Adds `Web3UnsignedPayloadReviewRequest` and `Web3UnsignedPayloadReviewReport` for local payload hash/label, nonce reservation, router/spender, gas cap, raw-calldata denial, audit replay, and SQLite checkpoint reopen without raw calldata generation, RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 43 | Local Web3 Broadcast Readiness Review | Implemented as local deterministic broadcast-readiness metadata review; real broadcast implementation deferred | +0% direct | Adds `Web3BroadcastReadinessRequest` and `Web3BroadcastReadinessReport` for local unsigned-payload/pre-sign prerequisite coherence, non-secret signer authorization/live adapter/operator approval references, broadcast-permission denial, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 44 | Local Web3 Unsigned Transaction Construction | Implemented as local deterministic unsigned transaction metadata construction; raw serialization/signing/broadcast deferred | +0% direct | Adds `Web3UnsignedTransactionConstructionRequest` and `Web3UnsignedTransactionConstructionReport` for local broadcast-readiness prerequisite coherence, payload/selector/digest/nonce/gas metadata, raw-calldata denial, raw-transaction serialization denial, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 45 | Local Web3 Provider Nonce Reconciliation | Implemented as local deterministic caller-supplied provider nonce snapshot reconciliation; real provider-backed nonce retrieval deferred | +0% direct | Adds `Web3ProviderNonceReconciliationRequest` and `Web3ProviderNonceReconciliationReport` for unsigned-transaction nonce coherence against sanitized local provider snapshot references, provider next nonce, pending nonce uniqueness, snapshot freshness, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 46 | Local Web3 Raw Transaction Serialization Review | Implemented as local deterministic serialization readiness metadata review; real raw serialization/signing/broadcast deferred | +0% direct | Adds `Web3RawTransactionSerializationReviewRequest` and `Web3RawTransactionSerializationReviewReport` for provider-nonce prerequisite coherence, transaction type, chain id, fee field, and access-list references, raw-byte denial, raw-calldata denial, raw-serialization denial, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 47 | Local Web3 Broadcast Adapter Control Review | Implemented as local deterministic broadcast-control metadata review; real broadcast submission deferred | +0% direct | Adds `Web3BroadcastAdapterControlReviewRequest` and `Web3BroadcastAdapterControlReviewReport` for raw-transaction-serialization prerequisite coherence, adapter/operator/audit-state references, kill-switch, rate-limit, replay/idempotency controls, broadcast-permission denial, audit replay, and SQLite checkpoint reopen without RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |
| 48 | Local Web3 Sandbox/Live Discrepancy Calibration | Implemented as local deterministic caller-supplied reference/tolerance calibration; external sandbox/live evidence deferred | +0% direct | Adds `Web3SandboxLiveDiscrepancyCalibrationRequest` and `Web3SandboxLiveDiscrepancyCalibrationReport` for broadcast-control prerequisite coherence, sanitized sandbox/live observation references, minimum sample counts, price/latency/fee discrepancy bounds, audit replay, and SQLite checkpoint reopen without external calls, credentials, RPC calls, signer material, signing, broadcast, live execution, or readiness claims. |

Potential total inside ChatGPT Project Mode: approximately 75-96% of code/documentation readiness, but not full production readiness because live infrastructure, external exchange credentials, real deployment, penetration testing, deployment-host audit validation, external sandbox/live calibration evidence, deployment-host durability validation, and live trading verification are environment-limited.

## Phase 0 Ã¢â‚¬â€ Governance Initialization

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

## Phase 1 Ã¢â‚¬â€ Rust Workspace Scaffold

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

## Phase 2 Ã¢â‚¬â€ Config, Secrets, and Mode Gates

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
- Added local authenticated keystore file loading for test/local entries using versioned XChaCha20-Poly1305 payloads with alias-bound associated data and tamper-rejection tests.
- Added local keystore entry preflight reports that validate alias entry metadata, authenticated `v1` payload shape, hex salt/nonce/ciphertext lengths, and no-material-loaded/no-plaintext-decrypted side-effect flags without live credentials or production custody claims.
- Added local non-mutating secret rotation planning records for distinct keystore aliases with audit/state recovery tests and no material loading, plaintext decryption, keystore entry writes, external revocation, or production custody claims.
- Added `arb-agent validate-secret-boundary-audit --workspace <fresh-dir>` to replay local secret-rotation plan audit records, recover the SQLite WAL rotation-plan checkpoint, and prove invalid material-loading audit records plus state-write failures fail closed without loading material, decrypting plaintext, writing keystore entries, revoking external credentials, or claiming production readiness.
- Added local signer secret-scope review records that require approved keystore alias, strategy id, and chain references before future signer work can proceed, with audit/state recovery tests and no key loading, plaintext decryption, signing, broadcasts, RPC calls, or production custody claims.
- Added `arb-agent validate-signer-boundary-audit --workspace <fresh-dir>` to replay local signer request and signer secret-scope audit records, recover SQLite WAL checkpoints, and prove invalid audit/state-write paths fail closed without custody or signing side effects.
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

## Phase 3 Ã¢â‚¬â€ Policy Engine and Trust Contract

### Status

Implemented in ChatGPT Project Mode; current workspace Rust/CI validation evidence exists and must be refreshed after changes.

### Goal

Implement deterministic deny-by-default policy enforcement before any live execution features exist.

### Completed Deliverables

- `PHASE_3_SUBROADMAP.md`
- `crates/arb-core/src/policy.rs`
- Policy exports from `crates/arb-core/src/lib.rs`
- CLI policy initialization message after config load
- Local policy-decision audit/checkpoint CLI validation with fail-closed audit/state probes
- Local withdrawal policy boundary CLI validation with fail-closed config/strategy/policy/destination/signing/audit-state probes
- Local execution-adapter audit/checkpoint CLI validation with fail-closed adapter-run and recovery-plan probes
- Local destination allowlist/ownership-reference audit checkpoint CLI validation with fail-closed audit/state probes
- Updated `scripts/validate_structure.py`
- Policy unit tests drafted for approval and denial paths

### Policy Coverage Implemented

- Runtime mode checks
- Observe/paper/live scope checks
- Live runtime unavailable by default
- Kill-switch denial
- Withdrawal denial across config, strategy profile, policy trust contract, destination allowlist, and signer-reference boundaries
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
- Destination allowlist audit/checkpoint replay and fail-closed validation
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

## Phase 4 Ã¢â‚¬â€ Audit Journal and State Store

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

## Phase 5 Ã¢â‚¬â€ Market Data Core

### Status

Implemented in ChatGPT Project Mode as model and trait boundaries with current workspace Rust/CI validation evidence; live provider validation remains deferred.

### Goal

Implement normalized quote, order book, fee, and freshness models.

### Completed Tasks

- Created `PHASE_5_SUBROADMAP.md`.
- Added `arb-core::market_data` with normalized market pairs, price levels, top-of-book quotes, order-book snapshots, freshness classification, market-data requests, provider capabilities, and `MarketDataProvider` trait.
- Added `arb-core::fees` with liquidity roles, fee schedules, fee estimates, fee-adjusted edge calculation, and `FeeProvider` trait.
- Added local market-data provider preflight, reconnect/backoff, quality-assessment, historical-persistence, and paid-provider evaluation audit/checkpoint helpers plus `arb-agent validate-market-data-boundary-audit --workspace <fresh-dir>` and `arb-agent validate-market-data-history-persistence --workspace <fresh-dir>` for replay/reopen/fail-closed validation without live provider side effects.
- Added local fee verification audit/checkpoint helpers plus `arb-agent validate-fee-boundary-audit --workspace <fresh-dir>` for replay/reopen/fail-closed validation without provider API, RPC, credential, or account-query side effects.
- Exported market-data and fee primitives through `arb-core`.
- Updated `arb-agent` status output to report market-data boundary availability without starting network providers.
- Updated structure validation to require Phase 5 files.

### Deferred Tasks

- Live REST/WebSocket provider implementations.
- Paid data-provider integration.
- Exchange-specific fee schedule validation.
- Live/provider-backed market-data latency, WebSocket reconnect, provider-side rate-limit, and quality validation.
- Opportunity-engine consumption of market-data models.

### Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Additional local boundary validation:

```bash
cargo run -p arb-agent -- validate-market-data-boundary-audit --workspace <fresh-dir>
cargo run -p arb-agent -- validate-fee-boundary-audit --workspace <fresh-dir>
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

## Phase 6 Ã¢â‚¬â€ Simulated/Paper Connectors

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

## Phase 7 Ã¢â‚¬â€ CEX Connector Framework

### Status

Implemented as typed framework boundary; current workspace Rust/CI validation covered and live exchange validation deferred.

### Goal

Define exchange connector traits, venue capability profiles, order request models, and policy-gated CEX validation without adding exchange-specific live adapters.

### Completed Tasks

- Created `PHASE_7_SUBROADMAP.md`.
- Added `arb-core::cex` with CEX venue profiles, connector capabilities, registry, order request models, order side/type/time-in-force enums, and order status boundary.
- Added `CexPolicyGate` to validate paper/sandbox CEX order requests through `PolicyEngine` while blocking live CEX orders in Phase 7.
- Added `arb-agent validate-connector-lifecycle-audit --workspace <fresh-dir>` to replay local/mock CEX lifecycle and local DEX/Web3 quote/simulation lifecycle audit records, recover SQLite WAL checkpoints, and prove invalid audit/state-write paths fail closed without exchange/RPC/signing/broadcast side effects.
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
- Real exchange-specific audit/state integration for future sandbox/live CEX order lifecycle events.

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

Met for ChatGPT Project Mode CEX framework boundary implementation with current workspace Rust/CI validation evidence. Exchange-specific adapter work, sandbox testing, external credential/account validation, provider-backed rate-limit validation, and external fee/terms/jurisdiction/API-capability/incident review remain required before any live CEX use.

## Phase 8 Ã¢â‚¬â€ DEX/Web3 Connector Framework

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

## Phase 9 Ã¢â‚¬â€ Opportunity Engine

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
- Added same-venue triangular path search over supplied local quotes and fee schedules.
- Added local replay corpus and false-positive expectation reports over supplied local records.
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

- Larger curated opportunity scenario corpora beyond the built-in CI replay gate.
- Broader inventory-aware sizing validation.
- Broader depth-aware slippage validation.
- External cross-venue transfer latency and settlement-risk calibration.
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

## Phase 10 Ã¢â‚¬â€ Execution Planner

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
- Added local execution-plan draft and per-intent policy-outcome audit records for deterministic planner handoff review.
- Added `arb-agent validate-execution-planner-audit --workspace <fresh-dir>` to verify local plan-draft/policy-outcome audit replay, SQLite checkpoint recovery, and fail-closed invalid-audit/state-write behavior without enabling execution.
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

Met for ChatGPT Project Mode execution-planner model boundary implementation with current workspace Rust/CI validation evidence. Local runtime lifecycle wiring now journals plan drafts plus per-intent policy outcomes and checkpoints plans before deterministic adapter-boundary evaluation. Signer/custody integration, live connector validation, restart replay, and production runtime validation remain required before any use with real funds.

## Phase 11 Ã¢â‚¬â€ Execution Adapters

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

## Phase 12 Ã¢â‚¬â€ Communications and CLI

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
- `ChannelAdapterValidationRequest`
- `ChannelAdapterValidationReport`

### Completed

- Added typed non-secret channel profiles.
- Added local CLI command parsing boundary.
- Added deterministic operator-command routing records.
- Rejected live execution, withdrawal, bridge, signing, and broadcast commands.
- Added typed notification payload and dispatch records.
- Added deterministic local notification publisher boundary.
- Added local authenticated channel-adapter validation records connecting ready remote envelopes to local dispatch records without delivery.
- Added local channel-session validation summaries for accepted, unauthenticated, replayed, and provider-unavailable adapter outcomes without outbound delivery.
- Added mocked platform command-ingress validation and remote-command envelope command-injection marker detection plus local platform-adapter control review records for token-reference metadata, raw-token-material denial, platform identity authorization, channel permission, command-injection blocking, token revocation, provider rate-limit, and provider outage outcomes without token storage, platform calls, or delivery.
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

Met for ChatGPT Project Mode communications and CLI model/trait boundary implementation with local command-source authorization, local remote-command security review, local mocked platform command-ingress validation, local remote-command envelope validation with command-injection marker detection, local authenticated channel-adapter validation, local channel-session validation summaries, local platform-adapter control reviews, local notification rate-limit/outage gating, local audit/state checkpoint helpers, repeatable `validate-communications-runtime` CLI audit/SQLite reopen validation for route/review/platform-ingress/envelope/channel-adapter/channel-session/platform-adapter/notification records, local runtime-smoke review/platform-ingress/envelope/channel-adapter/channel-session/platform-adapter checkpoint recovery, deployment-host runtime report wrapper support, and current workspace Rust/CI validation evidence. Real messaging adapters, platform authentication, platform identity authorization, platform-token storage, provider-side rate-limit reconciliation, real outage detection, real delivery validation, and production runtime operator UX validation remain required before production use.

## Phase 13 Ã¢â‚¬â€ Embedded Dashboard

### Status

Implemented for ChatGPT Project Mode as a deterministic embedded-dashboard model/trait boundary.

### Goal

Provide optional lightweight local dashboard boundaries without starting a persistent production web server or exposing a public network surface.

### Completed Tasks

- Created `PHASE_13_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/dashboard.rs`.
- Added dashboard boundary version marker.
- Added dashboard config and loopback-only server-binding model.
- Added snapshot, panel, item, severity, render-request, and render-record models.
- Added `DashboardRenderer` trait and `DeterministicDashboardRenderer`.
- Added fail-closed rejection for HTTP server startup, public exposure, non-loopback bind hosts, live controls, and secret rendering.
- Added secret-like display redaction for local render records.
- Added hosted-dashboard security review, hosted-request preflight, bounded one-shot authenticated loopback hosted-request validation that serves sanitized rendered-dashboard body content with byte/digest metadata, and local hosted-session validation summaries for accepted, unauthenticated, CSRF-rejected, and rate-limited request accounting.
- Exported dashboard types from `arb-core`.
- Surfaced the dashboard boundary version in `arb-agent` status output.
- Updated structure validator for Phase 13 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Real dashboard hosting.
- Persistent or production HTTP server startup.
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

Met for ChatGPT Project Mode embedded-dashboard model/trait boundary implementation with current workspace Rust/CI validation evidence. Real dashboard hosting, production authentication/session design, CSRF token serving, daemon secure-header serving, daemon rate limiting, UX validation, and penetration testing remain required before production dashboard use.

## Phase 14 Ã¢â‚¬â€ Observability and Runbooks

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

Met for ChatGPT Project Mode observability and runbook model/trait boundary implementation with current workspace Rust/CI validation evidence, including scoped local tracing subscriber capture and sandbox-only observability log retention/rotation execution. Daemon-wide/deployment-host tracing/logging subscriber installation, Prometheus/OpenTelemetry exporters, authenticated metrics endpoint design, alert routing, deployment-host log retention/rotation validation, incident drills, and production runtime validation remain required before production observability use.

## Phase 15 Ã¢â‚¬â€ Testing, Fuzzing, and Backtesting

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

## Phase 16 Ã¢â‚¬â€ Packaging and Deployment

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
- Added `deployment/container/Containerfile.production` and `scripts/validate_production_container.py` for a production-intent local/CI container build, Trivy image scan, critical-vulnerability enforcement, bounded Docker command timeouts, Dockerized Trivy no-pull/internal-timeout controls with timed-out scan-container cleanup, fail-closed unavailable-Docker reporting, inert CLI help smoke path, and hardened read-only/no-network help smoke path without pushing images, installing services, loading secrets, or claiming readiness.
- Added bounded timeout behavior to `scripts/validate_arm_cross_check.py` and optional `systemd-analyze verify` execution so missing or stalled deployment prerequisites fail closed instead of hanging validation.
- Added `scripts/validate_arm_cross_check.py` plus CI cross-compiler setup for `cargo check --workspace --target aarch64-unknown-linux-gnu --locked`, with bounded Docker fallback when the host cross compiler is unavailable, without running ARM binaries, inspecting devices, installing services, or claiming ARM deployment readiness.
- Added `scripts/validate_release_artifact.py` plus CI upload of `arbyclaw-release-artifact` for an unsigned locked release binary, SHA-256 manifest, unsigned provenance record, and local bundle-integrity verification without signing, attestation upload, publishing, deployment, or production-readiness claims.
- Added `scripts/validate_packaging_deployment_gate.py` plus CI aggregate gate execution so Phase 16 now composes release-artifact, systemd example, static hardening, ARM profile, and ARM cross-target validation through one local bounded packaging/deployment report while preserving the documented host-or-Docker ARM toolchain fallback path.
- Exported packaging/deployment types from `arb-core`.
- Surfaced the packaging/deployment boundary version in `arb-agent` status output.
- Updated structure validator for Phase 16 files.
- Updated governance docs and gap tracker.

### Explicitly Not Implemented

- Live trading.
- Image pushing, service installation, runtime deployment, or production-readiness validation.
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
python3 scripts/validate_packaging_deployment_gate.py --json
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

## Phase 17 Ã¢â‚¬â€ External Production Hardening

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
- `scripts/validate_hardening_core_gate.py`
- Structure validator updated for Phase 17 files
- CLI status text updated

### Validation Completed

- `python3 scripts/validate_structure.py`
- `python3 -m py_compile scripts/validate_structure.py`
- `python3 scripts/validate_hardening_core_gate.py --json`

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

## Phase 18 Ã¢â‚¬â€ Agentic Handoff Package

### Status

Implemented for ChatGPT Project Mode.

### Goal

Create final handoff instructions, prompts, and checklists for external coding agents and human maintainers while preserving all unresolved gaps and live-funds blockers.

### Completed Tasks

- Created `PHASE_18_SUBROADMAP.md` before implementation.
- Added `crates/arb-core/src/handoff.rs` deterministic handoff package model/trait boundary.
- Added conservative handoff package construction with authoritative files, completed phases, unresolved gaps, live-funds blockers, and non-executing artifacts.
- Added local review records that explicitly preserve `external_agents_executed = false`, `external_validation_claimed = false`, `production_ready = false`, `live_funds_approved = false`, `public_exposure_approved = false`, and `secret_material_recorded = false`.
- Added local handoff review audit journal and SQLite WAL checkpoint helpers plus `arb-agent validate-agentic-handoff-audit --workspace <fresh-dir>` to replay sanitized handoff-review records without external-agent execution.
- Added `handoff/AGENTIC_HANDOFF_PACKAGE.md`.
- Added `handoff/FUTURE_AGENT_PROMPTS.md`.
- Added `handoff/EXTERNAL_VALIDATION_CHECKLIST.md`.
- Added `scripts/validate_agentic_handoff_candidate_gate.py`.
- Exported handoff types from `arb-core`.
- Surfaced Phase 18 status in `arb-agent`.
- Updated structure validator for Phase 18 files.

### Deferred Tasks

- External agents were not executed.
- Rust/Cargo validation has current local and GitHub Actions evidence for the present workspace state.
- A repeatable local handoff-candidate aggregate gate now validates the handoff audit, hardening core, and external-validation checklist without external agent execution.
- CI, release build, dependency audit, dependency license policy validation, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing now run in GitHub Actions. Staging deployment, load testing, penetration testing, rollback drills, incident drills, exchange/RPC sandbox validation, custody review, compliance review, and production readiness review remain external.

### Acceptance Criteria

Met for deterministic handoff package records and documentation only. This phase adds no production readiness and does not approve live funds, public exposure, production deployment, or autonomous live execution.

### Next Required Action

Keep Rust/CI validation current and continue production-hardening evidence review plus external validation closure before any production, live-funds, public-service, or production-readiness claim.

## Phase 19 Ã¢â‚¬â€ Runtime Lifecycle Wiring

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
- No wallet custody, OS keyring integration, secret rotation, or signer-scoped production secret use.
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

## Phase 20 Ã¢â‚¬â€ SQLite WAL Durability Validation

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
- No wallet custody, OS keyring integration, secret rotation, or signer-scoped production secret use.
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

## Phase 21 Ã¢â‚¬â€ Paper Balance Ledgering

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
- No wallet custody, OS keyring integration, secret rotation, or signer-scoped production secret use.
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

## Phase 22 Ã¢â‚¬â€ Crash/Restart Durability Validation

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
- No wallet custody, OS keyring integration, secret rotation, or signer-scoped production secret use.
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
- No wallet custody, OS keyring integration, secret rotation, or signer-scoped production secret use.
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
- Added Rust tests covering paper intent/report/ledger mutation audit records and journal replay.
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
- Added `scripts/validate_systemd_lifecycle.py` for manual non-secret systemd lifecycle planning and bounded read-only deployment-host inspection without installing, enabling, reloading, starting, stopping, or restarting services.
- Added `scripts/validate_deployment_host_runtime.py` to combine non-mutating systemd lifecycle evidence through a bounded helper call with optional explicit local runtime-smoke, graceful-shutdown, backup/restore, backup/restore concurrent-load, runtime permission-denial, incomplete-recovery, restart-recovery, and process-supervised restart execution against fresh workspaces.
- Added `scripts/validate_rollback_drill.py` to validate sanitized rollback-drill metadata without changing services, files, deployments, or runtime state.
- Added `scripts/validate_incident_response_drill.py` to validate sanitized incident-response drill metadata without changing services, files, alert routes, deployments, or runtime state.
- Added `scripts/validate_deployment_evidence_bundle.py` to summarize non-mutating local validation helpers, including the aggregate deployment-runtime, opportunity-scenario, and connector-scenario gates, without embedding full artifact contents or changing services, files, alert routes, deployments, or runtime state.
- Added `scripts/validate_deployment_evidence_checklist.py` to map the remaining production evidence categories to sanitized external locators or explicit missing statuses without embedding artifact contents or claiming readiness.
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

Met for local deterministic audit append/replay, crash-like truncation rejection, tamper rejection, concurrent append replay, invalid filesystem fail-closed validation, simulated disk-full fail-closed validation, side-effect-free retention/rotation planning, side-effect-free stale-lock restart recheck planning, local runtime smoke validation, manual non-mutating systemd lifecycle plan/inspect tooling, combined deployment-host runtime report tooling with non-mutating audit/state filesystem and audit retention active/archive preflight reporting, non-mutating rollback-drill evidence tooling, non-mutating incident-response drill evidence tooling, non-mutating deployment evidence bundle indexing, and non-mutating deployment evidence checklist validation only. Deployment-host audit validation, physical disk-full evidence, retention/rotation execution validation, operator-controlled service-manager lifecycle execution validation, rollback execution validation, incident-response execution validation, live exchange/RPC validation, custody/signing validation, and production readiness are not claimed.

## Phase 28 - Deployment Runtime Aggregate Gate

### Status

Implemented for local deterministic aggregate deployment-runtime validation only.

### Goal

Compose existing local runtime and deployment helper probes into one gate that verifies their combined report preserves safety invariants across runtime smoke, audit durability, sandbox retention execution, graceful shutdown, backup/restore, backup/restore load, restart recovery, incomplete recovery, supervised restart, permission denial, blocked state/audit preflights, filesystem and retention preflights, communications, dashboard, observability, and panic-hook runtime checks.

### Completed Tasks

- Created `PHASE_28_SUBROADMAP.md`.
- Added `scripts/validate_deployment_runtime_gate.py`.
- The aggregate gate runs the existing `scripts/validate_deployment_host_runtime.py` helper with 18 local-only runtime/deployment components enabled against fresh `target/` workspaces, layers thirteen sanitized runtime/deployment transcript/rehearsal validators on top for a 31-probe aggregate, and explicitly enforces the local production-runtime preflight status, blocker count, and no-readiness evidence fields emitted by runtime smoke.
- The gate fails closed if any nested report claims service-manager action, external calls, live execution, secret loading, public exposure, telemetry export, outbound alert/network delivery, production-path mutation, or production readiness.
- Added the aggregate gate to CI after the existing local deployment-runtime report probes.
- Updated structure validation for Phase 28 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, or RPC calls.
- No signing, withdrawals, bridges, broadcasts, wallet custody, or external adapter submission.
- No service installation, systemd reload, enable, start, stop, restart, or deployment-state mutation outside the local `target/` validation workspace.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local aggregate deployment-runtime validation only. Operator-controlled deployment-host service-manager execution, physical disk-full validation, deployment-host retention execution, executed rollback and incident-response drills, actual daemon failure-capture execution, external sandbox/live calibration, live exchange/RPC validation, custody/signing validation, and production readiness remain unclaimed.

## Phase 29 - Opportunity Scenario Aggregate Gate

### Status

Implemented for local deterministic aggregate opportunity scenario-corpus validation only.

### Goal

Compose the existing local opportunity replay, quote-load, provider-ingestion, historical-fixture, planner-handoff, validation-run, property-check, fuzz-corpus replay, and trace-recovery CLI probes into one stronger gate that verifies the current local opportunity scenario corpus remains deterministic and free of live/external side effects.

### Completed Tasks

- Created `PHASE_29_SUBROADMAP.md`.
- Added `scripts/validate_opportunity_scenario_gate.py`.
- The aggregate gate now runs fourteen existing opportunity/testing CLIs and verifies replay iterations pass, quote-load backpressure is exercised, historical fixtures pass, planner handoff trace counts match, strategy replay and profitability tuning stay deterministic, local validation-run planning/checkpoint recovery remains intact, local property-check and fuzz-corpus replay probes stay fully passing, local validation-corpus property checks remain fully passing, local paper backtest replay stays validated with filled/partial/unfilled coverage, local validation coverage review remains ready for local review, and trace recovery reports no missing checkpoints.
- The gate fails closed if any nested command reports external calls, external data downloads, adapter submission, signing or broadcast, live execution, or production readiness.
- Added the aggregate gate to CI after the existing opportunity trace-recovery CLI.
- Updated structure validation for Phase 29 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, RPC, market-data provider, or backtest data calls.
- No adapter submission, signing, withdrawals, bridges, broadcasts, wallet custody, or external execution.
- No production deployment or production-readiness approval.
- No claim that local synthetic/recorded fixtures are external sandbox/live evidence.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local aggregate opportunity scenario-corpus validation only. Broader external/deployment scenario-corpus execution, live/provider-backed market-data validation, external sandbox/live calibration, production runtime validation, live exchange/RPC validation, custody/signing validation, and production readiness remain unclaimed.

## Phase 30 - Connector Scenario Aggregate Gate

### Status

Implemented for local deterministic aggregate connector scenario validation only.

### Goal

Compose the existing local market-data provider preflight, reconnect-plan, market-data boundary audit, fee verification, fee boundary audit, and CEX/DEX connector lifecycle audit CLI probes into one stronger gate that verifies current connector boundary fixtures remain deterministic and free of live/external side effects.

### Completed Tasks

- Created `PHASE_30_SUBROADMAP.md`.
- Added `scripts/validate_connector_scenario_gate.py`.
- The aggregate gate now runs seventeen connector-adjacent CLIs and verifies degraded provider/rate-limit/outage/stale/latency blocking, local provider reconciliation review, local market-data quality scoring, paid-provider dossier review, deterministic historical-batch persistence, reconnect-plan blocking, fee stale-review blocking, CEX and DEX request-plan shapes, CEX balance-snapshot parsing, audit/state fail-closed behavior, and local CEX/DEX lifecycle audit recovery.
- The gate fails closed if any nested command reports live network use, WebSocket connection opening, credential loading, live provider calls, external submission, RPC calls, signing, broadcasts, live execution, or production readiness.
- Added the aggregate gate to CI after the existing connector lifecycle audit CLI.
- Updated structure validation for Phase 30 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, signing, withdrawals, bridges, broadcasts, wallet custody, or external execution.
- No production deployment or production-readiness approval.
- No claim that local deterministic connector fixtures are external sandbox/live evidence.

### Validation

Must be refreshed after this patch:

```bash
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local aggregate connector scenario validation only. Live REST/WebSocket exchange adapters, provider-backed market-data and fee validation, external exchange sandbox/live lifecycle calibration, live DEX/RPC simulation and router validation, custody/signing validation, production deployment-host connector validation, and production readiness remain unclaimed.

## Phase 31 - Local CEX Market-Data Request Plans

### Status

Implemented for local deterministic exchange-specific request-plan validation only.

### Goal

Add typed Binance/Coinbase/Kraken market-data request plans for REST depth/book and WebSocket depth/book subscription shapes, and validate those plans against caller-supplied local transcripts without performing network calls.

### Completed Tasks

- Created `PHASE_31_SUBROADMAP.md`.
- Added `CexMarketDataRequestKind` and `CexMarketDataRequestPlan`.
- Added Binance, Coinbase, and Kraken REST/WebSocket request-plan constructors.
- Added fail-closed request-plan validation for malformed REST/WebSocket shapes and side-effect flags.
- Added request-plan transcript parsing that requires format, venue, and pair agreement before normalizing a supplied local transcript.
- Added Rust tests for exchange-specific REST/WebSocket shapes, local transcript parsing, side-effect denial, and plan/transcript mismatch denial.
- Added `arb-agent validate-cex-market-data-request-plans`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Updated structure validation for Phase 31 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, signing, withdrawals, bridges, broadcasts, wallet custody, or external execution.
- No production deployment or production-readiness approval.
- No claim that local request plans are live adapter implementations.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-cex-market-data-request-plans
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local exchange-specific CEX market-data request-plan modeling only. Live REST/WebSocket clients, credentialed account calls, sandbox/live provider validation, order submission/cancel adapters, deployment-host connector validation, custody/signing validation, and production readiness remain unclaimed.

## Phase 32 - Local DEX/Web3 Request Plans

### Status

Implemented for local deterministic DEX/Web3 request-plan validation only.

### Goal

Add typed local request plans for future DEX/router/RPC quote and simulation adapter shapes, and validate those plans through existing local quote/simulation request boundaries without performing HTTP calls, RPC calls, signing, broadcasts, bridges, credential loading, or live execution.

### Completed Tasks

- Created `PHASE_32_SUBROADMAP.md`.
- Added `DexRequestPlanKind` and `DexRequestPlan`.
- Added local Uniswap V3 quoter, 0x quote, Jupiter quote, and EVM simulation request-plan constructors.
- Added fail-closed validation for malformed HTTP/RPC shapes, invalid venue/pair metadata, and side-effect flags.
- Added quote-capable and simulation-capable conversion into existing local DEX request records.
- Added Rust tests for request-plan counts, local request conversion, side-effect denial, and wrong-capability conversion denial.
- Added `arb-agent validate-dex-request-plans`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Updated structure validation for Phase 32 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, signing, withdrawals, bridges, broadcasts, wallet custody, or external execution.
- No production deployment or production-readiness approval.
- No claim that local request plans are live adapter implementations.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-request-plans
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local DEX/Web3 request-plan modeling only. Live RPC clients, router/aggregator integrations, transaction simulation providers, production nonce handling, custody-backed signing, broadcasts, bridges, sandbox/live provider validation, deployment-host connector validation, and production readiness remain unclaimed.

## Phase 33 - Local DEX/Web3 Response Transcript Parsing

### Status

Implemented for local deterministic DEX/Web3 response transcript parsing only.

### Goal

Parse caller-supplied local DEX/router/RPC response transcript JSON for the Phase 32 request-plan shapes into existing local quote and simulation response records without performing HTTP calls, RPC calls, credential loading, signing, broadcasts, bridges, or live execution.

### Completed Tasks

- Created `PHASE_33_SUBROADMAP.md`.
- Added `DexResponseTranscript` for local response transcript metadata and JSON payloads.
- Added fail-closed transcript validation for malformed metadata, malformed JSON, side-effect flags, and request-plan mismatch.
- Added local parsing for Uniswap V3 quoter-style `eth_call`, 0x quote HTTP, Jupiter quote HTTP, and EVM simulation `eth_call` payload shapes.
- Added conversion into existing `DexSwapQuoteResponse` and `Web3TransactionSimulationResponse` records.
- Added Rust tests for quote transcript parsing, simulation transcript parsing, side-effect denial, and request-kind mismatch denial.
- Added `arb-agent validate-dex-response-transcripts`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.
- Updated structure validation for Phase 33 files.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, signing, withdrawals, bridges, broadcasts, wallet custody, or external execution.
- No production deployment or production-readiness approval.
- No claim that local response transcripts are live adapter responses.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-response-transcripts
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local DEX/Web3 response transcript parsing only. Live RPC clients, router/aggregator integrations, transaction simulation providers, production nonce handling, custody-backed signing, broadcasts, bridges, sandbox/live provider validation, deployment-host connector validation, and production readiness remain unclaimed.

## Phase 34 - Local CEX Order Lifecycle Transcript Parsing

### Status

Implemented for local deterministic CEX order lifecycle transcript parsing only.

### Goal

Parse caller-supplied local Binance-, Coinbase-, and Kraken-shaped order lifecycle JSON transcripts into existing local CEX lifecycle response and reconciliation records for filled and cancelled-after-partial paths without performing REST calls, opening WebSockets, loading credentials, submitting orders, cancelling orders, or claiming live adapter readiness.

### Completed Tasks

- Created `PHASE_34_SUBROADMAP.md`.
- Added `CexOrderLifecycleTranscript` and `CexOrderLifecycleTranscriptFormat`.
- Added fail-closed transcript validation for metadata, malformed JSON, side-effect flags, validation-record venue/pair mismatch, and unknown statuses.
- Added local parsing for Binance execution-report, Coinbase order-event, and Kraken order-status payload shapes.
- Wired `arb-agent validate-connector-lifecycle-audit` to parse local lifecycle transcripts before reconciliation.
- Added Rust tests for exchange-shaped lifecycle transcript parsing, side-effect denial, and validation mismatch denial.
- Added aggregate connector scenario assertion for parsed CEX lifecycle transcript count.
- Added local cancelled-after-partial lifecycle transcript reconciliation with audit replay, SQLite checkpoint recovery, and aggregate gate assertions.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No adapter submission, credential loading, signing, withdrawals, bridges, broadcasts, wallet custody, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local lifecycle transcripts are live exchange responses.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-connector-lifecycle-audit --workspace <fresh-dir>
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local CEX order lifecycle transcript parsing and local cancelled-after-partial reconciliation only. Live REST/WebSocket clients, credentialed account calls, sandbox/live exchange responses, production idempotency, rate-limit reconciliation, cancel/reconciliation adapters, deployment-host connector validation, and production readiness remain unclaimed.

## Phase 35 - Local CEX Balance Snapshot Transcript Parsing

### Status

Implemented for local deterministic CEX balance snapshot transcript parsing only.

### Goal

Parse caller-supplied local Binance-, Coinbase-, and Kraken-shaped balance snapshot JSON into normalized local CEX balance records without performing REST calls, opening WebSockets, loading credentials, querying account state, mutating balances, submitting or cancelling orders, or claiming live adapter readiness.

### Completed Tasks

- Created `PHASE_35_SUBROADMAP.md`.
- Added `CexBalanceSnapshotTranscript`, `CexBalanceSnapshotTranscriptFormat`, `CexAssetBalanceSnapshot`, and `CexBalanceSnapshotRecord`.
- Added fail-closed validation for malformed local JSON, duplicate assets, invalid balances, side-effect flags, credential loading, account-state query flags, and production-readiness claims.
- Added local parsing for Binance account balances, Coinbase accounts, and Kraken balance payload shapes.
- Added Rust tests for successful local exchange-shaped balance parsing, side-effect denial, and duplicate-asset denial.
- Added `arb-agent validate-cex-balance-snapshots`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, or backtest data calls.
- No account state queries, credential loading, balance mutation, adapter submission, signing, withdrawals, bridges, broadcasts, live order submission, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local balance snapshots are live exchange account reads.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-cex-balance-snapshots
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local CEX balance snapshot transcript parsing only. Authenticated live balance reads, credentialed account calls, sandbox/live exchange account validation, balance reconciliation against real venues, live REST/WebSocket adapters, and production readiness remain unclaimed.

## Phase 36 - Local DEX/Web3 Transaction Lifecycle Transcript Parsing

### Goal

Parse caller-supplied local EVM transaction receipt and Solana signature-status JSON into normalized local Web3 transaction lifecycle records with nonce and confirmation accounting, without performing RPC calls, loading credentials, loading signer material, signing, broadcasting, bridging, submitting transactions, or claiming live adapter readiness.

### Completed Tasks

- Created `PHASE_36_SUBROADMAP.md`.
- Added `Web3TransactionLifecycleTranscript`, `Web3TransactionLifecycleTranscriptFormat`, `Web3TransactionLifecycleRecord`, and `Web3TransactionLifecycleStatus`.
- Added local parsing for EVM transaction receipt/status payloads and Solana signature-status payloads.
- Added fail-closed validation for side-effect flags, live RPC response flags, signer material loading, signing, broadcast, bridge, live execution, production-readiness claims, and confirmed statuses without local confirmation evidence.
- Added Rust tests for successful local lifecycle parsing, nonce tracking, confirmation accounting, side-effect denial, and missing-confirmation denial.
- Added `arb-agent validate-dex-transaction-lifecycle-transcripts`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, signer, wallet, bridge, or backtest data calls.
- No credential loading, signer material loading, transaction construction, account query, adapter submission, signing, withdrawals, bridges, broadcasts, live order submission, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local transaction lifecycle records are live RPC, testnet, mainnet, signer, or broadcast validation.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-transaction-lifecycle-transcripts
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local DEX/Web3 transaction lifecycle transcript parsing only. Live RPC adapters, custody-backed signing, transaction construction, nonce management against live chains, broadcast controls, testnet/mainnet simulation replay, external confirmation reconciliation, deployment restart recovery, and production readiness remain unclaimed.

## Phase 37 - Local DEX/Web3 Protocol Risk Review

### Goal

Evaluate caller-supplied local DEX/Web3 protocol metadata for spender approval hygiene, gas and slippage limits, MEV controls, token metadata review, and protocol terms review without performing RPC calls, loading credentials, loading signer material, signing, broadcasting, bridging, submitting transactions, or claiming live adapter readiness.

### Completed Tasks

- Created `PHASE_37_SUBROADMAP.md`.
- Added `DexProtocolRiskReviewRequest`, `DexProtocolRiskReviewReport`, and `DexProtocolRiskReviewStatus`.
- Added deterministic local review logic for router and spender allowlisting, unlimited allowance denial, approval revocation planning, gas/slippage caps, MEV risk limits, public-mempool mitigation review, token metadata review, and protocol terms review.
- Added Rust tests for ready local metadata, blocked local metadata, and side-effect denial.
- Added `arb-agent validate-dex-protocol-risk-review`.
- Wired the CLI into CI and `scripts/validate_connector_scenario_gate.py`.

### Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, REST, WebSocket, RPC, market-data provider, fee-provider, DEX/router, signer, wallet, bridge, or backtest data calls.
- No credential loading, signer material loading, allowance submission, approval transaction construction, account query, adapter submission, signing, withdrawals, bridges, broadcasts, live order submission, live cancellation, or external execution.
- No production deployment or production-readiness approval.
- No claim that local protocol risk reviews are live contract, RPC, spender, MEV, testnet, mainnet, signer, or broadcast validation.

### Validation

Must be refreshed after this patch:

```bash
cargo run -p arb-agent -- validate-dex-protocol-risk-review
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local DEX/Web3 protocol risk review only. Live RPC adapters, custody-backed signing, transaction construction, real spender/allowance checks, live gas estimation, external MEV validation, protocol contract review, testnet/mainnet validation, broadcast controls, deployment restart recovery, and production readiness remain unclaimed.

## Phase 59 - Service-Manager Concurrent Lifecycle Transcript Hardening

### Goal

Require sanitized service-manager lifecycle transcript evidence to account for service-manager-controlled concurrent lifecycle validation before local transcript status can be ready for external review, without performing service-manager actions or changing deployment behavior.

### Completed Tasks

- Created `PHASE_59_SUBROADMAP.md`.
- Added concurrent lifecycle reference, worker-count, and success fields to `RuntimeServiceManagerLifecycleTranscript`.
- Added matching fields to `RuntimeServiceManagerLifecycleTranscriptReport`.
- Required concurrent lifecycle evidence with at least two workers for ready service-manager transcript status.
- Added `missing-concurrent-lifecycle-evidence` as a fail-closed blocker.
- Surfaced the new fields in `arb-agent validate-service-manager-lifecycle-transcript`.
- Updated `scripts/validate_deployment_runtime_gate.py` so the aggregate deployment-runtime gate enforces the new transcript fields.

### Explicit Non-Goals

- No live trading.
- No service-manager start, stop, restart, reload, enable, or install action.
- No deployment path mutation.
- No real exchange, sandbox, REST, WebSocket, RPC, signer, wallet, or credential calls.
- No signing, withdrawals, bridges, broadcasts, adapter submission, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core service_manager_lifecycle_transcript -- --nocapture
cargo run -p arb-agent -- validate-service-manager-lifecycle-transcript
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local sanitized service-manager concurrent lifecycle transcript enforcement only. Operator-controlled deployment-host service-manager execution, real deployment-host concurrent lifecycle validation, physical disk-full validation, deployment-host retention execution, executed rollback and incident-response drills, actual daemon failure-capture execution, external sandbox/live calibration, live exchange/RPC validation, custody/signing validation, and production readiness remain unclaimed.

## Phase 60 - Deployment Permission Runtime-Write Evidence Hardening

### Goal

Require sanitized deployment permission transcript evidence to account for runtime-write permission-denial behavior before local transcript status can be ready for external review, without changing deployment permissions, mutating deployment paths, or executing service-manager actions.

### Completed Tasks

- Created `PHASE_60_SUBROADMAP.md`.
- Added runtime-write attempt reference, runtime-write permission-denied, and runtime-write error-classification fields to `RuntimeDeploymentPermissionTranscript`.
- Added matching fields to `RuntimeDeploymentPermissionTranscriptReport`.
- Required runtime-write permission-denial evidence for ready deployment permission transcript status.
- Added fail-closed blockers for missing runtime-write attempt reference, missing runtime-write permission-denial evidence, and missing runtime-write error classification.
- Surfaced the new fields in `arb-agent validate-deployment-permission-transcript`.
- Updated `scripts/validate_deployment_runtime_gate.py` so the aggregate deployment-runtime gate enforces the new transcript fields.

### Explicit Non-Goals

- No live trading.
- No permission changes or production path mutation.
- No service-manager start, stop, restart, reload, enable, or install action.
- No real exchange, sandbox, REST, WebSocket, RPC, signer, wallet, or credential calls.
- No signing, withdrawals, bridges, broadcasts, adapter submission, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_permission_transcript -- --nocapture
cargo run -p arb-agent -- validate-deployment-permission-transcript
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local sanitized deployment permission runtime-write evidence enforcement only. Real deployment-host permission-denial execution during runtime writes, service-manager-controlled runtime execution, physical disk-full validation, deployment-host retention execution, executed rollback and incident-response drills, actual daemon failure-capture execution, external sandbox/live calibration, live exchange/RPC validation, custody/signing validation, and production readiness remain unclaimed.

## Phase 61 - Execution Adapter Submission Preconditions

### Goal

Require deterministic execution-adapter config and durable adapter-run records to preserve non-secret future-submission preconditions for kill-switch, audit/state preflight, and idempotency before any future live adapter can be wired.

### Completed Tasks

- Created `PHASE_61_SUBROADMAP.md`.
- Added future-submission precondition fields to `ExecutionAdapterConfig`.
- Required kill-switch, audit/state preflight, and idempotency preconditions in config validation.
- Added matching durable fields to `ExecutionAdapterRunRecord`.
- Required adapter run validation to fail closed if any future-submission precondition is absent.
- Added precondition fields to execution-adapter audit metadata and `arb-agent validate-execution-adapter-audit` output.
- Updated `scripts/validate_execution_path_gate.py` so the aggregate execution-path gate enforces those fields.
- Bumped `EXECUTION_ADAPTER_FRAMEWORK_VERSION` for the updated serialized adapter-run shape.

### Explicit Non-Goals

- No live trading.
- No adapter submission.
- No real exchange, sandbox, REST, WebSocket, RPC, signer, wallet, or credential calls.
- No signing, withdrawals, bridges, broadcasts, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core execution_adapter -- --nocapture
cargo run -p arb-agent -- validate-execution-adapter-audit --workspace <fresh-dir>
python3 scripts/validate_execution_path_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic execution-adapter future-submission precondition enforcement only. Sandbox/live adapter reconciliation, real exchange/RPC adapter execution, custody-backed signing, service-manager restart execution, deployment-host runtime validation, and production readiness remain unclaimed.

## Phase 62 - Communications Delivery Preconditions

### Goal

Require deterministic communications channel/platform adapter records to preserve non-secret future outbound-delivery preconditions for delivery kill-switch, audit/state preflight, idempotency, rate-limit controls, outage/backoff controls, and payload redaction before any real messaging adapter can be wired.

### Completed Tasks

- Created `PHASE_62_SUBROADMAP.md`.
- Added future-delivery precondition fields to `ChannelAdapterValidationRequest`, `ChannelAdapterValidationReport`, `PlatformAdapterReviewRequest`, and `PlatformAdapterReviewReport`.
- Required channel and platform adapter validation to fail closed when any future-delivery precondition is absent.
- Added precondition fields to channel-adapter and platform-adapter audit metadata and SQLite checkpoints.
- Surfaced the precondition fields in `arb-agent validate-communications-runtime` output.
- Wired the same preconditions into runtime-smoke communications records.
- Updated `scripts/validate_operator_surface_gate.py` so the aggregate operator-surface gate enforces those fields.

### Explicit Non-Goals

- No outbound network delivery.
- No real messaging platform API calls.
- No platform-token storage or secret handling.
- No live trading, adapter submission, signing, withdrawals, bridges, broadcasts, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core communications -- --nocapture
cargo run -p arb-agent -- validate-communications-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic communications future outbound-delivery precondition enforcement only. Real platform authentication, real platform identity authorization, platform-token handling, provider-side rate-limit reconciliation, real outage detection, outbound message delivery, production operator orchestration, external security review, and production readiness remain unclaimed.

## Phase 63 - Dashboard Hosting Preconditions

### Goal

Require deterministic hosted-dashboard security review records to preserve non-secret future-hosting preconditions for audit/state preflight, session revocation/logout, operator role review, and read-only controls before any real dashboard hosting path can be wired.

### Completed Tasks

- Created `PHASE_63_SUBROADMAP.md`.
- Added future-hosting precondition fields to `DashboardHostedSecurityPolicy`.
- Required hosted-security policy validation to fail closed when any future-hosting precondition is absent.
- Added matching durable fields to `DashboardHostedSecurityReviewReport`.
- Added precondition fields to hosted-security audit metadata and SQLite checkpoints.
- Surfaced the precondition fields in `arb-agent validate-dashboard-runtime` output.
- Wired the same preconditions into runtime-smoke hosted-dashboard security records.
- Updated `scripts/validate_operator_surface_gate.py` so the aggregate operator-surface gate enforces those fields.

### Explicit Non-Goals

- No public dashboard exposure.
- No persistent hosted dashboard server.
- No real authentication/session store, authorization system, or operator role administration.
- No live controls, adapter submission, signing, withdrawals, bridges, broadcasts, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core dashboard -- --nocapture
cargo run -p arb-agent -- validate-dashboard-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic dashboard future-hosting precondition enforcement only. Real dashboard hosting, real auth/session/authorization implementation, CSRF token serving from a live server, public-exposure validation, browser UX validation, command-injection testing, penetration testing, deployment-host orchestration, and production readiness remain unclaimed.

## Phase 64 - Observability Runtime Preconditions

### Goal

Require deterministic observability operations review records to preserve non-secret future-runtime preconditions for audit/state preflight, exporter kill-switch controls, alert authorization, rate-limit/backpressure controls, retry/backoff controls, and non-secret telemetry controls before any real observability exporter, alert, or daemon-hosted runtime path can be wired.

### Completed Tasks

- Created `PHASE_64_SUBROADMAP.md`.
- Added future-runtime precondition fields to `ObservabilityOperationsPolicy`.
- Required operations-review validation to fail closed when any future-runtime precondition is absent.
- Added matching durable fields to `ObservabilityOperationsReviewReport`.
- Added precondition fields to operations-review audit metadata and SQLite checkpoints.
- Surfaced the precondition fields in `arb-agent validate-observability-runtime` output.
- Wired the same preconditions into runtime-smoke observability operations records.
- Updated `scripts/validate_operator_surface_gate.py` so the aggregate operator-surface gate enforces those fields.

### Explicit Non-Goals

- No daemon-hosted metrics runtime.
- No real telemetry exporter sessions.
- No log shipping.
- No outbound alert delivery.
- No daemon-wide tracing or panic-hook installation under deployment orchestration.
- No live controls, adapter submission, signing, withdrawals, bridges, broadcasts, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core observability -- --nocapture
cargo run -p arb-agent -- validate-observability-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic observability future-runtime precondition enforcement only. Real daemon-hosted observability runtime, real exporter sessions, log shipping, real alert delivery, deployment-host retention/rotation execution, incident drills, daemon-wide/deployment-host tracing and panic-hook installation, external AppSec review, and production readiness remain unclaimed.

## Phase 65 - Validation Corpus Breadth Gate

### Goal

Require the deterministic local validation corpus runner to enforce explicit minimum coverage breadth for local validation plans, test cases, fixtures, fuzz corpora, and backtest scenarios before it reports ready for local review.

### Completed Tasks

- Created `PHASE_65_SUBROADMAP.md`.
- Added minimum local corpus breadth fields to `LocalValidationCorpusRequest`.
- Added matching durable fields to `LocalValidationCorpusReport`.
- Required validation corpus reports to fail closed when requested plan, test, fixture, fuzz, or backtest breadth is not met.
- Added breadth fields to validation-corpus audit metadata and SQLite checkpoints.
- Broadened the built-in local validation corpus CLI fixture to three plans with additional local security and backtest regression cases.
- Surfaced the breadth fields in `arb-agent validate-local-validation-corpus` output.
- Updated `scripts/validate_opportunity_scenario_gate.py` so the aggregate opportunity scenario gate enforces reported corpus breadth.
- Added unit coverage for insufficient local validation corpus breadth.

### Explicit Non-Goals

- No external fuzzer invocation.
- No external property-test engine execution beyond current local/Cargo validation.
- No external data downloads or credential-bearing fixture use.
- No production load test, penetration test, or production backtest execution.
- No live controls, adapter submission, signing, withdrawals, bridges, broadcasts, or external execution.
- No production deployment or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core local_validation_corpus -- --nocapture
cargo run -p arb-agent -- validate-local-validation-corpus --workspace <fresh-dir>
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deterministic validation-corpus breadth enforcement only. External fuzz engines, external property-test execution, curated external/deployment replay corpora, production load tests, penetration tests, production backtest evidence, production runtime validation, and production readiness remain unclaimed.

## Phase 66 - SQLite WAL State Schema Migration Guard

### Goal

Add local schema-version migration and fail-closed compatibility checks to the SQLite WAL state store before continuing broader deployment-host durability work.

### Completed Tasks

- Created `PHASE_66_SUBROADMAP.md`.
- Added `SQLITE_WAL_STATE_SCHEMA_VERSION`.
- Added `PRAGMA user_version` schema-version reads.
- Migrated legacy v0 local checkpoint tables to schema v1 while preserving existing rows.
- Rejected future schema versions before state-store reads or writes continue.
- Added schema-version evidence to `SqliteWalDurabilityReport`.
- Surfaced schema migration coverage in `arb-agent` status output.
- Added focused local Rust tests for v0 migration and future-version rejection.

### Explicit Non-Goals

- No production database migration.
- No deployment-host filesystem mutation beyond local temp SQLite test files.
- No real exchange, RPC, wallet, signer, bridge, broadcast, or credential handling.
- No service-manager action, production deployment, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core state::tests::sqlite_wal -- --nocapture
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local SQLite WAL state schema-version migration and fail-closed compatibility checks only. Deployment-host schema migration, deployment-host audit/SQLite recovery execution, production database migration operations, service-manager-controlled runtime validation, and production readiness remain unclaimed.

## Phase 67 - Deployment SQLite Schema Migration Transcript Gate

### Goal

Add a local sanitized transcript validator for deployment-host SQLite schema migration evidence so the deployment runtime gate can account for schema-migration review records without executing migrations or touching deployment paths.

### Completed Tasks

- Created `PHASE_67_SUBROADMAP.md`.
- Added typed deployment SQLite schema migration transcript request/report/status models.
- Required deployment-host evidence, service lifecycle reference, pre-migration backup reference, migration execution reference, schema-version transition evidence, SQLite integrity/checkpoint reopen evidence, audit replay after migration, rollback reference, runtime quiesce/degrade evidence, and operator/reviewer approval before a local transcript reports ready for external review.
- Rejected validator-side effects for migration execution, service-manager actions, deployment-path mutation, secret loading, external submission, live execution, and production-readiness claims.
- Surfaced `arb-agent validate-deployment-sqlite-schema-migration-transcript`.
- Added the transcript validator to deployment runtime and evidence aggregate scripts.
- Added focused local Rust tests for ready, blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No actual deployment-host migration execution.
- No production database migration.
- No service-manager action or deployment path mutation.
- No secret loading, external submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_sqlite_schema_migration -- --nocapture
cargo run -p arb-agent -- validate-deployment-sqlite-schema-migration-transcript
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local sanitized deployment SQLite schema migration transcript validation only. Real operator-controlled deployment-host schema migration execution, deployment-host database migration operations, deployment-host audit/SQLite recovery execution, service-manager-controlled runtime validation, and production readiness remain unclaimed.

## Phase 68 - Local Service-Manager Lifecycle Rehearsal Gate

### Goal

Add a local-only service-manager lifecycle rehearsal validator that proves ordered lifecycle evidence semantics without calling real service managers or changing deployment state.

### Completed Tasks

- Created `PHASE_68_SUBROADMAP.md`.
- Added typed service-manager lifecycle rehearsal request/report/status models.
- Required ordered unit-loaded, started, runtime-smoke, graceful-shutdown, stopped, restarted, and recovery events.
- Required audit replay, SQLite recovery, graceful-shutdown checkpoint, restart recovery, concurrent lifecycle, operator approval, and reviewer approval references.
- Rejected service-manager actions, deployment-path mutation, secret loading, external submission, live execution, and production-readiness claims.
- Surfaced `arb-agent validate-service-manager-lifecycle-rehearsal`.
- Added the rehearsal validator to deployment runtime and evidence bundle aggregate scripts.
- Added focused local Rust tests for ready, blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No `systemctl` calls or real service-manager execution.
- No deployment path mutation.
- No secret loading.
- No external submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core service_manager_lifecycle_rehearsal -- --nocapture
cargo run -p arb-agent -- validate-service-manager-lifecycle-rehearsal
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local ordered service-manager lifecycle rehearsal semantics only. Real operator-controlled deployment-host service-manager execution, production service supervision, deployment-host filesystem behavior, rollback execution, incident-response execution, and production readiness remain unclaimed.

## Phase 69 - Local Deployment Response Drill Rehearsal Gate

### Goal

Compose sanitized rollback execution, incident-response execution, and daemon failure-capture evidence reports into one local-only response drill rehearsal gate without executing rollback, incident actions, service-manager actions, alert delivery, failure injection, file mutation, external calls, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_69_SUBROADMAP.md`.
- Added typed deployment response drill rehearsal request/report/status models.
- Required ready rollback, incident-response, and failure-capture component reports.
- Required shared plan/run identity across composed reports.
- Required component and composed operator/reviewer approvals.
- Rejected rollback execution, incident-response execution, failure injection, service-manager actions, file mutation, alert delivery, external calls, live execution, and production-readiness claims.
- Surfaced `arb-agent validate-deployment-response-drill-rehearsal`.
- Added the rehearsal validator to deployment runtime and evidence bundle aggregate scripts.
- Added focused local Rust tests for ready, blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No rollback execution.
- No incident-response execution.
- No daemon panic/failure injection.
- No service-manager action or deployment mutation.
- No alert delivery, external submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_response_drill_rehearsal -- --nocapture
cargo run -p arb-agent -- validate-deployment-response-drill-rehearsal
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local composed response-drill rehearsal evidence only. Actual rollback execution, actual incident-response execution, daemon failure injection/capture under service orchestration, alert delivery, service-manager execution, deployment-host evidence collection, external review, and production readiness remain unclaimed.

## Phase 70 - Local Runtime Load Profile Review Gate

### Goal

Turn existing local runtime-smoke load evidence into an explicit local runtime load profile review covering caller-supplied local latency/resource budgets and replay-recovery coherence without production benchmark execution, service-manager actions, provider calls, external submission, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_70_SUBROADMAP.md`.
- Added typed runtime load profile review request/report/status models.
- Added local latency budget, resource budget, and replay/recovery evidence checks over runtime-smoke load reports.
- Rejected service-manager actions, external calls, live execution, and production-readiness claims.
- Surfaced the review through `arb-agent validate-runtime-smoke`.
- Added structured enforcement to `scripts/validate_deployment_host_runtime.py`.
- Added aggregate deployment-runtime gate assertions to `scripts/validate_deployment_runtime_gate.py`.
- Added focused local Rust tests for ready, blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No production benchmark execution.
- No deployment-host resource inspection.
- No service-manager action or deployment mutation.
- No live/provider feed, exchange, RPC, signer, adapter submission, or external call.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core runtime_load_profile -- --nocapture
cargo run -p arb-agent -- validate-runtime-smoke --config config.example.toml --workspace target/local-validation/runtime-load-profile-direct --iterations 2
python3 scripts/validate_deployment_host_runtime.py --run-runtime-smoke --runtime-smoke-iterations 2 --runtime-workspace target/local-validation/runtime-load-profile-host --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local runtime-smoke load profile review only. Real production load testing, live/provider quote ingestion load tests, deployment-host resource profiling, ARM or target-class runtime performance evidence, live-feed backpressure validation, dashboard/exporter latency validation, and production readiness remain unclaimed.

## Phase 71 - Local Opportunity Replay Latency Review Gate

### Goal

Turn existing local opportunity replay load evidence into an explicit local latency and throughput review covering replay iteration latency, scenario breadth, and candidate emission counts without external data download, external fuzzers, exchange/RPC calls, adapter submission, signing, broadcast, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_71_SUBROADMAP.md`.
- Added typed opportunity replay latency review request/report/status models.
- Added local latency budget and scenario/candidate throughput checks over opportunity replay load reports.
- Rejected external calls, external data downloads, live execution, and production-readiness claims.
- Surfaced the review through `arb-agent validate-opportunity-replay --iterations <n>`.
- Added aggregate opportunity scenario gate assertions to `scripts/validate_opportunity_scenario_gate.py`.
- Added focused local Rust tests for ready, blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No external fuzzer execution.
- No live/provider market-data download.
- No exchange/RPC calls.
- No adapter submission, signing, broadcast, bridge, withdrawal, or live execution.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core opportunity_replay_latency -- --nocapture
cargo run -p arb-agent -- validate-opportunity-replay --iterations 2
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local opportunity replay latency and throughput review only. Broader external/deployment scenario-corpus execution, external fuzz engines, live/provider-backed market-data validation, deployment-host resource profiling, production load tests, penetration tests, production backtest evidence, and production readiness remain unclaimed.

## Phase 72 - Local Market-Data Provider Latency Review Gate

### Goal

Turn existing local market-data provider preflight, reconnect/backoff, quality-assessment, and paid-provider evaluation evidence into an explicit local latency/backpressure review covering provider receive latency, capture latency, reconnect delay, local sample floor, and unresolved external evidence without live REST/WebSocket providers, provider credentials, exchange/RPC calls, adapter submission, signing, broadcast, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_72_SUBROADMAP.md`.
- Added typed market-data provider latency review request/report/status models.
- Added local provider latency, capture latency, reconnect-delay, quality-score, sample-floor, and remaining-external-evidence checks.
- Rejected live network use, WebSocket opening, credential loading, and production-readiness claims.
- Surfaced the review through `arb-agent validate-market-data-provider-preflight`.
- Added aggregate connector scenario gate assertions to `scripts/validate_connector_scenario_gate.py`.
- Added focused local Rust tests for ready, blocked-budget, and fail-closed side-effect cases.

### Explicit Non-Goals

- No live REST/WebSocket provider implementation.
- No provider-backed latency measurement.
- No provider account, credential, or API-key handling.
- No exchange/RPC calls.
- No adapter submission, signing, broadcast, bridge, withdrawal, or live execution.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core market_data_provider_latency_review -- --nocapture
cargo run -p arb-agent -- validate-market-data-provider-preflight
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local market-data provider latency/backpressure review only. Live REST/WebSocket providers, provider-backed reconnect/rate-limit/outage reconciliation, external latency measurement, paid-provider integration, real market-data quality evidence, deployment-host resource profiling, broader external validation, and production readiness remain unclaimed.

## Phase 73 - Local Validation Coverage Review Gate

### Goal

Turn existing local validation-run, property-check, fuzz-corpus replay, validation-corpus, and paper-backtest evidence into an explicit local validation coverage review covering local breadth and remaining external evidence without external fuzzing engines, external data download, provider calls, exchange/RPC calls, adapter submission, signing, broadcast, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_73_SUBROADMAP.md`.
- Added typed local validation coverage review request/report/status models.
- Added local validation-plan, property-check, fuzz-target, validation-corpus, paper-backtest, and remaining-external-evidence checks.
- Rejected live network use, external fuzzer invocation, live execution submission, signing/broadcast, and production-readiness claims.
- Surfaced the review through `arb-agent validate-local-validation-coverage-review`.
- Added aggregate opportunity scenario gate assertions to `scripts/validate_opportunity_scenario_gate.py`.
- Added focused local Rust tests for ready, missing-breadth, and fail-closed side-effect cases.

### Explicit Non-Goals

- No external fuzzing-engine execution.
- No external data download.
- No provider-backed market-data or exchange/RPC calls.
- No adapter submission, signing, broadcast, bridge, withdrawal, or live execution.
- No production load, security, or external backtest execution.
- No production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core local_validation_coverage_review -- --nocapture
cargo run -p arb-agent -- validate-local-validation-coverage-review
python3 scripts/validate_opportunity_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local validation coverage review only. External fuzz/property execution, broader external/deployment replay corpora, provider-backed market-data validation, production load/security validation, external backtest execution, deployment-host validation, and production readiness remain unclaimed.

## Phase 74 - Local Hosted Dashboard Runtime Readiness Review Gate

### Goal

Turn existing local hosted-dashboard security, request-preflight, one-shot loopback request, and session-validation evidence into an explicit local hosted-dashboard runtime readiness review covering authentication, authorization, CSRF, secure-header, rejection-accounting, loopback serving, and remaining external hosting evidence without persistent server startup, public exposure, live controls, external submission, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_74_SUBROADMAP.md`.
- Added typed hosted dashboard runtime readiness review request/report/status models.
- Added accepted-request, unauthenticated-rejection, CSRF-rejection, rate-limit-rejection, loopback-serving, secure-header, and remaining-external-evidence checks.
- Rejected persistent server startup, public network exposure, live controls, and production-readiness claims.
- Surfaced the review through `arb-agent validate-dashboard-runtime`.
- Added aggregate operator-surface and deployment-host runtime wrapper assertions.
- Added focused local Rust tests for ready, missing-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No persistent dashboard server.
- No public network exposure.
- No real browser session hosting.
- No production authentication/session/authorization implementation.
- No CSRF-token issuing from a daemon.
- No live controls, adapter submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core hosted_dashboard_runtime_readiness -- --nocapture
cargo run -p arb-agent -- validate-dashboard-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local hosted-dashboard runtime readiness review only. Persistent daemon hosting, browser authentication/session handling, CSRF token serving, secure-header serving from a live server, public-exposure validation, penetration testing, deployment-host orchestration, and production readiness remain unclaimed.

## Phase 75 - Local Deployment Backup/Restore Transcript Gate

### Goal

Add a typed local deployment-host backup/restore transcript validator so deployment-runtime evidence can account for service-lifecycle context, backup artifact references, restore execution references, deployment-load references, audit replay/hash-chain continuity, SQLite restore checks, runtime checkpoint restoration, post-restore smoke evidence, rollback references, and reviewer/operator approval without executing backup/restore actions, mutating deployment paths, calling service managers, loading secrets, submitting adapters, performing live execution, or claiming production readiness.

### Completed Tasks

- Created `PHASE_75_SUBROADMAP.md`.
- Added typed deployment backup/restore transcript request/report/status models.
- Added ready/blocked validation with blocker codes for missing backup artifact, restore execution, deployment load, audit restore, SQLite restore, runtime checkpoint restore, post-restore smoke, rollback, runbook, operator, and reviewer evidence references.
- Rejected validator-performed backup/restore execution, service-manager actions, deployment path mutation, secret loading, external submission, live execution, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-deployment-backup-restore-transcript`.
- Added the transcript to the aggregate deployment-runtime gate, deployment evidence bundle, and deployment evidence checklist.
- Added focused local Rust tests for ready, missing-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No real backup execution.
- No real restore execution.
- No service-manager actions.
- No deployment path mutation.
- No secret loading.
- No external calls, adapter submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_backup_restore_transcript -- --nocapture
cargo run -p arb-agent -- validate-deployment-backup-restore-transcript
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deployment backup/restore transcript validation only. Actual deployment-host backup execution, restore execution, service-manager-controlled recovery, deployment-load restore validation, production audit/SQLite recovery, rollback execution, and production readiness remain unclaimed.

## Phase 76 - Local Deployment Graceful-Shutdown Transcript Gate

### Goal

Add a typed local deployment-host graceful-shutdown transcript validator so deployment-runtime evidence can account for service-lifecycle context, shutdown request references, stop/quiesce observation references, graceful-shutdown checkpoint references, audit replay after shutdown, SQLite reopen after shutdown, restart recovery after shutdown, post-shutdown runtime smoke evidence, and reviewer/operator approval without stopping services, calling service managers, mutating deployment paths, loading secrets, submitting adapters, performing live execution, or claiming production readiness.

### Completed Tasks

- Created `PHASE_76_SUBROADMAP.md`.
- Added typed deployment graceful-shutdown transcript request/report/status models.
- Added ready/blocked validation with blocker codes for missing deployment-host, service-lifecycle, shutdown-request, service-stopped, graceful-shutdown checkpoint, audit replay, SQLite reopen, restart recovery, post-shutdown smoke, operator, and reviewer evidence references.
- Rejected validator-performed service-manager actions, deployment path mutation, secret loading, external submission, live execution, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-deployment-graceful-shutdown-transcript`.
- Added the transcript to the aggregate deployment-runtime gate, deployment evidence bundle, and deployment evidence checklist.
- Added focused local Rust tests for ready, missing-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No service stop/start/restart execution.
- No service-manager actions.
- No deployment path mutation.
- No secret loading.
- No external calls, adapter submission, live execution, or production-readiness approval.

### Validation

Must be refreshed after this patch:

```bash
cargo test -p arb-core deployment_graceful_shutdown_transcript -- --nocapture
cargo run -p arb-agent -- validate-deployment-graceful-shutdown-transcript
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deployment graceful-shutdown transcript validation only. Actual service-manager-controlled graceful shutdown, deployment-host stop/start/restart execution, deployment-host recovery behavior, production audit/SQLite recovery, and production readiness remain unclaimed.

## Phase 77 - Local Market-Data Provider Reconciliation Review Gate

### Goal

Add a typed local market-data provider rate-limit/outage reconciliation review so connector evidence can account for degraded provider preflight, retry-after/backoff handling, outage retry exhaustion, stale-data blocking, latency blocking, and unresolved external provider evidence without live REST/WebSocket calls, provider credentials, WebSocket connections, adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_77_SUBROADMAP.md`.
- Added `MarketDataProviderReconciliationReviewRequest`, `MarketDataProviderReconciliationReviewReport`, and `MarketDataProviderReconciliationReviewStatus`.
- Added `review_market_data_provider_reconciliation` over existing local latency/backpressure review, degraded provider preflight, ready rate-limit reconnect plan, and blocked outage reconnect plan.
- Required local rate-limit, outage, stale-data, latency, degraded-sample, retry-after/backoff, outage-exhaustion, and remaining-external-evidence fields before the review reports ready for local review.
- Rejected live network use, WebSocket opening, credential loading, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-market-data-provider-reconciliation`.
- Added the new CLI to `scripts/validate_connector_scenario_gate.py`, raising the connector scenario aggregate to 17 local components.
- Added focused local Rust tests for ready, missing-outage-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No live REST/WebSocket provider implementation.
- No provider credentials or account queries.
- No provider-backed reconnect loops.
- No external latency measurement.
- No adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core market_data_provider_reconciliation_review -- --nocapture
cargo run -p arb-agent -- validate-market-data-provider-reconciliation
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local market-data provider rate-limit/outage reconciliation review only. Live REST/WebSocket providers, provider-backed reconnect/rate-limit/outage validation, external latency/data-quality evidence, deployment-host resource profiling, sandbox/read-only provider validation, and production readiness remain unclaimed.

## Phase 78 - Local Market-Data Bad-Data Rejection Review Gate

### Goal

Add a typed local market-data bad-data rejection review so connector evidence can account for stale-data rejection, excessive-spread rejection, insufficient-depth rejection, capture-latency rejection, acceptable baseline evidence, and unresolved external provider evidence without live REST/WebSocket calls, provider credentials, WebSocket connections, adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_78_SUBROADMAP.md`.
- Added `MarketDataBadDataRejectionReviewRequest`, `MarketDataBadDataRejectionReviewReport`, and `MarketDataBadDataRejectionReviewStatus`.
- Added `review_market_data_bad_data_rejection` over existing local quality assessment evidence.
- Required acceptable baseline quality, stale-data rejection, excessive-spread rejection, insufficient-depth rejection, capture-latency rejection, bad-data fixture floor, and remaining-external-evidence fields before the review reports ready for local review.
- Rejected live network use, WebSocket opening, credential loading, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-market-data-bad-data-rejection`.
- Added the new CLI to `scripts/validate_connector_scenario_gate.py`, raising the connector scenario aggregate to 18 local components.
- Added focused local Rust tests for ready, missing fixture-floor, and fail-closed side-effect cases.

### Explicit Non-Goals

- No live REST/WebSocket provider implementation.
- No provider credentials or account queries.
- No external latency or data-quality measurement.
- No real historical dataset download.
- No adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core market_data_bad_data_rejection -- --nocapture
cargo run -p arb-agent -- validate-market-data-bad-data-rejection
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local market-data bad-data rejection review only. Live REST/WebSocket providers, provider-backed bad-data rejection validation, external latency/data-quality evidence, real historical dataset validation, sandbox/read-only provider validation, and production readiness remain unclaimed.

## Phase 79 - Local Fee Schedule Reconciliation Review Gate

### Goal

Add a typed local fee schedule reconciliation review so connector evidence can account for current fee-review readiness, unverified-schedule rejection, maker/taker tier rejection, network/gas fee rejection, withdrawal-fee rejection, stale-review rejection, and unresolved external fee evidence without provider API calls, RPC calls, credential loading, account queries, signing, broadcasts, live execution, or production-readiness approval.

### Completed Tasks

- Created `PHASE_79_SUBROADMAP.md`.
- Added `FeeScheduleReconciliationReviewRequest`, `FeeScheduleReconciliationReviewReport`, and `FeeScheduleReconciliationReviewStatus`.
- Added `review_fee_schedule_reconciliation` over existing local current and blocked fee verification reports.
- Required current fee-review readiness, unverified-schedule rejection, maker/taker rejection, network/gas fee rejection, withdrawal-fee rejection, stale-review rejection, and remaining-external-evidence fields before the review reports ready for local review.
- Rejected live provider calls, credential loading, and production-readiness claims.
- Surfaced the gate through `arb-agent validate-fee-schedule-reconciliation`.
- Added the new CLI to `scripts/validate_connector_scenario_gate.py`, raising the connector scenario aggregate to 19 local components.
- Added focused local Rust tests for ready, missing external-evidence, and fail-closed side-effect cases.

### Explicit Non-Goals

- No provider API calls or account queries.
- No exchange credentials or wallet credentials.
- No RPC/gas provider calls.
- No external fee schedule retrieval.
- No adapter submission, signing, broadcasts, live execution, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core fee_schedule_reconciliation -- --nocapture
cargo test -p arb-agent fee_schedule_reconciliation -- --nocapture
cargo run -p arb-agent -- validate-fee-schedule-reconciliation
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local fee schedule reconciliation review only. Real account-tier reconciliation, provider/API fee validation, gas/RPC fee validation, withdrawal-cost verification, external fee schedule review, and production readiness remain unclaimed.

## Phase 80 - Local Runtime Config Reload Validation Gate

### Goal

Add local runtime config reload validation so deployment/runtime evidence can account for safe reload parsing, non-live mode enforcement, local allowlist change detection, deployment-host wrapper reporting, and aggregate deployment-runtime enforcement without reloading service managers, starting services, loading secrets, submitting adapters, performing live execution, or claiming production readiness.

### Completed Tasks

- Created `PHASE_80_SUBROADMAP.md`.
- Added `RuntimeConfigReloadValidationRequest`, `RuntimeConfigReloadValidationReport`, and `RuntimeConfigReloadStatus`.
- Added `validate_runtime_config_reload` over two already parsed `AgentConfig` values.
- Required safe initial and reloaded modes, detected local config changes, CEX allowlist change detection, asset allowlist change detection, and no side-effect flags before the report is ready for local review.
- Surfaced the gate through `arb-agent validate-runtime-config-reload --workspace <fresh-dir>`.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-runtime-config-reload`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`, raising the aggregate to 32 local runtime/deployment components.
- Added focused local Rust tests for ready, unchanged-config blocked, and fail-closed side-effect cases.

### Explicit Non-Goals

- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core runtime_config_reload -- --nocapture
cargo test -p arb-agent runtime_config_reload -- --nocapture
cargo run -p arb-agent -- validate-runtime-config-reload --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-runtime-config-reload --runtime-config-reload-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local runtime config reload validation only. Service-manager-controlled reload, deployment-host config reload, daemon uptime soak, start/stop/restart validation, production filesystem behavior, real observability smoke, and production readiness remain unclaimed.

## Phase 81 - Local SQLite WAL Schema Migration Gate

### Goal

Add local SQLite WAL schema migration validation so deployment/runtime evidence can account for real `SqliteWalStateStore` legacy-schema migration, checkpoint preservation, and future-version fail-closed behavior without touching deployment hosts, service managers, secrets, external systems, live execution, or production readiness.

### Completed Tasks

- Created `PHASE_81_SUBROADMAP.md`.
- Added `SqliteWalSchemaMigrationStatus` and `SqliteWalSchemaMigrationValidationReport`.
- Added `validate_sqlite_wal_schema_migration` over a fresh local SQLite fixture path.
- Exercised actual `SqliteWalStateStore` migration from a legacy v0 checkpoint table to the supported schema version.
- Verified legacy non-secret checkpoint preservation and future-version fail-closed rejection.
- Surfaced the gate through `arb-agent validate-sqlite-wal-schema-migration --workspace <fresh-dir>`.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-sqlite-schema-migration`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`, raising the aggregate to 33 local runtime/deployment components.
- Added focused local Rust tests for ready local migration and stale-path rejection.

### Explicit Non-Goals

- No deployment-host schema migration execution.
- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core sqlite_wal_schema_migration -- --nocapture
cargo test -p arb-agent sqlite_wal_schema_migration -- --nocapture
cargo run -p arb-agent -- validate-sqlite-wal-schema-migration --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-sqlite-schema-migration --sqlite-schema-migration-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local SQLite WAL schema migration fixture validation only. Deployment-host schema migration execution under service lifecycle, deployment permissions, backup/restore load, restart recovery, physical disk constraints, and production readiness remain unclaimed.

## Phase 82 - Local Deployment Config Redaction Gate

### Goal

Add local deployment config loading and audit-redaction validation so deployment/runtime evidence can account for safe config parsing, audit redaction enforcement, unsafe secret-like metadata rejection, redacted audit append/replay, deployment-host wrapper reporting, and aggregate deployment-runtime enforcement without starting services, mutating deployment hosts, loading secrets, calling external systems, performing live execution, or claiming production readiness.

### Completed Tasks

- Created `PHASE_82_SUBROADMAP.md`.
- Added `arb-agent validate-deployment-config-redaction --workspace <fresh-dir>`.
- Added a local non-secret deployment config fixture that loads through the existing config parser with paper mode, kill switch enabled, withdrawals disabled, live execution disabled, disabled secret backends, and `audit.redact_secrets = true`.
- Exercised the actual append-only audit journal validation path by rejecting unsafe secret-like metadata.
- Appended a redacted configuration audit event and verified audit replay after reopen.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-deployment-config-redaction`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`, raising the aggregate to 34 local runtime/deployment components.
- Added focused local Rust tests for the CLI validation path.

### Explicit Non-Goals

- No deployment-host config loading under a service manager.
- No deployment-host log/audit redaction test under a deployed service.
- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-agent deployment_config_redaction -- --nocapture
cargo run -p arb-agent -- validate-deployment-config-redaction --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-deployment-config-redaction --deployment-config-redaction-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deployment config loading and audit-redaction fixture validation only. Deployment-host config loading under service orchestration, deployed log/audit redaction, startup/shutdown/restart, service hardening, production filesystem behavior, and production readiness remain unclaimed.

## Phase 83 - Local Deployment Log Redaction Gate

### Goal

Add local deployment log/audit redaction validation so deployment/runtime evidence can account for sanitized runtime log output, unsafe secret-like audit metadata rejection, redacted audit append/replay, deployment-host wrapper reporting, and aggregate deployment-runtime enforcement without starting services, mutating deployment hosts, reading secrets, touching deployed logs, calling external systems, performing live execution, or claiming production readiness.

### Completed Tasks

- Created `PHASE_83_SUBROADMAP.md`.
- Added `arb-agent validate-deployment-log-redaction --workspace <fresh-dir>`.
- Added a local sanitized deployment log fixture that writes only redacted credential and wallet references.
- Exercised the actual append-only audit journal validation path by rejecting unsafe secret-like metadata.
- Appended a redacted runtime audit event and verified audit replay after reopen.
- Added deployment-host wrapper support through `scripts/validate_deployment_host_runtime.py --run-deployment-log-redaction`.
- Added aggregate deployment-runtime enforcement through `scripts/validate_deployment_runtime_gate.py`, now raised to 36 local runtime/deployment components with deployment-host observability metrics runtime wrapper enforcement.
- Added focused local Rust tests for the CLI validation path.

### Explicit Non-Goals

- No deployment-host log or audit redaction under a service manager.
- No deployed service log scraping or mutation.
- No service-manager reload, daemon reload, service start, stop, or restart.
- No deployment-host mutation.
- No secret loading.
- No external calls, adapter submission, live execution, signing, broadcasts, or production-readiness approval.

### Validation

```bash
cargo test -p arb-agent deployment_log_redaction -- --nocapture
cargo run -p arb-agent -- validate-deployment-log-redaction --workspace <fresh-dir>
python3 scripts/validate_deployment_host_runtime.py --run-deployment-log-redaction --deployment-log-redaction-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local deployment log/audit redaction fixture validation only. Deployment-host log/audit redaction under service orchestration, deployed log collection, startup/shutdown/restart, service hardening, production filesystem behavior, and production readiness remain unclaimed.

## Phase 84 - Local Communications Outbox Aggregate Gate

### Goal

Wire the local communications outbox validator into the operator-surface aggregate gate so future-delivery evidence covers local outbox persistence, duplicate-dispatch rejection, rate-limit blocking, outage blocking, audit replay, SQLite checkpoint recovery, and unsafe side-effect denial without real outbound communication delivery or production-readiness claims.

### Completed Tasks

- Created `PHASE_84_SUBROADMAP.md`.
- Added `communications_outbox_cli` to `scripts/validate_operator_surface_gate.py`.
- Required the aggregate gate to validate outbox persistence, duplicate rejection, rate-limit blocking, outage blocking, audit replay, SQLite checkpoint recovery, and false unsafe side-effect flags.
- Preserved existing communications runtime, dashboard, observability, deployment-host wrapper, and runtime-smoke aggregate coverage.

### Explicit Non-Goals

- No real platform accounts, tokens, webhooks, messaging APIs, provider calls, or outbound delivery.
- No remote command enablement.
- No deployment-host service-manager action.
- No live execution, signing, broadcast, exchange call, RPC call, bridge, withdrawal, wallet custody, or production-readiness approval.

### Validation

```bash
cargo test -p arb-agent communications_outbox -- --nocapture
cargo run -p arb-agent -- validate-communications-outbox --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for local communications outbox aggregate validation only. Real outbound communication delivery, platform authentication, provider-side rate-limit/outage validation, remote command orchestration, external security review, and production readiness remain unclaimed.

## Phase 85 - Local Dashboard Loopback Runtime Gate

### Goal

Add bounded loopback-only dashboard runtime validation so dashboard hosting evidence covers a multi-request local listener lifecycle, audit replay, SQLite checkpoint recovery, clean shutdown, public exposure denial, live-control denial, and operator-surface aggregate enforcement without public hosting, deployment-host orchestration, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Created `PHASE_85_SUBROADMAP.md`.
- Added typed bounded loopback dashboard runtime probe records and reports.
- Added core audit and SQLite checkpoint helpers for the loopback runtime probe.
- Added `arb-agent validate-dashboard-loopback-runtime --workspace <fresh-dir>`.
- Added focused core and agent tests.
- Added `dashboard_loopback_runtime_cli` to `scripts/validate_operator_surface_gate.py`, raising the aggregate to 9 local operator-surface components.

### Explicit Non-Goals

- No public dashboard exposure.
- No persistent daemon-hosted dashboard service.
- No browser authentication/session implementation claim.
- No CSRF token issuance service claim.
- No deployment-host service-manager action.
- No live controls, external submission, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core dashboard_loopback_runtime -- --nocapture
cargo test -p arb-agent dashboard_loopback_runtime -- --nocapture
cargo run -p arb-agent -- validate-dashboard-loopback-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for bounded local loopback dashboard runtime validation only. Real daemon-hosted dashboard service execution, browser authentication/session validation, CSRF token serving, deployment-host orchestration, public-exposure validation, penetration testing, and production readiness remain unclaimed.

## Phase 86 - Local Observability Metrics Runtime Gate

### Goal

Add bounded loopback-only observability metrics runtime validation so observability evidence covers a multi-scrape local listener lifecycle, audit replay, SQLite checkpoint recovery, response consistency checks, clean shutdown, public exposure denial, telemetry-export denial, outbound-alert denial, and operator-surface aggregate enforcement without daemon hosting, exporter sessions, alert delivery, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Created `PHASE_86_SUBROADMAP.md`.
- Added typed bounded observability metrics runtime probe records and reports.
- Added core audit and SQLite checkpoint helpers for the metrics runtime probe.
- Added `arb-agent validate-observability-metrics-runtime --workspace <fresh-dir>`.
- Added focused core and agent tests.
- Added `observability_metrics_runtime_cli` to `scripts/validate_operator_surface_gate.py`, raising the aggregate to 10 local operator-surface components.

### Explicit Non-Goals

- No public metrics endpoint exposure.
- No persistent daemon-hosted observability endpoint.
- No Prometheus/OpenTelemetry exporter session.
- No log shipping or outbound alert delivery.
- No deployment-host service-manager action.
- No live controls, external submission, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core observability_metrics_runtime -- --nocapture
cargo test -p arb-agent observability_metrics_runtime -- --nocapture
cargo run -p arb-agent -- validate-observability-metrics-runtime --workspace <fresh-dir>
python3 scripts/validate_operator_surface_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for bounded local loopback observability metrics runtime validation only. Real daemon-hosted metrics endpoint operation, exporter sessions, log shipping, outbound alert delivery, deployment-host orchestration, incident drills, external AppSec review, and production readiness remain unclaimed.

## Phase 87 - Deployment Observability Metrics Runtime Wrapper Gate

### Goal

Wire the bounded local observability metrics runtime validator into the deployment-host runtime wrapper and aggregate deployment-runtime gate so deployment-facing local evidence accounts for the same multi-scrape metrics runtime controls as the operator-surface gate, without daemon hosting, telemetry export, alert delivery, service-manager actions, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Created `PHASE_87_SUBROADMAP.md`.
- Added `--run-observability-metrics-runtime` and `--observability-metrics-workspace` to `scripts/validate_deployment_host_runtime.py`.
- Added deployment-host wrapper execution plus JSON/text reporting for `arb-agent validate-observability-metrics-runtime`.
- Added explicit deployment-runtime aggregate enforcement for the observability metrics runtime wrapper report.
- Raised `scripts/validate_deployment_runtime_gate.py` to 36 total local runtime/deployment components and 23 nested runtime components.

### Explicit Non-Goals

- No daemon-hosted persistent metrics endpoint.
- No public network exposure.
- No telemetry export, log shipping, or outbound alert delivery.
- No deployment-host mutation or service-manager action.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_host_runtime.py --run-observability-metrics-runtime --observability-metrics-workspace <fresh-dir> --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for deployment-host wrapper and aggregate-gate coverage of the bounded local observability metrics runtime only. Real daemon-hosted metrics endpoint operation, exporter sessions, log shipping, outbound alert delivery, deployment-host service orchestration, incident drills, external AppSec review, and production readiness remain unclaimed.

## Phase 88 - Deployment Evidence Bundle Metrics Runtime Component Gate

### Goal

Add the deployment-host observability metrics runtime wrapper to the deployment evidence bundle as a direct bounded local component so bundle consumers can find that evidence without relying only on the aggregate deployment-runtime gate.

### Completed Tasks

- Created `PHASE_88_SUBROADMAP.md`.
- Added `deployment-host-observability-metrics-runtime` to `scripts/validate_deployment_evidence_bundle.py`.
- Scoped and refreshed the local bundle workspace under `target/deployment-evidence-bundle`.
- Verified the bundle reports 20 local components with the metrics runtime component present, all components passing, and no unsafe flags.

### Explicit Non-Goals

- No daemon-hosted persistent metrics endpoint.
- No public network exposure.
- No telemetry export, log shipping, or outbound alert delivery.
- No deployment-host mutation or service-manager action.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for direct deployment evidence bundle indexing of the bounded local observability metrics runtime wrapper only. Real daemon-hosted metrics endpoint operation, exporter sessions, log shipping, outbound alert delivery, deployment-host service orchestration, incident drills, external AppSec review, and production readiness remain unclaimed.

## Phase 89 - Deployment Evidence Checklist Required Bundle Component Gate

### Goal

Make the deployment evidence checklist fail closed when required bounded local bundle components are missing, starting with the deployment-host observability metrics runtime component added in Phase 88.

### Completed Tasks

- Created `PHASE_89_SUBROADMAP.md`.
- Added `REQUIRED_BUNDLE_COMPONENTS` to `scripts/validate_deployment_evidence_checklist.py`.
- Required `deployment-host-observability-metrics-runtime` to appear in the bundle index.
- Surfaced required and missing required bundle component names in checklist output.
- Verified the checklist reports one required component, zero missing required components, and the metrics runtime component present.

### Explicit Non-Goals

- No daemon-hosted persistent metrics endpoint.
- No public network exposure.
- No telemetry export, log shipping, or outbound alert delivery.
- No deployment-host mutation or service-manager action.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for fail-closed checklist enforcement of the required local deployment-host observability metrics runtime bundle component only. Real daemon-hosted metrics endpoint operation, exporter sessions, log shipping, outbound alert delivery, deployment-host service orchestration, incident drills, external AppSec review, and production readiness remain unclaimed.

## Phase 90 - Deployment Host Static Hardening Config Smoke Runtime Gate

### Goal

Wire static deployment hardening/config smoke validation into the deployment-host runtime wrapper and aggregate deployment-runtime gate so runtime/deployment evidence proves the committed example config still loads through the real `arb-agent --config` path with live execution disabled.

### Completed Tasks

- Created `PHASE_90_SUBROADMAP.md`.
- Added `--run-deployment-static-hardening` to `scripts/validate_deployment_host_runtime.py`.
- Composed `scripts/validate_deployment_static_hardening.py --run-config-smoke --json` into the deployment-host runtime wrapper.
- Surfaced config smoke, safe config, live-execution denial, service-action denial, network-listener denial, and secret-loading denial fields in wrapper JSON.
- Required the new `deployment_static_hardening` component in `scripts/validate_deployment_runtime_gate.py`, raising the aggregate to 37 total local runtime/deployment components and 24 nested runtime components.

### Explicit Non-Goals

- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
- No production config mutation or deployment-host path mutation.
- No public network exposure.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_host_runtime.py --run-deployment-static-hardening --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for deployment-host wrapper and aggregate-gate coverage of local static hardening/config smoke validation only. Real service-manager config loading, deployed config reload, production filesystem validation, deployment-host service orchestration, external review, and production readiness remain unclaimed.

## Phase 91 - Deployment Evidence Bundle Static Hardening Component Gate

### Goal

Expose the Phase 90 deployment-host static hardening/config smoke wrapper evidence as a direct deployment evidence bundle component and require it in the checklist.

### Completed Tasks

- Created `PHASE_91_SUBROADMAP.md`.
- Added `deployment-host-static-hardening-config-smoke` to `scripts/validate_deployment_evidence_bundle.py`.
- Added `deployment-host-static-hardening-config-smoke` to `REQUIRED_BUNDLE_COMPONENTS` in `scripts/validate_deployment_evidence_checklist.py`.
- Verified the deployment evidence bundle reports 21 bounded local components with the static hardening/config smoke and observability metrics runtime components present.
- Verified the deployment evidence checklist reports two required bundle components, zero missing required components, and the static hardening/config smoke component present.

### Explicit Non-Goals

- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
- No production config mutation or deployment-host path mutation.
- No public network exposure.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for direct deployment evidence bundle/checklist enforcement of local static hardening/config smoke wrapper evidence only. Real service-manager config loading, deployed config reload, production filesystem validation, deployment-host service orchestration, external review, and production readiness remain unclaimed.

## Phase 92 - Deployment Evidence Bundle Config And Log Redaction Component Gate

### Goal

Expose deployment-host config redaction and log redaction wrapper evidence as direct deployment evidence bundle components and require them in the checklist.

### Completed Tasks

- Created `PHASE_92_SUBROADMAP.md`.
- Added `deployment-host-config-redaction` and `deployment-host-log-redaction` to `scripts/validate_deployment_evidence_bundle.py`.
- Scoped both components to the per-process deployment evidence bundle workspace.
- Added both components to `REQUIRED_BUNDLE_COMPONENTS` in `scripts/validate_deployment_evidence_checklist.py`.
- Verified the deployment evidence bundle reports 23 bounded local components with both redaction components present.
- Verified the deployment evidence checklist reports four required bundle components and zero missing required components.

### Explicit Non-Goals

- No deployed config mutation or deployed log scraping.
- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
- No production path mutation.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for direct deployment evidence bundle/checklist enforcement of local deployment-host config/log redaction wrapper evidence only. Real deployed config loading, deployed log/audit redaction, production filesystem validation, deployment-host service orchestration, external review, and production readiness remain unclaimed.

## Phase 93 - Deployment Evidence Bundle Filesystem Transcript Component Gate

### Goal

Expose deployment disk-full, retention/rotation, and permission-denial transcript validators as direct deployment evidence bundle components and require them in the checklist.

### Completed Tasks

- Created `PHASE_93_SUBROADMAP.md`.
- Added `deployment-disk-full-transcript`, `deployment-retention-transcript`, and `deployment-permission-transcript` to `scripts/validate_deployment_evidence_bundle.py`.
- Added those components to `REQUIRED_BUNDLE_COMPONENTS` in `scripts/validate_deployment_evidence_checklist.py`.
- Verified the deployment evidence bundle reports 26 bounded local components with all three filesystem transcript components present.
- Verified the deployment evidence checklist reports seven required bundle components and zero missing required components.

### Explicit Non-Goals

- No physical disk filling.
- No retention rotation or deletion against production logs.
- No permission changes.
- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for direct deployment evidence bundle/checklist enforcement of local filesystem-related deployment transcript evidence only. Physical disk-full execution, deployment-host retention/rotation execution, permission changes under service orchestration, external review, and production readiness remain unclaimed.

## Phase 94 - Production Runtime Preflight Evidence Category Expansion

### Goal

Expand the typed local production-runtime preflight so GAP-0076 deployment-host runtime evidence categories are represented directly in Rust, CLI output, and aggregate deployment-runtime validation.

### Completed Tasks

- Created `PHASE_94_SUBROADMAP.md`.
- Added production-runtime preflight request/report fields for deployment-host backup/restore, graceful shutdown, audit/SQLite recovery, SQLite schema migration, daemon failure-capture, and concurrent lifecycle execution evidence.
- Added unresolved blocker generation for each new missing evidence category.
- Surfaced the new fields through `arb-agent validate-runtime-smoke` output.
- Parsed and enforced the new fields in `scripts/validate_deployment_host_runtime.py` and `scripts/validate_deployment_runtime_gate.py`.

### Explicit Non-Goals

- No service-manager actions.
- No deployment-host mutation.
- No backup/restore execution against production paths.
- No SQLite migration execution against deployment hosts.
- No daemon failure injection.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_host_runtime.py --run-runtime-smoke --json
python3 scripts/validate_deployment_runtime_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met for typed local production-runtime preflight accounting of additional deployment-host runtime evidence categories only. Actual deployment-host execution evidence, service-manager orchestration, external validation, and production readiness remain unclaimed.

## Phase 95 - CEX Live Adapter Boundary Review Gate

### Goal

Replace the loose CEX live-adapter "not implemented" dead end with a typed local review gate that validates local prerequisites and keeps live exchange implementation blocked until sandbox/live evidence exists.

### Completed Tasks

- Created `PHASE_95_SUBROADMAP.md`.
- Added `CexLiveAdapterBoundaryReviewRequest`, `CexLiveAdapterBoundaryReviewReport`, and `CexLiveAdapterBoundaryReviewStatus`.
- Added `review_cex_live_adapter_boundary()` with side-effect rejection and explicit blocker codes.
- Added focused Rust tests for blocked prerequisite review and side-effect rejection.
- Added `arb-agent validate-cex-live-adapter-boundary`.
- Wired `cex_live_adapter_boundary` into `scripts/validate_connector_scenario_gate.py`, raising the connector aggregate gate to 20 local components.

### Explicit Non-Goals

- No REST calls.
- No WebSocket connections.
- No credential loading.
- No account reads.
- No live exchange orders or cancels.
- No external submission, live execution, signing, broadcast, wallet custody, or production-readiness approval.

### Validation

```bash
cargo test -p arb-core cex_live_adapter_boundary -- --nocapture
cargo run -p arb-agent -- validate-cex-live-adapter-boundary
python3 scripts/validate_connector_scenario_gate.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Phase 96 - DEX/Web3 Live Adapter Boundary Review Gate

### Implementation

- Created `PHASE_96_SUBROADMAP.md`.
- Added typed local `DexLiveAdapterBoundaryReviewRequest` / `DexLiveAdapterBoundaryReviewReport` with fail-closed side-effect validation and blocker codes for missing testnet/provider/signer/broadcast evidence.
- Added `arb-agent validate-dex-live-adapter-boundary`.
- Wired `dex_live_adapter_boundary` into `scripts/validate_connector_scenario_gate.py`, raising the connector aggregate gate to 21 local components.
- Preserved no HTTP/RPC calls, credential loading, signing, broadcasts, bridges, external submission, live execution, or production-readiness claim.

### Validation

```text
cargo test -p arb-core dex_live_adapter_boundary -- --nocapture
cargo run -p arb-agent -- validate-dex-live-adapter-boundary
python scripts/validate_connector_scenario_gate.py --json
```

## Phase 97 - Deployment Evidence Checklist Required Transcript Expansion

### Implementation

- Created `PHASE_97_SUBROADMAP.md`.
- Expanded `scripts/validate_deployment_evidence_checklist.py` required bundle components so the checklist now fails closed if existing local transcript gates for audit/SQLite recovery, backup/restore, graceful shutdown, SQLite schema migration, rollback execution, incident-response execution, failure capture, or response-drill rehearsal are missing.
- Preserved reference-only, non-mutating checklist behavior.

### Validation

```text
python scripts/validate_deployment_evidence_bundle.py --json
python scripts/validate_deployment_evidence_checklist.py --json
```

## Phase 98 - Deployment Checklist Lifecycle and Drill Plan Requirement Expansion

### Implementation

- Created `PHASE_98_SUBROADMAP.md`.
- Expanded `scripts/validate_deployment_evidence_checklist.py` required bundle components so the checklist now fails closed if existing local systemd lifecycle plan, deployment-host runtime plan, deployment-host retention preflight, rollback-drill plan, incident-response drill plan, or service-manager lifecycle rehearsal components are missing.
- Preserved reference-only, non-mutating checklist behavior.

### Validation

```text
python scripts/validate_deployment_evidence_bundle.py --json
python scripts/validate_deployment_evidence_checklist.py --json
```

### Exit Criteria

Met for deployment checklist lifecycle and drill-plan component enforcement only. Real service-manager lifecycle execution, rollback execution, incident-response execution, deployment mutation, and production readiness remain unclaimed.

## Phase 99 - Production Container Aggregate Gate Enforcement

### Implementation

- Created `PHASE_99_SUBROADMAP.md`.
- Expanded `scripts/validate_packaging_deployment_gate.py` so the production-intent container validator is a required aggregate component, not only a standalone CI path.
- Propagated `service_installed: false` through `scripts/validate_hardening_core_gate.py` and `scripts/validate_agentic_handoff_candidate_gate.py`.

### Validation

```text
python scripts/validate_production_container.py --json
python scripts/validate_packaging_deployment_gate.py --json
python scripts/validate_hardening_core_gate.py --json
python scripts/validate_agentic_handoff_candidate_gate.py --json
```

### Exit Criteria

Met for local production-intent container aggregate enforcement only. Production image publishing, service installation, deployment-host lifecycle execution, and production readiness remain unclaimed.

## Phase 100 - Handoff Candidate Full Local Surface Gate

### Implementation

- Created `PHASE_100_SUBROADMAP.md`.
- Expanded `scripts/validate_agentic_handoff_candidate_gate.py` so the handoff candidate now requires the execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening-core, deployment-evidence checklist, and local handoff audit gates.
- Added nested aggregate assertions for unsafe side-effect flags and local-only non-claims.
- Added a combined remaining external-evidence summary from nested local software-surface gates.

### Validation

```text
python scripts/validate_agentic_handoff_candidate_gate.py --json
```

### Exit Criteria

Met for full local software-surface handoff candidate aggregation only. External agent execution, live/provider-backed validation, service deployment, rollback/incident execution, and production readiness remain unclaimed.
