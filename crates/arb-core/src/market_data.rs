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
        append_market_data_provider_preflight_audit, append_market_data_reconnect_plan_audit,
        persist_market_data_provider_preflight_checkpoint,
        persist_market_data_reconnect_plan_checkpoint, validate_market_data_provider_preflight,
        validate_market_data_reconnect_plan, FreshnessStatus, MarketDataProviderHealthObservation,
        MarketDataProviderPreflightReport, MarketDataProviderPreflightStatus,
        MarketDataReconnectPlanInput, MarketDataReconnectPlanReport, MarketDataReconnectPlanStatus,
        MarketPair, NormalizedQuote, OrderBookSnapshot, PriceLevel,
        MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY,
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
