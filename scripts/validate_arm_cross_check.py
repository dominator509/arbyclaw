#!/usr/bin/env python3
"""Run the non-secret ARM cross-target workspace check.

This gate verifies that the Rust target is installed and that Cargo can type
check the workspace for `aarch64-unknown-linux-gnu`. It does not run ARM
binaries, inspect devices, start services, call networks, load secrets, or claim
production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import shutil
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET = "aarch64-unknown-linux-gnu"
TARGET_CC = "aarch64-linux-gnu-gcc"
RUSTUP_LIST_TIMEOUT_SECONDS = 30
RUSTUP_INSTALL_TIMEOUT_SECONDS = 300
CARGO_CHECK_TIMEOUT_SECONDS = 600


def run(command: list[str], *, timeout: int) -> subprocess.CompletedProcess[str]:
    print(f"+ {' '.join(command)}", flush=True)
    try:
        return subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            encoding="utf-8",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        return subprocess.CompletedProcess(
            command,
            124,
            stdout=f"command timed out after {timeout}s: {' '.join(command)}\n",
        )


def target_installed() -> bool:
    completed = run(["rustup", "target", "list", "--installed"], timeout=RUSTUP_LIST_TIMEOUT_SECONDS)
    if completed.returncode != 0:
        raise RuntimeError(completed.stdout)
    return TARGET in completed.stdout.splitlines()


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    target_was_installed = target_installed()
    target_install_attempted = False
    target_install_succeeded = target_was_installed
    target_install_returncode: int | None = None
    target_install_output_tail: list[str] = []
    if not target_was_installed and args.install_target:
        target_install_attempted = True
        install = run(["rustup", "target", "add", TARGET], timeout=RUSTUP_INSTALL_TIMEOUT_SECONDS)
        target_install_returncode = install.returncode
        target_install_output_tail = install.stdout.splitlines()[-20:]
        target_install_succeeded = install.returncode == 0 and target_installed()
    cross_compiler_path = shutil.which(TARGET_CC)
    check_attempted = target_install_succeeded and cross_compiler_path is not None
    check_returncode: int | None = None
    check_output_tail: list[str] = []
    if check_attempted:
        check = run(
            ["cargo", "check", "--workspace", "--target", TARGET, "--locked"],
            timeout=CARGO_CHECK_TIMEOUT_SECONDS,
        )
        check_returncode = check.returncode
        check_output_tail = check.stdout.splitlines()[-20:]

    failures: list[str] = []
    if not target_install_succeeded:
        failures.append(f"Rust target is not installed: {TARGET}")
    if cross_compiler_path is None:
        failures.append(f"cross compiler is unavailable: {TARGET_CC}")
    if check_attempted and check_returncode != 0:
        failures.append("cargo ARM cross-target check failed")

    return {
        "schema": "arbyclaw.arm_cross_check.v1",
        "target": TARGET,
        "target_installed": target_install_succeeded,
        "target_install_attempted": target_install_attempted,
        "target_install_returncode": target_install_returncode,
        "target_install_output_tail": target_install_output_tail,
        "cross_compiler": TARGET_CC,
        "cross_compiler_available": cross_compiler_path is not None,
        "cross_compiler_path": cross_compiler_path,
        "cargo_check_attempted": check_attempted,
        "cargo_check_returncode": check_returncode,
        "cargo_check_output_tail": check_output_tail,
        "bounded_timeouts": {
            "rustup_list_seconds": RUSTUP_LIST_TIMEOUT_SECONDS,
            "rustup_install_seconds": RUSTUP_INSTALL_TIMEOUT_SECONDS,
            "cargo_check_seconds": CARGO_CHECK_TIMEOUT_SECONDS,
        },
        "passed": not failures,
        "failures": failures,
        "arm_binary_executed": False,
        "device_inspected": False,
        "emulator_used": False,
        "service_actions_performed": False,
        "external_calls_performed": target_install_attempted,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }


def print_text(report: dict[str, Any]) -> None:
    print("ARM cross-target check")
    print(f"passed: {str(report['passed']).lower()}")
    print(f"target: {report['target']}")
    print(f"target-installed: {str(report['target_installed']).lower()}")
    print(f"target-install-attempted: {str(report['target_install_attempted']).lower()}")
    print(f"target-install-returncode: {report['target_install_returncode']}")
    print(f"cross-compiler: {report['cross_compiler']}")
    print(f"cross-compiler-available: {str(report['cross_compiler_available']).lower()}")
    print(f"cargo-check-attempted: {str(report['cargo_check_attempted']).lower()}")
    print(f"cargo-check-returncode: {report['cargo_check_returncode']}")
    print("bounded-timeouts-enabled: true")
    print("arm-binary-executed: false")
    print("device-inspected: false")
    print("emulator-used: false")
    print("service-actions-performed: false")
    print(f"external-calls-performed: {str(report['external_calls_performed']).lower()}")
    print("secrets-loaded: false")
    print("production-readiness-claimed: false")
    for failure in report["failures"]:
        print(f"failure: {failure}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON report")
    parser.add_argument(
        "--install-target",
        action="store_true",
        help="install the Rust ARM target before checking if it is missing",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text(report)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
