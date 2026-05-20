#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable observability and runbook boundary version for audit, replay, and handoff surfaces.
pub const OBSERVABILITY_RUNBOOK_VERSION: &str = "phase-14-observability-runbook-v1";

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
    /// Optional operator-facing label. Must not contain secret material.
    pub operator_label: Option<String>,
    /// Collection timestamp in Unix epoch milliseconds.
    pub collected_at_ms: u64,
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

        if !request.config.local_collection_enabled {
            return Err(ObservabilityError::ValidationFailed {
                violations: vec![ObservabilityViolation::new(
                    "OBSERVABILITY_LOCAL_COLLECTION_DISABLED",
                    "local observability collection is disabled",
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
}

impl ObservabilityError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[ObservabilityViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
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
        ComponentHealthStatus, DeterministicObservabilityCollector, HealthStatus, MetricKind,
        MetricLabel, MetricSample, ObservabilityBoundaryConfig, ObservabilityCollectionRequest,
        ObservabilityCollector, ObservabilityEndpointBinding, ObservabilityError,
        ObservabilitySeverity, ObservabilitySnapshot, Runbook, RunbookStep, StructuredLogEvent,
        StructuredLogField,
    };

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
            ..ObservabilityBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("unsafe observability settings must fail closed");
        let ObservabilityError::ValidationFailed { violations } = error;
        let codes = violations
            .iter()
            .map(super::ObservabilityViolation::code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"OBSERVABILITY_METRICS_ENDPOINT_DENIED_IN_PHASE_14"));
        assert!(codes.contains(&"OBSERVABILITY_PUBLIC_NETWORK_DENIED_IN_PHASE_14"));
        assert!(codes.contains(&"OBSERVABILITY_BIND_HOST_NOT_LOOPBACK"));
        assert!(codes.contains(&"OBSERVABILITY_OUTBOUND_ALERTS_DENIED_IN_PHASE_14"));
    }

    #[test]
    fn collector_never_starts_endpoint_or_sends_alerts() {
        let collector = DeterministicObservabilityCollector;
        let record = collector
            .collect(ObservabilityCollectionRequest {
                config: ObservabilityBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                operator_label: Some("local-operator".to_owned()),
                collected_at_ms: 1_700_000_000_100,
            })
            .expect("local collection should succeed");

        assert_eq!(record.overall_health, HealthStatus::Healthy);
        assert!(!record.metrics_endpoint_started);
        assert!(!record.public_network_exposed);
        assert!(!record.outbound_alerts_sent);
        assert_eq!(record.logs.len(), 1);
        assert_eq!(record.metrics.len(), 1);
        assert_eq!(record.runbooks.len(), 1);
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
}
