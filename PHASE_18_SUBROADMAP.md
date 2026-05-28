# PHASE_18_SUBROADMAP.md

## Phase

Phase 18 — Agentic Handoff Package

## Status

Completed for ChatGPT Project Mode after Phase 18 agentic handoff package boundary patch and available validation.

## Governance Preconditions

- `ARCHITECTURE.md` reviewed before implementation.
- `ROADMAP.md` reviewed before implementation.
- `PHASE_17_SUBROADMAP.md` reviewed as the prior active phase.
- `AGENTS.md` reviewed before implementation.
- `PRODUCTION_GAP_TRACKER.md` reviewed before implementation.
- `HANDOFF_CONTEXT.md` and `STRUCTURE_MANIFEST.md` reviewed before implementation.
- `python3 scripts/validate_structure.py` passed before implementation.

## Scope

Create deterministic handoff-package models, prompts, and checklists for external coding agents and human maintainers.

Phase 18 is documentation- and handoff-focused only. It must preserve live-funds blockers, unresolved production gaps, and the distinction between ChatGPT-mode framework completion and external validation that has not been executed.

## In Scope

- Add `arb-core::handoff` deterministic model/trait boundary.
- Model handoff package configuration, target agent profiles, handoff artifacts, package review requests, and review records.
- Add conservative handoff package construction that references authoritative governance files.
- Add future-agent prompts/checklists in a dedicated `handoff/` directory.
- Update CLI/status output to report Phase 18 availability.
- Update `scripts/validate_structure.py` to require Phase 18 files.
- Update governance files, manifest, and gap tracker.

## Out of Scope

- Executing external agents.
- Calling Codex, Cursor, Jules, Claude, GitHub, CI, cloud, exchange, RPC, wallet, or messaging APIs.
- Creating credentials, API keys, bot tokens, wallet keys, seed phrases, mnemonics, provider tokens, signing material, or secret-bearing examples.
- Performing production hardening.
- Running release builds, image scans, staging deployments, load tests, penetration tests, rollback drills, incident drills, or live exchange/RPC validation.
- Enabling live trading, live adapter submission, signing, broadcasts, withdrawals, bridges, or public service exposure.
- Claiming production readiness or live-funds approval.

## Implementation Steps

1. Re-read authoritative governance files and confirm Phase 18 is next.
2. Run `python3 scripts/validate_structure.py` before code changes.
3. Create this `PHASE_18_SUBROADMAP.md` before implementation.
4. Add `crates/arb-core/src/handoff.rs` deterministic handoff package boundary.
5. Export Phase 18 types from `arb-core`.
6. Surface Phase 18 status in `arb-agent`.
7. Add `handoff/` documentation package for external agents and maintainers.
8. Update `scripts/validate_structure.py` required files.
9. Re-run available validations.
10. Update governance and handoff docs.
11. Recompute `STRUCTURE_MANIFEST.md`.
12. Prepare a commit-ready repository state.

## Acceptance Criteria

- `PHASE_18_SUBROADMAP.md` exists before implementation.
- `crates/arb-core/src/handoff.rs` exists and is exported.
- `handoff/AGENTIC_HANDOFF_PACKAGE.md` exists.
- `handoff/FUTURE_AGENT_PROMPTS.md` exists.
- `handoff/EXTERNAL_VALIDATION_CHECKLIST.md` exists.
- Handoff package records explicitly preserve unresolved gaps and live-funds blockers.
- Handoff package records reject secret-bearing artifacts, live-funds approval, public-exposure approval, external execution claims, and production-readiness claims.
- Structure validator passes.
- Cargo/Rust validation is attempted only if toolchain exists and otherwise tracked as deferred.

## Validation Commands

Available in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Required externally before production claims:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
cargo audit or approved dependency audit equivalent
SBOM generation and review
container build and image scan
systemd hardening validation
ARM target validation
load and soak tests
penetration test
rollback and incident-response drills
staging deployment validation
production readiness review
```

## Security Requirements

- No secrets in repository artifacts.
- No real credentials in prompts, checklists, Markdown, TOML, logs, service units, containers, or handoff records.
- No future-agent prompt may instruct an agent to bypass governance or policy.
- No future-agent prompt may instruct an agent to enable live funds or production deployment without external evidence and human approval.
- No public bind, live trading, signing, broadcasts, withdrawals, bridges, or external submission from Phase 18.
- All handoffs must preserve the latest gap tracker and explicit external-validation blockers.

## Rollback Plan

1. Remove `crates/arb-core/src/handoff.rs`.
2. Remove handoff exports from `crates/arb-core/src/lib.rs`.
3. Revert Phase 18 status text from `crates/arb-agent/src/main.rs`.
4. Remove `handoff/` documentation package.
5. Remove Phase 18 required files from `scripts/validate_structure.py`.
6. Revert governance docs to Phase 17 state.
7. Re-run `python3 scripts/validate_structure.py`.

## Completion State

Completed for ChatGPT Project Mode after Phase 18 handoff-package boundary patch and available validation.

Rust, Cargo, CI, external agent execution, container, systemd, ARM, SAST, dependency audit, SBOM, image scan, cloud/staging deployment, load, penetration, rollback, incident-response, live exchange/RPC, and production readiness validation remain external unless explicitly run in a capable environment.
