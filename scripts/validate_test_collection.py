#!/usr/bin/env python3
"""Fail if required Rust packages collect zero tests.

A successful `cargo test` command is not sufficient evidence when a package
silently collects zero tests. This guard enumerates tests package-by-package
before the real test run.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_PACKAGES = ("arb-core", "arb-agent")


def collect(package: str) -> tuple[int, list[str], str]:
    proc = subprocess.run(
        ["cargo", "test", "-p", package, "--locked", "--", "--list"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    output = f"{proc.stdout}\n{proc.stderr}".strip()
    tests = [
        line.strip()
        for line in proc.stdout.splitlines()
        if line.strip().endswith(": test")
    ]
    return proc.returncode, tests, output


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", dest="as_json")
    parser.add_argument("--package", action="append", dest="packages")
    args = parser.parse_args()

    packages = tuple(args.packages or DEFAULT_PACKAGES)
    if not packages:
        print("test collection validation failed: zero packages requested", file=sys.stderr)
        return 2

    results: list[dict[str, object]] = []
    failed = False
    for package in packages:
        returncode, tests, output = collect(package)
        result = {
            "package": package,
            "returncode": returncode,
            "test_count": len(tests),
            "tests": tests,
        }
        results.append(result)
        if returncode != 0 or not tests:
            failed = True
            if not args.as_json:
                print(
                    f"test collection validation failed for {package}: "
                    f"returncode={returncode}, test_count={len(tests)}",
                    file=sys.stderr,
                )
                if returncode != 0:
                    print("\n".join(output.splitlines()[-30:]), file=sys.stderr)

    payload = {
        "status": "failed" if failed else "passed",
        "package_count": len(packages),
        "total_test_count": sum(int(item["test_count"]) for item in results),
        "packages": results,
    }

    if args.as_json:
        print(json.dumps(payload, sort_keys=True))
    elif not failed:
        details = ", ".join(
            f"{item['package']}={item['test_count']}" for item in results
        )
        print(f"test collection validation passed ({details})")

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
