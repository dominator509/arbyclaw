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
7. Paper execution replay/backtest validation, direct audit integration, and external sandbox/live calibration evidence
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

The audit subsystem provides typed redacted events, secret-like metadata rejection, append-only JSONL records, and a local hash chain. This improves accountability but is not yet sufficient for live funds. Current workspace Rust validation exists for the modeled boundary, and the SQLite WAL state store now has local integrity/checkpoint/reopen/backup-restore/multi-handle durability validation. Audit crash recovery testing, concurrent append testing, filesystem permission hardening, external production-host SQLite validation, retention policy, and execution-adapter integration remain required before production use.


## Phase 5 Market Data and Fee Boundary

The market-data and fee subsystems provide normalized read-only models and provider trait boundaries only. They do not call live exchanges, DEXes, RPC providers, or paid data providers. They do not load credentials, sign transactions, submit orders, mutate balances, or approve execution. Live provider implementations, rate-limit handling, data-quality validation, fee-schedule verification, and stale-data integration tests remain required before production use.


## Phase 6 Paper Connector Boundary

The paper connector subsystem provides deterministic in-memory market data, static paper fee schedules, policy-gated paper execution reports, local report checkpoint helpers, local simulated balance ledgering, local realistic fill modeling, local venue matching profiles, adverse-selection modeling, reference-only calibration records, paper ledger replay validation, and local paper backtest execution only. It can consume caller-supplied order-book depth, model partial and unfilled outcomes, apply latency and queue-position assumptions, and release unfilled reserved notional through the paper ledger. It does not call live venues, connect to Web3 RPCs, load secrets, sign transactions, withdraw funds, or mutate real balances. Paper execution is not proof of production profitability. Direct append-only audit journal integration, external sandbox/live calibration evidence, and production-host runtime validation remain required before strategy decisions may be trusted beyond local paper fixtures.

## Phase 7 CEX Connector Framework Boundary

The CEX connector framework defines typed venue profiles, capability declarations, order request models, policy-gated validation, and connector traits only. It does not call exchange REST APIs, open WebSocket streams, load exchange credentials, read live balances, submit orders, cancel orders, withdraw funds, or mutate balances. Live CEX orders are explicitly unavailable in Phase 7. Exchange-specific adapters, sandbox tests, credential-scope validation, rate-limit verification, fee-schedule verification, audit/state integration, terms review, and jurisdiction review remain required before any real exchange use.

## Phase 8 DEX/Web3 Connector Framework Boundary

The DEX/Web3 connector framework defines typed chain profiles, token profiles, router profiles, capability declarations, swap quote models, local transaction simulation request/response models, policy-gated validation, and connector traits only. It does not call RPC endpoints, call router or aggregator APIs, load wallet keys, build arbitrary LLM-generated contract calls, sign transactions, broadcast transactions, execute bridges, approve unknown spenders, or mutate balances. Live DEX swaps, live RPC transaction simulation, signing, and broadcasts are explicitly unavailable in Phase 8. RPC adapters, signer/custody integration, testnet simulation, spender approval hygiene, gas/slippage/MEV validation, audit/state integration, protocol terms review, and jurisdiction review remain required before any real Web3 use.



## Phase 9 Opportunity Engine Boundary

The opportunity engine defines deterministic discovery and ranking models only. It consumes supplied normalized quotes and fee schedules, applies market-data freshness checks, calculates fee-aware edges, and returns non-executing opportunity records. It does not create execution intents, submit orders, sign transactions, call exchange APIs, call DEX/router/RPC APIs, withdraw funds, bridge assets, mutate balances, or bypass policy. Full triangular path search, inventory-aware sizing, depth-aware slippage, durable audit/state lifecycle integration, and live-data validation remain required before any strategy output may be trusted with real funds.

## Phase 10 Execution Planner Boundary

The execution planner defines draft-only planning models. It converts validated opportunity candidates into per-leg `ExecutionIntent` records, evaluates each draft intent through the policy engine, captures redacted policy outcomes, models sequencing plus failure boundaries, and can checkpoint draft plans to local SQLite WAL state. It does not submit to execution adapters, place orders, sign transactions, broadcast transactions, withdraw funds, bridge assets, call exchange APIs, call DEX/router/RPC APIs, or mutate balances. Live scope is rejected by the planner. Local runtime lifecycle wiring can now audit and checkpoint the plan before deterministic adapter-boundary evaluation, and Phase 23 direct paper reports can model local partial fills. Production runtime validation, planner-integrated partial-fill/cancellation handling, and real adapter orchestration remain required before any plan can be used beyond local deterministic lifecycle review.

## Phase 11 Execution Adapter Framework Boundary

The execution-adapter framework defines deterministic adapter-boundary records only. It consumes `ExecutionPlanDraft` records, revalidates each intent through the policy engine, and produces attempt, fill, and reconciliation records without external submission. It does not call exchange APIs, call DEX/router/RPC APIs, sign transactions, broadcast transactions, withdraw funds, bridge assets, mutate real balances, or load secrets. Live scope and external adapter submission are explicitly unavailable in Phase 11. Durable audit/state integration, sandbox adapters, exchange-specific live adapters, signer/custody integration, kill-switch enforcement, and duplicate-submission prevention remain required before any real execution use.


## Phase 12 Communications and CLI Boundary

The communications/CLI boundary is local and deterministic only. It defines typed operator commands, notification payloads, routing records, dispatch records, redaction/truncation helpers, and secret-like text checks. Unsafe command requests for live execution, withdrawals, bridges, signing, and broadcasts are rejected. Outbound network delivery is disabled, and dispatch records preserve `outbound_network_used = false`. No platform tokens, bot credentials, SMTP credentials, webhooks, remote command authentication, live trading, signing, broadcasts, or policy bypass are implemented. Future real communications adapters require authentication, authorization, redaction validation, audit/state persistence, rate limiting, platform-token custody, injection-resistance tests, and external validation.


## Phase 13 Embedded Dashboard Boundary

The embedded-dashboard subsystem provides local snapshot, panel, and render-record models only. It does not start an HTTP server, expose a network listener, provide remote access, authenticate browser sessions, execute dashboard commands, enable live controls, load credentials, render secret-like text, sign transactions, broadcast transactions, withdraw funds, bridge funds, or submit orders. Public exposure is explicitly rejected by the boundary model. Real dashboard hosting must be designed and validated later with loopback defaults, authentication, authorization, CSRF protection, clickjacking protections, rate limiting, audit/state integration, secure headers, UX review, and penetration testing before production use.

## Phase 14 Observability and Runbook Boundary

The observability/runbook subsystem provides deterministic local health, structured-log, metric, and runbook records only. It does not start metrics endpoints, expose public telemetry, ship logs, send alerts, call OpenTelemetry/Prometheus/SIEM/PagerDuty/Slack/email/webhook providers, load secrets, sign transactions, broadcast transactions, or call exchanges/RPC providers. Secret-like observability text is redacted before collection records are returned. Real tracing subscribers, exporters, authenticated metrics endpoints, alert routing, log retention/rotation, audit/state persistence, and incident drills remain required before production observability use.


## Phase 15 Testing, Fuzzing, and Backtesting Boundary

The testing/fuzzing/backtesting subsystem provides deterministic validation planning models, and Phase 24 adds local paper backtest execution over caller-supplied fixtures. It does not invoke external fuzzers, run live-network tests, download live market data, load credentials, submit orders or swaps, sign transactions, broadcast transactions, withdraw funds, bridge funds, or call exchanges/RPC providers. Credential-bearing fixtures are rejected, and secret-like operator labels are redacted in local validation records. Property-testing dependencies, fuzzing engine setup, broader curated corpus review, CI-scale replay validation, production backtest correctness checks, load tests, and penetration tests remain required before production validation claims.

## Phase 16 Packaging and Deployment Boundary

The packaging/deployment boundary provides deterministic plan records and example deployment templates only. It must not be treated as a completed release process. The boundary rejects public network exposure, live trading enablement, embedded secret material, build claims, deployment claims, and production deployment claims. Container recipes, systemd units, ARM notes, and deployment notes require external validation before use. Do not place secrets in container build contexts, service units, environment files, Markdown, logs, shell history, or release artifacts.



## Phase 17 External Hardening Evidence Boundary

The external hardening boundary provides local evidence plans and review records only. It must not be treated as completed production hardening. The boundary rejects external action execution claims, production-readiness claims, live-funds approval, public-exposure approval, and secret-bearing evidence. External evidence references must remain non-secret and must be generated in capable external environments before any production, live-funds, public-service, or deployment readiness claim is made.

## Phase 18 Agentic Handoff Boundary

The agentic handoff package is documentation and deterministic local model state only. It must not contain credentials, secret-bearing evidence, production approvals, live-funds approvals, public-exposure approvals, or instructions to bypass policy. Future-agent prompts must preserve governance reconciliation, unresolved gaps, external validation blockers, and live-trading denials. External coding agents, CI systems, cloud services, exchanges, RPC providers, and messaging systems are not invoked by Phase 18.

## Phase 19 Runtime Lifecycle Boundary

The runtime lifecycle boundary is local and deterministic only. It appends redacted audit events, persists the execution-plan draft before adapter evaluation, runs the deterministic execution-adapter boundary, persists the adapter run, and appends adapter-completion audit evidence. It rejects live-scope lifecycle requests before audit/state mutation and preserves `external_submission_performed = false` and `live_execution_performed = false`. It does not call exchanges, call RPC providers, submit orders, sign transactions, broadcast transactions, withdraw funds, bridge funds, load secrets, start services, expose public interfaces, or approve production readiness.

## Phase 20 SQLite WAL Durability Boundary

The SQLite WAL durability boundary is local and non-secret only. It validates WAL mode, synchronous FULL, SQLite integrity check, WAL checkpoint truncate, primary reopen, checkpointed backup/restore, and multi-handle checkpoint visibility. It records outcome booleans rather than database contents or filesystem paths, and preserves `live_execution_performed = false`, `external_network_used = false`, and `secret_material_recorded = false`. It does not validate production-host crash/restart behavior, disk-full handling, service deployment, live trading, signing, broadcasts, wallet custody, or production readiness.

## Phase 21 Paper Balance Ledger Boundary

The paper balance ledger boundary is local and simulated only. It tracks caller-supplied paper balances, reserves quote notional for paper intents, settles filled paper reports with net paper P&L, and fails closed on insufficient balances or missing reservations. It does not read real balances, mutate real accounts, call exchanges or RPC providers, sign transactions, broadcast transactions, withdraw funds, bridge funds, load secrets, or approve production readiness.

## Phase 22 Crash/Restart Durability Boundary

The crash/restart durability boundary is local and non-secret only. It validates that committed SQLite WAL checkpoints survive abrupt child-process termination and can be reopened with integrity checks by a fresh parent process. It does not simulate power loss, disk-full conditions, deployment service managers, public exposure, live trading, real exchange/RPC calls, signing, broadcasts, withdrawals, bridges, wallet custody, or production readiness.

## Phase 23 Realistic Paper Fill Boundary

The realistic paper fill boundary is local-only. It uses supplied normalized order-book snapshots to model full, partial, or unfilled paper outcomes with deterministic depth consumption, latency, queue-position haircuts, average price, slippage, and ledger settlement. It does not call exchanges, RPC providers, routers, wallets, signers, or external adapters, and it does not approve live execution or production readiness.

## Phase 24 Paper Replay, Calibration, Backtest, and Runtime Validation Boundary

The Phase 24 paper validation boundary is local-only. It models venue tick/step/min-notional behavior, adverse-selection penalties, reference-only calibration records, paper ledger replay, local historical-fixture backtests, and runtime validation records while preserving `production_ready = false`. It does not call sandbox or live exchanges, use live networks, download data, embed artifact contents, store secrets, mutate real balances, submit external orders, sign transactions, broadcast transactions, withdraw funds, bridge funds, or approve production readiness.
