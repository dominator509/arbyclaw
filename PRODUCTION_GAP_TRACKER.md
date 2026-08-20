# Production Gap Tracker

This tracker records unresolved capability gaps and their closure evidence. It is not a chronological release diary. Git history is the chronology.

## Evidence status

Current remediation work is occurring on `remediation/debug-drift-20260820`. The last independently retrieved successful GitHub Actions baseline during the 2026-08-20 remediation review was commit `0b98a9a31d3701704d950779ad989daefcf1193b` from 2026-05-26. That evidence applies to that commit only. Current remediation-branch execution must be established by fresh CI before being described as validated.

## Open gaps

| ID | Capability gap | Current state | Required closure evidence |
|---|---|---|---|
| GAP-001 | Current-head clean validation | `UNVERIFIED` | Fresh branch CI: hygiene, structure, test collection, tests, Clippy, release artifact, audit/SBOM, top hardening/handoff gate, CodeQL/Gitleaks where supported |
| GAP-002 | `arb-agent` monolithic CLI | `OPEN` | Mechanical module split with command/output compatibility tests and clean CI |
| GAP-003 | Oversized core domain modules | `OPEN` | Internal module decomposition with unchanged public semantics and clean CI |
| GAP-004 | Broad crate-root re-exports | `OPEN` | Domain-qualified import migration with API review and clean CI |
| GAP-005 | Deep validation overlap below top gate | `OPEN` | Validation DAG inventory proving each leaf executes once per top-level run without losing assertions |
| GAP-010 | Real live market-data providers | `MODELED` | Implemented provider sessions plus latency/rate-limit/outage/bad-data evidence |
| GAP-011 | Real provider-backed fee data | `MODELED` | Provider/API fee evidence, account-tier confirmation, network/gas/withdrawal-cost evidence as applicable |
| GAP-012 | Real CEX sandbox adapters | `MODELED/LOCAL FRAMEWORK` | Exchange-specific REST/WebSocket implementation; sandbox balances/order/cancel/reconciliation evidence |
| GAP-013 | Real DEX/RPC sandbox adapters | `MODELED/LOCAL FRAMEWORK` | Real RPC implementation; provider-backed simulation/nonce/receipt evidence without unauthorized broadcast |
| GAP-014 | Custody-backed signer | `LOCAL BOUNDARY ONLY` | Isolated custody implementation, secret-scope review, external security review |
| GAP-015 | Signing/broadcast | `MODELED/BLOCKED` | Explicitly authorized implementation; simulation, policy, destination, custody and external review prerequisites |
| GAP-016 | Withdrawals/bridges | `MODELED/BLOCKED` | Separate approved design, destination/custody policy, external review and explicit operator approval |
| GAP-020 | Production persistent dashboard | `MODELED/LOCAL PROBES` | Persistent host/browser implementation; authn/authz/session/CSRF/rate-limit/public-exposure security evidence |
| GAP-021 | Real outbound communications | `MODELED/LOCAL PREFLIGHT` | Provider adapters, credential isolation, delivery receipts, rate-limit/backoff/outage evidence |
| GAP-022 | Production observability | `MODELED/LOCAL PROBES` | Real exporter/log-shipping/alert sessions and deployment-host evidence |
| GAP-023 | Production deployment lifecycle | `MODELED/LOCAL PLANS` | Controlled service install/start/stop/restart/config reload evidence on target environment |
| GAP-024 | Deployment-host durability | `MODELED/LOCAL TESTS` | Real host backup/restore, graceful shutdown, audit/state recovery, schema migration, permissions, disk-full, retention/rotation evidence |
| GAP-025 | Rollback/incident execution | `MODELED/REHEARSED LOCALLY` | Controlled rollback and incident-response execution evidence |
| GAP-026 | Production load/performance | `UNVERIFIED` | Target-environment load/latency/resource testing with defined acceptance thresholds |
| GAP-027 | External scenario/fuzz/backtest breadth | `LOCAL` | Real external fuzzer/property runner and broader retained scenario/backtest corpus results |
| GAP-028 | Fresh AppSec/penetration review | `UNVERIFIED` | Current-commit real SAST/SCA plus human/AppSec/penetration evidence; no mocks counted as execution |
| GAP-029 | Production release approval | `BLOCKED` | All applicable checklist items complete and explicit accountable human approval |
| GAP-030 | Live-funds approval | `BLOCKED` | Live connector, custody, signing, deployment, audit, monitoring and operational controls externally validated plus explicit human approval |

## Closed remediation gaps

| ID | Closed condition | Evidence |
|---|---|---|
| REM-001 | Simulated/mock `security-audit/` evidence removed from canonical tree | Remediation commit `7b9843302bb4e403f668ebeb20c64229c8a64fd0` |
| REM-002 | Tracked temp/backup/Python cache/Obsidian workspace artifacts removed | Remediation commit `7b9843302bb4e403f668ebeb20c64229c8a64fd0` |
| REM-003 | Stale committed CycloneDX and Repomix snapshots removed | Remediation commit `7b9843302bb4e403f668ebeb20c64229c8a64fd0` |
| REM-004 | Repository hygiene regression guard added | `scripts/validate_repository_hygiene.py` |

## Tracker rules

- Never add a gap as “closed” because code was inspected, documentation was updated, or a mock transcript says ready.
- Never convert a missing external-evidence condition into a positive boolean merely to make an aggregate gate pass.
- Evidence must identify the commit/environment it applies to.
- A local-only validation can close a local gap, not an external or production gap.
- Production and live-funds approval require explicit human decisions; software does not self-approve.
