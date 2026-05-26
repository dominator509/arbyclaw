# HANDOFF_CONTEXT.md

## Purpose

This file is a compact continuation checkpoint for the Fully Autonomous Crypto Arbitrage Agent project. It is intended to prevent context drift when continuing in a new chat, external coding agent, local IDE, or CI environment.

## Authoritative Repository State

The current authoritative baseline is the Phase 18 agentic handoff package snapshot, continued from the Phase 17 external hardening ZIP after governance reconciliation and structure validation.

## Completed Phases

- Phase 0 — Governance Initialization
- Phase 1 — Rust Workspace Scaffold
- Phase 2 — Config, Secrets, and Mode Gates
- Phase 3 — Policy Engine and Trust Contract
- Phase 4 — Audit Journal and State Store Boundary
- Phase 5 — Market Data Core and Fee Models
- Phase 6 — Simulated/Paper Connectors
- Phase 7 — CEX Connector Framework
- Phase 8 — DEX/Web3 Connector Framework
- Phase 9 — Opportunity Engine
- Phase 10 — Execution Planner
- Phase 11 — Execution Adapters
- Phase 12 — Communications and CLI
- Phase 13 — Embedded Dashboard
- Phase 14 — Observability and Runbooks
- Phase 15 — Testing, Fuzzing, and Backtesting
- Phase 16 — Packaging and Deployment
- Phase 17 — External Production Hardening evidence boundary
- Phase 18 — Agentic Handoff Package

## Current Production Readiness

87% as of Phase 18 governance. Phase 18 did not increase readiness because handoff packaging is documentation/model-only and real external validation remains environment-limited. This percentage is a governance approximation only and does not imply readiness for live funds or production deployment.

## Non-Negotiable Safety State

The project is not ready for:

- real funds
- live exchange credentials
- wallet private keys
- seed phrases
- live CEX orders
- live DEX swaps
- wallet signing
- transaction broadcasts
- bridge execution
- autonomous live execution
- live adapter submission
- real outbound communications
- real dashboard hosting
- real observability/exporter/alert runtime
- real fuzzing engine execution
- real backtest execution
- production deployment
- production release claims
- external agent execution claims
- production container image readiness claims
- systemd service install claims
- ARM build validation claims

The agent must never store API keys, wallet keys, seed phrases, or provider tokens in Markdown, `config.toml`, source code, logs, or audit metadata.

## Current Rust Workspace Shape

Workspace root:

- `Cargo.toml`
- `rust-toolchain.toml`
- `rustfmt.toml`
- `.github/workflows/ci.yml`
- `scripts/validate_structure.py`

Crates:

- `crates/arb-core`
- `crates/arb-agent`

Current `arb-core` modules:

- `config`
- `secrets`
- `policy`
- `audit`
- `state`
- `market_data`
- `fees`
- `paper`
- `cex`
- `dex`
- `opportunity`
- `planner`
- `execution_adapter`
- `communications`
- `dashboard`
- `observability`
- `testing`
- `packaging`
- `hardening`
- `handoff`

## Current Validation Reality

Current local validation evidence exists for the ArbyClaw workspace:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Current GitHub Actions evidence exists for pushed commits on `dominator509/arbyclaw`:

- structure validation
- Rust formatting, workspace check, tests, and clippy
- locked release build
- dependency audit
- CycloneDX SBOM generation
- local-SARIF CodeQL SAST evidence
- example image scan
- Gitleaks secret-pattern scan
- hardening evidence artifact index

This is compile/test/lint and non-secret CI evidence only. It does not validate production deployment, live funds, real exchange/RPC integrations, signing, broadcasts, production containers, systemd, ARM, load testing, penetration testing, rollback drills, incident drills, external agent execution, or production readiness.

## Mandatory Governance Files To Reread Before Any New Work

1. `ARCHITECTURE.md`
2. `ROADMAP.md`
3. active `PHASE_X_SUBROADMAP.md`
4. `AGENTS.md`
5. `PRODUCTION_GAP_TRACKER.md`

For the next implementation or validation phase, create the next `PHASE_X_SUBROADMAP.md` before writing code unless the work is strictly external validation recorded against existing gaps.

## Next Recommended Work

External validation and production hardening evidence generation in a capable environment.

Scope should remain evidence-focused:

- keep Rust/Cargo validation current after each change before claiming compile/test confidence for that exact workspace state
- generate or refresh non-secret evidence for service hardening, ARM validation, staging deployment, load test, penetration test, rollback drill, incident drill, exchange sandbox validation, DEX/RPC sandbox validation without broadcasts, custody review, compliance review, and production readiness review
- preserve all external validation gaps and live-funds blockers until evidence exists
- no live exchange/RPC calls without approved sandbox scope and credentials handled outside the repo
- no real credentials or secrets in chat, Markdown, TOML, logs, images, containers, service units, handoff prompts, or artifacts
- no signing, withdrawals, bridges, broadcasts, or external submission
- no public service exposure without explicit fail-closed bindings and external security validation
- no policy bypass
- do not claim CI, Rust, container, systemd, ARM, cloud, penetration, load, rollback, incident-drill, hardening, production-readiness, or live exchange validation passed unless commands actually run

## Recommended New-Chat Continuation Prompt

Use this compact prompt in a fresh chat after uploading the latest checkpoint ZIP:

```text
You are continuing the Fully Autonomous Crypto Arbitrage Agent project. Unpack and inspect the latest repository ZIP first. Treat HANDOFF_CONTEXT.md, STRUCTURE_MANIFEST.md, ARCHITECTURE.md, ROADMAP.md, AGENTS.md, the latest PHASE_X_SUBROADMAP.md, and PRODUCTION_GAP_TRACKER.md as authoritative. Before any code, run the structure validator if available, reconcile roadmap position, confirm completed phases 0-18, and identify whether the next work is external validation or a new governed phase. Do not implement live trading, signing, secrets, withdrawals, bridges, broadcasts, public web exposure, real messaging tokens, real dashboard hosting, real observability exporters, real fuzzing execution, live network tests, real backtest downloads, real RPC/exchange calls, real cloud deployment, production release claims, production readiness claims, or live-funds approval. Implement only the next governed boundary or record external validation evidence honestly, then update ROADMAP.md and PRODUCTION_GAP_TRACKER.md and provide commit-ready output.
```

## Anti-Drift Rules For Future Agents

- Prefer the latest complete ZIP over partial file uploads when reconstructing state.
- Always run `python3 scripts/validate_structure.py` before modifying code.
- Never rely on memory for file contents; reread repository files from disk.
- Keep each phase patch small and reversible.
- Do not add live network calls until explicit connector phases and validations exist.
- Do not add secrets or secret examples with real-looking values.
- Do not claim Rust validation passed for a new change unless Cargo commands actually ran for that workspace state.
- Do not increase production readiness unless governance, code, and gap tracker are updated consistently.

## Known High-Risk Deferred Areas

- Phase 15 validation plans exist only as local model/trait boundaries; no actual property-test runner, fuzzing engine, curated corpus execution, property/fuzz/backtest CI runner execution, load test, penetration test, or production validation run exists.
- Phase 16 packaging/deployment plans exist only as local model/documentation boundaries; current CI evidence includes release-build and example-only container/image-scan gates, but no production container image validation, systemd install, ARM build, runtime deployment, rollback drill, incident drill, or production release validation exists.
- Phase 17 hardening records and CI evidence paths now cover current workspace Rust validation, locked release build, dependency audit, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening artifact indexing; no staging deployment, load test, penetration test, rollback drill, incident drill, live exchange/RPC validation, or production readiness review was executed.
- Phase 18 handoff records exist only as local prompts/checklists/package models; no external agents were executed and no validation was performed by the handoff boundary.
- No encrypted keystore backend.
- No signer boundary.
- No exchange-specific CEX adapters.
- No live DEX/Web3 adapters, RPC integrations, signer, or broadcasts.
- Opportunity engine advanced validation and live-data validation are incomplete.
- Execution planner and execution-adapter durable audit/state lifecycle integration are incomplete.
- No live execution-adapter submissions.
- No real outbound communications integrations or authenticated remote command channels.
- No real dashboard hosting/authentication.
- Observability runtime exists only as local model/trait boundaries; no exporters, metrics endpoint, log shipping, or alert delivery exist.
- Testing/backtesting runtime exists only as local plan/fixture model boundaries; no actual property/fuzz/backtest runner execution exists.
- No penetration, load, rollback, incident-drill, production container, systemd, ARM, cloud, or deployment validation.
