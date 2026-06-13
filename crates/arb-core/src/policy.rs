#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    AgentConfig, AppendOnlyAuditJournal, ApprovedDestinationEntry, AuditError, AuditEvent,
    AuditEventKind, AuditRecord, AuditValue, RuntimeMode, SecretBackend, StateCheckpoint,
    StateStore, StateStoreError, LIVE_ACKNOWLEDGEMENT,
};
use serde::{Deserialize, Serialize};

/// Phase 3 trust-contract identifier.
pub const TRUST_CONTRACT_VERSION: &str = "phase-3-deny-by-default-v1";

/// Default quote freshness ceiling for policy approval.
pub const DEFAULT_MAX_MARKET_DATA_AGE_MS: u64 = 5_000;

/// Owning subsystem label for local policy decision checkpoints.
pub const POLICY_STATE_SUBSYSTEM: &str = "policy";

/// Stable checkpoint key for the latest local policy decision summary.
pub const POLICY_LAST_DECISION_CHECKPOINT_KEY: &str = "policy:last-decision";

/// Deterministic policy engine.
///
/// The engine is intentionally deny-by-default. A later execution adapter must
/// call this engine before any order, swap, transfer, withdrawal, or signing
/// operation. Passing policy still does not execute anything by itself.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEngine {
    config: AgentConfig,
    context: PolicyContext,
}

impl PolicyEngine {
    /// Create a policy engine from validated runtime config and dynamic context.
    #[must_use]
    pub fn new(config: AgentConfig, context: PolicyContext) -> Self {
        Self { config, context }
    }

    /// Create a policy engine with the safest default context.
    #[must_use]
    pub fn from_config(config: AgentConfig) -> Self {
        Self::new(config, PolicyContext::default())
    }

    /// Return the active trust-contract version.
    #[must_use]
    pub const fn trust_contract_version(&self) -> &'static str {
        TRUST_CONTRACT_VERSION
    }

    /// Evaluate one proposed intent and return a deterministic decision.
    #[must_use]
    pub fn evaluate(&self, intent: &ExecutionIntent) -> PolicyDecision {
        let mut violations = Vec::new();

        Self::evaluate_identity(intent, &mut violations);
        self.evaluate_runtime(intent, &mut violations);
        self.evaluate_trust_contract(intent, &mut violations);

        if intent.kind.requires_funds_movement() {
            self.evaluate_audit(&mut violations);
            self.evaluate_venue(intent, &mut violations);
            self.evaluate_assets(intent, &mut violations);
            self.evaluate_chain(intent, &mut violations);
            self.evaluate_risk(intent, &mut violations);
            self.evaluate_freshness(intent, &mut violations);
            self.evaluate_destination(intent, &mut violations);
            self.evaluate_signing(intent, &mut violations);
        }

        if violations.is_empty() {
            PolicyDecision::Approved {
                approval: PolicyApproval {
                    trust_contract_version: TRUST_CONTRACT_VERSION,
                    intent_id: intent.id.clone(),
                    approved_scope: intent.scope,
                },
            }
        } else {
            PolicyDecision::Denied { violations }
        }
    }

    fn evaluate_identity(intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        if intent.id.trim().is_empty() {
            violations.push(PolicyViolation::new(
                "INTENT_ID_REQUIRED",
                "execution intent must have a stable non-empty id",
            ));
        }

        if intent.strategy_id.trim().is_empty() {
            violations.push(PolicyViolation::new(
                "STRATEGY_ID_REQUIRED",
                "execution intent must identify the strategy profile that produced it",
            ));
        }
    }

    fn evaluate_runtime(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        match self.config.runtime.mode {
            RuntimeMode::Observe => {
                if intent.scope != ExecutionScope::Observe || intent.kind.requires_funds_movement()
                {
                    violations.push(PolicyViolation::new(
                        "OBSERVE_MODE_DENIES_EXECUTION",
                        "observe mode may only approve non-executing observation intents",
                    ));
                }
            }
            RuntimeMode::Paper => {
                if intent.scope == ExecutionScope::Live {
                    violations.push(PolicyViolation::new(
                        "PAPER_MODE_DENIES_LIVE_SCOPE",
                        "paper mode cannot approve live execution intents",
                    ));
                }
            }
            RuntimeMode::LiveArmed => {
                if intent.scope == ExecutionScope::Live {
                    if !self.config.runtime.live_execution_enabled {
                        violations.push(PolicyViolation::new(
                            "LIVE_EXECUTION_SWITCH_DISABLED",
                            "live scope requires runtime.live_execution_enabled to be true",
                        ));
                    }

                    if self.config.runtime.operator_acknowledgement.as_deref()
                        != Some(LIVE_ACKNOWLEDGEMENT)
                    {
                        violations.push(PolicyViolation::new(
                            "LIVE_ACKNOWLEDGEMENT_MISSING",
                            "live scope requires the explicit operator risk acknowledgement",
                        ));
                    }

                    if !self.context.live_execution_runtime_available {
                        violations.push(PolicyViolation::new(
                            "LIVE_RUNTIME_NOT_AVAILABLE",
                            "Phase 3 policy denies live approval until later execution, audit, custody, and validation phases enable the runtime",
                        ));
                    }
                }
            }
        }

        if intent.scope == ExecutionScope::Observe && intent.kind.requires_funds_movement() {
            violations.push(PolicyViolation::new(
                "OBSERVE_SCOPE_CANNOT_MOVE_FUNDS",
                "observe-scoped intents cannot place orders, sign transactions, or move funds",
            ));
        }
    }

    fn evaluate_trust_contract(
        &self,
        intent: &ExecutionIntent,
        violations: &mut Vec<PolicyViolation>,
    ) {
        if self.context.kill_switch_engaged && intent.kind.requires_funds_movement() {
            violations.push(PolicyViolation::new(
                "KILL_SWITCH_ENGAGED",
                "kill switch is engaged; funds movement is denied",
            ));
        }

        if self.config.runtime.allow_withdrawals || intent.kind == ExecutionIntentKind::Withdrawal {
            violations.push(PolicyViolation::new(
                "WITHDRAWALS_DENIED_BY_TRUST_CONTRACT",
                "withdrawals are denied in Phase 3 regardless of strategy or runtime mode",
            ));
        }

        if intent.kind == ExecutionIntentKind::BridgeRoute {
            violations.push(PolicyViolation::new(
                "BRIDGE_ROUTES_DENIED_IN_PHASE_3",
                "cross-chain bridge routes require elevated custody, simulation, and rollback validation before approval",
            ));
        }
    }

    fn evaluate_audit(&self, violations: &mut Vec<PolicyViolation>) {
        if !self.config.audit.enabled || self.config.audit.redact_secrets != Some(true) {
            violations.push(PolicyViolation::new(
                "AUDIT_REDACTION_REQUIRED",
                "executable intents require enabled audit logging with secret redaction",
            ));
        }
    }

    fn evaluate_venue(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        if intent.venue.name.trim().is_empty() {
            violations.push(PolicyViolation::new(
                "VENUE_REQUIRED",
                "execution intent must include a non-empty venue name",
            ));
            return;
        }

        let allowed = match intent.venue.kind {
            VenueKind::Cex => contains_ignore_ascii(
                &self.config.venues.cex_allowlist,
                intent.venue.name.as_str(),
            ),
            VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge => contains_ignore_ascii(
                &self.config.venues.dex_allowlist,
                intent.venue.name.as_str(),
            ),
        };

        if !allowed {
            violations.push(PolicyViolation::new_owned(
                "VENUE_NOT_ALLOWLISTED",
                format!(
                    "venue {} is not present in the configured allowlist",
                    intent.venue.name
                ),
            ));
        }
    }

    fn evaluate_assets(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        for (label, asset) in [("base", &intent.base_asset), ("quote", &intent.quote_asset)] {
            if asset.trim().is_empty() {
                violations.push(PolicyViolation::new_owned(
                    "ASSET_REQUIRED",
                    format!("{label} asset must be non-empty"),
                ));
            } else if !contains_ignore_ascii(&self.config.venues.asset_allowlist, asset) {
                violations.push(PolicyViolation::new_owned(
                    "ASSET_NOT_ALLOWLISTED",
                    format!("{label} asset {asset} is not present in the configured allowlist"),
                ));
            }
        }
    }

    fn evaluate_chain(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        let chain_required = matches!(
            intent.venue.kind,
            VenueKind::Dex | VenueKind::Aggregator | VenueKind::Bridge
        );

        match intent.chain.as_deref() {
            Some(chain) => {
                if chain.trim().is_empty() {
                    violations.push(PolicyViolation::new(
                        "CHAIN_REFERENCE_EMPTY",
                        "chain reference cannot be empty when provided",
                    ));
                } else if !self.config.venues.chain_allowlist.is_empty()
                    && !contains_ignore_ascii(&self.config.venues.chain_allowlist, chain)
                {
                    violations.push(PolicyViolation::new_owned(
                        "CHAIN_NOT_ALLOWLISTED",
                        format!("chain {chain} is not present in the configured chain allowlist"),
                    ));
                }
            }
            None if chain_required => violations.push(PolicyViolation::new(
                "CHAIN_REQUIRED_FOR_WEB3_INTENT",
                "DEX, aggregator, and bridge intents require an explicit chain reference",
            )),
            None => {}
        }
    }

    fn evaluate_risk(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        if !is_positive_finite(intent.notional_quote) {
            violations.push(PolicyViolation::new(
                "NOTIONAL_MUST_BE_POSITIVE",
                "intent notional must be a positive finite quote amount",
            ));
        } else {
            if intent.notional_quote > self.config.risk.max_single_trade_quote {
                violations.push(PolicyViolation::new_owned(
                    "MAX_SINGLE_TRADE_EXCEEDED",
                    format!(
                        "intent notional {} exceeds max_single_trade_quote {}",
                        intent.notional_quote, self.config.risk.max_single_trade_quote
                    ),
                ));
            }

            if intent.notional_quote > self.config.risk.max_open_exposure_quote {
                violations.push(PolicyViolation::new_owned(
                    "MAX_OPEN_EXPOSURE_EXCEEDED",
                    format!(
                        "intent notional {} exceeds max_open_exposure_quote {}",
                        intent.notional_quote, self.config.risk.max_open_exposure_quote
                    ),
                ));
            }
        }

        if !is_non_negative_finite(intent.max_loss_quote) {
            violations.push(PolicyViolation::new(
                "MAX_LOSS_INVALID",
                "intent max loss must be finite and non-negative",
            ));
        } else if intent.max_loss_quote > self.config.risk.max_daily_loss_quote {
            violations.push(PolicyViolation::new_owned(
                "MAX_DAILY_LOSS_EXCEEDED",
                format!(
                    "intent max loss {} exceeds max_daily_loss_quote {}",
                    intent.max_loss_quote, self.config.risk.max_daily_loss_quote
                ),
            ));
        }

        if intent.slippage_bps > self.config.risk.slippage_bps {
            violations.push(PolicyViolation::new_owned(
                "SLIPPAGE_LIMIT_EXCEEDED",
                format!(
                    "intent slippage {} bps exceeds configured cap {} bps",
                    intent.slippage_bps, self.config.risk.slippage_bps
                ),
            ));
        }

        if !is_non_negative_finite(intent.estimated_fee_quote) {
            violations.push(PolicyViolation::new(
                "FEE_ESTIMATE_INVALID",
                "estimated venue fees must be finite and non-negative",
            ));
        }

        if !is_non_negative_finite(intent.gas_fee_quote) {
            violations.push(PolicyViolation::new(
                "GAS_FEE_INVALID",
                "estimated gas/network fees must be finite and non-negative",
            ));
        } else if intent.gas_fee_quote > self.config.risk.gas_fee_cap_quote {
            violations.push(PolicyViolation::new_owned(
                "GAS_FEE_CAP_EXCEEDED",
                format!(
                    "intent gas/network fee {} exceeds configured cap {}",
                    intent.gas_fee_quote, self.config.risk.gas_fee_cap_quote
                ),
            ));
        }

        if !is_positive_finite(intent.expected_profit_quote) {
            violations.push(PolicyViolation::new(
                "EXPECTED_PROFIT_MUST_BE_POSITIVE",
                "expected profit must be positive and finite before fees",
            ));
        } else if is_non_negative_finite(intent.estimated_fee_quote)
            && is_non_negative_finite(intent.gas_fee_quote)
            && intent.expected_profit_quote <= intent.estimated_fee_quote + intent.gas_fee_quote
        {
            violations.push(PolicyViolation::new(
                "NOT_PROFITABLE_AFTER_FEES",
                "expected profit must exceed estimated venue and gas/network fees",
            ));
        }
    }

    fn evaluate_freshness(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        if intent.market_data_age_ms > self.context.max_market_data_age_ms {
            violations.push(PolicyViolation::new_owned(
                "MARKET_DATA_STALE",
                format!(
                    "market data age {} ms exceeds policy freshness ceiling {} ms",
                    intent.market_data_age_ms, self.context.max_market_data_age_ms
                ),
            ));
        }
    }

    fn evaluate_destination(
        &self,
        intent: &ExecutionIntent,
        violations: &mut Vec<PolicyViolation>,
    ) {
        match &intent.destination {
            DestinationPolicy::None | DestinationPolicy::InternalAccount => {}
            DestinationPolicy::ApprovedAddress { chain, label } => {
                if label.trim().is_empty() {
                    violations.push(PolicyViolation::new(
                        "DESTINATION_LABEL_REQUIRED",
                        "approved external destinations must use a stable non-empty label",
                    ));
                }

                if !self.config.venues.chain_allowlist.is_empty()
                    && !contains_ignore_ascii(&self.config.venues.chain_allowlist, chain)
                {
                    violations.push(PolicyViolation::new_owned(
                        "DESTINATION_CHAIN_NOT_ALLOWLISTED",
                        format!("destination chain {chain} is not allowlisted"),
                    ));
                }

                if !self
                    .context
                    .destination_allowlist
                    .iter()
                    .any(|entry| entry.matches_policy_destination(chain, label))
                {
                    violations.push(PolicyViolation::new_owned(
                        "DESTINATION_NOT_APPROVED",
                        format!(
                            "destination label {label} on chain {chain} is not in the approved destination allowlist"
                        ),
                    ));
                }
            }
            DestinationPolicy::UnknownAddress { .. } => violations.push(PolicyViolation::new(
                "UNKNOWN_DESTINATION_DENIED",
                "unknown destination addresses are denied by the trust contract",
            )),
            DestinationPolicy::LlmGenerated => violations.push(PolicyViolation::new(
                "LLM_GENERATED_DESTINATION_DENIED",
                "LLM-generated destinations are denied by the trust contract",
            )),
        }
    }

    fn evaluate_signing(&self, intent: &ExecutionIntent, violations: &mut Vec<PolicyViolation>) {
        if !intent.requires_signing {
            return;
        }

        if self.config.secrets.backend == SecretBackend::Disabled {
            violations.push(PolicyViolation::new(
                "SECRET_BACKEND_REQUIRED_FOR_SIGNING",
                "signing intents require a configured secret backend",
            ));
        }

        if self.config.secrets.wallet_signer.is_disabled() {
            violations.push(PolicyViolation::new(
                "WALLET_SIGNER_REFERENCE_REQUIRED",
                "signing intents require a wallet signer secret reference",
            ));
        }
    }
}

/// Dynamic policy inputs that must not be stored as static strategy config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyContext {
    /// Emergency stop state. When true, executable intents are denied.
    pub kill_switch_engaged: bool,
    /// Maximum age of source market data in milliseconds.
    pub max_market_data_age_ms: u64,
    /// Whether later phases have enabled a live execution runtime.
    pub live_execution_runtime_available: bool,
    /// Local approved external destination labels.
    pub destination_allowlist: Vec<ApprovedDestinationEntry>,
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self {
            kill_switch_engaged: false,
            max_market_data_age_ms: DEFAULT_MAX_MARKET_DATA_AGE_MS,
            live_execution_runtime_available: false,
            destination_allowlist: Vec::new(),
        }
    }
}

/// Proposed intent scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionScope {
    /// No execution; observations and analytics only.
    Observe,
    /// Simulated or paper-only execution.
    Paper,
    /// Real funds, real venues, or real signing.
    Live,
}

/// Proposed intent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionIntentKind {
    /// Non-executing observation.
    Observation,
    /// Centralized-exchange order.
    CexOrder,
    /// Decentralized-exchange swap.
    DexSwap,
    /// Cross-exchange arbitrage route.
    CrossExchangeArbitrage,
    /// Triangular arbitrage route.
    TriangularArbitrage,
    /// Cross-chain bridge or route.
    BridgeRoute,
    /// Transfer between accounts or wallets.
    Transfer,
    /// Withdrawal out of controlled venues or wallets.
    Withdrawal,
}

impl ExecutionIntentKind {
    /// Returns true when the intent may move funds or create market exposure.
    #[must_use]
    pub const fn requires_funds_movement(self) -> bool {
        !matches!(self, Self::Observation)
    }
}

/// Venue class used for allowlist matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VenueKind {
    /// Centralized exchange.
    Cex,
    /// Decentralized exchange.
    Dex,
    /// DEX or routing aggregator.
    Aggregator,
    /// Cross-chain bridge provider.
    Bridge,
}

/// Non-secret venue reference.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VenueRef {
    /// Configured venue identifier.
    pub name: String,
    /// Venue class.
    pub kind: VenueKind,
}

/// Destination classification for trust-contract checks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DestinationPolicy {
    /// No destination address is involved.
    None,
    /// Funds remain inside an approved controlled venue account.
    InternalAccount,
    /// External destination already approved by a future allowlist subsystem.
    ApprovedAddress { chain: String, label: String },
    /// Unknown external destination.
    UnknownAddress { chain: String },
    /// Destination generated or supplied by an LLM.
    LlmGenerated,
}

/// Deterministic execution intent passed into policy before any adapter acts.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionIntent {
    /// Stable unique intent id.
    pub id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Intent type.
    pub kind: ExecutionIntentKind,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target venue.
    pub venue: VenueRef,
    /// Optional chain reference for Web3 or bridge intents.
    pub chain: Option<String>,
    /// Base asset symbol.
    pub base_asset: String,
    /// Quote asset symbol.
    pub quote_asset: String,
    /// Proposed notional in quote-currency units.
    pub notional_quote: f64,
    /// Estimated gross profit in quote-currency units.
    pub expected_profit_quote: f64,
    /// Worst accepted loss for this intent in quote-currency units.
    pub max_loss_quote: f64,
    /// Requested maximum slippage in basis points.
    pub slippage_bps: u16,
    /// Estimated venue fee in quote-currency units.
    pub estimated_fee_quote: f64,
    /// Estimated gas/network fee in quote-currency units.
    pub gas_fee_quote: f64,
    /// Age of the source market data in milliseconds.
    pub market_data_age_ms: u64,
    /// Destination trust classification.
    pub destination: DestinationPolicy,
    /// Whether the intent requires wallet or transaction signing.
    pub requires_signing: bool,
}

/// Positive approval produced by policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyApproval {
    /// Trust-contract version that approved the intent.
    pub trust_contract_version: &'static str,
    /// Approved intent id.
    pub intent_id: String,
    /// Approved execution scope.
    pub approved_scope: ExecutionScope,
}

/// Deterministic policy decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Intent passed policy. Execution adapters must still perform their own
    /// connector-specific validation and journaling.
    Approved { approval: PolicyApproval },
    /// Intent failed policy.
    Denied { violations: Vec<PolicyViolation> },
}

impl PolicyDecision {
    /// Returns true when the decision is approved.
    #[must_use]
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }

    /// Return denial violations, or an empty slice for approvals.
    #[must_use]
    pub fn violations(&self) -> &[PolicyViolation] {
        match self {
            Self::Approved { .. } => &[],
            Self::Denied { violations } => violations,
        }
    }
}

/// Non-secret durable summary of one local policy decision.
///
/// This is a decision record only. It never authorizes live submission, signs
/// transactions, embeds secrets, or replaces future production custody checks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDecisionRecord {
    /// Trust-contract version that produced the decision.
    pub trust_contract_version: String,
    /// Stable intent id evaluated by policy.
    pub intent_id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Intent type.
    pub intent_kind: ExecutionIntentKind,
    /// Requested execution scope.
    pub requested_scope: ExecutionScope,
    /// Non-secret venue reference.
    pub venue: VenueRef,
    /// True when the policy decision approved the intent.
    pub approved: bool,
    /// Approved scope for approvals only.
    pub approved_scope: Option<ExecutionScope>,
    /// Stable denial codes for denied decisions.
    pub violation_codes: Vec<String>,
    /// Number of policy violations observed.
    pub violation_count: u64,
    /// Whether live scope was requested by the intent.
    pub live_scope_requested: bool,
    /// Whether the intent could move funds or create market exposure.
    pub funds_movement_requested: bool,
    /// Whether the intent requested signing.
    pub signing_requested: bool,
    /// Local decision recording never performs external submission.
    pub external_submission_performed: bool,
    /// Local decision recording never records secret material.
    pub secret_material_recorded: bool,
    /// Operator-supplied non-secret decision record timestamp.
    pub recorded_at_unix_ms: u64,
}

impl PolicyDecisionRecord {
    /// Build a non-secret decision summary from one evaluated intent.
    #[must_use]
    pub fn from_decision(
        intent: &ExecutionIntent,
        decision: &PolicyDecision,
        recorded_at_unix_ms: u64,
    ) -> Self {
        let (approved, approved_scope, violation_codes) = match decision {
            PolicyDecision::Approved { approval } => (true, Some(approval.approved_scope), vec![]),
            PolicyDecision::Denied { violations } => (
                false,
                None,
                violations
                    .iter()
                    .map(|violation| violation.code().to_owned())
                    .collect(),
            ),
        };

        Self {
            trust_contract_version: TRUST_CONTRACT_VERSION.to_owned(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            intent_kind: intent.kind,
            requested_scope: intent.scope,
            venue: intent.venue.clone(),
            approved,
            approved_scope,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            live_scope_requested: intent.scope == ExecutionScope::Live,
            funds_movement_requested: intent.kind.requires_funds_movement(),
            signing_requested: intent.requires_signing,
            external_submission_performed: false,
            secret_material_recorded: false,
            recorded_at_unix_ms,
        }
    }

    /// Validate that the record is coherent and contains no execution side effects.
    pub fn validate(&self) -> Result<(), StateStoreError> {
        if self.trust_contract_version != TRUST_CONTRACT_VERSION {
            return Err(StateStoreError::ValidationFailed {
                reason: "policy decision trust-contract version mismatch".to_owned(),
            });
        }
        if self.intent_id.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "policy decision intent id is required".to_owned(),
            });
        }
        if self.strategy_id.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "policy decision strategy id is required".to_owned(),
            });
        }
        if self.recorded_at_unix_ms == 0 {
            return Err(StateStoreError::ValidationFailed {
                reason: "policy decision record timestamp is required".to_owned(),
            });
        }
        if self.external_submission_performed || self.secret_material_recorded {
            return Err(StateStoreError::ValidationFailed {
                reason:
                    "policy decision records must not contain submission or secret side effects"
                        .to_owned(),
            });
        }
        if self.approved {
            if self.approved_scope.is_none()
                || self.violation_count != 0
                || !self.violation_codes.is_empty()
            {
                return Err(StateStoreError::ValidationFailed {
                    reason: "approved policy decisions must not include denial violations"
                        .to_owned(),
                });
            }
        } else if self.approved_scope.is_some()
            || self.violation_count == 0
            || self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX)
        {
            return Err(StateStoreError::ValidationFailed {
                reason: "denied policy decisions require coherent violation codes".to_owned(),
            });
        }

        Ok(())
    }
}

/// Persist the latest local policy decision through the typed state boundary.
pub fn persist_policy_decision_checkpoint(
    store: &mut impl StateStore,
    record: &PolicyDecisionRecord,
) -> Result<StateCheckpoint, StateStoreError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: POLICY_LAST_DECISION_CHECKPOINT_KEY.to_owned(),
        subsystem: POLICY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize policy decision record: {error}"),
        })?,
        updated_at_unix_ms: record.recorded_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Append one local policy decision summary to the append-only audit journal.
pub fn append_policy_decision_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &PolicyDecisionRecord,
) -> Result<AuditRecord, AuditError> {
    record
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "POLICY_DECISION_RECORD_INVALID",
                error.to_string(),
            )],
        })?;

    let event = AuditEvent::new(
        format!("policy-decision-{}", record.intent_id),
        AuditEventKind::PolicyDecision,
        POLICY_STATE_SUBSYSTEM,
        "policy-engine",
        "local policy decision recorded without execution side effects",
    )
    .with_metadata(
        "trust_contract_version",
        AuditValue::Text(record.trust_contract_version.clone()),
    )
    .with_metadata("intent_id", AuditValue::Text(record.intent_id.clone()))
    .with_metadata("strategy_id", AuditValue::Text(record.strategy_id.clone()))
    .with_metadata(
        "intent_kind",
        AuditValue::Text(format!("{:?}", record.intent_kind)),
    )
    .with_metadata(
        "requested_scope",
        AuditValue::Text(format!("{:?}", record.requested_scope)),
    )
    .with_metadata("venue", AuditValue::Text(record.venue.name.clone()))
    .with_metadata("approved", AuditValue::Bool(record.approved))
    .with_metadata(
        "violation_count",
        AuditValue::Unsigned(record.violation_count),
    )
    .with_metadata(
        "live_scope_requested",
        AuditValue::Bool(record.live_scope_requested),
    )
    .with_metadata(
        "funds_movement_requested",
        AuditValue::Bool(record.funds_movement_requested),
    )
    .with_metadata(
        "signing_requested",
        AuditValue::Bool(record.signing_requested),
    )
    .with_metadata(
        "external_submission_performed",
        AuditValue::Bool(record.external_submission_performed),
    )
    .with_metadata(
        "sensitive_material_recorded",
        AuditValue::Bool(record.secret_material_recorded),
    );

    journal.append_event(event)
}

/// One deterministic policy violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyViolation {
    code: &'static str,
    message: String,
}

impl PolicyViolation {
    /// Create a policy violation.
    #[must_use]
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_owned(),
        }
    }

    /// Create a policy violation with owned message text.
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

fn contains_ignore_ascii(values: &[String], needle: &str) -> bool {
    values
        .iter()
        .any(|value| value.eq_ignore_ascii_case(needle))
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
        append_policy_decision_audit, persist_policy_decision_checkpoint, DestinationPolicy,
        ExecutionIntent, ExecutionIntentKind, ExecutionScope, PolicyContext, PolicyDecisionRecord,
        PolicyEngine, VenueKind, VenueRef, POLICY_LAST_DECISION_CHECKPOINT_KEY,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, AuditValue, SqliteWalStateStore, StateStore,
        LIVE_ACKNOWLEDGEMENT,
    };
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    const BASE_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 10.0
max_daily_loss_quote = 2.0
max_open_exposure_quote = 20.0
slippage_bps = 50
gas_fee_cap_quote = 1.0

[venues]
cex_allowlist = ["coinbase", "kraken"]
dex_allowlist = ["uniswap"]
chain_allowlist = ["ethereum"]
asset_allowlist = ["BTC", "ETH", "USDC"]

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

    fn config(text: &str) -> AgentConfig {
        AgentConfig::from_toml_str(text).expect("test config should validate")
    }

    fn paper_intent() -> ExecutionIntent {
        ExecutionIntent {
            id: "intent-001".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            kind: ExecutionIntentKind::CexOrder,
            scope: ExecutionScope::Paper,
            venue: VenueRef {
                name: "coinbase".to_owned(),
                kind: VenueKind::Cex,
            },
            chain: None,
            base_asset: "BTC".to_owned(),
            quote_asset: "USDC".to_owned(),
            notional_quote: 5.0,
            expected_profit_quote: 0.25,
            max_loss_quote: 0.5,
            slippage_bps: 10,
            estimated_fee_quote: 0.05,
            gas_fee_quote: 0.0,
            market_data_age_ms: 1_000,
            destination: DestinationPolicy::InternalAccount,
            requires_signing: false,
        }
    }

    #[test]
    fn paper_mode_approves_safe_paper_intent() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let decision = engine.evaluate(&paper_intent());
        assert!(decision.is_approved(), "decision: {decision:?}");
    }

    #[test]
    fn observe_mode_denies_funds_movement() {
        let observe_config = BASE_CONFIG.replace("mode = \"paper\"", "mode = \"observe\"");
        let engine = PolicyEngine::from_config(config(&observe_config));
        let decision = engine.evaluate(&paper_intent());
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "OBSERVE_MODE_DENIES_EXECUTION"));
    }

    #[test]
    fn unknown_destination_is_denied() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let mut intent = paper_intent();
        intent.destination = DestinationPolicy::UnknownAddress {
            chain: "ethereum".to_owned(),
        };
        let decision = engine.evaluate(&intent);
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "UNKNOWN_DESTINATION_DENIED"));
    }

    #[test]
    fn approved_address_requires_destination_allowlist_entry() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let mut intent = paper_intent();
        intent.destination = DestinationPolicy::ApprovedAddress {
            chain: "ethereum".to_owned(),
            label: "ops-vault".to_owned(),
        };
        let decision = engine.evaluate(&intent);
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "DESTINATION_NOT_APPROVED"));
    }

    #[test]
    fn stale_market_data_is_denied() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let mut intent = paper_intent();
        intent.market_data_age_ms = 30_000;
        let decision = engine.evaluate(&intent);
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "MARKET_DATA_STALE"));
    }

    #[test]
    fn withdrawal_is_denied_by_trust_contract() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let mut intent = paper_intent();
        intent.kind = ExecutionIntentKind::Withdrawal;
        let decision = engine.evaluate(&intent);
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "WITHDRAWALS_DENIED_BY_TRUST_CONTRACT"));
    }

    #[test]
    fn live_scope_is_denied_until_runtime_becomes_available() {
        let live_config = BASE_CONFIG
            .replace("mode = \"paper\"", "mode = \"live-armed\"")
            .replace("live_execution_enabled = false", "live_execution_enabled = true")
            .replace(
                "allow_withdrawals = false",
                &format!(
                    "operator_acknowledgement = \"{LIVE_ACKNOWLEDGEMENT}\"\nallow_withdrawals = false"
                ),
            )
            .replace("backend = \"disabled\"", "backend = \"env\"")
            .replace(
                "exchange_credentials = { source = \"disabled\" }",
                "exchange_credentials = { source = \"env\", name = \"ARB_EXCHANGE_REFERENCE\" }",
            )
            .replace(
                "wallet_signer = { source = \"disabled\" }",
                "wallet_signer = { source = \"env\", name = \"ARB_WALLET_REFERENCE\" }",
            );
        let engine = PolicyEngine::from_config(config(&live_config));
        let mut intent = paper_intent();
        intent.scope = ExecutionScope::Live;
        let decision = engine.evaluate(&intent);
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "LIVE_RUNTIME_NOT_AVAILABLE"));
    }

    #[test]
    fn kill_switch_denies_funds_movement() {
        let engine = PolicyEngine::new(
            config(BASE_CONFIG),
            PolicyContext {
                kill_switch_engaged: true,
                ..PolicyContext::default()
            },
        );
        let decision = engine.evaluate(&paper_intent());
        assert!(decision
            .violations()
            .iter()
            .any(|violation| violation.code() == "KILL_SWITCH_ENGAGED"));
    }

    #[test]
    fn approved_policy_decision_audits_and_persists_locally() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let intent = paper_intent();
        let decision = engine.evaluate(&intent);
        let record = PolicyDecisionRecord::from_decision(&intent, &decision, 1_719_000_000_001);

        assert!(record.approved);
        assert_eq!(record.violation_count, 0);
        assert!(!record.external_submission_performed);
        assert!(!record.secret_material_recorded);

        let audit_path = unique_temp_path("approved-policy-decision", "jsonl");
        let state_path = unique_temp_path("approved-policy-decision", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("audit journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");

        let audit_record =
            append_policy_decision_audit(&mut journal, &record).expect("audit append succeeds");
        let checkpoint =
            persist_policy_decision_checkpoint(&mut store, &record).expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, POLICY_LAST_DECISION_CHECKPOINT_KEY);

        let reopened_journal =
            AppendOnlyAuditJournal::open(&audit_path).expect("audit journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let checkpoint = reopened_store
            .get_checkpoint(POLICY_LAST_DECISION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: PolicyDecisionRecord =
            serde_json::from_str(&checkpoint.value).expect("policy record deserializes");
        assert_eq!(recovered, record);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn denied_policy_decision_audits_violation_summary_without_side_effects() {
        let engine = PolicyEngine::from_config(config(BASE_CONFIG));
        let mut intent = paper_intent();
        intent.scope = ExecutionScope::Live;
        intent.requires_signing = true;
        let decision = engine.evaluate(&intent);
        let record = PolicyDecisionRecord::from_decision(&intent, &decision, 1_719_000_000_002);

        assert!(!record.approved);
        assert!(record.live_scope_requested);
        assert!(record.signing_requested);
        assert!(record.violation_count > 0);
        assert!(record
            .violation_codes
            .iter()
            .any(|code| code == "PAPER_MODE_DENIES_LIVE_SCOPE"));
        assert!(!record.external_submission_performed);
        assert!(!record.secret_material_recorded);

        let audit_path = unique_temp_path("denied-policy-decision", "jsonl");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("audit journal opens");
        let audit_record =
            append_policy_decision_audit(&mut journal, &record).expect("audit append succeeds");

        assert_eq!(audit_record.sequence, 1);
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("external_submission_performed"),
            Some(AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("sensitive_material_recorded"),
            Some(AuditValue::Bool(false))
        ));

        let reopened_journal =
            AppendOnlyAuditJournal::open(&audit_path).expect("audit journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let _ = fs::remove_file(audit_path);
    }

    fn unique_temp_path(label: &str, extension: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("arbyclaw-policy-{label}-{nanos}-{n}.{extension}"))
    }
}
