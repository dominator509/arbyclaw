#!/usr/bin/env python3
"""Run the strongest local opportunity scenario-corpus validation bundle.

This gate composes existing opportunity replay, load, historical fixture,
planner-handoff, strategy replay/profitability tuning, and trace-recovery CLI
probes. It validates only local synthetic/recorded fixtures and fails if any
nested command reports external calls, data downloads, adapter submission,
signing, broadcasts, live execution, or production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT_SECONDS = 900

def parse_int(value: str | None) -> int:
    if value is None:
        return 0
    try:
        return int(value)
    except ValueError:
        return 0


def command_set(workspace_root: pathlib.Path) -> list[tuple[str, list[str]]]:
    return [
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
            "strategy_replay_corpus",
            ["cargo", "run", "-p", "arb-agent", "--", "validate-strategy-replay-corpus"],
        ),
        (
            "strategy_profitability_tuning",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-strategy-profitability-tuning",
            ],
        ),
        (
            "local_validation_run",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-local-validation-run",
                "--workspace",
                str(workspace_root / "validation-run"),
            ],
        ),
        (
            "local_property_checks",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-local-property-checks",
                "--workspace",
                str(workspace_root / "property-checks"),
            ],
        ),
        (
            "local_fuzz_corpus",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-local-fuzz-corpus",
                "--workspace",
                str(workspace_root / "fuzz-corpus"),
            ],
        ),
        (
            "local_validation_corpus",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-local-validation-corpus",
                "--workspace",
                str(workspace_root / "validation-corpus"),
            ],
        ),
        (
            "local_paper_backtest_corpus",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-local-paper-backtest-corpus",
                "--workspace",
                str(workspace_root / "paper-backtest-corpus"),
            ],
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
    "external-fuzzer-invoked",
    "live-execution-performed",
    "live-execution-submitted",
    "live-network-used",
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
    if replay.get("opportunity-replay-latency-review") != "ReadyForLocalReview":
        errors.append("opportunity replay latency review was not ReadyForLocalReview")
    if replay.get("opportunity-replay-latency-budget-met") != "true":
        errors.append("opportunity replay latency budget was not met")
    if replay.get("opportunity-replay-throughput-budget-met") != "true":
        errors.append("opportunity replay throughput budget was not met")
    remaining_external = replay.get(
        "opportunity-replay-latency-review-remaining-external-evidence-count"
    )
    try:
        if int(remaining_external or "0") <= 0:
            errors.append(
                "opportunity replay latency review did not preserve external evidence blockers"
            )
    except ValueError:
        errors.append("opportunity replay latency review external blocker count was invalid")

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

    strategy = components[5]["parsed"]
    if strategy.get("strategy-replay-status") != "passed":
        errors.append("strategy replay corpus status was not passed")
    if strategy.get("accepted-strategy-rejected-intents") not in {"0", None}:
        errors.append("accepted strategy replay profile rejected intents")
    if strategy.get("rejected-policy-denied-plans") != strategy.get(
        "discovered-candidates"
    ):
        errors.append("rejected strategy replay profile did not deny every discovered candidate")

    profitability = components[6]["parsed"]
    if profitability.get("strategy-profitability-status") != "passed":
        errors.append("strategy profitability tuning status was not passed")
    if profitability.get("monotonic-acceptance-validated") != "true":
        errors.append("strategy profitability tuning did not validate monotonic acceptance")
    if profitability.get("monotonic-rejection-validated") != "true":
        errors.append("strategy profitability tuning did not validate monotonic rejection")
    if profitability.get("profitability-threshold-transition-observed") != "true":
        errors.append("strategy profitability tuning did not observe a threshold transition")

    validation_run = components[7]["parsed"]
    if validation_run.get("validation-run-status") != "planned-only":
        errors.append("local validation run status was not planned-only")
    if validation_run.get("planned-test-cases") in {"0", None}:
        errors.append("local validation run did not report planned test cases")
    if validation_run.get("planned-fixtures") in {"0", None}:
        errors.append("local validation run did not report planned fixtures")
    if validation_run.get("planned-fuzz-corpora") in {"0", None}:
        errors.append("local validation run did not report planned fuzz corpora")
    if validation_run.get("planned-backtest-scenarios") in {"0", None}:
        errors.append("local validation run did not report planned backtest scenarios")
    if validation_run.get("state-checkpoint-recovered") != "true":
        errors.append("local validation run did not recover its state checkpoint")

    property_checks = components[8]["parsed"]
    if property_checks.get("property-checks-executed") != property_checks.get(
        "property-checks-passed"
    ):
        errors.append("local property checks did not pass every executed check")
    if property_checks.get("property-checks-failed") not in {"0", None}:
        errors.append("local property checks reported failed checks")
    if property_checks.get("missing-fixture-references") not in {"0", None}:
        errors.append("local property checks reported missing fixture references")
    if property_checks.get("empty-fuzz-corpora") not in {"0", None}:
        errors.append("local property checks reported empty fuzz corpora")
    if property_checks.get("nonlocal-backtest-datasets") not in {"0", None}:
        errors.append("local property checks reported nonlocal backtest datasets")
    if property_checks.get("state-checkpoint-recovered") != "true":
        errors.append("local property checks did not recover their state checkpoint")

    fuzz_corpus = components[9]["parsed"]
    if fuzz_corpus.get("fuzz-replay-status") != "ready-for-local-review":
        errors.append("local fuzz corpus status was not ready-for-local-review")
    if fuzz_corpus.get("fuzz-corpora") in {"0", None}:
        errors.append("local fuzz corpus did not report fuzz corpora")
    if fuzz_corpus.get("fuzz-seeds") in {"0", None}:
        errors.append("local fuzz corpus did not report fuzz seeds")
    if fuzz_corpus.get("fuzz-targets") in {"0", None}:
        errors.append("local fuzz corpus did not report fuzz targets")
    if fuzz_corpus.get("unique-fuzz-seeds") != fuzz_corpus.get("fuzz-seeds"):
        errors.append("local fuzz corpus unique seed count diverged from total seeds")
    if fuzz_corpus.get("state-checkpoint-recovered") != "true":
        errors.append("local fuzz corpus did not recover its state checkpoint")

    validation_corpus = components[10]["parsed"]
    if validation_corpus.get("validation-corpus-status") != "ready-for-local-review":
        errors.append("local validation corpus status was not ready-for-local-review")
    if validation_corpus.get("accepted-validation-plans") != validation_corpus.get(
        "validation-plans"
    ):
        errors.append("local validation corpus did not accept every validation plan")
    if validation_corpus.get("property-checks-failed") not in {"0", None}:
        errors.append("local validation corpus reported failed property checks")
    if validation_corpus.get("corpus-breadth-requirements-met") != "true":
        errors.append("local validation corpus breadth requirements were not met")
    for reported_key, minimum_key in (
        ("validation-plans", "min-validation-plans"),
        ("planned-test-cases", "min-test-cases"),
        ("planned-fixtures", "min-fixtures"),
        ("planned-fuzz-corpora", "min-fuzz-corpora"),
        ("planned-backtest-scenarios", "min-backtest-scenarios"),
    ):
        if parse_int(validation_corpus.get(reported_key)) < parse_int(
            validation_corpus.get(minimum_key)
        ):
            errors.append(
                f"local validation corpus {reported_key} below {minimum_key}"
            )
    if validation_corpus.get("state-checkpoint-recovered") != "true":
        errors.append("local validation corpus did not recover its state checkpoint")

    paper_backtest = components[11]["parsed"]
    if paper_backtest.get("paper-backtest-replay-validated") != "true":
        errors.append("local paper backtest corpus did not validate replay")
    if paper_backtest.get("paper-backtest-filled-steps") != "1":
        errors.append("local paper backtest corpus filled-step count changed")
    if paper_backtest.get("paper-backtest-partial-steps") != "1":
        errors.append("local paper backtest corpus partial-step count changed")
    if paper_backtest.get("paper-backtest-unfilled-steps") != "1":
        errors.append("local paper backtest corpus unfilled-step count changed")
    if paper_backtest.get("state-checkpoint-recovered") != "true":
        errors.append("local paper backtest corpus did not recover its state checkpoint")

    trace = components[12]["parsed"]
    if trace.get("trace-recovery-validated") != "true":
        errors.append("trace recovery was not validated")
    if trace.get("missing-trace-checkpoints") not in {"0", None}:
        errors.append("trace recovery reported missing checkpoints")

    return errors


def main() -> int:
    args = parse_args()
    with tempfile.TemporaryDirectory(prefix="opportunity-scenario-gate-", dir=ROOT / "target") as temp_dir:
        workspace_root = pathlib.Path(temp_dir)
        components = [
            run_component(name, command)
            for name, command in command_set(workspace_root)
        ]
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
        "external_fuzzer_invoked": False,
        "live_network_used": False,
        "signing_or_broadcast_performed": False,
        "live_execution_performed": False,
        "production_ready": False,
        "opportunity_replay_latency_review_enforced": True,
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
            "external fuzzing-engine and broader production backtest execution",
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
        print("external-fuzzer-invoked: false")
        print("live-network-used: false")
        print("signing-or-broadcast-performed: false")
        print("live-execution-performed: false")
        print("production-ready: false")
        print("opportunity-replay-latency-review-enforced: true")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
