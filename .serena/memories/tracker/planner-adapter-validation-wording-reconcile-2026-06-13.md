# Planner/Adapter tracker wording reconcile (2026-06-13)

- Reconciled `PRODUCTION_GAP_TRACKER.md` entries `GAP-0013`, `GAP-0056`, and `GAP-0058` so their remaining-validation text matches current local code evidence.
- The tracker now explicitly treats the following as already present locally rather than future-only:
  - planner audit/state replay and checkpoint recovery
  - duplicate intent/policy-outcome id rejection
  - route-specific failure-mode coverage
  - deterministic adapter handoff coverage
  - partial/no-fill recovery-plan coverage
  - recovery-plan restart/smoke checkpoint recovery
  - adapter policy revalidation / kill-switch denial / duplicate lifecycle rejection
- Remaining blockers stay open and external-facing: deployment-host/service-manager restart behavior, real connector behavior, sandbox/live reconciliation, external cancel/hedge execution, and live submission.
- Validation after reconcile stayed green:
  - `rtk python3 scripts/validate_structure.py`
  - `rtk cargo fmt --check`
  - `rtk cargo check --workspace`
  - `rtk cargo test --workspace` (`478 passed`)
  - `rtk cargo clippy --workspace --all-targets -- -D warnings`