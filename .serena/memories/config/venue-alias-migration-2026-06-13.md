# Config migration venue-alias expansion (2026-06-13)

- `crates/arb-core/src/config.rs::migrate_config_toml_to_current(...)` now migrates legacy allowlist field names inside a current `[venues]` section:
  - `allowed_exchanges` -> `cex_allowlist`
  - `allowed_dexes` -> `dex_allowlist`
  - `allowed_chains` -> `chain_allowlist`
  - `allowed_assets` -> `asset_allowlist`
- New action code: `CONFIG_MIGRATED_VENUE_FIELD_ALIASES`.
- Safety behavior is fail-closed: migration now returns `ConfigError::ParseFailed` when a `[venues]` table contains both a legacy field and its current equivalent.
- Added focused `arb-core` tests for successful migration and ambiguity rejection.
- `crates/arb-agent/src/main.rs::run_config_migration_validation()` now covers a second legacy fixture for the `[venues]` field-alias path and reports `legacy-venue-alias-status` / `legacy-venue-alias-action-codes`.
- Docs reconciled in `ARCHITECTURE.md` and `PRODUCTION_GAP_TRACKER.md`. `HANDOFF_CONTEXT.md` still has an encoding problem that prevented safe `apply_patch` editing in this pass.
- Full RTK validation after the change passed on 2026-06-13: structure manifest generation, py_compile for validation scripts, structure validation, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (470 passed), and `cargo clippy --workspace --all-targets -- -D warnings`.