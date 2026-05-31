#!/usr/bin/env python3
"""Validate the example ArbyClaw container image locally.

This script mirrors the non-secret Docker-dependent CI gate for the example
image. It builds the image, runs Trivy vulnerability checks, and smoke-runs the
container help path. It does not push images, start services, open network
listeners, load secrets, or claim production container readiness.
"""

from __future__ import annotations

import subprocess
import sys


IMAGE_TAG = "arbyclaw-example:ci"
CONTAINERFILE = "deployment/container/Containerfile.example"
TRIVY_IMAGE = "aquasec/trivy:latest"


def run(command: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    print(f"+ {' '.join(command)}", flush=True)
    return subprocess.run(
        command,
        check=True,
        encoding="utf-8",
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.STDOUT if capture else None,
        text=True,
    )


def main() -> int:
    run(["docker", "version"])
    run(["docker", "build", "-f", CONTAINERFILE, "-t", IMAGE_TAG, "."])
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
        ]
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
        ]
    )
    smoke = run(["docker", "run", "--rm", IMAGE_TAG], capture=True)
    print(smoke.stdout, end="")
    if "usage: arb-agent [--config <path>]" not in smoke.stdout:
        raise RuntimeError("container smoke output did not include arb-agent usage")

    print("example container validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
