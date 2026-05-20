# Production Readiness Checklist

This checklist is intentionally fail-closed. An unchecked item is a blocker.

## Build and Supply Chain

- [ ] Cargo formatting, check, test, and clippy pass externally.
- [ ] Release build succeeds with locked dependencies.
- [ ] Dependency audit is reviewed.
- [ ] SBOM is generated and reviewed.
- [ ] Release artifact provenance is documented.

## Runtime and Deployment

- [ ] Container image or package is built and scanned externally.
- [ ] Service runs as non-root with no-new-privileges where applicable.
- [ ] Filesystem write paths are minimal and documented.
- [ ] No public bind is enabled by default.
- [ ] Staging deployment validates config loading, startup, shutdown, restart, health, and logging.

## Security and Operations

- [ ] Secret redaction is validated in logs, audit, dashboard, notifications, and crash paths.
- [ ] Penetration test and AppSec review are complete.
- [ ] Rollback drill is complete.
- [ ] Incident-response drill is complete.
- [ ] Audit replay and state recovery are validated.
- [ ] Terms, jurisdiction, and exchange policy review are complete.

## Live-Funds Blockers

- [ ] Custody/signer boundary is implemented and externally reviewed.
- [ ] Exchange-specific live connectors are implemented and sandbox-validated.
- [ ] DEX/RPC adapters are implemented and simulation-validated.
- [ ] Transaction signing and broadcast controls are externally validated.
- [ ] Withdrawal and bridge policies are externally reviewed and disabled unless explicitly approved.
- [ ] Live-funds approval is granted outside ChatGPT Project Mode by the accountable operator.
