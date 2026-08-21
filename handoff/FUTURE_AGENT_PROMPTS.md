# Future Agent Prompts

These prompts are continuation templates. They do not authorize live trading, signing, external provider submission, production deployment, or credentials.

## General continuation

```text
Continue ArbyClaw from the exact current checkout. Read AGENTS.md, CAPABILITIES.md, ARCHITECTURE.md, docs/ai/ARCHITECTURE_MAP.md, docs/ai/API_CONTRACTS.md, PRODUCTION_GAP_TRACKER.md, ROADMAP.md, then relevant source/tests. Run repository hygiene, structure validation, test-collection guard, and available Rust checks before editing. Treat unavailable checks as UNVERIFIED. Verify every API/dependency against current source. Do not create numbered phase files or numeric production-readiness scores. Preserve fail-closed policy, secret/redaction, audit/state, destination, signer, and external-submission boundaries. A mock/fixture/transcript/preflight is local evidence only. Make the smallest reversible patch, add semantic regression tests, and claim completion only for checks actually executed on the exact code.
```

## Rust validation agent

```text
Validate the current ArbyClaw checkout. Run scripts/validate_repository_hygiene.py, scripts/validate_structure.py, cargo fmt --check, cargo check --workspace --locked, scripts/validate_test_collection.py, cargo test --workspace --locked, cargo clippy --workspace --all-targets --locked -- -D warnings, and scripts/validate_release_artifact.py. Run the top handoff/hardening gate if the environment supports its dependencies. Do not weaken tests or mark checks passed when they did not execute. Report PASSED, FAILED, BLOCKED, or UNVERIFIED per check with the exact commit/environment.
```

## Architecture/refactor agent

```text
Reduce ArbyClaw structural complexity without changing behavior. Prioritize mechanical decomposition of crates/arb-agent/src/main.rs, then oversized arb-core modules and broad crate-root re-exports. Preserve CLI command names, structured output fields consumed by scripts, policy/audit/state ordering, and fail-closed safety behavior. Add compatibility/regression tests before deleting old paths. Do not introduce new crates/frameworks until existing module boundaries are clear and justified.
```

## DevSecOps agent

```text
Review ArbyClaw's real build and deployment evidence. Use actual CI/tool execution for cargo audit, SBOM generation, CodeQL/SAST, secret scanning, container/image validation, release-artifact provenance, systemd/container hardening, rollback, incident and deployment-host checks. Never generate mock evidence as a substitute for a missing tool. Record missing external evidence as a gap. Do not approve production or live funds.
```

## AppSec agent

```text
Review current ArbyClaw source, tests and real scanner output. Focus on policy bypass, secret leakage, audit/state failure modes, destination/signer controls, public bind/exposure, command routing, supply chain and external-submission boundaries. Do not treat old simulated security-audit artifacts as evidence; they were removed during drift remediation. Static review alone cannot grant production readiness.
```

## Human maintainer

```text
Review the exact current ArbyClaw commit. Confirm CAPABILITIES.md and PRODUCTION_GAP_TRACKER.md match executable behavior and evidence. Require clean-checkout validation, non-vacuous tests, production-artifact smoke validation, and applicable external evidence before stronger claims. Decide the next smallest roadmap item. Live connectors, custody, signing/broadcast, withdrawals/bridges and live-funds approval require explicit human governance decisions.
```
