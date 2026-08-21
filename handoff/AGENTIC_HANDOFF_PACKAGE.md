# Agentic Handoff Package

## Purpose

This package gives future coding agents and human maintainers a deterministic continuation path for ArbyClaw without treating historical phase paperwork or generated snapshots as architecture.

It is not a production-readiness approval, live-funds approval, deployment record, security certification, or external validation result.

## Authoritative files

Read in this order:

1. `AGENTS.md`
2. `CAPABILITIES.md`
3. `ARCHITECTURE.md`
4. `docs/ai/ARCHITECTURE_MAP.md`
5. `docs/ai/API_CONTRACTS.md`
6. `PRODUCTION_GAP_TRACKER.md`
7. `ROADMAP.md`
8. relevant source code and tests

Git history preserves chronology. Tool memories are navigation aids only.

## Required opening procedure

1. Start from the exact current checkout/ref.
2. Run `python3 scripts/validate_repository_hygiene.py`.
3. Run `python3 scripts/validate_structure.py`.
4. Run the available Rust/test checks from `AGENTS.md`.
5. Mark unavailable checks `UNVERIFIED`; do not guess.
6. Reconcile the task against `CAPABILITIES.md` and open gap IDs.
7. Make the smallest behavior-preserving patch that satisfies the requirement.
8. Add regression evidence before claiming completion.

## Non-negotiable safety boundaries

Do not add or enable live trading, live order placement, live DEX swaps, wallet custody/signing, transaction broadcast, withdrawals, bridges, real exchange/RPC submission, public service exposure, outbound provider delivery, or production deployment without explicit human authorization and applicable external evidence.

Do not place credentials or secret material in Markdown, TOML, source, logs, audit records, prompts, generated evidence, screenshots, containers, or chat.

## Validation reality

A local validator may prove local behavior only. External/production claims require real external execution. Simulated/mock audit material is not acceptable evidence.

The top local handoff gate is:

```bash
python3 scripts/validate_agentic_handoff_candidate_gate.py --json
```

It executes the handoff-specific audit plus the single hardening-core aggregate. Hardening-core owns the lower execution/operator/opportunity/connector/deployment suites, preventing the handoff gate from rerunning them a second time.

## Handoff rule

Future work may continue only if the change preserves fail-closed policy, redaction, audit/state durability, destination/signer controls, and visible production blockers. The exact commit/environment being claimed validated must have matching execution evidence.
