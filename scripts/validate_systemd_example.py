#!/usr/bin/env python3
"""Validate the example ArbyClaw systemd unit without installing or starting it.

This script checks the committed example unit for non-secret, fail-closed
service-manager posture. When requested with `--systemd-analyze`, it also runs
a syntax-only `systemd-analyze verify` pass. It never copies the unit into a
system directory, reloads systemd, enables a service, starts a service, loads
secrets, or claims production deployment readiness.
"""

from __future__ import annotations

import argparse
import configparser
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
UNIT_PATH = ROOT / "deployment/systemd/arb-agent.service.example"
SYSTEMD_ANALYZE_TIMEOUT_SECONDS = 60

FORBIDDEN_DIRECTIVES = {
    "Environment",
    "EnvironmentFile",
    "PassEnvironment",
    "LoadCredential",
    "LoadCredentialEncrypted",
    "SetCredential",
    "SetCredentialEncrypted",
}

REQUIRED_SERVICE_VALUES = {
    "Type": "simple",
    "NoNewPrivileges": "true",
    "PrivateTmp": "true",
    "ProtectSystem": "strict",
    "ProtectHome": "true",
    "LockPersonality": "true",
    "MemoryDenyWriteExecute": "true",
    "RestrictRealtime": "true",
    "SystemCallArchitectures": "native",
}

REQUIRED_EMPTY_VALUES = {
    "CapabilityBoundingSet",
    "AmbientCapabilities",
}


def fail(message: str) -> int:
    print(f"systemd example validation failed: {message}", file=sys.stderr)
    return 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON report")
    parser.add_argument(
        "--systemd-analyze",
        action="store_true",
        help="also run syntax-only systemd-analyze verify when available",
    )
    parser.add_argument(
        "--require-systemd-analyze",
        action="store_true",
        help="fail if --systemd-analyze is requested but systemd-analyze is unavailable",
    )
    return parser.parse_args()


def read_unit() -> configparser.ConfigParser:
    parser = configparser.ConfigParser(
        interpolation=None,
        strict=False,
        delimiters=("=",),
        comment_prefixes=("#",),
        inline_comment_prefixes=(),
    )
    parser.optionxform = str
    with UNIT_PATH.open(encoding="utf-8") as handle:
        parser.read_file(handle)
    return parser


def validate_static_unit(parser: configparser.ConfigParser) -> list[str]:
    for section in ("Unit", "Service", "Install"):
        if not parser.has_section(section):
            raise ValueError(f"missing [{section}] section")

    service = parser["Service"]
    exec_start = service.get("ExecStart", "")
    if not exec_start.startswith("/usr/local/bin/arb-agent --config "):
        raise ValueError("ExecStart must run arb-agent with an explicit config path")
    if any(forbidden in exec_start for forbidden in (" live", "withdraw", "bridge", "sign", "broadcast")):
        raise ValueError("ExecStart must not contain live, withdraw, bridge, sign, or broadcast commands")

    for directive in FORBIDDEN_DIRECTIVES:
        if directive in service:
            raise ValueError(f"{directive} must not appear in the example unit")

    for key, expected in REQUIRED_SERVICE_VALUES.items():
        actual = service.get(key)
        if actual != expected:
            raise ValueError(f"Service.{key} must be {expected!r}, got {actual!r}")

    for key in REQUIRED_EMPTY_VALUES:
        if key not in service:
            raise ValueError(f"Service.{key} must be present and empty")
        if service.get(key, "") != "":
            raise ValueError(f"Service.{key} must be empty")

    read_write_paths = service.get("ReadWritePaths", "")
    for required_path in ("/var/lib/arb-agent", "/var/log/arb-agent"):
        if required_path not in read_write_paths.split():
            raise ValueError(f"ReadWritePaths must include {required_path}")

    if parser["Install"].get("WantedBy") != "multi-user.target":
        raise ValueError("Install.WantedBy must be multi-user.target")

    warnings = [
        "static systemd template checks only; service was not installed, enabled, reloaded, or started",
        "deployment-host service-manager restart execution evidence remains external",
    ]
    return warnings


def write_placeholder_executable(path: pathlib.Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    path.chmod(0o755)


def write_placeholder_target_units(root: pathlib.Path) -> None:
    target_dir = root / "usr/lib/systemd/system"
    target_dir.mkdir(parents=True, exist_ok=True)
    for target in ("sysinit.target", "basic.target", "network-online.target", "multi-user.target"):
        (target_dir / target).write_text(
            "[Unit]\n"
            f"Description=Temporary {target} for ArbyClaw syntax verification\n",
            encoding="utf-8",
        )


def run_systemd_analyze_if_requested(enabled: bool, required: bool) -> str:
    if not enabled:
        return "systemd-analyze verify not requested; skipped syntax verify"

    binary = shutil.which("systemd-analyze")
    if binary is None:
        if required:
            raise RuntimeError("systemd-analyze unavailable")
        return "systemd-analyze unavailable; skipped syntax verify"

    with tempfile.TemporaryDirectory(prefix="arbyclaw-systemd-verify-") as temp_dir:
        verify_root = pathlib.Path(temp_dir)
        verify_path = verify_root / "etc/systemd/system/arb-agent.service"
        verify_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(UNIT_PATH, verify_path)
        write_placeholder_target_units(verify_root)
        write_placeholder_executable(verify_root / "usr/local/bin/arb-agent")
        (verify_root / "etc/arb-agent").mkdir(parents=True, exist_ok=True)
        (verify_root / "var/lib/arb-agent").mkdir(parents=True, exist_ok=True)
        (verify_root / "var/log/arb-agent").mkdir(parents=True, exist_ok=True)
        command = [
            binary,
            f"--root={verify_root}",
            "verify",
            "/etc/systemd/system/arb-agent.service",
        ]
        print(f"+ {' '.join(command)}", flush=True)
        try:
            completed = subprocess.run(
                command,
                check=False,
                encoding="utf-8",
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                timeout=SYSTEMD_ANALYZE_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            raise RuntimeError(
                f"systemd-analyze verify timed out after {SYSTEMD_ANALYZE_TIMEOUT_SECONDS}s"
            ) from error
    if completed.stdout:
        print(completed.stdout, end="")
    if completed.returncode != 0:
        raise RuntimeError("systemd-analyze verify failed")
    return "systemd-analyze verify passed"


def build_report(args: argparse.Namespace) -> dict[str, object]:
    parser = read_unit()
    warnings = validate_static_unit(parser)
    systemd_result = run_systemd_analyze_if_requested(
        args.systemd_analyze,
        args.require_systemd_analyze,
    )
    systemd_analyze_requested = args.systemd_analyze
    systemd_analyze_verified = systemd_result == "systemd-analyze verify passed"
    systemd_analyze_available = "unavailable" not in systemd_result
    return {
        "schema": "arbyclaw.systemd_example_validation.v1",
        "passed": True,
        "static_validation_passed": True,
        "warnings": warnings,
        "systemd_analyze_requested": systemd_analyze_requested,
        "systemd_analyze_required": args.require_systemd_analyze,
        "systemd_analyze_available": systemd_analyze_available,
        "systemd_analyze_verified": systemd_analyze_verified,
        "systemd_analyze_result": systemd_result,
        "service_actions_performed": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }


def main() -> int:
    args = parse_args()
    if not UNIT_PATH.exists():
        return fail(f"missing unit template: {UNIT_PATH.relative_to(ROOT)}")

    try:
        report = build_report(args)
    except (configparser.Error, OSError, RuntimeError, ValueError) as error:
        return fail(str(error))

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0

    for warning in report["warnings"]:
        print(f"warning: {warning}")
    print(report["systemd_analyze_result"])
    print("systemd example validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
