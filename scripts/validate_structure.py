#!/usr/bin/env python3
"""Validate ArbyClaw's current repository structure and anti-drift invariants.

This validator intentionally checks the live Git tree instead of a generated hash
manifest. Git already preserves history and content identity; duplicating every
file hash in a committed Markdown manifest created a second, stale source of
truth and forced historical phase paperwork to remain architectural input.
"""

from __future__ import annotations

import pathlib
import re
import sys

from validate_repository_hygiene import tracked_files, violation_reason

ROOT = pathlib.Path(__file__).resolve().parents[1]

REQUIRED_FILES = [
    "README.md",
    "ARCHITECTURE.md",
    "CAPABILITIES.md",
    "ROADMAP.md",
    "PRODUCTION_GAP_TRACKER.md",
    "HANDOFF_CONTEXT.md",
    "AGENTS.md",
    "CLAUDE.md",
    "SECURITY.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rustfmt.toml",
    ".github/workflows/ci.yml",
    "docs/ai/ARCHITECTURE_MAP.md",
    "docs/ai/API_CONTRACTS.md",
    "docs/ai/REPO_BRIEF.md",
    "docs/history/PHASE_HISTORY.md",
    "crates/arb-core/Cargo.toml",
    "crates/arb-core/src/lib.rs",
    "crates/arb-core/src/config.rs",
    "crates/arb-core/src/secrets.rs",
    "crates/arb-core/src/state.rs",
    "crates/arb-core/src/strategy.rs",
    "crates/arb-core/src/destination.rs",
    "crates/arb-core/src/policy.rs",
    "crates/arb-core/src/audit.rs",
    "crates/arb-core/src/market_data.rs",
    "crates/arb-core/src/fees.rs",
    "crates/arb-core/src/hardening.rs",
    "crates/arb-core/src/handoff.rs",
    "crates/arb-core/src/opportunity.rs",
    "crates/arb-core/src/paper.rs",
    "crates/arb-core/src/planner.rs",
    "crates/arb-core/src/runtime.rs",
    "crates/arb-core/src/cex.rs",
    "crates/arb-core/src/dex.rs",
    "crates/arb-core/src/execution_adapter.rs",
    "crates/arb-core/src/communications.rs",
    "crates/arb-core/src/dashboard.rs",
    "crates/arb-core/src/observability.rs",
    "crates/arb-core/src/testing.rs",
    "crates/arb-core/src/packaging.rs",
    "crates/arb-core/tests/sqlite_wal_crash_restart.rs",
    "crates/arb-agent/Cargo.toml",
    "crates/arb-agent/src/main.rs",
    "config.example.toml",
    "deployment/README.md",
    "deployment/container/Containerfile.example",
    "deployment/container/Containerfile.production",
    "deployment/systemd/arb-agent.service.example",
    "deployment/arm/BUILD_PROFILES.md",
    "hardening/EXTERNAL_VALIDATION_RUNBOOK.md",
    "hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md",
    "hardening/PRODUCTION_READINESS_CHECKLIST.md",
    "hardening/INCIDENT_RESPONSE_DRILL_TEMPLATE.md",
    "handoff/AGENTIC_HANDOFF_PACKAGE.md",
    "handoff/FUTURE_AGENT_PROMPTS.md",
    "handoff/EXTERNAL_VALIDATION_CHECKLIST.md",
    "scripts/validate_repository_hygiene.py",
    "scripts/validate_test_collection.py",
    "scripts/validate_container_example.py",
    "scripts/validate_production_container.py",
    "scripts/validate_release_artifact.py",
    "scripts/validate_arm_build_profiles.py",
    "scripts/validate_arm_cross_check.py",
    "scripts/validate_packaging_deployment_gate.py",
    "scripts/validate_hardening_core_gate.py",
    "scripts/validate_agentic_handoff_candidate_gate.py",
    "scripts/validate_execution_path_gate.py",
    "scripts/validate_operator_surface_gate.py",
    "scripts/validate_dependency_license_policy.py",
    "scripts/validate_deployment_host_runtime.py",
    "scripts/validate_deployment_runtime_gate.py",
    "scripts/validate_deployment_static_hardening.py",
    "scripts/validate_opportunity_scenario_gate.py",
    "scripts/validate_connector_scenario_gate.py",
    "scripts/validate_deployment_evidence_checklist.py",
    "scripts/validate_deployment_evidence_bundle.py",
    "scripts/validate_incident_response_drill.py",
    "scripts/validate_rollback_drill.py",
    "scripts/validate_systemd_example.py",
    "scripts/validate_systemd_lifecycle.py",
]

FORBIDDEN_SECRET_ASSIGNMENT = re.compile(
    r"(?i)(api[_-]?key|secret|private[_-]?key|seed[_-]?phrase|mnemonic|token)\s*[:=]\s*['\"]?[A-Za-z0-9_/+=.-]{12,}"
)
NUMERIC_READINESS = re.compile(r"(?i)\b\d{1,3}%\s+(?:current\s+)?production\s+readiness\b")
SKIP_DIRS = {".git", "target"}
SCAN_SUFFIXES = {".md", ".toml", ".rs", ".yml", ".yaml", ".env", ".example", ".production"}
CANONICAL_STATUS_DOCS = (
    "README.md",
    "ARCHITECTURE.md",
    "CAPABILITIES.md",
    "ROADMAP.md",
    "PRODUCTION_GAP_TRACKER.md",
    "HANDOFF_CONTEXT.md",
)
AI_CONTEXT_DOCS = (
    "docs/ai/ARCHITECTURE_MAP.md",
    "docs/ai/API_CONTRACTS.md",
)


def fail(message: str) -> int:
    print(f"validation failed: {message}", file=sys.stderr)
    return 1


def main() -> int:
    for relative in REQUIRED_FILES:
        if not (ROOT / relative).is_file():
            return fail(f"missing required file: {relative}")

    legacy_phase_files = sorted(ROOT.glob("PHASE_*_SUBROADMAP.md"))
    if legacy_phase_files:
        return fail(
            "legacy numbered phase files reintroduced: "
            + ", ".join(path.name for path in legacy_phase_files[:5])
        )

    if (ROOT / "STRUCTURE_MANIFEST.md").exists():
        return fail("legacy STRUCTURE_MANIFEST.md reintroduced")

    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    for member in ("crates/arb-core", "crates/arb-agent"):
        if member not in cargo_toml:
            return fail(f"workspace member not registered: {member}")

    try:
        files = tracked_files()
    except RuntimeError as exc:
        return fail(f"unable to enumerate tracked files: {exc}")
    hygiene_violations = [
        (path, reason)
        for path in files
        if (reason := violation_reason(path)) is not None
    ]
    if hygiene_violations:
        path, reason = hygiene_violations[0]
        return fail(f"repository hygiene violation: {path}: {reason}")

    for relative in AI_CONTEXT_DOCS:
        text = (ROOT / relative).read_text(encoding="utf-8")
        normalized = text.strip()
        if len(normalized) < 500:
            return fail(f"AI context document is suspiciously empty: {relative}")
        if re.search(r"(?i)\bTODO\b", normalized):
            return fail(f"AI context document contains TODO placeholder: {relative}")

    for relative in CANONICAL_STATUS_DOCS:
        text = (ROOT / relative).read_text(encoding="utf-8")
        if NUMERIC_READINESS.search(text):
            return fail(f"numeric production-readiness score reintroduced in {relative}")

    for path in ROOT.rglob("*"):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        if not path.is_file():
            continue
        if path.suffix not in SCAN_SUFFIXES and path.name != ".env.example":
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if FORBIDDEN_SECRET_ASSIGNMENT.search(text):
            return fail(f"potential secret assignment found in {path.relative_to(ROOT)}")

    print(
        "repository structure validation passed "
        f"({len(REQUIRED_FILES)} required files; {len(files)} tracked files checked)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
