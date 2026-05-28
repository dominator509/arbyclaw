# Agentic Handoff Package

## Purpose

This package gives future coding agents and human maintainers a deterministic continuation path for the ArbyClaw repository.

It is not a production-readiness approval, deployment record, live-funds approval, security certification, or external validation result.

## Authoritative Files

Future agents must read these files before making changes:

1. `HANDOFF_CONTEXT.md`
2. `STRUCTURE_MANIFEST.md`
3. `ARCHITECTURE.md`
4. `ROADMAP.md`
5. active `PHASE_X_SUBROADMAP.md`
6. `AGENTS.md`
7. `PRODUCTION_GAP_TRACKER.md`
8. available source files and tests

## Required Opening Procedure

1. Use the latest repository checkout or approved archive.
2. Prefer the newest complete ZIP over older partial files.
3. Run `python3 scripts/validate_structure.py` before modifying files.
4. Reconcile current phase, completed phases, unresolved gaps, and subsystem boundaries.
5. Stop on governance conflicts and record assumptions, risks, blockers, and safest next step.
6. Create the next `PHASE_X_SUBROADMAP.md` before implementation if a new phase begins.
7. Keep patches small, reversible, and subsystem-isolated.

## Non-Negotiable Safety Boundaries

Do not add or enable:

- live trading
- live order placement
- live DEX swaps
- wallet signing
- transaction broadcasts
- withdrawals
- bridges
- real exchange calls
- real RPC calls
- public dashboard exposure
- public metrics exposure
- outbound messaging delivery
- external agent execution from the application
- production deployment claims
- production-readiness claims
- live-funds approval

Do not place credentials or secret material in Markdown, TOML, source code, logs, service units, containers, prompts, generated artifacts, screenshots, or chat.

## Current Checkpoint

The current checkpoint after Phase 18 includes deterministic model and documentation boundaries through:

- governance
- Rust workspace scaffold
- config and secret references
- policy engine
- audit and state boundaries
- market data and fee models
- paper connectors
- CEX connector framework
- DEX/Web3 connector framework
- opportunity engine
- execution planner
- execution adapter boundary
- communications/CLI boundary
- embedded dashboard boundary
- observability/runbook boundary
- testing/fuzzing/backtesting boundary
- packaging/deployment boundary
- external hardening evidence boundary
- agentic handoff package boundary

## Validation Reality

Validated in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Still required externally before production claims:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
```

Additional external evidence is required for dependency audit, SBOM review, image scanning, service hardening, ARM validation, staging deployment, load testing, penetration testing, rollback drills, incident drills, exchange sandbox validation, DEX/RPC sandbox validation without broadcasts, custody review, compliance review, and production readiness review.

## Handoff Rule

A future agent may continue implementation only after preserving the latest gap tracker and proving that its change does not weaken policy gates, redaction, auditability, rollback readiness, or live-funds blockers.
