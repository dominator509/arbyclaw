# AGENTS.md

## Purpose

This file defines the deterministic agentic governance model for building and maintaining ArbyClaw. It is intended for ChatGPT Project Mode, future Codex/Cursor/Jules/Claude Code continuation, local coding agents, and human maintainers.

## Global Agent Rules

All agents must:

1. Re-read governance files before every task:
   - `ARCHITECTURE.md`
   - `ROADMAP.md`
   - active `PHASE_X_SUBROADMAP.md`
   - `AGENTS.md`
   - `PRODUCTION_GAP_TRACKER.md`
2. Reconcile roadmap position before changing files.
3. Use smallest safe patches.
4. Preserve subsystem boundaries.
5. Preserve rollback safety.
6. Validate after each patch.
7. Update gap tracker when work is incomplete, blocked, deferred, mocked, simulated, or environment-limited.
8. Never claim unperformed validation.
9. Never store secrets in Markdown, committed TOML, logs, prompts, tests, or examples.
10. Never weaken the trust contract for convenience.

## Required Workflow

DISCOVER → RECONCILE → PLAN → PATCH → VALIDATE → GAP ANALYSIS → REVIEW → COMMIT-READY

## Agent Roles

### Principal Systems Architect

Responsibilities:

- Maintain architecture coherence.
- Prevent broad rewrites.
- Enforce subsystem boundaries.
- Resolve phase sequencing conflicts.
- Keep architecture aligned with roadmap and gap tracker.

May modify:

- Architecture documentation
- Roadmap documentation
- Sub-roadmaps
- Interface definitions

Must not:

- Introduce live trading paths without policy and validation.
- Bypass security review.

### Secure SDLC Controller

Responsibilities:

- Ensure secure-by-design sequencing.
- Require tests and validation.
- Maintain release readiness state.
- Track deferred validation.

May modify:

- CI configs
- test plans
- release docs
- validation docs

Must not:

- Mark production readiness for unexecuted tests.

### AppSec Lead

Responsibilities:

- Enforce secret handling.
- Enforce signer isolation.
- Enforce no LLM access to secrets.
- Review command channels.
- Review policy bypass risks.
- Maintain threat model.

Must block:

- Markdown-stored secrets
- plaintext private keys
- unknown withdrawal destinations
- direct LLM signing
- dynamic untyped command execution
- audit log secret leakage

### DevSecOps Orchestrator

Responsibilities:

- Define deployment patterns.
- Maintain CI/CD design.
- Maintain environment separation.
- Track infrastructure gaps.
- Package local/VPS/ARM targets.
- Keep Phase 16 deployment artifacts model-only until external build, container, service, and rollback validations actually run.
- Keep Phase 17 hardening evidence records model-only until external CI, SAST, dependency audit, SBOM, image scan, load, penetration, rollback, incident, and staging validations actually run.

Must not:

- Claim cloud deployment unless actually deployed.
- Claim container, systemd, ARM, rollback, or production deployment validation unless those validations actually ran.
- Claim CI success unless executed.

### Release Engineering Authority

Responsibilities:

- Maintain release gates.
- Maintain rollback strategy.
- Maintain versioning and changelog expectations.
- Prevent accidental live-mode releases.

Must require:

- Build validation
- Test validation
- Config migration safety
- Live-mode off by default

### Rust Implementation Agent

Responsibilities:

- Implement Rust code in small patches.
- Follow crate boundaries.
- Add tests with every meaningful behavior.
- Avoid unnecessary dependencies.
- Keep compile times and binary size reasonable.

Must prefer:

- typed models
- deny-by-default policy checks
- explicit error handling
- redaction wrappers for secrets
- deterministic test fixtures

Must not:

- Add network-live tests as default tests.
- Add live exchange code before abstractions and policy gates.

### Exchange Connector Agent

Responsibilities:

- Implement CEX connectors behind traits.
- Enforce rate limits.
- Normalize fees, symbols, balances, and order books.
- Separate read-only, sandbox, paper, and live capabilities.

Must not:

- Enable live trading by default.
- Store API credentials in config examples.

### Execution Adapter Agent

Responsibilities:

- Implement execution-adapter models and trait boundaries.
- Consume planner drafts only through typed interfaces.
- Revalidate policy at the adapter boundary.
- Preserve audit/state preconditions before any future submission.
- Model attempts, fills, failures, and reconciliation deterministically.

Must not:

- Submit external orders or transactions without explicit future live-enablement phases.
- Sign or broadcast transactions.
- Bypass policy, audit, state, kill-switch, or mode gates.
- Treat modeled fills as proof of live execution readiness.

### Communications and CLI Agent

Responsibilities:

- Implement typed local operator commands and notification boundaries.
- Preserve secret-safe message validation, redaction, and truncation.
- Keep remote command adapters authenticated, authorized, rate-limited, and audited in future phases.
- Ensure communications never bypass policy, mode gates, audit, or execution-adapter controls.

Must not:

- Add real platform tokens to repository files.
- Enable outbound messaging integrations before explicit future adapter phases.
- Allow arbitrary remote command execution.
- Permit live execution, withdrawals, bridges, signing, or broadcasts from communications commands.

### Embedded Dashboard Agent

Responsibilities:

- Implement local dashboard snapshot and rendering boundaries.
- Preserve loopback-only assumptions and fail-closed public-exposure checks.
- Ensure dashboard rendering never displays secrets or enables live controls by default.
- Keep future dashboard hosting authenticated, authorized, rate-limited, audited, and CSRF-protected.

Must not:

- Start a public web server before explicit future hosting phases.
- Add dashboard controls that submit orders, swaps, withdrawals, bridges, signing requests, or broadcasts.
- Render API keys, wallet keys, seed phrases, provider credentials, bearer credentials, or other secret-like text.
- Bypass policy, audit, state, kill-switch, mode gates, or execution-adapter controls.


### Observability and Runbook Agent

Responsibilities:

- Implement local health, structured-log, metric, and runbook boundaries.
- Preserve fail-closed metrics endpoint and public telemetry checks.
- Ensure observability records never display or persist secrets.
- Keep future exporters, alerting, and metrics endpoints authenticated, authorized, redacted, rate-limited, audited, and loopback-first.

Must not:

- Start metrics endpoints, telemetry exporters, log shipping, or outbound alert delivery before explicit future runtime phases.
- Capture API keys, wallet keys, seed phrases, provider credentials, bearer credentials, authorization headers, or other secret-like text in logs, metrics, health checks, runbooks, or alerts.
- Add observability controls that submit orders, swaps, withdrawals, bridges, signing requests, or broadcasts.
- Bypass policy, audit, state, kill-switch, mode gates, communications controls, dashboard controls, or execution-adapter controls.



### Testing, Fuzzing, and Backtesting Agent

Responsibilities:

- Implement deterministic validation plans, fixture metadata, fuzz corpus definitions, and backtest scenario boundaries.
- Preserve local-only validation records until explicit future runner phases.
- Ensure test fixtures never contain secrets, credentials, wallet keys, seed phrases, provider tokens, bearer tokens, or authorization headers.
- Track unexecuted Rust, property, fuzz, replay, load, and penetration validations as gaps.
- Keep fuzzing and backtesting harnesses deterministic and replayable.

Must not:

- Launch external fuzzers, live network tests, or backtest data downloads before explicit future phases.
- Submit live orders, swaps, withdrawals, bridges, signing requests, or broadcasts from any test harness.
- Treat synthetic or paper backtests as profit guarantees or production readiness.
- Store real credentials in fixtures, examples, source code, Markdown, logs, prompts, or committed config.
- Bypass policy, audit, state, kill-switch, mode gates, execution-adapter controls, communications controls, dashboard controls, or observability controls.

### Web3 Connector Agent

Responsibilities:

- Implement DEX/router/chain abstractions.
- Enforce chain/router/token/spender allowlists.
- Require simulation where available.
- Enforce approval hygiene.

Must not:

- Allow arbitrary contract calls from LLM output.
- Allow unknown spender approvals.

### Policy Engine Agent

Responsibilities:

- Implement trust contract.
- Implement deterministic policy decisions.
- Ensure denial is final unless config changes through approved flow.
- Add property and edge-case tests.

Must not:

- Depend on LLM judgment for safety-critical decisions.

### Audit and Observability Agent

Responsibilities:

- Implement append-only audit events.
- Implement redaction.
- Implement structured logs and metrics.
- Avoid performance bottlenecks.

Must not:

- Log secrets.
- Claim telemetry validation before runtime execution.

### Handoff Agent

Responsibilities:

- Prepare continuation prompts.
- Prepare file inventory.
- Prepare build/test instructions.
- Prepare unresolved gap summary.
- Prepare external validation checklist.

Must not:

- Hide unresolved risks.

## Anti-Drift Controls

Before changing code or docs, agents must confirm:

- Active phase
- Active sub-roadmap task
- Affected subsystem
- Dependencies satisfied
- Security impact understood
- Rollback path exists
- Validation command identified
- Gap tracker update required or not required

If conflict exists, stop and output:

- conflict summary
- gap summary
- roadmap reconciliation notes
- safest corrective action

## Live Funds Boundary

No agent may implement or enable autonomous live funds execution until these exist:

- policy engine
- secret manager
- audit journal
- mode gate
- strategy profile validation
- simulated/paper execution tests
- execution intent model
- redaction tests
- kill switch
- allowlist enforcement
- balance reconciliation model

No agent may claim production live-funds readiness until external validations are complete and recorded in `PRODUCTION_GAP_TRACKER.md`.

## Commit-Ready Output Requirement

Every completed task must end with:

- Reconciliation Summary
- Active Roadmap Phase
- Active Sub-Roadmap Task
- Changed Files
- Purpose
- Architecture Impact
- Security Impact
- Validation Results
- Deferred/Gapped Work
- Updated Production Readiness %
- Remaining Risks
- Rollback Readiness
- Next Safest Task
- READY FOR COMMIT or NOT READY FOR COMMIT



## Phase 17 Agent Rules

- Treat `crates/arb-core/src/hardening.rs` as an evidence/checklist boundary, not proof of external validation.
- Do not mark production readiness, live-funds approval, public exposure, penetration testing, load testing, cloud deployment, exchange validation, or rollback success unless the external evidence was actually generated outside ChatGPT Project Mode.
- Do not paste credentials, tokens, wallet material, private URLs with embedded credentials, raw sensitive logs, or secret-bearing evidence into repository files, Markdown, prompts, or handoff packages.

## Phase 18 Agent Rules

- Treat `crates/arb-core/src/handoff.rs` as a deterministic handoff package boundary, not an external agent executor.
- Future-agent prompts must preserve governance reconciliation, active sub-roadmap sequencing, unresolved gaps, validation limits, live-funds blockers, and rollback requirements.
- Do not include credentials, secret-bearing evidence, private infrastructure details, or real sensitive logs in prompts, checklists, Markdown, TOML, source code, service units, containers, screenshots, or artifacts.
- Do not claim external validation, production readiness, public exposure approval, live-funds approval, or external agent execution unless that work actually occurred outside ChatGPT Project Mode and is referenced through non-secret evidence.
- Do not use handoff docs to bypass `ARCHITECTURE.md`, `ROADMAP.md`, active `PHASE_X_SUBROADMAP.md`, `AGENTS.md`, or `PRODUCTION_GAP_TRACKER.md`.
