# Conventions

- Stable-first context order: `AGENTS.md`, `CLAUDE.md`, `docs/ai/REPO_BRIEF.md`, architecture/roadmap/gap docs, then current diff/errors.
- Use Serena symbol/reference tools before broad reads for code tasks.
- Preserve no-readiness language; separate "local/CI evidence exists" from "production validation missing".
- Keep changes scoped, tested, and roadmap-linked; do not create extra gates or documentation loops unless requested.
- Never touch secrets, `.env` files, production infrastructure, live RPC/exchange calls, signing, wallet custody, withdrawals, bridges, or broadcasts without explicit approval.
- Use `apply_patch` for manual edits; do not revert unrelated dirty worktree changes.
- Obsidian notes are references/summaries only under `Projects/arbyclaw/`; repo docs remain more authoritative than vault notes.