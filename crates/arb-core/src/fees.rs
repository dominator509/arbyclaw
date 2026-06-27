#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, MarketPair,
    StateCheckpoint, StateStore, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable fee model version for audit and replay surfaces.
pub const FEE_MODEL_VERSION: &str = "phase-5-fees-v1";

/// State-store subsystem name for local fee verification checkpoints.
pub const FEE_STATE_SUBSYSTEM: &str = "fees";

/// Checkpoint key for the latest local fee schedule verification report.
pub const FEE_LAST_VERIFICATION_CHECKPOINT_KEY: &str = "fees.last_verification";

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

/// Local fee verification input for one normalized fee schedule.
///
/// This records reference-only operator/provider review metadata. It does not
/// call exchange APIs, query RPC endpoints, load credentials, mutate accounts,
/// sign, broadcast, withdraw, bridge, or claim production readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeScheduleVerificationInput {
    /// Schedule being reviewed.
    pub schedule: FeeSchedule,
    /// Stable non-secret review id.
    pub review_id: String,
    /// Non-secret provider or venue reference.
    pub source_reference: String,
    /// Account tier or fee tier label used for the review.
    pub account_tier: String,
    /// Whether maker/taker tier was verified from a reference.
    pub maker_taker_tier_verified: bool,
    /// Whether network/gas estimate was verified from a reference.
    pub network_fee_verified: bool,
    /// Whether withdrawal fee review was required for this schedule.
    pub withdrawal_fee_review_required: bool,
    /// Whether withdrawal fee review was completed when required.
    pub withdrawal_fee_reviewed: bool,
    /// Review timestamp in Unix milliseconds.
    pub reviewed_at_unix_ms: u64,
    /// Current local timestamp in Unix milliseconds.
    pub now_unix_ms: u64,
    /// Maximum accepted age for a fee verification review.
    pub max_review_age_ms: u64,
    /// Whether any live provider/API call was performed. Must remain false here.
    pub live_provider_call_performed: bool,
    /// Whether any credential was loaded. Must remain false here.
    pub credential_loaded: bool,
}

/// Local fee verification status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeeScheduleVerificationStatus {
    /// Schedule verification is current enough for local review.
    ReadyForLocalReview,
    /// Schedule verification is missing or stale and must fail closed.
    Blocked,
}

/// Non-secret local fee verification report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeScheduleVerificationReport {
    /// Stable non-secret review id.
    pub review_id: String,
    /// Venue name for the reviewed schedule.
    pub venue_name: String,
    /// Optional pair symbol for the reviewed schedule.
    pub pair_symbol: Option<String>,
    /// Non-secret provider or venue reference.
    pub source_reference: String,
    /// Account tier or fee tier label used for the review.
    pub account_tier: String,
    /// Verification status.
    pub status: FeeScheduleVerificationStatus,
    /// Whether source schedule itself is externally verified.
    pub schedule_externally_verified: bool,
    /// Whether maker/taker tier was verified from a reference.
    pub maker_taker_tier_verified: bool,
    /// Whether network/gas estimate was verified from a reference.
    pub network_fee_verified: bool,
    /// Whether withdrawal fee review was required for this schedule.
    pub withdrawal_fee_review_required: bool,
    /// Whether withdrawal fee review was completed when required.
    pub withdrawal_fee_reviewed: bool,
    /// Review age in milliseconds.
    pub review_age_ms: u64,
    /// Maximum accepted age for a fee verification review.
    pub max_review_age_ms: u64,
    /// Whether review age exceeded the configured maximum.
    pub stale_review_blocked: bool,
    /// Whether any live provider/API call was performed. Always false here.
    pub live_provider_call_performed: bool,
    /// Whether any credential was loaded. Always false here.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

impl FeeScheduleVerificationInput {
    /// Validate local fee verification input shape.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        self.schedule.validate()?;
        let mut violations = Vec::new();
        validate_text("fee verification review", &self.review_id, &mut violations);
        validate_text(
            "fee verification source reference",
            &self.source_reference,
            &mut violations,
        );
        validate_text(
            "fee verification account tier",
            &self.account_tier,
            &mut violations,
        );
        if self.reviewed_at_unix_ms == 0 || self.now_unix_ms == 0 {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_TIMESTAMP_ZERO",
                "fee verification timestamps must be non-zero",
            ));
        }
        if self.reviewed_at_unix_ms > self.now_unix_ms {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_REVIEW_IN_FUTURE",
                "fee verification reviewed_at_unix_ms cannot be in the future",
            ));
        }
        if self.max_review_age_ms == 0 {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_MAX_AGE_ZERO",
                "fee verification max_review_age_ms must be positive",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }
}

impl FeeScheduleVerificationReport {
    /// Validate local fee verification report invariants.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        let mut violations = Vec::new();
        validate_text("fee verification report", &self.review_id, &mut violations);
        validate_text("fee verification venue", &self.venue_name, &mut violations);
        validate_text(
            "fee verification source reference",
            &self.source_reference,
            &mut violations,
        );
        validate_text(
            "fee verification account tier",
            &self.account_tier,
            &mut violations,
        );
        if self.max_review_age_ms == 0
            || self.review_age_ms > self.max_review_age_ms && !self.stale_review_blocked
        {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_AGE_INCOHERENT",
                "fee verification age must be bounded or marked stale",
            ));
        }
        let should_block = !self.schedule_externally_verified
            || !self.maker_taker_tier_verified
            || !self.network_fee_verified
            || (self.withdrawal_fee_review_required && !self.withdrawal_fee_reviewed)
            || self.stale_review_blocked
            || self.live_provider_call_performed
            || self.credential_loaded;
        if should_block && self.status != FeeScheduleVerificationStatus::Blocked {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_STATUS_SHOULD_BLOCK",
                "incomplete fee verification must produce blocked status",
            ));
        }
        if !should_block && self.status != FeeScheduleVerificationStatus::ReadyForLocalReview {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_STATUS_SHOULD_BE_READY",
                "complete local fee verification must be ready for local review",
            ));
        }
        if self.production_ready {
            violations.push(FeeModelViolation::new(
                "FEE_VERIFICATION_PRODUCTION_READY_FORBIDDEN",
                "local fee verification must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }
}

/// Validate local fee schedule verification metadata without external calls.
pub fn validate_fee_schedule_verification(
    input: FeeScheduleVerificationInput,
) -> Result<FeeScheduleVerificationReport, FeeModelError> {
    input.validate()?;

    let review_age_ms = input.now_unix_ms - input.reviewed_at_unix_ms;
    let stale_review_blocked = review_age_ms > input.max_review_age_ms;
    let withdrawal_blocked = input.withdrawal_fee_review_required && !input.withdrawal_fee_reviewed;
    let blocked = !input.schedule.externally_verified
        || !input.maker_taker_tier_verified
        || !input.network_fee_verified
        || withdrawal_blocked
        || stale_review_blocked
        || input.live_provider_call_performed
        || input.credential_loaded;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !input.schedule.externally_verified,
        "FEE_VERIFICATION_SCHEDULE_UNVERIFIED",
    );
    push_if(
        &mut violation_codes,
        !input.maker_taker_tier_verified,
        "FEE_VERIFICATION_MAKER_TAKER_UNVERIFIED",
    );
    push_if(
        &mut violation_codes,
        !input.network_fee_verified,
        "FEE_VERIFICATION_NETWORK_FEE_UNVERIFIED",
    );
    push_if(
        &mut violation_codes,
        withdrawal_blocked,
        "FEE_VERIFICATION_WITHDRAWAL_FEE_UNREVIEWED",
    );
    push_if(
        &mut violation_codes,
        stale_review_blocked,
        "FEE_VERIFICATION_REVIEW_STALE",
    );
    push_if(
        &mut violation_codes,
        input.live_provider_call_performed,
        "FEE_VERIFICATION_LIVE_PROVIDER_CALL",
    );
    push_if(
        &mut violation_codes,
        input.credential_loaded,
        "FEE_VERIFICATION_CREDENTIAL_LOADED",
    );

    let report = FeeScheduleVerificationReport {
        review_id: input.review_id,
        venue_name: input.schedule.venue.name,
        pair_symbol: input.schedule.pair.map(|pair| pair.symbol()),
        source_reference: input.source_reference,
        account_tier: input.account_tier,
        status: if blocked {
            FeeScheduleVerificationStatus::Blocked
        } else {
            FeeScheduleVerificationStatus::ReadyForLocalReview
        },
        schedule_externally_verified: input.schedule.externally_verified,
        maker_taker_tier_verified: input.maker_taker_tier_verified,
        network_fee_verified: input.network_fee_verified,
        withdrawal_fee_review_required: input.withdrawal_fee_review_required,
        withdrawal_fee_reviewed: input.withdrawal_fee_reviewed,
        review_age_ms,
        max_review_age_ms: input.max_review_age_ms,
        stale_review_blocked,
        live_provider_call_performed: input.live_provider_call_performed,
        credential_loaded: input.credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Persist the latest local fee verification report through state.
pub fn persist_fee_schedule_verification_checkpoint(
    store: &mut impl StateStore,
    report: &FeeScheduleVerificationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, FeeModelError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: FEE_LAST_VERIFICATION_CHECKPOINT_KEY.to_owned(),
        subsystem: FEE_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| FeeModelError::BoundaryFailed {
            reason: format!("failed to serialize fee verification checkpoint: {error}"),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(|error| FeeModelError::BoundaryFailed {
            reason: format!("failed to persist fee verification checkpoint: {error}"),
        })?;
    Ok(checkpoint)
}

/// Append one local fee verification report to the audit journal.
pub fn append_fee_schedule_verification_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &FeeScheduleVerificationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, FeeModelError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("fee-schedule-verification-{}", report.review_id),
        AuditEventKind::RuntimeLifecycle,
        FEE_STATE_SUBSYSTEM,
        "fee-schedule-verification",
        "Fee schedule verification recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "fee_model_version",
            AuditValue::Text(FEE_MODEL_VERSION.to_owned()),
        )
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata("venue_name", AuditValue::Text(report.venue_name.clone()))
        .with_metadata(
            "source_reference",
            AuditValue::Text(report.source_reference.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "schedule_externally_verified",
            AuditValue::Bool(report.schedule_externally_verified),
        )
        .with_metadata(
            "maker_taker_tier_verified",
            AuditValue::Bool(report.maker_taker_tier_verified),
        )
        .with_metadata(
            "network_fee_verified",
            AuditValue::Bool(report.network_fee_verified),
        )
        .with_metadata(
            "withdrawal_fee_review_required",
            AuditValue::Bool(report.withdrawal_fee_review_required),
        )
        .with_metadata(
            "withdrawal_fee_reviewed",
            AuditValue::Bool(report.withdrawal_fee_reviewed),
        )
        .with_metadata("review_age_ms", AuditValue::Unsigned(report.review_age_ms))
        .with_metadata(
            "stale_review_blocked",
            AuditValue::Bool(report.stale_review_blocked),
        )
        .with_metadata(
            "live_provider_call_performed",
            AuditValue::Bool(report.live_provider_call_performed),
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
        .map_err(|error| FeeModelError::BoundaryFailed {
            reason: format!("failed to append fee verification audit record: {error}"),
        })
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
    /// Local audit or state boundary failed.
    BoundaryFailed { reason: String },
}

impl FeeModelError {
    /// Return violations, if this is a validation error.
    #[must_use]
    pub fn violations(&self) -> &[FeeModelViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::ScheduleUnavailable { .. } | Self::BoundaryFailed { .. } => &[],
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
            Self::BoundaryFailed { reason } => write!(formatter, "fee boundary failed: {reason}"),
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

fn validate_text(label: &'static str, value: &str, violations: &mut Vec<FeeModelViolation>) {
    if value.trim().is_empty() {
        violations.push(FeeModelViolation::new_owned(
            "FEE_TEXT_EMPTY",
            format!("{label} must be non-empty"),
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

fn is_non_negative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::{
        append_fee_schedule_verification_audit, persist_fee_schedule_verification_checkpoint,
        validate_fee_schedule_verification, FeeAdjustedEdge, FeeSchedule,
        FeeScheduleVerificationInput, FeeScheduleVerificationReport, FeeScheduleVerificationStatus,
        LiquidityRole, FEE_LAST_VERIFICATION_CHECKPOINT_KEY, FEE_STATE_SUBSYSTEM,
    };
    use crate::{
        AppendOnlyAuditJournal, InMemoryStateStore, MarketPair, StateStore, VenueKind, VenueRef,
    };

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

    #[test]
    fn fee_schedule_verification_accepts_current_reference_only_review() {
        let report = validate_fee_schedule_verification(FeeScheduleVerificationInput {
            schedule: verified_schedule(),
            review_id: "fee-review-1".to_owned(),
            source_reference: "operator-fee-review-2026-06".to_owned(),
            account_tier: "paper-tier".to_owned(),
            maker_taker_tier_verified: true,
            network_fee_verified: true,
            withdrawal_fee_review_required: false,
            withdrawal_fee_reviewed: false,
            reviewed_at_unix_ms: 10_000,
            now_unix_ms: 10_500,
            max_review_age_ms: 1_000,
            live_provider_call_performed: false,
            credential_loaded: false,
        })
        .expect("current local fee review should validate");

        assert_eq!(
            report.status,
            FeeScheduleVerificationStatus::ReadyForLocalReview
        );
        assert!(report.schedule_externally_verified);
        assert!(report.maker_taker_tier_verified);
        assert!(report.network_fee_verified);
        assert_eq!(report.review_age_ms, 500);
        assert!(report.violation_codes.is_empty());
        assert!(!report.live_provider_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn fee_schedule_verification_blocks_stale_or_incomplete_reviews() {
        let mut schedule = verified_schedule();
        schedule.externally_verified = false;
        let report = validate_fee_schedule_verification(FeeScheduleVerificationInput {
            schedule,
            review_id: "fee-review-stale".to_owned(),
            source_reference: "operator-fee-review-old".to_owned(),
            account_tier: "paper-tier".to_owned(),
            maker_taker_tier_verified: false,
            network_fee_verified: false,
            withdrawal_fee_review_required: true,
            withdrawal_fee_reviewed: false,
            reviewed_at_unix_ms: 10_000,
            now_unix_ms: 12_500,
            max_review_age_ms: 1_000,
            live_provider_call_performed: false,
            credential_loaded: false,
        })
        .expect("incomplete local fee review should produce blocked report");

        assert_eq!(report.status, FeeScheduleVerificationStatus::Blocked);
        assert!(report.stale_review_blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_VERIFICATION_SCHEDULE_UNVERIFIED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_VERIFICATION_WITHDRAWAL_FEE_UNREVIEWED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_VERIFICATION_REVIEW_STALE"));
        assert!(!report.live_provider_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn fee_schedule_verification_blocks_live_provider_or_credential_use() {
        let report = validate_fee_schedule_verification(FeeScheduleVerificationInput {
            schedule: verified_schedule(),
            review_id: "fee-review-side-effect".to_owned(),
            source_reference: "operator-fee-review-side-effect".to_owned(),
            account_tier: "paper-tier".to_owned(),
            maker_taker_tier_verified: true,
            network_fee_verified: true,
            withdrawal_fee_review_required: false,
            withdrawal_fee_reviewed: false,
            reviewed_at_unix_ms: 10_000,
            now_unix_ms: 10_500,
            max_review_age_ms: 1_000,
            live_provider_call_performed: true,
            credential_loaded: true,
        })
        .expect("side-effect local fee review should produce blocked report");

        assert_eq!(report.status, FeeScheduleVerificationStatus::Blocked);
        assert!(report.live_provider_call_performed);
        assert!(report.credential_loaded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_VERIFICATION_LIVE_PROVIDER_CALL"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_VERIFICATION_CREDENTIAL_LOADED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn fee_schedule_verification_audit_and_state_reopen_locally() {
        let report = validate_fee_schedule_verification(FeeScheduleVerificationInput {
            schedule: verified_schedule(),
            review_id: "fee-review-audit-state".to_owned(),
            source_reference: "operator-fee-review-audit-state".to_owned(),
            account_tier: "paper-tier".to_owned(),
            maker_taker_tier_verified: true,
            network_fee_verified: true,
            withdrawal_fee_review_required: false,
            withdrawal_fee_reviewed: false,
            reviewed_at_unix_ms: 10_000,
            now_unix_ms: 10_500,
            max_review_age_ms: 1_000,
            live_provider_call_performed: false,
            credential_loaded: false,
        })
        .expect("fee review should validate");

        let audit_path = unique_temp_path("fee-verification-audit", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let record = append_fee_schedule_verification_audit(&mut journal, &report, 1_700_000_060)
            .expect("audit append should succeed");
        assert_eq!(record.event.subsystem, FEE_STATE_SUBSYSTEM);
        assert_eq!(record.event.actor, "fee-schedule-verification");

        let next_sequence = journal.next_sequence();
        let mut invalid = report.clone();
        invalid.live_provider_call_performed = true;
        invalid.status = FeeScheduleVerificationStatus::ReadyForLocalReview;
        assert!(
            append_fee_schedule_verification_audit(&mut journal, &invalid, 1_700_000_061).is_err()
        );
        assert_eq!(journal.next_sequence(), next_sequence);

        let mut store = InMemoryStateStore::new();
        let checkpoint =
            persist_fee_schedule_verification_checkpoint(&mut store, &report, 1_700_000_062)
                .expect("checkpoint persist should succeed");
        assert_eq!(checkpoint.key, FEE_LAST_VERIFICATION_CHECKPOINT_KEY);

        let recovered = store
            .get_checkpoint(FEE_LAST_VERIFICATION_CHECKPOINT_KEY)
            .expect("state read should succeed")
            .expect("checkpoint should exist");
        let recovered_report: FeeScheduleVerificationReport =
            serde_json::from_str(&recovered.value).expect("checkpoint JSON should decode");
        assert_eq!(recovered_report, report);
        assert_eq!(
            recovered_report.status,
            FeeScheduleVerificationStatus::ReadyForLocalReview
        );
    }

    fn verified_schedule() -> FeeSchedule {
        FeeSchedule {
            venue: VenueRef {
                kind: VenueKind::Cex,
                name: "paper-coinbase".to_owned(),
            },
            pair: Some(MarketPair::new("BTC", "USD").expect("pair should validate")),
            maker_bps: 2.0,
            taker_bps: 6.0,
            network_fee_quote: 0.50,
            externally_verified: true,
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
            "arbyclaw-fees-{label}-{}-{nanos}-{n}.{extension}",
            std::process::id()
        ))
    }
}
