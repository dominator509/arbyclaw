#!/usr/bin/env python3
"""Validate the locked Cargo dependency license policy for the workspace."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]

WORKSPACE_LICENSE = "UNLICENSED"
REQUIRED_CRATES = {"serde", "toml", "serde_json", "sha2"}
ALLOWED_LICENSES = {
    "MIT",
    "MIT OR Apache-2.0",
    "Apache-2.0 OR MIT",
    "Apache-2.0 / MIT",
    "MIT/Apache-2.0",
    "MIT OR Apache-2.0 OR LGPL-2.1-or-later",
    "BSD-3-Clause",
    "BSD-2-Clause OR Apache-2.0 OR MIT",
    "Unlicense OR MIT",
    "Zlib",
    "(MIT OR Apache-2.0) AND Unicode-3.0",
    "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    WORKSPACE_LICENSE,
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    return parser.parse_args()


def cargo_metadata() -> dict[str, Any]:
    completed = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--quiet"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "cargo metadata failed")
    return json.loads(completed.stdout)


def validate_license_policy(metadata: dict[str, Any]) -> dict[str, Any]:
    packages = metadata["packages"]
    workspace_members = {
        package_id
        for package_id in metadata["workspace_members"]
    }
    packages_by_name = {package["name"] for package in packages}
    missing_required = sorted(REQUIRED_CRATES - packages_by_name)

    denied_packages: list[dict[str, str]] = []
    missing_license: list[dict[str, str]] = []
    workspace_license_violations: list[dict[str, str]] = []

    for package in sorted(packages, key=lambda item: (item["name"], item["version"])):
        license_expression = package.get("license")
        package_record = {
            "name": package["name"],
            "version": package["version"],
            "license": license_expression or "",
        }
        if not license_expression:
            missing_license.append(package_record)
            continue
        if package["id"] in workspace_members:
            if license_expression != WORKSPACE_LICENSE:
                workspace_license_violations.append(package_record)
            continue
        if license_expression not in ALLOWED_LICENSES:
            denied_packages.append(package_record)

    return {
        "schema": "arbyclaw.dependency_license_policy.v1",
        "package_count": len(packages),
        "required_crates_present": len(missing_required) == 0,
        "required_crates_checked": sorted(REQUIRED_CRATES),
        "missing_required_crates": missing_required,
        "allowed_license_count": len(ALLOWED_LICENSES),
        "missing_license_count": len(missing_license),
        "denied_package_count": len(denied_packages),
        "workspace_license_violation_count": len(workspace_license_violations),
        "missing_license_packages": missing_license,
        "denied_packages": denied_packages,
        "workspace_license_violations": workspace_license_violations,
        "passed": not (
            missing_required
            or missing_license
            or denied_packages
            or workspace_license_violations
        ),
    }


def main() -> int:
    args = parse_args()
    try:
        report = validate_license_policy(cargo_metadata())
    except Exception as error:  # pragma: no cover - defensive CLI guard
        if args.json:
            print(
                json.dumps(
                    {
                        "schema": "arbyclaw.dependency_license_policy.v1",
                        "passed": False,
                        "error": str(error),
                    },
                    indent=2,
                )
            )
        else:
            print(f"dependency-license-policy: validation failed", file=sys.stderr)
            print(f"error: {error}", file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print("dependency-license-policy: validation passed")
        print(f"package-count: {report['package_count']}")
        print(f"required-crates-present: {str(report['required_crates_present']).lower()}")
        print(f"missing-license-count: {report['missing_license_count']}")
        print(f"denied-package-count: {report['denied_package_count']}")
        print(
            "workspace-license-violation-count: "
            f"{report['workspace_license_violation_count']}"
        )

    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
