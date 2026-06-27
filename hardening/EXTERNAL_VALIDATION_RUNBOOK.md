# External Validation Runbook

This runbook is a checklist for external operators or future coding agents. It is not evidence that validation has run.

## Required External Validation Before Production Claims

1. Run all Rust validation commands in a clean checkout.
2. Build release artifacts with locked dependencies.
3. Run dependency audit, dependency license policy review, and SBOM review.
4. Build and scan any container image in an approved environment.
5. Validate systemd/service hardening on a Linux host.
6. Validate ARM target build/runtime behavior where applicable.
7. Deploy only to a staging environment with no live funds.
8. Run audit replay, restart/recovery, redaction, and config-loading checks.
9. Run load, soak, and failure-injection tests.
10. Run penetration testing and AppSec review.
11. Execute rollback and incident-response drills.
12. Preserve non-secret evidence references in an external evidence store.

## CI Evidence Lookup

Recent CI hardening evidence is easiest to find from the GitHub Actions run page:

- Open the latest `ci` workflow run for `main`.
- Review the `hardening-evidence-index` job summary for the run URL, commit, producing job results, artifact names, and explicit non-claims.
- Download short-retention artifacts from the run page when needed: `hardening-evidence-index`, `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and any Docker Buildx build record artifact.
- Treat these artifacts as CI evidence only. They do not prove production deployment, live-funds readiness, public exposure readiness, SBOM review, GitHub code-scanning upload processing, penetration testing, load testing, rollback drills, incident drills, custody review, compliance review, or production readiness.
- If evidence must be retained beyond the GitHub Actions retention window, store only non-secret references or sanitized artifacts in an approved external evidence store.

## GitHub Code Scanning Evidence Path

GitHub code-scanning upload processing is an external repository setting and must be reviewed outside source-code changes:

1. Open the repository settings for `dominator509/arbyclaw`.
2. Review `Settings` -> `Code security and analysis` -> `Code scanning`.
3. If code scanning is available and approved, enable CodeQL/code-scanning support for the repository.
4. Update the CI CodeQL step in a reviewed follow-up so SARIF upload is enabled, then run the `ci` workflow on the expected branch and commit.
5. Confirm the CodeQL job completes and the GitHub `Security` -> `Code scanning` page processes the uploaded results for the same commit.
6. Record only non-secret evidence: repository, branch, commit, workflow run URL, CodeQL job result, code-scanning page URL or settings-review note, reviewer, outcome, and unresolved gaps.
7. If code scanning is unavailable, blocked by repository settings, or not approved, record that decision in release-review evidence and keep GAP-0075 open while relying on local SARIF artifact evidence only.

Current operator decision: keep `dominator509/arbyclaw` private and use the CI `codeql-sarif-evidence` artifact as the non-secret SAST evidence path for now. This decision preserves repository privacy and does not validate GitHub Security-tab code-scanning upload processing, production readiness, live-funds readiness, public exposure readiness, deployment readiness, custody readiness, or compliance approval.

Do not copy SARIF contents, source findings with sensitive context, private repository settings screenshots, credentials, tokens, or secret-bearing logs into Markdown, tickets, prompts, or release notes.

## Manual Evidence Review Checklist

Before using CI evidence in any future release review:

1. Confirm the workflow run belongs to the expected repository, branch, and commit.
2. Confirm the `rust-validation`, `codeql-sast`, `example-container-image-scan`, `secret-pattern-scan`, and `hardening-evidence-index` jobs completed successfully.
3. Confirm the downloaded `hardening-evidence-index` artifact matches the workflow run URL and commit under review.
4. Confirm retained evidence artifacts are non-empty and remain limited to non-secret CI outputs.
5. Confirm any redacted secret-scan evidence is reviewed without copying secrets, tokens, private URLs, wallet material, or sensitive logs into tickets, Markdown, prompts, or release notes.
6. Record only non-secret artifact names, run URLs, commit hashes, reviewer identity, and review outcome in release documentation.
7. Keep unresolved gaps open for missing SBOM review, GitHub code-scanning upload processing, production image validation, systemd or ARM validation, staging, load, penetration, rollback, incident, custody, compliance, and production-readiness review.

## Operator CI Artifact Review Checklist

Operators can use this lightweight checklist when reviewing retained CI artifacts without changing runtime behavior:

1. Record the repository, branch, commit, workflow run URL, reviewer, review date, and review outcome.
2. Record the artifact names exactly as shown by the workflow run: `hardening-evidence-index`, `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and any Docker Buildx build record artifact.
3. Confirm each artifact was produced by the expected successful job and is non-empty.
4. Confirm the review used the artifact contents only in the approved external evidence workflow.
5. Record unresolved gaps by identifier or short name, including SBOM review, GitHub code-scanning upload processing, production image validation, systemd or ARM validation, staging, load, penetration, rollback, incident, custody, compliance, and production-readiness review.
6. Do not copy SARIF contents, SBOM contents, vulnerability tables, secret-scan details, raw logs, private URLs, credentials, tokens, wallet material, screenshots, or sensitive environment details into repository Markdown, prompts, tickets, or release notes.
7. Treat the checklist as review routing evidence only; it does not prove production readiness, live-funds readiness, public exposure readiness, or deployment readiness.

## Evidence Expiration and Refresh

Before any release review, refresh CI evidence references when any of these are true:

1. The referenced workflow run no longer matches the commit, branch, or repository under review.
2. The referenced GitHub Actions artifacts have expired or are no longer downloadable from the run page.
3. The source tree, workflow file, dependency lockfile, container example, hardening runbook, release template, or security policy changed after the referenced run.
4. The reviewer cannot verify the artifact names, job outcomes, retention decision, or unresolved gaps from non-secret references.
5. The release review is using evidence older than the operator-approved review window.

To refresh, rerun the `ci` workflow on the exact commit under review, confirm the expected jobs and artifacts, and record only non-secret run URLs, artifact names, reviewer, outcome, review date, refresh reason, and unresolved gaps. Do not copy artifact contents, logs, credentials, private URLs, wallet material, or screenshots into repository files.

When recording a release evidence refresh trigger outcome, use only these non-secret values: `commit changed`, `artifact expired`, `workflow changed`, `reviewer unable to verify`, or `deferred`. The refresh trigger outcome is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

## Per-Artifact Expiration Outcomes

When reviewing retained CI evidence by artifact type, record only one of these non-secret expiration outcomes: `current`, `expired`, `unavailable`, or `refreshed`.

Apply the outcome separately to `hardening-evidence-index`, `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and any Docker Buildx build record artifact. These outcome fields are routing metadata only; they must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

## Release Evidence Reviewer Sign-Off

Use reviewer sign-off only to record whether the non-secret evidence package is accepted, rejected, or needs follow-up:

1. Record reviewer name or approved handle, reviewer role, reviewer independence/conflict note, review date, release evidence review window, reviewer decision rationale category, evidence scope, outcome, and unresolved gaps.
2. Record approval as evidence acceptance only; it must not claim production readiness, live-funds approval, public exposure approval, deployment readiness, compliance approval, custody readiness, signing readiness, bridge readiness, broadcast readiness, or exchange/RPC readiness.
3. If evidence is rejected or incomplete, record the non-secret reason and required follow-up without copying logs, artifact contents, credentials, private URLs, wallet material, screenshots, or sensitive environment details.
4. Attest that release evidence records contain non-secret references only, not embedded artifact contents, logs, SARIF/SBOM contents, vulnerability tables, screenshots, credentials, private URLs, wallet material, or sensitive environment details.
5. Use only these non-secret reviewer role values when a role is recorded: `operator`, `release reviewer`, `AppSec reviewer`, `DevSecOps reviewer`, or `deferred`.
6. Use only these non-secret reviewer independence/conflict note values when recorded: `independent`, `same operator`, `deferred`, or `not applicable`.
7. Use only these non-secret release evidence review-window values when recorded: `current`, `expired`, `deferred`, or `not applicable`.
8. Use only these non-secret reviewer decision rationale category values when recorded: `sufficient`, `insufficient`, `stale`, `deferred`, or `not applicable`.
9. Require a fresh sign-off whenever evidence is refreshed, the reviewed commit changes, or release-relevant gaps change.

## Release Evidence Status Legend

Use only these non-secret status values in release evidence records:

1. Accepted: evidence reference is in scope for the review, but this does not claim production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
2. Rejected: evidence is out of scope, failed, unsafe to retain, or otherwise unusable; record a non-secret reason category and follow-up.
3. Follow-up required: evidence may become usable after a non-secret action item is completed and reviewed.
4. Expired: artifact, run, retention window, or review window is no longer valid for the release under review.
5. Deferred: evidence requires external settings, infrastructure, reviewer approval, or validation that has not occurred.

## Release Evidence Rejection Reasons

When evidence is rejected or marked incomplete, record only non-secret reason categories and required follow-up:

1. Scope mismatch: repository, branch, commit, workflow run, artifact, or reviewer scope does not match the release under review.
2. Missing or expired evidence: expected job result, artifact, retention record, SBOM review, code-scanning setting review, or sign-off is unavailable.
3. Failed gate: CI job, dependency audit, dependency license policy validation, SBOM generation, SAST, image scan, secret scan, or structure validation failed.
4. Secret-handling concern: evidence may contain credentials, private URLs, wallet material, raw sensitive logs, screenshots, or unredacted findings.
5. Unresolved blocker: open gaps still require production image validation, systemd or ARM validation, staging, load, penetration, rollback, incident, custody, compliance, or production-readiness review.
6. Review-process issue: reviewer identity, review date, retention decision, refresh reason, or unresolved-gap list is missing or inconsistent.

Record follow-up as a non-secret action item with a non-secret owner or approved handle and target review date when assignment is needed. Assignment records are routing metadata only; they must not imply evidence acceptance, production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval. Do not copy raw logs, SARIF/SBOM contents, vulnerability tables, secret-scan details, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.

When recording reviewer follow-up status, use only these non-secret values: `open`, `assigned`, `resolved`, `deferred`, or `not applicable`. Follow-up status is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up closure reason, use only these non-secret values: `evidence refreshed`, `gap accepted for deferral`, `superseded by newer run`, `unable to verify`, or `not applicable`. Follow-up closure reason is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up verification method, use only these non-secret values: `CI run review`, `artifact reference review`, `external evidence store review`, `not verified`, or `not applicable`. Follow-up verification method is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up verification timestamp, record only the non-secret review date/time, preferably in UTC ISO 8601 format. The timestamp is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up evidence source, use only these non-secret values: `GitHub Actions`, `external evidence store`, `local operator record`, `not available`, or `not applicable`. Follow-up evidence source is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up evidence source locator, record only a non-secret run URL, artifact name, or approved external reference. The source locator is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up locator availability, use only these non-secret values: `available`, `expired`, `inaccessible`, `deferred`, or `not applicable`. Locator availability is routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up locator access note, use only short sanitized notes such as `artifact retained`, `artifact expired`, `permissions unavailable`, `external reference reviewed`, or `not applicable`. Locator access notes are routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

When recording reviewer follow-up locator recheck date, record only the non-secret date or date/time when the locator should be reviewed again, preferably in UTC ISO 8601 format. Locator recheck dates are routing metadata only and must not embed artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.

The release evidence reviewer follow-up metadata set is intentionally considered complete enough for the current Phase 17 / Phase 18 roadmap boundary. Do not add more reviewer micro-fields unless a concrete operator review workflow identifies missing non-secret routing metadata; prefer returning to roadmap-aligned production-gap audit and external validation closure work.

## Retained Artifact Review Checklist

Before retaining CI artifacts outside GitHub Actions:

1. Confirm the artifact came from the expected repository, branch, workflow run URL, and commit.
2. Confirm the artifact name matches the run summary and release-review evidence record.
3. Confirm the artifact is non-empty and was produced by a successful job.
4. Confirm the artifact does not contain credentials, wallet material, private URLs, raw sensitive logs, unredacted SARIF excerpts, or secret-bearing screenshots.
5. Store only sanitized artifacts or non-secret references in the approved external evidence store.
6. Record the retention location as a non-secret reference and classify it as `Actions artifact`, `approved external evidence store`, `unavailable`, or `deferred`, not as embedded evidence content.
7. Record the reviewer, review date, retention decision, expiration or review-by date, and unresolved gaps.
8. Delete any secret-bearing or wrongly scoped artifact from the retention path and record the issue without copying the sensitive content.

Retention location classification is routing metadata only. `Actions artifact` means the evidence remains in GitHub Actions short-retention artifacts, `approved external evidence store` means a sanitized external evidence reference exists, `unavailable` means no retained non-secret evidence is currently available, and `deferred` means retention requires later external approval or infrastructure.

## Dependency Audit And License Review Checklist

Before treating `cargo audit` and dependency license policy evidence as release-review input:

1. Confirm the dependency audit gate came from the expected repository, branch, workflow run URL, commit, and `rust-validation` job.
2. Confirm the `cargo audit` CI step completed successfully or record the failed gate as non-secret reference metadata only.
3. Confirm the dependency license policy gate completed successfully for the locked Cargo graph or record the failed gate as non-secret reference metadata only.
4. Record only the run URL, job name, gate name, commit hash, reviewer, review date, outcome, and unresolved follow-up gaps in release-review documentation.
5. Record the dependency-audit or license-policy review decision as non-secret reference metadata only: `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable`.
6. Do not copy advisory tables, dependency details, package inventories, license texts, CVE text, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into repository files.
7. Keep production-readiness, deployment-readiness, and live-funds claims blocked until dependency-audit and license-policy review are combined with the rest of the external hardening gates.

## SBOM Review Checklist

Before treating generated SBOM evidence as release-review input:

1. Confirm the SBOM artifact came from the expected repository, branch, workflow run URL, and commit.
2. Confirm the SBOM generation job completed successfully and produced non-empty SBOM output.
3. Confirm the SBOM artifact is reviewed in an approved external evidence workflow rather than copied into repository Markdown.
4. Confirm dependency names, versions, licenses, and package URLs are reviewed for unexpected packages, unsupported licenses, and stale or risky dependencies.
5. Confirm any vulnerability findings are correlated with the dependency-audit result and recorded as non-secret references only.
6. Confirm no environment variables, private registry URLs with credentials, internal hostnames, tokens, wallet material, or raw sensitive logs appear in retained SBOM evidence.
7. Record only the run URL, artifact name, reviewer, review date, outcome, and unresolved follow-up gaps in release-review documentation.
8. Record the SBOM review decision as non-secret reference metadata only: `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable`.
9. Do not copy dependency graphs, package inventories, vulnerability tables, SBOM contents, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into repository files.
10. Keep production-readiness, deployment-readiness, and live-funds claims blocked until SBOM review is combined with the rest of the external hardening gates.

## Image Scan Review Checklist

Before treating Trivy image-scan evidence as release-review input:

1. Confirm the image-scan artifact came from the expected repository, branch, workflow run URL, commit, and `example-container-image-scan` job.
2. Confirm the Trivy evidence artifact is named `trivy-image-scan-evidence` and was produced by the expected example image-scan gate.
3. Confirm the fixable critical vulnerability enforcement gate completed successfully or record the failed gate as non-secret reference metadata only.
4. Record only the run URL, job name, gate name, artifact name, commit hash, reviewer, review date, outcome, and unresolved follow-up gaps in release-review documentation.
5. Record the image-scan review decision as non-secret reference metadata only: `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable`.
6. Do not copy vulnerability tables, image layer details, package inventories, CVE text, base-image metadata beyond sanitized artifact references, private registry URLs, internal hostnames, credentials, wallet material, screenshots, or raw sensitive logs into repository files.
7. Keep production container readiness, deployment readiness, production readiness, and live-funds claims blocked until image-scan review is combined with the rest of the external hardening gates.

## Secret Scan Review Checklist

Before treating Gitleaks secret-scan evidence as release-review input:

1. Confirm the secret-scan artifact came from the expected repository, branch, workflow run URL, commit, and `secret-pattern-scan` job.
2. Confirm the Gitleaks evidence artifact is named `gitleaks-secret-scan-evidence` and was produced by the expected redacted secret-pattern scan gate.
3. Confirm the secret-pattern scan gate completed successfully or record the failed gate as non-secret reference metadata only.
4. Record only the run URL, job name, gate name, artifact name, commit hash, reviewer, review date, outcome, and unresolved follow-up gaps in release-review documentation.
5. Record the secret-scan review decision as non-secret reference metadata only: `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable`.
6. Do not copy secret-scan findings, secret-like snippets, match strings, file excerpts, private URLs, credentials, tokens, wallet material, screenshots, raw logs, or sensitive environment details into repository files.
7. Keep secret-handling approval, deployment readiness, production readiness, and live-funds claims blocked until secret-scan review is combined with the rest of the external hardening gates.

## Hard Stop Conditions

- Any secret appears in repository files, logs, telemetry, prompts, tickets, or evidence.
- Any service binds publicly before authentication, authorization, TLS, rate limits, and AppSec review are complete.
- Any live exchange, RPC, signer, withdrawal, bridge, or broadcast capability is enabled before policy, audit, simulation, and custody reviews pass.
- Any production-readiness claim lacks external evidence.
