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

## Manual Evidence Review Checklist

Before using CI evidence in any future release review:

1. Confirm the workflow run belongs to the expected repository, branch, and commit.
2. Confirm the `rust-validation`, `codeql-sast`, `example-container-image-scan`, `secret-pattern-scan`, and `hardening-evidence-index` jobs completed successfully.
3. Confirm the downloaded `hardening-evidence-index` artifact matches the workflow run URL and commit under review.
4. Confirm retained evidence artifacts are non-empty and remain limited to non-secret CI outputs.
5. Confirm any redacted secret-scan evidence is reviewed without copying secrets, tokens, private URLs, wallet material, or sensitive logs into tickets, Markdown, prompts, or release notes.
6. Record only non-secret artifact names, run URLs, commit hashes, reviewer identity, and review outcome in release documentation.
7. Keep unresolved gaps open for missing SBOM review, GitHub code-scanning upload processing, production image validation, systemd or ARM validation, staging, load, penetration, rollback, incident, custody, compliance, and production-readiness review.

## Hard Stop Conditions

- Any secret appears in repository files, logs, telemetry, prompts, tickets, or evidence.
- Any service binds publicly before authentication, authorization, TLS, rate limits, and AppSec review are complete.
- Any live exchange, RPC, signer, withdrawal, bridge, or broadcast capability is enabled before policy, audit, simulation, and custody reviews pass.
- Any production-readiness claim lacks external evidence.
