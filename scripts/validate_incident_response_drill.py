#!/usr/bin/env python3
"""Create a non-secret incident-response drill evidence plan.

This helper validates incident-drill metadata and emits a sanitized report. It
never starts services, stops services, changes files, calls networks, loads
secrets, escalates alerts, contacts responders, or claims production readiness.
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

INCIDENT_SCENARIOS = (
    "audit-or-state-recovery",
    "suspected-secret-exposure",
    "service-unhealthy",
    "failed-deployment",
    "policy-denial-review",
)

DRILL_STEPS = [
    "confirm live trading, signing, withdrawals, bridges, broadcasts, and external submission remain disabled",
    "record scenario, severity, responder role, reviewer, run URLs, artifact names, and non-secret evidence locators",
    "operator-controlled detection and triage step outside this script",
    "operator-controlled containment and recovery step outside this script",
    "run structure, Rust validation, runtime smoke, audit replay, and SQLite recovery checks after recovery",
    "record communications and escalation evidence references without copying message contents",
    "record reviewer, timestamp, outcome, unresolved gaps, and decision",
]

REMAINING_EXTERNAL_EVIDENCE = [
    "operator-controlled incident detection evidence",
    "operator-controlled containment and recovery evidence",
    "post-incident runtime smoke evidence on the target host",
    "post-incident audit/SQLite recovery evidence",
    "sanitized communications or escalation evidence references",
    "human incident-review decision",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", choices=INCIDENT_SCENARIOS, help="incident drill scenario")
    parser.add_argument(
        "--severity",
        choices=("low", "medium", "high", "critical"),
        help="sanitized incident severity label",
    )
    parser.add_argument("--responder", help="responder role or handle; no secrets")
    parser.add_argument("--reviewer", help="reviewer role or handle; no secrets")
    parser.add_argument("--artifact", action="append", default=[], help="non-secret artifact name or locator")
    parser.add_argument("--run-url", action="append", default=[], help="non-secret CI or validation run URL")
    parser.add_argument("--evidence", action="append", default=[], help="non-secret external evidence locator")
    parser.add_argument(
        "--outcome",
        choices=("planned", "ready-for-manual-drill", "executed-externally", "deferred"),
        default="planned",
        help="non-secret incident drill review outcome",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="require scenario, severity, responder, reviewer, and at least one evidence reference",
    )
    parser.add_argument("--json", action="store_true", help="emit JSON instead of text")
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"incident response drill validation failed: {message}", file=sys.stderr)
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
    validate_reference("responder", args.responder)
    validate_reference("reviewer", args.reviewer)
    validate_reference_list("artifact", args.artifact)
    validate_reference_list("run-url", args.run_url)
    validate_reference_list("evidence", args.evidence)

    if args.strict:
        missing = []
        if not args.scenario:
            missing.append("--scenario")
        if not args.severity:
            missing.append("--severity")
        if not args.responder:
            missing.append("--responder")
        if not args.reviewer:
            missing.append("--reviewer")
        if not args.artifact and not args.run_url and not args.evidence:
            missing.append("--artifact, --run-url, or --evidence")
        if missing:
            raise ValueError(f"strict mode missing required fields: {', '.join(missing)}")


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    validate_args(args)
    metadata_complete = bool(
        args.scenario
        and args.severity
        and args.responder
        and args.reviewer
        and (args.artifact or args.run_url or args.evidence)
    )
    return {
        "schema": "arbyclaw.incident_response_drill_validation.v1",
        "scenario": args.scenario,
        "severity": args.severity,
        "responder": args.responder,
        "reviewer": args.reviewer,
        "artifacts": args.artifact,
        "run_urls": args.run_url,
        "evidence": args.evidence,
        "outcome": args.outcome,
        "metadata_complete": metadata_complete,
        "drill_steps": DRILL_STEPS,
        "service_actions_performed": False,
        "files_changed": False,
        "secrets_loaded": False,
        "external_calls_performed": False,
        "alerts_sent": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
        "remaining_external_evidence": REMAINING_EXTERNAL_EVIDENCE,
    }


def print_text_report(report: dict[str, Any]) -> None:
    print("incident response drill validation report")
    print(f"scenario: {report['scenario'] or 'not provided'}")
    print(f"severity: {report['severity'] or 'not provided'}")
    print(f"responder: {report['responder'] or 'not provided'}")
    print(f"reviewer: {report['reviewer'] or 'not provided'}")
    print(f"outcome: {report['outcome']}")
    print(f"metadata complete: {str(report['metadata_complete']).lower()}")
    print(f"service actions performed: {str(report['service_actions_performed']).lower()}")
    print(f"files changed: {str(report['files_changed']).lower()}")
    print(f"secrets loaded: {str(report['secrets_loaded']).lower()}")
    print(f"external calls performed: {str(report['external_calls_performed']).lower()}")
    print(f"alerts sent: {str(report['alerts_sent']).lower()}")
    print(f"live execution enabled: {str(report['live_execution_enabled']).lower()}")
    print(f"production readiness claimed: {str(report['production_readiness_claimed']).lower()}")
    print("drill steps:")
    for step in report["drill_steps"]:
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
