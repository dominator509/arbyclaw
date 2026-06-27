# Older umbrella gap reconcile (2026-06-13)

- Updated `PRODUCTION_GAP_TRACKER.md` `GAP-0019` title to reflect that deployment packaging local and CI validation already exists; the remaining missing slice is external deployment validation.
- Updated `GAP-0019` blocker wording so it no longer implies local packaging/tooling work is blocked in Codex.
- Reworked `GAP-0020` from `Penetration Testing Missing` to `External Penetration Testing and Wallet-Custody Review Missing`.
- `GAP-0020` now reflects current repo reality: local-SARIF SAST, dependency audit, secret-pattern scan, command-injection denial, authentication/authorization denial paths, and signer/custody-side fail-closed local reviews exist, while external pen testing, DAST, deployed-surface exercise, and wallet-custody review remain missing.
- Updated `GAP-0020` future validation wording to preserve the current local hardening gates while keeping external testing/review work open.
- Fresh RTK validation after the tracker updates: structure validation passed, `cargo fmt --check` passed, `cargo check --workspace` passed, `cargo test --workspace` passed with 475 tests, and `cargo clippy --workspace --all-targets -- -D warnings` passed.