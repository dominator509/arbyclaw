# PHASE_16_SUBROADMAP.md

## Phase

Phase 16 — Packaging and Deployment.

## Governance Status

Created before Phase 16 implementation. This file is authoritative for the Phase 16 packaging and deployment boundary patch.

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
- `scripts/validate_structure.py` passes before Phase 16 changes.
- Live trading, signing, withdrawals, bridges, broadcasts, real RPC/exchange calls, public web exposure, real observability exporters, real messaging integrations, real backtest downloads, and external fuzzing remain unavailable.

## Objective

Add deterministic packaging and deployment planning boundaries that document how the system can be packaged for future local, VPS, and ARM deployments while preserving fail-closed defaults.

## In Scope

- Add packaging/deployment model and trait boundaries.
- Add local deployment plan records.
- Add local rollback validation records with audit/state recovery while preserving no-execution semantics.
- Add package target and hardening metadata.
- Add container, systemd, ARM, and deployment documentation templates with static validation for ARM target/command/no-claim profile notes.
- Add an unsigned local/CI release-artifact packaging script that produces and verifies a copied release binary plus SHA-256 manifest and unsigned provenance record with bounded build/smoke/metadata helper commands and without signing, attestation upload, publishing, or deployment claims.
- Add a local example-container validation script for Docker build, Trivy image scan, critical-vulnerability enforcement, bounded Docker command timeouts, fail-closed unavailable-Docker reporting, and CLI smoke checks.
- Add a production-intent container recipe and validation script for local/CI build, Trivy image scan, critical-vulnerability enforcement, bounded Docker command timeouts, fail-closed unavailable-Docker reporting, inert CLI smoke checks, and hardened read-only/no-network smoke checks without deployment or readiness claims.
- Add an ARM cross-target check script and CI gate for `cargo check --workspace --target aarch64-unknown-linux-gnu --locked` with bounded `rustup`/Cargo command timeouts, without executing ARM binaries or claiming target-class readiness.
- Update CLI status text to expose the Phase 16 boundary.
- Update the structure validator for Phase 16 files.
- Update roadmap, architecture, security, handoff, manifest, README, AGENTS, and gap tracker.

## Out of Scope

- Pushing production container images to registries.
- Installing systemd units.
- Starting daemons or services.
- Public network exposure.
- Production cloud deployment.
- Production release signing.
- Publishing release artifacts.
- Live exchange/RPC calls.
- Live trading, signing, withdrawals, bridges, or broadcasts.
- Real secrets, credentials, provider tokens, or key material.
- CI/CD execution claims.
- Load, penetration, disaster-recovery, rollback, or incident-drill execution.

## Implementation Sequence

1. Re-read governance files and confirm Phase 16 is next.
2. Create this sub-roadmap before code changes.
3. Add `arb-core::packaging` model/trait boundary.
4. Export Phase 16 types from `arb-core`.
5. Surface Phase 16 status in `arb-agent`.
6. Add deployment documentation templates that remain non-executable by default.
7. Update `scripts/validate_structure.py` required files.
8. Re-run available validations.
9. Update governance and handoff docs.
10. Recompute `STRUCTURE_MANIFEST.md`.
11. Prepare a commit-ready repository state.

## Acceptance Criteria

- `PHASE_16_SUBROADMAP.md` exists before Phase 16 implementation.
- `crates/arb-core/src/packaging.rs` exists and is exported.
- Packaging plans are model-only and deterministic.
- Deployment records explicitly state that no deployment was performed.
- Rollback validation records explicitly state that no rollback, service-manager action, file mutation, external call, live execution, or production readiness approval occurred.
- Public exposure, live trading, embedded secrets, and production claims fail closed.
- Container/systemd/ARM docs exist but do not claim execution.
- Structure validator passes.
- Cargo/Rust validation is attempted only if toolchain exists and otherwise tracked as deferred.

## Validation Commands

Available in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
python3 scripts/validate_packaging_deployment_gate.py --json
```

Required externally before production claims:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Additional external Phase 16 validation required:

```bash
cargo build --release --locked
python3 scripts/validate_release_artifact.py
cargo build --target aarch64-unknown-linux-gnu --release --locked
production-intent container build and scan validation in an approved local/CI runtime
cargo check --workspace --target aarch64-unknown-linux-gnu --locked
systemd unit linting in a Linux host or container
read-only filesystem and non-root runtime validation, with local static example hardening checks and bounded optional config smoke now covered by `scripts/validate_deployment_static_hardening.py`
rollback drill validation
```

## Security Requirements

- No secrets in repository artifacts.
- No real service tokens in examples.
- No live mode defaults.
- No public bind defaults.
- No outbound deployment action from model code.
- No release or deployment readiness claim without external validation.

## Rollback Plan

1. Remove `crates/arb-core/src/packaging.rs`.
2. Remove packaging exports from `crates/arb-core/src/lib.rs`.
3. Revert Phase 16 status text from `crates/arb-agent/src/main.rs`.
4. Remove deployment documentation templates.
5. Remove Phase 16 required files from `scripts/validate_structure.py`.
6. Revert governance docs to Phase 15 state.
7. Re-run `python3 scripts/validate_structure.py`.

## Completion State

Completed for ChatGPT Project Mode after Phase 16 patch and available validation.

Rust, Cargo, unsigned release-artifact packaging/provenance with bounded build/smoke/metadata helper commands, example-container, production-intent container with hardened local smoke and fail-closed Docker timeout reporting, static deployment hardening with bounded optional config smoke, ARM cross-target check with bounded prerequisite/check commands, and CI validation now have repeatable local/CI paths where the required tools are available. Signing, attestation upload, release publishing, systemd, ARM device/runtime, deployment, rollback, service lifecycle, and broader security hardening validation remain external unless explicitly run in a capable environment.
