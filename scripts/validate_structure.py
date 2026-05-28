#!/usr/bin/env python3
"""Repository structure and safety validation for local/CI use."""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

REQUIRED_FILES = [
    "ARCHITECTURE.md",
    "ROADMAP.md",
    "AGENTS.md",
    "PHASE_0_SUBROADMAP.md",
    "PHASE_1_SUBROADMAP.md",
    "PHASE_2_SUBROADMAP.md",
    "PHASE_3_SUBROADMAP.md",
    "PHASE_4_SUBROADMAP.md",
    "PHASE_5_SUBROADMAP.md",
    "PHASE_6_SUBROADMAP.md",
    "PHASE_7_SUBROADMAP.md",
    "PHASE_8_SUBROADMAP.md",
    "PHASE_9_SUBROADMAP.md",
    "PHASE_10_SUBROADMAP.md",
    "PHASE_11_SUBROADMAP.md",
    "PHASE_12_SUBROADMAP.md",
    "PHASE_13_SUBROADMAP.md",
    "PHASE_14_SUBROADMAP.md",
    "PHASE_15_SUBROADMAP.md",
    "PHASE_16_SUBROADMAP.md",
    "PHASE_17_SUBROADMAP.md",
    "PHASE_18_SUBROADMAP.md",
    "PHASE_19_SUBROADMAP.md",
    "PHASE_20_SUBROADMAP.md",
    "PHASE_21_SUBROADMAP.md",
    "PHASE_22_SUBROADMAP.md",
    "PHASE_23_SUBROADMAP.md",
    "PHASE_24_SUBROADMAP.md",
    "PRODUCTION_GAP_TRACKER.md",
    "HANDOFF_CONTEXT.md",
    "STRUCTURE_MANIFEST.md",
    "README.md",
    "SECURITY.md",
    "Cargo.toml",
    "rust-toolchain.toml",
    "rustfmt.toml",
    ".github/workflows/ci.yml",
    "crates/arb-core/Cargo.toml",
    "crates/arb-core/src/lib.rs",
    "crates/arb-core/src/config.rs",
    "crates/arb-core/src/secrets.rs",
    "crates/arb-core/src/state.rs",
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
    "deployment/systemd/arb-agent.service.example",
    "deployment/arm/BUILD_PROFILES.md",
    "hardening/EXTERNAL_VALIDATION_RUNBOOK.md",
    "hardening/RELEASE_REVIEW_EVIDENCE_TEMPLATE.md",
    "hardening/PRODUCTION_READINESS_CHECKLIST.md",
    "hardening/INCIDENT_RESPONSE_DRILL_TEMPLATE.md",
    "handoff/AGENTIC_HANDOFF_PACKAGE.md",
    "handoff/FUTURE_AGENT_PROMPTS.md",
    "handoff/EXTERNAL_VALIDATION_CHECKLIST.md",
]

FORBIDDEN_SECRET_ASSIGNMENT = re.compile(
    r"(?i)(api[_-]?key|secret|private[_-]?key|seed[_-]?phrase|mnemonic|token)\s*[:=]\s*['\"]?[A-Za-z0-9_/+=.-]{12,}"
)

SKIP_DIRS = {".git", "target"}
SCAN_SUFFIXES = {".md", ".toml", ".rs", ".yml", ".yaml", ".env", ".example"}


def fail(message: str) -> int:
    print(f"validation failed: {message}", file=sys.stderr)
    return 1


def main() -> int:
    for relative in REQUIRED_FILES:
        if not (ROOT / relative).exists():
            return fail(f"missing required file: {relative}")

    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    for member in ["crates/arb-core", "crates/arb-agent"]:
        if member not in cargo_toml:
            return fail(f"workspace member not registered: {member}")

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

    print("repository structure validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
