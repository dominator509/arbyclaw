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
pub mod destination;
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
pub mod signer;
pub mod state;
pub mod strategy;
pub mod testing;

pub use cex::{
    append_cex_order_lifecycle_audit, append_cex_order_validation_audit,
    persist_cex_order_lifecycle_checkpoint, persist_cex_order_validation_checkpoint,
    validate_cex_client_order_id_uniqueness, validate_cex_credential_scope_review,
    validate_cex_rate_limit, CexAssetBalanceSnapshot, CexBalanceSnapshotRecord,
    CexBalanceSnapshotTranscript, CexBalanceSnapshotTranscriptFormat, CexConnectorCapabilities,
    CexConnectorError, CexConnectorIdentity, CexConnectorRegistry, CexConnectorViolation,
    CexCredentialPermission, CexCredentialScopeReviewInput, CexCredentialScopeReviewReport,
    CexCredentialScopeReviewStatus, CexExchangeFixtureValidation, CexExchangeMarketDataFormat,
    CexExchangeMatchingRules, CexMarketDataRequestKind, CexMarketDataRequestPlan,
    CexMockMarketDataTranscript, CexOrderLifecycleRecord, CexOrderLifecycleResponse,
    CexOrderLifecycleTranscript, CexOrderLifecycleTranscriptFormat, CexOrderRequest, CexOrderSide,
    CexOrderStatus, CexOrderType, CexOrderValidationRecord, CexPolicyGate, CexRateLimitObservation,
    CexRateLimitReport, CexRateLimitScope, CexRateLimitStatus, CexReadOnlyConnector,
    CexTimeInForce, CexTradingConnector, CexVenueProfile, LocalDeterministicCexAdapter,
    CEX_CONNECTOR_FRAMEWORK_VERSION, CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY,
    CEX_LAST_ORDER_VALIDATION_CHECKPOINT_KEY, CEX_STATE_SUBSYSTEM,
};

pub use dex::{
    append_dex_swap_lifecycle_audit, append_dex_swap_validation_audit,
    append_web3_broadcast_adapter_control_review_audit, append_web3_broadcast_readiness_audit,
    append_web3_nonce_reservation_audit, append_web3_pre_sign_safety_audit,
    append_web3_provider_nonce_reconciliation_audit,
    append_web3_raw_transaction_serialization_review_audit,
    append_web3_sandbox_live_discrepancy_calibration_audit,
    append_web3_unsigned_payload_review_audit, append_web3_unsigned_transaction_construction_audit,
    persist_dex_swap_lifecycle_checkpoint, persist_dex_swap_validation_checkpoint,
    persist_web3_broadcast_adapter_control_review_checkpoint,
    persist_web3_broadcast_readiness_checkpoint, persist_web3_nonce_reservation_checkpoint,
    persist_web3_pre_sign_safety_checkpoint, persist_web3_provider_nonce_reconciliation_checkpoint,
    persist_web3_raw_transaction_serialization_review_checkpoint,
    persist_web3_sandbox_live_discrepancy_calibration_checkpoint,
    persist_web3_unsigned_payload_review_checkpoint,
    persist_web3_unsigned_transaction_construction_checkpoint, validate_dex_intent_id_uniqueness,
    DexConnectorError, DexConnectorIdentity, DexConnectorRegistry, DexConnectorViolation,
    DexPolicyGate, DexProtocolRiskReviewReport, DexProtocolRiskReviewRequest,
    DexProtocolRiskReviewStatus, DexQuoteConnector, DexRequestPlan, DexRequestPlanKind,
    DexResponseTranscript, DexRouteKind, DexRouterCapabilities, DexRouterProfile,
    DexSimulationStatus, DexSwapLifecycleRecord, DexSwapMode, DexSwapQuoteRequest,
    DexSwapQuoteResponse, DexSwapValidationRecord, DexTokenProfile, LocalDeterministicDexAdapter,
    Web3BroadcastAdapterControlReviewReport, Web3BroadcastAdapterControlReviewRequest,
    Web3BroadcastAdapterControlReviewStatus, Web3BroadcastReadinessReport,
    Web3BroadcastReadinessRequest, Web3BroadcastReadinessStatus, Web3ChainProfile,
    Web3NonceReservationReport, Web3NonceReservationRequest, Web3NonceReservationStatus,
    Web3PreSignSafetyReviewReport, Web3PreSignSafetyReviewRequest, Web3PreSignSafetyReviewStatus,
    Web3ProviderNonceReconciliationReport, Web3ProviderNonceReconciliationRequest,
    Web3ProviderNonceReconciliationStatus, Web3RawTransactionSerializationReviewReport,
    Web3RawTransactionSerializationReviewRequest, Web3RawTransactionSerializationReviewStatus,
    Web3SandboxLiveDiscrepancyCalibrationReport, Web3SandboxLiveDiscrepancyCalibrationRequest,
    Web3SandboxLiveDiscrepancyCalibrationStatus, Web3SimulationConnector,
    Web3TransactionLifecycleRecord, Web3TransactionLifecycleStatus,
    Web3TransactionLifecycleTranscript, Web3TransactionLifecycleTranscriptFormat,
    Web3TransactionSimulationRequest, Web3TransactionSimulationResponse,
    Web3UnsignedPayloadReviewReport, Web3UnsignedPayloadReviewRequest,
    Web3UnsignedPayloadReviewStatus, Web3UnsignedTransactionConstructionReport,
    Web3UnsignedTransactionConstructionRequest, Web3UnsignedTransactionConstructionStatus,
    DEX_CONNECTOR_FRAMEWORK_VERSION, DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY,
    DEX_LAST_SWAP_VALIDATION_CHECKPOINT_KEY,
    DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY,
    DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY,
    DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY, DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY,
    DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY,
    DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY,
    DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY,
    DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY,
    DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY, DEX_STATE_SUBSYSTEM,
};

pub use communications::{
    append_channel_adapter_validation_audit, append_channel_session_validation_audit,
    append_notification_dispatch_audit, append_platform_adapter_review_audit,
    append_platform_command_ingress_audit, append_remote_command_envelope_validation_audit,
    append_remote_command_security_review_audit, append_routed_operator_command_audit,
    parse_cli_command, persist_channel_adapter_validation_checkpoint,
    persist_channel_session_validation_checkpoint, persist_notification_dispatch_checkpoint,
    persist_platform_adapter_review_checkpoint, persist_platform_command_ingress_checkpoint,
    persist_remote_command_envelope_validation_checkpoint,
    persist_remote_command_security_review_checkpoint, persist_routed_operator_command_checkpoint,
    review_platform_adapter_controls, review_platform_command_ingress,
    review_remote_command_security, validate_channel_adapter, validate_channel_session,
    validate_remote_command_envelope, ChannelAdapterValidationReport,
    ChannelAdapterValidationRequest, ChannelAdapterValidationStatus,
    ChannelSessionValidationReport, ChannelSessionValidationStatus, CommunicationBoundaryConfig,
    CommunicationChannelKind, CommunicationError, CommunicationViolation,
    DeterministicNotificationBoundary, DeterministicOperatorCommandRouter,
    NotificationChannelDispatch, NotificationChannelDispatchStatus, NotificationChannelProfile,
    NotificationChannelSafetyState, NotificationDispatchRecord, NotificationDispatchStatus,
    NotificationPublishRequest, NotificationPublisher, NotificationSeverity, OperatorCommand,
    OperatorCommandAction, OperatorCommandAuthorizationStatus, OperatorCommandKind,
    OperatorCommandRouter, OperatorCommandRoutingRequest, OperatorCommandSource,
    OperatorNotification, PlatformAdapterReviewReport, PlatformAdapterReviewRequest,
    PlatformAdapterReviewStatus, PlatformCommandIngressReport, PlatformCommandIngressRequest,
    PlatformCommandIngressStatus, RemoteCommandEnvelopeValidationReport,
    RemoteCommandEnvelopeValidationRequest, RemoteCommandEnvelopeValidationStatus,
    RemoteCommandSecurityReviewReport, RemoteCommandSecurityReviewRequest,
    RemoteCommandSecurityReviewStatus, RoutedOperatorCommand, COMMUNICATIONS_CLI_VERSION,
    COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY,
    COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY, COMMUNICATIONS_STATE_SUBSYSTEM,
};

pub use dashboard::{
    append_dashboard_hosted_request_preflight_audit,
    append_dashboard_hosted_request_validation_audit,
    append_dashboard_hosted_security_review_audit,
    append_dashboard_hosted_session_validation_audit, append_dashboard_render_audit,
    persist_dashboard_hosted_request_preflight_checkpoint,
    persist_dashboard_hosted_request_validation_checkpoint,
    persist_dashboard_hosted_security_review_checkpoint,
    persist_dashboard_hosted_session_validation_checkpoint, persist_dashboard_render_checkpoint,
    preflight_dashboard_hosted_request, review_dashboard_hosted_runtime_readiness,
    review_dashboard_hosted_security, validate_dashboard_hosted_request,
    validate_dashboard_hosted_session, DashboardAccessAuthorizationStatus, DashboardAccessContext,
    DashboardAccessSource, DashboardBoundaryConfig, DashboardError, DashboardHostedRequestMethod,
    DashboardHostedRequestPreflight, DashboardHostedRequestPreflightReport,
    DashboardHostedRequestPreflightStatus, DashboardHostedRequestValidation,
    DashboardHostedRequestValidationReport, DashboardHostedRequestValidationStatus,
    DashboardHostedRuntimeReadinessReviewReport, DashboardHostedRuntimeReadinessReviewRequest,
    DashboardHostedRuntimeReadinessReviewStatus, DashboardHostedSecurityPolicy,
    DashboardHostedSecurityReviewReport, DashboardHostedSecurityReviewStatus,
    DashboardHostedSessionValidationReport, DashboardHostedSessionValidationStatus, DashboardPanel,
    DashboardPanelItem, DashboardPanelKind, DashboardRenderRecord, DashboardRenderRequest,
    DashboardRenderer, DashboardServerBinding, DashboardSeverity, DashboardSnapshot,
    DashboardViolation, DeterministicDashboardRenderer, DASHBOARD_BOUNDARY_VERSION,
    DASHBOARD_HOSTED_RUNTIME_READINESS_REVIEW_VERSION,
    DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY,
    DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY, DASHBOARD_LAST_RENDER_CHECKPOINT_KEY,
    DASHBOARD_STATE_SUBSYSTEM,
};

pub use destination::{
    append_destination_allowlist_audit, append_destination_ownership_review_audit,
    persist_destination_allowlist_checkpoint, persist_destination_ownership_review_checkpoint,
    ApprovedDestinationEntry, DestinationAllowlist, DestinationAllowlistError,
    DestinationAllowlistViolation, DestinationApprovalSource, DestinationOwnershipReviewReport,
    DestinationOwnershipReviewStatus, DESTINATION_ALLOWLIST_CHECKPOINT_KEY,
    DESTINATION_ALLOWLIST_STATE_SUBSYSTEM, DESTINATION_ALLOWLIST_VERSION,
    DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY,
};

pub use audit::{
    execute_local_audit_retention, plan_audit_journal_retention, plan_audit_stale_lock_recheck,
    validate_audit_journal_durability, validate_deployment_disk_full_transcript,
    validate_deployment_retention_transcript, AppendOnlyAuditJournal,
    AuditDeploymentDiskFullTranscript, AuditDeploymentDiskFullTranscriptReport,
    AuditDeploymentDiskFullTranscriptStatus, AuditDeploymentRetentionTranscript,
    AuditDeploymentRetentionTranscriptReport, AuditDeploymentRetentionTranscriptStatus,
    AuditDurabilityValidationReport, AuditError, AuditEvent, AuditEventKind,
    AuditJournalFileMetadata, AuditLockFileMetadata, AuditRecord, AuditRetentionExecutionReport,
    AuditRetentionExecutionRequest, AuditRetentionPlan, AuditRetentionPolicy, AuditStaleLockPlan,
    AuditStaleLockPolicy, AuditValue, AuditViolation,
    AUDIT_DEPLOYMENT_DISK_FULL_TRANSCRIPT_VERSION, AUDIT_DEPLOYMENT_RETENTION_TRANSCRIPT_VERSION,
    AUDIT_DURABILITY_VALIDATION_VERSION, AUDIT_GENESIS_HASH, AUDIT_JOURNAL_FORMAT_VERSION,
};

pub use config::{
    load_config_file, migrate_config_toml_to_current, validate_runtime_config_reload, AgentConfig,
    AuditConfig, CommunicationConfig, ConfigError, ConfigMigrationReport, ConfigMigrationStatus,
    ConfigViolation, RiskLimitsConfig, RuntimeConfig, RuntimeConfigReloadStatus,
    RuntimeConfigReloadValidationReport, RuntimeConfigReloadValidationRequest, SecretBackend,
    SecretsConfig, VenueAllowlistsConfig, CONFIG_SCHEMA_VERSION, LIVE_ACKNOWLEDGEMENT,
};
pub use secrets::{
    append_secret_backup_restore_review_audit, append_secret_rotation_plan_audit,
    persist_secret_backup_restore_review_checkpoint, persist_secret_rotation_plan_checkpoint,
    plan_local_secret_rotation, preflight_local_keystore_entry, review_local_secret_backup_restore,
    EnvSecretProvider, LocalKeystoreEntryPreflightReport, LocalKeystoreEntryPreflightRequest,
    SecretBackupRestoreReviewReport, SecretBackupRestoreReviewRequest,
    SecretBackupRestoreReviewStatus, SecretMaterial, SecretProvider, SecretRef,
    SecretRotationPlanReport, SecretRotationPlanRequest, SecretRotationPlanStatus,
    SecretStoreError, SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY,
    SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY, SECRET_LIFECYCLE_STATE_SUBSYSTEM,
};

pub use signer::{
    append_signer_authorization_envelope_audit, append_signer_request_audit,
    append_signer_secret_scope_review_audit, build_local_signer_authorization_envelope,
    evaluate_local_signer_request, persist_signer_authorization_envelope_checkpoint,
    persist_signer_request_checkpoint, persist_signer_secret_scope_review_checkpoint,
    review_signer_runtime_isolation, review_signer_secret_scope, SignerAuthorizationEnvelopeReport,
    SignerAuthorizationEnvelopeRequest, SignerAuthorizationEnvelopeStatus, SignerRequest,
    SignerRequestRecord, SignerRequestStatus, SignerRuntimeIsolationReviewReport,
    SignerRuntimeIsolationReviewRequest, SignerRuntimeIsolationReviewStatus,
    SignerSecretScopeReviewReport, SignerSecretScopeReviewRequest, SignerSecretScopeReviewStatus,
    SIGNER_BOUNDARY_VERSION, SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY,
    SIGNER_LAST_REQUEST_CHECKPOINT_KEY, SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY,
    SIGNER_STATE_SUBSYSTEM,
};

pub use strategy::{
    StrategyAlertParameters, StrategyCapitalParameters, StrategyExecutionParameters,
    StrategyOpportunityParameters, StrategyPolicyConstraintReport, StrategyPolicyConstraintStatus,
    StrategyProfile, StrategyProfileError, StrategyProfileViolation, StrategyRiskParameters,
    StrategyVenueParameters, STRATEGY_PROFILE_VERSION,
};

pub use fees::{
    append_fee_schedule_verification_audit, persist_fee_schedule_verification_checkpoint,
    review_fee_schedule_reconciliation, validate_fee_schedule_verification, FeeAdjustedEdge,
    FeeEstimate, FeeModelError, FeeModelViolation, FeeProvider, FeeSchedule,
    FeeScheduleReconciliationReviewReport, FeeScheduleReconciliationReviewRequest,
    FeeScheduleReconciliationReviewStatus, FeeScheduleVerificationInput,
    FeeScheduleVerificationReport, FeeScheduleVerificationStatus, LiquidityRole,
    FEE_LAST_VERIFICATION_CHECKPOINT_KEY, FEE_MODEL_VERSION,
    FEE_SCHEDULE_RECONCILIATION_REVIEW_VERSION, FEE_STATE_SUBSYSTEM,
};

pub use handoff::{
    append_agentic_handoff_review_audit, persist_agentic_handoff_review_checkpoint,
    AgenticHandoffBoundaryConfig, AgenticHandoffError, AgenticHandoffPackage,
    AgenticHandoffPackager, AgenticHandoffReviewRecord, AgenticHandoffReviewRequest,
    AgenticHandoffReviewStatus, AgenticHandoffViolation, DeterministicAgenticHandoffPackager,
    HandoffAgentKind, HandoffArtifactKind, HandoffInstructionArtifact,
    AGENTIC_HANDOFF_LAST_REVIEW_CHECKPOINT_KEY, AGENTIC_HANDOFF_STATE_SUBSYSTEM,
    AGENTIC_HANDOFF_VERSION,
};
pub use hardening::{
    DeterministicExternalHardeningReviewer, ExternalHardeningActivityKind,
    ExternalHardeningBoundaryConfig, ExternalHardeningError, ExternalHardeningReviewRecord,
    ExternalHardeningReviewRequest, ExternalHardeningReviewStatus, ExternalHardeningReviewer,
    ExternalHardeningViolation, HardeningEvidenceRecord, HardeningEvidenceStatus,
    ProductionHardeningPlan, EXTERNAL_HARDENING_VERSION,
};
pub use market_data::{
    append_historical_market_data_persistence_audit, append_market_data_provider_preflight_audit,
    append_market_data_quality_assessment_audit, append_market_data_reconnect_plan_audit,
    append_paid_market_data_provider_evaluation_audit, assess_market_data_quality,
    persist_historical_market_data_checkpoint, persist_market_data_provider_preflight_checkpoint,
    persist_market_data_quality_assessment_checkpoint,
    persist_market_data_reconnect_plan_checkpoint,
    persist_paid_market_data_provider_evaluation_checkpoint, review_market_data_bad_data_rejection,
    review_market_data_provider_latency, review_market_data_provider_reconciliation,
    validate_historical_market_data_persistence, validate_market_data_provider_preflight,
    validate_market_data_reconnect_plan, validate_paid_market_data_provider_evaluation,
    FreshnessStatus, HistoricalMarketDataPersistenceInput, HistoricalMarketDataPersistenceReport,
    HistoricalMarketDataPersistenceStatus, MarketDataBadDataRejectionReviewReport,
    MarketDataBadDataRejectionReviewRequest, MarketDataBadDataRejectionReviewStatus,
    MarketDataCapabilities, MarketDataError, MarketDataProvider,
    MarketDataProviderHealthObservation, MarketDataProviderLatencyReviewReport,
    MarketDataProviderLatencyReviewRequest, MarketDataProviderLatencyReviewStatus,
    MarketDataProviderPreflightReport, MarketDataProviderPreflightStatus,
    MarketDataProviderReconciliationReviewReport, MarketDataProviderReconciliationReviewRequest,
    MarketDataProviderReconciliationReviewStatus, MarketDataQualityAssessmentInput,
    MarketDataQualityAssessmentReport, MarketDataQualityAssessmentStatus,
    MarketDataReconnectPlanInput, MarketDataReconnectPlanReport, MarketDataReconnectPlanStatus,
    MarketDataRequest, MarketDataViolation, MarketPair, NormalizedQuote, OrderBookSnapshot,
    PaidMarketDataProviderEvaluationInput, PaidMarketDataProviderEvaluationReport,
    PaidMarketDataProviderEvaluationStatus, PriceLevel, DEFAULT_MARKET_DATA_FRESHNESS_MS,
    MARKET_DATA_BAD_DATA_REJECTION_REVIEW_VERSION,
    MARKET_DATA_LAST_HISTORICAL_PERSISTENCE_CHECKPOINT_KEY,
    MARKET_DATA_LAST_PAID_PROVIDER_EVALUATION_CHECKPOINT_KEY,
    MARKET_DATA_LAST_PROVIDER_PREFLIGHT_CHECKPOINT_KEY,
    MARKET_DATA_LAST_QUALITY_ASSESSMENT_CHECKPOINT_KEY,
    MARKET_DATA_LAST_RECONNECT_PLAN_CHECKPOINT_KEY, MARKET_DATA_MODEL_VERSION,
    MARKET_DATA_PROVIDER_LATENCY_REVIEW_VERSION,
    MARKET_DATA_PROVIDER_RECONCILIATION_REVIEW_VERSION, MARKET_DATA_STATE_SUBSYSTEM,
};

pub use packaging::{
    append_deployment_package_record_audit, append_rollback_validation_audit,
    persist_deployment_package_record_checkpoint, persist_rollback_validation_checkpoint,
    validate_deployment_failure_capture_transcript, validate_deployment_response_drill_rehearsal,
    validate_incident_response_execution_transcript, validate_local_deployment_rollback_plan,
    validate_rollback_execution_transcript, DeploymentEnvironmentKind,
    DeploymentFailureCaptureTranscript, DeploymentFailureCaptureTranscriptReport,
    DeploymentFailureCaptureTranscriptStatus, DeploymentNetworkExposure, DeploymentPackagePlan,
    DeploymentPackageRecord, DeploymentPackageRequest, DeploymentPackageStatus,
    DeploymentResponseDrillRehearsalReport, DeploymentResponseDrillRehearsalRequest,
    DeploymentResponseDrillRehearsalStatus, DeterministicPackagingDeploymentPlanner,
    IncidentResponseExecutionTranscript, IncidentResponseExecutionTranscriptReport,
    IncidentResponseExecutionTranscriptStatus, PackageArtifactKind, PackageTargetPlan,
    PackagingBoundaryConfig, PackagingBoundaryError, PackagingBoundaryViolation,
    PackagingDeploymentPlanner, ReleaseGate, RollbackExecutionTranscript,
    RollbackExecutionTranscriptReport, RollbackExecutionTranscriptStatus, RollbackStep,
    RollbackValidationRecord, RollbackValidationStatus, RuntimeConfigurationStrategy,
    ServiceHardeningProfile, PACKAGING_DEPLOYMENT_FAILURE_CAPTURE_TRANSCRIPT_VERSION,
    PACKAGING_DEPLOYMENT_RESPONSE_DRILL_REHEARSAL_VERSION, PACKAGING_DEPLOYMENT_VERSION,
    PACKAGING_INCIDENT_RESPONSE_EXECUTION_TRANSCRIPT_VERSION,
    PACKAGING_LAST_PACKAGE_RECORD_CHECKPOINT_KEY,
    PACKAGING_LAST_ROLLBACK_VALIDATION_CHECKPOINT_KEY,
    PACKAGING_ROLLBACK_EXECUTION_TRANSCRIPT_VERSION, PACKAGING_STATE_SUBSYSTEM,
};

pub use observability::{
    append_local_tracing_subscriber_audit, append_observability_alert_route_dispatch_audit,
    append_observability_endpoint_preflight_audit, append_observability_export_dry_run_audit,
    append_observability_log_retention_execution_audit,
    append_observability_loopback_bind_validation_audit,
    append_observability_metrics_endpoint_validation_audit,
    append_observability_metrics_scrape_preflight_audit,
    append_observability_operations_review_audit, append_observability_record_audit,
    append_runtime_failure_capture_audit, capture_local_panic_with_scoped_hook,
    capture_local_runtime_failure, execute_local_observability_log_retention,
    install_local_runtime_panic_hook, persist_local_tracing_subscriber_checkpoint,
    persist_observability_alert_route_dispatch_checkpoint,
    persist_observability_endpoint_preflight_checkpoint,
    persist_observability_export_dry_run_checkpoint,
    persist_observability_log_retention_execution_checkpoint,
    persist_observability_loopback_bind_validation_checkpoint,
    persist_observability_metrics_endpoint_validation_checkpoint,
    persist_observability_metrics_scrape_preflight_checkpoint,
    persist_observability_operations_review_checkpoint, persist_observability_record_checkpoint,
    persist_runtime_failure_capture_checkpoint, preflight_observability_endpoint,
    preflight_observability_metrics_scrape, record_observability_alert_route_dispatch,
    render_observability_export_dry_run, review_observability_operations,
    validate_local_tracing_subscriber, validate_observability_loopback_bind,
    validate_observability_metrics_endpoint, ComponentHealthStatus,
    DeterministicObservabilityCollector, HealthStatus, LocalPanicHookCaptureReport,
    LocalRuntimePanicHookGuard, LocalTracingSubscriberValidationReport,
    LocalTracingSubscriberValidationRequest, LocalTracingSubscriberValidationStatus, MetricKind,
    MetricLabel, MetricSample, ObservabilityAccessAuthorizationStatus, ObservabilityAccessContext,
    ObservabilityAccessSource, ObservabilityAlertRouteDispatchReport,
    ObservabilityAlertRouteDispatchRequest, ObservabilityAlertRouteDispatchStatus,
    ObservabilityBoundaryConfig, ObservabilityCollectionRequest, ObservabilityCollector,
    ObservabilityEndpointBinding, ObservabilityEndpointPreflight,
    ObservabilityEndpointPreflightReport, ObservabilityEndpointPreflightStatus, ObservabilityError,
    ObservabilityExportDryRunReport, ObservabilityExportDryRunRequest,
    ObservabilityLogRetentionExecutionReport, ObservabilityLogRetentionExecutionRequest,
    ObservabilityLoopbackBindValidationReport, ObservabilityLoopbackBindValidationRequest,
    ObservabilityLoopbackBindValidationStatus, ObservabilityMetricsEndpointValidationReport,
    ObservabilityMetricsEndpointValidationRequest, ObservabilityMetricsEndpointValidationStatus,
    ObservabilityMetricsScrapePreflightReport, ObservabilityMetricsScrapePreflightRequest,
    ObservabilityMetricsScrapePreflightStatus, ObservabilityOperationsPolicy,
    ObservabilityOperationsReviewReport, ObservabilityOperationsReviewStatus, ObservabilityRecord,
    ObservabilitySeverity, ObservabilitySnapshot, ObservabilityViolation, Runbook, RunbookStep,
    RuntimeFailureCaptureRecord, RuntimeFailureCaptureRequest, RuntimeFailureKind,
    RuntimePanicHookInstallationReport, RuntimePanicHookInstallationRequest, StructuredLogEvent,
    StructuredLogField, OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY, OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_LOG_RETENTION_EXECUTION_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY, OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY,
    OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY, OBSERVABILITY_RUNBOOK_VERSION,
    OBSERVABILITY_STATE_SUBSYSTEM,
};

pub use opportunity::{
    discover_opportunities_from_local_providers,
    phase27_local_opportunity_historical_fixture_corpus, phase27_local_opportunity_replay_corpus,
    review_opportunity_replay_latency, validate_local_opportunity_quote_ingestion_load,
    DeterministicOpportunityEngine, OpportunityCandidate, OpportunityDiscoveryConfig,
    OpportunityDiscoveryRequest, OpportunityEngine, OpportunityError,
    OpportunityHistoricalFixtureCorpus, OpportunityHistoricalFixtureRunReport,
    OpportunityInventoryLimit, OpportunityLeg, OpportunityLegSide, OpportunityLiquidityModel,
    OpportunityProviderIngestionReport, OpportunityProviderIngestionRequest,
    OpportunityQuoteIngestionLoadReport, OpportunityQuoteIngestionLoadRequest,
    OpportunityReplayCorpus, OpportunityReplayExpectation, OpportunityReplayLatencyReviewReport,
    OpportunityReplayLatencyReviewRequest, OpportunityReplayLatencyReviewStatus,
    OpportunityReplayLoadIteration, OpportunityReplayLoadReport, OpportunityReplayRouteCount,
    OpportunityReplayRunReport, OpportunityReplayScenario, OpportunityReplayScenarioReport,
    OpportunityReplayStatus, OpportunityReplayViolation, OpportunityRouteKind, OpportunityScore,
    OpportunityTransferRisk, OpportunityTransferRiskProfile, OpportunityViolation,
    OPPORTUNITY_ENGINE_VERSION, OPPORTUNITY_REPLAY_LATENCY_REVIEW_VERSION,
};

pub use paper::{
    append_paper_execution_intent_audit, append_paper_execution_report_audit,
    append_paper_ledger_entry_audit, append_paper_ledgered_execution_audit,
    ledger_execution_adapter_run_paper_fills, persist_paper_balance_ledger_checkpoint,
    persist_paper_execution_report_checkpoint, validate_paper_runtime, PaperAdapterRunLedgerReport,
    PaperAdverseSelectionConfig, PaperAdverseSelectionReport, PaperAssetBalance,
    PaperAuditJournalWriteReport, PaperAuditReplayValidationReport, PaperAuditedLedgeredExecution,
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
    append_fuzz_corpus_replay_report_audit, append_property_check_report_audit,
    append_validation_corpus_report_audit, append_validation_run_audit,
    persist_fuzz_corpus_replay_report_checkpoint, persist_property_check_report_checkpoint,
    persist_validation_corpus_report_checkpoint, persist_validation_run_checkpoint,
    review_local_validation_coverage, run_local_fuzz_corpus_replay, run_local_validation_corpus,
    run_local_validation_property_checks, BacktestDatasetDefinition, BacktestScenarioDefinition,
    DeterministicValidationHarness, ExpectedValidationOutcome, FixtureKind, FuzzCorpusDefinition,
    FuzzSeedRecord, FuzzTargetKind, LocalFuzzCorpusReplayReport, LocalFuzzCorpusReplayRequest,
    LocalFuzzCorpusReplayStatus, LocalFuzzTargetReplaySummary, LocalPropertyCheckReport,
    LocalValidationCorpusReport, LocalValidationCorpusRequest, LocalValidationCorpusStatus,
    LocalValidationCoverageReviewReport, LocalValidationCoverageReviewRequest,
    LocalValidationCoverageReviewStatus, ValidationExecutionMode, ValidationFixtureRecord,
    ValidationHarness, ValidationHarnessConfig, ValidationHarnessError, ValidationHarnessViolation,
    ValidationPlan, ValidationRunRecord, ValidationRunRequest, ValidationRunStatus,
    ValidationSuiteKind, ValidationTestCase, LOCAL_VALIDATION_COVERAGE_REVIEW_VERSION,
    TESTING_BACKTESTING_VERSION, TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY,
    TESTING_LAST_PROPERTY_CHECK_REPORT_KEY, TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY,
    TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY, TESTING_STATE_SUBSYSTEM,
};

pub use planner::{
    append_execution_plan_draft_audit, persist_execution_plan_draft_checkpoint,
    persist_opportunity_candidate_trace, validate_opportunity_candidate_trace_restart_recovery,
    validate_opportunity_planner_handoff, validate_opportunity_planner_handoff_with_trace,
    validate_strategy_profile_replay_corpus, validate_strategy_profitability_tuning,
    DeterministicExecutionPlanner, ExecutionPlanAuditReport, ExecutionPlanDraft,
    ExecutionPlanFailureMode, ExecutionPlanStatus, ExecutionPlanStep, ExecutionPlanStepAction,
    ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerError, ExecutionPlannerRequest,
    ExecutionPlannerViolation, OpportunityCandidateTraceError,
    OpportunityCandidateTracePersistence, OpportunityCandidateTraceRecord,
    OpportunityCandidateTraceRecoveryReport, OpportunityPlannerHandoffStatus,
    OpportunityPlannerHandoffValidationReport, PlannerPolicyOutcome, PlannerPolicyStatus,
    PlannerPolicyViolation, RecoveredOpportunityTraceSummary,
    StrategyConstrainedExecutionPlanDraft, StrategyProfileReplayValidationReport,
    StrategyProfileReplayValidationStatus, StrategyProfitabilityTuningPoint,
    StrategyProfitabilityTuningValidationReport, StrategyProfitabilityTuningValidationStatus,
    EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY, EXECUTION_PLANNER_STATE_SUBSYSTEM,
    EXECUTION_PLANNER_VERSION, OPPORTUNITY_CANDIDATE_TRACE_CHECKPOINT_KEY_PREFIX,
    OPPORTUNITY_CANDIDATE_TRACE_STATE_SUBSYSTEM,
};

pub use execution_adapter::{
    append_execution_adapter_recovery_plan_audit, append_execution_adapter_run_audit,
    persist_execution_adapter_recovery_plan_checkpoint, persist_execution_adapter_run_checkpoint,
    plan_execution_adapter_recovery, DeterministicExecutionAdapterBoundary, ExecutionAdapter,
    ExecutionAdapterAction, ExecutionAdapterAttempt, ExecutionAdapterAttemptStatus,
    ExecutionAdapterConfig, ExecutionAdapterError, ExecutionAdapterRecoveryAction,
    ExecutionAdapterRecoveryPlan, ExecutionAdapterRecoveryStep, ExecutionAdapterRequest,
    ExecutionAdapterRunRecord, ExecutionAdapterRunStatus, ExecutionAdapterViolation,
    ExecutionFillRecord, ExecutionFillStatus, ExecutionReconciliationRecord,
    ExecutionReconciliationStatus, EXECUTION_ADAPTER_FRAMEWORK_VERSION,
    EXECUTION_ADAPTER_LAST_RECOVERY_PLAN_CHECKPOINT_KEY, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY,
    EXECUTION_ADAPTER_STATE_SUBSYSTEM,
};

pub use state::{
    InMemoryStateStore, SqliteWalDurabilityReport, SqliteWalStateStore, StateCheckpoint,
    StateStore, StateStoreError, SQLITE_WAL_DURABILITY_VERSION, SQLITE_WAL_STATE_SCHEMA_VERSION,
};

pub use runtime::{
    preflight_production_runtime_validation, review_runtime_load_profile,
    run_local_graceful_shutdown_checkpoint, run_local_runtime_lifecycle,
    validate_deployment_audit_sqlite_transcript, validate_deployment_backup_restore_transcript,
    validate_deployment_graceful_shutdown_transcript, validate_deployment_permission_transcript,
    validate_deployment_sqlite_schema_migration_transcript, validate_local_runtime_backup_restore,
    validate_local_runtime_deployment_smoke, validate_local_runtime_restart_recovery,
    validate_local_runtime_restart_recovery_with_trace_recovery,
    validate_service_manager_lifecycle_rehearsal, validate_service_manager_lifecycle_transcript,
    RuntimeBackupRestoreValidationReport, RuntimeDeploymentAuditSqliteTranscript,
    RuntimeDeploymentAuditSqliteTranscriptReport, RuntimeDeploymentAuditSqliteTranscriptStatus,
    RuntimeDeploymentBackupRestoreTranscript, RuntimeDeploymentBackupRestoreTranscriptReport,
    RuntimeDeploymentBackupRestoreTranscriptStatus, RuntimeDeploymentGracefulShutdownTranscript,
    RuntimeDeploymentGracefulShutdownTranscriptReport,
    RuntimeDeploymentGracefulShutdownTranscriptStatus, RuntimeDeploymentPermissionTranscript,
    RuntimeDeploymentPermissionTranscriptReport, RuntimeDeploymentPermissionTranscriptStatus,
    RuntimeDeploymentSmokeLoadIteration, RuntimeDeploymentSmokeLoadValidationReport,
    RuntimeDeploymentSmokeValidationReport, RuntimeDeploymentSmokeValidationRequest,
    RuntimeDeploymentSqliteSchemaMigrationTranscript,
    RuntimeDeploymentSqliteSchemaMigrationTranscriptReport,
    RuntimeDeploymentSqliteSchemaMigrationTranscriptStatus, RuntimeGracefulShutdownRecord,
    RuntimeGracefulShutdownRequest, RuntimeLifecycleError, RuntimeLifecycleRecord,
    RuntimeLifecycleRequest, RuntimeLifecycleStatus, RuntimeLoadProfileReviewReport,
    RuntimeLoadProfileReviewRequest, RuntimeLoadProfileReviewStatus,
    RuntimeOpportunityTraceRecoverySummary, RuntimeProductionPreflightReport,
    RuntimeProductionPreflightRequest, RuntimeProductionPreflightStatus,
    RuntimeRecoveredOpportunityTraceSummary, RuntimeRestartRecoveryDisposition,
    RuntimeRestartRecoveryValidationReport, RuntimeServiceManagerKind,
    RuntimeServiceManagerLifecycleEvent, RuntimeServiceManagerLifecycleEventKind,
    RuntimeServiceManagerLifecycleRehearsalReport, RuntimeServiceManagerLifecycleRehearsalRequest,
    RuntimeServiceManagerLifecycleRehearsalStatus, RuntimeServiceManagerLifecycleTranscript,
    RuntimeServiceManagerLifecycleTranscriptReport, RuntimeServiceManagerLifecycleTranscriptStatus,
    RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION, RUNTIME_DEPLOYMENT_AUDIT_SQLITE_TRANSCRIPT_VERSION,
    RUNTIME_DEPLOYMENT_BACKUP_RESTORE_TRANSCRIPT_VERSION,
    RUNTIME_DEPLOYMENT_GRACEFUL_SHUTDOWN_TRANSCRIPT_VERSION,
    RUNTIME_DEPLOYMENT_PERMISSION_TRANSCRIPT_VERSION, RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION,
    RUNTIME_DEPLOYMENT_SQLITE_SCHEMA_MIGRATION_TRANSCRIPT_VERSION,
    RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY, RUNTIME_GRACEFUL_SHUTDOWN_VERSION,
    RUNTIME_LIFECYCLE_VERSION, RUNTIME_LOAD_PROFILE_REVIEW_VERSION,
    RUNTIME_PRODUCTION_PREFLIGHT_VALIDATION_VERSION, RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION,
    RUNTIME_SERVICE_MANAGER_LIFECYCLE_REHEARSAL_VERSION,
    RUNTIME_SERVICE_MANAGER_LIFECYCLE_TRANSCRIPT_VERSION,
};

pub use policy::{
    append_policy_decision_audit, persist_policy_decision_checkpoint, DestinationPolicy,
    ExecutionIntent, ExecutionIntentKind, ExecutionScope, PolicyApproval, PolicyContext,
    PolicyDecision, PolicyDecisionRecord, PolicyEngine, PolicyViolation, VenueKind, VenueRef,
    DEFAULT_MAX_MARKET_DATA_AGE_MS, POLICY_LAST_DECISION_CHECKPOINT_KEY, POLICY_STATE_SUBSYSTEM,
    TRUST_CONTRACT_VERSION,
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
