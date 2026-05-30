# CI/CD Pipeline Security & Build Provenance

## Pipeline Assessment
- **Secrets Management:** GitHub Actions / GitLab CI must inject secrets via OIDC or encrypted environment variables. Secrets should never be echoed to stdout.
- **Dependency Pinning:** `Cargo.lock` must be strictly enforced. CI should run `cargo fetch --locked` to ensure reproducibility.
- **Artifact Signing:** Final Rust binaries must be signed using `cosign` or a similar Sigstore tool to guarantee build provenance (SLSA Level 3+).
- **Container Images:** Dockerfiles must use distroless or minimal base images (e.g., `gcr.io/distroless/cc-debian12`), signed, and pushed to a secure registry.

## Action Items for DevOps
1. Enable Branch Protection (require signed commits, 2+ reviewers).
2. Integrate Phase 1 (Secrets Scanner) and Phase 2 (cargo-audit / clippy) into pre-commit and PR workflows.
3. Establish a reproducible build pipeline.
