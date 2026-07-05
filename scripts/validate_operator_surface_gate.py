#!/usr/bin/env python3
"""Run the strongest current local operator-surface aggregate validation bundle.

This gate composes the local communications, dashboard, and observability CLI
validators, their deployment-host wrapper reports, and runtime-smoke operator
integration. It preserves local-only/non-secret behavior: no outbound network
delivery, no public exposure, no service-manager actions, no external
submission, no signing/broadcast, and no production-readiness claims.
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT_SECONDS = 2_400


def parse_output(text: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or ": " not in line:
            continue
        key, value = line.split(": ", 1)
        parsed[key.strip()] = value.strip()
    return parsed


def parse_positive_int(value: str | None) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def run_text_command(name: str, command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "parsed": parse_output(output),
    }


def run_json_command(name: str, command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    report: dict[str, Any] | None = None
    if completed.returncode == 0:
        for index, line in enumerate(output.splitlines()):
            if line.lstrip().startswith("{"):
                candidate = "\n".join(output.splitlines()[index:])
                loaded = json.loads(candidate)
                if not isinstance(loaded, dict):
                    raise RuntimeError(f"{name} did not emit a JSON object")
                report = loaded
                break
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "json_report": report,
    }


def validate_communications_cli(parsed: dict[str, str]) -> list[str]:
    errors: list[str] = []
    exact_true = (
        "command-route-accepted",
        "command-operator-authorized",
        "remote-command-security-ready",
        "platform-command-ingress-ready",
        "platform-command-token-reference-present",
        "platform-command-signature-verified",
        "platform-command-identity-authorized",
        "platform-command-channel-permission-granted",
        "channel-adapter-ready",
        "channel-adapter-delivery-kill-switch-required",
        "channel-adapter-audit-state-preflight-required",
        "channel-adapter-idempotency-required",
        "channel-adapter-rate-limit-controls-required",
        "channel-adapter-outage-backoff-required",
        "channel-adapter-payload-redaction-required",
        "channel-session-ready",
        "platform-adapter-ready",
        "platform-adapter-token-reference-present",
        "platform-adapter-identity-verified",
        "platform-adapter-identity-authorized",
        "platform-adapter-channel-permission-granted",
        "platform-adapter-command-injection-blocked",
        "platform-adapter-delivery-kill-switch-required",
        "platform-adapter-audit-state-preflight-required",
        "platform-adapter-idempotency-required",
        "platform-adapter-rate-limit-controls-required",
        "platform-adapter-outage-backoff-required",
        "platform-adapter-payload-redaction-required",
    )
    exact_false = (
        "platform-command-token-secret-material-present",
        "platform-command-replay-nonce-reused",
        "platform-command-injection-detected",
        "platform-command-provider-rate-limited",
        "platform-command-provider-outage-observed",
        "remote-command-injection-detected",
        "channel-adapter-message-delivered",
        "platform-adapter-token-secret-material-present",
        "platform-adapter-token-revoked",
        "platform-adapter-provider-rate-limited",
        "platform-adapter-provider-outage-observed",
        "outbound-network-used",
        "remote-commands-enabled",
        "external-submission-performed",
        "live-execution-performed",
        "signing-or-broadcast-performed",
        "production-ready",
    )
    for key in exact_true:
        if parsed.get(key) != "true":
            errors.append(f"communications cli expected {key}=true")
    for key in exact_false:
        if parsed.get(key) != "false":
            errors.append(f"communications cli expected {key}=false")
    for key in (
        "communications-runtime-audit-records-replayed",
        "communications-runtime-checkpoints-recovered",
        "channel-session-validations",
        "channel-session-accepted",
        "channel-session-rejected-unauthenticated",
        "channel-session-rejected-replay",
        "channel-session-rejected-provider-unavailable",
        "notification-channel-count",
    ):
        if (parse_positive_int(parsed.get(key)) or 0) <= 0:
            errors.append(f"communications cli expected positive {key}")
    if parsed.get("communications-runtime") != "validation passed":
        errors.append("communications cli did not report validation passed")
    if parsed.get("notification-dispatch-status") != "RecordedLocally":
        errors.append("communications cli expected RecordedLocally notification dispatch")
    return errors


def validate_communications_outbox_cli(parsed: dict[str, str]) -> list[str]:
    errors: list[str] = []
    exact_true = (
        "communications-outbox-ready-written",
        "communications-outbox-duplicate-rejected",
        "communications-outbox-rate-limit-blocked",
        "communications-outbox-outage-blocked",
        "communications-outbox-checkpoint-recovered",
        "communications-outbox-secret-material-absent",
    )
    exact_false = (
        "communications-outbox-outbound-network-used",
        "communications-outbox-delivery-performed",
        "external-submission-performed",
        "live-execution-performed",
        "signing-or-broadcast-performed",
        "production-ready",
    )
    for key in exact_true:
        if parsed.get(key) != "true":
            errors.append(f"communications outbox cli expected {key}=true")
    for key in exact_false:
        if parsed.get(key) != "false":
            errors.append(f"communications outbox cli expected {key}=false")
    if parsed.get("communications-outbox") != "validation passed":
        errors.append("communications outbox cli did not report validation passed")
    if parse_positive_int(parsed.get("communications-outbox-recorded-count")) != 1:
        errors.append("communications outbox cli expected exactly one recorded outbox line")
    if parse_positive_int(parsed.get("communications-outbox-audit-records-replayed")) != 3:
        errors.append("communications outbox cli expected three replayed audit records")
    return errors


def validate_dashboard_cli(parsed: dict[str, str]) -> list[str]:
    errors: list[str] = []
    for key in (
        "dashboard-render-access-authorized",
        "dashboard-hosted-security-ready",
        "dashboard-hosted-audit-state-preflight-required",
        "dashboard-hosted-session-revocation-required",
        "dashboard-hosted-operator-role-review-required",
        "dashboard-hosted-read-only-controls-required",
        "dashboard-hosted-request-preflight-ready",
        "dashboard-hosted-request-validation-ready",
        "dashboard-hosted-session-validation-ready",
        "dashboard-hosted-runtime-readiness-review-ready",
        "dashboard-hosted-runtime-security-review-ready",
        "dashboard-hosted-runtime-preflight-ready",
        "dashboard-hosted-runtime-session-ready",
        "dashboard-hosted-runtime-accepted-request-validated",
        "dashboard-hosted-runtime-unauthenticated-rejection-validated",
        "dashboard-hosted-runtime-csrf-rejection-validated",
        "dashboard-hosted-runtime-rate-limit-rejection-validated",
        "dashboard-hosted-runtime-loopback-serving-validated",
        "dashboard-hosted-runtime-secure-headers-validated",
        "local-dashboard-server-started",
        "network-request-served",
    ):
        if parsed.get(key) != "true":
            errors.append(f"dashboard cli expected {key}=true")
    for key in (
        "public-network-exposed",
        "persistent-dashboard-server-started",
        "live-controls-enabled",
        "external-submission-performed",
        "live-execution-performed",
        "production-ready",
    ):
        if parsed.get(key) != "false":
            errors.append(f"dashboard cli expected {key}=false")
    for key in (
        "dashboard-runtime-audit-records-replayed",
        "dashboard-runtime-checkpoints-recovered",
        "dashboard-render-panel-count",
        "dashboard-hosted-session-requests",
        "dashboard-hosted-session-accepted",
        "dashboard-hosted-session-rejected-unauthenticated",
        "dashboard-hosted-session-rejected-csrf",
        "dashboard-hosted-session-rejected-rate-limited",
        "dashboard-hosted-runtime-remaining-external-evidence-count",
    ):
        if (parse_positive_int(parsed.get(key)) or 0) <= 0:
            errors.append(f"dashboard cli expected positive {key}")
    if parsed.get("dashboard-runtime") != "validation passed":
        errors.append("dashboard cli did not report validation passed")
    if parsed.get("local-http-status-code") != "200":
        errors.append("dashboard cli expected local-http-status-code=200")
    return errors


def validate_dashboard_loopback_runtime_cli(parsed: dict[str, str]) -> list[str]:
    errors: list[str] = []
    for key in (
        "dashboard-loopback-runtime-checkpoint-recovered",
        "dashboard-loopback-runtime-loopback-bind-validated",
        "dashboard-loopback-runtime-all-requests-returned-ok",
        "dashboard-loopback-runtime-response-digest-consistent",
        "dashboard-loopback-runtime-bounded-runtime-started",
        "dashboard-loopback-runtime-bounded-runtime-shutdown",
    ):
        if parsed.get(key) != "true":
            errors.append(f"dashboard loopback runtime cli expected {key}=true")
    for key in (
        "dashboard-loopback-runtime-public-network-exposed",
        "dashboard-loopback-runtime-live-controls-enabled",
        "public-network-exposed",
        "persistent-dashboard-server-started",
        "live-controls-enabled",
        "external-submission-performed",
        "live-execution-performed",
        "production-ready",
    ):
        if parsed.get(key) != "false":
            errors.append(f"dashboard loopback runtime cli expected {key}=false")
    if parsed.get("dashboard-loopback-runtime") != "validation passed":
        errors.append("dashboard loopback runtime cli did not report validation passed")
    if parse_positive_int(parsed.get("dashboard-loopback-runtime-audit-records-replayed")) != 1:
        errors.append("dashboard loopback runtime cli expected one replayed audit record")
    if parse_positive_int(parsed.get("dashboard-loopback-runtime-expected-requests")) != 3:
        errors.append("dashboard loopback runtime cli expected three requests")
    if parse_positive_int(parsed.get("dashboard-loopback-runtime-served-requests")) != 3:
        errors.append("dashboard loopback runtime cli expected three served requests")
    return errors


def validate_observability_cli(parsed: dict[str, str]) -> list[str]:
    errors: list[str] = []
    for key in (
        "observability-log-retention-rotate-active-requested",
        "observability-log-retention-new-active-created",
        "observability-runtime-audit-state-preflight-required",
        "observability-runtime-exporter-kill-switch-required",
        "observability-runtime-alert-authorization-required",
        "observability-runtime-rate-limit-backpressure-required",
        "observability-runtime-retry-backoff-required",
        "observability-runtime-no-secret-telemetry-required",
        "observability-runtime-loopback-bind-validated",
        "observability-runtime-listener-opened-and-closed",
        "observability-runtime-tracing-subscriber-captured",
        "local-metrics-endpoint-started",
        "metrics-endpoint-started",
        "network-request-served",
    ):
        if parsed.get(key) != "true":
            errors.append(f"observability cli expected {key}=true")
    for key in (
        "observability-log-retention-production-paths-touched",
        "observability-log-retention-service-manager-action-performed",
        "observability-log-retention-external-log-shipping-performed",
        "observability-alert-route-outbound-network-used",
        "observability-runtime-tracing-global-subscriber-installed",
        "public-network-exposed",
        "telemetry-exported",
        "outbound-alerts-sent",
        "external-submission-performed",
        "live-execution-performed",
        "production-ready",
    ):
        if parsed.get(key) != "false":
            errors.append(f"observability cli expected {key}=false")
    for key in (
        "observability-runtime-audit-records-replayed",
        "observability-runtime-checkpoints-recovered",
        "observability-log-retention-deleted-file-count",
        "observability-runtime-metric-lines",
        "observability-alert-route-local-channels-recorded",
        "observability-runtime-scrape-metric-lines",
        "observability-runtime-served-metric-lines",
    ):
        if (parse_positive_int(parsed.get(key)) or 0) <= 0:
            errors.append(f"observability cli expected positive {key}")
    if parsed.get("observability-runtime") != "validation passed":
        errors.append("observability cli did not report validation passed")
    if parsed.get("observability-alert-route-dispatch-status") != "ReadyForLocalReview":
        errors.append("observability cli expected ReadyForLocalReview alert-route dispatch")
    return errors


def validate_wrapper_common(report: dict[str, Any], key: str) -> tuple[list[str], dict[str, Any]]:
    errors: list[str] = []
    nested = report.get(key)
    if not isinstance(nested, dict):
        return [f"wrapper missing nested report {key}"], {}
    if report.get("schema") != "arbyclaw.deployment_host_runtime_validation.v1":
        errors.append("wrapper emitted unexpected schema")
    for field in (
        "external_calls_performed",
        "live_execution_enabled",
        "production_readiness_claimed",
        "secrets_loaded",
        "service_actions_performed",
    ):
        if report.get(field) is not False:
            errors.append(f"wrapper expected top-level {field}=false")
    return errors, nested


def validate_communications_wrapper(report: dict[str, Any]) -> list[str]:
    errors, nested = validate_wrapper_common(report, "communications_runtime")
    if report.get("communications_runtime_requested") is not True:
        errors.append("communications wrapper expected communications_runtime_requested=true")
    if nested.get("communications_runtime_passed") is not True:
        errors.append("communications wrapper did not pass")
    for key in (
        "audit_records_replayed",
        "checkpoints_recovered",
        "channel_session_validations",
        "channel_session_accepted",
        "channel_session_rejected_unauthenticated",
        "channel_session_rejected_replay",
        "channel_session_rejected_provider_unavailable",
        "notification_channel_count",
    ):
        if (parse_positive_int(nested.get(key)) or 0) <= 0:
            errors.append(f"communications wrapper expected positive {key}")
    for key in (
        "command_operator_authorized",
        "command_route_accepted",
        "remote_command_security_ready",
        "platform_command_ingress_ready",
        "platform_adapter_ready",
    ):
        if nested.get(key) != "true":
            errors.append(f"communications wrapper expected {key}=true")
    for key in (
        "platform_command_token_secret_material_present",
        "platform_command_replay_nonce_reused",
        "platform_command_injection_detected",
        "platform_command_provider_rate_limited",
        "platform_command_provider_outage_observed",
        "remote_command_injection_detected",
        "outbound_network_used",
        "remote_commands_enabled",
        "external_submission_performed",
        "live_execution_performed",
        "signing_or_broadcast_performed",
        "production_ready",
    ):
        if nested.get(key) != "false":
            errors.append(f"communications wrapper expected {key}=false")
    return errors


def validate_dashboard_wrapper(report: dict[str, Any]) -> list[str]:
    errors, nested = validate_wrapper_common(report, "dashboard_runtime")
    if report.get("dashboard_runtime_requested") is not True:
        errors.append("dashboard wrapper expected dashboard_runtime_requested=true")
    if nested.get("dashboard_runtime_passed") is not True:
        errors.append("dashboard wrapper did not pass")
    for key in (
        "audit_records_replayed",
        "checkpoints_recovered",
        "render_panel_count",
        "hosted_runtime_remaining_external_evidence_count",
    ):
        if (parse_positive_int(nested.get(key)) or 0) <= 0:
            errors.append(f"dashboard wrapper expected positive {key}")
    for key in (
        "render_access_authorized",
        "hosted_security_ready",
        "hosted_request_preflight_ready",
        "hosted_request_validation_ready",
        "hosted_runtime_readiness_review_ready",
        "hosted_runtime_security_review_ready",
        "hosted_runtime_preflight_ready",
        "hosted_runtime_session_ready",
        "hosted_runtime_accepted_request_validated",
        "hosted_runtime_unauthenticated_rejection_validated",
        "hosted_runtime_csrf_rejection_validated",
        "hosted_runtime_rate_limit_rejection_validated",
        "hosted_runtime_loopback_serving_validated",
        "hosted_runtime_secure_headers_validated",
        "local_dashboard_server_started",
        "network_request_served",
    ):
        if nested.get(key) != "true":
            errors.append(f"dashboard wrapper expected {key}=true")
    for key in (
        "live_controls_enabled",
        "persistent_dashboard_server_started",
        "external_submission_performed",
        "live_execution_performed",
        "production_ready",
        "public_network_exposed",
    ):
        if nested.get(key) != "false":
            errors.append(f"dashboard wrapper expected {key}=false")
    if nested.get("local_http_status_code") != "200":
        errors.append("dashboard wrapper expected local_http_status_code=200")
    return errors


def validate_observability_wrapper(report: dict[str, Any]) -> list[str]:
    errors, nested = validate_wrapper_common(report, "observability_runtime")
    if report.get("observability_runtime_requested") is not True:
        errors.append("observability wrapper expected observability_runtime_requested=true")
    if nested.get("observability_runtime_passed") is not True:
        errors.append("observability wrapper did not pass")
    for key in (
        "audit_records_replayed",
        "checkpoints_recovered",
        "metric_lines",
        "scrape_metric_lines",
        "served_metric_lines",
    ):
        if (parse_positive_int(nested.get(key)) or 0) <= 0:
            errors.append(f"observability wrapper expected positive {key}")
    for key in (
        "listener_opened_and_closed",
        "loopback_bind_validated",
        "local_metrics_endpoint_started",
        "metrics_endpoint_started",
        "network_request_served",
    ):
        if nested.get(key) != "true":
            errors.append(f"observability wrapper expected {key}=true")
    for key in (
        "external_submission_performed",
        "live_execution_performed",
        "outbound_alerts_sent",
        "production_ready",
        "public_network_exposed",
        "telemetry_exported",
    ):
        if nested.get(key) != "false":
            errors.append(f"observability wrapper expected {key}=false")
    return errors


def validate_runtime_smoke(parsed: dict[str, str]) -> list[str]:
    errors: list[str] = []
    true_fields = (
        "lifecycle-completed",
        "graceful-shutdown-checkpointed",
        "backup-restore-validated",
        "restart-recovery-validated",
        "audit-durability-validated",
        "concurrent-lifecycle-validated",
        "observability-collected",
        "observability-checkpoint-recovered",
        "observability-operations-reviewed",
        "observability-operations-checkpoint-recovered",
        "observability-export-dry-run-rendered",
        "observability-export-checkpoint-recovered",
        "observability-alert-route-dispatched",
        "observability-alert-route-checkpoint-recovered",
        "observability-endpoint-preflighted",
        "observability-endpoint-checkpoint-recovered",
        "observability-loopback-bind-validated",
        "observability-loopback-bind-checkpoint-recovered",
        "observability-metrics-scrape-preflighted",
        "observability-metrics-scrape-checkpoint-recovered",
        "observability-metrics-endpoint-validated",
        "observability-metrics-endpoint-checkpoint-recovered",
        "observability-tracing-captured",
        "observability-tracing-checkpoint-recovered",
        "communications-command-routed",
        "communications-command-route-checkpoint-recovered",
        "communications-remote-command-reviewed",
        "communications-remote-command-review-checkpoint-recovered",
        "communications-platform-command-ingress-validated",
        "communications-platform-command-ingress-checkpoint-recovered",
        "communications-remote-command-envelope-validated",
        "communications-remote-command-envelope-checkpoint-recovered",
        "communications-channel-adapter-validated",
        "communications-channel-adapter-checkpoint-recovered",
        "communications-channel-session-validated",
        "communications-channel-session-checkpoint-recovered",
        "communications-platform-adapter-reviewed",
        "communications-platform-adapter-checkpoint-recovered",
        "communications-notification-dispatched",
        "communications-notification-checkpoint-recovered",
        "dashboard-rendered",
        "dashboard-checkpoint-recovered",
        "dashboard-hosted-security-reviewed",
        "dashboard-hosted-security-checkpoint-recovered",
        "dashboard-hosted-request-preflighted",
        "dashboard-hosted-request-preflight-checkpoint-recovered",
        "dashboard-hosted-request-validated",
        "dashboard-hosted-request-validation-checkpoint-recovered",
        "validation-run-recorded",
        "validation-run-checkpoint-recovered",
        "validation-property-checks-passed",
        "validation-property-checkpoint-recovered",
        "failure-capture-validated",
        "failure-capture-checkpoint-recovered",
        "restart-plan-checkpoint-recovered",
        "restart-adapter-checkpoint-recovered",
        "restart-adapter-recovery-plan-checkpoint-recovered",
        "restart-graceful-shutdown-checkpoint-recovered",
        "restart-opportunity-trace-recovery-validated",
        "opportunity-trace-recovery-validated",
        "production-runtime-preflight-local-smoke-validated",
        "production-runtime-preflight-local-smoke-load-validated",
    )
    false_fields = (
        "concurrent-lifecycle-external-submission-performed",
        "concurrent-lifecycle-live-execution-performed",
        "observability-metrics-endpoint-started",
        "observability-public-network-exposed",
        "observability-outbound-alerts-sent",
        "observability-telemetry-exported",
        "observability-production-ready",
        "communications-execution-enabled",
        "communications-remote-commands-enabled",
        "communications-outbound-network-used",
        "dashboard-server-started",
        "dashboard-public-network-exposed",
        "dashboard-live-controls-enabled",
        "dashboard-hosted-production-ready",
        "validation-external-fuzzer-invoked",
        "validation-live-network-used",
        "validation-live-execution-submitted",
        "validation-signing-or-broadcast-performed",
        "failure-capture-metrics-endpoint-started",
        "failure-capture-public-network-exposed",
        "failure-capture-outbound-alerts-sent",
        "failure-capture-external-submission-performed",
        "failure-capture-live-execution-performed",
        "service-manager-action-performed",
        "external-submission-performed",
        "live-execution-performed",
        "production-ready",
        "production-runtime-preflight-service-manager-evidence-available",
        "production-runtime-preflight-disk-full-evidence-available",
        "production-runtime-preflight-production-ready",
    )
    for key in true_fields:
        if parsed.get(key) != "true":
            errors.append(f"runtime-smoke expected {key}=true")
    for key in false_fields:
        if parsed.get(key) != "false":
            errors.append(f"runtime-smoke expected {key}=false")
    for key in (
        "concurrent-lifecycle-workers",
        "concurrent-lifecycle-audit-records-replayed",
        "dashboard-panel-count",
        "restart-audit-records-replayed",
        "restart-opportunity-trace-discovered",
        "restart-opportunity-trace-recovered-checkpoints",
        "restart-opportunity-trace-recovered-summaries",
        "opportunity-trace-discovered",
        "opportunity-trace-audit-records-replayed",
        "opportunity-trace-recovered-checkpoints",
        "opportunity-trace-recovered-summaries",
        "runtime-smoke-iterations",
        "runtime-smoke-load-iterations-attempted",
        "runtime-smoke-load-iterations-passed",
        "runtime-smoke-load-total-elapsed-ms",
        "runtime-smoke-load-restart-audit-records-replayed",
        "runtime-smoke-load-backup-audit-records-replayed",
        "runtime-smoke-load-opportunity-trace-recovered-checkpoints",
        "runtime-smoke-load-opportunity-trace-recovered-summaries",
        "production-runtime-preflight-unresolved-blockers",
    ):
        if (parse_positive_int(parsed.get(key)) or 0) <= 0:
            errors.append(f"runtime-smoke expected positive {key}")
    if parsed.get("runtime-smoke") != "validation passed":
        errors.append("runtime-smoke did not report validation passed")
    if parsed.get("runtime-smoke-load-validation") != "passed":
        errors.append("runtime-smoke expected runtime-smoke-load-validation=passed")
    if parsed.get("production-runtime-preflight") != "validation passed":
        errors.append("runtime-smoke expected production-runtime-preflight=validation passed")
    if parsed.get("production-runtime-preflight-status") != "BlockedPendingProductionHostValidation":
        errors.append("runtime-smoke expected BlockedPendingProductionHostValidation preflight status")
    return errors


def main() -> int:
    with tempfile.TemporaryDirectory(
        prefix="operator-surface-gate-", dir=ROOT / "target"
    ) as temp_dir:
        workspace_root = pathlib.Path(temp_dir)
        components = [
            run_text_command(
                "communications_runtime_cli",
                [
                    "cargo",
                    "run",
                    "-p",
                    "arb-agent",
                    "--",
                    "validate-communications-runtime",
                    "--workspace",
                    str(workspace_root / "communications-runtime"),
                ],
            ),
            run_text_command(
                "communications_outbox_cli",
                [
                    "cargo",
                    "run",
                    "-p",
                    "arb-agent",
                    "--",
                    "validate-communications-outbox",
                    "--workspace",
                    str(workspace_root / "communications-outbox"),
                ],
            ),
            run_text_command(
                "dashboard_runtime_cli",
                [
                    "cargo",
                    "run",
                    "-p",
                    "arb-agent",
                    "--",
                    "validate-dashboard-runtime",
                    "--workspace",
                    str(workspace_root / "dashboard-runtime"),
                ],
            ),
            run_text_command(
                "dashboard_loopback_runtime_cli",
                [
                    "cargo",
                    "run",
                    "-p",
                    "arb-agent",
                    "--",
                    "validate-dashboard-loopback-runtime",
                    "--workspace",
                    str(workspace_root / "dashboard-loopback-runtime"),
                ],
            ),
            run_text_command(
                "observability_runtime_cli",
                [
                    "cargo",
                    "run",
                    "-p",
                    "arb-agent",
                    "--",
                    "validate-observability-runtime",
                    "--workspace",
                    str(workspace_root / "observability-runtime"),
                ],
            ),
            run_json_command(
                "communications_runtime_wrapper",
                [
                    sys.executable,
                    "scripts/validate_deployment_host_runtime.py",
                    "--run-communications-runtime",
                    "--communications-workspace",
                    str(workspace_root / "deployment-communications-runtime"),
                    "--json",
                ],
            ),
            run_json_command(
                "dashboard_runtime_wrapper",
                [
                    sys.executable,
                    "scripts/validate_deployment_host_runtime.py",
                    "--run-dashboard-runtime",
                    "--dashboard-workspace",
                    str(workspace_root / "deployment-dashboard-runtime"),
                    "--json",
                ],
            ),
            run_json_command(
                "observability_runtime_wrapper",
                [
                    sys.executable,
                    "scripts/validate_deployment_host_runtime.py",
                    "--run-observability-runtime",
                    "--observability-workspace",
                    str(workspace_root / "deployment-observability-runtime"),
                    "--json",
                ],
            ),
            run_text_command(
                "runtime_smoke",
                [
                    "cargo",
                    "run",
                    "-p",
                    "arb-agent",
                    "--",
                    "validate-runtime-smoke",
                    "--config",
                    "config.example.toml",
                    "--workspace",
                    str(workspace_root / "runtime-smoke"),
                    "--iterations",
                    "1",
                ],
            ),
        ]

    validators = {
        "communications_runtime_cli": lambda component: validate_communications_cli(component["parsed"]),
        "communications_outbox_cli": lambda component: validate_communications_outbox_cli(component["parsed"]),
        "dashboard_runtime_cli": lambda component: validate_dashboard_cli(component["parsed"]),
        "dashboard_loopback_runtime_cli": lambda component: validate_dashboard_loopback_runtime_cli(component["parsed"]),
        "observability_runtime_cli": lambda component: validate_observability_cli(component["parsed"]),
        "communications_runtime_wrapper": lambda component: validate_communications_wrapper(component["json_report"] or {}),
        "dashboard_runtime_wrapper": lambda component: validate_dashboard_wrapper(component["json_report"] or {}),
        "observability_runtime_wrapper": lambda component: validate_observability_wrapper(component["json_report"] or {}),
        "runtime_smoke": lambda component: validate_runtime_smoke(component["parsed"]),
    }

    errors: list[str] = []
    for component in components:
        if component["returncode"] != 0:
            errors.append(f"{component['name']} exited {component['returncode']}")
            continue
        errors.extend(validators[component["name"]](component))

    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    report = {
        "schema": "arbyclaw.operator_surface_aggregate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "outbound_network_used": False,
        "public_network_exposed": False,
        "service_actions_performed": False,
        "external_submission_performed": False,
        "signing_or_broadcast_performed": False,
        "live_execution_performed": False,
        "production_ready": False,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_validation": [
            "real platform authentication and delivery validation",
            "browser/server hosted dashboard validation under daemon orchestration",
            "daemon-hosted observability/exporter/alert validation",
            "deployment-host restart/recovery and external AppSec review",
        ],
    }
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
