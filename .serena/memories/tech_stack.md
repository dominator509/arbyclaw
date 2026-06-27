# Tech Stack

- Rust workspace, edition 2021, `unsafe_code = forbid`; crates: `arb-core` library and `arb-agent` CLI.
- Python scripts under `scripts/` provide structure, deployment, hardening, container/systemd, opportunity/connector, and runtime validation gates.
- Persistence surfaces are local only: SQLite WAL state store plus append-only JSONL audit journal.
- Deployment assets are example/static validation surfaces under `deployment/container`, `deployment/systemd`, and `deployment/arm`; do not treat them as production deployment proof.
- GitHub Actions exist for local/CI validation and non-secret hardening evidence. Code scanning may use local SARIF artifact evidence for private repo constraints.
- Obsidian optimization is reference-based: link/summarize `docs/ai/REPO_BRIEF.md`; do not copy huge repo content or artifacts.
- Serena config uses Rust LSP backend and ignores generated/local state such as `target/**`, `.serena/cache/**`, `.obsidian/**`, Python caches, coverage, temp files, and backups.