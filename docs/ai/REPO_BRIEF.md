# ArbyClaw Repo Brief

Compact context for coding agents. Repository source, tests, current diffs, and executed command output outrank this brief.

## Purpose

ArbyClaw is a Rust-first, local-first arbitrage research/simulation/validation system. It implements deterministic local/paper boundaries, policy, audit/state persistence, SQLite WAL recovery, opportunity/planning/execution-adapter modeling, local connector/provider reviews, packaging/hardening validation, and explicit production blockers. It does not currently implement production live trading.

## Stack

- Rust workspace, edition 2021: `arb-core` library + `arb-agent` CLI.
- SQLite WAL for local persistent state/checkpoints.
- Append-only local audit journal primitives.
- Python validation orchestration under `scripts/`.
- GitHub Actions under `.github/workflows/`.
- Example/production-intent deployment assets under `deployment/`.

## Read first

1. `AGENTS.md`
2. `CAPABILITIES.md`
3. `ARCHITECTURE.md`
4. `docs/ai/ARCHITECTURE_MAP.md`
5. `docs/ai/API_CONTRACTS.md`
6. `PRODUCTION_GAP_TRACKER.md`
7. `ROADMAP.md`

## Entrypoints

- `crates/arb-agent/src/main.rs` — CLI and local validation runners; currently oversized and targeted for mechanical decomposition.
- `crates/arb-core/src/lib.rs` — module declarations and broad re-exports; trace symbols to defining modules before edits.
- `crates/arb-core/src/runtime.rs` — local lifecycle/recovery boundaries.
- `crates/arb-core/src/audit.rs` / `state.rs` — audit and SQLite WAL persistence.
- `scripts/validate_structure.py` — current required-tree/anti-drift validation; no hash manifest.
- `scripts/validate_repository_hygiene.py` — tracked artifact hygiene.
- `scripts/validate_test_collection.py` — per-package zero-test guard.
- `scripts/validate_agentic_handoff_candidate_gate.py` — top local aggregate over handoff audit + hardening-core.

## Validation sequence

```bash
python3 scripts/validate_repository_hygiene.py
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace --locked
python3 scripts/validate_test_collection.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/validate_release_artifact.py
python3 scripts/validate_agentic_handoff_candidate_gate.py --json
```

Unavailable checks are `UNVERIFIED`, never implicitly passed.

## Safety boundaries

Do not enable or claim live exchange/RPC submission, live trading, wallet custody/signing, broadcasts, withdrawals, bridges, public service exposure, outbound provider delivery, production deployment, production readiness, or live-funds approval without explicit authorization and applicable external evidence.

Never commit raw secrets or generated/mock evidence as canonical source.

## Governance

- `CAPABILITIES.md` owns capability-state claims.
- `PRODUCTION_GAP_TRACKER.md` owns unresolved closure conditions.
- `ROADMAP.md` owns future work IDs.
- Git history owns chronology.
- Historical numbered phase files and `STRUCTURE_MANIFEST.md` are retired and must not be recreated.

`docs/ai/ARCHITECTURE_MAP.md` and `docs/ai/API_CONTRACTS.md` are now populated canonical AI context, not placeholders.
