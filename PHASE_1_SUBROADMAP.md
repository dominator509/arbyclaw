# PHASE_1_SUBROADMAP.md

## Phase

Phase 1 — Rust Workspace Scaffold

## Status

- Status: Scaffold implemented in ChatGPT Project Mode.
- Production readiness contribution: Partial +2% realized; external Rust validation remains deferred.
- Current phase result: Minimal Rust workspace files, CI skeleton, safety docs, and structural validator exist.

## Objectives

1. Create a minimal Rust workspace with deterministic crate boundaries.
2. Establish a single-binary entrypoint without trading, signing, or connector behavior.
3. Add initial CI workflow for formatting, compilation, tests, clippy, and structure validation.
4. Add repository-level safety and secret-handling documentation.
5. Keep the patch reversible and avoid implementing later-phase business logic.

## Deliverables

- Root `Cargo.toml` workspace.
- `crates/arb-core` library crate.
- `crates/arb-agent` binary crate.
- `rust-toolchain.toml`.
- `rustfmt.toml`.
- `.github/workflows/ci.yml`.
- `.gitignore`.
- `.env.example` with no secrets.
- `README.md`.
- `SECURITY.md`.
- `scripts/validate_structure.py`.

## Subsystem Boundaries

### Included

- Repository scaffold.
- Build metadata.
- Initial CLI binary entrypoint.
- Core identity and placeholder runtime-mode primitive.
- Static safety documentation.
- Structural validation script.

### Excluded

- Real configuration schema.
- Secret manager.
- Policy engine.
- Audit journal.
- Market data connectors.
- CEX connectors.
- DEX/Web3 connectors.
- Wallet signer.
- Opportunity engine.
- Execution planner.
- Live execution.
- Dashboard.
- Messaging integrations.

## Dependencies

- Phase 0 governance files complete.
- Rust stable toolchain required for external compile/test validation.
- GitHub or equivalent CI provider required for hosted CI validation.

## Implementation Sequence

1. Reconcile governance files.
2. Create `PHASE_1_SUBROADMAP.md`.
3. Add minimal root workspace configuration.
4. Add `arb-core` crate with safe primitives only.
5. Add `arb-agent` binary that reports scaffold-only status.
6. Add formatting/toolchain metadata.
7. Add CI workflow skeleton.
8. Add static structure validator.
9. Update roadmap and production gap tracker.

## Validation Sequence

### Completed In ChatGPT Project Mode

- Verified mandatory governance files exist.
- Verified Phase 1 scaffold files exist.
- Verified workspace member paths are registered.
- Ran `python3 scripts/validate_structure.py`.
- Confirmed no obvious secret assignment pattern was detected by the structure validator.

### Deferred Due Environment Limitations

The following commands were not executable because this environment does not include Rust/Cargo:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Rollback Strategy

To roll back Phase 1:

1. Remove `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `.gitignore`, `.env.example`, `README.md`, and `SECURITY.md` if they were introduced only by this phase.
2. Remove `.github/workflows/ci.yml`.
3. Remove `scripts/validate_structure.py`.
4. Remove `crates/arb-core` and `crates/arb-agent`.
5. Revert `ROADMAP.md` and `PRODUCTION_GAP_TRACKER.md` to Phase 0 status.

No runtime state, secrets, infrastructure, or live integrations were created.

## Drift-Prevention Constraints

- Do not add exchange-specific behavior in Phase 1.
- Do not add live trading behavior in Phase 1.
- Do not add wallet/signing behavior in Phase 1.
- Do not add secret parsing beyond `.env.example` documentation in Phase 1.
- Do not claim Rust validation until the commands actually run in a Rust-enabled environment.
- Keep `unsafe_code` forbidden.

## Environment Limitations

- Rust/Cargo unavailable in the ChatGPT execution environment.
- Hosted CI unavailable until the project is pushed to a repository with CI enabled.
- No target hardware, VPS, or ARM device validation was performed.

## Expected Unresolved Gaps

- Cargo validation deferred.
- Hosted CI execution deferred.
- Dependency audit deferred.
- Supply-chain validation deferred.
- Runtime execution validation deferred.
- Packaging and ARM validation deferred.

## Expected Future Continuation Tasks

1. Run Rust validation commands in a Rust-enabled local or CI environment.
2. Fix any compile, clippy, or formatting issues discovered externally.
3. Create `PHASE_2_SUBROADMAP.md` before implementing config, secrets, and mode gates.
4. Preserve the scaffold-only live-trading-disabled posture until policy and custody phases are complete.

## Phase Completion Update

### ROADMAP.md

Updated to reflect Phase 1 scaffold implementation and deferred Rust toolchain validation.

### PRODUCTION_GAP_TRACKER.md

Updated with Phase 1 status and new environment-limited validation gaps.

### Production Readiness

Updated from 2% to 4%.

### Risk Posture

Still high. The repository has a scaffold but no policy, secrets, audit, connectors, simulation, or execution safety implementation.
