# PHASE_2_SUBROADMAP.md

## Phase

Phase 2 — Config, Secrets, and Mode Gates

## Status

- Status: Implemented in ChatGPT Project Mode; Rust toolchain validation deferred.
- Production readiness contribution: Partial +5% realized; +1% deferred pending Cargo validation.
- Current phase result: Typed configuration, secret-reference abstractions, redacted secret material, mode-gate validation, config example, and CLI config loading path exist. No live trading, wallet signing, or external secret injection was implemented.

## Objectives

1. Add typed `config.toml` parsing for non-secret runtime configuration.
2. Add deterministic mode gates for `observe`, `paper`, and `live-armed` modes.
3. Add secret-reference types that support environment variable names and encrypted-keystore aliases without storing raw secrets in repository files.
4. Add a redacted secret-material wrapper and secret-provider interface.
5. Add validation rules that reject live mode unless explicit live acknowledgement, non-disabled secret backend, risk limits, and withdrawal restrictions are satisfied.
6. Keep the implementation isolated inside `arb-core` and the `arb-agent` entrypoint.

## Deliverables

- `crates/arb-core/src/config.rs`
- `crates/arb-core/src/secrets.rs`
- Updated `crates/arb-core/src/lib.rs`
- Updated `crates/arb-core/Cargo.toml`
- Updated `crates/arb-agent/src/main.rs`
- Updated `crates/arb-agent/Cargo.toml`
- `config.example.toml`
- Updated `.env.example`
- Updated `scripts/validate_structure.py`
- Updated `ARCHITECTURE.md`
- Updated `ROADMAP.md`
- Updated `PRODUCTION_GAP_TRACKER.md`

## Subsystem Boundaries

### Included

- Typed config schema.
- Non-secret config parsing.
- Environment-variable secret references.
- Encrypted-keystore alias references.
- Redacted in-memory secret wrapper.
- Secret-provider trait and environment-provider skeleton.
- Deny-by-default mode validation for live execution.
- CLI path to load and validate a config file.

### Excluded

- Actual encrypted keystore backend implementation.
- Exchange API client credentials usage.
- Wallet private-key storage or signer implementation.
- Policy engine beyond basic mode-gate validation.
- Audit journal persistence.
- Market-data connectors.
- CEX/DEX connectors.
- Opportunity engine.
- Live execution adapters.

## Dependencies

- Phase 0 governance files complete.
- Phase 1 Rust workspace scaffold complete.
- Rust toolchain required for external compile/test validation.
- Future encrypted keystore requires local filesystem or OS keyring validation.

## Implementation Sequence

1. Reconcile governance files.
2. Create `PHASE_2_SUBROADMAP.md` before code changes.
3. Add `serde` and `toml` dependencies to `arb-core` only.
4. Add `secrets` module with `SecretRef`, `SecretMaterial`, and `SecretProvider` boundary.
5. Add `config` module with typed config structs and validation rules.
6. Update `arb-agent` to optionally load `--config <path>`.
7. Add safe example config and `.env.example` reference names only.
8. Extend structural validation to require Phase 2 files.
9. Update architecture, roadmap, and gap tracker.
10. Run available validation and record unavailable Rust validation honestly.

## Validation Sequence

### Completed In ChatGPT Project Mode

- Verified mandatory governance files exist.
- Verified Phase 2 files exist after patching.
- Ran `python3 scripts/validate_structure.py`.
- Confirmed no obvious secret assignment pattern was detected by the structure validator.
- Confirmed no raw credential values were added to repository files.

### Deferred Due Environment Limitations

The following commands were not executable because this environment does not include Rust/Cargo:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Additional deferred validation:

- Real `.env` loading with operator-owned secrets.
- Encrypted-keystore backend validation.
- OS keyring/KMS validation.
- CI execution in hosted repository.
- Runtime validation on VPS/local/ARM devices.

## Rollback Strategy

To roll back Phase 2:

1. Remove `crates/arb-core/src/config.rs` and `crates/arb-core/src/secrets.rs`.
2. Revert `crates/arb-core/src/lib.rs`, `crates/arb-core/Cargo.toml`, `crates/arb-agent/src/main.rs`, and `crates/arb-agent/Cargo.toml` to Phase 1 versions.
3. Remove `config.example.toml` and Phase 2 additions to `.env.example`.
4. Revert `scripts/validate_structure.py` Phase 2 requirements.
5. Revert `ARCHITECTURE.md`, `ROADMAP.md`, and `PRODUCTION_GAP_TRACKER.md` to Phase 1 status.

No runtime state, live secrets, exchange connections, wallet keys, infrastructure, or deployed services were created.

## Drift-Prevention Constraints

- Do not store raw secrets in config, Markdown, examples, or logs.
- Do not implement exchange-specific behavior in Phase 2.
- Do not implement wallet signing in Phase 2.
- Do not implement live execution in Phase 2.
- Do not claim encrypted-keystore implementation; only the boundary exists.
- Do not claim Rust validation until commands run in a Rust-enabled environment.
- Keep `unsafe_code` forbidden.

## Environment Limitations

- Rust/Cargo unavailable in the ChatGPT execution environment.
- No local `.env` with real credentials was used or requested.
- No encrypted-keystore backend exists yet.
- No target hardware, VPS, or ARM device validation was performed.
- Hosted CI unavailable until pushed to a repository with CI enabled.

## Expected Unresolved Gaps

- Rust compile/test/clippy validation deferred.
- Encrypted-keystore backend missing.
- Secret lifecycle/zeroization hardening missing.
- Policy engine missing.
- Audit journal missing.
- Runtime live-mode validation missing.
- Operator secret provisioning missing.

## Expected Future Continuation Tasks

1. Run Rust validation commands in a Rust-enabled environment.
2. Fix any compile, clippy, or formatting issues discovered externally.
3. Implement Phase 3 deny-by-default policy engine and trust-contract checks.
4. Replace the keystore boundary with a validated encrypted backend.
5. Add redaction tests and no-secret-log tests under Cargo validation.

## Phase Completion Update

### ROADMAP.md

Updated to reflect Phase 2 implementation and deferred Rust validation.

### PRODUCTION_GAP_TRACKER.md

Updated with Phase 2 status, reduced config/mode-gate gap scope, and remaining secret-backend validation gaps.

### Production Readiness

Updated from 4% to 9%.

### Risk Posture

Still high. The repository has typed config and mode gates, but no policy engine, audit journal, custody backend, connectors, simulation, or live execution safety implementation.
