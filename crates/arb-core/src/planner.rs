#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope, OpportunityCandidate,
    OpportunityError, OpportunityLeg, OpportunityLegSide, PolicyDecision, PolicyEngine,
    StateCheckpoint, StateStore, StateStoreError, VenueKind, DEFAULT_MAX_MARKET_DATA_AGE_MS,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable planner version for audit, replay, and handoff surfaces.
pub const EXECUTION_PLANNER_VERSION: &str = "phase-10-execution-planner-v1";

/// State-store subsystem name for execution-planner checkpoints.
pub const EXECUTION_PLANNER_STATE_SUBSYSTEM: &str = "execution-planner";

/// State-store key for the latest deterministic execution-plan draft.
pub const EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY: &str = "execution-planner:last-draft";

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

        for intent in &self.intents {
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

#[cfg(test)]
mod tests {
    use super::{
        persist_execution_plan_draft_checkpoint, DeterministicExecutionPlanner,
        ExecutionPlanStatus, ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerRequest,
        EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY, EXECUTION_PLANNER_STATE_SUBSYSTEM,
    };
    use crate::{
        AgentConfig, FeeAdjustedEdge, FeeEstimate, LiquidityRole, MarketPair, OpportunityCandidate,
        OpportunityLeg, OpportunityLegSide, OpportunityRouteKind, OpportunityScore, PolicyEngine,
        SqliteWalStateStore, StateStore, VenueKind, VenueRef,
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

    fn cleanup_state_files(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", path.display())));
        }
    }
}
