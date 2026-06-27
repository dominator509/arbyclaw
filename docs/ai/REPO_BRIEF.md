# ArbyClaw Repo Brief

Compact context for Codex, Serena, DeepSeek-Claude, and Obsidian links. Repo code, current diffs, and command output are more authoritative than this brief.

## Purpose

ArbyClaw is a Rust-first, local-first crypto arbitrage agent scaffold. It currently implements deterministic paper/local boundaries, audit/state wiring, SQLite WAL validation, paper ledgering/fill models, deployment/hardening validation scripts, and roadmap/gap tracking. It is not production-ready and does not implement live trading.

## Stack

- Rust workspace, edition 2021, with `arb-core` library and `arb-agent` CLI.
- SQLite WAL state/audit validation inside Rust modules and tests.
- Python validation scripts under `scripts/`.
- GitHub Actions hardening/validation workflows under `.github/workflows/`.
- Example-only deployment assets under `deployment/` for container, systemd, and ARM profile checks.

## Entrypoints

- `crates/arb-agent/src/main.rs`: CLI validation commands and local report runners.
- `crates/arb-core/src/lib.rs`: public module exports.
- `crates/arb-core/src/runtime.rs`: local runtime lifecycle boundaries.
- `crates/arb-core/src/audit.rs` and `crates/arb-core/src/state.rs`: append-only audit journal and SQLite WAL state store.
- `crates/arb-core/src/paper.rs`, `planner.rs`, `execution_adapter.rs`, `opportunity.rs`: paper execution, planning, adapter, and opportunity models.
- `scripts/validate_structure.py`: structure manifest gate.
- `scripts/validate_deployment_runtime_gate.py`: local runtime/deployment probe composition.

## Main Commands

Use RTK for repo shell commands.

- Structure: `rtk python scripts/validate_structure.py`
- Format: `rtk cargo fmt --check`
- Check: `rtk cargo check --workspace`
- Test: `rtk cargo test --workspace`
- Lint: `rtk cargo clippy --workspace --all-targets -- -D warnings`
- Runtime gate: `rtk python scripts/validate_deployment_runtime_gate.py --json`
- If Cargo is not on `PATH`, use the local cargo binary through RTK instead of changing repo files.

## Important Directories

- `crates/arb-core/`: core domain, policy, audit/state, market data, connectors, planner, paper, runtime, packaging, hardening, and handoff logic.
- `crates/arb-agent/`: CLI wrapper for local validation/report commands.
- `scripts/`: Python validation gates and evidence/checklist validators.
- `deployment/`: example-only container, systemd, and ARM build-profile assets.
- `hardening/`: release review, external validation, incident drill, and production checklist templates.
- `handoff/`: agentic handoff package and future-agent prompts.
- `docs/ai/`: compact AI context files. Keep these stable and token-frugal.
- `PHASE_*_SUBROADMAP.md`, `ROADMAP.md`, `ARCHITECTURE.md`, `PRODUCTION_GAP_TRACKER.md`: roadmap authority and blocker tracking.

## Safety Boundaries

- Do not implement or enable live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, secrets, or outbound production side effects.
- Do not store raw API keys, wallet keys, seed phrases, mnemonics, tokens, or secret-like snippets in code, Markdown, TOML, logs, prompts, Serena memories, or Obsidian notes.
- Do not mutate production infrastructure, production databases, Docker infrastructure, deployment credentials, or `.env` files without explicit approval.
- Preserve fail-closed blockers and honest "not production-ready" language until real external/operator validation exists.
- Treat `.obsidian/`, `.serena/cache/`, `target/`, Python caches, coverage outputs, temp files, and backups as local/generated state.

## Current Unknowns / TODOs

- External live exchange/RPC adapters, production signing/custody, public dashboard hosting/auth, real communications adapters, exporter/alert delivery, production deployment, load tests, pen tests, rollback drills, and operator production approval remain outside local proof.
- `docs/ai/ARCHITECTURE_MAP.md` and `docs/ai/API_CONTRACTS.md` are still placeholders and should be filled from the actual code before being treated as authority.
