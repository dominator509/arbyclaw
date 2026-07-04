# Phase 56 Subroadmap - Local Deployment Failure-Capture Transcript Validation

## Goal

Add a typed local validator for sanitized deployment-host panic-hook, tracing-subscriber, and failure-capture evidence metadata. The validator checks that operator-owned references cover deployment host identity, daemon-wide panic-hook evidence, daemon-wide tracing/logging evidence, failure scenario, captured failure artifact locator, sanitized payload review, runtime quiesce/degrade behavior, post-failure runtime smoke, audit replay, SQLite recovery, alert-route reference, operator approval, and reviewer approval.

## Implemented in this phase

- Added deployment failure-capture transcript request/report types, status enum, and validation version in the packaging core.
- Added a local-only deployment failure-capture transcript validator with fail-closed blocker codes and explicit denial of panic-hook installation, tracing-subscriber installation, failure injection, service actions, file mutation, alert delivery, external calls, live execution, and production-readiness claims.
- Exported the new deployment failure-capture transcript validator through `arb-core`.
- Added `arb-agent validate-deployment-failure-capture-transcript` with ready and blocked local fixtures.
- Added unit tests for complete deployment failure-capture evidence, missing deployment failure-capture evidence, and validator side-effect denial.
- Added the validator to `scripts/validate_deployment_runtime_gate.py` so the aggregate local deployment-runtime gate checks it with the other transcript validators.

## Deferred work

- Actual daemon-wide panic-hook or tracing-subscriber installation on a deployment host.
- Actual failure injection or panic capture under service-manager orchestration.
- Real alerting or escalation delivery.
- Production-host runtime smoke and recovery execution after real daemon failure.
- Independent external review of deployment failure-capture evidence.

## Safety notes

This phase performs no panic-hook installation, tracing-subscriber installation, failure injection, service actions, alert delivery, file mutation, deployment mutation, live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, or secret handling.
