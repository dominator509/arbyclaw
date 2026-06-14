#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    append_policy_decision_audit, AppendOnlyAuditJournal, AuditError, AuditEvent, AuditEventKind,
    AuditRecord, AuditValue, DestinationPolicy, DeterministicOpportunityEngine, ExecutionIntent,
    ExecutionIntentKind, ExecutionScope, OpportunityCandidate, OpportunityEngine, OpportunityError,
    OpportunityHistoricalFixtureCorpus, OpportunityLeg, OpportunityLegSide, OpportunityRouteKind,
    PolicyDecision, PolicyDecisionRecord, PolicyEngine, SqliteWalStateStore, StateCheckpoint,
    StateStore, StateStoreError, StrategyPolicyConstraintReport, StrategyPolicyConstraintStatus,
    StrategyProfile, VenueKind, DEFAULT_MAX_MARKET_DATA_AGE_MS, TRUST_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error, fmt, path::Path};

/// Stable planner version for audit, replay, and handoff surfaces.
pub const EXECUTION_PLANNER_VERSION: &str = "phase-10-execution-planner-v1";

/// State-store subsystem name for execution-planner checkpoints.
pub const EXECUTION_PLANNER_STATE_SUBSYSTEM: &str = "execution-planner";

/// State-store key for the latest deterministic execution-plan draft.
pub const EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY: &str = "execution-planner:last-draft";

/// State checkpoint key prefix for local opportunity candidate trace records.
pub const OPPORTUNITY_CANDIDATE_TRACE_CHECKPOINT_KEY_PREFIX: &str = "opportunity-candidate-trace";

/// State-store subsystem name for local opportunity candidate trace checkpoints.
pub const OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM: &str = "opportunity-candidate-trace";

/// Conservative execution-planner settings.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlannerConfig {
    /// Requested planning scope. Phase 10 rejects live scope fail-closed.
    pub requested_scope: ExecutionScope,
    /// Maximum number of modeled opportunity legs accepted for one plan.
    pub max_plan_legs: usize,
    /// Maximum total notional across all modeled legs.
    pub max_total_notional_quote: f64,
    /// Slippage limit copied onto each generated draft intent.
    pub default_slippage_bps: u16,
    /// Maximum accepted market-data age for source opportunity legs.
    pub max_market_data_age_ms: u64,
    /// Whether policy denials should mark the plan as denied instead of ready.
    pub require_policy_preflight: bool,
}

impl Default for ExecutionPlannerConfig {
    fn default() -> Self {
        Self {
            requested_scope: ExecutionScope::Paper,
            max_plan_legs: 4,
            max_total_notional_quote: 100_000.0,
            default_slippage_bps: 50,
            max_market_data_age_ms: DEFAULT_MAX_MARKET_DATA_AGE_MS,
            require_policy_preflight: true,
        }
    }
}

impl ExecutionPlannerConfig {
    /// Validate planner settings before a candidate is converted into intents.
    pub fn validate(&self) -> Result<(), ExecutionPlannerError> {
        let mut violations = Vec::new();

        if self.requested_scope == ExecutionScope::Live {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_LIVE_SCOPE_DENIED",
                "Phase 10 planner rejects live scope; only observe or paper draft planning is allowed",
            ));
        }

        if self.max_plan_legs == 0 {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_MAX_LEGS_ZERO",
                "max_plan_legs must be positive",
            ));
        }

        if !is_positive_finite(self.max_total_notional_quote) {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_MAX_NOTIONAL_INVALID",
                "max_total_notional_quote must be positive and finite",
            ));
        }

        if self.max_market_data_age_ms == 0 {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_MAX_MARKET_DATA_AGE_ZERO",
                "max_market_data_age_ms must be positive",
            ));
        }

        finish_validation(violations)
    }
}

/// One deterministic planning request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlannerRequest {
    /// Stable request id for audit and deterministic replay.
    pub id: String,
    /// Strategy profile id that selected the opportunity.
    pub strategy_id: String,
    /// Validated opportunity candidate to convert into a draft plan.
    pub candidate: OpportunityCandidate,
    /// Conservative planner settings.
    pub config: ExecutionPlannerConfig,
    /// Optional chain reference for DEX or aggregator legs.
    pub default_chain: Option<String>,
    /// Runtime clock in Unix milliseconds used for plan records.
    pub now_unix_ms: u64,
}

impl ExecutionPlannerRequest {
    /// Validate a planner request before producing a draft plan.
    pub fn validate(&self) -> Result<(), ExecutionPlannerError> {
        let mut violations = Vec::new();
        validate_id("planner request", &self.id, &mut violations);
        validate_id("strategy", &self.strategy_id, &mut violations);

        if let Err(ExecutionPlannerError::ValidationFailed {
            violations: config_violations,
        }) = self.config.validate()
        {
            violations.extend(config_violations);
        }

        if let Err(error) = self.candidate.validate() {
            collect_opportunity_error(error, &mut violations);
        }

        if self.candidate.legs.len() > self.config.max_plan_legs {
            violations.push(ExecutionPlannerViolation::new_owned(
                "PLANNER_TOO_MANY_LEGS",
                format!(
                    "candidate has {} legs but planner max_plan_legs is {}",
                    self.candidate.legs.len(),
                    self.config.max_plan_legs
                ),
            ));
        }

        let total_notional_quote = total_leg_notional_quote(&self.candidate.legs);
        if total_notional_quote > self.config.max_total_notional_quote {
            violations.push(ExecutionPlannerViolation::new_owned(
                "PLANNER_TOTAL_NOTIONAL_EXCEEDED",
                format!(
                    "candidate total leg notional {total_notional_quote} exceeds planner cap {}",
                    self.config.max_total_notional_quote
                ),
            ));
        }

        for leg in &self.candidate.legs {
            if leg.market_data_age_ms > self.config.max_market_data_age_ms {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLANNER_LEG_MARKET_DATA_STALE",
                    format!(
                        "leg source quote {} age {} ms exceeds planner max {} ms",
                        leg.source_quote_id,
                        leg.market_data_age_ms,
                        self.config.max_market_data_age_ms
                    ),
                ));
            }
        }

        if let Some(chain) = &self.default_chain {
            if chain.trim().is_empty() {
                violations.push(ExecutionPlannerViolation::new(
                    "PLANNER_DEFAULT_CHAIN_EMPTY",
                    "default_chain cannot be empty when supplied",
                ));
            }
        }

        if self.now_unix_ms == 0 {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_TIME_ZERO",
                "now_unix_ms must be non-zero",
            ));
        }

        finish_validation(violations)
    }
}

/// Draft-only plan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionPlanStatus {
    /// All requested policy preflight checks approved, but no adapter submission is enabled.
    DraftReady,
    /// At least one policy preflight check denied the plan.
    PolicyDeniedDraft,
}

/// Sequenced draft step action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionPlanStepAction {
    /// Run policy before any future adapter receives an intent.
    PolicyPreflight,
    /// Record an observe-only model step.
    RecordObservation,
    /// Prepare a centralized-exchange order draft only.
    PrepareCexOrderDraft,
    /// Prepare a decentralized-exchange swap draft only.
    PrepareDexSwapDraft,
    /// Boundary requiring fill confirmation before a later leg could proceed.
    AwaitFillBoundary,
    /// Boundary that prevents adapter submission in Phase 10.
    AdapterSubmissionBlocked,
}

/// Deterministic failure behavior modeled by the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionPlanFailureMode {
    /// Abort before any adapter submission if preflight validation fails.
    AbortBeforeAdapterSubmission,
    /// Require reconciliation before another leg can proceed.
    ReconcileBeforeNextLeg,
    /// Cancel any future unfilled paper/live remainder before continuing.
    CancelUnfilledRemainder,
    /// Do not sign or broadcast Web3 transactions in this phase.
    DoNotSignOrBroadcast,
    /// Escalate to a human/operator review boundary.
    ManualReviewRequired,
}

/// One sequenced draft-plan step.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlanStep {
    /// One-based sequence number.
    pub sequence: u16,
    /// Draft action to perform or preserve as a boundary.
    pub action: ExecutionPlanStepAction,
    /// Intent id referenced by this step, when applicable.
    pub intent_id: Option<String>,
    /// Step dependency sequence ids.
    pub depends_on: Vec<u16>,
    /// Failure behavior for this step.
    pub failure_mode: ExecutionPlanFailureMode,
    /// Non-secret human-readable step description.
    pub description: String,
}

/// Serializable policy outcome captured by the draft planner.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerPolicyOutcome {
    /// Intent id that was evaluated.
    pub intent_id: String,
    /// Whether policy approved or denied the draft intent.
    pub status: PlannerPolicyStatus,
    /// Trust-contract version for approvals.
    pub trust_contract_version: Option<String>,
    /// Approved scope for approvals.
    pub approved_scope: Option<ExecutionScope>,
    /// Denial violations copied without secrets.
    pub violations: Vec<PlannerPolicyViolation>,
}

impl PlannerPolicyOutcome {
    fn from_decision(intent_id: &str, decision: PolicyDecision) -> Self {
        match decision {
            PolicyDecision::Approved { approval } => Self {
                intent_id: intent_id.to_owned(),
                status: PlannerPolicyStatus::Approved,
                trust_contract_version: Some(approval.trust_contract_version.to_owned()),
                approved_scope: Some(approval.approved_scope),
                violations: Vec::new(),
            },
            PolicyDecision::Denied { violations } => Self {
                intent_id: intent_id.to_owned(),
                status: PlannerPolicyStatus::Denied,
                trust_contract_version: None,
                approved_scope: None,
                violations: violations
                    .into_iter()
                    .map(|violation| PlannerPolicyViolation {
                        code: violation.code().to_owned(),
                        message: violation.message().to_owned(),
                    })
                    .collect(),
            },
        }
    }

    /// Return true when policy approved this draft intent.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        matches!(self.status, PlannerPolicyStatus::Approved)
    }
}

/// Policy preflight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlannerPolicyStatus {
    /// Policy approved the draft intent.
    Approved,
    /// Policy denied the draft intent.
    Denied,
}

/// One redacted policy violation copied into a plan record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerPolicyViolation {
    /// Stable policy violation code.
    pub code: String,
    /// Non-secret violation message.
    pub message: String,
}

/// Draft-only execution plan. This record never submits to adapters by itself.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlanDraft {
    /// Stable deterministic plan id.
    pub id: String,
    /// Source planner request id.
    pub request_id: String,
    /// Source opportunity candidate id.
    pub candidate_id: String,
    /// Planner version that generated the draft.
    pub planner_version: String,
    /// Draft status after policy preflight.
    pub status: ExecutionPlanStatus,
    /// Scope applied to generated intents.
    pub scope: ExecutionScope,
    /// Generated execution intents; these are not submitted in Phase 10.
    pub intents: Vec<ExecutionIntent>,
    /// Sequenced draft steps and safety boundaries.
    pub steps: Vec<ExecutionPlanStep>,
    /// Captured policy outcomes for every generated intent.
    pub policy_outcomes: Vec<PlannerPolicyOutcome>,
    /// Route-level failure-mode boundaries.
    pub failure_modes: Vec<ExecutionPlanFailureMode>,
    /// Total modeled leg notional.
    pub total_notional_quote: f64,
    /// Source opportunity expected net profit.
    pub expected_net_profit_quote: f64,
    /// Creation time in Unix milliseconds.
    pub created_at_unix_ms: u64,
    /// Always false in Phase 10; future adapters must not infer execution permission from this draft.
    pub adapter_submission_enabled: bool,
    /// Non-secret warnings for operators and future audit records.
    pub warnings: Vec<String>,
}

/// Strategy-constrained planner output.
///
/// This report composes the draft-only planner with the typed local strategy
/// profile boundary. It does not submit adapters, place orders, sign,
/// broadcast, withdraw, bridge, call exchanges/RPCs, or claim readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyConstrainedExecutionPlanDraft {
    /// Draft plan produced by the deterministic planner.
    pub draft: ExecutionPlanDraft,
    /// One local strategy constraint report per generated intent.
    pub strategy_constraint_reports: Vec<StrategyPolicyConstraintReport>,
    /// Number of generated intents rejected by strategy constraints.
    pub strategy_rejected_intents: usize,
    /// Whether any adapter submission occurred. Always false in this boundary.
    pub adapter_submission_performed: bool,
    /// Whether any live execution occurred. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Whether signing or broadcasting occurred. Always false in this boundary.
    pub signing_or_broadcast_performed: bool,
    /// Production readiness is never claimed by this local boundary.
    pub production_ready: bool,
}

impl StrategyConstrainedExecutionPlanDraft {
    /// Validate local strategy-constrained planner invariants.
    pub fn validate(&self) -> Result<(), ExecutionPlannerError> {
        let mut violations = Vec::new();

        if let Err(ExecutionPlannerError::ValidationFailed {
            violations: draft_violations,
        }) = self.draft.validate()
        {
            violations.extend(draft_violations);
        }

        if self.strategy_constraint_reports.len() != self.draft.intents.len() {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_STRATEGY_REPORT_COUNT_MISMATCH",
                "strategy-constrained plans must include one constraint report per intent",
            ));
        }

        let rejected = self
            .strategy_constraint_reports
            .iter()
            .filter(|report| report.status == StrategyPolicyConstraintStatus::Rejected)
            .count();
        if rejected != self.strategy_rejected_intents {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_STRATEGY_REJECTED_COUNT_MISMATCH",
                "strategy rejected intent count must match rejected reports",
            ));
        }

        if self.strategy_rejected_intents > 0
            && self.draft.status != ExecutionPlanStatus::PolicyDeniedDraft
        {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_STRATEGY_REJECTION_NOT_FAIL_CLOSED",
                "strategy rejections must leave the draft in a denied state",
            ));
        }

        if self.adapter_submission_performed
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(ExecutionPlannerViolation::new(
                "PLANNER_STRATEGY_SIDE_EFFECT_RECORDED",
                "strategy-constrained planner reports must not record side effects or readiness",
            ));
        }

        for report in &self.strategy_constraint_reports {
            if report.execution_performed
                || report.signing_or_broadcast_performed
                || report.live_network_used
            {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLANNER_STRATEGY_REPORT_SIDE_EFFECT",
                    format!(
                        "strategy constraint report for intent {} recorded a side effect",
                        report.intent_id
                    ),
                ));
            }
        }

        finish_validation(violations)
    }
}

impl ExecutionPlanDraft {
    /// Validate draft-only invariants.
    pub fn validate(&self) -> Result<(), ExecutionPlannerError> {
        let mut violations = Vec::new();
        validate_id("plan", &self.id, &mut violations);
        validate_id("planner request", &self.request_id, &mut violations);
        validate_id("candidate", &self.candidate_id, &mut violations);

        if self.planner_version != EXECUTION_PLANNER_VERSION {
            violations.push(ExecutionPlannerViolation::new_owned(
                "PLANNER_VERSION_MISMATCH",
                format!(
                    "planner_version must be {EXECUTION_PLANNER_VERSION}, got {}",
                    self.planner_version
                ),
            ));
        }

        if self.scope == ExecutionScope::Live {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_LIVE_SCOPE_DENIED",
                "Phase 10 plans must not contain live scope",
            ));
        }

        if self.adapter_submission_enabled {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_ADAPTER_SUBMISSION_ENABLED",
                "Phase 10 plan drafts must never enable adapter submission",
            ));
        }

        if self.intents.is_empty() {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_INTENTS_EMPTY",
                "plan must contain at least one generated draft intent",
            ));
        }

        if self.steps.is_empty() {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_STEPS_EMPTY",
                "plan must contain sequencing steps",
            ));
        }

        if self.policy_outcomes.len() != self.intents.len() {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_POLICY_OUTCOME_COUNT_MISMATCH",
                "plan must contain exactly one policy outcome per generated intent",
            ));
        }

        if !is_positive_finite(self.total_notional_quote) {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_TOTAL_NOTIONAL_INVALID",
                "total_notional_quote must be positive and finite",
            ));
        }

        if !self.expected_net_profit_quote.is_finite() {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_EXPECTED_PROFIT_INVALID",
                "expected_net_profit_quote must be finite",
            ));
        }

        if self.created_at_unix_ms == 0 {
            violations.push(ExecutionPlannerViolation::new(
                "PLAN_CREATED_TIME_ZERO",
                "created_at_unix_ms must be non-zero",
            ));
        }

        let mut intent_ids = HashSet::new();
        for intent in &self.intents {
            if !intent_ids.insert(intent.id.as_str()) {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLAN_DUPLICATE_INTENT_ID",
                    format!("plan contains duplicate intent id {}", intent.id),
                ));
            }
            if intent.scope != self.scope {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLAN_INTENT_SCOPE_MISMATCH",
                    format!("intent {} scope does not match plan scope", intent.id),
                ));
            }
            if intent.scope == ExecutionScope::Live {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLAN_INTENT_LIVE_SCOPE_DENIED",
                    format!("intent {} uses live scope", intent.id),
                ));
            }
        }

        let mut policy_outcome_intent_ids = HashSet::new();
        for outcome in &self.policy_outcomes {
            if !policy_outcome_intent_ids.insert(outcome.intent_id.as_str()) {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLAN_DUPLICATE_POLICY_OUTCOME_INTENT_ID",
                    format!(
                        "plan contains duplicate policy outcome for intent {}",
                        outcome.intent_id
                    ),
                ));
            }
            if !intent_ids.contains(outcome.intent_id.as_str()) {
                violations.push(ExecutionPlannerViolation::new_owned(
                    "PLAN_POLICY_OUTCOME_UNKNOWN_INTENT",
                    format!(
                        "policy outcome references unknown intent {}",
                        outcome.intent_id
                    ),
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Persist the latest deterministic execution-plan draft as a non-secret checkpoint.
///
/// This helper only writes through the typed local `StateStore` boundary. It
/// does not submit adapters, place orders, call exchanges/RPCs, sign payloads,
/// broadcast transactions, withdraw funds, or bridge assets.
pub fn persist_execution_plan_draft_checkpoint(
    store: &mut impl StateStore,
    draft: &ExecutionPlanDraft,
) -> Result<StateCheckpoint, StateStoreError> {
    draft
        .validate()
        .map_err(|error| StateStoreError::ValidationFailed {
            reason: error.to_string(),
        })?;
    let checkpoint = StateCheckpoint {
        key: EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY.to_owned(),
        subsystem: EXECUTION_PLANNER_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(draft).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize execution-plan draft checkpoint: {error}"),
        })?,
        updated_at_unix_ms: draft.created_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Local audit outcome for one execution-plan draft and its policy outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlanAuditReport {
    /// Stable plan id that was journaled.
    pub plan_id: String,
    /// Audit sequence for the draft-level planning event.
    pub draft_record_sequence: u64,
    /// Audit sequences for per-intent policy outcome events.
    pub policy_outcome_record_sequences: Vec<u64>,
    /// Total audit records appended for this plan.
    pub records_appended: usize,
    /// Whether external submission occurred. Always false for this boundary.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Always false for this boundary.
    pub live_execution_performed: bool,
    /// Operator-supplied audit timestamp.
    pub audited_at_unix_ms: u64,
}

/// Append one local execution-plan draft and each policy outcome to the audit journal.
///
/// This journals only deterministic draft-planning metadata and redacted policy
/// outcomes. It never submits to adapters, places orders, calls exchanges/RPCs,
/// signs payloads, broadcasts transactions, withdraws funds, or bridges assets.
pub fn append_execution_plan_draft_audit(
    journal: &mut AppendOnlyAuditJournal,
    draft: &ExecutionPlanDraft,
    audited_at_unix_ms: u64,
) -> Result<ExecutionPlanAuditReport, AuditError> {
    draft
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "EXECUTION_PLAN_DRAFT_INVALID",
                error.to_string(),
            )],
        })?;
    if audited_at_unix_ms == 0 {
        return Err(AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "EXECUTION_PLAN_AUDIT_TIMESTAMP_ZERO",
                "execution plan audit timestamp is required".to_owned(),
            )],
        });
    }

    let draft_record = journal.append_event(
        AuditEvent::new(
            format!("execution-plan-draft-{}", draft.id),
            AuditEventKind::ExecutionPlanning,
            EXECUTION_PLANNER_STATE_SUBSYSTEM,
            "execution-planner",
            "execution plan draft recorded before adapter handoff",
        )
        .with_metadata("plan_id", AuditValue::Text(draft.id.clone()))
        .with_metadata("request_id", AuditValue::Text(draft.request_id.clone()))
        .with_metadata("candidate_id", AuditValue::Text(draft.candidate_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", draft.status)))
        .with_metadata("scope", AuditValue::Text(format!("{:?}", draft.scope)))
        .with_metadata(
            "intent_count",
            AuditValue::Unsigned(draft.intents.len() as u64),
        )
        .with_metadata("step_count", AuditValue::Unsigned(draft.steps.len() as u64))
        .with_metadata(
            "policy_outcome_count",
            AuditValue::Unsigned(draft.policy_outcomes.len() as u64),
        )
        .with_metadata(
            "failure_mode_count",
            AuditValue::Unsigned(draft.failure_modes.len() as u64),
        )
        .with_metadata(
            "total_notional_quote",
            AuditValue::Text(draft.total_notional_quote.to_string()),
        )
        .with_metadata(
            "expected_net_profit_quote",
            AuditValue::Text(draft.expected_net_profit_quote.to_string()),
        )
        .with_metadata(
            "adapter_submission_enabled",
            AuditValue::Bool(draft.adapter_submission_enabled),
        )
        .with_metadata("external_submission_performed", AuditValue::Bool(false))
        .with_metadata("live_execution_performed", AuditValue::Bool(false)),
    )?;

    let mut policy_outcome_record_sequences = Vec::with_capacity(draft.policy_outcomes.len());
    for outcome in &draft.policy_outcomes {
        let intent = draft
            .intents
            .iter()
            .find(|intent| intent.id == outcome.intent_id)
            .ok_or_else(|| AuditError::ValidationFailed {
                violations: vec![crate::AuditViolation::new_owned(
                    "EXECUTION_PLAN_POLICY_OUTCOME_UNKNOWN_INTENT",
                    format!(
                        "execution plan audit could not find intent for policy outcome {}",
                        outcome.intent_id
                    ),
                )],
            })?;
        let violation_codes = outcome
            .violations
            .iter()
            .map(|violation| violation.code.clone())
            .collect::<Vec<_>>();
        let decision_record = PolicyDecisionRecord {
            trust_contract_version: outcome
                .trust_contract_version
                .clone()
                .unwrap_or_else(|| TRUST_CONTRACT_VERSION.to_owned()),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            intent_kind: intent.kind,
            requested_scope: intent.scope,
            venue: intent.venue.clone(),
            approved: outcome.is_approved(),
            approved_scope: outcome.approved_scope,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            live_scope_requested: intent.scope == ExecutionScope::Live,
            funds_movement_requested: intent.kind.requires_funds_movement(),
            signing_requested: intent.requires_signing,
            external_submission_performed: false,
            secret_material_recorded: false,
            recorded_at_unix_ms: audited_at_unix_ms,
        };
        let record = append_policy_decision_audit(journal, &decision_record)?;
        policy_outcome_record_sequences.push(record.sequence);
    }

    Ok(ExecutionPlanAuditReport {
        plan_id: draft.id.clone(),
        draft_record_sequence: draft_record.sequence,
        records_appended: 1 + policy_outcome_record_sequences.len(),
        policy_outcome_record_sequences,
        external_submission_performed: false,
        live_execution_performed: false,
        audited_at_unix_ms,
    })
}

/// Local audit/state trace for one discovered opportunity candidate before planner handoff.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityCandidateTraceRecord {
    /// Stable trace id.
    pub id: String,
    /// Strategy that will receive the planner draft request.
    pub strategy_id: String,
    /// Planner request id associated with this candidate handoff.
    pub planner_request_id: String,
    /// Candidate captured before planner handoff.
    pub candidate: OpportunityCandidate,
    /// Local audit journal sequence for the trace event.
    pub audit_sequence: u64,
    /// Local audit journal hash for replay verification.
    pub audit_record_hash: String,
    /// Trace timestamp in Unix milliseconds.
    pub traced_at_unix_ms: u64,
    /// Always false; this trace never submits to adapters.
    pub adapter_submission_enabled: bool,
    /// Always false; this trace consumes supplied local records only.
    pub external_calls_performed: bool,
    /// Always false; this trace never submits orders or moves funds.
    pub live_execution_performed: bool,
}

/// Non-secret summary of one recovered opportunity candidate trace checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveredOpportunityTraceSummary {
    /// Stable trace checkpoint id.
    pub trace_id: String,
    /// Strategy that received the planner draft request.
    pub strategy_id: String,
    /// Planner request id associated with this candidate handoff.
    pub planner_request_id: String,
    /// Local audit journal sequence for the recovered trace event.
    pub audit_sequence: u64,
    /// Trace timestamp in Unix milliseconds.
    pub traced_at_unix_ms: u64,
    /// Route kind for the recovered candidate.
    pub route_kind: OpportunityRouteKind,
    /// Number of candidate legs summarized without embedding the full candidate.
    pub leg_count: u64,
}

/// Local persistence outcome for an opportunity candidate trace.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityCandidateTracePersistence {
    /// Trace record persisted into state.
    pub trace: OpportunityCandidateTraceRecord,
    /// Audit record appended before state checkpoint persistence.
    pub audit_record: AuditRecord,
    /// State checkpoint containing the trace record.
    pub checkpoint: StateCheckpoint,
}

/// Opportunity candidate trace persistence failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpportunityCandidateTraceError {
    /// Trace input validation failed.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<ExecutionPlannerViolation>,
    },
    /// Audit append failed before state persistence.
    AuditJournalFailed {
        /// Sanitized reason.
        reason: String,
    },
    /// State checkpoint persistence failed after audit append.
    StateStoreFailed {
        /// Sanitized reason.
        reason: String,
    },
}

/// Persist a local opportunity candidate trace to audit and state before planner handoff.
///
/// This writes only caller-supplied local candidate metadata. It never submits
/// adapters, places orders, calls exchanges/RPCs, signs payloads, broadcasts
/// transactions, withdraws funds, bridges assets, or stores secrets.
pub fn persist_opportunity_candidate_trace(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    candidate: &OpportunityCandidate,
    strategy_id: &str,
    planner_request_id: &str,
    occurred_at_unix_ms: u64,
) -> Result<OpportunityCandidateTracePersistence, OpportunityCandidateTraceError> {
    validate_candidate_trace_input(
        candidate,
        strategy_id,
        planner_request_id,
        occurred_at_unix_ms,
    )?;

    let mut event = AuditEvent::new(
        format!("opportunity-candidate-trace-{}", candidate.id),
        AuditEventKind::IntentLifecycle,
        OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM,
        "opportunity-planner-handoff",
        "opportunity candidate traced before draft planner handoff",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata("candidate_id", AuditValue::Text(candidate.id.clone()))
        .with_metadata("strategy_id", AuditValue::Text(strategy_id.to_owned()))
        .with_metadata(
            "planner_request_id",
            AuditValue::Text(planner_request_id.to_owned()),
        )
        .with_metadata(
            "route_kind",
            AuditValue::Text(format!("{:?}", candidate.route_kind)),
        )
        .with_metadata(
            "leg_count",
            AuditValue::Unsigned(candidate.legs.len() as u64),
        )
        .with_metadata("adapter_submission_enabled", AuditValue::Bool(false))
        .with_metadata("external_calls_performed", AuditValue::Bool(false))
        .with_metadata("live_execution_performed", AuditValue::Bool(false));

    let audit_record = journal.append_event(event).map_err(|error| {
        OpportunityCandidateTraceError::AuditJournalFailed {
            reason: error.to_string(),
        }
    })?;

    let trace = OpportunityCandidateTraceRecord {
        id: opportunity_candidate_trace_key(candidate, planner_request_id),
        strategy_id: strategy_id.to_owned(),
        planner_request_id: planner_request_id.to_owned(),
        candidate: candidate.clone(),
        audit_sequence: audit_record.sequence,
        audit_record_hash: audit_record.record_hash.clone(),
        traced_at_unix_ms: occurred_at_unix_ms,
        adapter_submission_enabled: false,
        external_calls_performed: false,
        live_execution_performed: false,
    };

    let checkpoint = StateCheckpoint {
        key: trace.id.clone(),
        subsystem: OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(&trace).map_err(|error| {
            OpportunityCandidateTraceError::StateStoreFailed {
                reason: format!("failed to serialize opportunity candidate trace: {error}"),
            }
        })?,
        updated_at_unix_ms: occurred_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone()).map_err(|error| {
        OpportunityCandidateTraceError::StateStoreFailed {
            reason: error.to_string(),
        }
    })?;

    Ok(OpportunityCandidateTracePersistence {
        trace,
        audit_record,
        checkpoint,
    })
}

/// Planner handoff validation status for local opportunity replay candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpportunityPlannerHandoffStatus {
    /// All discoverable local replay candidates produced draft-only plans.
    Passed,
    /// One or more discoverable local replay candidates failed planner handoff.
    Failed,
}

/// Local opportunity-to-planner handoff validation report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityPlannerHandoffValidationReport {
    /// Source historical fixture corpus id.
    pub corpus_id: String,
    /// Number of replay windows inspected.
    pub replay_window_count: usize,
    /// Number of replay scenarios inspected.
    pub replay_scenario_count: usize,
    /// Number of discovery validation failures skipped as fail-closed replay windows.
    pub skipped_discovery_failures: usize,
    /// Number of opportunity candidates discovered and offered to the planner.
    pub discovered_candidates: usize,
    /// Number of candidates that produced draft-only plans.
    pub planned_candidates: usize,
    /// Number of draft plans whose policy preflight was ready.
    pub draft_ready_plans: usize,
    /// Number of draft plans with policy-denied metadata.
    pub policy_denied_plans: usize,
    /// Number of planner handoff failures.
    pub failed_planner_handoffs: usize,
    /// Number of local candidate trace audit records appended before planning.
    pub candidate_trace_audit_records: usize,
    /// Number of local candidate trace checkpoints persisted before planning.
    pub candidate_trace_checkpoints: usize,
    /// Total draft intents emitted across generated plans.
    pub total_intents: usize,
    /// True only if a bug enabled adapter submission.
    pub adapter_submission_enabled: bool,
    /// Always false; validation consumes supplied local records only.
    pub external_calls_performed: bool,
    /// Always false; validation never submits orders or moves funds.
    pub live_execution_performed: bool,
    /// Overall handoff validation status.
    pub status: OpportunityPlannerHandoffStatus,
}

/// Local restart/reopen recovery report for opportunity candidate traces.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityCandidateTraceRecoveryReport {
    /// Source historical fixture corpus id.
    pub corpus_id: String,
    /// Planner handoff report produced before reopening audit/state stores.
    pub handoff_report: OpportunityPlannerHandoffValidationReport,
    /// Number of audit records replayed after reopening the local journal.
    pub audit_replay_records: usize,
    /// Number of expected trace checkpoints recovered after reopening SQLite WAL state.
    pub recovered_trace_checkpoints: usize,
    /// Non-secret summaries for recovered trace checkpoints after reopen.
    pub recovered_trace_summaries: Vec<RecoveredOpportunityTraceSummary>,
    /// Expected trace checkpoint ids missing after reopen.
    pub missing_trace_checkpoints: Vec<String>,
    /// True when reopened audit/state records cover every traced candidate.
    pub trace_recovery_validated: bool,
    /// Always false; recovery validation consumes supplied local records only.
    pub external_calls_performed: bool,
    /// Always false; recovery validation never submits orders or moves funds.
    pub live_execution_performed: bool,
    /// Overall recovery validation status.
    pub status: OpportunityPlannerHandoffStatus,
}

/// Local strategy-profile replay validation status across the historical fixture corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrategyProfileReplayValidationStatus {
    /// Every discoverable replay candidate produced the expected accepted/rejected profile outcomes.
    Passed,
    /// One or more replay candidates failed accepted/rejected profile validation.
    Failed,
}

/// Local strategy-profile replay validation report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProfileReplayValidationReport {
    /// Source historical fixture corpus id.
    pub corpus_id: String,
    /// Number of replay windows inspected.
    pub replay_window_count: usize,
    /// Number of replay scenarios inspected.
    pub replay_scenario_count: usize,
    /// Number of discovery validation failures skipped as fail-closed replay windows.
    pub skipped_discovery_failures: usize,
    /// Number of opportunity candidates discovered and offered to the planner.
    pub discovered_candidates: usize,
    /// Number of accepted-profile planner runs that produced a draft.
    pub accepted_planned_candidates: usize,
    /// Number of accepted-profile planner runs that remained draft-ready.
    pub accepted_draft_ready_plans: usize,
    /// Number of accepted-profile constraint reports that remained satisfied.
    pub accepted_satisfied_constraint_reports: usize,
    /// Number of accepted-profile strategy-rejected intents.
    pub accepted_strategy_rejected_intents: usize,
    /// Number of rejected-profile planner runs that produced a draft.
    pub rejected_planned_candidates: usize,
    /// Number of rejected-profile planner runs that were denied by strategy constraints.
    pub rejected_policy_denied_plans: usize,
    /// Number of rejected-profile constraint reports that were rejected.
    pub rejected_constraint_reports: usize,
    /// Number of rejected-profile strategy-rejected intents.
    pub rejected_strategy_rejected_intents: usize,
    /// Number of strategy replay planner failures across both profiles.
    pub failed_strategy_planner_runs: usize,
    /// Total accepted-profile draft intents across generated plans.
    pub total_accepted_intents: usize,
    /// Total rejected-profile draft intents across generated plans.
    pub total_rejected_intents: usize,
    /// True only if a bug enabled adapter submission.
    pub adapter_submission_performed: bool,
    /// Always false; validation consumes supplied local records only.
    pub external_calls_performed: bool,
    /// Always false; validation never submits orders or moves funds.
    pub live_execution_performed: bool,
    /// True only if a bug enabled signing or broadcasting.
    pub signing_or_broadcast_performed: bool,
    /// True only if a bug marked the replay as production-ready.
    pub production_ready: bool,
    /// Overall strategy replay validation status.
    pub status: StrategyProfileReplayValidationStatus,
}

/// Local strategy profitability tuning validation status across the historical fixture corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrategyProfitabilityTuningValidationStatus {
    /// The local profitability threshold sweep behaved monotonically and remained side-effect free.
    Passed,
    /// One or more profitability threshold invariants failed.
    Failed,
}

/// Local profitability threshold point over the replay corpus.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProfitabilityTuningPoint {
    /// Minimum net profit threshold applied to every draft intent.
    pub min_net_profit_abs: f64,
    /// Number of candidate planner runs attempted at this threshold.
    pub planned_candidates: usize,
    /// Number of draft-ready plans at this threshold.
    pub draft_ready_plans: usize,
    /// Number of policy-denied drafts at this threshold.
    pub policy_denied_plans: usize,
    /// Number of draft intents accepted at this threshold.
    pub accepted_intents: usize,
    /// Number of draft intents rejected at this threshold.
    pub rejected_intents: usize,
    /// Number of planner runs that failed before reporting a draft.
    pub failed_planner_runs: usize,
    /// Aggregate expected net profit for accepted intents only.
    pub total_expected_net_profit_quote: f64,
    /// Lowest accepted intent net profit observed at this threshold.
    pub lowest_accepted_net_profit_quote: Option<f64>,
    /// Highest accepted intent net profit observed at this threshold.
    pub highest_accepted_net_profit_quote: Option<f64>,
}

/// Local strategy profitability tuning validation report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProfitabilityTuningValidationReport {
    /// Source historical fixture corpus id.
    pub corpus_id: String,
    /// Number of replay windows inspected.
    pub replay_window_count: usize,
    /// Number of replay scenarios inspected.
    pub replay_scenario_count: usize,
    /// Number of discovery validation failures skipped as fail-closed replay windows.
    pub skipped_discovery_failures: usize,
    /// Number of opportunity candidates discovered and offered to the planner.
    pub discovered_candidates: usize,
    /// Monotonic threshold points derived from the local corpus.
    pub profitability_points: Vec<StrategyProfitabilityTuningPoint>,
    /// Whether accepted-plan/intents counts decreased monotonically as thresholds rose.
    pub monotonic_acceptance_validated: bool,
    /// Whether denied-plan/rejected-intent counts increased monotonically as thresholds rose.
    pub monotonic_rejection_validated: bool,
    /// Whether the sweep observed at least one threshold transition across the corpus.
    pub threshold_transition_observed: bool,
    /// True only if a bug enabled adapter submission.
    pub adapter_submission_performed: bool,
    /// Always false; validation consumes supplied local records only.
    pub external_calls_performed: bool,
    /// Always false; validation never submits orders or moves funds.
    pub live_execution_performed: bool,
    /// True only if a bug enabled signing or broadcasting.
    pub signing_or_broadcast_performed: bool,
    /// True only if a bug marked the sweep as production-ready.
    pub production_ready: bool,
    /// Overall profitability tuning validation status.
    pub status: StrategyProfitabilityTuningValidationStatus,
}

/// Validate local historical replay candidates against accepted/rejected strategy profiles.
///
/// This consumes caller-supplied local replay records only. It does not call
/// exchanges/RPC endpoints, submit adapters, sign, broadcast, withdraw, bridge,
/// mutate balances, or claim production readiness.
pub fn validate_strategy_profile_replay_corpus(
    corpus: &OpportunityHistoricalFixtureCorpus,
    policy: &PolicyEngine,
    accepted_profile: &StrategyProfile,
    rejected_profile: &StrategyProfile,
) -> Result<StrategyProfileReplayValidationReport, ExecutionPlannerError> {
    corpus
        .validate()
        .map_err(opportunity_error_to_planner_error)?;

    let engine = DeterministicOpportunityEngine::new();
    let planner = DeterministicExecutionPlanner::new();
    let mut replay_scenario_count = 0;
    let mut skipped_discovery_failures = 0;
    let mut discovered_candidates = 0;
    let mut accepted_planned_candidates = 0;
    let mut accepted_draft_ready_plans = 0;
    let mut accepted_satisfied_constraint_reports = 0;
    let mut accepted_strategy_rejected_intents = 0;
    let mut rejected_planned_candidates = 0;
    let mut rejected_policy_denied_plans = 0;
    let mut rejected_constraint_reports = 0;
    let mut rejected_strategy_rejected_intents = 0;
    let mut failed_strategy_planner_runs = 0;
    let mut total_accepted_intents = 0;
    let mut total_rejected_intents = 0;
    let mut adapter_submission_performed = false;
    let mut live_execution_performed = false;
    let mut signing_or_broadcast_performed = false;
    let mut production_ready = false;

    for window in &corpus.replay_windows {
        for scenario in &window.scenarios {
            replay_scenario_count += 1;
            let Ok(candidates) = engine.discover(&scenario.request) else {
                skipped_discovery_failures += 1;
                continue;
            };

            for candidate in candidates {
                discovered_candidates += 1;
                let accepted_request = ExecutionPlannerRequest {
                    id: format!("phase27-strategy-replay-accepted-{discovered_candidates}"),
                    strategy_id: accepted_profile.id.clone(),
                    candidate: candidate.clone(),
                    config: ExecutionPlannerConfig {
                        requested_scope: ExecutionScope::Paper,
                        max_plan_legs: 4,
                        max_total_notional_quote: 1_000_000.0,
                        default_slippage_bps: 50,
                        max_market_data_age_ms: DEFAULT_MAX_MARKET_DATA_AGE_MS,
                        require_policy_preflight: true,
                    },
                    default_chain: Some("ethereum".to_owned()),
                    now_unix_ms: 10_000,
                };
                match planner.plan_with_strategy_profile(
                    &accepted_request,
                    policy,
                    accepted_profile,
                ) {
                    Ok(plan) => {
                        accepted_planned_candidates += 1;
                        total_accepted_intents += plan.draft.intents.len();
                        accepted_satisfied_constraint_reports += plan
                            .strategy_constraint_reports
                            .iter()
                            .filter(|report| {
                                report.status == StrategyPolicyConstraintStatus::Satisfied
                            })
                            .count();
                        accepted_strategy_rejected_intents += plan.strategy_rejected_intents;
                        adapter_submission_performed |= plan.adapter_submission_performed;
                        live_execution_performed |= plan.live_execution_performed;
                        signing_or_broadcast_performed |= plan.signing_or_broadcast_performed;
                        production_ready |= plan.production_ready;
                        if plan.draft.status == ExecutionPlanStatus::DraftReady {
                            accepted_draft_ready_plans += 1;
                        }
                    }
                    Err(_) => failed_strategy_planner_runs += 1,
                }

                let rejected_request = ExecutionPlannerRequest {
                    id: format!("phase27-strategy-replay-rejected-{discovered_candidates}"),
                    strategy_id: rejected_profile.id.clone(),
                    candidate,
                    config: ExecutionPlannerConfig {
                        requested_scope: ExecutionScope::Paper,
                        max_plan_legs: 4,
                        max_total_notional_quote: 1_000_000.0,
                        default_slippage_bps: 50,
                        max_market_data_age_ms: DEFAULT_MAX_MARKET_DATA_AGE_MS,
                        require_policy_preflight: true,
                    },
                    default_chain: Some("ethereum".to_owned()),
                    now_unix_ms: 10_000,
                };
                match planner.plan_with_strategy_profile(
                    &rejected_request,
                    policy,
                    rejected_profile,
                ) {
                    Ok(plan) => {
                        rejected_planned_candidates += 1;
                        total_rejected_intents += plan.draft.intents.len();
                        rejected_constraint_reports += plan
                            .strategy_constraint_reports
                            .iter()
                            .filter(|report| {
                                report.status == StrategyPolicyConstraintStatus::Rejected
                            })
                            .count();
                        rejected_strategy_rejected_intents += plan.strategy_rejected_intents;
                        adapter_submission_performed |= plan.adapter_submission_performed;
                        live_execution_performed |= plan.live_execution_performed;
                        signing_or_broadcast_performed |= plan.signing_or_broadcast_performed;
                        production_ready |= plan.production_ready;
                        if plan.draft.status == ExecutionPlanStatus::PolicyDeniedDraft {
                            rejected_policy_denied_plans += 1;
                        }
                    }
                    Err(_) => failed_strategy_planner_runs += 1,
                }
            }
        }
    }

    let status = if discovered_candidates > 0
        && accepted_planned_candidates == discovered_candidates
        && accepted_draft_ready_plans == discovered_candidates
        && accepted_satisfied_constraint_reports == total_accepted_intents
        && accepted_strategy_rejected_intents == 0
        && rejected_planned_candidates == discovered_candidates
        && rejected_policy_denied_plans == discovered_candidates
        && rejected_constraint_reports == total_rejected_intents
        && rejected_strategy_rejected_intents == total_rejected_intents
        && failed_strategy_planner_runs == 0
        && !adapter_submission_performed
        && !live_execution_performed
        && !signing_or_broadcast_performed
        && !production_ready
    {
        StrategyProfileReplayValidationStatus::Passed
    } else {
        StrategyProfileReplayValidationStatus::Failed
    };

    Ok(StrategyProfileReplayValidationReport {
        corpus_id: corpus.id.clone(),
        replay_window_count: corpus.replay_windows.len(),
        replay_scenario_count,
        skipped_discovery_failures,
        discovered_candidates,
        accepted_planned_candidates,
        accepted_draft_ready_plans,
        accepted_satisfied_constraint_reports,
        accepted_strategy_rejected_intents,
        rejected_planned_candidates,
        rejected_policy_denied_plans,
        rejected_constraint_reports,
        rejected_strategy_rejected_intents,
        failed_strategy_planner_runs,
        total_accepted_intents,
        total_rejected_intents,
        adapter_submission_performed,
        external_calls_performed: false,
        live_execution_performed,
        signing_or_broadcast_performed,
        production_ready,
        status,
    })
}

/// Validate local strategy profitability tuning over the historical fixture corpus.
///
/// This derives a low/median/high threshold sweep from observed local replay
/// intent profitability, then proves monotonic draft-ready vs policy-denied
/// behavior without adapter submission, signing, broadcasting, or live calls.
pub fn validate_strategy_profitability_tuning(
    corpus: &OpportunityHistoricalFixtureCorpus,
    policy: &PolicyEngine,
) -> Result<StrategyProfitabilityTuningValidationReport, ExecutionPlannerError> {
    corpus
        .validate()
        .map_err(opportunity_error_to_planner_error)?;

    let engine = DeterministicOpportunityEngine::new();
    let planner = DeterministicExecutionPlanner::new();
    let mut replay_scenario_count = 0;
    let mut skipped_discovery_failures = 0;
    let mut candidates = Vec::new();

    for window in &corpus.replay_windows {
        for scenario in &window.scenarios {
            replay_scenario_count += 1;
            let Ok(discovered) = engine.discover(&scenario.request) else {
                skipped_discovery_failures += 1;
                continue;
            };
            candidates.extend(discovered);
        }
    }

    if candidates.is_empty() {
        return Err(planner_validation_error(
            "STRATEGY_PROFITABILITY_TUNING_CANDIDATES_REQUIRED",
            "strategy profitability tuning requires at least one discovered candidate",
        ));
    }

    let baseline_profile = local_strategy_profitability_profile(
        "phase27-local-profitability-baseline".to_owned(),
        0.0,
    );
    let mut observed_intent_profits = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let request = local_strategy_planning_request(
            format!("phase27-strategy-profitability-baseline-{}", index + 1),
            baseline_profile.id.clone(),
            candidate.clone(),
        );
        let plan = planner
            .plan_with_strategy_profile(&request, policy, &baseline_profile)
            .map_err(|error| {
                planner_validation_error_owned(
                    "STRATEGY_PROFITABILITY_TUNING_BASELINE_FAILED",
                    error.to_string(),
                )
            })?;
        for intent in &plan.draft.intents {
            observed_intent_profits.push(intent_net_profit_quote(intent));
        }
    }

    if observed_intent_profits.is_empty() {
        return Err(planner_validation_error(
            "STRATEGY_PROFITABILITY_TUNING_INTENTS_REQUIRED",
            "strategy profitability tuning requires at least one draft intent",
        ));
    }

    observed_intent_profits.sort_by(f64::total_cmp);
    let highest_observed_profit = *observed_intent_profits.last().ok_or_else(|| {
        planner_validation_error(
            "STRATEGY_PROFITABILITY_TUNING_INTENTS_REQUIRED",
            "strategy profitability tuning requires at least one draft intent",
        )
    })?;
    let median_observed_profit =
        observed_intent_profits[observed_intent_profits.len() / 2].max(0.0);
    let derived_midpoint = if median_observed_profit > f64::EPSILON {
        median_observed_profit
    } else {
        (highest_observed_profit / 2.0).max(0.01)
    };
    let profitability_thresholds = [
        0.0,
        derived_midpoint,
        (highest_observed_profit + 0.01).max(derived_midpoint + 0.01),
    ];

    let mut profitability_points = Vec::with_capacity(profitability_thresholds.len());
    let mut adapter_submission_performed = false;
    let mut live_execution_performed = false;
    let mut signing_or_broadcast_performed = false;
    let mut production_ready = false;

    for (threshold_index, threshold) in profitability_thresholds.iter().enumerate() {
        let profile = local_strategy_profitability_profile(
            format!(
                "phase27-local-profitability-threshold-{}",
                threshold_index + 1
            ),
            *threshold,
        );
        let mut point = StrategyProfitabilityTuningPoint {
            min_net_profit_abs: *threshold,
            planned_candidates: 0,
            draft_ready_plans: 0,
            policy_denied_plans: 0,
            accepted_intents: 0,
            rejected_intents: 0,
            failed_planner_runs: 0,
            total_expected_net_profit_quote: 0.0,
            lowest_accepted_net_profit_quote: None,
            highest_accepted_net_profit_quote: None,
        };

        for (candidate_index, candidate) in candidates.iter().enumerate() {
            let request = local_strategy_planning_request(
                format!(
                    "phase27-strategy-profitability-{}-{}",
                    threshold_index + 1,
                    candidate_index + 1
                ),
                profile.id.clone(),
                candidate.clone(),
            );
            match planner.plan_with_strategy_profile(&request, policy, &profile) {
                Ok(plan) => {
                    point.planned_candidates += 1;
                    match plan.draft.status {
                        ExecutionPlanStatus::DraftReady => point.draft_ready_plans += 1,
                        ExecutionPlanStatus::PolicyDeniedDraft => point.policy_denied_plans += 1,
                    }
                    point.rejected_intents += plan.strategy_rejected_intents;
                    point.accepted_intents += plan
                        .draft
                        .intents
                        .len()
                        .saturating_sub(plan.strategy_rejected_intents);
                    for (intent, constraint_report) in plan
                        .draft
                        .intents
                        .iter()
                        .zip(plan.strategy_constraint_reports.iter())
                    {
                        if constraint_report.status != StrategyPolicyConstraintStatus::Satisfied {
                            continue;
                        }
                        let net_profit_quote = intent_net_profit_quote(intent);
                        point.total_expected_net_profit_quote += net_profit_quote;
                        point.lowest_accepted_net_profit_quote = Some(
                            point
                                .lowest_accepted_net_profit_quote
                                .map_or(net_profit_quote, |current| current.min(net_profit_quote)),
                        );
                        point.highest_accepted_net_profit_quote = Some(
                            point
                                .highest_accepted_net_profit_quote
                                .map_or(net_profit_quote, |current| current.max(net_profit_quote)),
                        );
                    }
                    adapter_submission_performed |= plan.adapter_submission_performed;
                    live_execution_performed |= plan.live_execution_performed;
                    signing_or_broadcast_performed |= plan.signing_or_broadcast_performed;
                    production_ready |= plan.production_ready;
                }
                Err(_) => point.failed_planner_runs += 1,
            }
        }

        profitability_points.push(point);
    }

    let monotonic_acceptance_validated = profitability_points.windows(2).all(|window| {
        window[1].draft_ready_plans <= window[0].draft_ready_plans
            && window[1].accepted_intents <= window[0].accepted_intents
            && window[1].total_expected_net_profit_quote
                <= window[0].total_expected_net_profit_quote + f64::EPSILON
    });
    let monotonic_rejection_validated = profitability_points.windows(2).all(|window| {
        window[1].policy_denied_plans >= window[0].policy_denied_plans
            && window[1].rejected_intents >= window[0].rejected_intents
    });
    let threshold_transition_observed = profitability_points
        .first()
        .zip(profitability_points.last())
        .is_some_and(|(first, last)| {
            first.accepted_intents > last.accepted_intents
                && last.policy_denied_plans > first.policy_denied_plans
                && last.accepted_intents == 0
                && last.policy_denied_plans == candidates.len()
        });

    let status = if profitability_points
        .iter()
        .all(|point| point.failed_planner_runs == 0)
        && monotonic_acceptance_validated
        && monotonic_rejection_validated
        && threshold_transition_observed
        && !adapter_submission_performed
        && !live_execution_performed
        && !signing_or_broadcast_performed
        && !production_ready
    {
        StrategyProfitabilityTuningValidationStatus::Passed
    } else {
        StrategyProfitabilityTuningValidationStatus::Failed
    };

    Ok(StrategyProfitabilityTuningValidationReport {
        corpus_id: corpus.id.clone(),
        replay_window_count: corpus.replay_windows.len(),
        replay_scenario_count,
        skipped_discovery_failures,
        discovered_candidates: candidates.len(),
        profitability_points,
        monotonic_acceptance_validated,
        monotonic_rejection_validated,
        threshold_transition_observed,
        adapter_submission_performed,
        external_calls_performed: false,
        live_execution_performed,
        signing_or_broadcast_performed,
        production_ready,
        status,
    })
}

fn local_strategy_profitability_profile(id: String, min_net_profit_abs: f64) -> StrategyProfile {
    let mut profile = StrategyProfile::conservative_paper(id, "USD");
    profile.capital.max_total_deployed = 1_000_000.0;
    profile.capital.max_per_opportunity = 1_000_000.0;
    profile.capital.reserve_minimum = 0.0;
    profile.risk.max_single_tx_value = 1_000_000.0;
    profile.opportunity.min_net_profit_abs = min_net_profit_abs;
    profile.execution.max_slippage_bps = u16::MAX;
    profile.venues.allowed_exchanges.clear();
    profile.venues.allowed_assets.clear();
    profile.venues.allowed_chains.clear();
    profile.venues.allowed_routers.clear();
    profile
}

fn local_strategy_planning_request(
    id: String,
    strategy_id: String,
    candidate: OpportunityCandidate,
) -> ExecutionPlannerRequest {
    ExecutionPlannerRequest {
        id,
        strategy_id,
        candidate,
        config: ExecutionPlannerConfig {
            requested_scope: ExecutionScope::Paper,
            max_plan_legs: 4,
            max_total_notional_quote: 1_000_000.0,
            default_slippage_bps: 50,
            max_market_data_age_ms: DEFAULT_MAX_MARKET_DATA_AGE_MS,
            require_policy_preflight: true,
        },
        default_chain: Some("ethereum".to_owned()),
        now_unix_ms: 10_000,
    }
}

fn intent_net_profit_quote(intent: &ExecutionIntent) -> f64 {
    intent.expected_profit_quote - intent.estimated_fee_quote - intent.gas_fee_quote
}

/// Validate local opportunity replay candidates can be handed to the draft planner.
///
/// This consumes caller-supplied local replay records only. It does not call
/// exchanges/RPC endpoints, submit adapters, sign, broadcast, withdraw, bridge,
/// mutate balances, or claim production readiness.
pub fn validate_opportunity_planner_handoff(
    corpus: &OpportunityHistoricalFixtureCorpus,
    policy: &PolicyEngine,
) -> Result<OpportunityPlannerHandoffValidationReport, ExecutionPlannerError> {
    validate_opportunity_planner_handoff_internal::<crate::InMemoryStateStore>(corpus, policy, None)
}

/// Validate local opportunity replay candidates with caller-supplied audit/state trace sinks.
///
/// This traced variant appends a local candidate audit event and persists a
/// local state checkpoint before each draft-only planner handoff.
pub fn validate_opportunity_planner_handoff_with_trace(
    corpus: &OpportunityHistoricalFixtureCorpus,
    policy: &PolicyEngine,
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
) -> Result<OpportunityPlannerHandoffValidationReport, ExecutionPlannerError> {
    validate_opportunity_planner_handoff_internal(
        corpus,
        policy,
        Some(OpportunityPlannerTraceSinks { journal, store }),
    )
}

/// Validate local opportunity candidate traces survive audit/state reopen.
///
/// The supplied paths are local non-secret evidence paths. This function writes
/// candidate traces through the regular traced planner handoff path, drops those
/// handles, reopens the audit journal and SQLite WAL state, and verifies every
/// expected trace checkpoint can be recovered. It does not start services,
/// submit adapters, call exchanges/RPCs, sign, broadcast, withdraw, bridge, or
/// claim production readiness.
pub fn validate_opportunity_candidate_trace_restart_recovery(
    corpus: &OpportunityHistoricalFixtureCorpus,
    policy: &PolicyEngine,
    audit_path: &Path,
    state_path: &Path,
) -> Result<OpportunityCandidateTraceRecoveryReport, ExecutionPlannerError> {
    if audit_path.as_os_str().is_empty() || state_path.as_os_str().is_empty() {
        return Err(planner_validation_error(
            "OPPORTUNITY_CANDIDATE_TRACE_RECOVERY_PATH_REQUIRED",
            "audit and state paths are required",
        ));
    }
    if audit_path == state_path {
        return Err(planner_validation_error(
            "OPPORTUNITY_CANDIDATE_TRACE_RECOVERY_PATH_CONFLICT",
            "audit and state paths must be different",
        ));
    }

    let handoff_report = {
        let mut journal = AppendOnlyAuditJournal::open(audit_path)
            .map_err(audit_error_to_trace_recovery_error)?;
        let mut store =
            SqliteWalStateStore::open(state_path).map_err(state_error_to_trace_recovery_error)?;
        validate_opportunity_planner_handoff_with_trace(corpus, policy, &mut journal, &mut store)?
    };

    let reopened_journal =
        AppendOnlyAuditJournal::open(audit_path).map_err(audit_error_to_trace_recovery_error)?;
    let audit_replay_records = usize::try_from(reopened_journal.next_sequence().saturating_sub(1))
        .map_err(|_| {
            planner_validation_error(
                "OPPORTUNITY_CANDIDATE_TRACE_RECOVERY_COUNT_OVERFLOW",
                "audit replay count exceeds usize",
            )
        })?;

    let reopened_store =
        SqliteWalStateStore::open(state_path).map_err(state_error_to_trace_recovery_error)?;
    reopened_store
        .integrity_check()
        .map_err(state_error_to_trace_recovery_error)?;
    reopened_store
        .wal_checkpoint_truncate()
        .map_err(state_error_to_trace_recovery_error)?;

    let expected_trace_keys = expected_candidate_trace_keys(corpus)?;
    let mut recovered_trace_checkpoints = 0;
    let mut recovered_trace_summaries = Vec::new();
    let mut missing_trace_checkpoints = Vec::new();
    let mut forbidden_side_effects = false;

    for expected_key in &expected_trace_keys {
        match reopened_store
            .get_checkpoint(expected_key)
            .map_err(state_error_to_trace_recovery_error)?
        {
            Some(checkpoint) => {
                let trace: OpportunityCandidateTraceRecord =
                    serde_json::from_str(&checkpoint.value).map_err(|error| {
                        planner_validation_error_owned(
                            "OPPORTUNITY_CANDIDATE_TRACE_RECOVERY_PARSE_FAILED",
                            format!(
                                "failed to parse recovered candidate trace checkpoint: {error}"
                            ),
                        )
                    })?;
                if trace.id != *expected_key
                    || trace.adapter_submission_enabled
                    || trace.external_calls_performed
                    || trace.live_execution_performed
                {
                    forbidden_side_effects = true;
                }
                recovered_trace_summaries.push(RecoveredOpportunityTraceSummary {
                    trace_id: trace.id,
                    strategy_id: trace.strategy_id,
                    planner_request_id: trace.planner_request_id,
                    audit_sequence: trace.audit_sequence,
                    traced_at_unix_ms: trace.traced_at_unix_ms,
                    route_kind: trace.candidate.route_kind,
                    leg_count: trace.candidate.legs.len() as u64,
                });
                recovered_trace_checkpoints += 1;
            }
            None => missing_trace_checkpoints.push(expected_key.clone()),
        }
    }

    let trace_recovery_validated = handoff_report.status == OpportunityPlannerHandoffStatus::Passed
        && audit_replay_records == handoff_report.candidate_trace_audit_records
        && recovered_trace_checkpoints == handoff_report.discovered_candidates
        && missing_trace_checkpoints.is_empty()
        && !forbidden_side_effects
        && !handoff_report.adapter_submission_enabled
        && !handoff_report.external_calls_performed
        && !handoff_report.live_execution_performed;
    let status = if trace_recovery_validated {
        OpportunityPlannerHandoffStatus::Passed
    } else {
        OpportunityPlannerHandoffStatus::Failed
    };

    Ok(OpportunityCandidateTraceRecoveryReport {
        corpus_id: corpus.id.clone(),
        handoff_report,
        audit_replay_records,
        recovered_trace_checkpoints,
        recovered_trace_summaries,
        missing_trace_checkpoints,
        trace_recovery_validated,
        external_calls_performed: false,
        live_execution_performed: false,
        status,
    })
}

struct OpportunityPlannerTraceSinks<'a, S: StateStore> {
    journal: &'a mut AppendOnlyAuditJournal,
    store: &'a mut S,
}

fn validate_opportunity_planner_handoff_internal<S: StateStore>(
    corpus: &OpportunityHistoricalFixtureCorpus,
    policy: &PolicyEngine,
    mut trace_sinks: Option<OpportunityPlannerTraceSinks<'_, S>>,
) -> Result<OpportunityPlannerHandoffValidationReport, ExecutionPlannerError> {
    corpus
        .validate()
        .map_err(opportunity_error_to_planner_error)?;

    let engine = DeterministicOpportunityEngine::new();
    let planner = DeterministicExecutionPlanner::new();
    let mut replay_scenario_count = 0;
    let mut skipped_discovery_failures = 0;
    let mut discovered_candidates = 0;
    let mut planned_candidates = 0;
    let mut draft_ready_plans = 0;
    let mut policy_denied_plans = 0;
    let mut failed_planner_handoffs = 0;
    let mut candidate_trace_audit_records = 0;
    let mut candidate_trace_checkpoints = 0;
    let mut total_intents = 0;
    let mut adapter_submission_enabled = false;

    for window in &corpus.replay_windows {
        for scenario in &window.scenarios {
            replay_scenario_count += 1;
            let Ok(candidates) = engine.discover(&scenario.request) else {
                skipped_discovery_failures += 1;
                continue;
            };

            for candidate in candidates {
                discovered_candidates += 1;
                let planner_request_id = format!("phase27-planner-handoff-{discovered_candidates}");
                let strategy_id = "phase27-local-replay-handoff";
                if let Some(sinks) = trace_sinks.as_mut() {
                    let persisted = persist_opportunity_candidate_trace(
                        sinks.journal,
                        sinks.store,
                        &candidate,
                        strategy_id,
                        &planner_request_id,
                        10_000,
                    )
                    .map_err(trace_error_to_planner_error)?;
                    candidate_trace_audit_records +=
                        usize::from(persisted.audit_record.sequence > 0);
                    candidate_trace_checkpoints += usize::from(
                        persisted.checkpoint.subsystem
                            == OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM,
                    );
                }
                let request = ExecutionPlannerRequest {
                    id: planner_request_id,
                    strategy_id: strategy_id.to_owned(),
                    candidate,
                    config: ExecutionPlannerConfig {
                        requested_scope: ExecutionScope::Paper,
                        max_plan_legs: 4,
                        max_total_notional_quote: 1_000_000.0,
                        default_slippage_bps: 50,
                        max_market_data_age_ms: DEFAULT_MAX_MARKET_DATA_AGE_MS,
                        require_policy_preflight: true,
                    },
                    default_chain: Some("ethereum".to_owned()),
                    now_unix_ms: 10_000,
                };

                match planner.plan(&request, policy) {
                    Ok(plan) => {
                        planned_candidates += 1;
                        total_intents += plan.intents.len();
                        adapter_submission_enabled |= plan.adapter_submission_enabled;
                        match plan.status {
                            ExecutionPlanStatus::DraftReady => draft_ready_plans += 1,
                            ExecutionPlanStatus::PolicyDeniedDraft => policy_denied_plans += 1,
                        }
                    }
                    Err(_) => failed_planner_handoffs += 1,
                }
            }
        }
    }

    let status = if discovered_candidates > 0
        && planned_candidates == discovered_candidates
        && failed_planner_handoffs == 0
        && !adapter_submission_enabled
    {
        OpportunityPlannerHandoffStatus::Passed
    } else {
        OpportunityPlannerHandoffStatus::Failed
    };

    Ok(OpportunityPlannerHandoffValidationReport {
        corpus_id: corpus.id.clone(),
        replay_window_count: corpus.replay_windows.len(),
        replay_scenario_count,
        skipped_discovery_failures,
        discovered_candidates,
        planned_candidates,
        draft_ready_plans,
        policy_denied_plans,
        failed_planner_handoffs,
        candidate_trace_audit_records,
        candidate_trace_checkpoints,
        total_intents,
        adapter_submission_enabled,
        external_calls_performed: false,
        live_execution_performed: false,
        status,
    })
}

/// Execution planner trait boundary.
pub trait ExecutionPlanner {
    /// Stable planner name for diagnostics and audit records.
    fn planner_name(&self) -> &str;

    /// Convert a validated opportunity into a policy-evaluated plan draft.
    fn plan(
        &self,
        request: &ExecutionPlannerRequest,
        policy: &PolicyEngine,
    ) -> Result<ExecutionPlanDraft, ExecutionPlannerError>;
}

/// Deterministic draft-only execution planner.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicExecutionPlanner;

impl DeterministicExecutionPlanner {
    /// Create a deterministic execution planner.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Plan through the typed local strategy profile constraint boundary.
    ///
    /// This composes deterministic draft planning, policy preflight, and
    /// strategy-profile intent constraints without submitting adapters or
    /// performing live side effects.
    pub fn plan_with_strategy_profile(
        &self,
        request: &ExecutionPlannerRequest,
        policy: &PolicyEngine,
        strategy_profile: &StrategyProfile,
    ) -> Result<StrategyConstrainedExecutionPlanDraft, ExecutionPlannerError> {
        strategy_profile.validate().map_err(|error| {
            let violations = error
                .violations()
                .iter()
                .map(|violation| {
                    ExecutionPlannerViolation::new_owned(
                        "PLANNER_STRATEGY_PROFILE_INVALID",
                        format!("{}: {}", violation.code(), violation.message()),
                    )
                })
                .collect();
            ExecutionPlannerError::ValidationFailed { violations }
        })?;

        if request.strategy_id != strategy_profile.id {
            return Err(planner_validation_error_owned(
                "PLANNER_STRATEGY_PROFILE_MISMATCH",
                format!(
                    "planner request strategy {} does not match strategy profile {}",
                    request.strategy_id, strategy_profile.id
                ),
            ));
        }

        let mut draft = self.plan(request, policy)?;
        let strategy_constraint_reports = draft
            .intents
            .iter()
            .map(|intent| strategy_profile.constrain_intent(intent))
            .collect::<Vec<_>>();
        let strategy_rejected_intents = strategy_constraint_reports
            .iter()
            .filter(|report| report.status == StrategyPolicyConstraintStatus::Rejected)
            .count();

        if strategy_rejected_intents > 0 {
            draft.status = ExecutionPlanStatus::PolicyDeniedDraft;
            draft.warnings.push(format!(
                "{strategy_rejected_intents} draft intent(s) rejected by strategy profile constraints"
            ));
            draft.validate()?;
        }

        let report = StrategyConstrainedExecutionPlanDraft {
            draft,
            strategy_constraint_reports,
            strategy_rejected_intents,
            adapter_submission_performed: false,
            live_execution_performed: false,
            signing_or_broadcast_performed: false,
            production_ready: false,
        };
        report.validate()?;
        Ok(report)
    }
}

impl ExecutionPlanner for DeterministicExecutionPlanner {
    fn planner_name(&self) -> &str {
        "deterministic-phase-10-execution-planner"
    }

    fn plan(
        &self,
        request: &ExecutionPlannerRequest,
        policy: &PolicyEngine,
    ) -> Result<ExecutionPlanDraft, ExecutionPlannerError> {
        request.validate()?;

        let intents = build_intents(request);
        let policy_outcomes = intents
            .iter()
            .map(|intent| PlannerPolicyOutcome::from_decision(&intent.id, policy.evaluate(intent)))
            .collect::<Vec<_>>();
        let status = if request.config.require_policy_preflight
            && policy_outcomes.iter().any(|outcome| !outcome.is_approved())
        {
            ExecutionPlanStatus::PolicyDeniedDraft
        } else {
            ExecutionPlanStatus::DraftReady
        };
        let steps = build_steps(&intents, request.config.requested_scope);
        let failure_modes = failure_modes_for(&request.candidate);
        let warnings = plan_warnings(request, status);

        let draft = ExecutionPlanDraft {
            id: format!("plan:{}:{}", request.candidate.id, request.id),
            request_id: request.id.clone(),
            candidate_id: request.candidate.id.clone(),
            planner_version: EXECUTION_PLANNER_VERSION.to_owned(),
            status,
            scope: request.config.requested_scope,
            intents,
            steps,
            policy_outcomes,
            failure_modes,
            total_notional_quote: total_leg_notional_quote(&request.candidate.legs),
            expected_net_profit_quote: request.candidate.edge.net_profit_quote,
            created_at_unix_ms: request.now_unix_ms,
            adapter_submission_enabled: false,
            warnings,
        };
        draft.validate()?;
        Ok(draft)
    }
}

fn build_intents(request: &ExecutionPlannerRequest) -> Vec<ExecutionIntent> {
    request
        .candidate
        .legs
        .iter()
        .enumerate()
        .map(|(index, leg)| build_intent(request, leg, index))
        .collect()
}

fn build_intent(
    request: &ExecutionPlannerRequest,
    leg: &OpportunityLeg,
    index: usize,
) -> ExecutionIntent {
    let scope = request.config.requested_scope;
    let intent_kind = if scope == ExecutionScope::Observe {
        ExecutionIntentKind::Observation
    } else {
        intent_kind_for_venue(leg.venue.kind)
    };

    ExecutionIntent {
        id: format!(
            "intent:{}:leg-{:02}:{}",
            request.candidate.id,
            index + 1,
            leg_side_label(leg.side)
        ),
        strategy_id: request.strategy_id.clone(),
        kind: intent_kind,
        scope,
        venue: leg.venue.clone(),
        chain: chain_for_leg(request, leg),
        base_asset: leg.pair.base.clone(),
        quote_asset: leg.pair.quote.clone(),
        notional_quote: leg.notional_quote,
        expected_profit_quote: request.candidate.edge.gross_profit_quote,
        max_loss_quote: max_loss_for_leg(leg, request.config.default_slippage_bps),
        slippage_bps: request.config.default_slippage_bps,
        estimated_fee_quote: leg.fee_estimate.venue_fee_quote,
        gas_fee_quote: leg.fee_estimate.network_fee_quote,
        market_data_age_ms: leg.market_data_age_ms,
        destination: destination_for_leg(leg),
        requires_signing: false,
    }
}

fn intent_kind_for_venue(venue_kind: VenueKind) -> ExecutionIntentKind {
    match venue_kind {
        VenueKind::Cex => ExecutionIntentKind::CexOrder,
        VenueKind::Dex | VenueKind::Aggregator => ExecutionIntentKind::DexSwap,
        VenueKind::Bridge => ExecutionIntentKind::BridgeRoute,
    }
}

fn chain_for_leg(request: &ExecutionPlannerRequest, leg: &OpportunityLeg) -> Option<String> {
    match leg.venue.kind {
        VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge => request.default_chain.clone(),
        VenueKind::Cex => None,
    }
}

fn destination_for_leg(leg: &OpportunityLeg) -> DestinationPolicy {
    match leg.venue.kind {
        VenueKind::Cex => DestinationPolicy::InternalAccount,
        VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge => DestinationPolicy::None,
    }
}

fn max_loss_for_leg(leg: &OpportunityLeg, slippage_bps: u16) -> f64 {
    let slippage_loss_quote = leg.notional_quote * (f64::from(slippage_bps) / 10_000.0);
    slippage_loss_quote + leg.fee_estimate.total_fee_quote
}

fn build_steps(intents: &[ExecutionIntent], scope: ExecutionScope) -> Vec<ExecutionPlanStep> {
    let mut steps = Vec::new();
    let mut previous_sequence = None;

    for intent in intents {
        let policy_sequence = next_sequence(&steps);
        steps.push(ExecutionPlanStep {
            sequence: policy_sequence,
            action: ExecutionPlanStepAction::PolicyPreflight,
            intent_id: Some(intent.id.clone()),
            depends_on: previous_sequence.into_iter().collect(),
            failure_mode: ExecutionPlanFailureMode::AbortBeforeAdapterSubmission,
            description: format!("preflight policy for draft intent {}", intent.id),
        });

        let prepare_sequence = next_sequence(&steps);
        steps.push(ExecutionPlanStep {
            sequence: prepare_sequence,
            action: step_action_for_intent(intent, scope),
            intent_id: Some(intent.id.clone()),
            depends_on: vec![policy_sequence],
            failure_mode: failure_mode_for_intent(intent),
            description: format!(
                "prepare draft intent {} without adapter submission",
                intent.id
            ),
        });

        let boundary_sequence = next_sequence(&steps);
        steps.push(ExecutionPlanStep {
            sequence: boundary_sequence,
            action: ExecutionPlanStepAction::AwaitFillBoundary,
            intent_id: Some(intent.id.clone()),
            depends_on: vec![prepare_sequence],
            failure_mode: ExecutionPlanFailureMode::ReconcileBeforeNextLeg,
            description: format!(
                "future execution must reconcile intent {} before any dependent leg",
                intent.id
            ),
        });
        previous_sequence = Some(boundary_sequence);
    }

    let blocked_sequence = next_sequence(&steps);
    steps.push(ExecutionPlanStep {
        sequence: blocked_sequence,
        action: ExecutionPlanStepAction::AdapterSubmissionBlocked,
        intent_id: None,
        depends_on: previous_sequence.into_iter().collect(),
        failure_mode: ExecutionPlanFailureMode::ManualReviewRequired,
        description: "Phase 10 stops at draft planning; no adapter submission is enabled"
            .to_owned(),
    });

    steps
}

fn step_action_for_intent(
    intent: &ExecutionIntent,
    scope: ExecutionScope,
) -> ExecutionPlanStepAction {
    if scope == ExecutionScope::Observe || intent.kind == ExecutionIntentKind::Observation {
        return ExecutionPlanStepAction::RecordObservation;
    }

    match intent.kind {
        ExecutionIntentKind::CexOrder | ExecutionIntentKind::CrossExchangeArbitrage => {
            ExecutionPlanStepAction::PrepareCexOrderDraft
        }
        ExecutionIntentKind::DexSwap | ExecutionIntentKind::TriangularArbitrage => {
            ExecutionPlanStepAction::PrepareDexSwapDraft
        }
        ExecutionIntentKind::Observation
        | ExecutionIntentKind::BridgeRoute
        | ExecutionIntentKind::Transfer
        | ExecutionIntentKind::Withdrawal => ExecutionPlanStepAction::AdapterSubmissionBlocked,
    }
}

fn failure_mode_for_intent(intent: &ExecutionIntent) -> ExecutionPlanFailureMode {
    match intent.kind {
        ExecutionIntentKind::DexSwap | ExecutionIntentKind::BridgeRoute => {
            ExecutionPlanFailureMode::DoNotSignOrBroadcast
        }
        ExecutionIntentKind::CexOrder | ExecutionIntentKind::CrossExchangeArbitrage => {
            ExecutionPlanFailureMode::CancelUnfilledRemainder
        }
        ExecutionIntentKind::Observation
        | ExecutionIntentKind::TriangularArbitrage
        | ExecutionIntentKind::Transfer
        | ExecutionIntentKind::Withdrawal => ExecutionPlanFailureMode::ManualReviewRequired,
    }
}

fn failure_modes_for(candidate: &OpportunityCandidate) -> Vec<ExecutionPlanFailureMode> {
    let mut modes = vec![
        ExecutionPlanFailureMode::AbortBeforeAdapterSubmission,
        ExecutionPlanFailureMode::ReconcileBeforeNextLeg,
        ExecutionPlanFailureMode::CancelUnfilledRemainder,
        ExecutionPlanFailureMode::ManualReviewRequired,
    ];

    if candidate.legs.iter().any(|leg| {
        matches!(
            leg.venue.kind,
            VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge
        )
    }) {
        modes.push(ExecutionPlanFailureMode::DoNotSignOrBroadcast);
    }

    modes
}

fn plan_warnings(request: &ExecutionPlannerRequest, status: ExecutionPlanStatus) -> Vec<String> {
    let mut warnings = vec![
        "Phase 10 execution plans are draft-only and cannot submit to adapters".to_owned(),
        "policy approval is preflight metadata, not execution permission".to_owned(),
    ];

    if status == ExecutionPlanStatus::PolicyDeniedDraft {
        warnings.push("one or more draft intents were denied by policy".to_owned());
    }

    if request.config.requested_scope == ExecutionScope::Observe {
        warnings.push("observe scope emits observation intents only".to_owned());
    }

    if request.candidate.legs.iter().any(|leg| {
        matches!(
            leg.venue.kind,
            VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge
        )
    }) {
        warnings.push("DEX/Web3 legs remain non-signing and non-broadcastable".to_owned());
    }

    warnings.extend(request.candidate.warnings.clone());
    warnings
}

fn next_sequence(steps: &[ExecutionPlanStep]) -> u16 {
    let next = steps.len() + 1;
    u16::try_from(next).unwrap_or(u16::MAX)
}

fn total_leg_notional_quote(legs: &[OpportunityLeg]) -> f64 {
    legs.iter().map(|leg| leg.notional_quote).sum()
}

fn leg_side_label(side: OpportunityLegSide) -> &'static str {
    match side {
        OpportunityLegSide::Buy => "buy",
        OpportunityLegSide::Sell => "sell",
        OpportunityLegSide::Swap => "swap",
    }
}

fn collect_opportunity_error(
    error: OpportunityError,
    violations: &mut Vec<ExecutionPlannerViolation>,
) {
    if error.violations().is_empty() {
        violations.push(ExecutionPlannerViolation::new_owned(
            "PLANNER_CANDIDATE_INVALID",
            error.to_string(),
        ));
        return;
    }

    for violation in error.violations() {
        violations.push(ExecutionPlannerViolation::new_owned(
            "PLANNER_CANDIDATE_INVALID",
            format!("{}: {}", violation.code(), violation.message()),
        ));
    }
}

fn opportunity_error_to_planner_error(error: OpportunityError) -> ExecutionPlannerError {
    let mut violations = Vec::new();
    collect_opportunity_error(error, &mut violations);
    ExecutionPlannerError::ValidationFailed { violations }
}

fn trace_error_to_planner_error(error: OpportunityCandidateTraceError) -> ExecutionPlannerError {
    let message = error.to_string();
    ExecutionPlannerError::ValidationFailed {
        violations: vec![ExecutionPlannerViolation::new_owned(
            "OPPORTUNITY_CANDIDATE_TRACE_FAILED",
            message,
        )],
    }
}

fn audit_error_to_trace_recovery_error(error: crate::AuditError) -> ExecutionPlannerError {
    planner_validation_error_owned(
        "OPPORTUNITY_CANDIDATE_TRACE_RECOVERY_AUDIT_FAILED",
        error.to_string(),
    )
}

fn state_error_to_trace_recovery_error(error: StateStoreError) -> ExecutionPlannerError {
    planner_validation_error_owned(
        "OPPORTUNITY_CANDIDATE_TRACE_RECOVERY_STATE_FAILED",
        error.to_string(),
    )
}

fn expected_candidate_trace_keys(
    corpus: &OpportunityHistoricalFixtureCorpus,
) -> Result<Vec<String>, ExecutionPlannerError> {
    corpus
        .validate()
        .map_err(opportunity_error_to_planner_error)?;
    let engine = DeterministicOpportunityEngine::new();
    let mut expected_trace_keys = Vec::new();
    let mut discovered_candidates = 0;

    for window in &corpus.replay_windows {
        for scenario in &window.scenarios {
            let Ok(candidates) = engine.discover(&scenario.request) else {
                continue;
            };

            for candidate in candidates {
                discovered_candidates += 1;
                let planner_request_id = format!("phase27-planner-handoff-{discovered_candidates}");
                expected_trace_keys.push(opportunity_candidate_trace_key(
                    &candidate,
                    &planner_request_id,
                ));
            }
        }
    }

    Ok(expected_trace_keys)
}

fn planner_validation_error(code: &'static str, message: &'static str) -> ExecutionPlannerError {
    planner_validation_error_owned(code, message.to_owned())
}

fn planner_validation_error_owned(code: &'static str, message: String) -> ExecutionPlannerError {
    ExecutionPlannerError::ValidationFailed {
        violations: vec![ExecutionPlannerViolation::new_owned(code, message)],
    }
}

fn validate_candidate_trace_input(
    candidate: &OpportunityCandidate,
    strategy_id: &str,
    planner_request_id: &str,
    occurred_at_unix_ms: u64,
) -> Result<(), OpportunityCandidateTraceError> {
    let mut violations = Vec::new();
    if let Err(error) = candidate.validate() {
        collect_opportunity_error(error, &mut violations);
    }
    validate_id("strategy_id", strategy_id, &mut violations);
    validate_id("planner_request_id", planner_request_id, &mut violations);
    if occurred_at_unix_ms == 0 {
        violations.push(ExecutionPlannerViolation::new(
            "OPPORTUNITY_CANDIDATE_TRACE_TIME_ZERO",
            "candidate trace timestamp must be non-zero",
        ));
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(OpportunityCandidateTraceError::ValidationFailed { violations })
    }
}

fn opportunity_candidate_trace_key(
    candidate: &OpportunityCandidate,
    planner_request_id: &str,
) -> String {
    format!(
        "{}:{}:{}",
        OPPORTUNITY_CANDIDATE_TRACE_CHECKPOINT_KEY_PREFIX, candidate.id, planner_request_id
    )
}

fn validate_id(label: &'static str, value: &str, violations: &mut Vec<ExecutionPlannerViolation>) {
    if value.trim().is_empty() {
        violations.push(ExecutionPlannerViolation::new_owned(
            "PLANNER_ID_REQUIRED",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn finish_validation(
    violations: Vec<ExecutionPlannerViolation>,
) -> Result<(), ExecutionPlannerError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ExecutionPlannerError::ValidationFailed { violations })
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// One execution-planner validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlannerViolation {
    code: &'static str,
    message: String,
}

impl ExecutionPlannerViolation {
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

/// Execution-planner boundary errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionPlannerError {
    /// Validation failed with deterministic violations.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<ExecutionPlannerViolation>,
    },
}

impl ExecutionPlannerError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[ExecutionPlannerViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
        }
    }
}

impl fmt::Display for ExecutionPlannerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "execution planner validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ExecutionPlannerError {}

impl fmt::Display for OpportunityCandidateTraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(
                    formatter,
                    "opportunity candidate trace validation failed with {} violation(s)",
                    violations.len()
                )
            }
            Self::AuditJournalFailed { reason } => {
                write!(
                    formatter,
                    "opportunity candidate trace audit failed: {reason}"
                )
            }
            Self::StateStoreFailed { reason } => {
                write!(
                    formatter,
                    "opportunity candidate trace state failed: {reason}"
                )
            }
        }
    }
}

impl Error for OpportunityCandidateTraceError {}

#[cfg(test)]
mod tests {
    use super::{
        append_execution_plan_draft_audit, persist_execution_plan_draft_checkpoint,
        persist_opportunity_candidate_trace, validate_opportunity_candidate_trace_restart_recovery,
        validate_opportunity_planner_handoff, validate_opportunity_planner_handoff_with_trace,
        validate_strategy_profile_replay_corpus, validate_strategy_profitability_tuning,
        DeterministicExecutionPlanner, ExecutionPlanFailureMode, ExecutionPlanStatus,
        ExecutionPlanStepAction, ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerRequest,
        OpportunityCandidateTraceRecord, OpportunityPlannerHandoffStatus,
        StrategyProfileReplayValidationStatus, StrategyProfitabilityTuningValidationStatus,
        EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY, EXECUTION_PLANNER_STATE_SUBSYSTEM,
        OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM,
    };
    use crate::{
        phase27_local_opportunity_historical_fixture_corpus, AgentConfig, AppendOnlyAuditJournal,
        FeeAdjustedEdge, FeeEstimate, FeeSchedule, LiquidityRole, MarketPair, NormalizedQuote,
        OpportunityCandidate, OpportunityDiscoveryConfig, OpportunityDiscoveryRequest,
        OpportunityHistoricalFixtureCorpus, OpportunityLeg, OpportunityLegSide,
        OpportunityReplayCorpus, OpportunityReplayExpectation, OpportunityReplayScenario,
        OpportunityRouteKind, OpportunityScore, PolicyEngine, PriceLevel, SqliteWalStateStore,
        StateStore, StrategyPolicyConstraintStatus, StrategyProfile, VenueKind, VenueRef,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process,
    };

    const BASE_CONFIG: &str = r#"
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

    const PHASE27_HANDOFF_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = false

[risk]
max_single_trade_quote = 1_000_000.0
max_daily_loss_quote = 100_000.0
max_open_exposure_quote = 2_000_000.0
slippage_bps = 100
gas_fee_cap_quote = 1_000.0

[venues]
cex_allowlist = ["paper-a", "paper-b", "paper-c", "paper-d"]
dex_allowlist = ["paper-dex-a", "paper-dex-b", "paper-aggregator-b"]
chain_allowlist = ["ethereum"]
asset_allowlist = ["BTC", "ETH", "SOL", "AVAX", "MATIC", "ATOM", "LINK", "ADA", "USD"]

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
    fn planner_rejects_live_scope() {
        let config = ExecutionPlannerConfig {
            requested_scope: crate::ExecutionScope::Live,
            ..ExecutionPlannerConfig::default()
        };

        let error = config.validate().expect_err("live scope must be denied");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "PLANNER_LIVE_SCOPE_DENIED"));
    }

    #[test]
    fn planner_creates_policy_evaluated_draft() {
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(BASE_CONFIG).expect("config should validate"),
        );
        let request = ExecutionPlannerRequest {
            id: "planner-request-1".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };

        let plan = DeterministicExecutionPlanner::new()
            .plan(&request, &policy)
            .expect("plan should be created");
        assert_eq!(plan.status, ExecutionPlanStatus::DraftReady);
        assert!(!plan.adapter_submission_enabled);
        assert_eq!(plan.intents.len(), 2);
        assert!(plan
            .policy_outcomes
            .iter()
            .all(|outcome| outcome.is_approved()));
    }

    #[test]
    fn execution_plan_draft_audits_plan_and_policy_outcomes_locally() {
        let audit_path = unique_audit_path("plan-draft-audit");
        let draft = plan();

        {
            let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
            let report = append_execution_plan_draft_audit(&mut journal, &draft, 10_100)
                .expect("plan draft audit should append");

            assert_eq!(report.plan_id, draft.id);
            assert_eq!(report.draft_record_sequence, 1);
            assert_eq!(report.policy_outcome_record_sequences, vec![2, 3]);
            assert_eq!(report.records_appended, 3);
            assert!(!report.external_submission_performed);
            assert!(!report.live_execution_performed);
            assert_eq!(report.audited_at_unix_ms, 10_100);
            assert_eq!(journal.next_sequence(), 4);
        }

        {
            let reopened = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
            assert_eq!(reopened.next_sequence(), 4);
            let lines = fs::read_to_string(&audit_path).expect("journal should be readable");
            assert!(lines.contains("\"kind\":\"execution-planning\""));
            assert!(lines.contains("\"kind\":\"policy-decision\""));
            assert!(lines.contains(&format!("\"id\":\"execution-plan-draft-{}\"", draft.id)));
            assert!(lines.contains(&draft.intents[0].id));
            assert!(lines.contains(&draft.intents[1].id));
        }

        cleanup_audit_files(&audit_path);
    }

    #[test]
    fn planner_applies_strategy_profile_constraints_before_adapter_boundary() {
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(BASE_CONFIG).expect("config should validate"),
        );
        let request = ExecutionPlannerRequest {
            id: "planner-request-strategy".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };
        let profile = strategy_profile();

        let report = DeterministicExecutionPlanner::new()
            .plan_with_strategy_profile(&request, &policy, &profile)
            .expect("strategy-constrained plan should be created");

        assert_eq!(report.draft.status, ExecutionPlanStatus::DraftReady);
        assert_eq!(
            report.strategy_constraint_reports.len(),
            report.draft.intents.len()
        );
        assert_eq!(report.strategy_rejected_intents, 0);
        assert!(report
            .strategy_constraint_reports
            .iter()
            .all(|strategy_report| {
                strategy_report.status == StrategyPolicyConstraintStatus::Satisfied
                    && !strategy_report.execution_performed
                    && !strategy_report.signing_or_broadcast_performed
                    && !strategy_report.live_network_used
            }));
        assert!(!report.adapter_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn planner_fails_closed_when_strategy_profile_rejects_generated_intents() {
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(BASE_CONFIG).expect("config should validate"),
        );
        let request = ExecutionPlannerRequest {
            id: "planner-request-strategy-rejected".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };
        let mut profile = strategy_profile();
        profile.opportunity.min_net_profit_abs = 1_000.0;

        let report = DeterministicExecutionPlanner::new()
            .plan_with_strategy_profile(&request, &policy, &profile)
            .expect("strategy rejection should produce a denied draft report");

        assert_eq!(report.draft.status, ExecutionPlanStatus::PolicyDeniedDraft);
        assert_eq!(report.strategy_rejected_intents, report.draft.intents.len());
        assert!(report
            .strategy_constraint_reports
            .iter()
            .all(|strategy_report| strategy_report.status
                == StrategyPolicyConstraintStatus::Rejected));
        assert!(report.draft.warnings.iter().any(|warning| {
            warning.contains("draft intent(s) rejected by strategy profile constraints")
        }));
        assert!(!report.adapter_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn planner_draft_rejects_duplicate_intent_and_policy_outcome_ids() {
        let mut plan = plan();
        plan.intents[1].id = plan.intents[0].id.clone();
        plan.policy_outcomes[1].intent_id = plan.policy_outcomes[0].intent_id.clone();

        let error = plan
            .validate()
            .expect_err("duplicate draft identifiers must be rejected");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "PLAN_DUPLICATE_INTENT_ID"));
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "PLAN_DUPLICATE_POLICY_OUTCOME_INTENT_ID" }));
    }

    #[test]
    fn planner_assigns_route_specific_failure_modes_to_draft_steps() {
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(BASE_CONFIG).expect("config should validate"),
        );
        let cex_request = ExecutionPlannerRequest {
            id: "planner-request-cex-failure-modes".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };
        let cex_plan = DeterministicExecutionPlanner::new()
            .plan(&cex_request, &policy)
            .expect("cex plan should be created");

        assert!(cex_plan
            .failure_modes
            .contains(&ExecutionPlanFailureMode::CancelUnfilledRemainder));
        assert!(!cex_plan
            .failure_modes
            .contains(&ExecutionPlanFailureMode::DoNotSignOrBroadcast));
        assert!(cex_plan.steps.iter().any(|step| {
            step.action == ExecutionPlanStepAction::PrepareCexOrderDraft
                && step.failure_mode == ExecutionPlanFailureMode::CancelUnfilledRemainder
        }));
        assert_eq!(
            cex_plan.steps.last().map(|step| step.failure_mode),
            Some(ExecutionPlanFailureMode::ManualReviewRequired)
        );

        let dex_request = ExecutionPlannerRequest {
            id: "planner-request-dex-failure-modes".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: dex_candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_001,
        };
        let dex_plan = DeterministicExecutionPlanner::new()
            .plan(&dex_request, &policy)
            .expect("dex plan should be created");

        assert!(dex_plan
            .failure_modes
            .contains(&ExecutionPlanFailureMode::DoNotSignOrBroadcast));
        assert!(dex_plan.steps.iter().any(|step| {
            step.action == ExecutionPlanStepAction::PrepareDexSwapDraft
                && step.failure_mode == ExecutionPlanFailureMode::DoNotSignOrBroadcast
        }));
    }

    #[test]
    fn phase27_replay_candidates_handoff_to_draft_planner_without_submission() {
        let corpus = phase27_local_opportunity_historical_fixture_corpus()
            .expect("phase 27 local corpus should build");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PHASE27_HANDOFF_CONFIG).expect("config should validate"),
        );

        let report = validate_opportunity_planner_handoff(&corpus, &policy)
            .expect("planner handoff validation should run");

        assert_eq!(report.status, OpportunityPlannerHandoffStatus::Passed);
        assert_eq!(report.replay_window_count, 2);
        assert_eq!(report.replay_scenario_count, 13);
        assert_eq!(report.skipped_discovery_failures, 2);
        assert_eq!(report.discovered_candidates, 12);
        assert_eq!(report.planned_candidates, 12);
        assert_eq!(report.failed_planner_handoffs, 0);
        assert_eq!(report.candidate_trace_audit_records, 0);
        assert_eq!(report.candidate_trace_checkpoints, 0);
        assert_eq!(report.policy_denied_plans, 0);
        assert_eq!(report.draft_ready_plans, 12);
        assert!(report.total_intents > report.planned_candidates);
        assert!(!report.adapter_submission_enabled);
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);
    }

    #[test]
    fn phase27_strategy_profiles_replay_against_historical_fixture_corpus() {
        let corpus = phase27_local_opportunity_historical_fixture_corpus()
            .expect("phase 27 local corpus should build");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PHASE27_HANDOFF_CONFIG).expect("config should validate"),
        );
        let mut accepted_profile =
            StrategyProfile::conservative_paper("phase27-local-replay-profile-accepted", "USD");
        accepted_profile.capital.max_total_deployed = 1_000_000.0;
        accepted_profile.capital.max_per_opportunity = 1_000_000.0;
        accepted_profile.capital.reserve_minimum = 0.0;
        accepted_profile.risk.max_single_tx_value = 1_000_000.0;
        accepted_profile.opportunity.min_net_profit_abs = 0.0;
        accepted_profile.execution.max_slippage_bps = u16::MAX;

        let mut rejected_profile = accepted_profile.clone();
        rejected_profile.id = "phase27-local-replay-profile-rejected".to_owned();
        rejected_profile.opportunity.min_net_profit_abs = 1_000_000.0;

        let report = validate_strategy_profile_replay_corpus(
            &corpus,
            &policy,
            &accepted_profile,
            &rejected_profile,
        )
        .expect("strategy replay validation should run");

        assert_eq!(report.status, StrategyProfileReplayValidationStatus::Passed);
        assert_eq!(report.replay_window_count, 2);
        assert_eq!(report.replay_scenario_count, 13);
        assert_eq!(report.skipped_discovery_failures, 2);
        assert_eq!(report.discovered_candidates, 12);
        assert_eq!(report.accepted_planned_candidates, 12);
        assert_eq!(report.accepted_draft_ready_plans, 12);
        assert_eq!(report.accepted_strategy_rejected_intents, 0);
        assert_eq!(
            report.accepted_satisfied_constraint_reports,
            report.total_accepted_intents
        );
        assert_eq!(report.rejected_planned_candidates, 12);
        assert_eq!(report.rejected_policy_denied_plans, 12);
        assert_eq!(
            report.rejected_constraint_reports,
            report.total_rejected_intents
        );
        assert_eq!(
            report.rejected_strategy_rejected_intents,
            report.total_rejected_intents
        );
        assert_eq!(report.failed_strategy_planner_runs, 0);
        assert!(!report.adapter_submission_performed);
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn phase27_strategy_profitability_tuning_is_monotonic_across_local_corpus() {
        let corpus = phase27_local_opportunity_historical_fixture_corpus()
            .expect("phase 27 local corpus should build");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PHASE27_HANDOFF_CONFIG).expect("config should validate"),
        );

        let report = validate_strategy_profitability_tuning(&corpus, &policy)
            .expect("strategy profitability tuning should run");

        assert_eq!(
            report.status,
            StrategyProfitabilityTuningValidationStatus::Passed
        );
        assert_eq!(report.replay_window_count, 2);
        assert_eq!(report.replay_scenario_count, 13);
        assert_eq!(report.skipped_discovery_failures, 2);
        assert_eq!(report.discovered_candidates, 12);
        assert_eq!(report.profitability_points.len(), 3);
        assert!(report.monotonic_acceptance_validated);
        assert!(report.monotonic_rejection_validated);
        assert!(report.threshold_transition_observed);
        assert_eq!(
            report
                .profitability_points
                .first()
                .map(|point| point.draft_ready_plans),
            Some(report.discovered_candidates)
        );
        assert_eq!(
            report
                .profitability_points
                .last()
                .map(|point| point.policy_denied_plans),
            Some(report.discovered_candidates)
        );
        assert_eq!(
            report
                .profitability_points
                .last()
                .map(|point| point.accepted_intents),
            Some(0)
        );
        assert!(!report.adapter_submission_performed);
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn opportunity_candidate_trace_persists_audit_then_state() {
        let audit_path = unique_audit_path("candidate-trace");
        let state_path = unique_state_path("candidate-trace");
        let candidate = candidate();
        let trace_key;

        {
            let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
            let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
            let persisted = persist_opportunity_candidate_trace(
                &mut journal,
                &mut store,
                &candidate,
                "strategy-basic-arb",
                "planner-request-1",
                10_000,
            )
            .expect("candidate trace should persist");

            assert_eq!(persisted.audit_record.sequence, 1);
            assert_eq!(
                persisted.checkpoint.subsystem,
                OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM
            );
            assert_eq!(persisted.trace.candidate, candidate);
            assert!(!persisted.trace.adapter_submission_enabled);
            assert!(!persisted.trace.external_calls_performed);
            assert!(!persisted.trace.live_execution_performed);
            trace_key = persisted.checkpoint.key.clone();
        }

        {
            let journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
            assert_eq!(journal.next_sequence(), 2);
            let store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
            let checkpoint = store
                .get_checkpoint(&trace_key)
                .expect("checkpoint should read")
                .expect("trace checkpoint should exist");
            let restored: OpportunityCandidateTraceRecord =
                serde_json::from_str(&checkpoint.value).expect("trace json should parse");
            assert_eq!(restored.candidate, candidate);
            assert_eq!(restored.audit_sequence, 1);
        }

        cleanup_audit_files(&audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn phase27_traced_replay_candidates_persist_before_draft_planner_handoff() {
        let audit_path = unique_audit_path("phase27-traced-handoff");
        let state_path = unique_state_path("phase27-traced-handoff");
        let corpus = phase27_local_opportunity_historical_fixture_corpus()
            .expect("phase 27 local corpus should build");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PHASE27_HANDOFF_CONFIG).expect("config should validate"),
        );

        {
            let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
            let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
            let report = validate_opportunity_planner_handoff_with_trace(
                &corpus,
                &policy,
                &mut journal,
                &mut store,
            )
            .expect("traced planner handoff should run");

            assert_eq!(report.status, OpportunityPlannerHandoffStatus::Passed);
            assert_eq!(report.discovered_candidates, 12);
            assert_eq!(report.planned_candidates, 12);
            assert_eq!(report.candidate_trace_audit_records, 12);
            assert_eq!(report.candidate_trace_checkpoints, 12);
            assert!(!report.adapter_submission_enabled);
            assert!(!report.external_calls_performed);
            assert!(!report.live_execution_performed);
        }

        {
            let journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
            assert_eq!(journal.next_sequence(), 13);
            let store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
            let checkpoint = store
                .get_checkpoint(
                    "opportunity-candidate-trace:opp:cex-cex:BTC/USD:paper-a:paper-b:phase27-cex-spread-request:phase27-planner-handoff-1",
                )
                .expect("checkpoint should read")
                .expect("first trace checkpoint should exist");
            let restored: OpportunityCandidateTraceRecord =
                serde_json::from_str(&checkpoint.value).expect("trace json should parse");
            assert_eq!(restored.planner_request_id, "phase27-planner-handoff-1");
            assert_eq!(restored.audit_sequence, 1);
        }

        cleanup_audit_files(&audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn phase27_candidate_trace_restart_recovery_reopens_audit_and_state() {
        let audit_path = unique_audit_path("phase27-trace-recovery");
        let state_path = unique_state_path("phase27-trace-recovery");
        let corpus = phase27_local_opportunity_historical_fixture_corpus()
            .expect("phase 27 local corpus should build");
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PHASE27_HANDOFF_CONFIG).expect("config should validate"),
        );

        let report = validate_opportunity_candidate_trace_restart_recovery(
            &corpus,
            &policy,
            &audit_path,
            &state_path,
        )
        .expect("candidate trace recovery should validate");

        assert_eq!(report.status, OpportunityPlannerHandoffStatus::Passed);
        assert!(report.trace_recovery_validated);
        assert_eq!(report.handoff_report.discovered_candidates, 12);
        assert_eq!(report.audit_replay_records, 12);
        assert_eq!(report.recovered_trace_checkpoints, 12);
        assert_eq!(report.recovered_trace_summaries.len(), 12);
        assert_eq!(
            report.recovered_trace_summaries[0].planner_request_id,
            "phase27-planner-handoff-1"
        );
        assert_eq!(report.recovered_trace_summaries[0].audit_sequence, 1);
        assert!(report.recovered_trace_summaries[0].leg_count > 0);
        assert!(report.missing_trace_checkpoints.is_empty());
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);

        cleanup_audit_files(&audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn traced_planner_handoff_deduplicates_duplicate_candidate_ids_before_persistence() {
        let audit_path = unique_audit_path("phase27-dedup-handoff");
        let state_path = unique_state_path("phase27-dedup-handoff");
        let corpus = duplicate_candidate_fixture_corpus();
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(PHASE27_HANDOFF_CONFIG).expect("config should validate"),
        );

        {
            let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
            let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
            let report = validate_opportunity_planner_handoff_with_trace(
                &corpus,
                &policy,
                &mut journal,
                &mut store,
            )
            .expect("deduplicated traced planner handoff should run");

            assert_eq!(report.status, OpportunityPlannerHandoffStatus::Passed);
            assert_eq!(report.replay_window_count, 1);
            assert_eq!(report.replay_scenario_count, 1);
            assert_eq!(report.discovered_candidates, 1);
            assert_eq!(report.planned_candidates, 1);
            assert_eq!(report.candidate_trace_audit_records, 1);
            assert_eq!(report.candidate_trace_checkpoints, 1);
            assert_eq!(report.draft_ready_plans, 1);
            assert_eq!(journal.next_sequence(), 2);
        }

        {
            let reopened_journal =
                AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
            assert_eq!(reopened_journal.next_sequence(), 2);
            let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
            assert!(reopened_store
                .get_checkpoint(
                    "opportunity-candidate-trace:opp:cex-cex:BTC/USD:paper-a:paper-b:dedup-request:phase27-planner-handoff-1",
                )
                .expect("checkpoint should read")
                .is_some());
        }

        cleanup_audit_files(&audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn planner_draft_persists_as_state_checkpoint() {
        let plan = plan();
        let mut store = crate::InMemoryStateStore::new();

        let checkpoint = persist_execution_plan_draft_checkpoint(&mut store, &plan)
            .expect("planner draft checkpoint should persist");

        assert_eq!(checkpoint.key, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY);
        assert_eq!(checkpoint.subsystem, EXECUTION_PLANNER_STATE_SUBSYSTEM);
        assert_eq!(checkpoint.updated_at_unix_ms, plan.created_at_unix_ms);
        let restored: super::ExecutionPlanDraft =
            serde_json::from_str(&checkpoint.value).expect("checkpoint json should parse");
        assert_eq!(restored, plan);
        assert_eq!(
            store
                .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
                .expect("checkpoint should read"),
            Some(checkpoint)
        );
    }

    #[test]
    fn planner_draft_persists_through_sqlite_wal_store() {
        let path = unique_state_path("planner-draft");
        let plan = plan();

        {
            let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");
            persist_execution_plan_draft_checkpoint(&mut store, &plan)
                .expect("planner draft checkpoint should persist");
        }

        {
            let store = SqliteWalStateStore::open(&path).expect("sqlite store reopens");
            let checkpoint = store
                .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
                .expect("checkpoint should read")
                .expect("checkpoint should exist");
            let restored: super::ExecutionPlanDraft =
                serde_json::from_str(&checkpoint.value).expect("checkpoint json should parse");
            assert_eq!(checkpoint.subsystem, EXECUTION_PLANNER_STATE_SUBSYSTEM);
            assert_eq!(checkpoint.updated_at_unix_ms, plan.created_at_unix_ms);
            assert_eq!(restored, plan);
        }

        cleanup_state_files(&path);
    }

    fn plan() -> super::ExecutionPlanDraft {
        let policy = PolicyEngine::from_config(
            AgentConfig::from_toml_str(BASE_CONFIG).expect("config should validate"),
        );
        let request = ExecutionPlannerRequest {
            id: "planner-request-1".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };

        DeterministicExecutionPlanner::new()
            .plan(&request, &policy)
            .expect("plan should be created")
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

    fn dex_candidate() -> OpportunityCandidate {
        let mut candidate = candidate();
        candidate.id = "opp-dex-dex-btc-usd".to_owned();
        candidate.route_kind = OpportunityRouteKind::DexDex;
        candidate.legs[0].venue = VenueRef {
            name: "paper-dex-a".to_owned(),
            kind: VenueKind::Dex,
        };
        candidate.legs[0].fee_estimate.venue = candidate.legs[0].venue.clone();
        candidate.legs[0].source_quote_id = "quote-paper-dex-a".to_owned();
        candidate.legs[1].venue = VenueRef {
            name: "paper-dex-b".to_owned(),
            kind: VenueKind::Dex,
        };
        candidate.legs[1].fee_estimate.venue = candidate.legs[1].venue.clone();
        candidate.legs[1].source_quote_id = "quote-paper-dex-b".to_owned();
        candidate.source_quote_ids = vec![
            "quote-paper-dex-a".to_owned(),
            "quote-paper-dex-b".to_owned(),
        ];
        candidate
    }

    fn strategy_profile() -> StrategyProfile {
        let mut profile = StrategyProfile::conservative_paper("strategy-basic-arb", "USD");
        profile.venues.allowed_exchanges = vec!["paper-a".to_owned(), "paper-b".to_owned()];
        profile.venues.allowed_assets = vec!["BTC".to_owned(), "USD".to_owned()];
        profile
    }

    fn duplicate_candidate_fixture_corpus() -> OpportunityHistoricalFixtureCorpus {
        let pair = MarketPair::new("BTC", "USD").expect("pair should validate");
        OpportunityHistoricalFixtureCorpus {
            id: "planner-dedup-fixture".to_owned(),
            historical_fixture_replay: true,
            replay_windows: vec![OpportunityReplayCorpus {
                id: "planner-dedup-window-1".to_owned(),
                scenarios: vec![OpportunityReplayScenario {
                    id: "planner-dedup-scenario-1".to_owned(),
                    request: OpportunityDiscoveryRequest {
                        id: "dedup-request".to_owned(),
                        quotes: vec![
                            planner_quote("buy-a-1", "paper-a", pair.clone(), 98.0, 99.0, 1.0),
                            planner_quote("buy-a-2", "paper-a", pair.clone(), 97.0, 98.0, 1.0),
                            planner_quote("sell-b", "paper-b", pair.clone(), 110.0, 111.0, 1.0),
                        ],
                        fee_schedules: vec![
                            planner_fee("paper-a", pair.clone()),
                            planner_fee("paper-b", pair),
                        ],
                        order_books: Vec::new(),
                        inventory_limits: Vec::new(),
                        transfer_risk_profiles: Vec::new(),
                        config: OpportunityDiscoveryConfig::default(),
                        now_unix_ms: 10_000,
                    },
                    expectation: OpportunityReplayExpectation {
                        min_candidates: 1,
                        max_candidates: Some(1),
                        required_route_kinds: vec![OpportunityRouteKind::CexCex],
                        forbidden_route_kinds: Vec::new(),
                        min_best_net_profit_quote: None,
                        expected_violation_codes: Vec::new(),
                    },
                }],
            }],
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

    fn planner_quote(
        id: &str,
        venue_name: &str,
        pair: MarketPair,
        bid_price: f64,
        ask_price: f64,
        quantity_base: f64,
    ) -> NormalizedQuote {
        NormalizedQuote {
            id: id.to_owned(),
            venue: VenueRef {
                name: venue_name.to_owned(),
                kind: VenueKind::Cex,
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
            captured_at_unix_ms: 9_500,
            received_at_unix_ms: 9_500,
        }
    }

    fn planner_fee(venue_name: &str, pair: MarketPair) -> FeeSchedule {
        FeeSchedule {
            venue: VenueRef {
                name: venue_name.to_owned(),
                kind: VenueKind::Cex,
            },
            pair: Some(pair),
            maker_bps: 5.0,
            taker_bps: 10.0,
            network_fee_quote: 0.0,
            externally_verified: false,
        }
    }

    fn unique_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-planner-{label}-{}-{nanos}.sqlite3",
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
            "arbyclaw-planner-{label}-{}-{nanos}.audit.jsonl",
            process::id()
        ));
        path
    }

    fn cleanup_state_files(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", path.display())));
        }
    }

    fn cleanup_audit_files(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(PathBuf::from(format!("{}.lock", path.display())));
    }
}
