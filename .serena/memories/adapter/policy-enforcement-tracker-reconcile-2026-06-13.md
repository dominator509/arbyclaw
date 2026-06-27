# Execution adapter policy-enforcement reconcile (2026-06-13)

- Fixed an operator-facing bug in `crates/arb-agent/src/main.rs`: `validate-execution-adapter-audit` was printing the source planner status under `adapter-run-status` instead of the actual adapter run status.
- Added a local `execution_adapter_run_status_label(...)` helper plus unit coverage for all adapter run statuses.
- `validate-execution-adapter-audit` now prints both `adapter-run-status` and `source-plan-status`, and also surfaces `adapter-policy-revalidated` and `adapter-policy-denied-attempts`.
- Strengthened the validator to fail if any adapter attempt lacks `policy_revalidated = true`.
- Reconciled `GAP-0034` in `PRODUCTION_GAP_TRACKER.md` from "Local Policy Decision Records Exist" to "Local Policy-to-Adapter Enforcement Exists" and updated the description/why incomplete/validation impact text to match the actual repo state: planner policy outcomes, adapter-time policy revalidation, and runtime audit/state-before-adapter sequencing exist locally; live connectors and deployment-host enforcement still do not.
- Full RTK validation after the change: structure validator passed, `cargo fmt --check` passed, `cargo check --workspace` passed, `cargo test --workspace` passed with 475 tests, and `cargo clippy --workspace --all-targets -- -D warnings` passed.