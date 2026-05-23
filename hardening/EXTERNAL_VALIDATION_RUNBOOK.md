# External Validation Runbook

This runbook is a checklist for external operators or future coding agents. It is not evidence that validation has run.

## Required External Validation Before Production Claims

1. Run all Rust validation commands in a clean checkout.
2. Build release artifacts with locked dependencies.
3. Run dependency audit and SBOM review.
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

## Retained Artifact Review Checklist

Before retaining CI artifacts outside GitHub Actions:

1. Confirm the artifact came from the expected repository, branch, workflow run URL, and commit.
2. Confirm the artifact name matches the run summary and release-review evidence record.
3. Confirm the artifact is non-empty and was produced by a successful job.
4. Confirm the artifact does not contain credentials, wallet material, private URLs, raw sensitive logs, unredacted SARIF excerpts, or secret-bearing screenshots.
5. Store only sanitized artifacts or non-secret references in the approved external evidence store.
6. Record the retention location as a non-secret reference, not as embedded evidence content.
7. Record the reviewer, review date, retention decision, expiration or review-by date, and unresolved gaps.
8. Delete any secret-bearing or wrongly scoped artifact from the retention path and record the issue without copying the sensitive content.

## Hard Stop Conditions

- Any secret appears in repository files, logs, telemetry, prompts, tickets, or evidence.
- Any service binds publicly before authentication, authorization, TLS, rate limits, and AppSec review are complete.
- Any live exchange, RPC, signer, withdrawal, bridge, or broadcast capability is enabled before policy, audit, simulation, and custody reviews pass.
- Any production-readiness claim lacks external evidence.
