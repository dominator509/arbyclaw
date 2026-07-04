#!/usr/bin/env python3
"""Run the strongest local deployment-runtime validation bundle.

This gate composes existing local-only runtime/deployment probes and verifies
their combined report preserves the hard safety invariants. It does not install
or control services, call exchanges/RPCs, load secrets, submit adapters, expose
public endpoints, or claim production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import shutil
import subprocess
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
HOST_RUNTIME_SCRIPT = ROOT / "scripts/validate_deployment_host_runtime.py"
DEFAULT_WORKSPACE = ROOT / "target/ci-deployment-runtime-gate"
TIMEOUT_SECONDS = 1500

EXPECTED_COMPONENTS = {
    "runtime_smoke": "runtime_smoke_requested",
    "audit_durability": "audit_durability_requested",
    "audit_retention_execution": "audit_retention_execution_requested",
    "graceful_shutdown": "graceful_shutdown_requested",
    "backup_restore": "backup_restore_requested",
    "backup_restore_load": "backup_restore_load_requested",
    "restart_recovery": "restart_recovery_requested",
    "incomplete_recovery": "incomplete_recovery_requested",
    "supervised_restart": "supervised_restart_requested",
    "permission_denial": "permission_denial_requested",
    "blocked_state_preflight": "blocked_state_preflight_requested",
    "blocked_audit_preflight": "blocked_audit_preflight_requested",
    "filesystem_preflight": "filesystem_preflight_requested",
    "retention_preflight": "retention_preflight_requested",
    "observability_runtime": "observability_runtime_requested",
    "runtime_panic_hook": "runtime_panic_hook_requested",
    "dashboard_runtime": "dashboard_runtime_requested",
    "communications_runtime": "communications_runtime_requested",
}

TRANSCRIPT_COMPONENTS = [
    {
        "name": "service-manager-lifecycle-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-service-manager-lifecycle-transcript",
        ],
        "expected": {
            "service-manager-lifecycle-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "ready-operator-lifecycle-rehearsal-reference-present": "true",
            "ready-emergency-stop-review-reference-present": "true",
            "ready-rollback-plan-review-reference-present": "true",
            "ready-operator-review-window-current": "true",
            "ready-concurrent-lifecycle-reference-present": "true",
            "ready-concurrent-lifecycle-worker-count": "3",
            "ready-concurrent-lifecycle-success": "true",
            "blocked-transcript-status": "blocked",
            "service-manager-action-performed-by-validator": "false",
            "external-submission-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "deployment-disk-full-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-disk-full-transcript",
        ],
        "expected": {
            "deployment-disk-full-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "blocked-transcript-status": "blocked",
            "disk-filled-by-validator": "false",
            "production-path-mutated-by-validator": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "deployment-retention-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-retention-transcript",
        ],
        "expected": {
            "deployment-retention-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "blocked-transcript-status": "blocked",
            "rotation-performed-by-validator": "false",
            "production-path-mutated-by-validator": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "deployment-permission-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-permission-transcript",
        ],
        "expected": {
            "deployment-permission-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "ready-runtime-write-attempt-reference-present": "true",
            "ready-runtime-write-permission-denied": "true",
            "ready-runtime-write-error-classified": "true",
            "blocked-transcript-status": "blocked",
            "permission-changed-by-validator": "false",
            "production-path-mutated-by-validator": "false",
            "service-manager-action-performed-by-validator": "false",
            "external-submission-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "deployment-audit-sqlite-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-audit-sqlite-transcript",
        ],
        "expected": {
            "deployment-audit-sqlite-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "blocked-transcript-status": "blocked",
            "service-manager-action-performed-by-validator": "false",
            "deployment-path-mutated-by-validator": "false",
            "secrets-loaded": "false",
            "external-submission-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "rollback-execution-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-rollback-execution-transcript",
        ],
        "expected": {
            "rollback-execution-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "blocked-transcript-status": "blocked",
            "rollback-executed-by-validator": "false",
            "service-manager-action-performed-by-validator": "false",
            "files-mutated-by-validator": "false",
            "external-calls-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "incident-response-execution-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-incident-response-execution-transcript",
        ],
        "expected": {
            "incident-response-execution-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "blocked-transcript-status": "blocked",
            "incident-response-executed-by-validator": "false",
            "service-manager-action-performed-by-validator": "false",
            "files-mutated-by-validator": "false",
            "alerts-sent-by-validator": "false",
            "external-calls-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "deployment-failure-capture-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-failure-capture-transcript",
        ],
        "expected": {
            "deployment-failure-capture-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "blocked-transcript-status": "blocked",
            "panic-hook-installed-by-validator": "false",
            "tracing-subscriber-installed-by-validator": "false",
            "failure-injected-by-validator": "false",
            "service-manager-action-performed-by-validator": "false",
            "files-mutated-by-validator": "false",
            "alerts-sent-by-validator": "false",
            "external-calls-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
]

DANGEROUS_TRUE_KEYS = {
    "external_calls_performed",
    "external_execution_performed",
    "external_log_shipping_performed",
    "external_secret_revoked",
    "external_submission_performed",
    "keystore_entry_written",
    "live_execution_enabled",
    "live_execution_performed",
    "live_network_used",
    "out_of_workspace_path_touched",
    "outbound_alerts_sent",
    "outbound_network_delivery_attempted",
    "outbound_network_used",
    "plaintext_decrypted",
    "production_log_paths_touched",
    "production_paths_touched",
    "production_readiness_claimed",
    "production_ready",
    "public_network_exposed",
    "secret_material_loaded",
    "secrets_loaded",
    "service_actions_performed",
    "service_manager_action_performed",
    "telemetry_exported",
}
TRANSCRIPT_TIMEOUT_SECONDS = 900


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--workspace-base",
        type=pathlib.Path,
        default=DEFAULT_WORKSPACE,
        help="fresh local workspace base under target/ for aggregate probe files",
    )
    parser.add_argument(
        "--config",
        type=pathlib.Path,
        default=ROOT / "config.example.toml",
        help="non-secret config used by the runtime-smoke component",
    )
    parser.add_argument(
        "--agent-bin",
        type=pathlib.Path,
        help="optional arb-agent binary forwarded to the host runtime helper",
    )
    parser.add_argument("--json", action="store_true", help="emit JSON")
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"deployment runtime gate failed: {message}", file=sys.stderr)
    return 1


def clean_workspace(path: pathlib.Path) -> pathlib.Path:
    resolved = path.resolve()
    target = (ROOT / "target").resolve()
    try:
        resolved.relative_to(target)
    except ValueError as exc:
        raise ValueError("workspace base must resolve inside repository target/") from exc
    if resolved.exists():
        shutil.rmtree(resolved)
    resolved.mkdir(parents=True)
    return resolved


def write_retention_fixture(path: pathlib.Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        '{"schema":"arbyclaw.local-retention-fixture.v1","secret_material_loaded":false}\n',
        encoding="utf-8",
    )


def build_command(args: argparse.Namespace, workspace: pathlib.Path) -> list[str]:
    filesystem_dir = workspace / "filesystem-preflight"
    filesystem_dir.mkdir(parents=True, exist_ok=True)
    retention_dir = workspace / "retention-preflight"
    archive_dir = retention_dir / "archive"
    archive_dir.mkdir(parents=True, exist_ok=True)
    active_journal = retention_dir / "active.audit.jsonl"
    write_retention_fixture(active_journal)

    command = [
        sys.executable,
        str(HOST_RUNTIME_SCRIPT),
        "--run-runtime-smoke",
        "--runtime-smoke-iterations",
        "1",
        "--config",
        str(args.config),
        "--runtime-workspace",
        str(workspace / "runtime-smoke"),
        "--run-audit-durability",
        "--audit-durability-workspace",
        str(workspace / "audit-durability"),
        "--run-audit-retention-execution",
        "--retention-workspace",
        str(workspace / "audit-retention-execution"),
        "--run-graceful-shutdown",
        "--graceful-shutdown-workspace",
        str(workspace / "graceful-shutdown"),
        "--run-backup-restore",
        "--backup-restore-workspace",
        str(workspace / "backup-restore"),
        "--run-backup-restore-load",
        "--backup-restore-load-workspace",
        str(workspace / "backup-restore-load"),
        "--run-restart-recovery",
        "--restart-recovery-workspace",
        str(workspace / "restart-recovery"),
        "--run-incomplete-recovery",
        "--incomplete-recovery-workspace",
        str(workspace / "incomplete-recovery"),
        "--run-supervised-restart",
        "--supervised-restart-workspace",
        str(workspace / "supervised-restart"),
        "--run-permission-denial",
        "--permission-denial-workspace",
        str(workspace / "permission-denial"),
        "--run-blocked-state-preflight",
        "--blocked-state-workspace",
        str(workspace / "blocked-state"),
        "--run-blocked-audit-preflight",
        "--blocked-audit-workspace",
        str(workspace / "blocked-audit"),
        "--run-filesystem-preflight",
        "--filesystem-audit-path",
        str(filesystem_dir / "runtime-audit.jsonl"),
        "--filesystem-state-path",
        str(filesystem_dir / "runtime-state.sqlite3"),
        "--run-retention-preflight",
        "--retention-active-path",
        str(active_journal),
        "--retention-archive-dir",
        str(archive_dir),
        "--run-observability-runtime",
        "--observability-workspace",
        str(workspace / "observability-runtime"),
        "--run-runtime-panic-hook",
        "--runtime-panic-hook-workspace",
        str(workspace / "runtime-panic-hook"),
        "--run-dashboard-runtime",
        "--dashboard-workspace",
        str(workspace / "dashboard-runtime"),
        "--run-communications-runtime",
        "--communications-workspace",
        str(workspace / "communications-runtime"),
        "--json",
    ]
    if args.agent_bin is not None:
        command.extend(["--agent-bin", str(args.agent_bin)])
    return command


def walk_report(value: Any, path: str = "$") -> list[tuple[str, Any]]:
    rows = [(path, value)]
    if isinstance(value, dict):
        for key, child in value.items():
            rows.extend(walk_report(child, f"{path}.{key}"))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            rows.extend(walk_report(child, f"{path}[{index}]"))
    return rows


def validate_report(report: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if report.get("schema") != "arbyclaw.deployment_host_runtime_validation.v1":
        errors.append("unexpected nested deployment-host runtime schema")

    for component, requested_key in EXPECTED_COMPONENTS.items():
        if report.get(requested_key) is not True:
            errors.append(f"{requested_key} was not true")
        if report.get(component) is None:
            errors.append(f"{component} report missing")

    for path, value in walk_report(report):
        if not isinstance(value, bool):
            continue
        key = path.rsplit(".", 1)[-1]
        if key in DANGEROUS_TRUE_KEYS and value:
            errors.append(f"unsafe side-effect flag set true at {path}")

    runtime_smoke = report.get("runtime_smoke")
    if not isinstance(runtime_smoke, dict):
        errors.append("runtime_smoke report missing or invalid")
        return errors

    production_preflight = runtime_smoke.get("production_runtime_preflight")
    if not isinstance(production_preflight, dict):
        errors.append("runtime_smoke.production_runtime_preflight missing or invalid")
        return errors

    if production_preflight.get("validation_passed") is not True:
        errors.append("runtime production preflight was not marked as passed")
    if (
        production_preflight.get("status")
        != "BlockedPendingProductionHostValidation"
    ):
        errors.append(
            "runtime production preflight status was not BlockedPendingProductionHostValidation"
        )
    for key in ("local_smoke_validated", "local_smoke_load_validated"):
        if production_preflight.get(key) != "true":
            errors.append(f"runtime production preflight {key} was not true")
    unresolved_blockers = production_preflight.get("unresolved_blockers")
    try:
        if int(unresolved_blockers) <= 0:
            errors.append("runtime production preflight unresolved_blockers was not positive")
    except (TypeError, ValueError):
        errors.append("runtime production preflight unresolved_blockers was not an integer")
    for key in ("service_manager_evidence_available", "disk_full_evidence_available", "production_ready"):
        if production_preflight.get(key) != "false":
            errors.append(f"runtime production preflight {key} was not false")

    return errors


def parse_key_value_lines(stdout: str) -> dict[str, str]:
    pairs: dict[str, str] = {}
    for line in stdout.splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if key:
            pairs[key] = value
    return pairs


def run_transcript_component(component: dict[str, Any]) -> dict[str, Any]:
    completed = subprocess.run(
        component["command"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TRANSCRIPT_TIMEOUT_SECONDS,
        check=False,
    )
    fields = parse_key_value_lines(completed.stdout)
    summary = {
        "name": component["name"],
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "stdout_line_count": len(completed.stdout.splitlines()),
        "timeout_seconds": TRANSCRIPT_TIMEOUT_SECONDS,
        "fields": fields,
    }
    return summary


def validate_transcript_component(component: dict[str, Any], summary: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if not summary["passed"]:
        errors.append(f"{component['name']} exited {summary['returncode']}")
        return errors

    fields = summary["fields"]
    for key, expected in component["expected"].items():
        actual = fields.get(key)
        if actual != expected:
            errors.append(
                f"{component['name']} expected {key}={expected!r} but saw {actual!r}"
            )
    blocked_count = fields.get("blocked-blocker-count")
    if blocked_count is None:
        errors.append(f"{component['name']} missing blocked-blocker-count")
    else:
        try:
            if int(blocked_count) <= 0:
                errors.append(f"{component['name']} blocked-blocker-count was not positive")
        except ValueError:
            errors.append(f"{component['name']} blocked-blocker-count was not an integer")
    return errors


def main() -> int:
    args = parse_args()
    try:
        workspace = clean_workspace(args.workspace_base)
    except ValueError as exc:
        return fail(str(exc))

    command = build_command(args, workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    if completed.returncode != 0:
        sys.stderr.write(completed.stderr)
        sys.stderr.write(completed.stdout)
        return fail(f"deployment-host runtime helper exited {completed.returncode}")

    try:
        nested_report = json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        return fail(f"deployment-host runtime helper did not emit JSON: {exc}")

    errors = validate_report(nested_report)
    transcript_summaries = [
        run_transcript_component(component) for component in TRANSCRIPT_COMPONENTS
    ]
    for component, summary in zip(TRANSCRIPT_COMPONENTS, transcript_summaries, strict=True):
        errors.extend(validate_transcript_component(component, summary))
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return fail("aggregate deployment-runtime invariants failed")

    total_component_count = len(EXPECTED_COMPONENTS) + len(TRANSCRIPT_COMPONENTS)

    report = {
        "schema": "arbyclaw.deployment_runtime_aggregate_gate.v1",
        "workspace_base": str(workspace.relative_to(ROOT)),
        "component_count": total_component_count,
        "nested_runtime_component_count": len(EXPECTED_COMPONENTS),
        "transcript_component_count": len(TRANSCRIPT_COMPONENTS),
        "all_components_requested": True,
        "all_components_reported": True,
        "unsafe_side_effect_flags_detected": False,
        "service_actions_performed": False,
        "external_calls_performed": False,
        "live_execution_enabled": False,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
        "deployment_host_report_schema": nested_report["schema"],
        "runtime_smoke_production_preflight_enforced": True,
        "transcript_component_names": [
            component["name"] for component in TRANSCRIPT_COMPONENTS
        ],
        "transcript_components_passed": True,
        "remaining_external_evidence": nested_report.get("remaining_external_evidence", []),
    }

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("deployment runtime aggregate gate passed")
        print(f"component-count: {report['component_count']}")
        print(
            f"transcript-component-count: {report['transcript_component_count']}"
        )
        print("unsafe-side-effect-flags-detected: false")
        print("service-actions-performed: false")
        print("external-calls-performed: false")
        print("live-execution-enabled: false")
        print("secrets-loaded: false")
        print("production-readiness-claimed: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
