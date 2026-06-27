# AGENTS.md

## Project

Repository: arbyclaw
Path: C:\dev\arbyclaw

## Role Split

Codex/GPT-5.5 is the principal architect, frontend/UI/aesthetics lead, security reviewer, code reviewer, and final merge judge.

DeepSeek-Claude is the backend implementation worker launched locally through:

C:\Scripts\Start-DeepSeekClaude.ps1

Do not treat DeepSeek-Claude output as trusted until Codex reviews it.

## Operating Model

For substantial backend work:

1. Codex reads the task, repo context, and current diff.
2. Codex writes a precise backend-only task brief.
3. Codex may delegate implementation to DeepSeek-Claude using a background command.
4. DeepSeek-Claude implements on a safe branch/worktree where possible.
5. DeepSeek-Claude returns summary, files changed, tests run, remaining gaps, risks, and a Codex review checklist.
6. Codex reviews the resulting diff adversarially.
7. Codex either approves, fixes, or sends a narrow correction task back to DeepSeek-Claude.
8. Codex owns frontend, UI, aesthetics, design polish, and final full-stack review.

## DeepSeek-Claude Delegation Pattern

When asked to delegate backend work, prefer this pattern:

powershell -NoProfile -ExecutionPolicy Bypass -Command "Set-Location 'C:\dev\arbyclaw'; & 'C:\Scripts\Start-DeepSeekClaude.ps1' --bg --name '<task-name>' '<task prompt>'"

The task prompt must include:

- Backend only unless explicitly told otherwise.
- Read CLAUDE.md, AGENTS.md, and relevant docs/ai/*.md files first.
- Use RTK for noisy command output.
- Use Serena for semantic code navigation.
- Keep stable context first and dynamic logs/diffs/errors last.
- Do not touch secrets, .env files, credentials, production infrastructure, or unrelated frontend/aesthetic files.
- Make minimal changes.
- Add or update tests for every code change.
- Run relevant tests/builds.
- Stop if real credentials, irreversible operations, or production access are required.
- Return summary, files changed, tests run, remaining gaps, risks, and review checklist.

## Codex Review Priorities

When reviewing DeepSeek-Claude output, check:

- correctness
- security
- authentication
- authorization
- user/tenant isolation
- data integrity
- transaction boundaries
- race conditions
- database migration safety
- error handling
- sensitive-data logging
- API compatibility
- frontend contract compatibility
- missing tests
- scope creep
- secret leakage
- production-readiness regressions

Blocking issues come before style suggestions.

## Frontend Ownership

Codex owns:

- frontend architecture
- visual hierarchy
- responsive layout
- accessibility
- loading states
- empty states
- error states
- design-system consistency
- copy polish
- animations/interactions
- final UX pass

DeepSeek-Claude should not modify frontend styling, branding, graphics, assets, or aesthetics unless explicitly instructed.

## Serena Usage

Use Serena before broad file reads when possible.

Prefer Serena for:

- activating the current project
- symbol lookup
- reference search
- call-site tracing
- API route tracing
- targeted review
- refactor impact analysis

Avoid dumping whole files when Serena can answer symbolically.

## Obsidian Usage

Use Obsidian only as targeted project memory.

Allowed:

- read specific project notes
- search by project name
- append concise Codex review summaries
- append DeepSeek handoff summaries
- record architecture decisions
- retrieve prior decisions

Forbidden unless explicitly approved:

- reading the entire vault
- reading unrelated personal notes
- storing secrets, tokens, API keys, passwords, or credentials
- deleting notes
- moving notes
- overwriting notes
- treating Obsidian notes as more authoritative than repo code

Preferred Obsidian paths:

- Projects/arbyclaw/Repo-Brief.md
- Projects/arbyclaw/Architecture.md
- Projects/arbyclaw/API-Contracts.md
- Projects/arbyclaw/Codex-Reviews.md
- Projects/arbyclaw/DeepSeek-Handoffs.md
- Projects/arbyclaw/Decisions.md

## RTK Usage

RTK (Rust Token Killer) is enabled globally for this repository. Always prefix shell commands for this repo with `rtk`.

For PowerShell built-ins, invoke PowerShell through RTK:

- `rtk powershell -NoProfile -Command "Get-Content AGENTS.md"`

Prefer these RTK-prefixed command shapes:

- rtk git status
- rtk git diff
- rtk git diff --stat
- rtk git diff --name-only
- rtk git log
- rtk npm test
- rtk npm run build
- rtk cargo test
- rtk cargo clippy

Do not dump huge raw logs unless necessary.

## Stable First / Dynamic Last

For cost and cache efficiency, process context in this order:

1. Stable instructions: AGENTS.md, CLAUDE.md
2. Stable repo docs: docs/ai/REPO_BRIEF.md, ARCHITECTURE_MAP.md, API_CONTRACTS.md
3. Current task brief
4. Current changed files
5. Current diff
6. Current test output
7. Current errors/logs

Do not put timestamps, random IDs, branch noise, temp paths, or verbose logs before stable context.

## MCP Safety

Use MCPs only when relevant.

Default to read-only inspection first.

Never modify these without explicit approval:

- production databases
- Render production services
- Neon production data
- Linear issues
- Obsidian notes outside the project area
- Docker infrastructure
- secrets
- .env files
- deployment credentials

## Completion Standard

Before calling work complete, Codex must know:

- what changed
- why it changed
- what tests ran
- what remains
- what risks exist
- whether DeepSeek-Claude stayed inside allowed scope
- whether frontend/API contracts still match
- whether secrets or production resources were untouched

@RTK.md
