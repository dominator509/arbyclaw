#!/usr/bin/env python3
"""Validate the local deployment evidence checklist without claiming readiness.

The checklist references non-secret evidence locators only. It does not embed
artifact contents, install units, start services, stop services, restart
services, change deployment state, send alerts, call networks, load secrets, or
claim production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]

SCHEMA = "arbyclaw.deployment_evidence_checklist.v1"
BUNDLE_SCHEMA = "arbyclaw.deployment_evidence_bundle.v1"

CATEGORIES: dict[str, str] = {
    "service-lifecycle": "operator-controlled service lifecycle execution evidence",
    "deployment-host-audit-sqlite": "deployment-host audit and SQLite recovery evidence",
    "physical-disk-full": "physical deployment-host disk-full fail-closed evidence",
    "retention-rotation": "deployment-host retention/rotation execution evidence",
    "rollback-drill": "executed rollback drill evidence",
    "incident-response-drill": "executed incident-response drill evidence",
    "production-readiness-review": "human production-readiness review evidence",
}

REFERENCE_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:/@#?=&+\-]{1,240}$")
SECRET_LIKE_PATTERN = re.compile(
    r"(?i)(api[_-]?key|secret|private[_-]?key|seed[_-]?phrase|mnemonic|token|bearer|password)"
)

BOOLEAN_SAFETY_FIELDS = (
    "service_actions_performed",
    "files_changed",
    "secrets_loaded",
    "external_calls_performed",
    "alerts_sent",
    "live_execution_enabled",
    "production_readiness_claimed",
    "artifact_contents_embedded",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON instead of text")
    parser.add_argument(
        "--evidence",
        action="append",
        default=[],
        metavar="CATEGORY=REFERENCE",
        help="record a non-secret evidence locator for one external category",
    )
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"deployment evidence checklist validation failed: {message}", file=sys.stderr)
    return 1


def parse_evidence(items: list[str]) -> dict[str, str]:
    references: dict[str, str] = {}
    for item in items:
        if "=" not in item:
            raise ValueError(f"evidence must use CATEGORY=REFERENCE: {item}")
        category, reference = item.split("=", 1)
        category = category.strip()
        reference = reference.strip()
        if category not in CATEGORIES:
            expected = ", ".join(sorted(CATEGORIES))
            raise ValueError(f"unknown evidence category '{category}', expected one of: {expected}")
        if category in references:
            raise ValueError(f"duplicate evidence category: {category}")
        validate_reference(reference)
        references[category] = reference
    return references


def validate_reference(reference: str) -> None:
    if SECRET_LIKE_PATTERN.search(reference):
        raise ValueError("evidence reference contains secret-like wording")
    if not REFERENCE_PATTERN.fullmatch(reference):
        raise ValueError(
            "evidence reference must be a short non-secret locator using letters, digits, and URL-safe punctuation"
        )


def load_bundle_report() -> dict[str, Any]:
    completed = subprocess.run(
        [sys.executable, "scripts/validate_deployment_evidence_bundle.py", "--json"],
        cwd=ROOT,
        check=False,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError("deployment evidence bundle validation did not pass")
    try:
        report = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"deployment evidence bundle did not emit valid JSON: {error}") from error
    if report.get("schema") != BUNDLE_SCHEMA:
        raise RuntimeError("deployment evidence bundle emitted an unexpected schema")
    return report


def bundle_safety_failures(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    for field in BOOLEAN_SAFETY_FIELDS:
        value = report.get(field)
        if value is not False:
            failures.append(field)
    if report.get("all_components_passed") is not True:
        failures.append("all_components_passed")
    unsafe_flags = report.get("unsafe_flags", [])
    if unsafe_flags:
        failures.append("unsafe_flags")
    return failures


def build_checklist(bundle: dict[str, Any], references: dict[str, str]) -> dict[str, Any]:
    safety_failures = bundle_safety_failures(bundle)
    if safety_failures:
        raise RuntimeError(f"bundle safety check failed: {', '.join(safety_failures)}")

    checklist = []
    for category, description in CATEGORIES.items():
        reference = references.get(category)
        checklist.append(
            {
                "category": category,
                "description": description,
                "status": "reference-provided" if reference else "missing-external-evidence",
                "reference": reference,
            }
        )

    missing = [item["category"] for item in checklist if item["status"] == "missing-external-evidence"]
    components = bundle.get("components", [])
    if not isinstance(components, list):
        components = []

    return {
        "schema": SCHEMA,
        "bundle_index": {
            "schema": bundle.get("schema"),
            "component_count": bundle.get("component_count"),
            "all_components_passed": bundle.get("all_components_passed"),
            "component_names": [component.get("name") for component in components],
        },
        "checklist": checklist,
        "remaining_missing_categories": missing,
        "all_external_evidence_referenced": not missing,
        "production_readiness_claimed": False,
        "service_actions_performed": False,
        "files_changed": False,
        "secrets_loaded": False,
        "external_calls_performed": False,
        "alerts_sent": False,
        "live_execution_enabled": False,
        "artifact_contents_embedded": False,
    }


def print_text_report(report: dict[str, Any]) -> None:
    print("deployment evidence checklist validation report")
    print(f"bundle schema: {report['bundle_index']['schema']}")
    print(f"bundle components passed: {str(report['bundle_index']['all_components_passed']).lower()}")
    print(f"all external evidence referenced: {str(report['all_external_evidence_referenced']).lower()}")
    print(f"production readiness claimed: {str(report['production_readiness_claimed']).lower()}")
    print(f"artifact contents embedded: {str(report['artifact_contents_embedded']).lower()}")
    print("external evidence checklist:")
    for item in report["checklist"]:
        reference = item["reference"] or "none"
        print(f"- {item['category']}: {item['status']}, reference={reference}")
    print("remaining missing categories:")
    for category in report["remaining_missing_categories"]:
        print(f"- {category}")


def main() -> int:
    args = parse_args()
    try:
        references = parse_evidence(args.evidence)
        bundle = load_bundle_report()
        report = build_checklist(bundle, references)
    except (OSError, RuntimeError, ValueError) as error:
        return fail(str(error))

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text_report(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
