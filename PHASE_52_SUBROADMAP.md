# Phase 52 Subroadmap - Local Deployment Retention Transcript Validation

## Scope

Add a typed local validator for sanitized deployment-host audit retention and rotation evidence metadata. The validator checks that physical/deployment-like host evidence references include active rotation, archive retention, expired archive deletion, post-rotation append validation, audit replay after rotation, retention policy reference, recovery/runbook reference, and operator approval.

## Implemented

- Added deployment retention transcript request/report types and status enum in the audit core.
- Added `validate_deployment_retention_transcript` for non-mutating transcript validation.
- Added `arb-agent validate-deployment-retention-transcript` with ready and blocked fixture paths.
- Added CI coverage through the Rust validation workflow.
- Added unit tests for complete deployment-host evidence, missing host/rotation evidence, and validator rotation denial.

## Deferred

- Actual retention/rotation execution on a deployment host.
- Deployment-host filesystem mutation under operator control.
- Runtime recovery evidence captured from a real service-managed deployment after rotation.
- Independent external review of retention/rotation evidence.

## Safety

This phase performs no log rotation, archive deletion, filesystem mounting, production-path mutation, service actions, live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, or secret handling.
