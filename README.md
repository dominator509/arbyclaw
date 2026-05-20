# Fully Autonomous Crypto Arbitrage Agent

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
- Phase 9 deterministic opportunity-engine discovery/ranking models implemented.
- Phase 10 draft-only execution-planner models, per-leg intent generation, policy preflight outcomes, sequencing, and failure-mode boundaries implemented.
- Phase 11 deterministic execution-adapter framework models, policy revalidation, attempt/fill/reconciliation records, and external-submission blocks implemented.
- Phase 12 communications/CLI command routing, notification models, redaction checks, and local dispatch-record boundaries implemented.
- Phase 13 embedded dashboard snapshot, panel, local rendering, secret-redaction, and fail-closed server-binding boundaries implemented.
- Phase 14 observability/runbook local health, structured-log, metric, runbook, redaction, and fail-closed telemetry endpoint boundaries implemented.
- Phase 15 testing/fuzzing/backtesting validation-plan, fixture, fuzz corpus, and backtest scenario boundaries implemented.
- Phase 16 packaging/deployment plan, target, release-gate, rollback-step, and example deployment documentation boundaries implemented.
- Phase 17 external hardening evidence, release-blocker, production-readiness checklist, and incident-drill template boundaries implemented.
- Phase 18 agentic handoff package records, future-agent prompts, governance checklist, and external validation checklist boundaries implemented.
- Live trading is not implemented.
- Wallet signing is not implemented.
- External adapter submission, exchange-specific live CEX adapters, live DEX/Web3 RPC/signing/broadcast adapters, real outbound communications adapters, real dashboard hosting, real observability/exporter/alert runtime, real fuzzing engines, real backtest execution, real build execution, container validation, systemd installation, ARM validation, external agent execution, and production deployment are not implemented.
- Live REST/WebSocket/RPC market-data providers are not implemented.
- Secrets must not be committed or pasted into chat, Markdown, TOML, logs, or prompts.

## Config Usage

The committed `config.example.toml` is observe-only and contains no raw credentials.

```bash
arb-agent --config config.example.toml
```

Secret configuration must use references only, such as environment variable names or future encrypted-keystore aliases. Raw API keys, wallet keys, seed phrases, mnemonics, and tokens are forbidden in repository files.

## Intended Local Validation

Run these commands in a Rust-enabled environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_structure.py
```

## Safety Position

This repository is not ready for live funds. Future live execution must remain gated behind typed config, encrypted secret handling, deny-by-default policy checks, durable redacted audit journaling, validated market-data freshness, simulation where available, and constrained signer boundaries. Phase 6 paper primitives, Phase 7 CEX framework primitives, Phase 8 DEX/Web3 framework primitives, Phase 9 opportunity-engine primitives, Phase 10 execution-planner draft primitives, Phase 11 execution-adapter framework primitives, Phase 12 communications/CLI primitives, Phase 13 embedded-dashboard primitives, Phase 14 observability/runbook primitives, Phase 15 testing/backtesting validation primitives, Phase 16 packaging/deployment planning primitives, Phase 17 external-hardening evidence/checklist primitives, and Phase 18 agentic handoff primitives exist, but Rust validation, external adapter submission, exchange-specific live adapters, live DEX/RPC adapters, signer/broadcast controls, live provider validation, realistic fill simulation, paper balance ledgering, real outbound communications adapters, authenticated remote command channels, real dashboard hosting/authentication, real metrics/exporter/alert runtime, real property/fuzz/backtest runner execution, real packaging/deployment validation, real external hardening execution, external agent execution validation, opportunity/planner/adapter/CEX/DEX/communications/dashboard/observability/testing/packaging/hardening/handoff audit-state integration, crash testing, concurrent append testing, filesystem hardening, SQLite WAL persistence, and live execution integration remain incomplete.


## Phase 12 Communications and CLI Boundary

The communications/CLI boundary defines deterministic local command and notification models only. It parses and routes local operator commands into typed records, rejects live execution, withdrawal, bridge, signing, and broadcast requests, and produces local notification dispatch records without outbound network delivery. It does not use messaging platform tokens, call Telegram/Discord/Slack/email/PagerDuty/webhook APIs, authenticate remote operators, execute trades, bypass policy, sign transactions, broadcast transactions, or load secrets. Durable audit/state integration, real channel adapters, authn/authz, rate limiting, operator UX validation, and Rust/Cargo validation remain required before production communications use.


## Phase 13 Embedded Dashboard Boundary

The embedded-dashboard boundary defines deterministic local dashboard snapshot and rendering models only. It can render sanitized in-process records for panels such as safety, market data, opportunities, planner status, execution-adapter status, communications, audit/state, and gaps. It rejects public exposure, rejects HTTP server startup, redacts secret-like display text, and keeps live controls disabled. It does not start a web server, expose localhost or public routes, authenticate users, execute commands, submit trades, sign transactions, broadcast transactions, or load secrets. Real dashboard hosting, authentication/session handling, CSRF protection, durable dashboard state, UX validation, penetration testing, and Rust/Cargo validation remain required before production dashboard use.

## Phase 14 Observability and Runbook Boundary

The observability/runbook boundary defines deterministic local health, structured-log, metric, and runbook records only. It can collect sanitized in-process records for operator review, but it rejects metrics endpoint startup, public telemetry exposure, outbound alert delivery, and secret-like observability text. It does not start Prometheus/OpenTelemetry exporters, ship logs, call SIEM or alerting providers, execute incident automation, sign transactions, broadcast transactions, load secrets, or make exchange/RPC calls. Real observability runtime integration, authenticated metrics endpoint design, alert routing, log retention/rotation, incident drills, audit/state integration, and Rust/Cargo validation remain required before production observability use.

## Phase 15 Testing, Fuzzing, and Backtesting Boundary

The testing/fuzzing/backtesting boundary defines deterministic validation plans, test case metadata, fixture metadata, fuzz corpus definitions, backtest dataset definitions, and local validation run records only. It validates that planned tests remain local and fail closed when external fuzzer invocation, live network testing, live execution, signing, broadcasts, or credential-bearing fixtures are requested. It does not invoke `cargo`, launch fuzzers, download market data, call exchanges/RPC providers, submit orders, sign transactions, broadcast transactions, or prove strategy profitability. Real Rust test execution, property-test runners, fuzzing engines, curated fixture corpora, replay/backtest execution, CI gates, load tests, penetration tests, and production validation remain required.

## Phase 16 Packaging and Deployment Boundary

The packaging/deployment boundary defines deterministic package target plans, service hardening metadata, release gates, rollback steps, and local package/deployment records only. It validates that plans do not claim builds, installs, public exposure, live trading, embedded secrets, or production deployment. Example container, systemd, ARM, and deployment notes are templates only; no container image, service unit, ARM binary, cloud deployment, rollback drill, or production release has been validated. Real Rust release builds, container builds, systemd validation, ARM target validation, SBOM/dependency audit, rollback drills, load tests, penetration tests, and deployment validation remain required.



## Phase 17 External Production Hardening Evidence Boundary

The external hardening boundary defines deterministic evidence records, hardening plans, release blockers, and local review records only. It validates that ChatGPT-mode records do not claim external action execution, production readiness, live-funds approval, public exposure approval, or secret-bearing evidence. The hardening runbook, readiness checklist, and incident-response drill template are operator checklists only; no CI run, release build, dependency audit, SBOM review, image scan, staging deployment, load test, penetration test, rollback drill, incident drill, live exchange validation, DEX/RPC validation, or production readiness review has been executed in this environment.

## Phase 18 Agentic Handoff Boundary

The handoff subsystem provides deterministic package records, continuation prompts, governance reconciliation checklists, and external validation checklists for future agents and maintainers. It does not call external coding-agent services, execute CI, run Rust validation, deploy infrastructure, approve production readiness, approve public exposure, approve live funds, or store credentials. See `handoff/AGENTIC_HANDOFF_PACKAGE.md`, `handoff/FUTURE_AGENT_PROMPTS.md`, and `handoff/EXTERNAL_VALIDATION_CHECKLIST.md`.

