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

/// Stable version for local fee schedule reconciliation review reports.
pub const FEE_SCHEDULE_RECONCILIATION_REVIEW_VERSION: &str =
    "fee-schedule-reconciliation-review-v1";

/// Stable version for local fee live-provider boundary review reports.
pub const FEE_LIVE_PROVIDER_BOUNDARY_REVIEW_VERSION: &str = "fee-live-provider-boundary-review-v1";

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

/// Local reconciliation request over current and intentionally blocked fee reviews.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeScheduleReconciliationReviewRequest {
    /// Stable non-secret reconciliation review id.
    pub review_id: String,
    /// Current fee review expected to be ready for local review.
    pub current_review: FeeScheduleVerificationReport,
    /// Blocked fee review expected to cover fail-closed paths.
    pub blocked_review: FeeScheduleVerificationReport,
    /// Minimum non-secret external evidence references still required.
    pub min_remaining_external_evidence: usize,
    /// Non-secret references to remaining external fee evidence gaps.
    pub remaining_external_evidence: Vec<String>,
    /// Whether any live provider/API call was performed by this review. Must remain false.
    pub live_provider_call_performed: bool,
    /// Whether any credential was loaded by this review. Must remain false.
    pub credential_loaded: bool,
    /// Whether this review claims production readiness. Must remain false.
    pub production_ready_claimed: bool,
}

/// Local fee reconciliation review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeeScheduleReconciliationReviewStatus {
    /// Local current/blocked fee evidence is coherent and ready for local review.
    ReadyForLocalReview,
    /// Local fee evidence is incomplete or unsafe.
    Blocked,
}

/// Non-secret local fee reconciliation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeScheduleReconciliationReviewReport {
    /// Stable report schema version.
    pub version: String,
    /// Stable non-secret reconciliation review id.
    pub review_id: String,
    /// Review status.
    pub status: FeeScheduleReconciliationReviewStatus,
    /// Venue name for the reconciled current fee review.
    pub venue_name: String,
    /// Whether the current fee review is ready for local review.
    pub current_fee_review_ready: bool,
    /// Whether unverified fee schedules are blocked.
    pub unverified_schedule_blocked: bool,
    /// Whether missing maker/taker tier review is blocked.
    pub maker_taker_unverified_blocked: bool,
    /// Whether missing network/gas fee review is blocked.
    pub network_fee_unverified_blocked: bool,
    /// Whether missing required withdrawal-fee review is blocked.
    pub withdrawal_fee_unreviewed_blocked: bool,
    /// Whether stale fee review evidence is blocked.
    pub stale_review_blocked: bool,
    /// Whether unresolved external fee evidence references were recorded.
    pub remaining_external_evidence_recorded: bool,
    /// Number of unresolved external fee evidence references.
    pub remaining_external_evidence_count: usize,
    /// Whether any live provider/API call was performed. Always false here.
    pub live_provider_call_performed: bool,
    /// Whether any credential was loaded. Always false here.
    pub credential_loaded: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Local fee live-provider boundary request.
///
/// This records whether local fee validation prerequisites exist and keeps the
/// missing provider-backed fee/account/gas/withdrawal evidence explicit. It
/// does not call exchanges, query RPC endpoints, load credentials, sign,
/// broadcast, withdraw, bridge, or approve production readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeLiveProviderBoundaryReviewRequest {
    /// Stable non-secret live-provider boundary review id.
    pub review_id: String,
    /// Local fee reconciliation prerequisite.
    pub reconciliation_review: FeeScheduleReconciliationReviewReport,
    /// Whether provider/API maker-taker fee evidence is available.
    pub provider_fee_evidence_available: bool,
    /// Whether account-tier fee evidence is available.
    pub account_tier_evidence_available: bool,
    /// Whether gas/RPC/network fee evidence is available.
    pub gas_fee_evidence_available: bool,
    /// Whether withdrawal-cost evidence is available.
    pub withdrawal_cost_evidence_available: bool,
    /// Minimum number of remaining external provider-backed evidence items.
    pub min_remaining_external_evidence: usize,
    /// Non-secret descriptions of external evidence still required.
    pub remaining_external_evidence: Vec<String>,
    /// Whether any live provider/API call was performed. Must remain false.
    pub live_provider_call_performed: bool,
    /// Whether any RPC call was performed. Must remain false.
    pub rpc_call_performed: bool,
    /// Whether any credential was loaded. Must remain false.
    pub credential_loaded: bool,
    /// Whether signing or broadcast was performed. Must remain false.
    pub signing_or_broadcast_performed: bool,
    /// Whether any withdrawal was performed. Must remain false.
    pub withdrawal_performed: bool,
    /// Whether this review claims production readiness. Must remain false.
    pub production_ready_claimed: bool,
}

/// Local fee live-provider boundary review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeeLiveProviderBoundaryReviewStatus {
    /// Local prerequisites exist but provider-backed fee validation is missing.
    BlockedPendingProviderFeeValidation,
    /// The boundary is unsafe or internally incomplete.
    Blocked,
}

/// Non-secret local fee live-provider boundary report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeeLiveProviderBoundaryReviewReport {
    /// Stable report schema version.
    pub version: String,
    /// Stable non-secret live-provider boundary review id.
    pub review_id: String,
    /// Review status.
    pub status: FeeLiveProviderBoundaryReviewStatus,
    /// Whether local fee schedule verification and reconciliation are ready.
    pub fee_reconciliation_review_ready: bool,
    /// Whether provider/API maker-taker fee evidence is available.
    pub provider_fee_evidence_available: bool,
    /// Whether account-tier fee evidence is available.
    pub account_tier_evidence_available: bool,
    /// Whether gas/RPC/network fee evidence is available.
    pub gas_fee_evidence_available: bool,
    /// Whether withdrawal-cost evidence is available.
    pub withdrawal_cost_evidence_available: bool,
    /// Whether unresolved external provider-backed evidence references were recorded.
    pub remaining_external_evidence_recorded: bool,
    /// Number of unresolved external provider-backed evidence references.
    pub remaining_external_evidence_count: usize,
    /// Whether any live provider/API call was performed. Always false here.
    pub live_provider_call_performed: bool,
    /// Whether any RPC call was performed. Always false here.
    pub rpc_call_performed: bool,
    /// Whether any credential was loaded. Always false here.
    pub credential_loaded: bool,
    /// Whether signing or broadcast was performed. Always false here.
    pub signing_or_broadcast_performed: bool,
    /// Whether any withdrawal was performed. Always false here.
    pub withdrawal_performed: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local blocker descriptions.
    pub blockers: Vec<String>,
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

impl FeeScheduleReconciliationReviewRequest {
    /// Validate local fee reconciliation request shape.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        self.current_review.validate()?;
        self.blocked_review.validate()?;
        let mut violations = Vec::new();
        validate_text(
            "fee schedule reconciliation review",
            &self.review_id,
            &mut violations,
        );
        if self.min_remaining_external_evidence == 0 {
            violations.push(FeeModelViolation::new(
                "FEE_RECONCILIATION_EXTERNAL_EVIDENCE_FLOOR_ZERO",
                "fee reconciliation must require at least one remaining external evidence reference",
            ));
        }
        if self
            .remaining_external_evidence
            .iter()
            .any(|item| item.trim().is_empty())
        {
            violations.push(FeeModelViolation::new(
                "FEE_RECONCILIATION_EXTERNAL_EVIDENCE_BLANK",
                "fee reconciliation external evidence references must be non-empty",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }
}

impl FeeScheduleReconciliationReviewReport {
    /// Validate local fee reconciliation report invariants.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        let mut violations = Vec::new();
        if self.version != FEE_SCHEDULE_RECONCILIATION_REVIEW_VERSION {
            violations.push(FeeModelViolation::new(
                "FEE_RECONCILIATION_VERSION_INVALID",
                "fee reconciliation report version is invalid",
            ));
        }
        validate_text(
            "fee schedule reconciliation report",
            &self.review_id,
            &mut violations,
        );
        validate_text(
            "fee schedule reconciliation venue",
            &self.venue_name,
            &mut violations,
        );

        let should_be_ready = self.current_fee_review_ready
            && self.unverified_schedule_blocked
            && self.maker_taker_unverified_blocked
            && self.network_fee_unverified_blocked
            && self.withdrawal_fee_unreviewed_blocked
            && self.stale_review_blocked
            && self.remaining_external_evidence_recorded
            && !self.live_provider_call_performed
            && !self.credential_loaded
            && !self.production_ready;
        if should_be_ready
            && self.status != FeeScheduleReconciliationReviewStatus::ReadyForLocalReview
        {
            violations.push(FeeModelViolation::new(
                "FEE_RECONCILIATION_STATUS_SHOULD_BE_READY",
                "complete local fee reconciliation evidence must be ready for local review",
            ));
        }
        if !should_be_ready && self.status != FeeScheduleReconciliationReviewStatus::Blocked {
            violations.push(FeeModelViolation::new(
                "FEE_RECONCILIATION_STATUS_SHOULD_BLOCK",
                "incomplete local fee reconciliation evidence must be blocked",
            ));
        }
        if self.production_ready {
            violations.push(FeeModelViolation::new(
                "FEE_RECONCILIATION_PRODUCTION_READY_FORBIDDEN",
                "local fee reconciliation must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }
}

impl FeeLiveProviderBoundaryReviewRequest {
    /// Validate local fee live-provider boundary request shape.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        self.reconciliation_review.validate()?;
        let mut violations = Vec::new();
        validate_text(
            "fee live-provider boundary review",
            &self.review_id,
            &mut violations,
        );
        if self.min_remaining_external_evidence == 0 {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_EXTERNAL_EVIDENCE_FLOOR_ZERO",
                "fee live-provider boundary must require remaining external evidence",
            ));
        }
        if self
            .remaining_external_evidence
            .iter()
            .any(|item| item.trim().is_empty())
        {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_EXTERNAL_EVIDENCE_BLANK",
                "fee live-provider boundary evidence references must be non-empty",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(FeeModelError::ValidationFailed { violations })
        }
    }
}

impl FeeLiveProviderBoundaryReviewReport {
    /// Validate local fee live-provider boundary report invariants.
    pub fn validate(&self) -> Result<(), FeeModelError> {
        let mut violations = Vec::new();
        if self.version != FEE_LIVE_PROVIDER_BOUNDARY_REVIEW_VERSION {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_VERSION_INVALID",
                "fee live-provider boundary report version is invalid",
            ));
        }
        validate_text(
            "fee live-provider boundary report",
            &self.review_id,
            &mut violations,
        );

        let no_side_effects = !self.live_provider_call_performed
            && !self.rpc_call_performed
            && !self.credential_loaded
            && !self.signing_or_broadcast_performed
            && !self.withdrawal_performed
            && !self.production_ready;
        let provider_evidence_missing = !self.provider_fee_evidence_available
            && !self.account_tier_evidence_available
            && !self.gas_fee_evidence_available
            && !self.withdrawal_cost_evidence_available;
        let should_be_pending_provider_validation = self.fee_reconciliation_review_ready
            && provider_evidence_missing
            && self.remaining_external_evidence_recorded
            && no_side_effects;
        if should_be_pending_provider_validation
            && self.status
                != FeeLiveProviderBoundaryReviewStatus::BlockedPendingProviderFeeValidation
        {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_STATUS_SHOULD_BE_PENDING_VALIDATION",
                "complete local fee prerequisites with missing provider evidence must remain blocked pending provider fee validation",
            ));
        }
        if !should_be_pending_provider_validation
            && self.status != FeeLiveProviderBoundaryReviewStatus::Blocked
        {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_STATUS_SHOULD_BLOCK",
                "unsafe or incomplete fee live-provider boundary evidence must be blocked",
            ));
        }
        if self.production_ready {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_PRODUCTION_READY_FORBIDDEN",
                "fee live-provider boundary must not approve production readiness",
            ));
        }
        if self.blockers.is_empty() {
            violations.push(FeeModelViolation::new(
                "FEE_LIVE_PROVIDER_BLOCKERS_EMPTY",
                "fee live-provider boundary must record unresolved blockers",
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

/// Review local current/blocked fee schedule evidence without external calls.
pub fn review_fee_schedule_reconciliation(
    request: FeeScheduleReconciliationReviewRequest,
) -> Result<FeeScheduleReconciliationReviewReport, FeeModelError> {
    request.validate()?;

    let current_fee_review_ready = request.current_review.status
        == FeeScheduleVerificationStatus::ReadyForLocalReview
        && request.current_review.schedule_externally_verified
        && request.current_review.maker_taker_tier_verified
        && request.current_review.network_fee_verified
        && !request.current_review.stale_review_blocked
        && !request.current_review.live_provider_call_performed
        && !request.current_review.credential_loaded
        && !request.current_review.production_ready;
    let blocked_review_failed_closed = request.blocked_review.status
        == FeeScheduleVerificationStatus::Blocked
        && !request.blocked_review.live_provider_call_performed
        && !request.blocked_review.credential_loaded
        && !request.blocked_review.production_ready;
    let has_blocker = |code: &str| {
        request
            .blocked_review
            .violation_codes
            .iter()
            .any(|violation| violation == code)
    };
    let unverified_schedule_blocked =
        blocked_review_failed_closed && has_blocker("FEE_VERIFICATION_SCHEDULE_UNVERIFIED");
    let maker_taker_unverified_blocked =
        blocked_review_failed_closed && has_blocker("FEE_VERIFICATION_MAKER_TAKER_UNVERIFIED");
    let network_fee_unverified_blocked =
        blocked_review_failed_closed && has_blocker("FEE_VERIFICATION_NETWORK_FEE_UNVERIFIED");
    let withdrawal_fee_unreviewed_blocked =
        blocked_review_failed_closed && has_blocker("FEE_VERIFICATION_WITHDRAWAL_FEE_UNREVIEWED");
    let stale_review_blocked =
        blocked_review_failed_closed && has_blocker("FEE_VERIFICATION_REVIEW_STALE");
    let remaining_external_evidence_recorded =
        request.remaining_external_evidence.len() >= request.min_remaining_external_evidence;
    let live_provider_call_performed = request.live_provider_call_performed
        || request.current_review.live_provider_call_performed
        || request.blocked_review.live_provider_call_performed;
    let credential_loaded = request.credential_loaded
        || request.current_review.credential_loaded
        || request.blocked_review.credential_loaded;
    let production_ready_claimed = request.production_ready_claimed
        || request.current_review.production_ready
        || request.blocked_review.production_ready;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !current_fee_review_ready,
        "FEE_RECONCILIATION_CURRENT_REVIEW_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !unverified_schedule_blocked,
        "FEE_RECONCILIATION_UNVERIFIED_SCHEDULE_NOT_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        !maker_taker_unverified_blocked,
        "FEE_RECONCILIATION_MAKER_TAKER_NOT_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        !network_fee_unverified_blocked,
        "FEE_RECONCILIATION_NETWORK_FEE_NOT_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        !withdrawal_fee_unreviewed_blocked,
        "FEE_RECONCILIATION_WITHDRAWAL_FEE_NOT_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        !stale_review_blocked,
        "FEE_RECONCILIATION_STALE_REVIEW_NOT_BLOCKED",
    );
    push_if(
        &mut violation_codes,
        !remaining_external_evidence_recorded,
        "FEE_RECONCILIATION_EXTERNAL_EVIDENCE_MISSING",
    );
    push_if(
        &mut violation_codes,
        live_provider_call_performed,
        "FEE_RECONCILIATION_LIVE_PROVIDER_CALL",
    );
    push_if(
        &mut violation_codes,
        credential_loaded,
        "FEE_RECONCILIATION_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        production_ready_claimed,
        "FEE_RECONCILIATION_PRODUCTION_READY_CLAIMED",
    );

    let report = FeeScheduleReconciliationReviewReport {
        version: FEE_SCHEDULE_RECONCILIATION_REVIEW_VERSION.to_owned(),
        review_id: request.review_id,
        status: if violation_codes.is_empty() {
            FeeScheduleReconciliationReviewStatus::ReadyForLocalReview
        } else {
            FeeScheduleReconciliationReviewStatus::Blocked
        },
        venue_name: request.current_review.venue_name,
        current_fee_review_ready,
        unverified_schedule_blocked,
        maker_taker_unverified_blocked,
        network_fee_unverified_blocked,
        withdrawal_fee_unreviewed_blocked,
        stale_review_blocked,
        remaining_external_evidence_recorded,
        remaining_external_evidence_count: request.remaining_external_evidence.len(),
        live_provider_call_performed,
        credential_loaded,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Review local fee live-provider prerequisites without external calls.
pub fn review_fee_live_provider_boundary(
    request: FeeLiveProviderBoundaryReviewRequest,
) -> Result<FeeLiveProviderBoundaryReviewReport, FeeModelError> {
    request.validate()?;

    let fee_reconciliation_review_ready = request.reconciliation_review.status
        == FeeScheduleReconciliationReviewStatus::ReadyForLocalReview
        && request.reconciliation_review.current_fee_review_ready
        && request.reconciliation_review.unverified_schedule_blocked
        && request.reconciliation_review.maker_taker_unverified_blocked
        && request.reconciliation_review.network_fee_unverified_blocked
        && request
            .reconciliation_review
            .withdrawal_fee_unreviewed_blocked
        && request.reconciliation_review.stale_review_blocked
        && !request.reconciliation_review.live_provider_call_performed
        && !request.reconciliation_review.credential_loaded
        && !request.reconciliation_review.production_ready;
    let remaining_external_evidence_recorded =
        request.remaining_external_evidence.len() >= request.min_remaining_external_evidence;
    let live_provider_call_performed = request.live_provider_call_performed
        || request.reconciliation_review.live_provider_call_performed;
    let credential_loaded =
        request.credential_loaded || request.reconciliation_review.credential_loaded;
    let production_ready_claimed =
        request.production_ready_claimed || request.reconciliation_review.production_ready;

    let mut violation_codes = Vec::new();
    push_if(
        &mut violation_codes,
        !fee_reconciliation_review_ready,
        "FEE_LIVE_PROVIDER_RECONCILIATION_NOT_READY",
    );
    push_if(
        &mut violation_codes,
        !remaining_external_evidence_recorded,
        "FEE_LIVE_PROVIDER_EXTERNAL_EVIDENCE_MISSING",
    );
    push_if(
        &mut violation_codes,
        live_provider_call_performed,
        "FEE_LIVE_PROVIDER_CALL_PERFORMED",
    );
    push_if(
        &mut violation_codes,
        request.rpc_call_performed,
        "FEE_LIVE_PROVIDER_RPC_CALL_PERFORMED",
    );
    push_if(
        &mut violation_codes,
        credential_loaded,
        "FEE_LIVE_PROVIDER_CREDENTIAL_LOADED",
    );
    push_if(
        &mut violation_codes,
        request.signing_or_broadcast_performed,
        "FEE_LIVE_PROVIDER_SIGNING_OR_BROADCAST",
    );
    push_if(
        &mut violation_codes,
        request.withdrawal_performed,
        "FEE_LIVE_PROVIDER_WITHDRAWAL_PERFORMED",
    );
    push_if(
        &mut violation_codes,
        production_ready_claimed,
        "FEE_LIVE_PROVIDER_PRODUCTION_READY_CLAIMED",
    );

    let mut blockers = Vec::new();
    if !request.provider_fee_evidence_available {
        blockers.push("provider-backed maker/taker fee validation missing".to_owned());
    }
    if !request.account_tier_evidence_available {
        blockers.push("external account-tier fee validation missing".to_owned());
    }
    if !request.gas_fee_evidence_available {
        blockers.push("gas/RPC/network fee validation missing".to_owned());
    }
    if !request.withdrawal_cost_evidence_available {
        blockers.push("withdrawal-cost validation missing".to_owned());
    }
    if !remaining_external_evidence_recorded {
        blockers.push("remaining external fee evidence references below floor".to_owned());
    }

    let provider_evidence_missing = !request.provider_fee_evidence_available
        && !request.account_tier_evidence_available
        && !request.gas_fee_evidence_available
        && !request.withdrawal_cost_evidence_available;
    let safe_pending_provider_validation = fee_reconciliation_review_ready
        && provider_evidence_missing
        && remaining_external_evidence_recorded
        && !live_provider_call_performed
        && !request.rpc_call_performed
        && !credential_loaded
        && !request.signing_or_broadcast_performed
        && !request.withdrawal_performed
        && !production_ready_claimed;

    let report = FeeLiveProviderBoundaryReviewReport {
        version: FEE_LIVE_PROVIDER_BOUNDARY_REVIEW_VERSION.to_owned(),
        review_id: request.review_id,
        status: if safe_pending_provider_validation {
            FeeLiveProviderBoundaryReviewStatus::BlockedPendingProviderFeeValidation
        } else {
            FeeLiveProviderBoundaryReviewStatus::Blocked
        },
        fee_reconciliation_review_ready,
        provider_fee_evidence_available: request.provider_fee_evidence_available,
        account_tier_evidence_available: request.account_tier_evidence_available,
        gas_fee_evidence_available: request.gas_fee_evidence_available,
        withdrawal_cost_evidence_available: request.withdrawal_cost_evidence_available,
        remaining_external_evidence_recorded,
        remaining_external_evidence_count: request.remaining_external_evidence.len(),
        live_provider_call_performed,
        rpc_call_performed: request.rpc_call_performed,
        credential_loaded,
        signing_or_broadcast_performed: request.signing_or_broadcast_performed,
        withdrawal_performed: request.withdrawal_performed,
        production_ready: false,
        blockers,
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
        review_fee_live_provider_boundary, review_fee_schedule_reconciliation,
        validate_fee_schedule_verification, FeeAdjustedEdge, FeeLiveProviderBoundaryReviewRequest,
        FeeLiveProviderBoundaryReviewStatus, FeeSchedule, FeeScheduleReconciliationReviewRequest,
        FeeScheduleReconciliationReviewStatus, FeeScheduleVerificationInput,
        FeeScheduleVerificationReport, FeeScheduleVerificationStatus, LiquidityRole,
        FEE_LAST_VERIFICATION_CHECKPOINT_KEY, FEE_STATE_SUBSYSTEM,
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
    fn fee_schedule_reconciliation_accepts_local_current_and_blocked_evidence() {
        let report = review_fee_schedule_reconciliation(fee_reconciliation_request(false, 4))
            .expect("fee reconciliation should validate");

        assert_eq!(
            report.status,
            FeeScheduleReconciliationReviewStatus::ReadyForLocalReview
        );
        assert!(report.current_fee_review_ready);
        assert!(report.unverified_schedule_blocked);
        assert!(report.maker_taker_unverified_blocked);
        assert!(report.network_fee_unverified_blocked);
        assert!(report.withdrawal_fee_unreviewed_blocked);
        assert!(report.stale_review_blocked);
        assert!(report.remaining_external_evidence_recorded);
        assert_eq!(report.remaining_external_evidence_count, 4);
        assert!(!report.live_provider_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn fee_schedule_reconciliation_blocks_missing_external_evidence() {
        let report = review_fee_schedule_reconciliation(fee_reconciliation_request(false, 5))
            .expect("fee reconciliation should produce blocked report");

        assert_eq!(
            report.status,
            FeeScheduleReconciliationReviewStatus::Blocked
        );
        assert!(!report.remaining_external_evidence_recorded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_RECONCILIATION_EXTERNAL_EVIDENCE_MISSING"));
        assert!(!report.live_provider_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.production_ready);
    }

    #[test]
    fn fee_schedule_reconciliation_fails_closed_on_side_effect_claims() {
        let report = review_fee_schedule_reconciliation(fee_reconciliation_request(true, 4))
            .expect("fee reconciliation should produce blocked report");

        assert_eq!(
            report.status,
            FeeScheduleReconciliationReviewStatus::Blocked
        );
        assert!(report.live_provider_call_performed);
        assert!(report.credential_loaded);
        assert!(!report.production_ready);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_RECONCILIATION_LIVE_PROVIDER_CALL"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_RECONCILIATION_CREDENTIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_RECONCILIATION_PRODUCTION_READY_CLAIMED"));
    }

    #[test]
    fn fee_live_provider_boundary_blocks_pending_provider_fee_validation() {
        let report =
            review_fee_live_provider_boundary(fee_live_provider_boundary_request(false, 4))
                .expect("fee live-provider boundary should validate");

        assert_eq!(
            report.status,
            FeeLiveProviderBoundaryReviewStatus::BlockedPendingProviderFeeValidation
        );
        assert!(report.fee_reconciliation_review_ready);
        assert!(!report.provider_fee_evidence_available);
        assert!(!report.account_tier_evidence_available);
        assert!(!report.gas_fee_evidence_available);
        assert!(!report.withdrawal_cost_evidence_available);
        assert!(report.remaining_external_evidence_recorded);
        assert_eq!(report.remaining_external_evidence_count, 4);
        assert_eq!(report.blockers.len(), 4);
        assert!(!report.live_provider_call_performed);
        assert!(!report.rpc_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.withdrawal_performed);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn fee_live_provider_boundary_fails_closed_on_side_effect_claims() {
        let report = review_fee_live_provider_boundary(fee_live_provider_boundary_request(true, 4))
            .expect("fee live-provider boundary should produce blocked report");

        assert_eq!(report.status, FeeLiveProviderBoundaryReviewStatus::Blocked);
        assert!(report.live_provider_call_performed);
        assert!(report.rpc_call_performed);
        assert!(report.credential_loaded);
        assert!(report.signing_or_broadcast_performed);
        assert!(report.withdrawal_performed);
        assert!(!report.production_ready);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_LIVE_PROVIDER_CALL_PERFORMED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_LIVE_PROVIDER_RPC_CALL_PERFORMED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_LIVE_PROVIDER_CREDENTIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_LIVE_PROVIDER_SIGNING_OR_BROADCAST"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_LIVE_PROVIDER_WITHDRAWAL_PERFORMED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "FEE_LIVE_PROVIDER_PRODUCTION_READY_CLAIMED"));
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

    fn fee_reconciliation_request(
        side_effect_claimed: bool,
        min_remaining_external_evidence: usize,
    ) -> FeeScheduleReconciliationReviewRequest {
        let current_review = validate_fee_schedule_verification(FeeScheduleVerificationInput {
            schedule: verified_schedule(),
            review_id: "fee-review-current-reconciliation".to_owned(),
            source_reference: "operator-fee-review-current-reconciliation".to_owned(),
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
        .expect("current fee review should validate");
        let mut blocked_schedule = verified_schedule();
        blocked_schedule.externally_verified = false;
        let blocked_review = validate_fee_schedule_verification(FeeScheduleVerificationInput {
            schedule: blocked_schedule,
            review_id: "fee-review-blocked-reconciliation".to_owned(),
            source_reference: "operator-fee-review-blocked-reconciliation".to_owned(),
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
        .expect("blocked fee review should validate");

        FeeScheduleReconciliationReviewRequest {
            review_id: "fee-schedule-reconciliation-review".to_owned(),
            current_review,
            blocked_review,
            min_remaining_external_evidence,
            remaining_external_evidence: vec![
                "venue account-tier fee evidence".to_owned(),
                "provider/API maker-taker fee evidence".to_owned(),
                "chain gas/network fee evidence".to_owned(),
                "withdrawal-cost fee evidence".to_owned(),
            ],
            live_provider_call_performed: side_effect_claimed,
            credential_loaded: side_effect_claimed,
            production_ready_claimed: side_effect_claimed,
        }
    }

    fn fee_live_provider_boundary_request(
        side_effect_claimed: bool,
        min_remaining_external_evidence: usize,
    ) -> FeeLiveProviderBoundaryReviewRequest {
        FeeLiveProviderBoundaryReviewRequest {
            review_id: "fee-live-provider-boundary-review".to_owned(),
            reconciliation_review: review_fee_schedule_reconciliation(fee_reconciliation_request(
                false, 4,
            ))
            .expect("fee reconciliation should validate"),
            provider_fee_evidence_available: false,
            account_tier_evidence_available: false,
            gas_fee_evidence_available: false,
            withdrawal_cost_evidence_available: false,
            min_remaining_external_evidence,
            remaining_external_evidence: vec![
                "provider-backed maker/taker fee evidence".to_owned(),
                "external account-tier fee evidence".to_owned(),
                "gas/RPC/network fee evidence".to_owned(),
                "withdrawal-cost fee evidence".to_owned(),
            ],
            live_provider_call_performed: side_effect_claimed,
            rpc_call_performed: side_effect_claimed,
            credential_loaded: side_effect_claimed,
            signing_or_broadcast_performed: side_effect_claimed,
            withdrawal_performed: side_effect_claimed,
            production_ready_claimed: side_effect_claimed,
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
