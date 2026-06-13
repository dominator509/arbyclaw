#!/usr/bin/env python3
"""Build and validate a non-secret ArbyClaw release artifact bundle.

This gate runs a locked release build, copies the `arb-agent` binary into an
ignored local artifact directory, writes a SHA-256 manifest plus unsigned
provenance record, and smoke-runs the copied binary help path. It does not sign
artifacts, publish releases, deploy services, call networks, load secrets, or
claim production readiness.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
ARTIFACT_DIR = ROOT / "target" / "release-artifacts"
MANIFEST_NAME = "arbyclaw-release-manifest.json"
PROVENANCE_NAME = "arbyclaw-release-provenance.json"
FALSE_CLAIM_FIELDS = [
    "release_published",
    "deployment_performed",
    "external_calls_performed",
    "secrets_loaded",
    "production_readiness_claimed",
]
MANIFEST_FALSE_CLAIM_FIELDS = ["signing_performed", *FALSE_CLAIM_FIELDS]
PROVENANCE_FALSE_CLAIM_FIELDS = [
    "provenance_signed",
    "attestation_uploaded",
    *FALSE_CLAIM_FIELDS,
]
RELEASE_BUILD_TIMEOUT_SECONDS = 600
RELEASE_SMOKE_TIMEOUT_SECONDS = 60
RELEASE_METADATA_TIMEOUT_SECONDS = 30


def bounded_timeouts() -> dict[str, int]:
    return {
        "release_build_seconds": RELEASE_BUILD_TIMEOUT_SECONDS,
        "release_smoke_seconds": RELEASE_SMOKE_TIMEOUT_SECONDS,
        "metadata_seconds": RELEASE_METADATA_TIMEOUT_SECONDS,
    }


def run(command: list[str], timeout_seconds: int) -> subprocess.CompletedProcess[str]:
    print(f"+ {' '.join(command)}", flush=True)
    try:
        return subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            encoding="utf-8",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        return subprocess.CompletedProcess(
            command,
            124,
            stdout=f"command timed out after {error.timeout} seconds: {' '.join(command)}\n",
            stderr=None,
        )


def binary_name() -> str:
    return "arb-agent.exe" if os.name == "nt" else "arb-agent"


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def command_text(command: list[str], timeout_seconds: int = RELEASE_METADATA_TIMEOUT_SECONDS) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            encoding="utf-8",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        return f"unavailable: {' '.join(command)} timed out after {error.timeout} seconds"
    if completed.returncode != 0:
        return f"unavailable: {' '.join(command)} exited {completed.returncode}"
    return completed.stdout.strip()


def workspace_member_metadata() -> list[dict[str, str]]:
    members: list[dict[str, str]] = []
    for cargo_toml in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        members.append(
            {
                "path": str(cargo_toml.parent.relative_to(ROOT)),
                "cargo_toml": str(cargo_toml.relative_to(ROOT)),
                "cargo_toml_sha256": sha256(cargo_toml),
            }
        )
    return members


def git_status() -> dict[str, Any]:
    head = command_text(["git", "rev-parse", "HEAD"])
    porcelain = command_text(["git", "status", "--short"])
    status_available = not head.startswith("unavailable:") and not porcelain.startswith("unavailable:")
    return {
        "head": head if status_available else "unavailable",
        "status_available": status_available,
        "source_tree_dirty": bool(porcelain.strip()) if status_available else None,
    }


def load_json(path: pathlib.Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        loaded = json.load(handle)
    if not isinstance(loaded, dict):
        raise ValueError(f"{path.name} must contain a JSON object")
    return loaded


def require_false_claims(record: dict[str, Any], fields: list[str], label: str) -> list[str]:
    failures: list[str] = []
    for field in fields:
        if record.get(field) is not False:
            failures.append(f"{label} field {field} must be false")
    return failures


def verify_bundle(
    artifact_binary: pathlib.Path, manifest_path: pathlib.Path, provenance_path: pathlib.Path
) -> list[str]:
    failures: list[str] = []
    if not artifact_binary.exists():
        failures.append("release artifact binary is missing")
    if not manifest_path.exists():
        failures.append("release artifact manifest is missing")
    if not provenance_path.exists():
        failures.append("release artifact provenance is missing")
    if failures:
        return failures

    try:
        manifest = load_json(manifest_path)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        failures.append(f"release artifact manifest is not valid JSON: {error}")
        manifest = {}
    try:
        provenance = load_json(provenance_path)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        failures.append(f"release artifact provenance is not valid JSON: {error}")
        provenance = {}

    if manifest:
        failures.extend(require_false_claims(manifest, MANIFEST_FALSE_CLAIM_FIELDS, "manifest"))
        if manifest.get("schema") != "arbyclaw.release_artifact_manifest.v1":
            failures.append("release artifact manifest schema mismatch")
        if manifest.get("artifact") != artifact_binary.name:
            failures.append("release artifact manifest artifact name mismatch")
        if manifest.get("artifact_sha256") != sha256(artifact_binary):
            failures.append("release artifact manifest sha256 mismatch")
        if manifest.get("artifact_size_bytes") != artifact_binary.stat().st_size:
            failures.append("release artifact manifest size mismatch")

    if provenance:
        failures.extend(require_false_claims(provenance, PROVENANCE_FALSE_CLAIM_FIELDS, "provenance"))
        if provenance.get("schema") != "arbyclaw.release_artifact_provenance.v1":
            failures.append("release artifact provenance schema mismatch")
        if provenance.get("artifact") != artifact_binary.name:
            failures.append("release artifact provenance artifact name mismatch")
        if provenance.get("artifact_sha256") != sha256(artifact_binary):
            failures.append("release artifact provenance artifact sha256 mismatch")
        if provenance.get("manifest") != MANIFEST_NAME:
            failures.append("release artifact provenance manifest name mismatch")
        if provenance.get("manifest_sha256") != sha256(manifest_path):
            failures.append("release artifact provenance manifest sha256 mismatch")

        source_inputs = provenance.get("source_inputs")
        if not isinstance(source_inputs, dict):
            failures.append("release artifact provenance source_inputs missing")
        else:
            expected_hashes = {
                "root_cargo_toml_sha256": ROOT / "Cargo.toml",
                "cargo_lock_sha256": ROOT / "Cargo.lock",
            }
            for field, path in expected_hashes.items():
                if source_inputs.get(field) != sha256(path):
                    failures.append(f"release artifact provenance {field} mismatch")
            members = source_inputs.get("workspace_members")
            if not isinstance(members, list) or not members:
                failures.append("release artifact provenance workspace_members missing")
            else:
                for member in members:
                    if not isinstance(member, dict):
                        failures.append("release artifact provenance workspace member must be an object")
                        continue
                    cargo_toml = member.get("cargo_toml")
                    cargo_hash = member.get("cargo_toml_sha256")
                    if not isinstance(cargo_toml, str) or not isinstance(cargo_hash, str):
                        failures.append("release artifact provenance workspace member metadata incomplete")
                        continue
                    path = ROOT / cargo_toml
                    if not path.exists() or cargo_hash != sha256(path):
                        failures.append(f"release artifact provenance workspace member hash mismatch: {cargo_toml}")

    return failures


def reset_artifact_dir() -> None:
    resolved = ARTIFACT_DIR.resolve()
    target_root = (ROOT / "target").resolve()
    if target_root not in resolved.parents:
        raise RuntimeError(f"refusing to clear unexpected artifact path: {resolved}")
    if ARTIFACT_DIR.exists():
        shutil.rmtree(ARTIFACT_DIR)
    ARTIFACT_DIR.mkdir(parents=True)


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    build = None
    if not args.skip_build:
        build = run(["cargo", "build", "--release", "--locked"], RELEASE_BUILD_TIMEOUT_SECONDS)
        if build.returncode != 0:
            return {
                "schema": "arbyclaw.release_artifact.v1",
                "passed": False,
                "failures": ["cargo release build failed"],
                "bounded_timeouts": bounded_timeouts(),
                "build_returncode": build.returncode,
                "build_output_tail": build.stdout.splitlines()[-20:],
                "artifact_created": False,
                "manifest_created": False,
                "provenance_created": False,
                "smoke_returncode": None,
                "signing_performed": False,
                "release_published": False,
                "deployment_performed": False,
                "external_calls_performed": False,
                "secrets_loaded": False,
                "production_readiness_claimed": False,
            }

    source_binary = ROOT / "target" / "release" / binary_name()
    if not source_binary.exists():
        raise RuntimeError(f"missing release binary: {source_binary.relative_to(ROOT)}")

    reset_artifact_dir()
    artifact_binary = ARTIFACT_DIR / binary_name()
    shutil.copy2(source_binary, artifact_binary)
    artifact_hash = sha256(artifact_binary)
    manifest = {
        "schema": "arbyclaw.release_artifact_manifest.v1",
        "artifact": artifact_binary.name,
        "artifact_sha256": artifact_hash,
        "artifact_size_bytes": artifact_binary.stat().st_size,
        "build_command": "cargo build --release --locked",
        "source_binary": str(source_binary.relative_to(ROOT)),
        "target_triple": platform.machine(),
        "signing_performed": False,
        "release_published": False,
        "deployment_performed": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }
    manifest_path = ARTIFACT_DIR / MANIFEST_NAME
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    manifest_hash = sha256(manifest_path)

    provenance = {
        "schema": "arbyclaw.release_artifact_provenance.v1",
        "artifact": artifact_binary.name,
        "artifact_sha256": artifact_hash,
        "manifest": MANIFEST_NAME,
        "manifest_sha256": manifest_hash,
        "source_inputs": {
            "root_cargo_toml": "Cargo.toml",
            "root_cargo_toml_sha256": sha256(ROOT / "Cargo.toml"),
            "cargo_lock": "Cargo.lock",
            "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
            "workspace_members": workspace_member_metadata(),
        },
        "toolchain": {
            "cargo_version": command_text(["cargo", "--version"]),
            "rustc_version": command_text(["rustc", "--version"]),
            "active_toolchain": command_text(["rustup", "show", "active-toolchain"]),
            "target_machine": platform.machine(),
            "platform": platform.platform(),
        },
        "git": git_status(),
        "build_command": "cargo build --release --locked",
        "provenance_signed": False,
        "attestation_uploaded": False,
        "release_published": False,
        "deployment_performed": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }
    provenance_path = ARTIFACT_DIR / PROVENANCE_NAME
    provenance_path.write_text(json.dumps(provenance, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    smoke = run([str(artifact_binary), "--help"], RELEASE_SMOKE_TIMEOUT_SECONDS)
    smoke_contains_usage = "usage: arb-agent [--config <path>]" in smoke.stdout
    failures: list[str] = []
    if smoke.returncode != 0:
        failures.append("release artifact smoke command failed")
    if not smoke_contains_usage:
        failures.append("release artifact smoke output did not include usage")
    bundle_failures = verify_bundle(artifact_binary, manifest_path, provenance_path)
    failures.extend(bundle_failures)

    return {
        "schema": "arbyclaw.release_artifact.v1",
        "artifact_dir": str(ARTIFACT_DIR.relative_to(ROOT)),
        "artifact": artifact_binary.name,
        "manifest": MANIFEST_NAME,
        "provenance": PROVENANCE_NAME,
        "artifact_sha256": artifact_hash,
        "manifest_sha256": manifest_hash,
        "artifact_size_bytes": artifact_binary.stat().st_size,
        "passed": not failures,
        "failures": failures,
        "bounded_timeouts": bounded_timeouts(),
        "build_returncode": None if build is None else build.returncode,
        "artifact_created": artifact_binary.exists(),
        "manifest_created": manifest_path.exists(),
        "provenance_created": provenance_path.exists(),
        "bundle_integrity_verified": not bundle_failures,
        "bundle_integrity_failures": bundle_failures,
        "smoke_returncode": smoke.returncode,
        "smoke_contains_usage": smoke_contains_usage,
        "signing_performed": False,
        "release_published": False,
        "deployment_performed": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "production_readiness_claimed": False,
    }


def print_text(report: dict[str, Any]) -> None:
    print("release artifact validation")
    print(f"passed: {str(report['passed']).lower()}")
    print(f"artifact-dir: {report.get('artifact_dir', '')}")
    print(f"artifact: {report.get('artifact', '')}")
    print(f"manifest: {report.get('manifest', '')}")
    print(f"provenance: {report.get('provenance', '')}")
    timeouts = report.get("bounded_timeouts", {})
    print(f"release-build-timeout-seconds: {timeouts.get('release_build_seconds')}")
    print(f"release-smoke-timeout-seconds: {timeouts.get('release_smoke_seconds')}")
    print(f"metadata-timeout-seconds: {timeouts.get('metadata_seconds')}")
    print(f"artifact-created: {str(report.get('artifact_created', False)).lower()}")
    print(f"manifest-created: {str(report.get('manifest_created', False)).lower()}")
    print(f"provenance-created: {str(report.get('provenance_created', False)).lower()}")
    print(f"bundle-integrity-verified: {str(report.get('bundle_integrity_verified', False)).lower()}")
    print(f"smoke-returncode: {report.get('smoke_returncode')}")
    print(f"smoke-contains-usage: {str(report.get('smoke_contains_usage', False)).lower()}")
    print("signing-performed: false")
    print("release-published: false")
    print("deployment-performed: false")
    print("external-calls-performed: false")
    print("secrets-loaded: false")
    print("production-readiness-claimed: false")
    for failure in report["failures"]:
        print(f"failure: {failure}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON report")
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="use an existing target/release/arb-agent binary",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text(report)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
