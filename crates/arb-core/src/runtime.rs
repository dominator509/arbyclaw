#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    append_channel_adapter_validation_audit, append_channel_session_validation_audit,
    append_dashboard_hosted_request_preflight_audit,
    append_dashboard_hosted_request_validation_audit,
    append_dashboard_hosted_security_review_audit, append_dashboard_render_audit,
    append_execution_adapter_recovery_plan_audit, append_execution_adapter_run_audit,
    append_execution_plan_draft_audit, append_local_tracing_subscriber_audit,
    append_notification_dispatch_audit, append_observability_alert_route_dispatch_audit,
    append_observability_endpoint_preflight_audit, append_observability_export_dry_run_audit,
    append_observability_loopback_bind_validation_audit,
    append_observability_metrics_endpoint_validation_audit,
    append_observability_metrics_scrape_preflight_audit,
    append_observability_operations_review_audit, append_observability_record_audit,
    append_paper_execution_intent_audit, append_paper_execution_report_audit,
    append_platform_adapter_review_audit, append_platform_command_ingress_audit,
    append_property_check_report_audit, append_remote_command_envelope_validation_audit,
    append_remote_command_security_review_audit, append_routed_operator_command_audit,
    append_runtime_failure_capture_audit, append_validation_run_audit,
    capture_local_runtime_failure, ledger_execution_adapter_run_paper_fills, parse_cli_command,
    persist_channel_adapter_validation_checkpoint, persist_channel_session_validation_checkpoint,
    persist_dashboard_hosted_request_preflight_checkpoint,
    persist_dashboard_hosted_request_validation_checkpoint,
    persist_dashboard_hosted_security_review_checkpoint, persist_dashboard_render_checkpoint,
    persist_execution_adapter_recovery_plan_checkpoint, persist_execution_adapter_run_checkpoint,
    persist_execution_plan_draft_checkpoint, persist_local_tracing_subscriber_checkpoint,
    persist_notification_dispatch_checkpoint,
    persist_observability_alert_route_dispatch_checkpoint,
    persist_observability_endpoint_preflight_checkpoint,
    persist_observability_export_dry_run_checkpoint,
    persist_observability_loopback_bind_validation_checkpoint,
    persist_observability_metrics_endpoint_validation_checkpoint,
    persist_observability_metrics_scrape_preflight_checkpoint,
    persist_observability_operations_review_checkpoint, persist_observability_record_checkpoint,
    persist_paper_execution_report_checkpoint, persist_platform_adapter_review_checkpoint,
    persist_platform_command_ingress_checkpoint, persist_property_check_report_checkpoint,
    persist_remote_command_envelope_validation_checkpoint,
    persist_remote_command_security_review_checkpoint, persist_routed_operator_command_checkpoint,
    persist_runtime_failure_capture_checkpoint, persist_validation_run_checkpoint,
    phase27_local_opportunity_historical_fixture_corpus, plan_execution_adapter_recovery,
    preflight_dashboard_hosted_request, preflight_observability_endpoint,
    preflight_observability_metrics_scrape, record_observability_alert_route_dispatch,
    render_observability_export_dry_run, review_dashboard_hosted_security,
    review_observability_operations, review_platform_adapter_controls,
    review_platform_command_ingress, review_remote_command_security,
    run_local_validation_property_checks, validate_audit_journal_durability,
    validate_channel_adapter, validate_channel_session, validate_dashboard_hosted_request,
    validate_local_tracing_subscriber, validate_observability_loopback_bind,
    validate_observability_metrics_endpoint, validate_opportunity_candidate_trace_restart_recovery,
    validate_remote_command_envelope, AppendOnlyAuditJournal, AuditError, AuditEvent,
    AuditEventKind, AuditValue, BacktestDatasetDefinition, BacktestScenarioDefinition,
    CexOrderLifecycleRecord, ChannelAdapterValidationReport, ChannelAdapterValidationRequest,
    ChannelAdapterValidationStatus, ChannelSessionValidationReport, ChannelSessionValidationStatus,
    CommunicationBoundaryConfig, ComponentHealthStatus, DashboardAccessContext,
    DashboardAccessSource, DashboardBoundaryConfig, DashboardHostedRequestMethod,
    DashboardHostedRequestPreflight, DashboardHostedRequestPreflightReport,
    DashboardHostedRequestPreflightStatus, DashboardHostedRequestValidation,
    DashboardHostedRequestValidationReport, DashboardHostedRequestValidationStatus,
    DashboardHostedSecurityPolicy, DashboardHostedSecurityReviewReport,
    DashboardHostedSecurityReviewStatus, DashboardPanel, DashboardPanelItem, DashboardPanelKind,
    DashboardRenderRecord, DashboardRenderRequest, DashboardRenderer, DashboardSeverity,
    DashboardSnapshot, DeterministicDashboardRenderer, DeterministicExecutionAdapterBoundary,
    DeterministicNotificationBoundary, DeterministicObservabilityCollector,
    DeterministicOperatorCommandRouter, DeterministicValidationHarness, DexSwapLifecycleRecord,
    ExecutionAdapter, ExecutionAdapterConfig, ExecutionAdapterError, ExecutionAdapterRequest,
    ExecutionAdapterRunRecord, ExecutionPlanDraft, ExecutionScope, ExpectedValidationOutcome,
    FixtureKind, FuzzCorpusDefinition, FuzzSeedRecord, FuzzTargetKind, HealthStatus,
    LocalPropertyCheckReport, LocalTracingSubscriberValidationReport,
    LocalTracingSubscriberValidationRequest, LocalTracingSubscriberValidationStatus, MetricKind,
    MetricSample, NotificationChannelProfile, NotificationChannelSafetyState,
    NotificationDispatchRecord, NotificationDispatchStatus, NotificationPublisher,
    NotificationSeverity, ObservabilityAccessContext, ObservabilityAlertRouteDispatchReport,
    ObservabilityAlertRouteDispatchRequest, ObservabilityAlertRouteDispatchStatus,
    ObservabilityBoundaryConfig, ObservabilityCollectionRequest, ObservabilityCollector,
    ObservabilityEndpointPreflight, ObservabilityEndpointPreflightReport,
    ObservabilityEndpointPreflightStatus, ObservabilityExportDryRunReport,
    ObservabilityExportDryRunRequest, ObservabilityLoopbackBindValidationReport,
    ObservabilityLoopbackBindValidationRequest, ObservabilityLoopbackBindValidationStatus,
    ObservabilityMetricsEndpointValidationReport, ObservabilityMetricsEndpointValidationRequest,
    ObservabilityMetricsEndpointValidationStatus, ObservabilityMetricsScrapePreflightReport,
    ObservabilityMetricsScrapePreflightRequest, ObservabilityMetricsScrapePreflightStatus,
    ObservabilityOperationsPolicy, ObservabilityOperationsReviewReport,
    ObservabilityOperationsReviewStatus, ObservabilityRecord, ObservabilitySeverity,
    ObservabilitySnapshot, OperatorCommandRouter, OperatorCommandRoutingRequest,
    OperatorCommandSource, OperatorNotification, OpportunityCandidateTraceRecoveryReport,
    OpportunityHistoricalFixtureCorpus, PaperAdapterRunLedgerReport, PaperAssetBalance,
    PaperBalanceLedger, PaperExecutionAdapter, PlatformAdapterReviewReport,
    PlatformAdapterReviewRequest, PlatformAdapterReviewStatus, PlatformCommandIngressReport,
    PlatformCommandIngressRequest, PlatformCommandIngressStatus, PolicyEngine,
    RecoveredOpportunityTraceSummary, RemoteCommandEnvelopeValidationReport,
    RemoteCommandEnvelopeValidationRequest, RemoteCommandEnvelopeValidationStatus,
    RemoteCommandSecurityReviewReport, RemoteCommandSecurityReviewRequest,
    RemoteCommandSecurityReviewStatus, RoutedOperatorCommand, RuntimeFailureCaptureRecord,
    RuntimeFailureCaptureRequest, RuntimeFailureKind, RuntimeMode, SqliteWalStateStore,
    StateCheckpoint, StateStore, StateStoreError, StructuredLogEvent, StructuredLogField,
    ValidationExecutionMode, ValidationFixtureRecord, ValidationHarness, ValidationHarnessConfig,
    ValidationPlan, ValidationRunRecord, ValidationRunRequest, ValidationRunStatus,
    ValidationSuiteKind, ValidationTestCase, CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY, DASHBOARD_LAST_RENDER_CHECKPOINT_KEY,
    DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY, EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY,
    EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY, OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY, OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY, PAPER_BALANCE_LEDGER_CHECKPOINT_KEY,
    PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY, TESTING_LAST_PROPERTY_CHECK_REPORT_KEY,
    TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process,
    sync::{Arc, Barrier, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

/// Stable runtime lifecycle version for audit, state, and handoff surfaces.
pub const RUNTIME_LIFECYCLE_VERSION: &str = "phase-runtime-local-lifecycle-v1";

/// Stable local graceful-shutdown validation version.
pub const RUNTIME_GRACEFUL_SHUTDOWN_VERSION: &str = "phase26-runtime-graceful-shutdown-local-v1";

/// Stable local runtime backup/restore validation version.
pub const RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION: &str =
    "phase26-runtime-backup-restore-local-v1";

/// Stable local deployment-host backup/restore transcript validation version.
pub const RUNTIME_DEPLOYMENT_BACKUP_RESTORE_TRANSCRIPT_VERSION: &str =
    "phase75-deployment-backup-restore-transcript-local-v1";

/// Stable local deployment-host graceful-shutdown transcript validation version.
pub const RUNTIME_DEPLOYMENT_GRACEFUL_SHUTDOWN_TRANSCRIPT_VERSION: &str =
    "phase76-deployment-graceful-shutdown-transcript-local-v1";

/// Stable local runtime restart recovery validation version.
pub const RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION: &str =
    "phase26-runtime-restart-recovery-local-v1";

/// Stable local runtime deployment-smoke validation version.
pub const RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION: &str =
    "phase26-runtime-deployment-smoke-local-v1";

/// Stable local runtime load profile review validation version.
pub const RUNTIME_LOAD_PROFILE_REVIEW_VERSION: &str =
    "phase70-runtime-load-profile-review-local-v1";

/// Stable local production-runtime preflight validation version.
pub const RUNTIME_PRODUCTION_PREFLIGHT_VALIDATION_VERSION: &str =
    "phase49-runtime-production-preflight-local-v1";

/// Stable local service-manager lifecycle transcript validation version.
pub const RUNTIME_SERVICE_MANAGER_LIFECYCLE_TRANSCRIPT_VERSION: &str =
    "phase59-service-manager-lifecycle-transcript-local-v2";

/// Stable local service-manager lifecycle rehearsal validation version.
pub const RUNTIME_SERVICE_MANAGER_LIFECYCLE_REHEARSAL_VERSION: &str =
    "phase68-service-manager-lifecycle-rehearsal-local-v1";

/// Stable local deployment permission transcript validation version.
pub const RUNTIME_DEPLOYMENT_PERMISSION_TRANSCRIPT_VERSION: &str =
    "phase60-deployment-permission-transcript-local-v2";

/// Stable local deployment-host audit/SQLite transcript validation version.
pub const RUNTIME_DEPLOYMENT_AUDIT_SQLITE_TRANSCRIPT_VERSION: &str =
    "phase57-deployment-audit-sqlite-transcript-local-v1";

/// Stable local deployment-host SQLite schema migration transcript validation version.
pub const RUNTIME_DEPLOYMENT_SQLITE_SCHEMA_MIGRATION_TRANSCRIPT_VERSION: &str =
    "phase67-deployment-sqlite-schema-migration-transcript-local-v1";

/// State checkpoint key for the last local graceful-shutdown record.
pub const RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY: &str = "runtime:last-graceful-shutdown";

/// Runtime lifecycle status for local-only adapter evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeLifecycleStatus {
    /// Plan state was persisted before adapter evaluation.
    PlanCheckpointed,
    /// Deterministic adapter run was evaluated and persisted.
    AdapterRunCheckpointed,
}

/// One local runtime lifecycle request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLifecycleRequest {
    /// Stable lifecycle id for audit and replay.
    pub id: String,
    /// Stable adapter request id.
    pub adapter_request_id: String,
    /// Draft plan to checkpoint and evaluate.
    pub plan: ExecutionPlanDraft,
    /// Adapter-boundary configuration.
    pub adapter_config: ExecutionAdapterConfig,
    /// Runtime clock in Unix milliseconds.
    pub now_unix_ms: u64,
}

impl RuntimeLifecycleRequest {
    /// Validate local runtime lifecycle boundaries before side effects.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle id is required".to_owned(),
            });
        }
        if self.adapter_request_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "adapter request id is required".to_owned(),
            });
        }
        if self.now_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "now_unix_ms must be non-zero".to_owned(),
            });
        }
        if self.plan.scope == ExecutionScope::Live {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle rejects live-scope plans".to_owned(),
            });
        }
        self.plan
            .validate()
            .map_err(RuntimeLifecycleError::Planner)?;
        self.adapter_config
            .validate()
            .map_err(RuntimeLifecycleError::Adapter)?;
        Ok(())
    }
}

/// Completed local runtime lifecycle record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLifecycleRecord {
    /// Stable lifecycle id.
    pub id: String,
    /// Runtime lifecycle model version.
    pub runtime_lifecycle_version: String,
    /// Source plan id.
    pub plan_id: String,
    /// Source adapter request id.
    pub adapter_request_id: String,
    /// Plan scope.
    pub scope: ExecutionScope,
    /// Completed lifecycle status.
    pub status: RuntimeLifecycleStatus,
    /// Plan checkpoint key.
    pub plan_checkpoint_key: String,
    /// Adapter run checkpoint key.
    pub adapter_run_checkpoint_key: String,
    /// Adapter recovery-plan checkpoint key.
    pub adapter_recovery_plan_checkpoint_key: String,
    /// Audit sequence for lifecycle start.
    pub start_audit_sequence: u64,
    /// Audit sequence for plan checkpoint.
    pub plan_checkpoint_audit_sequence: u64,
    /// Audit sequence for adapter completion.
    pub adapter_complete_audit_sequence: u64,
    /// Audit sequence for adapter recovery-plan persistence.
    pub adapter_recovery_plan_audit_sequence: u64,
    /// Deterministic adapter run.
    pub adapter_run: ExecutionAdapterRunRecord,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Record creation time.
    pub created_at_unix_ms: u64,
    /// Non-secret warnings.
    pub warnings: Vec<String>,
}

/// Local graceful-shutdown checkpoint request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeGracefulShutdownRequest {
    /// Stable shutdown id for audit and replay.
    pub id: String,
    /// Non-secret shutdown reason or operator note.
    pub reason: String,
    /// Runtime clock in Unix milliseconds.
    pub now_unix_ms: u64,
}

impl RuntimeGracefulShutdownRequest {
    /// Validate local graceful-shutdown checkpoint input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown id is required".to_owned(),
            });
        }
        if self.reason.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown reason is required".to_owned(),
            });
        }
        if self.now_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "now_unix_ms must be non-zero".to_owned(),
            });
        }
        Ok(())
    }
}

/// Completed local graceful-shutdown checkpoint record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeGracefulShutdownRecord {
    /// Stable shutdown id.
    pub id: String,
    /// Runtime graceful-shutdown model version.
    pub runtime_graceful_shutdown_version: String,
    /// Shutdown checkpoint key.
    pub shutdown_checkpoint_key: String,
    /// Non-secret checkpoint value persisted to the state store.
    pub shutdown_checkpoint_value: String,
    /// Audit sequence for shutdown start.
    pub shutdown_start_audit_sequence: u64,
    /// Audit sequence for shutdown checkpoint persistence.
    pub shutdown_checkpoint_audit_sequence: u64,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Whether this local record approves production readiness.
    pub production_ready: bool,
    /// Record creation time.
    pub created_at_unix_ms: u64,
    /// Non-secret warnings.
    pub warnings: Vec<String>,
}

/// Non-secret result of a local runtime audit/state backup-restore validation pass.
///
/// The report intentionally stores outcomes only. It does not include local
/// filesystem paths, audit payloads, checkpoint values, database contents,
/// secrets, dependency graphs, or embedded evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeBackupRestoreValidationReport {
    /// Runtime backup/restore validation model version.
    pub validation_version: String,
    /// Number of audit records replayed from the copied journal.
    pub audit_records_replayed: u64,
    /// True when the copied audit journal reopened with the same next sequence.
    pub audit_restore_check_passed: bool,
    /// True when the copied SQLite database reopened and passed integrity check.
    pub sqlite_restore_check_passed: bool,
    /// True when the restored planner checkpoint was present.
    pub plan_checkpoint_restored: bool,
    /// True when the restored adapter-run checkpoint was present.
    pub adapter_checkpoint_restored: bool,
    /// True when the restored adapter recovery-plan checkpoint was present.
    pub adapter_recovery_plan_checkpoint_restored: bool,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Whether this local validation approves production readiness.
    pub production_ready: bool,
}

/// Local restart recovery disposition for operator review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeRestartRecoveryDisposition {
    /// Required runtime checkpoints and graceful-shutdown checkpoint are present.
    ReadyForLocalReview,
    /// Required runtime checkpoints are present, but operator review is needed.
    NeedsOperatorReview,
}

/// Non-secret result of a local runtime restart recovery validation pass.
///
/// This report records only restart/replay outcomes and deliberately omits
/// filesystem paths, audit payloads, checkpoint values, secrets, database
/// contents, deployment metadata, and embedded evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRestartRecoveryValidationReport {
    /// Runtime restart recovery validation model version.
    pub validation_version: String,
    /// Number of audit records replayed from the local journal.
    pub audit_records_replayed: u64,
    /// True when audit replay reopened a non-empty journal.
    pub audit_replay_check_passed: bool,
    /// True when the SQLite store reopened and passed integrity check.
    pub sqlite_reopen_check_passed: bool,
    /// True when the planner checkpoint was present after reopen.
    pub plan_checkpoint_recovered: bool,
    /// True when the adapter-run checkpoint was present after reopen.
    pub adapter_checkpoint_recovered: bool,
    /// True when the adapter recovery-plan checkpoint was present after reopen.
    pub adapter_recovery_plan_checkpoint_recovered: bool,
    /// True when a graceful-shutdown checkpoint was present after reopen.
    pub graceful_shutdown_checkpoint_recovered: bool,
    /// Local recovery disposition for operator review.
    pub recovery_disposition: RuntimeRestartRecoveryDisposition,
    /// True when local lifecycle state is coherent enough for operator review.
    pub local_review_ready: bool,
    /// True when local connector lifecycle checkpoint recovery was validated.
    pub connector_lifecycle_recovery_validated: bool,
    /// True when the latest CEX lifecycle checkpoint was recovered after reopen.
    pub cex_lifecycle_checkpoint_recovered: bool,
    /// True when the latest DEX lifecycle checkpoint was recovered after reopen.
    pub dex_lifecycle_checkpoint_recovered: bool,
    /// Non-secret recovered CEX lifecycle summary available to restart recovery consumers.
    pub recovered_cex_lifecycle: Option<RuntimeRecoveredCexLifecycleSummary>,
    /// Non-secret recovered DEX lifecycle summary available to restart recovery consumers.
    pub recovered_dex_lifecycle: Option<RuntimeRecoveredDexLifecycleSummary>,
    /// True when opportunity trace checkpoint recovery was validated.
    pub opportunity_trace_recovery_validated: bool,
    /// Number of opportunity trace checkpoints discovered by local recovery.
    pub opportunity_trace_discovered_candidates: u64,
    /// Number of opportunity trace checkpoints recovered by local recovery.
    pub opportunity_trace_recovered_checkpoints: u64,
    /// Non-secret recovered opportunity trace summaries available to restart recovery consumers.
    pub opportunity_trace_recovered_summaries: Vec<RuntimeRecoveredOpportunityTraceSummary>,
    /// Number of opportunity trace checkpoints still missing after local recovery.
    pub opportunity_trace_missing_checkpoints: u64,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Optional local opportunity candidate trace recovery summary from a phase-27 corpus replay.
    pub opportunity_trace_recovery: Option<RuntimeOpportunityTraceRecoverySummary>,
    /// Whether this local validation approves production readiness.
    pub production_ready: bool,
}

/// Runtime opportunity trace recovery summary.
///
/// This summary records only local recovered-checkpoint accounting and avoids
/// filesystem paths, audit payloads, checkpoint values, secrets, and embedded
/// evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeOpportunityTraceRecoverySummary {
    /// Local opportunity corpus id used for this trace recovery pass.
    pub corpus_id: String,
    /// Number of discovered trace checkpoints expected.
    pub discovered_candidates: u64,
    /// Number of trace audit records replayed.
    pub audit_trace_records_replayed: u64,
    /// Number of trace checkpoints recovered from local state.
    pub recovered_trace_checkpoints: u64,
    /// Number of trace checkpoints still missing from local state.
    pub missing_trace_checkpoints: u64,
    /// Non-secret summaries for recovered trace checkpoints.
    pub recovered_trace_summaries: Vec<RuntimeRecoveredOpportunityTraceSummary>,
    /// Whether all expected trace recovery checks passed.
    pub trace_recovery_validated: bool,
}

/// Runtime-level non-secret summary of one recovered opportunity trace checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRecoveredOpportunityTraceSummary {
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
    /// Route kind label for the recovered candidate.
    pub route_kind: String,
    /// Number of candidate legs summarized without embedding the full candidate.
    pub leg_count: u64,
}

/// Non-secret recovered CEX lifecycle summary from local restart recovery.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRecoveredCexLifecycleSummary {
    /// Original request id.
    pub request_id: String,
    /// Client order id.
    pub client_order_id: String,
    /// Strategy profile id.
    pub strategy_id: String,
    /// Venue name.
    pub venue_name: String,
    /// Market pair label.
    pub market_pair: String,
    /// Final local/mock lifecycle status label.
    pub final_status: String,
    /// Number of reconciled transitions.
    pub transition_count: u64,
    /// Number of fill-bearing responses.
    pub fill_count: u64,
}

/// Non-secret recovered DEX lifecycle summary from local restart recovery.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRecoveredDexLifecycleSummary {
    /// Original request id.
    pub request_id: String,
    /// Strategy profile id.
    pub strategy_id: String,
    /// Venue name.
    pub venue_name: String,
    /// Chain label.
    pub chain: String,
    /// Market pair label.
    pub market_pair: String,
    /// Quote response id.
    pub quote_response_id: String,
    /// Simulation response id.
    pub simulation_response_id: String,
    /// Route kind label.
    pub route_kind: String,
    /// Final local simulation status label.
    pub simulation_status: String,
    /// Simulated gas used.
    pub gas_used: u64,
}

/// Non-secret result of a local deployment-like runtime smoke validation pass.
///
/// This report intentionally records outcomes only. It does not include paths,
/// audit payloads, checkpoint values, database contents, secrets, deployment
/// metadata, service-manager data, or embedded evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSmokeValidationReport {
    /// Runtime deployment-smoke validation model version.
    pub validation_version: String,
    /// Whether one local runtime lifecycle completed.
    pub lifecycle_completed: bool,
    /// Whether graceful-shutdown audit/state checkpointing completed.
    pub graceful_shutdown_checkpointed: bool,
    /// Whether local backup/restore validation completed.
    pub backup_restore_validated: bool,
    /// Whether restart recovery validation completed.
    pub restart_recovery_validated: bool,
    /// Whether local audit durability probes completed.
    pub audit_durability_validated: bool,
    /// Whether concurrent local lifecycle access over shared audit/SQLite paths completed.
    pub concurrent_lifecycle_validated: bool,
    /// Number of concurrent local lifecycle workers that completed.
    pub concurrent_lifecycle_workers: u64,
    /// Number of audit records replayed after concurrent lifecycle access.
    pub concurrent_lifecycle_audit_records_replayed: u64,
    /// Whether SQLite integrity passed after concurrent lifecycle access.
    pub concurrent_lifecycle_sqlite_integrity_check_passed: bool,
    /// Whether concurrent lifecycle access submitted an external adapter. Always false here.
    pub concurrent_lifecycle_external_submission_performed: bool,
    /// Whether concurrent lifecycle access performed live execution. Always false here.
    pub concurrent_lifecycle_live_execution_performed: bool,
    /// Whether local in-process observability collection completed.
    pub observability_collected: bool,
    /// Whether the observability checkpoint was recovered after SQLite reopen.
    pub observability_checkpoint_recovered: bool,
    /// Whether local observability operations controls were reviewed.
    pub observability_operations_reviewed: bool,
    /// Whether the operations-review checkpoint was recovered after SQLite reopen.
    pub observability_operations_checkpoint_recovered: bool,
    /// Whether local metrics/export dry-run rendering completed.
    pub observability_export_dry_run_rendered: bool,
    /// Whether the export dry-run checkpoint was recovered after SQLite reopen.
    pub observability_export_checkpoint_recovered: bool,
    /// Whether local alert-route dispatch reached the communications boundary.
    pub observability_alert_route_dispatched: bool,
    /// Whether the alert-route dispatch checkpoint was recovered after SQLite reopen.
    pub observability_alert_route_checkpoint_recovered: bool,
    /// Whether endpoint/exporter controls were locally preflighted.
    pub observability_endpoint_preflighted: bool,
    /// Whether the endpoint/exporter preflight checkpoint was recovered.
    pub observability_endpoint_checkpoint_recovered: bool,
    /// Whether an ephemeral loopback bind opened and closed locally.
    pub observability_loopback_bind_validated: bool,
    /// Whether the loopback-bind checkpoint was recovered.
    pub observability_loopback_bind_checkpoint_recovered: bool,
    /// Whether authenticated metrics scrape preflight completed without a socket listener.
    pub observability_metrics_scrape_preflighted: bool,
    /// Whether the metrics-scrape preflight checkpoint was recovered.
    pub observability_metrics_scrape_checkpoint_recovered: bool,
    /// Whether one authenticated loopback metrics request was locally served.
    pub observability_metrics_endpoint_validated: bool,
    /// Whether the metrics-endpoint validation checkpoint was recovered.
    pub observability_metrics_endpoint_checkpoint_recovered: bool,
    /// Whether scoped local tracing subscriber capture completed.
    pub observability_tracing_captured: bool,
    /// Whether the tracing-subscriber checkpoint was recovered.
    pub observability_tracing_checkpoint_recovered: bool,
    /// Whether the observability record started a metrics endpoint. Always false here.
    pub observability_metrics_endpoint_started: bool,
    /// Whether the bounded local one-shot metrics endpoint served one request.
    pub observability_local_metrics_request_served: bool,
    /// Whether the observability record exposed a public network binding. Always false here.
    pub observability_public_network_exposed: bool,
    /// Whether the observability record sent outbound alerts. Always false here.
    pub observability_outbound_alerts_sent: bool,
    /// Whether any telemetry export occurred. Always false here.
    pub observability_telemetry_exported: bool,
    /// Whether observability checks claimed production readiness. Always false here.
    pub observability_production_ready: bool,
    /// Whether local communications command routing completed.
    pub communications_command_routed: bool,
    /// Whether the communications command-route checkpoint was recovered after SQLite reopen.
    pub communications_command_route_checkpoint_recovered: bool,
    /// Whether local remote-command security review completed.
    pub communications_remote_command_reviewed: bool,
    /// Whether the remote-command security-review checkpoint was recovered after SQLite reopen.
    pub communications_remote_command_review_checkpoint_recovered: bool,
    /// Whether local mocked platform command-ingress validation completed.
    pub communications_platform_command_ingress_validated: bool,
    /// Whether the platform command-ingress checkpoint was recovered after SQLite reopen.
    pub communications_platform_command_ingress_checkpoint_recovered: bool,
    /// Whether local remote-command envelope validation completed without enabling routing.
    pub communications_remote_command_envelope_validated: bool,
    /// Whether the remote-command envelope checkpoint was recovered after SQLite reopen.
    pub communications_remote_command_envelope_checkpoint_recovered: bool,
    /// Whether local authenticated channel-adapter validation completed without delivery.
    pub communications_channel_adapter_validated: bool,
    /// Whether the channel-adapter validation checkpoint was recovered after SQLite reopen.
    pub communications_channel_adapter_checkpoint_recovered: bool,
    /// Whether local channel-session validation completed.
    pub communications_channel_session_validated: bool,
    /// Whether the channel-session validation checkpoint was recovered after SQLite reopen.
    pub communications_channel_session_checkpoint_recovered: bool,
    /// Whether local platform-adapter controls were reviewed.
    pub communications_platform_adapter_reviewed: bool,
    /// Whether the platform-adapter review checkpoint was recovered after SQLite reopen.
    pub communications_platform_adapter_checkpoint_recovered: bool,
    /// Whether local communications notification dispatch completed.
    pub communications_notification_dispatched: bool,
    /// Whether the communications notification-dispatch checkpoint was recovered after SQLite reopen.
    pub communications_notification_checkpoint_recovered: bool,
    /// Whether communications routing enabled command execution. Always false here.
    pub communications_execution_enabled: bool,
    /// Whether communications remote commands were enabled. Always false here.
    pub communications_remote_commands_enabled: bool,
    /// Whether communications dispatch used outbound network delivery. Always false here.
    pub communications_outbound_network_used: bool,
    /// Whether local dashboard rendering completed.
    pub dashboard_rendered: bool,
    /// Whether the dashboard render checkpoint was recovered after SQLite reopen.
    pub dashboard_checkpoint_recovered: bool,
    /// Whether hosted-dashboard security controls were locally reviewed.
    pub dashboard_hosted_security_reviewed: bool,
    /// Whether the hosted-dashboard security review checkpoint was recovered.
    pub dashboard_hosted_security_checkpoint_recovered: bool,
    /// Whether hosted-dashboard request controls were locally preflighted.
    pub dashboard_hosted_request_preflighted: bool,
    /// Whether the hosted-dashboard request preflight checkpoint was recovered.
    pub dashboard_hosted_request_preflight_checkpoint_recovered: bool,
    /// Whether one authenticated loopback hosted-dashboard request was locally validated.
    pub dashboard_hosted_request_validated: bool,
    /// Whether the hosted-dashboard request validation checkpoint was recovered.
    pub dashboard_hosted_request_validation_checkpoint_recovered: bool,
    /// Number of local dashboard panels rendered.
    pub dashboard_panel_count: u64,
    /// Whether the dashboard render started a server. Always false here.
    pub dashboard_server_started: bool,
    /// Whether the bounded hosted-dashboard one-shot listener served one loopback request.
    pub dashboard_local_one_shot_request_served: bool,
    /// Whether the dashboard render exposed public network bindings. Always false here.
    pub dashboard_public_network_exposed: bool,
    /// Whether dashboard live controls were enabled. Always false here.
    pub dashboard_live_controls_enabled: bool,
    /// Whether hosted-dashboard review/validation claimed production readiness. Always false here.
    pub dashboard_hosted_production_ready: bool,
    /// Whether local validation-run planning completed.
    pub validation_run_recorded: bool,
    /// Whether the validation-run checkpoint was recovered after SQLite reopen.
    pub validation_run_checkpoint_recovered: bool,
    /// Whether deterministic local property checks completed.
    pub validation_property_checks_passed: bool,
    /// Whether the property-check checkpoint was recovered after SQLite reopen.
    pub validation_property_checkpoint_recovered: bool,
    /// Whether the validation boundary invoked an external fuzzer. Always false here.
    pub validation_external_fuzzer_invoked: bool,
    /// Whether the validation boundary used live network access. Always false here.
    pub validation_live_network_used: bool,
    /// Whether the validation boundary submitted live execution. Always false here.
    pub validation_live_execution_submitted: bool,
    /// Whether the validation boundary signed or broadcast anything. Always false here.
    pub validation_signing_or_broadcast_performed: bool,
    /// Whether paper ledgering applied to this smoke pass.
    pub paper_ledger_applicable: bool,
    /// Whether a local paper execution report checkpoint was persisted.
    pub paper_execution_report_checkpointed: bool,
    /// Whether the paper execution report checkpoint was recovered after SQLite reopen.
    pub paper_execution_report_checkpoint_recovered: bool,
    /// Whether local paper ledger settlement checkpointing completed.
    pub paper_ledger_checkpointed: bool,
    /// Whether the paper ledger checkpoint was recovered after SQLite reopen.
    pub paper_ledger_checkpoint_recovered: bool,
    /// Number of modeled adapter fills settled into the paper ledger.
    pub paper_modeled_fills_settled: u64,
    /// Number of paper ledger audit records appended.
    pub paper_ledger_audit_records_appended: u64,
    /// Whether paper ledger replay validation passed after settlement.
    pub paper_ledger_replay_validated: bool,
    /// Whether paper ledgering submitted externally. Always false here.
    pub paper_ledger_external_submission_performed: bool,
    /// Whether paper ledgering performed live execution. Always false here.
    pub paper_ledger_live_execution_performed: bool,
    /// Whether local runtime failure-capture probe completed.
    pub failure_capture_validated: bool,
    /// Whether the runtime failure-capture checkpoint was recovered after SQLite reopen.
    pub failure_capture_checkpoint_recovered: bool,
    /// Whether the failure-capture probe started a metrics endpoint. Always false here.
    pub failure_capture_metrics_endpoint_started: bool,
    /// Whether the failure-capture probe exposed a public network binding. Always false here.
    pub failure_capture_public_network_exposed: bool,
    /// Whether the failure-capture probe sent outbound alerts. Always false here.
    pub failure_capture_outbound_alerts_sent: bool,
    /// Whether the failure-capture probe submitted an external adapter. Always false here.
    pub failure_capture_external_submission_performed: bool,
    /// Whether the failure-capture probe performed live execution. Always false here.
    pub failure_capture_live_execution_performed: bool,
    /// Number of audit records replayed by restart recovery.
    pub restart_audit_records_replayed: u64,
    /// Number of audit records replayed from backup/restore validation.
    pub backup_audit_records_replayed: u64,
    /// Whether plan checkpoint recovery was observed during restart recovery.
    pub restart_plan_checkpoint_recovered: bool,
    /// Whether adapter checkpoint recovery was observed during restart recovery.
    pub restart_adapter_checkpoint_recovered: bool,
    /// Whether adapter recovery-plan checkpoint recovery was observed during restart recovery.
    pub restart_adapter_recovery_plan_checkpoint_recovered: bool,
    /// Whether graceful-shutdown checkpoint recovery was observed during restart recovery.
    pub restart_graceful_shutdown_checkpoint_recovered: bool,
    /// Whether opportunity trace checkpoint recovery was observed during restart recovery.
    pub restart_opportunity_trace_recovery_validated: bool,
    /// Number of opportunity trace checkpoints discovered by restart recovery.
    pub restart_opportunity_trace_discovered_candidates: u64,
    /// Number of opportunity trace checkpoints recovered by restart recovery.
    pub restart_opportunity_trace_recovered_checkpoints: u64,
    /// Non-secret recovered opportunity trace summaries observed during restart recovery.
    pub restart_opportunity_trace_recovered_summaries: Vec<RuntimeRecoveredOpportunityTraceSummary>,
    /// Number of opportunity trace checkpoints still missing after restart recovery.
    pub restart_opportunity_trace_missing_checkpoints: u64,
    /// Opportunity trace recovery summary from a phase-27 local corpus replay.
    pub opportunity_trace_recovery: Option<RuntimeOpportunityTraceRecoverySummary>,
    /// Local recovery disposition for operator review.
    pub recovery_disposition: RuntimeRestartRecoveryDisposition,
    /// Whether any service manager action was performed. Always false here.
    pub service_manager_action_performed: bool,
    /// Whether any external adapter was submitted to. Always false here.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local validation approves production readiness.
    pub production_ready: bool,
    /// Remaining blockers in non-secret wording.
    pub unresolved_blockers: Vec<String>,
}

/// One measured local deployment-like runtime smoke iteration.
///
/// This record carries only a caller-supplied iteration label, elapsed wall
/// time in milliseconds, and the already-sanitized smoke report. It deliberately
/// avoids filesystem paths, audit payloads, checkpoint contents, service-manager
/// data, secrets, or embedded artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSmokeLoadIteration {
    /// Local iteration identifier.
    pub iteration_id: String,
    /// Measured local wall-clock elapsed time in milliseconds.
    pub elapsed_ms: u64,
    /// Sanitized local smoke report for this iteration.
    pub report: RuntimeDeploymentSmokeValidationReport,
}

/// Aggregate non-secret load/latency summary for repeated local runtime smoke passes.
///
/// This is local validation evidence only. It does not start services, inspect
/// deployment state, submit external adapters, perform live execution, or claim
/// production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSmokeLoadValidationReport {
    /// Runtime deployment-smoke validation model version.
    pub validation_version: String,
    /// Number of local smoke iterations included in the aggregate.
    pub iterations_attempted: u64,
    /// Number of local smoke iterations whose reports passed validation.
    pub iterations_passed: u64,
    /// Fastest local smoke iteration wall-clock duration.
    pub min_elapsed_ms: u64,
    /// Slowest local smoke iteration wall-clock duration.
    pub max_elapsed_ms: u64,
    /// Integer average local smoke iteration wall-clock duration.
    pub average_elapsed_ms: u64,
    /// Total local smoke iteration wall-clock duration.
    pub total_elapsed_ms: u64,
    /// Total restart audit records replayed across iterations.
    pub restart_audit_records_replayed: u64,
    /// Total backup audit records replayed across iterations.
    pub backup_audit_records_replayed: u64,
    /// Total recovered opportunity trace checkpoints across iterations.
    pub opportunity_trace_recovered_checkpoints: u64,
    /// Total recovered opportunity trace summaries across iterations.
    pub opportunity_trace_recovered_summaries: u64,
    /// Total missing opportunity trace checkpoints across iterations.
    pub opportunity_trace_missing_checkpoints: u64,
    /// Whether any service-manager action was performed. Always false here.
    pub service_manager_action_performed: bool,
    /// Whether any external adapter submission was performed. Always false here.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this aggregate approves production readiness. Always false here.
    pub production_ready: bool,
    /// Remaining blockers in non-secret wording.
    pub unresolved_blockers: Vec<String>,
}

/// Local runtime load profile review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeLoadProfileReviewStatus {
    /// Local load evidence meets the supplied local review budgets.
    ReadyForLocalReview,
    /// Local load evidence is missing, unsafe, or exceeds the supplied local budgets.
    Blocked,
}

/// Local runtime load profile review request.
///
/// This consumes sanitized local runtime-smoke load evidence and caller-supplied
/// local budget/resource observations. It does not execute benchmarks, inspect
/// host resources, start services, call providers, submit adapters, perform live
/// execution, or claim production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLoadProfileReviewRequest {
    /// Stable review id.
    pub review_id: String,
    /// Sanitized local runtime smoke/load report.
    pub load_report: RuntimeDeploymentSmokeLoadValidationReport,
    /// Maximum allowed local average iteration duration.
    pub max_average_elapsed_ms: u64,
    /// Maximum allowed local single-iteration duration.
    pub max_single_iteration_elapsed_ms: u64,
    /// Maximum allowed local total duration.
    pub max_total_elapsed_ms: u64,
    /// Observed local peak memory estimate in MiB.
    pub observed_peak_memory_mb: u64,
    /// Maximum allowed local peak memory estimate in MiB.
    pub max_peak_memory_mb: u64,
    /// Observed local peak CPU estimate in percent.
    pub observed_peak_cpu_percent: u64,
    /// Maximum allowed local peak CPU estimate in percent.
    pub max_peak_cpu_percent: u64,
    /// Whether deployment-host load evidence is available.
    pub deployment_host_load_evidence_available: bool,
    /// Whether live/provider feed backpressure evidence is available.
    pub live_feed_backpressure_evidence_available: bool,
    /// Whether ARM or target-class runtime evidence is available.
    pub target_runtime_evidence_available: bool,
    /// Whether this review performed service-manager actions.
    pub service_manager_action_performed: bool,
    /// Whether this review performed external calls.
    pub external_calls_performed: bool,
    /// Whether this review performed live execution.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness.
    pub production_ready_claimed: bool,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local runtime load profile review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLoadProfileReviewReport {
    /// Runtime load profile review version.
    pub validation_version: String,
    /// Stable review id.
    pub review_id: String,
    /// Validation status.
    pub status: RuntimeLoadProfileReviewStatus,
    /// Number of local smoke/load iterations reviewed.
    pub iterations_reviewed: u64,
    /// Whether local latency budgets were met.
    pub latency_budget_met: bool,
    /// Whether local resource budgets were met.
    pub resource_budget_met: bool,
    /// Whether local replay/recovery evidence is coherent.
    pub replay_recovery_evidence_validated: bool,
    /// Maximum observed local average iteration duration.
    pub observed_average_elapsed_ms: u64,
    /// Maximum observed local single iteration duration.
    pub observed_max_elapsed_ms: u64,
    /// Observed local total duration.
    pub observed_total_elapsed_ms: u64,
    /// Observed local peak memory estimate in MiB.
    pub observed_peak_memory_mb: u64,
    /// Observed local peak CPU estimate in percent.
    pub observed_peak_cpu_percent: u64,
    /// Whether deployment-host load evidence is available.
    pub deployment_host_load_evidence_available: bool,
    /// Whether live/provider feed backpressure evidence is available.
    pub live_feed_backpressure_evidence_available: bool,
    /// Whether ARM or target-class runtime evidence is available.
    pub target_runtime_evidence_available: bool,
    /// Local blocker codes.
    pub blocker_codes: Vec<String>,
    /// Remaining external evidence required before production performance claims.
    pub remaining_external_evidence: Vec<String>,
    /// Whether this review performed service-manager actions. Always false.
    pub service_manager_action_performed: bool,
    /// Whether this review performed external calls. Always false.
    pub external_calls_performed: bool,
    /// Whether this review performed live execution. Always false.
    pub live_execution_performed: bool,
    /// Whether this review approves production readiness. Always false.
    pub production_ready: bool,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Local production-runtime validation preflight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeProductionPreflightStatus {
    /// Local smoke evidence is coherent, but production-host validation is still missing.
    BlockedPendingProductionHostValidation,
}

/// Inputs for the local production-runtime validation preflight.
///
/// This request contains sanitized local reports and operator-supplied booleans
/// only. It must not include service-manager logs, host paths, artifact
/// contents, credentials, audit payloads, checkpoint values, or deployment
/// secrets.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProductionPreflightRequest {
    /// Stable preflight id.
    pub preflight_id: String,
    /// Latest local runtime smoke report.
    pub smoke_report: RuntimeDeploymentSmokeValidationReport,
    /// Aggregate local runtime smoke/load report.
    pub load_report: RuntimeDeploymentSmokeLoadValidationReport,
    /// Whether deployment-host service-manager lifecycle evidence is available.
    pub service_manager_lifecycle_evidence_available: bool,
    /// Whether deployment-host filesystem permission evidence is available.
    pub deployment_host_permission_evidence_available: bool,
    /// Whether physical deployment-host disk-full evidence is available.
    pub physical_disk_full_evidence_available: bool,
    /// Whether deployment-host retention/rotation execution evidence is available.
    pub retention_execution_evidence_available: bool,
    /// Whether executed rollback-drill evidence is available.
    pub rollback_drill_evidence_available: bool,
    /// Whether executed incident-response drill evidence is available.
    pub incident_response_evidence_available: bool,
    /// Whether real observability/exporter/alert runtime evidence is available.
    pub observability_runtime_evidence_available: bool,
    /// Whether any service-manager action was performed by this preflight.
    pub service_manager_action_performed: bool,
    /// Whether any external submission was performed by this preflight.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed by this preflight.
    pub live_execution_performed: bool,
    /// Whether this request attempts to claim production readiness.
    pub production_ready_claimed: bool,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local production-runtime validation preflight report.
///
/// The report deliberately separates current local smoke evidence from
/// production-host evidence that still must be gathered outside this local
/// boundary. It never approves production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProductionPreflightReport {
    /// Runtime production-preflight validation model version.
    pub validation_version: String,
    /// Stable preflight id.
    pub preflight_id: String,
    /// Whether the local runtime smoke report passed invariant validation.
    pub local_smoke_validated: bool,
    /// Whether the local runtime smoke load aggregate passed invariant validation.
    pub local_smoke_load_validated: bool,
    /// Whether deployment-host service-manager lifecycle evidence is available.
    pub service_manager_lifecycle_evidence_available: bool,
    /// Whether deployment-host filesystem permission evidence is available.
    pub deployment_host_permission_evidence_available: bool,
    /// Whether physical deployment-host disk-full evidence is available.
    pub physical_disk_full_evidence_available: bool,
    /// Whether deployment-host retention/rotation execution evidence is available.
    pub retention_execution_evidence_available: bool,
    /// Whether executed rollback-drill evidence is available.
    pub rollback_drill_evidence_available: bool,
    /// Whether executed incident-response drill evidence is available.
    pub incident_response_evidence_available: bool,
    /// Whether real observability/exporter/alert runtime evidence is available.
    pub observability_runtime_evidence_available: bool,
    /// Local preflight status.
    pub status: RuntimeProductionPreflightStatus,
    /// Whether any service-manager action was performed. Always false here.
    pub service_manager_action_performed: bool,
    /// Whether any external adapter was submitted to. Always false here.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local preflight approves production readiness. Always false here.
    pub production_ready: bool,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
    /// Remaining blockers in non-secret wording.
    pub unresolved_blockers: Vec<String>,
}

/// Service-manager family represented by sanitized lifecycle transcript metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeServiceManagerKind {
    /// systemd-shaped unit lifecycle evidence.
    Systemd,
    /// Other operator-controlled service manager.
    Other,
}

/// Sanitized service-manager lifecycle event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeServiceManagerLifecycleEventKind {
    /// Unit/service was loaded or discovered.
    UnitLoaded,
    /// Service start was observed.
    Started,
    /// Runtime smoke passed after service start.
    RuntimeSmokePassed,
    /// Graceful shutdown was requested or observed.
    GracefulShutdownRequested,
    /// Service stop was observed.
    Stopped,
    /// Service restart was observed.
    Restarted,
    /// Runtime recovery was validated after restart.
    RecoveryValidated,
}

/// Local validation status for sanitized service-manager lifecycle transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeServiceManagerLifecycleTranscriptStatus {
    /// Transcript contains all required lifecycle evidence references.
    ReadyForExternalReview,
    /// Transcript is missing required lifecycle evidence or contains unsafe flags.
    Blocked,
}

/// One sanitized service-manager lifecycle event.
///
/// This contains only event labels and reference presence flags. It must not
/// embed service logs, host paths, environment values, credentials, command
/// output, audit payloads, or checkpoint values.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeServiceManagerLifecycleEvent {
    /// Stable local event id.
    pub event_id: String,
    /// Lifecycle event kind.
    pub kind: RuntimeServiceManagerLifecycleEventKind,
    /// Event observation time in Unix milliseconds.
    pub observed_at_unix_ms: u64,
    /// Whether the event came from an operator-controlled service lifecycle action.
    pub operator_controlled: bool,
    /// Whether a non-secret external evidence reference exists for this event.
    pub non_secret_reference_present: bool,
    /// Whether the event outcome was successful.
    pub outcome_success: bool,
}

/// Sanitized local service-manager lifecycle transcript validation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeServiceManagerLifecycleTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Service manager family.
    pub service_manager: RuntimeServiceManagerKind,
    /// Unit or service name.
    pub unit_name: String,
    /// Sanitized lifecycle events.
    pub events: Vec<RuntimeServiceManagerLifecycleEvent>,
    /// Whether audit replay evidence reference is present.
    pub audit_replay_reference_present: bool,
    /// Whether SQLite recovery evidence reference is present.
    pub sqlite_recovery_reference_present: bool,
    /// Whether runtime smoke evidence reference is present.
    pub runtime_smoke_reference_present: bool,
    /// Whether service-manager-controlled concurrent lifecycle evidence is present.
    pub concurrent_lifecycle_reference_present: bool,
    /// Number of concurrent lifecycle workers covered by the external reference.
    pub concurrent_lifecycle_worker_count: u32,
    /// Whether the referenced concurrent lifecycle run completed successfully.
    pub concurrent_lifecycle_success: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether an operator lifecycle rehearsal reference is present.
    pub operator_lifecycle_rehearsal_reference_present: bool,
    /// Whether an emergency-stop or kill-switch review reference is present.
    pub emergency_stop_review_reference_present: bool,
    /// Whether a rollback-plan review reference is present.
    pub rollback_plan_review_reference_present: bool,
    /// Whether the operator review window is current.
    pub operator_review_window_current: bool,
    /// Whether this validator performed a service-manager action. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for a service-manager lifecycle transcript.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeServiceManagerLifecycleTranscriptReport {
    /// Service-manager transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Service manager family.
    pub service_manager: RuntimeServiceManagerKind,
    /// Unit or service name.
    pub unit_name: String,
    /// Number of lifecycle events reviewed.
    pub event_count: u64,
    /// Whether start evidence was present.
    pub start_evidence_present: bool,
    /// Whether runtime-smoke evidence was present.
    pub runtime_smoke_evidence_present: bool,
    /// Whether graceful-shutdown evidence was present.
    pub graceful_shutdown_evidence_present: bool,
    /// Whether stop evidence was present.
    pub stop_evidence_present: bool,
    /// Whether restart evidence was present.
    pub restart_evidence_present: bool,
    /// Whether recovery evidence was present.
    pub recovery_evidence_present: bool,
    /// Whether every lifecycle event is operator-controlled.
    pub operator_controlled_events: bool,
    /// Whether every lifecycle event has a non-secret reference.
    pub non_secret_references_present: bool,
    /// Whether all lifecycle event outcomes succeeded.
    pub successful_event_outcomes: bool,
    /// Whether audit replay evidence reference is present.
    pub audit_replay_reference_present: bool,
    /// Whether SQLite recovery evidence reference is present.
    pub sqlite_recovery_reference_present: bool,
    /// Whether service-manager-controlled concurrent lifecycle evidence is present.
    pub concurrent_lifecycle_reference_present: bool,
    /// Number of concurrent lifecycle workers covered by the external reference.
    pub concurrent_lifecycle_worker_count: u32,
    /// Whether the referenced concurrent lifecycle run completed successfully.
    pub concurrent_lifecycle_success: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether an operator lifecycle rehearsal reference is present.
    pub operator_lifecycle_rehearsal_reference_present: bool,
    /// Whether an emergency-stop or kill-switch review reference is present.
    pub emergency_stop_review_reference_present: bool,
    /// Whether a rollback-plan review reference is present.
    pub rollback_plan_review_reference_present: bool,
    /// Whether the operator review window is current.
    pub operator_review_window_current: bool,
    /// Validation status.
    pub status: RuntimeServiceManagerLifecycleTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator performed a service-manager action. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Local validation status for service-manager lifecycle rehearsals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeServiceManagerLifecycleRehearsalStatus {
    /// The local rehearsal proves ordered lifecycle evidence without side effects.
    Validated,
    /// The local rehearsal is missing required lifecycle evidence or contains unsafe flags.
    Blocked,
}

/// Local-only service-manager lifecycle rehearsal validation request.
///
/// This model proves ordered lifecycle evidence over sanitized event metadata.
/// It must not run `systemctl`, start/stop/restart real services, mutate
/// deployment paths, read service logs, load secrets, submit adapters, or claim
/// production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeServiceManagerLifecycleRehearsalRequest {
    /// Stable rehearsal id.
    pub rehearsal_id: String,
    /// Service manager family being modeled.
    pub service_manager: RuntimeServiceManagerKind,
    /// Unit or service name.
    pub unit_name: String,
    /// Sanitized ordered lifecycle events.
    pub events: Vec<RuntimeServiceManagerLifecycleEvent>,
    /// Whether audit replay evidence reference is present.
    pub audit_replay_reference_present: bool,
    /// Whether SQLite recovery evidence reference is present.
    pub sqlite_recovery_reference_present: bool,
    /// Whether runtime smoke evidence reference is present.
    pub runtime_smoke_reference_present: bool,
    /// Whether concurrent lifecycle evidence reference is present.
    pub concurrent_lifecycle_reference_present: bool,
    /// Number of concurrent lifecycle workers covered by the reference.
    pub concurrent_lifecycle_worker_count: u32,
    /// Whether the referenced concurrent lifecycle run completed successfully.
    pub concurrent_lifecycle_success: bool,
    /// Whether a graceful-shutdown checkpoint reference is present.
    pub graceful_shutdown_checkpoint_reference_present: bool,
    /// Whether restart recovery reference is present.
    pub restart_recovery_reference_present: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Whether this validator performed a service-manager action. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Must be false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Must be false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this request attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Rehearsal validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for a service-manager lifecycle rehearsal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeServiceManagerLifecycleRehearsalReport {
    /// Service-manager lifecycle rehearsal validation version.
    pub validation_version: String,
    /// Stable rehearsal id.
    pub rehearsal_id: String,
    /// Service manager family being modeled.
    pub service_manager: RuntimeServiceManagerKind,
    /// Unit or service name.
    pub unit_name: String,
    /// Number of lifecycle events reviewed.
    pub event_count: u64,
    /// Whether events appear in the required local lifecycle order.
    pub ordered_lifecycle_validated: bool,
    /// Whether every lifecycle event is operator-controlled.
    pub operator_controlled_events: bool,
    /// Whether every lifecycle event has a non-secret reference.
    pub non_secret_references_present: bool,
    /// Whether all lifecycle event outcomes succeeded.
    pub successful_event_outcomes: bool,
    /// Whether start evidence was present.
    pub start_evidence_present: bool,
    /// Whether runtime smoke evidence was present.
    pub runtime_smoke_evidence_present: bool,
    /// Whether graceful-shutdown evidence was present.
    pub graceful_shutdown_evidence_present: bool,
    /// Whether stop evidence was present.
    pub stop_evidence_present: bool,
    /// Whether restart evidence was present.
    pub restart_evidence_present: bool,
    /// Whether recovery evidence was present.
    pub recovery_evidence_present: bool,
    /// Whether audit replay evidence reference is present.
    pub audit_replay_reference_present: bool,
    /// Whether SQLite recovery evidence reference is present.
    pub sqlite_recovery_reference_present: bool,
    /// Whether concurrent lifecycle evidence reference is present.
    pub concurrent_lifecycle_reference_present: bool,
    /// Number of concurrent lifecycle workers covered by the reference.
    pub concurrent_lifecycle_worker_count: u32,
    /// Whether the referenced concurrent lifecycle run completed successfully.
    pub concurrent_lifecycle_success: bool,
    /// Whether graceful-shutdown checkpoint reference is present.
    pub graceful_shutdown_checkpoint_reference_present: bool,
    /// Whether restart recovery reference is present.
    pub restart_recovery_reference_present: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Validation status.
    pub status: RuntimeServiceManagerLifecycleRehearsalStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator performed a service-manager action. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Always false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Always false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Rehearsal validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Local validation status for sanitized deployment-host permission transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeDeploymentPermissionTranscriptStatus {
    /// Transcript contains all required permission-denial evidence references.
    ReadyForExternalReview,
    /// Transcript is missing required permission-denial evidence or contains unsafe flags.
    Blocked,
}

/// Local validation status for sanitized deployment-host audit/SQLite transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeDeploymentAuditSqliteTranscriptStatus {
    /// Transcript contains all required deployment-host audit and SQLite recovery evidence references.
    ReadyForExternalReview,
    /// Transcript is missing deployment-host audit/SQLite evidence or contains unsafe flags.
    Blocked,
}

/// Local validation status for sanitized deployment-host backup/restore transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeDeploymentBackupRestoreTranscriptStatus {
    /// Transcript contains all required deployment-host backup/restore evidence references.
    ReadyForExternalReview,
    /// Transcript is missing backup/restore evidence or contains unsafe flags.
    Blocked,
}

/// Local validation status for sanitized deployment-host graceful-shutdown transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeDeploymentGracefulShutdownTranscriptStatus {
    /// Transcript contains all required graceful-shutdown execution evidence references.
    ReadyForExternalReview,
    /// Transcript is missing graceful-shutdown evidence or contains unsafe flags.
    Blocked,
}

/// Local validation status for sanitized deployment-host SQLite schema migration transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus {
    /// Transcript contains all required schema migration execution evidence references.
    ReadyForExternalReview,
    /// Transcript is missing schema migration evidence or contains unsafe flags.
    Blocked,
}

/// Sanitized local deployment-host permission-denial validation request.
///
/// This contains only reference presence and outcome flags. It must not embed
/// host paths, permission bits, command output, logs, audit payloads, checkpoint
/// values, secrets, or evidence artifact contents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentPermissionTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether evidence came from a physical/deployment-like host.
    pub deployment_host_evidence: bool,
    /// Whether a runtime write attempt evidence reference is present.
    pub runtime_write_attempt_reference_present: bool,
    /// Whether the referenced runtime write was denied by filesystem permissions.
    pub runtime_write_permission_denied: bool,
    /// Whether the runtime write failure was classified as a permission denial.
    pub runtime_write_error_classified: bool,
    /// Whether audit append failed closed under permission denial.
    pub audit_write_failed_closed: bool,
    /// Whether SQLite/state writes failed closed under permission denial.
    pub state_write_failed_closed: bool,
    /// Whether adapter evaluation was blocked before side effects.
    pub adapter_evaluation_blocked: bool,
    /// Whether runtime quiesced or degraded without live execution.
    pub runtime_quiesced_or_degraded: bool,
    /// Whether audit replay was validated after permission restoration.
    pub audit_replay_after_restore_validated: bool,
    /// Whether SQLite reopen/integrity was validated after permission restoration.
    pub sqlite_reopen_after_restore_validated: bool,
    /// Whether recovery/runbook reference is present.
    pub recovery_runbook_reference_present: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether this validator changed permissions. Must be false.
    pub permission_changed_by_validator: bool,
    /// Whether this validator mutated production paths. Must be false.
    pub production_path_mutated_by_validator: bool,
    /// Whether this validator performed a service-manager action. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for deployment-host permission evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentPermissionTranscriptReport {
    /// Deployment permission transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether deployment-like host evidence is present.
    pub deployment_host_evidence: bool,
    /// Whether a runtime write attempt evidence reference is present.
    pub runtime_write_attempt_reference_present: bool,
    /// Whether the referenced runtime write was denied by filesystem permissions.
    pub runtime_write_permission_denied: bool,
    /// Whether the runtime write failure was classified as a permission denial.
    pub runtime_write_error_classified: bool,
    /// Whether audit write fail-closed evidence is present.
    pub audit_write_failed_closed: bool,
    /// Whether state write fail-closed evidence is present.
    pub state_write_failed_closed: bool,
    /// Whether adapter evaluation was blocked before side effects.
    pub adapter_evaluation_blocked: bool,
    /// Whether runtime quiesce/degrade evidence is present.
    pub runtime_quiesced_or_degraded: bool,
    /// Whether audit replay after restore evidence is present.
    pub audit_replay_after_restore_validated: bool,
    /// Whether SQLite recovery after restore evidence is present.
    pub sqlite_reopen_after_restore_validated: bool,
    /// Whether recovery/runbook reference is present.
    pub recovery_runbook_reference_present: bool,
    /// Count of non-secret references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Validation status.
    pub status: RuntimeDeploymentPermissionTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator changed permissions. Always false.
    pub permission_changed_by_validator: bool,
    /// Whether this validator mutated production paths. Always false.
    pub production_path_mutated_by_validator: bool,
    /// Whether this validator performed a service-manager action. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Sanitized deployment-host audit and SQLite recovery validation request.
///
/// This contains only reference presence and outcome flags. It must not embed
/// host paths, audit payloads, checkpoint values, logs, command output, secrets,
/// or evidence artifact contents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentAuditSqliteTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether evidence came from a physical/deployment-like host.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// Whether audit journal append evidence is present.
    pub audit_append_reference_present: bool,
    /// Whether audit replay evidence is present.
    pub audit_replay_validated: bool,
    /// Whether audit hash-chain continuity evidence is present.
    pub audit_hash_chain_validated: bool,
    /// Whether SQLite WAL mode evidence is present.
    pub sqlite_wal_mode_validated: bool,
    /// Whether SQLite integrity check evidence is present.
    pub sqlite_integrity_check_passed: bool,
    /// Whether SQLite checkpoint recovery evidence is present.
    pub sqlite_checkpoint_recovered: bool,
    /// Whether backup/restore evidence is present.
    pub backup_restore_validated: bool,
    /// Whether concurrent runtime access evidence is present.
    pub concurrent_access_validated: bool,
    /// Whether recovery/runbook reference is present.
    pub recovery_runbook_reference_present: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Whether this validator performed service-manager actions. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Must be false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Must be false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for deployment-host audit and SQLite evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentAuditSqliteTranscriptReport {
    /// Deployment audit/SQLite transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether deployment-like host evidence is present.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// Whether audit journal append evidence is present.
    pub audit_append_reference_present: bool,
    /// Whether audit replay evidence is present.
    pub audit_replay_validated: bool,
    /// Whether audit hash-chain continuity evidence is present.
    pub audit_hash_chain_validated: bool,
    /// Whether SQLite WAL mode evidence is present.
    pub sqlite_wal_mode_validated: bool,
    /// Whether SQLite integrity check evidence is present.
    pub sqlite_integrity_check_passed: bool,
    /// Whether SQLite checkpoint recovery evidence is present.
    pub sqlite_checkpoint_recovered: bool,
    /// Whether backup/restore evidence is present.
    pub backup_restore_validated: bool,
    /// Whether concurrent runtime access evidence is present.
    pub concurrent_access_validated: bool,
    /// Whether recovery/runbook reference is present.
    pub recovery_runbook_reference_present: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Validation status.
    pub status: RuntimeDeploymentAuditSqliteTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator performed service-manager actions. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Always false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Always false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Sanitized deployment-host backup/restore validation request.
///
/// This contains only reference presence and outcome flags. It must not embed
/// host paths, backup archives, audit payloads, SQLite contents, logs, command
/// output, secrets, or evidence artifact contents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentBackupRestoreTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether evidence came from a physical/deployment-like host.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// Whether backup artifact locator/reference evidence is present.
    pub backup_artifact_reference_present: bool,
    /// Whether restore execution evidence is present.
    pub restore_execution_reference_present: bool,
    /// Whether backup/restore load evidence is present.
    pub deployment_load_reference_present: bool,
    /// Whether audit replay after restore was validated.
    pub audit_replay_after_restore_validated: bool,
    /// Whether audit hash-chain continuity after restore was validated.
    pub audit_hash_chain_after_restore_validated: bool,
    /// Whether SQLite integrity after restore was validated.
    pub sqlite_integrity_after_restore_validated: bool,
    /// Whether SQLite WAL/checkpoint recovery after restore was validated.
    pub sqlite_checkpoint_after_restore_validated: bool,
    /// Whether restored runtime checkpoints were validated.
    pub runtime_checkpoint_restore_validated: bool,
    /// Whether post-restore runtime smoke evidence is present.
    pub post_restore_runtime_smoke_passed: bool,
    /// Whether rollback/fallback reference is present.
    pub rollback_reference_present: bool,
    /// Whether recovery/runbook reference is present.
    pub recovery_runbook_reference_present: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Whether this validator executed backup/restore actions. Must be false.
    pub backup_restore_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Must be false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Must be false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for deployment-host backup/restore evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentBackupRestoreTranscriptReport {
    /// Deployment backup/restore transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether deployment-like host evidence is present.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// Whether backup artifact locator/reference evidence is present.
    pub backup_artifact_reference_present: bool,
    /// Whether restore execution evidence is present.
    pub restore_execution_reference_present: bool,
    /// Whether backup/restore load evidence is present.
    pub deployment_load_reference_present: bool,
    /// Whether audit replay after restore was validated.
    pub audit_replay_after_restore_validated: bool,
    /// Whether audit hash-chain continuity after restore was validated.
    pub audit_hash_chain_after_restore_validated: bool,
    /// Whether SQLite integrity after restore was validated.
    pub sqlite_integrity_after_restore_validated: bool,
    /// Whether SQLite WAL/checkpoint recovery after restore was validated.
    pub sqlite_checkpoint_after_restore_validated: bool,
    /// Whether restored runtime checkpoints were validated.
    pub runtime_checkpoint_restore_validated: bool,
    /// Whether post-restore runtime smoke evidence is present.
    pub post_restore_runtime_smoke_passed: bool,
    /// Whether rollback/fallback reference is present.
    pub rollback_reference_present: bool,
    /// Whether recovery/runbook reference is present.
    pub recovery_runbook_reference_present: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Validation status.
    pub status: RuntimeDeploymentBackupRestoreTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator executed backup/restore actions. Always false.
    pub backup_restore_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Always false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Always false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Sanitized deployment-host graceful-shutdown validation request.
///
/// This contains only reference presence and outcome flags. It must not embed
/// host paths, service logs, command output, audit payloads, checkpoint values,
/// secrets, or evidence artifact contents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentGracefulShutdownTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether evidence came from a physical/deployment-like host.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// Whether graceful-shutdown request evidence is present.
    pub shutdown_request_reference_present: bool,
    /// Whether stop/quiesce observation evidence is present.
    pub service_stopped_reference_present: bool,
    /// Whether the local graceful-shutdown checkpoint evidence is present.
    pub graceful_shutdown_checkpoint_reference_present: bool,
    /// Whether audit replay after shutdown was validated.
    pub audit_replay_after_shutdown_validated: bool,
    /// Whether SQLite reopen/checkpoint recovery after shutdown was validated.
    pub sqlite_reopen_after_shutdown_validated: bool,
    /// Whether restart recovery after shutdown was validated.
    pub restart_recovery_after_shutdown_validated: bool,
    /// Whether post-shutdown runtime smoke evidence is present.
    pub post_shutdown_runtime_smoke_passed: bool,
    /// Whether operator review/approval reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether this validator performed service-manager actions. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Must be false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Must be false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for deployment-host graceful-shutdown evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentGracefulShutdownTranscriptReport {
    /// Deployment graceful-shutdown transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether deployment-like host evidence is present.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// Whether graceful-shutdown request evidence is present.
    pub shutdown_request_reference_present: bool,
    /// Whether stop/quiesce observation evidence is present.
    pub service_stopped_reference_present: bool,
    /// Whether the local graceful-shutdown checkpoint evidence is present.
    pub graceful_shutdown_checkpoint_reference_present: bool,
    /// Whether audit replay after shutdown was validated.
    pub audit_replay_after_shutdown_validated: bool,
    /// Whether SQLite reopen/checkpoint recovery after shutdown was validated.
    pub sqlite_reopen_after_shutdown_validated: bool,
    /// Whether restart recovery after shutdown was validated.
    pub restart_recovery_after_shutdown_validated: bool,
    /// Whether post-shutdown runtime smoke evidence is present.
    pub post_shutdown_runtime_smoke_passed: bool,
    /// Whether operator review/approval reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Validation status.
    pub status: RuntimeDeploymentGracefulShutdownTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator performed service-manager actions. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Always false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Always false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Sanitized deployment-host SQLite schema migration validation request.
///
/// This contains only reference presence and outcome flags. It must not embed
/// host paths, SQL dumps, migration output, database contents, logs, checkpoint
/// values, secrets, or evidence artifact contents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSqliteSchemaMigrationTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether evidence came from a physical/deployment-like host.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// SQLite schema version before migration.
    pub pre_migration_schema_version: i64,
    /// SQLite schema version after migration.
    pub post_migration_schema_version: i64,
    /// Expected schema version for the target binary.
    pub expected_schema_version: i64,
    /// Whether a pre-migration backup evidence reference is present.
    pub pre_migration_backup_reference_present: bool,
    /// Whether migration execution evidence is present.
    pub migration_execution_reference_present: bool,
    /// Whether schema version transition evidence is present and matched.
    pub schema_version_transition_validated: bool,
    /// Whether SQLite integrity check evidence is present after migration.
    pub sqlite_integrity_check_passed: bool,
    /// Whether checkpoint reopen evidence is present after migration.
    pub sqlite_checkpoint_reopened: bool,
    /// Whether audit replay evidence is present after migration.
    pub audit_replay_after_migration_validated: bool,
    /// Whether rollback plan/reference is present.
    pub rollback_reference_present: bool,
    /// Whether runtime remained quiesced or degraded during migration.
    pub runtime_quiesced_or_degraded: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Whether this validator executed the migration. Must be false.
    pub migration_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Must be false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Must be false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Must be false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for deployment-host SQLite schema migration evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSqliteSchemaMigrationTranscriptReport {
    /// Deployment SQLite schema migration transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Non-secret deployment-host or runner label.
    pub host_label: String,
    /// Whether deployment-like host evidence is present.
    pub deployment_host_evidence: bool,
    /// Whether service lifecycle context evidence is present.
    pub service_lifecycle_reference_present: bool,
    /// SQLite schema version before migration.
    pub pre_migration_schema_version: i64,
    /// SQLite schema version after migration.
    pub post_migration_schema_version: i64,
    /// Expected schema version for the target binary.
    pub expected_schema_version: i64,
    /// Whether a pre-migration backup evidence reference is present.
    pub pre_migration_backup_reference_present: bool,
    /// Whether migration execution evidence is present.
    pub migration_execution_reference_present: bool,
    /// Whether schema version transition evidence is present and matched.
    pub schema_version_transition_validated: bool,
    /// Whether SQLite integrity check evidence is present after migration.
    pub sqlite_integrity_check_passed: bool,
    /// Whether checkpoint reopen evidence is present after migration.
    pub sqlite_checkpoint_reopened: bool,
    /// Whether audit replay evidence is present after migration.
    pub audit_replay_after_migration_validated: bool,
    /// Whether rollback plan/reference is present.
    pub rollback_reference_present: bool,
    /// Whether runtime quiesce/degrade evidence is present.
    pub runtime_quiesced_or_degraded: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Validation status.
    pub status: RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator executed the migration. Always false.
    pub migration_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated deployment paths. Always false.
    pub deployment_path_mutated_by_validator: bool,
    /// Whether this validator loaded secrets. Always false.
    pub secrets_loaded: bool,
    /// Whether this validator submitted externally. Always false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Inputs for one local deployment-like runtime smoke validation pass.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSmokeValidationRequest {
    /// Runtime lifecycle request to execute locally.
    pub lifecycle_request: RuntimeLifecycleRequest,
    /// Graceful-shutdown checkpoint request to execute locally.
    pub shutdown_request: RuntimeGracefulShutdownRequest,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeConcurrentLifecycleValidationReport {
    workers_completed: u64,
    audit_records_replayed: u64,
    plan_checkpoint_recovered: bool,
    adapter_checkpoint_recovered: bool,
    adapter_recovery_plan_checkpoint_recovered: bool,
    sqlite_integrity_check_passed: bool,
    external_submission_performed: bool,
    live_execution_performed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSmokeTestingArtifacts {
    validation_run: ValidationRunRecord,
    property_check: LocalPropertyCheckReport,
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeSmokePaperArtifacts {
    applicable: bool,
    execution_report_checkpointed: bool,
    ledger_report: Option<PaperAdapterRunLedgerReport>,
    ledger_replay_validated: bool,
    external_submission_performed: bool,
    live_execution_performed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSmokeCommunicationsArtifacts {
    route: RoutedOperatorCommand,
    remote_review: RemoteCommandSecurityReviewReport,
    platform_ingress: PlatformCommandIngressReport,
    remote_envelope: RemoteCommandEnvelopeValidationReport,
    channel_adapter: ChannelAdapterValidationReport,
    channel_session: ChannelSessionValidationReport,
    platform_adapter: PlatformAdapterReviewReport,
    dispatch: NotificationDispatchRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSmokeObservabilityArtifacts {
    record: crate::ObservabilityRecord,
    operations_review: ObservabilityOperationsReviewReport,
    export_dry_run: ObservabilityExportDryRunReport,
    alert_route_dispatch: ObservabilityAlertRouteDispatchReport,
    endpoint_preflight: ObservabilityEndpointPreflightReport,
    loopback_bind: ObservabilityLoopbackBindValidationReport,
    metrics_scrape: ObservabilityMetricsScrapePreflightReport,
    metrics_endpoint: ObservabilityMetricsEndpointValidationReport,
    tracing: LocalTracingSubscriberValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSmokeDashboardArtifacts {
    render: DashboardRenderRecord,
    hosted_security: DashboardHostedSecurityReviewReport,
    hosted_preflight: DashboardHostedRequestPreflightReport,
    hosted_validation: DashboardHostedRequestValidationReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeSmokeRecoveredCheckpoints {
    observability_record: bool,
    observability_operations_review: bool,
    observability_export_dry_run: bool,
    observability_alert_route_dispatch: bool,
    observability_endpoint_preflight: bool,
    observability_loopback_bind: bool,
    observability_metrics_scrape: bool,
    observability_metrics_endpoint: bool,
    observability_tracing: bool,
    communications_route: bool,
    communications_remote_review: bool,
    communications_platform_ingress: bool,
    communications_remote_envelope: bool,
    communications_channel_adapter: bool,
    communications_channel_session: bool,
    communications_platform_adapter: bool,
    communications_notification: bool,
    dashboard_render: bool,
    dashboard_hosted_security: bool,
    dashboard_hosted_preflight: bool,
    dashboard_hosted_validation: bool,
    validation_run: bool,
    property_check: bool,
    paper_execution_report: bool,
    paper_ledger: bool,
    failure_capture: bool,
    adapter_recovery_plan: bool,
}

impl RuntimeDeploymentSmokeValidationRequest {
    /// Validate local deployment-smoke request invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke timestamp must be non-zero".to_owned(),
            });
        }
        self.lifecycle_request.validate()?;
        self.shutdown_request.validate()?;
        Ok(())
    }
}

impl RuntimeRestartRecoveryValidationReport {
    /// Validate local restart recovery report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION}"
                ),
            });
        }
        if self.audit_records_replayed == 0 || !self.audit_replay_check_passed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery requires replayed audit records".to_owned(),
            });
        }
        if !self.sqlite_reopen_check_passed
            || !self.plan_checkpoint_recovered
            || !self.adapter_checkpoint_recovered
            || !self.adapter_recovery_plan_checkpoint_recovered
            || !self.local_review_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery requires coherent local planner, adapter, and adapter recovery-plan checkpoints".to_owned(),
            });
        }
        match self.recovery_disposition {
            RuntimeRestartRecoveryDisposition::ReadyForLocalReview => {
                if !self.graceful_shutdown_checkpoint_recovered {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "ready recovery disposition requires graceful shutdown checkpoint"
                            .to_owned(),
                    });
                }
            }
            RuntimeRestartRecoveryDisposition::NeedsOperatorReview => {
                if self.graceful_shutdown_checkpoint_recovered {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "operator-review recovery disposition requires missing graceful shutdown checkpoint".to_owned(),
                    });
                }
            }
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery validation must not perform external submission or live execution".to_owned(),
            });
        }
        self.validate_connector_lifecycle_recovery_summary()?;
        self.validate_opportunity_trace_recovery_summary()?;
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery validation must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_opportunity_trace_recovery_summary(&self) -> Result<(), RuntimeLifecycleError> {
        let Some(opportunity_trace_recovery) = &self.opportunity_trace_recovery else {
            if self.opportunity_trace_recovery_validated
                || self.opportunity_trace_discovered_candidates != 0
                || self.opportunity_trace_recovered_checkpoints != 0
                || !self.opportunity_trace_recovered_summaries.is_empty()
                || self.opportunity_trace_missing_checkpoints != 0
            {
                return Err(RuntimeLifecycleError::ValidationFailed {
                    reason: "runtime restart recovery trace summary fields require opportunity trace recovery".to_owned(),
                });
            }
            return Ok(());
        };

        if !opportunity_trace_recovery.trace_recovery_validated {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason:
                    "runtime restart recovery requires valid opportunity trace recovery summary"
                        .to_owned(),
            });
        }
        if opportunity_trace_recovery.discovered_candidates == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery requires non-empty opportunity trace recovery"
                    .to_owned(),
            });
        }
        if opportunity_trace_recovery.recovered_trace_checkpoints
            > opportunity_trace_recovery.discovered_candidates
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason:
                    "opportunity trace recovery cannot recover more checkpoints than discovered"
                        .to_owned(),
            });
        }
        if opportunity_trace_recovery
            .recovered_trace_checkpoints
            .saturating_add(opportunity_trace_recovery.missing_trace_checkpoints)
            != opportunity_trace_recovery.discovered_candidates
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "opportunity trace recovery must account for all discovered checkpoints"
                    .to_owned(),
            });
        }
        if opportunity_trace_recovery.recovered_trace_summaries.len() as u64
            != opportunity_trace_recovery.recovered_trace_checkpoints
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason:
                    "opportunity trace recovery summaries must match recovered checkpoint count"
                        .to_owned(),
            });
        }
        for summary in &opportunity_trace_recovery.recovered_trace_summaries {
            if summary.trace_id.trim().is_empty()
                || summary.strategy_id.trim().is_empty()
                || summary.planner_request_id.trim().is_empty()
                || summary.audit_sequence == 0
                || summary.traced_at_unix_ms == 0
                || summary.route_kind.trim().is_empty()
                || summary.leg_count == 0
            {
                return Err(RuntimeLifecycleError::ValidationFailed {
                    reason: "opportunity trace recovery summaries require non-empty local identifiers and counts".to_owned(),
                });
            }
        }
        if self.opportunity_trace_recovered_summaries
            != opportunity_trace_recovery.recovered_trace_summaries
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery recovered trace summaries must match recovered opportunity trace summary".to_owned(),
            });
        }
        if !self.opportunity_trace_recovery_validated
            || self.opportunity_trace_discovered_candidates
                != opportunity_trace_recovery.discovered_candidates
            || self.opportunity_trace_recovered_checkpoints
                != opportunity_trace_recovery.recovered_trace_checkpoints
            || self.opportunity_trace_missing_checkpoints
                != opportunity_trace_recovery.missing_trace_checkpoints
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery trace summary fields must match recovered opportunity trace summary".to_owned(),
            });
        }
        Ok(())
    }

    fn validate_connector_lifecycle_recovery_summary(&self) -> Result<(), RuntimeLifecycleError> {
        match (
            &self.recovered_cex_lifecycle,
            &self.recovered_dex_lifecycle,
            self.connector_lifecycle_recovery_validated,
            self.cex_lifecycle_checkpoint_recovered,
            self.dex_lifecycle_checkpoint_recovered,
        ) {
            (None, None, false, false, false) => Ok(()),
            (Some(cex), Some(dex), true, true, true) => {
                if cex.request_id.trim().is_empty()
                    || cex.client_order_id.trim().is_empty()
                    || cex.strategy_id.trim().is_empty()
                    || cex.venue_name.trim().is_empty()
                    || cex.market_pair.trim().is_empty()
                    || cex.final_status.trim().is_empty()
                    || cex.transition_count == 0
                {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "runtime restart recovery CEX lifecycle summary requires non-empty identifiers and transitions".to_owned(),
                    });
                }
                if dex.request_id.trim().is_empty()
                    || dex.strategy_id.trim().is_empty()
                    || dex.venue_name.trim().is_empty()
                    || dex.chain.trim().is_empty()
                    || dex.market_pair.trim().is_empty()
                    || dex.quote_response_id.trim().is_empty()
                    || dex.simulation_response_id.trim().is_empty()
                    || dex.route_kind.trim().is_empty()
                    || dex.simulation_status.trim().is_empty()
                {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "runtime restart recovery DEX lifecycle summary requires non-empty identifiers".to_owned(),
                    });
                }
                Ok(())
            }
            _ => Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery connector lifecycle fields must be all present or all absent".to_owned(),
            }),
        }
    }
}

impl RuntimeDeploymentSmokeValidationReport {
    /// Validate local deployment-smoke report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION}"
                ),
            });
        }
        self.validate_required_smoke_checks()?;
        self.validate_observability_local_only()?;
        self.validate_communications_local_only()?;
        self.validate_dashboard_local_only()?;
        self.validate_testing_local_only()?;
        self.validate_paper_ledger_local_only()?;
        self.validate_concurrent_lifecycle_local_only()?;
        self.validate_failure_capture_local_only()?;
        if self.restart_audit_records_replayed == 0 || self.backup_audit_records_replayed == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires replayed audit records".to_owned(),
            });
        }
        if self.recovery_disposition != RuntimeRestartRecoveryDisposition::ReadyForLocalReview {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires clean local recovery disposition"
                    .to_owned(),
            });
        }
        if !self.restart_plan_checkpoint_recovered
            || !self.restart_adapter_checkpoint_recovered
            || !self.restart_adapter_recovery_plan_checkpoint_recovered
            || !self.restart_graceful_shutdown_checkpoint_recovered
            || !self.restart_opportunity_trace_recovery_validated
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires restart recovery of planner, adapter, adapter recovery-plan, graceful-shutdown, and opportunity trace checkpoints"
                    .to_owned(),
            });
        }
        if self.service_manager_action_performed
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke must not perform service-manager action, external submission, or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke must not approve production readiness".to_owned(),
            });
        }
        match &self.opportunity_trace_recovery {
            Some(trace_recovery) => {
                if !trace_recovery.trace_recovery_validated {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason:
                            "runtime deployment smoke requires opportunity trace recovery validation"
                                .to_owned(),
                    });
                }
                if self.restart_opportunity_trace_discovered_candidates
                    != trace_recovery.discovered_candidates
                    || self.restart_opportunity_trace_recovered_checkpoints
                        != trace_recovery.recovered_trace_checkpoints
                    || self.restart_opportunity_trace_recovered_summaries
                        != trace_recovery.recovered_trace_summaries
                    || self.restart_opportunity_trace_missing_checkpoints
                        != trace_recovery.missing_trace_checkpoints
                {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "runtime deployment smoke opportunity trace recovery fields must match restart recovery summary".to_owned(),
                    });
                }
                if trace_recovery.recovered_trace_summaries.len() as u64
                    != trace_recovery.recovered_trace_checkpoints
                {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "runtime deployment smoke recovered opportunity trace summaries must match restart recovery count".to_owned(),
                    });
                }
            }
            None => {
                return Err(RuntimeLifecycleError::ValidationFailed {
                    reason: "runtime deployment smoke requires opportunity trace recovery summary"
                        .to_owned(),
                });
            }
        }
        Ok(())
    }

    fn validate_observability_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if self.observability_metrics_endpoint_started
            || self.observability_public_network_exposed
            || self.observability_outbound_alerts_sent
            || self.observability_telemetry_exported
            || self.observability_production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed { reason: "runtime deployment smoke observability must remain local-only and non-production".to_owned() });
        }
        if !self.observability_local_metrics_request_served {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires one bounded local metrics request"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_concurrent_lifecycle_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if !self.concurrent_lifecycle_validated
            || self.concurrent_lifecycle_workers == 0
            || self.concurrent_lifecycle_audit_records_replayed == 0
            || !self.concurrent_lifecycle_sqlite_integrity_check_passed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires concurrent local lifecycle validation"
                    .to_owned(),
            });
        }
        if self.concurrent_lifecycle_external_submission_performed
            || self.concurrent_lifecycle_live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke concurrent lifecycle validation must remain local-only".to_owned(),
            });
        }
        Ok(())
    }

    fn validate_failure_capture_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if self.failure_capture_metrics_endpoint_started
            || self.failure_capture_public_network_exposed
            || self.failure_capture_outbound_alerts_sent
            || self.failure_capture_external_submission_performed
            || self.failure_capture_live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke failure capture must remain local-only"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_communications_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if !self.communications_command_routed
            || !self.communications_command_route_checkpoint_recovered
            || !self.communications_remote_command_reviewed
            || !self.communications_remote_command_review_checkpoint_recovered
            || !self.communications_platform_command_ingress_validated
            || !self.communications_platform_command_ingress_checkpoint_recovered
            || !self.communications_remote_command_envelope_validated
            || !self.communications_remote_command_envelope_checkpoint_recovered
            || !self.communications_channel_adapter_validated
            || !self.communications_channel_adapter_checkpoint_recovered
            || !self.communications_channel_session_validated
            || !self.communications_channel_session_checkpoint_recovered
            || !self.communications_platform_adapter_reviewed
            || !self.communications_platform_adapter_checkpoint_recovered
            || !self.communications_notification_dispatched
            || !self.communications_notification_checkpoint_recovered
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires communications route, remote command review, platform command ingress, remote command envelope, channel adapter, channel session, platform adapter, and notification checkpoint recovery".to_owned(),
            });
        }
        if self.communications_execution_enabled
            || self.communications_remote_commands_enabled
            || self.communications_outbound_network_used
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke communications checks must not enable execution, remote commands, or outbound network".to_owned(),
            });
        }
        Ok(())
    }

    fn validate_dashboard_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if !self.dashboard_rendered
            || !self.dashboard_checkpoint_recovered
            || !self.dashboard_hosted_security_reviewed
            || !self.dashboard_hosted_security_checkpoint_recovered
            || !self.dashboard_hosted_request_preflighted
            || !self.dashboard_hosted_request_preflight_checkpoint_recovered
            || !self.dashboard_hosted_request_validated
            || !self.dashboard_hosted_request_validation_checkpoint_recovered
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires dashboard render, hosted-security, hosted-preflight, and hosted-request checkpoint recovery".to_owned(),
            });
        }
        if self.dashboard_panel_count == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires at least one dashboard panel".to_owned(),
            });
        }
        if !self.dashboard_local_one_shot_request_served {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires one bounded local dashboard request"
                    .to_owned(),
            });
        }
        if self.dashboard_server_started
            || self.dashboard_public_network_exposed
            || self.dashboard_live_controls_enabled
            || self.dashboard_hosted_production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke dashboard checks must remain local-only and non-production"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_testing_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if !self.validation_run_recorded
            || !self.validation_run_checkpoint_recovered
            || !self.validation_property_checks_passed
            || !self.validation_property_checkpoint_recovered
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires local validation-run and property-check checkpoint recovery".to_owned(),
            });
        }
        if self.validation_external_fuzzer_invoked
            || self.validation_live_network_used
            || self.validation_live_execution_submitted
            || self.validation_signing_or_broadcast_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke validation-runner checks must remain local-only"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_paper_ledger_local_only(&self) -> Result<(), RuntimeLifecycleError> {
        if self.paper_ledger_applicable
            && (!self.paper_execution_report_checkpointed
                || !self.paper_execution_report_checkpoint_recovered
                || !self.paper_ledger_checkpointed
                || !self.paper_ledger_checkpoint_recovered
                || self.paper_modeled_fills_settled == 0
                || self.paper_ledger_audit_records_appended == 0
                || !self.paper_ledger_replay_validated)
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires paper execution report and ledger checkpoint recovery for paper plans".to_owned(),
            });
        }
        if self.paper_ledger_external_submission_performed
            || self.paper_ledger_live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke paper ledger checks must remain local-only"
                    .to_owned(),
            });
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn validate_required_smoke_checks(&self) -> Result<(), RuntimeLifecycleError> {
        for (passed, name) in [
            (self.lifecycle_completed, "lifecycle"),
            (self.graceful_shutdown_checkpointed, "graceful shutdown"),
            (self.backup_restore_validated, "backup restore"),
            (self.restart_recovery_validated, "restart recovery"),
            (self.audit_durability_validated, "audit durability"),
            (self.observability_collected, "observability collection"),
            (
                self.observability_checkpoint_recovered,
                "observability checkpoint recovery",
            ),
            (
                self.observability_operations_reviewed,
                "observability operations review",
            ),
            (
                self.observability_operations_checkpoint_recovered,
                "observability operations checkpoint recovery",
            ),
            (
                self.observability_export_dry_run_rendered,
                "observability export dry-run",
            ),
            (
                self.observability_export_checkpoint_recovered,
                "observability export checkpoint recovery",
            ),
            (
                self.observability_alert_route_dispatched,
                "observability alert-route dispatch",
            ),
            (
                self.observability_alert_route_checkpoint_recovered,
                "observability alert-route checkpoint recovery",
            ),
            (
                self.observability_endpoint_preflighted,
                "observability endpoint preflight",
            ),
            (
                self.observability_endpoint_checkpoint_recovered,
                "observability endpoint checkpoint recovery",
            ),
            (
                self.observability_loopback_bind_validated,
                "observability loopback bind validation",
            ),
            (
                self.observability_loopback_bind_checkpoint_recovered,
                "observability loopback bind checkpoint recovery",
            ),
            (
                self.observability_metrics_scrape_preflighted,
                "observability metrics scrape preflight",
            ),
            (
                self.observability_metrics_scrape_checkpoint_recovered,
                "observability metrics scrape checkpoint recovery",
            ),
            (
                self.observability_metrics_endpoint_validated,
                "observability metrics endpoint validation",
            ),
            (
                self.observability_metrics_endpoint_checkpoint_recovered,
                "observability metrics endpoint checkpoint recovery",
            ),
            (
                self.observability_tracing_captured,
                "observability tracing capture",
            ),
            (
                self.observability_tracing_checkpoint_recovered,
                "observability tracing checkpoint recovery",
            ),
            (
                self.communications_command_routed,
                "communications command route",
            ),
            (
                self.communications_command_route_checkpoint_recovered,
                "communications command-route checkpoint recovery",
            ),
            (
                self.communications_remote_command_reviewed,
                "communications remote command review",
            ),
            (
                self.communications_remote_command_review_checkpoint_recovered,
                "communications remote command review checkpoint recovery",
            ),
            (
                self.communications_platform_command_ingress_validated,
                "communications platform command ingress validation",
            ),
            (
                self.communications_platform_command_ingress_checkpoint_recovered,
                "communications platform command ingress checkpoint recovery",
            ),
            (
                self.communications_remote_command_envelope_validated,
                "communications remote command envelope validation",
            ),
            (
                self.communications_remote_command_envelope_checkpoint_recovered,
                "communications remote command envelope checkpoint recovery",
            ),
            (
                self.communications_channel_adapter_validated,
                "communications channel adapter validation",
            ),
            (
                self.communications_channel_adapter_checkpoint_recovered,
                "communications channel adapter checkpoint recovery",
            ),
            (
                self.communications_channel_session_validated,
                "communications channel session validation",
            ),
            (
                self.communications_channel_session_checkpoint_recovered,
                "communications channel session checkpoint recovery",
            ),
            (
                self.communications_platform_adapter_reviewed,
                "communications platform adapter review",
            ),
            (
                self.communications_platform_adapter_checkpoint_recovered,
                "communications platform adapter checkpoint recovery",
            ),
            (
                self.communications_notification_dispatched,
                "communications notification dispatch",
            ),
            (
                self.communications_notification_checkpoint_recovered,
                "communications notification checkpoint recovery",
            ),
            (self.dashboard_rendered, "dashboard render"),
            (
                self.dashboard_checkpoint_recovered,
                "dashboard checkpoint recovery",
            ),
            (
                self.dashboard_hosted_security_reviewed,
                "dashboard hosted-security review",
            ),
            (
                self.dashboard_hosted_security_checkpoint_recovered,
                "dashboard hosted-security checkpoint recovery",
            ),
            (
                self.dashboard_hosted_request_preflighted,
                "dashboard hosted-request preflight",
            ),
            (
                self.dashboard_hosted_request_preflight_checkpoint_recovered,
                "dashboard hosted-request preflight checkpoint recovery",
            ),
            (
                self.dashboard_hosted_request_validated,
                "dashboard hosted-request validation",
            ),
            (
                self.dashboard_hosted_request_validation_checkpoint_recovered,
                "dashboard hosted-request validation checkpoint recovery",
            ),
            (self.validation_run_recorded, "validation run"),
            (
                self.validation_run_checkpoint_recovered,
                "validation-run checkpoint recovery",
            ),
            (
                self.validation_property_checks_passed,
                "validation property checks",
            ),
            (
                self.validation_property_checkpoint_recovered,
                "validation property-check checkpoint recovery",
            ),
            (self.failure_capture_validated, "failure capture"),
            (
                self.failure_capture_checkpoint_recovered,
                "failure-capture checkpoint recovery",
            ),
        ] {
            if !passed {
                return Err(RuntimeLifecycleError::ValidationFailed {
                    reason: format!("runtime deployment smoke {name} check must pass"),
                });
            }
        }
        Ok(())
    }
}

impl RuntimeDeploymentSmokeLoadIteration {
    /// Validate one measured local runtime-smoke iteration.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.iteration_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load iteration id is required".to_owned(),
            });
        }
        self.report.validate()
    }
}

impl RuntimeDeploymentSmokeLoadValidationReport {
    /// Build and validate an aggregate local runtime smoke load/latency report.
    pub fn from_iterations(
        iterations: Vec<RuntimeDeploymentSmokeLoadIteration>,
    ) -> Result<Self, RuntimeLifecycleError> {
        if iterations.is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load validation requires at least one iteration".to_owned(),
            });
        }

        for iteration in &iterations {
            iteration.validate()?;
        }

        let iterations_attempted = iterations.len() as u64;
        let total_elapsed_ms: u64 = iterations
            .iter()
            .map(|iteration| iteration.elapsed_ms)
            .sum();
        let min_elapsed_ms = iterations
            .iter()
            .map(|iteration| iteration.elapsed_ms)
            .min()
            .unwrap_or(0);
        let max_elapsed_ms = iterations
            .iter()
            .map(|iteration| iteration.elapsed_ms)
            .max()
            .unwrap_or(0);
        let report = Self {
            validation_version: RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION.to_owned(),
            iterations_attempted,
            iterations_passed: iterations_attempted,
            min_elapsed_ms,
            max_elapsed_ms,
            average_elapsed_ms: total_elapsed_ms / iterations_attempted,
            total_elapsed_ms,
            restart_audit_records_replayed: iterations
                .iter()
                .map(|iteration| iteration.report.restart_audit_records_replayed)
                .sum(),
            backup_audit_records_replayed: iterations
                .iter()
                .map(|iteration| iteration.report.backup_audit_records_replayed)
                .sum(),
            opportunity_trace_recovered_checkpoints: iterations
                .iter()
                .map(|iteration| {
                    iteration
                        .report
                        .restart_opportunity_trace_recovered_checkpoints
                })
                .sum(),
            opportunity_trace_recovered_summaries: iterations
                .iter()
                .map(|iteration| {
                    u64::try_from(
                        iteration
                            .report
                            .restart_opportunity_trace_recovered_summaries
                            .len(),
                    )
                    .unwrap_or(u64::MAX)
                })
                .sum(),
            opportunity_trace_missing_checkpoints: iterations
                .iter()
                .map(|iteration| {
                    iteration
                        .report
                        .restart_opportunity_trace_missing_checkpoints
                })
                .sum(),
            service_manager_action_performed: iterations
                .iter()
                .any(|iteration| iteration.report.service_manager_action_performed),
            external_submission_performed: iterations
                .iter()
                .any(|iteration| iteration.report.external_submission_performed),
            live_execution_performed: iterations
                .iter()
                .any(|iteration| iteration.report.live_execution_performed),
            production_ready: false,
            unresolved_blockers: runtime_deployment_smoke_load_unresolved_blockers(),
        };
        report.validate()?;
        Ok(report)
    }

    /// Validate local runtime smoke load/latency aggregate invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION}"
                ),
            });
        }
        if self.iterations_attempted == 0 || self.iterations_passed != self.iterations_attempted {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load validation requires all iterations to pass".to_owned(),
            });
        }
        if self.max_elapsed_ms < self.min_elapsed_ms {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load max elapsed must be >= min elapsed".to_owned(),
            });
        }
        if self.restart_audit_records_replayed == 0
            || self.backup_audit_records_replayed == 0
            || self.opportunity_trace_recovered_checkpoints == 0
            || self.opportunity_trace_recovered_summaries
                != self.opportunity_trace_recovered_checkpoints
            || self.opportunity_trace_missing_checkpoints != 0
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load validation requires replay and complete trace recovery"
                    .to_owned(),
            });
        }
        if self.service_manager_action_performed
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load validation must remain local-only".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke load validation must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeLoadProfileReviewRequest {
    /// Validate local runtime load profile review input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.review_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review id is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review timestamp is required".to_owned(),
            });
        }
        self.load_report.validate()?;
        if self.max_average_elapsed_ms == 0
            || self.max_single_iteration_elapsed_ms == 0
            || self.max_total_elapsed_ms == 0
            || self.max_peak_memory_mb == 0
            || self.max_peak_cpu_percent == 0
            || self.observed_peak_cpu_percent > 100
            || self.max_peak_cpu_percent > 100
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review budgets must be non-zero and CPU percent must be <= 100".to_owned(),
            });
        }
        if self.service_manager_action_performed
            || self.external_calls_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review must remain local-only".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeLoadProfileReviewReport {
    /// Validate local runtime load profile review report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_LOAD_PROFILE_REVIEW_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!("validation_version must be {RUNTIME_LOAD_PROFILE_REVIEW_VERSION}"),
            });
        }
        if self.review_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review report id is required".to_owned(),
            });
        }
        if self.iterations_reviewed == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review requires at least one iteration".to_owned(),
            });
        }
        if self.service_manager_action_performed
            || self.external_calls_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime load profile review report must not contain side effects or production readiness".to_owned(),
            });
        }
        if self.status == RuntimeLoadProfileReviewStatus::ReadyForLocalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready runtime load profile review must not contain local blockers"
                    .to_owned(),
            });
        }
        if self.status == RuntimeLoadProfileReviewStatus::Blocked && self.blocker_codes.is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked runtime load profile review requires blocker codes".to_owned(),
            });
        }
        Ok(())
    }
}

/// Review local runtime smoke load/latency evidence against supplied local budgets.
///
/// This does not execute benchmarks, inspect host resources, call providers,
/// start services, perform live execution, or claim production readiness.
pub fn review_runtime_load_profile(
    request: RuntimeLoadProfileReviewRequest,
) -> Result<RuntimeLoadProfileReviewReport, RuntimeLifecycleError> {
    request.validate()?;
    let latency_budget_met = request.load_report.average_elapsed_ms
        <= request.max_average_elapsed_ms
        && request.load_report.max_elapsed_ms <= request.max_single_iteration_elapsed_ms
        && request.load_report.total_elapsed_ms <= request.max_total_elapsed_ms;
    let resource_budget_met = request.observed_peak_memory_mb <= request.max_peak_memory_mb
        && request.observed_peak_cpu_percent <= request.max_peak_cpu_percent;
    let replay_recovery_evidence_validated = request.load_report.restart_audit_records_replayed > 0
        && request.load_report.backup_audit_records_replayed > 0
        && request.load_report.opportunity_trace_recovered_checkpoints > 0
        && request.load_report.opportunity_trace_missing_checkpoints == 0;
    let blocker_codes = runtime_load_profile_blockers(
        latency_budget_met,
        resource_budget_met,
        replay_recovery_evidence_validated,
    );
    let status = if blocker_codes.is_empty() {
        RuntimeLoadProfileReviewStatus::ReadyForLocalReview
    } else {
        RuntimeLoadProfileReviewStatus::Blocked
    };
    let report = RuntimeLoadProfileReviewReport {
        validation_version: RUNTIME_LOAD_PROFILE_REVIEW_VERSION.to_owned(),
        review_id: request.review_id,
        status,
        iterations_reviewed: request.load_report.iterations_attempted,
        latency_budget_met,
        resource_budget_met,
        replay_recovery_evidence_validated,
        observed_average_elapsed_ms: request.load_report.average_elapsed_ms,
        observed_max_elapsed_ms: request.load_report.max_elapsed_ms,
        observed_total_elapsed_ms: request.load_report.total_elapsed_ms,
        observed_peak_memory_mb: request.observed_peak_memory_mb,
        observed_peak_cpu_percent: request.observed_peak_cpu_percent,
        deployment_host_load_evidence_available: request.deployment_host_load_evidence_available,
        live_feed_backpressure_evidence_available: request
            .live_feed_backpressure_evidence_available,
        target_runtime_evidence_available: request.target_runtime_evidence_available,
        blocker_codes,
        remaining_external_evidence: runtime_load_profile_remaining_external_evidence(),
        service_manager_action_performed: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: request.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

impl RuntimeProductionPreflightRequest {
    /// Validate local production-runtime preflight input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.preflight_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight id is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight timestamp must be non-zero".to_owned(),
            });
        }
        self.smoke_report.validate()?;
        self.load_report.validate()?;
        if self.service_manager_action_performed
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight must not perform service-manager action, external submission, or live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeProductionPreflightReport {
    /// Validate local production-runtime preflight report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_PRODUCTION_PREFLIGHT_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_PRODUCTION_PREFLIGHT_VALIDATION_VERSION}"
                ),
            });
        }
        if self.preflight_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight id is required".to_owned(),
            });
        }
        if !self.local_smoke_validated || !self.local_smoke_load_validated {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight requires local smoke and load validation"
                    .to_owned(),
            });
        }
        if self.service_manager_action_performed
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight must remain non-mutating and local-only"
                    .to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight must not approve production readiness"
                    .to_owned(),
            });
        }
        if self.unresolved_blockers.is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime production preflight must retain unresolved production blockers"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeServiceManagerLifecycleTranscript {
    /// Validate sanitized service-manager lifecycle transcript input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.transcript_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript id is required".to_owned(),
            });
        }
        validate_service_manager_unit_name(&self.unit_name)?;
        if self.events.is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript requires events".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript timestamp must be non-zero"
                    .to_owned(),
            });
        }
        if self.concurrent_lifecycle_reference_present
            && self.concurrent_lifecycle_worker_count == 0
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle concurrent worker count must be non-zero when concurrent evidence is referenced".to_owned(),
            });
        }
        if self.concurrent_lifecycle_success
            && (!self.concurrent_lifecycle_reference_present
                || self.concurrent_lifecycle_worker_count == 0)
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle concurrent success requires referenced concurrent evidence".to_owned(),
            });
        }
        for event in &self.events {
            event.validate()?;
        }
        if self.service_manager_action_performed_by_validator
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript validation must not perform service actions, external submission, or live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeServiceManagerLifecycleEvent {
    /// Validate one sanitized lifecycle event.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.event_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle event id is required".to_owned(),
            });
        }
        if self.observed_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle event timestamp must be non-zero".to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeServiceManagerLifecycleTranscriptReport {
    /// Validate service-manager lifecycle transcript report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_SERVICE_MANAGER_LIFECYCLE_TRANSCRIPT_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_SERVICE_MANAGER_LIFECYCLE_TRANSCRIPT_VERSION}"
                ),
            });
        }
        validate_service_manager_unit_name(&self.unit_name)?;
        if self.event_count == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript report requires events".to_owned(),
            });
        }
        if self.concurrent_lifecycle_reference_present
            && self.concurrent_lifecycle_worker_count == 0
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript report concurrent worker count must be non-zero when evidence is referenced".to_owned(),
            });
        }
        if self.service_manager_action_performed_by_validator
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle transcript report must remain local-only"
                    .to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason:
                    "service-manager lifecycle transcript report must not approve production readiness"
                        .to_owned(),
            });
        }
        if self.status == RuntimeServiceManagerLifecycleTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready service-manager lifecycle transcript must not contain blockers"
                    .to_owned(),
            });
        }
        if self.status == RuntimeServiceManagerLifecycleTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked service-manager lifecycle transcript requires blocker codes"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeServiceManagerLifecycleRehearsalRequest {
    /// Validate local-only service-manager lifecycle rehearsal input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.rehearsal_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal id is required".to_owned(),
            });
        }
        validate_service_manager_unit_name(&self.unit_name)?;
        if self.events.is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal requires events".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal timestamp must be non-zero".to_owned(),
            });
        }
        if self.concurrent_lifecycle_reference_present
            && self.concurrent_lifecycle_worker_count == 0
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal concurrent worker count must be non-zero when concurrent evidence is referenced".to_owned(),
            });
        }
        if self.concurrent_lifecycle_success
            && (!self.concurrent_lifecycle_reference_present
                || self.concurrent_lifecycle_worker_count == 0)
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal concurrent success requires referenced concurrent evidence".to_owned(),
            });
        }
        for event in &self.events {
            event.validate()?;
        }
        if self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal must not perform service actions, mutate deployment paths, load secrets, submit externally, or perform live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeServiceManagerLifecycleRehearsalReport {
    /// Validate local-only service-manager lifecycle rehearsal report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_SERVICE_MANAGER_LIFECYCLE_REHEARSAL_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_SERVICE_MANAGER_LIFECYCLE_REHEARSAL_VERSION}"
                ),
            });
        }
        validate_service_manager_unit_name(&self.unit_name)?;
        if self.rehearsal_id.trim().is_empty() || self.event_count == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal report requires id and events"
                    .to_owned(),
            });
        }
        if self.concurrent_lifecycle_reference_present
            && self.concurrent_lifecycle_worker_count == 0
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal report concurrent worker count must be non-zero when evidence is referenced".to_owned(),
            });
        }
        if self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "service-manager lifecycle rehearsal report must not contain side effects or production readiness".to_owned(),
            });
        }
        if self.status == RuntimeServiceManagerLifecycleRehearsalStatus::Validated
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "validated service-manager lifecycle rehearsal must not contain blockers"
                    .to_owned(),
            });
        }
        if self.status == RuntimeServiceManagerLifecycleRehearsalStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked service-manager lifecycle rehearsal requires blocker codes"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentPermissionTranscript {
    /// Validate sanitized deployment permission transcript input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.transcript_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission transcript id is required".to_owned(),
            });
        }
        if self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission host label is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission transcript timestamp must be non-zero".to_owned(),
            });
        }
        if self.permission_changed_by_validator
            || self.production_path_mutated_by_validator
            || self.service_manager_action_performed_by_validator
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission transcript validator must not change permissions, mutate production paths, perform service actions, submit externally, or perform live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission transcript must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentPermissionTranscriptReport {
    /// Validate deployment permission transcript report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_PERMISSION_TRANSCRIPT_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_PERMISSION_TRANSCRIPT_VERSION}"
                ),
            });
        }
        if self.transcript_id.trim().is_empty() || self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission report requires id and host label".to_owned(),
            });
        }
        if self.permission_changed_by_validator
            || self.production_path_mutated_by_validator
            || self.service_manager_action_performed_by_validator
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment permission report must not contain validator side effects or production readiness".to_owned(),
            });
        }
        if self.status == RuntimeDeploymentPermissionTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready deployment permission report must not contain blockers".to_owned(),
            });
        }
        if self.status == RuntimeDeploymentPermissionTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked deployment permission report requires blocker codes".to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentAuditSqliteTranscript {
    /// Validate sanitized deployment audit/SQLite transcript input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.transcript_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite transcript id is required".to_owned(),
            });
        }
        if self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite host label is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite transcript timestamp must be non-zero".to_owned(),
            });
        }
        if self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite transcript validator must not perform service actions, mutate deployment paths, load secrets, submit externally, or perform live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite transcript must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentAuditSqliteTranscriptReport {
    /// Validate deployment audit/SQLite transcript report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_AUDIT_SQLITE_TRANSCRIPT_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_AUDIT_SQLITE_TRANSCRIPT_VERSION}"
                ),
            });
        }
        if self.transcript_id.trim().is_empty() || self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite report requires id and host label".to_owned(),
            });
        }
        if self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment audit/sqlite report must not contain validator side effects or production readiness".to_owned(),
            });
        }
        if self.status == RuntimeDeploymentAuditSqliteTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready deployment audit/sqlite report must not contain blockers".to_owned(),
            });
        }
        if self.status == RuntimeDeploymentAuditSqliteTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked deployment audit/sqlite report requires blocker codes".to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentBackupRestoreTranscript {
    /// Validate sanitized deployment backup/restore transcript input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.transcript_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore transcript id is required".to_owned(),
            });
        }
        if self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore host label is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore transcript timestamp must be non-zero"
                    .to_owned(),
            });
        }
        if self.backup_restore_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore transcript validator must not execute backup/restore, perform service actions, mutate deployment paths, load secrets, submit externally, or perform live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore transcript must not claim production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentBackupRestoreTranscriptReport {
    /// Validate deployment backup/restore transcript report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_BACKUP_RESTORE_TRANSCRIPT_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_BACKUP_RESTORE_TRANSCRIPT_VERSION}"
                ),
            });
        }
        if self.transcript_id.trim().is_empty() || self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore report requires id and host label".to_owned(),
            });
        }
        if self.backup_restore_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment backup/restore report must not contain validator side effects or production readiness".to_owned(),
            });
        }
        if self.status == RuntimeDeploymentBackupRestoreTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready deployment backup/restore report must not contain blockers"
                    .to_owned(),
            });
        }
        if self.status == RuntimeDeploymentBackupRestoreTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked deployment backup/restore report requires blocker codes"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentGracefulShutdownTranscript {
    /// Validate sanitized deployment graceful-shutdown transcript input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.transcript_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment graceful-shutdown transcript id is required".to_owned(),
            });
        }
        if self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment graceful-shutdown host label is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment graceful-shutdown transcript timestamp must be non-zero"
                    .to_owned(),
            });
        }
        if self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment graceful-shutdown transcript validator must not perform service actions, mutate deployment paths, load secrets, submit externally, or perform live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason:
                    "deployment graceful-shutdown transcript must not claim production readiness"
                        .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentGracefulShutdownTranscriptReport {
    /// Validate deployment graceful-shutdown transcript report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_GRACEFUL_SHUTDOWN_TRANSCRIPT_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_GRACEFUL_SHUTDOWN_TRANSCRIPT_VERSION}"
                ),
            });
        }
        if self.transcript_id.trim().is_empty() || self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment graceful-shutdown report requires id and host label".to_owned(),
            });
        }
        if self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment graceful-shutdown report must not contain validator side effects or production readiness".to_owned(),
            });
        }
        if self.status == RuntimeDeploymentGracefulShutdownTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready deployment graceful-shutdown report must not contain blockers"
                    .to_owned(),
            });
        }
        if self.status == RuntimeDeploymentGracefulShutdownTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked deployment graceful-shutdown report requires blocker codes"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentSqliteSchemaMigrationTranscript {
    /// Validate sanitized deployment SQLite schema migration transcript input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.transcript_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration transcript id is required".to_owned(),
            });
        }
        if self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration host label is required".to_owned(),
            });
        }
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration transcript timestamp must be non-zero"
                    .to_owned(),
            });
        }
        if self.pre_migration_schema_version < 0
            || self.post_migration_schema_version < 0
            || self.expected_schema_version <= 0
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration versions must be non-negative with positive expected version".to_owned(),
            });
        }
        if self.migration_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration transcript validator must not execute migration, perform service actions, mutate deployment paths, load secrets, submit externally, or perform live execution".to_owned(),
            });
        }
        if self.production_ready_claimed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration transcript must not claim production readiness".to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentSqliteSchemaMigrationTranscriptReport {
    /// Validate deployment SQLite schema migration transcript report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_SQLITE_SCHEMA_MIGRATION_TRANSCRIPT_VERSION
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_SQLITE_SCHEMA_MIGRATION_TRANSCRIPT_VERSION}"
                ),
            });
        }
        if self.transcript_id.trim().is_empty() || self.host_label.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration report requires id and host label"
                    .to_owned(),
            });
        }
        if self.migration_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.deployment_path_mutated_by_validator
            || self.secrets_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "deployment sqlite schema migration report must not contain validator side effects or production readiness".to_owned(),
            });
        }
        if self.status
            == RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "ready deployment sqlite schema migration report must not contain blockers"
                    .to_owned(),
            });
        }
        if self.status == RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "blocked deployment sqlite schema migration report requires blocker codes"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeBackupRestoreValidationReport {
    /// Validate local backup/restore report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION}"
                ),
            });
        }
        if self.audit_records_replayed == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore validation requires audit records".to_owned(),
            });
        }
        if !self.audit_restore_check_passed
            || !self.sqlite_restore_check_passed
            || !self.plan_checkpoint_restored
            || !self.adapter_checkpoint_restored
            || !self.adapter_recovery_plan_checkpoint_restored
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore checks must recover plan, adapter, and adapter recovery-plan checkpoints".to_owned(),
            });
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore validation must not perform external submission or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore validation must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeGracefulShutdownRecord {
    /// Validate local graceful-shutdown invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown id is required".to_owned(),
            });
        }
        if self.runtime_graceful_shutdown_version != RUNTIME_GRACEFUL_SHUTDOWN_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "runtime_graceful_shutdown_version must be {RUNTIME_GRACEFUL_SHUTDOWN_VERSION}"
                ),
            });
        }
        if self.shutdown_checkpoint_key != RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected graceful shutdown checkpoint key".to_owned(),
            });
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown checkpoint must not perform external submission or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown checkpoint must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeLifecycleRecord {
    /// Validate local runtime lifecycle invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle id is required".to_owned(),
            });
        }
        if self.runtime_lifecycle_version != RUNTIME_LIFECYCLE_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!("runtime_lifecycle_version must be {RUNTIME_LIFECYCLE_VERSION}"),
            });
        }
        if self.scope == ExecutionScope::Live {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle records must not use live scope".to_owned(),
            });
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle must not perform external submission or live execution"
                    .to_owned(),
            });
        }
        if self.plan_checkpoint_key != EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected plan checkpoint key".to_owned(),
            });
        }
        if self.adapter_run_checkpoint_key != EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected adapter run checkpoint key".to_owned(),
            });
        }
        if self.adapter_recovery_plan_checkpoint_key
            != EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected adapter recovery-plan checkpoint key".to_owned(),
            });
        }
        self.adapter_run
            .validate()
            .map_err(RuntimeLifecycleError::Adapter)?;
        Ok(())
    }
}

/// Execute one local fail-closed runtime lifecycle.
///
/// The lifecycle appends audit events and writes the plan checkpoint before the
/// adapter boundary is evaluated. Any audit/state failure returns an error and
/// prevents later lifecycle steps. This function does not submit orders, call
/// exchanges/RPCs, sign payloads, broadcast transactions, withdraw funds, or
/// bridge assets.
pub fn run_local_runtime_lifecycle(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    policy: &PolicyEngine,
    request: RuntimeLifecycleRequest,
) -> Result<RuntimeLifecycleRecord, RuntimeLifecycleError> {
    request.validate()?;

    let start_record = journal.append_event(lifecycle_event(
        &request,
        AuditEventKind::RuntimeLifecycle,
        "runtime lifecycle started",
    ))?;

    let plan_checkpoint = persist_execution_plan_draft_checkpoint(store, &request.plan)?;
    let _plan_audit =
        append_execution_plan_draft_audit(journal, &request.plan, request.now_unix_ms)?;
    let plan_checkpoint_record = journal.append_event(checkpoint_event(
        &request,
        AuditEventKind::ExecutionPlanning,
        "execution plan checkpoint persisted before adapter evaluation",
        &plan_checkpoint,
    ))?;

    let adapter = DeterministicExecutionAdapterBoundary::new();
    let adapter_request = ExecutionAdapterRequest {
        id: request.adapter_request_id.clone(),
        plan: request.plan.clone(),
        config: request.adapter_config.clone(),
        now_unix_ms: request.now_unix_ms,
    };
    let adapter_run = adapter.evaluate_plan(&adapter_request, policy)?;
    let adapter_checkpoint = persist_execution_adapter_run_checkpoint(store, &adapter_run)?;
    let adapter_record = append_execution_adapter_run_audit(journal, &adapter_run)?;
    let recovery_plan =
        plan_execution_adapter_recovery(&request.plan, &adapter_run, request.now_unix_ms)?;
    let recovery_checkpoint =
        persist_execution_adapter_recovery_plan_checkpoint(store, &recovery_plan)?;
    let recovery_record = append_execution_adapter_recovery_plan_audit(journal, &recovery_plan)?;

    let record = RuntimeLifecycleRecord {
        id: request.id,
        runtime_lifecycle_version: RUNTIME_LIFECYCLE_VERSION.to_owned(),
        plan_id: request.plan.id,
        adapter_request_id: adapter_request.id,
        scope: adapter_run.scope,
        status: RuntimeLifecycleStatus::AdapterRunCheckpointed,
        plan_checkpoint_key: plan_checkpoint.key,
        adapter_run_checkpoint_key: adapter_checkpoint.key,
        adapter_recovery_plan_checkpoint_key: recovery_checkpoint.key,
        start_audit_sequence: start_record.sequence,
        plan_checkpoint_audit_sequence: plan_checkpoint_record.sequence,
        adapter_complete_audit_sequence: adapter_record.sequence,
        adapter_recovery_plan_audit_sequence: recovery_record.sequence,
        adapter_run,
        external_submission_performed: false,
        live_execution_performed: false,
        created_at_unix_ms: request.now_unix_ms,
        warnings: vec![
            "local runtime lifecycle only; no external submission, signing, broadcast, withdrawal, bridge, or live execution occurred".to_owned(),
        ],
    };
    record.validate()?;
    Ok(record)
}

/// Persist one local graceful-shutdown audit/state checkpoint.
///
/// This boundary models the local audit/state writes expected before a clean
/// runtime stop. It does not stop a process, interact with a service manager,
/// submit orders, call networks, sign payloads, broadcast transactions,
/// withdraw funds, or bridge assets.
pub fn run_local_graceful_shutdown_checkpoint(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    request: RuntimeGracefulShutdownRequest,
) -> Result<RuntimeGracefulShutdownRecord, RuntimeLifecycleError> {
    request.validate()?;

    let start_record = journal.append_event(graceful_shutdown_event(
        &request,
        "runtime graceful shutdown started",
    ))?;
    let checkpoint = StateCheckpoint {
        key: RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY.to_owned(),
        subsystem: "runtime-lifecycle".to_owned(),
        value: format!("graceful-shutdown:{}:{}", request.id, request.now_unix_ms),
        updated_at_unix_ms: request.now_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    let checkpoint_record = journal.append_event(
        graceful_shutdown_event(&request, "runtime graceful shutdown checkpoint persisted")
            .with_metadata("checkpoint_key", AuditValue::Text(checkpoint.key.clone()))
            .with_metadata(
                "checkpoint_subsystem",
                AuditValue::Text(checkpoint.subsystem.clone()),
            )
            .with_metadata(
                "checkpoint_updated_at_unix_ms",
                AuditValue::Unsigned(checkpoint.updated_at_unix_ms),
            ),
    )?;

    let record = RuntimeGracefulShutdownRecord {
        id: request.id,
        runtime_graceful_shutdown_version: RUNTIME_GRACEFUL_SHUTDOWN_VERSION.to_owned(),
        shutdown_checkpoint_key: checkpoint.key,
        shutdown_checkpoint_value: checkpoint.value,
        shutdown_start_audit_sequence: start_record.sequence,
        shutdown_checkpoint_audit_sequence: checkpoint_record.sequence,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        created_at_unix_ms: request.now_unix_ms,
        warnings: vec![
            "local graceful-shutdown checkpoint only; no service manager action, external submission, signing, broadcast, withdrawal, bridge, or live execution occurred".to_owned(),
        ],
    };
    record.validate()?;
    Ok(record)
}

/// Validate local backup/restore of runtime audit and SQLite state artifacts.
///
/// This boundary copies an existing non-secret local audit journal and
/// checkpointed SQLite database to caller-supplied backup paths, then reopens
/// the copies and verifies the runtime planner and adapter checkpoints can be
/// read. It does not start services, inspect deployment state, submit orders,
/// call networks, sign payloads, broadcast transactions, withdraw funds, or
/// bridge assets.
pub fn validate_local_runtime_backup_restore(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    backup_audit_path: impl AsRef<Path>,
    backup_state_path: impl AsRef<Path>,
) -> Result<RuntimeBackupRestoreValidationReport, RuntimeLifecycleError> {
    let audit_path = audit_path.as_ref();
    let state_path = state_path.as_ref();
    let backup_audit_path = backup_audit_path.as_ref();
    let backup_state_path = backup_state_path.as_ref();

    validate_runtime_backup_target(audit_path, backup_audit_path, "audit")?;
    validate_runtime_backup_target(state_path, backup_state_path, "state")?;

    let primary_journal = AppendOnlyAuditJournal::open(audit_path)?;
    let primary_next_sequence = primary_journal.next_sequence();
    let primary_audit_records = primary_next_sequence.saturating_sub(1);
    if primary_audit_records == 0 {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime backup/restore validation requires a non-empty audit journal"
                .to_owned(),
        });
    }

    let primary_state = SqliteWalStateStore::open(state_path)?;
    primary_state.integrity_check()?;
    primary_state.wal_checkpoint_truncate()?;
    let primary_plan_checkpoint = primary_state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let primary_adapter_checkpoint = primary_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();
    if !primary_plan_checkpoint || !primary_adapter_checkpoint {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime backup/restore validation requires planner and adapter checkpoints"
                .to_owned(),
        });
    }
    drop(primary_state);

    copy_runtime_backup_file(audit_path, backup_audit_path, "audit")?;
    copy_runtime_backup_file(state_path, backup_state_path, "state")?;

    let restored_journal = AppendOnlyAuditJournal::open(backup_audit_path)?;
    if restored_journal.next_sequence() != primary_next_sequence {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "restored audit journal sequence did not match primary".to_owned(),
        });
    }

    let restored_state = SqliteWalStateStore::open(backup_state_path)?;
    restored_state.integrity_check()?;
    let plan_checkpoint_restored = restored_state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let adapter_checkpoint_restored = restored_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();
    let adapter_recovery_plan_checkpoint_restored = restored_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)?
        .is_some();

    let report = RuntimeBackupRestoreValidationReport {
        validation_version: RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION.to_owned(),
        audit_records_replayed: restored_journal.next_sequence().saturating_sub(1),
        audit_restore_check_passed: true,
        sqlite_restore_check_passed: true,
        plan_checkpoint_restored,
        adapter_checkpoint_restored,
        adapter_recovery_plan_checkpoint_restored,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Validate local restart recovery from existing audit and SQLite state files.
///
/// This boundary reopens the local audit journal and SQLite state store, checks
/// audit replay, SQLite integrity, and required runtime lifecycle checkpoints,
/// then returns a non-secret recovery summary for operator review. It does not
/// start services, resume work, inspect deployment state, submit orders, call
/// networks, sign payloads, broadcast transactions, withdraw funds, or bridge
/// assets.
pub fn validate_local_runtime_restart_recovery(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
) -> Result<RuntimeRestartRecoveryValidationReport, RuntimeLifecycleError> {
    validate_local_runtime_restart_recovery_internal(audit_path, state_path, None)
}

/// Validate local restart recovery and include opportunity trace recovery summary
/// from the phase-27 local fixture corpus.
///
/// This function extends `validate_local_runtime_restart_recovery` by running the
/// opportunity candidate trace restart-recovery validation against dedicated local
/// fixture-backed evidence paths. It does not submit live work, call external
/// services, or perform production operations.
pub fn validate_local_runtime_restart_recovery_with_trace_recovery(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    policy: &PolicyEngine,
) -> Result<RuntimeRestartRecoveryValidationReport, RuntimeLifecycleError> {
    validate_local_runtime_restart_recovery_internal(audit_path, state_path, Some(policy))
}

#[allow(clippy::too_many_lines)]
fn validate_local_runtime_restart_recovery_internal(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    include_opportunity_trace_recovery: Option<&PolicyEngine>,
) -> Result<RuntimeRestartRecoveryValidationReport, RuntimeLifecycleError> {
    let audit_path = audit_path.as_ref();
    let state_path = state_path.as_ref();
    if audit_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery audit path is required".to_owned(),
        });
    }
    if state_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery state path is required".to_owned(),
        });
    }

    let journal = AppendOnlyAuditJournal::open(audit_path)?;
    let audit_records_replayed = journal.next_sequence().saturating_sub(1);
    if audit_records_replayed == 0 {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery requires a non-empty audit journal".to_owned(),
        });
    }

    let state = SqliteWalStateStore::open(state_path)?;
    state.integrity_check()?;
    let plan_checkpoint_recovered = state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let adapter_checkpoint_recovered = state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();
    let adapter_recovery_plan_checkpoint_recovered = state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)?
        .is_some();
    let graceful_shutdown_checkpoint_recovered = state
        .get_checkpoint(RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY)?
        .is_some();
    let connector_lifecycle_recovery = runtime_connector_lifecycle_recovery_summary(&state)?;
    let recovery_disposition = if graceful_shutdown_checkpoint_recovered {
        RuntimeRestartRecoveryDisposition::ReadyForLocalReview
    } else {
        RuntimeRestartRecoveryDisposition::NeedsOperatorReview
    };

    let report = RuntimeRestartRecoveryValidationReport {
        validation_version: RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION.to_owned(),
        audit_records_replayed,
        audit_replay_check_passed: true,
        sqlite_reopen_check_passed: true,
        plan_checkpoint_recovered,
        adapter_checkpoint_recovered,
        adapter_recovery_plan_checkpoint_recovered,
        graceful_shutdown_checkpoint_recovered,
        recovery_disposition,
        local_review_ready: plan_checkpoint_recovered
            && adapter_checkpoint_recovered
            && adapter_recovery_plan_checkpoint_recovered,
        connector_lifecycle_recovery_validated: false,
        cex_lifecycle_checkpoint_recovered: false,
        dex_lifecycle_checkpoint_recovered: false,
        recovered_cex_lifecycle: None,
        recovered_dex_lifecycle: None,
        opportunity_trace_recovery_validated: false,
        opportunity_trace_discovered_candidates: 0,
        opportunity_trace_recovered_checkpoints: 0,
        opportunity_trace_recovered_summaries: Vec::new(),
        opportunity_trace_missing_checkpoints: 0,
        opportunity_trace_recovery: None,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    let mut report = report;

    if let Some((cex_lifecycle, dex_lifecycle)) = connector_lifecycle_recovery {
        report.connector_lifecycle_recovery_validated = true;
        report.cex_lifecycle_checkpoint_recovered = true;
        report.dex_lifecycle_checkpoint_recovered = true;
        report.recovered_cex_lifecycle = Some(cex_lifecycle);
        report.recovered_dex_lifecycle = Some(dex_lifecycle);
    }

    if let Some(policy) = include_opportunity_trace_recovery {
        let opportunity_trace_audit_path =
            runtime_temp_path("arbyclaw-runtime-opportunity-trace-recovery-audit")?;
        let opportunity_trace_state_path =
            runtime_temp_path("arbyclaw-runtime-opportunity-trace-state")?;
        let opportunity_trace_recovery = {
            let recovered = validate_runtime_opportunity_trace_recovery(
                &opportunity_trace_audit_path,
                &opportunity_trace_state_path,
                policy,
            );
            cleanup_runtime_trace_recovery_paths(
                &opportunity_trace_audit_path,
                &opportunity_trace_state_path,
            );
            recovered?
        };
        report.opportunity_trace_recovery_validated =
            opportunity_trace_recovery.trace_recovery_validated;
        report.opportunity_trace_discovered_candidates =
            opportunity_trace_recovery.discovered_candidates;
        report.opportunity_trace_recovered_checkpoints =
            opportunity_trace_recovery.recovered_trace_checkpoints;
        report.opportunity_trace_recovered_summaries =
            opportunity_trace_recovery.recovered_trace_summaries.clone();
        report.opportunity_trace_missing_checkpoints =
            opportunity_trace_recovery.missing_trace_checkpoints;
        report.opportunity_trace_recovery = Some(opportunity_trace_recovery);
    }

    report.validate()?;
    Ok(report)
}

fn runtime_connector_lifecycle_recovery_summary(
    state: &SqliteWalStateStore,
) -> Result<
    Option<(
        RuntimeRecoveredCexLifecycleSummary,
        RuntimeRecoveredDexLifecycleSummary,
    )>,
    RuntimeLifecycleError,
> {
    let cex_checkpoint = state.get_checkpoint(CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY)?;
    let dex_checkpoint = state.get_checkpoint(DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY)?;
    match (cex_checkpoint, dex_checkpoint) {
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery connector lifecycle checkpoints must be recovered together".to_owned(),
        }),
        (Some(cex), Some(dex)) => Ok(Some((
            runtime_recovered_cex_lifecycle_summary(&cex)?,
            runtime_recovered_dex_lifecycle_summary(&dex)?,
        ))),
    }
}

fn runtime_recovered_cex_lifecycle_summary(
    checkpoint: &StateCheckpoint,
) -> Result<RuntimeRecoveredCexLifecycleSummary, RuntimeLifecycleError> {
    let record: CexOrderLifecycleRecord =
        serde_json::from_str(&checkpoint.value).map_err(|error| {
            RuntimeLifecycleError::ValidationFailed {
                reason: format!("failed to decode recovered CEX lifecycle checkpoint: {error}"),
            }
        })?;
    if record.external_submission_performed
        || record.live_execution_performed
        || record.production_ready
    {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "recovered CEX lifecycle checkpoint must remain local-only".to_owned(),
        });
    }
    Ok(RuntimeRecoveredCexLifecycleSummary {
        request_id: record.request_id,
        client_order_id: record.client_order_id,
        strategy_id: record.strategy_id,
        venue_name: record.venue.name,
        market_pair: format!("{}/{}", record.pair.base, record.pair.quote),
        final_status: format!("{:?}", record.final_status),
        transition_count: u64::try_from(record.transition_count).map_err(|_| {
            RuntimeLifecycleError::ValidationFailed {
                reason: "recovered CEX lifecycle transition count overflowed".to_owned(),
            }
        })?,
        fill_count: u64::try_from(record.fill_count).map_err(|_| {
            RuntimeLifecycleError::ValidationFailed {
                reason: "recovered CEX lifecycle fill count overflowed".to_owned(),
            }
        })?,
    })
}

fn runtime_recovered_dex_lifecycle_summary(
    checkpoint: &StateCheckpoint,
) -> Result<RuntimeRecoveredDexLifecycleSummary, RuntimeLifecycleError> {
    let record: DexSwapLifecycleRecord =
        serde_json::from_str(&checkpoint.value).map_err(|error| {
            RuntimeLifecycleError::ValidationFailed {
                reason: format!("failed to decode recovered DEX lifecycle checkpoint: {error}"),
            }
        })?;
    if record.rpc_call_performed
        || record.signing_performed
        || record.broadcast_performed
        || record.live_execution_performed
        || record.production_ready
    {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "recovered DEX lifecycle checkpoint must remain local-only".to_owned(),
        });
    }
    Ok(RuntimeRecoveredDexLifecycleSummary {
        request_id: record.request_id,
        strategy_id: record.strategy_id,
        venue_name: record.venue.name,
        chain: record.chain,
        market_pair: format!("{}/{}", record.pair.base, record.pair.quote),
        quote_response_id: record.quote_response_id,
        simulation_response_id: record.simulation_response_id,
        route_kind: format!("{:?}", record.route_kind),
        simulation_status: format!("{:?}", record.simulation_status),
        gas_used: record.gas_used,
    })
}

fn runtime_opportunity_trace_recovery_summary(
    report: OpportunityCandidateTraceRecoveryReport,
) -> Result<RuntimeOpportunityTraceRecoverySummary, RuntimeLifecycleError> {
    let discovered_candidates = u64::try_from(report.handoff_report.discovered_candidates)
        .map_err(|_| RuntimeLifecycleError::ValidationFailed {
            reason: "opportunity trace recovery discovered candidates count overflowed".to_owned(),
        })?;
    let audit_trace_records_replayed =
        u64::try_from(report.audit_replay_records).map_err(|_| {
            RuntimeLifecycleError::ValidationFailed {
                reason: "opportunity trace audit replay records count overflowed".to_owned(),
            }
        })?;
    let recovered_trace_checkpoints =
        u64::try_from(report.recovered_trace_checkpoints).map_err(|_| {
            RuntimeLifecycleError::ValidationFailed {
                reason: "opportunity trace recovered checkpoint count overflowed".to_owned(),
            }
        })?;
    let missing_trace_checkpoints =
        u64::try_from(report.missing_trace_checkpoints.len()).map_err(|_| {
            RuntimeLifecycleError::ValidationFailed {
                reason: "opportunity trace missing checkpoint count overflowed".to_owned(),
            }
        })?;
    let recovered_trace_summaries = report
        .recovered_trace_summaries
        .into_iter()
        .map(runtime_recovered_opportunity_trace_summary)
        .collect();

    Ok(RuntimeOpportunityTraceRecoverySummary {
        corpus_id: report.corpus_id,
        discovered_candidates,
        audit_trace_records_replayed,
        recovered_trace_checkpoints,
        missing_trace_checkpoints,
        recovered_trace_summaries,
        trace_recovery_validated: report.trace_recovery_validated,
    })
}

fn runtime_recovered_opportunity_trace_summary(
    summary: RecoveredOpportunityTraceSummary,
) -> RuntimeRecoveredOpportunityTraceSummary {
    RuntimeRecoveredOpportunityTraceSummary {
        trace_id: summary.trace_id,
        strategy_id: summary.strategy_id,
        planner_request_id: summary.planner_request_id,
        audit_sequence: summary.audit_sequence,
        traced_at_unix_ms: summary.traced_at_unix_ms,
        route_kind: format!("{:?}", summary.route_kind),
        leg_count: summary.leg_count,
    }
}

fn validate_runtime_opportunity_trace_recovery(
    audit_path: &Path,
    state_path: &Path,
    policy: &PolicyEngine,
) -> Result<RuntimeOpportunityTraceRecoverySummary, RuntimeLifecycleError> {
    let corpus = phase27_local_opportunity_historical_fixture_corpus().map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("failed to load local opportunity trace recovery corpus: {error}"),
        }
    })?;
    validate_runtime_opportunity_trace_recovery_for_corpus(&corpus, audit_path, state_path, policy)
}

fn validate_runtime_opportunity_trace_recovery_for_corpus(
    corpus: &OpportunityHistoricalFixtureCorpus,
    audit_path: &Path,
    state_path: &Path,
    policy: &PolicyEngine,
) -> Result<RuntimeOpportunityTraceRecoverySummary, RuntimeLifecycleError> {
    let trace_report = validate_opportunity_candidate_trace_restart_recovery(
        corpus, policy, audit_path, state_path,
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("opportunity trace recovery validation failed: {error}"),
    })?;
    runtime_opportunity_trace_recovery_summary(trace_report)
}

fn runtime_temp_path(prefix: &str) -> Result<PathBuf, RuntimeLifecycleError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("failed to construct temporary trace-recovery path: {error}"),
        })?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", process::id())))
}

fn cleanup_runtime_trace_recovery_paths(audit_path: &Path, state_path: &Path) {
    let _ = fs::remove_file(audit_path);
    let _ = fs::remove_file(state_path);
    for suffix in ["", "-wal", "-shm"] {
        let related = format!("{}{}", state_path.display(), suffix);
        let _ = fs::remove_file(related);
    }
}

/// Run a local deployment-like runtime smoke validation sequence.
///
/// The harness uses caller-supplied local paths, runs one lifecycle, records a
/// graceful-shutdown checkpoint, validates backup/restore and restart recovery,
/// and runs the audit durability probes in a separate workspace. It does not
/// start services, interact with a service manager, inspect deployment state,
/// submit orders, call networks, sign payloads, broadcast transactions,
/// withdraw funds, or bridge assets.
#[allow(clippy::too_many_lines)]
pub fn validate_local_runtime_deployment_smoke(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    backup_audit_path: impl AsRef<Path>,
    backup_state_path: impl AsRef<Path>,
    audit_validation_workspace: impl AsRef<Path>,
    policy: &PolicyEngine,
    request: RuntimeDeploymentSmokeValidationRequest,
) -> Result<RuntimeDeploymentSmokeValidationReport, RuntimeLifecycleError> {
    request.validate()?;

    let audit_path = audit_path.as_ref();
    let state_path = state_path.as_ref();
    let backup_audit_path = backup_audit_path.as_ref();
    let backup_state_path = backup_state_path.as_ref();
    let audit_validation_workspace = audit_validation_workspace.as_ref();

    validate_runtime_deployment_smoke_paths(
        audit_path,
        state_path,
        backup_audit_path,
        backup_state_path,
        audit_validation_workspace,
    )?;

    let runtime_smoke_plan = request.lifecycle_request.plan.clone();
    let concurrent_lifecycle_request = request.lifecycle_request.clone();
    let mut journal = AppendOnlyAuditJournal::open(audit_path)?;
    let mut store = SqliteWalStateStore::open(state_path)?;
    let lifecycle_record =
        run_local_runtime_lifecycle(&mut journal, &mut store, policy, request.lifecycle_request)?;
    let graceful_shutdown_record =
        run_local_graceful_shutdown_checkpoint(&mut journal, &mut store, request.shutdown_request)?;
    let communications_artifacts = record_local_runtime_smoke_communications(
        &mut journal,
        &mut store,
        &lifecycle_record,
        request.validated_at_unix_ms,
    )?;
    let observability_artifacts = record_local_runtime_smoke_observability(
        &mut journal,
        &mut store,
        &lifecycle_record,
        &graceful_shutdown_record,
        &communications_artifacts.dispatch,
        request.validated_at_unix_ms.saturating_add(1),
    )?;
    let dashboard_artifacts = record_local_runtime_smoke_dashboard(
        &mut journal,
        &mut store,
        &lifecycle_record,
        &communications_artifacts.dispatch,
        request.validated_at_unix_ms.saturating_add(2),
    )?;
    let validation_artifacts = record_local_runtime_smoke_testing(
        &mut journal,
        &mut store,
        &lifecycle_record,
        request.validated_at_unix_ms.saturating_add(3),
    )?;
    let paper_artifacts = record_local_runtime_smoke_paper_ledger(
        &mut journal,
        &mut store,
        policy,
        &runtime_smoke_plan,
        &lifecycle_record,
        request.validated_at_unix_ms.saturating_add(4),
    )?;
    let failure_capture_record = record_local_runtime_smoke_failure_capture(
        &mut journal,
        &mut store,
        &lifecycle_record,
        request.validated_at_unix_ms.saturating_add(5),
    )?;
    drop(store);
    drop(journal);

    let concurrent_lifecycle_report = validate_local_runtime_concurrent_lifecycle_access(
        audit_path,
        state_path,
        policy,
        &concurrent_lifecycle_request,
        request.validated_at_unix_ms.saturating_add(10),
    )?;
    let backup_report = validate_local_runtime_backup_restore(
        audit_path,
        state_path,
        backup_audit_path,
        backup_state_path,
    )?;
    let restart_report = validate_local_runtime_restart_recovery_with_trace_recovery(
        audit_path, state_path, policy,
    )?;
    let audit_report = validate_audit_journal_durability(
        audit_validation_workspace,
        request.validated_at_unix_ms,
    )?;

    let recovered_checkpoints = recover_runtime_smoke_checkpoints(state_path)?;

    let report = runtime_deployment_smoke_report(
        &lifecycle_record,
        &graceful_shutdown_record,
        &backup_report,
        &restart_report,
        &audit_report,
        &concurrent_lifecycle_report,
        &observability_artifacts,
        &communications_artifacts,
        &dashboard_artifacts,
        &validation_artifacts,
        &paper_artifacts,
        &failure_capture_record,
        recovered_checkpoints,
    );
    report.validate()?;
    Ok(report)
}

/// Build a non-mutating production-runtime preflight report from local smoke evidence.
///
/// This validates current local runtime evidence and reports the deployment-host
/// evidence that remains missing. It does not start services, mutate production
/// paths, call external systems, submit adapters, sign, broadcast, or claim
/// production readiness.
pub fn preflight_production_runtime_validation(
    request: RuntimeProductionPreflightRequest,
) -> Result<RuntimeProductionPreflightReport, RuntimeLifecycleError> {
    request.validate()?;
    let unresolved_blockers = runtime_production_preflight_unresolved_blockers(&request);
    let report = RuntimeProductionPreflightReport {
        validation_version: RUNTIME_PRODUCTION_PREFLIGHT_VALIDATION_VERSION.to_owned(),
        preflight_id: request.preflight_id,
        local_smoke_validated: true,
        local_smoke_load_validated: true,
        service_manager_lifecycle_evidence_available: request
            .service_manager_lifecycle_evidence_available,
        deployment_host_permission_evidence_available: request
            .deployment_host_permission_evidence_available,
        physical_disk_full_evidence_available: request.physical_disk_full_evidence_available,
        retention_execution_evidence_available: request.retention_execution_evidence_available,
        rollback_drill_evidence_available: request.rollback_drill_evidence_available,
        incident_response_evidence_available: request.incident_response_evidence_available,
        observability_runtime_evidence_available: request.observability_runtime_evidence_available,
        status: RuntimeProductionPreflightStatus::BlockedPendingProductionHostValidation,
        service_manager_action_performed: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: request.validated_at_unix_ms,
        unresolved_blockers,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized service-manager lifecycle transcript metadata.
///
/// This consumes operator-supplied event/reference metadata only. It does not
/// call `systemctl`, start/stop/restart services, read deployment logs, mutate
/// production paths, submit adapters, perform live execution, or claim
/// production readiness.
pub fn validate_service_manager_lifecycle_transcript(
    transcript: RuntimeServiceManagerLifecycleTranscript,
) -> Result<RuntimeServiceManagerLifecycleTranscriptReport, RuntimeLifecycleError> {
    transcript.validate()?;
    let start_evidence_present =
        transcript.has_successful_reference(RuntimeServiceManagerLifecycleEventKind::Started);
    let runtime_smoke_event_present = transcript
        .has_successful_reference(RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed);
    let graceful_shutdown_evidence_present = transcript.has_successful_reference(
        RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
    );
    let stop_evidence_present =
        transcript.has_successful_reference(RuntimeServiceManagerLifecycleEventKind::Stopped);
    let restart_evidence_present =
        transcript.has_successful_reference(RuntimeServiceManagerLifecycleEventKind::Restarted);
    let recovery_event_present = transcript
        .has_successful_reference(RuntimeServiceManagerLifecycleEventKind::RecoveryValidated);
    let operator_controlled_events = transcript
        .events
        .iter()
        .all(|event| event.operator_controlled);
    let non_secret_references_present = transcript
        .events
        .iter()
        .all(|event| event.non_secret_reference_present);
    let successful_event_outcomes = transcript.events.iter().all(|event| event.outcome_success);
    let runtime_smoke_evidence_present =
        runtime_smoke_event_present && transcript.runtime_smoke_reference_present;
    let recovery_evidence_present = recovery_event_present
        && transcript.audit_replay_reference_present
        && transcript.sqlite_recovery_reference_present;
    let concurrent_lifecycle_evidence_present = transcript.concurrent_lifecycle_reference_present
        && transcript.concurrent_lifecycle_worker_count >= 2
        && transcript.concurrent_lifecycle_success;
    let blocker_codes = service_manager_lifecycle_blockers(
        &transcript,
        ServiceManagerLifecycleEvidence {
            start_evidence_present,
            runtime_smoke_evidence_present,
            graceful_shutdown_evidence_present,
            stop_evidence_present,
            restart_evidence_present,
            recovery_evidence_present,
            operator_controlled_events,
            non_secret_references_present,
            successful_event_outcomes,
            concurrent_lifecycle_evidence_present,
        },
    );
    let status = if blocker_codes.is_empty() {
        RuntimeServiceManagerLifecycleTranscriptStatus::ReadyForExternalReview
    } else {
        RuntimeServiceManagerLifecycleTranscriptStatus::Blocked
    };
    let report = RuntimeServiceManagerLifecycleTranscriptReport {
        validation_version: RUNTIME_SERVICE_MANAGER_LIFECYCLE_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        service_manager: transcript.service_manager,
        unit_name: transcript.unit_name,
        event_count: transcript.events.len() as u64,
        start_evidence_present,
        runtime_smoke_evidence_present,
        graceful_shutdown_evidence_present,
        stop_evidence_present,
        restart_evidence_present,
        recovery_evidence_present,
        operator_controlled_events,
        non_secret_references_present,
        successful_event_outcomes,
        audit_replay_reference_present: transcript.audit_replay_reference_present,
        sqlite_recovery_reference_present: transcript.sqlite_recovery_reference_present,
        concurrent_lifecycle_reference_present: transcript.concurrent_lifecycle_reference_present,
        concurrent_lifecycle_worker_count: transcript.concurrent_lifecycle_worker_count,
        concurrent_lifecycle_success: transcript.concurrent_lifecycle_success,
        operator_approved: transcript.operator_approved,
        operator_lifecycle_rehearsal_reference_present: transcript
            .operator_lifecycle_rehearsal_reference_present,
        emergency_stop_review_reference_present: transcript.emergency_stop_review_reference_present,
        rollback_plan_review_reference_present: transcript.rollback_plan_review_reference_present,
        operator_review_window_current: transcript.operator_review_window_current,
        status,
        blocker_codes,
        service_manager_action_performed_by_validator: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a local-only service-manager lifecycle rehearsal.
///
/// This consumes sanitized event/reference metadata only. It does not call
/// service managers, start/stop/restart services, mutate deployment paths,
/// inspect logs, load secrets, submit adapters, perform live execution, or
/// claim production readiness.
pub fn validate_service_manager_lifecycle_rehearsal(
    request: RuntimeServiceManagerLifecycleRehearsalRequest,
) -> Result<RuntimeServiceManagerLifecycleRehearsalReport, RuntimeLifecycleError> {
    request.validate()?;
    let start_evidence_present = rehearsal_has_successful_reference(
        &request,
        RuntimeServiceManagerLifecycleEventKind::Started,
    );
    let runtime_smoke_event_present = rehearsal_has_successful_reference(
        &request,
        RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed,
    );
    let graceful_shutdown_evidence_present = rehearsal_has_successful_reference(
        &request,
        RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
    );
    let stop_evidence_present = rehearsal_has_successful_reference(
        &request,
        RuntimeServiceManagerLifecycleEventKind::Stopped,
    );
    let restart_evidence_present = rehearsal_has_successful_reference(
        &request,
        RuntimeServiceManagerLifecycleEventKind::Restarted,
    );
    let recovery_event_present = rehearsal_has_successful_reference(
        &request,
        RuntimeServiceManagerLifecycleEventKind::RecoveryValidated,
    );
    let ordered_lifecycle_validated = service_manager_lifecycle_order_validated(&request.events);
    let operator_controlled_events = request.events.iter().all(|event| event.operator_controlled);
    let non_secret_references_present = request
        .events
        .iter()
        .all(|event| event.non_secret_reference_present);
    let successful_event_outcomes = request.events.iter().all(|event| event.outcome_success);
    let runtime_smoke_evidence_present =
        runtime_smoke_event_present && request.runtime_smoke_reference_present;
    let recovery_evidence_present = recovery_event_present
        && request.audit_replay_reference_present
        && request.sqlite_recovery_reference_present
        && request.restart_recovery_reference_present;
    let blocker_codes = service_manager_lifecycle_rehearsal_blockers(
        &request,
        ServiceManagerLifecycleRehearsalEvidence {
            ordered_lifecycle_validated,
            operator_controlled_events,
            non_secret_references_present,
            successful_event_outcomes,
            start_evidence_present,
            runtime_smoke_evidence_present,
            graceful_shutdown_evidence_present,
            stop_evidence_present,
            restart_evidence_present,
            recovery_evidence_present,
        },
    );
    let status = if blocker_codes.is_empty() {
        RuntimeServiceManagerLifecycleRehearsalStatus::Validated
    } else {
        RuntimeServiceManagerLifecycleRehearsalStatus::Blocked
    };
    let report = RuntimeServiceManagerLifecycleRehearsalReport {
        validation_version: RUNTIME_SERVICE_MANAGER_LIFECYCLE_REHEARSAL_VERSION.to_owned(),
        rehearsal_id: request.rehearsal_id,
        service_manager: request.service_manager,
        unit_name: request.unit_name,
        event_count: request.events.len() as u64,
        ordered_lifecycle_validated,
        operator_controlled_events,
        non_secret_references_present,
        successful_event_outcomes,
        start_evidence_present,
        runtime_smoke_evidence_present,
        graceful_shutdown_evidence_present,
        stop_evidence_present,
        restart_evidence_present,
        recovery_evidence_present,
        audit_replay_reference_present: request.audit_replay_reference_present,
        sqlite_recovery_reference_present: request.sqlite_recovery_reference_present,
        concurrent_lifecycle_reference_present: request.concurrent_lifecycle_reference_present,
        concurrent_lifecycle_worker_count: request.concurrent_lifecycle_worker_count,
        concurrent_lifecycle_success: request.concurrent_lifecycle_success,
        graceful_shutdown_checkpoint_reference_present: request
            .graceful_shutdown_checkpoint_reference_present,
        restart_recovery_reference_present: request.restart_recovery_reference_present,
        operator_approved: request.operator_approved,
        reviewer_approved: request.reviewer_approved,
        status,
        blocker_codes,
        service_manager_action_performed_by_validator: false,
        deployment_path_mutated_by_validator: false,
        secrets_loaded: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: request.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized deployment-host filesystem permission evidence metadata.
///
/// This consumes operator-supplied reference metadata only. It does not change
/// permissions, inspect deployment paths, mutate production paths, call
/// service managers, submit adapters, perform live execution, or claim
/// production readiness.
pub fn validate_deployment_permission_transcript(
    transcript: RuntimeDeploymentPermissionTranscript,
) -> Result<RuntimeDeploymentPermissionTranscriptReport, RuntimeLifecycleError> {
    transcript.validate()?;
    let blocker_codes = deployment_permission_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        RuntimeDeploymentPermissionTranscriptStatus::ReadyForExternalReview
    } else {
        RuntimeDeploymentPermissionTranscriptStatus::Blocked
    };
    let report = RuntimeDeploymentPermissionTranscriptReport {
        validation_version: RUNTIME_DEPLOYMENT_PERMISSION_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        host_label: transcript.host_label,
        deployment_host_evidence: transcript.deployment_host_evidence,
        runtime_write_attempt_reference_present: transcript.runtime_write_attempt_reference_present,
        runtime_write_permission_denied: transcript.runtime_write_permission_denied,
        runtime_write_error_classified: transcript.runtime_write_error_classified,
        audit_write_failed_closed: transcript.audit_write_failed_closed,
        state_write_failed_closed: transcript.state_write_failed_closed,
        adapter_evaluation_blocked: transcript.adapter_evaluation_blocked,
        runtime_quiesced_or_degraded: transcript.runtime_quiesced_or_degraded,
        audit_replay_after_restore_validated: transcript.audit_replay_after_restore_validated,
        sqlite_reopen_after_restore_validated: transcript.sqlite_reopen_after_restore_validated,
        recovery_runbook_reference_present: transcript.recovery_runbook_reference_present,
        non_secret_reference_count: transcript.non_secret_reference_count,
        operator_approved: transcript.operator_approved,
        status,
        blocker_codes,
        permission_changed_by_validator: false,
        production_path_mutated_by_validator: false,
        service_manager_action_performed_by_validator: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized deployment-host audit and SQLite recovery evidence metadata.
///
/// This consumes operator-supplied reference metadata only. It does not inspect
/// deployment paths, mutate deployment paths, call service managers, load
/// secrets, submit adapters, perform live execution, or claim production
/// readiness.
pub fn validate_deployment_audit_sqlite_transcript(
    transcript: RuntimeDeploymentAuditSqliteTranscript,
) -> Result<RuntimeDeploymentAuditSqliteTranscriptReport, RuntimeLifecycleError> {
    transcript.validate()?;
    let blocker_codes = deployment_audit_sqlite_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        RuntimeDeploymentAuditSqliteTranscriptStatus::ReadyForExternalReview
    } else {
        RuntimeDeploymentAuditSqliteTranscriptStatus::Blocked
    };
    let report = RuntimeDeploymentAuditSqliteTranscriptReport {
        validation_version: RUNTIME_DEPLOYMENT_AUDIT_SQLITE_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        host_label: transcript.host_label,
        deployment_host_evidence: transcript.deployment_host_evidence,
        service_lifecycle_reference_present: transcript.service_lifecycle_reference_present,
        audit_append_reference_present: transcript.audit_append_reference_present,
        audit_replay_validated: transcript.audit_replay_validated,
        audit_hash_chain_validated: transcript.audit_hash_chain_validated,
        sqlite_wal_mode_validated: transcript.sqlite_wal_mode_validated,
        sqlite_integrity_check_passed: transcript.sqlite_integrity_check_passed,
        sqlite_checkpoint_recovered: transcript.sqlite_checkpoint_recovered,
        backup_restore_validated: transcript.backup_restore_validated,
        concurrent_access_validated: transcript.concurrent_access_validated,
        recovery_runbook_reference_present: transcript.recovery_runbook_reference_present,
        non_secret_reference_count: transcript.non_secret_reference_count,
        operator_approved: transcript.operator_approved,
        reviewer_approved: transcript.reviewer_approved,
        status,
        blocker_codes,
        service_manager_action_performed_by_validator: false,
        deployment_path_mutated_by_validator: false,
        secrets_loaded: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized deployment-host backup/restore evidence metadata.
///
/// This consumes operator-supplied reference metadata only. It does not execute
/// backups or restores, inspect deployment paths, mutate deployment paths, call
/// service managers, load secrets, submit adapters, perform live execution, or
/// claim production readiness.
pub fn validate_deployment_backup_restore_transcript(
    transcript: RuntimeDeploymentBackupRestoreTranscript,
) -> Result<RuntimeDeploymentBackupRestoreTranscriptReport, RuntimeLifecycleError> {
    transcript.validate()?;
    let blocker_codes = deployment_backup_restore_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        RuntimeDeploymentBackupRestoreTranscriptStatus::ReadyForExternalReview
    } else {
        RuntimeDeploymentBackupRestoreTranscriptStatus::Blocked
    };
    let report = RuntimeDeploymentBackupRestoreTranscriptReport {
        validation_version: RUNTIME_DEPLOYMENT_BACKUP_RESTORE_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        host_label: transcript.host_label,
        deployment_host_evidence: transcript.deployment_host_evidence,
        service_lifecycle_reference_present: transcript.service_lifecycle_reference_present,
        backup_artifact_reference_present: transcript.backup_artifact_reference_present,
        restore_execution_reference_present: transcript.restore_execution_reference_present,
        deployment_load_reference_present: transcript.deployment_load_reference_present,
        audit_replay_after_restore_validated: transcript.audit_replay_after_restore_validated,
        audit_hash_chain_after_restore_validated: transcript
            .audit_hash_chain_after_restore_validated,
        sqlite_integrity_after_restore_validated: transcript
            .sqlite_integrity_after_restore_validated,
        sqlite_checkpoint_after_restore_validated: transcript
            .sqlite_checkpoint_after_restore_validated,
        runtime_checkpoint_restore_validated: transcript.runtime_checkpoint_restore_validated,
        post_restore_runtime_smoke_passed: transcript.post_restore_runtime_smoke_passed,
        rollback_reference_present: transcript.rollback_reference_present,
        recovery_runbook_reference_present: transcript.recovery_runbook_reference_present,
        non_secret_reference_count: transcript.non_secret_reference_count,
        operator_approved: transcript.operator_approved,
        reviewer_approved: transcript.reviewer_approved,
        status,
        blocker_codes,
        backup_restore_executed_by_validator: false,
        service_manager_action_performed_by_validator: false,
        deployment_path_mutated_by_validator: false,
        secrets_loaded: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized deployment-host graceful-shutdown evidence metadata.
///
/// This consumes operator-supplied reference metadata only. It does not stop
/// services, inspect deployment paths, mutate deployment paths, call service
/// managers, load secrets, submit adapters, perform live execution, or claim
/// production readiness.
pub fn validate_deployment_graceful_shutdown_transcript(
    transcript: RuntimeDeploymentGracefulShutdownTranscript,
) -> Result<RuntimeDeploymentGracefulShutdownTranscriptReport, RuntimeLifecycleError> {
    transcript.validate()?;
    let blocker_codes = deployment_graceful_shutdown_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        RuntimeDeploymentGracefulShutdownTranscriptStatus::ReadyForExternalReview
    } else {
        RuntimeDeploymentGracefulShutdownTranscriptStatus::Blocked
    };
    let report = RuntimeDeploymentGracefulShutdownTranscriptReport {
        validation_version: RUNTIME_DEPLOYMENT_GRACEFUL_SHUTDOWN_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        host_label: transcript.host_label,
        deployment_host_evidence: transcript.deployment_host_evidence,
        service_lifecycle_reference_present: transcript.service_lifecycle_reference_present,
        shutdown_request_reference_present: transcript.shutdown_request_reference_present,
        service_stopped_reference_present: transcript.service_stopped_reference_present,
        graceful_shutdown_checkpoint_reference_present: transcript
            .graceful_shutdown_checkpoint_reference_present,
        audit_replay_after_shutdown_validated: transcript.audit_replay_after_shutdown_validated,
        sqlite_reopen_after_shutdown_validated: transcript.sqlite_reopen_after_shutdown_validated,
        restart_recovery_after_shutdown_validated: transcript
            .restart_recovery_after_shutdown_validated,
        post_shutdown_runtime_smoke_passed: transcript.post_shutdown_runtime_smoke_passed,
        operator_approved: transcript.operator_approved,
        reviewer_approved: transcript.reviewer_approved,
        non_secret_reference_count: transcript.non_secret_reference_count,
        status,
        blocker_codes,
        service_manager_action_performed_by_validator: false,
        deployment_path_mutated_by_validator: false,
        secrets_loaded: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized deployment-host SQLite schema migration evidence metadata.
///
/// This consumes operator-supplied reference metadata only. It does not execute
/// migrations, inspect deployment paths, mutate deployment paths, call service
/// managers, load secrets, submit adapters, perform live execution, or claim
/// production readiness.
pub fn validate_deployment_sqlite_schema_migration_transcript(
    transcript: RuntimeDeploymentSqliteSchemaMigrationTranscript,
) -> Result<RuntimeDeploymentSqliteSchemaMigrationTranscriptReport, RuntimeLifecycleError> {
    transcript.validate()?;
    let blocker_codes = deployment_sqlite_schema_migration_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::ReadyForExternalReview
    } else {
        RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::Blocked
    };
    let report = RuntimeDeploymentSqliteSchemaMigrationTranscriptReport {
        validation_version: RUNTIME_DEPLOYMENT_SQLITE_SCHEMA_MIGRATION_TRANSCRIPT_VERSION
            .to_owned(),
        transcript_id: transcript.transcript_id,
        host_label: transcript.host_label,
        deployment_host_evidence: transcript.deployment_host_evidence,
        service_lifecycle_reference_present: transcript.service_lifecycle_reference_present,
        pre_migration_schema_version: transcript.pre_migration_schema_version,
        post_migration_schema_version: transcript.post_migration_schema_version,
        expected_schema_version: transcript.expected_schema_version,
        pre_migration_backup_reference_present: transcript.pre_migration_backup_reference_present,
        migration_execution_reference_present: transcript.migration_execution_reference_present,
        schema_version_transition_validated: transcript.schema_version_transition_validated,
        sqlite_integrity_check_passed: transcript.sqlite_integrity_check_passed,
        sqlite_checkpoint_reopened: transcript.sqlite_checkpoint_reopened,
        audit_replay_after_migration_validated: transcript.audit_replay_after_migration_validated,
        rollback_reference_present: transcript.rollback_reference_present,
        runtime_quiesced_or_degraded: transcript.runtime_quiesced_or_degraded,
        non_secret_reference_count: transcript.non_secret_reference_count,
        operator_approved: transcript.operator_approved,
        reviewer_approved: transcript.reviewer_approved,
        status,
        blocker_codes,
        migration_executed_by_validator: false,
        service_manager_action_performed_by_validator: false,
        deployment_path_mutated_by_validator: false,
        secrets_loaded: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

fn recover_runtime_smoke_checkpoints(
    state_path: &Path,
) -> Result<RuntimeSmokeRecoveredCheckpoints, RuntimeLifecycleError> {
    let reopened_store = SqliteWalStateStore::open(state_path)?;
    Ok(RuntimeSmokeRecoveredCheckpoints {
        observability_record: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY)?
            .is_some(),
        observability_operations_review: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY)?
            .is_some(),
        observability_export_dry_run: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY)?
            .is_some(),
        observability_alert_route_dispatch: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY)?
            .is_some(),
        observability_endpoint_preflight: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY)?
            .is_some(),
        observability_loopback_bind: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY)?
            .is_some(),
        observability_metrics_scrape: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY)?
            .is_some(),
        observability_metrics_endpoint: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY)?
            .is_some(),
        observability_tracing: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY)?
            .is_some(),
        communications_route: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY)?
            .is_some(),
        communications_remote_review: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY)?
            .is_some(),
        communications_platform_ingress: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY)?
            .is_some(),
        communications_remote_envelope: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY)?
            .is_some(),
        communications_channel_adapter: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY)?
            .is_some(),
        communications_channel_session: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY)?
            .is_some(),
        communications_platform_adapter: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY)?
            .is_some(),
        communications_notification: reopened_store
            .get_checkpoint(COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY)?
            .is_some(),
        dashboard_render: reopened_store
            .get_checkpoint(DASHBOARD_LAST_RENDER_CHECKPOINT_KEY)?
            .is_some(),
        dashboard_hosted_security: reopened_store
            .get_checkpoint(DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY)?
            .is_some(),
        dashboard_hosted_preflight: reopened_store
            .get_checkpoint(DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY)?
            .is_some(),
        dashboard_hosted_validation: reopened_store
            .get_checkpoint(DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY)?
            .is_some(),
        validation_run: reopened_store
            .get_checkpoint(TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY)?
            .is_some(),
        property_check: reopened_store
            .get_checkpoint(TESTING_LAST_PROPERTY_CHECK_REPORT_KEY)?
            .is_some(),
        paper_execution_report: reopened_store
            .get_checkpoint(PAPER_EXECUTION_LAST_REPORT_CHECKPOINT_KEY)?
            .is_some(),
        paper_ledger: reopened_store
            .get_checkpoint(PAPER_BALANCE_LEDGER_CHECKPOINT_KEY)?
            .is_some(),
        failure_capture: reopened_store
            .get_checkpoint(OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY)?
            .is_some(),
        adapter_recovery_plan: reopened_store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)?
            .is_some(),
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn runtime_deployment_smoke_report(
    lifecycle_record: &RuntimeLifecycleRecord,
    graceful_shutdown_record: &RuntimeGracefulShutdownRecord,
    backup_report: &RuntimeBackupRestoreValidationReport,
    restart_report: &RuntimeRestartRecoveryValidationReport,
    audit_report: &crate::AuditDurabilityValidationReport,
    concurrent_lifecycle_report: &RuntimeConcurrentLifecycleValidationReport,
    observability_artifacts: &RuntimeSmokeObservabilityArtifacts,
    communications_artifacts: &RuntimeSmokeCommunicationsArtifacts,
    dashboard_artifacts: &RuntimeSmokeDashboardArtifacts,
    validation_artifacts: &RuntimeSmokeTestingArtifacts,
    paper_artifacts: &RuntimeSmokePaperArtifacts,
    failure_capture_record: &RuntimeFailureCaptureRecord,
    recovered_checkpoints: RuntimeSmokeRecoveredCheckpoints,
) -> RuntimeDeploymentSmokeValidationReport {
    let paper_ledger_report = paper_artifacts.ledger_report.as_ref();
    RuntimeDeploymentSmokeValidationReport {
        validation_version: RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION.to_owned(),
        lifecycle_completed: lifecycle_record.status
            == RuntimeLifecycleStatus::AdapterRunCheckpointed,
        graceful_shutdown_checkpointed: graceful_shutdown_record.shutdown_checkpoint_key
            == RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY,
        backup_restore_validated: backup_report.audit_restore_check_passed
            && backup_report.sqlite_restore_check_passed
            && backup_report.plan_checkpoint_restored
            && backup_report.adapter_checkpoint_restored
            && backup_report.adapter_recovery_plan_checkpoint_restored,
        restart_recovery_validated: runtime_smoke_restart_recovery_validated(restart_report),
        audit_durability_validated: runtime_smoke_audit_durability_validated(audit_report),
        concurrent_lifecycle_validated: runtime_smoke_concurrent_lifecycle_validated(
            concurrent_lifecycle_report,
        ),
        concurrent_lifecycle_workers: concurrent_lifecycle_report.workers_completed,
        concurrent_lifecycle_audit_records_replayed: concurrent_lifecycle_report
            .audit_records_replayed,
        concurrent_lifecycle_sqlite_integrity_check_passed: concurrent_lifecycle_report
            .sqlite_integrity_check_passed,
        concurrent_lifecycle_external_submission_performed: concurrent_lifecycle_report
            .external_submission_performed,
        concurrent_lifecycle_live_execution_performed: concurrent_lifecycle_report
            .live_execution_performed,
        observability_collected: observability_artifacts.record.access_authorized,
        observability_checkpoint_recovered: recovered_checkpoints.observability_record,
        observability_operations_reviewed: observability_artifacts.operations_review.status
            == ObservabilityOperationsReviewStatus::ReadyForLocalReview,
        observability_operations_checkpoint_recovered: recovered_checkpoints
            .observability_operations_review,
        observability_export_dry_run_rendered: !observability_artifacts
            .export_dry_run
            .prometheus_metric_lines
            .is_empty(),
        observability_export_checkpoint_recovered: recovered_checkpoints
            .observability_export_dry_run,
        observability_alert_route_dispatched: observability_artifacts.alert_route_dispatch.status
            == ObservabilityAlertRouteDispatchStatus::ReadyForLocalReview,
        observability_alert_route_checkpoint_recovered: recovered_checkpoints
            .observability_alert_route_dispatch,
        observability_endpoint_preflighted: observability_artifacts.endpoint_preflight.status
            == ObservabilityEndpointPreflightStatus::ReadyForLocalReview,
        observability_endpoint_checkpoint_recovered: recovered_checkpoints
            .observability_endpoint_preflight,
        observability_loopback_bind_validated: observability_artifacts.loopback_bind.status
            == ObservabilityLoopbackBindValidationStatus::ReadyForLocalReview
            && observability_artifacts
                .loopback_bind
                .listener_opened_and_closed,
        observability_loopback_bind_checkpoint_recovered: recovered_checkpoints
            .observability_loopback_bind,
        observability_metrics_scrape_preflighted: observability_artifacts.metrics_scrape.status
            == ObservabilityMetricsScrapePreflightStatus::ReadyForLocalReview,
        observability_metrics_scrape_checkpoint_recovered: recovered_checkpoints
            .observability_metrics_scrape,
        observability_metrics_endpoint_validated: observability_artifacts.metrics_endpoint.status
            == ObservabilityMetricsEndpointValidationStatus::ReadyForLocalReview,
        observability_metrics_endpoint_checkpoint_recovered: recovered_checkpoints
            .observability_metrics_endpoint,
        observability_tracing_captured: observability_artifacts.tracing.status
            == LocalTracingSubscriberValidationStatus::ReadyForLocalReview
            && observability_artifacts.tracing.event_captured,
        observability_tracing_checkpoint_recovered: recovered_checkpoints.observability_tracing,
        observability_metrics_endpoint_started: observability_artifacts
            .record
            .metrics_endpoint_started
            || observability_artifacts
                .operations_review
                .metrics_endpoint_started
            || observability_artifacts
                .export_dry_run
                .metrics_endpoint_started
            || observability_artifacts
                .endpoint_preflight
                .metrics_endpoint_started
            || observability_artifacts
                .loopback_bind
                .metrics_endpoint_started
            || observability_artifacts
                .metrics_scrape
                .metrics_endpoint_started,
        observability_local_metrics_request_served: observability_artifacts
            .metrics_endpoint
            .network_request_served,
        observability_public_network_exposed: observability_artifacts.record.public_network_exposed
            || observability_artifacts
                .operations_review
                .public_network_exposed
            || observability_artifacts
                .export_dry_run
                .public_network_exposed
            || observability_artifacts
                .alert_route_dispatch
                .outbound_network_used
            || observability_artifacts
                .endpoint_preflight
                .public_network_exposed
            || observability_artifacts.loopback_bind.public_network_exposed
            || observability_artifacts
                .metrics_scrape
                .public_network_exposed
            || observability_artifacts
                .metrics_endpoint
                .public_network_exposed
            || observability_artifacts.tracing.public_network_exposed,
        observability_outbound_alerts_sent: observability_artifacts.record.outbound_alerts_sent
            || observability_artifacts
                .operations_review
                .outbound_alerts_sent
            || observability_artifacts.export_dry_run.outbound_alerts_sent
            || observability_artifacts
                .alert_route_dispatch
                .outbound_alerts_sent
            || observability_artifacts
                .endpoint_preflight
                .outbound_alerts_sent
            || observability_artifacts.loopback_bind.outbound_alerts_sent
            || observability_artifacts.metrics_scrape.outbound_alerts_sent
            || observability_artifacts
                .metrics_endpoint
                .outbound_alerts_sent
            || observability_artifacts.tracing.outbound_alerts_sent,
        observability_telemetry_exported: observability_artifacts
            .operations_review
            .telemetry_exported
            || observability_artifacts.export_dry_run.telemetry_exported
            || observability_artifacts
                .alert_route_dispatch
                .telemetry_exported
            || observability_artifacts
                .endpoint_preflight
                .telemetry_exported
            || observability_artifacts.loopback_bind.telemetry_exported
            || observability_artifacts.metrics_scrape.telemetry_exported
            || observability_artifacts.metrics_endpoint.telemetry_exported
            || observability_artifacts.tracing.telemetry_exported,
        observability_production_ready: observability_artifacts.operations_review.production_ready
            || observability_artifacts.export_dry_run.production_ready
            || observability_artifacts
                .alert_route_dispatch
                .production_ready
            || observability_artifacts.endpoint_preflight.production_ready
            || observability_artifacts.loopback_bind.production_ready
            || observability_artifacts.metrics_scrape.production_ready
            || observability_artifacts.metrics_endpoint.production_ready
            || observability_artifacts.tracing.production_ready,
        communications_command_routed: communications_artifacts.route.accepted
            && communications_artifacts.route.operator_authorized,
        communications_command_route_checkpoint_recovered: recovered_checkpoints
            .communications_route,
        communications_remote_command_reviewed: communications_artifacts.remote_review.status
            == RemoteCommandSecurityReviewStatus::ReadyForLocalReview,
        communications_remote_command_review_checkpoint_recovered: recovered_checkpoints
            .communications_remote_review,
        communications_platform_command_ingress_validated: communications_artifacts
            .platform_ingress
            .status
            == PlatformCommandIngressStatus::ReadyForEnvelopeValidation,
        communications_platform_command_ingress_checkpoint_recovered: recovered_checkpoints
            .communications_platform_ingress,
        communications_remote_command_envelope_validated: communications_artifacts
            .remote_envelope
            .status
            == RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview,
        communications_remote_command_envelope_checkpoint_recovered: recovered_checkpoints
            .communications_remote_envelope,
        communications_channel_adapter_validated: communications_artifacts.channel_adapter.status
            == ChannelAdapterValidationStatus::ReadyForLocalReview,
        communications_channel_adapter_checkpoint_recovered: recovered_checkpoints
            .communications_channel_adapter,
        communications_channel_session_validated: communications_artifacts.channel_session.status
            == ChannelSessionValidationStatus::ReadyForLocalReview,
        communications_channel_session_checkpoint_recovered: recovered_checkpoints
            .communications_channel_session,
        communications_platform_adapter_reviewed: communications_artifacts.platform_adapter.status
            == PlatformAdapterReviewStatus::ReadyForLocalReview,
        communications_platform_adapter_checkpoint_recovered: recovered_checkpoints
            .communications_platform_adapter,
        communications_notification_dispatched: communications_artifacts.dispatch.status
            == NotificationDispatchStatus::RecordedLocally,
        communications_notification_checkpoint_recovered: recovered_checkpoints
            .communications_notification,
        communications_execution_enabled: communications_artifacts.route.execution_enabled,
        communications_remote_commands_enabled: communications_artifacts
            .remote_review
            .remote_commands_enabled
            || communications_artifacts
                .platform_ingress
                .remote_commands_enabled
            || communications_artifacts
                .remote_envelope
                .remote_commands_enabled
            || communications_artifacts
                .channel_adapter
                .remote_commands_enabled
            || communications_artifacts
                .platform_adapter
                .remote_commands_enabled,
        communications_outbound_network_used: communications_artifacts.route.outbound_network_used
            || communications_artifacts.remote_review.outbound_network_used
            || communications_artifacts
                .platform_ingress
                .outbound_network_used
            || communications_artifacts.platform_ingress.message_delivered
            || communications_artifacts
                .remote_envelope
                .outbound_network_used
            || communications_artifacts
                .channel_adapter
                .outbound_network_used
            || communications_artifacts.channel_adapter.message_delivered
            || communications_artifacts
                .channel_session
                .outbound_network_used
            || communications_artifacts.channel_session.message_delivered
            || communications_artifacts
                .platform_adapter
                .outbound_network_used
            || communications_artifacts.platform_adapter.message_delivered
            || communications_artifacts.dispatch.outbound_network_used,
        dashboard_rendered: dashboard_artifacts.render.access_authorized,
        dashboard_checkpoint_recovered: recovered_checkpoints.dashboard_render,
        dashboard_hosted_security_reviewed: dashboard_artifacts.hosted_security.status
            == DashboardHostedSecurityReviewStatus::ReadyForLocalReview,
        dashboard_hosted_security_checkpoint_recovered: recovered_checkpoints
            .dashboard_hosted_security,
        dashboard_hosted_request_preflighted: dashboard_artifacts.hosted_preflight.status
            == DashboardHostedRequestPreflightStatus::ReadyForLocalReview,
        dashboard_hosted_request_preflight_checkpoint_recovered: recovered_checkpoints
            .dashboard_hosted_preflight,
        dashboard_hosted_request_validated: dashboard_artifacts.hosted_validation.status
            == DashboardHostedRequestValidationStatus::ReadyForLocalReview,
        dashboard_hosted_request_validation_checkpoint_recovered: recovered_checkpoints
            .dashboard_hosted_validation,
        dashboard_panel_count: dashboard_artifacts.render.panels.len() as u64,
        dashboard_server_started: dashboard_artifacts.render.server_started
            || dashboard_artifacts.hosted_security.server_started
            || dashboard_artifacts.hosted_preflight.server_started,
        dashboard_local_one_shot_request_served: dashboard_artifacts
            .hosted_validation
            .network_request_served,
        dashboard_public_network_exposed: dashboard_artifacts.render.public_network_exposed
            || dashboard_artifacts.hosted_security.public_network_exposed
            || dashboard_artifacts.hosted_preflight.public_network_exposed
            || dashboard_artifacts.hosted_validation.public_network_exposed,
        dashboard_live_controls_enabled: dashboard_artifacts.render.live_controls_enabled
            || dashboard_artifacts.hosted_security.live_controls_enabled
            || dashboard_artifacts.hosted_preflight.live_controls_enabled
            || dashboard_artifacts.hosted_validation.live_controls_enabled,
        dashboard_hosted_production_ready: dashboard_artifacts.hosted_security.production_ready
            || dashboard_artifacts.hosted_preflight.production_ready
            || dashboard_artifacts.hosted_validation.production_ready,
        validation_run_recorded: validation_artifacts.validation_run.status
            == ValidationRunStatus::PlannedOnly,
        validation_run_checkpoint_recovered: recovered_checkpoints.validation_run,
        validation_property_checks_passed: validation_artifacts.property_check.checks_executed > 0
            && validation_artifacts.property_check.checks_failed == 0,
        validation_property_checkpoint_recovered: recovered_checkpoints.property_check,
        validation_external_fuzzer_invoked: validation_artifacts
            .validation_run
            .external_fuzzer_invoked
            || validation_artifacts.property_check.external_fuzzer_invoked,
        validation_live_network_used: validation_artifacts.validation_run.live_network_used
            || validation_artifacts.property_check.live_network_used,
        validation_live_execution_submitted: validation_artifacts
            .validation_run
            .live_execution_submitted
            || validation_artifacts.property_check.live_execution_submitted,
        validation_signing_or_broadcast_performed: validation_artifacts
            .validation_run
            .signing_or_broadcast_performed
            || validation_artifacts
                .property_check
                .signing_or_broadcast_performed,
        paper_ledger_applicable: paper_artifacts.applicable,
        paper_execution_report_checkpointed: paper_artifacts.execution_report_checkpointed,
        paper_execution_report_checkpoint_recovered: recovered_checkpoints.paper_execution_report,
        paper_ledger_checkpointed: paper_ledger_report
            .is_some_and(|report| report.ledger_checkpoint_persisted),
        paper_ledger_checkpoint_recovered: recovered_checkpoints.paper_ledger,
        paper_modeled_fills_settled: paper_ledger_report.map_or(0, |report| {
            u64::try_from(report.modeled_fills_settled).unwrap_or(u64::MAX)
        }),
        paper_ledger_audit_records_appended: paper_ledger_report.map_or(0, |report| {
            u64::try_from(report.audit_records_appended).unwrap_or(u64::MAX)
        }),
        paper_ledger_replay_validated: paper_artifacts.ledger_replay_validated,
        paper_ledger_external_submission_performed: paper_artifacts.external_submission_performed,
        paper_ledger_live_execution_performed: paper_artifacts.live_execution_performed,
        failure_capture_validated: failure_capture_record.access_authorized,
        failure_capture_checkpoint_recovered: recovered_checkpoints.failure_capture,
        restart_adapter_recovery_plan_checkpoint_recovered: restart_report
            .adapter_recovery_plan_checkpoint_recovered
            && recovered_checkpoints.adapter_recovery_plan,
        failure_capture_metrics_endpoint_started: failure_capture_record.metrics_endpoint_started,
        failure_capture_public_network_exposed: failure_capture_record.public_network_exposed,
        failure_capture_outbound_alerts_sent: failure_capture_record.outbound_alerts_sent,
        failure_capture_external_submission_performed: failure_capture_record
            .external_submission_performed,
        failure_capture_live_execution_performed: failure_capture_record.live_execution_performed,
        restart_audit_records_replayed: restart_report.audit_records_replayed,
        backup_audit_records_replayed: backup_report.audit_records_replayed,
        restart_plan_checkpoint_recovered: restart_report.plan_checkpoint_recovered,
        restart_adapter_checkpoint_recovered: restart_report.adapter_checkpoint_recovered,
        restart_graceful_shutdown_checkpoint_recovered: restart_report
            .graceful_shutdown_checkpoint_recovered,
        restart_opportunity_trace_recovery_validated: restart_report
            .opportunity_trace_recovery_validated,
        restart_opportunity_trace_discovered_candidates: restart_report
            .opportunity_trace_discovered_candidates,
        restart_opportunity_trace_recovered_checkpoints: restart_report
            .opportunity_trace_recovered_checkpoints,
        restart_opportunity_trace_recovered_summaries: restart_report
            .opportunity_trace_recovered_summaries
            .clone(),
        restart_opportunity_trace_missing_checkpoints: restart_report
            .opportunity_trace_missing_checkpoints,
        opportunity_trace_recovery: restart_report.opportunity_trace_recovery.clone(),
        recovery_disposition: restart_report.recovery_disposition,
        service_manager_action_performed: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        unresolved_blockers: runtime_deployment_smoke_unresolved_blockers(),
    }
}

fn validate_local_runtime_concurrent_lifecycle_access(
    audit_path: &Path,
    state_path: &Path,
    policy: &PolicyEngine,
    base_request: &RuntimeLifecycleRequest,
    started_at_unix_ms: u64,
) -> Result<RuntimeConcurrentLifecycleValidationReport, RuntimeLifecycleError> {
    let workers = 3_usize;
    let barrier = Arc::new(Barrier::new(workers));
    let open_lock = Arc::new(Mutex::new(()));
    let handles = (0..workers)
        .map(|worker| {
            let audit_path = audit_path.to_path_buf();
            let state_path = state_path.to_path_buf();
            let policy = policy.clone();
            let mut request = base_request.clone();
            let barrier = Arc::clone(&barrier);
            let open_lock = Arc::clone(&open_lock);
            thread::spawn(move || -> Result<RuntimeLifecycleRecord, String> {
                let worker_index = u64::try_from(worker).map_err(|error| error.to_string())?;
                request.id = format!("{}-concurrent-{worker}", request.id);
                request.adapter_request_id =
                    format!("{}-concurrent-{worker}", request.adapter_request_id);
                request.plan.id = format!("{}-concurrent-{worker}", request.plan.id);
                request.now_unix_ms = started_at_unix_ms.saturating_add(worker_index);

                let (mut journal, mut store) = {
                    let _guard = open_lock
                        .lock()
                        .map_err(|_| "runtime concurrent open lock poisoned".to_owned())?;
                    (
                        AppendOnlyAuditJournal::open(&audit_path)
                            .map_err(|error| error.to_string())?,
                        SqliteWalStateStore::open(&state_path)
                            .map_err(|error| error.to_string())?,
                    )
                };
                barrier.wait();
                run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
                    .map_err(|error| error.to_string())
            })
        })
        .collect::<Vec<_>>();

    let mut workers_completed = 0_u64;
    let mut external_submission_performed = false;
    let mut live_execution_performed = false;
    for handle in handles {
        let worker_result = handle
            .join()
            .map_err(|_| RuntimeLifecycleError::ValidationFailed {
                reason: "runtime concurrent lifecycle worker panicked".to_owned(),
            })?;
        let record = worker_result.map_err(|reason| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime concurrent lifecycle worker failed: {reason}"),
        })?;
        workers_completed = workers_completed.saturating_add(1);
        external_submission_performed |= record.external_submission_performed;
        live_execution_performed |= record.live_execution_performed;
    }

    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)?;
    let audit_records_replayed = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_store = SqliteWalStateStore::open(state_path)?;
    let plan_checkpoint_recovered = reopened_store
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let adapter_checkpoint_recovered = reopened_store
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();
    let adapter_recovery_plan_checkpoint_recovered = reopened_store
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)?
        .is_some();
    reopened_store.integrity_check()?;
    let sqlite_integrity_check_passed = true;

    let report = RuntimeConcurrentLifecycleValidationReport {
        workers_completed,
        audit_records_replayed,
        plan_checkpoint_recovered,
        adapter_checkpoint_recovered,
        adapter_recovery_plan_checkpoint_recovered,
        sqlite_integrity_check_passed,
        external_submission_performed,
        live_execution_performed,
    };
    if !runtime_smoke_concurrent_lifecycle_validated(&report) {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime concurrent lifecycle validation failed".to_owned(),
        });
    }
    Ok(report)
}

fn runtime_smoke_restart_recovery_validated(
    report: &RuntimeRestartRecoveryValidationReport,
) -> bool {
    report.audit_replay_check_passed
        && report.sqlite_reopen_check_passed
        && report.plan_checkpoint_recovered
        && report.adapter_checkpoint_recovered
        && report.adapter_recovery_plan_checkpoint_recovered
        && report.graceful_shutdown_checkpoint_recovered
        && report.opportunity_trace_recovery_validated
}

fn runtime_smoke_concurrent_lifecycle_validated(
    report: &RuntimeConcurrentLifecycleValidationReport,
) -> bool {
    report.workers_completed > 0
        && report.audit_records_replayed >= report.workers_completed.saturating_mul(3)
        && report.plan_checkpoint_recovered
        && report.adapter_checkpoint_recovered
        && report.adapter_recovery_plan_checkpoint_recovered
        && report.sqlite_integrity_check_passed
        && !report.external_submission_performed
        && !report.live_execution_performed
}

fn runtime_smoke_audit_durability_validated(
    report: &crate::AuditDurabilityValidationReport,
) -> bool {
    report.append_replay_validated
        && report.truncated_replay_rejected
        && report.tamper_replay_rejected
        && report.concurrent_append_validated
        && report.filesystem_failure_validated
        && report.disk_full_failure_validated
        && !report.live_network_used
        && !report.external_execution_performed
        && !report.production_ready
}

fn runtime_deployment_smoke_unresolved_blockers() -> Vec<String> {
    vec![
        "local runtime smoke does not install or supervise a production service-manager".to_owned(),
        "local runtime smoke does not validate production host, container, filesystem, or scheduler behavior".to_owned(),
        "local runtime smoke does not enable live execution, external submissions, signing, broadcasts, withdrawals, bridges, wallet custody, real RPC calls, or real exchange calls".to_owned(),
    ]
}

fn runtime_deployment_smoke_load_unresolved_blockers() -> Vec<String> {
    vec![
        "local smoke load validation does not prove deployment-host load behavior".to_owned(),
        "local smoke load validation does not start or supervise a production service-manager"
            .to_owned(),
        "local smoke load validation does not exercise live exchange, RPC, signer, exporter, dashboard, or alert integrations"
            .to_owned(),
        "local smoke load validation does not approve production readiness".to_owned(),
    ]
}

fn runtime_load_profile_blockers(
    latency_budget_met: bool,
    resource_budget_met: bool,
    replay_recovery_evidence_validated: bool,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !latency_budget_met {
        blockers.push("local-latency-budget-exceeded".to_owned());
    }
    if !resource_budget_met {
        blockers.push("local-resource-budget-exceeded".to_owned());
    }
    if !replay_recovery_evidence_validated {
        blockers.push("local-replay-recovery-evidence-incomplete".to_owned());
    }
    blockers
}

fn runtime_load_profile_remaining_external_evidence() -> Vec<String> {
    vec![
        "deployment-host load and soak evidence is missing".to_owned(),
        "live/provider feed backpressure evidence is missing".to_owned(),
        "target-class or ARM runtime performance evidence is missing".to_owned(),
        "dashboard and observability exporter latency evidence is missing".to_owned(),
        "production resource profiler evidence is missing".to_owned(),
    ]
}

fn runtime_production_preflight_unresolved_blockers(
    request: &RuntimeProductionPreflightRequest,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !request.service_manager_lifecycle_evidence_available {
        blockers.push(
            "service-manager-controlled deployment-host lifecycle evidence is missing".to_owned(),
        );
    }
    if !request.deployment_host_permission_evidence_available {
        blockers
            .push("deployment-host runtime filesystem permission evidence is missing".to_owned());
    }
    if !request.physical_disk_full_evidence_available {
        blockers.push("physical deployment-host disk-full evidence is missing".to_owned());
    }
    if !request.retention_execution_evidence_available {
        blockers.push(
            "deployment-host audit retention/rotation execution evidence is missing".to_owned(),
        );
    }
    if !request.rollback_drill_evidence_available {
        blockers.push("executed rollback-drill evidence is missing".to_owned());
    }
    if !request.incident_response_evidence_available {
        blockers.push("executed incident-response drill evidence is missing".to_owned());
    }
    if !request.observability_runtime_evidence_available {
        blockers.push("real observability/exporter/alert runtime evidence is missing".to_owned());
    }
    if blockers.is_empty() {
        blockers.push(
            "external production-readiness review remains required before any readiness claim"
                .to_owned(),
        );
    }
    blockers
}

#[derive(Debug, Clone, Copy)]
struct ServiceManagerLifecycleEvidence {
    start_evidence_present: bool,
    runtime_smoke_evidence_present: bool,
    graceful_shutdown_evidence_present: bool,
    stop_evidence_present: bool,
    restart_evidence_present: bool,
    recovery_evidence_present: bool,
    operator_controlled_events: bool,
    non_secret_references_present: bool,
    successful_event_outcomes: bool,
    concurrent_lifecycle_evidence_present: bool,
}

#[derive(Debug, Clone, Copy)]
struct ServiceManagerLifecycleRehearsalEvidence {
    ordered_lifecycle_validated: bool,
    operator_controlled_events: bool,
    non_secret_references_present: bool,
    successful_event_outcomes: bool,
    start_evidence_present: bool,
    runtime_smoke_evidence_present: bool,
    graceful_shutdown_evidence_present: bool,
    stop_evidence_present: bool,
    restart_evidence_present: bool,
    recovery_evidence_present: bool,
}

impl RuntimeServiceManagerLifecycleTranscript {
    fn has_successful_reference(&self, kind: RuntimeServiceManagerLifecycleEventKind) -> bool {
        self.events.iter().any(|event| {
            event.kind == kind
                && event.operator_controlled
                && event.non_secret_reference_present
                && event.outcome_success
        })
    }
}

fn rehearsal_has_successful_reference(
    request: &RuntimeServiceManagerLifecycleRehearsalRequest,
    kind: RuntimeServiceManagerLifecycleEventKind,
) -> bool {
    request.events.iter().any(|event| {
        event.kind == kind
            && event.operator_controlled
            && event.non_secret_reference_present
            && event.outcome_success
    })
}

fn service_manager_lifecycle_order_validated(
    events: &[RuntimeServiceManagerLifecycleEvent],
) -> bool {
    let required_order = [
        RuntimeServiceManagerLifecycleEventKind::UnitLoaded,
        RuntimeServiceManagerLifecycleEventKind::Started,
        RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed,
        RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
        RuntimeServiceManagerLifecycleEventKind::Stopped,
        RuntimeServiceManagerLifecycleEventKind::Restarted,
        RuntimeServiceManagerLifecycleEventKind::RecoveryValidated,
    ];
    let mut cursor = 0_usize;
    let mut last_observed_at = 0_u64;
    for event in events {
        if event.observed_at_unix_ms < last_observed_at {
            return false;
        }
        last_observed_at = event.observed_at_unix_ms;
        if cursor < required_order.len() && event.kind == required_order[cursor] {
            cursor = cursor.saturating_add(1);
        }
    }
    cursor == required_order.len()
}

fn service_manager_lifecycle_rehearsal_blockers(
    request: &RuntimeServiceManagerLifecycleRehearsalRequest,
    evidence: ServiceManagerLifecycleRehearsalEvidence,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !evidence.ordered_lifecycle_validated {
        blockers.push("missing-ordered-lifecycle-evidence".to_owned());
    }
    if !evidence.operator_controlled_events {
        blockers.push("non-operator-controlled-event".to_owned());
    }
    if !evidence.non_secret_references_present {
        blockers.push("missing-non-secret-event-reference".to_owned());
    }
    if !evidence.successful_event_outcomes {
        blockers.push("failed-lifecycle-event".to_owned());
    }
    if !evidence.start_evidence_present {
        blockers.push("missing-start-evidence".to_owned());
    }
    if !evidence.runtime_smoke_evidence_present {
        blockers.push("missing-runtime-smoke-evidence".to_owned());
    }
    if !evidence.graceful_shutdown_evidence_present {
        blockers.push("missing-graceful-shutdown-evidence".to_owned());
    }
    if !evidence.stop_evidence_present {
        blockers.push("missing-stop-evidence".to_owned());
    }
    if !evidence.restart_evidence_present {
        blockers.push("missing-restart-evidence".to_owned());
    }
    if !evidence.recovery_evidence_present {
        blockers.push("missing-recovery-evidence".to_owned());
    }
    if !request.audit_replay_reference_present {
        blockers.push("missing-audit-replay-reference".to_owned());
    }
    if !request.sqlite_recovery_reference_present {
        blockers.push("missing-sqlite-recovery-reference".to_owned());
    }
    if !request.concurrent_lifecycle_reference_present
        || request.concurrent_lifecycle_worker_count < 2
        || !request.concurrent_lifecycle_success
    {
        blockers.push("missing-concurrent-lifecycle-evidence".to_owned());
    }
    if !request.graceful_shutdown_checkpoint_reference_present {
        blockers.push("missing-graceful-shutdown-checkpoint-reference".to_owned());
    }
    if !request.restart_recovery_reference_present {
        blockers.push("missing-restart-recovery-reference".to_owned());
    }
    if !request.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !request.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

fn service_manager_lifecycle_blockers(
    transcript: &RuntimeServiceManagerLifecycleTranscript,
    evidence: ServiceManagerLifecycleEvidence,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !evidence.start_evidence_present {
        blockers.push("missing-start-evidence".to_owned());
    }
    if !evidence.runtime_smoke_evidence_present {
        blockers.push("missing-runtime-smoke-evidence".to_owned());
    }
    if !evidence.graceful_shutdown_evidence_present {
        blockers.push("missing-graceful-shutdown-evidence".to_owned());
    }
    if !evidence.stop_evidence_present {
        blockers.push("missing-stop-evidence".to_owned());
    }
    if !evidence.restart_evidence_present {
        blockers.push("missing-restart-evidence".to_owned());
    }
    if !evidence.recovery_evidence_present {
        blockers.push("missing-recovery-evidence".to_owned());
    }
    if !evidence.operator_controlled_events {
        blockers.push("non-operator-controlled-event".to_owned());
    }
    if !evidence.non_secret_references_present {
        blockers.push("missing-non-secret-event-reference".to_owned());
    }
    if !evidence.successful_event_outcomes {
        blockers.push("failed-lifecycle-event".to_owned());
    }
    if !evidence.concurrent_lifecycle_evidence_present {
        blockers.push("missing-concurrent-lifecycle-evidence".to_owned());
    }
    if !transcript.audit_replay_reference_present {
        blockers.push("missing-audit-replay-reference".to_owned());
    }
    if !transcript.sqlite_recovery_reference_present {
        blockers.push("missing-sqlite-recovery-reference".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.operator_lifecycle_rehearsal_reference_present {
        blockers.push("missing-operator-lifecycle-rehearsal-reference".to_owned());
    }
    if !transcript.emergency_stop_review_reference_present {
        blockers.push("missing-emergency-stop-review-reference".to_owned());
    }
    if !transcript.rollback_plan_review_reference_present {
        blockers.push("missing-rollback-plan-review-reference".to_owned());
    }
    if !transcript.operator_review_window_current {
        blockers.push("operator-review-window-not-current".to_owned());
    }
    blockers
}

fn deployment_permission_blockers(
    transcript: &RuntimeDeploymentPermissionTranscript,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.deployment_host_evidence {
        blockers.push("missing-deployment-host-evidence".to_owned());
    }
    if !transcript.runtime_write_attempt_reference_present {
        blockers.push("missing-runtime-write-attempt-reference".to_owned());
    }
    if !transcript.runtime_write_permission_denied {
        blockers.push("missing-runtime-write-permission-denial-evidence".to_owned());
    }
    if !transcript.runtime_write_error_classified {
        blockers.push("missing-runtime-write-error-classification".to_owned());
    }
    if !transcript.audit_write_failed_closed {
        blockers.push("missing-audit-write-fail-closed-evidence".to_owned());
    }
    if !transcript.state_write_failed_closed {
        blockers.push("missing-state-write-fail-closed-evidence".to_owned());
    }
    if !transcript.adapter_evaluation_blocked {
        blockers.push("missing-adapter-evaluation-blocked-evidence".to_owned());
    }
    if !transcript.runtime_quiesced_or_degraded {
        blockers.push("missing-runtime-quiesce-or-degrade-evidence".to_owned());
    }
    if !transcript.audit_replay_after_restore_validated {
        blockers.push("missing-audit-replay-after-restore-evidence".to_owned());
    }
    if !transcript.sqlite_reopen_after_restore_validated {
        blockers.push("missing-sqlite-reopen-after-restore-evidence".to_owned());
    }
    if !transcript.recovery_runbook_reference_present {
        blockers.push("missing-recovery-runbook-reference".to_owned());
    }
    if transcript.non_secret_reference_count < 7 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    blockers
}

fn deployment_audit_sqlite_blockers(
    transcript: &RuntimeDeploymentAuditSqliteTranscript,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.deployment_host_evidence {
        blockers.push("missing-deployment-host-evidence".to_owned());
    }
    if !transcript.service_lifecycle_reference_present {
        blockers.push("missing-service-lifecycle-reference".to_owned());
    }
    if !transcript.audit_append_reference_present {
        blockers.push("missing-audit-append-reference".to_owned());
    }
    if !transcript.audit_replay_validated {
        blockers.push("missing-audit-replay-evidence".to_owned());
    }
    if !transcript.audit_hash_chain_validated {
        blockers.push("missing-audit-hash-chain-evidence".to_owned());
    }
    if !transcript.sqlite_wal_mode_validated {
        blockers.push("missing-sqlite-wal-mode-evidence".to_owned());
    }
    if !transcript.sqlite_integrity_check_passed {
        blockers.push("missing-sqlite-integrity-check-evidence".to_owned());
    }
    if !transcript.sqlite_checkpoint_recovered {
        blockers.push("missing-sqlite-checkpoint-recovery-evidence".to_owned());
    }
    if !transcript.backup_restore_validated {
        blockers.push("missing-backup-restore-evidence".to_owned());
    }
    if !transcript.concurrent_access_validated {
        blockers.push("missing-concurrent-access-evidence".to_owned());
    }
    if !transcript.recovery_runbook_reference_present {
        blockers.push("missing-recovery-runbook-reference".to_owned());
    }
    if transcript.non_secret_reference_count < 7 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

fn deployment_backup_restore_blockers(
    transcript: &RuntimeDeploymentBackupRestoreTranscript,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.deployment_host_evidence {
        blockers.push("missing-deployment-host-evidence".to_owned());
    }
    if !transcript.service_lifecycle_reference_present {
        blockers.push("missing-service-lifecycle-reference".to_owned());
    }
    if !transcript.backup_artifact_reference_present {
        blockers.push("missing-backup-artifact-reference".to_owned());
    }
    if !transcript.restore_execution_reference_present {
        blockers.push("missing-restore-execution-reference".to_owned());
    }
    if !transcript.deployment_load_reference_present {
        blockers.push("missing-deployment-load-reference".to_owned());
    }
    if !transcript.audit_replay_after_restore_validated {
        blockers.push("missing-audit-replay-after-restore-evidence".to_owned());
    }
    if !transcript.audit_hash_chain_after_restore_validated {
        blockers.push("missing-audit-hash-chain-after-restore-evidence".to_owned());
    }
    if !transcript.sqlite_integrity_after_restore_validated {
        blockers.push("missing-sqlite-integrity-after-restore-evidence".to_owned());
    }
    if !transcript.sqlite_checkpoint_after_restore_validated {
        blockers.push("missing-sqlite-checkpoint-after-restore-evidence".to_owned());
    }
    if !transcript.runtime_checkpoint_restore_validated {
        blockers.push("missing-runtime-checkpoint-restore-evidence".to_owned());
    }
    if !transcript.post_restore_runtime_smoke_passed {
        blockers.push("missing-post-restore-runtime-smoke-evidence".to_owned());
    }
    if !transcript.rollback_reference_present {
        blockers.push("missing-rollback-reference".to_owned());
    }
    if !transcript.recovery_runbook_reference_present {
        blockers.push("missing-recovery-runbook-reference".to_owned());
    }
    if transcript.non_secret_reference_count < 9 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

fn deployment_graceful_shutdown_blockers(
    transcript: &RuntimeDeploymentGracefulShutdownTranscript,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.deployment_host_evidence {
        blockers.push("missing-deployment-host-evidence".to_owned());
    }
    if !transcript.service_lifecycle_reference_present {
        blockers.push("missing-service-lifecycle-reference".to_owned());
    }
    if !transcript.shutdown_request_reference_present {
        blockers.push("missing-shutdown-request-reference".to_owned());
    }
    if !transcript.service_stopped_reference_present {
        blockers.push("missing-service-stopped-reference".to_owned());
    }
    if !transcript.graceful_shutdown_checkpoint_reference_present {
        blockers.push("missing-graceful-shutdown-checkpoint-reference".to_owned());
    }
    if !transcript.audit_replay_after_shutdown_validated {
        blockers.push("missing-audit-replay-after-shutdown-evidence".to_owned());
    }
    if !transcript.sqlite_reopen_after_shutdown_validated {
        blockers.push("missing-sqlite-reopen-after-shutdown-evidence".to_owned());
    }
    if !transcript.restart_recovery_after_shutdown_validated {
        blockers.push("missing-restart-recovery-after-shutdown-evidence".to_owned());
    }
    if !transcript.post_shutdown_runtime_smoke_passed {
        blockers.push("missing-post-shutdown-runtime-smoke-evidence".to_owned());
    }
    if transcript.non_secret_reference_count < 8 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

fn deployment_sqlite_schema_migration_blockers(
    transcript: &RuntimeDeploymentSqliteSchemaMigrationTranscript,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.deployment_host_evidence {
        blockers.push("missing-deployment-host-evidence".to_owned());
    }
    if !transcript.service_lifecycle_reference_present {
        blockers.push("missing-service-lifecycle-reference".to_owned());
    }
    if !transcript.pre_migration_backup_reference_present {
        blockers.push("missing-pre-migration-backup-reference".to_owned());
    }
    if !transcript.migration_execution_reference_present {
        blockers.push("missing-migration-execution-reference".to_owned());
    }
    if !transcript.schema_version_transition_validated {
        blockers.push("missing-schema-version-transition-evidence".to_owned());
    }
    if transcript.post_migration_schema_version != transcript.expected_schema_version {
        blockers.push("schema-version-mismatch".to_owned());
    }
    if transcript.pre_migration_schema_version > transcript.post_migration_schema_version {
        blockers.push("schema-version-regressed".to_owned());
    }
    if !transcript.sqlite_integrity_check_passed {
        blockers.push("missing-sqlite-integrity-check-evidence".to_owned());
    }
    if !transcript.sqlite_checkpoint_reopened {
        blockers.push("missing-sqlite-checkpoint-reopen-evidence".to_owned());
    }
    if !transcript.audit_replay_after_migration_validated {
        blockers.push("missing-audit-replay-after-migration-evidence".to_owned());
    }
    if !transcript.rollback_reference_present {
        blockers.push("missing-rollback-reference".to_owned());
    }
    if !transcript.runtime_quiesced_or_degraded {
        blockers.push("missing-runtime-quiesce-or-degrade-evidence".to_owned());
    }
    if transcript.non_secret_reference_count < 8 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

fn validate_service_manager_unit_name(unit_name: &str) -> Result<(), RuntimeLifecycleError> {
    let trimmed = unit_name.trim();
    if trimmed.is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "service-manager unit name is required".to_owned(),
        });
    }
    if trimmed.len() > 96
        || !trimmed.ends_with(".service")
        || trimmed.contains(['/', '\\', ':'])
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '@' | '-'))
    {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "service-manager unit name must be a simple .service name".to_owned(),
        });
    }
    let lowered = trimmed.to_ascii_lowercase();
    if ["withdraw", "bridge", "sign", "broadcast", "live"]
        .iter()
        .any(|token| lowered.contains(token))
    {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "service-manager unit name must not imply live funds or signing".to_owned(),
        });
    }
    Ok(())
}

fn validate_runtime_deployment_smoke_paths(
    audit_path: &Path,
    state_path: &Path,
    backup_audit_path: &Path,
    backup_state_path: &Path,
    audit_validation_workspace: &Path,
) -> Result<(), RuntimeLifecycleError> {
    validate_runtime_smoke_target(audit_path, "audit")?;
    validate_runtime_smoke_target(state_path, "state")?;
    validate_runtime_backup_target(audit_path, backup_audit_path, "audit")?;
    validate_runtime_backup_target(state_path, backup_state_path, "state")?;
    if audit_validation_workspace.exists() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime deployment smoke audit validation workspace must not already exist"
                .to_owned(),
        });
    }
    Ok(())
}

fn record_local_runtime_smoke_observability(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    graceful_shutdown_record: &RuntimeGracefulShutdownRecord,
    communications_dispatch: &NotificationDispatchRecord,
    collected_at_ms: u64,
) -> Result<RuntimeSmokeObservabilityArtifacts, RuntimeLifecycleError> {
    let record = record_runtime_smoke_observability_collection(
        journal,
        store,
        lifecycle_record,
        graceful_shutdown_record,
        collected_at_ms,
    )?;
    let operations_review =
        record_runtime_smoke_observability_operations(journal, store, collected_at_ms)?;
    let export_dry_run = record_runtime_smoke_observability_export(
        journal,
        store,
        &record,
        &operations_review,
        collected_at_ms,
    )?;
    let alert_route_dispatch = record_runtime_smoke_observability_alert_route(
        journal,
        store,
        &export_dry_run,
        communications_dispatch,
        collected_at_ms,
    )?;
    let endpoint_preflight =
        record_runtime_smoke_observability_endpoint(journal, store, collected_at_ms)?;
    let loopback_bind =
        record_runtime_smoke_observability_loopback(journal, store, collected_at_ms)?;
    let metrics_scrape = record_runtime_smoke_observability_metrics_scrape(
        journal,
        store,
        &export_dry_run,
        collected_at_ms,
    )?;
    let metrics_endpoint = record_runtime_smoke_observability_metrics_endpoint(
        journal,
        store,
        &export_dry_run,
        collected_at_ms,
    )?;
    let tracing = record_runtime_smoke_observability_tracing(journal, store, collected_at_ms)?;

    Ok(RuntimeSmokeObservabilityArtifacts {
        record,
        operations_review,
        export_dry_run,
        alert_route_dispatch,
        endpoint_preflight,
        loopback_bind,
        metrics_scrape,
        metrics_endpoint,
        tracing,
    })
}

fn record_runtime_smoke_observability_collection(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    graceful_shutdown_record: &RuntimeGracefulShutdownRecord,
    collected_at_ms: u64,
) -> Result<ObservabilityRecord, RuntimeLifecycleError> {
    let record = collect_local_runtime_smoke_observability(
        lifecycle_record,
        graceful_shutdown_record,
        collected_at_ms,
    )?;
    append_observability_record_audit(journal, &record, collected_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability audit failed: {error}"),
        }
    })?;
    persist_observability_record_checkpoint(store, &record, collected_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability checkpoint failed: {error}"),
        }
    })?;
    Ok(record)
}

fn record_runtime_smoke_observability_operations(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    collected_at_ms: u64,
) -> Result<ObservabilityOperationsReviewReport, RuntimeLifecycleError> {
    let operations_review = review_observability_operations(&ObservabilityOperationsPolicy {
        review_id: "runtime-smoke-observability-operations".to_owned(),
        log_retention_required: true,
        retention_days: 30,
        redaction_required: true,
        alert_routing_required: true,
        alert_route_count: 1,
        incident_runbook_required: true,
        incident_runbook_count: 1,
        loopback_or_authenticated_endpoint_required: true,
        audit_state_preflight_required: true,
        exporter_kill_switch_required: true,
        alert_authorization_required: true,
        rate_limit_backpressure_required: true,
        retry_backoff_required: true,
        no_secret_telemetry_required: true,
        metrics_endpoint_requested: false,
        outbound_alert_delivery_requested: false,
        telemetry_export_requested: false,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability operations review failed: {error}"),
    })?;
    append_observability_operations_review_audit(
        journal,
        &operations_review,
        collected_at_ms.saturating_add(1),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability operations audit failed: {error}"),
    })?;
    persist_observability_operations_review_checkpoint(
        store,
        &operations_review,
        collected_at_ms.saturating_add(2),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability operations checkpoint failed: {error}"),
    })?;
    Ok(operations_review)
}

fn record_runtime_smoke_observability_export(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    record: &ObservabilityRecord,
    operations_review: &ObservabilityOperationsReviewReport,
    collected_at_ms: u64,
) -> Result<ObservabilityExportDryRunReport, RuntimeLifecycleError> {
    let export_dry_run = render_observability_export_dry_run(ObservabilityExportDryRunRequest {
        record: record.clone(),
        operations_review: operations_review.clone(),
        alert_route_references: vec!["runtime-smoke-alert-route".to_owned()],
        rendered_at_ms: collected_at_ms.saturating_add(3),
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability export dry-run failed: {error}"),
    })?;
    append_observability_export_dry_run_audit(
        journal,
        &export_dry_run,
        collected_at_ms.saturating_add(4),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability export audit failed: {error}"),
    })?;
    persist_observability_export_dry_run_checkpoint(
        store,
        &export_dry_run,
        collected_at_ms.saturating_add(5),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability export checkpoint failed: {error}"),
    })?;
    Ok(export_dry_run)
}

fn record_runtime_smoke_observability_alert_route(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    export_dry_run: &ObservabilityExportDryRunReport,
    communications_dispatch: &NotificationDispatchRecord,
    collected_at_ms: u64,
) -> Result<ObservabilityAlertRouteDispatchReport, RuntimeLifecycleError> {
    let alert_route_dispatch =
        record_observability_alert_route_dispatch(ObservabilityAlertRouteDispatchRequest {
            dispatch_review_id: "runtime-smoke-observability-alert-route".to_owned(),
            export_report: export_dry_run.clone(),
            alert_route_reference: "runtime-smoke-alert-route".to_owned(),
            notification_dispatch: communications_dispatch.clone(),
            local_dispatch_required: true,
            outbound_alert_delivery_requested: false,
            reviewed_at_ms: collected_at_ms.saturating_add(6),
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability alert-route dispatch failed: {error}"),
        })?;
    append_observability_alert_route_dispatch_audit(
        journal,
        &alert_route_dispatch,
        collected_at_ms.saturating_add(7),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability alert-route audit failed: {error}"),
    })?;
    persist_observability_alert_route_dispatch_checkpoint(
        store,
        &alert_route_dispatch,
        collected_at_ms.saturating_add(8),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability alert-route checkpoint failed: {error}"),
    })?;
    Ok(alert_route_dispatch)
}

fn record_runtime_smoke_observability_endpoint(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    collected_at_ms: u64,
) -> Result<ObservabilityEndpointPreflightReport, RuntimeLifecycleError> {
    let endpoint_preflight = preflight_observability_endpoint(&ObservabilityEndpointPreflight {
        preflight_id: "runtime-smoke-observability-endpoint".to_owned(),
        bind_host: "127.0.0.1".to_owned(),
        bind_port: 9_090,
        loopback_only_required: true,
        authentication_required: true,
        authorization_required: true,
        transport_protection_required: true,
        redaction_required: true,
        alert_routes_configured: true,
        alert_route_count: 1,
        exporter_backpressure_required: true,
        metrics_endpoint_start_requested: false,
        public_network_exposure_requested: false,
        telemetry_export_requested: false,
        outbound_alert_delivery_requested: false,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability endpoint preflight failed: {error}"),
    })?;
    append_observability_endpoint_preflight_audit(
        journal,
        &endpoint_preflight,
        collected_at_ms.saturating_add(9),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability endpoint preflight audit failed: {error}"),
    })?;
    persist_observability_endpoint_preflight_checkpoint(
        store,
        &endpoint_preflight,
        collected_at_ms.saturating_add(10),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!(
            "runtime smoke observability endpoint preflight checkpoint failed: {error}"
        ),
    })?;
    Ok(endpoint_preflight)
}

fn record_runtime_smoke_observability_loopback(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    collected_at_ms: u64,
) -> Result<ObservabilityLoopbackBindValidationReport, RuntimeLifecycleError> {
    let loopback_bind =
        validate_observability_loopback_bind(&ObservabilityLoopbackBindValidationRequest {
            validation_id: "runtime-smoke-observability-loopback-bind".to_owned(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            loopback_only_required: true,
            serve_requests_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability loopback bind failed: {error}"),
        })?;
    append_observability_loopback_bind_validation_audit(
        journal,
        &loopback_bind,
        collected_at_ms.saturating_add(11),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability loopback-bind audit failed: {error}"),
    })?;
    persist_observability_loopback_bind_validation_checkpoint(
        store,
        &loopback_bind,
        collected_at_ms.saturating_add(12),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability loopback-bind checkpoint failed: {error}"),
    })?;
    Ok(loopback_bind)
}

fn record_runtime_smoke_observability_metrics_scrape(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    export_dry_run: &ObservabilityExportDryRunReport,
    collected_at_ms: u64,
) -> Result<ObservabilityMetricsScrapePreflightReport, RuntimeLifecycleError> {
    let metrics_scrape =
        preflight_observability_metrics_scrape(ObservabilityMetricsScrapePreflightRequest {
            scrape_id: "runtime-smoke-observability-metrics-scrape".to_owned(),
            export_report: export_dry_run.clone(),
            request_method: "GET".to_owned(),
            request_path: "/metrics".to_owned(),
            source_host: "127.0.0.1".to_owned(),
            authentication_required: true,
            authorization_required: true,
            bearer_token_reference_present: true,
            metrics_endpoint_start_requested: false,
            public_network_exposure_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability metrics scrape failed: {error}"),
        })?;
    append_observability_metrics_scrape_preflight_audit(
        journal,
        &metrics_scrape,
        collected_at_ms.saturating_add(13),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability metrics scrape audit failed: {error}"),
    })?;
    persist_observability_metrics_scrape_preflight_checkpoint(
        store,
        &metrics_scrape,
        collected_at_ms.saturating_add(14),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability metrics scrape checkpoint failed: {error}"),
    })?;
    Ok(metrics_scrape)
}

fn record_runtime_smoke_observability_metrics_endpoint(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    export_dry_run: &ObservabilityExportDryRunReport,
    collected_at_ms: u64,
) -> Result<ObservabilityMetricsEndpointValidationReport, RuntimeLifecycleError> {
    let metrics_endpoint =
        validate_observability_metrics_endpoint(ObservabilityMetricsEndpointValidationRequest {
            validation_id: "runtime-smoke-observability-metrics-endpoint".to_owned(),
            export_report: export_dry_run.clone(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            request_method: "GET".to_owned(),
            request_path: "/metrics".to_owned(),
            loopback_only_required: true,
            authentication_required: true,
            authorization_required: true,
            bearer_token_reference_present: true,
            public_network_exposure_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability metrics endpoint failed: {error}"),
        })?;
    append_observability_metrics_endpoint_validation_audit(
        journal,
        &metrics_endpoint,
        collected_at_ms.saturating_add(15),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability metrics endpoint audit failed: {error}"),
    })?;
    persist_observability_metrics_endpoint_validation_checkpoint(
        store,
        &metrics_endpoint,
        collected_at_ms.saturating_add(16),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability metrics endpoint checkpoint failed: {error}"),
    })?;
    Ok(metrics_endpoint)
}

fn record_runtime_smoke_observability_tracing(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    collected_at_ms: u64,
) -> Result<LocalTracingSubscriberValidationReport, RuntimeLifecycleError> {
    let tracing = validate_local_tracing_subscriber(LocalTracingSubscriberValidationRequest {
        validation_id: "runtime-smoke-observability-tracing".to_owned(),
        subscriber_label: "runtime-smoke-observability-subscriber".to_owned(),
        event: StructuredLogEvent::new(
            "runtime-smoke-observability-tracing-event",
            ObservabilitySeverity::Info,
            "runtime-smoke-observability",
            "local runtime smoke captured scoped tracing event",
            vec![StructuredLogField::new("scope", "runtime-smoke")],
            collected_at_ms.saturating_add(17),
        ),
        config: ObservabilityBoundaryConfig::default(),
        access: ObservabilityAccessContext::local_collection(Some(
            "runtime-smoke-observability".to_owned(),
        )),
        local_capture_required: true,
        redaction_required: true,
        global_install_requested: false,
        telemetry_export_requested: false,
        outbound_alert_delivery_requested: false,
        public_network_exposure_requested: false,
        live_execution_requested: false,
        captured_at_ms: collected_at_ms.saturating_add(18),
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability tracing failed: {error}"),
    })?;
    append_local_tracing_subscriber_audit(journal, &tracing, collected_at_ms.saturating_add(19))
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability tracing audit failed: {error}"),
        })?;
    persist_local_tracing_subscriber_checkpoint(
        store,
        &tracing,
        collected_at_ms.saturating_add(20),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke observability tracing checkpoint failed: {error}"),
    })?;
    Ok(tracing)
}

fn record_local_runtime_smoke_communications(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    recorded_at_ms: u64,
) -> Result<RuntimeSmokeCommunicationsArtifacts, RuntimeLifecycleError> {
    let config = runtime_smoke_communications_config();
    let route = record_local_runtime_smoke_command_route(
        journal,
        store,
        lifecycle_record,
        &config,
        recorded_at_ms,
    )?;
    let remote_review =
        record_local_runtime_smoke_remote_review(journal, store, lifecycle_record, recorded_at_ms)?;
    let platform_ingress = record_local_runtime_smoke_platform_ingress(
        journal,
        store,
        lifecycle_record,
        recorded_at_ms,
    )?;
    let remote_envelope = record_local_runtime_smoke_remote_envelope(
        journal,
        store,
        &remote_review,
        &platform_ingress,
        recorded_at_ms,
    )?;
    let dispatch = record_local_runtime_smoke_notification(
        journal,
        store,
        lifecycle_record,
        config,
        recorded_at_ms,
    )?;
    let channel_adapter = record_local_runtime_smoke_channel_adapter(
        journal,
        store,
        &remote_envelope,
        &dispatch,
        recorded_at_ms,
    )?;
    let channel_session = record_local_runtime_smoke_channel_session(
        journal,
        store,
        &remote_envelope,
        &dispatch,
        &channel_adapter,
        recorded_at_ms,
    )?;
    let platform_adapter = record_local_runtime_smoke_platform_adapter(
        journal,
        store,
        &remote_envelope,
        recorded_at_ms,
    )?;

    Ok(RuntimeSmokeCommunicationsArtifacts {
        route,
        remote_review,
        platform_ingress,
        remote_envelope,
        channel_adapter,
        channel_session,
        platform_adapter,
        dispatch,
    })
}

fn record_local_runtime_smoke_command_route(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    config: &CommunicationBoundaryConfig,
    recorded_at_ms: u64,
) -> Result<RoutedOperatorCommand, RuntimeLifecycleError> {
    let command = parse_cli_command(&["status".to_owned()], recorded_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke communications command parse failed: {error}"),
        }
    })?;
    let route = DeterministicOperatorCommandRouter::new()
        .route(&OperatorCommandRoutingRequest {
            id: format!("runtime-smoke-command-route-{}", lifecycle_record.id),
            command,
            config: config.clone(),
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke communications route failed: {error}"),
        })?;
    append_routed_operator_command_audit(journal, &route, recorded_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke communications route audit failed: {error}"),
        }
    })?;
    persist_routed_operator_command_checkpoint(store, &route, recorded_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke communications route checkpoint failed: {error}"),
        }
    })?;
    Ok(route)
}

fn record_local_runtime_smoke_remote_review(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    recorded_at_ms: u64,
) -> Result<RemoteCommandSecurityReviewReport, RuntimeLifecycleError> {
    let remote_review = review_remote_command_security(&RemoteCommandSecurityReviewRequest {
        review_id: format!("runtime-smoke-remote-review-{}", lifecycle_record.id),
        source: OperatorCommandSource::MessagingChannel,
        channel_authentication_required: true,
        platform_identity_verification_required: true,
        platform_identity_authorization_required: true,
        replay_protection_required: true,
        command_allowlist_required: true,
        unsafe_commands_blocked: true,
        remote_command_enablement_requested: false,
        outbound_network_requested: false,
        live_execution_requested: false,
        reviewed_at_unix_ms: recorded_at_ms.saturating_add(1),
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke remote command review failed: {error}"),
    })?;
    append_remote_command_security_review_audit(
        journal,
        &remote_review,
        recorded_at_ms.saturating_add(2),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke remote command review audit failed: {error}"),
    })?;
    persist_remote_command_security_review_checkpoint(
        store,
        &remote_review,
        recorded_at_ms.saturating_add(3),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke remote command review checkpoint failed: {error}"),
    })?;
    Ok(remote_review)
}

fn record_local_runtime_smoke_platform_ingress(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    recorded_at_ms: u64,
) -> Result<PlatformCommandIngressReport, RuntimeLifecycleError> {
    let platform_ingress = review_platform_command_ingress(&PlatformCommandIngressRequest {
        ingress_id: format!("runtime-smoke-platform-ingress-{}", lifecycle_record.id),
        platform: "local-mock-platform".to_owned(),
        channel: NotificationChannelProfile::from_identifier("cli"),
        platform_message_id: format!("runtime-smoke-message-{}", lifecycle_record.id),
        platform_identity: "runtime-smoke-operator".to_owned(),
        command_text: "status".to_owned(),
        token_reference_present: true,
        token_secret_material_present: false,
        platform_signature_verified: true,
        platform_identity_authorized: true,
        channel_permission_granted: true,
        replay_nonce: format!("runtime-smoke-nonce-{recorded_at_ms}"),
        replay_nonce_reused: false,
        provider_rate_limited: false,
        provider_outage_observed: false,
        received_at_unix_ms: recorded_at_ms.saturating_add(4),
        now_unix_ms: recorded_at_ms.saturating_add(5),
        max_age_ms: 60_000,
        outbound_network_used: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke platform command ingress failed: {error}"),
    })?;
    append_platform_command_ingress_audit(
        journal,
        &platform_ingress,
        recorded_at_ms.saturating_add(6),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke platform command ingress audit failed: {error}"),
    })?;
    persist_platform_command_ingress_checkpoint(
        store,
        &platform_ingress,
        recorded_at_ms.saturating_add(7),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke platform command ingress checkpoint failed: {error}"),
    })?;
    Ok(platform_ingress)
}

fn record_local_runtime_smoke_remote_envelope(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    remote_review: &RemoteCommandSecurityReviewReport,
    platform_ingress: &PlatformCommandIngressReport,
    recorded_at_ms: u64,
) -> Result<RemoteCommandEnvelopeValidationReport, RuntimeLifecycleError> {
    let remote_envelope =
        validate_remote_command_envelope(&RemoteCommandEnvelopeValidationRequest {
            envelope_id: format!(
                "runtime-smoke-remote-envelope-{}",
                platform_ingress.ingress_id
            ),
            command: platform_ingress.command.clone(),
            security_review: remote_review.clone(),
            platform_identity: platform_ingress.platform_identity.clone(),
            authorization_policy: "runtime-smoke-local-review-only".to_owned(),
            authentication_reference: "local-auth-reference".to_owned(),
            replay_nonce: format!("runtime-smoke-nonce-{recorded_at_ms}"),
            channel_authenticated: true,
            platform_identity_verified: true,
            platform_identity_authorized: true,
            replay_protection_checked: true,
            replay_nonce_reused: false,
            command_allowlisted: true,
            received_at_unix_ms: recorded_at_ms.saturating_add(4),
            now_unix_ms: recorded_at_ms.saturating_add(5),
            max_age_ms: 60_000,
            remote_command_enablement_requested: false,
            outbound_network_used: false,
            live_execution_performed: false,
            signing_or_broadcast_performed: false,
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke remote command envelope failed: {error}"),
        })?;
    append_remote_command_envelope_validation_audit(
        journal,
        &remote_envelope,
        recorded_at_ms.saturating_add(6),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke remote command envelope audit failed: {error}"),
    })?;
    persist_remote_command_envelope_validation_checkpoint(
        store,
        &remote_envelope,
        recorded_at_ms.saturating_add(7),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke remote command envelope checkpoint failed: {error}"),
    })?;
    Ok(remote_envelope)
}

fn record_local_runtime_smoke_notification(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    config: CommunicationBoundaryConfig,
    recorded_at_ms: u64,
) -> Result<NotificationDispatchRecord, RuntimeLifecycleError> {
    let dispatch = DeterministicNotificationBoundary::new()
        .publish(&crate::NotificationPublishRequest {
            id: format!("runtime-smoke-notification-{}", lifecycle_record.id),
            notification: OperatorNotification {
                id: format!("runtime-smoke-notification-{}", lifecycle_record.id),
                severity: NotificationSeverity::Info,
                title: "Local runtime smoke communications validation".to_owned(),
                body: "Runtime smoke recorded local command, remote-review, envelope, channel-adapter, and notification boundaries".to_owned(),
                channels: vec!["cli".to_owned(), "local-stdout".to_owned()],
                created_at_unix_ms: recorded_at_ms.saturating_add(8),
            },
            config,
            channel_safety: vec![
                runtime_smoke_channel_safety("cli", recorded_at_ms),
                runtime_smoke_channel_safety("local-stdout", recorded_at_ms),
            ],
            now_unix_ms: recorded_at_ms.saturating_add(9),
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke communications notification failed: {error}"),
        })?;
    append_notification_dispatch_audit(journal, &dispatch, recorded_at_ms.saturating_add(10))
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke communications notification audit failed: {error}"),
        })?;
    persist_notification_dispatch_checkpoint(store, &dispatch, recorded_at_ms.saturating_add(11))
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke communications notification checkpoint failed: {error}"),
    })?;
    Ok(dispatch)
}

fn record_local_runtime_smoke_channel_adapter(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    recorded_at_ms: u64,
) -> Result<ChannelAdapterValidationReport, RuntimeLifecycleError> {
    let channel_adapter = validate_channel_adapter(&ChannelAdapterValidationRequest {
        validation_id: "runtime-smoke-channel-adapter".to_owned(),
        channel: NotificationChannelProfile::from_identifier("cli"),
        envelope: remote_envelope.clone(),
        dispatch: dispatch.clone(),
        adapter_authentication_reference: "runtime-smoke-local-channel-ref".to_owned(),
        platform_identity: "runtime-smoke-operator".to_owned(),
        replay_nonce: "runtime-smoke-channel-nonce".to_owned(),
        channel_authenticated: true,
        platform_identity_authorized: true,
        replay_protection_checked: true,
        require_delivery_kill_switch: true,
        require_audit_state_preflight: true,
        require_delivery_idempotency: true,
        require_rate_limit_controls: true,
        require_outage_backoff_controls: true,
        require_payload_redaction: true,
        replay_nonce_reused: false,
        provider_rate_limited: false,
        provider_outage_observed: false,
        outbound_delivery_requested: false,
        outbound_network_used: false,
        message_delivered: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
        validated_at_unix_ms: recorded_at_ms.saturating_add(12),
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel adapter validation failed: {error}"),
    })?;
    append_channel_adapter_validation_audit(
        journal,
        &channel_adapter,
        recorded_at_ms.saturating_add(13),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel adapter audit failed: {error}"),
    })?;
    persist_channel_adapter_validation_checkpoint(
        store,
        &channel_adapter,
        recorded_at_ms.saturating_add(14),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel adapter checkpoint failed: {error}"),
    })?;
    Ok(channel_adapter)
}

#[derive(Debug, Clone, Copy)]
enum RuntimeSmokeChannelAdapterScenario {
    MissingChannelLogin,
    ReplayNonceReused,
    ProviderUnavailable,
}

impl RuntimeSmokeChannelAdapterScenario {
    const fn validation_id(self) -> &'static str {
        match self {
            Self::MissingChannelLogin => "runtime-smoke-channel-no-login",
            Self::ReplayNonceReused => "runtime-smoke-channel-replay",
            Self::ProviderUnavailable => "runtime-smoke-channel-provider-down",
        }
    }

    const fn channel_authenticated(self) -> bool {
        !matches!(self, Self::MissingChannelLogin)
    }

    const fn platform_identity_authorized(self) -> bool {
        !matches!(self, Self::MissingChannelLogin)
    }

    const fn replay_nonce_reused(self) -> bool {
        matches!(self, Self::ReplayNonceReused)
    }

    const fn provider_unavailable(self) -> bool {
        matches!(self, Self::ProviderUnavailable)
    }
}

fn build_local_runtime_smoke_channel_adapter_scenario(
    scenario: RuntimeSmokeChannelAdapterScenario,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    validated_at_unix_ms: u64,
) -> Result<ChannelAdapterValidationReport, RuntimeLifecycleError> {
    let validation_id = scenario.validation_id();
    validate_channel_adapter(&ChannelAdapterValidationRequest {
        validation_id: validation_id.to_owned(),
        channel: NotificationChannelProfile::from_identifier("cli"),
        envelope: remote_envelope.clone(),
        dispatch: dispatch.clone(),
        adapter_authentication_reference: "runtime-smoke-local-channel-ref".to_owned(),
        platform_identity: "runtime-smoke-operator".to_owned(),
        replay_nonce: format!("{validation_id}-n"),
        channel_authenticated: scenario.channel_authenticated(),
        platform_identity_authorized: scenario.platform_identity_authorized(),
        replay_protection_checked: true,
        require_delivery_kill_switch: true,
        require_audit_state_preflight: true,
        require_delivery_idempotency: true,
        require_rate_limit_controls: true,
        require_outage_backoff_controls: true,
        require_payload_redaction: true,
        replay_nonce_reused: scenario.replay_nonce_reused(),
        provider_rate_limited: scenario.provider_unavailable(),
        provider_outage_observed: scenario.provider_unavailable(),
        outbound_delivery_requested: false,
        outbound_network_used: false,
        message_delivered: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
        validated_at_unix_ms,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel adapter scenario failed: {error}"),
    })
}

fn record_local_runtime_smoke_channel_session(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    accepted: &ChannelAdapterValidationReport,
    recorded_at_ms: u64,
) -> Result<ChannelSessionValidationReport, RuntimeLifecycleError> {
    let unauthenticated = build_local_runtime_smoke_channel_adapter_scenario(
        RuntimeSmokeChannelAdapterScenario::MissingChannelLogin,
        remote_envelope,
        dispatch,
        recorded_at_ms.saturating_add(15),
    )?;
    let replay = build_local_runtime_smoke_channel_adapter_scenario(
        RuntimeSmokeChannelAdapterScenario::ReplayNonceReused,
        remote_envelope,
        dispatch,
        recorded_at_ms.saturating_add(16),
    )?;
    let provider_unavailable = build_local_runtime_smoke_channel_adapter_scenario(
        RuntimeSmokeChannelAdapterScenario::ProviderUnavailable,
        remote_envelope,
        dispatch,
        recorded_at_ms.saturating_add(17),
    )?;
    let channel_session = validate_channel_session(
        "runtime-smoke-channel-session",
        &[
            accepted.clone(),
            unauthenticated,
            replay,
            provider_unavailable,
        ],
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel session failed: {error}"),
    })?;
    append_channel_session_validation_audit(
        journal,
        &channel_session,
        recorded_at_ms.saturating_add(18),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel session audit failed: {error}"),
    })?;
    persist_channel_session_validation_checkpoint(
        store,
        &channel_session,
        recorded_at_ms.saturating_add(19),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke channel session checkpoint failed: {error}"),
    })?;
    Ok(channel_session)
}

fn record_local_runtime_smoke_platform_adapter(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    recorded_at_ms: u64,
) -> Result<PlatformAdapterReviewReport, RuntimeLifecycleError> {
    let platform_adapter = review_platform_adapter_controls(&PlatformAdapterReviewRequest {
        review_id: "runtime-smoke-platform-adapter".to_owned(),
        channel: NotificationChannelProfile::from_identifier("cli"),
        envelope: remote_envelope.clone(),
        token_reference_present: true,
        token_secret_material_present: false,
        platform_identity_verified: true,
        platform_identity_authorized: true,
        channel_permission_granted: true,
        command_injection_blocked: true,
        require_delivery_kill_switch: true,
        require_audit_state_preflight: true,
        require_delivery_idempotency: true,
        require_rate_limit_controls: true,
        require_outage_backoff_controls: true,
        require_payload_redaction: true,
        token_revoked: false,
        provider_rate_limited: false,
        provider_outage_observed: false,
        outbound_delivery_requested: false,
        outbound_network_used: false,
        message_delivered: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
        reviewed_at_unix_ms: recorded_at_ms.saturating_add(20),
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke platform adapter failed: {error}"),
    })?;
    append_platform_adapter_review_audit(
        journal,
        &platform_adapter,
        recorded_at_ms.saturating_add(21),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke platform adapter audit failed: {error}"),
    })?;
    persist_platform_adapter_review_checkpoint(
        store,
        &platform_adapter,
        recorded_at_ms.saturating_add(22),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke platform adapter checkpoint failed: {error}"),
    })?;
    Ok(platform_adapter)
}

fn runtime_smoke_communications_config() -> CommunicationBoundaryConfig {
    CommunicationBoundaryConfig {
        notification_channels: vec![
            NotificationChannelProfile::from_identifier("cli"),
            NotificationChannelProfile::from_identifier("local-stdout"),
        ],
        ..CommunicationBoundaryConfig::default()
    }
}

fn runtime_smoke_channel_safety(
    channel_id: &str,
    recorded_at_ms: u64,
) -> NotificationChannelSafetyState {
    NotificationChannelSafetyState {
        channel_id: channel_id.to_owned(),
        messages_sent_in_window: 0,
        max_messages_per_window: 30,
        window_started_at_unix_ms: recorded_at_ms,
        window_ends_at_unix_ms: recorded_at_ms.saturating_add(60_000),
        outage_active: false,
        outage_reason: String::new(),
    }
}

#[allow(clippy::too_many_lines)]
fn record_local_runtime_smoke_dashboard(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    communications_dispatch: &NotificationDispatchRecord,
    rendered_at_ms: u64,
) -> Result<RuntimeSmokeDashboardArtifacts, RuntimeLifecycleError> {
    let renderer = DeterministicDashboardRenderer;
    let render = renderer
        .render(DashboardRenderRequest {
            config: DashboardBoundaryConfig::default(),
            snapshot: runtime_smoke_dashboard_snapshot(
                lifecycle_record,
                communications_dispatch,
                rendered_at_ms,
            ),
            access: DashboardAccessContext::local_render(Some("runtime-smoke-local".to_owned())),
            requested_panels: Vec::new(),
            operator_label: Some("runtime-smoke-local".to_owned()),
            rendered_at_ms,
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke dashboard render failed: {error}"),
        })?;
    append_dashboard_render_audit(journal, &render, rendered_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke dashboard audit failed: {error}"),
        }
    })?;
    persist_dashboard_render_checkpoint(store, &render, rendered_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke dashboard checkpoint failed: {error}"),
        }
    })?;

    let hosted_security = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
        review_id: "runtime-smoke-dashboard-hosted-security".to_owned(),
        authentication_required: true,
        authorization_required: true,
        csrf_protection_required: true,
        csrf_token_rotation_required: true,
        secure_headers_required: true,
        clickjacking_protection_required: true,
        rate_limit_required: true,
        max_requests_per_minute: 30,
        loopback_only_required: true,
        audit_state_preflight_required: true,
        session_revocation_required: true,
        operator_role_review_required: true,
        read_only_controls_required: true,
        public_exposure_requested: false,
        server_start_requested: false,
        live_controls_requested: false,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted security review failed: {error}"),
    })?;
    append_dashboard_hosted_security_review_audit(
        journal,
        &hosted_security,
        rendered_at_ms.saturating_add(1),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted security audit failed: {error}"),
    })?;
    persist_dashboard_hosted_security_review_checkpoint(
        store,
        &hosted_security,
        rendered_at_ms.saturating_add(2),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted security checkpoint failed: {error}"),
    })?;

    let hosted_preflight = preflight_dashboard_hosted_request(&DashboardHostedRequestPreflight {
        preflight_id: "runtime-smoke-dashboard-hosted-preflight".to_owned(),
        bind_host: "127.0.0.1".to_owned(),
        access_source: DashboardAccessSource::BrowserSession,
        method: DashboardHostedRequestMethod::Get,
        authenticated: true,
        authorized: true,
        csrf_token_present: false,
        csrf_token_valid: true,
        content_security_policy_present: true,
        frame_protection_present: true,
        content_type_options_present: true,
        referrer_policy_present: true,
        requests_in_current_window: 1,
        max_requests_per_minute: 30,
        public_exposure_requested: false,
        server_start_requested: false,
        live_controls_requested: false,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted preflight failed: {error}"),
    })?;
    append_dashboard_hosted_request_preflight_audit(
        journal,
        &hosted_preflight,
        rendered_at_ms.saturating_add(3),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted preflight audit failed: {error}"),
    })?;
    persist_dashboard_hosted_request_preflight_checkpoint(
        store,
        &hosted_preflight,
        rendered_at_ms.saturating_add(4),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted preflight checkpoint failed: {error}"),
    })?;

    let hosted_validation = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
        validation_id: "runtime-smoke-dashboard-hosted-request".to_owned(),
        render_record: render.clone(),
        bind_host: "127.0.0.1".to_owned(),
        requested_port: 0,
        method: DashboardHostedRequestMethod::Get,
        request_path: "/".to_owned(),
        authenticated: true,
        authorized: true,
        csrf_token_present: false,
        csrf_token_valid: true,
        secure_headers_required: true,
        requests_in_current_window: 1,
        max_requests_per_minute: 30,
        public_exposure_requested: false,
        live_controls_requested: false,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted request validation failed: {error}"),
    })?;
    append_dashboard_hosted_request_validation_audit(
        journal,
        &hosted_validation,
        rendered_at_ms.saturating_add(5),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted validation audit failed: {error}"),
    })?;
    persist_dashboard_hosted_request_validation_checkpoint(
        store,
        &hosted_validation,
        rendered_at_ms.saturating_add(6),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke dashboard hosted validation checkpoint failed: {error}"),
    })?;

    Ok(RuntimeSmokeDashboardArtifacts {
        render,
        hosted_security,
        hosted_preflight,
        hosted_validation,
    })
}

fn runtime_smoke_dashboard_snapshot(
    lifecycle_record: &RuntimeLifecycleRecord,
    communications_dispatch: &NotificationDispatchRecord,
    generated_at_ms: u64,
) -> DashboardSnapshot {
    DashboardSnapshot {
        snapshot_id: format!("runtime-smoke-dashboard-{}", lifecycle_record.id),
        generated_at_ms,
        runtime_mode: RuntimeMode::Paper,
        production_readiness_percent: 0,
        open_gap_count: 1,
        opportunity_count: 0,
        pending_plan_count: 0,
        notification_record_count: 1,
        panels: vec![
            DashboardPanel::new(
                DashboardPanelKind::SystemStatus,
                "Local runtime smoke",
                "Runtime smoke records are local only",
                vec![
                    DashboardPanelItem::new(
                        "lifecycle",
                        format!("{:?}", lifecycle_record.status),
                        DashboardSeverity::Ok,
                    ),
                    DashboardPanelItem::new(
                        "plan checkpoint",
                        lifecycle_record.plan_checkpoint_key.clone(),
                        DashboardSeverity::Ok,
                    ),
                ],
            ),
            DashboardPanel::new(
                DashboardPanelKind::Communications,
                "Local communications",
                "Notification dispatch stayed inside deterministic local boundary",
                vec![
                    DashboardPanelItem::new(
                        "dispatch status",
                        format!("{:?}", communications_dispatch.status),
                        DashboardSeverity::Ok,
                    ),
                    DashboardPanelItem::new(
                        "outbound network",
                        communications_dispatch.outbound_network_used.to_string(),
                        DashboardSeverity::Ok,
                    ),
                ],
            ),
            DashboardPanel::new(
                DashboardPanelKind::Gaps,
                "Production blockers",
                "Runtime smoke does not approve production deployment",
                vec![DashboardPanelItem::new(
                    "production ready",
                    "false",
                    DashboardSeverity::Warning,
                )],
            ),
        ],
        warnings: runtime_deployment_smoke_unresolved_blockers(),
    }
}

fn record_local_runtime_smoke_testing(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    recorded_at_ms: u64,
) -> Result<RuntimeSmokeTestingArtifacts, RuntimeLifecycleError> {
    let config = ValidationHarnessConfig::default();
    let plan = runtime_smoke_validation_plan(lifecycle_record, recorded_at_ms);
    let property_check = run_local_validation_property_checks(&plan, &config).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke property checks failed: {error}"),
        }
    })?;
    let validation_run = DeterministicValidationHarness
        .validate_plan(ValidationRunRequest {
            config,
            plan,
            requested_at_ms: recorded_at_ms,
            operator_label: Some("runtime-smoke-local-validation".to_owned()),
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke validation run failed: {error}"),
        })?;

    append_validation_run_audit(journal, &validation_run, recorded_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke validation-run audit failed: {error}"),
        }
    })?;
    persist_validation_run_checkpoint(store, &validation_run, recorded_at_ms).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke validation-run checkpoint failed: {error}"),
        }
    })?;
    append_property_check_report_audit(journal, &property_check, recorded_at_ms.saturating_add(1))
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke property-check audit failed: {error}"),
        })?;
    persist_property_check_report_checkpoint(
        store,
        &property_check,
        recorded_at_ms.saturating_add(1),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke property-check checkpoint failed: {error}"),
    })?;

    Ok(RuntimeSmokeTestingArtifacts {
        validation_run,
        property_check,
    })
}

fn runtime_smoke_validation_plan(
    lifecycle_record: &RuntimeLifecycleRecord,
    generated_at_ms: u64,
) -> ValidationPlan {
    const VALID_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let mut audit_state_case = ValidationTestCase::new(
        "runtime-smoke-audit-state-reopen",
        "Runtime smoke recovers local audit and SQLite checkpoints",
        ValidationSuiteKind::Replay,
        "runtime",
        ExpectedValidationOutcome::Pass,
    );
    audit_state_case
        .fixture_ids
        .push("runtime-smoke-lifecycle-fixture".to_owned());

    ValidationPlan {
        plan_id: format!("runtime-smoke-validation-{}", lifecycle_record.id),
        generated_at_ms,
        execution_mode: ValidationExecutionMode::FixtureReplayOnly,
        test_cases: vec![
            ValidationTestCase::new(
                "runtime-smoke-deny-live-execution",
                "Runtime smoke rejects live execution surfaces",
                ValidationSuiteKind::Policy,
                "runtime",
                ExpectedValidationOutcome::FailClosed,
            ),
            audit_state_case,
        ],
        fixtures: vec![ValidationFixtureRecord::synthetic(
            "runtime-smoke-lifecycle-fixture",
            FixtureKind::OperatorSurface,
            "runtime-smoke/local-lifecycle-checkpoint",
            Some(VALID_DIGEST.to_owned()),
            generated_at_ms,
        )],
        fuzz_corpora: vec![FuzzCorpusDefinition::local_only(
            "runtime-smoke-command-boundary-seeds",
            FuzzTargetKind::CommandParser,
            vec![FuzzSeedRecord::new(
                "runtime-smoke-status-command-seed",
                VALID_DIGEST,
                "runtime smoke local command boundary seed",
            )],
        )],
        backtest_scenarios: vec![BacktestScenarioDefinition {
            scenario_id: "runtime-smoke-paper-boundary-backtest".to_owned(),
            dataset: BacktestDatasetDefinition {
                dataset_id: "runtime-smoke-paper-fixture".to_owned(),
                base_asset: "BTC".to_owned(),
                quote_asset: "USD".to_owned(),
                venue_ids: vec!["paper-a".to_owned(), "paper-b".to_owned()],
                start_ms: generated_at_ms,
                end_ms: generated_at_ms.saturating_add(60_000),
                quote_count: 2,
                local_fixture_only: true,
                requires_live_network: false,
            },
            initial_balance_microunits: 1_000_000_000,
            max_drawdown_basis_points: 500,
            submits_live_orders: false,
            signs_or_broadcasts_transactions: false,
        }],
    }
}

fn record_local_runtime_smoke_paper_ledger(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    policy: &PolicyEngine,
    plan: &ExecutionPlanDraft,
    lifecycle_record: &RuntimeLifecycleRecord,
    recorded_at_ms: u64,
) -> Result<RuntimeSmokePaperArtifacts, RuntimeLifecycleError> {
    if plan.scope != ExecutionScope::Paper {
        return Ok(RuntimeSmokePaperArtifacts {
            applicable: false,
            execution_report_checkpointed: false,
            ledger_report: None,
            ledger_replay_validated: false,
            external_submission_performed: false,
            live_execution_performed: false,
        });
    }

    let first_intent =
        plan.intents
            .first()
            .ok_or_else(|| RuntimeLifecycleError::ValidationFailed {
                reason: "runtime smoke paper ledgering requires at least one paper intent"
                    .to_owned(),
            })?;
    let paper_adapter = PaperExecutionAdapter::new(
        format!("runtime-smoke-paper-{}", lifecycle_record.id),
        policy.clone(),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke paper adapter setup failed: {error}"),
    })?;
    append_paper_execution_intent_audit(journal, first_intent, recorded_at_ms).map_err(
        |error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke paper intent audit failed: {error}"),
        },
    )?;
    let paper_report = paper_adapter.submit(first_intent).map_err(|error| {
        RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke paper execution report failed: {error}"),
        }
    })?;
    append_paper_execution_report_audit(journal, &paper_report, recorded_at_ms).map_err(
        |error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke paper report audit failed: {error}"),
        },
    )?;
    persist_paper_execution_report_checkpoint(store, &paper_report, recorded_at_ms).map_err(
        |error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke paper report checkpoint failed: {error}"),
        },
    )?;

    let mut ledger = PaperBalanceLedger::new(
        runtime_smoke_paper_initial_balances(plan),
        recorded_at_ms.saturating_add(1),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke paper ledger setup failed: {error}"),
    })?;
    let ledger_report = ledger_execution_adapter_run_paper_fills(
        journal,
        store,
        &mut ledger,
        plan,
        &lifecycle_record.adapter_run,
        recorded_at_ms.saturating_add(2),
    )
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke adapter-run paper ledgering failed: {error}"),
    })?;
    let replay_report = ledger
        .validate_replay(recorded_at_ms.saturating_add(6))
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke paper ledger replay failed: {error}"),
        })?;

    Ok(RuntimeSmokePaperArtifacts {
        applicable: true,
        execution_report_checkpointed: true,
        external_submission_performed: ledger_report.external_submission_performed,
        live_execution_performed: ledger_report.live_execution_performed,
        ledger_report: Some(ledger_report),
        ledger_replay_validated: replay_report.balanced,
    })
}

fn runtime_smoke_paper_initial_balances(plan: &ExecutionPlanDraft) -> Vec<PaperAssetBalance> {
    let mut balances: Vec<PaperAssetBalance> = Vec::new();
    for intent in &plan.intents {
        let required_available = (intent.notional_quote * 4.0).max(1_000.0);
        if let Some(balance) = balances.iter_mut().find(|balance| {
            balance.venue == intent.venue && balance.asset.eq_ignore_ascii_case(&intent.quote_asset)
        }) {
            balance.available += required_available;
        } else {
            balances.push(PaperAssetBalance {
                venue: intent.venue.clone(),
                asset: intent.quote_asset.clone(),
                available: required_available,
                reserved: 0.0,
            });
        }
    }
    balances
}

fn record_local_runtime_smoke_failure_capture(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    lifecycle_record: &RuntimeLifecycleRecord,
    captured_at_ms: u64,
) -> Result<RuntimeFailureCaptureRecord, RuntimeLifecycleError> {
    let failure_capture_record = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
        failure_id: format!("runtime-smoke-failure-capture-{}", lifecycle_record.id),
        component: "runtime-lifecycle".to_owned(),
        kind: RuntimeFailureKind::ValidationFailure,
        severity: ObservabilitySeverity::Warning,
        summary: "local runtime smoke failure-capture probe".to_owned(),
        detail:
            "local smoke validates sanitized failure capture audit/state recovery without a real crash"
                .to_owned(),
        config: ObservabilityBoundaryConfig::default(),
        access: ObservabilityAccessContext::local_collection(Some(
            "runtime-smoke-failure-capture".to_owned(),
        )),
        captured_at_ms,
    })
    .map_err(|error| RuntimeLifecycleError::ValidationFailed {
        reason: format!("runtime smoke failure capture failed: {error}"),
    })?;
    append_runtime_failure_capture_audit(journal, &failure_capture_record, captured_at_ms)
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke failure capture audit failed: {error}"),
        })?;
    persist_runtime_failure_capture_checkpoint(store, &failure_capture_record, captured_at_ms)
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke failure capture checkpoint failed: {error}"),
        })?;
    Ok(failure_capture_record)
}

#[allow(clippy::too_many_lines)]
fn collect_local_runtime_smoke_observability(
    lifecycle_record: &RuntimeLifecycleRecord,
    graceful_shutdown_record: &RuntimeGracefulShutdownRecord,
    collected_at_ms: u64,
) -> Result<crate::ObservabilityRecord, RuntimeLifecycleError> {
    let forbidden_side_effect_count = [
        lifecycle_record.external_submission_performed,
        lifecycle_record.live_execution_performed,
        graceful_shutdown_record.external_submission_performed,
        graceful_shutdown_record.live_execution_performed,
    ]
    .iter()
    .filter(|performed| **performed)
    .count();
    let forbidden_side_effect_count = i64::try_from(forbidden_side_effect_count).map_err(|_| {
        RuntimeLifecycleError::ValidationFailed {
            reason: "runtime smoke observability side-effect count overflowed".to_owned(),
        }
    })?;

    let snapshot = ObservabilitySnapshot {
        snapshot_id: format!("runtime-smoke-{}", lifecycle_record.id),
        generated_at_ms: collected_at_ms,
        components: vec![
            ComponentHealthStatus::new(
                "runtime-lifecycle",
                HealthStatus::Healthy,
                "local lifecycle completed with adapter checkpoint",
                collected_at_ms,
            ),
            ComponentHealthStatus::new(
                "runtime-graceful-shutdown",
                HealthStatus::Healthy,
                "local graceful-shutdown checkpoint persisted",
                collected_at_ms,
            ),
            ComponentHealthStatus::new(
                "runtime-audit-state",
                HealthStatus::Healthy,
                "local audit and SQLite state checkpoints prepared for recovery validation",
                collected_at_ms,
            ),
        ],
        logs: vec![StructuredLogEvent::new(
            format!("runtime-smoke-log-{}", lifecycle_record.id),
            ObservabilitySeverity::Info,
            "runtime-smoke",
            "local runtime smoke collected lifecycle checkpoint summary",
            vec![
                StructuredLogField::new(
                    "plan_checkpoint",
                    lifecycle_record.plan_checkpoint_key.clone(),
                ),
                StructuredLogField::new(
                    "adapter_checkpoint",
                    lifecycle_record.adapter_run_checkpoint_key.clone(),
                ),
                StructuredLogField::new(
                    "adapter_recovery_plan_checkpoint",
                    lifecycle_record
                        .adapter_recovery_plan_checkpoint_key
                        .clone(),
                ),
                StructuredLogField::new(
                    "shutdown_checkpoint",
                    graceful_shutdown_record.shutdown_checkpoint_key.clone(),
                ),
            ],
            collected_at_ms,
        )],
        metrics: vec![
            MetricSample::new(
                "runtime_smoke_forbidden_side_effects",
                MetricKind::Counter,
                forbidden_side_effect_count,
                "count",
                Vec::new(),
                collected_at_ms,
            ),
            MetricSample::new(
                "runtime_smoke_checkpoint_records",
                MetricKind::Gauge,
                3,
                "count",
                Vec::new(),
                collected_at_ms,
            ),
        ],
        runbooks: Vec::new(),
        warnings: lifecycle_record
            .warnings
            .iter()
            .chain(graceful_shutdown_record.warnings.iter())
            .cloned()
            .collect(),
    };

    let collector = DeterministicObservabilityCollector;
    collector
        .collect(ObservabilityCollectionRequest {
            config: ObservabilityBoundaryConfig::default(),
            snapshot,
            access: ObservabilityAccessContext::local_collection(Some(
                "runtime-smoke-local".to_owned(),
            )),
            operator_label: Some("runtime-smoke-local".to_owned()),
            collected_at_ms,
        })
        .map_err(|error| RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime smoke observability collection failed: {error}"),
        })
}

/// Runtime lifecycle errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeLifecycleError {
    /// Request or record validation failed.
    ValidationFailed { reason: String },
    /// Planner or plan validation failed.
    Planner(crate::ExecutionPlannerError),
    /// Adapter validation or evaluation failed.
    Adapter(ExecutionAdapterError),
    /// Audit append/replay failed.
    Audit(AuditError),
    /// State checkpoint persistence failed.
    State(StateStoreError),
}

impl fmt::Display for RuntimeLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { reason } => {
                write!(formatter, "runtime lifecycle validation failed: {reason}")
            }
            Self::Planner(error) => write!(formatter, "runtime lifecycle planner failed: {error}"),
            Self::Adapter(error) => write!(formatter, "runtime lifecycle adapter failed: {error}"),
            Self::Audit(error) => write!(formatter, "runtime lifecycle audit failed: {error}"),
            Self::State(error) => write!(formatter, "runtime lifecycle state failed: {error}"),
        }
    }
}

impl Error for RuntimeLifecycleError {}

impl From<AuditError> for RuntimeLifecycleError {
    fn from(error: AuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<StateStoreError> for RuntimeLifecycleError {
    fn from(error: StateStoreError) -> Self {
        Self::State(error)
    }
}

impl From<ExecutionAdapterError> for RuntimeLifecycleError {
    fn from(error: ExecutionAdapterError) -> Self {
        Self::Adapter(error)
    }
}

fn lifecycle_event(
    request: &RuntimeLifecycleRequest,
    kind: AuditEventKind,
    message: &str,
) -> AuditEvent {
    AuditEvent::new(
        format!("runtime:{}:start", request.id),
        kind,
        "runtime-lifecycle",
        "runtime",
        message,
    )
    .with_metadata("lifecycle_id", AuditValue::Text(request.id.clone()))
    .with_metadata("plan_id", AuditValue::Text(request.plan.id.clone()))
    .with_metadata(
        "adapter_request_id",
        AuditValue::Text(request.adapter_request_id.clone()),
    )
    .with_metadata("live_execution", AuditValue::Bool(false))
    .with_metadata("external_submission", AuditValue::Bool(false))
}

fn checkpoint_event(
    request: &RuntimeLifecycleRequest,
    kind: AuditEventKind,
    message: &str,
    checkpoint: &StateCheckpoint,
) -> AuditEvent {
    AuditEvent::new(
        format!("runtime:{}:{}", request.id, checkpoint.key),
        kind,
        "runtime-lifecycle",
        "runtime",
        message,
    )
    .with_metadata("lifecycle_id", AuditValue::Text(request.id.clone()))
    .with_metadata("plan_id", AuditValue::Text(request.plan.id.clone()))
    .with_metadata("checkpoint_key", AuditValue::Text(checkpoint.key.clone()))
    .with_metadata(
        "checkpoint_subsystem",
        AuditValue::Text(checkpoint.subsystem.clone()),
    )
    .with_metadata(
        "checkpoint_updated_at_unix_ms",
        AuditValue::Unsigned(checkpoint.updated_at_unix_ms),
    )
    .with_metadata("live_execution", AuditValue::Bool(false))
    .with_metadata("external_submission", AuditValue::Bool(false))
}

fn graceful_shutdown_event(request: &RuntimeGracefulShutdownRequest, message: &str) -> AuditEvent {
    AuditEvent::new(
        format!("runtime:{}:graceful-shutdown", request.id),
        AuditEventKind::RuntimeLifecycle,
        "runtime-lifecycle",
        "runtime",
        message,
    )
    .with_metadata("shutdown_id", AuditValue::Text(request.id.clone()))
    .with_metadata("shutdown_reason", AuditValue::Text(request.reason.clone()))
    .with_metadata("live_execution", AuditValue::Bool(false))
    .with_metadata("external_submission", AuditValue::Bool(false))
}

fn validate_runtime_backup_target(
    primary_path: &Path,
    backup_path: &Path,
    artifact_label: &str,
) -> Result<(), RuntimeLifecycleError> {
    if primary_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} primary path is required"),
        });
    }
    if backup_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} backup path is required"),
        });
    }
    if primary_path == backup_path {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} backup path must differ from primary"),
        });
    }
    if backup_path.exists() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} backup path must not already exist"),
        });
    }
    Ok(())
}

fn validate_runtime_smoke_target(
    target_path: &Path,
    artifact_label: &str,
) -> Result<(), RuntimeLifecycleError> {
    if target_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} smoke path is required"),
        });
    }
    if target_path.exists() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} smoke path must not already exist"),
        });
    }
    if artifact_label == "state" {
        for suffix in ["-wal", "-shm"] {
            let related = std::path::PathBuf::from(format!("{}{suffix}", target_path.display()));
            if related.exists() {
                return Err(RuntimeLifecycleError::ValidationFailed {
                    reason: format!(
                        "runtime state smoke related path must not already exist: {suffix}"
                    ),
                });
            }
        }
    }
    Ok(())
}

fn copy_runtime_backup_file(
    source_path: &Path,
    destination_path: &Path,
    artifact_label: &str,
) -> Result<(), RuntimeLifecycleError> {
    if let Some(parent) = destination_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| StateStoreError::BackendFailed {
                reason: format!("failed to create runtime {artifact_label} backup parent: {error}"),
            })?;
        }
    }
    fs::copy(source_path, destination_path).map_err(|error| StateStoreError::BackendFailed {
        reason: format!("failed to copy runtime {artifact_label} backup: {error}"),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        review_runtime_load_profile, run_local_graceful_shutdown_checkpoint,
        run_local_runtime_lifecycle, validate_local_runtime_backup_restore,
        validate_local_runtime_deployment_smoke, validate_local_runtime_restart_recovery,
        validate_local_runtime_restart_recovery_with_trace_recovery,
        RuntimeDeploymentAuditSqliteTranscript, RuntimeDeploymentAuditSqliteTranscriptStatus,
        RuntimeDeploymentBackupRestoreTranscript, RuntimeDeploymentBackupRestoreTranscriptStatus,
        RuntimeDeploymentGracefulShutdownTranscript,
        RuntimeDeploymentGracefulShutdownTranscriptStatus, RuntimeDeploymentPermissionTranscript,
        RuntimeDeploymentPermissionTranscriptStatus, RuntimeDeploymentSmokeLoadIteration,
        RuntimeDeploymentSmokeLoadValidationReport, RuntimeDeploymentSmokeValidationReport,
        RuntimeDeploymentSmokeValidationRequest, RuntimeDeploymentSqliteSchemaMigrationTranscript,
        RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus, RuntimeGracefulShutdownRequest,
        RuntimeLifecycleError, RuntimeLifecycleRequest, RuntimeLifecycleStatus,
        RuntimeLoadProfileReviewRequest, RuntimeLoadProfileReviewStatus,
        RuntimeOpportunityTraceRecoverySummary, RuntimeProductionPreflightRequest,
        RuntimeProductionPreflightStatus, RuntimeRecoveredOpportunityTraceSummary,
        RuntimeRestartRecoveryDisposition, RuntimeServiceManagerKind,
        RuntimeServiceManagerLifecycleEvent, RuntimeServiceManagerLifecycleEventKind,
        RuntimeServiceManagerLifecycleRehearsalRequest,
        RuntimeServiceManagerLifecycleRehearsalStatus, RuntimeServiceManagerLifecycleTranscript,
        RuntimeServiceManagerLifecycleTranscriptStatus,
        EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY,
        EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
        RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION, RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, AuditEvent, AuditEventKind,
        DeterministicExecutionPlanner, ExecutionAdapterConfig, ExecutionPlanner,
        ExecutionPlannerConfig, ExecutionPlannerRequest, FeeAdjustedEdge, FeeEstimate, FeeSchedule,
        InMemoryStateStore, LiquidityRole, MarketPair, NormalizedQuote, OpportunityCandidate,
        OpportunityDiscoveryConfig, OpportunityDiscoveryRequest,
        OpportunityHistoricalFixtureCorpus, OpportunityLeg, OpportunityLegSide,
        OpportunityReplayCorpus, OpportunityReplayExpectation, OpportunityReplayScenario,
        OpportunityRouteKind, OpportunityScore, PolicyEngine, PriceLevel, SqliteWalStateStore,
        StateCheckpoint, StateStore, StateStoreError, VenueKind, VenueRef,
    };
    use std::{
        env, fs,
        path::PathBuf,
        process,
        sync::{Arc, Barrier, Mutex},
        thread,
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
    fn runtime_lifecycle_audits_and_persists_before_adapter_completion() {
        let path = temp_audit_path("runtime-lifecycle");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let mut store = InMemoryStateStore::new();
        let policy = policy();
        let request = request(&policy);

        let record = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");

        assert_eq!(
            record.status,
            RuntimeLifecycleStatus::AdapterRunCheckpointed
        );
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert_eq!(record.start_audit_sequence, 1);
        assert_eq!(record.plan_checkpoint_audit_sequence, 5);
        assert_eq!(record.adapter_complete_audit_sequence, 6);
        assert_eq!(record.adapter_recovery_plan_audit_sequence, 7);
        assert_eq!(journal.next_sequence(), 8);
        assert!(store
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_some());
        assert!(store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_some());
        assert!(store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
            .expect("adapter recovery-plan checkpoint reads")
            .is_some());

        let reopened = AppendOnlyAuditJournal::open(&path).expect("journal reopens");
        assert_eq!(reopened.next_sequence(), 8);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_lifecycle_rejects_live_scope_before_audit_or_state() {
        let path = temp_audit_path("runtime-live-denied");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let mut store = InMemoryStateStore::new();
        let policy = policy();
        let mut request = request(&policy);
        request.plan.scope = crate::ExecutionScope::Live;

        let error = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect_err("live lifecycle must be rejected");

        assert!(error.to_string().contains("live-scope"));
        assert_eq!(journal.next_sequence(), 1);
        assert!(store.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_lifecycle_state_permission_failure_stops_before_adapter() {
        let path = temp_audit_path("runtime-state-permission-denied");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let mut store = PermissionDeniedStateStore::default();
        let policy = policy();
        let request = request(&policy);

        let error = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect_err("state permission failure must fail closed");

        match error {
            RuntimeLifecycleError::State(StateStoreError::BackendFailed { reason }) => {
                assert!(reason.contains("simulated permission-denied state path"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(store.put_attempts, 1);
        assert_eq!(journal.next_sequence(), 2);

        let reopened = AppendOnlyAuditJournal::open(&path).expect("journal reopens");
        assert_eq!(reopened.next_sequence(), 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_lifecycle_persists_through_sqlite_wal_store() {
        let audit_path = temp_audit_path("runtime-sqlite");
        let state_path = temp_state_path("runtime-sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        let record = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");

        drop(store);

        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let plan_checkpoint = reopened
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .expect("plan checkpoint exists");
        let adapter_checkpoint = reopened
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .expect("adapter checkpoint exists");
        let recovery_checkpoint = reopened
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
            .expect("adapter recovery-plan checkpoint reads")
            .expect("adapter recovery-plan checkpoint exists");

        assert_eq!(plan_checkpoint.key, record.plan_checkpoint_key);
        assert_eq!(adapter_checkpoint.key, record.adapter_run_checkpoint_key);
        assert_eq!(
            recovery_checkpoint.key,
            record.adapter_recovery_plan_checkpoint_key
        );
        assert_eq!(
            adapter_checkpoint.updated_at_unix_ms,
            record.created_at_unix_ms
        );

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn graceful_shutdown_checkpoint_reopens_audit_and_sqlite_state() {
        let audit_path = temp_audit_path("runtime-graceful-shutdown");
        let state_path = temp_state_path("runtime-graceful-shutdown");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");

        let record = run_local_graceful_shutdown_checkpoint(
            &mut journal,
            &mut store,
            RuntimeGracefulShutdownRequest {
                id: "shutdown-1".to_owned(),
                reason: "operator-requested-local-stop".to_owned(),
                now_unix_ms: 30_000,
            },
        )
        .expect("graceful shutdown checkpoint should persist");

        assert_eq!(record.shutdown_start_audit_sequence, 1);
        assert_eq!(record.shutdown_checkpoint_audit_sequence, 2);
        assert_eq!(
            record.shutdown_checkpoint_key,
            RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY
        );
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);

        drop(store);
        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 3);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let checkpoint = reopened_store
            .get_checkpoint(RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY)
            .expect("shutdown checkpoint reads")
            .expect("shutdown checkpoint exists");
        assert_eq!(checkpoint.value, record.shutdown_checkpoint_value);
        assert_eq!(checkpoint.updated_at_unix_ms, record.created_at_unix_ms);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_backup_restore_replays_audit_and_sqlite_checkpoints() {
        let audit_path = temp_audit_path("runtime-backup-primary");
        let state_path = temp_state_path("runtime-backup-primary");
        let backup_audit_path = temp_audit_path("runtime-backup-copy");
        let backup_state_path = temp_state_path("runtime-backup-copy");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        let lifecycle_record =
            run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
                .expect("runtime lifecycle should complete");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_backup_restore(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
        )
        .expect("local runtime backup/restore validation should pass");

        assert_eq!(report.audit_records_replayed, 7);
        assert!(report.audit_restore_check_passed);
        assert!(report.sqlite_restore_check_passed);
        assert!(report.plan_checkpoint_restored);
        assert!(report.adapter_checkpoint_restored);
        assert!(report.adapter_recovery_plan_checkpoint_restored);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let restored_journal =
            AppendOnlyAuditJournal::open(&backup_audit_path).expect("backup journal reopens");
        assert_eq!(restored_journal.next_sequence(), 8);

        let restored_state =
            SqliteWalStateStore::open(&backup_state_path).expect("backup sqlite reopens");
        let plan_checkpoint = restored_state
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .expect("plan checkpoint exists");
        let adapter_checkpoint = restored_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .expect("adapter checkpoint exists");
        let recovery_checkpoint = restored_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
            .expect("adapter recovery-plan checkpoint reads")
            .expect("adapter recovery-plan checkpoint exists");
        assert_eq!(plan_checkpoint.key, lifecycle_record.plan_checkpoint_key);
        assert_eq!(
            adapter_checkpoint.key,
            lifecycle_record.adapter_run_checkpoint_key
        );
        assert_eq!(
            recovery_checkpoint.key,
            lifecycle_record.adapter_recovery_plan_checkpoint_key
        );

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
    }

    #[test]
    fn runtime_restart_recovery_replays_audit_and_reopens_sqlite_checkpoints() {
        let audit_path = temp_audit_path("runtime-restart-recovery");
        let state_path = temp_state_path("runtime-restart-recovery");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");
        run_local_graceful_shutdown_checkpoint(
            &mut journal,
            &mut store,
            RuntimeGracefulShutdownRequest {
                id: "shutdown-before-restart-recovery".to_owned(),
                reason: "local-restart-recovery-test".to_owned(),
                now_unix_ms: 50_000,
            },
        )
        .expect("graceful shutdown checkpoint should persist");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect("restart recovery validation should pass");

        assert_eq!(report.audit_records_replayed, 9);
        assert!(report.audit_replay_check_passed);
        assert!(report.sqlite_reopen_check_passed);
        assert!(report.plan_checkpoint_recovered);
        assert!(report.adapter_checkpoint_recovered);
        assert!(report.adapter_recovery_plan_checkpoint_recovered);
        assert!(report.graceful_shutdown_checkpoint_recovered);
        assert_eq!(
            report.recovery_disposition,
            RuntimeRestartRecoveryDisposition::ReadyForLocalReview
        );
        assert!(report.local_review_ready);
        assert!(!report.opportunity_trace_recovery_validated);
        assert_eq!(report.opportunity_trace_discovered_candidates, 0);
        assert_eq!(report.opportunity_trace_recovered_checkpoints, 0);
        assert!(report.opportunity_trace_recovered_summaries.is_empty());
        assert_eq!(report.opportunity_trace_missing_checkpoints, 0);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_restart_recovery_with_trace_recovery_includes_opportunity_trace_summary() {
        let audit_path = temp_audit_path("runtime-restart-recovery-with-trace");
        let state_path = temp_state_path("runtime-restart-recovery-with-trace");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");
        run_local_graceful_shutdown_checkpoint(
            &mut journal,
            &mut store,
            RuntimeGracefulShutdownRequest {
                id: "shutdown-before-trace-recovery".to_owned(),
                reason: "local-restart-recovery-trace-test".to_owned(),
                now_unix_ms: 50_000,
            },
        )
        .expect("graceful shutdown checkpoint should persist");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_restart_recovery_with_trace_recovery(
            &audit_path,
            &state_path,
            &policy,
        )
        .expect("restart recovery validation with trace recovery should pass");

        assert!(report.adapter_recovery_plan_checkpoint_recovered);
        let opportunity_trace_recovery = report
            .opportunity_trace_recovery
            .expect("opportunity trace recovery summary should be present");
        assert!(report.opportunity_trace_recovery_validated);
        assert_eq!(
            report.opportunity_trace_discovered_candidates,
            opportunity_trace_recovery.discovered_candidates
        );
        assert_eq!(
            report.opportunity_trace_recovered_checkpoints,
            opportunity_trace_recovery.recovered_trace_checkpoints
        );
        assert_eq!(
            report.opportunity_trace_recovered_summaries,
            opportunity_trace_recovery.recovered_trace_summaries
        );
        assert_eq!(
            report.opportunity_trace_missing_checkpoints,
            opportunity_trace_recovery.missing_trace_checkpoints
        );
        assert!(opportunity_trace_recovery.trace_recovery_validated);
        assert!(opportunity_trace_recovery.discovered_candidates > 0);
        assert_eq!(
            u64::try_from(opportunity_trace_recovery.recovered_trace_summaries.len())
                .expect("summary count fits u64"),
            opportunity_trace_recovery.recovered_trace_checkpoints
        );
        assert!(opportunity_trace_recovery
            .recovered_trace_summaries
            .iter()
            .all(|summary| summary.audit_sequence > 0
                && summary.leg_count > 0
                && !summary.trace_id.is_empty()
                && !summary.planner_request_id.is_empty()));
        assert_eq!(
            opportunity_trace_recovery.discovered_candidates,
            opportunity_trace_recovery.recovered_trace_checkpoints
                + opportunity_trace_recovery.missing_trace_checkpoints
        );

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_trace_recovery_from_duplicate_candidate_corpus_preserves_deduplicated_counts() {
        let audit_path = temp_audit_path("runtime-trace-dedup");
        let state_path = temp_state_path("runtime-trace-dedup");
        let policy = policy();
        let corpus = duplicate_candidate_trace_corpus();

        let summary = super::validate_runtime_opportunity_trace_recovery_for_corpus(
            &corpus,
            &audit_path,
            &state_path,
            &policy,
        )
        .expect("deduplicated runtime opportunity trace recovery should pass");

        assert_eq!(summary.corpus_id, "runtime-dedup-trace-corpus");
        assert!(summary.trace_recovery_validated);
        assert_eq!(summary.discovered_candidates, 1);
        assert_eq!(summary.audit_trace_records_replayed, 1);
        assert_eq!(summary.recovered_trace_checkpoints, 1);
        assert_eq!(summary.missing_trace_checkpoints, 0);
        assert_eq!(summary.recovered_trace_summaries.len(), 1);
        assert_eq!(
            summary.recovered_trace_summaries[0].planner_request_id,
            "phase27-planner-handoff-1"
        );
        assert_eq!(summary.recovered_trace_summaries[0].audit_sequence, 1);
        assert!(summary.recovered_trace_summaries[0].leg_count > 0);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_restart_recovery_needs_operator_review_without_shutdown_checkpoint() {
        let audit_path = temp_audit_path("runtime-restart-recovery-review-needed");
        let state_path = temp_state_path("runtime-restart-recovery-review-needed");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect("restart recovery validation should pass with operator review");

        assert_eq!(report.audit_records_replayed, 7);
        assert!(report.audit_replay_check_passed);
        assert!(report.sqlite_reopen_check_passed);
        assert!(report.plan_checkpoint_recovered);
        assert!(report.adapter_checkpoint_recovered);
        assert!(report.adapter_recovery_plan_checkpoint_recovered);
        assert!(!report.graceful_shutdown_checkpoint_recovered);
        assert_eq!(
            report.recovery_disposition,
            RuntimeRestartRecoveryDisposition::NeedsOperatorReview
        );
        assert!(report.local_review_ready);
        assert!(!report.opportunity_trace_recovery_validated);
        assert_eq!(report.opportunity_trace_discovered_candidates, 0);
        assert_eq!(report.opportunity_trace_recovered_checkpoints, 0);
        assert!(report.opportunity_trace_recovered_summaries.is_empty());
        assert_eq!(report.opportunity_trace_missing_checkpoints, 0);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_restart_recovery_fails_closed_when_sqlite_checkpoints_missing() {
        let audit_path = temp_audit_path("runtime-restart-recovery-missing-state");
        let state_path = temp_state_path("runtime-restart-recovery-missing-state");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");

        journal
            .append_event(AuditEvent::new(
                "runtime:incomplete-recovery:start",
                AuditEventKind::RuntimeLifecycle,
                "runtime-lifecycle",
                "runtime",
                "runtime lifecycle started without durable checkpoints",
            ))
            .expect("audit event should append");
        drop(store);
        drop(journal);

        let error = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect_err("restart recovery must fail closed when checkpoints are missing");

        assert!(error
            .to_string()
            .contains("coherent local planner, adapter, and adapter recovery-plan checkpoints"));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 2);
        let reopened_state = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        assert!(reopened_state
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_none());
        assert!(reopened_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_none());

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn concurrent_runtime_lifecycles_share_audit_and_sqlite_state() {
        let audit_path = temp_audit_path("runtime-concurrent");
        let state_path = temp_state_path("runtime-concurrent");
        let workers = 4_usize;
        let barrier = Arc::new(Barrier::new(workers));
        let open_lock = Arc::new(Mutex::new(()));
        let handles = (0..workers)
            .map(|worker| {
                let audit_path = audit_path.clone();
                let state_path = state_path.clone();
                let barrier = Arc::clone(&barrier);
                let open_lock = Arc::clone(&open_lock);
                thread::spawn(move || {
                    let policy = policy();
                    let mut request = request(&policy);
                    request.id = format!("runtime-concurrent-{worker}");
                    request.adapter_request_id = format!("adapter-concurrent-{worker}");
                    request.plan.id = format!("plan-concurrent-{worker}");
                    request.now_unix_ms = 40_000 + u64::try_from(worker).unwrap_or(u64::MAX);

                    let (mut journal, mut store) = {
                        let _guard = open_lock.lock().expect("open lock should not be poisoned");
                        (
                            AppendOnlyAuditJournal::open(&audit_path).expect("journal opens"),
                            SqliteWalStateStore::open(&state_path).expect("sqlite store opens"),
                        )
                    };
                    barrier.wait();

                    let record =
                        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
                            .expect("runtime lifecycle should complete");

                    assert!(!record.external_submission_performed);
                    assert!(!record.live_execution_performed);
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().expect("worker should not panic");
        }

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(
            reopened_journal.next_sequence(),
            1 + u64::try_from(workers * 7).unwrap_or(u64::MAX)
        );

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        assert!(reopened_store
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_some());
        assert!(reopened_store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_some());
        assert!(reopened_store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
            .expect("adapter recovery-plan checkpoint reads")
            .is_some());
        reopened_store
            .integrity_check()
            .expect("sqlite integrity check should pass");

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn runtime_deployment_smoke_validates_local_artifact_sequence() {
        let audit_path = temp_audit_path("runtime-deployment-smoke");
        let state_path = temp_state_path("runtime-deployment-smoke");
        let backup_audit_path = temp_audit_path("runtime-deployment-smoke-backup");
        let backup_state_path = temp_state_path("runtime-deployment-smoke-backup");
        let audit_validation_workspace = temp_workspace_path("runtime-deployment-smoke-audit");
        let policy = policy();
        let lifecycle_request = request(&policy);
        let shutdown_request = RuntimeGracefulShutdownRequest {
            id: "shutdown-before-deployment-smoke".to_owned(),
            reason: "local-deployment-smoke-test".to_owned(),
            now_unix_ms: 60_000,
        };

        let report = validate_local_runtime_deployment_smoke(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
            &audit_validation_workspace,
            &policy,
            RuntimeDeploymentSmokeValidationRequest {
                lifecycle_request,
                shutdown_request,
                validated_at_unix_ms: 70_000,
            },
        )
        .expect("local deployment-like smoke validation should pass");

        assert_eq!(
            report.validation_version,
            RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION
        );
        assert!(report.lifecycle_completed);
        assert!(report.graceful_shutdown_checkpointed);
        assert!(report.backup_restore_validated);
        assert!(report.restart_recovery_validated);
        assert!(report.audit_durability_validated);
        assert!(report.concurrent_lifecycle_validated);
        assert_eq!(report.concurrent_lifecycle_workers, 3);
        assert!(report.concurrent_lifecycle_audit_records_replayed >= 12);
        assert!(report.concurrent_lifecycle_sqlite_integrity_check_passed);
        assert!(!report.concurrent_lifecycle_external_submission_performed);
        assert!(!report.concurrent_lifecycle_live_execution_performed);
        assert!(report.observability_collected);
        assert!(report.observability_checkpoint_recovered);
        assert!(report.observability_operations_reviewed);
        assert!(report.observability_operations_checkpoint_recovered);
        assert!(report.observability_export_dry_run_rendered);
        assert!(report.observability_export_checkpoint_recovered);
        assert!(report.observability_alert_route_dispatched);
        assert!(report.observability_alert_route_checkpoint_recovered);
        assert!(report.observability_endpoint_preflighted);
        assert!(report.observability_endpoint_checkpoint_recovered);
        assert!(report.observability_loopback_bind_validated);
        assert!(report.observability_loopback_bind_checkpoint_recovered);
        assert!(report.observability_metrics_scrape_preflighted);
        assert!(report.observability_metrics_scrape_checkpoint_recovered);
        assert!(report.observability_metrics_endpoint_validated);
        assert!(report.observability_metrics_endpoint_checkpoint_recovered);
        assert!(report.observability_tracing_captured);
        assert!(report.observability_tracing_checkpoint_recovered);
        assert!(!report.observability_metrics_endpoint_started);
        assert!(report.observability_local_metrics_request_served);
        assert!(!report.observability_public_network_exposed);
        assert!(!report.observability_outbound_alerts_sent);
        assert!(!report.observability_telemetry_exported);
        assert!(!report.observability_production_ready);
        assert!(report.communications_command_routed);
        assert!(report.communications_command_route_checkpoint_recovered);
        assert!(report.communications_remote_command_reviewed);
        assert!(report.communications_remote_command_review_checkpoint_recovered);
        assert!(report.communications_platform_command_ingress_validated);
        assert!(report.communications_platform_command_ingress_checkpoint_recovered);
        assert!(report.communications_remote_command_envelope_validated);
        assert!(report.communications_remote_command_envelope_checkpoint_recovered);
        assert!(report.communications_channel_adapter_validated);
        assert!(report.communications_channel_adapter_checkpoint_recovered);
        assert!(report.communications_channel_session_validated);
        assert!(report.communications_channel_session_checkpoint_recovered);
        assert!(report.communications_platform_adapter_reviewed);
        assert!(report.communications_platform_adapter_checkpoint_recovered);
        assert!(report.communications_notification_dispatched);
        assert!(report.communications_notification_checkpoint_recovered);
        assert!(!report.communications_execution_enabled);
        assert!(!report.communications_remote_commands_enabled);
        assert!(!report.communications_outbound_network_used);
        assert!(report.dashboard_rendered);
        assert!(report.dashboard_checkpoint_recovered);
        assert!(report.dashboard_hosted_security_reviewed);
        assert!(report.dashboard_hosted_security_checkpoint_recovered);
        assert!(report.dashboard_hosted_request_preflighted);
        assert!(report.dashboard_hosted_request_preflight_checkpoint_recovered);
        assert!(report.dashboard_hosted_request_validated);
        assert!(report.dashboard_hosted_request_validation_checkpoint_recovered);
        assert!(report.dashboard_panel_count > 0);
        assert!(!report.dashboard_server_started);
        assert!(report.dashboard_local_one_shot_request_served);
        assert!(!report.dashboard_public_network_exposed);
        assert!(!report.dashboard_live_controls_enabled);
        assert!(!report.dashboard_hosted_production_ready);
        assert!(report.validation_run_recorded);
        assert!(report.validation_run_checkpoint_recovered);
        assert!(report.validation_property_checks_passed);
        assert!(report.validation_property_checkpoint_recovered);
        assert!(!report.validation_external_fuzzer_invoked);
        assert!(!report.validation_live_network_used);
        assert!(!report.validation_live_execution_submitted);
        assert!(!report.validation_signing_or_broadcast_performed);
        assert!(report.paper_ledger_applicable);
        assert!(report.paper_execution_report_checkpointed);
        assert!(report.paper_execution_report_checkpoint_recovered);
        assert!(report.paper_ledger_checkpointed);
        assert!(report.paper_ledger_checkpoint_recovered);
        assert!(report.paper_modeled_fills_settled > 0);
        assert!(report.paper_ledger_audit_records_appended > 0);
        assert!(report.paper_ledger_replay_validated);
        assert!(!report.paper_ledger_external_submission_performed);
        assert!(!report.paper_ledger_live_execution_performed);
        assert!(report.failure_capture_validated);
        assert!(report.failure_capture_checkpoint_recovered);
        assert!(!report.failure_capture_metrics_endpoint_started);
        assert!(!report.failure_capture_public_network_exposed);
        assert!(!report.failure_capture_outbound_alerts_sent);
        assert!(!report.failure_capture_external_submission_performed);
        assert!(!report.failure_capture_live_execution_performed);
        assert!(report.restart_audit_records_replayed >= 16);
        assert!(report.backup_audit_records_replayed >= 16);
        assert!(report.restart_plan_checkpoint_recovered);
        assert!(report.restart_adapter_checkpoint_recovered);
        assert!(report.restart_adapter_recovery_plan_checkpoint_recovered);
        assert!(report.restart_graceful_shutdown_checkpoint_recovered);
        assert!(report.restart_opportunity_trace_recovery_validated);
        let opportunity_trace_recovery = report
            .opportunity_trace_recovery
            .expect("runtime smoke should include opportunity trace recovery");
        assert_eq!(
            report.restart_opportunity_trace_discovered_candidates,
            opportunity_trace_recovery.discovered_candidates
        );
        assert_eq!(
            report.restart_opportunity_trace_recovered_checkpoints,
            opportunity_trace_recovery.recovered_trace_checkpoints
        );
        assert_eq!(
            report.restart_opportunity_trace_recovered_summaries,
            opportunity_trace_recovery.recovered_trace_summaries
        );
        assert_eq!(
            report.restart_opportunity_trace_missing_checkpoints,
            opportunity_trace_recovery.missing_trace_checkpoints
        );
        assert!(opportunity_trace_recovery.trace_recovery_validated);
        assert!(opportunity_trace_recovery.discovered_candidates > 0);
        assert_eq!(
            u64::try_from(opportunity_trace_recovery.recovered_trace_summaries.len())
                .expect("summary count fits u64"),
            opportunity_trace_recovery.recovered_trace_checkpoints
        );
        assert!(opportunity_trace_recovery
            .recovered_trace_summaries
            .iter()
            .all(|summary| summary.audit_sequence > 0
                && summary.leg_count > 0
                && !summary.trace_id.is_empty()
                && !summary.route_kind.is_empty()));
        assert_eq!(
            opportunity_trace_recovery.discovered_candidates,
            opportunity_trace_recovery.recovered_trace_checkpoints
                + opportunity_trace_recovery.missing_trace_checkpoints
        );
        assert_eq!(
            report.recovery_disposition,
            RuntimeRestartRecoveryDisposition::ReadyForLocalReview
        );
        assert!(!report.service_manager_action_performed);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        assert!(report
            .unresolved_blockers
            .iter()
            .any(|blocker| blocker.contains("service-manager")));

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
        let _ = fs::remove_dir_all(audit_validation_workspace);
    }

    #[test]
    fn runtime_deployment_smoke_rejects_blocked_state_path_before_artifact_creation() {
        let audit_path = temp_audit_path("runtime-smoke-blocked-state");
        let state_path = temp_state_path("runtime-smoke-blocked-state");
        let backup_audit_path = temp_audit_path("runtime-smoke-blocked-state-backup");
        let backup_state_path = temp_state_path("runtime-smoke-blocked-state-backup");
        let audit_validation_workspace = temp_workspace_path("runtime-smoke-blocked-state-audit");

        fs::write(
            &state_path,
            b"pre-existing deployment-host state placeholder",
        )
        .expect("blocked state placeholder should be written");

        let error = validate_local_runtime_deployment_smoke(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
            &audit_validation_workspace,
            &policy(),
            RuntimeDeploymentSmokeValidationRequest {
                lifecycle_request: request(&policy()),
                shutdown_request: RuntimeGracefulShutdownRequest {
                    id: "shutdown-before-blocked-state".to_owned(),
                    reason: "local-deployment-smoke-blocked-state-test".to_owned(),
                    now_unix_ms: 60_000,
                },
                validated_at_unix_ms: 70_000,
            },
        )
        .expect_err("pre-existing state path must fail closed before smoke artifacts");

        assert!(error
            .to_string()
            .contains("runtime state smoke path must not already exist"));
        assert!(
            !audit_path.exists(),
            "audit smoke artifact should not be created after preflight failure"
        );
        assert!(
            !backup_audit_path.exists(),
            "backup audit artifact should not be created after preflight failure"
        );
        assert!(
            !backup_state_path.exists(),
            "backup state artifact should not be created after preflight failure"
        );
        assert!(
            !audit_validation_workspace.exists(),
            "audit durability workspace should not be created after preflight failure"
        );

        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        let _ = fs::remove_dir_all(audit_validation_workspace);
    }

    #[test]
    fn runtime_deployment_smoke_rejects_blocked_audit_path_before_artifact_creation() {
        let audit_path = temp_audit_path("runtime-smoke-blocked-audit");
        let state_path = temp_state_path("runtime-smoke-blocked-audit");
        let backup_audit_path = temp_audit_path("runtime-smoke-blocked-audit-backup");
        let backup_state_path = temp_state_path("runtime-smoke-blocked-audit-backup");
        let audit_validation_workspace = temp_workspace_path("runtime-smoke-blocked-audit");

        fs::write(
            &audit_path,
            b"pre-existing deployment-host audit placeholder",
        )
        .expect("blocked audit placeholder should be written");

        let error = validate_local_runtime_deployment_smoke(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
            &audit_validation_workspace,
            &policy(),
            RuntimeDeploymentSmokeValidationRequest {
                lifecycle_request: request(&policy()),
                shutdown_request: RuntimeGracefulShutdownRequest {
                    id: "shutdown-before-blocked-audit".to_owned(),
                    reason: "local-deployment-smoke-blocked-audit-test".to_owned(),
                    now_unix_ms: 60_000,
                },
                validated_at_unix_ms: 70_000,
            },
        )
        .expect_err("pre-existing audit path must fail closed before smoke artifacts");

        assert!(error
            .to_string()
            .contains("runtime audit smoke path must not already exist"));
        assert!(
            !state_path.exists(),
            "state smoke artifact should not be created after preflight failure"
        );
        assert!(
            !backup_audit_path.exists(),
            "backup audit artifact should not be created after preflight failure"
        );
        assert!(
            !backup_state_path.exists(),
            "backup state artifact should not be created after preflight failure"
        );
        assert!(
            !audit_validation_workspace.exists(),
            "audit durability workspace should not be created after preflight failure"
        );

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
        let _ = fs::remove_file(backup_audit_path);
        let _ = fs::remove_dir_all(audit_validation_workspace);
    }

    #[test]
    fn runtime_deployment_smoke_accepts_cli_shaped_long_lifecycle_ids() {
        let audit_path = temp_audit_path("runtime-smoke-long-id");
        let state_path = temp_state_path("runtime-smoke-long-id");
        let backup_audit_path = temp_audit_path("runtime-smoke-long-id-backup");
        let backup_state_path = temp_state_path("runtime-smoke-long-id-backup");
        let audit_validation_workspace = temp_workspace_path("runtime-smoke-long-id-audit");
        let policy = policy();
        let mut lifecycle_request = request(&policy);
        lifecycle_request.id =
            "cli-runtime-smoke-lifecycle-run-1-with-local-validation-suffix".to_owned();
        lifecycle_request.adapter_request_id =
            "cli-runtime-smoke-adapter-request-run-1-with-local-validation-suffix".to_owned();
        lifecycle_request.plan.id =
            "cli-runtime-smoke-plan-run-1-with-local-validation-suffix".to_owned();

        let report = validate_local_runtime_deployment_smoke(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
            &audit_validation_workspace,
            &policy,
            RuntimeDeploymentSmokeValidationRequest {
                lifecycle_request,
                shutdown_request: RuntimeGracefulShutdownRequest {
                    id: "shutdown-before-long-id-smoke".to_owned(),
                    reason: "local-deployment-smoke-long-id-test".to_owned(),
                    now_unix_ms: 60_000,
                },
                validated_at_unix_ms: 70_000,
            },
        )
        .expect("local deployment-like smoke should accept CLI-shaped long ids");

        assert!(report.communications_remote_command_envelope_validated);
        assert!(report.communications_platform_command_ingress_validated);
        assert!(report.communications_platform_command_ingress_checkpoint_recovered);
        assert!(report.communications_remote_command_envelope_checkpoint_recovered);
        assert!(report.communications_channel_session_validated);
        assert!(report.communications_platform_adapter_reviewed);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
        let _ = fs::remove_dir_all(audit_validation_workspace);
    }

    #[test]
    fn runtime_deployment_smoke_load_report_aggregates_local_iterations() {
        let report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 7,
                report: valid_runtime_smoke_report(7, 12),
            },
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-2".to_owned(),
                elapsed_ms: 11,
                report: valid_runtime_smoke_report(8, 12),
            },
        ])
        .expect("valid local smoke iterations should aggregate");

        assert_eq!(
            report.validation_version,
            RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION
        );
        assert_eq!(report.iterations_attempted, 2);
        assert_eq!(report.iterations_passed, 2);
        assert_eq!(report.min_elapsed_ms, 7);
        assert_eq!(report.max_elapsed_ms, 11);
        assert_eq!(report.average_elapsed_ms, 9);
        assert_eq!(report.total_elapsed_ms, 18);
        assert_eq!(report.restart_audit_records_replayed, 15);
        assert_eq!(report.backup_audit_records_replayed, 15);
        assert_eq!(report.opportunity_trace_recovered_checkpoints, 24);
        assert_eq!(report.opportunity_trace_recovered_summaries, 24);
        assert_eq!(report.opportunity_trace_missing_checkpoints, 0);
        assert!(!report.service_manager_action_performed);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn runtime_deployment_smoke_load_report_rejects_side_effect_reports() {
        let mut side_effect_report = valid_runtime_smoke_report(7, 12);
        side_effect_report.external_submission_performed = true;

        let error = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 7,
                report: side_effect_report,
            },
        ])
        .expect_err("side-effect report should be rejected before aggregation");

        assert!(error.to_string().contains("external submission"));
    }

    #[test]
    fn runtime_load_profile_review_accepts_local_budgets_without_readiness() {
        let load_report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 40,
                report: valid_runtime_smoke_report(7, 12),
            },
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-2".to_owned(),
                elapsed_ms: 60,
                report: valid_runtime_smoke_report(8, 12),
            },
        ])
        .expect("valid local smoke iterations should aggregate");

        let report = review_runtime_load_profile(RuntimeLoadProfileReviewRequest {
            review_id: "local-runtime-load-profile".to_owned(),
            load_report,
            max_average_elapsed_ms: 60,
            max_single_iteration_elapsed_ms: 100,
            max_total_elapsed_ms: 120,
            observed_peak_memory_mb: 128,
            max_peak_memory_mb: 256,
            observed_peak_cpu_percent: 25,
            max_peak_cpu_percent: 80,
            deployment_host_load_evidence_available: false,
            live_feed_backpressure_evidence_available: false,
            target_runtime_evidence_available: false,
            service_manager_action_performed: false,
            external_calls_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 100_000,
        })
        .expect("local load profile should review successfully");

        assert_eq!(
            report.status,
            RuntimeLoadProfileReviewStatus::ReadyForLocalReview
        );
        assert_eq!(report.iterations_reviewed, 2);
        assert!(report.latency_budget_met);
        assert!(report.resource_budget_met);
        assert!(report.replay_recovery_evidence_validated);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.remaining_external_evidence.is_empty());
        assert!(!report.service_manager_action_performed);
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn runtime_load_profile_review_blocks_budget_overruns() {
        let load_report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 40,
                report: valid_runtime_smoke_report(7, 12),
            },
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-2".to_owned(),
                elapsed_ms: 60,
                report: valid_runtime_smoke_report(8, 12),
            },
        ])
        .expect("valid local smoke iterations should aggregate");

        let report = review_runtime_load_profile(RuntimeLoadProfileReviewRequest {
            review_id: "local-runtime-load-profile-blocked".to_owned(),
            load_report,
            max_average_elapsed_ms: 10,
            max_single_iteration_elapsed_ms: 20,
            max_total_elapsed_ms: 30,
            observed_peak_memory_mb: 512,
            max_peak_memory_mb: 256,
            observed_peak_cpu_percent: 95,
            max_peak_cpu_percent: 80,
            deployment_host_load_evidence_available: false,
            live_feed_backpressure_evidence_available: false,
            target_runtime_evidence_available: false,
            service_manager_action_performed: false,
            external_calls_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 100_001,
        })
        .expect("blocked local load profile should still return a report");

        assert_eq!(report.status, RuntimeLoadProfileReviewStatus::Blocked);
        assert!(!report.latency_budget_met);
        assert!(!report.resource_budget_met);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "local-latency-budget-exceeded"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "local-resource-budget-exceeded"));
        assert!(!report.production_ready);
    }

    #[test]
    fn runtime_load_profile_review_rejects_side_effect_claims() {
        let load_report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 40,
                report: valid_runtime_smoke_report(7, 12),
            },
        ])
        .expect("valid local smoke iterations should aggregate");

        let error = review_runtime_load_profile(RuntimeLoadProfileReviewRequest {
            review_id: "local-runtime-load-profile-unsafe".to_owned(),
            load_report,
            max_average_elapsed_ms: 100,
            max_single_iteration_elapsed_ms: 100,
            max_total_elapsed_ms: 100,
            observed_peak_memory_mb: 128,
            max_peak_memory_mb: 256,
            observed_peak_cpu_percent: 25,
            max_peak_cpu_percent: 80,
            deployment_host_load_evidence_available: false,
            live_feed_backpressure_evidence_available: false,
            target_runtime_evidence_available: false,
            service_manager_action_performed: false,
            external_calls_performed: false,
            live_execution_performed: true,
            production_ready_claimed: true,
            validated_at_unix_ms: 100_002,
        })
        .expect_err("side-effect/runtime readiness claims must be rejected");

        assert!(error.to_string().contains("local-only"));
    }

    #[test]
    fn runtime_production_preflight_blocks_until_production_host_evidence_exists() {
        let smoke_report = valid_runtime_smoke_report(7, 12);
        let load_report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 7,
                report: smoke_report.clone(),
            },
        ])
        .expect("local smoke load report should aggregate");

        let report =
            super::preflight_production_runtime_validation(RuntimeProductionPreflightRequest {
                preflight_id: "local-production-runtime-preflight".to_owned(),
                smoke_report,
                load_report,
                service_manager_lifecycle_evidence_available: false,
                deployment_host_permission_evidence_available: false,
                physical_disk_full_evidence_available: false,
                retention_execution_evidence_available: false,
                rollback_drill_evidence_available: false,
                incident_response_evidence_available: false,
                observability_runtime_evidence_available: false,
                service_manager_action_performed: false,
                external_submission_performed: false,
                live_execution_performed: false,
                production_ready_claimed: false,
                validated_at_unix_ms: 80_000,
            })
            .expect("local production-runtime preflight should produce a blocked report");

        assert!(report.local_smoke_validated);
        assert!(report.local_smoke_load_validated);
        assert_eq!(
            report.status,
            RuntimeProductionPreflightStatus::BlockedPendingProductionHostValidation
        );
        assert!(report.unresolved_blockers.iter().any(|blocker| {
            blocker.contains("service-manager-controlled deployment-host lifecycle")
        }));
        assert!(report
            .unresolved_blockers
            .iter()
            .any(|blocker| { blocker.contains("physical deployment-host disk-full evidence") }));
        assert!(!report.service_manager_action_performed);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn runtime_production_preflight_rejects_readiness_claims() {
        let smoke_report = valid_runtime_smoke_report(7, 12);
        let load_report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(vec![
            RuntimeDeploymentSmokeLoadIteration {
                iteration_id: "run-1".to_owned(),
                elapsed_ms: 7,
                report: smoke_report.clone(),
            },
        ])
        .expect("local smoke load report should aggregate");

        let error =
            super::preflight_production_runtime_validation(RuntimeProductionPreflightRequest {
                preflight_id: "local-production-runtime-preflight".to_owned(),
                smoke_report,
                load_report,
                service_manager_lifecycle_evidence_available: true,
                deployment_host_permission_evidence_available: true,
                physical_disk_full_evidence_available: true,
                retention_execution_evidence_available: true,
                rollback_drill_evidence_available: true,
                incident_response_evidence_available: true,
                observability_runtime_evidence_available: true,
                service_manager_action_performed: false,
                external_submission_performed: false,
                live_execution_performed: false,
                production_ready_claimed: true,
                validated_at_unix_ms: 80_000,
            })
            .expect_err("local preflight must reject production-readiness claims");

        assert!(error
            .to_string()
            .contains("must not claim production readiness"));
    }

    #[test]
    fn service_manager_lifecycle_transcript_validates_operator_evidence_shape() {
        let report = super::validate_service_manager_lifecycle_transcript(
            service_manager_lifecycle_transcript(true),
        )
        .expect("complete service-manager lifecycle transcript should validate");

        assert_eq!(
            report.status,
            RuntimeServiceManagerLifecycleTranscriptStatus::ReadyForExternalReview
        );
        assert_eq!(report.event_count, 7);
        assert!(report.start_evidence_present);
        assert!(report.runtime_smoke_evidence_present);
        assert!(report.graceful_shutdown_evidence_present);
        assert!(report.stop_evidence_present);
        assert!(report.restart_evidence_present);
        assert!(report.recovery_evidence_present);
        assert!(report.operator_controlled_events);
        assert!(report.non_secret_references_present);
        assert!(report.successful_event_outcomes);
        assert!(report.audit_replay_reference_present);
        assert!(report.sqlite_recovery_reference_present);
        assert!(report.concurrent_lifecycle_reference_present);
        assert_eq!(report.concurrent_lifecycle_worker_count, 3);
        assert!(report.concurrent_lifecycle_success);
        assert!(report.operator_approved);
        assert!(report.operator_lifecycle_rehearsal_reference_present);
        assert!(report.emergency_stop_review_reference_present);
        assert!(report.rollback_plan_review_reference_present);
        assert!(report.operator_review_window_current);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn service_manager_lifecycle_transcript_blocks_missing_restart_recovery() {
        let report = super::validate_service_manager_lifecycle_transcript(
            service_manager_lifecycle_transcript(false),
        )
        .expect("incomplete service-manager transcript should produce blocked report");

        assert_eq!(
            report.status,
            RuntimeServiceManagerLifecycleTranscriptStatus::Blocked
        );
        assert!(!report.restart_evidence_present);
        assert!(!report.recovery_evidence_present);
        assert!(!report.concurrent_lifecycle_reference_present);
        assert!(!report.concurrent_lifecycle_success);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-restart-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-recovery-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-operator-approval"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-concurrent-lifecycle-evidence"));
        assert!(!report.production_ready);
    }

    #[test]
    fn service_manager_lifecycle_transcript_rejects_validator_service_actions() {
        let mut transcript = service_manager_lifecycle_transcript(true);
        transcript.service_manager_action_performed_by_validator = true;

        let error = super::validate_service_manager_lifecycle_transcript(transcript)
            .expect_err("validator service-manager actions must fail closed");

        assert!(error
            .to_string()
            .contains("must not perform service actions"));
    }

    #[test]
    fn service_manager_lifecycle_rehearsal_validates_ordered_local_evidence() {
        let report = super::validate_service_manager_lifecycle_rehearsal(
            service_manager_lifecycle_rehearsal(true),
        )
        .expect("complete service-manager lifecycle rehearsal should validate");

        assert_eq!(
            report.status,
            RuntimeServiceManagerLifecycleRehearsalStatus::Validated
        );
        assert_eq!(report.event_count, 7);
        assert!(report.ordered_lifecycle_validated);
        assert!(report.operator_controlled_events);
        assert!(report.non_secret_references_present);
        assert!(report.successful_event_outcomes);
        assert!(report.start_evidence_present);
        assert!(report.runtime_smoke_evidence_present);
        assert!(report.graceful_shutdown_evidence_present);
        assert!(report.stop_evidence_present);
        assert!(report.restart_evidence_present);
        assert!(report.recovery_evidence_present);
        assert!(report.audit_replay_reference_present);
        assert!(report.sqlite_recovery_reference_present);
        assert!(report.concurrent_lifecycle_reference_present);
        assert_eq!(report.concurrent_lifecycle_worker_count, 3);
        assert!(report.concurrent_lifecycle_success);
        assert!(report.graceful_shutdown_checkpoint_reference_present);
        assert!(report.restart_recovery_reference_present);
        assert!(report.operator_approved);
        assert!(report.reviewer_approved);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.deployment_path_mutated_by_validator);
        assert!(!report.secrets_loaded);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn service_manager_lifecycle_rehearsal_blocks_missing_ordered_recovery() {
        let report = super::validate_service_manager_lifecycle_rehearsal(
            service_manager_lifecycle_rehearsal(false),
        )
        .expect("incomplete service-manager lifecycle rehearsal should block");

        assert_eq!(
            report.status,
            RuntimeServiceManagerLifecycleRehearsalStatus::Blocked
        );
        assert!(!report.ordered_lifecycle_validated);
        assert!(!report.restart_evidence_present);
        assert!(!report.recovery_evidence_present);
        assert!(!report.concurrent_lifecycle_reference_present);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-ordered-lifecycle-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-restart-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-restart-recovery-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-reviewer-approval"));
        assert!(!report.production_ready);
    }

    #[test]
    fn service_manager_lifecycle_rehearsal_rejects_validator_side_effects() {
        let mut request = service_manager_lifecycle_rehearsal(true);
        request.service_manager_action_performed_by_validator = true;

        let error = super::validate_service_manager_lifecycle_rehearsal(request)
            .expect_err("validator service-manager actions must fail closed");

        assert!(error
            .to_string()
            .contains("must not perform service actions"));
    }

    #[test]
    fn deployment_permission_transcript_validates_fail_closed_evidence_shape() {
        let report = super::validate_deployment_permission_transcript(
            deployment_permission_transcript(true),
        )
        .expect("complete deployment permission transcript should validate");

        assert_eq!(
            report.status,
            RuntimeDeploymentPermissionTranscriptStatus::ReadyForExternalReview
        );
        assert!(report.deployment_host_evidence);
        assert!(report.runtime_write_attempt_reference_present);
        assert!(report.runtime_write_permission_denied);
        assert!(report.runtime_write_error_classified);
        assert!(report.audit_write_failed_closed);
        assert!(report.state_write_failed_closed);
        assert!(report.adapter_evaluation_blocked);
        assert!(report.runtime_quiesced_or_degraded);
        assert!(report.audit_replay_after_restore_validated);
        assert!(report.sqlite_reopen_after_restore_validated);
        assert!(report.recovery_runbook_reference_present);
        assert_eq!(report.non_secret_reference_count, 9);
        assert!(report.operator_approved);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.permission_changed_by_validator);
        assert!(!report.production_path_mutated_by_validator);
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_permission_transcript_blocks_missing_host_permission_evidence() {
        let report = super::validate_deployment_permission_transcript(
            deployment_permission_transcript(false),
        )
        .expect("incomplete deployment permission transcript should produce blocked report");

        assert_eq!(
            report.status,
            RuntimeDeploymentPermissionTranscriptStatus::Blocked
        );
        assert!(!report.deployment_host_evidence);
        assert!(!report.runtime_write_attempt_reference_present);
        assert!(!report.runtime_write_permission_denied);
        assert!(!report.runtime_write_error_classified);
        assert!(!report.state_write_failed_closed);
        assert!(!report.adapter_evaluation_blocked);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-deployment-host-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-runtime-write-attempt-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-runtime-write-permission-denial-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-runtime-write-error-classification"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-state-write-fail-closed-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_permission_transcript_rejects_validator_permission_changes() {
        let mut transcript = deployment_permission_transcript(true);
        transcript.permission_changed_by_validator = true;

        let error = super::validate_deployment_permission_transcript(transcript)
            .expect_err("validator permission changes must fail closed");

        assert!(error.to_string().contains("must not change permissions"));
    }

    #[test]
    fn deployment_audit_sqlite_transcript_validates_recovery_evidence_shape() {
        let report = super::validate_deployment_audit_sqlite_transcript(
            deployment_audit_sqlite_transcript(true),
        )
        .expect("complete deployment audit/sqlite transcript should validate");

        assert_eq!(
            report.status,
            RuntimeDeploymentAuditSqliteTranscriptStatus::ReadyForExternalReview
        );
        assert!(report.deployment_host_evidence);
        assert!(report.service_lifecycle_reference_present);
        assert!(report.audit_append_reference_present);
        assert!(report.audit_replay_validated);
        assert!(report.audit_hash_chain_validated);
        assert!(report.sqlite_wal_mode_validated);
        assert!(report.sqlite_integrity_check_passed);
        assert!(report.sqlite_checkpoint_recovered);
        assert!(report.backup_restore_validated);
        assert!(report.concurrent_access_validated);
        assert!(report.recovery_runbook_reference_present);
        assert_eq!(report.non_secret_reference_count, 9);
        assert!(report.operator_approved);
        assert!(report.reviewer_approved);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.deployment_path_mutated_by_validator);
        assert!(!report.secrets_loaded);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_audit_sqlite_transcript_blocks_missing_recovery_evidence() {
        let report = super::validate_deployment_audit_sqlite_transcript(
            deployment_audit_sqlite_transcript(false),
        )
        .expect("incomplete deployment audit/sqlite transcript should produce blocked report");

        assert_eq!(
            report.status,
            RuntimeDeploymentAuditSqliteTranscriptStatus::Blocked
        );
        assert!(!report.deployment_host_evidence);
        assert!(!report.audit_replay_validated);
        assert!(!report.sqlite_checkpoint_recovered);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-deployment-host-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-audit-replay-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-sqlite-checkpoint-recovery-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_audit_sqlite_transcript_rejects_validator_side_effects() {
        let mut transcript = deployment_audit_sqlite_transcript(true);
        transcript.deployment_path_mutated_by_validator = true;

        let error = super::validate_deployment_audit_sqlite_transcript(transcript)
            .expect_err("validator deployment-path mutation must fail closed");

        assert!(error
            .to_string()
            .contains("must not perform service actions"));
    }

    #[test]
    fn deployment_backup_restore_transcript_validates_service_load_restore_evidence() {
        let report = super::validate_deployment_backup_restore_transcript(
            deployment_backup_restore_transcript(true),
        )
        .expect("complete deployment backup/restore transcript should validate");

        assert_eq!(
            report.status,
            RuntimeDeploymentBackupRestoreTranscriptStatus::ReadyForExternalReview
        );
        assert!(report.deployment_host_evidence);
        assert!(report.service_lifecycle_reference_present);
        assert!(report.backup_artifact_reference_present);
        assert!(report.restore_execution_reference_present);
        assert!(report.deployment_load_reference_present);
        assert!(report.audit_replay_after_restore_validated);
        assert!(report.audit_hash_chain_after_restore_validated);
        assert!(report.sqlite_integrity_after_restore_validated);
        assert!(report.sqlite_checkpoint_after_restore_validated);
        assert!(report.runtime_checkpoint_restore_validated);
        assert!(report.post_restore_runtime_smoke_passed);
        assert!(report.rollback_reference_present);
        assert!(report.recovery_runbook_reference_present);
        assert_eq!(report.non_secret_reference_count, 10);
        assert!(report.operator_approved);
        assert!(report.reviewer_approved);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.backup_restore_executed_by_validator);
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.deployment_path_mutated_by_validator);
        assert!(!report.secrets_loaded);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_backup_restore_transcript_blocks_missing_restore_evidence() {
        let report = super::validate_deployment_backup_restore_transcript(
            deployment_backup_restore_transcript(false),
        )
        .expect("incomplete deployment backup/restore transcript should block");

        assert_eq!(
            report.status,
            RuntimeDeploymentBackupRestoreTranscriptStatus::Blocked
        );
        assert!(!report.deployment_host_evidence);
        assert!(!report.restore_execution_reference_present);
        assert!(!report.deployment_load_reference_present);
        assert!(!report.runtime_checkpoint_restore_validated);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-restore-execution-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-deployment-load-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-runtime-checkpoint-restore-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_backup_restore_transcript_rejects_validator_execution() {
        let mut transcript = deployment_backup_restore_transcript(true);
        transcript.backup_restore_executed_by_validator = true;

        let error = super::validate_deployment_backup_restore_transcript(transcript)
            .expect_err("validator backup/restore execution must fail closed");

        assert!(error
            .to_string()
            .contains("must not execute backup/restore"));
    }

    #[test]
    fn deployment_graceful_shutdown_transcript_validates_shutdown_evidence() {
        let report = super::validate_deployment_graceful_shutdown_transcript(
            deployment_graceful_shutdown_transcript(true),
        )
        .expect("complete deployment graceful-shutdown transcript should validate");

        assert_eq!(
            report.status,
            RuntimeDeploymentGracefulShutdownTranscriptStatus::ReadyForExternalReview
        );
        assert!(report.deployment_host_evidence);
        assert!(report.service_lifecycle_reference_present);
        assert!(report.shutdown_request_reference_present);
        assert!(report.service_stopped_reference_present);
        assert!(report.graceful_shutdown_checkpoint_reference_present);
        assert!(report.audit_replay_after_shutdown_validated);
        assert!(report.sqlite_reopen_after_shutdown_validated);
        assert!(report.restart_recovery_after_shutdown_validated);
        assert!(report.post_shutdown_runtime_smoke_passed);
        assert_eq!(report.non_secret_reference_count, 9);
        assert!(report.operator_approved);
        assert!(report.reviewer_approved);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.deployment_path_mutated_by_validator);
        assert!(!report.secrets_loaded);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_graceful_shutdown_transcript_blocks_missing_shutdown_evidence() {
        let report = super::validate_deployment_graceful_shutdown_transcript(
            deployment_graceful_shutdown_transcript(false),
        )
        .expect("incomplete deployment graceful-shutdown transcript should block");

        assert_eq!(
            report.status,
            RuntimeDeploymentGracefulShutdownTranscriptStatus::Blocked
        );
        assert!(!report.deployment_host_evidence);
        assert!(!report.shutdown_request_reference_present);
        assert!(!report.service_stopped_reference_present);
        assert!(!report.restart_recovery_after_shutdown_validated);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-shutdown-request-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-restart-recovery-after-shutdown-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_graceful_shutdown_transcript_rejects_validator_service_actions() {
        let mut transcript = deployment_graceful_shutdown_transcript(true);
        transcript.service_manager_action_performed_by_validator = true;

        let error = super::validate_deployment_graceful_shutdown_transcript(transcript)
            .expect_err("validator service-manager action must fail closed");

        assert!(error
            .to_string()
            .contains("must not perform service actions"));
    }

    #[test]
    fn deployment_sqlite_schema_migration_transcript_validates_ready_evidence() {
        let report = super::validate_deployment_sqlite_schema_migration_transcript(
            deployment_sqlite_schema_migration_transcript(true),
        )
        .expect("complete deployment SQLite schema migration transcript should validate");

        assert_eq!(
            report.status,
            RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::ReadyForExternalReview
        );
        assert_eq!(report.pre_migration_schema_version, 0);
        assert_eq!(report.post_migration_schema_version, 1);
        assert_eq!(report.expected_schema_version, 1);
        assert!(report.deployment_host_evidence);
        assert!(report.service_lifecycle_reference_present);
        assert!(report.pre_migration_backup_reference_present);
        assert!(report.migration_execution_reference_present);
        assert!(report.schema_version_transition_validated);
        assert!(report.sqlite_integrity_check_passed);
        assert!(report.sqlite_checkpoint_reopened);
        assert!(report.audit_replay_after_migration_validated);
        assert!(report.rollback_reference_present);
        assert!(report.runtime_quiesced_or_degraded);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.migration_executed_by_validator);
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.deployment_path_mutated_by_validator);
        assert!(!report.secrets_loaded);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_sqlite_schema_migration_transcript_blocks_missing_evidence() {
        let report = super::validate_deployment_sqlite_schema_migration_transcript(
            deployment_sqlite_schema_migration_transcript(false),
        )
        .expect("incomplete deployment SQLite schema migration transcript should block");

        assert_eq!(
            report.status,
            RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::Blocked
        );
        assert!(!report.deployment_host_evidence);
        assert!(!report.migration_execution_reference_present);
        assert!(!report.schema_version_transition_validated);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-migration-execution-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "schema-version-mismatch"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn deployment_sqlite_schema_migration_transcript_rejects_validator_side_effects() {
        let mut transcript = deployment_sqlite_schema_migration_transcript(true);
        transcript.migration_executed_by_validator = true;

        let error = super::validate_deployment_sqlite_schema_migration_transcript(transcript)
            .expect_err("validator migration execution must fail closed");

        assert!(error.to_string().contains("must not execute migration"));
    }

    #[test]
    fn runtime_backup_restore_handles_deployment_style_concurrent_load() {
        let audit_path = temp_audit_path("runtime-load-backup-restore");
        let state_path = temp_state_path("runtime-load-backup-restore");
        let backup_audit_path = temp_audit_path("runtime-load-backup-restore-copy");
        let backup_state_path = temp_state_path("runtime-load-backup-restore-copy");
        let workers = 4_usize;
        let barrier = Arc::new(Barrier::new(workers));
        let open_lock = Arc::new(Mutex::new(()));
        let handles = (0..workers)
            .map(|worker| {
                let audit_path = audit_path.clone();
                let state_path = state_path.clone();
                let policy = policy();
                let barrier = Arc::clone(&barrier);
                let open_lock = Arc::clone(&open_lock);
                thread::spawn(move || {
                    let mut request = request(&policy);
                    request.id = format!("runtime-load-worker-{worker}");
                    request.adapter_request_id = format!("adapter-load-worker-{worker}");
                    request.plan.id = format!("plan-load-worker-{worker}");
                    request.now_unix_ms = 70_000 + u64::try_from(worker).unwrap_or(u64::MAX);

                    let (mut journal, mut store) = {
                        let _guard = open_lock.lock().expect("open lock should not be poisoned");
                        (
                            AppendOnlyAuditJournal::open(&audit_path).expect("journal opens"),
                            SqliteWalStateStore::open(&state_path).expect("sqlite store opens"),
                        )
                    };
                    barrier.wait();

                    let record =
                        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
                            .expect("runtime lifecycle should complete");
                    assert_eq!(
                        record.status,
                        RuntimeLifecycleStatus::AdapterRunCheckpointed
                    );
                    assert!(!record.external_submission_performed);
                    assert!(!record.live_execution_performed);
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().expect("worker should not panic");
        }

        let backup_report = validate_local_runtime_backup_restore(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
        )
        .expect("runtime backup/restore validation should pass after concurrent load");
        assert!(backup_report.audit_restore_check_passed);
        assert!(backup_report.sqlite_restore_check_passed);
        assert!(backup_report.plan_checkpoint_restored);
        assert!(backup_report.adapter_checkpoint_restored);
        assert!(backup_report.adapter_recovery_plan_checkpoint_restored);
        assert!(!backup_report.external_submission_performed);
        assert!(!backup_report.live_execution_performed);
        assert!(!backup_report.production_ready);

        let expected_replayed = u64::try_from(workers)
            .expect("worker count should fit u64")
            .saturating_mul(4);
        assert!(backup_report.audit_records_replayed >= expected_replayed);

        let restart_report = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect("runtime restart");
        assert!(restart_report.audit_replay_check_passed);
        assert!(restart_report.sqlite_reopen_check_passed);
        assert!(restart_report.plan_checkpoint_recovered);
        assert!(restart_report.adapter_checkpoint_recovered);
        assert!(restart_report.adapter_recovery_plan_checkpoint_recovered);
        assert!(!restart_report.external_submission_performed);
        assert!(!restart_report.live_execution_performed);
        assert!(!restart_report.production_ready);

        let backup_journal =
            AppendOnlyAuditJournal::open(&backup_audit_path).expect("backup journal reopens");
        assert_eq!(
            backup_journal.next_sequence(),
            backup_report.audit_records_replayed + 1
        );

        let reopened_state = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        reopened_state
            .integrity_check()
            .expect("sqlite integrity check should pass after deployment-style load");
        assert!(reopened_state
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_some());
        assert!(reopened_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_some());
        assert!(reopened_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
            .expect("adapter recovery-plan checkpoint reads")
            .is_some());

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
    }

    #[allow(clippy::too_many_lines)]
    fn valid_runtime_smoke_report(
        audit_records_replayed: u64,
        trace_candidates: u64,
    ) -> RuntimeDeploymentSmokeValidationReport {
        RuntimeDeploymentSmokeValidationReport {
            validation_version: RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION.to_owned(),
            lifecycle_completed: true,
            graceful_shutdown_checkpointed: true,
            backup_restore_validated: true,
            restart_recovery_validated: true,
            audit_durability_validated: true,
            concurrent_lifecycle_validated: true,
            concurrent_lifecycle_workers: 3,
            concurrent_lifecycle_audit_records_replayed: 16,
            concurrent_lifecycle_sqlite_integrity_check_passed: true,
            concurrent_lifecycle_external_submission_performed: false,
            concurrent_lifecycle_live_execution_performed: false,
            observability_collected: true,
            observability_checkpoint_recovered: true,
            observability_operations_reviewed: true,
            observability_operations_checkpoint_recovered: true,
            observability_export_dry_run_rendered: true,
            observability_export_checkpoint_recovered: true,
            observability_alert_route_dispatched: true,
            observability_alert_route_checkpoint_recovered: true,
            observability_endpoint_preflighted: true,
            observability_endpoint_checkpoint_recovered: true,
            observability_loopback_bind_validated: true,
            observability_loopback_bind_checkpoint_recovered: true,
            observability_metrics_scrape_preflighted: true,
            observability_metrics_scrape_checkpoint_recovered: true,
            observability_metrics_endpoint_validated: true,
            observability_metrics_endpoint_checkpoint_recovered: true,
            observability_tracing_captured: true,
            observability_tracing_checkpoint_recovered: true,
            observability_metrics_endpoint_started: false,
            observability_local_metrics_request_served: true,
            observability_public_network_exposed: false,
            observability_outbound_alerts_sent: false,
            observability_telemetry_exported: false,
            observability_production_ready: false,
            communications_command_routed: true,
            communications_command_route_checkpoint_recovered: true,
            communications_remote_command_reviewed: true,
            communications_remote_command_review_checkpoint_recovered: true,
            communications_platform_command_ingress_validated: true,
            communications_platform_command_ingress_checkpoint_recovered: true,
            communications_remote_command_envelope_validated: true,
            communications_remote_command_envelope_checkpoint_recovered: true,
            communications_channel_adapter_validated: true,
            communications_channel_adapter_checkpoint_recovered: true,
            communications_channel_session_validated: true,
            communications_channel_session_checkpoint_recovered: true,
            communications_platform_adapter_reviewed: true,
            communications_platform_adapter_checkpoint_recovered: true,
            communications_notification_dispatched: true,
            communications_notification_checkpoint_recovered: true,
            communications_execution_enabled: false,
            communications_remote_commands_enabled: false,
            communications_outbound_network_used: false,
            dashboard_rendered: true,
            dashboard_checkpoint_recovered: true,
            dashboard_hosted_security_reviewed: true,
            dashboard_hosted_security_checkpoint_recovered: true,
            dashboard_hosted_request_preflighted: true,
            dashboard_hosted_request_preflight_checkpoint_recovered: true,
            dashboard_hosted_request_validated: true,
            dashboard_hosted_request_validation_checkpoint_recovered: true,
            dashboard_panel_count: 3,
            dashboard_server_started: false,
            dashboard_local_one_shot_request_served: true,
            dashboard_public_network_exposed: false,
            dashboard_live_controls_enabled: false,
            dashboard_hosted_production_ready: false,
            validation_run_recorded: true,
            validation_run_checkpoint_recovered: true,
            validation_property_checks_passed: true,
            validation_property_checkpoint_recovered: true,
            validation_external_fuzzer_invoked: false,
            validation_live_network_used: false,
            validation_live_execution_submitted: false,
            validation_signing_or_broadcast_performed: false,
            paper_ledger_applicable: true,
            paper_execution_report_checkpointed: true,
            paper_execution_report_checkpoint_recovered: true,
            paper_ledger_checkpointed: true,
            paper_ledger_checkpoint_recovered: true,
            paper_modeled_fills_settled: 2,
            paper_ledger_audit_records_appended: 8,
            paper_ledger_replay_validated: true,
            paper_ledger_external_submission_performed: false,
            paper_ledger_live_execution_performed: false,
            failure_capture_validated: true,
            failure_capture_checkpoint_recovered: true,
            failure_capture_metrics_endpoint_started: false,
            failure_capture_public_network_exposed: false,
            failure_capture_outbound_alerts_sent: false,
            failure_capture_external_submission_performed: false,
            failure_capture_live_execution_performed: false,
            restart_audit_records_replayed: audit_records_replayed,
            backup_audit_records_replayed: audit_records_replayed,
            restart_plan_checkpoint_recovered: true,
            restart_adapter_checkpoint_recovered: true,
            restart_adapter_recovery_plan_checkpoint_recovered: true,
            restart_graceful_shutdown_checkpoint_recovered: true,
            restart_opportunity_trace_recovery_validated: true,
            restart_opportunity_trace_discovered_candidates: trace_candidates,
            restart_opportunity_trace_recovered_checkpoints: trace_candidates,
            restart_opportunity_trace_recovered_summaries: (0..trace_candidates)
                .map(|index| RuntimeRecoveredOpportunityTraceSummary {
                    trace_id: format!("local-smoke-load-trace-{index}"),
                    strategy_id: "local-smoke-load-strategy".to_owned(),
                    planner_request_id: format!("local-smoke-load-planner-{index}"),
                    audit_sequence: index.saturating_add(1),
                    traced_at_unix_ms: 70_000 + index,
                    route_kind: "CexCex".to_owned(),
                    leg_count: 2,
                })
                .collect(),
            restart_opportunity_trace_missing_checkpoints: 0,
            opportunity_trace_recovery: Some(RuntimeOpportunityTraceRecoverySummary {
                corpus_id: "local-smoke-load-test-corpus".to_owned(),
                discovered_candidates: trace_candidates,
                audit_trace_records_replayed: trace_candidates,
                recovered_trace_checkpoints: trace_candidates,
                missing_trace_checkpoints: 0,
                recovered_trace_summaries: (0..trace_candidates)
                    .map(|index| RuntimeRecoveredOpportunityTraceSummary {
                        trace_id: format!("local-smoke-load-trace-{index}"),
                        strategy_id: "local-smoke-load-strategy".to_owned(),
                        planner_request_id: format!("local-smoke-load-planner-{index}"),
                        audit_sequence: index.saturating_add(1),
                        traced_at_unix_ms: 70_000 + index,
                        route_kind: "CexCex".to_owned(),
                        leg_count: 2,
                    })
                    .collect(),
                trace_recovery_validated: true,
            }),
            recovery_disposition: RuntimeRestartRecoveryDisposition::ReadyForLocalReview,
            service_manager_action_performed: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready: false,
            unresolved_blockers: super::runtime_deployment_smoke_unresolved_blockers(),
        }
    }

    fn service_manager_lifecycle_transcript(
        complete: bool,
    ) -> RuntimeServiceManagerLifecycleTranscript {
        let mut events = vec![
            service_manager_lifecycle_event(
                "unit-loaded",
                RuntimeServiceManagerLifecycleEventKind::UnitLoaded,
                91_000,
                true,
            ),
            service_manager_lifecycle_event(
                "started",
                RuntimeServiceManagerLifecycleEventKind::Started,
                91_100,
                true,
            ),
            service_manager_lifecycle_event(
                "runtime-smoke",
                RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed,
                91_200,
                true,
            ),
            service_manager_lifecycle_event(
                "shutdown",
                RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
                91_300,
                true,
            ),
            service_manager_lifecycle_event(
                "stopped",
                RuntimeServiceManagerLifecycleEventKind::Stopped,
                91_400,
                true,
            ),
            service_manager_lifecycle_event(
                "restarted",
                RuntimeServiceManagerLifecycleEventKind::Restarted,
                91_500,
                true,
            ),
            service_manager_lifecycle_event(
                "recovery",
                RuntimeServiceManagerLifecycleEventKind::RecoveryValidated,
                91_600,
                true,
            ),
        ];
        if !complete {
            events.retain(|event| {
                event.kind != RuntimeServiceManagerLifecycleEventKind::Restarted
                    && event.kind != RuntimeServiceManagerLifecycleEventKind::RecoveryValidated
            });
        }
        RuntimeServiceManagerLifecycleTranscript {
            transcript_id: "service-manager-lifecycle-test".to_owned(),
            service_manager: RuntimeServiceManagerKind::Systemd,
            unit_name: "arb-agent.service".to_owned(),
            events,
            audit_replay_reference_present: complete,
            sqlite_recovery_reference_present: complete,
            runtime_smoke_reference_present: complete,
            concurrent_lifecycle_reference_present: complete,
            concurrent_lifecycle_worker_count: if complete { 3 } else { 0 },
            concurrent_lifecycle_success: complete,
            operator_approved: complete,
            operator_lifecycle_rehearsal_reference_present: complete,
            emergency_stop_review_reference_present: complete,
            rollback_plan_review_reference_present: complete,
            operator_review_window_current: complete,
            service_manager_action_performed_by_validator: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 92_000,
        }
    }

    fn service_manager_lifecycle_rehearsal(
        complete: bool,
    ) -> RuntimeServiceManagerLifecycleRehearsalRequest {
        let mut events = vec![
            service_manager_lifecycle_event(
                "rehearsal-unit-loaded",
                RuntimeServiceManagerLifecycleEventKind::UnitLoaded,
                93_000,
                true,
            ),
            service_manager_lifecycle_event(
                "rehearsal-started",
                RuntimeServiceManagerLifecycleEventKind::Started,
                93_100,
                true,
            ),
            service_manager_lifecycle_event(
                "rehearsal-runtime-smoke",
                RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed,
                93_200,
                true,
            ),
            service_manager_lifecycle_event(
                "rehearsal-shutdown",
                RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
                93_300,
                true,
            ),
            service_manager_lifecycle_event(
                "rehearsal-stopped",
                RuntimeServiceManagerLifecycleEventKind::Stopped,
                93_400,
                true,
            ),
            service_manager_lifecycle_event(
                "rehearsal-restarted",
                RuntimeServiceManagerLifecycleEventKind::Restarted,
                93_500,
                true,
            ),
            service_manager_lifecycle_event(
                "rehearsal-recovery",
                RuntimeServiceManagerLifecycleEventKind::RecoveryValidated,
                93_600,
                true,
            ),
        ];
        if !complete {
            events.retain(|event| {
                event.kind != RuntimeServiceManagerLifecycleEventKind::Restarted
                    && event.kind != RuntimeServiceManagerLifecycleEventKind::RecoveryValidated
            });
        }
        RuntimeServiceManagerLifecycleRehearsalRequest {
            rehearsal_id: if complete {
                "service-manager-lifecycle-rehearsal-ready".to_owned()
            } else {
                "service-manager-lifecycle-rehearsal-blocked".to_owned()
            },
            service_manager: RuntimeServiceManagerKind::Systemd,
            unit_name: "arb-agent.service".to_owned(),
            events,
            audit_replay_reference_present: complete,
            sqlite_recovery_reference_present: complete,
            runtime_smoke_reference_present: complete,
            concurrent_lifecycle_reference_present: complete,
            concurrent_lifecycle_worker_count: if complete { 3 } else { 0 },
            concurrent_lifecycle_success: complete,
            graceful_shutdown_checkpoint_reference_present: complete,
            restart_recovery_reference_present: complete,
            operator_approved: complete,
            reviewer_approved: complete,
            service_manager_action_performed_by_validator: false,
            deployment_path_mutated_by_validator: false,
            secrets_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 94_000,
        }
    }

    fn service_manager_lifecycle_event(
        event_id: &str,
        kind: RuntimeServiceManagerLifecycleEventKind,
        observed_at_unix_ms: u64,
        complete: bool,
    ) -> RuntimeServiceManagerLifecycleEvent {
        RuntimeServiceManagerLifecycleEvent {
            event_id: event_id.to_owned(),
            kind,
            observed_at_unix_ms,
            operator_controlled: true,
            non_secret_reference_present: complete,
            outcome_success: complete,
        }
    }

    fn deployment_permission_transcript(complete: bool) -> RuntimeDeploymentPermissionTranscript {
        RuntimeDeploymentPermissionTranscript {
            transcript_id: if complete {
                "deployment-permission-ready".to_owned()
            } else {
                "deployment-permission-blocked".to_owned()
            },
            host_label: "deployment-host-a".to_owned(),
            deployment_host_evidence: complete,
            runtime_write_attempt_reference_present: complete,
            runtime_write_permission_denied: complete,
            runtime_write_error_classified: complete,
            audit_write_failed_closed: true,
            state_write_failed_closed: complete,
            adapter_evaluation_blocked: complete,
            runtime_quiesced_or_degraded: complete,
            audit_replay_after_restore_validated: complete,
            sqlite_reopen_after_restore_validated: complete,
            recovery_runbook_reference_present: complete,
            non_secret_reference_count: if complete { 9 } else { 1 },
            operator_approved: complete,
            permission_changed_by_validator: false,
            production_path_mutated_by_validator: false,
            service_manager_action_performed_by_validator: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 95_000,
        }
    }

    fn deployment_audit_sqlite_transcript(
        complete: bool,
    ) -> RuntimeDeploymentAuditSqliteTranscript {
        RuntimeDeploymentAuditSqliteTranscript {
            transcript_id: if complete {
                "deployment-audit-sqlite-ready".to_owned()
            } else {
                "deployment-audit-sqlite-blocked".to_owned()
            },
            host_label: "deployment-host-a".to_owned(),
            deployment_host_evidence: complete,
            service_lifecycle_reference_present: complete,
            audit_append_reference_present: complete,
            audit_replay_validated: complete,
            audit_hash_chain_validated: complete,
            sqlite_wal_mode_validated: complete,
            sqlite_integrity_check_passed: complete,
            sqlite_checkpoint_recovered: complete,
            backup_restore_validated: complete,
            concurrent_access_validated: complete,
            recovery_runbook_reference_present: complete,
            non_secret_reference_count: if complete { 9 } else { 1 },
            operator_approved: complete,
            reviewer_approved: complete,
            service_manager_action_performed_by_validator: false,
            deployment_path_mutated_by_validator: false,
            secrets_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 99_000,
        }
    }

    fn deployment_backup_restore_transcript(
        complete: bool,
    ) -> RuntimeDeploymentBackupRestoreTranscript {
        RuntimeDeploymentBackupRestoreTranscript {
            transcript_id: if complete {
                "deployment-backup-restore-ready".to_owned()
            } else {
                "deployment-backup-restore-blocked".to_owned()
            },
            host_label: "deployment-host-a".to_owned(),
            deployment_host_evidence: complete,
            service_lifecycle_reference_present: complete,
            backup_artifact_reference_present: complete,
            restore_execution_reference_present: complete,
            deployment_load_reference_present: complete,
            audit_replay_after_restore_validated: complete,
            audit_hash_chain_after_restore_validated: complete,
            sqlite_integrity_after_restore_validated: complete,
            sqlite_checkpoint_after_restore_validated: complete,
            runtime_checkpoint_restore_validated: complete,
            post_restore_runtime_smoke_passed: complete,
            rollback_reference_present: complete,
            recovery_runbook_reference_present: complete,
            non_secret_reference_count: if complete { 10 } else { 1 },
            operator_approved: complete,
            reviewer_approved: complete,
            backup_restore_executed_by_validator: false,
            service_manager_action_performed_by_validator: false,
            deployment_path_mutated_by_validator: false,
            secrets_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 99_500,
        }
    }

    fn deployment_graceful_shutdown_transcript(
        complete: bool,
    ) -> RuntimeDeploymentGracefulShutdownTranscript {
        RuntimeDeploymentGracefulShutdownTranscript {
            transcript_id: if complete {
                "deployment-graceful-shutdown-ready".to_owned()
            } else {
                "deployment-graceful-shutdown-blocked".to_owned()
            },
            host_label: "deployment-host-a".to_owned(),
            deployment_host_evidence: complete,
            service_lifecycle_reference_present: complete,
            shutdown_request_reference_present: complete,
            service_stopped_reference_present: complete,
            graceful_shutdown_checkpoint_reference_present: complete,
            audit_replay_after_shutdown_validated: complete,
            sqlite_reopen_after_shutdown_validated: complete,
            restart_recovery_after_shutdown_validated: complete,
            post_shutdown_runtime_smoke_passed: complete,
            operator_approved: complete,
            reviewer_approved: complete,
            non_secret_reference_count: if complete { 9 } else { 1 },
            service_manager_action_performed_by_validator: false,
            deployment_path_mutated_by_validator: false,
            secrets_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 99_750,
        }
    }

    fn deployment_sqlite_schema_migration_transcript(
        complete: bool,
    ) -> RuntimeDeploymentSqliteSchemaMigrationTranscript {
        RuntimeDeploymentSqliteSchemaMigrationTranscript {
            transcript_id: if complete {
                "deployment-sqlite-schema-migration-ready".to_owned()
            } else {
                "deployment-sqlite-schema-migration-blocked".to_owned()
            },
            host_label: "deployment-host-a".to_owned(),
            deployment_host_evidence: complete,
            service_lifecycle_reference_present: complete,
            pre_migration_schema_version: 0,
            post_migration_schema_version: i64::from(complete),
            expected_schema_version: 1,
            pre_migration_backup_reference_present: complete,
            migration_execution_reference_present: complete,
            schema_version_transition_validated: complete,
            sqlite_integrity_check_passed: complete,
            sqlite_checkpoint_reopened: complete,
            audit_replay_after_migration_validated: complete,
            rollback_reference_present: complete,
            runtime_quiesced_or_degraded: complete,
            non_secret_reference_count: if complete { 9 } else { 1 },
            operator_approved: complete,
            reviewer_approved: complete,
            migration_executed_by_validator: false,
            service_manager_action_performed_by_validator: false,
            deployment_path_mutated_by_validator: false,
            secrets_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 96_000,
        }
    }

    fn request(policy: &PolicyEngine) -> RuntimeLifecycleRequest {
        RuntimeLifecycleRequest {
            id: "runtime-lifecycle-1".to_owned(),
            adapter_request_id: "adapter-request-1".to_owned(),
            plan: planner_plan(policy),
            adapter_config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        }
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

    fn duplicate_candidate_trace_corpus() -> OpportunityHistoricalFixtureCorpus {
        let pair = MarketPair::new("BTC", "USD").expect("pair should validate");
        OpportunityHistoricalFixtureCorpus {
            id: "runtime-dedup-trace-corpus".to_owned(),
            historical_fixture_replay: true,
            replay_windows: vec![OpportunityReplayCorpus {
                id: "runtime-dedup-trace-window-1".to_owned(),
                scenarios: vec![OpportunityReplayScenario {
                    id: "runtime-dedup-trace-scenario-1".to_owned(),
                    request: OpportunityDiscoveryRequest {
                        id: "runtime-dedup-request".to_owned(),
                        quotes: vec![
                            runtime_trace_quote(
                                "buy-a-1",
                                "paper-a",
                                pair.clone(),
                                98.0,
                                99.0,
                                1.0,
                            ),
                            runtime_trace_quote(
                                "buy-a-2",
                                "paper-a",
                                pair.clone(),
                                97.0,
                                98.0,
                                1.0,
                            ),
                            runtime_trace_quote(
                                "sell-b",
                                "paper-b",
                                pair.clone(),
                                110.0,
                                111.0,
                                1.0,
                            ),
                        ],
                        fee_schedules: vec![
                            runtime_trace_fee("paper-a", pair.clone()),
                            runtime_trace_fee("paper-b", pair),
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

    fn temp_audit_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-runtime-{label}-{}-{nanos}.jsonl",
            process::id()
        ));
        path
    }

    fn temp_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-runtime-{label}-{}-{nanos}.sqlite3",
            process::id()
        ));
        path
    }

    fn temp_workspace_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-runtime-{label}-{}-{nanos}",
            process::id()
        ));
        path
    }

    fn runtime_trace_quote(
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

    fn runtime_trace_fee(venue_name: &str, pair: MarketPair) -> FeeSchedule {
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

    fn cleanup_state_files(path: &std::path::Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", path.display())));
        }
    }

    #[derive(Default)]
    struct PermissionDeniedStateStore {
        put_attempts: usize,
    }

    impl StateStore for PermissionDeniedStateStore {
        fn put_checkpoint(&mut self, _checkpoint: StateCheckpoint) -> Result<(), StateStoreError> {
            self.put_attempts += 1;
            Err(StateStoreError::BackendFailed {
                reason: "simulated permission-denied state path".to_owned(),
            })
        }

        fn get_checkpoint(&self, _key: &str) -> Result<Option<StateCheckpoint>, StateStoreError> {
            Ok(None)
        }
    }
}
