#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::RuntimeMode;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable embedded dashboard boundary version for audit, replay, and handoff surfaces.
pub const DASHBOARD_BOUNDARY_VERSION: &str = "phase-13-dashboard-boundary-v1";

/// Conservative dashboard boundary settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardBoundaryConfig {
    /// Whether local dashboard snapshots may be rendered to in-process records.
    pub local_rendering_enabled: bool,
    /// Future server binding model. Phase 13 must not start a server.
    pub server_binding: DashboardServerBinding,
    /// Maximum panel items to include in a render record.
    pub max_panel_items: usize,
    /// Whether dashboard controls may trigger live execution. Phase 13 requires false.
    pub allow_live_controls: bool,
    /// Whether secret-like text may be rendered. Phase 13 requires false.
    pub allow_secret_rendering: bool,
}

impl Default for DashboardBoundaryConfig {
    fn default() -> Self {
        Self {
            local_rendering_enabled: true,
            server_binding: DashboardServerBinding::default(),
            max_panel_items: 100,
            allow_live_controls: false,
            allow_secret_rendering: false,
        }
    }
}

impl DashboardBoundaryConfig {
    /// Validate fail-closed Phase 13 dashboard settings.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();

        if self.max_panel_items == 0 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_MAX_PANEL_ITEMS_ZERO",
                "max_panel_items must be positive",
            ));
        }

        if self.server_binding.http_server_enabled {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HTTP_SERVER_DENIED_IN_PHASE_13",
                "Phase 13 dashboard boundaries must not start an HTTP server",
            ));
        }

        if self.server_binding.public_network_exposure {
            violations.push(DashboardViolation::new(
                "DASHBOARD_PUBLIC_NETWORK_DENIED_IN_PHASE_13",
                "Phase 13 dashboard boundaries must not expose public network bindings",
            ));
        }

        if !self.server_binding.require_loopback_only {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOOPBACK_REQUIRED",
                "dashboard binding must require loopback-only access",
            ));
        }

        if !is_loopback_host(&self.server_binding.bind_host) {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_BIND_HOST_NOT_LOOPBACK",
                format!(
                    "dashboard bind host {} is not loopback-only",
                    self.server_binding.bind_host
                ),
            ));
        }

        if self.allow_live_controls {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LIVE_CONTROLS_DENIED_IN_PHASE_13",
                "Phase 13 dashboard boundaries must not enable live execution controls",
            ));
        }

        if self.allow_secret_rendering {
            violations.push(DashboardViolation::new(
                "DASHBOARD_SECRET_RENDERING_DENIED",
                "dashboard rendering must redact secret-like text",
            ));
        }

        finish_validation(violations)
    }
}

/// Future local server binding model. It is not an active listener in Phase 13.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardServerBinding {
    /// Whether an HTTP server should be started. Phase 13 requires false.
    pub http_server_enabled: bool,
    /// Intended future bind host. Must remain loopback-only.
    pub bind_host: String,
    /// Intended future bind port.
    pub bind_port: u16,
    /// Whether loopback-only binding is mandatory.
    pub require_loopback_only: bool,
    /// Whether public network exposure is requested. Phase 13 requires false.
    pub public_network_exposure: bool,
}

impl Default for DashboardServerBinding {
    fn default() -> Self {
        Self {
            http_server_enabled: false,
            bind_host: "127.0.0.1".to_owned(),
            bind_port: 8_080,
            require_loopback_only: true,
            public_network_exposure: false,
        }
    }
}

/// Dashboard panel category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardPanelKind {
    /// Runtime status and build information.
    SystemStatus,
    /// Safety posture and mode gates.
    Safety,
    /// Market-data freshness and provider status.
    MarketData,
    /// Opportunity discovery summary.
    Opportunities,
    /// Draft-only execution planner summary.
    Planner,
    /// Execution adapter boundary summary.
    ExecutionAdapter,
    /// Communications and CLI summary.
    Communications,
    /// Audit and state-store summary.
    AuditState,
    /// Production gap summary.
    Gaps,
}

/// Dashboard display severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardSeverity {
    /// Informational item.
    Info,
    /// Healthy item.
    Ok,
    /// Warning item.
    Warning,
    /// Error item.
    Error,
    /// Critical safety item.
    Critical,
}

/// One dashboard item inside a panel.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardPanelItem {
    /// Display label.
    pub label: String,
    /// Display value.
    pub value: String,
    /// Display severity.
    pub severity: DashboardSeverity,
}

impl DashboardPanelItem {
    /// Create one display item.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        value: impl Into<String>,
        severity: DashboardSeverity,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            severity,
        }
    }

    fn redacted(&self, max_chars: usize) -> (Self, bool) {
        let (label, label_redacted) = sanitize_dashboard_text(&self.label, max_chars);
        let (value, value_redacted) = sanitize_dashboard_text(&self.value, max_chars);
        (
            Self {
                label,
                value,
                severity: self.severity,
            },
            label_redacted || value_redacted,
        )
    }
}

/// One dashboard panel.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardPanel {
    /// Panel category.
    pub kind: DashboardPanelKind,
    /// Display title.
    pub title: String,
    /// Display summary.
    pub summary: String,
    /// Panel items.
    pub items: Vec<DashboardPanelItem>,
}

impl DashboardPanel {
    /// Create one dashboard panel.
    #[must_use]
    pub fn new(
        kind: DashboardPanelKind,
        title: impl Into<String>,
        summary: impl Into<String>,
        items: Vec<DashboardPanelItem>,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            summary: summary.into(),
            items,
        }
    }

    fn redacted(&self, max_chars: usize, max_items: usize) -> (Self, bool) {
        let (title, title_redacted) = sanitize_dashboard_text(&self.title, max_chars);
        let (summary, summary_redacted) = sanitize_dashboard_text(&self.summary, max_chars);
        let mut redacted_any = title_redacted || summary_redacted;
        let items = self
            .items
            .iter()
            .take(max_items)
            .map(|item| {
                let (redacted_item, item_redacted) = item.redacted(max_chars);
                redacted_any |= item_redacted;
                redacted_item
            })
            .collect();

        (
            Self {
                kind: self.kind,
                title,
                summary,
                items,
            },
            redacted_any,
        )
    }
}

/// Dashboard snapshot supplied by future runtime components.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardSnapshot {
    /// Stable snapshot identifier. Must not contain secret material.
    pub snapshot_id: String,
    /// Snapshot creation timestamp in Unix epoch milliseconds.
    pub generated_at_ms: u64,
    /// Runtime mode represented by this snapshot.
    pub runtime_mode: RuntimeMode,
    /// Governance readiness estimate represented by this snapshot.
    pub production_readiness_percent: u8,
    /// Number of currently known open production gaps.
    pub open_gap_count: usize,
    /// Number of discovered opportunities represented in the snapshot.
    pub opportunity_count: usize,
    /// Number of draft execution plans represented in the snapshot.
    pub pending_plan_count: usize,
    /// Number of local notification records represented in the snapshot.
    pub notification_record_count: usize,
    /// Panels available to render locally.
    pub panels: Vec<DashboardPanel>,
    /// Snapshot warnings.
    pub warnings: Vec<String>,
}

impl DashboardSnapshot {
    /// Validate dashboard snapshot shape before local rendering.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();

        validate_id("snapshot", &self.snapshot_id, &mut violations);
        if self.production_readiness_percent > 100 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_READINESS_PERCENT_INVALID",
                "production_readiness_percent cannot exceed 100",
            ));
        }

        let mut panel_kinds = BTreeSet::new();
        for panel in &self.panels {
            if !panel_kinds.insert(panel.kind) {
                violations.push(DashboardViolation::new_owned(
                    "DASHBOARD_PANEL_DUPLICATE",
                    format!("dashboard panel {:?} is duplicated", panel.kind),
                ));
            }
            if panel.title.trim().is_empty() {
                violations.push(DashboardViolation::new(
                    "DASHBOARD_PANEL_TITLE_EMPTY",
                    "dashboard panel title must be non-empty",
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Local dashboard rendering request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardRenderRequest {
    /// Boundary configuration.
    pub config: DashboardBoundaryConfig,
    /// Runtime snapshot to render.
    pub snapshot: DashboardSnapshot,
    /// Optional panel allowlist. Empty means all panels.
    pub requested_panels: Vec<DashboardPanelKind>,
    /// Optional operator-facing label. Must not contain secret material.
    pub operator_label: Option<String>,
    /// Render timestamp in Unix epoch milliseconds.
    pub rendered_at_ms: u64,
}

/// Local dashboard render record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardRenderRecord {
    /// Boundary version that produced this record.
    pub dashboard_boundary_version: String,
    /// Rendered snapshot identifier.
    pub snapshot_id: String,
    /// Render timestamp in Unix epoch milliseconds.
    pub rendered_at_ms: u64,
    /// Runtime mode represented by the render.
    pub runtime_mode: RuntimeMode,
    /// Governance readiness estimate represented by the render.
    pub production_readiness_percent: u8,
    /// Sanitized operator label.
    pub operator_label: Option<String>,
    /// Locally rendered panels.
    pub panels: Vec<DashboardPanel>,
    /// Sanitized warnings.
    pub warnings: Vec<String>,
    /// Whether a server was started. Phase 13 always returns false.
    pub server_started: bool,
    /// Whether public network exposure occurred. Phase 13 always returns false.
    pub public_network_exposed: bool,
    /// Whether live controls are enabled. Phase 13 always returns false.
    pub live_controls_enabled: bool,
    /// Whether secret-like text was redacted before record creation.
    pub secret_redaction_applied: bool,
}

/// Dashboard renderer boundary.
pub trait DashboardRenderer {
    /// Render a dashboard request into a local, non-network render record.
    fn render(
        &self,
        request: DashboardRenderRequest,
    ) -> Result<DashboardRenderRecord, DashboardError>;
}

/// Deterministic local-only dashboard renderer.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicDashboardRenderer;

impl DashboardRenderer for DeterministicDashboardRenderer {
    fn render(
        &self,
        request: DashboardRenderRequest,
    ) -> Result<DashboardRenderRecord, DashboardError> {
        request.config.validate()?;
        request.snapshot.validate()?;

        if !request.config.local_rendering_enabled {
            return Err(DashboardError::ValidationFailed {
                violations: vec![DashboardViolation::new(
                    "DASHBOARD_LOCAL_RENDERING_DISABLED",
                    "local dashboard rendering is disabled",
                )],
            });
        }

        let mut redaction_applied = false;
        let max_chars = 512;
        let (snapshot_id, snapshot_redacted) =
            sanitize_dashboard_text(&request.snapshot.snapshot_id, max_chars);
        redaction_applied |= snapshot_redacted;

        let operator_label = request.operator_label.as_ref().map(|label| {
            let (sanitized, redacted) = sanitize_dashboard_text(label, max_chars);
            redaction_applied |= redacted;
            sanitized
        });

        let requested = requested_panel_set(&request.requested_panels);
        let panels = request
            .snapshot
            .panels
            .iter()
            .filter(|panel| match &requested {
                Some(set) => set.contains(&panel.kind),
                None => true,
            })
            .map(|panel| {
                let (redacted_panel, panel_redacted) =
                    panel.redacted(max_chars, request.config.max_panel_items);
                redaction_applied |= panel_redacted;
                redacted_panel
            })
            .collect();

        let warnings = request
            .snapshot
            .warnings
            .iter()
            .map(|warning| {
                let (sanitized, redacted) = sanitize_dashboard_text(warning, max_chars);
                redaction_applied |= redacted;
                sanitized
            })
            .collect();

        Ok(DashboardRenderRecord {
            dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
            snapshot_id,
            rendered_at_ms: request.rendered_at_ms,
            runtime_mode: request.snapshot.runtime_mode,
            production_readiness_percent: request.snapshot.production_readiness_percent,
            operator_label,
            panels,
            warnings,
            server_started: false,
            public_network_exposed: false,
            live_controls_enabled: false,
            secret_redaction_applied: redaction_applied,
        })
    }
}

/// One deterministic dashboard validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardViolation {
    code: &'static str,
    message: String,
}

impl DashboardViolation {
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

/// Dashboard boundary error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardError {
    /// Dashboard validation failed.
    ValidationFailed { violations: Vec<DashboardViolation> },
}

impl fmt::Display for DashboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                writeln!(
                    formatter,
                    "dashboard validation failed with {} violation(s):",
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

impl Error for DashboardError {}

fn finish_validation(violations: Vec<DashboardViolation>) -> Result<(), DashboardError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(DashboardError::ValidationFailed { violations })
    }
}

fn validate_id(kind: &'static str, id: &str, violations: &mut Vec<DashboardViolation>) {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        violations.push(DashboardViolation::new_owned(
            "DASHBOARD_ID_EMPTY",
            format!("{kind} id must be non-empty"),
        ));
    }
    if trimmed.len() > 128 {
        violations.push(DashboardViolation::new_owned(
            "DASHBOARD_ID_TOO_LONG",
            format!("{kind} id is too long"),
        ));
    }
    if contains_secret_like_text(trimmed) {
        violations.push(DashboardViolation::new_owned(
            "DASHBOARD_ID_SECRET_LIKE",
            format!("{kind} id looks like secret material"),
        ));
    }
}

fn requested_panel_set(kinds: &[DashboardPanelKind]) -> Option<BTreeSet<DashboardPanelKind>> {
    if kinds.is_empty() {
        None
    } else {
        Some(kinds.iter().copied().collect())
    }
}

fn is_loopback_host(host: &str) -> bool {
    matches!(
        host.trim().to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1"
    )
}

fn sanitize_dashboard_text(text: &str, max_chars: usize) -> (String, bool) {
    let mut sanitized = text.trim().to_owned();
    let mut changed = sanitized.len() != text.len();

    if contains_secret_like_text(&sanitized) {
        sanitized = "[REDACTED SECRET-LIKE DASHBOARD TEXT]".to_owned();
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
        DashboardBoundaryConfig, DashboardError, DashboardPanel, DashboardPanelItem,
        DashboardPanelKind, DashboardRenderRequest, DashboardRenderer, DashboardServerBinding,
        DashboardSeverity, DashboardSnapshot, DeterministicDashboardRenderer,
    };
    use crate::RuntimeMode;

    fn minimal_snapshot() -> DashboardSnapshot {
        DashboardSnapshot {
            snapshot_id: "snapshot-001".to_owned(),
            generated_at_ms: 1_700_000_000_000,
            runtime_mode: RuntimeMode::Paper,
            production_readiness_percent: 73,
            open_gap_count: 10,
            opportunity_count: 2,
            pending_plan_count: 1,
            notification_record_count: 3,
            panels: vec![DashboardPanel::new(
                DashboardPanelKind::Safety,
                "Safety",
                "Live controls are disabled",
                vec![DashboardPanelItem::new(
                    "Live controls",
                    "disabled",
                    DashboardSeverity::Ok,
                )],
            )],
            warnings: Vec::new(),
        }
    }

    #[test]
    fn dashboard_config_rejects_public_server_exposure() {
        let config = DashboardBoundaryConfig {
            server_binding: DashboardServerBinding {
                http_server_enabled: true,
                bind_host: "0.0.0.0".to_owned(),
                public_network_exposure: true,
                ..DashboardServerBinding::default()
            },
            ..DashboardBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("public server exposure must fail closed");
        let DashboardError::ValidationFailed { violations } = error;
        let codes = violations
            .iter()
            .map(super::DashboardViolation::code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"DASHBOARD_HTTP_SERVER_DENIED_IN_PHASE_13"));
        assert!(codes.contains(&"DASHBOARD_PUBLIC_NETWORK_DENIED_IN_PHASE_13"));
        assert!(codes.contains(&"DASHBOARD_BIND_HOST_NOT_LOOPBACK"));
    }

    #[test]
    fn renderer_never_starts_server_or_live_controls() {
        let renderer = DeterministicDashboardRenderer;
        let record = renderer
            .render(DashboardRenderRequest {
                config: DashboardBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                requested_panels: Vec::new(),
                operator_label: Some("local-operator".to_owned()),
                rendered_at_ms: 1_700_000_000_100,
            })
            .expect("local render should succeed");

        assert!(!record.server_started);
        assert!(!record.public_network_exposed);
        assert!(!record.live_controls_enabled);
        assert_eq!(record.panels.len(), 1);
    }

    #[test]
    fn renderer_redacts_secret_like_dashboard_text() {
        let mut snapshot = minimal_snapshot();
        snapshot.panels[0].items.push(DashboardPanelItem::new(
            "diagnostic",
            concat!("api", "_key=", "not-a-real-value-for-test-only"),
            DashboardSeverity::Warning,
        ));

        let renderer = DeterministicDashboardRenderer;
        let record = renderer
            .render(DashboardRenderRequest {
                config: DashboardBoundaryConfig::default(),
                snapshot,
                requested_panels: vec![DashboardPanelKind::Safety],
                operator_label: None,
                rendered_at_ms: 1_700_000_000_200,
            })
            .expect("local render should redact secret-like text");

        assert!(record.secret_redaction_applied);
        assert_eq!(
            record.panels[0].items[1].value,
            "[REDACTED SECRET-LIKE DASHBOARD TEXT]"
        );
    }
}
