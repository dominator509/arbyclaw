#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditError, AuditEvent, AuditEventKind, AuditRecord, AuditValue,
    ExecutionIntent, ExecutionIntentKind, ExecutionPlanDraft, ExecutionPlanStatus,
    ExecutionPlannerError, ExecutionScope, PlannerPolicyStatus, PolicyDecision, PolicyEngine,
    StateCheckpoint, StateStore, StateStoreError, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error, fmt};

/// Stable execution-adapter framework version for audit, replay, and handoff surfaces.
pub const EXECUTION_ADAPTER_FRAMEWORK_VERSION: &str = "phase-11-execution-adapter-framework-v1";

/// State-store subsystem name for execution-adapter checkpoints.
pub const EXECUTION_ADAPTER_STATE_SUBSYSTEM: &str = "execution-adapter";

/// State-store key for the latest deterministic execution-adapter run.
pub const EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY: &str = "execution-adapter:last-run";

/// State-store key for the latest deterministic execution-adapter recovery plan.
pub const EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY: &str =
    "execution-adapter:last-recovery-plan";

/// Phase 11 adapter configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdapterConfig {
    /// Stable non-secret adapter id.
    pub adapter_id: String,
    /// Maximum number of intents a single adapter run may model.
    pub max_plan_intents: usize,
    /// Whether deterministic paper fill records may be modeled without external submission.
    pub model_paper_fills: bool,
    /// Whether adapter submission is enabled. Phase 11 requires this to remain false.
    pub adapter_submission_enabled: bool,
}

impl Default for ExecutionAdapterConfig {
    fn default() -> Self {
        Self {
            adapter_id: "phase-11-deterministic-boundary".to_owned(),
            max_plan_intents: 4,
            model_paper_fills: true,
            adapter_submission_enabled: false,
        }
    }
}

impl ExecutionAdapterConfig {
    /// Validate adapter settings before a plan is evaluated.
    pub fn validate(&self) -> Result<(), ExecutionAdapterError> {
        let mut violations = Vec::new();
        validate_id("adapter", &self.adapter_id, &mut violations);

        if self.max_plan_intents == 0 {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_MAX_INTENTS_ZERO",
                "max_plan_intents must be positive",
            ));
        }

        if self.adapter_submission_enabled {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_SUBMISSION_DENIED_IN_PHASE_11",
                "Phase 11 adapter framework must not enable external adapter submission",
            ));
        }

        finish_validation(violations)
    }
}

/// One deterministic execution-adapter request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdapterRequest {
    /// Stable request id for audit and replay.
    pub id: String,
    /// Draft plan produced by the Phase 10 planner.
    pub plan: ExecutionPlanDraft,
    /// Adapter-boundary settings.
    pub config: ExecutionAdapterConfig,
    /// Runtime clock in Unix milliseconds used for records.
    pub now_unix_ms: u64,
}

impl ExecutionAdapterRequest {
    /// Validate a request before producing an adapter run record.
    pub fn validate(&self) -> Result<(), ExecutionAdapterError> {
        let mut violations = Vec::new();
        validate_id("adapter request", &self.id, &mut violations);

        if let Err(ExecutionAdapterError::ValidationFailed {
            violations: config_violations,
        }) = self.config.validate()
        {
            violations.extend(config_violations);
        }

        if let Err(error) = self.plan.validate() {
            collect_planner_error(error, &mut violations);
        }

        if self.plan.scope == ExecutionScope::Live {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_PLAN_LIVE_SCOPE_DENIED",
                "Phase 11 adapter framework rejects live-scope plans",
            ));
        }

        if self.plan.adapter_submission_enabled {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_PLAN_SUBMISSION_FLAG_DENIED",
                "Phase 11 adapter framework rejects plans with adapter submission enabled",
            ));
        }

        if self.plan.intents.len() > self.config.max_plan_intents {
            violations.push(ExecutionAdapterViolation::new_owned(
                "ADAPTER_TOO_MANY_INTENTS",
                format!(
                    "plan has {} intents but adapter max_plan_intents is {}",
                    self.plan.intents.len(),
                    self.config.max_plan_intents
                ),
            ));
        }

        if self.now_unix_ms == 0 {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_TIME_ZERO",
                "now_unix_ms must be non-zero",
            ));
        }

        finish_validation(violations)
    }
}

/// Overall run status for one adapter-boundary evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionAdapterRunStatus {
    /// Observe-only plan was recorded without execution.
    ObserveRecorded,
    /// Paper plan produced deterministic model fill and reconciliation records.
    PaperModelComplete,
    /// External submission was blocked by Phase 11 boundaries.
    SubmissionBlocked,
    /// Adapter boundary denied the plan because policy failed.
    PolicyDenied,
}

/// Adapter lifecycle action modeled for an intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionAdapterAction {
    /// Re-evaluate policy at the adapter boundary.
    PolicyRevalidation,
    /// Record observe-only intent without any execution.
    RecordObservation,
    /// Model a paper execution lifecycle without external submission.
    ModelPaperExecution,
    /// Block any external adapter submission.
    BlockExternalSubmission,
}

/// Status for a single adapter attempt record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionAdapterAttemptStatus {
    /// Policy approved the intent at adapter boundary.
    PolicyApproved,
    /// Policy denied the intent at adapter boundary.
    PolicyDenied,
    /// Observe-only intent was recorded.
    ObservationRecorded,
    /// Paper fill was modeled deterministically without external submission.
    PaperModelFilled,
    /// External adapter submission was blocked.
    SubmissionBlocked,
    /// Intent type is unsupported by this boundary implementation.
    UnsupportedIntent,
}

/// Fill lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionFillStatus {
    /// No fill occurred or was modeled.
    NoFill,
    /// Deterministic paper fill was modeled without external submission.
    ModeledFilled,
}

/// Reconciliation lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionReconciliationStatus {
    /// Nothing required reconciliation.
    NotRequired,
    /// Modeled fill reconciled exactly against the source intent.
    Reconciled,
    /// Reconciliation was blocked because no external adapter action occurred.
    Blocked,
}

/// One adapter-boundary attempt record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdapterAttempt {
    /// One-based sequence number.
    pub sequence: u16,
    /// Source intent id.
    pub intent_id: String,
    /// Target venue from the source intent.
    pub venue: VenueRef,
    /// Modeled lifecycle action.
    pub action: ExecutionAdapterAction,
    /// Deterministic status for this attempt.
    pub status: ExecutionAdapterAttemptStatus,
    /// Whether policy was revalidated at the adapter boundary for this attempt.
    pub policy_revalidated: bool,
    /// Whether an external adapter was called. Always false in Phase 11.
    pub submitted_to_external_adapter: bool,
    /// Non-secret reason text.
    pub reason: String,
}

/// One deterministic fill record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionFillRecord {
    /// Stable fill id.
    pub id: String,
    /// Source intent id.
    pub intent_id: String,
    /// Fill status.
    pub status: ExecutionFillStatus,
    /// Modeled filled notional in quote units.
    pub filled_notional_quote: f64,
    /// Modeled total fee in quote units.
    pub fee_quote: f64,
    /// Fill time in Unix milliseconds, or zero when no fill occurred.
    pub filled_at_unix_ms: u64,
    /// External order id is intentionally absent until a future external adapter phase.
    pub external_order_id: Option<String>,
}

/// One deterministic reconciliation record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReconciliationRecord {
    /// Stable reconciliation id.
    pub id: String,
    /// Source intent id.
    pub intent_id: String,
    /// Reconciliation status.
    pub status: ExecutionReconciliationStatus,
    /// Expected notional from the intent.
    pub expected_notional_quote: f64,
    /// Modeled observed notional.
    pub observed_notional_quote: f64,
    /// Difference between expected and modeled observed notional.
    pub difference_quote: f64,
    /// Reconciliation time in Unix milliseconds, or zero when not applicable.
    pub reconciled_at_unix_ms: u64,
}

/// Complete adapter-boundary record for a plan.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdapterRunRecord {
    /// Stable run id.
    pub id: String,
    /// Source request id.
    pub request_id: String,
    /// Source plan id.
    pub plan_id: String,
    /// Execution-adapter framework version.
    pub adapter_framework_version: String,
    /// Adapter id that produced this record.
    pub adapter_id: String,
    /// Plan scope.
    pub scope: ExecutionScope,
    /// Overall run status.
    pub status: ExecutionAdapterRunStatus,
    /// Run creation time.
    pub created_at_unix_ms: u64,
    /// Attempt records.
    pub attempts: Vec<ExecutionAdapterAttempt>,
    /// Fill records.
    pub fills: Vec<ExecutionFillRecord>,
    /// Reconciliation records.
    pub reconciliations: Vec<ExecutionReconciliationRecord>,
    /// Always false in Phase 11.
    pub external_submission_enabled: bool,
    /// Non-secret warnings.
    pub warnings: Vec<String>,
}

/// Local deterministic follow-up action for an adapter run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionAdapterRecoveryAction {
    /// No follow-up is required for a fully modeled/reconciled intent.
    Noop,
    /// Cancel or mark cancelled any unfilled remainder before future progress.
    CancelUnfilledRemainder,
    /// Hedge modeled filled exposure before future progress.
    HedgeFilledExposure,
    /// Escalate to local operator review without adapter submission.
    ManualReview,
}

/// One local recovery step derived from a deterministic adapter run.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdapterRecoveryStep {
    /// One-based sequence number.
    pub sequence: u16,
    /// Source intent id.
    pub intent_id: String,
    /// Local follow-up action.
    pub action: ExecutionAdapterRecoveryAction,
    /// Expected notional from the source intent.
    pub expected_notional_quote: f64,
    /// Modeled filled notional from the adapter run.
    pub filled_notional_quote: f64,
    /// Modeled unfilled notional requiring cancellation or review.
    pub unfilled_notional_quote: f64,
    /// Whether the step requires operator review before future execution.
    pub operator_review_required: bool,
    /// Whether any external submission was performed. Always false here.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Non-secret reason text.
    pub reason: String,
}

/// Local recovery plan for partial/no-fill adapter outcomes.
///
/// This record is planning metadata only. It does not submit cancels, hedges,
/// orders, swaps, transactions, broadcasts, withdrawals, or bridges.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdapterRecoveryPlan {
    /// Execution-adapter framework version.
    pub adapter_framework_version: String,
    /// Source execution plan id.
    pub plan_id: String,
    /// Source adapter run id.
    pub adapter_run_id: String,
    /// Creation timestamp.
    pub created_at_unix_ms: u64,
    /// Local recovery steps.
    pub steps: Vec<ExecutionAdapterRecoveryStep>,
    /// Number of partially filled intents.
    pub partial_fill_count: usize,
    /// Number of no-fill intents.
    pub no_fill_count: usize,
    /// Number of cancel-remainder steps.
    pub cancel_remainder_steps: usize,
    /// Number of hedge-exposure steps.
    pub hedge_exposure_steps: usize,
    /// Whether any operator review is required.
    pub operator_review_required: bool,
    /// Whether any external submission was performed. Always false here.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local plan claims production readiness. Always false.
    pub production_ready: bool,
}

impl ExecutionAdapterRunRecord {
    /// Validate execution-adapter run invariants.
    pub fn validate(&self) -> Result<(), ExecutionAdapterError> {
        let mut violations = Vec::new();
        validate_id("adapter run", &self.id, &mut violations);
        validate_id("adapter request", &self.request_id, &mut violations);
        validate_id("plan", &self.plan_id, &mut violations);
        validate_id("adapter", &self.adapter_id, &mut violations);

        if self.adapter_framework_version != EXECUTION_ADAPTER_FRAMEWORK_VERSION {
            violations.push(ExecutionAdapterViolation::new_owned(
                "ADAPTER_VERSION_MISMATCH",
                format!(
                    "adapter_framework_version must be {EXECUTION_ADAPTER_FRAMEWORK_VERSION}, got {}",
                    self.adapter_framework_version
                ),
            ));
        }

        if self.scope == ExecutionScope::Live {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_RUN_LIVE_SCOPE_DENIED",
                "Phase 11 run records must not use live scope",
            ));
        }

        if self.external_submission_enabled {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_EXTERNAL_SUBMISSION_ENABLED",
                "Phase 11 run records must never enable external submission",
            ));
        }

        if self.attempts.is_empty() {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_ATTEMPTS_EMPTY",
                "adapter run must contain attempt records",
            ));
        }

        if self.fills.len() != self.attempts.len() {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_FILL_COUNT_MISMATCH",
                "adapter run must contain exactly one fill record per attempt",
            ));
        }

        if self.reconciliations.len() != self.attempts.len() {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_RECONCILIATION_COUNT_MISMATCH",
                "adapter run must contain exactly one reconciliation record per attempt",
            ));
        }

        let mut attempt_sequences = HashSet::new();
        let mut attempt_intent_ids = HashSet::new();
        for attempt in &self.attempts {
            if !attempt_sequences.insert(attempt.sequence) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_DUPLICATE_ATTEMPT_SEQUENCE",
                    format!(
                        "adapter run contains duplicate attempt sequence {}",
                        attempt.sequence
                    ),
                ));
            }
            if !attempt_intent_ids.insert(attempt.intent_id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_DUPLICATE_ATTEMPT_INTENT_ID",
                    format!(
                        "adapter run contains duplicate attempt intent id {}",
                        attempt.intent_id
                    ),
                ));
            }
            if !attempt.policy_revalidated {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_ATTEMPT_POLICY_NOT_REVALIDATED",
                    format!(
                        "attempt {} did not record adapter-boundary policy revalidation",
                        attempt.sequence
                    ),
                ));
            }
            if attempt.submitted_to_external_adapter {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_ATTEMPT_EXTERNAL_SUBMISSION",
                    format!(
                        "attempt {} submitted to an external adapter",
                        attempt.sequence
                    ),
                ));
            }
        }

        let mut fill_ids = HashSet::new();
        let mut fill_intent_ids = HashSet::new();
        for fill in &self.fills {
            if !fill_ids.insert(fill.id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_DUPLICATE_FILL_ID",
                    format!("adapter run contains duplicate fill id {}", fill.id),
                ));
            }
            if !fill_intent_ids.insert(fill.intent_id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_DUPLICATE_FILL_INTENT_ID",
                    format!(
                        "adapter run contains duplicate fill intent id {}",
                        fill.intent_id
                    ),
                ));
            }
            if !attempt_intent_ids.contains(fill.intent_id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_FILL_UNKNOWN_INTENT",
                    format!(
                        "fill {} references unknown intent {}",
                        fill.id, fill.intent_id
                    ),
                ));
            }
            if fill.external_order_id.is_some() {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_EXTERNAL_ORDER_ID_PRESENT",
                    format!("fill {} must not contain an external order id", fill.id),
                ));
            }
            if !fill.filled_notional_quote.is_finite() || fill.filled_notional_quote < 0.0 {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_FILL_NOTIONAL_INVALID",
                    format!("fill {} has invalid filled_notional_quote", fill.id),
                ));
            }
            if !fill.fee_quote.is_finite() || fill.fee_quote < 0.0 {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_FILL_FEE_INVALID",
                    format!("fill {} has invalid fee_quote", fill.id),
                ));
            }
        }

        let mut reconciliation_ids = HashSet::new();
        let mut reconciliation_intent_ids = HashSet::new();
        for reconciliation in &self.reconciliations {
            if !reconciliation_ids.insert(reconciliation.id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_DUPLICATE_RECONCILIATION_ID",
                    format!(
                        "adapter run contains duplicate reconciliation id {}",
                        reconciliation.id
                    ),
                ));
            }
            if !reconciliation_intent_ids.insert(reconciliation.intent_id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_DUPLICATE_RECONCILIATION_INTENT_ID",
                    format!(
                        "adapter run contains duplicate reconciliation intent id {}",
                        reconciliation.intent_id
                    ),
                ));
            }
            if !attempt_intent_ids.contains(reconciliation.intent_id.as_str()) {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_RECONCILIATION_UNKNOWN_INTENT",
                    format!(
                        "reconciliation {} references unknown intent {}",
                        reconciliation.id, reconciliation.intent_id
                    ),
                ));
            }
            if !reconciliation.expected_notional_quote.is_finite()
                || reconciliation.expected_notional_quote < 0.0
                || !reconciliation.observed_notional_quote.is_finite()
                || reconciliation.observed_notional_quote < 0.0
                || !reconciliation.difference_quote.is_finite()
            {
                violations.push(ExecutionAdapterViolation::new_owned(
                    "ADAPTER_RECONCILIATION_AMOUNT_INVALID",
                    format!("reconciliation {} has invalid amounts", reconciliation.id),
                ));
            }
        }

        finish_validation(violations)
    }
}

impl ExecutionAdapterRecoveryPlan {
    /// Validate local adapter recovery plan invariants.
    pub fn validate(&self) -> Result<(), ExecutionAdapterError> {
        let mut violations = Vec::new();
        validate_id("plan", &self.plan_id, &mut violations);
        validate_id("adapter run", &self.adapter_run_id, &mut violations);

        if self.adapter_framework_version != EXECUTION_ADAPTER_FRAMEWORK_VERSION {
            violations.push(ExecutionAdapterViolation::new_owned(
                "ADAPTER_RECOVERY_VERSION_MISMATCH",
                format!(
                    "adapter_framework_version must be {EXECUTION_ADAPTER_FRAMEWORK_VERSION}, got {}",
                    self.adapter_framework_version
                ),
            ));
        }

        if self.created_at_unix_ms == 0 {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_TIME_ZERO",
                "recovery plan timestamp must be non-zero",
            ));
        }
        if self.steps.is_empty() {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_STEPS_EMPTY",
                "recovery plan must contain at least one step",
            ));
        }

        let partial_count = self
            .steps
            .iter()
            .filter(|step| step.filled_notional_quote > 0.0 && step.unfilled_notional_quote > 0.0)
            .map(|step| step.intent_id.as_str())
            .collect::<HashSet<_>>()
            .len();
        let no_fill_count = self
            .steps
            .iter()
            .filter(|step| {
                step.filled_notional_quote <= f64::EPSILON
                    && step.unfilled_notional_quote > f64::EPSILON
            })
            .map(|step| step.intent_id.as_str())
            .collect::<HashSet<_>>()
            .len();
        let cancel_count = self
            .steps
            .iter()
            .filter(|step| step.action == ExecutionAdapterRecoveryAction::CancelUnfilledRemainder)
            .count();
        let hedge_count = self
            .steps
            .iter()
            .filter(|step| step.action == ExecutionAdapterRecoveryAction::HedgeFilledExposure)
            .count();
        if partial_count != self.partial_fill_count
            || no_fill_count != self.no_fill_count
            || cancel_count != self.cancel_remainder_steps
            || hedge_count != self.hedge_exposure_steps
        {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_COUNTS_MISMATCH",
                "recovery plan summary counts must match recovery steps",
            ));
        }

        if self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
            || self
                .steps
                .iter()
                .any(|step| step.external_submission_performed || step.live_execution_performed)
        {
            violations.push(ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_SIDE_EFFECT_RECORDED",
                "recovery plans must not record external submission, live execution, or readiness",
            ));
        }

        finish_validation(violations)
    }
}

/// Build a deterministic local recovery plan for partial/no-fill adapter outcomes.
pub fn plan_execution_adapter_recovery(
    plan: &ExecutionPlanDraft,
    run: &ExecutionAdapterRunRecord,
    created_at_unix_ms: u64,
) -> Result<ExecutionAdapterRecoveryPlan, ExecutionAdapterError> {
    if let Err(error) = plan.validate() {
        let mut violations = Vec::new();
        collect_planner_error(error, &mut violations);
        return Err(ExecutionAdapterError::ValidationFailed { violations });
    }
    run.validate()?;
    if created_at_unix_ms == 0 {
        return Err(ExecutionAdapterError::ValidationFailed {
            violations: vec![ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_TIME_ZERO",
                "recovery plan timestamp must be non-zero",
            )],
        });
    }
    if plan.id != run.plan_id {
        return Err(ExecutionAdapterError::ValidationFailed {
            violations: vec![ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_PLAN_RUN_MISMATCH",
                "recovery plan requires matching plan and adapter run ids",
            )],
        });
    }
    if plan.scope == ExecutionScope::Live || run.scope == ExecutionScope::Live {
        return Err(ExecutionAdapterError::ValidationFailed {
            violations: vec![ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_LIVE_SCOPE_DENIED",
                "recovery plans must not use live scope",
            )],
        });
    }

    let mut steps = Vec::new();
    for intent in &plan.intents {
        let fill = run
            .fills
            .iter()
            .find(|fill| fill.intent_id == intent.id)
            .ok_or_else(|| ExecutionAdapterError::ValidationFailed {
                violations: vec![ExecutionAdapterViolation::new_owned(
                    "ADAPTER_RECOVERY_FILL_MISSING",
                    format!("missing fill for intent {}", intent.id),
                )],
            })?;
        let filled_notional_quote = fill.filled_notional_quote.min(intent.notional_quote);
        let unfilled_notional_quote = (intent.notional_quote - filled_notional_quote).max(0.0);
        append_recovery_steps_for_intent(
            &mut steps,
            intent,
            filled_notional_quote,
            unfilled_notional_quote,
        )?;
    }

    let partial_fill_count = steps
        .iter()
        .filter(|step| step.filled_notional_quote > 0.0 && step.unfilled_notional_quote > 0.0)
        .map(|step| step.intent_id.as_str())
        .collect::<HashSet<_>>()
        .len();
    let no_fill_count = steps
        .iter()
        .filter(|step| {
            step.filled_notional_quote <= f64::EPSILON
                && step.unfilled_notional_quote > f64::EPSILON
        })
        .map(|step| step.intent_id.as_str())
        .collect::<HashSet<_>>()
        .len();
    let cancel_remainder_steps = steps
        .iter()
        .filter(|step| step.action == ExecutionAdapterRecoveryAction::CancelUnfilledRemainder)
        .count();
    let hedge_exposure_steps = steps
        .iter()
        .filter(|step| step.action == ExecutionAdapterRecoveryAction::HedgeFilledExposure)
        .count();
    let operator_review_required = steps.iter().any(|step| step.operator_review_required);
    let recovery_plan = ExecutionAdapterRecoveryPlan {
        adapter_framework_version: EXECUTION_ADAPTER_FRAMEWORK_VERSION.to_owned(),
        plan_id: plan.id.clone(),
        adapter_run_id: run.id.clone(),
        created_at_unix_ms,
        steps,
        partial_fill_count,
        no_fill_count,
        cancel_remainder_steps,
        hedge_exposure_steps,
        operator_review_required,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    recovery_plan.validate()?;
    Ok(recovery_plan)
}

/// Persist the latest deterministic execution-adapter run as a non-secret checkpoint.
///
/// This helper only writes through the typed local `StateStore` boundary. It
/// does not submit adapters, place orders, call exchanges/RPCs, sign payloads,
/// broadcast transactions, withdraw funds, or bridge assets.
pub fn persist_execution_adapter_run_checkpoint(
    store: &mut impl StateStore,
    run: &ExecutionAdapterRunRecord,
) -> Result<StateCheckpoint, StateStoreError> {
    run.validate()
        .map_err(|error| StateStoreError::ValidationFailed {
            reason: error.to_string(),
        })?;
    let checkpoint = StateCheckpoint {
        key: EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY.to_owned(),
        subsystem: EXECUTION_ADAPTER_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(run).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize execution-adapter run checkpoint: {error}"),
        })?,
        updated_at_unix_ms: run.created_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Persist the latest local execution-adapter recovery plan.
pub fn persist_execution_adapter_recovery_plan_checkpoint(
    store: &mut impl StateStore,
    recovery_plan: &ExecutionAdapterRecoveryPlan,
) -> Result<StateCheckpoint, StateStoreError> {
    recovery_plan
        .validate()
        .map_err(|error| StateStoreError::ValidationFailed {
            reason: error.to_string(),
        })?;
    let checkpoint = StateCheckpoint {
        key: EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY.to_owned(),
        subsystem: EXECUTION_ADAPTER_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(recovery_plan).map_err(|error| {
            StateStoreError::BackendFailed {
                reason: format!(
                    "failed to serialize execution-adapter recovery plan checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms: recovery_plan.created_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Append a deterministic execution-adapter run to the local audit journal.
///
/// This helper records non-secret adapter lifecycle metadata only. It does not
/// submit adapters, place orders, call exchanges/RPCs, sign payloads, broadcast
/// transactions, withdraw funds, or bridge assets.
pub fn append_execution_adapter_run_audit(
    journal: &mut AppendOnlyAuditJournal,
    run: &ExecutionAdapterRunRecord,
) -> Result<AuditRecord, AuditError> {
    run.validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "AUDIT_EXECUTION_ADAPTER_RUN_INVALID",
                error.to_string(),
            )],
        })?;

    journal.append_event(
        AuditEvent::new(
            format!("execution-adapter-run:{}", run.id),
            AuditEventKind::ExecutionResult,
            EXECUTION_ADAPTER_STATE_SUBSYSTEM,
            "execution-adapter",
            "execution adapter run modeled locally without external submission",
        )
        .with_metadata("run_id", AuditValue::Text(run.id.clone()))
        .with_metadata("request_id", AuditValue::Text(run.request_id.clone()))
        .with_metadata("plan_id", AuditValue::Text(run.plan_id.clone()))
        .with_metadata("adapter_id", AuditValue::Text(run.adapter_id.clone()))
        .with_metadata(
            "adapter_framework_version",
            AuditValue::Text(run.adapter_framework_version.clone()),
        )
        .with_metadata(
            "status",
            AuditValue::Text(execution_adapter_run_status_label(run.status).to_owned()),
        )
        .with_metadata(
            "attempt_count",
            AuditValue::Unsigned(count_to_u64(run.attempts.len(), "attempt")?),
        )
        .with_metadata(
            "fill_count",
            AuditValue::Unsigned(count_to_u64(run.fills.len(), "fill")?),
        )
        .with_metadata(
            "reconciliation_count",
            AuditValue::Unsigned(count_to_u64(run.reconciliations.len(), "reconciliation")?),
        )
        .with_metadata(
            "external_submission_enabled",
            AuditValue::Bool(run.external_submission_enabled),
        )
        .with_metadata(
            "created_at_unix_ms",
            AuditValue::Unsigned(run.created_at_unix_ms),
        ),
    )
}

/// Append a local execution-adapter recovery plan to the audit journal.
pub fn append_execution_adapter_recovery_plan_audit(
    journal: &mut AppendOnlyAuditJournal,
    recovery_plan: &ExecutionAdapterRecoveryPlan,
) -> Result<AuditRecord, AuditError> {
    recovery_plan
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "AUDIT_EXECUTION_ADAPTER_RECOVERY_PLAN_INVALID",
                error.to_string(),
            )],
        })?;

    journal.append_event(
        AuditEvent::new(
            format!(
                "execution-adapter-recovery:{}",
                recovery_plan.adapter_run_id
            ),
            AuditEventKind::RuntimeLifecycle,
            EXECUTION_ADAPTER_STATE_SUBSYSTEM,
            "execution-adapter",
            "execution adapter recovery plan modeled locally without external submission",
        )
        .with_metadata("plan_id", AuditValue::Text(recovery_plan.plan_id.clone()))
        .with_metadata(
            "adapter_run_id",
            AuditValue::Text(recovery_plan.adapter_run_id.clone()),
        )
        .with_metadata(
            "partial_fill_count",
            AuditValue::Unsigned(count_to_u64(
                recovery_plan.partial_fill_count,
                "partial fill",
            )?),
        )
        .with_metadata(
            "no_fill_count",
            AuditValue::Unsigned(count_to_u64(recovery_plan.no_fill_count, "no fill")?),
        )
        .with_metadata(
            "cancel_remainder_steps",
            AuditValue::Unsigned(count_to_u64(
                recovery_plan.cancel_remainder_steps,
                "cancel remainder",
            )?),
        )
        .with_metadata(
            "hedge_exposure_steps",
            AuditValue::Unsigned(count_to_u64(
                recovery_plan.hedge_exposure_steps,
                "hedge exposure",
            )?),
        )
        .with_metadata(
            "operator_review_required",
            AuditValue::Bool(recovery_plan.operator_review_required),
        )
        .with_metadata(
            "external_submission_performed",
            AuditValue::Bool(recovery_plan.external_submission_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(recovery_plan.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(recovery_plan.production_ready),
        ),
    )
}

/// Execution-adapter trait boundary.
pub trait ExecutionAdapter {
    /// Stable adapter name for diagnostics and audit records.
    fn adapter_name(&self) -> &str;

    /// Evaluate a draft plan at the adapter boundary without external submission.
    fn evaluate_plan(
        &self,
        request: &ExecutionAdapterRequest,
        policy: &PolicyEngine,
    ) -> Result<ExecutionAdapterRunRecord, ExecutionAdapterError>;
}

/// Deterministic Phase 11 adapter boundary implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicExecutionAdapterBoundary;

impl DeterministicExecutionAdapterBoundary {
    /// Create a deterministic execution-adapter boundary.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ExecutionAdapter for DeterministicExecutionAdapterBoundary {
    fn adapter_name(&self) -> &str {
        "deterministic-phase-11-execution-adapter-boundary"
    }

    fn evaluate_plan(
        &self,
        request: &ExecutionAdapterRequest,
        policy: &PolicyEngine,
    ) -> Result<ExecutionAdapterRunRecord, ExecutionAdapterError> {
        request.validate()?;

        let mut attempts = Vec::new();
        let mut fills = Vec::new();
        let mut reconciliations = Vec::new();
        let mut warnings = Vec::new();

        for (index, intent) in request.plan.intents.iter().enumerate() {
            let sequence = sequence_for(index);
            let boundary = boundary_for_intent(request, policy, intent);
            attempts.push(ExecutionAdapterAttempt {
                sequence,
                intent_id: intent.id.clone(),
                venue: intent.venue.clone(),
                action: boundary.action,
                status: boundary.attempt_status,
                policy_revalidated: true,
                submitted_to_external_adapter: false,
                reason: boundary.reason,
            });
            fills.push(fill_record(
                intent,
                boundary.fill_status,
                request.now_unix_ms,
            ));
            reconciliations.push(reconciliation_record(
                intent,
                boundary.reconciliation_status,
                request.now_unix_ms,
            ));
        }

        if request.plan.status == ExecutionPlanStatus::PolicyDeniedDraft {
            warnings.push(
                "source plan status is policy-denied-draft; adapter boundary did not submit"
                    .to_owned(),
            );
        }

        warnings.push(
            "Phase 11 records are deterministic adapter-boundary models only; no external order, swap, signing, RPC, or broadcast occurred"
                .to_owned(),
        );

        let run = ExecutionAdapterRunRecord {
            id: format!("adapter-run:{}:{}", request.plan.id, request.id),
            request_id: request.id.clone(),
            plan_id: request.plan.id.clone(),
            adapter_framework_version: EXECUTION_ADAPTER_FRAMEWORK_VERSION.to_owned(),
            adapter_id: request.config.adapter_id.clone(),
            scope: request.plan.scope,
            status: run_status_for(&attempts, request.plan.scope),
            created_at_unix_ms: request.now_unix_ms,
            attempts,
            fills,
            reconciliations,
            external_submission_enabled: false,
            warnings,
        };
        run.validate()?;
        Ok(run)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntentBoundary {
    action: ExecutionAdapterAction,
    attempt_status: ExecutionAdapterAttemptStatus,
    fill_status: ExecutionFillStatus,
    reconciliation_status: ExecutionReconciliationStatus,
    reason: String,
}

fn boundary_for_intent(
    request: &ExecutionAdapterRequest,
    policy: &PolicyEngine,
    intent: &ExecutionIntent,
) -> IntentBoundary {
    if request.plan.status == ExecutionPlanStatus::PolicyDeniedDraft {
        return IntentBoundary {
            action: ExecutionAdapterAction::PolicyRevalidation,
            attempt_status: ExecutionAdapterAttemptStatus::PolicyDenied,
            fill_status: ExecutionFillStatus::NoFill,
            reconciliation_status: ExecutionReconciliationStatus::Blocked,
            reason: "source plan was policy-denied before adapter boundary".to_owned(),
        };
    }

    if !planner_record_approved(request, &intent.id) {
        return IntentBoundary {
            action: ExecutionAdapterAction::PolicyRevalidation,
            attempt_status: ExecutionAdapterAttemptStatus::PolicyDenied,
            fill_status: ExecutionFillStatus::NoFill,
            reconciliation_status: ExecutionReconciliationStatus::Blocked,
            reason: "source planner policy outcome was not approved".to_owned(),
        };
    }

    match policy.evaluate(intent) {
        PolicyDecision::Denied { violations } => IntentBoundary {
            action: ExecutionAdapterAction::PolicyRevalidation,
            attempt_status: ExecutionAdapterAttemptStatus::PolicyDenied,
            fill_status: ExecutionFillStatus::NoFill,
            reconciliation_status: ExecutionReconciliationStatus::Blocked,
            reason: format!(
                "adapter policy revalidation denied {} violation(s)",
                violations.len()
            ),
        },
        PolicyDecision::Approved { .. } => approved_boundary_for(request, intent),
    }
}

fn approved_boundary_for(
    request: &ExecutionAdapterRequest,
    intent: &ExecutionIntent,
) -> IntentBoundary {
    if intent.scope == ExecutionScope::Observe || intent.kind == ExecutionIntentKind::Observation {
        return IntentBoundary {
            action: ExecutionAdapterAction::RecordObservation,
            attempt_status: ExecutionAdapterAttemptStatus::ObservationRecorded,
            fill_status: ExecutionFillStatus::NoFill,
            reconciliation_status: ExecutionReconciliationStatus::NotRequired,
            reason: "observe-only intent recorded without execution".to_owned(),
        };
    }

    if intent.scope == ExecutionScope::Paper && request.config.model_paper_fills {
        return IntentBoundary {
            action: ExecutionAdapterAction::ModelPaperExecution,
            attempt_status: ExecutionAdapterAttemptStatus::PaperModelFilled,
            fill_status: ExecutionFillStatus::ModeledFilled,
            reconciliation_status: ExecutionReconciliationStatus::Reconciled,
            reason: "paper fill modeled deterministically without external adapter submission"
                .to_owned(),
        };
    }

    IntentBoundary {
        action: ExecutionAdapterAction::BlockExternalSubmission,
        attempt_status: if supported_intent_kind(intent.kind) {
            ExecutionAdapterAttemptStatus::SubmissionBlocked
        } else {
            ExecutionAdapterAttemptStatus::UnsupportedIntent
        },
        fill_status: ExecutionFillStatus::NoFill,
        reconciliation_status: ExecutionReconciliationStatus::Blocked,
        reason: "external adapter submission is disabled in Phase 11".to_owned(),
    }
}

fn append_recovery_steps_for_intent(
    steps: &mut Vec<ExecutionAdapterRecoveryStep>,
    intent: &ExecutionIntent,
    filled_notional_quote: f64,
    unfilled_notional_quote: f64,
) -> Result<(), ExecutionAdapterError> {
    if !filled_notional_quote.is_finite()
        || !unfilled_notional_quote.is_finite()
        || filled_notional_quote < 0.0
        || unfilled_notional_quote < 0.0
    {
        return Err(ExecutionAdapterError::ValidationFailed {
            violations: vec![ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_AMOUNT_INVALID",
                "recovery amounts must be finite and non-negative",
            )],
        });
    }
    if unfilled_notional_quote <= f64::EPSILON {
        steps.push(recovery_step(
            steps.len(),
            intent,
            ExecutionAdapterRecoveryAction::Noop,
            filled_notional_quote,
            0.0,
            false,
            "modeled fill fully reconciled; no local recovery action required",
        )?);
        return Ok(());
    }

    steps.push(recovery_step(
        steps.len(),
        intent,
        ExecutionAdapterRecoveryAction::CancelUnfilledRemainder,
        filled_notional_quote,
        unfilled_notional_quote,
        true,
        "unfilled remainder must be cancelled or marked cancelled before future progress",
    )?);
    if filled_notional_quote > f64::EPSILON {
        steps.push(recovery_step(
            steps.len(),
            intent,
            ExecutionAdapterRecoveryAction::HedgeFilledExposure,
            filled_notional_quote,
            unfilled_notional_quote,
            true,
            "partial modeled fill requires local hedge/exposure review before future progress",
        )?);
    }
    Ok(())
}

fn recovery_step(
    existing_steps: usize,
    intent: &ExecutionIntent,
    action: ExecutionAdapterRecoveryAction,
    filled_notional_quote: f64,
    unfilled_notional_quote: f64,
    operator_review_required: bool,
    reason: &str,
) -> Result<ExecutionAdapterRecoveryStep, ExecutionAdapterError> {
    let sequence = u16::try_from(existing_steps.saturating_add(1)).map_err(|_| {
        ExecutionAdapterError::ValidationFailed {
            violations: vec![ExecutionAdapterViolation::new(
                "ADAPTER_RECOVERY_SEQUENCE_OVERFLOW",
                "too many recovery steps",
            )],
        }
    })?;
    Ok(ExecutionAdapterRecoveryStep {
        sequence,
        intent_id: intent.id.clone(),
        action,
        expected_notional_quote: intent.notional_quote,
        filled_notional_quote,
        unfilled_notional_quote,
        operator_review_required,
        external_submission_performed: false,
        live_execution_performed: false,
        reason: reason.to_owned(),
    })
}

fn planner_record_approved(request: &ExecutionAdapterRequest, intent_id: &str) -> bool {
    request.plan.policy_outcomes.iter().any(|outcome| {
        outcome.intent_id == intent_id && outcome.status == PlannerPolicyStatus::Approved
    })
}

fn supported_intent_kind(kind: ExecutionIntentKind) -> bool {
    matches!(
        kind,
        ExecutionIntentKind::Observation
            | ExecutionIntentKind::CexOrder
            | ExecutionIntentKind::DexSwap
    )
}

fn fill_record(
    intent: &ExecutionIntent,
    status: ExecutionFillStatus,
    now_unix_ms: u64,
) -> ExecutionFillRecord {
    let filled_notional_quote = if status == ExecutionFillStatus::ModeledFilled {
        intent.notional_quote
    } else {
        0.0
    };
    let fee_quote = if status == ExecutionFillStatus::ModeledFilled {
        intent.estimated_fee_quote + intent.gas_fee_quote
    } else {
        0.0
    };
    ExecutionFillRecord {
        id: format!("fill:{}", intent.id),
        intent_id: intent.id.clone(),
        status,
        filled_notional_quote,
        fee_quote,
        filled_at_unix_ms: if status == ExecutionFillStatus::ModeledFilled {
            now_unix_ms
        } else {
            0
        },
        external_order_id: None,
    }
}

fn reconciliation_record(
    intent: &ExecutionIntent,
    status: ExecutionReconciliationStatus,
    now_unix_ms: u64,
) -> ExecutionReconciliationRecord {
    let observed_notional_quote = if status == ExecutionReconciliationStatus::Reconciled {
        intent.notional_quote
    } else {
        0.0
    };
    ExecutionReconciliationRecord {
        id: format!("reconciliation:{}", intent.id),
        intent_id: intent.id.clone(),
        status,
        expected_notional_quote: intent.notional_quote,
        observed_notional_quote,
        difference_quote: intent.notional_quote - observed_notional_quote,
        reconciled_at_unix_ms: if status == ExecutionReconciliationStatus::Reconciled {
            now_unix_ms
        } else {
            0
        },
    }
}

fn run_status_for(
    attempts: &[ExecutionAdapterAttempt],
    scope: ExecutionScope,
) -> ExecutionAdapterRunStatus {
    if attempts
        .iter()
        .any(|attempt| attempt.status == ExecutionAdapterAttemptStatus::PolicyDenied)
    {
        return ExecutionAdapterRunStatus::PolicyDenied;
    }

    if attempts.iter().any(|attempt| {
        matches!(
            attempt.status,
            ExecutionAdapterAttemptStatus::SubmissionBlocked
                | ExecutionAdapterAttemptStatus::UnsupportedIntent
        )
    }) {
        return ExecutionAdapterRunStatus::SubmissionBlocked;
    }

    if scope == ExecutionScope::Observe {
        ExecutionAdapterRunStatus::ObserveRecorded
    } else {
        ExecutionAdapterRunStatus::PaperModelComplete
    }
}

fn sequence_for(index: usize) -> u16 {
    u16::try_from(index + 1).unwrap_or(u16::MAX)
}

fn count_to_u64(count: usize, label: &str) -> Result<u64, AuditError> {
    u64::try_from(count).map_err(|_| AuditError::Serialize {
        reason: format!("execution adapter {label} count overflowed"),
    })
}

fn execution_adapter_run_status_label(status: ExecutionAdapterRunStatus) -> &'static str {
    match status {
        ExecutionAdapterRunStatus::ObserveRecorded => "observe-recorded",
        ExecutionAdapterRunStatus::PaperModelComplete => "paper-model-complete",
        ExecutionAdapterRunStatus::SubmissionBlocked => "submission-blocked",
        ExecutionAdapterRunStatus::PolicyDenied => "policy-denied",
    }
}

fn validate_id(label: &str, value: &str, violations: &mut Vec<ExecutionAdapterViolation>) {
    if value.trim().is_empty() {
        violations.push(ExecutionAdapterViolation::new_owned(
            "ADAPTER_ID_EMPTY",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn collect_planner_error(
    error: ExecutionPlannerError,
    violations: &mut Vec<ExecutionAdapterViolation>,
) {
    for violation in error.violations() {
        violations.push(ExecutionAdapterViolation::new_owned(
            "ADAPTER_PLAN_INVALID",
            format!("{}: {}", violation.code(), violation.message()),
        ));
    }
}

fn finish_validation(
    violations: Vec<ExecutionAdapterViolation>,
) -> Result<(), ExecutionAdapterError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ExecutionAdapterError::ValidationFailed { violations })
    }
}

/// One execution-adapter validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionAdapterViolation {
    code: &'static str,
    message: String,
}

impl ExecutionAdapterViolation {
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

/// Execution-adapter boundary errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionAdapterError {
    /// Validation failed with deterministic violations.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<ExecutionAdapterViolation>,
    },
}

impl ExecutionAdapterError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[ExecutionAdapterViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
        }
    }
}

impl fmt::Display for ExecutionAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "execution adapter validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ExecutionAdapterError {}

#[cfg(test)]
mod tests {
    use super::{
        append_execution_adapter_recovery_plan_audit, append_execution_adapter_run_audit,
        persist_execution_adapter_recovery_plan_checkpoint,
        persist_execution_adapter_run_checkpoint, plan_execution_adapter_recovery,
        DeterministicExecutionAdapterBoundary, ExecutionAdapter, ExecutionAdapterConfig,
        ExecutionAdapterRecoveryAction, ExecutionAdapterRecoveryPlan, ExecutionAdapterRequest,
        ExecutionAdapterRunStatus, EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY,
        EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, EXECUTION_ADAPTER_STATE_SUBSYSTEM,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, AuditEventKind, AuditValue,
        DeterministicExecutionPlanner, ExecutionPlanner, ExecutionPlannerConfig,
        ExecutionPlannerRequest, FeeAdjustedEdge, FeeEstimate, InMemoryStateStore, LiquidityRole,
        MarketPair, OpportunityCandidate, OpportunityLeg, OpportunityLegSide, OpportunityRouteKind,
        OpportunityScore, PolicyContext, PolicyEngine, SqliteWalStateStore, StateStore, VenueKind,
        VenueRef,
    };
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    const PAPER_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 1_000.0
max_daily_loss_quote = 100.0
max_open_exposure_quote = 2_000.0
slippage_bps = 100
gas_fee_cap_quote = 10.0

[venues]
cex_allowlist = ["paper-a", "paper-b"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC", "USD"]

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

    #[test]
    fn adapter_config_rejects_external_submission() {
        let config = ExecutionAdapterConfig {
            adapter_submission_enabled: true,
            ..ExecutionAdapterConfig::default()
        };

        let error = config
            .validate()
            .expect_err("external submission must be rejected");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "ADAPTER_SUBMISSION_DENIED_IN_PHASE_11" }));
    }

    #[test]
    fn adapter_models_paper_plan_without_external_submission() {
        let policy = policy();
        let plan = planner_plan(&policy);
        let request = ExecutionAdapterRequest {
            id: "adapter-request-1".to_owned(),
            plan,
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        };

        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &policy)
            .expect("adapter run should be modeled");

        assert_eq!(run.status, ExecutionAdapterRunStatus::PaperModelComplete);
        assert!(!run.external_submission_enabled);
        assert_eq!(run.attempts.len(), 2);
        assert!(run
            .attempts
            .iter()
            .all(|attempt| attempt.policy_revalidated));
        assert!(run
            .attempts
            .iter()
            .all(|attempt| !attempt.submitted_to_external_adapter));
        assert!(run
            .fills
            .iter()
            .all(|fill| fill.external_order_id.is_none()));
    }

    #[test]
    fn adapter_run_rejects_duplicate_lifecycle_identifiers() {
        let policy = policy();
        let plan = planner_plan(&policy);
        let request = ExecutionAdapterRequest {
            id: "adapter-request-duplicate-ids".to_owned(),
            plan,
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        };
        let mut run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &policy)
            .expect("adapter run should be modeled");

        run.attempts[0].policy_revalidated = false;
        run.attempts[1].sequence = run.attempts[0].sequence;
        run.attempts[1].intent_id = run.attempts[0].intent_id.clone();
        run.fills[1].id = run.fills[0].id.clone();
        run.fills[1].intent_id = run.fills[0].intent_id.clone();
        run.reconciliations[1].id = run.reconciliations[0].id.clone();
        run.reconciliations[1].intent_id = run.reconciliations[0].intent_id.clone();

        let error = run
            .validate()
            .expect_err("duplicate adapter lifecycle identifiers must be rejected");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "ADAPTER_DUPLICATE_ATTEMPT_SEQUENCE"));
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "ADAPTER_DUPLICATE_ATTEMPT_INTENT_ID"));
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "ADAPTER_ATTEMPT_POLICY_NOT_REVALIDATED" }));
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "ADAPTER_DUPLICATE_FILL_ID"));
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "ADAPTER_DUPLICATE_RECONCILIATION_ID" }));
    }

    #[test]
    fn adapter_policy_revalidation_honors_kill_switch_without_submission() {
        let planning_policy = policy();
        let plan = planner_plan(&planning_policy);
        let adapter_policy = PolicyEngine::new(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
            PolicyContext {
                kill_switch_engaged: true,
                ..PolicyContext::default()
            },
        );
        let request = ExecutionAdapterRequest {
            id: "adapter-request-kill-switch".to_owned(),
            plan,
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        };

        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &adapter_policy)
            .expect("adapter run should model policy-denied attempts");

        assert_eq!(run.status, ExecutionAdapterRunStatus::PolicyDenied);
        assert!(run
            .attempts
            .iter()
            .all(|attempt| attempt.policy_revalidated));
        assert!(run.attempts.iter().all(|attempt| {
            attempt.status == super::ExecutionAdapterAttemptStatus::PolicyDenied
        }));
        assert!(run
            .fills
            .iter()
            .all(|fill| fill.status == super::ExecutionFillStatus::NoFill));
        assert!(run.reconciliations.iter().all(|reconciliation| {
            reconciliation.status == super::ExecutionReconciliationStatus::Blocked
        }));
        assert!(!run.external_submission_enabled);
        assert!(run
            .attempts
            .iter()
            .all(|attempt| !attempt.submitted_to_external_adapter));
    }

    #[test]
    fn adapter_run_persists_as_state_checkpoint() {
        let policy = policy();
        let plan = planner_plan(&policy);
        let request = ExecutionAdapterRequest {
            id: "adapter-request-1".to_owned(),
            plan,
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        };
        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &policy)
            .expect("adapter run should be modeled");
        let mut store = InMemoryStateStore::new();

        let checkpoint = persist_execution_adapter_run_checkpoint(&mut store, &run)
            .expect("adapter run checkpoint should persist");

        assert_eq!(checkpoint.key, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY);
        assert_eq!(checkpoint.subsystem, EXECUTION_ADAPTER_STATE_SUBSYSTEM);
        assert_eq!(checkpoint.updated_at_unix_ms, run.created_at_unix_ms);
        let restored: super::ExecutionAdapterRunRecord =
            serde_json::from_str(&checkpoint.value).expect("checkpoint json should parse");
        assert_eq!(restored, run);
        assert_eq!(
            store
                .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
                .expect("checkpoint should read"),
            Some(checkpoint)
        );
    }

    #[test]
    fn adapter_run_appends_replayable_audit_record() {
        let policy = policy();
        let plan = planner_plan(&policy);
        let request = ExecutionAdapterRequest {
            id: "adapter-request-audit-1".to_owned(),
            plan,
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        };
        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &policy)
            .expect("adapter run should be modeled");
        let audit_path = temp_audit_path("execution-adapter-run");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");

        let audit_record = append_execution_adapter_run_audit(&mut journal, &run)
            .expect("adapter run audit record should append");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(audit_record.event.kind, AuditEventKind::ExecutionResult);
        assert_eq!(
            audit_record.event.subsystem,
            EXECUTION_ADAPTER_STATE_SUBSYSTEM
        );
        assert_eq!(
            audit_record.event.metadata.get("run_id"),
            Some(&AuditValue::Text(run.id.clone()))
        );
        assert_eq!(
            audit_record.event.metadata.get("status"),
            Some(&AuditValue::Text("paper-model-complete".to_owned()))
        );
        assert_eq!(
            audit_record
                .event
                .metadata
                .get("external_submission_enabled"),
            Some(&AuditValue::Bool(false))
        );
        drop(journal);

        let reopened = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened.next_sequence(), 2);

        let _ = fs::remove_file(audit_path);
    }

    #[test]
    fn adapter_recovery_plan_models_partial_cancel_and_hedge_without_submission() {
        let policy = policy();
        let plan = planner_plan(&policy);
        let request = ExecutionAdapterRequest {
            id: "adapter-request-recovery".to_owned(),
            plan: plan.clone(),
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        };
        let mut run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &policy)
            .expect("adapter run should be modeled");
        let expected_notional = plan.intents[0].notional_quote;
        let partial_notional = expected_notional / 2.0;

        run.fills[0].filled_notional_quote = partial_notional;
        run.fills[0].fee_quote /= 2.0;
        run.reconciliations[0].observed_notional_quote = partial_notional;
        run.reconciliations[0].difference_quote = expected_notional - partial_notional;

        let recovery = plan_execution_adapter_recovery(&plan, &run, 30_000)
            .expect("partial fill should produce a local recovery plan");

        assert_eq!(recovery.partial_fill_count, 1);
        assert_eq!(recovery.no_fill_count, 0);
        assert_eq!(recovery.cancel_remainder_steps, 1);
        assert_eq!(recovery.hedge_exposure_steps, 1);
        assert!(recovery.operator_review_required);
        assert!(!recovery.external_submission_performed);
        assert!(!recovery.live_execution_performed);
        assert!(!recovery.production_ready);
        assert!(recovery.steps.iter().any(|step| {
            step.action == ExecutionAdapterRecoveryAction::CancelUnfilledRemainder
        }));
        assert!(recovery
            .steps
            .iter()
            .any(|step| step.action == ExecutionAdapterRecoveryAction::HedgeFilledExposure));

        let audit_path = temp_audit_path("execution-adapter-recovery-plan");
        let state_path = temp_state_path("execution-adapter-recovery-plan");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("state store opens");

        let audit_record = append_execution_adapter_recovery_plan_audit(&mut journal, &recovery)
            .expect("recovery plan audit record should append");
        let checkpoint = persist_execution_adapter_recovery_plan_checkpoint(&mut store, &recovery)
            .expect("recovery plan checkpoint should persist");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(audit_record.event.kind, AuditEventKind::RuntimeLifecycle);
        assert_eq!(
            checkpoint.key,
            EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 2);
        drop(reopened_journal);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("state store reopens");
        let recovered_checkpoint = reopened_store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
            .expect("checkpoint read should succeed")
            .expect("checkpoint should exist");
        let restored: ExecutionAdapterRecoveryPlan =
            serde_json::from_str(&recovered_checkpoint.value)
                .expect("recovery checkpoint json should parse");
        assert_eq!(restored, recovery);
        drop(reopened_store);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn adapter_recovery_plan_models_no_fill_cancel_without_hedge() {
        let planning_policy = policy();
        let plan = planner_plan(&planning_policy);
        let adapter_policy = PolicyEngine::new(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
            PolicyContext {
                kill_switch_engaged: true,
                ..PolicyContext::default()
            },
        );
        let request = ExecutionAdapterRequest {
            id: "adapter-request-no-fill-recovery".to_owned(),
            plan: plan.clone(),
            config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_100,
        };

        let run = DeterministicExecutionAdapterBoundary::new()
            .evaluate_plan(&request, &adapter_policy)
            .expect("adapter run should model no-fill outcomes");
        let recovery = plan_execution_adapter_recovery(&plan, &run, 30_100)
            .expect("no-fill run should produce a local recovery plan");

        assert_eq!(recovery.partial_fill_count, 0);
        assert_eq!(recovery.no_fill_count, plan.intents.len());
        assert_eq!(recovery.cancel_remainder_steps, plan.intents.len());
        assert_eq!(recovery.hedge_exposure_steps, 0);
        assert!(recovery.operator_review_required);
        assert!(!recovery.external_submission_performed);
        assert!(!recovery.live_execution_performed);
        assert!(!recovery.production_ready);
        assert!(recovery.steps.iter().all(|step| {
            step.action == ExecutionAdapterRecoveryAction::CancelUnfilledRemainder
                && step.filled_notional_quote <= f64::EPSILON
                && step.unfilled_notional_quote > f64::EPSILON
        }));
    }

    fn policy() -> PolicyEngine {
        PolicyEngine::from_config(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
        )
    }

    fn planner_plan(policy: &PolicyEngine) -> crate::ExecutionPlanDraft {
        let request = ExecutionPlannerRequest {
            id: "planner-request-1".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };
        DeterministicExecutionPlanner::new()
            .plan(&request, policy)
            .expect("planner should create a draft")
    }

    fn candidate() -> OpportunityCandidate {
        let pair = MarketPair::new("BTC", "USD").expect("pair should validate");
        let edge = FeeAdjustedEdge::calculate(15.0, 2.0, 100.0).expect("edge should validate");
        OpportunityCandidate {
            id: "opp-cex-cex-btc-usd".to_owned(),
            route_kind: OpportunityRouteKind::CexCex,
            pair: pair.clone(),
            legs: vec![
                leg("paper-a", pair.clone(), OpportunityLegSide::Buy, 100.0, 1.0),
                leg("paper-b", pair, OpportunityLegSide::Sell, 115.0, 1.0),
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
            discovered_at_unix_ms: 9_900,
            source_quote_ids: vec!["quote-a".to_owned(), "quote-b".to_owned()],
            warnings: Vec::new(),
        }
    }

    fn leg(
        venue_name: &str,
        pair: MarketPair,
        side: OpportunityLegSide,
        price_quote: f64,
        quantity_base: f64,
    ) -> OpportunityLeg {
        let notional_quote = price_quote * quantity_base;
        OpportunityLeg {
            venue: VenueRef {
                name: venue_name.to_owned(),
                kind: VenueKind::Cex,
            },
            pair: pair.clone(),
            side,
            price_quote,
            quantity_base,
            notional_quote,
            fee_estimate: FeeEstimate {
                venue: VenueRef {
                    name: venue_name.to_owned(),
                    kind: VenueKind::Cex,
                },
                pair: Some(pair),
                notional_quote,
                liquidity_role: LiquidityRole::Taker,
                fee_bps: 10.0,
                venue_fee_quote: 1.0,
                network_fee_quote: 0.0,
                total_fee_quote: 1.0,
                externally_verified: true,
            },
            source_quote_id: format!("quote-{venue_name}"),
            market_data_age_ms: 100,
        }
    }

    fn temp_audit_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!(
            "arbyclaw-{label}-{}-{nanos}-{counter}.jsonl",
            std::process::id()
        ));
        path
    }

    fn temp_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!(
            "arbyclaw-{label}-{}-{nanos}-{counter}.sqlite",
            std::process::id()
        ));
        path
    }

    fn cleanup_state_files(path: &PathBuf) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite-shm"));
    }
}
