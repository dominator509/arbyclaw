# ArbyClaw Core

## Authority

- Repo files and current command output outrank memory.
- Read `AGENTS.md` and `CLAUDE.md` for operating rules.
- Read `PRODUCTION_GAP_TRACKER.md`, `ROADMAP.md`, `ARCHITECTURE.md`, and relevant `PHASE_*_SUBROADMAP.md` before gap work.

## Memory Graph

- Safety boundaries and forbidden work: `mem:safety`
- Codebase navigation map: `mem:navigation`
- Required local validation sequence: `mem:validation`

## Project Shape

- Rust workspace rooted at `C:\dev\arbyclaw`.
- Main crates: `crates/arb-core`, `crates/arb-agent`.
- Work is local-first: typed boundaries, deterministic fixtures, append-only audit records, SQLite WAL checkpoints, runtime recovery, paper simulation, CLI validation reports.
- User wants direct gap closure through code, not evidence-only loops or invented blockers.
