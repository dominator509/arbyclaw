# Historical Phase Index

ArbyClaw previously used `PHASE_0_SUBROADMAP.md` through `PHASE_129_SUBROADMAP.md` as an append-only implementation chronology. Those files duplicated Git history, README/roadmap status, gap tracking, and handoff context, and the structure validator made all 130 files mandatory.

The numbered phase files were retired during the 2026-08-20 drift remediation. Their complete contents remain permanently recoverable from Git history at and before baseline commit `e4971244f2a643a32a55ed58d941f03a7bc96ae3`.

## Historical ranges

- **0–18:** initial governance, Rust workspace, config/secrets, policy, audit/state, market data/fees, paper simulation, CEX/DEX frameworks, opportunity/planner/adapter boundaries, communications/dashboard/observability, testing, packaging/hardening, agent handoff.
- **19–44:** runtime lifecycle, SQLite WAL durability, paper ledger/replay, crash/restart and deployment-like local validation expansion.
- **45–80:** additional local runtime/deployment transcripts, evidence models, policy/signer/destination/provider boundary checks, and aggregate validation growth.
- **81–105:** schema migration, observability/dashboard/communications probes, provider boundary/reconciliation gates, structure-manifest enforcement, and broader aggregate coverage.
- **106–129:** further deployment evidence, readiness/preflight records, hardening aggregate wiring, dashboard persistent-host readiness, and communications provider-adapter readiness.

This summary is historical context only. It is not a capability claim and must not be used to infer that a live provider, production deployment, signer, broadcast path, or external validation existed.

## Current governance

New work uses stable IDs in `ROADMAP.md` and `PRODUCTION_GAP_TRACKER.md`. `CAPABILITIES.md` is the canonical capability-state matrix. Git history is the authoritative implementation chronology.

Do not create new numbered phase files. If a historical detail matters to a current decision, inspect the relevant Git commit and verify it against present source/tests before relying on it.
