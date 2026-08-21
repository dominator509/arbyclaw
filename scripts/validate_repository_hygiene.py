#!/usr/bin/env python3
"""Fail closed on tracked repository artifacts that should never be source of truth.

This validator is intentionally independent of generated manifests. It inspects the
actual Git index so ignored-but-still-tracked files cannot hide behind .gitignore or
manifest exclusions. Serena memories are allowed only for durable, generalizable
project knowledge; date-stamped or numbered-phase task snapshots belong in Git
history rather than the active agent-memory graph.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import re
from pathlib import PurePosixPath
import subprocess
import sys


FORBIDDEN_EXACT = {
    "docs/ai/repomix-summary.xml",
}

FORBIDDEN_PREFIXES = (
    ".obsidian/",
    "security-audit/",
)

FORBIDDEN_GLOBS = (
    "crates/*/arbyclaw.cdx.json",
    "**/__pycache__/**",
    "**/*.pyc",
    "**/*.pyo",
    "**/*.pyd",
)

SERENA_MEMORY_PREFIX = ".serena/memories/"
DATE_STAMP = re.compile(r"(?:^|[^0-9])\d{4}-\d{2}-\d{2}(?:[^0-9]|$)")
SERENA_PHASE_MEMORY = re.compile(r"^\.serena/memories/phase\d+(?:/|$)")


def tracked_files() -> list[str]:
    proc = subprocess.run(
        ["git", "ls-files", "-z"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.decode("utf-8", errors="replace").strip())
    files = [item for item in proc.stdout.decode("utf-8").split("\0") if item]
    if not files:
        raise RuntimeError("git ls-files returned zero tracked files")
    return files


def violation_reason(path: str) -> str | None:
    posix = PurePosixPath(path).as_posix()
    name = PurePosixPath(posix).name

    if posix in FORBIDDEN_EXACT:
        return "generated repository snapshot must not be tracked"
    if any(posix.startswith(prefix) for prefix in FORBIDDEN_PREFIXES):
        if posix.startswith("security-audit/"):
            return "legacy simulated/mock audit evidence must not be tracked"
        return "local workspace/editor state must not be tracked"
    if posix.startswith(SERENA_MEMORY_PREFIX):
        if SERENA_PHASE_MEMORY.match(posix):
            return "numbered-phase Serena memory is historical task state, not durable project memory"
        if DATE_STAMP.search(name):
            return "date-stamped Serena memory is episodic task state; preserve chronology in Git history instead"
    if name.startswith("__tmp_"):
        return "temporary scratch file must not be tracked"
    if ".bak-" in name:
        return "backup copy must not be tracked"
    for pattern in FORBIDDEN_GLOBS:
        if fnmatch.fnmatch(posix, pattern):
            return f"generated/cache artifact matches forbidden pattern {pattern}"
    return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args()

    try:
        files = tracked_files()
    except RuntimeError as exc:
        payload = {"status": "error", "reason": str(exc), "tracked_file_count": 0}
        if args.as_json:
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"repository hygiene validation error: {exc}", file=sys.stderr)
        return 2

    violations = [
        {"path": path, "reason": reason}
        for path in files
        if (reason := violation_reason(path)) is not None
    ]

    payload = {
        "status": "passed" if not violations else "failed",
        "tracked_file_count": len(files),
        "violation_count": len(violations),
        "violations": violations,
    }

    if args.as_json:
        print(json.dumps(payload, sort_keys=True))
    elif violations:
        print("repository hygiene validation failed:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation['path']}: {violation['reason']}", file=sys.stderr)
    else:
        print(f"repository hygiene validation passed ({len(files)} tracked files checked)")

    return 0 if not violations else 1


if __name__ == "__main__":
    raise SystemExit(main())
