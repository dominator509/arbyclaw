#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    ExecutionIntent, ExecutionScope, FeeAdjustedEdge, FeeModelError, FeeProvider, FeeSchedule,
    LiquidityRole, MarketDataCapabilities, MarketDataError, MarketDataProvider, MarketDataRequest,
    MarketPair, NormalizedQuote, OrderBookSnapshot, PolicyDecision, PolicyEngine, PolicyViolation,
    VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable paper connector version for audit and future replay surfaces.
pub const PAPER_CONNECTOR_VERSION: &str = "phase-6-paper-connector-v1";

/// Deterministic in-memory market-data provider for paper and simulation mode.
///
/// This provider never opens sockets, calls an exchange, loads secrets, mutates
/// balances, signs transactions, or places orders. It only returns caller-supplied
/// normalized snapshots after validating them.
#[derive(Debug, Clone, PartialEq)]
pub struct PaperMarketDataProvider {
    name: String,
    books: Vec<OrderBookSnapshot>,
}

impl PaperMarketDataProvider {
    /// Build a paper market-data provider from pre-normalized snapshots.
    pub fn new(
        name: impl Into<String>,
        books: Vec<OrderBookSnapshot>,
    ) -> Result<Self, PaperConnectorError> {
        let provider = Self {
            name: name.into(),
            books,
        };
        provider.validate()?;
        Ok(provider)
    }

    /// Return the number of configured paper snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.books.len()
    }

    /// Return true when no snapshots are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    fn validate(&self) -> Result<(), PaperConnectorError> {
        if self.name.trim().is_empty() {
            return Err(PaperConnectorError::InvalidProviderName);
        }

        if self.books.is_empty() {
            return Err(PaperConnectorError::EmptyMarketData);
        }

        for book in &self.books {
            book.validate()
                .map_err(|source| PaperConnectorError::InvalidMarketData {
                    reason: source.to_string(),
                })?;
        }

        Ok(())
    }

    fn best_matching_book(
        &self,
        request: &MarketDataRequest,
    ) -> Result<OrderBookSnapshot, MarketDataError> {
        request.validate()?;

        self.books
            .iter()
            .filter(|book| same_venue(&book.venue, &request.venue) && book.pair == request.pair)
            .max_by_key(|book| book.received_at_unix_ms)
            .cloned()
            .ok_or_else(|| MarketDataError::NoData {
                provider: self.name.clone(),
                reason: format!(
                    "no paper snapshot for venue {} and pair {}",
                    request.venue.name,
                    request.pair.symbol()
                ),
            })
    }
}

impl MarketDataProvider for PaperMarketDataProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> MarketDataCapabilities {
        MarketDataCapabilities {
            order_book: true,
            top_of_book: true,
            fees: false,
            websocket: false,
            rest: false,
        }
    }

    fn order_book(
        &self,
        request: &MarketDataRequest,
    ) -> Result<OrderBookSnapshot, MarketDataError> {
        self.best_matching_book(request)
    }

    fn top_of_book(&self, request: &MarketDataRequest) -> Result<NormalizedQuote, MarketDataError> {
        self.best_matching_book(request)?.to_quote()
    }
}

/// Deterministic static fee provider for paper and simulation mode.
///
/// Schedules are caller-supplied and may be deliberately conservative. They are
/// not treated as externally verified exchange schedules.
#[derive(Debug, Clone, PartialEq)]
pub struct PaperFeeProvider {
    name: String,
    schedules: Vec<FeeSchedule>,
}

impl PaperFeeProvider {
    /// Build a paper fee provider from static fee schedules.
    pub fn new(
        name: impl Into<String>,
        schedules: Vec<FeeSchedule>,
    ) -> Result<Self, PaperConnectorError> {
        let provider = Self {
            name: name.into(),
            schedules,
        };
        provider.validate()?;
        Ok(provider)
    }

    /// Return the number of configured fee schedules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.schedules.len()
    }

    /// Return true when no schedules are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.schedules.is_empty()
    }

    fn validate(&self) -> Result<(), PaperConnectorError> {
        if self.name.trim().is_empty() {
            return Err(PaperConnectorError::InvalidProviderName);
        }

        if self.schedules.is_empty() {
            return Err(PaperConnectorError::EmptyFeeSchedules);
        }

        for schedule in &self.schedules {
            schedule
                .validate()
                .map_err(|source| PaperConnectorError::InvalidFeeSchedule {
                    reason: source.to_string(),
                })?;
        }

        Ok(())
    }
}

impl FeeProvider for PaperFeeProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn fee_schedule(
        &self,
        venue: &VenueRef,
        pair: Option<&MarketPair>,
    ) -> Result<FeeSchedule, FeeModelError> {
        self.schedules
            .iter()
            .find(|schedule| schedule_matches(schedule, venue, pair))
            .cloned()
            .ok_or_else(|| FeeModelError::ScheduleUnavailable {
                provider: self.name.clone(),
                reason: format!("no paper fee schedule for venue {}", venue.name),
            })
    }
}

/// Policy-gated paper execution adapter.
///
/// This adapter does not place real orders or mutate real balances. It produces
/// deterministic paper reports only after `PolicyEngine` approval and only for
/// paper-scoped intents.
#[derive(Debug, Clone, PartialEq)]
pub struct PaperExecutionAdapter {
    name: String,
    policy: PolicyEngine,
}

impl PaperExecutionAdapter {
    /// Build a policy-gated paper execution adapter.
    pub fn new(name: impl Into<String>, policy: PolicyEngine) -> Result<Self, PaperConnectorError> {
        let adapter = Self {
            name: name.into(),
            policy,
        };
        if adapter.name.trim().is_empty() {
            return Err(PaperConnectorError::InvalidProviderName);
        }
        Ok(adapter)
    }

    /// Stable adapter name for diagnostics and future audit records.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.name
    }

    /// Submit a paper intent and return a deterministic paper report.
    pub fn submit(
        &self,
        intent: &ExecutionIntent,
    ) -> Result<PaperExecutionReport, PaperConnectorError> {
        if intent.scope != ExecutionScope::Paper {
            return Err(PaperConnectorError::NonPaperScope {
                scope: intent.scope,
            });
        }

        let decision = self.policy.evaluate(intent);
        let approval = match decision {
            PolicyDecision::Approved { approval } => approval,
            PolicyDecision::Denied { violations } => {
                return Err(PaperConnectorError::PolicyDenied { violations });
            }
        };

        let total_fees_quote = intent.estimated_fee_quote + intent.gas_fee_quote;
        let edge = FeeAdjustedEdge::calculate(
            intent.expected_profit_quote,
            total_fees_quote,
            intent.notional_quote,
        )
        .map_err(|source| PaperConnectorError::FeeCalculationFailed {
            reason: source.to_string(),
        })?;

        if !edge.is_profitable() {
            return Err(PaperConnectorError::NotProfitableAfterFees);
        }

        Ok(PaperExecutionReport {
            id: format!("paper-report-{}", intent.id),
            adapter: self.name.clone(),
            connector_version: PAPER_CONNECTOR_VERSION.to_owned(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            trust_contract_version: approval.trust_contract_version.to_owned(),
            status: PaperExecutionStatus::Filled,
            scope: approval.approved_scope,
            venue: intent.venue.clone(),
            base_asset: intent.base_asset.clone(),
            quote_asset: intent.quote_asset.clone(),
            notional_quote: intent.notional_quote,
            gross_profit_quote: intent.expected_profit_quote,
            total_fees_quote,
            net_profit_quote: edge.net_profit_quote,
            roi_bps: edge.roi_bps,
            liquidity_role: LiquidityRole::Taker,
        })
    }
}

/// Deterministic paper execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaperExecutionStatus {
    /// Paper order was accepted by policy and represented as a deterministic fill.
    Filled,
}

/// Deterministic paper execution report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperExecutionReport {
    /// Stable report id.
    pub id: String,
    /// Adapter that produced the report.
    pub adapter: String,
    /// Paper connector version.
    pub connector_version: String,
    /// Source policy-approved intent id.
    pub intent_id: String,
    /// Source strategy id.
    pub strategy_id: String,
    /// Trust-contract version that approved the intent.
    pub trust_contract_version: String,
    /// Paper execution status.
    pub status: PaperExecutionStatus,
    /// Approved scope.
    pub scope: ExecutionScope,
    /// Paper venue.
    pub venue: VenueRef,
    /// Base asset.
    pub base_asset: String,
    /// Quote asset.
    pub quote_asset: String,
    /// Proposed notional amount.
    pub notional_quote: f64,
    /// Gross expected paper profit before fees.
    pub gross_profit_quote: f64,
    /// Total estimated fees applied by the report.
    pub total_fees_quote: f64,
    /// Net paper profit after estimated fees.
    pub net_profit_quote: f64,
    /// Net paper ROI in basis points.
    pub roi_bps: f64,
    /// Liquidity role assumed by this Phase 6 paper adapter.
    pub liquidity_role: LiquidityRole,
}

/// Errors from deterministic paper connector scaffolds.
#[derive(Debug, Clone, PartialEq)]
pub enum PaperConnectorError {
    /// Provider or adapter name was empty.
    InvalidProviderName,
    /// No paper market-data snapshots were configured.
    EmptyMarketData,
    /// A supplied paper market-data snapshot failed validation.
    InvalidMarketData { reason: String },
    /// No paper fee schedules were configured.
    EmptyFeeSchedules,
    /// A supplied paper fee schedule failed validation.
    InvalidFeeSchedule { reason: String },
    /// Intent requested a non-paper execution scope.
    NonPaperScope { scope: ExecutionScope },
    /// Policy denied the paper intent.
    PolicyDenied { violations: Vec<PolicyViolation> },
    /// Fee-adjusted edge calculation failed.
    FeeCalculationFailed { reason: String },
    /// Intent was not profitable after estimated fees.
    NotProfitableAfterFees,
}

impl fmt::Display for PaperConnectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProviderName => {
                formatter.write_str("paper provider name must be non-empty")
            }
            Self::EmptyMarketData => {
                formatter.write_str("paper market-data provider requires at least one snapshot")
            }
            Self::InvalidMarketData { reason } => {
                write!(formatter, "invalid paper market data: {reason}")
            }
            Self::EmptyFeeSchedules => {
                formatter.write_str("paper fee provider requires at least one schedule")
            }
            Self::InvalidFeeSchedule { reason } => {
                write!(formatter, "invalid paper fee schedule: {reason}")
            }
            Self::NonPaperScope { scope } => write!(
                formatter,
                "paper adapter rejected non-paper scope: {scope:?}"
            ),
            Self::PolicyDenied { violations } => {
                write!(
                    formatter,
                    "paper intent denied by policy with {} violation(s)",
                    violations.len()
                )?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::FeeCalculationFailed { reason } => {
                write!(formatter, "paper fee calculation failed: {reason}")
            }
            Self::NotProfitableAfterFees => {
                formatter.write_str("paper intent is not profitable after estimated fees")
            }
        }
    }
}

impl Error for PaperConnectorError {}

fn same_venue(left: &VenueRef, right: &VenueRef) -> bool {
    left.kind == right.kind && left.name.eq_ignore_ascii_case(&right.name)
}

fn schedule_matches(schedule: &FeeSchedule, venue: &VenueRef, pair: Option<&MarketPair>) -> bool {
    if !same_venue(&schedule.venue, venue) {
        return false;
    }

    match (&schedule.pair, pair) {
        (Some(left), Some(right)) => left == right,
        (Some(_), None) => false,
        (None, _) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PaperExecutionAdapter, PaperExecutionStatus, PaperFeeProvider, PaperMarketDataProvider,
    };
    use crate::{
        AgentConfig, DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope,
        FeeProvider, FeeSchedule, LiquidityRole, MarketDataProvider, MarketDataRequest, MarketPair,
        OrderBookSnapshot, PolicyEngine, PriceLevel, VenueKind, VenueRef,
    };

    const PAPER_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 50.0
max_daily_loss_quote = 10.0
max_open_exposure_quote = 100.0
slippage_bps = 50
gas_fee_cap_quote = 2.0

[venues]
cex_allowlist = ["paper-coinbase"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC", "USDC"]

[secrets]
backend = "disabled"
exchange_credentials = { source = "disabled" }
wallet_signer = { source = "disabled" }

[communication]
cli_enabled = true
notify_channels = []

[audit]
enabled = true
redact_secrets = true
"#;

    fn venue() -> VenueRef {
        VenueRef {
            name: "paper-coinbase".to_owned(),
            kind: VenueKind::Cex,
        }
    }

    fn pair() -> MarketPair {
        MarketPair::new("BTC", "USDC").expect("pair should validate")
    }

    fn book() -> OrderBookSnapshot {
        OrderBookSnapshot {
            id: "paper-book-1".to_owned(),
            venue: venue(),
            pair: pair(),
            captured_at_unix_ms: 1_000,
            received_at_unix_ms: 1_001,
            bids: vec![PriceLevel::new(100.0, 1.0).expect("bid should validate")],
            asks: vec![PriceLevel::new(101.0, 1.0).expect("ask should validate")],
            source_sequence: Some("paper-seq-1".to_owned()),
        }
    }

    fn intent() -> ExecutionIntent {
        ExecutionIntent {
            id: "intent-paper-1".to_owned(),
            strategy_id: "strategy-paper".to_owned(),
            kind: ExecutionIntentKind::CexOrder,
            scope: ExecutionScope::Paper,
            venue: venue(),
            chain: None,
            base_asset: "BTC".to_owned(),
            quote_asset: "USDC".to_owned(),
            notional_quote: 25.0,
            expected_profit_quote: 1.0,
            max_loss_quote: 1.0,
            slippage_bps: 20,
            estimated_fee_quote: 0.10,
            gas_fee_quote: 0.0,
            market_data_age_ms: 1_000,
            destination: DestinationPolicy::InternalAccount,
            requires_signing: false,
        }
    }

    #[test]
    fn paper_market_data_provider_returns_top_of_book() {
        let provider = PaperMarketDataProvider::new("paper-md", vec![book()])
            .expect("provider should validate");
        let request = MarketDataRequest {
            venue: venue(),
            pair: pair(),
            max_age_ms: 5_000,
        };
        let quote = provider.top_of_book(&request).expect("quote should exist");
        assert_eq!(quote.venue, venue());
        assert!((quote.mid_price_quote() - 100.5).abs() < f64::EPSILON);
    }

    #[test]
    fn paper_fee_provider_returns_pair_schedule() {
        let schedule = FeeSchedule {
            venue: venue(),
            pair: Some(pair()),
            maker_bps: 1.0,
            taker_bps: 5.0,
            network_fee_quote: 0.0,
            externally_verified: false,
        };
        let provider = PaperFeeProvider::new("paper-fees", vec![schedule])
            .expect("fee provider should validate");
        let estimate = provider
            .fee_schedule(&venue(), Some(&pair()))
            .expect("schedule should exist")
            .estimate(100.0, LiquidityRole::Taker)
            .expect("estimate should validate");
        assert!((estimate.total_fee_quote - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn paper_execution_requires_policy_approval() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let report = adapter.submit(&intent()).expect("paper intent should fill");
        assert_eq!(report.status, PaperExecutionStatus::Filled);
        assert_eq!(report.scope, ExecutionScope::Paper);
        assert!((report.net_profit_quote - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn paper_execution_rejects_live_scope_before_policy() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let mut live_intent = intent();
        live_intent.scope = ExecutionScope::Live;
        let error = adapter
            .submit(&live_intent)
            .expect_err("live scope must be rejected");
        assert!(matches!(
            error,
            super::PaperConnectorError::NonPaperScope { .. }
        ));
    }
}
