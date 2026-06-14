# Historical Market-Data Persistence Boundary

- Added a local-only historical market-data persistence seam in `crates/arb-core/src/market_data.rs`.
- New types/functions: `HistoricalMarketDataPersistenceInput`, `HistoricalMarketDataPersistenceStatus`, `HistoricalMarketDataPersistenceReport`, `validate_historical_market_data_persistence`, plus audit/state helpers and checkpoint key `MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY`.
- The boundary stores normalized quotes and order books for later local replay, retains the latest records per kind under a deterministic retention cap, and fails closed on live-network use, credential loading, or production-ready claims.
- Added CLI surface `arb-agent validate-market-data-history-persistence --workspace <fresh-dir>` and wired it into `.github/workflows/ci.yml`.
- Updated Phase 5 docs/tracker wording to reflect local historical persistence now existing while keeping downloaded/provider-backed datasets and live-provider validation open.
- Full required validation passed after the change: structure, fmt --check, cargo check, cargo test, and clippy.