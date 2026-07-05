#!/usr/bin/env python3
"""Build a local non-secret deployment evidence index.

This runner executes only non-mutating validation helpers and emits a compact
operator-review index. It does not install units, start services, stop services,
restart services, change deployment state, send alerts, call networks, load
secrets, or claim production readiness.
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
COMPONENT_TIMEOUT_SECONDS = 300
BUNDLE_WORKSPACE_ROOT = ROOT / "target/deployment-evidence-bundle"
BUNDLE_WORKSPACE = BUNDLE_WORKSPACE_ROOT / f"run-{os.getpid()}"

def component_commands(bundle_workspace: pathlib.Path) -> list[tuple[str, list[str], bool]]:
    dashboard_runtime_workspace = bundle_workspace / "dashboard-runtime"
    dashboard_loopback_runtime_workspace = bundle_workspace / "dashboard-loopback-runtime"
    observability_metrics_workspace = bundle_workspace / "observability-metrics-runtime"
    observability_provider_boundary_workspace = (
        bundle_workspace / "observability-provider-boundary"
    )
    observability_provider_submission_workspace = (
        bundle_workspace / "observability-provider-submission"
    )
    communications_delivery_provider_workspace = (
        bundle_workspace / "communications-delivery-provider"
    )
    communications_provider_submission_workspace = (
        bundle_workspace / "communications-provider-submission"
    )
    deployment_config_redaction_workspace = bundle_workspace / "deployment-config-redaction"
    deployment_log_redaction_workspace = bundle_workspace / "deployment-log-redaction"
    deployment_runtime_workspace = bundle_workspace / "deployment-runtime-gate"
    return [
        (
        "structure",
        [sys.executable, "scripts/validate_structure.py"],
        False,
    ),
    (
        "systemd-example",
        [sys.executable, "scripts/validate_systemd_example.py"],
        False,
    ),
    (
        "systemd-lifecycle-plan",
        [sys.executable, "scripts/validate_systemd_lifecycle.py", "--json"],
        True,
    ),
    (
        "deployment-host-runtime-plan",
        [sys.executable, "scripts/validate_deployment_host_runtime.py", "--json"],
        True,
    ),
    (
        "deployment-host-dashboard-runtime",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-dashboard-runtime",
            "--dashboard-workspace",
            str(dashboard_runtime_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-dashboard-loopback-runtime",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-dashboard-loopback-runtime",
            "--dashboard-loopback-workspace",
            str(dashboard_loopback_runtime_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-observability-metrics-runtime",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-observability-metrics-runtime",
            "--observability-metrics-workspace",
            str(observability_metrics_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-observability-provider-boundary",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-observability-provider-boundary",
            "--observability-provider-boundary-workspace",
            str(observability_provider_boundary_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-observability-provider-submission",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-observability-provider-submission-preflight",
            "--observability-provider-submission-workspace",
            str(observability_provider_submission_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-communications-delivery-provider",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-communications-delivery-provider-boundary",
            "--communications-delivery-provider-workspace",
            str(communications_delivery_provider_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-communications-provider-submission",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-communications-provider-submission-preflight",
            "--communications-provider-submission-workspace",
            str(communications_provider_submission_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-static-hardening-config-smoke",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-deployment-static-hardening",
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-config-redaction",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-deployment-config-redaction",
            "--deployment-config-redaction-workspace",
            str(deployment_config_redaction_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-log-redaction",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-deployment-log-redaction",
            "--deployment-log-redaction-workspace",
            str(deployment_log_redaction_workspace),
            "--json",
        ],
        True,
    ),
    (
        "deployment-host-retention-preflight",
        [
            sys.executable,
            "scripts/validate_deployment_host_runtime.py",
            "--run-retention-preflight",
            "--retention-active-path",
            "deployment/audit.jsonl",
            "--retention-archive-dir",
            "deployment",
            "--json",
        ],
        True,
    ),
    (
        "deployment-runtime-gate",
        [
            sys.executable,
            "scripts/validate_deployment_runtime_gate.py",
            "--workspace-base",
            str(deployment_runtime_workspace),
            "--json",
        ],
        True,
    ),
    (
        "opportunity-scenario-gate",
        [sys.executable, "scripts/validate_opportunity_scenario_gate.py", "--json"],
        True,
    ),
    (
        "connector-scenario-gate",
        [sys.executable, "scripts/validate_connector_scenario_gate.py", "--json"],
        True,
    ),
    (
        "rollback-drill-plan",
        [sys.executable, "scripts/validate_rollback_drill.py", "--json"],
        True,
    ),
    (
        "rollback-execution-transcript",
        ["cargo", "run", "-p", "arb-agent", "--", "validate-rollback-execution-transcript"],
        False,
    ),
    (
        "incident-response-drill-plan",
        [sys.executable, "scripts/validate_incident_response_drill.py", "--json"],
        True,
    ),
    (
        "incident-response-execution-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-incident-response-execution-transcript",
        ],
        False,
    ),
    (
        "deployment-disk-full-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-disk-full-transcript",
        ],
        False,
    ),
    (
        "deployment-retention-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-retention-transcript",
        ],
        False,
    ),
    (
        "deployment-permission-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-permission-transcript",
        ],
        False,
    ),
    (
        "deployment-audit-sqlite-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-audit-sqlite-transcript",
        ],
        False,
    ),
    (
        "deployment-backup-restore-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-backup-restore-transcript",
        ],
        False,
    ),
    (
        "deployment-graceful-shutdown-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-graceful-shutdown-transcript",
        ],
        False,
    ),
    (
        "service-manager-lifecycle-rehearsal",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-service-manager-lifecycle-rehearsal",
        ],
        False,
    ),
    (
        "deployment-sqlite-schema-migration-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-sqlite-schema-migration-transcript",
        ],
        False,
    ),
    (
        "deployment-failure-capture-transcript",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-failure-capture-transcript",
        ],
        False,
    ),
    (
        "deployment-response-drill-rehearsal",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-deployment-response-drill-rehearsal",
        ],
        False,
    ),
    ]

BOOLEAN_SAFETY_FIELDS = (
    "service_actions_performed",
    "files_changed",
    "secrets_loaded",
    "external_calls_performed",
    "alerts_sent",
    "live_execution_enabled",
    "production_readiness_claimed",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON instead of text")
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"deployment evidence bundle validation failed: {message}", file=sys.stderr)
    return 1


def prepare_bundle_workspace() -> pathlib.Path:
    resolved = BUNDLE_WORKSPACE.resolve()
    target_root = (ROOT / "target").resolve()
    try:
        resolved.relative_to(target_root)
    except ValueError as exc:
        raise RuntimeError("bundle workspace must resolve inside repository target/") from exc
    if resolved.exists():
        shutil.rmtree(resolved)
    resolved.mkdir(parents=True)
    return resolved


def run_component(name: str, command: list[str], expects_json: bool) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=COMPONENT_TIMEOUT_SECONDS,
    )
    summary: dict[str, Any] = {
        "name": name,
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "stdout_line_count": len(completed.stdout.splitlines()),
        "json_report": expects_json,
        "timeout_seconds": COMPONENT_TIMEOUT_SECONDS,
    }
    if expects_json and completed.returncode == 0:
        try:
            report = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"{name} did not emit valid JSON: {error}") from error
        summary.update(summarize_json_report(report))
    return summary


def summarize_json_report(report: dict[str, Any]) -> dict[str, Any]:
    safety_flags = {
        field: report[field]
        for field in BOOLEAN_SAFETY_FIELDS
        if field in report and isinstance(report[field], bool)
    }
    remaining = report.get("remaining_external_evidence", [])
    if not isinstance(remaining, list):
        remaining = []
    return {
        "schema": report.get("schema"),
        "metadata_complete": report.get("metadata_complete"),
        "safety_flags": safety_flags,
        "remaining_external_evidence_count": len(remaining),
    }


def build_report() -> dict[str, Any]:
    bundle_workspace = prepare_bundle_workspace()
    components = [
        run_component(name, command, expects_json)
        for name, command, expects_json in component_commands(bundle_workspace)
    ]
    failed = [component["name"] for component in components if not component["passed"]]
    unsafe_flags: list[str] = []
    for component in components:
        flags = component.get("safety_flags", {})
        for key, value in flags.items():
            if key.endswith("_claimed") or key.endswith("_performed") or key.endswith("_enabled") or key == "secrets_loaded":
                if value is not False:
                    unsafe_flags.append(f"{component['name']}:{key}")

    return {
        "schema": "arbyclaw.deployment_evidence_bundle.v1",
        "components": components,
        "component_count": len(components),
        "bounded_timeouts": {
            "component_seconds": COMPONENT_TIMEOUT_SECONDS,
        },
        "failed_components": failed,
        "all_components_passed": not failed,
        "unsafe_flags": unsafe_flags,
        "service_actions_performed": False,
        "files_changed": False,
        "secrets_loaded": False,
        "external_calls_performed": False,
        "alerts_sent": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
        "artifact_contents_embedded": False,
        "remaining_external_evidence": [
            "operator-controlled service lifecycle execution evidence",
            "deployment-host backup/restore execution evidence",
            "deployment-host graceful shutdown execution evidence",
            "deployment-host audit and SQLite recovery evidence",
            "deployment-host SQLite schema migration execution evidence",
            "physical disk-full fail-closed evidence",
            "retention/rotation execution evidence",
            "executed rollback drill evidence",
            "executed incident-response drill evidence",
            "daemon failure-capture execution evidence",
            "human production-readiness review",
        ],
    }


def print_text_report(report: dict[str, Any]) -> None:
    print("deployment evidence bundle validation report")
    print(f"components: {report['component_count']}")
    print(f"component timeout seconds: {report['bounded_timeouts']['component_seconds']}")
    print(f"all components passed: {str(report['all_components_passed']).lower()}")
    print(f"service actions performed: {str(report['service_actions_performed']).lower()}")
    print(f"files changed: {str(report['files_changed']).lower()}")
    print(f"secrets loaded: {str(report['secrets_loaded']).lower()}")
    print(f"external calls performed: {str(report['external_calls_performed']).lower()}")
    print(f"alerts sent: {str(report['alerts_sent']).lower()}")
    print(f"live execution enabled: {str(report['live_execution_enabled']).lower()}")
    print(f"production readiness claimed: {str(report['production_readiness_claimed']).lower()}")
    print(f"artifact contents embedded: {str(report['artifact_contents_embedded']).lower()}")
    print("component summary:")
    for component in report["components"]:
        status = "passed" if component["passed"] else "failed"
        schema = component.get("schema") or "text"
        print(f"- {component['name']}: {status}, schema={schema}, lines={component['stdout_line_count']}")
    print("remaining external evidence:")
    for item in report["remaining_external_evidence"]:
        print(f"- {item}")


def main() -> int:
    args = parse_args()
    try:
        report = build_report()
    except subprocess.TimeoutExpired as error:
        return fail(f"bundle component timed out after {error.timeout} seconds")
    except (OSError, RuntimeError) as error:
        return fail(str(error))

    if report["failed_components"] or report["unsafe_flags"]:
        print(json.dumps(report, indent=2, sort_keys=True) if args.json else "", end="")
        return fail("one or more bundle components failed or reported unsafe flags")

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text_report(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
