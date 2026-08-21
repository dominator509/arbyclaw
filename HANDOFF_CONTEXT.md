# Handoff Context

## Current state

ArbyClaw is a local-first Rust arbitrage research/simulation/validation system. Its safety boundaries are real; its production live-trading integrations are not. Do not infer live capability from local readiness models, transcripts, fixtures, or aggregate gates.

The 2026-08-20 remediation branch is `remediation/debug-drift-20260820`.

## Read first

1. `AGENTS.md`
2. `CAPABILITIES.md`
3. `ARCHITECTURE.md`
4. `docs/ai/ARCHITECTURE_MAP.md`
5. `docs/ai/API_CONTRACTS.md`
6. `PRODUCTION_GAP_TRACKER.md`
7. `ROADMAP.md`
8. source code and tests relevant to the task

Git history is authoritative for chronology. Historical numbered phase files and generated structure manifests are not current architecture inputs.

## Non-negotiable boundaries

Unless a human explicitly authorizes a separately reviewed change, do not enable or represent as implemented:

- live exchange/RPC calls;
- live order placement or DEX swaps;
- wallet custody/signing;
- transaction broadcast;
- withdrawals or bridges;
- public dashboard/metrics exposure;
- real outbound messaging;
- production service installation/deployment;
- production-readiness or live-funds approval.

Never place credentials or secret material in repository files, prompts, logs, audit records, generated evidence, screenshots, or chat artifacts.

## Validation opening sequence

```bash
python3 scripts/validate_repository_hygiene.py
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace --locked
python3 scripts/validate_test_collection.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/validate_release_artifact.py
python3 scripts/validate_agentic_handoff_candidate_gate.py --json
```

If the environment cannot execute one of these, record it as `UNVERIFIED` and continue with the checks that are available. Never substitute a narrative claim for execution evidence.

## Remediation context

The 2026-08-20 debug/drift review confirmed and began remediating:

- simulated/mock security-audit evidence presented with misleading completion language;
- a mock SBOM containing a dependency not present in `arb-core`;
- stale generated SBOM/Repomix snapshots;
- tracked temp/backup/Python cache/Obsidian workspace artifacts;
- numeric production-readiness scoring that overstated real capability;
- 130 numbered phase files duplicating Git history and project status;
- recursive validation where the handoff gate reran suites already owned by hardening-core;
- empty AI architecture/API context documents;
- an unpinned Rust toolchain despite strict linting;
- absence of a per-package zero-test collection guard.

The first cleanup commit on the remediation branch is `7b9843302bb4e403f668ebeb20c64229c8a64fd0`.

## Remaining structural debt

The largest source files remain oversized. `crates/arb-agent/src/main.rs` should be decomposed mechanically after clean branch CI is established. Large core modules and broad crate-root re-exports should be reduced in subsequent work items without changing semantics.

## Evidence checkpoint

During the 2026-08-20 review, the most recent independently retrievable successful GitHub Actions run belonged to commit `0b98a9a31d3701704d950779ad989daefcf1193b` on 2026-05-26. Current remediation-branch validation must be established by fresh CI and must not inherit that older commit's pass status.

## Handoff rule

A future agent may modify the repository only if it preserves fail-closed policy, redaction, audit/state durability, destination/signer boundaries, and explicit production blockers. Completion claims require execution evidence for the exact code being claimed complete.
