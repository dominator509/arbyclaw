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
import os
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
DOCKER_PROBE_TIMEOUT_SECONDS = 20
DOCKER_CROSS_CHECK_TIMEOUT_SECONDS = 1_800
DOCKER_IMAGE = "rust:1.90"
DOCKER_WORKDIR = "/workspace"
DOCKER_RUST_PATH = (
    "/usr/local/cargo/bin:/usr/local/rustup/bin:"
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
)


def run(
    command: list[str],
    *,
    timeout: int,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
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
            env=env,
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


def docker_cross_check() -> subprocess.CompletedProcess[str]:
    mount_spec = f"{ROOT.as_posix()}:{DOCKER_WORKDIR}"
    script = "\n".join(
        [
            f"export PATH={DOCKER_RUST_PATH}",
            "apt-get update >/tmp/apt.log 2>&1",
            "rc1=$?",
            "apt-get install -y gcc-aarch64-linux-gnu pkg-config >/tmp/install.log 2>&1",
            "rc2=$?",
            f"rustup target add {TARGET} >/tmp/rustup.log 2>&1",
            "rc3=$?",
            f"cargo check --workspace --target {TARGET} --locked >/tmp/check.log 2>&1",
            "rc4=$?",
            'echo "STEP_RC apt_update=$rc1 install=$rc2 rustup=$rc3 cargo=$rc4"',
            "for f in /tmp/apt.log /tmp/install.log /tmp/rustup.log /tmp/check.log; do",
            '  echo "==== $f ===="',
            '  test -f "$f" && tail -n 80 "$f" || echo "missing"',
            "done",
            "exit $rc4",
        ]
    )
    return run(
        [
            "docker",
            "run",
            "--rm",
            "-v",
            mount_spec,
            "-w",
            DOCKER_WORKDIR,
            DOCKER_IMAGE,
            "bash",
            "-lc",
            script,
        ],
        timeout=DOCKER_CROSS_CHECK_TIMEOUT_SECONDS,
    )


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    host_target_was_installed = target_installed()
    target_install_attempted = False
    host_target_install_succeeded = host_target_was_installed
    target_install_returncode: int | None = None
    target_install_output_tail: list[str] = []
    if not host_target_was_installed and args.install_target:
        target_install_attempted = True
        install = run(["rustup", "target", "add", TARGET], timeout=RUSTUP_INSTALL_TIMEOUT_SECONDS)
        target_install_returncode = install.returncode
        target_install_output_tail = install.stdout.splitlines()[-20:]
        host_target_install_succeeded = install.returncode == 0 and target_installed()
    host_cross_compiler_path = shutil.which(TARGET_CC)
    host_check_attempted = (
        host_target_install_succeeded and host_cross_compiler_path is not None
    )
    check_returncode: int | None = None
    check_output_tail: list[str] = []
    cargo_check_environment = "none"
    docker_probe_returncode: int | None = None
    docker_probe_output_tail: list[str] = []
    docker_available = False
    docker_fallback_used = False
    docker_cross_check_attempted = False
    docker_cross_check_returncode: int | None = None
    docker_cross_check_output_tail: list[str] = []
    effective_target_installed = host_target_install_succeeded
    effective_cross_compiler_available = host_cross_compiler_path is not None
    effective_cross_compiler_path = host_cross_compiler_path
    external_calls_performed = target_install_attempted
    if host_check_attempted:
        cargo_env = os.environ.copy()
        cargo_env["CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER"] = TARGET_CC
        cargo_env["CC_aarch64_unknown_linux_gnu"] = TARGET_CC
        cargo_env["PKG_CONFIG_ALLOW_CROSS"] = "1"
        check = run(
            ["cargo", "check", "--workspace", "--target", TARGET, "--locked"],
            timeout=CARGO_CHECK_TIMEOUT_SECONDS,
            env=cargo_env,
        )
        check_returncode = check.returncode
        check_output_tail = check.stdout.splitlines()[-20:]
        cargo_check_environment = "host"
    else:
        docker_probe = run(["docker", "version"], timeout=DOCKER_PROBE_TIMEOUT_SECONDS)
        docker_probe_returncode = docker_probe.returncode
        docker_probe_output_tail = docker_probe.stdout.splitlines()[-20:]
        docker_available = docker_probe.returncode == 0
        if docker_available:
            docker_fallback_used = True
            docker_cross_check_attempted = True
            external_calls_performed = True
            docker_check = docker_cross_check()
            docker_cross_check_returncode = docker_check.returncode
            docker_cross_check_output_tail = docker_check.stdout.splitlines()[-40:]
            check_returncode = docker_check.returncode
            check_output_tail = docker_cross_check_output_tail
            cargo_check_environment = "docker"
            if docker_check.returncode == 0:
                effective_target_installed = True
                effective_cross_compiler_available = True
                effective_cross_compiler_path = TARGET_CC

    failures: list[str] = []
    if check_returncode != 0:
        if not host_target_install_succeeded:
            failures.append(f"Rust target is not installed on the host: {TARGET}")
        if host_cross_compiler_path is None:
            failures.append(f"host cross compiler is unavailable: {TARGET_CC}")
        if docker_cross_check_attempted:
            failures.append("docker ARM cross-target check failed")
        elif not docker_available:
            failures.append("docker fallback is unavailable")
    if host_check_attempted and check_returncode != 0:
        failures.append("cargo ARM cross-target check failed")

    return {
        "schema": "arbyclaw.arm_cross_check.v1",
        "target": TARGET,
        "target_installed": effective_target_installed,
        "host_target_installed": host_target_install_succeeded,
        "target_install_attempted": target_install_attempted,
        "target_install_returncode": target_install_returncode,
        "target_install_output_tail": target_install_output_tail,
        "cross_compiler": TARGET_CC,
        "cross_compiler_available": effective_cross_compiler_available,
        "cross_compiler_path": effective_cross_compiler_path,
        "host_cross_compiler_available": host_cross_compiler_path is not None,
        "host_cross_compiler_path": host_cross_compiler_path,
        "cargo_check_attempted": check_returncode is not None,
        "cargo_check_environment": cargo_check_environment,
        "cargo_check_returncode": check_returncode,
        "cargo_check_output_tail": check_output_tail,
        "docker_available": docker_available,
        "docker_fallback_used": docker_fallback_used,
        "docker_image": DOCKER_IMAGE,
        "docker_probe_returncode": docker_probe_returncode,
        "docker_probe_output_tail": docker_probe_output_tail,
        "docker_cross_check_attempted": docker_cross_check_attempted,
        "docker_cross_check_returncode": docker_cross_check_returncode,
        "docker_cross_check_output_tail": docker_cross_check_output_tail,
        "bounded_timeouts": {
            "rustup_list_seconds": RUSTUP_LIST_TIMEOUT_SECONDS,
            "rustup_install_seconds": RUSTUP_INSTALL_TIMEOUT_SECONDS,
            "cargo_check_seconds": CARGO_CHECK_TIMEOUT_SECONDS,
            "docker_probe_seconds": DOCKER_PROBE_TIMEOUT_SECONDS,
            "docker_cross_check_seconds": DOCKER_CROSS_CHECK_TIMEOUT_SECONDS,
        },
        "passed": not failures,
        "failures": failures,
        "arm_binary_executed": False,
        "device_inspected": False,
        "emulator_used": False,
        "service_actions_performed": False,
        "external_calls_performed": external_calls_performed,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }


def print_text(report: dict[str, Any]) -> None:
    print("ARM cross-target check")
    print(f"passed: {str(report['passed']).lower()}")
    print(f"target: {report['target']}")
    print(f"target-installed: {str(report['target_installed']).lower()}")
    print(f"host-target-installed: {str(report['host_target_installed']).lower()}")
    print(f"target-install-attempted: {str(report['target_install_attempted']).lower()}")
    print(f"target-install-returncode: {report['target_install_returncode']}")
    print(f"cross-compiler: {report['cross_compiler']}")
    print(f"cross-compiler-available: {str(report['cross_compiler_available']).lower()}")
    print(
        f"host-cross-compiler-available: {str(report['host_cross_compiler_available']).lower()}"
    )
    print(f"cargo-check-attempted: {str(report['cargo_check_attempted']).lower()}")
    print(f"cargo-check-environment: {report['cargo_check_environment']}")
    print(f"cargo-check-returncode: {report['cargo_check_returncode']}")
    print(f"docker-available: {str(report['docker_available']).lower()}")
    print(f"docker-fallback-used: {str(report['docker_fallback_used']).lower()}")
    print(
        f"docker-cross-check-attempted: {str(report['docker_cross_check_attempted']).lower()}"
    )
    print(f"docker-cross-check-returncode: {report['docker_cross_check_returncode']}")
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
