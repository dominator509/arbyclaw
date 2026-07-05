#!/usr/bin/env python3
"""Run the strongest local connector scenario validation bundle.

This gate composes existing market-data, fee, CEX/DEX request-plan, and CEX/DEX
connector lifecycle CLI probes. It validates only local deterministic fixtures and fails if any
nested command reports live network use, credential loading, RPC calls, signing,
broadcasts, external submission, live execution, or production readiness.
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
TIMEOUT_SECONDS = 900

DANGEROUS_TRUE_KEYS = {
    "account-state-queried",
    "broadcast-performed",
    "credential-loaded",
    "external-calls-performed",
    "external-execution-performed",
    "external-submission-performed",
    "live-execution-performed",
    "live-network-used",
    "live-provider-call-performed",
    "production-ready",
    "rpc-call-performed",
    "signing-or-broadcast-performed",
    "signing-performed",
    "signer-material-loaded",
    "websocket-connection-opened",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    return parser.parse_args()


def parse_output(text: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or ": " not in line:
            continue
        key, value = line.split(": ", 1)
        parsed[key.strip()] = value.strip()
    return parsed


def bool_value(value: str) -> bool | None:
    lowered = value.strip().lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    return None


def command_set(workspace_root: pathlib.Path) -> list[tuple[str, list[str]]]:
    return [
        (
            "market_data_provider_preflight",
            ["cargo", "run", "-p", "arb-agent", "--", "validate-market-data-provider-preflight"],
        ),
        (
            "market_data_provider_reconciliation",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-market-data-provider-reconciliation",
            ],
        ),
        (
            "market_data_reconnect_plan",
            ["cargo", "run", "-p", "arb-agent", "--", "validate-market-data-reconnect-plan"],
        ),
        (
            "market_data_quality_assessment",
            ["cargo", "run", "-p", "arb-agent", "--", "validate-market-data-quality-assessment"],
        ),
        (
            "market_data_bad_data_rejection",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-market-data-bad-data-rejection",
            ],
        ),
        (
            "paid_market_data_provider_evaluation",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-paid-market-data-provider-evaluation",
            ],
        ),
        (
            "market_data_boundary_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-market-data-boundary-audit",
                "--workspace",
                str(workspace_root / "market-data-boundary"),
            ],
        ),
        (
            "market_data_history_persistence",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-market-data-history-persistence",
                "--workspace",
                str(workspace_root / "market-data-history"),
            ],
        ),
        (
            "fee_schedule_verification",
            ["cargo", "run", "-p", "arb-agent", "--", "validate-fee-schedule-verification"],
        ),
        (
            "fee_schedule_reconciliation",
            ["cargo", "run", "-p", "arb-agent", "--", "validate-fee-schedule-reconciliation"],
        ),
        (
            "fee_boundary_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-fee-boundary-audit",
                "--workspace",
                str(workspace_root / "fee-boundary"),
            ],
        ),
        (
            "cex_governance_review",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-cex-governance-review",
            ],
        ),
        (
            "cex_market_data_request_plans",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-cex-market-data-request-plans",
            ],
        ),
        (
            "cex_balance_snapshots",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-cex-balance-snapshots",
            ],
        ),
        (
            "dex_request_plans",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-dex-request-plans",
            ],
        ),
        (
            "dex_response_transcripts",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-dex-response-transcripts",
            ],
        ),
        (
            "dex_transaction_lifecycle_transcripts",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-dex-transaction-lifecycle-transcripts",
            ],
        ),
        (
            "dex_protocol_risk_review",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-dex-protocol-risk-review",
            ],
        ),
        (
            "connector_lifecycle_audit",
            [
                "cargo",
                "run",
                "-p",
                "arb-agent",
                "--",
                "validate-connector-lifecycle-audit",
                "--workspace",
                str(workspace_root / "connector-lifecycle"),
            ],
        ),
    ]


def run_component(name: str, command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    output = f"{completed.stdout}\n{completed.stderr}".strip()
    parsed = parse_output(output)
    return {
        "name": name,
        "command": " ".join(command),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "parsed": parsed,
    }


def require(parsed: dict[str, str], key: str, expected: str, errors: list[str], name: str) -> None:
    if parsed.get(key) != expected:
        errors.append(f"{name} expected {key}: {expected}, got {parsed.get(key)!r}")


def validate_components(components: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    by_name = {component["name"]: component for component in components}

    for component in components:
        name = component["name"]
        if component["returncode"] != 0:
            errors.append(f"{name} exited {component['returncode']}")
        parsed = component["parsed"]
        for key in DANGEROUS_TRUE_KEYS:
            value = parsed.get(key)
            if value is not None and bool_value(value) is True:
                errors.append(f"{name} reported unsafe side-effect flag {key}: true")

    preflight = by_name["market_data_provider_preflight"]["parsed"]
    require(preflight, "clean-provider-status", "usable", errors, "market_data_provider_preflight")
    require(preflight, "degraded-provider-status", "blocked", errors, "market_data_provider_preflight")
    for key in ("rate-limit-blocked", "outage-blocked", "stale-data-blocked", "latency-blocked"):
        require(preflight, key, "true", errors, "market_data_provider_preflight")
    require(
        preflight,
        "market-data-provider-latency-review",
        "ready-for-local-review",
        errors,
        "market_data_provider_preflight",
    )
    for key in (
        "market-data-provider-latency-budget-met",
        "market-data-provider-capture-latency-budget-met",
        "market-data-provider-reconnect-delay-budget-met",
        "market-data-provider-quality-review-ready",
        "market-data-provider-paid-review-ready",
    ):
        require(preflight, key, "true", errors, "market_data_provider_preflight")
    require(
        preflight,
        "market-data-provider-latency-review-remaining-external-evidence-count",
        "5",
        errors,
        "market_data_provider_preflight",
    )

    reconciliation = by_name["market_data_provider_reconciliation"]["parsed"]
    require(
        reconciliation,
        "market-data-provider-reconciliation-review",
        "ready-for-local-review",
        errors,
        "market_data_provider_reconciliation",
    )
    for key in (
        "provider-reconciliation-latency-review-ready",
        "provider-reconciliation-rate-limit-fail-closed",
        "provider-reconciliation-outage-fail-closed",
        "provider-reconciliation-stale-data-fail-closed",
        "provider-reconciliation-latency-fail-closed",
        "provider-reconciliation-degraded-sample-floor-met",
        "provider-reconciliation-rate-limit-reconnect-ready",
        "provider-reconciliation-outage-reconnect-blocked",
    ):
        require(reconciliation, key, "true", errors, "market_data_provider_reconciliation")
    require(
        reconciliation,
        "provider-reconciliation-remaining-external-evidence-count",
        "5",
        errors,
        "market_data_provider_reconciliation",
    )

    reconnect = by_name["market_data_reconnect_plan"]["parsed"]
    require(reconnect, "ready-plan-status", "ready-for-local-review", errors, "market_data_reconnect_plan")
    require(reconnect, "blocked-plan-status", "blocked", errors, "market_data_reconnect_plan")
    require(reconnect, "outage-blocked", "true", errors, "market_data_reconnect_plan")
    require(reconnect, "retry-budget-exhausted", "true", errors, "market_data_reconnect_plan")

    quality = by_name["market_data_quality_assessment"]["parsed"]
    require(quality, "acceptable-status", "acceptable", errors, "market_data_quality_assessment")
    require(quality, "degraded-status", "degraded", errors, "market_data_quality_assessment")
    require(quality, "blocked-status", "blocked", errors, "market_data_quality_assessment")
    require(
        quality,
        "acceptable-quality-score",
        "100",
        errors,
        "market_data_quality_assessment",
    )
    require(
        quality,
        "acceptable-depth-levels",
        "2",
        errors,
        "market_data_quality_assessment",
    )
    require(
        quality,
        "blocked-violation-codes",
        "3",
        errors,
        "market_data_quality_assessment",
    )

    bad_data = by_name["market_data_bad_data_rejection"]["parsed"]
    require(
        bad_data,
        "market-data-bad-data-rejection-review",
        "ready-for-local-review",
        errors,
        "market_data_bad_data_rejection",
    )
    for key in (
        "bad-data-acceptable-quality-ready",
        "bad-data-stale-data-rejected",
        "bad-data-spread-rejected",
        "bad-data-depth-rejected",
        "bad-data-capture-latency-rejected",
        "bad-data-fixture-floor-met",
    ):
        require(bad_data, key, "true", errors, "market_data_bad_data_rejection")
    require(
        bad_data,
        "bad-data-remaining-external-evidence-count",
        "4",
        errors,
        "market_data_bad_data_rejection",
    )

    paid_provider = by_name["paid_market_data_provider_evaluation"]["parsed"]
    require(
        paid_provider,
        "ready-provider-status",
        "ready-for-local-review",
        errors,
        "paid_market_data_provider_evaluation",
    )
    require(
        paid_provider,
        "blocked-provider-status",
        "blocked",
        errors,
        "paid_market_data_provider_evaluation",
    )
    require(
        paid_provider,
        "ready-covered-venues",
        "2",
        errors,
        "paid_market_data_provider_evaluation",
    )
    require(
        paid_provider,
        "ready-covered-pairs",
        "2",
        errors,
        "paid_market_data_provider_evaluation",
    )
    require(
        paid_provider,
        "blocked-provider-violation-codes",
        "5",
        errors,
        "paid_market_data_provider_evaluation",
    )
    for key in (
        "latency-within-budget",
        "rate-limit-review-passed",
        "cost-review-passed",
        "failure-behavior-review-passed",
        "governance-review-passed",
    ):
        require(
            paid_provider,
            key,
            "true",
            errors,
            "paid_market_data_provider_evaluation",
        )

    market_audit = by_name["market_data_boundary_audit"]["parsed"]
    require(market_audit, "clean-provider-status", "usable", errors, "market_data_boundary_audit")
    require(market_audit, "degraded-provider-status", "blocked", errors, "market_data_boundary_audit")
    require(market_audit, "ready-reconnect-status", "ready-for-local-review", errors, "market_data_boundary_audit")
    require(market_audit, "blocked-reconnect-status", "blocked", errors, "market_data_boundary_audit")
    for key in ("preflight-audit-failed-closed", "reconnect-audit-failed-closed", "state-failure-failed-closed", "state-checkpoints-recovered"):
        require(market_audit, key, "true", errors, "market_data_boundary_audit")

    history = by_name["market_data_history_persistence"]["parsed"]
    require(
        history,
        "history-batch-status",
        "persisted-for-local-replay",
        errors,
        "market_data_history_persistence",
    )
    require(history, "stored-quote-count", "2", errors, "market_data_history_persistence")
    require(history, "stored-order-book-count", "2", errors, "market_data_history_persistence")
    require(history, "quotes-truncated", "true", errors, "market_data_history_persistence")
    require(history, "order-books-truncated", "true", errors, "market_data_history_persistence")
    require(history, "audit-failed-closed", "true", errors, "market_data_history_persistence")
    require(history, "state-failed-closed", "true", errors, "market_data_history_persistence")
    require(history, "audit-records-replayed", "1", errors, "market_data_history_persistence")
    require(
        history,
        "state-checkpoints-recovered",
        "true",
        errors,
        "market_data_history_persistence",
    )

    fee = by_name["fee_schedule_verification"]["parsed"]
    require(fee, "current-fee-review-status", "ready-for-local-review", errors, "fee_schedule_verification")
    require(fee, "blocked-fee-review-status", "blocked", errors, "fee_schedule_verification")
    require(fee, "stale-review-blocked", "true", errors, "fee_schedule_verification")

    fee_reconciliation = by_name["fee_schedule_reconciliation"]["parsed"]
    require(
        fee_reconciliation,
        "fee-schedule-reconciliation-review",
        "ready-for-local-review",
        errors,
        "fee_schedule_reconciliation",
    )
    for key in (
        "fee-reconciliation-current-review-ready",
        "fee-reconciliation-unverified-schedule-blocked",
        "fee-reconciliation-maker-taker-unverified-blocked",
        "fee-reconciliation-network-fee-unverified-blocked",
        "fee-reconciliation-withdrawal-fee-unreviewed-blocked",
        "fee-reconciliation-stale-review-blocked",
    ):
        require(fee_reconciliation, key, "true", errors, "fee_schedule_reconciliation")
    require(
        fee_reconciliation,
        "fee-reconciliation-remaining-external-evidence-count",
        "4",
        errors,
        "fee_schedule_reconciliation",
    )

    fee_audit = by_name["fee_boundary_audit"]["parsed"]
    require(fee_audit, "current-fee-review-status", "ready-for-local-review", errors, "fee_boundary_audit")
    require(fee_audit, "blocked-fee-review-status", "blocked", errors, "fee_boundary_audit")
    require(fee_audit, "fee-verification-audit-failed-closed", "true", errors, "fee_boundary_audit")
    require(fee_audit, "state-failure-failed-closed", "true", errors, "fee_boundary_audit")
    require(fee_audit, "state-checkpoints-recovered", "true", errors, "fee_boundary_audit")

    cex_governance = by_name["cex_governance_review"]["parsed"]
    require(cex_governance, "scope-ready-count", "1", errors, "cex_governance_review")
    require(cex_governance, "scope-blocked-count", "1", errors, "cex_governance_review")
    require(cex_governance, "rate-limit-ready-count", "1", errors, "cex_governance_review")
    require(cex_governance, "rate-limit-blocked-count", "1", errors, "cex_governance_review")
    for key in (
        "fee-review-ready",
        "rate-limit-documentation-ready",
        "terms-review-ready",
        "jurisdiction-review-ready",
        "api-capabilities-ready",
        "incident-review-ready",
        "governance-review-ready",
        "credential-reference-validated",
        "rate-limit-budget-blocked",
        "rate-limit-provider-blocked",
    ):
        require(cex_governance, key, "true", errors, "cex_governance_review")

    cex_request = by_name["cex_market_data_request_plans"]["parsed"]
    require(cex_request, "request-plan-count", "6", errors, "cex_market_data_request_plans")
    require(cex_request, "rest-request-plan-count", "3", errors, "cex_market_data_request_plans")
    require(cex_request, "websocket-request-plan-count", "3", errors, "cex_market_data_request_plans")
    require(cex_request, "parsed-transcript-count", "3", errors, "cex_market_data_request_plans")

    cex_balances = by_name["cex_balance_snapshots"]["parsed"]
    require(cex_balances, "balance-transcript-count", "3", errors, "cex_balance_snapshots")
    require(cex_balances, "parsed-balance-snapshot-count", "3", errors, "cex_balance_snapshots")
    require(cex_balances, "parsed-balance-asset-count", "6", errors, "cex_balance_snapshots")

    dex_request = by_name["dex_request_plans"]["parsed"]
    require(dex_request, "request-plan-count", "4", errors, "dex_request_plans")
    require(dex_request, "http-quote-plan-count", "1", errors, "dex_request_plans")
    require(dex_request, "solana-http-quote-plan-count", "1", errors, "dex_request_plans")
    require(dex_request, "rpc-quote-plan-count", "1", errors, "dex_request_plans")
    require(dex_request, "rpc-simulation-plan-count", "1", errors, "dex_request_plans")
    require(dex_request, "local-quote-request-count", "3", errors, "dex_request_plans")
    require(dex_request, "local-simulation-request-count", "1", errors, "dex_request_plans")

    dex_response = by_name["dex_response_transcripts"]["parsed"]
    require(dex_response, "response-transcript-count", "4", errors, "dex_response_transcripts")
    require(dex_response, "parsed-quote-response-count", "3", errors, "dex_response_transcripts")
    require(dex_response, "parsed-simulation-response-count", "1", errors, "dex_response_transcripts")
    require(dex_response, "simulation-status", "WouldSucceed", errors, "dex_response_transcripts")

    dex_tx_lifecycle = by_name["dex_transaction_lifecycle_transcripts"]["parsed"]
    require(dex_tx_lifecycle, "transaction-lifecycle-transcript-count", "4", errors, "dex_transaction_lifecycle_transcripts")
    require(dex_tx_lifecycle, "transaction-lifecycle-record-count", "4", errors, "dex_transaction_lifecycle_transcripts")
    require(dex_tx_lifecycle, "transaction-lifecycle-confirmed-count", "2", errors, "dex_transaction_lifecycle_transcripts")
    require(dex_tx_lifecycle, "transaction-lifecycle-reverted-count", "1", errors, "dex_transaction_lifecycle_transcripts")
    require(dex_tx_lifecycle, "transaction-lifecycle-failed-count", "1", errors, "dex_transaction_lifecycle_transcripts")
    require(dex_tx_lifecycle, "transaction-lifecycle-nonce-tracked-count", "2", errors, "dex_transaction_lifecycle_transcripts")

    dex_protocol = by_name["dex_protocol_risk_review"]["parsed"]
    require(dex_protocol, "protocol-risk-review-count", "2", errors, "dex_protocol_risk_review")
    require(dex_protocol, "protocol-risk-ready-count", "1", errors, "dex_protocol_risk_review")
    require(dex_protocol, "protocol-risk-blocked-count", "1", errors, "dex_protocol_risk_review")
    require(dex_protocol, "protocol-risk-blocker-count", "16", errors, "dex_protocol_risk_review")
    require(dex_protocol, "spender-hygiene-ready", "true", errors, "dex_protocol_risk_review")
    require(dex_protocol, "gas-slippage-ready", "true", errors, "dex_protocol_risk_review")
    require(dex_protocol, "mev-controls-ready", "true", errors, "dex_protocol_risk_review")
    require(dex_protocol, "terms-metadata-ready", "true", errors, "dex_protocol_risk_review")

    lifecycle = by_name["connector_lifecycle_audit"]["parsed"]
    require(lifecycle, "cex-lifecycle-final-status", "Filled", errors, "connector_lifecycle_audit")
    require(lifecycle, "cex-lifecycle-transcript-count", "3", errors, "connector_lifecycle_audit")
    require(lifecycle, "cex-cancel-lifecycle-final-status", "Cancelled", errors, "connector_lifecycle_audit")
    require(lifecycle, "cex-cancel-lifecycle-transcript-count", "3", errors, "connector_lifecycle_audit")
    require(lifecycle, "cex-cancel-lifecycle-remaining-quantity-base", "0.006", errors, "connector_lifecycle_audit")
    require(lifecycle, "dex-lifecycle-simulation-status", "LocallyValidated", errors, "connector_lifecycle_audit")
    for key in ("cex-audit-failed-closed", "dex-audit-failed-closed", "state-failure-failed-closed", "state-checkpoints-recovered"):
        require(lifecycle, key, "true", errors, "connector_lifecycle_audit")

    return errors


def main() -> int:
    args = parse_args()
    target = ROOT / "target"
    target.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="connector-scenario-gate-", dir=target) as workspace:
        components = [
            run_component(name, command)
            for name, command in command_set(pathlib.Path(workspace))
        ]

    errors = validate_components(components)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    audit_records = 0
    for component in components:
        value = component["parsed"].get("audit-records-replayed")
        if value is not None:
            try:
                audit_records += int(value)
            except ValueError:
                pass

    report = {
        "schema": "arbyclaw.connector_scenario_aggregate_gate.v1",
        "component_count": len(components),
        "all_components_passed": True,
        "unsafe_side_effect_flags_detected": False,
        "live_network_used": False,
        "credential_loaded": False,
        "websocket_connection_opened": False,
        "live_provider_call_performed": False,
        "external_submission_performed": False,
        "rpc_call_performed": False,
        "signing_or_broadcast_performed": False,
        "live_execution_performed": False,
        "production_ready": False,
        "audit_records_replayed": audit_records,
        "fee_schedule_reconciliation_review_enforced": True,
        "market_data_bad_data_rejection_review_enforced": True,
        "market_data_provider_latency_review_enforced": True,
        "market_data_provider_reconciliation_review_enforced": True,
        "components": [
            {
                "name": component["name"],
                "returncode": component["returncode"],
                "passed": component["passed"],
            }
            for component in components
        ],
        "remaining_external_evidence": [
            "live REST/WebSocket exchange adapters",
            "provider-backed market-data and fee validation",
            "external exchange sandbox/live order lifecycle calibration",
            "live DEX/RPC simulation and router validation without broadcasts",
            "external DEX/RPC nonce and confirmation validation without broadcasts",
            "external DEX protocol, spender, gas, slippage, and MEV validation without broadcasts",
            "production deployment-host connector validation",
        ],
    }
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print("connector scenario aggregate gate passed")
        print(f"component-count: {report['component_count']}")
        print("unsafe-side-effect-flags-detected: false")
        print("live-network-used: false")
        print("credential-loaded: false")
        print("websocket-connection-opened: false")
        print("live-provider-call-performed: false")
        print("external-submission-performed: false")
        print("rpc-call-performed: false")
        print("signing-or-broadcast-performed: false")
        print("live-execution-performed: false")
        print("production-ready: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
