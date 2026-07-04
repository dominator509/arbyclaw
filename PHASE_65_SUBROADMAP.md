# Phase 65 Subroadmap - Validation Corpus Breadth Gate

## Goal

Strengthen the deterministic local validation corpus runner so it explicitly enforces minimum local corpus breadth for plans, test cases, fixtures, fuzz corpora, and backtest scenarios before reporting ready for local review.

## Implemented in this phase

- Added minimum corpus breadth fields to `LocalValidationCorpusRequest`.
- Added matching durable report fields to `LocalValidationCorpusReport`.
- Required validation corpus reports to fail closed when requested local plan, test, fixture, fuzz, or backtest breadth is not met.
- Persisted the breadth requirements and result in validation-corpus audit metadata and SQLite checkpoints.
- Broadened the built-in local validation corpus CLI fixture from two plans to three plans with additional local-only security and backtest regression cases.
- Surfaced the breadth requirements in `arb-agent validate-local-validation-corpus`.
- Updated `scripts/validate_opportunity_scenario_gate.py` so the aggregate opportunity scenario gate enforces the reported corpus breadth requirements.
- Added fail-closed unit coverage for insufficient local validation corpus breadth.

## Deferred work

- External fuzz engine execution.
- External property-test execution beyond current Cargo/property coverage.
- Curated external/deployment replay corpora.
- Production load tests.
- Penetration tests.
- Production backtest evidence.
- Production runtime validation.

## Safety notes

This phase expands deterministic local validation only. It performs no external fuzzer invocation, live network tests, external data downloads, live trading, adapter submission, signing, withdrawals, bridges, broadcasts, real RPC calls, real exchange calls, wallet custody, secret loading, or production-readiness claim.
