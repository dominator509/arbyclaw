# PHASE_10_SUBROADMAP.md

## Phase

Phase 10 — Execution Planner

## Status

Implemented for ChatGPT Project Mode as a deterministic execution-planner model boundary; current workspace Rust/Cargo validation evidence exists and must be refreshed after changes.

## Governance Prerequisites

- `ARCHITECTURE.md` reread before implementation.
- `ROADMAP.md` reread before implementation.
- `PHASE_9_SUBROADMAP.md` reread as the latest completed phase before creating this file.
- `AGENTS.md` reread before implementation.
- `PRODUCTION_GAP_TRACKER.md` reread before implementation.
- `python3 scripts/validate_structure.py` passed before implementation.

## Goal

Convert validated opportunity candidates into deterministic, policy-evaluated execution-plan drafts without submitting orders, signing transactions, broadcasting transactions, withdrawing funds, bridging assets, or calling external venues.

## Scope

Phase 10 may define:

- planner configuration records
- planner request records
- deterministic execution-plan draft records
- per-leg execution intent generation for paper or observe scopes only
- policy preflight outcome records
- sequencing boundaries
- failure-mode model boundaries
- planner validation errors
- planner traits and deterministic implementation

## Explicit Non-Goals

Phase 10 must not implement:

- live trading
- live exchange order submission
- DEX transaction construction for broadcast
- wallet signing
- withdrawals
- bridges
- real RPC calls
- real CEX API calls
- real DEX/router calls
- adapter submission
- autonomous execution loops
- secret loading or custody behavior

## Implementation Plan

1. Add `arb-core::planner` with deterministic planner models and validation.
2. Generate one draft `ExecutionIntent` per opportunity leg.
3. Reject live planner scope fail-closed.
4. Evaluate every draft intent through the existing `PolicyEngine` and store redacted policy outcomes.
5. Model plan sequencing and failure boundaries without adapter submission.
6. Export planner types from `arb-core`.
7. Surface planner status from `arb-agent` without enabling execution.
8. Update structure validation requirements.
9. Update architecture, roadmap, gap tracker, README, security notes, handoff context, and manifest.

## Deliverables

- `crates/arb-core/src/planner.rs`
- `PHASE_10_SUBROADMAP.md`
- Updated `crates/arb-core/src/lib.rs`
- Updated `crates/arb-agent/src/main.rs`
- Updated `scripts/validate_structure.py`
- Updated governance and handoff files

## Validation

Executed in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Exit Criteria

Met when:

- Phase 10 sub-roadmap exists before code changes.
- Planner code is isolated to `arb-core::planner` and exports through `arb-core`.
- The planner rejects live scope.
- Generated plans are draft-only and cannot submit to adapters.
- Every generated draft intent receives a policy outcome record.
- Structure validation passes.
- Production gaps are updated for deferred Rust validation and future execution-adapter integration.

## Rollback Plan

1. Remove `crates/arb-core/src/planner.rs`.
2. Remove planner exports from `crates/arb-core/src/lib.rs`.
3. Revert planner status text in `crates/arb-agent/src/main.rs`.
4. Remove Phase 10 requirements from `scripts/validate_structure.py`.
5. Revert governance files to Phase 9 state.
6. Re-run `python3 scripts/validate_structure.py`.

## Deferred Work

- Keep Rust compilation, formatting, clippy, and unit tests current after future changes.
- Durable audit journal writes for plan creation and policy outcomes.
- Full runtime lifecycle wiring beyond the current local plan-draft checkpoint helper.
- Execution adapter handoff in Phase 11.
- Advanced partial-fill, timeout, cancellation, and hedge sequencing.
- Live mode enablement remains blocked by custody, signer, audit, state, connector, and external validation phases.
