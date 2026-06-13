# PHASE_8_SUBROADMAP.md

## Phase 8 — DEX/Web3 Connector Framework

## Objective

Create framework-only DEX/Web3 connector boundaries that future on-chain adapters can implement without broad rewrites. This phase defines chain, router, token, swap quote, and transaction-simulation models plus policy-gated DEX intent validation. It must not add live RPC calls, wallet signing, transaction broadcasts, bridges, withdrawals, private-key handling, or real DEX/router integrations.

## Governance Prerequisites

- Phase 0 governance files exist.
- Phases 1 through 7 are complete for ChatGPT Project Mode scope.
- `scripts/validate_structure.py` passes before implementation.
- `ARCHITECTURE.md`, `ROADMAP.md`, `AGENTS.md`, `PHASE_7_SUBROADMAP.md`, `HANDOFF_CONTEXT.md`, `STRUCTURE_MANIFEST.md`, and `PRODUCTION_GAP_TRACKER.md` have been reread and reconciled.
- Phase 8 implementation remains subordinate to the trust contract, policy engine, audit expectations, and deny-by-default live execution posture.

## In Scope

- DEX/Web3 framework version constant.
- Chain profile model.
- Token profile model.
- Router capability declarations.
- Router profile and registry.
- Token registry.
- Swap quote request/response models.
- Transaction simulation request/response models.
- DEX policy gate for paper/simulation-scoped swap intent validation.
- Connector trait boundaries for future quote and simulation adapters.
- Local quote/simulation lifecycle reconciliation records with audit/state recovery and duplicate intent-id rejection.
- CLI status text showing DEX/Web3 framework availability.
- Structure validator update to require Phase 8 files.
- Roadmap and gap tracker updates.

## Out of Scope

- Live RPC calls.
- Mainnet or testnet network access.
- Wallet signing or signer implementation.
- Private-key, mnemonic, seed phrase, or wallet secret handling.
- Raw transaction broadcast.
- Transaction construction from arbitrary LLM-supplied calldata.
- Cross-chain bridge execution.
- Token approvals against unknown spenders.
- Real Uniswap, Curve, Balancer, PancakeSwap, 0x, 1inch, Jupiter, or other adapter integrations.
- Production deployment, external validation, or live-funds readiness claims.

## Subsystem Boundaries

- `arb-core::dex` owns DEX/Web3 framework-only models, registries, validation, policy gating, and connector traits.
- `arb-core::policy` remains the only general execution-intent approval boundary.
- `arb-agent` may report framework availability but must not initiate RPC, signing, or trading.
- `scripts/validate_structure.py` provides static structure and no-secret assignment checks only.

## Implementation Sequence

1. Create this `PHASE_8_SUBROADMAP.md` before code changes.
2. Add `crates/arb-core/src/dex.rs` with framework-only DEX/Web3 models and deterministic validation.
3. Export DEX/Web3 framework types from `crates/arb-core/src/lib.rs`.
4. Update `crates/arb-agent/src/main.rs` status output to include DEX/Web3 framework availability.
5. Update `scripts/validate_structure.py` to require `PHASE_8_SUBROADMAP.md` and `crates/arb-core/src/dex.rs`.
6. Update `ROADMAP.md` to mark Phase 8 implemented as framework boundary and set Phase 9 as next.
7. Update `PRODUCTION_GAP_TRACKER.md` with Phase 8 completion state and remaining DEX/Web3 gaps.
8. Run available validation.

## Required Validation

Available in ChatGPT Project Mode:

```bash
python3 scripts/validate_structure.py
```

Current workspace validation now passes locally and in GitHub Actions:

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Security Requirements

- Fail closed for live DEX swaps in Phase 8.
- Fail closed for observe-scoped swap submission attempts.
- Require chain, router, token, and venue profile validation before policy evaluation.
- Keep transaction simulation request models non-broadcasting and non-signing.
- Keep local quote/simulation lifecycle reconciliation non-RPC, non-signing, non-broadcasting, non-bridging, and local-only.
- Do not store secrets, private keys, mnemonics, seed phrases, wallet tokens, RPC provider tokens, or exchange/API credentials.
- Do not accept arbitrary contract calls as executable adapter actions.
- Do not add network dependencies or connector implementations.

## Exit Criteria

Phase 8 may be marked complete for ChatGPT Project Mode only when:

- `PHASE_8_SUBROADMAP.md` exists.
- DEX/Web3 framework-only code exists in `arb-core::dex`.
- DEX/Web3 exports are available through `arb-core`.
- CLI status reports framework availability only.
- Structure validator passes.
- Roadmap and production gap tracker are updated.
- All live RPC/signing/broadcast/bridge behavior remains absent and deferred.
- Local quote/simulation lifecycle reconciliation records are audit-replayable and SQLite-recoverable without RPC, signing, broadcasts, or bridges.
- Local quote/simulation lifecycle reconciliation is covered by `arb-agent validate-connector-lifecycle-audit --workspace <fresh-dir>` together with the local/mock CEX lifecycle path, without exchange calls, RPC calls, signing, broadcasts, bridges, live execution, or production-readiness claims.
- Local signer request and signer secret-scope records are audit-replayable and SQLite-recoverable through `arb-agent validate-signer-boundary-audit --workspace <fresh-dir>` without loading keys, decrypting plaintext, signing, broadcasting, calling RPC, or claiming custody readiness.

## Rollback Plan

1. Remove `crates/arb-core/src/dex.rs`.
2. Remove DEX/Web3 exports and `pub mod dex` from `crates/arb-core/src/lib.rs`.
3. Remove DEX/Web3 CLI status text from `crates/arb-agent/src/main.rs`.
4. Revert `scripts/validate_structure.py` requirements for Phase 8 files.
5. Revert `ROADMAP.md` and `PRODUCTION_GAP_TRACKER.md` Phase 8 status updates.
6. Re-run `python3 scripts/validate_structure.py`.

## Deferred Work

- Keep Rust/Cargo validation current after future changes.
- Real chain RPC adapters.
- Testnet/mainnet transaction simulation integrations.
- Custody-backed signer implementation.
- Approval/spender management with durable audit records.
- Router-specific fee, slippage, gas, and MEV-risk validation.
- Bridge/route support after elevated risk review.
- External audit/state lifecycle integration for on-chain route planning, RPC responses, signer responses, broadcasts, confirmations, and nonce tracking beyond the local quote/simulation lifecycle boundary.
- Legal, tax, jurisdiction, terms-of-service, and protocol risk review.
