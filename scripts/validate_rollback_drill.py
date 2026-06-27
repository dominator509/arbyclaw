#!/usr/bin/env python3
"""Create a non-secret rollback-drill evidence plan without changing services.

This helper validates rollback metadata and emits a sanitized report. It never
installs units, reloads systemd, enables services, starts services, stops
services, restarts services, changes files, calls networks, loads secrets, or
claims production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
SECRET_LIKE = re.compile(
    r"(?i)(api[_-]?key|secret|private[_-]?key|seed[_-]?phrase|mnemonic|token|bearer|password)"
)
REFERENCE_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:/@#?=&+\-]{1,240}$")

ROLLBACK_STEPS = [
    "confirm live trading, signing, withdrawals, bridges, broadcasts, and external submission remain disabled",
    "record candidate commit, rollback target, artifact names, and non-secret evidence locators",
    "operator-controlled service quiesce step outside this script",
    "operator-controlled artifact/config restore step outside this script",
    "run structure, Rust validation, systemd lifecycle inspection, and runtime smoke validation after restore",
    "verify audit and SQLite recovery evidence references remain available",
    "record reviewer, timestamp, outcome, unresolved gaps, and decision",
]

REMAINING_EXTERNAL_EVIDENCE = [
    "operator-controlled service stop/start/restart evidence",
    "actual artifact restore evidence",
    "post-rollback runtime smoke evidence on the target host",
    "post-rollback audit/SQLite recovery evidence",
    "human release-review decision",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-ref", help="non-secret candidate commit, tag, or run reference")
    parser.add_argument("--rollback-ref", help="non-secret rollback commit, tag, or artifact reference")
    parser.add_argument("--artifact", action="append", default=[], help="non-secret artifact name or locator")
    parser.add_argument("--run-url", action="append", default=[], help="non-secret CI or validation run URL")
    parser.add_argument("--reviewer", help="reviewer name, role, or handle; no secrets")
    parser.add_argument(
        "--outcome",
        choices=("planned", "ready-for-manual-drill", "executed-externally", "deferred"),
        default="planned",
        help="non-secret rollback review outcome",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="require candidate, rollback target, reviewer, and at least one artifact or run URL",
    )
    parser.add_argument("--json", action="store_true", help="emit JSON instead of text")
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"rollback drill validation failed: {message}", file=sys.stderr)
    return 1


def validate_reference(label: str, value: str | None) -> None:
    if value is None:
        return
    if SECRET_LIKE.search(value):
        raise ValueError(f"{label} looks secret-like")
    if not REFERENCE_PATTERN.fullmatch(value):
        raise ValueError(f"{label} must be a short non-secret reference")


def validate_reference_list(label: str, values: list[str]) -> None:
    for value in values:
        validate_reference(label, value)


def validate_args(args: argparse.Namespace) -> None:
    validate_reference("candidate-ref", args.candidate_ref)
    validate_reference("rollback-ref", args.rollback_ref)
    validate_reference("reviewer", args.reviewer)
    validate_reference_list("artifact", args.artifact)
    validate_reference_list("run-url", args.run_url)

    if args.strict:
        missing = []
        if not args.candidate_ref:
            missing.append("--candidate-ref")
        if not args.rollback_ref:
            missing.append("--rollback-ref")
        if not args.reviewer:
            missing.append("--reviewer")
        if not args.artifact and not args.run_url:
            missing.append("--artifact or --run-url")
        if missing:
            raise ValueError(f"strict mode missing required fields: {', '.join(missing)}")


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    validate_args(args)
    metadata_complete = bool(
        args.candidate_ref and args.rollback_ref and args.reviewer and (args.artifact or args.run_url)
    )
    return {
        "schema": "arbyclaw.rollback_drill_validation.v1",
        "candidate_ref": args.candidate_ref,
        "rollback_ref": args.rollback_ref,
        "artifacts": args.artifact,
        "run_urls": args.run_url,
        "reviewer": args.reviewer,
        "outcome": args.outcome,
        "metadata_complete": metadata_complete,
        "rollback_steps": ROLLBACK_STEPS,
        "service_actions_performed": False,
        "files_changed": False,
        "secrets_loaded": False,
        "external_calls_performed": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
        "remaining_external_evidence": REMAINING_EXTERNAL_EVIDENCE,
    }


def print_text_report(report: dict[str, Any]) -> None:
    print("rollback drill validation report")
    print(f"candidate ref: {report['candidate_ref'] or 'not provided'}")
    print(f"rollback ref: {report['rollback_ref'] or 'not provided'}")
    print(f"reviewer: {report['reviewer'] or 'not provided'}")
    print(f"outcome: {report['outcome']}")
    print(f"metadata complete: {str(report['metadata_complete']).lower()}")
    print(f"service actions performed: {str(report['service_actions_performed']).lower()}")
    print(f"files changed: {str(report['files_changed']).lower()}")
    print(f"secrets loaded: {str(report['secrets_loaded']).lower()}")
    print(f"external calls performed: {str(report['external_calls_performed']).lower()}")
    print(f"live execution enabled: {str(report['live_execution_enabled']).lower()}")
    print(f"production readiness claimed: {str(report['production_readiness_claimed']).lower()}")
    print("rollback steps:")
    for step in report["rollback_steps"]:
        print(f"- {step}")
    print("remaining external evidence:")
    for item in report["remaining_external_evidence"]:
        print(f"- {item}")


def main() -> int:
    args = parse_args()
    try:
        report = build_report(args)
    except ValueError as error:
        return fail(str(error))

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text_report(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
