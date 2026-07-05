#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue,
    CommunicationConfig, StateCheckpoint, StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

/// Stable communications and CLI boundary version for audit, replay, and handoff surfaces.
pub const COMMUNICATIONS_CLI_VERSION: &str = "phase-12-communications-cli-v1";

/// State-store subsystem name for local communications checkpoints.
pub const COMMUNICATIONS_STATE_SUBSYSTEM: &str = "communications";

/// State-store key for the latest routed operator command.
pub const COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY: &str =
    "communications:last-command-route";

/// State-store key for the latest notification dispatch.
pub const COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY: &str =
    "communications:last-notification-dispatch";

/// State-store key for the latest remote command security review.
pub const COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY: &str =
    "communications:last-remote-command-review";

/// State-store key for the latest local remote command envelope validation.
pub const COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY: &str =
    "communications:last-remote-command-envelope";

/// State-store key for the latest local platform command ingress review.
pub const COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY: &str =
    "communications:last-platform-command-ingress";

/// State-store key for the latest local authenticated channel adapter validation.
pub const COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY: &str =
    "communications:last-channel-adapter-validation";

/// State-store key for the latest local channel session validation summary.
pub const COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY: &str =
    "communications:last-channel-session-validation";

/// State-store key for the latest local platform adapter control review.
pub const COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY: &str =
    "communications:last-platform-adapter-review";

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
    /// Whether command routing requires a local operator authorization decision.
    pub require_local_operator_authorization: bool,
    /// Whether future remote operator command sources are enabled. Phase 12 requires false.
    pub remote_operator_commands_enabled: bool,
}

impl Default for CommunicationBoundaryConfig {
    fn default() -> Self {
        Self {
            cli_enabled: true,
            notification_channels: Vec::new(),
            max_message_chars: 2_000,
            outbound_network_enabled: false,
            allow_execution_commands: false,
            require_local_operator_authorization: true,
            remote_operator_commands_enabled: false,
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

        if !self.require_local_operator_authorization {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_LOCAL_OPERATOR_AUTH_REQUIRED",
                "Phase 12 command routing must require local operator authorization",
            ));
        }

        if self.remote_operator_commands_enabled {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_OPERATOR_COMMANDS_DENIED_IN_PHASE_12",
                "Phase 12 command routing must not enable remote operator commands",
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
    /// Reject unauthenticated or unauthorized operator command source.
    RejectUnauthorizedCommand,
}

/// Local operator command authorization status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorCommandAuthorizationStatus {
    /// Local CLI command source is authorized for local handling.
    AuthorizedLocalCli,
    /// Local CLI command source is disabled by configuration.
    RejectedCliDisabled,
    /// Remote/scheduled/dashboard command source is not enabled in this phase.
    RejectedRemoteSource,
}

/// Local remote command security review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteCommandSecurityReviewStatus {
    /// Required controls are represented locally, but remote commands are still disabled.
    ReadyForLocalReview,
    /// Required controls are missing or unsafe side-effect flags were requested.
    BlockedMissingControls,
}

/// Local remote command security review request.
///
/// This is a non-secret review boundary for future remote operator channels. It
/// never enables remote commands, authenticates a real platform, delivers
/// messages, routes commands, performs network calls, or allows execution.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteCommandSecurityReviewRequest {
    /// Stable review id.
    pub review_id: String,
    /// Remote source being reviewed.
    pub source: OperatorCommandSource,
    /// Whether the future channel requires authentication.
    pub channel_authentication_required: bool,
    /// Whether platform identity must be verified.
    pub platform_identity_verification_required: bool,
    /// Whether platform identity authorization policy is required.
    pub platform_identity_authorization_required: bool,
    /// Whether replay protection is required for future commands.
    pub replay_protection_required: bool,
    /// Whether command allowlisting is required.
    pub command_allowlist_required: bool,
    /// Whether unsafe execution/funds-movement commands remain blocked.
    pub unsafe_commands_blocked: bool,
    /// Whether remote command enablement was requested. Must remain false here.
    pub remote_command_enablement_requested: bool,
    /// Whether outbound network use was requested. Must remain false here.
    pub outbound_network_requested: bool,
    /// Whether live execution was requested. Must remain false here.
    pub live_execution_requested: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local remote command security review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteCommandSecurityReviewReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable review id.
    pub review_id: String,
    /// Remote source reviewed.
    pub source: OperatorCommandSource,
    /// Review status.
    pub status: RemoteCommandSecurityReviewStatus,
    /// Whether the future channel requires authentication.
    pub channel_authentication_required: bool,
    /// Whether platform identity must be verified.
    pub platform_identity_verification_required: bool,
    /// Whether platform identity authorization policy is required.
    pub platform_identity_authorization_required: bool,
    /// Whether replay protection is required for future commands.
    pub replay_protection_required: bool,
    /// Whether command allowlisting is required.
    pub command_allowlist_required: bool,
    /// Whether unsafe execution/funds-movement commands remain blocked.
    pub unsafe_commands_blocked: bool,
    /// Number of missing or unsafe local controls.
    pub missing_control_count: u64,
    /// Whether remote commands were enabled. Always false for this review.
    pub remote_commands_enabled: bool,
    /// Whether outbound network was used. Always false for this review.
    pub outbound_network_used: bool,
    /// Whether live execution was performed. Always false for this review.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast was performed. Always false for this review.
    pub signing_or_broadcast_performed: bool,
    /// Whether this review approves production readiness. Always false.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local remote command envelope validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteCommandEnvelopeValidationStatus {
    /// Envelope controls are coherent for local review; routing still remains disabled.
    ReadyForLocalReview,
    /// Envelope controls are missing, stale, replayed, unsafe, or side-effectful.
    BlockedMissingControls,
}

/// Local authenticated channel adapter validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelAdapterValidationStatus {
    /// Adapter controls are coherent for local review; no outbound delivery occurred.
    ReadyForLocalReview,
    /// Adapter controls are missing, replayed, rate-limited, outage-blocked, or side-effectful.
    BlockedMissingControls,
}

/// Local mocked platform command ingress status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlatformCommandIngressStatus {
    /// Mocked platform command can continue to local envelope validation.
    ReadyForEnvelopeValidation,
    /// Mocked platform command is missing required controls or side-effect-free invariants.
    BlockedMissingControls,
}

/// Local remote command envelope validation input.
///
/// This validates non-secret metadata for a future remote command channel. It
/// does not authenticate a real platform, verify signatures, call a network,
/// route a remote command, or enable remote execution.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteCommandEnvelopeValidationRequest {
    /// Stable local envelope id.
    pub envelope_id: String,
    /// Parsed command carried by the envelope.
    pub command: OperatorCommand,
    /// Security review that established required local controls.
    pub security_review: RemoteCommandSecurityReviewReport,
    /// Non-secret platform identity label.
    pub platform_identity: String,
    /// Non-secret authorization policy label.
    pub authorization_policy: String,
    /// Non-secret authentication proof reference or digest.
    pub authentication_reference: String,
    /// Non-secret replay nonce/reference.
    pub replay_nonce: String,
    /// Whether channel authentication was represented as successful.
    pub channel_authenticated: bool,
    /// Whether platform identity was represented as verified.
    pub platform_identity_verified: bool,
    /// Whether platform identity was represented as authorized.
    pub platform_identity_authorized: bool,
    /// Whether replay protection was represented as checked.
    pub replay_protection_checked: bool,
    /// Whether this envelope nonce was already seen.
    pub replay_nonce_reused: bool,
    /// Whether the command kind is allowed for the represented remote channel.
    pub command_allowlisted: bool,
    /// Envelope receipt time in Unix milliseconds.
    pub received_at_unix_ms: u64,
    /// Current local validation time in Unix milliseconds.
    pub now_unix_ms: u64,
    /// Maximum envelope age in milliseconds.
    pub max_age_ms: u64,
    /// Whether remote command enablement was requested. Must remain false.
    pub remote_command_enablement_requested: bool,
    /// Whether outbound network use occurred. Must remain false.
    pub outbound_network_used: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Must remain false.
    pub signing_or_broadcast_performed: bool,
}

/// Local remote command envelope validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteCommandEnvelopeValidationReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable local envelope id.
    pub envelope_id: String,
    /// Source command id.
    pub command_id: String,
    /// Command source.
    pub source: OperatorCommandSource,
    /// Parsed command kind.
    pub command_kind: OperatorCommandKind,
    /// Validation status.
    pub status: RemoteCommandEnvelopeValidationStatus,
    /// Whether the security review was ready for local review.
    pub security_review_ready: bool,
    /// Whether channel authentication was represented as successful.
    pub channel_authenticated: bool,
    /// Whether platform identity was represented as verified.
    pub platform_identity_verified: bool,
    /// Whether platform identity was represented as authorized.
    pub platform_identity_authorized: bool,
    /// Whether replay protection was represented as checked.
    pub replay_protection_checked: bool,
    /// Whether this envelope nonce was already seen.
    pub replay_nonce_reused: bool,
    /// Whether the command kind is allowed for the represented remote channel.
    pub command_allowlisted: bool,
    /// Whether sanitized command text or args contain shell/control-injection markers.
    pub command_injection_detected: bool,
    /// Whether the envelope is stale by `max_age_ms`.
    pub stale_envelope: bool,
    /// Number of missing or unsafe controls.
    pub missing_control_count: u64,
    /// Whether remote commands were enabled. Always false for this validation.
    pub remote_commands_enabled: bool,
    /// Whether outbound network was used. Always false for ready reports.
    pub outbound_network_used: bool,
    /// Whether live execution occurred. Always false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Always false.
    pub signing_or_broadcast_performed: bool,
    /// Whether production readiness is approved. Always false.
    pub production_ready: bool,
}

/// Local mocked platform command ingress input.
///
/// This models a caller-supplied platform message without calling a platform,
/// loading tokens, delivering messages, or enabling remote command routing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformCommandIngressRequest {
    /// Stable local ingress id.
    pub ingress_id: String,
    /// Non-secret platform label such as `slack-mock` or `discord-mock`.
    pub platform: String,
    /// Non-secret channel profile.
    pub channel: NotificationChannelProfile,
    /// Non-secret platform message id.
    pub platform_message_id: String,
    /// Non-secret platform identity label.
    pub platform_identity: String,
    /// Mocked command text supplied by the platform fixture.
    pub command_text: String,
    /// Whether a non-secret token reference or alias exists.
    pub token_reference_present: bool,
    /// Whether raw token material was observed. Must remain false.
    pub token_secret_material_present: bool,
    /// Whether caller-supplied platform signature/authentication verified.
    pub platform_signature_verified: bool,
    /// Whether platform identity authorization was represented as successful.
    pub platform_identity_authorized: bool,
    /// Whether the channel permission was represented as granted.
    pub channel_permission_granted: bool,
    /// Non-secret replay nonce/reference.
    pub replay_nonce: String,
    /// Whether this message nonce was already seen.
    pub replay_nonce_reused: bool,
    /// Whether caller-supplied provider rate-limit observation blocks processing.
    pub provider_rate_limited: bool,
    /// Whether caller-supplied provider outage observation blocks processing.
    pub provider_outage_observed: bool,
    /// Message receipt time in Unix milliseconds.
    pub received_at_unix_ms: u64,
    /// Current local validation time in Unix milliseconds.
    pub now_unix_ms: u64,
    /// Maximum mocked platform message age in milliseconds.
    pub max_age_ms: u64,
    /// Whether outbound network was used. Must remain false.
    pub outbound_network_used: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Must remain false.
    pub signing_or_broadcast_performed: bool,
}

/// Local mocked platform command ingress report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformCommandIngressReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable local ingress id.
    pub ingress_id: String,
    /// Non-secret platform label.
    pub platform: String,
    /// Channel id.
    pub channel_id: String,
    /// Channel kind.
    pub channel_kind: CommunicationChannelKind,
    /// Non-secret platform message id.
    pub platform_message_id: String,
    /// Non-secret platform identity label.
    pub platform_identity: String,
    /// Parsed sanitized remote operator command.
    pub command: OperatorCommand,
    /// Ingress status.
    pub status: PlatformCommandIngressStatus,
    /// Whether a non-secret token reference or alias exists.
    pub token_reference_present: bool,
    /// Whether raw token material was observed. Always false for ready reports.
    pub token_secret_material_present: bool,
    /// Whether caller-supplied platform signature/authentication verified.
    pub platform_signature_verified: bool,
    /// Whether platform identity authorization was represented as successful.
    pub platform_identity_authorized: bool,
    /// Whether the channel permission was represented as granted.
    pub channel_permission_granted: bool,
    /// Whether this message nonce was already seen.
    pub replay_nonce_reused: bool,
    /// Whether command text/args contain injection markers.
    pub command_injection_detected: bool,
    /// Whether the mocked platform message is stale.
    pub stale_message: bool,
    /// Whether caller-supplied provider rate-limit observation blocks processing.
    pub provider_rate_limited: bool,
    /// Whether caller-supplied provider outage observation blocks processing.
    pub provider_outage_observed: bool,
    /// Number of missing or unsafe controls.
    pub missing_control_count: u64,
    /// Whether remote commands were enabled. Always false.
    pub remote_commands_enabled: bool,
    /// Whether outbound network was used. Always false.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Always false.
    pub message_delivered: bool,
    /// Whether live execution occurred. Always false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Always false.
    pub signing_or_broadcast_performed: bool,
    /// Whether production readiness is approved. Always false.
    pub production_ready: bool,
}

/// Local authenticated channel adapter validation request.
///
/// This models the final non-secret adapter seam between a reviewed remote
/// command envelope and a notification dispatch record. It does not hold
/// platform tokens, call a channel API, deliver messages, or enable remote
/// commands.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelAdapterValidationRequest {
    /// Stable local validation id.
    pub validation_id: String,
    /// Non-secret channel profile being validated.
    pub channel: NotificationChannelProfile,
    /// Remote envelope validation that authorized local review.
    pub envelope: RemoteCommandEnvelopeValidationReport,
    /// Local notification dispatch associated with this adapter check.
    pub dispatch: NotificationDispatchRecord,
    /// Non-secret adapter authentication reference or digest.
    pub adapter_authentication_reference: String,
    /// Non-secret platform identity label.
    pub platform_identity: String,
    /// Non-secret replay nonce/reference for this adapter handoff.
    pub replay_nonce: String,
    /// Whether channel authentication was represented as successful.
    pub channel_authenticated: bool,
    /// Whether platform identity was represented as authorized.
    pub platform_identity_authorized: bool,
    /// Whether replay protection was represented as checked.
    pub replay_protection_checked: bool,
    /// Whether a future outbound adapter must retain a delivery kill switch.
    pub require_delivery_kill_switch: bool,
    /// Whether future outbound delivery requires audit/state preflight.
    pub require_audit_state_preflight: bool,
    /// Whether future outbound delivery requires idempotency controls.
    pub require_delivery_idempotency: bool,
    /// Whether future outbound delivery requires rate-limit controls.
    pub require_rate_limit_controls: bool,
    /// Whether future outbound delivery requires outage/backoff controls.
    pub require_outage_backoff_controls: bool,
    /// Whether future outbound delivery requires payload redaction controls.
    pub require_payload_redaction: bool,
    /// Whether this adapter nonce was already used.
    pub replay_nonce_reused: bool,
    /// Whether a caller-supplied local provider rate limit observation blocks delivery.
    pub provider_rate_limited: bool,
    /// Whether a caller-supplied local outage observation blocks delivery.
    pub provider_outage_observed: bool,
    /// Whether outbound delivery was requested. Must remain false here.
    pub outbound_delivery_requested: bool,
    /// Whether outbound network was used. Must remain false.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Must remain false.
    pub message_delivered: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Must remain false.
    pub signing_or_broadcast_performed: bool,
    /// Operator-supplied non-secret validation timestamp.
    pub validated_at_unix_ms: u64,
}

/// Local authenticated channel adapter validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelAdapterValidationReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable local validation id.
    pub validation_id: String,
    /// Channel id.
    pub channel_id: String,
    /// Channel kind.
    pub channel_kind: CommunicationChannelKind,
    /// Source remote envelope id.
    pub envelope_id: String,
    /// Source dispatch id.
    pub dispatch_id: String,
    /// Validation status.
    pub status: ChannelAdapterValidationStatus,
    /// Whether the remote envelope was ready for local review.
    pub envelope_ready: bool,
    /// Whether the local notification dispatch was recorded locally.
    pub dispatch_recorded_locally: bool,
    /// Whether channel authentication was represented as successful.
    pub channel_authenticated: bool,
    /// Whether platform identity was represented as authorized.
    pub platform_identity_authorized: bool,
    /// Whether replay protection was represented as checked.
    pub replay_protection_checked: bool,
    /// Whether a future outbound adapter must retain a delivery kill switch.
    pub require_delivery_kill_switch: bool,
    /// Whether future outbound delivery requires audit/state preflight.
    pub require_audit_state_preflight: bool,
    /// Whether future outbound delivery requires idempotency controls.
    pub require_delivery_idempotency: bool,
    /// Whether future outbound delivery requires rate-limit controls.
    pub require_rate_limit_controls: bool,
    /// Whether future outbound delivery requires outage/backoff controls.
    pub require_outage_backoff_controls: bool,
    /// Whether future outbound delivery requires payload redaction controls.
    pub require_payload_redaction: bool,
    /// Whether this adapter nonce was already used.
    pub replay_nonce_reused: bool,
    /// Whether a caller-supplied provider rate limit blocks delivery.
    pub provider_rate_limited: bool,
    /// Whether a caller-supplied provider outage blocks delivery.
    pub provider_outage_observed: bool,
    /// Number of missing or unsafe controls.
    pub missing_control_count: u64,
    /// Whether outbound delivery was requested. Always false for ready reports.
    pub outbound_delivery_requested: bool,
    /// Whether outbound network was used. Always false.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Always false.
    pub message_delivered: bool,
    /// Whether remote commands were enabled. Always false.
    pub remote_commands_enabled: bool,
    /// Whether live execution occurred. Always false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Always false.
    pub signing_or_broadcast_performed: bool,
    /// Whether production readiness is approved. Always false.
    pub production_ready: bool,
    /// Operator-supplied non-secret validation timestamp.
    pub validated_at_unix_ms: u64,
}

/// Local channel session validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelSessionValidationStatus {
    /// Local channel controls accepted one adapter path and rejected unsafe shapes.
    ReadyForLocalReview,
    /// Required channel controls were not proven.
    BlockedMissingControls,
}

/// Local platform adapter control review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlatformAdapterReviewStatus {
    /// Platform-adapter controls are coherent for local review only.
    ReadyForLocalReview,
    /// Platform-adapter controls are missing, revoked, rate-limited, outage-blocked, or side-effectful.
    BlockedMissingControls,
}

/// Local channel session validation summary.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSessionValidationReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable local session validation id.
    pub session_id: String,
    /// Validation status.
    pub status: ChannelSessionValidationStatus,
    /// Channel adapter validation ids included in this summary.
    pub validation_ids: Vec<String>,
    /// Number of summarized adapter validations.
    pub total_validation_count: u32,
    /// Number of accepted local adapter validations.
    pub accepted_validation_count: u32,
    /// Number of validations rejected for missing channel authentication.
    pub rejected_unauthenticated_count: u32,
    /// Number of validations rejected for replay nonce reuse.
    pub rejected_replay_count: u32,
    /// Number of validations rejected for provider rate-limit or outage observations.
    pub rejected_provider_unavailable_count: u32,
    /// Whether all reports had ready remote envelopes.
    pub envelope_ready: bool,
    /// Whether all reports used local recorded dispatches.
    pub dispatch_recorded_locally: bool,
    /// Total missing-control findings across summarized reports.
    pub missing_control_count: u64,
    /// Whether outbound delivery was requested. Always false here.
    pub outbound_delivery_requested: bool,
    /// Whether outbound network was used. Always false here.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Always false here.
    pub message_delivered: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Always false here.
    pub signing_or_broadcast_performed: bool,
    /// Whether production readiness is approved. Always false here.
    pub production_ready: bool,
}

/// Local platform adapter control review input.
///
/// This models non-secret controls a future messaging adapter would need. It
/// does not store platform tokens, call platform APIs, deliver messages, or
/// enable remote command execution.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformAdapterReviewRequest {
    /// Stable local review id.
    pub review_id: String,
    /// Non-secret channel profile being reviewed.
    pub channel: NotificationChannelProfile,
    /// Remote envelope validation represented at this adapter boundary.
    pub envelope: RemoteCommandEnvelopeValidationReport,
    /// Whether a non-secret token reference or alias exists.
    pub token_reference_present: bool,
    /// Whether raw token material was observed. Must remain false.
    pub token_secret_material_present: bool,
    /// Whether platform identity verification was represented as successful.
    pub platform_identity_verified: bool,
    /// Whether platform identity authorization was represented as successful.
    pub platform_identity_authorized: bool,
    /// Whether the operator/channel permission was represented as granted.
    pub channel_permission_granted: bool,
    /// Whether command-injection-like input was blocked before adapter use.
    pub command_injection_blocked: bool,
    /// Whether a future outbound adapter must retain a delivery kill switch.
    pub require_delivery_kill_switch: bool,
    /// Whether future outbound delivery requires audit/state preflight.
    pub require_audit_state_preflight: bool,
    /// Whether future outbound delivery requires idempotency controls.
    pub require_delivery_idempotency: bool,
    /// Whether future outbound delivery requires rate-limit controls.
    pub require_rate_limit_controls: bool,
    /// Whether future outbound delivery requires outage/backoff controls.
    pub require_outage_backoff_controls: bool,
    /// Whether future outbound delivery requires payload redaction controls.
    pub require_payload_redaction: bool,
    /// Whether the caller-supplied token reference is revoked.
    pub token_revoked: bool,
    /// Whether caller-supplied provider rate-limit observation blocks delivery.
    pub provider_rate_limited: bool,
    /// Whether caller-supplied provider outage observation blocks delivery.
    pub provider_outage_observed: bool,
    /// Whether outbound delivery was requested. Must remain false here.
    pub outbound_delivery_requested: bool,
    /// Whether outbound network was used. Must remain false.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Must remain false.
    pub message_delivered: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Must remain false.
    pub signing_or_broadcast_performed: bool,
    /// Operator-supplied non-secret review timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local platform adapter control review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformAdapterReviewReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable local review id.
    pub review_id: String,
    /// Channel id.
    pub channel_id: String,
    /// Channel kind.
    pub channel_kind: CommunicationChannelKind,
    /// Source remote envelope id.
    pub envelope_id: String,
    /// Review status.
    pub status: PlatformAdapterReviewStatus,
    /// Whether the remote envelope was ready for local review.
    pub envelope_ready: bool,
    /// Whether a non-secret token reference or alias exists.
    pub token_reference_present: bool,
    /// Whether raw token material was observed. Always false for ready reports.
    pub token_secret_material_present: bool,
    /// Whether platform identity verification was represented as successful.
    pub platform_identity_verified: bool,
    /// Whether platform identity authorization was represented as successful.
    pub platform_identity_authorized: bool,
    /// Whether the operator/channel permission was represented as granted.
    pub channel_permission_granted: bool,
    /// Whether command-injection-like input was blocked before adapter use.
    pub command_injection_blocked: bool,
    /// Whether a future outbound adapter must retain a delivery kill switch.
    pub require_delivery_kill_switch: bool,
    /// Whether future outbound delivery requires audit/state preflight.
    pub require_audit_state_preflight: bool,
    /// Whether future outbound delivery requires idempotency controls.
    pub require_delivery_idempotency: bool,
    /// Whether future outbound delivery requires rate-limit controls.
    pub require_rate_limit_controls: bool,
    /// Whether future outbound delivery requires outage/backoff controls.
    pub require_outage_backoff_controls: bool,
    /// Whether future outbound delivery requires payload redaction controls.
    pub require_payload_redaction: bool,
    /// Whether the caller-supplied token reference is revoked.
    pub token_revoked: bool,
    /// Whether caller-supplied provider rate-limit observation blocks delivery.
    pub provider_rate_limited: bool,
    /// Whether caller-supplied provider outage observation blocks delivery.
    pub provider_outage_observed: bool,
    /// Number of missing or unsafe controls.
    pub missing_control_count: u64,
    /// Whether outbound delivery was requested. Always false for ready reports.
    pub outbound_delivery_requested: bool,
    /// Whether outbound network was used. Always false.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Always false.
    pub message_delivered: bool,
    /// Whether remote commands were enabled. Always false.
    pub remote_commands_enabled: bool,
    /// Whether live execution occurred. Always false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Always false.
    pub signing_or_broadcast_performed: bool,
    /// Whether production readiness is approved. Always false.
    pub production_ready: bool,
    /// Operator-supplied non-secret review timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local communications delivery-provider boundary request.
///
/// This composes local channel/session/platform adapter prerequisites and keeps
/// real provider delivery evidence explicit. It does not load tokens, call
/// platform APIs, deliver messages, enable remote commands, or approve
/// production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommunicationDeliveryProviderBoundaryRequest {
    /// Stable local review id.
    pub review_id: String,
    /// Local channel-session validation prerequisite.
    pub channel_session: ChannelSessionValidationReport,
    /// Local platform-adapter review prerequisite.
    pub platform_adapter: PlatformAdapterReviewReport,
    /// Whether authenticated real provider delivery evidence is available.
    pub provider_delivery_evidence_available: bool,
    /// Whether provider-side rate-limit reconciliation evidence is available.
    pub provider_rate_limit_evidence_available: bool,
    /// Whether real provider outage/backoff evidence is available.
    pub provider_outage_evidence_available: bool,
    /// Whether production platform identity/authorization evidence is available.
    pub platform_identity_evidence_available: bool,
    /// Minimum remaining external evidence references required.
    pub min_remaining_external_evidence: usize,
    /// Non-secret descriptions of remaining external delivery evidence.
    pub remaining_external_evidence: Vec<String>,
    /// Whether outbound network was used. Must remain false.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Must remain false.
    pub message_delivered: bool,
    /// Whether a provider API call was performed. Must remain false.
    pub provider_call_performed: bool,
    /// Whether token or secret material was loaded. Must remain false.
    pub token_secret_material_loaded: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Must remain false.
    pub signing_or_broadcast_performed: bool,
    /// Whether this review claims production readiness. Must remain false.
    pub production_ready_claimed: bool,
}

/// Local communications delivery-provider boundary status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommunicationDeliveryProviderBoundaryStatus {
    /// Local prerequisites exist but real provider delivery validation is missing.
    BlockedPendingProviderDeliveryValidation,
    /// The boundary is unsafe or internally incomplete.
    Blocked,
}

/// Non-secret local communications delivery-provider boundary report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommunicationDeliveryProviderBoundaryReport {
    /// Communications/CLI boundary version.
    pub communications_version: String,
    /// Stable local review id.
    pub review_id: String,
    /// Review status.
    pub status: CommunicationDeliveryProviderBoundaryStatus,
    /// Whether the local channel-session prerequisite is ready.
    pub channel_session_ready: bool,
    /// Whether the local platform-adapter prerequisite is ready.
    pub platform_adapter_ready: bool,
    /// Whether authenticated real provider delivery evidence is available.
    pub provider_delivery_evidence_available: bool,
    /// Whether provider-side rate-limit reconciliation evidence is available.
    pub provider_rate_limit_evidence_available: bool,
    /// Whether real provider outage/backoff evidence is available.
    pub provider_outage_evidence_available: bool,
    /// Whether production platform identity/authorization evidence is available.
    pub platform_identity_evidence_available: bool,
    /// Whether remaining external evidence references were recorded.
    pub remaining_external_evidence_recorded: bool,
    /// Count of remaining external evidence references.
    pub remaining_external_evidence_count: usize,
    /// Whether outbound network was used. Always false here.
    pub outbound_network_used: bool,
    /// Whether a real message was delivered. Always false here.
    pub message_delivered: bool,
    /// Whether a provider API call was performed. Always false here.
    pub provider_call_performed: bool,
    /// Whether token or secret material was loaded. Always false here.
    pub token_secret_material_loaded: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether signing or broadcast occurred. Always false here.
    pub signing_or_broadcast_performed: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized blocker descriptions.
    pub blockers: Vec<String>,
    /// Sanitized validation codes.
    pub violation_codes: Vec<String>,
}

impl RemoteCommandSecurityReviewRequest {
    /// Validate local remote command security review input.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "remote command security review",
            &self.review_id,
            &mut violations,
        );
        if self.source == OperatorCommandSource::LocalCli {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_REVIEW_SOURCE_LOCAL",
                "remote command security reviews require a remote, scheduled, or dashboard source",
            ));
        }
        if self.reviewed_at_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_REVIEW_TIMESTAMP_ZERO",
                "remote command security review timestamp must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl RemoteCommandSecurityReviewReport {
    /// Validate local remote command security review invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "remote command security review",
            &self.review_id,
            &mut violations,
        );
        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }
        if self.source == OperatorCommandSource::LocalCli {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_REVIEW_SOURCE_LOCAL",
                "remote command security reviews require a remote, scheduled, or dashboard source",
            ));
        }
        if self.reviewed_at_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_REVIEW_TIMESTAMP_ZERO",
                "remote command security review timestamp must be non-zero",
            ));
        }
        if self.remote_commands_enabled
            || self.outbound_network_used
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_REVIEW_SIDE_EFFECT",
                "remote command security reviews must not enable remote commands, use network, execute live actions, sign, broadcast, or approve production readiness",
            ));
        }
        match self.status {
            RemoteCommandSecurityReviewStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.channel_authentication_required
                    || !self.platform_identity_verification_required
                    || !self.platform_identity_authorization_required
                    || !self.replay_protection_required
                    || !self.command_allowlist_required
                    || !self.unsafe_commands_blocked
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_REMOTE_REVIEW_READY_MISMATCH",
                        "ready remote command reviews require authentication, identity verification, identity authorization, replay protection, command allowlisting, unsafe-command blocking, and zero missing controls",
                    ));
                }
            }
            RemoteCommandSecurityReviewStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_REMOTE_REVIEW_BLOCKED_MISMATCH",
                        "blocked remote command reviews require at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl RemoteCommandEnvelopeValidationRequest {
    /// Validate local remote command envelope input.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "remote command envelope",
            &self.envelope_id,
            &mut violations,
        );
        validate_id(
            "remote command platform identity",
            &self.platform_identity,
            &mut violations,
        );
        validate_id(
            "remote command authorization policy",
            &self.authorization_policy,
            &mut violations,
        );
        validate_id(
            "remote command authentication reference",
            &self.authentication_reference,
            &mut violations,
        );
        validate_id(
            "remote command replay nonce",
            &self.replay_nonce,
            &mut violations,
        );
        if let Err(CommunicationError::ValidationFailed {
            violations: command_violations,
        }) = self.command.validate()
        {
            violations.extend(command_violations);
        }
        if let Err(CommunicationError::ValidationFailed {
            violations: review_violations,
        }) = self.security_review.validate()
        {
            violations.extend(review_violations);
        }
        if self.command.source == OperatorCommandSource::LocalCli {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_ENVELOPE_SOURCE_LOCAL",
                "remote command envelopes require a remote, scheduled, or dashboard source",
            ));
        }
        if self.command.source != self.security_review.source {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_ENVELOPE_SOURCE_MISMATCH",
                "remote command envelope source must match the security review source",
            ));
        }
        if self.received_at_unix_ms == 0 || self.now_unix_ms == 0 || self.max_age_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_ENVELOPE_TIME_INVALID",
                "remote command envelope requires non-zero received, now, and max-age timestamps",
            ));
        }
        if self.now_unix_ms < self.received_at_unix_ms {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_ENVELOPE_TIME_IN_FUTURE",
                "remote command envelope received timestamp cannot be in the future",
            ));
        }
        for value in [
            &self.platform_identity,
            &self.authorization_policy,
            &self.authentication_reference,
            &self.replay_nonce,
        ] {
            if contains_secret_like_text(value) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_REMOTE_ENVELOPE_SECRET_LIKE",
                    "remote command envelope metadata looks like it may contain secret material",
                ));
            }
        }
        finish_validation(violations)
    }
}

impl RemoteCommandEnvelopeValidationReport {
    /// Validate local remote command envelope report invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "remote command envelope",
            &self.envelope_id,
            &mut violations,
        );
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
        if self.source == OperatorCommandSource::LocalCli {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_ENVELOPE_SOURCE_LOCAL",
                "remote command envelopes require a remote, scheduled, or dashboard source",
            ));
        }
        if self.remote_commands_enabled
            || self.outbound_network_used
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_REMOTE_ENVELOPE_SIDE_EFFECT",
                "remote command envelope validation must not enable remote commands, use network, execute live actions, sign, broadcast, or approve production readiness",
            ));
        }
        match self.status {
            RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.security_review_ready
                    || !self.channel_authenticated
                    || !self.platform_identity_verified
                    || !self.platform_identity_authorized
                    || !self.replay_protection_checked
                    || self.replay_nonce_reused
                    || !self.command_allowlisted
                    || self.command_injection_detected
                    || self.stale_envelope
                    || self.command_kind.is_unsafe_execution_request()
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_REMOTE_ENVELOPE_READY_MISMATCH",
                        "ready remote command envelope validations require reviewed security controls, auth, identity, replay protection, allowlisted safe command text, freshness, and zero missing controls",
                    ));
                }
            }
            RemoteCommandEnvelopeValidationStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_REMOTE_ENVELOPE_BLOCKED_MISMATCH",
                        "blocked remote command envelope validations require at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl PlatformCommandIngressRequest {
    /// Validate local mocked platform command ingress input.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "platform command ingress",
            &self.ingress_id,
            &mut violations,
        );
        validate_id("platform label", &self.platform, &mut violations);
        validate_id(
            "platform message",
            &self.platform_message_id,
            &mut violations,
        );
        validate_id(
            "platform identity",
            &self.platform_identity,
            &mut violations,
        );
        validate_id("platform replay nonce", &self.replay_nonce, &mut violations);
        validate_id("platform channel", &self.channel.id, &mut violations);
        if self.command_text.trim().is_empty() {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_COMMAND_TEXT_EMPTY",
                "platform command text must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.command_text) {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_COMMAND_SECRET_LIKE",
                "platform command text must not look like secret material",
            ));
        }
        if self.received_at_unix_ms == 0 || self.now_unix_ms == 0 || self.max_age_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_COMMAND_TIMESTAMP_INVALID",
                "platform command ingress timestamps and max age must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl PlatformCommandIngressReport {
    /// Validate local mocked platform command ingress invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "platform command ingress",
            &self.ingress_id,
            &mut violations,
        );
        validate_id("platform label", &self.platform, &mut violations);
        validate_id(
            "platform message",
            &self.platform_message_id,
            &mut violations,
        );
        validate_id(
            "platform identity",
            &self.platform_identity,
            &mut violations,
        );
        validate_id("platform channel", &self.channel_id, &mut violations);
        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }
        if self.remote_commands_enabled
            || self.outbound_network_used
            || self.message_delivered
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
            || self.token_secret_material_present
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_COMMAND_SIDE_EFFECT",
                "platform command ingress must not store token material, enable remote commands, use network, deliver messages, execute live actions, sign, broadcast, or approve production readiness",
            ));
        }
        match self.status {
            PlatformCommandIngressStatus::ReadyForEnvelopeValidation => {
                if self.missing_control_count != 0
                    || !self.token_reference_present
                    || !self.platform_signature_verified
                    || !self.platform_identity_authorized
                    || !self.channel_permission_granted
                    || self.replay_nonce_reused
                    || self.command_injection_detected
                    || self.stale_message
                    || self.provider_rate_limited
                    || self.provider_outage_observed
                    || self.command.kind.is_unsafe_execution_request()
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_PLATFORM_COMMAND_READY_MISMATCH",
                        "ready platform command ingress requires token-reference metadata, platform authentication and authorization, channel permission, fresh safe command text, no replay, no provider block, and zero missing controls",
                    ));
                }
            }
            PlatformCommandIngressStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_PLATFORM_COMMAND_BLOCKED_MISMATCH",
                        "blocked platform command ingress requires at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl ChannelAdapterValidationRequest {
    /// Validate local authenticated channel adapter validation input.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "channel adapter validation",
            &self.validation_id,
            &mut violations,
        );
        validate_id(
            "adapter authentication reference",
            &self.adapter_authentication_reference,
            &mut violations,
        );
        validate_id(
            "adapter platform identity",
            &self.platform_identity,
            &mut violations,
        );
        validate_id("adapter replay nonce", &self.replay_nonce, &mut violations);
        if let Err(CommunicationError::ValidationFailed {
            violations: envelope_violations,
        }) = self.envelope.validate()
        {
            violations.extend(envelope_violations);
        }
        if let Err(CommunicationError::ValidationFailed {
            violations: dispatch_violations,
        }) = self.dispatch.validate()
        {
            violations.extend(dispatch_violations);
        }
        if self.channel.id.trim().is_empty() {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_CHANNEL_ADAPTER_CHANNEL_EMPTY",
                "channel adapter validation requires a channel id",
            ));
        }
        for value in [
            &self.validation_id,
            &self.channel.id,
            &self.adapter_authentication_reference,
            &self.platform_identity,
            &self.replay_nonce,
        ] {
            if contains_secret_like_text(value) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_CHANNEL_ADAPTER_SECRET_LIKE",
                    "channel adapter validation metadata looks like it may contain secret material",
                ));
            }
        }
        if self.validated_at_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_CHANNEL_ADAPTER_TIMESTAMP_ZERO",
                "channel adapter validation timestamp must be non-zero",
            ));
        }
        for (enabled, code, message) in [
            (
                self.require_delivery_kill_switch,
                "COMMUNICATION_CHANNEL_ADAPTER_DELIVERY_KILL_SWITCH_REQUIRED",
                "future channel delivery requires a kill switch",
            ),
            (
                self.require_audit_state_preflight,
                "COMMUNICATION_CHANNEL_ADAPTER_AUDIT_STATE_PREFLIGHT_REQUIRED",
                "future channel delivery requires audit/state preflight",
            ),
            (
                self.require_delivery_idempotency,
                "COMMUNICATION_CHANNEL_ADAPTER_IDEMPOTENCY_REQUIRED",
                "future channel delivery requires idempotency controls",
            ),
            (
                self.require_rate_limit_controls,
                "COMMUNICATION_CHANNEL_ADAPTER_RATE_LIMIT_CONTROLS_REQUIRED",
                "future channel delivery requires rate-limit controls",
            ),
            (
                self.require_outage_backoff_controls,
                "COMMUNICATION_CHANNEL_ADAPTER_OUTAGE_BACKOFF_REQUIRED",
                "future channel delivery requires outage/backoff controls",
            ),
            (
                self.require_payload_redaction,
                "COMMUNICATION_CHANNEL_ADAPTER_PAYLOAD_REDACTION_REQUIRED",
                "future channel delivery requires payload redaction controls",
            ),
        ] {
            if !enabled {
                violations.push(CommunicationViolation::new(code, message));
            }
        }
        finish_validation(violations)
    }
}

impl ChannelAdapterValidationReport {
    /// Validate local authenticated channel adapter report invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "channel adapter validation",
            &self.validation_id,
            &mut violations,
        );
        validate_id("notification channel", &self.channel_id, &mut violations);
        validate_id(
            "remote command envelope",
            &self.envelope_id,
            &mut violations,
        );
        validate_id("notification dispatch", &self.dispatch_id, &mut violations);
        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }
        if self.remote_commands_enabled
            || self.outbound_network_used
            || self.message_delivered
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_CHANNEL_ADAPTER_SIDE_EFFECT",
                "channel adapter validation must not enable remote commands, use network, deliver messages, execute live actions, sign, broadcast, or approve production readiness",
            ));
        }
        match self.status {
            ChannelAdapterValidationStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.envelope_ready
                    || !self.dispatch_recorded_locally
                    || !self.channel_authenticated
                    || !self.platform_identity_authorized
                    || !self.replay_protection_checked
                    || !self.require_delivery_kill_switch
                    || !self.require_audit_state_preflight
                    || !self.require_delivery_idempotency
                    || !self.require_rate_limit_controls
                    || !self.require_outage_backoff_controls
                    || !self.require_payload_redaction
                    || self.replay_nonce_reused
                    || self.provider_rate_limited
                    || self.provider_outage_observed
                    || self.outbound_delivery_requested
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_CHANNEL_ADAPTER_READY_MISMATCH",
                        "ready channel adapter validations require ready envelope, local dispatch, auth, authorization, replay protection, no rate limit, no outage, no outbound delivery request, and zero missing controls",
                    ));
                }
            }
            ChannelAdapterValidationStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_CHANNEL_ADAPTER_BLOCKED_MISMATCH",
                        "blocked channel adapter validations require at least one missing or unsafe control",
                    ));
                }
            }
        }
        if self.validated_at_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_CHANNEL_ADAPTER_TIMESTAMP_ZERO",
                "channel adapter validation timestamp must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl ChannelSessionValidationReport {
    /// Validate local channel session summary invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "channel session validation",
            &self.session_id,
            &mut violations,
        );
        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }
        if self.validation_ids.is_empty() {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_CHANNEL_SESSION_VALIDATIONS_REQUIRED",
                "channel session validation requires summarized adapter validation ids",
            ));
        }
        let mut unique_ids = BTreeSet::new();
        for validation_id in &self.validation_ids {
            validate_id(
                "channel session adapter validation id",
                validation_id,
                &mut violations,
            );
            if !unique_ids.insert(validation_id) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_CHANNEL_SESSION_DUPLICATE_VALIDATION_ID",
                    "channel session validation ids must be unique",
                ));
            }
        }
        if self.outbound_network_used
            || self.message_delivered
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_CHANNEL_SESSION_SIDE_EFFECT",
                "channel session validation must not use network, deliver messages, execute live actions, sign, broadcast, or approve production readiness",
            ));
        }
        match self.status {
            ChannelSessionValidationStatus::ReadyForLocalReview => {
                if self.total_validation_count < 4
                    || self.accepted_validation_count == 0
                    || self.rejected_unauthenticated_count == 0
                    || self.rejected_replay_count == 0
                    || self.rejected_provider_unavailable_count == 0
                    || !self.envelope_ready
                    || !self.dispatch_recorded_locally
                    || self.outbound_delivery_requested
                    || self.validation_ids.len()
                        != usize::try_from(self.total_validation_count).unwrap_or(usize::MAX)
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_CHANNEL_SESSION_READY_MISMATCH",
                        "ready channel session validation requires accepted local adapter coverage plus unauthenticated, replay, and provider-unavailable rejections",
                    ));
                }
            }
            ChannelSessionValidationStatus::BlockedMissingControls => {
                if self.accepted_validation_count > 0
                    && self.rejected_unauthenticated_count > 0
                    && self.rejected_replay_count > 0
                    && self.rejected_provider_unavailable_count > 0
                    && self.envelope_ready
                    && self.dispatch_recorded_locally
                    && !self.outbound_delivery_requested
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_CHANNEL_SESSION_BLOCKED_MISMATCH",
                        "blocked channel session validation must be missing at least one required control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl PlatformAdapterReviewRequest {
    /// Validate local platform adapter review input.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("platform adapter review", &self.review_id, &mut violations);
        validate_id(
            "platform adapter channel",
            &self.channel.id,
            &mut violations,
        );
        if self.reviewed_at_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_ADAPTER_REVIEW_TIMESTAMP_ZERO",
                "platform adapter review timestamp must be non-zero",
            ));
        }
        if contains_secret_like_text(&self.channel.id) {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_ADAPTER_CHANNEL_SECRET_LIKE",
                "platform adapter channel id must not look like secret material",
            ));
        }
        for (enabled, code, message) in [
            (
                self.require_delivery_kill_switch,
                "COMMUNICATION_PLATFORM_ADAPTER_DELIVERY_KILL_SWITCH_REQUIRED",
                "future platform delivery requires a kill switch",
            ),
            (
                self.require_audit_state_preflight,
                "COMMUNICATION_PLATFORM_ADAPTER_AUDIT_STATE_PREFLIGHT_REQUIRED",
                "future platform delivery requires audit/state preflight",
            ),
            (
                self.require_delivery_idempotency,
                "COMMUNICATION_PLATFORM_ADAPTER_IDEMPOTENCY_REQUIRED",
                "future platform delivery requires idempotency controls",
            ),
            (
                self.require_rate_limit_controls,
                "COMMUNICATION_PLATFORM_ADAPTER_RATE_LIMIT_CONTROLS_REQUIRED",
                "future platform delivery requires rate-limit controls",
            ),
            (
                self.require_outage_backoff_controls,
                "COMMUNICATION_PLATFORM_ADAPTER_OUTAGE_BACKOFF_REQUIRED",
                "future platform delivery requires outage/backoff controls",
            ),
            (
                self.require_payload_redaction,
                "COMMUNICATION_PLATFORM_ADAPTER_PAYLOAD_REDACTION_REQUIRED",
                "future platform delivery requires payload redaction controls",
            ),
        ] {
            if !enabled {
                violations.push(CommunicationViolation::new(code, message));
            }
        }
        finish_validation(violations)
    }
}

impl PlatformAdapterReviewReport {
    /// Validate local platform adapter review invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id("platform adapter review", &self.review_id, &mut violations);
        validate_id(
            "platform adapter channel",
            &self.channel_id,
            &mut violations,
        );
        validate_id(
            "remote command envelope",
            &self.envelope_id,
            &mut violations,
        );
        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }
        if self.remote_commands_enabled
            || self.outbound_network_used
            || self.message_delivered
            || self.live_execution_performed
            || self.signing_or_broadcast_performed
            || self.production_ready
            || self.token_secret_material_present
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_ADAPTER_SIDE_EFFECT",
                "platform adapter review must not store token material, enable remote commands, use network, deliver messages, execute live actions, sign, broadcast, or approve production readiness",
            ));
        }
        match self.status {
            PlatformAdapterReviewStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.envelope_ready
                    || !self.token_reference_present
                    || !self.platform_identity_verified
                    || !self.platform_identity_authorized
                    || !self.channel_permission_granted
                    || !self.command_injection_blocked
                    || !self.require_delivery_kill_switch
                    || !self.require_audit_state_preflight
                    || !self.require_delivery_idempotency
                    || !self.require_rate_limit_controls
                    || !self.require_outage_backoff_controls
                    || !self.require_payload_redaction
                    || self.token_revoked
                    || self.provider_rate_limited
                    || self.provider_outage_observed
                    || self.outbound_delivery_requested
                {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_PLATFORM_ADAPTER_READY_MISMATCH",
                        "ready platform adapter reviews require ready envelope, token reference metadata, identity verification and authorization, channel permission, injection blocking, no token revocation, no provider outage/rate-limit, no outbound delivery request, and zero missing controls",
                    ));
                }
            }
            PlatformAdapterReviewStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(CommunicationViolation::new(
                        "COMMUNICATION_PLATFORM_ADAPTER_BLOCKED_MISMATCH",
                        "blocked platform adapter reviews require at least one missing or unsafe control",
                    ));
                }
            }
        }
        if self.reviewed_at_unix_ms == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_PLATFORM_ADAPTER_REVIEW_TIMESTAMP_ZERO",
                "platform adapter review timestamp must be non-zero",
            ));
        }
        finish_validation(violations)
    }
}

impl CommunicationDeliveryProviderBoundaryRequest {
    /// Validate local communications delivery-provider boundary input.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        self.channel_session.validate()?;
        self.platform_adapter.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "communications delivery provider boundary",
            &self.review_id,
            &mut violations,
        );
        if self.min_remaining_external_evidence == 0 {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DELIVERY_PROVIDER_EXTERNAL_EVIDENCE_FLOOR_ZERO",
                "communications delivery provider boundary requires remaining external evidence",
            ));
        }
        if self
            .remaining_external_evidence
            .iter()
            .any(|item| item.trim().is_empty())
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DELIVERY_PROVIDER_EXTERNAL_EVIDENCE_BLANK",
                "communications delivery provider boundary evidence references must be non-empty",
            ));
        }
        finish_validation(violations)
    }
}

impl CommunicationDeliveryProviderBoundaryReport {
    /// Validate local communications delivery-provider boundary invariants.
    pub fn validate(&self) -> Result<(), CommunicationError> {
        let mut violations = Vec::new();
        validate_id(
            "communications delivery provider boundary",
            &self.review_id,
            &mut violations,
        );
        if self.communications_version != COMMUNICATIONS_CLI_VERSION {
            violations.push(CommunicationViolation::new_owned(
                "COMMUNICATION_VERSION_MISMATCH",
                format!(
                    "communications_version must be {COMMUNICATIONS_CLI_VERSION}, got {}",
                    self.communications_version
                ),
            ));
        }
        let no_side_effects = !self.outbound_network_used
            && !self.message_delivered
            && !self.provider_call_performed
            && !self.token_secret_material_loaded
            && !self.live_execution_performed
            && !self.signing_or_broadcast_performed
            && !self.production_ready;
        let provider_evidence_missing = !self.provider_delivery_evidence_available
            && !self.provider_rate_limit_evidence_available
            && !self.provider_outage_evidence_available
            && !self.platform_identity_evidence_available;
        let should_be_pending = self.channel_session_ready
            && self.platform_adapter_ready
            && provider_evidence_missing
            && self.remaining_external_evidence_recorded
            && no_side_effects;
        if should_be_pending
            && self.status
                != CommunicationDeliveryProviderBoundaryStatus::BlockedPendingProviderDeliveryValidation
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DELIVERY_PROVIDER_STATUS_SHOULD_BE_PENDING",
                "ready local communications prerequisites with missing provider evidence must remain blocked pending provider delivery validation",
            ));
        }
        if !should_be_pending && self.status != CommunicationDeliveryProviderBoundaryStatus::Blocked
        {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DELIVERY_PROVIDER_STATUS_SHOULD_BLOCK",
                "unsafe or incomplete communications delivery-provider evidence must be blocked",
            ));
        }
        if self.blockers.is_empty() {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_DELIVERY_PROVIDER_BLOCKERS_EMPTY",
                "communications delivery-provider boundary must record unresolved blockers",
            ));
        }
        finish_validation(violations)
    }
}

/// Review future remote operator command controls without enabling remote commands.
pub fn review_remote_command_security(
    request: &RemoteCommandSecurityReviewRequest,
) -> Result<RemoteCommandSecurityReviewReport, CommunicationError> {
    request.validate()?;
    let mut missing_control_count = 0_u64;
    for missing in [
        !request.channel_authentication_required,
        !request.platform_identity_verification_required,
        !request.platform_identity_authorization_required,
        !request.replay_protection_required,
        !request.command_allowlist_required,
        !request.unsafe_commands_blocked,
        request.remote_command_enablement_requested,
        request.outbound_network_requested,
        request.live_execution_requested,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let status = if missing_control_count == 0 {
        RemoteCommandSecurityReviewStatus::ReadyForLocalReview
    } else {
        RemoteCommandSecurityReviewStatus::BlockedMissingControls
    };
    let report = RemoteCommandSecurityReviewReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        review_id: request.review_id.clone(),
        source: request.source,
        status,
        channel_authentication_required: request.channel_authentication_required,
        platform_identity_verification_required: request.platform_identity_verification_required,
        platform_identity_authorization_required: request.platform_identity_authorization_required,
        replay_protection_required: request.replay_protection_required,
        command_allowlist_required: request.command_allowlist_required,
        unsafe_commands_blocked: request.unsafe_commands_blocked,
        missing_control_count,
        remote_commands_enabled: false,
        outbound_network_used: false,
        live_execution_performed: false,
        signing_or_broadcast_performed: false,
        production_ready: false,
        reviewed_at_unix_ms: request.reviewed_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate one local remote command envelope without enabling remote routing.
pub fn review_platform_command_ingress(
    request: &PlatformCommandIngressRequest,
) -> Result<PlatformCommandIngressReport, CommunicationError> {
    request.validate()?;
    let sanitized_raw = sanitize_command_text(&request.command_text);
    let args = request
        .command_text
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let kind = command_kind_for(&args);
    let command = OperatorCommand {
        id: format!("{}-command", request.ingress_id),
        source: OperatorCommandSource::MessagingChannel,
        kind,
        args: command_args_for(&args, kind),
        sanitized_raw,
        received_at_unix_ms: request.received_at_unix_ms,
    };
    command.validate()?;
    let command_injection_detected = contains_command_injection_text(&command.sanitized_raw)
        || command
            .args
            .iter()
            .any(|arg| contains_command_injection_text(arg));
    let stale_message = request
        .now_unix_ms
        .saturating_sub(request.received_at_unix_ms)
        > request.max_age_ms;
    let mut missing_control_count = 0_u64;
    for missing in [
        !request.channel.enabled,
        !request.token_reference_present,
        request.token_secret_material_present,
        !request.platform_signature_verified,
        !request.platform_identity_authorized,
        !request.channel_permission_granted,
        request.replay_nonce_reused,
        command_injection_detected,
        stale_message,
        request.provider_rate_limited,
        request.provider_outage_observed,
        command.kind.is_unsafe_execution_request(),
        request.outbound_network_used,
        request.live_execution_performed,
        request.signing_or_broadcast_performed,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let report = PlatformCommandIngressReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        ingress_id: request.ingress_id.clone(),
        platform: request.platform.clone(),
        channel_id: request.channel.id.clone(),
        channel_kind: request.channel.kind,
        platform_message_id: request.platform_message_id.clone(),
        platform_identity: request.platform_identity.clone(),
        command,
        status: if missing_control_count == 0 {
            PlatformCommandIngressStatus::ReadyForEnvelopeValidation
        } else {
            PlatformCommandIngressStatus::BlockedMissingControls
        },
        token_reference_present: request.token_reference_present,
        token_secret_material_present: request.token_secret_material_present,
        platform_signature_verified: request.platform_signature_verified,
        platform_identity_authorized: request.platform_identity_authorized,
        channel_permission_granted: request.channel_permission_granted,
        replay_nonce_reused: request.replay_nonce_reused,
        command_injection_detected,
        stale_message,
        provider_rate_limited: request.provider_rate_limited,
        provider_outage_observed: request.provider_outage_observed,
        missing_control_count,
        remote_commands_enabled: false,
        outbound_network_used: request.outbound_network_used,
        message_delivered: false,
        live_execution_performed: request.live_execution_performed,
        signing_or_broadcast_performed: request.signing_or_broadcast_performed,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Validate one local remote command envelope without enabling remote routing.
pub fn validate_remote_command_envelope(
    request: &RemoteCommandEnvelopeValidationRequest,
) -> Result<RemoteCommandEnvelopeValidationReport, CommunicationError> {
    request.validate()?;
    let security_review_ready =
        request.security_review.status == RemoteCommandSecurityReviewStatus::ReadyForLocalReview;
    let stale_envelope = request
        .now_unix_ms
        .saturating_sub(request.received_at_unix_ms)
        > request.max_age_ms;
    let unsafe_command = request.command.kind.is_unsafe_execution_request();
    let command_injection_detected =
        contains_command_injection_text(&request.command.sanitized_raw)
            || request
                .command
                .args
                .iter()
                .any(|arg| contains_command_injection_text(arg));
    let mut missing_control_count = 0_u64;
    for missing in [
        !security_review_ready,
        !request.channel_authenticated,
        !request.platform_identity_verified,
        !request.platform_identity_authorized,
        !request.replay_protection_checked,
        request.replay_nonce_reused,
        !request.command_allowlisted,
        command_injection_detected,
        stale_envelope,
        unsafe_command,
        request.remote_command_enablement_requested,
        request.outbound_network_used,
        request.live_execution_performed,
        request.signing_or_broadcast_performed,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let report = RemoteCommandEnvelopeValidationReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        envelope_id: request.envelope_id.clone(),
        command_id: request.command.id.clone(),
        source: request.command.source,
        command_kind: request.command.kind,
        status: if missing_control_count == 0 {
            RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview
        } else {
            RemoteCommandEnvelopeValidationStatus::BlockedMissingControls
        },
        security_review_ready,
        channel_authenticated: request.channel_authenticated,
        platform_identity_verified: request.platform_identity_verified,
        platform_identity_authorized: request.platform_identity_authorized,
        replay_protection_checked: request.replay_protection_checked,
        replay_nonce_reused: request.replay_nonce_reused,
        command_allowlisted: request.command_allowlisted,
        command_injection_detected,
        stale_envelope,
        missing_control_count,
        remote_commands_enabled: false,
        outbound_network_used: request.outbound_network_used,
        live_execution_performed: request.live_execution_performed,
        signing_or_broadcast_performed: request.signing_or_broadcast_performed,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a local authenticated channel adapter seam without sending messages.
pub fn validate_channel_adapter(
    request: &ChannelAdapterValidationRequest,
) -> Result<ChannelAdapterValidationReport, CommunicationError> {
    request.validate()?;
    let envelope_ready =
        request.envelope.status == RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview;
    let dispatch_recorded_locally = request.dispatch.status
        == NotificationDispatchStatus::RecordedLocally
        && !request.dispatch.outbound_network_used;
    let mut missing_control_count = 0_u64;
    for missing in [
        !envelope_ready,
        !dispatch_recorded_locally,
        !request.channel.enabled,
        !request.channel_authenticated,
        !request.platform_identity_authorized,
        !request.replay_protection_checked,
        !request.require_delivery_kill_switch,
        !request.require_audit_state_preflight,
        !request.require_delivery_idempotency,
        !request.require_rate_limit_controls,
        !request.require_outage_backoff_controls,
        !request.require_payload_redaction,
        request.replay_nonce_reused,
        request.provider_rate_limited,
        request.provider_outage_observed,
        request.outbound_delivery_requested,
        request.outbound_network_used,
        request.message_delivered,
        request.live_execution_performed,
        request.signing_or_broadcast_performed,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let report = ChannelAdapterValidationReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        validation_id: request.validation_id.clone(),
        channel_id: request.channel.id.clone(),
        channel_kind: request.channel.kind,
        envelope_id: request.envelope.envelope_id.clone(),
        dispatch_id: request.dispatch.id.clone(),
        status: if missing_control_count == 0 {
            ChannelAdapterValidationStatus::ReadyForLocalReview
        } else {
            ChannelAdapterValidationStatus::BlockedMissingControls
        },
        envelope_ready,
        dispatch_recorded_locally,
        channel_authenticated: request.channel_authenticated,
        platform_identity_authorized: request.platform_identity_authorized,
        replay_protection_checked: request.replay_protection_checked,
        require_delivery_kill_switch: request.require_delivery_kill_switch,
        require_audit_state_preflight: request.require_audit_state_preflight,
        require_delivery_idempotency: request.require_delivery_idempotency,
        require_rate_limit_controls: request.require_rate_limit_controls,
        require_outage_backoff_controls: request.require_outage_backoff_controls,
        require_payload_redaction: request.require_payload_redaction,
        replay_nonce_reused: request.replay_nonce_reused,
        provider_rate_limited: request.provider_rate_limited,
        provider_outage_observed: request.provider_outage_observed,
        missing_control_count,
        outbound_delivery_requested: request.outbound_delivery_requested,
        outbound_network_used: request.outbound_network_used,
        message_delivered: request.message_delivered,
        remote_commands_enabled: false,
        live_execution_performed: request.live_execution_performed,
        signing_or_broadcast_performed: request.signing_or_broadcast_performed,
        production_ready: false,
        validated_at_unix_ms: request.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Summarize multiple local channel adapter validations without sending messages.
pub fn validate_channel_session(
    session_id: impl Into<String>,
    reports: &[ChannelAdapterValidationReport],
) -> Result<ChannelSessionValidationReport, CommunicationError> {
    let session_id = session_id.into();
    let mut validation_ids = Vec::with_capacity(reports.len());
    let mut accepted_validation_count = 0_u32;
    let mut rejected_unauthenticated_count = 0_u32;
    let mut rejected_replay_count = 0_u32;
    let mut rejected_provider_unavailable_count = 0_u32;
    let mut envelope_ready = !reports.is_empty();
    let mut dispatch_recorded_locally = !reports.is_empty();
    let mut missing_control_count = 0_u64;
    let mut outbound_delivery_requested = false;
    let mut outbound_network_used = false;
    let mut message_delivered = false;
    let mut live_execution_performed = false;
    let mut signing_or_broadcast_performed = false;
    let mut production_ready = false;

    for report in reports {
        report.validate()?;
        validation_ids.push(report.validation_id.clone());
        missing_control_count = missing_control_count.saturating_add(report.missing_control_count);
        envelope_ready &= report.envelope_ready;
        dispatch_recorded_locally &= report.dispatch_recorded_locally;
        outbound_delivery_requested |= report.outbound_delivery_requested;
        outbound_network_used |= report.outbound_network_used;
        message_delivered |= report.message_delivered;
        live_execution_performed |= report.live_execution_performed;
        signing_or_broadcast_performed |= report.signing_or_broadcast_performed;
        production_ready |= report.production_ready;
        if report.status == ChannelAdapterValidationStatus::ReadyForLocalReview {
            accepted_validation_count = accepted_validation_count.saturating_add(1);
        }
        if report.status == ChannelAdapterValidationStatus::BlockedMissingControls
            && !report.channel_authenticated
        {
            rejected_unauthenticated_count = rejected_unauthenticated_count.saturating_add(1);
        }
        if report.status == ChannelAdapterValidationStatus::BlockedMissingControls
            && report.replay_nonce_reused
        {
            rejected_replay_count = rejected_replay_count.saturating_add(1);
        }
        if report.status == ChannelAdapterValidationStatus::BlockedMissingControls
            && (report.provider_rate_limited || report.provider_outage_observed)
        {
            rejected_provider_unavailable_count =
                rejected_provider_unavailable_count.saturating_add(1);
        }
    }

    let status = if accepted_validation_count > 0
        && rejected_unauthenticated_count > 0
        && rejected_replay_count > 0
        && rejected_provider_unavailable_count > 0
        && envelope_ready
        && dispatch_recorded_locally
        && !outbound_delivery_requested
        && !outbound_network_used
        && !message_delivered
        && !live_execution_performed
        && !signing_or_broadcast_performed
        && !production_ready
    {
        ChannelSessionValidationStatus::ReadyForLocalReview
    } else {
        ChannelSessionValidationStatus::BlockedMissingControls
    };
    let report = ChannelSessionValidationReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        session_id,
        status,
        validation_ids,
        total_validation_count: u32::try_from(reports.len()).map_err(|_| {
            CommunicationError::StateStoreFailed {
                reason: "channel session validation count overflowed".to_owned(),
            }
        })?,
        accepted_validation_count,
        rejected_unauthenticated_count,
        rejected_replay_count,
        rejected_provider_unavailable_count,
        envelope_ready,
        dispatch_recorded_locally,
        missing_control_count,
        outbound_delivery_requested,
        outbound_network_used,
        message_delivered,
        live_execution_performed,
        signing_or_broadcast_performed,
        production_ready,
    };
    report.validate()?;
    Ok(report)
}

/// Review local platform adapter controls without storing tokens or delivering messages.
pub fn review_platform_adapter_controls(
    request: &PlatformAdapterReviewRequest,
) -> Result<PlatformAdapterReviewReport, CommunicationError> {
    request.validate()?;
    let envelope_ready =
        request.envelope.status == RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview;
    let mut missing_control_count = 0_u64;
    for missing in [
        !envelope_ready,
        !request.channel.enabled,
        !request.token_reference_present,
        request.token_secret_material_present,
        !request.platform_identity_verified,
        !request.platform_identity_authorized,
        !request.channel_permission_granted,
        !request.command_injection_blocked,
        !request.require_delivery_kill_switch,
        !request.require_audit_state_preflight,
        !request.require_delivery_idempotency,
        !request.require_rate_limit_controls,
        !request.require_outage_backoff_controls,
        !request.require_payload_redaction,
        request.token_revoked,
        request.provider_rate_limited,
        request.provider_outage_observed,
        request.outbound_delivery_requested,
        request.outbound_network_used,
        request.message_delivered,
        request.live_execution_performed,
        request.signing_or_broadcast_performed,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let report = PlatformAdapterReviewReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        review_id: request.review_id.clone(),
        channel_id: request.channel.id.clone(),
        channel_kind: request.channel.kind,
        envelope_id: request.envelope.envelope_id.clone(),
        status: if missing_control_count == 0 {
            PlatformAdapterReviewStatus::ReadyForLocalReview
        } else {
            PlatformAdapterReviewStatus::BlockedMissingControls
        },
        envelope_ready,
        token_reference_present: request.token_reference_present,
        token_secret_material_present: request.token_secret_material_present,
        platform_identity_verified: request.platform_identity_verified,
        platform_identity_authorized: request.platform_identity_authorized,
        channel_permission_granted: request.channel_permission_granted,
        command_injection_blocked: request.command_injection_blocked,
        require_delivery_kill_switch: request.require_delivery_kill_switch,
        require_audit_state_preflight: request.require_audit_state_preflight,
        require_delivery_idempotency: request.require_delivery_idempotency,
        require_rate_limit_controls: request.require_rate_limit_controls,
        require_outage_backoff_controls: request.require_outage_backoff_controls,
        require_payload_redaction: request.require_payload_redaction,
        token_revoked: request.token_revoked,
        provider_rate_limited: request.provider_rate_limited,
        provider_outage_observed: request.provider_outage_observed,
        missing_control_count,
        outbound_delivery_requested: request.outbound_delivery_requested,
        outbound_network_used: request.outbound_network_used,
        message_delivered: request.message_delivered,
        remote_commands_enabled: false,
        live_execution_performed: request.live_execution_performed,
        signing_or_broadcast_performed: request.signing_or_broadcast_performed,
        production_ready: false,
        reviewed_at_unix_ms: request.reviewed_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Review local communications delivery-provider prerequisites without delivery.
pub fn review_communication_delivery_provider_boundary(
    request: &CommunicationDeliveryProviderBoundaryRequest,
) -> Result<CommunicationDeliveryProviderBoundaryReport, CommunicationError> {
    request.validate()?;
    let channel_session_ready = request.channel_session.status
        == ChannelSessionValidationStatus::ReadyForLocalReview
        && request.channel_session.accepted_validation_count > 0
        && request.channel_session.rejected_provider_unavailable_count > 0
        && !request.channel_session.outbound_delivery_requested
        && !request.channel_session.outbound_network_used
        && !request.channel_session.message_delivered
        && !request.channel_session.live_execution_performed
        && !request.channel_session.signing_or_broadcast_performed
        && !request.channel_session.production_ready;
    let platform_adapter_ready = request.platform_adapter.status
        == PlatformAdapterReviewStatus::ReadyForLocalReview
        && request.platform_adapter.envelope_ready
        && request.platform_adapter.token_reference_present
        && !request.platform_adapter.token_secret_material_present
        && request.platform_adapter.platform_identity_verified
        && request.platform_adapter.platform_identity_authorized
        && request.platform_adapter.channel_permission_granted
        && request.platform_adapter.command_injection_blocked
        && request.platform_adapter.require_delivery_kill_switch
        && request.platform_adapter.require_audit_state_preflight
        && request.platform_adapter.require_delivery_idempotency
        && request.platform_adapter.require_rate_limit_controls
        && request.platform_adapter.require_outage_backoff_controls
        && request.platform_adapter.require_payload_redaction
        && !request.platform_adapter.token_revoked
        && !request.platform_adapter.provider_rate_limited
        && !request.platform_adapter.provider_outage_observed
        && !request.platform_adapter.outbound_delivery_requested
        && !request.platform_adapter.outbound_network_used
        && !request.platform_adapter.message_delivered
        && !request.platform_adapter.remote_commands_enabled
        && !request.platform_adapter.live_execution_performed
        && !request.platform_adapter.signing_or_broadcast_performed
        && !request.platform_adapter.production_ready;
    let remaining_external_evidence_recorded =
        request.remaining_external_evidence.len() >= request.min_remaining_external_evidence;
    let outbound_network_used = request.outbound_network_used
        || request.channel_session.outbound_network_used
        || request.platform_adapter.outbound_network_used;
    let message_delivered = request.message_delivered
        || request.channel_session.message_delivered
        || request.platform_adapter.message_delivered;
    let token_secret_material_loaded = request.token_secret_material_loaded
        || request.platform_adapter.token_secret_material_present;
    let live_execution_performed = request.live_execution_performed
        || request.channel_session.live_execution_performed
        || request.platform_adapter.live_execution_performed;
    let signing_or_broadcast_performed = request.signing_or_broadcast_performed
        || request.channel_session.signing_or_broadcast_performed
        || request.platform_adapter.signing_or_broadcast_performed;
    let production_ready_claimed = request.production_ready_claimed
        || request.channel_session.production_ready
        || request.platform_adapter.production_ready;

    let mut blockers = Vec::new();
    if !request.provider_delivery_evidence_available {
        blockers.push("authenticated provider delivery evidence missing".to_owned());
    }
    if !request.provider_rate_limit_evidence_available {
        blockers.push("provider-side rate-limit reconciliation evidence missing".to_owned());
    }
    if !request.provider_outage_evidence_available {
        blockers.push("provider outage/backoff evidence missing".to_owned());
    }
    if !request.platform_identity_evidence_available {
        blockers.push("production platform identity authorization evidence missing".to_owned());
    }
    if !remaining_external_evidence_recorded {
        blockers
            .push("remaining external communications evidence references below floor".to_owned());
    }

    let mut violation_codes = Vec::new();
    push_code(
        &mut violation_codes,
        !channel_session_ready,
        "COMMUNICATION_DELIVERY_PROVIDER_CHANNEL_SESSION_NOT_READY",
    );
    push_code(
        &mut violation_codes,
        !platform_adapter_ready,
        "COMMUNICATION_DELIVERY_PROVIDER_PLATFORM_ADAPTER_NOT_READY",
    );
    push_code(
        &mut violation_codes,
        !remaining_external_evidence_recorded,
        "COMMUNICATION_DELIVERY_PROVIDER_EXTERNAL_EVIDENCE_MISSING",
    );
    push_code(
        &mut violation_codes,
        outbound_network_used,
        "COMMUNICATION_DELIVERY_PROVIDER_OUTBOUND_NETWORK_USED",
    );
    push_code(
        &mut violation_codes,
        message_delivered,
        "COMMUNICATION_DELIVERY_PROVIDER_MESSAGE_DELIVERED",
    );
    push_code(
        &mut violation_codes,
        request.provider_call_performed,
        "COMMUNICATION_DELIVERY_PROVIDER_CALL_PERFORMED",
    );
    push_code(
        &mut violation_codes,
        token_secret_material_loaded,
        "COMMUNICATION_DELIVERY_PROVIDER_TOKEN_SECRET_LOADED",
    );
    push_code(
        &mut violation_codes,
        live_execution_performed,
        "COMMUNICATION_DELIVERY_PROVIDER_LIVE_EXECUTION",
    );
    push_code(
        &mut violation_codes,
        signing_or_broadcast_performed,
        "COMMUNICATION_DELIVERY_PROVIDER_SIGNING_OR_BROADCAST",
    );
    push_code(
        &mut violation_codes,
        production_ready_claimed,
        "COMMUNICATION_DELIVERY_PROVIDER_PRODUCTION_READY_CLAIMED",
    );

    let provider_evidence_missing = !request.provider_delivery_evidence_available
        && !request.provider_rate_limit_evidence_available
        && !request.provider_outage_evidence_available
        && !request.platform_identity_evidence_available;
    let safe_pending_provider_validation = channel_session_ready
        && platform_adapter_ready
        && provider_evidence_missing
        && remaining_external_evidence_recorded
        && !outbound_network_used
        && !message_delivered
        && !request.provider_call_performed
        && !token_secret_material_loaded
        && !live_execution_performed
        && !signing_or_broadcast_performed
        && !production_ready_claimed;

    let report = CommunicationDeliveryProviderBoundaryReport {
        communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
        review_id: request.review_id.clone(),
        status: if safe_pending_provider_validation {
            CommunicationDeliveryProviderBoundaryStatus::BlockedPendingProviderDeliveryValidation
        } else {
            CommunicationDeliveryProviderBoundaryStatus::Blocked
        },
        channel_session_ready,
        platform_adapter_ready,
        provider_delivery_evidence_available: request.provider_delivery_evidence_available,
        provider_rate_limit_evidence_available: request.provider_rate_limit_evidence_available,
        provider_outage_evidence_available: request.provider_outage_evidence_available,
        platform_identity_evidence_available: request.platform_identity_evidence_available,
        remaining_external_evidence_recorded,
        remaining_external_evidence_count: request.remaining_external_evidence.len(),
        outbound_network_used,
        message_delivered,
        provider_call_performed: request.provider_call_performed,
        token_secret_material_loaded,
        live_execution_performed,
        signing_or_broadcast_performed,
        production_ready: false,
        blockers,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
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
    /// Whether the command source passed local operator authorization.
    pub operator_authorized: bool,
    /// Local operator authorization status.
    pub authorization_status: OperatorCommandAuthorizationStatus,
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

        if self.accepted && !self.operator_authorized {
            violations.push(CommunicationViolation::new(
                "COMMUNICATION_ROUTE_ACCEPTED_WITHOUT_OPERATOR_AUTH",
                "accepted routed commands must have local operator authorization",
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

        let authorization = authorize_operator_command(&request.command, &request.config);
        let (action, accepted, reason) = if authorization.operator_authorized {
            route_command(&request.command)
        } else {
            (
                OperatorCommandAction::RejectUnauthorizedCommand,
                false,
                authorization.reason.clone(),
            )
        };
        let route = RoutedOperatorCommand {
            id: format!("command-route:{}", request.command.id),
            request_id: request.id.clone(),
            command_id: request.command.id.clone(),
            communications_version: COMMUNICATIONS_CLI_VERSION.to_owned(),
            action,
            accepted,
            operator_authorized: authorization.operator_authorized,
            authorization_status: authorization.status,
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
    /// Optional caller-supplied local channel safety observations.
    pub channel_safety: Vec<NotificationChannelSafetyState>,
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

        let mut safety_channel_ids = BTreeSet::new();
        for safety in &self.channel_safety {
            validate_id(
                "notification channel safety",
                &safety.channel_id,
                &mut violations,
            );
            if contains_secret_like_text(&safety.channel_id) {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_SAFETY_ID_SECRET_LIKE",
                    format!(
                        "notification channel safety id {} looks like secret material",
                        safety.channel_id
                    ),
                ));
            }
            if !safety_channel_ids.insert(safety.channel_id.to_ascii_lowercase()) {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_SAFETY_DUPLICATE",
                    format!(
                        "notification channel safety {} is duplicated",
                        safety.channel_id
                    ),
                ));
            }
            if safety.max_messages_per_window == 0 {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_RATE_LIMIT_ZERO",
                    format!(
                        "notification channel safety {} must have a positive max_messages_per_window",
                        safety.channel_id
                    ),
                ));
            }
            if safety.window_started_at_unix_ms > safety.window_ends_at_unix_ms {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_RATE_LIMIT_WINDOW_INVALID",
                    format!(
                        "notification channel safety {} has an invalid rate-limit window",
                        safety.channel_id
                    ),
                ));
            }
            if safety.outage_reason.trim().is_empty() && safety.outage_active {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_OUTAGE_REASON_EMPTY",
                    format!(
                        "notification channel safety {} must include a non-secret outage reason",
                        safety.channel_id
                    ),
                ));
            }
            if contains_secret_like_text(&safety.outage_reason) {
                violations.push(CommunicationViolation::new(
                    "COMMUNICATION_CHANNEL_OUTAGE_REASON_SECRET_LIKE",
                    "notification channel outage reason looks like it may contain secret material",
                ));
            }
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

/// Local caller-supplied channel safety observation for deterministic dispatch gating.
///
/// This is a non-secret reference-only record. It does not query messaging
/// platforms, inspect remote outages, call APIs, or deliver notifications.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationChannelSafetyState {
    /// Channel id this local observation applies to.
    pub channel_id: String,
    /// Messages already recorded for this channel in the local window.
    pub messages_sent_in_window: u64,
    /// Maximum local messages allowed for the window.
    pub max_messages_per_window: u64,
    /// Local rate-limit window start time in Unix milliseconds.
    pub window_started_at_unix_ms: u64,
    /// Local rate-limit window end time in Unix milliseconds.
    pub window_ends_at_unix_ms: u64,
    /// Whether the caller has marked the local channel as unavailable.
    pub outage_active: bool,
    /// Short sanitized reason when `outage_active` is true.
    pub outage_reason: String,
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
    /// All selected channels were blocked by local rate limits.
    BlockedRateLimited,
    /// All selected channels were blocked by local outage observations.
    BlockedChannelOutage,
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
    /// Channel was blocked by a caller-supplied local rate-limit observation.
    RateLimited,
    /// Channel was blocked by a caller-supplied local outage observation.
    ChannelOutage,
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
    /// Whether the local channel rate limit blocked this dispatch.
    pub rate_limited: bool,
    /// Whether the caller-supplied local outage observation blocked this dispatch.
    pub outage_blocked: bool,
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
            if channel.rate_limited
                && channel.status != NotificationChannelDispatchStatus::RateLimited
            {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_RATE_LIMIT_STATUS_MISMATCH",
                    format!(
                        "channel {} rate_limited flag must match rate-limited status",
                        channel.channel_id
                    ),
                ));
            }
            if channel.outage_blocked
                && channel.status != NotificationChannelDispatchStatus::ChannelOutage
            {
                violations.push(CommunicationViolation::new_owned(
                    "COMMUNICATION_CHANNEL_OUTAGE_STATUS_MISMATCH",
                    format!(
                        "channel {} outage_blocked flag must match channel-outage status",
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
        let channel_safety = channel_safety_by_id(&request.channel_safety);
        let channels = channel_profiles
            .iter()
            .map(|channel| {
                channel_dispatch_for(
                    channel,
                    channel_safety.get(&channel.id.to_ascii_lowercase()),
                )
            })
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

/// Persist the latest routed operator command through the typed local state boundary.
///
/// This stores sanitized route metadata only. It does not authenticate remote
/// channels, execute commands, call networks, submit orders, or approve live
/// execution.
pub fn persist_routed_operator_command_checkpoint(
    store: &mut impl StateStore,
    route: &RoutedOperatorCommand,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    route.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(route).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!("failed to serialize routed operator command checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest notification dispatch through the typed local state boundary.
///
/// This stores redacted dispatch metadata only. It does not deliver outbound
/// notifications, call platform APIs, or store channel credentials.
pub fn persist_notification_dispatch_checkpoint(
    store: &mut impl StateStore,
    dispatch: &NotificationDispatchRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    dispatch.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(dispatch).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!("failed to serialize notification dispatch checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest remote command security review through the typed local state boundary.
///
/// This stores sanitized review metadata only. It does not enable remote
/// commands, authenticate platforms, call networks, execute commands, sign, or
/// broadcast.
pub fn persist_remote_command_security_review_checkpoint(
    store: &mut impl StateStore,
    report: &RemoteCommandSecurityReviewReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!(
                    "failed to serialize remote command security review checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local remote command envelope validation through typed state.
///
/// This stores sanitized metadata only. It does not enable remote commands,
/// execute commands, call a platform, sign, broadcast, or deliver messages.
pub fn persist_remote_command_envelope_validation_checkpoint(
    store: &mut impl StateStore,
    report: &RemoteCommandEnvelopeValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!(
                    "failed to serialize remote command envelope validation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local platform command ingress review through typed state.
///
/// This stores sanitized mocked-platform metadata only. It does not store
/// platform tokens, call a platform, deliver messages, or enable remote
/// command execution.
pub fn persist_platform_command_ingress_checkpoint(
    store: &mut impl StateStore,
    report: &PlatformCommandIngressReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!("failed to serialize platform command ingress checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local authenticated channel adapter validation through typed state.
///
/// This stores sanitized metadata only. It does not deliver messages, call a
/// platform, store tokens, enable remote commands, sign, broadcast, or execute
/// live actions.
pub fn persist_channel_adapter_validation_checkpoint(
    store: &mut impl StateStore,
    report: &ChannelAdapterValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!(
                    "failed to serialize channel adapter validation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local channel session validation summary through the typed state boundary.
///
/// This stores sanitized local session-control metadata only. It does not
/// deliver messages, authenticate real channel tokens, call networks, or
/// approve production readiness.
pub fn persist_channel_session_validation_checkpoint(
    store: &mut impl StateStore,
    report: &ChannelSessionValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!(
                    "failed to serialize channel session validation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local platform adapter control review through typed state.
///
/// This stores sanitized control metadata only. It does not store platform
/// tokens, call a platform, deliver messages, or approve production readiness.
pub fn persist_platform_adapter_review_checkpoint(
    store: &mut impl StateStore,
    report: &PlatformAdapterReviewReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CommunicationError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: COMMUNICATIONS_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            CommunicationError::StateStoreFailed {
                reason: format!("failed to serialize platform adapter review checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CommunicationError::from)?;
    Ok(checkpoint)
}

/// Append one sanitized routed-operator-command record to the local audit journal.
///
/// The record is replayable audit evidence only. It never enables execution or
/// outbound network behavior.
pub fn append_routed_operator_command_audit(
    journal: &mut AppendOnlyAuditJournal,
    route: &RoutedOperatorCommand,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    route.validate()?;
    let mut event = AuditEvent::new(
        format!("communications-command-route-{}", route.id),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "operator-command-router",
        "operator command route recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(COMMUNICATIONS_CLI_VERSION.to_owned()),
        )
        .with_metadata("route_id", AuditValue::Text(route.id.clone()))
        .with_metadata("request_id", AuditValue::Text(route.request_id.clone()))
        .with_metadata("command_id", AuditValue::Text(route.command_id.clone()))
        .with_metadata("action", AuditValue::Text(format!("{:?}", route.action)))
        .with_metadata("accepted", AuditValue::Bool(route.accepted))
        .with_metadata(
            "operator_authorized",
            AuditValue::Bool(route.operator_authorized),
        )
        .with_metadata(
            "authorization_status",
            AuditValue::Text(format!("{:?}", route.authorization_status)),
        )
        .with_metadata(
            "execution_enabled",
            AuditValue::Bool(route.execution_enabled),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(route.outbound_network_used),
        )
        .with_metadata("reason", AuditValue::Text(route.reason.clone()));
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one redacted notification dispatch record to the local audit journal.
///
/// The record captures dispatch outcomes only. It never sends a message to an
/// external channel.
pub fn append_notification_dispatch_audit(
    journal: &mut AppendOnlyAuditJournal,
    dispatch: &NotificationDispatchRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    dispatch.validate()?;
    let mut event = AuditEvent::new(
        format!("communications-notification-dispatch-{}", dispatch.id),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "notification-boundary",
        "notification dispatch recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(COMMUNICATIONS_CLI_VERSION.to_owned()),
        )
        .with_metadata("dispatch_id", AuditValue::Text(dispatch.id.clone()))
        .with_metadata("request_id", AuditValue::Text(dispatch.request_id.clone()))
        .with_metadata(
            "notification_id",
            AuditValue::Text(dispatch.notification_id.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", dispatch.status)))
        .with_metadata(
            "channel_count",
            AuditValue::Text(dispatch.channels.len().to_string()),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(dispatch.outbound_network_used),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one remote command security review to the local audit journal.
///
/// The record captures future remote-channel control status only. It never
/// enables remote command routing or performs network, signing, broadcast, or
/// live-execution behavior.
pub fn append_remote_command_security_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &RemoteCommandSecurityReviewReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("communications-remote-command-review-{}", report.review_id),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "remote-command-security-review",
        "remote command security review recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(COMMUNICATIONS_CLI_VERSION.to_owned()),
        )
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata("source", AuditValue::Text(format!("{:?}", report.source)))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "channel_authentication_required",
            AuditValue::Bool(report.channel_authentication_required),
        )
        .with_metadata(
            "platform_identity_verification_required",
            AuditValue::Bool(report.platform_identity_verification_required),
        )
        .with_metadata(
            "platform_identity_authorization_required",
            AuditValue::Bool(report.platform_identity_authorization_required),
        )
        .with_metadata(
            "replay_protection_required",
            AuditValue::Bool(report.replay_protection_required),
        )
        .with_metadata(
            "command_allowlist_required",
            AuditValue::Bool(report.command_allowlist_required),
        )
        .with_metadata(
            "unsafe_commands_blocked",
            AuditValue::Bool(report.unsafe_commands_blocked),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "remote_commands_enabled",
            AuditValue::Bool(report.remote_commands_enabled),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one local remote command envelope validation to the audit journal.
///
/// This records sanitized control outcomes only. It never enables remote
/// routing, performs network I/O, executes live actions, signs, or broadcasts.
pub fn append_remote_command_envelope_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &RemoteCommandEnvelopeValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "communications-remote-command-envelope-{}",
            report.envelope_id
        ),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "remote-command-envelope-validation",
        "remote command envelope validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(report.communications_version.clone()),
        )
        .with_metadata("envelope_id", AuditValue::Text(report.envelope_id.clone()))
        .with_metadata("command_id", AuditValue::Text(report.command_id.clone()))
        .with_metadata("source", AuditValue::Text(format!("{:?}", report.source)))
        .with_metadata(
            "command_kind",
            AuditValue::Text(format!("{:?}", report.command_kind)),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "security_review_ready",
            AuditValue::Bool(report.security_review_ready),
        )
        .with_metadata(
            "channel_authenticated",
            AuditValue::Bool(report.channel_authenticated),
        )
        .with_metadata(
            "platform_identity_verified",
            AuditValue::Bool(report.platform_identity_verified),
        )
        .with_metadata(
            "platform_identity_authorized",
            AuditValue::Bool(report.platform_identity_authorized),
        )
        .with_metadata(
            "replay_protection_checked",
            AuditValue::Bool(report.replay_protection_checked),
        )
        .with_metadata(
            "replay_nonce_reused",
            AuditValue::Bool(report.replay_nonce_reused),
        )
        .with_metadata(
            "command_allowlisted",
            AuditValue::Bool(report.command_allowlisted),
        )
        .with_metadata(
            "command_injection_detected",
            AuditValue::Bool(report.command_injection_detected),
        )
        .with_metadata("stale_envelope", AuditValue::Bool(report.stale_envelope))
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "remote_commands_enabled",
            AuditValue::Bool(report.remote_commands_enabled),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one local platform command ingress review to the audit journal.
///
/// This records sanitized mocked-platform control outcomes only. It never
/// stores token material, records command text, delivers messages, performs
/// network I/O, enables remote commands, executes live actions, signs, or
/// broadcasts.
pub fn append_platform_command_ingress_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &PlatformCommandIngressReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "communications-platform-command-ingress-{}",
            report.ingress_id
        ),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "platform-command-ingress",
        "platform command ingress recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(report.communications_version.clone()),
        )
        .with_metadata("ingress_id", AuditValue::Text(report.ingress_id.clone()))
        .with_metadata("platform", AuditValue::Text(report.platform.clone()))
        .with_metadata("channel_id", AuditValue::Text(report.channel_id.clone()))
        .with_metadata(
            "channel_kind",
            AuditValue::Text(format!("{:?}", report.channel_kind)),
        )
        .with_metadata(
            "platform_message_id",
            AuditValue::Text(report.platform_message_id.clone()),
        )
        .with_metadata(
            "platform_identity",
            AuditValue::Text(report.platform_identity.clone()),
        )
        .with_metadata(
            "command_kind",
            AuditValue::Text(format!("{:?}", report.command.kind)),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "token_reference_present",
            AuditValue::Bool(report.token_reference_present),
        )
        .with_metadata(
            "raw_material_observed",
            AuditValue::Bool(report.token_secret_material_present),
        )
        .with_metadata(
            "platform_signature_verified",
            AuditValue::Bool(report.platform_signature_verified),
        )
        .with_metadata(
            "platform_identity_authorized",
            AuditValue::Bool(report.platform_identity_authorized),
        )
        .with_metadata(
            "channel_permission_granted",
            AuditValue::Bool(report.channel_permission_granted),
        )
        .with_metadata(
            "replay_nonce_reused",
            AuditValue::Bool(report.replay_nonce_reused),
        )
        .with_metadata(
            "command_injection_detected",
            AuditValue::Bool(report.command_injection_detected),
        )
        .with_metadata("stale_message", AuditValue::Bool(report.stale_message))
        .with_metadata(
            "provider_rate_limited",
            AuditValue::Bool(report.provider_rate_limited),
        )
        .with_metadata(
            "provider_outage_observed",
            AuditValue::Bool(report.provider_outage_observed),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "remote_commands_enabled",
            AuditValue::Bool(report.remote_commands_enabled),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "message_delivered",
            AuditValue::Bool(report.message_delivered),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one local authenticated channel adapter validation to the audit journal.
///
/// This records sanitized channel-adapter control outcomes only. It never
/// delivers messages, performs network I/O, enables remote commands, executes
/// live actions, signs, or broadcasts.
pub fn append_channel_adapter_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ChannelAdapterValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "communications-channel-adapter-validation-{}",
            report.validation_id
        ),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "channel-adapter-validation",
        "channel adapter validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(report.communications_version.clone()),
        )
        .with_metadata(
            "validation_id",
            AuditValue::Text(report.validation_id.clone()),
        )
        .with_metadata("channel_id", AuditValue::Text(report.channel_id.clone()))
        .with_metadata(
            "channel_kind",
            AuditValue::Text(format!("{:?}", report.channel_kind)),
        )
        .with_metadata("envelope_id", AuditValue::Text(report.envelope_id.clone()))
        .with_metadata("dispatch_id", AuditValue::Text(report.dispatch_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("envelope_ready", AuditValue::Bool(report.envelope_ready))
        .with_metadata(
            "dispatch_recorded_locally",
            AuditValue::Bool(report.dispatch_recorded_locally),
        )
        .with_metadata(
            "channel_authenticated",
            AuditValue::Bool(report.channel_authenticated),
        )
        .with_metadata(
            "platform_identity_authorized",
            AuditValue::Bool(report.platform_identity_authorized),
        )
        .with_metadata(
            "replay_protection_checked",
            AuditValue::Bool(report.replay_protection_checked),
        )
        .with_metadata(
            "replay_nonce_reused",
            AuditValue::Bool(report.replay_nonce_reused),
        )
        .with_metadata(
            "provider_rate_limited",
            AuditValue::Bool(report.provider_rate_limited),
        )
        .with_metadata(
            "provider_outage_observed",
            AuditValue::Bool(report.provider_outage_observed),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "outbound_delivery_requested",
            AuditValue::Bool(report.outbound_delivery_requested),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "message_delivered",
            AuditValue::Bool(report.message_delivered),
        )
        .with_metadata(
            "remote_commands_enabled",
            AuditValue::Bool(report.remote_commands_enabled),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one local channel session validation summary to the audit journal.
///
/// This records sanitized channel-session control outcomes only. It never
/// delivers messages, performs network I/O, enables remote commands, executes
/// live actions, signs, or broadcasts.
pub fn append_channel_session_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &ChannelSessionValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "communications-channel-session-validation-{}",
            report.session_id
        ),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "channel-session-validation",
        "channel session validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(report.communications_version.clone()),
        )
        .with_metadata("session_id", AuditValue::Text(report.session_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "total_validation_count",
            AuditValue::Text(report.total_validation_count.to_string()),
        )
        .with_metadata(
            "accepted_validation_count",
            AuditValue::Text(report.accepted_validation_count.to_string()),
        )
        .with_metadata(
            "rejected_unauthenticated_count",
            AuditValue::Text(report.rejected_unauthenticated_count.to_string()),
        )
        .with_metadata(
            "rejected_replay_count",
            AuditValue::Text(report.rejected_replay_count.to_string()),
        )
        .with_metadata(
            "rejected_provider_unavailable_count",
            AuditValue::Text(report.rejected_provider_unavailable_count.to_string()),
        )
        .with_metadata("envelope_ready", AuditValue::Bool(report.envelope_ready))
        .with_metadata(
            "dispatch_recorded_locally",
            AuditValue::Bool(report.dispatch_recorded_locally),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "outbound_delivery_requested",
            AuditValue::Bool(report.outbound_delivery_requested),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "message_delivered",
            AuditValue::Bool(report.message_delivered),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
}

/// Append one local platform adapter control review to the audit journal.
///
/// This records sanitized platform-control outcomes only. It never stores
/// tokens, delivers messages, performs network I/O, enables remote commands,
/// executes live actions, signs, or broadcasts.
pub fn append_platform_adapter_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &PlatformAdapterReviewReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CommunicationError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "communications-platform-adapter-review-{}",
            report.review_id
        ),
        AuditEventKind::SecurityAlert,
        COMMUNICATIONS_STATE_SUBSYSTEM,
        "platform-adapter-review",
        "platform adapter review recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "communications_version",
            AuditValue::Text(report.communications_version.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata("channel_id", AuditValue::Text(report.channel_id.clone()))
        .with_metadata(
            "channel_kind",
            AuditValue::Text(format!("{:?}", report.channel_kind)),
        )
        .with_metadata("envelope_id", AuditValue::Text(report.envelope_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("envelope_ready", AuditValue::Bool(report.envelope_ready))
        .with_metadata(
            "token_reference_present",
            AuditValue::Bool(report.token_reference_present),
        )
        .with_metadata(
            "raw_material_observed",
            AuditValue::Bool(report.token_secret_material_present),
        )
        .with_metadata(
            "platform_identity_verified",
            AuditValue::Bool(report.platform_identity_verified),
        )
        .with_metadata(
            "platform_identity_authorized",
            AuditValue::Bool(report.platform_identity_authorized),
        )
        .with_metadata(
            "channel_permission_granted",
            AuditValue::Bool(report.channel_permission_granted),
        )
        .with_metadata(
            "command_injection_blocked",
            AuditValue::Bool(report.command_injection_blocked),
        )
        .with_metadata(
            "require_delivery_kill_switch",
            AuditValue::Bool(report.require_delivery_kill_switch),
        )
        .with_metadata(
            "require_audit_state_preflight",
            AuditValue::Bool(report.require_audit_state_preflight),
        )
        .with_metadata(
            "require_delivery_idempotency",
            AuditValue::Bool(report.require_delivery_idempotency),
        )
        .with_metadata(
            "require_rate_limit_controls",
            AuditValue::Bool(report.require_rate_limit_controls),
        )
        .with_metadata(
            "require_outage_backoff_controls",
            AuditValue::Bool(report.require_outage_backoff_controls),
        )
        .with_metadata(
            "require_payload_redaction",
            AuditValue::Bool(report.require_payload_redaction),
        )
        .with_metadata("token_revoked", AuditValue::Bool(report.token_revoked))
        .with_metadata(
            "provider_rate_limited",
            AuditValue::Bool(report.provider_rate_limited),
        )
        .with_metadata(
            "provider_outage_observed",
            AuditValue::Bool(report.provider_outage_observed),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "outbound_delivery_requested",
            AuditValue::Bool(report.outbound_delivery_requested),
        )
        .with_metadata(
            "outbound_network_used",
            AuditValue::Bool(report.outbound_network_used),
        )
        .with_metadata(
            "message_delivered",
            AuditValue::Bool(report.message_delivered),
        )
        .with_metadata(
            "remote_commands_enabled",
            AuditValue::Bool(report.remote_commands_enabled),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(CommunicationError::from)
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

fn route_command(command: &OperatorCommand) -> (OperatorCommandAction, bool, String) {
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

struct OperatorCommandAuthorizationDecision {
    operator_authorized: bool,
    status: OperatorCommandAuthorizationStatus,
    reason: String,
}

fn authorize_operator_command(
    command: &OperatorCommand,
    config: &CommunicationBoundaryConfig,
) -> OperatorCommandAuthorizationDecision {
    if command.source == OperatorCommandSource::LocalCli {
        if !config.cli_enabled {
            return OperatorCommandAuthorizationDecision {
                operator_authorized: false,
                status: OperatorCommandAuthorizationStatus::RejectedCliDisabled,
                reason: "local CLI command routing is disabled by configuration".to_owned(),
            };
        }
        return OperatorCommandAuthorizationDecision {
            operator_authorized: true,
            status: OperatorCommandAuthorizationStatus::AuthorizedLocalCli,
            reason: "local CLI command source authorized for local handling".to_owned(),
        };
    }

    OperatorCommandAuthorizationDecision {
        operator_authorized: false,
        status: OperatorCommandAuthorizationStatus::RejectedRemoteSource,
        reason:
            "remote operator command sources require external authentication and remain disabled"
                .to_owned(),
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

fn channel_safety_by_id(
    safety_states: &[NotificationChannelSafetyState],
) -> BTreeMap<String, &NotificationChannelSafetyState> {
    safety_states
        .iter()
        .map(|safety| (safety.channel_id.to_ascii_lowercase(), safety))
        .collect()
}

fn channel_dispatch_for(
    channel: &NotificationChannelProfile,
    safety: Option<&&NotificationChannelSafetyState>,
) -> NotificationChannelDispatch {
    if !channel.enabled {
        return NotificationChannelDispatch {
            channel_id: channel.id.clone(),
            kind: channel.kind,
            status: NotificationChannelDispatchStatus::Disabled,
            outbound_network_used: false,
            rate_limited: false,
            outage_blocked: false,
            reason: "channel is disabled".to_owned(),
        };
    }

    if let Some(safety) = safety {
        if safety.outage_active {
            return NotificationChannelDispatch {
                channel_id: channel.id.clone(),
                kind: channel.kind,
                status: NotificationChannelDispatchStatus::ChannelOutage,
                outbound_network_used: false,
                rate_limited: false,
                outage_blocked: true,
                reason: format!("local channel outage observation: {}", safety.outage_reason),
            };
        }
        if safety.messages_sent_in_window >= safety.max_messages_per_window {
            return NotificationChannelDispatch {
                channel_id: channel.id.clone(),
                kind: channel.kind,
                status: NotificationChannelDispatchStatus::RateLimited,
                outbound_network_used: false,
                rate_limited: true,
                outage_blocked: false,
                reason: "local channel rate limit reached for current window".to_owned(),
            };
        }
    }

    match channel.kind {
        CommunicationChannelKind::Cli | CommunicationChannelKind::LocalStdout => {
            NotificationChannelDispatch {
                channel_id: channel.id.clone(),
                kind: channel.kind,
                status: NotificationChannelDispatchStatus::RecordedLocally,
                outbound_network_used: false,
                rate_limited: false,
                outage_blocked: false,
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
            rate_limited: false,
            outage_blocked: false,
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

    if channels
        .iter()
        .all(|channel| channel.status == NotificationChannelDispatchStatus::RateLimited)
    {
        return NotificationDispatchStatus::BlockedRateLimited;
    }

    if channels
        .iter()
        .all(|channel| channel.status == NotificationChannelDispatchStatus::ChannelOutage)
    {
        return NotificationDispatchStatus::BlockedChannelOutage;
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

fn push_code(codes: &mut Vec<String>, condition: bool, code: &'static str) {
    if condition {
        codes.push(code.to_owned());
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

fn contains_command_injection_text(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let blocked_fragments = [
        "&&",
        "||",
        "$(",
        "`",
        ";",
        "|",
        ">",
        "<",
        "\n",
        "\r",
        "../",
        "..\\",
        "curl ",
        "wget ",
        "powershell",
        "cmd.exe",
        "bash ",
        "sh -c",
    ];
    blocked_fragments
        .iter()
        .any(|fragment| lower.contains(fragment))
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

impl CommunicationError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[CommunicationViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::AuditJournalFailed { .. } | Self::StateStoreFailed { .. } => &[],
        }
    }
}

impl From<crate::AuditError> for CommunicationError {
    fn from(error: crate::AuditError) -> Self {
        Self::AuditJournalFailed {
            reason: error.to_string(),
        }
    }
}

impl From<StateStoreError> for CommunicationError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
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
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "communication audit journal failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(formatter, "communication state store failed: {reason}")
            }
        }
    }
}

impl Error for CommunicationError {}

#[cfg(test)]
mod tests {
    use super::{
        append_channel_adapter_validation_audit, append_channel_session_validation_audit,
        append_notification_dispatch_audit, append_platform_adapter_review_audit,
        append_platform_command_ingress_audit, append_remote_command_envelope_validation_audit,
        append_remote_command_security_review_audit, append_routed_operator_command_audit,
        parse_cli_command, persist_channel_adapter_validation_checkpoint,
        persist_channel_session_validation_checkpoint, persist_notification_dispatch_checkpoint,
        persist_platform_adapter_review_checkpoint, persist_platform_command_ingress_checkpoint,
        persist_remote_command_envelope_validation_checkpoint,
        persist_remote_command_security_review_checkpoint,
        persist_routed_operator_command_checkpoint,
        review_communication_delivery_provider_boundary, review_platform_adapter_controls,
        review_platform_command_ingress, review_remote_command_security, validate_channel_adapter,
        validate_channel_session, validate_remote_command_envelope, ChannelAdapterValidationReport,
        ChannelAdapterValidationRequest, ChannelAdapterValidationStatus,
        ChannelSessionValidationReport, ChannelSessionValidationStatus,
        CommunicationBoundaryConfig, CommunicationDeliveryProviderBoundaryRequest,
        CommunicationDeliveryProviderBoundaryStatus, DeterministicNotificationBoundary,
        DeterministicOperatorCommandRouter, NotificationChannelDispatchStatus,
        NotificationChannelProfile, NotificationChannelSafetyState, NotificationDispatchRecord,
        NotificationDispatchStatus, NotificationPublishRequest, NotificationPublisher,
        NotificationSeverity, OperatorCommand, OperatorCommandAction,
        OperatorCommandAuthorizationStatus, OperatorCommandKind, OperatorCommandRouter,
        OperatorCommandRoutingRequest, OperatorCommandSource, OperatorNotification,
        PlatformAdapterReviewReport, PlatformAdapterReviewRequest, PlatformAdapterReviewStatus,
        PlatformCommandIngressReport, PlatformCommandIngressRequest, PlatformCommandIngressStatus,
        RemoteCommandEnvelopeValidationReport, RemoteCommandEnvelopeValidationRequest,
        RemoteCommandEnvelopeValidationStatus, RemoteCommandSecurityReviewReport,
        RemoteCommandSecurityReviewRequest, RemoteCommandSecurityReviewStatus,
        COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY,
        COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY,
    };
    use crate::{AppendOnlyAuditJournal, CommunicationConfig, SqliteWalStateStore, StateStore};
    use std::{env, fs, path::PathBuf, process};

    fn ready_remote_review(source: OperatorCommandSource) -> RemoteCommandSecurityReviewReport {
        review_remote_command_security(&RemoteCommandSecurityReviewRequest {
            review_id: "remote-envelope-security-review".to_owned(),
            source,
            channel_authentication_required: true,
            platform_identity_verification_required: true,
            platform_identity_authorization_required: true,
            replay_protection_required: true,
            command_allowlist_required: true,
            unsafe_commands_blocked: true,
            remote_command_enablement_requested: false,
            outbound_network_requested: false,
            live_execution_requested: false,
            reviewed_at_unix_ms: 1_700_000_000_201,
        })
        .expect("ready remote security review should validate")
    }

    fn remote_status_command() -> OperatorCommand {
        OperatorCommand {
            id: "operator-command-remote-envelope-status".to_owned(),
            source: OperatorCommandSource::MessagingChannel,
            kind: OperatorCommandKind::Status,
            args: Vec::new(),
            sanitized_raw: "status".to_owned(),
            received_at_unix_ms: 1_700_000_000_210,
        }
    }

    fn ready_remote_envelope_request() -> RemoteCommandEnvelopeValidationRequest {
        RemoteCommandEnvelopeValidationRequest {
            envelope_id: "remote-command-envelope-ready".to_owned(),
            command: remote_status_command(),
            security_review: ready_remote_review(OperatorCommandSource::MessagingChannel),
            platform_identity: "operator-chat-id-1".to_owned(),
            authorization_policy: "status-readonly-allowlist".to_owned(),
            authentication_reference: "local-auth-proof-digest-ref-1".to_owned(),
            replay_nonce: "remote-envelope-nonce-1".to_owned(),
            channel_authenticated: true,
            platform_identity_verified: true,
            platform_identity_authorized: true,
            replay_protection_checked: true,
            replay_nonce_reused: false,
            command_allowlisted: true,
            received_at_unix_ms: 1_700_000_000_210,
            now_unix_ms: 1_700_000_000_220,
            max_age_ms: 60_000,
            remote_command_enablement_requested: false,
            outbound_network_used: false,
            live_execution_performed: false,
            signing_or_broadcast_performed: false,
        }
    }

    fn ready_platform_command_ingress_request() -> PlatformCommandIngressRequest {
        PlatformCommandIngressRequest {
            ingress_id: "platform-command-ingress-ready".to_owned(),
            platform: "slack-mock".to_owned(),
            channel: NotificationChannelProfile::from_identifier("chat:ops"),
            platform_message_id: "mock-platform-message-1".to_owned(),
            platform_identity: "operator-chat-id-1".to_owned(),
            command_text: "status".to_owned(),
            token_reference_present: true,
            token_secret_material_present: false,
            platform_signature_verified: true,
            platform_identity_authorized: true,
            channel_permission_granted: true,
            replay_nonce: "platform-command-nonce-1".to_owned(),
            replay_nonce_reused: false,
            provider_rate_limited: false,
            provider_outage_observed: false,
            received_at_unix_ms: 1_700_000_000_210,
            now_unix_ms: 1_700_000_000_220,
            max_age_ms: 60_000,
            outbound_network_used: false,
            live_execution_performed: false,
            signing_or_broadcast_performed: false,
        }
    }

    fn ready_local_notification_dispatch() -> NotificationDispatchRecord {
        DeterministicNotificationBoundary::new()
            .publish(&NotificationPublishRequest {
                id: "channel-adapter-notification-request".to_owned(),
                notification: OperatorNotification {
                    id: "channel-adapter-notification".to_owned(),
                    severity: NotificationSeverity::Info,
                    title: "Local channel adapter validation".to_owned(),
                    body: "Local channel adapter validation recorded without delivery".to_owned(),
                    channels: vec!["cli".to_owned()],
                    created_at_unix_ms: 1_700_000_000_230,
                },
                config: CommunicationBoundaryConfig {
                    notification_channels: vec![NotificationChannelProfile::from_identifier("cli")],
                    ..CommunicationBoundaryConfig::default()
                },
                channel_safety: Vec::new(),
                now_unix_ms: 1_700_000_000_231,
            })
            .expect("local notification dispatch should record")
    }

    fn ready_channel_adapter_request() -> ChannelAdapterValidationRequest {
        ChannelAdapterValidationRequest {
            validation_id: "channel-adapter-validation-ready".to_owned(),
            channel: NotificationChannelProfile::from_identifier("cli"),
            envelope: validate_remote_command_envelope(&ready_remote_envelope_request())
                .expect("ready envelope should validate locally"),
            dispatch: ready_local_notification_dispatch(),
            adapter_authentication_reference: "local-channel-auth-digest-ref".to_owned(),
            platform_identity: "operator-chat-id-1".to_owned(),
            replay_nonce: "channel-adapter-nonce-1".to_owned(),
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
            validated_at_unix_ms: 1_700_000_000_240,
        }
    }

    fn ready_platform_adapter_review_request() -> PlatformAdapterReviewRequest {
        PlatformAdapterReviewRequest {
            review_id: "platform-adapter-review-ready".to_owned(),
            channel: NotificationChannelProfile::from_identifier("cli"),
            envelope: validate_remote_command_envelope(&ready_remote_envelope_request())
                .expect("ready envelope should validate locally"),
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
            reviewed_at_unix_ms: 1_700_000_000_260,
        }
    }

    fn ready_channel_session_report() -> ChannelSessionValidationReport {
        let accepted = validate_channel_adapter(&ready_channel_adapter_request())
            .expect("ready channel adapter validation should pass locally");
        let mut unauthenticated_request = ready_channel_adapter_request();
        unauthenticated_request.validation_id = "delivery-provider-session-unauth".to_owned();
        unauthenticated_request.channel_authenticated = false;
        unauthenticated_request.platform_identity_authorized = false;
        let unauthenticated = validate_channel_adapter(&unauthenticated_request)
            .expect("unauthenticated channel adapter should be blocked locally");
        let mut replay_request = ready_channel_adapter_request();
        replay_request.validation_id = "delivery-provider-session-replay".to_owned();
        replay_request.replay_nonce_reused = true;
        let replay = validate_channel_adapter(&replay_request)
            .expect("replayed channel adapter should be blocked locally");
        let mut provider_unavailable_request = ready_channel_adapter_request();
        provider_unavailable_request.validation_id =
            "delivery-provider-session-unavailable".to_owned();
        provider_unavailable_request.provider_rate_limited = true;
        provider_unavailable_request.provider_outage_observed = true;
        let provider_unavailable = validate_channel_adapter(&provider_unavailable_request)
            .expect("provider unavailable channel adapter should be blocked locally");

        validate_channel_session(
            "delivery-provider-channel-session",
            &[accepted, unauthenticated, replay, provider_unavailable],
        )
        .expect("channel session should summarize local controls")
    }

    fn ready_communication_delivery_provider_boundary_request(
    ) -> CommunicationDeliveryProviderBoundaryRequest {
        CommunicationDeliveryProviderBoundaryRequest {
            review_id: "communications-delivery-provider-boundary".to_owned(),
            channel_session: ready_channel_session_report(),
            platform_adapter: review_platform_adapter_controls(
                &ready_platform_adapter_review_request(),
            )
            .expect("ready platform adapter review should pass locally"),
            provider_delivery_evidence_available: false,
            provider_rate_limit_evidence_available: false,
            provider_outage_evidence_available: false,
            platform_identity_evidence_available: false,
            min_remaining_external_evidence: 4,
            remaining_external_evidence: vec![
                "authenticated real provider delivery evidence".to_owned(),
                "provider-side rate-limit reconciliation evidence".to_owned(),
                "provider outage/backoff evidence".to_owned(),
                "production platform identity authorization evidence".to_owned(),
            ],
            outbound_network_used: false,
            message_delivered: false,
            provider_call_performed: false,
            token_secret_material_loaded: false,
            live_execution_performed: false,
            signing_or_broadcast_performed: false,
            production_ready_claimed: false,
        }
    }

    #[test]
    fn communication_config_rejects_outbound_network_and_execution_commands() {
        let config = CommunicationBoundaryConfig {
            outbound_network_enabled: true,
            allow_execution_commands: true,
            require_local_operator_authorization: false,
            remote_operator_commands_enabled: true,
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
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "COMMUNICATION_LOCAL_OPERATOR_AUTH_REQUIRED" }));
        assert!(error.violations().iter().any(|violation| {
            violation.code() == "COMMUNICATION_REMOTE_OPERATOR_COMMANDS_DENIED_IN_PHASE_12"
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
        assert!(route.operator_authorized);
        assert_eq!(
            route.authorization_status,
            OperatorCommandAuthorizationStatus::AuthorizedLocalCli
        );
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
        assert!(route.operator_authorized);
        assert_eq!(
            route.authorization_status,
            OperatorCommandAuthorizationStatus::AuthorizedLocalCli
        );
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
        assert!(route.operator_authorized);
        assert_eq!(
            route.authorization_status,
            OperatorCommandAuthorizationStatus::AuthorizedLocalCli
        );
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
    }

    #[test]
    fn command_router_rejects_disabled_local_cli_authorization() {
        let command =
            parse_cli_command(&["status".to_owned()], 1_000).expect("status command should parse");
        let request = OperatorCommandRoutingRequest {
            id: "route-request-auth-disabled".to_owned(),
            command,
            config: CommunicationBoundaryConfig {
                cli_enabled: false,
                ..CommunicationBoundaryConfig::default()
            },
        };

        let route = DeterministicOperatorCommandRouter::new()
            .route(&request)
            .expect("disabled CLI should produce a rejection route");

        assert_eq!(
            route.action,
            OperatorCommandAction::RejectUnauthorizedCommand
        );
        assert!(!route.accepted);
        assert!(!route.operator_authorized);
        assert_eq!(
            route.authorization_status,
            OperatorCommandAuthorizationStatus::RejectedCliDisabled
        );
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
    }

    #[test]
    fn command_router_rejects_remote_command_sources_without_external_auth() {
        let command = OperatorCommand {
            id: "operator-command-remote-status".to_owned(),
            source: OperatorCommandSource::MessagingChannel,
            kind: OperatorCommandKind::Status,
            args: Vec::new(),
            sanitized_raw: "status".to_owned(),
            received_at_unix_ms: 1_000,
        };
        let request = OperatorCommandRoutingRequest {
            id: "route-request-remote-source".to_owned(),
            command,
            config: CommunicationBoundaryConfig::default(),
        };

        let route = DeterministicOperatorCommandRouter::new()
            .route(&request)
            .expect("remote command should produce a fail-closed rejection route");

        assert_eq!(
            route.action,
            OperatorCommandAction::RejectUnauthorizedCommand
        );
        assert!(!route.accepted);
        assert!(!route.operator_authorized);
        assert_eq!(
            route.authorization_status,
            OperatorCommandAuthorizationStatus::RejectedRemoteSource
        );
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
    }

    #[test]
    fn remote_command_security_review_accepts_complete_controls_without_enabling_remote() {
        let report = review_remote_command_security(&RemoteCommandSecurityReviewRequest {
            review_id: "remote-command-review-complete".to_owned(),
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
            reviewed_at_unix_ms: 1_700_000_000_101,
        })
        .expect("complete remote command review should be ready locally");

        assert_eq!(
            report.status,
            RemoteCommandSecurityReviewStatus::ReadyForLocalReview
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(report.channel_authentication_required);
        assert!(report.platform_identity_verification_required);
        assert!(report.platform_identity_authorization_required);
        assert!(report.replay_protection_required);
        assert!(report.command_allowlist_required);
        assert!(report.unsafe_commands_blocked);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn remote_command_security_review_blocks_missing_or_unsafe_controls() {
        let report = review_remote_command_security(&RemoteCommandSecurityReviewRequest {
            review_id: "remote-command-review-blocked".to_owned(),
            source: OperatorCommandSource::Dashboard,
            channel_authentication_required: false,
            platform_identity_verification_required: false,
            platform_identity_authorization_required: false,
            replay_protection_required: false,
            command_allowlist_required: false,
            unsafe_commands_blocked: false,
            remote_command_enablement_requested: true,
            outbound_network_requested: true,
            live_execution_requested: true,
            reviewed_at_unix_ms: 1_700_000_000_102,
        })
        .expect("blocked remote command review should still report local controls");

        assert_eq!(
            report.status,
            RemoteCommandSecurityReviewStatus::BlockedMissingControls
        );
        assert_eq!(report.missing_control_count, 9);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn remote_command_security_review_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("remote-command-security-review");
        let state_path = temp_state_path("remote-command-security-review");
        let report = review_remote_command_security(&RemoteCommandSecurityReviewRequest {
            review_id: "remote-command-review-audit-state".to_owned(),
            source: OperatorCommandSource::Scheduler,
            channel_authentication_required: true,
            platform_identity_verification_required: true,
            platform_identity_authorization_required: true,
            replay_protection_required: true,
            command_allowlist_required: true,
            unsafe_commands_blocked: true,
            remote_command_enablement_requested: false,
            outbound_network_requested: false,
            live_execution_requested: false,
            reviewed_at_unix_ms: 1_700_000_000_103,
        })
        .expect("remote command review should be ready locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_remote_command_security_review_audit(&mut journal, &report, 1_700_000_000_104)
                .expect("remote command review audit writes");
        let checkpoint = persist_remote_command_security_review_checkpoint(
            &mut store,
            &report,
            1_700_000_000_105,
        )
        .expect("remote command review checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_REMOTE_COMMAND_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("remote command review checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"remote_commands_enabled\":false"));
        assert!(recovered.value.contains("\"outbound_network_used\":false"));
        assert!(recovered
            .value
            .contains("\"live_execution_performed\":false"));
        assert!(recovered
            .value
            .contains("\"signing_or_broadcast_performed\":false"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn remote_command_envelope_validation_accepts_authenticated_safe_command_without_routing() {
        let report = validate_remote_command_envelope(&ready_remote_envelope_request())
            .expect("ready remote envelope should validate locally");

        assert_eq!(
            report.status,
            RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview
        );
        assert_eq!(report.command_kind, OperatorCommandKind::Status);
        assert_eq!(report.missing_control_count, 0);
        assert!(report.security_review_ready);
        assert!(report.channel_authenticated);
        assert!(report.platform_identity_verified);
        assert!(report.platform_identity_authorized);
        assert!(report.replay_protection_checked);
        assert!(!report.replay_nonce_reused);
        assert!(report.command_allowlisted);
        assert!(!report.command_injection_detected);
        assert!(!report.stale_envelope);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn platform_command_ingress_accepts_mocked_authenticated_command_without_delivery() {
        let report = review_platform_command_ingress(&ready_platform_command_ingress_request())
            .expect("ready mocked platform command should validate locally");

        assert_eq!(
            report.status,
            PlatformCommandIngressStatus::ReadyForEnvelopeValidation
        );
        assert_eq!(report.command.kind, OperatorCommandKind::Status);
        assert_eq!(report.missing_control_count, 0);
        assert!(report.token_reference_present);
        assert!(!report.token_secret_material_present);
        assert!(report.platform_signature_verified);
        assert!(report.platform_identity_authorized);
        assert!(report.channel_permission_granted);
        assert!(!report.replay_nonce_reused);
        assert!(!report.command_injection_detected);
        assert!(!report.stale_message);
        assert!(!report.provider_rate_limited);
        assert!(!report.provider_outage_observed);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn platform_command_ingress_blocks_unsafe_replay_and_provider_failures() {
        let mut request = ready_platform_command_ingress_request();
        request.ingress_id = "platform-command-ingress-blocked".to_owned();
        request.platform_signature_verified = false;
        request.platform_identity_authorized = false;
        request.channel_permission_granted = false;
        request.replay_nonce_reused = true;
        request.provider_rate_limited = true;
        request.provider_outage_observed = true;
        request.command_text = "status && curl http://example.invalid".to_owned();

        let report = review_platform_command_ingress(&request)
            .expect("blocked mocked platform command should produce local report");

        assert_eq!(
            report.status,
            PlatformCommandIngressStatus::BlockedMissingControls
        );
        assert!(!report.platform_signature_verified);
        assert!(!report.platform_identity_authorized);
        assert!(!report.channel_permission_granted);
        assert!(report.replay_nonce_reused);
        assert!(report.command_injection_detected);
        assert!(report.provider_rate_limited);
        assert!(report.provider_outage_observed);
        assert!(report.missing_control_count >= 7);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn platform_command_ingress_fails_closed_on_token_material_or_side_effects() {
        let mut request = ready_platform_command_ingress_request();
        request.ingress_id = "platform-command-ingress-side-effect".to_owned();
        request.token_secret_material_present = true;
        request.outbound_network_used = true;
        request.live_execution_performed = true;

        let error = review_platform_command_ingress(&request)
            .expect_err("platform command side-effect report must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "COMMUNICATION_PLATFORM_COMMAND_SIDE_EFFECT" }));
    }

    #[test]
    fn platform_command_ingress_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("platform-command-ingress");
        let state_path = temp_state_path("platform-command-ingress");
        let report = review_platform_command_ingress(&ready_platform_command_ingress_request())
            .expect("ready mocked platform command should validate locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_platform_command_ingress_audit(&mut journal, &report, 1_700_000_000_211)
                .expect("platform command ingress audit writes");
        let checkpoint =
            persist_platform_command_ingress_checkpoint(&mut store, &report, 1_700_000_000_212)
                .expect("platform command ingress checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_PLATFORM_COMMAND_INGRESS_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("platform command ingress checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: PlatformCommandIngressReport =
            serde_json::from_str(&recovered.value).expect("platform command report parses");
        assert_eq!(
            recovered_report.status,
            PlatformCommandIngressStatus::ReadyForEnvelopeValidation
        );
        assert_eq!(recovered_report.command.kind, OperatorCommandKind::Status);
        assert!(recovered_report.token_reference_present);
        assert!(!recovered_report.token_secret_material_present);
        assert!(recovered_report.platform_signature_verified);
        assert!(recovered_report.platform_identity_authorized);
        assert!(!recovered_report.command_injection_detected);
        assert!(!recovered_report.remote_commands_enabled);
        assert!(!recovered_report.outbound_network_used);
        assert!(!recovered_report.message_delivered);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn remote_command_envelope_validation_blocks_replay_stale_and_unsafe_command() {
        let mut request = ready_remote_envelope_request();
        request.envelope_id = "remote-command-envelope-blocked".to_owned();
        request.command.kind = OperatorCommandKind::WithdrawalRequest;
        request.command.sanitized_raw = "withdraw".to_owned();
        request.command_allowlisted = false;
        request.replay_nonce_reused = true;
        request.now_unix_ms = request.received_at_unix_ms + 120_000;

        let report = validate_remote_command_envelope(&request)
            .expect("blocked remote envelope should still produce a sanitized report");

        assert_eq!(
            report.status,
            RemoteCommandEnvelopeValidationStatus::BlockedMissingControls
        );
        assert_eq!(report.command_kind, OperatorCommandKind::WithdrawalRequest);
        assert!(report.replay_nonce_reused);
        assert!(report.stale_envelope);
        assert!(!report.command_allowlisted);
        assert!(!report.command_injection_detected);
        assert!(report.missing_control_count >= 4);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn remote_command_envelope_validation_blocks_command_injection_text() {
        let mut request = ready_remote_envelope_request();
        request.envelope_id = "remote-command-envelope-injection".to_owned();
        request.command.sanitized_raw = "status; curl http://example.invalid".to_owned();
        request.command.args = vec!["status".to_owned(), "&&".to_owned(), "withdraw".to_owned()];

        let report = validate_remote_command_envelope(&request)
            .expect("injection-like remote envelope should produce local blocked report");

        assert_eq!(
            report.status,
            RemoteCommandEnvelopeValidationStatus::BlockedMissingControls
        );
        assert!(report.command_allowlisted);
        assert!(report.command_injection_detected);
        assert!(report.missing_control_count >= 1);
        assert!(!report.remote_commands_enabled);
        assert!(!report.outbound_network_used);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn remote_command_envelope_validation_fails_closed_on_side_effect_flags() {
        let mut request = ready_remote_envelope_request();
        request.envelope_id = "remote-command-envelope-side-effect".to_owned();
        request.outbound_network_used = true;
        request.live_execution_performed = true;

        let error = validate_remote_command_envelope(&request)
            .expect_err("side-effect remote envelope report must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "COMMUNICATION_REMOTE_ENVELOPE_SIDE_EFFECT" }));
    }

    #[test]
    fn remote_command_envelope_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("remote-command-envelope-validation");
        let state_path = temp_state_path("remote-command-envelope-validation");
        let report = validate_remote_command_envelope(&ready_remote_envelope_request())
            .expect("ready remote envelope should validate locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_remote_command_envelope_validation_audit(
            &mut journal,
            &report,
            1_700_000_000_221,
        )
        .expect("remote envelope audit writes");
        let checkpoint = persist_remote_command_envelope_validation_checkpoint(
            &mut store,
            &report,
            1_700_000_000_222,
        )
        .expect("remote envelope checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_REMOTE_COMMAND_ENVELOPE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("remote envelope checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: RemoteCommandEnvelopeValidationReport =
            serde_json::from_str(&recovered.value).expect("remote envelope report parses");
        assert_eq!(
            recovered_report.status,
            RemoteCommandEnvelopeValidationStatus::ReadyForLocalReview
        );
        assert!(!recovered_report.remote_commands_enabled);
        assert!(!recovered_report.command_injection_detected);
        assert!(!recovered_report.outbound_network_used);
        assert!(!recovered_report.live_execution_performed);
        assert!(!recovered_report.signing_or_broadcast_performed);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn channel_adapter_validation_accepts_local_authenticated_channel_without_delivery() {
        let report = validate_channel_adapter(&ready_channel_adapter_request())
            .expect("ready channel adapter validation should pass locally");

        assert_eq!(
            report.status,
            ChannelAdapterValidationStatus::ReadyForLocalReview
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(report.envelope_ready);
        assert!(report.dispatch_recorded_locally);
        assert!(report.channel_authenticated);
        assert!(report.platform_identity_authorized);
        assert!(report.replay_protection_checked);
        assert!(report.require_delivery_kill_switch);
        assert!(report.require_audit_state_preflight);
        assert!(report.require_delivery_idempotency);
        assert!(report.require_rate_limit_controls);
        assert!(report.require_outage_backoff_controls);
        assert!(report.require_payload_redaction);
        assert!(!report.replay_nonce_reused);
        assert!(!report.provider_rate_limited);
        assert!(!report.provider_outage_observed);
        assert!(!report.outbound_delivery_requested);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.remote_commands_enabled);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn channel_adapter_validation_blocks_replay_rate_limit_and_outage() {
        let mut request = ready_channel_adapter_request();
        request.validation_id = "channel-adapter-validation-blocked".to_owned();
        request.replay_nonce_reused = true;
        request.provider_rate_limited = true;
        request.provider_outage_observed = true;

        let report = validate_channel_adapter(&request)
            .expect("blocked channel adapter validation should produce local report");

        assert_eq!(
            report.status,
            ChannelAdapterValidationStatus::BlockedMissingControls
        );
        assert!(report.replay_nonce_reused);
        assert!(report.provider_rate_limited);
        assert!(report.provider_outage_observed);
        assert!(report.missing_control_count >= 3);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.remote_commands_enabled);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn channel_adapter_validation_requires_future_delivery_preconditions() {
        let mut request = ready_channel_adapter_request();
        request.validation_id = "channel-adapter-validation-missing-preconditions".to_owned();
        request.require_delivery_kill_switch = false;
        request.require_audit_state_preflight = false;
        request.require_delivery_idempotency = false;
        request.require_rate_limit_controls = false;
        request.require_outage_backoff_controls = false;
        request.require_payload_redaction = false;

        let error = validate_channel_adapter(&request)
            .expect_err("channel adapter must require future delivery preconditions");
        for expected in [
            "COMMUNICATION_CHANNEL_ADAPTER_DELIVERY_KILL_SWITCH_REQUIRED",
            "COMMUNICATION_CHANNEL_ADAPTER_AUDIT_STATE_PREFLIGHT_REQUIRED",
            "COMMUNICATION_CHANNEL_ADAPTER_IDEMPOTENCY_REQUIRED",
            "COMMUNICATION_CHANNEL_ADAPTER_RATE_LIMIT_CONTROLS_REQUIRED",
            "COMMUNICATION_CHANNEL_ADAPTER_OUTAGE_BACKOFF_REQUIRED",
            "COMMUNICATION_CHANNEL_ADAPTER_PAYLOAD_REDACTION_REQUIRED",
        ] {
            assert!(
                error
                    .violations()
                    .iter()
                    .any(|violation| violation.code() == expected),
                "missing expected violation {expected}"
            );
        }
    }

    #[test]
    fn channel_adapter_validation_fails_closed_on_delivery_side_effects() {
        let mut request = ready_channel_adapter_request();
        request.validation_id = "channel-adapter-validation-side-effect".to_owned();
        request.outbound_delivery_requested = true;
        request.outbound_network_used = true;
        request.message_delivered = true;

        let error = validate_channel_adapter(&request)
            .expect_err("channel adapter side-effect report must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "COMMUNICATION_CHANNEL_ADAPTER_SIDE_EFFECT" }));
    }

    #[test]
    fn channel_adapter_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("channel-adapter-validation");
        let state_path = temp_state_path("channel-adapter-validation");
        let report = validate_channel_adapter(&ready_channel_adapter_request())
            .expect("ready channel adapter validation should pass locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_channel_adapter_validation_audit(&mut journal, &report, 1_700_000_000_241)
                .expect("channel adapter audit writes");
        let checkpoint =
            persist_channel_adapter_validation_checkpoint(&mut store, &report, 1_700_000_000_242)
                .expect("channel adapter checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_CHANNEL_ADAPTER_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("channel adapter checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: ChannelAdapterValidationReport =
            serde_json::from_str(&recovered.value).expect("channel adapter report parses");
        assert_eq!(
            recovered_report.status,
            ChannelAdapterValidationStatus::ReadyForLocalReview
        );
        assert!(!recovered_report.outbound_delivery_requested);
        assert!(recovered_report.require_delivery_kill_switch);
        assert!(recovered_report.require_audit_state_preflight);
        assert!(recovered_report.require_delivery_idempotency);
        assert!(recovered_report.require_rate_limit_controls);
        assert!(recovered_report.require_outage_backoff_controls);
        assert!(recovered_report.require_payload_redaction);
        assert!(!recovered_report.outbound_network_used);
        assert!(!recovered_report.message_delivered);
        assert!(!recovered_report.remote_commands_enabled);
        assert!(!recovered_report.live_execution_performed);
        assert!(!recovered_report.signing_or_broadcast_performed);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn channel_session_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("channel-session-validation");
        let state_path = temp_state_path("channel-session-validation");
        let accepted = validate_channel_adapter(&ready_channel_adapter_request())
            .expect("ready channel adapter validation should pass locally");
        let mut unauthenticated_request = ready_channel_adapter_request();
        unauthenticated_request.validation_id = "channel-session-unauthenticated".to_owned();
        unauthenticated_request.channel_authenticated = false;
        unauthenticated_request.platform_identity_authorized = false;
        let unauthenticated = validate_channel_adapter(&unauthenticated_request)
            .expect("unauthenticated channel adapter should be blocked locally");
        let mut replay_request = ready_channel_adapter_request();
        replay_request.validation_id = "channel-session-replay".to_owned();
        replay_request.replay_nonce_reused = true;
        let replay = validate_channel_adapter(&replay_request)
            .expect("replayed channel adapter should be blocked locally");
        let mut provider_unavailable_request = ready_channel_adapter_request();
        provider_unavailable_request.validation_id =
            "channel-session-provider-unavailable".to_owned();
        provider_unavailable_request.provider_rate_limited = true;
        provider_unavailable_request.provider_outage_observed = true;
        let provider_unavailable = validate_channel_adapter(&provider_unavailable_request)
            .expect("provider unavailable channel adapter should be blocked locally");
        let report = validate_channel_session(
            "channel-session-audit-state",
            &[accepted, unauthenticated, replay, provider_unavailable],
        )
        .expect("channel session should summarize local controls");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_channel_session_validation_audit(&mut journal, &report, 1_700_000_000_251)
                .expect("channel session audit writes");
        let checkpoint =
            persist_channel_session_validation_checkpoint(&mut store, &report, 1_700_000_000_252)
                .expect("channel session checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY
        );
        assert_eq!(
            report.status,
            ChannelSessionValidationStatus::ReadyForLocalReview
        );
        assert_eq!(report.total_validation_count, 4);
        assert_eq!(report.accepted_validation_count, 1);
        assert_eq!(report.rejected_unauthenticated_count, 1);
        assert_eq!(report.rejected_replay_count, 1);
        assert_eq!(report.rejected_provider_unavailable_count, 1);
        assert!(!report.outbound_delivery_requested);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_CHANNEL_SESSION_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("channel session checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: ChannelSessionValidationReport =
            serde_json::from_str(&recovered.value).expect("channel session report parses");
        assert_eq!(
            recovered_report.status,
            ChannelSessionValidationStatus::ReadyForLocalReview
        );
        assert_eq!(recovered_report.validation_ids.len(), 4);
        assert_eq!(recovered_report.rejected_unauthenticated_count, 1);
        assert_eq!(recovered_report.rejected_replay_count, 1);
        assert_eq!(recovered_report.rejected_provider_unavailable_count, 1);
        assert!(!recovered_report.outbound_network_used);
        assert!(!recovered_report.message_delivered);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn platform_adapter_review_accepts_local_controls_without_token_or_delivery() {
        let report = review_platform_adapter_controls(&ready_platform_adapter_review_request())
            .expect("ready platform adapter controls should validate locally");

        assert_eq!(
            report.status,
            PlatformAdapterReviewStatus::ReadyForLocalReview
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(report.envelope_ready);
        assert!(report.token_reference_present);
        assert!(!report.token_secret_material_present);
        assert!(report.platform_identity_verified);
        assert!(report.platform_identity_authorized);
        assert!(report.channel_permission_granted);
        assert!(report.command_injection_blocked);
        assert!(report.require_delivery_kill_switch);
        assert!(report.require_audit_state_preflight);
        assert!(report.require_delivery_idempotency);
        assert!(report.require_rate_limit_controls);
        assert!(report.require_outage_backoff_controls);
        assert!(report.require_payload_redaction);
        assert!(!report.token_revoked);
        assert!(!report.provider_rate_limited);
        assert!(!report.provider_outage_observed);
        assert!(!report.outbound_delivery_requested);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.remote_commands_enabled);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn communication_delivery_provider_boundary_blocks_pending_real_provider_evidence() {
        let report = review_communication_delivery_provider_boundary(
            &ready_communication_delivery_provider_boundary_request(),
        )
        .expect("local delivery-provider boundary should produce a blocked report");

        assert_eq!(
            report.status,
            CommunicationDeliveryProviderBoundaryStatus::BlockedPendingProviderDeliveryValidation
        );
        assert!(report.channel_session_ready);
        assert!(report.platform_adapter_ready);
        assert!(!report.provider_delivery_evidence_available);
        assert!(!report.provider_rate_limit_evidence_available);
        assert!(!report.provider_outage_evidence_available);
        assert!(!report.platform_identity_evidence_available);
        assert!(report.remaining_external_evidence_recorded);
        assert_eq!(report.remaining_external_evidence_count, 4);
        assert_eq!(report.blockers.len(), 4);
        assert!(report.violation_codes.is_empty());
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.provider_call_performed);
        assert!(!report.token_secret_material_loaded);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn communication_delivery_provider_boundary_fails_closed_on_side_effects() {
        let mut request = ready_communication_delivery_provider_boundary_request();
        request.review_id = "communications-delivery-provider-side-effect".to_owned();
        request.outbound_network_used = true;
        request.message_delivered = true;
        request.provider_call_performed = true;
        request.token_secret_material_loaded = true;
        request.live_execution_performed = true;
        request.signing_or_broadcast_performed = true;
        request.production_ready_claimed = true;

        let report = review_communication_delivery_provider_boundary(&request)
            .expect("side-effect report should be blocked without throwing away evidence");

        assert_eq!(
            report.status,
            CommunicationDeliveryProviderBoundaryStatus::Blocked
        );
        assert!(report.outbound_network_used);
        assert!(report.message_delivered);
        assert!(report.provider_call_performed);
        assert!(report.token_secret_material_loaded);
        assert!(report.live_execution_performed);
        assert!(report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
        for expected in [
            "COMMUNICATION_DELIVERY_PROVIDER_OUTBOUND_NETWORK_USED",
            "COMMUNICATION_DELIVERY_PROVIDER_MESSAGE_DELIVERED",
            "COMMUNICATION_DELIVERY_PROVIDER_CALL_PERFORMED",
            "COMMUNICATION_DELIVERY_PROVIDER_TOKEN_SECRET_LOADED",
            "COMMUNICATION_DELIVERY_PROVIDER_LIVE_EXECUTION",
            "COMMUNICATION_DELIVERY_PROVIDER_SIGNING_OR_BROADCAST",
            "COMMUNICATION_DELIVERY_PROVIDER_PRODUCTION_READY_CLAIMED",
        ] {
            assert!(
                report.violation_codes.iter().any(|code| code == expected),
                "missing expected violation code {expected}"
            );
        }
    }

    #[test]
    fn platform_adapter_review_blocks_revoked_token_injection_and_permission_gap() {
        let mut request = ready_platform_adapter_review_request();
        request.review_id = "platform-adapter-review-blocked".to_owned();
        request.channel_permission_granted = false;
        request.command_injection_blocked = false;
        request.token_revoked = true;
        request.provider_rate_limited = true;
        request.provider_outage_observed = true;

        let report = review_platform_adapter_controls(&request)
            .expect("blocked platform adapter controls should produce local report");

        assert_eq!(
            report.status,
            PlatformAdapterReviewStatus::BlockedMissingControls
        );
        assert!(!report.channel_permission_granted);
        assert!(!report.command_injection_blocked);
        assert!(report.token_revoked);
        assert!(report.provider_rate_limited);
        assert!(report.provider_outage_observed);
        assert!(report.missing_control_count >= 5);
        assert!(!report.outbound_network_used);
        assert!(!report.message_delivered);
        assert!(!report.remote_commands_enabled);
        assert!(!report.live_execution_performed);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn platform_adapter_review_requires_future_delivery_preconditions() {
        let mut request = ready_platform_adapter_review_request();
        request.review_id = "platform-adapter-review-missing-preconditions".to_owned();
        request.require_delivery_kill_switch = false;
        request.require_audit_state_preflight = false;
        request.require_delivery_idempotency = false;
        request.require_rate_limit_controls = false;
        request.require_outage_backoff_controls = false;
        request.require_payload_redaction = false;

        let error = review_platform_adapter_controls(&request)
            .expect_err("platform adapter must require future delivery preconditions");
        for expected in [
            "COMMUNICATION_PLATFORM_ADAPTER_DELIVERY_KILL_SWITCH_REQUIRED",
            "COMMUNICATION_PLATFORM_ADAPTER_AUDIT_STATE_PREFLIGHT_REQUIRED",
            "COMMUNICATION_PLATFORM_ADAPTER_IDEMPOTENCY_REQUIRED",
            "COMMUNICATION_PLATFORM_ADAPTER_RATE_LIMIT_CONTROLS_REQUIRED",
            "COMMUNICATION_PLATFORM_ADAPTER_OUTAGE_BACKOFF_REQUIRED",
            "COMMUNICATION_PLATFORM_ADAPTER_PAYLOAD_REDACTION_REQUIRED",
        ] {
            assert!(
                error
                    .violations()
                    .iter()
                    .any(|violation| violation.code() == expected),
                "missing expected violation {expected}"
            );
        }
    }

    #[test]
    fn platform_adapter_review_fails_closed_on_token_material_or_delivery_side_effects() {
        let mut request = ready_platform_adapter_review_request();
        request.review_id = "platform-adapter-review-side-effect".to_owned();
        request.token_secret_material_present = true;
        request.outbound_delivery_requested = true;
        request.outbound_network_used = true;
        request.message_delivered = true;

        let error = review_platform_adapter_controls(&request)
            .expect_err("platform adapter side-effect report must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "COMMUNICATION_PLATFORM_ADAPTER_SIDE_EFFECT" }));
    }

    #[test]
    fn platform_adapter_review_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("platform-adapter-review");
        let state_path = temp_state_path("platform-adapter-review");
        let report = review_platform_adapter_controls(&ready_platform_adapter_review_request())
            .expect("ready platform adapter controls should validate locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_platform_adapter_review_audit(&mut journal, &report, 1_700_000_000_261)
                .expect("platform adapter audit writes");
        let checkpoint =
            persist_platform_adapter_review_checkpoint(&mut store, &report, 1_700_000_000_262)
                .expect("platform adapter checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_PLATFORM_ADAPTER_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("platform adapter checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: PlatformAdapterReviewReport =
            serde_json::from_str(&recovered.value).expect("platform adapter report parses");
        assert_eq!(
            recovered_report.status,
            PlatformAdapterReviewStatus::ReadyForLocalReview
        );
        assert!(recovered_report.token_reference_present);
        assert!(!recovered_report.token_secret_material_present);
        assert!(recovered_report.channel_permission_granted);
        assert!(recovered_report.command_injection_blocked);
        assert!(recovered_report.require_delivery_kill_switch);
        assert!(recovered_report.require_audit_state_preflight);
        assert!(recovered_report.require_delivery_idempotency);
        assert!(recovered_report.require_rate_limit_controls);
        assert!(recovered_report.require_outage_backoff_controls);
        assert!(recovered_report.require_payload_redaction);
        assert!(!recovered_report.token_revoked);
        assert!(!recovered_report.outbound_network_used);
        assert!(!recovered_report.message_delivered);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
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
            channel_safety: Vec::new(),
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
            channel_safety: Vec::new(),
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

    #[test]
    fn notification_boundary_blocks_local_rate_limited_channel_without_network() {
        let request = NotificationPublishRequest {
            id: "publish-request-rate-limited".to_owned(),
            notification: OperatorNotification {
                id: "notification-rate-limited".to_owned(),
                severity: NotificationSeverity::Warning,
                title: "Local rate limit".to_owned(),
                body: "Notification stayed local because the channel limit was reached".to_owned(),
                channels: vec!["cli".to_owned()],
                created_at_unix_ms: 2_000,
            },
            config: CommunicationBoundaryConfig::default(),
            channel_safety: vec![NotificationChannelSafetyState {
                channel_id: "cli".to_owned(),
                messages_sent_in_window: 5,
                max_messages_per_window: 5,
                window_started_at_unix_ms: 1_000,
                window_ends_at_unix_ms: 3_000,
                outage_active: false,
                outage_reason: String::new(),
            }],
            now_unix_ms: 2_001,
        };

        let record = DeterministicNotificationBoundary::new()
            .publish(&request)
            .expect("rate-limited notification should produce a blocked local record");

        assert_eq!(
            record.status,
            NotificationDispatchStatus::BlockedRateLimited
        );
        assert!(!record.outbound_network_used);
        assert_eq!(record.channels.len(), 1);
        assert_eq!(
            record.channels[0].status,
            NotificationChannelDispatchStatus::RateLimited
        );
        assert!(record.channels[0].rate_limited);
        assert!(!record.channels[0].outage_blocked);
        assert!(!record.channels[0].outbound_network_used);
    }

    #[test]
    fn notification_boundary_blocks_local_outage_without_network() {
        let request = NotificationPublishRequest {
            id: "publish-request-outage".to_owned(),
            notification: OperatorNotification {
                id: "notification-outage".to_owned(),
                severity: NotificationSeverity::Critical,
                title: "Local channel outage".to_owned(),
                body: "Notification stayed local because the channel was unavailable".to_owned(),
                channels: vec!["cli".to_owned()],
                created_at_unix_ms: 2_000,
            },
            config: CommunicationBoundaryConfig::default(),
            channel_safety: vec![NotificationChannelSafetyState {
                channel_id: "cli".to_owned(),
                messages_sent_in_window: 0,
                max_messages_per_window: 5,
                window_started_at_unix_ms: 1_000,
                window_ends_at_unix_ms: 3_000,
                outage_active: true,
                outage_reason: "operator marked local channel unavailable".to_owned(),
            }],
            now_unix_ms: 2_001,
        };

        let record = DeterministicNotificationBoundary::new()
            .publish(&request)
            .expect("outage notification should produce a blocked local record");

        assert_eq!(
            record.status,
            NotificationDispatchStatus::BlockedChannelOutage
        );
        assert!(!record.outbound_network_used);
        assert_eq!(record.channels.len(), 1);
        assert_eq!(
            record.channels[0].status,
            NotificationChannelDispatchStatus::ChannelOutage
        );
        assert!(!record.channels[0].rate_limited);
        assert!(record.channels[0].outage_blocked);
        assert!(!record.channels[0].outbound_network_used);
    }

    #[test]
    fn command_route_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("communications-command-route");
        let state_path = temp_state_path("communications-command-route");
        let command = parse_cli_command(&["status".to_owned()], 1_700_000_000_000)
            .expect("status command parses");
        let request = OperatorCommandRoutingRequest {
            id: "route-request-audit-state".to_owned(),
            command,
            config: CommunicationBoundaryConfig::default(),
        };
        let route = DeterministicOperatorCommandRouter::new()
            .route(&request)
            .expect("command routes locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_routed_operator_command_audit(&mut journal, &route, 1_700_000_000_001)
                .expect("command route audit writes");
        let checkpoint =
            persist_routed_operator_command_checkpoint(&mut store, &route, 1_700_000_000_002)
                .expect("command route checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY
        );
        assert!(!route.execution_enabled);
        assert!(!route.outbound_network_used);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_COMMAND_ROUTE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("command checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"outbound_network_used\":false"));
        assert!(recovered.value.contains("\"execution_enabled\":false"));
        assert!(recovered.value.contains("\"operator_authorized\":true"));
        assert!(recovered
            .value
            .contains("\"authorization_status\":\"authorized-local-cli\""));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn notification_dispatch_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("communications-notification-dispatch");
        let state_path = temp_state_path("communications-notification-dispatch");
        let runtime_config = CommunicationConfig {
            cli_enabled: true,
            notify_channels: vec!["cli".to_owned(), "email:ops-alerts".to_owned()],
        };
        let request = NotificationPublishRequest {
            id: "publish-request-audit-state".to_owned(),
            notification: OperatorNotification {
                id: "notification-audit-state".to_owned(),
                severity: NotificationSeverity::Warning,
                title: "Local runtime notice".to_owned(),
                body: "Command boundary remained local and no outbound delivery occurred"
                    .to_owned(),
                channels: Vec::new(),
                created_at_unix_ms: 1_700_000_000_100,
            },
            config: CommunicationBoundaryConfig::from_config(&runtime_config),
            channel_safety: Vec::new(),
            now_unix_ms: 1_700_000_000_101,
        };
        let dispatch = DeterministicNotificationBoundary::new()
            .publish(&request)
            .expect("notification dispatch records locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_notification_dispatch_audit(&mut journal, &dispatch, 1_700_000_000_102)
                .expect("notification dispatch audit writes");
        let checkpoint =
            persist_notification_dispatch_checkpoint(&mut store, &dispatch, 1_700_000_000_103)
                .expect("notification dispatch checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY
        );
        assert!(!dispatch.outbound_network_used);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("notification checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"outbound_network_used\":false"));
        assert!(recovered.value.contains("notification-audit-state"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn notification_rate_limit_record_audits_and_reopens_locally() {
        let audit_path = temp_audit_path("communications-notification-rate-limit");
        let state_path = temp_state_path("communications-notification-rate-limit");
        let request = NotificationPublishRequest {
            id: "publish-request-rate-limit-audit-state".to_owned(),
            notification: OperatorNotification {
                id: "notification-rate-limit-audit-state".to_owned(),
                severity: NotificationSeverity::Warning,
                title: "Local notification rate limit".to_owned(),
                body: "Rate limit blocked dispatch before any outbound delivery".to_owned(),
                channels: vec!["cli".to_owned()],
                created_at_unix_ms: 1_700_000_000_200,
            },
            config: CommunicationBoundaryConfig::default(),
            channel_safety: vec![NotificationChannelSafetyState {
                channel_id: "cli".to_owned(),
                messages_sent_in_window: 2,
                max_messages_per_window: 2,
                window_started_at_unix_ms: 1_700_000_000_000,
                window_ends_at_unix_ms: 1_700_000_060_000,
                outage_active: false,
                outage_reason: String::new(),
            }],
            now_unix_ms: 1_700_000_000_201,
        };
        let dispatch = DeterministicNotificationBoundary::new()
            .publish(&request)
            .expect("rate-limit dispatch records locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_notification_dispatch_audit(&mut journal, &dispatch, 1_700_000_000_202)
                .expect("rate-limit dispatch audit writes");
        let checkpoint =
            persist_notification_dispatch_checkpoint(&mut store, &dispatch, 1_700_000_000_203)
                .expect("rate-limit dispatch checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            dispatch.status,
            NotificationDispatchStatus::BlockedRateLimited
        );
        assert!(!dispatch.outbound_network_used);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(COMMUNICATIONS_LAST_NOTIFICATION_DISPATCH_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("notification checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"status\":\"blocked-rate-limited\""));
        assert!(recovered.value.contains("\"rate_limited\":true"));
        assert!(recovered.value.contains("\"outbound_network_used\":false"));

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

    fn cleanup_state_files(path: &PathBuf) {
        let _ = fs::remove_file(path);
        for suffix in ["-wal", "-shm"] {
            let related = format!("{}{}", path.display(), suffix);
            let _ = fs::remove_file(related);
        }
    }
}
