#!/usr/bin/env python3
"""Validate non-secret ARM build profile documentation.

This static gate checks that the committed ARM deployment notes name the
expected targets, include reproducible future commands, and explicitly avoid
claiming that an ARM build or deployment has been executed. It does not install
Rust targets, cross-compile, run emulators, inspect devices, call networks, or
claim production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
PROFILE = ROOT / "deployment/arm/BUILD_PROFILES.md"
REQUIRED_TARGETS = [
    "aarch64-unknown-linux-gnu",
    "armv7-unknown-linux-gnueabihf",
]
REQUIRED_COMMANDS = [
    "rustup target add aarch64-unknown-linux-gnu",
    "cargo build --target aarch64-unknown-linux-gnu --release --locked",
    "cargo test --workspace --target aarch64-unknown-linux-gnu",
]
REQUIRED_EXTERNAL_TERMS = [
    "Cross-linker",
    "libc",
    "CPU feature",
    "clock",
    "filesystem durability",
    "service-manager behavior",
    "actual target class",
]
SECRET_ASSIGNMENT = re.compile(
    r"(?i)(api[_-]?key|secret|private[_-]?key|seed[_-]?phrase|mnemonic|token)\s*[:=]\s*['\"]?[A-Za-z0-9_/+=.-]{12,}"
)


def build_report() -> dict[str, Any]:
    if not PROFILE.exists():
        raise RuntimeError("missing deployment/arm/BUILD_PROFILES.md")

    text = PROFILE.read_text(encoding="utf-8")
    lower = text.lower()
    missing_targets = [target for target in REQUIRED_TARGETS if target not in text]
    missing_commands = [command for command in REQUIRED_COMMANDS if command not in text]
    missing_external_terms = [term for term in REQUIRED_EXTERNAL_TERMS if term.lower() not in lower]
    no_execution_claim = "no arm build was executed" in lower
    production_claim_denied = "before any production claim" in lower
    secret_like_assignment = SECRET_ASSIGNMENT.search(text) is not None
    failures = []
    if missing_targets:
        failures.append(f"missing ARM targets: {', '.join(missing_targets)}")
    if missing_commands:
        failures.append(f"missing ARM commands: {', '.join(missing_commands)}")
    if missing_external_terms:
        failures.append(f"missing external validation terms: {', '.join(missing_external_terms)}")
    if not no_execution_claim:
        failures.append("ARM profile does not deny local ARM build execution")
    if not production_claim_denied:
        failures.append("ARM profile does not deny production claims before target validation")
    if secret_like_assignment:
        failures.append("ARM profile contains secret-like assignment")

    return {
        "schema": "arbyclaw.arm_build_profile_validation.v1",
        "profile": str(PROFILE.relative_to(ROOT)),
        "targets": REQUIRED_TARGETS,
        "missing_targets": missing_targets,
        "missing_commands": missing_commands,
        "missing_external_terms": missing_external_terms,
        "no_execution_claim": no_execution_claim,
        "production_claim_denied": production_claim_denied,
        "secret_like_assignment": secret_like_assignment,
        "passed": not failures,
        "failures": failures,
        "targets_installed": False,
        "cross_build_performed": False,
        "emulator_used": False,
        "device_inspected": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }


def print_text(report: dict[str, Any]) -> None:
    print("ARM build profile validation")
    print(f"passed: {str(report['passed']).lower()}")
    print(f"profile: {report['profile']}")
    print(f"target-count: {len(report['targets'])}")
    print(f"no-execution-claim: {str(report['no_execution_claim']).lower()}")
    print(f"production-claim-denied: {str(report['production_claim_denied']).lower()}")
    print(f"targets-installed: {str(report['targets_installed']).lower()}")
    print(f"cross-build-performed: {str(report['cross_build_performed']).lower()}")
    print(f"emulator-used: {str(report['emulator_used']).lower()}")
    print(f"device-inspected: {str(report['device_inspected']).lower()}")
    print(f"external-calls-performed: {str(report['external_calls_performed']).lower()}")
    print(f"secrets-loaded: {str(report['secrets_loaded']).lower()}")
    print(f"production-readiness-claimed: {str(report['production_readiness_claimed']).lower()}")
    for failure in report["failures"]:
        print(f"failure: {failure}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON report")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report()
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text(report)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
