#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{MarketPair, VenueRef};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable fee model version for audit and replay surfaces.
pub const FEE_MODEL_VERSION: &str = "phase-5-fees-v1";

/// Liquidity role used for CEX fee estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiquidityRole {
    /// Maker/post-only style fee tier.
    Maker,
    /// Taker/immediate execution fee tier.
    Taker,
}

/// Normalized fee schedule for one venue/pair context.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeSchedule {
    /// Venue that owns this fee schedule.
    pub venue: VenueRef,
    /// Optional pair-specific fee scope.
    pub pair: Option<MarketPair>,
    /// Maker fee in basis points.
    pub maker_bps: f64,
    /// Taker fee in basis points.
    pub taker_bps: f64,
    /// Optional network/gas estimate in quote units.
    pub network_fee_quote: f64,
    /// Whether the fee was externally verified against the live venue account.
    pub externally_verified: bool,
}

impl FeeSchedule {
    /// Validate fee schedule safety bounds.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        let mut violations = Vec::new();

        if self.venue.name.trim().is_empty() {
            violations.push(FeeModelViolation::new(
                "FEE_VENUE_REQUIRED",
                "fee schedule venue name must be non-empty",
            ));
        }

        if let Some(pair) = &self.pair {
            if let Err(error) = pair.validate() {
                for violation in error.violations() {
                    violations.push(FeeModelViolation::new_owned(
                        "FEE_PAIR_INVALID",
                        format!("{}: {}", violation.code(), violation.message()),
                    ));
                }
            }
        }

        validate_bps("maker", self.maker_bps, &mut violations);
        validate_bps("taker", self.taker_bps, &mut violations);

        if !is_non_negative_finite(self.network_fee_quote) {
            violations.push(FeeModelViolation::new(
                "NETWORK_FEE_INVALID",
                "network_fee_quote must be finite and non-negative",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }

    /// Estimate execution fees for a notional quote amount.
    pub fn estimate(
        &self,
        notional_quote: f64,
        role: LiquidityRole,
    ) -> Result<FeeEstimate, FeeModelError> {
        self.validate()?;

        if !is_positive_finite(notional_quote) {
            return Err(FeeModelError::ValidationFailed {
                violations: vec![FeeModelViolation::new(
                    "FEE_NOTIONAL_INVALID",
                    "fee notional must be positive and finite",
                )],
            });
        }

        let fee_bps = match role {
            LiquidityRole::Maker => self.maker_bps,
            LiquidityRole::Taker => self.taker_bps,
        };
        let venue_fee_quote = notional_quote * (fee_bps / 10_000.0);
        let total_fee_quote = venue_fee_quote + self.network_fee_quote;

        Ok(FeeEstimate {
            venue: self.venue.clone(),
            pair: self.pair.clone(),
            notional_quote,
            liquidity_role: role,
            fee_bps,
            venue_fee_quote,
            network_fee_quote: self.network_fee_quote,
            total_fee_quote,
            externally_verified: self.externally_verified,
        })
    }
}

/// Deterministic fee estimate for one proposed execution leg.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeEstimate {
    /// Venue associated with the estimate.
    pub venue: VenueRef,
    /// Optional pair associated with the estimate.
    pub pair: Option<MarketPair>,
    /// Notional quote amount used for the estimate.
    pub notional_quote: f64,
    /// Liquidity role used to select fee basis points.
    pub liquidity_role: LiquidityRole,
    /// Effective venue fee in basis points.
    pub fee_bps: f64,
    /// Venue trading fee in quote units.
    pub venue_fee_quote: f64,
    /// Gas/network fee in quote units.
    pub network_fee_quote: f64,
    /// Total fee in quote units.
    pub total_fee_quote: f64,
    /// Whether source schedule was externally verified.
    pub externally_verified: bool,
}

impl FeeEstimate {
    /// Validate the estimate before policy or opportunity code uses it.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        let mut violations = Vec::new();

        if self.venue.name.trim().is_empty() {
            violations.push(FeeModelViolation::new(
                "FEE_ESTIMATE_VENUE_REQUIRED",
                "fee estimate venue name must be non-empty",
            ));
        }

        if !is_positive_finite(self.notional_quote) {
            violations.push(FeeModelViolation::new(
                "FEE_ESTIMATE_NOTIONAL_INVALID",
                "fee estimate notional must be positive and finite",
            ));
        }

        validate_bps("effective", self.fee_bps, &mut violations);

        for (label, value) in [
            ("venue", self.venue_fee_quote),
            ("network", self.network_fee_quote),
            ("total", self.total_fee_quote),
        ] {
            if !is_non_negative_finite(value) {
                violations.push(FeeModelViolation::new_owned(
                    "FEE_ESTIMATE_AMOUNT_INVALID",
                    format!("{label} fee amount must be finite and non-negative"),
                ));
            }
        }

        let recomputed_total = self.venue_fee_quote + self.network_fee_quote;
        if (recomputed_total - self.total_fee_quote).abs() > 0.000_000_01 {
            violations.push(FeeModelViolation::new(
                "FEE_ESTIMATE_TOTAL_MISMATCH",
                "total_fee_quote must equal venue_fee_quote plus network_fee_quote",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }
}

/// Fee-adjusted edge for a potential arbitrage route.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeAdjustedEdge {
    /// Gross profit before fees in quote units.
    pub gross_profit_quote: f64,
    /// Total known fees in quote units.
    pub total_fees_quote: f64,
    /// Net profit after known fees in quote units.
    pub net_profit_quote: f64,
    /// Net return on notional in basis points.
    pub roi_bps: f64,
}

impl FeeAdjustedEdge {
    /// Calculate net edge from a gross profit, total fees, and notional.
    pub fn calculate(
        gross_profit_quote: f64,
        total_fees_quote: f64,
        notional_quote: f64,
    ) -> Result<Self, FeeModelError> {
        let mut violations = Vec::new();

        if !is_non_negative_finite(gross_profit_quote) {
            violations.push(FeeModelViolation::new(
                "GROSS_PROFIT_INVALID",
                "gross profit must be finite and non-negative",
            ));
        }

        if !is_non_negative_finite(total_fees_quote) {
            violations.push(FeeModelViolation::new(
                "TOTAL_FEES_INVALID",
                "total fees must be finite and non-negative",
            ));
        }

        if !is_positive_finite(notional_quote) {
            violations.push(FeeModelViolation::new(
                "ROI_NOTIONAL_INVALID",
                "ROI notional must be positive and finite",
            ));
        }

        if !violations.is_empty() {
            return Err(FeeModelError::ValidationFailed { violations });
        }

        let net_profit_quote = gross_profit_quote - total_fees_quote;
        let roi_bps = (net_profit_quote / notional_quote) * 10_000.0;

        Ok(Self {
            gross_profit_quote,
            total_fees_quote,
            net_profit_quote,
            roi_bps,
        })
    }

    /// Return true only when the edge is profitable after known fees.
    #[must_use]
    pub fn is_profitable(self) -> bool {
        self.net_profit_quote.is_finite() && self.net_profit_quote > 0.0
    }
}

/// Boundary trait for future fee providers.
///
/// Implementations must not trade, sign, withdraw, or mutate exchange state.
pub trait FeeProvider {
    /// Stable provider name for diagnostics and audit records.
    fn provider_name(&self) -> &str;

    /// Return the best-known schedule for a venue and optional pair.
    fn fee_schedule(
        &self,
        venue: &VenueRef,
        pair: Option<&MarketPair>,
    ) -> Result<FeeSchedule, FeeModelError>;
}

/// One fee-model validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeModelViolation {
    code: &'static str,
    message: String,
}

impl FeeModelViolation {
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

/// Fee model errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeeModelError {
    /// Validation failed with deterministic violations.
    ValidationFailed { violations: Vec<FeeModelViolation> },
    /// Provider returned no schedule for the requested venue/pair.
    ScheduleUnavailable { provider: String, reason: String },
}

impl FeeModelError {
    /// Return violations, if this is a validation error.
    #[must_use]
    pub fn violations(&self) -> &[FeeModelViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::ScheduleUnavailable { .. } => &[],
        }
    }
}

impl fmt::Display for FeeModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "fee model validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::ScheduleUnavailable { provider, reason } => {
                write!(
                    formatter,
                    "fee provider {provider} has no available schedule: {reason}"
                )
            }
        }
    }
}

impl Error for FeeModelError {}

fn validate_bps(label: &'static str, value: f64, violations: &mut Vec<FeeModelViolation>) {
    if !is_non_negative_finite(value) {
        violations.push(FeeModelViolation::new_owned(
            "FEE_BPS_INVALID",
            format!("{label} fee bps must be finite and non-negative"),
        ));
        return;
    }

    if value > 1_000.0 {
        violations.push(FeeModelViolation::new_owned(
            "FEE_BPS_EXCESSIVE",
            format!("{label} fee bps exceeds Phase 5 safety ceiling of 1000 bps"),
        ));
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn is_non_negative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::{FeeAdjustedEdge, FeeSchedule, LiquidityRole};
    use crate::{MarketPair, VenueKind, VenueRef};

    #[test]
    fn fee_schedule_estimates_taker_fee() {
        let schedule = FeeSchedule {
            venue: VenueRef {
                kind: VenueKind::Cex,
                name: "paper-coinbase".to_owned(),
            },
            pair: Some(MarketPair::new("BTC", "USD").expect("pair should validate")),
            maker_bps: 2.0,
            taker_bps: 6.0,
            network_fee_quote: 0.50,
            externally_verified: false,
        };

        let estimate = schedule
            .estimate(1_000.0, LiquidityRole::Taker)
            .expect("fee estimate should validate");
        assert!((estimate.venue_fee_quote - 0.6).abs() < f64::EPSILON);
        assert!((estimate.total_fee_quote - 1.1).abs() < f64::EPSILON);
    }

    #[test]
    fn fee_adjusted_edge_requires_positive_net_profit() {
        let edge = FeeAdjustedEdge::calculate(10.0, 2.5, 100.0).expect("edge should calculate");
        assert!(edge.is_profitable());
        assert!((edge.net_profit_quote - 7.5).abs() < f64::EPSILON);
    }
}
