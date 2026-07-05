## Phase 93 - Deployment Evidence Bundle Filesystem Transcript Component Gate

### Goal

Expose deployment disk-full, retention/rotation, and permission-denial transcript validators as direct deployment evidence bundle components and require them in the deployment evidence checklist. This makes filesystem-related deployment blockers visible in CI/handoff evidence without performing disk filling, retention deletion, permission changes, service-manager actions, external calls, live execution, or production-readiness claims.

### Completed Tasks

- Add `deployment-disk-full-transcript`, `deployment-retention-transcript`, and `deployment-permission-transcript` to `scripts/validate_deployment_evidence_bundle.py`.
- Require those three components in `scripts/validate_deployment_evidence_checklist.py`.
- Update governance docs and gap tracker after validation.

### Explicit Non-Goals

- No physical disk filling.
- No retention rotation or deletion against production logs.
- No permission changes.
- No systemd install, reload, start, stop, restart, enable, or daemon deployment.
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

Met when the deployment evidence bundle and checklist fail closed unless deployment disk-full, retention/rotation, and permission-denial transcript evidence is present and passing.
