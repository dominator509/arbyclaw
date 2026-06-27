# Suggested Commands

All repo shell commands should be RTK-prefixed.

- Inspect status: `rtk git status --short --branch`
- Changed files: `rtk git diff --name-only`
- Structure gate: `rtk python scripts/validate_structure.py`
- Format: `rtk cargo fmt --check`
- Check: `rtk cargo check --workspace`
- Tests: `rtk cargo test --workspace`
- Clippy: `rtk cargo clippy --workspace --all-targets -- -D warnings`
- Runtime/deployment aggregate: `rtk python scripts/validate_deployment_runtime_gate.py --json`
- Container example gate when Docker is available: `rtk python scripts/validate_container_example.py`
- Systemd example static gate: `rtk python scripts/validate_systemd_example.py`

If `cargo` is missing from PATH, use the local cargo binary through RTK instead of changing repo files. Do not install packages unless explicitly asked.