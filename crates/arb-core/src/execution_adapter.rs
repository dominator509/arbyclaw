#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    ExecutionIntent, ExecutionIntentKind, ExecutionPlanDraft, ExecutionPlanStatus,
    ExecutionPlannerError, ExecutionScope, PlannerPolicyStatus, PolicyDecision, PolicyEngine,
    StateCheckpoint, StateStore, StateStoreError, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable execution-adapter framework version for audit, replay, and handoff surfaces.
pub const EXECUTION_ADAPTER_FRAMEWORK_VERSION: &str = "phase-11-execution-adapter-framework-v1";

/// State-store subsystem name for execution-adapter checkpoints.
pub const EXECUTION_ADAPTER_STATE_SUBSYSTEM: &str = "execution-adapter";

/// State-store key for the latest deterministic execution-adapter run.
pub const EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY: &str = "execution-adapter:last-run";

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

        for attempt in &self.attempts {
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

        for fill in &self.fills {
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

        for reconciliation in &self.reconciliations {
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
        persist_execution_adapter_run_checkpoint, DeterministicExecutionAdapterBoundary,
        ExecutionAdapter, ExecutionAdapterConfig, ExecutionAdapterRequest,
        ExecutionAdapterRunStatus, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY,
        EXECUTION_ADAPTER_STATE_SUBSYSTEM,
    };
    use crate::{
        AgentConfig, DeterministicExecutionPlanner, ExecutionPlanner, ExecutionPlannerConfig,
        ExecutionPlannerRequest, FeeAdjustedEdge, FeeEstimate, InMemoryStateStore, LiquidityRole,
        MarketPair, OpportunityCandidate, OpportunityLeg, OpportunityLegSide, OpportunityRouteKind,
        OpportunityScore, PolicyEngine, StateStore, VenueKind, VenueRef,
    };

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
            .all(|attempt| !attempt.submitted_to_external_adapter));
        assert!(run
            .fills
            .iter()
            .all(|fill| fill.external_order_id.is_none()));
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
}
