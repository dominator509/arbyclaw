#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{ExecutionIntent, ExecutionIntentKind, ExecutionScope, RuntimeMode, VenueKind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable strategy profile boundary version.
pub const STRATEGY_PROFILE_VERSION: &str = "phase-2-9-10-strategy-profile-v1";

/// Typed local strategy profile.
///
/// Profiles constrain candidate intents only. They do not execute, sign,
/// broadcast, withdraw, bridge, call exchanges/RPC providers, or bypass policy.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProfile {
    /// Stable non-secret profile identifier.
    pub id: String,
    /// Boundary version that produced this profile.
    pub strategy_profile_version: String,
    /// Maximum runtime scope this strategy may request.
    pub mode: RuntimeMode,
    /// Capital allocation parameters.
    pub capital: StrategyCapitalParameters,
    /// Risk ceiling parameters.
    pub risk: StrategyRiskParameters,
    /// Opportunity acceptance parameters.
    pub opportunity: StrategyOpportunityParameters,
    /// Execution-shape parameters.
    pub execution: StrategyExecutionParameters,
    /// Venue, chain, router, and asset allowlists.
    pub venues: StrategyVenueParameters,
    /// Operator notification preferences.
    pub alerts: StrategyAlertParameters,
}

impl StrategyProfile {
    /// Build a conservative paper-mode strategy profile.
    #[must_use]
    pub fn conservative_paper(id: impl Into<String>, base_asset: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            strategy_profile_version: STRATEGY_PROFILE_VERSION.to_owned(),
            mode: RuntimeMode::Paper,
            capital: StrategyCapitalParameters {
                base_asset: base_asset.into(),
                max_total_deployed: 10_000.0,
                max_per_opportunity: 1_000.0,
                reserve_minimum: 1_000.0,
                compound_enabled: false,
                compound_rate: 0.0,
            },
            risk: StrategyRiskParameters {
                max_daily_loss: 100.0,
                max_daily_drawdown_pct: 5.0,
                max_open_exposure_pct: 25.0,
                max_venue_exposure_pct: 20.0,
                max_chain_exposure_pct: 10.0,
                max_asset_exposure_pct: 30.0,
                max_single_tx_value: 1_000.0,
                consecutive_failure_limit: 3,
            },
            opportunity: StrategyOpportunityParameters {
                min_net_profit_abs: 1.0,
                min_net_profit_pct: 0.01,
                min_roi_after_fees_pct: 0.01,
                max_quote_age_ms: 5_000,
                min_liquidity_depth: 1.0,
                min_confidence_score: 0.0,
            },
            execution: StrategyExecutionParameters {
                max_slippage_bps: 50,
                max_gas_native: 0.0,
                max_gas_quote: 25.0,
                order_timeout_ms: 30_000,
                cancel_on_partial_fill: true,
                allow_market_orders: false,
                allow_limit_orders: true,
                allow_ioc: true,
                allow_fok: false,
                allow_flashloans: false,
                allow_bridges: false,
                allow_withdrawals: false,
                allowed_destination_labels: Vec::new(),
            },
            venues: StrategyVenueParameters {
                allowed_exchanges: Vec::new(),
                allowed_chains: Vec::new(),
                allowed_routers: Vec::new(),
                allowed_assets: Vec::new(),
            },
            alerts: StrategyAlertParameters::default(),
        }
    }

    /// Validate the profile shape and fail-closed safety constraints.
    pub fn validate(&self) -> Result<(), StrategyProfileError> {
        let mut violations = Vec::new();

        validate_id("strategy profile", &self.id, &mut violations);
        if self.strategy_profile_version != STRATEGY_PROFILE_VERSION {
            violations.push(StrategyProfileViolation::new_owned(
                "STRATEGY_PROFILE_VERSION_MISMATCH",
                format!(
                    "strategy profile version {} does not match {}",
                    self.strategy_profile_version, STRATEGY_PROFILE_VERSION
                ),
            ));
        }

        self.capital.validate(&mut violations);
        self.risk.validate(&mut violations);
        self.opportunity.validate(&mut violations);
        self.execution.validate(&mut violations);
        self.venues.validate(&mut violations);

        if self.mode == RuntimeMode::LiveArmed {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_LIVE_MODE_DENIED",
                "strategy profiles cannot enable live-armed scope in this boundary",
            ));
        }

        finish_validation(violations)
    }

    /// Evaluate whether one candidate intent stays inside this profile.
    ///
    /// This is a local constraint report only. It does not approve execution and
    /// does not replace the policy engine.
    #[must_use]
    pub fn constrain_intent(&self, intent: &ExecutionIntent) -> StrategyPolicyConstraintReport {
        let mut violations = Vec::new();

        if let Err(error) = self.validate() {
            violations.extend(error.violations().iter().cloned());
        }

        if intent.strategy_id != self.id {
            violations.push(StrategyProfileViolation::new_owned(
                "STRATEGY_INTENT_PROFILE_MISMATCH",
                format!(
                    "intent strategy id {} does not match profile {}",
                    intent.strategy_id, self.id
                ),
            ));
        }

        if !scope_allowed(self.mode, intent.scope) {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_INTENT_SCOPE_DENIED",
                "intent scope exceeds the strategy profile mode",
            ));
        }

        if intent.kind == ExecutionIntentKind::Withdrawal || self.execution.allow_withdrawals {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_WITHDRAWALS_DENIED",
                "strategy profiles cannot enable withdrawals in this boundary",
            ));
        }

        if intent.kind == ExecutionIntentKind::BridgeRoute || self.execution.allow_bridges {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_BRIDGES_DENIED",
                "strategy profiles cannot enable bridges in this boundary",
            ));
        }

        if self.execution.allow_flashloans {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_FLASHLOANS_DENIED",
                "strategy profiles cannot enable flashloans in this boundary",
            ));
        }

        if !is_positive_finite(intent.notional_quote) {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_INTENT_NOTIONAL_INVALID",
                "intent notional must be positive and finite",
            ));
        } else {
            if intent.notional_quote > self.capital.max_per_opportunity {
                violations.push(StrategyProfileViolation::new_owned(
                    "STRATEGY_MAX_PER_OPPORTUNITY_EXCEEDED",
                    format!(
                        "intent notional {} exceeds profile max_per_opportunity {}",
                        intent.notional_quote, self.capital.max_per_opportunity
                    ),
                ));
            }
            if intent.notional_quote > self.risk.max_single_tx_value {
                violations.push(StrategyProfileViolation::new_owned(
                    "STRATEGY_MAX_SINGLE_TX_EXCEEDED",
                    format!(
                        "intent notional {} exceeds profile max_single_tx_value {}",
                        intent.notional_quote, self.risk.max_single_tx_value
                    ),
                ));
            }
        }

        let net_profit =
            intent.expected_profit_quote - intent.estimated_fee_quote - intent.gas_fee_quote;
        if !is_positive_finite(net_profit) || net_profit < self.opportunity.min_net_profit_abs {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_MIN_NET_PROFIT_NOT_MET",
                "intent net profit does not meet the strategy absolute threshold",
            ));
        }

        if is_positive_finite(intent.notional_quote) {
            let roi_pct = (net_profit / intent.notional_quote) * 100.0;
            if !roi_pct.is_finite() || roi_pct < self.opportunity.min_roi_after_fees_pct {
                violations.push(StrategyProfileViolation::new(
                    "STRATEGY_MIN_ROI_NOT_MET",
                    "intent ROI after fees does not meet the strategy threshold",
                ));
            }
        }

        if intent.max_loss_quote > self.risk.max_daily_loss {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_MAX_DAILY_LOSS_EXCEEDED",
                "intent max loss exceeds the strategy daily loss cap",
            ));
        }

        if intent.slippage_bps > self.execution.max_slippage_bps {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_SLIPPAGE_EXCEEDED",
                "intent slippage exceeds the strategy cap",
            ));
        }

        if intent.gas_fee_quote > self.execution.max_gas_quote {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_GAS_QUOTE_EXCEEDED",
                "intent gas/network fee exceeds the strategy quote cap",
            ));
        }

        if intent.market_data_age_ms > self.opportunity.max_quote_age_ms {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_QUOTE_AGE_EXCEEDED",
                "intent market data age exceeds the strategy freshness cap",
            ));
        }

        self.venues
            .validate_intent_allowlists(intent, &mut violations);

        let status = if violations.is_empty() {
            StrategyPolicyConstraintStatus::Satisfied
        } else {
            StrategyPolicyConstraintStatus::Rejected
        };

        StrategyPolicyConstraintReport {
            strategy_profile_version: STRATEGY_PROFILE_VERSION.to_owned(),
            strategy_id: self.id.clone(),
            intent_id: intent.id.clone(),
            status,
            violations,
            execution_performed: false,
            signing_or_broadcast_performed: false,
            live_network_used: false,
        }
    }
}

/// Strategy capital allocation parameters.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyCapitalParameters {
    pub base_asset: String,
    pub max_total_deployed: f64,
    pub max_per_opportunity: f64,
    pub reserve_minimum: f64,
    pub compound_enabled: bool,
    pub compound_rate: f64,
}

impl StrategyCapitalParameters {
    fn validate(&self, violations: &mut Vec<StrategyProfileViolation>) {
        validate_symbol("capital base asset", &self.base_asset, violations);
        validate_positive(
            "capital.max_total_deployed",
            self.max_total_deployed,
            violations,
        );
        validate_positive(
            "capital.max_per_opportunity",
            self.max_per_opportunity,
            violations,
        );
        validate_non_negative("capital.reserve_minimum", self.reserve_minimum, violations);
        validate_non_negative("capital.compound_rate", self.compound_rate, violations);
        if self.compound_rate > 1.0 {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_COMPOUND_RATE_INVALID",
                "capital.compound_rate must not exceed 1.0",
            ));
        }
        if self.max_per_opportunity > self.max_total_deployed {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_MAX_PER_OPPORTUNITY_GT_TOTAL",
                "capital.max_per_opportunity must not exceed max_total_deployed",
            ));
        }
    }
}

/// Strategy risk ceiling parameters.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyRiskParameters {
    pub max_daily_loss: f64,
    pub max_daily_drawdown_pct: f64,
    pub max_open_exposure_pct: f64,
    pub max_venue_exposure_pct: f64,
    pub max_chain_exposure_pct: f64,
    pub max_asset_exposure_pct: f64,
    pub max_single_tx_value: f64,
    pub consecutive_failure_limit: u32,
}

impl StrategyRiskParameters {
    fn validate(&self, violations: &mut Vec<StrategyProfileViolation>) {
        validate_non_negative("risk.max_daily_loss", self.max_daily_loss, violations);
        validate_percent(
            "risk.max_daily_drawdown_pct",
            self.max_daily_drawdown_pct,
            violations,
        );
        validate_percent(
            "risk.max_open_exposure_pct",
            self.max_open_exposure_pct,
            violations,
        );
        validate_percent(
            "risk.max_venue_exposure_pct",
            self.max_venue_exposure_pct,
            violations,
        );
        validate_percent(
            "risk.max_chain_exposure_pct",
            self.max_chain_exposure_pct,
            violations,
        );
        validate_percent(
            "risk.max_asset_exposure_pct",
            self.max_asset_exposure_pct,
            violations,
        );
        validate_positive(
            "risk.max_single_tx_value",
            self.max_single_tx_value,
            violations,
        );
        if self.consecutive_failure_limit == 0 {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_FAILURE_LIMIT_ZERO",
                "risk.consecutive_failure_limit must be positive",
            ));
        }
    }
}

/// Strategy opportunity acceptance parameters.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyOpportunityParameters {
    pub min_net_profit_abs: f64,
    pub min_net_profit_pct: f64,
    pub min_roi_after_fees_pct: f64,
    pub max_quote_age_ms: u64,
    pub min_liquidity_depth: f64,
    pub min_confidence_score: f64,
}

impl StrategyOpportunityParameters {
    fn validate(&self, violations: &mut Vec<StrategyProfileViolation>) {
        validate_non_negative(
            "opportunity.min_net_profit_abs",
            self.min_net_profit_abs,
            violations,
        );
        validate_non_negative(
            "opportunity.min_net_profit_pct",
            self.min_net_profit_pct,
            violations,
        );
        validate_non_negative(
            "opportunity.min_roi_after_fees_pct",
            self.min_roi_after_fees_pct,
            violations,
        );
        validate_positive(
            "opportunity.min_liquidity_depth",
            self.min_liquidity_depth,
            violations,
        );
        validate_percent(
            "opportunity.min_confidence_score",
            self.min_confidence_score,
            violations,
        );
        if self.max_quote_age_ms == 0 {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_MAX_QUOTE_AGE_ZERO",
                "opportunity.max_quote_age_ms must be positive",
            ));
        }
    }
}

/// Strategy execution-shape parameters.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyExecutionParameters {
    pub max_slippage_bps: u16,
    pub max_gas_native: f64,
    pub max_gas_quote: f64,
    pub order_timeout_ms: u64,
    pub cancel_on_partial_fill: bool,
    pub allow_market_orders: bool,
    pub allow_limit_orders: bool,
    pub allow_ioc: bool,
    pub allow_fok: bool,
    pub allow_flashloans: bool,
    pub allow_bridges: bool,
    pub allow_withdrawals: bool,
    pub allowed_destination_labels: Vec<String>,
}

impl StrategyExecutionParameters {
    fn validate(&self, violations: &mut Vec<StrategyProfileViolation>) {
        validate_non_negative("execution.max_gas_native", self.max_gas_native, violations);
        validate_non_negative("execution.max_gas_quote", self.max_gas_quote, violations);
        if self.order_timeout_ms == 0 {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_ORDER_TIMEOUT_ZERO",
                "execution.order_timeout_ms must be positive",
            ));
        }
        if !self.allow_market_orders && !self.allow_limit_orders {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_ORDER_TYPE_EMPTY",
                "at least one local order shape must be allowed",
            ));
        }
        if self.allow_flashloans {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_FLASHLOANS_DENIED",
                "execution.allow_flashloans must remain false",
            ));
        }
        if self.allow_bridges {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_BRIDGES_DENIED",
                "execution.allow_bridges must remain false",
            ));
        }
        if self.allow_withdrawals {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_WITHDRAWALS_DENIED",
                "execution.allow_withdrawals must remain false",
            ));
        }
        validate_unique_ids(
            "destination label",
            &self.allowed_destination_labels,
            violations,
        );
    }
}

/// Strategy venue and asset allowlists.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyVenueParameters {
    pub allowed_exchanges: Vec<String>,
    pub allowed_chains: Vec<String>,
    pub allowed_routers: Vec<String>,
    pub allowed_assets: Vec<String>,
}

impl StrategyVenueParameters {
    fn validate(&self, violations: &mut Vec<StrategyProfileViolation>) {
        validate_unique_ids("exchange", &self.allowed_exchanges, violations);
        validate_unique_ids("chain", &self.allowed_chains, violations);
        validate_unique_ids("router", &self.allowed_routers, violations);
        validate_unique_ids("asset", &self.allowed_assets, violations);
    }

    fn validate_intent_allowlists(
        &self,
        intent: &ExecutionIntent,
        violations: &mut Vec<StrategyProfileViolation>,
    ) {
        if !self.allowed_exchanges.is_empty()
            && !contains_ignore_ascii(&self.allowed_exchanges, &intent.venue.name)
        {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_VENUE_NOT_ALLOWED",
                "intent venue is not allowed by the strategy profile",
            ));
        }

        if matches!(
            intent.venue.kind,
            VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge
        ) && !self.allowed_routers.is_empty()
            && !contains_ignore_ascii(&self.allowed_routers, &intent.venue.name)
        {
            violations.push(StrategyProfileViolation::new(
                "STRATEGY_ROUTER_NOT_ALLOWED",
                "intent router is not allowed by the strategy profile",
            ));
        }

        if let Some(chain) = &intent.chain {
            if !self.allowed_chains.is_empty()
                && !contains_ignore_ascii(&self.allowed_chains, chain)
            {
                violations.push(StrategyProfileViolation::new(
                    "STRATEGY_CHAIN_NOT_ALLOWED",
                    "intent chain is not allowed by the strategy profile",
                ));
            }
        }

        for asset in [&intent.base_asset, &intent.quote_asset] {
            if !self.allowed_assets.is_empty()
                && !contains_ignore_ascii(&self.allowed_assets, asset)
            {
                violations.push(StrategyProfileViolation::new(
                    "STRATEGY_ASSET_NOT_ALLOWED",
                    "intent asset is not allowed by the strategy profile",
                ));
            }
        }
    }
}

/// Strategy notification preferences.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyAlertParameters {
    pub notify_on_opportunity: bool,
    pub notify_on_execution: bool,
    pub notify_on_policy_denial: bool,
    pub notify_on_loss: bool,
    pub notify_on_kill_switch: bool,
}

impl Default for StrategyAlertParameters {
    fn default() -> Self {
        Self {
            notify_on_opportunity: true,
            notify_on_execution: true,
            notify_on_policy_denial: true,
            notify_on_loss: true,
            notify_on_kill_switch: true,
        }
    }
}

/// Strategy constraint report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrategyPolicyConstraintStatus {
    /// Intent stayed inside strategy constraints.
    Satisfied,
    /// Intent violated one or more strategy constraints.
    Rejected,
}

/// Deterministic report describing local strategy constraint checks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyPolicyConstraintReport {
    pub strategy_profile_version: String,
    pub strategy_id: String,
    pub intent_id: String,
    pub status: StrategyPolicyConstraintStatus,
    pub violations: Vec<StrategyProfileViolation>,
    pub execution_performed: bool,
    pub signing_or_broadcast_performed: bool,
    pub live_network_used: bool,
}

/// Strategy profile validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyProfileError {
    /// Validation failed.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<StrategyProfileViolation>,
    },
}

impl StrategyProfileError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[StrategyProfileViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
        }
    }
}

impl fmt::Display for StrategyProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                writeln!(
                    formatter,
                    "strategy profile validation failed with {} violation(s):",
                    violations.len()
                )?;
                for violation in violations {
                    writeln!(formatter, "- {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for StrategyProfileError {}

/// One deterministic strategy profile violation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProfileViolation {
    code: String,
    message: String,
}

impl StrategyProfileViolation {
    /// Create a validation violation.
    #[must_use]
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
        }
    }

    /// Create a validation violation with owned message text.
    #[must_use]
    pub fn new_owned(code: &'static str, message: String) -> Self {
        Self {
            code: code.to_owned(),
            message,
        }
    }

    /// Stable violation code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Human-readable violation detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

fn finish_validation(
    violations: Vec<StrategyProfileViolation>,
) -> Result<(), StrategyProfileError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(StrategyProfileError::ValidationFailed { violations })
    }
}

fn scope_allowed(profile_mode: RuntimeMode, intent_scope: ExecutionScope) -> bool {
    match profile_mode {
        RuntimeMode::Observe => intent_scope == ExecutionScope::Observe,
        RuntimeMode::Paper => intent_scope != ExecutionScope::Live,
        RuntimeMode::LiveArmed => false,
    }
}

fn validate_id(kind: &'static str, value: &str, violations: &mut Vec<StrategyProfileViolation>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        violations.push(StrategyProfileViolation::new_owned(
            "STRATEGY_ID_EMPTY",
            format!("{kind} id must be non-empty"),
        ));
    }
    if trimmed.len() > 128 {
        violations.push(StrategyProfileViolation::new_owned(
            "STRATEGY_ID_TOO_LONG",
            format!("{kind} id must be 128 characters or fewer"),
        ));
    }
}

fn validate_symbol(
    kind: &'static str,
    value: &str,
    violations: &mut Vec<StrategyProfileViolation>,
) {
    validate_id(kind, value, violations);
    if value.chars().any(|character| {
        !(character.is_ascii_alphanumeric() || character == '-' || character == '_')
    }) {
        violations.push(StrategyProfileViolation::new_owned(
            "STRATEGY_SYMBOL_INVALID",
            format!("{kind} contains unsupported characters"),
        ));
    }
}

fn validate_unique_ids(
    kind: &'static str,
    values: &[String],
    violations: &mut Vec<StrategyProfileViolation>,
) {
    let mut ids = BTreeSet::new();
    for value in values {
        validate_id(kind, value, violations);
        if !ids.insert(value.to_ascii_lowercase()) {
            violations.push(StrategyProfileViolation::new_owned(
                "STRATEGY_ID_DUPLICATE",
                format!("{kind} {value} is duplicated"),
            ));
        }
    }
}

fn validate_positive(
    label: &'static str,
    value: f64,
    violations: &mut Vec<StrategyProfileViolation>,
) {
    if !is_positive_finite(value) {
        violations.push(StrategyProfileViolation::new_owned(
            "STRATEGY_VALUE_NOT_POSITIVE",
            format!("{label} must be positive and finite"),
        ));
    }
}

fn validate_non_negative(
    label: &'static str,
    value: f64,
    violations: &mut Vec<StrategyProfileViolation>,
) {
    if !is_non_negative_finite(value) {
        violations.push(StrategyProfileViolation::new_owned(
            "STRATEGY_VALUE_NEGATIVE_OR_INVALID",
            format!("{label} must be finite and non-negative"),
        ));
    }
}

fn validate_percent(
    label: &'static str,
    value: f64,
    violations: &mut Vec<StrategyProfileViolation>,
) {
    if !is_non_negative_finite(value) || value > 100.0 {
        violations.push(StrategyProfileViolation::new_owned(
            "STRATEGY_PERCENT_INVALID",
            format!("{label} must be finite and between 0 and 100"),
        ));
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn is_non_negative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn contains_ignore_ascii(values: &[String], needle: &str) -> bool {
    values
        .iter()
        .any(|value| value.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    use super::{
        StrategyPolicyConstraintStatus, StrategyProfile, StrategyProfileError,
        StrategyProfileViolation,
    };
    use crate::{
        DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope, RuntimeMode,
        VenueKind, VenueRef,
    };

    fn intent() -> ExecutionIntent {
        ExecutionIntent {
            id: "intent-paper-btc-usd".to_owned(),
            strategy_id: "paper-cross-venue".to_owned(),
            kind: ExecutionIntentKind::CrossExchangeArbitrage,
            scope: ExecutionScope::Paper,
            venue: VenueRef {
                name: "paper-a".to_owned(),
                kind: VenueKind::Cex,
            },
            chain: None,
            base_asset: "BTC".to_owned(),
            quote_asset: "USD".to_owned(),
            notional_quote: 500.0,
            expected_profit_quote: 15.0,
            max_loss_quote: 50.0,
            slippage_bps: 20,
            estimated_fee_quote: 2.0,
            gas_fee_quote: 0.0,
            market_data_age_ms: 1_000,
            destination: DestinationPolicy::InternalAccount,
            requires_signing: false,
        }
    }

    fn profile() -> StrategyProfile {
        let mut profile = StrategyProfile::conservative_paper("paper-cross-venue", "USD");
        profile.venues.allowed_exchanges = vec!["paper-a".to_owned(), "paper-b".to_owned()];
        profile.venues.allowed_assets = vec!["BTC".to_owned(), "USD".to_owned()];
        profile
    }

    #[test]
    fn conservative_strategy_profile_constrains_paper_intent() {
        let profile = profile();
        profile.validate().expect("profile should validate");

        let report = profile.constrain_intent(&intent());

        assert_eq!(report.status, StrategyPolicyConstraintStatus::Satisfied);
        assert!(report.violations.is_empty());
        assert!(!report.execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.live_network_used);
    }

    #[test]
    fn strategy_profile_rejects_live_withdrawal_bridge_flags() {
        let mut profile = profile();
        profile.mode = RuntimeMode::LiveArmed;
        profile.execution.allow_withdrawals = true;
        profile.execution.allow_bridges = true;
        profile.execution.allow_flashloans = true;

        let error = profile
            .validate()
            .expect_err("unsafe strategy profile must fail closed");
        let StrategyProfileError::ValidationFailed { violations } = error;
        let codes = violation_codes(&violations);

        assert!(codes.contains(&"STRATEGY_LIVE_MODE_DENIED"));
        assert!(codes.contains(&"STRATEGY_WITHDRAWALS_DENIED"));
        assert!(codes.contains(&"STRATEGY_BRIDGES_DENIED"));
        assert!(codes.contains(&"STRATEGY_FLASHLOANS_DENIED"));
    }

    #[test]
    fn strategy_profile_rejects_intent_outside_thresholds() {
        let profile = profile();
        let mut intent = intent();
        intent.strategy_id = "other-profile".to_owned();
        intent.scope = ExecutionScope::Live;
        intent.notional_quote = 2_000.0;
        intent.expected_profit_quote = 1.0;
        intent.max_loss_quote = 200.0;
        intent.slippage_bps = 100;
        intent.market_data_age_ms = 10_000;
        intent.venue.name = "paper-c".to_owned();
        intent.base_asset = "ETH".to_owned();

        let report = profile.constrain_intent(&intent);
        let codes = violation_codes(&report.violations);

        assert_eq!(report.status, StrategyPolicyConstraintStatus::Rejected);
        assert!(codes.contains(&"STRATEGY_INTENT_PROFILE_MISMATCH"));
        assert!(codes.contains(&"STRATEGY_INTENT_SCOPE_DENIED"));
        assert!(codes.contains(&"STRATEGY_MAX_PER_OPPORTUNITY_EXCEEDED"));
        assert!(codes.contains(&"STRATEGY_MIN_NET_PROFIT_NOT_MET"));
        assert!(codes.contains(&"STRATEGY_SLIPPAGE_EXCEEDED"));
        assert!(codes.contains(&"STRATEGY_QUOTE_AGE_EXCEEDED"));
        assert!(codes.contains(&"STRATEGY_VENUE_NOT_ALLOWED"));
        assert!(codes.contains(&"STRATEGY_ASSET_NOT_ALLOWED"));
    }

    #[test]
    fn strategy_profile_rejects_invalid_bounds() {
        let mut profile = profile();
        profile.capital.max_total_deployed = 0.0;
        profile.capital.compound_rate = 2.0;
        profile.risk.max_open_exposure_pct = 101.0;
        profile.opportunity.max_quote_age_ms = 0;
        profile.execution.order_timeout_ms = 0;
        profile.execution.allow_limit_orders = false;
        profile.venues.allowed_exchanges.push("paper-a".to_owned());

        let error = profile
            .validate()
            .expect_err("invalid profile bounds must fail closed");
        let StrategyProfileError::ValidationFailed { violations } = error;
        let codes = violation_codes(&violations);

        assert!(codes.contains(&"STRATEGY_VALUE_NOT_POSITIVE"));
        assert!(codes.contains(&"STRATEGY_COMPOUND_RATE_INVALID"));
        assert!(codes.contains(&"STRATEGY_PERCENT_INVALID"));
        assert!(codes.contains(&"STRATEGY_MAX_QUOTE_AGE_ZERO"));
        assert!(codes.contains(&"STRATEGY_ORDER_TIMEOUT_ZERO"));
        assert!(codes.contains(&"STRATEGY_ORDER_TYPE_EMPTY"));
        assert!(codes.contains(&"STRATEGY_ID_DUPLICATE"));
    }

    fn violation_codes(violations: &[StrategyProfileViolation]) -> Vec<&str> {
        violations
            .iter()
            .map(StrategyProfileViolation::code)
            .collect::<Vec<_>>()
    }
}
