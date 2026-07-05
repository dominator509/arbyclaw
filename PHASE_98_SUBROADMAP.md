# Phase 98 - Deployment Checklist Lifecycle and Drill Plan Requirement Expansion

## Scope

- Require existing local lifecycle, runtime-plan, retention-preflight, rollback-drill-plan, incident-drill-plan, and service-manager rehearsal components in the deployment evidence checklist.
- Keep the checklist non-mutating and reference-only.
- Do not install services, start/stop/restart services, mutate deployment paths, load secrets, call networks, send alerts, enable live execution, or claim production readiness.

## Implementation

- Expanded `scripts/validate_deployment_evidence_checklist.py` required bundle components to include:
  - `systemd-lifecycle-plan`
  - `deployment-host-runtime-plan`
  - `deployment-host-retention-preflight`
  - `rollback-drill-plan`
  - `incident-response-drill-plan`
  - `service-manager-lifecycle-rehearsal`
- This makes the checklist fail closed if existing local lifecycle/drill planning and rehearsal components disappear from the bundle.

## Validation

Required local validation for this phase:

```text
python scripts/validate_deployment_evidence_bundle.py --json
python scripts/validate_deployment_evidence_checklist.py --json
python scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Remaining Blockers

- Real service-manager lifecycle execution remains external.
- Executed rollback and incident-response drills remain external.
- Real deployment-host runtime validation remains external.

## Completion Note

The deployment evidence checklist now requires the existing local lifecycle/drill planning and rehearsal components without changing runtime behavior or claiming external production evidence.
