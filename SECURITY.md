# SECURITY.md

## Current Security Status

This repository is not ready for live funds, live exchange keys, wallet custody, or production deployment.

## Secret Handling Rules

Never commit or paste the following into this repository, Markdown files, TOML files, logs, prompts, tickets, or chat transcripts:

- Exchange API keys
- Provider credentials
- Wallet private keys
- Seed phrases
- Mnemonics
- Bearer tokens
- RPC provider secrets
- Notification service credentials

Phase 2 supports reference-only secret configuration through environment variable names and future encrypted-keystore aliases. The encrypted keystore backend is not implemented yet.

## Live Trading Rules

Live execution remains blocked until all of the following exist and are validated:

1. Encrypted secret backend
2. Deny-by-default policy engine
3. Durable redacted audit journal
4. Market-data freshness checks and externally verified fee models
5. Draft-only execution planner
6. Simulated or paper-trading validation
7. Paper execution audit integration and scenario validation
8. Constrained signer boundary
9. Deterministic opportunity discovery and policy-preflight execution planning
10. Execution adapters with durable audit/state preconditions
11. External adapter submission controls with sandbox/live validation
12. Real dashboard hosting with authentication, authorization, CSRF protection, and no live-control bypass
13. Observability/runtime telemetry that cannot leak secrets or expose public control surfaces
14. Testing/fuzzing/backtesting harnesses that cannot use live networks, secrets, signing, broadcasts, or live execution
15. Packaging/deployment boundaries that cannot claim builds, service installs, public exposure, embedded secrets, live trading, or production deployment
16. External-hardening evidence boundaries that cannot claim external execution, production readiness, live-funds approval, public exposure, or secret-bearing evidence
17. External Rust, CI, security, packaging, deployment, and runtime validation

## Vulnerability Handling

Until a private disclosure process exists, treat all security findings as blocking defects and do not enable live modes.


## Phase 3 Policy Boundary

The policy engine is deny-by-default and must be called by every future execution adapter before orders, swaps, transfers, withdrawals, or signing operations. Phase 3 explicitly denies withdrawals, bridge routes, unknown destinations, LLM-generated destinations, stale market data, over-limit risk, and live runtime approval by default.

## Phase 4 Audit Boundary

The audit subsystem provides typed redacted events, secret-like metadata rejection, append-only JSONL records, and a local hash chain. This improves accountability but is not yet sufficient for live funds. Current workspace Rust validation exists for the modeled boundary, while crash recovery testing, concurrent append testing, filesystem permission hardening, production SQLite WAL durability validation, retention policy, and execution-adapter integration remain required before production use.


## Phase 5 Market Data and Fee Boundary

The market-data and fee subsystems provide normalized read-only models and provider trait boundaries only. They do not call live exchanges, DEXes, RPC providers, or paid data providers. They do not load credentials, sign transactions, submit orders, mutate balances, or approve execution. Live provider implementations, rate-limit handling, data-quality validation, fee-schedule verification, and stale-data integration tests remain required before production use.


## Phase 6 Paper Connector Boundary

The paper connector subsystem provides deterministic in-memory market data, static paper fee schedules, policy-gated paper execution reports, and local report checkpoint helpers only. It does not call live venues, connect to Web3 RPCs, load secrets, sign transactions, withdraw funds, or mutate real balances. Paper execution is not proof of production profitability. Paper balance ledgering, realistic fill/slippage/latency modeling, audit/runtime lifecycle integration, and backtesting scenario validation remain required before strategy decisions may be trusted.

## Phase 7 CEX Connector Framework Boundary

The CEX connector framework defines typed venue profiles, capability declarations, order request models, policy-gated validation, and connector traits only. It does not call exchange REST APIs, open WebSocket streams, load exchange credentials, read live balances, submit orders, cancel orders, withdraw funds, or mutate balances. Live CEX orders are explicitly unavailable in Phase 7. Exchange-specific adapters, sandbox tests, credential-scope validation, rate-limit verification, fee-schedule verification, audit/state integration, terms review, and jurisdiction review remain required before any real exchange use.

## Phase 8 DEX/Web3 Connector Framework Boundary

The DEX/Web3 connector framework defines typed chain profiles, token profiles, router profiles, capability declarations, swap quote models, local transaction simulation request/response models, policy-gated validation, and connector traits only. It does not call RPC endpoints, call router or aggregator APIs, load wallet keys, build arbitrary LLM-generated contract calls, sign transactions, broadcast transactions, execute bridges, approve unknown spenders, or mutate balances. Live DEX swaps, live RPC transaction simulation, signing, and broadcasts are explicitly unavailable in Phase 8. RPC adapters, signer/custody integration, testnet simulation, spender approval hygiene, gas/slippage/MEV validation, audit/state integration, protocol terms review, and jurisdiction review remain required before any real Web3 use.



## Phase 9 Opportunity Engine Boundary

The opportunity engine defines deterministic discovery and ranking models only. It consumes supplied normalized quotes and fee schedules, applies market-data freshness checks, calculates fee-aware edges, and returns non-executing opportunity records. It does not create execution intents, submit orders, sign transactions, call exchange APIs, call DEX/router/RPC APIs, withdraw funds, bridge assets, mutate balances, or bypass policy. Full triangular path search, inventory-aware sizing, depth-aware slippage, durable audit/state lifecycle integration, and live-data validation remain required before any strategy output may be trusted with real funds.

## Phase 10 Execution Planner Boundary

The execution planner defines draft-only planning models. It converts validated opportunity candidates into per-leg `ExecutionIntent` records, evaluates each draft intent through the policy engine, captures redacted policy outcomes, models sequencing plus failure boundaries, and can checkpoint draft plans to local SQLite WAL state. It does not submit to execution adapters, place orders, sign transactions, broadcast transactions, withdraw funds, bridge assets, call exchange APIs, call DEX/router/RPC APIs, or mutate balances. Live scope is rejected by the planner. Durable audit/runtime lifecycle integration, adapter handoff, partial-fill/cancellation handling, and runtime orchestration remain required before any plan can be used beyond draft review.

## Phase 11 Execution Adapter Framework Boundary

The execution-adapter framework defines deterministic adapter-boundary records only. It consumes `ExecutionPlanDraft` records, revalidates each intent through the policy engine, and produces attempt, fill, and reconciliation records without external submission. It does not call exchange APIs, call DEX/router/RPC APIs, sign transactions, broadcast transactions, withdraw funds, bridge assets, mutate real balances, or load secrets. Live scope and external adapter submission are explicitly unavailable in Phase 11. Durable audit/state integration, paper balance ledgering, sandbox adapters, exchange-specific live adapters, signer/custody integration, kill-switch enforcement, and duplicate-submission prevention remain required before any real execution use.


## Phase 12 Communications and CLI Boundary

The communications/CLI boundary is local and deterministic only. It defines typed operator commands, notification payloads, routing records, dispatch records, redaction/truncation helpers, and secret-like text checks. Unsafe command requests for live execution, withdrawals, bridges, signing, and broadcasts are rejected. Outbound network delivery is disabled, and dispatch records preserve `outbound_network_used = false`. No platform tokens, bot credentials, SMTP credentials, webhooks, remote command authentication, live trading, signing, broadcasts, or policy bypass are implemented. Future real communications adapters require authentication, authorization, redaction validation, audit/state persistence, rate limiting, platform-token custody, injection-resistance tests, and external validation.


## Phase 13 Embedded Dashboard Boundary

The embedded-dashboard subsystem provides local snapshot, panel, and render-record models only. It does not start an HTTP server, expose a network listener, provide remote access, authenticate browser sessions, execute dashboard commands, enable live controls, load credentials, render secret-like text, sign transactions, broadcast transactions, withdraw funds, bridge funds, or submit orders. Public exposure is explicitly rejected by the boundary model. Real dashboard hosting must be designed and validated later with loopback defaults, authentication, authorization, CSRF protection, clickjacking protections, rate limiting, audit/state integration, secure headers, UX review, and penetration testing before production use.

## Phase 14 Observability and Runbook Boundary

The observability/runbook subsystem provides deterministic local health, structured-log, metric, and runbook records only. It does not start metrics endpoints, expose public telemetry, ship logs, send alerts, call OpenTelemetry/Prometheus/SIEM/PagerDuty/Slack/email/webhook providers, load secrets, sign transactions, broadcast transactions, or call exchanges/RPC providers. Secret-like observability text is redacted before collection records are returned. Real tracing subscribers, exporters, authenticated metrics endpoints, alert routing, log retention/rotation, audit/state persistence, and incident drills remain required before production observability use.


## Phase 15 Testing, Fuzzing, and Backtesting Boundary

The testing/fuzzing/backtesting subsystem provides deterministic validation planning models only. It does not invoke external fuzzers, run live-network tests, download live market data, load credentials, submit orders or swaps, sign transactions, broadcast transactions, withdraw funds, bridge funds, or call exchanges/RPC providers. Credential-bearing fixtures are rejected, and secret-like operator labels are redacted in local validation records. Property-testing dependencies, fuzzing engine setup, curated corpus review, deterministic replay validation, backtest correctness checks, load tests, and penetration tests remain required before production validation claims.

## Phase 16 Packaging and Deployment Boundary

The packaging/deployment boundary provides deterministic plan records and example deployment templates only. It must not be treated as a completed release process. The boundary rejects public network exposure, live trading enablement, embedded secret material, build claims, deployment claims, and production deployment claims. Container recipes, systemd units, ARM notes, and deployment notes require external validation before use. Do not place secrets in container build contexts, service units, environment files, Markdown, logs, shell history, or release artifacts.



## Phase 17 External Hardening Evidence Boundary

The external hardening boundary provides local evidence plans and review records only. It must not be treated as completed production hardening. The boundary rejects external action execution claims, production-readiness claims, live-funds approval, public-exposure approval, and secret-bearing evidence. External evidence references must remain non-secret and must be generated in capable external environments before any production, live-funds, public-service, or deployment readiness claim is made.

## Phase 18 Agentic Handoff Boundary

The agentic handoff package is documentation and deterministic local model state only. It must not contain credentials, secret-bearing evidence, production approvals, live-funds approvals, public-exposure approvals, or instructions to bypass policy. Future-agent prompts must preserve governance reconciliation, unresolved gaps, external validation blockers, and live-trading denials. External coding agents, CI systems, cloud services, exchanges, RPC providers, and messaging systems are not invoked by Phase 18.
