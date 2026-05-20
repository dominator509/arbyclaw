#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::CommunicationConfig;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable communications and CLI boundary version for audit, replay, and handoff surfaces.
pub const COMMUNICATIONS_CLI_VERSION: &str = "phase-12-communications-cli-v1";

/// Conservative communication boundary settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommunicationBoundaryConfig {
    /// Whether local CLI commands may be routed.
    pub cli_enabled: bool,
    /// Non-secret notification channel profiles.
    pub notification_channels: Vec<NotificationChannelProfile>,
    /// Maximum message size allowed before local truncation.
    pub max_message_chars: usize,
    /// Whether outbound network delivery is enabled. Phase 12 requires this to remain false.
    pub outbound_network_enabled: bool,
    /// Whether operator commands may trigger autonomous execution. Phase 12 requires false.
    pub allow_execution_commands: bool,
}

impl Default for CommunicationBoundaryConfig {
    fn default() -> Self {
        Self {
            cli_enabled: true,
            notification_channels: Vec::new(),
            max_message_chars: 2_000,
            outbound_network_enabled: false,
            allow_execution_commands: false,
        }
    }
}

impl CommunicationBoundaryConfig {
    /// Build a communication boundary config from non-secret runtime configuration.
    #[must_use]
    pub fn from_config(config: &CommunicationConfig) -> Self {
        Self {
            cli_enabled: config.cli_enabled,
            notification_channels: config
                .notify_channels
                .iter()
                .map(|channel| NotificationChannelProfile::from_identifier(channel))
                .collect(),
            ..Self::default()
        }
    }

    /// Validate fail-closed Phase 12 communication settings.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();

        if self.max_message_chars == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_MAX_MESSAGE_CHARS_ZERO",
                "max_message_chars must be positive",
            ));
        }

        if self.outbound_network_enabled {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_OUTBOUND_NETWORK_DENIED_IN_PHASE_12",
                "Phase 12 communication boundaries must not enable outbound network delivery",
            ));
        }

        if self.allow_execution_commands {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_EXECUTION_COMMANDS_DENIED_IN_PHASE_12",
                "Phase 12 communication boundaries must not enable autonomous execution commands",
            ));
        }

        let mut channel_ids = BTreeSet::new();
        for channel in &self.notification_channels {
            validate_id("notification channel", &channel.id, &mut violations);
            if contains_secret_like_text(&channel.id) {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_ID_SECRET_LIKE",
                    format!(
                        "notification channel {} looks like secret material",
                        channel.id
                    ),
                ));
            }
            if !channel_ids.insert(channel.id.to_ascii_lowercase()) {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_DUPLICATE",
                    format!("notification channel {} is duplicated", channel.id),
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Non-secret notification channel class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommunicationChannelKind {
    /// Local CLI or terminal interaction.
    Cli,
    /// Local stdout/stderr-style diagnostic sink.
    LocalStdout,
    /// Email channel identifier only; no SMTP integration exists in Phase 12.
    Email,
    /// Chat or messaging channel identifier only; no platform integration exists in Phase 12.
    Chat,
    /// Webhook channel identifier only; no outbound HTTP integration exists in Phase 12.
    Webhook,
    /// Paging channel identifier only; no paging integration exists in Phase 12.
    Pager,
    /// Unknown non-secret identifier preserved for future adapters.
    Other,
}

/// Non-secret notification channel profile.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationChannelProfile {
    /// Stable non-secret channel identifier.
    pub id: String,
    /// Channel class.
    pub kind: CommunicationChannelKind,
    /// Whether this channel should be considered by local dispatch records.
    pub enabled: bool,
}

impl NotificationChannelProfile {
    /// Create a channel profile from a non-secret identifier.
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Self {
        let trimmed = identifier.trim();
        let lower = trimmed.to_ascii_lowercase();
        let kind = if lower == "cli" {
            CommunicationChannelKind::Cli
        } else if lower == "stdout" || lower == "local-stdout" {
            CommunicationChannelKind::LocalStdout
        } else if lower.starts_with("email:") {
            CommunicationChannelKind::Email
        } else if lower.starts_with("chat:")
            || lower.starts_with("telegram:")
            || lower.starts_with("discord:")
            || lower.starts_with("slack:")
            || lower.starts_with("matrix:")
        {
            CommunicationChannelKind::Chat
        } else if lower.starts_with("webhook:") {
            CommunicationChannelKind::Webhook
        } else if lower.starts_with("pager:") || lower.starts_with("pagerduty:") {
            CommunicationChannelKind::Pager
        } else {
            CommunicationChannelKind::Other
        };

        Self {
            id: trimmed.to_owned(),
            kind,
            enabled: true,
        }
    }
}

/// Source of an operator command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorCommandSource {
    /// Local process command line.
    LocalCli,
    /// Future scheduled operator task boundary.
    Scheduler,
    /// Future messaging platform boundary.
    MessagingChannel,
    /// Future local dashboard boundary.
    Dashboard,
}

/// Parsed operator command kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorCommandKind {
    /// Show local runtime and build status.
    Status,
    /// Show help text.
    Help,
    /// Load and validate a config file.
    ValidateConfig,
    /// Explain the current safety posture.
    ShowSafety,
    /// Explain roadmap position.
    ShowRoadmapPosition,
    /// Plan-only diagnostic boundary.
    PlanOnly,
    /// Request to trade or execute live.
    LiveExecutionRequest,
    /// Request to withdraw funds.
    WithdrawalRequest,
    /// Request to bridge funds.
    BridgeRequest,
    /// Request to sign payloads or transactions.
    SignRequest,
    /// Request to broadcast transactions.
    BroadcastRequest,
    /// Unknown command.
    Unknown,
}

impl OperatorCommandKind {
    /// Returns whether the command would require unavailable funds-movement capability.
    #[must_use]
    pub const fn is_unsafe_execution_request(self) -> bool {
        matches!(
            self,
            Self::LiveExecutionRequest
                | Self::WithdrawalRequest
                | Self::BridgeRequest
                | Self::SignRequest
                | Self::BroadcastRequest
        )
    }
}

/// Parsed operator command record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorCommand {
    /// Stable command id for audit and replay.
    pub id: String,
    /// Command source.
    pub source: OperatorCommandSource,
    /// Parsed command kind.
    pub kind: OperatorCommandKind,
    /// Non-secret command arguments.
    pub args: Vec<String>,
    /// Sanitized raw command text.
    pub sanitized_raw: String,
    /// Receive time in Unix milliseconds, or zero in tests/offline parsing.
    pub received_at_unix_ms: u64,
}

impl OperatorCommand {
    /// Validate a parsed operator command.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("operator command", &self.id, &mut violations);

        if contains_secret_like_text(&self.sanitized_raw) {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_COMMAND_SECRET_LIKE",
                "operator command text looks like it may contain secret material",
            ));
        }

        for arg in &self.args {
            if arg.trim().is_empty() {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_COMMAND_ARG_EMPTY",
                    "operator command args cannot contain empty values",
                ));
            }
            if contains_secret_like_text(arg) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_COMMAND_ARG_SECRET_LIKE",
                    "operator command arg looks like it may contain secret material",
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Parse local CLI arguments into a typed command boundary.
pub fn parse_cli_command(
    args: &[String],
    received_at_unix_ms: u64,
) -> Result<OperatorCommand, CommunicationError> {
    let kind = command_kind_for(args);
    let command = OperatorCommand {
        id: format!(
            "operator-command:{}:{}:{}",
            received_at_unix_ms,
            command_kind_label(kind),
            args.len()
        ),
        source: OperatorCommandSource::LocalCli,
        kind,
        args: command_args_for(args, kind),
        sanitized_raw: sanitize_command_text(&args.join(" ")),
        received_at_unix_ms,
    };
    command.validate()?;
    Ok(command)
}

/// Routed action for a parsed operator command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorCommandAction {
    /// Show local status only.
    ShowStatus,
    /// Show help only.
    ShowHelp,
    /// Validate one config file.
    ValidateConfigFile { path: String },
    /// Show safety posture.
    ShowSafety,
    /// Show roadmap position.
    ShowRoadmapPosition,
    /// Preserve a plan-only boundary without execution.
    PlanOnlyBoundary,
    /// Reject an unsafe or unavailable command.
    RejectUnsafeCommand,
    /// Reject unknown command.
    RejectUnknownCommand,
}

/// Operator command routing request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorCommandRoutingRequest {
    /// Stable routing request id.
    pub id: String,
    /// Parsed command to route.
    pub command: OperatorCommand,
    /// Communication safety config.
    pub config: CommunicationBoundaryConfig,
}

impl OperatorCommandRoutingRequest {
    /// Validate the routing request.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("command routing request", &self.id, &mut violations);

        if let Err(CommunicationError::ValidationFailed {
            violations: command_violations,
        }) = self.command.validate()
        {
            violations.extend(command_violations);
        }

        if let Err(CommunicationError::ValidationFailed {
            violations: config_violations,
        }) = self.config.validate()
        {
            violations.extend(config_violations);
        }

        finish_validation(violations)
    }
}

/// Deterministic routed operator command record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoutedOperatorCommand {
    /// Stable route id.
    pub id: String,
    /// Source routing request id.
    pub request_id: String,
    /// Source command id.
    pub command_id: String,
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Routed action.
    pub action: OperatorCommandAction,
    /// Whether the command is accepted for local handling.
    pub accepted: bool,
    /// Whether any autonomous execution could occur. Always false in Phase 12.
    pub execution_enabled: bool,
    /// Whether any outbound network was used. Always false in Phase 12.
    pub outbound_network_used: bool,
    /// Non-secret reason.
    pub reason: String,
}

impl RoutedOperatorCommand {
    /// Validate command-route invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("routed command", &self.id, &mut violations);
        validate_id("routing request", &self.request_id, &mut violations);
        validate_id("operator command", &self.command_id, &mut violations);

        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }

        if self.execution_enabled {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_ROUTE_EXECUTION_ENABLED",
                "Phase 12 routed commands must never enable autonomous execution",
            ));
        }

        if self.outbound_network_used {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_ROUTE_OUTBOUND_NETWORK_USED",
                "Phase 12 routed commands must not use outbound network delivery",
            ));
        }

        if contains_secret_like_text(&self.reason) {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_ROUTE_REASON_SECRET_LIKE",
                "routed command reason looks like it may contain secret material",
            ));
        }

        finish_validation(violations)
    }
}

/// Command-router trait boundary.
pub trait OperatorCommandRouter {
    /// Stable router name for diagnostics and audit records.
    fn router_name(&self) -> &str;

    /// Route a parsed command without executing trades, signing, or network calls.
    fn route(
        &self,
        request: &OperatorCommandRoutingRequest,
    ) -> Result<RoutedOperatorCommand, CommunicationError>;
}

/// Deterministic Phase 12 local command router.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicOperatorCommandRouter;

impl DeterministicOperatorCommandRouter {
    /// Create a deterministic local command router.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl OperatorCommandRouter for DeterministicOperatorCommandRouter {
    fn router_name(&self) -> &str {
        "deterministic-phase-12-operator-command-router"
    }

    fn route(
        &self,
        request: &OperatorCommandRoutingRequest,
    ) -> Result<RoutedOperatorCommand, CommunicationError> {
        request.validate()?;

        let (action, accepted, reason) = route_command(&request.command, &request.config);
        let route = RoutedOperatorCommand {
            id: format!("command-route:{}", request.command.id),
            request_id: request.id.clone(),
            command_id: request.command.id.clone(),
            communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
            action,
            accepted,
            execution_enabled: false,
            outbound_network_used: false,
            reason,
        };
        route.validate()?;
        Ok(route)
    }
}

/// Notification severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationSeverity {
    /// Informational operator message.
    Info,
    /// Warning-level operator message.
    Warning,
    /// Security-sensitive operator message.
    Security,
    /// Critical operator message.
    Critical,
}

/// Operator notification payload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorNotification {
    /// Stable notification id.
    pub id: String,
    /// Severity.
    pub severity: NotificationSeverity,
    /// Non-secret title.
    pub title: String,
    /// Non-secret body.
    pub body: String,
    /// Optional explicit channel ids. If empty, enabled config channels are used.
    pub channels: Vec<String>,
    /// Creation time in Unix milliseconds, or zero in tests/offline construction.
    pub created_at_unix_ms: u64,
}

impl OperatorNotification {
    /// Validate notification payload before local dispatch records are produced.
    pub fn validate(&self, max_message_chars: usize) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("notification", &self.id, &mut violations);

        if self.title.trim().is_empty() {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_NOTIFICATION_TITLE_EMPTY",
                "notification title must be non-empty",
            ));
        }
        if self.body.trim().is_empty() {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_NOTIFICATION_BODY_EMPTY",
                "notification body must be non-empty",
            ));
        }
        if self.title.chars().count() + self.body.chars().count() > max_message_chars {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_NOTIFICATION_TOO_LONG",
                "notification title and body exceed configured max_message_chars",
            ));
        }
        if contains_secret_like_text(&self.title) || contains_secret_like_text(&self.body) {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_NOTIFICATION_SECRET_LIKE",
                "notification text looks like it may contain secret material",
            ));
        }

        for channel in &self.channels {
            validate_id("notification target channel", channel, &mut violations);
            if contains_secret_like_text(channel) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_NOTIFICATION_CHANNEL_SECRET_LIKE",
                    "notification target channel looks like it may contain secret material",
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Notification publish request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationPublishRequest {
    /// Stable publish request id.
    pub id: String,
    /// Notification payload.
    pub notification: OperatorNotification,
    /// Communication safety config.
    pub config: CommunicationBoundaryConfig,
    /// Runtime clock in Unix milliseconds used for records.
    pub now_unix_ms: u64,
}

impl NotificationPublishRequest {
    /// Validate a notification publish request.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("notification publish request", &self.id, &mut violations);

        if let Err(CommunicationError::ValidationFailed {
            violations: config_violations,
        }) = self.config.validate()
        {
            violations.extend(config_violations);
        }

        if let Err(CommunicationError::ValidationFailed {
            violations: notification_violations,
        }) = self.notification.validate(self.config.max_message_chars)
        {
            violations.extend(notification_violations);
        }

        if self.now_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PUBLISH_TIME_ZERO",
                "now_unix_ms must be non-zero",
            ));
        }

        finish_validation(violations)
    }
}

/// Overall notification dispatch status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationDispatchStatus {
    /// Notification was recorded locally for enabled channel boundaries.
    RecordedLocally,
    /// No enabled notification channels were available.
    BlockedNoChannels,
    /// Outbound network delivery is disabled by Phase 12.
    BlockedOutboundNetwork,
}

/// Per-channel dispatch status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationChannelDispatchStatus {
    /// Channel boundary was recorded locally only.
    RecordedLocally,
    /// Channel was disabled.
    Disabled,
    /// Channel would require outbound network delivery and was blocked.
    OutboundNetworkBlocked,
}

/// One per-channel notification dispatch record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationChannelDispatch {
    /// Channel id.
    pub channel_id: String,
    /// Channel kind.
    pub kind: CommunicationChannelKind,
    /// Channel dispatch status.
    pub status: NotificationChannelDispatchStatus,
    /// Whether outbound network was used. Always false in Phase 12.
    pub outbound_network_used: bool,
    /// Non-secret reason.
    pub reason: String,
}

/// Deterministic notification dispatch record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationDispatchRecord {
    /// Stable dispatch id.
    pub id: String,
    /// Source publish request id.
    pub request_id: String,
    /// Source notification id.
    pub notification_id: String,
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Overall status.
    pub status: NotificationDispatchStatus,
    /// Locally redacted title.
    pub redacted_title: String,
    /// Locally redacted body.
    pub redacted_body: String,
    /// Dispatch creation time.
    pub created_at_unix_ms: u64,
    /// Per-channel dispatch records.
    pub channels: Vec<NotificationChannelDispatch>,
    /// Whether outbound network was used. Always false in Phase 12.
    pub outbound_network_used: bool,
}

impl NotificationDispatchRecord {
    /// Validate notification dispatch invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("notification dispatch", &self.id, &mut violations);
        validate_id(
            "notification publish request",
            &self.request_id,
            &mut violations,
        );
        validate_id("notification", &self.notification_id, &mut violations);

        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }

        if self.outbound_network_used {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DISPATCH_OUTBOUND_NETWORK_USED",
                "Phase 12 notification dispatch records must not use outbound network delivery",
            ));
        }

        if contains_secret_like_text(&self.redacted_title)
            || contains_secret_like_text(&self.redacted_body)
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DISPATCH_SECRET_LIKE",
                "redacted notification text still looks like it may contain secret material",
            ));
        }

        for channel in &self.channels {
            if channel.outbound_network_used {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_OUTBOUND_NETWORK_USED",
                    format!(
                        "channel {} used outbound network delivery",
                        channel.channel_id
                    ),
                ));
            }
            if contains_secret_like_text(&channel.reason) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_CHANNEL_REASON_SECRET_LIKE",
                    "channel dispatch reason looks like it may contain secret material",
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Notification publisher trait boundary.
pub trait NotificationPublisher {
    /// Stable publisher name for diagnostics and audit records.
    fn publisher_name(&self) -> &str;

    /// Produce local notification dispatch records without external delivery.
    fn publish(
        &self,
        request: &NotificationPublishRequest,
    ) -> Result<NotificationDispatchRecord, CommunicationError>;
}

/// Deterministic Phase 12 notification boundary.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicNotificationBoundary;

impl DeterministicNotificationBoundary {
    /// Create a deterministic notification boundary.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl NotificationPublisher for DeterministicNotificationBoundary {
    fn publisher_name(&self) -> &str {
        "deterministic-phase-12-notification-boundary"
    }

    fn publish(
        &self,
        request: &NotificationPublishRequest,
    ) -> Result<NotificationDispatchRecord, CommunicationError> {
        request.validate()?;

        let channel_profiles = selected_channels(request);
        let channels = channel_profiles
            .iter()
            .map(channel_dispatch_for)
            .collect::<Vec<_>>();
        let status = dispatch_status_for(&channels, &channel_profiles);
        let record = NotificationDispatchRecord {
            id: format!("notification-dispatch:{}", request.notification.id),
            request_id: request.id.clone(),
            notification_id: request.notification.id.clone(),
            communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
            status,
            redacted_title: redact_operator_message(
                &request.notification.title,
                request.config.max_message_chars,
            ),
            redacted_body: redact_operator_message(
                &request.notification.body,
                request.config.max_message_chars,
            ),
            created_at_unix_ms: request.now_unix_ms,
            channels,
            outbound_network_used: false,
        };
        record.validate()?;
        Ok(record)
    }
}

/// Redact or truncate operator-facing text before display or local dispatch records.
#[must_use]
pub fn redact_operator_message(message: &str, max_chars: usize) -> String {
    let redacted_lines = message
        .lines()
        .map(|line| {
            if contains_secret_like_text(line) {
                "[REDACTED]".to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    truncate_chars(&redacted_lines, max_chars)
}

fn command_kind_for(args: &[String]) -> OperatorCommandKind {
    let Some(first) = args.first() else {
        return OperatorCommandKind::Status;
    };
    let lower = first.to_ascii_lowercase();

    match lower.as_str() {
        "--help" | "-h" | "help" => OperatorCommandKind::Help,
        "--config" | "validate-config" => OperatorCommandKind::ValidateConfig,
        "status" => OperatorCommandKind::Status,
        "safety" | "show-safety" => OperatorCommandKind::ShowSafety,
        "roadmap" | "roadmap-position" => OperatorCommandKind::ShowRoadmapPosition,
        "plan" | "plan-only" => OperatorCommandKind::PlanOnly,
        "trade" | "execute" | "order" | "swap" | "live" => {
            OperatorCommandKind::LiveExecutionRequest
        }
        "withdraw" | "withdrawal" => OperatorCommandKind::WithdrawalRequest,
        "bridge" => OperatorCommandKind::BridgeRequest,
        "sign" | "sign-transaction" => OperatorCommandKind::SignRequest,
        "broadcast" | "submit-transaction" => OperatorCommandKind::BroadcastRequest,
        _ => OperatorCommandKind::Unknown,
    }
}

fn command_args_for(args: &[String], kind: OperatorCommandKind) -> Vec<String> {
    match kind {
        OperatorCommandKind::ValidateConfig => args
            .get(1)
            .map_or_else(Vec::new, |value| vec![value.clone()]),
        _ => args.iter().skip(1).cloned().collect(),
    }
}

fn route_command(
    command: &OperatorCommand,
    config: &CommunicationBoundaryConfig,
) -> (OperatorCommandAction, bool, String) {
    if command.source == OperatorCommandSource::LocalCli && !config.cli_enabled {
        return (
            OperatorCommandAction::RejectUnsafeCommand,
            false,
            "local CLI command routing is disabled by configuration".to_owned(),
        );
    }

    if command.kind.is_unsafe_execution_request() {
        return (
            OperatorCommandAction::RejectUnsafeCommand,
            false,
            "Phase 12 command routing rejects live execution, withdrawals, bridges, signing, and broadcasts".to_owned(),
        );
    }

    match command.kind {
        OperatorCommandKind::Status => (
            OperatorCommandAction::ShowStatus,
            true,
            "status command accepted for local display only".to_owned(),
        ),
        OperatorCommandKind::Help => (
            OperatorCommandAction::ShowHelp,
            true,
            "help command accepted for local display only".to_owned(),
        ),
        OperatorCommandKind::ValidateConfig => command.args.first().map_or_else(
            || {
                (
                    OperatorCommandAction::RejectUnknownCommand,
                    false,
                    "validate-config requires a path argument".to_owned(),
                )
            },
            |path| {
                (
                    OperatorCommandAction::ValidateConfigFile { path: path.clone() },
                    true,
                    "config validation command accepted; loading remains local and non-secret"
                        .to_owned(),
                )
            },
        ),
        OperatorCommandKind::ShowSafety => (
            OperatorCommandAction::ShowSafety,
            true,
            "safety command accepted for local display only".to_owned(),
        ),
        OperatorCommandKind::ShowRoadmapPosition => (
            OperatorCommandAction::ShowRoadmapPosition,
            true,
            "roadmap command accepted for local display only".to_owned(),
        ),
        OperatorCommandKind::PlanOnly => (
            OperatorCommandAction::PlanOnlyBoundary,
            true,
            "plan-only command accepted as a non-executing boundary".to_owned(),
        ),
        OperatorCommandKind::Unknown => (
            OperatorCommandAction::RejectUnknownCommand,
            false,
            "unknown command rejected; use help for available local commands".to_owned(),
        ),
        OperatorCommandKind::LiveExecutionRequest
        | OperatorCommandKind::WithdrawalRequest
        | OperatorCommandKind::BridgeRequest
        | OperatorCommandKind::SignRequest
        | OperatorCommandKind::BroadcastRequest => {
            unreachable!("unsafe commands are returned above")
        }
    }
}

fn selected_channels(request: &NotificationPublishRequest) -> Vec<NotificationChannelProfile> {
    if request.notification.channels.is_empty() {
        return request
            .config
            .notification_channels
            .iter()
            .filter(|channel| channel.enabled)
            .cloned()
            .collect();
    }

    request
        .notification
        .channels
        .iter()
        .map(|channel| NotificationChannelProfile::from_identifier(channel))
        .collect()
}

fn channel_dispatch_for(channel: &NotificationChannelProfile) -> NotificationChannelDispatch {
    if !channel.enabled {
        return NotificationChannelDispatch {
            channel_id: channel.id.clone(),
            kind: channel.kind,
            status: NotificationChannelDispatchStatus::Disabled,
            outbound_network_used: false,
            reason: "channel is disabled".to_owned(),
        };
    }

    match channel.kind {
        CommunicationChannelKind::Cli | CommunicationChannelKind::LocalStdout => {
            NotificationChannelDispatch {
                channel_id: channel.id.clone(),
                kind: channel.kind,
                status: NotificationChannelDispatchStatus::RecordedLocally,
                outbound_network_used: false,
                reason: "local channel recorded without network delivery".to_owned(),
            }
        }
        CommunicationChannelKind::Email
        | CommunicationChannelKind::Chat
        | CommunicationChannelKind::Webhook
        | CommunicationChannelKind::Pager
        | CommunicationChannelKind::Other => NotificationChannelDispatch {
            channel_id: channel.id.clone(),
            kind: channel.kind,
            status: NotificationChannelDispatchStatus::OutboundNetworkBlocked,
            outbound_network_used: false,
            reason: "outbound notification integration is disabled in Phase 12".to_owned(),
        },
    }
}

fn dispatch_status_for(
    channels: &[NotificationChannelDispatch],
    profiles: &[NotificationChannelProfile],
) -> NotificationDispatchStatus {
    if profiles.is_empty() {
        return NotificationDispatchStatus::BlockedNoChannels;
    }

    if channels
        .iter()
        .all(|channel| channel.status == NotificationChannelDispatchStatus::OutboundNetworkBlocked)
    {
        return NotificationDispatchStatus::BlockedOutboundNetwork;
    }

    NotificationDispatchStatus::RecordedLocally
}

fn validate_id(label: &str, value: &str, violations: &mut Vec<CommunicationViolation>) {
    if value.trim().is_empty() {
        violations.push(CommunicationViolation::new_owned(
            "COMMUNICATION_ID_EMPTY",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn contains_secret_like_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let secret_terms = [
        "api key",
        "api-key",
        "apikey",
        "private key",
        "private-key",
        "seed phrase",
        "seed-phrase",
        "mnemonic",
        "secret=",
        "secret:",
        "token=",
        "token:",
    ];

    if secret_terms.iter().any(|term| lower.contains(term)) {
        return true;
    }

    value.split_whitespace().any(|segment| {
        segment.chars().count() >= 48
            && segment.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '_' | '-' | '/' | '+' | '=' | '.')
            })
    })
}

fn sanitize_command_text(value: &str) -> String {
    redact_operator_message(value, 500)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if max_chars == 0 || value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let mut truncated = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

fn command_kind_label(kind: OperatorCommandKind) -> &'static str {
    match kind {
        OperatorCommandKind::Status => "status",
        OperatorCommandKind::Help => "help",
        OperatorCommandKind::ValidateConfig => "validate-config",
        OperatorCommandKind::ShowSafety => "show-safety",
        OperatorCommandKind::ShowRoadmapPosition => "show-roadmap-position",
        OperatorCommandKind::PlanOnly => "plan-only",
        OperatorCommandKind::LiveExecutionRequest => "live-execution-request",
        OperatorCommandKind::WithdrawalRequest => "withdrawal-request",
        OperatorCommandKind::BridgeRequest => "bridge-request",
        OperatorCommandKind::SignRequest => "sign-request",
        OperatorCommandKind::BroadcastRequest => "broadcast-request",
        OperatorCommandKind::Unknown => "unknown",
    }
}

fn finish_validation(violations: Vec<CommunicationViolation>) -> Result<(), CommunicationError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(CommunicationError::ValidationFailed { violations })
    }
}

/// One communication validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationViolation {
    code: &'static str,
    message: String,
}

impl CommunicationViolation {
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

/// Communications and CLI boundary errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationError {
    /// Validation failed with deterministic violations.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<CommunicationViolation>,
    },
}

impl CommunicationError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[CommunicationViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
        }
    }
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "communication boundary validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for CommunicationError {}

#[cfg(test)]
mod tests {
    use super::{
        parse_cli_command, CommunicationBoundaryConfig, DeterministicNotificationBoundary,
        DeterministicOperatorCommandRouter, NotificationDispatchStatus, NotificationPublishRequest,
        NotificationPublisher, NotificationSeverity, OperatorCommandAction, OperatorCommandKind,
        OperatorCommandRouter, OperatorCommandRoutingRequest, OperatorNotification,
    };
    use crate::CommunicationConfig;

    #[test]
    fn communication_config_rejects_outbound_network_and_execution_commands() {
        let config = CommunicationBoundaryConfig {
            outbound_network_enabled: true,
            allow_execution_commands: true,
            ..CommunicationBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("unsafe communication settings must be rejected");
        assert!(error.violations().iter().any(|violation| {
            violation.code() == "COMMUNICATION_OUTBOUND_NETWORK_DENIED_IN_PHASE_12"
        }));
        assert!(error.violations().iter().any(|violation| {
            violation.code() == "COMMUNICATION_EXECUTION_COMMANDS_DENIED_IN_PHASE_12"
        }));
    }

    #[test]
    fn command_router_accepts_status_without_execution_or_network() {
        let command =
            parse_cli_command(&["status".to_owned()], 1_000).expect("status command should parse");
        let request = OperatorCommandRoutingRequest {
            id: "route-request-1".to_owned(),
            command,
            config: CommunicationBoundaryConfig::default(),
        };

        let route = DeterministicOperatorCommandRouter::new()
            .route(&request)
            .expect("status command should route");

        assert_eq!(route.action, OperatorCommandAction::ShowStatus);
        assert!(route.accepted);
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
    }

    #[test]
    fn command_router_rejects_live_execution_request() {
        let command = parse_cli_command(&["execute".to_owned()], 1_000)
            .expect("execute command should parse as blocked request");
        assert_eq!(command.kind, OperatorCommandKind::LiveExecutionRequest);

        let request = OperatorCommandRoutingRequest {
            id: "route-request-2".to_owned(),
            command,
            config: CommunicationBoundaryConfig::default(),
        };

        let route = DeterministicOperatorCommandRouter::new()
            .route(&request)
            .expect("unsafe command should produce a rejection route");

        assert_eq!(route.action, OperatorCommandAction::RejectUnsafeCommand);
        assert!(!route.accepted);
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
    }

    #[test]
    fn notification_boundary_records_local_channels_only() {
        let runtime_config = CommunicationConfig {
            cli_enabled: true,
            notify_channels: vec!["cli".to_owned(), "email:ops-alerts".to_owned()],
        };
        let config = CommunicationBoundaryConfig::from_config(&runtime_config);
        let request = NotificationPublishRequest {
            id: "publish-request-1".to_owned(),
            notification: OperatorNotification {
                id: "notification-1".to_owned(),
                severity: NotificationSeverity::Info,
                title: "Agent status".to_owned(),
                body: "Phase 12 communication boundary recorded a local status notification"
                    .to_owned(),
                channels: Vec::new(),
                created_at_unix_ms: 1_000,
            },
            config,
            now_unix_ms: 1_001,
        };

        let record = DeterministicNotificationBoundary::new()
            .publish(&request)
            .expect("notification should produce a local dispatch record");

        assert_eq!(record.status, NotificationDispatchStatus::RecordedLocally);
        assert!(!record.outbound_network_used);
        assert_eq!(record.channels.len(), 2);
        assert!(record
            .channels
            .iter()
            .all(|channel| !channel.outbound_network_used));
    }

    #[test]
    fn notification_rejects_secret_like_text() {
        let request = NotificationPublishRequest {
            id: "publish-request-2".to_owned(),
            notification: OperatorNotification {
                id: "notification-2".to_owned(),
                severity: NotificationSeverity::Security,
                title: "Operator pasted private key material".to_owned(),
                body: "Reject this before dispatch".to_owned(),
                channels: vec!["cli".to_owned()],
                created_at_unix_ms: 1_000,
            },
            config: CommunicationBoundaryConfig::default(),
            now_unix_ms: 1_001,
        };

        let error = DeterministicNotificationBoundary::new()
            .publish(&request)
            .expect_err("secret-like notification must be rejected");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "COMMUNICATION_NOTIFICATION_SECRET_LIKE" }));
    }
}
