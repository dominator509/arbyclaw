# AI Architecture Map

This file is a compact navigation map for coding agents. It is intentionally shorter than `ARCHITECTURE.md` and must stay synchronized with real source ownership.

## Entry points

### `crates/arb-agent/src/main.rs`

Current binary entrypoint:

```text
main()
  -> run()
     -> run_with_args(env::args().skip(1))
        -> command dispatch
           -> local validation/config/status runners
              -> arb-core domain functions
```

The file currently owns too many responsibilities: command dispatch, CLI option parsing, embedded local fixtures, validation runner orchestration, output formatting, temporary-workspace helpers, and tests. Planned decomposition must keep `main.rs` thin while preserving command/output compatibility.

## Domain ownership

Use qualified module ownership when searching or editing:

```text
arb_core::config              typed config, migrations, runtime mode gates
arb_core::secrets             secret refs/material wrappers/lifecycle reviews
arb_core::state               in-memory + SQLite WAL state/checkpoints
arb_core::policy              deny-by-default intent policy
arb_core::audit               append-only journal/replay/retention/durability
arb_core::destination         allowlist + ownership-evidence-reference controls
arb_core::market_data         normalized data/provider boundaries/freshness
arb_core::fees                fee models/schedules/provider reviews
arb_core::paper               deterministic local provider/execution/ledger/backtest
arb_core::cex                 CEX models/fixtures/transcripts/local adapter
arb_core::dex                 DEX/Web3 models/simulation/nonce/receipt/non-broadcast
arb_core::opportunity         discovery/ranking/replay/false-positive review
arb_core::strategy            strategy/risk/venue parameters
arb_core::planner             draft-only execution planning
arb_core::execution_adapter   local attempt/recovery boundary, no real submission
arb_core::runtime             lifecycle/recovery/shutdown/preflight
arb_core::communications      local command/notification/outbox/provider preflight
arb_core::dashboard           local render/auth/session/loopback/readiness
arb_core::observability       health/log/metric/runbook/loopback/provider preflight
arb_core::testing             validation/fuzz/property/backtest metadata/support
arb_core::packaging           release/package/deployment evidence models
arb_core::hardening           external evidence/checklist boundaries
arb_core::handoff             agent/human handoff records
```

The current `arb_core` crate root re-exports many symbols. Do not infer ownership from root-level imports; trace the defining module before editing.

## Local execution flow

```text
AgentConfig
  -> local/provider-fixture market + fee inputs
  -> OpportunityEngine
  -> Strategy constraints
  -> ExecutionPlanner draft
  -> PolicyEngine / destination / signer safety preflight
  -> local ExecutionAdapter evaluation
  -> AppendOnlyAuditJournal + SqliteWalStateStore
```

No arrow in this diagram implies a real exchange/RPC/provider call. Real external adapters are separate missing capabilities listed in `CAPABILITIES.md` and `PRODUCTION_GAP_TRACKER.md`.

## Persistence boundaries

- Audit: append-only local journal with replay/hash-chain checks.
- State: SQLite WAL implementation plus in-memory testing boundary.
- Crash/restart: `crates/arb-core/tests/sqlite_wal_crash_restart.rs` exercises process-level recovery behavior.
- Current SQLite state is not SQLCipher-encrypted; do not claim at-rest encryption unless implemented later.

## Validation ownership

```text
Rust tests
   ↓
leaf `arb-agent validate-*` commands
   ↓
domain aggregate scripts
   ├── execution path
   ├── operator surface
   ├── opportunity scenario
   ├── connector scenario
   ├── deployment evidence/runtime
   └── packaging
         ↓
validate_hardening_core_gate.py
         ↓
validate_agentic_handoff_candidate_gate.py
```

`validate_agentic_handoff_candidate_gate.py` must add only handoff-specific validation above hardening-core. It must not rerun the execution/operator/opportunity/connector/deployment suites that hardening-core already owns.

Repository-level guards:
- `validate_repository_hygiene.py` — actual tracked-tree hygiene.
- `validate_structure.py` — required current files, anti-drift, AI-context, secret scan.
- `validate_test_collection.py` — per-package zero-test guard.
- `validate_release_artifact.py` — locked release build, copied artifact, hashes/provenance, artifact smoke path.

## Safety tripwires

Treat these as high-risk edit areas:
- policy decisions and kill switch;
- secret material/redaction;
- destination allowlist/ownership evidence;
- signer authorization and isolation;
- external-submission flags;
- public bind/exposure controls;
- audit append/checkpoint failure handling;
- production/live-funds claim fields.

Any change touching these areas needs explicit negative-path regression coverage.

## Current non-capabilities

Do not hallucinate implementations for:
- exchange REST/WebSocket clients;
- real Web3/RPC providers;
- custody-backed signer;
- transaction broadcast;
- withdrawal/bridge execution;
- persistent production dashboard server;
- outbound messaging provider sessions;
- production telemetry exporters/log shipping/alert delivery;
- installed production service lifecycle.

If a task needs one of those, start from the capability/gap documents and implement the real boundary only after required authorization.
