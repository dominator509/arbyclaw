#!/usr/bin/env python3
"""Run the strongest current local execution-path aggregate validation bundle.

This gate composes local planner handoff, planner audit, adapter audit,
destination/signer controls, and the local Web3 non-broadcast review chain. It
preserves local-only/non-secret behavior: no external submission, no signing,
no broadcasts, no real RPC/exchange calls, and no production-readiness claims.
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT_SECONDS = 2_400


def command_specs(workspace_root: pathlib.Path) -> list[dict[str, Any]]:
    return [
        {
            "name": "opportunity_planner_handoff",
            "command": ["cargo", "run", "-p", "arb-agent", "--", "validate-opportunity-planner-handoff"],
            "exact_fields": {
                "opportunity-planner-handoff-status": "passed",
            },
            "nonzero_fields": (
                "discovered-candidates",
                "planned-candidates",
                "draft-ready-plans",
                "candidate-trace-audit-records",
                "candidate-trace-checkpoints",
                "total-intents",
            ),
            "false_fields": (
                "adapter-submission-enabled",
                "external-calls-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "strategy_constrained_planner",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-strategy-constrained-planner",
            ],
            "exact_fields": {
                "strategy-constrained-planner": "validation passed",
                "accepted-plan-status": "draft-ready",
                "rejected-plan-status": "policy-denied-draft",
                "accepted-strategy-rejected-intents": "0",
            },
            "nonzero_fields": (
                "accepted-intents",
                "rejected-strategy-rejected-intents",
            ),
            "false_fields": (
                "adapter-submission-performed",
                "live-execution-performed",
                "signing-or-broadcast-performed",
                "production-ready",
            ),
        },
        {
            "name": "execution_planner_audit",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-execution-planner-audit",
                "--workspace",
                str(workspace_root / "execution-planner-audit"),
            ],
            "exact_fields": {
                "execution-planner-audit": "validation passed",
            },
            "nonzero_fields": (
                "plan-intents",
                "plan-policy-outcomes",
                "plan-failure-modes",
                "audit-records-replayed",
            ),
            "true_fields": (
                "audit-append-failure-failed-closed",
                "state-failure-failed-closed",
                "state-checkpoints-recovered",
            ),
            "false_fields": (
                "adapter-submission-enabled",
                "external-submission-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "policy_decision_audit",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-policy-decision-audit",
                "--workspace",
                str(workspace_root / "policy-decision-audit"),
            ],
            "exact_fields": {
                "policy-decision-audit": "validation passed",
            },
            "nonzero_fields": (
                "denied-policy-violations",
                "audit-records-replayed",
            ),
            "true_fields": (
                "approved-policy-decision",
                "denied-policy-decision",
                "audit-append-failure-failed-closed",
                "state-failure-failed-closed",
                "state-checkpoint-recovered",
            ),
            "false_fields": (
                "external-submission-performed",
                "secret-material-recorded",
                "production-ready",
            ),
        },
        {
            "name": "destination_boundary_audit",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-destination-boundary-audit",
                "--workspace",
                str(workspace_root / "destination-boundary-audit"),
            ],
            "exact_fields": {
                "destination-boundary-audit": "validation passed",
            },
            "nonzero_fields": (
                "destination-enabled-entry-count",
                "destination-referenced-evidence-count",
                "audit-records-replayed",
            ),
            "true_fields": (
                "destination-allowlist-audit-failed-closed",
                "destination-ownership-review-audit-failed-closed",
                "state-failure-failed-closed",
                "state-checkpoints-recovered",
            ),
            "false_fields": (
                "chain-ownership-verified",
                "signer-material-loaded",
                "challenge-signed",
                "production-ready",
            ),
        },
        {
            "name": "execution_adapter_audit",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-execution-adapter-audit",
                "--workspace",
                str(workspace_root / "execution-adapter-audit"),
            ],
            "exact_fields": {
                "execution-adapter-audit": "validation passed",
            },
            "nonzero_fields": (
                "adapter-run-attempts",
                "adapter-run-fills",
                "adapter-run-reconciliations",
                "adapter-recovery-steps",
                "audit-records-replayed",
            ),
            "true_fields": (
                "adapter-policy-revalidated",
                "audit-append-failure-failed-closed",
                "recovery-audit-append-failure-failed-closed",
                "state-failure-failed-closed",
                "state-checkpoints-recovered",
            ),
            "false_fields": (
                "external-submission-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "signer_boundary_audit",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-signer-boundary-audit",
                "--workspace",
                str(workspace_root / "signer-boundary-audit"),
            ],
            "exact_fields": {
                "signer-boundary-audit": "validation passed",
            },
            "nonzero_fields": ("audit-records-replayed",),
            "true_fields": (
                "signer-request-audit-failed-closed",
                "signer-scope-audit-failed-closed",
                "state-failure-failed-closed",
                "state-checkpoints-recovered",
            ),
            "false_fields": (
                "signer-material-loaded",
                "plaintext-decrypted",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "production-ready",
            ),
        },
        {
            "name": "signer_runtime_isolation",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-signer-runtime-isolation",
            ],
            "exact_fields": {
                "signer-runtime-isolation": "validation passed",
            },
            "nonzero_fields": (
                "runtime-isolation-ready-count",
                "runtime-isolation-blocked-count",
                "runtime-isolation-blocker-count",
            ),
            "true_fields": (
                "llm-signer-access-denied",
                "plaintext-key-exposure-denied",
                "policy-destination-scope-required",
                "audit-state-before-signing-required",
            ),
            "false_fields": (
                "signer-material-loaded",
                "plaintext-decrypted",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "production-ready",
            ),
        },
        {
            "name": "signer_authorization_envelope",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-signer-authorization-envelope",
            ],
            "exact_fields": {
                "signer-authorization-envelope": "validation passed",
            },
            "nonzero_fields": (
                "signer-authorization-ready-count",
                "signer-authorization-blocked-count",
                "signer-authorization-blocker-count",
                "audit-records-replayed",
                "state-checkpoints-recovered",
            ),
            "true_fields": (
                "policy-destination-ready",
                "secret-scope-ready",
                "runtime-isolation-ready",
                "transaction-safety-references-ready",
                "audit-state-references-ready",
            ),
            "false_fields": (
                "signer-material-loaded",
                "plaintext-decrypted",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "production-ready",
            ),
        },
        {
            "name": "web3_nonce_reservation",
            "command": ["cargo", "run", "-p", "arb-agent", "--", "validate-web3-nonce-reservation"],
            "exact_fields": {
                "web3-nonce-reservation": "validation passed",
            },
            "false_fields": (
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_unsigned_payload_review",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-unsigned-payload-review",
            ],
            "exact_fields": {
                "web3-unsigned-payload-review": "validation passed",
            },
            "false_fields": (
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_pre_sign_safety",
            "command": ["cargo", "run", "-p", "arb-agent", "--", "validate-web3-pre-sign-safety"],
            "exact_fields": {
                "web3-pre-sign-safety": "validation passed",
            },
            "false_fields": (
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_broadcast_readiness",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-broadcast-readiness",
            ],
            "exact_fields": {
                "web3-broadcast-readiness": "validation passed",
            },
            "false_fields": (
                "broadcast-allowed",
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_unsigned_transaction_construction",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-unsigned-transaction-construction",
            ],
            "exact_fields": {
                "web3-unsigned-transaction-construction": "validation passed",
            },
            "false_fields": (
                "raw-calldata-embedded",
                "raw-transaction-serialized",
                "broadcast-allowed",
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "rpc-called",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_provider_nonce_reconciliation",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-provider-nonce-reconciliation",
            ],
            "exact_fields": {
                "web3-provider-nonce-reconciliation": "validation passed",
            },
            "false_fields": (
                "rpc-called",
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_raw_transaction_serialization_review",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-raw-transaction-serialization-review",
            ],
            "exact_fields": {
                "web3-raw-transaction-serialization-review": "validation passed",
            },
            "false_fields": (
                "raw-transaction-bytes-embedded",
                "raw-calldata-embedded",
                "raw-transaction-serialized",
                "broadcast-allowed",
                "rpc-called",
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_broadcast_adapter_control_review",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-broadcast-adapter-control-review",
            ],
            "exact_fields": {
                "web3-broadcast-adapter-control-review": "validation passed",
            },
            "false_fields": (
                "broadcast-permission-granted",
                "raw-transaction-bytes-embedded",
                "raw-transaction-serialized",
                "rpc-called",
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
        {
            "name": "web3_sandbox_live_discrepancy_calibration",
            "command": [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-web3-sandbox-live-discrepancy-calibration",
            ],
            "exact_fields": {
                "web3-sandbox-live-discrepancy-calibration": "validation passed",
            },
            "false_fields": (
                "external-call-performed",
                "credential-loaded",
                "rpc-called",
                "signer-material-loaded",
                "signing-performed",
                "broadcast-performed",
                "live-execution-performed",
                "production-ready",
            ),
        },
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


def run_component(spec: dict[str, Any]) -> dict[str, Any]:
    completed = subprocess.run(
        spec["command"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    return {
        "name": spec["name"],
        "command": " ".join(spec["command"]),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "parsed": parse_output(output),
        "spec": spec,
    }


def parse_positive_int(value: str | None) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def validate_component(component: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    name = component["name"]
    parsed = component["parsed"]
    spec = component["spec"]

    if component["returncode"] != 0:
        errors.append(f"{name} exited {component['returncode']}")
        return errors

    for key, expected in spec.get("exact_fields", {}).items():
        actual = parsed.get(key)
        if actual != expected:
            errors.append(f"{name} expected {key}={expected!r}, got {actual!r}")

    for key in spec.get("true_fields", ()):
        if parsed.get(key) != "true":
            errors.append(f"{name} expected {key}=true")

    for key in spec.get("false_fields", ()):
        if parsed.get(key) != "false":
            errors.append(f"{name} expected {key}=false")

    for key in spec.get("nonzero_fields", ()):
        actual = parse_positive_int(parsed.get(key))
        if actual is None or actual <= 0:
            errors.append(f"{name} expected positive integer {key}")

    return errors


def main() -> int:
    with tempfile.TemporaryDirectory(
        prefix="execution-path-gate-", dir=ROOT / "target"
    ) as temp_dir:
        workspace_root = pathlib.Path(temp_dir)
        components = [
            run_component(spec) for spec in command_specs(workspace_root)
        ]

    errors: list[str] = []
    for component in components:
        errors.extend(validate_component(component))

    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    report = {
        "schema": "arbyclaw.execution_path_aggregate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "external_calls_performed": False,
        "external_submission_performed": False,
        "signer_material_loaded": False,
        "plaintext_decrypted": False,
        "signing_performed": False,
        "broadcast_performed": False,
        "live_execution_performed": False,
        "production_ready": False,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_validation": [
            "sandbox/live adapter reconciliation",
            "live exchange/RPC adapter execution",
            "custody-backed signing and provider-backed nonce validation",
            "deployment-host restart/service-orchestrated runtime validation",
        ],
    }
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
