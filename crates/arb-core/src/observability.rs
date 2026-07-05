#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    execute_local_audit_retention, AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord,
    AuditRetentionExecutionReport, AuditRetentionExecutionRequest, AuditValue,
    NotificationChannelDispatchStatus, NotificationDispatchRecord, NotificationDispatchStatus,
    SqliteWalStateStore, StateCheckpoint, StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    error::Error,
    fmt,
    io::{Read, Write},
    net::{IpAddr, TcpListener, TcpStream},
    panic::{self, AssertUnwindSafe},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tracing_subscriber::fmt::MakeWriter;

/// Stable observability and runbook boundary version for audit, replay, and handoff surfaces.
pub const OBSERVABILITY_RUNBOOK_VERSION: &str = "phase-14-observability-runbook-v1";

/// State-store subsystem name for local observability checkpoints.
pub const OBSERVABILITY_STATE_SUBSYSTEM: &str = "observability";

/// State-store key for the latest local observability collection record.
pub const OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY: &str = "observability:last-record";

/// State-store key for the latest local runtime failure capture record.
pub const OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY: &str = "observability:last-failure-capture";

/// State-store key for the latest local observability operations review.
pub const OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY: &str =
    "observability:last-operations-review";

/// State-store key for the latest local observability export/alert dry run.
pub const OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY: &str =
    "observability:last-export-dry-run";

/// State-store key for the latest local observability endpoint/exporter preflight.
pub const OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY: &str =
    "observability:last-endpoint-preflight";

/// State-store key for the latest local observability loopback bind validation.
pub const OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY: &str =
    "observability:last-loopback-bind-validation";

/// State-store key for the latest local authenticated metrics scrape preflight.
pub const OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY: &str =
    "observability:last-metrics-scrape-preflight";

/// State-store key for the latest local one-shot metrics endpoint validation.
pub const OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY: &str =
    "observability:last-metrics-endpoint-validation";

/// State-store key for the latest bounded local metrics runtime probe.
pub const OBSERVABILITY_LAST_METRICS_RUNTIME_PROBE_CHECKPOINT_KEY: &str =
    "observability:last-metrics-runtime-probe";

/// State-store key for the latest local tracing subscriber validation.
pub const OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY: &str =
    "observability:last-tracing-subscriber";

/// State-store key for the latest local observability log retention execution.
pub const OBSERVABILITY_LAST_LOG_RETENTION_EXECUTION_CHECKPOINT_KEY: &str =
    "observability:last-log-retention-execution";

/// State-store key for the latest local observability alert-route dispatch review.
pub const OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY: &str =
    "observability:last-alert-route-dispatch";

/// Conservative observability boundary settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityBoundaryConfig {
    /// Whether local in-process observability records may be collected.
    pub local_collection_enabled: bool,
    /// Future metrics endpoint model. Phase 14 must not start an endpoint.
    pub metrics_endpoint: ObservabilityEndpointBinding,
    /// Maximum structured log fields to include per event.
    pub max_log_fields: usize,
    /// Maximum metric labels to include per sample.
    pub max_metric_labels: usize,
    /// Maximum runbook steps to include per runbook.
    pub max_runbook_steps: usize,
    /// Whether outbound alert delivery is enabled. Phase 14 requires false.
    pub outbound_alerts_enabled: bool,
    /// Whether secret-like telemetry may be collected. Phase 14 requires false.
    pub allow_secret_observability: bool,
    /// Whether observability collection requires a local access authorization decision.
    pub require_local_observability_authorization: bool,
    /// Whether exporter or outbound alert sessions are enabled. Phase 14 requires false.
    pub external_observability_sessions_enabled: bool,
}

impl Default for ObservabilityBoundaryConfig {
    fn default() -> Self {
        Self {
            local_collection_enabled: true,
            metrics_endpoint: ObservabilityEndpointBinding::default(),
            max_log_fields: 32,
            max_metric_labels: 16,
            max_runbook_steps: 24,
            outbound_alerts_enabled: false,
            allow_secret_observability: false,
            require_local_observability_authorization: true,
            external_observability_sessions_enabled: false,
        }
    }
}

impl ObservabilityBoundaryConfig {
    /// Validate fail-closed Phase 14 observability settings.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();

        if self.max_log_fields == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_MAX_LOG_FIELDS_ZERO",
                "max_log_fields must be positive",
            ));
        }

        if self.max_metric_labels == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_MAX_METRIC_LABELS_ZERO",
                "max_metric_labels must be positive",
            ));
        }

        if self.max_runbook_steps == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_MAX_RUNBOOK_STEPS_ZERO",
                "max_runbook_steps must be positive",
            ));
        }

        if self.metrics_endpoint.metrics_endpoint_enabled {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_DENIED_IN_PHASE_14",
                "Phase 14 observability boundaries must not start a metrics endpoint",
            ));
        }

        if self.metrics_endpoint.public_network_exposure {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PUBLIC_NETWORK_DENIED_IN_PHASE_14",
                "Phase 14 observability boundaries must not expose public network bindings",
            ));
        }

        if !self.metrics_endpoint.require_loopback_only {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOOPBACK_REQUIRED",
                "metrics endpoint model must require loopback-only access",
            ));
        }

        if !is_loopback_host(&self.metrics_endpoint.bind_host) {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_BIND_HOST_NOT_LOOPBACK",
                format!(
                    "metrics bind host {} is not loopback-only",
                    self.metrics_endpoint.bind_host
                ),
            ));
        }

        if self.outbound_alerts_enabled {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_OUTBOUND_ALERTS_DENIED_IN_PHASE_14",
                "Phase 14 observability boundaries must not send outbound alerts",
            ));
        }

        if self.allow_secret_observability {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_SECRET_COLLECTION_DENIED",
                "observability records must redact secret-like text",
            ));
        }

        if !self.require_local_observability_authorization {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOCAL_AUTH_REQUIRED",
                "Phase 14 observability collection must require a local authorization decision",
            ));
        }

        if self.external_observability_sessions_enabled {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXTERNAL_SESSIONS_DENIED_IN_PHASE_14",
                "Phase 14 observability boundaries must not enable exporter or outbound alert sessions",
            ));
        }

        finish_validation(violations)
    }
}

/// Future metrics endpoint binding model. It is not an active listener in Phase 14.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityEndpointBinding {
    /// Whether a metrics endpoint should be started. Phase 14 requires false.
    pub metrics_endpoint_enabled: bool,
    /// Intended future bind host. Must remain loopback-only.
    pub bind_host: String,
    /// Intended future bind port.
    pub bind_port: u16,
    /// Whether loopback-only binding is mandatory.
    pub require_loopback_only: bool,
    /// Whether public network exposure is requested. Phase 14 requires false.
    pub public_network_exposure: bool,
}

impl Default for ObservabilityEndpointBinding {
    fn default() -> Self {
        Self {
            metrics_endpoint_enabled: false,
            bind_host: "127.0.0.1".to_owned(),
            bind_port: 9_090,
            require_loopback_only: true,
            public_network_exposure: false,
        }
    }
}

/// Local observability retention and alert-route review input.
///
/// This is a non-secret, side-effect-free review record. It does not start
/// metrics endpoints, export telemetry, ship logs, deliver alerts, mutate
/// retention storage, or contact communications platforms.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityOperationsPolicy {
    /// Stable local review id.
    pub review_id: String,
    /// Whether structured-log retention is required.
    pub log_retention_required: bool,
    /// Retention period in days for local logs/records.
    pub retention_days: u32,
    /// Whether retention review requires redaction validation.
    pub redaction_required: bool,
    /// Whether alert routing must be configured before production observability use.
    pub alert_routing_required: bool,
    /// Number of configured local alert route references.
    pub alert_route_count: u32,
    /// Whether an incident runbook reference is required.
    pub incident_runbook_required: bool,
    /// Number of configured incident runbook references.
    pub incident_runbook_count: u32,
    /// Whether metrics/exporter endpoints must remain loopback/authenticated.
    pub loopback_or_authenticated_endpoint_required: bool,
    /// Whether audit/state preflight must pass before future observability runtime activation.
    pub audit_state_preflight_required: bool,
    /// Whether future exporter sessions require an explicit kill switch.
    pub exporter_kill_switch_required: bool,
    /// Whether future alert routes require an authorization review.
    pub alert_authorization_required: bool,
    /// Whether future exporter/alert paths require rate-limit or backpressure controls.
    pub rate_limit_backpressure_required: bool,
    /// Whether future exporter/alert paths require retry and outage backoff controls.
    pub retry_backoff_required: bool,
    /// Whether future telemetry paths require non-secret telemetry controls.
    pub no_secret_telemetry_required: bool,
    /// Whether a metrics endpoint was requested. Must remain false here.
    pub metrics_endpoint_requested: bool,
    /// Whether outbound alert delivery was requested. Must remain false here.
    pub outbound_alert_delivery_requested: bool,
    /// Whether telemetry export was requested. Must remain false here.
    pub telemetry_export_requested: bool,
}

/// Local observability operations review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityOperationsReviewStatus {
    /// Required controls are represented locally, but production observability is still not approved.
    ReadyForLocalReview,
    /// Required controls are missing or unsafe side-effect flags were requested.
    BlockedMissingControls,
}

/// Local observability retention and alert-route review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityOperationsReviewReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local review id.
    pub review_id: String,
    /// Review status.
    pub status: ObservabilityOperationsReviewStatus,
    /// Whether log retention is required.
    pub log_retention_required: bool,
    /// Retention period in days represented by this local review.
    pub retention_days: u32,
    /// Whether redaction validation is required.
    pub redaction_required: bool,
    /// Whether alert routing is required.
    pub alert_routing_required: bool,
    /// Number of alert route references represented by this local review.
    pub alert_route_count: u32,
    /// Whether incident runbook references are required.
    pub incident_runbook_required: bool,
    /// Number of incident runbook references represented by this local review.
    pub incident_runbook_count: u32,
    /// Whether loopback/authenticated endpoint policy is required.
    pub loopback_or_authenticated_endpoint_required: bool,
    /// Whether audit/state preflight is required before future observability runtime activation.
    pub audit_state_preflight_required: bool,
    /// Whether future exporter sessions require an explicit kill switch.
    pub exporter_kill_switch_required: bool,
    /// Whether future alert routes require an authorization review.
    pub alert_authorization_required: bool,
    /// Whether future exporter/alert paths require rate-limit or backpressure controls.
    pub rate_limit_backpressure_required: bool,
    /// Whether future exporter/alert paths require retry and outage backoff controls.
    pub retry_backoff_required: bool,
    /// Whether future telemetry paths require non-secret telemetry controls.
    pub no_secret_telemetry_required: bool,
    /// Number of missing or unsafe controls.
    pub missing_control_count: u32,
    /// Whether a metrics endpoint was started. Always false for this review.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Always false for this review.
    pub public_network_exposed: bool,
    /// Whether outbound alerts were sent. Always false for this review.
    pub outbound_alerts_sent: bool,
    /// Whether telemetry was exported. Always false for this review.
    pub telemetry_exported: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

/// Local sandbox-only observability log retention/rotation execution request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityLogRetentionExecutionRequest {
    /// Stable execution identifier.
    pub execution_id: String,
    /// Ready local operations review that authorized retention controls.
    pub operations_review: ObservabilityOperationsReviewReport,
    /// Sandbox workspace and caller-supplied local log metadata.
    pub retention_request: AuditRetentionExecutionRequest,
    /// Whether this is explicitly a local sandbox execution. Must be true.
    pub local_sandbox_only: bool,
    /// Whether production log paths are requested. Must be false.
    pub production_log_paths_requested: bool,
    /// Whether service-manager interaction is requested. Must be false.
    pub service_manager_action_requested: bool,
    /// Whether external log shipping is requested. Must be false.
    pub external_log_shipping_requested: bool,
}

/// Local sandbox-only observability log retention/rotation execution report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityLogRetentionExecutionReport {
    /// Observability boundary version.
    pub observability_runbook_version: String,
    /// Stable execution identifier.
    pub execution_id: String,
    /// Linked operations review id.
    pub review_id: String,
    /// Whether the operations review was ready for local review.
    pub operations_review_ready: bool,
    /// Whether active log rotation was requested by the local policy.
    pub rotate_active_requested: bool,
    /// Whether a new active log was created after local rotation.
    pub new_active_created: bool,
    /// Archive files retained inside the sandbox.
    pub retained_archives: Vec<String>,
    /// Archive files deleted inside the sandbox.
    pub expired_archives_deleted: Vec<String>,
    /// Number of sandbox log files deleted.
    pub deleted_file_count: u64,
    /// Whether filesystem mutation occurred inside the explicit sandbox.
    pub sandbox_filesystem_mutated: bool,
    /// Whether deletion was performed inside the explicit sandbox.
    pub sandbox_deletion_performed: bool,
    /// Whether any path outside the sandbox was touched. Always false on success.
    pub out_of_workspace_path_touched: bool,
    /// Whether production log paths were touched. Always false on success.
    pub production_log_paths_touched: bool,
    /// Whether a service-manager action was performed. Always false.
    pub service_manager_action_performed: bool,
    /// Whether external log shipping was performed. Always false.
    pub external_log_shipping_performed: bool,
    /// Whether live network access was used. Always false.
    pub live_network_used: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
}

/// Local non-network observability export and alert-route dry-run request.
///
/// This renders deterministic local telemetry and alert-route accounting from an
/// already sanitized observability record and operations review. It never starts
/// a metrics endpoint, exports telemetry, ships logs, delivers alerts, mutates
/// retention storage, or approves production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityExportDryRunRequest {
    /// Already collected local observability record.
    pub record: ObservabilityRecord,
    /// Local operations review proving required controls are represented.
    pub operations_review: ObservabilityOperationsReviewReport,
    /// Sanitized local alert route references.
    pub alert_route_references: Vec<String>,
    /// Dry-run timestamp in Unix epoch milliseconds.
    pub rendered_at_ms: u64,
}

/// Local non-network observability export and alert-route dry-run report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityExportDryRunReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Observability snapshot identifier.
    pub snapshot_id: String,
    /// Operations review identifier used for this dry run.
    pub review_id: String,
    /// Dry-run timestamp in Unix epoch milliseconds.
    pub rendered_at_ms: u64,
    /// Prometheus-style local metric lines rendered without serving an endpoint.
    pub prometheus_metric_lines: Vec<String>,
    /// Number of structured log events available for future log shipping.
    pub log_event_count: u64,
    /// Number of runbooks available for future alert/runbook routing.
    pub runbook_count: u64,
    /// Number of configured sanitized alert route references.
    pub alert_route_count: u64,
    /// Number of local alert-route decisions that would require operator attention.
    pub alert_dry_run_count: u64,
    /// Whether loopback/authenticated endpoint policy was represented.
    pub loopback_or_authenticated_endpoint_required: bool,
    /// Whether a metrics endpoint was started. Always false for this dry run.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Always false for this dry run.
    pub public_network_exposed: bool,
    /// Whether outbound alerts were sent. Always false for this dry run.
    pub outbound_alerts_sent: bool,
    /// Whether telemetry was exported. Always false for this dry run.
    pub telemetry_exported: bool,
    /// Whether any live execution was performed. Always false for this dry run.
    pub live_execution_performed: bool,
    /// Whether this dry run approves production readiness. Always false.
    pub production_ready: bool,
}

/// Local observability alert-route dispatch bridge input.
///
/// This records that alert-route decisions from the observability dry run were
/// handed to the deterministic local communications notification boundary. It
/// never delivers outbound alerts, calls messaging platforms, exports
/// telemetry, executes live actions, or approves production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityAlertRouteDispatchRequest {
    /// Stable local bridge id.
    pub dispatch_review_id: String,
    /// Local export/alert-route dry run that produced alert decisions.
    pub export_report: ObservabilityExportDryRunReport,
    /// Sanitized alert route reference represented by the local notification.
    pub alert_route_reference: String,
    /// Communications notification dispatch produced by the deterministic local boundary.
    pub notification_dispatch: NotificationDispatchRecord,
    /// Whether routing through the local communications boundary is required.
    pub local_dispatch_required: bool,
    /// Whether outbound alert delivery was requested. Must remain false here.
    pub outbound_alert_delivery_requested: bool,
    /// Local review timestamp in Unix epoch milliseconds.
    pub reviewed_at_ms: u64,
}

/// Local observability alert-route dispatch bridge status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityAlertRouteDispatchStatus {
    /// Alert-route decisions reached the local communications notification boundary.
    ReadyForLocalReview,
    /// Local communications dispatch is missing or unsafe.
    Blocked,
}

/// Local observability alert-route dispatch bridge report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityAlertRouteDispatchReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local bridge id.
    pub dispatch_review_id: String,
    /// Local bridge status.
    pub status: ObservabilityAlertRouteDispatchStatus,
    /// Source observability snapshot id.
    pub snapshot_id: String,
    /// Source operations review id.
    pub review_id: String,
    /// Sanitized alert route reference represented.
    pub alert_route_reference: String,
    /// Communications notification dispatch id.
    pub notification_dispatch_id: String,
    /// Communications notification id.
    pub notification_id: String,
    /// Communications dispatch status.
    pub notification_dispatch_status: NotificationDispatchStatus,
    /// Number of local channel dispatches recorded.
    pub recorded_local_channel_count: u64,
    /// Number of selected channels blocked by local gating.
    pub blocked_channel_count: u64,
    /// Whether local communications dispatch was required.
    pub local_dispatch_required: bool,
    /// Whether outbound alerts were sent. Always false for this bridge.
    pub outbound_alerts_sent: bool,
    /// Whether outbound network was used. Always false for accepted reports.
    pub outbound_network_used: bool,
    /// Whether telemetry was exported. Always false for this bridge.
    pub telemetry_exported: bool,
    /// Whether any live execution was performed. Always false for this bridge.
    pub live_execution_performed: bool,
    /// Whether this bridge approves production readiness. Always false.
    pub production_ready: bool,
    /// Local review timestamp in Unix epoch milliseconds.
    pub reviewed_at_ms: u64,
}

/// Local authenticated metrics scrape preflight input.
///
/// This models one future scrape request against already-rendered local metric
/// lines. It never starts a server, accepts network connections, exports
/// telemetry, ships logs, or sends alerts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityMetricsScrapePreflightRequest {
    /// Stable local scrape preflight id.
    pub scrape_id: String,
    /// Local export dry-run whose rendered metric lines are being scraped.
    pub export_report: ObservabilityExportDryRunReport,
    /// HTTP method represented by the future scrape request.
    pub request_method: String,
    /// Request path represented by the future scrape request.
    pub request_path: String,
    /// Source host represented by the future scrape request.
    pub source_host: String,
    /// Whether authentication is required before serving metrics.
    pub authentication_required: bool,
    /// Whether authorization is required before serving metrics.
    pub authorization_required: bool,
    /// Whether a non-secret token reference/header reference was present.
    pub bearer_token_reference_present: bool,
    /// Whether a metrics endpoint startup was requested. Must remain false here.
    pub metrics_endpoint_start_requested: bool,
    /// Whether public network exposure was requested. Must remain false here.
    pub public_network_exposure_requested: bool,
    /// Whether telemetry export was requested. Must remain false here.
    pub telemetry_export_requested: bool,
    /// Whether outbound alert delivery was requested. Must remain false here.
    pub outbound_alert_delivery_requested: bool,
}

/// Local authenticated metrics scrape preflight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityMetricsScrapePreflightStatus {
    /// The represented scrape is locally coherent, but no endpoint was hosted.
    ReadyForLocalReview,
    /// Required local scrape controls are missing or unsafe.
    Blocked,
}

/// Local authenticated metrics scrape preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityMetricsScrapePreflightReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local scrape preflight id.
    pub scrape_id: String,
    /// Preflight status.
    pub status: ObservabilityMetricsScrapePreflightStatus,
    /// Source snapshot id from the rendered export dry-run.
    pub snapshot_id: String,
    /// HTTP method represented by the future scrape request.
    pub request_method: String,
    /// Request path represented by the future scrape request.
    pub request_path: String,
    /// Source host represented by the future scrape request.
    pub source_host: String,
    /// Whether the represented scrape source is loopback-only.
    pub loopback_source_validated: bool,
    /// Whether authentication was represented as required.
    pub authentication_required: bool,
    /// Whether authorization was represented as required.
    pub authorization_required: bool,
    /// Whether a non-secret token/header reference was represented.
    pub bearer_token_reference_present: bool,
    /// Local HTTP status that a future scrape handler would return.
    pub local_http_status_code: u16,
    /// Number of rendered metrics available to the local scrape.
    pub response_metric_line_count: u64,
    /// Sanitized local metric lines returned by the in-process scrape preflight.
    pub response_metric_lines: Vec<String>,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a metrics endpoint was started. Always false for this preflight.
    pub metrics_endpoint_started: bool,
    /// Whether any requests were served over a socket. Always false here.
    pub network_request_served: bool,
    /// Whether public network exposure occurred. Always false for this preflight.
    pub public_network_exposed: bool,
    /// Whether telemetry was exported. Always false for this preflight.
    pub telemetry_exported: bool,
    /// Whether outbound alerts were sent. Always false for this preflight.
    pub outbound_alerts_sent: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local warnings.
    pub warnings: Vec<String>,
}

/// Local one-shot authenticated metrics endpoint validation input.
///
/// This briefly hosts a loopback-only listener, serves one authenticated
/// `GET /metrics` response from already-rendered local metric lines, and then
/// closes the listener. It does not expose public interfaces, export telemetry,
/// ship logs, send alerts, submit adapters, execute live actions, or approve
/// production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityMetricsEndpointValidationRequest {
    /// Stable local endpoint validation id.
    pub validation_id: String,
    /// Local export dry-run whose rendered metric lines are served.
    pub export_report: ObservabilityExportDryRunReport,
    /// Loopback host to bind.
    pub bind_host: String,
    /// Requested port. Use 0 for an ephemeral local port.
    pub requested_port: u16,
    /// HTTP method to send to the local one-shot handler.
    pub request_method: String,
    /// Request path to send to the local one-shot handler.
    pub request_path: String,
    /// Whether loopback-only binding is required.
    pub loopback_only_required: bool,
    /// Whether authentication is required before serving metrics.
    pub authentication_required: bool,
    /// Whether authorization is required before serving metrics.
    pub authorization_required: bool,
    /// Whether a non-secret bearer-token reference/header reference is present.
    pub bearer_token_reference_present: bool,
    /// Whether public network exposure was requested. Must remain false.
    pub public_network_exposure_requested: bool,
    /// Whether telemetry export was requested. Must remain false.
    pub telemetry_export_requested: bool,
    /// Whether outbound alert delivery was requested. Must remain false.
    pub outbound_alert_delivery_requested: bool,
}

/// Local one-shot metrics endpoint validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityMetricsEndpointValidationStatus {
    /// One local authenticated loopback scrape was served and the listener closed.
    ReadyForLocalReview,
    /// The request was blocked before serving or the one-shot local exchange failed.
    Blocked,
}

/// Local one-shot authenticated metrics endpoint validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityMetricsEndpointValidationReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local endpoint validation id.
    pub validation_id: String,
    /// Validation status.
    pub status: ObservabilityMetricsEndpointValidationStatus,
    /// Source snapshot id from the rendered export dry-run.
    pub snapshot_id: String,
    /// Loopback host that was bound.
    pub bind_host: String,
    /// Requested port.
    pub requested_port: u16,
    /// Actual local port assigned by the operating system.
    pub bound_port: Option<u16>,
    /// HTTP method sent to the local handler.
    pub request_method: String,
    /// Request path sent to the local handler.
    pub request_path: String,
    /// Whether the bind host was loopback-only.
    pub loopback_bind_validated: bool,
    /// Whether authentication was required.
    pub authentication_required: bool,
    /// Whether authorization was required.
    pub authorization_required: bool,
    /// Whether a non-secret token/header reference was present.
    pub bearer_token_reference_present: bool,
    /// Local HTTP status served by the one-shot handler.
    pub local_http_status_code: u16,
    /// Number of rendered metric lines served.
    pub response_metric_line_count: u64,
    /// Sanitized metric lines served by the local one-shot handler.
    pub response_metric_lines: Vec<String>,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a local one-shot metrics endpoint was opened.
    pub local_metrics_endpoint_started: bool,
    /// Whether exactly one local socket request was served.
    pub network_request_served: bool,
    /// Whether public network exposure occurred. Always false for ready reports.
    pub public_network_exposed: bool,
    /// Whether telemetry was exported. Always false.
    pub telemetry_exported: bool,
    /// Whether outbound alerts were sent. Always false.
    pub outbound_alerts_sent: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Sanitized local validation warnings.
    pub warnings: Vec<String>,
}

/// Bounded local metrics runtime probe request.
///
/// This serves multiple authenticated local `GET /metrics` scrapes on one
/// loopback listener, then shuts the listener down. It does not expose public
/// interfaces, export telemetry, ship logs, send alerts, or approve production
/// readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityMetricsRuntimeProbe {
    /// Stable local metrics runtime probe id.
    pub probe_id: String,
    /// Local export dry-run whose rendered metric lines are served.
    pub export_report: ObservabilityExportDryRunReport,
    /// Loopback host to bind.
    pub bind_host: String,
    /// Requested port. Use 0 for an ephemeral local port.
    pub requested_port: u16,
    /// Number of authenticated scrape requests to serve before shutdown.
    pub scrape_count: u32,
    /// Whether public network exposure was requested. Must remain false.
    pub public_network_exposure_requested: bool,
    /// Whether telemetry export was requested. Must remain false.
    pub telemetry_export_requested: bool,
    /// Whether outbound alert delivery was requested. Must remain false.
    pub outbound_alert_delivery_requested: bool,
}

/// Local bounded metrics runtime probe status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityMetricsRuntimeProbeStatus {
    /// Local bounded metrics runtime probe succeeded.
    ReadyForLocalReview,
    /// Runtime probe was blocked by missing controls or unsafe flags.
    Blocked,
}

/// Local bounded metrics runtime probe report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityMetricsRuntimeProbeReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local metrics runtime probe id.
    pub probe_id: String,
    /// Probe status.
    pub status: ObservabilityMetricsRuntimeProbeStatus,
    /// Source snapshot id from the rendered export dry-run.
    pub snapshot_id: String,
    /// Loopback host that was bound.
    pub bind_host: String,
    /// Requested port.
    pub requested_port: u16,
    /// Actual local port assigned by the operating system.
    pub bound_port: Option<u16>,
    /// Whether the bind host was loopback-only.
    pub loopback_bind_validated: bool,
    /// Number of scrape requests expected.
    pub expected_scrape_count: u32,
    /// Number of scrape requests served.
    pub served_scrape_count: u32,
    /// Whether every scrape returned HTTP 200.
    pub all_scrapes_returned_ok: bool,
    /// Number of metric lines returned per scrape.
    pub response_metric_line_count: u64,
    /// Whether all served metric lines matched the rendered export report.
    pub response_metric_lines_consistent: bool,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether the bounded local metrics runtime started.
    pub local_metrics_runtime_started: bool,
    /// Whether the bounded local metrics runtime shut down.
    pub local_metrics_runtime_shutdown: bool,
    /// Whether public network exposure occurred. Always false for ready reports.
    pub public_network_exposed: bool,
    /// Whether telemetry was exported. Always false.
    pub telemetry_exported: bool,
    /// Whether outbound alerts were sent. Always false.
    pub outbound_alerts_sent: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Sanitized local validation warnings.
    pub warnings: Vec<String>,
}

/// Local observability endpoint/exporter preflight input.
///
/// This models future metrics/export/alert runtime controls without starting a
/// metrics endpoint, binding sockets, exporting telemetry, shipping logs, or
/// delivering alerts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityEndpointPreflight {
    /// Stable local preflight id.
    pub preflight_id: String,
    /// Intended future metrics/export endpoint bind host.
    pub bind_host: String,
    /// Intended future metrics/export endpoint bind port.
    pub bind_port: u16,
    /// Whether the represented endpoint remains loopback-only.
    pub loopback_only_required: bool,
    /// Whether authentication would be required before endpoint access.
    pub authentication_required: bool,
    /// Whether authorization would be required before endpoint access.
    pub authorization_required: bool,
    /// Whether TLS or an approved local transport exception is required.
    pub transport_protection_required: bool,
    /// Whether telemetry redaction has been required before export/log shipping.
    pub redaction_required: bool,
    /// Whether alert route references are configured.
    pub alert_routes_configured: bool,
    /// Number of sanitized alert route references represented.
    pub alert_route_count: u32,
    /// Whether exporter backpressure/fail-closed behavior is required.
    pub exporter_backpressure_required: bool,
    /// Whether a metrics endpoint startup was requested. Must remain false here.
    pub metrics_endpoint_start_requested: bool,
    /// Whether public exposure was requested. Must remain false here.
    pub public_network_exposure_requested: bool,
    /// Whether telemetry export was requested. Must remain false here.
    pub telemetry_export_requested: bool,
    /// Whether outbound alert delivery was requested. Must remain false here.
    pub outbound_alert_delivery_requested: bool,
}

/// Local observability endpoint/exporter preflight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityEndpointPreflightStatus {
    /// Endpoint/exporter controls are locally coherent, but runtime remains disabled.
    ReadyForLocalReview,
    /// Required controls are missing or unsafe side-effect flags were requested.
    BlockedMissingControls,
}

/// Local observability endpoint/exporter preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityEndpointPreflightReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local preflight id.
    pub preflight_id: String,
    /// Preflight status.
    pub status: ObservabilityEndpointPreflightStatus,
    /// Whether the represented bind host is loopback-only.
    pub loopback_bind_validated: bool,
    /// Whether endpoint authentication was represented as required.
    pub authentication_required: bool,
    /// Whether endpoint authorization was represented as required.
    pub authorization_required: bool,
    /// Whether transport protection was represented as required.
    pub transport_protection_required: bool,
    /// Whether telemetry redaction was represented as required.
    pub redaction_required: bool,
    /// Number of sanitized alert route references represented.
    pub alert_route_count: u32,
    /// Whether alert routes were represented as configured.
    pub alert_routes_configured: bool,
    /// Whether exporter backpressure/fail-closed behavior was represented as required.
    pub exporter_backpressure_required: bool,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a metrics endpoint was started. Always false for this preflight.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Always false for this preflight.
    pub public_network_exposed: bool,
    /// Whether telemetry was exported. Always false for this preflight.
    pub telemetry_exported: bool,
    /// Whether outbound alerts were sent. Always false for this preflight.
    pub outbound_alerts_sent: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

/// Local loopback metrics endpoint bind validation input.
///
/// Unlike endpoint preflight, this probe briefly binds an ephemeral local
/// listener to prove the configured loopback host is bindable. It never exposes
/// a public interface, serves requests, exports telemetry, ships logs, sends
/// alerts, or starts a long-lived runtime endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityLoopbackBindValidationRequest {
    /// Stable local validation id.
    pub validation_id: String,
    /// Loopback host to bind.
    pub bind_host: String,
    /// Requested port. Use 0 for an ephemeral local port.
    pub requested_port: u16,
    /// Whether loopback-only binding is required.
    pub loopback_only_required: bool,
    /// Whether serving requests was requested. Must remain false here.
    pub serve_requests_requested: bool,
    /// Whether telemetry export was requested. Must remain false here.
    pub telemetry_export_requested: bool,
    /// Whether outbound alert delivery was requested. Must remain false here.
    pub outbound_alert_delivery_requested: bool,
}

/// Local loopback bind validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityLoopbackBindValidationStatus {
    /// Ephemeral loopback bind was validated for local review.
    ReadyForLocalReview,
    /// The request was blocked before binding or bind failed.
    Blocked,
}

/// Local loopback metrics endpoint bind validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityLoopbackBindValidationReport {
    /// Boundary version that produced this report.
    pub observability_runbook_version: String,
    /// Stable local validation id.
    pub validation_id: String,
    /// Validation status.
    pub status: ObservabilityLoopbackBindValidationStatus,
    /// Loopback host that was validated.
    pub bind_host: String,
    /// Requested port.
    pub requested_port: u16,
    /// Actual local port assigned by the operating system.
    pub bound_port: Option<u16>,
    /// Whether the bind host was loopback-only.
    pub loopback_bind_validated: bool,
    /// Whether a listener was opened and immediately closed.
    pub listener_opened_and_closed: bool,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a long-lived metrics endpoint was started. Always false here.
    pub metrics_endpoint_started: bool,
    /// Whether requests were served. Always false here.
    pub requests_served: bool,
    /// Whether public network exposure occurred. Always false here.
    pub public_network_exposed: bool,
    /// Whether telemetry was exported. Always false here.
    pub telemetry_exported: bool,
    /// Whether outbound alerts were sent. Always false here.
    pub outbound_alerts_sent: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local validation warnings.
    pub warnings: Vec<String>,
}

/// Health status for one runtime component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HealthStatus {
    /// Component health is not known.
    Unknown,
    /// Component is healthy.
    Healthy,
    /// Component is operating with reduced guarantees.
    Degraded,
    /// Component is unhealthy.
    Unhealthy,
    /// Component is in a critical state.
    Critical,
}

impl HealthStatus {
    /// Return whether this status requires operator attention.
    #[must_use]
    pub const fn requires_attention(self) -> bool {
        matches!(self, Self::Degraded | Self::Unhealthy | Self::Critical)
    }
}

/// Operator-facing severity for logs, warnings, and runbooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilitySeverity {
    /// Trace-level diagnostic event.
    Trace,
    /// Debug-level diagnostic event.
    Debug,
    /// Informational event.
    Info,
    /// Warning requiring review.
    Warning,
    /// Error requiring action.
    Error,
    /// Critical safety event requiring immediate action.
    Critical,
}

/// Deterministic metric kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetricKind {
    /// Monotonic counter value.
    Counter,
    /// Point-in-time gauge value.
    Gauge,
    /// Distribution bucket or latency value.
    Histogram,
}

/// Health status for one component.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentHealthStatus {
    /// Stable component name.
    pub component: String,
    /// Health status.
    pub status: HealthStatus,
    /// Operator-facing status detail.
    pub message: String,
    /// Last check timestamp in Unix epoch milliseconds.
    pub checked_at_ms: u64,
}

impl ComponentHealthStatus {
    /// Create one component health status.
    #[must_use]
    pub fn new(
        component: impl Into<String>,
        status: HealthStatus,
        message: impl Into<String>,
        checked_at_ms: u64,
    ) -> Self {
        Self {
            component: component.into(),
            status,
            message: message.into(),
            checked_at_ms,
        }
    }

    fn redacted(&self, max_chars: usize) -> (Self, bool) {
        let (component, component_redacted) =
            sanitize_observability_text(&self.component, max_chars);
        let (message, message_redacted) = sanitize_observability_text(&self.message, max_chars);
        (
            Self {
                component,
                status: self.status,
                message,
                checked_at_ms: self.checked_at_ms,
            },
            component_redacted || message_redacted,
        )
    }
}

/// Key/value field attached to a structured log event.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredLogField {
    /// Field key.
    pub key: String,
    /// Field value.
    pub value: String,
}

impl StructuredLogField {
    /// Create one structured log field.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    fn redacted(&self, max_chars: usize) -> (Self, bool) {
        let (key, key_redacted) = sanitize_observability_text(&self.key, max_chars);
        let (value, value_redacted) = sanitize_observability_text(&self.value, max_chars);
        (Self { key, value }, key_redacted || value_redacted)
    }
}

/// Local structured log event model.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredLogEvent {
    /// Stable event identifier.
    pub id: String,
    /// Event severity.
    pub severity: ObservabilitySeverity,
    /// Event target/component.
    pub target: String,
    /// Event message.
    pub message: String,
    /// Event fields.
    pub fields: Vec<StructuredLogField>,
    /// Creation timestamp in Unix epoch milliseconds.
    pub created_at_ms: u64,
}

impl StructuredLogEvent {
    /// Create one structured log event.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        severity: ObservabilitySeverity,
        target: impl Into<String>,
        message: impl Into<String>,
        fields: Vec<StructuredLogField>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            target: target.into(),
            message: message.into(),
            fields,
            created_at_ms,
        }
    }

    fn redacted(&self, max_chars: usize, max_fields: usize) -> (Self, bool) {
        let mut redacted_any = false;
        let (id, id_redacted) = sanitize_observability_text(&self.id, max_chars);
        let (target, target_redacted) = sanitize_observability_text(&self.target, max_chars);
        let (message, message_redacted) = sanitize_observability_text(&self.message, max_chars);
        redacted_any |= id_redacted || target_redacted || message_redacted;

        let fields = self
            .fields
            .iter()
            .take(max_fields)
            .map(|field| {
                let (redacted_field, field_redacted) = field.redacted(max_chars);
                redacted_any |= field_redacted;
                redacted_field
            })
            .collect();

        (
            Self {
                id,
                severity: self.severity,
                target,
                message,
                fields,
                created_at_ms: self.created_at_ms,
            },
            redacted_any,
        )
    }
}

/// Metric label model.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetricLabel {
    /// Label key.
    pub key: String,
    /// Label value.
    pub value: String,
}

impl MetricLabel {
    /// Create one metric label.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    fn redacted(&self, max_chars: usize) -> (Self, bool) {
        let (key, key_redacted) = sanitize_observability_text(&self.key, max_chars);
        let (value, value_redacted) = sanitize_observability_text(&self.value, max_chars);
        (Self { key, value }, key_redacted || value_redacted)
    }
}

/// Local deterministic metric sample.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSample {
    /// Stable metric name.
    pub name: String,
    /// Metric kind.
    pub kind: MetricKind,
    /// Integer micro-unit value to avoid nondeterministic floating-point formatting.
    pub value_microunits: i64,
    /// Unit name, for example `count`, `milliseconds`, or `quote-microunits`.
    pub unit: String,
    /// Deterministic labels.
    pub labels: Vec<MetricLabel>,
    /// Sample timestamp in Unix epoch milliseconds.
    pub sampled_at_ms: u64,
}

impl MetricSample {
    /// Create one metric sample.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        kind: MetricKind,
        value_microunits: i64,
        unit: impl Into<String>,
        labels: Vec<MetricLabel>,
        sampled_at_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            value_microunits,
            unit: unit.into(),
            labels,
            sampled_at_ms,
        }
    }

    fn redacted(&self, max_chars: usize, max_labels: usize) -> (Self, bool) {
        let mut redacted_any = false;
        let (name, name_redacted) = sanitize_observability_text(&self.name, max_chars);
        let (unit, unit_redacted) = sanitize_observability_text(&self.unit, max_chars);
        redacted_any |= name_redacted || unit_redacted;

        let labels = self
            .labels
            .iter()
            .take(max_labels)
            .map(|label| {
                let (redacted_label, label_redacted) = label.redacted(max_chars);
                redacted_any |= label_redacted;
                redacted_label
            })
            .collect();

        (
            Self {
                name,
                kind: self.kind,
                value_microunits: self.value_microunits,
                unit,
                labels,
                sampled_at_ms: self.sampled_at_ms,
            },
            redacted_any,
        )
    }
}

/// One operator runbook step.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunbookStep {
    /// Deterministic step number.
    pub ordinal: u16,
    /// Step title.
    pub title: String,
    /// Operator instruction.
    pub instruction: String,
}

impl RunbookStep {
    /// Create one runbook step.
    #[must_use]
    pub fn new(ordinal: u16, title: impl Into<String>, instruction: impl Into<String>) -> Self {
        Self {
            ordinal,
            title: title.into(),
            instruction: instruction.into(),
        }
    }

    fn redacted(&self, max_chars: usize) -> (Self, bool) {
        let (title, title_redacted) = sanitize_observability_text(&self.title, max_chars);
        let (instruction, instruction_redacted) =
            sanitize_observability_text(&self.instruction, max_chars);
        (
            Self {
                ordinal: self.ordinal,
                title,
                instruction,
            },
            title_redacted || instruction_redacted,
        )
    }
}

/// Operator runbook model.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Runbook {
    /// Stable runbook identifier.
    pub id: String,
    /// Runbook title.
    pub title: String,
    /// Runbook severity.
    pub severity: ObservabilitySeverity,
    /// Trigger description.
    pub trigger: String,
    /// Operator steps.
    pub steps: Vec<RunbookStep>,
}

impl Runbook {
    /// Create one runbook.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        severity: ObservabilitySeverity,
        trigger: impl Into<String>,
        steps: Vec<RunbookStep>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            severity,
            trigger: trigger.into(),
            steps,
        }
    }

    fn redacted(&self, max_chars: usize, max_steps: usize) -> (Self, bool) {
        let mut redacted_any = false;
        let (id, id_redacted) = sanitize_observability_text(&self.id, max_chars);
        let (title, title_redacted) = sanitize_observability_text(&self.title, max_chars);
        let (trigger, trigger_redacted) = sanitize_observability_text(&self.trigger, max_chars);
        redacted_any |= id_redacted || title_redacted || trigger_redacted;

        let steps = self
            .steps
            .iter()
            .take(max_steps)
            .map(|step| {
                let (redacted_step, step_redacted) = step.redacted(max_chars);
                redacted_any |= step_redacted;
                redacted_step
            })
            .collect();

        (
            Self {
                id,
                title,
                severity: self.severity,
                trigger,
                steps,
            },
            redacted_any,
        )
    }
}

/// Local observability snapshot supplied by future runtime components.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilitySnapshot {
    /// Stable snapshot identifier. Must not contain secret material.
    pub snapshot_id: String,
    /// Snapshot creation timestamp in Unix epoch milliseconds.
    pub generated_at_ms: u64,
    /// Component health states.
    pub components: Vec<ComponentHealthStatus>,
    /// Structured log events.
    pub logs: Vec<StructuredLogEvent>,
    /// Metric samples.
    pub metrics: Vec<MetricSample>,
    /// Operator runbooks.
    pub runbooks: Vec<Runbook>,
    /// Snapshot warnings.
    pub warnings: Vec<String>,
}

impl ObservabilitySnapshot {
    /// Validate observability snapshot shape before local collection.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id("snapshot", &self.snapshot_id, &mut violations);

        let mut component_names = BTreeSet::new();
        for component in &self.components {
            validate_name("component", &component.component, &mut violations);
            if !component_names.insert(component.component.to_ascii_lowercase()) {
                violations.push(ObservabilityViolation::new_owned(
                    "OBSERVABILITY_COMPONENT_DUPLICATE",
                    format!("component {} is duplicated", component.component),
                ));
            }
        }

        let mut runbook_ids = BTreeSet::new();
        for runbook in &self.runbooks {
            validate_id("runbook", &runbook.id, &mut violations);
            if !runbook_ids.insert(runbook.id.to_ascii_lowercase()) {
                violations.push(ObservabilityViolation::new_owned(
                    "OBSERVABILITY_RUNBOOK_DUPLICATE",
                    format!("runbook {} is duplicated", runbook.id),
                ));
            }
        }

        for log in &self.logs {
            validate_id("log", &log.id, &mut violations);
            validate_name("log target", &log.target, &mut violations);
        }

        for metric in &self.metrics {
            validate_name("metric", &metric.name, &mut violations);
            validate_name("metric unit", &metric.unit, &mut violations);
        }

        finish_validation(violations)
    }

    /// Calculate the worst health status represented by this snapshot.
    #[must_use]
    pub fn overall_health(&self) -> HealthStatus {
        self.components
            .iter()
            .map(|component| component.status)
            .max()
            .unwrap_or(HealthStatus::Unknown)
    }
}

/// Local observability collection request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityCollectionRequest {
    /// Boundary configuration.
    pub config: ObservabilityBoundaryConfig,
    /// Runtime snapshot to collect.
    pub snapshot: ObservabilitySnapshot,
    /// Observability collection access context.
    pub access: ObservabilityAccessContext,
    /// Optional operator-facing label. Must not contain secret material.
    pub operator_label: Option<String>,
    /// Collection timestamp in Unix epoch milliseconds.
    pub collected_at_ms: u64,
}

/// Observability collection source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityAccessSource {
    /// In-process local collection path.
    LocalCollection,
    /// Future metrics endpoint scrape source.
    MetricsEndpoint,
    /// Future exporter session source.
    ExporterSession,
    /// Future outbound alert delivery source.
    AlertDelivery,
}

/// Observability access authorization status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilityAccessAuthorizationStatus {
    /// Local in-process collection is authorized.
    AuthorizedLocalCollection,
    /// Local collection is disabled by config.
    RejectedLocalCollectionDisabled,
    /// External exporter or alert source is not enabled in this phase.
    RejectedExternalSession,
}

/// Non-secret observability access context.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityAccessContext {
    /// Access source.
    pub source: ObservabilityAccessSource,
    /// Stable non-secret collector/operator label.
    pub collector_label: Option<String>,
}

impl ObservabilityAccessContext {
    /// Local in-process observability access context.
    #[must_use]
    pub fn local_collection(collector_label: Option<String>) -> Self {
        Self {
            source: ObservabilityAccessSource::LocalCollection,
            collector_label,
        }
    }

    fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        if let Some(label) = &self.collector_label {
            if contains_secret_like_text(label) {
                violations.push(ObservabilityViolation::new(
                    "OBSERVABILITY_ACCESS_LABEL_SECRET_LIKE",
                    "observability access label looks like secret material",
                ));
            }
        }
        finish_validation(violations)
    }
}

/// Local observability collection record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityRecord {
    /// Boundary version that produced this record.
    pub observability_runbook_version: String,
    /// Observability snapshot identifier.
    pub snapshot_id: String,
    /// Collection timestamp in Unix epoch milliseconds.
    pub collected_at_ms: u64,
    /// Overall health derived from component statuses.
    pub overall_health: HealthStatus,
    /// Sanitized operator label.
    pub operator_label: Option<String>,
    /// Whether observability collection access was locally authorized.
    pub access_authorized: bool,
    /// Observability access authorization status.
    pub access_authorization_status: ObservabilityAccessAuthorizationStatus,
    /// Sanitized component health states.
    pub components: Vec<ComponentHealthStatus>,
    /// Sanitized structured log events.
    pub logs: Vec<StructuredLogEvent>,
    /// Sanitized metric samples.
    pub metrics: Vec<MetricSample>,
    /// Sanitized runbooks.
    pub runbooks: Vec<Runbook>,
    /// Sanitized warnings.
    pub warnings: Vec<String>,
    /// Whether a metrics endpoint was started. Phase 14 always returns false.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Phase 14 always returns false.
    pub public_network_exposed: bool,
    /// Whether outbound alerts were sent. Phase 14 always returns false.
    pub outbound_alerts_sent: bool,
    /// Whether secret-like text was redacted before record creation.
    pub secret_redaction_applied: bool,
}

/// Local runtime failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeFailureKind {
    /// Panic-like local failure report.
    Panic,
    /// Crash or abrupt-exit local failure report.
    Crash,
    /// Health-check failure report.
    HealthCheckFailure,
    /// Operator-visible validation failure report.
    ValidationFailure,
}

/// Local runtime failure capture request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeFailureCaptureRequest {
    /// Stable failure id.
    pub failure_id: String,
    /// Component that observed or owns the failure.
    pub component: String,
    /// Failure category.
    pub kind: RuntimeFailureKind,
    /// Operator-facing severity.
    pub severity: ObservabilitySeverity,
    /// Non-secret failure summary.
    pub summary: String,
    /// Non-secret failure detail.
    pub detail: String,
    /// Boundary configuration.
    pub config: ObservabilityBoundaryConfig,
    /// Observability collection access context.
    pub access: ObservabilityAccessContext,
    /// Capture timestamp in Unix epoch milliseconds.
    pub captured_at_ms: u64,
}

/// Local runtime failure capture record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeFailureCaptureRecord {
    /// Boundary version that produced this record.
    pub observability_runbook_version: String,
    /// Stable failure id.
    pub failure_id: String,
    /// Sanitized component name.
    pub component: String,
    /// Failure category.
    pub kind: RuntimeFailureKind,
    /// Operator-facing severity.
    pub severity: ObservabilitySeverity,
    /// Sanitized failure summary.
    pub summary: String,
    /// Sanitized failure detail.
    pub detail: String,
    /// Capture timestamp in Unix epoch milliseconds.
    pub captured_at_ms: u64,
    /// Whether observability collection access was locally authorized.
    pub access_authorized: bool,
    /// Observability access authorization status.
    pub access_authorization_status: ObservabilityAccessAuthorizationStatus,
    /// Whether secret-like text was redacted before record creation.
    pub secret_redaction_applied: bool,
    /// Whether a metrics endpoint was started. Always false for this local boundary.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Always false for this local boundary.
    pub public_network_exposed: bool,
    /// Whether outbound alerts were sent. Always false for this local boundary.
    pub outbound_alerts_sent: bool,
    /// Whether any external adapter was submitted to. Always false for this local boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false for this local boundary.
    pub live_execution_performed: bool,
    /// Whether this local record approves production readiness. Always false.
    pub production_ready: bool,
}

/// Result of a scoped local panic-hook failure-capture probe.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalPanicHookCaptureReport {
    /// Whether a scoped panic hook was installed for the local probe.
    pub hook_installed: bool,
    /// Whether the previous panic hook was restored before returning.
    pub hook_restored: bool,
    /// Whether the scoped operation panicked.
    pub panic_observed: bool,
    /// Sanitized runtime failure record produced from the panic, if any.
    pub failure_record: Option<RuntimeFailureCaptureRecord>,
    /// Audit journal sequence written for the failure capture, if any.
    pub audit_sequence: Option<u64>,
    /// State checkpoint key written for the failure capture, if any.
    pub checkpoint_key: Option<String>,
    /// Whether a metrics endpoint was started. Always false for this local boundary.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Always false for this local boundary.
    pub public_network_exposed: bool,
    /// Whether outbound alerts were sent. Always false for this local boundary.
    pub outbound_alerts_sent: bool,
    /// Whether any external adapter was submitted to. Always false for this local boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false for this local boundary.
    pub live_execution_performed: bool,
    /// Whether this local report approves production readiness. Always false.
    pub production_ready: bool,
}

/// Request to install a local runtime panic hook for audit/state failure capture.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePanicHookInstallationRequest {
    /// Stable failure id to use for the first captured panic.
    pub failure_id: String,
    /// Runtime component protected by the hook.
    pub component: String,
    /// Operator-facing severity.
    pub severity: ObservabilitySeverity,
    /// Non-secret failure summary.
    pub summary: String,
    /// Non-secret base failure detail.
    pub detail: String,
    /// Local audit journal path to append the captured failure record.
    pub audit_path: PathBuf,
    /// Local SQLite WAL state path to checkpoint the captured failure record.
    pub state_path: PathBuf,
    /// Boundary configuration.
    pub config: ObservabilityBoundaryConfig,
    /// Observability collection access context.
    pub access: ObservabilityAccessContext,
    /// Capture timestamp in Unix epoch milliseconds.
    pub captured_at_ms: u64,
}

/// Result of installing a local runtime panic hook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePanicHookInstallationReport {
    /// Whether the local runtime hook was installed.
    pub hook_installed: bool,
    /// Failure id that will be used by the first captured panic.
    pub failure_id: String,
    /// Whether metrics endpoints were started. Always false for this local boundary.
    pub metrics_endpoint_started: bool,
    /// Whether public network exposure occurred. Always false for this local boundary.
    pub public_network_exposed: bool,
    /// Whether outbound alerts were sent. Always false for this local boundary.
    pub outbound_alerts_sent: bool,
    /// Whether any external adapter was submitted to. Always false for this local boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false for this local boundary.
    pub live_execution_performed: bool,
    /// Whether this local report approves production readiness. Always false.
    pub production_ready: bool,
}

/// Request to validate a scoped local tracing subscriber capture.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalTracingSubscriberValidationRequest {
    /// Stable validation identifier.
    pub validation_id: String,
    /// Local subscriber label.
    pub subscriber_label: String,
    /// Structured event to emit through the scoped subscriber.
    pub event: StructuredLogEvent,
    /// Boundary configuration.
    pub config: ObservabilityBoundaryConfig,
    /// Observability collection access context.
    pub access: ObservabilityAccessContext,
    /// Whether local in-process capture is required.
    pub local_capture_required: bool,
    /// Whether redaction is required before emission.
    pub redaction_required: bool,
    /// Whether a global subscriber install is requested. Must be false.
    pub global_install_requested: bool,
    /// Whether telemetry export is requested. Must be false.
    pub telemetry_export_requested: bool,
    /// Whether outbound alert delivery is requested. Must be false.
    pub outbound_alert_delivery_requested: bool,
    /// Whether public network exposure is requested. Must be false.
    pub public_network_exposure_requested: bool,
    /// Whether live execution is requested. Must be false.
    pub live_execution_requested: bool,
    /// Validation timestamp in Unix epoch milliseconds.
    pub captured_at_ms: u64,
}

/// Local tracing subscriber validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalTracingSubscriberValidationStatus {
    /// A scoped local subscriber captured a sanitized event.
    ReadyForLocalReview,
    /// The requested subscriber behavior would require unsafe or missing controls.
    Blocked,
}

/// Report for a scoped local tracing subscriber capture.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalTracingSubscriberValidationReport {
    /// Observability boundary version.
    pub observability_runbook_version: String,
    /// Stable validation identifier.
    pub validation_id: String,
    /// Local subscriber label.
    pub subscriber_label: String,
    /// Validation status.
    pub status: LocalTracingSubscriberValidationStatus,
    /// Sanitized structured event id.
    pub event_id: String,
    /// Sanitized structured event target.
    pub event_target: String,
    /// Whether a scoped subscriber was installed for the local capture.
    pub scoped_subscriber_installed: bool,
    /// Whether the scoped subscriber captured at least one event.
    pub event_captured: bool,
    /// Number of emitted events captured by the local writer.
    pub captured_event_count: u32,
    /// Sanitized captured output excerpt.
    pub captured_output_excerpt: String,
    /// Number of missing or unsafe controls.
    pub missing_control_count: u32,
    /// Whether secret-like text was redacted before emission.
    pub secret_redaction_applied: bool,
    /// Whether access was locally authorized.
    pub access_authorized: bool,
    /// Access authorization status.
    pub access_authorization_status: ObservabilityAccessAuthorizationStatus,
    /// Whether a global subscriber was installed. Always false for this boundary.
    pub global_subscriber_installed: bool,
    /// Whether telemetry was exported. Always false for this boundary.
    pub telemetry_exported: bool,
    /// Whether outbound alerts were sent. Always false for this boundary.
    pub outbound_alerts_sent: bool,
    /// Whether public network exposure occurred. Always false for this boundary.
    pub public_network_exposed: bool,
    /// Whether live execution was performed. Always false for this boundary.
    pub live_execution_performed: bool,
    /// Whether this local report approves production readiness. Always false.
    pub production_ready: bool,
}

#[allow(deprecated)]
type LocalRuntimePanicHook = Box<dyn Fn(&panic::PanicInfo<'_>) + Sync + Send + 'static>;

/// Guard that restores the previous panic hook when dropped.
pub struct LocalRuntimePanicHookGuard {
    previous_hook: Option<LocalRuntimePanicHook>,
    report: RuntimePanicHookInstallationReport,
    panic_captured: Arc<Mutex<bool>>,
    last_capture_error: Arc<Mutex<Option<String>>>,
}

impl LocalRuntimePanicHookGuard {
    /// Installation report for this hook guard.
    #[must_use]
    pub const fn report(&self) -> &RuntimePanicHookInstallationReport {
        &self.report
    }

    /// Whether the installed hook captured at least one panic.
    #[must_use]
    pub fn panic_captured(&self) -> bool {
        self.panic_captured.lock().is_ok_and(|captured| *captured)
    }

    /// Sanitized capture error, if the hook could not persist the panic record.
    #[must_use]
    pub fn last_capture_error(&self) -> Option<String> {
        self.last_capture_error
            .lock()
            .ok()
            .and_then(|error| error.clone())
    }
}

impl Drop for LocalRuntimePanicHookGuard {
    fn drop(&mut self) {
        if let Some(previous_hook) = self.previous_hook.take() {
            panic::set_hook(previous_hook);
        }
    }
}

/// Observability collector boundary.
pub trait ObservabilityCollector {
    /// Collect an observability request into a local, non-network record.
    fn collect(
        &self,
        request: ObservabilityCollectionRequest,
    ) -> Result<ObservabilityRecord, ObservabilityError>;
}

/// Deterministic local-only observability collector.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicObservabilityCollector;

impl ObservabilityCollector for DeterministicObservabilityCollector {
    fn collect(
        &self,
        request: ObservabilityCollectionRequest,
    ) -> Result<ObservabilityRecord, ObservabilityError> {
        request.config.validate()?;
        request.snapshot.validate()?;
        request.access.validate()?;

        if !request.config.local_collection_enabled {
            return Err(ObservabilityError::ValidationFailed {
                violations: vec![ObservabilityViolation::new(
                    "OBSERVABILITY_LOCAL_COLLECTION_DISABLED",
                    "local observability collection is disabled",
                )],
            });
        }

        let access_decision = authorize_observability_access(&request.access, &request.config);
        if !access_decision.access_authorized {
            return Err(ObservabilityError::ValidationFailed {
                violations: vec![ObservabilityViolation::new_owned(
                    "OBSERVABILITY_ACCESS_DENIED",
                    access_decision.reason,
                )],
            });
        }

        let mut redaction_applied = false;
        let max_chars = 512;
        let (snapshot_id, snapshot_redacted) =
            sanitize_observability_text(&request.snapshot.snapshot_id, max_chars);
        redaction_applied |= snapshot_redacted;

        let operator_label = request.operator_label.as_ref().map(|label| {
            let (sanitized, redacted) = sanitize_observability_text(label, max_chars);
            redaction_applied |= redacted;
            sanitized
        });

        let components = request
            .snapshot
            .components
            .iter()
            .map(|component| {
                let (redacted_component, component_redacted) = component.redacted(max_chars);
                redaction_applied |= component_redacted;
                redacted_component
            })
            .collect();

        let logs = request
            .snapshot
            .logs
            .iter()
            .map(|log| {
                let (redacted_log, log_redacted) =
                    log.redacted(max_chars, request.config.max_log_fields);
                redaction_applied |= log_redacted;
                redacted_log
            })
            .collect();

        let metrics = request
            .snapshot
            .metrics
            .iter()
            .map(|metric| {
                let (redacted_metric, metric_redacted) =
                    metric.redacted(max_chars, request.config.max_metric_labels);
                redaction_applied |= metric_redacted;
                redacted_metric
            })
            .collect();

        let runbooks = request
            .snapshot
            .runbooks
            .iter()
            .map(|runbook| {
                let (redacted_runbook, runbook_redacted) =
                    runbook.redacted(max_chars, request.config.max_runbook_steps);
                redaction_applied |= runbook_redacted;
                redacted_runbook
            })
            .collect();

        let warnings = request
            .snapshot
            .warnings
            .iter()
            .map(|warning| {
                let (sanitized, redacted) = sanitize_observability_text(warning, max_chars);
                redaction_applied |= redacted;
                sanitized
            })
            .collect();

        Ok(ObservabilityRecord {
            observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
            snapshot_id,
            collected_at_ms: request.collected_at_ms,
            overall_health: request.snapshot.overall_health(),
            operator_label,
            access_authorized: access_decision.access_authorized,
            access_authorization_status: access_decision.status,
            components,
            logs,
            metrics,
            runbooks,
            warnings,
            metrics_endpoint_started: false,
            public_network_exposed: false,
            outbound_alerts_sent: false,
            secret_redaction_applied: redaction_applied,
        })
    }
}

impl ObservabilityRecord {
    /// Validate local observability record invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id("observability snapshot", &self.snapshot_id, &mut violations);
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.metrics_endpoint_started {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_STARTED",
                "observability records must not start a metrics endpoint",
            ));
        }
        if self.public_network_exposed {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PUBLIC_NETWORK_EXPOSED",
                "observability records must not expose public network bindings",
            ));
        }
        if self.outbound_alerts_sent {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_OUTBOUND_ALERTS_SENT",
                "observability records must not send outbound alerts",
            ));
        }
        if !self.access_authorized {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_RECORD_ACCESS_NOT_AUTHORIZED",
                "observability records must be locally access-authorized",
            ));
        }
        let mut component_names = BTreeSet::new();
        for component in &self.components {
            validate_name("component", &component.component, &mut violations);
            if !component_names.insert(component.component.to_ascii_lowercase()) {
                violations.push(ObservabilityViolation::new_owned(
                    "OBSERVABILITY_COMPONENT_DUPLICATE",
                    format!("component {} is duplicated", component.component),
                ));
            }
        }
        for log in &self.logs {
            validate_id("log", &log.id, &mut violations);
            validate_name("log target", &log.target, &mut violations);
        }
        for metric in &self.metrics {
            validate_name("metric", &metric.name, &mut violations);
            validate_name("metric unit", &metric.unit, &mut violations);
        }
        let mut runbook_ids = BTreeSet::new();
        for runbook in &self.runbooks {
            validate_id("runbook", &runbook.id, &mut violations);
            if !runbook_ids.insert(runbook.id.to_ascii_lowercase()) {
                violations.push(ObservabilityViolation::new_owned(
                    "OBSERVABILITY_RUNBOOK_DUPLICATE",
                    format!("runbook {} is duplicated", runbook.id),
                ));
            }
        }
        for warning in &self.warnings {
            if contains_secret_like_text(warning) {
                violations.push(ObservabilityViolation::new(
                    "OBSERVABILITY_WARNING_SECRET_LIKE",
                    "observability warning still looks like it may contain secret material",
                ));
            }
        }
        finish_validation(violations)
    }
}

impl ObservabilityOperationsPolicy {
    /// Validate local observability operations review input.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability operations review",
            &self.review_id,
            &mut violations,
        );
        if self.log_retention_required && self.retention_days == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_RETENTION_DAYS_ZERO",
                "observability retention_days must be positive when retention is required",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityOperationsReviewReport {
    /// Validate local observability operations review invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability operations review",
            &self.review_id,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.metrics_endpoint_started
            || self.public_network_exposed
            || self.outbound_alerts_sent
            || self.telemetry_exported
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_OPERATIONS_REVIEW_SIDE_EFFECT",
                "observability operations reviews must not start endpoints, expose public networks, send alerts, or export telemetry",
            ));
        }
        if self.production_ready {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_OPERATIONS_REVIEW_PRODUCTION_READY",
                "observability operations reviews must not approve production readiness",
            ));
        }
        match self.status {
            ObservabilityOperationsReviewStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.log_retention_required
                    || self.retention_days == 0
                    || !self.redaction_required
                    || !self.alert_routing_required
                    || self.alert_route_count == 0
                    || !self.incident_runbook_required
                    || self.incident_runbook_count == 0
                    || !self.loopback_or_authenticated_endpoint_required
                    || !self.audit_state_preflight_required
                    || !self.exporter_kill_switch_required
                    || !self.alert_authorization_required
                    || !self.rate_limit_backpressure_required
                    || !self.retry_backoff_required
                    || !self.no_secret_telemetry_required
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_OPERATIONS_REVIEW_READY_MISMATCH",
                        "ready observability operations reviews require retention, redaction, alert routing, incident runbooks, endpoint policy, future runtime controls, and zero missing controls",
                    ));
                }
            }
            ObservabilityOperationsReviewStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_OPERATIONS_REVIEW_BLOCKED_MISMATCH",
                        "blocked observability operations reviews require at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl ObservabilityLogRetentionExecutionRequest {
    /// Validate local observability log retention execution input.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.operations_review.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "observability log retention execution",
            &self.execution_id,
            &mut violations,
        );
        if self.operations_review.status != ObservabilityOperationsReviewStatus::ReadyForLocalReview
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOG_RETENTION_REVIEW_NOT_READY",
                "observability log retention execution requires a ready operations review",
            ));
        }
        if !self.operations_review.log_retention_required
            || self.operations_review.retention_days == 0
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOG_RETENTION_CONTROL_MISSING",
                "observability log retention execution requires reviewed retention controls",
            ));
        }
        if !self.local_sandbox_only
            || self.production_log_paths_requested
            || self.service_manager_action_requested
            || self.external_log_shipping_requested
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOG_RETENTION_SIDE_EFFECT_REQUESTED",
                "observability log retention execution must be local-sandbox only and must not touch production logs, service managers, or external log shipping",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityLogRetentionExecutionReport {
    /// Validate local observability log retention execution output.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability log retention execution",
            &self.execution_id,
            &mut violations,
        );
        validate_id(
            "observability operations review",
            &self.review_id,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if !self.operations_review_ready {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOG_RETENTION_REVIEW_NOT_READY",
                "observability log retention execution report requires a ready operations review",
            ));
        }
        if self.out_of_workspace_path_touched
            || self.production_log_paths_touched
            || self.service_manager_action_performed
            || self.external_log_shipping_performed
            || self.live_network_used
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOG_RETENTION_FORBIDDEN_SIDE_EFFECT",
                "observability log retention execution must not touch paths outside the sandbox, production logs, service managers, external log shipping, live networks, or approve production readiness",
            ));
        }
        for path in self
            .retained_archives
            .iter()
            .chain(self.expired_archives_deleted.iter())
        {
            if contains_secret_like_text(path) {
                violations.push(ObservabilityViolation::new(
                    "OBSERVABILITY_LOG_RETENTION_PATH_SECRET_LIKE",
                    "observability log retention path looks like secret material",
                ));
            }
        }
        finish_validation(violations)
    }
}

impl ObservabilityExportDryRunRequest {
    /// Validate local export/alert dry-run input.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.record.validate()?;
        self.operations_review.validate()?;
        let mut violations = Vec::new();
        if self.rendered_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_TIMESTAMP_ZERO",
                "observability export dry-run timestamp must be non-zero",
            ));
        }
        if self.operations_review.status != ObservabilityOperationsReviewStatus::ReadyForLocalReview
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_REVIEW_NOT_READY",
                "observability export dry runs require a ready local operations review",
            ));
        }
        if self.operations_review.review_id.trim().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_REVIEW_EMPTY",
                "observability export dry-run review id must be non-empty",
            ));
        }
        if u64::try_from(self.alert_route_references.len()).unwrap_or(u64::MAX)
            != u64::from(self.operations_review.alert_route_count)
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_ROUTE_COUNT_MISMATCH",
                "observability export dry-run alert route references must match the operations review route count",
            ));
        }
        for route in &self.alert_route_references {
            validate_id(
                "observability alert route reference",
                route,
                &mut violations,
            );
        }
        finish_validation(violations)
    }
}

impl ObservabilityExportDryRunReport {
    /// Validate local export/alert dry-run invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id("observability snapshot", &self.snapshot_id, &mut violations);
        validate_id(
            "observability operations review",
            &self.review_id,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.rendered_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_TIMESTAMP_ZERO",
                "observability export dry-run timestamp must be non-zero",
            ));
        }
        if self.prometheus_metric_lines.is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_METRICS_EMPTY",
                "observability export dry runs require at least one rendered local metric line",
            ));
        }
        if self.alert_route_count == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_ALERT_ROUTES_EMPTY",
                "observability export dry runs require sanitized alert route references",
            ));
        }
        if !self.loopback_or_authenticated_endpoint_required {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_ENDPOINT_POLICY_MISSING",
                "observability export dry runs require loopback or authenticated endpoint policy",
            ));
        }
        if self.metrics_endpoint_started
            || self.public_network_exposed
            || self.outbound_alerts_sent
            || self.telemetry_exported
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_EXPORT_DRY_RUN_SIDE_EFFECT",
                "observability export dry runs must not start endpoints, expose networks, send alerts, export telemetry, execute live actions, or approve production readiness",
            ));
        }
        for line in &self.prometheus_metric_lines {
            if contains_secret_like_text(line) {
                violations.push(ObservabilityViolation::new(
                    "OBSERVABILITY_EXPORT_DRY_RUN_SECRET_LIKE_LINE",
                    "observability export dry-run metric line still looks like secret material",
                ));
            }
        }
        finish_validation(violations)
    }
}

impl ObservabilityAlertRouteDispatchRequest {
    /// Validate local alert-route dispatch bridge input.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.export_report.validate()?;
        if let Err(error) = self.notification_dispatch.validate() {
            return Err(ObservabilityError::ValidationFailed {
                violations: vec![ObservabilityViolation::new_owned(
                    "OBSERVABILITY_ALERT_ROUTE_NOTIFICATION_INVALID",
                    format!("notification dispatch failed local validation: {error}"),
                )],
            });
        }

        let mut violations = Vec::new();
        validate_id(
            "observability alert-route dispatch review",
            &self.dispatch_review_id,
            &mut violations,
        );
        validate_id(
            "observability alert route reference",
            &self.alert_route_reference,
            &mut violations,
        );
        if self.reviewed_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_TIMESTAMP_ZERO",
                "observability alert-route dispatch review timestamp must be non-zero",
            ));
        }
        if !self.local_dispatch_required {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_NOT_REQUIRED",
                "observability alert-route dispatch reviews require local communications dispatch",
            ));
        }
        if self.outbound_alert_delivery_requested {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_OUTBOUND_DELIVERY_REQUESTED",
                "observability alert-route dispatch reviews must not request outbound alert delivery",
            ));
        }
        if self.export_report.alert_route_count == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_NO_ALERTS",
                "observability alert-route dispatch reviews require local alert route references",
            ));
        }
        if self.notification_dispatch.status != NotificationDispatchStatus::RecordedLocally {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_NOT_LOCAL",
                "observability alert-route dispatch must be recorded by the local communications boundary",
            ));
        }
        if self.notification_dispatch.outbound_network_used {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_OUTBOUND_NETWORK",
                "observability alert-route dispatch must not use outbound network delivery",
            ));
        }

        finish_validation(violations)
    }
}

impl ObservabilityAlertRouteDispatchReport {
    /// Validate local alert-route dispatch bridge invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability alert-route dispatch review",
            &self.dispatch_review_id,
            &mut violations,
        );
        validate_id("observability snapshot", &self.snapshot_id, &mut violations);
        validate_id(
            "observability operations review",
            &self.review_id,
            &mut violations,
        );
        validate_id(
            "observability alert route reference",
            &self.alert_route_reference,
            &mut violations,
        );
        validate_id(
            "notification dispatch",
            &self.notification_dispatch_id,
            &mut violations,
        );
        validate_id("notification", &self.notification_id, &mut violations);

        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.reviewed_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_TIMESTAMP_ZERO",
                "observability alert-route dispatch review timestamp must be non-zero",
            ));
        }
        if self.status != ObservabilityAlertRouteDispatchStatus::ReadyForLocalReview {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_BLOCKED",
                "observability alert-route dispatch report must be ready for local review",
            ));
        }
        if !self.local_dispatch_required || self.recorded_local_channel_count == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_LOCAL_RECORD_MISSING",
                "observability alert-route dispatch requires at least one local channel record",
            ));
        }
        if self.notification_dispatch_status != NotificationDispatchStatus::RecordedLocally {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_NOTIFICATION_NOT_LOCAL",
                "observability alert-route notification dispatch must be recorded locally",
            ));
        }
        if self.outbound_alerts_sent
            || self.outbound_network_used
            || self.telemetry_exported
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ALERT_ROUTE_DISPATCH_SIDE_EFFECT",
                "observability alert-route dispatch must not send alerts, use outbound network, export telemetry, execute live actions, or approve production readiness",
            ));
        }

        finish_validation(violations)
    }
}

impl ObservabilityMetricsScrapePreflightRequest {
    /// Validate local metrics scrape preflight input invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.export_report.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "observability metrics scrape preflight",
            &self.scrape_id,
            &mut violations,
        );
        validate_name(
            "observability metrics scrape method",
            &self.request_method,
            &mut violations,
        );
        validate_name(
            "observability metrics scrape path",
            &self.request_path,
            &mut violations,
        );
        validate_name(
            "observability metrics scrape source host",
            &self.source_host,
            &mut violations,
        );
        if contains_secret_like_text(&self.source_host) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_SCRAPE_SOURCE_SECRET_LIKE",
                "observability metrics scrape source host looks like secret material",
            ));
        }
        if self.metrics_endpoint_start_requested
            || self.public_network_exposure_requested
            || self.telemetry_export_requested
            || self.outbound_alert_delivery_requested
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_SCRAPE_SIDE_EFFECT_REQUESTED",
                "metrics scrape preflights must not request endpoint startup, public exposure, telemetry export, or outbound alert delivery",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityMetricsScrapePreflightReport {
    /// Validate local metrics scrape preflight report invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability metrics scrape preflight",
            &self.scrape_id,
            &mut violations,
        );
        validate_id("observability snapshot", &self.snapshot_id, &mut violations);
        validate_name(
            "observability metrics scrape method",
            &self.request_method,
            &mut violations,
        );
        validate_name(
            "observability metrics scrape path",
            &self.request_path,
            &mut violations,
        );
        validate_name(
            "observability metrics scrape source host",
            &self.source_host,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.metrics_endpoint_started
            || self.network_request_served
            || self.public_network_exposed
            || self.telemetry_exported
            || self.outbound_alerts_sent
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_SCRAPE_SIDE_EFFECT",
                "metrics scrape preflights must not start endpoints, serve socket requests, expose networks, export telemetry, send alerts, or approve production readiness",
            ));
        }
        if usize::try_from(self.response_metric_line_count) != Ok(self.response_metric_lines.len())
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_SCRAPE_LINE_COUNT_MISMATCH",
                "metrics scrape line count must match returned local metric lines",
            ));
        }
        for line in &self.response_metric_lines {
            if contains_secret_like_text(line) {
                violations.push(ObservabilityViolation::new(
                    "OBSERVABILITY_METRICS_SCRAPE_SECRET_LIKE_LINE",
                    "metrics scrape preflight line still looks like secret material",
                ));
            }
        }
        match self.status {
            ObservabilityMetricsScrapePreflightStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || self.local_http_status_code != 200
                    || self.request_method != "GET"
                    || self.request_path != "/metrics"
                    || !self.loopback_source_validated
                    || !self.authentication_required
                    || !self.authorization_required
                    || !self.bearer_token_reference_present
                    || self.response_metric_lines.is_empty()
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_METRICS_SCRAPE_READY_MISMATCH",
                        "ready metrics scrape preflights require GET /metrics, loopback source, auth, authorization, token reference, local metric lines, and zero missing controls",
                    ));
                }
            }
            ObservabilityMetricsScrapePreflightStatus::Blocked => {
                if self.missing_control_count == 0 || self.local_http_status_code == 200 {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_METRICS_SCRAPE_BLOCKED_MISMATCH",
                        "blocked metrics scrape preflights require at least one missing control and a non-200 local status",
                    ));
                }
            }
        }
        for warning in &self.warnings {
            validate_name(
                "observability metrics scrape warning",
                warning,
                &mut violations,
            );
        }
        finish_validation(violations)
    }
}

impl ObservabilityMetricsEndpointValidationRequest {
    /// Validate local one-shot endpoint validation input invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.export_report.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "observability metrics endpoint validation",
            &self.validation_id,
            &mut violations,
        );
        validate_name(
            "observability metrics endpoint bind host",
            &self.bind_host,
            &mut violations,
        );
        validate_name(
            "observability metrics endpoint method",
            &self.request_method,
            &mut violations,
        );
        validate_name(
            "observability metrics endpoint path",
            &self.request_path,
            &mut violations,
        );
        if contains_secret_like_text(&self.bind_host) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_BIND_SECRET_LIKE",
                "observability metrics endpoint bind host looks like secret material",
            ));
        }
        if !self.loopback_only_required {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_LOOPBACK_REQUIRED",
                "local metrics endpoint validation requires loopback-only binding",
            ));
        }
        if self.public_network_exposure_requested
            || self.telemetry_export_requested
            || self.outbound_alert_delivery_requested
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_SIDE_EFFECT_REQUESTED",
                "local metrics endpoint validation must not request public exposure, telemetry export, or outbound alert delivery",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityMetricsEndpointValidationReport {
    /// Validate local one-shot endpoint validation report invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability metrics endpoint validation",
            &self.validation_id,
            &mut violations,
        );
        validate_id("observability snapshot", &self.snapshot_id, &mut violations);
        validate_name(
            "observability metrics endpoint bind host",
            &self.bind_host,
            &mut violations,
        );
        validate_name(
            "observability metrics endpoint method",
            &self.request_method,
            &mut violations,
        );
        validate_name(
            "observability metrics endpoint path",
            &self.request_path,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.public_network_exposed
            || self.telemetry_exported
            || self.outbound_alerts_sent
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_FORBIDDEN_SIDE_EFFECT",
                "local metrics endpoint validation must not expose public networks, export telemetry, send alerts, or approve production readiness",
            ));
        }
        if usize::try_from(self.response_metric_line_count) != Ok(self.response_metric_lines.len())
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_ENDPOINT_LINE_COUNT_MISMATCH",
                "served metrics endpoint line count must match returned local metric lines",
            ));
        }
        for line in &self.response_metric_lines {
            if contains_secret_like_text(line) {
                violations.push(ObservabilityViolation::new(
                    "OBSERVABILITY_METRICS_ENDPOINT_SECRET_LIKE_LINE",
                    "served metrics endpoint line still looks like secret material",
                ));
            }
        }
        match self.status {
            ObservabilityMetricsEndpointValidationStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || self.local_http_status_code != 200
                    || self.request_method != "GET"
                    || self.request_path != "/metrics"
                    || self.bound_port.is_none()
                    || !self.loopback_bind_validated
                    || !self.authentication_required
                    || !self.authorization_required
                    || !self.bearer_token_reference_present
                    || !self.local_metrics_endpoint_started
                    || !self.network_request_served
                    || self.response_metric_lines.is_empty()
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_METRICS_ENDPOINT_READY_MISMATCH",
                        "ready metrics endpoint validations require one authenticated loopback GET /metrics response, local metric lines, and zero missing controls",
                    ));
                }
            }
            ObservabilityMetricsEndpointValidationStatus::Blocked => {
                if self.missing_control_count == 0 || self.local_http_status_code == 200 {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_METRICS_ENDPOINT_BLOCKED_MISMATCH",
                        "blocked metrics endpoint validations require at least one missing control and a non-200 local status",
                    ));
                }
            }
        }
        for warning in &self.warnings {
            validate_name(
                "observability metrics endpoint warning",
                warning,
                &mut violations,
            );
        }
        finish_validation(violations)
    }
}

impl ObservabilityMetricsRuntimeProbe {
    /// Validate bounded local metrics runtime probe input invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.export_report.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "observability metrics runtime probe",
            &self.probe_id,
            &mut violations,
        );
        validate_name(
            "observability metrics runtime bind host",
            &self.bind_host,
            &mut violations,
        );
        if !is_loopback_host(&self.bind_host) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_RUNTIME_BIND_NOT_LOOPBACK",
                "observability metrics runtime probe requires numeric loopback binding",
            ));
        }
        if self.scrape_count == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_RUNTIME_SCRAPE_COUNT_ZERO",
                "observability metrics runtime probe requires at least one scrape",
            ));
        }
        if self.public_network_exposure_requested
            || self.telemetry_export_requested
            || self.outbound_alert_delivery_requested
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_RUNTIME_SIDE_EFFECT_REQUESTED",
                "observability metrics runtime probe must not request public exposure, telemetry export, or outbound alert delivery",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityMetricsRuntimeProbeReport {
    /// Validate bounded local metrics runtime probe report invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability metrics runtime probe",
            &self.probe_id,
            &mut violations,
        );
        validate_id("observability snapshot", &self.snapshot_id, &mut violations);
        validate_name(
            "observability metrics runtime bind host",
            &self.bind_host,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if !is_loopback_host(&self.bind_host) || !self.loopback_bind_validated {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_RUNTIME_BIND_NOT_VALIDATED",
                "observability metrics runtime report requires validated numeric loopback binding",
            ));
        }
        if self.public_network_exposed
            || self.telemetry_exported
            || self.outbound_alerts_sent
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_METRICS_RUNTIME_FORBIDDEN_SIDE_EFFECT",
                "observability metrics runtime report must not expose public networks, export telemetry, send alerts, or approve production readiness",
            ));
        }
        match self.status {
            ObservabilityMetricsRuntimeProbeStatus::ReadyForLocalReview => {
                if self.expected_scrape_count == 0
                    || self.served_scrape_count != self.expected_scrape_count
                    || !self.all_scrapes_returned_ok
                    || self.response_metric_line_count == 0
                    || !self.response_metric_lines_consistent
                    || self.missing_control_count != 0
                    || !self.local_metrics_runtime_started
                    || !self.local_metrics_runtime_shutdown
                    || self.bound_port.is_none()
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_METRICS_RUNTIME_READY_MISMATCH",
                        "ready observability metrics runtime reports require all expected local scrapes to return 200 with consistent metric lines and clean shutdown",
                    ));
                }
            }
            ObservabilityMetricsRuntimeProbeStatus::Blocked => {
                if self.expected_scrape_count > 0
                    && self.served_scrape_count == self.expected_scrape_count
                    && self.all_scrapes_returned_ok
                    && self.response_metric_line_count > 0
                    && self.response_metric_lines_consistent
                    && self.missing_control_count == 0
                    && self.local_metrics_runtime_started
                    && self.local_metrics_runtime_shutdown
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_METRICS_RUNTIME_BLOCKED_MISMATCH",
                        "blocked observability metrics runtime reports must have at least one missing control or failed scrape",
                    ));
                }
            }
        }
        for warning in &self.warnings {
            validate_name(
                "observability metrics runtime warning",
                warning,
                &mut violations,
            );
        }
        finish_validation(violations)
    }
}

impl ObservabilityEndpointPreflight {
    /// Validate local endpoint/exporter preflight input invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability endpoint preflight",
            &self.preflight_id,
            &mut violations,
        );
        if contains_secret_like_text(&self.bind_host) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ENDPOINT_PREFLIGHT_BIND_SECRET_LIKE",
                "observability endpoint preflight bind host looks like secret material",
            ));
        }
        if self.alert_routes_configured && self.alert_route_count == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ENDPOINT_PREFLIGHT_ALERT_ROUTES_ZERO",
                "configured alert routes require at least one route reference",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityEndpointPreflightReport {
    /// Validate local endpoint/exporter preflight report invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability endpoint preflight",
            &self.preflight_id,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.metrics_endpoint_started
            || self.public_network_exposed
            || self.telemetry_exported
            || self.outbound_alerts_sent
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_ENDPOINT_PREFLIGHT_SIDE_EFFECT",
                "observability endpoint preflights must not start endpoints, expose networks, export telemetry, send alerts, or approve production readiness",
            ));
        }
        match self.status {
            ObservabilityEndpointPreflightStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.loopback_bind_validated
                    || !self.authentication_required
                    || !self.authorization_required
                    || !self.transport_protection_required
                    || !self.redaction_required
                    || !self.alert_routes_configured
                    || self.alert_route_count == 0
                    || !self.exporter_backpressure_required
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_ENDPOINT_PREFLIGHT_READY_MISMATCH",
                        "ready endpoint preflights require loopback binding, auth, transport protection, redaction, alert routes, exporter backpressure, and zero missing controls",
                    ));
                }
            }
            ObservabilityEndpointPreflightStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_ENDPOINT_PREFLIGHT_BLOCKED_MISMATCH",
                        "blocked endpoint preflights require at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl ObservabilityLoopbackBindValidationRequest {
    /// Validate local loopback bind probe input invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability loopback bind validation",
            &self.validation_id,
            &mut violations,
        );
        if contains_secret_like_text(&self.bind_host) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOOPBACK_BIND_HOST_SECRET_LIKE",
                "observability loopback bind host looks like secret material",
            ));
        }
        if !self.loopback_only_required {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOOPBACK_BIND_REQUIRED",
                "observability loopback bind validation requires loopback-only binding",
            ));
        }
        if self.serve_requests_requested
            || self.telemetry_export_requested
            || self.outbound_alert_delivery_requested
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOOPBACK_BIND_SIDE_EFFECT_REQUESTED",
                "observability loopback bind validation must not serve requests, export telemetry, or deliver alerts",
            ));
        }
        finish_validation(violations)
    }
}

impl ObservabilityLoopbackBindValidationReport {
    /// Validate local loopback bind report invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability loopback bind validation",
            &self.validation_id,
            &mut violations,
        );
        validate_name(
            "observability loopback bind host",
            &self.bind_host,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if self.metrics_endpoint_started
            || self.requests_served
            || self.public_network_exposed
            || self.telemetry_exported
            || self.outbound_alerts_sent
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_LOOPBACK_BIND_SIDE_EFFECT",
                "observability loopback bind validation must not start long-lived endpoints, serve requests, expose public networks, export telemetry, send alerts, or approve production readiness",
            ));
        }
        match self.status {
            ObservabilityLoopbackBindValidationStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.loopback_bind_validated
                    || !self.listener_opened_and_closed
                    || self.bound_port.is_none()
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_LOOPBACK_BIND_READY_MISMATCH",
                        "ready loopback bind validation requires loopback bind success, an assigned local port, listener closure, and zero missing controls",
                    ));
                }
            }
            ObservabilityLoopbackBindValidationStatus::Blocked => {
                if self.missing_control_count == 0 {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_LOOPBACK_BIND_BLOCKED_MISMATCH",
                        "blocked loopback bind validation requires at least one finding",
                    ));
                }
            }
        }
        for warning in &self.warnings {
            validate_name(
                "observability loopback bind warning",
                warning,
                &mut violations,
            );
        }
        finish_validation(violations)
    }
}

/// Review local observability retention and alert-route controls without side effects.
pub fn review_observability_operations(
    policy: &ObservabilityOperationsPolicy,
) -> Result<ObservabilityOperationsReviewReport, ObservabilityError> {
    policy.validate()?;
    let mut missing_control_count = 0_u32;
    for missing in [
        !policy.log_retention_required,
        policy.retention_days == 0,
        !policy.redaction_required,
        !policy.alert_routing_required,
        policy.alert_route_count == 0,
        !policy.incident_runbook_required,
        policy.incident_runbook_count == 0,
        !policy.loopback_or_authenticated_endpoint_required,
        !policy.audit_state_preflight_required,
        !policy.exporter_kill_switch_required,
        !policy.alert_authorization_required,
        !policy.rate_limit_backpressure_required,
        !policy.retry_backoff_required,
        !policy.no_secret_telemetry_required,
        policy.metrics_endpoint_requested,
        policy.outbound_alert_delivery_requested,
        policy.telemetry_export_requested,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let status = if missing_control_count == 0 {
        ObservabilityOperationsReviewStatus::ReadyForLocalReview
    } else {
        ObservabilityOperationsReviewStatus::BlockedMissingControls
    };
    let report = ObservabilityOperationsReviewReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        review_id: policy.review_id.clone(),
        status,
        log_retention_required: policy.log_retention_required,
        retention_days: policy.retention_days,
        redaction_required: policy.redaction_required,
        alert_routing_required: policy.alert_routing_required,
        alert_route_count: policy.alert_route_count,
        incident_runbook_required: policy.incident_runbook_required,
        incident_runbook_count: policy.incident_runbook_count,
        loopback_or_authenticated_endpoint_required: policy
            .loopback_or_authenticated_endpoint_required,
        audit_state_preflight_required: policy.audit_state_preflight_required,
        exporter_kill_switch_required: policy.exporter_kill_switch_required,
        alert_authorization_required: policy.alert_authorization_required,
        rate_limit_backpressure_required: policy.rate_limit_backpressure_required,
        retry_backoff_required: policy.retry_backoff_required,
        no_secret_telemetry_required: policy.no_secret_telemetry_required,
        missing_control_count,
        metrics_endpoint_started: false,
        public_network_exposed: false,
        outbound_alerts_sent: false,
        telemetry_exported: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Execute observability log retention/rotation inside an explicit local sandbox only.
///
/// This reuses the audit retention executor for workspace containment and
/// mutation safety. It does not touch production log paths, contact service
/// managers, ship logs externally, use live networks, or claim production
/// readiness.
pub fn execute_local_observability_log_retention(
    request: &ObservabilityLogRetentionExecutionRequest,
) -> Result<ObservabilityLogRetentionExecutionReport, ObservabilityError> {
    request.validate()?;
    let execution = execute_local_audit_retention(&request.retention_request)?;
    let report = observability_log_retention_report(
        &request.execution_id,
        &request.operations_review,
        &execution,
    );
    report.validate()?;
    Ok(report)
}

fn observability_log_retention_report(
    execution_id: &str,
    operations_review: &ObservabilityOperationsReviewReport,
    execution: &AuditRetentionExecutionReport,
) -> ObservabilityLogRetentionExecutionReport {
    ObservabilityLogRetentionExecutionReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        execution_id: execution_id.to_owned(),
        review_id: operations_review.review_id.clone(),
        operations_review_ready: operations_review.status
            == ObservabilityOperationsReviewStatus::ReadyForLocalReview,
        rotate_active_requested: execution.rotate_active_requested,
        new_active_created: execution.new_active_created,
        retained_archives: execution.retained_archives.clone(),
        expired_archives_deleted: execution.expired_archives_deleted.clone(),
        deleted_file_count: execution.deleted_file_count,
        sandbox_filesystem_mutated: execution.filesystem_mutated,
        sandbox_deletion_performed: execution.deletion_performed,
        out_of_workspace_path_touched: execution.out_of_workspace_path_touched,
        production_log_paths_touched: false,
        service_manager_action_performed: false,
        external_log_shipping_performed: false,
        live_network_used: execution.live_network_used,
        production_ready: false,
    }
}

/// Preflight future observability endpoint/exporter controls without side effects.
pub fn preflight_observability_endpoint(
    preflight: &ObservabilityEndpointPreflight,
) -> Result<ObservabilityEndpointPreflightReport, ObservabilityError> {
    preflight.validate()?;
    let loopback_bind_validated =
        preflight.loopback_only_required && is_loopback_host(&preflight.bind_host);
    let alert_routes_ready = preflight.alert_routes_configured && preflight.alert_route_count > 0;

    let mut missing_control_count = 0_u32;
    for missing in [
        !loopback_bind_validated,
        !preflight.authentication_required,
        !preflight.authorization_required,
        !preflight.transport_protection_required,
        !preflight.redaction_required,
        !alert_routes_ready,
        !preflight.exporter_backpressure_required,
        preflight.metrics_endpoint_start_requested,
        preflight.public_network_exposure_requested,
        preflight.telemetry_export_requested,
        preflight.outbound_alert_delivery_requested,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }

    let status = if missing_control_count == 0 {
        ObservabilityEndpointPreflightStatus::ReadyForLocalReview
    } else {
        ObservabilityEndpointPreflightStatus::BlockedMissingControls
    };
    let report = ObservabilityEndpointPreflightReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        preflight_id: preflight.preflight_id.clone(),
        status,
        loopback_bind_validated,
        authentication_required: preflight.authentication_required,
        authorization_required: preflight.authorization_required,
        transport_protection_required: preflight.transport_protection_required,
        redaction_required: preflight.redaction_required,
        alert_route_count: preflight.alert_route_count,
        alert_routes_configured: preflight.alert_routes_configured,
        exporter_backpressure_required: preflight.exporter_backpressure_required,
        missing_control_count,
        metrics_endpoint_started: false,
        public_network_exposed: false,
        telemetry_exported: false,
        outbound_alerts_sent: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Validate an ephemeral loopback-only metrics endpoint bind locally.
///
/// The listener is opened only long enough to capture the assigned local port
/// and is dropped before the report is returned. No request is served and no
/// telemetry/export/alert side effects are performed.
pub fn validate_observability_loopback_bind(
    request: &ObservabilityLoopbackBindValidationRequest,
) -> Result<ObservabilityLoopbackBindValidationReport, ObservabilityError> {
    request.validate()?;

    let mut missing_control_count = 0_u32;
    let mut warnings = Vec::new();
    let bind_host = request.bind_host.trim().to_owned();
    let loopback_ip = parse_loopback_ip(&bind_host);
    let loopback_bind_validated = loopback_ip.is_some();

    if !loopback_bind_validated {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("bind host is not a numeric loopback address".to_owned());
    }

    let mut bound_port = None;
    let mut listener_opened_and_closed = false;
    if let Some(ip) = loopback_ip {
        match TcpListener::bind((ip, request.requested_port)) {
            Ok(listener) => {
                bound_port = listener.local_addr().ok().map(|address| address.port());
                drop(listener);
                listener_opened_and_closed = bound_port.is_some();
                if !listener_opened_and_closed {
                    missing_control_count = missing_control_count.saturating_add(1);
                    warnings
                        .push("loopback listener opened without a readable local port".to_owned());
                }
            }
            Err(error) => {
                missing_control_count = missing_control_count.saturating_add(1);
                warnings.push(format!("loopback bind failed: {}", error.kind()));
            }
        }
    }

    let status = if missing_control_count == 0 {
        ObservabilityLoopbackBindValidationStatus::ReadyForLocalReview
    } else {
        ObservabilityLoopbackBindValidationStatus::Blocked
    };
    let report = ObservabilityLoopbackBindValidationReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        validation_id: request.validation_id.clone(),
        status,
        bind_host,
        requested_port: request.requested_port,
        bound_port,
        loopback_bind_validated,
        listener_opened_and_closed,
        missing_control_count,
        metrics_endpoint_started: false,
        requests_served: false,
        public_network_exposed: false,
        telemetry_exported: false,
        outbound_alerts_sent: false,
        production_ready: false,
        warnings,
    };
    report.validate()?;
    Ok(report)
}

/// Render a local metrics/export and alert-route dry run without network side effects.
pub fn render_observability_export_dry_run(
    request: ObservabilityExportDryRunRequest,
) -> Result<ObservabilityExportDryRunReport, ObservabilityError> {
    request.validate()?;
    let prometheus_metric_lines = request
        .record
        .metrics
        .iter()
        .map(render_prometheus_metric_line)
        .collect::<Vec<_>>();
    let alert_dry_run_count = count_observability_alert_dry_runs(&request.record);
    let report = ObservabilityExportDryRunReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        snapshot_id: request.record.snapshot_id,
        review_id: request.operations_review.review_id,
        rendered_at_ms: request.rendered_at_ms,
        prometheus_metric_lines,
        log_event_count: u64::try_from(request.record.logs.len()).map_err(|_| {
            ObservabilityError::StateStoreFailed {
                reason: "observability log event count overflowed".to_owned(),
            }
        })?,
        runbook_count: u64::try_from(request.record.runbooks.len()).map_err(|_| {
            ObservabilityError::StateStoreFailed {
                reason: "observability runbook count overflowed".to_owned(),
            }
        })?,
        alert_route_count: u64::try_from(request.alert_route_references.len()).map_err(|_| {
            ObservabilityError::StateStoreFailed {
                reason: "observability alert route count overflowed".to_owned(),
            }
        })?,
        alert_dry_run_count,
        loopback_or_authenticated_endpoint_required: request
            .operations_review
            .loopback_or_authenticated_endpoint_required,
        metrics_endpoint_started: false,
        public_network_exposed: false,
        outbound_alerts_sent: false,
        telemetry_exported: false,
        live_execution_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Record an observability alert-route dispatch through the local communications boundary.
///
/// This accepts only deterministic local notification dispatch records. It does
/// not send outbound alerts, call messaging platforms, export telemetry, execute
/// live actions, or approve production readiness.
pub fn record_observability_alert_route_dispatch(
    request: ObservabilityAlertRouteDispatchRequest,
) -> Result<ObservabilityAlertRouteDispatchReport, ObservabilityError> {
    request.validate()?;
    let recorded_local_channel_count = request
        .notification_dispatch
        .channels
        .iter()
        .filter(|channel| channel.status == NotificationChannelDispatchStatus::RecordedLocally)
        .count();
    let blocked_channel_count = request
        .notification_dispatch
        .channels
        .len()
        .saturating_sub(recorded_local_channel_count);
    let report = ObservabilityAlertRouteDispatchReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        dispatch_review_id: request.dispatch_review_id,
        status: ObservabilityAlertRouteDispatchStatus::ReadyForLocalReview,
        snapshot_id: request.export_report.snapshot_id,
        review_id: request.export_report.review_id,
        alert_route_reference: request.alert_route_reference,
        notification_dispatch_id: request.notification_dispatch.id,
        notification_id: request.notification_dispatch.notification_id,
        notification_dispatch_status: request.notification_dispatch.status,
        recorded_local_channel_count: u64::try_from(recorded_local_channel_count).map_err(
            |_| ObservabilityError::StateStoreFailed {
                reason: "observability alert-route local channel count overflowed".to_owned(),
            },
        )?,
        blocked_channel_count: u64::try_from(blocked_channel_count).map_err(|_| {
            ObservabilityError::StateStoreFailed {
                reason: "observability alert-route blocked channel count overflowed".to_owned(),
            }
        })?,
        local_dispatch_required: request.local_dispatch_required,
        outbound_alerts_sent: false,
        outbound_network_used: request.notification_dispatch.outbound_network_used,
        telemetry_exported: false,
        live_execution_performed: false,
        production_ready: false,
        reviewed_at_ms: request.reviewed_at_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate one local authenticated metrics scrape against rendered metric lines.
///
/// This is an in-process preflight only. It does not bind a socket, start a
/// metrics endpoint, serve a network request, export telemetry, or deliver
/// alerts.
pub fn preflight_observability_metrics_scrape(
    request: ObservabilityMetricsScrapePreflightRequest,
) -> Result<ObservabilityMetricsScrapePreflightReport, ObservabilityError> {
    request.validate()?;

    let mut missing_control_count = 0_u32;
    let mut warnings = Vec::new();
    let method = request.request_method.trim().to_ascii_uppercase();
    let path = request.request_path.trim().to_owned();
    let source_host = request.source_host.trim().to_owned();
    let loopback_source_validated = parse_loopback_ip(&source_host).is_some();

    if method != "GET" {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics scrape method is not GET".to_owned());
    }
    if path != "/metrics" {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics scrape path is not /metrics".to_owned());
    }
    if !loopback_source_validated {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics scrape source is not a numeric loopback address".to_owned());
    }
    if !request.authentication_required {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics scrape authentication is not required".to_owned());
    }
    if !request.authorization_required {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics scrape authorization is not required".to_owned());
    }
    if !request.bearer_token_reference_present {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics scrape token reference is missing".to_owned());
    }

    let ready = missing_control_count == 0;
    let response_metric_lines = if ready {
        request.export_report.prometheus_metric_lines.clone()
    } else {
        Vec::new()
    };
    let response_metric_line_count = u64::try_from(response_metric_lines.len()).map_err(|_| {
        ObservabilityError::StateStoreFailed {
            reason: "observability metrics scrape line count overflowed".to_owned(),
        }
    })?;
    let report = ObservabilityMetricsScrapePreflightReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        scrape_id: request.scrape_id,
        status: if ready {
            ObservabilityMetricsScrapePreflightStatus::ReadyForLocalReview
        } else {
            ObservabilityMetricsScrapePreflightStatus::Blocked
        },
        snapshot_id: request.export_report.snapshot_id,
        request_method: method,
        request_path: path,
        source_host,
        loopback_source_validated,
        authentication_required: request.authentication_required,
        authorization_required: request.authorization_required,
        bearer_token_reference_present: request.bearer_token_reference_present,
        local_http_status_code: if ready { 200 } else { 403 },
        response_metric_line_count,
        response_metric_lines,
        missing_control_count,
        metrics_endpoint_started: false,
        network_request_served: false,
        public_network_exposed: false,
        telemetry_exported: false,
        outbound_alerts_sent: false,
        production_ready: false,
        warnings,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a local one-shot authenticated metrics endpoint over loopback.
///
/// This opens a short-lived local listener, serves one `GET /metrics` response
/// from already-rendered metric lines, then closes the listener. It does not
/// expose public interfaces, export telemetry, ship logs, send alerts, submit
/// adapters, execute live actions, or approve production readiness.
pub fn validate_observability_metrics_endpoint(
    request: ObservabilityMetricsEndpointValidationRequest,
) -> Result<ObservabilityMetricsEndpointValidationReport, ObservabilityError> {
    request.validate()?;

    let mut missing_control_count = 0_u32;
    let mut warnings = Vec::new();
    let bind_host = request.bind_host.trim().to_owned();
    let method = request.request_method.trim().to_ascii_uppercase();
    let path = request.request_path.trim().to_owned();
    let loopback_ip = parse_loopback_ip(&bind_host);
    let loopback_bind_validated = loopback_ip.is_some();

    if !loopback_bind_validated {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics endpoint bind host is not numeric loopback".to_owned());
    }
    if method != "GET" {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics endpoint request method is not GET".to_owned());
    }
    if path != "/metrics" {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics endpoint request path is not /metrics".to_owned());
    }
    if !request.authentication_required {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics endpoint authentication is not required".to_owned());
    }
    if !request.authorization_required {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics endpoint authorization is not required".to_owned());
    }
    if !request.bearer_token_reference_present {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics endpoint bearer-token reference is missing".to_owned());
    }

    let mut bound_port = None;
    let mut local_metrics_endpoint_started = false;
    let mut network_request_served = false;
    let mut local_http_status_code = 403_u16;
    let mut response_metric_lines = Vec::new();

    if missing_control_count == 0 {
        let Some(ip) = loopback_ip else {
            missing_control_count = missing_control_count.saturating_add(1);
            warnings.push("metrics endpoint loopback address disappeared".to_owned());
            return build_observability_metrics_endpoint_report(
                request,
                bind_host,
                method,
                path,
                loopback_bind_validated,
                None,
                false,
                false,
                403,
                Vec::new(),
                missing_control_count,
                warnings,
            );
        };
        match serve_one_local_metrics_request(
            ip,
            request.requested_port,
            &method,
            &path,
            &request.export_report.prometheus_metric_lines,
        ) {
            Ok(exchange) => {
                bound_port = Some(exchange.bound_port);
                local_metrics_endpoint_started = true;
                network_request_served = exchange.network_request_served;
                local_http_status_code = exchange.local_http_status_code;
                response_metric_lines = exchange.response_metric_lines;
                if local_http_status_code != 200 || !network_request_served {
                    missing_control_count = missing_control_count.saturating_add(1);
                    warnings.push("metrics endpoint local scrape did not return 200".to_owned());
                }
            }
            Err(error) => {
                missing_control_count = missing_control_count.saturating_add(1);
                warnings.push(format!("metrics endpoint local scrape failed: {error}"));
            }
        }
    }

    build_observability_metrics_endpoint_report(
        request,
        bind_host,
        method,
        path,
        loopback_bind_validated,
        bound_port,
        local_metrics_endpoint_started,
        network_request_served,
        local_http_status_code,
        response_metric_lines,
        missing_control_count,
        warnings,
    )
}

/// Validate a bounded local authenticated metrics runtime over loopback.
///
/// This opens one short-lived local listener, serves multiple authenticated
/// `GET /metrics` responses from already-rendered metric lines, then closes the
/// listener. It does not expose public interfaces, export telemetry, ship logs,
/// send alerts, submit adapters, execute live actions, or approve production
/// readiness.
pub fn validate_observability_metrics_runtime_probe(
    probe: ObservabilityMetricsRuntimeProbe,
) -> Result<ObservabilityMetricsRuntimeProbeReport, ObservabilityError> {
    probe.validate()?;
    let mut missing_control_count = 0_u32;
    let mut warnings = Vec::new();
    let bind_host = probe.bind_host.trim().to_owned();
    let loopback_ip = parse_loopback_ip(&bind_host);
    let loopback_bind_validated = loopback_ip.is_some();
    if !loopback_bind_validated {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics runtime bind host is not numeric loopback".to_owned());
    }

    let mut bound_port = None;
    let mut served_scrape_count = 0_u32;
    let mut all_scrapes_returned_ok = false;
    let mut response_metric_lines_consistent = false;
    let mut local_metrics_runtime_started = false;
    let mut local_metrics_runtime_shutdown = false;
    let response_metric_line_count =
        u64::try_from(probe.export_report.prometheus_metric_lines.len()).map_err(|_| {
            ObservabilityError::StateStoreFailed {
                reason: "metrics runtime line count overflowed".to_owned(),
            }
        })?;
    if response_metric_line_count == 0 {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("metrics runtime has no rendered metric lines".to_owned());
    }

    if missing_control_count == 0 {
        if let Some(ip) = loopback_ip {
            match serve_bounded_local_metrics_runtime(
                ip,
                probe.requested_port,
                &probe.export_report.prometheus_metric_lines,
                probe.scrape_count,
            ) {
                Ok(exchange) => {
                    bound_port = Some(exchange.bound_port);
                    served_scrape_count = exchange.served_scrape_count;
                    all_scrapes_returned_ok = exchange.all_scrapes_returned_ok;
                    response_metric_lines_consistent = exchange.response_metric_lines_consistent;
                    local_metrics_runtime_started = true;
                    local_metrics_runtime_shutdown = exchange.local_metrics_runtime_shutdown;
                    if served_scrape_count != probe.scrape_count
                        || !all_scrapes_returned_ok
                        || !response_metric_lines_consistent
                        || !local_metrics_runtime_shutdown
                    {
                        missing_control_count = missing_control_count.saturating_add(1);
                        warnings.push(
                            "metrics runtime did not serve all expected scrapes cleanly".to_owned(),
                        );
                    }
                }
                Err(error) => {
                    missing_control_count = missing_control_count.saturating_add(1);
                    warnings.push(format!("metrics runtime local scrape failed: {error}"));
                }
            }
        }
    }

    let report = ObservabilityMetricsRuntimeProbeReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        probe_id: probe.probe_id,
        status: if missing_control_count == 0 {
            ObservabilityMetricsRuntimeProbeStatus::ReadyForLocalReview
        } else {
            ObservabilityMetricsRuntimeProbeStatus::Blocked
        },
        snapshot_id: probe.export_report.snapshot_id,
        bind_host,
        requested_port: probe.requested_port,
        bound_port,
        loopback_bind_validated,
        expected_scrape_count: probe.scrape_count,
        served_scrape_count,
        all_scrapes_returned_ok,
        response_metric_line_count,
        response_metric_lines_consistent,
        missing_control_count,
        local_metrics_runtime_started,
        local_metrics_runtime_shutdown,
        public_network_exposed: false,
        telemetry_exported: false,
        outbound_alerts_sent: false,
        production_ready: false,
        warnings,
    };
    report.validate()?;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn build_observability_metrics_endpoint_report(
    request: ObservabilityMetricsEndpointValidationRequest,
    bind_host: String,
    method: String,
    path: String,
    loopback_bind_validated: bool,
    bound_port: Option<u16>,
    local_metrics_endpoint_started: bool,
    network_request_served: bool,
    local_http_status_code: u16,
    response_metric_lines: Vec<String>,
    missing_control_count: u32,
    warnings: Vec<String>,
) -> Result<ObservabilityMetricsEndpointValidationReport, ObservabilityError> {
    let response_metric_line_count = u64::try_from(response_metric_lines.len()).map_err(|_| {
        ObservabilityError::StateStoreFailed {
            reason: "observability metrics endpoint line count overflowed".to_owned(),
        }
    })?;
    let report = ObservabilityMetricsEndpointValidationReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        validation_id: request.validation_id,
        status: if missing_control_count == 0 {
            ObservabilityMetricsEndpointValidationStatus::ReadyForLocalReview
        } else {
            ObservabilityMetricsEndpointValidationStatus::Blocked
        },
        snapshot_id: request.export_report.snapshot_id,
        bind_host,
        requested_port: request.requested_port,
        bound_port,
        request_method: method,
        request_path: path,
        loopback_bind_validated,
        authentication_required: request.authentication_required,
        authorization_required: request.authorization_required,
        bearer_token_reference_present: request.bearer_token_reference_present,
        local_http_status_code,
        response_metric_line_count,
        response_metric_lines,
        missing_control_count,
        local_metrics_endpoint_started,
        network_request_served,
        public_network_exposed: false,
        telemetry_exported: false,
        outbound_alerts_sent: false,
        production_ready: false,
        warnings,
    };
    report.validate()?;
    Ok(report)
}

struct LocalMetricsEndpointExchange {
    bound_port: u16,
    local_http_status_code: u16,
    response_metric_lines: Vec<String>,
    network_request_served: bool,
}

struct LocalMetricsRuntimeExchange {
    bound_port: u16,
    served_scrape_count: u32,
    all_scrapes_returned_ok: bool,
    response_metric_lines_consistent: bool,
    local_metrics_runtime_shutdown: bool,
}

fn serve_one_local_metrics_request(
    ip: IpAddr,
    requested_port: u16,
    method: &str,
    path: &str,
    metric_lines: &[String],
) -> Result<LocalMetricsEndpointExchange, String> {
    let listener = TcpListener::bind((ip, requested_port))
        .map_err(|error| format!("bind failed: {}", error.kind()))?;
    listener
        .set_nonblocking(false)
        .map_err(|error| format!("listener mode failed: {}", error.kind()))?;
    let bound_port = listener
        .local_addr()
        .map_err(|error| format!("local address failed: {}", error.kind()))?
        .port();
    let response_body = format!("{}\n", metric_lines.join("\n"));
    let server_body = response_body.clone();
    let server_handle = thread::spawn(move || serve_metrics_connection(listener, server_body));

    let mut stream = TcpStream::connect((ip, bound_port))
        .map_err(|error| format!("client connect failed: {}", error.kind()))?;
    let timeout = Some(Duration::from_secs(2));
    stream
        .set_read_timeout(timeout)
        .map_err(|error| format!("client read timeout failed: {}", error.kind()))?;
    stream
        .set_write_timeout(timeout)
        .map_err(|error| format!("client write timeout failed: {}", error.kind()))?;
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {ip}:{bound_port}\r\nAuthorization: Bearer local-reference\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("client write failed: {}", error.kind()))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("client read failed: {}", error.kind()))?;
    let request_served = server_handle
        .join()
        .map_err(|_| "server thread panicked".to_owned())?
        .map_err(|error| format!("server failed: {error}"))?;
    let local_http_status_code = parse_http_status_code(&response).unwrap_or(0);
    let response_metric_lines = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    Ok(LocalMetricsEndpointExchange {
        bound_port,
        local_http_status_code,
        response_metric_lines,
        network_request_served: request_served,
    })
}

fn serve_bounded_local_metrics_runtime(
    ip: IpAddr,
    requested_port: u16,
    metric_lines: &[String],
    scrape_count: u32,
) -> Result<LocalMetricsRuntimeExchange, String> {
    if scrape_count == 0 {
        return Err("scrape count must be positive".to_owned());
    }
    let listener = TcpListener::bind((ip, requested_port))
        .map_err(|error| format!("bind failed: {}", error.kind()))?;
    listener
        .set_nonblocking(false)
        .map_err(|error| format!("listener mode failed: {}", error.kind()))?;
    let bound_port = listener
        .local_addr()
        .map_err(|error| format!("local address failed: {}", error.kind()))?
        .port();
    let response_body = format!("{}\n", metric_lines.join("\n"));
    let expected_metric_lines = metric_lines.to_vec();
    let server_handle =
        thread::spawn(move || serve_metrics_connections(listener, response_body, scrape_count));
    let mut served_scrape_count = 0_u32;
    let mut all_scrapes_returned_ok = true;
    let mut response_metric_lines_consistent = true;
    for _ in 0..scrape_count {
        let mut stream = TcpStream::connect((ip, bound_port))
            .map_err(|error| format!("client connect failed: {}", error.kind()))?;
        let timeout = Some(Duration::from_secs(2));
        stream
            .set_read_timeout(timeout)
            .map_err(|error| format!("client read timeout failed: {}", error.kind()))?;
        stream
            .set_write_timeout(timeout)
            .map_err(|error| format!("client write timeout failed: {}", error.kind()))?;
        let request = format!(
            "GET /metrics HTTP/1.1\r\nHost: {ip}:{bound_port}\r\nAuthorization: Bearer local-reference\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| format!("client write failed: {}", error.kind()))?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|error| format!("client read failed: {}", error.kind()))?;
        served_scrape_count = served_scrape_count.saturating_add(1);
        all_scrapes_returned_ok &= parse_http_status_code(&response) == Some(200);
        let response_metric_lines = response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        response_metric_lines_consistent &= response_metric_lines == expected_metric_lines;
    }
    let server_served_count = server_handle
        .join()
        .map_err(|_| "server thread panicked".to_owned())?
        .map_err(|error| format!("server failed: {error}"))?;
    Ok(LocalMetricsRuntimeExchange {
        bound_port,
        served_scrape_count,
        all_scrapes_returned_ok,
        response_metric_lines_consistent,
        local_metrics_runtime_shutdown: server_served_count == scrape_count
            && served_scrape_count == scrape_count,
    })
}

fn serve_metrics_connection(listener: TcpListener, response_body: String) -> Result<bool, String> {
    let (mut stream, _) = listener
        .accept()
        .map_err(|error| format!("accept failed: {}", error.kind()))?;
    let timeout = Some(Duration::from_secs(2));
    stream
        .set_read_timeout(timeout)
        .map_err(|error| format!("server read timeout failed: {}", error.kind()))?;
    stream
        .set_write_timeout(timeout)
        .map_err(|error| format!("server write timeout failed: {}", error.kind()))?;
    let mut buffer = [0_u8; 1024];
    let bytes_read = stream
        .read(&mut buffer)
        .map_err(|error| format!("server read failed: {}", error.kind()))?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let authorized = request.starts_with("GET /metrics ")
        && request.contains("\r\nAuthorization: Bearer local-reference\r\n");
    let (status, body) = if authorized {
        ("200 OK", response_body)
    } else {
        ("403 Forbidden", String::new())
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("server write failed: {}", error.kind()))?;
    Ok(true)
}

fn serve_metrics_connections(
    listener: TcpListener,
    response_body: String,
    scrape_count: u32,
) -> Result<u32, String> {
    let mut served = 0_u32;
    for _ in 0..scrape_count {
        let (mut stream, _) = listener
            .accept()
            .map_err(|error| format!("accept failed: {}", error.kind()))?;
        let timeout = Some(Duration::from_secs(2));
        stream
            .set_read_timeout(timeout)
            .map_err(|error| format!("server read timeout failed: {}", error.kind()))?;
        stream
            .set_write_timeout(timeout)
            .map_err(|error| format!("server write timeout failed: {}", error.kind()))?;
        let mut buffer = [0_u8; 1024];
        let bytes_read = stream
            .read(&mut buffer)
            .map_err(|error| format!("server read failed: {}", error.kind()))?;
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let authorized = request.starts_with("GET /metrics ")
            && request.contains("\r\nAuthorization: Bearer local-reference\r\n");
        let (status, body) = if authorized {
            ("200 OK", response_body.as_str())
        } else {
            ("403 Forbidden", "")
        };
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .map_err(|error| format!("server write failed: {}", error.kind()))?;
        served = served.saturating_add(1);
    }
    Ok(served)
}

fn parse_http_status_code(response: &str) -> Option<u16> {
    response
        .lines()
        .next()?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

impl RuntimeFailureCaptureRequest {
    /// Validate local runtime failure capture request invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.config.validate()?;
        self.access.validate()?;
        let mut violations = Vec::new();
        validate_id("runtime failure", &self.failure_id, &mut violations);
        validate_name(
            "runtime failure component",
            &self.component,
            &mut violations,
        );
        if self.summary.trim().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_FAILURE_SUMMARY_EMPTY",
                "runtime failure summary must be non-empty",
            ));
        }
        if self.detail.trim().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_FAILURE_DETAIL_EMPTY",
                "runtime failure detail must be non-empty",
            ));
        }
        if self.captured_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_FAILURE_TIMESTAMP_ZERO",
                "runtime failure capture timestamp must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl RuntimePanicHookInstallationRequest {
    /// Validate local runtime panic-hook installation request invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.config.validate()?;
        self.access.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "runtime panic hook failure",
            &self.failure_id,
            &mut violations,
        );
        validate_name(
            "runtime panic hook component",
            &self.component,
            &mut violations,
        );
        if self.summary.trim().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PANIC_HOOK_SUMMARY_EMPTY",
                "runtime panic hook summary must be non-empty",
            ));
        }
        if self.detail.trim().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PANIC_HOOK_DETAIL_EMPTY",
                "runtime panic hook detail must be non-empty",
            ));
        }
        if self.audit_path.as_os_str().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PANIC_HOOK_AUDIT_PATH_EMPTY",
                "runtime panic hook audit path must be non-empty",
            ));
        }
        if self.state_path.as_os_str().is_empty() {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PANIC_HOOK_STATE_PATH_EMPTY",
                "runtime panic hook state path must be non-empty",
            ));
        }
        if self.captured_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_PANIC_HOOK_TIMESTAMP_ZERO",
                "runtime panic hook capture timestamp must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl RuntimeFailureCaptureRecord {
    /// Validate local runtime failure capture record invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id("runtime failure", &self.failure_id, &mut violations);
        validate_name(
            "runtime failure component",
            &self.component,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if contains_secret_like_text(&self.summary) || contains_secret_like_text(&self.detail) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_FAILURE_SECRET_LIKE",
                "runtime failure capture still looks like it may contain secret material",
            ));
        }
        if !self.access_authorized {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_FAILURE_ACCESS_NOT_AUTHORIZED",
                "runtime failure capture must be locally access-authorized",
            ));
        }
        if self.metrics_endpoint_started
            || self.public_network_exposed
            || self.outbound_alerts_sent
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_FAILURE_FORBIDDEN_SIDE_EFFECT",
                "runtime failure capture must not start endpoints, send alerts, submit adapters, execute live actions, or approve production readiness",
            ));
        }
        finish_validation(violations)
    }
}

/// Capture one local runtime failure through the deterministic observability boundary.
///
/// This records sanitized local failure metadata only. It does not install panic
/// hooks, start endpoints, export telemetry, send alerts, submit adapters,
/// execute live actions, inspect deployment state, or claim production readiness.
pub fn capture_local_runtime_failure(
    request: RuntimeFailureCaptureRequest,
) -> Result<RuntimeFailureCaptureRecord, ObservabilityError> {
    request.validate()?;
    let access_decision = authorize_observability_access(&request.access, &request.config);
    if !access_decision.access_authorized {
        return Err(ObservabilityError::ValidationFailed {
            violations: vec![ObservabilityViolation::new_owned(
                "OBSERVABILITY_ACCESS_DENIED",
                access_decision.reason,
            )],
        });
    }

    let max_chars = 512;
    let mut redaction_applied = false;
    let (component, component_redacted) =
        sanitize_observability_text(&request.component, max_chars);
    let (summary, summary_redacted) = sanitize_observability_text(&request.summary, max_chars);
    let (detail, detail_redacted) = sanitize_observability_text(&request.detail, max_chars);
    redaction_applied |= component_redacted || summary_redacted || detail_redacted;

    let record = RuntimeFailureCaptureRecord {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        failure_id: request.failure_id,
        component,
        kind: request.kind,
        severity: request.severity,
        summary,
        detail,
        captured_at_ms: request.captured_at_ms,
        access_authorized: access_decision.access_authorized,
        access_authorization_status: access_decision.status,
        secret_redaction_applied: redaction_applied,
        metrics_endpoint_started: false,
        public_network_exposed: false,
        outbound_alerts_sent: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    record.validate()?;
    Ok(record)
}

/// Capture a local panic through a scoped panic hook and existing audit/state boundaries.
///
/// The hook is installed only for the supplied local operation and the previous
/// hook is restored before returning. The panic is caught, sanitized, audited,
/// and checkpointed locally; this does not start endpoints, export telemetry,
/// send alerts, submit adapters, execute live actions, inspect deployment
/// state, or claim production readiness.
pub fn capture_local_panic_with_scoped_hook(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    mut request: RuntimeFailureCaptureRequest,
    operation: impl FnOnce(),
) -> Result<LocalPanicHookCaptureReport, ObservabilityError> {
    if request.kind != RuntimeFailureKind::Panic {
        return Err(ObservabilityError::ValidationFailed {
            violations: vec![ObservabilityViolation::new(
                "OBSERVABILITY_PANIC_HOOK_KIND_REQUIRED",
                "scoped panic-hook capture requires RuntimeFailureKind::Panic",
            )],
        });
    }
    request.validate()?;

    let captured_detail: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let captured_for_hook = Arc::clone(&captured_detail);
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload().downcast_ref::<&str>().map_or_else(
            || {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map_or("non-string panic payload", String::as_str)
            },
            |message| *message,
        );
        let location = panic_info.location().map_or_else(
            || "unknown location".to_owned(),
            |location| format!("{}:{}", location.file(), location.line()),
        );
        if let Ok(mut captured) = captured_for_hook.lock() {
            *captured = Some(format!("panic captured at {location}: {payload}"));
        }
    }));

    let panic_result = panic::catch_unwind(AssertUnwindSafe(operation));
    panic::set_hook(previous_hook);

    if panic_result.is_ok() {
        return Ok(LocalPanicHookCaptureReport {
            hook_installed: true,
            hook_restored: true,
            panic_observed: false,
            failure_record: None,
            audit_sequence: None,
            checkpoint_key: None,
            metrics_endpoint_started: false,
            public_network_exposed: false,
            outbound_alerts_sent: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready: false,
        });
    }

    let detail = captured_detail
        .lock()
        .ok()
        .and_then(|captured| captured.clone())
        .unwrap_or_else(|| "panic hook observed local panic without payload detail".to_owned());
    request.detail = format!("{}; {detail}", request.detail);
    let record = capture_local_runtime_failure(request)?;
    let audit_record =
        append_runtime_failure_capture_audit(journal, &record, record.captured_at_ms)?;
    let checkpoint =
        persist_runtime_failure_capture_checkpoint(store, &record, record.captured_at_ms)?;

    Ok(LocalPanicHookCaptureReport {
        hook_installed: true,
        hook_restored: true,
        panic_observed: true,
        failure_record: Some(record),
        audit_sequence: Some(audit_record.sequence),
        checkpoint_key: Some(checkpoint.key),
        metrics_endpoint_started: false,
        public_network_exposed: false,
        outbound_alerts_sent: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    })
}

/// Install a local runtime panic hook that captures the first panic to audit/state.
///
/// The returned guard restores the previous panic hook on drop. The hook opens
/// only caller-supplied local audit and SQLite paths and records sanitized
/// failure metadata through the existing observability boundary. It does not
/// start endpoints, export telemetry, send alerts, submit adapters, execute
/// live actions, inspect deployment state, or claim production readiness.
pub fn install_local_runtime_panic_hook(
    request: RuntimePanicHookInstallationRequest,
) -> Result<LocalRuntimePanicHookGuard, ObservabilityError> {
    request.validate()?;

    let failure_id = request.failure_id.clone();
    let report = RuntimePanicHookInstallationReport {
        hook_installed: true,
        failure_id,
        metrics_endpoint_started: false,
        public_network_exposed: false,
        outbound_alerts_sent: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    let panic_captured = Arc::new(Mutex::new(false));
    let last_capture_error = Arc::new(Mutex::new(None));
    let hook_panic_captured = Arc::clone(&panic_captured);
    let hook_last_capture_error = Arc::clone(&last_capture_error);
    let previous_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        let Ok(mut already_captured) = hook_panic_captured.lock() else {
            return;
        };
        if *already_captured {
            return;
        }
        *already_captured = true;
        drop(already_captured);

        let payload = panic_info.payload().downcast_ref::<&str>().map_or_else(
            || {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map_or("non-string panic payload", String::as_str)
            },
            |message| *message,
        );
        let location = panic_info.location().map_or_else(
            || "unknown location".to_owned(),
            |location| format!("{}:{}", location.file(), location.line()),
        );
        let capture_result = (|| -> Result<(), ObservabilityError> {
            let record = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
                failure_id: request.failure_id.clone(),
                component: request.component.clone(),
                kind: RuntimeFailureKind::Panic,
                severity: request.severity,
                summary: request.summary.clone(),
                detail: format!(
                    "{}; panic captured at {location}: {payload}",
                    request.detail
                ),
                config: request.config.clone(),
                access: request.access.clone(),
                captured_at_ms: request.captured_at_ms,
            })?;
            let mut journal = AppendOnlyAuditJournal::open(&request.audit_path)?;
            let mut store = SqliteWalStateStore::open(&request.state_path)?;
            append_runtime_failure_capture_audit(&mut journal, &record, request.captured_at_ms)?;
            persist_runtime_failure_capture_checkpoint(
                &mut store,
                &record,
                request.captured_at_ms,
            )?;
            Ok(())
        })();

        if let Err(error) = capture_result {
            if let Ok(mut last_error) = hook_last_capture_error.lock() {
                *last_error = Some(error.to_string());
            }
        }
    }));

    Ok(LocalRuntimePanicHookGuard {
        previous_hook: Some(previous_hook),
        report,
        panic_captured,
        last_capture_error,
    })
}

impl LocalTracingSubscriberValidationRequest {
    /// Validate scoped tracing subscriber request invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        self.config.validate()?;
        self.access.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "observability tracing subscriber validation",
            &self.validation_id,
            &mut violations,
        );
        validate_name(
            "observability tracing subscriber label",
            &self.subscriber_label,
            &mut violations,
        );
        validate_id(
            "observability tracing event",
            &self.event.id,
            &mut violations,
        );
        validate_name(
            "observability tracing event target",
            &self.event.target,
            &mut violations,
        );
        if !self.local_capture_required {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_LOCAL_CAPTURE_REQUIRED",
                "local tracing subscriber validation requires local in-process capture",
            ));
        }
        if !self.redaction_required {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_REDACTION_REQUIRED",
                "local tracing subscriber validation requires redaction before emission",
            ));
        }
        if self.global_install_requested
            || self.telemetry_export_requested
            || self.outbound_alert_delivery_requested
            || self.public_network_exposure_requested
            || self.live_execution_requested
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_SIDE_EFFECT_REQUESTED",
                "local tracing subscriber validation must not install a global subscriber, export telemetry, send alerts, expose public networks, or execute live actions",
            ));
        }
        if self.captured_at_ms == 0 {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_TIMESTAMP_ZERO",
                "local tracing subscriber validation timestamp must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl LocalTracingSubscriberValidationReport {
    /// Validate scoped tracing subscriber report invariants.
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        let mut violations = Vec::new();
        validate_id(
            "observability tracing subscriber validation",
            &self.validation_id,
            &mut violations,
        );
        validate_name(
            "observability tracing subscriber label",
            &self.subscriber_label,
            &mut violations,
        );
        validate_id(
            "observability tracing event",
            &self.event_id,
            &mut violations,
        );
        validate_name(
            "observability tracing event target",
            &self.event_target,
            &mut violations,
        );
        if self.observability_runbook_version != OBSERVABILITY_RUNBOOK_VERSION {
            violations.push(ObservabilityViolation::new_owned(
                "OBSERVABILITY_VERSION_MISMATCH",
                format!(
                    "observability_runbook_version must be {OBSERVABILITY_RUNBOOK_VERSION}, got {}",
                    self.observability_runbook_version
                ),
            ));
        }
        if contains_secret_like_text(&self.captured_output_excerpt) {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_OUTPUT_SECRET_LIKE",
                "local tracing subscriber output still looks like secret material",
            ));
        }
        if !self.access_authorized {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_ACCESS_NOT_AUTHORIZED",
                "local tracing subscriber validation must be locally access-authorized",
            ));
        }
        if self.global_subscriber_installed
            || self.telemetry_exported
            || self.outbound_alerts_sent
            || self.public_network_exposed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_FORBIDDEN_SIDE_EFFECT",
                "local tracing subscriber validation must not install global subscribers, export telemetry, send alerts, expose public networks, execute live actions, or approve production readiness",
            ));
        }
        match self.status {
            LocalTracingSubscriberValidationStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.scoped_subscriber_installed
                    || !self.event_captured
                    || self.captured_event_count == 0
                    || self.captured_output_excerpt.trim().is_empty()
                {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_TRACING_READY_MISMATCH",
                        "ready local tracing subscriber validation requires scoped install, captured event output, and zero missing controls",
                    ));
                }
            }
            LocalTracingSubscriberValidationStatus::Blocked => {
                if self.missing_control_count == 0 {
                    violations.push(ObservabilityViolation::new(
                        "OBSERVABILITY_TRACING_BLOCKED_MISMATCH",
                        "blocked local tracing subscriber validation requires at least one finding",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

/// Validate a scoped local tracing subscriber by emitting one sanitized event.
///
/// This installs the subscriber only inside the current call, captures output in
/// memory, and never installs a global subscriber, starts endpoints, exports
/// telemetry, sends alerts, exposes public networks, executes live actions, or
/// claims production readiness.
pub fn validate_local_tracing_subscriber(
    request: LocalTracingSubscriberValidationRequest,
) -> Result<LocalTracingSubscriberValidationReport, ObservabilityError> {
    request.validate()?;
    let access_decision = authorize_observability_access(&request.access, &request.config);
    if !access_decision.access_authorized {
        return Err(ObservabilityError::ValidationFailed {
            violations: vec![ObservabilityViolation::new_owned(
                "OBSERVABILITY_ACCESS_DENIED",
                access_decision.reason,
            )],
        });
    }

    let (event, secret_redaction_applied) =
        request.event.redacted(512, request.config.max_log_fields);
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = LocalTraceBuffer {
        buffer: Arc::clone(&buffer),
    };
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(writer)
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        tracing::event!(
            target: "arbyclaw.local_observability",
            tracing::Level::INFO,
            validation_id = %request.validation_id,
            subscriber = %request.subscriber_label,
            event_id = %event.id,
            event_target = %event.target,
            fields = %event.fields.len(),
            "{}",
            event.message
        );
    });

    let captured = buffer
        .lock()
        .map_err(|_| ObservabilityError::ValidationFailed {
            violations: vec![ObservabilityViolation::new(
                "OBSERVABILITY_TRACING_BUFFER_LOCK_FAILED",
                "local tracing subscriber capture buffer lock failed",
            )],
        })?
        .clone();
    let captured_output = String::from_utf8_lossy(&captured).into_owned();
    let (captured_output_excerpt, output_redacted) =
        sanitize_observability_text(captured_output.trim(), 512);
    let event_captured = captured_output_excerpt.contains(&event.id)
        && captured_output_excerpt.contains(&event.target)
        && !captured_output_excerpt.trim().is_empty();
    let missing_control_count = u32::from(!event_captured);
    let status = if missing_control_count == 0 {
        LocalTracingSubscriberValidationStatus::ReadyForLocalReview
    } else {
        LocalTracingSubscriberValidationStatus::Blocked
    };
    let report = LocalTracingSubscriberValidationReport {
        observability_runbook_version: OBSERVABILITY_RUNBOOK_VERSION.to_owned(),
        validation_id: request.validation_id,
        subscriber_label: request.subscriber_label,
        status,
        event_id: event.id,
        event_target: event.target,
        scoped_subscriber_installed: true,
        event_captured,
        captured_event_count: u32::from(event_captured),
        captured_output_excerpt,
        missing_control_count,
        secret_redaction_applied: secret_redaction_applied || output_redacted,
        access_authorized: access_decision.access_authorized,
        access_authorization_status: access_decision.status,
        global_subscriber_installed: false,
        telemetry_exported: false,
        outbound_alerts_sent: false,
        public_network_exposed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

#[derive(Clone)]
struct LocalTraceBuffer {
    buffer: Arc<Mutex<Vec<u8>>>,
}

struct LocalTraceWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for LocalTraceWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.buffer
            .lock()
            .map_err(|_| std::io::Error::other("local tracing buffer lock failed"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for LocalTraceBuffer {
    type Writer = LocalTraceWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        LocalTraceWriter {
            buffer: Arc::clone(&self.buffer),
        }
    }
}

/// Persist the latest local observability record through the typed state boundary.
///
/// This stores redacted local observability metadata only. It does not start a
/// metrics endpoint, export telemetry, ship logs, or send alerts.
pub fn persist_observability_record_checkpoint(
    store: &mut impl StateStore,
    record: &ObservabilityRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!("failed to serialize observability record checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability collection record to the append-only audit journal.
///
/// This records sanitized collection outcomes only. It does not start endpoints,
/// export telemetry, ship logs, or deliver alerts.
pub fn append_observability_record_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &ObservabilityRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-record-{}", record.snapshot_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-collector",
        "observability record collected",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata("snapshot_id", AuditValue::Text(record.snapshot_id.clone()))
        .with_metadata(
            "overall_health",
            AuditValue::Text(format!("{:?}", record.overall_health)),
        )
        .with_metadata(
            "access_authorized",
            AuditValue::Bool(record.access_authorized),
        )
        .with_metadata(
            "access_authorization_status",
            AuditValue::Text(format!("{:?}", record.access_authorization_status)),
        )
        .with_metadata(
            "component_count",
            AuditValue::Text(record.components.len().to_string()),
        )
        .with_metadata("log_count", AuditValue::Text(record.logs.len().to_string()))
        .with_metadata(
            "metric_count",
            AuditValue::Text(record.metrics.len().to_string()),
        )
        .with_metadata(
            "runbook_count",
            AuditValue::Text(record.runbooks.len().to_string()),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(record.metrics_endpoint_started),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(record.public_network_exposed),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(record.outbound_alerts_sent),
        )
        .with_metadata(
            "telemetry_text_redaction_applied",
            AuditValue::Bool(record.secret_redaction_applied),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local tracing subscriber validation through the typed state boundary.
///
/// This stores sanitized local capture metadata only. It does not install a
/// global subscriber, export telemetry, ship logs, send alerts, or claim
/// production readiness.
pub fn persist_local_tracing_subscriber_checkpoint(
    store: &mut impl StateStore,
    report: &LocalTracingSubscriberValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!("failed to serialize local tracing subscriber checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local tracing subscriber validation to the append-only audit journal.
///
/// This records sanitized local subscriber capture outcomes only. It does not
/// install a global subscriber, export telemetry, ship logs, send alerts, or
/// approve production readiness.
pub fn append_local_tracing_subscriber_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &LocalTracingSubscriberValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-tracing-subscriber-{}", report.validation_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "local-tracing-subscriber",
        "local tracing subscriber captured sanitized event",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata(
            "validation_id",
            AuditValue::Text(report.validation_id.clone()),
        )
        .with_metadata(
            "subscriber_label",
            AuditValue::Text(report.subscriber_label.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("event_id", AuditValue::Text(report.event_id.clone()))
        .with_metadata(
            "scoped_subscriber_installed",
            AuditValue::Bool(report.scoped_subscriber_installed),
        )
        .with_metadata("event_captured", AuditValue::Bool(report.event_captured))
        .with_metadata(
            "captured_event_count",
            AuditValue::Text(report.captured_event_count.to_string()),
        )
        .with_metadata(
            "access_authorized",
            AuditValue::Bool(report.access_authorized),
        )
        .with_metadata(
            "global_subscriber_installed",
            AuditValue::Bool(report.global_subscriber_installed),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local observability log retention execution through the typed state boundary.
///
/// This stores sandbox-only retention results. It does not touch production
/// logs, service managers, external log shipping, live networks, or approve
/// production readiness.
pub fn persist_observability_log_retention_execution_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityLogRetentionExecutionReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_LOG_RETENTION_EXECUTION_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability log retention execution checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability log retention execution to the append-only audit journal.
///
/// This records sandbox-only retention outcomes. It does not touch production
/// logs, service managers, external log shipping, live networks, or approve
/// production readiness.
pub fn append_observability_log_retention_execution_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityLogRetentionExecutionReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-log-retention-{}", report.execution_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-log-retention",
        "observability log retention executed inside local sandbox",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata(
            "execution_id",
            AuditValue::Text(report.execution_id.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata(
            "rotate_active_requested",
            AuditValue::Bool(report.rotate_active_requested),
        )
        .with_metadata(
            "new_active_created",
            AuditValue::Bool(report.new_active_created),
        )
        .with_metadata(
            "retained_archive_count",
            AuditValue::Text(report.retained_archives.len().to_string()),
        )
        .with_metadata(
            "expired_archive_deleted_count",
            AuditValue::Text(report.expired_archives_deleted.len().to_string()),
        )
        .with_metadata(
            "deleted_file_count",
            AuditValue::Unsigned(report.deleted_file_count),
        )
        .with_metadata(
            "sandbox_filesystem_mutated",
            AuditValue::Bool(report.sandbox_filesystem_mutated),
        )
        .with_metadata(
            "out_of_workspace_path_touched",
            AuditValue::Bool(report.out_of_workspace_path_touched),
        )
        .with_metadata(
            "production_log_paths_touched",
            AuditValue::Bool(report.production_log_paths_touched),
        )
        .with_metadata(
            "service_manager_action_performed",
            AuditValue::Bool(report.service_manager_action_performed),
        )
        .with_metadata(
            "external_log_shipping_performed",
            AuditValue::Bool(report.external_log_shipping_performed),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local observability operations review through the typed state boundary.
///
/// This stores sanitized local review metadata only. It does not start endpoints,
/// export telemetry, ship logs, deliver alerts, mutate retention storage, or
/// approve production readiness.
pub fn persist_observability_operations_review_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityOperationsReviewReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability operations review checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability operations review to the append-only audit journal.
///
/// This records retention and alert-route review outcomes only. It does not start
/// endpoints, export telemetry, ship logs, deliver alerts, or mutate retention storage.
pub fn append_observability_operations_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityOperationsReviewReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-operations-review-{}", report.review_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-operations-review",
        "observability operations review recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "log_retention_required",
            AuditValue::Bool(report.log_retention_required),
        )
        .with_metadata(
            "retention_days",
            AuditValue::Text(report.retention_days.to_string()),
        )
        .with_metadata(
            "alert_routing_required",
            AuditValue::Bool(report.alert_routing_required),
        )
        .with_metadata(
            "alert_route_count",
            AuditValue::Text(report.alert_route_count.to_string()),
        )
        .with_metadata(
            "incident_runbook_count",
            AuditValue::Text(report.incident_runbook_count.to_string()),
        )
        .with_metadata(
            "audit_state_preflight_required",
            AuditValue::Bool(report.audit_state_preflight_required),
        )
        .with_metadata(
            "exporter_kill_switch_required",
            AuditValue::Bool(report.exporter_kill_switch_required),
        )
        .with_metadata(
            "alert_authorization_required",
            AuditValue::Bool(report.alert_authorization_required),
        )
        .with_metadata(
            "rate_limit_backpressure_required",
            AuditValue::Bool(report.rate_limit_backpressure_required),
        )
        .with_metadata(
            "retry_backoff_required",
            AuditValue::Bool(report.retry_backoff_required),
        )
        .with_metadata(
            "telemetry_sanitization_required",
            AuditValue::Bool(report.no_secret_telemetry_required),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(report.metrics_endpoint_started),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local observability export dry run through the typed state boundary.
///
/// This stores sanitized local dry-run metadata only. It does not start
/// endpoints, export telemetry, ship logs, deliver alerts, or approve
/// production readiness.
pub fn persist_observability_export_dry_run_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityExportDryRunReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability export dry-run checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability export dry run to the append-only audit journal.
///
/// This records metrics/export and alert-route dry-run outcomes only. It does
/// not start endpoints, export telemetry, ship logs, deliver alerts, or mutate
/// retention storage.
pub fn append_observability_export_dry_run_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityExportDryRunReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-export-dry-run-{}", report.snapshot_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-export-dry-run",
        "observability export dry-run recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata("snapshot_id", AuditValue::Text(report.snapshot_id.clone()))
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata(
            "metric_line_count",
            AuditValue::Text(report.prometheus_metric_lines.len().to_string()),
        )
        .with_metadata(
            "log_event_count",
            AuditValue::Text(report.log_event_count.to_string()),
        )
        .with_metadata(
            "runbook_count",
            AuditValue::Text(report.runbook_count.to_string()),
        )
        .with_metadata(
            "alert_route_count",
            AuditValue::Text(report.alert_route_count.to_string()),
        )
        .with_metadata(
            "alert_dry_run_count",
            AuditValue::Text(report.alert_dry_run_count.to_string()),
        )
        .with_metadata(
            "loopback_or_authenticated_endpoint_required",
            AuditValue::Bool(report.loopback_or_authenticated_endpoint_required),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(report.metrics_endpoint_started),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local observability alert-route dispatch review through the typed state boundary.
///
/// This stores sanitized bridge metadata only. It does not deliver alerts, call
/// messaging platforms, export telemetry, execute live actions, or approve
/// production readiness.
pub fn persist_observability_alert_route_dispatch_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityAlertRouteDispatchReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability alert-route dispatch checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability alert-route dispatch review to the audit journal.
///
/// This records only the local communications bridge decision. It does not
/// deliver alerts, call messaging platforms, export telemetry, execute live
/// actions, or approve production readiness.
pub fn append_observability_alert_route_dispatch_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityAlertRouteDispatchReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "observability-alert-route-dispatch-{}",
            report.dispatch_review_id
        ),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-alert-route-dispatch",
        "observability alert-route dispatch recorded through local communications boundary",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata(
            "dispatch_review_id",
            AuditValue::Text(report.dispatch_review_id.clone()),
        )
        .with_metadata("snapshot_id", AuditValue::Text(report.snapshot_id.clone()))
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata(
            "alert_route_reference",
            AuditValue::Text(report.alert_route_reference.clone()),
        )
        .with_metadata(
            "notification_dispatch_id",
            AuditValue::Text(report.notification_dispatch_id.clone()),
        )
        .with_metadata(
            "notification_id",
            AuditValue::Text(report.notification_id.clone()),
        )
        .with_metadata(
            "notification_dispatch_status",
            AuditValue::Text(format!("{:?}", report.notification_dispatch_status)),
        )
        .with_metadata(
            "recorded_local_channel_count",
            AuditValue::Text(report.recorded_local_channel_count.to_string()),
        )
        .with_metadata(
            "blocked_channel_count",
            AuditValue::Text(report.blocked_channel_count.to_string()),
        )
        .with_metadata(
            "local_dispatch_required",
            AuditValue::Bool(report.local_dispatch_required),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local observability endpoint/exporter preflight through the typed state boundary.
///
/// This stores sanitized local preflight metadata only. It does not start
/// endpoints, export telemetry, ship logs, deliver alerts, or approve
/// production readiness.
pub fn persist_observability_endpoint_preflight_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityEndpointPreflightReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability endpoint preflight checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability endpoint/exporter preflight to the append-only audit journal.
///
/// This records endpoint/exporter preflight outcomes only. It does not start
/// endpoints, export telemetry, ship logs, deliver alerts, or mutate retention storage.
pub fn append_observability_endpoint_preflight_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityEndpointPreflightReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-endpoint-preflight-{}", report.preflight_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-endpoint-preflight",
        "observability endpoint preflight recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata(
            "preflight_id",
            AuditValue::Text(report.preflight_id.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "loopback_bind_validated",
            AuditValue::Bool(report.loopback_bind_validated),
        )
        .with_metadata(
            "authentication_required",
            AuditValue::Bool(report.authentication_required),
        )
        .with_metadata(
            "authorization_required",
            AuditValue::Bool(report.authorization_required),
        )
        .with_metadata(
            "transport_protection_required",
            AuditValue::Bool(report.transport_protection_required),
        )
        .with_metadata(
            "redaction_required",
            AuditValue::Bool(report.redaction_required),
        )
        .with_metadata(
            "alert_route_count",
            AuditValue::Text(report.alert_route_count.to_string()),
        )
        .with_metadata(
            "exporter_backpressure_required",
            AuditValue::Bool(report.exporter_backpressure_required),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(report.metrics_endpoint_started),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local observability loopback bind validation through the typed state boundary.
///
/// This stores sanitized local validation metadata only. It does not keep a
/// listener running, serve requests, expose public networks, export telemetry,
/// ship logs, deliver alerts, or approve production readiness.
pub fn persist_observability_loopback_bind_validation_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityLoopbackBindValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability loopback bind validation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local observability loopback bind validation to the append-only audit journal.
pub fn append_observability_loopback_bind_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityLoopbackBindValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "observability-loopback-bind-validation-{}",
            report.validation_id
        ),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-loopback-bind-validation",
        "observability loopback bind validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata(
            "validation_id",
            AuditValue::Text(report.validation_id.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("bind_host", AuditValue::Text(report.bind_host.clone()))
        .with_metadata(
            "requested_port",
            AuditValue::Text(report.requested_port.to_string()),
        )
        .with_metadata(
            "bound_port",
            AuditValue::Text(report.bound_port.unwrap_or(0).to_string()),
        )
        .with_metadata(
            "loopback_bind_validated",
            AuditValue::Bool(report.loopback_bind_validated),
        )
        .with_metadata(
            "listener_opened_and_closed",
            AuditValue::Bool(report.listener_opened_and_closed),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(report.metrics_endpoint_started),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local authenticated metrics scrape preflight through the typed state boundary.
///
/// This stores sanitized local scrape metadata only. It does not start a
/// metrics endpoint, serve socket requests, expose public networks, export
/// telemetry, ship logs, deliver alerts, or approve production readiness.
pub fn persist_observability_metrics_scrape_preflight_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityMetricsScrapePreflightReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability metrics scrape preflight checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local authenticated metrics scrape preflight to the append-only audit journal.
pub fn append_observability_metrics_scrape_preflight_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityMetricsScrapePreflightReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "observability-metrics-scrape-preflight-{}",
            report.scrape_id
        ),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-metrics-scrape-preflight",
        "observability metrics scrape preflight recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata("scrape_id", AuditValue::Text(report.scrape_id.clone()))
        .with_metadata("snapshot_id", AuditValue::Text(report.snapshot_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "request_method",
            AuditValue::Text(report.request_method.clone()),
        )
        .with_metadata(
            "request_path",
            AuditValue::Text(report.request_path.clone()),
        )
        .with_metadata("source_host", AuditValue::Text(report.source_host.clone()))
        .with_metadata(
            "loopback_source_validated",
            AuditValue::Bool(report.loopback_source_validated),
        )
        .with_metadata(
            "authentication_required",
            AuditValue::Bool(report.authentication_required),
        )
        .with_metadata(
            "authorization_required",
            AuditValue::Bool(report.authorization_required),
        )
        .with_metadata(
            "bearer_token_reference_present",
            AuditValue::Bool(report.bearer_token_reference_present),
        )
        .with_metadata(
            "local_http_status_code",
            AuditValue::Text(report.local_http_status_code.to_string()),
        )
        .with_metadata(
            "response_metric_line_count",
            AuditValue::Text(report.response_metric_line_count.to_string()),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(report.metrics_endpoint_started),
        )
        .with_metadata(
            "network_request_served",
            AuditValue::Bool(report.network_request_served),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local one-shot metrics endpoint validation through the typed state boundary.
pub fn persist_observability_metrics_endpoint_validation_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityMetricsEndpointValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability metrics endpoint validation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local one-shot metrics endpoint validation to the append-only audit journal.
pub fn append_observability_metrics_endpoint_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityMetricsEndpointValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "observability-metrics-endpoint-validation-{}",
            report.validation_id
        ),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-metrics-endpoint-validation",
        "observability metrics endpoint validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata(
            "validation_id",
            AuditValue::Text(report.validation_id.clone()),
        )
        .with_metadata("snapshot_id", AuditValue::Text(report.snapshot_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("bind_host", AuditValue::Text(report.bind_host.clone()))
        .with_metadata(
            "bound_port",
            AuditValue::Text(
                report
                    .bound_port
                    .map_or_else(|| "none".to_owned(), |port| port.to_string()),
            ),
        )
        .with_metadata(
            "loopback_bind_validated",
            AuditValue::Bool(report.loopback_bind_validated),
        )
        .with_metadata(
            "authentication_required",
            AuditValue::Bool(report.authentication_required),
        )
        .with_metadata(
            "authorization_required",
            AuditValue::Bool(report.authorization_required),
        )
        .with_metadata(
            "bearer_token_reference_present",
            AuditValue::Bool(report.bearer_token_reference_present),
        )
        .with_metadata(
            "local_http_status_code",
            AuditValue::Text(report.local_http_status_code.to_string()),
        )
        .with_metadata(
            "response_metric_line_count",
            AuditValue::Text(report.response_metric_line_count.to_string()),
        )
        .with_metadata(
            "local_metrics_endpoint_started",
            AuditValue::Bool(report.local_metrics_endpoint_started),
        )
        .with_metadata(
            "network_request_served",
            AuditValue::Bool(report.network_request_served),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest bounded local metrics runtime probe through the typed state boundary.
pub fn persist_observability_metrics_runtime_probe_checkpoint(
    store: &mut impl StateStore,
    report: &ObservabilityMetricsRuntimeProbeReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_METRICS_RUNTIME_PROBE_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!(
                    "failed to serialize observability metrics runtime probe checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one bounded local metrics runtime probe to the append-only audit journal.
pub fn append_observability_metrics_runtime_probe_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ObservabilityMetricsRuntimeProbeReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-metrics-runtime-probe-{}", report.probe_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "observability-metrics-runtime-probe",
        "observability metrics runtime probe recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata("probe_id", AuditValue::Text(report.probe_id.clone()))
        .with_metadata("snapshot_id", AuditValue::Text(report.snapshot_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("bind_host", AuditValue::Text(report.bind_host.clone()))
        .with_metadata(
            "expected_scrape_count",
            AuditValue::Text(report.expected_scrape_count.to_string()),
        )
        .with_metadata(
            "served_scrape_count",
            AuditValue::Text(report.served_scrape_count.to_string()),
        )
        .with_metadata(
            "all_scrapes_returned_ok",
            AuditValue::Bool(report.all_scrapes_returned_ok),
        )
        .with_metadata(
            "response_metric_line_count",
            AuditValue::Text(report.response_metric_line_count.to_string()),
        )
        .with_metadata(
            "response_metric_lines_consistent",
            AuditValue::Bool(report.response_metric_lines_consistent),
        )
        .with_metadata(
            "local_metrics_runtime_started",
            AuditValue::Bool(report.local_metrics_runtime_started),
        )
        .with_metadata(
            "local_metrics_runtime_shutdown",
            AuditValue::Bool(report.local_metrics_runtime_shutdown),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "telemetry_exported",
            AuditValue::Bool(report.telemetry_exported),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(report.outbound_alerts_sent),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// Persist the latest local runtime failure capture through the typed state boundary.
///
/// This stores sanitized local failure metadata only. It does not start a
/// metrics endpoint, export telemetry, ship logs, send alerts, submit adapters,
/// execute live actions, or approve production readiness.
pub fn persist_runtime_failure_capture_checkpoint(
    store: &mut impl StateStore,
    record: &RuntimeFailureCaptureRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ObservabilityError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY.to_owned(),
        subsystem: OBSERVABILITY_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            ObservabilityError::StateStoreFailed {
                reason: format!("failed to serialize runtime failure capture checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ObservabilityError::from)?;
    Ok(checkpoint)
}

/// Append one local runtime failure capture to the append-only audit journal.
///
/// This records sanitized local failure metadata only. It does not start
/// endpoints, export telemetry, ship logs, deliver alerts, submit adapters, or
/// execute live actions.
pub fn append_runtime_failure_capture_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &RuntimeFailureCaptureRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ObservabilityError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("observability-failure-{}", record.failure_id),
        AuditEventKind::RuntimeLifecycle,
        OBSERVABILITY_STATE_SUBSYSTEM,
        "runtime-failure-capture",
        "runtime failure captured locally",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "observability_runbook_version",
            AuditValue::Text(OBSERVABILITY_RUNBOOK_VERSION.to_owned()),
        )
        .with_metadata("failure_id", AuditValue::Text(record.failure_id.clone()))
        .with_metadata("component", AuditValue::Text(record.component.clone()))
        .with_metadata(
            "failure_kind",
            AuditValue::Text(format!("{:?}", record.kind)),
        )
        .with_metadata(
            "severity",
            AuditValue::Text(format!("{:?}", record.severity)),
        )
        .with_metadata(
            "access_authorized",
            AuditValue::Bool(record.access_authorized),
        )
        .with_metadata(
            "metrics_endpoint_started",
            AuditValue::Bool(record.metrics_endpoint_started),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(record.public_network_exposed),
        )
        .with_metadata(
            "outbound_alerts_sent",
            AuditValue::Bool(record.outbound_alerts_sent),
        )
        .with_metadata(
            "external_submission_performed",
            AuditValue::Bool(record.external_submission_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(record.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(record.production_ready),
        )
        .with_metadata(
            "failure_text_redaction_applied",
            AuditValue::Bool(record.secret_redaction_applied),
        );
    journal
        .append_event(event)
        .map_err(ObservabilityError::from)
}

/// One deterministic observability validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilityViolation {
    code: &'static str,
    message: String,
}

impl ObservabilityViolation {
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

    /// Human-readable violation detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Observability boundary error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservabilityError {
    /// Observability validation failed.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<ObservabilityViolation>,
    },
    /// Local audit journal write failed.
    AuditJournalFailed {
        /// Non-secret failure reason.
        reason: String,
    },
    /// Local state-store write failed.
    StateStoreFailed {
        /// Non-secret failure reason.
        reason: String,
    },
}

impl ObservabilityError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[ObservabilityViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::AuditJournalFailed { .. } | Self::StateStoreFailed { .. } => &[],
        }
    }
}

impl From<crate::AuditError> for ObservabilityError {
    fn from(error: crate::AuditError) -> Self {
        Self::AuditJournalFailed {
            reason: error.to_string(),
        }
    }
}

impl From<StateStoreError> for ObservabilityError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                writeln!(
                    formatter,
                    "observability validation failed with {} violation(s):",
                    violations.len()
                )?;
                for violation in violations {
                    writeln!(formatter, "- {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "observability audit journal failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(formatter, "observability state store failed: {reason}")
            }
        }
    }
}

impl Error for ObservabilityError {}

fn finish_validation(violations: Vec<ObservabilityViolation>) -> Result<(), ObservabilityError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ObservabilityError::ValidationFailed { violations })
    }
}

struct ObservabilityAccessAuthorizationDecision {
    access_authorized: bool,
    status: ObservabilityAccessAuthorizationStatus,
    reason: String,
}

fn authorize_observability_access(
    access: &ObservabilityAccessContext,
    config: &ObservabilityBoundaryConfig,
) -> ObservabilityAccessAuthorizationDecision {
    match access.source {
        ObservabilityAccessSource::LocalCollection => {
            if !config.local_collection_enabled {
                return ObservabilityAccessAuthorizationDecision {
                    access_authorized: false,
                    status: ObservabilityAccessAuthorizationStatus::RejectedLocalCollectionDisabled,
                    reason: "local observability collection is disabled by configuration"
                        .to_owned(),
                };
            }
            ObservabilityAccessAuthorizationDecision {
                access_authorized: true,
                status: ObservabilityAccessAuthorizationStatus::AuthorizedLocalCollection,
                reason: "local in-process observability collection authorized".to_owned(),
            }
        }
        ObservabilityAccessSource::MetricsEndpoint
        | ObservabilityAccessSource::ExporterSession
        | ObservabilityAccessSource::AlertDelivery => ObservabilityAccessAuthorizationDecision {
            access_authorized: false,
            status: ObservabilityAccessAuthorizationStatus::RejectedExternalSession,
            reason: "metrics endpoint, exporter, and alert delivery sessions require external authentication and remain disabled".to_owned(),
        },
    }
}

fn validate_id(kind: &'static str, id: &str, violations: &mut Vec<ObservabilityViolation>) {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        violations.push(ObservabilityViolation::new_owned(
            "OBSERVABILITY_ID_EMPTY",
            format!("{kind} id must be non-empty"),
        ));
    }
    if trimmed.len() > 128 {
        violations.push(ObservabilityViolation::new_owned(
            "OBSERVABILITY_ID_TOO_LONG",
            format!("{kind} id is too long"),
        ));
    }
    if contains_secret_like_text(trimmed) {
        violations.push(ObservabilityViolation::new_owned(
            "OBSERVABILITY_ID_SECRET_LIKE",
            format!("{kind} id looks like secret material"),
        ));
    }
}

fn validate_name(kind: &'static str, name: &str, violations: &mut Vec<ObservabilityViolation>) {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        violations.push(ObservabilityViolation::new_owned(
            "OBSERVABILITY_NAME_EMPTY",
            format!("{kind} name must be non-empty"),
        ));
    }
    if trimmed.len() > 128 {
        violations.push(ObservabilityViolation::new_owned(
            "OBSERVABILITY_NAME_TOO_LONG",
            format!("{kind} name is too long"),
        ));
    }
    if contains_secret_like_text(trimmed) {
        violations.push(ObservabilityViolation::new_owned(
            "OBSERVABILITY_NAME_SECRET_LIKE",
            format!("{kind} name looks like secret material"),
        ));
    }
}

fn is_loopback_host(host: &str) -> bool {
    matches!(
        host.trim().to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1"
    )
}

fn parse_loopback_ip(host: &str) -> Option<IpAddr> {
    let parsed = host.trim().parse::<IpAddr>().ok()?;
    parsed.is_loopback().then_some(parsed)
}

fn render_prometheus_metric_line(metric: &MetricSample) -> String {
    let metric_name = prometheus_identifier(&format!("{}_microunits", metric.name));
    let labels = metric
        .labels
        .iter()
        .map(|label| {
            format!(
                "{}=\"{}\"",
                prometheus_identifier(&label.key),
                prometheus_label_value(&label.value)
            )
        })
        .collect::<Vec<_>>();
    if labels.is_empty() {
        format!(
            "{} {} {}",
            metric_name, metric.value_microunits, metric.sampled_at_ms
        )
    } else {
        format!(
            "{}{{{}}} {} {}",
            metric_name,
            labels.join(","),
            metric.value_microunits,
            metric.sampled_at_ms
        )
    }
}

fn count_observability_alert_dry_runs(record: &ObservabilityRecord) -> u64 {
    let component_alerts = record
        .components
        .iter()
        .filter(|component| component.status.requires_attention())
        .count();
    let log_alerts = record
        .logs
        .iter()
        .filter(|log| log.severity >= ObservabilitySeverity::Error)
        .count();
    let runbook_alerts = record
        .runbooks
        .iter()
        .filter(|runbook| runbook.severity >= ObservabilitySeverity::Error)
        .count();
    u64::try_from(
        component_alerts
            .saturating_add(log_alerts)
            .saturating_add(runbook_alerts),
    )
    .unwrap_or(u64::MAX)
}

fn prometheus_identifier(value: &str) -> String {
    let mut identifier = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if identifier.is_empty() || identifier.starts_with(|character: char| character.is_ascii_digit())
    {
        identifier.insert(0, '_');
    }
    identifier
}

fn prometheus_label_value(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' | '\r' => " ".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

fn sanitize_observability_text(text: &str, max_chars: usize) -> (String, bool) {
    let mut sanitized = text.trim().to_owned();
    let mut changed = sanitized.len() != text.len();

    if contains_secret_like_text(&sanitized) {
        sanitized = "[REDACTED SECRET-LIKE OBSERVABILITY TEXT]".to_owned();
        changed = true;
    }

    if sanitized.chars().count() > max_chars {
        sanitized = sanitized.chars().take(max_chars).collect::<String>();
        sanitized.push('…');
        changed = true;
    }

    (sanitized, changed)
}

fn contains_secret_like_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let secret_markers = [
        "api_key=",
        "api-key=",
        "apikey=",
        "secret=",
        "private_key=",
        "private-key=",
        "seed_phrase=",
        "seed-phrase=",
        "mnemonic=",
        "bearer ",
        "authorization:",
        "provider token",
        "wallet key",
    ];

    secret_markers.iter().any(|marker| lower.contains(marker)) || looks_like_long_hex(&lower)
}

fn looks_like_long_hex(text: &str) -> bool {
    let longest = text
        .split(|character: char| !character.is_ascii_hexdigit())
        .map(str::len)
        .max()
        .unwrap_or_default();
    longest >= 48
}

#[cfg(test)]
mod tests {
    use super::{
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
        persist_observability_operations_review_checkpoint,
        persist_observability_record_checkpoint, persist_runtime_failure_capture_checkpoint,
        preflight_observability_endpoint, preflight_observability_metrics_scrape,
        record_observability_alert_route_dispatch, render_observability_export_dry_run,
        review_observability_operations, validate_local_tracing_subscriber,
        validate_observability_loopback_bind, validate_observability_metrics_endpoint,
        ComponentHealthStatus, DeterministicObservabilityCollector, HealthStatus,
        LocalTracingSubscriberValidationRequest, LocalTracingSubscriberValidationStatus,
        MetricKind, MetricLabel, MetricSample, ObservabilityAccessAuthorizationStatus,
        ObservabilityAccessContext, ObservabilityAccessSource,
        ObservabilityAlertRouteDispatchRequest, ObservabilityAlertRouteDispatchStatus,
        ObservabilityBoundaryConfig, ObservabilityCollectionRequest, ObservabilityCollector,
        ObservabilityEndpointBinding, ObservabilityEndpointPreflight,
        ObservabilityEndpointPreflightStatus, ObservabilityError, ObservabilityExportDryRunReport,
        ObservabilityExportDryRunRequest, ObservabilityLogRetentionExecutionRequest,
        ObservabilityLoopbackBindValidationReport, ObservabilityLoopbackBindValidationRequest,
        ObservabilityLoopbackBindValidationStatus, ObservabilityMetricsEndpointValidationReport,
        ObservabilityMetricsEndpointValidationRequest,
        ObservabilityMetricsEndpointValidationStatus, ObservabilityMetricsScrapePreflightReport,
        ObservabilityMetricsScrapePreflightRequest, ObservabilityMetricsScrapePreflightStatus,
        ObservabilityOperationsPolicy, ObservabilityOperationsReviewStatus, ObservabilitySeverity,
        ObservabilitySnapshot, Runbook, RunbookStep, RuntimeFailureCaptureRequest,
        RuntimeFailureKind, RuntimePanicHookInstallationRequest, StructuredLogEvent,
        StructuredLogField, OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_LOG_RETENTION_EXECUTION_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY,
        OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY,
    };
    use crate::{
        AppendOnlyAuditJournal, AuditJournalFileMetadata, AuditRetentionExecutionRequest,
        AuditRetentionPolicy, CommunicationChannelKind, NotificationChannelDispatch,
        NotificationChannelDispatchStatus, NotificationDispatchRecord, NotificationDispatchStatus,
        SqliteWalStateStore, StateStore, COMMUNICATIONS_CLI_VERSION,
    };
    use std::{env, fs, panic, panic::AssertUnwindSafe, path::PathBuf, process};

    fn minimal_snapshot() -> ObservabilitySnapshot {
        ObservabilitySnapshot {
            snapshot_id: "observability-snapshot-001".to_owned(),
            generated_at_ms: 1_700_000_000_000,
            components: vec![ComponentHealthStatus::new(
                "policy",
                HealthStatus::Healthy,
                "deny-by-default policy boundary available",
                1_700_000_000_000,
            )],
            logs: vec![StructuredLogEvent::new(
                "log-001",
                ObservabilitySeverity::Info,
                "arb-core",
                "local observability boundary collected one event",
                vec![StructuredLogField::new("phase", "14")],
                1_700_000_000_001,
            )],
            metrics: vec![MetricSample::new(
                "open_gap_count",
                MetricKind::Gauge,
                10_000_000,
                "count",
                vec![MetricLabel::new("source", "governance")],
                1_700_000_000_002,
            )],
            runbooks: vec![Runbook::new(
                "runbook-policy-denial",
                "Policy denial triage",
                ObservabilitySeverity::Warning,
                "policy boundary denied an execution intent",
                vec![RunbookStep::new(
                    1,
                    "Preserve evidence",
                    "Review the local audit record and do not retry with broader permissions",
                )],
            )],
            warnings: Vec::new(),
        }
    }

    fn ready_operations_review() -> super::ObservabilityOperationsReviewReport {
        review_observability_operations(&ObservabilityOperationsPolicy {
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
        .expect("ready operations review should validate")
    }

    fn ready_export_dry_run_report(label: &str) -> ObservabilityExportDryRunReport {
        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: ObservabilityAccessContext::local_collection(Some(label.to_owned())),
                operator_label: Some(label.to_owned()),
                collected_at_ms: 1_700_000_000_520,
            })
            .expect("local collection should succeed");
        let review = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: format!("observability-export-review-{label}"),
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
        .expect("ready operations review should pass");

        render_observability_export_dry_run(ObservabilityExportDryRunRequest {
            record,
            operations_review: review,
            alert_route_references: vec![format!("alert-route-{label}")],
            rendered_at_ms: 1_700_000_000_530,
        })
        .expect("export dry-run should render local metrics")
    }

    fn local_notification_dispatch(label: &str) -> NotificationDispatchRecord {
        NotificationDispatchRecord {
            id: format!("notification-dispatch:observability-alert-{label}"),
            request_id: format!("observability-alert-request-{label}"),
            notification_id: format!("observability-alert-{label}"),
            communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
            status: NotificationDispatchStatus::RecordedLocally,
            redacted_title: "Local observability alert".to_owned(),
            redacted_body: "Local alert route reached the communications boundary".to_owned(),
            created_at_unix_ms: 1_700_000_000_560,
            channels: vec![NotificationChannelDispatch {
                channel_id: "cli".to_owned(),
                kind: CommunicationChannelKind::Cli,
                status: NotificationChannelDispatchStatus::RecordedLocally,
                outbound_network_used: false,
                rate_limited: false,
                outage_blocked: false,
                reason: "local channel recorded without outbound delivery".to_owned(),
            }],
            outbound_network_used: false,
        }
    }

    #[test]
    fn observability_config_rejects_endpoint_and_outbound_alerts() {
        let config = ObservabilityBoundaryConfig {
            metrics_endpoint: ObservabilityEndpointBinding {
                metrics_endpoint_enabled: true,
                bind_host: "0.0.0.0".to_owned(),
                public_network_exposure: true,
                ..ObservabilityEndpointBinding::default()
            },
            outbound_alerts_enabled: true,
            require_local_observability_authorization: false,
            external_observability_sessions_enabled: true,
            ..ObservabilityBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("unsafe observability settings must fail closed");
        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        let codes = violations
            .iter()
            .map(super::ObservabilityViolation::code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"OBSERVABILITY_METRICS_ENDPOINT_DENIED_IN_PHASE_14"));
        assert!(codes.contains(&"OBSERVABILITY_PUBLIC_NETWORK_DENIED_IN_PHASE_14"));
        assert!(codes.contains(&"OBSERVABILITY_BIND_HOST_NOT_LOOPBACK"));
        assert!(codes.contains(&"OBSERVABILITY_OUTBOUND_ALERTS_DENIED_IN_PHASE_14"));
        assert!(codes.contains(&"OBSERVABILITY_LOCAL_AUTH_REQUIRED"));
        assert!(codes.contains(&"OBSERVABILITY_EXTERNAL_SESSIONS_DENIED_IN_PHASE_14"));
    }

    #[test]
    fn collector_never_starts_endpoint_or_sends_alerts() {
        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: ObservabilityAccessContext::local_collection(Some(
                    "local-collector".to_owned(),
                )),
                operator_label: Some("local-operator".to_owned()),
                collected_at_ms: 1_700_000_000_100,
            })
            .expect("local collection should succeed");

        assert_eq!(record.overall_health, HealthStatus::Healthy);
        assert!(!record.metrics_endpoint_started);
        assert!(!record.public_network_exposed);
        assert!(!record.outbound_alerts_sent);
        assert!(record.access_authorized);
        assert_eq!(
            record.access_authorization_status,
            ObservabilityAccessAuthorizationStatus::AuthorizedLocalCollection
        );
        assert_eq!(record.logs.len(), 1);
        assert_eq!(record.metrics.len(), 1);
        assert_eq!(record.runbooks.len(), 1);
    }

    #[test]
    fn collector_rejects_external_observability_sessions_without_auth() {
        let collector = DeterministicObservabilityCollector;
        let error = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: ObservabilityAccessContext {
                    source: ObservabilityAccessSource::ExporterSession,
                    collector_label: Some("otel-exporter".to_owned()),
                },
                operator_label: Some("local-operator".to_owned()),
                collected_at_ms: 1_700_000_000_150,
            })
            .expect_err("external observability sessions must fail closed without auth");

        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations
            .iter()
            .any(|violation| violation.code() == "OBSERVABILITY_ACCESS_DENIED"));
    }

    #[test]
    fn collector_redacts_secret_like_observability_text() {
        let mut snapshot = minimal_snapshot();
        snapshot.logs[0].fields.push(StructuredLogField::new(
            "diagnostic",
            concat!("api", "_key=", "not-a-real-value-for-test-only"),
        ));

        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot,
                access: ObservabilityAccessContext::local_collection(None),
                operator_label: None,
                collected_at_ms: 1_700_000_000_200,
            })
            .expect("local collection should redact secret-like text");

        assert!(record.secret_redaction_applied);
        assert_eq!(
            record.logs[0].fields[1].value,
            "[REDACTED SECRET-LIKE OBSERVABILITY TEXT]"
        );
    }

    #[test]
    fn observability_record_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-record");
        let state_path = temp_state_path("observability-record");
        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: ObservabilityAccessContext::local_collection(Some(
                    "local-observability-review".to_owned(),
                )),
                operator_label: Some("local-observability-review".to_owned()),
                collected_at_ms: 1_700_000_000_300,
            })
            .expect("local collection should succeed");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_observability_record_audit(&mut journal, &record, 1_700_000_000_301)
                .expect("observability record audit writes");
        let checkpoint =
            persist_observability_record_checkpoint(&mut store, &record, 1_700_000_000_302)
                .expect("observability record checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY);
        assert!(!record.metrics_endpoint_started);
        assert!(!record.public_network_exposed);
        assert!(!record.outbound_alerts_sent);
        assert!(record.access_authorized);
        assert_eq!(
            record.access_authorization_status,
            ObservabilityAccessAuthorizationStatus::AuthorizedLocalCollection
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_RECORD_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("observability checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"metrics_endpoint_started\":false"));
        assert!(recovered.value.contains("\"public_network_exposed\":false"));
        assert!(recovered.value.contains("\"outbound_alerts_sent\":false"));
        assert!(recovered.value.contains("\"access_authorized\":true"));
        assert!(recovered
            .value
            .contains("\"access_authorization_status\":\"authorized-local-collection\""));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_operations_review_requires_retention_alert_routes_and_runbooks() {
        let report = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: "observability-operations-ready".to_owned(),
            log_retention_required: true,
            retention_days: 30,
            redaction_required: true,
            alert_routing_required: true,
            alert_route_count: 2,
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
        .expect("complete local observability operations policy should review");

        assert_eq!(
            report.status,
            ObservabilityOperationsReviewStatus::ReadyForLocalReview
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(report.log_retention_required);
        assert_eq!(report.retention_days, 30);
        assert!(report.alert_routing_required);
        assert_eq!(report.alert_route_count, 2);
        assert!(report.incident_runbook_required);
        assert_eq!(report.incident_runbook_count, 1);
        assert!(report.audit_state_preflight_required);
        assert!(report.exporter_kill_switch_required);
        assert!(report.alert_authorization_required);
        assert!(report.rate_limit_backpressure_required);
        assert!(report.retry_backoff_required);
        assert!(report.no_secret_telemetry_required);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.telemetry_exported);
        assert!(!report.production_ready);
    }

    #[test]
    fn observability_operations_review_blocks_missing_controls_and_side_effect_requests() {
        let report = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: "observability-operations-blocked".to_owned(),
            log_retention_required: false,
            retention_days: 0,
            redaction_required: false,
            alert_routing_required: false,
            alert_route_count: 0,
            incident_runbook_required: false,
            incident_runbook_count: 0,
            loopback_or_authenticated_endpoint_required: false,
            audit_state_preflight_required: false,
            exporter_kill_switch_required: false,
            alert_authorization_required: false,
            rate_limit_backpressure_required: false,
            retry_backoff_required: false,
            no_secret_telemetry_required: false,
            metrics_endpoint_requested: true,
            outbound_alert_delivery_requested: true,
            telemetry_export_requested: true,
        })
        .expect("incomplete local observability operations policy should produce blocked report");

        assert_eq!(
            report.status,
            ObservabilityOperationsReviewStatus::BlockedMissingControls
        );
        assert!(report.missing_control_count > 0);
        assert!(!report.log_retention_required);
        assert!(!report.alert_routing_required);
        assert!(!report.incident_runbook_required);
        assert!(!report.audit_state_preflight_required);
        assert!(!report.exporter_kill_switch_required);
        assert!(!report.alert_authorization_required);
        assert!(!report.rate_limit_backpressure_required);
        assert!(!report.retry_backoff_required);
        assert!(!report.no_secret_telemetry_required);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.telemetry_exported);
        assert!(!report.production_ready);
    }

    #[test]
    fn observability_operations_review_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-operations-review");
        let state_path = temp_state_path("observability-operations-review");
        let report = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: "observability-operations-audit-state".to_owned(),
            log_retention_required: true,
            retention_days: 14,
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
        .expect("observability operations review should produce local report");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_observability_operations_review_audit(&mut journal, &report, 1_700_000_000_351)
                .expect("operations review audit writes");
        let checkpoint = persist_observability_operations_review_checkpoint(
            &mut store,
            &report,
            1_700_000_000_352,
        )
        .expect("operations review checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY
        );
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.telemetry_exported);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_OPERATIONS_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("operations review checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"log_retention_required\":true"));
        assert!(recovered.value.contains("\"alert_routing_required\":true"));
        assert!(recovered
            .value
            .contains("\"incident_runbook_required\":true"));
        assert!(recovered
            .value
            .contains("\"audit_state_preflight_required\":true"));
        assert!(recovered
            .value
            .contains("\"exporter_kill_switch_required\":true"));
        assert!(recovered
            .value
            .contains("\"alert_authorization_required\":true"));
        assert!(recovered
            .value
            .contains("\"rate_limit_backpressure_required\":true"));
        assert!(recovered.value.contains("\"retry_backoff_required\":true"));
        assert!(recovered
            .value
            .contains("\"no_secret_telemetry_required\":true"));
        assert!(recovered.value.contains("\"outbound_alerts_sent\":false"));
        assert!(recovered.value.contains("\"telemetry_exported\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_export_dry_run_renders_metrics_and_alert_accounting_locally() {
        let collector = DeterministicObservabilityCollector;
        let mut snapshot = minimal_snapshot();
        snapshot.components[0] = ComponentHealthStatus::new(
            "runtime",
            HealthStatus::Unhealthy,
            "local runtime requires operator review",
            1_700_000_000_500,
        );
        snapshot.logs[0] = StructuredLogEvent::new(
            "log-critical-001",
            ObservabilitySeverity::Error,
            "runtime",
            "local runtime failure dry-run alert",
            vec![StructuredLogField::new("route", "operator-primary")],
            1_700_000_000_501,
        );
        snapshot.runbooks[0] = Runbook::new(
            "runbook-runtime-failure",
            "Runtime failure triage",
            ObservabilitySeverity::Critical,
            "runtime failure capture requires review",
            vec![RunbookStep::new(
                1,
                "Inspect local evidence",
                "Review audit and SQLite checkpoint references before any restart",
            )],
        );
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot,
                access: ObservabilityAccessContext::local_collection(Some(
                    "local-export-dry-run".to_owned(),
                )),
                operator_label: Some("local export dry run".to_owned()),
                collected_at_ms: 1_700_000_000_510,
            })
            .expect("local observability record should collect");
        let operations_review = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: "observability-export-review".to_owned(),
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
        .expect("operations review should be ready locally");

        let report = render_observability_export_dry_run(ObservabilityExportDryRunRequest {
            record,
            operations_review,
            alert_route_references: vec!["ops-alert-route-primary".to_owned()],
            rendered_at_ms: 1_700_000_000_520,
        })
        .expect("local export dry run should render");

        assert_eq!(report.snapshot_id, "observability-snapshot-001");
        assert_eq!(report.review_id, "observability-export-review");
        assert_eq!(report.prometheus_metric_lines.len(), 1);
        assert_eq!(
            report.prometheus_metric_lines[0],
            "open_gap_count_microunits{source=\"governance\"} 10000000 1700000000002"
        );
        assert_eq!(report.log_event_count, 1);
        assert_eq!(report.runbook_count, 1);
        assert_eq!(report.alert_route_count, 1);
        assert_eq!(report.alert_dry_run_count, 3);
        assert!(report.loopback_or_authenticated_endpoint_required);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.telemetry_exported);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn observability_export_dry_run_audits_and_reopens_locally() {
        let audit_path = temp_audit_path("observability-export-dry-run");
        let state_path = temp_state_path("observability-export-dry-run");
        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: ObservabilityAccessContext::local_collection(Some(
                    "local-export-audit".to_owned(),
                )),
                operator_label: Some("local export audit".to_owned()),
                collected_at_ms: 1_700_000_000_530,
            })
            .expect("local observability record should collect");
        let operations_review = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: "observability-export-audit-review".to_owned(),
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
        .expect("operations review should be ready locally");
        let report = render_observability_export_dry_run(ObservabilityExportDryRunRequest {
            record,
            operations_review,
            alert_route_references: vec!["ops-alert-route-primary".to_owned()],
            rendered_at_ms: 1_700_000_000_540,
        })
        .expect("local export dry run should render");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_observability_export_dry_run_audit(&mut journal, &report, 1_700_000_000_541)
                .expect("export dry-run audit writes");
        let checkpoint =
            persist_observability_export_dry_run_checkpoint(&mut store, &report, 1_700_000_000_542)
                .expect("export dry-run checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_EXPORT_DRY_RUN_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("export dry-run checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"metrics_endpoint_started\":false"));
        assert!(recovered.value.contains("\"outbound_alerts_sent\":false"));
        assert!(recovered.value.contains("\"telemetry_exported\":false"));
        assert!(recovered
            .value
            .contains("\"live_execution_performed\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_export_dry_run_rejects_blocked_review() {
        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: ObservabilityAccessContext::local_collection(None),
                operator_label: None,
                collected_at_ms: 1_700_000_000_550,
            })
            .expect("local observability record should collect");
        let blocked_review = review_observability_operations(&ObservabilityOperationsPolicy {
            review_id: "observability-export-blocked-review".to_owned(),
            log_retention_required: false,
            retention_days: 0,
            redaction_required: false,
            alert_routing_required: false,
            alert_route_count: 0,
            incident_runbook_required: false,
            incident_runbook_count: 0,
            loopback_or_authenticated_endpoint_required: false,
            audit_state_preflight_required: false,
            exporter_kill_switch_required: false,
            alert_authorization_required: false,
            rate_limit_backpressure_required: false,
            retry_backoff_required: false,
            no_secret_telemetry_required: false,
            metrics_endpoint_requested: true,
            outbound_alert_delivery_requested: true,
            telemetry_export_requested: true,
        })
        .expect("blocked operations review should still be reportable");

        let error = render_observability_export_dry_run(ObservabilityExportDryRunRequest {
            record,
            operations_review: blocked_review,
            alert_route_references: Vec::new(),
            rendered_at_ms: 1_700_000_000_560,
        })
        .expect_err("blocked operations review should fail export dry-run");

        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations.iter().any(|violation| {
            violation.code() == "OBSERVABILITY_EXPORT_DRY_RUN_REVIEW_NOT_READY"
        }));
    }

    #[test]
    fn observability_alert_route_dispatch_records_local_communications_boundary() {
        let report =
            record_observability_alert_route_dispatch(ObservabilityAlertRouteDispatchRequest {
                dispatch_review_id: "observability-alert-route-dispatch-local".to_owned(),
                export_report: ready_export_dry_run_report("dispatch-local"),
                alert_route_reference: "alert-route-dispatch-local".to_owned(),
                notification_dispatch: local_notification_dispatch("dispatch-local"),
                local_dispatch_required: true,
                outbound_alert_delivery_requested: false,
                reviewed_at_ms: 1_700_000_000_570,
            })
            .expect("local alert-route dispatch should validate");

        assert_eq!(
            report.status,
            ObservabilityAlertRouteDispatchStatus::ReadyForLocalReview
        );
        assert_eq!(report.recorded_local_channel_count, 1);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.outbound_network_used);
        assert!(!report.telemetry_exported);
        assert!(!report.production_ready);
    }

    #[test]
    fn observability_alert_route_dispatch_rejects_outbound_or_blocked_dispatch() {
        let mut dispatch = local_notification_dispatch("dispatch-blocked");
        dispatch.status = NotificationDispatchStatus::BlockedOutboundNetwork;

        let error =
            record_observability_alert_route_dispatch(ObservabilityAlertRouteDispatchRequest {
                dispatch_review_id: "observability-alert-route-dispatch-blocked".to_owned(),
                export_report: ready_export_dry_run_report("dispatch-blocked"),
                alert_route_reference: "alert-route-dispatch-blocked".to_owned(),
                notification_dispatch: dispatch,
                local_dispatch_required: true,
                outbound_alert_delivery_requested: true,
                reviewed_at_ms: 1_700_000_000_571,
            })
            .expect_err("unsafe alert-route dispatch must fail closed");

        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        let codes = violations
            .iter()
            .map(super::ObservabilityViolation::code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"OBSERVABILITY_ALERT_ROUTE_OUTBOUND_DELIVERY_REQUESTED"));
        assert!(codes.contains(&"OBSERVABILITY_ALERT_ROUTE_DISPATCH_NOT_LOCAL"));
    }

    #[test]
    fn observability_alert_route_dispatch_audit_and_checkpoint_reopen_locally() {
        let report =
            record_observability_alert_route_dispatch(ObservabilityAlertRouteDispatchRequest {
                dispatch_review_id: "observability-alert-route-dispatch-audit".to_owned(),
                export_report: ready_export_dry_run_report("dispatch-audit"),
                alert_route_reference: "alert-route-dispatch-audit".to_owned(),
                notification_dispatch: local_notification_dispatch("dispatch-audit"),
                local_dispatch_required: true,
                outbound_alert_delivery_requested: false,
                reviewed_at_ms: 1_700_000_000_572,
            })
            .expect("local alert-route dispatch should validate");
        let audit_path = temp_audit_path("observability-alert-route-dispatch");
        let state_path = temp_state_path("observability-alert-route-dispatch");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_observability_alert_route_dispatch_audit(
            &mut journal,
            &report,
            1_700_000_000_573,
        )
        .expect("alert-route dispatch audit writes");
        let checkpoint = persist_observability_alert_route_dispatch_checkpoint(
            &mut store,
            &report,
            1_700_000_000_574,
        )
        .expect("alert-route dispatch checkpoint writes");
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("alert-route dispatch checkpoint exists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(replayed.next_sequence(), 2);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_ALERT_ROUTE_DISPATCH_CHECKPOINT_KEY
        );
        assert!(recovered.value.contains("\"outbound_alerts_sent\":false"));
        assert!(recovered.value.contains("\"outbound_network_used\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_metrics_scrape_preflight_accepts_authenticated_loopback_get() {
        let export_report = ready_export_dry_run_report("metrics-scrape-ready");
        let expected_lines = export_report.prometheus_metric_lines.clone();

        let report =
            preflight_observability_metrics_scrape(ObservabilityMetricsScrapePreflightRequest {
                scrape_id: "metrics-scrape-ready".to_owned(),
                export_report,
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
            .expect("authenticated loopback scrape preflight should pass");

        assert_eq!(
            report.status,
            ObservabilityMetricsScrapePreflightStatus::ReadyForLocalReview
        );
        assert_eq!(report.request_method, "GET");
        assert_eq!(report.request_path, "/metrics");
        assert_eq!(report.source_host, "127.0.0.1");
        assert!(report.loopback_source_validated);
        assert!(report.authentication_required);
        assert!(report.authorization_required);
        assert!(report.bearer_token_reference_present);
        assert_eq!(report.local_http_status_code, 200);
        assert_eq!(report.response_metric_lines, expected_lines);
        assert_eq!(
            report.response_metric_line_count,
            u64::try_from(report.response_metric_lines.len()).expect("line count fits u64")
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.network_request_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn observability_metrics_scrape_preflight_blocks_missing_auth_public_or_side_effect_requests() {
        let report =
            preflight_observability_metrics_scrape(ObservabilityMetricsScrapePreflightRequest {
                scrape_id: "metrics-scrape-blocked".to_owned(),
                export_report: ready_export_dry_run_report("metrics-scrape-blocked"),
                request_method: "POST".to_owned(),
                request_path: "/admin".to_owned(),
                source_host: "0.0.0.0".to_owned(),
                authentication_required: false,
                authorization_required: false,
                bearer_token_reference_present: false,
                metrics_endpoint_start_requested: false,
                public_network_exposure_requested: false,
                telemetry_export_requested: false,
                outbound_alert_delivery_requested: false,
            })
            .expect("unsafe scrape shape should produce a blocked local report");

        assert_eq!(
            report.status,
            ObservabilityMetricsScrapePreflightStatus::Blocked
        );
        assert_eq!(report.local_http_status_code, 403);
        assert!(!report.loopback_source_validated);
        assert!(!report.authentication_required);
        assert!(!report.authorization_required);
        assert!(!report.bearer_token_reference_present);
        assert!(report.response_metric_lines.is_empty());
        assert!(report.missing_control_count >= 6);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.network_request_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);

        let error =
            preflight_observability_metrics_scrape(ObservabilityMetricsScrapePreflightRequest {
                scrape_id: "metrics-scrape-side-effect-blocked".to_owned(),
                export_report: ready_export_dry_run_report("metrics-scrape-side-effect-blocked"),
                request_method: "GET".to_owned(),
                request_path: "/metrics".to_owned(),
                source_host: "127.0.0.1".to_owned(),
                authentication_required: true,
                authorization_required: true,
                bearer_token_reference_present: true,
                metrics_endpoint_start_requested: true,
                public_network_exposure_requested: true,
                telemetry_export_requested: true,
                outbound_alert_delivery_requested: true,
            })
            .expect_err("side-effect scrape request should fail before serving");
        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations.iter().any(|violation| {
            violation.code() == "OBSERVABILITY_METRICS_SCRAPE_SIDE_EFFECT_REQUESTED"
        }));
    }

    #[test]
    fn observability_metrics_scrape_preflight_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-metrics-scrape-preflight");
        let state_path = temp_state_path("observability-metrics-scrape-preflight");
        let report =
            preflight_observability_metrics_scrape(ObservabilityMetricsScrapePreflightRequest {
                scrape_id: "metrics-scrape-audit-state".to_owned(),
                export_report: ready_export_dry_run_report("metrics-scrape-audit-state"),
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
            .expect("authenticated loopback scrape preflight should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_observability_metrics_scrape_preflight_audit(
            &mut journal,
            &report,
            1_700_000_000_621,
        )
        .expect("metrics scrape preflight audit writes");
        let checkpoint = persist_observability_metrics_scrape_preflight_checkpoint(
            &mut store,
            &report,
            1_700_000_000_622,
        )
        .expect("metrics scrape preflight checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY
        );
        assert!(!report.metrics_endpoint_started);
        assert!(!report.network_request_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_METRICS_SCRAPE_PREFLIGHT_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("metrics scrape preflight checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: ObservabilityMetricsScrapePreflightReport =
            serde_json::from_str(&recovered.value).expect("metrics scrape checkpoint parses");
        assert_eq!(
            recovered_report.status,
            ObservabilityMetricsScrapePreflightStatus::ReadyForLocalReview
        );
        assert_eq!(recovered_report.local_http_status_code, 200);
        assert!(recovered_report.loopback_source_validated);
        assert!(!recovered_report.response_metric_lines.is_empty());
        assert!(!recovered_report.metrics_endpoint_started);
        assert!(!recovered_report.network_request_served);
        assert!(!recovered_report.public_network_exposed);
        assert!(!recovered_report.telemetry_exported);
        assert!(!recovered_report.outbound_alerts_sent);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_metrics_endpoint_serves_authenticated_loopback_scrape() {
        let export_report = ready_export_dry_run_report("metrics-endpoint-ready");
        let expected_lines = export_report.prometheus_metric_lines.clone();

        let report = validate_observability_metrics_endpoint(
            ObservabilityMetricsEndpointValidationRequest {
                validation_id: "metrics-endpoint-ready".to_owned(),
                export_report,
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
            },
        )
        .expect("authenticated loopback metrics endpoint should pass");

        assert_eq!(
            report.status,
            ObservabilityMetricsEndpointValidationStatus::ReadyForLocalReview
        );
        assert_eq!(report.bind_host, "127.0.0.1");
        assert_eq!(report.request_method, "GET");
        assert_eq!(report.request_path, "/metrics");
        assert!(report.bound_port.is_some());
        assert!(report.loopback_bind_validated);
        assert!(report.authentication_required);
        assert!(report.authorization_required);
        assert!(report.bearer_token_reference_present);
        assert_eq!(report.local_http_status_code, 200);
        assert_eq!(report.response_metric_lines, expected_lines);
        assert!(report.local_metrics_endpoint_started);
        assert!(report.network_request_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn observability_metrics_endpoint_blocks_public_missing_auth_or_side_effect_requests() {
        let report = validate_observability_metrics_endpoint(
            ObservabilityMetricsEndpointValidationRequest {
                validation_id: "metrics-endpoint-blocked".to_owned(),
                export_report: ready_export_dry_run_report("metrics-endpoint-blocked"),
                bind_host: "0.0.0.0".to_owned(),
                requested_port: 0,
                request_method: "POST".to_owned(),
                request_path: "/admin".to_owned(),
                loopback_only_required: true,
                authentication_required: false,
                authorization_required: false,
                bearer_token_reference_present: false,
                public_network_exposure_requested: false,
                telemetry_export_requested: false,
                outbound_alert_delivery_requested: false,
            },
        )
        .expect("unsafe endpoint shape should produce a blocked local report");

        assert_eq!(
            report.status,
            ObservabilityMetricsEndpointValidationStatus::Blocked
        );
        assert_eq!(report.local_http_status_code, 403);
        assert!(report.bound_port.is_none());
        assert!(!report.loopback_bind_validated);
        assert!(!report.authentication_required);
        assert!(!report.authorization_required);
        assert!(!report.bearer_token_reference_present);
        assert!(!report.local_metrics_endpoint_started);
        assert!(!report.network_request_served);
        assert!(report.response_metric_lines.is_empty());
        assert!(report.missing_control_count >= 6);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);

        let error = validate_observability_metrics_endpoint(
            ObservabilityMetricsEndpointValidationRequest {
                validation_id: "metrics-endpoint-side-effect-blocked".to_owned(),
                export_report: ready_export_dry_run_report("metrics-endpoint-side-effect-blocked"),
                bind_host: "127.0.0.1".to_owned(),
                requested_port: 0,
                request_method: "GET".to_owned(),
                request_path: "/metrics".to_owned(),
                loopback_only_required: true,
                authentication_required: true,
                authorization_required: true,
                bearer_token_reference_present: true,
                public_network_exposure_requested: true,
                telemetry_export_requested: true,
                outbound_alert_delivery_requested: true,
            },
        )
        .expect_err("side-effect endpoint request should fail before binding");
        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations.iter().any(|violation| {
            violation.code() == "OBSERVABILITY_METRICS_ENDPOINT_SIDE_EFFECT_REQUESTED"
        }));
    }

    #[test]
    fn observability_metrics_endpoint_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-metrics-endpoint-validation");
        let state_path = temp_state_path("observability-metrics-endpoint-validation");
        let report = validate_observability_metrics_endpoint(
            ObservabilityMetricsEndpointValidationRequest {
                validation_id: "metrics-endpoint-audit-state".to_owned(),
                export_report: ready_export_dry_run_report("metrics-endpoint-audit-state"),
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
            },
        )
        .expect("authenticated loopback metrics endpoint should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_observability_metrics_endpoint_validation_audit(
            &mut journal,
            &report,
            1_700_000_000_631,
        )
        .expect("metrics endpoint validation audit writes");
        let checkpoint = persist_observability_metrics_endpoint_validation_checkpoint(
            &mut store,
            &report,
            1_700_000_000_632,
        )
        .expect("metrics endpoint validation checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY
        );
        assert!(report.local_metrics_endpoint_started);
        assert!(report.network_request_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_METRICS_ENDPOINT_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("metrics endpoint validation checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: ObservabilityMetricsEndpointValidationReport =
            serde_json::from_str(&recovered.value).expect("metrics endpoint checkpoint parses");
        assert_eq!(
            recovered_report.status,
            ObservabilityMetricsEndpointValidationStatus::ReadyForLocalReview
        );
        assert_eq!(recovered_report.local_http_status_code, 200);
        assert!(recovered_report.loopback_bind_validated);
        assert!(recovered_report.local_metrics_endpoint_started);
        assert!(recovered_report.network_request_served);
        assert!(!recovered_report.response_metric_lines.is_empty());
        assert!(!recovered_report.public_network_exposed);
        assert!(!recovered_report.telemetry_exported);
        assert!(!recovered_report.outbound_alerts_sent);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_endpoint_preflight_accepts_loopback_auth_routes_and_backpressure() {
        let report = preflight_observability_endpoint(&ObservabilityEndpointPreflight {
            preflight_id: "observability-endpoint-preflight-ready".to_owned(),
            bind_host: "127.0.0.1".to_owned(),
            bind_port: 9_090,
            loopback_only_required: true,
            authentication_required: true,
            authorization_required: true,
            transport_protection_required: true,
            redaction_required: true,
            alert_routes_configured: true,
            alert_route_count: 2,
            exporter_backpressure_required: true,
            metrics_endpoint_start_requested: false,
            public_network_exposure_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
        })
        .expect("complete endpoint preflight should review locally");

        assert_eq!(
            report.status,
            ObservabilityEndpointPreflightStatus::ReadyForLocalReview
        );
        assert!(report.loopback_bind_validated);
        assert!(report.authentication_required);
        assert!(report.authorization_required);
        assert!(report.transport_protection_required);
        assert!(report.redaction_required);
        assert!(report.alert_routes_configured);
        assert_eq!(report.alert_route_count, 2);
        assert!(report.exporter_backpressure_required);
        assert_eq!(report.missing_control_count, 0);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
    }

    #[test]
    fn observability_endpoint_preflight_blocks_public_missing_auth_and_side_effect_requests() {
        let report = preflight_observability_endpoint(&ObservabilityEndpointPreflight {
            preflight_id: "observability-endpoint-preflight-blocked".to_owned(),
            bind_host: "0.0.0.0".to_owned(),
            bind_port: 9_090,
            loopback_only_required: false,
            authentication_required: false,
            authorization_required: false,
            transport_protection_required: false,
            redaction_required: false,
            alert_routes_configured: false,
            alert_route_count: 0,
            exporter_backpressure_required: false,
            metrics_endpoint_start_requested: true,
            public_network_exposure_requested: true,
            telemetry_export_requested: true,
            outbound_alert_delivery_requested: true,
        })
        .expect("unsafe endpoint preflight should produce blocked report");

        assert_eq!(
            report.status,
            ObservabilityEndpointPreflightStatus::BlockedMissingControls
        );
        assert!(!report.loopback_bind_validated);
        assert!(!report.authentication_required);
        assert!(!report.authorization_required);
        assert!(!report.transport_protection_required);
        assert!(!report.redaction_required);
        assert!(!report.alert_routes_configured);
        assert!(!report.exporter_backpressure_required);
        assert!(report.missing_control_count >= 10);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
    }

    #[test]
    fn observability_loopback_bind_validation_opens_and_closes_ephemeral_loopback_listener() {
        let report =
            validate_observability_loopback_bind(&ObservabilityLoopbackBindValidationRequest {
                validation_id: "loopback-bind-local".to_owned(),
                bind_host: "127.0.0.1".to_owned(),
                requested_port: 0,
                loopback_only_required: true,
                serve_requests_requested: false,
                telemetry_export_requested: false,
                outbound_alert_delivery_requested: false,
            })
            .expect("local loopback bind validation should pass");

        assert_eq!(
            report.status,
            ObservabilityLoopbackBindValidationStatus::ReadyForLocalReview
        );
        assert_eq!(report.bind_host, "127.0.0.1");
        assert_eq!(report.requested_port, 0);
        assert!(report.bound_port.is_some_and(|port| port > 0));
        assert!(report.loopback_bind_validated);
        assert!(report.listener_opened_and_closed);
        assert_eq!(report.missing_control_count, 0);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.requests_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn observability_loopback_bind_validation_blocks_public_or_side_effect_requests() {
        let report =
            validate_observability_loopback_bind(&ObservabilityLoopbackBindValidationRequest {
                validation_id: "loopback-bind-public-blocked".to_owned(),
                bind_host: "0.0.0.0".to_owned(),
                requested_port: 0,
                loopback_only_required: true,
                serve_requests_requested: false,
                telemetry_export_requested: false,
                outbound_alert_delivery_requested: false,
            })
            .expect("public bind host should produce a blocked local report");

        assert_eq!(
            report.status,
            ObservabilityLoopbackBindValidationStatus::Blocked
        );
        assert_eq!(report.bound_port, None);
        assert!(!report.loopback_bind_validated);
        assert!(!report.listener_opened_and_closed);
        assert!(report.missing_control_count > 0);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.requests_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);

        let error =
            validate_observability_loopback_bind(&ObservabilityLoopbackBindValidationRequest {
                validation_id: "loopback-bind-side-effect-blocked".to_owned(),
                bind_host: "127.0.0.1".to_owned(),
                requested_port: 0,
                loopback_only_required: true,
                serve_requests_requested: true,
                telemetry_export_requested: true,
                outbound_alert_delivery_requested: true,
            })
            .expect_err("side-effect request should fail before binding");
        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations.iter().any(|violation| {
            violation.code() == "OBSERVABILITY_LOOPBACK_BIND_SIDE_EFFECT_REQUESTED"
        }));
    }

    #[test]
    fn observability_loopback_bind_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-loopback-bind-validation");
        let state_path = temp_state_path("observability-loopback-bind-validation");
        let report =
            validate_observability_loopback_bind(&ObservabilityLoopbackBindValidationRequest {
                validation_id: "loopback-bind-audit-state".to_owned(),
                bind_host: "127.0.0.1".to_owned(),
                requested_port: 0,
                loopback_only_required: true,
                serve_requests_requested: false,
                telemetry_export_requested: false,
                outbound_alert_delivery_requested: false,
            })
            .expect("local loopback bind validation should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_observability_loopback_bind_validation_audit(
            &mut journal,
            &report,
            1_700_000_000_611,
        )
        .expect("loopback bind validation audit writes");
        let checkpoint = persist_observability_loopback_bind_validation_checkpoint(
            &mut store,
            &report,
            1_700_000_000_612,
        )
        .expect("loopback bind validation checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY
        );
        assert!(!report.metrics_endpoint_started);
        assert!(!report.requests_served);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_LOOPBACK_BIND_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("loopback bind validation checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: ObservabilityLoopbackBindValidationReport =
            serde_json::from_str(&recovered.value).expect("loopback bind checkpoint parses");
        assert_eq!(
            recovered_report.status,
            ObservabilityLoopbackBindValidationStatus::ReadyForLocalReview
        );
        assert!(recovered_report.loopback_bind_validated);
        assert!(recovered_report.listener_opened_and_closed);
        assert!(!recovered_report.metrics_endpoint_started);
        assert!(!recovered_report.requests_served);
        assert!(!recovered_report.public_network_exposed);
        assert!(!recovered_report.telemetry_exported);
        assert!(!recovered_report.outbound_alerts_sent);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_endpoint_preflight_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-endpoint-preflight");
        let state_path = temp_state_path("observability-endpoint-preflight");
        let report = preflight_observability_endpoint(&ObservabilityEndpointPreflight {
            preflight_id: "observability-endpoint-preflight-audit-state".to_owned(),
            bind_host: "localhost".to_owned(),
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
        .expect("endpoint preflight should produce local report");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_observability_endpoint_preflight_audit(&mut journal, &report, 1_700_000_000_601)
                .expect("endpoint preflight audit writes");
        let checkpoint = persist_observability_endpoint_preflight_checkpoint(
            &mut store,
            &report,
            1_700_000_000_602,
        )
        .expect("endpoint preflight checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY
        );
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_ENDPOINT_PREFLIGHT_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("endpoint preflight checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"loopback_bind_validated\":true"));
        assert!(recovered.value.contains("\"authentication_required\":true"));
        assert!(recovered.value.contains("\"authorization_required\":true"));
        assert!(recovered
            .value
            .contains("\"exporter_backpressure_required\":true"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn observability_log_retention_executes_only_inside_local_sandbox() {
        let workspace = temp_dir("observability-log-retention");
        let active = workspace.join("observability-active.log");
        let retained = workspace.join("observability-retained.log");
        let expired = workspace.join("observability-expired.log");
        write_local_log_fixture(&active, "active");
        write_local_log_fixture(&retained, "retained");
        write_local_log_fixture(&expired, "expired");

        let report =
            execute_local_observability_log_retention(&ObservabilityLogRetentionExecutionRequest {
                execution_id: "observability-log-retention".to_owned(),
                operations_review: ready_operations_review(),
                retention_request: AuditRetentionExecutionRequest {
                    workspace_dir: workspace.clone(),
                    policy: AuditRetentionPolicy {
                        max_active_bytes: 1,
                        max_archived_files: 1,
                        retention_window_ms: 1_000,
                    },
                    files: vec![
                        log_file_metadata(&active, 10_000, true),
                        log_file_metadata(&retained, 9_500, false),
                        log_file_metadata(&expired, 8_000, false),
                    ],
                    now_unix_ms: 10_000,
                },
                local_sandbox_only: true,
                production_log_paths_requested: false,
                service_manager_action_requested: false,
                external_log_shipping_requested: false,
            })
            .expect("local observability log retention should execute in sandbox");

        assert!(report.operations_review_ready);
        assert!(report.rotate_active_requested);
        assert!(report.new_active_created);
        assert!(report.sandbox_filesystem_mutated);
        assert!(report.sandbox_deletion_performed);
        assert_eq!(report.deleted_file_count, 1);
        assert!(active.exists());
        assert!(retained.exists());
        assert!(!expired.exists());
        assert!(!report.out_of_workspace_path_touched);
        assert!(!report.production_log_paths_touched);
        assert!(!report.service_manager_action_performed);
        assert!(!report.external_log_shipping_performed);
        assert!(!report.live_network_used);
        assert!(!report.production_ready);

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn observability_log_retention_rejects_production_or_shipping_requests() {
        let workspace = temp_dir("observability-log-retention-blocked");
        let active = workspace.join("observability-active.log");
        write_local_log_fixture(&active, "active");

        let error =
            execute_local_observability_log_retention(&ObservabilityLogRetentionExecutionRequest {
                execution_id: "observability-log-retention-blocked".to_owned(),
                operations_review: ready_operations_review(),
                retention_request: AuditRetentionExecutionRequest {
                    workspace_dir: workspace.clone(),
                    policy: AuditRetentionPolicy {
                        max_active_bytes: 1,
                        max_archived_files: 1,
                        retention_window_ms: 1_000,
                    },
                    files: vec![log_file_metadata(&active, 10_000, true)],
                    now_unix_ms: 10_000,
                },
                local_sandbox_only: false,
                production_log_paths_requested: true,
                service_manager_action_requested: false,
                external_log_shipping_requested: true,
            })
            .expect_err("production or external log shipping request should fail closed");

        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations.iter().any(|violation| {
            violation.code() == "OBSERVABILITY_LOG_RETENTION_SIDE_EFFECT_REQUESTED"
        }));
        assert!(active.exists());

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn observability_log_retention_audit_and_state_reopen_locally() {
        let workspace = temp_dir("observability-log-retention-audit");
        let audit_path = temp_audit_path("observability-log-retention");
        let state_path = temp_state_path("observability-log-retention");
        let active = workspace.join("observability-active.log");
        let expired = workspace.join("observability-expired.log");
        write_local_log_fixture(&active, "active");
        write_local_log_fixture(&expired, "expired");

        let report =
            execute_local_observability_log_retention(&ObservabilityLogRetentionExecutionRequest {
                execution_id: "observability-log-retention-audit".to_owned(),
                operations_review: ready_operations_review(),
                retention_request: AuditRetentionExecutionRequest {
                    workspace_dir: workspace.clone(),
                    policy: AuditRetentionPolicy {
                        max_active_bytes: 1,
                        max_archived_files: 0,
                        retention_window_ms: 1_000,
                    },
                    files: vec![
                        log_file_metadata(&active, 10_000, true),
                        log_file_metadata(&expired, 8_000, false),
                    ],
                    now_unix_ms: 10_000,
                },
                local_sandbox_only: true,
                production_log_paths_requested: false,
                service_manager_action_requested: false,
                external_log_shipping_requested: false,
            })
            .expect("local observability log retention should execute");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_observability_log_retention_execution_audit(
            &mut journal,
            &report,
            1_700_000_000_730,
        )
        .expect("observability log retention audit writes");
        let checkpoint = persist_observability_log_retention_execution_checkpoint(
            &mut store,
            &report,
            1_700_000_000_731,
        )
        .expect("observability log retention checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_LOG_RETENTION_EXECUTION_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_LOG_RETENTION_EXECUTION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("observability log retention checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"sandbox_filesystem_mutated\":true"));
        assert!(recovered
            .value
            .contains("\"production_log_paths_touched\":false"));
        assert!(recovered
            .value
            .contains("\"service_manager_action_performed\":false"));
        assert!(recovered
            .value
            .contains("\"external_log_shipping_performed\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn local_tracing_subscriber_captures_sanitized_event_without_global_install() {
        let report = validate_local_tracing_subscriber(LocalTracingSubscriberValidationRequest {
            validation_id: "local-tracing-subscriber-001".to_owned(),
            subscriber_label: "local-runtime-subscriber".to_owned(),
            event: StructuredLogEvent::new(
                "local-trace-event-001",
                ObservabilitySeverity::Info,
                "runtime-observability",
                "local tracing subscriber captured event",
                vec![StructuredLogField::new("scope", "local-validation")],
                1_700_000_000_700,
            ),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-tracing-test".to_owned(),
            )),
            local_capture_required: true,
            redaction_required: true,
            global_install_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
            public_network_exposure_requested: false,
            live_execution_requested: false,
            captured_at_ms: 1_700_000_000_701,
        })
        .expect("local tracing subscriber validation should capture event");

        assert_eq!(
            report.status,
            LocalTracingSubscriberValidationStatus::ReadyForLocalReview
        );
        assert!(report.scoped_subscriber_installed);
        assert!(report.event_captured);
        assert_eq!(report.captured_event_count, 1);
        assert!(report
            .captured_output_excerpt
            .contains("local-trace-event-001"));
        assert!(report
            .captured_output_excerpt
            .contains("runtime-observability"));
        assert!(report.access_authorized);
        assert!(!report.global_subscriber_installed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.public_network_exposed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn local_tracing_subscriber_rejects_export_or_global_side_effects() {
        let error = validate_local_tracing_subscriber(LocalTracingSubscriberValidationRequest {
            validation_id: "local-tracing-side-effect".to_owned(),
            subscriber_label: "local-runtime-subscriber".to_owned(),
            event: StructuredLogEvent::new(
                "local-trace-side-effect",
                ObservabilitySeverity::Info,
                "runtime-observability",
                "local tracing subscriber should fail closed",
                Vec::new(),
                1_700_000_000_710,
            ),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-tracing-test".to_owned(),
            )),
            local_capture_required: true,
            redaction_required: true,
            global_install_requested: true,
            telemetry_export_requested: true,
            outbound_alert_delivery_requested: false,
            public_network_exposure_requested: false,
            live_execution_requested: false,
            captured_at_ms: 1_700_000_000_711,
        })
        .expect_err("global install or telemetry export requests should fail closed");

        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations
            .iter()
            .any(|violation| violation.code() == "OBSERVABILITY_TRACING_SIDE_EFFECT_REQUESTED"));
    }

    #[test]
    fn local_tracing_subscriber_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("observability-tracing-subscriber");
        let state_path = temp_state_path("observability-tracing-subscriber");
        let report = validate_local_tracing_subscriber(LocalTracingSubscriberValidationRequest {
            validation_id: "local-tracing-audit".to_owned(),
            subscriber_label: "local-runtime-subscriber".to_owned(),
            event: StructuredLogEvent::new(
                "local-trace-audit",
                ObservabilitySeverity::Info,
                "runtime-observability",
                "local tracing subscriber audited event",
                vec![StructuredLogField::new("scope", "audit-state")],
                1_700_000_000_720,
            ),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-tracing-audit".to_owned(),
            )),
            local_capture_required: true,
            redaction_required: true,
            global_install_requested: false,
            telemetry_export_requested: false,
            outbound_alert_delivery_requested: false,
            public_network_exposure_requested: false,
            live_execution_requested: false,
            captured_at_ms: 1_700_000_000_721,
        })
        .expect("local tracing subscriber validation should capture event");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_local_tracing_subscriber_audit(&mut journal, &report, 1_700_000_000_722)
                .expect("local tracing subscriber audit writes");
        let checkpoint =
            persist_local_tracing_subscriber_checkpoint(&mut store, &report, 1_700_000_000_723)
                .expect("local tracing subscriber checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY
        );
        assert!(!report.global_subscriber_installed);
        assert!(!report.telemetry_exported);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.public_network_exposed);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_TRACING_SUBSCRIBER_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("local tracing subscriber checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"scoped_subscriber_installed\":true"));
        assert!(recovered.value.contains("\"event_captured\":true"));
        assert!(recovered
            .value
            .contains("\"global_subscriber_installed\":false"));
        assert!(recovered.value.contains("\"telemetry_exported\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_failure_capture_records_local_only_failure_without_alerts() {
        let record = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
            failure_id: "runtime-failure-001".to_owned(),
            component: "runtime-lifecycle".to_owned(),
            kind: RuntimeFailureKind::Panic,
            severity: ObservabilitySeverity::Critical,
            summary: "local runtime panic captured for operator review".to_owned(),
            detail: "panic boundary captured sanitized local failure metadata".to_owned(),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-failure-capture".to_owned(),
            )),
            captured_at_ms: 1_700_000_000_400,
        })
        .expect("local runtime failure capture should succeed");

        assert_eq!(record.failure_id, "runtime-failure-001");
        assert_eq!(record.kind, RuntimeFailureKind::Panic);
        assert_eq!(record.severity, ObservabilitySeverity::Critical);
        assert!(record.access_authorized);
        assert_eq!(
            record.access_authorization_status,
            ObservabilityAccessAuthorizationStatus::AuthorizedLocalCollection
        );
        assert!(!record.metrics_endpoint_started);
        assert!(!record.public_network_exposed);
        assert!(!record.outbound_alerts_sent);
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);
    }

    #[test]
    fn runtime_failure_capture_redacts_secret_like_detail() {
        let record = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
            failure_id: "runtime-failure-redacted".to_owned(),
            component: "runtime-lifecycle".to_owned(),
            kind: RuntimeFailureKind::ValidationFailure,
            severity: ObservabilitySeverity::Error,
            summary: "local validation failure captured".to_owned(),
            detail: concat!("authorization:", " not-a-real-value-for-test-only").to_owned(),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(None),
            captured_at_ms: 1_700_000_000_410,
        })
        .expect("local runtime failure capture should redact secret-like detail");

        assert!(record.secret_redaction_applied);
        assert_eq!(record.detail, "[REDACTED SECRET-LIKE OBSERVABILITY TEXT]");
    }

    #[test]
    fn runtime_failure_capture_rejects_external_sessions() {
        let error = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
            failure_id: "runtime-failure-external".to_owned(),
            component: "runtime-lifecycle".to_owned(),
            kind: RuntimeFailureKind::HealthCheckFailure,
            severity: ObservabilitySeverity::Warning,
            summary: "external exporter attempted failure capture".to_owned(),
            detail: "external exporter sessions remain disabled".to_owned(),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext {
                source: ObservabilityAccessSource::ExporterSession,
                collector_label: Some("otel-exporter".to_owned()),
            },
            captured_at_ms: 1_700_000_000_420,
        })
        .expect_err("external failure capture session should fail closed");

        let ObservabilityError::ValidationFailed { violations } = error else {
            panic!("expected observability validation error");
        };
        assert!(violations
            .iter()
            .any(|violation| violation.code() == "OBSERVABILITY_ACCESS_DENIED"));
    }

    #[test]
    fn runtime_failure_capture_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("runtime-failure-capture");
        let state_path = temp_state_path("runtime-failure-capture");
        let record = capture_local_runtime_failure(RuntimeFailureCaptureRequest {
            failure_id: "runtime-failure-audit".to_owned(),
            component: "runtime-lifecycle".to_owned(),
            kind: RuntimeFailureKind::Crash,
            severity: ObservabilitySeverity::Error,
            summary: "local abrupt exit captured for replay".to_owned(),
            detail: "local crash-style failure metadata captured without service actions"
                .to_owned(),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-failure-audit".to_owned(),
            )),
            captured_at_ms: 1_700_000_000_430,
        })
        .expect("local runtime failure capture should succeed");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_runtime_failure_capture_audit(&mut journal, &record, 1_700_000_000_431)
                .expect("runtime failure capture audit writes");
        let checkpoint =
            persist_runtime_failure_capture_checkpoint(&mut store, &record, 1_700_000_000_432)
                .expect("runtime failure capture checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY);
        assert!(!record.metrics_endpoint_started);
        assert!(!record.public_network_exposed);
        assert!(!record.outbound_alerts_sent);
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("runtime failure capture checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"metrics_endpoint_started\":false"));
        assert!(recovered.value.contains("\"public_network_exposed\":false"));
        assert!(recovered.value.contains("\"outbound_alerts_sent\":false"));
        assert!(recovered
            .value
            .contains("\"external_submission_performed\":false"));
        assert!(recovered
            .value
            .contains("\"live_execution_performed\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn scoped_panic_hook_capture_audits_and_checkpoints_local_panic() {
        let audit_path = temp_audit_path("runtime-panic-hook-capture");
        let state_path = temp_state_path("runtime-panic-hook-capture");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let report = capture_local_panic_with_scoped_hook(
            &mut journal,
            &mut store,
            RuntimeFailureCaptureRequest {
                failure_id: "runtime-panic-hook-capture".to_owned(),
                component: "runtime-lifecycle".to_owned(),
                kind: RuntimeFailureKind::Panic,
                severity: ObservabilitySeverity::Critical,
                summary: "local scoped panic hook captured runtime failure".to_owned(),
                detail: "scoped local panic hook stores sanitized failure metadata".to_owned(),
                config: ObservabilityBoundaryConfig::default(),
                access: ObservabilityAccessContext::local_collection(Some(
                    "local-panic-hook-capture".to_owned(),
                )),
                captured_at_ms: 1_700_000_000_440,
            },
            || panic!("local scoped panic hook sentinel"),
        )
        .expect("scoped panic hook capture should succeed");

        assert!(report.hook_installed);
        assert!(report.hook_restored);
        assert!(report.panic_observed);
        assert_eq!(report.audit_sequence, Some(1));
        assert_eq!(
            report.checkpoint_key.as_deref(),
            Some(OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY)
        );
        let record = report
            .failure_record
            .expect("panic capture should produce a failure record");
        assert_eq!(record.kind, RuntimeFailureKind::Panic);
        assert!(record.detail.contains("local scoped panic hook sentinel"));
        assert!(!record.metrics_endpoint_started);
        assert!(!record.public_network_exposed);
        assert!(!record.outbound_alerts_sent);
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);
        assert!(!report.metrics_endpoint_started);
        assert!(!report.public_network_exposed);
        assert!(!report.outbound_alerts_sent);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("runtime panic hook checkpoint exists");
        assert!(recovered.value.contains("local scoped panic hook sentinel"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_panic_hook_guard_captures_local_panic_and_restores_hook() {
        let audit_path = temp_audit_path("runtime-panic-hook-guard");
        let state_path = temp_state_path("runtime-panic-hook-guard");
        let guard = install_local_runtime_panic_hook(RuntimePanicHookInstallationRequest {
            failure_id: "runtime-panic-hook-guard".to_owned(),
            component: "runtime-lifecycle".to_owned(),
            severity: ObservabilitySeverity::Critical,
            summary: "local runtime panic hook captured failure".to_owned(),
            detail: "daemon-style local panic hook stores sanitized failure metadata".to_owned(),
            audit_path: audit_path.clone(),
            state_path: state_path.clone(),
            config: ObservabilityBoundaryConfig::default(),
            access: ObservabilityAccessContext::local_collection(Some(
                "local-runtime-panic-hook".to_owned(),
            )),
            captured_at_ms: 1_700_000_000_450,
        })
        .expect("runtime panic hook should install");

        assert!(guard.report().hook_installed);
        assert_eq!(guard.report().failure_id, "runtime-panic-hook-guard");
        assert!(!guard.report().metrics_endpoint_started);
        assert!(!guard.report().public_network_exposed);
        assert!(!guard.report().outbound_alerts_sent);
        assert!(!guard.report().external_submission_performed);
        assert!(!guard.report().live_execution_performed);
        assert!(!guard.report().production_ready);

        let panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
            panic!("local runtime panic hook guard sentinel");
        }));
        assert!(panic_result.is_err());
        assert!(guard.panic_captured());
        assert_eq!(guard.last_capture_error(), None);
        drop(guard);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(OBSERVABILITY_LAST_FAILURE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("runtime panic hook checkpoint exists");
        assert!(recovered
            .value
            .contains("local runtime panic hook guard sentinel"));
        assert!(recovered
            .value
            .contains("\"metrics_endpoint_started\":false"));
        assert!(recovered.value.contains("\"outbound_alerts_sent\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    fn temp_audit_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!("arbyclaw-{label}-{}-{nanos}.jsonl", process::id()));
        path
    }

    fn temp_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-{label}-{}-{nanos}.sqlite3",
            process::id()
        ));
        path
    }

    fn temp_dir(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!("arbyclaw-{label}-{}-{nanos}", process::id()));
        fs::create_dir_all(&path).expect("temp directory should be created");
        path
    }

    fn write_local_log_fixture(path: &PathBuf, label: &str) {
        fs::write(path, format!("{label} observability log fixture\n"))
            .expect("local log fixture should be written");
    }

    fn log_file_metadata(
        path: &PathBuf,
        modified_at_unix_ms: u64,
        active: bool,
    ) -> AuditJournalFileMetadata {
        AuditJournalFileMetadata {
            path: path.display().to_string(),
            size_bytes: fs::metadata(path)
                .expect("local log metadata should be readable")
                .len(),
            modified_at_unix_ms,
            active,
        }
    }

    fn cleanup_state_files(path: &PathBuf) {
        let _ = fs::remove_file(path);
        for suffix in ["-wal", "-shm"] {
            let related = format!("{}{}", path.display(), suffix);
            let _ = fs::remove_file(related);
        }
    }
}
