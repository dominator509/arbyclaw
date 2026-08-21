#!/usr/bin/env python3
"""Run the single-owner local hardening aggregate.

This gate composes each lower aggregate once, plus the unique secret lifecycle,
secret backup/restore, withdrawal-policy, and dependency-license checks that do
not belong to another aggregate. Policy, destination, and signer boundary
commands are owned by `validate_execution_path_gate.py` and are intentionally
not re-executed here.
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
        ("execution_path_gate", [sys.executable, "scripts/validate_execution_path_gate.py", "--json"]),
        ("operator_surface_gate", [sys.executable, "scripts/validate_operator_surface_gate.py", "--json"]),
        ("opportunity_scenario_gate", [sys.executable, "scripts/validate_opportunity_scenario_gate.py", "--json"]),
        ("connector_scenario_gate", [sys.executable, "scripts/validate_connector_scenario_gate.py", "--json"]),
        ("deployment_evidence_checklist", [sys.executable, "scripts/validate_deployment_evidence_checklist.py", "--json"]),
        ("dependency_license_policy", [sys.executable, "scripts/validate_dependency_license_policy.py", "--json"]),
        (
            "secret_boundary_audit",
            [
                "cargo", "run", "-p", "arb-agent", "--", "validate-secret-boundary-audit",
                "--workspace", str(workspace_root / "secret-boundary-audit"),
            ],
        ),
        (
            "secret_backup_restore",
            [
                "cargo", "run", "-p", "arb-agent", "--", "validate-secret-backup-restore",
                "--workspace", str(workspace_root / "secret-backup-restore"),
            ],
        ),
        (
            "withdrawal_policy_boundary",
            [
                "cargo", "run", "-p", "arb-agent", "--", "validate-withdrawal-policy-boundary",
                "--workspace", str(workspace_root / "withdrawal-policy-boundary"),
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


def extract_json_report(output: str) -> dict[str, Any] | None:
    decoder = json.JSONDecoder()
    for index, char in enumerate(output):
        if char != "{":
            continue
        try:
            loaded, _ = decoder.raw_decode(output[index:])
        except json.JSONDecodeError:
            continue
        if isinstance(loaded, dict):
            return loaded
    return None


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
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "json_report": extract_json_report(output) if "--json" in command else None,
        "parsed": parse_output(output),
        "output_tail": output.splitlines()[-30:],
    }


def require_false(report: dict[str, Any], fields: tuple[str, ...], errors: list[str], prefix: str) -> None:
    for field in fields:
        if report.get(field) is not False:
            errors.append(f"{prefix} reported unsafe field {field}")


def validate_components(components: list[dict[str, Any]]) -> tuple[list[str], bool]:
    errors: list[str] = []
    by_name = {component["name"]: component for component in components}
    expected_names = {
        "packaging_deployment_gate",
        "execution_path_gate",
        "operator_surface_gate",
        "opportunity_scenario_gate",
        "connector_scenario_gate",
        "deployment_evidence_checklist",
        "dependency_license_policy",
        "secret_boundary_audit",
        "secret_backup_restore",
        "withdrawal_policy_boundary",
    }
    if len(components) != 10 or set(by_name) != expected_names:
        errors.append("hardening command graph drifted; expected exactly ten single-owner components")
        return errors, False

    for component in components:
        if component["returncode"] != 0:
            errors.append(f"{component['name']} exited {component['returncode']}")
            errors.extend(f"{component['name']} output: {line}" for line in component["output_tail"])
    if errors:
        return errors, False

    packaging = by_name["packaging_deployment_gate"]["json_report"]
    if not isinstance(packaging, dict):
        errors.append("packaging deployment gate did not emit JSON")
        packaging = {}
    if packaging.get("all_components_passed") is not True:
        errors.append("packaging deployment gate did not pass every component")
    if packaging.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("packaging deployment gate detected unsafe side-effect flags")
    require_false(
        packaging,
        (
            "release_published", "deployment_performed", "service_installed",
            "service_actions_performed", "network_listeners_started", "secrets_loaded",
            "live_execution_enabled", "production_readiness_claimed", "arm_binary_executed",
            "device_inspected", "emulator_used",
        ),
        errors,
        "packaging deployment gate",
    )
    bounded_toolchain_external_path_used = packaging.get("bounded_toolchain_external_path_used") is True

    execution = by_name["execution_path_gate"]["json_report"]
    if not isinstance(execution, dict):
        errors.append("execution path gate did not emit JSON")
        execution = {}
    if execution.get("all_components_passed") is not True or execution.get("component_count") != 18:
        errors.append("execution path gate did not preserve its 18-component passing contract")
    if execution.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("execution path gate detected unsafe side-effect flags")
    require_false(
        execution,
        (
            "external_calls_performed", "external_submission_performed", "signer_material_loaded",
            "plaintext_decrypted", "signing_performed", "broadcast_performed",
            "live_execution_performed", "production_ready",
        ),
        errors,
        "execution path gate",
    )

    operator = by_name["operator_surface_gate"]["json_report"]
    if not isinstance(operator, dict):
        errors.append("operator surface gate did not emit JSON")
        operator = {}
    if operator.get("all_components_passed") is not True or operator.get("component_count") != 17:
        errors.append("operator surface gate did not preserve its 17-component passing contract")
    if operator.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("operator surface gate detected unsafe side-effect flags")
    require_false(
        operator,
        (
            "outbound_network_used", "public_network_exposed", "service_actions_performed",
            "external_submission_performed", "signing_or_broadcast_performed",
            "live_execution_performed", "production_ready",
        ),
        errors,
        "operator surface gate",
    )

    opportunity = by_name["opportunity_scenario_gate"]["json_report"]
    if not isinstance(opportunity, dict):
        errors.append("opportunity scenario gate did not emit JSON")
        opportunity = {}
    if opportunity.get("all_components_passed") is not True or opportunity.get("component_count") != 14:
        errors.append("opportunity scenario gate did not preserve its 14-component passing contract")
    if opportunity.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("opportunity scenario gate detected unsafe side-effect flags")
    if opportunity.get("opportunity_replay_latency_review_enforced") is not True:
        errors.append("opportunity scenario gate lost replay latency review")
    if opportunity.get("local_validation_coverage_review_enforced") is not True:
        errors.append("opportunity scenario gate lost local validation coverage review")
    if not isinstance(opportunity.get("total_candidate_mentions"), int) or opportunity.get("total_candidate_mentions", 0) <= 0:
        errors.append("opportunity scenario gate lost candidate coverage")
    require_false(
        opportunity,
        (
            "external_calls_performed", "external_data_downloaded", "adapter_submission_performed",
            "external_fuzzer_invoked", "live_network_used", "signing_or_broadcast_performed",
            "live_execution_performed", "production_ready",
        ),
        errors,
        "opportunity scenario gate",
    )

    connector = by_name["connector_scenario_gate"]["json_report"]
    if not isinstance(connector, dict):
        errors.append("connector scenario gate did not emit JSON")
        connector = {}
    if connector.get("all_components_passed") is not True or connector.get("component_count") != 25:
        errors.append("connector scenario gate did not preserve its 25-component passing contract")
    if connector.get("unsafe_side_effect_flags_detected") is not False:
        errors.append("connector scenario gate detected unsafe side-effect flags")
    for field in (
        "fee_live_provider_boundary_enforced",
        "fee_schedule_reconciliation_review_enforced",
        "web3_provider_nonce_reconciliation_enforced",
        "web3_sandbox_live_discrepancy_calibration_enforced",
        "market_data_bad_data_rejection_review_enforced",
        "market_data_live_provider_boundary_enforced",
        "market_data_provider_latency_review_enforced",
        "market_data_provider_reconciliation_review_enforced",
    ):
        if connector.get(field) is not True:
            errors.append(f"connector scenario gate lost enforcement field {field}")
    if not isinstance(connector.get("audit_records_replayed"), int) or connector.get("audit_records_replayed", 0) <= 0:
        errors.append("connector scenario gate lost audit replay coverage")
    require_false(
        connector,
        (
            "live_network_used", "credential_loaded", "websocket_connection_opened",
            "live_provider_call_performed", "external_submission_performed", "rpc_call_performed",
            "signing_or_broadcast_performed", "live_execution_performed", "production_ready",
        ),
        errors,
        "connector scenario gate",
    )

    checklist = by_name["deployment_evidence_checklist"]["json_report"]
    if not isinstance(checklist, dict):
        errors.append("deployment evidence checklist did not emit JSON")
        checklist = {}
    bundle = checklist.get("bundle_index")
    if not isinstance(bundle, dict):
        errors.append("deployment evidence checklist did not report bundle_index")
        bundle = {}
    if bundle.get("all_components_passed") is not True:
        errors.append("deployment evidence bundle did not pass every component")
    if bundle.get("component_count") != 29:
        errors.append("deployment evidence bundle must contain exactly the 29 required deployment components")
    if bundle.get("missing_required_component_names") != []:
        errors.append("deployment evidence bundle is missing required component names")
    if checklist.get("all_external_evidence_referenced") is not False:
        errors.append("deployment evidence checklist unexpectedly claims complete external evidence")
    missing = checklist.get("remaining_missing_categories")
    if not isinstance(missing, list) or len(missing) != 11:
        errors.append("deployment evidence checklist must retain 11 external evidence categories")
    require_false(
        checklist,
        (
            "production_readiness_claimed", "service_actions_performed", "files_changed",
            "secrets_loaded", "external_calls_performed", "alerts_sent",
            "live_execution_enabled", "artifact_contents_embedded",
        ),
        errors,
        "deployment evidence checklist",
    )

    license_report = by_name["dependency_license_policy"]["json_report"]
    if not isinstance(license_report, dict):
        errors.append("dependency license policy did not emit JSON")
        license_report = {}
    if license_report.get("passed") is not True or license_report.get("required_crates_present") is not True:
        errors.append("dependency license policy did not pass required-crate validation")
    for field in ("missing_license_count", "denied_package_count", "workspace_license_violation_count"):
        if license_report.get(field) not in {0, None}:
            errors.append(f"dependency license policy reported nonzero {field}")

    secret = by_name["secret_boundary_audit"]["parsed"]
    if secret.get("ready-rotation-plan") in {None, ""} or secret.get("rejected-rotation-plan") in {None, ""}:
        errors.append("secret boundary audit lost ready/rejected rotation review coverage")
    if secret.get("rejected-rotation-validation-codes") in {None, "0"}:
        errors.append("secret boundary audit lost rejected validation codes")
    for field in ("audit-append-failure-failed-closed", "state-failure-failed-closed", "state-checkpoint-recovered"):
        if secret.get(field) != "true":
            errors.append(f"secret boundary audit expected {field}=true")
    for field in ("secret-material-loaded", "plaintext-decrypted", "keystore-entry-written", "external-secret-revoked", "production-ready"):
        if secret.get(field) != "false":
            errors.append(f"secret boundary audit reported unsafe field {field}")

    backup = by_name["secret_backup_restore"]["parsed"]
    if backup.get("ready-backup-restore-review") in {None, ""} or backup.get("blocked-backup-restore-review") in {None, ""}:
        errors.append("secret backup/restore lost ready/blocked review coverage")
    if backup.get("blocked-backup-restore-validation-codes") in {None, "0"}:
        errors.append("secret backup/restore lost blocked validation codes")
    for field in (
        "backup-reference-present", "backup-payload-shape-verified", "restore-verification-passed",
        "references-sanitized", "review-window-valid", "audit-append-failure-failed-closed",
        "state-failure-failed-closed", "state-checkpoint-recovered",
    ):
        if backup.get(field) != "true":
            errors.append(f"secret backup/restore expected {field}=true")
    if backup.get("audit-records-replayed") != "2":
        errors.append("secret backup/restore must replay exactly two audit records")
    for field in (
        "secret-material-loaded", "plaintext-decrypted", "keystore-entry-written",
        "external-secret-restored", "signing-or-broadcast-performed", "production-ready",
    ):
        if backup.get(field) != "false":
            errors.append(f"secret backup/restore reported unsafe field {field}")

    withdrawal = by_name["withdrawal_policy_boundary"]["parsed"]
    for field in (
        "config-guard-active", "strategy-flag-guard-active", "strategy-intent-guard-active",
        "trust-contract-guard-active", "destination-allowlist-guard-active",
        "signing-boundary-guard-active", "audit-append-failure-failed-closed",
        "state-failure-failed-closed", "state-checkpoint-recovered",
    ):
        if withdrawal.get(field) != "true":
            errors.append(f"withdrawal policy boundary expected {field}=true")
    if withdrawal.get("audit-records-replayed") != "1":
        errors.append("withdrawal policy boundary must replay exactly one audit record")
    for field in ("external-submission-performed", "secret-material-recorded", "production-ready"):
        if withdrawal.get(field) != "false":
            errors.append(f"withdrawal policy boundary reported unsafe field {field}")

    return errors, bounded_toolchain_external_path_used


def main() -> int:
    args = parse_args()
    (ROOT / "target").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="hardening-core-gate-", dir=ROOT / "target") as temp_dir:
        components = [
            run_component(name, command)
            for name, command in command_set(pathlib.Path(temp_dir), args)
        ]

    errors, bounded_toolchain_external_path_used = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    report = {
        "schema": "arbyclaw.hardening_core_aggregate_gate.v2",
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
            {"name": component["name"], "returncode": component["returncode"], "passed": component["passed"]}
            for component in components
        ],
        "remaining_external_evidence": [
            "SBOM reviewer sign-off and provenance review",
            "CodeQL processing and security review",
            "secret-pattern scan evidence",
            "deployment-surface validation under real service lifecycle",
            "penetration, load, rollback, incident, and production-readiness reviews",
        ],
    }
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("hardening core aggregate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print(f"bounded-toolchain-external-path-used: {str(bounded_toolchain_external_path_used).lower()}")
        print("production-readiness-claimed: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
