# Behavior Compatibility Contract

This contract governs behavior-preserving refactors of ArbyClaw. It is not a claim that static inspection proves runtime behavior. The machine-readable companion is `validation/behavior_contract.json`; CI must execute the existing Rust/CLI tests against the exact refactored commit.

## Refactor law

A structural refactor may move code, split modules, qualify imports, and replace internal ownership boundaries, but it must not intentionally change externally observable behavior in the same commit.

During structural refactors:

1. existing CLI command names remain stable;
2. exit-code semantics remain stable;
3. JSON schema names remain stable;
4. fail-closed defaults remain stable;
5. safety-relevant text/JSON fields remain semantically equivalent;
6. structural and behavioral changes are committed separately;
7. a failed equivalence/CI check blocks the refactor rather than being weakened to fit it.

## Stable safety requirements

### BC-001 — Policy fail-closed boundary

`validate-policy-decision-audit` must continue to prove that denied/approved policy paths are audited without external submission or a production-readiness claim.

### BC-002 — Destination ownership boundary

`validate-destination-boundary-audit` must remain reference/evidence based. The validator may not load signer material or sign an ownership challenge.

### BC-003 — Signer isolation boundary

`validate-signer-boundary-audit` must continue to reject unavailable signing and report no signer material load, plaintext decrypt, signing, broadcast, or RPC call.

### BC-004 — Secret lifecycle boundary

`validate-secret-boundary-audit` must remain non-secret and local: no secret material load, plaintext decrypt, keystore write, or external secret revocation.

### BC-005 — Secret backup/restore boundary

`validate-secret-backup-restore` may validate references and recovery metadata only; it must not restore external secret material or perform signing/broadcast.

### BC-006 — Withdrawal boundary

`validate-withdrawal-policy-boundary` remains blocked/fail-closed and performs no external submission.

### BC-007 — Execution-adapter boundary

`validate-execution-adapter-audit` may exercise local attempt/recovery semantics only; it must not submit externally or perform live execution.

### BC-008 — Runtime recovery contract

`validate-runtime-smoke` must preserve local recovery/operator integration while external submission, live execution, and production-readiness claims remain disabled.

### BC-009 — Communications boundary

`validate-communications-runtime` remains local and must not perform outbound provider/network delivery or live execution.

### BC-010 — Dashboard exposure boundary

`validate-dashboard-runtime` remains loopback/local and must not expose a persistent/public production server or enable live controls.

### BC-011 — Observability egress boundary

`validate-observability-runtime` must not export production telemetry or send outbound alerts.

### BC-012 — Handoff cannot self-approve

`validate-agentic-handoff-audit` must not claim external validation, production readiness, live-funds approval, or secret material recording.

## Golden-master execution contract

Before the first large Rust source move, a known-green commit is the behavioral anchor. For each mechanical extraction, CI must compare the new commit against the same command-level invariants represented above and run the full existing semantic suite. When output contains unstable values such as temporary paths, timestamps, or generated identifiers, equivalence tests should normalize only those unstable values; they must not normalize safety/status/result fields.

A future dedicated characterization harness should record, per supported command family:

- command and arguments;
- exit code;
- required stdout/JSON semantic fields;
- stderr class when applicable;
- files/artifacts created;
- SQLite/audit side effects;
- explicitly forbidden network/signing/broadcast/secret side effects.

Until such a harness is executable on a commit, source movement is not allowed to be described as behaviorally proven merely because it compiles.

## Ratchet rule

After a monolithic file is split, lower its size baseline in `validation/architecture_ratchets.json`. Never raise a baseline merely to make CI green. If new functionality genuinely needs additional code, add it in an appropriately owned module rather than regrowing a legacy monolith.
