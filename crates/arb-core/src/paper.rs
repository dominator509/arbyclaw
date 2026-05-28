#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    ExecutionIntent, ExecutionScope, FeeAdjustedEdge, FeeModelError, FeeProvider, FeeSchedule,
    LiquidityRole, MarketDataCapabilities, MarketDataError, MarketDataProvider, MarketDataRequest,
    MarketPair, NormalizedQuote, OrderBookSnapshot, PolicyDecision, PolicyEngine, PolicyViolation,
    PriceLevel, StateCheckpoint, StateStore, StateStoreError, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable paper connector version for audit and future replay surfaces.
pub const PAPER_CONNECTOR_VERSION: &str = "phase-6-paper-connector-v1";

/// Stable realistic paper fill model version for audit and replay surfaces.
pub const PAPER_REALISTIC_FILL_MODEL_VERSION: &str = "phase-23-realistic-paper-fills-v1";

/// State-store subsystem name for paper execution checkpoints.
pub const PAPER_EXECUTION_STATE_SUBSYSTEM: &str = "paper-execution";

/// State-store key for the latest deterministic paper execution report.
pub const PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY: &str = "paper-execution:last-report";

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
        self.submit_internal(intent, None)
    }

    /// Submit a paper intent using a caller-supplied local order book and fill model.
    ///
    /// This path never calls an exchange. It depth-walks the supplied snapshot,
    /// applies deterministic latency and queue-position haircuts, and records
    /// partial or unfilled paper outcomes without external submission.
    pub fn submit_with_fill_model(
        &self,
        request: &PaperFillSimulationRequest,
    ) -> Result<PaperExecutionReport, PaperConnectorError> {
        request.validate()?;
        self.submit_internal(&request.intent, Some(request))
    }

    fn submit_internal(
        &self,
        intent: &ExecutionIntent,
        fill_request: Option<&PaperFillSimulationRequest>,
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

        let fill = if let Some(request) = fill_request {
            simulate_paper_fill(request)?
        } else {
            PaperFillSimulationReport::full_notional(intent, 0)
        };

        let fill_ratio = if intent.notional_quote > 0.0 {
            fill.filled_notional_quote / intent.notional_quote
        } else {
            0.0
        };
        let gross_profit_quote = intent.expected_profit_quote * fill_ratio;
        let modeled_fees_quote = total_fees_quote * fill_ratio;
        let net_profit_quote = gross_profit_quote - modeled_fees_quote;
        let roi_bps = if fill.filled_notional_quote > 0.0 {
            (net_profit_quote / fill.filled_notional_quote) * 10_000.0
        } else {
            0.0
        };
        let status = match fill.status {
            PaperFillSimulationStatus::Filled => PaperExecutionStatus::Filled,
            PaperFillSimulationStatus::PartiallyFilled => PaperExecutionStatus::PartiallyFilled,
            PaperFillSimulationStatus::Unfilled => PaperExecutionStatus::Unfilled,
        };

        Ok(PaperExecutionReport {
            id: format!("paper-report-{}", intent.id),
            adapter: self.name.clone(),
            connector_version: PAPER_CONNECTOR_VERSION.to_owned(),
            fill_model_version: fill.fill_model_version.clone(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            trust_contract_version: approval.trust_contract_version.to_owned(),
            status,
            scope: approval.approved_scope,
            venue: intent.venue.clone(),
            base_asset: intent.base_asset.clone(),
            quote_asset: intent.quote_asset.clone(),
            notional_quote: intent.notional_quote,
            filled_notional_quote: fill.filled_notional_quote,
            unfilled_notional_quote: fill.unfilled_notional_quote,
            average_fill_price_quote: fill.average_fill_price_quote,
            gross_profit_quote,
            total_fees_quote: modeled_fees_quote,
            net_profit_quote,
            roi_bps,
            liquidity_role: LiquidityRole::Taker,
            fill_simulation: fill,
        })
    }

    /// Submit a paper intent through a local paper balance ledger.
    ///
    /// This helper does not call external venues. It produces a deterministic
    /// paper report only after policy/profit validation and a successful local
    /// notional reservation, then settles the modeled fill into the ledger.
    pub fn submit_with_ledger(
        &self,
        intent: &ExecutionIntent,
        ledger: &mut PaperBalanceLedger,
        now_unix_ms: u64,
    ) -> Result<PaperLedgeredExecution, PaperConnectorError> {
        let report = self.submit(intent)?;
        let reserve_entry = ledger.reserve_for_intent(intent, now_unix_ms)?;
        let settlement_entry = ledger.settle_report(&report, now_unix_ms.saturating_add(1))?;
        Ok(PaperLedgeredExecution {
            report,
            reserve_entry,
            settlement_entry,
        })
    }

    /// Submit a paper intent through realistic fill simulation and local ledgering.
    ///
    /// The ledger reserves the requested notional, then settles filled notional,
    /// modeled net P&L, and any unfilled remainder back to available balance.
    pub fn submit_with_fill_model_and_ledger(
        &self,
        request: &PaperFillSimulationRequest,
        ledger: &mut PaperBalanceLedger,
    ) -> Result<PaperLedgeredExecution, PaperConnectorError> {
        let report = self.submit_with_fill_model(request)?;
        let reserve_entry = ledger.reserve_for_intent(&request.intent, request.now_unix_ms)?;
        let settlement_entry =
            ledger.settle_report(&report, request.now_unix_ms.saturating_add(1))?;
        Ok(PaperLedgeredExecution {
            report,
            reserve_entry,
            settlement_entry,
        })
    }
}

/// Deterministic side for consuming a supplied paper order book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaperFillSide {
    /// Buy base asset with quote notional by walking asks.
    BuyBase,
    /// Sell base asset into quote notional by walking bids.
    SellBase,
}

/// Deterministic fill status for a supplied local order book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaperFillSimulationStatus {
    /// Requested notional filled after depth, latency, queue, and slippage checks.
    Filled,
    /// Some, but not all, requested notional filled.
    PartiallyFilled,
    /// No modeled fill was available.
    Unfilled,
}

/// Local-only fill realism settings.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperFillModelConfig {
    /// Maximum tolerated slippage against the top-of-book reference.
    pub max_slippage_bps: u16,
    /// Deterministic latency to add to the modeled fill timestamp.
    pub latency_ms: u64,
    /// Queue-position haircut in basis points applied to every level quantity.
    pub queue_position_bps: u16,
    /// Whether partial fills are allowed when depth or slippage is insufficient.
    pub allow_partial_fills: bool,
    /// Minimum filled share in basis points when partial fills are allowed.
    pub min_partial_fill_bps: u16,
}

impl Default for PaperFillModelConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 50,
            latency_ms: 250,
            queue_position_bps: 0,
            allow_partial_fills: true,
            min_partial_fill_bps: 1,
        }
    }
}

impl PaperFillModelConfig {
    /// Validate deterministic fill realism settings.
    pub fn validate(&self) -> Result<(), PaperConnectorError> {
        if self.max_slippage_bps > 10_000 {
            return Err(PaperConnectorError::InvalidFillModel {
                reason: "max_slippage_bps must be at most 10000".to_owned(),
            });
        }
        if self.queue_position_bps > 10_000 {
            return Err(PaperConnectorError::InvalidFillModel {
                reason: "queue_position_bps must be at most 10000".to_owned(),
            });
        }
        if self.min_partial_fill_bps > 10_000 {
            return Err(PaperConnectorError::InvalidFillModel {
                reason: "min_partial_fill_bps must be at most 10000".to_owned(),
            });
        }
        Ok(())
    }
}

/// Request for local deterministic realistic paper fill simulation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperFillSimulationRequest {
    /// Paper-scoped execution intent.
    pub intent: ExecutionIntent,
    /// Caller-supplied normalized book. No live provider is called.
    pub order_book: OrderBookSnapshot,
    /// Book side to consume.
    pub side: PaperFillSide,
    /// Local fill realism settings.
    pub config: PaperFillModelConfig,
    /// Runtime clock in Unix milliseconds.
    pub now_unix_ms: u64,
}

impl PaperFillSimulationRequest {
    /// Validate the local simulation request without side effects.
    pub fn validate(&self) -> Result<(), PaperConnectorError> {
        if self.intent.scope != ExecutionScope::Paper {
            return Err(PaperConnectorError::NonPaperScope {
                scope: self.intent.scope,
            });
        }
        validate_positive_finite("paper notional", self.intent.notional_quote)?;
        self.order_book
            .validate()
            .map_err(|source| PaperConnectorError::InvalidMarketData {
                reason: source.to_string(),
            })?;
        if !same_venue(&self.intent.venue, &self.order_book.venue) {
            return Err(PaperConnectorError::InvalidFillModel {
                reason: "intent venue must match order book venue".to_owned(),
            });
        }
        if self.intent.base_asset != self.order_book.pair.base
            || self.intent.quote_asset != self.order_book.pair.quote
        {
            return Err(PaperConnectorError::InvalidFillModel {
                reason: "intent assets must match order book pair".to_owned(),
            });
        }
        if self.now_unix_ms == 0 {
            return Err(PaperConnectorError::InvalidFillModel {
                reason: "fill simulation timestamp must be non-zero".to_owned(),
            });
        }
        self.config.validate()?;
        Ok(())
    }
}

/// Deterministic local paper fill simulation result.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperFillSimulationReport {
    /// Stable fill model version.
    pub fill_model_version: String,
    /// Fill outcome.
    pub status: PaperFillSimulationStatus,
    /// Consumed side of the supplied order book.
    pub side: PaperFillSide,
    /// Requested notional in quote units.
    pub requested_notional_quote: f64,
    /// Modeled filled notional in quote units.
    pub filled_notional_quote: f64,
    /// Unfilled notional in quote units.
    pub unfilled_notional_quote: f64,
    /// Modeled filled base quantity.
    pub filled_base_quantity: f64,
    /// Weighted-average fill price, or zero when unfilled.
    pub average_fill_price_quote: f64,
    /// Top-of-book reference price.
    pub reference_price_quote: f64,
    /// Worst consumed level price, or zero when unfilled.
    pub worst_fill_price_quote: f64,
    /// Modeled slippage in basis points against the reference price.
    pub slippage_bps: f64,
    /// Number of book levels consumed.
    pub consumed_levels: usize,
    /// Configured latency applied to fill timestamp.
    pub latency_ms: u64,
    /// Configured queue-position haircut.
    pub queue_position_bps: u16,
    /// Modeled fill timestamp.
    pub filled_at_unix_ms: u64,
    /// Non-secret explanation for the outcome.
    pub reason: String,
}

impl PaperFillSimulationReport {
    fn full_notional(intent: &ExecutionIntent, latency_ms: u64) -> Self {
        Self {
            fill_model_version: PAPER_CONNECTOR_VERSION.to_owned(),
            status: PaperFillSimulationStatus::Filled,
            side: PaperFillSide::BuyBase,
            requested_notional_quote: intent.notional_quote,
            filled_notional_quote: intent.notional_quote,
            unfilled_notional_quote: 0.0,
            filled_base_quantity: 0.0,
            average_fill_price_quote: 0.0,
            reference_price_quote: 0.0,
            worst_fill_price_quote: 0.0,
            slippage_bps: 0.0,
            consumed_levels: 0,
            latency_ms,
            queue_position_bps: 0,
            filled_at_unix_ms: 0,
            reason: "legacy deterministic full-notional paper fill".to_owned(),
        }
    }
}

/// State-store key for the latest deterministic paper balance ledger.
pub const PAPER_BALANCE_LEDGER_CHECKPOINT_KEY: &str = "paper-execution:balance-ledger";

/// Paper balance ledger version for replay and handoff surfaces.
pub const PAPER_BALANCE_LEDGER_VERSION: &str = "phase-21-paper-balance-ledger-v1";

/// One local paper balance for a venue/asset pair.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAssetBalance {
    /// Paper venue.
    pub venue: VenueRef,
    /// Asset symbol.
    pub asset: String,
    /// Available paper balance.
    pub available: f64,
    /// Reserved paper balance.
    pub reserved: f64,
}

impl PaperAssetBalance {
    /// Build a paper balance with no reserved amount.
    pub fn available(
        venue: VenueRef,
        asset: impl Into<String>,
        amount: f64,
    ) -> Result<Self, PaperConnectorError> {
        let balance = Self {
            venue,
            asset: asset.into(),
            available: amount,
            reserved: 0.0,
        };
        balance.validate()?;
        Ok(balance)
    }

    /// Total paper balance including reserved funds.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.available + self.reserved
    }

    fn validate(&self) -> Result<(), PaperConnectorError> {
        validate_asset_symbol(&self.asset)?;
        validate_non_negative_finite("available balance", self.available)?;
        validate_non_negative_finite("reserved balance", self.reserved)?;
        Ok(())
    }
}

/// Paper ledger entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaperLedgerEntryKind {
    /// Initial balance loaded into the paper ledger.
    InitialBalance,
    /// Paper notional reserved before a modeled fill is recorded.
    ReserveNotional,
    /// Paper modeled fill settled back into available balance with net P&L.
    SettleFill,
}

/// One deterministic paper balance mutation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperLedgerEntry {
    /// Stable entry id.
    pub id: String,
    /// Mutation kind.
    pub kind: PaperLedgerEntryKind,
    /// Paper venue.
    pub venue: VenueRef,
    /// Asset symbol.
    pub asset: String,
    /// Optional execution intent id.
    pub intent_id: Option<String>,
    /// Optional paper report id.
    pub report_id: Option<String>,
    /// Available-balance delta.
    pub available_delta: f64,
    /// Reserved-balance delta.
    pub reserved_delta: f64,
    /// Available balance after this mutation.
    pub resulting_available: f64,
    /// Reserved balance after this mutation.
    pub resulting_reserved: f64,
    /// Runtime clock in Unix milliseconds.
    pub recorded_at_unix_ms: u64,
}

/// Local deterministic paper balance ledger.
///
/// This ledger only tracks simulated balances supplied by the caller. It does
/// not query exchanges, mutate real accounts, custody assets, sign payloads, or
/// prove strategy profitability.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperBalanceLedger {
    /// Ledger version.
    pub ledger_version: String,
    /// Current balances.
    pub balances: Vec<PaperAssetBalance>,
    /// Deterministic mutation history.
    pub entries: Vec<PaperLedgerEntry>,
}

impl PaperBalanceLedger {
    /// Create a ledger from explicit non-secret paper balances.
    pub fn new(
        initial_balances: Vec<PaperAssetBalance>,
        now_unix_ms: u64,
    ) -> Result<Self, PaperConnectorError> {
        if initial_balances.is_empty() {
            return Err(PaperConnectorError::EmptyPaperBalances);
        }
        if now_unix_ms == 0 {
            return Err(PaperConnectorError::InvalidPaperLedger {
                reason: "ledger timestamp must be non-zero".to_owned(),
            });
        }

        let mut ledger = Self {
            ledger_version: PAPER_BALANCE_LEDGER_VERSION.to_owned(),
            balances: Vec::new(),
            entries: Vec::new(),
        };
        for balance in initial_balances {
            ledger.add_initial_balance(balance, now_unix_ms)?;
        }
        Ok(ledger)
    }

    /// Return available balance for a venue/asset pair.
    #[must_use]
    pub fn available_balance(&self, venue: &VenueRef, asset: &str) -> f64 {
        self.find_balance(venue, asset)
            .map_or(0.0, |balance| balance.available)
    }

    /// Return reserved balance for a venue/asset pair.
    #[must_use]
    pub fn reserved_balance(&self, venue: &VenueRef, asset: &str) -> f64 {
        self.find_balance(venue, asset)
            .map_or(0.0, |balance| balance.reserved)
    }

    /// Reserve quote notional for a paper intent.
    pub fn reserve_for_intent(
        &mut self,
        intent: &ExecutionIntent,
        now_unix_ms: u64,
    ) -> Result<PaperLedgerEntry, PaperConnectorError> {
        if intent.scope != ExecutionScope::Paper {
            return Err(PaperConnectorError::NonPaperScope {
                scope: intent.scope,
            });
        }
        validate_positive_finite("paper notional", intent.notional_quote)?;
        self.mutate_balance(
            PaperLedgerMutation {
                kind: PaperLedgerEntryKind::ReserveNotional,
                venue: intent.venue.clone(),
                asset: intent.quote_asset.clone(),
                intent_id: Some(intent.id.clone()),
                report_id: None,
                available_delta: -intent.notional_quote,
                reserved_delta: intent.notional_quote,
                recorded_at_unix_ms: now_unix_ms,
            },
            true,
        )
    }

    /// Settle a deterministic paper fill into quote balance.
    pub fn settle_report(
        &mut self,
        report: &PaperExecutionReport,
        now_unix_ms: u64,
    ) -> Result<PaperLedgerEntry, PaperConnectorError> {
        if report.scope != ExecutionScope::Paper {
            return Err(PaperConnectorError::NonPaperScope {
                scope: report.scope,
            });
        }
        if report.status == PaperExecutionStatus::Unfilled && report.filled_notional_quote > 0.0 {
            return Err(PaperConnectorError::InvalidPaperLedger {
                reason: "unfilled paper reports must not carry filled notional".to_owned(),
            });
        }
        validate_positive_finite("paper notional", report.notional_quote)?;
        validate_non_negative_finite("filled paper notional", report.filled_notional_quote)?;
        validate_non_negative_finite("unfilled paper notional", report.unfilled_notional_quote)?;
        validate_finite("paper net profit", report.net_profit_quote)?;
        if self.reserved_balance(&report.venue, &report.quote_asset) + f64::EPSILON
            < report.notional_quote
        {
            return Err(PaperConnectorError::MissingPaperReservation {
                venue: report.venue.name.clone(),
                asset: report.quote_asset.clone(),
                required: report.notional_quote,
                reserved: self.reserved_balance(&report.venue, &report.quote_asset),
            });
        }

        self.mutate_balance(
            PaperLedgerMutation {
                kind: PaperLedgerEntryKind::SettleFill,
                venue: report.venue.clone(),
                asset: report.quote_asset.clone(),
                intent_id: Some(report.intent_id.clone()),
                report_id: Some(report.id.clone()),
                available_delta: report.notional_quote + report.net_profit_quote,
                reserved_delta: -report.notional_quote,
                recorded_at_unix_ms: now_unix_ms,
            },
            false,
        )
    }

    fn add_initial_balance(
        &mut self,
        balance: PaperAssetBalance,
        now_unix_ms: u64,
    ) -> Result<(), PaperConnectorError> {
        balance.validate()?;
        if self.find_balance(&balance.venue, &balance.asset).is_some() {
            return Err(PaperConnectorError::DuplicatePaperBalance {
                venue: balance.venue.name,
                asset: balance.asset,
            });
        }
        let entry = PaperLedgerEntry {
            id: format!(
                "paper-ledger-entry-{}",
                self.entries.len().saturating_add(1)
            ),
            kind: PaperLedgerEntryKind::InitialBalance,
            venue: balance.venue.clone(),
            asset: balance.asset.clone(),
            intent_id: None,
            report_id: None,
            available_delta: balance.available,
            reserved_delta: balance.reserved,
            resulting_available: balance.available,
            resulting_reserved: balance.reserved,
            recorded_at_unix_ms: now_unix_ms,
        };
        self.balances.push(balance);
        self.entries.push(entry);
        Ok(())
    }

    fn mutate_balance(
        &mut self,
        mutation: PaperLedgerMutation,
        require_available: bool,
    ) -> Result<PaperLedgerEntry, PaperConnectorError> {
        if mutation.recorded_at_unix_ms == 0 {
            return Err(PaperConnectorError::InvalidPaperLedger {
                reason: "ledger timestamp must be non-zero".to_owned(),
            });
        }
        validate_asset_symbol(&mutation.asset)?;
        validate_finite("available delta", mutation.available_delta)?;
        validate_finite("reserved delta", mutation.reserved_delta)?;
        let next_entry_number = self.entries.len().saturating_add(1);
        let balance = self
            .find_balance_mut(&mutation.venue, &mutation.asset)
            .ok_or_else(|| PaperConnectorError::MissingPaperBalance {
                venue: mutation.venue.name.clone(),
                asset: mutation.asset.clone(),
            })?;

        if require_available && balance.available + f64::EPSILON < -mutation.available_delta {
            return Err(PaperConnectorError::InsufficientPaperBalance {
                venue: mutation.venue.name,
                asset: mutation.asset,
                required: -mutation.available_delta,
                available: balance.available,
            });
        }

        let resulting_available = balance.available + mutation.available_delta;
        let resulting_reserved = balance.reserved + mutation.reserved_delta;
        validate_non_negative_finite("resulting available balance", resulting_available)?;
        validate_non_negative_finite("resulting reserved balance", resulting_reserved)?;

        balance.available = normalize_zero(resulting_available);
        balance.reserved = normalize_zero(resulting_reserved);

        let entry = PaperLedgerEntry {
            id: format!("paper-ledger-entry-{next_entry_number}"),
            kind: mutation.kind,
            venue: mutation.venue,
            asset: mutation.asset,
            intent_id: mutation.intent_id,
            report_id: mutation.report_id,
            available_delta: mutation.available_delta,
            reserved_delta: mutation.reserved_delta,
            resulting_available: balance.available,
            resulting_reserved: balance.reserved,
            recorded_at_unix_ms: mutation.recorded_at_unix_ms,
        };
        self.entries.push(entry.clone());
        Ok(entry)
    }

    fn find_balance(&self, venue: &VenueRef, asset: &str) -> Option<&PaperAssetBalance> {
        self.balances
            .iter()
            .find(|balance| same_venue(&balance.venue, venue) && balance.asset == asset)
    }

    fn find_balance_mut(
        &mut self,
        venue: &VenueRef,
        asset: &str,
    ) -> Option<&mut PaperAssetBalance> {
        self.balances
            .iter_mut()
            .find(|balance| same_venue(&balance.venue, venue) && balance.asset == asset)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PaperLedgerMutation {
    kind: PaperLedgerEntryKind,
    venue: VenueRef,
    asset: String,
    intent_id: Option<String>,
    report_id: Option<String>,
    available_delta: f64,
    reserved_delta: f64,
    recorded_at_unix_ms: u64,
}

/// Paper execution report plus the ledger entries that made it balance-safe.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperLedgeredExecution {
    /// Deterministic paper execution report.
    pub report: PaperExecutionReport,
    /// Ledger entry that reserved quote notional.
    pub reserve_entry: PaperLedgerEntry,
    /// Ledger entry that settled the modeled fill.
    pub settlement_entry: PaperLedgerEntry,
}

/// Deterministic paper execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaperExecutionStatus {
    /// Paper order was accepted by policy and represented as a deterministic fill.
    Filled,
    /// Paper order was accepted by policy but only partially filled by the local depth model.
    PartiallyFilled,
    /// Paper order was accepted by policy but received no local modeled fill.
    Unfilled,
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
    /// Fill model version.
    pub fill_model_version: String,
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
    /// Modeled filled notional amount.
    pub filled_notional_quote: f64,
    /// Modeled unfilled notional amount.
    pub unfilled_notional_quote: f64,
    /// Weighted-average fill price, or zero for legacy/unfilled reports.
    pub average_fill_price_quote: f64,
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
    /// Local deterministic fill simulation detail.
    pub fill_simulation: PaperFillSimulationReport,
}

/// Persist the latest deterministic paper execution report as a non-secret checkpoint.
///
/// This helper only writes through the typed local `StateStore` boundary. It
/// does not submit orders, mutate balances, call exchanges, sign payloads, or
/// broadcast transactions.
pub fn persist_paper_execution_report_checkpoint(
    store: &mut impl StateStore,
    report: &PaperExecutionReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, StateStoreError> {
    let checkpoint = StateCheckpoint {
        key: PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY.to_owned(),
        subsystem: PAPER_EXECUTION_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize paper execution report checkpoint: {error}"),
        })?,
        updated_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Persist the latest deterministic paper balance ledger as a non-secret checkpoint.
///
/// This helper writes only modeled paper balances and ledger entries through the
/// typed local `StateStore` boundary. It never reads real balances or touches
/// exchange accounts.
pub fn persist_paper_balance_ledger_checkpoint(
    store: &mut impl StateStore,
    ledger: &PaperBalanceLedger,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, StateStoreError> {
    let checkpoint = StateCheckpoint {
        key: PAPER_BALANCE_LEDGER_CHECKPOINT_KEY.to_owned(),
        subsystem: PAPER_EXECUTION_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(ledger).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize paper balance ledger checkpoint: {error}"),
        })?,
        updated_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
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
    /// No initial paper balances were supplied.
    EmptyPaperBalances,
    /// A venue/asset balance appeared more than once.
    DuplicatePaperBalance { venue: String, asset: String },
    /// A required paper balance does not exist.
    MissingPaperBalance { venue: String, asset: String },
    /// A paper balance did not have enough available funds.
    InsufficientPaperBalance {
        venue: String,
        asset: String,
        required: f64,
        available: f64,
    },
    /// A settlement did not have enough reserved funds.
    MissingPaperReservation {
        venue: String,
        asset: String,
        required: f64,
        reserved: f64,
    },
    /// Paper ledger validation failed.
    InvalidPaperLedger { reason: String },
    /// Realistic paper fill model validation failed.
    InvalidFillModel { reason: String },
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
            Self::EmptyPaperBalances => {
                formatter.write_str("paper balance ledger requires at least one balance")
            }
            Self::DuplicatePaperBalance { venue, asset } => {
                write!(formatter, "duplicate paper balance for {venue}:{asset}")
            }
            Self::MissingPaperBalance { venue, asset } => {
                write!(formatter, "missing paper balance for {venue}:{asset}")
            }
            Self::InsufficientPaperBalance {
                venue,
                asset,
                required,
                available,
            } => write!(
                formatter,
                "insufficient paper balance for {venue}:{asset}; required {required}, available {available}"
            ),
            Self::MissingPaperReservation {
                venue,
                asset,
                required,
                reserved,
            } => write!(
                formatter,
                "missing paper reservation for {venue}:{asset}; required {required}, reserved {reserved}"
            ),
            Self::InvalidPaperLedger { reason } => {
                write!(formatter, "invalid paper balance ledger: {reason}")
            }
            Self::InvalidFillModel { reason } => {
                write!(formatter, "invalid paper fill model: {reason}")
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

fn validate_asset_symbol(asset: &str) -> Result<(), PaperConnectorError> {
    if asset.trim().is_empty() {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "asset symbol is required".to_owned(),
        });
    }
    if asset.to_ascii_lowercase().contains("secret")
        || asset.to_ascii_lowercase().contains("private")
        || asset.to_ascii_lowercase().contains("token")
    {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "asset symbol contains secret-like text".to_owned(),
        });
    }
    Ok(())
}

fn validate_positive_finite(label: &str, value: f64) -> Result<(), PaperConnectorError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(PaperConnectorError::InvalidPaperLedger {
            reason: format!("{label} must be positive and finite"),
        })
    }
}

fn validate_non_negative_finite(label: &str, value: f64) -> Result<(), PaperConnectorError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(PaperConnectorError::InvalidPaperLedger {
            reason: format!("{label} must be non-negative and finite"),
        })
    }
}

fn validate_finite(label: &str, value: f64) -> Result<(), PaperConnectorError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(PaperConnectorError::InvalidPaperLedger {
            reason: format!("{label} must be finite"),
        })
    }
}

fn normalize_zero(value: f64) -> f64 {
    if value.abs() <= f64::EPSILON {
        0.0
    } else {
        value
    }
}

#[allow(clippy::too_many_lines)]
fn simulate_paper_fill(
    request: &PaperFillSimulationRequest,
) -> Result<PaperFillSimulationReport, PaperConnectorError> {
    request.validate()?;
    let levels = match request.side {
        PaperFillSide::BuyBase => &request.order_book.asks,
        PaperFillSide::SellBase => &request.order_book.bids,
    };
    let reference_price_quote = levels
        .first()
        .map(|level| level.price_quote)
        .ok_or_else(|| PaperConnectorError::InvalidFillModel {
            reason: "order book side has no levels".to_owned(),
        })?;
    let queue_multiplier = 1.0 - (f64::from(request.config.queue_position_bps) / 10_000.0);
    let requested_notional_quote = request.intent.notional_quote;
    let max_average_price =
        reference_price_quote * (1.0 + (f64::from(request.config.max_slippage_bps) / 10_000.0));
    let min_average_price =
        reference_price_quote * (1.0 - (f64::from(request.config.max_slippage_bps) / 10_000.0));

    let mut remaining_notional = requested_notional_quote;
    let mut filled_notional_quote = 0.0;
    let mut filled_base_quantity = 0.0;
    let mut worst_fill_price_quote = 0.0;
    let mut consumed_levels = 0usize;

    for level in levels {
        let effective_level = effective_level(*level, queue_multiplier)?;
        if effective_level.quantity_base <= 0.0 {
            continue;
        }
        let level_notional = effective_level.notional_quote();
        if level_notional <= 0.0 {
            continue;
        }

        let candidate_notional = remaining_notional.min(level_notional);
        let candidate_base = candidate_notional / effective_level.price_quote;
        let next_filled_notional = filled_notional_quote + candidate_notional;
        let next_filled_base = filled_base_quantity + candidate_base;
        let next_average_price = next_filled_notional / next_filled_base;
        let slippage_ok = match request.side {
            PaperFillSide::BuyBase => next_average_price <= max_average_price + f64::EPSILON,
            PaperFillSide::SellBase => next_average_price + f64::EPSILON >= min_average_price,
        };

        if !slippage_ok {
            break;
        }

        filled_notional_quote = next_filled_notional;
        filled_base_quantity = next_filled_base;
        worst_fill_price_quote = effective_level.price_quote;
        consumed_levels = consumed_levels.saturating_add(1);
        remaining_notional -= candidate_notional;
        if remaining_notional <= f64::EPSILON {
            break;
        }
    }

    let filled_ratio_bps = if requested_notional_quote > 0.0 {
        (filled_notional_quote / requested_notional_quote) * 10_000.0
    } else {
        0.0
    };
    if filled_notional_quote > 0.0
        && filled_notional_quote + f64::EPSILON < requested_notional_quote
        && (!request.config.allow_partial_fills
            || filled_ratio_bps + f64::EPSILON < f64::from(request.config.min_partial_fill_bps))
    {
        filled_notional_quote = 0.0;
        filled_base_quantity = 0.0;
        worst_fill_price_quote = 0.0;
        consumed_levels = 0;
    }

    let unfilled_notional_quote = normalize_zero(requested_notional_quote - filled_notional_quote);
    let average_fill_price_quote = if filled_base_quantity > 0.0 {
        filled_notional_quote / filled_base_quantity
    } else {
        0.0
    };
    let slippage_bps = if average_fill_price_quote > 0.0 {
        match request.side {
            PaperFillSide::BuyBase => {
                ((average_fill_price_quote - reference_price_quote) / reference_price_quote)
                    * 10_000.0
            }
            PaperFillSide::SellBase => {
                ((reference_price_quote - average_fill_price_quote) / reference_price_quote)
                    * 10_000.0
            }
        }
    } else {
        0.0
    };
    let status = if filled_notional_quote <= f64::EPSILON {
        PaperFillSimulationStatus::Unfilled
    } else if unfilled_notional_quote <= f64::EPSILON {
        PaperFillSimulationStatus::Filled
    } else {
        PaperFillSimulationStatus::PartiallyFilled
    };
    let reason = match status {
        PaperFillSimulationStatus::Filled => "requested notional filled within supplied depth and slippage limits",
        PaperFillSimulationStatus::PartiallyFilled => {
            "partial fill modeled because supplied depth or slippage limit stopped remaining notional"
        }
        PaperFillSimulationStatus::Unfilled => {
            "no fill modeled because supplied depth, slippage, or partial-fill policy was insufficient"
        }
    }
    .to_owned();

    Ok(PaperFillSimulationReport {
        fill_model_version: PAPER_REALISTIC_FILL_MODEL_VERSION.to_owned(),
        status,
        side: request.side,
        requested_notional_quote,
        filled_notional_quote: normalize_zero(filled_notional_quote),
        unfilled_notional_quote,
        filled_base_quantity: normalize_zero(filled_base_quantity),
        average_fill_price_quote: normalize_zero(average_fill_price_quote),
        reference_price_quote,
        worst_fill_price_quote: normalize_zero(worst_fill_price_quote),
        slippage_bps: normalize_zero(slippage_bps.max(0.0)),
        consumed_levels,
        latency_ms: request.config.latency_ms,
        queue_position_bps: request.config.queue_position_bps,
        filled_at_unix_ms: if status == PaperFillSimulationStatus::Unfilled {
            0
        } else {
            request
                .now_unix_ms
                .saturating_add(request.config.latency_ms)
        },
        reason,
    })
}

fn effective_level(
    level: PriceLevel,
    queue_multiplier: f64,
) -> Result<PriceLevel, PaperConnectorError> {
    let quantity_base = level.quantity_base * queue_multiplier;
    if quantity_base <= 0.0 {
        return Ok(PriceLevel {
            price_quote: level.price_quote,
            quantity_base: 0.0,
        });
    }
    PriceLevel::new(level.price_quote, quantity_base).map_err(|source| {
        PaperConnectorError::InvalidFillModel {
            reason: source.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        persist_paper_balance_ledger_checkpoint, persist_paper_execution_report_checkpoint,
        PaperAssetBalance, PaperBalanceLedger, PaperExecutionAdapter, PaperExecutionStatus,
        PaperFeeProvider, PaperFillModelConfig, PaperFillSide, PaperFillSimulationRequest,
        PaperFillSimulationStatus, PaperLedgerEntryKind, PaperMarketDataProvider,
        PAPER_BALANCE_LEDGER_CHECKPOINT_KEY, PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY,
        PAPER_EXECUTION_STATE_SUBSYSTEM,
    };
    use crate::{
        AgentConfig, DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope,
        FeeProvider, FeeSchedule, LiquidityRole, MarketDataProvider, MarketDataRequest, MarketPair,
        OrderBookSnapshot, PolicyEngine, PriceLevel, SqliteWalStateStore, StateStore, VenueKind,
        VenueRef,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process,
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

    fn depth_book() -> OrderBookSnapshot {
        OrderBookSnapshot {
            id: "paper-depth-book-1".to_owned(),
            venue: venue(),
            pair: pair(),
            captured_at_unix_ms: 1_000,
            received_at_unix_ms: 1_001,
            bids: vec![
                PriceLevel::new(100.0, 0.10).expect("bid level validates"),
                PriceLevel::new(99.0, 0.10).expect("bid level validates"),
            ],
            asks: vec![
                PriceLevel::new(101.0, 0.10).expect("ask level validates"),
                PriceLevel::new(102.0, 0.10).expect("ask level validates"),
                PriceLevel::new(104.0, 1.00).expect("ask level validates"),
            ],
            source_sequence: Some("paper-depth-seq-1".to_owned()),
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

    fn fill_request(
        notional_quote: f64,
        max_slippage_bps: u16,
        allow_partial_fills: bool,
    ) -> PaperFillSimulationRequest {
        let mut intent = intent();
        intent.id = format!("intent-fill-{notional_quote:.1}").replace('.', "-");
        intent.notional_quote = notional_quote;
        intent.expected_profit_quote = notional_quote * 0.04;
        intent.estimated_fee_quote = notional_quote * 0.001;
        PaperFillSimulationRequest {
            intent,
            order_book: depth_book(),
            side: PaperFillSide::BuyBase,
            config: PaperFillModelConfig {
                max_slippage_bps,
                latency_ms: 25,
                queue_position_bps: 0,
                allow_partial_fills,
                min_partial_fill_bps: 1,
            },
            now_unix_ms: 1_700_000_000_500,
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

    #[test]
    fn paper_execution_report_persists_as_state_checkpoint() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let report = adapter.submit(&intent()).expect("paper intent should fill");
        let mut store = crate::InMemoryStateStore::new();

        let checkpoint =
            persist_paper_execution_report_checkpoint(&mut store, &report, 1_700_000_000_000)
                .expect("paper report checkpoint should persist");

        assert_eq!(checkpoint.key, PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY);
        assert_eq!(checkpoint.subsystem, PAPER_EXECUTION_STATE_SUBSYSTEM);
        assert_eq!(checkpoint.updated_at_unix_ms, 1_700_000_000_000);
        let restored: super::PaperExecutionReport =
            serde_json::from_str(&checkpoint.value).expect("checkpoint json should parse");
        assert_eq!(restored, report);
        assert_eq!(
            store
                .get_checkpoint(PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY)
                .expect("checkpoint should read"),
            Some(checkpoint)
        );
    }

    #[test]
    fn paper_execution_report_persists_through_sqlite_wal_store() {
        let path = unique_state_path("paper-report");
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let report = adapter.submit(&intent()).expect("paper intent should fill");

        {
            let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");
            persist_paper_execution_report_checkpoint(&mut store, &report, 1_700_000_000_001)
                .expect("paper report checkpoint should persist");
        }

        {
            let store = SqliteWalStateStore::open(&path).expect("sqlite store reopens");
            let checkpoint = store
                .get_checkpoint(PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY)
                .expect("checkpoint should read")
                .expect("checkpoint should exist");
            let restored: super::PaperExecutionReport =
                serde_json::from_str(&checkpoint.value).expect("checkpoint json should parse");
            assert_eq!(checkpoint.subsystem, PAPER_EXECUTION_STATE_SUBSYSTEM);
            assert_eq!(checkpoint.updated_at_unix_ms, 1_700_000_000_001);
            assert_eq!(restored, report);
        }

        cleanup_state_files(&path);
    }

    #[test]
    fn paper_balance_ledger_reserves_and_settles_modeled_fill() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_000_000,
            )
            .expect("ledger validates");

        let execution = adapter
            .submit_with_ledger(&intent(), &mut ledger, 1_700_000_000_100)
            .expect("ledgered paper execution succeeds");

        assert_eq!(
            execution.reserve_entry.kind,
            PaperLedgerEntryKind::ReserveNotional
        );
        assert_eq!(
            execution.settlement_entry.kind,
            PaperLedgerEntryKind::SettleFill
        );
        assert_eq!(execution.report.status, PaperExecutionStatus::Filled);
        assert!((ledger.available_balance(&venue(), "USDC") - 1_000.9).abs() < f64::EPSILON);
        assert!(ledger.reserved_balance(&venue(), "USDC").abs() < f64::EPSILON);
        assert_eq!(ledger.entries.len(), 3);
    }

    #[test]
    fn realistic_paper_fill_walks_depth_and_records_average_price() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");

        let report = adapter
            .submit_with_fill_model(&fill_request(20.3, 100, true))
            .expect("realistic fill should complete");

        assert_eq!(report.status, PaperExecutionStatus::Filled);
        assert_eq!(
            report.fill_simulation.status,
            PaperFillSimulationStatus::Filled
        );
        assert_eq!(report.fill_simulation.consumed_levels, 2);
        assert!((report.filled_notional_quote - 20.3).abs() < 0.000_000_1);
        assert!(report.average_fill_price_quote > 101.0);
        assert_eq!(report.fill_simulation.filled_at_unix_ms, 1_700_000_000_525);
    }

    #[test]
    fn realistic_paper_fill_partially_fills_and_ledger_releases_remainder() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_000_000,
            )
            .expect("ledger validates");
        let request = fill_request(40.0, 80, true);

        let execution = adapter
            .submit_with_fill_model_and_ledger(&request, &mut ledger)
            .expect("partial realistic fill settles");

        assert_eq!(
            execution.report.status,
            PaperExecutionStatus::PartiallyFilled
        );
        assert_eq!(
            execution.report.fill_simulation.status,
            PaperFillSimulationStatus::PartiallyFilled
        );
        assert!(execution.report.filled_notional_quote < request.intent.notional_quote);
        assert!(execution.report.unfilled_notional_quote > 0.0);
        assert_eq!(execution.report.fill_simulation.consumed_levels, 2);
        assert!(ledger.reserved_balance(&venue(), "USDC").abs() < f64::EPSILON);
        assert!(ledger.available_balance(&venue(), "USDC") > 1_000.0);
    }

    #[test]
    fn realistic_paper_fill_can_fail_closed_when_partials_are_disallowed() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let report = adapter
            .submit_with_fill_model(&fill_request(40.0, 80, false))
            .expect("unfilled model report should still be deterministic");

        assert_eq!(report.status, PaperExecutionStatus::Unfilled);
        assert_eq!(
            report.fill_simulation.status,
            PaperFillSimulationStatus::Unfilled
        );
        assert!(report.filled_notional_quote.abs() < f64::EPSILON);
        assert!((report.unfilled_notional_quote - 40.0).abs() < f64::EPSILON);
        assert!(report.net_profit_quote.abs() < f64::EPSILON);
    }

    #[test]
    fn paper_balance_ledger_blocks_insufficient_available_balance() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let mut ledger = PaperBalanceLedger::new(
            vec![PaperAssetBalance::available(venue(), "USDC", 10.0).expect("balance validates")],
            1_700_000_000_000,
        )
        .expect("ledger validates");

        let error = adapter
            .submit_with_ledger(&intent(), &mut ledger, 1_700_000_000_100)
            .expect_err("insufficient paper balance fails closed");

        assert!(matches!(
            error,
            super::PaperConnectorError::InsufficientPaperBalance { .. }
        ));
        assert!((ledger.available_balance(&venue(), "USDC") - 10.0).abs() < f64::EPSILON);
        assert!(ledger.reserved_balance(&venue(), "USDC").abs() < f64::EPSILON);
        assert_eq!(ledger.entries.len(), 1);
    }

    #[test]
    fn paper_balance_ledger_settlement_requires_reservation() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let report = adapter.submit(&intent()).expect("paper intent should fill");
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_000_000,
            )
            .expect("ledger validates");

        let error = ledger
            .settle_report(&report, 1_700_000_000_100)
            .expect_err("settlement without reservation fails closed");

        assert!(matches!(
            error,
            super::PaperConnectorError::MissingPaperReservation { .. }
        ));
        assert!((ledger.available_balance(&venue(), "USDC") - 1_000.0).abs() < f64::EPSILON);
        assert!(ledger.reserved_balance(&venue(), "USDC").abs() < f64::EPSILON);
    }

    #[test]
    fn paper_balance_ledger_persists_through_sqlite_wal_store() {
        let path = unique_state_path("paper-ledger");
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_000_000,
            )
            .expect("ledger validates");
        adapter
            .submit_with_ledger(&intent(), &mut ledger, 1_700_000_000_100)
            .expect("ledgered paper execution succeeds");

        {
            let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");
            persist_paper_balance_ledger_checkpoint(&mut store, &ledger, 1_700_000_000_200)
                .expect("paper ledger checkpoint should persist");
        }

        {
            let store = SqliteWalStateStore::open(&path).expect("sqlite store reopens");
            let checkpoint = store
                .get_checkpoint(PAPER_BALANCE_LEDGER_CHECKPOINT_KEY)
                .expect("checkpoint should read")
                .expect("checkpoint should exist");
            let restored: PaperBalanceLedger =
                serde_json::from_str(&checkpoint.value).expect("checkpoint json should parse");
            assert_eq!(checkpoint.subsystem, PAPER_EXECUTION_STATE_SUBSYSTEM);
            assert_eq!(restored, ledger);
            assert!((restored.available_balance(&venue(), "USDC") - 1_000.9).abs() < f64::EPSILON);
        }

        cleanup_state_files(&path);
    }

    fn unique_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-paper-{label}-{}-{nanos}.sqlite3",
            process::id()
        ));
        path
    }

    fn cleanup_state_files(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", path.display())));
        }
    }
}
