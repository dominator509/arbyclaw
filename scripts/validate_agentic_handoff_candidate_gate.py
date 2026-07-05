#!/usr/bin/env python3
"""Run the strongest current local agentic handoff candidate validation bundle.

This gate composes the local Phase 18 handoff audit/state replay validator, the
execution-path, operator-surface, opportunity-scenario, connector-scenario,
Phase 17 hardening-core aggregate gate, and the local deployment evidence
checklist. It preserves local-only/non-secret behavior: no external agent
execution, no production deployment, no live trading, no signing, no
broadcasts, and no production-readiness claims.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT_SECONDS = 2_400


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    parser.add_argument(
        "--require-systemd-analyze",
        action="store_true",
        help="require systemd-analyze inside the nested hardening-core gate",
    )
    return parser.parse_args()


def command_set(
    workspace_root: pathlib.Path, args: argparse.Namespace
) -> list[tuple[str, list[str]]]:
    hardening = [sys.executable, "scripts/validate_hardening_core_gate.py", "--json"]
    if args.require_systemd_analyze:
        hardening.append("--require-systemd-analyze")
    return [
        (
            "agentic_handoff_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-agentic-handoff-audit",
                "--workspace",
                str(workspace_root / "agentic-handoff-audit"),
            ],
        ),
        (
            "execution_path_gate",
            [sys.executable, "scripts/validate_execution_path_gate.py"],
        ),
        (
            "operator_surface_gate",
            [sys.executable, "scripts/validate_operator_surface_gate.py"],
        ),
        (
            "opportunity_scenario_gate",
            [sys.executable, "scripts/validate_opportunity_scenario_gate.py", "--json"],
        ),
        (
            "connector_scenario_gate",
            [sys.executable, "scripts/validate_connector_scenario_gate.py", "--json"],
        ),
        ("hardening_core_gate", hardening),
        (
            "deployment_evidence_checklist",
            [sys.executable, "scripts/validate_deployment_evidence_checklist.py", "--json"],
        ),
    ]


def parse_output(text: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or ": " not in line:
            continue
        key, value = line.split(": ", 1)
        parsed[key.strip()] = value.strip()
    return parsed


def run_component(name: str, command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    json_report: dict[str, Any] | None = None
    parsed = parse_output(output)
    if completed.returncode == 0:
        json_report = extract_json_report(output)
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "json_report": json_report,
        "parsed": parsed,
        "output_tail": output.splitlines()[-20:],
    }


def extract_json_report(output: str) -> dict[str, Any] | None:
    decoder = json.JSONDecoder()
    for index, char in enumerate(output):
        if char != "{":
            continue
        loaded, _ = decoder.raw_decode(output[index:])
        if not isinstance(loaded, dict):
            raise RuntimeError("validator did not emit a JSON object")
        return loaded
    return None


def require_false_fields(
    report: dict[str, Any],
    gate_name: str,
    fields: tuple[str, ...],
    errors: list[str],
) -> None:
    for field in fields:
        if report.get(field) is not False:
            errors.append(f"{gate_name} reported unsafe field {field}")


def validate_json_gate(
    report: dict[str, Any],
    gate_name: str,
    false_fields: tuple[str, ...],
    errors: list[str],
) -> None:
    if report.get("all_components_passed") is not True:
        errors.append(f"{gate_name} did not pass every component")
    if report.get("unsafe_side_effect_flags_detected") is not False:
        errors.append(f"{gate_name} detected unsafe side-effect flags")
    require_false_fields(report, gate_name, false_fields, errors)


def json_report_for(
    component_by_name: dict[str, dict[str, Any]],
    component_name: str,
    gate_name: str,
    errors: list[str],
) -> dict[str, Any]:
    report = component_by_name[component_name]["json_report"]
    if not isinstance(report, dict):
        errors.append(f"{gate_name} did not produce a JSON report")
        return {}
    return report


def validate_components(components: list[dict[str, Any]]) -> tuple[list[str], bool]:
    errors: list[str] = []
    bounded_toolchain_external_path_used = False
    component_by_name = {component["name"]: component for component in components}

    for component in components:
        if component["returncode"] != 0:
            errors.append(f"{component['name']} exited {component['returncode']}")
            errors.extend(f"{component['name']} output: {line}" for line in component["output_tail"])

    if errors:
        return errors, bounded_toolchain_external_path_used

    handoff = component_by_name["agentic_handoff_audit"]["parsed"]
    if handoff.get("handoff-package") in {None, ""}:
        errors.append("agentic handoff audit did not report a package id")
    for positive_key in (
        "handoff-artifacts",
        "handoff-unresolved-gaps",
        "handoff-live-funds-blockers",
        "audit-records-replayed",
    ):
        if handoff.get(positive_key) in {None, "", "0"}:
            errors.append(f"agentic handoff audit did not report positive {positive_key}")
    for true_key in (
        "handoff-audit-failed-closed",
        "state-failure-failed-closed",
        "state-checkpoints-recovered",
    ):
        if handoff.get(true_key) != "true":
            errors.append(f"agentic handoff audit did not confirm {true_key}")
    for false_key in (
        "external-agents-executed",
        "external-validation-claimed",
        "production-ready",
        "live-funds-approved",
        "public-exposure-approved",
        "secret-material-recorded",
    ):
        if handoff.get(false_key) != "false":
            errors.append(f"agentic handoff audit reported unsafe field {false_key}")

    execution_path = json_report_for(component_by_name, "execution_path_gate", "execution path gate", errors)
    validate_json_gate(
        execution_path,
        "execution path gate",
        (
            "external_calls_performed",
            "external_submission_performed",
            "signer_material_loaded",
            "plaintext_decrypted",
            "signing_performed",
            "broadcast_performed",
            "live_execution_performed",
            "production_ready",
        ),
        errors,
    )

    operator_surface = json_report_for(component_by_name, "operator_surface_gate", "operator surface gate", errors)
    validate_json_gate(
        operator_surface,
        "operator surface gate",
        (
            "outbound_network_used",
            "public_network_exposed",
            "service_actions_performed",
            "external_submission_performed",
            "signing_or_broadcast_performed",
            "live_execution_performed",
            "production_ready",
        ),
        errors,
    )

    opportunity_scenario = json_report_for(
        component_by_name, "opportunity_scenario_gate", "opportunity scenario gate", errors
    )
    validate_json_gate(
        opportunity_scenario,
        "opportunity scenario gate",
        (
            "external_calls_performed",
            "external_data_downloaded",
            "adapter_submission_performed",
            "external_fuzzer_invoked",
            "live_network_used",
            "signing_or_broadcast_performed",
            "live_execution_performed",
            "production_ready",
        ),
        errors,
    )
    if opportunity_scenario.get("opportunity_replay_latency_review_enforced") is not True:
        errors.append("opportunity scenario gate did not enforce replay latency review")
    if opportunity_scenario.get("local_validation_coverage_review_enforced") is not True:
        errors.append("opportunity scenario gate did not enforce validation coverage review")

    connector_scenario = json_report_for(
        component_by_name, "connector_scenario_gate", "connector scenario gate", errors
    )
    validate_json_gate(
        connector_scenario,
        "connector scenario gate",
        (
            "live_network_used",
            "credential_loaded",
            "websocket_connection_opened",
            "live_provider_call_performed",
            "external_submission_performed",
            "rpc_call_performed",
            "signing_or_broadcast_performed",
            "live_execution_performed",
            "production_ready",
        ),
        errors,
    )

    hardening = json_report_for(component_by_name, "hardening_core_gate", "hardening core gate", errors)
    if hardening.get("all_components_passed") is not True:
        errors.append("hardening core gate did not pass every component")
    if hardening.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("hardening core gate detected unsafe side-effect flags")
    for field in (
        "deployment_performed",
        "service_installed",
        "service_actions_performed",
        "network_listeners_started",
        "secrets_loaded",
        "live_execution_enabled",
        "production_readiness_claimed",
    ):
        if hardening.get(field) is not False:
            errors.append(f"hardening core gate reported unsafe field {field}")
    bounded_toolchain_external_path_used = (
        hardening.get("bounded_toolchain_external_path_used") is True
    )

    checklist = json_report_for(
        component_by_name, "deployment_evidence_checklist", "deployment evidence checklist", errors
    )
    bundle_index = checklist.get("bundle_index", {})
    if bundle_index.get("all_components_passed") is not True:
        errors.append("deployment evidence checklist bundle index did not pass")
    for field in (
        "production_readiness_claimed",
        "service_actions_performed",
        "files_changed",
        "secrets_loaded",
        "external_calls_performed",
        "alerts_sent",
        "live_execution_enabled",
        "artifact_contents_embedded",
    ):
        if checklist.get(field) is not False:
            errors.append(f"deployment evidence checklist reported unsafe field {field}")

    return errors, bounded_toolchain_external_path_used


def main() -> int:
    args = parse_args()
    with tempfile.TemporaryDirectory(
        prefix="agentic-handoff-candidate-gate-", dir=ROOT / "target"
    ) as temp_dir:
        workspace_root = pathlib.Path(temp_dir)
        components = [
            run_component(name, command)
            for name, command in command_set(workspace_root, args)
        ]
    errors, bounded_toolchain_external_path_used = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    hardening = next(
        component["json_report"]
        for component in components
        if component["name"] == "hardening_core_gate"
    )
    checklist = next(
        component["json_report"]
        for component in components
        if component["name"] == "deployment_evidence_checklist"
    )
    assert hardening is not None
    assert checklist is not None
    extra_remaining_evidence = []
    for component in components:
        report = component["json_report"]
        if not isinstance(report, dict):
            continue
        for key in ("remaining_external_evidence", "remaining_external_validation"):
            values = report.get(key, [])
            if isinstance(values, list):
                extra_remaining_evidence.extend(str(value) for value in values)

    report = {
        "schema": "arbyclaw.agentic_handoff_candidate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "bounded_toolchain_external_path_used": bounded_toolchain_external_path_used,
        "external_agent_execution_performed": False,
        "deployment_performed": False,
        "service_installed": False,
        "service_actions_performed": False,
        "network_listeners_started": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "external_validation_claimed": False,
        "production_readiness_claimed": False,
        "live_funds_approved": False,
        "public_exposure_approved": False,
        "secret_material_recorded": False,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_evidence": hardening.get("remaining_external_evidence", []),
        "remaining_software_surface_external_evidence": sorted(set(extra_remaining_evidence)),
        "remaining_external_checklist_categories": checklist.get(
            "remaining_missing_categories", []
        ),
    }
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("agentic handoff candidate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print(
            "bounded-toolchain-external-path-used: "
            f"{str(report['bounded_toolchain_external_path_used']).lower()}"
        )
        print("external-agent-execution-performed: false")
        print("deployment-performed: false")
        print("service-installed: false")
        print("service-actions-performed: false")
        print("network-listeners-started: false")
        print("secrets-loaded: false")
        print("live-execution-enabled: false")
        print("external-validation-claimed: false")
        print("production-readiness-claimed: false")
        print("live-funds-approved: false")
        print("public-exposure-approved: false")
        print("secret-material-recorded: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
