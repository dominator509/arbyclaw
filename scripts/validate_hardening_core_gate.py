#!/usr/bin/env python3
"""Run the strongest current local hardening core aggregate validation bundle.

This gate composes the existing local packaging/deployment aggregate gate, the
locked dependency license-policy validator, and the local secret/policy
boundary plus secret backup/restore, withdrawal-policy, signer-boundary, and
destination-boundary validators. It preserves local-only/non-secret behavior:
no live trading, no exchange/RPC calls, no service-manager actions, no
credential loading, no signing, publishing, withdrawals, transfers, or
production-readiness claims.
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
TIMEOUT_SECONDS = 2_400


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    parser.add_argument(
        "--require-systemd-analyze",
        action="store_true",
        help="require systemd-analyze inside the nested packaging gate",
    )
    return parser.parse_args()


def command_set(workspace_root: pathlib.Path, args: argparse.Namespace) -> list[tuple[str, list[str]]]:
    packaging = [sys.executable, "scripts/validate_packaging_deployment_gate.py", "--json"]
    if args.require_systemd_analyze:
        packaging.append("--require-systemd-analyze")
    return [
        ("packaging_deployment_gate", packaging),
        (
            "dependency_license_policy",
            [sys.executable, "scripts/validate_dependency_license_policy.py", "--json"],
        ),
        (
            "secret_boundary_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-secret-boundary-audit",
                "--workspace",
                str(workspace_root / "secret-boundary-audit"),
            ],
        ),
        (
            "secret_backup_restore",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-secret-backup-restore",
                "--workspace",
                str(workspace_root / "secret-backup-restore"),
            ],
        ),
        (
            "withdrawal_policy_boundary",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-withdrawal-policy-boundary",
                "--workspace",
                str(workspace_root / "withdrawal-policy-boundary"),
            ],
        ),
        (
            "signer_boundary_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-signer-boundary-audit",
                "--workspace",
                str(workspace_root / "signer-boundary-audit"),
            ],
        ),
        (
            "destination_boundary_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-destination-boundary-audit",
                "--workspace",
                str(workspace_root / "destination-boundary-audit"),
            ],
        ),
        (
            "policy_decision_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-policy-decision-audit",
                "--workspace",
                str(workspace_root / "policy-decision-audit"),
            ],
        ),
    ]


def parse_output(text: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or ": " not in line:
            continue
        key, value = line.split(": ", 1)
        parsed[key.strip()] = value.strip()
    return parsed


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
    json_report: dict[str, Any] | None = None
    parsed = parse_output(output)
    if "--json" in command and completed.returncode == 0:
        for index, line in enumerate(output.splitlines()):
            if line.lstrip().startswith("{"):
                candidate = "\n".join(output.splitlines()[index:])
                loaded = json.loads(candidate)
                if not isinstance(loaded, dict):
                    raise RuntimeError(f"{name} did not emit a JSON object")
                json_report = loaded
                break
        if json_report is None:
            raise RuntimeError(f"{name} did not emit JSON output")
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "json_report": json_report,
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

    if errors:
        return errors, bounded_toolchain_external_path_used

    packaging = component_by_name["packaging_deployment_gate"]["json_report"]
    assert packaging is not None
    if packaging.get("all_components_passed") is not True:
        errors.append("packaging deployment gate did not pass every component")
    if packaging.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("packaging deployment gate detected unsafe side-effect flags")
    for field in (
        "release_published",
        "deployment_performed",
        "service_installed",
        "service_actions_performed",
        "network_listeners_started",
        "secrets_loaded",
        "live_execution_enabled",
        "production_readiness_claimed",
        "arm_binary_executed",
        "device_inspected",
        "emulator_used",
    ):
        if packaging.get(field) is not False:
            errors.append(f"packaging deployment gate reported unsafe field {field}")
    bounded_toolchain_external_path_used = (
        packaging.get("bounded_toolchain_external_path_used") is True
    )

    dependency_license = component_by_name["dependency_license_policy"]["json_report"]
    assert dependency_license is not None
    if dependency_license.get("passed") is not True:
        errors.append("dependency license policy validation did not pass")
    if dependency_license.get("required_crates_present") is not True:
        errors.append("dependency license policy did not confirm required crates")
    if dependency_license.get("missing_license_count") not in {0, None}:
        errors.append("dependency license policy reported missing licenses")
    if dependency_license.get("denied_package_count") not in {0, None}:
        errors.append("dependency license policy reported denied packages")
    if dependency_license.get("workspace_license_violation_count") not in {0, None}:
        errors.append("dependency license policy reported workspace license violations")

    secret_boundary = component_by_name["secret_boundary_audit"]["parsed"]
    if secret_boundary.get("ready-rotation-plan") in {None, ""}:
        errors.append("secret boundary audit did not report a ready rotation plan")
    if secret_boundary.get("rejected-rotation-plan") in {None, ""}:
        errors.append("secret boundary audit did not report a rejected rotation plan")
    if secret_boundary.get("rejected-rotation-validation-codes") in {"0", None}:
        errors.append("secret boundary audit did not report rejected validation codes")
    if secret_boundary.get("audit-append-failure-failed-closed") != "true":
        errors.append("secret boundary audit did not fail closed on audit append failure")
    if secret_boundary.get("state-failure-failed-closed") != "true":
        errors.append("secret boundary audit did not fail closed on state failure")
    if secret_boundary.get("state-checkpoint-recovered") != "true":
        errors.append("secret boundary audit did not recover its state checkpoint")
    for key in (
        "secret-material-loaded",
        "plaintext-decrypted",
        "keystore-entry-written",
        "external-secret-revoked",
        "production-ready",
    ):
        if secret_boundary.get(key) != "false":
            errors.append(f"secret boundary audit reported unsafe field {key}")

    secret_backup_restore = component_by_name["secret_backup_restore"]["parsed"]
    if secret_backup_restore.get("ready-backup-restore-review") in {None, ""}:
        errors.append("secret backup/restore did not report a ready review")
    if secret_backup_restore.get("blocked-backup-restore-review") in {None, ""}:
        errors.append("secret backup/restore did not report a blocked review")
    if secret_backup_restore.get("blocked-backup-restore-validation-codes") in {"0", None}:
        errors.append("secret backup/restore did not report blocked validation codes")
    for key in (
        "backup-reference-present",
        "backup-payload-shape-verified",
        "restore-verification-passed",
        "references-sanitized",
        "review-window-valid",
        "audit-append-failure-failed-closed",
        "state-failure-failed-closed",
        "state-checkpoint-recovered",
    ):
        if secret_backup_restore.get(key) != "true":
            errors.append(f"secret backup/restore did not report {key}=true")
    if secret_backup_restore.get("audit-records-replayed") != "2":
        errors.append("secret backup/restore did not replay exactly two audit records")
    for key in (
        "secret-material-loaded",
        "plaintext-decrypted",
        "keystore-entry-written",
        "external-secret-restored",
        "signing-or-broadcast-performed",
        "production-ready",
    ):
        if secret_backup_restore.get(key) != "false":
            errors.append(f"secret backup/restore reported unsafe field {key}")

    withdrawal_policy = component_by_name["withdrawal_policy_boundary"]["parsed"]
    for key in (
        "config-guard-active",
        "strategy-flag-guard-active",
        "strategy-intent-guard-active",
        "trust-contract-guard-active",
        "destination-allowlist-guard-active",
        "signing-boundary-guard-active",
        "audit-append-failure-failed-closed",
        "state-failure-failed-closed",
        "state-checkpoint-recovered",
    ):
        if withdrawal_policy.get(key) != "true":
            errors.append(f"withdrawal policy boundary did not report {key}=true")
    if withdrawal_policy.get("audit-records-replayed") != "1":
        errors.append("withdrawal policy boundary did not replay exactly one audit record")
    for key in (
        "external-submission-performed",
        "secret-material-recorded",
        "production-ready",
    ):
        if withdrawal_policy.get(key) != "false":
            errors.append(f"withdrawal policy boundary reported unsafe field {key}")

    signer_boundary = component_by_name["signer_boundary_audit"]["parsed"]
    if signer_boundary.get("signer-request-status") != "RejectedSignerUnavailable":
        errors.append("signer boundary did not reject unavailable signer requests")
    if signer_boundary.get("signer-scope-status") != "ReadyForLocalReview":
        errors.append("signer boundary did not report signer scope ready for local review")
    for key in (
        "signer-request-audit-failed-closed",
        "signer-scope-audit-failed-closed",
        "state-failure-failed-closed",
        "state-checkpoints-recovered",
    ):
        if signer_boundary.get(key) != "true":
            errors.append(f"signer boundary did not report {key}=true")
    if signer_boundary.get("audit-records-replayed") != "2":
        errors.append("signer boundary did not replay exactly two audit records")
    for key in (
        "signer-material-loaded",
        "plaintext-decrypted",
        "signing-performed",
        "broadcast-performed",
        "rpc-called",
        "production-ready",
    ):
        if signer_boundary.get(key) != "false":
            errors.append(f"signer boundary reported unsafe field {key}")

    destination_boundary = component_by_name["destination_boundary_audit"]["parsed"]
    if destination_boundary.get("destination-allowlist-version") in {None, ""}:
        errors.append("destination boundary did not report an allowlist version")
    for key, expected in (
        ("destination-enabled-entry-count", "1"),
        ("destination-referenced-evidence-count", "1"),
        ("audit-records-replayed", "2"),
    ):
        if destination_boundary.get(key) != expected:
            errors.append(f"destination boundary did not report {key}={expected}")
    for key in (
        "destination-allowlist-audit-failed-closed",
        "destination-ownership-review-audit-failed-closed",
        "state-failure-failed-closed",
        "state-checkpoints-recovered",
    ):
        if destination_boundary.get(key) != "true":
            errors.append(f"destination boundary did not report {key}=true")
    for key in (
        "chain-ownership-verified",
        "signer-material-loaded",
        "challenge-signed",
        "production-ready",
    ):
        if destination_boundary.get(key) != "false":
            errors.append(f"destination boundary reported unsafe field {key}")

    policy_audit = component_by_name["policy_decision_audit"]["parsed"]
    if policy_audit.get("approved-policy-decision") != "true":
        errors.append("policy decision audit did not report an approved policy decision")
    if policy_audit.get("denied-policy-decision") != "true":
        errors.append("policy decision audit did not report a denied policy decision")
    if policy_audit.get("denied-policy-violations") in {"0", None}:
        errors.append("policy decision audit did not report denied policy violations")
    if policy_audit.get("audit-append-failure-failed-closed") != "true":
        errors.append("policy decision audit did not fail closed on audit append failure")
    if policy_audit.get("state-failure-failed-closed") != "true":
        errors.append("policy decision audit did not fail closed on state failure")
    if policy_audit.get("state-checkpoint-recovered") != "true":
        errors.append("policy decision audit did not recover its state checkpoint")
    for key in (
        "external-submission-performed",
        "secret-material-recorded",
        "production-ready",
    ):
        if policy_audit.get(key) != "false":
            errors.append(f"policy decision audit reported unsafe field {key}")

    return errors, bounded_toolchain_external_path_used


def main() -> int:
    args = parse_args()
    with tempfile.TemporaryDirectory(prefix="hardening-core-gate-", dir=ROOT / "target") as temp_dir:
        workspace_root = pathlib.Path(temp_dir)
        components = [
            run_component(name, command)
            for name, command in command_set(workspace_root, args)
        ]
    errors, bounded_toolchain_external_path_used = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    report = {
        "schema": "arbyclaw.hardening_core_aggregate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "bounded_toolchain_external_path_used": bounded_toolchain_external_path_used,
        "deployment_performed": False,
        "service_installed": False,
        "service_actions_performed": False,
        "network_listeners_started": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_evidence": [
            "SBOM reviewer sign-off and provenance review",
            "CodeQL upload processing or accepted local-SARIF-only governance review",
            "secret-pattern scan evidence on hosts where gitleaks is approved and installed",
            "deployment-surface validation under service lifecycle",
            "penetration, load, rollback, incident, and production-readiness reviews",
        ],
    }
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("hardening core aggregate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print(
            "bounded-toolchain-external-path-used: "
            f"{str(report['bounded_toolchain_external_path_used']).lower()}"
        )
        print("deployment-performed: false")
        print("service-installed: false")
        print("service-actions-performed: false")
        print("network-listeners-started: false")
        print("secrets-loaded: false")
        print("live-execution-enabled: false")
        print("production-readiness-claimed: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
