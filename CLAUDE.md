# CLAUDE.md

Claude-compatible coding agents must follow `AGENTS.md`; this file adds no competing architecture or completion rules.

## Required context order

1. `AGENTS.md`
2. `CAPABILITIES.md`
3. `ARCHITECTURE.md`
4. `docs/ai/ARCHITECTURE_MAP.md`
5. `docs/ai/API_CONTRACTS.md`
6. `PRODUCTION_GAP_TRACKER.md`
7. `ROADMAP.md`
8. relevant source/tests

Do not load generated whole-repository snapshots as canonical context. Serena/Obsidian/RTK configuration may help navigation on a maintainer workstation, but tool-local memory is subordinate to repository source and tests.

## Work style

- Inspect symbols/references before broad full-file edits where tooling supports it.
- Prefer focused patches over append-only growth.
- Verify every dependency/API before using it.
- Preserve fail-closed policy, audit/state, secret, destination, and signer boundaries.
- Do not create numbered phase files or numeric production-readiness scores.
- Do not represent local fixtures, transcripts, mocks, dry runs, or preflights as real external execution.
- Run the validation sequence in `AGENTS.md` to the extent the environment permits and mark unavailable checks `UNVERIFIED`.

## Current structural direction

The next major refactor is mechanical decomposition of the oversized `arb-agent/src/main.rs` and large `arb-core` domain modules. Preserve command names, output contracts, and safety behavior while reducing ownership ambiguity.

No live trading, signing, withdrawals, bridges, broadcasts, public service exposure, real provider submission, or production approval may be introduced without explicit human authorization and applicable external validation.
