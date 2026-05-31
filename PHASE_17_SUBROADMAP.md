# PHASE_17_SUBROADMAP.md

## Phase

Phase 17 — External Production Hardening.

## Governance Status

Created before Phase 17 implementation. This file is authoritative for the Phase 17 external hardening evidence-boundary patch.

## Preconditions Confirmed

- Phase 0 governance files exist.
- Phase 1 Rust workspace scaffold exists.
- Phase 2 config, secrets, and mode gates exist.
- Phase 3 deny-by-default policy engine exists.
- Phase 4 audit/state boundaries exist.
- Phase 5 market-data and fee boundaries exist.
- Phase 6 deterministic paper connector boundaries exist.
- Phase 7 CEX connector framework boundaries exist.
- Phase 8 DEX/Web3 connector framework boundaries exist.
- Phase 9 opportunity-engine boundaries exist.
- Phase 10 execution-planner boundaries exist.
- Phase 11 execution-adapter framework boundaries exist.
- Phase 12 communications/CLI boundaries exist.
- Phase 13 embedded-dashboard boundaries exist.
- Phase 14 observability/runbook boundaries exist.
- Phase 15 testing/fuzzing/backtesting boundaries exist.
- Phase 16 packaging/deployment boundaries and deployment templates exist.
- `scripts/validate_structure.py` passes before Phase 17 changes.
- Live trading, signing, withdrawals, bridges, broadcasts, real RPC/exchange calls, public web exposure, real observability exporters, real messaging integrations, real backtest downloads, external fuzzing, container builds, service installs, cloud deployment, penetration tests, and production release actions remain unavailable in ChatGPT Project Mode.

## Objective

Add deterministic external-production-hardening evidence boundaries and checklists that make future external validation explicit without claiming that any environment-limited work has been performed by ChatGPT.

## In Scope

- Add external hardening evidence model and trait boundaries.
- Add planned/externally observed evidence records.
- Add production readiness review records that remain fail-closed by default.
- Add hardening documentation templates for external operators and future agents.
- Update CLI status text to expose the Phase 17 boundary.
- Update the structure validator for Phase 17 files.
- Update roadmap, architecture, security, handoff, manifest, README, AGENTS, and gap tracker.

## Out of Scope

- Running Rust/Cargo validation.
- Running CI/CD.
- Running SAST, dependency audit, SBOM, image scanning, load tests, penetration tests, cloud deployment, live exchange validation, DEX/RPC validation, rollback drills, incident drills, or production readiness reviews.
- Building production container images or release binaries.
- Installing or starting services.
- Public network exposure.
- Live trading, signing, withdrawals, bridges, broadcasts, real exchange/RPC calls, real secrets, credentials, provider tokens, or wallet key material.
- Claiming production readiness or live-funds readiness.

## Implementation Sequence

1. Re-read governance files and confirm Phase 17 is next.
2. Create this sub-roadmap before code changes.
3. Add `arb-core::hardening` model/trait boundary.
4. Export Phase 17 types from `arb-core`.
5. Surface Phase 17 status in `arb-agent`.
6. Add hardening documentation templates that remain checklists only.
7. Update `scripts/validate_structure.py` required files.
8. Re-run available validations.
9. Update governance and handoff docs.
10. Recompute `STRUCTURE_MANIFEST.md`.
11. Prepare a commit-ready repository state.

## Acceptance Criteria

- `PHASE_17_SUBROADMAP.md` exists before Phase 17 implementation.
- `crates/arb-core/src/hardening.rs` exists and is exported.
- Hardening evidence plans are model-only and deterministic.
- Review records explicitly state that no external hardening was performed by this boundary.
- Production readiness, live-funds approval, public exposure, embedded secrets, and live validation claims fail closed.
- External hardening docs exist but do not claim execution.
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
- No real credentials in evidence references.
- No live mode defaults.
- No public bind defaults.
- No outbound hardening action from model code.
- No production readiness claim without externally generated evidence.
- No live-funds approval from ChatGPT Project Mode.

## Rollback Plan

1. Remove `crates/arb-core/src/hardening.rs`.
2. Remove hardening exports from `crates/arb-core/src/lib.rs`.
3. Revert Phase 17 status text from `crates/arb-agent/src/main.rs`.
4. Remove hardening documentation templates.
5. Remove Phase 17 required files from `scripts/validate_structure.py`.
6. Revert governance docs to Phase 16 state.
7. Re-run `python3 scripts/validate_structure.py`.

## Completion State

Completed for ChatGPT Project Mode after Phase 17 evidence-boundary patch and available validation.

Rust, Cargo, CI, local example-container build, and example image scan validation now have repeatable local/CI paths where the required tools are available. Production container, systemd, ARM, SAST review, dependency audit review, SBOM review, cloud/staging deployment, load, penetration, rollback, incident-response, live exchange/RPC, and production readiness validation remain external unless explicitly run in a capable environment.
