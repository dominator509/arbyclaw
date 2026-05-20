# External Validation Checklist

This checklist records work that must occur outside ChatGPT Project Mode before any production, public-service, or live-funds claim.

## Repository Validation

- Run structure validator.
- Run Rust formatting validation.
- Run Rust workspace check.
- Run all Rust tests.
- Run clippy with warnings denied.
- Confirm generated docs and prompts contain no secret material.

## Security Validation

- Review deny-by-default policy engine behavior.
- Review mode gates and live-scope rejection.
- Review redaction paths for audit, communications, dashboard, observability, testing, hardening, and handoff records.
- Review command routing for injection and authorization risks.
- Verify dashboard and metrics boundaries do not expose public services by default.
- Verify packaging templates do not embed credentials.

## Release Engineering Validation

- Build release artifacts.
- Generate and review SBOM.
- Run dependency vulnerability review.
- Build and scan container image.
- Validate systemd hardening on a disposable host.
- Validate ARM build path or target runtime.
- Validate rollback procedure.
- Validate startup, shutdown, restart, and failure recovery.

## Runtime Validation

- Run deterministic paper-mode scenarios.
- Run replay/backtest fixtures from reviewed non-secret datasets.
- Run property tests and fuzzing in an isolated environment.
- Validate audit replay and state recovery.
- Validate observability without secret leakage.
- Validate alert routing only with approved non-production endpoints.

## External Integration Validation

- Validate exchange sandbox adapters without real funds.
- Validate DEX/RPC sandbox behavior without signing or broadcasting.
- Review custody and signer design before any key handling exists.
- Review exchange, protocol, jurisdiction, tax, and compliance terms.

## Production Approval Gate

Production approval requires all relevant evidence to be generated outside ChatGPT Project Mode, stored in non-secret evidence systems, reviewed by accountable humans, and linked from the gap tracker or hardening records by reference only.
