# PRODUCTION_GAP_TRACKER.md

## Executive Status Summary

The project is in Phase 18 agentic-handoff-package-complete status for ChatGPT Project Mode. Architecture, roadmap, agent governance, phase sub-roadmaps, gap tracking, a minimal Rust workspace scaffold, typed non-secret config, reference-only secret abstractions, redacted secret material, initial live-mode validation, an isolated deny-by-default policy engine, append-only audit journal primitives, a state-store trait boundary, normalized market-data models, freshness classification, fee models, provider trait boundaries, deterministic paper connectors, CEX framework types/traits, DEX/Web3 framework types/traits, deterministic opportunity-engine types/traits, draft-only execution-planner types/traits, execution-adapter boundary records/traits, communications/CLI command and notification boundary records/traits, embedded-dashboard local render records/traits, observability/runbook local health/log/metric/runbook records/traits, deterministic validation plan/fixture/fuzz/backtest records/traits, deterministic packaging/deployment plan records/traits plus example deployment templates, deterministic external hardening evidence/review records/traits plus hardening checklists, and deterministic agentic handoff package records/traits plus future-agent prompts and checklists exist. The 2026-05-25 ArbyClaw local validation sequence completed for repository structure, formatting, workspace compilation, tests, and clippy; this is local validation evidence only. GitHub Actions CI also runs structure, Rust validation, locked release build, dependency audit, SBOM generation, local-SARIF SAST, example image scan, secret-pattern scan, and hardening evidence indexing for pushed commits. The project is not ready for live funds, live exchange credentials, wallet keys, production deployment, transaction signing, broadcasts, live adapter submission, real outbound communications, real dashboard hosting, real observability/exporter/alert runtime, real fuzzing engine execution, real backtest execution, production container/systemd/ARM validation, broader external hardening execution, external agent execution validation, rollback drills, cloud deployment, production release, live-funds approval, or autonomous execution.

## Latest Local Validation Attempt

2026-05-27 ArbyClaw local validation attempt:

- `python3 scripts/validate_structure.py` passed.
- `cargo fmt --check` passed.
- `cargo check --workspace` passed.
- `cargo test --workspace` passed with 76 tests across 3 suites.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- This validates local structure, formatting, compilation, tests, and linting only. It does not validate production deployment, live funds, live exchange/RPC integrations, signing, broadcasts, containers, systemd, ARM, CI, penetration testing, load testing, rollback drills, incident drills, external hardening, or production readiness.

## Latest CI Validation Attempt

2026-05-26 ArbyClaw GitHub Actions CI validation snapshot:

- Repository: `dominator509/arbyclaw`
- Branch: `main`
- Latest validated commit: `0b98a9a31d3701704d950779ad989daefcf1193b`
- Workflow run: `https://github.com/dominator509/arbyclaw/actions/runs/26443625602`
- Result: passed.
- Completed CI steps: checkout via `actions/checkout@v6`, Rust stable toolchain install with rustfmt and clippy, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release --locked`, hardening tool installation, `cargo audit`, CycloneDX SBOM generation with non-empty file checks, CodeQL Rust SAST analysis with local SARIF generation, non-empty SARIF verification, short-retention SARIF artifact upload, example container image build, Trivy image scan evidence artifact upload, fixable critical image-vulnerability enforcement, Gitleaks redacted secret-pattern scan artifact upload, lightweight hardening evidence index artifact upload, GitHub Step Summary hardening evidence pointer generation, and `python3 scripts/validate_structure.py`.
- Artifact references from that run: `hardening-evidence-index` `7213484905`, `codeql-sarif-evidence` `7213379997`, `trivy-image-scan-evidence` `7213343486`, `gitleaks-secret-scan-evidence` `7213308494`, and Docker Buildx build record `7213344332`.
- Node.js 24 migration status: the workflow uses `actions/checkout@v6`.
- This validates the pushed repository structure, formatting, compilation, tests, linting, locked release build, dependency audit, SBOM generation gate, local-SARIF CodeQL SAST gate, short-retention SAST artifact retention, example container image build, example Trivy image-scan gate, and Gitleaks secret-pattern scan gate in GitHub Actions only. It does not validate production deployment, live funds, live exchange/RPC integrations, signing, broadcasts, production containers, systemd, ARM, penetration testing, load testing, rollback drills, incident drills, SBOM review, GitHub code scanning upload processing, broader external hardening, or production readiness.

## Latest Gap Tracker Audit

2026-05-26 ArbyClaw roadmap-anchored production gap tracker audit:

- Local validation rerun passed: `python3 scripts/validate_structure.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` with 76 tests across 3 suites, and `cargo clippy --workspace --all-targets -- -D warnings`.
- Latest referenced GitHub Actions validation passed on `main` in run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602` for commit `0b98a9a31d3701704d950779ad989daefcf1193b`.
- The Rust/Cargo validation class of gaps is now locally and CI-covered for the current workspace state: GAP-0001, GAP-0029, GAP-0031, GAP-0033, GAP-0036, GAP-0040, GAP-0043, GAP-0046, GAP-0050, GAP-0053, GAP-0055, GAP-0057, GAP-0059, GAP-0061, GAP-0063, GAP-0065, GAP-0067, GAP-0069, and GAP-0071.
- These validation updates do not prove production readiness, deployment readiness, live-funds readiness, public exposure readiness, code-scanning upload processing, SBOM review, penetration testing, load testing, rollback drills, incident drills, custody readiness, or live exchange/RPC readiness.
- Release evidence reviewer follow-up metadata is now considered complete enough for the current Phase 17 / Phase 18 roadmap boundary; additional reviewer micro-fields should not be added unless a concrete operator workflow identifies missing non-secret routing metadata.
- The safest fillable next gaps are roadmap-aligned tracker corrections and external validation closure records: stale CI/local validation statements, SBOM review evidence, external artifact review, code-scanning settings evidence, and operator review records.
- The non-fillable-in-repo gaps remain open where they require credentials, wallet custody, signer design, live exchange/RPC validation, production infrastructure, public exposure review, penetration testing, load testing, rollback drills, incident drills, legal/compliance review, or accountable human approval.

## Latest External Hardening Evidence Attempt

2026-05-26 ArbyClaw release-build, dependency-audit, SBOM-generation, SAST, and example-container hardening evidence refresh:

- Local `cargo build --release --locked` passed.
- Local `cargo audit` passed after fetching the RustSec advisory database and scanning `Cargo.lock`.
- Local CycloneDX SBOM generation passed with non-empty SBOM files for `arb-core` and `arb-agent`; generated SBOM files were removed from the working tree and not committed.
- GitHub Actions `cargo build --release --locked`, `cargo audit`, CycloneDX SBOM generation, CodeQL Rust SAST local-SARIF analysis, example container image build, Trivy image-scan gate, Gitleaks secret-pattern scan gate, lightweight hardening evidence index artifact upload, and GitHub Step Summary hardening evidence pointer generation passed in the latest referenced run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602` for commit `0b98a9a31d3701704d950779ad989daefcf1193b`.
- The initial upload-based CodeQL attempt in run `https://github.com/dominator509/arbyclaw/actions/runs/26199105621` failed because GitHub code scanning is not enabled for this repository. The workflow was narrowed to generate and verify local SARIF in CI without uploading to the GitHub Security tab.
- A 2026-05-21 attempt to enable CodeQL default setup through the GitHub API failed with `Code scanning is not enabled for this repository`, confirming that GitHub Security-tab upload processing remains blocked by repository security settings or plan/support constraints rather than by local source code.
- The CI workflow now keeps local-SARIF SAST evidence available as a short-retention Actions artifact while GitHub code scanning upload remains disabled.
- A local example-only container image build using `deployment/container/Containerfile.example` passed after adding `.dockerignore` to keep local build outputs and secret-like environment files out of the Docker build context. This does not validate a production image, deployment, runtime service, or rollout.
- GitHub Actions run `https://github.com/dominator509/arbyclaw/actions/runs/26209401284` built the example container image and uploaded Trivy evidence, then failed the critical-vulnerability gate because the Debian slim runtime included fixable `libgnutls30` critical findings (`CVE-2026-33845` and `CVE-2026-42010`). The runtime base was changed to a nonroot distroless Debian 12 image, locally rebuilt successfully, and validated by the passing example image-scan gate in run `https://github.com/dominator509/arbyclaw/actions/runs/26210031540`.
- Local Gitleaks `v8.30.1` secret-pattern scanning via the pinned container image passed with no leaks found; the redacted JSON evidence file was removed from the working tree. The CI Gitleaks secret-pattern scan gate also passed and uploaded a short-retention redacted evidence artifact in run `https://github.com/dominator509/arbyclaw/actions/runs/26271152507`.
- GitHub Actions run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602` uploaded the lightweight `hardening-evidence-index` artifact, plus `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and the Docker Buildx build record artifact, making non-secret hardening evidence easier to locate without changing runtime behavior.
- The same run wrote a GitHub Step Summary from the `hardening-evidence-index` job with non-secret artifact pointers, producing job results, run URL, commit, and explicit non-claims for easier lookup from the workflow page.
- Release-review records should reference the latest run URL, commit, artifact names, and artifact IDs as locator evidence only; SBOM review, dependency-audit review, production image review, GitHub code scanning upload processing or accepted deferral, staging, load, penetration, rollback, incident, live exchange/RPC, custody, compliance, and production-readiness reviews remain open until separately reviewed and recorded.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` now includes a CI evidence lookup note for finding the Step Summary and short-retention artifacts without changing CI or runtime behavior.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` now includes a manual evidence review checklist for confirming run identity, job outcomes, non-empty non-secret artifacts, and remaining release-review gaps before future release use.
- `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now provides a short non-secret release-review record template for run URLs, artifact names, reviewer, outcome, and unresolved gaps without changing CI or runtime behavior.
- `hardening/PRODUCTION_READINESS_CHECKLIST.md` now points release reviewers to `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` for non-secret evidence records before any future readiness review.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md`, `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md`, and `hardening/PRODUCTION_READINESS_CHECKLIST.md` now document the non-secret GitHub code-scanning settings evidence path for GAP-0075 without changing CI upload behavior, runtime behavior, or repository secrets handling.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a retained-artifact review checklist and non-secret retention fields so operators can preserve CI evidence references without copying secret-bearing content into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret SBOM review checklist and SBOM review evidence fields so operators can review generated SBOM artifacts without storing dependency graphs or sensitive environment details in repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a lightweight operator CI artifact review checklist and evidence field for recording run URLs, artifact names, reviewer, outcome, and unresolved gaps without copying artifact contents or secret-bearing logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include evidence expiration and refresh guidance so retained CI artifact references are refreshed when they no longer match the release-review commit, branch, artifact availability, changed inputs, or operator-approved review window.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret release evidence reviewer sign-off fields for recording evidence acceptance, rejection, or follow-up without implying production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret release evidence rejection reason categories so rejected or incomplete evidence can be recorded consistently without copying sensitive artifact contents, logs, credentials, private URLs, wallet material, screenshots, or findings into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret release evidence follow-up owner and target review date fields so rejected or incomplete evidence can be assigned without implying evidence acceptance, production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence status legend so reviewers use consistent accepted, rejected, follow-up required, expired, and deferred outcomes without implying production readiness or live-funds approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret evidence retention location classification values for Actions artifact, approved external evidence store, unavailable, or deferred without copying evidence contents, paths, credentials, private URLs, wallet material, screenshots, or sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret reviewer attestation that release evidence records contain references only, not embedded artifact contents, logs, SARIF/SBOM contents, vulnerability tables, screenshots, credentials, private URLs, wallet material, or sensitive environment details.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer role field constrained to operator, release reviewer, AppSec reviewer, DevSecOps reviewer, or deferred without implying approval authority, production readiness, or live-funds approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret reviewer independence/conflict note constrained to independent, same operator, deferred, or not applicable without implying production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence review-window field constrained to current, expired, deferred, or not applicable without implying production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret reviewer decision rationale category constrained to sufficient, insufficient, stale, deferred, or not applicable without copying artifact contents, logs, SARIF/SBOM contents, vulnerability tables, credentials, private URLs, wallet material, sensitive environment details, or production-readiness claims.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now record the operator decision to keep `dominator509/arbyclaw` private and use the CI `codeql-sarif-evidence` artifact as the local SARIF-only SAST evidence path for now while GAP-0075 remains open/deferred for GitHub Security-tab upload processing.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret SBOM review decision record and SBOM generation artifact reference field so reviewers can reference run URLs, job names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying dependency graphs, package inventories, vulnerability tables, SBOM contents, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret dependency-audit review decision record and `cargo audit` gate reference field so reviewers can reference run URLs, job names, gate names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying advisory tables, dependency details, vulnerable package lists, CVE text, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret image-scan review decision record and Trivy `trivy-image-scan-evidence` artifact reference field so reviewers can reference run URLs, job names, gate names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying vulnerability tables, image layer details, package inventories, CVE text, base-image metadata beyond sanitized artifact references, private registry URLs, internal hostnames, credentials, wallet material, screenshots, or raw sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret secret-scan review decision record and Gitleaks `gitleaks-secret-scan-evidence` artifact reference field so reviewers can reference run URLs, job names, gate names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying secret-scan findings, secret-like snippets, match strings, file excerpts, private URLs, credentials, tokens, wallet material, screenshots, raw logs, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret retained-artifact expiration outcome fields per artifact type so reviewers can mark `hardening-evidence-index`, `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and Docker Buildx build record artifacts as `current`, `expired`, `unavailable`, or `refreshed` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence refresh trigger outcome field so reviewers can mark refresh reasons as `commit changed`, `artifact expired`, `workflow changed`, `reviewer unable to verify`, or `deferred` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up status field so reviewers can mark follow-up as `open`, `assigned`, `resolved`, `deferred`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up closure reason field so reviewers can mark closure reasons as `evidence refreshed`, `gap accepted for deferral`, `superseded by newer run`, `unable to verify`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up verification method field so reviewers can mark verification methods as `CI run review`, `artifact reference review`, `external evidence store review`, `not verified`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up verification timestamp field so reviewers can record review date/time without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up evidence source field so reviewers can mark sources as `GitHub Actions`, `external evidence store`, `local operator record`, `not available`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up evidence source locator field so reviewers can record a run URL, artifact name, or approved external reference without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up locator availability field so reviewers can mark locators as `available`, `expired`, `inaccessible`, `deferred`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up locator access note field so reviewers can record short sanitized notes such as `artifact retained`, `artifact expired`, `permissions unavailable`, `external reference reviewed`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up locator recheck date field so reviewers can record when the locator should be reviewed again without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- The release evidence reviewer follow-up metadata set is now considered complete enough for the current Phase 17 / Phase 18 roadmap boundary; future safe tasks should return to roadmap-aligned production-gap audit and external validation closure work unless an operator identifies a concrete missing non-secret routing field.
- This is release-build, dependency-audit, SBOM-generation, local-SARIF SAST, example-container image-scan, and secret-pattern scan evidence only. It does not validate SBOM review, GitHub code scanning upload processing, production container image readiness, systemd hardening, ARM validation, staging deployment, load testing, penetration testing, rollback drills, incident drills, live exchange/RPC sandbox validation, custody review, compliance review, or production readiness.

## Latest Candidate External Validation Review Record

2026-05-26 ArbyClaw non-secret external-validation review candidate:

- Review scope: Candidate CI locator evidence for release-build, dependency-audit gate, SBOM-generation gate, local-SARIF SAST artifact, example Trivy image-scan artifact, Gitleaks secret-pattern scan artifact, Docker Buildx record artifact, and hardening-evidence index.
- Repository / branch / commit: `dominator509/arbyclaw` / `main` / `0b98a9a31d3701704d950779ad989daefcf1193b`.
- Workflow run URL: `https://github.com/dominator509/arbyclaw/actions/runs/26443625602`.
- Evidence source: GitHub Actions.
- Evidence retention location classification: Actions artifact.
- Evidence references-only attestation: This record contains run URL, commit, artifact names, and artifact IDs only; it does not embed artifact contents, logs, SARIF contents, SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.
- Artifact locator references: `hardening-evidence-index` `7213484905`; `codeql-sarif-evidence` `7213379997`; `trivy-image-scan-evidence` `7213343486`; `gitleaks-secret-scan-evidence` `7213308494`; Docker Buildx build record `7213344332`.
- Reviewer follow-up status: follow-up required.
- Reviewer follow-up verification method: CI run review and artifact reference review.
- Reviewer decision rationale category: sufficient for candidate locator evidence; insufficient for production validation.
- Review outcome: Candidate evidence recorded for future external-validation review only. No SBOM review, dependency-audit review, production image review, GitHub code scanning upload processing, staging validation, load testing, penetration testing, rollback drill, incident drill, live exchange/RPC validation, custody review, compliance review, public exposure approval, deployment readiness, live-funds readiness, or production-readiness blocker is closed.

## Current Production Readiness %

87%

Reasoning:

- Governance and architecture foundation exist.
- Minimal Rust workspace scaffold, CI skeleton, safety docs, and structure validation script exist.
- Typed config, environment secret-reference handling, encrypted-keystore interface boundary, redacted secret material, initial mode gates, deny-by-default policy checks, append-only audit/state primitives, SQLite WAL local checkpoints, normalized market-data models, freshness classification, fee models, deterministic paper market-data, static paper fees, policy-gated paper execution reports, local paper-report checkpoint helper, CEX framework models/traits, DEX/Web3 framework models/traits, opportunity-engine models/traits, execution-planner draft models/traits, execution-adapter boundary records/traits, communications/CLI command and notification boundaries, embedded-dashboard local render boundaries, observability/runbook local record boundaries, deterministic testing/fuzzing/backtesting validation boundaries, deterministic packaging/deployment planning boundaries, deterministic external hardening evidence/checklist boundaries, and deterministic agentic handoff boundaries exist.
- Encrypted-keystore backend, SQLite WAL production durability validation, audit crash/concurrency validation, live market-data providers, exchange-specific CEX adapters, live DEX/RPC adapters, signer/custody backends, transaction broadcast controls, external adapter submission, real outbound communications adapters, real dashboard hosting/authentication, real observability/exporter/alert runtime, real property/fuzz/backtest runner execution, durable planner/adapter/communications/dashboard/observability/testing audit-state lifecycle beyond the local paper-report checkpoint helper, container/systemd/ARM validation, runtime deployment, broad external hardening execution, external agent execution validation, rollback drills, incident drills, and production validations are still incomplete.
- Live-funds risk remains high.

## Current Completed Phases

- Phase 0 — Governance Initialization
- Phase 1 — Rust Workspace Scaffold (scaffold created; current workspace Rust/CI validation covered)
- Phase 2 — Config, Secrets, and Mode Gates (implemented; current workspace Rust/CI validation covered)
- Phase 3 — Policy Engine and Trust Contract (implemented; current workspace Rust/CI validation covered)
- Phase 4 — Audit Journal and State Store (implemented as boundary; current workspace Rust/CI validation covered; SQLite WAL checkpoint store implemented; production durability validation deferred)
- Phase 5 — Market Data Core (implemented as boundary; current workspace Rust/CI validation covered; live provider validation deferred)
- Phase 6 — Simulated/Paper Connectors (implemented as deterministic boundary; current workspace Rust/CI validation covered; paper-model limitations deferred)
- Phase 7 — CEX Connector Framework (implemented as typed framework boundary; current workspace Rust/CI validation covered; live exchange validation deferred)
- Phase 8 — DEX/Web3 Connector Framework (implemented as typed framework boundary; current workspace Rust/CI validation covered; live RPC validation, signer validation, and broadcast validation deferred)
- Phase 9 — Opportunity Engine (implemented as deterministic discovery/ranking boundary; current workspace Rust/CI validation covered; advanced route modeling and live-data validation deferred)
- Phase 10 — Execution Planner (implemented as draft-only planning boundary; current workspace Rust/CI validation covered; audit/state lifecycle, adapter handoff, and live execution validation deferred)
- Phase 11 — Execution Adapters (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; durable audit/state lifecycle and live submission validation deferred)
- Phase 12 — Communications and CLI (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real outbound integrations and audit/state lifecycle deferred)
- Phase 13 — Embedded Dashboard (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real hosting, auth, and audit/state lifecycle deferred)
- Phase 14 — Observability and Runbooks (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real telemetry runtime, exporters, alerts, and audit/state lifecycle deferred)
- Phase 15 — Testing, Fuzzing, and Backtesting (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real property/fuzz/backtest execution, load testing, and penetration testing deferred)
- Phase 16 — Packaging and Deployment (implemented as deterministic model/docs boundary; current release-build CI gate covered; production container/systemd/ARM validation, runtime deployment, and rollback drills deferred)
- Phase 17 — External Production Hardening (implemented as deterministic evidence/checklist boundary; real external hardening execution deferred)
- Phase 18 — Agentic Handoff Package (implemented as deterministic model/docs boundary; external agent execution and production validation deferred)

## Current Incomplete Phases

None inside the current ChatGPT Project Mode roadmap. Production readiness remains blocked by external validation and infrastructure gaps below.

## Deferred Production Tasks

See gap entries below.

## Environment-Limited Tasks

See gap entries below.

## Missing Infrastructure Tasks

See gap entries below.

## Missing Deployment Tasks

See gap entries below.

## Missing Security Validations

See gap entries below.

## Missing Runtime Validations

See gap entries below.

## Missing CI/CD Validations

See gap entries below.

## Missing External Integration Tests

See gap entries below.

## Missing Penetration Tests

See gap entries below.

## Missing Observability Validation

See gap entries below.

## Missing Rollback Validation

See gap entries below.

## Missing Load/Performance Tests

See gap entries below.

## Temporary Mock/Stubs To Replace

- `arb-agent` binary is scaffold-only and must be replaced/extended with real runtime orchestration in later phases.
- `PolicyEngine` is implemented and connected to Phase 11 adapter-boundary revalidation only; live execution-adapter integration remains missing.
- `AppendOnlyAuditJournal` is implemented but not yet integrated into runtime/execution paths or externally durability-tested.
- `InMemoryStateStore` remains a non-production test/local wiring implementation; `SqliteWalStateStore` now provides local non-secret SQLite WAL checkpoint persistence, but runtime lifecycle wiring and production durability validation are still incomplete.
- Encrypted-keystore provider is an interface boundary only and must be replaced with a real encrypted backend.
- Market-data and fee provider traits exist, but all real REST/WebSocket/paid-provider implementations are missing.
- `PaperMarketDataProvider`, `PaperFeeProvider`, and `PaperExecutionAdapter` are deterministic paper scaffolds and must not be treated as live connector implementations.
- `CexVenueProfile`, `CexConnectorRegistry`, `CexOrderRequest`, `CexPolicyGate`, and CEX connector traits are framework boundaries only and must not be treated as exchange-specific live adapters.
- `Web3ChainProfile`, `DexTokenProfile`, `DexRouterProfile`, `DexConnectorRegistry`, `DexSwapQuoteRequest`, `Web3TransactionSimulationRequest`, `DexPolicyGate`, and DEX/Web3 connector traits are framework boundaries only and must not be treated as live RPC, signing, simulation, bridge, or broadcast adapters.
- `ExecutionPlannerConfig`, `ExecutionPlannerRequest`, `ExecutionPlanDraft`, `ExecutionPlanStep`, `PlannerPolicyOutcome`, and planner traits are draft-only boundaries and must not be treated as adapter submission, order placement, signing, broadcast, or live execution implementations.
- `ExecutionAdapterConfig`, `ExecutionAdapterRequest`, `ExecutionAdapterRunRecord`, `ExecutionAdapterAttempt`, `ExecutionFillRecord`, `ExecutionReconciliationRecord`, and execution-adapter traits are framework boundaries only and must not be treated as live exchange adapters, RPC adapters, signing, broadcast, or production execution implementations.
- `CommunicationBoundaryConfig`, `OperatorCommand`, `RoutedOperatorCommand`, `OperatorNotification`, `NotificationDispatchRecord`, command-router traits, and notification-publisher traits are local model boundaries only and must not be treated as real messaging integrations, platform-token handling, remote command execution, or live operator control infrastructure.
- `DashboardBoundaryConfig`, `DashboardServerBinding`, `DashboardSnapshot`, `DashboardPanel`, `DashboardRenderRequest`, `DashboardRenderRecord`, and dashboard-renderer traits are local model boundaries only and must not be treated as real dashboard hosting, browser delivery, authentication, public exposure, or live operator control infrastructure.
- `ValidationHarnessConfig`, `ValidationTestCase`, `ValidationFixtureRecord`, `FuzzCorpusDefinition`, `BacktestScenarioDefinition`, `ValidationPlan`, `ValidationRunRecord`, and validation-harness traits are local model boundaries only and must not be treated as actual Rust test execution, external fuzzer execution, live network testing, backtest result validation, CI success, load testing, or penetration testing.
- `PackagingBoundaryConfig`, `PackageTargetPlan`, `ServiceHardeningProfile`, `ReleaseGate`, `RollbackStep`, `DeploymentPackagePlan`, `DeploymentPackageRecord`, and packaging/deployment planner traits are local model boundaries only and must not be treated as actual builds, container images, service installs, ARM binaries, runtime deployments, rollback-drill success, CI success, or production release readiness.
- `ExternalHardeningBoundaryConfig`, `HardeningEvidenceRecord`, `ProductionHardeningPlan`, `ExternalHardeningReviewRecord`, and hardening-review traits are local evidence/checklist boundaries only and must not be treated as completed CI, release build, dependency audit, SBOM, image scan, staging deployment, load test, penetration test, rollback drill, incident drill, live exchange/RPC validation, production readiness review, public exposure approval, or live-funds approval.

## Manual Human Tasks Required

See gap entries below.

## Future Agentic Continuation Tasks

See gap entries below.

## Highest-Risk Remaining Gaps

1. No encrypted secret/custody backend or signer boundary.
2. Audit journal is not externally validated, crash-tested, concurrency-tested, or fully connected to execution adapters.
3. SQLite WAL state store exists for local checkpoints, but production durability validation and runtime lifecycle wiring are missing.
4. Policy engine is connected to paper execution, Phase 7 CEX validation, Phase 8 DEX/Web3 framework validation, Phase 10 draft planner preflight, and Phase 11 adapter-boundary revalidation only; no live execution adapters or external submission exist.
5. No wallet signer boundary.
6. No exchange-specific live CEX adapters, rate-limit validation, or fee-schedule verification.
7. No live DEX/Web3 RPC adapters, signer integration, transaction simulation integration, spender approval controls, or broadcast controls.
8. No real outbound communications adapters, platform-token handling, or authenticated remote command channels.
9. No real dashboard hosting, browser authentication, CSRF protection, or penetration-tested operator UI.
10. No real observability exporters, metrics endpoint, log shipping, alert routing, or incident-drill validation.
11. No actual Rust/property/fuzz/backtest execution or curated validation corpus.
12. No actual package build, container build, systemd install, ARM build, rollback drill, or deployment validation.
13. Initial CI, locked release-build, dependency-audit, SBOM-generation, and local-SARIF CodeQL SAST evidence exists; SBOM review, GitHub code scanning upload processing, image scan, staging, load, penetration, rollback, incident, and production-readiness evidence remain missing.
14. No production security review.
15. No runtime testing with real market conditions.
16. No legal, jurisdiction, tax, or exchange terms-of-service review.

## Recommended Next Production Phase

Phase 18 — Agentic Handoff Package.

Before implementation begins, create `PHASE_18_SUBROADMAP.md`. Also run the deferred Rust validation commands, Phase 16 package/deployment validations, and Phase 17 external hardening validations in a capable external environment as soon as available.

---

# Gap Entries

## GAP-0001 — Rust Workspace Validation Deferred

- Unique ID: GAP-0001
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 1 / Phase 2
- Subsystem association: Repository scaffold / build system / config subsystem
- Description: A minimal Rust workspace scaffold and Phase 2 config modules now exist, and Cargo-based validation has current local and GitHub Actions evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and structure validation after future changes.
- Exact future tooling/environment required: Local development machine or CI runner with Rust stable, Cargo, rustfmt, clippy, Python 3, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 2 config changes first, then Phase 1 scaffold files if external validation exposes unrecoverable defects.

## GAP-0002 — `PHASE_1_SUBROADMAP.md` Created

- Unique ID: GAP-0002
- Phase association: Phase 1
- Subsystem association: Roadmap governance
- Description: `PHASE_1_SUBROADMAP.md` now exists and records Phase 1 objectives, deliverables, sequence, validation, rollback, limitations, and continuation tasks.
- Why incomplete: Not incomplete for document creation; external validation items remain tracked under GAP-0001 and GAP-0029.
- Why blocked in ChatGPT Project Mode: Not blocked.
- Risk level: Low
- Dependency requirements: Continued alignment with `ROADMAP.md` and `ARCHITECTURE.md`.
- Exact future validation required: Reconfirm Phase 1 roadmap alignment after future material validation or governance changes.
- Exact future tooling/environment required: Markdown review plus local/CI validation evidence when linked implementation changes occur.
- Recommended future agent type: Principal Systems Architect
- Estimated production impact: Deterministic sequencing restored for Phase 1.
- Completion criteria: Sub-roadmap remains aligned with scaffold and gap tracker.
- Rollback considerations: Revert the sub-roadmap file and roadmap/gap tracker updates if Phase 1 is rolled back.

## GAP-0003 — Encrypted Secret Backend Not Implemented

- Unique ID: GAP-0003
- Phase association: Phase 2 / Phase 17
- Subsystem association: Secrets and custody
- Description: Phase 2 added reference-only secret types, redacted `SecretMaterial`, a `SecretProvider` trait, and an environment provider skeleton, but no encrypted local keystore backend exists.
- Why incomplete: Encrypted storage, key derivation, file permissions, OS keyring integration, lifecycle handling, and zeroization have not been implemented or validated.
- Why blocked in ChatGPT Project Mode: Real encrypted-at-rest behavior, OS keyring behavior, filesystem permissions, and secret injection require local/CI/runtime environments outside ChatGPT.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, dependency review for encryption/zeroization crates, local filesystem or OS keyring target, policy engine, audit redaction.
- Exact future validation required: redaction tests, no-secret-log tests, encrypted-at-rest tests, key-load failure tests, zeroization tests, file-permission tests, backup/restore tests, secret rotation tests.
- Exact future tooling/environment required: Rust toolchain, local keyring or encrypted file backend, test secrets only, filesystem permission controls, CI secret-scanning.
- Recommended future agent type: AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks live credentials and wallet custody.
- Completion criteria: Secrets can be loaded via approved provider without appearing in logs, config, prompts, telemetry, or persisted plaintext; encrypted backend passes local and CI validation.
- Rollback considerations: Disable all live modes, remove secret backend integration, revoke any test credentials, and wipe local encrypted test stores.

## GAP-0004 — API Keys and Wallet Credentials Not Available

- Unique ID: GAP-0004
- Phase association: Phase 17
- Subsystem association: External integrations / secrets
- Description: No real exchange API keys, paid market-data keys, or wallet credentials are available.
- Why incomplete: User has not created or supplied credentials, and credentials must not be placed in chat or Markdown.
- Why blocked in ChatGPT Project Mode: Real secret provisioning must occur outside ChatGPT in the target runtime environment.
- Risk level: Critical
- Dependency requirements: Secret manager, config references, operator-created exchange accounts and API keys.
- Exact future validation required: Read-only credential validation first, sandbox credential validation where available, least-privilege scope verification.
- Exact future tooling/environment required: Exchange accounts, wallet, local encrypted secret store, network access.
- Recommended future agent type: DevSecOps Orchestrator + Human Operator
- Estimated production impact: Blocks live integrations.
- Completion criteria: Credentials are provisioned outside repo, scoped minimally, and validated without secret leakage.
- Rollback considerations: Revoke keys at exchanges/providers and wipe local encrypted keystore.

## GAP-0005 — Policy Engine Implemented; External Validation and Integration Pending

- Unique ID: GAP-0005
- Phase association: Phase 3 / Phase 11 / Phase 15
- Subsystem association: Policy and trust contract
- Description: Deny-by-default policy engine and trust-contract enforcement now exist in `crates/arb-core/src/policy.rs`, with current local and GitHub Actions Rust/Cargo validation evidence. Deeper property/fuzz validation and durable runtime integration remain incomplete.
- Why incomplete: Current workspace Rust/Cargo validation is covered, but property-test runners, fuzzers, durable audit integration, and real runtime integration remain incomplete.
- Why blocked in ChatGPT Project Mode: Not blocked for local Rust/Cargo validation in the current workspace; deeper property/fuzz/runtime validations require future tooling and scope.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 4 audit journal, future execution adapters, property/fuzz test framework.
- Exact future validation required: Property tests, fuzz tests, denial-path tests, mode-gate tests, unknown-destination tests, stale-data tests, kill-switch tests, live-runtime-denial tests, connector integration tests, plus rerun standard Cargo validation after changes.
- Exact future tooling/environment required: Rust test runner, clippy, property testing crate, fuzzing harness, CI runner.
- Recommended future agent type: Policy Engine Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Policy architecture now exists, but safe live execution remains blocked until policy is validated and mandatory in every execution path.
- Completion criteria: Policy code compiles, tests pass, future execution adapters cannot submit orders or sign transactions without a policy approval and durable audit record.
- Rollback considerations: Remove `policy.rs`, remove policy exports, revert CLI policy initialization, disable execution adapters, and force Observe/Paper modes.

## GAP-0006 — Wallet Signer Boundary Not Implemented

- Unique ID: GAP-0006
- Phase association: Phase 8 / Phase 11
- Subsystem association: Web3 custody / signer
- Description: No constrained signer boundary exists for wallet-controlled funds.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Code can be drafted, but hardware wallet, keyring, chain RPC, and real transaction validation require external environment.
- Risk level: Critical
- Dependency requirements: Secret manager, policy engine, DEX transaction model, audit journal.
- Exact future validation required: signer isolation tests, no-LLM-access tests, unauthorized destination denial, transaction simulation, chain testnet execution.
- Exact future tooling/environment required: Test wallet, testnet RPCs, local runtime, optional hardware wallet.
- Recommended future agent type: Web3 Connector Agent + AppSec Lead
- Estimated production impact: Blocks DEX live execution and any wallet autonomy.
- Completion criteria: Signer only signs policy-approved intents and never exposes raw keys.
- Rollback considerations: Remove signer provider and disable Web3 live mode.

## GAP-0007 — Audit Journal Implemented; External Validation and Runtime Integration Pending

- Unique ID: GAP-0007
- Phase association: Phase 4 / Phase 11 / Phase 17
- Subsystem association: Audit journal / state store
- Description: Phase 4 added `AppendOnlyAuditJournal`, typed audit events, redacted metadata values, hash-chained JSONL records, replay validation, `StateStore`, `StateCheckpoint`, a non-production in-memory state store, and a SQLite WAL-backed checkpoint store for local non-secret state.
- Why incomplete: Current Rust/Cargo validation exists for the audit/state boundary, but the audit journal and SQLite WAL state store are not yet wired into runtime, policy, planner, connector, signer, or execution-adapter paths; crash/concurrency/filesystem durability validation remains incomplete.
- Why blocked in ChatGPT Project Mode: Current local and CI validation covers compile/test/lint only; filesystem durability tests, crash tests, concurrent append tests, and real runtime validation require future implementation and a capable external environment.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 5 market data, future execution planner/adapters, SQLite WAL state backend, filesystem permission model.
- Exact future validation required: Append/reopen tests, tamper-detection tests, redaction tests, crash-recovery tests, concurrent append tests, file-permission tests, WAL persistence tests, schema migration tests, plus rerun standard Cargo validation after changes.
- Exact future tooling/environment required: Rust, Cargo, local filesystem, CI runner, SQLite runtime, and migration tooling.
- Recommended future agent type: Audit and Observability Agent + Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Accountability architecture now exists, but live trading remains blocked until every execution path writes durable redacted audit records and durability is validated.
- Completion criteria: Every intent, policy decision, execution, signer request, connector result, failure, and reconciliation event is durably journaled without secrets; journal replay detects tampering; crash/concurrency tests pass.
- Rollback considerations: Disable live execution, revert audit/state modules, remove audit dependencies, and force Observe/Paper modes if validation fails.

## GAP-0008 — Market Data Core Boundary Implemented; Live Providers Missing

- Unique ID: GAP-0008
- Phase association: Phase 5
- Subsystem association: Market data
- Description: Phase 5 normalized market-data models, freshness classification, fee models, and provider trait boundaries exist with current local and GitHub Actions validation evidence. No live REST/WebSocket CEX provider, DEX quote provider, paid data-provider adapter, reconnect logic, or rate-limit logic exists.
- Why incomplete: The deterministic boundary exists, but live provider implementation and external provider validation remain incomplete.
- Why blocked in ChatGPT Project Mode: Implementation possible; live provider validation requires external network and credentials.
- Risk level: High
- Dependency requirements: Current Rust validation baseline and config subsystem.
- Exact future validation required: quote normalization tests, stale-data tests, fee model tests, order-book depth tests.
- Exact future tooling/environment required: Rust test runner; later live network access.
- Recommended future agent type: Rust Implementation Agent + Exchange Connector Agent
- Estimated production impact: Blocks reliable opportunity detection.
- Completion criteria: Market data is normalized, provenance-tracked, and freshness-checked.
- Rollback considerations: Revert market-data crate changes.

## GAP-0009 — Simulated/Paper Connector Boundary Implemented; Production Realism Missing

- Unique ID: GAP-0009
- Phase association: Phase 6
- Subsystem association: Simulation / paper trading
- Description: Phase 6 deterministic in-memory paper market data, static paper fee schedules, and policy-gated paper execution reports exist with current local and GitHub Actions validation evidence. Production-realistic fill modeling, balance constraints, durable audit/state integration, and richer replay validation remain incomplete.
- Why incomplete: The deterministic paper boundary exists, but production-realistic paper simulation and lifecycle integration remain incomplete.
- Why blocked in ChatGPT Project Mode: Not blocked.
- Risk level: High
- Dependency requirements: Market-data core, execution intent model, audit journal.
- Exact future validation required: deterministic replay tests, paper fill tests, fee/slippage simulation tests, failed-trade simulation tests.
- Exact future tooling/environment required: Rust test runner, fixture data.
- Recommended future agent type: Rust Implementation Agent
- Estimated production impact: Blocks safe validation before live execution.
- Completion criteria: Strategies can run without live funds and produce reproducible results.
- Rollback considerations: Disable simulation feature flag or revert crate changes.

## GAP-0010 — CEX Framework Implemented; Exchange-Specific Adapters Missing

- Unique ID: GAP-0010
- Phase association: Phase 7
- Subsystem association: CEX connectors
- Description: Phase 7 CEX framework types, venue profiles, capability registry, order request models, policy gates, and connector traits exist with current local and GitHub Actions validation evidence. No exchange-specific REST, WebSocket, sandbox, balance, order, cancel, fee, or rate-limit adapters exist.
- Why incomplete: The CEX framework boundary exists, but exchange-specific adapters and external validation remain incomplete.
- Why blocked in ChatGPT Project Mode: Live credentialed testing, sandbox testing, and rate-limit behavior require external network and accounts.
- Risk level: High
- Dependency requirements: Market data core, secret manager, config, policy, audit.
- Exact future validation required: mocked API tests, sandbox tests, read-only credential tests, order placement tests in sandbox where supported.
- Exact future tooling/environment required: Exchange accounts, API credentials, network access.
- Recommended future agent type: Exchange Connector Agent
- Estimated production impact: Blocks cross-exchange arbitrage.
- Completion criteria: At least one read-only connector and one sandbox/paper execution path work behind policy gates.
- Rollback considerations: Disable connector in capability registry and revoke credentials.

## GAP-0011 — Live DEX/Web3 Adapters Missing

- Unique ID: GAP-0011
- Phase association: Phase 8 / Phase 11 / Phase 17
- Subsystem association: DEX/Web3 connectors
- Description: Phase 8 now defines framework-only chain, token, router, quote, local simulation, policy-gate, and connector-trait boundaries, but no live chain RPC, router, aggregator, signer, transaction simulation, bridge, or broadcast adapter exists.
- Why incomplete: The smallest safe Phase 8 patch created typed boundaries before any external Web3 integration or signing behavior.
- Why blocked in ChatGPT Project Mode: Live RPC, testnet/mainnet simulation, wallet validation, protocol documentation checks, and signer custody require external environment and reviewed credentials/wallet setup.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, policy engine, signer boundary, encrypted custody backend, market data, audit journal, state store, protocol allowlists, and external Web3 runtime.
- Exact future validation required: contract allowlist tests, router/spender tests, transaction simulation tests, slippage tests, gas estimation tests, MEV-risk tests, approval hygiene tests, nonce tests, testnet execution tests, and no-broadcast-until-approved tests.
- Exact future tooling/environment required: RPC endpoints, test wallet, chain testnets, local runtime, mocked RPC fixtures, CI runner, and signer test harness with non-production keys outside the repository.
- Recommended future agent type: Web3 Connector Agent + AppSec Lead + DevSecOps Orchestrator
- Estimated production impact: Blocks live DEX/CEX and on-chain arbitrage.
- Completion criteria: At least one DEX/router quote path and one non-broadcasting testnet/simulated transaction path pass policy, audit, state, signer, and external validation without secret leakage.
- Rollback considerations: Disable Web3 feature flag, remove router registration, revoke provider/test credentials, preserve audit records, and remove signer provider references.

## GAP-0012 — Opportunity Engine Advanced Validation and Integration Incomplete

- Unique ID: GAP-0012
- Phase association: Phase 9 / Phase 10 / Phase 15 / Phase 17
- Subsystem association: Opportunity engine
- Description: Phase 9 added deterministic opportunity-engine models, freshness checks, fee-aware cross-venue top-of-book discovery, and deterministic ranking, but advanced path discovery, inventory-aware sizing, depth-aware slippage, planner integration, and external validation are incomplete.
- Why incomplete: Phase 9 intentionally added the smallest safe discovery/ranking boundary without execution-intent generation, live data, live connectors, or advanced route search.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the opportunity-engine boundary, but fixture replay at production scale, live market data, exchange/account context, and external production validation require tooling outside this environment.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, market-data core, fee models, simulated connectors, Phase 10 execution planner, Phase 15 backtesting/scenario harness, live provider fixtures.
- Exact future validation required: unit tests, replay tests, fee-aware ROI tests, stale-data denial tests, false-positive tests, triangular-route tests, depth/slippage tests, inventory constraints, settlement-latency tests, planner handoff tests, and backtesting over historical fixtures.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, fixture datasets, mocked market providers, later live data providers, CI runner.
- Recommended future agent type: Rust Implementation Agent + Strategy/Backtesting Agent + AppSec Lead
- Estimated production impact: Core discovery boundary exists, but safe and profitable production opportunity selection remains blocked until advanced validation and planner integration are complete.
- Completion criteria: Engine produces deterministic, fee-adjusted, policy-ready opportunity records; advanced route/search models pass replay and false-positive tests; execution planner consumes only validated candidates.
- Rollback considerations: Disable opportunity engine feature or revert `crates/arb-core/src/opportunity.rs`, exports, CLI status text, and roadmap/gap updates.

## GAP-0013 — Execution Planner Implemented; Adapter, Audit, and External Validation Pending

- Unique ID: GAP-0013
- Phase association: Phase 10 / Phase 11 / Phase 14 / Phase 17
- Subsystem association: Execution planner
- Description: Phase 10 added deterministic draft-only conversion from validated opportunities to per-leg `ExecutionIntent` records, policy preflight outcomes, sequencing steps, and failure-mode boundaries, but no adapter submission, durable audit/state lifecycle, partial-fill engine, or live execution validation exists.
- Why incomplete: Phase 10 intentionally stops at model-only plan drafts; future phases must integrate durable audit/state writes and adapter handoff.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the execution-planner boundary, but filesystem/database durability, adapter lifecycle tests, and live/sandbox venue behavior require future implementation and external environments.
- Risk level: High
- Dependency requirements: Opportunity engine, policy engine, audit journal, durable state store, Phase 11 execution adapters, Phase 15 scenario/backtesting harness.
- Exact future validation required: intent-generation tests, policy-preflight tests, partial-fill tests, timeout tests, cancellation tests, failure-mode tests, audit-before-adapter tests, restart/recovery tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem/database, mocked adapter fixtures, CI runner.
- Recommended future agent type: Rust Implementation Agent + Policy Engine Agent + Audit and Observability Agent
- Estimated production impact: Draft planning no longer blocks architecture, but live and paper execution safety still depend on adapter, audit, and state integration.
- Completion criteria: Opportunities become durable, auditable, policy-checked plans that adapters can consume only after audit/state writes succeed and external tests pass.
- Rollback considerations: Remove `crates/arb-core/src/planner.rs`, remove planner exports/status text, force Paper/Observe modes, and return roadmap/gap tracker to Phase 9 state.

## GAP-0014 — Live Execution Adapter Submission Missing

- Unique ID: GAP-0014
- Phase association: Phase 11 / Phase 17
- Subsystem association: Execution adapters
- Description: Phase 11 adds deterministic execution-adapter model/trait boundaries, policy revalidation, attempt records, fill records, and reconciliation records, but no live order submission, transaction submission, external exchange/RPC adapter, live fill tracking, or real balance reconciliation exists.
- Why incomplete: Phase 11 intentionally stops at fail-closed model boundaries; live submission requires later custody, audit/state, connector, and external validation work.
- Why blocked in ChatGPT Project Mode: Real live validation requires exchange accounts, network access, testnet/mainnet environments, signer/custody setup, and controlled funds outside the repository.
- Risk level: Critical
- Dependency requirements: Policy engine, secret manager, audit journal, durable state store, connector framework, execution planner, signer boundary, exchange-specific adapters.
- Exact future validation required: sandbox tests, paper scenario tests, small-capital guarded live tests, reconciliation tests, failure recovery tests, audit-before-submit tests, kill-switch tests, duplicate submission tests.
- Exact future tooling/environment required: Exchange accounts, test wallets, network access, staging runtime, CI runner, external audit/state backend.
- Recommended future agent type: Exchange Connector Agent + Web3 Connector Agent + Release Engineering Authority
- Estimated production impact: Blocks autonomous profit/loss generation.
- Completion criteria: Live adapters execute only policy-approved, audit-journaled, state-checkpointed intents and reconcile outcomes without exposing secrets.
- Rollback considerations: Disable live mode, revoke keys, halt signer, preserve audit records, revert adapter feature flags.

## GAP-0015 — Real Communications Channels Missing

- Unique ID: GAP-0015
- Phase association: Phase 12 / Phase 17
- Subsystem association: Communications
- Description: Phase 12 adds typed command, notification, redaction, routing, and local dispatch-record boundaries, but Telegram, Discord, Matrix, email, Slack, PagerDuty, Signal, iMessage, webhook, SMS, and other real outbound integrations are not implemented.
- Why incomplete: Phase 12 intentionally stops at local deterministic model/trait boundaries with outbound network delivery disabled.
- Why blocked in ChatGPT Project Mode: Real integrations require external accounts, platform tokens, device/app approvals, network access, abuse-prevention review, and channel-specific security validation.
- Risk level: Medium
- Dependency requirements: CLI, config, command routing, redaction layer, authentication/authorization model, audit/state integration, platform-specific adapters.
- Exact future validation required: mocked command tests, auth tests, notification delivery tests, replay tests, injection-resistance tests, no-secret-render tests, channel permission tests, rate-limit tests, and fail-closed token revocation tests.
- Exact future tooling/environment required: platform tokens, test channels, local runtime, network access, CI secrets manager, external integration test accounts.
- Recommended future agent type: Rust Implementation Agent + AppSec Lead + Communications Integration Agent
- Estimated production impact: Reduces operator control and alerting.
- Completion criteria: Typed commands and notifications work over approved channels without exposing secrets, bypassing policy, or enabling unauthorized execution.
- Rollback considerations: Disable affected channel adapter, revoke tokens, preserve audit records, and fall back to local CLI only.

## GAP-0016 — Real Dashboard Hosting Missing

- Unique ID: GAP-0016
- Phase association: Phase 13 / Phase 17
- Subsystem association: Dashboard
- Description: Phase 13 local dashboard render models exist, but no real HTTP server, browser delivery, authentication, authorization, CSRF protection, or hosted operator UI exists.
- Why incomplete: Phase 13 intentionally stopped at local deterministic model/trait boundaries and rejects server startup/public exposure.
- Why blocked in ChatGPT Project Mode: Secure browser/server validation requires Rust runtime execution, a local browser/server harness, network binding inspection, authentication design, persistence, and AppSec review.
- Risk level: Medium
- Dependency requirements: Runtime, audit/state store, auth/session model, secure local web host design.
- Exact future validation required: local auth tests, CSRF tests if applicable, no-secret-render tests, localhost-only default tests, public-bind denial tests, live-control denial tests.
- Exact future tooling/environment required: Rust web framework, local browser for manual validation, CI runner, AppSec review.
- Recommended future agent type: Embedded Dashboard Agent + AppSec Lead
- Estimated production impact: Low to medium; dashboard is optional.
- Completion criteria: Dashboard hosting is disabled or localhost-only by default, authenticated where exposed, and exposes no secrets or live controls.
- Rollback considerations: Disable dashboard feature flag and fall back to local CLI/status records.

## GAP-0017 — Real Observability Runtime Missing

- Unique ID: GAP-0017
- Phase association: Phase 14 / Phase 17
- Subsystem association: Observability
- Description: Phase 14 local health, structured-log, metric, and runbook records exist, but no real tracing subscriber, metrics endpoint, OpenTelemetry/Prometheus exporter, log shipping, alert escalation, incident drill, or production telemetry runtime exists.
- Why incomplete: Phase 14 intentionally stopped at local deterministic model/trait boundaries and rejects metrics endpoint startup, public exposure, outbound alerts, and secret observability.
- Why blocked in ChatGPT Project Mode: Production telemetry validation requires deployed runtime, Rust execution, exporter infrastructure, authenticated endpoints, network binding inspection, alerting providers, and AppSec review.
- Risk level: High
- Dependency requirements: Runtime scaffold, subsystem events, audit/state integration, communications adapters for alert routing, secure endpoint design.
- Exact future validation required: structured logging tests, redaction tests, metrics endpoint tests, health endpoint tests, public-bind denial tests, exporter redaction tests, alert routing tests, incident drill tests, no-secret-telemetry tests.
- Exact future tooling/environment required: local runtime, optional Prometheus/OpenTelemetry stack, mocked alerting channels, CI runner, AppSec review.
- Recommended future agent type: Audit and Observability Agent
- Estimated production impact: Blocks production operations and incident response.
- Completion criteria: Runtime emits redacted logs, metrics, health status, and critical alerts through authenticated, audited, fail-closed channels without exposing secrets.
- Rollback considerations: Disable exporters and alerts while preserving local records.

## GAP-0018 — CI/CD Execution Covered for Current Workspace; Production CI Gates Still Limited

- Unique ID: GAP-0018
- Phase association: Phase 16 / Phase 17
- Subsystem association: CI/CD
- Description: GitHub Actions now runs on `dominator509/arbyclaw` and covers structure validation, formatting, workspace compilation, tests, clippy, locked release build, dependency audit, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing for pushed commits.
- Why incomplete: Current CI evidence exists, but production deployment gates, staging gates, release approval gates, rollback-drill gates, and live-integration gates remain incomplete.
- Why blocked in ChatGPT Project Mode: No longer blocked for hosted CI execution on the current repository; production-class CI gates require future infrastructure and operator-approved release workflows.
- Risk level: High
- Dependency requirements: Keep GitHub Actions, Rust stable, Cargo, rustfmt, clippy, Python 3, dependency-audit tooling, SBOM tooling, SAST artifact generation, image scanning, and secret scanning available for future runs.
- Exact future validation required: Re-run CI after future changes and preserve non-secret evidence references for formatting, check, tests, clippy, locked release build, dependency audit, SBOM generation, local-SARIF SAST, example image scan, secret-pattern scan, and hardening evidence indexing.
- Exact future tooling/environment required: GitHub Actions or equivalent CI service.
- Recommended future agent type: DevSecOps Orchestrator
- Estimated production impact: Blocks reliable releases.
- Completion criteria: CI pipeline runs and passes on hosted repository.
- Rollback considerations: Revert CI workflow changes.

## GAP-0019 — Deployment Packaging External Validation Missing

- Unique ID: GAP-0019
- Phase association: Phase 16 / Phase 17
- Subsystem association: Packaging/deployment
- Description: Phase 16 added deterministic packaging/deployment models and example-only container, systemd, ARM, and deployment documentation. Current evidence exists for locked release-build validation, an example-only container image build, CI image-scan evidence, dependency audit, SBOM generation, local-SARIF SAST, secret-pattern scanning, and hardening evidence indexing. Production validation is still missing: no production release artifact, production container image, service installation, ARM build, runtime deployment, or rollback drill has been executed.
- Why incomplete: Phase 16 intentionally produced plan records and templates only. Current CI evidence covers release-build and example-image feedback, but production artifact builds and deployment validation require target infrastructure and operator review.
- Why blocked in ChatGPT Project Mode: Production deployment validation requires a container runtime or packaging target, systemd host, ARM target/cross toolchain, filesystem permissions, rollback environment, and deployment infrastructure outside this chat.
- Risk level: Medium
- Dependency requirements: Keep current Rust/CI, locked release-build, example-image, image-scan, dependency-audit, SBOM, local-SARIF, secret-scan, and hardening-index evidence refreshable; add package/deployment plans, target host profile, runtime config, rollback procedure, and production release artifact storage.
- Exact future validation required: Refresh CI evidence for the candidate commit, then perform production release artifact build, production container build, image scan review, ARM build, service lint, service start/stop, config loading, non-root runtime validation, read-only filesystem validation, rollback drill, incident drill, and log/audit review.
- Exact future tooling/environment required: Current Rust/Cargo and CI runner for evidence refresh; container runtime, systemd Linux host or test container, ARM device or cross-compile target, and release artifact storage for production validation.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority
- Estimated production impact: Blocks production deployment.
- Completion criteria: Binary can be packaged and run on intended targets with documented, tested rollback and no secret leakage or accidental live-mode enablement.
- Rollback considerations: Stop service, restore previous binary/config, remove package/image/unit, preserve audit evidence, and keep Observe/Paper modes only.

## GAP-0020 — Penetration Testing Missing

- Unique ID: GAP-0020
- Phase association: Phase 17
- Subsystem association: Security validation
- Description: No penetration testing, wallet custody review, or adversarial testing has been performed.
- Why incomplete: No implementation exists yet.
- Why blocked in ChatGPT Project Mode: Requires external tools, running system, security expertise, and test environment.
- Risk level: Critical
- Dependency requirements: Implemented runtime, dashboard/communications, policy, secrets, execution paths.
- Exact future validation required: SAST, DAST where applicable, dependency audit, secret scan, command-injection tests, authz tests, wallet-safety review.
- Exact future tooling/environment required: Security testing environment, scanners, external reviewer.
- Recommended future agent type: AppSec Lead + External Security Engineer
- Estimated production impact: Blocks responsible live-funds deployment.
- Completion criteria: Findings remediated or accepted with documented risk.
- Rollback considerations: Disable live features until vulnerabilities are resolved.

## GAP-0021 — Load and Latency Testing Missing

- Unique ID: GAP-0021
- Phase association: Phase 15 / Phase 17
- Subsystem association: Performance validation
- Description: No throughput, latency, or resource-use tests exist.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Meaningful performance validation requires compiled runtime, network conditions, and target hardware.
- Risk level: High
- Dependency requirements: Runtime, market data, opportunity engine, connectors.
- Exact future validation required: quote ingestion load tests, opportunity ranking latency tests, memory footprint tests, ARM device tests, backpressure tests.
- Exact future tooling/environment required: benchmarking harness, target machines, network access.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Directly affects speed and missed opportunities.
- Completion criteria: Performance targets documented and met on target profiles.
- Rollback considerations: Disable high-frequency connectors or reduce polling/subscription rates.

## GAP-0022 — Legal, Tax, Jurisdiction, and Terms Review Missing

- Unique ID: GAP-0022
- Phase association: Phase 17
- Subsystem association: Compliance / operations
- Description: No legal, tax, exchange terms-of-service, or jurisdiction-specific review has been completed.
- Why incomplete: Requires human/external professional review.
- Why blocked in ChatGPT Project Mode: Legal and tax review cannot be completed by this environment.
- Risk level: Critical
- Dependency requirements: Intended jurisdiction, target exchanges, trading behavior, entity/account structure.
- Exact future validation required: legal review, tax/accounting review, exchange terms review, recordkeeping review.
- Exact future tooling/environment required: Qualified legal/tax professionals and exchange documentation.
- Recommended future agent type: Human Operator + External Counsel/CPA
- Estimated production impact: Could block or constrain live trading.
- Completion criteria: Documented approval or documented restrictions integrated into config/policy.
- Rollback considerations: Disable non-approved venues, strategies, and jurisdictions.

## GAP-0023 — Real Runtime Validation Missing

- Unique ID: GAP-0023
- Phase association: Phase 17
- Subsystem association: Runtime validation
- Description: No daemon, service, CLI, network, or long-running process validation has been performed.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Requires compiled runtime and target environment.
- Risk level: High
- Dependency requirements: Runtime implementation, config, logging, health checks.
- Exact future validation required: start/stop tests, crash recovery, config reload, service restart, daemon uptime soak test.
- Exact future tooling/environment required: local machine, VPS, systemd or process supervisor.
- Recommended future agent type: DevSecOps Orchestrator
- Estimated production impact: Blocks unattended operation.
- Completion criteria: Runtime runs reliably under target deployment profile.
- Rollback considerations: Stop service and restore previous binary/config.

## GAP-0024 — Rollback Validation Missing

- Unique ID: GAP-0024
- Phase association: Phase 16 / Phase 17
- Subsystem association: Release engineering
- Description: No production rollback drill has been executed.
- Why incomplete: No deployable runtime exists yet.
- Why blocked in ChatGPT Project Mode: Requires deployed environment and release artifacts.
- Risk level: High
- Dependency requirements: Packaging, deployment, persistent state, config migration strategy.
- Exact future validation required: binary rollback, config rollback, DB migration rollback/forward recovery, kill-switch verification.
- Exact future tooling/environment required: staging VPS/local service, release artifacts.
- Recommended future agent type: Release Engineering Authority
- Estimated production impact: Blocks safe production release.
- Completion criteria: Rollback procedures are tested and documented.
- Rollback considerations: This gap itself is about validating rollback; until complete, live releases must remain guarded.

## GAP-0025 — Handoff Package Implemented; External Handoff Execution Missing

- Unique ID: GAP-0025
- Phase association: Phase 18
- Subsystem association: Agentic handoff
- Description: Phase 18 deterministic handoff package records, continuation prompts, repository maps, external validation checklists, and future-agent instructions exist. No external coding agent, AppSec reviewer, DevSecOps reviewer, compliance reviewer, or human production reviewer has executed the handoff package and recorded non-secret evidence.
- Why incomplete: The handoff package boundary exists, but external handoff execution and evidence review remain incomplete.
- Why blocked in ChatGPT Project Mode: External handoff execution requires a real external agent or human reviewer workflow outside this local model boundary.
- Risk level: Medium
- Dependency requirements: Stable repo structure and current gap inventory.
- Exact future validation required: Handoff docs align with current architecture, roadmap, tests, gaps, and build commands.
- Exact future tooling/environment required: Markdown docs and optional zip packaging.
- Recommended future agent type: Handoff Agent
- Estimated production impact: Affects future continuation quality.
- Completion criteria: External agent can resume from docs without losing roadmap position.
- Rollback considerations: Revert handoff docs if stale or inaccurate.

## GAP-0026 — Paid Market Data Provider Evaluation Missing

- Unique ID: GAP-0026
- Phase association: Phase 5 / Phase 17
- Subsystem association: Market data integrations
- Description: Paid market-data providers have not been selected, contracted, or validated.
- Why incomplete: Requires budget, accounts, API credentials, and provider comparison.
- Why blocked in ChatGPT Project Mode: Provider signup, billing, key provisioning, and live testing require external action.
- Risk level: Medium
- Dependency requirements: Market-data provider abstraction and budget decision.
- Exact future validation required: latency comparison, coverage comparison, rate-limit testing, cost analysis, failure behavior tests.
- Exact future tooling/environment required: Provider accounts, API keys, benchmark environment.
- Recommended future agent type: DevSecOps Orchestrator + Human Operator
- Estimated production impact: Affects opportunity coverage and speed.
- Completion criteria: Provider list selected and integrated behind market-data abstraction.
- Rollback considerations: Disable provider connector and fall back to exchange-native data.

## GAP-0027 — Withdrawal Policy Not Implemented or Validated

- Unique ID: GAP-0027
- Phase association: Phase 3 / Phase 11 / Phase 17
- Subsystem association: Custody / execution policy
- Description: The user wants full fund control eventually, but safe withdrawal policy is not implemented or validated.
- Why incomplete: Policy engine, signer boundary, allowlists, audit, and external validation do not exist yet.
- Why blocked in ChatGPT Project Mode: Real withdrawal testing requires live accounts/wallets and must be performed cautiously outside this environment.
- Risk level: Critical
- Dependency requirements: Policy engine, secret manager, audit journal, allowlist manager, execution adapters, human-reviewed destination list.
- Exact future validation required: unknown-address denial, allowlisted-address approval, per-period withdrawal limits, operator confirmation, sandbox/testnet withdrawal tests, revocation tests.
- Exact future tooling/environment required: Exchange accounts, wallet, test funds, local runtime.
- Recommended future agent type: AppSec Lead + Release Engineering Authority + Human Operator
- Estimated production impact: Mishandled withdrawals could cause irreversible loss.
- Completion criteria: Withdrawals are disabled by default and only enabled under a signed/explicit local policy with tested allowlists and limits.
- Rollback considerations: Disable withdrawal capability, revoke API withdrawal permissions, rotate wallet keys if compromised.

## GAP-0028 — Strategy Parameter Library Not Implemented

- Unique ID: GAP-0028
- Phase association: Phase 2 / Phase 9 / Phase 10
- Subsystem association: Strategy profiles
- Description: The extensive strategy and command parameter library is only architecturally defined, not implemented.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Not blocked for code; real profitability tuning is environment-limited.
- Risk level: High
- Dependency requirements: Config models, policy engine, opportunity engine, execution planner.
- Exact future validation required: schema validation, invalid-profile denial tests, boundary tests, strategy replay tests, config migration tests.
- Exact future tooling/environment required: Rust test runner, fixture configs.
- Recommended future agent type: Rust Implementation Agent + Policy Engine Agent
- Estimated production impact: Blocks customizable autonomous behavior.
- Completion criteria: Strategy profiles are typed, validated, documented, and policy constrained.
- Rollback considerations: Revert strategy schema or disable unsafe profile fields.

## GAP-0029 — Local Cargo Validation Covered for Current Workspace

- Unique ID: GAP-0029
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 1
- Subsystem association: Build validation
- Description: Local Rust formatting, workspace compilation, tests, and clippy validation have current evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: High
- Dependency requirements: Rust stable toolchain with rustfmt and clippy.
- Exact future validation required: Run `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- Exact future tooling/environment required: Local development machine or CI runner with Rust stable.
- Recommended future agent type: Rust Implementation Agent
- Estimated production impact: The scaffold cannot be considered build-verified until this is complete.
- Completion criteria: All listed commands pass with no warnings or errors.
- Rollback considerations: Revert or patch Phase 1 scaffold files if validation fails.

## GAP-0030 — Dependency Audit Covered; Broader Supply-Chain Review Missing

- Unique ID: GAP-0030
- Phase association: Phase 1 / Phase 16 / Phase 17
- Subsystem association: Supply-chain security
- Description: `cargo audit` now runs locally/through GitHub Actions for the current workspace, and CycloneDX SBOM generation is gated in CI. Broader supply-chain review, license review, SBOM reviewer sign-off, provenance review, and release artifact attestation remain incomplete.
- Why incomplete: Dependency audit and SBOM generation evidence exists, but broader supply-chain governance and human/operator review remain incomplete.
- Why blocked in ChatGPT Project Mode: No longer blocked for the current `cargo audit`/SBOM generation gates; broader supply-chain review requires future operator review and release evidence workflow.
- Risk level: Medium
- Dependency requirements: Cargo, cargo-deny or equivalent, repository dependency manifest, dependency lockfile after external Cargo resolution.
- Exact future validation required: Run dependency audit and license policy checks for `serde`, `toml`, `serde_json`, `sha2`, and transitive dependencies after external Cargo resolution.
- Exact future tooling/environment required: Rust toolchain, `cargo-deny` or equivalent, CI runner.
- Recommended future agent type: AppSec Lead + DevSecOps Orchestrator
- Estimated production impact: Required before adding exchange, Web3, crypto, network, or database dependencies.
- Completion criteria: Dependency and license checks pass under documented policy.
- Rollback considerations: Remove or pin problematic dependencies and rerun validation.


## GAP-0031 — Phase 2 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0031
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 2
- Subsystem association: Config / secrets / mode gates
- Description: Phase 2 Rust modules and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: High
- Dependency requirements: Rust stable toolchain, Cargo, rustfmt, clippy, dependency download access.
- Exact future validation required: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Exact future tooling/environment required: Rust stable toolchain and internet/dependency cache access in local or CI environment.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future Phase 2 changes must rerun the validation commands.
- Rollback considerations: Revert Phase 2 config/secrets changes if validation exposes unrecoverable defects.

## GAP-0032 — Secret Zeroization and Lifecycle Hardening Missing

- Unique ID: GAP-0032
- Phase association: Phase 2 / Phase 3 / Phase 17
- Subsystem association: Secrets and custody
- Description: `SecretMaterial` redacts debug output, but zeroization, memory lifecycle hardening, and secret-use scoping are not implemented.
- Why incomplete: Phase 2 intentionally added only the smallest reference and redaction boundary.
- Why blocked in ChatGPT Project Mode: Code can be added later, but meaningful validation requires Rust tests, dependency review, and runtime inspection outside ChatGPT.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, dependency policy for zeroization crate, encrypted keystore implementation, signer boundary.
- Exact future validation required: zeroization tests, no-clone policy review where feasible, panic-path review, log/prompt/telemetry leak tests.
- Exact future tooling/environment required: Rust toolchain, memory/lifecycle test harness, AppSec review.
- Recommended future agent type: AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe live custody and wallet operation.
- Completion criteria: Secret material is minimized, redacted, zeroized on drop, scoped to constrained call sites, and covered by tests.
- Rollback considerations: Disable live modes and remove secret-loading paths if hardening cannot be validated.

## GAP-0033 — Phase 3 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0033
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 3
- Subsystem association: Policy engine / build system
- Description: Phase 3 policy code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for policy code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 3 policy files and docs if validation exposes unrecoverable defects.

## GAP-0034 — Policy Not Yet Mandatory in Execution Paths

- Unique ID: GAP-0034
- Phase association: Phase 3 / Phase 11
- Subsystem association: Policy engine / execution adapters
- Description: Policy engine and deterministic execution-adapter boundaries exist, but there is still no live runtime path proving real orders, swaps, transfers, withdrawals, and signing requests pass policy before external execution.
- Why incomplete: Execution adapters are model/trait boundaries only; live connectors, signer/custody, durable audit/state lifecycle, and external submission remain unavailable.
- Why blocked in ChatGPT Project Mode: Not blocked for future code, but real adapter behavior and live connector validation require external environments.
- Risk level: Critical
- Dependency requirements: Phase 4 audit journal, connector frameworks, execution planner, execution adapters.
- Exact future validation required: integration tests proving every execution path requires policy approval and fails closed on policy errors.
- Exact future tooling/environment required: Rust test runner, simulated adapters, later sandbox exchange/chain environments.
- Recommended future agent type: Execution Adapter Agent + AppSec Lead
- Estimated production impact: Blocks live execution and wallet signing.
- Completion criteria: No adapter can place an order, create a transaction, transfer funds, withdraw funds, or request signing without a policy-approved intent and audit record.
- Rollback considerations: Disable adapters and force Observe/Paper modes.

## GAP-0035 — Persistent Destination Allowlist Missing

- Unique ID: GAP-0035
- Phase association: Phase 3 / Phase 4 / Phase 8 / Phase 11
- Subsystem association: Policy engine / custody / state store
- Description: Phase 3 models destination trust classifications, but no persistent approved-address allowlist, address ownership proof, or operator approval workflow exists.
- Why incomplete: State store, signer boundary, and Web3 connector phases are not implemented yet.
- Why blocked in ChatGPT Project Mode: Real wallet address ownership, address-book storage, and transaction validation require external runtime tooling and operator actions.
- Risk level: Critical
- Dependency requirements: Audit journal, encrypted secret backend, signer boundary, Web3 connector framework, operator approval UX.
- Exact future validation required: destination allowlist tests, unknown-address denial tests, LLM-generated destination denial tests, address ownership verification where applicable, rollback tests.
- Exact future tooling/environment required: Rust test runner, local state store, test wallet, testnet RPCs, operator-controlled address book.
- Recommended future agent type: AppSec Lead + Web3 Connector Agent + Audit and Observability Agent
- Estimated production impact: Blocks any safe external transfer or withdrawal behavior.
- Completion criteria: External destinations can only be added through an auditable non-LLM-controlled workflow and are enforced by policy before signing or transfer.
- Rollback considerations: Clear destination allowlist, disable transfers/withdrawals, revoke any signer permissions.

## GAP-0036 — Phase 4 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0036
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 4
- Subsystem association: Audit journal / state store / build system
- Description: Phase 4 audit and state code plus tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for audit and state code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 4 audit/state files and docs if validation exposes unrecoverable defects.

## GAP-0037 — SQLite WAL State Store Implemented; Production Durability Validation Missing

- Unique ID: GAP-0037
- Phase association: Phase 4 / Phase 14 / Phase 17
- Subsystem association: State store / persistence
- Description: Phase 4 now includes `SqliteWalStateStore`, a local SQLite WAL-backed checkpoint store for non-secret operational state, plus round-trip, replacement, secret-like rejection, and reopen tests.
- Why incomplete: The store exists and has local Rust validation, but production durability still needs crash/recovery, file-locking, migration, backup/restore, runtime lifecycle wiring, and filesystem permission validation.
- Why blocked in ChatGPT Project Mode: Deeper crash testing, concurrent process behavior, backup/restore validation, filesystem permission checks, and runtime lifecycle validation require targeted local or CI runtime scenarios beyond unit tests.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, SQLite crate supply-chain review, schema migration plan, backup/restore plan, and runtime lifecycle integration.
- Exact future validation required: schema migration tests, crash-recovery tests, file-locking tests, concurrent access tests, backup/restore tests, and runtime checkpoint lifecycle tests.
- Exact future tooling/environment required: Rust toolchain, SQLite runtime, local filesystem, CI runner.
- Recommended future agent type: Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Removes the pure implementation gap for local SQLite checkpoint persistence, but production-grade restart recovery and reconciliation remain blocked until lifecycle wiring and durability tests are complete.
- Completion criteria: SQLite WAL-backed state store is wired into runtime checkpoints, persists and restores checkpoints safely across process restarts, and passes migration/recovery/locking/backup tests.
- Rollback considerations: Disable durable state features, fall back to observe/paper modes, and revert database migrations if validation fails.

## GAP-0038 — Audit Durability, Concurrency, and Filesystem Validation Missing

- Unique ID: GAP-0038
- Phase association: Phase 4 / Phase 17
- Subsystem association: Audit journal / runtime validation
- Description: The append-only JSONL audit journal has not been externally tested for crash recovery, concurrent append behavior, fsync guarantees, file permissions, rotation, retention, or disk-full behavior.
- Why incomplete: These validations require runtime filesystem behavior unavailable in ChatGPT Project Mode.
- Why blocked in ChatGPT Project Mode: Crash simulation, concurrent process tests, permission hardening, and disk-pressure tests require local/CI/system-level tooling.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, local filesystem test harness, future runtime supervisor design.
- Exact future validation required: crash during append, replay after partial write, concurrent writer denial or serialization, permission mode checks, retention/rotation checks, disk-full handling.
- Exact future tooling/environment required: Rust toolchain, local filesystem, process test harness, CI runner with filesystem controls.
- Recommended future agent type: Audit and Observability Agent + DevSecOps Orchestrator
- Estimated production impact: Blocks use of audit journal as production evidence for live-fund execution.
- Completion criteria: Audit journal behavior is deterministic and safe under crash, restart, permission, and disk-pressure scenarios.
- Rollback considerations: Disable live execution and require external log shipping or database-backed audit before re-enabling.

## GAP-0039 — Audit Not Yet Mandatory in Execution Paths

- Unique ID: GAP-0039
- Phase association: Phase 4 / Phase 10 / Phase 11
- Subsystem association: Audit journal / execution planner / execution adapters
- Description: Audit primitives, draft execution-planner records, and deterministic execution-adapter boundary records exist, but no connector, signer, or runtime adapter path proves every live-relevant action is durably journaled before and after action.
- Why incomplete: Phase 10 planner drafts and Phase 11 adapter records are current model boundaries only; they are not wired to durable audit/state preconditions, live connector submissions, signer requests, fills, failures, or reconciliation lifecycle records.
- Why blocked in ChatGPT Project Mode: Future local model wiring can be drafted here, but durable runtime validation plus real connector and signer validation require external environments.
- Risk level: Critical
- Dependency requirements: Phase 10 planner audit integration, Phase 11 execution adapters, Phase 8 signer boundary, durable audit/state validation.
- Exact future validation required: integration tests proving intents, policy decisions, connector submissions, fills, failures, signer requests, and reconciliations are audit-recorded and fail closed if audit append fails.
- Exact future tooling/environment required: Rust test runner, simulated connectors, future sandbox connectors, local audit journal or SQLite backend.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + AppSec Lead
- Estimated production impact: Blocks live execution and wallet signing.
- Completion criteria: No adapter can place an order, submit a swap, transfer funds, withdraw funds, or request signing unless pre-action audit append succeeds and post-action outcome is recorded.
- Rollback considerations: Disable adapters and force Observe/Paper modes if audit enforcement cannot be proven.



## GAP-0040 — Phase 5 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0040
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 5
- Subsystem association: Market data / fee model / build system
- Description: Phase 5 market-data and fee-model code plus tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for market-data and fee-model code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 5 market-data/fee files and docs if validation exposes unrecoverable defects.

## GAP-0041 — Live Market-Data Providers Not Implemented or Validated

- Unique ID: GAP-0041
- Phase association: Phase 5 / Phase 7 / Phase 8 / Phase 17
- Subsystem association: Market data / CEX connectors / DEX connectors / external integrations
- Description: Phase 5 added read-only provider trait boundaries and normalized models, but no live REST/WebSocket CEX provider, DEX quote provider, paid data-provider adapter, reconnect logic, or rate-limit logic exists.
- Why incomplete: The smallest safe Phase 5 patch created models and boundaries only; live providers require exchange/API review, network runtime, credentials where applicable, and external validation.
- Why blocked in ChatGPT Project Mode: Real network connections, provider accounts, API limits, WebSocket behavior, latency measurement, and data-quality validation require external runtime environments.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, provider selection, exchange/provider terms review, future connector framework, observability hooks, optional paid market-data credentials provisioned outside the repo.
- Exact future validation required: REST polling tests, WebSocket reconnect tests, stale-data tests, rate-limit tests, bad-data rejection tests, latency measurement, provider outage handling, no-secret-log tests, sandbox/read-only integration tests.
- Exact future tooling/environment required: Rust runtime, network access, test provider accounts or public endpoints, CI/integration environment, telemetry backend for latency and error metrics.
- Recommended future agent type: Market Data Connector Agent + DevSecOps Orchestrator + Audit and Observability Agent
- Estimated production impact: Blocks real opportunity discovery and production market-data confidence.
- Completion criteria: At least one reputable CEX read-only provider and one simulated provider pass validation with deterministic freshness, rate-limit, reconnect, and stale-data behavior.
- Rollback considerations: Disable live providers, force simulated/paper providers only, and revert connector-specific adapter code if validation fails.

## GAP-0042 — Fee Schedules Not Externally Verified

- Unique ID: GAP-0042
- Phase association: Phase 5 / Phase 7 / Phase 8 / Phase 17
- Subsystem association: Fee model / market data / execution planning
- Description: Phase 5 added fee schedules and fee-adjusted edge calculation, but fee rates, account tiers, gas/network costs, withdrawal costs, and venue-specific fee rules have not been externally verified.
- Why incomplete: Real fee data depends on venue account tier, jurisdiction, asset, pair, route, gas market, execution type, and provider-specific API behavior.
- Why blocked in ChatGPT Project Mode: Real exchange accounts, API access, chain RPCs, and live gas/fee observations are unavailable here.
- Risk level: High
- Dependency requirements: Read-only exchange credentials where needed, DEX/RPC quote providers, external fee schedule review, runtime observability, account-tier configuration.
- Exact future validation required: compare configured fees against venue API/account UI, validate maker/taker tiers, validate gas/network fee estimates, validate fee-adjusted edge against paper fills, reject unverified schedules for live execution.
- Exact future tooling/environment required: Rust runtime, exchange accounts, provider APIs, test wallets, chain RPCs, audit journal, simulated/paper connectors.
- Recommended future agent type: Market Data Connector Agent + Execution Planner Agent + AppSec Lead
- Estimated production impact: Incorrect fee estimates can turn apparent arbitrage into guaranteed loss; live execution must remain blocked until fee verification is enforced.
- Completion criteria: Fee schedules are externally verified, tagged with verification metadata, audited, and enforced by opportunity/planner/policy paths before live execution.
- Rollback considerations: Treat all fee schedules as unverified, disable live opportunities using them, and revert provider-specific fee integration if discrepancies are found.


## GAP-0043 — Phase 6 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0043
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 6
- Subsystem association: Paper connectors / build system
- Description: Phase 6 paper connector code plus tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for paper connector code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/paper.rs`, paper exports, CLI text, validator requirements, and Phase 6 governance updates if validation exposes unrecoverable defects.

## GAP-0044 — Paper Connector Model Is Not Production-Realistic

- Unique ID: GAP-0044
- Phase association: Phase 6 / Phase 9 / Phase 15
- Subsystem association: Paper execution / simulation / backtesting
- Description: Phase 6 provides deterministic in-memory market data, static fee schedules, and simple paper execution reports, but it does not model order-book depth consumption, partial fills, latency, queue position, adverse selection, balance constraints, position reconciliation, or exchange-specific matching behavior.
- Why incomplete: The smallest safe Phase 6 patch created testable boundaries only; realistic simulation belongs to later opportunity, backtesting, and testing phases.
- Why blocked in ChatGPT Project Mode: Realistic calibration requires historical market data, live venue behavior samples, latency observations, and Rust test execution unavailable here.
- Risk level: High
- Dependency requirements: Phase 9 opportunity engine, Phase 15 backtesting/scenario harness, external market-data samples, and current Rust validation baseline.
- Exact future validation required: scenario replay tests, depth-aware fill tests, partial-fill tests, latency/slippage tests, balance-ledger reconciliation tests, fee-model comparison tests, and paper-vs-sandbox discrepancy analysis.
- Exact future tooling/environment required: Rust test runner, historical data fixtures, sandbox exchange access where available, local SQLite or fixture store, CI runner.
- Recommended future agent type: Simulation/Backtesting Agent + Market Data Connector Agent + Rust Implementation Agent
- Estimated production impact: Paper profits may overstate achievable live results; live execution must remain blocked until realistic simulation and sandbox validation narrow the gap.
- Completion criteria: Paper execution models depth, slippage, latency, fees, balances, and partial fills with deterministic scenario tests and documented limitations.
- Rollback considerations: Force Observe mode or disable paper-derived strategy promotion if paper/live discrepancies exceed thresholds.

## GAP-0045 — Paper Execution Audit and State Integration Missing

- Unique ID: GAP-0045
- Phase association: Phase 6 / Phase 4 / Phase 10 / Phase 11
- Subsystem association: Paper execution / audit journal / state store
- Description: Phase 6 paper execution calls policy before producing a report, and the latest deterministic paper report can now be persisted through a typed local `StateStore` checkpoint helper with SQLite WAL reopen coverage. Paper execution still does not append audit records, reconcile balances, or fail closed through a complete runtime lifecycle.
- Why incomplete: Local checkpoint persistence exists for the latest report, but mandatory audit-before-action, audit-after-action, balance reconciliation, restart replay, and fail-closed runtime orchestration remain deferred.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the paper connector and local checkpoint boundary, but filesystem durability validation, audit replay validation, and full runtime lifecycle wiring remain incomplete.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 planner, Phase 11 adapter lifecycle, and current Rust validation baseline.
- Exact future validation required: paper intent audit-before-action test, paper report audit-after-action test, audit-fail-closed test, runtime state checkpoint orchestration test, restart/replay consistency test, and production durability persistence test.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, CI runner, audit replay harness.
- Recommended future agent type: Audit and Observability Agent + Execution Adapter Agent + Rust Implementation Agent
- Estimated production impact: Blocks using paper execution as auditable evidence for promotion toward live strategy controls.
- Completion criteria: Every paper execution intent and result is journaled, state is checkpointed, replay is deterministic, and execution fails closed when audit/state writes fail.
- Rollback considerations: Disable paper execution adapter use in strategy promotion and revert to read-only market-data simulation until audit/state integration is proven.


## GAP-0046 — Phase 7 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0046
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 7
- Subsystem association: CEX connector framework / build system
- Description: Phase 7 CEX framework code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for CEX framework code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/cex.rs`, remove exports, and return roadmap/gap tracker to Phase 6 state if validation exposes unrecoverable defects.

## GAP-0047 — Exchange-Specific CEX Adapters Not Implemented

- Unique ID: GAP-0047
- Phase association: Phase 7 / Phase 11 / Phase 17
- Subsystem association: CEX connectors / external integrations
- Description: Phase 7 defines CEX framework types and traits, but no exchange-specific REST, WebSocket, sandbox, balance, order, or cancel adapters exist.
- Why incomplete: The smallest safe Phase 7 patch created framework boundaries before exchange-specific implementations.
- Why blocked in ChatGPT Project Mode: Live exchange integration requires network access, exchange accounts, credentials, sandbox environments where available, and external API documentation validation.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, secret backend, rate-limit controller, audit/state integration, exchange account setup, sandbox access where available.
- Exact future validation required: public market-data tests, WebSocket reconnect tests, sandbox order lifecycle tests, cancel tests, balance read tests, idempotency tests, rate-limit tests, and failure-mode tests per exchange.
- Exact future tooling/environment required: Rust toolchain, network access, exchange sandbox or restricted test accounts, test credentials stored outside the repository, CI secrets, and mocked API fixtures.
- Recommended future agent type: CEX Connector Agent + AppSec Lead + DevSecOps Orchestrator
- Estimated production impact: Blocks live CEX arbitrage and real exchange order routing.
- Completion criteria: At least one exchange-specific connector passes read-only and sandbox validation without secret leakage and without live-funds permissions.
- Rollback considerations: Disable the affected connector, revoke test credentials, remove connector registration, and force Observe/Paper modes.

## GAP-0048 — CEX Fee, Rate-Limit, Terms, and Jurisdiction Validation Missing

- Unique ID: GAP-0048
- Phase association: Phase 7 / Phase 17
- Subsystem association: CEX governance / compliance / connector validation
- Description: CEX profiles include review flags, but no exchange-specific fee schedule, rate-limit, terms-of-service, incident-history, or jurisdiction review has been completed.
- Why incomplete: These checks require external sources, exchange documentation, legal/operational review, and often current account context.
- Why blocked in ChatGPT Project Mode: Real terms, account availability, jurisdiction constraints, fee tiers, and rate limits must be validated outside this environment and may change over time.
- Risk level: Critical
- Dependency requirements: Exchange selection, human operator jurisdiction, account tier, legal/tax review, up-to-date exchange documentation, and external review workflow.
- Exact future validation required: fee tier verification, withdrawal/transfer permission review, order-type support review, rate-limit validation, terms-of-service review, jurisdiction check, API-scope review, and incident/reputation review.
- Exact future tooling/environment required: Browser/network access, exchange accounts, legal/compliance review process, connector test harness, and credential-scope inspection.
- Recommended future agent type: CEX Connector Agent + AppSec Lead + Human Legal/Compliance Reviewer
- Estimated production impact: Blocks safe enablement of any real CEX venue.
- Completion criteria: Each enabled CEX profile records verified fees, rate limits, API capabilities, jurisdiction status, and terms review before use beyond paper/sandbox mode.
- Rollback considerations: Disable the CEX venue profile, remove it from allowlists, revoke credentials, and preserve audit records explaining disablement.

## GAP-0049 — CEX Framework Audit and State Integration Missing

- Unique ID: GAP-0049
- Phase association: Phase 7 / Phase 10 / Phase 11 / Phase 14
- Subsystem association: CEX execution lifecycle / audit journal / state store
- Description: CEX order requests can be policy-gated in Phase 7, but no durable audit-before-action, order lifecycle state machine, fill reconciliation, or restart recovery exists for CEX workflows.
- Why incomplete: Phase 7 framework models, Phase 10 planner records, and Phase 11 adapter boundary records now exist, but CEX order lifecycle handling still lacks durable audit/state gating, live or sandbox adapter wiring, fill reconciliation, and restart recovery.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the CEX/planner/adapter boundaries, but durable lifecycle validation requires filesystem/database persistence, simulated and sandbox exchange responses, and runtime restart tests.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 execution planner, Phase 11 adapters, and current Rust validation baseline.
- Exact future validation required: audit-before-order tests, audit-after-response tests, audit-fail-closed tests, order state transition tests, fill reconciliation tests, restart/replay tests, and duplicate client-order-id tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked exchange fixtures, sandbox exchange accounts, and CI runner.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks production-grade CEX order lifecycle management and post-incident forensic reliability.
- Completion criteria: Every CEX order lifecycle event is journaled, state transitions are durable and replayable, execution fails closed when audit/state writes fail, and restart recovery preserves idempotency.
- Rollback considerations: Disable CEX order submission, preserve existing audit files, roll back connector registration, and force Observe/Paper modes.

## GAP-0050 — Phase 8 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0050
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 8
- Subsystem association: DEX/Web3 connector framework / build system
- Description: Phase 8 DEX/Web3 framework code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for DEX/Web3 framework code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/dex.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 7 state if validation exposes unrecoverable defects.

## GAP-0051 — DEX/Web3 Protocol, Token, Gas, MEV, and Terms Validation Missing

- Unique ID: GAP-0051
- Phase association: Phase 8 / Phase 17
- Subsystem association: DEX/Web3 governance / protocol validation
- Description: Phase 8 profiles include framework metadata, but no router contract, token metadata, gas behavior, slippage behavior, MEV risk, protocol incident history, terms-of-service, or jurisdiction review has been completed.
- Why incomplete: These checks require external sources, protocol documentation, live/testnet behavior, legal/operational review, and frequently changing on-chain conditions.
- Why blocked in ChatGPT Project Mode: Real protocol and token validation require network access, chain explorers/RPC, legal/compliance review, and human operator jurisdiction context.
- Risk level: Critical
- Dependency requirements: Chain/router/token selection, external documentation review, chain explorer/RPC access, legal/tax review, and protocol risk workflow.
- Exact future validation required: router bytecode/address review, token contract review, decimals verification, spender allowlist verification, gas stress tests, slippage tests, MEV/sandwich risk review, protocol terms review, jurisdiction check, and incident/reputation review.
- Exact future tooling/environment required: Browser/network access, chain explorers, RPC endpoints, protocol docs, legal/compliance review process, and Web3 test harness.
- Recommended future agent type: Web3 Connector Agent + AppSec Lead + Human Legal/Compliance Reviewer
- Estimated production impact: Blocks safe enablement of any real DEX/router/token profile.
- Completion criteria: Each enabled DEX/Web3 profile records verified chain, router, spender, token, gas, slippage, MEV, jurisdiction, and protocol-risk status before use beyond paper/simulation mode.
- Rollback considerations: Disable the DEX/router/token profile, remove it from allowlists, revoke provider credentials, and preserve audit records explaining disablement.

## GAP-0052 — DEX/Web3 Framework Audit and State Integration Missing

- Unique ID: GAP-0052
- Phase association: Phase 8 / Phase 10 / Phase 11 / Phase 14
- Subsystem association: DEX/Web3 execution lifecycle / audit journal / state store
- Description: DEX swap quote requests can be policy-gated in Phase 8, but no durable audit-before-action, transaction lifecycle state machine, nonce tracking, simulation replay, confirmation tracking, or restart recovery exists for DEX/Web3 workflows.
- Why incomplete: Phase 8 framework models, Phase 10 planner records, and Phase 11 adapter boundary records now exist, but DEX/Web3 lifecycle handling still lacks durable audit/state gating, live RPC adapters, signer boundary implementation, nonce/confirmation tracking, and restart recovery.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the DEX/planner/adapter boundaries, but durable lifecycle validation requires filesystem/database persistence, mocked RPC fixtures, testnet responses, signer harnesses, and runtime restart tests.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 8 framework validation, Phase 10 execution planner, Phase 11 adapters, signer boundary, and current Rust validation baseline.
- Exact future validation required: audit-before-simulation tests, audit-before-signing tests, audit-fail-closed tests, nonce/state transition tests, transaction confirmation tests, simulation replay tests, restart/recovery tests, and duplicate intent-id tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked RPC fixtures, testnet RPC endpoints, signer test harness, and CI runner.
- Recommended future agent type: Execution Adapter Agent + Web3 Connector Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks production-grade DEX/Web3 lifecycle management and post-incident forensic reliability.
- Completion criteria: Every DEX/Web3 lifecycle event is journaled, state transitions are durable and replayable, signing/broadcast fails closed when audit/state writes fail, and restart recovery preserves idempotency.
- Rollback considerations: Disable DEX/Web3 execution, preserve existing audit files, roll back router/token registration, and force Observe/Paper modes.



## GAP-0053 — Phase 9 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0053
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 9
- Subsystem association: Opportunity engine / build system
- Description: Phase 9 opportunity-engine code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for opportunity-engine code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/opportunity.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 8 state if validation exposes unrecoverable defects.

## GAP-0054 — Opportunity Engine Audit, State, and Planner Integration Missing

- Unique ID: GAP-0054
- Phase association: Phase 9 / Phase 10 / Phase 14 / Phase 15
- Subsystem association: Opportunity engine / audit journal / state store / execution planner
- Description: Opportunity candidates are deterministic data records in Phase 9 and can now be consumed by the Phase 10 draft planner, but they are not durably journaled, checkpointed, replayed, or integrated into a runtime lifecycle.
- Why incomplete: Phase 9 intentionally stopped before runtime lifecycle integration; Phase 10 added draft planner consumption but not durable audit/state persistence.
- Why blocked in ChatGPT Project Mode: Durable lifecycle validation requires Rust tests, filesystem/database persistence, replay fixtures, planner implementation, and runtime restart tests.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 9 and Phase 10 validation baselines, and Phase 15 backtesting/scenario harness.
- Exact future validation required: opportunity audit-record tests, replay determinism tests, candidate deduplication tests, state checkpoint tests, planner handoff tests, restart/recovery tests, and historical backtest replay.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, fixture market data, CI runner.
- Recommended future agent type: Strategy/Backtesting Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks reliable production traceability and safe handoff from discovery to planning.
- Completion criteria: Every discovered candidate is journaled without secrets, replayable, deduplicated, state-checkpointed, and consumed by the planner only after durable validation.
- Rollback considerations: Disable opportunity-engine runtime integration, preserve audit files, revert planner handoff wiring, and force Observe/Paper modes.

## GAP-0055 — Phase 10 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0055
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 10
- Subsystem association: Execution planner / build system
- Description: Phase 10 execution-planner code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for execution-planner code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/planner.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 9 state if validation exposes unrecoverable defects.

## GAP-0056 — Execution Planner Audit, State, and Adapter Handoff Integration Missing

- Unique ID: GAP-0056
- Phase association: Phase 10 / Phase 11 / Phase 14 / Phase 15
- Subsystem association: Execution planner / audit journal / state store / execution adapters
- Description: Execution-plan drafts contain deterministic intents, sequencing steps, policy outcomes, and failure-mode metadata, and can now be consumed by the Phase 11 adapter-boundary model, but they are not durably journaled, checkpointed, replayed, or handed to real execution adapters.
- Why incomplete: Phase 10 intentionally implemented only the planner model boundary, and Phase 11 added only deterministic adapter-boundary records without runtime orchestration or durable audit/state gating.
- Why blocked in ChatGPT Project Mode: Durable lifecycle validation requires Rust tests, filesystem/database persistence, mocked adapters, restart fixtures, and CI/runtime execution.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 validation baseline, Phase 11 execution adapters, and Phase 15 scenario/backtesting harness.
- Exact future validation required: plan audit-record tests, preflight replay tests, duplicate plan-id tests, state checkpoint tests, adapter handoff tests, fail-closed audit-write tests, restart/recovery tests, and historical scenario replay.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked adapter fixtures, CI runner.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks safe production handoff from planning to execution.
- Completion criteria: Every plan and policy outcome is journaled without secrets, replayable, state-checkpointed, and adapters reject plans unless audit/state preconditions pass.
- Rollback considerations: Disable planner-to-adapter handoff, preserve audit files, revert adapter wiring, and force Observe/Paper modes.

## GAP-0057 — Phase 11 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0057
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 11
- Subsystem association: Execution adapter framework / build system
- Description: Phase 11 execution-adapter framework code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for execution-adapter framework code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/execution_adapter.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 10 state if validation exposes unrecoverable defects.

## GAP-0058 — Execution Adapter Audit, State, and Live Submission Integration Missing

- Unique ID: GAP-0058
- Phase association: Phase 11 / Phase 14 / Phase 15 / Phase 17
- Subsystem association: Execution adapters / audit journal / state store / live connectors
- Description: Execution-adapter framework records model attempts, fills, and reconciliation outcomes, but they are not durably journaled, checkpointed, replayed across restarts, connected to real paper balance ledgers, or connected to live exchange/RPC adapters.
- Why incomplete: Phase 11 intentionally implemented only deterministic model/trait boundaries with external submission disabled.
- Why blocked in ChatGPT Project Mode: Durable lifecycle validation requires Rust tests, filesystem/database persistence, mocked and sandbox adapters, network access, restart fixtures, and CI/runtime execution.
- Risk level: Critical
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 planner validation, Phase 11 validation baseline, exchange-specific CEX adapters, DEX/RPC adapters, signer boundary, and Phase 15 scenario harness.
- Exact future validation required: audit-before-adapter tests, state checkpoint tests, duplicate submission prevention tests, modeled fill replay tests, reconciliation replay tests, crash/restart tests, sandbox adapter tests, kill-switch tests, live-scope denial tests, no-broadcast-until-approved tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked adapter fixtures, sandbox exchange accounts, testnet RPC, CI runner.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe transition from model-only planning to real execution.
- Completion criteria: Every adapter run is durably journaled and state-checkpointed before any external submission, can be replayed/reconciled after restart, and live adapters cannot submit without policy, audit, state, and kill-switch approval.
- Rollback considerations: Disable planner-to-adapter handoff, preserve audit files, revoke credentials, remove live adapter registrations, and force Observe/Paper modes.

## GAP-0059 — Phase 12 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0059
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 12
- Subsystem association: Communications and CLI / build system
- Description: Phase 12 communications/CLI code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for communications and CLI code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/communications.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 11 state if validation exposes unrecoverable defects.

## GAP-0060 — Communications Audit, Auth, and Runtime Lifecycle Integration Missing

- Unique ID: GAP-0060
- Phase association: Phase 12 / Phase 14 / Phase 15 / Phase 17
- Subsystem association: Communications / audit journal / state store / runtime control
- Description: Phase 12 communication records are deterministic local models only. They are not durably journaled, authenticated, authorized, replayed, rate-limited against real channels, or integrated into a runtime operator-control lifecycle.
- Why incomplete: Phase 12 intentionally implements only model/trait boundaries and disables outbound integrations.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the communications boundary, but durable lifecycle validation, persistence, runtime orchestration, real channel authentication, platform accounts, network access, and external security review remain missing.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 12 validation baseline, authentication/authorization model, communications channel adapters, Phase 14 observability, and Phase 15 scenario tests.
- Exact future validation required: command audit-record tests, notification audit-record tests, replay determinism tests, operator authorization tests, command injection tests, no-secret-dispatch tests, rate-limit tests, channel outage tests, and restart/recovery tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked channel adapters, platform test accounts, CI runner.
- Recommended future agent type: Communications Integration Agent + Audit and Observability Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe remote operator control and production alerting.
- Completion criteria: Every command and notification is authenticated where remote, authorized, redacted, durably journaled, replayable, rate-limited, and fail-closed without enabling direct live execution.
- Rollback considerations: Disable remote channel adapters, revoke tokens, preserve audit records, and fall back to local CLI status-only operation.



## GAP-0061 — Phase 13 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0061
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 13
- Subsystem association: Embedded dashboard / build system
- Description: Phase 13 embedded-dashboard code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for embedded-dashboard code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/dashboard.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 12 state if validation exposes unrecoverable defects.

## GAP-0062 — Dashboard Hosting, Auth, and Runtime Lifecycle Integration Missing

- Unique ID: GAP-0062
- Phase association: Phase 13 / Phase 14 / Phase 15 / Phase 17
- Subsystem association: Embedded dashboard / authentication / audit journal / state store / runtime control
- Description: Phase 13 dashboard records are deterministic local models only. No HTTP server, browser delivery, authentication, authorization, CSRF protection, rate limiting, durable state, audit lifecycle, or penetration-tested hosting exists.
- Why incomplete: Phase 13 intentionally implements only model/trait boundaries and rejects server startup, public exposure, live controls, and secret rendering.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the dashboard boundary, but secure hosting validation requires runtime orchestration, local browser/server testing, network binding inspection, authentication design, persistence, and external security review.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 13 validation baseline, authentication/authorization model, secure local web host design, Phase 14 observability, Phase 15 scenario tests, and Phase 17 security review.
- Exact future validation required: loopback binding tests, public-bind denial tests, auth-required tests, CSRF tests, clickjacking/header tests, no-secret-render tests, live-control denial tests, command-injection tests, audit-record tests, restart/recovery tests, and penetration testing.
- Exact future tooling/environment required: Rust test runner, local browser/server harness, temporary filesystem, SQLite WAL backend, mocked runtime snapshots, CI runner, and AppSec review workflow.
- Recommended future agent type: Embedded Dashboard Agent + Audit and Observability Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe operator dashboard use beyond local in-process render records.
- Completion criteria: Dashboard hosting is loopback by default, authenticated where exposed, authorized, CSRF-protected, rate-limited, audited, redacted, fail-closed, and unable to trigger live execution without policy/audit/state approval.
- Rollback considerations: Disable dashboard hosting, preserve audit records, remove dashboard route registration, and fall back to CLI status-only operation.


## GAP-0063 — Phase 14 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0063
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 14
- Subsystem association: Observability and runbooks / build system
- Description: Phase 14 observability/runbook code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for observability/runbook code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/observability.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 13 state if validation exposes unrecoverable defects.

## GAP-0064 — Observability Runtime, Exporter, Alert, and Audit Lifecycle Integration Missing

- Unique ID: GAP-0064
- Phase association: Phase 14 / Phase 15 / Phase 17
- Subsystem association: Observability / runbooks / audit journal / state store / communications
- Description: Phase 14 observability records are deterministic local models only. No tracing subscriber, metrics endpoint, OpenTelemetry/Prometheus exporter, log shipping, alert routing, durable state, audit lifecycle, retention policy, or incident-drill validation exists.
- Why incomplete: Phase 14 intentionally implements only model/trait boundaries and disables metrics endpoints, public exposure, outbound alerts, and secret observability.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the observability/runbook boundary, but runtime lifecycle validation, alert/exporter validation, persistence, network binding inspection, mocked or real observability stacks, communication channel adapters, and external security review remain missing.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 12 communications validation, Phase 14 validation baseline, secure metrics endpoint design, Phase 15 scenario tests, and Phase 17 security review.
- Exact future validation required: tracing subscriber tests, redaction tests, no-secret-telemetry tests, metrics endpoint loopback tests, public-bind denial tests, exporter tests, log retention tests, alert routing tests, incident runbook drills, audit-record tests, restart/recovery tests, and panic/failure capture tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked observability/exporter fixtures, mocked communications channels, local runtime, CI runner, and AppSec review workflow.
- Recommended future agent type: Audit and Observability Agent + Communications Integration Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks production-grade operations, incident response, and safe deployment monitoring.
- Completion criteria: Observability runtime emits redacted structured logs, health status, metrics, and critical alerts through authenticated, audited, fail-closed paths without exposing secrets or enabling live execution controls.
- Rollback considerations: Disable metrics endpoints, exporters, log shipping, and alert adapters; preserve local records and audit evidence; fall back to CLI/dashboard local status only.


## GAP-0065 — Phase 15 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0065
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 15
- Subsystem association: Testing, fuzzing, and backtesting / build system
- Description: Phase 15 testing/fuzzing/backtesting boundary code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + Testing, Fuzzing, and Backtesting Agent
- Estimated production impact: Current compile/test/lint confidence for testing/backtesting boundary code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/testing.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 14 state if validation exposes unrecoverable defects.

## GAP-0066 — Validation Runner, Fuzzing Engine, Corpus, and Backtest Execution Missing

- Unique ID: GAP-0066
- Phase association: Phase 15 / Phase 17
- Subsystem association: Testing / fuzzing / backtesting / CI / runtime validation
- Description: Phase 15 validation records are deterministic local models only. No actual property-test framework integration, external fuzzing engine execution, curated fuzz corpus, deterministic market replay runner, backtest execution engine, load testing, penetration testing, CI gate execution, or production validation run exists.
- Why incomplete: Phase 15 intentionally implements only model/trait boundaries and disables external fuzzer invocation, live network tests, live execution, credential-bearing fixtures, signing, and broadcasts.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo and CI evidence exists for the validation-plan boundary, but real runner execution requires fixture files, fuzzing dependencies, replay datasets, temporary filesystems/databases, security tooling, and external runtime environments.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 4 audit/state validation, Phase 5 market data validation, Phase 6 paper validation, Phase 9 opportunity validation, Phase 10 planner validation, Phase 11 adapter validation, Phase 14 observability validation, curated fixtures, and CI runner.
- Exact future validation required: unit tests, integration tests, property tests, fuzz tests, deny-path tests, audit replay tests, deterministic backtest replay tests, scenario regression tests, load tests, rollback tests, incident-drill tests, and penetration tests.
- Exact future tooling/environment required: Rust test runner, property-test crate, fuzzing engine, fixture corpus, temporary filesystem, SQLite WAL backend, mocked CEX/DEX/RPC fixtures, CI runner, load-test tooling, and AppSec review workflow.
- Recommended future agent type: Testing, Fuzzing, and Backtesting Agent + AppSec Lead + Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Blocks evidence-based confidence in safety, strategy correctness, replay determinism, and production readiness.
- Completion criteria: Validation plans execute in CI/local environments with deterministic fixture corpora, fuzz/property test coverage, replay/backtest records, load/security validation evidence, and no secret leakage or live side effects.
- Rollback considerations: Disable advanced runners, preserve fixtures and audit evidence, revert failing harness integrations, and keep Observe/Paper modes only until validation is restored.

## GAP-0067 — Phase 16 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0067
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 16
- Subsystem association: Packaging/deployment / build system
- Description: Phase 16 packaging/deployment boundary code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + Release Engineering Authority
- Estimated production impact: Current compile/test/lint confidence for packaging/deployment boundary code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/packaging.rs`, remove exports, remove CLI status text, remove deployment templates, and return roadmap/gap tracker to Phase 15 state if validation exposes unrecoverable defects.

## GAP-0068 — Phase 16 Packaging, Deployment, and Rollback Execution Missing

- Unique ID: GAP-0068
- Phase association: Phase 16 / Phase 17
- Subsystem association: Packaging / deployment / release engineering / operations
- Description: Phase 16 package and deployment records are deterministic local models only. Current evidence exists for Rust validation, locked release-build validation, dependency audit, SBOM generation, an example-only container image build, CI Trivy example image-scan evidence, and hardening artifact indexing. The first CI image scan exposed fixable critical Debian slim runtime findings that were patched by moving the example runtime to nonroot distroless Debian 12. Production validation is still missing: no production release artifact was built, no production container image was validated, no systemd unit was installed, no ARM binary was produced, no runtime deployment occurred, no rollback drill was executed, and no production release was validated.
- Why incomplete: Current non-secret CI and example-container evidence improves packaging feedback, but Phase 16 intentionally avoids external side effects, service installation, public exposure, embedded secrets, live trading enablement, and production claims.
- Why blocked in ChatGPT Project Mode: Real deployment validation requires Rust tooling, container/systemd/ARM infrastructure, target hosts, filesystem controls, release artifact storage, rollback environment, security tooling, and operator credentials handled outside the repo.
- Risk level: High
- Dependency requirements: Keep current Rust/CI, locked release-build, SBOM-generation, dependency-audit, local-SARIF, example-image, image-scan, secret-scan, and hardening-index evidence refreshable; add a production packaging target, systemd/Linux validation host, ARM validation target, signed release workflow, rollback procedure, observability integration, and incident runbooks.
- Exact future validation required: Refresh release-build, SBOM, dependency-audit, SAST, image-scan, secret-scan, and hardening-index evidence for the candidate commit, then execute production release artifact build, production container build, image scan review, SBOM review, service hardening validation, non-root runtime test, read-only filesystem test, health check test, config loading test, log/audit redaction test, rollback drill, incident drill, startup/shutdown soak test, and production readiness review.
- Exact future tooling/environment required: Current Rust/Cargo and CI runner for evidence refresh; container runtime, Linux systemd host, ARM target or verified cross target, SAST/dependency tools, artifact repository, staging environment, and human operator review for production validation.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority + AppSec Lead
- Estimated production impact: Example container build and scanner feedback improved the template, but deployable, operable, rollback-safe production release remains blocked.
- Completion criteria: Release artifacts are built, scanned, deployed to a staging target, validated under hardened runtime settings, rolled back successfully, and documented without secret leakage or live-funds exposure.
- Rollback considerations: Remove generated artifacts, disable services, restore previous configuration and binary, preserve logs/audit evidence, revoke any accidentally exposed credentials, and keep live execution disabled.



## GAP-0069 — Phase 17 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0069
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 17
- Subsystem association: External hardening evidence boundary / build system
- Description: Phase 17 external hardening boundary Rust/Cargo validation was previously unexecuted, but the current workspace has since passed local and GitHub Actions format, check, test, and clippy validation.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external hardening gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable toolchain, Cargo, rustfmt, clippy, CI runner, and repository checkout available for future validation runs.
- Exact future validation required: Re-run `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, CI runner.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + AppSec Lead
- Estimated production impact: Compile/test/lint confidence for the hardening evidence boundary is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/hardening.rs`, remove exports, remove CLI status text, remove hardening docs, and return roadmap/gap tracker to Phase 16 state if validation exposes unrecoverable defects.

## GAP-0070 — External Production Hardening Execution Missing

- Unique ID: GAP-0070
- Phase association: Phase 17 / Phase 18
- Subsystem association: External hardening / AppSec / release engineering / operations / compliance
- Description: Phase 17 added deterministic evidence models, release blockers, review records, and operator checklists. GitHub Actions CI, locked release-build validation, dependency audit, CycloneDX SBOM generation, local-SARIF CodeQL SAST, local example-only container build, CI Trivy example image-scan evidence, Gitleaks secret-pattern scan evidence, and hardening evidence indexing are passing as of the 2026-05-26 run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602`. The first CI Trivy image-scan gate correctly failed on fixable critical Debian slim runtime findings and preserved non-secret evidence; a distroless runtime repair then passed follow-up CI validation. Production validation is still missing: SBOM review, GitHub code scanning upload processing, production container validation, systemd hardening validation, ARM validation, staging deployment, load test, penetration test, rollback drill, incident-response drill, live exchange/RPC validation, and production readiness review have not been executed.
- Why incomplete: Current CI evidence exists for release-build, dependency-audit, SBOM-generation, local-SARIF SAST, local example-container build, image-scan, secret-scan, and hardening-index gates. Phase 17 intentionally avoids credentials, public exposure, live funds, production claims, live exchange/RPC calls, signing, broadcasts, and deployment execution; the broader external hardening checklist remains incomplete until production-context reviews and drills are performed.
- Why blocked in ChatGPT Project Mode: Real production hardening requires external CI, runtime infrastructure, staging hosts, security tooling, target devices, controlled credentials outside the repo, human review, and accountable operator approval.
- Risk level: Critical
- Dependency requirements: Keep current CI, release-build, dependency-audit, SBOM-generation, local-SARIF SAST, example image-scan, secret-scan, and hardening-index evidence refreshable; add SBOM reviewer workflow, GitHub code scanning upload processing or accepted deferral, staging environment, observability runtime, incident runbooks, rollback procedure, AppSec review, exchange/RPC sandbox environments, custody/signer design, and compliance review.
- Exact future validation required: Refresh CI, release-build, SBOM-generation, dependency-audit, SAST, image-scan, secret-scan, and hardening-index evidence for the candidate commit, then complete SBOM review, GitHub code scanning upload processing if enabled, service hardening test, read-only filesystem test, config loading test, log/audit redaction test, staging deployment, startup/shutdown/restart tests, load and soak tests, penetration test, rollback drill, incident-response drill, exchange sandbox validation, DEX/RPC sandbox validation without broadcasts, key custody review, and production readiness review.
- Exact future tooling/environment required: Current Rust/Cargo, CI runner, SAST/dependency tools, SBOM generator, and container scanner for evidence refresh; Linux/systemd host, ARM target or cross-build runner, staging host, observability stack, load-test tooling, security testing workflow, sandbox exchange/RPC accounts, and human operator review for production validation.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority + AppSec Lead + Audit and Observability Agent + Compliance Reviewer
- Estimated production impact: CI, locked release-build, dependency-audit, SBOM-generation, local-SARIF SAST, SAST artifact retention, local example-container build, image-scan failure evidence, and secret-pattern scan evidence improved hardening feedback, but the remaining missing hardening evidence still blocks any credible production-readiness, public-service, live-funds, or autonomous-execution claim.
- Completion criteria: External hardening evidence is generated, reviewed, non-secret, linked from an external evidence store, and confirms all required production gates pass without enabling live funds prematurely.
- Rollback considerations: Preserve evidence, disable candidate services, restore prior artifact/configuration, revoke any exposed credentials, keep Observe/Paper modes only, and return to a known validated checkpoint.

## GAP-0071 — Phase 18 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0071
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 18
- Subsystem association: Agentic handoff package boundary / build system
- Description: Phase 18 handoff boundary code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + Handoff Agent
- Estimated production impact: Current compile/test/lint confidence for the handoff package boundary is improved, but this does not prove production readiness or external-agent execution success.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/handoff.rs`, remove exports, remove CLI status text, remove `handoff/` docs, and return roadmap/gap tracker to Phase 17 state if validation exposes unrecoverable defects.

## GAP-0072 — External Agentic Handoff Execution and Evidence Review Missing

- Unique ID: GAP-0072
- Phase association: Phase 18
- Subsystem association: Agentic handoff / external validation / human review
- Description: Phase 18 added deterministic handoff records, prompts, and checklists, and current repository validation plus non-secret CI evidence references exist for the handoff baseline. Production handoff validation is still missing: no external coding agent, DevSecOps agent, AppSec reviewer, compliance reviewer, or human production reviewer has consumed the package and produced independent non-secret review evidence.
- Why incomplete: Phase 18 intentionally creates local documentation/model boundaries only and does not invoke external agents, production reviewers, infrastructure, or services.
- Why blocked in ChatGPT Project Mode: Real handoff execution requires external repositories or IDEs, CI systems, agent runtimes, human reviewers, non-secret evidence storage, and accountable approval workflows outside this chat.
- Risk level: Medium
- Dependency requirements: Latest repository checkout or ZIP, complete governance files, current local/CI validation references, non-secret evidence workflow, DevSecOps/AppSec review process, and human maintainer approval.
- Exact future validation required: Refresh repository validation and CI evidence for the handoff candidate, run future-agent prompts against the latest repo, verify agents read governance files, verify no gaps are dropped, verify no secrets are requested or stored, verify all generated changes preserve policy gates and live-funds blockers, and record independent non-secret evidence references.
- Exact future tooling/environment required: External coding-agent runtime or IDE, Git repository, CI runner, issue tracker or evidence store, and human reviewer; no live credentials, signing keys, broadcasts, withdrawals, or real exchange/RPC calls are required for handoff validation.
- Recommended future agent type: Handoff Agent + DevSecOps Orchestrator + AppSec Lead + Human Maintainer
- Estimated production impact: Blocks reliable multi-agent continuation and accountable production-readiness review, but does not by itself block local model/documentation completeness.
- Completion criteria: External agents or human reviewers consume the handoff package, generate non-secret evidence, preserve all governance constraints, and update the gap tracker without claiming production readiness prematurely.
- Rollback considerations: Revert unsafe handoff-generated changes, discard secret-bearing artifacts if any appear, preserve the last validated ZIP, and resume from the latest known-good governance checkpoint.

## GAP-0073 - 2026-05-19 Local Toolchain Validation Resolved Locally

- Unique ID: GAP-0073
- Phase association: Phase 18 / post-handoff validation
- Subsystem association: Local validation toolchain / build system
- Description: The requested local validation sequence was attempted for ArbyClaw on 2026-05-19 and was initially blocked because `python3`, Cargo, and `rustc` were unavailable on PATH. On 2026-05-20, after the local toolchain was made available, the requested structure, format, compile, test, and clippy validation sequence completed successfully.
- Why incomplete: No longer incomplete for the requested local validation sequence. Production hardening, CI, deployment, live integration, security, load, rollback, incident, and external validation gaps remain tracked separately.
- Why blocked in ChatGPT/Codex environment: No longer blocked for the requested local validation sequence.
- Risk level: Low for local compile/test/lint evidence; high production and live-funds risks remain tracked in the other gap entries.
- Dependency requirements: Continue to keep Python 3, Rust stable, Cargo, rustfmt, clippy, and rustc available on PATH for future validation runs.
- Exact future validation required: Re-run `python3 scripts/validate_structure.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` after future changes.
- Exact future tooling/environment required: Python 3 launcher named `python3`, Rust stable toolchain, Cargo, rustfmt, clippy, rustc, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Local compile/test/lint confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met on 2026-05-20 for the requested local validation sequence.
- Rollback considerations: If a future validation run fails, keep live execution disabled, patch only the failing source-level issue, and update this tracker with the new failure evidence.

## GAP-0074 - 2026-05-20 CI Validation Repository Target Resolved

- Unique ID: GAP-0074
- Phase association: Phase 18 / post-handoff CI validation
- Subsystem association: CI/CD / GitHub Actions / repository publishing
- Description: The requested CI validation sequence was initially blocked because no repository target was visible. On 2026-05-20 the new GitHub repository `dominator509/arbyclaw` was connected, the ArbyClaw checkpoint was pushed to `main`, and GitHub Actions CI passed. The latest recorded validated run is `26443625602` for commit `0b98a9a31d3701704d950779ad989daefcf1193b`, including checkout via `actions/checkout@v6`, the original validation sequence, `cargo build --release --locked`, `cargo audit`, CycloneDX SBOM generation with non-empty file checks, CodeQL Rust SAST local-SARIF generation with non-empty SARIF verification, short-retention SARIF artifact upload, example image scan, secret-pattern scan, hardening evidence index generation, and GitHub Step Summary evidence pointers.
- Why incomplete: No longer incomplete for initial GitHub Actions CI execution. Production hardening, deployment, live integration, security, load, rollback, incident, and external validation gaps remain tracked separately.
- Why blocked in ChatGPT/Codex environment: No longer blocked for initial GitHub Actions CI execution after the repository target was created and connected.
- Risk level: Low for initial CI compile/test/lint evidence; high production and live-funds risks remain tracked in the other gap entries.
- Dependency requirements: Keep `dominator509/arbyclaw` connected to this workspace, keep GitHub Actions enabled, and keep Rust stable, Cargo, rustfmt, clippy, and Python 3 available in CI.
- Exact future validation required: Re-run the GitHub Actions CI workflow containing `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release --locked`, `cargo audit`, CycloneDX SBOM generation, CodeQL Rust SAST local-SARIF generation, short-retention SARIF artifact retention, and `python3 scripts/validate_structure.py` after future changes; record non-secret run URLs and pass/fail results when they are material to release gating.
- Exact future tooling/environment required: Git checkout, configured remote, GitHub repository with Actions enabled, branch or pull request, workflow runner, Rust stable toolchain, Cargo, rustfmt, clippy, cargo-audit, cargo-cyclonedx, CodeQL Action availability, `actions/upload-artifact`, and Python 3.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority
- Estimated production impact: External CI confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met on 2026-05-20 for initial GitHub Actions CI execution on `main`.
- Rollback considerations: If future CI setup changes are unsafe or point at the wrong repository, disable the incorrect workflow/branch, preserve local and CI validation evidence, and reconnect the workspace to the correct repository before retrying.

## GAP-0075 - GitHub Code Scanning Upload Not Enabled

- Unique ID: GAP-0075
- Phase association: Phase 17 / Phase 18 / post-handoff hardening evidence
- Subsystem association: SAST / GitHub Actions / GitHub code scanning
- Description: CodeQL Rust SAST now runs in GitHub Actions, verifies non-empty local SARIF output, and keeps that SARIF available as a short-retention Actions artifact. An earlier upload-based CodeQL run failed because GitHub code scanning is not enabled for the repository, and a later GitHub API default-setup enablement attempt returned the same code-scanning-disabled response. On 2026-05-24 the operator chose to keep `dominator509/arbyclaw` private and rely on the CI `codeql-sarif-evidence` artifact as the local SARIF-only SAST evidence path for now, so SARIF upload to the GitHub Security tab remains intentionally deferred.
- Why incomplete: The repository-level code scanning feature is disabled or unavailable and the operator has not approved making the repository public or enabling paid/private-repository code-scanning support, so CodeQL evidence is limited to local SARIF generation and artifact retention inside CI rather than GitHub code scanning alert processing.
- Why blocked in ChatGPT/Codex environment: Enabling repository code scanning may require an external repository settings change, plan/support entitlement, or explicit owner decision outside safe local source-code changes.
- Risk level: Medium
- Dependency requirements: Repository owner review of GitHub code scanning availability, repository settings access, any required GitHub security entitlement, and a successful upload-capable CodeQL run after the feature is enabled.
- Exact future validation required: Enable GitHub code scanning if appropriate, remove or revise `upload: never`, run CodeQL with SARIF upload enabled, and confirm the GitHub Security/code-scanning alert processing completes without secret leakage or production-readiness claims.
- Exact future tooling/environment required: GitHub repository settings access, GitHub Actions, CodeQL Action, code scanning feature availability, and a non-secret evidence reference.
- Recommended future agent type: DevSecOps Orchestrator + AppSec Lead
- Estimated production impact: Local SAST evidence and evidence retention are improved, but centralized GitHub code scanning alert processing and review evidence remain incomplete.
- Completion criteria: CodeQL results are uploaded to and processed by GitHub code scanning, or the documented local SARIF-only governance decision remains accepted for this private repository and release reviewers continue to record GAP-0075 as open/deferred.
- Rollback considerations: Keep local SARIF-only SAST gate if upload remains unavailable; if upload is enabled and later fails, revert to the last passing local-SARIF gate and record the failure evidence.
