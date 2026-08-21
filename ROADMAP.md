# Roadmap

ArbyClaw no longer uses numbered implementation phases or percentage production-readiness scores. Work is tracked by stable work-item IDs, explicit acceptance criteria, and the capability states in `CAPABILITIES.md`.

## Priority 0 — restore current-head execution evidence

### ARB-RM-001 — Clean current-head CI baseline
**Goal:** prove the remediation branch builds and validates from a clean checkout.

Acceptance criteria:
- repository hygiene and anti-drift validation pass;
- both Rust packages collect at least one test;
- formatting, locked workspace check, tests, and Clippy pass;
- locked release artifact is built and smoke-tested;
- dependency audit and ephemeral SBOM generation pass;
- top handoff/hardening gate passes without duplicate top-level suite execution;
- CodeQL and Gitleaks complete successfully where the GitHub environment supports them.

No production capability state may be promoted merely because this work item closes.

## Priority 1 — reduce cognitive and validation complexity

### ARB-RM-010 — Decompose `arb-agent` CLI
Move command dispatch, fixtures, parsers, validation runners, and report formatting out of the monolithic `crates/arb-agent/src/main.rs` into focused modules. Preserve command names and output contracts unless a separately reviewed compatibility change is required.

### ARB-RM-011 — Decompose giant core modules
Split oversized domain modules such as CEX, DEX/Web3, communications, dashboard, market data, and observability into internal submodules by responsibility. Do not introduce new crates until module boundaries are proven useful.

### ARB-RM-012 — Reduce crate-root re-exports
Prefer domain-qualified imports (`arb_core::dex::...`, `arb_core::communications::...`) so symbol ownership is visible to humans and coding agents.

### ARB-RM-013 — Validation DAG consolidation
Continue replacing recursive process spawning with single-execution leaf validation and structured result aggregation. The immediate remediation removes CI/top-handoff duplication; deeper deployment/runtime overlaps should be collapsed only with regression evidence.

### ARB-RM-014 — Common evidence envelope
Evaluate a small shared evidence type for universal safety facts such as external calls, credential access, signing, broadcast, live execution, and production approval. Preserve domain-specific typed evidence and redaction boundaries.

## Priority 2 — improve real-world validation

### ARB-RM-020 — Deployment-host durability evidence
Execute audit/state recovery, permissions, backup/restore, graceful shutdown, schema migration, rotation/retention, disk-full, rollback, and incident drills on a controlled external host. Retain non-secret evidence references.

### ARB-RM-021 — Broader external scenario corpus
Run larger opportunity, backtest, fuzz/property, latency, and failure corpora outside the local deterministic fixtures. Record exact corpus identity and results.

### ARB-RM-022 — Production observability implementation
Implement and externally validate the real exporter/log-shipping/alert-delivery path before promoting production observability beyond `MODELED`.

### ARB-RM-023 — Persistent dashboard implementation
Implement persistent hosted runtime/browser integration and complete security validation before promoting production dashboard hosting beyond `MODELED`.

### ARB-RM-024 — Outbound communications implementation
Implement real provider adapters with credential isolation, idempotency, rate limits, retry/backoff, outage handling, receipts, and delivery evidence before promotion beyond `MODELED`.

## Priority 3 — live connector program (separate approval boundary)

These items require explicit human authorization before implementation because they introduce real external execution surfaces.

### ARB-RM-030 — Live CEX sandbox adapters
Implement real exchange-specific REST/WebSocket sandbox adapters with provider-backed market data, balances, order lifecycle, cancellation, rate-limit behavior, and fee evidence.

### ARB-RM-031 — Live DEX/RPC sandbox adapters
Implement real RPC/provider integrations and simulation-only transaction lifecycle handling. Signing and broadcast remain separately gated.

### ARB-RM-032 — Custody-backed signer
Design and implement isolated custody/signing with external review. No LLM or policy-bypassing path may directly access signing material.

### ARB-RM-033 — Signing/broadcast controls
Only after custody, provider, policy, destination, simulation, audit, and external review prerequisites exist, implement controlled signing/broadcast with explicit operator approval.

### ARB-RM-034 — Live-funds approval
Live-funds operation is a final human governance decision, not a software-generated status. It requires all applicable capability states and external evidence to be reviewed.

## Roadmap rules

1. Every promised behavior gets a stable requirement/work-item ID and acceptance test where applicable.
2. Do not create numbered `PHASE_*_SUBROADMAP.md` files; Git history preserves chronology.
3. Do not use numeric production-readiness percentages.
4. A local model, transcript, fixture, preflight, mock, or dry run may not be presented as external validation.
5. New validation layers must add a distinct assertion or execution environment; wrappers that merely rerun existing suites are rejected.
6. Keep live execution, signing, withdrawal, bridge, public exposure, and production-deployment boundaries fail-closed until explicitly authorized.
7. Update `CAPABILITIES.md` and `PRODUCTION_GAP_TRACKER.md` only when evidence or closure conditions materially change.
