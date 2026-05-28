# PHASE_12_SUBROADMAP.md

## Phase

Phase 12 — Communications and CLI

## Status

Implemented for ChatGPT Project Mode as a deterministic communications and CLI model/trait boundary. Current workspace Rust/Cargo validation evidence exists and must be refreshed after changes.

## Governance Inputs

Authoritative files reconciled for this phase:

1. `ARCHITECTURE.md`
2. `ROADMAP.md`
3. `PHASE_11_SUBROADMAP.md`
4. `AGENTS.md`
5. `PRODUCTION_GAP_TRACKER.md`
6. Validated code/tests from the Phase 11 checkpoint

## Scope

Create operator-control and alerting boundaries without adding real outbound communication integrations or live execution capabilities.

In scope:

- Typed communication and CLI version marker.
- Non-secret communication boundary configuration.
- Notification channel profiles.
- Operator command model.
- Local CLI command parser boundary.
- Deterministic command router trait and implementation.
- Operator notification model.
- Deterministic notification publisher trait and implementation.
- Local dispatch records with outbound network disabled.
- Secret-like text detection, redaction, and truncation helpers.
- Structure validator update.
- Governance documentation updates.

Out of scope:

- Live trading.
- Autonomous execution commands.
- Exchange orders.
- DEX swaps.
- Transaction signing.
- Transaction broadcasts.
- Withdrawals.
- Bridges.
- Real messaging platform tokens.
- Telegram, Discord, Slack, Matrix, email, PagerDuty, Signal, or webhook delivery.
- Outbound HTTP, SMTP, WebSocket, or platform API calls.
- Real RPC or exchange calls.
- Secret storage or credential loading.

## Subsystem Boundaries

### Communications and CLI

Owns:

- Operator command parsing and routing models.
- Notification payload and dispatch-record models.
- Secret-safe message constraints.
- Local-only deterministic dispatch records.

Does not own:

- Trading execution.
- Policy bypass.
- Secrets or tokens.
- Network delivery.
- Exchange/RPC adapters.
- Wallet signing.

### Policy Engine

Remains authoritative for execution-intent approval. Phase 12 command routing does not replace or bypass policy checks.

### Execution Planner and Execution Adapter

Remain draft-only and external-submission-disabled. Phase 12 cannot invoke them to submit live orders or transactions.

## Implementation Tasks

### Task 12.1 — Communications Model Module

- Add `crates/arb-core/src/communications.rs`.
- Define a stable `COMMUNICATIONS_CLI_VERSION`.
- Define communication config, channel, command, notification, route, and dispatch record types.

### Task 12.2 — Command Routing Boundary

- Add `OperatorCommandRouter` trait.
- Add `DeterministicOperatorCommandRouter` implementation.
- Reject live execution, withdrawal, bridge, signing, and broadcast command requests.
- Keep `execution_enabled` false for every routed command.
- Keep `outbound_network_used` false for every routed command.

### Task 12.3 — Notification Boundary

- Add `NotificationPublisher` trait.
- Add `DeterministicNotificationBoundary` implementation.
- Produce local dispatch records only.
- Block or locally record channels without using external integrations.
- Keep `outbound_network_used` false for every dispatch and channel record.

### Task 12.4 — Secret-Safe Message Constraints

- Add secret-like text detection for commands, notifications, channel identifiers, route reasons, and dispatch records.
- Add redaction/truncation helper for operator-facing text.
- Reject notification payloads that look like they contain secrets.

### Task 12.5 — Exports and CLI Status Surface

- Export Phase 12 communication/CLI types from `arb-core`.
- Surface the communication/CLI boundary version in `arb-agent` status output.

### Task 12.6 — Governance and Validation

- Update `ARCHITECTURE.md`, `ROADMAP.md`, `PRODUCTION_GAP_TRACKER.md`, `HANDOFF_CONTEXT.md`, `STRUCTURE_MANIFEST.md`, `README.md`, `SECURITY.md`, and `AGENTS.md`.
- Update `scripts/validate_structure.py` to require Phase 12 files.
- Run available validation.

## Acceptance Criteria

- `PHASE_12_SUBROADMAP.md` exists.
- `crates/arb-core/src/communications.rs` exists.
- `arb-core` exports Phase 12 communications and CLI types.
- The structure validator requires Phase 12 files and passes.
- Command routing rejects unsafe live-action commands.
- Notification records never use outbound network delivery.
- Secret-like notification text is rejected before dispatch.
- Documentation clearly states that real communication integrations and live execution remain unavailable.

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

- No real messaging tokens.
- No API keys.
- No secrets in config, Markdown, source, logs, or dispatch records.
- No outbound network calls.
- No live trading commands.
- No withdrawals.
- No bridges.
- No signing.
- No transaction broadcasts.
- No policy bypass.

## Rollback Plan

1. Remove `crates/arb-core/src/communications.rs`.
2. Remove `communications` module exports from `crates/arb-core/src/lib.rs`.
3. Remove communication version output from `crates/arb-agent/src/main.rs`.
4. Revert validator Phase 12 requirements.
5. Revert governance docs to Phase 11 state.
6. Run `python3 scripts/validate_structure.py`.

## Completion Notes

Phase 12 is complete for ChatGPT Project Mode framework scope only. Real messaging adapters, authentication, channel-token storage, durable notification audit/state integration, external adapter validation, and runtime operator UX validation remain future work.
