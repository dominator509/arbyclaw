# Phase 58 Subroadmap - Container Validator Fail-Closed Timeout Hardening

## Goal

Strengthen local container validation so Dockerized image-scan helpers fail closed without leaving long-running scan containers when Docker or Trivy behavior stalls. This supports the packaging/deployment and external-hardening blockers without publishing images, installing services, loading secrets, or claiming production readiness.

## Implemented in this phase

- Added timeout override flags to `scripts/validate_production_container.py` and `scripts/validate_container_example.py` for local fail-closed validation of Docker probe, image build, Trivy scan, and smoke steps.
- Added deterministic names for Dockerized Trivy scan containers.
- Added Docker `--pull never` to the Dockerized Trivy scan path so local validation does not implicitly pull scanner images.
- Added Trivy `--timeout <seconds>` inside scan containers so the scanner can fail closed before the outer Python subprocess timeout has to terminate Docker.
- Added best-effort forced cleanup for timed-out named scan containers.

## Deferred work

- Production image publishing.
- Service installation or service-manager lifecycle execution.
- Registry retention/provenance validation.
- Deployment-host runtime validation under service orchestration.
- External production-readiness review.

## Safety notes

This phase performs no image push, service install, service start/stop/restart, deployment mutation, secret loading, live trading, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, or production-readiness claims.
