# Task Completion

- For setup/doc-only changes: validate affected YAML/JSON/Markdown readability and run targeted `git diff --name-only` plus `git status --short` for setup files.
- For Rust/code changes, default validation sequence remains: `rtk python scripts/validate_structure.py`; `rtk cargo fmt --check`; `rtk cargo check --workspace`; `rtk cargo test --workspace`; `rtk cargo clippy --workspace --all-targets -- -D warnings`.
- Patch compile/test/lint failures with smallest safe changes; do not expand runtime behavior beyond the task.
- If a command cannot run due to local environment, report exact blocker and do not count it as repo failure.
- Update `PRODUCTION_GAP_TRACKER.md` only when remaining failures/gaps changed or the task is tracker reconciliation.
- Completion report should name files changed, checks run/results, remaining gaps/risks, and avoid production-readiness claims.