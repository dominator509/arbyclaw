#!/usr/bin/env python3
"""Generate STRUCTURE_MANIFEST.md for the repository."""

from __future__ import annotations

import hashlib
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
SKIP_DIRS = {".git", ".serena", "target", "__pycache__"}
SKIP_FILE_PREFIXES = ("__tmp",)
SKIP_FILE_MARKERS = (".bak-",)
MANIFEST = "STRUCTURE_MANIFEST.md"


def included_files() -> list[pathlib.Path]:
    files: list[pathlib.Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        if relative.as_posix() == MANIFEST:
            continue
        if any(part in SKIP_DIRS for part in relative.parts):
            continue
        if relative.name.startswith(SKIP_FILE_PREFIXES):
            continue
        if any(marker in relative.name for marker in SKIP_FILE_MARKERS):
            continue
        files.append(path)
    return sorted(files, key=lambda path: path.relative_to(ROOT).as_posix())


def main() -> int:
    rows = []
    for path in included_files():
        relative = path.relative_to(ROOT).as_posix()
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        rows.append(f"| `{relative}` | {len(data)} | `{digest}` |")

    lines = [
        "# STRUCTURE_MANIFEST.md",
        "",
        "Generated during Phase 53 local deployment permission transcript validation after Phase 52 local deployment retention transcript validation, Phase 51 local deployment disk-full transcript validation, Phase 50 local service-manager lifecycle transcript validation, Phase 49 local production runtime preflight, Phase 48 local Web3 sandbox/live discrepancy calibration, Phase 47 local Web3 broadcast adapter control review, Phase 46 local Web3 raw transaction serialization review, Phase 45 local Web3 provider nonce reconciliation, Phase 44 local Web3 unsigned transaction construction, Phase 43 local Web3 broadcast-readiness review, Phase 42 local Web3 unsigned payload review, Phase 41 local Web3 nonce reservation, Phase 40 local Web3 pre-sign safety review, Phase 39 local signer authorization envelope review, Phase 38 local signer runtime isolation review, Phase 37 local DEX/Web3 protocol risk review, Phase 36 local DEX/Web3 transaction lifecycle transcript parsing, Phase 35 local CEX balance snapshot transcript parsing, Phase 34 local CEX order lifecycle transcript parsing, Phase 33 local DEX/Web3 response-transcript parsing, Phase 32 local DEX/Web3 request-plan validation, Phase 31 local CEX market-data request-plan validation, Phase 30 aggregate connector scenario gate validation, Phase 29 aggregate opportunity scenario gate validation, Phase 28 aggregate deployment-runtime gate validation, and Phase 27 opportunity depth, paper inventory, transfer-risk, same-venue triangular path, local replay/false-positive, built-in local regression-corpus, local historical fixture replay, candidate audit/state trace, candidate trace restart/reopen recovery, replay-candidate planner handoff, CLI replay-validation modeling, local market-data reconnect/backoff plan validation, local CEX exchange-fixture matching validation, local CEX request-plan validation, local CEX balance snapshot transcript parsing, local CEX order lifecycle transcript parsing, local DEX/Web3 request-plan validation, local DEX/Web3 response-transcript parsing, local DEX/Web3 transaction lifecycle transcript parsing, local DEX/Web3 protocol risk review, local Web3 nonce reservation, local Web3 unsigned payload review, local Web3 pre-sign safety review, local Web3 broadcast-readiness review, local Web3 unsigned transaction construction, local Web3 provider nonce reconciliation, local Web3 raw transaction serialization review, local Web3 broadcast adapter control review, local Web3 sandbox/live discrepancy calibration, local production runtime preflight, local service-manager lifecycle transcript validation, local deployment disk-full transcript validation, local deployment retention transcript validation, local deployment permission transcript validation, local signer runtime isolation review, local signer authorization envelope review, local CEX mocked transcript parsing validation, local CEX rate-limit validation, local CEX credential/API-scope review, local authenticated-keystore encryption, local secret-rotation planning, local signer-secret-scope review, local destination ownership-evidence reference review checkpointing, local CEX/DEX framework validation audit/state checkpointing, local execution-adapter recovery-plan audit/state checkpointing, local scoped observability tracing subscriber capture, local sandbox observability log retention execution, local observability alert-route dispatch bridging, and local validation property-check audit/state checkpointing after Phase 26 audit/runtime crash, concurrency, filesystem, disk-full, audit-durability-cli, sandbox-retention-execution, blocked-state-preflight, blocked-audit-preflight, deployment-filesystem-preflight, communications-runtime-cli five-record recovery, local remote-command envelope validation, local authenticated channel-adapter validation, dashboard-runtime-cli, observability-runtime-cli, observability-runtime-reporting, rendered one-shot-dashboard-hosted-request, one-shot-observability-metrics-endpoint, stale-lock, lifecycle-concurrency, runtime-smoke adapter recovery-plan recovery, runtime-smoke communications review/envelope/channel-adapter recovery, runtime-smoke dashboard hosted-security/request recovery, runtime-smoke observability operations/export/endpoint/metrics/tracing recovery, runtime-smoke validation-runner recovery, runtime-smoke paper ledger recovery, runtime-smoke concurrent-lifecycle validation, state-permission, graceful-shutdown, backup-restore, restart-recovery, recovery-disposition, CLI-status, container-validation, systemd-lifecycle-planning, deployment-host-runtime-reporting, aggregate deployment-runtime gate validation, aggregate opportunity scenario gate validation, aggregate connector scenario gate validation, rollback-drill-planning, incident-response-drill-planning, deployment-evidence-bundle-indexing, deployment-evidence-checklist validation, and incomplete-recovery fail-closed validation.",
        "",
        "Note: this manifest intentionally excludes `STRUCTURE_MANIFEST.md` itself to avoid self-referential hash drift.",
        "",
        "## File Manifest",
        "",
        "| Path | Bytes | SHA-256 |",
        "|---|---:|---|",
        *rows,
    ]
    (ROOT / MANIFEST).write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
