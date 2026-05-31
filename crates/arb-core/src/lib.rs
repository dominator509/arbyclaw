#![forbid(unsafe_code)]
#![allow(
    clippy::assigning_clones,
    clippy::cast_precision_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::doc_markdown,
    clippy::if_not_else,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::redundant_closure_for_method_calls,
    clippy::struct_excessive_bools,
    clippy::unnecessary_literal_bound,
    clippy::useless_let_if_seq
)]
#![allow(clippy::module_name_repetitions)]

//! Core primitives for ArbyClaw.

pub mod audit;
pub mod cex;
pub mod communications;
pub mod config;
pub mod dashboard;
pub mod dex;
pub mod execution_adapter;
pub mod fees;
pub mod handoff;
pub mod hardening;
pub mod market_data;
pub mod observability;
pub mod opportunity;
pub mod packaging;
pub mod paper;
pub mod planner;
pub mod policy;
pub mod runtime;
pub mod secrets;
pub mod state;
pub mod testing;

pub use cex::{
    CexConnectorCapabilities, CexConnectorError, CexConnectorIdentity, CexConnectorRegistry,
    CexConnectorViolation, CexOrderRequest, CexOrderSide, CexOrderStatus, CexOrderType,
    CexPolicyGate, CexReadOnlyConnector, CexTimeInForce, CexTradingConnector, CexVenueProfile,
    CEX_CONNECTOR_FRAMEWORK_VERSION,
};

pub use dex::{
    DexConnectorError, DexConnectorIdentity, DexConnectorRegistry, DexConnectorViolation,
    DexPolicyGate, DexQuoteConnector, DexRouteKind, DexRouterCapabilities, DexRouterProfile,
    DexSimulationStatus, DexSwapMode, DexSwapQuoteRequest, DexSwapQuoteResponse, DexTokenProfile,
    Web3ChainProfile, Web3SimulationConnector, Web3TransactionSimulationRequest,
    Web3TransactionSimulationResponse, DEX_CONNECTOR_FRAMEWORK_VERSION,
};

pub use communications::{
    parse_cli_command, CommunicationBoundaryConfig, CommunicationChannelKind, CommunicationError,
    CommunicationViolation, DeterministicNotificationBoundary, DeterministicOperatorCommandRouter,
    NotificationChannelDispatch, NotificationChannelDispatchStatus, NotificationChannelProfile,
    NotificationDispatchRecord, NotificationDispatchStatus, NotificationPublishRequest,
    NotificationPublisher, NotificationSeverity, OperatorCommand, OperatorCommandAction,
    OperatorCommandKind, OperatorCommandRouter, OperatorCommandRoutingRequest,
    OperatorCommandSource, OperatorNotification, RoutedOperatorCommand, COMMUNICATIONS_CLI_VERSION,
};

pub use dashboard::{
    DashboardBoundaryConfig, DashboardError, DashboardPanel, DashboardPanelItem,
    DashboardPanelKind, DashboardRenderRecord, DashboardRenderRequest, DashboardRenderer,
    DashboardServerBinding, DashboardSeverity, DashboardSnapshot, DashboardViolation,
    DeterministicDashboardRenderer, DASHBOARD_BOUNDARY_VERSION,
};

pub use audit::{
    AppendOnlyAuditJournal, AuditError, AuditEvent, AuditEventKind, AuditRecord, AuditValue,
    AuditViolation, AUDIT_GENESIS_HASH, AUDIT_JOURNAL_FORMAT_VERSION,
};

pub use config::{
    load_config_file, AgentConfig, AuditConfig, CommunicationConfig, ConfigError, ConfigViolation,
    RiskLimitsConfig, RuntimeConfig, SecretBackend, SecretsConfig, VenueAllowlistsConfig,
    LIVE_ACKNOWLEDGEMENT,
};
pub use secrets::{EnvSecretProvider, SecretMaterial, SecretProvider, SecretRef, SecretStoreError};

pub use fees::{
    FeeAdjustedEdge, FeeEstimate, FeeModelError, FeeModelViolation, FeeProvider, FeeSchedule,
    LiquidityRole, FEE_MODEL_VERSION,
};

pub use handoff::{
    AgenticHandoffBoundaryConfig, AgenticHandoffError, AgenticHandoffPackage,
    AgenticHandoffPackager, AgenticHandoffReviewRecord, AgenticHandoffReviewRequest,
    AgenticHandoffReviewStatus, AgenticHandoffViolation, DeterministicAgenticHandoffPackager,
    HandoffAgentKind, HandoffArtifactKind, HandoffInstructionArtifact, AGENTIC_HANDOFF_VERSION,
};
pub use hardening::{
    DeterministicExternalHardeningReviewer, ExternalHardeningActivityKind,
    ExternalHardeningBoundaryConfig, ExternalHardeningError, ExternalHardeningReviewRecord,
    ExternalHardeningReviewRequest, ExternalHardeningReviewStatus, ExternalHardeningReviewer,
    ExternalHardeningViolation, HardeningEvidenceRecord, HardeningEvidenceStatus,
    ProductionHardeningPlan, EXTERNAL_HARDENING_VERSION,
};
pub use market_data::{
    FreshnessStatus, MarketDataCapabilities, MarketDataError, MarketDataProvider,
    MarketDataRequest, MarketDataViolation, MarketPair, NormalizedQuote, OrderBookSnapshot,
    PriceLevel, DEFAULT_MARKET_DATA_FRESHNESS_MS, MARKET_DATA_MODEL_VERSION,
};

pub use packaging::{
    DeploymentEnvironmentKind, DeploymentNetworkExposure, DeploymentPackagePlan,
    DeploymentPackageRecord, DeploymentPackageRequest, DeploymentPackageStatus,
    DeterministicPackagingDeploymentPlanner, PackageArtifactKind, PackageTargetPlan,
    PackagingBoundaryConfig, PackagingBoundaryError, PackagingBoundaryViolation,
    PackagingDeploymentPlanner, ReleaseGate, RollbackStep, RuntimeConfigurationStrategy,
    ServiceHardeningProfile, PACKAGING_DEPLOYMENT_VERSION,
};

pub use observability::{
    ComponentHealthStatus, DeterministicObservabilityCollector, HealthStatus, MetricKind,
    MetricLabel, MetricSample, ObservabilityBoundaryConfig, ObservabilityCollectionRequest,
    ObservabilityCollector, ObservabilityEndpointBinding, ObservabilityError, ObservabilityRecord,
    ObservabilitySeverity, ObservabilitySnapshot, ObservabilityViolation, Runbook, RunbookStep,
    StructuredLogEvent, StructuredLogField, OBSERVABILITY_RUNBOOK_VERSION,
};

pub use opportunity::{
    DeterministicOpportunityEngine, OpportunityCandidate, OpportunityDiscoveryConfig,
    OpportunityDiscoveryRequest, OpportunityEngine, OpportunityError, OpportunityLeg,
    OpportunityLegSide, OpportunityRouteKind, OpportunityScore, OpportunityViolation,
    OPPORTUNITY_ENGINE_VERSION,
};

pub use paper::{
    append_paper_execution_report_audit, append_paper_ledger_entry_audit,
    append_paper_ledgered_execution_audit, persist_paper_balance_ledger_checkpoint,
    persist_paper_execution_report_checkpoint, validate_paper_runtime, PaperAdverseSelectionConfig,
    PaperAdverseSelectionReport, PaperAssetBalance, PaperAuditJournalWriteReport,
    PaperAuditReplayValidationReport, PaperAuditedLedgeredExecution,
    PaperAuditedVenueRealismLedgeredExecution, PaperBacktestCorpus, PaperBacktestRunReport,
    PaperBacktestScenario, PaperBacktestScenarioReport, PaperBacktestStep, PaperBalanceLedger,
    PaperCalibrationApplicationReport, PaperConnectorError, PaperExchangeMatchingProfile,
    PaperExecutionAdapter, PaperExecutionReport, PaperExecutionStatus, PaperFeeProvider,
    PaperFillModelConfig, PaperFillSide, PaperFillSimulationReport, PaperFillSimulationRequest,
    PaperFillSimulationStatus, PaperLedgerEntry, PaperLedgerEntryKind, PaperLedgeredExecution,
    PaperMarketDataProvider, PaperMatchingProfileReport, PaperReplayValidationStatus,
    PaperReplayViolation, PaperRuntimeValidationReport, PaperRuntimeValidationRequest,
    PaperVenueCalibrationRecord, PaperVenueRealismExecution, PaperVenueRealismLedgeredExecution,
    PaperVenueRealismRequest, PAPER_AUDIT_INTEGRATION_VERSION, PAPER_BALANCE_LEDGER_CHECKPOINT_KEY,
    PAPER_BALANCE_LEDGER_VERSION, PAPER_CONNECTOR_VERSION,
    PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY, PAPER_EXECUTION_STATE_SUBSYSTEM,
    PAPER_REALISM_VALIDATION_VERSION, PAPER_REALISTIC_FILL_MODEL_VERSION,
};

pub use testing::{
    BacktestDatasetDefinition, BacktestScenarioDefinition, DeterministicValidationHarness,
    ExpectedValidationOutcome, FixtureKind, FuzzCorpusDefinition, FuzzSeedRecord, FuzzTargetKind,
    ValidationExecutionMode, ValidationFixtureRecord, ValidationHarness, ValidationHarnessConfig,
    ValidationHarnessError, ValidationHarnessViolation, ValidationPlan, ValidationRunRecord,
    ValidationRunRequest, ValidationRunStatus, ValidationSuiteKind, ValidationTestCase,
    TESTING_BACKTESTING_VERSION,
};

pub use planner::{
    persist_execution_plan_draft_checkpoint, DeterministicExecutionPlanner, ExecutionPlanDraft,
    ExecutionPlanFailureMode, ExecutionPlanStatus, ExecutionPlanStep, ExecutionPlanStepAction,
    ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerError, ExecutionPlannerRequest,
    ExecutionPlannerViolation, PlannerPolicyOutcome, PlannerPolicyStatus, PlannerPolicyViolation,
    EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY, EXECUTION_PLANNER_STATE_SUBSYSTEM,
    EXECUTION_PLANNER_VERSION,
};

pub use execution_adapter::{
    persist_execution_adapter_run_checkpoint, DeterministicExecutionAdapterBoundary,
    ExecutionAdapter, ExecutionAdapterAction, ExecutionAdapterAttempt,
    ExecutionAdapterAttemptStatus, ExecutionAdapterConfig, ExecutionAdapterError,
    ExecutionAdapterRequest, ExecutionAdapterRunRecord, ExecutionAdapterRunStatus,
    ExecutionAdapterViolation, ExecutionFillRecord, ExecutionFillStatus,
    ExecutionReconciliationRecord, ExecutionReconciliationStatus,
    EXECUTION_ADAPTER_FRAMEWORK_VERSION, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY,
    EXECUTION_ADAPTER_STATE_SUBSYSTEM,
};

pub use state::{
    InMemoryStateStore, SqliteWalDurabilityReport, SqliteWalStateStore, StateCheckpoint,
    StateStore, StateStoreError, SQLITE_WAL_DURABILITY_VERSION,
};

pub use runtime::{
    run_local_runtime_lifecycle, RuntimeLifecycleError, RuntimeLifecycleRecord,
    RuntimeLifecycleRequest, RuntimeLifecycleStatus, RUNTIME_LIFECYCLE_VERSION,
};

pub use policy::{
    DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope, PolicyApproval,
    PolicyContext, PolicyDecision, PolicyEngine, PolicyViolation, VenueKind, VenueRef,
    DEFAULT_MAX_MARKET_DATA_AGE_MS, TRUST_CONTRACT_VERSION,
};

use serde::{Deserialize, Serialize};

/// Stable project display name used by CLI/runtime surfaces.
pub const AGENT_NAME: &str = "ArbyClaw";

/// Runtime execution mode.
///
/// Live execution remains unavailable until later phases implement typed policy,
/// secret custody, audit journaling, execution adapters, and external validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeMode {
    /// Observe opportunities without placing orders or signing transactions.
    Observe,
    /// Use simulated or paper venues only.
    Paper,
    /// Permit live execution only after explicit future policy gates approve it.
    LiveArmed,
}

impl RuntimeMode {
    /// Returns whether the mode is intended to permit live execution.
    ///
    /// This is not sufficient authorization by itself. Future phases must layer
    /// deny-by-default policy checks, audit journaling, stale-data checks,
    /// exchange capability checks, and signer constraints on top.
    #[must_use]
    pub const fn permits_live_execution(self) -> bool {
        matches!(self, Self::LiveArmed)
    }
}

/// Build identity exposed by the binary for deterministic diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildIdentity {
    name: &'static str,
    version: &'static str,
}

impl BuildIdentity {
    /// Returns the current crate build identity.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            name: AGENT_NAME,
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    /// Returns the project name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the semantic package version.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildIdentity, RuntimeMode, AGENT_NAME};

    #[test]
    fn live_execution_is_only_permitted_by_live_armed_mode() {
        assert!(!RuntimeMode::Observe.permits_live_execution());
        assert!(!RuntimeMode::Paper.permits_live_execution());
        assert!(RuntimeMode::LiveArmed.permits_live_execution());
    }

    #[test]
    fn build_identity_is_stable_and_non_empty() {
        let identity = BuildIdentity::current();
        assert_eq!(identity.name(), AGENT_NAME);
        assert!(!identity.version().is_empty());
    }
}
