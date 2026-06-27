# DEX router/spender contract hygiene local gate

- Extended `DexProtocolRiskReviewRequest` with `router_allowlisted: bool` and default local constructor value `true`.
- Extended `DexProtocolRiskReviewReport` with `contract_hygiene_passed`, combining router allowlist and spender hygiene outcomes.
- `DexProtocolRiskReviewRequest::review()` now blocks on `router-not-allowlisted` before reporting ready local review.
- Updated `arb-agent validate-dex-protocol-risk-review` to print `contract-hygiene-ready` and expect 10 blocked-path blocker codes.
- Tracker/roadmap/architecture wording was reconciled to say local router/spender contract hygiene exists, and stale `no-broadcast-until-approved` wording was narrowed to future live-adapter external-fixture enforcement where applicable.
- Local validation on 2026-06-13: `rtk cargo test -p arb-core dex_protocol_risk_review -- --nocapture`, `rtk cargo run -p arb-agent -- validate-dex-protocol-risk-review`, full structure/fmt/check/test/clippy sequence passed.