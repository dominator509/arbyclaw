# Release Review Evidence Template

This template is for recording non-secret release-review evidence references. It is not proof of production readiness by itself.

## Review Metadata

- Review date:
- Reviewer:
- Reviewer role:
- Reviewer independence/conflict note:
- Release evidence review window:
- Reviewer decision rationale category:
- Review scope:
- Repository:
- Branch:
- Commit:
- Workflow run URL:
- Local SARIF-only operator decision:
- Review outcome:
- Evidence status:
- Evidence references-only attestation:
- Evidence reviewer sign-off:
- Evidence sign-off scope:
- Evidence sign-off unresolved gaps:
- Evidence rejection reason category:
- Evidence rejection required follow-up:
- Evidence rejection follow-up owner:
- Evidence rejection follow-up target review date:
- External evidence store reference:
- Artifact retention decision:
- Evidence retention location classification:
- Artifact review-by or expiration date:
- Evidence refresh required:
- Evidence refresh reason:
- Operator CI artifact review outcome:
- Dependency audit gate reference:
- Dependency audit review decision:
- SBOM generation artifact reference:
- SBOM review decision:
- SBOM review outcome:
- Image scan artifact reference:
- Image scan review decision:

## Evidence References

Record only non-secret references or sanitized artifact names.

| Evidence item | Reference | Review note |
|---|---|---|
| CI workflow run |  |  |
| `hardening-evidence-index` artifact |  |  |
| `codeql-sarif-evidence` artifact |  |  |
| `trivy-image-scan-evidence` artifact |  |  |
| `gitleaks-secret-scan-evidence` artifact |  |  |
| Docker Buildx build record artifact |  |  |
| Dependency audit gate |  |  |
| Dependency audit gate reference |  |  |
| Dependency audit review decision |  |  |
| SBOM generation gate |  |  |
| SBOM generation artifact reference |  |  |
| SBOM review |  |  |
| SBOM review decision |  |  |
| Image scan artifact reference |  |  |
| Image scan review decision |  |  |
| GitHub code-scanning settings review |  |  |
| GitHub code-scanning processing result or local-SARIF-only decision |  |  |
| Local SARIF-only operator decision review |  |  |
| Retained artifact review |  |  |
| Operator CI artifact review |  |  |
| Evidence expiration or refresh review |  |  |
| Evidence retention location classification |  |  |
| Evidence references-only attestation |  |  |
| Release evidence reviewer sign-off |  |  |
| Evidence reviewer role review |  |  |
| Evidence reviewer independence/conflict note review |  |  |
| Release evidence review-window review |  |  |
| Reviewer decision rationale category review |  |  |
| Evidence status legend review |  |  |
| Evidence rejection reason review |  |  |
| Evidence rejection follow-up assignment |  |  |

## Required Checks

- Confirm the workflow run belongs to the expected repository, branch, and commit.
- Confirm all CI jobs required for the review completed with the expected result.
- Confirm downloaded evidence artifacts are non-empty.
- Confirm retained evidence is limited to non-secret CI outputs.
- Confirm retained artifacts were reviewed against `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` before external storage.
- Confirm evidence retention location classification is limited to `Actions artifact`, `approved external evidence store`, `unavailable`, or `deferred`.
- Confirm the operator CI artifact review checklist in `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` was used when recording artifact names, reviewer, outcome, and unresolved gaps.
- Confirm evidence expiration and refresh triggers in `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` were checked before relying on retained CI evidence.
- Confirm evidence status uses only the release evidence status legend from `hardening/EXTERNAL_VALIDATION_RUNBOOK.md`.
- Confirm the reviewer attests that evidence records contain references only and no embedded artifact contents, logs, SARIF/SBOM contents, vulnerability tables, screenshots, credentials, private URLs, wallet material, or sensitive environment details.
- Confirm reviewer role is limited to `operator`, `release reviewer`, `AppSec reviewer`, `DevSecOps reviewer`, or `deferred`.
- Confirm reviewer independence/conflict note is limited to `independent`, `same operator`, `deferred`, or `not applicable` and does not imply production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- Confirm release evidence review window is limited to `current`, `expired`, `deferred`, or `not applicable` and does not imply production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- Confirm reviewer decision rationale category is limited to `sufficient`, `insufficient`, `stale`, `deferred`, or `not applicable` and does not include artifact contents, logs, SARIF/SBOM contents, vulnerability tables, credentials, private URLs, wallet material, sensitive environment details, or production-readiness claims.
- Confirm release evidence reviewer sign-off records evidence acceptance, rejection, or follow-up only and does not claim production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- Confirm rejected or incomplete evidence uses only non-secret rejection reason categories and required follow-up from `hardening/EXTERNAL_VALIDATION_RUNBOOK.md`.
- Confirm rejected or incomplete evidence follow-up owner and target review date are non-secret routing metadata only and do not imply evidence acceptance, production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- Confirm dependency audit gate reference is limited to non-secret run URLs, job names, gate names, commit hashes, reviewer, review date, outcome, and unresolved gaps; do not copy advisory tables, dependency details, vulnerable package lists, CVE text, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into this record.
- Confirm dependency audit review decision is limited to `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable` and does not imply production readiness, live-funds readiness, public exposure readiness, deployment readiness, custody readiness, or compliance approval.
- Confirm SBOM evidence was reviewed against `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and only non-secret references are recorded here.
- Confirm SBOM generation artifact reference is limited to non-secret run URLs, job names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps; do not copy dependency graphs, package inventories, vulnerability tables, SBOM contents, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into this record.
- Confirm SBOM review decision is limited to `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable` and does not imply production readiness, live-funds readiness, public exposure readiness, deployment readiness, custody readiness, or compliance approval.
- Confirm Trivy image-scan evidence was reviewed against `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and only non-secret references are recorded here.
- Confirm image scan artifact reference is limited to non-secret run URLs, job names, gate names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps; do not copy vulnerability tables, image layer details, package inventories, CVE text, base-image metadata beyond sanitized artifact references, private registry URLs, internal hostnames, credentials, wallet material, screenshots, or raw sensitive logs into this record.
- Confirm image scan review decision is limited to `accepted`, `rejected`, `follow-up required`, `deferred`, or `not applicable` and does not imply production container readiness, production readiness, live-funds readiness, public exposure readiness, deployment readiness, custody readiness, or compliance approval.
- Confirm GitHub code-scanning upload processing is either validated through repository settings, workflow run URL, and Security-tab processing result, or explicitly recorded as unavailable/deferred with GAP-0075 still open.
- Confirm any local SARIF-only operator decision preserves repository privacy, relies only on the `codeql-sarif-evidence` Actions artifact as non-secret SAST evidence, keeps GAP-0075 open/deferred for GitHub Security-tab upload processing, and does not imply production readiness, live-funds readiness, public exposure readiness, deployment readiness, custody readiness, or compliance approval.
- Confirm no credentials, wallet material, private URLs, raw sensitive logs, or secret-bearing screenshots are copied into this record.
- Confirm unresolved gaps remain listed below and are not treated as complete.

## Unresolved Gaps

List every release-relevant gap that remains open.

| Gap | Status | Required follow-up |
|---|---|---|
| SBOM review |  |  |
| GitHub code-scanning upload processing |  |  |
| Production container validation |  |  |
| Systemd or ARM validation |  |  |
| Staging validation |  |  |
| Load or soak testing |  |  |
| Penetration testing |  |  |
| Rollback or incident drill |  |  |
| Custody or signer review |  |  |
| Compliance review |  |  |
| Production readiness review |  |  |

## Explicit Non-Claims

This record does not validate production deployment, live funds, public exposure, signing, broadcasts, withdrawals, bridges, live exchange/RPC calls, custody readiness, compliance approval, or production readiness unless those validations were actually completed and referenced through non-secret evidence.
