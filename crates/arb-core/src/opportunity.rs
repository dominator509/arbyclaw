#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    FeeAdjustedEdge, FeeEstimate, FeeModelError, FeeSchedule, FreshnessStatus, LiquidityRole,
    MarketPair, NormalizedQuote, OrderBookSnapshot, PriceLevel, VenueKind, VenueRef,
    DEFAULT_MARKET_DATA_FRESHNESS_MS,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, error::Error, fmt};

/// Stable opportunity-engine version for audit and replay surfaces.
pub const OPPORTUNITY_ENGINE_VERSION: &str = "phase-27-opportunity-risk-v1";

/// Deterministic route classification for opportunity records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpportunityRouteKind {
    /// Buy and sell across centralized exchanges.
    CexCex,
    /// Buy and sell across decentralized venues or aggregators.
    DexDex,
    /// Route crosses a centralized and decentralized venue boundary.
    CexDex,
    /// Multi-leg same-venue or cross-venue triangular route boundary.
    Triangular,
}

/// Non-executing leg side for opportunity modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpportunityLegSide {
    /// Model-side buy at the ask.
    Buy,
    /// Model-side sell at the bid.
    Sell,
    /// Model-side swap boundary for future DEX/triangular route models.
    Swap,
}

/// Conservative discovery settings.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityDiscoveryConfig {
    /// Maximum accepted age for each source quote.
    pub max_market_data_age_ms: u64,
    /// Minimum net profit required before a candidate is emitted.
    pub min_net_profit_quote: f64,
    /// Minimum net return required before a candidate is emitted.
    pub min_roi_bps: f64,
    /// Maximum returned candidates after deterministic ranking.
    pub max_candidates: usize,
    /// Penalty applied when either leg uses unverified fee metadata.
    pub unverified_fee_penalty_bps: f64,
    /// Additional risk penalty for DEX/DEX candidates.
    pub dex_dex_risk_penalty_bps: f64,
    /// Additional risk penalty for CEX/DEX candidates.
    pub cex_dex_risk_penalty_bps: f64,
    /// Maximum freshness penalty at the configured age ceiling.
    pub freshness_penalty_ceiling_bps: f64,
    /// Default fraction of supplied paper inventory allowed per candidate.
    pub default_inventory_fraction: f64,
    /// Reject transfer-risk profiles above this local latency ceiling.
    pub max_transfer_latency_ms: u64,
}

impl Default for OpportunityDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_market_data_age_ms: DEFAULT_MARKET_DATA_FRESHNESS_MS,
            min_net_profit_quote: 0.0,
            min_roi_bps: 0.0,
            max_candidates: 25,
            unverified_fee_penalty_bps: 5.0,
            dex_dex_risk_penalty_bps: 10.0,
            cex_dex_risk_penalty_bps: 15.0,
            freshness_penalty_ceiling_bps: 2.0,
            default_inventory_fraction: 1.0,
            max_transfer_latency_ms: 300_000,
        }
    }
}

impl OpportunityDiscoveryConfig {
    /// Validate discovery settings before any ranking is attempted.
    pub fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();

        if self.max_market_data_age_ms == 0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_MAX_AGE_ZERO",
                "max_market_data_age_ms must be positive",
            ));
        }

        if self.max_candidates == 0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_MAX_CANDIDATES_ZERO",
                "max_candidates must be positive",
            ));
        }

        if !self.default_inventory_fraction.is_finite()
            || self.default_inventory_fraction <= 0.0
            || self.default_inventory_fraction > 1.0
        {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_INVENTORY_FRACTION_INVALID",
                "default_inventory_fraction must be finite and in the interval (0, 1]",
            ));
        }

        if self.max_transfer_latency_ms == 0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_TRANSFER_LATENCY_LIMIT_ZERO",
                "max_transfer_latency_ms must be positive",
            ));
        }

        for (field, value) in [
            ("min_net_profit_quote", self.min_net_profit_quote),
            ("min_roi_bps", self.min_roi_bps),
            (
                "unverified_fee_penalty_bps",
                self.unverified_fee_penalty_bps,
            ),
            ("dex_dex_risk_penalty_bps", self.dex_dex_risk_penalty_bps),
            ("cex_dex_risk_penalty_bps", self.cex_dex_risk_penalty_bps),
            (
                "freshness_penalty_ceiling_bps",
                self.freshness_penalty_ceiling_bps,
            ),
        ] {
            if !is_non_negative_finite(value) {
                violations.push(OpportunityViolation::new_owned(
                    "OPPORTUNITY_CONFIG_VALUE_INVALID",
                    format!("{field} must be finite and non-negative"),
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Non-executing opportunity discovery request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityDiscoveryRequest {
    /// Request id for audit and deterministic replay.
    pub id: String,
    /// Already-normalized market quotes supplied by a caller or paper provider.
    pub quotes: Vec<NormalizedQuote>,
    /// Fee schedules supplied by a caller or paper provider.
    pub fee_schedules: Vec<FeeSchedule>,
    /// Optional local order books used for depth-aware sizing and slippage.
    pub order_books: Vec<OrderBookSnapshot>,
    /// Optional local paper inventory caps used to avoid oversizing candidates.
    pub inventory_limits: Vec<OpportunityInventoryLimit>,
    /// Optional local transfer-latency and settlement-risk profiles.
    pub transfer_risk_profiles: Vec<OpportunityTransferRiskProfile>,
    /// Discovery configuration.
    pub config: OpportunityDiscoveryConfig,
    /// Runtime clock in Unix milliseconds used for freshness checks.
    pub now_unix_ms: u64,
}

impl OpportunityDiscoveryRequest {
    /// Validate request structure, quote freshness, and supplied fee schedules.
    pub fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        validate_id("request", &self.id, &mut violations);

        if let Err(OpportunityError::ValidationFailed {
            violations: config_violations,
        }) = self.config.validate()
        {
            violations.extend(config_violations);
        }

        if self.quotes.len() < 2 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_QUOTES_INSUFFICIENT",
                "at least two normalized quotes are required for cross-venue discovery",
            ));
        }

        if self.fee_schedules.is_empty() {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_FEE_SCHEDULES_EMPTY",
                "at least one fee schedule is required for fee-aware discovery",
            ));
        }

        for quote in &self.quotes {
            collect_quote_violations(
                quote,
                self.now_unix_ms,
                self.config.max_market_data_age_ms,
                &mut violations,
            );
        }

        for book in &self.order_books {
            collect_order_book_violations(
                book,
                self.now_unix_ms,
                self.config.max_market_data_age_ms,
                &mut violations,
            );
        }

        for limit in &self.inventory_limits {
            if let Err(OpportunityError::ValidationFailed {
                violations: limit_violations,
            }) = limit.validate()
            {
                violations.extend(limit_violations);
            }
        }

        for profile in &self.transfer_risk_profiles {
            if let Err(OpportunityError::ValidationFailed {
                violations: profile_violations,
            }) = profile.validate(self.config.max_transfer_latency_ms)
            {
                violations.extend(profile_violations);
            }
        }

        for schedule in &self.fee_schedules {
            if let Err(error) = schedule.validate() {
                collect_fee_model_error(
                    "OPPORTUNITY_FEE_SCHEDULE_INVALID",
                    &error,
                    &mut violations,
                );
            }
        }

        finish_validation(violations)
    }
}

/// Local paper inventory available to the opportunity engine for sizing only.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityInventoryLimit {
    /// Venue where the local simulated inventory is available.
    pub venue: VenueRef,
    /// Market pair the inventory applies to.
    pub pair: MarketPair,
    /// Available base units for sell-side sizing.
    pub available_base: f64,
    /// Available quote units for buy-side sizing.
    pub available_quote: f64,
    /// Optional per-limit fraction override in the interval (0, 1].
    pub max_fraction: Option<f64>,
}

impl OpportunityInventoryLimit {
    /// Validate local paper inventory sizing input.
    pub fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        validate_venue(&self.venue, &mut violations);
        if let Err(error) = self.pair.validate() {
            collect_market_data_error(
                "OPPORTUNITY_INVENTORY_PAIR_INVALID",
                &error,
                &mut violations,
            );
        }
        if !is_non_negative_finite(self.available_base) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_INVENTORY_BASE_INVALID",
                "available_base must be finite and non-negative",
            ));
        }
        if !is_non_negative_finite(self.available_quote) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_INVENTORY_QUOTE_INVALID",
                "available_quote must be finite and non-negative",
            ));
        }
        if let Some(max_fraction) = self.max_fraction {
            if !max_fraction.is_finite() || max_fraction <= 0.0 || max_fraction > 1.0 {
                violations.push(OpportunityViolation::new(
                    "OPPORTUNITY_INVENTORY_LIMIT_FRACTION_INVALID",
                    "inventory max_fraction must be finite and in the interval (0, 1]",
                ));
            }
        }
        finish_validation(violations)
    }
}

/// Reference-only local transfer risk used for scoring cross-venue candidates.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityTransferRiskProfile {
    /// Source venue for modeled inventory movement.
    pub from_venue: VenueRef,
    /// Destination venue for modeled inventory movement.
    pub to_venue: VenueRef,
    /// Market pair this local risk profile applies to.
    pub pair: MarketPair,
    /// Operator-supplied local latency estimate.
    pub estimated_latency_ms: u64,
    /// Additional deterministic score penalty.
    pub risk_penalty_bps: f64,
    /// Sanitized non-secret evidence or assumption label.
    pub evidence_label: String,
}

impl OpportunityTransferRiskProfile {
    /// Validate transfer-risk scoring input without calling external systems.
    pub fn validate(&self, max_latency_ms: u64) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        validate_venue(&self.from_venue, &mut violations);
        validate_venue(&self.to_venue, &mut violations);
        if same_venue(&self.from_venue, &self.to_venue) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_TRANSFER_SAME_VENUE",
                "transfer-risk profiles require distinct venues",
            ));
        }
        if let Err(error) = self.pair.validate() {
            collect_market_data_error("OPPORTUNITY_TRANSFER_PAIR_INVALID", &error, &mut violations);
        }
        if self.estimated_latency_ms == 0 || self.estimated_latency_ms > max_latency_ms {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_TRANSFER_LATENCY_INVALID",
                "estimated transfer latency must be positive and within the configured ceiling",
            ));
        }
        if !is_non_negative_finite(self.risk_penalty_bps) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_TRANSFER_RISK_PENALTY_INVALID",
                "transfer risk penalty must be finite and non-negative",
            ));
        }
        validate_id(
            "transfer risk evidence",
            &self.evidence_label,
            &mut violations,
        );
        finish_validation(violations)
    }
}

/// Local depth and inventory sizing details attached to a candidate.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityLiquidityModel {
    /// Whether full order-book depth was available for both legs.
    pub depth_aware: bool,
    /// Requested size before depth and inventory caps.
    pub requested_quantity_base: f64,
    /// Final executable local model size.
    pub executable_quantity_base: f64,
    /// Buy-side average price after walking local asks.
    pub buy_average_price_quote: f64,
    /// Sell-side average price after walking local bids.
    pub sell_average_price_quote: f64,
    /// Buy-side slippage relative to the top ask.
    pub buy_slippage_bps: f64,
    /// Sell-side slippage relative to the top bid.
    pub sell_slippage_bps: f64,
    /// True when supplied paper inventory reduced the candidate size.
    pub inventory_capped: bool,
}

impl OpportunityLiquidityModel {
    fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        for (field, value) in [
            ("requested_quantity_base", self.requested_quantity_base),
            ("executable_quantity_base", self.executable_quantity_base),
            ("buy_average_price_quote", self.buy_average_price_quote),
            ("sell_average_price_quote", self.sell_average_price_quote),
        ] {
            if !is_positive_finite(value) {
                violations.push(OpportunityViolation::new_owned(
                    "OPPORTUNITY_LIQUIDITY_VALUE_INVALID",
                    format!("{field} must be positive and finite"),
                ));
            }
        }
        for (field, value) in [
            ("buy_slippage_bps", self.buy_slippage_bps),
            ("sell_slippage_bps", self.sell_slippage_bps),
        ] {
            if !is_non_negative_finite(value) {
                violations.push(OpportunityViolation::new_owned(
                    "OPPORTUNITY_LIQUIDITY_SLIPPAGE_INVALID",
                    format!("{field} must be finite and non-negative"),
                ));
            }
        }
        if self.executable_quantity_base > self.requested_quantity_base {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_LIQUIDITY_SIZE_INCREASED",
                "executable quantity cannot exceed requested quantity",
            ));
        }
        finish_validation(violations)
    }
}

/// Transfer-risk scoring details attached to a cross-venue candidate.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityTransferRisk {
    /// Operator-supplied local latency estimate.
    pub estimated_latency_ms: u64,
    /// Additional score penalty in basis points.
    pub risk_penalty_bps: f64,
    /// Sanitized non-secret evidence or assumption label.
    pub evidence_label: String,
}

impl OpportunityTransferRisk {
    fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        if self.estimated_latency_ms == 0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_TRANSFER_LATENCY_INVALID",
                "candidate transfer latency must be positive",
            ));
        }
        if !is_non_negative_finite(self.risk_penalty_bps) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_TRANSFER_RISK_PENALTY_INVALID",
                "candidate transfer risk penalty must be finite and non-negative",
            ));
        }
        validate_id(
            "transfer risk evidence",
            &self.evidence_label,
            &mut violations,
        );
        finish_validation(violations)
    }
}

/// One non-executing leg in an opportunity record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityLeg {
    /// Venue referenced by this model leg.
    pub venue: VenueRef,
    /// Market pair for this leg.
    pub pair: MarketPair,
    /// Non-executing model side.
    pub side: OpportunityLegSide,
    /// Price in quote units used for the edge model.
    pub price_quote: f64,
    /// Modeled quantity in base units.
    pub quantity_base: f64,
    /// Modeled notional in quote units.
    pub notional_quote: f64,
    /// Deterministic fee estimate for this model leg.
    pub fee_estimate: FeeEstimate,
    /// Source quote id used for deterministic replay.
    pub source_quote_id: String,
    /// Age of the source market data at discovery time.
    pub market_data_age_ms: u64,
}

impl OpportunityLeg {
    /// Validate a non-executing opportunity leg.
    pub fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        validate_venue(&self.venue, &mut violations);

        if let Err(error) = self.pair.validate() {
            collect_market_data_error("OPPORTUNITY_LEG_PAIR_INVALID", &error, &mut violations);
        }

        if !is_positive_finite(self.price_quote) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_LEG_PRICE_INVALID",
                "leg price_quote must be positive and finite",
            ));
        }

        if !is_positive_finite(self.quantity_base) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_LEG_QUANTITY_INVALID",
                "leg quantity_base must be positive and finite",
            ));
        }

        if !is_positive_finite(self.notional_quote) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_LEG_NOTIONAL_INVALID",
                "leg notional_quote must be positive and finite",
            ));
        }

        let expected_notional = self.price_quote * self.quantity_base;
        if expected_notional.is_finite()
            && (expected_notional - self.notional_quote).abs() > 0.000_000_01
        {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_LEG_NOTIONAL_MISMATCH",
                "leg notional_quote must equal price_quote multiplied by quantity_base",
            ));
        }

        if let Err(error) = self.fee_estimate.validate() {
            collect_fee_model_error("OPPORTUNITY_LEG_FEE_INVALID", &error, &mut violations);
        }

        validate_id("source quote", &self.source_quote_id, &mut violations);

        finish_validation(violations)
    }
}

/// Deterministic score used only for ranking candidates.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityScore {
    /// Return after fees before penalties.
    pub roi_bps: f64,
    /// Penalty derived from quote age.
    pub freshness_penalty_bps: f64,
    /// Penalty derived from route kind and metadata confidence.
    pub risk_penalty_bps: f64,
    /// Final ranking score.
    pub score_bps: f64,
}

impl OpportunityScore {
    /// Create a deterministic score from edge, freshness, and route risk.
    pub fn calculate(
        edge: FeeAdjustedEdge,
        max_quote_age_ms: u64,
        config: OpportunityDiscoveryConfig,
        route_kind: OpportunityRouteKind,
        fees_externally_verified: bool,
        extra_risk_penalty_bps: f64,
    ) -> Result<Self, OpportunityError> {
        let mut violations = Vec::new();

        if !edge.net_profit_quote.is_finite() || !edge.roi_bps.is_finite() {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_EDGE_INVALID",
                "fee-adjusted edge must contain finite net profit and ROI",
            ));
        }

        if config.max_market_data_age_ms == 0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_MAX_AGE_ZERO",
                "max_market_data_age_ms must be positive before scoring",
            ));
        }

        if !is_non_negative_finite(extra_risk_penalty_bps) {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_EXTRA_RISK_PENALTY_INVALID",
                "extra risk penalty must be finite and non-negative before scoring",
            ));
        }

        finish_validation(violations)?;

        let age_ratio = (max_quote_age_ms as f64 / config.max_market_data_age_ms as f64).min(1.0);
        let freshness_penalty_bps = age_ratio * config.freshness_penalty_ceiling_bps;
        let route_penalty_bps = match route_kind {
            OpportunityRouteKind::CexCex | OpportunityRouteKind::Triangular => 0.0,
            OpportunityRouteKind::DexDex => config.dex_dex_risk_penalty_bps,
            OpportunityRouteKind::CexDex => config.cex_dex_risk_penalty_bps,
        };
        let fee_penalty_bps = if fees_externally_verified {
            0.0
        } else {
            config.unverified_fee_penalty_bps
        };
        let risk_penalty_bps = route_penalty_bps + fee_penalty_bps + extra_risk_penalty_bps;
        let score_bps = edge.roi_bps - freshness_penalty_bps - risk_penalty_bps;

        Ok(Self {
            roi_bps: edge.roi_bps,
            freshness_penalty_bps,
            risk_penalty_bps,
            score_bps,
        })
    }

    /// Validate score fields.
    pub fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        for (field, value) in [
            ("roi_bps", self.roi_bps),
            ("freshness_penalty_bps", self.freshness_penalty_bps),
            ("risk_penalty_bps", self.risk_penalty_bps),
            ("score_bps", self.score_bps),
        ] {
            if !value.is_finite() {
                violations.push(OpportunityViolation::new_owned(
                    "OPPORTUNITY_SCORE_VALUE_INVALID",
                    format!("{field} must be finite"),
                ));
            }
        }

        if self.freshness_penalty_bps < 0.0 || self.risk_penalty_bps < 0.0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_SCORE_PENALTY_NEGATIVE",
                "score penalties must be non-negative",
            ));
        }

        finish_validation(violations)
    }
}

#[derive(Debug, Clone, Copy)]
struct DepthWalk {
    quantity_base: f64,
    average_price_quote: f64,
}

fn build_liquidity_model(
    buy_quote: &NormalizedQuote,
    sell_quote: &NormalizedQuote,
    requested_quantity_base: f64,
    request: &OpportunityDiscoveryRequest,
) -> Result<Option<OpportunityLiquidityModel>, OpportunityError> {
    let inventory_capped_quantity =
        apply_inventory_caps(buy_quote, sell_quote, requested_quantity_base, request);
    if !is_positive_finite(inventory_capped_quantity) {
        return Ok(None);
    }

    let buy_book = find_order_book(&request.order_books, &buy_quote.venue, &buy_quote.pair);
    let sell_book = find_order_book(&request.order_books, &sell_quote.venue, &sell_quote.pair);
    let depth_aware = buy_book.is_some() && sell_book.is_some();

    let buy_depth = match buy_book {
        Some(book) => walk_depth(&book.asks, inventory_capped_quantity),
        None => Some(DepthWalk {
            quantity_base: inventory_capped_quantity.min(buy_quote.ask.quantity_base),
            average_price_quote: buy_quote.ask.price_quote,
        }),
    };
    let sell_depth = match sell_book {
        Some(book) => walk_depth(&book.bids, inventory_capped_quantity),
        None => Some(DepthWalk {
            quantity_base: inventory_capped_quantity.min(sell_quote.bid.quantity_base),
            average_price_quote: sell_quote.bid.price_quote,
        }),
    };

    let (Some(buy_depth), Some(sell_depth)) = (buy_depth, sell_depth) else {
        return Ok(None);
    };
    let executable_quantity_base = buy_depth.quantity_base.min(sell_depth.quantity_base);
    if !is_positive_finite(executable_quantity_base) {
        return Ok(None);
    }

    let buy_average_price_quote = if buy_depth.quantity_base > executable_quantity_base {
        walk_depth_average(
            buy_book.map_or(&[buy_quote.ask][..], |book| book.asks.as_slice()),
            executable_quantity_base,
        )
        .ok_or_else(|| OpportunityError::ValidationFailed {
            violations: vec![OpportunityViolation::new(
                "OPPORTUNITY_DEPTH_BUY_REWALK_FAILED",
                "buy depth could not be re-walked at executable size",
            )],
        })?
    } else {
        buy_depth.average_price_quote
    };
    let sell_average_price_quote = if sell_depth.quantity_base > executable_quantity_base {
        walk_depth_average(
            sell_book.map_or(&[sell_quote.bid][..], |book| book.bids.as_slice()),
            executable_quantity_base,
        )
        .ok_or_else(|| OpportunityError::ValidationFailed {
            violations: vec![OpportunityViolation::new(
                "OPPORTUNITY_DEPTH_SELL_REWALK_FAILED",
                "sell depth could not be re-walked at executable size",
            )],
        })?
    } else {
        sell_depth.average_price_quote
    };

    let buy_slippage_bps = ((buy_average_price_quote - buy_quote.ask.price_quote)
        / buy_quote.ask.price_quote)
        .max(0.0)
        * 10_000.0;
    let sell_slippage_bps = ((sell_quote.bid.price_quote - sell_average_price_quote)
        / sell_quote.bid.price_quote)
        .max(0.0)
        * 10_000.0;

    Ok(Some(OpportunityLiquidityModel {
        depth_aware,
        requested_quantity_base,
        executable_quantity_base,
        buy_average_price_quote,
        sell_average_price_quote,
        buy_slippage_bps,
        sell_slippage_bps,
        inventory_capped: executable_quantity_base < requested_quantity_base,
    }))
}

fn apply_inventory_caps(
    buy_quote: &NormalizedQuote,
    sell_quote: &NormalizedQuote,
    requested_quantity_base: f64,
    request: &OpportunityDiscoveryRequest,
) -> f64 {
    let mut capped = requested_quantity_base;
    for limit in &request.inventory_limits {
        if limit.pair != buy_quote.pair {
            continue;
        }
        let fraction = limit
            .max_fraction
            .unwrap_or(request.config.default_inventory_fraction);
        if same_venue(&limit.venue, &buy_quote.venue) {
            capped = capped.min((limit.available_quote * fraction) / buy_quote.ask.price_quote);
        }
        if same_venue(&limit.venue, &sell_quote.venue) {
            capped = capped.min(limit.available_base * fraction);
        }
    }
    capped
}

fn find_order_book<'a>(
    books: &'a [OrderBookSnapshot],
    venue: &VenueRef,
    pair: &MarketPair,
) -> Option<&'a OrderBookSnapshot> {
    books
        .iter()
        .find(|book| same_venue(&book.venue, venue) && &book.pair == pair)
}

fn walk_depth(levels: &[PriceLevel], requested_quantity_base: f64) -> Option<DepthWalk> {
    let quantity_base = levels
        .iter()
        .map(|level| level.quantity_base)
        .sum::<f64>()
        .min(requested_quantity_base);
    if !is_positive_finite(quantity_base) {
        return None;
    }
    let average_price_quote = walk_depth_average(levels, quantity_base)?;
    Some(DepthWalk {
        quantity_base,
        average_price_quote,
    })
}

fn walk_depth_average(levels: &[PriceLevel], quantity_base: f64) -> Option<f64> {
    if !is_positive_finite(quantity_base) {
        return None;
    }
    let mut remaining = quantity_base;
    let mut notional_quote = 0.0;
    for level in levels {
        if remaining <= 0.0 {
            break;
        }
        let consumed = remaining.min(level.quantity_base);
        notional_quote = consumed.mul_add(level.price_quote, notional_quote);
        remaining -= consumed;
    }
    if remaining > 0.000_000_01 {
        return None;
    }
    Some(notional_quote / quantity_base)
}

fn find_transfer_risk(
    request: &OpportunityDiscoveryRequest,
    from_venue: &VenueRef,
    to_venue: &VenueRef,
    pair: &MarketPair,
) -> Option<OpportunityTransferRisk> {
    request
        .transfer_risk_profiles
        .iter()
        .find(|profile| {
            same_venue(&profile.from_venue, from_venue)
                && same_venue(&profile.to_venue, to_venue)
                && &profile.pair == pair
        })
        .map(|profile| OpportunityTransferRisk {
            estimated_latency_ms: profile.estimated_latency_ms,
            risk_penalty_bps: profile.risk_penalty_bps,
            evidence_label: profile.evidence_label.clone(),
        })
}

/// Non-executing opportunity candidate.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityCandidate {
    /// Stable deterministic candidate id.
    pub id: String,
    /// Route class.
    pub route_kind: OpportunityRouteKind,
    /// Candidate market pair.
    pub pair: MarketPair,
    /// Non-executing modeled legs.
    pub legs: Vec<OpportunityLeg>,
    /// Fee-adjusted edge.
    pub edge: FeeAdjustedEdge,
    /// Deterministic ranking score.
    pub score: OpportunityScore,
    /// Depth and local inventory sizing details.
    pub liquidity_model: Option<OpportunityLiquidityModel>,
    /// Transfer-latency and settlement-risk scoring details.
    pub transfer_risk: Option<OpportunityTransferRisk>,
    /// Runtime discovery timestamp.
    pub discovered_at_unix_ms: u64,
    /// Source quote ids used for deterministic replay.
    pub source_quote_ids: Vec<String>,
    /// Non-secret caution notes for operators and future audit events.
    pub warnings: Vec<String>,
}

impl OpportunityCandidate {
    /// Validate a candidate before ranking or handoff to a future planner.
    pub fn validate(&self) -> Result<(), OpportunityError> {
        let mut violations = Vec::new();
        validate_id("candidate", &self.id, &mut violations);

        if let Err(error) = self.pair.validate() {
            collect_market_data_error("OPPORTUNITY_PAIR_INVALID", &error, &mut violations);
        }

        let expected_leg_count = match self.route_kind {
            OpportunityRouteKind::CexCex
            | OpportunityRouteKind::DexDex
            | OpportunityRouteKind::CexDex => 2,
            OpportunityRouteKind::Triangular => 3,
        };
        if self.legs.len() != expected_leg_count {
            violations.push(OpportunityViolation::new_owned(
                "OPPORTUNITY_LEG_COUNT_INVALID",
                format!(
                    "{:?} candidates require {expected_leg_count} legs",
                    self.route_kind
                ),
            ));
        }

        for leg in &self.legs {
            if let Err(OpportunityError::ValidationFailed {
                violations: leg_violations,
            }) = leg.validate()
            {
                violations.extend(leg_violations);
            }
        }

        if self.route_kind == OpportunityRouteKind::Triangular {
            collect_triangular_leg_violations(&self.legs, &mut violations);
        } else {
            for leg in &self.legs {
                if leg.pair != self.pair {
                    violations.push(OpportunityViolation::new(
                    "OPPORTUNITY_LEG_PAIR_MISMATCH",
                    "each leg pair must match the candidate pair for Phase 9 cross-venue records",
                ));
                }
            }
        }

        if !self.edge.is_profitable() {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_EDGE_NOT_PROFITABLE",
                "candidate edge must be profitable after known fees",
            ));
        }

        if let Err(error) = self.score.validate() {
            collect_opportunity_error(error, &mut violations);
        }

        if let Some(liquidity_model) = &self.liquidity_model {
            if let Err(error) = liquidity_model.validate() {
                collect_opportunity_error(error, &mut violations);
            }
        }

        if let Some(transfer_risk) = &self.transfer_risk {
            if let Err(error) = transfer_risk.validate() {
                collect_opportunity_error(error, &mut violations);
            }
        }

        if self.discovered_at_unix_ms == 0 {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_DISCOVERY_TIME_ZERO",
                "discovered_at_unix_ms must be non-zero",
            ));
        }

        if self.source_quote_ids.is_empty() {
            violations.push(OpportunityViolation::new(
                "OPPORTUNITY_SOURCE_QUOTES_EMPTY",
                "candidate must record source quote ids",
            ));
        }

        for source_quote_id in &self.source_quote_ids {
            validate_id("source quote", source_quote_id, &mut violations);
        }

        finish_validation(violations)
    }
}

/// Boundary trait for future opportunity engines.
///
/// Implementations must not place orders, sign transactions, withdraw funds,
/// bridge assets, mutate balances, or call live exchange/RPC endpoints.
pub trait OpportunityEngine {
    /// Stable engine name for diagnostics and audit records.
    fn engine_name(&self) -> &str;

    /// Discover and rank non-executing opportunity candidates.
    fn discover(
        &self,
        request: &OpportunityDiscoveryRequest,
    ) -> Result<Vec<OpportunityCandidate>, OpportunityError>;
}

/// Deterministic top-of-book opportunity engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicOpportunityEngine;

impl DeterministicOpportunityEngine {
    /// Create a deterministic opportunity engine.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl OpportunityEngine for DeterministicOpportunityEngine {
    fn engine_name(&self) -> &str {
        "deterministic-phase-9-opportunity-engine"
    }

    fn discover(
        &self,
        request: &OpportunityDiscoveryRequest,
    ) -> Result<Vec<OpportunityCandidate>, OpportunityError> {
        request.validate()?;

        let mut candidates = Vec::new();
        for buy_quote in &request.quotes {
            for sell_quote in &request.quotes {
                if same_venue(&buy_quote.venue, &sell_quote.venue)
                    || buy_quote.pair != sell_quote.pair
                {
                    continue;
                }

                let Some(route_kind) = route_kind_for(&buy_quote.venue, &sell_quote.venue) else {
                    continue;
                };

                if let Some(candidate) =
                    build_cross_venue_candidate(buy_quote, sell_quote, route_kind, request)?
                {
                    candidates.push(candidate);
                }
            }
        }
        discover_triangular_candidates(request, &mut candidates)?;

        rank_candidates(candidates, request.config.max_candidates)
    }
}

fn build_cross_venue_candidate(
    buy_quote: &NormalizedQuote,
    sell_quote: &NormalizedQuote,
    route_kind: OpportunityRouteKind,
    request: &OpportunityDiscoveryRequest,
) -> Result<Option<OpportunityCandidate>, OpportunityError> {
    let requested_quantity_base = buy_quote
        .ask
        .quantity_base
        .min(sell_quote.bid.quantity_base);
    if !is_positive_finite(requested_quantity_base) {
        return Ok(None);
    }

    let Some(sizing) =
        build_liquidity_model(buy_quote, sell_quote, requested_quantity_base, request)?
    else {
        return Ok(None);
    };
    let quantity_base = sizing.executable_quantity_base;
    let buy_notional_quote = sizing.buy_average_price_quote * quantity_base;
    let sell_notional_quote = sizing.sell_average_price_quote * quantity_base;
    let gross_profit_quote = sell_notional_quote - buy_notional_quote;

    if gross_profit_quote <= 0.0 || !gross_profit_quote.is_finite() {
        return Ok(None);
    }

    let buy_schedule =
        find_fee_schedule(&request.fee_schedules, &buy_quote.venue, &buy_quote.pair)?;
    let sell_schedule =
        find_fee_schedule(&request.fee_schedules, &sell_quote.venue, &sell_quote.pair)?;
    let buy_fee = estimate_fee(buy_schedule, buy_notional_quote)?;
    let sell_fee = estimate_fee(sell_schedule, sell_notional_quote)?;
    let total_fees_quote = buy_fee.total_fee_quote + sell_fee.total_fee_quote;
    let edge = FeeAdjustedEdge::calculate(gross_profit_quote, total_fees_quote, buy_notional_quote)
        .map_err(OpportunityError::FeeModel)?;

    if edge.net_profit_quote < request.config.min_net_profit_quote
        || edge.roi_bps < request.config.min_roi_bps
    {
        return Ok(None);
    }

    let buy_age_ms = quote_age_ms(buy_quote, request.now_unix_ms)?;
    let sell_age_ms = quote_age_ms(sell_quote, request.now_unix_ms)?;
    let max_quote_age_ms = buy_age_ms.max(sell_age_ms);
    let fees_externally_verified = buy_fee.externally_verified && sell_fee.externally_verified;
    let transfer_risk = find_transfer_risk(
        request,
        &buy_quote.venue,
        &sell_quote.venue,
        &buy_quote.pair,
    );
    let extra_risk_penalty_bps = transfer_risk
        .as_ref()
        .map_or(0.0, |risk| risk.risk_penalty_bps);
    let score = OpportunityScore::calculate(
        edge,
        max_quote_age_ms,
        request.config,
        route_kind,
        fees_externally_verified,
        extra_risk_penalty_bps,
    )?;

    let candidate = OpportunityCandidate {
        id: format!(
            "opp:{}:{}:{}:{}:{}",
            route_kind_label(route_kind),
            buy_quote.pair.symbol(),
            buy_quote.venue.name,
            sell_quote.venue.name,
            request.id
        ),
        route_kind,
        pair: buy_quote.pair.clone(),
        legs: vec![
            OpportunityLeg {
                venue: buy_quote.venue.clone(),
                pair: buy_quote.pair.clone(),
                side: OpportunityLegSide::Buy,
                price_quote: sizing.buy_average_price_quote,
                quantity_base,
                notional_quote: buy_notional_quote,
                fee_estimate: buy_fee,
                source_quote_id: buy_quote.id.clone(),
                market_data_age_ms: buy_age_ms,
            },
            OpportunityLeg {
                venue: sell_quote.venue.clone(),
                pair: sell_quote.pair.clone(),
                side: OpportunityLegSide::Sell,
                price_quote: sizing.sell_average_price_quote,
                quantity_base,
                notional_quote: sell_notional_quote,
                fee_estimate: sell_fee,
                source_quote_id: sell_quote.id.clone(),
                market_data_age_ms: sell_age_ms,
            },
        ],
        edge,
        score,
        liquidity_model: Some(sizing),
        transfer_risk: transfer_risk.clone(),
        discovered_at_unix_ms: request.now_unix_ms,
        source_quote_ids: vec![buy_quote.id.clone(), sell_quote.id.clone()],
        warnings: candidate_warnings(route_kind, fees_externally_verified, transfer_risk.as_ref()),
    };
    candidate.validate()?;
    Ok(Some(candidate))
}

fn discover_triangular_candidates(
    request: &OpportunityDiscoveryRequest,
    candidates: &mut Vec<OpportunityCandidate>,
) -> Result<(), OpportunityError> {
    for first_quote in &request.quotes {
        for second_quote in &request.quotes {
            if first_quote.id == second_quote.id
                || !same_venue(&first_quote.venue, &second_quote.venue)
                || first_quote.venue.kind == VenueKind::Bridge
                || first_quote.pair.base != second_quote.pair.base
                || first_quote.pair.quote == second_quote.pair.quote
            {
                continue;
            }

            for third_quote in &request.quotes {
                if third_quote.id == first_quote.id
                    || third_quote.id == second_quote.id
                    || !same_venue(&first_quote.venue, &third_quote.venue)
                {
                    continue;
                }

                if third_quote.pair.base != second_quote.pair.quote {
                    continue;
                }

                if third_quote.pair.quote != first_quote.pair.quote {
                    continue;
                }

                if let Some(candidate) =
                    build_triangular_candidate(first_quote, second_quote, third_quote, request)?
                {
                    candidates.push(candidate);
                }
            }
        }
    }
    Ok(())
}

fn build_triangular_candidate(
    first_quote: &NormalizedQuote,
    second_quote: &NormalizedQuote,
    third_quote: &NormalizedQuote,
    request: &OpportunityDiscoveryRequest,
) -> Result<Option<OpportunityCandidate>, OpportunityError> {
    let quantity_base = triangular_first_leg_quantity(first_quote, second_quote, third_quote);
    if !is_positive_finite(quantity_base) {
        return Ok(None);
    }

    let first_notional_quote = quantity_base * first_quote.ask.price_quote;
    let second_notional_quote = quantity_base * second_quote.bid.price_quote;
    let third_quantity_base = second_notional_quote;
    let third_notional_quote = third_quantity_base * third_quote.bid.price_quote;
    let gross_profit_quote = third_notional_quote - first_notional_quote;
    if gross_profit_quote <= 0.0 || !gross_profit_quote.is_finite() {
        return Ok(None);
    }

    let first_schedule = find_fee_schedule(
        &request.fee_schedules,
        &first_quote.venue,
        &first_quote.pair,
    )?;
    let second_schedule = find_fee_schedule(
        &request.fee_schedules,
        &second_quote.venue,
        &second_quote.pair,
    )?;
    let third_schedule = find_fee_schedule(
        &request.fee_schedules,
        &third_quote.venue,
        &third_quote.pair,
    )?;
    let first_fee = estimate_fee(first_schedule, first_notional_quote)?;
    let second_fee = estimate_fee(second_schedule, second_notional_quote)?;
    let third_fee = estimate_fee(third_schedule, third_notional_quote)?;
    let second_fee_in_start_quote = second_fee.total_fee_quote * third_quote.bid.price_quote;
    let total_fees_quote =
        first_fee.total_fee_quote + second_fee_in_start_quote + third_fee.total_fee_quote;
    let edge =
        FeeAdjustedEdge::calculate(gross_profit_quote, total_fees_quote, first_notional_quote)
            .map_err(OpportunityError::FeeModel)?;

    if edge.net_profit_quote < request.config.min_net_profit_quote
        || edge.roi_bps < request.config.min_roi_bps
    {
        return Ok(None);
    }

    let first_age_ms = quote_age_ms(first_quote, request.now_unix_ms)?;
    let second_age_ms = quote_age_ms(second_quote, request.now_unix_ms)?;
    let third_age_ms = quote_age_ms(third_quote, request.now_unix_ms)?;
    let max_quote_age_ms = first_age_ms.max(second_age_ms).max(third_age_ms);
    let fees_externally_verified = first_fee.externally_verified
        && second_fee.externally_verified
        && third_fee.externally_verified;
    let score = OpportunityScore::calculate(
        edge,
        max_quote_age_ms,
        request.config,
        OpportunityRouteKind::Triangular,
        fees_externally_verified,
        0.0,
    )?;

    let candidate = OpportunityCandidate {
        id: format!(
            "opp:triangular:{}:{}-{}-{}:{}",
            first_quote.venue.name,
            first_quote.pair.base,
            second_quote.pair.quote,
            first_quote.pair.quote,
            request.id
        ),
        route_kind: OpportunityRouteKind::Triangular,
        pair: first_quote.pair.clone(),
        legs: vec![
            OpportunityLeg {
                venue: first_quote.venue.clone(),
                pair: first_quote.pair.clone(),
                side: OpportunityLegSide::Buy,
                price_quote: first_quote.ask.price_quote,
                quantity_base,
                notional_quote: first_notional_quote,
                fee_estimate: first_fee,
                source_quote_id: first_quote.id.clone(),
                market_data_age_ms: first_age_ms,
            },
            OpportunityLeg {
                venue: second_quote.venue.clone(),
                pair: second_quote.pair.clone(),
                side: OpportunityLegSide::Sell,
                price_quote: second_quote.bid.price_quote,
                quantity_base,
                notional_quote: second_notional_quote,
                fee_estimate: second_fee,
                source_quote_id: second_quote.id.clone(),
                market_data_age_ms: second_age_ms,
            },
            OpportunityLeg {
                venue: third_quote.venue.clone(),
                pair: third_quote.pair.clone(),
                side: OpportunityLegSide::Sell,
                price_quote: third_quote.bid.price_quote,
                quantity_base: third_quantity_base,
                notional_quote: third_notional_quote,
                fee_estimate: third_fee,
                source_quote_id: third_quote.id.clone(),
                market_data_age_ms: third_age_ms,
            },
        ],
        edge,
        score,
        liquidity_model: None,
        transfer_risk: None,
        discovered_at_unix_ms: request.now_unix_ms,
        source_quote_ids: vec![
            first_quote.id.clone(),
            second_quote.id.clone(),
            third_quote.id.clone(),
        ],
        warnings: candidate_warnings(
            OpportunityRouteKind::Triangular,
            fees_externally_verified,
            None,
        ),
    };
    candidate.validate()?;
    Ok(Some(candidate))
}

fn triangular_first_leg_quantity(
    first_quote: &NormalizedQuote,
    second_quote: &NormalizedQuote,
    third_quote: &NormalizedQuote,
) -> f64 {
    first_quote
        .ask
        .quantity_base
        .min(second_quote.bid.quantity_base)
        .min(third_quote.bid.quantity_base / second_quote.bid.price_quote)
}

fn collect_triangular_leg_violations(
    legs: &[OpportunityLeg],
    violations: &mut Vec<OpportunityViolation>,
) {
    if legs.len() != 3 {
        return;
    }

    let first = &legs[0];
    let second = &legs[1];
    let third = &legs[2];

    if first.side != OpportunityLegSide::Buy
        || second.side != OpportunityLegSide::Sell
        || third.side != OpportunityLegSide::Sell
    {
        violations.push(OpportunityViolation::new(
            "OPPORTUNITY_TRIANGULAR_SIDE_SEQUENCE_INVALID",
            "triangular candidates require buy, sell, sell leg ordering",
        ));
    }

    if !same_venue(&first.venue, &second.venue) || !same_venue(&first.venue, &third.venue) {
        violations.push(OpportunityViolation::new(
            "OPPORTUNITY_TRIANGULAR_VENUE_MISMATCH",
            "Phase 27 triangular discovery requires all legs to use one local venue",
        ));
    }

    if first.venue.kind == VenueKind::Bridge {
        violations.push(OpportunityViolation::new(
            "OPPORTUNITY_TRIANGULAR_BRIDGE_UNSUPPORTED",
            "triangular discovery must not use bridge venues",
        ));
    }

    let first_and_second_share_base = first.pair.base == second.pair.base;
    let second_quote_feeds_third_base = second.pair.quote == third.pair.base;
    let third_returns_to_first_quote = third.pair.quote == first.pair.quote;
    if !first_and_second_share_base
        || !second_quote_feeds_third_base
        || !third_returns_to_first_quote
    {
        violations.push(OpportunityViolation::new(
            "OPPORTUNITY_TRIANGULAR_PATH_INVALID",
            "triangular legs must form A/B buy, A/C sell, C/B sell cycle",
        ));
    }
}

fn rank_candidates(
    mut candidates: Vec<OpportunityCandidate>,
    max_candidates: usize,
) -> Result<Vec<OpportunityCandidate>, OpportunityError> {
    for candidate in &candidates {
        candidate.validate()?;
    }

    candidates.sort_by(compare_candidates);
    candidates.truncate(max_candidates);
    Ok(candidates)
}

fn compare_candidates(left: &OpportunityCandidate, right: &OpportunityCandidate) -> Ordering {
    compare_f64_desc(left.score.score_bps, right.score.score_bps)
        .then_with(|| compare_f64_desc(left.edge.net_profit_quote, right.edge.net_profit_quote))
        .then_with(|| compare_f64_desc(left.edge.roi_bps, right.edge.roi_bps))
        .then_with(|| left.id.cmp(&right.id))
}

fn compare_f64_desc(left: f64, right: f64) -> Ordering {
    match right.partial_cmp(&left) {
        Some(ordering) => ordering,
        None => Ordering::Equal,
    }
}

fn find_fee_schedule<'a>(
    schedules: &'a [FeeSchedule],
    venue: &VenueRef,
    pair: &MarketPair,
) -> Result<&'a FeeSchedule, OpportunityError> {
    schedules
        .iter()
        .find(|schedule| {
            same_venue(&schedule.venue, venue)
                && schedule
                    .pair
                    .as_ref()
                    .map_or(true, |schedule_pair| schedule_pair == pair)
        })
        .ok_or_else(|| OpportunityError::ScheduleUnavailable {
            venue: venue.name.clone(),
            pair: pair.symbol(),
        })
}

fn estimate_fee(
    schedule: &FeeSchedule,
    notional_quote: f64,
) -> Result<FeeEstimate, OpportunityError> {
    schedule
        .estimate(notional_quote, LiquidityRole::Taker)
        .map_err(OpportunityError::FeeModel)
}

fn route_kind_for(buy_venue: &VenueRef, sell_venue: &VenueRef) -> Option<OpportunityRouteKind> {
    match (buy_venue.kind, sell_venue.kind) {
        (VenueKind::Cex, VenueKind::Cex) => Some(OpportunityRouteKind::CexCex),
        (VenueKind::Dex | VenueKind::Aggregator, VenueKind::Dex | VenueKind::Aggregator) => {
            Some(OpportunityRouteKind::DexDex)
        }
        (VenueKind::Cex, VenueKind::Dex | VenueKind::Aggregator)
        | (VenueKind::Dex | VenueKind::Aggregator, VenueKind::Cex) => {
            Some(OpportunityRouteKind::CexDex)
        }
        (VenueKind::Bridge, _) | (_, VenueKind::Bridge) => None,
    }
}

fn route_kind_label(route_kind: OpportunityRouteKind) -> &'static str {
    match route_kind {
        OpportunityRouteKind::CexCex => "cex-cex",
        OpportunityRouteKind::DexDex => "dex-dex",
        OpportunityRouteKind::CexDex => "cex-dex",
        OpportunityRouteKind::Triangular => "triangular",
    }
}

fn candidate_warnings(
    route_kind: OpportunityRouteKind,
    fees_externally_verified: bool,
    transfer_risk: Option<&OpportunityTransferRisk>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if !fees_externally_verified {
        warnings.push("fee metadata is not externally verified".to_owned());
    }
    if matches!(
        route_kind,
        OpportunityRouteKind::DexDex | OpportunityRouteKind::CexDex
    ) {
        warnings.push(
            "DEX/Web3 opportunity is discovery-only; signing and broadcasts are unavailable"
                .to_owned(),
        );
    }
    if route_kind == OpportunityRouteKind::Triangular {
        warnings.push(
            "triangular route is discovery-only; no execution or transfers were performed"
                .to_owned(),
        );
    }
    if let Some(transfer_risk) = transfer_risk {
        warnings.push(format!(
            "transfer-risk profile applied from sanitized evidence label {}; no external transfer was performed",
            transfer_risk.evidence_label
        ));
    }
    warnings
}

fn collect_quote_violations(
    quote: &NormalizedQuote,
    now_unix_ms: u64,
    max_age_ms: u64,
    violations: &mut Vec<OpportunityViolation>,
) {
    if let Err(error) = quote.validate() {
        collect_market_data_error("OPPORTUNITY_QUOTE_INVALID", &error, violations);
    }

    match quote.freshness(now_unix_ms, max_age_ms) {
        FreshnessStatus::Fresh { .. } => {}
        FreshnessStatus::Stale { age_ms, max_age_ms } => {
            violations.push(OpportunityViolation::new_owned(
                "OPPORTUNITY_QUOTE_STALE",
                format!(
                    "quote {} is stale: age {age_ms} ms exceeds max {max_age_ms} ms",
                    quote.id
                ),
            ));
        }
        FreshnessStatus::FutureTimestamp { future_by_ms } => {
            violations.push(OpportunityViolation::new_owned(
                "OPPORTUNITY_QUOTE_FUTURE_TIMESTAMP",
                format!("quote {} is {future_by_ms} ms in the future", quote.id),
            ));
        }
    }
}

fn collect_order_book_violations(
    book: &OrderBookSnapshot,
    now_unix_ms: u64,
    max_age_ms: u64,
    violations: &mut Vec<OpportunityViolation>,
) {
    if let Err(error) = book.validate() {
        collect_market_data_error("OPPORTUNITY_ORDER_BOOK_INVALID", &error, violations);
    }

    match book.freshness(now_unix_ms, max_age_ms) {
        FreshnessStatus::Fresh { .. } => {}
        FreshnessStatus::Stale { age_ms, max_age_ms } => {
            violations.push(OpportunityViolation::new_owned(
                "OPPORTUNITY_ORDER_BOOK_STALE",
                format!(
                    "order book {} is stale: age {age_ms} ms exceeds max {max_age_ms} ms",
                    book.id
                ),
            ));
        }
        FreshnessStatus::FutureTimestamp { future_by_ms } => {
            violations.push(OpportunityViolation::new_owned(
                "OPPORTUNITY_ORDER_BOOK_FUTURE_TIMESTAMP",
                format!("order book {} is {future_by_ms} ms in the future", book.id),
            ));
        }
    }
}

fn quote_age_ms(quote: &NormalizedQuote, now_unix_ms: u64) -> Result<u64, OpportunityError> {
    if quote.received_at_unix_ms > now_unix_ms {
        return Err(OpportunityError::ValidationFailed {
            violations: vec![OpportunityViolation::new_owned(
                "OPPORTUNITY_QUOTE_FUTURE_TIMESTAMP",
                format!("quote {} is in the future", quote.id),
            )],
        });
    }
    Ok(now_unix_ms - quote.received_at_unix_ms)
}

fn collect_market_data_error(
    code: &'static str,
    error: &crate::MarketDataError,
    violations: &mut Vec<OpportunityViolation>,
) {
    if error.violations().is_empty() {
        violations.push(OpportunityViolation::new_owned(code, error.to_string()));
        return;
    }

    for violation in error.violations() {
        violations.push(OpportunityViolation::new_owned(
            code,
            format!("{}: {}", violation.code(), violation.message()),
        ));
    }
}

fn collect_fee_model_error(
    code: &'static str,
    error: &FeeModelError,
    violations: &mut Vec<OpportunityViolation>,
) {
    if error.violations().is_empty() {
        violations.push(OpportunityViolation::new_owned(code, error.to_string()));
        return;
    }

    for violation in error.violations() {
        violations.push(OpportunityViolation::new_owned(
            code,
            format!("{}: {}", violation.code(), violation.message()),
        ));
    }
}

fn collect_opportunity_error(error: OpportunityError, violations: &mut Vec<OpportunityViolation>) {
    match error {
        OpportunityError::ValidationFailed {
            violations: nested_violations,
        } => violations.extend(nested_violations),
        OpportunityError::ScheduleUnavailable { venue, pair } => {
            violations.push(OpportunityViolation::new_owned(
                "OPPORTUNITY_FEE_SCHEDULE_UNAVAILABLE",
                format!("missing fee schedule for {venue} {pair}"),
            ));
        }
        OpportunityError::FeeModel(error) => {
            collect_fee_model_error("OPPORTUNITY_FEE_MODEL_ERROR", &error, violations);
        }
    }
}

fn same_venue(left: &VenueRef, right: &VenueRef) -> bool {
    left.kind == right.kind && left.name == right.name
}

fn validate_venue(venue: &VenueRef, violations: &mut Vec<OpportunityViolation>) {
    if venue.name.trim().is_empty() {
        violations.push(OpportunityViolation::new(
            "OPPORTUNITY_VENUE_REQUIRED",
            "venue name must be non-empty",
        ));
    }
}

fn validate_id(label: &'static str, value: &str, violations: &mut Vec<OpportunityViolation>) {
    if value.trim().is_empty() {
        violations.push(OpportunityViolation::new_owned(
            "OPPORTUNITY_ID_REQUIRED",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn finish_validation(violations: Vec<OpportunityViolation>) -> Result<(), OpportunityError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(OpportunityError::ValidationFailed { violations })
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn is_non_negative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

/// One opportunity-engine validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpportunityViolation {
    code: &'static str,
    message: String,
}

impl OpportunityViolation {
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

/// Opportunity-engine boundary errors.
#[derive(Debug, Clone, PartialEq)]
pub enum OpportunityError {
    /// Validation failed with deterministic violations.
    ValidationFailed {
        violations: Vec<OpportunityViolation>,
    },
    /// Fee schedule was unavailable for a candidate leg.
    ScheduleUnavailable { venue: String, pair: String },
    /// Fee model failed while calculating a candidate.
    FeeModel(FeeModelError),
}

impl OpportunityError {
    /// Return validation violations, if present.
    #[must_use]
    pub fn violations(&self) -> &[OpportunityViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::ScheduleUnavailable { .. } | Self::FeeModel(_) => &[],
        }
    }
}

impl fmt::Display for OpportunityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "opportunity validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::ScheduleUnavailable { venue, pair } => {
                write!(
                    formatter,
                    "fee schedule unavailable for venue {venue} pair {pair}"
                )
            }
            Self::FeeModel(error) => write!(
                formatter,
                "fee model error during opportunity discovery: {error}"
            ),
        }
    }
}

impl Error for OpportunityError {}

#[cfg(test)]
mod tests {
    use super::{
        DeterministicOpportunityEngine, OpportunityDiscoveryConfig, OpportunityDiscoveryRequest,
        OpportunityEngine, OpportunityInventoryLimit, OpportunityRouteKind,
        OpportunityTransferRiskProfile,
    };
    use crate::{
        FeeSchedule, MarketPair, NormalizedQuote, OrderBookSnapshot, PriceLevel, VenueKind,
        VenueRef,
    };

    #[test]
    fn discovers_fee_adjusted_cex_cex_candidate() {
        let pair = MarketPair::new("BTC", "USD").expect("pair should validate");
        let request = OpportunityDiscoveryRequest {
            id: "req-1".to_owned(),
            quotes: vec![
                quote("buy-quote", "paper-a", pair.clone(), 99.0, 100.0, 2.0),
                quote("sell-quote", "paper-b", pair.clone(), 105.0, 106.0, 1.0),
            ],
            fee_schedules: vec![fee("paper-a", pair.clone()), fee("paper-b", pair)],
            order_books: Vec::new(),
            inventory_limits: Vec::new(),
            transfer_risk_profiles: Vec::new(),
            config: OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        };

        let candidates = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect("discovery should succeed");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].route_kind, OpportunityRouteKind::CexCex);
        assert!(candidates[0].edge.net_profit_quote > 0.0);
    }

    #[test]
    fn rejects_stale_market_data() {
        let pair = MarketPair::new("ETH", "USD").expect("pair should validate");
        let request = OpportunityDiscoveryRequest {
            id: "req-stale".to_owned(),
            quotes: vec![
                quote_with_time(
                    "buy-quote",
                    "paper-a",
                    pair.clone(),
                    99.0,
                    100.0,
                    1.0,
                    1_000,
                ),
                quote_with_time(
                    "sell-quote",
                    "paper-b",
                    pair.clone(),
                    105.0,
                    106.0,
                    1.0,
                    1_000,
                ),
            ],
            fee_schedules: vec![fee("paper-a", pair.clone()), fee("paper-b", pair)],
            order_books: Vec::new(),
            inventory_limits: Vec::new(),
            transfer_risk_profiles: Vec::new(),
            config: OpportunityDiscoveryConfig {
                max_market_data_age_ms: 10,
                ..OpportunityDiscoveryConfig::default()
            },
            now_unix_ms: 10_000,
        };

        let error = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect_err("stale market data must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "OPPORTUNITY_QUOTE_STALE"));
    }

    #[test]
    fn ranks_by_score_then_profit_deterministically() {
        let pair = MarketPair::new("SOL", "USD").expect("pair should validate");
        let request = OpportunityDiscoveryRequest {
            id: "req-rank".to_owned(),
            quotes: vec![
                quote("buy-a", "paper-a", pair.clone(), 99.0, 100.0, 1.0),
                quote("sell-b", "paper-b", pair.clone(), 110.0, 111.0, 1.0),
                quote("sell-c", "paper-c", pair.clone(), 108.0, 109.0, 1.0),
            ],
            fee_schedules: vec![
                fee("paper-a", pair.clone()),
                fee("paper-b", pair.clone()),
                fee("paper-c", pair),
            ],
            order_books: Vec::new(),
            inventory_limits: Vec::new(),
            transfer_risk_profiles: Vec::new(),
            config: OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        };

        let candidates = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect("discovery should succeed");
        assert!(candidates.len() >= 2);
        assert!(candidates[0].edge.net_profit_quote >= candidates[1].edge.net_profit_quote);
    }

    #[test]
    fn applies_depth_and_inventory_caps_to_cross_venue_candidate() {
        let pair = MarketPair::new("BTC", "USD").expect("pair should validate");
        let request = OpportunityDiscoveryRequest {
            id: "req-depth".to_owned(),
            quotes: vec![
                quote("buy-quote", "paper-a", pair.clone(), 99.0, 100.0, 3.0),
                quote("sell-quote", "paper-b", pair.clone(), 108.0, 109.0, 3.0),
            ],
            fee_schedules: vec![fee("paper-a", pair.clone()), fee("paper-b", pair.clone())],
            order_books: vec![
                book(
                    "book-a",
                    "paper-a",
                    pair.clone(),
                    vec![(99.0, 3.0)],
                    vec![(100.0, 1.0), (102.0, 2.0)],
                ),
                book(
                    "book-b",
                    "paper-b",
                    pair.clone(),
                    vec![(108.0, 1.0), (106.0, 2.0)],
                    vec![(109.0, 3.0)],
                ),
            ],
            inventory_limits: vec![OpportunityInventoryLimit {
                venue: venue("paper-b"),
                pair,
                available_base: 1.5,
                available_quote: 0.0,
                max_fraction: Some(1.0),
            }],
            transfer_risk_profiles: Vec::new(),
            config: OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        };

        let candidates = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect("discovery should succeed");
        let liquidity = candidates[0]
            .liquidity_model
            .as_ref()
            .expect("liquidity model should be attached");

        assert!(liquidity.depth_aware);
        assert!(liquidity.inventory_capped);
        assert!((liquidity.executable_quantity_base - 1.5).abs() < 0.000_000_01);
        assert!(liquidity.buy_slippage_bps > 0.0);
        assert!(liquidity.sell_slippage_bps > 0.0);
    }

    #[test]
    fn applies_transfer_risk_penalty_without_external_calls() {
        let pair = MarketPair::new("ETH", "USD").expect("pair should validate");
        let request = OpportunityDiscoveryRequest {
            id: "req-transfer".to_owned(),
            quotes: vec![
                quote("buy-quote", "paper-a", pair.clone(), 99.0, 100.0, 1.0),
                quote("sell-quote", "paper-b", pair.clone(), 110.0, 111.0, 1.0),
            ],
            fee_schedules: vec![fee("paper-a", pair.clone()), fee("paper-b", pair.clone())],
            order_books: Vec::new(),
            inventory_limits: Vec::new(),
            transfer_risk_profiles: vec![OpportunityTransferRiskProfile {
                from_venue: venue("paper-a"),
                to_venue: venue("paper-b"),
                pair,
                estimated_latency_ms: 15_000,
                risk_penalty_bps: 25.0,
                evidence_label: "local-paper-transfer-assumption".to_owned(),
            }],
            config: OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        };

        let candidates = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect("discovery should succeed");

        let risk_penalty_bps = candidates[0]
            .transfer_risk
            .as_ref()
            .expect("transfer risk should be attached")
            .risk_penalty_bps;
        assert!((risk_penalty_bps - 25.0).abs() < 0.000_000_01);
        assert!(candidates[0].score.risk_penalty_bps >= 30.0);
        assert!(candidates[0]
            .warnings
            .iter()
            .any(|warning| warning.contains("no external transfer was performed")));
    }

    #[test]
    fn discovers_same_venue_triangular_candidate_without_external_calls() {
        let btc_usd = MarketPair::new("BTC", "USD").expect("pair should validate");
        let btc_eth = MarketPair::new("BTC", "ETH").expect("pair should validate");
        let eth_usd = MarketPair::new("ETH", "USD").expect("pair should validate");
        let request = OpportunityDiscoveryRequest {
            id: "req-triangular".to_owned(),
            quotes: vec![
                quote("btc-usd", "paper-a", btc_usd.clone(), 99.0, 100.0, 1.0),
                quote("btc-eth", "paper-a", btc_eth.clone(), 3.0, 3.1, 1.0),
                quote("eth-usd", "paper-a", eth_usd.clone(), 40.0, 41.0, 3.0),
            ],
            fee_schedules: vec![
                fee("paper-a", btc_usd),
                fee("paper-a", btc_eth),
                fee("paper-a", eth_usd),
            ],
            order_books: Vec::new(),
            inventory_limits: Vec::new(),
            transfer_risk_profiles: Vec::new(),
            config: OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        };

        let candidates = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect("discovery should succeed");
        let triangular = candidates
            .iter()
            .find(|candidate| candidate.route_kind == OpportunityRouteKind::Triangular)
            .expect("triangular candidate should be discovered");

        assert_eq!(triangular.legs.len(), 3);
        assert_eq!(triangular.source_quote_ids.len(), 3);
        assert!(triangular.edge.net_profit_quote > 0.0);
        assert!(triangular
            .warnings
            .iter()
            .any(|warning| warning.contains("triangular route is discovery-only")));
    }

    fn quote(
        id: &str,
        venue_name: &str,
        pair: MarketPair,
        bid_price: f64,
        ask_price: f64,
        quantity_base: f64,
    ) -> NormalizedQuote {
        quote_with_time(
            id,
            venue_name,
            pair,
            bid_price,
            ask_price,
            quantity_base,
            9_500,
        )
    }

    fn quote_with_time(
        id: &str,
        venue_name: &str,
        pair: MarketPair,
        bid_price: f64,
        ask_price: f64,
        quantity_base: f64,
        received_at_unix_ms: u64,
    ) -> NormalizedQuote {
        NormalizedQuote {
            id: id.to_owned(),
            venue: VenueRef {
                kind: VenueKind::Cex,
                name: venue_name.to_owned(),
            },
            pair,
            bid: PriceLevel {
                price_quote: bid_price,
                quantity_base,
            },
            ask: PriceLevel {
                price_quote: ask_price,
                quantity_base,
            },
            captured_at_unix_ms: received_at_unix_ms,
            received_at_unix_ms,
        }
    }

    fn book(
        id: &str,
        venue_name: &str,
        pair: MarketPair,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    ) -> OrderBookSnapshot {
        OrderBookSnapshot {
            id: id.to_owned(),
            venue: venue(venue_name),
            pair,
            captured_at_unix_ms: 9_500,
            received_at_unix_ms: 9_500,
            bids: bids
                .into_iter()
                .map(|(price_quote, quantity_base)| PriceLevel {
                    price_quote,
                    quantity_base,
                })
                .collect(),
            asks: asks
                .into_iter()
                .map(|(price_quote, quantity_base)| PriceLevel {
                    price_quote,
                    quantity_base,
                })
                .collect(),
            source_sequence: None,
        }
    }

    fn fee(venue_name: &str, pair: MarketPair) -> FeeSchedule {
        FeeSchedule {
            venue: venue(venue_name),
            pair: Some(pair),
            maker_bps: 5.0,
            taker_bps: 10.0,
            network_fee_quote: 0.0,
            externally_verified: false,
        }
    }

    fn venue(venue_name: &str) -> VenueRef {
        VenueRef {
            kind: VenueKind::Cex,
            name: venue_name.to_owned(),
        }
    }
}
