#!/usr/bin/env python3
"""Validate aggregate ownership as a real acyclic single-path graph.

The graph file is the authoritative ownership map for aggregate validators. This
script cross-checks that map against actual Python source, rejects cycles and
multiple aggregate paths, enforces single ownership for selected safety-critical
leaf commands, and prevents CI from directly invoking lower aggregate suites in
addition to the top handoff gate.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from collections import defaultdict
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "validation/validation_graph.json"
CI_PATH = ROOT / ".github/workflows/ci.yml"
AGGREGATE_REF = re.compile(r"scripts/(validate_[A-Za-z0-9_]+\.py)")


def load_graph() -> dict[str, Any]:
    try:
        graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"unable to load validation graph: {exc}") from exc
    if graph.get("schema") != "arbyclaw.validation_graph.v1":
        raise RuntimeError("validation graph has unexpected schema")
    return graph


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args()

    try:
        graph = load_graph()
    except RuntimeError as exc:
        print(f"validation graph check failed: {exc}", file=sys.stderr)
        return 2

    errors: list[str] = []
    raw_nodes = graph.get("nodes")
    if not isinstance(raw_nodes, list) or not raw_nodes:
        print("validation graph check failed: graph contains no nodes", file=sys.stderr)
        return 2

    nodes: dict[str, dict[str, Any]] = {}
    script_to_id: dict[str, str] = {}
    for raw in raw_nodes:
        if not isinstance(raw, dict):
            errors.append("graph node is not an object")
            continue
        node_id = raw.get("id")
        script = raw.get("script")
        children = raw.get("children")
        if not isinstance(node_id, str) or not node_id:
            errors.append("graph node has invalid id")
            continue
        if node_id in nodes:
            errors.append(f"duplicate graph node id: {node_id}")
            continue
        if not isinstance(script, str) or not script.startswith("scripts/"):
            errors.append(f"graph node {node_id} has invalid script")
            continue
        if script in script_to_id:
            errors.append(f"graph script owned by multiple nodes: {script}")
            continue
        if not isinstance(children, list) or any(not isinstance(item, str) for item in children):
            errors.append(f"graph node {node_id} has invalid children")
            continue
        nodes[node_id] = raw
        script_to_id[script] = node_id

    root = graph.get("root")
    if not isinstance(root, str) or root not in nodes:
        errors.append("graph root is missing or unknown")

    for node_id, node in nodes.items():
        for child in node["children"]:
            if child not in nodes:
                errors.append(f"{node_id} references unknown child {child}")

    # Source-to-manifest cross-check for aggregate calls.
    for node_id, node in nodes.items():
        path = ROOT / node["script"]
        if not path.is_file():
            errors.append(f"aggregate script missing: {node['script']}")
            continue
        text = path.read_text(encoding="utf-8")
        actual_child_scripts = {
            f"scripts/{match}"
            for match in AGGREGATE_REF.findall(text)
            if f"scripts/{match}" in script_to_id
        }
        expected_child_scripts = {nodes[child]["script"] for child in node["children"]}
        if actual_child_scripts != expected_child_scripts:
            errors.append(
                f"{node_id} aggregate edges differ from manifest: "
                f"actual={sorted(actual_child_scripts)}, expected={sorted(expected_child_scripts)}"
            )

    # Cycle and multiple-path detection from the one supported top-level root.
    if isinstance(root, str) and root in nodes:
        visiting: set[str] = set()
        visited: set[str] = set()

        def visit(node_id: str) -> None:
            if node_id in visiting:
                errors.append(f"validation graph cycle detected at {node_id}")
                return
            if node_id in visited:
                return
            visiting.add(node_id)
            for child in nodes[node_id]["children"]:
                if child in nodes:
                    visit(child)
            visiting.remove(node_id)
            visited.add(node_id)

        visit(root)
        unreachable = sorted(set(nodes) - visited)
        if unreachable:
            errors.append(f"aggregate nodes unreachable from root: {unreachable}")

        path_counts: dict[str, int] = defaultdict(int)

        def count_paths(node_id: str) -> None:
            path_counts[node_id] += 1
            for child in nodes[node_id]["children"]:
                if path_counts[child] <= 1:
                    count_paths(child)

        count_paths(root)
        duplicated_paths = sorted(node for node, count in path_counts.items() if count > 1)
        if duplicated_paths:
            errors.append(f"aggregate nodes reachable by multiple paths: {duplicated_paths}")

    # Safety-critical leaf commands have exactly one aggregate owner.
    leaf_owners = graph.get("single_owner_leaf_commands", {})
    if not isinstance(leaf_owners, dict):
        errors.append("single_owner_leaf_commands must be an object")
    else:
        source_cache: dict[str, str] = {}
        for node_id, node in nodes.items():
            path = ROOT / node["script"]
            source_cache[node_id] = path.read_text(encoding="utf-8") if path.is_file() else ""
        for command, owner in leaf_owners.items():
            if not isinstance(command, str) or not isinstance(owner, str) or owner not in nodes:
                errors.append(f"invalid leaf ownership entry: {command} -> {owner}")
                continue
            occurrences = [node_id for node_id, text in source_cache.items() if command in text]
            if occurrences != [owner]:
                errors.append(
                    f"leaf command {command} must appear only in {owner}; found in {sorted(occurrences)}"
                )

    # Known historical recursion points may not reappear.
    forbidden = graph.get("forbidden_substrings", {})
    if not isinstance(forbidden, dict):
        errors.append("forbidden_substrings must be an object")
    else:
        for node_id, substrings in forbidden.items():
            if node_id not in nodes or not isinstance(substrings, list):
                errors.append(f"invalid forbidden-substring rule for {node_id}")
                continue
            text = (ROOT / nodes[node_id]["script"]).read_text(encoding="utf-8")
            for substring in substrings:
                if not isinstance(substring, str):
                    errors.append(f"non-string forbidden substring for {node_id}")
                elif substring in text:
                    errors.append(f"{node_id} reintroduced forbidden recursive invocation: {substring}")

    # CI owns repository/build checks and invokes the aggregate tree once at its root.
    try:
        ci_text = CI_PATH.read_text(encoding="utf-8")
    except OSError as exc:
        errors.append(f"unable to read CI workflow: {exc}")
        ci_text = ""
    ci_root = graph.get("ci_root_script")
    if isinstance(ci_root, str):
        if ci_text.count(ci_root) != 1:
            errors.append(f"CI must invoke aggregate root exactly once: {ci_root}")
        for script in script_to_id:
            if script != ci_root and script in ci_text:
                errors.append(f"CI directly invokes lower aggregate instead of root ownership: {script}")
    else:
        errors.append("ci_root_script is missing")

    payload = {
        "schema": "arbyclaw.validation_graph_check.v1",
        "status": "failed" if errors else "passed",
        "root": root,
        "aggregate_node_count": len(nodes),
        "error_count": len(errors),
        "errors": errors,
    }
    if args.as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif errors:
        print("validation graph check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
    else:
        print(f"validation graph check passed ({len(nodes)} aggregate nodes; single-path ownership)")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
