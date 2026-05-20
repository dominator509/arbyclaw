#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{VenueKind, VenueRef};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable market-data model version for audit and replay surfaces.
pub const MARKET_DATA_MODEL_VERSION: &str = "phase-5-market-data-v1";

/// Conservative default freshness ceiling for normalized market snapshots.
pub const DEFAULT_MARKET_DATA_FRESHNESS_MS: u64 = 5_000;

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

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

#[cfg(test)]
mod tests {
    use super::{FreshnessStatus, MarketPair, NormalizedQuote, OrderBookSnapshot, PriceLevel};
    use crate::{VenueKind, VenueRef};

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
}
