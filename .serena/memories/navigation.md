# ArbyClaw Navigation

## Semantic Workflow

- Use Serena symbol overview/reference search before broad source reads.
- Use `rg` for fast text/file discovery when symbol names are unknown.
- Keep stable docs first, current diff/errors last.

## Primary Code Areas

- Runtime lifecycle, smoke, recovery: `crates/arb-core/src/runtime.rs`
- CLI commands and operator output: `crates/arb-agent/src/main.rs`
- Audit journal and durability: `crates/arb-core/src/audit.rs`
- State/WAL exports: inspect `crates/arb-core/src/lib.rs` and state symbols.
- Paper execution, ledger, fills: `crates/arb-core/src/paper.rs`
- Planner/opportunity traces: `crates/arb-core/src/planner.rs`, `crates/arb-core/src/opportunity.rs`
- Observability: `crates/arb-core/src/observability.rs`
- Dashboard and communications: `crates/arb-core/src/dashboard.rs`, `crates/arb-core/src/communications.rs`
- Policy/config/secrets/signing guardrails: `crates/arb-core/src/policy.rs`, `crates/arb-core/src/config.rs`, `crates/arb-core/src/secrets.rs`, `crates/arb-core/src/signer.rs`

## Obsidian Project Notes

- Vault path prefix: `Projects/arbyclaw/`
- Preferred notes: `Repo-Brief.md`, `Architecture.md`, `API-Contracts.md`, `Codex-Reviews.md`, `DeepSeek-Handoffs.md`, `Decisions.md`
- Use as targeted memory only; verify against repo before acting.
