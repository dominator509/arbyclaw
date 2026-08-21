#!/usr/bin/env python3
"""Enforce monotonic architecture-size ratchets during incremental refactors.

Existing monoliths are grandfathered at the exact Phase-5 baseline size. They
may shrink or disappear, but may not grow. New source files have a much lower
ceiling so a refactor cannot simply move a monolith into a differently named
file. This is an architectural regression guard, not a claim that current file
sizes are desirable.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "validation/architecture_ratchets.json"


def load_policy() -> dict[str, Any]:
    try:
        loaded = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"unable to load {POLICY_PATH.relative_to(ROOT)}: {exc}") from exc
    if loaded.get("schema") != "arbyclaw.architecture_ratchets.v1":
        raise RuntimeError("architecture ratchet policy has unexpected schema")
    return loaded


def source_files() -> list[pathlib.Path]:
    roots = (ROOT / "crates", ROOT / "scripts")
    files: list[pathlib.Path] = []
    for source_root in roots:
        if not source_root.exists():
            continue
        for path in source_root.rglob("*"):
            if not path.is_file():
                continue
            if path.suffix not in {".rs", ".py"}:
                continue
            if "target" in path.parts or "__pycache__" in path.parts:
                continue
            files.append(path)
    return sorted(files)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args()

    try:
        policy = load_policy()
    except RuntimeError as exc:
        print(f"architecture ratchet validation failed: {exc}", file=sys.stderr)
        return 2

    ratchets = policy.get("ratcheted_files")
    if not isinstance(ratchets, dict) or not ratchets:
        print("architecture ratchet validation failed: no ratcheted files configured", file=sys.stderr)
        return 2

    new_rust_max = policy.get("new_rust_file_max_bytes")
    new_python_max = policy.get("new_python_file_max_bytes")
    if not isinstance(new_rust_max, int) or new_rust_max <= 0:
        print("architecture ratchet validation failed: invalid Rust new-file ceiling", file=sys.stderr)
        return 2
    if not isinstance(new_python_max, int) or new_python_max <= 0:
        print("architecture ratchet validation failed: invalid Python new-file ceiling", file=sys.stderr)
        return 2

    violations: list[dict[str, object]] = []
    observations: list[dict[str, object]] = []

    for relative, maximum in sorted(ratchets.items()):
        if not isinstance(relative, str) or not isinstance(maximum, int) or maximum <= 0:
            violations.append({"path": str(relative), "reason": "invalid ratchet entry"})
            continue
        path = ROOT / relative
        if not path.exists():
            observations.append({"path": relative, "status": "removed-or-decomposed", "baseline_bytes": maximum})
            continue
        actual = path.stat().st_size
        observations.append(
            {
                "path": relative,
                "status": "within-ratchet" if actual <= maximum else "grew",
                "actual_bytes": actual,
                "baseline_bytes": maximum,
                "delta_bytes": actual - maximum,
            }
        )
        if actual > maximum:
            violations.append(
                {
                    "path": relative,
                    "reason": "legacy monolith grew instead of shrinking",
                    "actual_bytes": actual,
                    "maximum_bytes": maximum,
                }
            )

    ratcheted_paths = set(ratchets)
    for path in source_files():
        relative = path.relative_to(ROOT).as_posix()
        if relative in ratcheted_paths:
            continue
        maximum = new_rust_max if path.suffix == ".rs" else new_python_max
        actual = path.stat().st_size
        if actual > maximum:
            violations.append(
                {
                    "path": relative,
                    "reason": "new/unratcheted source file exceeds monolith-prevention ceiling",
                    "actual_bytes": actual,
                    "maximum_bytes": maximum,
                }
            )

    payload = {
        "schema": "arbyclaw.architecture_ratchet_validation.v1",
        "status": "failed" if violations else "passed",
        "baseline_commit": policy.get("baseline_commit"),
        "ratcheted_file_count": len(ratchets),
        "violation_count": len(violations),
        "violations": violations,
        "observations": observations,
    }

    if args.as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif violations:
        print("architecture ratchet validation failed:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation['path']}: {violation['reason']}", file=sys.stderr)
    else:
        print(
            "architecture ratchet validation passed "
            f"({len(ratchets)} legacy files may only shrink; new source ceilings enforced)"
        )

    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())
