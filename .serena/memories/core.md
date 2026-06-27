# ArbyClaw Core

## Authority
- Repo files, current diff, and command output outrank memory.
- Stable operating docs: `AGENTS.md`, `CLAUDE.md`, `docs/ai/REPO_BRIEF.md`.
- Roadmap/gap authority for production work: `PRODUCTION_GAP_TRACKER.md`, `ROADMAP.md`, `ARCHITECTURE.md`, `STRUCTURE_MANIFEST.md`, relevant `PHASE_*_SUBROADMAP.md`.

## Memory Graph
- Safety/forbidden work: `mem:safety`.
- Codebase navigation map: `mem:navigation`.
- Validation expectations: `mem:validation` and `mem:task_completion`.
- Stack/tooling snapshot: `mem:tech_stack`.
- Repo command cheatsheet: `mem:suggested_commands`.
- Agent conventions: `mem:conventions`.
- Core Rust module map: `mem:backend/core`.

## Project Shape
- Rust workspace rooted at `C:\dev\arbyclaw`; main crates are `crates/arb-core` and `crates/arb-agent`.
- Local-first arbitrage agent scaffold: deterministic fixtures, typed fail-closed boundaries, append-only audit journal, SQLite WAL checkpoints, paper execution/ledgering, runtime recovery, CLI validation reports.
- User preference: close real roadmap gaps with code; avoid evidence-only loops, invented blockers, and readiness overclaims.