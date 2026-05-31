# Deployment Notes

Phase 16 adds deployment documentation templates only. These files do not prove that a build, container run, service install, cloud deployment, rollback drill, or production release was performed.

## Safety Defaults

- Live trading must remain disabled.
- Runtime mode must remain observe or paper unless future phases complete custody, signer, connector, audit, policy, and external validation gates.
- Do not place secrets in this repository, Markdown, container build context, systemd units, shell history, logs, or generated artifacts.
- Do not expose dashboard, metrics, command, or control surfaces publicly.
- Treat all container, systemd, ARM, CI, and release steps as externally validated only after they are executed in a capable environment.

## Required External Validation Before Deployment Claims

```bash
python3 scripts/validate_structure.py
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
python3 scripts/validate_systemd_example.py
```

`scripts/validate_systemd_example.py` checks the committed example unit only. On Linux hosts with `systemd-analyze`, `python3 scripts/validate_systemd_example.py --systemd-analyze` also runs syntax verification against a temporary fake root without installing, enabling, reloading, or starting a service. Passing it is not production service-manager validation.

Additional target-specific validation is required for containers, systemd, ARM builds, rollback drills, incident drills, and security review.

## Phase 16 Boundary

The Rust `arb-core::packaging` module records deterministic package/deployment plans and rejects live trading, public exposure, embedded secret material, build claims, deployment claims, and production claims.
