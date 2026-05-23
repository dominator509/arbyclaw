# Release Review Evidence Template

This template is for recording non-secret release-review evidence references. It is not proof of production readiness by itself.

## Review Metadata

- Review date:
- Reviewer:
- Review scope:
- Repository:
- Branch:
- Commit:
- Workflow run URL:
- Review outcome:
- External evidence store reference:
- Artifact retention decision:
- Artifact review-by or expiration date:
- Evidence refresh required:
- Evidence refresh reason:
- Operator CI artifact review outcome:
- SBOM review outcome:

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
| SBOM generation gate |  |  |
| SBOM review |  |  |
| GitHub code-scanning settings review |  |  |
| GitHub code-scanning processing result or local-SARIF-only decision |  |  |
| Retained artifact review |  |  |
| Operator CI artifact review |  |  |
| Evidence expiration or refresh review |  |  |

## Required Checks

- Confirm the workflow run belongs to the expected repository, branch, and commit.
- Confirm all CI jobs required for the review completed with the expected result.
- Confirm downloaded evidence artifacts are non-empty.
- Confirm retained evidence is limited to non-secret CI outputs.
- Confirm retained artifacts were reviewed against `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` before external storage.
- Confirm the operator CI artifact review checklist in `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` was used when recording artifact names, reviewer, outcome, and unresolved gaps.
- Confirm evidence expiration and refresh triggers in `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` were checked before relying on retained CI evidence.
- Confirm SBOM evidence was reviewed against `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and only non-secret references are recorded here.
- Confirm GitHub code-scanning upload processing is either validated through repository settings, workflow run URL, and Security-tab processing result, or explicitly recorded as unavailable/deferred with GAP-0075 still open.
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
