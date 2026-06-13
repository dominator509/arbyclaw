#!/usr/bin/env python3
"""Run the strongest local opportunity scenario-corpus validation bundle.

This gate composes existing opportunity replay, load, historical fixture,
planner-handoff, and trace-recovery CLI probes. It validates only local
synthetic/recorded fixtures and fails if any nested command reports external
calls, data downloads, adapter submission, signing, broadcasts, live execution,
or production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT_SECONDS = 900

COMMANDS = [
    (
        "opportunity_replay",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-opportunity-replay",
            "--iterations",
            "2",
        ],
    ),
    (
        "opportunity_quote_load",
        [
            "cargo",
            "run",
            "-p",
            "arb-agent",
            "--",
            "validate-opportunity-quote-load",
            "--venue-pairs",
            "8",
            "--max-candidates",
            "3",
        ],
    ),
    (
        "opportunity_provider_ingestion",
        ["cargo", "run", "-p", "arb-agent", "--", "validate-opportunity-provider-ingestion"],
    ),
    (
        "opportunity_historical_fixtures",
        ["cargo", "run", "-p", "arb-agent", "--", "validate-opportunity-historical-fixtures"],
    ),
    (
        "opportunity_planner_handoff",
        ["cargo", "run", "-p", "arb-agent", "--", "validate-opportunity-planner-handoff"],
    ),
    (
        "opportunity_trace_recovery",
        ["cargo", "run", "-p", "arb-agent", "--", "validate-opportunity-trace-recovery"],
    ),
]

DANGEROUS_TRUE_KEYS = {
    "adapter-submission-enabled",
    "adapter-submission-performed",
    "external-calls-performed",
    "external-data-downloaded",
    "external-execution-performed",
    "live-execution-performed",
    "production-ready",
    "signing-or-broadcast-performed",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    return parser.parse_args()


def parse_output(text: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or ": " not in line:
            continue
        key, value = line.split(": ", 1)
        parsed[key.strip()] = value.strip()
    return parsed


def bool_value(value: str) -> bool | None:
    lowered = value.strip().lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    return None


def run_component(name: str, command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    parsed = parse_output(output)
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "parsed": parsed,
    }


def validate_components(components: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    for component in components:
        name = component["name"]
        if component["returncode"] != 0:
            errors.append(f"{name} exited {component['returncode']}")
        parsed = component["parsed"]
        for key in DANGEROUS_TRUE_KEYS:
            value = parsed.get(key)
            if value is not None and bool_value(value) is True:
                errors.append(f"{name} reported unsafe side-effect flag {key}: true")

    replay = components[0]["parsed"]
    if replay.get("opportunity-replay-failed") not in {"0", None}:
        errors.append("opportunity replay reported failed scenarios")
    if replay.get("opportunity-replay-iterations-passed") != replay.get(
        "opportunity-replay-iterations-attempted"
    ):
        errors.append("opportunity replay iterations did not all pass")

    quote_load = components[1]["parsed"]
    if quote_load.get("candidate-backpressure-applied") != "true":
        errors.append("quote-load did not prove candidate backpressure")

    historical = components[3]["parsed"]
    if historical.get("opportunity-historical-fixture-status") not in {
        "local-fixture-replay-passed",
        "passed",
    }:
        errors.append("historical fixture status was not a local pass")

    planner = components[4]["parsed"]
    if planner.get("candidate-trace-audit-records") != planner.get(
        "candidate-trace-checkpoints"
    ):
        errors.append("planner handoff trace audit/checkpoint counts diverged")

    trace = components[5]["parsed"]
    if trace.get("trace-recovery-validated") != "true":
        errors.append("trace recovery was not validated")
    if trace.get("missing-trace-checkpoints") not in {"0", None}:
        errors.append("trace recovery reported missing checkpoints")

    return errors


def main() -> int:
    args = parse_args()
    components = [run_component(name, command) for name, command in COMMANDS]
    errors = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    total_candidates = 0
    for component in components:
        for key, value in component["parsed"].items():
            if "candidates" in key or key.endswith("-candidates"):
                try:
                    total_candidates += int(value)
                except ValueError:
                    pass

    report = {
        "schema": "arbyclaw.opportunity_scenario_aggregate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "external_calls_performed": False,
        "external_data_downloaded": False,
        "adapter_submission_performed": False,
        "signing_or_broadcast_performed": False,
        "live_execution_performed": False,
        "production_ready": False,
        "total_candidate_mentions": total_candidates,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_evidence": [
            "broader external/deployment scenario-corpus execution",
            "external sandbox/live calibration evidence",
            "live/provider-backed market-data validation",
            "production runtime validation",
        ],
    }
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("opportunity scenario aggregate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print("external-calls-performed: false")
        print("external-data-downloaded: false")
        print("adapter-submission-performed: false")
        print("signing-or-broadcast-performed: false")
        print("live-execution-performed: false")
        print("production-ready: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
