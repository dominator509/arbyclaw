# Deployment Notes

Phase 16 adds deployment documentation templates only. These files do not prove that a build, container run, service install, cloud deployment, rollback drill, or production release was performed.

## Safety Defaults

- Live trading must remain disabled.
- Runtime mode must remain observe or paper unless future phases complete custody, signer, connector, audit, policy, and external validation gates.
- Do not place secrets in this repository, Markdown, container build context, systemd units, shell history, logs, or generated artifacts.
- Do not expose dashboard, metrics, command, or control surfaces publicly.
- Treat all container, systemd, ARM, CI, and release steps as externally validated only after they are executed in a capable environment.

## Required External Validation Before Deployment Claims

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
python3 scripts/validate_systemd_example.py
```

`scripts/validate_systemd_example.py` checks the committed example unit only. On Linux hosts with `systemd-analyze`, `python3 scripts/validate_systemd_example.py --systemd-analyze` also runs syntax verification against a temporary fake root without installing, enabling, reloading, or starting a service. Passing it is not production service-manager validation.

`scripts/validate_systemd_lifecycle.py` adds a manual deployment-host lifecycle evidence helper. Its default mode is a non-mutating plan:

```bash
python3 scripts/validate_systemd_lifecycle.py
```

On a Linux deployment host where the unit already exists, inspect mode can collect read-only `systemctl show` state without installing, enabling, reloading, starting, stopping, or restarting the service:

```bash
python3 scripts/validate_systemd_lifecycle.py --mode inspect --unit arb-agent.service --json
```

Inspect output is reference evidence only. Operator-controlled service start, graceful shutdown, restart, recovery, rollback, and deployment-host audit/SQLite validation still require separate non-secret evidence before any deployment claim.

`scripts/validate_deployment_host_runtime.py` composes the lifecycle helper with the existing local runtime smoke runner. Default mode is a non-mutating plan:

```bash
python3 scripts/validate_deployment_host_runtime.py
```

To also run the local smoke sequence against a fresh non-secret workspace:

```bash
python3 scripts/validate_deployment_host_runtime.py --run-runtime-smoke --config config.example.toml --runtime-workspace target/deployment-host-runtime-smoke
```

This still does not install, enable, reload, start, stop, or restart a service. It records only local smoke flags and remaining external evidence requirements.

`scripts/validate_rollback_drill.py` creates a sanitized rollback-drill evidence plan without changing services, files, deployments, or runtime state:

```bash
python3 scripts/validate_rollback_drill.py
```

When operator-owned non-secret references exist, strict mode verifies that the rollback drill record has the minimum metadata needed for review:

```bash
python3 scripts/validate_rollback_drill.py --strict --candidate-ref <commit-or-run> --rollback-ref <commit-or-artifact> --reviewer <role-or-handle> --run-url <non-secret-run-url>
```

This is evidence planning only. Actual service quiesce, artifact restore, post-rollback runtime smoke, audit/SQLite recovery, and reviewer approval remain external manual evidence requirements.

`scripts/validate_incident_response_drill.py` creates a sanitized incident-response drill evidence plan without changing services, files, deployments, alert routes, or runtime state:

```bash
python3 scripts/validate_incident_response_drill.py
```

When operator-owned non-secret references exist, strict mode verifies that the incident drill record has the minimum metadata needed for review:

```bash
python3 scripts/validate_incident_response_drill.py --strict --scenario service-unhealthy --severity medium --responder <role-or-handle> --reviewer <role-or-handle> --run-url <non-secret-run-url>
```

This is evidence planning only. Actual detection, triage, containment, recovery, communications escalation, post-incident runtime smoke, audit/SQLite recovery, and reviewer approval remain external manual evidence requirements.

`scripts/validate_deployment_evidence_bundle.py` runs the non-mutating local evidence helpers and emits a compact operator-review index without embedding full artifact contents:

```bash
python3 scripts/validate_deployment_evidence_bundle.py
python3 scripts/validate_deployment_evidence_bundle.py --json
```

The bundle includes structure validation, static systemd example validation, lifecycle plan validation, deployment-host runtime plan validation, rollback-drill plan validation, and incident-response drill plan validation. It does not install, enable, reload, start, stop, or restart services, and it does not claim deployment or production readiness.

`scripts/validate_deployment_evidence_checklist.py` builds on the bundle index and marks each remaining external evidence category as either missing or referenced by a sanitized locator:

```bash
python3 scripts/validate_deployment_evidence_checklist.py
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_deployment_evidence_checklist.py --evidence service-lifecycle=<non-secret-run-or-artifact-ref>
```

The checklist stores references only. It rejects secret-like locator text and does not copy artifact contents, call networks, mutate deployment state, or claim production readiness.

GitHub Actions also generates the same checklist as a short-retention `deployment-evidence-checklist` artifact and links it from the hardening evidence job summary. The CI artifact is a missing-evidence index only; it is not deployment-host validation.

Additional target-specific validation is required for containers, systemd, ARM builds, rollback drills, incident drills, and security review.

## Phase 16 Boundary

The Rust `arb-core::packaging` module records deterministic package/deployment plans and rejects live trading, public exposure, embedded secret material, build claims, deployment claims, and production claims.
