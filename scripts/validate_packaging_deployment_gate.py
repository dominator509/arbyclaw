#!/usr/bin/env python3
"""Run the strongest local packaging/deployment aggregate validation bundle.

This gate composes existing release-artifact, production-container, systemd
example, static deployment hardening, ARM profile, and ARM cross-target
validators. It preserves the local-only boundary: no signing, publishing,
deployment, service-manager actions, secret loading, ARM binary execution, or
production-readiness claims.
When the ARM cross-target check uses the documented host-or-Docker fallback
path, the gate records that bounded toolchain/dependency path separately rather
than treating it as deployment execution.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT_SECONDS = 2_400


def binary_name() -> str:
    return "arb-agent.exe" if os.name == "nt" else "arb-agent"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    parser.add_argument(
        "--require-systemd-analyze",
        action="store_true",
        help="fail if systemd-analyze is unavailable for the example unit check",
    )
    return parser.parse_args()


def release_command() -> list[str]:
    command = [sys.executable, "scripts/validate_release_artifact.py", "--json"]
    if (ROOT / "target" / "release" / binary_name()).exists():
        command.insert(2, "--skip-build")
    return command


def command_set(args: argparse.Namespace) -> list[tuple[str, list[str]]]:
    systemd = [sys.executable, "scripts/validate_systemd_example.py", "--json", "--systemd-analyze"]
    if args.require_systemd_analyze:
        systemd.append("--require-systemd-analyze")
    return [
        ("release_artifact", release_command()),
        (
            "production_container",
            [sys.executable, "scripts/validate_production_container.py", "--json"],
        ),
        ("systemd_example", systemd),
        (
            "deployment_static_hardening",
            [
                sys.executable,
                "scripts/validate_deployment_static_hardening.py",
                "--run-config-smoke",
                "--json",
            ],
        ),
        (
            "arm_build_profiles",
            [sys.executable, "scripts/validate_arm_build_profiles.py", "--json"],
        ),
        (
            "arm_cross_check",
            [sys.executable, "scripts/validate_arm_cross_check.py", "--json"],
        ),
    ]


def extract_json_report(output: str) -> dict[str, Any]:
    decoder = json.JSONDecoder()
    for index, char in enumerate(output):
        if char != "{":
            continue
        loaded, _ = decoder.raw_decode(output[index:])
        if not isinstance(loaded, dict):
            raise RuntimeError("validator did not emit a JSON object")
        return loaded
    raise RuntimeError("validator did not emit a JSON object")


def run_component(name: str, command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    parsed: dict[str, Any] | None = None
    try:
        parsed = extract_json_report(output)
    except (json.JSONDecodeError, RuntimeError):
        parsed = None
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "parsed": parsed,
        "output_tail": output.splitlines()[-20:],
    }


def validate_components(components: list[dict[str, Any]]) -> tuple[list[str], bool]:
    errors: list[str] = []
    bounded_toolchain_external_path_used = False
    component_by_name = {component["name"]: component for component in components}

    for component in components:
        if component["returncode"] != 0:
            errors.append(f"{component['name']} exited {component['returncode']}")
            errors.extend(f"{component['name']} output: {line}" for line in component["output_tail"])
        if component["parsed"] is None:
            errors.append(f"{component['name']} did not produce a JSON report")

    if errors:
        return errors, bounded_toolchain_external_path_used

    release = component_by_name["release_artifact"]["parsed"]
    if release.get("passed") is not True:
        errors.append("release artifact validation did not pass")
    if release.get("artifact_created") is not True:
        errors.append("release artifact bundle was not created")
    if release.get("manifest_created") is not True:
        errors.append("release artifact manifest was not created")
    if release.get("provenance_created") is not True:
        errors.append("release artifact provenance was not created")
    if release.get("bundle_integrity_verified") is not True:
        errors.append("release artifact bundle integrity was not verified")
    if release.get("smoke_contains_usage") is not True:
        errors.append("release artifact smoke output did not include usage")
    for field in (
        "signing_performed",
        "release_published",
        "deployment_performed",
        "external_calls_performed",
        "secrets_loaded",
        "production_readiness_claimed",
    ):
        if release.get(field) is not False:
            errors.append(f"release artifact reported unsafe field {field}")

    production_container = component_by_name["production_container"]["parsed"]
    if production_container.get("passed") is not True:
        errors.append("production container validation did not pass")
    if production_container.get("docker_validation_completed") is not True:
        errors.append("production container validation did not complete Docker validation")
    if production_container.get("hardened_runtime_smoke_passed") is not True:
        errors.append("production container validation did not pass hardened runtime smoke")
    for field in (
        "read_only_filesystem",
        "network_disabled",
        "capabilities_dropped",
        "no_new_privileges",
    ):
        if production_container.get(field) is not True:
            errors.append(f"production container validation did not confirm {field}")
    for field in (
        "deployment_performed",
        "service_installed",
        "network_listeners_started",
        "secrets_loaded",
        "live_execution_enabled",
        "production_readiness_claimed",
    ):
        if production_container.get(field) is not False:
            errors.append(f"production container reported unsafe field {field}")

    systemd = component_by_name["systemd_example"]["parsed"]
    if systemd.get("passed") is not True or systemd.get("static_validation_passed") is not True:
        errors.append("systemd example validation did not pass")
    if systemd.get("systemd_analyze_requested") is not True:
        errors.append("systemd example did not request syntax verification")
    if systemd.get("systemd_analyze_required") is True and systemd.get("systemd_analyze_verified") is not True:
        errors.append("systemd example required syntax verification but did not verify successfully")
    for field in (
        "service_actions_performed",
        "external_calls_performed",
        "secrets_loaded",
        "production_readiness_claimed",
    ):
        if systemd.get(field) is not False:
            errors.append(f"systemd example reported unsafe field {field}")

    static_hardening = component_by_name["deployment_static_hardening"]["parsed"]
    if static_hardening.get("passed") is not True:
        errors.append("deployment static hardening validation did not pass")
    if static_hardening.get("config_smoke_requested") is not True:
        errors.append("deployment static hardening did not request config smoke")
    config_smoke = static_hardening.get("config_smoke")
    if not isinstance(config_smoke, dict) or config_smoke.get("passed") is not True:
        errors.append("deployment static hardening config smoke did not pass")
    if static_hardening.get("production_container", {}).get("distroless_runtime") is not True:
        errors.append("production container hardening report lost distroless runtime validation")
    if static_hardening.get("production_container", {}).get("nonroot_runtime") is not True:
        errors.append("production container hardening report lost nonroot runtime validation")
    if static_hardening.get("systemd", {}).get("protect_system_strict") is not True:
        errors.append("systemd hardening report lost ProtectSystem=strict validation")
    for field in (
        "service_actions_performed",
        "network_listeners_started",
        "external_calls_performed",
        "secrets_loaded",
        "live_execution_enabled",
        "production_readiness_claimed",
    ):
        if static_hardening.get(field) is not False:
            errors.append(f"deployment static hardening reported unsafe field {field}")

    arm_profiles = component_by_name["arm_build_profiles"]["parsed"]
    if arm_profiles.get("passed") is not True:
        errors.append("ARM build profile validation did not pass")
    for field in (
        "cross_build_performed",
        "device_inspected",
        "emulator_used",
        "external_calls_performed",
        "secrets_loaded",
        "production_readiness_claimed",
    ):
        if arm_profiles.get(field) is not False:
            errors.append(f"ARM build profile validation reported unsafe field {field}")
    if arm_profiles.get("no_execution_claim") is not True:
        errors.append("ARM build profile validation lost no-execution claim enforcement")
    if arm_profiles.get("production_claim_denied") is not True:
        errors.append("ARM build profile validation lost production-claim denial")

    arm_cross = component_by_name["arm_cross_check"]["parsed"]
    if arm_cross.get("passed") is not True:
        errors.append("ARM cross-target check did not pass")
    if arm_cross.get("cargo_check_attempted") is not True:
        errors.append("ARM cross-target check did not attempt cargo check")
    if arm_cross.get("cargo_check_returncode") != 0:
        errors.append("ARM cross-target cargo check did not return success")
    if arm_cross.get("target_installed") is not True:
        errors.append("ARM cross-target check did not confirm target availability")
    if arm_cross.get("cross_compiler_available") is not True:
        errors.append("ARM cross-target check did not confirm cross compiler availability")
    for field in (
        "arm_binary_executed",
        "device_inspected",
        "emulator_used",
        "service_actions_performed",
        "secrets_loaded",
        "production_readiness_claimed",
    ):
        if arm_cross.get(field) is not False:
            errors.append(f"ARM cross-target check reported unsafe field {field}")
    external_calls_performed = arm_cross.get("external_calls_performed")
    docker_fallback_used = arm_cross.get("docker_fallback_used")
    target_install_attempted = arm_cross.get("target_install_attempted")
    if external_calls_performed is True:
        if docker_fallback_used is True or target_install_attempted is True:
            bounded_toolchain_external_path_used = True
        else:
            errors.append(
                "ARM cross-target check reported external calls without the documented host-or-Docker fallback path"
            )
    elif external_calls_performed is not False:
        errors.append("ARM cross-target check external call state was not boolean false/true")

    return errors, bounded_toolchain_external_path_used


def main() -> int:
    args = parse_args()
    components = [run_component(name, command) for name, command in command_set(args)]
    errors, bounded_toolchain_external_path_used = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    report = {
        "schema": "arbyclaw.packaging_deployment_aggregate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "bounded_toolchain_external_path_used": bounded_toolchain_external_path_used,
        "release_published": False,
        "deployment_performed": False,
        "service_installed": False,
        "service_actions_performed": False,
        "network_listeners_started": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
        "arm_binary_executed": False,
        "device_inspected": False,
        "emulator_used": False,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_evidence": [
            "signed release workflow and artifact repository retention review",
            "deployment-host systemd installation and hardened runtime validation",
            "ARM target-class runtime smoke and deployment validation",
            "rollback and incident execution evidence",
            "production readiness review",
        ],
    }
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("packaging deployment aggregate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print(
            "bounded-toolchain-external-path-used: "
            f"{str(report['bounded_toolchain_external_path_used']).lower()}"
        )
        print("release-published: false")
        print("deployment-performed: false")
        print("service-actions-performed: false")
        print("network-listeners-started: false")
        print("secrets-loaded: false")
        print("live-execution-enabled: false")
        print("production-readiness-claimed: false")
        print("arm-binary-executed: false")
        print("device-inspected: false")
        print("emulator-used: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
