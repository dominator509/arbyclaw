# PHASE_11_SUBROADMAP.md

## Phase 11 — Execution Adapter Boundaries

## Governance Status

- Created before Phase 11 implementation.
- Depends on completed Phases 0–10.
- Scope is model/trait boundaries only in ChatGPT Project Mode.
- Live order placement, live DEX swaps, wallet signing, withdrawals, bridges, transaction broadcasts, real RPC calls, real exchange calls, and secrets remain prohibited.

## Phase Goal

Introduce a fail-closed execution-adapter framework that consumes Phase 10 `ExecutionPlanDraft` records, revalidates policy outcomes, models adapter lifecycle status, records deterministic fill/reconciliation records, and preserves hard boundaries before any future live connector implementation.

## In Scope

- `arb-core::execution_adapter` module.
- Stable execution-adapter framework version constant.
- Execution-adapter configuration and request models.
- Adapter lifecycle, attempt, fill, and reconciliation records.
- Execution-adapter trait boundary.
- Deterministic no-network adapter boundary implementation.
- Policy revalidation before any modeled adapter action.
- Durable adapter-attempt policy-revalidation evidence and local kill-switch denial coverage.
- Duplicate planner/adapter lifecycle identifier rejection before local audit or checkpoint persistence.
- Reconciliation replay before modeled-fill paper ledger settlement.
- Duplicate modeled-fill settlement rejection after local ledger checkpoint reopen.
- Local partial/no-fill recovery planning with sanitized audit and SQLite WAL checkpoint persistence, without submitting cancels, hedges, orders, swaps, transactions, broadcasts, withdrawals, or bridges.
- Fail-closed rejection of live scope and adapter submission.
- Structure validator update requiring Phase 11 files.
- Governance documentation updates.

## Out of Scope

- Live CEX orders.
- Live DEX swaps.
- RPC calls.
- Wallet signing.
- Transaction broadcasts.
- Withdrawals.
- Bridges.
- Real exchange APIs.
- Secrets, private keys, seed phrases, API keys, provider tokens, or credentials.
- Durable audit/state runtime wiring.
- Balance mutation against real accounts.
- Exchange-specific order schemas.
- Nonce, gas, approval, or mempool handling.

## Required Inputs

- `ARCHITECTURE.md`
- `ROADMAP.md`
- `AGENTS.md`
- `PRODUCTION_GAP_TRACKER.md`
- `PHASE_10_SUBROADMAP.md`
- `HANDOFF_CONTEXT.md`
- `STRUCTURE_MANIFEST.md`
- `scripts/validate_structure.py`
- `crates/arb-core/src/planner.rs`
- `crates/arb-core/src/policy.rs`

## Implementation Tasks

1. Re-run repository structure validation before code.
2. Add `PHASE_11_SUBROADMAP.md` before code.
3. Add an execution-adapter framework module in `arb-core`.
4. Export Phase 11 adapter types from `arb-core`.
5. Surface Phase 11 status in `arb-agent` without enabling execution.
6. Update the structure validator for Phase 11 governance and module files.
7. Update architecture, roadmap, security, README, handoff context, manifest, and gap tracker.
8. Re-run available validation.
9. Prepare a commit-ready repository state.

## Security Requirements

- Adapters must not submit external orders or transactions.
- Adapter records must explicitly show external submission is disabled.
- Live scope must be rejected fail-closed.
- Policy must be re-evaluated at adapter boundary.
- Adapter attempts must record policy revalidation before local audit or checkpoint persistence.
- Planner policy outcomes must not be blindly trusted.
- Duplicate draft intent/policy-outcome ids and duplicate adapter attempt/fill/reconciliation ids must be rejected fail-closed.
- Modeled fills must match reconciled adapter records before local paper ledger mutation.
- Replaying the same adapter-run modeled fills into a restored local paper ledger must fail closed before new ledger or audit mutation.
- Partial/no-fill recovery plans must remain local metadata only, require operator review where exposure remains, and record no external submission or live execution.
- DEX/Web3 intents must not be signed or broadcast.
- No secrets may be added to code, docs, config, logs, test fixtures, or audit-like records.
- Reconciliation records are deterministic model records only.

## Validation Plan

Available in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Required externally because Cargo is not available in this environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo run -p arb-agent -- validate-execution-adapter-audit --workspace target/ci-execution-adapter-audit
cargo clippy --workspace --all-targets -- -D warnings
```

## Completion Criteria

- Phase 11 sub-roadmap exists before code.
- Structure validator passes.
- `arb-core::execution_adapter` exists and is exported.
- Execution-adapter framework consumes `ExecutionPlanDraft` records only.
- Execution-adapter framework produces deterministic attempt, fill, and reconciliation records.
- Execution-adapter framework revalidates policy.
- Adapter attempts preserve durable policy-revalidation evidence and local kill-switch denial remains covered.
- Duplicate local lifecycle identifiers are rejected before checkpoint or audit persistence.
- Adapter reconciliation records are replayed and checked before modeled paper fill settlement.
- Duplicate modeled-fill settlement after local ledger checkpoint reopen is rejected before mutation.
- Partial/no-fill adapter outcomes produce local recovery-plan records with audit/checkpoint persistence and no external side effects.
- `arb-agent validate-execution-adapter-audit --workspace <fresh-dir>` replays adapter-run and recovery-plan audit records, recovers SQLite WAL checkpoints, and proves invalid audit/state-write paths fail closed.
- Live scope and external adapter submission remain unavailable.
- Governance docs and gap tracker reflect Phase 11 state.

## Rollback Plan

1. Remove `crates/arb-core/src/execution_adapter.rs`.
2. Remove execution-adapter exports from `crates/arb-core/src/lib.rs`.
3. Revert `arb-agent` status text to Phase 10.
4. Remove Phase 11 requirements from `scripts/validate_structure.py`.
5. Revert governance docs to Phase 10 state.
6. Re-run `python3 scripts/validate_structure.py`.

## Phase 11 Result

Completed in ChatGPT Project Mode as a framework-only execution-adapter boundary. Production/external validation remains required before production claims.
