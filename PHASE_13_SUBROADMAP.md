# PHASE_13_SUBROADMAP.md

## Phase

Phase 13 — Embedded Dashboard

## Status

Implemented for ChatGPT Project Mode as a deterministic embedded-dashboard model/trait boundary. Current workspace Rust/Cargo validation evidence exists and must be refreshed after changes.

## Governance Inputs

Authoritative files reconciled for this phase:

1. `ARCHITECTURE.md`
2. `ROADMAP.md`
3. `PHASE_12_SUBROADMAP.md`
4. `AGENTS.md`
5. `PRODUCTION_GAP_TRACKER.md`
6. Validated code/tests from the Phase 12 checkpoint

## Scope

Create dashboard state, rendering, and safety boundaries without adding a persistent production web server or remote operator surface.

In scope:

- Typed dashboard boundary version marker.
- Local dashboard boundary configuration.
- Server-binding model with fail-closed public-exposure rejection.
- Dashboard panel, item, severity, and snapshot models.
- Deterministic dashboard renderer trait and implementation.
- Local-only render records.
- Local audit-journal and SQLite WAL checkpoint helpers for sanitized render outcomes.
- Bounded local one-shot loopback hosted-request validation that serves sanitized rendered-dashboard body content and records byte/digest metadata.
- Local hosted-session validation summary that records accepted loopback traffic plus unauthenticated, CSRF-rejected, and rate-limited request controls without a persistent server.
- Secret-like dashboard text redaction.
- Live-control denial flags.
- Structure validator update.
- Governance documentation updates.

Out of scope:

- Live trading.
- Public web exposure.
- Persistent or production HTTP server startup.
- WebSocket, SSE, or polling endpoints.
- Authentication, sessions, cookies, OAuth, SSO, or platform identity integrations.
- Remote command execution.
- Dashboard buttons that invoke execution, signing, withdrawals, bridges, broadcasts, or live adapter submission.
- Real messaging tokens.
- Outbound HTTP, SMTP, WebSocket, or platform API calls.
- Real RPC or exchange calls.
- Secret storage or credential loading.

## Subsystem Boundaries

### Embedded Dashboard

Owns:

- Dashboard snapshot models.
- Local panel rendering records.
- Local-only rendering configuration.
- Fail-closed dashboard server-binding validation.
- Secret-safe display constraints.

Does not own:

- Trading execution.
- Policy bypass.
- Authentication/session infrastructure.
- Network listener startup.
- Secrets or tokens.
- Exchange/RPC adapters.
- Wallet signing.
- Operator command execution.

### Communications and CLI

Remain typed command and notification boundaries only. The dashboard can represent local status data but cannot invoke communications adapters or outbound integrations.

### Execution Planner and Execution Adapter

Remain draft-only and external-submission-disabled. Phase 13 dashboard models cannot submit plans, orders, swaps, transactions, or adapter requests.

## Implementation Tasks

### Task 13.1 — Dashboard Model Module

- Add `crates/arb-core/src/dashboard.rs`.
- Define a stable `DASHBOARD_BOUNDARY_VERSION`.
- Define dashboard config, server binding, snapshot, panel, item, and severity types.

### Task 13.2 — Local Rendering Boundary

- Add `DashboardRenderer` trait.
- Add `DeterministicDashboardRenderer` implementation.
- Produce local render records only.
- Keep `server_started` false for every render.
- Keep `public_network_exposed` false for every render.
- Keep `live_controls_enabled` false for every render.

### Task 13.3 — Fail-Closed Server Binding

- Reject enabled HTTP server mode in Phase 13.
- Reject public exposure and non-loopback bind hosts.
- Preserve loopback-only modeling for a future explicitly scoped server phase.

### Task 13.4 — Secret-Safe Display Constraints

- Add secret-like text detection for snapshot IDs, operator labels, panel titles, summaries, labels, values, and warnings.
- Redact secret-like display text before creating local render records.
- Reject configuration that permits secret rendering.

### Task 13.5 — Exports and CLI Status Surface

- Export Phase 13 dashboard types from `arb-core`.
- Surface the dashboard boundary version in `arb-agent` status output.

### Task 13.6 — Governance and Validation

- Update `ARCHITECTURE.md`, `ROADMAP.md`, `PRODUCTION_GAP_TRACKER.md`, `HANDOFF_CONTEXT.md`, `STRUCTURE_MANIFEST.md`, `README.md`, `SECURITY.md`, and `AGENTS.md`.
- Update `scripts/validate_structure.py` to require Phase 13 files.
- Run available validation.

## Acceptance Criteria

- `PHASE_13_SUBROADMAP.md` exists.
- `crates/arb-core/src/dashboard.rs` exists.
- `arb-core` exports Phase 13 dashboard types.
- The structure validator requires Phase 13 files and passes.
- Dashboard rendering creates local records only.
- Dashboard rendering never starts an HTTP server.
- Dashboard rendering never exposes a public network binding.
- Dashboard rendering never enables live controls.
- Dashboard render outcomes can be locally journaled and checkpointed, then recovered after audit/SQLite reopen.
- Secret-like dashboard text is redacted before render records are produced.
- Documentation clearly states that public web exposure, real server startup, authentication/session handling, and live execution controls remain unavailable.

## Validation Commands

Available in ChatGPT environment:

```bash
python3 scripts/validate_structure.py
python3 -m py_compile scripts/validate_structure.py
```

Required outside ChatGPT environment:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Security Invariants

- No real web server startup.
- No public web exposure.
- No live dashboard controls.
- No API keys.
- No secrets in config, Markdown, source, logs, snapshots, or render records.
- No outbound network calls.
- No live trading commands.
- No withdrawals.
- No bridges.
- No signing.
- No transaction broadcasts.
- No policy bypass.

## Rollback Plan

1. Remove `crates/arb-core/src/dashboard.rs`.
2. Remove `dashboard` module exports from `crates/arb-core/src/lib.rs`.
3. Remove dashboard version output from `crates/arb-agent/src/main.rs`.
4. Revert validator Phase 13 requirements.
5. Revert governance docs to Phase 12 state.
6. Run `python3 scripts/validate_structure.py`.

## Completion Notes

Phase 13 is complete for ChatGPT Project Mode framework scope only, including local sanitized dashboard render audit journal records, local hosted-dashboard security review records for CSRF/header/rate-limit controls, local hosted-request preflight records for loopback/auth/CSRF/header/rate-limit accounting, local one-shot authenticated loopback hosted-request validation that serves sanitized rendered-dashboard body content with byte/digest metadata, local hosted-session validation summaries for accepted/unauthenticated/CSRF/rate-limit request accounting, a repeatable `validate-dashboard-runtime` CLI audit/SQLite reopen gate, deployment-host report wrapper composition, and SQLite WAL checkpoint helpers. Real dashboard hosting, production authentication/session implementation, authorization implementation, CSRF token serving/enforcement from a live server, secure-header serving from a live server, runtime rate limiting beyond the bounded local probe, runtime UX validation, public-exposure validation, command-injection testing, and penetration testing remain future work.
