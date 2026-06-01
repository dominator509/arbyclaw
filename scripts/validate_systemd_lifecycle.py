#!/usr/bin/env python3
"""Plan or inspect ArbyClaw systemd lifecycle validation without service actions.

The default plan mode is host-agnostic and non-mutating. Inspect mode is for a
Linux deployment host and only runs read-only systemctl queries. This script
never installs units, reloads systemd, enables services, starts services, stops
services, restarts services, loads secrets, or claims production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import platform
import re
import shutil
import subprocess
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_UNIT = "arb-agent.service"
UNIT_NAME_PATTERN = re.compile(r"^[A-Za-z0-9_.@-]+\.service$")
EXPECTED_TEMPLATE = ROOT / "deployment/systemd/arb-agent.service.example"

PLAN_STEPS = [
    "validate repository structure and Rust workspace on the candidate commit",
    "build or select the exact non-secret arb-agent artifact for the deployment host",
    "review deployment/systemd/arb-agent.service.example for the target host",
    "install the unit through an operator-controlled change outside this script",
    "run inspect mode on the deployment host after the unit exists",
    "run local runtime smoke validation against a fresh non-secret workspace",
    "perform operator-controlled start, graceful shutdown, restart, and recovery checks",
    "record only non-secret run URLs, artifact names, timestamps, and outcomes",
]

NON_CLAIMS = [
    "no service was installed",
    "no systemd daemon reload was performed",
    "no service was enabled",
    "no service was started",
    "no service was stopped",
    "no service was restarted",
    "no live trading was enabled",
    "no production readiness is claimed",
]

READ_ONLY_PROPERTIES = [
    "LoadState",
    "ActiveState",
    "SubState",
    "FragmentPath",
    "UnitFileState",
    "ExecMainStatus",
    "NRestarts",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--mode",
        choices=("plan", "inspect"),
        default="plan",
        help="plan is non-mutating and host-agnostic; inspect runs read-only systemctl queries",
    )
    parser.add_argument(
        "--unit",
        default=DEFAULT_UNIT,
        help="systemd unit name to plan or inspect; must end in .service",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="emit a JSON report instead of text",
    )
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"systemd lifecycle validation failed: {message}", file=sys.stderr)
    return 1


def validate_unit_name(unit: str) -> None:
    if not UNIT_NAME_PATTERN.fullmatch(unit):
        raise ValueError("unit must be a simple systemd .service name")
    lowered = unit.lower()
    if any(token in lowered for token in ("withdraw", "bridge", "sign", "broadcast", "live")):
        raise ValueError("unit name must not imply live funds, signing, bridges, or broadcasts")


def base_report(unit: str, mode: str) -> dict[str, Any]:
    return {
        "schema": "arbyclaw.systemd_lifecycle_validation.v1",
        "mode": mode,
        "unit": unit,
        "template_present": EXPECTED_TEMPLATE.exists(),
        "service_actions_performed": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
        "non_claims": NON_CLAIMS,
    }


def build_plan_report(unit: str) -> dict[str, Any]:
    report = base_report(unit, "plan")
    report.update(
        {
            "host_inspected": False,
            "systemctl_used": False,
            "operator_steps": PLAN_STEPS,
            "remaining_external_evidence": [
                "deployment-host unit installation evidence",
                "deployment-host start/shutdown/restart evidence",
                "deployment-host audit and SQLite recovery evidence",
                "deployment-host backup/restore evidence under runtime load",
                "rollback drill evidence",
            ],
        }
    )
    return report


def run_read_only_systemctl(unit: str) -> dict[str, Any]:
    binary = shutil.which("systemctl")
    if binary is None:
        raise RuntimeError("systemctl unavailable on this host")
    if platform.system() != "Linux":
        raise RuntimeError("inspect mode requires a Linux deployment host")

    command = [
        binary,
        "show",
        unit,
        "--no-pager",
        *(f"--property={name}" for name in READ_ONLY_PROPERTIES),
    ]
    completed = subprocess.run(
        command,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )

    properties: dict[str, str] = {}
    for line in completed.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in READ_ONLY_PROPERTIES:
            properties[key] = value

    return {
        "command": " ".join(command),
        "returncode": completed.returncode,
        "properties": properties,
        "raw_output_line_count": len(completed.stdout.splitlines()),
    }


def build_inspect_report(unit: str) -> dict[str, Any]:
    systemctl_report = run_read_only_systemctl(unit)
    if systemctl_report["returncode"] != 0:
        raise RuntimeError("read-only systemctl show inspection failed")
    if not systemctl_report["properties"].get("LoadState"):
        raise RuntimeError("read-only systemctl show did not return LoadState")
    report = base_report(unit, "inspect")
    report.update(
        {
            "host_inspected": True,
            "systemctl_used": True,
            "systemctl_show": systemctl_report,
            "inspection_passed": systemctl_report["returncode"] == 0,
            "remaining_external_evidence": [
                "operator-controlled service start/shutdown/restart results",
                "runtime smoke validation output from the deployment host",
                "non-secret audit/SQLite recovery evidence from the deployment host",
                "rollback drill result",
            ],
        }
    )
    return report


def print_text_report(report: dict[str, Any]) -> None:
    print(f"systemd lifecycle validation mode: {report['mode']}")
    print(f"unit: {report['unit']}")
    print(f"template present: {str(report['template_present']).lower()}")
    print(f"service actions performed: {str(report['service_actions_performed']).lower()}")
    print(f"secrets loaded: {str(report['secrets_loaded']).lower()}")
    print(f"live execution enabled: {str(report['live_execution_enabled']).lower()}")
    print(f"production readiness claimed: {str(report['production_readiness_claimed']).lower()}")

    if report["mode"] == "plan":
        print("operator steps:")
        for step in report["operator_steps"]:
            print(f"- {step}")
    else:
        show = report["systemctl_show"]
        print(f"systemctl show returncode: {show['returncode']}")
        print(f"systemctl output lines: {show['raw_output_line_count']}")
        for key, value in sorted(show["properties"].items()):
            print(f"{key}: {value}")

    print("remaining external evidence:")
    for item in report["remaining_external_evidence"]:
        print(f"- {item}")
    print("systemd lifecycle validation report generated")


def main() -> int:
    args = parse_args()
    try:
        validate_unit_name(args.unit)
        report = build_plan_report(args.unit) if args.mode == "plan" else build_inspect_report(args.unit)
    except (OSError, RuntimeError, ValueError) as error:
        return fail(str(error))

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text_report(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
