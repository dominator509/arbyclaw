#!/usr/bin/env python3
"""Compose non-secret deployment-host runtime validation evidence.

By default this script runs only the non-mutating systemd lifecycle plan helper.
When `--run-runtime-smoke` is provided, it also runs the local
`validate-runtime-smoke` CLI against a caller-supplied fresh workspace. It never
installs units, reloads systemd, enables services, starts services, stops
services, restarts services, loads secrets, calls exchanges/RPCs, or claims
production readiness. When `--run-filesystem-preflight` is provided, it inspects
candidate audit/state path parent permissions without creating, opening,
locking, or fsyncing runtime files. When `--run-observability-runtime` is
provided, it composes the local observability runtime CLI output into the same
non-secret report without starting exporters, serving public endpoints, sending
alerts, or changing runtime deployment behavior. When
`--run-observability-metrics-runtime` is provided, it composes the bounded local
metrics runtime CLI output into the same report without daemon hosting,
telemetry export, alert delivery, public exposure, or deployment mutation. When
`--run-observability-provider-boundary` is provided, it composes the local
provider-boundary CLI output into the same report without exporter sessions, log
shipping, alert delivery, public exposure, service-manager actions, sensitive
material loading, or deployment mutation. When `--run-graceful-shutdown` is
provided, it runs the local graceful-shutdown checkpoint/reopen CLI against a
fresh workspace without stopping services or mutating deployment state. When
`--run-deployment-static-hardening` is provided, it runs the static deployment
hardening/config smoke validator through the same report without installing or
controlling services, mutating deployment paths, starting listeners, loading
secrets, or enabling live execution. When
`--run-backup-restore` is provided, it runs the local runtime backup/restore
CLI against a fresh workspace without copying production files or mutating
deployment state. When `--run-backup-restore-load` is provided, it runs the
local concurrent backup/restore load CLI against a fresh workspace without
copying production files or mutating deployment state. When
`--run-restart-recovery` is provided, it runs the local runtime restart-recovery
CLI against a fresh workspace without service-manager actions or mutating
deployment state. When `--run-incomplete-recovery` is provided, it runs the
local missing-checkpoint recovery fail-closed CLI against a fresh workspace
without service-manager actions or mutating deployment state. When
`--run-supervised-restart` is
provided, it runs a local child-process restart harness against a fresh
workspace without service-manager actions or mutating deployment state.
When `--run-permission-denial` is provided, it runs the local runtime
permission-denial fail-closed CLI against a fresh workspace without mutating
deployment state. When `--run-runtime-panic-hook` is provided, it runs the
local runtime panic-hook capture CLI against a fresh workspace without starting
services, exporters, public endpoints, or alerts.
When `--run-deployment-log-redaction` is provided, it runs a local sanitized
deployment log/audit redaction CLI against a fresh workspace without touching
deployment logs, service managers, secrets, external systems, or live
execution. When `--run-communications-delivery-provider-boundary` is provided,
it composes the local delivery-provider boundary CLI output into the same report
without provider calls, message delivery, token loading, outbound network use,
service-manager actions, or deployment mutation.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import shutil
import subprocess
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
SYSTEMD_LIFECYCLE_SCRIPT = ROOT / "scripts/validate_systemd_lifecycle.py"
DEFAULT_CONFIG = ROOT / "config.example.toml"
SYSTEMD_LIFECYCLE_TIMEOUT_SECONDS = 45
LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS = 300


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--systemd-mode",
        choices=("plan", "inspect"),
        default="plan",
        help="systemd lifecycle helper mode; inspect is Linux-only and read-only",
    )
    parser.add_argument(
        "--unit",
        default="arb-agent.service",
        help="systemd unit name passed to the lifecycle helper",
    )
    parser.add_argument(
        "--run-runtime-smoke",
        action="store_true",
        help="run arb-agent validate-runtime-smoke against --runtime-workspace",
    )
    parser.add_argument(
        "--run-audit-retention-execution",
        action="store_true",
        help="run arb-agent validate-audit-retention-execution against --retention-workspace",
    )
    parser.add_argument(
        "--run-audit-durability",
        action="store_true",
        help="run arb-agent validate-audit-durability against --audit-durability-workspace",
    )
    parser.add_argument(
        "--run-runtime-config-reload",
        action="store_true",
        help="run arb-agent validate-runtime-config-reload against --runtime-config-reload-workspace",
    )
    parser.add_argument(
        "--run-deployment-static-hardening",
        action="store_true",
        help="run static deployment hardening/config smoke validation",
    )
    parser.add_argument(
        "--run-sqlite-schema-migration",
        action="store_true",
        help="run arb-agent validate-sqlite-wal-schema-migration against --sqlite-schema-migration-workspace",
    )
    parser.add_argument(
        "--run-deployment-config-redaction",
        action="store_true",
        help="run arb-agent validate-deployment-config-redaction against --deployment-config-redaction-workspace",
    )
    parser.add_argument(
        "--run-deployment-log-redaction",
        action="store_true",
        help="run arb-agent validate-deployment-log-redaction against --deployment-log-redaction-workspace",
    )
    parser.add_argument(
        "--run-graceful-shutdown",
        action="store_true",
        help="run arb-agent validate-runtime-graceful-shutdown against --graceful-shutdown-workspace",
    )
    parser.add_argument(
        "--run-backup-restore",
        action="store_true",
        help="run arb-agent validate-runtime-backup-restore against --backup-restore-workspace",
    )
    parser.add_argument(
        "--run-backup-restore-load",
        action="store_true",
        help="run arb-agent validate-runtime-backup-restore-load against --backup-restore-load-workspace",
    )
    parser.add_argument(
        "--run-restart-recovery",
        action="store_true",
        help="run arb-agent validate-runtime-restart-recovery against --restart-recovery-workspace",
    )
    parser.add_argument(
        "--run-incomplete-recovery",
        action="store_true",
        help="run arb-agent validate-runtime-incomplete-recovery against --incomplete-recovery-workspace",
    )
    parser.add_argument(
        "--run-supervised-restart",
        action="store_true",
        help="run arb-agent validate-runtime-supervised-restart against --supervised-restart-workspace",
    )
    parser.add_argument(
        "--run-permission-denial",
        action="store_true",
        help="run arb-agent validate-runtime-permission-denial against --permission-denial-workspace",
    )
    parser.add_argument(
        "--run-blocked-state-preflight",
        action="store_true",
        help="run arb-agent validate-runtime-blocked-state-preflight against --blocked-state-workspace",
    )
    parser.add_argument(
        "--run-blocked-audit-preflight",
        action="store_true",
        help="run arb-agent validate-runtime-blocked-audit-preflight against --blocked-audit-workspace",
    )
    parser.add_argument(
        "--run-filesystem-preflight",
        action="store_true",
        help="inspect deployment audit/state path parents without creating files",
    )
    parser.add_argument(
        "--run-retention-preflight",
        action="store_true",
        help="inspect deployment audit retention paths without rotating or deleting logs",
    )
    parser.add_argument(
        "--run-observability-runtime",
        action="store_true",
        help="run arb-agent validate-observability-runtime against --observability-workspace",
    )
    parser.add_argument(
        "--run-observability-metrics-runtime",
        action="store_true",
        help="run arb-agent validate-observability-metrics-runtime against --observability-metrics-workspace",
    )
    parser.add_argument(
        "--run-observability-provider-boundary",
        action="store_true",
        help="run arb-agent validate-observability-provider-boundary against --observability-provider-boundary-workspace",
    )
    parser.add_argument(
        "--run-runtime-panic-hook",
        action="store_true",
        help="run arb-agent validate-runtime-panic-hook against --runtime-panic-hook-workspace",
    )
    parser.add_argument(
        "--run-dashboard-runtime",
        action="store_true",
        help="run arb-agent validate-dashboard-runtime against --dashboard-workspace",
    )
    parser.add_argument(
        "--run-communications-runtime",
        action="store_true",
        help="run arb-agent validate-communications-runtime against --communications-workspace",
    )
    parser.add_argument(
        "--run-communications-delivery-provider-boundary",
        action="store_true",
        help="run arb-agent validate-communications-delivery-provider-boundary against --communications-delivery-provider-workspace",
    )
    parser.add_argument(
        "--runtime-smoke-iterations",
        type=int,
        default=1,
        help="iterations for validate-runtime-smoke when --run-runtime-smoke is set",
    )
    parser.add_argument(
        "--config",
        type=pathlib.Path,
        default=DEFAULT_CONFIG,
        help="non-secret config for validate-runtime-smoke",
    )
    parser.add_argument(
        "--runtime-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-smoke",
    )
    parser.add_argument(
        "--retention-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-audit-retention-execution",
    )
    parser.add_argument(
        "--audit-durability-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-audit-durability",
    )
    parser.add_argument(
        "--runtime-config-reload-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-config-reload",
    )
    parser.add_argument(
        "--sqlite-schema-migration-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-sqlite-wal-schema-migration",
    )
    parser.add_argument(
        "--deployment-config-redaction-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-deployment-config-redaction",
    )
    parser.add_argument(
        "--deployment-log-redaction-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-deployment-log-redaction",
    )
    parser.add_argument(
        "--graceful-shutdown-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-graceful-shutdown",
    )
    parser.add_argument(
        "--backup-restore-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-backup-restore",
    )
    parser.add_argument(
        "--backup-restore-load-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-backup-restore-load",
    )
    parser.add_argument(
        "--restart-recovery-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-restart-recovery",
    )
    parser.add_argument(
        "--incomplete-recovery-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-incomplete-recovery",
    )
    parser.add_argument(
        "--supervised-restart-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-supervised-restart",
    )
    parser.add_argument(
        "--permission-denial-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-permission-denial",
    )
    parser.add_argument(
        "--blocked-state-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-blocked-state-preflight",
    )
    parser.add_argument(
        "--blocked-audit-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-blocked-audit-preflight",
    )
    parser.add_argument(
        "--observability-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-observability-runtime",
    )
    parser.add_argument(
        "--observability-metrics-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-observability-metrics-runtime",
    )
    parser.add_argument(
        "--observability-provider-boundary-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-observability-provider-boundary",
    )
    parser.add_argument(
        "--runtime-panic-hook-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-runtime-panic-hook",
    )
    parser.add_argument(
        "--dashboard-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-dashboard-runtime",
    )
    parser.add_argument(
        "--communications-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-communications-runtime",
    )
    parser.add_argument(
        "--communications-delivery-provider-workspace",
        type=pathlib.Path,
        help="fresh non-secret workspace for validate-communications-delivery-provider-boundary",
    )
    parser.add_argument(
        "--filesystem-audit-path",
        type=pathlib.Path,
        help="candidate deployment audit journal path inspected by --run-filesystem-preflight",
    )
    parser.add_argument(
        "--filesystem-state-path",
        type=pathlib.Path,
        help="candidate deployment SQLite state path inspected by --run-filesystem-preflight",
    )
    parser.add_argument(
        "--retention-active-path",
        type=pathlib.Path,
        help="candidate active deployment audit journal path inspected by --run-retention-preflight",
    )
    parser.add_argument(
        "--retention-archive-dir",
        type=pathlib.Path,
        help="candidate deployment audit archive directory inspected by --run-retention-preflight",
    )
    parser.add_argument(
        "--agent-bin",
        type=pathlib.Path,
        help="optional arb-agent binary; default uses cargo run -p arb-agent",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="emit JSON instead of text",
    )
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"deployment-host runtime validation failed: {message}", file=sys.stderr)
    return 1


def relative_or_absolute(path: pathlib.Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path.resolve())


def run_json_command(
    command: list[str],
    cwd: pathlib.Path,
    timeout_seconds: int = SYSTEMD_LIFECYCLE_TIMEOUT_SECONDS,
) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=timeout_seconds,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stdout.strip() or "command failed")
    try:
        report = json.loads(completed.stdout)
        if isinstance(report, dict):
            report.setdefault("wrapper_timeout_seconds", timeout_seconds)
        return report
    except json.JSONDecodeError as error:
        raise RuntimeError(f"command did not emit valid JSON: {error}") from error


def run_systemd_lifecycle(mode: str, unit: str) -> dict[str, Any]:
    command = [
        sys.executable,
        str(SYSTEMD_LIFECYCLE_SCRIPT),
        "--mode",
        mode,
        "--unit",
        unit,
        "--json",
    ]
    return run_json_command(command, ROOT)


def parse_key_value_output(output: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in output.splitlines():
        if ": " not in line:
            continue
        key, value = line.split(": ", 1)
        if key:
            values[key.strip()] = value.strip()
    return values


def secret_like_path(path: pathlib.Path) -> bool:
    lowered = path.as_posix().lower()
    return any(marker in lowered for marker in ("secret", "token", "password", ".env", "key"))


def inspect_candidate_file_path(path: pathlib.Path | None, label: str) -> dict[str, Any]:
    if path is None:
        return {
            "label": label,
            "provided": False,
            "usable_for_runtime_artifacts": False,
            "reason": "path not provided",
        }
    resolved = path.resolve()
    parent = resolved.parent
    path_exists = resolved.exists()
    parent_exists = parent.exists()
    parent_is_dir = parent.is_dir()
    parent_readable = os.access(parent, os.R_OK) if parent_exists else False
    parent_writable = os.access(parent, os.W_OK) if parent_exists else False
    parent_executable = os.access(parent, os.X_OK) if parent_exists else False
    existing_file_ok = not path_exists or resolved.is_file()
    secret_like = secret_like_path(resolved)
    usable = (
        parent_exists
        and parent_is_dir
        and parent_readable
        and parent_writable
        and parent_executable
        and existing_file_ok
        and not secret_like
    )
    if usable:
        reason = "parent allows runtime artifact access"
    elif secret_like:
        reason = "path name looks secret-like"
    elif not parent_exists:
        reason = "parent directory missing"
    elif not parent_is_dir:
        reason = "parent is not a directory"
    elif not existing_file_ok:
        reason = "target exists but is not a file"
    else:
        reason = "parent permission check failed"

    return {
        "label": label,
        "provided": True,
        "path": relative_or_absolute(resolved),
        "parent": relative_or_absolute(parent),
        "path_exists": path_exists,
        "parent_exists": parent_exists,
        "parent_is_dir": parent_is_dir,
        "parent_readable": parent_readable,
        "parent_writable": parent_writable,
        "parent_executable": parent_executable,
        "existing_file_ok": existing_file_ok,
        "secret_like_path": secret_like,
        "usable_for_runtime_artifacts": usable,
        "reason": reason,
    }


def inspect_candidate_directory_path(path: pathlib.Path | None, label: str) -> dict[str, Any]:
    if path is None:
        return {
            "label": label,
            "provided": False,
            "usable_for_runtime_artifacts": False,
            "reason": "path not provided",
        }
    resolved = path.resolve()
    exists = resolved.exists()
    is_dir = resolved.is_dir()
    readable = os.access(resolved, os.R_OK) if exists else False
    writable = os.access(resolved, os.W_OK) if exists else False
    executable = os.access(resolved, os.X_OK) if exists else False
    secret_like = secret_like_path(resolved)
    usable = exists and is_dir and readable and writable and executable and not secret_like
    if usable:
        reason = "directory allows runtime artifact access"
    elif secret_like:
        reason = "path name looks secret-like"
    elif not exists:
        reason = "directory missing"
    elif not is_dir:
        reason = "path is not a directory"
    else:
        reason = "directory permission check failed"

    return {
        "label": label,
        "provided": True,
        "path": relative_or_absolute(resolved),
        "exists": exists,
        "is_dir": is_dir,
        "readable": readable,
        "writable": writable,
        "executable": executable,
        "secret_like_path": secret_like,
        "usable_for_runtime_artifacts": usable,
        "reason": reason,
    }


def parse_runtime_smoke_output(output: str) -> tuple[list[dict[str, str]], bool]:
    iterations: list[dict[str, str]] = []
    current: dict[str, str] | None = None
    saw_iteration_header = False
    for line in output.splitlines():
        if line.startswith("runtime-smoke-iteration: "):
            if current is not None:
                iterations.append(current)
            current = {"runtime-smoke-iteration": line.split(": ", 1)[1].strip()}
            saw_iteration_header = True
            continue
        if ": " not in line:
            continue
        key, value = line.split(": ", 1)
        if key and current is not None:
            current[key.strip()] = value.strip()
    if current is not None:
        iterations.append(current)
    if not saw_iteration_header and output.strip():
        return ([parse_key_value_output(output)], False)
    return (iterations, saw_iteration_header)


def parse_recovered_trace_summary(value: str) -> dict[str, str]:
    summary: dict[str, str] = {}
    for field in value.split(";"):
        if "=" not in field:
            continue
        key, field_value = field.split("=", 1)
        if key:
            summary[key.strip()] = field_value.strip()
    return summary


def parse_recovered_trace_summaries(parsed: dict[str, str]) -> list[dict[str, str]]:
    prefix = "restart-opportunity-trace-recovered-summary-"
    indexed: list[tuple[int, dict[str, str]]] = []
    for key, value in parsed.items():
        if not key.startswith(prefix):
            continue
        index_text = key.removeprefix(prefix)
        try:
            index = int(index_text)
        except ValueError:
            continue
        summary = parse_recovered_trace_summary(value)
        if summary:
            indexed.append((index, summary))
    return [summary for _, summary in sorted(indexed, key=lambda item: item[0])]


def parsed_count_matches(actual: int, expected: str | None) -> bool:
    if expected is None:
        return actual == 0
    try:
        return actual == int(expected)
    except ValueError:
        return False


def runtime_restart_recovery_report(parsed: dict[str, str]) -> dict[str, Any]:
    recovered_summaries = parse_recovered_trace_summaries(parsed)
    recovered_summary_count = parsed.get("restart-opportunity-trace-recovered-summaries")
    return {
        "plan_checkpoint_recovered": parsed.get("restart-plan-checkpoint-recovered"),
        "adapter_checkpoint_recovered": parsed.get("restart-adapter-checkpoint-recovered"),
        "graceful_shutdown_checkpoint_recovered": parsed.get(
            "restart-graceful-shutdown-checkpoint-recovered"
        ),
        "opportunity_trace_recovered_summary_count": recovered_summary_count,
        "opportunity_trace_recovered_summaries_match_count": parsed_count_matches(
            len(recovered_summaries),
            recovered_summary_count,
        ),
        "opportunity_trace_recovered_summaries": recovered_summaries,
        "opportunity_trace_recovery": {
            "corpus": parsed.get("opportunity-trace-corpus"),
            "discovered": parsed.get("opportunity-trace-discovered"),
            "recovered_checkpoints": parsed.get("opportunity-trace-recovered-checkpoints"),
            "missing_checkpoints": parsed.get("opportunity-trace-missing-checkpoints"),
            "audit_records_replayed": parsed.get("opportunity-trace-audit-records-replayed"),
            "validated": parsed.get("opportunity-trace-recovery-validated") == "true",
        },
    }


def production_runtime_preflight_report(parsed: dict[str, str]) -> dict[str, Any]:
    return {
        "validation_passed": parsed.get("production-runtime-preflight")
        == "validation passed",
        "status": parsed.get("production-runtime-preflight-status"),
        "local_smoke_validated": parsed.get(
            "production-runtime-preflight-local-smoke-validated"
        ),
        "local_smoke_load_validated": parsed.get(
            "production-runtime-preflight-local-smoke-load-validated"
        ),
        "unresolved_blockers": parsed.get(
            "production-runtime-preflight-unresolved-blockers"
        ),
        "service_manager_evidence_available": parsed.get(
            "production-runtime-preflight-service-manager-evidence-available"
        ),
        "disk_full_evidence_available": parsed.get(
            "production-runtime-preflight-disk-full-evidence-available"
        ),
        "retention_execution_evidence_available": parsed.get(
            "production-runtime-preflight-retention-execution-evidence-available"
        ),
        "backup_restore_evidence_available": parsed.get(
            "production-runtime-preflight-backup-restore-evidence-available"
        ),
        "graceful_shutdown_evidence_available": parsed.get(
            "production-runtime-preflight-graceful-shutdown-evidence-available"
        ),
        "audit_sqlite_recovery_evidence_available": parsed.get(
            "production-runtime-preflight-audit-sqlite-recovery-evidence-available"
        ),
        "sqlite_schema_migration_evidence_available": parsed.get(
            "production-runtime-preflight-sqlite-schema-migration-evidence-available"
        ),
        "daemon_failure_capture_evidence_available": parsed.get(
            "production-runtime-preflight-daemon-failure-capture-evidence-available"
        ),
        "concurrent_lifecycle_evidence_available": parsed.get(
            "production-runtime-preflight-concurrent-lifecycle-evidence-available"
        ),
        "production_ready": parsed.get("production-runtime-preflight-production-ready"),
    }


def runtime_load_profile_review_report(parsed: dict[str, str]) -> dict[str, Any]:
    return {
        "status": parsed.get("runtime-load-profile-review"),
        "latency_budget_met": parsed.get("runtime-load-profile-latency-budget-met"),
        "resource_budget_met": parsed.get("runtime-load-profile-resource-budget-met"),
        "replay_recovery_evidence_validated": parsed.get(
            "runtime-load-profile-replay-recovery-evidence-validated"
        ),
        "remaining_external_evidence_count": parsed.get(
            "runtime-load-profile-remaining-external-evidence-count"
        ),
    }


def validate_runtime_load_profile_review(report: dict[str, Any]) -> None:
    load_profile = report.get("runtime_load_profile_review")
    if not isinstance(load_profile, dict):
        raise RuntimeError("runtime load profile review missing")
    expected = {
        "status": "ReadyForLocalReview",
        "latency_budget_met": "true",
        "resource_budget_met": "true",
        "replay_recovery_evidence_validated": "true",
    }
    for key, value in expected.items():
        if load_profile.get(key) != value:
            raise RuntimeError(
                f"runtime load profile review expected {key}={value}"
            )
    try:
        remaining = int(load_profile.get("remaining_external_evidence_count", "0"))
    except ValueError as exc:
        raise RuntimeError(
            "runtime load profile remaining external evidence count is not an integer"
        ) from exc
    if remaining <= 0:
        raise RuntimeError(
            "runtime load profile review must preserve external evidence blockers"
        )


def run_retention_preflight(
    active_path: pathlib.Path | None,
    archive_dir: pathlib.Path | None,
) -> dict[str, Any]:
    active = inspect_candidate_file_path(active_path, "active-audit-journal")
    archive = inspect_candidate_directory_path(archive_dir, "audit-archive-directory")
    passed = (
        active["provided"]
        and archive["provided"]
        and active["usable_for_runtime_artifacts"]
        and archive["usable_for_runtime_artifacts"]
    )
    return {
        "schema": "arbyclaw.deployment_retention_preflight.v1",
        "active_journal": active,
        "archive_directory": archive,
        "retention_preflight_passed": passed,
        "files_changed": False,
        "rotation_performed": False,
        "deletion_performed": False,
        "production_paths_touched": False,
        "service_actions_performed": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
    }


def validate_fresh_workspace(
    workspace: pathlib.Path | None,
    workspace_option: str,
    command_label: str,
) -> pathlib.Path:
    if workspace is None:
        raise ValueError(f"{workspace_option} is required with {command_label}")
    if workspace.exists():
        raise ValueError(f"workspace must be fresh: {relative_or_absolute(workspace)}")
    return workspace


def validate_runtime_smoke_inputs(config: pathlib.Path, workspace: pathlib.Path | None) -> pathlib.Path:
    if not config.exists():
        raise ValueError(f"config does not exist: {relative_or_absolute(config)}")
    return validate_fresh_workspace(
        workspace,
        "--runtime-workspace",
        "--run-runtime-smoke",
    )


def runtime_smoke_command(agent_bin: pathlib.Path | None, config: pathlib.Path, workspace: pathlib.Path) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-smoke",
            "--config",
            str(config),
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-smoke",
        "--config",
        str(config),
        "--workspace",
        str(workspace),
    ]


def audit_retention_execution_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-audit-retention-execution",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-audit-retention-execution",
        "--workspace",
        str(workspace),
    ]


def runtime_config_reload_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-config-reload",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-config-reload",
        "--workspace",
        str(workspace),
    ]


def deployment_static_hardening_command(agent_bin: pathlib.Path | None) -> list[str]:
    command = [
        sys.executable,
        "scripts/validate_deployment_static_hardening.py",
        "--run-config-smoke",
        "--json",
    ]
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        command.extend(["--agent-bin", str(agent_bin)])
    return command


def sqlite_schema_migration_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-sqlite-wal-schema-migration",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-sqlite-wal-schema-migration",
        "--workspace",
        str(workspace),
    ]


def deployment_config_redaction_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-deployment-config-redaction",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-deployment-config-redaction",
        "--workspace",
        str(workspace),
    ]


def deployment_log_redaction_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-deployment-log-redaction",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-deployment-log-redaction",
        "--workspace",
        str(workspace),
    ]


def audit_durability_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-audit-durability",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-audit-durability",
        "--workspace",
        str(workspace),
    ]


def graceful_shutdown_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-graceful-shutdown",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-graceful-shutdown",
        "--workspace",
        str(workspace),
    ]


def backup_restore_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-backup-restore",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-backup-restore",
        "--workspace",
        str(workspace),
    ]


def backup_restore_load_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-backup-restore-load",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-backup-restore-load",
        "--workspace",
        str(workspace),
    ]


def restart_recovery_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-restart-recovery",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-restart-recovery",
        "--workspace",
        str(workspace),
    ]


def incomplete_recovery_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-incomplete-recovery",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-incomplete-recovery",
        "--workspace",
        str(workspace),
    ]


def supervised_restart_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-supervised-restart",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-supervised-restart",
        "--workspace",
        str(workspace),
    ]


def permission_denial_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-permission-denial",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-permission-denial",
        "--workspace",
        str(workspace),
    ]


def blocked_state_preflight_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-blocked-state-preflight",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-blocked-state-preflight",
        "--workspace",
        str(workspace),
    ]


def blocked_audit_preflight_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-blocked-audit-preflight",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-blocked-audit-preflight",
        "--workspace",
        str(workspace),
    ]


def observability_runtime_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-observability-runtime",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-observability-runtime",
        "--workspace",
        str(workspace),
    ]


def observability_metrics_runtime_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-observability-metrics-runtime",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-observability-metrics-runtime",
        "--workspace",
        str(workspace),
    ]


def observability_provider_boundary_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-observability-provider-boundary",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-observability-provider-boundary",
        "--workspace",
        str(workspace),
    ]


def runtime_panic_hook_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-runtime-panic-hook",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-runtime-panic-hook",
        "--workspace",
        str(workspace),
    ]


def dashboard_runtime_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-dashboard-runtime",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-dashboard-runtime",
        "--workspace",
        str(workspace),
    ]


def communications_runtime_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-communications-runtime",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-communications-runtime",
        "--workspace",
        str(workspace),
    ]


def communications_delivery_provider_command(
    agent_bin: pathlib.Path | None, workspace: pathlib.Path
) -> list[str]:
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"agent binary does not exist: {relative_or_absolute(agent_bin)}")
        return [
            str(agent_bin),
            "validate-communications-delivery-provider-boundary",
            "--workspace",
            str(workspace),
        ]

    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("cargo unavailable and --agent-bin was not provided")
    return [
        cargo,
        "run",
        "-p",
        "arb-agent",
        "--",
        "validate-communications-delivery-provider-boundary",
        "--workspace",
        str(workspace),
    ]


def run_runtime_smoke(
    config: pathlib.Path,
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
    iterations: int,
) -> dict[str, Any]:
    smoke_workspace = validate_runtime_smoke_inputs(config, workspace)
    command = runtime_smoke_command(agent_bin, config, smoke_workspace)
    if iterations > 1:
        command.extend(["--iterations", str(iterations)])
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    iteration_reports, has_iterations = parse_runtime_smoke_output(completed.stdout)
    structured_iteration_reports = [
        {
            **iteration_report,
            "runtime_restart_recovery_report": runtime_restart_recovery_report(iteration_report),
        }
        for iteration_report in iteration_reports
    ]

    opportunity_trace_discovered = parsed.get("opportunity-trace-discovered")
    opportunity_trace_recovered = parsed.get("opportunity-trace-recovered-checkpoints")
    opportunity_trace_missing = parsed.get("opportunity-trace-missing-checkpoints")
    opportunity_trace_audit_records = parsed.get("opportunity-trace-audit-records-replayed")
    opportunity_trace_validated = parsed.get("opportunity-trace-recovery-validated") == "true"
    opportunity_trace_corpus = parsed.get("opportunity-trace-corpus")
    restart_recovery_report = runtime_restart_recovery_report(parsed)
    restart_plan_checkpoint_recovered = restart_recovery_report["plan_checkpoint_recovered"]
    restart_adapter_checkpoint_recovered = restart_recovery_report["adapter_checkpoint_recovered"]
    restart_graceful_shutdown_checkpoint_recovered = restart_recovery_report[
        "graceful_shutdown_checkpoint_recovered"
    ]
    restart_opportunity_trace_recovered_summary_count = restart_recovery_report[
        "opportunity_trace_recovered_summary_count"
    ]
    restart_opportunity_trace_recovered_summaries_match_count = restart_recovery_report[
        "opportunity_trace_recovered_summaries_match_count"
    ]
    restart_opportunity_trace_recovered_summaries = restart_recovery_report[
        "opportunity_trace_recovered_summaries"
    ]

    report = {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(smoke_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "restart_plan_checkpoint_recovered": restart_plan_checkpoint_recovered,
        "restart_adapter_checkpoint_recovered": restart_adapter_checkpoint_recovered,
        "restart_graceful_shutdown_checkpoint_recovered": restart_graceful_shutdown_checkpoint_recovered,
        "restart_opportunity_trace_recovered_summary_count": restart_opportunity_trace_recovered_summary_count,
        "restart_opportunity_trace_recovered_summaries_match_count": restart_opportunity_trace_recovered_summaries_match_count,
        "restart_opportunity_trace_recovered_summaries": restart_opportunity_trace_recovered_summaries,
        "opportunity_trace_recovery": {
            "corpus": opportunity_trace_corpus,
            "discovered": opportunity_trace_discovered,
            "recovered_checkpoints": opportunity_trace_recovered,
            "missing_checkpoints": opportunity_trace_missing,
            "audit_records_replayed": opportunity_trace_audit_records,
            "validated": opportunity_trace_validated,
        },
        "concurrent_lifecycle": {
            "validated": parsed.get("concurrent-lifecycle-validated"),
            "workers": parsed.get("concurrent-lifecycle-workers"),
            "audit_records_replayed": parsed.get(
                "concurrent-lifecycle-audit-records-replayed"
            ),
            "sqlite_integrity_check_passed": parsed.get(
                "concurrent-lifecycle-sqlite-integrity-check-passed"
            ),
            "external_submission_performed": parsed.get(
                "concurrent-lifecycle-external-submission-performed"
            ),
            "live_execution_performed": parsed.get(
                "concurrent-lifecycle-live-execution-performed"
            ),
        },
        "production_ready": parsed.get("production-ready"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "runtime_load_profile_review": runtime_load_profile_review_report(parsed),
        "production_runtime_preflight": production_runtime_preflight_report(parsed),
        "runtime_smoke_passed": completed.returncode == 0,
        "iterations": iterations,
        "runtime_restart_recovery_report": restart_recovery_report,
        "runtime_smoke_iteration_reports": structured_iteration_reports,
        "runtime_smoke_iteration_reports_available": has_iterations,
    }
    if completed.returncode == 0:
        validate_runtime_load_profile_review(report)
    return report


def run_audit_retention_execution(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    retention_workspace = validate_fresh_workspace(
        workspace,
        "--retention-workspace",
        "--run-audit-retention-execution",
    )
    command = audit_retention_execution_command(agent_bin, retention_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(retention_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "rotate_active_requested": parsed.get("audit-retention-rotate-active-requested"),
        "new_active_created": parsed.get("audit-retention-new-active-created"),
        "retained_archives": parsed.get("audit-retention-retained-archives"),
        "expired_archives_deleted": parsed.get("audit-retention-expired-archives-deleted"),
        "deleted_file_count": parsed.get("audit-retention-deleted-file-count"),
        "deletion_performed": parsed.get("audit-retention-deletion-performed"),
        "filesystem_mutated": parsed.get("audit-retention-filesystem-mutated"),
        "out_of_workspace_path_touched": parsed.get(
            "audit-retention-out-of-workspace-path-touched"
        ),
        "live_network_used": parsed.get("audit-retention-live-network-used"),
        "external_execution_performed": parsed.get(
            "audit-retention-external-execution-performed"
        ),
        "production_ready": parsed.get("production-ready"),
        "audit_retention_execution_passed": completed.returncode == 0,
    }


def run_audit_durability(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    audit_workspace = validate_fresh_workspace(
        workspace,
        "--audit-durability-workspace",
        "--run-audit-durability",
    )
    command = audit_durability_command(agent_bin, audit_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(audit_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "append_replay_validated": parsed.get("audit-durability-append-replay-validated"),
        "truncated_replay_rejected": parsed.get("audit-durability-truncated-replay-rejected"),
        "tamper_replay_rejected": parsed.get("audit-durability-tamper-replay-rejected"),
        "concurrent_append_validated": parsed.get(
            "audit-durability-concurrent-append-validated"
        ),
        "filesystem_failure_validated": parsed.get(
            "audit-durability-filesystem-failure-validated"
        ),
        "disk_full_failure_validated": parsed.get(
            "audit-durability-disk-full-failure-validated"
        ),
        "append_records": parsed.get("audit-durability-append-records"),
        "concurrent_records": parsed.get("audit-durability-concurrent-records"),
        "live_network_used": parsed.get("audit-durability-live-network-used"),
        "external_execution_performed": parsed.get(
            "audit-durability-external-execution-performed"
        ),
        "unresolved_blockers": parsed.get("audit-durability-unresolved-blockers"),
        "production_ready": parsed.get("production-ready"),
        "audit_durability_passed": completed.returncode == 0,
    }


def run_runtime_config_reload(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    reload_workspace = validate_fresh_workspace(
        workspace,
        "--runtime-config-reload-workspace",
        "--run-runtime-config-reload",
    )
    command = runtime_config_reload_command(agent_bin, reload_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(reload_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "status": parsed.get("runtime-config-reload-status"),
        "initial_mode_safe": parsed.get("initial-mode-safe"),
        "reloaded_mode_safe": parsed.get("reloaded-mode-safe"),
        "reload_change_detected": parsed.get("reload-change-detected"),
        "cex_allowlist_changed": parsed.get("cex-allowlist-changed"),
        "asset_allowlist_changed": parsed.get("asset-allowlist-changed"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "secret_material_loaded": parsed.get("secret-material-loaded"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "runtime_config_reload_passed": completed.returncode == 0,
    }


def run_deployment_static_hardening(agent_bin: pathlib.Path | None) -> dict[str, Any]:
    command = deployment_static_hardening_command(agent_bin)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    report: dict[str, Any] = {}
    if completed.returncode == 0:
        try:
            parsed = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"deployment static hardening did not emit valid JSON: {error}") from error
        config = parsed.get("config", {})
        config_smoke = parsed.get("config_smoke", {})
        report = {
            "schema": parsed.get("schema"),
            "passed": parsed.get("passed"),
            "config_smoke_requested": parsed.get("config_smoke_requested"),
            "config_observe_or_paper_mode": config.get("observe_or_paper_mode")
            if isinstance(config, dict)
            else None,
            "config_live_execution_disabled": config.get("live_execution_disabled")
            if isinstance(config, dict)
            else None,
            "config_secret_like_assignment": config.get("secret_like_assignment")
            if isinstance(config, dict)
            else None,
            "config_smoke_passed": config_smoke.get("passed")
            if isinstance(config_smoke, dict)
            else None,
            "config_smoke_config_loaded": config_smoke.get("config_loaded")
            if isinstance(config_smoke, dict)
            else None,
            "config_smoke_observe_or_paper_mode": config_smoke.get("observe_or_paper_mode")
            if isinstance(config_smoke, dict)
            else None,
            "config_smoke_live_execution_disabled": config_smoke.get("live_execution_disabled")
            if isinstance(config_smoke, dict)
            else None,
            "config_smoke_secret_like_output": config_smoke.get("secret_like_output")
            if isinstance(config_smoke, dict)
            else None,
            "service_actions_performed": parsed.get("service_actions_performed"),
            "network_listeners_started": parsed.get("network_listeners_started"),
            "external_calls_performed": parsed.get("external_calls_performed"),
            "secrets_loaded": parsed.get("secrets_loaded"),
            "live_execution_enabled": parsed.get("live_execution_enabled"),
            "production_readiness_claimed": parsed.get("production_readiness_claimed"),
        }
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "stdout_line_count": len(completed.stdout.splitlines()),
        "deployment_static_hardening_passed": completed.returncode == 0,
    } | report


def run_sqlite_schema_migration(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    migration_workspace = validate_fresh_workspace(
        workspace,
        "--sqlite-schema-migration-workspace",
        "--run-sqlite-schema-migration",
    )
    command = sqlite_schema_migration_command(agent_bin, migration_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(migration_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "status": parsed.get("sqlite-wal-schema-migration-status"),
        "legacy_pre_schema_version": parsed.get("legacy-pre-schema-version"),
        "migrated_schema_version": parsed.get("migrated-schema-version"),
        "expected_schema_version": parsed.get("expected-schema-version"),
        "legacy_checkpoint_preserved": parsed.get("legacy-checkpoint-preserved"),
        "future_version_rejected": parsed.get("future-version-rejected"),
        "migration_performed": parsed.get("migration-performed"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "external_network_used": parsed.get("external-network-used"),
        "secret_material_recorded": parsed.get("secret-material-recorded"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "sqlite_schema_migration_passed": completed.returncode == 0,
    }


def run_deployment_config_redaction(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    redaction_workspace = validate_fresh_workspace(
        workspace,
        "--deployment-config-redaction-workspace",
        "--run-deployment-config-redaction",
    )
    command = deployment_config_redaction_command(agent_bin, redaction_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(redaction_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "config_loaded": parsed.get("config-loaded"),
        "config_mode_safe": parsed.get("config-mode-safe"),
        "audit_redaction_required": parsed.get("audit-redaction-required"),
        "unsafe_metadata_rejected": parsed.get("unsafe-metadata-rejected"),
        "redacted_event_appended": parsed.get("redacted-event-appended"),
        "audit_replay_validated": parsed.get("audit-replay-validated"),
        "secret_material_recorded": parsed.get("secret-material-recorded"),
        "external_network_used": parsed.get("external-network-used"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "deployment_config_redaction_passed": completed.returncode == 0,
    }


def run_deployment_log_redaction(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    redaction_workspace = validate_fresh_workspace(
        workspace,
        "--deployment-log-redaction-workspace",
        "--run-deployment-log-redaction",
    )
    command = deployment_log_redaction_command(agent_bin, redaction_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(redaction_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "sanitized_log_written": parsed.get("sanitized-log-written"),
        "log_redaction_applied": parsed.get("log-redaction-applied"),
        "unsafe_log_material_absent": parsed.get("unsafe-log-material-absent"),
        "unsafe_metadata_rejected": parsed.get("unsafe-metadata-rejected"),
        "redacted_event_appended": parsed.get("redacted-event-appended"),
        "audit_replay_validated": parsed.get("audit-replay-validated"),
        "secret_material_recorded": parsed.get("secret-material-recorded"),
        "external_network_used": parsed.get("external-network-used"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "deployment_log_redaction_passed": completed.returncode == 0,
    }


def run_graceful_shutdown(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    shutdown_workspace = validate_fresh_workspace(
        workspace,
        "--graceful-shutdown-workspace",
        "--run-graceful-shutdown",
    )
    command = graceful_shutdown_command(agent_bin, shutdown_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(shutdown_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-graceful-shutdown-validation"),
        "shutdown_id": parsed.get("runtime-graceful-shutdown-id"),
        "version": parsed.get("runtime-graceful-shutdown-version"),
        "audit_records_replayed": parsed.get(
            "runtime-graceful-shutdown-audit-records-replayed"
        ),
        "start_audit_sequence": parsed.get(
            "runtime-graceful-shutdown-start-audit-sequence"
        ),
        "checkpoint_audit_sequence": parsed.get(
            "runtime-graceful-shutdown-checkpoint-audit-sequence"
        ),
        "checkpoint_key": parsed.get("runtime-graceful-shutdown-checkpoint-key"),
        "checkpoint_recovered": parsed.get(
            "runtime-graceful-shutdown-checkpoint-recovered"
        ),
        "checkpoint_matches_record": parsed.get(
            "runtime-graceful-shutdown-checkpoint-matches-record"
        ),
        "audit_replayed": parsed.get("runtime-graceful-shutdown-audit-replayed"),
        "sqlite_integrity_check_passed": parsed.get(
            "runtime-graceful-shutdown-sqlite-integrity-check-passed"
        ),
        "service_manager_action_performed": parsed.get(
            "runtime-graceful-shutdown-service-manager-action-performed"
        ),
        "external_submission_performed": parsed.get(
            "runtime-graceful-shutdown-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-graceful-shutdown-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-graceful-shutdown-production-ready"),
        "graceful_shutdown_passed": completed.returncode == 0,
    }


def run_backup_restore(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    backup_workspace = validate_fresh_workspace(
        workspace,
        "--backup-restore-workspace",
        "--run-backup-restore",
    )
    command = backup_restore_command(agent_bin, backup_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(backup_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-backup-restore-validation"),
        "lifecycle_id": parsed.get("runtime-backup-restore-lifecycle-id"),
        "version": parsed.get("runtime-backup-restore-version"),
        "audit_records_replayed": parsed.get(
            "runtime-backup-restore-audit-records-replayed"
        ),
        "audit_restore_check_passed": parsed.get(
            "runtime-backup-restore-audit-restore-check-passed"
        ),
        "sqlite_restore_check_passed": parsed.get(
            "runtime-backup-restore-sqlite-restore-check-passed"
        ),
        "plan_checkpoint_restored": parsed.get(
            "runtime-backup-restore-plan-checkpoint-restored"
        ),
        "adapter_checkpoint_restored": parsed.get(
            "runtime-backup-restore-adapter-checkpoint-restored"
        ),
        "adapter_recovery_plan_checkpoint_restored": parsed.get(
            "runtime-backup-restore-adapter-recovery-plan-checkpoint-restored"
        ),
        "external_submission_performed": parsed.get(
            "runtime-backup-restore-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-backup-restore-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-backup-restore-production-ready"),
        "backup_restore_passed": completed.returncode == 0,
    }


def run_backup_restore_load(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    load_workspace = validate_fresh_workspace(
        workspace,
        "--backup-restore-load-workspace",
        "--run-backup-restore-load",
    )
    command = backup_restore_load_command(agent_bin, load_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(load_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-backup-restore-load-validation"),
        "workers": parsed.get("runtime-backup-restore-load-workers"),
        "audit_records_replayed": parsed.get(
            "runtime-backup-restore-load-audit-records-replayed"
        ),
        "audit_restore_check_passed": parsed.get(
            "runtime-backup-restore-load-audit-restore-check-passed"
        ),
        "sqlite_restore_check_passed": parsed.get(
            "runtime-backup-restore-load-sqlite-restore-check-passed"
        ),
        "plan_checkpoint_restored": parsed.get(
            "runtime-backup-restore-load-plan-checkpoint-restored"
        ),
        "adapter_checkpoint_restored": parsed.get(
            "runtime-backup-restore-load-adapter-checkpoint-restored"
        ),
        "adapter_recovery_plan_checkpoint_restored": parsed.get(
            "runtime-backup-restore-load-adapter-recovery-plan-checkpoint-restored"
        ),
        "restart_audit_replay_check_passed": parsed.get(
            "runtime-backup-restore-load-restart-audit-replay-check-passed"
        ),
        "restart_sqlite_reopen_check_passed": parsed.get(
            "runtime-backup-restore-load-restart-sqlite-reopen-check-passed"
        ),
        "backup_journal_sequence_matches": parsed.get(
            "runtime-backup-restore-load-backup-journal-sequence-matches"
        ),
        "external_submission_performed": parsed.get(
            "runtime-backup-restore-load-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-backup-restore-load-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-backup-restore-load-production-ready"),
        "backup_restore_load_passed": completed.returncode == 0,
    }


def run_restart_recovery(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    restart_workspace = validate_fresh_workspace(
        workspace,
        "--restart-recovery-workspace",
        "--run-restart-recovery",
    )
    command = restart_recovery_command(agent_bin, restart_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(restart_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-restart-recovery-validation"),
        "lifecycle_id": parsed.get("runtime-restart-recovery-lifecycle-id"),
        "shutdown_id": parsed.get("runtime-restart-recovery-shutdown-id"),
        "version": parsed.get("runtime-restart-recovery-version"),
        "audit_records_replayed": parsed.get(
            "runtime-restart-recovery-audit-records-replayed"
        ),
        "audit_replay_check_passed": parsed.get(
            "runtime-restart-recovery-audit-replay-check-passed"
        ),
        "sqlite_reopen_check_passed": parsed.get(
            "runtime-restart-recovery-sqlite-reopen-check-passed"
        ),
        "plan_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-plan-checkpoint-recovered"
        ),
        "adapter_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-adapter-checkpoint-recovered"
        ),
        "adapter_recovery_plan_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-adapter-recovery-plan-checkpoint-recovered"
        ),
        "graceful_shutdown_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-graceful-shutdown-checkpoint-recovered"
        ),
        "recovery_disposition": parsed.get("runtime-restart-recovery-disposition"),
        "local_review_ready": parsed.get(
            "runtime-restart-recovery-local-review-ready"
        ),
        "connector_lifecycle_validated": parsed.get(
            "runtime-restart-recovery-connector-lifecycle-validated"
        ),
        "cex_lifecycle_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-cex-lifecycle-checkpoint-recovered"
        ),
        "dex_lifecycle_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-dex-lifecycle-checkpoint-recovered"
        ),
        "opportunity_trace_validated": parsed.get(
            "runtime-restart-recovery-opportunity-trace-validated"
        ),
        "opportunity_trace_discovered_candidates": parsed.get(
            "runtime-restart-recovery-opportunity-trace-discovered-candidates"
        ),
        "opportunity_trace_recovered_checkpoints": parsed.get(
            "runtime-restart-recovery-opportunity-trace-recovered-checkpoints"
        ),
        "opportunity_trace_missing_checkpoints": parsed.get(
            "runtime-restart-recovery-opportunity-trace-missing-checkpoints"
        ),
        "opportunity_trace_recovered_summary_count": parsed.get(
            "runtime-restart-recovery-opportunity-trace-recovered-summary-count"
        ),
        "opportunity_trace_recovered_summaries_match_count": parsed.get(
            "runtime-restart-recovery-opportunity-trace-recovered-summaries-match-count"
        ),
        "external_submission_performed": parsed.get(
            "runtime-restart-recovery-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-restart-recovery-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-restart-recovery-production-ready"),
        "restart_recovery_passed": completed.returncode == 0,
    }


def run_incomplete_recovery(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    incomplete_workspace = validate_fresh_workspace(
        workspace,
        "--incomplete-recovery-workspace",
        "--run-incomplete-recovery",
    )
    command = incomplete_recovery_command(agent_bin, incomplete_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(incomplete_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-incomplete-recovery-validation"),
        "expected_failure": parsed.get("runtime-incomplete-recovery-expected-failure"),
        "audit_records_before_validation": parsed.get(
            "runtime-incomplete-recovery-audit-records-before-validation"
        ),
        "reopened_audit_records": parsed.get(
            "runtime-incomplete-recovery-reopened-audit-records"
        ),
        "plan_checkpoint_recovered": parsed.get(
            "runtime-incomplete-recovery-plan-checkpoint-recovered"
        ),
        "adapter_checkpoint_recovered": parsed.get(
            "runtime-incomplete-recovery-adapter-checkpoint-recovered"
        ),
        "adapter_recovery_plan_checkpoint_recovered": parsed.get(
            "runtime-incomplete-recovery-adapter-recovery-plan-checkpoint-recovered"
        ),
        "service_manager_action_performed": parsed.get(
            "runtime-incomplete-recovery-service-manager-action-performed"
        ),
        "external_submission_performed": parsed.get(
            "runtime-incomplete-recovery-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-incomplete-recovery-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-incomplete-recovery-production-ready"),
        "incomplete_recovery_passed": completed.returncode == 0,
    }


def run_supervised_restart(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    supervised_workspace = validate_fresh_workspace(
        workspace,
        "--supervised-restart-workspace",
        "--run-supervised-restart",
    )
    command = supervised_restart_command(agent_bin, supervised_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(supervised_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-supervised-restart-validation"),
        "child_exit_code": parsed.get("runtime-supervised-restart-child-exit-code"),
        "child_stdout_lines": parsed.get(
            "runtime-supervised-restart-child-stdout-lines"
        ),
        "audit_records_replayed": parsed.get(
            "runtime-restart-recovery-audit-records-replayed"
        ),
        "sqlite_reopen_check_passed": parsed.get(
            "runtime-restart-recovery-sqlite-reopen-check-passed"
        ),
        "plan_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-plan-checkpoint-recovered"
        ),
        "adapter_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-adapter-checkpoint-recovered"
        ),
        "adapter_recovery_plan_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-adapter-recovery-plan-checkpoint-recovered"
        ),
        "graceful_shutdown_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-graceful-shutdown-checkpoint-recovered"
        ),
        "connector_lifecycle_validated": parsed.get(
            "runtime-restart-recovery-connector-lifecycle-validated"
        ),
        "cex_lifecycle_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-cex-lifecycle-checkpoint-recovered"
        ),
        "dex_lifecycle_checkpoint_recovered": parsed.get(
            "runtime-restart-recovery-dex-lifecycle-checkpoint-recovered"
        ),
        "opportunity_trace_recovered_checkpoints": parsed.get(
            "runtime-restart-recovery-opportunity-trace-recovered-checkpoints"
        ),
        "external_submission_performed": parsed.get(
            "runtime-restart-recovery-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-restart-recovery-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-restart-recovery-production-ready"),
        "supervised_restart_passed": completed.returncode == 0,
    }


def run_permission_denial(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    permission_workspace = validate_fresh_workspace(
        workspace,
        "--permission-denial-workspace",
        "--run-permission-denial",
    )
    command = permission_denial_command(agent_bin, permission_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(permission_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-permission-denial-validation"),
        "expected_failure": parsed.get("runtime-permission-denial-expected-failure"),
        "state_put_attempts": parsed.get("runtime-permission-denial-state-put-attempts"),
        "audit_records_replayed": parsed.get(
            "runtime-permission-denial-audit-records-replayed"
        ),
        "reopened_audit_records": parsed.get(
            "runtime-permission-denial-reopened-audit-records"
        ),
        "adapter_evaluated": parsed.get("runtime-permission-denial-adapter-evaluated"),
        "service_manager_action_performed": parsed.get(
            "runtime-permission-denial-service-manager-action-performed"
        ),
        "external_submission_performed": parsed.get(
            "runtime-permission-denial-external-submission-performed"
        ),
        "live_execution_performed": parsed.get(
            "runtime-permission-denial-live-execution-performed"
        ),
        "production_ready": parsed.get("runtime-permission-denial-production-ready"),
        "permission_denial_passed": completed.returncode == 0,
    }


def run_blocked_state_preflight(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    blocked_state_workspace = validate_fresh_workspace(
        workspace,
        "--blocked-state-workspace",
        "--run-blocked-state-preflight",
    )
    command = blocked_state_preflight_command(agent_bin, blocked_state_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(blocked_state_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "expected_failure": parsed.get("runtime-blocked-state-expected-failure"),
        "artifacts_created": parsed.get("runtime-blocked-state-artifacts-created"),
        "audit_created": parsed.get("runtime-blocked-state-audit-created"),
        "backup_audit_created": parsed.get("runtime-blocked-state-backup-audit-created"),
        "backup_state_created": parsed.get("runtime-blocked-state-backup-state-created"),
        "audit_workspace_created": parsed.get(
            "runtime-blocked-state-audit-workspace-created"
        ),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "blocked_state_preflight_passed": completed.returncode == 0,
    }


def run_blocked_audit_preflight(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    blocked_audit_workspace = validate_fresh_workspace(
        workspace,
        "--blocked-audit-workspace",
        "--run-blocked-audit-preflight",
    )
    command = blocked_audit_preflight_command(agent_bin, blocked_audit_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(blocked_audit_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "expected_failure": parsed.get("runtime-blocked-audit-expected-failure"),
        "artifacts_created": parsed.get("runtime-blocked-audit-artifacts-created"),
        "placeholder_created": parsed.get("runtime-blocked-audit-placeholder-created"),
        "state_created": parsed.get("runtime-blocked-audit-state-created"),
        "backup_audit_created": parsed.get("runtime-blocked-audit-backup-audit-created"),
        "backup_state_created": parsed.get("runtime-blocked-audit-backup-state-created"),
        "audit_workspace_created": parsed.get(
            "runtime-blocked-audit-audit-workspace-created"
        ),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "blocked_audit_preflight_passed": completed.returncode == 0,
    }


def run_filesystem_preflight(
    audit_path: pathlib.Path | None,
    state_path: pathlib.Path | None,
) -> dict[str, Any]:
    audit = inspect_candidate_file_path(audit_path, "audit")
    state = inspect_candidate_file_path(state_path, "state")
    passed = bool(
        audit["usable_for_runtime_artifacts"]
        and state["usable_for_runtime_artifacts"]
        and audit.get("path") != state.get("path")
    )
    return {
        "audit": audit,
        "state": state,
        "audit_state_paths_distinct": audit.get("path") != state.get("path"),
        "filesystem_preflight_passed": passed,
        "filesystem_mutated": False,
        "service_manager_action_performed": False,
        "external_submission_performed": False,
        "live_execution_performed": False,
        "production_ready": False,
        "unresolved_blockers": [
            "non-mutating preflight does not create, open, lock, or fsync production audit/state files",
            "non-mutating preflight does not prove runtime behavior under service-manager orchestration",
            "non-mutating preflight does not prove physical disk-full behavior",
        ],
    }


def run_observability_runtime(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    observability_workspace = validate_fresh_workspace(
        workspace,
        "--observability-workspace",
        "--run-observability-runtime",
    )
    command = observability_runtime_command(agent_bin, observability_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(observability_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "audit_records_replayed": parsed.get("observability-runtime-audit-records-replayed"),
        "checkpoints_recovered": parsed.get("observability-runtime-checkpoints-recovered"),
        "metric_lines": parsed.get("observability-runtime-metric-lines"),
        "scrape_metric_lines": parsed.get("observability-runtime-scrape-metric-lines"),
        "served_metric_lines": parsed.get("observability-runtime-served-metric-lines"),
        "loopback_bind_validated": parsed.get(
            "observability-runtime-loopback-bind-validated"
        ),
        "listener_opened_and_closed": parsed.get(
            "observability-runtime-listener-opened-and-closed"
        ),
        "local_metrics_endpoint_started": parsed.get("local-metrics-endpoint-started"),
        "metrics_endpoint_started": parsed.get("metrics-endpoint-started"),
        "network_request_served": parsed.get("network-request-served"),
        "public_network_exposed": parsed.get("public-network-exposed"),
        "telemetry_exported": parsed.get("telemetry-exported"),
        "outbound_alerts_sent": parsed.get("outbound-alerts-sent"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "observability_runtime_passed": completed.returncode == 0,
    }


def run_observability_metrics_runtime(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    metrics_workspace = validate_fresh_workspace(
        workspace,
        "--observability-metrics-workspace",
        "--run-observability-metrics-runtime",
    )
    command = observability_metrics_runtime_command(agent_bin, metrics_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(metrics_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "audit_records_replayed": parsed.get(
            "observability-metrics-runtime-audit-records-replayed"
        ),
        "checkpoint_recovered": parsed.get(
            "observability-metrics-runtime-checkpoint-recovered"
        ),
        "loopback_bind_validated": parsed.get(
            "observability-metrics-runtime-loopback-bind-validated"
        ),
        "expected_scrapes": parsed.get("observability-metrics-runtime-expected-scrapes"),
        "served_scrapes": parsed.get("observability-metrics-runtime-served-scrapes"),
        "all_scrapes_returned_ok": parsed.get(
            "observability-metrics-runtime-all-scrapes-returned-ok"
        ),
        "response_lines_consistent": parsed.get(
            "observability-metrics-runtime-response-lines-consistent"
        ),
        "response_metric_lines": parsed.get(
            "observability-metrics-runtime-response-metric-lines"
        ),
        "local_metrics_runtime_started": parsed.get("observability-metrics-runtime-started"),
        "local_metrics_runtime_shutdown": parsed.get(
            "observability-metrics-runtime-shutdown"
        ),
        "public_network_exposed": parsed.get(
            "observability-metrics-runtime-public-network-exposed"
        ),
        "telemetry_exported": parsed.get("observability-metrics-runtime-telemetry-exported"),
        "outbound_alerts_sent": parsed.get(
            "observability-metrics-runtime-outbound-alerts-sent"
        ),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "observability_metrics_runtime_passed": completed.returncode == 0,
    }


def run_observability_provider_boundary(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    provider_workspace = validate_fresh_workspace(
        workspace,
        "--observability-provider-boundary-workspace",
        "--run-observability-provider-boundary",
    )
    command = observability_provider_boundary_command(agent_bin, provider_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(provider_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "audit_records_replayed": parsed.get(
            "observability-provider-boundary-audit-records-replayed"
        ),
        "checkpoint_recovered": parsed.get(
            "observability-provider-boundary-checkpoint-recovered"
        ),
        "status": parsed.get("observability-provider-boundary-status"),
        "operations_review_ready": parsed.get(
            "observability-provider-boundary-operations-review-ready"
        ),
        "export_dry_run_ready": parsed.get(
            "observability-provider-boundary-export-dry-run-ready"
        ),
        "alert_route_dispatch_ready": parsed.get(
            "observability-provider-boundary-alert-route-dispatch-ready"
        ),
        "endpoint_preflight_ready": parsed.get(
            "observability-provider-boundary-endpoint-preflight-ready"
        ),
        "metrics_runtime_ready": parsed.get(
            "observability-provider-boundary-metrics-runtime-ready"
        ),
        "missing_local_controls": parsed.get(
            "observability-provider-boundary-missing-local-controls"
        ),
        "remaining_provider_evidence_count": parsed.get(
            "observability-provider-boundary-remaining-provider-evidence-count"
        ),
        "provider_validation_performed": parsed.get(
            "observability-provider-boundary-provider-validation-performed"
        ),
        "public_network_exposed": parsed.get("public-network-exposed"),
        "telemetry_exported": parsed.get("telemetry-exported"),
        "outbound_alerts_sent": parsed.get("outbound-alerts-sent"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "sensitive_material_loaded": parsed.get("sensitive-material-loaded"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "observability_provider_boundary_passed": completed.returncode == 0,
    }


def run_runtime_panic_hook(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    panic_hook_workspace = validate_fresh_workspace(
        workspace,
        "--runtime-panic-hook-workspace",
        "--run-runtime-panic-hook",
    )
    command = runtime_panic_hook_command(agent_bin, panic_hook_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(panic_hook_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "validation": parsed.get("runtime-panic-hook-validation"),
        "hook_installed": parsed.get("runtime-panic-hook-installed"),
        "hook_restored": parsed.get("runtime-panic-hook-restored"),
        "panic_observed": parsed.get("runtime-panic-hook-panic-observed"),
        "panic_captured": parsed.get("runtime-panic-hook-panic-captured"),
        "audit_records_replayed": parsed.get("runtime-panic-hook-audit-records-replayed"),
        "failure_checkpoint_recovered": parsed.get(
            "runtime-panic-hook-failure-checkpoint-recovered"
        ),
        "failure_checkpoint_contains_sentinel": parsed.get(
            "runtime-panic-hook-failure-checkpoint-contains-sentinel"
        ),
        "metrics_endpoint_started": parsed.get("runtime-panic-hook-metrics-endpoint-started"),
        "public_network_exposed": parsed.get("runtime-panic-hook-public-network-exposed"),
        "outbound_alerts_sent": parsed.get("runtime-panic-hook-outbound-alerts-sent"),
        "external_submission_performed": parsed.get(
            "runtime-panic-hook-external-submission-performed"
        ),
        "live_execution_performed": parsed.get("runtime-panic-hook-live-execution-performed"),
        "production_ready": parsed.get("runtime-panic-hook-production-ready"),
        "runtime_panic_hook_passed": completed.returncode == 0,
    }


def run_dashboard_runtime(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    dashboard_workspace = validate_fresh_workspace(
        workspace,
        "--dashboard-workspace",
        "--run-dashboard-runtime",
    )
    command = dashboard_runtime_command(agent_bin, dashboard_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(dashboard_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "audit_records_replayed": parsed.get("dashboard-runtime-audit-records-replayed"),
        "checkpoints_recovered": parsed.get("dashboard-runtime-checkpoints-recovered"),
        "render_access_authorized": parsed.get("dashboard-render-access-authorized"),
        "render_panel_count": parsed.get("dashboard-render-panel-count"),
        "hosted_security_ready": parsed.get("dashboard-hosted-security-ready"),
        "hosted_request_preflight_ready": parsed.get(
            "dashboard-hosted-request-preflight-ready"
        ),
        "hosted_request_validation_ready": parsed.get(
            "dashboard-hosted-request-validation-ready"
        ),
        "hosted_runtime_readiness_review_ready": parsed.get(
            "dashboard-hosted-runtime-readiness-review-ready"
        ),
        "hosted_runtime_security_review_ready": parsed.get(
            "dashboard-hosted-runtime-security-review-ready"
        ),
        "hosted_runtime_preflight_ready": parsed.get(
            "dashboard-hosted-runtime-preflight-ready"
        ),
        "hosted_runtime_session_ready": parsed.get(
            "dashboard-hosted-runtime-session-ready"
        ),
        "hosted_runtime_accepted_request_validated": parsed.get(
            "dashboard-hosted-runtime-accepted-request-validated"
        ),
        "hosted_runtime_unauthenticated_rejection_validated": parsed.get(
            "dashboard-hosted-runtime-unauthenticated-rejection-validated"
        ),
        "hosted_runtime_csrf_rejection_validated": parsed.get(
            "dashboard-hosted-runtime-csrf-rejection-validated"
        ),
        "hosted_runtime_rate_limit_rejection_validated": parsed.get(
            "dashboard-hosted-runtime-rate-limit-rejection-validated"
        ),
        "hosted_runtime_loopback_serving_validated": parsed.get(
            "dashboard-hosted-runtime-loopback-serving-validated"
        ),
        "hosted_runtime_secure_headers_validated": parsed.get(
            "dashboard-hosted-runtime-secure-headers-validated"
        ),
        "hosted_runtime_remaining_external_evidence_count": parsed.get(
            "dashboard-hosted-runtime-remaining-external-evidence-count"
        ),
        "local_dashboard_server_started": parsed.get("local-dashboard-server-started"),
        "persistent_dashboard_server_started": parsed.get(
            "persistent-dashboard-server-started"
        ),
        "network_request_served": parsed.get("network-request-served"),
        "local_http_status_code": parsed.get("local-http-status-code"),
        "public_network_exposed": parsed.get("public-network-exposed"),
        "live_controls_enabled": parsed.get("live-controls-enabled"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "production_ready": parsed.get("production-ready"),
        "dashboard_runtime_passed": completed.returncode == 0,
    }


def run_communications_runtime(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    communications_workspace = validate_fresh_workspace(
        workspace,
        "--communications-workspace",
        "--run-communications-runtime",
    )
    command = communications_runtime_command(agent_bin, communications_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(communications_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "audit_records_replayed": parsed.get("communications-runtime-audit-records-replayed"),
        "checkpoints_recovered": parsed.get("communications-runtime-checkpoints-recovered"),
        "command_route_accepted": parsed.get("command-route-accepted"),
        "command_operator_authorized": parsed.get("command-operator-authorized"),
        "remote_command_security_ready": parsed.get("remote-command-security-ready"),
        "platform_command_ingress_ready": parsed.get("platform-command-ingress-ready"),
        "platform_command_token_reference_present": parsed.get(
            "platform-command-token-reference-present"
        ),
        "platform_command_token_secret_material_present": parsed.get(
            "platform-command-token-secret-material-present"
        ),
        "platform_command_signature_verified": parsed.get(
            "platform-command-signature-verified"
        ),
        "platform_command_identity_authorized": parsed.get(
            "platform-command-identity-authorized"
        ),
        "platform_command_channel_permission_granted": parsed.get(
            "platform-command-channel-permission-granted"
        ),
        "platform_command_replay_nonce_reused": parsed.get(
            "platform-command-replay-nonce-reused"
        ),
        "platform_command_injection_detected": parsed.get(
            "platform-command-injection-detected"
        ),
        "platform_command_provider_rate_limited": parsed.get(
            "platform-command-provider-rate-limited"
        ),
        "platform_command_provider_outage_observed": parsed.get(
            "platform-command-provider-outage-observed"
        ),
        "remote_command_injection_detected": parsed.get(
            "remote-command-injection-detected"
        ),
        "channel_session_ready": parsed.get("channel-session-ready"),
        "channel_session_validations": parsed.get("channel-session-validations"),
        "channel_session_accepted": parsed.get("channel-session-accepted"),
        "channel_session_rejected_unauthenticated": parsed.get(
            "channel-session-rejected-unauthenticated"
        ),
        "channel_session_rejected_replay": parsed.get("channel-session-rejected-replay"),
        "channel_session_rejected_provider_unavailable": parsed.get(
            "channel-session-rejected-provider-unavailable"
        ),
        "platform_adapter_ready": parsed.get("platform-adapter-ready"),
        "platform_adapter_token_reference_present": parsed.get(
            "platform-adapter-token-reference-present"
        ),
        "platform_adapter_token_secret_material_present": parsed.get(
            "platform-adapter-token-secret-material-present"
        ),
        "platform_adapter_identity_verified": parsed.get(
            "platform-adapter-identity-verified"
        ),
        "platform_adapter_identity_authorized": parsed.get(
            "platform-adapter-identity-authorized"
        ),
        "platform_adapter_channel_permission_granted": parsed.get(
            "platform-adapter-channel-permission-granted"
        ),
        "platform_adapter_command_injection_blocked": parsed.get(
            "platform-adapter-command-injection-blocked"
        ),
        "platform_adapter_token_revoked": parsed.get("platform-adapter-token-revoked"),
        "platform_adapter_provider_rate_limited": parsed.get(
            "platform-adapter-provider-rate-limited"
        ),
        "platform_adapter_provider_outage_observed": parsed.get(
            "platform-adapter-provider-outage-observed"
        ),
        "notification_dispatch_status": parsed.get("notification-dispatch-status"),
        "notification_channel_count": parsed.get("notification-channel-count"),
        "outbound_network_used": parsed.get("outbound-network-used"),
        "remote_commands_enabled": parsed.get("remote-commands-enabled"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "signing_or_broadcast_performed": parsed.get("signing-or-broadcast-performed"),
        "production_ready": parsed.get("production-ready"),
        "communications_runtime_passed": completed.returncode == 0,
    }


def run_communications_delivery_provider(
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    delivery_workspace = validate_fresh_workspace(
        workspace,
        "--communications-delivery-provider-workspace",
        "--run-communications-delivery-provider-boundary",
    )
    command = communications_delivery_provider_command(agent_bin, delivery_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(delivery_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "status": parsed.get("communications-delivery-provider-boundary-status"),
        "channel_session_ready": parsed.get(
            "communications-delivery-provider-channel-session-ready"
        ),
        "platform_adapter_ready": parsed.get(
            "communications-delivery-provider-platform-adapter-ready"
        ),
        "delivery_evidence_available": parsed.get(
            "communications-delivery-provider-delivery-evidence-available"
        ),
        "rate_limit_evidence_available": parsed.get(
            "communications-delivery-provider-rate-limit-evidence-available"
        ),
        "outage_evidence_available": parsed.get(
            "communications-delivery-provider-outage-evidence-available"
        ),
        "platform_identity_evidence_available": parsed.get(
            "communications-delivery-provider-platform-identity-evidence-available"
        ),
        "remaining_external_evidence_count": parsed.get(
            "communications-delivery-provider-remaining-external-evidence-count"
        ),
        "blocker_count": parsed.get("communications-delivery-provider-blocker-count"),
        "audit_records_replayed": parsed.get(
            "communications-delivery-provider-audit-records-replayed"
        ),
        "checkpoints_recovered": parsed.get(
            "communications-delivery-provider-checkpoints-recovered"
        ),
        "outbound_network_used": parsed.get("outbound-network-used"),
        "message_delivered": parsed.get("message-delivered"),
        "provider_call_performed": parsed.get("provider-call-performed"),
        "token_secret_material_loaded": parsed.get("token-secret-material-loaded"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "signing_or_broadcast_performed": parsed.get("signing-or-broadcast-performed"),
        "production_ready": parsed.get("production-ready"),
        "communications_delivery_provider_passed": completed.returncode == 0,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    systemd_report = run_systemd_lifecycle(args.systemd_mode, args.unit)
    runtime_report = None
    if args.run_runtime_smoke:
        runtime_report = run_runtime_smoke(
            args.config,
            args.runtime_workspace,
            args.agent_bin,
            args.runtime_smoke_iterations,
        )
        if runtime_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-smoke failed")
    retention_report = None
    if args.run_audit_retention_execution:
        retention_report = run_audit_retention_execution(
            args.retention_workspace,
            args.agent_bin,
        )
        if retention_report["returncode"] != 0:
            raise RuntimeError("validate-audit-retention-execution failed")
    audit_durability_report = None
    if args.run_audit_durability:
        audit_durability_report = run_audit_durability(
            args.audit_durability_workspace,
            args.agent_bin,
        )
        if audit_durability_report["returncode"] != 0:
            raise RuntimeError("validate-audit-durability failed")
    runtime_config_reload_report = None
    if args.run_runtime_config_reload:
        runtime_config_reload_report = run_runtime_config_reload(
            args.runtime_config_reload_workspace,
            args.agent_bin,
        )
        if runtime_config_reload_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-config-reload failed")
    deployment_static_hardening_report = None
    if args.run_deployment_static_hardening:
        deployment_static_hardening_report = run_deployment_static_hardening(
            args.agent_bin,
        )
        if deployment_static_hardening_report["returncode"] != 0:
            raise RuntimeError("validate-deployment-static-hardening failed")
    sqlite_schema_migration_report = None
    if args.run_sqlite_schema_migration:
        sqlite_schema_migration_report = run_sqlite_schema_migration(
            args.sqlite_schema_migration_workspace,
            args.agent_bin,
        )
        if sqlite_schema_migration_report["returncode"] != 0:
            raise RuntimeError("validate-sqlite-wal-schema-migration failed")
    deployment_config_redaction_report = None
    if args.run_deployment_config_redaction:
        deployment_config_redaction_report = run_deployment_config_redaction(
            args.deployment_config_redaction_workspace,
            args.agent_bin,
        )
        if deployment_config_redaction_report["returncode"] != 0:
            raise RuntimeError("validate-deployment-config-redaction failed")
    deployment_log_redaction_report = None
    if args.run_deployment_log_redaction:
        deployment_log_redaction_report = run_deployment_log_redaction(
            args.deployment_log_redaction_workspace,
            args.agent_bin,
        )
        if deployment_log_redaction_report["returncode"] != 0:
            raise RuntimeError("validate-deployment-log-redaction failed")
    graceful_shutdown_report = None
    if args.run_graceful_shutdown:
        graceful_shutdown_report = run_graceful_shutdown(
            args.graceful_shutdown_workspace,
            args.agent_bin,
        )
        if graceful_shutdown_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-graceful-shutdown failed")
    backup_restore_report = None
    if args.run_backup_restore:
        backup_restore_report = run_backup_restore(
            args.backup_restore_workspace,
            args.agent_bin,
        )
        if backup_restore_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-backup-restore failed")
    backup_restore_load_report = None
    if args.run_backup_restore_load:
        backup_restore_load_report = run_backup_restore_load(
            args.backup_restore_load_workspace,
            args.agent_bin,
        )
        if backup_restore_load_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-backup-restore-load failed")
    restart_recovery_report = None
    if args.run_restart_recovery:
        restart_recovery_report = run_restart_recovery(
            args.restart_recovery_workspace,
            args.agent_bin,
        )
        if restart_recovery_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-restart-recovery failed")
    incomplete_recovery_report = None
    if args.run_incomplete_recovery:
        incomplete_recovery_report = run_incomplete_recovery(
            args.incomplete_recovery_workspace,
            args.agent_bin,
        )
        if incomplete_recovery_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-incomplete-recovery failed")
    supervised_restart_report = None
    if args.run_supervised_restart:
        supervised_restart_report = run_supervised_restart(
            args.supervised_restart_workspace,
            args.agent_bin,
        )
        if supervised_restart_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-supervised-restart failed")
    permission_denial_report = None
    if args.run_permission_denial:
        permission_denial_report = run_permission_denial(
            args.permission_denial_workspace,
            args.agent_bin,
        )
        if permission_denial_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-permission-denial failed")
    blocked_state_report = None
    if args.run_blocked_state_preflight:
        blocked_state_report = run_blocked_state_preflight(
            args.blocked_state_workspace,
            args.agent_bin,
        )
        if blocked_state_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-blocked-state-preflight failed")
    blocked_audit_report = None
    if args.run_blocked_audit_preflight:
        blocked_audit_report = run_blocked_audit_preflight(
            args.blocked_audit_workspace,
            args.agent_bin,
        )
        if blocked_audit_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-blocked-audit-preflight failed")
    filesystem_report = None
    if args.run_filesystem_preflight:
        filesystem_report = run_filesystem_preflight(
            args.filesystem_audit_path,
            args.filesystem_state_path,
        )
        if not filesystem_report["filesystem_preflight_passed"]:
            raise RuntimeError("deployment filesystem preflight failed")
    retention_preflight_report = None
    if args.run_retention_preflight:
        retention_preflight_report = run_retention_preflight(
            args.retention_active_path,
            args.retention_archive_dir,
        )
        if not retention_preflight_report["retention_preflight_passed"]:
            raise RuntimeError("deployment retention preflight failed")
    observability_report = None
    if args.run_observability_runtime:
        observability_report = run_observability_runtime(
            args.observability_workspace,
            args.agent_bin,
        )
        if observability_report["returncode"] != 0:
            raise RuntimeError("validate-observability-runtime failed")
    observability_metrics_report = None
    if args.run_observability_metrics_runtime:
        observability_metrics_report = run_observability_metrics_runtime(
            args.observability_metrics_workspace,
            args.agent_bin,
        )
        if observability_metrics_report["returncode"] != 0:
            raise RuntimeError("validate-observability-metrics-runtime failed")
    observability_provider_boundary_report = None
    if args.run_observability_provider_boundary:
        observability_provider_boundary_report = run_observability_provider_boundary(
            args.observability_provider_boundary_workspace,
            args.agent_bin,
        )
        if observability_provider_boundary_report["returncode"] != 0:
            raise RuntimeError("validate-observability-provider-boundary failed")
    panic_hook_report = None
    if args.run_runtime_panic_hook:
        panic_hook_report = run_runtime_panic_hook(
            args.runtime_panic_hook_workspace,
            args.agent_bin,
        )
        if panic_hook_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-panic-hook failed")
    dashboard_report = None
    if args.run_dashboard_runtime:
        dashboard_report = run_dashboard_runtime(
            args.dashboard_workspace,
            args.agent_bin,
        )
        if dashboard_report["returncode"] != 0:
            raise RuntimeError("validate-dashboard-runtime failed")
    communications_report = None
    if args.run_communications_runtime:
        communications_report = run_communications_runtime(
            args.communications_workspace,
            args.agent_bin,
        )
        if communications_report["returncode"] != 0:
            raise RuntimeError("validate-communications-runtime failed")
    communications_delivery_report = None
    if args.run_communications_delivery_provider_boundary:
        communications_delivery_report = run_communications_delivery_provider(
            args.communications_delivery_provider_workspace,
            args.agent_bin,
        )
        if communications_delivery_report["returncode"] != 0:
            raise RuntimeError("validate-communications-delivery-provider-boundary failed")

    return {
        "schema": "arbyclaw.deployment_host_runtime_validation.v1",
        "systemd_lifecycle": systemd_report,
        "runtime_smoke": runtime_report,
        "runtime_smoke_requested": args.run_runtime_smoke,
        "audit_retention_execution": retention_report,
        "audit_retention_execution_requested": args.run_audit_retention_execution,
        "audit_durability": audit_durability_report,
        "audit_durability_requested": args.run_audit_durability,
        "runtime_config_reload": runtime_config_reload_report,
        "runtime_config_reload_requested": args.run_runtime_config_reload,
        "deployment_static_hardening": deployment_static_hardening_report,
        "deployment_static_hardening_requested": args.run_deployment_static_hardening,
        "sqlite_schema_migration": sqlite_schema_migration_report,
        "sqlite_schema_migration_requested": args.run_sqlite_schema_migration,
        "deployment_config_redaction": deployment_config_redaction_report,
        "deployment_config_redaction_requested": args.run_deployment_config_redaction,
        "deployment_log_redaction": deployment_log_redaction_report,
        "deployment_log_redaction_requested": args.run_deployment_log_redaction,
        "graceful_shutdown": graceful_shutdown_report,
        "graceful_shutdown_requested": args.run_graceful_shutdown,
        "backup_restore": backup_restore_report,
        "backup_restore_requested": args.run_backup_restore,
        "backup_restore_load": backup_restore_load_report,
        "backup_restore_load_requested": args.run_backup_restore_load,
        "restart_recovery": restart_recovery_report,
        "restart_recovery_requested": args.run_restart_recovery,
        "incomplete_recovery": incomplete_recovery_report,
        "incomplete_recovery_requested": args.run_incomplete_recovery,
        "supervised_restart": supervised_restart_report,
        "supervised_restart_requested": args.run_supervised_restart,
        "permission_denial": permission_denial_report,
        "permission_denial_requested": args.run_permission_denial,
        "blocked_state_preflight": blocked_state_report,
        "blocked_state_preflight_requested": args.run_blocked_state_preflight,
        "blocked_audit_preflight": blocked_audit_report,
        "blocked_audit_preflight_requested": args.run_blocked_audit_preflight,
        "filesystem_preflight": filesystem_report,
        "filesystem_preflight_requested": args.run_filesystem_preflight,
        "retention_preflight": retention_preflight_report,
        "retention_preflight_requested": args.run_retention_preflight,
        "observability_runtime": observability_report,
        "observability_runtime_requested": args.run_observability_runtime,
        "observability_metrics_runtime": observability_metrics_report,
        "observability_metrics_runtime_requested": args.run_observability_metrics_runtime,
        "observability_provider_boundary": observability_provider_boundary_report,
        "observability_provider_boundary_requested": args.run_observability_provider_boundary,
        "runtime_panic_hook": panic_hook_report,
        "runtime_panic_hook_requested": args.run_runtime_panic_hook,
        "dashboard_runtime": dashboard_report,
        "dashboard_runtime_requested": args.run_dashboard_runtime,
        "communications_runtime": communications_report,
        "communications_runtime_requested": args.run_communications_runtime,
        "communications_delivery_provider": communications_delivery_report,
        "communications_delivery_provider_requested": args.run_communications_delivery_provider_boundary,
        "bounded_timeouts": {
            "systemd_lifecycle_seconds": SYSTEMD_LIFECYCLE_TIMEOUT_SECONDS,
            "local_runtime_helper_seconds": LOCAL_RUNTIME_HELPER_TIMEOUT_SECONDS,
        },
        "service_actions_performed": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "external_calls_performed": False,
        "production_readiness_claimed": False,
        "remaining_external_evidence": [
            "operator-controlled service start/shutdown/restart evidence",
            "deployment-host audit and SQLite recovery evidence under service lifecycle",
            "deployment-host SQLite schema migration execution evidence beyond local fixture validation",
            "deployment-host config loading under service lifecycle beyond local static/config-smoke validation",
            "deployment-host log/audit redaction under service lifecycle beyond local sanitized fixture validation",
            "physical disk-full fail-closed evidence",
            "deployment-host retention/rotation execution evidence",
            "rollback drill evidence",
            "incident-response drill evidence",
            "daemon failure-capture execution evidence",
        ],
    }


def print_text_report(report: dict[str, Any]) -> None:
    systemd = report["systemd_lifecycle"]
    print("deployment-host runtime validation report")
    print(f"systemd mode: {systemd['mode']}")
    print(f"unit: {systemd['unit']}")
    timeouts = report["bounded_timeouts"]
    print(f"systemd lifecycle timeout seconds: {timeouts['systemd_lifecycle_seconds']}")
    print(f"local runtime helper timeout seconds: {timeouts['local_runtime_helper_seconds']}")
    print(f"runtime smoke requested: {str(report['runtime_smoke_requested']).lower()}")
    print(
        "audit retention execution requested: "
        f"{str(report['audit_retention_execution_requested']).lower()}"
    )
    print(
        "audit durability requested: "
        f"{str(report['audit_durability_requested']).lower()}"
    )
    print(
        "runtime config reload requested: "
        f"{str(report['runtime_config_reload_requested']).lower()}"
    )
    print(
        "deployment static hardening requested: "
        f"{str(report['deployment_static_hardening_requested']).lower()}"
    )
    print(
        "sqlite schema migration requested: "
        f"{str(report['sqlite_schema_migration_requested']).lower()}"
    )
    print(
        "deployment config redaction requested: "
        f"{str(report['deployment_config_redaction_requested']).lower()}"
    )
    print(
        "deployment log redaction requested: "
        f"{str(report['deployment_log_redaction_requested']).lower()}"
    )
    print(
        "graceful shutdown requested: "
        f"{str(report['graceful_shutdown_requested']).lower()}"
    )
    print(
        "backup restore requested: "
        f"{str(report['backup_restore_requested']).lower()}"
    )
    print(
        "backup restore load requested: "
        f"{str(report['backup_restore_load_requested']).lower()}"
    )
    print(
        "restart recovery requested: "
        f"{str(report['restart_recovery_requested']).lower()}"
    )
    print(
        "incomplete recovery requested: "
        f"{str(report['incomplete_recovery_requested']).lower()}"
    )
    print(
        "supervised restart requested: "
        f"{str(report['supervised_restart_requested']).lower()}"
    )
    print(
        "permission denial requested: "
        f"{str(report['permission_denial_requested']).lower()}"
    )
    print(
        "blocked state preflight requested: "
        f"{str(report['blocked_state_preflight_requested']).lower()}"
    )
    print(
        "blocked audit preflight requested: "
        f"{str(report['blocked_audit_preflight_requested']).lower()}"
    )
    print(
        "filesystem preflight requested: "
        f"{str(report['filesystem_preflight_requested']).lower()}"
    )
    print(
        "retention preflight requested: "
        f"{str(report['retention_preflight_requested']).lower()}"
    )
    print(
        "observability runtime requested: "
        f"{str(report['observability_runtime_requested']).lower()}"
    )
    print(
        "observability provider boundary requested: "
        f"{str(report['observability_provider_boundary_requested']).lower()}"
    )
    print(
        "runtime panic hook requested: "
        f"{str(report['runtime_panic_hook_requested']).lower()}"
    )
    print(
        "dashboard runtime requested: "
        f"{str(report['dashboard_runtime_requested']).lower()}"
    )
    print(
        "communications runtime requested: "
        f"{str(report['communications_runtime_requested']).lower()}"
    )
    print(
        "communications delivery provider requested: "
        f"{str(report['communications_delivery_provider_requested']).lower()}"
    )
    if report["runtime_smoke"] is not None:
        smoke = report["runtime_smoke"]
        trace = smoke.get("opportunity_trace_recovery") if isinstance(smoke, dict) else None
        iterations = smoke.get("iterations", 1)
        restart_recovery_report = smoke.get("runtime_restart_recovery_report", {})
        iteration_reports = smoke.get("runtime_smoke_iteration_reports", [])
        concurrent_lifecycle = smoke.get("concurrent_lifecycle", {})
        print(f"runtime smoke iterations: {iterations}")
        print(f"runtime smoke passed: {str(smoke['runtime_smoke_passed']).lower()}")
        print(f"runtime smoke workspace: {smoke['workspace']}")
        print(
            f"restart-plan-checkpoint-recovered: {smoke.get('restart_plan_checkpoint_recovered', '')}"
        )
        print(
            f"restart-adapter-checkpoint-recovered: {smoke.get('restart_adapter_checkpoint_recovered', '')}"
        )
        print(
            f"restart-graceful-shutdown-checkpoint-recovered: {smoke.get('restart_graceful_shutdown_checkpoint_recovered', '')}"
        )
        print(
            "restart-opportunity-trace-recovered-summaries: "
            f"{smoke.get('restart_opportunity_trace_recovered_summary_count', '')}"
        )
        print(
            "restart-opportunity-trace-recovered-summaries-match-count: "
            f"{str(smoke.get('restart_opportunity_trace_recovered_summaries_match_count')).lower()}"
        )
        if isinstance(concurrent_lifecycle, dict):
            print(
                "concurrent-lifecycle-validated: "
                f"{concurrent_lifecycle.get('validated', '')}"
            )
            print(
                "concurrent-lifecycle-workers: "
                f"{concurrent_lifecycle.get('workers', '')}"
            )
            print(
                "concurrent-lifecycle-audit-records-replayed: "
                f"{concurrent_lifecycle.get('audit_records_replayed', '')}"
            )
            print(
                "concurrent-lifecycle-sqlite-integrity-check-passed: "
                f"{concurrent_lifecycle.get('sqlite_integrity_check_passed', '')}"
            )
            print(
                "concurrent-lifecycle-external-submission-performed: "
                f"{concurrent_lifecycle.get('external_submission_performed', '')}"
            )
            print(
                "concurrent-lifecycle-live-execution-performed: "
                f"{concurrent_lifecycle.get('live_execution_performed', '')}"
            )
        load_profile = smoke.get("runtime_load_profile_review", {})
        if isinstance(load_profile, dict):
            print(
                "runtime-load-profile-review: "
                f"{load_profile.get('status', '')}"
            )
            print(
                "runtime-load-profile-latency-budget-met: "
                f"{load_profile.get('latency_budget_met', '')}"
            )
            print(
                "runtime-load-profile-resource-budget-met: "
                f"{load_profile.get('resource_budget_met', '')}"
            )
            print(
                "runtime-load-profile-replay-recovery-evidence-validated: "
                f"{load_profile.get('replay_recovery_evidence_validated', '')}"
            )
            print(
                "runtime-load-profile-remaining-external-evidence-count: "
                f"{load_profile.get('remaining_external_evidence_count', '')}"
            )
        print(f"production-ready: {smoke['production_ready']}")
        print(f"service-manager-action-performed: {smoke['service_manager_action_performed']}")
        print(f"external-submission-performed: {smoke['external_submission_performed']}")
        print(f"live-execution-performed: {smoke['live_execution_performed']}")
        if isinstance(trace, dict):
            print(f"opportunity-trace-corpus: {trace.get('corpus', '')}")
            print(f"opportunity-trace-discovered: {trace.get('discovered', '')}")
            print(f"opportunity-trace-recovered-checkpoints: {trace.get('recovered_checkpoints', '')}")
            print(f"opportunity-trace-missing-checkpoints: {trace.get('missing_checkpoints', '')}")
            print(f"opportunity-trace-audit-records-replayed: {trace.get('audit_records_replayed', '')}")
            print(f"opportunity-trace-recovery-validated: {str(trace.get('validated')).lower()}")
        if isinstance(restart_recovery_report, dict):
            print("runtime restart recovery report:")
            print(
                "restart-plan-checkpoint-recovered: "
                f"{restart_recovery_report.get('plan_checkpoint_recovered', '')}"
            )
            print(
                "restart-adapter-checkpoint-recovered: "
                f"{restart_recovery_report.get('adapter_checkpoint_recovered', '')}"
            )
            print(
                "restart-graceful-shutdown-checkpoint-recovered: "
                f"{restart_recovery_report.get('graceful_shutdown_checkpoint_recovered', '')}"
            )
            trace_report = restart_recovery_report.get("opportunity_trace_recovery")
            recovered_summaries = restart_recovery_report.get(
                "opportunity_trace_recovered_summaries",
                [],
            )
            if isinstance(trace_report, dict):
                print("runtime restart opportunity trace recovery:")
                print(
                    "opportunity-trace-corpus: "
                    f"{trace_report.get('corpus', '')}"
                )
                print(
                    "opportunity-trace-discovered: "
                    f"{trace_report.get('discovered', '')}"
                )
                print(
                    "opportunity-trace-recovered-checkpoints: "
                    f"{trace_report.get('recovered_checkpoints', '')}"
                )
                print(
                    "opportunity-trace-missing-checkpoints: "
                    f"{trace_report.get('missing_checkpoints', '')}"
                )
                print(
                    "opportunity-trace-audit-records-replayed: "
                    f"{trace_report.get('audit_records_replayed', '')}"
                )
                print(
                    "opportunity-trace-recovery-validated: "
                    f"{str(trace_report.get('validated')).lower()}"
                )
                if isinstance(recovered_summaries, list):
                    print(
                        "opportunity-trace-recovered-summaries: "
                        f"{len(recovered_summaries)}"
                    )
                    print(
                        "opportunity-trace-recovered-summaries-match-count: "
                        f"{str(restart_recovery_report.get('opportunity_trace_recovered_summaries_match_count')).lower()}"
                    )
                    for summary in recovered_summaries:
                        if not isinstance(summary, dict):
                            continue
                        print(
                            "- recovered trace summary: "
                            f"trace_id={summary.get('trace_id', '')}; "
                            f"planner_request_id={summary.get('planner_request_id', '')}; "
                            f"audit_sequence={summary.get('audit_sequence', '')}; "
                            f"route_kind={summary.get('route_kind', '')}; "
                            f"leg_count={summary.get('leg_count', '')}"
                        )
        if isinstance(iteration_reports, list) and iteration_reports:
            print("runtime smoke iteration reports:")
            for iteration_report in iteration_reports:
                if not isinstance(iteration_report, dict):
                    continue
                iteration_id = iteration_report.get("runtime-smoke-iteration", "unknown")
                if iterations == 1 and iteration_id == "unknown":
                    print(f"- run: single")
                else:
                    status = iteration_report.get(
                        "runtime-smoke",
                        iteration_report.get("runtime-smoke-passed", ""),
                    )
                    recovery = iteration_report.get("runtime_restart_recovery_report", {})
                    trace_recovery = (
                        recovery.get("opportunity_trace_recovery", {})
                        if isinstance(recovery, dict)
                        else {}
                    )
                    trace_summaries = (
                        recovery.get("opportunity_trace_recovered_summaries", [])
                        if isinstance(recovery, dict)
                        else []
                    )
                    print(
                        f"- {iteration_id}: runtime-smoke={status}; "
                        "restart-plan-checkpoint-recovered="
                        f"{iteration_report.get('restart-plan-checkpoint-recovered', '')}; "
                        "restart-adapter-checkpoint-recovered="
                        f"{iteration_report.get('restart-adapter-checkpoint-recovered', '')}; "
                        "opportunity-trace-recovered-checkpoints="
                        f"{trace_recovery.get('recovered_checkpoints', '')}; "
                        "opportunity-trace-recovered-summaries="
                        f"{len(trace_summaries) if isinstance(trace_summaries, list) else ''}; "
                        "opportunity-trace-recovery-validated="
                        f"{str(trace_recovery.get('validated')).lower()}"
                    )
    if report["audit_retention_execution"] is not None:
        retention = report["audit_retention_execution"]
        print(
            "audit retention execution passed: "
            f"{str(retention['audit_retention_execution_passed']).lower()}"
        )
        print(f"audit retention workspace: {retention['workspace']}")
        print(
            "audit-retention-rotate-active-requested: "
            f"{retention.get('rotate_active_requested', '')}"
        )
        print(
            "audit-retention-new-active-created: "
            f"{retention.get('new_active_created', '')}"
        )
        print(
            "audit-retention-expired-archives-deleted: "
            f"{retention.get('expired_archives_deleted', '')}"
        )
        print(
            "audit-retention-out-of-workspace-path-touched: "
            f"{retention.get('out_of_workspace_path_touched', '')}"
        )
        print(
            "audit-retention-live-network-used: "
            f"{retention.get('live_network_used', '')}"
        )
        print(
            "audit-retention-external-execution-performed: "
            f"{retention.get('external_execution_performed', '')}"
        )
        print(f"audit-retention-production-ready: {retention.get('production_ready', '')}")
    if report["audit_durability"] is not None:
        audit = report["audit_durability"]
        print(f"audit durability passed: {str(audit['audit_durability_passed']).lower()}")
        print(f"audit durability workspace: {audit['workspace']}")
        print(
            "audit-durability-append-replay-validated: "
            f"{audit.get('append_replay_validated', '')}"
        )
        print(
            "audit-durability-truncated-replay-rejected: "
            f"{audit.get('truncated_replay_rejected', '')}"
        )
        print(
            "audit-durability-tamper-replay-rejected: "
            f"{audit.get('tamper_replay_rejected', '')}"
        )
        print(
            "audit-durability-concurrent-append-validated: "
            f"{audit.get('concurrent_append_validated', '')}"
        )
        print(
            "audit-durability-filesystem-failure-validated: "
            f"{audit.get('filesystem_failure_validated', '')}"
        )
        print(
            "audit-durability-disk-full-failure-validated: "
            f"{audit.get('disk_full_failure_validated', '')}"
        )
        print(f"audit-durability-live-network-used: {audit.get('live_network_used', '')}")
        print(
            "audit-durability-external-execution-performed: "
            f"{audit.get('external_execution_performed', '')}"
        )
        print(f"audit-durability-production-ready: {audit.get('production_ready', '')}")
    if report["retention_preflight"] is not None:
        retention = report["retention_preflight"]
        active = retention["active_journal"]
        archive = retention["archive_directory"]
        print(
            "deployment-retention-preflight-passed: "
            f"{str(retention['retention_preflight_passed']).lower()}"
        )
        print(
            "deployment-retention-active-usable: "
            f"{str(active.get('usable_for_runtime_artifacts', False)).lower()}"
        )
        print(f"deployment-retention-active-reason: {active.get('reason', '')}")
        print(
            "deployment-retention-archive-usable: "
            f"{str(archive.get('usable_for_runtime_artifacts', False)).lower()}"
        )
        print(f"deployment-retention-archive-reason: {archive.get('reason', '')}")
        print(
            "deployment-retention-rotation-performed: "
            f"{str(retention['rotation_performed']).lower()}"
        )
        print(
            "deployment-retention-deletion-performed: "
            f"{str(retention['deletion_performed']).lower()}"
        )
        print(
            "deployment-retention-production-paths-touched: "
            f"{str(retention['production_paths_touched']).lower()}"
        )
    if report["graceful_shutdown"] is not None:
        shutdown = report["graceful_shutdown"]
        print(
            "graceful shutdown passed: "
            f"{str(shutdown['graceful_shutdown_passed']).lower()}"
        )
        print(f"graceful shutdown workspace: {shutdown['workspace']}")
        print(
            "runtime-graceful-shutdown-id: "
            f"{shutdown.get('shutdown_id', '')}"
        )
        print(
            "runtime-graceful-shutdown-audit-records-replayed: "
            f"{shutdown.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-graceful-shutdown-checkpoint-recovered: "
            f"{shutdown.get('checkpoint_recovered', '')}"
        )
        print(
            "runtime-graceful-shutdown-checkpoint-matches-record: "
            f"{shutdown.get('checkpoint_matches_record', '')}"
        )
        print(
            "runtime-graceful-shutdown-sqlite-integrity-check-passed: "
            f"{shutdown.get('sqlite_integrity_check_passed', '')}"
        )
        print(
            "runtime-graceful-shutdown-service-manager-action-performed: "
            f"{shutdown.get('service_manager_action_performed', '')}"
        )
        print(
            "runtime-graceful-shutdown-external-submission-performed: "
            f"{shutdown.get('external_submission_performed', '')}"
        )
        print(
            "runtime-graceful-shutdown-live-execution-performed: "
            f"{shutdown.get('live_execution_performed', '')}"
        )
        print(
            "runtime-graceful-shutdown-production-ready: "
            f"{shutdown.get('production_ready', '')}"
        )
    if report["backup_restore"] is not None:
        backup = report["backup_restore"]
        print(
            "backup restore passed: "
            f"{str(backup['backup_restore_passed']).lower()}"
        )
        print(f"backup restore workspace: {backup['workspace']}")
        print(
            "runtime-backup-restore-lifecycle-id: "
            f"{backup.get('lifecycle_id', '')}"
        )
        print(
            "runtime-backup-restore-audit-records-replayed: "
            f"{backup.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-backup-restore-audit-restore-check-passed: "
            f"{backup.get('audit_restore_check_passed', '')}"
        )
        print(
            "runtime-backup-restore-sqlite-restore-check-passed: "
            f"{backup.get('sqlite_restore_check_passed', '')}"
        )
        print(
            "runtime-backup-restore-plan-checkpoint-restored: "
            f"{backup.get('plan_checkpoint_restored', '')}"
        )
        print(
            "runtime-backup-restore-adapter-checkpoint-restored: "
            f"{backup.get('adapter_checkpoint_restored', '')}"
        )
        print(
            "runtime-backup-restore-adapter-recovery-plan-checkpoint-restored: "
            f"{backup.get('adapter_recovery_plan_checkpoint_restored', '')}"
        )
        print(
            "runtime-backup-restore-external-submission-performed: "
            f"{backup.get('external_submission_performed', '')}"
        )
        print(
            "runtime-backup-restore-live-execution-performed: "
            f"{backup.get('live_execution_performed', '')}"
        )
        print(
            "runtime-backup-restore-production-ready: "
            f"{backup.get('production_ready', '')}"
        )
    if report["backup_restore_load"] is not None:
        load = report["backup_restore_load"]
        print(
            "backup restore load passed: "
            f"{str(load['backup_restore_load_passed']).lower()}"
        )
        print(f"backup restore load workspace: {load['workspace']}")
        print(f"runtime-backup-restore-load-workers: {load.get('workers', '')}")
        print(
            "runtime-backup-restore-load-audit-records-replayed: "
            f"{load.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-backup-restore-load-sqlite-restore-check-passed: "
            f"{load.get('sqlite_restore_check_passed', '')}"
        )
        print(
            "runtime-backup-restore-load-plan-checkpoint-restored: "
            f"{load.get('plan_checkpoint_restored', '')}"
        )
        print(
            "runtime-backup-restore-load-adapter-checkpoint-restored: "
            f"{load.get('adapter_checkpoint_restored', '')}"
        )
        print(
            "runtime-backup-restore-load-adapter-recovery-plan-checkpoint-restored: "
            f"{load.get('adapter_recovery_plan_checkpoint_restored', '')}"
        )
        print(
            "runtime-backup-restore-load-backup-journal-sequence-matches: "
            f"{load.get('backup_journal_sequence_matches', '')}"
        )
        print(
            "runtime-backup-restore-load-external-submission-performed: "
            f"{load.get('external_submission_performed', '')}"
        )
        print(
            "runtime-backup-restore-load-live-execution-performed: "
            f"{load.get('live_execution_performed', '')}"
        )
        print(
            "runtime-backup-restore-load-production-ready: "
            f"{load.get('production_ready', '')}"
        )
    if report["restart_recovery"] is not None:
        restart = report["restart_recovery"]
        print(
            "restart recovery passed: "
            f"{str(restart['restart_recovery_passed']).lower()}"
        )
        print(f"restart recovery workspace: {restart['workspace']}")
        print(
            "runtime-restart-recovery-lifecycle-id: "
            f"{restart.get('lifecycle_id', '')}"
        )
        print(
            "runtime-restart-recovery-audit-records-replayed: "
            f"{restart.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-restart-recovery-sqlite-reopen-check-passed: "
            f"{restart.get('sqlite_reopen_check_passed', '')}"
        )
        print(
            "runtime-restart-recovery-plan-checkpoint-recovered: "
            f"{restart.get('plan_checkpoint_recovered', '')}"
        )
        print(
            "runtime-restart-recovery-adapter-checkpoint-recovered: "
            f"{restart.get('adapter_checkpoint_recovered', '')}"
        )
        print(
            "runtime-restart-recovery-adapter-recovery-plan-checkpoint-recovered: "
            f"{restart.get('adapter_recovery_plan_checkpoint_recovered', '')}"
        )
        print(
            "runtime-restart-recovery-graceful-shutdown-checkpoint-recovered: "
            f"{restart.get('graceful_shutdown_checkpoint_recovered', '')}"
        )
        print(
            "runtime-restart-recovery-opportunity-trace-recovered-checkpoints: "
            f"{restart.get('opportunity_trace_recovered_checkpoints', '')}"
        )
        print(
            "runtime-restart-recovery-external-submission-performed: "
            f"{restart.get('external_submission_performed', '')}"
        )
        print(
            "runtime-restart-recovery-live-execution-performed: "
            f"{restart.get('live_execution_performed', '')}"
        )
        print(
            "runtime-restart-recovery-production-ready: "
            f"{restart.get('production_ready', '')}"
        )
    if report["incomplete_recovery"] is not None:
        incomplete = report["incomplete_recovery"]
        print(
            "incomplete recovery passed: "
            f"{str(incomplete['incomplete_recovery_passed']).lower()}"
        )
        print(f"incomplete recovery workspace: {incomplete['workspace']}")
        print(
            "runtime-incomplete-recovery-expected-failure: "
            f"{incomplete.get('expected_failure', '')}"
        )
        print(
            "runtime-incomplete-recovery-audit-records-before-validation: "
            f"{incomplete.get('audit_records_before_validation', '')}"
        )
        print(
            "runtime-incomplete-recovery-reopened-audit-records: "
            f"{incomplete.get('reopened_audit_records', '')}"
        )
        print(
            "runtime-incomplete-recovery-plan-checkpoint-recovered: "
            f"{incomplete.get('plan_checkpoint_recovered', '')}"
        )
        print(
            "runtime-incomplete-recovery-adapter-checkpoint-recovered: "
            f"{incomplete.get('adapter_checkpoint_recovered', '')}"
        )
        print(
            "runtime-incomplete-recovery-adapter-recovery-plan-checkpoint-recovered: "
            f"{incomplete.get('adapter_recovery_plan_checkpoint_recovered', '')}"
        )
        print(
            "runtime-incomplete-recovery-service-manager-action-performed: "
            f"{incomplete.get('service_manager_action_performed', '')}"
        )
        print(
            "runtime-incomplete-recovery-external-submission-performed: "
            f"{incomplete.get('external_submission_performed', '')}"
        )
        print(
            "runtime-incomplete-recovery-live-execution-performed: "
            f"{incomplete.get('live_execution_performed', '')}"
        )
        print(
            "runtime-incomplete-recovery-production-ready: "
            f"{incomplete.get('production_ready', '')}"
        )
    if report["supervised_restart"] is not None:
        supervised = report["supervised_restart"]
        print(
            "supervised restart passed: "
            f"{str(supervised['supervised_restart_passed']).lower()}"
        )
        print(f"supervised restart workspace: {supervised['workspace']}")
        print(
            "runtime-supervised-restart-child-exit-code: "
            f"{supervised.get('child_exit_code', '')}"
        )
        print(
            "runtime-supervised-restart-child-stdout-lines: "
            f"{supervised.get('child_stdout_lines', '')}"
        )
        print(
            "runtime-supervised-restart-audit-records-replayed: "
            f"{supervised.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-supervised-restart-sqlite-reopen-check-passed: "
            f"{supervised.get('sqlite_reopen_check_passed', '')}"
        )
        print(
            "runtime-supervised-restart-plan-checkpoint-recovered: "
            f"{supervised.get('plan_checkpoint_recovered', '')}"
        )
        print(
            "runtime-supervised-restart-adapter-checkpoint-recovered: "
            f"{supervised.get('adapter_checkpoint_recovered', '')}"
        )
        print(
            "runtime-supervised-restart-adapter-recovery-plan-checkpoint-recovered: "
            f"{supervised.get('adapter_recovery_plan_checkpoint_recovered', '')}"
        )
        print(
            "runtime-supervised-restart-graceful-shutdown-checkpoint-recovered: "
            f"{supervised.get('graceful_shutdown_checkpoint_recovered', '')}"
        )
        print(
            "runtime-supervised-restart-opportunity-trace-recovered-checkpoints: "
            f"{supervised.get('opportunity_trace_recovered_checkpoints', '')}"
        )
        print(
            "runtime-supervised-restart-external-submission-performed: "
            f"{supervised.get('external_submission_performed', '')}"
        )
        print(
            "runtime-supervised-restart-live-execution-performed: "
            f"{supervised.get('live_execution_performed', '')}"
        )
        print(
            "runtime-supervised-restart-production-ready: "
            f"{supervised.get('production_ready', '')}"
        )
    if report["permission_denial"] is not None:
        permission = report["permission_denial"]
        print(
            "permission denial passed: "
            f"{str(permission['permission_denial_passed']).lower()}"
        )
        print(f"permission denial workspace: {permission['workspace']}")
        print(
            "runtime-permission-denial-expected-failure: "
            f"{permission.get('expected_failure', '')}"
        )
        print(
            "runtime-permission-denial-state-put-attempts: "
            f"{permission.get('state_put_attempts', '')}"
        )
        print(
            "runtime-permission-denial-audit-records-replayed: "
            f"{permission.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-permission-denial-adapter-evaluated: "
            f"{permission.get('adapter_evaluated', '')}"
        )
        print(
            "runtime-permission-denial-service-manager-action-performed: "
            f"{permission.get('service_manager_action_performed', '')}"
        )
        print(
            "runtime-permission-denial-external-submission-performed: "
            f"{permission.get('external_submission_performed', '')}"
        )
        print(
            "runtime-permission-denial-live-execution-performed: "
            f"{permission.get('live_execution_performed', '')}"
        )
        print(
            "runtime-permission-denial-production-ready: "
            f"{permission.get('production_ready', '')}"
        )
    if report["blocked_state_preflight"] is not None:
        blocked = report["blocked_state_preflight"]
        print(
            "blocked state preflight passed: "
            f"{str(blocked['blocked_state_preflight_passed']).lower()}"
        )
        print(f"blocked state workspace: {blocked['workspace']}")
        print(f"runtime-blocked-state-expected-failure: {blocked.get('expected_failure', '')}")
        print(f"runtime-blocked-state-artifacts-created: {blocked.get('artifacts_created', '')}")
        print(f"runtime-blocked-state-audit-created: {blocked.get('audit_created', '')}")
        print(
            "runtime-blocked-state-backup-audit-created: "
            f"{blocked.get('backup_audit_created', '')}"
        )
        print(
            "runtime-blocked-state-backup-state-created: "
            f"{blocked.get('backup_state_created', '')}"
        )
        print(
            "runtime-blocked-state-audit-workspace-created: "
            f"{blocked.get('audit_workspace_created', '')}"
        )
        print(
            "runtime-blocked-state-service-manager-action-performed: "
            f"{blocked.get('service_manager_action_performed', '')}"
        )
        print(
            "runtime-blocked-state-external-submission-performed: "
            f"{blocked.get('external_submission_performed', '')}"
        )
        print(
            "runtime-blocked-state-live-execution-performed: "
            f"{blocked.get('live_execution_performed', '')}"
        )
        print(f"runtime-blocked-state-production-ready: {blocked.get('production_ready', '')}")
    if report["blocked_audit_preflight"] is not None:
        blocked = report["blocked_audit_preflight"]
        print(
            "blocked audit preflight passed: "
            f"{str(blocked['blocked_audit_preflight_passed']).lower()}"
        )
        print(f"blocked audit workspace: {blocked['workspace']}")
        print(f"runtime-blocked-audit-expected-failure: {blocked.get('expected_failure', '')}")
        print(f"runtime-blocked-audit-artifacts-created: {blocked.get('artifacts_created', '')}")
        print(
            "runtime-blocked-audit-placeholder-created: "
            f"{blocked.get('placeholder_created', '')}"
        )
        print(f"runtime-blocked-audit-state-created: {blocked.get('state_created', '')}")
        print(
            "runtime-blocked-audit-backup-audit-created: "
            f"{blocked.get('backup_audit_created', '')}"
        )
        print(
            "runtime-blocked-audit-backup-state-created: "
            f"{blocked.get('backup_state_created', '')}"
        )
        print(
            "runtime-blocked-audit-audit-workspace-created: "
            f"{blocked.get('audit_workspace_created', '')}"
        )
        print(
            "runtime-blocked-audit-service-manager-action-performed: "
            f"{blocked.get('service_manager_action_performed', '')}"
        )
        print(
            "runtime-blocked-audit-external-submission-performed: "
            f"{blocked.get('external_submission_performed', '')}"
        )
        print(
            "runtime-blocked-audit-live-execution-performed: "
            f"{blocked.get('live_execution_performed', '')}"
        )
        print(f"runtime-blocked-audit-production-ready: {blocked.get('production_ready', '')}")
    if report["filesystem_preflight"] is not None:
        filesystem = report["filesystem_preflight"]
        print(
            "filesystem preflight passed: "
            f"{str(filesystem['filesystem_preflight_passed']).lower()}"
        )
        print(
            "filesystem preflight mutated filesystem: "
            f"{str(filesystem['filesystem_mutated']).lower()}"
        )
        print(
            "filesystem audit/state paths distinct: "
            f"{str(filesystem['audit_state_paths_distinct']).lower()}"
        )
        for path_report in (filesystem.get("audit"), filesystem.get("state")):
            if not isinstance(path_report, dict):
                continue
            label = path_report.get("label", "unknown")
            print(f"filesystem {label} path provided: {str(path_report.get('provided')).lower()}")
            print(
                f"filesystem {label} usable: "
                f"{str(path_report.get('usable_for_runtime_artifacts')).lower()}"
            )
            print(f"filesystem {label} reason: {path_report.get('reason', '')}")
            print(
                f"filesystem {label} parent writable: "
                f"{str(path_report.get('parent_writable')).lower()}"
            )
            print(
                f"filesystem {label} secret-like path: "
                f"{str(path_report.get('secret_like_path')).lower()}"
            )
        print(
            "filesystem-preflight-service-manager-action-performed: "
            f"{filesystem.get('service_manager_action_performed', '')}"
        )
        print(
            "filesystem-preflight-external-submission-performed: "
            f"{filesystem.get('external_submission_performed', '')}"
        )
        print(
            "filesystem-preflight-live-execution-performed: "
            f"{filesystem.get('live_execution_performed', '')}"
        )
        print(f"filesystem-preflight-production-ready: {filesystem.get('production_ready', '')}")
    if report["observability_runtime"] is not None:
        observability = report["observability_runtime"]
        print(
            "observability runtime passed: "
            f"{str(observability['observability_runtime_passed']).lower()}"
        )
        print(f"observability runtime workspace: {observability['workspace']}")
        print(
            "observability-runtime-audit-records-replayed: "
            f"{observability.get('audit_records_replayed', '')}"
        )
        print(
            "observability-runtime-checkpoints-recovered: "
            f"{observability.get('checkpoints_recovered', '')}"
        )
        print(
            "observability-runtime-metric-lines: "
            f"{observability.get('metric_lines', '')}"
        )
        print(
            "observability-runtime-scrape-metric-lines: "
            f"{observability.get('scrape_metric_lines', '')}"
        )
        print(
            "observability-runtime-served-metric-lines: "
            f"{observability.get('served_metric_lines', '')}"
        )
        print(
            "observability-runtime-loopback-bind-validated: "
            f"{observability.get('loopback_bind_validated', '')}"
        )
        print(
            "observability-runtime-listener-opened-and-closed: "
            f"{observability.get('listener_opened_and_closed', '')}"
        )
        print(
            "local-metrics-endpoint-started: "
            f"{observability.get('local_metrics_endpoint_started', '')}"
        )
        print(f"metrics-endpoint-started: {observability.get('metrics_endpoint_started', '')}")
        print(f"network-request-served: {observability.get('network_request_served', '')}")
        print(f"public-network-exposed: {observability.get('public_network_exposed', '')}")
        print(f"telemetry-exported: {observability.get('telemetry_exported', '')}")
        print(f"outbound-alerts-sent: {observability.get('outbound_alerts_sent', '')}")
        print(
            "external-submission-performed: "
            f"{observability.get('external_submission_performed', '')}"
        )
        print(f"live-execution-performed: {observability.get('live_execution_performed', '')}")
        print(f"production-ready: {observability.get('production_ready', '')}")
    if report["observability_metrics_runtime"] is not None:
        metrics_runtime = report["observability_metrics_runtime"]
        print(
            "observability metrics runtime passed: "
            f"{str(metrics_runtime['observability_metrics_runtime_passed']).lower()}"
        )
        print(f"observability metrics runtime workspace: {metrics_runtime['workspace']}")
        print(
            "observability-metrics-runtime-audit-records-replayed: "
            f"{metrics_runtime.get('audit_records_replayed', '')}"
        )
        print(
            "observability-metrics-runtime-checkpoint-recovered: "
            f"{metrics_runtime.get('checkpoint_recovered', '')}"
        )
        print(
            "observability-metrics-runtime-loopback-bind-validated: "
            f"{metrics_runtime.get('loopback_bind_validated', '')}"
        )
        print(
            "observability-metrics-runtime-expected-scrapes: "
            f"{metrics_runtime.get('expected_scrapes', '')}"
        )
        print(
            "observability-metrics-runtime-served-scrapes: "
            f"{metrics_runtime.get('served_scrapes', '')}"
        )
        print(
            "observability-metrics-runtime-all-scrapes-returned-ok: "
            f"{metrics_runtime.get('all_scrapes_returned_ok', '')}"
        )
        print(
            "observability-metrics-runtime-response-lines-consistent: "
            f"{metrics_runtime.get('response_lines_consistent', '')}"
        )
        print(
            "observability-metrics-runtime-response-metric-lines: "
            f"{metrics_runtime.get('response_metric_lines', '')}"
        )
        print(
            "observability-metrics-runtime-started: "
            f"{metrics_runtime.get('local_metrics_runtime_started', '')}"
        )
        print(
            "observability-metrics-runtime-shutdown: "
            f"{metrics_runtime.get('local_metrics_runtime_shutdown', '')}"
        )
        print(
            "observability-metrics-runtime-public-network-exposed: "
            f"{metrics_runtime.get('public_network_exposed', '')}"
        )
        print(
            "observability-metrics-runtime-telemetry-exported: "
            f"{metrics_runtime.get('telemetry_exported', '')}"
        )
        print(
            "observability-metrics-runtime-outbound-alerts-sent: "
            f"{metrics_runtime.get('outbound_alerts_sent', '')}"
        )
        print(
            "external-submission-performed: "
            f"{metrics_runtime.get('external_submission_performed', '')}"
        )
        print(
            "live-execution-performed: "
            f"{metrics_runtime.get('live_execution_performed', '')}"
        )
        print(f"production-ready: {metrics_runtime.get('production_ready', '')}")
    if report["observability_provider_boundary"] is not None:
        provider_boundary = report["observability_provider_boundary"]
        print(
            "observability provider boundary passed: "
            f"{str(provider_boundary['observability_provider_boundary_passed']).lower()}"
        )
        print(f"observability provider boundary workspace: {provider_boundary['workspace']}")
        print(
            "observability-provider-boundary-audit-records-replayed: "
            f"{provider_boundary.get('audit_records_replayed', '')}"
        )
        print(
            "observability-provider-boundary-checkpoint-recovered: "
            f"{provider_boundary.get('checkpoint_recovered', '')}"
        )
        print(
            "observability-provider-boundary-status: "
            f"{provider_boundary.get('status', '')}"
        )
        print(
            "observability-provider-boundary-operations-review-ready: "
            f"{provider_boundary.get('operations_review_ready', '')}"
        )
        print(
            "observability-provider-boundary-export-dry-run-ready: "
            f"{provider_boundary.get('export_dry_run_ready', '')}"
        )
        print(
            "observability-provider-boundary-alert-route-dispatch-ready: "
            f"{provider_boundary.get('alert_route_dispatch_ready', '')}"
        )
        print(
            "observability-provider-boundary-endpoint-preflight-ready: "
            f"{provider_boundary.get('endpoint_preflight_ready', '')}"
        )
        print(
            "observability-provider-boundary-metrics-runtime-ready: "
            f"{provider_boundary.get('metrics_runtime_ready', '')}"
        )
        print(
            "observability-provider-boundary-missing-local-controls: "
            f"{provider_boundary.get('missing_local_controls', '')}"
        )
        print(
            "observability-provider-boundary-remaining-provider-evidence-count: "
            f"{provider_boundary.get('remaining_provider_evidence_count', '')}"
        )
        print(
            "observability-provider-boundary-provider-validation-performed: "
            f"{provider_boundary.get('provider_validation_performed', '')}"
        )
        print(f"public-network-exposed: {provider_boundary.get('public_network_exposed', '')}")
        print(f"telemetry-exported: {provider_boundary.get('telemetry_exported', '')}")
        print(f"outbound-alerts-sent: {provider_boundary.get('outbound_alerts_sent', '')}")
        print(
            "external-submission-performed: "
            f"{provider_boundary.get('external_submission_performed', '')}"
        )
        print(
            "service-manager-action-performed: "
            f"{provider_boundary.get('service_manager_action_performed', '')}"
        )
        print(
            "sensitive-material-loaded: "
            f"{provider_boundary.get('sensitive_material_loaded', '')}"
        )
        print(
            "live-execution-performed: "
            f"{provider_boundary.get('live_execution_performed', '')}"
        )
        print(f"production-ready: {provider_boundary.get('production_ready', '')}")
    if report["runtime_panic_hook"] is not None:
        panic_hook = report["runtime_panic_hook"]
        print(
            "runtime panic hook passed: "
            f"{str(panic_hook['runtime_panic_hook_passed']).lower()}"
        )
        print(f"runtime panic hook workspace: {panic_hook['workspace']}")
        print(f"runtime-panic-hook-installed: {panic_hook.get('hook_installed', '')}")
        print(f"runtime-panic-hook-restored: {panic_hook.get('hook_restored', '')}")
        print(f"runtime-panic-hook-panic-observed: {panic_hook.get('panic_observed', '')}")
        print(f"runtime-panic-hook-panic-captured: {panic_hook.get('panic_captured', '')}")
        print(
            "runtime-panic-hook-audit-records-replayed: "
            f"{panic_hook.get('audit_records_replayed', '')}"
        )
        print(
            "runtime-panic-hook-failure-checkpoint-recovered: "
            f"{panic_hook.get('failure_checkpoint_recovered', '')}"
        )
        print(
            "runtime-panic-hook-failure-checkpoint-contains-sentinel: "
            f"{panic_hook.get('failure_checkpoint_contains_sentinel', '')}"
        )
        print(
            "runtime-panic-hook-metrics-endpoint-started: "
            f"{panic_hook.get('metrics_endpoint_started', '')}"
        )
        print(
            "runtime-panic-hook-public-network-exposed: "
            f"{panic_hook.get('public_network_exposed', '')}"
        )
        print(
            "runtime-panic-hook-outbound-alerts-sent: "
            f"{panic_hook.get('outbound_alerts_sent', '')}"
        )
        print(
            "runtime-panic-hook-external-submission-performed: "
            f"{panic_hook.get('external_submission_performed', '')}"
        )
        print(
            "runtime-panic-hook-live-execution-performed: "
            f"{panic_hook.get('live_execution_performed', '')}"
        )
        print(
            "runtime-panic-hook-production-ready: "
            f"{panic_hook.get('production_ready', '')}"
        )
    if report["dashboard_runtime"] is not None:
        dashboard = report["dashboard_runtime"]
        print(
            "dashboard runtime passed: "
            f"{str(dashboard['dashboard_runtime_passed']).lower()}"
        )
        print(f"dashboard runtime workspace: {dashboard['workspace']}")
        print(
            "dashboard-runtime-audit-records-replayed: "
            f"{dashboard.get('audit_records_replayed', '')}"
        )
        print(
            "dashboard-runtime-checkpoints-recovered: "
            f"{dashboard.get('checkpoints_recovered', '')}"
        )
        print(
            "dashboard-render-access-authorized: "
            f"{dashboard.get('render_access_authorized', '')}"
        )
        print(f"dashboard-render-panel-count: {dashboard.get('render_panel_count', '')}")
        print(
            "dashboard-hosted-security-ready: "
            f"{dashboard.get('hosted_security_ready', '')}"
        )
        print(
            "dashboard-hosted-request-preflight-ready: "
            f"{dashboard.get('hosted_request_preflight_ready', '')}"
        )
        print(
            "dashboard-hosted-request-validation-ready: "
            f"{dashboard.get('hosted_request_validation_ready', '')}"
        )
        print(
            "dashboard-hosted-runtime-readiness-review-ready: "
            f"{dashboard.get('hosted_runtime_readiness_review_ready', '')}"
        )
        print(
            "dashboard-hosted-runtime-security-review-ready: "
            f"{dashboard.get('hosted_runtime_security_review_ready', '')}"
        )
        print(
            "dashboard-hosted-runtime-preflight-ready: "
            f"{dashboard.get('hosted_runtime_preflight_ready', '')}"
        )
        print(
            "dashboard-hosted-runtime-session-ready: "
            f"{dashboard.get('hosted_runtime_session_ready', '')}"
        )
        print(
            "dashboard-hosted-runtime-accepted-request-validated: "
            f"{dashboard.get('hosted_runtime_accepted_request_validated', '')}"
        )
        print(
            "dashboard-hosted-runtime-unauthenticated-rejection-validated: "
            f"{dashboard.get('hosted_runtime_unauthenticated_rejection_validated', '')}"
        )
        print(
            "dashboard-hosted-runtime-csrf-rejection-validated: "
            f"{dashboard.get('hosted_runtime_csrf_rejection_validated', '')}"
        )
        print(
            "dashboard-hosted-runtime-rate-limit-rejection-validated: "
            f"{dashboard.get('hosted_runtime_rate_limit_rejection_validated', '')}"
        )
        print(
            "dashboard-hosted-runtime-loopback-serving-validated: "
            f"{dashboard.get('hosted_runtime_loopback_serving_validated', '')}"
        )
        print(
            "dashboard-hosted-runtime-secure-headers-validated: "
            f"{dashboard.get('hosted_runtime_secure_headers_validated', '')}"
        )
        print(
            "dashboard-hosted-runtime-remaining-external-evidence-count: "
            f"{dashboard.get('hosted_runtime_remaining_external_evidence_count', '')}"
        )
        print(
            "local-dashboard-server-started: "
            f"{dashboard.get('local_dashboard_server_started', '')}"
        )
        print(
            "persistent-dashboard-server-started: "
            f"{dashboard.get('persistent_dashboard_server_started', '')}"
        )
        print(f"network-request-served: {dashboard.get('network_request_served', '')}")
        print(f"local-http-status-code: {dashboard.get('local_http_status_code', '')}")
        print(f"public-network-exposed: {dashboard.get('public_network_exposed', '')}")
        print(f"live-controls-enabled: {dashboard.get('live_controls_enabled', '')}")
        print(
            "external-submission-performed: "
            f"{dashboard.get('external_submission_performed', '')}"
        )
        print(f"live-execution-performed: {dashboard.get('live_execution_performed', '')}")
        print(f"production-ready: {dashboard.get('production_ready', '')}")
    if report["communications_runtime"] is not None:
        communications = report["communications_runtime"]
        print(
            "communications runtime passed: "
            f"{str(communications['communications_runtime_passed']).lower()}"
        )
        print(f"communications runtime workspace: {communications['workspace']}")
        print(
            "communications-runtime-audit-records-replayed: "
            f"{communications.get('audit_records_replayed', '')}"
        )
        print(
            "communications-runtime-checkpoints-recovered: "
            f"{communications.get('checkpoints_recovered', '')}"
        )
        print(
            "command-route-accepted: "
            f"{communications.get('command_route_accepted', '')}"
        )
        print(
            "command-operator-authorized: "
            f"{communications.get('command_operator_authorized', '')}"
        )
        print(
            "remote-command-security-ready: "
            f"{communications.get('remote_command_security_ready', '')}"
        )
        print(
            "notification-dispatch-status: "
            f"{communications.get('notification_dispatch_status', '')}"
        )
        print(
            "notification-channel-count: "
            f"{communications.get('notification_channel_count', '')}"
        )
        print(f"outbound-network-used: {communications.get('outbound_network_used', '')}")
        print(f"remote-commands-enabled: {communications.get('remote_commands_enabled', '')}")
        print(
            "external-submission-performed: "
            f"{communications.get('external_submission_performed', '')}"
        )
        print(
            "live-execution-performed: "
            f"{communications.get('live_execution_performed', '')}"
        )
        print(
            "signing-or-broadcast-performed: "
            f"{communications.get('signing_or_broadcast_performed', '')}"
        )
        print(f"production-ready: {communications.get('production_ready', '')}")
    if report["communications_delivery_provider"] is not None:
        delivery = report["communications_delivery_provider"]
        print(
            "communications delivery provider passed: "
            f"{str(delivery['communications_delivery_provider_passed']).lower()}"
        )
        print(f"communications delivery provider workspace: {delivery['workspace']}")
        print(
            "communications-delivery-provider-boundary-status: "
            f"{delivery.get('status', '')}"
        )
        print(
            "communications-delivery-provider-channel-session-ready: "
            f"{delivery.get('channel_session_ready', '')}"
        )
        print(
            "communications-delivery-provider-platform-adapter-ready: "
            f"{delivery.get('platform_adapter_ready', '')}"
        )
        print(
            "communications-delivery-provider-delivery-evidence-available: "
            f"{delivery.get('delivery_evidence_available', '')}"
        )
        print(
            "communications-delivery-provider-rate-limit-evidence-available: "
            f"{delivery.get('rate_limit_evidence_available', '')}"
        )
        print(
            "communications-delivery-provider-outage-evidence-available: "
            f"{delivery.get('outage_evidence_available', '')}"
        )
        print(
            "communications-delivery-provider-platform-identity-evidence-available: "
            f"{delivery.get('platform_identity_evidence_available', '')}"
        )
        print(
            "communications-delivery-provider-remaining-external-evidence-count: "
            f"{delivery.get('remaining_external_evidence_count', '')}"
        )
        print(
            "communications-delivery-provider-blocker-count: "
            f"{delivery.get('blocker_count', '')}"
        )
        print(
            "communications-delivery-provider-audit-records-replayed: "
            f"{delivery.get('audit_records_replayed', '')}"
        )
        print(
            "communications-delivery-provider-checkpoints-recovered: "
            f"{delivery.get('checkpoints_recovered', '')}"
        )
        print(f"outbound-network-used: {delivery.get('outbound_network_used', '')}")
        print(f"message-delivered: {delivery.get('message_delivered', '')}")
        print(f"provider-call-performed: {delivery.get('provider_call_performed', '')}")
        print(
            "token-secret-material-loaded: "
            f"{delivery.get('token_secret_material_loaded', '')}"
        )
        print(f"live-execution-performed: {delivery.get('live_execution_performed', '')}")
        print(
            "signing-or-broadcast-performed: "
            f"{delivery.get('signing_or_broadcast_performed', '')}"
        )
        print(f"production-ready: {delivery.get('production_ready', '')}")
    print(f"service actions performed: {str(report['service_actions_performed']).lower()}")
    print(f"secrets loaded: {str(report['secrets_loaded']).lower()}")
    print(f"external calls performed: {str(report['external_calls_performed']).lower()}")
    print(f"production readiness claimed: {str(report['production_readiness_claimed']).lower()}")
    print("remaining external evidence:")
    for item in report["remaining_external_evidence"]:
        print(f"- {item}")


def main() -> int:
    args = parse_args()
    try:
        report = build_report(args)
    except subprocess.TimeoutExpired as error:
        return fail(f"validation helper timed out after {error.timeout} seconds")
    except (OSError, RuntimeError, ValueError) as error:
        return fail(str(error))

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text_report(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
