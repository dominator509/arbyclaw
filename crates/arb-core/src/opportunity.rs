#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    FeeAdjustedEdge, FeeEstimate, FeeModelError, FeeSchedule, FreshnessStatus, LiquidityRole,
    MarketPair, NormalizedQuote, VenueKind, VenueRef, DEFAULT_MARKET_DATA_FRESHNESS_MS,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, error::Error, fmt};

/// Stable opportunity-engine version for audit and replay surfaces.
pub const OPPORTUNITY_ENGINE_VERSION: &str = "phase-9-opportunity-engine-v1";

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
        let risk_penalty_bps = route_penalty_bps + fee_penalty_bps;
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

            if leg.pair != self.pair {
                violations.push(OpportunityViolation::new(
                    "OPPORTUNITY_LEG_PAIR_MISMATCH",
                    "each leg pair must match the candidate pair for Phase 9 cross-venue records",
                ));
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

        rank_candidates(candidates, request.config.max_candidates)
    }
}

fn build_cross_venue_candidate(
    buy_quote: &NormalizedQuote,
    sell_quote: &NormalizedQuote,
    route_kind: OpportunityRouteKind,
    request: &OpportunityDiscoveryRequest,
) -> Result<Option<OpportunityCandidate>, OpportunityError> {
    let quantity_base = buy_quote
        .ask
        .quantity_base
        .min(sell_quote.bid.quantity_base);
    if !is_positive_finite(quantity_base) {
        return Ok(None);
    }

    let buy_notional_quote = buy_quote.ask.price_quote * quantity_base;
    let sell_notional_quote = sell_quote.bid.price_quote * quantity_base;
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
    let score = OpportunityScore::calculate(
        edge,
        max_quote_age_ms,
        request.config,
        route_kind,
        fees_externally_verified,
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
                price_quote: buy_quote.ask.price_quote,
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
                price_quote: sell_quote.bid.price_quote,
                quantity_base,
                notional_quote: sell_notional_quote,
                fee_estimate: sell_fee,
                source_quote_id: sell_quote.id.clone(),
                market_data_age_ms: sell_age_ms,
            },
        ],
        edge,
        score,
        discovered_at_unix_ms: request.now_unix_ms,
        source_quote_ids: vec![buy_quote.id.clone(), sell_quote.id.clone()],
        warnings: candidate_warnings(route_kind, fees_externally_verified),
    };
    candidate.validate()?;
    Ok(Some(candidate))
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
        OpportunityEngine, OpportunityRouteKind,
    };
    use crate::{FeeSchedule, MarketPair, NormalizedQuote, PriceLevel, VenueKind, VenueRef};

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
            config: OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        };

        let candidates = DeterministicOpportunityEngine::new()
            .discover(&request)
            .expect("discovery should succeed");
        assert!(candidates.len() >= 2);
        assert!(candidates[0].edge.net_profit_quote >= candidates[1].edge.net_profit_quote);
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

    fn fee(venue_name: &str, pair: MarketPair) -> FeeSchedule {
        FeeSchedule {
            venue: VenueRef {
                kind: VenueKind::Cex,
                name: venue_name.to_owned(),
            },
            pair: Some(pair),
            maker_bps: 5.0,
            taker_bps: 10.0,
            network_fee_quote: 0.0,
            externally_verified: false,
        }
    }
}
