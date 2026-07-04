#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, StateCheckpoint,
    StateStore, StateStoreError, VenueKind, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable market-data model version for audit and replay surfaces.
pub const MARKET_DATA_MODEL_VERSION: &str = "phase-5-market-data-v1";

/// Conservative default freshness ceiling for normalized market snapshots.
pub const DEFAULT_MARKET_DATA_FRESHNESS_MS: u64 = 5_000;

/// State-store subsystem name for local market-data validation checkpoints.
pub const MARKET_DATA_STATE_SUBSYSTEM: &str = "market-data";

/// Checkpoint key for the latest local market-data provider preflight validation.
pub const MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY: &str =
    "market_data.last_provider_preflight";

/// Checkpoint key for the latest local market-data reconnect plan validation.
pub const MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY: &str = "market_data.last_reconnect_plan";

/// Checkpoint key for the latest local paid market-data provider evaluation.
pub const MARKET_DATA_LAST_PAID_PROVIDER_EVALUATION_CHECKPOINT_KEY: &str =
    "market_data.last_paid_provider_evaluation";

/// Checkpoint key for the latest local market-data quality assessment.
pub const MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY: &str =
    "market_data.last_quality_assessment";

/// Checkpoint key for the latest local historical market-data persistence batch.
pub const MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY: &str =
    "market_data.last_historical_persistence";

/// Stable version for local market-data provider latency/backpressure review records.
pub const MARKET_DATA_PROVIDER_LATENCY_REVIEW_VERSION: &str =
    "market-data-provider-latency-review-v1";

/// Stable version for local market-data provider rate-limit/outage reconciliation records.
pub const MARKET_DATA_PROVIDER_RECONCILIATION_REVIEW_VERSION: &str =
    "market-data-provider-reconciliation-review-v1";

/// A normalized base/quote market pair.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketPair {
    /// Base asset symbol, such as BTC or ETH.
    pub base: String,
    /// Quote asset symbol, such as USD, USDT, or USDC.
    pub quote: String,
}

impl MarketPair {
    /// Create a normalized market pair.
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Result<Self, MarketDataError> {
        let pair = Self {
            base: normalize_symbol(base.into()),
            quote: normalize_symbol(quote.into()),
        };
        pair.validate()?;
        Ok(pair)
    }

    /// Validate this pair without mutating it.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_symbol("base", &self.base, &mut violations);
        validate_symbol("quote", &self.quote, &mut violations);

        if !self.base.trim().is_empty() && self.base.eq_ignore_ascii_case(&self.quote) {
            violations.push(MarketDataViolation::new(
                "PAIR_BASE_EQUALS_QUOTE",
                "market pair base and quote symbols must differ",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }

    /// Return a stable slash-delimited pair symbol.
    #[must_use]
    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

/// One price level from a normalized order book.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PriceLevel {
    /// Price in quote units per one base unit.
    pub price_quote: f64,
    /// Available quantity in base units.
    pub quantity_base: f64,
}

impl PriceLevel {
    /// Create one price level after deterministic validation.
    pub fn new(price_quote: f64, quantity_base: f64) -> Result<Self, MarketDataError> {
        let level = Self {
            price_quote,
            quantity_base,
        };
        level.validate("level")?;
        Ok(level)
    }

    /// Validate this price level.
    pub fn validate(&self, side_label: &'static str) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        self.collect_violations(side_label, &mut violations);

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }

    fn collect_violations(
        &self,
        side_label: &'static str,
        violations: &mut Vec<MarketDataViolation>,
    ) {
        if !is_positive_finite(self.price_quote) {
            violations.push(MarketDataViolation::new_owned(
                "PRICE_LEVEL_PRICE_INVALID",
                format!("{side_label} price must be positive and finite"),
            ));
        }

        if !is_positive_finite(self.quantity_base) {
            violations.push(MarketDataViolation::new_owned(
                "PRICE_LEVEL_QUANTITY_INVALID",
                format!("{side_label} quantity must be positive and finite"),
            ));
        }
    }

    /// Return this level's notional in quote units.
    #[must_use]
    pub fn notional_quote(&self) -> f64 {
        self.price_quote * self.quantity_base
    }
}

/// Best bid/ask quote extracted from an order book or ticker provider.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedQuote {
    /// Stable quote id from the provider/runtime.
    pub id: String,
    /// Venue that produced this quote.
    pub venue: VenueRef,
    /// Market pair.
    pub pair: MarketPair,
    /// Best bid level.
    pub bid: PriceLevel,
    /// Best ask level.
    pub ask: PriceLevel,
    /// Provider capture timestamp in Unix milliseconds.
    pub captured_at_unix_ms: u64,
    /// Local receive timestamp in Unix milliseconds.
    pub received_at_unix_ms: u64,
}

impl NormalizedQuote {
    /// Validate the quote for later strategy consumption.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id("quote", &self.id, &mut violations);
        validate_venue(&self.venue, &mut violations);

        if let Err(MarketDataError::ValidationFailed {
            violations: pair_violations,
        }) = self.pair.validate()
        {
            violations.extend(pair_violations);
        }

        self.bid.collect_violations("bid", &mut violations);
        self.ask.collect_violations("ask", &mut violations);

        if self.bid.price_quote >= self.ask.price_quote {
            violations.push(MarketDataViolation::new(
                "QUOTE_SPREAD_CROSSED_OR_LOCKED",
                "best bid must be lower than best ask for a normalized quote",
            ));
        }

        validate_timestamps(
            self.captured_at_unix_ms,
            self.received_at_unix_ms,
            &mut violations,
        );

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }

    /// Return the midpoint price in quote units.
    #[must_use]
    pub fn mid_price_quote(&self) -> f64 {
        (self.bid.price_quote + self.ask.price_quote) / 2.0
    }

    /// Return the spread in quote units.
    #[must_use]
    pub fn spread_quote(&self) -> f64 {
        self.ask.price_quote - self.bid.price_quote
    }

    /// Return the spread in basis points relative to mid-price.
    #[must_use]
    pub fn spread_bps(&self) -> f64 {
        let mid = self.mid_price_quote();
        if mid <= 0.0 || !mid.is_finite() {
            return f64::NAN;
        }
        (self.spread_quote() / mid) * 10_000.0
    }

    /// Determine quote freshness at a caller-supplied runtime timestamp.
    #[must_use]
    pub fn freshness(&self, now_unix_ms: u64, max_age_ms: u64) -> FreshnessStatus {
        classify_freshness(self.received_at_unix_ms, now_unix_ms, max_age_ms)
    }
}

/// Normalized order-book snapshot.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderBookSnapshot {
    /// Stable snapshot id from provider/runtime.
    pub id: String,
    /// Venue that produced this book.
    pub venue: VenueRef,
    /// Market pair.
    pub pair: MarketPair,
    /// Provider capture timestamp in Unix milliseconds.
    pub captured_at_unix_ms: u64,
    /// Local receive timestamp in Unix milliseconds.
    pub received_at_unix_ms: u64,
    /// Bid side sorted best-first by provider/normalizer.
    pub bids: Vec<PriceLevel>,
    /// Ask side sorted best-first by provider/normalizer.
    pub asks: Vec<PriceLevel>,
    /// Optional provider sequence or checksum reference; never a secret.
    pub source_sequence: Option<String>,
}

impl OrderBookSnapshot {
    /// Validate this book for strategy consumption.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id("order book", &self.id, &mut violations);
        validate_venue(&self.venue, &mut violations);

        if let Err(MarketDataError::ValidationFailed {
            violations: pair_violations,
        }) = self.pair.validate()
        {
            violations.extend(pair_violations);
        }

        if self.bids.is_empty() {
            violations.push(MarketDataViolation::new(
                "ORDER_BOOK_BIDS_EMPTY",
                "order book snapshot must contain at least one bid level",
            ));
        }

        if self.asks.is_empty() {
            violations.push(MarketDataViolation::new(
                "ORDER_BOOK_ASKS_EMPTY",
                "order book snapshot must contain at least one ask level",
            ));
        }

        collect_levels("bid", &self.bids, &mut violations);
        collect_levels("ask", &self.asks, &mut violations);
        validate_timestamps(
            self.captured_at_unix_ms,
            self.received_at_unix_ms,
            &mut violations,
        );

        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            if best_bid.price_quote >= best_ask.price_quote {
                violations.push(MarketDataViolation::new(
                    "ORDER_BOOK_CROSSED_OR_LOCKED",
                    "best bid must be lower than best ask for a normalized order book",
                ));
            }
        }

        if let Some(source_sequence) = self.source_sequence.as_deref() {
            if source_sequence.trim().is_empty() {
                violations.push(MarketDataViolation::new(
                    "SOURCE_SEQUENCE_EMPTY",
                    "source sequence cannot be empty when provided",
                ));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }

    /// Return the best bid if present.
    #[must_use]
    pub fn best_bid(&self) -> Option<PriceLevel> {
        self.bids.first().copied()
    }

    /// Return the best ask if present.
    #[must_use]
    pub fn best_ask(&self) -> Option<PriceLevel> {
        self.asks.first().copied()
    }

    /// Convert this snapshot into a top-of-book quote.
    pub fn to_quote(&self) -> Result<NormalizedQuote, MarketDataError> {
        self.validate()?;
        let bid = self
            .best_bid()
            .ok_or_else(|| MarketDataError::InvariantViolation {
                reason: "validated book unexpectedly lacks best bid".to_owned(),
            })?;
        let ask = self
            .best_ask()
            .ok_or_else(|| MarketDataError::InvariantViolation {
                reason: "validated book unexpectedly lacks best ask".to_owned(),
            })?;

        let quote = NormalizedQuote {
            id: format!("{}:top-of-book", self.id),
            venue: self.venue.clone(),
            pair: self.pair.clone(),
            bid,
            ask,
            captured_at_unix_ms: self.captured_at_unix_ms,
            received_at_unix_ms: self.received_at_unix_ms,
        };
        quote.validate()?;
        Ok(quote)
    }

    /// Determine snapshot freshness at a caller-supplied runtime timestamp.
    #[must_use]
    pub fn freshness(&self, now_unix_ms: u64, max_age_ms: u64) -> FreshnessStatus {
        classify_freshness(self.received_at_unix_ms, now_unix_ms, max_age_ms)
    }
}

/// Market-data freshness classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessStatus {
    /// Snapshot age is within allowed bounds.
    Fresh { age_ms: u64 },
    /// Snapshot age exceeds the configured limit.
    Stale { age_ms: u64, max_age_ms: u64 },
    /// Snapshot appears to come from the future relative to local runtime clock.
    FutureTimestamp { future_by_ms: u64 },
}

impl FreshnessStatus {
    /// Return true only when this status is fresh.
    #[must_use]
    pub const fn is_fresh(self) -> bool {
        matches!(self, Self::Fresh { .. })
    }
}

/// Capabilities advertised by a market-data provider implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataCapabilities {
    /// Provider supports order-book snapshots.
    pub order_book: bool,
    /// Provider supports top-of-book/ticker quotes.
    pub top_of_book: bool,
    /// Provider supports fee lookup or fee hints.
    pub fees: bool,
    /// Provider uses WebSocket streams.
    pub websocket: bool,
    /// Provider uses REST polling.
    pub rest: bool,
}

impl MarketDataCapabilities {
    /// Return conservative no-network capabilities for scaffolds/tests.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            order_book: false,
            top_of_book: false,
            fees: false,
            websocket: false,
            rest: false,
        }
    }
}

/// Request for provider-owned market data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataRequest {
    /// Target venue.
    pub venue: VenueRef,
    /// Target pair.
    pub pair: MarketPair,
    /// Maximum acceptable quote age in milliseconds.
    pub max_age_ms: u64,
}

impl MarketDataRequest {
    /// Validate this request before a provider handles it.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_venue(&self.venue, &mut violations);

        if let Err(MarketDataError::ValidationFailed {
            violations: pair_violations,
        }) = self.pair.validate()
        {
            violations.extend(pair_violations);
        }

        if self.max_age_ms == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_MAX_AGE_ZERO",
                "market-data requests must provide a positive max_age_ms",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

/// Caller-supplied local market-data provider health observation.
///
/// This record is a non-secret local preflight input. It does not open sockets,
/// authenticate to providers, download market data, or measure live networks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataProviderHealthObservation {
    /// Stable provider name or local fixture label.
    pub provider_name: String,
    /// Whether the provider was configured as a local/read-only source.
    pub read_only: bool,
    /// Whether the caller observed a rate-limit condition.
    pub rate_limited: bool,
    /// Whether the caller observed a provider outage condition.
    pub outage_observed: bool,
    /// Whether reconnect/backoff would be required before use.
    pub reconnect_required: bool,
    /// Whether reconnect/backoff was locally planned instead of attempted.
    pub reconnect_backoff_planned: bool,
    /// Number of local samples inspected by the caller.
    pub samples_checked: u64,
    /// Fresh samples among the inspected local samples.
    pub fresh_samples: u64,
    /// Stale samples among the inspected local samples.
    pub stale_samples: u64,
    /// Maximum local receive latency observed in milliseconds.
    pub max_observed_latency_ms: u64,
    /// Allowed receive latency ceiling in milliseconds.
    pub max_allowed_latency_ms: u64,
    /// Whether any live network call was performed. Must remain false here.
    pub live_network_used: bool,
    /// Whether any provider credential was loaded. Must remain false here.
    pub credential_loaded: bool,
}

/// Local market-data provider preflight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketDataProviderPreflightStatus {
    /// Local observations allow later read-only use.
    Usable,
    /// Local observations require fail-closed handling before use.
    Blocked,
}

/// Non-secret local market-data provider preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataProviderPreflightReport {
    /// Provider name or local fixture label.
    pub provider_name: String,
    /// Overall local preflight status.
    pub status: MarketDataProviderPreflightStatus,
    /// Whether read-only mode was confirmed.
    pub read_only_confirmed: bool,
    /// Whether local rate-limit handling blocked use.
    pub rate_limit_blocked: bool,
    /// Whether local outage handling blocked use.
    pub outage_blocked: bool,
    /// Whether reconnect/backoff is required before use.
    pub reconnect_required: bool,
    /// Whether reconnect/backoff was planned locally.
    pub reconnect_backoff_planned: bool,
    /// Number of local samples inspected.
    pub samples_checked: u64,
    /// Fresh local sample count.
    pub fresh_samples: u64,
    /// Stale local sample count.
    pub stale_samples: u64,
    /// Whether any stale local sample blocked use.
    pub stale_data_blocked: bool,
    /// Maximum local receive latency observed in milliseconds.
    pub max_observed_latency_ms: u64,
    /// Allowed receive latency ceiling in milliseconds.
    pub max_allowed_latency_ms: u64,
    /// Whether local latency exceeded the configured ceiling.
    pub latency_blocked: bool,
    /// Whether live network use occurred. Always false for this boundary.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Always false for this boundary.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Caller-supplied local reconnect/backoff plan for a market-data provider.
///
/// This validates timing and fail-closed metadata only. It does not open REST
/// sessions, WebSocket streams, provider accounts, or credential material.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataReconnectPlanInput {
    /// Stable local plan id.
    pub plan_id: String,
    /// Stable provider name or local fixture label.
    pub provider_name: String,
    /// Venue represented by the provider plan.
    pub venue: VenueRef,
    /// Local disconnect observation timestamp.
    pub disconnected_at_unix_ms: u64,
    /// Local plan creation timestamp.
    pub planned_at_unix_ms: u64,
    /// One-based reconnect attempt number.
    pub attempt_number: u32,
    /// Maximum allowed reconnect attempts before fail-closed outage handling.
    pub max_attempts: u32,
    /// Base exponential-backoff delay in milliseconds.
    pub base_backoff_ms: u64,
    /// Maximum exponential-backoff delay in milliseconds.
    pub max_backoff_ms: u64,
    /// Locally planned delay before the next attempt in milliseconds.
    pub planned_delay_ms: u64,
    /// Optional provider retry-after hint in milliseconds.
    pub provider_retry_after_ms: Option<u64>,
    /// Whether the caller observed provider-side rate limiting.
    pub rate_limited: bool,
    /// Whether the caller observed an active provider outage.
    pub outage_observed: bool,
    /// Whether a live network call occurred. Must remain false here.
    pub live_network_used: bool,
    /// Whether a WebSocket connection was opened. Must remain false here.
    pub websocket_connection_opened: bool,
    /// Whether provider credentials were loaded. Must remain false here.
    pub credential_loaded: bool,
}

/// Local market-data reconnect plan validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketDataReconnectPlanStatus {
    /// Plan is coherent enough for local review before a future connector uses it.
    ReadyForLocalReview,
    /// Plan must fail closed before any reconnect attempt.
    Blocked,
}

/// Non-secret local reconnect/backoff validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataReconnectPlanReport {
    /// Stable local plan id.
    pub plan_id: String,
    /// Provider name or local fixture label.
    pub provider_name: String,
    /// Venue represented by the plan.
    pub venue: VenueRef,
    /// Validation status.
    pub status: MarketDataReconnectPlanStatus,
    /// One-based reconnect attempt number.
    pub attempt_number: u32,
    /// Maximum allowed reconnect attempts before fail-closed outage handling.
    pub max_attempts: u32,
    /// Minimum delay required by local exponential backoff.
    pub expected_backoff_ms: u64,
    /// Retry-after hint copied from caller-supplied metadata.
    pub provider_retry_after_ms: Option<u64>,
    /// Minimum effective delay after rate-limit retry-after accounting.
    pub effective_min_delay_ms: u64,
    /// Locally planned delay before any future attempt.
    pub planned_delay_ms: u64,
    /// Earliest local timestamp at which a future attempt may be considered.
    pub next_attempt_at_unix_ms: u64,
    /// Whether local rate-limit handling affected the plan.
    pub rate_limit_blocked: bool,
    /// Whether local outage handling blocked the plan.
    pub outage_blocked: bool,
    /// Whether retry attempts were exhausted.
    pub retry_budget_exhausted: bool,
    /// Whether live network use occurred. Always false for a ready report.
    pub live_network_used: bool,
    /// Whether a WebSocket connection was opened. Always false for a ready report.
    pub websocket_connection_opened: bool,
    /// Whether provider credentials were loaded. Always false for a ready report.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Caller-supplied local market-data quality assessment input.
///
/// This boundary scores already-normalized local quote/order-book records. It
/// does not fetch provider data, open sockets, or load credentials.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataQualityAssessmentInput {
    /// Stable local assessment id.
    pub assessment_id: String,
    /// Stable provider name or fixture label.
    pub provider_name: String,
    /// Request context for the assessed pair and freshness ceiling.
    pub request: MarketDataRequest,
    /// Normalized top-of-book quote to assess.
    pub quote: NormalizedQuote,
    /// Optional normalized order book for depth scoring.
    pub order_book: Option<OrderBookSnapshot>,
    /// Caller-supplied local assessment time.
    pub now_unix_ms: u64,
    /// Maximum acceptable spread in basis points.
    pub max_spread_bps: u64,
    /// Minimum required levels on both sides for depth quality.
    pub min_depth_levels: u32,
    /// Maximum acceptable capture-to-receive latency in milliseconds.
    pub max_capture_latency_ms: u64,
    /// Whether any live network was used. Must remain false here.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Must remain false here.
    pub credential_loaded: bool,
    /// Whether the caller tried to claim production readiness. Must remain false here.
    pub production_ready_claimed: bool,
}

/// Local market-data quality assessment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketDataQualityAssessmentStatus {
    /// Data quality is strong enough for local review.
    Acceptable,
    /// Data remains local-only but quality is degraded.
    Degraded,
    /// Data quality must fail closed before future use.
    Blocked,
}

/// Non-secret local market-data quality assessment report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataQualityAssessmentReport {
    /// Stable local assessment id.
    pub assessment_id: String,
    /// Provider name or fixture label.
    pub provider_name: String,
    /// Assessed request context.
    pub request: MarketDataRequest,
    /// Overall local quality status.
    pub status: MarketDataQualityAssessmentStatus,
    /// Freshness classification at assessment time.
    pub freshness_status: FreshnessStatus,
    /// Quote age in milliseconds.
    pub quote_age_ms: u64,
    /// Observed spread in basis points.
    pub observed_spread_bps: f64,
    /// Whether spread stayed within the caller limit.
    pub spread_within_limit: bool,
    /// Whether an order book was available for depth scoring.
    pub order_book_present: bool,
    /// Number of depth levels available on both sides.
    pub depth_levels_available: u32,
    /// Whether depth met the caller requirement.
    pub depth_levels_sufficient: bool,
    /// Capture-to-receive latency in milliseconds.
    pub capture_latency_ms: u64,
    /// Whether capture latency stayed within the caller limit.
    pub capture_latency_within_limit: bool,
    /// Deterministic local quality score from 0 to 100.
    pub quality_score: u8,
    /// Whether any live network was used. Always false for an acceptable report.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Always false for an acceptable report.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Caller-supplied local market-data provider latency/backpressure review input.
///
/// This composes existing local provider preflight, reconnect, quality, and
/// paid-provider dossier evidence. It does not open provider sessions, download
/// market data, inspect host resources, load credentials, or approve production.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataProviderLatencyReviewRequest {
    /// Stable local review id.
    pub review_id: String,
    /// Clean local provider preflight report.
    pub clean_preflight: MarketDataProviderPreflightReport,
    /// Degraded local provider preflight report proving fail-closed behavior.
    pub degraded_preflight: MarketDataProviderPreflightReport,
    /// Ready local reconnect/backoff report.
    pub ready_reconnect: MarketDataReconnectPlanReport,
    /// Acceptable local market-data quality report.
    pub acceptable_quality: MarketDataQualityAssessmentReport,
    /// Ready local paid-provider evaluation dossier.
    pub paid_provider_evaluation: PaidMarketDataProviderEvaluationReport,
    /// Local provider receive latency budget in milliseconds.
    pub max_provider_latency_ms: u64,
    /// Local capture-to-receive latency budget in milliseconds.
    pub max_capture_latency_ms: u64,
    /// Local reconnect delay budget in milliseconds.
    pub max_reconnect_delay_ms: u64,
    /// Minimum acceptable local quality score.
    pub min_quality_score: u8,
    /// Minimum local samples required in the clean preflight.
    pub min_samples_checked: u64,
    /// Remaining external evidence that this local review cannot satisfy.
    pub remaining_external_evidence: Vec<String>,
    /// Whether a live network was used. Must remain false here.
    pub live_network_used: bool,
    /// Whether a WebSocket connection was opened. Must remain false here.
    pub websocket_connection_opened: bool,
    /// Whether provider credentials were loaded. Must remain false here.
    pub credential_loaded: bool,
    /// Whether the caller tried to claim production readiness. Must remain false here.
    pub production_ready_claimed: bool,
}

/// Local market-data provider latency/backpressure review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketDataProviderLatencyReviewStatus {
    /// Existing local evidence is coherent enough for local review only.
    ReadyForLocalReview,
    /// Evidence is incomplete or unsafe and must fail closed.
    Blocked,
}

/// Non-secret local market-data provider latency/backpressure review report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataProviderLatencyReviewReport {
    /// Stable review schema version.
    pub version: String,
    /// Stable local review id.
    pub review_id: String,
    /// Overall local review status.
    pub status: MarketDataProviderLatencyReviewStatus,
    /// Provider represented by the clean preflight.
    pub provider_name: String,
    /// Whether clean preflight was usable and read-only.
    pub clean_preflight_ready: bool,
    /// Whether degraded preflight demonstrated fail-closed behavior.
    pub degraded_preflight_failed_closed: bool,
    /// Whether reconnect/backoff evidence is ready and within budget.
    pub reconnect_review_ready: bool,
    /// Whether quality evidence is acceptable and within budget.
    pub quality_review_ready: bool,
    /// Whether the paid-provider dossier is ready for local review.
    pub paid_provider_review_ready: bool,
    /// Whether provider receive latency stayed within the local budget.
    pub provider_latency_budget_met: bool,
    /// Whether capture-to-receive latency stayed within the local budget.
    pub capture_latency_budget_met: bool,
    /// Whether reconnect delay stayed within the local budget.
    pub reconnect_delay_budget_met: bool,
    /// Whether local sample count met the review floor.
    pub sample_floor_met: bool,
    /// Whether remaining external evidence is still explicitly recorded.
    pub remaining_external_evidence_recorded: bool,
    /// Count of remaining external evidence items.
    pub remaining_external_evidence_count: usize,
    /// Whether a live network was used. Always false for a ready report.
    pub live_network_used: bool,
    /// Whether a WebSocket connection was opened. Always false for a ready report.
    pub websocket_connection_opened: bool,
    /// Whether provider credentials were loaded. Always false for a ready report.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Caller-supplied local provider rate-limit/outage reconciliation evidence.
///
/// This composes existing local preflight, reconnect, and latency/backpressure
/// reports. It does not call providers, open sockets, load credentials, or
/// approve production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataProviderReconciliationReviewRequest {
    /// Stable local review id.
    pub review_id: String,
    /// Ready local latency/backpressure review.
    pub latency_review: MarketDataProviderLatencyReviewReport,
    /// Degraded preflight proving rate-limit and outage inputs fail closed.
    pub degraded_preflight: MarketDataProviderPreflightReport,
    /// Ready reconnect/backoff report for retry-after accounting.
    pub rate_limit_reconnect: MarketDataReconnectPlanReport,
    /// Blocked reconnect/backoff report for outage exhaustion accounting.
    pub outage_reconnect: MarketDataReconnectPlanReport,
    /// Minimum local degraded samples required before this evidence is useful.
    pub min_degraded_samples_checked: u64,
    /// Remaining external evidence that this local review cannot satisfy.
    pub remaining_external_evidence: Vec<String>,
    /// Whether a live network was used. Must remain false here.
    pub live_network_used: bool,
    /// Whether a WebSocket connection was opened. Must remain false here.
    pub websocket_connection_opened: bool,
    /// Whether provider credentials were loaded. Must remain false here.
    pub credential_loaded: bool,
    /// Whether the caller tried to claim production readiness. Must remain false here.
    pub production_ready_claimed: bool,
}

/// Local provider rate-limit/outage reconciliation review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketDataProviderReconciliationReviewStatus {
    /// Existing local evidence is coherent enough for local review only.
    ReadyForLocalReview,
    /// Evidence is incomplete or unsafe and must fail closed.
    Blocked,
}

/// Non-secret local provider rate-limit/outage reconciliation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDataProviderReconciliationReviewReport {
    /// Stable review schema version.
    pub version: String,
    /// Stable local review id.
    pub review_id: String,
    /// Overall local review status.
    pub status: MarketDataProviderReconciliationReviewStatus,
    /// Provider represented by the local evidence.
    pub provider_name: String,
    /// Whether prerequisite latency/backpressure review is ready.
    pub latency_review_ready: bool,
    /// Whether degraded preflight blocked on rate-limit evidence.
    pub rate_limit_fail_closed: bool,
    /// Whether degraded preflight blocked on outage evidence.
    pub outage_fail_closed: bool,
    /// Whether stale data was also blocked during degraded provider evidence.
    pub stale_data_fail_closed: bool,
    /// Whether degraded provider latency was blocked.
    pub latency_fail_closed: bool,
    /// Whether local degraded sample count met the review floor.
    pub degraded_sample_floor_met: bool,
    /// Whether retry-after/backoff accounting produced a ready reconnect plan.
    pub rate_limit_reconnect_ready: bool,
    /// Whether outage exhaustion produced a blocked reconnect plan.
    pub outage_reconnect_blocked: bool,
    /// Whether remaining external evidence is still explicitly recorded.
    pub remaining_external_evidence_recorded: bool,
    /// Count of remaining external evidence items.
    pub remaining_external_evidence_count: usize,
    /// Whether a live network was used. Always false for a ready report.
    pub live_network_used: bool,
    /// Whether a WebSocket connection was opened. Always false for a ready report.
    pub websocket_connection_opened: bool,
    /// Whether provider credentials were loaded. Always false for a ready report.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Caller-supplied local historical market-data persistence input.
///
/// This boundary persists already-normalized quotes and order books for later
/// local replay. It does not fetch provider data, open sockets, or load
/// credentials.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalMarketDataPersistenceInput {
    /// Stable local batch id.
    pub batch_id: String,
    /// Stable provider name or fixture label.
    pub provider_name: String,
    /// Venue represented by the persisted batch.
    pub venue: VenueRef,
    /// Market pair represented by the persisted batch.
    pub pair: MarketPair,
    /// Normalized quotes to persist.
    pub quotes: Vec<NormalizedQuote>,
    /// Normalized order books to persist.
    pub order_books: Vec<OrderBookSnapshot>,
    /// Maximum number of retained records per kind.
    pub max_retained_records_per_kind: u32,
    /// Caller-supplied local persistence timestamp.
    pub persisted_at_unix_ms: u64,
    /// Whether any live network was used. Must remain false here.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Must remain false here.
    pub credential_loaded: bool,
    /// Whether the caller tried to claim production readiness. Must remain false here.
    pub production_ready_claimed: bool,
}

/// Local historical market-data persistence status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HistoricalMarketDataPersistenceStatus {
    /// History batch was stored for later local replay.
    PersistedForLocalReplay,
    /// History batch must fail closed before future use.
    Blocked,
}

/// Non-secret local historical market-data persistence report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalMarketDataPersistenceReport {
    /// Stable local batch id.
    pub batch_id: String,
    /// Provider name or fixture label.
    pub provider_name: String,
    /// Venue represented by the persisted batch.
    pub venue: VenueRef,
    /// Market pair represented by the persisted batch.
    pub pair: MarketPair,
    /// Overall local persistence status.
    pub status: HistoricalMarketDataPersistenceStatus,
    /// Persisted quotes after deterministic truncation.
    pub stored_quotes: Vec<NormalizedQuote>,
    /// Persisted order books after deterministic truncation.
    pub stored_order_books: Vec<OrderBookSnapshot>,
    /// Whether quotes were truncated to the configured retention ceiling.
    pub quotes_truncated: bool,
    /// Whether order books were truncated to the configured retention ceiling.
    pub order_books_truncated: bool,
    /// Oldest received timestamp across stored records, if any.
    pub oldest_received_at_unix_ms: Option<u64>,
    /// Newest received timestamp across stored records, if any.
    pub newest_received_at_unix_ms: Option<u64>,
    /// Window span across stored records.
    pub window_span_ms: u64,
    /// Maximum retained record count per kind.
    pub max_retained_records_per_kind: u32,
    /// Whether any live network was used. Always false for a persisted report.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Always false for a persisted report.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Caller-supplied local paid market-data provider evaluation input.
///
/// This record captures sanitized comparison metadata only. It does not open
/// provider connections, sign contracts, provision accounts, or load
/// credentials.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaidMarketDataProviderEvaluationInput {
    /// Stable local evaluation id.
    pub evaluation_id: String,
    /// Stable provider name or local fixture label.
    pub provider_name: String,
    /// Venues this provider is expected to cover.
    pub covered_venues: Vec<VenueRef>,
    /// Market pairs this provider is expected to cover.
    pub covered_pairs: Vec<MarketPair>,
    /// Local capability snapshot for the provider.
    pub capabilities: MarketDataCapabilities,
    /// Documented or fixture-derived latency reference in milliseconds.
    pub documented_latency_ms: u64,
    /// Maximum locally accepted latency ceiling in milliseconds.
    pub max_allowed_latency_ms: u64,
    /// Documented request budget per minute for the planned account tier.
    pub max_requests_per_minute: u64,
    /// Estimated monthly spend in whole USD for the evaluated tier.
    pub monthly_cost_usd: u64,
    /// Failure modes reviewed locally from non-secret references.
    pub failure_modes_reviewed: Vec<String>,
    /// Whether rate-limit documentation was reviewed locally.
    pub rate_limit_documentation_reviewed: bool,
    /// Whether pricing documentation was reviewed locally.
    pub pricing_documentation_reviewed: bool,
    /// Whether terms and use restrictions were reviewed locally.
    pub terms_reviewed: bool,
    /// Whether planned credential scope was reviewed without loading secrets.
    pub credential_scope_reviewed: bool,
    /// Whether any live network was used. Must remain false here.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Must remain false here.
    pub credential_loaded: bool,
    /// Whether the caller tried to claim production readiness. Must remain false here.
    pub production_ready_claimed: bool,
}

/// Local paid market-data provider evaluation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaidMarketDataProviderEvaluationStatus {
    /// Comparison metadata is coherent enough for later human review.
    ReadyForLocalReview,
    /// Evaluation must fail closed before any future provider choice.
    Blocked,
}

/// Non-secret local paid market-data provider evaluation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaidMarketDataProviderEvaluationReport {
    /// Stable local evaluation id.
    pub evaluation_id: String,
    /// Provider name or local fixture label.
    pub provider_name: String,
    /// Evaluation status.
    pub status: PaidMarketDataProviderEvaluationStatus,
    /// Covered venues reviewed locally.
    pub covered_venues: Vec<VenueRef>,
    /// Covered pairs reviewed locally.
    pub covered_pairs: Vec<MarketPair>,
    /// Local capability snapshot for the provider.
    pub capabilities: MarketDataCapabilities,
    /// Documented or fixture-derived latency reference in milliseconds.
    pub documented_latency_ms: u64,
    /// Maximum locally accepted latency ceiling in milliseconds.
    pub max_allowed_latency_ms: u64,
    /// Whether the documented latency stays within the local ceiling.
    pub latency_within_budget: bool,
    /// Documented request budget per minute for the planned account tier.
    pub max_requests_per_minute: u64,
    /// Whether rate-limit documentation and budget metadata passed local review.
    pub rate_limit_review_passed: bool,
    /// Estimated monthly spend in whole USD for the evaluated tier.
    pub monthly_cost_usd: u64,
    /// Whether pricing documentation and cost metadata passed local review.
    pub cost_review_passed: bool,
    /// Failure modes reviewed locally from non-secret references.
    pub failure_modes_reviewed: Vec<String>,
    /// Whether failure behavior metadata passed local review.
    pub failure_behavior_review_passed: bool,
    /// Whether coverage metadata passed local review.
    pub coverage_review_passed: bool,
    /// Whether terms and credential-scope governance checks passed locally.
    pub governance_review_passed: bool,
    /// Whether rate-limit documentation was reviewed locally.
    pub rate_limit_documentation_reviewed: bool,
    /// Whether pricing documentation was reviewed locally.
    pub pricing_documentation_reviewed: bool,
    /// Whether terms and use restrictions were reviewed locally.
    pub terms_reviewed: bool,
    /// Whether planned credential scope was reviewed without loading secrets.
    pub credential_scope_reviewed: bool,
    /// Whether any live network was used. Always false for a ready report.
    pub live_network_used: bool,
    /// Whether provider credentials were loaded. Always false for a ready report.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

impl MarketDataProviderHealthObservation {
    /// Validate local market-data provider health observation shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id("market-data provider", &self.provider_name, &mut violations);
        if self.samples_checked == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_SAMPLES_EMPTY",
                "market-data provider preflight requires at least one local sample",
            ));
        }
        if self.fresh_samples.saturating_add(self.stale_samples) != self.samples_checked {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_SAMPLE_COUNT_MISMATCH",
                "fresh and stale market-data sample counts must equal samples_checked",
            ));
        }
        if self.max_allowed_latency_ms == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_LATENCY_LIMIT_ZERO",
                "market-data provider preflight requires a positive latency ceiling",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataProviderPreflightReport {
    /// Validate local preflight report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data provider preflight",
            &self.provider_name,
            &mut violations,
        );
        if self.samples_checked == 0
            || self.fresh_samples.saturating_add(self.stale_samples) != self.samples_checked
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_REPORT_SAMPLE_COUNT_INVALID",
                "market-data provider preflight report sample counts must be coherent",
            ));
        }
        if self.max_allowed_latency_ms == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_REPORT_LATENCY_LIMIT_ZERO",
                "market-data provider preflight report requires a positive latency ceiling",
            ));
        }
        let should_block = !self.read_only_confirmed
            || self.rate_limit_blocked
            || self.outage_blocked
            || (self.reconnect_required && !self.reconnect_backoff_planned)
            || self.stale_data_blocked
            || self.latency_blocked
            || self.live_network_used
            || self.credential_loaded;
        if should_block && self.status != MarketDataProviderPreflightStatus::Blocked {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_STATUS_SHOULD_BLOCK",
                "blocked market-data observations must produce blocked status",
            ));
        }
        if !should_block && self.status != MarketDataProviderPreflightStatus::Usable {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_STATUS_SHOULD_BE_USABLE",
                "clean market-data observations must produce usable status",
            ));
        }
        if (self.live_network_used || self.credential_loaded || self.production_ready)
            && self.status == MarketDataProviderPreflightStatus::Usable
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PREFLIGHT_FORBIDDEN_SIDE_EFFECT",
                "market-data provider preflight must remain local-only and not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataReconnectPlanInput {
    /// Validate local reconnect plan input shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id("market-data reconnect plan", &self.plan_id, &mut violations);
        validate_id("market-data provider", &self.provider_name, &mut violations);
        validate_venue(&self.venue, &mut violations);

        if self.disconnected_at_unix_ms > self.planned_at_unix_ms {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_DISCONNECT_AFTER_PLAN",
                "disconnect timestamp must not be after reconnect planning timestamp",
            ));
        }
        if self.attempt_number == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_ATTEMPT_ZERO",
                "reconnect attempt number must be one-based",
            ));
        }
        if self.max_attempts == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_MAX_ATTEMPTS_ZERO",
                "reconnect max_attempts must be positive",
            ));
        }
        if self.base_backoff_ms == 0 || self.max_backoff_ms == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_BACKOFF_ZERO",
                "reconnect backoff delays must be positive",
            ));
        }
        if self.base_backoff_ms > self.max_backoff_ms {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_BACKOFF_INVERTED",
                "base backoff must not exceed max backoff",
            ));
        }
        if self.planned_delay_ms == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_PLANNED_DELAY_ZERO",
                "planned reconnect delay must be positive",
            ));
        }
        if self.rate_limited && self.provider_retry_after_ms.is_none() {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_RATE_LIMIT_MISSING_RETRY_AFTER",
                "rate-limited reconnect plans require a retry-after hint",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataReconnectPlanReport {
    /// Validate local reconnect plan report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id("market-data reconnect plan", &self.plan_id, &mut violations);
        validate_id("market-data provider", &self.provider_name, &mut violations);
        validate_venue(&self.venue, &mut violations);

        if self.attempt_number == 0 || self.max_attempts == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_REPORT_ATTEMPTS_INVALID",
                "reconnect report attempt counters must be positive",
            ));
        }
        if self.expected_backoff_ms == 0
            || self.effective_min_delay_ms == 0
            || self.planned_delay_ms == 0
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_REPORT_DELAY_INVALID",
                "reconnect report delays must be positive",
            ));
        }
        if self.effective_min_delay_ms < self.expected_backoff_ms {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_REPORT_EFFECTIVE_DELAY_TOO_LOW",
                "effective reconnect delay must include exponential backoff",
            ));
        }
        if self.planned_delay_ms < self.effective_min_delay_ms
            && self.status != MarketDataReconnectPlanStatus::Blocked
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_REPORT_STATUS_SHOULD_BLOCK_DELAY",
                "delay-short reconnect plans must be blocked",
            ));
        }
        let side_effected =
            self.live_network_used || self.websocket_connection_opened || self.credential_loaded;
        let should_block = self.outage_blocked
            || self.retry_budget_exhausted
            || self.planned_delay_ms < self.effective_min_delay_ms
            || side_effected;
        if should_block && self.status != MarketDataReconnectPlanStatus::Blocked {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_STATUS_SHOULD_BLOCK",
                "unsafe reconnect plans must produce blocked status",
            ));
        }
        if !should_block && self.status != MarketDataReconnectPlanStatus::ReadyForLocalReview {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_STATUS_SHOULD_BE_READY",
                "coherent reconnect plans must be ready for local review",
            ));
        }
        if side_effected && self.status == MarketDataReconnectPlanStatus::ReadyForLocalReview {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_FORBIDDEN_SIDE_EFFECT",
                "market-data reconnect validation must not use network, WebSocket, or credentials",
            ));
        }
        if self.production_ready {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_RECONNECT_PRODUCTION_READY_FORBIDDEN",
                "local reconnect validation must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataQualityAssessmentInput {
    /// Validate local market-data quality assessment input shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data quality assessment",
            &self.assessment_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);
        if let Err(MarketDataError::ValidationFailed {
            violations: request_violations,
        }) = self.request.validate()
        {
            violations.extend(request_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: quote_violations,
        }) = self.quote.validate()
        {
            violations.extend(quote_violations);
        }
        if self.quote.venue != self.request.venue || self.quote.pair != self.request.pair {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_QUOTE_REQUEST_MISMATCH",
                "quality assessment quote must match the assessed request venue and pair",
            ));
        }
        if let Some(order_book) = &self.order_book {
            if let Err(MarketDataError::ValidationFailed {
                violations: order_book_violations,
            }) = order_book.validate()
            {
                violations.extend(order_book_violations);
            }
            if order_book.venue != self.request.venue || order_book.pair != self.request.pair {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_QUALITY_ORDER_BOOK_REQUEST_MISMATCH",
                    "quality assessment order book must match the assessed request venue and pair",
                ));
            }
        }
        if self.now_unix_ms < self.quote.received_at_unix_ms {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_NOW_BEFORE_QUOTE",
                "quality assessment time must not be before quote receipt",
            ));
        }
        if self.max_spread_bps == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_MAX_SPREAD_ZERO",
                "quality assessment requires a positive spread ceiling",
            ));
        }
        if self.min_depth_levels == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_MIN_DEPTH_ZERO",
                "quality assessment requires a positive minimum depth-level count",
            ));
        }
        if self.max_capture_latency_ms == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_LATENCY_LIMIT_ZERO",
                "quality assessment requires a positive capture latency ceiling",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataQualityAssessmentReport {
    /// Validate local market-data quality assessment report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data quality assessment",
            &self.assessment_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);
        if let Err(MarketDataError::ValidationFailed {
            violations: request_violations,
        }) = self.request.validate()
        {
            violations.extend(request_violations);
        }
        if !self.observed_spread_bps.is_finite() || self.observed_spread_bps < 0.0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_SPREAD_INVALID",
                "quality assessment spread must be finite and non-negative",
            ));
        }
        if self.quality_score > 100 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_SCORE_OUT_OF_RANGE",
                "quality assessment score must be within 0..=100",
            ));
        }
        if !self.order_book_present && self.depth_levels_available != 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_DEPTH_WITHOUT_BOOK",
                "quality assessment depth cannot be positive without an order book",
            ));
        }
        let freshness_is_fresh = self.freshness_status.is_fresh();
        if freshness_is_fresh && self.quote_age_ms > self.request.max_age_ms {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_FRESHNESS_AGE_MISMATCH",
                "fresh quality assessments must not exceed the request age ceiling",
            ));
        }
        let should_block = !freshness_is_fresh
            || self.live_network_used
            || self.credential_loaded
            || self.production_ready;
        let should_degrade = !should_block
            && (!self.spread_within_limit
                || !self.depth_levels_sufficient
                || !self.capture_latency_within_limit);
        let expected_status = if should_block {
            MarketDataQualityAssessmentStatus::Blocked
        } else if should_degrade {
            MarketDataQualityAssessmentStatus::Degraded
        } else {
            MarketDataQualityAssessmentStatus::Acceptable
        };
        if self.status != expected_status {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_STATUS_MISMATCH",
                "quality assessment status must match freshness, thresholds, and side-effect flags",
            ));
        }
        if self.production_ready {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_QUALITY_PRODUCTION_READY_FORBIDDEN",
                "quality assessment must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataProviderLatencyReviewRequest {
    /// Validate local provider latency/backpressure review input shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data provider latency review",
            &self.review_id,
            &mut violations,
        );
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.clean_preflight.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.degraded_preflight.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.ready_reconnect.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.acceptable_quality.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.paid_provider_evaluation.validate()
        {
            violations.extend(report_violations);
        }
        if self.max_provider_latency_ms == 0
            || self.max_capture_latency_ms == 0
            || self.max_reconnect_delay_ms == 0
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_BUDGET_ZERO",
                "market-data provider latency review budgets must be positive",
            ));
        }
        if self.min_samples_checked == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_SAMPLE_FLOOR_ZERO",
                "market-data provider latency review requires a positive sample floor",
            ));
        }
        if self.min_quality_score > 100 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_SCORE_INVALID",
                "market-data provider latency review quality floor must be within 0..=100",
            ));
        }
        if self.remaining_external_evidence.is_empty()
            || self
                .remaining_external_evidence
                .iter()
                .any(|item| item.trim().is_empty())
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_EXTERNAL_EVIDENCE_MISSING",
                "market-data provider latency review must keep unresolved external evidence explicit",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataProviderLatencyReviewReport {
    /// Validate local provider latency/backpressure review report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data provider latency review",
            &self.review_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);
        if self.version != MARKET_DATA_PROVIDER_LATENCY_REVIEW_VERSION {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_VERSION_MISMATCH",
                "market-data provider latency review version is not recognized",
            ));
        }
        let side_effected =
            self.live_network_used || self.websocket_connection_opened || self.credential_loaded;
        let should_block = !self.clean_preflight_ready
            || !self.degraded_preflight_failed_closed
            || !self.reconnect_review_ready
            || !self.quality_review_ready
            || !self.paid_provider_review_ready
            || !self.provider_latency_budget_met
            || !self.capture_latency_budget_met
            || !self.reconnect_delay_budget_met
            || !self.sample_floor_met
            || !self.remaining_external_evidence_recorded
            || side_effected
            || self.production_ready;
        if should_block && self.status != MarketDataProviderLatencyReviewStatus::Blocked {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_STATUS_SHOULD_BLOCK",
                "incomplete or unsafe market-data provider latency review evidence must block",
            ));
        }
        if !should_block
            && self.status != MarketDataProviderLatencyReviewStatus::ReadyForLocalReview
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_STATUS_SHOULD_BE_READY",
                "coherent local market-data provider latency evidence must be ready for local review",
            ));
        }
        if side_effected
            && self.status == MarketDataProviderLatencyReviewStatus::ReadyForLocalReview
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_SIDE_EFFECT_FORBIDDEN",
                "market-data provider latency review must not use live network, WebSocket, or credentials",
            ));
        }
        if self.production_ready {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PRODUCTION_READY_FORBIDDEN",
                "market-data provider latency review must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataProviderReconciliationReviewRequest {
    /// Validate local provider reconciliation review input shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data provider reconciliation review",
            &self.review_id,
            &mut violations,
        );
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.latency_review.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.degraded_preflight.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.rate_limit_reconnect.validate()
        {
            violations.extend(report_violations);
        }
        if let Err(MarketDataError::ValidationFailed {
            violations: report_violations,
        }) = self.outage_reconnect.validate()
        {
            violations.extend(report_violations);
        }
        if self.min_degraded_samples_checked == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_SAMPLE_FLOOR_ZERO",
                "market-data provider reconciliation review requires a positive degraded sample floor",
            ));
        }
        if self.remaining_external_evidence.is_empty()
            || self
                .remaining_external_evidence
                .iter()
                .any(|item| item.trim().is_empty())
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_EXTERNAL_EVIDENCE_MISSING",
                "market-data provider reconciliation review must keep unresolved external evidence explicit",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl MarketDataProviderReconciliationReviewReport {
    /// Validate local provider reconciliation review report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "market-data provider reconciliation review",
            &self.review_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);
        if self.version != MARKET_DATA_PROVIDER_RECONCILIATION_REVIEW_VERSION {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_VERSION_MISMATCH",
                "market-data provider reconciliation review version is not recognized",
            ));
        }
        let side_effected =
            self.live_network_used || self.websocket_connection_opened || self.credential_loaded;
        let should_block = !self.latency_review_ready
            || !self.rate_limit_fail_closed
            || !self.outage_fail_closed
            || !self.stale_data_fail_closed
            || !self.latency_fail_closed
            || !self.degraded_sample_floor_met
            || !self.rate_limit_reconnect_ready
            || !self.outage_reconnect_blocked
            || !self.remaining_external_evidence_recorded
            || side_effected
            || self.production_ready;
        if should_block && self.status != MarketDataProviderReconciliationReviewStatus::Blocked {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_STATUS_SHOULD_BLOCK",
                "incomplete or unsafe market-data provider reconciliation evidence must block",
            ));
        }
        if !should_block
            && self.status != MarketDataProviderReconciliationReviewStatus::ReadyForLocalReview
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_STATUS_SHOULD_BE_READY",
                "coherent local market-data provider reconciliation evidence must be ready for local review",
            ));
        }
        if side_effected
            && self.status == MarketDataProviderReconciliationReviewStatus::ReadyForLocalReview
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_SIDE_EFFECT_FORBIDDEN",
                "market-data provider reconciliation review must not use live network, WebSocket, or credentials",
            ));
        }
        if self.production_ready {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_PROVIDER_RECONCILIATION_PRODUCTION_READY_FORBIDDEN",
                "market-data provider reconciliation review must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl HistoricalMarketDataPersistenceInput {
    /// Validate local historical market-data persistence input shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "historical market-data batch",
            &self.batch_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);
        validate_venue(&self.venue, &mut violations);
        if let Err(MarketDataError::ValidationFailed {
            violations: pair_violations,
        }) = self.pair.validate()
        {
            violations.extend(pair_violations);
        }
        if self.quotes.is_empty() && self.order_books.is_empty() {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_EMPTY",
                "historical market-data persistence requires at least one quote or order book",
            ));
        }
        if self.max_retained_records_per_kind == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_RETENTION_ZERO",
                "historical market-data persistence requires a positive retention ceiling",
            ));
        }
        for quote in &self.quotes {
            if let Err(MarketDataError::ValidationFailed {
                violations: quote_violations,
            }) = quote.validate()
            {
                violations.extend(quote_violations);
            }
            if quote.venue != self.venue || quote.pair != self.pair {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_HISTORY_QUOTE_SCOPE_MISMATCH",
                    "historical market-data quotes must match the batch venue and pair",
                ));
            }
            if quote.received_at_unix_ms > self.persisted_at_unix_ms {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_HISTORY_QUOTE_AFTER_PERSISTED_AT",
                    "historical market-data quote cannot be received after the batch persistence time",
                ));
            }
        }
        for order_book in &self.order_books {
            if let Err(MarketDataError::ValidationFailed {
                violations: order_book_violations,
            }) = order_book.validate()
            {
                violations.extend(order_book_violations);
            }
            if order_book.venue != self.venue || order_book.pair != self.pair {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_HISTORY_ORDER_BOOK_SCOPE_MISMATCH",
                    "historical market-data order books must match the batch venue and pair",
                ));
            }
            if order_book.received_at_unix_ms > self.persisted_at_unix_ms {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_HISTORY_ORDER_BOOK_AFTER_PERSISTED_AT",
                    "historical market-data order book cannot be received after the batch persistence time",
                ));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl HistoricalMarketDataPersistenceReport {
    /// Validate local historical market-data persistence report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "historical market-data batch",
            &self.batch_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);
        validate_venue(&self.venue, &mut violations);
        if let Err(MarketDataError::ValidationFailed {
            violations: pair_violations,
        }) = self.pair.validate()
        {
            violations.extend(pair_violations);
        }
        if self.stored_quotes.is_empty() && self.stored_order_books.is_empty() {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_REPORT_EMPTY",
                "historical market-data persistence report requires stored records",
            ));
        }
        if self.max_retained_records_per_kind == 0 {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_REPORT_RETENTION_ZERO",
                "historical market-data persistence report requires a positive retention ceiling",
            ));
        }
        let retention_limit =
            usize::try_from(self.max_retained_records_per_kind).unwrap_or(usize::MAX);
        if self.stored_quotes.len() > retention_limit
            || self.stored_order_books.len() > retention_limit
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_REPORT_RETENTION_EXCEEDED",
                "historical market-data persistence report exceeds its retention ceiling",
            ));
        }
        for quote in &self.stored_quotes {
            if let Err(MarketDataError::ValidationFailed {
                violations: quote_violations,
            }) = quote.validate()
            {
                violations.extend(quote_violations);
            }
            if quote.venue != self.venue || quote.pair != self.pair {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_HISTORY_REPORT_QUOTE_SCOPE_MISMATCH",
                    "stored historical quote must match the batch venue and pair",
                ));
            }
        }
        for order_book in &self.stored_order_books {
            if let Err(MarketDataError::ValidationFailed {
                violations: order_book_violations,
            }) = order_book.validate()
            {
                violations.extend(order_book_violations);
            }
            if order_book.venue != self.venue || order_book.pair != self.pair {
                violations.push(MarketDataViolation::new(
                    "MARKET_DATA_HISTORY_REPORT_ORDER_BOOK_SCOPE_MISMATCH",
                    "stored historical order book must match the batch venue and pair",
                ));
            }
        }
        let expected_oldest = self
            .stored_quotes
            .iter()
            .map(|quote| quote.received_at_unix_ms)
            .chain(
                self.stored_order_books
                    .iter()
                    .map(|order_book| order_book.received_at_unix_ms),
            )
            .min();
        let expected_newest = self
            .stored_quotes
            .iter()
            .map(|quote| quote.received_at_unix_ms)
            .chain(
                self.stored_order_books
                    .iter()
                    .map(|order_book| order_book.received_at_unix_ms),
            )
            .max();
        if self.oldest_received_at_unix_ms != expected_oldest
            || self.newest_received_at_unix_ms != expected_newest
        {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_REPORT_WINDOW_MISMATCH",
                "historical market-data persistence report window bounds must match stored records",
            ));
        }
        let expected_span = match (expected_oldest, expected_newest) {
            (Some(oldest), Some(newest)) => newest.saturating_sub(oldest),
            _ => 0,
        };
        if self.window_span_ms != expected_span {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_REPORT_WINDOW_SPAN_MISMATCH",
                "historical market-data persistence report window span must match stored records",
            ));
        }
        let should_block =
            self.live_network_used || self.credential_loaded || self.production_ready;
        let expected_status = if should_block {
            HistoricalMarketDataPersistenceStatus::Blocked
        } else {
            HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay
        };
        if self.status != expected_status {
            violations.push(MarketDataViolation::new(
                "MARKET_DATA_HISTORY_REPORT_STATUS_MISMATCH",
                "historical market-data persistence status must match side-effect flags",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl PaidMarketDataProviderEvaluationInput {
    /// Validate local paid provider evaluation input shape.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "paid market-data provider evaluation",
            &self.evaluation_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);

        if self.covered_venues.is_empty() {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_VENUES_EMPTY",
                "paid market-data provider evaluation requires at least one covered venue",
            ));
        }
        for venue in &self.covered_venues {
            validate_venue(venue, &mut violations);
        }

        if self.covered_pairs.is_empty() {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_PAIRS_EMPTY",
                "paid market-data provider evaluation requires at least one covered market pair",
            ));
        }
        for pair in &self.covered_pairs {
            if let Err(MarketDataError::ValidationFailed {
                violations: pair_violations,
            }) = pair.validate()
            {
                violations.extend(pair_violations);
            }
        }

        if !self.capabilities.order_book && !self.capabilities.top_of_book {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_CAPABILITIES_INSUFFICIENT",
                "paid market-data provider evaluation requires top-of-book or order-book coverage",
            ));
        }
        if self.documented_latency_ms == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_LATENCY_ZERO",
                "paid market-data provider evaluation requires positive documented latency",
            ));
        }
        if self.max_allowed_latency_ms == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_LATENCY_LIMIT_ZERO",
                "paid market-data provider evaluation requires a positive latency ceiling",
            ));
        }
        if self.max_requests_per_minute == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_RATE_LIMIT_ZERO",
                "paid market-data provider evaluation requires a positive request budget",
            ));
        }
        if self.monthly_cost_usd == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_COST_ZERO",
                "paid market-data provider evaluation requires a non-zero monthly cost estimate",
            ));
        }
        if self.failure_modes_reviewed.is_empty() {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_EVALUATION_FAILURE_MODES_EMPTY",
                "paid market-data provider evaluation requires at least one reviewed failure mode",
            ));
        }
        for failure_mode in &self.failure_modes_reviewed {
            validate_id("market-data failure mode", failure_mode, &mut violations);
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

impl PaidMarketDataProviderEvaluationReport {
    /// Validate local paid provider evaluation report invariants.
    pub fn validate(&self) -> Result<(), MarketDataError> {
        let mut violations = Vec::new();
        validate_id(
            "paid market-data provider evaluation",
            &self.evaluation_id,
            &mut violations,
        );
        validate_id("market-data provider", &self.provider_name, &mut violations);

        if self.covered_venues.is_empty() {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_VENUES_EMPTY",
                "paid market-data provider evaluation report requires covered venues",
            ));
        }
        for venue in &self.covered_venues {
            validate_venue(venue, &mut violations);
        }

        if self.covered_pairs.is_empty() {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_PAIRS_EMPTY",
                "paid market-data provider evaluation report requires covered market pairs",
            ));
        }
        for pair in &self.covered_pairs {
            if let Err(MarketDataError::ValidationFailed {
                violations: pair_violations,
            }) = pair.validate()
            {
                violations.extend(pair_violations);
            }
        }

        if self.documented_latency_ms == 0 || self.max_allowed_latency_ms == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_LATENCY_INVALID",
                "paid market-data provider evaluation report requires positive latency metadata",
            ));
        }
        if self.max_requests_per_minute == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_RATE_LIMIT_ZERO",
                "paid market-data provider evaluation report requires a positive request budget",
            ));
        }
        if self.monthly_cost_usd == 0 {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_COST_ZERO",
                "paid market-data provider evaluation report requires a non-zero monthly cost estimate",
            ));
        }
        if self.failure_modes_reviewed.is_empty() {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_FAILURE_MODES_EMPTY",
                "paid market-data provider evaluation report requires reviewed failure modes",
            ));
        }
        for failure_mode in &self.failure_modes_reviewed {
            validate_id("market-data failure mode", failure_mode, &mut violations);
        }

        let expected_coverage_review_passed = !self.covered_venues.is_empty()
            && !self.covered_pairs.is_empty()
            && (self.capabilities.order_book || self.capabilities.top_of_book);
        if self.coverage_review_passed != expected_coverage_review_passed {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_COVERAGE_MISMATCH",
                "paid market-data provider evaluation coverage flag must match reviewed venues, pairs, and capabilities",
            ));
        }
        let expected_latency_within_budget = self.documented_latency_ms > 0
            && self.documented_latency_ms <= self.max_allowed_latency_ms;
        if self.latency_within_budget != expected_latency_within_budget {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_LATENCY_MISMATCH",
                "paid market-data provider evaluation latency flag must match local latency metadata",
            ));
        }
        let expected_rate_limit_review_passed =
            self.rate_limit_documentation_reviewed && self.max_requests_per_minute > 0;
        if self.rate_limit_review_passed != expected_rate_limit_review_passed {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_RATE_LIMIT_MISMATCH",
                "paid market-data provider evaluation rate-limit flag must match documentation review state",
            ));
        }
        let expected_cost_review_passed =
            self.pricing_documentation_reviewed && self.monthly_cost_usd > 0;
        if self.cost_review_passed != expected_cost_review_passed {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_COST_MISMATCH",
                "paid market-data provider evaluation cost flag must match pricing review state",
            ));
        }
        let expected_failure_behavior_review_passed = !self.failure_modes_reviewed.is_empty();
        if self.failure_behavior_review_passed != expected_failure_behavior_review_passed {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_FAILURE_BEHAVIOR_MISMATCH",
                "paid market-data provider evaluation failure-behavior flag must match reviewed failure modes",
            ));
        }
        let expected_governance_review_passed =
            self.terms_reviewed && self.credential_scope_reviewed;
        if self.governance_review_passed != expected_governance_review_passed {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_GOVERNANCE_MISMATCH",
                "paid market-data provider evaluation governance flag must match terms and credential-scope review state",
            ));
        }

        let should_block = !self.coverage_review_passed
            || !self.latency_within_budget
            || !self.rate_limit_review_passed
            || !self.cost_review_passed
            || !self.failure_behavior_review_passed
            || !self.governance_review_passed
            || self.live_network_used
            || self.credential_loaded
            || self.production_ready;
        if should_block && self.status != PaidMarketDataProviderEvaluationStatus::Blocked {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_STATUS_SHOULD_BLOCK",
                "blocked paid market-data provider evaluations must produce blocked status",
            ));
        }
        if !should_block
            && self.status != PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
        {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_STATUS_SHOULD_BE_READY",
                "clean paid market-data provider evaluations must produce ready-for-local-review status",
            ));
        }
        if self.production_ready {
            violations.push(MarketDataViolation::new(
                "PAID_MARKET_DATA_REPORT_PRODUCTION_READY_FORBIDDEN",
                "paid market-data provider evaluation must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(MarketDataError::ValidationFailed { violations })
        }
    }
}

/// Evaluate local market-data provider health observations without network use.
pub fn validate_market_data_provider_preflight(
    observation: MarketDataProviderHealthObservation,
) -> Result<MarketDataProviderPreflightReport, MarketDataError> {
    observation.validate()?;

    let stale_data_blocked = observation.stale_samples > 0;
    let latency_blocked = observation.max_observed_latency_ms > observation.max_allowed_latency_ms;
    let reconnect_unplanned =
        observation.reconnect_required && !observation.reconnect_backoff_planned;
    let blocked = !observation.read_only
        || observation.rate_limited
        || observation.outage_observed
        || reconnect_unplanned
        || stale_data_blocked
        || latency_blocked
        || observation.live_network_used
        || observation.credential_loaded;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !observation.read_only,
        "MARKET_DATA_PREFLIGHT_NOT_READ_ONLY",
    );
    push_if(
        &mut violation_codes,
        observation.rate_limited,
        "MARKET_DATA_PREFLIGHT_RATE_LIMITED",
    );
    push_if(
        &mut violation_codes,
        observation.outage_observed,
        "MARKET_DATA_PREFLIGHT_OUTAGE",
    );
    push_if(
        &mut violation_codes,
        reconnect_unplanned,
        "MARKET_DATA_PREFLIGHT_RECONNECT_UNPLANNED",
    );
    push_if(
        &mut violation_codes,
        stale_data_blocked,
        "MARKET_DATA_PREFLIGHT_STALE_DATA",
    );
    push_if(
        &mut violation_codes,
        latency_blocked,
        "MARKET_DATA_PREFLIGHT_LATENCY_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        observation.live_network_used,
        "MARKET_DATA_PREFLIGHT_LIVE_NETWORK",
    );
    push_if(
        &mut violation_codes,
        observation.credential_loaded,
        "MARKET_DATA_PREFLIGHT_CREDENTIAL_LOADED",
    );

    let report = MarketDataProviderPreflightReport {
        provider_name: observation.provider_name,
        status: if blocked {
            MarketDataProviderPreflightStatus::Blocked
        } else {
            MarketDataProviderPreflightStatus::Usable
        },
        read_only_confirmed: observation.read_only,
        rate_limit_blocked: observation.rate_limited,
        outage_blocked: observation.outage_observed,
        reconnect_required: observation.reconnect_required,
        reconnect_backoff_planned: observation.reconnect_backoff_planned,
        samples_checked: observation.samples_checked,
        fresh_samples: observation.fresh_samples,
        stale_samples: observation.stale_samples,
        stale_data_blocked,
        max_observed_latency_ms: observation.max_observed_latency_ms,
        max_allowed_latency_ms: observation.max_allowed_latency_ms,
        latency_blocked,
        live_network_used: observation.live_network_used,
        credential_loaded: observation.credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a local market-data reconnect/backoff plan without network use.
pub fn validate_market_data_reconnect_plan(
    input: MarketDataReconnectPlanInput,
) -> Result<MarketDataReconnectPlanReport, MarketDataError> {
    input.validate()?;

    let expected_backoff_ms = exponential_backoff_ms(
        input.base_backoff_ms,
        input.max_backoff_ms,
        input.attempt_number,
    );
    let retry_after_ms = input.provider_retry_after_ms.unwrap_or(0);
    let effective_min_delay_ms = expected_backoff_ms.max(retry_after_ms);
    let retry_budget_exhausted = input.attempt_number > input.max_attempts;
    let delay_too_short = input.planned_delay_ms < effective_min_delay_ms;
    let side_effected =
        input.live_network_used || input.websocket_connection_opened || input.credential_loaded;
    let blocked =
        input.outage_observed || retry_budget_exhausted || delay_too_short || side_effected;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        input.rate_limited && input.provider_retry_after_ms.is_some(),
        "MARKET_DATA_RECONNECT_RATE_LIMIT_RETRY_AFTER_APPLIED",
    );
    push_if(
        &mut violation_codes,
        input.outage_observed,
        "MARKET_DATA_RECONNECT_OUTAGE_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        retry_budget_exhausted,
        "MARKET_DATA_RECONNECT_RETRY_BUDGET_EXHAUSTED",
    );
    push_if(
        &mut violation_codes,
        delay_too_short,
        "MARKET_DATA_RECONNECT_DELAY_TOO_SHORT",
    );
    push_if(
        &mut violation_codes,
        input.live_network_used,
        "MARKET_DATA_RECONNECT_LIVE_NETWORK",
    );
    push_if(
        &mut violation_codes,
        input.websocket_connection_opened,
        "MARKET_DATA_RECONNECT_WEBSOCKET_OPENED",
    );
    push_if(
        &mut violation_codes,
        input.credential_loaded,
        "MARKET_DATA_RECONNECT_CREDENTIAL_LOADED",
    );

    let report = MarketDataReconnectPlanReport {
        plan_id: input.plan_id,
        provider_name: input.provider_name,
        venue: input.venue,
        status: if blocked {
            MarketDataReconnectPlanStatus::Blocked
        } else {
            MarketDataReconnectPlanStatus::ReadyForLocalReview
        },
        attempt_number: input.attempt_number,
        max_attempts: input.max_attempts,
        expected_backoff_ms,
        provider_retry_after_ms: input.provider_retry_after_ms,
        effective_min_delay_ms,
        planned_delay_ms: input.planned_delay_ms,
        next_attempt_at_unix_ms: input
            .planned_at_unix_ms
            .saturating_add(input.planned_delay_ms),
        rate_limit_blocked: delay_too_short && input.rate_limited,
        outage_blocked: input.outage_observed,
        retry_budget_exhausted,
        live_network_used: input.live_network_used,
        websocket_connection_opened: input.websocket_connection_opened,
        credential_loaded: input.credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Assess local normalized market-data quality without network use.
pub fn assess_market_data_quality(
    input: MarketDataQualityAssessmentInput,
) -> Result<MarketDataQualityAssessmentReport, MarketDataError> {
    input.validate()?;

    let freshness_status = input
        .quote
        .freshness(input.now_unix_ms, input.request.max_age_ms);
    let quote_age_ms = input
        .now_unix_ms
        .saturating_sub(input.quote.received_at_unix_ms);
    let observed_spread_bps = input.quote.spread_bps();
    let spread_within_limit =
        observed_spread_bps.is_finite() && observed_spread_bps <= input.max_spread_bps as f64;
    let (order_book_present, depth_levels_available) =
        input
            .order_book
            .as_ref()
            .map_or((false, 0_u32), |order_book| {
                let available = order_book.bids.len().min(order_book.asks.len());
                let available = u32::try_from(available).unwrap_or(u32::MAX);
                (true, available)
            });
    let depth_levels_sufficient = depth_levels_available >= input.min_depth_levels;
    let capture_latency_ms = input
        .quote
        .received_at_unix_ms
        .saturating_sub(input.quote.captured_at_unix_ms);
    let capture_latency_within_limit = capture_latency_ms <= input.max_capture_latency_ms;

    let mut quality_score = 0_u8;
    if freshness_status.is_fresh() {
        quality_score = quality_score.saturating_add(40);
    }
    if spread_within_limit {
        quality_score = quality_score.saturating_add(25);
    } else if observed_spread_bps.is_finite()
        && observed_spread_bps <= (input.max_spread_bps as f64 * 2.0)
    {
        quality_score = quality_score.saturating_add(10);
    }
    if depth_levels_sufficient {
        quality_score = quality_score.saturating_add(20);
    } else if depth_levels_available > 0 {
        quality_score = quality_score.saturating_add(5);
    }
    if capture_latency_within_limit {
        quality_score = quality_score.saturating_add(15);
    } else if capture_latency_ms <= input.max_capture_latency_ms.saturating_mul(2) {
        quality_score = quality_score.saturating_add(5);
    }

    let blocked = !freshness_status.is_fresh()
        || input.live_network_used
        || input.credential_loaded
        || input.production_ready_claimed;
    let degraded = !blocked
        && (!spread_within_limit || !depth_levels_sufficient || !capture_latency_within_limit);

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !freshness_status.is_fresh(),
        "MARKET_DATA_QUALITY_NOT_FRESH",
    );
    push_if(
        &mut violation_codes,
        !spread_within_limit,
        "MARKET_DATA_QUALITY_SPREAD_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        !depth_levels_sufficient,
        "MARKET_DATA_QUALITY_DEPTH_INSUFFICIENT",
    );
    push_if(
        &mut violation_codes,
        !capture_latency_within_limit,
        "MARKET_DATA_QUALITY_CAPTURE_LATENCY_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        input.live_network_used,
        "MARKET_DATA_QUALITY_LIVE_NETWORK_USED",
    );
    push_if(
        &mut violation_codes,
        input.credential_loaded,
        "MARKET_DATA_QUALITY_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        input.production_ready_claimed,
        "MARKET_DATA_QUALITY_PRODUCTION_READY_CLAIMED",
    );

    let report = MarketDataQualityAssessmentReport {
        assessment_id: input.assessment_id,
        provider_name: input.provider_name,
        request: input.request,
        status: if blocked {
            MarketDataQualityAssessmentStatus::Blocked
        } else if degraded {
            MarketDataQualityAssessmentStatus::Degraded
        } else {
            MarketDataQualityAssessmentStatus::Acceptable
        },
        freshness_status,
        quote_age_ms,
        observed_spread_bps,
        spread_within_limit,
        order_book_present,
        depth_levels_available,
        depth_levels_sufficient,
        capture_latency_ms,
        capture_latency_within_limit,
        quality_score,
        live_network_used: input.live_network_used,
        credential_loaded: input.credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Review local market-data provider latency/backpressure evidence without side effects.
pub fn review_market_data_provider_latency(
    request: MarketDataProviderLatencyReviewRequest,
) -> Result<MarketDataProviderLatencyReviewReport, MarketDataError> {
    request.validate()?;

    let clean_preflight_ready = request.clean_preflight.status
        == MarketDataProviderPreflightStatus::Usable
        && request.clean_preflight.read_only_confirmed
        && !request.clean_preflight.rate_limit_blocked
        && !request.clean_preflight.outage_blocked
        && !request.clean_preflight.stale_data_blocked
        && !request.clean_preflight.latency_blocked
        && !request.clean_preflight.live_network_used
        && !request.clean_preflight.credential_loaded
        && !request.clean_preflight.production_ready;
    let degraded_preflight_failed_closed = request.degraded_preflight.status
        == MarketDataProviderPreflightStatus::Blocked
        && !request.degraded_preflight.violation_codes.is_empty()
        && !request.degraded_preflight.production_ready;
    let provider_latency_budget_met = request.clean_preflight.max_observed_latency_ms
        <= request.max_provider_latency_ms
        && request.paid_provider_evaluation.documented_latency_ms
            <= request.max_provider_latency_ms;
    let sample_floor_met = request.clean_preflight.samples_checked >= request.min_samples_checked;
    let reconnect_delay_budget_met = request.ready_reconnect.planned_delay_ms
        <= request.max_reconnect_delay_ms
        && request.ready_reconnect.effective_min_delay_ms <= request.max_reconnect_delay_ms;
    let reconnect_review_ready = request.ready_reconnect.status
        == MarketDataReconnectPlanStatus::ReadyForLocalReview
        && reconnect_delay_budget_met
        && !request.ready_reconnect.live_network_used
        && !request.ready_reconnect.websocket_connection_opened
        && !request.ready_reconnect.credential_loaded
        && !request.ready_reconnect.production_ready;
    let capture_latency_budget_met =
        request.acceptable_quality.capture_latency_ms <= request.max_capture_latency_ms;
    let quality_review_ready = request.acceptable_quality.status
        == MarketDataQualityAssessmentStatus::Acceptable
        && request.acceptable_quality.quality_score >= request.min_quality_score
        && request.acceptable_quality.capture_latency_within_limit
        && capture_latency_budget_met
        && !request.acceptable_quality.live_network_used
        && !request.acceptable_quality.credential_loaded
        && !request.acceptable_quality.production_ready;
    let paid_provider_review_ready = request.paid_provider_evaluation.status
        == PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
        && request.paid_provider_evaluation.latency_within_budget
        && request.paid_provider_evaluation.rate_limit_review_passed
        && request
            .paid_provider_evaluation
            .failure_behavior_review_passed
        && request.paid_provider_evaluation.governance_review_passed
        && !request.paid_provider_evaluation.live_network_used
        && !request.paid_provider_evaluation.credential_loaded
        && !request.paid_provider_evaluation.production_ready;
    let remaining_external_evidence_recorded = !request.remaining_external_evidence.is_empty();
    let live_network_used = request.live_network_used
        || request.clean_preflight.live_network_used
        || request.degraded_preflight.live_network_used
        || request.ready_reconnect.live_network_used
        || request.acceptable_quality.live_network_used
        || request.paid_provider_evaluation.live_network_used;
    let websocket_connection_opened =
        request.websocket_connection_opened || request.ready_reconnect.websocket_connection_opened;
    let credential_loaded = request.credential_loaded
        || request.clean_preflight.credential_loaded
        || request.degraded_preflight.credential_loaded
        || request.ready_reconnect.credential_loaded
        || request.acceptable_quality.credential_loaded
        || request.paid_provider_evaluation.credential_loaded;
    let blocked = !clean_preflight_ready
        || !degraded_preflight_failed_closed
        || !reconnect_review_ready
        || !quality_review_ready
        || !paid_provider_review_ready
        || !provider_latency_budget_met
        || !capture_latency_budget_met
        || !reconnect_delay_budget_met
        || !sample_floor_met
        || !remaining_external_evidence_recorded
        || live_network_used
        || websocket_connection_opened
        || credential_loaded
        || request.production_ready_claimed;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !clean_preflight_ready,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PREFLIGHT_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !degraded_preflight_failed_closed,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_DEGRADED_NOT_FAIL_CLOSED",
    );
    push_if(
        &mut violation_codes,
        !reconnect_review_ready,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_RECONNECT_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !quality_review_ready,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_QUALITY_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !paid_provider_review_ready,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PAID_PROVIDER_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !provider_latency_budget_met,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PROVIDER_LATENCY_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        !capture_latency_budget_met,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_CAPTURE_LATENCY_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        !reconnect_delay_budget_met,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_RECONNECT_DELAY_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        !sample_floor_met,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_SAMPLE_FLOOR_MISSING",
    );
    push_if(
        &mut violation_codes,
        live_network_used,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_LIVE_NETWORK_USED",
    );
    push_if(
        &mut violation_codes,
        websocket_connection_opened,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_WEBSOCKET_OPENED",
    );
    push_if(
        &mut violation_codes,
        credential_loaded,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        request.production_ready_claimed,
        "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PRODUCTION_READY_CLAIMED",
    );

    let report = MarketDataProviderLatencyReviewReport {
        version: MARKET_DATA_PROVIDER_LATENCY_REVIEW_VERSION.to_owned(),
        review_id: request.review_id,
        status: if blocked {
            MarketDataProviderLatencyReviewStatus::Blocked
        } else {
            MarketDataProviderLatencyReviewStatus::ReadyForLocalReview
        },
        provider_name: request.clean_preflight.provider_name,
        clean_preflight_ready,
        degraded_preflight_failed_closed,
        reconnect_review_ready,
        quality_review_ready,
        paid_provider_review_ready,
        provider_latency_budget_met,
        capture_latency_budget_met,
        reconnect_delay_budget_met,
        sample_floor_met,
        remaining_external_evidence_recorded,
        remaining_external_evidence_count: request.remaining_external_evidence.len(),
        live_network_used,
        websocket_connection_opened,
        credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Review local market-data provider rate-limit/outage reconciliation without side effects.
pub fn review_market_data_provider_reconciliation(
    request: MarketDataProviderReconciliationReviewRequest,
) -> Result<MarketDataProviderReconciliationReviewReport, MarketDataError> {
    request.validate()?;

    let latency_review_ready = request.latency_review.status
        == MarketDataProviderLatencyReviewStatus::ReadyForLocalReview
        && !request.latency_review.live_network_used
        && !request.latency_review.websocket_connection_opened
        && !request.latency_review.credential_loaded
        && !request.latency_review.production_ready;
    let rate_limit_fail_closed = request.degraded_preflight.status
        == MarketDataProviderPreflightStatus::Blocked
        && request.degraded_preflight.rate_limit_blocked
        && request
            .degraded_preflight
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_RATE_LIMITED");
    let outage_fail_closed = request.degraded_preflight.status
        == MarketDataProviderPreflightStatus::Blocked
        && request.degraded_preflight.outage_blocked
        && request
            .degraded_preflight
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_OUTAGE");
    let stale_data_fail_closed = request.degraded_preflight.stale_data_blocked;
    let latency_fail_closed = request.degraded_preflight.latency_blocked;
    let degraded_sample_floor_met =
        request.degraded_preflight.samples_checked >= request.min_degraded_samples_checked;
    let rate_limit_reconnect_ready = request.rate_limit_reconnect.status
        == MarketDataReconnectPlanStatus::ReadyForLocalReview
        && request
            .rate_limit_reconnect
            .provider_retry_after_ms
            .is_some()
        && request
            .rate_limit_reconnect
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_RATE_LIMIT_RETRY_AFTER_APPLIED")
        && !request.rate_limit_reconnect.outage_blocked
        && !request.rate_limit_reconnect.retry_budget_exhausted
        && !request.rate_limit_reconnect.live_network_used
        && !request.rate_limit_reconnect.websocket_connection_opened
        && !request.rate_limit_reconnect.credential_loaded
        && !request.rate_limit_reconnect.production_ready;
    let outage_reconnect_blocked = request.outage_reconnect.status
        == MarketDataReconnectPlanStatus::Blocked
        && request.outage_reconnect.outage_blocked
        && request.outage_reconnect.retry_budget_exhausted
        && !request.outage_reconnect.live_network_used
        && !request.outage_reconnect.websocket_connection_opened
        && !request.outage_reconnect.credential_loaded
        && !request.outage_reconnect.production_ready;
    let remaining_external_evidence_recorded = !request.remaining_external_evidence.is_empty();
    let live_network_used = request.live_network_used
        || request.latency_review.live_network_used
        || request.degraded_preflight.live_network_used
        || request.rate_limit_reconnect.live_network_used
        || request.outage_reconnect.live_network_used;
    let websocket_connection_opened = request.websocket_connection_opened
        || request.latency_review.websocket_connection_opened
        || request.rate_limit_reconnect.websocket_connection_opened
        || request.outage_reconnect.websocket_connection_opened;
    let credential_loaded = request.credential_loaded
        || request.latency_review.credential_loaded
        || request.degraded_preflight.credential_loaded
        || request.rate_limit_reconnect.credential_loaded
        || request.outage_reconnect.credential_loaded;
    let blocked = !latency_review_ready
        || !rate_limit_fail_closed
        || !outage_fail_closed
        || !stale_data_fail_closed
        || !latency_fail_closed
        || !degraded_sample_floor_met
        || !rate_limit_reconnect_ready
        || !outage_reconnect_blocked
        || !remaining_external_evidence_recorded
        || live_network_used
        || websocket_connection_opened
        || credential_loaded
        || request.production_ready_claimed;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !latency_review_ready,
        "MARKET_DATA_PROVIDER_RECONCILIATION_LATENCY_REVIEW_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !rate_limit_fail_closed,
        "MARKET_DATA_PROVIDER_RECONCILIATION_RATE_LIMIT_NOT_FAIL_CLOSED",
    );
    push_if(
        &mut violation_codes,
        !outage_fail_closed,
        "MARKET_DATA_PROVIDER_RECONCILIATION_OUTAGE_NOT_FAIL_CLOSED",
    );
    push_if(
        &mut violation_codes,
        !stale_data_fail_closed,
        "MARKET_DATA_PROVIDER_RECONCILIATION_STALE_DATA_NOT_FAIL_CLOSED",
    );
    push_if(
        &mut violation_codes,
        !latency_fail_closed,
        "MARKET_DATA_PROVIDER_RECONCILIATION_LATENCY_NOT_FAIL_CLOSED",
    );
    push_if(
        &mut violation_codes,
        !degraded_sample_floor_met,
        "MARKET_DATA_PROVIDER_RECONCILIATION_SAMPLE_FLOOR_MISSING",
    );
    push_if(
        &mut violation_codes,
        !rate_limit_reconnect_ready,
        "MARKET_DATA_PROVIDER_RECONCILIATION_RATE_LIMIT_RECONNECT_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !outage_reconnect_blocked,
        "MARKET_DATA_PROVIDER_RECONCILIATION_OUTAGE_RECONNECT_NOT_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        !remaining_external_evidence_recorded,
        "MARKET_DATA_PROVIDER_RECONCILIATION_EXTERNAL_EVIDENCE_MISSING",
    );
    push_if(
        &mut violation_codes,
        live_network_used,
        "MARKET_DATA_PROVIDER_RECONCILIATION_LIVE_NETWORK_USED",
    );
    push_if(
        &mut violation_codes,
        websocket_connection_opened,
        "MARKET_DATA_PROVIDER_RECONCILIATION_WEBSOCKET_OPENED",
    );
    push_if(
        &mut violation_codes,
        credential_loaded,
        "MARKET_DATA_PROVIDER_RECONCILIATION_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        request.production_ready_claimed,
        "MARKET_DATA_PROVIDER_RECONCILIATION_PRODUCTION_READY_CLAIMED",
    );

    let report = MarketDataProviderReconciliationReviewReport {
        version: MARKET_DATA_PROVIDER_RECONCILIATION_REVIEW_VERSION.to_owned(),
        review_id: request.review_id,
        status: if blocked {
            MarketDataProviderReconciliationReviewStatus::Blocked
        } else {
            MarketDataProviderReconciliationReviewStatus::ReadyForLocalReview
        },
        provider_name: request.latency_review.provider_name,
        latency_review_ready,
        rate_limit_fail_closed,
        outage_fail_closed,
        stale_data_fail_closed,
        latency_fail_closed,
        degraded_sample_floor_met,
        rate_limit_reconnect_ready,
        outage_reconnect_blocked,
        remaining_external_evidence_recorded,
        remaining_external_evidence_count: request.remaining_external_evidence.len(),
        live_network_used,
        websocket_connection_opened,
        credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Validate and prepare a local historical market-data persistence batch.
pub fn validate_historical_market_data_persistence(
    input: HistoricalMarketDataPersistenceInput,
) -> Result<HistoricalMarketDataPersistenceReport, MarketDataError> {
    input.validate()?;

    let mut stored_quotes = input.quotes;
    stored_quotes.sort_by(|left, right| {
        left.received_at_unix_ms
            .cmp(&right.received_at_unix_ms)
            .then_with(|| left.captured_at_unix_ms.cmp(&right.captured_at_unix_ms))
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut stored_order_books = input.order_books;
    stored_order_books.sort_by(|left, right| {
        left.received_at_unix_ms
            .cmp(&right.received_at_unix_ms)
            .then_with(|| left.captured_at_unix_ms.cmp(&right.captured_at_unix_ms))
            .then_with(|| left.id.cmp(&right.id))
    });

    let retention_limit =
        usize::try_from(input.max_retained_records_per_kind).unwrap_or(usize::MAX);
    let quotes_truncated = stored_quotes.len() > retention_limit;
    if quotes_truncated {
        let retained_start = stored_quotes.len().saturating_sub(retention_limit);
        stored_quotes = stored_quotes.split_off(retained_start);
    }
    let order_books_truncated = stored_order_books.len() > retention_limit;
    if order_books_truncated {
        let retained_start = stored_order_books.len().saturating_sub(retention_limit);
        stored_order_books = stored_order_books.split_off(retained_start);
    }

    let oldest_received_at_unix_ms = stored_quotes
        .iter()
        .map(|quote| quote.received_at_unix_ms)
        .chain(
            stored_order_books
                .iter()
                .map(|order_book| order_book.received_at_unix_ms),
        )
        .min();
    let newest_received_at_unix_ms = stored_quotes
        .iter()
        .map(|quote| quote.received_at_unix_ms)
        .chain(
            stored_order_books
                .iter()
                .map(|order_book| order_book.received_at_unix_ms),
        )
        .max();
    let window_span_ms = match (oldest_received_at_unix_ms, newest_received_at_unix_ms) {
        (Some(oldest), Some(newest)) => newest.saturating_sub(oldest),
        _ => 0,
    };

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        input.live_network_used,
        "MARKET_DATA_HISTORY_LIVE_NETWORK_USED",
    );
    push_if(
        &mut violation_codes,
        input.credential_loaded,
        "MARKET_DATA_HISTORY_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        input.production_ready_claimed,
        "MARKET_DATA_HISTORY_PRODUCTION_READY_CLAIMED",
    );

    let report = HistoricalMarketDataPersistenceReport {
        batch_id: input.batch_id,
        provider_name: input.provider_name,
        venue: input.venue,
        pair: input.pair,
        status: if input.live_network_used
            || input.credential_loaded
            || input.production_ready_claimed
        {
            HistoricalMarketDataPersistenceStatus::Blocked
        } else {
            HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay
        },
        stored_quotes,
        stored_order_books,
        quotes_truncated,
        order_books_truncated,
        oldest_received_at_unix_ms,
        newest_received_at_unix_ms,
        window_span_ms,
        max_retained_records_per_kind: input.max_retained_records_per_kind,
        live_network_used: input.live_network_used,
        credential_loaded: input.credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a local paid market-data provider comparison dossier without network use.
pub fn validate_paid_market_data_provider_evaluation(
    input: PaidMarketDataProviderEvaluationInput,
) -> Result<PaidMarketDataProviderEvaluationReport, MarketDataError> {
    input.validate()?;

    let coverage_review_passed = !input.covered_venues.is_empty()
        && !input.covered_pairs.is_empty()
        && (input.capabilities.order_book || input.capabilities.top_of_book);
    let latency_within_budget = input.documented_latency_ms <= input.max_allowed_latency_ms;
    let rate_limit_review_passed =
        input.rate_limit_documentation_reviewed && input.max_requests_per_minute > 0;
    let cost_review_passed = input.pricing_documentation_reviewed && input.monthly_cost_usd > 0;
    let failure_behavior_review_passed = !input.failure_modes_reviewed.is_empty();
    let governance_review_passed = input.terms_reviewed && input.credential_scope_reviewed;
    let blocked = !coverage_review_passed
        || !latency_within_budget
        || !rate_limit_review_passed
        || !cost_review_passed
        || !failure_behavior_review_passed
        || !governance_review_passed
        || input.live_network_used
        || input.credential_loaded
        || input.production_ready_claimed;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !coverage_review_passed,
        "PAID_MARKET_DATA_EVALUATION_COVERAGE_INCOMPLETE",
    );
    push_if(
        &mut violation_codes,
        !latency_within_budget,
        "PAID_MARKET_DATA_EVALUATION_LATENCY_EXCEEDED",
    );
    push_if(
        &mut violation_codes,
        !rate_limit_review_passed,
        "PAID_MARKET_DATA_EVALUATION_RATE_LIMIT_REVIEW_MISSING",
    );
    push_if(
        &mut violation_codes,
        !cost_review_passed,
        "PAID_MARKET_DATA_EVALUATION_COST_REVIEW_MISSING",
    );
    push_if(
        &mut violation_codes,
        !failure_behavior_review_passed,
        "PAID_MARKET_DATA_EVALUATION_FAILURE_BEHAVIOR_REVIEW_MISSING",
    );
    push_if(
        &mut violation_codes,
        !input.terms_reviewed,
        "PAID_MARKET_DATA_EVALUATION_TERMS_REVIEW_MISSING",
    );
    push_if(
        &mut violation_codes,
        !input.credential_scope_reviewed,
        "PAID_MARKET_DATA_EVALUATION_CREDENTIAL_SCOPE_REVIEW_MISSING",
    );
    push_if(
        &mut violation_codes,
        input.live_network_used,
        "PAID_MARKET_DATA_EVALUATION_LIVE_NETWORK_USED",
    );
    push_if(
        &mut violation_codes,
        input.credential_loaded,
        "PAID_MARKET_DATA_EVALUATION_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        input.production_ready_claimed,
        "PAID_MARKET_DATA_EVALUATION_PRODUCTION_READY_CLAIMED",
    );

    let report = PaidMarketDataProviderEvaluationReport {
        evaluation_id: input.evaluation_id,
        provider_name: input.provider_name,
        status: if blocked {
            PaidMarketDataProviderEvaluationStatus::Blocked
        } else {
            PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
        },
        covered_venues: input.covered_venues,
        covered_pairs: input.covered_pairs,
        capabilities: input.capabilities,
        documented_latency_ms: input.documented_latency_ms,
        max_allowed_latency_ms: input.max_allowed_latency_ms,
        latency_within_budget,
        max_requests_per_minute: input.max_requests_per_minute,
        rate_limit_review_passed,
        monthly_cost_usd: input.monthly_cost_usd,
        cost_review_passed,
        failure_modes_reviewed: input.failure_modes_reviewed,
        failure_behavior_review_passed,
        coverage_review_passed,
        governance_review_passed,
        rate_limit_documentation_reviewed: input.rate_limit_documentation_reviewed,
        pricing_documentation_reviewed: input.pricing_documentation_reviewed,
        terms_reviewed: input.terms_reviewed,
        credential_scope_reviewed: input.credential_scope_reviewed,
        live_network_used: input.live_network_used,
        credential_loaded: input.credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Persist the latest local market-data provider preflight through state.
pub fn persist_market_data_provider_preflight_checkpoint(
    store: &mut impl StateStore,
    report: &MarketDataProviderPreflightReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, MarketDataError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY.to_owned(),
        subsystem: MARKET_DATA_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            MarketDataError::InvariantViolation {
                reason: format!("failed to serialize market-data preflight checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(MarketDataError::from)?;
    Ok(checkpoint)
}

/// Append one local market-data provider preflight report to the audit journal.
pub fn append_market_data_provider_preflight_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &MarketDataProviderPreflightReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, MarketDataError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("market-data-provider-preflight-{}", report.provider_name),
        AuditEventKind::RuntimeLifecycle,
        MARKET_DATA_STATE_SUBSYSTEM,
        "market-data-provider",
        "Market-data provider preflight validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "market_data_model_version",
            AuditValue::Text(MARKET_DATA_MODEL_VERSION.to_owned()),
        )
        .with_metadata(
            "provider_name",
            AuditValue::Text(report.provider_name.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "samples_checked",
            AuditValue::Unsigned(report.samples_checked),
        )
        .with_metadata("fresh_samples", AuditValue::Unsigned(report.fresh_samples))
        .with_metadata("stale_samples", AuditValue::Unsigned(report.stale_samples))
        .with_metadata(
            "rate_limit_blocked",
            AuditValue::Bool(report.rate_limit_blocked),
        )
        .with_metadata("outage_blocked", AuditValue::Bool(report.outage_blocked))
        .with_metadata(
            "stale_data_blocked",
            AuditValue::Bool(report.stale_data_blocked),
        )
        .with_metadata("latency_blocked", AuditValue::Bool(report.latency_blocked))
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "credential_loaded",
            AuditValue::Bool(report.credential_loaded),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| MarketDataError::InvariantViolation {
            reason: format!("failed to append market-data preflight audit record: {error}"),
        })
}

/// Persist the latest local market-data reconnect plan through state.
pub fn persist_market_data_reconnect_plan_checkpoint(
    store: &mut impl StateStore,
    report: &MarketDataReconnectPlanReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, MarketDataError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY.to_owned(),
        subsystem: MARKET_DATA_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            MarketDataError::InvariantViolation {
                reason: format!("failed to serialize market-data reconnect checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(MarketDataError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local paid market-data provider evaluation through state.
pub fn persist_paid_market_data_provider_evaluation_checkpoint(
    store: &mut impl StateStore,
    report: &PaidMarketDataProviderEvaluationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, MarketDataError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: MARKET_DATA_LAST_PAID_PROVIDER_EVALUATION_CHECKPOINT_KEY.to_owned(),
        subsystem: MARKET_DATA_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            MarketDataError::InvariantViolation {
                reason: format!(
                    "failed to serialize paid market-data provider evaluation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(MarketDataError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local market-data quality assessment through state.
pub fn persist_market_data_quality_assessment_checkpoint(
    store: &mut impl StateStore,
    report: &MarketDataQualityAssessmentReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, MarketDataError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY.to_owned(),
        subsystem: MARKET_DATA_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            MarketDataError::InvariantViolation {
                reason: format!(
                    "failed to serialize market-data quality assessment checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(MarketDataError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local historical market-data batch through state.
pub fn persist_historical_market_data_checkpoint(
    store: &mut impl StateStore,
    report: &HistoricalMarketDataPersistenceReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, MarketDataError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY.to_owned(),
        subsystem: MARKET_DATA_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            MarketDataError::InvariantViolation {
                reason: format!(
                    "failed to serialize historical market-data persistence checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(MarketDataError::from)?;
    Ok(checkpoint)
}

/// Append one local market-data reconnect plan report to the audit journal.
pub fn append_market_data_reconnect_plan_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &MarketDataReconnectPlanReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, MarketDataError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("market-data-reconnect-plan-{}", report.plan_id),
        AuditEventKind::RuntimeLifecycle,
        MARKET_DATA_STATE_SUBSYSTEM,
        "market-data-provider",
        "Market-data reconnect/backoff plan validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "market_data_model_version",
            AuditValue::Text(MARKET_DATA_MODEL_VERSION.to_owned()),
        )
        .with_metadata("plan_id", AuditValue::Text(report.plan_id.clone()))
        .with_metadata(
            "provider_name",
            AuditValue::Text(report.provider_name.clone()),
        )
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "attempt_number",
            AuditValue::Unsigned(u64::from(report.attempt_number)),
        )
        .with_metadata(
            "max_attempts",
            AuditValue::Unsigned(u64::from(report.max_attempts)),
        )
        .with_metadata(
            "effective_min_delay_ms",
            AuditValue::Unsigned(report.effective_min_delay_ms),
        )
        .with_metadata(
            "planned_delay_ms",
            AuditValue::Unsigned(report.planned_delay_ms),
        )
        .with_metadata("outage_blocked", AuditValue::Bool(report.outage_blocked))
        .with_metadata(
            "retry_budget_exhausted",
            AuditValue::Bool(report.retry_budget_exhausted),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "websocket_connection_opened",
            AuditValue::Bool(report.websocket_connection_opened),
        )
        .with_metadata(
            "credential_loaded",
            AuditValue::Bool(report.credential_loaded),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| MarketDataError::InvariantViolation {
            reason: format!("failed to append market-data reconnect audit record: {error}"),
        })
}

/// Append one local paid market-data provider evaluation report to the audit journal.
pub fn append_paid_market_data_provider_evaluation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &PaidMarketDataProviderEvaluationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, MarketDataError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "paid-market-data-provider-evaluation-{}",
            report.provider_name
        ),
        AuditEventKind::RuntimeLifecycle,
        MARKET_DATA_STATE_SUBSYSTEM,
        "market-data-provider",
        "Paid market-data provider evaluation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "market_data_model_version",
            AuditValue::Text(MARKET_DATA_MODEL_VERSION.to_owned()),
        )
        .with_metadata(
            "evaluation_id",
            AuditValue::Text(report.evaluation_id.clone()),
        )
        .with_metadata(
            "provider_name",
            AuditValue::Text(report.provider_name.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "covered_venue_count",
            AuditValue::Unsigned(report.covered_venues.len() as u64),
        )
        .with_metadata(
            "covered_pair_count",
            AuditValue::Unsigned(report.covered_pairs.len() as u64),
        )
        .with_metadata(
            "latency_within_budget",
            AuditValue::Bool(report.latency_within_budget),
        )
        .with_metadata(
            "rate_limit_review_passed",
            AuditValue::Bool(report.rate_limit_review_passed),
        )
        .with_metadata(
            "cost_review_passed",
            AuditValue::Bool(report.cost_review_passed),
        )
        .with_metadata(
            "failure_behavior_review_passed",
            AuditValue::Bool(report.failure_behavior_review_passed),
        )
        .with_metadata(
            "governance_review_passed",
            AuditValue::Bool(report.governance_review_passed),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "credential_loaded",
            AuditValue::Bool(report.credential_loaded),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| MarketDataError::InvariantViolation {
            reason: format!(
                "failed to append paid market-data provider evaluation audit record: {error}"
            ),
        })
}

/// Append one local market-data quality assessment report to the audit journal.
pub fn append_market_data_quality_assessment_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &MarketDataQualityAssessmentReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, MarketDataError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("market-data-quality-assessment-{}", report.assessment_id),
        AuditEventKind::RuntimeLifecycle,
        MARKET_DATA_STATE_SUBSYSTEM,
        "market-data-provider",
        "Market-data quality assessment recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "market_data_model_version",
            AuditValue::Text(MARKET_DATA_MODEL_VERSION.to_owned()),
        )
        .with_metadata(
            "assessment_id",
            AuditValue::Text(report.assessment_id.clone()),
        )
        .with_metadata(
            "provider_name",
            AuditValue::Text(report.provider_name.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "quality_score",
            AuditValue::Unsigned(u64::from(report.quality_score)),
        )
        .with_metadata("quote_age_ms", AuditValue::Unsigned(report.quote_age_ms))
        .with_metadata(
            "spread_within_limit",
            AuditValue::Bool(report.spread_within_limit),
        )
        .with_metadata(
            "depth_levels_available",
            AuditValue::Unsigned(u64::from(report.depth_levels_available)),
        )
        .with_metadata(
            "depth_levels_sufficient",
            AuditValue::Bool(report.depth_levels_sufficient),
        )
        .with_metadata(
            "capture_latency_within_limit",
            AuditValue::Bool(report.capture_latency_within_limit),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "credential_loaded",
            AuditValue::Bool(report.credential_loaded),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| MarketDataError::InvariantViolation {
            reason: format!(
                "failed to append market-data quality assessment audit record: {error}"
            ),
        })
}

/// Append one local historical market-data persistence report to the audit journal.
pub fn append_historical_market_data_persistence_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &HistoricalMarketDataPersistenceReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, MarketDataError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("historical-market-data-persistence-{}", report.batch_id),
        AuditEventKind::RuntimeLifecycle,
        MARKET_DATA_STATE_SUBSYSTEM,
        "market-data-provider",
        "Historical market-data persistence recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "market_data_model_version",
            AuditValue::Text(MARKET_DATA_MODEL_VERSION.to_owned()),
        )
        .with_metadata("batch_id", AuditValue::Text(report.batch_id.clone()))
        .with_metadata(
            "provider_name",
            AuditValue::Text(report.provider_name.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "stored_quote_count",
            AuditValue::Unsigned(report.stored_quotes.len() as u64),
        )
        .with_metadata(
            "stored_order_book_count",
            AuditValue::Unsigned(report.stored_order_books.len() as u64),
        )
        .with_metadata(
            "quotes_truncated",
            AuditValue::Bool(report.quotes_truncated),
        )
        .with_metadata(
            "order_books_truncated",
            AuditValue::Bool(report.order_books_truncated),
        )
        .with_metadata(
            "window_span_ms",
            AuditValue::Unsigned(report.window_span_ms),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "credential_loaded",
            AuditValue::Bool(report.credential_loaded),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| MarketDataError::InvariantViolation {
            reason: format!(
                "failed to append historical market-data persistence audit record: {error}"
            ),
        })
}

/// Boundary trait for future read-only market-data connectors.
///
/// Implementations must not place orders, sign transactions, load trading keys,
/// or mutate balances. Live network implementations are intentionally deferred.
pub trait MarketDataProvider {
    /// Stable provider name for diagnostics and audit records.
    fn provider_name(&self) -> &str;

    /// Connector capability declaration.
    fn capabilities(&self) -> MarketDataCapabilities;

    /// Fetch or return a normalized order book for the request.
    fn order_book(&self, request: &MarketDataRequest)
        -> Result<OrderBookSnapshot, MarketDataError>;

    /// Fetch or return a normalized quote for the request.
    fn top_of_book(&self, request: &MarketDataRequest) -> Result<NormalizedQuote, MarketDataError>;
}

/// One market-data validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketDataViolation {
    code: &'static str,
    message: String,
}

impl MarketDataViolation {
    /// Create a validation violation.
    #[must_use]
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_owned(),
        }
    }

    /// Create a validation violation with owned message text.
    #[must_use]
    pub fn new_owned(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// Stable violation code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Human-readable violation message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Market-data boundary errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketDataError {
    /// Validation failed with one or more deterministic violations.
    ValidationFailed {
        violations: Vec<MarketDataViolation>,
    },
    /// Provider capability is unavailable.
    CapabilityUnavailable {
        provider: String,
        capability: &'static str,
    },
    /// Provider returned no data for the request.
    NoData { provider: String, reason: String },
    /// Internal invariant failure in scaffold logic.
    InvariantViolation { reason: String },
}

impl MarketDataError {
    /// Return violations, if this is a validation error.
    #[must_use]
    pub fn violations(&self) -> &[MarketDataViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::CapabilityUnavailable { .. }
            | Self::NoData { .. }
            | Self::InvariantViolation { .. } => &[],
        }
    }
}

impl fmt::Display for MarketDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "market-data validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::CapabilityUnavailable {
                provider,
                capability,
            } => {
                write!(
                    formatter,
                    "market-data provider {provider} does not support {capability}"
                )
            }
            Self::NoData { provider, reason } => {
                write!(
                    formatter,
                    "market-data provider {provider} returned no data: {reason}"
                )
            }
            Self::InvariantViolation { reason } => {
                write!(formatter, "market-data invariant violation: {reason}")
            }
        }
    }
}

impl Error for MarketDataError {}

impl From<StateStoreError> for MarketDataError {
    fn from(error: StateStoreError) -> Self {
        Self::InvariantViolation {
            reason: error.to_string(),
        }
    }
}

fn classify_freshness(
    received_at_unix_ms: u64,
    now_unix_ms: u64,
    max_age_ms: u64,
) -> FreshnessStatus {
    if received_at_unix_ms > now_unix_ms {
        return FreshnessStatus::FutureTimestamp {
            future_by_ms: received_at_unix_ms - now_unix_ms,
        };
    }

    let age_ms = now_unix_ms - received_at_unix_ms;
    if age_ms <= max_age_ms {
        FreshnessStatus::Fresh { age_ms }
    } else {
        FreshnessStatus::Stale { age_ms, max_age_ms }
    }
}

fn collect_levels(
    side_label: &'static str,
    levels: &[PriceLevel],
    violations: &mut Vec<MarketDataViolation>,
) {
    for level in levels {
        level.collect_violations(side_label, violations);
    }
}

fn normalize_symbol(symbol: String) -> String {
    symbol.trim().to_ascii_uppercase()
}

fn exponential_backoff_ms(base_backoff_ms: u64, max_backoff_ms: u64, attempt_number: u32) -> u64 {
    let multiplier_shifts = attempt_number.saturating_sub(1).min(63);
    let multiplier = 1_u64.checked_shl(multiplier_shifts).unwrap_or(u64::MAX);
    base_backoff_ms
        .saturating_mul(multiplier)
        .min(max_backoff_ms)
}

fn validate_symbol(label: &'static str, symbol: &str, violations: &mut Vec<MarketDataViolation>) {
    if symbol.trim().is_empty() {
        violations.push(MarketDataViolation::new_owned(
            "ASSET_SYMBOL_EMPTY",
            format!("{label} asset symbol must be non-empty"),
        ));
        return;
    }

    if !symbol
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        violations.push(MarketDataViolation::new_owned(
            "ASSET_SYMBOL_INVALID_CHARACTERS",
            format!("{label} asset symbol contains unsupported characters"),
        ));
    }
}

fn validate_id(label: &'static str, id: &str, violations: &mut Vec<MarketDataViolation>) {
    if id.trim().is_empty() {
        violations.push(MarketDataViolation::new_owned(
            "MARKET_DATA_ID_EMPTY",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn validate_venue(venue: &VenueRef, violations: &mut Vec<MarketDataViolation>) {
    if venue.name.trim().is_empty() {
        violations.push(MarketDataViolation::new(
            "VENUE_NAME_EMPTY",
            "market-data venue name must be non-empty",
        ));
    }

    if matches!(venue.kind, VenueKind::Bridge) {
        violations.push(MarketDataViolation::new(
            "BRIDGE_MARKET_DATA_DEFERRED",
            "bridge-route market data is deferred until elevated Web3 custody phases",
        ));
    }
}

fn validate_timestamps(
    captured_at_unix_ms: u64,
    received_at_unix_ms: u64,
    violations: &mut Vec<MarketDataViolation>,
) {
    if captured_at_unix_ms == 0 {
        violations.push(MarketDataViolation::new(
            "CAPTURED_TIMESTAMP_ZERO",
            "captured_at_unix_ms must be non-zero",
        ));
    }

    if received_at_unix_ms == 0 {
        violations.push(MarketDataViolation::new(
            "RECEIVED_TIMESTAMP_ZERO",
            "received_at_unix_ms must be non-zero",
        ));
    }

    if captured_at_unix_ms > received_at_unix_ms {
        violations.push(MarketDataViolation::new(
            "CAPTURED_AFTER_RECEIVED",
            "captured_at_unix_ms cannot be later than received_at_unix_ms",
        ));
    }
}

fn push_if(codes: &mut Vec<String>, condition: bool, code: &'static str) {
    if condition {
        codes.push(code.to_owned());
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

#[cfg(test)]
mod tests {
    use super::{
        append_historical_market_data_persistence_audit,
        append_market_data_provider_preflight_audit, append_market_data_quality_assessment_audit,
        append_market_data_reconnect_plan_audit, append_paid_market_data_provider_evaluation_audit,
        assess_market_data_quality, persist_historical_market_data_checkpoint,
        persist_market_data_provider_preflight_checkpoint,
        persist_market_data_quality_assessment_checkpoint,
        persist_market_data_reconnect_plan_checkpoint,
        persist_paid_market_data_provider_evaluation_checkpoint,
        review_market_data_provider_latency, review_market_data_provider_reconciliation,
        validate_historical_market_data_persistence, validate_market_data_provider_preflight,
        validate_market_data_reconnect_plan, validate_paid_market_data_provider_evaluation,
        FreshnessStatus, HistoricalMarketDataPersistenceInput,
        HistoricalMarketDataPersistenceReport, HistoricalMarketDataPersistenceStatus,
        MarketDataCapabilities, MarketDataProviderHealthObservation,
        MarketDataProviderLatencyReviewRequest, MarketDataProviderLatencyReviewStatus,
        MarketDataProviderPreflightReport, MarketDataProviderPreflightStatus,
        MarketDataProviderReconciliationReviewRequest,
        MarketDataProviderReconciliationReviewStatus, MarketDataQualityAssessmentInput,
        MarketDataQualityAssessmentReport, MarketDataQualityAssessmentStatus,
        MarketDataReconnectPlanInput, MarketDataReconnectPlanReport, MarketDataReconnectPlanStatus,
        MarketDataRequest, MarketPair, NormalizedQuote, OrderBookSnapshot,
        PaidMarketDataProviderEvaluationInput, PaidMarketDataProviderEvaluationReport,
        PaidMarketDataProviderEvaluationStatus, PriceLevel,
        MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY,
        MARKET_DATA_LAST_PAID_PROVIDER_EVALUATION_CHECKPOINT_KEY,
        MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY,
        MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY,
        MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY, MARKET_DATA_STATE_SUBSYSTEM,
    };
    use crate::{AppendOnlyAuditJournal, InMemoryStateStore, StateStore, VenueKind, VenueRef};

    #[test]
    fn market_pair_normalizes_symbols() {
        let pair = MarketPair::new(" btc ", "usdc").expect("pair should validate");
        assert_eq!(pair.symbol(), "BTC/USDC");
    }

    #[test]
    fn order_book_converts_to_quote() {
        let book = OrderBookSnapshot {
            id: "book-1".to_owned(),
            venue: VenueRef {
                kind: VenueKind::Cex,
                name: "paper-binance".to_owned(),
            },
            pair: MarketPair::new("ETH", "USDT").expect("pair should validate"),
            captured_at_unix_ms: 1_000,
            received_at_unix_ms: 1_010,
            bids: vec![PriceLevel::new(100.0, 2.0).expect("bid should validate")],
            asks: vec![PriceLevel::new(101.0, 2.0).expect("ask should validate")],
            source_sequence: Some("seq-1".to_owned()),
        };

        let quote = book.to_quote().expect("quote should validate");
        assert!((quote.mid_price_quote() - 100.5).abs() < f64::EPSILON);
    }

    #[test]
    fn quote_reports_stale_freshness() {
        let quote = NormalizedQuote {
            id: "quote-1".to_owned(),
            venue: VenueRef {
                kind: VenueKind::Cex,
                name: "paper-kraken".to_owned(),
            },
            pair: MarketPair::new("BTC", "USD").expect("pair should validate"),
            bid: PriceLevel::new(99.0, 1.0).expect("bid should validate"),
            ask: PriceLevel::new(100.0, 1.0).expect("ask should validate"),
            captured_at_unix_ms: 1_000,
            received_at_unix_ms: 1_000,
        };

        assert_eq!(
            quote.freshness(2_001, 1_000),
            FreshnessStatus::Stale {
                age_ms: 1_001,
                max_age_ms: 1_000,
            }
        );
    }

    #[test]
    fn market_data_provider_preflight_accepts_clean_local_read_only_observations() {
        let report = validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
            provider_name: "local-fixture-provider".to_owned(),
            read_only: true,
            rate_limited: false,
            outage_observed: false,
            reconnect_required: true,
            reconnect_backoff_planned: true,
            samples_checked: 4,
            fresh_samples: 4,
            stale_samples: 0,
            max_observed_latency_ms: 12,
            max_allowed_latency_ms: 50,
            live_network_used: false,
            credential_loaded: false,
        })
        .expect("clean local read-only preflight should pass");

        assert_eq!(report.status, MarketDataProviderPreflightStatus::Usable);
        assert!(report.read_only_confirmed);
        assert!(report.reconnect_required);
        assert!(report.reconnect_backoff_planned);
        assert!(report.violation_codes.is_empty());
        assert!(!report.live_network_used);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_preflight_blocks_rate_limit_outage_stale_and_latency() {
        let report = validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
            provider_name: "local-degraded-provider".to_owned(),
            read_only: true,
            rate_limited: true,
            outage_observed: true,
            reconnect_required: true,
            reconnect_backoff_planned: false,
            samples_checked: 5,
            fresh_samples: 3,
            stale_samples: 2,
            max_observed_latency_ms: 250,
            max_allowed_latency_ms: 100,
            live_network_used: false,
            credential_loaded: false,
        })
        .expect("degraded local preflight should produce blocked report");

        assert_eq!(report.status, MarketDataProviderPreflightStatus::Blocked);
        assert!(report.rate_limit_blocked);
        assert!(report.outage_blocked);
        assert!(report.stale_data_blocked);
        assert!(report.latency_blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_RATE_LIMITED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_OUTAGE"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_STALE_DATA"));
        assert!(!report.live_network_used);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_preflight_rejects_side_effect_observations() {
        let report = validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
            provider_name: "local-side-effect-provider".to_owned(),
            read_only: true,
            rate_limited: false,
            outage_observed: false,
            reconnect_required: false,
            reconnect_backoff_planned: false,
            samples_checked: 1,
            fresh_samples: 1,
            stale_samples: 0,
            max_observed_latency_ms: 1,
            max_allowed_latency_ms: 10,
            live_network_used: true,
            credential_loaded: true,
        })
        .expect("side-effect observations should still produce a fail-closed report");

        assert_eq!(report.status, MarketDataProviderPreflightStatus::Blocked);
        assert!(report.live_network_used);
        assert!(report.credential_loaded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_LIVE_NETWORK"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PREFLIGHT_CREDENTIAL_LOADED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_reconnect_plan_accepts_retry_after_backoff_without_side_effects() {
        let report = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
            plan_id: "reconnect-plan-ready".to_owned(),
            provider_name: "local-websocket-provider".to_owned(),
            venue: local_cex_venue("paper-binance"),
            disconnected_at_unix_ms: 10_000,
            planned_at_unix_ms: 10_050,
            attempt_number: 3,
            max_attempts: 5,
            base_backoff_ms: 100,
            max_backoff_ms: 1_000,
            planned_delay_ms: 500,
            provider_retry_after_ms: Some(450),
            rate_limited: true,
            outage_observed: false,
            live_network_used: false,
            websocket_connection_opened: false,
            credential_loaded: false,
        })
        .expect("coherent reconnect plan should validate");

        assert_eq!(
            report.status,
            MarketDataReconnectPlanStatus::ReadyForLocalReview
        );
        assert_eq!(report.expected_backoff_ms, 400);
        assert_eq!(report.effective_min_delay_ms, 450);
        assert_eq!(report.next_attempt_at_unix_ms, 10_550);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_RATE_LIMIT_RETRY_AFTER_APPLIED"));
        assert!(!report.rate_limit_blocked);
        assert!(!report.outage_blocked);
        assert!(!report.retry_budget_exhausted);
        assert!(!report.live_network_used);
        assert!(!report.websocket_connection_opened);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_reconnect_plan_blocks_short_delay_outage_and_retry_exhaustion() {
        let report = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
            plan_id: "reconnect-plan-blocked".to_owned(),
            provider_name: "local-degraded-provider".to_owned(),
            venue: local_cex_venue("paper-kraken"),
            disconnected_at_unix_ms: 20_000,
            planned_at_unix_ms: 20_010,
            attempt_number: 6,
            max_attempts: 5,
            base_backoff_ms: 100,
            max_backoff_ms: 1_000,
            planned_delay_ms: 200,
            provider_retry_after_ms: Some(800),
            rate_limited: true,
            outage_observed: true,
            live_network_used: false,
            websocket_connection_opened: false,
            credential_loaded: false,
        })
        .expect("blocked reconnect plan should still produce report");

        assert_eq!(report.status, MarketDataReconnectPlanStatus::Blocked);
        assert_eq!(report.expected_backoff_ms, 1_000);
        assert_eq!(report.effective_min_delay_ms, 1_000);
        assert!(report.rate_limit_blocked);
        assert!(report.outage_blocked);
        assert!(report.retry_budget_exhausted);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_OUTAGE_BLOCKED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_RETRY_BUDGET_EXHAUSTED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_DELAY_TOO_SHORT"));
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_reconnect_plan_fails_closed_on_side_effect_flags() {
        let report = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
            plan_id: "reconnect-plan-side-effect".to_owned(),
            provider_name: "local-side-effect-provider".to_owned(),
            venue: local_cex_venue("paper-coinbase"),
            disconnected_at_unix_ms: 30_000,
            planned_at_unix_ms: 30_100,
            attempt_number: 1,
            max_attempts: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 1_000,
            planned_delay_ms: 100,
            provider_retry_after_ms: None,
            rate_limited: false,
            outage_observed: false,
            live_network_used: true,
            websocket_connection_opened: true,
            credential_loaded: true,
        })
        .expect("side-effect reconnect plan should produce blocked report");

        assert_eq!(report.status, MarketDataReconnectPlanStatus::Blocked);
        assert!(report.live_network_used);
        assert!(report.websocket_connection_opened);
        assert!(report.credential_loaded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_LIVE_NETWORK"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_WEBSOCKET_OPENED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_RECONNECT_CREDENTIAL_LOADED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn historical_market_data_persistence_retains_latest_records_locally() {
        let venue = local_cex_venue("paper-history");
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let report =
            validate_historical_market_data_persistence(HistoricalMarketDataPersistenceInput {
                batch_id: "history-batch".to_owned(),
                provider_name: "local-history-provider".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                quotes: vec![
                    NormalizedQuote {
                        id: "quote-1".to_owned(),
                        venue: venue.clone(),
                        pair: pair.clone(),
                        bid: PriceLevel::new(100.0, 1.0).expect("bid should validate"),
                        ask: PriceLevel::new(100.1, 1.0).expect("ask should validate"),
                        captured_at_unix_ms: 1_000,
                        received_at_unix_ms: 1_010,
                    },
                    NormalizedQuote {
                        id: "quote-2".to_owned(),
                        venue: venue.clone(),
                        pair: pair.clone(),
                        bid: PriceLevel::new(101.0, 1.0).expect("bid should validate"),
                        ask: PriceLevel::new(101.1, 1.0).expect("ask should validate"),
                        captured_at_unix_ms: 2_000,
                        received_at_unix_ms: 2_010,
                    },
                    NormalizedQuote {
                        id: "quote-3".to_owned(),
                        venue: venue.clone(),
                        pair: pair.clone(),
                        bid: PriceLevel::new(102.0, 1.0).expect("bid should validate"),
                        ask: PriceLevel::new(102.1, 1.0).expect("ask should validate"),
                        captured_at_unix_ms: 3_000,
                        received_at_unix_ms: 3_010,
                    },
                ],
                order_books: vec![
                    OrderBookSnapshot {
                        id: "book-1".to_owned(),
                        venue: venue.clone(),
                        pair: pair.clone(),
                        captured_at_unix_ms: 1_000,
                        received_at_unix_ms: 1_015,
                        bids: vec![PriceLevel::new(100.0, 1.0).expect("bid should validate")],
                        asks: vec![PriceLevel::new(100.1, 1.0).expect("ask should validate")],
                        source_sequence: Some("book-seq-1".to_owned()),
                    },
                    OrderBookSnapshot {
                        id: "book-2".to_owned(),
                        venue: venue.clone(),
                        pair: pair.clone(),
                        captured_at_unix_ms: 2_000,
                        received_at_unix_ms: 2_015,
                        bids: vec![PriceLevel::new(101.0, 1.0).expect("bid should validate")],
                        asks: vec![PriceLevel::new(101.1, 1.0).expect("ask should validate")],
                        source_sequence: Some("book-seq-2".to_owned()),
                    },
                    OrderBookSnapshot {
                        id: "book-3".to_owned(),
                        venue: venue.clone(),
                        pair: pair.clone(),
                        captured_at_unix_ms: 3_000,
                        received_at_unix_ms: 3_015,
                        bids: vec![PriceLevel::new(102.0, 1.0).expect("bid should validate")],
                        asks: vec![PriceLevel::new(102.1, 1.0).expect("ask should validate")],
                        source_sequence: Some("book-seq-3".to_owned()),
                    },
                ],
                max_retained_records_per_kind: 2,
                persisted_at_unix_ms: 4_000,
                live_network_used: false,
                credential_loaded: false,
                production_ready_claimed: false,
            })
            .expect("historical persistence should validate");

        assert_eq!(
            report.status,
            HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay
        );
        assert_eq!(report.stored_quotes.len(), 2);
        assert_eq!(report.stored_order_books.len(), 2);
        assert!(report.quotes_truncated);
        assert!(report.order_books_truncated);
        assert_eq!(report.stored_quotes[0].id, "quote-2");
        assert_eq!(report.stored_quotes[1].id, "quote-3");
        assert_eq!(report.stored_order_books[0].id, "book-2");
        assert_eq!(report.stored_order_books[1].id, "book-3");
        assert_eq!(report.oldest_received_at_unix_ms, Some(2_010));
        assert_eq!(report.newest_received_at_unix_ms, Some(3_015));
        assert_eq!(report.window_span_ms, 1_005);
        assert!(!report.production_ready);
    }

    #[test]
    fn historical_market_data_persistence_fails_closed_on_side_effect_flags() {
        let venue = local_cex_venue("paper-history-blocked");
        let pair = MarketPair::new("ETH", "USDC").expect("pair should validate");
        let report =
            validate_historical_market_data_persistence(HistoricalMarketDataPersistenceInput {
                batch_id: "history-batch-blocked".to_owned(),
                provider_name: "local-history-provider-blocked".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                quotes: vec![NormalizedQuote {
                    id: "quote-blocked".to_owned(),
                    venue,
                    pair,
                    bid: PriceLevel::new(200.0, 1.0).expect("bid should validate"),
                    ask: PriceLevel::new(200.1, 1.0).expect("ask should validate"),
                    captured_at_unix_ms: 5_000,
                    received_at_unix_ms: 5_010,
                }],
                order_books: Vec::new(),
                max_retained_records_per_kind: 4,
                persisted_at_unix_ms: 6_000,
                live_network_used: true,
                credential_loaded: true,
                production_ready_claimed: true,
            })
            .expect("blocked historical persistence should still produce a report");

        assert_eq!(
            report.status,
            HistoricalMarketDataPersistenceStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_HISTORY_LIVE_NETWORK_USED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_HISTORY_CREDENTIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_HISTORY_PRODUCTION_READY_CLAIMED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_quality_assessment_accepts_clean_local_quote_and_depth() {
        let venue = local_cex_venue("paper-binance");
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let report = assess_market_data_quality(MarketDataQualityAssessmentInput {
            assessment_id: "quality-ready".to_owned(),
            provider_name: "local-market-data-quality".to_owned(),
            request: MarketDataRequest {
                venue: venue.clone(),
                pair: pair.clone(),
                max_age_ms: 250,
            },
            quote: NormalizedQuote {
                id: "quality-quote-ready".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                bid: PriceLevel::new(100.0, 1.0).expect("bid should validate"),
                ask: PriceLevel::new(100.1, 1.0).expect("ask should validate"),
                captured_at_unix_ms: 1_000,
                received_at_unix_ms: 1_015,
            },
            order_book: Some(OrderBookSnapshot {
                id: "quality-book-ready".to_owned(),
                venue: venue.clone(),
                pair,
                captured_at_unix_ms: 1_000,
                received_at_unix_ms: 1_015,
                bids: vec![
                    PriceLevel::new(100.0, 1.0).expect("bid should validate"),
                    PriceLevel::new(99.9, 1.0).expect("bid should validate"),
                ],
                asks: vec![
                    PriceLevel::new(100.1, 1.0).expect("ask should validate"),
                    PriceLevel::new(100.2, 1.0).expect("ask should validate"),
                ],
                source_sequence: Some("seq-quality-ready".to_owned()),
            }),
            now_unix_ms: 1_120,
            max_spread_bps: 20,
            min_depth_levels: 2,
            max_capture_latency_ms: 25,
            live_network_used: false,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .expect("quality assessment should validate");

        assert_eq!(report.status, MarketDataQualityAssessmentStatus::Acceptable);
        assert!(report.freshness_status.is_fresh());
        assert!(report.spread_within_limit);
        assert!(report.depth_levels_sufficient);
        assert!(report.capture_latency_within_limit);
        assert_eq!(report.quality_score, 100);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn market_data_quality_assessment_degrades_on_spread_depth_and_latency() {
        let venue = local_cex_venue("paper-coinbase");
        let pair = MarketPair::new("ETH", "USDC").expect("pair should validate");
        let report = assess_market_data_quality(MarketDataQualityAssessmentInput {
            assessment_id: "quality-degraded".to_owned(),
            provider_name: "local-market-data-quality-degraded".to_owned(),
            request: MarketDataRequest {
                venue: venue.clone(),
                pair: pair.clone(),
                max_age_ms: 500,
            },
            quote: NormalizedQuote {
                id: "quality-quote-degraded".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                bid: PriceLevel::new(200.0, 1.0).expect("bid should validate"),
                ask: PriceLevel::new(201.0, 1.0).expect("ask should validate"),
                captured_at_unix_ms: 10_000,
                received_at_unix_ms: 10_060,
            },
            order_book: Some(OrderBookSnapshot {
                id: "quality-book-degraded".to_owned(),
                venue: venue.clone(),
                pair,
                captured_at_unix_ms: 10_000,
                received_at_unix_ms: 10_060,
                bids: vec![PriceLevel::new(200.0, 1.0).expect("bid should validate")],
                asks: vec![PriceLevel::new(201.0, 1.0).expect("ask should validate")],
                source_sequence: Some("seq-quality-degraded".to_owned()),
            }),
            now_unix_ms: 10_200,
            max_spread_bps: 20,
            min_depth_levels: 2,
            max_capture_latency_ms: 25,
            live_network_used: false,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .expect("degraded quality assessment should validate");

        assert_eq!(report.status, MarketDataQualityAssessmentStatus::Degraded);
        assert!(report.freshness_status.is_fresh());
        assert!(!report.spread_within_limit);
        assert!(!report.depth_levels_sufficient);
        assert!(!report.capture_latency_within_limit);
        assert!(report.quality_score < 100);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_QUALITY_SPREAD_EXCEEDED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_QUALITY_DEPTH_INSUFFICIENT"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_QUALITY_CAPTURE_LATENCY_EXCEEDED"));
    }

    #[test]
    fn market_data_quality_assessment_blocks_stale_or_side_effect_records() {
        let venue = local_cex_venue("paper-kraken");
        let pair = MarketPair::new("SOL", "USDC").expect("pair should validate");
        let report = assess_market_data_quality(MarketDataQualityAssessmentInput {
            assessment_id: "quality-blocked".to_owned(),
            provider_name: "local-market-data-quality-blocked".to_owned(),
            request: MarketDataRequest {
                venue: venue.clone(),
                pair: pair.clone(),
                max_age_ms: 100,
            },
            quote: NormalizedQuote {
                id: "quality-quote-blocked".to_owned(),
                venue,
                pair,
                bid: PriceLevel::new(50.0, 1.0).expect("bid should validate"),
                ask: PriceLevel::new(50.2, 1.0).expect("ask should validate"),
                captured_at_unix_ms: 20_000,
                received_at_unix_ms: 20_010,
            },
            order_book: None,
            now_unix_ms: 20_500,
            max_spread_bps: 50,
            min_depth_levels: 1,
            max_capture_latency_ms: 30,
            live_network_used: true,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .expect("blocked quality assessment should validate");

        assert_eq!(report.status, MarketDataQualityAssessmentStatus::Blocked);
        assert!(!report.freshness_status.is_fresh());
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_QUALITY_NOT_FRESH"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_QUALITY_LIVE_NETWORK_USED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn paid_market_data_provider_evaluation_accepts_complete_local_comparison() {
        let report =
            validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
                evaluation_id: "paid-eval-ready".to_owned(),
                provider_name: "local-paid-provider".to_owned(),
                covered_venues: vec![
                    local_cex_venue("paper-binance"),
                    local_cex_venue("paper-coinbase"),
                ],
                covered_pairs: vec![
                    MarketPair::new("BTC", "USDC").expect("pair should validate"),
                    MarketPair::new("ETH", "USDC").expect("pair should validate"),
                ],
                capabilities: MarketDataCapabilities {
                    order_book: true,
                    top_of_book: true,
                    fees: false,
                    websocket: true,
                    rest: true,
                },
                documented_latency_ms: 35,
                max_allowed_latency_ms: 50,
                max_requests_per_minute: 1_200,
                monthly_cost_usd: 499,
                failure_modes_reviewed: vec![
                    "provider-outage".to_owned(),
                    "stale-book".to_owned(),
                    "rate-limit-burst".to_owned(),
                ],
                rate_limit_documentation_reviewed: true,
                pricing_documentation_reviewed: true,
                terms_reviewed: true,
                credential_scope_reviewed: true,
                live_network_used: false,
                credential_loaded: false,
                production_ready_claimed: false,
            })
            .expect("paid provider evaluation should validate");

        assert_eq!(
            report.status,
            PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
        );
        assert!(report.coverage_review_passed);
        assert!(report.latency_within_budget);
        assert!(report.rate_limit_review_passed);
        assert!(report.cost_review_passed);
        assert!(report.failure_behavior_review_passed);
        assert!(report.governance_review_passed);
        assert!(!report.live_network_used);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn paid_market_data_provider_evaluation_blocks_missing_comparison_metadata() {
        let report =
            validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
                evaluation_id: "paid-eval-blocked".to_owned(),
                provider_name: "local-incomplete-provider".to_owned(),
                covered_venues: vec![local_cex_venue("paper-kraken")],
                covered_pairs: vec![MarketPair::new("BTC", "USDT").expect("pair should validate")],
                capabilities: MarketDataCapabilities {
                    order_book: true,
                    top_of_book: true,
                    fees: false,
                    websocket: false,
                    rest: true,
                },
                documented_latency_ms: 120,
                max_allowed_latency_ms: 50,
                max_requests_per_minute: 600,
                monthly_cost_usd: 250,
                failure_modes_reviewed: vec!["provider-outage".to_owned()],
                rate_limit_documentation_reviewed: false,
                pricing_documentation_reviewed: false,
                terms_reviewed: false,
                credential_scope_reviewed: false,
                live_network_used: false,
                credential_loaded: false,
                production_ready_claimed: false,
            })
            .expect("blocked evaluation should still produce a report");

        assert_eq!(
            report.status,
            PaidMarketDataProviderEvaluationStatus::Blocked
        );
        assert!(!report.latency_within_budget);
        assert!(!report.rate_limit_review_passed);
        assert!(!report.cost_review_passed);
        assert!(!report.governance_review_passed);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_LATENCY_EXCEEDED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_RATE_LIMIT_REVIEW_MISSING"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_COST_REVIEW_MISSING"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_TERMS_REVIEW_MISSING"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_CREDENTIAL_SCOPE_REVIEW_MISSING"));
    }

    #[test]
    fn paid_market_data_provider_evaluation_fails_closed_on_side_effect_flags() {
        let report =
            validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
                evaluation_id: "paid-eval-side-effect".to_owned(),
                provider_name: "local-side-effect-provider".to_owned(),
                covered_venues: vec![local_cex_venue("paper-binance")],
                covered_pairs: vec![MarketPair::new("SOL", "USDC").expect("pair should validate")],
                capabilities: MarketDataCapabilities {
                    order_book: true,
                    top_of_book: false,
                    fees: false,
                    websocket: true,
                    rest: true,
                },
                documented_latency_ms: 40,
                max_allowed_latency_ms: 60,
                max_requests_per_minute: 900,
                monthly_cost_usd: 399,
                failure_modes_reviewed: vec!["rate-limit-burst".to_owned()],
                rate_limit_documentation_reviewed: true,
                pricing_documentation_reviewed: true,
                terms_reviewed: true,
                credential_scope_reviewed: true,
                live_network_used: true,
                credential_loaded: true,
                production_ready_claimed: true,
            })
            .expect("side-effect evaluation should still produce a blocked report");

        assert_eq!(
            report.status,
            PaidMarketDataProviderEvaluationStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_LIVE_NETWORK_USED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_CREDENTIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "PAID_MARKET_DATA_EVALUATION_PRODUCTION_READY_CLAIMED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_latency_review_accepts_local_evidence_with_open_external_gaps() {
        let report = review_market_data_provider_latency(
            market_data_provider_latency_review_request(false, 50, 25, 500),
        )
        .expect("local provider latency review should validate");

        assert_eq!(
            report.status,
            MarketDataProviderLatencyReviewStatus::ReadyForLocalReview
        );
        assert!(report.clean_preflight_ready);
        assert!(report.degraded_preflight_failed_closed);
        assert!(report.reconnect_review_ready);
        assert!(report.quality_review_ready);
        assert!(report.paid_provider_review_ready);
        assert!(report.provider_latency_budget_met);
        assert!(report.capture_latency_budget_met);
        assert!(report.reconnect_delay_budget_met);
        assert!(report.sample_floor_met);
        assert!(report.remaining_external_evidence_recorded);
        assert_eq!(report.remaining_external_evidence_count, 5);
        assert!(report.violation_codes.is_empty());
        assert!(!report.live_network_used);
        assert!(!report.websocket_connection_opened);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_latency_review_blocks_budget_misses() {
        let report = review_market_data_provider_latency(
            market_data_provider_latency_review_request(false, 10, 5, 100),
        )
        .expect("budget misses should still produce fail-closed report");

        assert_eq!(
            report.status,
            MarketDataProviderLatencyReviewStatus::Blocked
        );
        assert!(!report.provider_latency_budget_met);
        assert!(!report.capture_latency_budget_met);
        assert!(!report.reconnect_delay_budget_met);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PROVIDER_LATENCY_EXCEEDED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_CAPTURE_LATENCY_EXCEEDED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_RECONNECT_DELAY_EXCEEDED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_latency_review_fails_closed_on_side_effect_claims() {
        let report = review_market_data_provider_latency(
            market_data_provider_latency_review_request(true, 50, 25, 500),
        )
        .expect("side-effect flags should still produce fail-closed report");

        assert_eq!(
            report.status,
            MarketDataProviderLatencyReviewStatus::Blocked
        );
        assert!(report.live_network_used);
        assert!(report.websocket_connection_opened);
        assert!(report.credential_loaded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_LIVE_NETWORK_USED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_WEBSOCKET_OPENED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_CREDENTIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_LATENCY_REVIEW_PRODUCTION_READY_CLAIMED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_reconciliation_review_accepts_local_fail_closed_evidence() {
        let report = review_market_data_provider_reconciliation(
            market_data_provider_reconciliation_review_request(false, true),
        )
        .expect("local provider reconciliation review should validate");

        assert_eq!(
            report.status,
            MarketDataProviderReconciliationReviewStatus::ReadyForLocalReview
        );
        assert!(report.latency_review_ready);
        assert!(report.rate_limit_fail_closed);
        assert!(report.outage_fail_closed);
        assert!(report.stale_data_fail_closed);
        assert!(report.latency_fail_closed);
        assert!(report.degraded_sample_floor_met);
        assert!(report.rate_limit_reconnect_ready);
        assert!(report.outage_reconnect_blocked);
        assert!(report.remaining_external_evidence_recorded);
        assert_eq!(report.remaining_external_evidence_count, 5);
        assert!(report.violation_codes.is_empty());
        assert!(!report.live_network_used);
        assert!(!report.websocket_connection_opened);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_reconciliation_review_blocks_missing_outage_evidence() {
        let report = review_market_data_provider_reconciliation(
            market_data_provider_reconciliation_review_request(false, false),
        )
        .expect("missing outage evidence should still produce fail-closed report");

        assert_eq!(
            report.status,
            MarketDataProviderReconciliationReviewStatus::Blocked
        );
        assert!(!report.outage_reconnect_blocked);
        assert!(
            report
                .violation_codes
                .iter()
                .any(|code| code
                    == "MARKET_DATA_PROVIDER_RECONCILIATION_OUTAGE_RECONNECT_NOT_BLOCKED")
        );
        assert!(!report.production_ready);
    }

    #[test]
    fn market_data_provider_reconciliation_review_fails_closed_on_side_effect_claims() {
        let report = review_market_data_provider_reconciliation(
            market_data_provider_reconciliation_review_request(true, true),
        )
        .expect("side-effect flags should still produce fail-closed report");

        assert_eq!(
            report.status,
            MarketDataProviderReconciliationReviewStatus::Blocked
        );
        assert!(report.live_network_used);
        assert!(report.websocket_connection_opened);
        assert!(report.credential_loaded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_RECONCILIATION_LIVE_NETWORK_USED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_RECONCILIATION_WEBSOCKET_OPENED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_RECONCILIATION_CREDENTIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "MARKET_DATA_PROVIDER_RECONCILIATION_PRODUCTION_READY_CLAIMED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn paid_market_data_provider_evaluation_audit_and_state_reopen_locally() {
        let report =
            validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
                evaluation_id: "paid-eval-audit-state".to_owned(),
                provider_name: "local-audit-provider".to_owned(),
                covered_venues: vec![
                    local_cex_venue("paper-binance"),
                    local_cex_venue("paper-coinbase"),
                ],
                covered_pairs: vec![MarketPair::new("BTC", "USDC").expect("pair should validate")],
                capabilities: MarketDataCapabilities {
                    order_book: true,
                    top_of_book: true,
                    fees: false,
                    websocket: true,
                    rest: true,
                },
                documented_latency_ms: 25,
                max_allowed_latency_ms: 40,
                max_requests_per_minute: 1_000,
                monthly_cost_usd: 350,
                failure_modes_reviewed: vec!["provider-outage".to_owned(), "stale-book".to_owned()],
                rate_limit_documentation_reviewed: true,
                pricing_documentation_reviewed: true,
                terms_reviewed: true,
                credential_scope_reviewed: true,
                live_network_used: false,
                credential_loaded: false,
                production_ready_claimed: false,
            })
            .expect("paid provider evaluation should validate");

        let audit_path = unique_temp_path("paid-market-data-provider-evaluation-audit", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let record =
            append_paid_market_data_provider_evaluation_audit(&mut journal, &report, 1_700_000_039)
                .expect("audit append should succeed");
        assert_eq!(record.event.subsystem, MARKET_DATA_STATE_SUBSYSTEM);
        assert_eq!(record.event.actor, "market-data-provider");

        let mut invalid = report.clone();
        invalid.production_ready = true;
        assert!(append_paid_market_data_provider_evaluation_audit(
            &mut journal,
            &invalid,
            1_700_000_040,
        )
        .is_err());

        let mut store = InMemoryStateStore::new();
        let checkpoint = persist_paid_market_data_provider_evaluation_checkpoint(
            &mut store,
            &report,
            1_700_000_041,
        )
        .expect("checkpoint persist should succeed");
        assert_eq!(
            checkpoint.key,
            MARKET_DATA_LAST_PAID_PROVIDER_EVALUATION_CHECKPOINT_KEY
        );

        let recovered = store
            .get_checkpoint(MARKET_DATA_LAST_PAID_PROVIDER_EVALUATION_CHECKPOINT_KEY)
            .expect("state read should succeed")
            .expect("checkpoint should exist");
        let recovered_report: PaidMarketDataProviderEvaluationReport =
            serde_json::from_str(&recovered.value).expect("checkpoint JSON should decode");
        assert_eq!(recovered_report, report);
        assert_eq!(
            recovered_report.status,
            PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
        );
    }

    #[test]
    fn historical_market_data_persistence_audit_and_state_reopen_locally() {
        let venue = local_cex_venue("paper-history-audit");
        let pair = MarketPair::new("SOL", "USDC").expect("pair should validate");
        let report =
            validate_historical_market_data_persistence(HistoricalMarketDataPersistenceInput {
                batch_id: "history-audit-state".to_owned(),
                provider_name: "local-history-audit-provider".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                quotes: vec![NormalizedQuote {
                    id: "history-audit-quote".to_owned(),
                    venue: venue.clone(),
                    pair: pair.clone(),
                    bid: PriceLevel::new(50.0, 1.0).expect("bid should validate"),
                    ask: PriceLevel::new(50.1, 1.0).expect("ask should validate"),
                    captured_at_unix_ms: 7_000,
                    received_at_unix_ms: 7_010,
                }],
                order_books: vec![OrderBookSnapshot {
                    id: "history-audit-book".to_owned(),
                    venue,
                    pair,
                    captured_at_unix_ms: 7_000,
                    received_at_unix_ms: 7_015,
                    bids: vec![PriceLevel::new(50.0, 1.0).expect("bid should validate")],
                    asks: vec![PriceLevel::new(50.1, 1.0).expect("ask should validate")],
                    source_sequence: Some("history-audit-seq".to_owned()),
                }],
                max_retained_records_per_kind: 4,
                persisted_at_unix_ms: 8_000,
                live_network_used: false,
                credential_loaded: false,
                production_ready_claimed: false,
            })
            .expect("historical persistence should validate");

        let audit_path = unique_temp_path("historical-market-data-persistence-audit", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let record =
            append_historical_market_data_persistence_audit(&mut journal, &report, 1_700_000_037)
                .expect("audit append should succeed");
        assert_eq!(record.event.subsystem, MARKET_DATA_STATE_SUBSYSTEM);
        assert_eq!(record.event.actor, "market-data-provider");

        let mut invalid = report.clone();
        invalid.production_ready = true;
        assert!(append_historical_market_data_persistence_audit(
            &mut journal,
            &invalid,
            1_700_000_038
        )
        .is_err());

        let mut store = InMemoryStateStore::new();
        let checkpoint =
            persist_historical_market_data_checkpoint(&mut store, &report, 1_700_000_039)
                .expect("checkpoint persist should succeed");
        assert_eq!(
            checkpoint.key,
            MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY
        );

        let recovered = store
            .get_checkpoint(MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY)
            .expect("state read should succeed")
            .expect("checkpoint should exist");
        let recovered_report: HistoricalMarketDataPersistenceReport =
            serde_json::from_str(&recovered.value).expect("checkpoint JSON should decode");
        assert_eq!(recovered_report, report);
        assert_eq!(
            recovered_report.status,
            HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay
        );
    }

    #[test]
    fn market_data_quality_assessment_audit_and_state_reopen_locally() {
        let venue = local_cex_venue("paper-audit-quality");
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let report = assess_market_data_quality(MarketDataQualityAssessmentInput {
            assessment_id: "quality-audit-state".to_owned(),
            provider_name: "local-quality-audit-provider".to_owned(),
            request: MarketDataRequest {
                venue: venue.clone(),
                pair: pair.clone(),
                max_age_ms: 200,
            },
            quote: NormalizedQuote {
                id: "quality-audit-quote".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                bid: PriceLevel::new(100.0, 1.0).expect("bid should validate"),
                ask: PriceLevel::new(100.1, 1.0).expect("ask should validate"),
                captured_at_unix_ms: 50_000,
                received_at_unix_ms: 50_010,
            },
            order_book: Some(OrderBookSnapshot {
                id: "quality-audit-book".to_owned(),
                venue,
                pair,
                captured_at_unix_ms: 50_000,
                received_at_unix_ms: 50_010,
                bids: vec![PriceLevel::new(100.0, 1.0).expect("bid should validate")],
                asks: vec![PriceLevel::new(100.1, 1.0).expect("ask should validate")],
                source_sequence: Some("seq-quality-audit".to_owned()),
            }),
            now_unix_ms: 50_100,
            max_spread_bps: 20,
            min_depth_levels: 1,
            max_capture_latency_ms: 25,
            live_network_used: false,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .expect("quality assessment should validate");

        let audit_path = unique_temp_path("market-data-quality-assessment-audit", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let record =
            append_market_data_quality_assessment_audit(&mut journal, &report, 1_700_000_038)
                .expect("audit append should succeed");
        assert_eq!(record.event.subsystem, MARKET_DATA_STATE_SUBSYSTEM);
        assert_eq!(record.event.actor, "market-data-provider");

        let mut invalid = report.clone();
        invalid.production_ready = true;
        assert!(
            append_market_data_quality_assessment_audit(&mut journal, &invalid, 1_700_000_039)
                .is_err()
        );

        let mut store = InMemoryStateStore::new();
        let checkpoint =
            persist_market_data_quality_assessment_checkpoint(&mut store, &report, 1_700_000_040)
                .expect("checkpoint persist should succeed");
        assert_eq!(
            checkpoint.key,
            MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY
        );

        let recovered = store
            .get_checkpoint(MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY)
            .expect("state read should succeed")
            .expect("checkpoint should exist");
        let recovered_report: MarketDataQualityAssessmentReport =
            serde_json::from_str(&recovered.value).expect("checkpoint JSON should decode");
        assert_eq!(recovered_report, report);
        assert_eq!(
            recovered_report.status,
            MarketDataQualityAssessmentStatus::Acceptable
        );
    }

    #[test]
    fn market_data_provider_preflight_audit_and_state_reopen_locally() {
        let report = validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
            provider_name: "local-preflight-audit-provider".to_owned(),
            read_only: true,
            rate_limited: false,
            outage_observed: false,
            reconnect_required: true,
            reconnect_backoff_planned: true,
            samples_checked: 4,
            fresh_samples: 4,
            stale_samples: 0,
            max_observed_latency_ms: 12,
            max_allowed_latency_ms: 50,
            live_network_used: false,
            credential_loaded: false,
        })
        .expect("preflight report should validate");

        let audit_path = unique_temp_path("market-data-preflight-audit", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let record =
            append_market_data_provider_preflight_audit(&mut journal, &report, 1_700_000_040)
                .expect("audit append should succeed");
        assert_eq!(record.event.subsystem, MARKET_DATA_STATE_SUBSYSTEM);
        assert_eq!(record.event.actor, "market-data-provider");

        let next_sequence = journal.next_sequence();
        let mut invalid = report.clone();
        invalid.live_network_used = true;
        invalid.status = MarketDataProviderPreflightStatus::Usable;
        assert!(
            append_market_data_provider_preflight_audit(&mut journal, &invalid, 1_700_000_041)
                .is_err()
        );
        assert_eq!(journal.next_sequence(), next_sequence);

        let mut store = InMemoryStateStore::new();
        let checkpoint =
            persist_market_data_provider_preflight_checkpoint(&mut store, &report, 1_700_000_042)
                .expect("checkpoint persist should succeed");
        assert_eq!(
            checkpoint.key,
            MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY
        );

        let recovered = store
            .get_checkpoint(MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY)
            .expect("state read should succeed")
            .expect("checkpoint should exist");
        let recovered_report: MarketDataProviderPreflightReport =
            serde_json::from_str(&recovered.value).expect("checkpoint JSON should decode");
        assert_eq!(recovered_report, report);
        assert_eq!(
            recovered_report.status,
            MarketDataProviderPreflightStatus::Usable
        );
    }

    #[test]
    fn market_data_reconnect_plan_audit_and_state_reopen_locally() {
        let report = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
            plan_id: "reconnect-plan-audit-state".to_owned(),
            provider_name: "local-audit-provider".to_owned(),
            venue: local_cex_venue("paper-a"),
            disconnected_at_unix_ms: 40_000,
            planned_at_unix_ms: 40_010,
            attempt_number: 2,
            max_attempts: 4,
            base_backoff_ms: 100,
            max_backoff_ms: 800,
            planned_delay_ms: 250,
            provider_retry_after_ms: None,
            rate_limited: false,
            outage_observed: false,
            live_network_used: false,
            websocket_connection_opened: false,
            credential_loaded: false,
        })
        .expect("reconnect plan should validate");

        let audit_path = unique_temp_path("market-data-reconnect-plan-audit", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let record = append_market_data_reconnect_plan_audit(&mut journal, &report, 1_700_000_050)
            .expect("audit append should succeed");
        assert_eq!(record.event.subsystem, MARKET_DATA_STATE_SUBSYSTEM);
        assert_eq!(record.event.actor, "market-data-provider");

        let mut store = InMemoryStateStore::new();
        let checkpoint =
            persist_market_data_reconnect_plan_checkpoint(&mut store, &report, 1_700_000_051)
                .expect("checkpoint persist should succeed");
        assert_eq!(
            checkpoint.key,
            MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY
        );

        let recovered = store
            .get_checkpoint(MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY)
            .expect("state read should succeed")
            .expect("checkpoint should exist");
        let recovered_report: MarketDataReconnectPlanReport =
            serde_json::from_str(&recovered.value).expect("checkpoint JSON should decode");
        assert_eq!(recovered_report, report);
        assert_eq!(
            recovered_report.status,
            MarketDataReconnectPlanStatus::ReadyForLocalReview
        );
    }

    fn local_cex_venue(name: &str) -> VenueRef {
        VenueRef {
            kind: VenueKind::Cex,
            name: name.to_owned(),
        }
    }

    fn market_data_provider_latency_review_request(
        side_effect_claimed: bool,
        max_provider_latency_ms: u64,
        max_capture_latency_ms: u64,
        max_reconnect_delay_ms: u64,
    ) -> MarketDataProviderLatencyReviewRequest {
        let clean_preflight =
            validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
                provider_name: "local-review-clean".to_owned(),
                read_only: true,
                rate_limited: false,
                outage_observed: false,
                reconnect_required: true,
                reconnect_backoff_planned: true,
                samples_checked: 4,
                fresh_samples: 4,
                stale_samples: 0,
                max_observed_latency_ms: 12,
                max_allowed_latency_ms: 50,
                live_network_used: false,
                credential_loaded: false,
            })
            .expect("clean preflight should validate");
        let degraded_preflight =
            validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
                provider_name: "local-review-degraded".to_owned(),
                read_only: true,
                rate_limited: true,
                outage_observed: true,
                reconnect_required: true,
                reconnect_backoff_planned: false,
                samples_checked: 5,
                fresh_samples: 3,
                stale_samples: 2,
                max_observed_latency_ms: 250,
                max_allowed_latency_ms: 100,
                live_network_used: false,
                credential_loaded: false,
            })
            .expect("degraded preflight should validate");
        let ready_reconnect = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
            plan_id: "local-review-reconnect".to_owned(),
            provider_name: "local-review-clean".to_owned(),
            venue: local_cex_venue("paper-review"),
            disconnected_at_unix_ms: 10_000,
            planned_at_unix_ms: 10_050,
            attempt_number: 3,
            max_attempts: 5,
            base_backoff_ms: 100,
            max_backoff_ms: 1_000,
            planned_delay_ms: 500,
            provider_retry_after_ms: Some(450),
            rate_limited: true,
            outage_observed: false,
            live_network_used: false,
            websocket_connection_opened: false,
            credential_loaded: false,
        })
        .expect("reconnect plan should validate");
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let venue = local_cex_venue("paper-review-quality");
        let acceptable_quality = assess_market_data_quality(MarketDataQualityAssessmentInput {
            assessment_id: "local-review-quality".to_owned(),
            provider_name: "local-review-clean".to_owned(),
            request: MarketDataRequest {
                venue: venue.clone(),
                pair: pair.clone(),
                max_age_ms: 250,
            },
            quote: NormalizedQuote {
                id: "local-review-quality-quote".to_owned(),
                venue: venue.clone(),
                pair: pair.clone(),
                bid: PriceLevel::new(100.0, 1.0).expect("bid should validate"),
                ask: PriceLevel::new(100.1, 1.0).expect("ask should validate"),
                captured_at_unix_ms: 1_000,
                received_at_unix_ms: 1_015,
            },
            order_book: Some(OrderBookSnapshot {
                id: "local-review-quality-book".to_owned(),
                venue,
                pair,
                captured_at_unix_ms: 1_000,
                received_at_unix_ms: 1_015,
                bids: vec![
                    PriceLevel::new(100.0, 1.0).expect("bid should validate"),
                    PriceLevel::new(99.9, 1.0).expect("bid should validate"),
                ],
                asks: vec![
                    PriceLevel::new(100.1, 1.0).expect("ask should validate"),
                    PriceLevel::new(100.2, 1.0).expect("ask should validate"),
                ],
                source_sequence: Some("local-review-quality-seq".to_owned()),
            }),
            now_unix_ms: 1_120,
            max_spread_bps: 20,
            min_depth_levels: 2,
            max_capture_latency_ms: 25,
            live_network_used: false,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .expect("quality assessment should validate");
        let paid_provider_evaluation =
            validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
                evaluation_id: "local-review-paid-provider".to_owned(),
                provider_name: "local-review-paid-provider".to_owned(),
                covered_venues: vec![
                    local_cex_venue("paper-binance"),
                    local_cex_venue("paper-coinbase"),
                ],
                covered_pairs: vec![
                    MarketPair::new("BTC", "USDC").expect("pair should validate"),
                    MarketPair::new("ETH", "USDC").expect("pair should validate"),
                ],
                capabilities: MarketDataCapabilities {
                    order_book: true,
                    top_of_book: true,
                    fees: false,
                    websocket: true,
                    rest: true,
                },
                documented_latency_ms: 35,
                max_allowed_latency_ms: 50,
                max_requests_per_minute: 1_200,
                monthly_cost_usd: 499,
                failure_modes_reviewed: vec![
                    "provider-outage".to_owned(),
                    "stale-book".to_owned(),
                    "rate-limit-burst".to_owned(),
                ],
                rate_limit_documentation_reviewed: true,
                pricing_documentation_reviewed: true,
                terms_reviewed: true,
                credential_scope_reviewed: true,
                live_network_used: false,
                credential_loaded: false,
                production_ready_claimed: false,
            })
            .expect("paid-provider evaluation should validate");

        MarketDataProviderLatencyReviewRequest {
            review_id: "local-provider-latency-review".to_owned(),
            clean_preflight,
            degraded_preflight,
            ready_reconnect,
            acceptable_quality,
            paid_provider_evaluation,
            max_provider_latency_ms,
            max_capture_latency_ms,
            max_reconnect_delay_ms,
            min_quality_score: 100,
            min_samples_checked: 4,
            remaining_external_evidence: vec![
                "live REST/WebSocket exchange adapters".to_owned(),
                "provider-backed latency and throughput measurement".to_owned(),
                "provider-side rate-limit and outage reconciliation".to_owned(),
                "deployment-host market-data resource profiling".to_owned(),
                "external sandbox/live calibration".to_owned(),
            ],
            live_network_used: side_effect_claimed,
            websocket_connection_opened: side_effect_claimed,
            credential_loaded: side_effect_claimed,
            production_ready_claimed: side_effect_claimed,
        }
    }

    fn market_data_provider_reconciliation_review_request(
        side_effect_claimed: bool,
        outage_evidence_ready: bool,
    ) -> MarketDataProviderReconciliationReviewRequest {
        let latency_review = review_market_data_provider_latency(
            market_data_provider_latency_review_request(false, 50, 25, 500),
        )
        .expect("latency review should validate");
        let degraded_preflight =
            validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
                provider_name: "local-review-degraded".to_owned(),
                read_only: true,
                rate_limited: true,
                outage_observed: true,
                reconnect_required: true,
                reconnect_backoff_planned: false,
                samples_checked: 5,
                fresh_samples: 3,
                stale_samples: 2,
                max_observed_latency_ms: 250,
                max_allowed_latency_ms: 100,
                live_network_used: false,
                credential_loaded: false,
            })
            .expect("degraded preflight should validate");
        let rate_limit_reconnect =
            validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
                plan_id: "local-provider-reconciliation-rate-limit".to_owned(),
                provider_name: "local-review-degraded".to_owned(),
                venue: local_cex_venue("paper-provider-reconciliation"),
                disconnected_at_unix_ms: 20_000,
                planned_at_unix_ms: 20_050,
                attempt_number: 3,
                max_attempts: 5,
                base_backoff_ms: 100,
                max_backoff_ms: 1_000,
                planned_delay_ms: 500,
                provider_retry_after_ms: Some(450),
                rate_limited: true,
                outage_observed: false,
                live_network_used: false,
                websocket_connection_opened: false,
                credential_loaded: false,
            })
            .expect("rate-limit reconnect plan should validate");
        let outage_reconnect = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
            plan_id: "local-provider-reconciliation-outage".to_owned(),
            provider_name: "local-review-degraded".to_owned(),
            venue: local_cex_venue("paper-provider-reconciliation"),
            disconnected_at_unix_ms: 30_000,
            planned_at_unix_ms: 30_010,
            attempt_number: if outage_evidence_ready { 6 } else { 2 },
            max_attempts: 5,
            base_backoff_ms: 100,
            max_backoff_ms: 1_000,
            planned_delay_ms: 1_000,
            provider_retry_after_ms: Some(800),
            rate_limited: true,
            outage_observed: outage_evidence_ready,
            live_network_used: false,
            websocket_connection_opened: false,
            credential_loaded: false,
        })
        .expect("outage reconnect plan should validate");

        MarketDataProviderReconciliationReviewRequest {
            review_id: "local-provider-reconciliation-review".to_owned(),
            latency_review,
            degraded_preflight,
            rate_limit_reconnect,
            outage_reconnect,
            min_degraded_samples_checked: 5,
            remaining_external_evidence: vec![
                "live REST/WebSocket exchange adapters".to_owned(),
                "provider-backed rate-limit reconciliation".to_owned(),
                "provider-backed outage reconciliation".to_owned(),
                "deployment-host market-data resource profiling".to_owned(),
                "external sandbox/live calibration".to_owned(),
            ],
            live_network_used: side_effect_claimed,
            websocket_connection_opened: side_effect_claimed,
            credential_loaded: side_effect_claimed,
            production_ready_claimed: side_effect_claimed,
        }
    }

    fn unique_temp_path(label: &str, extension: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "arbyclaw-market-data-{label}-{}-{nanos}-{n}.{extension}",
            std::process::id()
        ))
    }
}
