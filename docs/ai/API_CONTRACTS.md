# AI API and Command Contracts

This document records stable contracts that coding agents must preserve unless a change explicitly updates the contract and its tests. ArbyClaw currently exposes a Rust library surface plus a CLI; it does not expose a production HTTP/GraphQL API.

## CLI process contract

Binary: `arb-agent`

General behavior:
- prints build identity before command output;
- exits success only when the selected command/validation succeeds;
- prints validation errors to stderr and returns failure on rejected/invalid execution;
- unknown commands fail closed and direct the operator to `--help`;
- validation workspaces are local filesystem paths and must not imply external execution.

Primary command families include:
- configuration/status and config migration;
- runtime smoke/recovery/shutdown/preflight;
- opportunity/strategy/planner validation;
- market-data and fee validation;
- CEX and DEX/Web3 fixture/boundary validation;
- policy/destination/signer/secret audit validation;
- communications/dashboard/observability validation;
- local property/fuzz-corpus/backtest/coverage validation;
- deployment transcript/rehearsal validation.

Do not add a command that claims a real provider call unless the implementation actually performs that call and the capability is authorized.

## Structured validator output contract

Leaf validators commonly emit line-oriented `key: value` fields. Aggregate Python gates may emit JSON.

Safety fields follow these semantics:
- `external_calls_performed=false` means the validator did not perform external calls; it does not prove an external integration is safe.
- `external_submission_performed=false`, `signing_performed=false`, `broadcast_performed=false`, and `live_execution_performed=false` are local negative assertions.
- `production_ready=false` / `production_readiness_claimed=false` must remain false unless explicit production approval exists.
- missing external evidence must remain represented as missing/blocking evidence rather than being converted to a passing boolean.

Changing a field name consumed by an aggregate gate is a compatibility change and requires updating all consumers plus regression tests in the same patch.

## `arb-core` library boundaries

### Configuration
`arb_core::config` owns configuration schema, loading, migrations, and runtime mode constraints. Callers must not bypass configuration guards with ad hoc environment parsing.

### Policy
`arb_core::policy` owns authorization of intents. Execution/planner/adapter paths must use policy results rather than inventing parallel allow/deny logic.

### Audit and state
`arb_core::audit` owns append-only audit semantics. `arb_core::state` owns checkpoint persistence. Side-effecting local workflows that require durability should record audit/state in the documented sequence and fail closed on required persistence failure.

### Secrets
`arb_core::secrets` owns secret references and local secret material wrappers. Secret values must not be serialized into reports, audit metadata, dashboard/communication output, prompts, or repository files.

### Destination and signer
`arb_core::destination` and `arb_core::signer` are independent safety boundaries. Policy approval alone does not authorize an unknown destination or make signer material available.

### Market data and fees
`arb_core::market_data` and `arb_core::fees` define normalized local/provider boundaries. Local fixtures are deterministic test inputs, not live feeds.

### Paper execution
`arb_core::paper` may model fills, balances, P&L, replay and backtest behavior locally. Paper outcomes must not be described as actual exchange fills.

### CEX
`arb_core::cex` currently supports framework/local deterministic behavior, fixture/transcript parsing, rate-limit/credential-scope/governance reviews and live-adapter boundary checks. It does not currently provide production REST/WebSocket order submission.

### DEX/Web3
`arb_core::dex` currently supports local request/simulation/nonce/receipt/non-broadcast models and reviews. It does not currently provide production RPC submission, signing, or broadcast.

### Opportunity / strategy / planner / adapter
These domains form the local decision path. Planner output is draft-only; the execution adapter remains a local boundary and must not turn a draft into real external execution without a separately implemented and authorized adapter.

### Communications / dashboard / observability
These modules contain local runtime/probe/provider-preflight boundaries. They do not constitute production provider integrations or persistent public services.

## Aggregate gate ownership

- `validate_execution_path_gate.py` owns execution-path aggregate assertions.
- `validate_operator_surface_gate.py` owns communications/dashboard/observability local aggregate assertions.
- `validate_opportunity_scenario_gate.py` owns opportunity/test-corpus aggregate assertions.
- `validate_connector_scenario_gate.py` owns market-data/fee/CEX/DEX/Web3 aggregate assertions.
- `validate_packaging_deployment_gate.py` and deployment scripts own package/deployment local evidence.
- `validate_hardening_core_gate.py` owns the combined hardening surface.
- `validate_agentic_handoff_candidate_gate.py` owns only handoff-specific audit plus one hardening-core result.

A parent gate must not duplicate child execution merely to restate the same result.

## Capability-state contract

Only the state names in `CAPABILITIES.md` may be used as canonical capability claims. A CI pass can establish execution evidence for a commit, but cannot automatically promote a capability to `EXTERNALLY_VALIDATED` or `PRODUCTION_APPROVED` unless that CI actually executes the applicable real external environment and approval requirements.

## Compatibility expectations

When refactoring oversized source files:
- preserve CLI command names;
- preserve required key/value and JSON fields consumed by scripts;
- preserve error/fail-closed behavior;
- preserve audit/checkpoint ordering where tested;
- preserve deterministic fixture semantics;
- add or update tests before deleting compatibility shims.

Do not preserve a hallucinated or never-implemented API merely because an old mock document mentioned it. The real source and executable tests define compatibility.
