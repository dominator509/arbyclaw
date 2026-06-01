#!/usr/bin/env python3
"""Compose non-secret deployment-host runtime validation evidence.

By default this script runs only the non-mutating systemd lifecycle plan helper.
When `--run-runtime-smoke` is provided, it also runs the local
`validate-runtime-smoke` CLI against a caller-supplied fresh workspace. It never
installs units, reloads systemd, enables services, starts services, stops
services, restarts services, loads secrets, calls exchanges/RPCs, or claims
production readiness.
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
SYSTEMD_LIFECYCLE_SCRIPT = ROOT / "scripts/validate_systemd_lifecycle.py"
DEFAULT_CONFIG = ROOT / "config.example.toml"


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


def run_json_command(command: list[str], cwd: pathlib.Path) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stdout.strip() or "command failed")
    try:
        return json.loads(completed.stdout)
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


def validate_runtime_smoke_inputs(config: pathlib.Path, workspace: pathlib.Path | None) -> pathlib.Path:
    if not config.exists():
        raise ValueError(f"config does not exist: {relative_or_absolute(config)}")
    if workspace is None:
        raise ValueError("--runtime-workspace is required with --run-runtime-smoke")
    if workspace.exists():
        raise ValueError(f"runtime workspace must be fresh: {relative_or_absolute(workspace)}")
    return workspace


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


def run_runtime_smoke(
    config: pathlib.Path,
    workspace: pathlib.Path | None,
    agent_bin: pathlib.Path | None,
) -> dict[str, Any]:
    smoke_workspace = validate_runtime_smoke_inputs(config, workspace)
    command = runtime_smoke_command(agent_bin, config, smoke_workspace)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    parsed = parse_key_value_output(completed.stdout)
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "workspace": relative_or_absolute(smoke_workspace),
        "stdout_line_count": len(completed.stdout.splitlines()),
        "production_ready": parsed.get("production-ready"),
        "service_manager_action_performed": parsed.get("service-manager-action-performed"),
        "external_submission_performed": parsed.get("external-submission-performed"),
        "live_execution_performed": parsed.get("live-execution-performed"),
        "runtime_smoke_passed": completed.returncode == 0,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    systemd_report = run_systemd_lifecycle(args.systemd_mode, args.unit)
    runtime_report = None
    if args.run_runtime_smoke:
        runtime_report = run_runtime_smoke(args.config, args.runtime_workspace, args.agent_bin)
        if runtime_report["returncode"] != 0:
            raise RuntimeError("validate-runtime-smoke failed")

    return {
        "schema": "arbyclaw.deployment_host_runtime_validation.v1",
        "systemd_lifecycle": systemd_report,
        "runtime_smoke": runtime_report,
        "runtime_smoke_requested": args.run_runtime_smoke,
        "service_actions_performed": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "external_calls_performed": False,
        "production_readiness_claimed": False,
        "remaining_external_evidence": [
            "operator-controlled service start/shutdown/restart evidence",
            "deployment-host audit and SQLite recovery evidence under service lifecycle",
            "physical disk-full fail-closed evidence",
            "retention/rotation execution evidence",
            "rollback drill evidence",
        ],
    }


def print_text_report(report: dict[str, Any]) -> None:
    systemd = report["systemd_lifecycle"]
    print("deployment-host runtime validation report")
    print(f"systemd mode: {systemd['mode']}")
    print(f"unit: {systemd['unit']}")
    print(f"runtime smoke requested: {str(report['runtime_smoke_requested']).lower()}")
    if report["runtime_smoke"] is not None:
        smoke = report["runtime_smoke"]
        print(f"runtime smoke passed: {str(smoke['runtime_smoke_passed']).lower()}")
        print(f"runtime smoke workspace: {smoke['workspace']}")
        print(f"production-ready: {smoke['production_ready']}")
        print(f"service-manager-action-performed: {smoke['service_manager_action_performed']}")
        print(f"external-submission-performed: {smoke['external_submission_performed']}")
        print(f"live-execution-performed: {smoke['live_execution_performed']}")
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
    except (OSError, RuntimeError, ValueError) as error:
        return fail(str(error))

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text_report(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
