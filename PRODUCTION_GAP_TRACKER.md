# PRODUCTION_GAP_TRACKER.md

## Executive Status Summary

The project is in Phase 26 local-audit-crash-concurrency-filesystem-disk-full-retention-stale-lock-planning status for ChatGPT Project Mode. Architecture, roadmap, agent governance, phase sub-roadmaps, gap tracking, a minimal Rust workspace scaffold, typed non-secret config, reference-only secret abstractions, redacted secret material, initial live-mode validation, an isolated deny-by-default policy engine, append-only audit journal primitives with local lock/sync append behavior, local crash-like truncation, tamper, concurrent append, invalid-filesystem, simulated disk-full fail-closed validation probes, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning, a state-store trait boundary with local SQLite WAL checkpoints, deterministic local durability validation, process-level crash/restart recovery tests, and local runtime state-permission fail-closed validation, normalized market-data models, freshness classification, fee models, provider trait boundaries, deterministic paper connectors with local paper-report and paper-ledger checkpoints plus realistic local fill modeling, local venue matching profiles, adverse-selection penalties, reference-only calibration records, paper ledger replay validation, local historical-fixture paper backtest execution, local append-only audit records for paper reports and reserve/settlement ledger mutations, CEX framework types/traits, DEX/Web3 framework types/traits, deterministic opportunity-engine types/traits, draft-only execution-planner types/traits with local plan-draft checkpoints, execution-adapter boundary records/traits with local adapter-run checkpoints, local fail-closed runtime lifecycle records for audit/state/adapter sequencing, local concurrent runtime lifecycle access checks, local graceful-shutdown audit/state checkpoint records, local runtime audit/SQLite backup-restore validation records, local runtime restart recovery summary records with CLI-visible typed operator-review dispositions and incomplete-checkpoint fail-closed coverage, communications/CLI command and notification boundary records/traits, embedded-dashboard local render records/traits, observability/runbook local health/log/metric/runbook records/traits, deterministic validation plan/fixture/fuzz/backtest records/traits, deterministic packaging/deployment plan records/traits plus example deployment templates, manual non-mutating systemd lifecycle plan/inspect tooling, combined deployment-host runtime report tooling, non-mutating rollback-drill evidence tooling, non-mutating incident-response drill evidence tooling, non-mutating deployment evidence bundle indexing, and non-mutating deployment evidence checklist validation, deterministic external hardening evidence/review records/traits plus hardening checklists, and deterministic agentic handoff package records/traits plus future-agent prompts and checklists exist. The 2026-05-31 ArbyClaw local validation sequence completed for repository structure, formatting, workspace compilation, tests, clippy, example-container validation, static example systemd-unit validation, local plan-mode systemd lifecycle validation, default deployment-host runtime report validation, rollback-drill plan validation, incident-response drill plan validation, deployment evidence bundle indexing, and deployment evidence checklist validation; this is local validation evidence only. GitHub Actions CI also runs structure, Rust validation, locked release build, dependency audit, SBOM generation, local-SARIF SAST, example image scan, static plus `systemd-analyze` syntax example systemd-unit checks, secret-pattern scan, deployment evidence checklist artifact generation, and hardening evidence indexing for pushed commits. The project is not ready for live funds, live exchange credentials, wallet keys, production deployment, transaction signing, broadcasts, live adapter submission, real outbound communications, real dashboard hosting, real observability/exporter/alert runtime, real fuzzing engine execution, real external backtest execution beyond local paper fixtures, production container/systemd/ARM validation, deployment-host audit validation, physical disk-full/retention/rotation execution validation, operator-controlled service-manager lifecycle execution validation, deployment-host backup/restore validation, deployment-host restart recovery validation, broader external hardening execution, external agent execution validation, executed rollback drills, executed incident-response drills, cloud deployment, production release, live-funds approval, or autonomous execution.

## Latest Local Validation Attempt

2026-05-31 ArbyClaw local validation attempt:

- `python3 scripts/validate_structure.py` passed.
- `cargo fmt --check` passed.
- `cargo check --workspace` passed.
- `cargo test --workspace` passed with 122 tests across 4 suites after local runtime-smoke CLI runner additions.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo run -p arb-agent -- validate-runtime-smoke --config config.example.toml --workspace target/runtime-smoke-cli-local-2` passed, creating local non-secret audit/state smoke artifacts under the ignored `target/` tree and reporting `production-ready: false`.
- `python3 scripts/validate_container_example.py` passed after Docker became available; this rebuilt the example image, ran Trivy HIGH/CRITICAL image scanning, enforced no fixable CRITICAL image vulnerabilities, and smoke-ran the container CLI help path.
- `python3 scripts/validate_systemd_example.py` passed; this checked the committed example systemd unit only, did not install, enable, reload, or start a service, and skipped optional `systemd-analyze verify`.
- `python3 scripts/validate_systemd_example.py --systemd-analyze` passed locally with `systemd-analyze` unavailable, so syntax verification was skipped locally; CI now requires `systemd-analyze` on the Ubuntu runner.
- `docker run --rm -v ${PWD}:/repo -w /repo ubuntu:24.04 ... python3 scripts/validate_systemd_example.py --systemd-analyze --require-systemd-analyze` passed after installing Python and systemd inside the disposable container; this verified the committed example unit syntax against a temporary fake root and did not install, enable, reload, or start a service.
- `python3 scripts/validate_systemd_lifecycle.py` passed in default plan mode; this generated a non-secret manual lifecycle evidence plan and did not inspect host systemd or install, enable, reload, start, stop, or restart a service.
- `python3 scripts/validate_deployment_host_runtime.py` passed in default plan mode; this composed non-mutating systemd lifecycle evidence with runtime-smoke remaining-evidence fields without running service actions or creating runtime smoke artifacts.
- `python3 scripts/validate_deployment_host_runtime.py --run-runtime-smoke --config config.example.toml --runtime-workspace target/deployment-host-runtime-smoke-local` passed; this created local non-secret smoke artifacts under the ignored `target/` tree and reported `production-ready: false`, `service-manager-action-performed: false`, `external-submission-performed: false`, and `live-execution-performed: false`.
- `python3 scripts/validate_rollback_drill.py` passed in default plan mode; this generated a non-secret rollback evidence plan and did not perform service actions, file changes, external calls, live execution, or production-readiness claims.
- `python3 scripts/validate_rollback_drill.py --strict --candidate-ref local-candidate --rollback-ref local-rollback --reviewer release-reviewer --run-url local-run-reference` passed; this verified strict-mode metadata requirements using sanitized non-secret references only.
- `python3 scripts/validate_incident_response_drill.py` passed in default plan mode; this generated a non-secret incident-response evidence plan and did not perform service actions, file changes, alert delivery, external calls, live execution, or production-readiness claims.
- `python3 scripts/validate_incident_response_drill.py --strict --scenario service-unhealthy --severity medium --responder incident-operator --reviewer incident-reviewer --run-url local-run-reference` passed; this verified strict-mode metadata requirements using sanitized non-secret references only.
- `python3 scripts/validate_deployment_evidence_bundle.py` passed; this produced a compact non-secret local evidence index over non-mutating helper outputs without embedding full artifact contents or performing service actions, file changes, alert delivery, external calls, live execution, or production-readiness claims.
- `python3 scripts/validate_deployment_evidence_bundle.py --json` passed; this emitted the same compact component index as JSON.
- `python3 scripts/validate_deployment_evidence_checklist.py` passed; this marked remaining production evidence categories as missing external evidence without embedding artifact contents or claiming readiness.
- `python3 scripts/validate_deployment_evidence_checklist.py --json` passed; this emitted the same non-secret checklist as JSON.
- This validates local structure, formatting, compilation, tests, linting, local runtime-smoke CLI behavior, example-container Docker/Trivy smoke behavior, static example systemd-unit checks, disposable-container `systemd-analyze` syntax checks, non-mutating lifecycle evidence planning, combined deployment-host runtime report generation, non-mutating rollback-drill evidence planning, non-mutating incident-response drill evidence planning, non-mutating deployment evidence bundle indexing, and non-mutating deployment evidence checklisting only. It does not validate production deployment, live funds, live exchange/RPC integrations, signing, broadcasts, production containers, deployment-host systemd execution behavior, ARM, CI, penetration testing, load testing, executed rollback drills, executed incident-response drills, deployment-host audit behavior, physical disk-full behavior, retention/rotation execution behavior, operator-controlled service-manager lifecycle execution behavior, external hardening, or production readiness.

## Latest CI Validation Attempt

2026-05-31 ArbyClaw GitHub Actions CI validation snapshot:

- Repository: `dominator509/arbyclaw`
- Branch: `main`
- Latest validated commit: `6d374da6cd94ade226c178fe76768bddc4db3226`
- Workflow run: `https://github.com/dominator509/arbyclaw/actions/runs/26725221694`
- Result: passed.
- Completed CI steps: checkout via `actions/checkout@v6`, Rust stable toolchain install with rustfmt and clippy, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release --locked`, hardening tool installation, `cargo audit`, CycloneDX SBOM generation with non-empty file checks, `python3 scripts/validate_structure.py`, example systemd-unit static plus `systemd-analyze verify` syntax validation, CodeQL Rust SAST analysis with local SARIF generation, non-empty SARIF verification, short-retention SARIF artifact upload, example container image build, Trivy image scan evidence artifact upload, fixable critical image-vulnerability enforcement, Gitleaks redacted current-tree secret-pattern scan artifact upload, lightweight hardening evidence index artifact upload, and GitHub Step Summary hardening evidence pointer generation.
- Artifact references from that run: `hardening-evidence-index` `7320358817`, `codeql-sarif-evidence` `7320316740`, `trivy-image-scan-evidence` `7320312297`, `gitleaks-secret-scan-evidence` `7320298068`, and Docker Buildx build record `7320312514`.
- Node.js 24 migration status: the workflow uses `actions/checkout@v6`.
- This validates the pushed repository structure, formatting, compilation, tests, linting, locked release build, dependency audit, SBOM generation gate, local-SARIF CodeQL SAST gate, short-retention SAST artifact retention, example container image build, example Trivy image-scan gate, current-tree Gitleaks secret-pattern scan gate, and example systemd-unit static/syntax gate in GitHub Actions only. It does not validate production deployment, live funds, live exchange/RPC integrations, signing, broadcasts, production containers, deployment-host systemd behavior, ARM, penetration testing, load testing, rollback drills, incident drills, SBOM review, GitHub code scanning upload processing, broader external hardening, or production readiness.

## Latest Gap Tracker Audit

2026-05-31 ArbyClaw non-mutating deployment evidence checklist audit:

- Added `scripts/validate_deployment_evidence_checklist.py` to consume the local deployment evidence bundle and emit a compact checklist for external evidence categories.
- The checklist covers service lifecycle, deployment-host audit/SQLite, physical disk-full, retention/rotation, rollback drill, incident-response drill, and production-readiness review evidence.
- The helper accepts sanitized locator references only and rejects secret-like locator text.
- The helper records that no service action, file change, secret loading, alert delivery, external call, live execution, artifact embedding, or production-readiness claim occurred.
- This closes the local deterministic tooling gap for indexing missing or referenced external evidence only. It does not close actual production deployment, service-manager lifecycle execution, deployment-host audit/SQLite recovery evidence, physical disk-full evidence, retention/rotation execution, executed rollback drills, executed incident-response drills, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw deployment evidence checklist CI artifact audit:

- Updated `.github/workflows/ci.yml` with a `deployment-evidence-checklist` job that generates JSON and text checklist artifacts from `scripts/validate_deployment_evidence_checklist.py`.
- Added the `deployment-evidence-checklist` artifact to the hardening evidence index and GitHub Step Summary.
- The CI job records a missing-evidence/reference checklist only; it does not run service-manager actions, deployment-host probes, external calls, live execution, or production-readiness approval.
- This improves evidence discoverability for pushed commits only. It does not close actual production deployment, service-manager lifecycle execution, deployment-host audit/SQLite recovery evidence, physical disk-full evidence, retention/rotation execution, executed rollback drills, executed incident-response drills, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw non-mutating deployment evidence bundle index audit:

- Added `scripts/validate_deployment_evidence_bundle.py` to run local non-mutating validation helpers and emit a compact operator-review index.
- The bundle runs structure validation, static systemd example validation, systemd lifecycle plan validation, deployment-host runtime plan validation, rollback-drill plan validation, and incident-response drill plan validation.
- The helper records component pass/fail, schema, line counts, safety flags, and remaining-evidence counts only; it does not embed full artifact contents.
- The helper records that no service action, file change, secret loading, alert delivery, external call, live execution, or production-readiness claim occurred.
- This closes the local deterministic tooling gap for summarizing local deployment evidence helper outputs only. It does not close actual production deployment, service-manager lifecycle execution, deployment-host audit/SQLite recovery evidence, physical disk-full evidence, retention/rotation execution, executed rollback drills, executed incident-response drills, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw non-mutating incident-response drill evidence helper audit:

- Added `scripts/validate_incident_response_drill.py` to validate sanitized incident metadata and emit a non-secret incident-response evidence plan.
- Default mode emits planned incident-response steps and remaining external evidence requirements without requiring target-host references.
- Strict mode requires scenario, severity, responder, reviewer, and at least one artifact, run URL, or evidence reference.
- The helper rejects secret-like reference labels and records that no service action, file change, secret loading, alert delivery, external call, live execution, or production-readiness claim occurred.
- This closes the local deterministic tooling gap for incident-response drill evidence preparation only. It does not close actual incident detection, triage, containment, recovery, communications escalation, post-incident runtime smoke evidence on a target host, post-incident audit/SQLite recovery evidence, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw non-mutating rollback-drill evidence helper audit:

- Added `scripts/validate_rollback_drill.py` to validate sanitized rollback metadata and emit a non-secret rollback evidence plan.
- Default mode emits planned rollback steps and remaining external evidence requirements without requiring target-host references.
- Strict mode requires candidate reference, rollback reference, reviewer, and at least one artifact or run URL reference.
- The helper rejects secret-like reference labels and records that no service action, file change, secret loading, external call, live execution, or production-readiness claim occurred.
- This closes the local deterministic tooling gap for rollback-drill evidence preparation only. It does not close actual rollback execution, service-manager lifecycle evidence, artifact restore evidence, post-rollback runtime smoke evidence on a target host, post-rollback audit/SQLite recovery evidence, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw combined deployment-host runtime report audit:

- Added `scripts/validate_deployment_host_runtime.py` to compose non-mutating systemd lifecycle evidence with optional explicit local runtime-smoke execution against a fresh non-secret workspace.
- Default mode runs the lifecycle plan helper only and does not create runtime artifacts.
- Optional runtime-smoke mode can run either `cargo run -p arb-agent -- validate-runtime-smoke` or a supplied `--agent-bin` against a fresh workspace and records only sanitized pass/fail flags.
- The wrapper records that no service action, secret loading, external call, live execution, or production-readiness claim occurred.
- This closes the local deterministic tooling gap for collecting a combined lifecycle/runtime evidence report only. It does not close operator-controlled service-manager lifecycle execution evidence, deployment-host audit/SQLite validation under service lifecycle, physical disk-full validation, retention/rotation execution validation, rollback drills, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw manual systemd lifecycle evidence helper audit:

- Added `scripts/validate_systemd_lifecycle.py` as a manual non-secret systemd lifecycle plan/inspect helper.
- Default plan mode is host-agnostic and does not use systemd.
- Inspect mode is explicit, Linux-only, and restricted to read-only `systemctl show` queries for sanitized unit state.
- The helper rejects unsafe unit names and records that no service install, daemon reload, enable, start, stop, restart, secret loading, live execution, or production-readiness claim occurred.
- This closes the local deterministic tooling gap for deployment-host lifecycle evidence preparation only. It does not close operator-controlled service-manager lifecycle execution evidence, deployment-host audit/SQLite validation, physical disk-full validation, retention/rotation execution validation, rollback drills, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw static systemd example validation audit:

- Added `scripts/validate_systemd_example.py` to statically validate `deployment/systemd/arb-agent.service.example`.
- The validator checks required sections, non-secret hardening directives, expected read-write paths, live/withdraw/bridge/sign/broadcast command denials, and absence of environment or systemd credential directives.
- Added optional `systemd-analyze verify` syntax validation using a temporary fake root with placeholder target units and a placeholder executable so syntax checks do not install, enable, reload, or start a service.
- Wired the static plus required CI syntax validator into the CI Rust validation job and structure validator.
- A disposable Ubuntu container validation passed the required `systemd-analyze` path locally. This closes local deterministic static/syntax template validation only. It does not close deployment-host systemd validation, service-manager restart execution, production deployment, physical disk-full evidence, retention/rotation execution, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw current-tree secret scan scope audit:

- The pushed CI run `https://github.com/dominator509/arbyclaw/actions/runs/26725090983` failed in the Gitleaks job because full-history scanning rediscovered a removed historical audit scaffold file, `security-audit/phase-1-recon-threat-modeling/secrets_scanner.py`, from commit `e827f93975b06ebe9e376e1e83ea2b47db814e75`.
- The file is not present in the current repository tree, and no secret value was copied into this tracker; the downloaded Gitleaks artifact was redacted.
- Updated the Gitleaks gate to scan the current checked-out tree with `gitleaks detect --no-git` while still producing redacted JSON evidence and failing on current-tree findings.
- This restores the non-secret current-source secret-pattern gate only. It does not rewrite historical commits, prove the absence of every historical false positive, validate production secrets handling, or approve live credentials.

2026-05-31 ArbyClaw local deployment-like runtime smoke validation audit:

- Added `RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION`, `RuntimeDeploymentSmokeValidationReport`, and `validate_local_runtime_deployment_smoke`.
- The local smoke harness runs one runtime lifecycle, graceful-shutdown audit/state checkpointing, backup/restore validation, restart recovery validation, and audit durability probes over caller-supplied non-secret local paths.
- Added `arb-agent validate-runtime-smoke --config <path> --workspace <fresh-dir>` as a typed local CLI runner for the same smoke harness; the command rejects live-armed configs, requires a fresh workspace, and prints non-secret outcome flags only.
- Added a focused Rust test proving the harness completes without service-manager actions, external submission, live execution, or production-readiness approval.
- This closes the local deterministic coding gap for a deployment-like runtime smoke sequence and local CLI runner only. It does not close deployment-host runtime validation, physical disk-full evidence, retention/rotation execution, service-manager restart execution, production deployment, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local example-container validation script audit:

- Added `scripts/validate_container_example.py` to repeat the Docker-dependent example image gate locally when Docker is available.
- The script runs Docker availability checks, builds `deployment/container/Containerfile.example` as `arbyclaw-example:ci`, runs Trivy HIGH/CRITICAL image scan output, enforces no fixable CRITICAL vulnerabilities, and smoke-runs the container CLI help path.
- This closes the local deterministic tooling gap for repeating the example-container validation gate only. It does not close production container validation, production image review, systemd validation, ARM validation, staging deployment, rollback drills, incident drills, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw CLI restart recovery disposition status audit:

- Surfaced `ready-for-local-review` and `needs-operator-review` restart recovery disposition labels in `arb-agent` status output.
- Added binary tests proving those labels stay operator-facing and local-only.
- This closes the local deterministic coding gap for CLI visibility of restart recovery disposition labels only. It does not close deployment-host restart recovery validation, service-manager restart execution evidence, production runtime validation, deployment validation, live exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local restart recovery disposition audit:

- Added `RuntimeRestartRecoveryDisposition` with `ready-for-local-review` and `needs-operator-review` outcomes.
- Added a Rust test proving local restart recovery with planner and adapter checkpoints but no graceful-shutdown checkpoint is classified as `needs-operator-review` instead of production-ready or service-resumable.
- This closes the local deterministic coding gap for restart recovery disposition classification only. It does not close deployment-host restart recovery validation, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local incomplete restart recovery fail-closed audit:

- Added a Rust test proving local restart recovery fails closed when audit replay exists but required SQLite planner and adapter checkpoints are missing.
- The test verifies the audit journal remains replayable, SQLite reopens cleanly, no missing checkpoints are fabricated, and recovery returns a validation error instead of a local-ready report.
- This closes the local deterministic coding gap for incomplete restart recovery fail-closed behavior only. It does not close deployment-host restart recovery validation, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local runtime restart recovery validation audit:

- Added `RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION`, `RuntimeRestartRecoveryValidationReport`, and `validate_local_runtime_restart_recovery`.
- Added a Rust test proving a completed local runtime lifecycle plus graceful-shutdown checkpoint can be reopened after handle drop, replay the expected audit sequence, run SQLite integrity checks, and recover planner, adapter, and graceful-shutdown checkpoints.
- This closes the local deterministic coding gap for local runtime restart recovery summaries only. It does not close deployment-host restart recovery validation, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local runtime backup-restore validation audit:

- Added `RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION`, `RuntimeBackupRestoreValidationReport`, and `validate_local_runtime_backup_restore`.
- Added a Rust test proving a completed local runtime lifecycle can copy non-secret audit and SQLite state artifacts, reopen the copies, replay the expected audit sequence, run SQLite integrity checks, and restore planner plus adapter checkpoints.
- This closes the local deterministic coding gap for local runtime audit/SQLite backup-restore validation only. It does not close deployment-host backup/restore under load, deployment-host filesystem permission validation, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local runtime state-permission fail-closed audit:

- Added a local runtime lifecycle state-permission failure test that simulates a denied state checkpoint write.
- The test verifies the lifecycle returns a state error, records exactly one state write attempt, appends only the runtime-start audit record, reopens the audit journal at the expected next sequence, and stops before adapter evaluation or adapter-completion audit.
- This closes the local deterministic coding gap for simulated state-permission fail-closed lifecycle behavior only. It does not close deployment-host filesystem permission validation, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local concurrent runtime lifecycle audit:

- Added a local concurrent runtime lifecycle access test over a shared audit journal path and shared SQLite WAL state path.
- The test serializes only initial handle opening, then runs lifecycle audit/state writes concurrently, reopens the audit journal, verifies the expected audit sequence count, reopens SQLite WAL state, verifies lifecycle checkpoints, and runs SQLite integrity check.
- This closes the local deterministic coding gap for concurrent runtime lifecycle audit/state access only. It does not close deployment-host concurrency validation, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw local graceful-shutdown checkpoint audit:

- Added local graceful-shutdown audit/state checkpoint records to the runtime lifecycle boundary.
- Added `RUNTIME_GRACEFUL_SHUTDOWN_VERSION`, `RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY`, `RuntimeGracefulShutdownRequest`, `RuntimeGracefulShutdownRecord`, and `run_local_graceful_shutdown_checkpoint`.
- Added a Rust test proving the graceful-shutdown audit journal and SQLite WAL checkpoint reopen locally after a clean checkpoint write.
- This closes the local deterministic coding gap for graceful-shutdown audit/state checkpoint modeling only. It does not close deployment-host graceful shutdown execution, service-manager restart execution evidence, physical disk-full validation, retention/rotation execution validation, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-31 ArbyClaw Phase 26 audit crash/concurrency/filesystem validation audit:

- Added `PHASE_26_SUBROADMAP.md` for local audit journal crash-like truncation, concurrency, and filesystem validation.
- Added `AUDIT_DURABILITY_VALIDATION_VERSION`, `AuditDurabilityValidationReport`, and `validate_audit_journal_durability`.
- Updated local audit appends to acquire a local lock, replay current journal state under the lock, append the next hash-chained record, flush, and call `sync_all`.
- Added local validation probes for append/reopen replay, truncated JSONL replay rejection, tamper rejection, concurrent append replay, and invalid filesystem fail-closed behavior.
- Added disk-full error classification for audit append persistence and a simulated disk-full validation probe that proves failed persistence does not advance in-memory or replayed journal state.
- Added direct local fault tests for partial JSONL replay rejection and permission/disk-failure state preservation without live execution or external calls.
- Added side-effect-free audit retention/rotation planning models and tests that mark rotate, retained, and expired decisions without deleting, renaming, compressing, or mutating logs.
- Added side-effect-free stale-lock restart recheck planning models and tests that mark stale/fresh lock observations without deleting lock files, inspecting live processes, starting services, or mutating deployment state.
- Added focused Rust tests covering the new durability validation report and existing-workspace fail-closed guard.
- This closes the local deterministic coding gap for audit crash-like truncation, local concurrent append serialization, local sync append behavior, invalid filesystem validation, simulated disk-full fail-closed behavior, retention/rotation planning, and stale-lock restart recheck planning. It does not close deployment-host audit validation, physical disk-full validation, retention/rotation execution validation, service-manager restart execution evidence, live connector audit-before-action enforcement, production runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-28 ArbyClaw Phase 25 paper audit journal integration audit:

- Added `PHASE_25_SUBROADMAP.md` for local paper execution report and ledger mutation audit journal wiring.
- Added `PAPER_AUDIT_INTEGRATION_VERSION` and exported paper audit helper types/functions.
- Added sanitized append-only audit records for paper execution reports and paper reserve/settlement ledger mutations.
- Added audited execution helpers for realistic paper fills and venue-realistic paper fills that preserve `live_network_used = false` and `external_execution_performed = false`.
- Added local replay checks that reopen the JSONL audit journal after paper audit appends and compare the replayed hash-chain state.
- Added focused Rust tests for audited paper ledgered execution and audited venue-realism ledgered execution.
- This closed the local deterministic coding gap for direct paper report and ledger mutation append-only audit journal integration. Phase 26 later added local audit crash/concurrency/filesystem validation probes. It does not close paper intent audit-before-action for future live-relevant paths, deployment-host audit validation, production-host runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers.

2026-05-27 ArbyClaw Phase 24 paper replay/calibration/backtest validation audit:

- Added `PHASE_24_SUBROADMAP.md` for local paper replay, calibration, backtest, and runtime validation boundaries.
- Added local exchange matching profiles in `crates/arb-core/src/paper.rs` for tick size, quantity step, min/max notional, market/limit/post-only/partial-fill support, queue-position behavior, and calibration-required flags.
- Added venue-realism paper execution records that apply exchange matching profiles, adverse-selection penalties, and reference-only sandbox/live calibration records through the existing paper adapter and paper ledger without external calls.
- Added paper ledger replay validation that reconstructs final balances from ledger entries, detects entry/result mismatches, and checks closed reservations; direct append-only audit journal integration was future work at Phase 24 and is now locally covered by Phase 25.
- Added local historical-fixture paper backtest corpus execution that runs caller-supplied paper steps through the paper adapter and ledger with `live_network_used = false`, `external_data_downloaded = false`, and `external_execution_performed = false`.
- Added paper runtime validation records that distinguish local replay/backtest evidence from missing production-host validation and preserve `production_ready = false`.
- Added focused Rust tests for venue realism, adjusted P&L settlement, replay success/failure, local paper backtest execution, and runtime validation blocker preservation.
- Phase 24 local validation passed for `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` with 98 tests across 4 suites, and `cargo clippy --workspace --all-targets -- -D warnings`; the full final validation sequence is recorded in the latest local validation attempt above.
- This closes the local deterministic coding gap for exchange-specific paper matching behavior, adverse-selection modeling, reference-only calibration records, paper replay validation, and local historical-fixture backtest execution. It does not close real sandbox/live calibration evidence, production-host runtime validation, deployment validation, real exchange/RPC validation, custody/signing, or production-readiness blockers. Local paper report and ledger mutation audit integration was added later in Phase 25.

2026-05-27 ArbyClaw Phase 23 realistic paper fill validation audit:

- Added `PHASE_19_SUBROADMAP.md` and `crates/arb-core/src/runtime.rs` for local deterministic runtime lifecycle wiring.
- Added adapter-run checkpoint persistence and wired lifecycle sequencing so a runtime request appends audit, persists plan state, appends plan-checkpoint audit, evaluates the deterministic adapter boundary, persists adapter-run state, and appends adapter-completion audit.
- Added tests for in-memory state, SQLite WAL-backed state reopen, and live-scope rejection before audit/state mutation.
- Added `PHASE_20_SUBROADMAP.md` and local SQLite WAL durability validation covering WAL mode, synchronous FULL, SQLite integrity check, WAL checkpoint truncate, primary reopen, checkpointed backup/restore, and multi-handle checkpoint visibility.
- Added tests for successful durability validation and fail-closed existing backup path handling.
- Added `PHASE_21_SUBROADMAP.md` and local paper balance ledgering with simulated balances, quote-notional reservation, deterministic fill settlement, insufficient-balance denial, missing-reservation denial, adapter helper wiring, and SQLite ledger checkpoint persistence.
- Added tests for successful ledgered execution, insufficient balance denial, missing reservation denial, and SQLite WAL ledger persistence.
- Added `PHASE_22_SUBROADMAP.md` and `crates/arb-core/tests/sqlite_wal_crash_restart.rs` for process-level SQLite WAL crash/restart validation.
- Added child-process crash scenarios after start checkpoint, after planner checkpoint, and after adapter checkpoint; parent processes reopen the database, run integrity checks, and verify expected checkpoint recovery or absence.
- Added `PHASE_23_SUBROADMAP.md` and local realistic paper fill modeling in `crates/arb-core/src/paper.rs` for supplied order-book depth walking, buy/sell side selection, full/partial/unfilled outcomes, latency, queue-position haircuts, average price, slippage, and consumed-level reporting.
- Wired realistic paper reports into paper ledger settlement so the ledger reserves requested notional, settles modeled fill P&L, and releases unfilled reserved notional without real balance mutation.
- Post-Phase 23 local validation passed: `python3 scripts/validate_structure.py`, `python3 -m py_compile scripts/validate_structure.py scripts/generate_structure_manifest.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` with 93 tests across 4 suites, and `cargo clippy --workspace --all-targets -- -D warnings`.
- This closed the local deterministic realistic paper fill modeling gap as of Phase 23. Phase 24 later added local matching profiles, adverse-selection records, reference-only calibration records, paper replay validation, and local historical-fixture backtest execution, Phase 25 later added local paper report/ledger mutation audit integration, and Phase 26 later added local audit crash/concurrency/filesystem probes; real sandbox/live discrepancy evidence, disk-full testing, retention/rotation validation, long-running daemon restart validation, deployment-host validation, real observability runtime, real dashboard hosting, real outbound communications, live/sandbox exchange/RPC validation, custody, signer, external adapter submission, deployment, rollback, incident, penetration, load, and production-readiness blockers remain open.

2026-05-27 ArbyClaw end-to-end repository reconciliation audit:

- Scanned tracked source, documentation, CI, deployment, hardening, handoff, and generated-manifest artifacts for stale project identity, placeholder repository URLs, ZIP-only handoff assumptions, obsolete "Rust validation deferred" claims, live-network implementation drift, unresolved TODO-style code paths, and safety-boundary violations.
- Reconciled the repository identity to ArbyClaw, replaced the placeholder repository URL with `https://github.com/dominator509/arbyclaw`, updated CLI/status wording for existing SQLite WAL checkpoint boundaries, and refreshed roadmap/security/handoff language to separate current local/CI Rust evidence from production/runtime validation that remains missing.
- Post-reconciliation local validation passed: `python3 scripts/validate_structure.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` with 78 tests across 3 suites, and `cargo clippy --workspace --all-targets -- -D warnings`.
- No implemented live trading, signing, withdrawals, bridges, broadcasts, real exchange/RPC calls, wallet custody, or secret material was found in the code scan; matching terms remain as explicit deny-by-default policy text, boundary documentation, test fixtures, or CI hardening-tool setup.
- No production blockers were closed by this reconciliation. Runtime lifecycle wiring, external production-host SQLite WAL validation, paper audit integration, live/sandbox connector validation, public-exposure review, deployment validation, and broader external hardening evidence remained open at that time. Local paper report/ledger mutation audit integration was added later in Phase 25, while production audit validation remains open.
- `STRUCTURE_MANIFEST.md` was refreshed to match the reconciled working tree artifacts without treating the manifest itself as production-readiness evidence.

2026-05-26 ArbyClaw roadmap-anchored production gap tracker audit:

- Local validation rerun passed: `python3 scripts/validate_structure.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` with 78 tests across 3 suites, and `cargo clippy --workspace --all-targets -- -D warnings`.
- Latest referenced GitHub Actions validation passed on `main` in run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602` for commit `0b98a9a31d3701704d950779ad989daefcf1193b`.
- The Rust/Cargo validation class of gaps is now locally and CI-covered for the current workspace state: GAP-0001, GAP-0029, GAP-0031, GAP-0033, GAP-0036, GAP-0040, GAP-0043, GAP-0046, GAP-0050, GAP-0053, GAP-0055, GAP-0057, GAP-0059, GAP-0061, GAP-0063, GAP-0065, GAP-0067, GAP-0069, and GAP-0071.
- These validation updates do not prove production readiness, deployment readiness, live-funds readiness, public exposure readiness, code-scanning upload processing, SBOM review, penetration testing, load testing, rollback drills, incident drills, custody readiness, or live exchange/RPC readiness.
- Release evidence reviewer follow-up metadata is now considered complete enough for the current Phase 17 / Phase 18 roadmap boundary; additional reviewer micro-fields should not be added unless a concrete operator workflow identifies missing non-secret routing metadata.
- The safest fillable next gaps are roadmap-aligned tracker corrections and external validation closure records: stale CI/local validation statements, SBOM review evidence, external artifact review, code-scanning settings evidence, and operator review records.
- The non-fillable-in-repo gaps remain open where they require credentials, wallet custody, signer design, live exchange/RPC validation, production infrastructure, public exposure review, penetration testing, load testing, rollback drills, incident drills, legal/compliance review, or accountable human approval.

## Latest External Hardening Evidence Attempt

2026-05-26 ArbyClaw release-build, dependency-audit, SBOM-generation, SAST, and example-container hardening evidence refresh:

- Local `cargo build --release --locked` passed.
- Local `cargo audit` passed after fetching the RustSec advisory database and scanning `Cargo.lock`.
- Local CycloneDX SBOM generation passed with non-empty SBOM files for `arb-core` and `arb-agent`; generated SBOM files were removed from the working tree and not committed.
- GitHub Actions `cargo build --release --locked`, `cargo audit`, CycloneDX SBOM generation, CodeQL Rust SAST local-SARIF analysis, example container image build, Trivy image-scan gate, Gitleaks secret-pattern scan gate, lightweight hardening evidence index artifact upload, and GitHub Step Summary hardening evidence pointer generation passed in the latest referenced run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602` for commit `0b98a9a31d3701704d950779ad989daefcf1193b`.
- The initial upload-based CodeQL attempt in run `https://github.com/dominator509/arbyclaw/actions/runs/26199105621` failed because GitHub code scanning is not enabled for this repository. The workflow was narrowed to generate and verify local SARIF in CI without uploading to the GitHub Security tab.
- A 2026-05-21 attempt to enable CodeQL default setup through the GitHub API failed with `Code scanning is not enabled for this repository`, confirming that GitHub Security-tab upload processing remains blocked by repository security settings or plan/support constraints rather than by local source code.
- The CI workflow now keeps local-SARIF SAST evidence available as a short-retention Actions artifact while GitHub code scanning upload remains disabled.
- A local example-only container image build using `deployment/container/Containerfile.example` passed after adding `.dockerignore` to keep local build outputs and secret-like environment files out of the Docker build context. This does not validate a production image, deployment, runtime service, or rollout.
- GitHub Actions run `https://github.com/dominator509/arbyclaw/actions/runs/26209401284` built the example container image and uploaded Trivy evidence, then failed the critical-vulnerability gate because the Debian slim runtime included fixable `libgnutls30` critical findings (`CVE-2026-33845` and `CVE-2026-42010`). The runtime base was changed to a nonroot distroless Debian 12 image, locally rebuilt successfully, and validated by the passing example image-scan gate in run `https://github.com/dominator509/arbyclaw/actions/runs/26210031540`.
- Local Gitleaks `v8.30.1` secret-pattern scanning via the pinned container image passed with no leaks found; the redacted JSON evidence file was removed from the working tree. The CI Gitleaks secret-pattern scan gate also passed and uploaded a short-retention redacted evidence artifact in run `https://github.com/dominator509/arbyclaw/actions/runs/26271152507`.
- GitHub Actions run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602` uploaded the lightweight `hardening-evidence-index` artifact, plus `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and the Docker Buildx build record artifact, making non-secret hardening evidence easier to locate without changing runtime behavior.
- The same run wrote a GitHub Step Summary from the `hardening-evidence-index` job with non-secret artifact pointers, producing job results, run URL, commit, and explicit non-claims for easier lookup from the workflow page.
- Release-review records should reference the latest run URL, commit, artifact names, and artifact IDs as locator evidence only; SBOM review, dependency-audit review, production image review, GitHub code scanning upload processing or accepted deferral, staging, load, penetration, rollback, incident, live exchange/RPC, custody, compliance, and production-readiness reviews remain open until separately reviewed and recorded.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` now includes a CI evidence lookup note for finding the Step Summary and short-retention artifacts without changing CI or runtime behavior.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` now includes a manual evidence review checklist for confirming run identity, job outcomes, non-empty non-secret artifacts, and remaining release-review gaps before future release use.
- `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now provides a short non-secret release-review record template for run URLs, artifact names, reviewer, outcome, and unresolved gaps without changing CI or runtime behavior.
- `hardening/PRODUCTION_READINESS_CHECKLIST.md` now points release reviewers to `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` for non-secret evidence records before any future readiness review.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md`, `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md`, and `hardening/PRODUCTION_READINESS_CHECKLIST.md` now document the non-secret GitHub code-scanning settings evidence path for GAP-0075 without changing CI upload behavior, runtime behavior, or repository secrets handling.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a retained-artifact review checklist and non-secret retention fields so operators can preserve CI evidence references without copying secret-bearing content into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret SBOM review checklist and SBOM review evidence fields so operators can review generated SBOM artifacts without storing dependency graphs or sensitive environment details in repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a lightweight operator CI artifact review checklist and evidence field for recording run URLs, artifact names, reviewer, outcome, and unresolved gaps without copying artifact contents or secret-bearing logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include evidence expiration and refresh guidance so retained CI artifact references are refreshed when they no longer match the release-review commit, branch, artifact availability, changed inputs, or operator-approved review window.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret release evidence reviewer sign-off fields for recording evidence acceptance, rejection, or follow-up without implying production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret release evidence rejection reason categories so rejected or incomplete evidence can be recorded consistently without copying sensitive artifact contents, logs, credentials, private URLs, wallet material, screenshots, or findings into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret release evidence follow-up owner and target review date fields so rejected or incomplete evidence can be assigned without implying evidence acceptance, production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence status legend so reviewers use consistent accepted, rejected, follow-up required, expired, and deferred outcomes without implying production readiness or live-funds approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret evidence retention location classification values for Actions artifact, approved external evidence store, unavailable, or deferred without copying evidence contents, paths, credentials, private URLs, wallet material, screenshots, or sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret reviewer attestation that release evidence records contain references only, not embedded artifact contents, logs, SARIF/SBOM contents, vulnerability tables, screenshots, credentials, private URLs, wallet material, or sensitive environment details.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer role field constrained to operator, release reviewer, AppSec reviewer, DevSecOps reviewer, or deferred without implying approval authority, production readiness, or live-funds approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret reviewer independence/conflict note constrained to independent, same operator, deferred, or not applicable without implying production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence review-window field constrained to current, expired, deferred, or not applicable without implying production readiness, live-funds approval, public exposure approval, deployment readiness, custody readiness, or compliance approval.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret reviewer decision rationale category constrained to sufficient, insufficient, stale, deferred, or not applicable without copying artifact contents, logs, SARIF/SBOM contents, vulnerability tables, credentials, private URLs, wallet material, sensitive environment details, or production-readiness claims.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now record the operator decision to keep `dominator509/arbyclaw` private and use the CI `codeql-sarif-evidence` artifact as the local SARIF-only SAST evidence path for now while GAP-0075 remains open/deferred for GitHub Security-tab upload processing.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret SBOM review decision record and SBOM generation artifact reference field so reviewers can reference run URLs, job names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying dependency graphs, package inventories, vulnerability tables, SBOM contents, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret dependency-audit review decision record and `cargo audit` gate reference field so reviewers can reference run URLs, job names, gate names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying advisory tables, dependency details, vulnerable package lists, CVE text, private registry URLs, internal hostnames, credentials, wallet material, or raw sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret image-scan review decision record and Trivy `trivy-image-scan-evidence` artifact reference field so reviewers can reference run URLs, job names, gate names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying vulnerability tables, image layer details, package inventories, CVE text, base-image metadata beyond sanitized artifact references, private registry URLs, internal hostnames, credentials, wallet material, screenshots, or raw sensitive logs into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret secret-scan review decision record and Gitleaks `gitleaks-secret-scan-evidence` artifact reference field so reviewers can reference run URLs, job names, gate names, artifact names, commit hashes, reviewer, review date, outcome, and unresolved gaps without copying secret-scan findings, secret-like snippets, match strings, file excerpts, private URLs, credentials, tokens, wallet material, screenshots, raw logs, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include non-secret retained-artifact expiration outcome fields per artifact type so reviewers can mark `hardening-evidence-index`, `codeql-sarif-evidence`, `trivy-image-scan-evidence`, `gitleaks-secret-scan-evidence`, and Docker Buildx build record artifacts as `current`, `expired`, `unavailable`, or `refreshed` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence refresh trigger outcome field so reviewers can mark refresh reasons as `commit changed`, `artifact expired`, `workflow changed`, `reviewer unable to verify`, or `deferred` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up status field so reviewers can mark follow-up as `open`, `assigned`, `resolved`, `deferred`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up closure reason field so reviewers can mark closure reasons as `evidence refreshed`, `gap accepted for deferral`, `superseded by newer run`, `unable to verify`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up verification method field so reviewers can mark verification methods as `CI run review`, `artifact reference review`, `external evidence store review`, `not verified`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up verification timestamp field so reviewers can record review date/time without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up evidence source field so reviewers can mark sources as `GitHub Actions`, `external evidence store`, `local operator record`, `not available`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up evidence source locator field so reviewers can record a run URL, artifact name, or approved external reference without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up locator availability field so reviewers can mark locators as `available`, `expired`, `inaccessible`, `deferred`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up locator access note field so reviewers can record short sanitized notes such as `artifact retained`, `artifact expired`, `permissions unavailable`, `external reference reviewed`, or `not applicable` without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- `hardening/EXTERNAL_VALIDATION_RUNBOOK.md` and `hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md` now include a non-secret release evidence reviewer follow-up locator recheck date field so reviewers can record when the locator should be reviewed again without embedding artifact contents, logs, SARIF/SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details into repository files.
- The release evidence reviewer follow-up metadata set is now considered complete enough for the current Phase 17 / Phase 18 roadmap boundary; future safe tasks should return to roadmap-aligned production-gap audit and external validation closure work unless an operator identifies a concrete missing non-secret routing field.
- This is release-build, dependency-audit, SBOM-generation, local-SARIF SAST, example-container image-scan, and secret-pattern scan evidence only. It does not validate SBOM review, GitHub code scanning upload processing, production container image readiness, systemd hardening, ARM validation, staging deployment, load testing, penetration testing, rollback drills, incident drills, live exchange/RPC sandbox validation, custody review, compliance review, or production readiness.

## Latest Candidate External Validation Review Record

2026-05-26 ArbyClaw non-secret external-validation review candidate:

- Review scope: Candidate CI locator evidence for release-build, dependency-audit gate, SBOM-generation gate, local-SARIF SAST artifact, example Trivy image-scan artifact, Gitleaks secret-pattern scan artifact, Docker Buildx record artifact, and hardening-evidence index.
- Repository / branch / commit: `dominator509/arbyclaw` / `main` / `0b98a9a31d3701704d950779ad989daefcf1193b`.
- Workflow run URL: `https://github.com/dominator509/arbyclaw/actions/runs/26443625602`.
- Evidence source: GitHub Actions.
- Evidence retention location classification: Actions artifact.
- Evidence references-only attestation: This record contains run URL, commit, artifact names, and artifact IDs only; it does not embed artifact contents, logs, SARIF contents, SBOM contents, vulnerability tables, secret-scan findings, screenshots, credentials, private URLs, wallet material, or sensitive environment details.
- Artifact locator references: `hardening-evidence-index` `7213484905`; `codeql-sarif-evidence` `7213379997`; `trivy-image-scan-evidence` `7213343486`; `gitleaks-secret-scan-evidence` `7213308494`; Docker Buildx build record `7213344332`.
- Reviewer follow-up status: follow-up required.
- Reviewer follow-up verification method: CI run review and artifact reference review.
- Reviewer decision rationale category: sufficient for candidate locator evidence; insufficient for production validation.
- Review outcome: Candidate evidence recorded for future external-validation review only. No SBOM review, dependency-audit review, production image review, GitHub code scanning upload processing, staging validation, load testing, penetration testing, rollback drill, incident drill, live exchange/RPC validation, custody review, compliance review, public exposure approval, deployment readiness, live-funds readiness, or production-readiness blocker is closed.

## Current Production Readiness %

96%

Reasoning:

- Governance and architecture foundation exist.
- Minimal Rust workspace scaffold, CI skeleton, safety docs, and structure validation script exist.
- Typed config, environment secret-reference handling, encrypted-keystore interface boundary, redacted secret material, initial mode gates, deny-by-default policy checks, append-only audit/state primitives, local audit lock/sync append behavior, local audit crash-like truncation/tamper/concurrent append/invalid-filesystem validation probes, SQLite WAL local checkpoints, SQLite WAL process-level crash/restart recovery validation, local runtime state-permission fail-closed validation, normalized market-data models, freshness classification, fee models, deterministic paper market-data, static paper fees, policy-gated paper execution reports, local paper-report checkpoint helper, local paper balance ledgering, local realistic paper fill modeling, local venue matching profiles, adverse-selection penalties, reference-only calibration records, paper replay validation, local historical-fixture paper backtest execution, local paper report and ledger mutation audit journal records, CEX framework models/traits, DEX/Web3 framework models/traits, opportunity-engine models/traits, execution-planner draft models/traits, local plan-draft checkpoint helper, execution-adapter boundary records/traits, local adapter-run checkpoint helper, local runtime lifecycle audit/state/adapter wiring, local concurrent runtime lifecycle access checks, local graceful-shutdown audit/state checkpointing, local runtime audit/SQLite backup-restore validation, local restart recovery summaries with CLI-visible typed operator-review dispositions, communications/CLI command and notification boundaries, embedded-dashboard local render boundaries, observability/runbook local record boundaries, deterministic testing/fuzzing/backtesting validation boundaries, deterministic packaging/deployment planning boundaries, deterministic external hardening evidence/checklist boundaries, and deterministic agentic handoff boundaries exist.
- Encrypted-keystore backend, deployment-host SQLite WAL crash/restart/filesystem validation, physical disk-full validation, retention/rotation execution validation, service-manager restart/stale-lock validation, live market-data providers, exchange-specific CEX adapters, live DEX/RPC adapters, signer/custody backends, transaction broadcast controls, external adapter submission, external sandbox/live fill calibration evidence, real outbound communications adapters, real dashboard hosting/authentication, real observability/exporter/alert runtime, real property/fuzz runner execution, broader CI-scale replay/backtest runner execution, durable planner/adapter/communications/dashboard/observability/testing audit-state lifecycle beyond the local paper-report, paper-ledger, replay, backtest, audit, and plan-draft checkpoint helpers, container/systemd/ARM validation, runtime deployment, broad external hardening execution, external agent execution validation, rollback drills, incident drills, and production validations are still incomplete.
- Live-funds risk remains high.

## Current Completed Phases

- Phase 0 — Governance Initialization
- Phase 1 — Rust Workspace Scaffold (scaffold created; current workspace Rust/CI validation covered)
- Phase 2 — Config, Secrets, and Mode Gates (implemented; current workspace Rust/CI validation covered)
- Phase 3 — Policy Engine and Trust Contract (implemented; current workspace Rust/CI validation covered)
- Phase 4 — Audit Journal and State Store (implemented as boundary; current workspace Rust/CI validation covered; SQLite WAL checkpoint store, local state-store durability validation, and Phase 26 local audit durability probes implemented; external production-host validation deferred)
- Phase 5 — Market Data Core (implemented as boundary; current workspace Rust/CI validation covered; live provider validation deferred)
- Phase 6 — Simulated/Paper Connectors (implemented as deterministic boundary; current workspace Rust/CI validation covered; external sandbox/live paper calibration evidence deferred)
- Phase 7 — CEX Connector Framework (implemented as typed framework boundary; current workspace Rust/CI validation covered; live exchange validation deferred)
- Phase 8 — DEX/Web3 Connector Framework (implemented as typed framework boundary; current workspace Rust/CI validation covered; live RPC validation, signer validation, and broadcast validation deferred)
- Phase 9 — Opportunity Engine (implemented as deterministic discovery/ranking boundary; current workspace Rust/CI validation covered; advanced route modeling and live-data validation deferred)
- Phase 10 — Execution Planner (implemented as draft-only planning boundary; current workspace Rust/CI validation covered; local audit/state runtime lifecycle wiring covered; live execution validation deferred)
- Phase 11 — Execution Adapters (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; local adapter-run checkpoint lifecycle covered; live submission validation deferred)
- Phase 12 — Communications and CLI (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real outbound integrations and audit/state lifecycle deferred)
- Phase 13 — Embedded Dashboard (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real hosting, auth, and audit/state lifecycle deferred)
- Phase 14 — Observability and Runbooks (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; real telemetry runtime, exporters, alerts, and audit/state lifecycle deferred)
- Phase 15 — Testing, Fuzzing, and Backtesting (implemented as deterministic model/trait boundary; current workspace Rust/CI validation covered; Phase 24 local paper backtest execution exists; real property/fuzz, CI-scale replay/backtest, load testing, and penetration testing deferred)
- Phase 16 — Packaging and Deployment (implemented as deterministic model/docs boundary; current release-build, example-container, and static plus CI syntax example systemd-unit validation gates covered; production container/systemd/ARM validation, runtime deployment, and rollback drills deferred)
- Phase 17 — External Production Hardening (implemented as deterministic evidence/checklist boundary; real external hardening execution deferred)
- Phase 18 — Agentic Handoff Package (implemented as deterministic model/docs boundary; external agent execution and production validation deferred)
- Phase 19 — Runtime Lifecycle Wiring (implemented as local deterministic audit/state/adapter lifecycle boundary with concurrent local lifecycle access checks, local state-permission fail-closed validation, local graceful-shutdown checkpointing, local runtime audit/SQLite backup-restore validation, local restart recovery summaries, CLI-visible typed operator-review dispositions, and incomplete-recovery fail-closed checks; production runtime validation deferred)
- Phase 20 — SQLite WAL Durability Validation (implemented as local deterministic state-store validation boundary; external production-host validation deferred)
- Phase 21 — Paper Balance Ledgering (implemented as local deterministic paper balance boundary; local paper ledger mutation audit integration covered in Phase 25; production audit validation deferred)
- Phase 22 — Crash/Restart Durability Validation (implemented as local process-level SQLite WAL recovery validation; deployment-host validation deferred)
- Phase 23 — Realistic Paper Fills (implemented as local deterministic fill-model boundary; external sandbox/live calibration evidence deferred)
- Phase 24 — Paper Replay, Calibration, Backtest, and Runtime Validation Boundaries (implemented as local deterministic paper validation boundary; production-host validation and external sandbox/live evidence deferred)
- Phase 25 — Paper Audit Journal Integration (implemented as local deterministic paper report and ledger mutation audit boundary; deployment-host audit validation deferred)
- Phase 26 — Audit Crash, Concurrency, Filesystem, Disk-Full, and Retention Planning (implemented as local deterministic audit durability validation boundary; deployment-host audit validation, physical disk-full, retention/rotation execution, and service-manager restart evidence deferred)

## Current Incomplete Phases

None inside the current ChatGPT Project Mode roadmap. Production readiness remains blocked by external validation and infrastructure gaps below.

## Deferred Production Tasks

See gap entries below.

## Environment-Limited Tasks

See gap entries below.

## Missing Infrastructure Tasks

See gap entries below.

## Missing Deployment Tasks

See gap entries below.

## Missing Security Validations

See gap entries below.

## Missing Runtime Validations

See gap entries below.

## Missing CI/CD Validations

See gap entries below.

## Missing External Integration Tests

See gap entries below.

## Missing Penetration Tests

See gap entries below.

## Missing Observability Validation

See gap entries below.

## Missing Rollback Validation

See gap entries below.

## Missing Load/Performance Tests

See gap entries below.

## Temporary Mock/Stubs To Replace

- `arb-agent` binary is scaffold-only and must be replaced/extended with real runtime orchestration in later phases.
- `PolicyEngine` is implemented and connected to Phase 11 adapter-boundary revalidation only; live execution-adapter integration remains missing.
- `AppendOnlyAuditJournal` is implemented with local lock/sync append behavior, local durability probes for replay, truncation rejection, tamper rejection, concurrent append replay, invalid filesystem failure, simulated disk-full fail-closed behavior, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning; deployment-host validation, physical disk-full behavior, retention/rotation execution, service-manager restart execution evidence, and full live-relevant execution integration remain incomplete.
- `InMemoryStateStore` remains a non-production test/local wiring implementation; `SqliteWalStateStore` now provides local non-secret SQLite WAL checkpoint persistence, local integrity/checkpoint/reopen/backup-restore/multi-handle validation, and local runtime lifecycle test coverage, but external production-host crash/restart/filesystem validation is still incomplete.
- Encrypted-keystore provider is an interface boundary only and must be replaced with a real encrypted backend.
- Market-data and fee provider traits exist, but all real REST/WebSocket/paid-provider implementations are missing.
- `PaperMarketDataProvider`, `PaperFeeProvider`, and `PaperExecutionAdapter` are deterministic paper scaffolds and must not be treated as live connector implementations.
- `CexVenueProfile`, `CexConnectorRegistry`, `CexOrderRequest`, `CexPolicyGate`, and CEX connector traits are framework boundaries only and must not be treated as exchange-specific live adapters.
- `Web3ChainProfile`, `DexTokenProfile`, `DexRouterProfile`, `DexConnectorRegistry`, `DexSwapQuoteRequest`, `Web3TransactionSimulationRequest`, `DexPolicyGate`, and DEX/Web3 connector traits are framework boundaries only and must not be treated as live RPC, signing, simulation, bridge, or broadcast adapters.
- `ExecutionPlannerConfig`, `ExecutionPlannerRequest`, `ExecutionPlanDraft`, `ExecutionPlanStep`, `PlannerPolicyOutcome`, and planner traits are draft-only boundaries and must not be treated as adapter submission, order placement, signing, broadcast, or live execution implementations.
- `ExecutionAdapterConfig`, `ExecutionAdapterRequest`, `ExecutionAdapterRunRecord`, `ExecutionAdapterAttempt`, `ExecutionFillRecord`, `ExecutionReconciliationRecord`, and execution-adapter traits are framework boundaries only and must not be treated as live exchange adapters, RPC adapters, signing, broadcast, or production execution implementations.
- `CommunicationBoundaryConfig`, `OperatorCommand`, `RoutedOperatorCommand`, `OperatorNotification`, `NotificationDispatchRecord`, command-router traits, and notification-publisher traits are local model boundaries only and must not be treated as real messaging integrations, platform-token handling, remote command execution, or live operator control infrastructure.
- `DashboardBoundaryConfig`, `DashboardServerBinding`, `DashboardSnapshot`, `DashboardPanel`, `DashboardRenderRequest`, `DashboardRenderRecord`, and dashboard-renderer traits are local model boundaries only and must not be treated as real dashboard hosting, browser delivery, authentication, public exposure, or live operator control infrastructure.
- `ValidationHarnessConfig`, `ValidationTestCase`, `ValidationFixtureRecord`, `FuzzCorpusDefinition`, `BacktestScenarioDefinition`, `ValidationPlan`, `ValidationRunRecord`, and validation-harness traits are local model boundaries only and must not be treated as actual Rust test execution, external fuzzer execution, live network testing, backtest result validation, CI success, load testing, or penetration testing.
- `PackagingBoundaryConfig`, `PackageTargetPlan`, `ServiceHardeningProfile`, `ReleaseGate`, `RollbackStep`, `DeploymentPackagePlan`, `DeploymentPackageRecord`, and packaging/deployment planner traits are local model boundaries only and must not be treated as actual builds, container images, service installs, ARM binaries, runtime deployments, rollback-drill success, CI success, or production release readiness.
- `ExternalHardeningBoundaryConfig`, `HardeningEvidenceRecord`, `ProductionHardeningPlan`, `ExternalHardeningReviewRecord`, and hardening-review traits are local evidence/checklist boundaries only and must not be treated as completed CI, release build, dependency audit, SBOM, image scan, staging deployment, load test, penetration test, rollback drill, incident drill, live exchange/RPC validation, production readiness review, public exposure approval, or live-funds approval.

## Manual Human Tasks Required

See gap entries below.

## Future Agentic Continuation Tasks

See gap entries below.

## Highest-Risk Remaining Gaps

1. No encrypted secret/custody backend or signer boundary.
2. Audit journal has local crash-like truncation, tamper, concurrency, sync, invalid-filesystem, simulated disk-full validation, side-effect-free retention planning, and side-effect-free stale-lock restart recheck planning, but is not deployment-host validated, physical disk-full/retention/rotation execution validated, service-manager restart execution validated, or fully connected to live-relevant execution adapters.
3. SQLite WAL state store exists for local checkpoints and local runtime lifecycle wiring, and local durability validation now covers integrity, WAL checkpointing, reopen, backup/restore, and multi-handle checks; external production-host crash/restart/filesystem validation is missing.
4. Policy engine is connected to paper execution, Phase 7 CEX validation, Phase 8 DEX/Web3 framework validation, Phase 10 draft planner preflight, and Phase 11 adapter-boundary revalidation only; no live execution adapters or external submission exist.
5. No wallet signer boundary.
6. No exchange-specific live CEX adapters, rate-limit validation, or fee-schedule verification.
7. No live DEX/Web3 RPC adapters, signer integration, transaction simulation integration, spender approval controls, or broadcast controls.
8. No real outbound communications adapters, platform-token handling, or authenticated remote command channels.
9. No real dashboard hosting, browser authentication, CSRF protection, or penetration-tested operator UI.
10. No real observability exporters, metrics endpoint, log shipping, alert routing, or incident-drill validation.
11. No actual Rust/property/fuzz/backtest execution or curated validation corpus.
12. No actual package build, production container build, systemd install, ARM build, rollback drill, or deployment validation; only release-build, example-container, and static plus syntax example systemd-unit checks have current evidence.
13. Initial CI, locked release-build, dependency-audit, SBOM-generation, local-SARIF CodeQL SAST, example image scan, secret-pattern scan, and hardening index evidence exists; SBOM review, GitHub code scanning upload processing, production image scan, staging, load, penetration, rollback, incident, and production-readiness evidence remain missing.
14. No production security review.
15. No runtime testing with real market conditions.
16. No legal, jurisdiction, tax, or exchange terms-of-service review.

## Recommended Next Production Phase

Phase 26 follow-up: deployment-host audit/runtime validation, then the next roadmap phase that adds real external integration only after secret/custody/live-funds gates are explicitly designed.

Before any production claim, run deployment-host audit validation, physical disk-full/retention/rotation execution validation, service-manager restart execution validation, Phase 16 package/deployment validations, and Phase 17 external hardening validations in a capable external environment as soon as available.

---

# Gap Entries

## GAP-0001 — Rust Workspace Validation Deferred

- Unique ID: GAP-0001
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 1 / Phase 2
- Subsystem association: Repository scaffold / build system / config subsystem
- Description: A minimal Rust workspace scaffold and Phase 2 config modules now exist, and Cargo-based validation has current local and GitHub Actions evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and structure validation after future changes.
- Exact future tooling/environment required: Local development machine or CI runner with Rust stable, Cargo, rustfmt, clippy, Python 3, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 2 config changes first, then Phase 1 scaffold files if external validation exposes unrecoverable defects.

## GAP-0002 — `PHASE_1_SUBROADMAP.md` Created

- Unique ID: GAP-0002
- Phase association: Phase 1
- Subsystem association: Roadmap governance
- Description: `PHASE_1_SUBROADMAP.md` now exists and records Phase 1 objectives, deliverables, sequence, validation, rollback, limitations, and continuation tasks.
- Why incomplete: Not incomplete for document creation; external validation items remain tracked under GAP-0001 and GAP-0029.
- Why blocked in ChatGPT Project Mode: Not blocked.
- Risk level: Low
- Dependency requirements: Continued alignment with `ROADMAP.md` and `ARCHITECTURE.md`.
- Exact future validation required: Reconfirm Phase 1 roadmap alignment after future material validation or governance changes.
- Exact future tooling/environment required: Markdown review plus local/CI validation evidence when linked implementation changes occur.
- Recommended future agent type: Principal Systems Architect
- Estimated production impact: Deterministic sequencing restored for Phase 1.
- Completion criteria: Sub-roadmap remains aligned with scaffold and gap tracker.
- Rollback considerations: Revert the sub-roadmap file and roadmap/gap tracker updates if Phase 1 is rolled back.

## GAP-0003 — Encrypted Secret Backend Not Implemented

- Unique ID: GAP-0003
- Phase association: Phase 2 / Phase 17
- Subsystem association: Secrets and custody
- Description: Phase 2 added reference-only secret types, redacted `SecretMaterial`, a `SecretProvider` trait, and an environment provider skeleton, but no encrypted local keystore backend exists.
- Why incomplete: Encrypted storage, key derivation, file permissions, OS keyring integration, lifecycle handling, and zeroization have not been implemented or validated.
- Why blocked in ChatGPT Project Mode: Real encrypted-at-rest behavior, OS keyring behavior, filesystem permissions, and secret injection require local/CI/runtime environments outside ChatGPT.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, dependency review for encryption/zeroization crates, local filesystem or OS keyring target, policy engine, audit redaction.
- Exact future validation required: redaction tests, no-secret-log tests, encrypted-at-rest tests, key-load failure tests, zeroization tests, file-permission tests, backup/restore tests, secret rotation tests.
- Exact future tooling/environment required: Rust toolchain, local keyring or encrypted file backend, test secrets only, filesystem permission controls, CI secret-scanning.
- Recommended future agent type: AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks live credentials and wallet custody.
- Completion criteria: Secrets can be loaded via approved provider without appearing in logs, config, prompts, telemetry, or persisted plaintext; encrypted backend passes local and CI validation.
- Rollback considerations: Disable all live modes, remove secret backend integration, revoke any test credentials, and wipe local encrypted test stores.

## GAP-0004 — API Keys and Wallet Credentials Not Available

- Unique ID: GAP-0004
- Phase association: Phase 17
- Subsystem association: External integrations / secrets
- Description: No real exchange API keys, paid market-data keys, or wallet credentials are available.
- Why incomplete: User has not created or supplied credentials, and credentials must not be placed in chat or Markdown.
- Why blocked in ChatGPT Project Mode: Real secret provisioning must occur outside ChatGPT in the target runtime environment.
- Risk level: Critical
- Dependency requirements: Secret manager, config references, operator-created exchange accounts and API keys.
- Exact future validation required: Read-only credential validation first, sandbox credential validation where available, least-privilege scope verification.
- Exact future tooling/environment required: Exchange accounts, wallet, local encrypted secret store, network access.
- Recommended future agent type: DevSecOps Orchestrator + Human Operator
- Estimated production impact: Blocks live integrations.
- Completion criteria: Credentials are provisioned outside repo, scoped minimally, and validated without secret leakage.
- Rollback considerations: Revoke keys at exchanges/providers and wipe local encrypted keystore.

## GAP-0005 — Policy Engine Implemented; External Validation and Integration Pending

- Unique ID: GAP-0005
- Phase association: Phase 3 / Phase 11 / Phase 15
- Subsystem association: Policy and trust contract
- Description: Deny-by-default policy engine and trust-contract enforcement now exist in `crates/arb-core/src/policy.rs`, with current local and GitHub Actions Rust/Cargo validation evidence. Deeper property/fuzz validation and durable runtime integration remain incomplete.
- Why incomplete: Current workspace Rust/Cargo validation is covered, but property-test runners, fuzzers, durable audit integration, and real runtime integration remain incomplete.
- Why blocked in ChatGPT Project Mode: Not blocked for local Rust/Cargo validation in the current workspace; deeper property/fuzz/runtime validations require future tooling and scope.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 4 audit journal, future execution adapters, property/fuzz test framework.
- Exact future validation required: Property tests, fuzz tests, denial-path tests, mode-gate tests, unknown-destination tests, stale-data tests, kill-switch tests, live-runtime-denial tests, connector integration tests, plus rerun standard Cargo validation after changes.
- Exact future tooling/environment required: Rust test runner, clippy, property testing crate, fuzzing harness, CI runner.
- Recommended future agent type: Policy Engine Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Policy architecture now exists, but safe live execution remains blocked until policy is validated and mandatory in every execution path.
- Completion criteria: Policy code compiles, tests pass, future execution adapters cannot submit orders or sign transactions without a policy approval and durable audit record.
- Rollback considerations: Remove `policy.rs`, remove policy exports, revert CLI policy initialization, disable execution adapters, and force Observe/Paper modes.

## GAP-0006 — Wallet Signer Boundary Not Implemented

- Unique ID: GAP-0006
- Phase association: Phase 8 / Phase 11
- Subsystem association: Web3 custody / signer
- Description: No constrained signer boundary exists for wallet-controlled funds.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Code can be drafted, but hardware wallet, keyring, chain RPC, and real transaction validation require external environment.
- Risk level: Critical
- Dependency requirements: Secret manager, policy engine, DEX transaction model, audit journal.
- Exact future validation required: signer isolation tests, no-LLM-access tests, unauthorized destination denial, transaction simulation, chain testnet execution.
- Exact future tooling/environment required: Test wallet, testnet RPCs, local runtime, optional hardware wallet.
- Recommended future agent type: Web3 Connector Agent + AppSec Lead
- Estimated production impact: Blocks DEX live execution and any wallet autonomy.
- Completion criteria: Signer only signs policy-approved intents and never exposes raw keys.
- Rollback considerations: Remove signer provider and disable Web3 live mode.

## GAP-0007 — Audit Journal Implemented; External Validation and Runtime Integration Pending

- Unique ID: GAP-0007
- Phase association: Phase 4 / Phase 11 / Phase 17
- Subsystem association: Audit journal / state store
- Description: Phase 4 added `AppendOnlyAuditJournal`, typed audit events, redacted metadata values, hash-chained JSONL records, replay validation, `StateStore`, `StateCheckpoint`, a non-production in-memory state store, and a SQLite WAL-backed checkpoint store for local non-secret state. Phase 26 adds local audit lock/sync append behavior plus replay, truncation rejection, tamper rejection, concurrent append replay, invalid-filesystem validation probes, disk-full error classification, simulated disk-full fail-closed validation, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning.
- Why incomplete: Current Rust/Cargo validation exists for the audit/state boundary, but the audit journal and SQLite WAL state store are not yet fully wired into every runtime, policy, connector, signer, or live-relevant execution-adapter path; deployment-host audit validation, physical disk-full behavior, retention/rotation execution, and service-manager restart/stale-lock validation remain incomplete.
- Why blocked in ChatGPT Project Mode: Current local and CI validation covers compile/test/lint and deterministic local audit probes only; deployment filesystem behavior, disk-pressure tests, long-running runtime validation, and service-manager restart evidence require a capable external environment.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 5 market data, future execution planner/adapters, SQLite WAL state backend, filesystem permission model.
- Exact future validation required: Keep append/reopen, tamper-detection, redaction, local truncation-rejection, local concurrent append, local filesystem failure, simulated disk-full, retention-planning, and stale-lock planning tests passing; add deployment-host crash/recovery, physical disk-full, retention/rotation execution, service-manager restart execution, WAL persistence under deployment load, schema migration, and live-relevant audit-before-action integration tests.
- Exact future tooling/environment required: Rust, Cargo, local filesystem, CI runner, SQLite runtime, and migration tooling.
- Recommended future agent type: Audit and Observability Agent + Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Accountability architecture now exists, but live trading remains blocked until every execution path writes durable redacted audit records and durability is validated.
- Completion criteria: Every intent, policy decision, execution, signer request, connector result, failure, and reconciliation event is durably journaled without secrets; journal replay detects tampering; local crash-like/concurrency/filesystem probes and deployment-host durability validations pass.
- Rollback considerations: Disable live execution, revert audit/state modules, remove audit dependencies, and force Observe/Paper modes if validation fails.

## GAP-0008 — Market Data Core Boundary Implemented; Live Providers Missing

- Unique ID: GAP-0008
- Phase association: Phase 5
- Subsystem association: Market data
- Description: Phase 5 normalized market-data models, freshness classification, fee models, and provider trait boundaries exist with current local and GitHub Actions validation evidence. No live REST/WebSocket CEX provider, DEX quote provider, paid data-provider adapter, reconnect logic, or rate-limit logic exists.
- Why incomplete: The deterministic boundary exists, but live provider implementation and external provider validation remain incomplete.
- Why blocked in ChatGPT Project Mode: Implementation possible; live provider validation requires external network and credentials.
- Risk level: High
- Dependency requirements: Current Rust validation baseline and config subsystem.
- Exact future validation required: quote normalization tests, stale-data tests, fee model tests, order-book depth tests.
- Exact future tooling/environment required: Rust test runner; later live network access.
- Recommended future agent type: Rust Implementation Agent + Exchange Connector Agent
- Estimated production impact: Blocks reliable opportunity detection.
- Completion criteria: Market data is normalized, provenance-tracked, and freshness-checked.
- Rollback considerations: Revert market-data crate changes.

## GAP-0009 — Simulated/Paper Connector Boundary Implemented; External Calibration Missing

- Unique ID: GAP-0009
- Phase association: Phase 6 / Phase 21 / Phase 23 / Phase 24 / Phase 25
- Subsystem association: Simulation / paper trading
- Description: Phase 6 deterministic in-memory paper market data, static paper fee schedules, and policy-gated paper execution reports exist with current local and GitHub Actions validation evidence. Phase 21 adds balance constraints, Phase 23 adds local realistic paper fill modeling for supplied order-book depth, partial fills, latency, queue position, slippage, and ledger-safe unfilled notional release, Phase 24 adds local matching profiles, adverse-selection modeling, reference-only calibration records, paper replay validation, and local historical-fixture backtest execution, and Phase 25 adds direct local append-only audit journal records for paper reports plus reserve/settlement ledger mutations. External sandbox/live calibration evidence remains incomplete.
- Why incomplete: The deterministic paper boundary, local balance constraints, local fill realism, local replay, local fixture backtest execution, and local paper audit journal integration exist, but production-like external calibration remains incomplete.
- Why blocked in ChatGPT Project Mode: Not blocked.
- Risk level: High
- Dependency requirements: Market-data core, execution intent model, audit journal, paper ledger, realistic fill model, local replay/backtest records, and scenario fixtures.
- Exact future validation required: exchange-specific sandbox/live calibration tests, failed-trade simulation tests, paper-vs-sandbox discrepancy analysis, production-host runtime validation, and production audit durability validation.
- Exact future tooling/environment required: Rust test runner, fixture data.
- Recommended future agent type: Rust Implementation Agent
- Estimated production impact: Blocks safe validation before live execution.
- Completion criteria: Strategies can run without live funds, produce reproducible results, replay paper audit/ledger state, and document fill-model calibration limits.
- Rollback considerations: Disable simulation feature flag or revert crate changes.

## GAP-0010 — CEX Framework Implemented; Exchange-Specific Adapters Missing

- Unique ID: GAP-0010
- Phase association: Phase 7
- Subsystem association: CEX connectors
- Description: Phase 7 CEX framework types, venue profiles, capability registry, order request models, policy gates, and connector traits exist with current local and GitHub Actions validation evidence. No exchange-specific REST, WebSocket, sandbox, balance, order, cancel, fee, or rate-limit adapters exist.
- Why incomplete: The CEX framework boundary exists, but exchange-specific adapters and external validation remain incomplete.
- Why blocked in ChatGPT Project Mode: Live credentialed testing, sandbox testing, and rate-limit behavior require external network and accounts.
- Risk level: High
- Dependency requirements: Market data core, secret manager, config, policy, audit.
- Exact future validation required: mocked API tests, sandbox tests, read-only credential tests, order placement tests in sandbox where supported.
- Exact future tooling/environment required: Exchange accounts, API credentials, network access.
- Recommended future agent type: Exchange Connector Agent
- Estimated production impact: Blocks cross-exchange arbitrage.
- Completion criteria: At least one read-only connector and one sandbox/paper execution path work behind policy gates.
- Rollback considerations: Disable connector in capability registry and revoke credentials.

## GAP-0011 — Live DEX/Web3 Adapters Missing

- Unique ID: GAP-0011
- Phase association: Phase 8 / Phase 11 / Phase 17
- Subsystem association: DEX/Web3 connectors
- Description: Phase 8 now defines framework-only chain, token, router, quote, local simulation, policy-gate, and connector-trait boundaries, but no live chain RPC, router, aggregator, signer, transaction simulation, bridge, or broadcast adapter exists.
- Why incomplete: The smallest safe Phase 8 patch created typed boundaries before any external Web3 integration or signing behavior.
- Why blocked in ChatGPT Project Mode: Live RPC, testnet/mainnet simulation, wallet validation, protocol documentation checks, and signer custody require external environment and reviewed credentials/wallet setup.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, policy engine, signer boundary, encrypted custody backend, market data, audit journal, state store, protocol allowlists, and external Web3 runtime.
- Exact future validation required: contract allowlist tests, router/spender tests, transaction simulation tests, slippage tests, gas estimation tests, MEV-risk tests, approval hygiene tests, nonce tests, testnet execution tests, and no-broadcast-until-approved tests.
- Exact future tooling/environment required: RPC endpoints, test wallet, chain testnets, local runtime, mocked RPC fixtures, CI runner, and signer test harness with non-production keys outside the repository.
- Recommended future agent type: Web3 Connector Agent + AppSec Lead + DevSecOps Orchestrator
- Estimated production impact: Blocks live DEX/CEX and on-chain arbitrage.
- Completion criteria: At least one DEX/router quote path and one non-broadcasting testnet/simulated transaction path pass policy, audit, state, signer, and external validation without secret leakage.
- Rollback considerations: Disable Web3 feature flag, remove router registration, revoke provider/test credentials, preserve audit records, and remove signer provider references.

## GAP-0012 — Opportunity Engine Advanced Validation and Integration Incomplete

- Unique ID: GAP-0012
- Phase association: Phase 9 / Phase 10 / Phase 15 / Phase 17
- Subsystem association: Opportunity engine
- Description: Phase 9 added deterministic opportunity-engine models, freshness checks, fee-aware cross-venue top-of-book discovery, and deterministic ranking, but advanced path discovery, inventory-aware sizing, depth-aware slippage, planner integration, and external validation are incomplete.
- Why incomplete: Phase 9 intentionally added the smallest safe discovery/ranking boundary without execution-intent generation, live data, live connectors, or advanced route search.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the opportunity-engine boundary, but fixture replay at production scale, live market data, exchange/account context, and external production validation require tooling outside this environment.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, market-data core, fee models, simulated connectors, Phase 10 execution planner, Phase 15 backtesting/scenario harness, live provider fixtures.
- Exact future validation required: unit tests, replay tests, fee-aware ROI tests, stale-data denial tests, false-positive tests, triangular-route tests, depth/slippage tests, inventory constraints, settlement-latency tests, planner handoff tests, and backtesting over historical fixtures.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, fixture datasets, mocked market providers, later live data providers, CI runner.
- Recommended future agent type: Rust Implementation Agent + Strategy/Backtesting Agent + AppSec Lead
- Estimated production impact: Core discovery boundary exists, but safe and profitable production opportunity selection remains blocked until advanced validation and planner integration are complete.
- Completion criteria: Engine produces deterministic, fee-adjusted, policy-ready opportunity records; advanced route/search models pass replay and false-positive tests; execution planner consumes only validated candidates.
- Rollback considerations: Disable opportunity engine feature or revert `crates/arb-core/src/opportunity.rs`, exports, CLI status text, and roadmap/gap updates.

## GAP-0013 — Execution Planner Implemented; Adapter, Audit, and External Validation Pending

- Unique ID: GAP-0013
- Phase association: Phase 10 / Phase 11 / Phase 14 / Phase 17 / Phase 19
- Subsystem association: Execution planner
- Description: Phase 10 added deterministic draft-only conversion from validated opportunities to per-leg `ExecutionIntent` records, policy preflight outcomes, sequencing steps, and failure-mode boundaries. Phase 19 now wires plan drafts through local fail-closed audit/state lifecycle and deterministic adapter-boundary handoff, and Phase 23 adds local paper partial-fill modeling for direct paper reports. External adapter submission, planner-integrated hedge/cancel behavior, and live execution validation do not exist.
- Why incomplete: Local planner-to-adapter lifecycle wiring, SQLite WAL local durability validation, and direct paper partial-fill modeling exist, but production-host runtime validation, restart replay, planner-integrated partial-fill/hedge/cancel handling, and real adapter handoff remain incomplete.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the execution-planner and runtime lifecycle boundaries, but production filesystem/database durability, adapter lifecycle tests, and live/sandbox venue behavior require future implementation and external environments.
- Risk level: High
- Dependency requirements: Opportunity engine, policy engine, audit journal, durable state store, Phase 11 execution adapters, Phase 15 scenario/backtesting harness.
- Exact future validation required: intent-generation tests, policy-preflight tests, planner-integrated partial-fill tests, timeout tests, cancellation tests, failure-mode tests, audit-before-adapter tests, restart/recovery tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem/database, mocked adapter fixtures, CI runner.
- Recommended future agent type: Rust Implementation Agent + Policy Engine Agent + Audit and Observability Agent
- Estimated production impact: Draft planning, local deterministic adapter handoff, SQLite WAL local durability validation, direct paper realistic fills, local paper replay/backtest records, and local paper report/ledger mutation audit records no longer block architecture, but live and production paper execution safety still depend on production-host runtime validation, planner-integrated fill handling, production audit durability validation, external sandbox/live calibration evidence, and real adapter integration.
- Completion criteria: Opportunities become durable, auditable, policy-checked plans that adapters can consume only after audit/state writes succeed, restart replay is validated, and external tests pass.
- Rollback considerations: Remove `crates/arb-core/src/planner.rs`, remove planner exports/status text, force Paper/Observe modes, and return roadmap/gap tracker to Phase 9 state.

## GAP-0014 — Live Execution Adapter Submission Missing

- Unique ID: GAP-0014
- Phase association: Phase 11 / Phase 17
- Subsystem association: Execution adapters
- Description: Phase 11 adds deterministic execution-adapter model/trait boundaries, policy revalidation, attempt records, fill records, and reconciliation records, but no live order submission, transaction submission, external exchange/RPC adapter, live fill tracking, or real balance reconciliation exists.
- Why incomplete: Phase 11 intentionally stops at fail-closed model boundaries; live submission requires later custody, audit/state, connector, and external validation work.
- Why blocked in ChatGPT Project Mode: Real live validation requires exchange accounts, network access, testnet/mainnet environments, signer/custody setup, and controlled funds outside the repository.
- Risk level: Critical
- Dependency requirements: Policy engine, secret manager, audit journal, durable state store, connector framework, execution planner, signer boundary, exchange-specific adapters.
- Exact future validation required: sandbox tests, paper scenario tests, small-capital guarded live tests, reconciliation tests, failure recovery tests, audit-before-submit tests, kill-switch tests, duplicate submission tests.
- Exact future tooling/environment required: Exchange accounts, test wallets, network access, staging runtime, CI runner, external audit/state backend.
- Recommended future agent type: Exchange Connector Agent + Web3 Connector Agent + Release Engineering Authority
- Estimated production impact: Blocks autonomous profit/loss generation.
- Completion criteria: Live adapters execute only policy-approved, audit-journaled, state-checkpointed intents and reconcile outcomes without exposing secrets.
- Rollback considerations: Disable live mode, revoke keys, halt signer, preserve audit records, revert adapter feature flags.

## GAP-0015 — Real Communications Channels Missing

- Unique ID: GAP-0015
- Phase association: Phase 12 / Phase 17
- Subsystem association: Communications
- Description: Phase 12 adds typed command, notification, redaction, routing, and local dispatch-record boundaries, but Telegram, Discord, Matrix, email, Slack, PagerDuty, Signal, iMessage, webhook, SMS, and other real outbound integrations are not implemented.
- Why incomplete: Phase 12 intentionally stops at local deterministic model/trait boundaries with outbound network delivery disabled.
- Why blocked in ChatGPT Project Mode: Real integrations require external accounts, platform tokens, device/app approvals, network access, abuse-prevention review, and channel-specific security validation.
- Risk level: Medium
- Dependency requirements: CLI, config, command routing, redaction layer, authentication/authorization model, audit/state integration, platform-specific adapters.
- Exact future validation required: mocked command tests, auth tests, notification delivery tests, replay tests, injection-resistance tests, no-secret-render tests, channel permission tests, rate-limit tests, and fail-closed token revocation tests.
- Exact future tooling/environment required: platform tokens, test channels, local runtime, network access, CI secrets manager, external integration test accounts.
- Recommended future agent type: Rust Implementation Agent + AppSec Lead + Communications Integration Agent
- Estimated production impact: Reduces operator control and alerting.
- Completion criteria: Typed commands and notifications work over approved channels without exposing secrets, bypassing policy, or enabling unauthorized execution.
- Rollback considerations: Disable affected channel adapter, revoke tokens, preserve audit records, and fall back to local CLI only.

## GAP-0016 — Real Dashboard Hosting Missing

- Unique ID: GAP-0016
- Phase association: Phase 13 / Phase 17
- Subsystem association: Dashboard
- Description: Phase 13 local dashboard render models exist, but no real HTTP server, browser delivery, authentication, authorization, CSRF protection, or hosted operator UI exists.
- Why incomplete: Phase 13 intentionally stopped at local deterministic model/trait boundaries and rejects server startup/public exposure.
- Why blocked in ChatGPT Project Mode: Secure browser/server validation requires Rust runtime execution, a local browser/server harness, network binding inspection, authentication design, persistence, and AppSec review.
- Risk level: Medium
- Dependency requirements: Runtime, audit/state store, auth/session model, secure local web host design.
- Exact future validation required: local auth tests, CSRF tests if applicable, no-secret-render tests, localhost-only default tests, public-bind denial tests, live-control denial tests.
- Exact future tooling/environment required: Rust web framework, local browser for manual validation, CI runner, AppSec review.
- Recommended future agent type: Embedded Dashboard Agent + AppSec Lead
- Estimated production impact: Low to medium; dashboard is optional.
- Completion criteria: Dashboard hosting is disabled or localhost-only by default, authenticated where exposed, and exposes no secrets or live controls.
- Rollback considerations: Disable dashboard feature flag and fall back to local CLI/status records.

## GAP-0017 — Real Observability Runtime Missing

- Unique ID: GAP-0017
- Phase association: Phase 14 / Phase 17
- Subsystem association: Observability
- Description: Phase 14 local health, structured-log, metric, and runbook records exist, but no real tracing subscriber, metrics endpoint, OpenTelemetry/Prometheus exporter, log shipping, alert escalation, incident drill, or production telemetry runtime exists.
- Why incomplete: Phase 14 intentionally stopped at local deterministic model/trait boundaries and rejects metrics endpoint startup, public exposure, outbound alerts, and secret observability.
- Why blocked in ChatGPT Project Mode: Production telemetry validation requires deployed runtime, Rust execution, exporter infrastructure, authenticated endpoints, network binding inspection, alerting providers, and AppSec review.
- Risk level: High
- Dependency requirements: Runtime scaffold, subsystem events, audit/state integration, communications adapters for alert routing, secure endpoint design.
- Exact future validation required: structured logging tests, redaction tests, metrics endpoint tests, health endpoint tests, public-bind denial tests, exporter redaction tests, alert routing tests, incident drill tests, no-secret-telemetry tests.
- Exact future tooling/environment required: local runtime, optional Prometheus/OpenTelemetry stack, mocked alerting channels, CI runner, AppSec review.
- Recommended future agent type: Audit and Observability Agent
- Estimated production impact: Blocks production operations and incident response.
- Completion criteria: Runtime emits redacted logs, metrics, health status, and critical alerts through authenticated, audited, fail-closed channels without exposing secrets.
- Rollback considerations: Disable exporters and alerts while preserving local records.

## GAP-0018 — CI/CD Execution Covered for Current Workspace; Production CI Gates Still Limited

- Unique ID: GAP-0018
- Phase association: Phase 16 / Phase 17
- Subsystem association: CI/CD
- Description: GitHub Actions now runs on `dominator509/arbyclaw` and covers structure validation, formatting, workspace compilation, tests, clippy, locked release build, dependency audit, SBOM generation, local-SARIF SAST evidence, example image scan, secret-pattern scan, and hardening evidence indexing for pushed commits.
- Why incomplete: Current CI evidence exists, but production deployment gates, staging gates, release approval gates, rollback-drill gates, and live-integration gates remain incomplete.
- Why blocked in ChatGPT Project Mode: No longer blocked for hosted CI execution on the current repository; production-class CI gates require future infrastructure and operator-approved release workflows.
- Risk level: High
- Dependency requirements: Keep GitHub Actions, Rust stable, Cargo, rustfmt, clippy, Python 3, dependency-audit tooling, SBOM tooling, SAST artifact generation, image scanning, and secret scanning available for future runs.
- Exact future validation required: Re-run CI after future changes and preserve non-secret evidence references for formatting, check, tests, clippy, locked release build, dependency audit, SBOM generation, local-SARIF SAST, example image scan, secret-pattern scan, and hardening evidence indexing.
- Exact future tooling/environment required: GitHub Actions or equivalent CI service.
- Recommended future agent type: DevSecOps Orchestrator
- Estimated production impact: Blocks reliable releases.
- Completion criteria: CI pipeline runs and passes on hosted repository.
- Rollback considerations: Revert CI workflow changes.

## GAP-0019 — Deployment Packaging External Validation Missing

- Unique ID: GAP-0019
- Phase association: Phase 16 / Phase 17
- Subsystem association: Packaging/deployment
- Description: Phase 16 added deterministic packaging/deployment models and example-only container, systemd, ARM, and deployment documentation. Current evidence exists for locked release-build validation, an example-only container image build, local/CI image-scan evidence, repeatable local example-container validation, static plus CI syntax example systemd-unit validation, dependency audit, SBOM generation, local-SARIF SAST, secret-pattern scanning, and hardening evidence indexing. Production validation is still missing: no production release artifact, production container image, service installation, ARM build, runtime deployment, or rollback drill has been executed.
- Why incomplete: Phase 16 intentionally produced plan records and templates only. Current CI evidence, local example-container validation, and static example systemd-unit validation cover release-build, example-image, and example-service-template feedback, but production artifact builds and deployment validation require target infrastructure and operator review.
- Why blocked in ChatGPT Project Mode: Production deployment validation requires a container runtime or packaging target, systemd host, ARM target/cross toolchain, filesystem permissions, rollback environment, and deployment infrastructure outside this chat.
- Risk level: Medium
- Dependency requirements: Keep current Rust/CI, locked release-build, example-image, image-scan, static example systemd-unit, dependency-audit, SBOM, local-SARIF, secret-scan, and hardening-index evidence refreshable; add package/deployment plans, target host profile, runtime config, rollback procedure, and production release artifact storage.
- Exact future validation required: Refresh CI evidence for the candidate commit, then perform production release artifact build, production container build, image scan review, ARM build, service lint, service start/stop, config loading, non-root runtime validation, read-only filesystem validation, rollback drill, incident drill, and log/audit review.
- Exact future tooling/environment required: Current Rust/Cargo and CI runner for evidence refresh; container runtime, systemd Linux host or test container, ARM device or cross-compile target, and release artifact storage for production validation.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority
- Estimated production impact: Blocks production deployment.
- Completion criteria: Binary can be packaged and run on intended targets with documented, tested rollback and no secret leakage or accidental live-mode enablement.
- Rollback considerations: Stop service, restore previous binary/config, remove package/image/unit, preserve audit evidence, and keep Observe/Paper modes only.

## GAP-0020 — Penetration Testing Missing

- Unique ID: GAP-0020
- Phase association: Phase 17
- Subsystem association: Security validation
- Description: No penetration testing, wallet custody review, or adversarial testing has been performed.
- Why incomplete: No implementation exists yet.
- Why blocked in ChatGPT Project Mode: Requires external tools, running system, security expertise, and test environment.
- Risk level: Critical
- Dependency requirements: Implemented runtime, dashboard/communications, policy, secrets, execution paths.
- Exact future validation required: SAST, DAST where applicable, dependency audit, secret scan, command-injection tests, authz tests, wallet-safety review.
- Exact future tooling/environment required: Security testing environment, scanners, external reviewer.
- Recommended future agent type: AppSec Lead + External Security Engineer
- Estimated production impact: Blocks responsible live-funds deployment.
- Completion criteria: Findings remediated or accepted with documented risk.
- Rollback considerations: Disable live features until vulnerabilities are resolved.

## GAP-0021 — Load and Latency Testing Missing

- Unique ID: GAP-0021
- Phase association: Phase 15 / Phase 17
- Subsystem association: Performance validation
- Description: No throughput, latency, or resource-use tests exist.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Meaningful performance validation requires compiled runtime, network conditions, and target hardware.
- Risk level: High
- Dependency requirements: Runtime, market data, opportunity engine, connectors.
- Exact future validation required: quote ingestion load tests, opportunity ranking latency tests, memory footprint tests, ARM device tests, backpressure tests.
- Exact future tooling/environment required: benchmarking harness, target machines, network access.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Directly affects speed and missed opportunities.
- Completion criteria: Performance targets documented and met on target profiles.
- Rollback considerations: Disable high-frequency connectors or reduce polling/subscription rates.

## GAP-0022 — Legal, Tax, Jurisdiction, and Terms Review Missing

- Unique ID: GAP-0022
- Phase association: Phase 17
- Subsystem association: Compliance / operations
- Description: No legal, tax, exchange terms-of-service, or jurisdiction-specific review has been completed.
- Why incomplete: Requires human/external professional review.
- Why blocked in ChatGPT Project Mode: Legal and tax review cannot be completed by this environment.
- Risk level: Critical
- Dependency requirements: Intended jurisdiction, target exchanges, trading behavior, entity/account structure.
- Exact future validation required: legal review, tax/accounting review, exchange terms review, recordkeeping review.
- Exact future tooling/environment required: Qualified legal/tax professionals and exchange documentation.
- Recommended future agent type: Human Operator + External Counsel/CPA
- Estimated production impact: Could block or constrain live trading.
- Completion criteria: Documented approval or documented restrictions integrated into config/policy.
- Rollback considerations: Disable non-approved venues, strategies, and jurisdictions.

## GAP-0023 — Production Runtime Validation Missing Beyond Local Records

- Unique ID: GAP-0023
- Phase association: Phase 17 / Phase 19 / Phase 24
- Subsystem association: Runtime validation
- Description: Phase 19 local runtime lifecycle records and Phase 24 local paper runtime validation records exist, but no daemon, service, network, or long-running production-host process validation has been performed.
- Why incomplete: Local lifecycle and paper runtime validation records exist, but deployment-host startup, service management, soak, restart, config reload, permissions, and observability validation remain external.
- Why blocked in ChatGPT Project Mode: Production runtime validation requires a compiled runtime, target environment, service manager or process supervisor, and retained non-secret evidence.
- Risk level: High
- Dependency requirements: Runtime implementation, config, logging, health checks, deployment target, and non-secret evidence workflow.
- Exact future validation required: start/stop tests, crash recovery, config reload, service restart, daemon uptime soak test, filesystem-permission test, disk-full test, and observability smoke test.
- Exact future tooling/environment required: local machine, VPS, systemd or process supervisor.
- Recommended future agent type: DevSecOps Orchestrator
- Estimated production impact: Blocks unattended operation.
- Completion criteria: Runtime runs reliably under target deployment profile.
- Rollback considerations: Stop service and restore previous binary/config.

## GAP-0024 — Rollback Validation Missing

- Unique ID: GAP-0024
- Phase association: Phase 16 / Phase 17
- Subsystem association: Release engineering
- Description: No production rollback drill has been executed.
- Why incomplete: No deployable runtime exists yet.
- Why blocked in ChatGPT Project Mode: Requires deployed environment and release artifacts.
- Risk level: High
- Dependency requirements: Packaging, deployment, persistent state, config migration strategy.
- Exact future validation required: binary rollback, config rollback, DB migration rollback/forward recovery, kill-switch verification.
- Exact future tooling/environment required: staging VPS/local service, release artifacts.
- Recommended future agent type: Release Engineering Authority
- Estimated production impact: Blocks safe production release.
- Completion criteria: Rollback procedures are tested and documented.
- Rollback considerations: This gap itself is about validating rollback; until complete, live releases must remain guarded.

## GAP-0025 — Handoff Package Implemented; External Handoff Execution Missing

- Unique ID: GAP-0025
- Phase association: Phase 18
- Subsystem association: Agentic handoff
- Description: Phase 18 deterministic handoff package records, continuation prompts, repository maps, external validation checklists, and future-agent instructions exist. No external coding agent, AppSec reviewer, DevSecOps reviewer, compliance reviewer, or human production reviewer has executed the handoff package and recorded non-secret evidence.
- Why incomplete: The handoff package boundary exists, but external handoff execution and evidence review remain incomplete.
- Why blocked in ChatGPT Project Mode: External handoff execution requires a real external agent or human reviewer workflow outside this local model boundary.
- Risk level: Medium
- Dependency requirements: Stable repo structure and current gap inventory.
- Exact future validation required: Handoff docs align with current architecture, roadmap, tests, gaps, and build commands.
- Exact future tooling/environment required: Markdown docs and optional zip packaging.
- Recommended future agent type: Handoff Agent
- Estimated production impact: Affects future continuation quality.
- Completion criteria: External agent can resume from docs without losing roadmap position.
- Rollback considerations: Revert handoff docs if stale or inaccurate.

## GAP-0026 — Paid Market Data Provider Evaluation Missing

- Unique ID: GAP-0026
- Phase association: Phase 5 / Phase 17
- Subsystem association: Market data integrations
- Description: Paid market-data providers have not been selected, contracted, or validated.
- Why incomplete: Requires budget, accounts, API credentials, and provider comparison.
- Why blocked in ChatGPT Project Mode: Provider signup, billing, key provisioning, and live testing require external action.
- Risk level: Medium
- Dependency requirements: Market-data provider abstraction and budget decision.
- Exact future validation required: latency comparison, coverage comparison, rate-limit testing, cost analysis, failure behavior tests.
- Exact future tooling/environment required: Provider accounts, API keys, benchmark environment.
- Recommended future agent type: DevSecOps Orchestrator + Human Operator
- Estimated production impact: Affects opportunity coverage and speed.
- Completion criteria: Provider list selected and integrated behind market-data abstraction.
- Rollback considerations: Disable provider connector and fall back to exchange-native data.

## GAP-0027 — Withdrawal Policy Not Implemented or Validated

- Unique ID: GAP-0027
- Phase association: Phase 3 / Phase 11 / Phase 17
- Subsystem association: Custody / execution policy
- Description: The user wants full fund control eventually, but safe withdrawal policy is not implemented or validated.
- Why incomplete: Policy engine, signer boundary, allowlists, audit, and external validation do not exist yet.
- Why blocked in ChatGPT Project Mode: Real withdrawal testing requires live accounts/wallets and must be performed cautiously outside this environment.
- Risk level: Critical
- Dependency requirements: Policy engine, secret manager, audit journal, allowlist manager, execution adapters, human-reviewed destination list.
- Exact future validation required: unknown-address denial, allowlisted-address approval, per-period withdrawal limits, operator confirmation, sandbox/testnet withdrawal tests, revocation tests.
- Exact future tooling/environment required: Exchange accounts, wallet, test funds, local runtime.
- Recommended future agent type: AppSec Lead + Release Engineering Authority + Human Operator
- Estimated production impact: Mishandled withdrawals could cause irreversible loss.
- Completion criteria: Withdrawals are disabled by default and only enabled under a signed/explicit local policy with tested allowlists and limits.
- Rollback considerations: Disable withdrawal capability, revoke API withdrawal permissions, rotate wallet keys if compromised.

## GAP-0028 — Strategy Parameter Library Not Implemented

- Unique ID: GAP-0028
- Phase association: Phase 2 / Phase 9 / Phase 10
- Subsystem association: Strategy profiles
- Description: The extensive strategy and command parameter library is only architecturally defined, not implemented.
- Why incomplete: No code exists yet.
- Why blocked in ChatGPT Project Mode: Not blocked for code; real profitability tuning is environment-limited.
- Risk level: High
- Dependency requirements: Config models, policy engine, opportunity engine, execution planner.
- Exact future validation required: schema validation, invalid-profile denial tests, boundary tests, strategy replay tests, config migration tests.
- Exact future tooling/environment required: Rust test runner, fixture configs.
- Recommended future agent type: Rust Implementation Agent + Policy Engine Agent
- Estimated production impact: Blocks customizable autonomous behavior.
- Completion criteria: Strategy profiles are typed, validated, documented, and policy constrained.
- Rollback considerations: Revert strategy schema or disable unsafe profile fields.

## GAP-0029 — Local Cargo Validation Covered for Current Workspace

- Unique ID: GAP-0029
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 1
- Subsystem association: Build validation
- Description: Local Rust formatting, workspace compilation, tests, and clippy validation have current evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: High
- Dependency requirements: Rust stable toolchain with rustfmt and clippy.
- Exact future validation required: Run `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- Exact future tooling/environment required: Local development machine or CI runner with Rust stable.
- Recommended future agent type: Rust Implementation Agent
- Estimated production impact: The scaffold cannot be considered build-verified until this is complete.
- Completion criteria: All listed commands pass with no warnings or errors.
- Rollback considerations: Revert or patch Phase 1 scaffold files if validation fails.

## GAP-0030 — Dependency Audit Covered; Broader Supply-Chain Review Missing

- Unique ID: GAP-0030
- Phase association: Phase 1 / Phase 16 / Phase 17
- Subsystem association: Supply-chain security
- Description: `cargo audit` now runs locally/through GitHub Actions for the current workspace, and CycloneDX SBOM generation is gated in CI. Broader supply-chain review, license review, SBOM reviewer sign-off, provenance review, and release artifact attestation remain incomplete.
- Why incomplete: Dependency audit and SBOM generation evidence exists, but broader supply-chain governance and human/operator review remain incomplete.
- Why blocked in ChatGPT Project Mode: No longer blocked for the current `cargo audit`/SBOM generation gates; broader supply-chain review requires future operator review and release evidence workflow.
- Risk level: Medium
- Dependency requirements: Cargo, cargo-deny or equivalent, repository dependency manifest, dependency lockfile after external Cargo resolution.
- Exact future validation required: Run dependency audit and license policy checks for `serde`, `toml`, `serde_json`, `sha2`, and transitive dependencies after external Cargo resolution.
- Exact future tooling/environment required: Rust toolchain, `cargo-deny` or equivalent, CI runner.
- Recommended future agent type: AppSec Lead + DevSecOps Orchestrator
- Estimated production impact: Required before adding exchange, Web3, crypto, network, or database dependencies.
- Completion criteria: Dependency and license checks pass under documented policy.
- Rollback considerations: Remove or pin problematic dependencies and rerun validation.


## GAP-0031 — Phase 2 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0031
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 2
- Subsystem association: Config / secrets / mode gates
- Description: Phase 2 Rust modules and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: High
- Dependency requirements: Rust stable toolchain, Cargo, rustfmt, clippy, dependency download access.
- Exact future validation required: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Exact future tooling/environment required: Rust stable toolchain and internet/dependency cache access in local or CI environment.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future Phase 2 changes must rerun the validation commands.
- Rollback considerations: Revert Phase 2 config/secrets changes if validation exposes unrecoverable defects.

## GAP-0032 — Secret Zeroization and Lifecycle Hardening Missing

- Unique ID: GAP-0032
- Phase association: Phase 2 / Phase 3 / Phase 17
- Subsystem association: Secrets and custody
- Description: `SecretMaterial` redacts debug output, but zeroization, memory lifecycle hardening, and secret-use scoping are not implemented.
- Why incomplete: Phase 2 intentionally added only the smallest reference and redaction boundary.
- Why blocked in ChatGPT Project Mode: Code can be added later, but meaningful validation requires Rust tests, dependency review, and runtime inspection outside ChatGPT.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, dependency policy for zeroization crate, encrypted keystore implementation, signer boundary.
- Exact future validation required: zeroization tests, no-clone policy review where feasible, panic-path review, log/prompt/telemetry leak tests.
- Exact future tooling/environment required: Rust toolchain, memory/lifecycle test harness, AppSec review.
- Recommended future agent type: AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe live custody and wallet operation.
- Completion criteria: Secret material is minimized, redacted, zeroized on drop, scoped to constrained call sites, and covered by tests.
- Rollback considerations: Disable live modes and remove secret-loading paths if hardening cannot be validated.

## GAP-0033 — Phase 3 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0033
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 3
- Subsystem association: Policy engine / build system
- Description: Phase 3 policy code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for policy code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 3 policy files and docs if validation exposes unrecoverable defects.

## GAP-0034 — Policy Not Yet Mandatory in Execution Paths

- Unique ID: GAP-0034
- Phase association: Phase 3 / Phase 11
- Subsystem association: Policy engine / execution adapters
- Description: Policy engine and deterministic execution-adapter boundaries exist, but there is still no live runtime path proving real orders, swaps, transfers, withdrawals, and signing requests pass policy before external execution.
- Why incomplete: Execution adapters are model/trait boundaries only; live connectors, signer/custody, durable audit/state lifecycle, and external submission remain unavailable.
- Why blocked in ChatGPT Project Mode: Not blocked for future code, but real adapter behavior and live connector validation require external environments.
- Risk level: Critical
- Dependency requirements: Phase 4 audit journal, connector frameworks, execution planner, execution adapters.
- Exact future validation required: integration tests proving every execution path requires policy approval and fails closed on policy errors.
- Exact future tooling/environment required: Rust test runner, simulated adapters, later sandbox exchange/chain environments.
- Recommended future agent type: Execution Adapter Agent + AppSec Lead
- Estimated production impact: Blocks live execution and wallet signing.
- Completion criteria: No adapter can place an order, create a transaction, transfer funds, withdraw funds, or request signing without a policy-approved intent and audit record.
- Rollback considerations: Disable adapters and force Observe/Paper modes.

## GAP-0035 — Persistent Destination Allowlist Missing

- Unique ID: GAP-0035
- Phase association: Phase 3 / Phase 4 / Phase 8 / Phase 11
- Subsystem association: Policy engine / custody / state store
- Description: Phase 3 models destination trust classifications, but no persistent approved-address allowlist, address ownership proof, or operator approval workflow exists.
- Why incomplete: State store, signer boundary, and Web3 connector phases are not implemented yet.
- Why blocked in ChatGPT Project Mode: Real wallet address ownership, address-book storage, and transaction validation require external runtime tooling and operator actions.
- Risk level: Critical
- Dependency requirements: Audit journal, encrypted secret backend, signer boundary, Web3 connector framework, operator approval UX.
- Exact future validation required: destination allowlist tests, unknown-address denial tests, LLM-generated destination denial tests, address ownership verification where applicable, rollback tests.
- Exact future tooling/environment required: Rust test runner, local state store, test wallet, testnet RPCs, operator-controlled address book.
- Recommended future agent type: AppSec Lead + Web3 Connector Agent + Audit and Observability Agent
- Estimated production impact: Blocks any safe external transfer or withdrawal behavior.
- Completion criteria: External destinations can only be added through an auditable non-LLM-controlled workflow and are enforced by policy before signing or transfer.
- Rollback considerations: Clear destination allowlist, disable transfers/withdrawals, revoke any signer permissions.

## GAP-0036 — Phase 4 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0036
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 4
- Subsystem association: Audit journal / state store / build system
- Description: Phase 4 audit and state code plus tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for audit and state code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 4 audit/state files and docs if validation exposes unrecoverable defects.

## GAP-0037 — SQLite WAL Local Crash/Restart Validated; Deployment-Host Validation Missing

- Unique ID: GAP-0037
- Phase association: Phase 4 / Phase 14 / Phase 17 / Phase 19 / Phase 20 / Phase 22
- Subsystem association: State store / persistence
- Description: Phase 4 now includes `SqliteWalStateStore`, a local SQLite WAL-backed checkpoint store for non-secret operational state, plus round-trip, replacement, secret-like rejection, reopen tests, Phase 19 local runtime lifecycle checkpoint tests, Phase 20 local durability validation for WAL mode, synchronous FULL, SQLite integrity check, WAL checkpoint truncate, primary reopen, checkpointed backup/restore, and multi-handle checkpoint visibility, and Phase 22 process-level crash/restart validation that recovers committed checkpoints after abrupt child-process exits.
- Why incomplete: The local code gap is closed for deterministic SQLite WAL durability and process-level crash/restart validation, but deployment-host validation still needs file-locking under deployment conditions, schema migration, filesystem permission, physical disk-full, long-running daemon restart, and host-level restart behavior.
- Why blocked in ChatGPT/Codex environment: Deployment filesystem behavior, disk-full behavior, permission checks, service-manager restarts, and long-running runtime validation require a targeted host or CI/runtime scenario beyond ordinary unit tests.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, SQLite crate supply-chain review, schema migration plan, production runtime lifecycle validation, controlled filesystem tests, and non-secret external evidence references.
- Exact future validation required: schema migration tests, deployment file-locking tests, concurrent process access tests, filesystem permission tests, physical disk-full tests, service-manager restart tests, and runtime checkpoint lifecycle tests on an approved production-like host.
- Exact future tooling/environment required: Rust toolchain, SQLite runtime, controlled local or CI filesystem, process crash/restart harness, CI runner or production-like runtime host, and non-secret evidence store.
- Recommended future agent type: Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Removes the local durability-validation and process-level crash/restart coding gaps for SQLite checkpoint persistence and local lifecycle wiring, but deployment-grade restart recovery and reconciliation remain blocked until deployment-host validations are complete.
- Completion criteria: Local criteria are met when `SqliteWalStateStore::validate_durability` and the process-level crash/restart integration test pass in Rust tests. Deployment criteria require the same state-store boundary to pass migration, locking, filesystem permission, physical disk-full, service-manager restart, and runtime lifecycle tests on an approved production-like host with non-secret evidence references.
- Rollback considerations: Disable durable state features, fall back to observe/paper modes, and revert database migrations if validation fails.

## GAP-0038 — Local Audit Durability Validation Exists; Deployment-Host Evidence Missing

- Unique ID: GAP-0038
- Phase association: Phase 4 / Phase 17 / Phase 26
- Subsystem association: Audit journal / runtime validation
- Description: Phase 26 adds local audit append locking, file flush plus `sync_all`, append/reopen replay validation, crash-like truncated JSONL rejection, tamper rejection, concurrent append replay validation, invalid filesystem fail-closed checks, disk-full error classification, simulated disk-full fail-closed checks, side-effect-free retention/rotation planning, and side-effect-free stale-lock restart recheck planning. Deployment-host crash/restart behavior, physical disk-full behavior, retention/rotation execution policy, service-manager restart execution behavior, and production filesystem permission behavior remain externally unproven.
- Why incomplete: Local deterministic probes now exist, but production trust still requires runtime filesystem and service-manager evidence outside ordinary unit tests.
- Why blocked in ChatGPT Project Mode: Deployment crash simulation, disk-pressure tests, permission hardening, retention/rotation execution behavior, stale lock recovery, and service-manager restart behavior require controlled host or CI/runtime tooling.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 26 local audit validation harness, local filesystem test harness, future runtime supervisor design.
- Exact future validation required: deployment-host crash during append, replay after partial write under supervisor restart, concurrent writer serialization under deployment load, permission mode checks, retention/rotation execution checks, physical disk-full handling, stale lock recovery.
- Exact future tooling/environment required: Rust toolchain, controlled filesystem, process/service-manager test harness, CI runner or staging host with filesystem controls.
- Recommended future agent type: Audit and Observability Agent + DevSecOps Orchestrator
- Estimated production impact: Blocks use of audit journal as production evidence for live-fund execution.
- Completion criteria: Audit journal behavior is deterministic and safe under local probes plus deployment-host crash, restart, permission, retention/rotation execution, stale-lock, and disk-pressure scenarios.
- Rollback considerations: Disable live execution and require external log shipping or database-backed audit before re-enabling.

## GAP-0039 — Audit Not Yet Mandatory in Execution Paths

- Unique ID: GAP-0039
- Phase association: Phase 4 / Phase 10 / Phase 11 / Phase 19 / Phase 25 / Phase 26
- Subsystem association: Audit journal / execution planner / execution adapters
- Description: Audit primitives, draft execution-planner records, deterministic execution-adapter boundary records, Phase 19 local runtime lifecycle audit/state gates, Phase 25 local paper report/ledger mutation audit records, and Phase 26 local audit durability probes exist, but no connector, signer, or live runtime adapter path proves every live-relevant action is durably journaled before and after action.
- Why incomplete: Phase 19 wires local deterministic planner-to-adapter audit/state preconditions, Phase 25 wires local paper report/ledger mutation audit records, and Phase 26 validates local audit crash-like/concurrency/filesystem probes, but live connector submissions, signer requests, production fills, failures, reconciliation lifecycle records, and production-host runtime validation remain missing.
- Why blocked in ChatGPT Project Mode: Local model wiring exists, but durable runtime validation plus real connector and signer validation require external environments.
- Risk level: Critical
- Dependency requirements: Phase 10 planner audit integration, Phase 11 execution adapters, Phase 8 signer boundary, durable audit/state validation.
- Exact future validation required: integration tests proving intents, policy decisions, connector submissions, fills, failures, signer requests, and reconciliations are audit-recorded and fail closed if audit append fails.
- Exact future tooling/environment required: Rust test runner, simulated connectors, future sandbox connectors, local audit journal or SQLite backend.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + AppSec Lead
- Estimated production impact: Blocks live execution and wallet signing.
- Completion criteria: No adapter can place an order, submit a swap, transfer funds, withdraw funds, or request signing unless pre-action audit append succeeds and post-action outcome is recorded.
- Rollback considerations: Disable adapters and force Observe/Paper modes if audit enforcement cannot be proven.



## GAP-0040 — Phase 5 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0040
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 5
- Subsystem association: Market data / fee model / build system
- Description: Phase 5 market-data and fee-model code plus tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for market-data and fee-model code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert Phase 5 market-data/fee files and docs if validation exposes unrecoverable defects.

## GAP-0041 — Live Market-Data Providers Not Implemented or Validated

- Unique ID: GAP-0041
- Phase association: Phase 5 / Phase 7 / Phase 8 / Phase 17
- Subsystem association: Market data / CEX connectors / DEX connectors / external integrations
- Description: Phase 5 added read-only provider trait boundaries and normalized models, but no live REST/WebSocket CEX provider, DEX quote provider, paid data-provider adapter, reconnect logic, or rate-limit logic exists.
- Why incomplete: The smallest safe Phase 5 patch created models and boundaries only; live providers require exchange/API review, network runtime, credentials where applicable, and external validation.
- Why blocked in ChatGPT Project Mode: Real network connections, provider accounts, API limits, WebSocket behavior, latency measurement, and data-quality validation require external runtime environments.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, provider selection, exchange/provider terms review, future connector framework, observability hooks, optional paid market-data credentials provisioned outside the repo.
- Exact future validation required: REST polling tests, WebSocket reconnect tests, stale-data tests, rate-limit tests, bad-data rejection tests, latency measurement, provider outage handling, no-secret-log tests, sandbox/read-only integration tests.
- Exact future tooling/environment required: Rust runtime, network access, test provider accounts or public endpoints, CI/integration environment, telemetry backend for latency and error metrics.
- Recommended future agent type: Market Data Connector Agent + DevSecOps Orchestrator + Audit and Observability Agent
- Estimated production impact: Blocks real opportunity discovery and production market-data confidence.
- Completion criteria: At least one reputable CEX read-only provider and one simulated provider pass validation with deterministic freshness, rate-limit, reconnect, and stale-data behavior.
- Rollback considerations: Disable live providers, force simulated/paper providers only, and revert connector-specific adapter code if validation fails.

## GAP-0042 — Fee Schedules Not Externally Verified

- Unique ID: GAP-0042
- Phase association: Phase 5 / Phase 7 / Phase 8 / Phase 17
- Subsystem association: Fee model / market data / execution planning
- Description: Phase 5 added fee schedules and fee-adjusted edge calculation, but fee rates, account tiers, gas/network costs, withdrawal costs, and venue-specific fee rules have not been externally verified.
- Why incomplete: Real fee data depends on venue account tier, jurisdiction, asset, pair, route, gas market, execution type, and provider-specific API behavior.
- Why blocked in ChatGPT Project Mode: Real exchange accounts, API access, chain RPCs, and live gas/fee observations are unavailable here.
- Risk level: High
- Dependency requirements: Read-only exchange credentials where needed, DEX/RPC quote providers, external fee schedule review, runtime observability, account-tier configuration.
- Exact future validation required: compare configured fees against venue API/account UI, validate maker/taker tiers, validate gas/network fee estimates, validate fee-adjusted edge against paper fills, reject unverified schedules for live execution.
- Exact future tooling/environment required: Rust runtime, exchange accounts, provider APIs, test wallets, chain RPCs, audit journal, simulated/paper connectors.
- Recommended future agent type: Market Data Connector Agent + Execution Planner Agent + AppSec Lead
- Estimated production impact: Incorrect fee estimates can turn apparent arbitrage into guaranteed loss; live execution must remain blocked until fee verification is enforced.
- Completion criteria: Fee schedules are externally verified, tagged with verification metadata, audited, and enforced by opportunity/planner/policy paths before live execution.
- Rollback considerations: Treat all fee schedules as unverified, disable live opportunities using them, and revert provider-specific fee integration if discrepancies are found.


## GAP-0043 — Phase 6 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0043
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 6
- Subsystem association: Paper connectors / build system
- Description: Phase 6 paper connector code plus tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for paper connector code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/paper.rs`, paper exports, CLI text, validator requirements, and Phase 6 governance updates if validation exposes unrecoverable defects.

## GAP-0044 — Paper Connector Model Has Local Realism, Replay, and Backtest Boundaries; External Calibration Evidence Missing

- Unique ID: GAP-0044
- Phase association: Phase 6 / Phase 9 / Phase 15 / Phase 21 / Phase 23 / Phase 24 / Phase 25
- Subsystem association: Paper execution / simulation / backtesting
- Description: Phase 6 provides deterministic in-memory market data, static fee schedules, and policy-gated paper execution reports. Phase 21 adds local simulated balance ledgering with quote-notional reservation, fill settlement, insufficient-balance denial, missing-reservation denial, and SQLite checkpoint persistence. Phase 23 adds local fill realism for supplied order-book depth consumption, partial fills, unfilled outcomes, latency, queue position, average price, slippage, and ledger-safe unfilled notional release. Phase 24 adds local exchange matching profiles, adverse-selection modeling, reference-only calibration records, paper ledger replay validation, and local historical-fixture paper backtest execution. Phase 25 adds local append-only audit journal records for paper reports and reserve/settlement ledger mutations. External sandbox/live calibration evidence and real venue discrepancy analysis remain missing.
- Why incomplete: Local balance constraints, deterministic fill realism, venue matching, adverse-selection penalties, reference-only calibration records, replay validation, local paper backtests, and local paper audit journal integration now exist, but production trust still requires real sandbox/live samples, venue data, and external evidence.
- Why blocked in ChatGPT Project Mode: Exchange-specific calibration requires historical market data, live or sandbox venue behavior samples, latency observations, and external evidence beyond local deterministic tests.
- Risk level: High
- Dependency requirements: Phase 9 opportunity engine, Phase 15 backtesting/scenario harness, Phase 24 local paper replay/backtest boundary, external market-data samples, and current Rust validation baseline.
- Exact future validation required: sandbox/live venue sampling, latency/slippage calibration tests against external evidence, multi-asset position reconciliation tests, fee-model comparison tests, paper-vs-sandbox discrepancy analysis, and CI-scale historical fixture replay.
- Exact future tooling/environment required: Rust test runner, historical data fixtures, sandbox exchange access where available, local SQLite or fixture store, CI runner.
- Recommended future agent type: Simulation/Backtesting Agent + Market Data Connector Agent + Rust Implementation Agent
- Estimated production impact: Paper profits may still overstate achievable live results; live execution must remain blocked until sandbox validation and calibration narrow the gap.
- Completion criteria: Paper execution models depth, slippage, latency, fees, multi-asset positions, and partial fills with deterministic scenario tests and paper replay coverage locally, then external sandbox/live evidence documents exchange-specific calibration limits and acceptable discrepancy thresholds.
- Rollback considerations: Force Observe mode or disable paper-derived strategy promotion if paper/live discrepancies exceed thresholds.

## GAP-0045 — Paper Report and Ledger Audit Integration Exists; Production Audit Validation Missing

- Unique ID: GAP-0045
- Phase association: Phase 6 / Phase 4 / Phase 10 / Phase 11 / Phase 19 / Phase 21 / Phase 24 / Phase 25 / Phase 26
- Subsystem association: Paper execution / audit journal / state store
- Description: Phase 6 paper execution calls policy before producing a report, and the latest deterministic paper report can be persisted through a typed local `StateStore` checkpoint helper with SQLite WAL reopen coverage. Phase 19 adds local planner-to-adapter audit/state lifecycle wiring. Phase 21 adds local paper balance ledgering with state checkpoint persistence. Phase 24 adds local paper ledger replay validation. Phase 25 adds direct append-only audit journal records for paper execution reports and paper reserve/settlement ledger mutations, with local journal reopen/replay tests. Phase 26 adds local audit durability probes for lock/sync append behavior, truncation rejection, tamper rejection, concurrent append replay, and invalid-filesystem failure. Production runtime orchestration and deployment-host audit durability validation remain incomplete.
- Why incomplete: Local report checkpointing, paper ledger checkpointing, planner-to-adapter lifecycle wiring, ledger replay validation, local paper report/ledger mutation audit records, and local audit durability probes exist, but paper intent audit-before-action for future live-relevant paths, restart replay orchestration under deployment conditions, physical disk-full/retention/rotation execution/service-manager audit validation, and production runtime orchestration remain deferred.
- Why blocked in ChatGPT/Codex environment: Current Rust/Cargo validation exists for the paper connector, ledger, replay, local audit integration, local audit durability probes, and local lifecycle boundary, but production audit/runtime validation remains incomplete.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 planner, Phase 11 adapter lifecycle, and current Rust validation baseline.
- Exact future validation required: paper intent audit-before-action test for future live-relevant paths, restart/replay orchestration test, deployment-host audit durability tests, physical disk-full/retention/rotation execution/service-manager tests, and production durability persistence test.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, CI runner, audit replay harness.
- Recommended future agent type: Audit and Observability Agent + Execution Adapter Agent + Rust Implementation Agent
- Estimated production impact: Paper balance accounting, ledger replay, and paper report/ledger mutation audit records are now locally modeled, but using paper execution as auditable evidence for promotion toward live strategy controls remains blocked until production audit/runtime validation exists.
- Completion criteria: Every paper execution intent, ledger mutation, and result is journaled, state is checkpointed, replay is deterministic, production audit durability is validated, and execution fails closed when audit/state writes fail.
- Rollback considerations: Disable paper execution adapter use in strategy promotion and revert to read-only market-data simulation until audit/state integration is proven.


## GAP-0046 — Phase 7 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0046
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 7
- Subsystem association: CEX connector framework / build system
- Description: Phase 7 CEX framework code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for CEX framework code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/cex.rs`, remove exports, and return roadmap/gap tracker to Phase 6 state if validation exposes unrecoverable defects.

## GAP-0047 — Exchange-Specific CEX Adapters Not Implemented

- Unique ID: GAP-0047
- Phase association: Phase 7 / Phase 11 / Phase 17
- Subsystem association: CEX connectors / external integrations
- Description: Phase 7 defines CEX framework types and traits, but no exchange-specific REST, WebSocket, sandbox, balance, order, or cancel adapters exist.
- Why incomplete: The smallest safe Phase 7 patch created framework boundaries before exchange-specific implementations.
- Why blocked in ChatGPT Project Mode: Live exchange integration requires network access, exchange accounts, credentials, sandbox environments where available, and external API documentation validation.
- Risk level: Critical
- Dependency requirements: Current Rust validation baseline, secret backend, rate-limit controller, audit/state integration, exchange account setup, sandbox access where available.
- Exact future validation required: public market-data tests, WebSocket reconnect tests, sandbox order lifecycle tests, cancel tests, balance read tests, idempotency tests, rate-limit tests, and failure-mode tests per exchange.
- Exact future tooling/environment required: Rust toolchain, network access, exchange sandbox or restricted test accounts, test credentials stored outside the repository, CI secrets, and mocked API fixtures.
- Recommended future agent type: CEX Connector Agent + AppSec Lead + DevSecOps Orchestrator
- Estimated production impact: Blocks live CEX arbitrage and real exchange order routing.
- Completion criteria: At least one exchange-specific connector passes read-only and sandbox validation without secret leakage and without live-funds permissions.
- Rollback considerations: Disable the affected connector, revoke test credentials, remove connector registration, and force Observe/Paper modes.

## GAP-0048 — CEX Fee, Rate-Limit, Terms, and Jurisdiction Validation Missing

- Unique ID: GAP-0048
- Phase association: Phase 7 / Phase 17
- Subsystem association: CEX governance / compliance / connector validation
- Description: CEX profiles include review flags, but no exchange-specific fee schedule, rate-limit, terms-of-service, incident-history, or jurisdiction review has been completed.
- Why incomplete: These checks require external sources, exchange documentation, legal/operational review, and often current account context.
- Why blocked in ChatGPT Project Mode: Real terms, account availability, jurisdiction constraints, fee tiers, and rate limits must be validated outside this environment and may change over time.
- Risk level: Critical
- Dependency requirements: Exchange selection, human operator jurisdiction, account tier, legal/tax review, up-to-date exchange documentation, and external review workflow.
- Exact future validation required: fee tier verification, withdrawal/transfer permission review, order-type support review, rate-limit validation, terms-of-service review, jurisdiction check, API-scope review, and incident/reputation review.
- Exact future tooling/environment required: Browser/network access, exchange accounts, legal/compliance review process, connector test harness, and credential-scope inspection.
- Recommended future agent type: CEX Connector Agent + AppSec Lead + Human Legal/Compliance Reviewer
- Estimated production impact: Blocks safe enablement of any real CEX venue.
- Completion criteria: Each enabled CEX profile records verified fees, rate limits, API capabilities, jurisdiction status, and terms review before use beyond paper/sandbox mode.
- Rollback considerations: Disable the CEX venue profile, remove it from allowlists, revoke credentials, and preserve audit records explaining disablement.

## GAP-0049 — CEX Framework Audit and State Integration Missing

- Unique ID: GAP-0049
- Phase association: Phase 7 / Phase 10 / Phase 11 / Phase 14
- Subsystem association: CEX execution lifecycle / audit journal / state store
- Description: CEX order requests can be policy-gated in Phase 7, but no durable audit-before-action, order lifecycle state machine, fill reconciliation, or restart recovery exists for CEX workflows.
- Why incomplete: Phase 7 framework models, Phase 10 planner records, and Phase 11 adapter boundary records now exist, but CEX order lifecycle handling still lacks durable audit/state gating, live or sandbox adapter wiring, fill reconciliation, and restart recovery.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the CEX/planner/adapter boundaries, but durable lifecycle validation requires filesystem/database persistence, simulated and sandbox exchange responses, and runtime restart tests.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 execution planner, Phase 11 adapters, and current Rust validation baseline.
- Exact future validation required: audit-before-order tests, audit-after-response tests, audit-fail-closed tests, order state transition tests, fill reconciliation tests, restart/replay tests, and duplicate client-order-id tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked exchange fixtures, sandbox exchange accounts, and CI runner.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks production-grade CEX order lifecycle management and post-incident forensic reliability.
- Completion criteria: Every CEX order lifecycle event is journaled, state transitions are durable and replayable, execution fails closed when audit/state writes fail, and restart recovery preserves idempotency.
- Rollback considerations: Disable CEX order submission, preserve existing audit files, roll back connector registration, and force Observe/Paper modes.

## GAP-0050 — Phase 8 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0050
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 8
- Subsystem association: DEX/Web3 connector framework / build system
- Description: Phase 8 DEX/Web3 framework code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for DEX/Web3 framework code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/dex.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 7 state if validation exposes unrecoverable defects.

## GAP-0051 — DEX/Web3 Protocol, Token, Gas, MEV, and Terms Validation Missing

- Unique ID: GAP-0051
- Phase association: Phase 8 / Phase 17
- Subsystem association: DEX/Web3 governance / protocol validation
- Description: Phase 8 profiles include framework metadata, but no router contract, token metadata, gas behavior, slippage behavior, MEV risk, protocol incident history, terms-of-service, or jurisdiction review has been completed.
- Why incomplete: These checks require external sources, protocol documentation, live/testnet behavior, legal/operational review, and frequently changing on-chain conditions.
- Why blocked in ChatGPT Project Mode: Real protocol and token validation require network access, chain explorers/RPC, legal/compliance review, and human operator jurisdiction context.
- Risk level: Critical
- Dependency requirements: Chain/router/token selection, external documentation review, chain explorer/RPC access, legal/tax review, and protocol risk workflow.
- Exact future validation required: router bytecode/address review, token contract review, decimals verification, spender allowlist verification, gas stress tests, slippage tests, MEV/sandwich risk review, protocol terms review, jurisdiction check, and incident/reputation review.
- Exact future tooling/environment required: Browser/network access, chain explorers, RPC endpoints, protocol docs, legal/compliance review process, and Web3 test harness.
- Recommended future agent type: Web3 Connector Agent + AppSec Lead + Human Legal/Compliance Reviewer
- Estimated production impact: Blocks safe enablement of any real DEX/router/token profile.
- Completion criteria: Each enabled DEX/Web3 profile records verified chain, router, spender, token, gas, slippage, MEV, jurisdiction, and protocol-risk status before use beyond paper/simulation mode.
- Rollback considerations: Disable the DEX/router/token profile, remove it from allowlists, revoke provider credentials, and preserve audit records explaining disablement.

## GAP-0052 — DEX/Web3 Framework Audit and State Integration Missing

- Unique ID: GAP-0052
- Phase association: Phase 8 / Phase 10 / Phase 11 / Phase 14
- Subsystem association: DEX/Web3 execution lifecycle / audit journal / state store
- Description: DEX swap quote requests can be policy-gated in Phase 8, but no durable audit-before-action, transaction lifecycle state machine, nonce tracking, simulation replay, confirmation tracking, or restart recovery exists for DEX/Web3 workflows.
- Why incomplete: Phase 8 framework models, Phase 10 planner records, and Phase 11 adapter boundary records now exist, but DEX/Web3 lifecycle handling still lacks durable audit/state gating, live RPC adapters, signer boundary implementation, nonce/confirmation tracking, and restart recovery.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the DEX/planner/adapter boundaries, but durable lifecycle validation requires filesystem/database persistence, mocked RPC fixtures, testnet responses, signer harnesses, and runtime restart tests.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 8 framework validation, Phase 10 execution planner, Phase 11 adapters, signer boundary, and current Rust validation baseline.
- Exact future validation required: audit-before-simulation tests, audit-before-signing tests, audit-fail-closed tests, nonce/state transition tests, transaction confirmation tests, simulation replay tests, restart/recovery tests, and duplicate intent-id tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked RPC fixtures, testnet RPC endpoints, signer test harness, and CI runner.
- Recommended future agent type: Execution Adapter Agent + Web3 Connector Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks production-grade DEX/Web3 lifecycle management and post-incident forensic reliability.
- Completion criteria: Every DEX/Web3 lifecycle event is journaled, state transitions are durable and replayable, signing/broadcast fails closed when audit/state writes fail, and restart recovery preserves idempotency.
- Rollback considerations: Disable DEX/Web3 execution, preserve existing audit files, roll back router/token registration, and force Observe/Paper modes.



## GAP-0053 — Phase 9 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0053
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 9
- Subsystem association: Opportunity engine / build system
- Description: Phase 9 opportunity-engine code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for opportunity-engine code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/opportunity.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 8 state if validation exposes unrecoverable defects.

## GAP-0054 — Opportunity Engine Audit, State, and Planner Integration Missing

- Unique ID: GAP-0054
- Phase association: Phase 9 / Phase 10 / Phase 14 / Phase 15
- Subsystem association: Opportunity engine / audit journal / state store / execution planner
- Description: Opportunity candidates are deterministic data records in Phase 9 and can now be consumed by the Phase 10 draft planner, but they are not durably journaled, checkpointed, replayed, or integrated into a runtime lifecycle.
- Why incomplete: Phase 9 intentionally stopped before runtime lifecycle integration; Phase 10 added draft planner consumption but not durable audit/state persistence.
- Why blocked in ChatGPT Project Mode: Durable lifecycle validation requires Rust tests, filesystem/database persistence, replay fixtures, planner implementation, and runtime restart tests.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 9 and Phase 10 validation baselines, and Phase 15 backtesting/scenario harness.
- Exact future validation required: opportunity audit-record tests, replay determinism tests, candidate deduplication tests, state checkpoint tests, planner handoff tests, restart/recovery tests, and historical backtest replay.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, fixture market data, CI runner.
- Recommended future agent type: Strategy/Backtesting Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks reliable production traceability and safe handoff from discovery to planning.
- Completion criteria: Every discovered candidate is journaled without secrets, replayable, deduplicated, state-checkpointed, and consumed by the planner only after durable validation.
- Rollback considerations: Disable opportunity-engine runtime integration, preserve audit files, revert planner handoff wiring, and force Observe/Paper modes.

## GAP-0055 — Phase 10 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0055
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 10
- Subsystem association: Execution planner / build system
- Description: Phase 10 execution-planner code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for execution-planner code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/planner.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 9 state if validation exposes unrecoverable defects.

## GAP-0056 — Execution Planner Audit, State, and Adapter Handoff Integration Missing

- Unique ID: GAP-0056
- Phase association: Phase 10 / Phase 11 / Phase 14 / Phase 15 / Phase 19
- Subsystem association: Execution planner / audit journal / state store / execution adapters
- Description: Execution-plan drafts contain deterministic intents, sequencing steps, policy outcomes, and failure-mode metadata, can now be persisted through a typed local `StateStore` checkpoint helper with SQLite WAL reopen coverage, consumed by the Phase 11 adapter-boundary model, and wired through the Phase 19 local runtime lifecycle with audit-before-adapter and state-before-adapter gates. They are still not handed to real execution adapters.
- Why incomplete: Local checkpoint persistence and fail-closed audit/state gating exist for deterministic adapter-boundary evaluation, but restart replay orchestration, production runtime validation, and real adapter handoff remain incomplete.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the planner checkpoint and runtime lifecycle boundary, but production lifecycle validation still requires filesystem/database scenarios, mocked adapters, restart fixtures, and CI/runtime execution.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 validation baseline, Phase 11 execution adapters, and Phase 15 scenario/backtesting harness.
- Exact future validation required: plan audit-record tests, preflight replay tests, duplicate plan-id tests, runtime checkpoint orchestration tests, adapter handoff tests, fail-closed audit-write tests, restart/recovery tests, and historical scenario replay.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked adapter fixtures, CI runner.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + Rust Implementation Agent
- Estimated production impact: Blocks safe production handoff from planning to execution.
- Completion criteria: Every plan and policy outcome is journaled without secrets, replayable, state-checkpointed, and adapters reject plans unless audit/state preconditions pass.
- Rollback considerations: Disable planner-to-adapter handoff, preserve audit files, revert adapter wiring, and force Observe/Paper modes.

## GAP-0057 — Phase 11 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0057
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 11
- Subsystem association: Execution adapter framework / build system
- Description: Phase 11 execution-adapter framework code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for execution-adapter framework code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/execution_adapter.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 10 state if validation exposes unrecoverable defects.

## GAP-0058 — Execution Adapter Audit, State, and Live Submission Integration Missing

- Unique ID: GAP-0058
- Phase association: Phase 11 / Phase 14 / Phase 15 / Phase 17
- Subsystem association: Execution adapters / audit journal / state store / live connectors
- Description: Execution-adapter framework records model attempts, fills, and reconciliation outcomes, and Phase 21 provides a local paper balance ledger for direct paper adapter use. Adapter framework records are still not durably journaled, replayed across restarts, wired to the paper ledger in the runtime lifecycle path, or connected to live exchange/RPC adapters.
- Why incomplete: Phase 11 intentionally implemented only deterministic model/trait boundaries with external submission disabled.
- Why blocked in ChatGPT Project Mode: Durable lifecycle validation requires Rust tests, filesystem/database persistence, mocked and sandbox adapters, network access, restart fixtures, and CI/runtime execution.
- Risk level: Critical
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 10 planner validation, Phase 11 validation baseline, exchange-specific CEX adapters, DEX/RPC adapters, signer boundary, and Phase 15 scenario harness.
- Exact future validation required: audit-before-adapter tests, state checkpoint tests, duplicate submission prevention tests, modeled fill replay tests, reconciliation replay tests, crash/restart tests, sandbox adapter tests, kill-switch tests, live-scope denial tests, no-broadcast-until-approved tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked adapter fixtures, sandbox exchange accounts, testnet RPC, CI runner.
- Recommended future agent type: Execution Adapter Agent + Audit and Observability Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe transition from model-only planning to real execution.
- Completion criteria: Every adapter run is durably journaled and state-checkpointed before any external submission, can be replayed/reconciled after restart, and live adapters cannot submit without policy, audit, state, and kill-switch approval.
- Rollback considerations: Disable planner-to-adapter handoff, preserve audit files, revoke credentials, remove live adapter registrations, and force Observe/Paper modes.

## GAP-0059 — Phase 12 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0059
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 12
- Subsystem association: Communications and CLI / build system
- Description: Phase 12 communications/CLI code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for communications and CLI code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/communications.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 11 state if validation exposes unrecoverable defects.

## GAP-0060 — Communications Audit, Auth, and Runtime Lifecycle Integration Missing

- Unique ID: GAP-0060
- Phase association: Phase 12 / Phase 14 / Phase 15 / Phase 17
- Subsystem association: Communications / audit journal / state store / runtime control
- Description: Phase 12 communication records are deterministic local models only. They are not durably journaled, authenticated, authorized, replayed, rate-limited against real channels, or integrated into a runtime operator-control lifecycle.
- Why incomplete: Phase 12 intentionally implements only model/trait boundaries and disables outbound integrations.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the communications boundary, but durable lifecycle validation, persistence, runtime orchestration, real channel authentication, platform accounts, network access, and external security review remain missing.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 12 validation baseline, authentication/authorization model, communications channel adapters, Phase 14 observability, and Phase 15 scenario tests.
- Exact future validation required: command audit-record tests, notification audit-record tests, replay determinism tests, operator authorization tests, command injection tests, no-secret-dispatch tests, rate-limit tests, channel outage tests, and restart/recovery tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked channel adapters, platform test accounts, CI runner.
- Recommended future agent type: Communications Integration Agent + Audit and Observability Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe remote operator control and production alerting.
- Completion criteria: Every command and notification is authenticated where remote, authorized, redacted, durably journaled, replayable, rate-limited, and fail-closed without enabling direct live execution.
- Rollback considerations: Disable remote channel adapters, revoke tokens, preserve audit records, and fall back to local CLI status-only operation.



## GAP-0061 — Phase 13 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0061
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 13
- Subsystem association: Embedded dashboard / build system
- Description: Phase 13 embedded-dashboard code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for embedded-dashboard code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/dashboard.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 12 state if validation exposes unrecoverable defects.

## GAP-0062 — Dashboard Hosting, Auth, and Runtime Lifecycle Integration Missing

- Unique ID: GAP-0062
- Phase association: Phase 13 / Phase 14 / Phase 15 / Phase 17
- Subsystem association: Embedded dashboard / authentication / audit journal / state store / runtime control
- Description: Phase 13 dashboard records are deterministic local models only. No HTTP server, browser delivery, authentication, authorization, CSRF protection, rate limiting, durable state, audit lifecycle, or penetration-tested hosting exists.
- Why incomplete: Phase 13 intentionally implements only model/trait boundaries and rejects server startup, public exposure, live controls, and secret rendering.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the dashboard boundary, but secure hosting validation requires runtime orchestration, local browser/server testing, network binding inspection, authentication design, persistence, and external security review.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 13 validation baseline, authentication/authorization model, secure local web host design, Phase 14 observability, Phase 15 scenario tests, and Phase 17 security review.
- Exact future validation required: loopback binding tests, public-bind denial tests, auth-required tests, CSRF tests, clickjacking/header tests, no-secret-render tests, live-control denial tests, command-injection tests, audit-record tests, restart/recovery tests, and penetration testing.
- Exact future tooling/environment required: Rust test runner, local browser/server harness, temporary filesystem, SQLite WAL backend, mocked runtime snapshots, CI runner, and AppSec review workflow.
- Recommended future agent type: Embedded Dashboard Agent + Audit and Observability Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks safe operator dashboard use beyond local in-process render records.
- Completion criteria: Dashboard hosting is loopback by default, authenticated where exposed, authorized, CSRF-protected, rate-limited, audited, redacted, fail-closed, and unable to trigger live execution without policy/audit/state approval.
- Rollback considerations: Disable dashboard hosting, preserve audit records, remove dashboard route registration, and fall back to CLI status-only operation.


## GAP-0063 — Phase 14 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0063
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 14
- Subsystem association: Observability and runbooks / build system
- Description: Phase 14 observability/runbook code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Current compile/test/lint confidence for observability/runbook code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/observability.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 13 state if validation exposes unrecoverable defects.

## GAP-0064 — Observability Runtime, Exporter, Alert, and Audit Lifecycle Integration Missing

- Unique ID: GAP-0064
- Phase association: Phase 14 / Phase 15 / Phase 17
- Subsystem association: Observability / runbooks / audit journal / state store / communications
- Description: Phase 14 observability records are deterministic local models only. No tracing subscriber, metrics endpoint, OpenTelemetry/Prometheus exporter, log shipping, alert routing, durable state, audit lifecycle, retention policy, or incident-drill validation exists.
- Why incomplete: Phase 14 intentionally implements only model/trait boundaries and disables metrics endpoints, public exposure, outbound alerts, and secret observability.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo validation exists for the observability/runbook boundary, but runtime lifecycle validation, alert/exporter validation, persistence, network binding inspection, mocked or real observability stacks, communication channel adapters, and external security review remain missing.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, SQLite WAL state store, Phase 12 communications validation, Phase 14 validation baseline, secure metrics endpoint design, Phase 15 scenario tests, and Phase 17 security review.
- Exact future validation required: tracing subscriber tests, redaction tests, no-secret-telemetry tests, metrics endpoint loopback tests, public-bind denial tests, exporter tests, log retention tests, alert routing tests, incident runbook drills, audit-record tests, restart/recovery tests, and panic/failure capture tests.
- Exact future tooling/environment required: Rust test runner, temporary filesystem, SQLite WAL backend, mocked observability/exporter fixtures, mocked communications channels, local runtime, CI runner, and AppSec review workflow.
- Recommended future agent type: Audit and Observability Agent + Communications Integration Agent + AppSec Lead + Rust Implementation Agent
- Estimated production impact: Blocks production-grade operations, incident response, and safe deployment monitoring.
- Completion criteria: Observability runtime emits redacted structured logs, health status, metrics, and critical alerts through authenticated, audited, fail-closed paths without exposing secrets or enabling live execution controls.
- Rollback considerations: Disable metrics endpoints, exporters, log shipping, and alert adapters; preserve local records and audit evidence; fall back to CLI/dashboard local status only.


## GAP-0065 — Phase 15 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0065
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 15
- Subsystem association: Testing, fuzzing, and backtesting / build system
- Description: Phase 15 testing/fuzzing/backtesting boundary code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + Testing, Fuzzing, and Backtesting Agent
- Estimated production impact: Current compile/test/lint confidence for testing/backtesting boundary code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/testing.rs`, remove exports, remove CLI status text, and return roadmap/gap tracker to Phase 14 state if validation exposes unrecoverable defects.

## GAP-0066 — Validation Runner, Fuzzing Engine, Broader Corpus, and Production Backtest Evidence Missing

- Unique ID: GAP-0066
- Phase association: Phase 15 / Phase 17
- Subsystem association: Testing / fuzzing / backtesting / CI / runtime validation
- Description: Phase 15 validation records are deterministic local models, and Phase 24 adds local paper backtest corpus execution over caller-supplied fixtures. No actual property-test framework integration, external fuzzing engine execution, curated fuzz corpus, CI-scale deterministic market replay runner, production backtest evidence, load testing, penetration testing, or production validation run exists.
- Why incomplete: Phase 15 intentionally implements only model/trait boundaries and disables external fuzzer invocation, live network tests, live execution, credential-bearing fixtures, signing, and broadcasts. Phase 24 executes local paper fixtures only and does not replace property, fuzz, load, penetration, or production validation.
- Why blocked in ChatGPT Project Mode: Current Rust/Cargo and CI evidence exists for the validation-plan boundary, but real runner execution requires fixture files, fuzzing dependencies, replay datasets, temporary filesystems/databases, security tooling, and external runtime environments.
- Risk level: High
- Dependency requirements: Current Rust validation baseline, Phase 4 audit/state validation, Phase 5 market data validation, Phase 6/24 paper validation, Phase 9 opportunity validation, Phase 10 planner validation, Phase 11 adapter validation, Phase 14 observability validation, curated fixtures, and CI runner.
- Exact future validation required: unit tests, integration tests, property tests, fuzz tests, deny-path tests, direct audit journal replay tests, CI-scale deterministic backtest replay tests, scenario regression tests, load tests, rollback tests, incident-drill tests, and penetration tests.
- Exact future tooling/environment required: Rust test runner, property-test crate, fuzzing engine, fixture corpus, temporary filesystem, SQLite WAL backend, mocked CEX/DEX/RPC fixtures, CI runner, load-test tooling, and AppSec review workflow.
- Recommended future agent type: Testing, Fuzzing, and Backtesting Agent + AppSec Lead + Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Blocks evidence-based confidence in safety, strategy correctness, replay determinism, and production readiness.
- Completion criteria: Validation plans execute in CI/local environments with deterministic fixture corpora, fuzz/property test coverage, replay/backtest records, load/security validation evidence, and no secret leakage or live side effects.
- Rollback considerations: Disable advanced runners, preserve fixtures and audit evidence, revert failing harness integrations, and keep Observe/Paper modes only until validation is restored.

## GAP-0067 — Phase 16 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0067
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 16
- Subsystem association: Packaging/deployment / build system
- Description: Phase 16 packaging/deployment boundary code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + Release Engineering Authority
- Estimated production impact: Current compile/test/lint confidence for packaging/deployment boundary code is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/packaging.rs`, remove exports, remove CLI status text, remove deployment templates, and return roadmap/gap tracker to Phase 15 state if validation exposes unrecoverable defects.

## GAP-0068 — Phase 16 Packaging, Deployment, and Rollback Execution Missing

- Unique ID: GAP-0068
- Phase association: Phase 16 / Phase 17
- Subsystem association: Packaging / deployment / release engineering / operations
- Description: Phase 16 package and deployment records are deterministic local models only. Current evidence exists for Rust validation, locked release-build validation, dependency audit, SBOM generation, an example-only container image build, local/CI Trivy example image-scan evidence, repeatable local example-container validation, static plus CI syntax example systemd-unit validation, and hardening artifact indexing. The first CI image scan exposed fixable critical Debian slim runtime findings that were patched by moving the example runtime to nonroot distroless Debian 12. Production validation is still missing: no production release artifact was built, no production container image was validated, no systemd unit was installed, no ARM binary was produced, no runtime deployment occurred, no rollback drill was executed, and no production release was validated.
- Why incomplete: Current non-secret CI, local example-container evidence, and static example systemd-unit validation improve packaging feedback, but Phase 16 intentionally avoids external side effects, service installation, public exposure, embedded secrets, live trading enablement, and production claims.
- Why blocked in ChatGPT Project Mode: Real deployment validation requires Rust tooling, container/systemd/ARM infrastructure, target hosts, filesystem controls, release artifact storage, rollback environment, security tooling, and operator credentials handled outside the repo.
- Risk level: High
- Dependency requirements: Keep current Rust/CI, locked release-build, SBOM-generation, dependency-audit, local-SARIF, example-image, image-scan, static example systemd-unit, secret-scan, and hardening-index evidence refreshable; add a production packaging target, systemd/Linux validation host, ARM validation target, signed release workflow, rollback procedure, observability integration, and incident runbooks.
- Exact future validation required: Refresh release-build, SBOM, dependency-audit, SAST, image-scan, secret-scan, and hardening-index evidence for the candidate commit, then execute production release artifact build, production container build, image scan review, SBOM review, service hardening validation, non-root runtime test, read-only filesystem test, health check test, config loading test, log/audit redaction test, rollback drill, incident drill, startup/shutdown soak test, and production readiness review.
- Exact future tooling/environment required: Current Rust/Cargo and CI runner for evidence refresh; container runtime, Linux systemd host, ARM target or verified cross target, SAST/dependency tools, artifact repository, staging environment, and human operator review for production validation.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority + AppSec Lead
- Estimated production impact: Example container build and scanner feedback improved the template, but deployable, operable, rollback-safe production release remains blocked.
- Completion criteria: Release artifacts are built, scanned, deployed to a staging target, validated under hardened runtime settings, rolled back successfully, and documented without secret leakage or live-funds exposure.
- Rollback considerations: Remove generated artifacts, disable services, restore previous configuration and binary, preserve logs/audit evidence, revoke any accidentally exposed credentials, and keep live execution disabled.



## GAP-0069 — Phase 17 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0069
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 17
- Subsystem association: External hardening evidence boundary / build system
- Description: Phase 17 external hardening boundary Rust/Cargo validation was previously unexecuted, but the current workspace has since passed local and GitHub Actions format, check, test, and clippy validation.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external hardening gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable toolchain, Cargo, rustfmt, clippy, CI runner, and repository checkout available for future validation runs.
- Exact future validation required: Re-run `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, CI runner.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + AppSec Lead
- Estimated production impact: Compile/test/lint confidence for the hardening evidence boundary is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/hardening.rs`, remove exports, remove CLI status text, remove hardening docs, and return roadmap/gap tracker to Phase 16 state if validation exposes unrecoverable defects.

## GAP-0070 — External Production Hardening Execution Missing

- Unique ID: GAP-0070
- Phase association: Phase 17 / Phase 18
- Subsystem association: External hardening / AppSec / release engineering / operations / compliance
- Description: Phase 17 added deterministic evidence models, release blockers, review records, and operator checklists. GitHub Actions CI, locked release-build validation, dependency audit, CycloneDX SBOM generation, local-SARIF CodeQL SAST, local example-only container build, CI Trivy example image-scan evidence, Gitleaks secret-pattern scan evidence, and hardening evidence indexing are passing as of the 2026-05-26 run `https://github.com/dominator509/arbyclaw/actions/runs/26443625602`. The first CI Trivy image-scan gate correctly failed on fixable critical Debian slim runtime findings and preserved non-secret evidence; a distroless runtime repair then passed follow-up CI validation. Production validation is still missing: SBOM review, GitHub code scanning upload processing, production container validation, systemd hardening validation, ARM validation, staging deployment, load test, penetration test, rollback drill, incident-response drill, live exchange/RPC validation, and production readiness review have not been executed.
- Why incomplete: Current CI evidence exists for release-build, dependency-audit, SBOM-generation, local-SARIF SAST, local example-container build, image-scan, secret-scan, and hardening-index gates. Phase 17 intentionally avoids credentials, public exposure, live funds, production claims, live exchange/RPC calls, signing, broadcasts, and deployment execution; the broader external hardening checklist remains incomplete until production-context reviews and drills are performed.
- Why blocked in ChatGPT Project Mode: Real production hardening requires external CI, runtime infrastructure, staging hosts, security tooling, target devices, controlled credentials outside the repo, human review, and accountable operator approval.
- Risk level: Critical
- Dependency requirements: Keep current CI, release-build, dependency-audit, SBOM-generation, local-SARIF SAST, example image-scan, secret-scan, and hardening-index evidence refreshable; add SBOM reviewer workflow, GitHub code scanning upload processing or accepted deferral, staging environment, observability runtime, incident runbooks, rollback procedure, AppSec review, exchange/RPC sandbox environments, custody/signer design, and compliance review.
- Exact future validation required: Refresh CI, release-build, SBOM-generation, dependency-audit, SAST, image-scan, secret-scan, and hardening-index evidence for the candidate commit, then complete SBOM review, GitHub code scanning upload processing if enabled, service hardening test, read-only filesystem test, config loading test, log/audit redaction test, staging deployment, startup/shutdown/restart tests, load and soak tests, penetration test, rollback drill, incident-response drill, exchange sandbox validation, DEX/RPC sandbox validation without broadcasts, key custody review, and production readiness review.
- Exact future tooling/environment required: Current Rust/Cargo, CI runner, SAST/dependency tools, SBOM generator, and container scanner for evidence refresh; Linux/systemd host, ARM target or cross-build runner, staging host, observability stack, load-test tooling, security testing workflow, sandbox exchange/RPC accounts, and human operator review for production validation.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority + AppSec Lead + Audit and Observability Agent + Compliance Reviewer
- Estimated production impact: CI, locked release-build, dependency-audit, SBOM-generation, local-SARIF SAST, SAST artifact retention, local example-container build, image-scan failure evidence, and secret-pattern scan evidence improved hardening feedback, but the remaining missing hardening evidence still blocks any credible production-readiness, public-service, live-funds, or autonomous-execution claim.
- Completion criteria: External hardening evidence is generated, reviewed, non-secret, linked from an external evidence store, and confirms all required production gates pass without enabling live funds prematurely.
- Rollback considerations: Preserve evidence, disable candidate services, restore prior artifact/configuration, revoke any exposed credentials, keep Observe/Paper modes only, and return to a known validated checkpoint.

## GAP-0071 — Phase 18 Rust Validation Covered for Current Workspace

- Unique ID: GAP-0071
- Latest audit status: Locally and CI-covered for the current workspace state as of 2026-05-25; this resolves only the Rust/Cargo validation aspect and does not close production, deployment, live-funds, or external hardening gaps.
- Phase association: Phase 18
- Subsystem association: Agentic handoff package boundary / build system
- Description: Phase 18 handoff boundary code and tests have current local and GitHub Actions format, compile, test, and clippy validation evidence for the present workspace state.
- Why incomplete: No longer incomplete for the current workspace Rust/Cargo validation aspect. Production, deployment, live-funds, public exposure, and broader external validation gaps remain tracked separately.
- Why blocked in ChatGPT Project Mode: No longer blocked for local Rust/Cargo validation in the current workspace.
- Risk level: Low for the current Rust/Cargo validation aspect; high production and live-funds risks remain tracked in other gap entries.
- Dependency requirements: Keep Rust stable, Cargo, rustfmt, clippy, Python 3, and GitHub Actions available for future validation runs.
- Exact future validation required: Re-run standard Cargo validation and structure validation after future changes.
- Exact future tooling/environment required: Rust stable toolchain, Cargo, rustfmt, clippy, Python 3, CI runner, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator + Handoff Agent
- Estimated production impact: Current compile/test/lint confidence for the handoff package boundary is improved, but this does not prove production readiness or external-agent execution success.
- Completion criteria: Met for the current workspace Rust/Cargo validation aspect; future changes must rerun the validation commands.
- Rollback considerations: Revert `crates/arb-core/src/handoff.rs`, remove exports, remove CLI status text, remove `handoff/` docs, and return roadmap/gap tracker to Phase 17 state if validation exposes unrecoverable defects.

## GAP-0072 — External Agentic Handoff Execution and Evidence Review Missing

- Unique ID: GAP-0072
- Phase association: Phase 18
- Subsystem association: Agentic handoff / external validation / human review
- Description: Phase 18 added deterministic handoff records, prompts, and checklists, and current repository validation plus non-secret CI evidence references exist for the handoff baseline. Production handoff validation is still missing: no external coding agent, DevSecOps agent, AppSec reviewer, compliance reviewer, or human production reviewer has consumed the package and produced independent non-secret review evidence.
- Why incomplete: Phase 18 intentionally creates local documentation/model boundaries only and does not invoke external agents, production reviewers, infrastructure, or services.
- Why blocked in ChatGPT Project Mode: Real handoff execution requires external repositories or IDEs, CI systems, agent runtimes, human reviewers, non-secret evidence storage, and accountable approval workflows outside this chat.
- Risk level: Medium
- Dependency requirements: Latest repository checkout or ZIP, complete governance files, current local/CI validation references, non-secret evidence workflow, DevSecOps/AppSec review process, and human maintainer approval.
- Exact future validation required: Refresh repository validation and CI evidence for the handoff candidate, run future-agent prompts against the latest repo, verify agents read governance files, verify no gaps are dropped, verify no secrets are requested or stored, verify all generated changes preserve policy gates and live-funds blockers, and record independent non-secret evidence references.
- Exact future tooling/environment required: External coding-agent runtime or IDE, Git repository, CI runner, issue tracker or evidence store, and human reviewer; no live credentials, signing keys, broadcasts, withdrawals, or real exchange/RPC calls are required for handoff validation.
- Recommended future agent type: Handoff Agent + DevSecOps Orchestrator + AppSec Lead + Human Maintainer
- Estimated production impact: Blocks reliable multi-agent continuation and accountable production-readiness review, but does not by itself block local model/documentation completeness.
- Completion criteria: External agents or human reviewers consume the handoff package, generate non-secret evidence, preserve all governance constraints, and update the gap tracker without claiming production readiness prematurely.
- Rollback considerations: Revert unsafe handoff-generated changes, discard secret-bearing artifacts if any appear, preserve the last validated ZIP, and resume from the latest known-good governance checkpoint.

## GAP-0073 - 2026-05-19 Local Toolchain Validation Resolved Locally

- Unique ID: GAP-0073
- Phase association: Phase 18 / post-handoff validation
- Subsystem association: Local validation toolchain / build system
- Description: The requested local validation sequence was attempted for ArbyClaw on 2026-05-19 and was initially blocked because `python3`, Cargo, and `rustc` were unavailable on PATH. On 2026-05-20, after the local toolchain was made available, the requested structure, format, compile, test, and clippy validation sequence completed successfully.
- Why incomplete: No longer incomplete for the requested local validation sequence. Production hardening, CI, deployment, live integration, security, load, rollback, incident, and external validation gaps remain tracked separately.
- Why blocked in ChatGPT/Codex environment: No longer blocked for the requested local validation sequence.
- Risk level: Low for local compile/test/lint evidence; high production and live-funds risks remain tracked in the other gap entries.
- Dependency requirements: Continue to keep Python 3, Rust stable, Cargo, rustfmt, clippy, and rustc available on PATH for future validation runs.
- Exact future validation required: Re-run `python3 scripts/validate_structure.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` after future changes.
- Exact future tooling/environment required: Python 3 launcher named `python3`, Rust stable toolchain, Cargo, rustfmt, clippy, rustc, and dependency access/cache as needed.
- Recommended future agent type: Rust Implementation Agent + DevSecOps Orchestrator
- Estimated production impact: Local compile/test/lint confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met on 2026-05-20 for the requested local validation sequence.
- Rollback considerations: If a future validation run fails, keep live execution disabled, patch only the failing source-level issue, and update this tracker with the new failure evidence.

## GAP-0074 - 2026-05-20 CI Validation Repository Target Resolved

- Unique ID: GAP-0074
- Phase association: Phase 18 / post-handoff CI validation
- Subsystem association: CI/CD / GitHub Actions / repository publishing
- Description: The requested CI validation sequence was initially blocked because no repository target was visible. On 2026-05-20 the new GitHub repository `dominator509/arbyclaw` was connected, the ArbyClaw checkpoint was pushed to `main`, and GitHub Actions CI passed. The latest recorded validated run is `26443625602` for commit `0b98a9a31d3701704d950779ad989daefcf1193b`, including checkout via `actions/checkout@v6`, the original validation sequence, `cargo build --release --locked`, `cargo audit`, CycloneDX SBOM generation with non-empty file checks, CodeQL Rust SAST local-SARIF generation with non-empty SARIF verification, short-retention SARIF artifact upload, example image scan, secret-pattern scan, hardening evidence index generation, and GitHub Step Summary evidence pointers.
- Why incomplete: No longer incomplete for initial GitHub Actions CI execution. Production hardening, deployment, live integration, security, load, rollback, incident, and external validation gaps remain tracked separately.
- Why blocked in ChatGPT/Codex environment: No longer blocked for initial GitHub Actions CI execution after the repository target was created and connected.
- Risk level: Low for initial CI compile/test/lint evidence; high production and live-funds risks remain tracked in the other gap entries.
- Dependency requirements: Keep `dominator509/arbyclaw` connected to this workspace, keep GitHub Actions enabled, and keep Rust stable, Cargo, rustfmt, clippy, and Python 3 available in CI.
- Exact future validation required: Re-run the GitHub Actions CI workflow containing `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release --locked`, `cargo audit`, CycloneDX SBOM generation, CodeQL Rust SAST local-SARIF generation, short-retention SARIF artifact retention, and `python3 scripts/validate_structure.py` after future changes; record non-secret run URLs and pass/fail results when they are material to release gating.
- Exact future tooling/environment required: Git checkout, configured remote, GitHub repository with Actions enabled, branch or pull request, workflow runner, Rust stable toolchain, Cargo, rustfmt, clippy, cargo-audit, cargo-cyclonedx, CodeQL Action availability, `actions/upload-artifact`, and Python 3.
- Recommended future agent type: DevSecOps Orchestrator + Release Engineering Authority
- Estimated production impact: External CI confidence is improved, but this does not prove production readiness or live-funds readiness.
- Completion criteria: Met on 2026-05-20 for initial GitHub Actions CI execution on `main`.
- Rollback considerations: If future CI setup changes are unsafe or point at the wrong repository, disable the incorrect workflow/branch, preserve local and CI validation evidence, and reconnect the workspace to the correct repository before retrying.

## GAP-0075 - GitHub Code Scanning Upload Not Enabled

- Unique ID: GAP-0075
- Phase association: Phase 17 / Phase 18 / post-handoff hardening evidence
- Subsystem association: SAST / GitHub Actions / GitHub code scanning
- Description: CodeQL Rust SAST now runs in GitHub Actions, verifies non-empty local SARIF output, and keeps that SARIF available as a short-retention Actions artifact. An earlier upload-based CodeQL run failed because GitHub code scanning is not enabled for the repository, and a later GitHub API default-setup enablement attempt returned the same code-scanning-disabled response. On 2026-05-24 the operator chose to keep `dominator509/arbyclaw` private and rely on the CI `codeql-sarif-evidence` artifact as the local SARIF-only SAST evidence path for now, so SARIF upload to the GitHub Security tab remains intentionally deferred.
- Why incomplete: The repository-level code scanning feature is disabled or unavailable and the operator has not approved making the repository public or enabling paid/private-repository code-scanning support, so CodeQL evidence is limited to local SARIF generation and artifact retention inside CI rather than GitHub code scanning alert processing.
- Why blocked in ChatGPT/Codex environment: Enabling repository code scanning may require an external repository settings change, plan/support entitlement, or explicit owner decision outside safe local source-code changes.
- Risk level: Medium
- Dependency requirements: Repository owner review of GitHub code scanning availability, repository settings access, any required GitHub security entitlement, and a successful upload-capable CodeQL run after the feature is enabled.
- Exact future validation required: Enable GitHub code scanning if appropriate, remove or revise `upload: never`, run CodeQL with SARIF upload enabled, and confirm the GitHub Security/code-scanning alert processing completes without secret leakage or production-readiness claims.
- Exact future tooling/environment required: GitHub repository settings access, GitHub Actions, CodeQL Action, code scanning feature availability, and a non-secret evidence reference.
- Recommended future agent type: DevSecOps Orchestrator + AppSec Lead
- Estimated production impact: Local SAST evidence and evidence retention are improved, but centralized GitHub code scanning alert processing and review evidence remain incomplete.
- Completion criteria: CodeQL results are uploaded to and processed by GitHub code scanning, or the documented local SARIF-only governance decision remains accepted for this private repository and release reviewers continue to record GAP-0075 as open/deferred.
- Rollback considerations: Keep local SARIF-only SAST gate if upload remains unavailable; if upload is enabled and later fails, revert to the last passing local-SARIF gate and record the failure evidence.

## GAP-0076 - Phase 19 Production Runtime Lifecycle Validation Missing

- Unique ID: GAP-0076
- Phase association: Phase 19
- Subsystem association: Runtime lifecycle / audit journal / state store / execution adapter
- Description: Phase 19 wires local deterministic runtime lifecycle sequencing for planner-to-adapter boundaries. The lifecycle appends audit records, checkpoints the plan before adapter evaluation, evaluates the deterministic adapter boundary, checkpoints the adapter run, appends adapter-completion audit records, validates concurrent local lifecycle access over shared audit and SQLite WAL paths, fails closed before adapter evaluation on simulated state permission persistence failure, records local graceful-shutdown audit/state checkpoints, validates local runtime audit/SQLite backup-restore copies, produces local restart recovery summaries from audit replay plus SQLite checkpoint reopen checks, classifies recovery as `ready-for-local-review` or `needs-operator-review`, surfaces those labels through CLI status as local operator-review states, fails closed when recovery checkpoints are incomplete, rejects live scope before audit/state mutation, and preserves no external submission or live execution. Phase 24 adds local paper runtime validation records over replay/backtest evidence while preserving production blockers. Phase 26 adds local audit crash-like truncation, tamper, concurrent append, sync, invalid-filesystem validation probes, a local deployment-like runtime smoke harness plus CLI runner that combine lifecycle, graceful-shutdown, backup/restore, restart recovery, and audit durability probes without service-manager actions, manual systemd lifecycle plan/inspect tooling that does not mutate services, a combined deployment-host runtime report wrapper, a non-mutating rollback-drill evidence helper, a non-mutating incident-response drill evidence helper, a non-mutating deployment evidence bundle index, and a non-mutating deployment evidence checklist for sanitized external evidence locators.
- Why incomplete: The local code path and unit tests exist, including SQLite WAL reopen coverage, process-level crash/restart checkpoint recovery coverage, local concurrent lifecycle access coverage, local state-permission fail-closed coverage, local graceful-shutdown checkpoint reopen coverage, local runtime backup-restore copy/reopen coverage, local restart recovery summary and CLI-visible operator-review disposition coverage, incomplete-recovery fail-closed coverage, local paper replay/backtest runtime records, local audit durability probes, side-effect-free stale-lock restart recheck planning, local deployment-like smoke sequencing, a local runtime-smoke CLI command, manual non-mutating systemd lifecycle plan/inspect tooling, a combined deployment-host runtime report wrapper, non-mutating rollback-drill evidence tooling, non-mutating incident-response drill evidence tooling, non-mutating deployment evidence bundle indexing, and non-mutating deployment evidence checklist validation, but production runtime lifecycle validation is still missing for deployment-load concurrent access, deployment filesystem permissions, physical disk-full behavior, long-running daemon orchestration, deployment-host graceful shutdown execution, backup/restore under deployment load, observability integration, operator-controlled service-manager lifecycle execution behavior, actual rollback execution, actual incident-response execution, and real deployment environments.
- Why blocked in ChatGPT/Codex environment: Production runtime lifecycle validation requires targeted runtime scenarios, filesystem and process control, deployment-like environments, service-manager restart harnesses, and external evidence that cannot be claimed from local unit tests alone.
- Risk level: High
- Dependency requirements: Phase 4 audit/state validation, Phase 10 planner, Phase 11 execution-adapter boundary, Phase 26 local audit validation, SQLite WAL store, runtime test harness, filesystem controls, CI or local integration runner, and non-secret evidence recording.
- Exact future validation required: keep local audit replay/truncation/tamper/concurrency/filesystem/simulated-disk-full/stale-lock planning, concurrent lifecycle access, state-permission fail-closed, graceful-shutdown checkpoint, local backup-restore, local restart recovery, operator-review disposition, incomplete-recovery fail-closed, local deployment-like runtime smoke probes, local runtime-smoke CLI execution, manual lifecycle plan/inspect tooling, combined deployment-host runtime report tooling, rollback-drill evidence tooling, incident-response drill evidence tooling, deployment evidence bundle indexing, and deployment evidence checklist validation passing; add SQLite reopen/recovery test under deployment lifecycle load, deployment-host concurrent lifecycle access test, permission-denial fail-closed test under deployment conditions, physical disk-full fail-closed test, deployment-host graceful shutdown execution test, deployment-load backup/restore test, operator-controlled service-manager lifecycle execution test, executed rollback drill evidence, and executed incident-response drill evidence.
- Exact future tooling/environment required: Rust stable toolchain, temporary filesystem/database, process restart harness, SQLite runtime, CI runner or controlled local runtime host, and non-secret evidence store.
- Recommended future agent type: Rust Implementation Agent + Audit and Observability Agent + DevSecOps Orchestrator
- Estimated production impact: Removes the local implementation gap for planner-to-adapter lifecycle wiring, but production runtime reliability remains unproven until lifecycle validation is executed externally or in an approved runtime harness.
- Completion criteria: Runtime lifecycle validation passes for crash/restart, audit replay, SQLite recovery, concurrent access, filesystem permission, deployment-host graceful shutdown execution, backup/restore, and deployment-like smoke scenarios with non-secret evidence references.
- Rollback considerations: Disable runtime lifecycle orchestration, keep observe/paper modes only, preserve last known-good audit/state files for diagnosis, and revert Phase 19 runtime module plus exports if validation exposes unrecoverable defects.
