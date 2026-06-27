# Phase 26 - Audit Crash, Concurrency, Filesystem, Disk-Full, and Stale-Lock Validation

## Status

Implemented for local deterministic audit journal validation scope.

## Goal

Fill the local coding gap for append-only audit journal crash, concurrency, and filesystem validation without adding live execution, external calls, signing, withdrawals, bridges, broadcasts, wallet custody, or secrets.

## Scope

- Serialize local audit appends with a lock file.
- Recompute journal replay state while holding the append lock.
- Flush and `sync_all` audit records after append.
- Add a local audit durability validation report.
- Add a dedicated local audit durability CLI/report path for the Phase 26 probes.
- Validate append/reopen replay.
- Validate crash-like truncated JSONL replay rejection.
- Validate tamper/hash-chain replay rejection.
- Validate concurrent local append replay.
- Validate fail-closed filesystem shape errors.
- Validate simulated disk-full append failure classification and fail-closed journal state.
- Model audit retention and rotation decisions without deleting files or mutating the filesystem.
- Execute audit retention and rotation only inside an explicit fresh local sandbox workspace, with deployment-host retention execution still deferred.
- Model stale-lock/service-manager restart recheck decisions without deleting lock files, inspecting live processes, starting services, or mutating deployment state.
- Add a local deployment-like runtime smoke harness and CLI load/latency runner that combine lifecycle, graceful-shutdown, backup/restore, restart recovery, local communications command/notification recovery, local dashboard render recovery, local validation-run/property-check recovery, paper execution report/ledger recovery for paper-scoped plans, and audit durability probes without starting services or touching deployment state.
- Add a local runtime blocked-state preflight CLI/report path proving pre-existing state artifacts fail closed before smoke artifacts are created.
- Add a non-mutating deployment filesystem preflight report path for candidate audit/state paths and a non-mutating deployment retention preflight report path for candidate active/archive audit retention paths.
- Add static and optional syntax validation for the committed example systemd unit without installing, enabling, reloading, starting services, or mutating deployment state.
- Add a manual systemd lifecycle plan/inspect helper that produces non-secret deployment-host evidence structure and read-only `systemctl show` output without service-manager mutation.
- Add a deployment-host runtime report wrapper that composes non-mutating systemd lifecycle evidence with the existing local runtime-smoke CLI, local audit durability CLI, local sandbox audit-retention execution CLI, local graceful-shutdown checkpoint/reopen CLI, local backup/restore copy/reopen CLI, local backup/restore concurrent-load CLI, local runtime permission-denial CLI, local runtime incomplete-recovery CLI, local runtime panic-hook CLI, local restart-recovery replay/reopen CLI, local process-supervised restart CLI, local blocked-state preflight CLI, non-mutating filesystem preflight, non-mutating retention preflight, and local observability-runtime CLI when explicitly requested.
- Add a non-mutating rollback-drill evidence wrapper that validates sanitized rollback metadata without changing services, files, deployments, or runtime state.
- Add a non-mutating incident-response drill evidence wrapper that validates sanitized incident metadata without changing services, files, alert routes, deployments, or runtime state.
- Add a non-mutating deployment evidence bundle index that summarizes local validation helper outputs without embedding full artifact contents.
- Add a non-mutating deployment evidence checklist validator that references sanitized external evidence locators and keeps missing production evidence categories explicit.
- Add CI artifact and Step Summary wiring for the deployment evidence checklist without changing runtime behavior or claiming readiness.
- Add Rust tests for the validation harness, partial JSONL replay rejection, permission/disk-failure state preservation, and fail-closed workspace handling.
- Add Rust tests for side-effect-free retention/rotation planning and local sandbox-only retention/rotation execution.
- Add Rust tests for side-effect-free stale-lock restart recheck planning.
- Add Rust tests for the local deployment-like runtime smoke harness and repeated-iteration load/latency aggregate.
- Add a CLI smoke-runner check using non-secret local paths.
- Update roadmap, architecture, handoff, manifest, and gap tracker language.

## Explicit Non-Goals

- No live trading.
- No real exchange, sandbox, or RPC calls.
- No real balance reads or real account mutation.
- No signing, withdrawals, bridges, broadcasts, wallet custody, or external adapter submission.
- No production deployment or production-readiness approval.
- No claim that local tests prove every production filesystem, disk-full condition, retention/rotation policy, service-manager restart path, container runtime, or remote storage layer.

## Acceptance Criteria

- `PHASE_26_SUBROADMAP.md` exists.
- `AppendOnlyAuditJournal::append_event` serializes local appenders before sequence/hash calculation.
- Audit appends call `sync_all` after writing.
- Local validation rejects truncated crash-like records.
- Local validation rejects tampered records.
- Local validation proves concurrent local appenders produce a replayable journal.
- Local validation proves invalid filesystem shape fails closed.
- Local validation proves simulated disk-full append failure is classified and does not advance in-memory or replayed journal state.
- Local audit durability CLI validation reports append/replay, truncation, tamper, concurrency, filesystem failure, and simulated disk-full probes without live network, external execution, or production claims.
- Local retention/rotation planning marks rotate/retain/expired decisions without deleting files, renaming files, or mutating the filesystem.
- Local sandbox retention/rotation execution rotates an active journal and deletes expired archives only inside an explicit fresh workspace, rejects out-of-workspace paths, and preserves no live network, external execution, or production-readiness flags.
- Local stale-lock restart recheck planning marks stale/fresh lock observations without deleting lock files, starting services, or mutating deployment state.
- Local deployment-like runtime smoke validation runs lifecycle, graceful-shutdown, backup/restore, restart recovery, local communications command/notification checkpoint recovery, local dashboard render checkpoint recovery, local validation-run/property-check checkpoint recovery, paper execution report/ledger checkpoint recovery for paper-scoped plans, and audit durability probes from a typed CLI command, aggregates repeated local iteration elapsed-time/replay/trace-recovery outcomes, and preserves no service-manager actions, external calls, external fuzzers, outbound-network notification delivery, public dashboard hosting, or production claims.
- Local runtime blocked-state preflight validation proves pre-existing state artifacts fail closed before audit, backup, or audit-durability workspaces are created.
- Non-mutating deployment filesystem preflight reporting inspects candidate audit/state parent access and path shape without creating, opening, locking, or fsyncing production files.
- Non-mutating deployment retention preflight reporting inspects candidate active audit journal and audit archive directory access without rotating, deleting, creating, opening, locking, or fsyncing production files.
- Static example systemd-unit validation checks hardening directives and secret-free service configuration, and CI runs syntax verification against a temporary fake root, without installing, enabling, reloading, or starting services.
- Manual systemd lifecycle plan/inspect validation records operator steps by default and can perform bounded read-only deployment-host `systemctl show` inspection when run explicitly on Linux.
- Deployment-host runtime wrapper validation produces a combined non-secret report without service-manager mutation, bounds the lifecycle helper call, and can explicitly run the local runtime-smoke CLI, local audit durability CLI, local sandbox audit-retention execution CLI, local graceful-shutdown checkpoint/reopen CLI, local backup/restore copy/reopen CLI, local backup/restore concurrent-load CLI, local runtime permission-denial CLI, local runtime incomplete-recovery CLI, local runtime panic-hook CLI, local restart-recovery replay/reopen CLI, local process-supervised restart CLI, local blocked-state preflight CLI, non-mutating filesystem preflight reporting, non-mutating retention preflight reporting, and local observability-runtime CLI reporting.
- Rollback-drill wrapper validation produces a sanitized rollback plan and strict metadata check without service-manager actions, file mutation, external calls, secrets, or production claims.
- Incident-response drill wrapper validation produces a sanitized incident plan and strict metadata check without service-manager actions, file mutation, alert delivery, external calls, secrets, or production claims.
- Deployment evidence bundle validation produces a compact component index over bounded non-mutating local helpers, including the deployment-host retention preflight component, without service-manager actions, file mutation, alert delivery, external calls, secrets, full artifact embedding, or production claims.
- Deployment evidence checklist validation invokes the bundle through a bounded local helper call and marks service lifecycle, deployment-host audit/SQLite, physical disk-full, retention/rotation, rollback, incident-response, and production-readiness review evidence as referenced or missing without embedding artifact contents or production claims.
- CI deployment evidence checklist artifact generation preserves the same non-secret, missing-evidence-only scope and is represented in the hardening evidence index and job summary.
- Standard structure and Cargo validation pass.

## Validation

Required after this patch:

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Rollback Plan

1. Revert audit lock, sync, validation report, and validation harness changes in `crates/arb-core/src/audit.rs`.
2. Remove Phase 26 audit exports from `crates/arb-core/src/lib.rs`.
3. Remove Phase 26 audit status text from `crates/arb-agent/src/main.rs`.
4. Revert `PHASE_26_SUBROADMAP.md`, validator, manifest, and governance updates.
5. Re-run structure and Cargo validation.

## Deferred Work

- Deployment-host audit validation.
- Physical deployment-host disk-full evidence.
- Deployment-host retention and rotation execution evidence beyond the local sandbox CLI/report path.
- Deployment-host service-manager restart execution evidence.
- Operator-controlled deployment-host lifecycle execution evidence after manual plan/inspect evidence is captured.
- Deployment-host execution evidence after the combined report is run on a real target host.
- Operator-controlled rollback drill execution evidence after the non-mutating rollback plan is reviewed.
- Operator-controlled incident-response drill execution evidence after the non-mutating incident plan is reviewed.
- Production runtime validation with external non-secret evidence references.
