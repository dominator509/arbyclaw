#!/usr/bin/env python3
"""Prevent assurance claims from outrunning retained evidence.

Capability promotion to EXTERNALLY_VALIDATED or PRODUCTION_APPROVED requires a
matching evidence-registry record. Mock/simulation/fixture/plan/preflight/local
transcript records can never satisfy that requirement. This validator protects
claim integrity; it does not itself prove that a referenced external artifact is
genuine, so human/external review remains required.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
CAPABILITIES_PATH = ROOT / "CAPABILITIES.md"
EVIDENCE_PATH = ROOT / "validation/external_evidence.json"
ARCH_MAP_PATH = ROOT / "docs/ai/ARCHITECTURE_MAP.md"
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
ALLOWED_STATES = {
    "MODELED",
    "LOCAL",
    "INTEGRATED_LOCAL",
    "EXTERNALLY_VALIDATED",
    "PRODUCTION_APPROVED",
}
REQUIRED_NONCAPABILITIES = (
    "exchange REST/WebSocket clients",
    "real Web3/RPC providers",
    "custody-backed signer",
    "transaction broadcast",
    "withdrawal/bridge execution",
    "persistent production dashboard server",
    "outbound messaging provider sessions",
    "production telemetry exporters/log shipping/alert delivery",
    "installed production service lifecycle",
)
SECRET_LIKE = re.compile(r"(?i)(api[_-]?key|private[_-]?key|seed[_-]?phrase|mnemonic|password|bearer\s+[A-Za-z0-9])")


def parse_capabilities(text: str) -> dict[str, str]:
    capabilities: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) < 4:
            continue
        name = cells[0].strip("`")
        state = cells[1].strip("`")
        if name in {"Capability", "---"} or state in {"Current state", "---"}:
            continue
        if state in ALLOWED_STATES:
            if name in capabilities:
                raise RuntimeError(f"duplicate capability row: {name}")
            capabilities[name] = state
    return capabilities


def load_registry() -> dict[str, Any]:
    try:
        registry = json.loads(EVIDENCE_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"unable to load external evidence registry: {exc}") from exc
    if registry.get("schema") != "arbyclaw.external_evidence_registry.v1":
        raise RuntimeError("external evidence registry has unexpected schema")
    return registry


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args()

    errors: list[str] = []
    try:
        capabilities = parse_capabilities(CAPABILITIES_PATH.read_text(encoding="utf-8"))
        registry = load_registry()
        architecture_map = ARCH_MAP_PATH.read_text(encoding="utf-8")
    except (OSError, RuntimeError) as exc:
        print(f"assurance integrity validation failed: {exc}", file=sys.stderr)
        return 2

    if len(capabilities) < 20:
        errors.append("capability matrix parsed fewer than 20 current capability rows")

    rules = registry.get("rules")
    records = registry.get("records")
    if not isinstance(rules, dict):
        errors.append("external evidence registry rules must be an object")
        rules = {}
    if not isinstance(records, list):
        errors.append("external evidence registry records must be a list")
        records = []

    accepted_types = set(rules.get("accepted_external_evidence_types", []))
    forbidden_types = set(rules.get("forbidden_as_external_proof", []))
    if not accepted_types or not forbidden_types:
        errors.append("external evidence registry must define accepted and forbidden evidence types")

    evidence_by_capability: dict[str, list[dict[str, Any]]] = {}
    seen_ids: set[str] = set()
    for record in records:
        if not isinstance(record, dict):
            errors.append("external evidence record is not an object")
            continue
        record_id = record.get("id")
        capability = record.get("capability")
        evidence_type = record.get("evidence_type")
        commit = record.get("commit")
        environment = record.get("environment")
        evidence_reference = record.get("evidence_reference")
        if not isinstance(record_id, str) or not record_id:
            errors.append("external evidence record has no id")
            continue
        if record_id in seen_ids:
            errors.append(f"duplicate external evidence record id: {record_id}")
        seen_ids.add(record_id)
        if not isinstance(capability, str) or capability not in capabilities:
            errors.append(f"{record_id} references unknown capability {capability}")
            continue
        if evidence_type in forbidden_types:
            errors.append(f"{record_id} uses forbidden simulated/local evidence type as external proof: {evidence_type}")
        if evidence_type not in accepted_types:
            errors.append(f"{record_id} uses unapproved external evidence type: {evidence_type}")
        if not isinstance(commit, str) or not COMMIT_RE.fullmatch(commit):
            errors.append(f"{record_id} must identify an exact 40-character commit SHA")
        if not isinstance(environment, str) or len(environment.strip()) < 3:
            errors.append(f"{record_id} must identify the external environment")
        if not isinstance(evidence_reference, str) or len(evidence_reference.strip()) < 3:
            errors.append(f"{record_id} must identify an evidence reference")
        elif SECRET_LIKE.search(evidence_reference):
            errors.append(f"{record_id} evidence reference contains secret-like material")
        if record.get("result") not in {"passed", "failed", "blocked"}:
            errors.append(f"{record_id} must record result as passed, failed, or blocked")
        evidence_by_capability.setdefault(capability, []).append(record)

    for capability, state in capabilities.items():
        records_for_capability = [
            record
            for record in evidence_by_capability.get(capability, [])
            if record.get("result") == "passed"
        ]
        if state in {"EXTERNALLY_VALIDATED", "PRODUCTION_APPROVED"} and not records_for_capability:
            errors.append(f"{capability} claims {state} without passed external evidence")
        if state == "PRODUCTION_APPROVED":
            approvals = [
                record
                for record in records_for_capability
                if record.get("evidence_type") == "human-production-approval"
                and isinstance(record.get("approved_by"), str)
                and record.get("approved_by", "").strip()
                and isinstance(record.get("approval_reference"), str)
                and record.get("approval_reference", "").strip()
            ]
            if not approvals:
                errors.append(f"{capability} claims PRODUCTION_APPROVED without accountable human approval evidence")

    for phrase in REQUIRED_NONCAPABILITIES:
        if phrase not in architecture_map:
            errors.append(f"AI architecture map lost explicit non-capability guard: {phrase}")

    payload = {
        "schema": "arbyclaw.assurance_integrity_validation.v1",
        "status": "failed" if errors else "passed",
        "capability_count": len(capabilities),
        "external_evidence_record_count": len(records),
        "error_count": len(errors),
        "errors": errors,
    }
    if args.as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif errors:
        print("assurance integrity validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
    else:
        print(
            "assurance integrity validation passed "
            f"({len(capabilities)} capabilities; {len(records)} external evidence records)"
        )
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
