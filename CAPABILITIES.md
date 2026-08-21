# Capability State Matrix

This file is the canonical statement of what ArbyClaw can actually do. It replaces numeric production-readiness percentages.

## State definitions

| State | Meaning |
|---|---|
| `MODELED` | Types, plans, reviews, or fixtures exist; the real integration does not. |
| `LOCAL` | Executable local implementation exists without the real external provider/system. |
| `INTEGRATED_LOCAL` | The capability participates in the local runtime/validation path and has executable regression coverage. |
| `EXTERNALLY_VALIDATED` | The real external system/provider/environment has been exercised with retained evidence. |
| `PRODUCTION_APPROVED` | Applicable external validation is complete and an accountable human has explicitly approved production use. |

A higher state may only be claimed when evidence for that state exists. Local transcripts, mocks, plans, fixtures, dry runs, and preflights do not establish external validation.

## Current capabilities

| Capability | Current state | What is real now | Missing before next state |
|---|---|---|---|
| Configuration + mode gates | `INTEGRATED_LOCAL` | Typed config, migrations, fail-closed mode constraints | External deployment-host validation |
| Policy engine | `INTEGRATED_LOCAL` | Deny-by-default intent evaluation and local audit/state integration | External production review |
| Audit journal | `INTEGRATED_LOCAL` | Append-only local journal, replay, failure handling | Deployment-host durability/rotation evidence |
| SQLite WAL state | `INTEGRATED_LOCAL` | WAL store, migration, recovery, crash/restart tests | Deployment-host durability/load evidence |
| Secret references/redaction | `INTEGRATED_LOCAL` | Reference-only/local secret boundaries and redaction checks | Production secret backend and external review |
| Destination controls | `INTEGRATED_LOCAL` | Local allowlist/evidence-reference enforcement | Real ownership validation process |
| Paper market data + fees | `INTEGRATED_LOCAL` | Deterministic local providers and fixtures | Provider-backed calibration |
| Paper execution + balance ledger | `INTEGRATED_LOCAL` | Local policy-gated execution simulation and ledgering | External calibration against venue behavior |
| Opportunity engine | `INTEGRATED_LOCAL` | Deterministic local discovery/ranking/replay | Broader external scenario corpus |
| Execution planner | `INTEGRATED_LOCAL` | Draft-only plan construction and policy preflight | Real adapter integration |
| Execution adapter framework | `INTEGRATED_LOCAL` | Local attempt/recovery modeling; external submission blocked | Real sandbox adapter |
| CEX framework | `LOCAL` | Profiles, fixtures, transcripts, rate-limit/credential-scope reviews | Real REST/WebSocket sandbox implementation |
| DEX/Web3 framework | `LOCAL` | Request/receipt/nonce/simulation/non-broadcast review models | Real RPC sandbox implementation |
| Live market-data provider | `MODELED` | Local request/preflight/reconciliation boundaries only | Real provider session implementation and evidence |
| Live fee provider | `MODELED` | Local reconciliation/boundary reviews | Real provider/API fee evidence |
| Custody/signer | `LOCAL` | Fail-closed signer boundary and authorization records | Custody-backed signer implementation and review |
| Transaction signing/broadcast | `MODELED` | Non-signing/non-broadcast safety reviews only | Real implementation, sandbox validation, approval |
| Withdrawals/bridges | `MODELED` | Explicit denial/policy boundaries | Separate approved design and external review |
| Communications CLI/local routing | `INTEGRATED_LOCAL` | Local command/notification/outbox validation | Real outbound provider implementation |
| Outbound communications | `MODELED` | Provider plan/preflight/readiness records only | Real provider integration and delivery evidence |
| Dashboard local surface | `INTEGRATED_LOCAL` | Local rendering/auth/session/loopback validation | Persistent hosted implementation and security validation |
| Production dashboard hosting | `MODELED` | Readiness/preflight models only | Real server/browser/deployment integration |
| Local observability | `INTEGRATED_LOCAL` | Health/log/metric/runbook and bounded loopback probes | Production exporter/log-shipping/alert sessions |
| Production observability | `MODELED` | Provider boundary/preflight models only | Real external exporter/provider evidence |
| Release artifact | `INTEGRATED_LOCAL` | Locked release build, copied artifact, hash/provenance, smoke run | Signing/publishing/release review if required |
| Container/deployment validation | `LOCAL` | Local/example production-intent validation and plans | Real deployment-host lifecycle evidence |
| Production service deployment | `MODELED` | Systemd/container/rollback/incident plans and transcripts | Actual controlled deployment execution |
| Security audit | `LOCAL` | Real CI/static checks where executed; no simulated audit evidence is canonical | Fresh current-HEAD AppSec/penetration/external review |
| Live-funds operation | `MODELED` | Explicitly blocked | All live connectors/custody/signing/deployment controls plus explicit approval |

## Evidence rule

When evidence is unavailable, stale, mocked, or belongs to another commit, record the capability as `UNVERIFIED` in the relevant report rather than promoting its state. GitHub Actions success for an older commit is evidence for that older commit only.
