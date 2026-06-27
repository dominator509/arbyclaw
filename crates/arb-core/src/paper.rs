#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue,
    ExecutionAdapterRunRecord, ExecutionFillRecord, ExecutionFillStatus, ExecutionIntent,
    ExecutionPlanDraft, ExecutionReconciliationStatus, ExecutionScope, FeeAdjustedEdge,
    FeeModelError, FeeProvider, FeeSchedule, LiquidityRole, MarketDataCapabilities,
    MarketDataError, MarketDataProvider, MarketDataRequest, MarketPair, NormalizedQuote,
    OrderBookSnapshot, PolicyDecision, PolicyEngine, PolicyViolation, PriceLevel, StateCheckpoint,
    StateStore, StateStoreError, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable paper connector version for audit and future replay surfaces.
pub const PAPER_CONNECTOR_VERSION: &str = "phase-6-paper-connector-v1";

/// Stable realistic paper fill model version for audit and replay surfaces.
pub const PAPER_REALISTIC_FILL_MODEL_VERSION: &str = "phase-23-realistic-paper-fills-v1";

/// Stable venue-realism, replay, and backtest validation version.
pub const PAPER_REALISM_VALIDATION_VERSION: &str = "phase-24-paper-replay-calibration-runtime-v1";

/// Stable paper audit journal integration version.
pub const PAPER_AUDIT_INTEGRATION_VERSION: &str = "phase-25-paper-audit-journal-v1";

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

    /// Submit a paper intent with local venue-specific realism controls.
    ///
    /// This path applies caller-supplied exchange matching constraints,
    /// adverse-selection assumptions, and reference-only calibration records to
    /// the existing paper fill model. It never calls sandbox or live venues.
    pub fn submit_with_venue_realism(
        &self,
        request: &PaperVenueRealismRequest,
    ) -> Result<PaperVenueRealismExecution, PaperConnectorError> {
        request.validate()?;
        let (adjusted_request, matching) =
            apply_matching_profile_to_request(&request.fill_request, &request.matching_profile)?;
        let mut report = self.submit_internal(&adjusted_request.intent, Some(&adjusted_request))?;
        let adverse_selection = simulate_adverse_selection(
            &request.adverse_selection,
            &report.fill_simulation,
            report.net_profit_quote,
        )?;
        let calibration = if let Some(record) = &request.calibration {
            Some(apply_calibration_record(
                record,
                &report.fill_simulation,
                report.net_profit_quote - adverse_selection.penalty_quote,
            )?)
        } else {
            None
        };
        let calibration_adjustment_quote = calibration
            .as_ref()
            .map_or(0.0, |record| record.penalty_quote);
        report.fill_model_version = PAPER_REALISM_VALIDATION_VERSION.to_owned();
        report.adverse_selection_quote = adverse_selection.penalty_quote;
        report.calibration_adjustment_quote = calibration_adjustment_quote;
        report.net_profit_quote -= adverse_selection.penalty_quote + calibration_adjustment_quote;
        report.roi_bps = roi_bps(report.net_profit_quote, report.filled_notional_quote);

        Ok(PaperVenueRealismExecution {
            report,
            matching,
            adverse_selection,
            calibration,
        })
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
            adverse_selection_quote: 0.0,
            calibration_adjustment_quote: 0.0,
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

    /// Submit a realistic local paper fill, mutate the local paper ledger, and
    /// append the report plus ledger mutations to the local audit journal.
    pub fn submit_with_fill_model_ledger_and_audit(
        &self,
        request: &PaperFillSimulationRequest,
        ledger: &mut PaperBalanceLedger,
        journal: &mut AppendOnlyAuditJournal,
    ) -> Result<PaperAuditedLedgeredExecution, PaperConnectorError> {
        let intent_record =
            append_paper_execution_intent_audit(journal, &request.intent, request.now_unix_ms)?;
        let ledgered_execution = self.submit_with_fill_model_and_ledger(request, ledger)?;
        let audit_report = append_paper_ledgered_execution_audit(
            journal,
            &ledgered_execution,
            request.now_unix_ms.saturating_add(2),
            Some(intent_record.sequence),
        )?;
        Ok(PaperAuditedLedgeredExecution {
            ledgered_execution,
            audit_report,
        })
    }

    /// Submit a venue-realism paper execution and settle it into the local ledger.
    ///
    /// The ledger reserves the requested notional and settles the adjusted paper
    /// P&L after matching, adverse-selection, and calibration penalties.
    pub fn submit_with_venue_realism_and_ledger(
        &self,
        request: &PaperVenueRealismRequest,
        ledger: &mut PaperBalanceLedger,
    ) -> Result<PaperVenueRealismLedgeredExecution, PaperConnectorError> {
        let execution = self.submit_with_venue_realism(request)?;
        let reserve_entry = ledger.reserve_for_intent(
            &request.fill_request.intent,
            request.fill_request.now_unix_ms,
        )?;
        let settlement_entry = ledger.settle_report(
            &execution.report,
            request.fill_request.now_unix_ms.saturating_add(1),
        )?;
        Ok(PaperVenueRealismLedgeredExecution {
            execution,
            reserve_entry,
            settlement_entry,
        })
    }

    /// Submit a venue-realistic local paper fill, mutate the local paper ledger,
    /// and append the report plus ledger mutations to the local audit journal.
    pub fn submit_with_venue_realism_ledger_and_audit(
        &self,
        request: &PaperVenueRealismRequest,
        ledger: &mut PaperBalanceLedger,
        journal: &mut AppendOnlyAuditJournal,
    ) -> Result<PaperAuditedVenueRealismLedgeredExecution, PaperConnectorError> {
        let intent_record = append_paper_execution_intent_audit(
            journal,
            &request.fill_request.intent,
            request.fill_request.now_unix_ms,
        )?;
        let ledgered_execution = self.submit_with_venue_realism_and_ledger(request, ledger)?;
        let plain_ledgered_execution = PaperLedgeredExecution {
            report: ledgered_execution.execution.report.clone(),
            reserve_entry: ledgered_execution.reserve_entry.clone(),
            settlement_entry: ledgered_execution.settlement_entry.clone(),
        };
        let audit_report = append_paper_ledgered_execution_audit(
            journal,
            &plain_ledgered_execution,
            request.fill_request.now_unix_ms.saturating_add(2),
            Some(intent_record.sequence),
        )?;
        Ok(PaperAuditedVenueRealismLedgeredExecution {
            ledgered_execution,
            audit_report,
        })
    }

    /// Execute a local-only historical-fixture backtest corpus.
    ///
    /// Corpus execution uses caller-supplied paper fixtures only. It does not
    /// download data, use live networks, submit orders, or mutate real balances.
    pub fn run_backtest_corpus(
        &self,
        corpus: &PaperBacktestCorpus,
        now_unix_ms: u64,
    ) -> Result<PaperBacktestRunReport, PaperConnectorError> {
        run_paper_backtest_corpus(self, corpus, now_unix_ms)
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

/// Local paper matching profile for one venue.
///
/// The profile models exchange-specific constraints without contacting the
/// exchange. It is caller-supplied and must not be treated as live venue proof.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperExchangeMatchingProfile {
    /// Paper venue the profile applies to.
    pub venue: VenueRef,
    /// Minimum price increment in quote units.
    pub price_tick_quote: f64,
    /// Minimum base-quantity increment.
    pub quantity_step_base: f64,
    /// Minimum accepted order notional in quote units.
    pub min_notional_quote: f64,
    /// Optional maximum accepted order notional in quote units.
    pub max_notional_quote: Option<f64>,
    /// Whether marketable orders are supported by the modeled venue profile.
    pub supports_market_orders: bool,
    /// Whether limit orders are supported by the modeled venue profile.
    pub supports_limit_orders: bool,
    /// Whether post-only behavior is supported by the modeled venue profile.
    pub supports_post_only: bool,
    /// Whether partial fills are supported by the modeled venue profile.
    pub supports_partial_fills: bool,
    /// Maker queue-position haircut in basis points.
    pub maker_queue_position_bps: u16,
    /// Taker queue-position haircut in basis points.
    pub taker_queue_position_bps: u16,
    /// Whether future production use requires sandbox/live calibration evidence.
    pub sandbox_live_calibration_required: bool,
}

impl PaperExchangeMatchingProfile {
    /// Validate local venue matching constraints.
    pub fn validate(&self) -> Result<(), PaperConnectorError> {
        if self.venue.name.trim().is_empty() {
            return Err(PaperConnectorError::InvalidMatchingProfile {
                reason: "matching profile venue is required".to_owned(),
            });
        }
        validate_positive_finite("price tick", self.price_tick_quote)?;
        validate_positive_finite("quantity step", self.quantity_step_base)?;
        validate_non_negative_finite("minimum notional", self.min_notional_quote)?;
        if let Some(max_notional_quote) = self.max_notional_quote {
            validate_positive_finite("maximum notional", max_notional_quote)?;
            if max_notional_quote + f64::EPSILON < self.min_notional_quote {
                return Err(PaperConnectorError::InvalidMatchingProfile {
                    reason: "maximum notional must be at least minimum notional".to_owned(),
                });
            }
        }
        if self.maker_queue_position_bps > 10_000 || self.taker_queue_position_bps > 10_000 {
            return Err(PaperConnectorError::InvalidMatchingProfile {
                reason: "queue-position basis points must be at most 10000".to_owned(),
            });
        }
        if !self.supports_market_orders && !self.supports_limit_orders {
            return Err(PaperConnectorError::InvalidMatchingProfile {
                reason: "matching profile must support at least one order type".to_owned(),
            });
        }
        Ok(())
    }
}

/// Result of applying a local venue matching profile to a fill request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperMatchingProfileReport {
    /// Stable validation version.
    pub validation_version: String,
    /// Venue that was modeled.
    pub venue: VenueRef,
    /// Requested notional in quote units.
    pub requested_notional_quote: f64,
    /// Minimum venue notional.
    pub min_notional_quote: f64,
    /// Optional maximum venue notional.
    pub max_notional_quote: Option<f64>,
    /// Original number of levels on the consumed book side.
    pub original_levels: usize,
    /// Number of exchange-adjusted levels retained after tick/step rounding.
    pub exchange_adjusted_levels: usize,
    /// Whether the profile permits partial fills.
    pub partial_fills_supported: bool,
    /// Queue-position haircut applied after merging request and venue assumptions.
    pub applied_queue_position_bps: u16,
    /// Whether the request passed local matching-profile validation.
    pub accepted: bool,
    /// Non-secret explanation of the local matching outcome.
    pub reason: String,
}

/// Deterministic adverse-selection settings for paper fills.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAdverseSelectionConfig {
    /// Whether adverse-selection penalty modeling is enabled.
    pub enabled: bool,
    /// Penalty basis points per 100ms of modeled latency.
    pub latency_penalty_bps_per_100ms: f64,
    /// Local volatility penalty in basis points.
    pub volatility_penalty_bps: f64,
    /// Local order-book imbalance penalty in basis points.
    pub order_book_imbalance_penalty_bps: f64,
    /// Maximum adverse-selection penalty in basis points.
    pub max_penalty_bps: f64,
}

impl Default for PaperAdverseSelectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            latency_penalty_bps_per_100ms: 0.0,
            volatility_penalty_bps: 0.0,
            order_book_imbalance_penalty_bps: 0.0,
            max_penalty_bps: 0.0,
        }
    }
}

impl PaperAdverseSelectionConfig {
    /// Validate local adverse-selection settings.
    pub fn validate(&self) -> Result<(), PaperConnectorError> {
        validate_non_negative_finite(
            "latency adverse-selection penalty",
            self.latency_penalty_bps_per_100ms,
        )?;
        validate_non_negative_finite(
            "volatility adverse-selection penalty",
            self.volatility_penalty_bps,
        )?;
        validate_non_negative_finite(
            "order-book imbalance adverse-selection penalty",
            self.order_book_imbalance_penalty_bps,
        )?;
        validate_non_negative_finite("maximum adverse-selection penalty", self.max_penalty_bps)?;
        Ok(())
    }
}

/// Deterministic adverse-selection report for one paper fill.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAdverseSelectionReport {
    /// Stable validation version.
    pub validation_version: String,
    /// Whether the model was enabled.
    pub enabled: bool,
    /// Fill latency used for the penalty.
    pub latency_ms: u64,
    /// Applied penalty in basis points.
    pub penalty_bps: f64,
    /// Applied penalty in quote units.
    pub penalty_quote: f64,
    /// Net paper profit after this penalty only.
    pub net_profit_after_penalty_quote: f64,
    /// Non-secret explanation of the local adverse-selection outcome.
    pub reason: String,
}

/// Reference-only sandbox/live discrepancy calibration record.
///
/// This record may reference external evidence by name or URL, but it must not
/// embed credential-bearing logs, dependency tables, secret snippets, or raw
/// private evidence contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperVenueCalibrationRecord {
    /// Stable calibration record id.
    pub calibration_id: String,
    /// Venue the calibration applies to.
    pub venue: VenueRef,
    /// Number of observations summarized by the record.
    pub sample_count: usize,
    /// Average local paper slippage in basis points.
    pub paper_slippage_bps: f64,
    /// Average sandbox slippage in basis points.
    pub sandbox_slippage_bps: f64,
    /// Optional average live slippage in basis points.
    pub live_slippage_bps: Option<f64>,
    /// Optional non-secret evidence locator.
    pub evidence_reference: Option<String>,
    /// Whether external observation evidence was actually available to the caller.
    pub external_observation_available: bool,
    /// Must remain false; calibration records store references only.
    pub secret_material_recorded: bool,
}

impl PaperVenueCalibrationRecord {
    /// Validate a reference-only calibration record.
    pub fn validate(&self, venue: &VenueRef) -> Result<(), PaperConnectorError> {
        if self.calibration_id.trim().is_empty() {
            return Err(PaperConnectorError::InvalidCalibrationRecord {
                reason: "calibration id is required".to_owned(),
            });
        }
        if !same_venue(&self.venue, venue) {
            return Err(PaperConnectorError::InvalidCalibrationRecord {
                reason: "calibration venue must match execution venue".to_owned(),
            });
        }
        if self.sample_count == 0 {
            return Err(PaperConnectorError::InvalidCalibrationRecord {
                reason: "calibration sample count must be non-zero".to_owned(),
            });
        }
        validate_non_negative_finite("paper slippage", self.paper_slippage_bps)?;
        validate_non_negative_finite("sandbox slippage", self.sandbox_slippage_bps)?;
        if let Some(live_slippage_bps) = self.live_slippage_bps {
            validate_non_negative_finite("live slippage", live_slippage_bps)?;
        }
        if self.secret_material_recorded {
            return Err(PaperConnectorError::InvalidCalibrationRecord {
                reason: "calibration records must not embed secret material".to_owned(),
            });
        }
        validate_non_secret_reference(
            "calibration evidence reference",
            self.evidence_reference.as_deref(),
        )?;
        Ok(())
    }
}

/// Applied calibration result for one paper fill.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperCalibrationApplicationReport {
    /// Stable validation version.
    pub validation_version: String,
    /// Calibration id that was applied.
    pub calibration_id: String,
    /// Modeled sandbox/live discrepancy in basis points.
    pub discrepancy_bps: f64,
    /// Applied P&L penalty in quote units.
    pub penalty_quote: f64,
    /// Net paper profit after adverse-selection and calibration penalties.
    pub net_profit_after_penalty_quote: f64,
    /// Whether external evidence was available to the caller.
    pub external_observation_available: bool,
    /// Optional non-secret evidence locator.
    pub evidence_reference: Option<String>,
    /// Non-secret explanation of the local calibration outcome.
    pub reason: String,
}

/// Request for a venue-realism paper execution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperVenueRealismRequest {
    /// Base realistic fill request.
    pub fill_request: PaperFillSimulationRequest,
    /// Local exchange-specific matching profile.
    pub matching_profile: PaperExchangeMatchingProfile,
    /// Local adverse-selection configuration.
    pub adverse_selection: PaperAdverseSelectionConfig,
    /// Optional reference-only calibration record.
    pub calibration: Option<PaperVenueCalibrationRecord>,
}

impl PaperVenueRealismRequest {
    /// Validate local venue-realism inputs.
    pub fn validate(&self) -> Result<(), PaperConnectorError> {
        self.fill_request.validate()?;
        self.matching_profile.validate()?;
        self.adverse_selection.validate()?;
        if !same_venue(
            &self.fill_request.intent.venue,
            &self.matching_profile.venue,
        ) {
            return Err(PaperConnectorError::InvalidMatchingProfile {
                reason: "matching profile venue must match fill request venue".to_owned(),
            });
        }
        if self.fill_request.intent.notional_quote + f64::EPSILON
            < self.matching_profile.min_notional_quote
        {
            return Err(PaperConnectorError::InvalidMatchingProfile {
                reason: "fill request notional is below venue minimum".to_owned(),
            });
        }
        if let Some(max_notional_quote) = self.matching_profile.max_notional_quote {
            if self.fill_request.intent.notional_quote > max_notional_quote + f64::EPSILON {
                return Err(PaperConnectorError::InvalidMatchingProfile {
                    reason: "fill request notional exceeds venue maximum".to_owned(),
                });
            }
        }
        if !self.matching_profile.supports_market_orders {
            return Err(PaperConnectorError::InvalidMatchingProfile {
                reason: "venue profile does not support marketable paper fills".to_owned(),
            });
        }
        if let Some(record) = &self.calibration {
            record.validate(&self.fill_request.intent.venue)?;
        }
        Ok(())
    }
}

/// Venue-realism execution report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperVenueRealismExecution {
    /// Adjusted deterministic paper execution report.
    pub report: PaperExecutionReport,
    /// Matching-profile application report.
    pub matching: PaperMatchingProfileReport,
    /// Adverse-selection report.
    pub adverse_selection: PaperAdverseSelectionReport,
    /// Optional calibration application report.
    pub calibration: Option<PaperCalibrationApplicationReport>,
}

/// Venue-realism execution plus ledger entries.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperVenueRealismLedgeredExecution {
    /// Venue-realism execution report.
    pub execution: PaperVenueRealismExecution,
    /// Ledger entry that reserved quote notional.
    pub reserve_entry: PaperLedgerEntry,
    /// Ledger entry that settled the adjusted modeled fill.
    pub settlement_entry: PaperLedgerEntry,
}

/// Replay validation status for paper ledger entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaperReplayValidationStatus {
    /// Replay balanced against the final ledger state.
    Passed,
    /// Replay found at least one ledger inconsistency.
    Failed,
}

/// One paper replay violation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperReplayViolation {
    /// Ledger entry id or logical location.
    pub entry_id: String,
    /// Stable non-secret violation code.
    pub code: String,
    /// Human-readable non-secret message.
    pub message: String,
}

/// Paper audit/replay validation report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAuditReplayValidationReport {
    /// Stable validation version.
    pub validation_version: String,
    /// Replay status.
    pub status: PaperReplayValidationStatus,
    /// Number of ledger entries replayed.
    pub entries_replayed: usize,
    /// Number of reserve entries replayed.
    pub reserve_entries: usize,
    /// Number of settlement entries replayed.
    pub settlement_entries: usize,
    /// Whether replayed balances match the ledger's final balances.
    pub final_balances_match: bool,
    /// Whether no non-zero reservations remain.
    pub reservations_closed: bool,
    /// Whether all replay invariants passed.
    pub balanced: bool,
    /// Final balances reconstructed by replay.
    pub replayed_final_balances: Vec<PaperAssetBalance>,
    /// Non-secret replay violations.
    pub violations: Vec<PaperReplayViolation>,
    /// Runtime clock in Unix milliseconds.
    pub replayed_at_unix_ms: u64,
    /// Direct audit journal integration remains future work.
    pub direct_audit_journal_integrated: bool,
}

/// One paper backtest step over local fixtures.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperBacktestStep {
    /// Stable step id.
    pub step_id: String,
    /// Local fill request fixture.
    pub fill_request: PaperFillSimulationRequest,
    /// Optional venue matching profile.
    pub matching_profile: Option<PaperExchangeMatchingProfile>,
    /// Optional adverse-selection config.
    pub adverse_selection: Option<PaperAdverseSelectionConfig>,
    /// Optional reference-only calibration record.
    pub calibration: Option<PaperVenueCalibrationRecord>,
}

/// One local historical-fixture paper backtest scenario.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperBacktestScenario {
    /// Stable scenario id.
    pub scenario_id: String,
    /// Initial paper balances for the scenario.
    pub initial_balances: Vec<PaperAssetBalance>,
    /// Deterministic steps to execute.
    pub steps: Vec<PaperBacktestStep>,
}

/// Local-only paper backtest corpus.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperBacktestCorpus {
    /// Stable corpus id.
    pub corpus_id: String,
    /// Whether fixtures are recorded historical local fixtures rather than live downloads.
    pub historical_fixture_replay: bool,
    /// Whether all fixture material is local only.
    pub local_fixture_only: bool,
    /// Must remain false; this runner never downloads market data.
    pub external_data_downloaded: bool,
    /// Scenarios to execute.
    pub scenarios: Vec<PaperBacktestScenario>,
}

/// Backtest report for one scenario.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperBacktestScenarioReport {
    /// Scenario id.
    pub scenario_id: String,
    /// Number of executed steps.
    pub steps_executed: usize,
    /// Filled step count.
    pub filled_steps: usize,
    /// Partially filled step count.
    pub partially_filled_steps: usize,
    /// Unfilled step count.
    pub unfilled_steps: usize,
    /// Aggregate modeled net paper P&L in quote units.
    pub net_profit_quote: f64,
    /// Final local paper balances.
    pub final_balances: Vec<PaperAssetBalance>,
    /// Replay report for the scenario ledger.
    pub replay_validation: PaperAuditReplayValidationReport,
}

/// Backtest corpus execution report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperBacktestRunReport {
    /// Stable validation version.
    pub validation_version: String,
    /// Corpus id.
    pub corpus_id: String,
    /// Number of executed scenarios.
    pub scenarios_executed: usize,
    /// Number of executed steps.
    pub total_steps: usize,
    /// Filled step count.
    pub filled_steps: usize,
    /// Partially filled step count.
    pub partially_filled_steps: usize,
    /// Unfilled step count.
    pub unfilled_steps: usize,
    /// Aggregate modeled net paper P&L in quote units.
    pub net_profit_quote: f64,
    /// Whether this was historical fixture replay.
    pub historical_fixture_replay: bool,
    /// Whether all fixtures were local.
    pub local_fixture_only: bool,
    /// Whether external data was downloaded.
    pub external_data_downloaded: bool,
    /// Always false for this local runner.
    pub live_network_used: bool,
    /// Always false for this local runner.
    pub external_execution_performed: bool,
    /// Whether every scenario replay report balanced.
    pub replay_validated: bool,
    /// Scenario reports.
    pub scenario_reports: Vec<PaperBacktestScenarioReport>,
    /// Runtime clock in Unix milliseconds.
    pub executed_at_unix_ms: u64,
}

/// Runtime validation request over local paper replay/backtest evidence.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperRuntimeValidationRequest {
    /// Local ledger replay validation.
    pub replay_validation: PaperAuditReplayValidationReport,
    /// Local backtest corpus execution report.
    pub backtest_report: PaperBacktestRunReport,
    /// Sanitized host or environment label.
    pub runtime_host_label: String,
    /// Optional non-secret production-host evidence reference.
    pub production_host_evidence_reference: Option<String>,
    /// Whether an external production runtime validation was actually performed.
    pub external_runtime_validation_performed: bool,
    /// Must remain false for this local paper validation boundary.
    pub live_network_used: bool,
    /// Must remain false for this local paper validation boundary.
    pub external_execution_performed: bool,
    /// Runtime clock in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Runtime validation report that preserves production blockers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperRuntimeValidationReport {
    /// Stable validation version.
    pub validation_version: String,
    /// Sanitized host or environment label.
    pub runtime_host_label: String,
    /// Whether local replay validation passed.
    pub local_replay_validated: bool,
    /// Whether local backtest execution passed.
    pub local_backtest_executed: bool,
    /// Whether local runtime evidence passed without live side effects.
    pub local_runtime_validation_passed: bool,
    /// Whether a production-host evidence locator was supplied.
    pub production_host_evidence_referenced: bool,
    /// Whether external production runtime validation was recorded by the caller.
    pub external_runtime_validation_recorded: bool,
    /// This boundary alone never approves production readiness.
    pub production_ready: bool,
    /// Whether live network access was used.
    pub live_network_used: bool,
    /// Whether external execution was performed.
    pub external_execution_performed: bool,
    /// Remaining blockers in non-secret wording.
    pub unresolved_blockers: Vec<String>,
    /// Runtime clock in Unix milliseconds.
    pub validated_at_unix_ms: u64,
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

    /// Replay the paper ledger and verify final balances are reproducible.
    ///
    /// This is a local audit/replay validation for paper balances only. It does
    /// not read or write the append-only audit journal and does not validate any
    /// production deployment host.
    pub fn validate_replay(
        &self,
        replayed_at_unix_ms: u64,
    ) -> Result<PaperAuditReplayValidationReport, PaperConnectorError> {
        validate_paper_ledger_replay(self, replayed_at_unix_ms)
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

/// Audit journal append summary for one ledgered paper execution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAuditJournalWriteReport {
    /// Audit integration version.
    pub integration_version: String,
    /// Paper execution report id.
    pub execution_report_id: String,
    /// Source execution intent id.
    pub intent_id: String,
    /// Audit record sequence for the pre-execution paper intent audit, when recorded.
    pub intent_record_sequence: Option<u64>,
    /// Audit record sequence for the reserve ledger mutation.
    pub reserve_record_sequence: u64,
    /// Audit record sequence for the settlement ledger mutation.
    pub settlement_record_sequence: u64,
    /// Audit record sequence for the paper execution report.
    pub report_record_sequence: u64,
    /// Number of audit records appended by this operation.
    pub records_appended: usize,
    /// Whether reopening the journal replayed the appended hash chain.
    pub journal_replay_verified: bool,
    /// Whether live network access was used by this local integration.
    pub live_network_used: bool,
    /// Whether external execution was performed by this local integration.
    pub external_execution_performed: bool,
    /// Runtime clock in Unix milliseconds.
    pub audited_at_unix_ms: u64,
}

/// Local adapter-run paper ledgering summary.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAdapterRunLedgerReport {
    /// Source execution-adapter run id.
    pub adapter_run_id: String,
    /// Source execution-plan id.
    pub plan_id: String,
    /// Number of modeled paper fills settled into the ledger.
    pub modeled_fills_settled: usize,
    /// Number of adapter reconciliation records replayed against modeled fills.
    pub reconciliations_replayed: usize,
    /// Number of ledger entries written by this settlement.
    pub ledger_entries_written: usize,
    /// Number of paper ledger audit records appended.
    pub audit_records_appended: usize,
    /// Whether a final paper ledger checkpoint was persisted.
    pub ledger_checkpoint_persisted: bool,
    /// State-store key for the final paper ledger checkpoint.
    pub ledger_checkpoint_key: String,
    /// Whether external adapter submission occurred. Always false here.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local report claims production readiness. Always false.
    pub production_ready: bool,
}

/// Paper ledgered execution paired with append-only audit journal records.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAuditedLedgeredExecution {
    /// Local deterministic ledgered execution.
    pub ledgered_execution: PaperLedgeredExecution,
    /// Audit journal append summary.
    pub audit_report: PaperAuditJournalWriteReport,
}

/// Venue-realistic paper ledgered execution paired with audit journal records.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaperAuditedVenueRealismLedgeredExecution {
    /// Local deterministic venue-realistic ledgered execution.
    pub ledgered_execution: PaperVenueRealismLedgeredExecution,
    /// Audit journal append summary.
    pub audit_report: PaperAuditJournalWriteReport,
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
    /// Deterministic paper P&L haircut from adverse-selection assumptions.
    pub adverse_selection_quote: f64,
    /// Deterministic paper P&L haircut from venue calibration discrepancy records.
    pub calibration_adjustment_quote: f64,
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

/// Append one local paper execution report to the append-only audit journal.
///
/// This records sanitized identifiers and modeled paper amounts only. It does
/// not submit externally, call a venue, sign, broadcast, or record secrets.
pub fn append_paper_execution_report_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &PaperExecutionReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, PaperConnectorError> {
    validate_audit_timestamp(occurred_at_unix_ms)?;
    let mut event = AuditEvent::new(
        format!("paper-report-audit-{}", report.id),
        AuditEventKind::ExecutionResult,
        PAPER_EXECUTION_STATE_SUBSYSTEM,
        "paper-execution-adapter",
        "paper execution report recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "paper_audit_integration_version",
            AuditValue::Text(PAPER_AUDIT_INTEGRATION_VERSION.to_owned()),
        )
        .with_metadata("report_id", AuditValue::Text(report.id.clone()))
        .with_metadata("intent_id", AuditValue::Text(report.intent_id.clone()))
        .with_metadata("strategy_id", AuditValue::Text(report.strategy_id.clone()))
        .with_metadata("adapter", AuditValue::Text(report.adapter.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("base_asset", AuditValue::Text(report.base_asset.clone()))
        .with_metadata("quote_asset", AuditValue::Text(report.quote_asset.clone()))
        .with_metadata("scope", AuditValue::Text(format!("{:?}", report.scope)))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "notional_quote",
            AuditValue::Text(format_audit_amount(report.notional_quote)),
        )
        .with_metadata(
            "filled_notional_quote",
            AuditValue::Text(format_audit_amount(report.filled_notional_quote)),
        )
        .with_metadata(
            "unfilled_notional_quote",
            AuditValue::Text(format_audit_amount(report.unfilled_notional_quote)),
        )
        .with_metadata(
            "net_profit_quote",
            AuditValue::Text(format_audit_amount(report.net_profit_quote)),
        )
        .with_metadata(
            "connector_version",
            AuditValue::Text(report.connector_version.clone()),
        )
        .with_metadata(
            "fill_model_version",
            AuditValue::Text(report.fill_model_version.clone()),
        )
        .with_metadata("live_network_used", AuditValue::Bool(false))
        .with_metadata("external_execution_performed", AuditValue::Bool(false));
    journal
        .append_event(event)
        .map_err(|source| PaperConnectorError::AuditJournalFailed {
            reason: source.to_string(),
        })
}

/// Append one local paper execution intent to the append-only audit journal.
///
/// This records sanitized intent metadata before local modeled execution. It
/// does not call a venue, sign payloads, broadcast transactions, mutate real
/// balances, or load secrets.
pub fn append_paper_execution_intent_audit(
    journal: &mut AppendOnlyAuditJournal,
    intent: &ExecutionIntent,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, PaperConnectorError> {
    validate_audit_timestamp(occurred_at_unix_ms)?;
    let mut event = AuditEvent::new(
        format!("paper-intent-audit-{}", intent.id),
        AuditEventKind::IntentLifecycle,
        PAPER_EXECUTION_STATE_SUBSYSTEM,
        "paper-execution-adapter",
        "paper execution intent recorded before local modeling",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "paper_audit_integration_version",
            AuditValue::Text(PAPER_AUDIT_INTEGRATION_VERSION.to_owned()),
        )
        .with_metadata("intent_id", AuditValue::Text(intent.id.clone()))
        .with_metadata("strategy_id", AuditValue::Text(intent.strategy_id.clone()))
        .with_metadata(
            "intent_kind",
            AuditValue::Text(format!("{:?}", intent.kind)),
        )
        .with_metadata("scope", AuditValue::Text(format!("{:?}", intent.scope)))
        .with_metadata("venue", AuditValue::Text(intent.venue.name.clone()))
        .with_metadata("base_asset", AuditValue::Text(intent.base_asset.clone()))
        .with_metadata("quote_asset", AuditValue::Text(intent.quote_asset.clone()))
        .with_metadata(
            "notional_quote",
            AuditValue::Text(format_audit_amount(intent.notional_quote)),
        )
        .with_metadata(
            "expected_profit_quote",
            AuditValue::Text(format_audit_amount(intent.expected_profit_quote)),
        )
        .with_metadata(
            "max_loss_quote",
            AuditValue::Text(format_audit_amount(intent.max_loss_quote)),
        )
        .with_metadata(
            "slippage_bps",
            AuditValue::Unsigned(u64::from(intent.slippage_bps)),
        )
        .with_metadata(
            "estimated_fee_quote",
            AuditValue::Text(format_audit_amount(intent.estimated_fee_quote)),
        )
        .with_metadata(
            "gas_fee_quote",
            AuditValue::Text(format_audit_amount(intent.gas_fee_quote)),
        )
        .with_metadata(
            "market_data_age_ms",
            AuditValue::Unsigned(intent.market_data_age_ms),
        )
        .with_metadata(
            "destination",
            AuditValue::Text(format!("{:?}", intent.destination)),
        )
        .with_metadata(
            "requires_signing",
            AuditValue::Bool(intent.requires_signing),
        )
        .with_metadata("live_network_used", AuditValue::Bool(false))
        .with_metadata("external_execution_performed", AuditValue::Bool(false));
    if let Some(chain) = &intent.chain {
        event = event.with_metadata("chain", AuditValue::Text(chain.clone()));
    }
    journal
        .append_event(event)
        .map_err(|source| PaperConnectorError::AuditJournalFailed {
            reason: source.to_string(),
        })
}

/// Append one local paper ledger mutation to the append-only audit journal.
///
/// This records modeled paper balance deltas only and never reads or mutates
/// real account balances.
pub fn append_paper_ledger_entry_audit(
    journal: &mut AppendOnlyAuditJournal,
    entry: &PaperLedgerEntry,
) -> Result<AuditRecord, PaperConnectorError> {
    validate_audit_timestamp(entry.recorded_at_unix_ms)?;
    let mut event = AuditEvent::new(
        format!("paper-ledger-audit-{}", entry.id),
        AuditEventKind::Reconciliation,
        PAPER_EXECUTION_STATE_SUBSYSTEM,
        "paper-balance-ledger",
        "paper ledger mutation recorded",
    );
    event.occurred_at_unix_ms = entry.recorded_at_unix_ms;
    event = event
        .with_metadata(
            "paper_audit_integration_version",
            AuditValue::Text(PAPER_AUDIT_INTEGRATION_VERSION.to_owned()),
        )
        .with_metadata("ledger_entry_id", AuditValue::Text(entry.id.clone()))
        .with_metadata(
            "ledger_entry_kind",
            AuditValue::Text(format!("{:?}", entry.kind)),
        )
        .with_metadata("venue", AuditValue::Text(entry.venue.name.clone()))
        .with_metadata("asset", AuditValue::Text(entry.asset.clone()))
        .with_metadata(
            "available_delta",
            AuditValue::Text(format_audit_amount(entry.available_delta)),
        )
        .with_metadata(
            "reserved_delta",
            AuditValue::Text(format_audit_amount(entry.reserved_delta)),
        )
        .with_metadata(
            "resulting_available",
            AuditValue::Text(format_audit_amount(entry.resulting_available)),
        )
        .with_metadata(
            "resulting_reserved",
            AuditValue::Text(format_audit_amount(entry.resulting_reserved)),
        )
        .with_metadata("live_network_used", AuditValue::Bool(false))
        .with_metadata("external_execution_performed", AuditValue::Bool(false));
    if let Some(intent_id) = &entry.intent_id {
        event = event.with_metadata("intent_id", AuditValue::Text(intent_id.clone()));
    }
    if let Some(report_id) = &entry.report_id {
        event = event.with_metadata("report_id", AuditValue::Text(report_id.clone()));
    }
    journal
        .append_event(event)
        .map_err(|source| PaperConnectorError::AuditJournalFailed {
            reason: source.to_string(),
        })
}

/// Append audit records for both paper ledger mutations and the resulting
/// paper execution report, then reopen the journal to verify local replay.
pub fn append_paper_ledgered_execution_audit(
    journal: &mut AppendOnlyAuditJournal,
    execution: &PaperLedgeredExecution,
    audited_at_unix_ms: u64,
    intent_record_sequence: Option<u64>,
) -> Result<PaperAuditJournalWriteReport, PaperConnectorError> {
    validate_audit_timestamp(audited_at_unix_ms)?;
    let reserve_record = append_paper_ledger_entry_audit(journal, &execution.reserve_entry)?;
    let settlement_record = append_paper_ledger_entry_audit(journal, &execution.settlement_entry)?;
    let report_record =
        append_paper_execution_report_audit(journal, &execution.report, audited_at_unix_ms)?;
    let reopened = AppendOnlyAuditJournal::open(journal.path()).map_err(|source| {
        PaperConnectorError::AuditJournalFailed {
            reason: source.to_string(),
        }
    })?;
    let journal_replay_verified = reopened.next_sequence() == journal.next_sequence()
        && reopened.previous_hash() == journal.previous_hash();

    Ok(PaperAuditJournalWriteReport {
        integration_version: PAPER_AUDIT_INTEGRATION_VERSION.to_owned(),
        execution_report_id: execution.report.id.clone(),
        intent_id: execution.report.intent_id.clone(),
        intent_record_sequence,
        reserve_record_sequence: reserve_record.sequence,
        settlement_record_sequence: settlement_record.sequence,
        report_record_sequence: report_record.sequence,
        records_appended: 3 + usize::from(intent_record_sequence.is_some()),
        journal_replay_verified,
        live_network_used: false,
        external_execution_performed: false,
        audited_at_unix_ms,
    })
}

/// Settle local modeled execution-adapter fills into a paper balance ledger.
///
/// This helper accepts only paper-scope deterministic adapter records. It writes
/// local paper ledger audit records and a final ledger checkpoint, and never
/// submits orders, calls exchanges/RPCs, signs payloads, broadcasts
/// transactions, withdraws funds, bridges assets, or claims production
/// readiness.
pub fn ledger_execution_adapter_run_paper_fills(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    ledger: &mut PaperBalanceLedger,
    plan: &ExecutionPlanDraft,
    run: &ExecutionAdapterRunRecord,
    settled_at_unix_ms: u64,
) -> Result<PaperAdapterRunLedgerReport, PaperConnectorError> {
    validate_adapter_run_ledger_inputs(plan, run, settled_at_unix_ms)?;

    let mut modeled_fills_settled = 0usize;
    let mut reconciliations_replayed = 0usize;
    let mut ledger_entries_written = 0usize;
    let mut audit_records_appended = 0usize;

    for fill in run
        .fills
        .iter()
        .filter(|fill| fill.status == ExecutionFillStatus::ModeledFilled)
    {
        let report_id = adapter_fill_paper_report_id(run, fill);
        if ledger
            .entries
            .iter()
            .any(|entry| entry.report_id.as_deref() == Some(report_id.as_str()))
        {
            return Err(PaperConnectorError::InvalidPaperLedger {
                reason: format!(
                    "adapter-run paper ledger refuses duplicate modeled fill report {report_id}"
                ),
            });
        }
    }

    for fill in run
        .fills
        .iter()
        .filter(|fill| fill.status == ExecutionFillStatus::ModeledFilled)
    {
        let intent = plan_intent_for_fill(plan, fill)?;
        validate_reconciliation_for_fill(run, fill)?;
        let intent_record =
            append_paper_execution_intent_audit(journal, intent, settled_at_unix_ms)?;
        let reserve_entry = ledger.reserve_for_intent(intent, settled_at_unix_ms)?;
        let report = adapter_fill_paper_report(plan, run, intent, fill)?;
        let settlement_entry =
            ledger.settle_report(&report, settled_at_unix_ms.saturating_add(1))?;
        let audit_report = append_paper_ledgered_execution_audit(
            journal,
            &PaperLedgeredExecution {
                report,
                reserve_entry,
                settlement_entry,
            },
            settled_at_unix_ms.saturating_add(2),
            Some(intent_record.sequence),
        )?;
        modeled_fills_settled = modeled_fills_settled.saturating_add(1);
        reconciliations_replayed = reconciliations_replayed.saturating_add(1);
        ledger_entries_written = ledger_entries_written.saturating_add(2);
        audit_records_appended =
            audit_records_appended.saturating_add(audit_report.records_appended);
    }

    if modeled_fills_settled == 0 {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "adapter run did not contain modeled paper fills to ledger".to_owned(),
        });
    }

    let checkpoint = persist_paper_balance_ledger_checkpoint(
        store,
        ledger,
        settled_at_unix_ms.saturating_add(3),
    )
    .map_err(|source| PaperConnectorError::InvalidPaperLedger {
        reason: format!("adapter-run paper ledger checkpoint failed: {source}"),
    })?;

    Ok(PaperAdapterRunLedgerReport {
        adapter_run_id: run.id.clone(),
        plan_id: plan.id.clone(),
        modeled_fills_settled,
        reconciliations_replayed,
        ledger_entries_written,
        audit_records_appended,
        ledger_checkpoint_persisted: true,
        ledger_checkpoint_key: checkpoint.key,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    })
}

fn adapter_fill_paper_report_id(
    run: &ExecutionAdapterRunRecord,
    fill: &ExecutionFillRecord,
) -> String {
    format!("paper-adapter-run:{}:{}", run.id, fill.id)
}

fn validate_reconciliation_for_fill(
    run: &ExecutionAdapterRunRecord,
    fill: &ExecutionFillRecord,
) -> Result<(), PaperConnectorError> {
    let reconciliation = run
        .reconciliations
        .iter()
        .find(|reconciliation| reconciliation.intent_id == fill.intent_id)
        .ok_or_else(|| PaperConnectorError::InvalidPaperLedger {
            reason: format!(
                "adapter-run paper ledger fill {} has no matching reconciliation",
                fill.id
            ),
        })?;
    if reconciliation.status != ExecutionReconciliationStatus::Reconciled {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: format!(
                "adapter-run paper ledger fill {} requires reconciled status",
                fill.id
            ),
        });
    }
    if (reconciliation.observed_notional_quote - fill.filled_notional_quote).abs() > 0.000_000_1 {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: format!(
                "adapter-run paper ledger fill {} does not match reconciled notional",
                fill.id
            ),
        });
    }
    if reconciliation.difference_quote.abs() > 0.000_000_1 {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: format!(
                "adapter-run paper ledger fill {} has non-zero reconciliation difference",
                fill.id
            ),
        });
    }
    Ok(())
}

fn validate_adapter_run_ledger_inputs(
    plan: &ExecutionPlanDraft,
    run: &ExecutionAdapterRunRecord,
    settled_at_unix_ms: u64,
) -> Result<(), PaperConnectorError> {
    if settled_at_unix_ms == 0 {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "adapter-run paper ledger timestamp must be non-zero".to_owned(),
        });
    }
    plan.validate()
        .map_err(|error| PaperConnectorError::InvalidPaperLedger {
            reason: format!("adapter-run paper ledger plan validation failed: {error}"),
        })?;
    run.validate()
        .map_err(|error| PaperConnectorError::InvalidPaperLedger {
            reason: format!("adapter-run paper ledger run validation failed: {error}"),
        })?;
    if plan.scope != ExecutionScope::Paper || run.scope != ExecutionScope::Paper {
        return Err(PaperConnectorError::NonPaperScope { scope: run.scope });
    }
    if run.plan_id != plan.id {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "adapter-run paper ledger requires matching plan id".to_owned(),
        });
    }
    if run.external_submission_enabled
        || run
            .attempts
            .iter()
            .any(|attempt| attempt.submitted_to_external_adapter)
    {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "adapter-run paper ledger refuses external submission records".to_owned(),
        });
    }
    Ok(())
}

fn plan_intent_for_fill<'a>(
    plan: &'a ExecutionPlanDraft,
    fill: &ExecutionFillRecord,
) -> Result<&'a ExecutionIntent, PaperConnectorError> {
    plan.intents
        .iter()
        .find(|intent| intent.id == fill.intent_id)
        .ok_or_else(|| PaperConnectorError::InvalidPaperLedger {
            reason: format!(
                "adapter-run paper ledger fill {} has no matching plan intent",
                fill.id
            ),
        })
}

fn adapter_fill_paper_report(
    plan: &ExecutionPlanDraft,
    run: &ExecutionAdapterRunRecord,
    intent: &ExecutionIntent,
    fill: &ExecutionFillRecord,
) -> Result<PaperExecutionReport, PaperConnectorError> {
    if fill.status != ExecutionFillStatus::ModeledFilled {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "adapter-run paper ledger accepts modeled fills only".to_owned(),
        });
    }
    if (fill.filled_notional_quote - intent.notional_quote).abs() > 0.000_000_1 {
        return Err(PaperConnectorError::InvalidPaperLedger {
            reason: "adapter-run modeled fill notional must match source intent notional"
                .to_owned(),
        });
    }
    let net_profit_quote = intent.expected_profit_quote - fill.fee_quote;
    let roi_bps = if intent.notional_quote.abs() <= f64::EPSILON {
        0.0
    } else {
        (net_profit_quote / intent.notional_quote) * 10_000.0
    };
    Ok(PaperExecutionReport {
        id: adapter_fill_paper_report_id(run, fill),
        adapter: run.adapter_id.clone(),
        connector_version: PAPER_CONNECTOR_VERSION.to_owned(),
        fill_model_version: PAPER_CONNECTOR_VERSION.to_owned(),
        intent_id: intent.id.clone(),
        strategy_id: intent.strategy_id.clone(),
        trust_contract_version: plan
            .policy_outcomes
            .iter()
            .find(|outcome| outcome.intent_id == intent.id)
            .and_then(|outcome| outcome.trust_contract_version.as_deref())
            .unwrap_or("phase-11-adapter-policy-revalidation")
            .to_owned(),
        status: PaperExecutionStatus::Filled,
        scope: intent.scope,
        venue: intent.venue.clone(),
        base_asset: intent.base_asset.clone(),
        quote_asset: intent.quote_asset.clone(),
        notional_quote: intent.notional_quote,
        filled_notional_quote: fill.filled_notional_quote,
        unfilled_notional_quote: 0.0,
        average_fill_price_quote: 0.0,
        gross_profit_quote: intent.expected_profit_quote,
        total_fees_quote: fill.fee_quote,
        adverse_selection_quote: 0.0,
        calibration_adjustment_quote: 0.0,
        net_profit_quote,
        roi_bps,
        liquidity_role: LiquidityRole::Taker,
        fill_simulation: PaperFillSimulationReport {
            fill_model_version: PAPER_CONNECTOR_VERSION.to_owned(),
            status: PaperFillSimulationStatus::Filled,
            side: PaperFillSide::BuyBase,
            requested_notional_quote: intent.notional_quote,
            filled_notional_quote: fill.filled_notional_quote,
            unfilled_notional_quote: 0.0,
            filled_base_quantity: 0.0,
            average_fill_price_quote: 0.0,
            reference_price_quote: 0.0,
            worst_fill_price_quote: 0.0,
            slippage_bps: f64::from(intent.slippage_bps),
            consumed_levels: 0,
            latency_ms: 0,
            queue_position_bps: 0,
            filled_at_unix_ms: fill.filled_at_unix_ms,
            reason: "local execution-adapter modeled fill settled into paper ledger".to_owned(),
        },
    })
}

/// Validate local paper replay/backtest evidence while preserving production blockers.
///
/// This records local validation status only. A production-ready result is never
/// returned by this function because deployment-host evidence, security review,
/// live/sandbox venue validation, and operator approval must happen outside this
/// local paper boundary.
pub fn validate_paper_runtime(
    request: &PaperRuntimeValidationRequest,
) -> Result<PaperRuntimeValidationReport, PaperConnectorError> {
    if request.runtime_host_label.trim().is_empty() {
        return Err(PaperConnectorError::InvalidRuntimeValidation {
            reason: "runtime host label is required".to_owned(),
        });
    }
    if request.validated_at_unix_ms == 0 {
        return Err(PaperConnectorError::InvalidRuntimeValidation {
            reason: "runtime validation timestamp must be non-zero".to_owned(),
        });
    }
    validate_non_secret_reference("runtime host label", Some(&request.runtime_host_label))?;
    validate_non_secret_reference(
        "production runtime evidence reference",
        request.production_host_evidence_reference.as_deref(),
    )?;

    let local_replay_validated = request.replay_validation.balanced;
    let local_backtest_executed = request.backtest_report.total_steps > 0
        && request.backtest_report.replay_validated
        && !request.backtest_report.live_network_used
        && !request.backtest_report.external_execution_performed;
    let local_runtime_validation_passed = local_replay_validated
        && local_backtest_executed
        && !request.live_network_used
        && !request.external_execution_performed;
    let production_host_evidence_referenced = request.production_host_evidence_reference.is_some();

    let mut unresolved_blockers = Vec::new();
    if !local_replay_validated {
        unresolved_blockers.push("local paper replay validation failed".to_owned());
    }
    if !local_backtest_executed {
        unresolved_blockers.push("local paper backtest corpus execution is incomplete".to_owned());
    }
    if !request.external_runtime_validation_performed {
        unresolved_blockers
            .push("external production runtime validation has not been performed".to_owned());
    }
    if !production_host_evidence_referenced {
        unresolved_blockers
            .push("production-host runtime evidence reference is unavailable".to_owned());
    }
    if request.live_network_used || request.external_execution_performed {
        unresolved_blockers.push(
            "local paper runtime validation must not use live networks or external execution"
                .to_owned(),
        );
    }

    Ok(PaperRuntimeValidationReport {
        validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
        runtime_host_label: request.runtime_host_label.clone(),
        local_replay_validated,
        local_backtest_executed,
        local_runtime_validation_passed,
        production_host_evidence_referenced,
        external_runtime_validation_recorded: request.external_runtime_validation_performed,
        production_ready: false,
        live_network_used: request.live_network_used || request.backtest_report.live_network_used,
        external_execution_performed: request.external_execution_performed
            || request.backtest_report.external_execution_performed,
        unresolved_blockers,
        validated_at_unix_ms: request.validated_at_unix_ms,
    })
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
    /// Venue-specific matching profile validation failed.
    InvalidMatchingProfile { reason: String },
    /// Sandbox/live calibration record validation failed.
    InvalidCalibrationRecord { reason: String },
    /// Paper replay validation failed before a report could be built.
    InvalidReplayRecord { reason: String },
    /// Paper backtest corpus validation failed.
    InvalidBacktestCorpus { reason: String },
    /// Paper runtime validation request failed.
    InvalidRuntimeValidation { reason: String },
    /// Paper audit journal append or replay failed.
    AuditJournalFailed { reason: String },
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
            Self::InvalidMatchingProfile { reason } => {
                write!(formatter, "invalid paper matching profile: {reason}")
            }
            Self::InvalidCalibrationRecord { reason } => {
                write!(formatter, "invalid paper calibration record: {reason}")
            }
            Self::InvalidReplayRecord { reason } => {
                write!(formatter, "invalid paper replay record: {reason}")
            }
            Self::InvalidBacktestCorpus { reason } => {
                write!(formatter, "invalid paper backtest corpus: {reason}")
            }
            Self::InvalidRuntimeValidation { reason } => {
                write!(formatter, "invalid paper runtime validation: {reason}")
            }
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "paper audit journal failed: {reason}")
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

fn validate_audit_timestamp(value: u64) -> Result<(), PaperConnectorError> {
    if value == 0 {
        Err(PaperConnectorError::AuditJournalFailed {
            reason: "paper audit timestamp must be non-zero".to_owned(),
        })
    } else {
        Ok(())
    }
}

fn format_audit_amount(value: f64) -> String {
    format!("{value:.8}")
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

fn apply_matching_profile_to_request(
    request: &PaperFillSimulationRequest,
    profile: &PaperExchangeMatchingProfile,
) -> Result<(PaperFillSimulationRequest, PaperMatchingProfileReport), PaperConnectorError> {
    request.validate()?;
    profile.validate()?;

    let levels = match request.side {
        PaperFillSide::BuyBase => &request.order_book.asks,
        PaperFillSide::SellBase => &request.order_book.bids,
    };
    let adjusted_levels = exchange_adjusted_levels(levels, request.side, profile)?;
    let mut order_book = request.order_book.clone();
    match request.side {
        PaperFillSide::BuyBase => order_book.asks = adjusted_levels,
        PaperFillSide::SellBase => order_book.bids = adjusted_levels,
    }

    let mut config = request.config.clone();
    config.queue_position_bps = config
        .queue_position_bps
        .max(profile.taker_queue_position_bps);
    if !profile.supports_partial_fills {
        config.allow_partial_fills = false;
    }

    let adjusted_request = PaperFillSimulationRequest {
        order_book,
        config,
        ..request.clone()
    };
    let exchange_adjusted_levels = match request.side {
        PaperFillSide::BuyBase => adjusted_request.order_book.asks.len(),
        PaperFillSide::SellBase => adjusted_request.order_book.bids.len(),
    };

    Ok((
        adjusted_request,
        PaperMatchingProfileReport {
            validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
            venue: profile.venue.clone(),
            requested_notional_quote: request.intent.notional_quote,
            min_notional_quote: profile.min_notional_quote,
            max_notional_quote: profile.max_notional_quote,
            original_levels: levels.len(),
            exchange_adjusted_levels,
            partial_fills_supported: profile.supports_partial_fills,
            applied_queue_position_bps: request
                .config
                .queue_position_bps
                .max(profile.taker_queue_position_bps),
            accepted: true,
            reason: "local venue matching profile applied to supplied paper order book".to_owned(),
        },
    ))
}

fn exchange_adjusted_levels(
    levels: &[PriceLevel],
    side: PaperFillSide,
    profile: &PaperExchangeMatchingProfile,
) -> Result<Vec<PriceLevel>, PaperConnectorError> {
    let mut adjusted = Vec::with_capacity(levels.len());
    for level in levels {
        let price_quote = match side {
            PaperFillSide::BuyBase => round_up_to_step(level.price_quote, profile.price_tick_quote),
            PaperFillSide::SellBase => {
                round_down_to_step(level.price_quote, profile.price_tick_quote)
            }
        };
        let quantity_base = round_down_to_step(level.quantity_base, profile.quantity_step_base);
        if quantity_base <= f64::EPSILON {
            continue;
        }
        adjusted.push(
            PriceLevel::new(price_quote, quantity_base).map_err(|source| {
                PaperConnectorError::InvalidMatchingProfile {
                    reason: source.to_string(),
                }
            })?,
        );
    }
    if adjusted.is_empty() {
        return Err(PaperConnectorError::InvalidMatchingProfile {
            reason: "venue tick/step rules removed all usable order-book levels".to_owned(),
        });
    }
    Ok(adjusted)
}

fn round_up_to_step(value: f64, step: f64) -> f64 {
    (value / step).ceil() * step
}

fn round_down_to_step(value: f64, step: f64) -> f64 {
    (value / step).floor() * step
}

fn simulate_adverse_selection(
    config: &PaperAdverseSelectionConfig,
    fill: &PaperFillSimulationReport,
    current_net_profit_quote: f64,
) -> Result<PaperAdverseSelectionReport, PaperConnectorError> {
    config.validate()?;
    if !config.enabled || fill.filled_notional_quote <= f64::EPSILON {
        return Ok(PaperAdverseSelectionReport {
            validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
            enabled: config.enabled,
            latency_ms: fill.latency_ms,
            penalty_bps: 0.0,
            penalty_quote: 0.0,
            net_profit_after_penalty_quote: current_net_profit_quote,
            reason: "adverse-selection penalty disabled or no filled notional".to_owned(),
        });
    }

    let latency_units = fill.latency_ms as f64 / 100.0;
    let raw_penalty_bps = latency_units.mul_add(
        config.latency_penalty_bps_per_100ms,
        config.volatility_penalty_bps + config.order_book_imbalance_penalty_bps,
    );
    let penalty_bps = raw_penalty_bps.min(config.max_penalty_bps);
    let penalty_quote = fill.filled_notional_quote * (penalty_bps / 10_000.0);
    Ok(PaperAdverseSelectionReport {
        validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
        enabled: true,
        latency_ms: fill.latency_ms,
        penalty_bps: normalize_zero(penalty_bps),
        penalty_quote: normalize_zero(penalty_quote),
        net_profit_after_penalty_quote: current_net_profit_quote - penalty_quote,
        reason: "local adverse-selection penalty applied to modeled paper fill".to_owned(),
    })
}

fn apply_calibration_record(
    record: &PaperVenueCalibrationRecord,
    fill: &PaperFillSimulationReport,
    current_net_profit_quote: f64,
) -> Result<PaperCalibrationApplicationReport, PaperConnectorError> {
    record.validate(&record.venue)?;
    let comparison_slippage_bps = if let Some(live_slippage_bps) = record.live_slippage_bps {
        (record.sandbox_slippage_bps + live_slippage_bps) / 2.0
    } else {
        record.sandbox_slippage_bps
    };
    let discrepancy_bps = normalize_zero(
        (comparison_slippage_bps - record.paper_slippage_bps)
            .max(0.0)
            .max(fill.slippage_bps - record.paper_slippage_bps),
    );
    let penalty_quote = fill.filled_notional_quote * (discrepancy_bps / 10_000.0);
    Ok(PaperCalibrationApplicationReport {
        validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
        calibration_id: record.calibration_id.clone(),
        discrepancy_bps,
        penalty_quote: normalize_zero(penalty_quote),
        net_profit_after_penalty_quote: current_net_profit_quote - penalty_quote,
        external_observation_available: record.external_observation_available,
        evidence_reference: record.evidence_reference.clone(),
        reason: "reference-only sandbox/live discrepancy calibration applied locally".to_owned(),
    })
}

fn roi_bps(net_profit_quote: f64, filled_notional_quote: f64) -> f64 {
    if filled_notional_quote > 0.0 {
        (net_profit_quote / filled_notional_quote) * 10_000.0
    } else {
        0.0
    }
}

#[allow(clippy::too_many_lines)]
fn validate_paper_ledger_replay(
    ledger: &PaperBalanceLedger,
    replayed_at_unix_ms: u64,
) -> Result<PaperAuditReplayValidationReport, PaperConnectorError> {
    if replayed_at_unix_ms == 0 {
        return Err(PaperConnectorError::InvalidReplayRecord {
            reason: "replay timestamp must be non-zero".to_owned(),
        });
    }
    let mut replay_balances: Vec<PaperAssetBalance> = Vec::new();
    let mut violations = Vec::new();
    let mut reserve_entries = 0usize;
    let mut settlement_entries = 0usize;

    for (index, entry) in ledger.entries.iter().enumerate() {
        let expected_id = format!("paper-ledger-entry-{}", index.saturating_add(1));
        if entry.id != expected_id {
            violations.push(replay_violation(
                &entry.id,
                "entry-id-sequence",
                "ledger entry id does not match replay order",
            ));
        }
        if entry.recorded_at_unix_ms == 0 {
            violations.push(replay_violation(
                &entry.id,
                "entry-timestamp",
                "ledger entry timestamp is zero",
            ));
        }
        if let Err(error) = replay_entry(entry, &mut replay_balances) {
            violations.push(error);
            continue;
        }
        match entry.kind {
            PaperLedgerEntryKind::InitialBalance => {}
            PaperLedgerEntryKind::ReserveNotional => {
                reserve_entries = reserve_entries.saturating_add(1);
            }
            PaperLedgerEntryKind::SettleFill => {
                settlement_entries = settlement_entries.saturating_add(1);
            }
        }
    }

    let final_balances_match = balances_match(&replay_balances, &ledger.balances, &mut violations);
    let reservations_closed = replay_balances
        .iter()
        .all(|balance| balance.reserved.abs() <= f64::EPSILON);
    if !reservations_closed {
        violations.push(replay_violation(
            "paper-ledger-final",
            "reservation-open",
            "paper ledger replay ended with non-zero reserved balances",
        ));
    }
    let balanced = violations.is_empty() && final_balances_match && reservations_closed;
    let status = if balanced {
        PaperReplayValidationStatus::Passed
    } else {
        PaperReplayValidationStatus::Failed
    };

    Ok(PaperAuditReplayValidationReport {
        validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
        status,
        entries_replayed: ledger.entries.len(),
        reserve_entries,
        settlement_entries,
        final_balances_match,
        reservations_closed,
        balanced,
        replayed_final_balances: replay_balances,
        violations,
        replayed_at_unix_ms,
        direct_audit_journal_integrated: false,
    })
}

fn replay_entry(
    entry: &PaperLedgerEntry,
    replay_balances: &mut Vec<PaperAssetBalance>,
) -> Result<(), PaperReplayViolation> {
    validate_finite("available delta", entry.available_delta)
        .map_err(|_| replay_violation(&entry.id, "available-delta", "invalid available delta"))?;
    validate_finite("reserved delta", entry.reserved_delta)
        .map_err(|_| replay_violation(&entry.id, "reserved-delta", "invalid reserved delta"))?;

    if entry.kind == PaperLedgerEntryKind::InitialBalance {
        if find_replay_balance(replay_balances, &entry.venue, &entry.asset).is_some() {
            return Err(replay_violation(
                &entry.id,
                "duplicate-initial-balance",
                "duplicate initial balance entry",
            ));
        }
        replay_balances.push(PaperAssetBalance {
            venue: entry.venue.clone(),
            asset: entry.asset.clone(),
            available: normalize_zero(entry.resulting_available),
            reserved: normalize_zero(entry.resulting_reserved),
        });
        return Ok(());
    }

    let Some(balance) = find_replay_balance_mut(replay_balances, &entry.venue, &entry.asset) else {
        return Err(replay_violation(
            &entry.id,
            "missing-replay-balance",
            "mutation has no replay balance",
        ));
    };
    let available = normalize_zero(balance.available + entry.available_delta);
    let reserved = normalize_zero(balance.reserved + entry.reserved_delta);
    if available < -f64::EPSILON || reserved < -f64::EPSILON {
        return Err(replay_violation(
            &entry.id,
            "negative-replay-balance",
            "mutation would make a replay balance negative",
        ));
    }
    if (available - entry.resulting_available).abs() > 0.000_000_1
        || (reserved - entry.resulting_reserved).abs() > 0.000_000_1
    {
        return Err(replay_violation(
            &entry.id,
            "resulting-balance-mismatch",
            "entry resulting balance does not match replayed balance",
        ));
    }
    balance.available = available;
    balance.reserved = reserved;
    Ok(())
}

fn balances_match(
    replay_balances: &[PaperAssetBalance],
    ledger_balances: &[PaperAssetBalance],
    violations: &mut Vec<PaperReplayViolation>,
) -> bool {
    if replay_balances.len() != ledger_balances.len() {
        violations.push(replay_violation(
            "paper-ledger-final",
            "balance-count-mismatch",
            "replayed final balance count does not match ledger balance count",
        ));
        return false;
    }
    let mut matched = true;
    for balance in ledger_balances {
        let Some(replay_balance) =
            find_replay_balance(replay_balances, &balance.venue, &balance.asset)
        else {
            violations.push(replay_violation(
                "paper-ledger-final",
                "missing-final-balance",
                "ledger final balance is missing from replay",
            ));
            matched = false;
            continue;
        };
        if (replay_balance.available - balance.available).abs() > 0.000_000_1
            || (replay_balance.reserved - balance.reserved).abs() > 0.000_000_1
        {
            violations.push(replay_violation(
                "paper-ledger-final",
                "final-balance-mismatch",
                "replayed final balance does not match ledger final balance",
            ));
            matched = false;
        }
    }
    matched
}

fn replay_violation(entry_id: &str, code: &str, message: &str) -> PaperReplayViolation {
    PaperReplayViolation {
        entry_id: entry_id.to_owned(),
        code: code.to_owned(),
        message: message.to_owned(),
    }
}

fn find_replay_balance<'a>(
    balances: &'a [PaperAssetBalance],
    venue: &VenueRef,
    asset: &str,
) -> Option<&'a PaperAssetBalance> {
    balances
        .iter()
        .find(|balance| same_venue(&balance.venue, venue) && balance.asset == asset)
}

fn find_replay_balance_mut<'a>(
    balances: &'a mut [PaperAssetBalance],
    venue: &VenueRef,
    asset: &str,
) -> Option<&'a mut PaperAssetBalance> {
    balances
        .iter_mut()
        .find(|balance| same_venue(&balance.venue, venue) && balance.asset == asset)
}

#[allow(clippy::too_many_lines)]
fn run_paper_backtest_corpus(
    adapter: &PaperExecutionAdapter,
    corpus: &PaperBacktestCorpus,
    now_unix_ms: u64,
) -> Result<PaperBacktestRunReport, PaperConnectorError> {
    validate_backtest_corpus(corpus, now_unix_ms)?;
    let mut scenario_reports = Vec::with_capacity(corpus.scenarios.len());
    let mut total_steps = 0usize;
    let mut filled_steps = 0usize;
    let mut partially_filled_steps = 0usize;
    let mut unfilled_steps = 0usize;
    let mut net_profit_quote = 0.0;

    for scenario in &corpus.scenarios {
        let mut ledger = PaperBalanceLedger::new(scenario.initial_balances.clone(), now_unix_ms)?;
        let mut scenario_filled = 0usize;
        let mut scenario_partially_filled = 0usize;
        let mut scenario_unfilled = 0usize;
        let mut scenario_net_profit_quote = 0.0;

        for step in &scenario.steps {
            let report = execute_backtest_step(adapter, step, &mut ledger)?;
            match report.status {
                PaperExecutionStatus::Filled => {
                    filled_steps = filled_steps.saturating_add(1);
                    scenario_filled = scenario_filled.saturating_add(1);
                }
                PaperExecutionStatus::PartiallyFilled => {
                    partially_filled_steps = partially_filled_steps.saturating_add(1);
                    scenario_partially_filled = scenario_partially_filled.saturating_add(1);
                }
                PaperExecutionStatus::Unfilled => {
                    unfilled_steps = unfilled_steps.saturating_add(1);
                    scenario_unfilled = scenario_unfilled.saturating_add(1);
                }
            }
            total_steps = total_steps.saturating_add(1);
            scenario_net_profit_quote += report.net_profit_quote;
            net_profit_quote += report.net_profit_quote;
        }

        let replay_validation = ledger.validate_replay(now_unix_ms.saturating_add(1))?;
        scenario_reports.push(PaperBacktestScenarioReport {
            scenario_id: scenario.scenario_id.clone(),
            steps_executed: scenario.steps.len(),
            filled_steps: scenario_filled,
            partially_filled_steps: scenario_partially_filled,
            unfilled_steps: scenario_unfilled,
            net_profit_quote: scenario_net_profit_quote,
            final_balances: ledger.balances,
            replay_validation,
        });
    }

    let replay_validated = scenario_reports
        .iter()
        .all(|report| report.replay_validation.balanced);

    Ok(PaperBacktestRunReport {
        validation_version: PAPER_REALISM_VALIDATION_VERSION.to_owned(),
        corpus_id: corpus.corpus_id.clone(),
        scenarios_executed: corpus.scenarios.len(),
        total_steps,
        filled_steps,
        partially_filled_steps,
        unfilled_steps,
        net_profit_quote,
        historical_fixture_replay: corpus.historical_fixture_replay,
        local_fixture_only: corpus.local_fixture_only,
        external_data_downloaded: corpus.external_data_downloaded,
        live_network_used: false,
        external_execution_performed: false,
        replay_validated,
        scenario_reports,
        executed_at_unix_ms: now_unix_ms,
    })
}

fn execute_backtest_step(
    adapter: &PaperExecutionAdapter,
    step: &PaperBacktestStep,
    ledger: &mut PaperBalanceLedger,
) -> Result<PaperExecutionReport, PaperConnectorError> {
    validate_backtest_step(step)?;
    if let Some(matching_profile) = &step.matching_profile {
        let request = PaperVenueRealismRequest {
            fill_request: step.fill_request.clone(),
            matching_profile: matching_profile.clone(),
            adverse_selection: step.adverse_selection.clone().unwrap_or_default(),
            calibration: step.calibration.clone(),
        };
        Ok(adapter
            .submit_with_venue_realism_and_ledger(&request, ledger)?
            .execution
            .report)
    } else {
        Ok(adapter
            .submit_with_fill_model_and_ledger(&step.fill_request, ledger)?
            .report)
    }
}

fn validate_backtest_corpus(
    corpus: &PaperBacktestCorpus,
    now_unix_ms: u64,
) -> Result<(), PaperConnectorError> {
    if now_unix_ms == 0 {
        return Err(PaperConnectorError::InvalidBacktestCorpus {
            reason: "backtest execution timestamp must be non-zero".to_owned(),
        });
    }
    if corpus.corpus_id.trim().is_empty() {
        return Err(PaperConnectorError::InvalidBacktestCorpus {
            reason: "backtest corpus id is required".to_owned(),
        });
    }
    if !corpus.local_fixture_only || corpus.external_data_downloaded {
        return Err(PaperConnectorError::InvalidBacktestCorpus {
            reason: "paper backtests must use local fixtures without external downloads".to_owned(),
        });
    }
    if corpus.scenarios.is_empty() {
        return Err(PaperConnectorError::InvalidBacktestCorpus {
            reason: "backtest corpus requires at least one scenario".to_owned(),
        });
    }
    for scenario in &corpus.scenarios {
        if scenario.scenario_id.trim().is_empty() {
            return Err(PaperConnectorError::InvalidBacktestCorpus {
                reason: "backtest scenario id is required".to_owned(),
            });
        }
        if scenario.initial_balances.is_empty() || scenario.steps.is_empty() {
            return Err(PaperConnectorError::InvalidBacktestCorpus {
                reason: "backtest scenarios require balances and steps".to_owned(),
            });
        }
        for step in &scenario.steps {
            validate_backtest_step(step)?;
        }
    }
    Ok(())
}

fn validate_backtest_step(step: &PaperBacktestStep) -> Result<(), PaperConnectorError> {
    if step.step_id.trim().is_empty() {
        return Err(PaperConnectorError::InvalidBacktestCorpus {
            reason: "backtest step id is required".to_owned(),
        });
    }
    step.fill_request.validate()?;
    if let Some(matching_profile) = &step.matching_profile {
        matching_profile.validate()?;
    }
    if let Some(adverse_selection) = &step.adverse_selection {
        adverse_selection.validate()?;
    }
    if let Some(calibration) = &step.calibration {
        calibration.validate(&step.fill_request.intent.venue)?;
    }
    Ok(())
}

fn validate_non_secret_reference(
    label: &str,
    value: Option<&str>,
) -> Result<(), PaperConnectorError> {
    let Some(value) = value else {
        return Ok(());
    };
    let lowered = value.to_ascii_lowercase();
    let forbidden = [
        "api_key=",
        "apikey=",
        "secret=",
        "private_key",
        "seed phrase",
        "mnemonic=",
        "bearer ",
        "authorization:",
        "token=",
    ];
    if forbidden.iter().any(|needle| lowered.contains(needle)) {
        return Err(PaperConnectorError::InvalidRuntimeValidation {
            reason: format!("{label} contains secret-like text"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ledger_execution_adapter_run_paper_fills, persist_paper_balance_ledger_checkpoint,
        persist_paper_execution_report_checkpoint, validate_paper_runtime,
        PaperAdverseSelectionConfig, PaperAssetBalance, PaperBacktestCorpus, PaperBacktestScenario,
        PaperBacktestStep, PaperBalanceLedger, PaperExchangeMatchingProfile, PaperExecutionAdapter,
        PaperExecutionStatus, PaperFeeProvider, PaperFillModelConfig, PaperFillSide,
        PaperFillSimulationRequest, PaperFillSimulationStatus, PaperLedgerEntryKind,
        PaperMarketDataProvider, PaperRuntimeValidationRequest, PaperVenueCalibrationRecord,
        PaperVenueRealismRequest, PAPER_BALANCE_LEDGER_CHECKPOINT_KEY,
        PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY, PAPER_EXECUTION_STATE_SUBSYSTEM,
        PAPER_REALISM_VALIDATION_VERSION,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, DestinationPolicy,
        DeterministicExecutionAdapterBoundary, DeterministicExecutionPlanner, ExecutionAdapter,
        ExecutionAdapterConfig, ExecutionAdapterRequest, ExecutionIntent, ExecutionIntentKind,
        ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerRequest, ExecutionScope,
        FeeAdjustedEdge, FeeEstimate, FeeProvider, FeeSchedule, LiquidityRole, MarketDataProvider,
        MarketDataRequest, MarketPair, OpportunityCandidate, OpportunityLeg, OpportunityLegSide,
        OpportunityRouteKind, OpportunityScore, OrderBookSnapshot, PolicyEngine, PriceLevel,
        SqliteWalStateStore, StateStore, VenueKind, VenueRef,
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

    fn adapter_ledger_candidate() -> OpportunityCandidate {
        let pair = pair();
        let edge = FeeAdjustedEdge::calculate(2.0, 0.20, 50.0).expect("edge should validate");
        OpportunityCandidate {
            id: "opp-adapter-ledger-btc-usdc".to_owned(),
            route_kind: OpportunityRouteKind::CexCex,
            pair: pair.clone(),
            legs: vec![
                opportunity_leg(
                    "quote-ledger-a",
                    pair.clone(),
                    OpportunityLegSide::Buy,
                    25.0,
                ),
                opportunity_leg("quote-ledger-b", pair, OpportunityLegSide::Sell, 25.0),
            ],
            edge,
            score: OpportunityScore {
                roi_bps: edge.roi_bps,
                freshness_penalty_bps: 0.0,
                risk_penalty_bps: 0.0,
                score_bps: edge.roi_bps,
            },
            liquidity_model: None,
            transfer_risk: None,
            discovered_at_unix_ms: 1_700_000_000_900,
            source_quote_ids: vec!["quote-ledger-a".to_owned(), "quote-ledger-b".to_owned()],
            warnings: Vec::new(),
        }
    }

    fn opportunity_leg(
        source_quote_id: &str,
        pair: MarketPair,
        side: OpportunityLegSide,
        notional_quote: f64,
    ) -> OpportunityLeg {
        OpportunityLeg {
            venue: venue(),
            pair: pair.clone(),
            side,
            price_quote: 100.0,
            quantity_base: notional_quote / 100.0,
            notional_quote,
            fee_estimate: FeeEstimate {
                venue: venue(),
                pair: Some(pair),
                notional_quote,
                liquidity_role: LiquidityRole::Taker,
                fee_bps: 10.0,
                venue_fee_quote: 0.10,
                network_fee_quote: 0.0,
                total_fee_quote: 0.10,
                externally_verified: true,
            },
            source_quote_id: source_quote_id.to_owned(),
            market_data_age_ms: 100,
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

    fn matching_profile() -> PaperExchangeMatchingProfile {
        PaperExchangeMatchingProfile {
            venue: venue(),
            price_tick_quote: 0.5,
            quantity_step_base: 0.05,
            min_notional_quote: 5.0,
            max_notional_quote: Some(50.0),
            supports_market_orders: true,
            supports_limit_orders: true,
            supports_post_only: false,
            supports_partial_fills: true,
            maker_queue_position_bps: 0,
            taker_queue_position_bps: 500,
            sandbox_live_calibration_required: true,
        }
    }

    fn adverse_selection() -> PaperAdverseSelectionConfig {
        PaperAdverseSelectionConfig {
            enabled: true,
            latency_penalty_bps_per_100ms: 1.0,
            volatility_penalty_bps: 2.0,
            order_book_imbalance_penalty_bps: 1.5,
            max_penalty_bps: 8.0,
        }
    }

    fn calibration_record() -> PaperVenueCalibrationRecord {
        PaperVenueCalibrationRecord {
            calibration_id: "calibration-paper-coinbase-1".to_owned(),
            venue: venue(),
            sample_count: 32,
            paper_slippage_bps: 3.0,
            sandbox_slippage_bps: 6.0,
            live_slippage_bps: None,
            evidence_reference: Some("github-actions-artifact:paper-calibration-local".to_owned()),
            external_observation_available: false,
            secret_material_recorded: false,
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
    fn paper_ledgered_execution_appends_report_and_mutations_to_audit_journal() {
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
        let path = unique_audit_path("paper-ledgered-execution");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("audit journal opens");

        let execution = adapter
            .submit_with_fill_model_ledger_and_audit(
                &fill_request(20.3, 120, true),
                &mut ledger,
                &mut journal,
            )
            .expect("audited ledgered fill succeeds");

        assert_eq!(execution.audit_report.records_appended, 4);
        assert!(execution.audit_report.journal_replay_verified);
        assert_eq!(execution.audit_report.intent_record_sequence, Some(1));
        assert_eq!(execution.audit_report.reserve_record_sequence, 2);
        assert_eq!(execution.audit_report.settlement_record_sequence, 3);
        assert_eq!(execution.audit_report.report_record_sequence, 4);
        assert!(!execution.audit_report.live_network_used);
        assert!(!execution.audit_report.external_execution_performed);

        let reopened = AppendOnlyAuditJournal::open(&path).expect("audit journal replays");
        assert_eq!(reopened.next_sequence(), 5);
        assert_eq!(journal.previous_hash(), reopened.previous_hash());
        let lines = fs::read_to_string(&path).expect("journal should be readable");
        assert!(lines.contains("\"kind\":\"intent-lifecycle\""));
        assert!(lines.contains("\"kind\":\"reconciliation\""));
        assert!(lines.contains("\"kind\":\"execution-result\""));
        assert!(lines.contains("paper_audit_integration_version"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn venue_realism_ledgered_execution_appends_to_audit_journal() {
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
        let path = unique_audit_path("paper-venue-realism");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("audit journal opens");
        let request = PaperVenueRealismRequest {
            fill_request: fill_request(20.3, 120, true),
            matching_profile: matching_profile(),
            adverse_selection: adverse_selection(),
            calibration: Some(calibration_record()),
        };

        let execution = adapter
            .submit_with_venue_realism_ledger_and_audit(&request, &mut ledger, &mut journal)
            .expect("audited venue-realistic execution succeeds");

        assert_eq!(execution.audit_report.records_appended, 4);
        assert!(execution.audit_report.journal_replay_verified);
        assert_eq!(execution.audit_report.intent_record_sequence, Some(1));
        assert_eq!(
            execution.audit_report.execution_report_id,
            execution.ledgered_execution.execution.report.id
        );
        assert_eq!(journal.next_sequence(), 5);

        let _ = fs::remove_file(path);
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
    fn venue_realism_applies_matching_adverse_selection_and_calibration() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let request = PaperVenueRealismRequest {
            fill_request: fill_request(20.3, 120, true),
            matching_profile: matching_profile(),
            adverse_selection: adverse_selection(),
            calibration: Some(calibration_record()),
        };

        let execution = adapter
            .submit_with_venue_realism(&request)
            .expect("venue realism should produce a report");

        assert_eq!(
            execution.report.fill_model_version,
            PAPER_REALISM_VALIDATION_VERSION
        );
        assert!(execution.matching.accepted);
        assert_eq!(execution.matching.exchange_adjusted_levels, 3);
        assert_eq!(execution.matching.applied_queue_position_bps, 500);
        assert!(execution.adverse_selection.penalty_quote > 0.0);
        assert!(
            execution
                .calibration
                .as_ref()
                .expect("calibration applied")
                .penalty_quote
                > 0.0
        );
        assert!(
            execution.report.net_profit_quote
                < execution.report.gross_profit_quote - execution.report.total_fees_quote
        );
        assert!(execution.report.adverse_selection_quote > 0.0);
        assert!(execution.report.calibration_adjustment_quote > 0.0);
    }

    #[test]
    fn venue_realism_ledger_settles_adjusted_profit_and_replays() {
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
        let request = PaperVenueRealismRequest {
            fill_request: fill_request(20.3, 120, true),
            matching_profile: matching_profile(),
            adverse_selection: adverse_selection(),
            calibration: Some(calibration_record()),
        };

        let execution = adapter
            .submit_with_venue_realism_and_ledger(&request, &mut ledger)
            .expect("venue realism ledger should settle");

        assert_eq!(
            execution.reserve_entry.kind,
            PaperLedgerEntryKind::ReserveNotional
        );
        assert_eq!(
            execution.settlement_entry.kind,
            PaperLedgerEntryKind::SettleFill
        );
        assert!(ledger.reserved_balance(&venue(), "USDC").abs() < f64::EPSILON);
        let replay = ledger
            .validate_replay(1_700_000_000_900)
            .expect("replay should build");
        assert!(replay.balanced);
        assert_eq!(replay.status, super::PaperReplayValidationStatus::Passed);
        assert_eq!(replay.reserve_entries, 1);
        assert_eq!(replay.settlement_entries, 1);
    }

    #[test]
    fn paper_replay_detects_tampered_resulting_balance() {
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_000_000,
            )
            .expect("ledger validates");
        ledger.entries[0].resulting_available = 999.0;

        let replay = ledger
            .validate_replay(1_700_000_000_900)
            .expect("replay report should build for tampered ledger");

        assert!(!replay.balanced);
        assert_eq!(replay.status, super::PaperReplayValidationStatus::Failed);
        assert!(replay
            .violations
            .iter()
            .any(|violation| violation.code == "final-balance-mismatch"));
    }

    #[test]
    fn paper_backtest_corpus_executes_local_historical_fixtures() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let corpus = PaperBacktestCorpus {
            corpus_id: "paper-history-corpus-1".to_owned(),
            historical_fixture_replay: true,
            local_fixture_only: true,
            external_data_downloaded: false,
            scenarios: vec![PaperBacktestScenario {
                scenario_id: "scenario-1".to_owned(),
                initial_balances: vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                steps: vec![
                    PaperBacktestStep {
                        step_id: "step-1".to_owned(),
                        fill_request: fill_request(20.3, 120, true),
                        matching_profile: Some(matching_profile()),
                        adverse_selection: Some(adverse_selection()),
                        calibration: Some(calibration_record()),
                    },
                    PaperBacktestStep {
                        step_id: "step-2".to_owned(),
                        fill_request: fill_request(40.0, 80, true),
                        matching_profile: None,
                        adverse_selection: None,
                        calibration: None,
                    },
                ],
            }],
        };

        let report = adapter
            .run_backtest_corpus(&corpus, 1_700_000_001_000)
            .expect("local backtest corpus should execute");

        assert_eq!(report.validation_version, PAPER_REALISM_VALIDATION_VERSION);
        assert_eq!(report.scenarios_executed, 1);
        assert_eq!(report.total_steps, 2);
        assert!(report.filled_steps >= 1);
        assert!(report.partially_filled_steps >= 1);
        assert!(report.replay_validated);
        assert!(!report.live_network_used);
        assert!(!report.external_execution_performed);
        assert!(report.local_fixture_only);
        assert!(report.historical_fixture_replay);
    }

    #[test]
    fn paper_backtest_corpus_tracks_unfilled_failed_steps_locally() {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        let adapter = PaperExecutionAdapter::new("paper-exec", PolicyEngine::from_config(config))
            .expect("adapter should validate");
        let corpus = PaperBacktestCorpus {
            corpus_id: "paper-history-corpus-unfilled".to_owned(),
            historical_fixture_replay: true,
            local_fixture_only: true,
            external_data_downloaded: false,
            scenarios: vec![PaperBacktestScenario {
                scenario_id: "scenario-unfilled".to_owned(),
                initial_balances: vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                steps: vec![PaperBacktestStep {
                    step_id: "step-unfilled".to_owned(),
                    fill_request: fill_request(40.0, 80, false),
                    matching_profile: None,
                    adverse_selection: None,
                    calibration: None,
                }],
            }],
        };

        let report = adapter
            .run_backtest_corpus(&corpus, 1_700_000_001_500)
            .expect("local backtest corpus should execute");

        assert_eq!(report.scenarios_executed, 1);
        assert_eq!(report.total_steps, 1);
        assert_eq!(report.filled_steps, 0);
        assert_eq!(report.partially_filled_steps, 0);
        assert_eq!(report.unfilled_steps, 1);
        assert!(report.net_profit_quote.abs() < f64::EPSILON);
        assert!(report.replay_validated);
        assert!(report.local_fixture_only);
        assert!(report.historical_fixture_replay);
        assert!(!report.external_data_downloaded);
        assert!(!report.live_network_used);
        assert!(!report.external_execution_performed);

        let scenario = &report.scenario_reports[0];
        assert_eq!(scenario.steps_executed, 1);
        assert_eq!(scenario.filled_steps, 0);
        assert_eq!(scenario.partially_filled_steps, 0);
        assert_eq!(scenario.unfilled_steps, 1);
        assert!(scenario.net_profit_quote.abs() < f64::EPSILON);
        assert!(scenario.replay_validation.final_balances_match);
        assert!(scenario.replay_validation.reservations_closed);
        assert!(scenario.replay_validation.balanced);
        assert_eq!(scenario.final_balances.len(), 1);
        assert!((scenario.final_balances[0].available - 1_000.0).abs() < f64::EPSILON);
        assert!(scenario.final_balances[0].reserved.abs() < f64::EPSILON);
    }

    #[test]
    fn paper_runtime_validation_preserves_production_blockers() {
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
            .submit_with_fill_model_and_ledger(&fill_request(20.3, 120, true), &mut ledger)
            .expect("ledgered fill succeeds");
        let replay = ledger
            .validate_replay(1_700_000_000_900)
            .expect("replay succeeds");
        let corpus = PaperBacktestCorpus {
            corpus_id: "paper-history-corpus-2".to_owned(),
            historical_fixture_replay: true,
            local_fixture_only: true,
            external_data_downloaded: false,
            scenarios: vec![PaperBacktestScenario {
                scenario_id: "scenario-runtime".to_owned(),
                initial_balances: vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                steps: vec![PaperBacktestStep {
                    step_id: "step-runtime".to_owned(),
                    fill_request: fill_request(20.3, 120, true),
                    matching_profile: None,
                    adverse_selection: None,
                    calibration: None,
                }],
            }],
        };
        let backtest = adapter
            .run_backtest_corpus(&corpus, 1_700_000_001_000)
            .expect("backtest succeeds");
        let runtime_report = validate_paper_runtime(&PaperRuntimeValidationRequest {
            replay_validation: replay,
            backtest_report: backtest,
            runtime_host_label: "local-cargo-test".to_owned(),
            production_host_evidence_reference: None,
            external_runtime_validation_performed: false,
            live_network_used: false,
            external_execution_performed: false,
            validated_at_unix_ms: 1_700_000_001_500,
        })
        .expect("runtime validation report should build");

        assert!(runtime_report.local_runtime_validation_passed);
        assert!(!runtime_report.production_ready);
        assert!(!runtime_report.production_host_evidence_referenced);
        assert!(runtime_report
            .unresolved_blockers
            .iter()
            .any(|blocker| blocker.contains("external production runtime validation")));
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

    #[test]
    fn adapter_run_paper_fills_settle_ledger_audit_and_state_locally() {
        let audit_path = unique_audit_path("adapter-run-paper-ledger");
        let state_path = unique_state_path("adapter-run-paper-ledger");
        cleanup_state_files(&state_path);
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
        );
        let plan = DeterministicExecutionPlanner::new()
            .plan(
                &ExecutionPlannerRequest {
                    id: "adapter-ledger-planner-request".to_owned(),
                    strategy_id: "strategy-paper".to_owned(),
                    candidate: adapter_ledger_candidate(),
                    config: ExecutionPlannerConfig::default(),
                    default_chain: None,
                    now_unix_ms: 1_700_000_001_000,
                },
                &policy,
            )
            .expect("planner should create paper draft");
        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(
                &ExecutionAdapterRequest {
                    id: "adapter-ledger-request".to_owned(),
                    plan: plan.clone(),
                    config: ExecutionAdapterConfig::default(),
                    now_unix_ms: 1_700_000_001_100,
                },
                &policy,
            )
            .expect("adapter should model paper fills");
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_001_200,
            )
            .expect("ledger validates");

        let report = ledger_execution_adapter_run_paper_fills(
            &mut journal,
            &mut store,
            &mut ledger,
            &plan,
            &run,
            1_700_000_001_300,
        )
        .expect("adapter fills settle into paper ledger");

        assert_eq!(report.modeled_fills_settled, 2);
        assert_eq!(report.reconciliations_replayed, 2);
        assert_eq!(report.ledger_entries_written, 4);
        assert_eq!(report.audit_records_appended, 8);
        assert!(report.ledger_checkpoint_persisted);
        assert_eq!(
            report.ledger_checkpoint_key,
            PAPER_BALANCE_LEDGER_CHECKPOINT_KEY
        );
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        assert_eq!(journal.next_sequence(), 9);
        assert!(ledger.reserved_balance(&venue(), "USDC").abs() < f64::EPSILON);
        let replay = ledger
            .validate_replay(1_700_000_001_400)
            .expect("ledger replay validates");
        assert_eq!(replay.reserve_entries, 2);
        assert_eq!(replay.settlement_entries, 2);
        assert!(replay.balanced);
        drop(store);
        drop(journal);

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 9);
        let lines = fs::read_to_string(&audit_path).expect("journal should be readable");
        assert!(lines.contains("\"kind\":\"intent-lifecycle\""));
        let reopened_state = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let checkpoint = reopened_state
            .get_checkpoint(PAPER_BALANCE_LEDGER_CHECKPOINT_KEY)
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        let restored: PaperBalanceLedger =
            serde_json::from_str(&checkpoint.value).expect("ledger checkpoint decodes");
        assert_eq!(restored, ledger);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn adapter_run_paper_fill_replay_rejects_duplicate_settlement_after_reopen() {
        let audit_path = unique_audit_path("adapter-run-paper-ledger-idempotency");
        let state_path = unique_state_path("adapter-run-paper-ledger-idempotency");
        cleanup_state_files(&state_path);
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
        );
        let plan = DeterministicExecutionPlanner::new()
            .plan(
                &ExecutionPlannerRequest {
                    id: "adapter-ledger-idempotency-planner-request".to_owned(),
                    strategy_id: "strategy-paper".to_owned(),
                    candidate: adapter_ledger_candidate(),
                    config: ExecutionPlannerConfig::default(),
                    default_chain: None,
                    now_unix_ms: 1_700_000_002_000,
                },
                &policy,
            )
            .expect("planner should create paper draft");
        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(
                &ExecutionAdapterRequest {
                    id: "adapter-ledger-idempotency-request".to_owned(),
                    plan: plan.clone(),
                    config: ExecutionAdapterConfig::default(),
                    now_unix_ms: 1_700_000_002_100,
                },
                &policy,
            )
            .expect("adapter should model paper fills");
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_002_200,
            )
            .expect("ledger validates");

        {
            let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
            let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
            ledger_execution_adapter_run_paper_fills(
                &mut journal,
                &mut store,
                &mut ledger,
                &plan,
                &run,
                1_700_000_002_300,
            )
            .expect("first adapter fill settlement succeeds");
        }

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let checkpoint = store
            .get_checkpoint(PAPER_BALANCE_LEDGER_CHECKPOINT_KEY)
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        let mut restored: PaperBalanceLedger =
            serde_json::from_str(&checkpoint.value).expect("ledger checkpoint decodes");
        let original_entries = restored.entries.len();
        let original_next_sequence = journal.next_sequence();

        let error = ledger_execution_adapter_run_paper_fills(
            &mut journal,
            &mut store,
            &mut restored,
            &plan,
            &run,
            1_700_000_002_400,
        )
        .expect_err("duplicate restart replay settlement must fail closed");

        assert!(error
            .to_string()
            .contains("refuses duplicate modeled fill report"));
        assert_eq!(restored.entries.len(), original_entries);
        assert_eq!(journal.next_sequence(), original_next_sequence);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn adapter_run_paper_fill_reconciliation_mismatch_fails_before_mutation() {
        let audit_path = unique_audit_path("adapter-run-paper-ledger-reconciliation");
        let state_path = unique_state_path("adapter-run-paper-ledger-reconciliation");
        cleanup_state_files(&state_path);
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
        );
        let plan = DeterministicExecutionPlanner::new()
            .plan(
                &ExecutionPlannerRequest {
                    id: "adapter-ledger-reconciliation-planner-request".to_owned(),
                    strategy_id: "strategy-paper".to_owned(),
                    candidate: adapter_ledger_candidate(),
                    config: ExecutionPlannerConfig::default(),
                    default_chain: None,
                    now_unix_ms: 1_700_000_003_000,
                },
                &policy,
            )
            .expect("planner should create paper draft");
        let mut run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(
                &ExecutionAdapterRequest {
                    id: "adapter-ledger-reconciliation-request".to_owned(),
                    plan: plan.clone(),
                    config: ExecutionAdapterConfig::default(),
                    now_unix_ms: 1_700_000_003_100,
                },
                &policy,
            )
            .expect("adapter should model paper fills");
        run.reconciliations[0].observed_notional_quote += 1.0;
        run.reconciliations[0].difference_quote = 1.0;
        let mut ledger =
            PaperBalanceLedger::new(
                vec![PaperAssetBalance::available(venue(), "USDC", 1_000.0)
                    .expect("balance validates")],
                1_700_000_003_200,
            )
            .expect("ledger validates");
        let initial_entries = ledger.entries.len();

        let error = ledger_execution_adapter_run_paper_fills(
            &mut journal,
            &mut store,
            &mut ledger,
            &plan,
            &run,
            1_700_000_003_300,
        )
        .expect_err("reconciliation mismatch must fail before settlement");

        assert!(error
            .to_string()
            .contains("does not match reconciled notional"));
        assert_eq!(ledger.entries.len(), initial_entries);
        assert_eq!(journal.next_sequence(), 1);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
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

    fn unique_audit_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-paper-audit-{label}-{}-{nanos}.jsonl",
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
