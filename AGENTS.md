# AGENTS.md

This file defines repository-wide coding-agent rules for ArbyClaw.

## 1. Repository scope

ArbyClaw is a Rust CLI/library workspace for local arbitrage research, simulation, policy, audit, persistence, validation, packaging, and hardening. Do not assume a frontend, REST API, SaaS tenancy layer, cloud database, or production web service unless those components actually exist in source.

Workspace members:
- `crates/arb-core`
- `crates/arb-agent`

## 2. Source-of-truth order

When sources disagree, use this order:

1. executable source code and tests;
2. `validation/behavior_contract.json` for behavior-preserving refactor invariants;
3. `CAPABILITIES.md` plus `validation/external_evidence.json` for capability/evidence claims;
4. `ARCHITECTURE.md` and `docs/ai/ARCHITECTURE_MAP.md`;
5. `docs/ai/API_CONTRACTS.md`;
6. `validation/validation_graph.json` for aggregate validation ownership;
7. `PRODUCTION_GAP_TRACKER.md` and `ROADMAP.md`;
8. handoff/tool memories as navigation aids only.

Do not use historical phase files, stale generated snapshots, mocks, or prior narrative claims as proof of current behavior.

## 3. Anti-hallucination invariant

Before adding an import, dependency, method, endpoint, service, provider, configuration key, or external capability, verify it exists in the current repository or explicitly add it as part of the change.

Never fabricate compatibility shims around a nonexistent API merely to make a prompt appear satisfied. If a requested capability is absent, say so and implement the real boundary or record the gap.

A mock, fixture, transcript, preflight, plan, dry run, or local review must be named and reported as such. It may not be described as a completed external audit, live provider validation, production deployment, or real execution.

An `EXTERNALLY_VALIDATED` or `PRODUCTION_APPROVED` capability must satisfy `scripts/validate_assurance_integrity.py`. Human approval is a separate decision and does not substitute for technical external validation.

## 4. Safety boundaries

Unless an accountable human explicitly authorizes a separately reviewed change, do not enable:

- live order placement or live DEX swaps;
- real exchange/RPC submission;
- wallet custody or production signing;
- transaction broadcasts;
- withdrawals or bridges;
- public dashboard/metrics exposure;
- outbound provider delivery;
- production service installation/deployment;
- live-funds approval.

No LLM path may directly sign transactions, access production signing material, or bypass policy/destination controls.

## 5. Required opening checks

Run what the environment supports before changing code:

```bash
python3 scripts/validate_repository_hygiene.py
python3 scripts/validate_structure.py
python3 scripts/validate_validation_graph.py
python3 scripts/validate_architecture_ratchets.py
python3 scripts/validate_behavior_contract.py
python3 scripts/validate_assurance_integrity.py
cargo fmt --check
cargo check --workspace --locked
python3 scripts/validate_test_collection.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

If a check cannot run, record it as `UNVERIFIED`; do not convert an environmental limitation into a pass or failure.

## 6. Definition of Done

A task, feature, milestone, or project is not complete because code looks correct, compiles once, mocks pass, screenshots look right, or documentation says complete.

For every applicable change:

1. Promised behavior has a stable requirement/work-item ID and an acceptance test, **because** requirements without executable anchors drift; **or else** the claim is not complete.
2. The workspace builds from a clean checkout with locked dependencies, **because** dirty local state can hide missing inputs; **or else** build evidence is invalid.
3. The production-intent artifact is created successfully when the change affects releasable behavior, **because** source compilation is not proof the artifact can be produced; **or else** release readiness is unverified.
4. Final smoke/E2E validation runs against the built artifact when applicable, **because** source-only execution can bypass packaging defects; **or else** the packaged path is unverified.
5. Required tests run in a clean/ephemeral environment where practical, **because** cached state can mask defects; **or else** state the limitation explicitly.
6. No required test is silently skipped, disabled, ignored, pending, or xfailed, **because** skipped coverage is not passing coverage; **or else** the requirement remains open.
7. Test collection is non-vacuous, **because** zero tests can return success; **or else** CI must fail.
8. Tests assert semantic results, boundaries, errors, and unsafe-path denial—not merely that functions return—**because** superficial tests permit simulated functionality; **or else** coverage is insufficient.
9. External/production capability claims require evidence from the real external system/environment, **because** local models cannot validate provider behavior; **or else** the capability remains `MODELED`, `LOCAL`, or `UNVERIFIED`.
10. No completion claim may rely on generated narrative evidence produced by the same code path being assessed without an independent assertion, **because** self-attestation is circular; **or else** the evidence is advisory only.
11. A structural refactor must preserve the applicable `BC-*` requirements in `validation/behavior_contract.json`, **because** moving code is not permission to silently change behavior; **or else** the structural change is incomplete.
12. Architecture ratchets may only stay equal or decrease, **because** a refactor that merely regrows or relocates a monolith has not reduced structural debt; **or else** `validate_architecture_ratchets.py` must fail.

## 7. Validation ownership

`validation/validation_graph.json` is the aggregate-ownership contract. Each aggregate node must have one path from the top handoff gate, and each safety-critical leaf command named in `single_owner_leaf_commands` must have exactly one aggregate owner.

A new validation layer must add at least one of:
- a new semantic assertion;
- a new failure mode;
- a new environment;
- a new artifact boundary;
- an independent side-effect tripwire.

Do not add a wrapper whose only purpose is to repeat an existing check, increase a phase number, or create the appearance of additional evidence. `scripts/validate_validation_graph.py` must remain green after any validation-graph edit.

## 8. Repository hygiene

Do not commit:
- `__pycache__`, `.pyc`, temp or backup files;
- local Obsidian/editor workspace state;
- generated whole-repo snapshots such as Repomix output;
- generated CycloneDX files from ordinary CI runs;
- mock/simulated audit evidence presented as real evidence;
- secrets or credentials.

`python3 scripts/validate_repository_hygiene.py` is the enforcement boundary.

## 9. Governance

Do not create new `PHASE_*_SUBROADMAP.md` files. Use stable IDs in `ROADMAP.md` and `PRODUCTION_GAP_TRACKER.md`. Git history preserves chronology.

Do not use numeric production-readiness percentages. Use the state vocabulary in `CAPABILITIES.md`.

Do not promote a capability to `EXTERNALLY_VALIDATED` without a passed technical record in `validation/external_evidence.json` for the exact commit/environment. Do not promote to `PRODUCTION_APPROVED` without both technical external validation and a separate accountable human approval record.

Update canonical docs only when architecture, capability state, gap closure criteria, or agent contracts materially change.

## 10. Refactoring rules

Prefer mechanical, behavior-preserving decomposition before introducing new crates or frameworks. Keep changes small enough that a failing regression can be attributed to a narrow patch.

For structural refactors:
- follow `docs/refactoring/BEHAVIOR_COMPATIBILITY_CONTRACT.md`;
- preserve every applicable `BC-*` requirement;
- do not intentionally change behavior in the same commit as code movement;
- never raise a size baseline in `validation/architecture_ratchets.json` merely to get CI green;
- when a monolith shrinks, lower its ratchet in the same or immediately following structural commit;
- keep new Rust/Python source under the configured new-file ceiling rather than moving a monolith intact.

For the current structural debt:
- make `arb-agent/src/main.rs` a thin entrypoint over focused CLI modules;
- split giant domain modules internally;
- reduce broad crate-root re-exports in favor of domain-qualified imports;
- preserve existing command names and structured output contracts unless explicitly reviewed.

## 11. Evidence language

Use exact labels:
- `PASSED` — executed successfully for the exact code/environment identified;
- `FAILED` — executed and failed;
- `BLOCKED` — execution was attempted but a prerequisite/capability prevented it;
- `UNVERIFIED` — not executed or evidence not available;
- `MODELED` / `LOCAL` / `INTEGRATED_LOCAL` / `EXTERNALLY_VALIDATED` / `PRODUCTION_APPROVED` — capability states defined in `CAPABILITIES.md`.

Never use “verified,” “validated,” “production-ready,” or “audit complete” without evidence matching the scope of the statement.
