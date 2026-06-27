# Operational Resilience and Compliance Mappings

## 1. Compliance Framework Mappings

### ISO/IEC 27001
- **A.9 Access Control:** Implemented via RBAC mock tests (Phase 3). Policy Engine denies execution by default.
- **A.10 Cryptography:** TLS 1.3 enforced for connectors; local state SQLite encryption verified.
- **A.12 Operations Security:** Strict boundaries on Phase 14 (Observability) ensuring no secrets are logged. Change management handled by Phase 16/17 deployment boundaries.

### SOC 2 (Security, Availability, Processing Integrity)
- **Security:** Firewalled local interactions, secrets scanning enforced (Phase 1), and SAST/SCA integrated (Phase 2).
- **Availability:** Rate limiting and boundary enforcement on CEX/DEX connectors. Local SQLite state ensures deterministic restarts.
- **Processing Integrity:** Phase 11 Execution Adapters explicitly model attempts, fills, and failures, preventing silent drops of critical data.

### PCI DSS (v4.0)
*(Applied broadly to financial data protection, even if not handling CCs)*
- **Req 3 (Protect Stored Data):** Encryption at rest via SQLCipher.
- **Req 4 (Protect Transmitted Data):** TLS 1.3 enforced. No downgrade attacks permitted.
- **Req 10 (Log & Monitor):** Append-only audit journal implemented.

## 2. Chaos Engineering & Fault Injection (Mock Scenarios)

**Scenario 1: Unexpected Network Partition (Split-Brain)**
- *Action:* Inject `tc qdisc` rules to simulate 100% packet loss to Binance/Ethereum RPC during an active execution plan.
- *Expected Resilience:* The Execution Adapter (Phase 11) must gracefully time out, mark the intent as `FAILED`, and trigger a local reconciliation task. The Policy Engine must prevent subsequent legs of an arbitrage from firing if leg 1 fails.

**Scenario 2: Disk Exhaustion**
- *Action:* Fill the `/var/lib/arb-core` partition to 100%.
- *Expected Resilience:* The SQLite WAL commit fails. The system must `panic!` cleanly or enter a safe, read-only degradation mode rather than corrupting state or executing trades without an audit trail.

**Scenario 3: Corrupt CEX Payload (Fuzzing Response)**
- *Action:* Proxy CEX responses through a mutating proxy that alters JSON types (e.g., changing a price from `String` to `Array`).
- *Expected Resilience:* Serde deserialization fails securely. The Opportunity Engine logs a warning and discards the tick. No panic occurs.

## 3. Incident Response & Disaster Recovery
- **Runbooks:** Refer to Phase 14 boundary.
- **RTO/RPO:** Recovery Time Objective < 5 minutes (stateless restart from SQLite WAL). Recovery Point Objective < 1 second.
