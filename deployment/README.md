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
python3 scripts/validate_deployment_static_hardening.py --run-config-smoke
python3 scripts/validate_arm_build_profiles.py
```

`scripts/validate_release_artifact.py` builds the locked release binary, copies it into `target/release-artifacts/`, writes a SHA-256 manifest plus unsigned provenance record, verifies the generated bundle hashes and false-claim fields, and smoke-runs the copied binary help path. Release build, smoke, and metadata helper commands use bounded timeouts and fail closed on timeout:

```bash
python3 scripts/validate_release_artifact.py
```

Passing this script is unsigned local release-artifact build/provenance evidence only. It does not sign artifacts, upload attestations, publish releases, push images, install services, load secrets, deploy infrastructure, or claim production readiness.

On a host with the Rust ARM target and `aarch64-linux-gnu-gcc` cross compiler installed, `scripts/validate_arm_cross_check.py` runs the non-secret ARM workspace check with bounded `rustup` and Cargo command timeouts:

```bash
python3 scripts/validate_arm_cross_check.py --install-target
```

This runs `cargo check --workspace --target aarch64-unknown-linux-gnu --locked`. With `--install-target`, it may install the Rust target before checking. Missing prerequisites or command timeouts fail the gate without running ARM binaries, inspecting devices, starting services, loading secrets, or approving ARM deployment readiness.

`scripts/validate_production_container.py` builds `deployment/container/Containerfile.production`, runs Trivy HIGH/CRITICAL image scanning, enforces no fixable CRITICAL vulnerabilities, smoke-runs the inert `arb-agent --help` container path, and repeats that smoke under Docker `--read-only --network none --cap-drop ALL --security-opt no-new-privileges`. Docker probe, build, scan, and smoke commands use bounded timeouts, Dockerized Trivy runs with `--pull never` plus an in-container scan timeout, and timed-out named scan containers are force-removed before the script returns fail-closed with explicit non-claims if Docker is unavailable or unresponsive; `--json` emits the same fail-closed result as structured evidence:

```bash
python3 scripts/validate_production_container.py
```

Passing this script is production-intent container build/scan and hardened local container-smoke evidence only. Failing it proves no production-container evidence was collected for that run. It does not push an image, install or start a service, mount production state, open listeners, load secrets, validate service-manager lifecycle behavior, or claim production readiness.

`scripts/validate_container_example.py --json` mirrors the example-container build/scan/smoke gate, including Dockerized Trivy no-pull/internal-timeout controls and timed-out scan-container cleanup, and also emits structured fail-closed JSON when Docker is unavailable or unresponsive.

`scripts/validate_systemd_example.py` checks the committed example unit only. On Linux hosts with `systemd-analyze`, `python3 scripts/validate_systemd_example.py --systemd-analyze` also runs bounded syntax verification against a temporary fake root without installing, enabling, reloading, or starting a service. Passing it is not production service-manager validation.

`scripts/validate_deployment_static_hardening.py --run-config-smoke` checks committed example and production-intent container, systemd, and config invariants such as distroless/non-root runtime, no exposed ports, no embedded environment values, strict systemd filesystem hardening, bounded write paths, observe-or-paper config loading, live-execution denial, and secret-like output denial. The optional local config smoke command is bounded and fails closed on timeout. It does not build or push images, install services, open listeners, call networks, load secrets, or claim production readiness.

`scripts/validate_arm_build_profiles.py` checks the committed ARM build-profile notes for required target triples, future cross-build commands, external target-class validation requirements, no-execution language, and no production-readiness claim. It does not install Rust targets, cross-compile, run emulators, inspect devices, call networks, or approve ARM deployment readiness.

`scripts/validate_systemd_lifecycle.py` adds a manual deployment-host lifecycle evidence helper. Its default mode is a non-mutating plan:

```bash
python3 scripts/validate_systemd_lifecycle.py
```

On a Linux deployment host where the unit already exists, inspect mode can collect read-only `systemctl show` state with a bounded timeout, without installing, enabling, reloading, starting, stopping, or restarting the service:

```bash
python3 scripts/validate_systemd_lifecycle.py --mode inspect --unit arb-agent.service --json
```

Inspect output is reference evidence only. Timeout or unavailable-systemd behavior fails closed and does not prove service lifecycle execution. Operator-controlled service start, graceful shutdown, restart, recovery, rollback, and deployment-host audit/SQLite validation still require separate non-secret evidence before any deployment claim.

`scripts/validate_deployment_host_runtime.py` composes the lifecycle helper with the existing local runtime smoke runner. Default mode is a non-mutating plan:

```bash
python3 scripts/validate_deployment_host_runtime.py
```

To also run the local smoke sequence against a fresh non-secret workspace:

```bash
python3 scripts/validate_deployment_host_runtime.py --run-runtime-smoke --config config.example.toml --runtime-workspace target/deployment-host-runtime-smoke
```

This still does not install, enable, reload, start, stop, or restart a service. Its lifecycle helper call is bounded and records only local smoke flags and remaining external evidence requirements.

To inspect candidate deployment-host audit retention paths without rotating or deleting logs:

```bash
python3 scripts/validate_deployment_host_runtime.py --run-retention-preflight --retention-active-path /var/log/arb-agent/audit.jsonl --retention-archive-dir /var/log/arb-agent/archive --json
```

Retention preflight checks only path shape, parent/directory access, and secret-like path names. It reports `rotation_performed: false`, `deletion_performed: false`, and `production_paths_touched: false`; actual deployment-host retention/rotation execution remains a separate external evidence requirement.

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

The bundle includes structure validation, static systemd example validation, lifecycle plan validation, deployment-host runtime plan validation, deployment-host retention preflight validation, rollback-drill plan validation, and incident-response drill plan validation with bounded local helper execution. It does not rotate, delete, create, open, lock, or fsync production audit logs; it does not install, enable, reload, start, stop, or restart services; and it does not claim deployment or production readiness.

`scripts/validate_deployment_evidence_checklist.py` builds on the bundle index and marks each remaining external evidence category as either missing or referenced by a sanitized locator:

```bash
python3 scripts/validate_deployment_evidence_checklist.py
python3 scripts/validate_deployment_evidence_checklist.py --json
python3 scripts/validate_deployment_evidence_checklist.py --evidence service-lifecycle=<non-secret-run-or-artifact-ref>
```

The checklist stores references only and loads the bundle through a bounded local helper call. It rejects secret-like locator text and does not copy artifact contents, call networks, mutate deployment state, or claim production readiness.

GitHub Actions also generates the same checklist as a short-retention `deployment-evidence-checklist` artifact and links it from the hardening evidence job summary. The CI artifact is a missing-evidence index only; it is not deployment-host validation.

Additional target-specific validation is required for containers, systemd, ARM builds, rollback drills, incident drills, and security review.

## Phase 16 Boundary

The Rust `arb-core::packaging` module records deterministic package/deployment plans and rejects live trading, public exposure, embedded secret material, build claims, deployment claims, and production claims.
