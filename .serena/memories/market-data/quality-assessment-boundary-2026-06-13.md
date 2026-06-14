# Market-Data Quality Assessment Boundary

- Added a local-only market-data quality scoring seam in `crates/arb-core/src/market_data.rs`.
- New types/functions: `MarketDataQualityAssessmentInput`, `MarketDataQualityAssessmentStatus`, `MarketDataQualityAssessmentReport`, `assess_market_data_quality`, plus audit/state helpers and checkpoint key `MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY`.
- The boundary scores normalized quote/order-book inputs for freshness, spread, depth, and capture latency while failing closed on live-network use, credential loading, or production-ready claims.
- Added CLI surface `arb-agent validate-market-data-quality-assessment` and wired it into `.github/workflows/ci.yml`.
- Updated Phase 5 docs/tracker wording to reflect local quality scoring now existing while keeping real provider-backed quality evidence and live-provider validation open.
- Full required validation passed after the change: structure, fmt --check, cargo check, cargo test, and clippy.