#!/usr/bin/env python3
"""Enforce monotonic architecture ratchets during incremental refactors.

Existing monoliths are grandfathered at the exact Phase-5 baseline size. They
may shrink or disappear, but may not grow. New source files have a much lower
ceiling so a refactor cannot simply move a monolith into a differently named
file. The arb-core crate-root re-export surface may shrink but cannot expand to
new root-exported domain modules while domain-qualified imports are being
introduced. This is an architectural regression guard, not a claim that current
file sizes or exports are desirable.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "validation/architecture_ratchets.json"
CORE_LIB_PATH = ROOT / "crates/arb-core/src/lib.rs"
ROOT_REEXPORT = re.compile(r"(?m)^pub\s+use\s+([A-Za-z_][A-Za-z0-9_]*)::")


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

    allowed_reexports_raw = policy.get("allowed_root_reexport_modules")
    max_reexport_modules = policy.get("max_root_reexport_module_count")
    if (
        not isinstance(allowed_reexports_raw, list)
        or not allowed_reexports_raw
        or any(not isinstance(item, str) or not item for item in allowed_reexports_raw)
    ):
        print("architecture ratchet validation failed: invalid root re-export module allowlist", file=sys.stderr)
        return 2
    if not isinstance(max_reexport_modules, int) or max_reexport_modules <= 0:
        print("architecture ratchet validation failed: invalid root re-export module ceiling", file=sys.stderr)
        return 2
    allowed_reexports = set(allowed_reexports_raw)
    if len(allowed_reexports) != len(allowed_reexports_raw):
        print("architecture ratchet validation failed: duplicate root re-export module entry", file=sys.stderr)
        return 2
    if max_reexport_modules != len(allowed_reexports):
        print(
            "architecture ratchet validation failed: root re-export module ceiling must equal baseline allowlist size",
            file=sys.stderr,
        )
        return 2

    violations: list[dict[str, object]] = []
    observations: list[dict[str, object]] = []

    for relative, maximum in sorted(ratchets.items()):
        if not isinstance(relative, str) or not isinstance(maximum, int) or maximum <= 0:
            violations.append({"path": str(relative), "reason": "invalid ratchet entry"})
            continue
        path = ROOT / relative
        if not path.exists():
            observations.append(
                {"path": relative, "status": "removed-or-decomposed", "baseline_bytes": maximum}
            )
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

    try:
        core_lib = CORE_LIB_PATH.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"architecture ratchet validation failed: unable to read arb-core lib.rs: {exc}", file=sys.stderr)
        return 2
    reexport_modules = ROOT_REEXPORT.findall(core_lib)
    reexport_set = set(reexport_modules)
    if len(reexport_modules) != len(reexport_set):
        violations.append(
            {
                "path": "crates/arb-core/src/lib.rs",
                "reason": "a domain is root-reexported in multiple blocks; consolidate before migration",
            }
        )
    unexpected_reexports = sorted(reexport_set - allowed_reexports)
    if unexpected_reexports:
        violations.append(
            {
                "path": "crates/arb-core/src/lib.rs",
                "reason": "new root-exported domain modules expand the legacy flat API surface",
                "modules": unexpected_reexports,
            }
        )
    if len(reexport_set) > max_reexport_modules:
        violations.append(
            {
                "path": "crates/arb-core/src/lib.rs",
                "reason": "root re-export module count exceeded baseline",
                "actual_modules": len(reexport_set),
                "maximum_modules": max_reexport_modules,
            }
        )

    payload = {
        "schema": "arbyclaw.architecture_ratchet_validation.v2",
        "status": "failed" if violations else "passed",
        "baseline_commit": policy.get("baseline_commit"),
        "ratcheted_file_count": len(ratchets),
        "root_reexport_module_count": len(reexport_set),
        "root_reexport_modules": sorted(reexport_set),
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
            f"({len(ratchets)} legacy files may only shrink; "
            f"{len(reexport_set)} root-exported domains may only shrink; new source ceilings enforced)"
        )

    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())
