# ArbyClaw

Rust-first, governance-driven crypto arbitrage agent scaffold.

## Current Status

- Phase 0 governance initialized.
- Phase 1 Rust workspace scaffold created.
- Phase 2 typed config, reference-only secret boundaries, and initial mode gates implemented.
- Phase 3 deny-by-default policy engine implemented.
- Phase 4 append-only audit journal primitives and state-store trait boundary implemented.
- Phase 5 normalized market-data, freshness, and fee-model boundaries implemented.
- Phase 6 deterministic in-memory paper market-data, static paper-fee, and policy-gated paper execution boundaries implemented.
- Phase 7 CEX connector framework types, capability registry, order request model, policy gate, and connector traits implemented.
- Phase 8 DEX/Web3 framework chain, token, router, swap quote, local transaction simulation, policy gate, and connector traits implemented.
- Phase 9 deterministic opportunity-engine discovery/ranking models implemented; Phase 27 adds local depth, paper inventory, transfer-risk, triangular path, replay/false-positive modeling, local historical fixture replay, candidate audit/state trace persistence, and replay-candidate planner handoff validation without live calls.
- Phase 10 draft-only execution-planner models, per-leg intent generation, policy preflight outcomes, sequencing, and failure-mode boundaries implemented.
- Phase 11 deterministic execution-adapter framework models, policy revalidation, attempt/fill/reconciliation records, and external-submission blocks implemented.
- Phase 12 communications/CLI command routing, notification models, redaction checks, and local dispatch-record boundaries implemented.
- Phase 13 embedded dashboard snapshot, panel, local rendering, secret-redaction, and fail-closed server-binding boundaries implemented.
- Phase 14 observability/runbook local health, structured-log, metric, runbook, redaction, and fail-closed telemetry endpoint boundaries implemented.
- Phase 15 testing/fuzzing/backtesting validation-plan, fixture, fuzz corpus, and backtest scenario boundaries implemented.
- Phase 16 packaging/deployment plan, target, release-gate, rollback-step, example deployment documentation, local example-container validation, and static example systemd-unit validation boundaries implemented.
- Phase 17 external hardening evidence, release-blocker, production-readiness checklist, and incident-drill template boundaries implemented.
- Phase 18 agentic handoff package records, future-agent prompts, governance checklist, and external validation checklist boundaries implemented.
- Phase 19 local runtime lifecycle wiring implemented for fail-closed audit/state/adapter sequencing, concurrent local lifecycle access checks, simulated state-permission fail-closed checks, local graceful-shutdown checkpointing, local runtime audit/SQLite backup-restore validation, local restart recovery summaries with operator-review dispositions, CLI status labels for those local dispositions, incomplete-recovery fail-closed checks, and local deployment-like runtime smoke validation plus CLI runner without service-manager actions.
- Phase 20 local SQLite WAL durability validation implemented for integrity, WAL checkpoint, reopen, backup/restore, and multi-handle state-store checks.
- Phase 21 local paper balance ledgering implemented for simulated balances, reservations, fill settlement, insufficient-balance denial, and SQLite checkpoints.
- Phase 22 local process-level crash/restart durability validation implemented for SQLite WAL checkpoint recovery.
- Phase 23 local realistic paper fills implemented for supplied order-book depth, partial fills, unfilled outcomes, latency, queue position, slippage, and ledger-safe unfilled notional release.
- Phase 25 local paper execution report and ledger mutation audit journal integration implemented.
- Phase 26 local audit journal crash-like truncation, tamper, concurrent append, sync, invalid-filesystem, simulated disk-full validation, retention planning, and stale-lock restart recheck planning implemented.
- Live trading is not implemented.
- Wallet signing is not implemented.
- External adapter submission, exchange-specific live CEX adapters, live DEX/Web3 RPC/signing/broadcast adapters, real outbound communications adapters, real dashboard hosting, real observability/exporter/alert runtime, real fuzzing engines, real external backtest execution beyond local paper fixtures, production container validation, systemd installation, ARM validation, external agent execution, and production deployment are not implemented.
- Live REST/WebSocket/RPC market-data providers are not implemented.
- Secrets must not be committed or pasted into chat, Markdown, TOML, logs, or prompts.

## Config Usage

The committed `config.example.toml` is observe-only and contains no raw credentials.

```bash
arb-agent --config config.example.toml
```

Secret configuration must use references only, such as environment variable names or future encrypted-keystore aliases. Raw API keys, wallet keys, seed phrases, mnemonics, and tokens are forbidden in repository files.

## Local Validation

Current local/CI validation command set:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_structure.py
```

## Safety Position

This repository is not ready for live funds. Future live execution must remain gated behind typed config, encrypted secret handling, deny-by-default policy checks, durable redacted audit journaling, validated market-data freshness, simulation where available, and constrained signer boundaries. Phase 6 paper primitives, Phase 7 CEX framework primitives, Phase 8 DEX/Web3 framework primitives, Phase 9 opportunity-engine primitives, Phase 10 execution-planner draft primitives, Phase 11 execution-adapter framework primitives, Phase 12 communications/CLI primitives, Phase 13 embedded-dashboard primitives, Phase 14 observability/runbook primitives, Phase 15 testing/backtesting validation primitives, Phase 16 packaging/deployment planning primitives, Phase 17 external-hardening evidence/checklist primitives, Phase 18 agentic handoff primitives, Phase 19 local runtime lifecycle primitives with concurrent local lifecycle access checks, simulated state-permission fail-closed checks, graceful-shutdown checkpointing, local runtime audit/SQLite backup-restore validation, local restart recovery summaries with CLI-visible operator-review dispositions, incomplete-recovery fail-closed checks, and local deployment-like runtime smoke validation plus CLI runner, Phase 20 local SQLite WAL durability validation primitives, Phase 21 local paper balance ledger primitives, Phase 22 local process-level crash/restart validation primitives, Phase 23 realistic paper fill primitives, Phase 24 paper replay/calibration/backtest/runtime validation primitives, Phase 25 local paper audit journal integration primitives, Phase 26 local audit journal crash/concurrency/filesystem/simulated-disk-full validation plus retention/rotation planning and stale-lock restart recheck planning primitives, and Phase 27 local opportunity depth/inventory/transfer-risk/triangular path/replay/regression-corpus/historical-fixture modeling plus route-classification, truncation, stale-data fail-closed, and CLI/CI validation primitives exist, and local/CI Rust validation is current for this workspace state. External adapter submission, exchange-specific live adapters, live DEX/RPC adapters, signer/broadcast controls, live provider validation, external sandbox/live fill calibration evidence, broader external/deployment opportunity scenario-corpus validation, real outbound communications adapters, authenticated remote command channels, real dashboard hosting/authentication, real metrics/exporter/alert runtime, real property/fuzz runner execution, production packaging/deployment validation, real external hardening execution, external agent execution validation, physical disk-full testing, retention/rotation execution validation, service-manager restart execution validation, deployment-host backup/restore validation, deployment-host SQLite WAL durability validation, production runtime validation, and live execution integration remain incomplete.


## Phase 12 Communications and CLI Boundary

The communications/CLI boundary defines deterministic local command and notification models only. It parses and routes local operator commands into typed records, rejects live execution, withdrawal, bridge, signing, and broadcast requests, and produces local notification dispatch records without outbound network delivery. It does not use messaging platform tokens, call Telegram/Discord/Slack/email/PagerDuty/webhook APIs, authenticate remote operators, execute trades, bypass policy, sign transactions, broadcast transactions, or load secrets. Durable audit/state integration, real channel adapters, authn/authz, rate limiting, and operator UX validation remain required before production communications use.


## Phase 13 Embedded Dashboard Boundary

The embedded-dashboard boundary defines deterministic local dashboard snapshot and rendering models only. It can render sanitized in-process records for panels such as safety, market data, opportunities, planner status, execution-adapter status, communications, audit/state, and gaps. It rejects public exposure, rejects HTTP server startup, redacts secret-like display text, and keeps live controls disabled. It does not start a web server, expose localhost or public routes, authenticate users, execute commands, submit trades, sign transactions, broadcast transactions, or load secrets. Real dashboard hosting, authentication/session handling, CSRF protection, durable dashboard state, UX validation, and penetration testing remain required before production dashboard use.

## Phase 14 Observability and Runbook Boundary

The observability/runbook boundary defines deterministic local health, structured-log, metric, and runbook records only. It can collect sanitized in-process records for operator review, but it rejects metrics endpoint startup, public telemetry exposure, outbound alert delivery, and secret-like observability text. It does not start Prometheus/OpenTelemetry exporters, ship logs, call SIEM or alerting providers, execute incident automation, sign transactions, broadcast transactions, load secrets, or make exchange/RPC calls. Real observability runtime integration, authenticated metrics endpoint design, alert routing, log retention/rotation, incident drills, and audit/state integration remain required before production observability use.

## Phase 15 Testing, Fuzzing, and Backtesting Boundary

The testing/fuzzing/backtesting boundary defines deterministic validation plans, test case metadata, fixture metadata, fuzz corpus definitions, backtest dataset definitions, and local validation run records only. It validates that planned tests remain local and fail closed when external fuzzer invocation, live network testing, live execution, signing, broadcasts, or credential-bearing fixtures are requested. Phase 24 adds local paper backtest execution over caller-supplied fixtures, and Phase 27 wires the built-in local opportunity replay, local historical fixture corpora, candidate audit/state trace persistence, and replay-candidate planner handoff into CLI/CI validation. It does not launch fuzzers, download market data, call exchanges/RPC providers, submit orders, sign transactions, broadcast transactions, or prove strategy profitability. Property-test runners, fuzzing engines, broader external/deployment fixture corpora beyond the built-in local gates, load tests, penetration tests, and production validation remain required.

## Phase 16 Packaging and Deployment Boundary

The packaging/deployment boundary defines deterministic package target plans, service hardening metadata, release gates, rollback steps, and local package/deployment records only. It validates that plans do not claim builds, installs, public exposure, live trading, embedded secrets, or production deployment. Current CI covers locked release build, dependency audit, SBOM generation, static plus `systemd-analyze` syntax validation for the example systemd unit, and an example-only container image scan; `scripts/validate_container_example.py` repeats the local example image build, Trivy gates, and container CLI smoke check when Docker is available, while `scripts/validate_systemd_example.py` statically checks the committed example unit and can run syntax verification against a temporary fake root without installing, enabling, reloading, or starting services. No production container image, deployed service unit, ARM binary, cloud deployment, rollback drill, or production release has been validated. Production container validation, deployment-host systemd validation, ARM target validation, rollback drills, load tests, penetration tests, and deployment validation remain required.



## Phase 17 External Production Hardening Evidence Boundary

The external hardening boundary defines deterministic evidence records, hardening plans, release blockers, and local review records only. It validates that ChatGPT-mode records do not claim external action execution, production readiness, live-funds approval, public exposure approval, or secret-bearing evidence. The hardening runbook, readiness checklist, and incident-response drill template are operator checklists only. CI evidence exists for release build, dependency audit, SBOM generation, local-SARIF SAST, example image scan, secret-pattern scan, and hardening evidence indexing; the local example-container validation script can refresh Docker/Trivy smoke evidence without retaining artifacts. SBOM review, production image review, staging deployment, load test, penetration test, rollback drill, incident drill, live exchange validation, DEX/RPC validation, and production readiness review remain unexecuted.

## Phase 18 Agentic Handoff Boundary

The handoff subsystem provides deterministic package records, continuation prompts, governance reconciliation checklists, and external validation checklists for future agents and maintainers. It does not call external coding-agent services, deploy infrastructure, approve production readiness, approve public exposure, approve live funds, or store credentials. See `handoff/AGENTIC_HANDOFF_PACKAGE.md`, `handoff/FUTURE_AGENT_PROMPTS.md`, and `handoff/EXTERNAL_VALIDATION_CHECKLIST.md`.

## Phase 19 Runtime Lifecycle Boundary

The runtime lifecycle boundary wires local audit/state preconditions around deterministic adapter evaluation. It appends audit events, checkpoints the plan before adapter evaluation, evaluates the deterministic adapter boundary, checkpoints the adapter run, and records that no external submission or live execution occurred. It has a local concurrent lifecycle test over shared audit journal and SQLite WAL state paths, a simulated state-permission failure test that stops before adapter evaluation, a local graceful-shutdown checkpoint boundary that writes shutdown audit records and a SQLite-reopenable state checkpoint without stopping services, a local backup-restore validation that copies non-secret audit/SQLite artifacts and verifies restored runtime checkpoints, a local restart recovery summary that replays audit plus SQLite checkpoints without service resume, and a local deployment-like smoke harness plus `validate-runtime-smoke` CLI runner that combine lifecycle, graceful-shutdown, backup/restore, restart recovery, and audit durability probes without service-manager actions. Restart recovery classifies locally coherent state as ready-for-local-review or needs-operator-review, surfaces those labels in CLI status as local operator-review states, and fails closed when audit replay exists but required SQLite lifecycle checkpoints are missing. It rejects live-scope lifecycle requests before audit/state mutation. This is local lifecycle wiring only, not production deployment or live execution readiness.

## Phase 20 SQLite WAL Durability Boundary

The SQLite WAL durability boundary validates local non-secret checkpoint persistence. It verifies WAL mode, synchronous FULL, SQLite integrity check, WAL checkpoint truncate, primary database reopen, checkpointed backup/restore, and multi-handle visibility. It does not store secrets, call networks, trade, sign, broadcast, deploy services, or approve production readiness. External production-host crash/restart/filesystem validation remains required.

## Phase 21 Paper Balance Ledger Boundary

The paper balance ledger boundary tracks local simulated balances only. It reserves quote notional for paper intents, settles filled paper reports with net paper P&L, fails closed on insufficient balances or missing reservations, and can persist the ledger through SQLite WAL checkpoints. It does not read real balances, mutate real accounts, call venues, sign, broadcast, withdraw, bridge, or approve production readiness.

## Phase 22 Crash/Restart Durability Boundary

The crash/restart durability boundary is a local Cargo integration harness. It starts child test processes that write SQLite WAL checkpoints and exit abruptly after start, planner, or adapter stages; the parent process reopens the database, runs integrity checks, and verifies expected checkpoint recovery. It does not simulate power loss, disk-full behavior, deployment services, live trading, real venues, signing, broadcasts, withdrawals, bridges, custody, or production readiness.

## Phase 23 Realistic Paper Fill Boundary

The realistic paper fill boundary consumes caller-supplied normalized order-book snapshots and models local paper fills with depth walking, buy/sell side selection, partial and unfilled outcomes, latency, queue-position haircuts, average price, slippage, and ledger settlement that releases unfilled reserved notional. It does not call live venues, read or mutate real balances, submit orders, sign transactions, broadcast transactions, withdraw, bridge, or prove production profitability.

## Phase 24 Paper Replay, Calibration, Backtest, and Runtime Validation Boundary

The paper replay/calibration/backtest boundary models local venue matching profiles, adverse-selection penalties, reference-only calibration records, ledger replay validation, local historical-fixture paper backtests, and runtime validation records. It does not call sandbox or live venues, download market data, embed evidence contents, use secrets, submit external orders, mutate real balances, sign, broadcast, withdraw, bridge, or approve production readiness. Production-host validation and real sandbox/live calibration evidence remain required.

## Phase 25 Paper Audit Journal Integration Boundary

The paper audit integration boundary appends sanitized paper execution report records and paper reserve/settlement ledger mutation records to the local append-only audit journal, then reopens the journal to verify hash-chain replay in local tests. It does not call exchanges or RPC providers, submit external orders, mutate real balances, sign, broadcast, withdraw, bridge, store secrets, or approve production readiness. Deployment-host audit validation remains required.

## Phase 26 Audit Crash, Concurrency, and Filesystem Validation Boundary

The audit durability boundary adds local append locking, file flush plus `sync_all`, append/reopen replay validation, crash-like truncated JSONL replay rejection, tamper rejection, concurrent append replay validation, invalid filesystem fail-closed checks, disk-full error classification, a simulated disk-full append probe that confirms journal state is not advanced after failed persistence, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning. The retention plan only marks rotate, retained, and expired file labels; it does not delete, rename, compress, or mutate logs. The stale-lock plan only marks whether a caller-supplied lock observation should be rechecked; it does not delete lock files, inspect live processes, start services, or mutate deployment state. It does not simulate every production filesystem failure, physical disk-full behavior, retention/rotation execution policy, service-manager restart execution, live trading, signing, broadcasts, withdrawals, bridges, exchange/RPC calls, wallet custody, or production readiness.
