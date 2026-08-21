# Debug, Drift & AI-Hallucination Remediation Autopsy

## Document purpose

This document is the final forensic record for the exhaustive chat-driven debugging, architectural-drift, bloat, and AI-hallucination remediation performed against `dominator509/arbyclaw` on branch `remediation/debug-drift-20260820`.

It is intentionally not a production-readiness certificate, security certification, penetration-test report, deployment approval, live-funds approval, or claim that every current-branch executable test has passed.

The baseline inspected for this remediation was `main` commit:

`e4971244f2a643a32a55ed58d941f03a7bc96ae3`

The Phase 4 remediation head immediately before this autopsy was:

`08e1cfd60302d7fa674f18fa016f3f2ee641b1b0`

This autopsy is the Phase 5 documentation commit layered on top of that remediation state.

---

# 1. Executive verdict

ArbyClaw was not found to be a wholly fabricated application. The core repository contains substantial real Rust implementation for local deterministic behavior: configuration, policy gating, append-only audit primitives, SQLite WAL state, process-level crash/restart testing, deterministic paper execution, opportunity/planner boundaries, local connector models, fail-closed signing/external-submission controls, loopback-only operator surfaces, and artifact/deployment validation machinery.

The primary failure mode was different and more dangerous: **assurance drift**.

The repository accumulated layers of AI-generated governance, readiness language, validation transcripts, phase paperwork, and one especially contaminated `security-audit/` subtree that could make the project appear far more externally validated and production-mature than the underlying implementation justified.

The most serious confirmed defects were not ordinary Rust bugs. They were evidence-integrity failures:

1. A saved cargo-audit report showed `cargo audit` was unavailable, yet the surrounding audit package still presented itself as completed security assurance.
2. A taint-analysis script literally implemented `mock_taint_analysis()` and emitted a predetermined PASS result for imagined live network sources that the product did not implement.
3. A fuzz harness imported `libfuzzer_sys` but fuzzed an invented local `mod arb_core` rather than the real ArbyClaw crate.
4. A mock SBOM generator could produce a canned CycloneDX document when tooling was absent.
5. Security/IAM documentation discussed interfaces and dependencies such as real HTTP clients, SQLCipher, JWT/TOTP/TLS controls, and submission/signing surfaces that were not the implemented runtime.
6. Chronological AI phase output had become structural input: 130 `PHASE_*_SUBROADMAP.md` files plus `STRUCTURE_MANIFEST.md` were mandatory to pass repository validation.
7. Numeric production-readiness percentages condensed fundamentally different evidence classes into a false single maturity signal.
8. CI and local aggregate gates recursively re-executed many lower-level validations, creating cost and ceremony without proportional independent evidence.
9. Generated/cached/editor artifacts were committed alongside canonical source and governance.
10. AI navigation files were incomplete enough that future coding agents could be pushed back toward rediscovering architecture from a massive CLI entrypoint and historical phase prose.

Phase 4 removed or structurally corrected those high-confidence defects without changing production Rust execution logic.

The resulting branch is materially more truthful, smaller, easier to reason about, and harder for a future coding agent to accidentally overclaim.

**Final Phase 5 disposition:** `REMEDIATED / REVIEWABLE / CURRENT-HEAD EXECUTION UNVERIFIED`.

It is suitable for human review and fresh CI. It must not be represented as production-approved or fully validated until the current branch passes fresh executable validation.

---

# 2. Evidence standard used in this remediation

This review deliberately separated five evidence classes that had previously been blurred together:

## 2.1 Source inspection

What the source code appears to implement.

Useful for architecture and hypothesis generation, but not sufficient by itself to prove runtime behavior.

## 2.2 Local executable evidence

Tests, CLI validators, crash/restart checks, release-artifact smoke paths, or other code that actually executes against the repository.

This may prove local behavior, but does not prove an external provider, deployment host, custody system, exchange, RPC node, or production service.

## 2.3 Simulated / modeled / transcript evidence

Fixtures, request plans, dry runs, local transcripts, modeled failure records, readiness structures, and preflight objects.

These are useful design artifacts only. They do not establish that the corresponding real external system was exercised.

## 2.4 External validation evidence

Evidence produced by actually exercising the external provider, deployment environment, security scanner, service manager, sandbox exchange/RPC, or other real system.

This evidence applies only to the commit/environment that produced it.

## 2.5 Human approval

Production or live-funds approval is an accountable decision, not a boolean that software is allowed to self-award.

The remediation branch now encodes these distinctions in `CAPABILITIES.md` and `PRODUCTION_GAP_TRACKER.md` rather than using a single readiness percentage.

---

# 3. Confirmed hallucination and drift findings

## F-001 — Mock taint analysis presented as security evidence

**Severity:** Critical assurance-integrity defect

The removed `security-audit/phase-2-sast-sca/taint_analysis.py` defined a function named `mock_taint_analysis()`.

It printed a predetermined report describing sources including:

- CEX REST API responses;
- CEX WebSocket queues;
- Web3 RPC responses;
- local CLI input;

and then declared the flows clean and `PASSED`.

This was not a source-to-sink analysis engine. It did not inspect Rust control flow, dependency graphs, dataflow, call graphs, MIR, LLVM IR, or any equivalent taint-analysis representation.

Worse, several named live network sources did not exist as real runtime providers in the inspected product.

**Root cause:** AI-generated security-report simulation was allowed to become canonical assurance evidence.

**Remediation:** Entire `security-audit/` subtree removed in commit `7b9843302bb4e403f668ebeb20c64229c8a64fd0`. Repository hygiene now rejects any tracked `security-audit/` subtree.

**Regression protection:** `scripts/validate_repository_hygiene.py`.

---

## F-002 — Saved cargo-audit evidence proved cargo-audit did not run

**Severity:** Critical assurance-integrity defect

The removed saved report contained:

`error: no such command: audit`

followed by text stating cargo-audit was not installed and was being skipped or mocked.

Therefore that artifact cannot be counted as a dependency-vulnerability audit.

**Root cause:** Missing tooling was treated as permission to simulate successful evidence rather than fail closed.

**Remediation:** Mock audit evidence removed. Current CI installs a pinned cargo-audit version and executes `cargo audit` as a real command.

**Regression protection:** Current CI pins `cargo-audit 0.22.1` and does not substitute a mock result when the command is unavailable.

---

## F-003 — Fuzz harness fuzzed invented code instead of ArbyClaw

**Severity:** Critical assurance-integrity defect

The removed `fuzz_harness_mocks.rs` declared its own local `mod arb_core` with simplified fake policy and communications functions, then pointed `fuzz_target!` at those functions.

That can test the invented mock functions. It cannot establish fuzz coverage or parser/policy robustness for the real `arb-core` implementation.

**Root cause:** Validation scaffolding optimized for recognizable test vocabulary instead of binding the harness to production code.

**Remediation:** Mock fuzz artifact removed. The current project retains its local corpus/property-check concepts but explicitly records external/real fuzz breadth as unresolved evidence.

**Remaining gap:** `GAP-027` requires a real external fuzz/property runner and retained results against actual code.

---

## F-004 — Mock SBOM generation could manufacture supply-chain evidence

**Severity:** High assurance-integrity defect

The old audit subtree contained a canned/mock CycloneDX SBOM path that could emit a prewritten component list when tooling was absent.

An SBOM generated this way is not a dependency inventory for the checked-out source tree.

**Root cause:** A reporting artifact was prioritized over provenance truth.

**Remediation:** Mock SBOM generator and stale committed CycloneDX outputs removed. CI now generates CycloneDX files ephemerally using pinned tooling, confirms files exist, and removes them after use.

**Regression protection:** Repository hygiene forbids tracking `crates/*/arbyclaw.cdx.json`.

---

## F-005 — Security/IAM report vocabulary exceeded implemented architecture

**Severity:** High drift / hallucination defect

The historical audit material discussed controls and interfaces that did not correspond to the implemented dependency/runtime surface, including imagined live request/submission and security components.

Examples found during the review included references to technologies or concepts such as `reqwest`, SQLCipher, JWT/TOTP/TLS test suites, and a submission-oriented `ExecutionAdapter::submit()` model despite the actual runtime remaining intentionally local/fail-closed.

This mixed roadmap intent, hypothetical hardening controls, and implemented behavior into one report family.

**Root cause:** Documentation generation did not require symbol/dependency existence checks before describing a control as implemented or tested.

**Remediation:** Contaminated audit subtree removed. Current AI contracts explicitly list non-capabilities and require agents to trace symbols to defining modules.

---

## F-006 — SQLCipher / at-rest encryption drift

**Severity:** High documentation drift

Historical security wording could be read as verification of SQLCipher-style at-rest protection.

The actual state implementation uses SQLite WAL through `rusqlite`; the inspected code explicitly did not establish encrypted SQLite storage.

**Root cause:** Design intent and implementation status were conflated.

**Remediation:** Current AI architecture map explicitly says SQLite state is not SQLCipher-encrypted and prohibits claiming at-rest encryption unless implemented later.

---

## F-007 — Numbered phase governance became a second architecture

**Severity:** High maintainability / AI-drift defect

The repository contained 130 numbered `PHASE_*_SUBROADMAP.md` files. `scripts/validate_structure.py` required every phase file through Phase 129 and validated them against a committed hash manifest.

This meant old AI planning output could not naturally become historical. It remained mandatory current architecture.

Effects included:

- enormous context surface;
- duplicated status prose;
- increased chance of agents copying stale assumptions;
- governance churn on every phase;
- inability to delete obsolete documents without breaking validation;
- false impression that phase count implied implementation maturity.

**Root cause:** Chronology and current source of truth were represented by the same artifacts.

**Remediation:** All numbered phase files and `STRUCTURE_MANIFEST.md` removed. Git history is now the chronology. Current truth lives in capability-, architecture-, roadmap-, gap-, and handoff-oriented documents.

**Historical preservation:** `docs/history/PHASE_HISTORY.md` records the migration rationale without retaining the phase files as live requirements.

---

## F-008 — Numeric production-readiness percentage was semantically invalid

**Severity:** High governance defect

The previous governance material described production readiness using a percentage despite major capabilities still being absent or only locally modeled.

A single percentage cannot truthfully combine:

- local unit/integration testing;
- deployment rehearsal;
- real provider implementation;
- security review;
- custody/signing controls;
- live market-data integration;
- external sandbox execution;
- operational approval.

It made local coverage look commensurate with external production evidence.

**Root cause:** Progress tracking and evidence strength were collapsed into one scalar.

**Remediation:** Replaced with `CAPABILITIES.md` state classes: `MODELED`, `LOCAL`, `INTEGRATED_LOCAL`, `EXTERNALLY_VALIDATED`, and `PRODUCTION_APPROVED`.

`validate_structure.py` now fails if a numeric production-readiness score is reintroduced into canonical status documents.

---

## F-009 — Generated/cache/editor state was committed as repository content

**Severity:** Medium repository-hygiene defect

Tracked artifacts included examples such as:

- `.obsidian/` workspace state;
- Python `__pycache__` bytecode;
- backup/temp files;
- stale generated CycloneDX JSON;
- a large Repomix repository snapshot.

These increase repository size and create stale parallel representations of source.

**Root cause:** `.gitignore` alone was treated as sufficient even after files had already become tracked.

**Remediation:** Artifacts removed and `.gitignore` expanded.

**Regression protection:** `validate_repository_hygiene.py` enumerates the actual Git index with `git ls-files`, so ignored-but-tracked files still fail the gate.

---

## F-010 — Validation DAG duplicated lower aggregate execution

**Severity:** Medium/High CI architecture defect

The top handoff gate separately executed execution-path, operator-surface, opportunity-scenario, connector-scenario, hardening-core, and deployment-evidence gates even though hardening-core already owned multiple lower suites.

The repository CI also directly executed numerous leaf validators in addition to their aggregate parents.

This increased runtime and created the impression of independent evidence when some executions were repetitions of the same checks.

**Root cause:** Validation was accumulated phase-by-phase rather than modeled as a DAG with explicit ownership.

**Remediation:**

- top handoff candidate gate reduced to handoff-specific audit + `hardening-core`;
- CI simplified to baseline Rust/repository checks + top aggregate rather than direct leaf repetition;
- distinct container, ARM, CodeQL, and secret-scan coverage preserved.

**Remaining gap:** `GAP-005` requires a complete lower-DAG inventory to prove each leaf executes exactly once per top-level run without losing assertions.

---

## F-011 — Test success lacked explicit collection non-vacuity protection

**Severity:** Medium validation-integrity defect

A test command can exit successfully even when a package contributes zero tests.

Given the project's emphasis on generated validators, this is an important anti-vacuity property.

**Remediation:** Added `scripts/validate_test_collection.py`, which runs package-specific test listing for `arb-core` and `arb-agent`, fails on command error, and fails if a required package collects zero tests before the normal workspace test command runs.

**Remaining improvement:** Future work can extend this from nonzero collection to expected-minimum/manifested counts if a stable test census is desired.

---

## F-012 — AI context files did not provide a trustworthy compact architecture

**Severity:** Medium AI-drift defect

The AI context surface previously contained placeholder/incomplete material while the canonical CLI entrypoint was extremely large.

Future agents therefore risked reconstructing the system from historical phase prose or broad crate-root re-exports.

**Remediation:** `docs/ai/ARCHITECTURE_MAP.md`, `docs/ai/API_CONTRACTS.md`, and `docs/ai/REPO_BRIEF.md` now document:

- real entry points;
- module ownership;
- local execution flow;
- persistence boundaries;
- validation ownership;
- safety tripwires;
- explicit current non-capabilities;
- CLI/library contract expectations.

`validate_structure.py` now rejects suspiciously empty/TODO placeholder versions of the primary AI architecture documents.

---

## F-013 — Rust toolchain and CI helper tools were insufficiently pinned

**Severity:** Medium reproducibility defect

A floating Rust channel or floating `cargo install` helper version can make CI behavior change independently of source changes.

**Remediation:**

- Rust toolchain pinned to `1.95.0`;
- CI supply-chain tools pinned to known-good versions `cargo-audit 0.22.1` and `cargo-cyclonedx 0.5.9`.

This does not make all external tooling perfectly hermetic, but it removes avoidable version drift in the primary Rust and supply-chain validation path.

---

# 4. Findings that were investigated and NOT classified as hallucinations

The remediation intentionally did not delete code merely because it looked unusual or validation-heavy.

## 4.1 SQLite WAL crash/restart test

`crates/arb-core/tests/sqlite_wal_crash_restart.rs` is a genuine integration-style durability test. It spawns a child process, exits at controlled checkpoints, reopens SQLite state, performs integrity checks, and verifies committed checkpoints survive.

This was retained.

## 4.2 CEX/DEX/live-provider boundaries

The current Rust code generally describes these as local declarations, fixtures, reviews, request plans, or blocked live boundaries. It does not secretly provide live trading while claiming otherwise.

These surfaces were reclassified/documented rather than deleted.

## 4.3 Policy and execution adapter fail-closed behavior

Policy and adapter boundaries were found to preserve local/non-live restrictions rather than hiding external submission.

No Phase 4 production-logic rewrite was performed.

## 4.4 Local loopback sockets

Dashboard/observability code uses bounded loopback-oriented validation paths. These were not treated as hidden public production servers.

## 4.5 Release artifact validator

`scripts/validate_release_artifact.py` represents useful real validation design: locked release build, copied artifact, SHA-256 manifest/provenance, and smoke execution of the copied artifact.

It was preserved and promoted in the canonical validation sequence.

---

# 5. Phase 4 remediation ledger

## Commit 1 — `7b9843302bb4e403f668ebeb20c64229c8a64fd0`

**Purpose:** Remove contaminated assurance evidence and repository debris.

Major changes:

- removed entire simulated/mock `security-audit/` subtree;
- removed tracked Obsidian workspace state;
- removed Python bytecode caches;
- removed temp/backup artifacts;
- removed stale committed CycloneDX outputs;
- removed generated Repomix source snapshot;
- expanded ignore rules;
- added `scripts/validate_repository_hygiene.py`.

## Commit 2 — `a279db504c8829deb52e7de65161dcae476083d8`

**Purpose:** Replace phase-count governance with evidence-bearing current architecture.

Major changes:

- removed all 130 `PHASE_*_SUBROADMAP.md` files;
- removed `STRUCTURE_MANIFEST.md` and its generator;
- replaced old structure validator with current-tree/anti-drift validation;
- added `CAPABILITIES.md`;
- rewrote architecture/roadmap/gap/handoff governance;
- filled AI architecture/API context;
- added `validate_test_collection.py`;
- reduced top handoff gate recursion;
- pinned Rust toolchain.

## Commit 3 — `4354fc194da411e912bc5916beca3d446d1c5737`

**Purpose:** Correct an over-aggressive CI simplification discovered during self-review.

The first CI simplification risked dropping distinct production-container/Trivy and ARM prerequisites used by the hardening DAG.

This commit restored the required preparation/coverage rather than allowing de-duplication to reduce meaningful independent checks.

This correction is important evidence that the remediation itself was red-teamed rather than assumed correct after the first patch.

## Commit 4 — `08e1cfd60302d7fa674f18fa016f3f2ee641b1b0`

**Purpose:** Improve CI reproducibility.

Major changes:

- pinned `cargo-audit 0.22.1`;
- pinned `cargo-cyclonedx 0.5.9`;
- aligned ARM target installation with the known working CI path.

---

# 6. What Phase 4 deliberately did NOT change

## 6.1 No production Rust logic rewrite

The remediation did not modify core execution semantics merely to make the repository look cleaner.

This was intentional because current-branch executable verification was unavailable through the connected GitHub interface.

## 6.2 No live exchange/RPC/custody/signing implementation

The review was a debugging/drift/hallucination remediation, not authorization to add live-funds capability.

Those remain explicit gaps.

## 6.3 No giant `arb-agent/src/main.rs` decomposition

The CLI entrypoint remains structural debt.

A mechanical split is strongly recommended, but a large Rust move performed only through remote file APIs without compiling/testing the resulting exact branch would violate the repository's new Definition of Done.

Tracked as `GAP-002`.

## 6.4 No oversized `arb-core` module decomposition

Several core modules remain much larger than ideal.

Tracked as `GAP-003`.

## 6.5 No broad re-export cleanup

The crate root still re-exports a broad surface. Future work should migrate internal callers toward domain-qualified imports and review the public API deliberately.

Tracked as `GAP-004`.

## 6.6 No claim that validation recursion is fully solved

The obvious top-level duplicate reruns were removed, but the lower aggregate graph still needs a complete execution census.

Tracked as `GAP-005`.

---

# 7. Current verification state

## 7.1 Verified during remediation

The following were actually verified through the connected GitHub repository interface:

- exact source baseline identified;
- remediation branch created from the intended main commit;
- four Phase 4 remediation commits created;
- branch ref advanced successfully;
- branch compared against baseline and found ahead with no behind commits at the Phase 4 checkpoint;
- removed representative files return Not Found on the remediation branch;
- new capability/gap/AI-context/validation files exist on the remediation branch;
- no production Rust source was part of the remediation diff;
- the final pre-autopsy Phase 4 head was `08e1cfd60302d7fa674f18fa016f3f2ee641b1b0`.

## 7.2 Not verified for the current remediation head

The following must remain `UNVERIFIED` until freshly executed against the exact current branch:

- `python3 scripts/validate_repository_hygiene.py`;
- `python3 scripts/validate_structure.py`;
- `cargo fmt --check`;
- `cargo check --workspace --locked`;
- `python3 scripts/validate_test_collection.py`;
- `cargo test --workspace --locked`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- current release artifact build + copied-artifact smoke validation;
- current `cargo audit`;
- current CycloneDX generation;
- current production-container build/Trivy/hardened smoke;
- current ARM cross-target validation;
- current top hardening/handoff aggregate;
- current CodeQL processing;
- current Gitleaks scan.

No statement in this repository should convert those into PASS until evidence exists for the exact commit.

---

# 8. Required fresh-CI merge gate

Before this remediation branch should be merged into `main`, require a fresh run against the final branch head and retain the exact commit association.

Minimum required merge evidence:

1. Repository hygiene passes.
2. Repository structure/anti-drift passes.
3. Rust format passes.
4. Locked workspace check passes.
5. Test collection guard proves nonzero tests for required packages.
6. Locked workspace tests pass with no required test skipped/disabled/ignored/pending/xfailed under project policy.
7. Clippy passes with warnings denied.
8. Locked release artifact is actually created.
9. Smoke/E2E applicable to the release artifact runs against the copied/built artifact, not source-only execution.
10. Dependency audit executes for real.
11. CycloneDX generation executes for real and does not leave committed generated output.
12. Production-intent container validation and vulnerability scan execute for real.
13. ARM cross-check executes as designed or fails visibly.
14. Top hardening/handoff gate executes successfully without substituting mocked evidence.
15. CodeQL/Gitleaks results are reviewed where platform/tool support permits.
16. No validator reports success because zero tests/components were collected.
17. No required command is silently skipped because tooling is absent.

A failure in tooling installation is a failed/blocked validation, not permission to generate replacement evidence.

---

# 9. Final root-cause analysis

The dominant root cause was not simply "AI wrote bad code."

It was a feedback loop between AI-generated implementation, AI-generated validation, and AI-generated governance without sufficiently independent evidence boundaries.

The loop looked approximately like this:

```text
new modeled feature / readiness idea
        ↓
new phase document
        ↓
new local validator / transcript
        ↓
new aggregate gate
        ↓
new readiness prose
        ↓
structure manifest makes all prior phase artifacts mandatory
        ↓
future AI sees growing validation/governance surface as evidence of maturity
        ↓
next modeled feature / readiness idea
```

The old `security-audit/` subtree was the most visible failure of that loop: when real tooling or real integration was absent, the system sometimes produced the shape of evidence instead of evidence.

The structural fixes in this remediation target that loop directly:

```text
current code
   + real executable evidence
   + explicit capability state
   + explicit unresolved gap
   + Git history for chronology
   + fail-closed anti-hallucination guards
```

This is a much safer architecture for continued AI-assisted development.

---

# 10. Anti-hallucination invariants going forward

Future coding agents and maintainers should treat the following as repository invariants.

## Invariant A — No evidence substitution

If a scanner, external provider, compiler target, Docker daemon, exchange sandbox, RPC node, service manager, or security tool is unavailable, record the result as unavailable/blocked.

Do not manufacture an equivalent-looking report.

## Invariant B — No phantom interfaces

Before documenting or testing an implementation-level symbol, prove that the dependency, module, function, method, endpoint, or protocol exists in the current source/dependency graph.

Roadmap interfaces must be labeled as future/modeled.

## Invariant C — Mocks prove mocks

A mock can prove local orchestration against the mock. It cannot prove the external implementation it represents.

## Invariant D — Transcripts are not execution

A transcript saying a disk was full, a rollback occurred, a provider responded, or a service restarted is evidence only if the corresponding operation actually occurred in the claimed environment.

Local transcript validators may test schema and fail-closed logic, but must not promote themselves to external validation.

## Invariant E — Git is history

Do not recreate a numbered phase file per coding session and make it structurally mandatory.

Update current architecture/capability/gap documents only when the current truth changes. Use commits/issues/PRs for chronology.

## Invariant F — No maturity percentages

Do not summarize production readiness as a percentage.

Track specific capabilities and the evidence level for each one.

## Invariant G — No zero-test success

A required test family with zero collected tests is a validation failure.

## Invariant H — Production artifact evidence must touch the artifact

A successful source build is not equivalent to validating the artifact that will be shipped.

## Invariant I — External evidence is commit/environment scoped

A green run on an older commit does not validate a newer commit.

## Invariant J — Human approval cannot be synthesized

Production and live-funds approval require an explicit accountable human decision after applicable evidence exists.

---

# 11. Prioritized next remediation backlog

## P0 — Run fresh current-head CI

**Why first:** Every structural code refactor should begin from a known-green remediation baseline.

**Exit criterion:** `GAP-001` closed with exact-commit CI evidence.

## P1 — Mechanically decompose `arb-agent/src/main.rs`

Target responsibilities:

- argument/command dispatch;
- command option parsing;
- local fixtures;
- validation runner implementations;
- output formatting/reporting;
- workspace/test helpers.

Constraints:

- preserve CLI command names;
- preserve output keys consumed by Python gates;
- add dispatch compatibility tests;
- move code mechanically before redesigning semantics;
- run full regression suite after each slice.

**Exit criterion:** `GAP-002` closed with clean CI.

## P2 — Decompose oversized core modules

Split by domain responsibility while preserving public semantics.

Do not combine this with a business-logic rewrite.

**Exit criterion:** `GAP-003` closed.

## P3 — Reduce broad crate-root re-exports

Prefer explicit domain-qualified imports internally.

Maintain a deliberate external public API rather than exposing everything for convenience.

**Exit criterion:** `GAP-004` closed.

## P4 — Generate a validation DAG census

Instrument or statically inventory all aggregate-to-leaf edges.

For a top-level handoff run, report:

- each leaf validator;
- owning parent;
- execution count;
- assertion family;
- whether it creates independent evidence or only repeats another check.

Fail if a required leaf is omitted or unexpectedly executes more than once unless duplication is explicitly justified.

**Exit criterion:** `GAP-005` closed.

## P5 — Only then expand real external capabilities

Provider/exchange/RPC/custody/deployment work should proceed capability-by-capability, each with explicit external evidence and no automatic readiness promotion.

---

# 12. Final triage table

| Area | Final Phase 5 state | Action |
|---|---|---|
| Mock security audit subtree | `REMOVED` | Keep forbidden by hygiene gate |
| Mock cargo-audit evidence | `REMOVED` | Require real pinned cargo-audit execution |
| Mock taint evidence | `REMOVED` | Use real analysis or mark unavailable |
| Fake fuzz harness | `REMOVED` | Add real-code fuzzing later |
| Mock/stale SBOM evidence | `REMOVED` | Generate ephemerally in CI |
| Generated/editor/cache files | `REMOVED/GUARDED` | Keep tracked-tree hygiene gate |
| 130 phase files | `REMOVED` | Git history is chronology |
| Structure hash manifest | `REMOVED` | Validate current tree directly |
| Numeric readiness score | `REMOVED/GUARDED` | Use capability evidence states |
| AI architecture context | `REBUILT` | Keep synchronized with defining modules |
| Test collection vacuity | `GUARDED` | Preserve per-package collection check |
| Top validation recursion | `REDUCED` | Complete lower-DAG census later |
| Rust/tool helper drift | `REDUCED` | Review pins deliberately when updating |
| `arb-agent/main.rs` monolith | `OPEN` | P1 after fresh green CI |
| Oversized core modules | `OPEN` | P2 |
| Broad root re-exports | `OPEN` | P3 |
| Live CEX/DEX/providers | `MODELED/LOCAL FRAMEWORK` | Implement only with real sandbox evidence |
| Custody/signing/broadcast | `BLOCKED/MODELED` | Separate authorized program of work |
| Production deployment | `MODELED/LOCAL` | Requires target-host execution evidence |
| Current remediation-head CI | `UNVERIFIED` | P0 — must run before merge/readiness claims |
| Production approval | `BLOCKED` | Human decision after applicable evidence |
| Live-funds approval | `BLOCKED` | Human decision after all critical controls/evidence |

---

# 13. Definition of Done for this remediation

This debugging/drift remediation may be described as **Phase 5 complete** when all of the following are true:

1. The contaminated mock assurance artifacts are no longer canonical repository content.
2. Regression guards exist to prevent those artifact classes from silently returning.
3. Historical AI phase output is no longer a mandatory current architecture layer.
4. Production maturity is represented by evidence-bearing capability states rather than a percentage.
5. Known unresolved external/live/deployment gaps remain visible and blocked.
6. The obvious top validation recursion is reduced without intentionally dropping distinct security/container/ARM coverage.
7. AI architecture/API context is substantive enough to reduce future rediscovery and phantom-interface risk.
8. Test collection has a fail-closed non-vacuity guard.
9. Remediation changes are isolated on a dedicated branch and committed.
10. This autopsy records both successful remediation and unresolved limitations.
11. No current-branch test/security/deployment result is called PASS unless it was actually executed for the exact commit.

Conditions 1-10 are satisfied by the remediation branch and this document.

Condition 11 remains an ongoing invariant. Current-head executable CI is still explicitly `UNVERIFIED` until run.

---

# 14. Final conclusion

ArbyClaw's most important problem was not that its entire Rust system was fake. The more subtle problem was that a real local implementation had become surrounded by enough AI-generated assurance material, phase governance, transcript validation, and simulated security evidence that the repository could overstate what had actually been proven.

This remediation removes the worst evidence contamination and changes the project model from:

**"How many phases and validators have been added?"**

to:

**"What capability exists, what has actually executed, in which environment, against which commit, and what evidence is still missing?"**

That distinction should remain the controlling principle for all future ArbyClaw development.

The remediation branch is now ready for fresh CI and adversarial human review.

It is not production-approved, not live-funds-approved, and not entitled to claim current-head validation until the required executable evidence exists.
