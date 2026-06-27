# Backend Core

- `crates/arb-core/src/lib.rs` exports domain modules; keep module boundaries visible there when adding new core surfaces.
- Important local-only modules: `config`, `secrets`, `strategy`, `destination`, `policy`, `audit`, `state`, `market_data`, `fees`, `cex`, `dex`, `opportunity`, `planner`, `execution_adapter`, `paper`, `communications`, `dashboard`, `observability`, `testing`, `packaging`, `hardening`, `handoff`, `runtime`.
- `crates/arb-agent/src/main.rs` is the CLI entrypoint for validation/report commands; new local gates usually need CLI wiring plus tests/scripts if roadmap-facing.
- SQLite WAL and audit JSONL behavior must remain local/non-secret/fail-closed. Production lifecycle/deployment claims require explicit external validation, not just local harnesses.
- Existing integration tests include crash/restart durability under `crates/arb-core/tests/`; prefer adding narrow tests near touched behavior.