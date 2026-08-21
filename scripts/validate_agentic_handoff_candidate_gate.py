#!/usr/bin/env python3
"""Run the top local handoff gate without recursively re-running sibling suites.

`validate_hardening_core_gate.py` is the single aggregate owner for packaging,
execution-path, operator-surface, opportunity, connector, deployment-evidence,
license, secret, withdrawal, signer, destination, and policy checks. This gate
adds only the handoff-specific audit/state replay check and consumes the
hardening result once.
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


def command_set(workspace_root: pathlib.Path, args: argparse.Namespace) -> list[tuple[str, list[str]]]:
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
        ("hardening_core_gate", hardening),
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


def extract_json_report(output: str) -> dict[str, Any] | None:
    decoder = json.JSONDecoder()
    for index, char in enumerate(output):
        if char != "{":
            continue
        try:
            loaded, _ = decoder.raw_decode(output[index:])
        except json.JSONDecodeError:
            continue
        if isinstance(loaded, dict):
            return loaded
    return None


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
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "parsed": parse_output(output),
        "json_report": extract_json_report(output) if "--json" in command else None,
        "output_tail": output.splitlines()[-30:],
    }


def validate_components(components: list[dict[str, Any]]) -> tuple[list[str], bool, list[str]]:
    errors: list[str] = []
    by_name = {component["name"]: component for component in components}

    if len(components) != 2 or set(by_name) != {"agentic_handoff_audit", "hardening_core_gate"}:
        errors.append("handoff gate command graph drifted; expected exactly two top-level components")
        return errors, False, []

    for component in components:
        if component["returncode"] != 0:
            errors.append(f"{component['name']} exited {component['returncode']}")
            errors.extend(
                f"{component['name']} output: {line}"
                for line in component["output_tail"]
            )

    if errors:
        return errors, False, []

    handoff = by_name["agentic_handoff_audit"]["parsed"]
    if handoff.get("handoff-package") in {None, ""}:
        errors.append("agentic handoff audit did not report a package id")
    for key in (
        "handoff-artifacts",
        "handoff-unresolved-gaps",
        "handoff-live-funds-blockers",
        "audit-records-replayed",
    ):
        if handoff.get(key) in {None, "", "0"}:
            errors.append(f"agentic handoff audit did not report positive {key}")
    for key in (
        "handoff-audit-failed-closed",
        "state-failure-failed-closed",
        "state-checkpoints-recovered",
    ):
        if handoff.get(key) != "true":
            errors.append(f"agentic handoff audit did not confirm {key}")
    for key in (
        "external-agents-executed",
        "external-validation-claimed",
        "production-ready",
        "live-funds-approved",
        "public-exposure-approved",
        "secret-material-recorded",
    ):
        if handoff.get(key) != "false":
            errors.append(f"agentic handoff audit reported unsafe field {key}")

    hardening = by_name["hardening_core_gate"]["json_report"]
    if not isinstance(hardening, dict):
        errors.append("hardening core gate did not emit a JSON report")
        return errors, False, []
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

    remaining = hardening.get("remaining_external_evidence", [])
    if not isinstance(remaining, list):
        errors.append("hardening core gate remaining_external_evidence must be a list")
        remaining = []

    return (
        errors,
        hardening.get("bounded_toolchain_external_path_used") is True,
        [str(item) for item in remaining],
    )


def main() -> int:
    args = parse_args()
    (ROOT / "target").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="agentic-handoff-gate-", dir=ROOT / "target") as temp_dir:
        components = [
            run_component(name, command)
            for name, command in command_set(pathlib.Path(temp_dir), args)
        ]

    errors, bounded_toolchain_external_path_used, remaining = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    report = {
        "schema": "arbyclaw.agentic_handoff_candidate_gate.v2",
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
        "remaining_external_evidence": remaining,
    }

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("agentic handoff candidate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print("external-agent-execution-performed: false")
        print("production-readiness-claimed: false")
        print("live-funds-approved: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
