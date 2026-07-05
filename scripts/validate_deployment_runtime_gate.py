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
    "runtime_config_reload": "runtime_config_reload_requested",
    "deployment_static_hardening": "deployment_static_hardening_requested",
    "sqlite_schema_migration": "sqlite_schema_migration_requested",
    "deployment_config_redaction": "deployment_config_redaction_requested",
    "deployment_log_redaction": "deployment_log_redaction_requested",
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
    "observability_metrics_runtime": "observability_metrics_runtime_requested",
    "observability_provider_boundary": "observability_provider_boundary_requested",
    "observability_provider_submission": "observability_provider_submission_requested",
    "runtime_panic_hook": "runtime_panic_hook_requested",
    "dashboard_runtime": "dashboard_runtime_requested",
    "communications_runtime": "communications_runtime_requested",
    "communications_delivery_provider": "communications_delivery_provider_requested",
    "communications_provider_submission": "communications_provider_submission_requested",
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
        "name": "service-manager-lifecycle-rehearsal",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-service-manager-lifecycle-rehearsal",
        ],
        "expected": {
            "service-manager-lifecycle-rehearsal": "validation passed",
            "ready-rehearsal-status": "validated",
            "ready-ordered-lifecycle-validated": "true",
            "ready-operator-controlled-events": "true",
            "ready-non-secret-references-present": "true",
            "ready-graceful-shutdown-checkpoint-reference-present": "true",
            "ready-restart-recovery-reference-present": "true",
            "ready-concurrent-lifecycle-reference-present": "true",
            "ready-concurrent-lifecycle-worker-count": "3",
            "ready-concurrent-lifecycle-success": "true",
            "ready-operator-approved": "true",
            "ready-reviewer-approved": "true",
            "blocked-rehearsal-status": "blocked",
            "service-manager-action-performed-by-validator": "false",
            "deployment-path-mutated-by-validator": "false",
            "secrets-loaded": "false",
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
        "name": "deployment-backup-restore-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-backup-restore-transcript",
        ],
        "expected": {
            "deployment-backup-restore-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "ready-deployment-host-evidence": "true",
            "ready-service-lifecycle-reference-present": "true",
            "ready-backup-artifact-reference-present": "true",
            "ready-restore-execution-reference-present": "true",
            "ready-deployment-load-reference-present": "true",
            "ready-audit-restore-validated": "true",
            "ready-sqlite-restore-validated": "true",
            "ready-runtime-checkpoint-restore-validated": "true",
            "ready-post-restore-runtime-smoke-passed": "true",
            "blocked-transcript-status": "blocked",
            "backup-restore-executed-by-validator": "false",
            "service-manager-action-performed-by-validator": "false",
            "deployment-path-mutated-by-validator": "false",
            "secrets-loaded": "false",
            "external-submission-performed": "false",
            "live-execution-performed": "false",
            "production-ready": "false",
        },
    },
    {
        "name": "deployment-graceful-shutdown-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-graceful-shutdown-transcript",
        ],
        "expected": {
            "deployment-graceful-shutdown-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "ready-deployment-host-evidence": "true",
            "ready-service-lifecycle-reference-present": "true",
            "ready-shutdown-request-reference-present": "true",
            "ready-service-stopped-reference-present": "true",
            "ready-graceful-shutdown-checkpoint-reference-present": "true",
            "ready-audit-shutdown-validated": "true",
            "ready-sqlite-shutdown-validated": "true",
            "ready-restart-recovery-after-shutdown-validated": "true",
            "ready-post-shutdown-runtime-smoke-passed": "true",
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
        "name": "deployment-sqlite-schema-migration-transcript",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-sqlite-schema-migration-transcript",
        ],
        "expected": {
            "deployment-sqlite-schema-migration-transcript": "validation passed",
            "ready-transcript-status": "ready-for-external-review",
            "ready-deployment-host-evidence": "true",
            "ready-service-lifecycle-reference-present": "true",
            "ready-schema-version-transition-validated": "true",
            "ready-sqlite-recovery-validated": "true",
            "ready-audit-replay-after-migration-validated": "true",
            "ready-rollback-reference-present": "true",
            "blocked-transcript-status": "blocked",
            "migration-executed-by-validator": "false",
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
    {
        "name": "deployment-response-drill-rehearsal",
        "command": [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-response-drill-rehearsal",
        ],
        "expected": {
            "deployment-response-drill-rehearsal": "validation passed",
            "ready-rehearsal-status": "validated",
            "ready-rollback-ready": "true",
            "ready-incident-response-ready": "true",
            "ready-failure-capture-ready": "true",
            "ready-plan-ids-match": "true",
            "ready-component-operator-approvals-present": "true",
            "ready-component-reviewer-approvals-present": "true",
            "ready-operator-approved": "true",
            "ready-reviewer-approved": "true",
            "blocked-rehearsal-status": "blocked",
            "rollback-executed-by-validator": "false",
            "incident-response-executed-by-validator": "false",
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
    "backup_restore_executed_by_validator",
    "external_calls_performed",
    "external_execution_performed",
    "external_log_shipping_performed",
    "external_secret_revoked",
    "external_submission_performed",
    "keystore_entry_written",
    "live_execution_enabled",
    "live_execution_performed",
    "live_network_used",
    "network_listeners_started",
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
        "--run-runtime-config-reload",
        "--runtime-config-reload-workspace",
        str(workspace / "runtime-config-reload"),
        "--run-deployment-static-hardening",
        "--run-sqlite-schema-migration",
        "--sqlite-schema-migration-workspace",
        str(workspace / "sqlite-schema-migration"),
        "--run-deployment-config-redaction",
        "--deployment-config-redaction-workspace",
        str(workspace / "deployment-config-redaction"),
        "--run-deployment-log-redaction",
        "--deployment-log-redaction-workspace",
        str(workspace / "deployment-log-redaction"),
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
        "--run-observability-metrics-runtime",
        "--observability-metrics-workspace",
        str(workspace / "observability-metrics-runtime"),
        "--run-observability-provider-boundary",
        "--observability-provider-boundary-workspace",
        str(workspace / "observability-provider-boundary"),
        "--run-observability-provider-submission-preflight",
        "--observability-provider-submission-workspace",
        str(workspace / "observability-provider-submission"),
        "--run-runtime-panic-hook",
        "--runtime-panic-hook-workspace",
        str(workspace / "runtime-panic-hook"),
        "--run-dashboard-runtime",
        "--dashboard-workspace",
        str(workspace / "dashboard-runtime"),
        "--run-communications-runtime",
        "--communications-workspace",
        str(workspace / "communications-runtime"),
        "--run-communications-delivery-provider-boundary",
        "--communications-delivery-provider-workspace",
        str(workspace / "communications-delivery-provider"),
        "--run-communications-provider-submission-preflight",
        "--communications-provider-submission-workspace",
        str(workspace / "communications-provider-submission"),
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

    metrics_runtime = report.get("observability_metrics_runtime")
    if not isinstance(metrics_runtime, dict):
        errors.append("observability_metrics_runtime report missing or invalid")
    else:
        if metrics_runtime.get("observability_metrics_runtime_passed") is not True:
            errors.append("observability metrics runtime was not marked as passed")
        for key in (
            "checkpoint_recovered",
            "loopback_bind_validated",
            "all_scrapes_returned_ok",
            "response_lines_consistent",
            "local_metrics_runtime_started",
            "local_metrics_runtime_shutdown",
        ):
            if metrics_runtime.get(key) != "true":
                errors.append(f"observability metrics runtime {key} was not true")
        for key in (
            "public_network_exposed",
            "telemetry_exported",
            "outbound_alerts_sent",
            "external_submission_performed",
            "live_execution_performed",
            "production_ready",
        ):
            if metrics_runtime.get(key) != "false":
                errors.append(f"observability metrics runtime {key} was not false")
        for key, expected in (
            ("audit_records_replayed", 1),
            ("expected_scrapes", 3),
            ("served_scrapes", 3),
        ):
            try:
                if int(metrics_runtime.get(key, "0")) != expected:
                    errors.append(
                        f"observability metrics runtime {key} was not {expected}"
                    )
            except ValueError:
                errors.append(f"observability metrics runtime {key} was not an integer")
        try:
            if int(metrics_runtime.get("response_metric_lines", "0")) <= 0:
                errors.append(
                    "observability metrics runtime response_metric_lines was not positive"
                )
        except ValueError:
            errors.append(
                "observability metrics runtime response_metric_lines was not an integer"
            )

    provider_boundary = report.get("observability_provider_boundary")
    if not isinstance(provider_boundary, dict):
        errors.append("observability_provider_boundary report missing or invalid")
    else:
        if provider_boundary.get("observability_provider_boundary_passed") is not True:
            errors.append("observability provider boundary was not marked as passed")
        for key in (
            "checkpoint_recovered",
            "operations_review_ready",
            "export_dry_run_ready",
            "alert_route_dispatch_ready",
            "endpoint_preflight_ready",
            "metrics_runtime_ready",
        ):
            if provider_boundary.get(key) != "true":
                errors.append(f"observability provider boundary {key} was not true")
        for key in (
            "provider_validation_performed",
            "public_network_exposed",
            "telemetry_exported",
            "outbound_alerts_sent",
            "external_submission_performed",
            "service_manager_action_performed",
            "sensitive_material_loaded",
            "live_execution_performed",
            "production_ready",
        ):
            if provider_boundary.get(key) != "false":
                errors.append(f"observability provider boundary {key} was not false")
        if provider_boundary.get("status") != "BlockedPendingProviderValidation":
            errors.append(
                "observability provider boundary status was not BlockedPendingProviderValidation"
            )
        for key, expected in (
            ("audit_records_replayed", 1),
            ("missing_local_controls", 0),
            ("remaining_provider_evidence_count", 5),
        ):
            try:
                if int(provider_boundary.get(key, "0")) != expected:
                    errors.append(
                        f"observability provider boundary {key} was not {expected}"
                    )
            except ValueError:
                errors.append(f"observability provider boundary {key} was not an integer")

    observability_submission = report.get("observability_provider_submission")
    if not isinstance(observability_submission, dict):
        errors.append("observability_provider_submission report missing or invalid")
    else:
        if observability_submission.get("observability_provider_submission_passed") is not True:
            errors.append("observability provider submission was not marked as passed")
        for key in (
            "provider_boundary_ready",
            "telemetry_kill_switch_armed",
            "audit_state_preflight_required",
            "export_idempotency_required",
            "exporter_backpressure_required",
            "alert_delivery_authorization_required",
            "telemetry_redaction_required",
            "checkpoint_recovered",
        ):
            if observability_submission.get(key) != "true":
                errors.append(f"observability provider submission {key} was not true")
        for key in (
            "provider_validation_evidence_available",
            "telemetry_export_requested",
            "outbound_alert_delivery_requested",
            "external_submission_requested",
            "public_network_exposure_requested",
            "service_manager_action_requested",
            "sensitive_material_loaded",
            "live_execution_requested",
            "production_ready",
        ):
            if observability_submission.get(key) != "false":
                errors.append(f"observability provider submission {key} was not false")
        if observability_submission.get("status") != "blocked-pending-provider-validation":
            errors.append(
                "observability provider submission status was not blocked-pending-provider-validation"
            )
        for key, expected in (
            ("blocker_count", 1),
            ("audit_records_replayed", 1),
        ):
            try:
                if int(observability_submission.get(key, "0")) != expected:
                    errors.append(
                        f"observability provider submission {key} was not {expected}"
                    )
            except ValueError:
                errors.append(f"observability provider submission {key} was not an integer")

    delivery_provider = report.get("communications_delivery_provider")
    if not isinstance(delivery_provider, dict):
        errors.append("communications_delivery_provider report missing or invalid")
    else:
        if delivery_provider.get("communications_delivery_provider_passed") is not True:
            errors.append("communications delivery provider was not marked as passed")
        for key in (
            "channel_session_ready",
            "platform_adapter_ready",
        ):
            if delivery_provider.get(key) != "true":
                errors.append(f"communications delivery provider {key} was not true")
        for key in (
            "delivery_evidence_available",
            "rate_limit_evidence_available",
            "outage_evidence_available",
            "platform_identity_evidence_available",
            "outbound_network_used",
            "message_delivered",
            "provider_call_performed",
            "token_secret_material_loaded",
            "live_execution_performed",
            "signing_or_broadcast_performed",
            "production_ready",
        ):
            if delivery_provider.get(key) != "false":
                errors.append(f"communications delivery provider {key} was not false")
        if delivery_provider.get("status") != "blocked-pending-provider-delivery-validation":
            errors.append(
                "communications delivery provider status was not blocked-pending-provider-delivery-validation"
            )
        for key, expected in (
            ("remaining_external_evidence_count", 4),
            ("blocker_count", 4),
            ("audit_records_replayed", 8),
            ("checkpoints_recovered", 8),
        ):
            try:
                if int(delivery_provider.get(key, "0")) != expected:
                    errors.append(
                        f"communications delivery provider {key} was not {expected}"
                    )
            except ValueError:
                errors.append(f"communications delivery provider {key} was not an integer")

    provider_submission = report.get("communications_provider_submission")
    if not isinstance(provider_submission, dict):
        errors.append("communications_provider_submission report missing or invalid")
    else:
        if provider_submission.get("communications_provider_submission_passed") is not True:
            errors.append("communications provider submission was not marked as passed")
        for key in (
            "delivery_provider_boundary_ready",
            "delivery_kill_switch_armed",
            "audit_state_preflight_required",
            "delivery_idempotency_required",
            "rate_limit_controls_required",
            "outage_backoff_controls_required",
            "payload_redaction_required",
        ):
            if provider_submission.get(key) != "true":
                errors.append(f"communications provider submission {key} was not true")
        for key in (
            "provider_validation_evidence_available",
            "outbound_delivery_requested",
            "outbound_network_used",
            "message_delivered",
            "provider_call_performed",
            "token_secret_material_loaded",
            "live_execution_performed",
            "signing_or_broadcast_performed",
            "production_ready",
        ):
            if provider_submission.get(key) != "false":
                errors.append(f"communications provider submission {key} was not false")
        if provider_submission.get("status") != "blocked-pending-provider-validation":
            errors.append(
                "communications provider submission status was not blocked-pending-provider-validation"
            )
        for key, expected in (
            ("blocker_count", 1),
            ("violation_count", 0),
            ("audit_records_replayed", 8),
            ("checkpoints_recovered", 8),
        ):
            try:
                if int(provider_submission.get(key, "0")) != expected:
                    errors.append(
                        f"communications provider submission {key} was not {expected}"
                    )
            except ValueError:
                errors.append(f"communications provider submission {key} was not an integer")

    runtime_smoke = report.get("runtime_smoke")
    if not isinstance(runtime_smoke, dict):
        errors.append("runtime_smoke report missing or invalid")
        return errors

    load_profile = runtime_smoke.get("runtime_load_profile_review")
    if not isinstance(load_profile, dict):
        errors.append("runtime_smoke.runtime_load_profile_review missing or invalid")
        return errors

    if load_profile.get("status") != "ReadyForLocalReview":
        errors.append("runtime load profile review was not ReadyForLocalReview")
    for key in (
        "latency_budget_met",
        "resource_budget_met",
        "replay_recovery_evidence_validated",
    ):
        if load_profile.get(key) != "true":
            errors.append(f"runtime load profile review {key} was not true")
    try:
        if int(load_profile.get("remaining_external_evidence_count", "0")) <= 0:
            errors.append(
                "runtime load profile remaining_external_evidence_count was not positive"
            )
    except ValueError:
        errors.append(
            "runtime load profile remaining_external_evidence_count was not an integer"
        )

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
    for key in (
        "service_manager_evidence_available",
        "disk_full_evidence_available",
        "retention_execution_evidence_available",
        "backup_restore_evidence_available",
        "graceful_shutdown_evidence_available",
        "audit_sqlite_recovery_evidence_available",
        "sqlite_schema_migration_evidence_available",
        "daemon_failure_capture_evidence_available",
        "concurrent_lifecycle_evidence_available",
        "production_ready",
    ):
        if production_preflight.get(key) != "false":
            errors.append(f"runtime production preflight {key} was not false")

    config_reload = report.get("runtime_config_reload")
    if not isinstance(config_reload, dict):
        errors.append("runtime_config_reload report missing or invalid")
        return errors
    expected_reload_values = {
        "status": "ready-for-local-review",
        "initial_mode_safe": "true",
        "reloaded_mode_safe": "true",
        "reload_change_detected": "true",
        "cex_allowlist_changed": "true",
        "asset_allowlist_changed": "true",
        "service_manager_action_performed": "false",
        "secret_material_loaded": "false",
        "external_submission_performed": "false",
        "live_execution_performed": "false",
        "production_ready": "false",
    }
    for key, expected in expected_reload_values.items():
        if config_reload.get(key) != expected:
            errors.append(
                f"runtime config reload expected {key}={expected!r} but saw {config_reload.get(key)!r}"
            )
    if config_reload.get("runtime_config_reload_passed") is not True:
        errors.append("runtime config reload did not pass")

    deployment_static_hardening = report.get("deployment_static_hardening")
    if not isinstance(deployment_static_hardening, dict):
        errors.append("deployment_static_hardening report missing or invalid")
        return errors
    expected_static_hardening_values = {
        "schema": "arbyclaw.deployment_static_hardening.v1",
        "passed": True,
        "config_smoke_requested": True,
        "config_observe_or_paper_mode": True,
        "config_live_execution_disabled": True,
        "config_secret_like_assignment": False,
        "config_smoke_passed": True,
        "config_smoke_config_loaded": True,
        "config_smoke_observe_or_paper_mode": True,
        "config_smoke_live_execution_disabled": True,
        "config_smoke_secret_like_output": False,
        "service_actions_performed": False,
        "network_listeners_started": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
    }
    for key, expected in expected_static_hardening_values.items():
        if deployment_static_hardening.get(key) != expected:
            errors.append(
                f"deployment static hardening expected {key}={expected!r} "
                f"but saw {deployment_static_hardening.get(key)!r}"
            )
    if deployment_static_hardening.get("deployment_static_hardening_passed") is not True:
        errors.append("deployment static hardening did not pass")

    sqlite_schema_migration = report.get("sqlite_schema_migration")
    if not isinstance(sqlite_schema_migration, dict):
        errors.append("sqlite_schema_migration report missing or invalid")
        return errors
    expected_sqlite_schema_values = {
        "status": "ready-for-local-review",
        "legacy_pre_schema_version": "0",
        "migrated_schema_version": "1",
        "expected_schema_version": "1",
        "legacy_checkpoint_preserved": "true",
        "future_version_rejected": "true",
        "migration_performed": "true",
        "service_manager_action_performed": "false",
        "external_network_used": "false",
        "secret_material_recorded": "false",
        "live_execution_performed": "false",
        "production_ready": "false",
    }
    for key, expected in expected_sqlite_schema_values.items():
        if sqlite_schema_migration.get(key) != expected:
            errors.append(
                f"sqlite schema migration expected {key}={expected!r} "
                f"but saw {sqlite_schema_migration.get(key)!r}"
            )
    if sqlite_schema_migration.get("sqlite_schema_migration_passed") is not True:
        errors.append("sqlite schema migration did not pass")

    deployment_config_redaction = report.get("deployment_config_redaction")
    if not isinstance(deployment_config_redaction, dict):
        errors.append("deployment_config_redaction report missing or invalid")
        return errors
    expected_config_redaction_values = {
        "config_loaded": "true",
        "config_mode_safe": "true",
        "audit_redaction_required": "true",
        "unsafe_metadata_rejected": "true",
        "redacted_event_appended": "true",
        "audit_replay_validated": "true",
        "secret_material_recorded": "false",
        "external_network_used": "false",
        "service_manager_action_performed": "false",
        "live_execution_performed": "false",
        "production_ready": "false",
    }
    for key, expected in expected_config_redaction_values.items():
        if deployment_config_redaction.get(key) != expected:
            errors.append(
                f"deployment config redaction expected {key}={expected!r} "
                f"but saw {deployment_config_redaction.get(key)!r}"
            )
    if deployment_config_redaction.get("deployment_config_redaction_passed") is not True:
        errors.append("deployment config redaction did not pass")

    deployment_log_redaction = report.get("deployment_log_redaction")
    if not isinstance(deployment_log_redaction, dict):
        errors.append("deployment_log_redaction report missing or invalid")
        return errors
    expected_log_redaction_values = {
        "sanitized_log_written": "true",
        "log_redaction_applied": "true",
        "unsafe_log_material_absent": "true",
        "unsafe_metadata_rejected": "true",
        "redacted_event_appended": "true",
        "audit_replay_validated": "true",
        "secret_material_recorded": "false",
        "external_network_used": "false",
        "service_manager_action_performed": "false",
        "live_execution_performed": "false",
        "production_ready": "false",
    }
    for key, expected in expected_log_redaction_values.items():
        if deployment_log_redaction.get(key) != expected:
            errors.append(
                f"deployment log redaction expected {key}={expected!r} "
                f"but saw {deployment_log_redaction.get(key)!r}"
            )
    if deployment_log_redaction.get("deployment_log_redaction_passed") is not True:
        errors.append("deployment log redaction did not pass")

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
        "runtime_config_reload_enforced": True,
        "deployment_static_hardening_enforced": True,
        "sqlite_schema_migration_enforced": True,
        "deployment_config_redaction_enforced": True,
        "deployment_log_redaction_enforced": True,
        "runtime_load_profile_review_enforced": True,
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
        print("runtime-config-reload-enforced: true")
        print("deployment-static-hardening-enforced: true")
        print("sqlite-schema-migration-enforced: true")
        print("deployment-config-redaction-enforced: true")
        print("deployment-log-redaction-enforced: true")
        print("runtime-load-profile-review-enforced: true")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
