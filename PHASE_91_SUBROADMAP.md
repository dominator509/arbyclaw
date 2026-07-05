## Phase 91 - Deployment Evidence Bundle Static Hardening Component Gate

### Goal

Expose the Phase 90 deployment-host static hardening/config smoke runtime evidence as a direct deployment evidence bundle component and require it in the deployment evidence checklist. This keeps CI/handoff evidence aligned with the runtime gate without service-manager actions, deployment mutation, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Add `deployment-host-static-hardening-config-smoke` to `scripts/validate_deployment_evidence_bundle.py`.
- Require the new bundle component in `scripts/validate_deployment_evidence_checklist.py`.
- Keep the component local-only and bounded through `scripts/validate_deployment_host_runtime.py --run-deployment-static-hardening --json`.
- Isolate deployment evidence bundle workspaces per process and pass a run-scoped workspace to the nested deployment-runtime gate so concurrent bundle/checklist runs cannot delete each other's local validation state.
- Update governance docs and gap tracker after validation.

### Explicit Non-Goals

- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
- No production config mutation or deployment-host path mutation.
- No public network exposure.
- No external calls, live execution, signing, broadcast, exchange call, RPC call, wallet custody, or production-readiness approval.

### Validation

```bash
python3 scripts/validate_deployment_evidence_bundle.py --json
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Exit Criteria

Met when the deployment evidence bundle and checklist fail closed unless the Phase 90 static hardening/config smoke deployment-host wrapper evidence is present and passing.
