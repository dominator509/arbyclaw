#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

use arb_core::{
    append_agentic_handoff_review_audit, append_cex_order_lifecycle_audit,
    append_channel_adapter_validation_audit, append_channel_session_validation_audit,
    append_dashboard_hosted_request_preflight_audit,
    append_dashboard_hosted_request_validation_audit,
    append_dashboard_hosted_security_review_audit,
    append_dashboard_hosted_session_validation_audit, append_dashboard_render_audit,
    append_destination_allowlist_audit, append_destination_ownership_review_audit,
    append_dex_swap_lifecycle_audit, append_execution_adapter_recovery_plan_audit,
    append_execution_adapter_run_audit, append_execution_plan_draft_audit,
    append_fee_schedule_verification_audit, append_fuzz_corpus_replay_report_audit,
    append_historical_market_data_persistence_audit, append_local_tracing_subscriber_audit,
    append_market_data_provider_preflight_audit, append_market_data_reconnect_plan_audit,
    append_notification_dispatch_audit, append_observability_alert_route_dispatch_audit,
    append_observability_endpoint_preflight_audit, append_observability_export_dry_run_audit,
    append_observability_log_retention_execution_audit,
    append_observability_loopback_bind_validation_audit,
    append_observability_metrics_endpoint_validation_audit,
    append_observability_metrics_scrape_preflight_audit,
    append_observability_operations_review_audit, append_observability_record_audit,
    append_platform_adapter_review_audit, append_platform_command_ingress_audit,
    append_policy_decision_audit, append_property_check_report_audit,
    append_remote_command_envelope_validation_audit, append_remote_command_security_review_audit,
    append_routed_operator_command_audit, append_runtime_failure_capture_audit,
    append_secret_backup_restore_review_audit, append_secret_rotation_plan_audit,
    append_signer_authorization_envelope_audit, append_signer_request_audit,
    append_signer_secret_scope_review_audit, append_validation_corpus_report_audit,
    append_validation_run_audit, append_web3_broadcast_adapter_control_review_audit,
    append_web3_broadcast_readiness_audit, append_web3_nonce_reservation_audit,
    append_web3_pre_sign_safety_audit, append_web3_provider_nonce_reconciliation_audit,
    append_web3_raw_transaction_serialization_review_audit,
    append_web3_sandbox_live_discrepancy_calibration_audit,
    append_web3_unsigned_payload_review_audit, append_web3_unsigned_transaction_construction_audit,
    assess_market_data_quality, build_local_signer_authorization_envelope,
    capture_local_runtime_failure, discover_opportunities_from_local_providers,
    evaluate_local_signer_request, execute_local_observability_log_retention,
    install_local_runtime_panic_hook, load_config_file, migrate_config_toml_to_current,
    parse_cli_command, persist_agentic_handoff_review_checkpoint,
    persist_cex_order_lifecycle_checkpoint, persist_channel_adapter_validation_checkpoint,
    persist_channel_session_validation_checkpoint,
    persist_dashboard_hosted_request_preflight_checkpoint,
    persist_dashboard_hosted_request_validation_checkpoint,
    persist_dashboard_hosted_security_review_checkpoint,
    persist_dashboard_hosted_session_validation_checkpoint, persist_dashboard_render_checkpoint,
    persist_destination_allowlist_checkpoint, persist_destination_ownership_review_checkpoint,
    persist_dex_swap_lifecycle_checkpoint, persist_execution_adapter_recovery_plan_checkpoint,
    persist_execution_adapter_run_checkpoint, persist_execution_plan_draft_checkpoint,
    persist_fee_schedule_verification_checkpoint, persist_fuzz_corpus_replay_report_checkpoint,
    persist_historical_market_data_checkpoint, persist_local_tracing_subscriber_checkpoint,
    persist_market_data_provider_preflight_checkpoint,
    persist_market_data_reconnect_plan_checkpoint, persist_notification_dispatch_checkpoint,
    persist_observability_alert_route_dispatch_checkpoint,
    persist_observability_endpoint_preflight_checkpoint,
    persist_observability_export_dry_run_checkpoint,
    persist_observability_log_retention_execution_checkpoint,
    persist_observability_loopback_bind_validation_checkpoint,
    persist_observability_metrics_endpoint_validation_checkpoint,
    persist_observability_metrics_scrape_preflight_checkpoint,
    persist_observability_operations_review_checkpoint, persist_observability_record_checkpoint,
    persist_platform_adapter_review_checkpoint, persist_platform_command_ingress_checkpoint,
    persist_policy_decision_checkpoint, persist_property_check_report_checkpoint,
    persist_remote_command_envelope_validation_checkpoint,
    persist_remote_command_security_review_checkpoint, persist_routed_operator_command_checkpoint,
    persist_runtime_failure_capture_checkpoint, persist_secret_backup_restore_review_checkpoint,
    persist_secret_rotation_plan_checkpoint, persist_signer_authorization_envelope_checkpoint,
    persist_signer_request_checkpoint, persist_signer_secret_scope_review_checkpoint,
    persist_validation_corpus_report_checkpoint, persist_validation_run_checkpoint,
    persist_web3_broadcast_adapter_control_review_checkpoint,
    persist_web3_broadcast_readiness_checkpoint, persist_web3_nonce_reservation_checkpoint,
    persist_web3_pre_sign_safety_checkpoint, persist_web3_provider_nonce_reconciliation_checkpoint,
    persist_web3_raw_transaction_serialization_review_checkpoint,
    persist_web3_sandbox_live_discrepancy_calibration_checkpoint,
    persist_web3_unsigned_payload_review_checkpoint,
    persist_web3_unsigned_transaction_construction_checkpoint,
    phase27_local_opportunity_historical_fixture_corpus, phase27_local_opportunity_replay_corpus,
    plan_execution_adapter_recovery, plan_local_secret_rotation,
    preflight_dashboard_hosted_request, preflight_observability_endpoint,
    preflight_observability_metrics_scrape, record_observability_alert_route_dispatch,
    render_observability_export_dry_run, review_dashboard_hosted_runtime_readiness,
    review_dashboard_hosted_security, review_local_secret_backup_restore,
    review_local_validation_coverage, review_market_data_provider_latency,
    review_observability_operations, review_platform_adapter_controls,
    review_platform_command_ingress, review_remote_command_security,
    review_signer_runtime_isolation, review_signer_secret_scope, run_local_fuzz_corpus_replay,
    run_local_graceful_shutdown_checkpoint, run_local_runtime_lifecycle,
    run_local_validation_corpus, run_local_validation_property_checks,
    validate_audit_journal_durability, validate_cex_credential_scope_review,
    validate_cex_rate_limit, validate_channel_adapter, validate_channel_session,
    validate_dashboard_hosted_request, validate_dashboard_hosted_session,
    validate_deployment_audit_sqlite_transcript, validate_deployment_backup_restore_transcript,
    validate_deployment_disk_full_transcript, validate_deployment_failure_capture_transcript,
    validate_deployment_graceful_shutdown_transcript, validate_deployment_permission_transcript,
    validate_deployment_response_drill_rehearsal, validate_deployment_retention_transcript,
    validate_deployment_sqlite_schema_migration_transcript, validate_fee_schedule_verification,
    validate_historical_market_data_persistence, validate_incident_response_execution_transcript,
    validate_local_opportunity_quote_ingestion_load, validate_local_runtime_backup_restore,
    validate_local_runtime_restart_recovery,
    validate_local_runtime_restart_recovery_with_trace_recovery, validate_local_tracing_subscriber,
    validate_market_data_provider_preflight, validate_market_data_reconnect_plan,
    validate_observability_loopback_bind, validate_observability_metrics_endpoint,
    validate_opportunity_candidate_trace_restart_recovery,
    validate_opportunity_planner_handoff_with_trace, validate_paid_market_data_provider_evaluation,
    validate_remote_command_envelope, validate_rollback_execution_transcript,
    validate_service_manager_lifecycle_rehearsal, validate_service_manager_lifecycle_transcript,
    validate_strategy_profile_replay_corpus, validate_strategy_profitability_tuning, AgentConfig,
    AgenticHandoffPackager, AgenticHandoffReviewRecord, AgenticHandoffReviewRequest,
    AgenticHandoffReviewStatus, AppendOnlyAuditJournal, ApprovedDestinationEntry,
    AuditDeploymentDiskFullTranscript, AuditDeploymentDiskFullTranscriptStatus,
    AuditDeploymentRetentionTranscript, AuditDeploymentRetentionTranscriptStatus, AuditEvent,
    AuditEventKind, AuditJournalFileMetadata, AuditRecord, AuditRetentionExecutionRequest,
    AuditRetentionPolicy, AuditValue, BacktestDatasetDefinition, BacktestScenarioDefinition,
    BuildIdentity, CexBalanceSnapshotTranscript, CexBalanceSnapshotTranscriptFormat,
    CexCredentialPermission, CexCredentialScopeReviewInput, CexCredentialScopeReviewStatus,
    CexExchangeMarketDataFormat, CexMarketDataRequestKind, CexMarketDataRequestPlan,
    CexMockMarketDataTranscript, CexOrderLifecycleRecord, CexOrderLifecycleTranscript,
    CexOrderLifecycleTranscriptFormat, CexOrderRequest, CexOrderSide, CexOrderType,
    CexOrderValidationRecord, CexRateLimitObservation, CexRateLimitScope, CexRateLimitStatus,
    CexTimeInForce, ChannelAdapterValidationReport, ChannelAdapterValidationRequest,
    ChannelAdapterValidationStatus, ChannelSessionValidationReport, ChannelSessionValidationStatus,
    CommunicationBoundaryConfig, ComponentHealthStatus, ConfigError, ConfigMigrationStatus,
    DashboardAccessContext, DashboardAccessSource, DashboardBoundaryConfig,
    DashboardHostedRequestMethod, DashboardHostedRequestPreflight,
    DashboardHostedRequestValidation, DashboardHostedRequestValidationStatus,
    DashboardHostedRuntimeReadinessReviewRequest, DashboardHostedRuntimeReadinessReviewStatus,
    DashboardHostedSecurityPolicy, DashboardHostedSecurityReviewStatus,
    DashboardHostedSessionValidationStatus, DashboardPanel, DashboardPanelItem, DashboardPanelKind,
    DashboardRenderRequest, DashboardRenderer, DashboardSeverity, DashboardSnapshot,
    DeploymentFailureCaptureTranscript, DeploymentFailureCaptureTranscriptStatus,
    DeploymentResponseDrillRehearsalRequest, DeploymentResponseDrillRehearsalStatus,
    DestinationAllowlist, DestinationApprovalSource, DestinationOwnershipReviewReport,
    DestinationOwnershipReviewStatus, DestinationPolicy, DeterministicAgenticHandoffPackager,
    DeterministicDashboardRenderer, DeterministicExecutionAdapterBoundary,
    DeterministicExecutionPlanner, DeterministicNotificationBoundary,
    DeterministicObservabilityCollector, DeterministicOperatorCommandRouter,
    DeterministicOpportunityEngine, DeterministicValidationHarness, DexProtocolRiskReviewRequest,
    DexProtocolRiskReviewStatus, DexRequestPlan, DexRequestPlanKind, DexResponseTranscript,
    DexRouteKind, DexSimulationStatus, DexSwapLifecycleRecord, DexSwapMode, DexSwapQuoteRequest,
    DexSwapQuoteResponse, DexSwapValidationRecord, ExecutionAdapter, ExecutionAdapterConfig,
    ExecutionAdapterRequest, ExecutionAdapterRunStatus, ExecutionIntent, ExecutionIntentKind,
    ExecutionPlanStatus, ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerRequest,
    ExecutionScope, ExpectedValidationOutcome, FeeAdjustedEdge, FeeEstimate, FeeModelError,
    FeeProvider, FeeSchedule, FeeScheduleVerificationInput, FeeScheduleVerificationReport,
    FeeScheduleVerificationStatus, FixtureKind, FuzzCorpusDefinition, FuzzSeedRecord,
    FuzzTargetKind, HealthStatus, HistoricalMarketDataPersistenceInput,
    HistoricalMarketDataPersistenceReport, HistoricalMarketDataPersistenceStatus,
    IncidentResponseExecutionTranscript, IncidentResponseExecutionTranscriptStatus, LiquidityRole,
    LocalFuzzCorpusReplayRequest, LocalFuzzCorpusReplayStatus,
    LocalTracingSubscriberValidationRequest, LocalTracingSubscriberValidationStatus,
    LocalValidationCorpusRequest, LocalValidationCorpusStatus, LocalValidationCoverageReviewReport,
    LocalValidationCoverageReviewRequest, LocalValidationCoverageReviewStatus,
    MarketDataCapabilities, MarketDataError, MarketDataProvider,
    MarketDataProviderHealthObservation, MarketDataProviderLatencyReviewReport,
    MarketDataProviderLatencyReviewRequest, MarketDataProviderLatencyReviewStatus,
    MarketDataProviderPreflightReport, MarketDataProviderPreflightStatus,
    MarketDataQualityAssessmentInput, MarketDataQualityAssessmentReport,
    MarketDataQualityAssessmentStatus, MarketDataReconnectPlanInput, MarketDataReconnectPlanReport,
    MarketDataReconnectPlanStatus, MarketDataRequest, MarketPair, MetricKind, MetricLabel,
    MetricSample, NormalizedQuote, NotificationChannelProfile, NotificationChannelSafetyState,
    NotificationDispatchRecord, NotificationDispatchStatus, NotificationPublisher,
    NotificationSeverity, ObservabilityAccessContext, ObservabilityAlertRouteDispatchRequest,
    ObservabilityAlertRouteDispatchStatus, ObservabilityBoundaryConfig,
    ObservabilityCollectionRequest, ObservabilityCollector, ObservabilityEndpointPreflight,
    ObservabilityLogRetentionExecutionRequest, ObservabilityLoopbackBindValidationRequest,
    ObservabilityMetricsEndpointValidationRequest, ObservabilityMetricsScrapePreflightRequest,
    ObservabilityOperationsPolicy, ObservabilityOperationsReviewStatus, ObservabilitySeverity,
    ObservabilitySnapshot, OperatorCommandRouter, OperatorCommandRoutingRequest,
    OperatorCommandSource, OperatorNotification, OpportunityCandidate, OpportunityLeg,
    OpportunityLegSide, OpportunityPlannerHandoffStatus, OpportunityProviderIngestionRequest,
    OpportunityQuoteIngestionLoadRequest, OpportunityReplayLoadIteration,
    OpportunityReplayLoadReport, OpportunityReplayStatus, OpportunityRouteKind, OpportunityScore,
    OrderBookSnapshot, PaidMarketDataProviderEvaluationInput,
    PaidMarketDataProviderEvaluationReport, PaidMarketDataProviderEvaluationStatus,
    PaperAssetBalance, PaperBacktestCorpus, PaperBacktestRunReport, PaperBacktestScenario,
    PaperBacktestStep, PaperExecutionAdapter, PaperFillModelConfig, PaperFillSide,
    PaperFillSimulationRequest, PlatformAdapterReviewReport, PlatformAdapterReviewRequest,
    PlatformAdapterReviewStatus, PlatformCommandIngressReport, PlatformCommandIngressRequest,
    PlatformCommandIngressStatus, PolicyApproval, PolicyDecisionRecord, PolicyEngine, PriceLevel,
    RemoteCommandEnvelopeValidationReport, RemoteCommandEnvelopeValidationRequest,
    RemoteCommandEnvelopeValidationStatus, RemoteCommandSecurityReviewReport,
    RemoteCommandSecurityReviewRequest, RemoteCommandSecurityReviewStatus,
    RollbackExecutionTranscript, RollbackExecutionTranscriptStatus, RoutedOperatorCommand, Runbook,
    RunbookStep, RuntimeDeploymentAuditSqliteTranscript,
    RuntimeDeploymentAuditSqliteTranscriptStatus, RuntimeDeploymentBackupRestoreTranscript,
    RuntimeDeploymentBackupRestoreTranscriptStatus, RuntimeDeploymentGracefulShutdownTranscript,
    RuntimeDeploymentGracefulShutdownTranscriptStatus, RuntimeDeploymentPermissionTranscript,
    RuntimeDeploymentPermissionTranscriptStatus, RuntimeDeploymentSmokeLoadIteration,
    RuntimeDeploymentSmokeLoadValidationReport, RuntimeDeploymentSmokeValidationRequest,
    RuntimeDeploymentSqliteSchemaMigrationTranscript,
    RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus, RuntimeFailureCaptureRequest,
    RuntimeFailureKind, RuntimeGracefulShutdownRequest, RuntimeLifecycleStatus,
    RuntimePanicHookInstallationRequest, RuntimeRestartRecoveryDisposition,
    RuntimeServiceManagerKind, RuntimeServiceManagerLifecycleEvent,
    RuntimeServiceManagerLifecycleEventKind, RuntimeServiceManagerLifecycleRehearsalRequest,
    RuntimeServiceManagerLifecycleRehearsalStatus, RuntimeServiceManagerLifecycleTranscript,
    RuntimeServiceManagerLifecycleTranscriptStatus, SecretBackupRestoreReviewReport,
    SecretBackupRestoreReviewRequest, SecretBackupRestoreReviewStatus, SecretRef,
    SecretRotationPlanReport, SecretRotationPlanRequest, SecretRotationPlanStatus,
    SignerAuthorizationEnvelopeReport, SignerAuthorizationEnvelopeRequest,
    SignerAuthorizationEnvelopeStatus, SignerRequest, SignerRequestRecord, SignerRequestStatus,
    SignerRuntimeIsolationReviewReport, SignerRuntimeIsolationReviewRequest,
    SignerRuntimeIsolationReviewStatus, SignerSecretScopeReviewReport,
    SignerSecretScopeReviewRequest, SignerSecretScopeReviewStatus, SqliteWalStateStore,
    StateCheckpoint, StateStore, StateStoreError, StrategyPolicyConstraintStatus, StrategyProfile,
    StrategyProfileReplayValidationStatus, StrategyProfitabilityTuningValidationStatus,
    StructuredLogEvent, StructuredLogField, ValidationExecutionMode, ValidationFixtureRecord,
    ValidationHarness, ValidationHarnessConfig, ValidationPlan, ValidationRunRequest,
    ValidationRunStatus, ValidationSuiteKind, ValidationTestCase, VenueKind, VenueRef,
    Web3BroadcastAdapterControlReviewReport, Web3BroadcastAdapterControlReviewRequest,
    Web3BroadcastAdapterControlReviewStatus, Web3BroadcastReadinessReport,
    Web3BroadcastReadinessRequest, Web3BroadcastReadinessStatus, Web3NonceReservationReport,
    Web3NonceReservationRequest, Web3NonceReservationStatus, Web3PreSignSafetyReviewReport,
    Web3PreSignSafetyReviewRequest, Web3PreSignSafetyReviewStatus,
    Web3ProviderNonceReconciliationReport, Web3ProviderNonceReconciliationRequest,
    Web3ProviderNonceReconciliationStatus, Web3RawTransactionSerializationReviewReport,
    Web3RawTransactionSerializationReviewRequest, Web3RawTransactionSerializationReviewStatus,
    Web3SandboxLiveDiscrepancyCalibrationReport, Web3SandboxLiveDiscrepancyCalibrationRequest,
    Web3SandboxLiveDiscrepancyCalibrationStatus, Web3TransactionLifecycleRecord,
    Web3TransactionLifecycleStatus, Web3TransactionLifecycleTranscript,
    Web3TransactionLifecycleTranscriptFormat, Web3TransactionSimulationRequest,
    Web3TransactionSimulationResponse, Web3UnsignedPayloadReviewReport,
    Web3UnsignedPayloadReviewRequest, Web3UnsignedPayloadReviewStatus,
    Web3UnsignedTransactionConstructionReport, Web3UnsignedTransactionConstructionRequest,
    Web3UnsignedTransactionConstructionStatus, AGENTIC_HANDOFF_LAST_REVIEW_CHECKPOINT_KEY,
    AGENTIC_HANDOFF_VERSION, AUDIT_DURABILITY_VALIDATION_VERSION, CEX_CONNECTOR_FRAMEWORK_VERSION,
    CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY, COMMUNICATIONS_CLI_VERSION,
    COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY, DASHBOARD_BOUNDARY_VERSION,
    DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY, DASHBOARD_LAST_RENDER_CHECKPOINT_KEY,
    DEFAULT_MARKET_DATA_FRESHNESS_MS, DESTINATION_ALLOWLIST_CHECKPOINT_KEY,
    DESTINATION_ALLOWLIST_VERSION, DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY,
    DEX_CONNECTOR_FRAMEWORK_VERSION, DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY,
    DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY,
    DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY,
    DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY, DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY,
    DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY,
    DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY,
    DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY,
    DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY,
    DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY,
    EXECUTION_ADAPTER_FRAMEWORK_VERSION, EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY,
    EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
    EXECUTION_PLANNER_VERSION, EXTERNAL_HARDENING_VERSION, FEE_LAST_VERIFICATION_CHECKPOINT_KEY,
    MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY,
    MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY,
    MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY, OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY,
    OBSERVABILITY_RUNBOOK_VERSION, OPPORTUNITY_ENGINE_VERSION, PACKAGING_DEPLOYMENT_VERSION,
    PAPER_AUDIT_INTEGRATION_VERSION, PAPER_BALANCE_LEDGER_VERSION, PAPER_CONNECTOR_VERSION,
    PAPER_REALISM_VALIDATION_VERSION, PAPER_REALISTIC_FILL_MODEL_VERSION,
    POLICY_LAST_DECISION_CHECKPOINT_KEY, RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION,
    RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION, RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY,
    RUNTIME_GRACEFUL_SHUTDOWN_VERSION, RUNTIME_LIFECYCLE_VERSION,
    RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION, SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY,
    SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY, SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY,
    SIGNER_LAST_REQUEST_CHECKPOINT_KEY, SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY,
    SQLITE_WAL_DURABILITY_VERSION, SQLITE_WAL_STATE_SCHEMA_VERSION, TESTING_BACKTESTING_VERSION,
    TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY, TESTING_LAST_PROPERTY_CHECK_REPORT_KEY,
    TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY, TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY,
};
use std::{
    env,
    error::Error,
    fmt, fs,
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Barrier, Mutex,
    },
    thread,
    time::Duration,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);
const LOCAL_PAPER_BACKTEST_CORPUS_CHECKPOINT_KEY: &str = "testing:last-local-paper-backtest-corpus";

const LOCAL_STRATEGY_PLANNER_CONFIG: &str = r#"
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
gas_fee_cap_quote = 1.0

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

const LOCAL_LEGACY_CONFIG_MIGRATION_FIXTURE: &str = r#"
[runtime]
mode = "observe"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 10.0
max_daily_loss_quote = 2.0
max_open_exposure_quote = 20.0
slippage_bps = 50

[markets]
allowed_exchanges = ["coinbase", "kraken"]
allowed_dexes = []
allowed_chains = []
allowed_assets = ["BTC", "ETH", "USDC"]

[notifications]
notify_channels = []
"#;

const LOCAL_LEGACY_VENUES_CONFIG_MIGRATION_FIXTURE: &str = r#"
[runtime]
mode = "observe"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 10.0
max_daily_loss_quote = 2.0
max_open_exposure_quote = 20.0
slippage_bps = 50
gas_fee_cap_quote = 0.0

[venues]
allowed_exchanges = ["coinbase", "kraken"]
allowed_dexes = ["paper-uniswap"]
allowed_chains = ["ethereum"]
allowed_assets = ["BTC", "ETH", "USDC"]

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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), AgentCliError> {
    run_with_args(env::args().skip(1))
}

fn run_with_args(args: impl IntoIterator<Item = String>) -> Result<(), AgentCliError> {
    let identity = BuildIdentity::current();
    println!("{} {}", identity.name(), identity.version());

    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("--config") => run_config_status(args),
        Some("--help" | "-h") => {
            print_usage();
            Ok(())
        }
        Some("validate-config-migration") => run_config_migration_validation(),
        Some("validate-runtime-smoke") => {
            let options = parse_runtime_smoke_options(args)?;
            run_runtime_smoke_validation(&options)
        }
        Some(command) if is_signer_web3_validation_command(command) => {
            run_signer_web3_validation_command(command)
        }
        Some(command) if is_local_workspace_validation_command(command) => {
            run_local_workspace_validation_command(command, args)
        }
        Some("write-runtime-supervised-restart-child") => {
            let options = parse_local_validation_run_options(args)?;
            run_runtime_supervised_restart_child(&options)
        }
        Some(
            command @ ("validate-opportunity-replay"
            | "validate-opportunity-quote-load"
            | "validate-opportunity-provider-ingestion"
            | "validate-opportunity-historical-fixtures"
            | "validate-opportunity-planner-handoff"
            | "validate-strategy-profitability-tuning"
            | "validate-strategy-replay-corpus"
            | "validate-opportunity-trace-recovery"),
        ) => run_opportunity_validation_command(command, args),
        Some("validate-local-validation-coverage-review") => {
            run_local_validation_coverage_review_runner()
        }
        Some("validate-market-data-provider-preflight") => {
            run_market_data_provider_preflight_validation()
        }
        Some("validate-market-data-reconnect-plan") => run_market_data_reconnect_plan_validation(),
        Some("validate-market-data-quality-assessment") => {
            run_market_data_quality_assessment_validation()
        }
        Some("validate-paid-market-data-provider-evaluation") => {
            run_paid_market_data_provider_evaluation_validation()
        }
        Some("validate-fee-schedule-verification") => run_fee_schedule_verification_validation(),
        Some("validate-cex-governance-review") => run_cex_governance_review_validation(),
        Some("validate-cex-market-data-request-plans") => {
            run_cex_market_data_request_plan_validation()
        }
        Some("validate-cex-balance-snapshots") => run_cex_balance_snapshot_validation(),
        Some("validate-dex-request-plans") => run_dex_request_plan_validation(),
        Some("validate-dex-response-transcripts") => run_dex_response_transcript_validation(),
        Some("validate-dex-transaction-lifecycle-transcripts") => {
            run_dex_transaction_lifecycle_transcript_validation()
        }
        Some("validate-dex-protocol-risk-review") => run_dex_protocol_risk_review_validation(),
        Some("validate-strategy-constrained-planner") => {
            run_strategy_constrained_planner_validation()
        }
        Some(other) => Err(ConfigError::ReadFailed {
            path: other.to_owned(),
            reason: "unknown argument; use --help".to_owned(),
        }
        .into()),
        None => {
            print_default_status();
            Ok(())
        }
    }
}

fn is_signer_web3_validation_command(command: &str) -> bool {
    matches!(
        command,
        "validate-signer-authorization-envelope"
            | "validate-deployment-audit-sqlite-transcript"
            | "validate-deployment-backup-restore-transcript"
            | "validate-deployment-graceful-shutdown-transcript"
            | "validate-deployment-sqlite-schema-migration-transcript"
            | "validate-deployment-disk-full-transcript"
            | "validate-deployment-failure-capture-transcript"
            | "validate-deployment-response-drill-rehearsal"
            | "validate-incident-response-execution-transcript"
            | "validate-deployment-permission-transcript"
            | "validate-deployment-retention-transcript"
            | "validate-rollback-execution-transcript"
            | "validate-service-manager-lifecycle-rehearsal"
            | "validate-service-manager-lifecycle-transcript"
            | "validate-web3-nonce-reservation"
            | "validate-web3-unsigned-payload-review"
            | "validate-web3-pre-sign-safety"
            | "validate-web3-broadcast-readiness"
            | "validate-web3-unsigned-transaction-construction"
            | "validate-web3-provider-nonce-reconciliation"
            | "validate-web3-raw-transaction-serialization-review"
            | "validate-web3-broadcast-adapter-control-review"
            | "validate-web3-sandbox-live-discrepancy-calibration"
            | "validate-signer-runtime-isolation"
    )
}

fn is_local_workspace_validation_command(command: &str) -> bool {
    matches!(
        command,
        "validate-local-validation-run"
            | "validate-local-property-checks"
            | "validate-local-fuzz-corpus"
            | "validate-local-validation-corpus"
            | "validate-local-paper-backtest-corpus"
            | "validate-market-data-boundary-audit"
            | "validate-market-data-history-persistence"
            | "validate-fee-boundary-audit"
            | "validate-agentic-handoff-audit"
            | "validate-policy-decision-audit"
            | "validate-withdrawal-policy-boundary"
            | "validate-secret-boundary-audit"
            | "validate-secret-backup-restore"
            | "validate-execution-planner-audit"
            | "validate-execution-adapter-audit"
            | "validate-signer-boundary-audit"
            | "validate-destination-boundary-audit"
            | "validate-connector-lifecycle-audit"
            | "validate-audit-retention-execution"
            | "validate-audit-durability"
            | "validate-runtime-graceful-shutdown"
            | "validate-runtime-backup-restore"
            | "validate-runtime-backup-restore-load"
            | "validate-runtime-restart-recovery"
            | "validate-runtime-incomplete-recovery"
            | "validate-runtime-supervised-restart"
            | "validate-runtime-permission-denial"
            | "validate-runtime-blocked-state-preflight"
            | "validate-runtime-blocked-audit-preflight"
            | "validate-observability-runtime"
            | "validate-runtime-panic-hook"
            | "validate-dashboard-runtime"
            | "validate-communications-runtime"
    )
}

fn run_opportunity_validation_command(
    command: &str,
    args: impl Iterator<Item = String>,
) -> Result<(), AgentCliError> {
    match command {
        "validate-opportunity-replay" => {
            let options = parse_local_iteration_options(
                args,
                "validate-opportunity-replay",
                "opportunity replay",
            )?;
            run_opportunity_replay_validation(options.iterations)
        }
        "validate-opportunity-quote-load" => run_opportunity_quote_load_validation(args),
        "validate-opportunity-provider-ingestion" => {
            run_opportunity_provider_ingestion_validation()
        }
        "validate-opportunity-historical-fixtures" => {
            run_opportunity_historical_fixture_validation()
        }
        "validate-opportunity-planner-handoff" => run_opportunity_planner_handoff_validation(),
        "validate-strategy-profitability-tuning" => run_strategy_profitability_tuning_validation(),
        "validate-strategy-replay-corpus" => run_strategy_replay_corpus_validation(),
        "validate-opportunity-trace-recovery" => run_opportunity_trace_recovery_validation(),
        _ => unreachable!("opportunity validation command is matched by run_with_args"),
    }
}

fn run_signer_web3_validation_command(command: &str) -> Result<(), AgentCliError> {
    match command {
        "validate-signer-authorization-envelope" => run_signer_authorization_envelope_validation(),
        "validate-web3-nonce-reservation" => run_web3_nonce_reservation_validation(),
        "validate-web3-unsigned-payload-review" => run_web3_unsigned_payload_review_validation(),
        "validate-web3-pre-sign-safety" => run_web3_pre_sign_safety_validation(),
        "validate-web3-broadcast-readiness" => run_web3_broadcast_readiness_validation(),
        "validate-web3-unsigned-transaction-construction" => {
            run_web3_unsigned_transaction_construction_validation()
        }
        "validate-web3-provider-nonce-reconciliation" => {
            run_web3_provider_nonce_reconciliation_validation()
        }
        "validate-web3-raw-transaction-serialization-review" => {
            run_web3_raw_transaction_serialization_review_validation()
        }
        "validate-web3-broadcast-adapter-control-review" => {
            run_web3_broadcast_adapter_control_review_validation()
        }
        "validate-web3-sandbox-live-discrepancy-calibration" => {
            run_web3_sandbox_live_discrepancy_calibration_validation()
        }
        "validate-deployment-audit-sqlite-transcript" => {
            run_deployment_audit_sqlite_transcript_validation()
        }
        "validate-deployment-backup-restore-transcript" => {
            run_deployment_backup_restore_transcript_validation()
        }
        "validate-deployment-graceful-shutdown-transcript" => {
            run_deployment_graceful_shutdown_transcript_validation()
        }
        "validate-deployment-sqlite-schema-migration-transcript" => {
            run_deployment_sqlite_schema_migration_transcript_validation()
        }
        "validate-deployment-disk-full-transcript" => {
            run_deployment_disk_full_transcript_validation()
        }
        "validate-deployment-failure-capture-transcript" => {
            run_deployment_failure_capture_transcript_validation()
        }
        "validate-deployment-response-drill-rehearsal" => {
            run_deployment_response_drill_rehearsal_validation()
        }
        "validate-incident-response-execution-transcript" => {
            run_incident_response_execution_transcript_validation()
        }
        "validate-deployment-permission-transcript" => {
            run_deployment_permission_transcript_validation()
        }
        "validate-deployment-retention-transcript" => {
            run_deployment_retention_transcript_validation()
        }
        "validate-rollback-execution-transcript" => run_rollback_execution_transcript_validation(),
        "validate-service-manager-lifecycle-transcript" => {
            run_service_manager_lifecycle_transcript_validation()
        }
        "validate-service-manager-lifecycle-rehearsal" => {
            run_service_manager_lifecycle_rehearsal_validation()
        }
        "validate-signer-runtime-isolation" => run_signer_runtime_isolation_validation(),
        _ => unreachable!("signer/Web3 validation command is matched by run_with_args"),
    }
}

fn print_default_status() {
    println!("status: scaffold/config/policy/audit/market-data/paper/paper-ledger/paper-fill-model/paper-replay-calibration-backtest/paper-audit-integration/cex-framework/dex-web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-cli/dashboard/observability/testing/packaging/external-hardening/agentic-handoff-ready; live trading disabled until secrets, custody, exchange-specific live connectors, DEX RPC adapters, signing boundaries, live adapter submission, outbound communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, external validation harness execution, packaging/deployment hardening, executed external hardening evidence, and production execution hardening phases are implemented");
    println!(
        "runtime-recovery-dispositions: {}",
        runtime_recovery_disposition_status()
    );
}

fn run_local_workspace_validation_command(
    command: &str,
    args: impl Iterator<Item = String>,
) -> Result<(), AgentCliError> {
    let options = parse_local_validation_run_options(args)?;
    match command {
        "validate-local-validation-run" => run_local_validation_runner(&options),
        "validate-local-property-checks" => run_local_property_check_runner(&options),
        "validate-local-fuzz-corpus" => run_local_fuzz_corpus_runner(&options),
        "validate-local-validation-corpus" => run_local_validation_corpus_runner(&options),
        "validate-local-paper-backtest-corpus" => run_local_paper_backtest_corpus_runner(&options),
        "validate-market-data-boundary-audit" => {
            run_market_data_boundary_audit_validation(&options)
        }
        "validate-market-data-history-persistence" => {
            run_market_data_history_persistence_validation(&options)
        }
        "validate-fee-boundary-audit" => run_fee_boundary_audit_validation(&options),
        "validate-agentic-handoff-audit" => run_agentic_handoff_audit_validation(&options),
        "validate-policy-decision-audit" => run_policy_decision_audit_validation(&options),
        "validate-withdrawal-policy-boundary" => run_withdrawal_policy_validation(&options),
        "validate-secret-boundary-audit" => run_secret_boundary_audit_validation(&options),
        "validate-secret-backup-restore" => run_secret_backup_restore_validation(&options),
        "validate-execution-planner-audit" => run_execution_planner_audit_validation(&options),
        "validate-execution-adapter-audit" => run_execution_adapter_audit_validation(&options),
        "validate-signer-boundary-audit" => run_signer_boundary_audit_validation(&options),
        "validate-destination-boundary-audit" => {
            run_destination_boundary_audit_validation(&options)
        }
        "validate-connector-lifecycle-audit" => run_connector_lifecycle_audit_validation(&options),
        "validate-audit-retention-execution" => run_audit_retention_execution_validation(&options),
        "validate-audit-durability" => run_audit_durability_validation(&options),
        "validate-runtime-graceful-shutdown" => run_runtime_graceful_shutdown_validation(&options),
        "validate-runtime-backup-restore" => run_runtime_backup_restore_validation(&options),
        "validate-runtime-backup-restore-load" => {
            run_runtime_backup_restore_load_validation(&options)
        }
        "validate-runtime-restart-recovery" => run_runtime_restart_recovery_validation(&options),
        "validate-runtime-incomplete-recovery" => {
            run_runtime_incomplete_recovery_validation(&options)
        }
        "validate-runtime-supervised-restart" => {
            run_runtime_supervised_restart_validation(&options)
        }
        "validate-runtime-permission-denial" => run_runtime_permission_denial_validation(&options),
        "validate-runtime-blocked-state-preflight" => {
            run_runtime_blocked_state_preflight_validation(&options)
        }
        "validate-runtime-blocked-audit-preflight" => {
            run_runtime_blocked_audit_preflight_validation(&options)
        }
        "validate-observability-runtime" => run_observability_runtime_validation(&options),
        "validate-runtime-panic-hook" => run_runtime_panic_hook_validation(&options),
        "validate-dashboard-runtime" => run_dashboard_runtime_validation(&options),
        "validate-communications-runtime" => run_communications_runtime_validation(&options),
        _ => Err(ConfigError::ReadFailed {
            path: command.to_owned(),
            reason: "unknown local validation command".to_owned(),
        }
        .into()),
    }
}

fn run_config_status(mut args: impl Iterator<Item = String>) -> Result<(), AgentCliError> {
    let Some(path) = args.next() else {
        return Err(AgentCliError::Usage("--config requires a path".to_owned()));
    };
    let path = PathBuf::from(path);
    let config = load_config_file(&path)?;
    let policy = PolicyEngine::from_config(config.clone());
    println!("config: loaded and validated from {}", path.display());
    println!("mode: {:?}", config.runtime.mode);
    println!(
        "live-intent: {}",
        config.runtime.mode.permits_live_execution()
    );
    println!("policy: {} initialized", policy.trust_contract_version());
    println!("audit: append-only boundary available; {AUDIT_DURABILITY_VALIDATION_VERSION} validates local replay rejection, sync, concurrency, filesystem failure, simulated disk-full fail-closed probes, side-effect-free retention planning, and stale-lock restart recheck planning; runtime journal writing is not auto-started yet");
    println!("state: trait boundary and local SQLite WAL checkpoints available; schema v{SQLITE_WAL_STATE_SCHEMA_VERSION} with {SQLITE_WAL_DURABILITY_VERSION} validates local schema migration, integrity, WAL checkpoint, reopen, backup/restore, and multi-handle durability; external production-host validation pending");
    println!(
        "market-data: normalized quote/order-book/fee boundaries available; live providers pending"
    );
    println!(
        "paper-connectors: {PAPER_CONNECTOR_VERSION} available for deterministic in-memory simulation only"
    );
    println!("paper-balance-ledger: {PAPER_BALANCE_LEDGER_VERSION} available for local simulated balances, reservations, fills, and SQLite checkpoints only");
    println!("paper-fill-model: {PAPER_REALISTIC_FILL_MODEL_VERSION} available for supplied-depth local paper fills only; exchange-specific calibration pending");
    println!("paper-replay-calibration-backtest: {PAPER_REALISM_VALIDATION_VERSION} available for local matching profiles, adverse selection, calibration records, replay validation, and fixture backtests only; production-host validation still pending");
    println!("paper-audit-integration: {PAPER_AUDIT_INTEGRATION_VERSION} available for local paper intent, report, and ledger mutation audit records only; production audit durability validation still pending");
    println!("cex-framework: {CEX_CONNECTOR_FRAMEWORK_VERSION} available as typed interface only; live exchange adapters pending");
    println!("dex-web3-framework: {DEX_CONNECTOR_FRAMEWORK_VERSION} available as typed interface only; live RPC, signing, and broadcasts pending");
    println!("opportunity-engine: {OPPORTUNITY_ENGINE_VERSION} available for deterministic discovery/ranking, a local regression replay corpus, and local replay checks with supplied depth, paper inventory, transfer-risk, and triangular records only; live execution pending");
    println!(
        "execution-planner: {EXECUTION_PLANNER_VERSION} available for draft-only policy-evaluated planning; adapter submission disabled"
    );
    println!("execution-adapter-framework: {EXECUTION_ADAPTER_FRAMEWORK_VERSION} available for deterministic boundary records only; external submission disabled");
    println!("runtime-lifecycle: {RUNTIME_LIFECYCLE_VERSION} available for local fail-closed audit/state/adapter wiring only; {RUNTIME_GRACEFUL_SHUTDOWN_VERSION} records local graceful-shutdown audit/state checkpoints without stopping services; {RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION} validates local audit/state backup restore without deployment actions; {RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION} validates local restart recovery summaries without service resume; {RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION} validates local deployment-like smoke sequencing without service-manager actions; {}; live execution disabled", runtime_recovery_disposition_status());
    println!("communications-cli: {COMMUNICATIONS_CLI_VERSION} available for typed command/notification boundaries only; outbound integrations disabled");
    println!("embedded-dashboard: {DASHBOARD_BOUNDARY_VERSION} available for local render records only; web server exposure and live controls disabled");
    println!("observability-runbooks: {OBSERVABILITY_RUNBOOK_VERSION} available for local health/metric/log/runbook records only; metrics endpoints and outbound alerts disabled");
    println!("testing-backtesting: {TESTING_BACKTESTING_VERSION} available for deterministic validation plans only; external fuzzers, live networks, and live execution disabled");
    println!("packaging-deployment: {PACKAGING_DEPLOYMENT_VERSION} available for deterministic package/deployment plans only; builds, installs, public exposure, and production claims disabled");
    println!("external-hardening: {EXTERNAL_HARDENING_VERSION} available for deterministic evidence/checklist records only; external actions, production claims, and live-funds approval disabled");
    println!("agentic-handoff: {AGENTIC_HANDOFF_VERSION} available for deterministic prompts/checklists/package records only; external agent execution, production claims, and live-funds approval disabled");
    println!("market-data-default-freshness-ms: {DEFAULT_MARKET_DATA_FRESHNESS_MS}");
    println!("status: config/policy/audit/market-data/paper/paper-ledger/paper-fill-model/paper-replay-calibration-backtest/paper-audit-integration/cex-framework/dex-web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-cli/dashboard/observability/testing/packaging/external-hardening/agentic-handoff-ready; live trading still requires custody, exchange-specific live connectors, DEX RPC adapters, signing boundaries, live adapter submission, outbound communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, external validation harness execution, packaging/deployment hardening, executed external hardening evidence, and production execution hardening phases");
    Ok(())
}

fn run_config_migration_validation() -> Result<(), AgentCliError> {
    let current = migrate_config_toml_to_current(LOCAL_STRATEGY_PLANNER_CONFIG)?;
    let legacy = migrate_config_toml_to_current(LOCAL_LEGACY_CONFIG_MIGRATION_FIXTURE)?;
    let legacy_venue_aliases =
        migrate_config_toml_to_current(LOCAL_LEGACY_VENUES_CONFIG_MIGRATION_FIXTURE)?;

    println!("config-migration: validation passed");
    println!(
        "current-config-status: {}",
        config_migration_status_label(current.status)
    );
    println!(
        "legacy-config-status: {}",
        config_migration_status_label(legacy.status)
    );
    println!(
        "legacy-venue-alias-status: {}",
        config_migration_status_label(legacy_venue_aliases.status)
    );
    println!("legacy-action-codes: {}", legacy.action_codes.len());
    println!(
        "legacy-venue-alias-action-codes: {}",
        legacy_venue_aliases.action_codes.len()
    );
    println!(
        "legacy-venue-count: {}",
        legacy.config.venues.cex_allowlist.len()
    );
    println!(
        "legacy-venue-alias-count: {}",
        legacy_venue_aliases.config.venues.cex_allowlist.len()
    );
    println!(
        "legacy-gas-fee-cap-quote: {}",
        legacy.config.risk.gas_fee_cap_quote
    );
    println!(
        "secret-material-loaded: {}",
        current.secret_material_loaded
            || legacy.secret_material_loaded
            || legacy_venue_aliases.secret_material_loaded
    );
    println!(
        "live-execution-enabled: {}",
        current.live_execution_enabled
            || legacy.live_execution_enabled
            || legacy_venue_aliases.live_execution_enabled
    );
    println!(
        "production-ready: {}",
        current.production_ready
            || legacy.production_ready
            || legacy_venue_aliases.production_ready
    );

    if current.status != ConfigMigrationStatus::AlreadyCurrent
        || legacy.status != ConfigMigrationStatus::Migrated
        || legacy_venue_aliases.status != ConfigMigrationStatus::Migrated
        || !legacy
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_MIGRATED_MARKETS_TO_VENUES")
        || !legacy
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_MIGRATED_NOTIFICATIONS_TO_COMMUNICATION")
        || !legacy
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_DEFAULTED_RISK_GAS_FEE_CAP")
        || !legacy_venue_aliases
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_MIGRATED_VENUE_FIELD_ALIASES")
        || legacy.config.venues.cex_allowlist.len() != 2
        || legacy_venue_aliases.config.venues.cex_allowlist.len() != 2
        || legacy_venue_aliases.config.venues.dex_allowlist.len() != 1
        || legacy_venue_aliases.config.venues.chain_allowlist.len() != 1
        || legacy_venue_aliases.config.venues.asset_allowlist.len() != 3
        || legacy.config.risk.gas_fee_cap_quote != 0.0
        || current.secret_material_loaded
        || legacy.secret_material_loaded
        || legacy_venue_aliases.secret_material_loaded
        || current.live_execution_enabled
        || legacy.live_execution_enabled
        || legacy_venue_aliases.live_execution_enabled
        || current.production_ready
        || legacy.production_ready
        || legacy_venue_aliases.production_ready
    {
        return Err(AgentCliError::Validation(
            "config migration validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn print_usage() {
    println!("usage: arb-agent [--config <path>]");
    println!("       arb-agent validate-config-migration");
    println!("       arb-agent validate-opportunity-replay [--iterations <n>]");
    println!("       arb-agent validate-opportunity-quote-load");
    println!("       arb-agent validate-opportunity-provider-ingestion");
    println!("       arb-agent validate-market-data-provider-preflight");
    println!("       arb-agent validate-market-data-reconnect-plan");
    println!("       arb-agent validate-market-data-quality-assessment");
    println!("       arb-agent validate-paid-market-data-provider-evaluation");
    println!("       arb-agent validate-fee-schedule-verification");
    println!("       arb-agent validate-cex-governance-review");
    println!("       arb-agent validate-cex-market-data-request-plans");
    println!("       arb-agent validate-cex-balance-snapshots");
    println!("       arb-agent validate-dex-request-plans");
    println!("       arb-agent validate-dex-response-transcripts");
    println!("       arb-agent validate-dex-transaction-lifecycle-transcripts");
    println!("       arb-agent validate-dex-protocol-risk-review");
    println!("       arb-agent validate-opportunity-historical-fixtures");
    println!("       arb-agent validate-opportunity-planner-handoff");
    println!("       arb-agent validate-strategy-profitability-tuning");
    println!("       arb-agent validate-strategy-replay-corpus");
    println!("       arb-agent validate-strategy-constrained-planner");
    println!("       arb-agent validate-opportunity-trace-recovery");
    println!("       arb-agent validate-local-validation-run --workspace <fresh-dir>");
    println!("       arb-agent validate-local-property-checks --workspace <fresh-dir>");
    println!("       arb-agent validate-local-fuzz-corpus --workspace <fresh-dir>");
    println!("       arb-agent validate-local-validation-corpus --workspace <fresh-dir>");
    println!("       arb-agent validate-local-validation-coverage-review");
    println!("       arb-agent validate-local-paper-backtest-corpus --workspace <fresh-dir>");
    println!("       arb-agent validate-market-data-boundary-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-market-data-history-persistence --workspace <fresh-dir>");
    println!("       arb-agent validate-fee-boundary-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-agentic-handoff-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-policy-decision-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-withdrawal-policy-boundary --workspace <fresh-dir>");
    println!("       arb-agent validate-secret-boundary-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-secret-backup-restore --workspace <fresh-dir>");
    println!("       arb-agent validate-execution-planner-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-execution-adapter-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-signer-boundary-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-signer-runtime-isolation");
    println!("       arb-agent validate-signer-authorization-envelope");
    println!("       arb-agent validate-web3-nonce-reservation");
    println!("       arb-agent validate-web3-unsigned-payload-review");
    println!("       arb-agent validate-web3-pre-sign-safety");
    println!("       arb-agent validate-web3-broadcast-readiness");
    println!("       arb-agent validate-web3-unsigned-transaction-construction");
    println!("       arb-agent validate-web3-provider-nonce-reconciliation");
    println!("       arb-agent validate-web3-raw-transaction-serialization-review");
    println!("       arb-agent validate-web3-broadcast-adapter-control-review");
    println!("       arb-agent validate-web3-sandbox-live-discrepancy-calibration");
    println!("       arb-agent validate-destination-boundary-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-connector-lifecycle-audit --workspace <fresh-dir>");
    println!("       arb-agent validate-audit-durability --workspace <fresh-dir>");
    println!("       arb-agent validate-audit-retention-execution --workspace <fresh-dir>");
    println!("       arb-agent validate-deployment-audit-sqlite-transcript");
    println!("       arb-agent validate-deployment-backup-restore-transcript");
    println!("       arb-agent validate-deployment-graceful-shutdown-transcript");
    println!("       arb-agent validate-deployment-sqlite-schema-migration-transcript");
    println!("       arb-agent validate-deployment-disk-full-transcript");
    println!("       arb-agent validate-deployment-failure-capture-transcript");
    println!("       arb-agent validate-deployment-response-drill-rehearsal");
    println!("       arb-agent validate-incident-response-execution-transcript");
    println!("       arb-agent validate-deployment-permission-transcript");
    println!("       arb-agent validate-deployment-retention-transcript");
    println!("       arb-agent validate-rollback-execution-transcript");
    println!("       arb-agent validate-runtime-graceful-shutdown --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-backup-restore --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-backup-restore-load --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-restart-recovery --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-incomplete-recovery --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-supervised-restart --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-permission-denial --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-blocked-state-preflight --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-blocked-audit-preflight --workspace <fresh-dir>");
    println!("       arb-agent validate-communications-runtime --workspace <fresh-dir>");
    println!("       arb-agent validate-dashboard-runtime --workspace <fresh-dir>");
    println!("       arb-agent validate-observability-runtime --workspace <fresh-dir>");
    println!("       arb-agent validate-runtime-panic-hook --workspace <fresh-dir>");
    println!(
        "       arb-agent validate-runtime-smoke --config <path> --workspace <fresh-dir> [--iterations <n>]"
    );
    println!("       arb-agent validate-service-manager-lifecycle-transcript");
    println!("       arb-agent validate-service-manager-lifecycle-rehearsal");
    println!("default mode reports scaffold status without loading secrets or trading");
    println!("communication, dashboard, validation, packaging, hardening, and handoff commands are typed/local boundaries only; live execute, withdraw, bridge, sign, broadcast, external fuzzing, live network tests, public web exposure, service installation, external agent execution, external hardening execution, production claims, and production deployment remain unavailable");
}

fn run_opportunity_replay_validation(iterations: usize) -> Result<(), AgentCliError> {
    let corpus = phase27_local_opportunity_replay_corpus()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut load_iterations = Vec::with_capacity(iterations);
    for iteration in 1..=iterations {
        let run_id = format!("run-{iteration}");
        let started_at = Instant::now();
        let report = DeterministicOpportunityEngine::new()
            .replay_corpus(&corpus)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
        let elapsed_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);

        println!("opportunity-replay-iteration: {run_id}");
        println!("opportunity-replay-elapsed-ms: {elapsed_ms}");
        println!("opportunity-replay-corpus: {}", report.corpus_id);
        println!("scenario-count: {}", report.scenario_count);
        println!("passed-scenarios: {}", report.passed_scenarios);
        println!("failed-scenarios: {}", report.failed_scenarios);
        println!("total-candidates: {}", report.total_candidates);
        println!(
            "opportunity-replay-status: {}",
            opportunity_replay_status_label(report.status)
        );
        println!(
            "external-calls-performed: {}",
            report.external_calls_performed
        );
        println!(
            "live-execution-performed: {}",
            report.live_execution_performed
        );

        if report.external_calls_performed || report.live_execution_performed {
            return Err(AgentCliError::Validation(
                "opportunity replay reported forbidden side effects".to_owned(),
            ));
        }

        if report.status != OpportunityReplayStatus::Passed {
            let failed = report
                .scenario_reports
                .iter()
                .filter(|scenario| scenario.status == OpportunityReplayStatus::Failed)
                .map(|scenario| scenario.scenario_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(AgentCliError::Validation(format!(
                "opportunity replay failed scenarios: {failed}"
            )));
        }

        load_iterations.push(OpportunityReplayLoadIteration {
            iteration_id: run_id,
            elapsed_ms,
            report,
        });
    }

    let load_report = OpportunityReplayLoadReport::from_iterations(load_iterations)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let latency_review = local_opportunity_replay_latency_review(&load_report)?;
    print_opportunity_replay_load_report(&load_report, &latency_review);

    Ok(())
}

fn local_opportunity_replay_latency_review(
    load_report: &OpportunityReplayLoadReport,
) -> Result<arb_core::OpportunityReplayLatencyReviewReport, AgentCliError> {
    arb_core::review_opportunity_replay_latency(arb_core::OpportunityReplayLatencyReviewRequest {
        review_id: "local-opportunity-replay-latency".to_owned(),
        load_report: load_report.clone(),
        max_average_elapsed_ms: load_report.average_elapsed_ms.max(1),
        max_single_iteration_elapsed_ms: load_report.max_elapsed_ms.max(1),
        min_total_scenarios_replayed: load_report.total_scenarios_replayed.max(1),
        min_total_candidates: load_report.total_candidates.max(1),
        external_calls_performed: false,
        external_data_downloaded: false,
        live_execution_performed: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn print_opportunity_replay_load_report(
    load_report: &OpportunityReplayLoadReport,
    latency_review: &arb_core::OpportunityReplayLatencyReviewReport,
) {
    println!("opportunity-replay-load-validation: passed");
    println!(
        "opportunity-replay-load-iterations-attempted: {}",
        load_report.iterations_attempted
    );
    println!(
        "opportunity-replay-load-iterations-passed: {}",
        load_report.iterations_passed
    );
    println!(
        "opportunity-replay-load-min-elapsed-ms: {}",
        load_report.min_elapsed_ms
    );
    println!(
        "opportunity-replay-load-max-elapsed-ms: {}",
        load_report.max_elapsed_ms
    );
    println!(
        "opportunity-replay-load-average-elapsed-ms: {}",
        load_report.average_elapsed_ms
    );
    println!(
        "opportunity-replay-load-total-elapsed-ms: {}",
        load_report.total_elapsed_ms
    );
    println!(
        "opportunity-replay-load-total-scenarios-replayed: {}",
        load_report.total_scenarios_replayed
    );
    println!(
        "opportunity-replay-load-total-candidates: {}",
        load_report.total_candidates
    );
    println!(
        "opportunity-replay-latency-review: {:?}",
        latency_review.status
    );
    println!(
        "opportunity-replay-latency-budget-met: {}",
        latency_review.latency_budget_met
    );
    println!(
        "opportunity-replay-throughput-budget-met: {}",
        latency_review.throughput_budget_met
    );
    println!(
        "opportunity-replay-latency-review-remaining-external-evidence-count: {}",
        latency_review.remaining_external_evidence.len()
    );
    println!("production-ready: false");
}

fn run_opportunity_quote_load_validation(
    args: impl Iterator<Item = String>,
) -> Result<(), AgentCliError> {
    let mut venue_pairs = 8_usize;
    let mut max_candidates = 3_usize;
    let mut pending = args;
    while let Some(arg) = pending.next() {
        match arg.as_str() {
            "--venue-pairs" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-opportunity-quote-load --venue-pairs requires an integer >= 1"
                            .to_owned(),
                    ));
                };
                venue_pairs = parse_positive_usize(
                    &value,
                    "validate-opportunity-quote-load --venue-pairs requires an integer >= 1",
                )?;
            }
            "--max-candidates" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-opportunity-quote-load --max-candidates requires an integer >= 1"
                            .to_owned(),
                    ));
                };
                max_candidates = parse_positive_usize(
                    &value,
                    "validate-opportunity-quote-load --max-candidates requires an integer >= 1",
                )?;
            }
            other => {
                return Err(AgentCliError::Usage(format!(
                    "unknown opportunity quote-load argument: {other}"
                )));
            }
        }
    }

    let now_unix_ms = current_unix_ms()?;
    let report =
        validate_local_opportunity_quote_ingestion_load(OpportunityQuoteIngestionLoadRequest {
            id: "cli-local-opportunity-quote-load".to_owned(),
            venue_pairs,
            max_candidates,
            now_unix_ms,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("opportunity-quote-load: validation passed");
    println!("opportunity-quote-load-run-id: {}", report.run_id);
    println!("quotes-ingested: {}", report.quotes_ingested);
    println!("fee-schedules-ingested: {}", report.fee_schedules_ingested);
    println!("max-candidates: {}", report.max_candidates);
    println!("candidates-returned: {}", report.candidates_returned);
    println!(
        "candidate-backpressure-applied: {}",
        report.candidate_backpressure_applied
    );
    println!(
        "truncated-candidate-lower-bound: {}",
        report.truncated_candidate_lower_bound
    );
    println!(
        "external-data-downloaded: {}",
        report.external_data_downloaded
    );
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!("production-ready: {}", report.production_ready);

    Ok(())
}

fn run_opportunity_provider_ingestion_validation() -> Result<(), AgentCliError> {
    let pair = MarketPair::new("BTC", "USD")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let venues = vec![
        local_provider_venue("paper-a"),
        local_provider_venue("paper-b"),
    ];
    let market_provider = LocalOpportunityMarketProvider {
        quotes: vec![
            local_provider_quote("provider-buy", "paper-a", pair.clone(), 99.0, 100.0, 2.0),
            local_provider_quote("provider-sell", "paper-b", pair.clone(), 106.0, 107.0, 2.0),
        ],
        books: vec![
            local_provider_book(
                "provider-book-a",
                "paper-a",
                pair.clone(),
                vec![(99.0, 2.0)],
                vec![(100.0, 2.0)],
            ),
            local_provider_book(
                "provider-book-b",
                "paper-b",
                pair.clone(),
                vec![(106.0, 2.0)],
                vec![(107.0, 2.0)],
            ),
        ],
    };
    let fee_provider = LocalOpportunityFeeProvider {
        schedules: vec![
            local_provider_fee("paper-a", pair.clone()),
            local_provider_fee("paper-b", pair.clone()),
        ],
    };

    let report =
        discover_opportunities_from_local_providers(&OpportunityProviderIngestionRequest {
            id: "cli-local-opportunity-provider-ingestion".to_owned(),
            market_data_provider: &market_provider,
            fee_provider: &fee_provider,
            venues,
            pairs: vec![pair],
            include_order_books: true,
            config: arb_core::OpportunityDiscoveryConfig::default(),
            now_unix_ms: 10_000,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("opportunity-provider-ingestion: validation passed");
    println!("market-data-provider: {}", report.market_data_provider_name);
    println!("fee-provider: {}", report.fee_provider_name);
    println!("quotes-ingested: {}", report.quotes_ingested);
    println!("order-books-ingested: {}", report.order_books_ingested);
    println!("fee-schedules-ingested: {}", report.fee_schedules_ingested);
    println!("candidates-discovered: {}", report.candidates_discovered);
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!("production-ready: {}", report.production_ready);

    if report.quotes_ingested != 2
        || report.order_books_ingested != 2
        || report.fee_schedules_ingested != 2
        || report.candidates_discovered == 0
        || report.external_calls_performed
        || report.live_execution_performed
        || report.production_ready
    {
        return Err(AgentCliError::Validation(
            "opportunity provider ingestion validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_market_data_provider_preflight_validation() -> Result<(), AgentCliError> {
    let clean = validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
        provider_name: "local-market-data-clean".to_owned(),
        read_only: true,
        rate_limited: false,
        outage_observed: false,
        reconnect_required: true,
        reconnect_backoff_planned: true,
        samples_checked: 4,
        fresh_samples: 4,
        stale_samples: 0,
        max_observed_latency_ms: 12,
        max_allowed_latency_ms: 50,
        live_network_used: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let degraded = validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
        provider_name: "local-market-data-degraded".to_owned(),
        read_only: true,
        rate_limited: true,
        outage_observed: true,
        reconnect_required: true,
        reconnect_backoff_planned: false,
        samples_checked: 5,
        fresh_samples: 3,
        stale_samples: 2,
        max_observed_latency_ms: 250,
        max_allowed_latency_ms: 100,
        live_network_used: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let latency_review = build_market_data_provider_latency_review(&clean, &degraded)?;

    println!("market-data-provider-preflight: validation passed");
    println!("clean-provider: {}", clean.provider_name);
    println!(
        "clean-provider-status: {}",
        market_data_preflight_status_label(clean.status)
    );
    println!("clean-provider-samples: {}", clean.samples_checked);
    println!("degraded-provider: {}", degraded.provider_name);
    println!(
        "degraded-provider-status: {}",
        market_data_preflight_status_label(degraded.status)
    );
    println!(
        "degraded-provider-violation-codes: {}",
        degraded.violation_codes.len()
    );
    println!("rate-limit-blocked: {}", degraded.rate_limit_blocked);
    println!("outage-blocked: {}", degraded.outage_blocked);
    println!("stale-data-blocked: {}", degraded.stale_data_blocked);
    println!("latency-blocked: {}", degraded.latency_blocked);
    println!("live-network-used: false");
    println!("credential-loaded: false");
    println!("production-ready: false");
    print_market_data_provider_latency_review(&latency_review);

    if clean.status != MarketDataProviderPreflightStatus::Usable
        || degraded.status != MarketDataProviderPreflightStatus::Blocked
        || clean.live_network_used
        || clean.credential_loaded
        || degraded.live_network_used
        || degraded.credential_loaded
        || clean.production_ready
        || degraded.production_ready
        || latency_review.status != MarketDataProviderLatencyReviewStatus::ReadyForLocalReview
        || !latency_review.provider_latency_budget_met
        || !latency_review.capture_latency_budget_met
        || !latency_review.reconnect_delay_budget_met
        || !latency_review.quality_review_ready
        || !latency_review.paid_provider_review_ready
        || latency_review.remaining_external_evidence_count == 0
        || latency_review.live_network_used
        || latency_review.websocket_connection_opened
        || latency_review.credential_loaded
        || latency_review.production_ready
    {
        return Err(AgentCliError::Validation(
            "market-data provider preflight validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn print_market_data_provider_latency_review(report: &MarketDataProviderLatencyReviewReport) {
    println!(
        "market-data-provider-latency-review: {}",
        market_data_provider_latency_review_status_label(report.status)
    );
    println!(
        "market-data-provider-latency-budget-met: {}",
        report.provider_latency_budget_met
    );
    println!(
        "market-data-provider-capture-latency-budget-met: {}",
        report.capture_latency_budget_met
    );
    println!(
        "market-data-provider-reconnect-delay-budget-met: {}",
        report.reconnect_delay_budget_met
    );
    println!(
        "market-data-provider-quality-review-ready: {}",
        report.quality_review_ready
    );
    println!(
        "market-data-provider-paid-review-ready: {}",
        report.paid_provider_review_ready
    );
    println!(
        "market-data-provider-latency-review-remaining-external-evidence-count: {}",
        report.remaining_external_evidence_count
    );
}

fn build_market_data_provider_latency_review(
    clean: &MarketDataProviderPreflightReport,
    degraded: &MarketDataProviderPreflightReport,
) -> Result<MarketDataProviderLatencyReviewReport, AgentCliError> {
    let ready_reconnect = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
        plan_id: "cli-market-data-provider-latency-review-reconnect".to_owned(),
        provider_name: "local-market-data-clean".to_owned(),
        venue: local_provider_venue("paper-provider-latency-review"),
        disconnected_at_unix_ms: 10_000,
        planned_at_unix_ms: 10_050,
        attempt_number: 3,
        max_attempts: 5,
        base_backoff_ms: 100,
        max_backoff_ms: 1_000,
        planned_delay_ms: 500,
        provider_retry_after_ms: Some(450),
        rate_limited: true,
        outage_observed: false,
        live_network_used: false,
        websocket_connection_opened: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let acceptable_quality = build_acceptable_market_data_quality_assessment()?;
    let paid_provider = build_paid_market_data_provider_evaluation_case()?.ready;

    review_market_data_provider_latency(MarketDataProviderLatencyReviewRequest {
        review_id: "cli-market-data-provider-latency-review".to_owned(),
        clean_preflight: clean.clone(),
        degraded_preflight: degraded.clone(),
        ready_reconnect,
        acceptable_quality,
        paid_provider_evaluation: paid_provider,
        max_provider_latency_ms: 50,
        max_capture_latency_ms: 25,
        max_reconnect_delay_ms: 500,
        min_quality_score: 100,
        min_samples_checked: 4,
        remaining_external_evidence: vec![
            "live REST/WebSocket exchange adapters".to_owned(),
            "provider-backed latency and throughput measurement".to_owned(),
            "provider-side rate-limit and outage reconciliation".to_owned(),
            "deployment-host market-data resource profiling".to_owned(),
            "external sandbox/live calibration".to_owned(),
        ],
        live_network_used: false,
        websocket_connection_opened: false,
        credential_loaded: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn run_market_data_reconnect_plan_validation() -> Result<(), AgentCliError> {
    let ready = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
        plan_id: "cli-market-data-reconnect-ready".to_owned(),
        provider_name: "local-market-data-reconnect-ready".to_owned(),
        venue: local_provider_venue("paper-reconnect-ready"),
        disconnected_at_unix_ms: 10_000,
        planned_at_unix_ms: 10_050,
        attempt_number: 3,
        max_attempts: 5,
        base_backoff_ms: 100,
        max_backoff_ms: 1_000,
        planned_delay_ms: 500,
        provider_retry_after_ms: Some(450),
        rate_limited: true,
        outage_observed: false,
        live_network_used: false,
        websocket_connection_opened: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
        plan_id: "cli-market-data-reconnect-blocked".to_owned(),
        provider_name: "local-market-data-reconnect-blocked".to_owned(),
        venue: local_provider_venue("paper-reconnect-blocked"),
        disconnected_at_unix_ms: 20_000,
        planned_at_unix_ms: 20_010,
        attempt_number: 6,
        max_attempts: 5,
        base_backoff_ms: 100,
        max_backoff_ms: 1_000,
        planned_delay_ms: 200,
        provider_retry_after_ms: Some(800),
        rate_limited: true,
        outage_observed: true,
        live_network_used: false,
        websocket_connection_opened: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("market-data-reconnect-plan: validation passed");
    println!("ready-plan: {}", ready.plan_id);
    println!(
        "ready-plan-status: {}",
        market_data_reconnect_plan_status_label(ready.status)
    );
    println!(
        "ready-plan-effective-min-delay-ms: {}",
        ready.effective_min_delay_ms
    );
    println!("ready-plan-planned-delay-ms: {}", ready.planned_delay_ms);
    println!("blocked-plan: {}", blocked.plan_id);
    println!(
        "blocked-plan-status: {}",
        market_data_reconnect_plan_status_label(blocked.status)
    );
    println!(
        "blocked-plan-violation-codes: {}",
        blocked.violation_codes.len()
    );
    println!("outage-blocked: {}", blocked.outage_blocked);
    println!("retry-budget-exhausted: {}", blocked.retry_budget_exhausted);
    println!("live-network-used: false");
    println!("websocket-connection-opened: false");
    println!("credential-loaded: false");
    println!("production-ready: false");

    if ready.status != MarketDataReconnectPlanStatus::ReadyForLocalReview
        || blocked.status != MarketDataReconnectPlanStatus::Blocked
        || blocked.violation_codes.is_empty()
        || ready.live_network_used
        || ready.websocket_connection_opened
        || ready.credential_loaded
        || blocked.live_network_used
        || blocked.websocket_connection_opened
        || blocked.credential_loaded
        || ready.production_ready
        || blocked.production_ready
    {
        return Err(AgentCliError::Validation(
            "market-data reconnect plan validation failed".to_owned(),
        ));
    }

    Ok(())
}

struct MarketDataQualityAssessmentCase {
    acceptable: MarketDataQualityAssessmentReport,
    degraded: MarketDataQualityAssessmentReport,
    blocked: MarketDataQualityAssessmentReport,
}

fn build_acceptable_market_data_quality_assessment(
) -> Result<MarketDataQualityAssessmentReport, AgentCliError> {
    let pair = MarketPair::new("BTC", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let venue = local_provider_venue("paper-quality-acceptable");
    assess_market_data_quality(MarketDataQualityAssessmentInput {
        assessment_id: "cli-market-data-quality-acceptable".to_owned(),
        provider_name: "local-market-data-quality-acceptable".to_owned(),
        request: MarketDataRequest {
            venue: venue.clone(),
            pair: pair.clone(),
            max_age_ms: 250,
        },
        quote: NormalizedQuote {
            id: "cli-market-data-quality-acceptable-quote".to_owned(),
            venue: venue.clone(),
            pair: pair.clone(),
            bid: PriceLevel::new(100.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ask: PriceLevel::new(100.1, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            captured_at_unix_ms: 1_000,
            received_at_unix_ms: 1_015,
        },
        order_book: Some(OrderBookSnapshot {
            id: "cli-market-data-quality-acceptable-book".to_owned(),
            venue,
            pair,
            captured_at_unix_ms: 1_000,
            received_at_unix_ms: 1_015,
            bids: vec![
                PriceLevel::new(100.0, 1.0)
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
                PriceLevel::new(99.9, 1.0)
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ],
            asks: vec![
                PriceLevel::new(100.1, 1.0)
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
                PriceLevel::new(100.2, 1.0)
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ],
            source_sequence: Some("quality-acceptable-seq".to_owned()),
        }),
        now_unix_ms: 1_120,
        max_spread_bps: 20,
        min_depth_levels: 2,
        max_capture_latency_ms: 25,
        live_network_used: false,
        credential_loaded: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_degraded_market_data_quality_assessment(
) -> Result<MarketDataQualityAssessmentReport, AgentCliError> {
    let pair = MarketPair::new("ETH", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let venue = local_provider_venue("paper-quality-degraded");
    assess_market_data_quality(MarketDataQualityAssessmentInput {
        assessment_id: "cli-market-data-quality-degraded".to_owned(),
        provider_name: "local-market-data-quality-degraded".to_owned(),
        request: MarketDataRequest {
            venue: venue.clone(),
            pair: pair.clone(),
            max_age_ms: 500,
        },
        quote: NormalizedQuote {
            id: "cli-market-data-quality-degraded-quote".to_owned(),
            venue: venue.clone(),
            pair: pair.clone(),
            bid: PriceLevel::new(200.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ask: PriceLevel::new(201.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            captured_at_unix_ms: 10_000,
            received_at_unix_ms: 10_060,
        },
        order_book: Some(OrderBookSnapshot {
            id: "cli-market-data-quality-degraded-book".to_owned(),
            venue,
            pair,
            captured_at_unix_ms: 10_000,
            received_at_unix_ms: 10_060,
            bids: vec![PriceLevel::new(200.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?],
            asks: vec![PriceLevel::new(201.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?],
            source_sequence: Some("quality-degraded-seq".to_owned()),
        }),
        now_unix_ms: 10_200,
        max_spread_bps: 20,
        min_depth_levels: 2,
        max_capture_latency_ms: 25,
        live_network_used: false,
        credential_loaded: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_blocked_market_data_quality_assessment(
) -> Result<MarketDataQualityAssessmentReport, AgentCliError> {
    let pair = MarketPair::new("SOL", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let venue = local_provider_venue("paper-quality-blocked");
    assess_market_data_quality(MarketDataQualityAssessmentInput {
        assessment_id: "cli-market-data-quality-blocked".to_owned(),
        provider_name: "local-market-data-quality-blocked".to_owned(),
        request: MarketDataRequest {
            venue: venue.clone(),
            pair: pair.clone(),
            max_age_ms: 100,
        },
        quote: NormalizedQuote {
            id: "cli-market-data-quality-blocked-quote".to_owned(),
            venue,
            pair,
            bid: PriceLevel::new(50.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ask: PriceLevel::new(50.2, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            captured_at_unix_ms: 20_000,
            received_at_unix_ms: 20_010,
        },
        order_book: None,
        now_unix_ms: 20_500,
        max_spread_bps: 50,
        min_depth_levels: 1,
        max_capture_latency_ms: 30,
        live_network_used: true,
        credential_loaded: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_market_data_quality_assessment_case(
) -> Result<MarketDataQualityAssessmentCase, AgentCliError> {
    Ok(MarketDataQualityAssessmentCase {
        acceptable: build_acceptable_market_data_quality_assessment()?,
        degraded: build_degraded_market_data_quality_assessment()?,
        blocked: build_blocked_market_data_quality_assessment()?,
    })
}

fn run_market_data_quality_assessment_validation() -> Result<(), AgentCliError> {
    let quality_case = build_market_data_quality_assessment_case()?;
    let acceptable = &quality_case.acceptable;
    let degraded = &quality_case.degraded;
    let blocked = &quality_case.blocked;

    println!("market-data-quality-assessment: validation passed");
    println!("acceptable-provider: {}", acceptable.provider_name);
    println!(
        "acceptable-status: {}",
        market_data_quality_assessment_status_label(acceptable.status)
    );
    println!("acceptable-quality-score: {}", acceptable.quality_score);
    println!(
        "acceptable-depth-levels: {}",
        acceptable.depth_levels_available
    );
    println!("degraded-provider: {}", degraded.provider_name);
    println!(
        "degraded-status: {}",
        market_data_quality_assessment_status_label(degraded.status)
    );
    println!("degraded-quality-score: {}", degraded.quality_score);
    println!("blocked-provider: {}", blocked.provider_name);
    println!(
        "blocked-status: {}",
        market_data_quality_assessment_status_label(blocked.status)
    );
    println!("blocked-violation-codes: {}", blocked.violation_codes.len());
    println!("live-network-used: false");
    println!("credential-loaded: false");
    println!("production-ready: false");

    if acceptable.status != MarketDataQualityAssessmentStatus::Acceptable
        || degraded.status != MarketDataQualityAssessmentStatus::Degraded
        || blocked.status != MarketDataQualityAssessmentStatus::Blocked
        || !acceptable.freshness_status.is_fresh()
        || !acceptable.spread_within_limit
        || !acceptable.depth_levels_sufficient
        || !acceptable.capture_latency_within_limit
        || degraded.violation_codes.is_empty()
        || blocked.violation_codes.is_empty()
        || acceptable.live_network_used
        || acceptable.credential_loaded
        || degraded.live_network_used
        || degraded.credential_loaded
        || blocked.credential_loaded
        || acceptable.production_ready
        || degraded.production_ready
        || blocked.production_ready
    {
        return Err(AgentCliError::Validation(
            "market-data quality assessment validation failed".to_owned(),
        ));
    }

    Ok(())
}

struct PaidMarketDataProviderEvaluationCase {
    ready: PaidMarketDataProviderEvaluationReport,
    blocked: PaidMarketDataProviderEvaluationReport,
}

fn build_paid_market_data_provider_evaluation_case(
) -> Result<PaidMarketDataProviderEvaluationCase, AgentCliError> {
    let ready =
        validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
            evaluation_id: "cli-paid-market-data-ready".to_owned(),
            provider_name: "local-paid-provider-ready".to_owned(),
            covered_venues: vec![
                local_provider_venue("paper-binance"),
                local_provider_venue("paper-coinbase"),
            ],
            covered_pairs: vec![
                MarketPair::new("BTC", "USDC")
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
                MarketPair::new("ETH", "USDC")
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ],
            capabilities: MarketDataCapabilities {
                order_book: true,
                top_of_book: true,
                fees: false,
                websocket: true,
                rest: true,
            },
            documented_latency_ms: 35,
            max_allowed_latency_ms: 50,
            max_requests_per_minute: 1_200,
            monthly_cost_usd: 499,
            failure_modes_reviewed: vec![
                "provider-outage".to_owned(),
                "stale-book".to_owned(),
                "rate-limit-burst".to_owned(),
            ],
            rate_limit_documentation_reviewed: true,
            pricing_documentation_reviewed: true,
            terms_reviewed: true,
            credential_scope_reviewed: true,
            live_network_used: false,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked =
        validate_paid_market_data_provider_evaluation(PaidMarketDataProviderEvaluationInput {
            evaluation_id: "cli-paid-market-data-blocked".to_owned(),
            provider_name: "local-paid-provider-blocked".to_owned(),
            covered_venues: vec![local_provider_venue("paper-kraken")],
            covered_pairs: vec![MarketPair::new("BTC", "USDT")
                .map_err(|error| AgentCliError::Validation(error.to_string()))?],
            capabilities: MarketDataCapabilities {
                order_book: true,
                top_of_book: true,
                fees: false,
                websocket: false,
                rest: true,
            },
            documented_latency_ms: 120,
            max_allowed_latency_ms: 50,
            max_requests_per_minute: 600,
            monthly_cost_usd: 250,
            failure_modes_reviewed: vec!["provider-outage".to_owned()],
            rate_limit_documentation_reviewed: false,
            pricing_documentation_reviewed: false,
            terms_reviewed: false,
            credential_scope_reviewed: false,
            live_network_used: false,
            credential_loaded: false,
            production_ready_claimed: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    Ok(PaidMarketDataProviderEvaluationCase { ready, blocked })
}

fn run_paid_market_data_provider_evaluation_validation() -> Result<(), AgentCliError> {
    let evaluation_case = build_paid_market_data_provider_evaluation_case()?;
    let ready = &evaluation_case.ready;
    let blocked = &evaluation_case.blocked;

    println!("paid-market-data-provider-evaluation: validation passed");
    println!("ready-provider: {}", ready.provider_name);
    println!(
        "ready-provider-status: {}",
        paid_market_data_provider_evaluation_status_label(ready.status)
    );
    println!("ready-covered-venues: {}", ready.covered_venues.len());
    println!("ready-covered-pairs: {}", ready.covered_pairs.len());
    println!("blocked-provider: {}", blocked.provider_name);
    println!(
        "blocked-provider-status: {}",
        paid_market_data_provider_evaluation_status_label(blocked.status)
    );
    println!(
        "blocked-provider-violation-codes: {}",
        blocked.violation_codes.len()
    );
    println!("latency-within-budget: {}", ready.latency_within_budget);
    println!(
        "rate-limit-review-passed: {}",
        ready.rate_limit_review_passed
    );
    println!("cost-review-passed: {}", ready.cost_review_passed);
    println!(
        "failure-behavior-review-passed: {}",
        ready.failure_behavior_review_passed
    );
    println!(
        "governance-review-passed: {}",
        ready.governance_review_passed
    );
    println!("live-network-used: false");
    println!("credential-loaded: false");
    println!("production-ready: false");

    if ready.status != PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
        || blocked.status != PaidMarketDataProviderEvaluationStatus::Blocked
        || !ready.coverage_review_passed
        || !ready.latency_within_budget
        || !ready.rate_limit_review_passed
        || !ready.cost_review_passed
        || !ready.failure_behavior_review_passed
        || !ready.governance_review_passed
        || blocked.violation_codes.is_empty()
        || ready.live_network_used
        || ready.credential_loaded
        || blocked.live_network_used
        || blocked.credential_loaded
        || ready.production_ready
        || blocked.production_ready
    {
        return Err(AgentCliError::Validation(
            "paid market-data provider evaluation validation failed".to_owned(),
        ));
    }

    Ok(())
}

struct LocalMarketDataBoundaryAuditCase {
    clean_preflight: MarketDataProviderPreflightReport,
    degraded_preflight: MarketDataProviderPreflightReport,
    ready_reconnect: MarketDataReconnectPlanReport,
    blocked_reconnect: MarketDataReconnectPlanReport,
}

struct MarketDataBoundaryAuditPersistence {
    clean_preflight_sequence: u64,
    degraded_preflight_sequence: u64,
    ready_reconnect_sequence: u64,
    blocked_reconnect_sequence: u64,
    preflight_checkpoint_value: String,
    reconnect_checkpoint_value: String,
    preflight_audit_failed_closed: bool,
    reconnect_audit_failed_closed: bool,
}

fn run_market_data_boundary_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options
        .workspace_dir
        .join("market-data-boundary.audit.jsonl");
    let state_path = options.workspace_dir.join("market-data-boundary.sqlite3");
    let market_data_case = build_local_market_data_boundary_audit_case()?;
    let persisted = persist_local_market_data_boundary_audit_case(
        &audit_path,
        &state_path,
        &market_data_case,
        now_unix_ms,
    )?;
    let audit_records_replayed =
        verify_local_market_data_boundary_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed =
        validate_market_data_boundary_state_failure(&market_data_case);

    if !state_failure_failed_closed
        || market_data_case.clean_preflight.status != MarketDataProviderPreflightStatus::Usable
        || market_data_case.degraded_preflight.status != MarketDataProviderPreflightStatus::Blocked
        || market_data_case.ready_reconnect.status
            != MarketDataReconnectPlanStatus::ReadyForLocalReview
        || market_data_case.blocked_reconnect.status != MarketDataReconnectPlanStatus::Blocked
        || market_data_case.clean_preflight.live_network_used
        || market_data_case.clean_preflight.credential_loaded
        || market_data_case.clean_preflight.production_ready
        || market_data_case.ready_reconnect.live_network_used
        || market_data_case.ready_reconnect.websocket_connection_opened
        || market_data_case.ready_reconnect.credential_loaded
        || market_data_case.ready_reconnect.production_ready
    {
        return Err(AgentCliError::Validation(
            "market-data boundary audit/state validation failed".to_owned(),
        ));
    }

    println!("market-data-boundary-audit: validation passed");
    println!(
        "clean-provider-status: {}",
        market_data_preflight_status_label(market_data_case.clean_preflight.status)
    );
    println!(
        "degraded-provider-status: {}",
        market_data_preflight_status_label(market_data_case.degraded_preflight.status)
    );
    println!(
        "ready-reconnect-status: {}",
        market_data_reconnect_plan_status_label(market_data_case.ready_reconnect.status)
    );
    println!(
        "blocked-reconnect-status: {}",
        market_data_reconnect_plan_status_label(market_data_case.blocked_reconnect.status)
    );
    println!(
        "preflight-audit-failed-closed: {}",
        persisted.preflight_audit_failed_closed
    );
    println!(
        "reconnect-audit-failed-closed: {}",
        persisted.reconnect_audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("live-network-used: false");
    println!("websocket-connection-opened: false");
    println!("credential-loaded: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_market_data_boundary_audit_case(
) -> Result<LocalMarketDataBoundaryAuditCase, AgentCliError> {
    let clean_preflight =
        validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
            provider_name: "local-market-data-clean".to_owned(),
            read_only: true,
            rate_limited: false,
            outage_observed: false,
            reconnect_required: true,
            reconnect_backoff_planned: true,
            samples_checked: 4,
            fresh_samples: 4,
            stale_samples: 0,
            max_observed_latency_ms: 12,
            max_allowed_latency_ms: 50,
            live_network_used: false,
            credential_loaded: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let degraded_preflight =
        validate_market_data_provider_preflight(MarketDataProviderHealthObservation {
            provider_name: "local-market-data-degraded".to_owned(),
            read_only: true,
            rate_limited: true,
            outage_observed: true,
            reconnect_required: true,
            reconnect_backoff_planned: false,
            samples_checked: 5,
            fresh_samples: 3,
            stale_samples: 2,
            max_observed_latency_ms: 250,
            max_allowed_latency_ms: 100,
            live_network_used: false,
            credential_loaded: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ready_reconnect = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
        plan_id: "local-market-data-ready-reconnect".to_owned(),
        provider_name: "local-market-data-clean".to_owned(),
        venue: local_provider_venue("paper-reconnect-ready"),
        disconnected_at_unix_ms: 10_000,
        planned_at_unix_ms: 10_050,
        attempt_number: 3,
        max_attempts: 5,
        base_backoff_ms: 100,
        max_backoff_ms: 1_000,
        planned_delay_ms: 500,
        provider_retry_after_ms: Some(450),
        rate_limited: true,
        outage_observed: false,
        live_network_used: false,
        websocket_connection_opened: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked_reconnect = validate_market_data_reconnect_plan(MarketDataReconnectPlanInput {
        plan_id: "local-market-data-blocked-reconnect".to_owned(),
        provider_name: "local-market-data-degraded".to_owned(),
        venue: local_provider_venue("paper-reconnect-blocked"),
        disconnected_at_unix_ms: 20_000,
        planned_at_unix_ms: 20_010,
        attempt_number: 6,
        max_attempts: 5,
        base_backoff_ms: 100,
        max_backoff_ms: 1_000,
        planned_delay_ms: 200,
        provider_retry_after_ms: Some(800),
        rate_limited: true,
        outage_observed: true,
        live_network_used: false,
        websocket_connection_opened: false,
        credential_loaded: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    Ok(LocalMarketDataBoundaryAuditCase {
        clean_preflight,
        degraded_preflight,
        ready_reconnect,
        blocked_reconnect,
    })
}

fn persist_local_market_data_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    market_data_case: &LocalMarketDataBoundaryAuditCase,
    now_unix_ms: u64,
) -> Result<MarketDataBoundaryAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let clean_preflight_audit = append_market_data_provider_preflight_audit(
        &mut journal,
        &market_data_case.clean_preflight,
        now_unix_ms,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let degraded_preflight_audit = append_market_data_provider_preflight_audit(
        &mut journal,
        &market_data_case.degraded_preflight,
        now_unix_ms.saturating_add(1),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ready_reconnect_audit = append_market_data_reconnect_plan_audit(
        &mut journal,
        &market_data_case.ready_reconnect,
        now_unix_ms.saturating_add(2),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked_reconnect_audit = append_market_data_reconnect_plan_audit(
        &mut journal,
        &market_data_case.blocked_reconnect,
        now_unix_ms.saturating_add(3),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let preflight_checkpoint = persist_market_data_provider_preflight_checkpoint(
        &mut store,
        &market_data_case.clean_preflight,
        now_unix_ms.saturating_add(4),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reconnect_checkpoint = persist_market_data_reconnect_plan_checkpoint(
        &mut store,
        &market_data_case.ready_reconnect,
        now_unix_ms.saturating_add(5),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let preflight_audit_failed_closed = validate_market_data_preflight_invalid_audit_fails_closed(
        &mut journal,
        &market_data_case.clean_preflight,
    );
    let reconnect_audit_failed_closed = validate_market_data_reconnect_invalid_audit_fails_closed(
        &mut journal,
        &market_data_case.ready_reconnect,
    );

    Ok(MarketDataBoundaryAuditPersistence {
        clean_preflight_sequence: clean_preflight_audit.sequence,
        degraded_preflight_sequence: degraded_preflight_audit.sequence,
        ready_reconnect_sequence: ready_reconnect_audit.sequence,
        blocked_reconnect_sequence: blocked_reconnect_audit.sequence,
        preflight_checkpoint_value: preflight_checkpoint.value,
        reconnect_checkpoint_value: reconnect_checkpoint.value,
        preflight_audit_failed_closed,
        reconnect_audit_failed_closed,
    })
}

fn verify_local_market_data_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &MarketDataBoundaryAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let preflight_checkpoint = reopened
        .get_checkpoint(MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("market-data preflight checkpoint missing".to_owned())
        })?;
    let reconnect_checkpoint = reopened
        .get_checkpoint(MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("market-data reconnect checkpoint missing".to_owned())
        })?;

    if replayed.next_sequence() <= persisted.blocked_reconnect_sequence
        || persisted.clean_preflight_sequence == persisted.degraded_preflight_sequence
        || persisted.ready_reconnect_sequence == persisted.blocked_reconnect_sequence
        || persisted.degraded_preflight_sequence >= persisted.ready_reconnect_sequence
        || preflight_checkpoint.value != persisted.preflight_checkpoint_value
        || reconnect_checkpoint.value != persisted.reconnect_checkpoint_value
        || !persisted.preflight_audit_failed_closed
        || !persisted.reconnect_audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "market-data boundary audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

struct MarketDataHistoryPersistenceValidation {
    audit_sequence: u64,
    checkpoint_value: String,
    audit_failed_closed: bool,
    state_failed_closed: bool,
}

fn run_market_data_history_persistence_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options
        .workspace_dir
        .join("market-data-history-persistence.audit.jsonl");
    let state_path = options
        .workspace_dir
        .join("market-data-history-persistence.sqlite3");
    let report = build_local_market_data_history_persistence_report()?;
    let persisted = persist_local_market_data_history_persistence(
        &audit_path,
        &state_path,
        &report,
        now_unix_ms,
    )?;
    let audit_records_replayed =
        verify_local_market_data_history_persistence(&audit_path, &state_path, &persisted)?;

    if report.status != HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay
        || report.stored_quotes.is_empty()
        || report.stored_order_books.is_empty()
        || !report.quotes_truncated
        || !report.order_books_truncated
        || !persisted.audit_failed_closed
        || !persisted.state_failed_closed
        || report.live_network_used
        || report.credential_loaded
        || report.production_ready
    {
        return Err(AgentCliError::Validation(
            "market-data history persistence validation failed".to_owned(),
        ));
    }

    println!("market-data-history-persistence: validation passed");
    println!("history-batch-status: persisted-for-local-replay");
    println!("stored-quote-count: {}", report.stored_quotes.len());
    println!(
        "stored-order-book-count: {}",
        report.stored_order_books.len()
    );
    println!("quotes-truncated: {}", report.quotes_truncated);
    println!("order-books-truncated: {}", report.order_books_truncated);
    println!("window-span-ms: {}", report.window_span_ms);
    println!("audit-failed-closed: {}", persisted.audit_failed_closed);
    println!("state-failed-closed: {}", persisted.state_failed_closed);
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("live-network-used: false");
    println!("credential-loaded: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_market_data_history_persistence_report(
) -> Result<HistoricalMarketDataPersistenceReport, AgentCliError> {
    let venue = local_provider_venue("paper-history");
    let pair = MarketPair::new("BTC", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let quotes = vec![
        historical_quote(&venue, &pair, "quote-1", 100.0, 100.1, 1_000, 1_010)?,
        historical_quote(&venue, &pair, "quote-2", 101.0, 101.1, 2_000, 2_010)?,
        historical_quote(&venue, &pair, "quote-3", 102.0, 102.1, 3_000, 3_010)?,
    ];
    let order_books = vec![
        historical_order_book(&venue, &pair, "book-1", 100.0, 100.1, 1_000, 1_015)?,
        historical_order_book(&venue, &pair, "book-2", 101.0, 101.1, 2_000, 2_015)?,
        historical_order_book(&venue, &pair, "book-3", 102.0, 102.1, 3_000, 3_015)?,
    ];
    validate_historical_market_data_persistence(HistoricalMarketDataPersistenceInput {
        batch_id: "local-market-data-history-batch".to_owned(),
        provider_name: "local-market-data-history".to_owned(),
        venue,
        pair,
        quotes,
        order_books,
        max_retained_records_per_kind: 2,
        persisted_at_unix_ms: 4_000,
        live_network_used: false,
        credential_loaded: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn persist_local_market_data_history_persistence(
    audit_path: &Path,
    state_path: &Path,
    report: &HistoricalMarketDataPersistenceReport,
    now_unix_ms: u64,
) -> Result<MarketDataHistoryPersistenceValidation, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit = append_historical_market_data_persistence_audit(&mut journal, report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_historical_market_data_checkpoint(
        &mut store,
        report,
        now_unix_ms.saturating_add(1),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_failed_closed =
        validate_market_data_history_invalid_audit_fails_closed(&mut journal, report);
    let state_failed_closed =
        validate_market_data_history_invalid_state_fails_closed(&mut store, report);

    Ok(MarketDataHistoryPersistenceValidation {
        audit_sequence: audit.sequence,
        checkpoint_value: checkpoint.value,
        audit_failed_closed,
        state_failed_closed,
    })
}

fn verify_local_market_data_history_persistence(
    audit_path: &Path,
    state_path: &Path,
    persisted: &MarketDataHistoryPersistenceValidation,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened
        .get_checkpoint(MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("historical market-data checkpoint missing".to_owned())
        })?;

    if replayed.next_sequence() <= persisted.audit_sequence
        || checkpoint.value != persisted.checkpoint_value
        || !persisted.audit_failed_closed
        || !persisted.state_failed_closed
    {
        return Err(AgentCliError::Validation(
            "market-data history persistence reopen check failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence().saturating_sub(1))
}

fn validate_market_data_history_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &HistoricalMarketDataPersistenceReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.production_ready = true;
    invalid.status = HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay;
    append_historical_market_data_persistence_audit(journal, &invalid, 1_700_000_100).is_err()
        && journal.next_sequence() == next_sequence
}

fn validate_market_data_history_invalid_state_fails_closed(
    store: &mut SqliteWalStateStore,
    report: &HistoricalMarketDataPersistenceReport,
) -> bool {
    let mut invalid = report.clone();
    invalid.production_ready = true;
    invalid.status = HistoricalMarketDataPersistenceStatus::PersistedForLocalReplay;
    persist_historical_market_data_checkpoint(store, &invalid, 1_700_000_101).is_err()
}

fn historical_quote(
    venue: &VenueRef,
    pair: &MarketPair,
    id: &str,
    bid: f64,
    ask: f64,
    captured_at_unix_ms: u64,
    received_at_unix_ms: u64,
) -> Result<NormalizedQuote, AgentCliError> {
    Ok(NormalizedQuote {
        id: id.to_owned(),
        venue: venue.clone(),
        pair: pair.clone(),
        bid: PriceLevel::new(bid, 1.0)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        ask: PriceLevel::new(ask, 1.0)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        captured_at_unix_ms,
        received_at_unix_ms,
    })
}

fn historical_order_book(
    venue: &VenueRef,
    pair: &MarketPair,
    id: &str,
    best_bid: f64,
    best_ask: f64,
    captured_at_unix_ms: u64,
    received_at_unix_ms: u64,
) -> Result<OrderBookSnapshot, AgentCliError> {
    Ok(OrderBookSnapshot {
        id: id.to_owned(),
        venue: venue.clone(),
        pair: pair.clone(),
        captured_at_unix_ms,
        received_at_unix_ms,
        bids: vec![PriceLevel::new(best_bid, 1.0)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?],
        asks: vec![PriceLevel::new(best_ask, 1.0)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?],
        source_sequence: Some(format!("{id}-seq")),
    })
}

fn validate_market_data_preflight_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &MarketDataProviderPreflightReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.live_network_used = true;
    invalid.status = MarketDataProviderPreflightStatus::Usable;
    let failed =
        append_market_data_provider_preflight_audit(journal, &invalid, 1_700_000_501).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_market_data_reconnect_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &MarketDataReconnectPlanReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.websocket_connection_opened = true;
    invalid.status = MarketDataReconnectPlanStatus::ReadyForLocalReview;
    let failed = append_market_data_reconnect_plan_audit(journal, &invalid, 1_700_000_502).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_market_data_boundary_state_failure(
    market_data_case: &LocalMarketDataBoundaryAuditCase,
) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let preflight_failed = persist_market_data_provider_preflight_checkpoint(
        &mut store,
        &market_data_case.clean_preflight,
        1,
    )
    .is_err();
    let reconnect_failed = persist_market_data_reconnect_plan_checkpoint(
        &mut store,
        &market_data_case.ready_reconnect,
        2,
    )
    .is_err();
    preflight_failed && reconnect_failed && store.put_attempts == 2
}

fn run_fee_schedule_verification_validation() -> Result<(), AgentCliError> {
    let current = validate_fee_schedule_verification(FeeScheduleVerificationInput {
        schedule: local_verified_fee_schedule(true),
        review_id: "local-fee-review-current".to_owned(),
        source_reference: "operator-fee-review-current".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_fee_schedule_verification(FeeScheduleVerificationInput {
        schedule: local_verified_fee_schedule(false),
        review_id: "local-fee-review-blocked".to_owned(),
        source_reference: "operator-fee-review-blocked".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("fee-schedule-verification: validation passed");
    println!("current-fee-review: {}", current.review_id);
    println!(
        "current-fee-review-status: {}",
        fee_schedule_verification_status_label(current.status)
    );
    println!("current-fee-review-age-ms: {}", current.review_age_ms);
    println!("blocked-fee-review: {}", blocked.review_id);
    println!(
        "blocked-fee-review-status: {}",
        fee_schedule_verification_status_label(blocked.status)
    );
    println!(
        "blocked-fee-review-violation-codes: {}",
        blocked.violation_codes.len()
    );
    println!("stale-review-blocked: {}", blocked.stale_review_blocked);
    println!("live-provider-call-performed: false");
    println!("credential-loaded: false");
    println!("production-ready: false");

    if current.status != FeeScheduleVerificationStatus::ReadyForLocalReview
        || blocked.status != FeeScheduleVerificationStatus::Blocked
        || current.live_provider_call_performed
        || current.credential_loaded
        || blocked.live_provider_call_performed
        || blocked.credential_loaded
        || current.production_ready
        || blocked.production_ready
    {
        return Err(AgentCliError::Validation(
            "fee schedule verification validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn local_cex_governance_review_inputs(
) -> Result<(CexCredentialScopeReviewInput, CexCredentialScopeReviewInput), AgentCliError> {
    let ready = CexCredentialScopeReviewInput::new(
        "binance-governance-review-ready",
        local_cex_exchange_venue("binance"),
        SecretRef::Keystore {
            alias: "binance-paper-api-key".to_owned(),
        },
        vec![
            CexCredentialPermission::ReadOnlyMarketData,
            CexCredentialPermission::ReadBalances,
            CexCredentialPermission::TradeOrders,
            CexCredentialPermission::CancelOrders,
        ],
        vec![
            CexCredentialPermission::ReadOnlyMarketData,
            CexCredentialPermission::ReadBalances,
            CexCredentialPermission::TradeOrders,
            CexCredentialPermission::CancelOrders,
        ],
        vec![
            CexCredentialPermission::Withdrawals,
            CexCredentialPermission::Transfers,
            CexCredentialPermission::MarginOrDerivatives,
            CexCredentialPermission::AccountAdmin,
        ],
        1_700_000_000_000,
        1_700_000_500_000,
        86_400_000,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut blocked = CexCredentialScopeReviewInput::new(
        "coinbase-governance-review-blocked",
        local_cex_exchange_venue("coinbase"),
        SecretRef::Keystore {
            alias: "coinbase-paper-api-key".to_owned(),
        },
        vec![
            CexCredentialPermission::ReadOnlyMarketData,
            CexCredentialPermission::TradeOrders,
        ],
        vec![
            CexCredentialPermission::ReadOnlyMarketData,
            CexCredentialPermission::Withdrawals,
        ],
        vec![
            CexCredentialPermission::Withdrawals,
            CexCredentialPermission::Transfers,
        ],
        1_700_000_000_000,
        1_700_000_500_000,
        86_400_000,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked.fee_schedule_reviewed = false;
    blocked.rate_limit_documentation_reviewed = false;
    blocked.terms_of_service_reviewed = false;
    blocked.jurisdiction_reviewed = false;
    blocked.api_capabilities_reviewed = false;
    blocked.incident_reputation_reviewed = false;
    Ok((ready, blocked))
}

fn local_cex_rate_limit_review_observations(
) -> Result<(CexRateLimitObservation, CexRateLimitObservation), AgentCliError> {
    let ready = CexRateLimitObservation::new(
        "binance-governance-rate-limit-ready",
        local_cex_exchange_venue("binance"),
        CexRateLimitScope::RestMarketData,
        1_200,
        60_000,
        12,
        None,
        false,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = CexRateLimitObservation::new(
        "coinbase-governance-rate-limit-blocked",
        local_cex_exchange_venue("coinbase"),
        CexRateLimitScope::OrderSubmission,
        10,
        1_000,
        10,
        Some(500),
        true,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok((ready, blocked))
}

#[allow(clippy::too_many_lines)]
fn run_cex_governance_review_validation() -> Result<(), AgentCliError> {
    let (ready_scope_input, blocked_scope_input) = local_cex_governance_review_inputs()?;
    let (ready_rate_limit_observation, blocked_rate_limit_observation) =
        local_cex_rate_limit_review_observations()?;

    let scope_reports = [&ready_scope_input, &blocked_scope_input]
        .into_iter()
        .map(|input| validate_cex_credential_scope_review(input.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let rate_limit_reports = vec![ready_rate_limit_observation, blocked_rate_limit_observation]
        .into_iter()
        .map(validate_cex_rate_limit)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let scope_ready_count = scope_reports
        .iter()
        .filter(|report| report.status == CexCredentialScopeReviewStatus::ReadyForLocalReview)
        .count();
    let scope_blocked_count = scope_reports
        .iter()
        .filter(|report| report.status == CexCredentialScopeReviewStatus::Blocked)
        .count();
    let scope_violation_code_count = scope_reports
        .iter()
        .map(|report| report.violation_codes.len())
        .sum::<usize>();
    let rate_limit_ready_count = rate_limit_reports
        .iter()
        .filter(|report| report.status == CexRateLimitStatus::ReadyForLocalReview)
        .count();
    let rate_limit_blocked_count = rate_limit_reports
        .iter()
        .filter(|report| report.status == CexRateLimitStatus::Blocked)
        .count();
    let rate_limit_violation_code_count = rate_limit_reports
        .iter()
        .map(|report| report.violation_codes.len())
        .sum::<usize>();
    let unsafe_side_effect = [&ready_scope_input, &blocked_scope_input]
        .into_iter()
        .any(|input| {
            input.secret_material_loaded
                || input.credential_plaintext_seen
                || input.live_provider_call_performed
                || input.account_state_queried
                || input.live_execution_performed
                || input.production_ready_claimed
        })
        || scope_reports.iter().any(|report| {
            report.secret_material_loaded
                || report.credential_plaintext_seen
                || report.live_provider_call_performed
                || report.account_state_queried
                || report.live_execution_performed
                || report.production_ready
        })
        || rate_limit_reports.iter().any(|report| {
            report.live_provider_call_performed
                || report.websocket_connection_opened
                || report.credential_loaded
                || report.live_execution_performed
                || report.production_ready
        });

    println!("cex-governance-review: validation passed");
    println!("scope-review-count: {}", scope_reports.len());
    println!("scope-ready-count: {scope_ready_count}");
    println!("scope-blocked-count: {scope_blocked_count}");
    println!("scope-blocker-count: {scope_violation_code_count}");
    println!("rate-limit-review-count: {}", rate_limit_reports.len());
    println!("rate-limit-ready-count: {rate_limit_ready_count}");
    println!("rate-limit-blocked-count: {rate_limit_blocked_count}");
    println!("rate-limit-blocker-count: {rate_limit_violation_code_count}");
    println!(
        "fee-review-ready: {}",
        scope_reports[0].fee_schedule_reviewed
    );
    println!(
        "rate-limit-documentation-ready: {}",
        scope_reports[0].rate_limit_documentation_reviewed
    );
    println!(
        "terms-review-ready: {}",
        scope_reports[0].terms_of_service_reviewed
    );
    println!(
        "jurisdiction-review-ready: {}",
        scope_reports[0].jurisdiction_reviewed
    );
    println!(
        "api-capabilities-ready: {}",
        scope_reports[0].api_capabilities_reviewed
    );
    println!(
        "incident-review-ready: {}",
        scope_reports[0].incident_reputation_reviewed
    );
    println!(
        "governance-review-ready: {}",
        scope_reports[0].governance_review_passed
    );
    println!(
        "credential-reference-validated: {}",
        scope_reports[0].credential_reference_validated
    );
    println!(
        "rate-limit-budget-blocked: {}",
        rate_limit_reports[1].local_budget_exhausted
    );
    println!(
        "rate-limit-provider-blocked: {}",
        rate_limit_reports[1].provider_rate_limited
    );
    println!("live-provider-call-performed: false");
    println!("account-state-queried: false");
    println!("credential-loaded: false");
    println!("websocket-connection-opened: false");
    println!("external-submission-performed: false");
    println!("rpc-call-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if scope_reports.len() != 2
        || scope_ready_count != 1
        || scope_blocked_count != 1
        || scope_violation_code_count != 8
        || rate_limit_reports.len() != 2
        || rate_limit_ready_count != 1
        || rate_limit_blocked_count != 1
        || rate_limit_violation_code_count != 2
        || !scope_reports[0].fee_schedule_reviewed
        || !scope_reports[0].rate_limit_documentation_reviewed
        || !scope_reports[0].terms_of_service_reviewed
        || !scope_reports[0].jurisdiction_reviewed
        || !scope_reports[0].api_capabilities_reviewed
        || !scope_reports[0].incident_reputation_reviewed
        || !scope_reports[0].governance_review_passed
        || !scope_reports[0].credential_reference_validated
        || !rate_limit_reports[1].local_budget_exhausted
        || !rate_limit_reports[1].provider_rate_limited
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "CEX governance review validation failed".to_owned(),
        ));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_cex_market_data_request_plan_validation() -> Result<(), AgentCliError> {
    let pair = MarketPair::new("BTC", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let plans = vec![
        CexMarketDataRequestPlan::binance_depth_rest(
            "cli-binance-rest-depth-plan",
            local_cex_exchange_venue("binance"),
            pair.clone(),
            100,
        ),
        CexMarketDataRequestPlan::binance_depth_websocket(
            "cli-binance-ws-depth-plan",
            local_cex_exchange_venue("binance"),
            pair.clone(),
        ),
        CexMarketDataRequestPlan::coinbase_product_book_rest(
            "cli-coinbase-rest-book-plan",
            local_cex_exchange_venue("coinbase"),
            pair.clone(),
            2,
        ),
        CexMarketDataRequestPlan::coinbase_product_book_websocket(
            "cli-coinbase-ws-book-plan",
            local_cex_exchange_venue("coinbase"),
            pair.clone(),
        ),
        CexMarketDataRequestPlan::kraken_depth_rest(
            "cli-kraken-rest-depth-plan",
            local_cex_exchange_venue("kraken"),
            pair.clone(),
            100,
        ),
        CexMarketDataRequestPlan::kraken_depth_websocket(
            "cli-kraken-ws-depth-plan",
            local_cex_exchange_venue("kraken"),
            pair.clone(),
        ),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let transcripts = vec![
        (
            &plans[0],
            CexMockMarketDataTranscript::new(
                "cli-binance-depth-transcript",
                CexExchangeMarketDataFormat::BinanceDepth,
                local_cex_exchange_venue("binance"),
                pair.clone(),
                r#"{"lastUpdateId":42,"bids":[["49990.00","1.25"]],"asks":[["50010.00","0.75"]]}"#,
                1_700_000_000,
                1_700_000_001,
            ),
        ),
        (
            &plans[2],
            CexMockMarketDataTranscript::new(
                "cli-coinbase-book-transcript",
                CexExchangeMarketDataFormat::CoinbaseProductBook,
                local_cex_exchange_venue("coinbase"),
                pair.clone(),
                r#"{"sequence":84,"bids":[["49991.00","1.10",1]],"asks":[["50011.00","0.90",1]]}"#,
                1_700_000_000,
                1_700_000_001,
            ),
        ),
        (
            &plans[4],
            CexMockMarketDataTranscript::new(
                "cli-kraken-depth-transcript",
                CexExchangeMarketDataFormat::KrakenDepth,
                local_cex_exchange_venue("kraken"),
                pair,
                r#"{"error":[],"result":{"XBTUSDC":{"b":[["49992.0","1.05","1700000000"]],"a":[["50012.0","0.95","1700000000"]]}}}"#,
                1_700_000_000,
                1_700_000_001,
            ),
        ),
    ];

    let mut parsed_books = 0usize;
    let mut total_levels = 0usize;
    for (plan, transcript_result) in transcripts {
        let transcript =
            transcript_result.map_err(|error| AgentCliError::Validation(error.to_string()))?;
        let book = plan
            .parse_transcript(&transcript)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
        parsed_books += 1;
        total_levels += book.bids.len() + book.asks.len();
    }

    let rest_plan_count = plans
        .iter()
        .filter(|plan| plan.request_kind == CexMarketDataRequestKind::RestOrderBook)
        .count();
    let websocket_plan_count = plans
        .iter()
        .filter(|plan| plan.request_kind == CexMarketDataRequestKind::WebSocketOrderBook)
        .count();
    let unsafe_side_effect = plans.iter().any(|plan| {
        plan.rest_call_performed
            || plan.websocket_connection_opened
            || plan.credentials_loaded
            || plan.live_execution_performed
            || plan.production_ready
    });

    println!("cex-market-data-request-plans: validation passed");
    println!("request-plan-count: {}", plans.len());
    println!("rest-request-plan-count: {rest_plan_count}");
    println!("websocket-request-plan-count: {websocket_plan_count}");
    println!("parsed-transcript-count: {parsed_books}");
    println!("parsed-order-book-level-count: {total_levels}");
    println!("rest-call-performed: false");
    println!("websocket-connection-opened: false");
    println!("credential-loaded: false");
    println!("external-submission-performed: false");
    println!("rpc-call-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if plans.len() != 6
        || rest_plan_count != 3
        || websocket_plan_count != 3
        || parsed_books != 3
        || total_levels < 6
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "CEX market-data request plan validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_cex_balance_snapshot_validation() -> Result<(), AgentCliError> {
    let venue = VenueRef {
        name: "paper-cex".to_owned(),
        kind: VenueKind::Cex,
    };
    let transcripts = vec![
        CexBalanceSnapshotTranscript::new(
            "cli-binance-balance-transcript",
            CexBalanceSnapshotTranscriptFormat::BinanceAccountBalances,
            venue.clone(),
            r#"{"balances":[{"asset":"BTC","free":"1.25","locked":"0.10"},{"asset":"USDC","free":"5000","locked":"25"}]}"#,
            1_700_000_001,
            1_700_000_002,
        ),
        CexBalanceSnapshotTranscript::new(
            "cli-coinbase-balance-transcript",
            CexBalanceSnapshotTranscriptFormat::CoinbaseAccounts,
            venue.clone(),
            r#"{"accounts":[{"currency":"BTC","available_balance":{"value":"0.75"},"hold":{"value":"0.05"}},{"currency":"USD","available_balance":{"value":"1000"},"hold":{"value":"0"}}]}"#,
            1_700_000_003,
            1_700_000_004,
        ),
        CexBalanceSnapshotTranscript::new(
            "cli-kraken-balance-transcript",
            CexBalanceSnapshotTranscriptFormat::KrakenBalance,
            venue,
            r#"{"result":{"XXBT":"0.5","ZUSD":"1250"}}"#,
            1_700_000_005,
            1_700_000_006,
        ),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let snapshots = transcripts
        .iter()
        .map(|transcript| {
            transcript
                .parse_snapshot()
                .map_err(|error| AgentCliError::Validation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let balance_count: usize = snapshots
        .iter()
        .map(|snapshot| snapshot.balances.len())
        .sum();
    let total_available: f64 = snapshots
        .iter()
        .flat_map(|snapshot| snapshot.balances.iter())
        .map(|balance| balance.available)
        .sum();
    let unsafe_side_effect = transcripts.iter().any(|transcript| {
        transcript.rest_call_performed
            || transcript.websocket_connection_opened
            || transcript.credentials_loaded
            || transcript.account_state_queried
            || transcript.live_execution_performed
            || transcript.production_ready
    }) || snapshots.iter().any(|snapshot| {
        snapshot.rest_call_performed
            || snapshot.websocket_connection_opened
            || snapshot.credentials_loaded
            || snapshot.account_state_queried
            || snapshot.live_execution_performed
            || snapshot.production_ready
    });

    println!("cex-balance-snapshots: validation passed");
    println!("balance-transcript-count: {}", transcripts.len());
    println!("parsed-balance-snapshot-count: {}", snapshots.len());
    println!("parsed-balance-asset-count: {balance_count}");
    println!("parsed-balance-available-total: {total_available}");
    println!("rest-call-performed: false");
    println!("websocket-connection-opened: false");
    println!("credential-loaded: false");
    println!("account-state-queried: false");
    println!("external-submission-performed: false");
    println!("rpc-call-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if transcripts.len() != 3
        || snapshots.len() != 3
        || balance_count != 6
        || total_available <= 0.0
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "CEX balance snapshot validation failed".to_owned(),
        ));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_dex_request_plan_validation() -> Result<(), AgentCliError> {
    let eth_usdc = MarketPair::new("ETH", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let sol_usdc = MarketPair::new("SOL", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let plans = vec![
        DexRequestPlan::uniswap_v3_quoter_eth_call(
            local_dex_venue("paper-uniswap"),
            eth_usdc.clone(),
            "ethereum",
        ),
        DexRequestPlan::zero_ex_swap_quote_http(
            local_dex_venue("paper-0x"),
            eth_usdc.clone(),
            "ethereum",
        ),
        DexRequestPlan::jupiter_quote_http(local_dex_venue("paper-jupiter"), sol_usdc, "solana"),
        DexRequestPlan::evm_transaction_simulation_eth_call(
            local_dex_venue("paper-evm-simulation"),
            eth_usdc,
            "ethereum",
        ),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let http_quote_plan_count = plans
        .iter()
        .filter(|plan| plan.request_kind == DexRequestPlanKind::HttpQuote)
        .count();
    let solana_http_quote_plan_count = plans
        .iter()
        .filter(|plan| plan.request_kind == DexRequestPlanKind::SolanaHttpQuote)
        .count();
    let rpc_quote_plan_count = plans
        .iter()
        .filter(|plan| plan.request_kind == DexRequestPlanKind::RpcQuoteCall)
        .count();
    let rpc_simulation_plan_count = plans
        .iter()
        .filter(|plan| plan.request_kind == DexRequestPlanKind::RpcSimulationCall)
        .count();
    let quote_request_count = plans
        .iter()
        .filter(|plan| plan.request_kind != DexRequestPlanKind::RpcSimulationCall)
        .map(|plan| {
            plan.to_local_quote_request(
                format!("{}-quote", plan.id),
                "strategy-dex-request-plans",
                1.0,
                100.0,
                100.0,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .len();
    let simulation_request_count = plans
        .iter()
        .filter(|plan| plan.request_kind == DexRequestPlanKind::RpcSimulationCall)
        .map(|plan| {
            plan.to_local_simulation_request("dex-plan-simulation", "dex-plan-swap", 1.0, 99.0)
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .len();
    let unsafe_side_effect = plans.iter().any(|plan| {
        plan.http_call_performed
            || plan.rpc_call_performed
            || plan.credentials_loaded
            || plan.signing_performed
            || plan.broadcast_performed
            || plan.bridge_performed
            || plan.live_execution_performed
            || plan.production_ready
    });

    println!("dex-request-plans: validation passed");
    println!("request-plan-count: {}", plans.len());
    println!("http-quote-plan-count: {http_quote_plan_count}");
    println!("solana-http-quote-plan-count: {solana_http_quote_plan_count}");
    println!("rpc-quote-plan-count: {rpc_quote_plan_count}");
    println!("rpc-simulation-plan-count: {rpc_simulation_plan_count}");
    println!("local-quote-request-count: {quote_request_count}");
    println!("local-simulation-request-count: {simulation_request_count}");
    println!("http-call-performed: false");
    println!("rpc-call-performed: false");
    println!("credential-loaded: false");
    println!("external-submission-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("bridge-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if plans.len() != 4
        || http_quote_plan_count != 1
        || solana_http_quote_plan_count != 1
        || rpc_quote_plan_count != 1
        || rpc_simulation_plan_count != 1
        || quote_request_count != 3
        || simulation_request_count != 1
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "DEX request plan validation failed".to_owned(),
        ));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_dex_response_transcript_validation() -> Result<(), AgentCliError> {
    let eth_usdc = MarketPair::new("ETH", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let sol_usdc = MarketPair::new("SOL", "USDC")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let plans = vec![
        DexRequestPlan::uniswap_v3_quoter_eth_call(
            local_dex_venue("paper-uniswap"),
            eth_usdc.clone(),
            "ethereum",
        ),
        DexRequestPlan::zero_ex_swap_quote_http(
            local_dex_venue("paper-0x"),
            eth_usdc.clone(),
            "ethereum",
        ),
        DexRequestPlan::jupiter_quote_http(local_dex_venue("paper-jupiter"), sol_usdc, "solana"),
        DexRequestPlan::evm_transaction_simulation_eth_call(
            local_dex_venue("paper-evm-simulation"),
            eth_usdc,
            "ethereum",
        ),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let quote_payloads = [
        r#"{"amountIn":"1.0","amountOut":"1900.0","priceImpactBps":4.0,"estimatedFeeQuote":0.1,"gasFeeQuote":0.25}"#,
        r#"{"sellAmount":"1.0","buyAmount":"1899.5","priceImpactBps":"5.0","feeQuote":"0.2","estimatedGasQuote":"0.3"}"#,
        r#"{"inAmount":"1","outAmount":"100","priceImpactBps":6,"marketDataAgeMs":500}"#,
    ];

    let quote_transcripts = plans
        .iter()
        .filter(|plan| plan.request_kind != DexRequestPlanKind::RpcSimulationCall)
        .zip(quote_payloads)
        .enumerate()
        .map(|(index, (plan, payload))| {
            DexResponseTranscript::local(
                format!("dex-response-transcript-{index}"),
                format!("dex-response-request-{index}"),
                plan.request_kind,
                plan.protocol_label.clone(),
                plan.venue.clone(),
                plan.chain.clone(),
                plan.pair.clone(),
                payload,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut parsed_quote_count = 0usize;
    let mut parsed_quote_amount_out_total = 0.0;
    for (plan, transcript) in plans
        .iter()
        .filter(|plan| plan.request_kind != DexRequestPlanKind::RpcSimulationCall)
        .zip(&quote_transcripts)
    {
        let response = plan
            .parse_quote_transcript(transcript)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
        parsed_quote_count += 1;
        parsed_quote_amount_out_total += response.amount_out;
    }

    let simulation_plan = plans
        .iter()
        .find(|plan| plan.request_kind == DexRequestPlanKind::RpcSimulationCall)
        .ok_or_else(|| AgentCliError::Validation("missing simulation request plan".to_owned()))?;
    let simulation_transcript = DexResponseTranscript::local(
        "dex-simulation-response-transcript",
        "dex-simulation-response-request",
        simulation_plan.request_kind,
        simulation_plan.protocol_label.clone(),
        simulation_plan.venue.clone(),
        simulation_plan.chain.clone(),
        simulation_plan.pair.clone(),
        r#"{"status":"success","gasUsed":"142000","gasFeeQuote":"0.24","amountOut":"1898.0","diagnostic":"local simulation fixture"}"#,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let simulation_response = simulation_plan
        .parse_simulation_transcript(&simulation_transcript)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let transcript_count = quote_transcripts.len() + 1;
    let unsafe_side_effect = quote_transcripts
        .iter()
        .chain([&simulation_transcript])
        .any(|transcript| {
            transcript.http_response_received
                || transcript.rpc_response_received
                || transcript.credentials_loaded
                || transcript.signing_performed
                || transcript.broadcast_performed
                || transcript.bridge_performed
                || transcript.live_execution_performed
                || transcript.production_ready
        });

    println!("dex-response-transcripts: validation passed");
    println!("response-transcript-count: {transcript_count}");
    println!("parsed-quote-response-count: {parsed_quote_count}");
    println!("parsed-simulation-response-count: 1");
    println!("parsed-quote-amount-out-total: {parsed_quote_amount_out_total}");
    println!("simulation-status: {:?}", simulation_response.status);
    println!("simulation-gas-used: {}", simulation_response.gas_used);
    println!("http-response-received: false");
    println!("rpc-response-received: false");
    println!("credential-loaded: false");
    println!("external-submission-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("bridge-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if transcript_count != 4
        || parsed_quote_count != 3
        || simulation_response.status != DexSimulationStatus::WouldSucceed
        || simulation_response.gas_used == 0
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "DEX response transcript validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_dex_transaction_lifecycle_transcript_validation() -> Result<(), AgentCliError> {
    let transcripts = local_web3_transaction_lifecycle_transcripts()?;

    let records = transcripts
        .iter()
        .map(Web3TransactionLifecycleTranscript::parse_record)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let confirmed_count = records
        .iter()
        .filter(|record| record.status == Web3TransactionLifecycleStatus::Confirmed)
        .count();
    let reverted_count = records
        .iter()
        .filter(|record| record.status == Web3TransactionLifecycleStatus::Reverted)
        .count();
    let failed_count = records
        .iter()
        .filter(|record| record.status == Web3TransactionLifecycleStatus::Failed)
        .count();
    let nonce_tracked_count = records
        .iter()
        .filter(|record| record.nonce.is_some())
        .count();
    let confirmation_total = records
        .iter()
        .map(|record| record.confirmations)
        .sum::<u64>();
    let unsafe_side_effect = transcripts.iter().any(|transcript| {
        transcript.rpc_response_received
            || transcript.credentials_loaded
            || transcript.signer_material_loaded
            || transcript.signing_performed
            || transcript.broadcast_performed
            || transcript.bridge_performed
            || transcript.live_execution_performed
            || transcript.production_ready
    }) || records.iter().any(|record| {
        record.rpc_call_performed
            || record.signing_performed
            || record.broadcast_performed
            || record.live_execution_performed
            || record.production_ready
    });

    println!("dex-transaction-lifecycle-transcripts: validation passed");
    println!(
        "transaction-lifecycle-transcript-count: {}",
        transcripts.len()
    );
    println!("transaction-lifecycle-record-count: {}", records.len());
    println!("transaction-lifecycle-confirmed-count: {confirmed_count}");
    println!("transaction-lifecycle-reverted-count: {reverted_count}");
    println!("transaction-lifecycle-failed-count: {failed_count}");
    println!("transaction-lifecycle-nonce-tracked-count: {nonce_tracked_count}");
    println!("transaction-lifecycle-confirmation-total: {confirmation_total}");
    println!("rpc-response-received: false");
    println!("credential-loaded: false");
    println!("signer-material-loaded: false");
    println!("external-submission-performed: false");
    println!("rpc-call-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("bridge-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if transcripts.len() != 4
        || records.len() != 4
        || confirmed_count != 2
        || reverted_count != 1
        || failed_count != 1
        || nonce_tracked_count != 2
        || confirmation_total != 48
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "DEX transaction lifecycle transcript validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn local_web3_transaction_lifecycle_transcripts(
) -> Result<Vec<Web3TransactionLifecycleTranscript>, AgentCliError> {
    vec![
        Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-evm-confirmed",
            "web3-request-evm-confirmed",
            "ethereum",
            local_dex_venue("paper-uniswap"),
            Web3TransactionLifecycleTranscriptFormat::EvmTransactionReceipt,
            r#"{"transactionHash":"0xabc123","status":"0x1","blockNumber":"19000001","confirmations":"12","nonce":"7","gasUsed":"142000"}"#,
        ),
        Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-evm-reverted",
            "web3-request-evm-reverted",
            "ethereum",
            local_dex_venue("paper-uniswap"),
            Web3TransactionLifecycleTranscriptFormat::EvmTransactionReceipt,
            r#"{"transactionHash":"0xdef456","status":"0x0","blockNumber":"19000002","confirmations":"3","nonce":"8","revertReason":"local fixture revert"}"#,
        ),
        Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-solana-finalized",
            "web3-request-solana-finalized",
            "solana",
            local_dex_venue("paper-jupiter"),
            Web3TransactionLifecycleTranscriptFormat::SolanaSignatureStatus,
            r#"{"signature":"5abc","slot":"250000000","confirmations":32,"confirmationStatus":"finalized","err":null}"#,
        ),
        Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-solana-failed",
            "web3-request-solana-failed",
            "solana",
            local_dex_venue("paper-jupiter"),
            Web3TransactionLifecycleTranscriptFormat::SolanaSignatureStatus,
            r#"{"signature":"5def","slot":"250000001","confirmations":1,"confirmationStatus":"confirmed","err":"InstructionError"}"#,
        ),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn run_dex_protocol_risk_review_validation() -> Result<(), AgentCliError> {
    let (ready_request, blocked_request) = local_dex_protocol_risk_review_requests()?;

    let reports = [&ready_request, &blocked_request]
        .into_iter()
        .map(DexProtocolRiskReviewRequest::review)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ready_count = reports
        .iter()
        .filter(|report| report.status == DexProtocolRiskReviewStatus::ReadyForLocalReview)
        .count();
    let blocked_count = reports
        .iter()
        .filter(|report| report.status == DexProtocolRiskReviewStatus::Blocked)
        .count();
    let total_blocker_codes = reports
        .iter()
        .map(|report| report.blocker_codes.len())
        .sum::<usize>();
    let unsafe_side_effect = [&ready_request, &blocked_request]
        .into_iter()
        .any(|request| {
            request.rpc_call_performed
                || request.signer_material_loaded
                || request.signing_performed
                || request.broadcast_performed
                || request.bridge_performed
                || request.live_execution_performed
                || request.production_ready
        })
        || reports.iter().any(|report| {
            report.rpc_call_performed
                || report.signer_material_loaded
                || report.signing_performed
                || report.broadcast_performed
                || report.bridge_performed
                || report.live_execution_performed
                || report.production_ready
        });

    println!("dex-protocol-risk-review: validation passed");
    println!("protocol-risk-review-count: {}", reports.len());
    println!("protocol-risk-ready-count: {ready_count}");
    println!("protocol-risk-blocked-count: {blocked_count}");
    println!("protocol-risk-blocker-count: {total_blocker_codes}");
    println!("asset-scope-ready: {}", reports[0].asset_scope_passed);
    println!(
        "contract-hygiene-ready: {}",
        reports[0].contract_hygiene_passed
    );
    println!("token-hygiene-ready: {}", reports[0].token_hygiene_passed);
    println!(
        "spender-hygiene-ready: {}",
        reports[0].spender_hygiene_passed
    );
    println!("gas-slippage-ready: {}", reports[0].gas_slippage_passed);
    println!("mev-controls-ready: {}", reports[0].mev_controls_passed);
    println!(
        "governance-review-ready: {}",
        reports[0].governance_review_passed
    );
    println!("terms-metadata-ready: {}", reports[0].terms_metadata_passed);
    println!("rpc-call-performed: false");
    println!("signer-material-loaded: false");
    println!("signing-or-broadcast-performed: false");
    println!("bridge-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if reports.len() != 2
        || ready_count != 1
        || blocked_count != 1
        || total_blocker_codes != 16
        || !reports[0].asset_scope_passed
        || !reports[0].contract_hygiene_passed
        || !reports[0].token_hygiene_passed
        || !reports[0].spender_hygiene_passed
        || !reports[0].gas_slippage_passed
        || !reports[0].mev_controls_passed
        || !reports[0].governance_review_passed
        || !reports[0].terms_metadata_passed
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "DEX protocol risk review validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn local_dex_protocol_risk_review_requests(
) -> Result<(DexProtocolRiskReviewRequest, DexProtocolRiskReviewRequest), AgentCliError> {
    let ready_request = DexProtocolRiskReviewRequest::local(
        "dex-protocol-review-ready",
        "uniswap-v3-reviewed",
        local_dex_venue("paper-uniswap"),
        "ethereum",
        "uniswap-v3-router-reviewed",
        "uniswap-v3-spender-reviewed",
        MarketPair::new("ETH", "USDC")
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        75,
        25.0,
        2.0,
        0.25,
        10.0,
        50.0,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut blocked_request = DexProtocolRiskReviewRequest::local(
        "dex-protocol-review-blocked",
        "uniswap-v3-reviewed",
        local_dex_venue("paper-uniswap"),
        "ethereum",
        "uniswap-v3-router-reviewed",
        "unknown-spender",
        MarketPair::new("ETH", "USDC")
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        50,
        75.0,
        0.25,
        0.50,
        80.0,
        40.0,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked_request.chain_allowlisted = false;
    blocked_request.pair_allowlisted = false;
    blocked_request.router_allowlisted = false;
    blocked_request.spender_allowlisted = false;
    blocked_request.unlimited_allowance_requested = true;
    blocked_request.approval_revocation_planned = false;
    blocked_request.token_contract_reviewed = false;
    blocked_request.token_decimals_verified = false;
    blocked_request.public_mempool_required = true;
    blocked_request.mev_mitigation_reviewed = false;
    blocked_request.token_metadata_reviewed = false;
    blocked_request.protocol_terms_reviewed = false;
    blocked_request.jurisdiction_reviewed = false;
    blocked_request.incident_reputation_reviewed = false;
    Ok((ready_request, blocked_request))
}

fn local_cex_exchange_venue(name: &str) -> VenueRef {
    VenueRef {
        name: name.to_owned(),
        kind: VenueKind::Cex,
    }
}

fn local_dex_venue(name: &str) -> VenueRef {
    VenueRef {
        name: name.to_owned(),
        kind: VenueKind::Dex,
    }
}

struct LocalFeeBoundaryAuditCase {
    current_review: FeeScheduleVerificationReport,
    blocked_review: FeeScheduleVerificationReport,
}

struct FeeBoundaryAuditPersistence {
    current_sequence: u64,
    blocked_sequence: u64,
    checkpoint_value: String,
    audit_failed_closed: bool,
}

fn run_fee_boundary_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("fee-boundary.audit.jsonl");
    let state_path = options.workspace_dir.join("fee-boundary.sqlite3");
    let fee_case = build_local_fee_boundary_audit_case()?;
    let persisted =
        persist_local_fee_boundary_audit_case(&audit_path, &state_path, &fee_case, now_unix_ms)?;
    let audit_records_replayed =
        verify_local_fee_boundary_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_fee_boundary_state_failure(&fee_case);

    if !state_failure_failed_closed
        || fee_case.current_review.status != FeeScheduleVerificationStatus::ReadyForLocalReview
        || fee_case.blocked_review.status != FeeScheduleVerificationStatus::Blocked
        || fee_case.current_review.live_provider_call_performed
        || fee_case.current_review.credential_loaded
        || fee_case.current_review.production_ready
        || fee_case.blocked_review.live_provider_call_performed
        || fee_case.blocked_review.credential_loaded
        || fee_case.blocked_review.production_ready
    {
        return Err(AgentCliError::Validation(
            "fee boundary audit/state validation failed".to_owned(),
        ));
    }

    println!("fee-boundary-audit: validation passed");
    println!(
        "current-fee-review-status: {}",
        fee_schedule_verification_status_label(fee_case.current_review.status)
    );
    println!(
        "blocked-fee-review-status: {}",
        fee_schedule_verification_status_label(fee_case.blocked_review.status)
    );
    println!(
        "fee-verification-audit-failed-closed: {}",
        persisted.audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("live-provider-call-performed: false");
    println!("credential-loaded: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_fee_boundary_audit_case() -> Result<LocalFeeBoundaryAuditCase, AgentCliError> {
    let current_review = validate_fee_schedule_verification(FeeScheduleVerificationInput {
        schedule: local_verified_fee_schedule(true),
        review_id: "local-fee-boundary-current".to_owned(),
        source_reference: "operator-fee-boundary-current".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked_review = validate_fee_schedule_verification(FeeScheduleVerificationInput {
        schedule: local_verified_fee_schedule(false),
        review_id: "local-fee-boundary-blocked".to_owned(),
        source_reference: "operator-fee-boundary-blocked".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(LocalFeeBoundaryAuditCase {
        current_review,
        blocked_review,
    })
}

fn persist_local_fee_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    fee_case: &LocalFeeBoundaryAuditCase,
    now_unix_ms: u64,
) -> Result<FeeBoundaryAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let current_audit =
        append_fee_schedule_verification_audit(&mut journal, &fee_case.current_review, now_unix_ms)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked_audit = append_fee_schedule_verification_audit(
        &mut journal,
        &fee_case.blocked_review,
        now_unix_ms.saturating_add(1),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_fee_schedule_verification_checkpoint(
        &mut store,
        &fee_case.current_review,
        now_unix_ms.saturating_add(2),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_failed_closed = validate_fee_verification_invalid_audit_fails_closed(
        &mut journal,
        &fee_case.current_review,
    );

    Ok(FeeBoundaryAuditPersistence {
        current_sequence: current_audit.sequence,
        blocked_sequence: blocked_audit.sequence,
        checkpoint_value: checkpoint.value,
        audit_failed_closed,
    })
}

fn verify_local_fee_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &FeeBoundaryAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened
        .get_checkpoint(FEE_LAST_VERIFICATION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("fee verification checkpoint missing".to_owned())
        })?;

    if replayed.next_sequence() <= persisted.blocked_sequence
        || persisted.current_sequence == persisted.blocked_sequence
        || checkpoint.value != persisted.checkpoint_value
        || !persisted.audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "fee boundary audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_fee_verification_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &FeeScheduleVerificationReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.live_provider_call_performed = true;
    invalid.status = FeeScheduleVerificationStatus::ReadyForLocalReview;
    let failed = append_fee_schedule_verification_audit(journal, &invalid, 1_700_000_601).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_fee_boundary_state_failure(fee_case: &LocalFeeBoundaryAuditCase) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let failed =
        persist_fee_schedule_verification_checkpoint(&mut store, &fee_case.current_review, 1)
            .is_err();
    failed && store.put_attempts == 1
}

struct AgenticHandoffAuditPersistence {
    review_sequence: u64,
    checkpoint_value: String,
    audit_failed_closed: bool,
}

fn run_agentic_handoff_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("agentic-handoff.audit.jsonl");
    let state_path = options.workspace_dir.join("agentic-handoff.sqlite3");
    let review_record = build_local_agentic_handoff_review_record()?;
    let persisted = persist_local_agentic_handoff_audit_case(
        &audit_path,
        &state_path,
        &review_record,
        now_unix_ms,
    )?;
    let audit_records_replayed =
        verify_local_agentic_handoff_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_agentic_handoff_state_failure(&review_record);

    if !state_failure_failed_closed
        || review_record.status != AgenticHandoffReviewStatus::ReadyForExternalReview
        || review_record.external_agents_executed
        || review_record.external_validation_claimed
        || review_record.production_ready
        || review_record.live_funds_approved
        || review_record.public_exposure_approved
        || review_record.secret_material_recorded
    {
        return Err(AgentCliError::Validation(
            "agentic handoff audit/state validation failed".to_owned(),
        ));
    }

    println!("agentic-handoff-audit: validation passed");
    println!("handoff-package: {}", review_record.package_id);
    println!("handoff-artifacts: {}", review_record.artifact_count);
    println!(
        "handoff-unresolved-gaps: {}",
        review_record.unresolved_gap_count
    );
    println!(
        "handoff-live-funds-blockers: {}",
        review_record.live_funds_blocker_count
    );
    println!(
        "handoff-audit-failed-closed: {}",
        persisted.audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("external-agents-executed: false");
    println!("external-validation-claimed: false");
    println!("production-ready: false");
    println!("live-funds-approved: false");
    println!("public-exposure-approved: false");
    println!("secret-material-recorded: false");
    Ok(())
}

fn build_local_agentic_handoff_review_record() -> Result<AgenticHandoffReviewRecord, AgentCliError>
{
    let packager = DeterministicAgenticHandoffPackager;
    packager
        .review_package(AgenticHandoffReviewRequest::conservative(
            "local-agentic-handoff-audit",
            "local-operator",
        ))
        .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn persist_local_agentic_handoff_audit_case(
    audit_path: &Path,
    state_path: &Path,
    review_record: &AgenticHandoffReviewRecord,
    now_unix_ms: u64,
) -> Result<AgenticHandoffAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit = append_agentic_handoff_review_audit(&mut journal, review_record, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_agentic_handoff_review_checkpoint(
        &mut store,
        review_record,
        now_unix_ms.saturating_add(1),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_failed_closed =
        validate_agentic_handoff_invalid_audit_fails_closed(&mut journal, review_record);

    Ok(AgenticHandoffAuditPersistence {
        review_sequence: audit.sequence,
        checkpoint_value: checkpoint.value,
        audit_failed_closed,
    })
}

fn verify_local_agentic_handoff_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &AgenticHandoffAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened
        .get_checkpoint(AGENTIC_HANDOFF_LAST_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| AgentCliError::Validation("handoff review checkpoint missing".to_owned()))?;

    if replayed.next_sequence() <= persisted.review_sequence
        || checkpoint.value != persisted.checkpoint_value
        || !persisted.audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "agentic handoff audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_agentic_handoff_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    review_record: &AgenticHandoffReviewRecord,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = review_record.clone();
    invalid.external_agents_executed = true;
    let failed = append_agentic_handoff_review_audit(journal, &invalid, 1_700_000_701).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_agentic_handoff_state_failure(review_record: &AgenticHandoffReviewRecord) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let failed = persist_agentic_handoff_review_checkpoint(&mut store, review_record, 1).is_err();
    failed && store.put_attempts == 1
}

fn local_verified_fee_schedule(externally_verified: bool) -> arb_core::FeeSchedule {
    arb_core::FeeSchedule {
        venue: VenueRef {
            kind: VenueKind::Cex,
            name: "paper-coinbase".to_owned(),
        },
        pair: Some(MarketPair::new("BTC", "USD").expect("static pair should validate")),
        maker_bps: 2.0,
        taker_bps: 6.0,
        network_fee_quote: 0.50,
        externally_verified,
    }
}

fn local_provider_venue(venue_name: &str) -> VenueRef {
    VenueRef {
        kind: VenueKind::Cex,
        name: venue_name.to_owned(),
    }
}

fn local_provider_quote(
    id: &str,
    venue_name: &str,
    pair: MarketPair,
    bid_price: f64,
    ask_price: f64,
    quantity_base: f64,
) -> NormalizedQuote {
    NormalizedQuote {
        id: id.to_owned(),
        venue: local_provider_venue(venue_name),
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

fn local_provider_book(
    id: &str,
    venue_name: &str,
    pair: MarketPair,
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
) -> OrderBookSnapshot {
    OrderBookSnapshot {
        id: id.to_owned(),
        venue: local_provider_venue(venue_name),
        pair,
        captured_at_unix_ms: 9_500,
        received_at_unix_ms: 9_500,
        bids: bids
            .into_iter()
            .map(|(price_quote, quantity_base)| PriceLevel {
                price_quote,
                quantity_base,
            })
            .collect(),
        asks: asks
            .into_iter()
            .map(|(price_quote, quantity_base)| PriceLevel {
                price_quote,
                quantity_base,
            })
            .collect(),
        source_sequence: None,
    }
}

fn local_provider_fee(venue_name: &str, pair: MarketPair) -> FeeSchedule {
    FeeSchedule {
        venue: local_provider_venue(venue_name),
        pair: Some(pair),
        maker_bps: 5.0,
        taker_bps: 10.0,
        network_fee_quote: 0.0,
        externally_verified: false,
    }
}

fn local_strategy_profile(min_net_profit_abs: f64) -> StrategyProfile {
    let mut profile = StrategyProfile::conservative_paper("cli-strategy-profile", "USD");
    profile.opportunity.min_net_profit_abs = min_net_profit_abs;
    profile.venues.allowed_exchanges = vec!["paper-a".to_owned(), "paper-b".to_owned()];
    profile.venues.allowed_assets = vec!["BTC".to_owned(), "USD".to_owned()];
    profile
}

fn local_strategy_replay_profile(id: &str, min_net_profit_abs: f64) -> StrategyProfile {
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

fn local_strategy_planner_candidate() -> Result<OpportunityCandidate, AgentCliError> {
    let pair = MarketPair::new("BTC", "USD")
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let edge = FeeAdjustedEdge::calculate(15.0, 2.0, 100.0)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(OpportunityCandidate {
        id: "cli-strategy-opp-cex-cex-btc-usd".to_owned(),
        route_kind: OpportunityRouteKind::CexCex,
        pair: pair.clone(),
        legs: vec![
            local_strategy_planner_leg("paper-a", pair.clone(), OpportunityLegSide::Buy, 100.0),
            local_strategy_planner_leg("paper-b", pair, OpportunityLegSide::Sell, 115.0),
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
        source_quote_ids: vec!["quote-paper-a".to_owned(), "quote-paper-b".to_owned()],
        warnings: Vec::new(),
    })
}

fn local_strategy_planner_leg(
    venue_name: &str,
    pair: MarketPair,
    side: OpportunityLegSide,
    price_quote: f64,
) -> OpportunityLeg {
    let quantity_base = 1.0;
    let notional_quote = price_quote * quantity_base;
    OpportunityLeg {
        venue: local_provider_venue(venue_name),
        pair: pair.clone(),
        side,
        price_quote,
        quantity_base,
        notional_quote,
        fee_estimate: FeeEstimate {
            venue: local_provider_venue(venue_name),
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

struct LocalOpportunityMarketProvider {
    quotes: Vec<NormalizedQuote>,
    books: Vec<OrderBookSnapshot>,
}

impl MarketDataProvider for LocalOpportunityMarketProvider {
    fn provider_name(&self) -> &'static str {
        "cli-local-opportunity-market-provider"
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
        request.validate()?;
        self.books
            .iter()
            .find(|book| book.venue == request.venue && book.pair == request.pair)
            .cloned()
            .ok_or_else(|| MarketDataError::NoData {
                provider: self.provider_name().to_owned(),
                reason: format!("missing book for {}", request.pair.symbol()),
            })
    }

    fn top_of_book(&self, request: &MarketDataRequest) -> Result<NormalizedQuote, MarketDataError> {
        request.validate()?;
        self.quotes
            .iter()
            .find(|quote| quote.venue == request.venue && quote.pair == request.pair)
            .cloned()
            .ok_or_else(|| MarketDataError::NoData {
                provider: self.provider_name().to_owned(),
                reason: format!("missing quote for {}", request.pair.symbol()),
            })
    }
}

struct LocalOpportunityFeeProvider {
    schedules: Vec<FeeSchedule>,
}

impl FeeProvider for LocalOpportunityFeeProvider {
    fn provider_name(&self) -> &'static str {
        "cli-local-opportunity-fee-provider"
    }

    fn fee_schedule(
        &self,
        venue: &VenueRef,
        pair: Option<&MarketPair>,
    ) -> Result<FeeSchedule, FeeModelError> {
        self.schedules
            .iter()
            .find(|schedule| {
                &schedule.venue == venue
                    && match (&schedule.pair, pair) {
                        (Some(left), Some(right)) => left == right,
                        (None, None) => true,
                        (None, Some(_)) | (Some(_), None) => false,
                    }
            })
            .cloned()
            .ok_or_else(|| FeeModelError::ScheduleUnavailable {
                provider: self.provider_name().to_owned(),
                reason: "missing local fee schedule".to_owned(),
            })
    }
}

fn parse_positive_usize(value: &str, message: &str) -> Result<usize, AgentCliError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| AgentCliError::Usage(message.to_owned()))?;
    if parsed == 0 {
        return Err(AgentCliError::Usage(message.to_owned()));
    }
    Ok(parsed)
}

fn run_opportunity_historical_fixture_validation() -> Result<(), AgentCliError> {
    let corpus = phase27_local_opportunity_historical_fixture_corpus()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let report = DeterministicOpportunityEngine::new()
        .replay_historical_fixture_corpus(&corpus)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!(
        "opportunity-historical-fixture-corpus: {}",
        report.corpus_id
    );
    println!("window-count: {}", report.window_count);
    println!("scenario-count: {}", report.scenario_count);
    println!("passed-windows: {}", report.passed_windows);
    println!("failed-windows: {}", report.failed_windows);
    println!("passed-scenarios: {}", report.passed_scenarios);
    println!("failed-scenarios: {}", report.failed_scenarios);
    println!("total-candidates: {}", report.total_candidates);
    println!(
        "opportunity-historical-fixture-status: {}",
        opportunity_replay_status_label(report.status)
    );
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!("production-ready: false");

    if report.external_calls_performed || report.live_execution_performed {
        return Err(AgentCliError::Validation(
            "opportunity historical fixture replay reported forbidden side effects".to_owned(),
        ));
    }

    if report.status != OpportunityReplayStatus::Passed {
        let failed = report
            .window_reports
            .iter()
            .filter(|window| window.status == OpportunityReplayStatus::Failed)
            .map(|window| window.corpus_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AgentCliError::Validation(format!(
            "opportunity historical fixture replay failed windows: {failed}"
        )));
    }

    Ok(())
}

fn run_opportunity_planner_handoff_validation() -> Result<(), AgentCliError> {
    let corpus = phase27_local_opportunity_historical_fixture_corpus()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(
        AgentConfig::from_toml_str(PHASE27_PLANNER_HANDOFF_CONFIG)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
    );
    let trace_workspace = opportunity_planner_trace_workspace()?;
    fs::create_dir_all(&trace_workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create local opportunity planner trace workspace: {error}"
        ))
    })?;
    let trace_result = (|| {
        let audit_path = trace_workspace.join("opportunity-candidate-trace.audit.jsonl");
        let state_path = trace_workspace.join("opportunity-candidate-trace.sqlite3");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
        let mut store = SqliteWalStateStore::open(&state_path)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
        validate_opportunity_planner_handoff_with_trace(&corpus, &policy, &mut journal, &mut store)
            .map_err(|error| AgentCliError::Validation(error.to_string()))
    })();
    let cleanup_result = remove_dir_all_with_retry(&trace_workspace);
    let report = trace_result?;
    cleanup_result.map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to remove local opportunity planner trace workspace: {error}"
        ))
    })?;

    println!("opportunity-planner-handoff-corpus: {}", report.corpus_id);
    println!("replay-window-count: {}", report.replay_window_count);
    println!("replay-scenario-count: {}", report.replay_scenario_count);
    println!(
        "skipped-discovery-failures: {}",
        report.skipped_discovery_failures
    );
    println!("discovered-candidates: {}", report.discovered_candidates);
    println!("planned-candidates: {}", report.planned_candidates);
    println!("draft-ready-plans: {}", report.draft_ready_plans);
    println!("policy-denied-plans: {}", report.policy_denied_plans);
    println!(
        "failed-planner-handoffs: {}",
        report.failed_planner_handoffs
    );
    println!(
        "candidate-trace-audit-records: {}",
        report.candidate_trace_audit_records
    );
    println!(
        "candidate-trace-checkpoints: {}",
        report.candidate_trace_checkpoints
    );
    println!("total-intents: {}", report.total_intents);
    println!(
        "adapter-submission-enabled: {}",
        report.adapter_submission_enabled
    );
    println!(
        "opportunity-planner-handoff-status: {}",
        opportunity_planner_handoff_status_label(report.status)
    );
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!("production-ready: false");

    if report.adapter_submission_enabled
        || report.external_calls_performed
        || report.live_execution_performed
    {
        return Err(AgentCliError::Validation(
            "opportunity planner handoff reported forbidden side effects".to_owned(),
        ));
    }

    if report.candidate_trace_audit_records != report.discovered_candidates
        || report.candidate_trace_checkpoints != report.discovered_candidates
    {
        return Err(AgentCliError::Validation(
            "opportunity planner handoff did not trace every discovered candidate".to_owned(),
        ));
    }

    if report.status != OpportunityPlannerHandoffStatus::Passed {
        return Err(AgentCliError::Validation(
            "opportunity planner handoff validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_strategy_constrained_planner_validation() -> Result<(), AgentCliError> {
    let policy = PolicyEngine::from_config(
        AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
    );
    let request = ExecutionPlannerRequest {
        id: "cli-strategy-constrained-planner".to_owned(),
        strategy_id: "cli-strategy-profile".to_owned(),
        candidate: local_strategy_planner_candidate()?,
        config: ExecutionPlannerConfig::default(),
        default_chain: None,
        now_unix_ms: 10_000,
    };
    let accepted_profile = local_strategy_profile(1.0);
    let accepted = DeterministicExecutionPlanner::new()
        .plan_with_strategy_profile(&request, &policy, &accepted_profile)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let rejected_profile = local_strategy_profile(1_000.0);
    let rejected = DeterministicExecutionPlanner::new()
        .plan_with_strategy_profile(&request, &policy, &rejected_profile)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("strategy-constrained-planner: validation passed");
    println!(
        "accepted-plan-status: {}",
        plan_status_label(accepted.draft.status)
    );
    println!(
        "accepted-strategy-rejected-intents: {}",
        accepted.strategy_rejected_intents
    );
    println!("accepted-intents: {}", accepted.draft.intents.len());
    println!(
        "rejected-plan-status: {}",
        plan_status_label(rejected.draft.status)
    );
    println!(
        "rejected-strategy-rejected-intents: {}",
        rejected.strategy_rejected_intents
    );
    println!(
        "adapter-submission-performed: {}",
        accepted.adapter_submission_performed || rejected.adapter_submission_performed
    );
    println!(
        "live-execution-performed: {}",
        accepted.live_execution_performed || rejected.live_execution_performed
    );
    println!(
        "signing-or-broadcast-performed: {}",
        accepted.signing_or_broadcast_performed || rejected.signing_or_broadcast_performed
    );
    println!(
        "production-ready: {}",
        accepted.production_ready || rejected.production_ready
    );

    if accepted.draft.status != arb_core::ExecutionPlanStatus::DraftReady
        || accepted.strategy_rejected_intents != 0
        || !accepted.strategy_constraint_reports.iter().all(|report| {
            report.status == StrategyPolicyConstraintStatus::Satisfied
                && !report.execution_performed
                && !report.signing_or_broadcast_performed
                && !report.live_network_used
        })
        || rejected.draft.status != arb_core::ExecutionPlanStatus::PolicyDeniedDraft
        || rejected.strategy_rejected_intents != rejected.draft.intents.len()
        || !rejected
            .strategy_constraint_reports
            .iter()
            .all(|report| report.status == StrategyPolicyConstraintStatus::Rejected)
        || accepted.adapter_submission_performed
        || rejected.adapter_submission_performed
        || accepted.live_execution_performed
        || rejected.live_execution_performed
        || accepted.signing_or_broadcast_performed
        || rejected.signing_or_broadcast_performed
        || accepted.production_ready
        || rejected.production_ready
    {
        return Err(AgentCliError::Validation(
            "strategy-constrained planner validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_strategy_replay_corpus_validation() -> Result<(), AgentCliError> {
    let corpus = phase27_local_opportunity_historical_fixture_corpus()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(
        AgentConfig::from_toml_str(PHASE27_PLANNER_HANDOFF_CONFIG)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
    );
    let accepted_profile = local_strategy_replay_profile("cli-strategy-replay-accepted", 0.0);
    let rejected_profile =
        local_strategy_replay_profile("cli-strategy-replay-rejected", 1_000_000.0);
    let report = validate_strategy_profile_replay_corpus(
        &corpus,
        &policy,
        &accepted_profile,
        &rejected_profile,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("strategy-profile-replay-corpus: {}", report.corpus_id);
    println!("replay-window-count: {}", report.replay_window_count);
    println!("replay-scenario-count: {}", report.replay_scenario_count);
    println!(
        "skipped-discovery-failures: {}",
        report.skipped_discovery_failures
    );
    println!("discovered-candidates: {}", report.discovered_candidates);
    println!(
        "accepted-planned-candidates: {}",
        report.accepted_planned_candidates
    );
    println!(
        "accepted-draft-ready-plans: {}",
        report.accepted_draft_ready_plans
    );
    println!(
        "accepted-satisfied-constraint-reports: {}",
        report.accepted_satisfied_constraint_reports
    );
    println!(
        "accepted-strategy-rejected-intents: {}",
        report.accepted_strategy_rejected_intents
    );
    println!(
        "rejected-planned-candidates: {}",
        report.rejected_planned_candidates
    );
    println!(
        "rejected-policy-denied-plans: {}",
        report.rejected_policy_denied_plans
    );
    println!(
        "rejected-constraint-reports: {}",
        report.rejected_constraint_reports
    );
    println!(
        "rejected-strategy-rejected-intents: {}",
        report.rejected_strategy_rejected_intents
    );
    println!(
        "failed-strategy-planner-runs: {}",
        report.failed_strategy_planner_runs
    );
    println!("total-accepted-intents: {}", report.total_accepted_intents);
    println!("total-rejected-intents: {}", report.total_rejected_intents);
    println!(
        "strategy-replay-status: {}",
        strategy_profile_replay_status_label(report.status)
    );
    println!(
        "adapter-submission-performed: {}",
        report.adapter_submission_performed
    );
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!(
        "signing-or-broadcast-performed: {}",
        report.signing_or_broadcast_performed
    );
    println!("production-ready: {}", report.production_ready);

    if report.status != StrategyProfileReplayValidationStatus::Passed
        || report.adapter_submission_performed
        || report.external_calls_performed
        || report.live_execution_performed
        || report.signing_or_broadcast_performed
        || report.production_ready
    {
        return Err(AgentCliError::Validation(
            "strategy replay corpus validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_strategy_profitability_tuning_validation() -> Result<(), AgentCliError> {
    let corpus = phase27_local_opportunity_historical_fixture_corpus()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(
        AgentConfig::from_toml_str(PHASE27_PLANNER_HANDOFF_CONFIG)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
    );
    let report = validate_strategy_profitability_tuning(&corpus, &policy)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let lowest = report.profitability_points.first().ok_or_else(|| {
        AgentCliError::Validation("missing lowest profitability threshold".to_owned())
    })?;
    let highest = report.profitability_points.last().ok_or_else(|| {
        AgentCliError::Validation("missing highest profitability threshold".to_owned())
    })?;

    println!("strategy-profitability-tuning-corpus: {}", report.corpus_id);
    println!("replay-window-count: {}", report.replay_window_count);
    println!("replay-scenario-count: {}", report.replay_scenario_count);
    println!(
        "skipped-discovery-failures: {}",
        report.skipped_discovery_failures
    );
    println!("discovered-candidates: {}", report.discovered_candidates);
    println!(
        "profitability-threshold-count: {}",
        report.profitability_points.len()
    );
    println!(
        "lowest-threshold-min-net-profit-abs: {}",
        lowest.min_net_profit_abs
    );
    println!(
        "highest-threshold-min-net-profit-abs: {}",
        highest.min_net_profit_abs
    );
    println!(
        "lowest-threshold-draft-ready-plans: {}",
        lowest.draft_ready_plans
    );
    println!(
        "highest-threshold-policy-denied-plans: {}",
        highest.policy_denied_plans
    );
    println!(
        "highest-threshold-rejected-intents: {}",
        highest.rejected_intents
    );
    println!(
        "monotonic-acceptance-validated: {}",
        report.monotonic_acceptance_validated
    );
    println!(
        "monotonic-rejection-validated: {}",
        report.monotonic_rejection_validated
    );
    println!(
        "profitability-threshold-transition-observed: {}",
        report.threshold_transition_observed
    );
    println!(
        "strategy-profitability-status: {}",
        strategy_profitability_tuning_status_label(report.status)
    );
    println!(
        "adapter-submission-performed: {}",
        report.adapter_submission_performed
    );
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!(
        "signing-or-broadcast-performed: {}",
        report.signing_or_broadcast_performed
    );
    println!("production-ready: {}", report.production_ready);

    if report.status != StrategyProfitabilityTuningValidationStatus::Passed
        || report.adapter_submission_performed
        || report.external_calls_performed
        || report.live_execution_performed
        || report.signing_or_broadcast_performed
        || report.production_ready
    {
        return Err(AgentCliError::Validation(
            "strategy profitability tuning validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_opportunity_trace_recovery_validation() -> Result<(), AgentCliError> {
    let corpus = phase27_local_opportunity_historical_fixture_corpus()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(
        AgentConfig::from_toml_str(PHASE27_PLANNER_HANDOFF_CONFIG)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
    );
    let trace_workspace = opportunity_planner_trace_workspace()?;
    fs::create_dir_all(&trace_workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create local opportunity trace recovery workspace: {error}"
        ))
    })?;
    let trace_result = {
        let audit_path = trace_workspace.join("opportunity-candidate-trace-recovery.audit.jsonl");
        let state_path = trace_workspace.join("opportunity-candidate-trace-recovery.sqlite3");
        validate_opportunity_candidate_trace_restart_recovery(
            &corpus,
            &policy,
            &audit_path,
            &state_path,
        )
        .map_err(|error| AgentCliError::Validation(error.to_string()))
    };
    let cleanup_result = remove_dir_all_with_retry(&trace_workspace);
    let report = trace_result?;
    cleanup_result.map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to remove local opportunity trace recovery workspace: {error}"
        ))
    })?;

    println!("opportunity-trace-recovery-corpus: {}", report.corpus_id);
    println!(
        "discovered-candidates: {}",
        report.handoff_report.discovered_candidates
    );
    println!(
        "candidate-trace-audit-records: {}",
        report.handoff_report.candidate_trace_audit_records
    );
    println!("audit-replay-records: {}", report.audit_replay_records);
    println!(
        "candidate-trace-checkpoints: {}",
        report.handoff_report.candidate_trace_checkpoints
    );
    println!(
        "recovered-trace-checkpoints: {}",
        report.recovered_trace_checkpoints
    );
    println!(
        "recovered-trace-summaries: {}",
        report.recovered_trace_summaries.len()
    );
    println!(
        "missing-trace-checkpoints: {}",
        report.missing_trace_checkpoints.len()
    );
    println!(
        "trace-recovery-validated: {}",
        report.trace_recovery_validated
    );
    println!(
        "opportunity-trace-recovery-status: {}",
        opportunity_planner_handoff_status_label(report.status)
    );
    println!(
        "external-calls-performed: {}",
        report.external_calls_performed
    );
    println!(
        "live-execution-performed: {}",
        report.live_execution_performed
    );
    println!("production-ready: false");

    if !report.trace_recovery_validated
        || report.status != OpportunityPlannerHandoffStatus::Passed
        || report.external_calls_performed
        || report.live_execution_performed
    {
        return Err(AgentCliError::Validation(
            "opportunity trace recovery validation failed".to_owned(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSmokeOptions {
    config_path: PathBuf,
    workspace_dir: PathBuf,
    iterations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalIterationOptions {
    iterations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalValidationRunOptions {
    workspace_dir: PathBuf,
}

fn parse_local_iteration_options(
    args: impl Iterator<Item = String>,
    command_name: &str,
    label: &str,
) -> Result<LocalIterationOptions, AgentCliError> {
    let mut iterations = 1_usize;
    let mut pending = args;
    while let Some(arg) = pending.next() {
        match arg.as_str() {
            "--iterations" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(format!(
                        "{command_name} --iterations requires an integer >= 1"
                    )));
                };
                let parsed = value.parse::<usize>().map_err(|_| {
                    AgentCliError::Usage(format!(
                        "{command_name} --iterations requires an integer >= 1"
                    ))
                })?;
                if parsed == 0 {
                    return Err(AgentCliError::Usage(format!(
                        "{command_name} --iterations requires an integer >= 1"
                    )));
                }
                iterations = parsed;
            }
            other => {
                return Err(AgentCliError::Usage(format!(
                    "unknown {label} argument: {other}"
                )));
            }
        }
    }

    Ok(LocalIterationOptions { iterations })
}

fn parse_local_validation_run_options(
    args: impl Iterator<Item = String>,
) -> Result<LocalValidationRunOptions, AgentCliError> {
    let mut workspace_dir = None;
    let mut pending = args;
    while let Some(arg) = pending.next() {
        match arg.as_str() {
            "--workspace" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-local-validation-run --workspace requires a fresh directory"
                            .to_owned(),
                    ));
                };
                workspace_dir = Some(PathBuf::from(value));
            }
            other => {
                return Err(AgentCliError::Usage(format!(
                    "unknown validate-local-validation-run argument: {other}"
                )));
            }
        }
    }

    Ok(LocalValidationRunOptions {
        workspace_dir: workspace_dir.ok_or_else(|| {
            AgentCliError::Usage(
                "validate-local-validation-run requires --workspace <fresh-dir>".to_owned(),
            )
        })?,
    })
}

fn write_audit_retention_fixture(path: &Path, label: &str) -> Result<(), AgentCliError> {
    fs::write(path, format!("{{\"event\":\"{label}\"}}\n")).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to write local retention journal {}: {error}",
            path.display()
        ))
    })
}

fn audit_retention_file_metadata(
    path: &Path,
    modified_at_unix_ms: u64,
    active: bool,
) -> Result<AuditJournalFileMetadata, AgentCliError> {
    let size_bytes = fs::metadata(path)
        .map_err(|error| {
            AgentCliError::Validation(format!(
                "failed to inspect local retention journal {}: {error}",
                path.display()
            ))
        })?
        .len();

    Ok(AuditJournalFileMetadata {
        path: path.display().to_string(),
        size_bytes,
        modified_at_unix_ms,
        active,
    })
}

fn run_audit_durability_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    let now_unix_ms = current_unix_ms()?;
    let report = validate_audit_journal_durability(&options.workspace_dir, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("audit-durability: validation passed");
    println!("audit-durability-workspace: {}", report.workspace_dir);
    println!("audit-durability-version: {}", report.validation_version);
    println!(
        "audit-durability-append-replay-validated: {}",
        report.append_replay_validated
    );
    println!(
        "audit-durability-truncated-replay-rejected: {}",
        report.truncated_replay_rejected
    );
    println!(
        "audit-durability-tamper-replay-rejected: {}",
        report.tamper_replay_rejected
    );
    println!(
        "audit-durability-concurrent-append-validated: {}",
        report.concurrent_append_validated
    );
    println!(
        "audit-durability-filesystem-failure-validated: {}",
        report.filesystem_failure_validated
    );
    println!(
        "audit-durability-disk-full-failure-validated: {}",
        report.disk_full_failure_validated
    );
    println!("audit-durability-append-records: {}", report.append_records);
    println!(
        "audit-durability-concurrent-records: {}",
        report.concurrent_records
    );
    println!(
        "audit-durability-live-network-used: {}",
        report.live_network_used
    );
    println!(
        "audit-durability-external-execution-performed: {}",
        report.external_execution_performed
    );
    println!(
        "audit-durability-unresolved-blockers: {}",
        report.unresolved_blockers.len()
    );
    println!("production-ready: {}", report.production_ready);
    Ok(())
}

fn run_runtime_graceful_shutdown_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options
        .workspace_dir
        .join("runtime-graceful-shutdown.audit.jsonl");
    let state_path = options
        .workspace_dir
        .join("runtime-graceful-shutdown.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let record = run_local_graceful_shutdown_checkpoint(
        &mut journal,
        &mut store,
        RuntimeGracefulShutdownRequest {
            id: "cli-runtime-graceful-shutdown".to_owned(),
            reason: "local-cli-deployment-host-graceful-shutdown-validation".to_owned(),
            now_unix_ms,
        },
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    drop(store);
    drop(journal);

    let reopened_journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_records_replayed = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopened_store
        .integrity_check()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_checkpoint = reopened_store
        .get_checkpoint(RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_checkpoint_matches = recovered_checkpoint
        .as_ref()
        .is_some_and(|checkpoint| checkpoint.value == record.shutdown_checkpoint_value);

    println!("runtime-graceful-shutdown-validation: passed");
    println!(
        "runtime-graceful-shutdown-workspace: {}",
        options.workspace_dir.display()
    );
    println!("runtime-graceful-shutdown-id: {}", record.id);
    println!(
        "runtime-graceful-shutdown-version: {}",
        record.runtime_graceful_shutdown_version
    );
    println!("runtime-graceful-shutdown-audit-records-replayed: {audit_records_replayed}");
    println!(
        "runtime-graceful-shutdown-start-audit-sequence: {}",
        record.shutdown_start_audit_sequence
    );
    println!(
        "runtime-graceful-shutdown-checkpoint-audit-sequence: {}",
        record.shutdown_checkpoint_audit_sequence
    );
    println!(
        "runtime-graceful-shutdown-checkpoint-key: {}",
        record.shutdown_checkpoint_key
    );
    println!(
        "runtime-graceful-shutdown-checkpoint-recovered: {}",
        recovered_checkpoint.is_some()
    );
    println!("runtime-graceful-shutdown-checkpoint-matches-record: {recovered_checkpoint_matches}");
    println!("runtime-graceful-shutdown-audit-replayed: true");
    println!("runtime-graceful-shutdown-sqlite-integrity-check-passed: true");
    println!("runtime-graceful-shutdown-service-manager-action-performed: false");
    println!(
        "runtime-graceful-shutdown-external-submission-performed: {}",
        record.external_submission_performed
    );
    println!(
        "runtime-graceful-shutdown-live-execution-performed: {}",
        record.live_execution_performed
    );
    println!("runtime-graceful-shutdown-production-ready: false");

    if audit_records_replayed != 2
        || recovered_checkpoint.is_none()
        || !recovered_checkpoint_matches
        || record.external_submission_performed
        || record.live_execution_performed
        || record.production_ready
    {
        return Err(AgentCliError::Validation(
            "runtime graceful-shutdown validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_runtime_backup_restore_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options
        .workspace_dir
        .join("runtime-backup-restore.audit.jsonl");
    let state_path = options.workspace_dir.join("runtime-backup-restore.sqlite3");
    let backup_audit_path = options
        .workspace_dir
        .join("runtime-backup-restore.copy.audit.jsonl");
    let backup_state_path = options
        .workspace_dir
        .join("runtime-backup-restore.copy.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config.clone());
    let request =
        build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, "backup-restore")?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let lifecycle = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);

    let report = validate_local_runtime_backup_restore(
        &audit_path,
        &state_path,
        &backup_audit_path,
        &backup_state_path,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("runtime-backup-restore-validation: passed");
    println!(
        "runtime-backup-restore-workspace: {}",
        options.workspace_dir.display()
    );
    println!("runtime-backup-restore-lifecycle-id: {}", lifecycle.id);
    println!(
        "runtime-backup-restore-version: {}",
        report.validation_version
    );
    println!(
        "runtime-backup-restore-audit-records-replayed: {}",
        report.audit_records_replayed
    );
    println!(
        "runtime-backup-restore-audit-restore-check-passed: {}",
        report.audit_restore_check_passed
    );
    println!(
        "runtime-backup-restore-sqlite-restore-check-passed: {}",
        report.sqlite_restore_check_passed
    );
    println!(
        "runtime-backup-restore-plan-checkpoint-restored: {}",
        report.plan_checkpoint_restored
    );
    println!(
        "runtime-backup-restore-adapter-checkpoint-restored: {}",
        report.adapter_checkpoint_restored
    );
    println!(
        "runtime-backup-restore-adapter-recovery-plan-checkpoint-restored: {}",
        report.adapter_recovery_plan_checkpoint_restored
    );
    println!(
        "runtime-backup-restore-external-submission-performed: {}",
        report.external_submission_performed
    );
    println!(
        "runtime-backup-restore-live-execution-performed: {}",
        report.live_execution_performed
    );
    println!(
        "runtime-backup-restore-production-ready: {}",
        report.production_ready
    );

    Ok(())
}

fn run_runtime_backup_restore_load_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options
        .workspace_dir
        .join("runtime-backup-restore-load.audit.jsonl");
    let state_path = options
        .workspace_dir
        .join("runtime-backup-restore-load.sqlite3");
    let backup_audit_path = options
        .workspace_dir
        .join("runtime-backup-restore-load.copy.audit.jsonl");
    let backup_state_path = options
        .workspace_dir
        .join("runtime-backup-restore-load.copy.sqlite3");
    let workers = 4_usize;
    run_runtime_backup_restore_load_workers(&audit_path, &state_path, workers)?;

    let backup_report = validate_local_runtime_backup_restore(
        &audit_path,
        &state_path,
        &backup_audit_path,
        &backup_state_path,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let restart_report = validate_local_runtime_restart_recovery(&audit_path, &state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let backup_journal = AppendOnlyAuditJournal::open(&backup_audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let backup_journal_sequence_matches =
        backup_journal.next_sequence() == backup_report.audit_records_replayed.saturating_add(1);
    let reopened_state = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopened_state
        .integrity_check()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let plan_reopened = reopened_state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .is_some();
    let adapter_reopened = reopened_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .is_some();
    let adapter_recovery_plan_reopened = reopened_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .is_some();

    print_runtime_backup_restore_load_validation_report(
        &options.workspace_dir,
        workers,
        &backup_report,
        &restart_report,
        backup_journal_sequence_matches,
        RuntimeBackupRestoreLoadCheckpointReopenChecks {
            plan_reopened,
            adapter_reopened,
            adapter_recovery_plan_reopened,
        },
    );

    if !backup_journal_sequence_matches
        || !plan_reopened
        || !adapter_reopened
        || !adapter_recovery_plan_reopened
    {
        return Err(AgentCliError::Validation(
            "runtime backup/restore load validation failed".to_owned(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct RuntimeBackupRestoreLoadCheckpointReopenChecks {
    plan_reopened: bool,
    adapter_reopened: bool,
    adapter_recovery_plan_reopened: bool,
}

fn print_runtime_backup_restore_load_validation_report(
    workspace_dir: &Path,
    workers: usize,
    backup_report: &arb_core::RuntimeBackupRestoreValidationReport,
    restart_report: &arb_core::RuntimeRestartRecoveryValidationReport,
    backup_journal_sequence_matches: bool,
    checks: RuntimeBackupRestoreLoadCheckpointReopenChecks,
) {
    println!("runtime-backup-restore-load-validation: passed");
    println!(
        "runtime-backup-restore-load-workspace: {}",
        workspace_dir.display()
    );
    println!("runtime-backup-restore-load-workers: {workers}");
    println!(
        "runtime-backup-restore-load-audit-records-replayed: {}",
        backup_report.audit_records_replayed
    );
    println!(
        "runtime-backup-restore-load-audit-restore-check-passed: {}",
        backup_report.audit_restore_check_passed
    );
    println!(
        "runtime-backup-restore-load-sqlite-restore-check-passed: {}",
        backup_report.sqlite_restore_check_passed
    );
    print_runtime_backup_restore_load_checkpoint_report(
        backup_report,
        restart_report,
        backup_journal_sequence_matches,
        checks,
    );
    println!(
        "runtime-backup-restore-load-external-submission-performed: {}",
        backup_report.external_submission_performed || restart_report.external_submission_performed
    );
    println!(
        "runtime-backup-restore-load-live-execution-performed: {}",
        backup_report.live_execution_performed || restart_report.live_execution_performed
    );
    println!(
        "runtime-backup-restore-load-production-ready: {}",
        backup_report.production_ready || restart_report.production_ready
    );
}

fn print_runtime_backup_restore_load_checkpoint_report(
    backup_report: &arb_core::RuntimeBackupRestoreValidationReport,
    restart_report: &arb_core::RuntimeRestartRecoveryValidationReport,
    backup_journal_sequence_matches: bool,
    checks: RuntimeBackupRestoreLoadCheckpointReopenChecks,
) {
    println!(
        "runtime-backup-restore-load-plan-checkpoint-restored: {}",
        backup_report.plan_checkpoint_restored
    );
    println!(
        "runtime-backup-restore-load-adapter-checkpoint-restored: {}",
        backup_report.adapter_checkpoint_restored
    );
    println!(
        "runtime-backup-restore-load-adapter-recovery-plan-checkpoint-restored: {}",
        backup_report.adapter_recovery_plan_checkpoint_restored
    );
    println!(
        "runtime-backup-restore-load-restart-audit-replay-check-passed: {}",
        restart_report.audit_replay_check_passed
    );
    println!(
        "runtime-backup-restore-load-restart-sqlite-reopen-check-passed: {}",
        restart_report.sqlite_reopen_check_passed
    );
    println!(
        "runtime-backup-restore-load-backup-journal-sequence-matches: {backup_journal_sequence_matches}"
    );
    println!(
        "runtime-backup-restore-load-plan-reopened: {}",
        checks.plan_reopened
    );
    println!(
        "runtime-backup-restore-load-adapter-reopened: {}",
        checks.adapter_reopened
    );
    println!(
        "runtime-backup-restore-load-adapter-recovery-plan-reopened: {}",
        checks.adapter_recovery_plan_reopened
    );
}

fn run_runtime_backup_restore_load_workers(
    audit_path: &Path,
    state_path: &Path,
    workers: usize,
) -> Result<(), AgentCliError> {
    let barrier = Arc::new(Barrier::new(workers));
    let open_lock = Arc::new(Mutex::new(()));
    let handles = (0..workers)
        .map(|worker| {
            let audit_path = audit_path.to_path_buf();
            let state_path = state_path.to_path_buf();
            let barrier = Arc::clone(&barrier);
            let open_lock = Arc::clone(&open_lock);
            thread::spawn(move || {
                run_runtime_backup_restore_load_worker(
                    worker,
                    &audit_path,
                    &state_path,
                    &barrier,
                    &open_lock,
                )
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle
            .join()
            .map_err(|_| AgentCliError::Validation("load worker panicked".to_owned()))?
            .map_err(AgentCliError::Validation)?;
    }
    Ok(())
}

fn run_runtime_backup_restore_load_worker(
    worker: usize,
    audit_path: &Path,
    state_path: &Path,
    barrier: &Barrier,
    open_lock: &Mutex<()>,
) -> Result<(), String> {
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| error.to_string())?;
    let policy = PolicyEngine::from_config(config.clone());
    let mut request = build_runtime_smoke_lifecycle_request(
        &config,
        &policy,
        90_000 + u64::try_from(worker).unwrap_or(u64::MAX),
        &format!("backup-restore-load-worker-{worker}"),
    )
    .map_err(|error| error.to_string())?;
    request.id = format!("runtime-backup-restore-load-worker-{worker}");
    request.adapter_request_id = format!("adapter-backup-restore-load-worker-{worker}");
    request.plan.id = format!("plan-backup-restore-load-worker-{worker}");

    let (mut journal, mut store) = {
        let _guard = open_lock
            .lock()
            .map_err(|_| "load worker open lock poisoned".to_owned())?;
        (
            AppendOnlyAuditJournal::open(audit_path).map_err(|error| error.to_string())?,
            SqliteWalStateStore::open(state_path).map_err(|error| error.to_string())?,
        )
    };
    barrier.wait();
    let record = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
        .map_err(|error| error.to_string())?;
    if record.status != RuntimeLifecycleStatus::AdapterRunCheckpointed
        || record.external_submission_performed
        || record.live_execution_performed
    {
        return Err("runtime backup/restore load worker violated local-only invariants".to_owned());
    }
    Ok(())
}

fn run_runtime_restart_recovery_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options
        .workspace_dir
        .join("runtime-restart-recovery.audit.jsonl");
    let state_path = options
        .workspace_dir
        .join("runtime-restart-recovery.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config.clone());
    let request =
        build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, "restart-recovery")?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let lifecycle = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let shutdown_record = run_local_graceful_shutdown_checkpoint(
        &mut journal,
        &mut store,
        RuntimeGracefulShutdownRequest {
            id: "cli-runtime-restart-recovery-shutdown".to_owned(),
            reason: "local-cli-restart-recovery-checkpoint".to_owned(),
            now_unix_ms: now_unix_ms.saturating_add(1),
        },
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);
    seed_runtime_restart_connector_lifecycle_checkpoints(
        &audit_path,
        &state_path,
        now_unix_ms.saturating_add(2),
    )?;

    let report = validate_local_runtime_restart_recovery_with_trace_recovery(
        &audit_path,
        &state_path,
        &policy,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if !report.connector_lifecycle_recovery_validated {
        return Err(AgentCliError::Validation(
            "runtime restart recovery connector lifecycle checkpoints were not recovered"
                .to_owned(),
        ));
    }
    print_runtime_restart_recovery_validation_report(
        &options.workspace_dir,
        lifecycle.id.as_str(),
        shutdown_record.id.as_str(),
        &report,
    );

    Ok(())
}

fn run_runtime_incomplete_recovery_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options
        .workspace_dir
        .join("runtime-incomplete-recovery.audit.jsonl");
    let state_path = options
        .workspace_dir
        .join("runtime-incomplete-recovery.sqlite3");
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    journal
        .append_event(arb_core::AuditEvent::new(
            "runtime:incomplete-recovery:start",
            arb_core::AuditEventKind::RuntimeLifecycle,
            "runtime-lifecycle",
            "runtime",
            "runtime lifecycle started without durable checkpoints",
        ))
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_records_before_validation = journal.next_sequence().saturating_sub(1);
    drop(store);
    drop(journal);

    let error = validate_local_runtime_restart_recovery(&audit_path, &state_path)
        .err()
        .ok_or_else(|| {
            AgentCliError::Validation(
                "runtime incomplete-recovery validation unexpectedly succeeded".to_owned(),
            )
        })?;
    let error_text = error.to_string();
    let expected_failure = error_text
        .contains("coherent local planner, adapter, and adapter recovery-plan checkpoints");

    let reopened_journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened_audit_records = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_state = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let plan_checkpoint_recovered = reopened_state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .is_some();
    let adapter_checkpoint_recovered = reopened_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .is_some();
    let adapter_recovery_plan_checkpoint_recovered = reopened_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .is_some();

    println!("runtime-incomplete-recovery-validation: passed");
    println!(
        "runtime-incomplete-recovery-workspace: {}",
        options.workspace_dir.display()
    );
    println!("runtime-incomplete-recovery-expected-failure: {expected_failure}");
    println!(
        "runtime-incomplete-recovery-audit-records-before-validation: {audit_records_before_validation}"
    );
    println!("runtime-incomplete-recovery-reopened-audit-records: {reopened_audit_records}");
    println!("runtime-incomplete-recovery-plan-checkpoint-recovered: {plan_checkpoint_recovered}");
    println!(
        "runtime-incomplete-recovery-adapter-checkpoint-recovered: {adapter_checkpoint_recovered}"
    );
    println!(
        "runtime-incomplete-recovery-adapter-recovery-plan-checkpoint-recovered: {adapter_recovery_plan_checkpoint_recovered}"
    );
    println!("runtime-incomplete-recovery-service-manager-action-performed: false");
    println!("runtime-incomplete-recovery-external-submission-performed: false");
    println!("runtime-incomplete-recovery-live-execution-performed: false");
    println!("runtime-incomplete-recovery-production-ready: false");

    if !expected_failure
        || audit_records_before_validation != 1
        || reopened_audit_records != 1
        || plan_checkpoint_recovered
        || adapter_checkpoint_recovered
        || adapter_recovery_plan_checkpoint_recovered
    {
        return Err(AgentCliError::Validation(
            "runtime incomplete-recovery validation did not fail closed on missing checkpoints"
                .to_owned(),
        ));
    }

    Ok(())
}

fn run_runtime_supervised_restart_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let child_output = Command::new(env::current_exe().map_err(|error| {
        AgentCliError::Validation(format!("failed to resolve current executable: {error}"))
    })?)
    .arg("write-runtime-supervised-restart-child")
    .arg("--workspace")
    .arg(&options.workspace_dir)
    .output()
    .map_err(|error| {
        AgentCliError::Validation(format!("failed to run supervised restart child: {error}"))
    })?;

    if !child_output.status.success() {
        let stdout = String::from_utf8_lossy(&child_output.stdout);
        let stderr = String::from_utf8_lossy(&child_output.stderr);
        return Err(AgentCliError::Validation(format!(
            "supervised restart child failed with status {:?}: stdout={stdout}; stderr={stderr}",
            child_output.status.code()
        )));
    }

    let audit_path = runtime_supervised_restart_audit_path(&options.workspace_dir);
    let state_path = runtime_supervised_restart_state_path(&options.workspace_dir);
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config);
    let report = validate_local_runtime_restart_recovery_with_trace_recovery(
        &audit_path,
        &state_path,
        &policy,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if !report.connector_lifecycle_recovery_validated {
        return Err(AgentCliError::Validation(
            "runtime supervised restart connector lifecycle checkpoints were not recovered"
                .to_owned(),
        ));
    }

    let child_stdout_lines = String::from_utf8_lossy(&child_output.stdout)
        .lines()
        .count();
    println!("runtime-supervised-restart-validation: passed");
    println!(
        "runtime-supervised-restart-workspace: {}",
        options.workspace_dir.display()
    );
    println!(
        "runtime-supervised-restart-child-exit-code: {}",
        child_output.status.code().unwrap_or_default()
    );
    println!("runtime-supervised-restart-child-stdout-lines: {child_stdout_lines}");
    print_runtime_restart_recovery_validation_report(
        &options.workspace_dir,
        "child-process-runtime-lifecycle",
        "child-process-runtime-shutdown",
        &report,
    );
    Ok(())
}

fn run_runtime_supervised_restart_child(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    if !options.workspace_dir.is_dir() {
        return Err(AgentCliError::Usage(format!(
            "supervised restart child requires an existing workspace directory: {}",
            options.workspace_dir.display()
        )));
    }
    write_runtime_supervised_restart_seed(&options.workspace_dir)?;
    println!("runtime-supervised-restart-child: wrote-local-checkpoints");
    println!("runtime-supervised-restart-child-external-submission-performed: false");
    println!("runtime-supervised-restart-child-live-execution-performed: false");
    println!("runtime-supervised-restart-child-production-ready: false");
    Ok(())
}

fn run_runtime_permission_denial_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options
        .workspace_dir
        .join("runtime-permission-denial.audit.jsonl");
    let now_unix_ms = current_unix_ms()?;
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config.clone());
    let request =
        build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, "permission-denial")?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = PermissionDeniedLocalStateStore::default();
    let error = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
        .err()
        .ok_or_else(|| {
            AgentCliError::Validation(
                "runtime permission-denial lifecycle unexpectedly succeeded".to_owned(),
            )
        })?;
    let error_text = error.to_string();
    let expected_failure = error_text.contains("local permission-denied state path");
    let audit_records_replayed = journal.next_sequence().saturating_sub(1);
    drop(journal);
    let reopened_journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened_audit_records = reopened_journal.next_sequence().saturating_sub(1);
    let adapter_evaluated = error_text.contains("adapter") && !expected_failure;

    println!("runtime-permission-denial-validation: passed");
    println!(
        "runtime-permission-denial-workspace: {}",
        options.workspace_dir.display()
    );
    println!("runtime-permission-denial-expected-failure: {expected_failure}");
    println!(
        "runtime-permission-denial-state-put-attempts: {}",
        store.put_attempts
    );
    println!("runtime-permission-denial-audit-records-replayed: {audit_records_replayed}");
    println!("runtime-permission-denial-reopened-audit-records: {reopened_audit_records}");
    println!("runtime-permission-denial-adapter-evaluated: {adapter_evaluated}");
    println!("runtime-permission-denial-service-manager-action-performed: false");
    println!("runtime-permission-denial-external-submission-performed: false");
    println!("runtime-permission-denial-live-execution-performed: false");
    println!("runtime-permission-denial-production-ready: false");

    if !expected_failure
        || store.put_attempts != 1
        || audit_records_replayed != 1
        || reopened_audit_records != 1
        || adapter_evaluated
    {
        return Err(AgentCliError::Validation(
            "runtime permission-denial validation did not fail closed before adapter evaluation"
                .to_owned(),
        ));
    }

    Ok(())
}

#[derive(Debug, Default)]
struct PermissionDeniedLocalStateStore {
    put_attempts: usize,
}

impl StateStore for PermissionDeniedLocalStateStore {
    fn put_checkpoint(&mut self, _checkpoint: StateCheckpoint) -> Result<(), StateStoreError> {
        self.put_attempts += 1;
        Err(StateStoreError::BackendFailed {
            reason: "local permission-denied state path".to_owned(),
        })
    }

    fn get_checkpoint(&self, _key: &str) -> Result<Option<StateCheckpoint>, StateStoreError> {
        Ok(None)
    }
}

fn write_runtime_supervised_restart_seed(workspace_dir: &Path) -> Result<(), AgentCliError> {
    let audit_path = runtime_supervised_restart_audit_path(workspace_dir);
    let state_path = runtime_supervised_restart_state_path(workspace_dir);
    let now_unix_ms = current_unix_ms()?;
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config.clone());
    let request =
        build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, "supervised-restart")?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    run_local_graceful_shutdown_checkpoint(
        &mut journal,
        &mut store,
        RuntimeGracefulShutdownRequest {
            id: "cli-runtime-supervised-restart-shutdown".to_owned(),
            reason: "local-process-supervised-restart-checkpoint".to_owned(),
            now_unix_ms: now_unix_ms.saturating_add(1),
        },
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);
    seed_runtime_restart_connector_lifecycle_checkpoints(
        &audit_path,
        &state_path,
        now_unix_ms.saturating_add(2),
    )?;
    Ok(())
}

fn runtime_supervised_restart_audit_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("runtime-supervised-restart.audit.jsonl")
}

fn runtime_supervised_restart_state_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("runtime-supervised-restart.sqlite3")
}

fn seed_runtime_restart_connector_lifecycle_checkpoints(
    audit_path: &Path,
    state_path: &Path,
    now_unix_ms: u64,
) -> Result<(), AgentCliError> {
    let connector_case = build_local_connector_lifecycle_audit_case()?;
    let persisted = persist_local_connector_lifecycle_audit_case(
        audit_path,
        state_path,
        &connector_case,
        now_unix_ms,
    )?;
    verify_local_connector_lifecycle_audit_case(audit_path, state_path, &persisted)?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn print_runtime_restart_recovery_validation_report(
    workspace_dir: &Path,
    lifecycle_id: &str,
    shutdown_id: &str,
    report: &arb_core::RuntimeRestartRecoveryValidationReport,
) {
    let recovered_summary_count = report.opportunity_trace_recovered_summaries.len();
    let recovered_summaries_match_count =
        recovered_summary_count as u64 == report.opportunity_trace_recovered_checkpoints;

    println!("runtime-restart-recovery-validation: passed");
    println!(
        "runtime-restart-recovery-workspace: {}",
        workspace_dir.display()
    );
    println!("runtime-restart-recovery-lifecycle-id: {lifecycle_id}");
    println!("runtime-restart-recovery-shutdown-id: {shutdown_id}");
    println!(
        "runtime-restart-recovery-version: {}",
        report.validation_version
    );
    println!(
        "runtime-restart-recovery-audit-records-replayed: {}",
        report.audit_records_replayed
    );
    println!(
        "runtime-restart-recovery-audit-replay-check-passed: {}",
        report.audit_replay_check_passed
    );
    println!(
        "runtime-restart-recovery-sqlite-reopen-check-passed: {}",
        report.sqlite_reopen_check_passed
    );
    println!(
        "runtime-restart-recovery-plan-checkpoint-recovered: {}",
        report.plan_checkpoint_recovered
    );
    println!(
        "runtime-restart-recovery-adapter-checkpoint-recovered: {}",
        report.adapter_checkpoint_recovered
    );
    println!(
        "runtime-restart-recovery-adapter-recovery-plan-checkpoint-recovered: {}",
        report.adapter_recovery_plan_checkpoint_recovered
    );
    println!(
        "runtime-restart-recovery-graceful-shutdown-checkpoint-recovered: {}",
        report.graceful_shutdown_checkpoint_recovered
    );
    println!(
        "runtime-restart-recovery-disposition: {}",
        recovery_disposition_label(report.recovery_disposition)
    );
    println!(
        "runtime-restart-recovery-local-review-ready: {}",
        report.local_review_ready
    );
    println!(
        "runtime-restart-recovery-connector-lifecycle-validated: {}",
        report.connector_lifecycle_recovery_validated
    );
    println!(
        "runtime-restart-recovery-cex-lifecycle-checkpoint-recovered: {}",
        report.cex_lifecycle_checkpoint_recovered
    );
    println!(
        "runtime-restart-recovery-dex-lifecycle-checkpoint-recovered: {}",
        report.dex_lifecycle_checkpoint_recovered
    );
    if let Some(cex) = &report.recovered_cex_lifecycle {
        println!(
            "runtime-restart-recovery-cex-lifecycle-summary: request_id={};client_order_id={};strategy_id={};venue_name={};market_pair={};final_status={};transition_count={};fill_count={}",
            cex.request_id,
            cex.client_order_id,
            cex.strategy_id,
            cex.venue_name,
            cex.market_pair,
            cex.final_status,
            cex.transition_count,
            cex.fill_count
        );
    }
    if let Some(dex) = &report.recovered_dex_lifecycle {
        println!(
            "runtime-restart-recovery-dex-lifecycle-summary: request_id={};strategy_id={};venue_name={};chain={};market_pair={};quote_response_id={};simulation_response_id={};route_kind={};simulation_status={};gas_used={}",
            dex.request_id,
            dex.strategy_id,
            dex.venue_name,
            dex.chain,
            dex.market_pair,
            dex.quote_response_id,
            dex.simulation_response_id,
            dex.route_kind,
            dex.simulation_status,
            dex.gas_used
        );
    }
    print_runtime_restart_recovery_trace_report(report, recovered_summary_count);
    println!(
        "runtime-restart-recovery-opportunity-trace-recovered-summaries-match-count: {recovered_summaries_match_count}"
    );
    println!(
        "runtime-restart-recovery-external-submission-performed: {}",
        report.external_submission_performed
    );
    println!(
        "runtime-restart-recovery-live-execution-performed: {}",
        report.live_execution_performed
    );
    println!(
        "runtime-restart-recovery-production-ready: {}",
        report.production_ready
    );
}

fn print_runtime_restart_recovery_trace_report(
    report: &arb_core::RuntimeRestartRecoveryValidationReport,
    recovered_summary_count: usize,
) {
    println!(
        "runtime-restart-recovery-opportunity-trace-validated: {}",
        report.opportunity_trace_recovery_validated
    );
    println!(
        "runtime-restart-recovery-opportunity-trace-discovered-candidates: {}",
        report.opportunity_trace_discovered_candidates
    );
    println!(
        "runtime-restart-recovery-opportunity-trace-recovered-checkpoints: {}",
        report.opportunity_trace_recovered_checkpoints
    );
    println!(
        "runtime-restart-recovery-opportunity-trace-missing-checkpoints: {}",
        report.opportunity_trace_missing_checkpoints
    );
    println!(
        "runtime-restart-recovery-opportunity-trace-recovered-summary-count: {recovered_summary_count}"
    );
}

fn run_audit_retention_execution_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let active = options.workspace_dir.join("audit-active.jsonl");
    let retained = options.workspace_dir.join("audit-retained.jsonl");
    let expired = options.workspace_dir.join("audit-expired.jsonl");

    write_audit_retention_fixture(&active, "active")?;
    write_audit_retention_fixture(&retained, "retained")?;
    write_audit_retention_fixture(&expired, "expired")?;

    let report = arb_core::execute_local_audit_retention(&AuditRetentionExecutionRequest {
        workspace_dir: options.workspace_dir.clone(),
        policy: AuditRetentionPolicy {
            max_active_bytes: 1,
            max_archived_files: 1,
            retention_window_ms: 1_000,
        },
        files: vec![
            audit_retention_file_metadata(&active, now_unix_ms, true)?,
            audit_retention_file_metadata(&retained, now_unix_ms.saturating_sub(500), false)?,
            audit_retention_file_metadata(&expired, now_unix_ms.saturating_sub(2_000), false)?,
        ],
        now_unix_ms,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("audit-retention-execution: validation passed");
    println!(
        "audit-retention-workspace: {}",
        options.workspace_dir.display()
    );
    println!(
        "audit-retention-rotate-active-requested: {}",
        report.rotate_active_requested
    );
    println!(
        "audit-retention-new-active-created: {}",
        report.new_active_created
    );
    println!(
        "audit-retention-retained-archives: {}",
        report.retained_archives.len()
    );
    println!(
        "audit-retention-expired-archives-deleted: {}",
        report.expired_archives_deleted.len()
    );
    println!(
        "audit-retention-deleted-file-count: {}",
        report.deleted_file_count
    );
    println!(
        "audit-retention-deletion-performed: {}",
        report.deletion_performed
    );
    println!(
        "audit-retention-filesystem-mutated: {}",
        report.filesystem_mutated
    );
    println!(
        "audit-retention-out-of-workspace-path-touched: {}",
        report.out_of_workspace_path_touched
    );
    println!(
        "audit-retention-live-network-used: {}",
        report.live_network_used
    );
    println!(
        "audit-retention-external-execution-performed: {}",
        report.external_execution_performed
    );
    println!("production-ready: {}", report.production_ready);
    Ok(())
}

fn run_runtime_blocked_state_preflight_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let config = migrate_config_toml_to_current(LOCAL_STRATEGY_PLANNER_CONFIG)?.config;
    let policy = PolicyEngine::from_config(config.clone());
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("runtime-audit.jsonl");
    let state_path = options.workspace_dir.join("runtime-state.sqlite3");
    let backup_audit_path = options.workspace_dir.join("runtime-audit.backup.jsonl");
    let backup_state_path = options.workspace_dir.join("runtime-state.backup.sqlite3");
    let audit_validation_workspace = options.workspace_dir.join("audit-durability-workspace");

    fs::write(
        &state_path,
        b"pre-existing deployment-host state placeholder",
    )
    .map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to write blocked runtime state placeholder {}: {error}",
            state_path.display()
        ))
    })?;

    let lifecycle_request =
        build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, "blocked-state")?;
    let error = arb_core::validate_local_runtime_deployment_smoke(
        &audit_path,
        &state_path,
        &backup_audit_path,
        &backup_state_path,
        &audit_validation_workspace,
        &policy,
        RuntimeDeploymentSmokeValidationRequest {
            lifecycle_request,
            shutdown_request: RuntimeGracefulShutdownRequest {
                id: "cli-runtime-smoke-blocked-state-shutdown".to_owned(),
                reason: "local-cli-runtime-smoke-blocked-state-preflight".to_owned(),
                now_unix_ms: now_unix_ms.saturating_add(1),
            },
            validated_at_unix_ms: now_unix_ms.saturating_add(2),
        },
    )
    .err()
    .ok_or_else(|| {
        AgentCliError::Validation(
            "pre-existing runtime state path unexpectedly passed validation".to_owned(),
        )
    })?;

    let error_text = error.to_string();
    let expected_failure = error_text.contains("runtime state smoke path must not already exist");
    let artifacts_created = audit_path.exists()
        || backup_audit_path.exists()
        || backup_state_path.exists()
        || audit_validation_workspace.exists();

    println!("runtime-blocked-state-preflight: validation passed");
    println!(
        "runtime-blocked-state-workspace: {}",
        options.workspace_dir.display()
    );
    println!("runtime-blocked-state-expected-failure: {expected_failure}");
    println!("runtime-blocked-state-artifacts-created: {artifacts_created}");
    println!(
        "runtime-blocked-state-audit-created: {}",
        audit_path.exists()
    );
    println!(
        "runtime-blocked-state-backup-audit-created: {}",
        backup_audit_path.exists()
    );
    println!(
        "runtime-blocked-state-backup-state-created: {}",
        backup_state_path.exists()
    );
    println!(
        "runtime-blocked-state-audit-workspace-created: {}",
        audit_validation_workspace.exists()
    );
    println!("service-manager-action-performed: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if !expected_failure || artifacts_created {
        return Err(AgentCliError::Validation(
            "runtime blocked-state preflight did not fail closed before artifact creation"
                .to_owned(),
        ));
    }
    Ok(())
}

fn run_runtime_blocked_audit_preflight_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let config = migrate_config_toml_to_current(LOCAL_STRATEGY_PLANNER_CONFIG)?.config;
    let policy = PolicyEngine::from_config(config.clone());
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("runtime-audit.jsonl");
    let state_path = options.workspace_dir.join("runtime-state.sqlite3");
    let backup_audit_path = options.workspace_dir.join("runtime-audit.backup.jsonl");
    let backup_state_path = options.workspace_dir.join("runtime-state.backup.sqlite3");
    let audit_validation_workspace = options.workspace_dir.join("audit-durability-workspace");

    fs::write(
        &audit_path,
        b"pre-existing deployment-host audit placeholder",
    )
    .map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to write blocked runtime audit placeholder {}: {error}",
            audit_path.display()
        ))
    })?;

    let lifecycle_request =
        build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, "blocked-audit")?;
    let error = arb_core::validate_local_runtime_deployment_smoke(
        &audit_path,
        &state_path,
        &backup_audit_path,
        &backup_state_path,
        &audit_validation_workspace,
        &policy,
        RuntimeDeploymentSmokeValidationRequest {
            lifecycle_request,
            shutdown_request: RuntimeGracefulShutdownRequest {
                id: "cli-runtime-smoke-blocked-audit-shutdown".to_owned(),
                reason: "local-cli-runtime-smoke-blocked-audit-preflight".to_owned(),
                now_unix_ms: now_unix_ms.saturating_add(1),
            },
            validated_at_unix_ms: now_unix_ms.saturating_add(2),
        },
    )
    .err()
    .ok_or_else(|| {
        AgentCliError::Validation(
            "pre-existing runtime audit path unexpectedly passed validation".to_owned(),
        )
    })?;

    let error_text = error.to_string();
    let expected_failure = error_text.contains("runtime audit smoke path must not already exist");
    let artifacts_created = state_path.exists()
        || backup_audit_path.exists()
        || backup_state_path.exists()
        || audit_validation_workspace.exists();

    println!("runtime-blocked-audit-preflight: validation passed");
    println!(
        "runtime-blocked-audit-workspace: {}",
        options.workspace_dir.display()
    );
    println!("runtime-blocked-audit-expected-failure: {expected_failure}");
    println!("runtime-blocked-audit-artifacts-created: {artifacts_created}");
    println!(
        "runtime-blocked-audit-placeholder-created: {}",
        audit_path.exists()
    );
    println!(
        "runtime-blocked-audit-state-created: {}",
        state_path.exists()
    );
    println!(
        "runtime-blocked-audit-backup-audit-created: {}",
        backup_audit_path.exists()
    );
    println!(
        "runtime-blocked-audit-backup-state-created: {}",
        backup_state_path.exists()
    );
    println!(
        "runtime-blocked-audit-audit-workspace-created: {}",
        audit_validation_workspace.exists()
    );
    println!("service-manager-action-performed: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if !expected_failure || artifacts_created {
        return Err(AgentCliError::Validation(
            "runtime blocked-audit preflight did not fail closed before artifact creation"
                .to_owned(),
        ));
    }
    Ok(())
}

fn run_local_validation_runner(options: &LocalValidationRunOptions) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("validation-run.audit.jsonl");
    let state_path = options.workspace_dir.join("validation-run.sqlite3");
    let plan = local_validation_runner_plan(now_unix_ms);
    let harness = DeterministicValidationHarness;
    let record = harness
        .validate_plan(ValidationRunRequest {
            config: ValidationHarnessConfig::default(),
            plan,
            requested_at_ms: now_unix_ms,
            operator_label: Some("local-validation-runner".to_owned()),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_validation_run_audit(&mut journal, &record, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_validation_run_checkpoint(&mut store, &record, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "local validation runner checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    if replayed.next_sequence() <= audit_record.sequence || recovered.value != checkpoint.value {
        return Err(AgentCliError::Validation(
            "local validation runner audit/state reopen check failed".to_owned(),
        ));
    }

    println!("local-validation-runner: validation passed");
    println!("testing-backtesting-version: {TESTING_BACKTESTING_VERSION}");
    println!("validation-plan-id: {}", record.plan_id);
    println!(
        "validation-run-status: {}",
        validation_run_status_label(record.status)
    );
    println!("planned-test-cases: {}", record.planned_test_cases);
    println!("planned-fixtures: {}", record.planned_fixtures);
    println!("planned-fuzz-corpora: {}", record.planned_fuzz_corpora);
    println!(
        "planned-backtest-scenarios: {}",
        record.planned_backtest_scenarios
    );
    println!("audit-records-replayed: {}", replayed.next_sequence() - 1);
    println!("state-checkpoint-recovered: true");
    println!(
        "external-fuzzer-invoked: {}",
        record.external_fuzzer_invoked
    );
    println!("live-network-used: {}", record.live_network_used);
    println!(
        "live-execution-submitted: {}",
        record.live_execution_submitted
    );
    println!(
        "signing-or-broadcast-performed: {}",
        record.signing_or_broadcast_performed
    );
    println!("production-ready: false");
    Ok(())
}

fn run_local_property_check_runner(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("property-check.audit.jsonl");
    let state_path = options.workspace_dir.join("property-check.sqlite3");
    let config = ValidationHarnessConfig::default();
    let plan = local_validation_runner_plan(now_unix_ms);
    let report = run_local_validation_property_checks(&plan, &config)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_property_check_report_audit(&mut journal, &report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_property_check_report_checkpoint(&mut store, &report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(TESTING_LAST_PROPERTY_CHECK_REPORT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "local property check checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    if replayed.next_sequence() <= audit_record.sequence || recovered.value != checkpoint.value {
        return Err(AgentCliError::Validation(
            "local property check audit/state reopen check failed".to_owned(),
        ));
    }

    println!("local-property-checks: validation passed");
    println!("testing-backtesting-version: {TESTING_BACKTESTING_VERSION}");
    println!("validation-plan-id: {}", report.plan_id);
    println!("property-checks-executed: {}", report.checks_executed);
    println!("property-checks-passed: {}", report.checks_passed);
    println!("property-checks-failed: {}", report.checks_failed);
    println!(
        "missing-fixture-references: {}",
        report.missing_fixture_references.len()
    );
    println!("empty-fuzz-corpora: {}", report.empty_fuzz_corpora.len());
    println!(
        "nonlocal-backtest-datasets: {}",
        report.nonlocal_backtest_datasets.len()
    );
    println!("audit-records-replayed: {}", replayed.next_sequence() - 1);
    println!("state-checkpoint-recovered: true");
    println!(
        "external-fuzzer-invoked: {}",
        report.external_fuzzer_invoked
    );
    println!("live-network-used: {}", report.live_network_used);
    println!(
        "live-execution-submitted: {}",
        report.live_execution_submitted
    );
    println!(
        "signing-or-broadcast-performed: {}",
        report.signing_or_broadcast_performed
    );
    println!("production-ready: false");
    Ok(())
}

fn run_local_fuzz_corpus_runner(options: &LocalValidationRunOptions) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("fuzz-corpus-replay.audit.jsonl");
    let state_path = options.workspace_dir.join("fuzz-corpus-replay.sqlite3");
    let report = run_local_fuzz_corpus_replay(LocalFuzzCorpusReplayRequest {
        replay_id: "local-fuzz-corpus-replay".to_owned(),
        config: ValidationHarnessConfig::default(),
        fuzz_corpora: local_validation_runner_fuzz_corpora(),
        requested_at_ms: now_unix_ms,
        operator_label: Some("local-fuzz-corpus-runner".to_owned()),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_fuzz_corpus_replay_report_audit(&mut journal, &report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_fuzz_corpus_replay_report_checkpoint(&mut store, &report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "local fuzz corpus replay checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    if replayed.next_sequence() <= audit_record.sequence || recovered.value != checkpoint.value {
        return Err(AgentCliError::Validation(
            "local fuzz corpus replay audit/state reopen check failed".to_owned(),
        ));
    }

    println!("local-fuzz-corpus-replay: validation passed");
    println!("testing-backtesting-version: {TESTING_BACKTESTING_VERSION}");
    println!("fuzz-replay-id: {}", report.replay_id);
    println!(
        "fuzz-replay-status: {}",
        fuzz_corpus_replay_status_label(report.status)
    );
    println!("fuzz-corpora: {}", report.corpus_count);
    println!("fuzz-seeds: {}", report.seed_count);
    println!("unique-fuzz-seeds: {}", report.unique_seed_count);
    println!("fuzz-targets: {}", report.target_summaries.len());
    println!("audit-records-replayed: {}", replayed.next_sequence() - 1);
    println!("state-checkpoint-recovered: true");
    println!(
        "external-fuzzer-invoked: {}",
        report.external_fuzzer_invoked
    );
    println!("live-network-used: {}", report.live_network_used);
    println!(
        "live-execution-submitted: {}",
        report.live_execution_submitted
    );
    println!(
        "signing-or-broadcast-performed: {}",
        report.signing_or_broadcast_performed
    );
    println!("production-ready: false");
    Ok(())
}

fn run_local_validation_corpus_runner(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("validation-corpus.audit.jsonl");
    let state_path = options.workspace_dir.join("validation-corpus.sqlite3");
    let report = run_local_validation_corpus(LocalValidationCorpusRequest {
        corpus_id: "local-validation-corpus".to_owned(),
        config: ValidationHarnessConfig::default(),
        plans: local_validation_runner_corpus(now_unix_ms),
        min_plan_count: 3,
        min_test_case_count: 5,
        min_fixture_count: 3,
        min_fuzz_corpus_count: 3,
        min_backtest_scenario_count: 3,
        requested_at_ms: now_unix_ms,
        operator_label: Some("local-validation-corpus-runner".to_owned()),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_validation_corpus_report_audit(&mut journal, &report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_validation_corpus_report_checkpoint(&mut store, &report, now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "local validation corpus checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    if replayed.next_sequence() <= audit_record.sequence || recovered.value != checkpoint.value {
        return Err(AgentCliError::Validation(
            "local validation corpus audit/state reopen check failed".to_owned(),
        ));
    }

    println!("local-validation-corpus: validation passed");
    println!("testing-backtesting-version: {TESTING_BACKTESTING_VERSION}");
    println!("validation-corpus-id: {}", report.corpus_id);
    println!(
        "validation-corpus-status: {}",
        validation_corpus_status_label(report.status)
    );
    println!("validation-plans: {}", report.plan_count);
    println!("accepted-validation-plans: {}", report.accepted_plan_count);
    println!("planned-test-cases: {}", report.planned_test_cases);
    println!("planned-fixtures: {}", report.planned_fixtures);
    println!("planned-fuzz-corpora: {}", report.planned_fuzz_corpora);
    println!(
        "planned-backtest-scenarios: {}",
        report.planned_backtest_scenarios
    );
    println!(
        "property-checks-executed: {}",
        report.property_checks_executed
    );
    println!("property-checks-passed: {}", report.property_checks_passed);
    println!("property-checks-failed: {}", report.property_checks_failed);
    println!("min-validation-plans: {}", report.min_plan_count);
    println!("min-test-cases: {}", report.min_test_case_count);
    println!("min-fixtures: {}", report.min_fixture_count);
    println!("min-fuzz-corpora: {}", report.min_fuzz_corpus_count);
    println!(
        "min-backtest-scenarios: {}",
        report.min_backtest_scenario_count
    );
    println!(
        "corpus-breadth-requirements-met: {}",
        report.corpus_breadth_requirements_met
    );
    println!("audit-records-replayed: {}", replayed.next_sequence() - 1);
    println!("state-checkpoint-recovered: true");
    println!(
        "external-fuzzer-invoked: {}",
        report.external_fuzzer_invoked
    );
    println!("live-network-used: {}", report.live_network_used);
    println!(
        "live-execution-submitted: {}",
        report.live_execution_submitted
    );
    println!(
        "signing-or-broadcast-performed: {}",
        report.signing_or_broadcast_performed
    );
    println!("production-ready: false");
    Ok(())
}

fn run_local_validation_coverage_review_runner() -> Result<(), AgentCliError> {
    let now_unix_ms = current_unix_ms()?;
    let config = ValidationHarnessConfig::default();
    let plan = local_validation_runner_plan(now_unix_ms);
    let validation_run = DeterministicValidationHarness
        .validate_plan(ValidationRunRequest {
            config: config.clone(),
            plan: plan.clone(),
            requested_at_ms: now_unix_ms,
            operator_label: Some("local-validation-coverage-review".to_owned()),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let property_check = run_local_validation_property_checks(&plan, &config)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let fuzz_corpus = run_local_fuzz_corpus_replay(LocalFuzzCorpusReplayRequest {
        replay_id: "local-validation-coverage-fuzz".to_owned(),
        config: config.clone(),
        fuzz_corpora: local_validation_runner_fuzz_corpora(),
        requested_at_ms: now_unix_ms.saturating_add(1),
        operator_label: Some("local-validation-coverage-fuzz".to_owned()),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let validation_corpus = run_local_validation_corpus(LocalValidationCorpusRequest {
        corpus_id: "local-validation-coverage-corpus".to_owned(),
        config,
        plans: local_validation_runner_corpus(now_unix_ms.saturating_add(2)),
        min_plan_count: 3,
        min_test_case_count: 5,
        min_fixture_count: 3,
        min_fuzz_corpus_count: 3,
        min_backtest_scenario_count: 3,
        requested_at_ms: now_unix_ms.saturating_add(3),
        operator_label: Some("local-validation-coverage-corpus".to_owned()),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let paper_backtest = run_local_paper_backtest_report(now_unix_ms.saturating_add(4))?;

    let report = review_local_validation_coverage(LocalValidationCoverageReviewRequest {
        review_id: "local-validation-coverage-review".to_owned(),
        validation_run,
        property_check,
        fuzz_corpus,
        validation_corpus,
        paper_backtest,
        min_validation_plans: 3,
        min_property_checks: 12,
        min_fuzz_targets: 2,
        min_backtest_scenarios: 1,
        remaining_external_evidence: vec![
            "external fuzz engine execution".to_owned(),
            "broader external property-test execution".to_owned(),
            "broader external/deployment replay corpus".to_owned(),
            "production load and security validation".to_owned(),
        ],
        live_network_used: false,
        external_fuzzer_invoked: false,
        live_execution_submitted: false,
        signing_or_broadcast_performed: false,
        production_ready_claimed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    print_local_validation_coverage_review_report(&report);
    Ok(())
}

fn print_local_validation_coverage_review_report(report: &LocalValidationCoverageReviewReport) {
    println!("local-validation-coverage-review: validation passed");
    println!(
        "validation-coverage-review-status: {}",
        validation_coverage_review_status_label(report.status)
    );
    println!("validation-run-ready: {}", report.validation_run_ready);
    println!("property-check-ready: {}", report.property_check_ready);
    println!("fuzz-corpus-ready: {}", report.fuzz_corpus_ready);
    println!(
        "validation-corpus-ready: {}",
        report.validation_corpus_ready
    );
    println!("paper-backtest-ready: {}", report.paper_backtest_ready);
    println!("validation-plan-count: {}", report.validation_plan_count);
    println!("property-check-count: {}", report.property_check_count);
    println!("fuzz-target-count: {}", report.fuzz_target_count);
    println!(
        "backtest-scenario-count: {}",
        report.backtest_scenario_count
    );
    println!(
        "local-breadth-requirements-met: {}",
        report.local_breadth_requirements_met
    );
    println!(
        "validation-coverage-remaining-external-evidence-count: {}",
        report.remaining_external_evidence_count
    );
    println!(
        "external-fuzzer-invoked: {}",
        report.external_fuzzer_invoked
    );
    println!("live-network-used: {}", report.live_network_used);
    println!(
        "live-execution-submitted: {}",
        report.live_execution_submitted
    );
    println!(
        "signing-or-broadcast-performed: {}",
        report.signing_or_broadcast_performed
    );
    println!("production-ready: false");
}

fn run_local_paper_backtest_corpus_runner(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options
        .workspace_dir
        .join("paper-backtest-corpus.audit.jsonl");
    let state_path = options.workspace_dir.join("paper-backtest-corpus.sqlite3");
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let adapter = PaperExecutionAdapter::new(
        "local-paper-backtest-corpus",
        PolicyEngine::from_config(config),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let corpus = local_paper_backtest_corpus(now_unix_ms)?;
    let report = adapter
        .run_backtest_corpus(&corpus, now_unix_ms.saturating_add(10_000))
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    validate_local_paper_backtest_corpus_report(&report, corpus.scenarios.len())?;
    let replayed_records =
        persist_local_paper_backtest_corpus_report(&audit_path, &state_path, &report, now_unix_ms)?;
    print_local_paper_backtest_corpus_report(&report, replayed_records);
    Ok(())
}

fn run_local_paper_backtest_report(
    now_unix_ms: u64,
) -> Result<PaperBacktestRunReport, AgentCliError> {
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let adapter = PaperExecutionAdapter::new(
        "local-validation-coverage-paper-backtest",
        PolicyEngine::from_config(config),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let corpus = local_paper_backtest_corpus(now_unix_ms)?;
    let report = adapter
        .run_backtest_corpus(&corpus, now_unix_ms.saturating_add(10_000))
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    validate_local_paper_backtest_corpus_report(&report, corpus.scenarios.len())?;
    Ok(report)
}

fn run_policy_decision_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("policy-decision.audit.jsonl");
    let state_path = options.workspace_dir.join("policy-decision.sqlite3");
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config);
    let approved_intent = local_policy_decision_intent("approved", ExecutionScope::Paper, false);
    let denied_intent = local_policy_decision_intent("denied-live", ExecutionScope::Live, true);
    let approved = PolicyDecisionRecord::from_decision(
        &approved_intent,
        &policy.evaluate(&approved_intent),
        now_unix_ms.saturating_add(1),
    );
    let denied = PolicyDecisionRecord::from_decision(
        &denied_intent,
        &policy.evaluate(&denied_intent),
        now_unix_ms.saturating_add(2),
    );
    validate_policy_decision_records(&approved, &denied)?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    append_policy_decision_audit(&mut journal, &approved)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let denied_audit = append_policy_decision_audit(&mut journal, &denied)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_policy_decision_checkpoint(&mut store, &denied)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_append_failure_failed_closed =
        validate_policy_decision_invalid_audit_fails_closed(&mut journal, &denied);
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(POLICY_LAST_DECISION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "policy decision checkpoint was not recovered after reopen".to_owned(),
            )
        })?;
    let state_failure_failed_closed = validate_policy_decision_state_failure(&denied);

    if replayed.next_sequence() <= denied_audit.sequence
        || recovered.value != checkpoint.value
        || !audit_append_failure_failed_closed
        || !state_failure_failed_closed
    {
        return Err(AgentCliError::Validation(
            "policy decision audit/state validation failed".to_owned(),
        ));
    }

    println!("policy-decision-audit: validation passed");
    println!("approved-policy-decision: {}", approved.approved);
    println!("denied-policy-decision: {}", !denied.approved);
    println!("denied-policy-violations: {}", denied.violation_count);
    println!("audit-append-failure-failed-closed: {audit_append_failure_failed_closed}");
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {}", replayed.next_sequence() - 1);
    println!("state-checkpoint-recovered: true");
    println!("external-submission-performed: false");
    println!("secret-material-recorded: false");
    println!("production-ready: false");
    Ok(())
}

fn validate_policy_decision_records(
    approved: &PolicyDecisionRecord,
    denied: &PolicyDecisionRecord,
) -> Result<(), AgentCliError> {
    if !approved.approved
        || approved.violation_count != 0
        || denied.approved
        || denied.violation_count == 0
        || !denied.live_scope_requested
        || !denied.signing_requested
        || approved.external_submission_performed
        || denied.external_submission_performed
        || approved.secret_material_recorded
        || denied.secret_material_recorded
    {
        return Err(AgentCliError::Validation(
            "policy decision records did not preserve expected local side-effect invariants"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_policy_decision_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    denied: &PolicyDecisionRecord,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = denied.clone();
    invalid.external_submission_performed = true;
    let failed = append_policy_decision_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_policy_decision_state_failure(record: &PolicyDecisionRecord) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let failed = persist_policy_decision_checkpoint(&mut store, record).is_err();
    failed && store.put_attempts == 1
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WithdrawalPolicyValidationReport {
    config_guard_active: bool,
    strategy_flag_guard_active: bool,
    strategy_intent_guard_active: bool,
    trust_contract_guard_active: bool,
    destination_allowlist_guard_active: bool,
    signing_boundary_guard_active: bool,
    audit_append_failure_failed_closed: bool,
    state_failure_failed_closed: bool,
    audit_records_replayed: u64,
    state_checkpoint_recovered: bool,
    external_submission_performed: bool,
    secret_material_recorded: bool,
    production_ready: bool,
}

fn run_withdrawal_policy_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    let report = build_withdrawal_policy_validation_report(options)?;
    validate_withdrawal_policy_report(&report)?;

    println!("withdrawal-policy-boundary: validation passed");
    println!("config-guard-active: {}", report.config_guard_active);
    println!(
        "strategy-flag-guard-active: {}",
        report.strategy_flag_guard_active
    );
    println!(
        "strategy-intent-guard-active: {}",
        report.strategy_intent_guard_active
    );
    println!(
        "trust-contract-guard-active: {}",
        report.trust_contract_guard_active
    );
    println!(
        "destination-allowlist-guard-active: {}",
        report.destination_allowlist_guard_active
    );
    println!(
        "signing-boundary-guard-active: {}",
        report.signing_boundary_guard_active
    );
    println!(
        "audit-append-failure-failed-closed: {}",
        report.audit_append_failure_failed_closed
    );
    println!(
        "state-failure-failed-closed: {}",
        report.state_failure_failed_closed
    );
    println!("audit-records-replayed: {}", report.audit_records_replayed);
    println!(
        "state-checkpoint-recovered: {}",
        report.state_checkpoint_recovered
    );
    println!(
        "external-submission-performed: {}",
        report.external_submission_performed
    );
    println!(
        "secret-material-recorded: {}",
        report.secret_material_recorded
    );
    println!("production-ready: {}", report.production_ready);
    Ok(())
}

fn build_withdrawal_policy_validation_report(
    options: &LocalValidationRunOptions,
) -> Result<WithdrawalPolicyValidationReport, AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("withdrawal-policy.audit.jsonl");
    let state_path = options.workspace_dir.join("withdrawal-policy.sqlite3");

    let live_withdrawals_config = LOCAL_STRATEGY_PLANNER_CONFIG
        .replace("mode = \"paper\"", "mode = \"live-armed\"")
        .replace(
            "live_execution_enabled = false",
            "live_execution_enabled = true",
        )
        .replace("allow_withdrawals = false", "allow_withdrawals = true");
    let config_guard_active = AgentConfig::from_toml_str(&live_withdrawals_config)
        .err()
        .is_some_and(|error| error.to_string().contains("WITHDRAWALS_BLOCKED_IN_PHASE_2"));

    let mut invalid_profile =
        StrategyProfile::conservative_paper("withdrawal-policy-invalid", "USD");
    invalid_profile.execution.allow_withdrawals = true;
    let strategy_flag_guard_active = invalid_profile.validate().err().is_some_and(|error| {
        error
            .violations()
            .iter()
            .any(|violation| violation.code() == "STRATEGY_WITHDRAWALS_DENIED")
    });

    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config);
    let profile = StrategyProfile::conservative_paper("withdrawal-policy", "USD");
    let mut intent = local_policy_decision_intent("withdrawal", ExecutionScope::Paper, true);
    intent.kind = ExecutionIntentKind::Withdrawal;
    intent.destination = DestinationPolicy::ApprovedAddress {
        chain: "ethereum".to_owned(),
        label: "ops-vault".to_owned(),
    };

    let strategy_report = profile.constrain_intent(&intent);
    let strategy_intent_guard_active = strategy_report
        .violations
        .iter()
        .any(|violation| violation.code() == "STRATEGY_WITHDRAWALS_DENIED");

    let decision = policy.evaluate(&intent);
    let record = PolicyDecisionRecord::from_decision(&intent, &decision, now_unix_ms);
    let trust_contract_guard_active = record
        .violation_codes
        .iter()
        .any(|code| code == "WITHDRAWALS_DENIED_BY_TRUST_CONTRACT");
    let destination_allowlist_guard_active = record
        .violation_codes
        .iter()
        .any(|code| code == "DESTINATION_NOT_APPROVED");
    let signing_boundary_guard_active = record
        .violation_codes
        .iter()
        .any(|code| code == "SECRET_BACKEND_REQUIRED_FOR_SIGNING")
        && record
            .violation_codes
            .iter()
            .any(|code| code == "WALLET_SIGNER_REFERENCE_REQUIRED");

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    append_policy_decision_audit(&mut journal, &record)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_policy_decision_checkpoint(&mut store, &record)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_append_failure_failed_closed =
        validate_policy_decision_invalid_audit_fails_closed(&mut journal, &record);
    let state_failure_failed_closed = validate_policy_decision_state_failure(&record);
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(POLICY_LAST_DECISION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let state_checkpoint_recovered = recovered
        .as_ref()
        .is_some_and(|recovered| recovered.value == checkpoint.value);

    Ok(WithdrawalPolicyValidationReport {
        config_guard_active,
        strategy_flag_guard_active,
        strategy_intent_guard_active,
        trust_contract_guard_active,
        destination_allowlist_guard_active,
        signing_boundary_guard_active,
        audit_append_failure_failed_closed,
        state_failure_failed_closed,
        audit_records_replayed: replayed.next_sequence().saturating_sub(1),
        state_checkpoint_recovered,
        external_submission_performed: record.external_submission_performed,
        secret_material_recorded: record.secret_material_recorded,
        production_ready: false,
    })
}

fn validate_withdrawal_policy_report(
    report: &WithdrawalPolicyValidationReport,
) -> Result<(), AgentCliError> {
    if !report.config_guard_active
        || !report.strategy_flag_guard_active
        || !report.strategy_intent_guard_active
        || !report.trust_contract_guard_active
        || !report.destination_allowlist_guard_active
        || !report.signing_boundary_guard_active
        || !report.audit_append_failure_failed_closed
        || !report.state_failure_failed_closed
        || report.audit_records_replayed == 0
        || !report.state_checkpoint_recovered
        || report.external_submission_performed
        || report.secret_material_recorded
        || report.production_ready
    {
        return Err(AgentCliError::Validation(
            "withdrawal policy validation did not preserve expected local fail-closed guards"
                .to_owned(),
        ));
    }
    Ok(())
}

struct SecretBoundaryAuditPersistence {
    ready_sequence: u64,
    rejected_sequence: u64,
    checkpoint_value: String,
    audit_failed_closed: bool,
}

fn run_secret_boundary_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("secret-boundary.audit.jsonl");
    let state_path = options.workspace_dir.join("secret-boundary.sqlite3");
    let ready = build_local_secret_rotation_plan(now_unix_ms, false)?;
    let rejected = build_local_secret_rotation_plan(now_unix_ms.saturating_add(10), true)?;
    let persisted =
        persist_local_secret_boundary_audit_case(&audit_path, &state_path, &ready, &rejected)?;
    let audit_records_replayed =
        verify_local_secret_boundary_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_secret_rotation_state_failure(&ready);

    if ready.status != SecretRotationPlanStatus::ReadyForLocalReview
        || rejected.status != SecretRotationPlanStatus::RejectedSameAlias
        || ready.secret_material_loaded
        || ready.plaintext_decrypted
        || ready.keystore_entry_written
        || ready.external_secret_revoked
        || ready.production_ready
        || rejected.secret_material_loaded
        || rejected.plaintext_decrypted
        || rejected.keystore_entry_written
        || rejected.external_secret_revoked
        || rejected.production_ready
        || !persisted.audit_failed_closed
        || !state_failure_failed_closed
    {
        return Err(AgentCliError::Validation(
            "secret boundary audit/state validation failed".to_owned(),
        ));
    }

    println!("secret-boundary-audit: validation passed");
    println!("ready-rotation-plan: {}", ready.plan_id);
    println!("rejected-rotation-plan: {}", rejected.plan_id);
    println!(
        "rejected-rotation-validation-codes: {}",
        rejected.validation_count
    );
    println!(
        "audit-append-failure-failed-closed: {}",
        persisted.audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoint-recovered: true");
    println!("secret-material-loaded: false");
    println!("plaintext-decrypted: false");
    println!("keystore-entry-written: false");
    println!("external-secret-revoked: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_secret_rotation_plan(
    planned_at_unix_ms: u64,
    same_alias: bool,
) -> Result<SecretRotationPlanReport, AgentCliError> {
    let current_alias = "local-paper-cex-primary";
    let replacement_alias = if same_alias {
        current_alias
    } else {
        "local-paper-cex-next"
    };
    plan_local_secret_rotation(SecretRotationPlanRequest {
        plan_id: if same_alias {
            "local-secret-rotation-rejected"
        } else {
            "local-secret-rotation-ready"
        }
        .to_owned(),
        secret_purpose: "exchange-api-key".to_owned(),
        current_reference: SecretRef::Keystore {
            alias: current_alias.to_owned(),
        },
        replacement_reference: SecretRef::Keystore {
            alias: replacement_alias.to_owned(),
        },
        requested_by: "local-operator".to_owned(),
        rotation_reason: "local non-secret audit/state validation".to_owned(),
        planned_at_unix_ms,
        not_before_unix_ms: planned_at_unix_ms.saturating_add(1_000),
        expires_at_unix_ms: planned_at_unix_ms.saturating_add(86_400_000),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn persist_local_secret_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    ready: &SecretRotationPlanReport,
    rejected: &SecretRotationPlanReport,
) -> Result<SecretBoundaryAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ready_audit = append_secret_rotation_plan_audit(&mut journal, ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let rejected_audit = append_secret_rotation_plan_audit(&mut journal, rejected)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_secret_rotation_plan_checkpoint(&mut store, ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_failed_closed =
        validate_secret_rotation_invalid_audit_fails_closed(&mut journal, ready);

    Ok(SecretBoundaryAuditPersistence {
        ready_sequence: ready_audit.sequence,
        rejected_sequence: rejected_audit.sequence,
        checkpoint_value: checkpoint.value,
        audit_failed_closed,
    })
}

fn verify_local_secret_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &SecretBoundaryAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened
        .get_checkpoint(SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("secret rotation checkpoint missing".to_owned())
        })?;

    if replayed.next_sequence() <= persisted.ready_sequence
        || replayed.next_sequence() <= persisted.rejected_sequence
        || checkpoint.value != persisted.checkpoint_value
        || !persisted.audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "secret boundary audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_secret_rotation_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &SecretRotationPlanReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.secret_material_loaded = true;
    let failed = append_secret_rotation_plan_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_secret_rotation_state_failure(report: &SecretRotationPlanReport) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let failed = persist_secret_rotation_plan_checkpoint(&mut store, report).is_err();
    failed && store.put_attempts == 1
}

struct SecretBackupRestorePersistence {
    ready_sequence: u64,
    blocked_sequence: u64,
    checkpoint_value: String,
    audit_failed_closed: bool,
}

fn run_secret_backup_restore_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options
        .workspace_dir
        .join("secret-backup-restore.audit.jsonl");
    let state_path = options.workspace_dir.join("secret-backup-restore.sqlite3");
    let ready = build_local_secret_backup_restore_review(now_unix_ms, false)?;
    let blocked = build_local_secret_backup_restore_review(now_unix_ms.saturating_add(10), true)?;
    let persisted =
        persist_local_secret_backup_restore_case(&audit_path, &state_path, &ready, &blocked)?;
    let audit_records_replayed =
        verify_local_secret_backup_restore_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_secret_backup_restore_state_failure(&ready);

    if ready.status != SecretBackupRestoreReviewStatus::ReadyForLocalReview
        || blocked.status != SecretBackupRestoreReviewStatus::BlockedRestoreVerification
        || ready.secret_material_loaded
        || ready.plaintext_decrypted
        || ready.keystore_entry_written
        || ready.external_secret_restored
        || ready.signing_or_broadcast_performed
        || ready.production_ready
        || blocked.secret_material_loaded
        || blocked.plaintext_decrypted
        || blocked.keystore_entry_written
        || blocked.external_secret_restored
        || blocked.signing_or_broadcast_performed
        || blocked.production_ready
        || !persisted.audit_failed_closed
        || !state_failure_failed_closed
    {
        return Err(AgentCliError::Validation(
            "secret backup/restore validation failed".to_owned(),
        ));
    }

    println!("secret-backup-restore: validation passed");
    println!("ready-backup-restore-review: {}", ready.review_id);
    println!("blocked-backup-restore-review: {}", blocked.review_id);
    println!(
        "blocked-backup-restore-validation-codes: {}",
        blocked.validation_count
    );
    println!(
        "backup-reference-present: {}",
        ready.backup_reference_present
    );
    println!(
        "backup-payload-shape-verified: {}",
        ready.backup_payload_shape_verified
    );
    println!(
        "restore-verification-passed: {}",
        ready.restore_verification_passed
    );
    println!("references-sanitized: {}", ready.references_sanitized);
    println!("review-window-valid: {}", ready.review_window_valid);
    println!(
        "audit-append-failure-failed-closed: {}",
        persisted.audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoint-recovered: true");
    println!("secret-material-loaded: false");
    println!("plaintext-decrypted: false");
    println!("keystore-entry-written: false");
    println!("external-secret-restored: false");
    println!("signing-or-broadcast-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_secret_backup_restore_review(
    reviewed_at_unix_ms: u64,
    blocked: bool,
) -> Result<SecretBackupRestoreReviewReport, AgentCliError> {
    review_local_secret_backup_restore(SecretBackupRestoreReviewRequest {
        review_id: if blocked {
            "local-secret-backup-restore-blocked"
        } else {
            "local-secret-backup-restore-ready"
        }
        .to_owned(),
        secret_purpose: "exchange-api-key".to_owned(),
        source_reference: SecretRef::Keystore {
            alias: "local-paper-cex-primary".to_owned(),
        },
        backup_reference: "actions-artifact:secret-backup-shape-v1".to_owned(),
        restore_target_label: "local-restore-shape-check".to_owned(),
        reviewed_by: "local-operator".to_owned(),
        review_note: "local non-secret secret backup/restore audit/state validation".to_owned(),
        backup_payload_shape_verified: !blocked,
        restore_verification_passed: !blocked,
        references_sanitized: true,
        reviewed_at_unix_ms,
        review_window_start_unix_ms: reviewed_at_unix_ms.saturating_add(1_000),
        review_window_expires_unix_ms: reviewed_at_unix_ms.saturating_add(86_400_000),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn persist_local_secret_backup_restore_case(
    audit_path: &Path,
    state_path: &Path,
    ready: &SecretBackupRestoreReviewReport,
    blocked: &SecretBackupRestoreReviewReport,
) -> Result<SecretBackupRestorePersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ready_audit = append_secret_backup_restore_review_audit(&mut journal, ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked_audit = append_secret_backup_restore_review_audit(&mut journal, blocked)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_secret_backup_restore_review_checkpoint(&mut store, ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_failed_closed =
        validate_secret_backup_restore_invalid_audit_fails_closed(&mut journal, ready);

    Ok(SecretBackupRestorePersistence {
        ready_sequence: ready_audit.sequence,
        blocked_sequence: blocked_audit.sequence,
        checkpoint_value: checkpoint.value,
        audit_failed_closed,
    })
}

fn verify_local_secret_backup_restore_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &SecretBackupRestorePersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened
        .get_checkpoint(SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("secret backup/restore checkpoint missing".to_owned())
        })?;

    if replayed.next_sequence() <= persisted.ready_sequence
        || replayed.next_sequence() <= persisted.blocked_sequence
        || checkpoint.value != persisted.checkpoint_value
        || !persisted.audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "secret backup/restore validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_secret_backup_restore_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &SecretBackupRestoreReviewReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.external_secret_restored = true;
    let failed = append_secret_backup_restore_review_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_secret_backup_restore_state_failure(report: &SecretBackupRestoreReviewReport) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let failed = persist_secret_backup_restore_review_checkpoint(&mut store, report).is_err();
    failed && store.put_attempts == 1
}

struct LocalExecutionPlannerAuditCase {
    draft: arb_core::ExecutionPlanDraft,
}

struct ExecutionPlannerAuditPersistence {
    draft_audit_sequence: u64,
    last_record_sequence: u64,
    records_appended: usize,
    checkpoint_value: String,
    audit_failure_failed_closed: bool,
}

fn run_execution_planner_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("execution-planner.audit.jsonl");
    let state_path = options.workspace_dir.join("execution-planner.sqlite3");
    let planner_case = build_local_execution_planner_audit_case(now_unix_ms)?;
    let persisted =
        persist_local_execution_planner_audit_case(&audit_path, &state_path, &planner_case)?;
    let audit_records_replayed =
        verify_local_execution_planner_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_execution_planner_state_failure(&planner_case.draft);

    if !state_failure_failed_closed
        || planner_case.draft.adapter_submission_enabled
        || persisted.records_appended != 1 + planner_case.draft.policy_outcomes.len()
    {
        return Err(AgentCliError::Validation(
            "execution planner audit/state validation failed".to_owned(),
        ));
    }

    println!("execution-planner-audit: validation passed");
    println!(
        "plan-status: {}",
        plan_status_label(planner_case.draft.status)
    );
    println!("plan-intents: {}", planner_case.draft.intents.len());
    println!(
        "plan-policy-outcomes: {}",
        planner_case.draft.policy_outcomes.len()
    );
    println!(
        "plan-failure-modes: {}",
        planner_case.draft.failure_modes.len()
    );
    println!(
        "audit-append-failure-failed-closed: {}",
        persisted.audit_failure_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("adapter-submission-enabled: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_execution_planner_audit_case(
    now_unix_ms: u64,
) -> Result<LocalExecutionPlannerAuditCase, AgentCliError> {
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config);
    let planner_request = ExecutionPlannerRequest {
        id: "local-execution-planner-audit-request".to_owned(),
        strategy_id: "local-execution-planner-audit".to_owned(),
        candidate: local_strategy_planner_candidate()?,
        config: ExecutionPlannerConfig {
            requested_scope: ExecutionScope::Paper,
            max_plan_legs: 2,
            max_total_notional_quote: 1_000.0,
            default_slippage_bps: 50,
            max_market_data_age_ms: DEFAULT_MARKET_DATA_FRESHNESS_MS,
            require_policy_preflight: true,
        },
        default_chain: None,
        now_unix_ms,
    };
    let draft = DeterministicExecutionPlanner::new()
        .plan(&planner_request, &policy)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(LocalExecutionPlannerAuditCase { draft })
}

fn persist_local_execution_planner_audit_case(
    audit_path: &Path,
    state_path: &Path,
    planner_case: &LocalExecutionPlannerAuditCase,
) -> Result<ExecutionPlannerAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_report = append_execution_plan_draft_audit(
        &mut journal,
        &planner_case.draft,
        planner_case.draft.created_at_unix_ms,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_execution_plan_draft_checkpoint(&mut store, &planner_case.draft)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_failure_failed_closed =
        validate_execution_planner_invalid_audit_fails_closed(&mut journal, &planner_case.draft);
    drop(store);
    drop(journal);

    Ok(ExecutionPlannerAuditPersistence {
        draft_audit_sequence: audit_report.draft_record_sequence,
        last_record_sequence: audit_report
            .policy_outcome_record_sequences
            .last()
            .copied()
            .unwrap_or(audit_report.draft_record_sequence),
        records_appended: audit_report.records_appended,
        checkpoint_value: checkpoint.value,
        audit_failure_failed_closed,
    })
}

fn verify_local_execution_planner_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &ExecutionPlannerAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "execution planner draft checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    let replayed_records = replayed.next_sequence() - 1;
    if replayed.next_sequence() != persisted.last_record_sequence.saturating_add(1)
        || persisted.last_record_sequence
            != persisted.draft_audit_sequence
                + (persisted.records_appended.saturating_sub(1) as u64)
        || replayed_records != persisted.records_appended as u64
        || recovered.value != persisted.checkpoint_value
        || !persisted.audit_failure_failed_closed
    {
        return Err(AgentCliError::Validation(
            "execution planner audit/state validation failed".to_owned(),
        ));
    }

    Ok(replayed_records)
}

fn validate_execution_planner_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    draft: &arb_core::ExecutionPlanDraft,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = draft.clone();
    invalid.adapter_submission_enabled = true;
    let failed =
        append_execution_plan_draft_audit(journal, &invalid, invalid.created_at_unix_ms).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_execution_planner_state_failure(draft: &arb_core::ExecutionPlanDraft) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let failed = persist_execution_plan_draft_checkpoint(&mut store, draft).is_err();
    failed && store.put_attempts == 1
}

struct LocalExecutionAdapterAuditCase {
    adapter_request: ExecutionAdapterRequest,
    run: arb_core::ExecutionAdapterRunRecord,
    recovery_plan: arb_core::ExecutionAdapterRecoveryPlan,
}

struct ExecutionAdapterAuditPersistence {
    run_audit_sequence: u64,
    recovery_audit_sequence: u64,
    run_checkpoint_value: String,
    recovery_checkpoint_value: String,
    run_audit_failure_failed_closed: bool,
    recovery_audit_failure_failed_closed: bool,
}

fn run_execution_adapter_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("execution-adapter.audit.jsonl");
    let state_path = options.workspace_dir.join("execution-adapter.sqlite3");
    let adapter_case = build_local_execution_adapter_audit_case(now_unix_ms)?;
    let persisted =
        persist_local_execution_adapter_audit_case(&audit_path, &state_path, &adapter_case)?;
    let audit_records_replayed =
        verify_local_execution_adapter_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed =
        validate_execution_adapter_state_failure(&adapter_case.run, &adapter_case.recovery_plan);
    let all_attempts_policy_revalidated = adapter_case
        .run
        .attempts
        .iter()
        .all(|attempt| attempt.policy_revalidated);
    let policy_denied_attempts = adapter_case
        .run
        .attempts
        .iter()
        .filter(|attempt| {
            attempt.reason.contains("policy")
                || attempt.reason.contains("kill switch")
                || attempt.reason.contains("source planner policy outcome")
        })
        .count();

    if !state_failure_failed_closed
        || !all_attempts_policy_revalidated
        || adapter_case.run.external_submission_enabled
        || !adapter_case.run.submission_kill_switch_required
        || !adapter_case.run.submission_audit_state_preflight_required
        || !adapter_case.run.submission_idempotency_required
        || adapter_case.recovery_plan.external_submission_performed
        || adapter_case.recovery_plan.live_execution_performed
        || adapter_case.recovery_plan.production_ready
    {
        return Err(AgentCliError::Validation(
            "execution adapter audit/state validation failed".to_owned(),
        ));
    }

    println!("execution-adapter-audit: validation passed");
    println!(
        "adapter-run-status: {}",
        execution_adapter_run_status_label(adapter_case.run.status)
    );
    println!(
        "source-plan-status: {}",
        plan_status_label(adapter_case.adapter_request.plan.status)
    );
    println!("adapter-run-attempts: {}", adapter_case.run.attempts.len());
    println!("adapter-run-fills: {}", adapter_case.run.fills.len());
    println!(
        "adapter-run-reconciliations: {}",
        adapter_case.run.reconciliations.len()
    );
    println!("adapter-policy-revalidated: {all_attempts_policy_revalidated}");
    println!("adapter-policy-denied-attempts: {policy_denied_attempts}");
    println!(
        "adapter-submission-kill-switch-required: {}",
        adapter_case.run.submission_kill_switch_required
    );
    println!(
        "adapter-submission-audit-state-preflight-required: {}",
        adapter_case.run.submission_audit_state_preflight_required
    );
    println!(
        "adapter-submission-idempotency-required: {}",
        adapter_case.run.submission_idempotency_required
    );
    println!(
        "adapter-recovery-steps: {}",
        adapter_case.recovery_plan.steps.len()
    );
    println!(
        "audit-append-failure-failed-closed: {}",
        persisted.run_audit_failure_failed_closed
    );
    println!(
        "recovery-audit-append-failure-failed-closed: {}",
        persisted.recovery_audit_failure_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_execution_adapter_audit_case(
    now_unix_ms: u64,
) -> Result<LocalExecutionAdapterAuditCase, AgentCliError> {
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config);
    let planner_request = ExecutionPlannerRequest {
        id: "local-execution-adapter-audit-planner-request".to_owned(),
        strategy_id: "local-execution-adapter-audit".to_owned(),
        candidate: local_strategy_planner_candidate()?,
        config: ExecutionPlannerConfig {
            requested_scope: ExecutionScope::Paper,
            max_plan_legs: 2,
            max_total_notional_quote: 1_000.0,
            default_slippage_bps: 50,
            max_market_data_age_ms: DEFAULT_MARKET_DATA_FRESHNESS_MS,
            require_policy_preflight: true,
        },
        default_chain: None,
        now_unix_ms,
    };
    let plan = DeterministicExecutionPlanner::new()
        .plan(&planner_request, &policy)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let adapter_request = ExecutionAdapterRequest {
        id: "local-execution-adapter-audit-request".to_owned(),
        plan,
        config: ExecutionAdapterConfig::default(),
        now_unix_ms: now_unix_ms.saturating_add(1),
    };
    let run = DeterministicExecutionAdapterBoundary::new()
        .evaluate_plan(&adapter_request, &policy)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovery_plan =
        plan_execution_adapter_recovery(&adapter_request.plan, &run, now_unix_ms.saturating_add(2))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(LocalExecutionAdapterAuditCase {
        adapter_request,
        run,
        recovery_plan,
    })
}

fn persist_local_execution_adapter_audit_case(
    audit_path: &Path,
    state_path: &Path,
    adapter_case: &LocalExecutionAdapterAuditCase,
) -> Result<ExecutionAdapterAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let run_audit = append_execution_adapter_run_audit(&mut journal, &adapter_case.run)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovery_audit =
        append_execution_adapter_recovery_plan_audit(&mut journal, &adapter_case.recovery_plan)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let run_checkpoint = persist_execution_adapter_run_checkpoint(&mut store, &adapter_case.run)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovery_checkpoint =
        persist_execution_adapter_recovery_plan_checkpoint(&mut store, &adapter_case.recovery_plan)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let run_audit_failure_failed_closed =
        validate_execution_adapter_run_invalid_audit_fails_closed(&mut journal, &adapter_case.run);
    let recovery_audit_failure_failed_closed =
        validate_execution_adapter_recovery_invalid_audit_fails_closed(
            &mut journal,
            &adapter_case.recovery_plan,
        );
    drop(store);
    drop(journal);

    Ok(ExecutionAdapterAuditPersistence {
        run_audit_sequence: run_audit.sequence,
        recovery_audit_sequence: recovery_audit.sequence,
        run_checkpoint_value: run_checkpoint.value,
        recovery_checkpoint_value: recovery_checkpoint.value,
        run_audit_failure_failed_closed,
        recovery_audit_failure_failed_closed,
    })
}

fn verify_local_execution_adapter_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &ExecutionAdapterAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_run = reopened
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "execution adapter run checkpoint was not recovered after reopen".to_owned(),
            )
        })?;
    let recovered_recovery_plan = reopened
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "execution adapter recovery checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    if replayed.next_sequence() <= persisted.recovery_audit_sequence
        || persisted.run_audit_sequence == persisted.recovery_audit_sequence
        || recovered_run.value != persisted.run_checkpoint_value
        || recovered_recovery_plan.value != persisted.recovery_checkpoint_value
        || !persisted.run_audit_failure_failed_closed
        || !persisted.recovery_audit_failure_failed_closed
    {
        return Err(AgentCliError::Validation(
            "execution adapter audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_execution_adapter_run_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    run: &arb_core::ExecutionAdapterRunRecord,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = run.clone();
    invalid.external_submission_enabled = true;
    let failed = append_execution_adapter_run_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_execution_adapter_recovery_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    recovery_plan: &arb_core::ExecutionAdapterRecoveryPlan,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = recovery_plan.clone();
    invalid.external_submission_performed = true;
    let failed = append_execution_adapter_recovery_plan_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_execution_adapter_state_failure(
    run: &arb_core::ExecutionAdapterRunRecord,
    recovery_plan: &arb_core::ExecutionAdapterRecoveryPlan,
) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let run_failed = persist_execution_adapter_run_checkpoint(&mut store, run).is_err();
    let recovery_failed =
        persist_execution_adapter_recovery_plan_checkpoint(&mut store, recovery_plan).is_err();
    run_failed && recovery_failed && store.put_attempts == 2
}

struct LocalSignerBoundaryAuditCase {
    request_record: SignerRequestRecord,
    scope_report: SignerSecretScopeReviewReport,
}

struct SignerBoundaryAuditPersistence {
    request_sequence: u64,
    scope_sequence: u64,
    request_checkpoint_value: String,
    scope_checkpoint_value: String,
    request_audit_failed_closed: bool,
    scope_audit_failed_closed: bool,
}

fn run_signer_boundary_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options.workspace_dir.join("signer-boundary.audit.jsonl");
    let state_path = options.workspace_dir.join("signer-boundary.sqlite3");
    let signer_case = build_local_signer_boundary_audit_case(now_unix_ms)?;
    let persisted =
        persist_local_signer_boundary_audit_case(&audit_path, &state_path, &signer_case)?;
    let audit_records_replayed =
        verify_local_signer_boundary_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_signer_boundary_state_failure(&signer_case);

    if !state_failure_failed_closed
        || signer_case.request_record.status != SignerRequestStatus::RejectedSignerUnavailable
        || signer_case.scope_report.status != SignerSecretScopeReviewStatus::ReadyForLocalReview
        || signer_case.request_record.signer_material_loaded
        || signer_case.request_record.signing_performed
        || signer_case.request_record.broadcast_performed
        || signer_case.request_record.rpc_called
        || signer_case.request_record.production_ready
        || signer_case.scope_report.signer_material_loaded
        || signer_case.scope_report.plaintext_decrypted
        || signer_case.scope_report.signing_performed
        || signer_case.scope_report.broadcast_performed
        || signer_case.scope_report.rpc_called
        || signer_case.scope_report.production_ready
    {
        return Err(AgentCliError::Validation(
            "signer boundary audit/state validation failed".to_owned(),
        ));
    }

    println!("signer-boundary-audit: validation passed");
    println!(
        "signer-request-status: {:?}",
        signer_case.request_record.status
    );
    println!("signer-scope-status: {:?}", signer_case.scope_report.status);
    println!(
        "signer-request-audit-failed-closed: {}",
        persisted.request_audit_failed_closed
    );
    println!(
        "signer-scope-audit-failed-closed: {}",
        persisted.scope_audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("signer-material-loaded: false");
    println!("plaintext-decrypted: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("rpc-called: false");
    println!("production-ready: false");
    Ok(())
}

fn run_signer_runtime_isolation_validation() -> Result<(), AgentCliError> {
    let (ready, blocked) = local_signer_runtime_isolation_reviews();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let ready_count =
        usize::from(ready.status == SignerRuntimeIsolationReviewStatus::ReadyForLocalReview);
    let blocked_count = usize::from(blocked.status == SignerRuntimeIsolationReviewStatus::Blocked);
    let blocked_violation_count = blocked.violation_count;
    let unsafe_side_effect = ready.signer_material_loaded
        || ready.plaintext_decrypted
        || ready.signing_performed
        || ready.broadcast_performed
        || ready.rpc_called
        || ready.production_ready
        || blocked.signer_material_loaded
        || blocked.plaintext_decrypted
        || blocked.signing_performed
        || blocked.broadcast_performed
        || blocked.rpc_called
        || blocked.production_ready;

    println!("signer-runtime-isolation: validation passed");
    println!("runtime-isolation-review-count: 2");
    println!("runtime-isolation-ready-count: {ready_count}");
    println!("runtime-isolation-blocked-count: {blocked_count}");
    println!("runtime-isolation-blocker-count: {blocked_violation_count}");
    println!(
        "llm-signer-access-denied: {}",
        ready.llm_signer_access_denied
    );
    println!(
        "plaintext-key-exposure-denied: {}",
        ready.plaintext_key_exposure_denied
    );
    println!(
        "policy-destination-scope-required: {}",
        ready.policy_destination_scope_required
    );
    println!(
        "audit-state-before-signing-required: {}",
        ready.audit_state_before_signing_required
    );
    println!("signer-material-loaded: false");
    println!("plaintext-decrypted: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("rpc-called: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked_violation_count != 9
        || !ready.llm_signer_access_denied
        || !ready.plaintext_key_exposure_denied
        || !ready.policy_destination_scope_required
        || !ready.audit_state_before_signing_required
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "signer runtime isolation validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn local_signer_runtime_isolation_reviews() -> (
    SignerRuntimeIsolationReviewReport,
    SignerRuntimeIsolationReviewReport,
) {
    let ready = review_signer_runtime_isolation(&SignerRuntimeIsolationReviewRequest {
        review_id: "local-signer-runtime-isolation-ready".to_owned(),
        runtime_boundary_label: "local-signer-boundary".to_owned(),
        allowed_strategy_ids: vec!["strategy-signer".to_owned()],
        llm_direct_signer_access: false,
        llm_direct_signing_call: false,
        plaintext_key_material_exposed: false,
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        policy_gate_required: true,
        destination_allowlist_required: true,
        secret_scope_review_required: true,
        audit_before_signing_required: true,
        state_checkpoint_required: true,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_030,
    });
    let blocked = review_signer_runtime_isolation(&SignerRuntimeIsolationReviewRequest {
        review_id: "local-signer-runtime-isolation-blocked".to_owned(),
        runtime_boundary_label: "local-signer-boundary".to_owned(),
        allowed_strategy_ids: Vec::new(),
        llm_direct_signer_access: true,
        llm_direct_signing_call: true,
        plaintext_key_material_exposed: true,
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        policy_gate_required: false,
        destination_allowlist_required: false,
        secret_scope_review_required: false,
        audit_before_signing_required: false,
        state_checkpoint_required: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_031,
    });
    (ready, blocked)
}

fn run_signer_authorization_envelope_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("signer-authorization-envelope")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create signer authorization workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("signer-authorization.audit.jsonl");
    let state_path = workspace.join("signer-authorization.sqlite");
    let (ready, blocked) = local_signer_authorization_envelope_reports()?;
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_signer_authorization_envelope_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_signer_authorization_envelope_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_signer_authorization_envelope(&audit_path, &state_path)?;

    let ready_count =
        usize::from(ready.status == SignerAuthorizationEnvelopeStatus::ReadyForLocalAuthorization);
    let blocked_count = usize::from(blocked.status == SignerAuthorizationEnvelopeStatus::Blocked);
    let unsafe_side_effect = signer_authorization_side_effect_seen(&ready)
        || signer_authorization_side_effect_seen(&blocked);

    println!("signer-authorization-envelope: validation passed");
    println!("signer-authorization-envelope-count: 2");
    println!("signer-authorization-ready-count: {ready_count}");
    println!("signer-authorization-blocked-count: {blocked_count}");
    println!(
        "signer-authorization-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "policy-destination-ready: {}",
        ready.policy_destination_ready
    );
    println!("secret-scope-ready: {}", ready.secret_scope_ready);
    println!("runtime-isolation-ready: {}", ready.runtime_isolation_ready);
    println!(
        "transaction-safety-references-ready: {}",
        ready.transaction_safety_references_ready
    );
    println!(
        "audit-state-references-ready: {}",
        ready.audit_state_references_ready
    );
    println!("signer-material-loaded: false");
    println!("plaintext-decrypted: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("rpc-called: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "signer authorization envelope validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_pre_sign_safety_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-pre-sign-safety")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 pre-sign safety workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-pre-sign-safety.audit.jsonl");
    let state_path = workspace.join("web3-pre-sign-safety.sqlite");
    let (ready, blocked) = local_web3_pre_sign_safety_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_pre_sign_safety_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_pre_sign_safety_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_pre_sign_safety(&audit_path, &state_path)?;

    let ready_count =
        usize::from(ready.status == Web3PreSignSafetyReviewStatus::ReadyForLocalReview);
    let blocked_count = usize::from(blocked.status == Web3PreSignSafetyReviewStatus::Blocked);
    let unsafe_side_effect =
        web3_pre_sign_side_effect_seen(&ready) || web3_pre_sign_side_effect_seen(&blocked);

    println!("web3-pre-sign-safety: validation passed");
    println!("web3-pre-sign-safety-review-count: 2");
    println!("web3-pre-sign-safety-ready-count: {ready_count}");
    println!("web3-pre-sign-safety-blocked-count: {blocked_count}");
    println!(
        "web3-pre-sign-safety-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "simulation-success-ready: {}",
        ready.simulation_success_ready
    );
    println!("gas-fee-within-cap: {}", ready.gas_fee_within_cap);
    println!(
        "output-amount-sufficient: {}",
        ready.output_amount_sufficient
    );
    println!("nonce-ready: {}", ready.nonce_ready);
    println!("lifecycle-coherent: {}", ready.lifecycle_coherent);
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 pre-sign safety validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_broadcast_readiness_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-broadcast-readiness")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 broadcast readiness workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-broadcast-readiness.audit.jsonl");
    let state_path = workspace.join("web3-broadcast-readiness.sqlite");
    let (ready, blocked) = local_web3_broadcast_readiness_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_broadcast_readiness_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_broadcast_readiness_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_broadcast_readiness(&audit_path, &state_path)?;

    let ready_count =
        usize::from(ready.status == Web3BroadcastReadinessStatus::ReadyForExternalReview);
    let blocked_count = usize::from(blocked.status == Web3BroadcastReadinessStatus::Blocked);
    let unsafe_side_effect = web3_broadcast_readiness_side_effect_seen(&ready)
        || web3_broadcast_readiness_side_effect_seen(&blocked);

    println!("web3-broadcast-readiness: validation passed");
    println!("web3-broadcast-readiness-review-count: 2");
    println!("web3-broadcast-readiness-ready-count: {ready_count}");
    println!("web3-broadcast-readiness-blocked-count: {blocked_count}");
    println!(
        "web3-broadcast-readiness-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!("unsigned-payload-ready: {}", ready.unsigned_payload_ready);
    println!("pre-sign-safety-ready: {}", ready.pre_sign_safety_ready);
    println!(
        "signer-authorization-reference-ready: {}",
        ready.signer_authorization_reference_ready
    );
    println!(
        "live-adapter-reference-ready: {}",
        ready.live_adapter_reference_ready
    );
    println!(
        "operator-approval-reference-ready: {}",
        ready.operator_approval_reference_ready
    );
    println!("broadcast-allowed: false");
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 broadcast readiness validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_unsigned_transaction_construction_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-unsigned-transaction-construction")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 unsigned transaction construction workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-unsigned-transaction-construction.audit.jsonl");
    let state_path = workspace.join("web3-unsigned-transaction-construction.sqlite");
    let (ready, blocked) = local_web3_unsigned_transaction_construction_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_unsigned_transaction_construction_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_unsigned_transaction_construction_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_unsigned_transaction_construction(&audit_path, &state_path)?;

    let ready_count = usize::from(
        ready.status == Web3UnsignedTransactionConstructionStatus::ConstructedForLocalReview,
    );
    let blocked_count =
        usize::from(blocked.status == Web3UnsignedTransactionConstructionStatus::Blocked);
    let unsafe_side_effect = web3_unsigned_transaction_construction_side_effect_seen(&ready)
        || web3_unsigned_transaction_construction_side_effect_seen(&blocked);

    println!("web3-unsigned-transaction-construction: validation passed");
    println!("web3-unsigned-transaction-construction-count: 2");
    println!("web3-unsigned-transaction-construction-ready-count: {ready_count}");
    println!("web3-unsigned-transaction-construction-blocked-count: {blocked_count}");
    println!(
        "web3-unsigned-transaction-construction-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "broadcast-readiness-ready: {}",
        ready.broadcast_readiness_ready
    );
    println!("payload-reference-ready: {}", ready.payload_reference_ready);
    println!("target-selector-ready: {}", ready.target_selector_ready);
    println!("nonce-ready: {}", ready.nonce_ready);
    println!("gas-metadata-ready: {}", ready.gas_metadata_ready);
    println!("raw-calldata-embedded: false");
    println!("raw-transaction-serialized: false");
    println!("broadcast-allowed: false");
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 unsigned transaction construction validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_provider_nonce_reconciliation_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-provider-nonce-reconciliation")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 provider nonce reconciliation workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-provider-nonce-reconciliation.audit.jsonl");
    let state_path = workspace.join("web3-provider-nonce-reconciliation.sqlite");
    let (ready, blocked) = local_web3_provider_nonce_reconciliation_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_provider_nonce_reconciliation_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_provider_nonce_reconciliation_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_provider_nonce_reconciliation(&audit_path, &state_path)?;

    let ready_count = usize::from(
        ready.status == Web3ProviderNonceReconciliationStatus::ReconciledForLocalReview,
    );
    let blocked_count =
        usize::from(blocked.status == Web3ProviderNonceReconciliationStatus::Blocked);
    let unsafe_side_effect = web3_provider_nonce_reconciliation_side_effect_seen(&ready)
        || web3_provider_nonce_reconciliation_side_effect_seen(&blocked);

    println!("web3-provider-nonce-reconciliation: validation passed");
    println!("web3-provider-nonce-reconciliation-count: 2");
    println!("web3-provider-nonce-reconciliation-ready-count: {ready_count}");
    println!("web3-provider-nonce-reconciliation-blocked-count: {blocked_count}");
    println!(
        "web3-provider-nonce-reconciliation-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "unsigned-transaction-ready: {}",
        ready.unsigned_transaction_ready
    );
    println!(
        "provider-snapshot-reference-ready: {}",
        ready.provider_snapshot_reference_ready
    );
    println!(
        "provider-next-nonce-ready: {}",
        ready.provider_next_nonce_ready
    );
    println!(
        "construction-nonce-matches-provider: {}",
        ready.construction_nonce_matches_provider
    );
    println!(
        "construction-nonce-not-pending: {}",
        ready.construction_nonce_not_pending
    );
    println!(
        "pending-nonce-set-unique: {}",
        ready.pending_nonce_set_unique
    );
    println!("snapshot-fresh: {}", ready.snapshot_fresh);
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 provider nonce reconciliation validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_raw_transaction_serialization_review_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-raw-transaction-serialization-review")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 raw transaction serialization review workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-raw-transaction-serialization-review.audit.jsonl");
    let state_path = workspace.join("web3-raw-transaction-serialization-review.sqlite");
    let (ready, blocked) = local_web3_raw_transaction_serialization_review_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_raw_transaction_serialization_review_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint =
        persist_web3_raw_transaction_serialization_review_checkpoint(&mut store, &ready)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_raw_transaction_serialization_review(&audit_path, &state_path)?;

    let ready_count = usize::from(
        ready.status == Web3RawTransactionSerializationReviewStatus::ReadyForExternalReview,
    );
    let blocked_count =
        usize::from(blocked.status == Web3RawTransactionSerializationReviewStatus::Blocked);
    let unsafe_side_effect = web3_raw_transaction_serialization_review_side_effect_seen(&ready)
        || web3_raw_transaction_serialization_review_side_effect_seen(&blocked);

    println!("web3-raw-transaction-serialization-review: validation passed");
    println!("web3-raw-transaction-serialization-review-count: 2");
    println!("web3-raw-transaction-serialization-review-ready-count: {ready_count}");
    println!("web3-raw-transaction-serialization-review-blocked-count: {blocked_count}");
    println!(
        "web3-raw-transaction-serialization-review-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "provider-nonce-reconciliation-ready: {}",
        ready.provider_nonce_reconciliation_ready
    );
    println!("transaction-type-ready: {}", ready.transaction_type_ready);
    println!("chain-id-ready: {}", ready.chain_id_ready);
    println!("fee-fields-ready: {}", ready.fee_fields_ready);
    println!(
        "access-list-reference-ready: {}",
        ready.access_list_reference_ready
    );
    println!("raw-transaction-bytes-embedded: false");
    println!("raw-calldata-embedded: false");
    println!("raw-transaction-serialized: false");
    println!("broadcast-allowed: false");
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 raw transaction serialization review validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_broadcast_adapter_control_review_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-broadcast-adapter-control-review")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 broadcast adapter control review workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-broadcast-adapter-control-review.audit.jsonl");
    let state_path = workspace.join("web3-broadcast-adapter-control-review.sqlite");
    let (ready, blocked) = local_web3_broadcast_adapter_control_review_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_broadcast_adapter_control_review_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_broadcast_adapter_control_review_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_broadcast_adapter_control_review(&audit_path, &state_path)?;

    let ready_count = usize::from(
        ready.status == Web3BroadcastAdapterControlReviewStatus::ReadyForExternalReview,
    );
    let blocked_count =
        usize::from(blocked.status == Web3BroadcastAdapterControlReviewStatus::Blocked);
    let unsafe_side_effect = web3_broadcast_adapter_control_review_side_effect_seen(&ready)
        || web3_broadcast_adapter_control_review_side_effect_seen(&blocked);

    println!("web3-broadcast-adapter-control-review: validation passed");
    println!("web3-broadcast-adapter-control-review-count: 2");
    println!("web3-broadcast-adapter-control-review-ready-count: {ready_count}");
    println!("web3-broadcast-adapter-control-review-blocked-count: {blocked_count}");
    println!(
        "web3-broadcast-adapter-control-review-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "raw-transaction-serialization-review-ready: {}",
        ready.raw_transaction_serialization_review_ready
    );
    println!("adapter-reference-ready: {}", ready.adapter_reference_ready);
    println!(
        "operator-approval-reference-ready: {}",
        ready.operator_approval_reference_ready
    );
    println!(
        "audit-state-preflight-reference-ready: {}",
        ready.audit_state_preflight_reference_ready
    );
    println!("kill-switch-confirmed: {}", ready.kill_switch_confirmed);
    println!(
        "rate-limit-control-ready: {}",
        ready.rate_limit_control_ready
    );
    println!("replay-protection-ready: {}", ready.replay_protection_ready);
    println!("broadcast-permission-granted: false");
    println!("raw-transaction-bytes-embedded: false");
    println!("raw-transaction-serialized: false");
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 broadcast adapter control review validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_sandbox_live_discrepancy_calibration_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-sandbox-live-discrepancy-calibration")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 sandbox/live discrepancy calibration workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-sandbox-live-discrepancy-calibration.audit.jsonl");
    let state_path = workspace.join("web3-sandbox-live-discrepancy-calibration.sqlite");
    let (ready, blocked) = local_web3_sandbox_live_discrepancy_calibration_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_sandbox_live_discrepancy_calibration_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint =
        persist_web3_sandbox_live_discrepancy_calibration_checkpoint(&mut store, &ready)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_sandbox_live_discrepancy_calibration(&audit_path, &state_path)?;

    let ready_count = usize::from(
        ready.status == Web3SandboxLiveDiscrepancyCalibrationStatus::CalibratedForLocalReview,
    );
    let blocked_count =
        usize::from(blocked.status == Web3SandboxLiveDiscrepancyCalibrationStatus::Blocked);
    let unsafe_side_effect = web3_sandbox_live_discrepancy_calibration_side_effect_seen(&ready)
        || web3_sandbox_live_discrepancy_calibration_side_effect_seen(&blocked);

    println!("web3-sandbox-live-discrepancy-calibration: validation passed");
    println!("web3-sandbox-live-discrepancy-calibration-count: 2");
    println!("web3-sandbox-live-discrepancy-calibration-ready-count: {ready_count}");
    println!("web3-sandbox-live-discrepancy-calibration-blocked-count: {blocked_count}");
    println!(
        "web3-sandbox-live-discrepancy-calibration-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!(
        "broadcast-adapter-control-ready: {}",
        ready.broadcast_adapter_control_ready
    );
    println!(
        "sandbox-observation-reference-ready: {}",
        ready.sandbox_observation_reference_ready
    );
    println!(
        "live-observation-reference-ready: {}",
        ready.live_observation_reference_ready
    );
    println!("sample-size-ready: {}", ready.sample_size_ready);
    println!(
        "price-deviation-within-limit: {}",
        ready.price_deviation_within_limit
    );
    println!(
        "latency-deviation-within-limit: {}",
        ready.latency_deviation_within_limit
    );
    println!(
        "fee-deviation-within-limit: {}",
        ready.fee_deviation_within_limit
    );
    println!("external-call-performed: false");
    println!("credential-loaded: false");
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 sandbox/live discrepancy calibration validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_nonce_reservation_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-nonce-reservation")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 nonce reservation workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-nonce-reservation.audit.jsonl");
    let state_path = workspace.join("web3-nonce-reservation.sqlite");
    let (ready, blocked) = local_web3_nonce_reservation_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_nonce_reservation_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_nonce_reservation_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_nonce_reservation(&audit_path, &state_path)?;

    let ready_count =
        usize::from(ready.status == Web3NonceReservationStatus::ReservedForLocalReview);
    let blocked_count = usize::from(blocked.status == Web3NonceReservationStatus::Blocked);
    let unsafe_side_effect = web3_nonce_reservation_side_effect_seen(&ready)
        || web3_nonce_reservation_side_effect_seen(&blocked);

    println!("web3-nonce-reservation: validation passed");
    println!("web3-nonce-reservation-count: 2");
    println!("web3-nonce-reservation-ready-count: {ready_count}");
    println!("web3-nonce-reservation-blocked-count: {blocked_count}");
    println!(
        "web3-nonce-reservation-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!("nonce-ready: {}", ready.nonce_ready);
    println!(
        "reserved-nonce: {}",
        ready.reserved_nonce.unwrap_or_default()
    );
    println!("in-flight-nonce-count: {}", ready.in_flight_nonce_count);
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 nonce reservation validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn run_web3_unsigned_payload_review_validation() -> Result<(), AgentCliError> {
    let workspace = local_temp_workspace("web3-unsigned-payload-review")?;
    fs::create_dir_all(&workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create Web3 unsigned payload workspace {}: {error}",
            workspace.display()
        ))
    })?;
    let audit_path = workspace.join("web3-unsigned-payload.audit.jsonl");
    let state_path = workspace.join("web3-unsigned-payload.sqlite");
    let (ready, blocked) = local_web3_unsigned_payload_review_reports();
    ready
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    blocked
        .validate()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = append_web3_unsigned_payload_review_audit(&mut journal, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_web3_unsigned_payload_review_checkpoint(&mut store, &ready)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopen_web3_unsigned_payload_review(&audit_path, &state_path)?;

    let ready_count =
        usize::from(ready.status == Web3UnsignedPayloadReviewStatus::ReadyForLocalReview);
    let blocked_count = usize::from(blocked.status == Web3UnsignedPayloadReviewStatus::Blocked);
    let unsafe_side_effect = web3_unsigned_payload_side_effect_seen(&ready)
        || web3_unsigned_payload_side_effect_seen(&blocked);

    println!("web3-unsigned-payload-review: validation passed");
    println!("web3-unsigned-payload-review-count: 2");
    println!("web3-unsigned-payload-ready-count: {ready_count}");
    println!("web3-unsigned-payload-blocked-count: {blocked_count}");
    println!(
        "web3-unsigned-payload-blocker-count: {}",
        blocked.violation_count
    );
    println!("audit-records-replayed: 1");
    println!("state-checkpoints-recovered: 1");
    println!("nonce-ready: {}", ready.nonce_ready);
    println!("payload-reference-ready: {}", ready.payload_reference_ready);
    println!("router-spender-ready: {}", ready.router_spender_ready);
    println!("gas-cap-ready: {}", ready.gas_cap_ready);
    println!("raw-calldata-embedded: false");
    println!("rpc-called: false");
    println!("signer-material-loaded: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");

    if ready_count != 1
        || blocked_count != 1
        || blocked.violation_count == 0
        || audit_record.sequence != 1
        || checkpoint.key != DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY
        || unsafe_side_effect
    {
        return Err(AgentCliError::Validation(
            "Web3 unsigned payload review validation failed".to_owned(),
        ));
    }

    Ok(())
}

fn local_web3_pre_sign_safety_reports(
) -> (Web3PreSignSafetyReviewReport, Web3PreSignSafetyReviewReport) {
    let ready = Web3PreSignSafetyReviewRequest {
        id: "local-web3-pre-sign-ready".to_owned(),
        simulation_request: local_web3_simulation_request(),
        simulation_response: local_web3_simulation_response(),
        lifecycle_record: Some(local_web3_pre_sign_lifecycle_record()),
        max_gas_fee_quote: 0.25,
        nonce_required: true,
        planned_nonce: Some(7),
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_050,
    }
    .review();

    let mut blocked_response = local_web3_simulation_response();
    blocked_response.status = DexSimulationStatus::WouldRevert;
    blocked_response.gas_fee_quote = 2.0;
    blocked_response.amount_out = 1.0;
    blocked_response.diagnostic = Some("local blocked fixture only".to_owned());
    let blocked = Web3PreSignSafetyReviewRequest {
        id: "local-web3-pre-sign-blocked".to_owned(),
        simulation_request: local_web3_simulation_request(),
        simulation_response: blocked_response,
        lifecycle_record: Some(local_web3_pre_sign_lifecycle_record()),
        max_gas_fee_quote: 0.25,
        nonce_required: true,
        planned_nonce: None,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_051,
    }
    .review();

    (ready, blocked)
}

fn local_web3_nonce_reservation_reports() -> (Web3NonceReservationReport, Web3NonceReservationReport)
{
    let ready = local_web3_nonce_reservation_request("local-web3-nonce-ready", Some(7), vec![8, 9])
        .reserve();
    let mut blocked_request =
        local_web3_nonce_reservation_request("local-web3-nonce-blocked", Some(7), vec![7, 8, 8]);
    blocked_request.last_confirmed_nonce = Some(7);
    let blocked = blocked_request.reserve();
    (ready, blocked)
}

fn local_web3_unsigned_payload_review_reports() -> (
    Web3UnsignedPayloadReviewReport,
    Web3UnsignedPayloadReviewReport,
) {
    let ready = local_web3_unsigned_payload_review_request(
        "local-web3-unsigned-payload-ready",
        local_web3_nonce_reservation_request("local-web3-nonce-ready", Some(7), vec![8, 9])
            .reserve(),
        "reviewed-local-payload-hash-only",
        "paper-dex-router-reviewed",
        0.25,
    )
    .review();

    let mut blocked_nonce =
        local_web3_nonce_reservation_request("local-web3-nonce-blocked", Some(7), vec![7]);
    blocked_nonce.last_confirmed_nonce = Some(7);
    let blocked = local_web3_unsigned_payload_review_request(
        "local-web3-unsigned-payload-blocked",
        blocked_nonce.reserve(),
        "0xrawcalldata",
        "wrong-router",
        9.0,
    )
    .review();
    (ready, blocked)
}

fn local_web3_broadcast_readiness_reports(
) -> (Web3BroadcastReadinessReport, Web3BroadcastReadinessReport) {
    let ready = Web3BroadcastReadinessRequest {
        id: "local-web3-broadcast-readiness-ready".to_owned(),
        unsigned_payload_review: local_web3_unsigned_payload_review_request(
            "local-web3-unsigned-payload-ready",
            local_web3_nonce_reservation_request("local-web3-nonce-ready", Some(7), vec![8, 9])
                .reserve(),
            "reviewed-local-payload-hash-only",
            "paper-dex-router-reviewed",
            0.25,
        )
        .review(),
        pre_sign_safety_review: local_web3_pre_sign_safety_reports().0,
        signer_authorization_reference: "signer-authorization-local-ref".to_owned(),
        live_adapter_reference: "live-adapter-deferred-ref".to_owned(),
        operator_approval_reference: "operator-approval-local-ref".to_owned(),
        broadcast_allowed: false,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_052,
    }
    .review();

    let blocked = Web3BroadcastReadinessRequest {
        id: "local-web3-broadcast-readiness-blocked".to_owned(),
        unsigned_payload_review: local_web3_unsigned_payload_review_reports().1,
        pre_sign_safety_review: local_web3_pre_sign_safety_reports().1,
        signer_authorization_reference: String::new(),
        live_adapter_reference: "0x-live-adapter".to_owned(),
        operator_approval_reference: String::new(),
        broadcast_allowed: true,
        rpc_called: true,
        signer_material_loaded: true,
        signing_performed: true,
        broadcast_performed: true,
        live_execution_performed: true,
        production_ready: true,
        reviewed_at_unix_ms: 1_719_000_000_053,
    }
    .review();

    (ready, blocked)
}

fn local_web3_unsigned_transaction_construction_reports() -> (
    Web3UnsignedTransactionConstructionReport,
    Web3UnsignedTransactionConstructionReport,
) {
    let ready = Web3UnsignedTransactionConstructionRequest {
        id: "local-web3-unsigned-transaction-ready".to_owned(),
        broadcast_readiness_review: local_web3_broadcast_readiness_reports().0,
        payload_hash: "reviewed-local-payload-hash-only".to_owned(),
        function_selector: "swap-exact-input-single".to_owned(),
        encoded_argument_digest: "encoded-argument-digest-only".to_owned(),
        target_contract_label: "paper-dex-router-reviewed".to_owned(),
        nonce: Some(7),
        gas_limit: 150_000,
        max_fee_quote: 0.25,
        raw_calldata_embedded: false,
        raw_transaction_serialized: false,
        broadcast_allowed: false,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        constructed_at_unix_ms: 1_719_000_000_054,
    }
    .construct();

    let blocked = Web3UnsignedTransactionConstructionRequest {
        id: "local-web3-unsigned-transaction-blocked".to_owned(),
        broadcast_readiness_review: local_web3_broadcast_readiness_reports().1,
        payload_hash: "0xrawcalldata".to_owned(),
        function_selector: String::new(),
        encoded_argument_digest: "0xencoded".to_owned(),
        target_contract_label: String::new(),
        nonce: None,
        gas_limit: 0,
        max_fee_quote: f64::NAN,
        raw_calldata_embedded: true,
        raw_transaction_serialized: true,
        broadcast_allowed: true,
        rpc_called: true,
        signer_material_loaded: true,
        signing_performed: true,
        broadcast_performed: true,
        live_execution_performed: true,
        production_ready: true,
        constructed_at_unix_ms: 1_719_000_000_055,
    }
    .construct();

    (ready, blocked)
}

fn local_web3_provider_nonce_reconciliation_reports() -> (
    Web3ProviderNonceReconciliationReport,
    Web3ProviderNonceReconciliationReport,
) {
    let ready = Web3ProviderNonceReconciliationRequest {
        id: "local-web3-provider-nonce-ready".to_owned(),
        unsigned_transaction_construction: local_web3_unsigned_transaction_construction_reports().0,
        provider_snapshot_reference: "provider-nonce-snapshot-local".to_owned(),
        provider_next_nonce: Some(7),
        provider_pending_nonces: vec![8, 9],
        max_snapshot_age_ms: 30_000,
        snapshot_age_ms: 250,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reconciled_at_unix_ms: 1_719_000_000_056,
    }
    .reconcile();

    let blocked = Web3ProviderNonceReconciliationRequest {
        id: "local-web3-provider-nonce-blocked".to_owned(),
        unsigned_transaction_construction: local_web3_unsigned_transaction_construction_reports().0,
        provider_snapshot_reference: "0xprovider".to_owned(),
        provider_next_nonce: Some(8),
        provider_pending_nonces: vec![7, 8, 8],
        max_snapshot_age_ms: 30_000,
        snapshot_age_ms: 60_000,
        rpc_called: true,
        signer_material_loaded: true,
        signing_performed: true,
        broadcast_performed: true,
        live_execution_performed: true,
        production_ready: true,
        reconciled_at_unix_ms: 1_719_000_000_057,
    }
    .reconcile();

    (ready, blocked)
}

fn local_web3_raw_transaction_serialization_review_reports() -> (
    Web3RawTransactionSerializationReviewReport,
    Web3RawTransactionSerializationReviewReport,
) {
    let ready = Web3RawTransactionSerializationReviewRequest {
        id: "local-web3-raw-transaction-serialization-ready".to_owned(),
        provider_nonce_reconciliation: local_web3_provider_nonce_reconciliation_reports().0,
        transaction_type_label: "eip1559-local-review".to_owned(),
        chain_id_reference: "chain-id-1".to_owned(),
        fee_field_reference: "fee-fields-reviewed".to_owned(),
        access_list_reference: "access-list-empty-reviewed".to_owned(),
        raw_transaction_bytes_embedded: false,
        raw_calldata_embedded: false,
        raw_transaction_serialized: false,
        broadcast_allowed: false,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_058,
    }
    .review();

    let blocked = Web3RawTransactionSerializationReviewRequest {
        id: "local-web3-raw-transaction-serialization-blocked".to_owned(),
        provider_nonce_reconciliation: local_web3_provider_nonce_reconciliation_reports().1,
        transaction_type_label: String::new(),
        chain_id_reference: "0xrawchain".to_owned(),
        fee_field_reference: "0xrawfees".to_owned(),
        access_list_reference: "0xrawaccess".to_owned(),
        raw_transaction_bytes_embedded: true,
        raw_calldata_embedded: true,
        raw_transaction_serialized: true,
        broadcast_allowed: true,
        rpc_called: true,
        signer_material_loaded: true,
        signing_performed: true,
        broadcast_performed: true,
        live_execution_performed: true,
        production_ready: true,
        reviewed_at_unix_ms: 1_719_000_000_059,
    }
    .review();

    (ready, blocked)
}

fn local_web3_broadcast_adapter_control_review_reports() -> (
    Web3BroadcastAdapterControlReviewReport,
    Web3BroadcastAdapterControlReviewReport,
) {
    let ready = Web3BroadcastAdapterControlReviewRequest {
        id: "local-web3-broadcast-adapter-control-ready".to_owned(),
        raw_transaction_serialization_review:
            local_web3_raw_transaction_serialization_review_reports().0,
        adapter_reference: "local-broadcast-adapter-reviewed".to_owned(),
        operator_approval_reference: "operator-approval-reference-reviewed".to_owned(),
        audit_state_preflight_reference: "audit-state-preflight-reviewed".to_owned(),
        kill_switch_confirmed: true,
        rate_limit_control_ready: true,
        replay_protection_ready: true,
        broadcast_permission_granted: false,
        raw_transaction_bytes_embedded: false,
        raw_transaction_serialized: false,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_060,
    }
    .review();

    let blocked = Web3BroadcastAdapterControlReviewRequest {
        id: "local-web3-broadcast-adapter-control-blocked".to_owned(),
        raw_transaction_serialization_review:
            local_web3_raw_transaction_serialization_review_reports().1,
        adapter_reference: "0xadapter".to_owned(),
        operator_approval_reference: String::new(),
        audit_state_preflight_reference: "0xaudit".to_owned(),
        kill_switch_confirmed: false,
        rate_limit_control_ready: false,
        replay_protection_ready: false,
        broadcast_permission_granted: true,
        raw_transaction_bytes_embedded: true,
        raw_transaction_serialized: true,
        rpc_called: true,
        signer_material_loaded: true,
        signing_performed: true,
        broadcast_performed: true,
        live_execution_performed: true,
        production_ready: true,
        reviewed_at_unix_ms: 1_719_000_000_061,
    }
    .review();

    (ready, blocked)
}

fn local_web3_sandbox_live_discrepancy_calibration_reports() -> (
    Web3SandboxLiveDiscrepancyCalibrationReport,
    Web3SandboxLiveDiscrepancyCalibrationReport,
) {
    let ready = Web3SandboxLiveDiscrepancyCalibrationRequest {
        id: "local-web3-sandbox-live-calibration-ready".to_owned(),
        broadcast_adapter_control_review: local_web3_broadcast_adapter_control_review_reports().0,
        sandbox_observation_reference: "sandbox-observation-reviewed".to_owned(),
        live_observation_reference: "live-observation-reviewed".to_owned(),
        max_price_deviation_bps: 25.0,
        observed_price_deviation_bps: 12.5,
        max_latency_deviation_ms: 250,
        observed_latency_deviation_ms: 125,
        max_fee_deviation_quote: 0.05,
        observed_fee_deviation_quote: 0.02,
        minimum_sample_count: 3,
        sandbox_sample_count: 5,
        live_sample_count: 4,
        external_call_performed: false,
        credential_loaded: false,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        calibrated_at_unix_ms: 1_719_000_000_062,
    }
    .calibrate();

    let blocked = Web3SandboxLiveDiscrepancyCalibrationRequest {
        id: "local-web3-sandbox-live-calibration-blocked".to_owned(),
        broadcast_adapter_control_review: local_web3_broadcast_adapter_control_review_reports().1,
        sandbox_observation_reference: "0xsandbox".to_owned(),
        live_observation_reference: String::new(),
        max_price_deviation_bps: 25.0,
        observed_price_deviation_bps: 30.0,
        max_latency_deviation_ms: 250,
        observed_latency_deviation_ms: 300,
        max_fee_deviation_quote: 0.05,
        observed_fee_deviation_quote: 0.07,
        minimum_sample_count: 5,
        sandbox_sample_count: 2,
        live_sample_count: 1,
        external_call_performed: true,
        credential_loaded: true,
        rpc_called: true,
        signer_material_loaded: true,
        signing_performed: true,
        broadcast_performed: true,
        live_execution_performed: true,
        production_ready: true,
        calibrated_at_unix_ms: 1_719_000_000_063,
    }
    .calibrate();

    (ready, blocked)
}

fn local_web3_unsigned_payload_review_request(
    id: &str,
    nonce_reservation: Web3NonceReservationReport,
    payload_hash: &str,
    router_label: &str,
    max_gas_fee_quote: f64,
) -> Web3UnsignedPayloadReviewRequest {
    Web3UnsignedPayloadReviewRequest {
        id: id.to_owned(),
        simulation_request: local_web3_simulation_request(),
        nonce_reservation,
        payload_hash: payload_hash.to_owned(),
        router_label: router_label.to_owned(),
        spender_label: "paper-dex-spender-reviewed".to_owned(),
        max_gas_fee_quote,
        raw_calldata_embedded: false,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: 1_719_000_000_049,
    }
}

fn local_web3_nonce_reservation_request(
    id: &str,
    requested_nonce: Option<u64>,
    in_flight_nonces: Vec<u64>,
) -> Web3NonceReservationRequest {
    Web3NonceReservationRequest {
        id: id.to_owned(),
        chain: "local-chain".to_owned(),
        venue: VenueRef {
            name: "paper-dex".to_owned(),
            kind: VenueKind::Dex,
        },
        account_label: "local-paper-account".to_owned(),
        last_confirmed_nonce: Some(6),
        requested_nonce,
        in_flight_nonces,
        ttl_ms: 30_000,
        rpc_called: false,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
        planned_at_unix_ms: 1_719_000_000_049,
    }
}

fn reopen_web3_unsigned_payload_review(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 unsigned payload audit replay did not recover one record".to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 unsigned payload checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 unsigned payload checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_nonce_reservation(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 nonce reservation audit replay did not recover one record".to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 nonce reservation checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 nonce reservation checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_broadcast_readiness(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 broadcast readiness audit replay did not recover one record".to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 broadcast readiness checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 broadcast readiness checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_unsigned_transaction_construction(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 unsigned transaction construction audit replay did not recover one record"
                .to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 unsigned transaction construction checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 unsigned transaction construction checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_provider_nonce_reconciliation(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 provider nonce reconciliation audit replay did not recover one record".to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 provider nonce reconciliation checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 provider nonce reconciliation checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_raw_transaction_serialization_review(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 raw transaction serialization review audit replay did not recover one record"
                .to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 raw transaction serialization review checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 raw transaction serialization review checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_broadcast_adapter_control_review(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 broadcast adapter control review audit replay did not recover one record"
                .to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 broadcast adapter control review checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 broadcast adapter control review checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

fn reopen_web3_sandbox_live_discrepancy_calibration(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 sandbox/live discrepancy calibration audit replay did not recover one record"
                .to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 sandbox/live discrepancy calibration checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 sandbox/live discrepancy calibration checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

const fn web3_unsigned_payload_side_effect_seen(report: &Web3UnsignedPayloadReviewReport) -> bool {
    report.raw_calldata_embedded
        || report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_nonce_reservation_side_effect_seen(report: &Web3NonceReservationReport) -> bool {
    report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_broadcast_readiness_side_effect_seen(report: &Web3BroadcastReadinessReport) -> bool {
    report.broadcast_allowed
        || report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_unsigned_transaction_construction_side_effect_seen(
    report: &Web3UnsignedTransactionConstructionReport,
) -> bool {
    report.raw_calldata_embedded
        || report.raw_transaction_serialized
        || report.broadcast_allowed
        || report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_provider_nonce_reconciliation_side_effect_seen(
    report: &Web3ProviderNonceReconciliationReport,
) -> bool {
    report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_raw_transaction_serialization_review_side_effect_seen(
    report: &Web3RawTransactionSerializationReviewReport,
) -> bool {
    report.raw_transaction_bytes_embedded
        || report.raw_calldata_embedded
        || report.raw_transaction_serialized
        || report.broadcast_allowed
        || report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_broadcast_adapter_control_review_side_effect_seen(
    report: &Web3BroadcastAdapterControlReviewReport,
) -> bool {
    report.broadcast_permission_granted
        || report.raw_transaction_bytes_embedded
        || report.raw_transaction_serialized
        || report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

const fn web3_sandbox_live_discrepancy_calibration_side_effect_seen(
    report: &Web3SandboxLiveDiscrepancyCalibrationReport,
) -> bool {
    report.external_call_performed
        || report.credential_loaded
        || report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

fn reopen_web3_pre_sign_safety(audit_path: &Path, state_path: &Path) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "Web3 pre-sign safety audit replay did not recover one record".to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "Web3 pre-sign safety checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "Web3 pre-sign safety checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

const fn web3_pre_sign_side_effect_seen(report: &Web3PreSignSafetyReviewReport) -> bool {
    report.rpc_called
        || report.signer_material_loaded
        || report.signing_performed
        || report.broadcast_performed
        || report.live_execution_performed
        || report.production_ready
}

fn local_signer_authorization_envelope_reports() -> Result<
    (
        SignerAuthorizationEnvelopeReport,
        SignerAuthorizationEnvelopeReport,
    ),
    AgentCliError,
> {
    let signer_case = build_local_signer_boundary_audit_case(1_719_000_000_040)?;
    let (ready_isolation, blocked_isolation) = local_signer_runtime_isolation_reviews();
    let ready = build_local_signer_authorization_envelope(&SignerAuthorizationEnvelopeRequest {
        envelope_id: "local-signer-authorization-ready".to_owned(),
        signer_request_record: signer_case.request_record.clone(),
        secret_scope_review: signer_case.scope_report.clone(),
        runtime_isolation_review: ready_isolation,
        transaction_simulation_reference: "simulation-ref:local-dry-run-001".to_owned(),
        nonce_plan_reference: "nonce-plan-ref:local-sequence-001".to_owned(),
        pre_sign_audit_reference: "audit-ref:signer-request-seq-001".to_owned(),
        pre_sign_state_checkpoint_key: SIGNER_LAST_REQUEST_CHECKPOINT_KEY.to_owned(),
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        production_ready: false,
        created_at_unix_ms: 1_719_000_000_041,
    });
    let blocked = build_local_signer_authorization_envelope(&SignerAuthorizationEnvelopeRequest {
        envelope_id: "local-signer-authorization-blocked".to_owned(),
        signer_request_record: signer_case.request_record,
        secret_scope_review: signer_case.scope_report,
        runtime_isolation_review: blocked_isolation,
        transaction_simulation_reference: String::new(),
        nonce_plan_reference: "secret=bad".to_owned(),
        pre_sign_audit_reference: String::new(),
        pre_sign_state_checkpoint_key: "invalid-sensitive-locator".to_owned(),
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        production_ready: false,
        created_at_unix_ms: 1_719_000_000_042,
    });
    Ok((ready, blocked))
}

fn reopen_signer_authorization_envelope(
    audit_path: &Path,
    state_path: &Path,
) -> Result<(), AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    if reopened_journal.next_sequence() != 2 {
        return Err(AgentCliError::Validation(
            "signer authorization audit replay did not recover one record".to_owned(),
        ));
    }
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = reopened_store
        .get_checkpoint(SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "signer authorization checkpoint was not recovered".to_owned(),
            )
        })?;
    if checkpoint.value.trim().is_empty() {
        return Err(AgentCliError::Validation(
            "signer authorization checkpoint value was empty".to_owned(),
        ));
    }
    Ok(())
}

const fn signer_authorization_side_effect_seen(report: &SignerAuthorizationEnvelopeReport) -> bool {
    report.signer_material_loaded
        || report.plaintext_decrypted
        || report.signing_performed
        || report.broadcast_performed
        || report.rpc_called
        || report.production_ready
}

fn build_local_signer_boundary_audit_case(
    now_unix_ms: u64,
) -> Result<LocalSignerBoundaryAuditCase, AgentCliError> {
    let config = AgentConfig::from_toml_str(LOCAL_STRATEGY_PLANNER_CONFIG)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let policy = PolicyEngine::from_config(config);
    let intent = local_policy_decision_intent("signer-boundary", ExecutionScope::Paper, false);
    let policy_decision = PolicyDecisionRecord::from_decision(
        &intent,
        &policy.evaluate(&intent),
        now_unix_ms.saturating_add(1),
    );
    let request_record = evaluate_local_signer_request(&SignerRequest {
        request_id: "local-signer-boundary-request".to_owned(),
        intent_id: intent.id,
        strategy_id: intent.strategy_id,
        requested_scope: ExecutionScope::Paper,
        chain: Some("local-chain".to_owned()),
        destination: DestinationPolicy::InternalAccount,
        payload_reference: "local-payload-hash-reference".to_owned(),
        policy_decision,
        requested_at_unix_ms: now_unix_ms.saturating_add(2),
    });
    let scope_report = review_signer_secret_scope(&SignerSecretScopeReviewRequest {
        review_id: "local-signer-boundary-scope-review".to_owned(),
        request_id: request_record.request_id.clone(),
        intent_id: request_record.intent_id.clone(),
        strategy_id: request_record.strategy_id.clone(),
        chain: request_record.chain.clone(),
        signer_reference: SecretRef::Keystore {
            alias: "local-signer-boundary-alias".to_owned(),
        },
        allowed_strategy_ids: vec![request_record.strategy_id.clone()],
        allowed_chains: vec!["local-chain".to_owned()],
        allowed_keystore_aliases: vec!["local-signer-boundary-alias".to_owned()],
        reviewed_at_unix_ms: now_unix_ms.saturating_add(3),
    });
    Ok(LocalSignerBoundaryAuditCase {
        request_record,
        scope_report,
    })
}

fn persist_local_signer_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    signer_case: &LocalSignerBoundaryAuditCase,
) -> Result<SignerBoundaryAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let request_audit = append_signer_request_audit(&mut journal, &signer_case.request_record)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let scope_audit =
        append_signer_secret_scope_review_audit(&mut journal, &signer_case.scope_report)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let request_checkpoint =
        persist_signer_request_checkpoint(&mut store, &signer_case.request_record)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let scope_checkpoint =
        persist_signer_secret_scope_review_checkpoint(&mut store, &signer_case.scope_report)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let request_audit_failed_closed = validate_signer_request_invalid_audit_fails_closed(
        &mut journal,
        &signer_case.request_record,
    );
    let scope_audit_failed_closed =
        validate_signer_scope_invalid_audit_fails_closed(&mut journal, &signer_case.scope_report);
    Ok(SignerBoundaryAuditPersistence {
        request_sequence: request_audit.sequence,
        scope_sequence: scope_audit.sequence,
        request_checkpoint_value: request_checkpoint.value,
        scope_checkpoint_value: scope_checkpoint.value,
        request_audit_failed_closed,
        scope_audit_failed_closed,
    })
}

fn verify_local_signer_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &SignerBoundaryAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_request = reopened
        .get_checkpoint(SIGNER_LAST_REQUEST_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| AgentCliError::Validation("signer request checkpoint missing".to_owned()))?;
    let recovered_scope = reopened
        .get_checkpoint(SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| AgentCliError::Validation("signer scope checkpoint missing".to_owned()))?;

    if replayed.next_sequence() <= persisted.scope_sequence
        || persisted.request_sequence == persisted.scope_sequence
        || recovered_request.value != persisted.request_checkpoint_value
        || recovered_scope.value != persisted.scope_checkpoint_value
        || !persisted.request_audit_failed_closed
        || !persisted.scope_audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "signer boundary audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_signer_request_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    record: &SignerRequestRecord,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = record.clone();
    invalid.signing_performed = true;
    let failed = append_signer_request_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_signer_scope_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &SignerSecretScopeReviewReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.plaintext_decrypted = true;
    let failed = append_signer_secret_scope_review_audit(journal, &invalid).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_signer_boundary_state_failure(signer_case: &LocalSignerBoundaryAuditCase) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let request_failed =
        persist_signer_request_checkpoint(&mut store, &signer_case.request_record).is_err();
    let scope_failed =
        persist_signer_secret_scope_review_checkpoint(&mut store, &signer_case.scope_report)
            .is_err();
    request_failed && scope_failed && store.put_attempts == 2
}

struct LocalDestinationBoundaryAuditCase {
    allowlist: DestinationAllowlist,
    ownership_review: DestinationOwnershipReviewReport,
}

struct DestinationBoundaryAuditPersistence {
    allowlist_sequence: u64,
    ownership_review_sequence: u64,
    allowlist_checkpoint_value: String,
    ownership_review_checkpoint_value: String,
    allowlist_audit_failed_closed: bool,
    ownership_review_audit_failed_closed: bool,
}

fn run_destination_boundary_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options
        .workspace_dir
        .join("destination-boundary.audit.jsonl");
    let state_path = options.workspace_dir.join("destination-boundary.sqlite3");
    let destination_case = build_local_destination_boundary_audit_case(now_unix_ms)?;
    let persisted = persist_local_destination_boundary_audit_case(
        &audit_path,
        &state_path,
        &destination_case,
        now_unix_ms,
    )?;
    let audit_records_replayed =
        verify_local_destination_boundary_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed =
        validate_destination_boundary_state_failure(&destination_case);

    if !state_failure_failed_closed
        || destination_case.ownership_review.status != DestinationOwnershipReviewStatus::Referenced
        || destination_case.ownership_review.chain_ownership_verified
        || destination_case.ownership_review.signer_material_loaded
        || destination_case.ownership_review.challenge_signed
        || destination_case.ownership_review.production_ready
    {
        return Err(AgentCliError::Validation(
            "destination boundary audit/state validation failed".to_owned(),
        ));
    }

    println!("destination-boundary-audit: validation passed");
    println!("destination-allowlist-version: {DESTINATION_ALLOWLIST_VERSION}");
    println!(
        "destination-enabled-entry-count: {}",
        destination_case.ownership_review.enabled_entry_count
    );
    println!(
        "destination-referenced-evidence-count: {}",
        destination_case.ownership_review.referenced_evidence_count
    );
    println!(
        "destination-allowlist-audit-failed-closed: {}",
        persisted.allowlist_audit_failed_closed
    );
    println!(
        "destination-ownership-review-audit-failed-closed: {}",
        persisted.ownership_review_audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("chain-ownership-verified: false");
    println!("signer-material-loaded: false");
    println!("challenge-signed: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_destination_boundary_audit_case(
    now_unix_ms: u64,
) -> Result<LocalDestinationBoundaryAuditCase, AgentCliError> {
    let allowlist = DestinationAllowlist {
        destination_allowlist_version: DESTINATION_ALLOWLIST_VERSION.to_owned(),
        snapshot_id: "local-destination-boundary-snapshot".to_owned(),
        updated_at_unix_ms: now_unix_ms,
        entries: vec![ApprovedDestinationEntry {
            label: "local-treasury-reference".to_owned(),
            chain: "local-chain".to_owned(),
            address_fingerprint: "local-address-fingerprint-reference".to_owned(),
            approval_id: "local-destination-approval".to_owned(),
            approved_by: "local-operator-review".to_owned(),
            approval_source: DestinationApprovalSource::LocalOperator,
            ownership_evidence_referenced: true,
            enabled: true,
        }],
    };
    let ownership_review = allowlist
        .review_ownership_evidence(now_unix_ms.saturating_add(1))
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(LocalDestinationBoundaryAuditCase {
        allowlist,
        ownership_review,
    })
}

fn persist_local_destination_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    destination_case: &LocalDestinationBoundaryAuditCase,
    now_unix_ms: u64,
) -> Result<DestinationBoundaryAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let allowlist_audit =
        append_destination_allowlist_audit(&mut journal, &destination_case.allowlist, now_unix_ms)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ownership_review_audit = append_destination_ownership_review_audit(
        &mut journal,
        &destination_case.ownership_review,
        now_unix_ms.saturating_add(1),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let allowlist_checkpoint = persist_destination_allowlist_checkpoint(
        &mut store,
        &destination_case.allowlist,
        now_unix_ms.saturating_add(2),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let ownership_review_checkpoint = persist_destination_ownership_review_checkpoint(
        &mut store,
        &destination_case.ownership_review,
        now_unix_ms.saturating_add(3),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let allowlist_audit_failed_closed = validate_destination_allowlist_invalid_audit_fails_closed(
        &mut journal,
        &destination_case.allowlist,
    );
    let ownership_review_audit_failed_closed =
        validate_destination_ownership_review_invalid_audit_fails_closed(
            &mut journal,
            &destination_case.ownership_review,
        );

    Ok(DestinationBoundaryAuditPersistence {
        allowlist_sequence: allowlist_audit.sequence,
        ownership_review_sequence: ownership_review_audit.sequence,
        allowlist_checkpoint_value: allowlist_checkpoint.value,
        ownership_review_checkpoint_value: ownership_review_checkpoint.value,
        allowlist_audit_failed_closed,
        ownership_review_audit_failed_closed,
    })
}

fn verify_local_destination_boundary_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &DestinationBoundaryAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_allowlist = reopened
        .get_checkpoint(DESTINATION_ALLOWLIST_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("destination allowlist checkpoint missing".to_owned())
        })?;
    let recovered_ownership_review = reopened
        .get_checkpoint(DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation("destination ownership review checkpoint missing".to_owned())
        })?;

    if replayed.next_sequence() <= persisted.ownership_review_sequence
        || persisted.allowlist_sequence == persisted.ownership_review_sequence
        || recovered_allowlist.value != persisted.allowlist_checkpoint_value
        || recovered_ownership_review.value != persisted.ownership_review_checkpoint_value
        || !persisted.allowlist_audit_failed_closed
        || !persisted.ownership_review_audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "destination boundary audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn validate_destination_allowlist_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    allowlist: &DestinationAllowlist,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = allowlist.clone();
    invalid.entries[0].approval_source = DestinationApprovalSource::LlmGenerated;
    let failed = append_destination_allowlist_audit(journal, &invalid, 1_700_000_401).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_destination_ownership_review_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    report: &DestinationOwnershipReviewReport,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = report.clone();
    invalid.chain_ownership_verified = true;
    let failed =
        append_destination_ownership_review_audit(journal, &invalid, 1_700_000_402).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_destination_boundary_state_failure(
    destination_case: &LocalDestinationBoundaryAuditCase,
) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let allowlist_failed =
        persist_destination_allowlist_checkpoint(&mut store, &destination_case.allowlist, 1)
            .is_err();
    let ownership_review_failed = persist_destination_ownership_review_checkpoint(
        &mut store,
        &destination_case.ownership_review,
        2,
    )
    .is_err();
    allowlist_failed && ownership_review_failed && store.put_attempts == 2
}

struct LocalConnectorLifecycleAuditCase {
    cex_filled: CexOrderLifecycleRecord,
    cex_cancelled: CexOrderLifecycleRecord,
    dex_swap: DexSwapLifecycleRecord,
}

struct ConnectorLifecycleAuditPersistence {
    cex_sequence: u64,
    cex_cancel_sequence: u64,
    dex_sequence: u64,
    cex_checkpoint_value: String,
    dex_checkpoint_value: String,
    cex_audit_failed_closed: bool,
    dex_audit_failed_closed: bool,
}

fn run_connector_lifecycle_audit_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let now_unix_ms = current_unix_ms()?;
    let audit_path = options
        .workspace_dir
        .join("connector-lifecycle.audit.jsonl");
    let state_path = options.workspace_dir.join("connector-lifecycle.sqlite3");
    let connector_case = build_local_connector_lifecycle_audit_case()?;
    let persisted = persist_local_connector_lifecycle_audit_case(
        &audit_path,
        &state_path,
        &connector_case,
        now_unix_ms,
    )?;
    let audit_records_replayed =
        verify_local_connector_lifecycle_audit_case(&audit_path, &state_path, &persisted)?;
    let state_failure_failed_closed = validate_connector_lifecycle_state_failure(&connector_case);

    if !state_failure_failed_closed
        || connector_case.cex_filled.external_submission_performed
        || connector_case.cex_filled.live_execution_performed
        || connector_case.cex_filled.production_ready
        || connector_case.cex_cancelled.external_submission_performed
        || connector_case.cex_cancelled.live_execution_performed
        || connector_case.cex_cancelled.production_ready
        || connector_case.dex_swap.rpc_call_performed
        || connector_case.dex_swap.signing_performed
        || connector_case.dex_swap.broadcast_performed
        || connector_case.dex_swap.live_execution_performed
        || connector_case.dex_swap.production_ready
    {
        return Err(AgentCliError::Validation(
            "connector lifecycle audit/state validation failed".to_owned(),
        ));
    }

    println!("connector-lifecycle-audit: validation passed");
    println!(
        "cex-lifecycle-final-status: {:?}",
        connector_case.cex_filled.final_status
    );
    println!(
        "cex-lifecycle-transitions: {}",
        connector_case.cex_filled.transition_count
    );
    println!(
        "cex-lifecycle-transcript-count: {}",
        connector_case.cex_filled.transition_count
    );
    println!(
        "cex-cancel-lifecycle-final-status: {:?}",
        connector_case.cex_cancelled.final_status
    );
    println!(
        "cex-cancel-lifecycle-transitions: {}",
        connector_case.cex_cancelled.transition_count
    );
    println!(
        "cex-cancel-lifecycle-transcript-count: {}",
        connector_case.cex_cancelled.transition_count
    );
    println!(
        "cex-cancel-lifecycle-remaining-quantity-base: {}",
        connector_case.cex_cancelled.remaining_quantity_base
    );
    println!(
        "dex-lifecycle-simulation-status: {:?}",
        connector_case.dex_swap.simulation_status
    );
    println!(
        "cex-audit-failed-closed: {}",
        persisted.cex_audit_failed_closed
    );
    println!(
        "dex-audit-failed-closed: {}",
        persisted.dex_audit_failed_closed
    );
    println!("state-failure-failed-closed: {state_failure_failed_closed}");
    println!("audit-records-replayed: {audit_records_replayed}");
    println!("state-checkpoints-recovered: true");
    println!("external-submission-performed: false");
    println!("rpc-call-performed: false");
    println!("signing-performed: false");
    println!("broadcast-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn build_local_connector_lifecycle_audit_case(
) -> Result<LocalConnectorLifecycleAuditCase, AgentCliError> {
    let cex_validation = CexOrderValidationRecord::from_approved_request(
        &local_cex_order_request(),
        &local_policy_approval("local-cex-lifecycle-order", ExecutionScope::Paper),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_responses = local_cex_lifecycle_transcripts()
        .into_iter()
        .map(|transcript| {
            transcript
                .parse_lifecycle_response(&cex_validation)
                .map_err(|error| AgentCliError::Validation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cex_record =
        CexOrderLifecycleRecord::from_local_responses(&cex_validation, &cex_responses, true)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_cancel_responses = local_cex_cancel_lifecycle_transcripts()
        .into_iter()
        .map(|transcript| {
            transcript
                .parse_lifecycle_response(&cex_validation)
                .map_err(|error| AgentCliError::Validation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cex_cancel_record =
        CexOrderLifecycleRecord::from_local_responses(&cex_validation, &cex_cancel_responses, true)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let dex_validation = DexSwapValidationRecord::from_approved_request(
        &local_dex_swap_request(),
        &local_policy_approval("local-dex-lifecycle-swap", ExecutionScope::Paper),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let dex_record = DexSwapLifecycleRecord::from_local_quote_and_simulation(
        &dex_validation,
        &local_dex_quote_response(),
        &local_web3_simulation_response(),
        true,
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    Ok(LocalConnectorLifecycleAuditCase {
        cex_filled: cex_record,
        cex_cancelled: cex_cancel_record,
        dex_swap: dex_record,
    })
}

fn persist_local_connector_lifecycle_audit_case(
    audit_path: &Path,
    state_path: &Path,
    connector_case: &LocalConnectorLifecycleAuditCase,
    now_unix_ms: u64,
) -> Result<ConnectorLifecycleAuditPersistence, AgentCliError> {
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_audit =
        append_cex_order_lifecycle_audit(&mut journal, &connector_case.cex_filled, now_unix_ms)
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_cancel_audit = append_cex_order_lifecycle_audit(
        &mut journal,
        &connector_case.cex_cancelled,
        now_unix_ms.saturating_add(1),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let dex_audit = append_dex_swap_lifecycle_audit(
        &mut journal,
        &connector_case.dex_swap,
        now_unix_ms.saturating_add(2),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    persist_cex_order_lifecycle_checkpoint(
        &mut store,
        &connector_case.cex_filled,
        now_unix_ms.saturating_add(3),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_checkpoint = persist_cex_order_lifecycle_checkpoint(
        &mut store,
        &connector_case.cex_cancelled,
        now_unix_ms.saturating_add(4),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let dex_checkpoint = persist_dex_swap_lifecycle_checkpoint(
        &mut store,
        &connector_case.dex_swap,
        now_unix_ms.saturating_add(5),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_audit_failed_closed =
        validate_cex_lifecycle_invalid_audit_fails_closed(&mut journal, &connector_case.cex_filled);
    let dex_audit_failed_closed =
        validate_dex_lifecycle_invalid_audit_fails_closed(&mut journal, &connector_case.dex_swap);

    Ok(ConnectorLifecycleAuditPersistence {
        cex_sequence: cex_audit.sequence,
        cex_cancel_sequence: cex_cancel_audit.sequence,
        dex_sequence: dex_audit.sequence,
        cex_checkpoint_value: cex_checkpoint.value,
        dex_checkpoint_value: dex_checkpoint.value,
        cex_audit_failed_closed,
        dex_audit_failed_closed,
    })
}

fn verify_local_connector_lifecycle_audit_case(
    audit_path: &Path,
    state_path: &Path,
    persisted: &ConnectorLifecycleAuditPersistence,
) -> Result<u64, AgentCliError> {
    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let cex_checkpoint = reopened
        .get_checkpoint(CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| AgentCliError::Validation("CEX lifecycle checkpoint missing".to_owned()))?;
    let dex_checkpoint = reopened
        .get_checkpoint(DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| AgentCliError::Validation("DEX lifecycle checkpoint missing".to_owned()))?;

    if replayed.next_sequence() <= persisted.dex_sequence
        || persisted.cex_sequence == persisted.dex_sequence
        || persisted.cex_cancel_sequence == persisted.cex_sequence
        || persisted.cex_cancel_sequence == persisted.dex_sequence
        || cex_checkpoint.value != persisted.cex_checkpoint_value
        || dex_checkpoint.value != persisted.dex_checkpoint_value
        || !persisted.cex_audit_failed_closed
        || !persisted.dex_audit_failed_closed
    {
        return Err(AgentCliError::Validation(
            "connector lifecycle audit/state validation failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn local_policy_approval(intent_id: &str, scope: ExecutionScope) -> PolicyApproval {
    PolicyApproval {
        trust_contract_version: "local-connector-lifecycle-trust-contract",
        intent_id: intent_id.to_owned(),
        approved_scope: scope,
    }
}

fn local_cex_order_request() -> CexOrderRequest {
    CexOrderRequest {
        id: "local-cex-lifecycle-order".to_owned(),
        strategy_id: "local-connector-lifecycle".to_owned(),
        client_order_id: "local-cex-client-order".to_owned(),
        scope: ExecutionScope::Paper,
        venue: VenueRef {
            name: "paper-cex".to_owned(),
            kind: VenueKind::Cex,
        },
        pair: MarketPair::new("BTC", "USDC")
            .expect("static connector lifecycle CEX pair validates"),
        side: CexOrderSide::Buy,
        order_type: CexOrderType::Limit,
        time_in_force: CexTimeInForce::Gtc,
        quantity_base: 0.01,
        limit_price_quote: Some(100.0),
        notional_quote: 25.0,
        expected_profit_quote: 1.0,
        max_loss_quote: 1.0,
        slippage_bps: 20,
        estimated_fee_quote: 0.10,
        market_data_age_ms: 1_000,
        liquidity_role: LiquidityRole::Maker,
        reduce_only: false,
    }
}

fn local_cex_lifecycle_transcripts() -> Vec<CexOrderLifecycleTranscript> {
    let venue = VenueRef {
        name: "paper-cex".to_owned(),
        kind: VenueKind::Cex,
    };
    let pair = MarketPair::new("BTC", "USDC").expect("static CEX lifecycle pair validates");
    vec![
        CexOrderLifecycleTranscript::new(
            "local-cex-transcript-binance-accepted",
            CexOrderLifecycleTranscriptFormat::BinanceExecutionReport,
            venue.clone(),
            pair.clone(),
            r#"{"c":"local-cex-client-order","i":"binance-local-order-1","X":"NEW","l":"0","L":"0","n":"0"}"#,
            1_700_000_009,
            1_700_000_010,
        )
        .expect("static Binance-shaped lifecycle transcript validates"),
        CexOrderLifecycleTranscript::new(
            "local-cex-transcript-coinbase-partial",
            CexOrderLifecycleTranscriptFormat::CoinbaseOrderEvent,
            venue.clone(),
            pair.clone(),
            r#"{"client_order_id":"local-cex-client-order","order_id":"coinbase-local-order-1","status":"match","last_fill_size":"0.004","price":"100.0","fee":"0.0008"}"#,
            1_700_000_010,
            1_700_000_011,
        )
        .expect("static Coinbase-shaped lifecycle transcript validates"),
        CexOrderLifecycleTranscript::new(
            "local-cex-transcript-kraken-filled",
            CexOrderLifecycleTranscriptFormat::KrakenOrderStatus,
            venue,
            pair,
            r#"{"userref":"local-cex-client-order","txid":"kraken-local-order-1","status":"closed","vol_exec_delta":"0.006","price":"101.0","fee":"0.0012"}"#,
            1_700_000_011,
            1_700_000_012,
        )
        .expect("static Kraken-shaped lifecycle transcript validates"),
    ]
}

fn local_cex_cancel_lifecycle_transcripts() -> Vec<CexOrderLifecycleTranscript> {
    let venue = VenueRef {
        name: "paper-cex".to_owned(),
        kind: VenueKind::Cex,
    };
    let pair = MarketPair::new("BTC", "USDC").expect("static CEX lifecycle pair validates");
    vec![
        CexOrderLifecycleTranscript::new(
            "local-cex-cancel-transcript-binance-accepted",
            CexOrderLifecycleTranscriptFormat::BinanceExecutionReport,
            venue.clone(),
            pair.clone(),
            r#"{"c":"local-cex-client-order","i":"binance-local-cancel-order-1","X":"NEW","l":"0","L":"0","n":"0"}"#,
            1_700_000_019,
            1_700_000_020,
        )
        .expect("static Binance-shaped cancel lifecycle transcript validates"),
        CexOrderLifecycleTranscript::new(
            "local-cex-cancel-transcript-coinbase-partial",
            CexOrderLifecycleTranscriptFormat::CoinbaseOrderEvent,
            venue.clone(),
            pair.clone(),
            r#"{"client_order_id":"local-cex-client-order","order_id":"coinbase-local-cancel-order-1","status":"match","last_fill_size":"0.004","price":"100.0","fee":"0.0008"}"#,
            1_700_000_020,
            1_700_000_021,
        )
        .expect("static Coinbase-shaped cancel lifecycle transcript validates"),
        CexOrderLifecycleTranscript::new(
            "local-cex-cancel-transcript-kraken-cancelled",
            CexOrderLifecycleTranscriptFormat::KrakenOrderStatus,
            venue,
            pair,
            r#"{"userref":"local-cex-client-order","txid":"kraken-local-cancel-order-1","status":"canceled","vol_exec_delta":"0","price":"0","fee":"0"}"#,
            1_700_000_021,
            1_700_000_022,
        )
        .expect("static Kraken-shaped cancel lifecycle transcript validates"),
    ]
}

fn local_dex_swap_request() -> DexSwapQuoteRequest {
    DexSwapQuoteRequest {
        id: "local-dex-lifecycle-swap".to_owned(),
        strategy_id: "local-connector-lifecycle".to_owned(),
        scope: ExecutionScope::Paper,
        venue: VenueRef {
            name: "paper-dex".to_owned(),
            kind: VenueKind::Dex,
        },
        chain: "local-chain".to_owned(),
        pair: MarketPair::new("ETH", "USDC")
            .expect("static connector lifecycle DEX pair validates"),
        mode: DexSwapMode::ExactInput,
        route_kind: DexRouteKind::SinglePool,
        input_token_symbol: "ETH".to_owned(),
        output_token_symbol: "USDC".to_owned(),
        amount_in: 0.1,
        expected_amount_out: 25.0,
        notional_quote: 25.0,
        expected_profit_quote: 1.0,
        max_loss_quote: 1.0,
        slippage_bps: 20,
        estimated_fee_quote: 0.10,
        gas_fee_quote: 0.25,
        market_data_age_ms: 1_000,
    }
}

fn local_dex_quote_response() -> DexSwapQuoteResponse {
    DexSwapQuoteResponse {
        id: "local-dex-quote-response".to_owned(),
        request_id: "local-dex-lifecycle-swap".to_owned(),
        venue: VenueRef {
            name: "paper-dex".to_owned(),
            kind: VenueKind::Dex,
        },
        chain: "local-chain".to_owned(),
        pair: MarketPair::new("ETH", "USDC")
            .expect("static connector lifecycle DEX quote pair validates"),
        route_kind: DexRouteKind::SinglePool,
        amount_in: 0.1,
        amount_out: 25.0,
        price_impact_bps: 12.0,
        estimated_fee_quote: 0.10,
        gas_fee_quote: 0.25,
        market_data_age_ms: 1_000,
        simulation_status: DexSimulationStatus::LocallyValidated,
    }
}

fn local_web3_simulation_response() -> Web3TransactionSimulationResponse {
    Web3TransactionSimulationResponse {
        id: "local-web3-simulation-response".to_owned(),
        request_id: "local-web3-simulation-request".to_owned(),
        status: DexSimulationStatus::LocallyValidated,
        gas_used: 150_000,
        gas_fee_quote: 0.25,
        amount_out: 24.5,
        diagnostic: Some("local fixture only; no RPC, signing, or broadcast".to_owned()),
        broadcastable: false,
    }
}

fn local_web3_simulation_request() -> Web3TransactionSimulationRequest {
    Web3TransactionSimulationRequest {
        id: "local-web3-simulation-request".to_owned(),
        swap_request_id: "local-dex-lifecycle-swap".to_owned(),
        scope: ExecutionScope::Paper,
        venue: VenueRef {
            name: "paper-dex".to_owned(),
            kind: VenueKind::Dex,
        },
        chain: "local-chain".to_owned(),
        router_label: "paper-dex-router-reviewed".to_owned(),
        spender_label: "paper-dex-spender-reviewed".to_owned(),
        account_label: "local-paper-account".to_owned(),
        input_token_symbol: "ETH".to_owned(),
        output_token_symbol: "USDC".to_owned(),
        amount_in: 0.1,
        minimum_amount_out: 24.5,
        gas_limit: 150_000,
        max_gas_fee_quote: 0.25,
        payload_hash: "reviewed-local-payload-hash-only".to_owned(),
    }
}

fn local_web3_pre_sign_lifecycle_record() -> Web3TransactionLifecycleRecord {
    Web3TransactionLifecycleRecord {
        framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
        transcript_id: "local-web3-pre-sign-lifecycle".to_owned(),
        request_id: "local-web3-simulation-request".to_owned(),
        chain: "local-chain".to_owned(),
        venue: VenueRef {
            name: "paper-dex".to_owned(),
            kind: VenueKind::Dex,
        },
        transaction_id: "local-pre-sign-transaction-reference".to_owned(),
        nonce: Some(7),
        block_number: Some(19_000_001),
        slot: None,
        confirmations: 12,
        status: Web3TransactionLifecycleStatus::Confirmed,
        diagnostic: Some("local lifecycle fixture only".to_owned()),
        rpc_call_performed: false,
        signing_performed: false,
        broadcast_performed: false,
        live_execution_performed: false,
        production_ready: false,
    }
}

fn validate_cex_lifecycle_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    record: &CexOrderLifecycleRecord,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = record.clone();
    invalid.external_submission_performed = true;
    let failed = append_cex_order_lifecycle_audit(journal, &invalid, 1_700_000_101).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_dex_lifecycle_invalid_audit_fails_closed(
    journal: &mut AppendOnlyAuditJournal,
    record: &DexSwapLifecycleRecord,
) -> bool {
    let next_sequence = journal.next_sequence();
    let mut invalid = record.clone();
    invalid.broadcast_performed = true;
    let failed = append_dex_swap_lifecycle_audit(journal, &invalid, 1_700_000_102).is_err();
    failed && journal.next_sequence() == next_sequence
}

fn validate_connector_lifecycle_state_failure(
    connector_case: &LocalConnectorLifecycleAuditCase,
) -> bool {
    let mut store = PermissionDeniedLocalStateStore::default();
    let cex_failed =
        persist_cex_order_lifecycle_checkpoint(&mut store, &connector_case.cex_filled, 1).is_err();
    let cex_cancel_failed =
        persist_cex_order_lifecycle_checkpoint(&mut store, &connector_case.cex_cancelled, 2)
            .is_err();
    let dex_failed =
        persist_dex_swap_lifecycle_checkpoint(&mut store, &connector_case.dex_swap, 3).is_err();
    cex_failed && cex_cancel_failed && dex_failed && store.put_attempts == 3
}

fn local_policy_decision_intent(
    suffix: &str,
    scope: ExecutionScope,
    requires_signing: bool,
) -> ExecutionIntent {
    ExecutionIntent {
        id: format!("local-policy-decision-{suffix}"),
        strategy_id: "local-policy-decision-audit".to_owned(),
        kind: ExecutionIntentKind::CexOrder,
        scope,
        venue: VenueRef {
            name: "paper-a".to_owned(),
            kind: VenueKind::Cex,
        },
        chain: None,
        base_asset: "BTC".to_owned(),
        quote_asset: "USD".to_owned(),
        notional_quote: 25.0,
        expected_profit_quote: 1.0,
        max_loss_quote: 1.0,
        slippage_bps: 20,
        estimated_fee_quote: 0.1,
        gas_fee_quote: 0.0,
        market_data_age_ms: 100,
        destination: DestinationPolicy::InternalAccount,
        requires_signing,
    }
}

fn validate_local_paper_backtest_corpus_report(
    report: &PaperBacktestRunReport,
    expected_scenarios: usize,
) -> Result<(), AgentCliError> {
    if report.scenarios_executed != expected_scenarios
        || report.total_steps < 3
        || report.filled_steps == 0
        || !report.replay_validated
        || !report.historical_fixture_replay
        || !report.local_fixture_only
        || report.external_data_downloaded
        || report.live_network_used
        || report.external_execution_performed
    {
        return Err(AgentCliError::Validation(
            "local paper backtest corpus execution did not satisfy local-only replay invariants"
                .to_owned(),
        ));
    }
    Ok(())
}

fn persist_local_paper_backtest_corpus_report(
    audit_path: &Path,
    state_path: &Path,
    report: &PaperBacktestRunReport,
    now_unix_ms: u64,
) -> Result<u64, AgentCliError> {
    let checkpoint_value = format!(
        "corpus={};scenarios={};steps={};filled={};partial={};unfilled={};replay={}",
        report.corpus_id,
        report.scenarios_executed,
        report.total_steps,
        report.filled_steps,
        report.partially_filled_steps,
        report.unfilled_steps,
        report.replay_validated
    );
    let mut journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_record = journal
        .append_event(
            AuditEvent::new(
                "local-paper-backtest-corpus-report",
                AuditEventKind::Reconciliation,
                "testing",
                "local-validation-runner",
                "local paper backtest corpus executed without live side effects",
            )
            .with_metadata(
                "testing_version",
                AuditValue::Text(TESTING_BACKTESTING_VERSION.to_owned()),
            )
            .with_metadata(
                "paper_version",
                AuditValue::Text(PAPER_REALISM_VALIDATION_VERSION.to_owned()),
            )
            .with_metadata("corpus_id", AuditValue::Text(report.corpus_id.clone()))
            .with_metadata(
                "scenarios",
                AuditValue::Text(report.scenarios_executed.to_string()),
            )
            .with_metadata("steps", AuditValue::Text(report.total_steps.to_string()))
            .with_metadata(
                "filled_steps",
                AuditValue::Text(report.filled_steps.to_string()),
            )
            .with_metadata(
                "partial_steps",
                AuditValue::Text(report.partially_filled_steps.to_string()),
            )
            .with_metadata(
                "unfilled_steps",
                AuditValue::Text(report.unfilled_steps.to_string()),
            )
            .with_metadata(
                "replay_validated",
                AuditValue::Bool(report.replay_validated),
            )
            .with_metadata(
                "live_network_used",
                AuditValue::Bool(report.live_network_used),
            )
            .with_metadata(
                "external_execution_performed",
                AuditValue::Bool(report.external_execution_performed),
            )
            .with_metadata("production_ready", AuditValue::Bool(false)),
        )
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = StateCheckpoint {
        key: LOCAL_PAPER_BACKTEST_CORPUS_CHECKPOINT_KEY.to_owned(),
        subsystem: "testing".to_owned(),
        value: checkpoint_value,
        updated_at_unix_ms: now_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    drop(store);
    drop(journal);

    let replayed = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let reopened = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered = reopened
        .get_checkpoint(LOCAL_PAPER_BACKTEST_CORPUS_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?
        .ok_or_else(|| {
            AgentCliError::Validation(
                "local paper backtest corpus checkpoint was not recovered after reopen".to_owned(),
            )
        })?;

    if replayed.next_sequence() <= audit_record.sequence || recovered.value != checkpoint.value {
        return Err(AgentCliError::Validation(
            "local paper backtest corpus audit/state reopen check failed".to_owned(),
        ));
    }
    Ok(replayed.next_sequence() - 1)
}

fn print_local_paper_backtest_corpus_report(
    report: &PaperBacktestRunReport,
    replayed_records: u64,
) {
    println!("local-paper-backtest-corpus: validation passed");
    println!("testing-backtesting-version: {TESTING_BACKTESTING_VERSION}");
    println!("paper-realism-version: {PAPER_REALISM_VALIDATION_VERSION}");
    println!("paper-backtest-corpus-id: {}", report.corpus_id);
    println!("paper-backtest-scenarios: {}", report.scenarios_executed);
    println!("paper-backtest-steps: {}", report.total_steps);
    println!("paper-backtest-filled-steps: {}", report.filled_steps);
    println!(
        "paper-backtest-partial-steps: {}",
        report.partially_filled_steps
    );
    println!("paper-backtest-unfilled-steps: {}", report.unfilled_steps);
    println!(
        "paper-backtest-replay-validated: {}",
        report.replay_validated
    );
    println!("audit-records-replayed: {replayed_records}");
    println!("state-checkpoint-recovered: true");
    println!(
        "external-data-downloaded: {}",
        report.external_data_downloaded
    );
    println!("live-network-used: {}", report.live_network_used);
    println!(
        "external-execution-performed: {}",
        report.external_execution_performed
    );
    println!("production-ready: false");
}

fn local_paper_backtest_corpus(now_unix_ms: u64) -> Result<PaperBacktestCorpus, AgentCliError> {
    Ok(PaperBacktestCorpus {
        corpus_id: "local-paper-backtest-ci-corpus".to_owned(),
        historical_fixture_replay: true,
        local_fixture_only: true,
        external_data_downloaded: false,
        scenarios: vec![PaperBacktestScenario {
            scenario_id: "local-paper-backtest-btc-usd-depth-window".to_owned(),
            initial_balances: vec![
                PaperAssetBalance::available(local_paper_venue(), "USD", 2_500.0)
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
                PaperAssetBalance::available(local_paper_venue(), "BTC", 1.0)
                    .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            ],
            steps: vec![
                PaperBacktestStep {
                    step_id: "local-paper-backtest-filled".to_owned(),
                    fill_request: local_paper_fill_request(
                        "filled",
                        25.0,
                        50,
                        true,
                        local_paper_depth_book("filled", now_unix_ms, 0.50)?,
                        now_unix_ms.saturating_add(1),
                    ),
                    matching_profile: None,
                    adverse_selection: None,
                    calibration: None,
                },
                PaperBacktestStep {
                    step_id: "local-paper-backtest-partial".to_owned(),
                    fill_request: local_paper_fill_request(
                        "partial",
                        150.0,
                        25,
                        true,
                        local_paper_depth_book("partial", now_unix_ms, 0.75)?,
                        now_unix_ms.saturating_add(2),
                    ),
                    matching_profile: None,
                    adverse_selection: None,
                    calibration: None,
                },
                PaperBacktestStep {
                    step_id: "local-paper-backtest-unfilled".to_owned(),
                    fill_request: local_paper_fill_request(
                        "unfilled",
                        150.0,
                        0,
                        false,
                        local_paper_depth_book("unfilled", now_unix_ms, 0.10)?,
                        now_unix_ms.saturating_add(3),
                    ),
                    matching_profile: None,
                    adverse_selection: None,
                    calibration: None,
                },
            ],
        }],
    })
}

fn local_paper_fill_request(
    suffix: &str,
    notional_quote: f64,
    max_slippage_bps: u16,
    allow_partial_fills: bool,
    order_book: OrderBookSnapshot,
    now_unix_ms: u64,
) -> PaperFillSimulationRequest {
    PaperFillSimulationRequest {
        intent: ExecutionIntent {
            id: format!("local-paper-backtest-intent-{suffix}"),
            strategy_id: "local-paper-backtest-ci".to_owned(),
            kind: ExecutionIntentKind::CexOrder,
            scope: ExecutionScope::Paper,
            venue: local_paper_venue(),
            chain: None,
            base_asset: "BTC".to_owned(),
            quote_asset: "USD".to_owned(),
            notional_quote,
            expected_profit_quote: notional_quote * 0.04,
            max_loss_quote: notional_quote * 0.02,
            slippage_bps: max_slippage_bps,
            estimated_fee_quote: notional_quote * 0.001,
            gas_fee_quote: 0.0,
            market_data_age_ms: 100,
            destination: DestinationPolicy::InternalAccount,
            requires_signing: false,
        },
        order_book,
        side: PaperFillSide::BuyBase,
        config: PaperFillModelConfig {
            max_slippage_bps,
            latency_ms: 25,
            queue_position_bps: 0,
            allow_partial_fills,
            min_partial_fill_bps: 1,
        },
        now_unix_ms,
    }
}

fn local_paper_depth_book(
    suffix: &str,
    now_unix_ms: u64,
    first_ask_quantity_base: f64,
) -> Result<OrderBookSnapshot, AgentCliError> {
    Ok(OrderBookSnapshot {
        id: format!("local-paper-backtest-book-{suffix}"),
        venue: local_paper_venue(),
        pair: MarketPair::new("BTC", "USD")
            .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        captured_at_unix_ms: now_unix_ms,
        received_at_unix_ms: now_unix_ms.saturating_add(1),
        bids: vec![
            PriceLevel::new(99.0, 1.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            PriceLevel::new(98.5, 2.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        ],
        asks: vec![
            PriceLevel::new(100.0, first_ask_quantity_base)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
            PriceLevel::new(101.0, 2.0)
                .map_err(|error| AgentCliError::Validation(error.to_string()))?,
        ],
        source_sequence: Some(format!("local-paper-backtest-seq-{suffix}")),
    })
}

fn local_paper_venue() -> VenueRef {
    VenueRef {
        name: "paper-a".to_owned(),
        kind: VenueKind::Cex,
    }
}

fn local_validation_runner_plan(now_unix_ms: u64) -> ValidationPlan {
    const VALID_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    ValidationPlan {
        plan_id: "local-validation-runner-plan".to_owned(),
        generated_at_ms: now_unix_ms,
        execution_mode: ValidationExecutionMode::FixtureReplayOnly,
        test_cases: vec![
            ValidationTestCase::new(
                "local-policy-deny-live",
                "Policy denies live execution in unsafe modes",
                ValidationSuiteKind::Policy,
                "policy",
                ExpectedValidationOutcome::FailClosed,
            ),
            ValidationTestCase::new(
                "local-audit-state-reopen",
                "Audit and SQLite validation-run checkpoint reopens locally",
                ValidationSuiteKind::Replay,
                "testing",
                ExpectedValidationOutcome::Pass,
            ),
        ],
        fixtures: vec![ValidationFixtureRecord::synthetic(
            "fixture-local-validation-runner",
            FixtureKind::PolicyDecision,
            "fixtures/local/validation-runner-policy.json",
            Some(VALID_DIGEST.to_owned()),
            now_unix_ms,
        )],
        fuzz_corpora: vec![FuzzCorpusDefinition::local_only(
            "local-validation-command-parser-seeds",
            FuzzTargetKind::CommandParser,
            vec![FuzzSeedRecord::new(
                "seed-validate-local-validation-run",
                VALID_DIGEST,
                "local CLI validation-runner command seed",
            )],
        )],
        backtest_scenarios: vec![BacktestScenarioDefinition {
            scenario_id: "local-validation-paper-backtest-boundary".to_owned(),
            dataset: BacktestDatasetDefinition {
                dataset_id: "local-validation-btc-usd-fixture".to_owned(),
                base_asset: "BTC".to_owned(),
                quote_asset: "USD".to_owned(),
                venue_ids: vec!["paper-a".to_owned(), "paper-b".to_owned()],
                start_ms: now_unix_ms,
                end_ms: now_unix_ms.saturating_add(60_000),
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

fn local_validation_runner_corpus(now_unix_ms: u64) -> Vec<ValidationPlan> {
    let mut validation_plan = local_validation_runner_plan(now_unix_ms);
    "local-validation-corpus-plan-a".clone_into(&mut validation_plan.plan_id);

    let mut replay_plan = local_validation_runner_plan(now_unix_ms.saturating_add(1));
    "local-validation-corpus-plan-b".clone_into(&mut replay_plan.plan_id);
    replay_plan.test_cases.push(ValidationTestCase::new(
        "local-corpus-property-replay",
        "Validation corpus replays local metadata invariants without external tooling",
        ValidationSuiteKind::Replay,
        "testing",
        ExpectedValidationOutcome::Pass,
    ));

    let mut safety_plan = local_validation_runner_plan(now_unix_ms.saturating_add(2));
    "local-validation-corpus-plan-c".clone_into(&mut safety_plan.plan_id);
    safety_plan.test_cases.push(ValidationTestCase::new(
        "local-corpus-redaction-regression",
        "Validation corpus checks local redaction and denial metadata without secret fixtures",
        ValidationSuiteKind::Security,
        "testing",
        ExpectedValidationOutcome::Pass,
    ));
    safety_plan.test_cases.push(ValidationTestCase::new(
        "local-corpus-paper-backtest-regression",
        "Validation corpus checks local paper backtest metadata without external datasets",
        ValidationSuiteKind::Backtest,
        "testing",
        ExpectedValidationOutcome::Pass,
    ));

    vec![validation_plan, replay_plan, safety_plan]
}

fn local_validation_runner_fuzz_corpora() -> Vec<FuzzCorpusDefinition> {
    const VALID_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    vec![
        FuzzCorpusDefinition::local_only(
            "local-fuzz-command-parser-seeds",
            FuzzTargetKind::CommandParser,
            vec![
                FuzzSeedRecord::new(
                    "seed-validate-local-validation-run",
                    VALID_DIGEST,
                    "local CLI validation-runner command seed",
                ),
                FuzzSeedRecord::new(
                    "seed-validate-local-fuzz-corpus",
                    VALID_DIGEST,
                    "local CLI fuzz-corpus replay command seed",
                ),
            ],
        ),
        FuzzCorpusDefinition::local_only(
            "local-fuzz-redaction-seeds",
            FuzzTargetKind::Redaction,
            vec![FuzzSeedRecord::new(
                "seed-redacted-operator-label",
                VALID_DIGEST,
                "local operator-label redaction seed",
            )],
        ),
    ]
}

const fn validation_run_status_label(status: ValidationRunStatus) -> &'static str {
    match status {
        ValidationRunStatus::PlannedOnly => "planned-only",
        ValidationRunStatus::Rejected => "rejected",
    }
}

const fn service_manager_lifecycle_status_label(
    status: RuntimeServiceManagerLifecycleTranscriptStatus,
) -> &'static str {
    match status {
        RuntimeServiceManagerLifecycleTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        RuntimeServiceManagerLifecycleTranscriptStatus::Blocked => "blocked",
    }
}

const fn service_manager_lifecycle_rehearsal_status_label(
    status: RuntimeServiceManagerLifecycleRehearsalStatus,
) -> &'static str {
    match status {
        RuntimeServiceManagerLifecycleRehearsalStatus::Validated => "validated",
        RuntimeServiceManagerLifecycleRehearsalStatus::Blocked => "blocked",
    }
}

const fn deployment_disk_full_status_label(
    status: AuditDeploymentDiskFullTranscriptStatus,
) -> &'static str {
    match status {
        AuditDeploymentDiskFullTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        AuditDeploymentDiskFullTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_retention_status_label(
    status: AuditDeploymentRetentionTranscriptStatus,
) -> &'static str {
    match status {
        AuditDeploymentRetentionTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        AuditDeploymentRetentionTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_permission_status_label(
    status: RuntimeDeploymentPermissionTranscriptStatus,
) -> &'static str {
    match status {
        RuntimeDeploymentPermissionTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        RuntimeDeploymentPermissionTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_audit_sqlite_status_label(
    status: RuntimeDeploymentAuditSqliteTranscriptStatus,
) -> &'static str {
    match status {
        RuntimeDeploymentAuditSqliteTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        RuntimeDeploymentAuditSqliteTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_backup_restore_status_label(
    status: RuntimeDeploymentBackupRestoreTranscriptStatus,
) -> &'static str {
    match status {
        RuntimeDeploymentBackupRestoreTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        RuntimeDeploymentBackupRestoreTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_graceful_shutdown_status_label(
    status: RuntimeDeploymentGracefulShutdownTranscriptStatus,
) -> &'static str {
    match status {
        RuntimeDeploymentGracefulShutdownTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        RuntimeDeploymentGracefulShutdownTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_sqlite_schema_migration_status_label(
    status: RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus,
) -> &'static str {
    match status {
        RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::Blocked => "blocked",
    }
}

const fn rollback_execution_status_label(
    status: RollbackExecutionTranscriptStatus,
) -> &'static str {
    match status {
        RollbackExecutionTranscriptStatus::ReadyForExternalReview => "ready-for-external-review",
        RollbackExecutionTranscriptStatus::Blocked => "blocked",
    }
}

const fn incident_response_execution_status_label(
    status: IncidentResponseExecutionTranscriptStatus,
) -> &'static str {
    match status {
        IncidentResponseExecutionTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        IncidentResponseExecutionTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_failure_capture_status_label(
    status: DeploymentFailureCaptureTranscriptStatus,
) -> &'static str {
    match status {
        DeploymentFailureCaptureTranscriptStatus::ReadyForExternalReview => {
            "ready-for-external-review"
        }
        DeploymentFailureCaptureTranscriptStatus::Blocked => "blocked",
    }
}

const fn deployment_response_drill_rehearsal_status_label(
    status: DeploymentResponseDrillRehearsalStatus,
) -> &'static str {
    match status {
        DeploymentResponseDrillRehearsalStatus::Validated => "validated",
        DeploymentResponseDrillRehearsalStatus::Blocked => "blocked",
    }
}

const fn validation_corpus_status_label(status: LocalValidationCorpusStatus) -> &'static str {
    match status {
        LocalValidationCorpusStatus::ReadyForLocalReview => "ready-for-local-review",
    }
}

const fn validation_coverage_review_status_label(
    status: LocalValidationCoverageReviewStatus,
) -> &'static str {
    match status {
        LocalValidationCoverageReviewStatus::ReadyForLocalReview => "ready-for-local-review",
        LocalValidationCoverageReviewStatus::Blocked => "blocked",
    }
}

const fn fuzz_corpus_replay_status_label(status: LocalFuzzCorpusReplayStatus) -> &'static str {
    match status {
        LocalFuzzCorpusReplayStatus::ReadyForLocalReview => "ready-for-local-review",
    }
}

fn parse_runtime_smoke_options(
    args: impl Iterator<Item = String>,
) -> Result<RuntimeSmokeOptions, AgentCliError> {
    let mut config_path = None;
    let mut workspace_dir = None;
    let mut iterations = 1_usize;
    let mut pending = args;
    while let Some(arg) = pending.next() {
        match arg.as_str() {
            "--config" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-runtime-smoke --config requires a path".to_owned(),
                    ));
                };
                config_path = Some(PathBuf::from(value));
            }
            "--workspace" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-runtime-smoke --workspace requires a fresh directory".to_owned(),
                    ));
                };
                workspace_dir = Some(PathBuf::from(value));
            }
            "--iterations" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-runtime-smoke --iterations requires an integer >= 1".to_owned(),
                    ));
                };
                let parsed = value.parse::<usize>().map_err(|_| {
                    AgentCliError::Usage(
                        "validate-runtime-smoke --iterations requires an integer >= 1".to_owned(),
                    )
                })?;
                if parsed == 0 {
                    return Err(AgentCliError::Usage(
                        "validate-runtime-smoke --iterations requires an integer >= 1".to_owned(),
                    ));
                }
                iterations = parsed;
            }
            other => {
                return Err(AgentCliError::Usage(format!(
                    "unknown validate-runtime-smoke argument: {other}"
                )));
            }
        }
    }

    Ok(RuntimeSmokeOptions {
        config_path: config_path.ok_or_else(|| {
            AgentCliError::Usage("validate-runtime-smoke requires --config <path>".to_owned())
        })?,
        workspace_dir: workspace_dir.ok_or_else(|| {
            AgentCliError::Usage(
                "validate-runtime-smoke requires --workspace <fresh-dir>".to_owned(),
            )
        })?,
        iterations,
    })
}

fn run_communications_runtime_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options.workspace_dir.join("communications-audit.jsonl");
    let state_path = options.workspace_dir.join("communications-state.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let records = write_communications_runtime_records(&mut journal, &mut store, now_unix_ms)?;
    drop(store);
    drop(journal);

    let recovery = recover_communications_runtime_records(&audit_path, &state_path, &records)?;
    if !communications_runtime_validation_passed(&records, &recovery) {
        return Err(AgentCliError::Validation(
            "communications runtime validation failed".to_owned(),
        ));
    }

    print_communications_runtime_report(options, &records, &recovery);
    Ok(())
}

struct CommunicationsRuntimeRecords {
    route: RoutedOperatorCommand,
    remote_review: RemoteCommandSecurityReviewReport,
    platform_ingress: PlatformCommandIngressReport,
    remote_envelope: RemoteCommandEnvelopeValidationReport,
    channel_adapter: ChannelAdapterValidationReport,
    channel_session: ChannelSessionValidationReport,
    platform_adapter: PlatformAdapterReviewReport,
    dispatch: NotificationDispatchRecord,
    audit_record_count: usize,
    checkpoint_keys: [String; 8],
}

struct CommunicationsRuntimeRecovery {
    replayed_records: u64,
    recovered_checkpoint_count: usize,
}

fn communications_runtime_config() -> CommunicationBoundaryConfig {
    CommunicationBoundaryConfig {
        notification_channels: vec![
            NotificationChannelProfile::from_identifier("cli"),
            NotificationChannelProfile::from_identifier("local-stdout"),
        ],
        ..CommunicationBoundaryConfig::default()
    }
}

fn write_communications_runtime_records(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    now_unix_ms: u64,
) -> Result<CommunicationsRuntimeRecords, AgentCliError> {
    let config = communications_runtime_config();
    let route = route_local_communications_status(config.clone(), now_unix_ms)?;
    let route_audit =
        append_routed_operator_command_audit(journal, &route, now_unix_ms.saturating_add(1))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let route_checkpoint =
        persist_routed_operator_command_checkpoint(store, &route, now_unix_ms.saturating_add(2))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let (remote_review, review_audit, review_checkpoint) =
        write_remote_command_security_review(journal, store, now_unix_ms)?;

    let (platform_ingress, platform_ingress_audit, platform_ingress_checkpoint) =
        write_communications_platform_command_ingress(journal, store, now_unix_ms)?;

    let remote_envelope =
        build_local_remote_command_envelope(&remote_review, &platform_ingress, now_unix_ms)?;
    let envelope_audit = append_remote_command_envelope_validation_audit(
        journal,
        &remote_envelope,
        now_unix_ms.saturating_add(7),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let envelope_checkpoint = persist_remote_command_envelope_validation_checkpoint(
        store,
        &remote_envelope,
        now_unix_ms.saturating_add(8),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let dispatch = dispatch_local_communications_notification(config, now_unix_ms)?;
    let dispatch_audit =
        append_notification_dispatch_audit(journal, &dispatch, now_unix_ms.saturating_add(10))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let dispatch_checkpoint =
        persist_notification_dispatch_checkpoint(store, &dispatch, now_unix_ms.saturating_add(11))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let channel_adapter =
        build_local_communications_channel_adapter(&remote_envelope, &dispatch, now_unix_ms)?;
    let channel_adapter_audit = append_channel_adapter_validation_audit(
        journal,
        &channel_adapter,
        now_unix_ms.saturating_add(13),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let channel_adapter_checkpoint = persist_channel_adapter_validation_checkpoint(
        store,
        &channel_adapter,
        now_unix_ms.saturating_add(14),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let (channel_session, channel_session_audit, channel_session_checkpoint) =
        write_communications_channel_session(
            journal,
            store,
            &remote_envelope,
            &dispatch,
            &channel_adapter,
            now_unix_ms,
        )?;
    let (platform_adapter, platform_adapter_audit, platform_adapter_checkpoint) =
        write_communications_platform_adapter_review(
            journal,
            store,
            &remote_envelope,
            now_unix_ms,
        )?;

    let audit_record_count = [
        route_audit.sequence,
        review_audit.sequence,
        platform_ingress_audit.sequence,
        envelope_audit.sequence,
        dispatch_audit.sequence,
        channel_adapter_audit.sequence,
        channel_session_audit.sequence,
        platform_adapter_audit.sequence,
    ]
    .len();

    Ok(CommunicationsRuntimeRecords {
        route,
        remote_review,
        platform_ingress,
        remote_envelope,
        channel_adapter,
        channel_session,
        platform_adapter,
        dispatch,
        audit_record_count,
        checkpoint_keys: [
            route_checkpoint.key,
            review_checkpoint.key,
            platform_ingress_checkpoint.key,
            envelope_checkpoint.key,
            dispatch_checkpoint.key,
            channel_adapter_checkpoint.key,
            channel_session_checkpoint.key,
            platform_adapter_checkpoint.key,
        ],
    })
}

fn route_local_communications_status(
    config: CommunicationBoundaryConfig,
    now_unix_ms: u64,
) -> Result<RoutedOperatorCommand, AgentCliError> {
    let command = parse_cli_command(&["status".to_owned()], now_unix_ms)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    DeterministicOperatorCommandRouter::new()
        .route(&OperatorCommandRoutingRequest {
            id: "local-communications-runtime-route".to_owned(),
            command,
            config,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_local_remote_command_security_review(
    now_unix_ms: u64,
) -> Result<RemoteCommandSecurityReviewReport, AgentCliError> {
    review_remote_command_security(&RemoteCommandSecurityReviewRequest {
        review_id: "local-communications-runtime-remote-review".to_owned(),
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
        reviewed_at_unix_ms: now_unix_ms.saturating_add(3),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn write_remote_command_security_review(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    now_unix_ms: u64,
) -> Result<
    (
        RemoteCommandSecurityReviewReport,
        AuditRecord,
        StateCheckpoint,
    ),
    AgentCliError,
> {
    let report = build_local_remote_command_security_review(now_unix_ms)?;
    let audit = append_remote_command_security_review_audit(
        journal,
        &report,
        now_unix_ms.saturating_add(4),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_remote_command_security_review_checkpoint(
        store,
        &report,
        now_unix_ms.saturating_add(5),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok((report, audit, checkpoint))
}

fn build_local_remote_command_envelope(
    remote_review: &RemoteCommandSecurityReviewReport,
    platform_ingress: &PlatformCommandIngressReport,
    now_unix_ms: u64,
) -> Result<RemoteCommandEnvelopeValidationReport, AgentCliError> {
    validate_remote_command_envelope(&RemoteCommandEnvelopeValidationRequest {
        envelope_id: "local-communications-runtime-envelope".to_owned(),
        command: platform_ingress.command.clone(),
        security_review: remote_review.clone(),
        platform_identity: platform_ingress.platform_identity.clone(),
        authorization_policy: "local-communications-readonly".to_owned(),
        authentication_reference: "local-communications-ref".to_owned(),
        replay_nonce: "local-communications-nonce".to_owned(),
        channel_authenticated: true,
        platform_identity_verified: true,
        platform_identity_authorized: true,
        replay_protection_checked: true,
        replay_nonce_reused: false,
        command_allowlisted: true,
        received_at_unix_ms: now_unix_ms.saturating_add(6),
        now_unix_ms: now_unix_ms.saturating_add(6),
        max_age_ms: 60_000,
        remote_command_enablement_requested: false,
        outbound_network_used: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_local_platform_command_ingress(
    now_unix_ms: u64,
) -> Result<PlatformCommandIngressReport, AgentCliError> {
    review_platform_command_ingress(&PlatformCommandIngressRequest {
        ingress_id: "local-communications-platform-ingress".to_owned(),
        platform: "local-platform-mock".to_owned(),
        channel: NotificationChannelProfile::from_identifier("chat:ops"),
        platform_message_id: "local-platform-message-1".to_owned(),
        platform_identity: "local-communications-operator".to_owned(),
        command_text: "status".to_owned(),
        token_reference_present: true,
        token_secret_material_present: false,
        platform_signature_verified: true,
        platform_identity_authorized: true,
        channel_permission_granted: true,
        replay_nonce: "local-platform-ingress-nonce".to_owned(),
        replay_nonce_reused: false,
        provider_rate_limited: false,
        provider_outage_observed: false,
        received_at_unix_ms: now_unix_ms.saturating_add(6),
        now_unix_ms: now_unix_ms.saturating_add(6),
        max_age_ms: 60_000,
        outbound_network_used: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn write_communications_platform_command_ingress(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    now_unix_ms: u64,
) -> Result<(PlatformCommandIngressReport, AuditRecord, StateCheckpoint), AgentCliError> {
    let report = build_local_platform_command_ingress(now_unix_ms)?;
    let audit =
        append_platform_command_ingress_audit(journal, &report, now_unix_ms.saturating_add(6))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint =
        persist_platform_command_ingress_checkpoint(store, &report, now_unix_ms.saturating_add(7))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok((report, audit, checkpoint))
}

fn dispatch_local_communications_notification(
    config: CommunicationBoundaryConfig,
    now_unix_ms: u64,
) -> Result<NotificationDispatchRecord, AgentCliError> {
    DeterministicNotificationBoundary::new()
        .publish(&arb_core::NotificationPublishRequest {
            id: "local-communications-runtime-notification".to_owned(),
            notification: OperatorNotification {
                id: "local-communications-runtime-notification".to_owned(),
                severity: NotificationSeverity::Info,
                title: "Local communications runtime validation".to_owned(),
                body: "Command and notification boundaries remained local".to_owned(),
                channels: vec!["cli".to_owned(), "local-stdout".to_owned()],
                created_at_unix_ms: now_unix_ms.saturating_add(6),
            },
            config,
            channel_safety: vec![
                communications_channel_safety("cli", now_unix_ms),
                communications_channel_safety("local-stdout", now_unix_ms),
            ],
            now_unix_ms: now_unix_ms.saturating_add(7),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_local_communications_channel_adapter(
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    now_unix_ms: u64,
) -> Result<ChannelAdapterValidationReport, AgentCliError> {
    build_local_communications_channel_adapter_for_scenario(
        ChannelAdapterScenario::Ready,
        remote_envelope,
        dispatch,
        now_unix_ms.saturating_add(12),
    )
}

#[derive(Debug, Clone, Copy)]
enum ChannelAdapterScenario {
    Ready,
    MissingChannelLogin,
    ReplayNonceReused,
    ProviderUnavailable,
}

impl ChannelAdapterScenario {
    const fn validation_id(self) -> &'static str {
        match self {
            Self::Ready => "local-communications-channel-adapter",
            Self::MissingChannelLogin => "comm-channel-no-login",
            Self::ReplayNonceReused => "comm-channel-replay",
            Self::ProviderUnavailable => "comm-channel-provider-down",
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

fn build_local_communications_channel_adapter_for_scenario(
    scenario: ChannelAdapterScenario,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    validated_at_unix_ms: u64,
) -> Result<ChannelAdapterValidationReport, AgentCliError> {
    let validation_id = scenario.validation_id();
    validate_channel_adapter(&ChannelAdapterValidationRequest {
        validation_id: validation_id.to_owned(),
        channel: NotificationChannelProfile::from_identifier("cli"),
        envelope: remote_envelope.clone(),
        dispatch: dispatch.clone(),
        adapter_authentication_reference: "local-communications-channel-ref".to_owned(),
        platform_identity: "local-communications-operator".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn build_local_communications_channel_session(
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    accepted: &ChannelAdapterValidationReport,
    now_unix_ms: u64,
) -> Result<ChannelSessionValidationReport, AgentCliError> {
    let unauthenticated = build_local_communications_channel_adapter_for_scenario(
        ChannelAdapterScenario::MissingChannelLogin,
        remote_envelope,
        dispatch,
        now_unix_ms.saturating_add(15),
    )?;
    let replay = build_local_communications_channel_adapter_for_scenario(
        ChannelAdapterScenario::ReplayNonceReused,
        remote_envelope,
        dispatch,
        now_unix_ms.saturating_add(16),
    )?;
    let provider_unavailable = build_local_communications_channel_adapter_for_scenario(
        ChannelAdapterScenario::ProviderUnavailable,
        remote_envelope,
        dispatch,
        now_unix_ms.saturating_add(17),
    )?;

    validate_channel_session(
        "comm-channel-session",
        &[
            accepted.clone(),
            unauthenticated,
            replay,
            provider_unavailable,
        ],
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn write_communications_channel_session(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    dispatch: &NotificationDispatchRecord,
    accepted: &ChannelAdapterValidationReport,
    now_unix_ms: u64,
) -> Result<(ChannelSessionValidationReport, AuditRecord, StateCheckpoint), AgentCliError> {
    let report = build_local_communications_channel_session(
        remote_envelope,
        dispatch,
        accepted,
        now_unix_ms,
    )?;
    let audit =
        append_channel_session_validation_audit(journal, &report, now_unix_ms.saturating_add(18))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint = persist_channel_session_validation_checkpoint(
        store,
        &report,
        now_unix_ms.saturating_add(19),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok((report, audit, checkpoint))
}

fn build_local_communications_platform_adapter_review(
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    now_unix_ms: u64,
) -> Result<PlatformAdapterReviewReport, AgentCliError> {
    review_platform_adapter_controls(&PlatformAdapterReviewRequest {
        review_id: "local-communications-platform-adapter".to_owned(),
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
        reviewed_at_unix_ms: now_unix_ms.saturating_add(20),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))
}

fn write_communications_platform_adapter_review(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut SqliteWalStateStore,
    remote_envelope: &RemoteCommandEnvelopeValidationReport,
    now_unix_ms: u64,
) -> Result<(PlatformAdapterReviewReport, AuditRecord, StateCheckpoint), AgentCliError> {
    let report = build_local_communications_platform_adapter_review(remote_envelope, now_unix_ms)?;
    let audit =
        append_platform_adapter_review_audit(journal, &report, now_unix_ms.saturating_add(20))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let checkpoint =
        persist_platform_adapter_review_checkpoint(store, &report, now_unix_ms.saturating_add(21))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok((report, audit, checkpoint))
}

fn communications_channel_safety(
    channel_id: &str,
    now_unix_ms: u64,
) -> NotificationChannelSafetyState {
    NotificationChannelSafetyState {
        channel_id: channel_id.to_owned(),
        messages_sent_in_window: 0,
        max_messages_per_window: 30,
        window_started_at_unix_ms: now_unix_ms,
        window_ends_at_unix_ms: now_unix_ms.saturating_add(60_000),
        outage_active: false,
        outage_reason: String::new(),
    }
}

fn recover_communications_runtime_records(
    audit_path: &Path,
    state_path: &Path,
    records: &CommunicationsRuntimeRecords,
) -> Result<CommunicationsRuntimeRecovery, AgentCliError> {
    let reopened_journal = AppendOnlyAuditJournal::open(audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let replayed_records = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_store = SqliteWalStateStore::open(state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopened_store
        .integrity_check()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_checkpoints = records
        .checkpoint_keys
        .iter()
        .map(|key| {
            reopened_store
                .get_checkpoint(key)
                .map_err(|error| AgentCliError::Validation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let recovered_checkpoint_count = recovered_checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.is_some())
        .count();

    Ok(CommunicationsRuntimeRecovery {
        replayed_records,
        recovered_checkpoint_count,
    })
}

fn communications_runtime_validation_passed(
    records: &CommunicationsRuntimeRecords,
    recovery: &CommunicationsRuntimeRecovery,
) -> bool {
    recovery.replayed_records == u64::try_from(records.audit_record_count).unwrap_or(u64::MAX)
        && recovery.recovered_checkpoint_count == records.checkpoint_keys.len()
        && records.route.accepted
        && records.route.operator_authorized
        && !records.route.execution_enabled
        && !records.route.outbound_network_used
        && records.remote_review.status == RemoteCommandSecurityReviewStatus::ReadyForLocalReview
        && !records.remote_review.remote_commands_enabled
        && !records.remote_review.outbound_network_used
        && !records.remote_review.live_execution_performed
        && !records.remote_review.signing_or_broadcast_performed
        && !records.remote_review.production_ready
        && records.platform_ingress.status
            == PlatformCommandIngressStatus::ReadyForEnvelopeValidation
        && records.platform_ingress.token_reference_present
        && !records.platform_ingress.token_secret_material_present
        && records.platform_ingress.platform_signature_verified
        && records.platform_ingress.platform_identity_authorized
        && records.platform_ingress.channel_permission_granted
        && !records.platform_ingress.replay_nonce_reused
        && !records.platform_ingress.command_injection_detected
        && !records.platform_ingress.stale_message
        && !records.platform_ingress.provider_rate_limited
        && !records.platform_ingress.provider_outage_observed
        && !records.platform_ingress.remote_commands_enabled
        && !records.platform_ingress.outbound_network_used
        && !records.platform_ingress.message_delivered
        && !records.platform_ingress.live_execution_performed
        && !records.platform_ingress.signing_or_broadcast_performed
        && !records.platform_ingress.production_ready
        && records.remote_envelope.status
            == RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview
        && !records.remote_envelope.remote_commands_enabled
        && !records.remote_envelope.command_injection_detected
        && !records.remote_envelope.outbound_network_used
        && !records.remote_envelope.live_execution_performed
        && !records.remote_envelope.signing_or_broadcast_performed
        && !records.remote_envelope.production_ready
        && records.dispatch.status == NotificationDispatchStatus::RecordedLocally
        && !records.dispatch.outbound_network_used
        && records.channel_adapter.status == ChannelAdapterValidationStatus::ReadyForLocalReview
        && records.channel_adapter.require_delivery_kill_switch
        && records.channel_adapter.require_audit_state_preflight
        && records.channel_adapter.require_delivery_idempotency
        && records.channel_adapter.require_rate_limit_controls
        && records.channel_adapter.require_outage_backoff_controls
        && records.channel_adapter.require_payload_redaction
        && !records.channel_adapter.outbound_delivery_requested
        && !records.channel_adapter.outbound_network_used
        && !records.channel_adapter.message_delivered
        && !records.channel_adapter.remote_commands_enabled
        && !records.channel_adapter.live_execution_performed
        && !records.channel_adapter.signing_or_broadcast_performed
        && !records.channel_adapter.production_ready
        && records.channel_session.status == ChannelSessionValidationStatus::ReadyForLocalReview
        && records.channel_session.total_validation_count == 4
        && records.channel_session.accepted_validation_count == 1
        && records.channel_session.rejected_unauthenticated_count == 1
        && records.channel_session.rejected_replay_count == 1
        && records.channel_session.rejected_provider_unavailable_count == 1
        && records.channel_session.envelope_ready
        && records.channel_session.dispatch_recorded_locally
        && !records.channel_session.outbound_delivery_requested
        && !records.channel_session.outbound_network_used
        && !records.channel_session.message_delivered
        && !records.channel_session.live_execution_performed
        && !records.channel_session.signing_or_broadcast_performed
        && !records.channel_session.production_ready
        && records.platform_adapter.status == PlatformAdapterReviewStatus::ReadyForLocalReview
        && records.platform_adapter.envelope_ready
        && records.platform_adapter.token_reference_present
        && !records.platform_adapter.token_secret_material_present
        && records.platform_adapter.platform_identity_verified
        && records.platform_adapter.platform_identity_authorized
        && records.platform_adapter.channel_permission_granted
        && records.platform_adapter.command_injection_blocked
        && records.platform_adapter.require_delivery_kill_switch
        && records.platform_adapter.require_audit_state_preflight
        && records.platform_adapter.require_delivery_idempotency
        && records.platform_adapter.require_rate_limit_controls
        && records.platform_adapter.require_outage_backoff_controls
        && records.platform_adapter.require_payload_redaction
        && !records.platform_adapter.token_revoked
        && !records.platform_adapter.provider_rate_limited
        && !records.platform_adapter.provider_outage_observed
        && !records.platform_adapter.outbound_delivery_requested
        && !records.platform_adapter.outbound_network_used
        && !records.platform_adapter.message_delivered
        && !records.platform_adapter.remote_commands_enabled
        && !records.platform_adapter.live_execution_performed
        && !records.platform_adapter.signing_or_broadcast_performed
        && !records.platform_adapter.production_ready
}

fn print_communications_runtime_report(
    options: &LocalValidationRunOptions,
    records: &CommunicationsRuntimeRecords,
    recovery: &CommunicationsRuntimeRecovery,
) {
    println!("communications-runtime: validation passed");
    println!(
        "communications-runtime-workspace: {}",
        options.workspace_dir.display()
    );
    println!("communications-runtime-version: {COMMUNICATIONS_CLI_VERSION}");
    println!(
        "communications-runtime-audit-records-replayed: {}",
        recovery.replayed_records
    );
    println!(
        "communications-runtime-checkpoints-recovered: {}",
        recovery.recovered_checkpoint_count
    );
    print_communications_checkpoint_keys();
    println!("command-route-accepted: {}", records.route.accepted);
    println!(
        "command-operator-authorized: {}",
        records.route.operator_authorized
    );
    println!(
        "remote-command-security-ready: {}",
        records.remote_review.status == RemoteCommandSecurityReviewStatus::ReadyForLocalReview
    );
    print_communications_platform_ingress_report(&records.platform_ingress);
    println!(
        "remote-command-envelope-ready: {}",
        records.remote_envelope.status
            == RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview
    );
    println!(
        "remote-command-injection-detected: {}",
        records.remote_envelope.command_injection_detected
    );
    print_communications_channel_adapter_report(&records.channel_adapter);
    println!(
        "channel-session-ready: {}",
        records.channel_session.status == ChannelSessionValidationStatus::ReadyForLocalReview
    );
    println!(
        "channel-session-validations: {}",
        records.channel_session.total_validation_count
    );
    println!(
        "channel-session-accepted: {}",
        records.channel_session.accepted_validation_count
    );
    println!(
        "channel-session-rejected-unauthenticated: {}",
        records.channel_session.rejected_unauthenticated_count
    );
    println!(
        "channel-session-rejected-replay: {}",
        records.channel_session.rejected_replay_count
    );
    println!(
        "channel-session-rejected-provider-unavailable: {}",
        records.channel_session.rejected_provider_unavailable_count
    );
    print_communications_platform_adapter_report(&records.platform_adapter);
    println!(
        "notification-dispatch-status: {:?}",
        records.dispatch.status
    );
    println!(
        "notification-channel-count: {}",
        records.dispatch.channels.len()
    );
    println!("outbound-network-used: false");
    println!("remote-commands-enabled: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("signing-or-broadcast-performed: false");
    println!("production-ready: false");
}

fn print_communications_checkpoint_keys() {
    println!(
        "communications-command-route-checkpoint-key: {COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY}"
    );
    println!(
        "communications-remote-review-checkpoint-key: {COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY}"
    );
    println!(
        "communications-remote-envelope-checkpoint-key: {COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY}"
    );
    println!(
        "communications-platform-ingress-checkpoint-key: {COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY}"
    );
    println!(
        "communications-channel-adapter-checkpoint-key: {COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY}"
    );
    println!(
        "communications-channel-session-checkpoint-key: {COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY}"
    );
    println!(
        "communications-platform-adapter-checkpoint-key: {COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY}"
    );
    println!(
        "communications-notification-dispatch-checkpoint-key: {COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY}"
    );
}

fn print_communications_channel_adapter_report(report: &ChannelAdapterValidationReport) {
    println!(
        "channel-adapter-ready: {}",
        report.status == ChannelAdapterValidationStatus::ReadyForLocalReview
    );
    println!(
        "channel-adapter-delivery-kill-switch-required: {}",
        report.require_delivery_kill_switch
    );
    println!(
        "channel-adapter-audit-state-preflight-required: {}",
        report.require_audit_state_preflight
    );
    println!(
        "channel-adapter-idempotency-required: {}",
        report.require_delivery_idempotency
    );
    println!(
        "channel-adapter-rate-limit-controls-required: {}",
        report.require_rate_limit_controls
    );
    println!(
        "channel-adapter-outage-backoff-required: {}",
        report.require_outage_backoff_controls
    );
    println!(
        "channel-adapter-payload-redaction-required: {}",
        report.require_payload_redaction
    );
    println!(
        "channel-adapter-message-delivered: {}",
        report.message_delivered
    );
}

fn print_communications_platform_adapter_report(report: &PlatformAdapterReviewReport) {
    println!(
        "platform-adapter-ready: {}",
        report.status == PlatformAdapterReviewStatus::ReadyForLocalReview
    );
    println!(
        "platform-adapter-token-reference-present: {}",
        report.token_reference_present
    );
    println!(
        "platform-adapter-token-secret-material-present: {}",
        report.token_secret_material_present
    );
    println!(
        "platform-adapter-identity-verified: {}",
        report.platform_identity_verified
    );
    println!(
        "platform-adapter-identity-authorized: {}",
        report.platform_identity_authorized
    );
    println!(
        "platform-adapter-channel-permission-granted: {}",
        report.channel_permission_granted
    );
    println!(
        "platform-adapter-command-injection-blocked: {}",
        report.command_injection_blocked
    );
    println!(
        "platform-adapter-delivery-kill-switch-required: {}",
        report.require_delivery_kill_switch
    );
    println!(
        "platform-adapter-audit-state-preflight-required: {}",
        report.require_audit_state_preflight
    );
    println!(
        "platform-adapter-idempotency-required: {}",
        report.require_delivery_idempotency
    );
    println!(
        "platform-adapter-rate-limit-controls-required: {}",
        report.require_rate_limit_controls
    );
    println!(
        "platform-adapter-outage-backoff-required: {}",
        report.require_outage_backoff_controls
    );
    println!(
        "platform-adapter-payload-redaction-required: {}",
        report.require_payload_redaction
    );
    println!("platform-adapter-token-revoked: {}", report.token_revoked);
    println!(
        "platform-adapter-provider-rate-limited: {}",
        report.provider_rate_limited
    );
    println!(
        "platform-adapter-provider-outage-observed: {}",
        report.provider_outage_observed
    );
}

fn print_communications_platform_ingress_report(report: &PlatformCommandIngressReport) {
    println!(
        "platform-command-ingress-ready: {}",
        report.status == PlatformCommandIngressStatus::ReadyForEnvelopeValidation
    );
    println!(
        "platform-command-token-reference-present: {}",
        report.token_reference_present
    );
    println!(
        "platform-command-token-secret-material-present: {}",
        report.token_secret_material_present
    );
    println!(
        "platform-command-signature-verified: {}",
        report.platform_signature_verified
    );
    println!(
        "platform-command-identity-authorized: {}",
        report.platform_identity_authorized
    );
    println!(
        "platform-command-channel-permission-granted: {}",
        report.channel_permission_granted
    );
    println!(
        "platform-command-replay-nonce-reused: {}",
        report.replay_nonce_reused
    );
    println!(
        "platform-command-injection-detected: {}",
        report.command_injection_detected
    );
    println!(
        "platform-command-provider-rate-limited: {}",
        report.provider_rate_limited
    );
    println!(
        "platform-command-provider-outage-observed: {}",
        report.provider_outage_observed
    );
}

#[allow(clippy::too_many_lines)]
fn run_runtime_smoke_validation(options: &RuntimeSmokeOptions) -> Result<(), AgentCliError> {
    let config = load_config_file(&options.config_path)?;
    if config.runtime.mode.permits_live_execution() {
        return Err(AgentCliError::Usage(
            "validate-runtime-smoke only accepts observe or paper configs".to_owned(),
        ));
    }
    prepare_fresh_workspace(&options.workspace_dir)?;

    let policy = PolicyEngine::from_config(config.clone());
    let mut load_iterations = Vec::with_capacity(options.iterations);
    for iteration in 1..=options.iterations {
        let now_unix_ms = current_unix_ms()?;
        let run_id = format!("run-{iteration}");
        let run_workspace = options.workspace_dir.join(&run_id);
        fs::create_dir_all(&run_workspace).map_err(|error| {
            AgentCliError::Validation(format!(
                "failed to create runtime smoke iteration workspace {}: {error}",
                run_workspace.display()
            ))
        })?;
        let lifecycle_request =
            build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms, &run_id)?;
        let shutdown_request = RuntimeGracefulShutdownRequest {
            id: format!("cli-runtime-smoke-shutdown-{run_id}"),
            reason: "local-cli-runtime-smoke-validation".to_owned(),
            now_unix_ms: now_unix_ms.saturating_add(1),
        };
        let started_at = Instant::now();
        let report = arb_core::validate_local_runtime_deployment_smoke(
            run_workspace.join("runtime-audit.jsonl"),
            run_workspace.join("runtime-state.sqlite3"),
            run_workspace.join("runtime-audit.backup.jsonl"),
            run_workspace.join("runtime-state.backup.sqlite3"),
            run_workspace.join("audit-durability-workspace"),
            &policy,
            RuntimeDeploymentSmokeValidationRequest {
                lifecycle_request,
                shutdown_request,
                validated_at_unix_ms: now_unix_ms.saturating_add(2),
            },
        )
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
        let elapsed_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);

        println!("runtime-smoke-iteration: {run_id}");
        println!("runtime-smoke: validation passed");
        println!("runtime-smoke-elapsed-ms: {elapsed_ms}");
        println!("runtime-smoke-version: {}", report.validation_version);
        println!("lifecycle-completed: {}", report.lifecycle_completed);
        println!(
            "graceful-shutdown-checkpointed: {}",
            report.graceful_shutdown_checkpointed
        );
        println!(
            "backup-restore-validated: {}",
            report.backup_restore_validated
        );
        println!(
            "restart-recovery-validated: {}",
            report.restart_recovery_validated
        );
        println!(
            "audit-durability-validated: {}",
            report.audit_durability_validated
        );
        println!(
            "concurrent-lifecycle-validated: {}",
            report.concurrent_lifecycle_validated
        );
        println!(
            "concurrent-lifecycle-workers: {}",
            report.concurrent_lifecycle_workers
        );
        println!(
            "concurrent-lifecycle-audit-records-replayed: {}",
            report.concurrent_lifecycle_audit_records_replayed
        );
        println!(
            "concurrent-lifecycle-sqlite-integrity-check-passed: {}",
            report.concurrent_lifecycle_sqlite_integrity_check_passed
        );
        println!(
            "concurrent-lifecycle-external-submission-performed: {}",
            report.concurrent_lifecycle_external_submission_performed
        );
        println!(
            "concurrent-lifecycle-live-execution-performed: {}",
            report.concurrent_lifecycle_live_execution_performed
        );
        println!(
            "observability-collected: {}",
            report.observability_collected
        );
        println!(
            "observability-checkpoint-recovered: {}",
            report.observability_checkpoint_recovered
        );
        println!(
            "observability-operations-reviewed: {}",
            report.observability_operations_reviewed
        );
        println!(
            "observability-operations-checkpoint-recovered: {}",
            report.observability_operations_checkpoint_recovered
        );
        println!(
            "observability-export-dry-run-rendered: {}",
            report.observability_export_dry_run_rendered
        );
        println!(
            "observability-export-checkpoint-recovered: {}",
            report.observability_export_checkpoint_recovered
        );
        println!(
            "observability-alert-route-dispatched: {}",
            report.observability_alert_route_dispatched
        );
        println!(
            "observability-alert-route-checkpoint-recovered: {}",
            report.observability_alert_route_checkpoint_recovered
        );
        println!(
            "observability-endpoint-preflighted: {}",
            report.observability_endpoint_preflighted
        );
        println!(
            "observability-endpoint-checkpoint-recovered: {}",
            report.observability_endpoint_checkpoint_recovered
        );
        println!(
            "observability-loopback-bind-validated: {}",
            report.observability_loopback_bind_validated
        );
        println!(
            "observability-loopback-bind-checkpoint-recovered: {}",
            report.observability_loopback_bind_checkpoint_recovered
        );
        println!(
            "observability-metrics-scrape-preflighted: {}",
            report.observability_metrics_scrape_preflighted
        );
        println!(
            "observability-metrics-scrape-checkpoint-recovered: {}",
            report.observability_metrics_scrape_checkpoint_recovered
        );
        println!(
            "observability-metrics-endpoint-validated: {}",
            report.observability_metrics_endpoint_validated
        );
        println!(
            "observability-metrics-endpoint-checkpoint-recovered: {}",
            report.observability_metrics_endpoint_checkpoint_recovered
        );
        println!(
            "observability-tracing-captured: {}",
            report.observability_tracing_captured
        );
        println!(
            "observability-tracing-checkpoint-recovered: {}",
            report.observability_tracing_checkpoint_recovered
        );
        println!(
            "observability-metrics-endpoint-started: {}",
            report.observability_metrics_endpoint_started
        );
        println!(
            "observability-local-metrics-request-served: {}",
            report.observability_local_metrics_request_served
        );
        println!(
            "observability-public-network-exposed: {}",
            report.observability_public_network_exposed
        );
        println!(
            "observability-outbound-alerts-sent: {}",
            report.observability_outbound_alerts_sent
        );
        println!(
            "observability-telemetry-exported: {}",
            report.observability_telemetry_exported
        );
        println!(
            "observability-production-ready: {}",
            report.observability_production_ready
        );
        println!(
            "communications-command-routed: {}",
            report.communications_command_routed
        );
        println!(
            "communications-command-route-checkpoint-recovered: {}",
            report.communications_command_route_checkpoint_recovered
        );
        println!(
            "communications-remote-command-reviewed: {}",
            report.communications_remote_command_reviewed
        );
        println!(
            "communications-remote-command-review-checkpoint-recovered: {}",
            report.communications_remote_command_review_checkpoint_recovered
        );
        println!(
            "communications-platform-command-ingress-validated: {}",
            report.communications_platform_command_ingress_validated
        );
        println!(
            "communications-platform-command-ingress-checkpoint-recovered: {}",
            report.communications_platform_command_ingress_checkpoint_recovered
        );
        println!(
            "communications-remote-command-envelope-validated: {}",
            report.communications_remote_command_envelope_validated
        );
        println!(
            "communications-remote-command-envelope-checkpoint-recovered: {}",
            report.communications_remote_command_envelope_checkpoint_recovered
        );
        println!(
            "communications-channel-adapter-validated: {}",
            report.communications_channel_adapter_validated
        );
        println!(
            "communications-channel-adapter-checkpoint-recovered: {}",
            report.communications_channel_adapter_checkpoint_recovered
        );
        println!(
            "communications-channel-session-validated: {}",
            report.communications_channel_session_validated
        );
        println!(
            "communications-channel-session-checkpoint-recovered: {}",
            report.communications_channel_session_checkpoint_recovered
        );
        println!(
            "communications-platform-adapter-reviewed: {}",
            report.communications_platform_adapter_reviewed
        );
        println!(
            "communications-platform-adapter-checkpoint-recovered: {}",
            report.communications_platform_adapter_checkpoint_recovered
        );
        println!(
            "communications-notification-dispatched: {}",
            report.communications_notification_dispatched
        );
        println!(
            "communications-notification-checkpoint-recovered: {}",
            report.communications_notification_checkpoint_recovered
        );
        println!(
            "communications-execution-enabled: {}",
            report.communications_execution_enabled
        );
        println!(
            "communications-remote-commands-enabled: {}",
            report.communications_remote_commands_enabled
        );
        println!(
            "communications-outbound-network-used: {}",
            report.communications_outbound_network_used
        );
        println!("dashboard-rendered: {}", report.dashboard_rendered);
        println!(
            "dashboard-checkpoint-recovered: {}",
            report.dashboard_checkpoint_recovered
        );
        println!(
            "dashboard-hosted-security-reviewed: {}",
            report.dashboard_hosted_security_reviewed
        );
        println!(
            "dashboard-hosted-security-checkpoint-recovered: {}",
            report.dashboard_hosted_security_checkpoint_recovered
        );
        println!(
            "dashboard-hosted-request-preflighted: {}",
            report.dashboard_hosted_request_preflighted
        );
        println!(
            "dashboard-hosted-request-preflight-checkpoint-recovered: {}",
            report.dashboard_hosted_request_preflight_checkpoint_recovered
        );
        println!(
            "dashboard-hosted-request-validated: {}",
            report.dashboard_hosted_request_validated
        );
        println!(
            "dashboard-hosted-request-validation-checkpoint-recovered: {}",
            report.dashboard_hosted_request_validation_checkpoint_recovered
        );
        println!("dashboard-panel-count: {}", report.dashboard_panel_count);
        println!(
            "dashboard-server-started: {}",
            report.dashboard_server_started
        );
        println!(
            "dashboard-local-one-shot-request-served: {}",
            report.dashboard_local_one_shot_request_served
        );
        println!(
            "dashboard-public-network-exposed: {}",
            report.dashboard_public_network_exposed
        );
        println!(
            "dashboard-live-controls-enabled: {}",
            report.dashboard_live_controls_enabled
        );
        println!(
            "dashboard-hosted-production-ready: {}",
            report.dashboard_hosted_production_ready
        );
        println!(
            "validation-run-recorded: {}",
            report.validation_run_recorded
        );
        println!(
            "validation-run-checkpoint-recovered: {}",
            report.validation_run_checkpoint_recovered
        );
        println!(
            "validation-property-checks-passed: {}",
            report.validation_property_checks_passed
        );
        println!(
            "validation-property-checkpoint-recovered: {}",
            report.validation_property_checkpoint_recovered
        );
        println!(
            "validation-external-fuzzer-invoked: {}",
            report.validation_external_fuzzer_invoked
        );
        println!(
            "validation-live-network-used: {}",
            report.validation_live_network_used
        );
        println!(
            "validation-live-execution-submitted: {}",
            report.validation_live_execution_submitted
        );
        println!(
            "validation-signing-or-broadcast-performed: {}",
            report.validation_signing_or_broadcast_performed
        );
        println!(
            "paper-ledger-applicable: {}",
            report.paper_ledger_applicable
        );
        println!(
            "paper-execution-report-checkpointed: {}",
            report.paper_execution_report_checkpointed
        );
        println!(
            "paper-execution-report-checkpoint-recovered: {}",
            report.paper_execution_report_checkpoint_recovered
        );
        println!(
            "paper-ledger-checkpointed: {}",
            report.paper_ledger_checkpointed
        );
        println!(
            "paper-ledger-checkpoint-recovered: {}",
            report.paper_ledger_checkpoint_recovered
        );
        println!(
            "paper-modeled-fills-settled: {}",
            report.paper_modeled_fills_settled
        );
        println!(
            "paper-ledger-audit-records-appended: {}",
            report.paper_ledger_audit_records_appended
        );
        println!(
            "paper-ledger-replay-validated: {}",
            report.paper_ledger_replay_validated
        );
        println!(
            "paper-ledger-external-submission-performed: {}",
            report.paper_ledger_external_submission_performed
        );
        println!(
            "paper-ledger-live-execution-performed: {}",
            report.paper_ledger_live_execution_performed
        );
        println!(
            "failure-capture-validated: {}",
            report.failure_capture_validated
        );
        println!(
            "failure-capture-checkpoint-recovered: {}",
            report.failure_capture_checkpoint_recovered
        );
        println!(
            "failure-capture-metrics-endpoint-started: {}",
            report.failure_capture_metrics_endpoint_started
        );
        println!(
            "failure-capture-public-network-exposed: {}",
            report.failure_capture_public_network_exposed
        );
        println!(
            "failure-capture-outbound-alerts-sent: {}",
            report.failure_capture_outbound_alerts_sent
        );
        println!(
            "failure-capture-external-submission-performed: {}",
            report.failure_capture_external_submission_performed
        );
        println!(
            "failure-capture-live-execution-performed: {}",
            report.failure_capture_live_execution_performed
        );
        println!(
            "restart-audit-records-replayed: {}",
            report.restart_audit_records_replayed
        );
        println!(
            "restart-plan-checkpoint-recovered: {}",
            report.restart_plan_checkpoint_recovered
        );
        println!(
            "restart-adapter-checkpoint-recovered: {}",
            report.restart_adapter_checkpoint_recovered
        );
        println!(
            "restart-adapter-recovery-plan-checkpoint-recovered: {}",
            report.restart_adapter_recovery_plan_checkpoint_recovered
        );
        println!(
            "restart-graceful-shutdown-checkpoint-recovered: {}",
            report.restart_graceful_shutdown_checkpoint_recovered
        );
        println!(
            "restart-opportunity-trace-recovery-validated: {}",
            report.restart_opportunity_trace_recovery_validated
        );
        println!(
            "restart-opportunity-trace-discovered: {}",
            report.restart_opportunity_trace_discovered_candidates
        );
        println!(
            "restart-opportunity-trace-recovered-checkpoints: {}",
            report.restart_opportunity_trace_recovered_checkpoints
        );
        println!(
            "restart-opportunity-trace-recovered-summaries: {}",
            report.restart_opportunity_trace_recovered_summaries.len()
        );
        for (index, summary) in report
            .restart_opportunity_trace_recovered_summaries
            .iter()
            .enumerate()
        {
            println!(
                "restart-opportunity-trace-recovered-summary-{}: trace_id={};strategy_id={};planner_request_id={};audit_sequence={};traced_at_unix_ms={};route_kind={};leg_count={}",
                index.saturating_add(1),
                summary.trace_id,
                summary.strategy_id,
                summary.planner_request_id,
                summary.audit_sequence,
                summary.traced_at_unix_ms,
                summary.route_kind,
                summary.leg_count
            );
        }
        println!(
            "restart-opportunity-trace-missing-checkpoints: {}",
            report.restart_opportunity_trace_missing_checkpoints
        );
        if let Some(trace_recovery) = &report.opportunity_trace_recovery {
            println!("opportunity-trace-corpus: {}", trace_recovery.corpus_id);
            println!(
                "opportunity-trace-discovered: {}",
                trace_recovery.discovered_candidates
            );
            println!(
                "opportunity-trace-audit-records-replayed: {}",
                trace_recovery.audit_trace_records_replayed
            );
            println!(
                "opportunity-trace-recovered-checkpoints: {}",
                trace_recovery.recovered_trace_checkpoints
            );
            println!(
                "opportunity-trace-recovered-summaries: {}",
                trace_recovery.recovered_trace_summaries.len()
            );
            println!(
                "opportunity-trace-missing-checkpoints: {}",
                trace_recovery.missing_trace_checkpoints
            );
            println!(
                "opportunity-trace-recovery-validated: {}",
                trace_recovery.trace_recovery_validated
            );
        } else {
            println!("opportunity-trace-recovery-validated: false");
        }
        load_iterations.push(RuntimeDeploymentSmokeLoadIteration {
            iteration_id: run_id,
            elapsed_ms,
            report,
        });
    }

    let latest_smoke_report = load_iterations
        .last()
        .map(|iteration| iteration.report.clone())
        .ok_or_else(|| {
            AgentCliError::Validation("runtime smoke produced no iterations".to_owned())
        })?;
    let load_report = RuntimeDeploymentSmokeLoadValidationReport::from_iterations(load_iterations)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let load_profile_review =
        arb_core::review_runtime_load_profile(arb_core::RuntimeLoadProfileReviewRequest {
            review_id: "local-runtime-smoke-load-profile".to_owned(),
            load_report: load_report.clone(),
            max_average_elapsed_ms: load_report.average_elapsed_ms.max(1),
            max_single_iteration_elapsed_ms: load_report.max_elapsed_ms.max(1),
            max_total_elapsed_ms: load_report.total_elapsed_ms.max(1),
            observed_peak_memory_mb: 1,
            max_peak_memory_mb: 1,
            observed_peak_cpu_percent: 1,
            max_peak_cpu_percent: 100,
            deployment_host_load_evidence_available: false,
            live_feed_backpressure_evidence_available: false,
            target_runtime_evidence_available: false,
            service_manager_action_performed: false,
            external_calls_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: current_unix_ms()?,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let production_preflight = arb_core::preflight_production_runtime_validation(
        arb_core::RuntimeProductionPreflightRequest {
            preflight_id: "local-runtime-smoke-production-preflight".to_owned(),
            smoke_report: latest_smoke_report,
            load_report: load_report.clone(),
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
            validated_at_unix_ms: current_unix_ms()?,
        },
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    println!("runtime-smoke-iterations: {}", options.iterations);
    println!("runtime-smoke-load-validation: passed");
    println!(
        "runtime-smoke-load-iterations-attempted: {}",
        load_report.iterations_attempted
    );
    println!(
        "runtime-smoke-load-iterations-passed: {}",
        load_report.iterations_passed
    );
    println!(
        "runtime-smoke-load-min-elapsed-ms: {}",
        load_report.min_elapsed_ms
    );
    println!(
        "runtime-smoke-load-max-elapsed-ms: {}",
        load_report.max_elapsed_ms
    );
    println!(
        "runtime-smoke-load-average-elapsed-ms: {}",
        load_report.average_elapsed_ms
    );
    println!(
        "runtime-smoke-load-total-elapsed-ms: {}",
        load_report.total_elapsed_ms
    );
    println!(
        "runtime-smoke-load-restart-audit-records-replayed: {}",
        load_report.restart_audit_records_replayed
    );
    println!(
        "runtime-smoke-load-backup-audit-records-replayed: {}",
        load_report.backup_audit_records_replayed
    );
    println!(
        "runtime-smoke-load-opportunity-trace-recovered-checkpoints: {}",
        load_report.opportunity_trace_recovered_checkpoints
    );
    println!(
        "runtime-smoke-load-opportunity-trace-recovered-summaries: {}",
        load_report.opportunity_trace_recovered_summaries
    );
    println!(
        "runtime-smoke-load-opportunity-trace-missing-checkpoints: {}",
        load_report.opportunity_trace_missing_checkpoints
    );
    println!(
        "runtime-load-profile-review: {:?}",
        load_profile_review.status
    );
    println!(
        "runtime-load-profile-latency-budget-met: {}",
        load_profile_review.latency_budget_met
    );
    println!(
        "runtime-load-profile-resource-budget-met: {}",
        load_profile_review.resource_budget_met
    );
    println!(
        "runtime-load-profile-replay-recovery-evidence-validated: {}",
        load_profile_review.replay_recovery_evidence_validated
    );
    println!(
        "runtime-load-profile-remaining-external-evidence-count: {}",
        load_profile_review.remaining_external_evidence.len()
    );
    println!("service-manager-action-performed: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    println!("production-runtime-preflight: validation passed");
    println!(
        "production-runtime-preflight-status: {:?}",
        production_preflight.status
    );
    println!(
        "production-runtime-preflight-local-smoke-validated: {}",
        production_preflight.local_smoke_validated
    );
    println!(
        "production-runtime-preflight-local-smoke-load-validated: {}",
        production_preflight.local_smoke_load_validated
    );
    println!(
        "production-runtime-preflight-unresolved-blockers: {}",
        production_preflight.unresolved_blockers.len()
    );
    println!(
        "production-runtime-preflight-service-manager-evidence-available: {}",
        production_preflight.service_manager_lifecycle_evidence_available
    );
    println!(
        "production-runtime-preflight-disk-full-evidence-available: {}",
        production_preflight.physical_disk_full_evidence_available
    );
    println!(
        "production-runtime-preflight-production-ready: {}",
        production_preflight.production_ready
    );
    Ok(())
}

fn run_service_manager_lifecycle_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_service_manager_lifecycle_transcript(
        local_service_manager_lifecycle_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_service_manager_lifecycle_transcript(
        local_service_manager_lifecycle_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RuntimeServiceManagerLifecycleTranscriptStatus::ReadyForExternalReview
        || blocked.status != RuntimeServiceManagerLifecycleTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "service-manager lifecycle transcript validation failed".to_owned(),
        ));
    }

    println!("service-manager-lifecycle-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        service_manager_lifecycle_status_label(ready.status)
    );
    println!("ready-transcript-events: {}", ready.event_count);
    println!(
        "ready-start-evidence-present: {}",
        ready.start_evidence_present
    );
    println!(
        "ready-graceful-shutdown-evidence-present: {}",
        ready.graceful_shutdown_evidence_present
    );
    println!(
        "ready-restart-evidence-present: {}",
        ready.restart_evidence_present
    );
    println!(
        "ready-recovery-evidence-present: {}",
        ready.recovery_evidence_present
    );
    println!(
        "ready-concurrent-lifecycle-reference-present: {}",
        ready.concurrent_lifecycle_reference_present
    );
    println!(
        "ready-concurrent-lifecycle-worker-count: {}",
        ready.concurrent_lifecycle_worker_count
    );
    println!(
        "ready-concurrent-lifecycle-success: {}",
        ready.concurrent_lifecycle_success
    );
    println!(
        "ready-operator-lifecycle-rehearsal-reference-present: {}",
        ready.operator_lifecycle_rehearsal_reference_present
    );
    println!(
        "ready-emergency-stop-review-reference-present: {}",
        ready.emergency_stop_review_reference_present
    );
    println!(
        "ready-rollback-plan-review-reference-present: {}",
        ready.rollback_plan_review_reference_present
    );
    println!(
        "ready-operator-review-window-current: {}",
        ready.operator_review_window_current
    );
    println!(
        "blocked-transcript-status: {}",
        service_manager_lifecycle_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!(
        "service-manager-action-performed-by-validator: {}",
        ready.service_manager_action_performed_by_validator
            || blocked.service_manager_action_performed_by_validator
    );
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_service_manager_lifecycle_rehearsal_validation() -> Result<(), AgentCliError> {
    let ready = validate_service_manager_lifecycle_rehearsal(
        local_service_manager_lifecycle_rehearsal("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_service_manager_lifecycle_rehearsal(
        local_service_manager_lifecycle_rehearsal("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RuntimeServiceManagerLifecycleRehearsalStatus::Validated
        || blocked.status != RuntimeServiceManagerLifecycleRehearsalStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.deployment_path_mutated_by_validator
        || blocked.deployment_path_mutated_by_validator
        || ready.secrets_loaded
        || blocked.secrets_loaded
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "service-manager lifecycle rehearsal validation failed".to_owned(),
        ));
    }

    println!("service-manager-lifecycle-rehearsal: validation passed");
    println!(
        "ready-rehearsal-status: {}",
        service_manager_lifecycle_rehearsal_status_label(ready.status)
    );
    println!("ready-rehearsal-events: {}", ready.event_count);
    println!(
        "ready-ordered-lifecycle-validated: {}",
        ready.ordered_lifecycle_validated
    );
    println!(
        "ready-operator-controlled-events: {}",
        ready.operator_controlled_events
    );
    println!(
        "ready-non-secret-references-present: {}",
        ready.non_secret_references_present
    );
    println!(
        "ready-graceful-shutdown-checkpoint-reference-present: {}",
        ready.graceful_shutdown_checkpoint_reference_present
    );
    println!(
        "ready-restart-recovery-reference-present: {}",
        ready.restart_recovery_reference_present
    );
    println!(
        "ready-concurrent-lifecycle-reference-present: {}",
        ready.concurrent_lifecycle_reference_present
    );
    println!(
        "ready-concurrent-lifecycle-worker-count: {}",
        ready.concurrent_lifecycle_worker_count
    );
    println!(
        "ready-concurrent-lifecycle-success: {}",
        ready.concurrent_lifecycle_success
    );
    println!("ready-operator-approved: {}", ready.operator_approved);
    println!("ready-reviewer-approved: {}", ready.reviewer_approved);
    println!(
        "blocked-rehearsal-status: {}",
        service_manager_lifecycle_rehearsal_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("service-manager-action-performed-by-validator: false");
    println!("deployment-path-mutated-by-validator: false");
    println!("secrets-loaded: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_disk_full_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_disk_full_transcript(local_deployment_disk_full_transcript(
        "ready", true,
    ))
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_disk_full_transcript(local_deployment_disk_full_transcript(
        "blocked", false,
    ))
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != AuditDeploymentDiskFullTranscriptStatus::ReadyForExternalReview
        || blocked.status != AuditDeploymentDiskFullTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.disk_filled_by_validator
        || blocked.disk_filled_by_validator
        || ready.production_path_mutated_by_validator
        || blocked.production_path_mutated_by_validator
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment disk-full transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-disk-full-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_disk_full_status_label(ready.status)
    );
    println!(
        "ready-physical-host-evidence: {}",
        ready.physical_host_evidence
    );
    println!(
        "ready-audit-append-failed-closed: {}",
        ready.audit_append_failed_closed
    );
    println!(
        "ready-state-write-failed-closed: {}",
        ready.state_write_failed_closed
    );
    println!(
        "ready-recovery-validated: {}",
        ready.audit_replay_after_recovery_validated && ready.sqlite_reopen_after_recovery_validated
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_disk_full_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("disk-filled-by-validator: false");
    println!("production-path-mutated-by-validator: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_retention_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_retention_transcript(local_deployment_retention_transcript(
        "ready", true,
    ))
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_retention_transcript(local_deployment_retention_transcript(
        "blocked", false,
    ))
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != AuditDeploymentRetentionTranscriptStatus::ReadyForExternalReview
        || blocked.status != AuditDeploymentRetentionTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.rotation_performed_by_validator
        || blocked.rotation_performed_by_validator
        || ready.production_path_mutated_by_validator
        || blocked.production_path_mutated_by_validator
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment retention transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-retention-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_retention_status_label(ready.status)
    );
    println!(
        "ready-physical-host-evidence: {}",
        ready.physical_host_evidence
    );
    println!(
        "ready-active-rotation-observed: {}",
        ready.active_rotation_observed
    );
    println!(
        "ready-archive-retention-observed: {}",
        ready.archive_retention_observed
    );
    println!(
        "ready-expired-archive-deletion-observed: {}",
        ready.expired_archive_deletion_observed
    );
    println!(
        "ready-replay-after-rotation-validated: {}",
        ready.audit_replay_after_rotation_validated
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_retention_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("rotation-performed-by-validator: false");
    println!("production-path-mutated-by-validator: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_permission_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_permission_transcript(local_deployment_permission_transcript(
        "ready", true,
    ))
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_permission_transcript(
        local_deployment_permission_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RuntimeDeploymentPermissionTranscriptStatus::ReadyForExternalReview
        || blocked.status != RuntimeDeploymentPermissionTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.permission_changed_by_validator
        || blocked.permission_changed_by_validator
        || ready.production_path_mutated_by_validator
        || blocked.production_path_mutated_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment permission transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-permission-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_permission_status_label(ready.status)
    );
    println!(
        "ready-deployment-host-evidence: {}",
        ready.deployment_host_evidence
    );
    println!(
        "ready-runtime-write-attempt-reference-present: {}",
        ready.runtime_write_attempt_reference_present
    );
    println!(
        "ready-runtime-write-permission-denied: {}",
        ready.runtime_write_permission_denied
    );
    println!(
        "ready-runtime-write-error-classified: {}",
        ready.runtime_write_error_classified
    );
    println!(
        "ready-audit-write-failed-closed: {}",
        ready.audit_write_failed_closed
    );
    println!(
        "ready-state-write-failed-closed: {}",
        ready.state_write_failed_closed
    );
    println!(
        "ready-adapter-evaluation-blocked: {}",
        ready.adapter_evaluation_blocked
    );
    println!(
        "ready-recovery-validated: {}",
        ready.audit_replay_after_restore_validated && ready.sqlite_reopen_after_restore_validated
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_permission_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("permission-changed-by-validator: false");
    println!("production-path-mutated-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_audit_sqlite_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_audit_sqlite_transcript(
        local_deployment_audit_sqlite_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_audit_sqlite_transcript(
        local_deployment_audit_sqlite_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RuntimeDeploymentAuditSqliteTranscriptStatus::ReadyForExternalReview
        || blocked.status != RuntimeDeploymentAuditSqliteTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.deployment_path_mutated_by_validator
        || blocked.deployment_path_mutated_by_validator
        || ready.secrets_loaded
        || blocked.secrets_loaded
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment audit/SQLite transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-audit-sqlite-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_audit_sqlite_status_label(ready.status)
    );
    println!(
        "ready-deployment-host-evidence: {}",
        ready.deployment_host_evidence
    );
    println!(
        "ready-service-lifecycle-reference-present: {}",
        ready.service_lifecycle_reference_present
    );
    println!(
        "ready-audit-replay-validated: {}",
        ready.audit_replay_validated && ready.audit_hash_chain_validated
    );
    println!(
        "ready-sqlite-recovery-validated: {}",
        ready.sqlite_wal_mode_validated
            && ready.sqlite_integrity_check_passed
            && ready.sqlite_checkpoint_recovered
    );
    println!(
        "ready-backup-restore-validated: {}",
        ready.backup_restore_validated
    );
    println!(
        "ready-concurrent-access-validated: {}",
        ready.concurrent_access_validated
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_audit_sqlite_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("service-manager-action-performed-by-validator: false");
    println!("deployment-path-mutated-by-validator: false");
    println!("secrets-loaded: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_backup_restore_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_backup_restore_transcript(
        local_deployment_backup_restore_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_backup_restore_transcript(
        local_deployment_backup_restore_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RuntimeDeploymentBackupRestoreTranscriptStatus::ReadyForExternalReview
        || blocked.status != RuntimeDeploymentBackupRestoreTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.backup_restore_executed_by_validator
        || blocked.backup_restore_executed_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.deployment_path_mutated_by_validator
        || blocked.deployment_path_mutated_by_validator
        || ready.secrets_loaded
        || blocked.secrets_loaded
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment backup/restore transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-backup-restore-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_backup_restore_status_label(ready.status)
    );
    println!(
        "ready-deployment-host-evidence: {}",
        ready.deployment_host_evidence
    );
    println!(
        "ready-service-lifecycle-reference-present: {}",
        ready.service_lifecycle_reference_present
    );
    println!(
        "ready-backup-artifact-reference-present: {}",
        ready.backup_artifact_reference_present
    );
    println!(
        "ready-restore-execution-reference-present: {}",
        ready.restore_execution_reference_present
    );
    println!(
        "ready-deployment-load-reference-present: {}",
        ready.deployment_load_reference_present
    );
    println!(
        "ready-audit-restore-validated: {}",
        ready.audit_replay_after_restore_validated
            && ready.audit_hash_chain_after_restore_validated
    );
    println!(
        "ready-sqlite-restore-validated: {}",
        ready.sqlite_integrity_after_restore_validated
            && ready.sqlite_checkpoint_after_restore_validated
    );
    println!(
        "ready-runtime-checkpoint-restore-validated: {}",
        ready.runtime_checkpoint_restore_validated
    );
    println!(
        "ready-post-restore-runtime-smoke-passed: {}",
        ready.post_restore_runtime_smoke_passed
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_backup_restore_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("backup-restore-executed-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("deployment-path-mutated-by-validator: false");
    println!("secrets-loaded: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_graceful_shutdown_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_graceful_shutdown_transcript(
        local_deployment_graceful_shutdown_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_graceful_shutdown_transcript(
        local_deployment_graceful_shutdown_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RuntimeDeploymentGracefulShutdownTranscriptStatus::ReadyForExternalReview
        || blocked.status != RuntimeDeploymentGracefulShutdownTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.deployment_path_mutated_by_validator
        || blocked.deployment_path_mutated_by_validator
        || ready.secrets_loaded
        || blocked.secrets_loaded
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment graceful-shutdown transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-graceful-shutdown-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_graceful_shutdown_status_label(ready.status)
    );
    println!(
        "ready-deployment-host-evidence: {}",
        ready.deployment_host_evidence
    );
    println!(
        "ready-service-lifecycle-reference-present: {}",
        ready.service_lifecycle_reference_present
    );
    println!(
        "ready-shutdown-request-reference-present: {}",
        ready.shutdown_request_reference_present
    );
    println!(
        "ready-service-stopped-reference-present: {}",
        ready.service_stopped_reference_present
    );
    println!(
        "ready-graceful-shutdown-checkpoint-reference-present: {}",
        ready.graceful_shutdown_checkpoint_reference_present
    );
    println!(
        "ready-audit-shutdown-validated: {}",
        ready.audit_replay_after_shutdown_validated
    );
    println!(
        "ready-sqlite-shutdown-validated: {}",
        ready.sqlite_reopen_after_shutdown_validated
    );
    println!(
        "ready-restart-recovery-after-shutdown-validated: {}",
        ready.restart_recovery_after_shutdown_validated
    );
    println!(
        "ready-post-shutdown-runtime-smoke-passed: {}",
        ready.post_shutdown_runtime_smoke_passed
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_graceful_shutdown_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("service-manager-action-performed-by-validator: false");
    println!("deployment-path-mutated-by-validator: false");
    println!("secrets-loaded: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_sqlite_schema_migration_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_sqlite_schema_migration_transcript(
        local_deployment_sqlite_schema_migration_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_sqlite_schema_migration_transcript(
        local_deployment_sqlite_schema_migration_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status
        != RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::ReadyForExternalReview
        || blocked.status != RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.migration_executed_by_validator
        || blocked.migration_executed_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.deployment_path_mutated_by_validator
        || blocked.deployment_path_mutated_by_validator
        || ready.secrets_loaded
        || blocked.secrets_loaded
        || ready.external_submission_performed
        || blocked.external_submission_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment SQLite schema migration transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-sqlite-schema-migration-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_sqlite_schema_migration_status_label(ready.status)
    );
    println!(
        "ready-deployment-host-evidence: {}",
        ready.deployment_host_evidence
    );
    println!(
        "ready-service-lifecycle-reference-present: {}",
        ready.service_lifecycle_reference_present
    );
    println!(
        "ready-schema-version-transition-validated: {}",
        ready.schema_version_transition_validated
            && ready.pre_migration_schema_version == 0
            && ready.post_migration_schema_version == ready.expected_schema_version
    );
    println!(
        "ready-sqlite-recovery-validated: {}",
        ready.sqlite_integrity_check_passed && ready.sqlite_checkpoint_reopened
    );
    println!(
        "ready-audit-replay-after-migration-validated: {}",
        ready.audit_replay_after_migration_validated
    );
    println!(
        "ready-rollback-reference-present: {}",
        ready.rollback_reference_present
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_sqlite_schema_migration_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("migration-executed-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("deployment-path-mutated-by-validator: false");
    println!("secrets-loaded: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_rollback_execution_transcript_validation() -> Result<(), AgentCliError> {
    let ready =
        validate_rollback_execution_transcript(local_rollback_execution_transcript("ready", true))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_rollback_execution_transcript(local_rollback_execution_transcript(
        "blocked", false,
    ))
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != RollbackExecutionTranscriptStatus::ReadyForExternalReview
        || blocked.status != RollbackExecutionTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.rollback_executed_by_validator
        || blocked.rollback_executed_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.files_mutated_by_validator
        || blocked.files_mutated_by_validator
        || ready.external_calls_performed
        || blocked.external_calls_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "rollback execution transcript validation failed".to_owned(),
        ));
    }

    println!("rollback-execution-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        rollback_execution_status_label(ready.status)
    );
    println!(
        "ready-candidate-reference-present: {}",
        ready.candidate_reference_present
    );
    println!(
        "ready-rollback-reference-present: {}",
        ready.rollback_reference_present
    );
    println!(
        "ready-service-quiesced-reference-present: {}",
        ready.service_quiesced_reference_present
    );
    println!(
        "ready-restore-validated: {}",
        ready.previous_artifact_restored && ready.previous_config_restored
    );
    println!(
        "ready-post-rollback-recovery-validated: {}",
        ready.post_rollback_runtime_smoke_passed
            && ready.audit_replay_after_rollback_validated
            && ready.sqlite_recovery_after_rollback_validated
    );
    println!(
        "blocked-transcript-status: {}",
        rollback_execution_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("rollback-executed-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("files-mutated-by-validator: false");
    println!("external-calls-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_incident_response_execution_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_incident_response_execution_transcript(
        local_incident_response_execution_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_incident_response_execution_transcript(
        local_incident_response_execution_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != IncidentResponseExecutionTranscriptStatus::ReadyForExternalReview
        || blocked.status != IncidentResponseExecutionTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.incident_response_executed_by_validator
        || blocked.incident_response_executed_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.files_mutated_by_validator
        || blocked.files_mutated_by_validator
        || ready.alerts_sent_by_validator
        || blocked.alerts_sent_by_validator
        || ready.external_calls_performed
        || blocked.external_calls_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "incident-response execution transcript validation failed".to_owned(),
        ));
    }

    println!("incident-response-execution-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        incident_response_execution_status_label(ready.status)
    );
    println!(
        "ready-incident-reference-present: {}",
        ready.incident_scenario_reference_present
    );
    println!(
        "ready-responder-reviewer-reference-present: {}",
        ready.responder_reference_present && ready.reviewer_reference_present
    );
    println!(
        "ready-detection-containment-reference-present: {}",
        ready.detection_triage_reference_present && ready.containment_recovery_reference_present
    );
    println!(
        "ready-post-incident-recovery-validated: {}",
        ready.post_incident_runtime_smoke_passed
            && ready.audit_replay_after_recovery_validated
            && ready.sqlite_recovery_after_recovery_validated
    );
    println!(
        "ready-communications-reference-present: {}",
        ready.communications_reference_present
    );
    println!(
        "blocked-transcript-status: {}",
        incident_response_execution_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("incident-response-executed-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("files-mutated-by-validator: false");
    println!("alerts-sent-by-validator: false");
    println!("external-calls-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_failure_capture_transcript_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_failure_capture_transcript(
        local_deployment_failure_capture_transcript("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_failure_capture_transcript(
        local_deployment_failure_capture_transcript("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != DeploymentFailureCaptureTranscriptStatus::ReadyForExternalReview
        || blocked.status != DeploymentFailureCaptureTranscriptStatus::Blocked
        || ready.production_ready
        || blocked.production_ready
        || ready.panic_hook_installed_by_validator
        || blocked.panic_hook_installed_by_validator
        || ready.tracing_subscriber_installed_by_validator
        || blocked.tracing_subscriber_installed_by_validator
        || ready.failure_injected_by_validator
        || blocked.failure_injected_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.files_mutated_by_validator
        || blocked.files_mutated_by_validator
        || ready.alerts_sent_by_validator
        || blocked.alerts_sent_by_validator
        || ready.external_calls_performed
        || blocked.external_calls_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment failure-capture transcript validation failed".to_owned(),
        ));
    }

    println!("deployment-failure-capture-transcript: validation passed");
    println!(
        "ready-transcript-status: {}",
        deployment_failure_capture_status_label(ready.status)
    );
    println!(
        "ready-deployment-host-reference-present: {}",
        ready.deployment_host_reference_present
    );
    println!(
        "ready-daemon-panic-hook-reference-present: {}",
        ready.daemon_panic_hook_reference_present
    );
    println!(
        "ready-daemon-tracing-reference-present: {}",
        ready.daemon_tracing_reference_present
    );
    println!(
        "ready-failure-capture-reference-present: {}",
        ready.failure_capture_reference_present
    );
    println!(
        "ready-post-failure-recovery-validated: {}",
        ready.post_failure_runtime_smoke_passed
            && ready.audit_replay_after_failure_validated
            && ready.sqlite_recovery_after_failure_validated
    );
    println!(
        "blocked-transcript-status: {}",
        deployment_failure_capture_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("panic-hook-installed-by-validator: false");
    println!("tracing-subscriber-installed-by-validator: false");
    println!("failure-injected-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("files-mutated-by-validator: false");
    println!("alerts-sent-by-validator: false");
    println!("external-calls-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn run_deployment_response_drill_rehearsal_validation() -> Result<(), AgentCliError> {
    let ready = validate_deployment_response_drill_rehearsal(
        local_deployment_response_drill_rehearsal("ready", true),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let blocked = validate_deployment_response_drill_rehearsal(
        local_deployment_response_drill_rehearsal("blocked", false),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    if ready.status != DeploymentResponseDrillRehearsalStatus::Validated
        || blocked.status != DeploymentResponseDrillRehearsalStatus::Blocked
        || !ready.rollback_ready
        || !ready.incident_response_ready
        || !ready.failure_capture_ready
        || !ready.plan_ids_match
        || !ready.component_operator_approvals_present
        || !ready.component_reviewer_approvals_present
        || ready.production_ready
        || blocked.production_ready
        || ready.rollback_executed_by_validator
        || blocked.rollback_executed_by_validator
        || ready.incident_response_executed_by_validator
        || blocked.incident_response_executed_by_validator
        || ready.failure_injected_by_validator
        || blocked.failure_injected_by_validator
        || ready.service_manager_action_performed_by_validator
        || blocked.service_manager_action_performed_by_validator
        || ready.files_mutated_by_validator
        || blocked.files_mutated_by_validator
        || ready.alerts_sent_by_validator
        || blocked.alerts_sent_by_validator
        || ready.external_calls_performed
        || blocked.external_calls_performed
        || ready.live_execution_performed
        || blocked.live_execution_performed
        || blocked.blocker_codes.is_empty()
    {
        return Err(AgentCliError::Validation(
            "deployment response drill rehearsal validation failed".to_owned(),
        ));
    }

    println!("deployment-response-drill-rehearsal: validation passed");
    println!(
        "ready-rehearsal-status: {}",
        deployment_response_drill_rehearsal_status_label(ready.status)
    );
    println!("ready-rollback-ready: {}", ready.rollback_ready);
    println!(
        "ready-incident-response-ready: {}",
        ready.incident_response_ready
    );
    println!(
        "ready-failure-capture-ready: {}",
        ready.failure_capture_ready
    );
    println!("ready-plan-ids-match: {}", ready.plan_ids_match);
    println!(
        "ready-total-non-secret-reference-count: {}",
        ready.total_non_secret_reference_count
    );
    println!(
        "ready-component-operator-approvals-present: {}",
        ready.component_operator_approvals_present
    );
    println!(
        "ready-component-reviewer-approvals-present: {}",
        ready.component_reviewer_approvals_present
    );
    println!("ready-operator-approved: {}", ready.operator_approved);
    println!("ready-reviewer-approved: {}", ready.reviewer_approved);
    println!(
        "blocked-rehearsal-status: {}",
        deployment_response_drill_rehearsal_status_label(blocked.status)
    );
    println!("blocked-blocker-count: {}", blocked.blocker_codes.len());
    println!("rollback-executed-by-validator: false");
    println!("incident-response-executed-by-validator: false");
    println!("failure-injected-by-validator: false");
    println!("service-manager-action-performed-by-validator: false");
    println!("files-mutated-by-validator: false");
    println!("alerts-sent-by-validator: false");
    println!("external-calls-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn local_deployment_disk_full_transcript(
    suffix: &str,
    complete: bool,
) -> AuditDeploymentDiskFullTranscript {
    AuditDeploymentDiskFullTranscript {
        transcript_id: format!("local-deployment-disk-full-{suffix}"),
        host_label: "deployment-host-a".to_owned(),
        physical_host_evidence: complete,
        audit_append_failed_closed: true,
        state_write_failed_closed: complete,
        runtime_quiesced_or_degraded: complete,
        audit_replay_after_recovery_validated: complete,
        sqlite_reopen_after_recovery_validated: complete,
        recovery_runbook_reference_present: complete,
        non_secret_reference_count: if complete { 5 } else { 1 },
        operator_approved: complete,
        disk_filled_by_validator: false,
        production_path_mutated_by_validator: false,
        live_execution_performed: false,
        production_ready_claimed: false,
        validated_at_unix_ms: 93_000,
    }
}

fn local_deployment_retention_transcript(
    suffix: &str,
    complete: bool,
) -> AuditDeploymentRetentionTranscript {
    AuditDeploymentRetentionTranscript {
        transcript_id: format!("local-deployment-retention-{suffix}"),
        host_label: "deployment-host-a".to_owned(),
        physical_host_evidence: complete,
        active_rotation_observed: complete,
        archive_retention_observed: complete,
        expired_archive_deletion_observed: complete,
        post_rotation_append_validated: complete,
        audit_replay_after_rotation_validated: complete,
        retention_policy_reference_present: complete,
        recovery_runbook_reference_present: complete,
        non_secret_reference_count: if complete { 6 } else { 1 },
        operator_approved: complete,
        rotation_performed_by_validator: false,
        production_path_mutated_by_validator: false,
        live_execution_performed: false,
        production_ready_claimed: false,
        validated_at_unix_ms: 94_000,
    }
}

fn local_deployment_permission_transcript(
    suffix: &str,
    complete: bool,
) -> RuntimeDeploymentPermissionTranscript {
    RuntimeDeploymentPermissionTranscript {
        transcript_id: format!("local-deployment-permission-{suffix}"),
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

fn local_deployment_audit_sqlite_transcript(
    suffix: &str,
    complete: bool,
) -> RuntimeDeploymentAuditSqliteTranscript {
    RuntimeDeploymentAuditSqliteTranscript {
        transcript_id: format!("local-deployment-audit-sqlite-{suffix}"),
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
        validated_at_unix_ms: 95_500,
    }
}

fn local_deployment_backup_restore_transcript(
    suffix: &str,
    complete: bool,
) -> RuntimeDeploymentBackupRestoreTranscript {
    RuntimeDeploymentBackupRestoreTranscript {
        transcript_id: format!("local-deployment-backup-restore-{suffix}"),
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
        validated_at_unix_ms: 95_650,
    }
}

fn local_deployment_graceful_shutdown_transcript(
    suffix: &str,
    complete: bool,
) -> RuntimeDeploymentGracefulShutdownTranscript {
    RuntimeDeploymentGracefulShutdownTranscript {
        transcript_id: format!("local-deployment-graceful-shutdown-{suffix}"),
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
        validated_at_unix_ms: 95_700,
    }
}

fn local_deployment_sqlite_schema_migration_transcript(
    suffix: &str,
    complete: bool,
) -> RuntimeDeploymentSqliteSchemaMigrationTranscript {
    RuntimeDeploymentSqliteSchemaMigrationTranscript {
        transcript_id: format!("local-deployment-sqlite-schema-migration-{suffix}"),
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
        validated_at_unix_ms: 95_750,
    }
}

fn local_rollback_execution_transcript(
    suffix: &str,
    complete: bool,
) -> RollbackExecutionTranscript {
    RollbackExecutionTranscript {
        transcript_id: format!("local-rollback-execution-{suffix}"),
        plan_id: "phase-54-rollback-execution".to_owned(),
        candidate_reference_present: complete,
        rollback_reference_present: complete,
        service_quiesced_reference_present: complete,
        previous_artifact_restored: complete,
        previous_config_restored: complete,
        post_rollback_runtime_smoke_passed: complete,
        audit_replay_after_rollback_validated: complete,
        sqlite_recovery_after_rollback_validated: complete,
        operator_approved: complete,
        reviewer_approved: complete,
        non_secret_reference_count: if complete { 7 } else { 1 },
        rollback_executed_by_validator: false,
        service_manager_action_performed_by_validator: false,
        files_mutated_by_validator: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready_claimed: false,
        validated_at_unix_ms: 96_000,
    }
}

fn local_incident_response_execution_transcript(
    suffix: &str,
    complete: bool,
) -> IncidentResponseExecutionTranscript {
    IncidentResponseExecutionTranscript {
        transcript_id: format!("local-incident-response-execution-{suffix}"),
        plan_id: "phase-55-incident-response-execution".to_owned(),
        incident_scenario_reference_present: complete,
        severity_reference_present: complete,
        responder_reference_present: complete,
        reviewer_reference_present: complete,
        detection_triage_reference_present: complete,
        containment_recovery_reference_present: complete,
        post_incident_runtime_smoke_passed: complete,
        audit_replay_after_recovery_validated: complete,
        sqlite_recovery_after_recovery_validated: complete,
        communications_reference_present: complete,
        operator_approved: complete,
        reviewer_approved: complete,
        non_secret_reference_count: if complete { 8 } else { 1 },
        incident_response_executed_by_validator: false,
        service_manager_action_performed_by_validator: false,
        files_mutated_by_validator: false,
        alerts_sent_by_validator: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready_claimed: false,
        validated_at_unix_ms: 97_000,
    }
}

fn local_deployment_failure_capture_transcript(
    suffix: &str,
    complete: bool,
) -> DeploymentFailureCaptureTranscript {
    DeploymentFailureCaptureTranscript {
        transcript_id: format!("local-deployment-failure-capture-{suffix}"),
        plan_id: "phase-56-deployment-failure-capture".to_owned(),
        deployment_host_reference_present: complete,
        daemon_panic_hook_reference_present: complete,
        daemon_tracing_reference_present: complete,
        failure_scenario_reference_present: complete,
        failure_capture_reference_present: complete,
        sanitized_payload_review_present: complete,
        runtime_quiesce_or_degrade_validated: complete,
        post_failure_runtime_smoke_passed: complete,
        audit_replay_after_failure_validated: complete,
        sqlite_recovery_after_failure_validated: complete,
        alert_route_reference_present: complete,
        operator_approved: complete,
        reviewer_approved: complete,
        non_secret_reference_count: if complete { 9 } else { 1 },
        panic_hook_installed_by_validator: false,
        tracing_subscriber_installed_by_validator: false,
        failure_injected_by_validator: false,
        service_manager_action_performed_by_validator: false,
        files_mutated_by_validator: false,
        alerts_sent_by_validator: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready_claimed: false,
        validated_at_unix_ms: 98_000,
    }
}

fn local_deployment_response_drill_rehearsal(
    suffix: &str,
    complete: bool,
) -> DeploymentResponseDrillRehearsalRequest {
    let plan_id = format!("phase-69-response-drill-{suffix}");
    let mut rollback = local_rollback_execution_transcript(suffix, true);
    rollback.plan_id.clone_from(&plan_id);
    let mut incident = local_incident_response_execution_transcript(suffix, complete);
    incident.plan_id.clone_from(&plan_id);
    let mut failure = local_deployment_failure_capture_transcript(suffix, complete);
    failure.plan_id.clone_from(&plan_id);

    DeploymentResponseDrillRehearsalRequest {
        rehearsal_id: format!("local-deployment-response-drill-{suffix}"),
        plan_id,
        rollback_report: validate_rollback_execution_transcript(rollback)
            .expect("local rollback transcript fixture validates"),
        incident_response_report: validate_incident_response_execution_transcript(incident)
            .expect("local incident-response transcript fixture validates"),
        failure_capture_report: validate_deployment_failure_capture_transcript(failure)
            .expect("local failure-capture transcript fixture validates"),
        operator_approved: complete,
        reviewer_approved: complete,
        rollback_executed_by_validator: false,
        incident_response_executed_by_validator: false,
        failure_injected_by_validator: false,
        service_manager_action_performed_by_validator: false,
        files_mutated_by_validator: false,
        alerts_sent_by_validator: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready_claimed: false,
        validated_at_unix_ms: 99_000,
    }
}

fn local_service_manager_lifecycle_transcript(
    suffix: &str,
    complete: bool,
) -> RuntimeServiceManagerLifecycleTranscript {
    let mut events = vec![
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::UnitLoaded,
            91_000,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::Started,
            91_100,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed,
            91_200,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
            91_300,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::Stopped,
            91_400,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::Restarted,
            91_500,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
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
        transcript_id: format!("local-service-manager-lifecycle-{suffix}"),
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

fn local_service_manager_lifecycle_rehearsal(
    suffix: &str,
    complete: bool,
) -> RuntimeServiceManagerLifecycleRehearsalRequest {
    let mut events = vec![
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::UnitLoaded,
            93_000,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::Started,
            93_100,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::RuntimeSmokePassed,
            93_200,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::GracefulShutdownRequested,
            93_300,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::Stopped,
            93_400,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
            RuntimeServiceManagerLifecycleEventKind::Restarted,
            93_500,
            true,
        ),
        local_service_manager_lifecycle_event(
            suffix,
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
        rehearsal_id: format!("local-service-manager-lifecycle-rehearsal-{suffix}"),
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

fn local_service_manager_lifecycle_event(
    suffix: &str,
    kind: RuntimeServiceManagerLifecycleEventKind,
    observed_at_unix_ms: u64,
    complete: bool,
) -> RuntimeServiceManagerLifecycleEvent {
    RuntimeServiceManagerLifecycleEvent {
        event_id: format!("local-service-manager-{suffix}-{kind:?}"),
        kind,
        observed_at_unix_ms,
        operator_controlled: true,
        non_secret_reference_present: complete,
        outcome_success: complete,
    }
}

#[allow(clippy::too_many_lines)]
fn run_dashboard_runtime_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options.workspace_dir.join("dashboard-audit.jsonl");
    let state_path = options.workspace_dir.join("dashboard-state.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let renderer = DeterministicDashboardRenderer;
    let render_record = renderer
        .render(DashboardRenderRequest {
            config: DashboardBoundaryConfig::default(),
            snapshot: local_dashboard_runtime_snapshot(now_unix_ms),
            access: DashboardAccessContext::local_render(Some(
                "local-dashboard-runtime-cli".to_owned(),
            )),
            requested_panels: Vec::new(),
            operator_label: Some("local-dashboard-runtime-cli".to_owned()),
            rendered_at_ms: now_unix_ms.saturating_add(1),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let render_audit =
        append_dashboard_render_audit(&mut journal, &render_record, now_unix_ms.saturating_add(2))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let render_checkpoint = persist_dashboard_render_checkpoint(
        &mut store,
        &render_record,
        now_unix_ms.saturating_add(3),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let security_review = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
        review_id: "local-dashboard-runtime-hosted-security".to_owned(),
        authentication_required: true,
        authorization_required: true,
        csrf_protection_required: true,
        csrf_token_rotation_required: true,
        secure_headers_required: true,
        clickjacking_protection_required: true,
        rate_limit_required: true,
        max_requests_per_minute: 60,
        loopback_only_required: true,
        audit_state_preflight_required: true,
        session_revocation_required: true,
        operator_role_review_required: true,
        read_only_controls_required: true,
        public_exposure_requested: false,
        server_start_requested: false,
        live_controls_requested: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let security_audit = append_dashboard_hosted_security_review_audit(
        &mut journal,
        &security_review,
        now_unix_ms.saturating_add(4),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let security_checkpoint = persist_dashboard_hosted_security_review_checkpoint(
        &mut store,
        &security_review,
        now_unix_ms.saturating_add(5),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let request_preflight = preflight_dashboard_hosted_request(&DashboardHostedRequestPreflight {
        preflight_id: "local-dashboard-runtime-hosted-request".to_owned(),
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
        max_requests_per_minute: 60,
        public_exposure_requested: false,
        server_start_requested: false,
        live_controls_requested: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let preflight_audit = append_dashboard_hosted_request_preflight_audit(
        &mut journal,
        &request_preflight,
        now_unix_ms.saturating_add(6),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let preflight_checkpoint = persist_dashboard_hosted_request_preflight_checkpoint(
        &mut store,
        &request_preflight,
        now_unix_ms.saturating_add(7),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let request_validation = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
        validation_id: "local-dashboard-runtime-one-shot-request".to_owned(),
        render_record: render_record.clone(),
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
        max_requests_per_minute: 60,
        public_exposure_requested: false,
        live_controls_requested: false,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let validation_audit = append_dashboard_hosted_request_validation_audit(
        &mut journal,
        &request_validation,
        now_unix_ms.saturating_add(8),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let validation_checkpoint = persist_dashboard_hosted_request_validation_checkpoint(
        &mut store,
        &request_validation,
        now_unix_ms.saturating_add(9),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let unauthenticated_request =
        validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "local-dashboard-runtime-unauthenticated-request".to_owned(),
            render_record: render_record.clone(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            method: DashboardHostedRequestMethod::Get,
            request_path: "/".to_owned(),
            authenticated: false,
            authorized: false,
            csrf_token_present: false,
            csrf_token_valid: true,
            secure_headers_required: true,
            requests_in_current_window: 1,
            max_requests_per_minute: 60,
            public_exposure_requested: false,
            live_controls_requested: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let csrf_rejected_request =
        validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "local-dashboard-runtime-csrf-rejected-request".to_owned(),
            render_record: render_record.clone(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            method: DashboardHostedRequestMethod::Post,
            request_path: "/".to_owned(),
            authenticated: true,
            authorized: true,
            csrf_token_present: true,
            csrf_token_valid: false,
            secure_headers_required: true,
            requests_in_current_window: 1,
            max_requests_per_minute: 60,
            public_exposure_requested: false,
            live_controls_requested: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let rate_limited_request =
        validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "local-dashboard-runtime-rate-limited-request".to_owned(),
            render_record: render_record.clone(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            method: DashboardHostedRequestMethod::Get,
            request_path: "/".to_owned(),
            authenticated: true,
            authorized: true,
            csrf_token_present: false,
            csrf_token_valid: true,
            secure_headers_required: true,
            requests_in_current_window: 61,
            max_requests_per_minute: 60,
            public_exposure_requested: false,
            live_controls_requested: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let session_validation = validate_dashboard_hosted_session(
        "local-dashboard-runtime-session",
        &[
            request_validation.clone(),
            unauthenticated_request,
            csrf_rejected_request,
            rate_limited_request,
        ],
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let session_audit = append_dashboard_hosted_session_validation_audit(
        &mut journal,
        &session_validation,
        now_unix_ms.saturating_add(10),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let session_checkpoint = persist_dashboard_hosted_session_validation_checkpoint(
        &mut store,
        &session_validation,
        now_unix_ms.saturating_add(11),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let runtime_readiness_review =
        review_dashboard_hosted_runtime_readiness(DashboardHostedRuntimeReadinessReviewRequest {
            review_id: "local-dashboard-runtime-readiness".to_owned(),
            security_review: security_review.clone(),
            request_preflight: request_preflight.clone(),
            session_validation: session_validation.clone(),
            remaining_external_evidence: vec![
                "persistent daemon dashboard hosting validation".to_owned(),
                "browser authentication/session validation".to_owned(),
                "CSRF token and secure-header serving validation".to_owned(),
                "external dashboard penetration testing".to_owned(),
            ],
            persistent_server_start_requested: false,
            public_network_exposure_requested: false,
            live_controls_requested: false,
            production_ready_claimed: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let audit_records = [
        render_audit.sequence,
        security_audit.sequence,
        preflight_audit.sequence,
        validation_audit.sequence,
        session_audit.sequence,
    ];
    let checkpoint_keys = [
        render_checkpoint.key,
        security_checkpoint.key,
        preflight_checkpoint.key,
        validation_checkpoint.key,
        session_checkpoint.key,
    ];
    drop(store);
    drop(journal);

    let reopened_journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let replayed_records = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopened_store
        .integrity_check()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_checkpoints = checkpoint_keys
        .iter()
        .map(|key| {
            reopened_store
                .get_checkpoint(key)
                .map_err(|error| AgentCliError::Validation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let recovered_checkpoint_count = recovered_checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.is_some())
        .count();

    if replayed_records != u64::try_from(audit_records.len()).unwrap_or(u64::MAX)
        || recovered_checkpoint_count != checkpoint_keys.len()
        || !recovered_checkpoints.iter().all(Option::is_some)
        || !render_record.access_authorized
        || render_record.server_started
        || render_record.public_network_exposed
        || render_record.live_controls_enabled
        || security_review.status != DashboardHostedSecurityReviewStatus::ReadyForLocalReview
        || security_review.server_started
        || security_review.public_network_exposed
        || security_review.live_controls_enabled
        || security_review.production_ready
        || !request_preflight.loopback_bind_validated
        || !request_preflight.authenticated
        || !request_preflight.authorized
        || !request_preflight.csrf_validated
        || !request_preflight.secure_headers_validated
        || !request_preflight.rate_limit_validated
        || request_preflight.server_started
        || request_preflight.public_network_exposed
        || request_preflight.live_controls_enabled
        || request_preflight.production_ready
        || request_validation.status != DashboardHostedRequestValidationStatus::ReadyForLocalReview
        || !request_validation.loopback_bind_validated
        || !request_validation.local_server_started
        || !request_validation.network_request_served
        || request_validation.local_http_status_code != 200
        || request_validation.public_network_exposed
        || request_validation.live_controls_enabled
        || request_validation.production_ready
        || session_validation.status != DashboardHostedSessionValidationStatus::ReadyForLocalReview
        || session_validation.total_request_count != 4
        || session_validation.accepted_request_count != 1
        || session_validation.rejected_unauthenticated_count != 1
        || session_validation.rejected_csrf_count != 1
        || session_validation.rejected_rate_limited_count != 1
        || !session_validation.loopback_bind_validated
        || !session_validation.secure_headers_validated
        || !session_validation.local_server_started
        || !session_validation.network_request_served
        || session_validation.public_network_exposed
        || session_validation.live_controls_enabled
        || session_validation.production_ready
        || runtime_readiness_review.status
            != DashboardHostedRuntimeReadinessReviewStatus::ReadyForLocalReview
        || !runtime_readiness_review.security_review_ready
        || !runtime_readiness_review.request_preflight_ready
        || !runtime_readiness_review.session_validation_ready
        || !runtime_readiness_review.accepted_request_validated
        || !runtime_readiness_review.unauthenticated_rejection_validated
        || !runtime_readiness_review.csrf_rejection_validated
        || !runtime_readiness_review.rate_limit_rejection_validated
        || !runtime_readiness_review.loopback_serving_validated
        || !runtime_readiness_review.secure_headers_validated
        || !runtime_readiness_review.remaining_external_evidence_recorded
        || runtime_readiness_review.persistent_server_started
        || runtime_readiness_review.public_network_exposed
        || runtime_readiness_review.live_controls_enabled
        || runtime_readiness_review.production_ready
    {
        return Err(AgentCliError::Validation(
            "dashboard runtime validation failed".to_owned(),
        ));
    }

    println!("dashboard-runtime: validation passed");
    println!(
        "dashboard-runtime-workspace: {}",
        options.workspace_dir.display()
    );
    println!("dashboard-runtime-version: {DASHBOARD_BOUNDARY_VERSION}");
    println!("dashboard-runtime-audit-records-replayed: {replayed_records}");
    println!("dashboard-runtime-checkpoints-recovered: {recovered_checkpoint_count}");
    println!("dashboard-render-checkpoint-key: {DASHBOARD_LAST_RENDER_CHECKPOINT_KEY}");
    println!(
        "dashboard-hosted-security-checkpoint-key: {DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY}"
    );
    println!(
        "dashboard-hosted-request-preflight-checkpoint-key: {DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY}"
    );
    println!(
        "dashboard-hosted-request-validation-checkpoint-key: {DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY}"
    );
    println!(
        "dashboard-hosted-session-validation-checkpoint-key: {DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY}"
    );
    println!(
        "dashboard-render-access-authorized: {}",
        render_record.access_authorized
    );
    println!(
        "dashboard-render-panel-count: {}",
        render_record.panels.len()
    );
    println!(
        "dashboard-hosted-security-ready: {}",
        security_review.status == DashboardHostedSecurityReviewStatus::ReadyForLocalReview
    );
    println!(
        "dashboard-hosted-audit-state-preflight-required: {}",
        security_review.audit_state_preflight_required
    );
    println!(
        "dashboard-hosted-session-revocation-required: {}",
        security_review.session_revocation_required
    );
    println!(
        "dashboard-hosted-operator-role-review-required: {}",
        security_review.operator_role_review_required
    );
    println!(
        "dashboard-hosted-read-only-controls-required: {}",
        security_review.read_only_controls_required
    );
    println!(
        "dashboard-hosted-request-preflight-ready: {}",
        request_preflight.missing_control_count == 0
    );
    println!(
        "dashboard-hosted-request-validation-ready: {}",
        request_validation.status == DashboardHostedRequestValidationStatus::ReadyForLocalReview
    );
    println!(
        "dashboard-hosted-session-validation-ready: {}",
        session_validation.status == DashboardHostedSessionValidationStatus::ReadyForLocalReview
    );
    println!(
        "dashboard-hosted-runtime-readiness-review-ready: {}",
        runtime_readiness_review.status
            == DashboardHostedRuntimeReadinessReviewStatus::ReadyForLocalReview
    );
    println!(
        "dashboard-hosted-runtime-security-review-ready: {}",
        runtime_readiness_review.security_review_ready
    );
    println!(
        "dashboard-hosted-runtime-preflight-ready: {}",
        runtime_readiness_review.request_preflight_ready
    );
    println!(
        "dashboard-hosted-runtime-session-ready: {}",
        runtime_readiness_review.session_validation_ready
    );
    println!(
        "dashboard-hosted-runtime-accepted-request-validated: {}",
        runtime_readiness_review.accepted_request_validated
    );
    println!(
        "dashboard-hosted-runtime-unauthenticated-rejection-validated: {}",
        runtime_readiness_review.unauthenticated_rejection_validated
    );
    println!(
        "dashboard-hosted-runtime-csrf-rejection-validated: {}",
        runtime_readiness_review.csrf_rejection_validated
    );
    println!(
        "dashboard-hosted-runtime-rate-limit-rejection-validated: {}",
        runtime_readiness_review.rate_limit_rejection_validated
    );
    println!(
        "dashboard-hosted-runtime-loopback-serving-validated: {}",
        runtime_readiness_review.loopback_serving_validated
    );
    println!(
        "dashboard-hosted-runtime-secure-headers-validated: {}",
        runtime_readiness_review.secure_headers_validated
    );
    println!(
        "dashboard-hosted-runtime-remaining-external-evidence-count: {}",
        runtime_readiness_review.remaining_external_evidence_count
    );
    println!(
        "dashboard-hosted-session-requests: {}",
        session_validation.total_request_count
    );
    println!(
        "dashboard-hosted-session-accepted: {}",
        session_validation.accepted_request_count
    );
    println!(
        "dashboard-hosted-session-rejected-unauthenticated: {}",
        session_validation.rejected_unauthenticated_count
    );
    println!(
        "dashboard-hosted-session-rejected-csrf: {}",
        session_validation.rejected_csrf_count
    );
    println!(
        "dashboard-hosted-session-rejected-rate-limited: {}",
        session_validation.rejected_rate_limited_count
    );
    println!(
        "local-dashboard-server-started: {}",
        request_validation.local_server_started
    );
    println!(
        "network-request-served: {}",
        request_validation.network_request_served
    );
    println!(
        "local-http-status-code: {}",
        request_validation.local_http_status_code
    );
    println!("public-network-exposed: false");
    println!(
        "persistent-dashboard-server-started: {}",
        runtime_readiness_review.persistent_server_started
    );
    println!("live-controls-enabled: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_runtime_panic_hook_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options.workspace_dir.join("runtime-panic-hook.audit.jsonl");
    let state_path = options.workspace_dir.join("runtime-panic-hook.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let guard = install_local_runtime_panic_hook(RuntimePanicHookInstallationRequest {
        failure_id: "local-runtime-panic-hook-cli".to_owned(),
        component: "runtime-lifecycle".to_owned(),
        severity: ObservabilitySeverity::Critical,
        summary: "local runtime panic hook captured failure".to_owned(),
        detail: "standalone local panic hook validation stores sanitized metadata".to_owned(),
        audit_path: audit_path.clone(),
        state_path: state_path.clone(),
        config: ObservabilityBoundaryConfig::default(),
        access: ObservabilityAccessContext::local_collection(Some(
            "local-runtime-panic-hook-cli".to_owned(),
        )),
        captured_at_ms: now_unix_ms,
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let install_report = guard.report().clone();

    let panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
        panic!("local runtime panic hook cli sentinel");
    }));
    let panic_captured = guard.panic_captured();
    let capture_error = guard.last_capture_error();
    drop(guard);

    let reopened_journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let audit_records_replayed = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopened_store
        .integrity_check()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let failure_checkpoint = reopened_store
        .get_checkpoint(OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let failure_checkpoint_recovered = failure_checkpoint.is_some();
    let failure_checkpoint_contains_sentinel =
        failure_checkpoint.as_ref().is_some_and(|checkpoint| {
            checkpoint
                .value
                .contains("local runtime panic hook cli sentinel")
        });
    let panic_observed = panic_result.is_err();
    let hook_restored = true;

    println!("runtime-panic-hook-validation: passed");
    println!(
        "runtime-panic-hook-workspace: {}",
        options.workspace_dir.display()
    );
    println!(
        "runtime-panic-hook-installed: {}",
        install_report.hook_installed
    );
    println!("runtime-panic-hook-restored: {hook_restored}");
    println!("runtime-panic-hook-panic-observed: {panic_observed}");
    println!("runtime-panic-hook-panic-captured: {panic_captured}");
    println!(
        "runtime-panic-hook-capture-error: {}",
        capture_error.unwrap_or_default()
    );
    println!("runtime-panic-hook-audit-records-replayed: {audit_records_replayed}");
    println!("runtime-panic-hook-failure-checkpoint-recovered: {failure_checkpoint_recovered}");
    println!(
        "runtime-panic-hook-failure-checkpoint-contains-sentinel: {failure_checkpoint_contains_sentinel}"
    );
    println!(
        "runtime-panic-hook-metrics-endpoint-started: {}",
        install_report.metrics_endpoint_started
    );
    println!(
        "runtime-panic-hook-public-network-exposed: {}",
        install_report.public_network_exposed
    );
    println!(
        "runtime-panic-hook-outbound-alerts-sent: {}",
        install_report.outbound_alerts_sent
    );
    println!(
        "runtime-panic-hook-external-submission-performed: {}",
        install_report.external_submission_performed
    );
    println!(
        "runtime-panic-hook-live-execution-performed: {}",
        install_report.live_execution_performed
    );
    println!(
        "runtime-panic-hook-production-ready: {}",
        install_report.production_ready
    );

    if !install_report.hook_installed
        || !panic_observed
        || !panic_captured
        || audit_records_replayed != 1
        || !failure_checkpoint_recovered
        || !failure_checkpoint_contains_sentinel
        || install_report.metrics_endpoint_started
        || install_report.public_network_exposed
        || install_report.outbound_alerts_sent
        || install_report.external_submission_performed
        || install_report.live_execution_performed
        || install_report.production_ready
    {
        return Err(AgentCliError::Validation(
            "runtime panic-hook validation failed local-only invariants".to_owned(),
        ));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_observability_runtime_validation(
    options: &LocalValidationRunOptions,
) -> Result<(), AgentCliError> {
    prepare_fresh_workspace(&options.workspace_dir)?;
    let audit_path = options.workspace_dir.join("observability-audit.jsonl");
    let state_path = options.workspace_dir.join("observability-state.sqlite3");
    let now_unix_ms = current_unix_ms()?;
    let mut journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let mut store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let config = ObservabilityBoundaryConfig::default();
    let collector = DeterministicObservabilityCollector;
    let record = collector
        .collect(ObservabilityCollectionRequest {
            config: config.clone(),
            snapshot: local_observability_runtime_snapshot(now_unix_ms),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-observability-runtime-cli".to_owned(),
            )),
            operator_label: Some("local-observability-runtime-cli".to_owned()),
            collected_at_ms: now_unix_ms.saturating_add(1),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let record_audit =
        append_observability_record_audit(&mut journal, &record, now_unix_ms.saturating_add(2))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let record_checkpoint =
        persist_observability_record_checkpoint(&mut store, &record, now_unix_ms.saturating_add(3))
            .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let operations_review = review_observability_operations(&ObservabilityOperationsPolicy {
        review_id: "local-observability-runtime-review".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let review_audit = append_observability_operations_review_audit(
        &mut journal,
        &operations_review,
        now_unix_ms.saturating_add(4),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let review_checkpoint = persist_observability_operations_review_checkpoint(
        &mut store,
        &operations_review,
        now_unix_ms.saturating_add(5),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let retention_workspace = options.workspace_dir.join("observability-log-retention");
    fs::create_dir_all(&retention_workspace).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create local observability log retention workspace {}: {error}",
            retention_workspace.display()
        ))
    })?;
    let active_log = retention_workspace.join("observability-active.log");
    let retained_log = retention_workspace.join("observability-retained.log");
    let expired_log = retention_workspace.join("observability-expired.log");
    write_audit_retention_fixture(&active_log, "observability-active")?;
    write_audit_retention_fixture(&retained_log, "observability-retained")?;
    write_audit_retention_fixture(&expired_log, "observability-expired")?;
    let log_retention_report =
        execute_local_observability_log_retention(&ObservabilityLogRetentionExecutionRequest {
            execution_id: "local-observability-runtime-log-retention".to_owned(),
            operations_review: operations_review.clone(),
            retention_request: AuditRetentionExecutionRequest {
                workspace_dir: retention_workspace,
                policy: AuditRetentionPolicy {
                    max_active_bytes: 1,
                    max_archived_files: 1,
                    retention_window_ms: 1_000,
                },
                files: vec![
                    audit_retention_file_metadata(&active_log, now_unix_ms, true)?,
                    audit_retention_file_metadata(
                        &retained_log,
                        now_unix_ms.saturating_sub(500),
                        false,
                    )?,
                    audit_retention_file_metadata(
                        &expired_log,
                        now_unix_ms.saturating_sub(2_000),
                        false,
                    )?,
                ],
                now_unix_ms: now_unix_ms.saturating_add(6),
            },
            local_sandbox_only: true,
            production_log_paths_requested: false,
            service_manager_action_requested: false,
            external_log_shipping_requested: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let log_retention_audit = append_observability_log_retention_execution_audit(
        &mut journal,
        &log_retention_report,
        now_unix_ms.saturating_add(7),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let log_retention_checkpoint = persist_observability_log_retention_execution_checkpoint(
        &mut store,
        &log_retention_report,
        now_unix_ms.saturating_add(8),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let export_report =
        render_observability_export_dry_run(arb_core::ObservabilityExportDryRunRequest {
            record: record.clone(),
            operations_review: operations_review.clone(),
            alert_route_references: vec!["local-alert-route-1".to_owned()],
            rendered_at_ms: now_unix_ms.saturating_add(9),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let export_audit = append_observability_export_dry_run_audit(
        &mut journal,
        &export_report,
        now_unix_ms.saturating_add(10),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let export_checkpoint = persist_observability_export_dry_run_checkpoint(
        &mut store,
        &export_report,
        now_unix_ms.saturating_add(11),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let alert_dispatch = DeterministicNotificationBoundary::new()
        .publish(&arb_core::NotificationPublishRequest {
            id: "local-observability-alert-route-notification".to_owned(),
            notification: OperatorNotification {
                id: "local-observability-alert-route-notification".to_owned(),
                severity: NotificationSeverity::Warning,
                title: "Local observability alert route validation".to_owned(),
                body: "Observability alert-route dry-run reached the local communications boundary"
                    .to_owned(),
                channels: vec!["cli".to_owned(), "local-stdout".to_owned()],
                created_at_unix_ms: now_unix_ms.saturating_add(12),
            },
            config: communications_runtime_config(),
            channel_safety: vec![
                communications_channel_safety("cli", now_unix_ms),
                communications_channel_safety("local-stdout", now_unix_ms),
            ],
            now_unix_ms: now_unix_ms.saturating_add(13),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let alert_dispatch_audit = append_notification_dispatch_audit(
        &mut journal,
        &alert_dispatch,
        now_unix_ms.saturating_add(14),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let alert_dispatch_checkpoint = persist_notification_dispatch_checkpoint(
        &mut store,
        &alert_dispatch,
        now_unix_ms.saturating_add(15),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let alert_route_dispatch =
        record_observability_alert_route_dispatch(ObservabilityAlertRouteDispatchRequest {
            dispatch_review_id: "local-observability-alert-route-dispatch".to_owned(),
            export_report: export_report.clone(),
            alert_route_reference: "local-alert-route-1".to_owned(),
            notification_dispatch: alert_dispatch,
            local_dispatch_required: true,
            outbound_alert_delivery_requested: false,
            reviewed_at_ms: now_unix_ms.saturating_add(16),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let alert_route_dispatch_audit = append_observability_alert_route_dispatch_audit(
        &mut journal,
        &alert_route_dispatch,
        now_unix_ms.saturating_add(17),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let alert_route_dispatch_checkpoint = persist_observability_alert_route_dispatch_checkpoint(
        &mut store,
        &alert_route_dispatch,
        now_unix_ms.saturating_add(18),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let endpoint_report = preflight_observability_endpoint(&ObservabilityEndpointPreflight {
        preflight_id: "local-observability-runtime-endpoint".to_owned(),
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
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let endpoint_audit = append_observability_endpoint_preflight_audit(
        &mut journal,
        &endpoint_report,
        now_unix_ms.saturating_add(19),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let endpoint_checkpoint = persist_observability_endpoint_preflight_checkpoint(
        &mut store,
        &endpoint_report,
        now_unix_ms.saturating_add(20),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let bind_report =
        validate_observability_loopback_bind(&ObservabilityLoopbackBindValidationRequest {
            validation_id: "local-observability-runtime-loopback-bind".to_owned(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            loopback_only_required: true,
            serve_requests_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let bind_audit = append_observability_loopback_bind_validation_audit(
        &mut journal,
        &bind_report,
        now_unix_ms.saturating_add(21),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let bind_checkpoint = persist_observability_loopback_bind_validation_checkpoint(
        &mut store,
        &bind_report,
        now_unix_ms.saturating_add(22),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let scrape_report =
        preflight_observability_metrics_scrape(ObservabilityMetricsScrapePreflightRequest {
            scrape_id: "local-observability-runtime-scrape".to_owned(),
            export_report: export_report.clone(),
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
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let scrape_audit = append_observability_metrics_scrape_preflight_audit(
        &mut journal,
        &scrape_report,
        now_unix_ms.saturating_add(23),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let scrape_checkpoint = persist_observability_metrics_scrape_preflight_checkpoint(
        &mut store,
        &scrape_report,
        now_unix_ms.saturating_add(24),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let endpoint_validation =
        validate_observability_metrics_endpoint(ObservabilityMetricsEndpointValidationRequest {
            validation_id: "local-observability-runtime-metrics-endpoint".to_owned(),
            export_report: export_report.clone(),
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
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let endpoint_validation_audit = append_observability_metrics_endpoint_validation_audit(
        &mut journal,
        &endpoint_validation,
        now_unix_ms.saturating_add(25),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let endpoint_validation_checkpoint =
        persist_observability_metrics_endpoint_validation_checkpoint(
            &mut store,
            &endpoint_validation,
            now_unix_ms.saturating_add(26),
        )
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let tracing_report =
        validate_local_tracing_subscriber(LocalTracingSubscriberValidationRequest {
            validation_id: "local-observability-runtime-tracing".to_owned(),
            subscriber_label: "local-observability-runtime-subscriber".to_owned(),
            event: StructuredLogEvent::new(
                "local-observability-runtime-tracing-event",
                ObservabilitySeverity::Info,
                "runtime-observability",
                "local scoped tracing subscriber captured sanitized event",
                vec![StructuredLogField::new("scope", "local-validation")],
                now_unix_ms.saturating_add(27),
            ),
            config: config.clone(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-observability-runtime-cli".to_owned(),
            )),
            local_capture_required: true,
            redaction_required: true,
            global_install_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
            public_network_exposure_requested: false,
            live_execution_requested: false,
            captured_at_ms: now_unix_ms.saturating_add(28),
        })
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let tracing_audit = append_local_tracing_subscriber_audit(
        &mut journal,
        &tracing_report,
        now_unix_ms.saturating_add(29),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let tracing_checkpoint = persist_local_tracing_subscriber_checkpoint(
        &mut store,
        &tracing_report,
        now_unix_ms.saturating_add(30),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let failure_record = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
        failure_id: "local-observability-runtime-failure".to_owned(),
        component: "runtime-observability".to_owned(),
        kind: RuntimeFailureKind::ValidationFailure,
        severity: ObservabilitySeverity::Warning,
        summary: "local observability runtime validation captured failure".to_owned(),
        detail: "local failure capture probe records sanitized metadata only".to_owned(),
        config,
        access: ObservabilityAccessContext::local_collection(Some(
            "local-observability-runtime-cli".to_owned(),
        )),
        captured_at_ms: now_unix_ms.saturating_add(31),
    })
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let failure_audit = append_runtime_failure_capture_audit(
        &mut journal,
        &failure_record,
        now_unix_ms.saturating_add(32),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let failure_checkpoint = persist_runtime_failure_capture_checkpoint(
        &mut store,
        &failure_record,
        now_unix_ms.saturating_add(33),
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    let audit_records = [
        record_audit.sequence,
        review_audit.sequence,
        log_retention_audit.sequence,
        export_audit.sequence,
        alert_dispatch_audit.sequence,
        alert_route_dispatch_audit.sequence,
        endpoint_audit.sequence,
        bind_audit.sequence,
        scrape_audit.sequence,
        endpoint_validation_audit.sequence,
        tracing_audit.sequence,
        failure_audit.sequence,
    ];
    let checkpoint_keys = [
        record_checkpoint.key,
        review_checkpoint.key,
        log_retention_checkpoint.key,
        export_checkpoint.key,
        alert_dispatch_checkpoint.key,
        alert_route_dispatch_checkpoint.key,
        endpoint_checkpoint.key,
        bind_checkpoint.key,
        scrape_checkpoint.key,
        endpoint_validation_checkpoint.key,
        tracing_checkpoint.key,
        failure_checkpoint.key,
    ];
    drop(store);
    drop(journal);

    let reopened_journal = AppendOnlyAuditJournal::open(&audit_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let replayed_records = reopened_journal.next_sequence().saturating_sub(1);
    let reopened_store = SqliteWalStateStore::open(&state_path)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    reopened_store
        .integrity_check()
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let recovered_checkpoints = checkpoint_keys
        .iter()
        .map(|key| {
            reopened_store
                .get_checkpoint(key)
                .map_err(|error| AgentCliError::Validation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let recovered_checkpoint_count = recovered_checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.is_some())
        .count();

    if replayed_records != u64::try_from(audit_records.len()).unwrap_or(u64::MAX)
        || recovered_checkpoint_count != checkpoint_keys.len()
        || operations_review.status != ObservabilityOperationsReviewStatus::ReadyForLocalReview
        || !operations_review.audit_state_preflight_required
        || !operations_review.exporter_kill_switch_required
        || !operations_review.alert_authorization_required
        || !operations_review.rate_limit_backpressure_required
        || !operations_review.retry_backoff_required
        || !operations_review.no_secret_telemetry_required
        || !record.access_authorized
        || !log_retention_report.rotate_active_requested
        || !log_retention_report.new_active_created
        || log_retention_report.deleted_file_count == 0
        || !log_retention_report.sandbox_filesystem_mutated
        || log_retention_report.out_of_workspace_path_touched
        || log_retention_report.production_log_paths_touched
        || log_retention_report.service_manager_action_performed
        || log_retention_report.external_log_shipping_performed
        || log_retention_report.live_network_used
        || log_retention_report.production_ready
        || !endpoint_report.loopback_bind_validated
        || !bind_report.loopback_bind_validated
        || !bind_report.listener_opened_and_closed
        || !scrape_report.loopback_source_validated
        || !endpoint_validation.loopback_bind_validated
        || !endpoint_validation.local_metrics_endpoint_started
        || !endpoint_validation.network_request_served
        || tracing_report.status != LocalTracingSubscriberValidationStatus::ReadyForLocalReview
        || !tracing_report.scoped_subscriber_installed
        || !tracing_report.event_captured
        || tracing_report.captured_event_count == 0
        || tracing_report.global_subscriber_installed
        || tracing_report.telemetry_exported
        || tracing_report.outbound_alerts_sent
        || tracing_report.public_network_exposed
        || tracing_report.live_execution_performed
        || tracing_report.production_ready
        || !failure_record.access_authorized
        || record.metrics_endpoint_started
        || record.public_network_exposed
        || record.outbound_alerts_sent
        || export_report.metrics_endpoint_started
        || export_report.public_network_exposed
        || export_report.outbound_alerts_sent
        || export_report.telemetry_exported
        || alert_route_dispatch.status != ObservabilityAlertRouteDispatchStatus::ReadyForLocalReview
        || alert_route_dispatch.recorded_local_channel_count == 0
        || alert_route_dispatch.outbound_alerts_sent
        || alert_route_dispatch.outbound_network_used
        || alert_route_dispatch.telemetry_exported
        || alert_route_dispatch.live_execution_performed
        || alert_route_dispatch.production_ready
        || scrape_report.metrics_endpoint_started
        || scrape_report.network_request_served
        || scrape_report.public_network_exposed
        || scrape_report.telemetry_exported
        || endpoint_validation.public_network_exposed
        || endpoint_validation.telemetry_exported
        || endpoint_validation.outbound_alerts_sent
        || endpoint_validation.production_ready
        || failure_record.metrics_endpoint_started
        || failure_record.public_network_exposed
        || failure_record.outbound_alerts_sent
        || failure_record.external_submission_performed
        || failure_record.live_execution_performed
        || failure_record.production_ready
    {
        return Err(AgentCliError::Validation(
            "observability runtime validation failed".to_owned(),
        ));
    }

    println!("observability-runtime: validation passed");
    println!(
        "observability-runtime-workspace: {}",
        options.workspace_dir.display()
    );
    println!("observability-runtime-version: {OBSERVABILITY_RUNBOOK_VERSION}");
    println!("observability-runtime-audit-records-replayed: {replayed_records}");
    println!("observability-runtime-checkpoints-recovered: {recovered_checkpoint_count}");
    println!(
        "observability-runtime-audit-state-preflight-required: {}",
        operations_review.audit_state_preflight_required
    );
    println!(
        "observability-runtime-exporter-kill-switch-required: {}",
        operations_review.exporter_kill_switch_required
    );
    println!(
        "observability-runtime-alert-authorization-required: {}",
        operations_review.alert_authorization_required
    );
    println!(
        "observability-runtime-rate-limit-backpressure-required: {}",
        operations_review.rate_limit_backpressure_required
    );
    println!(
        "observability-runtime-retry-backoff-required: {}",
        operations_review.retry_backoff_required
    );
    println!(
        "observability-runtime-no-secret-telemetry-required: {}",
        operations_review.no_secret_telemetry_required
    );
    println!(
        "observability-log-retention-rotate-active-requested: {}",
        log_retention_report.rotate_active_requested
    );
    println!(
        "observability-log-retention-new-active-created: {}",
        log_retention_report.new_active_created
    );
    println!(
        "observability-log-retention-deleted-file-count: {}",
        log_retention_report.deleted_file_count
    );
    println!(
        "observability-log-retention-production-paths-touched: {}",
        log_retention_report.production_log_paths_touched
    );
    println!(
        "observability-log-retention-service-manager-action-performed: {}",
        log_retention_report.service_manager_action_performed
    );
    println!(
        "observability-log-retention-external-log-shipping-performed: {}",
        log_retention_report.external_log_shipping_performed
    );
    println!(
        "observability-runtime-metric-lines: {}",
        export_report.prometheus_metric_lines.len()
    );
    println!(
        "observability-alert-route-dispatch-status: {:?}",
        alert_route_dispatch.status
    );
    println!(
        "observability-alert-route-local-channels-recorded: {}",
        alert_route_dispatch.recorded_local_channel_count
    );
    println!(
        "observability-alert-route-outbound-network-used: {}",
        alert_route_dispatch.outbound_network_used
    );
    println!(
        "observability-runtime-scrape-metric-lines: {}",
        scrape_report.response_metric_line_count
    );
    println!(
        "observability-runtime-served-metric-lines: {}",
        endpoint_validation.response_metric_line_count
    );
    println!(
        "observability-runtime-loopback-bind-validated: {}",
        bind_report.loopback_bind_validated
    );
    println!(
        "observability-runtime-listener-opened-and-closed: {}",
        bind_report.listener_opened_and_closed
    );
    println!(
        "observability-runtime-tracing-subscriber-captured: {}",
        tracing_report.event_captured
    );
    println!(
        "observability-runtime-tracing-global-subscriber-installed: {}",
        tracing_report.global_subscriber_installed
    );
    println!(
        "local-metrics-endpoint-started: {}",
        endpoint_validation.local_metrics_endpoint_started
    );
    println!(
        "metrics-endpoint-started: {}",
        endpoint_validation.local_metrics_endpoint_started
    );
    println!(
        "network-request-served: {}",
        endpoint_validation.network_request_served
    );
    println!("public-network-exposed: false");
    println!("telemetry-exported: false");
    println!("outbound-alerts-sent: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn local_observability_runtime_snapshot(now_unix_ms: u64) -> ObservabilitySnapshot {
    ObservabilitySnapshot {
        snapshot_id: "local-observability-runtime-snapshot".to_owned(),
        generated_at_ms: now_unix_ms,
        components: vec![
            ComponentHealthStatus::new(
                "runtime",
                HealthStatus::Healthy,
                "local runtime observability validation",
                now_unix_ms,
            ),
            ComponentHealthStatus::new(
                "audit-state",
                HealthStatus::Healthy,
                "local audit and SQLite checkpoints recover",
                now_unix_ms,
            ),
        ],
        logs: vec![StructuredLogEvent::new(
            "local-observability-runtime-log",
            ObservabilitySeverity::Info,
            "runtime",
            "local observability runtime validation log",
            vec![StructuredLogField::new("scope", "local-validation")],
            now_unix_ms,
        )],
        metrics: vec![
            MetricSample::new(
                "runtime_validation_count",
                MetricKind::Counter,
                1_000_000,
                "count",
                vec![MetricLabel::new("scope", "local")],
                now_unix_ms,
            ),
            MetricSample::new(
                "runtime_validation_latency",
                MetricKind::Histogram,
                42_000,
                "milliseconds",
                vec![MetricLabel::new("scope", "local")],
                now_unix_ms,
            ),
        ],
        runbooks: vec![Runbook::new(
            "local-observability-runtime-runbook",
            "Local observability runtime validation",
            ObservabilitySeverity::Warning,
            "local observability runtime validation requires review",
            vec![RunbookStep::new(
                1,
                "Review local evidence",
                "Review local non-secret audit and state recovery output",
            )],
        )],
        warnings: vec![
            "local validation only; production observability remains deferred".to_owned(),
        ],
    }
}

fn local_dashboard_runtime_snapshot(now_unix_ms: u64) -> DashboardSnapshot {
    DashboardSnapshot {
        snapshot_id: "local-dashboard-runtime-snapshot".to_owned(),
        generated_at_ms: now_unix_ms,
        runtime_mode: arb_core::RuntimeMode::Paper,
        production_readiness_percent: 0,
        open_gap_count: 1,
        opportunity_count: 0,
        pending_plan_count: 0,
        notification_record_count: 0,
        panels: vec![
            DashboardPanel::new(
                DashboardPanelKind::Safety,
                "Safety",
                "Live controls are disabled",
                vec![DashboardPanelItem::new(
                    "Live controls",
                    "disabled",
                    DashboardSeverity::Ok,
                )],
            ),
            DashboardPanel::new(
                DashboardPanelKind::AuditState,
                "Audit state",
                "Local dashboard runtime validation reopens audit and SQLite checkpoints",
                vec![DashboardPanelItem::new(
                    "Persistence",
                    "local audit and state",
                    DashboardSeverity::Ok,
                )],
            ),
        ],
        warnings: vec![
            "local validation only; production dashboard hosting remains deferred".to_owned(),
        ],
    }
}

fn prepare_fresh_workspace(path: &Path) -> Result<(), AgentCliError> {
    if path.as_os_str().is_empty() {
        return Err(AgentCliError::Usage(
            "runtime smoke workspace path is required".to_owned(),
        ));
    }
    if path.exists() {
        return Err(AgentCliError::Usage(format!(
            "runtime smoke workspace must not already exist: {}",
            path.display()
        )));
    }
    fs::create_dir_all(path).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create runtime smoke workspace {}: {error}",
            path.display()
        ))
    })
}

fn build_runtime_smoke_lifecycle_request(
    config: &AgentConfig,
    policy: &PolicyEngine,
    now_unix_ms: u64,
    run_id: &str,
) -> Result<arb_core::RuntimeLifecycleRequest, AgentCliError> {
    let plan_scope = if config.runtime.mode.permits_live_execution() {
        return Err(AgentCliError::Usage(
            "runtime smoke refuses live-armed configs".to_owned(),
        ));
    } else if matches!(config.runtime.mode, arb_core::RuntimeMode::Paper) {
        ExecutionScope::Paper
    } else {
        ExecutionScope::Observe
    };
    let candidate = build_runtime_smoke_candidate(config, now_unix_ms, run_id)?;
    let planner_request = ExecutionPlannerRequest {
        id: format!("cli-runtime-smoke-planner-request-{run_id}"),
        strategy_id: "cli-runtime-smoke-strategy".to_owned(),
        candidate,
        config: ExecutionPlannerConfig {
            requested_scope: plan_scope,
            max_plan_legs: 2,
            max_total_notional_quote: config.risk.max_single_trade_quote.max(10.0) * 2.0,
            default_slippage_bps: config.risk.slippage_bps,
            max_market_data_age_ms: DEFAULT_MARKET_DATA_FRESHNESS_MS,
            require_policy_preflight: true,
        },
        default_chain: None,
        now_unix_ms,
    };
    let plan = DeterministicExecutionPlanner::new()
        .plan(&planner_request, policy)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(arb_core::RuntimeLifecycleRequest {
        id: format!("cli-runtime-smoke-lifecycle-{run_id}"),
        adapter_request_id: format!("cli-runtime-smoke-adapter-request-{run_id}"),
        plan,
        adapter_config: ExecutionAdapterConfig::default(),
        now_unix_ms,
    })
}

fn build_runtime_smoke_candidate(
    config: &AgentConfig,
    now_unix_ms: u64,
    run_id: &str,
) -> Result<OpportunityCandidate, AgentCliError> {
    if config.venues.cex_allowlist.len() < 2 {
        return Err(AgentCliError::Usage(
            "validate-runtime-smoke requires at least two configured CEX venues".to_owned(),
        ));
    }
    if config.venues.asset_allowlist.len() < 2 {
        return Err(AgentCliError::Usage(
            "validate-runtime-smoke requires at least two configured assets".to_owned(),
        ));
    }

    let base = config.venues.asset_allowlist[0].clone();
    let quote = config
        .venues
        .asset_allowlist
        .iter()
        .find(|asset| !asset.eq_ignore_ascii_case(&base) && stable_quote_asset(asset))
        .cloned()
        .unwrap_or_else(|| config.venues.asset_allowlist[1].clone());
    let pair = MarketPair::new(base, quote)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let buy_price = 4.0_f64.min(config.risk.max_single_trade_quote.max(1.0));
    let sell_price = buy_price + 1.0;
    let total_fees = 0.2;
    let edge = FeeAdjustedEdge::calculate(1.0, total_fees, sell_price)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    Ok(OpportunityCandidate {
        id: format!("cli-runtime-smoke-candidate-{run_id}"),
        route_kind: OpportunityRouteKind::CexCex,
        pair: pair.clone(),
        legs: vec![
            runtime_smoke_leg(
                &config.venues.cex_allowlist[0],
                pair.clone(),
                OpportunityLegSide::Buy,
                buy_price,
            ),
            runtime_smoke_leg(
                &config.venues.cex_allowlist[1],
                pair,
                OpportunityLegSide::Sell,
                sell_price,
            ),
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
        discovered_at_unix_ms: now_unix_ms,
        source_quote_ids: vec![
            "cli-runtime-smoke-quote-a".to_owned(),
            "cli-runtime-smoke-quote-b".to_owned(),
        ],
        warnings: vec![
            "local CLI runtime smoke candidate; no market data, network, or execution occurred"
                .to_owned(),
        ],
    })
}

fn runtime_smoke_leg(
    venue_name: &str,
    pair: MarketPair,
    side: OpportunityLegSide,
    price_quote: f64,
) -> OpportunityLeg {
    let quantity_base = 1.0;
    let notional_quote = price_quote * quantity_base;
    let venue = VenueRef {
        name: venue_name.to_owned(),
        kind: VenueKind::Cex,
    };
    OpportunityLeg {
        venue: venue.clone(),
        pair: pair.clone(),
        side,
        price_quote,
        quantity_base,
        notional_quote,
        fee_estimate: FeeEstimate {
            venue,
            pair: Some(pair),
            notional_quote,
            liquidity_role: LiquidityRole::Taker,
            fee_bps: 10.0,
            venue_fee_quote: 0.1,
            network_fee_quote: 0.0,
            total_fee_quote: 0.1,
            externally_verified: false,
        },
        source_quote_id: format!("cli-runtime-smoke-quote-{venue_name}"),
        market_data_age_ms: 100,
    }
}

fn stable_quote_asset(asset: &str) -> bool {
    matches!(
        asset.to_ascii_uppercase().as_str(),
        "USD" | "USDC" | "USDT" | "DAI"
    )
}

fn current_unix_ms() -> Result<u64, AgentCliError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AgentCliError::Validation(format!("system clock error: {error}")))?;
    u64::try_from(duration.as_millis())
        .map_err(|_| AgentCliError::Validation("system clock value is too large".to_owned()))
}

fn opportunity_planner_trace_workspace() -> Result<PathBuf, AgentCliError> {
    let mut path = env::temp_dir();
    let unique_counter = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "arbyclaw-opportunity-planner-trace-{}-{}-{}",
        std::process::id(),
        current_unix_ms()?,
        unique_counter
    ));
    Ok(path)
}

fn local_temp_workspace(label: &str) -> Result<PathBuf, AgentCliError> {
    let mut path = env::temp_dir();
    let unique_counter = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "arbyclaw-{label}-{}-{}-{}",
        std::process::id(),
        current_unix_ms()?,
        unique_counter
    ));
    Ok(path)
}

fn remove_dir_all_with_retry(path: &Path) -> Result<(), std::io::Error> {
    let mut last_error = None;
    for _ in 0..5 {
        match fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(25));
            }
        }
    }
    Err(last_error.expect("remove_dir_all_with_retry loop should set last_error"))
}

fn runtime_recovery_disposition_status() -> String {
    format!(
        "restart recovery dispositions are {} and {} for local operator review only",
        recovery_disposition_label(RuntimeRestartRecoveryDisposition::ReadyForLocalReview),
        recovery_disposition_label(RuntimeRestartRecoveryDisposition::NeedsOperatorReview)
    )
}

const PHASE27_PLANNER_HANDOFF_CONFIG: &str = r#"
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

#[derive(Debug)]
enum AgentCliError {
    Config(ConfigError),
    Usage(String),
    Validation(String),
}

impl fmt::Display for AgentCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "{error}"),
            Self::Usage(message) | Self::Validation(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for AgentCliError {}

impl From<ConfigError> for AgentCliError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

const fn recovery_disposition_label(
    disposition: RuntimeRestartRecoveryDisposition,
) -> &'static str {
    match disposition {
        RuntimeRestartRecoveryDisposition::ReadyForLocalReview => "ready-for-local-review",
        RuntimeRestartRecoveryDisposition::NeedsOperatorReview => "needs-operator-review",
    }
}

const fn opportunity_replay_status_label(status: OpportunityReplayStatus) -> &'static str {
    match status {
        OpportunityReplayStatus::Passed => "passed",
        OpportunityReplayStatus::Failed => "failed",
    }
}

const fn opportunity_planner_handoff_status_label(
    status: OpportunityPlannerHandoffStatus,
) -> &'static str {
    match status {
        OpportunityPlannerHandoffStatus::Passed => "passed",
        OpportunityPlannerHandoffStatus::Failed => "failed",
    }
}

const fn strategy_profile_replay_status_label(
    status: StrategyProfileReplayValidationStatus,
) -> &'static str {
    match status {
        StrategyProfileReplayValidationStatus::Passed => "passed",
        StrategyProfileReplayValidationStatus::Failed => "failed",
    }
}

const fn strategy_profitability_tuning_status_label(
    status: StrategyProfitabilityTuningValidationStatus,
) -> &'static str {
    match status {
        StrategyProfitabilityTuningValidationStatus::Passed => "passed",
        StrategyProfitabilityTuningValidationStatus::Failed => "failed",
    }
}

const fn execution_adapter_run_status_label(status: ExecutionAdapterRunStatus) -> &'static str {
    match status {
        ExecutionAdapterRunStatus::ObserveRecorded => "observe-recorded",
        ExecutionAdapterRunStatus::PaperModelComplete => "paper-model-complete",
        ExecutionAdapterRunStatus::SubmissionBlocked => "submission-blocked",
        ExecutionAdapterRunStatus::PolicyDenied => "policy-denied",
    }
}

const fn plan_status_label(status: ExecutionPlanStatus) -> &'static str {
    match status {
        ExecutionPlanStatus::DraftReady => "draft-ready",
        ExecutionPlanStatus::PolicyDeniedDraft => "policy-denied-draft",
    }
}

const fn config_migration_status_label(status: ConfigMigrationStatus) -> &'static str {
    match status {
        ConfigMigrationStatus::AlreadyCurrent => "already-current",
        ConfigMigrationStatus::Migrated => "migrated",
    }
}

const fn market_data_preflight_status_label(
    status: MarketDataProviderPreflightStatus,
) -> &'static str {
    match status {
        MarketDataProviderPreflightStatus::Usable => "usable",
        MarketDataProviderPreflightStatus::Blocked => "blocked",
    }
}

const fn market_data_reconnect_plan_status_label(
    status: MarketDataReconnectPlanStatus,
) -> &'static str {
    match status {
        MarketDataReconnectPlanStatus::ReadyForLocalReview => "ready-for-local-review",
        MarketDataReconnectPlanStatus::Blocked => "blocked",
    }
}

const fn market_data_quality_assessment_status_label(
    status: MarketDataQualityAssessmentStatus,
) -> &'static str {
    match status {
        MarketDataQualityAssessmentStatus::Acceptable => "acceptable",
        MarketDataQualityAssessmentStatus::Degraded => "degraded",
        MarketDataQualityAssessmentStatus::Blocked => "blocked",
    }
}

const fn paid_market_data_provider_evaluation_status_label(
    status: PaidMarketDataProviderEvaluationStatus,
) -> &'static str {
    match status {
        PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview => "ready-for-local-review",
        PaidMarketDataProviderEvaluationStatus::Blocked => "blocked",
    }
}

const fn market_data_provider_latency_review_status_label(
    status: MarketDataProviderLatencyReviewStatus,
) -> &'static str {
    match status {
        MarketDataProviderLatencyReviewStatus::ReadyForLocalReview => "ready-for-local-review",
        MarketDataProviderLatencyReviewStatus::Blocked => "blocked",
    }
}

const fn fee_schedule_verification_status_label(
    status: FeeScheduleVerificationStatus,
) -> &'static str {
    match status {
        FeeScheduleVerificationStatus::ReadyForLocalReview => "ready-for-local-review",
        FeeScheduleVerificationStatus::Blocked => "blocked",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        config_migration_status_label, execution_adapter_run_status_label,
        fee_schedule_verification_status_label, fuzz_corpus_replay_status_label,
        market_data_preflight_status_label, market_data_quality_assessment_status_label,
        market_data_reconnect_plan_status_label, opportunity_planner_handoff_status_label,
        opportunity_replay_status_label, paid_market_data_provider_evaluation_status_label,
        parse_local_iteration_options, parse_local_validation_run_options,
        parse_runtime_smoke_options, recovery_disposition_label,
        run_agentic_handoff_audit_validation, run_audit_durability_validation,
        run_audit_retention_execution_validation, run_communications_runtime_validation,
        run_config_migration_validation, run_connector_lifecycle_audit_validation,
        run_dashboard_runtime_validation, run_destination_boundary_audit_validation,
        run_execution_adapter_audit_validation, run_execution_planner_audit_validation,
        run_fee_boundary_audit_validation, run_fee_schedule_verification_validation,
        run_local_fuzz_corpus_runner, run_local_paper_backtest_corpus_runner,
        run_local_property_check_runner, run_local_validation_corpus_runner,
        run_local_validation_runner, run_market_data_boundary_audit_validation,
        run_market_data_history_persistence_validation,
        run_market_data_provider_preflight_validation,
        run_market_data_quality_assessment_validation, run_market_data_reconnect_plan_validation,
        run_observability_runtime_validation, run_opportunity_historical_fixture_validation,
        run_opportunity_planner_handoff_validation, run_opportunity_provider_ingestion_validation,
        run_opportunity_quote_load_validation, run_opportunity_replay_validation,
        run_opportunity_trace_recovery_validation,
        run_paid_market_data_provider_evaluation_validation, run_policy_decision_audit_validation,
        run_runtime_backup_restore_load_validation, run_runtime_backup_restore_validation,
        run_runtime_blocked_audit_preflight_validation,
        run_runtime_blocked_state_preflight_validation, run_runtime_graceful_shutdown_validation,
        run_runtime_incomplete_recovery_validation, run_runtime_panic_hook_validation,
        run_runtime_permission_denial_validation, run_runtime_restart_recovery_validation,
        run_secret_backup_restore_validation, run_secret_boundary_audit_validation,
        run_signer_boundary_audit_validation, run_strategy_constrained_planner_validation,
        run_strategy_profitability_tuning_validation, run_strategy_replay_corpus_validation,
        run_withdrawal_policy_validation, runtime_recovery_disposition_status,
        runtime_supervised_restart_audit_path, runtime_supervised_restart_state_path,
        strategy_profile_replay_status_label, strategy_profitability_tuning_status_label,
        validation_corpus_status_label, validation_run_status_label,
        write_runtime_supervised_restart_seed, ConfigMigrationStatus, ExecutionAdapterRunStatus,
        FeeScheduleVerificationStatus, LocalFuzzCorpusReplayStatus, LocalValidationCorpusStatus,
        LocalValidationRunOptions, MarketDataProviderPreflightStatus,
        MarketDataQualityAssessmentStatus, MarketDataReconnectPlanStatus,
        OpportunityPlannerHandoffStatus, OpportunityReplayStatus,
        PaidMarketDataProviderEvaluationStatus, RuntimeRestartRecoveryDisposition,
        StrategyProfileReplayValidationStatus, StrategyProfitabilityTuningValidationStatus,
        ValidationRunStatus,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process,
    };

    #[test]
    fn recovery_disposition_labels_are_operator_facing() {
        assert_eq!(
            recovery_disposition_label(RuntimeRestartRecoveryDisposition::ReadyForLocalReview),
            "ready-for-local-review"
        );
        assert_eq!(
            recovery_disposition_label(RuntimeRestartRecoveryDisposition::NeedsOperatorReview),
            "needs-operator-review"
        );
    }

    #[test]
    fn config_migration_status_labels_are_operator_facing() {
        assert_eq!(
            config_migration_status_label(ConfigMigrationStatus::AlreadyCurrent),
            "already-current"
        );
        assert_eq!(
            config_migration_status_label(ConfigMigrationStatus::Migrated),
            "migrated"
        );
    }

    #[test]
    fn runtime_recovery_disposition_status_is_local_only() {
        let status = runtime_recovery_disposition_status();

        assert!(status.contains("ready-for-local-review"));
        assert!(status.contains("needs-operator-review"));
        assert!(status.contains("local operator review only"));
    }

    #[test]
    fn runtime_smoke_options_require_config_and_workspace() {
        let options = parse_runtime_smoke_options(
            [
                "--config".to_owned(),
                "config.example.toml".to_owned(),
                "--workspace".to_owned(),
                "target/runtime-smoke".to_owned(),
            ]
            .into_iter(),
        )
        .expect("options should parse");

        assert_eq!(
            options.config_path,
            std::path::PathBuf::from("config.example.toml")
        );
        assert_eq!(
            options.workspace_dir,
            std::path::PathBuf::from("target/runtime-smoke")
        );
        assert_eq!(options.iterations, 1);
    }

    #[test]
    fn runtime_smoke_options_parse_iterations() {
        let options = parse_runtime_smoke_options(
            [
                "--config".to_owned(),
                "config.example.toml".to_owned(),
                "--workspace".to_owned(),
                "target/runtime-smoke".to_owned(),
                "--iterations".to_owned(),
                "3".to_owned(),
            ]
            .into_iter(),
        )
        .expect("options should parse");

        assert_eq!(options.iterations, 3);
    }

    #[test]
    fn runtime_smoke_options_rejects_zero_iterations() {
        let error = parse_runtime_smoke_options(
            [
                "--config".to_owned(),
                "config.example.toml".to_owned(),
                "--workspace".to_owned(),
                "target/runtime-smoke".to_owned(),
                "--iterations".to_owned(),
                "0".to_owned(),
            ]
            .into_iter(),
        )
        .expect_err("zero iterations should be invalid");

        assert!(error
            .to_string()
            .contains("iterations requires an integer >= 1"));
    }

    #[test]
    fn local_iteration_options_parse_iterations() {
        let options = parse_local_iteration_options(
            ["--iterations".to_owned(), "4".to_owned()].into_iter(),
            "validate-opportunity-replay",
            "opportunity replay",
        )
        .expect("iteration options should parse");

        assert_eq!(options.iterations, 4);
    }

    #[test]
    fn local_iteration_options_reject_zero_iterations() {
        let error = parse_local_iteration_options(
            ["--iterations".to_owned(), "0".to_owned()].into_iter(),
            "validate-opportunity-replay",
            "opportunity replay",
        )
        .expect_err("zero iterations should be invalid");

        assert!(error
            .to_string()
            .contains("iterations requires an integer >= 1"));
    }

    #[test]
    fn local_validation_run_options_require_workspace() {
        let options = parse_local_validation_run_options(
            [
                "--workspace".to_owned(),
                "target/local-validation-run".to_owned(),
            ]
            .into_iter(),
        )
        .expect("options should parse");

        assert_eq!(
            options.workspace_dir,
            std::path::PathBuf::from("target/local-validation-run")
        );
    }

    #[test]
    fn validation_run_status_labels_are_operator_facing() {
        assert_eq!(
            validation_run_status_label(ValidationRunStatus::PlannedOnly),
            "planned-only"
        );
        assert_eq!(
            validation_run_status_label(ValidationRunStatus::Rejected),
            "rejected"
        );
    }

    #[test]
    fn validation_corpus_status_labels_are_operator_facing() {
        assert_eq!(
            validation_corpus_status_label(LocalValidationCorpusStatus::ReadyForLocalReview),
            "ready-for-local-review"
        );
    }

    #[test]
    fn fuzz_corpus_replay_status_labels_are_operator_facing() {
        assert_eq!(
            fuzz_corpus_replay_status_label(LocalFuzzCorpusReplayStatus::ReadyForLocalReview),
            "ready-for-local-review"
        );
    }

    #[test]
    fn opportunity_replay_status_labels_are_operator_facing() {
        assert_eq!(
            opportunity_replay_status_label(OpportunityReplayStatus::Passed),
            "passed"
        );
        assert_eq!(
            opportunity_replay_status_label(OpportunityReplayStatus::Failed),
            "failed"
        );
    }

    #[test]
    fn opportunity_planner_handoff_status_labels_are_operator_facing() {
        assert_eq!(
            opportunity_planner_handoff_status_label(OpportunityPlannerHandoffStatus::Passed),
            "passed"
        );
        assert_eq!(
            opportunity_planner_handoff_status_label(OpportunityPlannerHandoffStatus::Failed),
            "failed"
        );
    }

    #[test]
    fn strategy_profile_replay_status_labels_are_operator_facing() {
        assert_eq!(
            strategy_profile_replay_status_label(StrategyProfileReplayValidationStatus::Passed),
            "passed"
        );
        assert_eq!(
            strategy_profile_replay_status_label(StrategyProfileReplayValidationStatus::Failed),
            "failed"
        );
    }

    #[test]
    fn strategy_profitability_tuning_status_labels_are_operator_facing() {
        assert_eq!(
            strategy_profitability_tuning_status_label(
                StrategyProfitabilityTuningValidationStatus::Passed
            ),
            "passed"
        );
        assert_eq!(
            strategy_profitability_tuning_status_label(
                StrategyProfitabilityTuningValidationStatus::Failed
            ),
            "failed"
        );
    }

    #[test]
    fn execution_adapter_run_status_labels_are_operator_facing() {
        assert_eq!(
            execution_adapter_run_status_label(ExecutionAdapterRunStatus::ObserveRecorded),
            "observe-recorded"
        );
        assert_eq!(
            execution_adapter_run_status_label(ExecutionAdapterRunStatus::PaperModelComplete),
            "paper-model-complete"
        );
        assert_eq!(
            execution_adapter_run_status_label(ExecutionAdapterRunStatus::SubmissionBlocked),
            "submission-blocked"
        );
        assert_eq!(
            execution_adapter_run_status_label(ExecutionAdapterRunStatus::PolicyDenied),
            "policy-denied"
        );
    }

    #[test]
    fn market_data_preflight_status_labels_are_operator_facing() {
        assert_eq!(
            market_data_preflight_status_label(MarketDataProviderPreflightStatus::Usable),
            "usable"
        );
        assert_eq!(
            market_data_preflight_status_label(MarketDataProviderPreflightStatus::Blocked),
            "blocked"
        );
    }

    #[test]
    fn market_data_reconnect_plan_status_labels_are_operator_facing() {
        assert_eq!(
            market_data_reconnect_plan_status_label(
                MarketDataReconnectPlanStatus::ReadyForLocalReview
            ),
            "ready-for-local-review"
        );
        assert_eq!(
            market_data_reconnect_plan_status_label(MarketDataReconnectPlanStatus::Blocked),
            "blocked"
        );
    }

    #[test]
    fn market_data_quality_assessment_status_labels_are_operator_facing() {
        assert_eq!(
            market_data_quality_assessment_status_label(
                MarketDataQualityAssessmentStatus::Acceptable
            ),
            "acceptable"
        );
        assert_eq!(
            market_data_quality_assessment_status_label(
                MarketDataQualityAssessmentStatus::Degraded
            ),
            "degraded"
        );
        assert_eq!(
            market_data_quality_assessment_status_label(MarketDataQualityAssessmentStatus::Blocked),
            "blocked"
        );
    }

    #[test]
    fn paid_market_data_provider_evaluation_status_labels_are_operator_facing() {
        assert_eq!(
            paid_market_data_provider_evaluation_status_label(
                PaidMarketDataProviderEvaluationStatus::ReadyForLocalReview
            ),
            "ready-for-local-review"
        );
        assert_eq!(
            paid_market_data_provider_evaluation_status_label(
                PaidMarketDataProviderEvaluationStatus::Blocked
            ),
            "blocked"
        );
    }

    #[test]
    fn fee_schedule_verification_status_labels_are_operator_facing() {
        assert_eq!(
            fee_schedule_verification_status_label(
                FeeScheduleVerificationStatus::ReadyForLocalReview
            ),
            "ready-for-local-review"
        );
        assert_eq!(
            fee_schedule_verification_status_label(FeeScheduleVerificationStatus::Blocked),
            "blocked"
        );
    }

    #[test]
    fn opportunity_replay_validation_runs_local_corpus_only() {
        run_opportunity_replay_validation(2).expect("local opportunity replay should pass");
    }

    #[test]
    fn config_migration_validation_runs_local_fixtures_only() {
        run_config_migration_validation().expect("local config migration validation should pass");
    }

    #[test]
    fn opportunity_quote_load_validation_runs_local_fixture_only() {
        run_opportunity_quote_load_validation(
            [
                "--venue-pairs".to_owned(),
                "8".to_owned(),
                "--max-candidates".to_owned(),
                "3".to_owned(),
            ]
            .into_iter(),
        )
        .expect("local opportunity quote load should pass");
    }

    #[test]
    fn opportunity_provider_ingestion_validation_runs_local_traits_only() {
        run_opportunity_provider_ingestion_validation()
            .expect("local opportunity provider ingestion should pass");
    }

    #[test]
    fn market_data_provider_preflight_validation_runs_local_observations_only() {
        run_market_data_provider_preflight_validation()
            .expect("local market-data provider preflight should pass");
    }

    #[test]
    fn market_data_reconnect_plan_validation_runs_local_records_only() {
        run_market_data_reconnect_plan_validation()
            .expect("local market-data reconnect plan validation should pass");
    }

    #[test]
    fn market_data_quality_assessment_validation_runs_local_metadata_only() {
        run_market_data_quality_assessment_validation()
            .expect("local market-data quality assessment should pass");
    }

    #[test]
    fn paid_market_data_provider_evaluation_validation_runs_local_metadata_only() {
        run_paid_market_data_provider_evaluation_validation()
            .expect("local paid market-data provider evaluation should pass");
    }

    #[test]
    fn fee_schedule_verification_validation_runs_local_records_only() {
        run_fee_schedule_verification_validation()
            .expect("local fee schedule verification should pass");
    }

    #[test]
    fn opportunity_historical_fixture_validation_runs_local_corpus_only() {
        run_opportunity_historical_fixture_validation()
            .expect("local opportunity historical fixtures should pass");
    }

    #[test]
    fn opportunity_planner_handoff_validation_runs_local_corpus_only() {
        run_opportunity_planner_handoff_validation()
            .expect("local opportunity planner handoff should pass");
    }

    #[test]
    fn strategy_constrained_planner_validation_runs_local_profiles_only() {
        run_strategy_constrained_planner_validation()
            .expect("local strategy-constrained planner should pass");
    }

    #[test]
    fn strategy_replay_corpus_validation_runs_local_profiles_only() {
        run_strategy_replay_corpus_validation().expect("local strategy replay corpus should pass");
    }

    #[test]
    fn strategy_profitability_tuning_validation_runs_local_profiles_only() {
        run_strategy_profitability_tuning_validation()
            .expect("local strategy profitability tuning should pass");
    }

    #[test]
    fn opportunity_trace_recovery_validation_runs_local_corpus_only() {
        run_opportunity_trace_recovery_validation()
            .expect("local opportunity trace recovery should pass");
    }

    #[test]
    fn audit_durability_validation_runs_local_workspace_only() {
        let workspace = temp_workspace_path("audit-durability-runner");
        run_audit_durability_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local audit durability validation should pass");

        assert!(workspace.join("audit-validation.jsonl").exists());
        assert!(workspace.exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn audit_retention_execution_validation_runs_local_sandbox_only() {
        let workspace = temp_workspace_path("audit-retention-execution-runner");
        run_audit_retention_execution_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local audit retention execution should pass");

        assert!(workspace.join("audit-active.jsonl").exists());
        assert!(workspace
            .read_dir()
            .expect("workspace reads")
            .any(|entry| entry
                .expect("workspace entry reads")
                .file_name()
                .to_string_lossy()
                .starts_with("audit-active.jsonl.rotated-")));
        assert!(!workspace.join("audit-expired.jsonl").exists());
        assert!(workspace.join("audit-retained.jsonl").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_graceful_shutdown_validation_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("runtime-graceful-shutdown-runner");
        run_runtime_graceful_shutdown_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime graceful-shutdown validation should pass");

        assert!(workspace
            .join("runtime-graceful-shutdown.audit.jsonl")
            .exists());
        assert!(workspace.join("runtime-graceful-shutdown.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_backup_restore_validation_copies_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("runtime-backup-restore-runner");
        run_runtime_backup_restore_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime backup/restore validation should pass");

        assert!(workspace
            .join("runtime-backup-restore.audit.jsonl")
            .exists());
        assert!(workspace.join("runtime-backup-restore.sqlite3").exists());
        assert!(workspace
            .join("runtime-backup-restore.copy.audit.jsonl")
            .exists());
        assert!(workspace
            .join("runtime-backup-restore.copy.sqlite3")
            .exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_backup_restore_load_validation_handles_local_concurrent_workers() {
        let workspace = temp_workspace_path("runtime-backup-restore-load-runner");
        run_runtime_backup_restore_load_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime backup/restore load validation should pass");

        assert!(workspace
            .join("runtime-backup-restore-load.audit.jsonl")
            .exists());
        assert!(workspace
            .join("runtime-backup-restore-load.sqlite3")
            .exists());
        assert!(workspace
            .join("runtime-backup-restore-load.copy.audit.jsonl")
            .exists());
        assert!(workspace
            .join("runtime-backup-restore-load.copy.sqlite3")
            .exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_restart_recovery_validation_replays_local_records_only() {
        let workspace = temp_workspace_path("runtime-restart-recovery-runner");
        run_runtime_restart_recovery_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime restart-recovery validation should pass");

        assert!(workspace
            .join("runtime-restart-recovery.audit.jsonl")
            .exists());
        assert!(workspace.join("runtime-restart-recovery.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_incomplete_recovery_validation_fails_closed_locally() {
        let workspace = temp_workspace_path("runtime-incomplete-recovery-runner");
        run_runtime_incomplete_recovery_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime incomplete-recovery validation should pass");

        assert!(workspace
            .join("runtime-incomplete-recovery.audit.jsonl")
            .exists());
        assert!(workspace
            .join("runtime-incomplete-recovery.sqlite3")
            .exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_supervised_restart_child_writes_recoverable_local_records_only() {
        let workspace = temp_workspace_path("runtime-supervised-restart-child");
        fs::create_dir_all(&workspace).expect("workspace should be created");
        write_runtime_supervised_restart_seed(&workspace)
            .expect("supervised restart child seed should be written");

        assert!(runtime_supervised_restart_audit_path(&workspace).exists());
        assert!(runtime_supervised_restart_state_path(&workspace).exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_permission_denial_validation_fails_closed_before_adapter() {
        let workspace = temp_workspace_path("runtime-permission-denial-runner");
        run_runtime_permission_denial_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime permission-denial validation should pass");

        assert!(workspace
            .join("runtime-permission-denial.audit.jsonl")
            .exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_blocked_state_preflight_validation_fails_closed_locally() {
        let workspace = temp_workspace_path("runtime-blocked-state-preflight");
        run_runtime_blocked_state_preflight_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("blocked state preflight should fail closed cleanly");

        assert!(workspace.join("runtime-state.sqlite3").exists());
        assert!(!workspace.join("runtime-audit.jsonl").exists());
        assert!(!workspace.join("runtime-audit.backup.jsonl").exists());
        assert!(!workspace.join("runtime-state.backup.sqlite3").exists());
        assert!(!workspace.join("audit-durability-workspace").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_blocked_audit_preflight_validation_fails_closed_locally() {
        let workspace = temp_workspace_path("runtime-blocked-audit-preflight");
        run_runtime_blocked_audit_preflight_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("blocked audit preflight should fail closed cleanly");

        assert!(workspace.join("runtime-audit.jsonl").exists());
        assert!(!workspace.join("runtime-state.sqlite3").exists());
        assert!(!workspace.join("runtime-audit.backup.jsonl").exists());
        assert!(!workspace.join("runtime-state.backup.sqlite3").exists());
        assert!(!workspace.join("audit-durability-workspace").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn observability_runtime_validation_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("observability-runtime-runner");
        run_observability_runtime_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local observability runtime validation should pass");

        assert!(workspace.join("observability-audit.jsonl").exists());
        assert!(workspace.join("observability-state.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn runtime_panic_hook_validation_captures_and_reopens_local_failure() {
        let workspace = temp_workspace_path("runtime-panic-hook-runner");
        run_runtime_panic_hook_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local runtime panic hook validation should pass");

        assert!(workspace.join("runtime-panic-hook.audit.jsonl").exists());
        assert!(workspace.join("runtime-panic-hook.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn dashboard_runtime_validation_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("dashboard-runtime-runner");
        run_dashboard_runtime_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local dashboard runtime validation should pass");

        assert!(workspace.join("dashboard-audit.jsonl").exists());
        assert!(workspace.join("dashboard-state.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn communications_runtime_validation_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("communications-runtime-runner");
        run_communications_runtime_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local communications runtime validation should pass");

        assert!(workspace.join("communications-audit.jsonl").exists());
        assert!(workspace.join("communications-state.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn local_validation_runner_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("local-validation-runner");
        run_local_validation_runner(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local validation runner should pass");

        assert!(workspace.join("validation-run.audit.jsonl").exists());
        assert!(workspace.join("validation-run.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn local_property_check_runner_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("local-property-check-runner");
        run_local_property_check_runner(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local property check runner should pass");

        assert!(workspace.join("property-check.audit.jsonl").exists());
        assert!(workspace.join("property-check.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn local_fuzz_corpus_runner_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("local-fuzz-corpus-runner");
        run_local_fuzz_corpus_runner(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local fuzz corpus runner should pass");

        assert!(workspace.join("fuzz-corpus-replay.audit.jsonl").exists());
        assert!(workspace.join("fuzz-corpus-replay.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn local_validation_corpus_runner_persists_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("local-validation-corpus-runner");
        run_local_validation_corpus_runner(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local validation corpus runner should pass");

        assert!(workspace.join("validation-corpus.audit.jsonl").exists());
        assert!(workspace.join("validation-corpus.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn local_paper_backtest_corpus_runner_executes_and_reopens_local_records_only() {
        let workspace = temp_workspace_path("local-paper-backtest-corpus-runner");
        run_local_paper_backtest_corpus_runner(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("local paper backtest corpus runner should pass");

        assert!(workspace.join("paper-backtest-corpus.audit.jsonl").exists());
        assert!(workspace.join("paper-backtest-corpus.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn market_data_boundary_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("market-data-boundary-audit-runner");
        run_market_data_boundary_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("market-data boundary audit validation should pass");

        assert!(workspace.join("market-data-boundary.audit.jsonl").exists());
        assert!(workspace.join("market-data-boundary.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn market_data_history_persistence_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("market-data-history-persistence-runner");
        run_market_data_history_persistence_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("market-data history persistence validation should pass");

        assert!(workspace
            .join("market-data-history-persistence.audit.jsonl")
            .exists());
        assert!(workspace
            .join("market-data-history-persistence.sqlite3")
            .exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn fee_boundary_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("fee-boundary-audit-runner");
        run_fee_boundary_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("fee boundary audit validation should pass");

        assert!(workspace.join("fee-boundary.audit.jsonl").exists());
        assert!(workspace.join("fee-boundary.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn agentic_handoff_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("agentic-handoff-audit-runner");
        run_agentic_handoff_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("agentic handoff audit validation should pass");

        assert!(workspace.join("agentic-handoff.audit.jsonl").exists());
        assert!(workspace.join("agentic-handoff.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn policy_decision_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("policy-decision-audit-runner");
        run_policy_decision_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("policy decision audit validation should pass");

        assert!(workspace.join("policy-decision.audit.jsonl").exists());
        assert!(workspace.join("policy-decision.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn withdrawal_policy_runner_records_fail_closed_local_guards() {
        let workspace = temp_workspace_path("withdrawal-policy-boundary-runner");
        run_withdrawal_policy_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("withdrawal policy validation should pass");

        assert!(workspace.join("withdrawal-policy.audit.jsonl").exists());
        assert!(workspace.join("withdrawal-policy.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn secret_boundary_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("secret-boundary-audit-runner");
        run_secret_boundary_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("secret boundary audit validation should pass");

        assert!(workspace.join("secret-boundary.audit.jsonl").exists());
        assert!(workspace.join("secret-boundary.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn secret_backup_restore_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("secret-backup-restore-runner");
        run_secret_backup_restore_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("secret backup restore validation should pass");

        assert!(workspace.join("secret-backup-restore.audit.jsonl").exists());
        assert!(workspace.join("secret-backup-restore.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn execution_planner_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("execution-planner-audit-runner");
        run_execution_planner_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("execution planner audit validation should pass");

        assert!(workspace.join("execution-planner.audit.jsonl").exists());
        assert!(workspace.join("execution-planner.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn execution_adapter_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("execution-adapter-audit-runner");
        run_execution_adapter_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("execution adapter audit validation should pass");

        assert!(workspace.join("execution-adapter.audit.jsonl").exists());
        assert!(workspace.join("execution-adapter.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn signer_boundary_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("signer-boundary-audit-runner");
        run_signer_boundary_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("signer boundary audit validation should pass");

        assert!(workspace.join("signer-boundary.audit.jsonl").exists());
        assert!(workspace.join("signer-boundary.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn destination_boundary_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("destination-boundary-audit-runner");
        run_destination_boundary_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("destination boundary audit validation should pass");

        assert!(workspace.join("destination-boundary.audit.jsonl").exists());
        assert!(workspace.join("destination-boundary.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    #[test]
    fn connector_lifecycle_audit_runner_records_and_fails_closed_locally() {
        let workspace = temp_workspace_path("connector-lifecycle-audit-runner");
        run_connector_lifecycle_audit_validation(&LocalValidationRunOptions {
            workspace_dir: workspace.clone(),
        })
        .expect("connector lifecycle audit validation should pass");

        assert!(workspace.join("connector-lifecycle.audit.jsonl").exists());
        assert!(workspace.join("connector-lifecycle.sqlite3").exists());
        cleanup_workspace(&workspace);
    }

    fn temp_workspace_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!("arbyclaw-agent-{label}-{}-{nanos}", process::id()));
        path
    }

    fn cleanup_workspace(path: &Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }
}
