#!/usr/bin/env python3
"""Validate the stable behavior contract used during structural refactors.

This gate does not pretend that static contract inspection is runtime evidence.
It guarantees that safety-critical requirement IDs remain unique, their CLI
entrypoints still exist somewhere in the arb-agent source tree, and the
human-readable refactor contract remains synchronized. Runtime equivalence is
established separately by real tests/CLI execution in CI.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTRACT_PATH = ROOT / "validation/behavior_contract.json"
DOC_PATH = ROOT / "docs/refactoring/BEHAVIOR_COMPATIBILITY_CONTRACT.md"
AGENT_SOURCE_ROOT = ROOT / "crates/arb-agent/src"
REQ_ID = re.compile(r"^BC-\d{3}$")


def load_contract() -> dict[str, Any]:
    try:
        loaded = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"unable to load behavior contract: {exc}") from exc
    if loaded.get("schema") != "arbyclaw.behavior_contract.v1":
        raise RuntimeError("behavior contract has unexpected schema")
    return loaded


def load_agent_source() -> str:
    files = sorted(path for path in AGENT_SOURCE_ROOT.rglob("*.rs") if path.is_file())
    if not files:
        raise RuntimeError("arb-agent source tree contains no Rust files")
    return "\n".join(path.read_text(encoding="utf-8") for path in files)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args()

    try:
        contract = load_contract()
        agent_source = load_agent_source()
        doc = DOC_PATH.read_text(encoding="utf-8")
    except (RuntimeError, OSError) as exc:
        print(f"behavior contract validation failed: {exc}", file=sys.stderr)
        return 2

    errors: list[str] = []
    rules = contract.get("compatibility_rules")
    required_rule_names = (
        "preserve_command_names_during_structural_refactors",
        "preserve_exit_code_semantics",
        "preserve_json_schema_names",
        "preserve_fail_closed_safety_defaults",
        "structural_and_behavioral_changes_must_be_separate_commits",
    )
    if not isinstance(rules, dict):
        errors.append("compatibility_rules must be an object")
    else:
        for name in required_rule_names:
            if rules.get(name) is not True:
                errors.append(f"compatibility rule must remain true: {name}")

    requirements = contract.get("requirements")
    if not isinstance(requirements, list) or not requirements:
        errors.append("behavior contract has no requirements")
        requirements = []

    seen_ids: set[str] = set()
    seen_commands: set[str] = set()
    for requirement in requirements:
        if not isinstance(requirement, dict):
            errors.append("behavior requirement is not an object")
            continue
        requirement_id = requirement.get("id")
        if not isinstance(requirement_id, str) or not REQ_ID.fullmatch(requirement_id):
            errors.append(f"invalid behavior requirement id: {requirement_id}")
            continue
        if requirement_id in seen_ids:
            errors.append(f"duplicate behavior requirement id: {requirement_id}")
        seen_ids.add(requirement_id)
        if requirement_id not in doc:
            errors.append(f"human-readable behavior contract is missing {requirement_id}")

        description = requirement.get("description")
        if not isinstance(description, str) or len(description.strip()) < 20:
            errors.append(f"{requirement_id} has an incomplete description")

        commands = requirement.get("commands")
        if not isinstance(commands, list) or not commands:
            errors.append(f"{requirement_id} has no CLI evidence commands")
            continue
        for command in commands:
            if not isinstance(command, str) or not command.startswith("validate-"):
                errors.append(f"{requirement_id} contains invalid command {command}")
                continue
            if command not in agent_source:
                errors.append(
                    f"{requirement_id} references CLI command not found in arb-agent source tree: {command}"
                )
            seen_commands.add(command)

        must_report = requirement.get("must_report")
        if not isinstance(must_report, dict) or not must_report:
            errors.append(f"{requirement_id} has no observable output invariants")
        else:
            for field, value in must_report.items():
                if not isinstance(field, str) or not field:
                    errors.append(f"{requirement_id} contains invalid output field")
                if value not in {"true", "false"} and not isinstance(value, str):
                    errors.append(f"{requirement_id} contains unsupported expected output value")

    if len(seen_ids) < 10:
        errors.append("behavior contract must retain at least ten independent safety requirements")
    if len(seen_commands) < 10:
        errors.append("behavior contract must cover at least ten distinct CLI commands")

    payload = {
        "schema": "arbyclaw.behavior_contract_validation.v1",
        "status": "failed" if errors else "passed",
        "requirement_count": len(seen_ids),
        "command_count": len(seen_commands),
        "error_count": len(errors),
        "errors": errors,
    }
    if args.as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif errors:
        print("behavior contract validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
    else:
        print(
            f"behavior contract validation passed ({len(seen_ids)} requirements; "
            f"{len(seen_commands)} CLI commands anchored)"
        )
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
