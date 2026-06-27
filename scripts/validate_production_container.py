#!/usr/bin/env python3
"""Validate the ArbyClaw production-intent container image locally.

This script builds the committed production-intent image, runs Trivy
vulnerability checks, and smoke-runs the inert help path. It does not push
images, install services, open listeners, load secrets, call exchanges/RPCs, or
claim production readiness.
"""

from __future__ import annotations

import argparse
import json
import subprocess


IMAGE_TAG = "arbyclaw-production:local-validation"
CONTAINERFILE = "deployment/container/Containerfile.production"
TRIVY_IMAGE = "aquasec/trivy:latest"
DOCKER_PROBE_TIMEOUT_SECONDS = 20
DOCKER_BUILD_TIMEOUT_SECONDS = 600
TRIVY_TIMEOUT_SECONDS = 600
SMOKE_TIMEOUT_SECONDS = 60


def run(
    command: list[str], *, capture: bool = False, timeout: int, verbose: bool = True
) -> subprocess.CompletedProcess[str]:
    if verbose:
        print(f"+ {' '.join(command)}", flush=True)
    try:
        return subprocess.run(
            command,
            check=True,
            encoding="utf-8",
            stdout=subprocess.PIPE if capture else None,
            stderr=subprocess.STDOUT if capture else None,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(
            f"command timed out after {timeout}s: {' '.join(command)}"
        ) from error
    except subprocess.CalledProcessError as error:
        raise RuntimeError(f"command failed: {' '.join(command)}") from error


def print_non_claims() -> None:
    print("deployment-performed: false")
    print("service-installed: false")
    print("network-listeners-started: false")
    print("secrets-loaded: false")
    print("live-execution-enabled: false")
    print("production-readiness-claimed: false")


def non_claims() -> dict[str, bool]:
    return {
        "deployment_performed": False,
        "service_installed": False,
        "network_listeners_started": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
    }


def build_report(json_mode: bool) -> dict[str, object]:
    report: dict[str, object] = {
        "schema": "arbyclaw.production_container_validation.v1",
        "image": IMAGE_TAG,
        "containerfile": CONTAINERFILE,
        "docker_validation_completed": False,
        "hardened_runtime_smoke_passed": False,
        "read_only_filesystem": False,
        "network_disabled": False,
        "capabilities_dropped": False,
        "no_new_privileges": False,
        "failures": [],
        "bounded_timeouts": {
            "docker_probe_seconds": DOCKER_PROBE_TIMEOUT_SECONDS,
            "docker_build_seconds": DOCKER_BUILD_TIMEOUT_SECONDS,
            "trivy_seconds": TRIVY_TIMEOUT_SECONDS,
            "smoke_seconds": SMOKE_TIMEOUT_SECONDS,
        },
        **non_claims(),
    }
    try:
        run(["docker", "version"], timeout=DOCKER_PROBE_TIMEOUT_SECONDS, verbose=not json_mode)
        run(
            ["docker", "build", "-f", CONTAINERFILE, "-t", IMAGE_TAG, "."],
            timeout=DOCKER_BUILD_TIMEOUT_SECONDS,
            verbose=not json_mode,
        )
        run(
            [
                "docker",
                "run",
                "--rm",
                "-v",
                "/var/run/docker.sock:/var/run/docker.sock",
                TRIVY_IMAGE,
                "image",
                "--severity",
                "HIGH,CRITICAL",
                "--ignore-unfixed",
                "--format",
                "table",
                IMAGE_TAG,
            ],
            timeout=TRIVY_TIMEOUT_SECONDS,
            verbose=not json_mode,
        )
        run(
            [
                "docker",
                "run",
                "--rm",
                "-v",
                "/var/run/docker.sock:/var/run/docker.sock",
                TRIVY_IMAGE,
                "image",
                "--severity",
                "CRITICAL",
                "--ignore-unfixed",
                "--exit-code",
                "1",
                "--scanners",
                "vuln",
                IMAGE_TAG,
            ],
            timeout=TRIVY_TIMEOUT_SECONDS,
            verbose=not json_mode,
        )
        smoke = run(
            ["docker", "run", "--rm", IMAGE_TAG],
            capture=True,
            timeout=SMOKE_TIMEOUT_SECONDS,
            verbose=not json_mode,
        )
        if not json_mode:
            print(smoke.stdout, end="")
        if "usage: arb-agent [--config <path>]" not in smoke.stdout:
            raise RuntimeError("production container smoke output did not include arb-agent usage")
        hardened_smoke = run(
            [
                "docker",
                "run",
                "--rm",
                "--read-only",
                "--network",
                "none",
                "--cap-drop",
                "ALL",
                "--security-opt",
                "no-new-privileges",
                IMAGE_TAG,
            ],
            capture=True,
            timeout=SMOKE_TIMEOUT_SECONDS,
            verbose=not json_mode,
        )
        if not json_mode:
            print(hardened_smoke.stdout, end="")
        if "usage: arb-agent [--config <path>]" not in hardened_smoke.stdout:
            raise RuntimeError(
                "hardened production container smoke output did not include arb-agent usage"
            )
    except RuntimeError as error:
        report["failures"] = [str(error)]
        report["passed"] = False
        return report

    report["docker_validation_completed"] = True
    report["hardened_runtime_smoke_passed"] = True
    report["read_only_filesystem"] = True
    report["network_disabled"] = True
    report["capabilities_dropped"] = True
    report["no_new_privileges"] = True
    report["passed"] = True
    return report


def print_text(report: dict[str, object]) -> None:
    if report["passed"]:
        print("production container validation passed")
    else:
        print(f"production container validation failed: {report['failures'][0]}")
    print(f"hardened-runtime-smoke-passed: {str(report['hardened_runtime_smoke_passed']).lower()}")
    print(f"read-only-filesystem: {str(report['read_only_filesystem']).lower()}")
    print(f"network-disabled: {str(report['network_disabled']).lower()}")
    print(f"capabilities-dropped: {str(report['capabilities_dropped']).lower()}")
    print(f"no-new-privileges: {str(report['no_new_privileges']).lower()}")
    print(f"docker-validation-completed: {str(report['docker_validation_completed']).lower()}")
    print_non_claims()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON report")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args.json)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text(report)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
