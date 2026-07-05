#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, RuntimeMode,
    StateCheckpoint, StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    error::Error,
    fmt::{self, Write as FmtWrite},
    io::{Read, Write as IoWrite},
    net::{IpAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

/// Stable embedded dashboard boundary version for audit, replay, and handoff surfaces.
pub const DASHBOARD_BOUNDARY_VERSION: &str = "phase-13-dashboard-boundary-v1";

/// State-store subsystem name for local dashboard checkpoints.
pub const DASHBOARD_STATE_SUBSYSTEM: &str = "dashboard";

/// State-store key for the latest local dashboard render record.
pub const DASHBOARD_LAST_RENDER_CHECKPOINT_KEY: &str = "dashboard:last-render";

/// State-store key for the latest hosted-dashboard security review.
pub const DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY: &str =
    "dashboard:last-hosted-security-review";

/// State-store key for the latest hosted-dashboard request preflight.
pub const DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY: &str =
    "dashboard:last-hosted-request-preflight";

/// State-store key for the latest local one-shot hosted dashboard request validation.
pub const DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY: &str =
    "dashboard:last-hosted-request-validation";

/// State-store key for the latest local hosted dashboard session validation summary.
pub const DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY: &str =
    "dashboard:last-hosted-session-validation";

/// State-store key for the latest local hosted dashboard session lifecycle validation.
pub const DASHBOARD_LAST_HOSTED_SESSION_LIFECYCLE_CHECKPOINT_KEY: &str =
    "dashboard:last-hosted-session-lifecycle";

/// State-store key for the latest bounded loopback dashboard runtime probe.
pub const DASHBOARD_LAST_LOOPBACK_RUNTIME_PROBE_CHECKPOINT_KEY: &str =
    "dashboard:last-loopback-runtime-probe";

/// Stable local hosted-dashboard runtime readiness review version.
pub const DASHBOARD_HOSTED_RUNTIME_READINESS_REVIEW_VERSION: &str =
    "local-hosted-dashboard-runtime-readiness-review-v1";

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
    /// Whether dashboard rendering requires a local access authorization decision.
    pub require_local_access_authorization: bool,
    /// Whether hosted/browser dashboard sessions are enabled. Phase 13 requires false.
    pub hosted_sessions_enabled: bool,
}

impl Default for DashboardBoundaryConfig {
    fn default() -> Self {
        Self {
            local_rendering_enabled: true,
            server_binding: DashboardServerBinding::default(),
            max_panel_items: 100,
            allow_live_controls: false,
            allow_secret_rendering: false,
            require_local_access_authorization: true,
            hosted_sessions_enabled: false,
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

        if !self.require_local_access_authorization {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOCAL_ACCESS_AUTH_REQUIRED",
                "Phase 13 dashboard rendering must require a local access authorization decision",
            ));
        }

        if self.hosted_sessions_enabled {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SESSIONS_DENIED_IN_PHASE_13",
                "Phase 13 dashboard boundaries must not enable hosted browser sessions",
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

/// Local hosted-dashboard security policy review input.
///
/// This is a non-secret, side-effect-free review record for future hosted
/// dashboard work. It does not start a server, bind sockets, authenticate a
/// browser, issue CSRF tokens, or expose a dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedSecurityPolicy {
    /// Stable local review id.
    pub review_id: String,
    /// Whether hosted access must require authentication.
    pub authentication_required: bool,
    /// Whether hosted access must require authorization.
    pub authorization_required: bool,
    /// Whether CSRF protection must be required for hosted state-changing requests.
    pub csrf_protection_required: bool,
    /// Whether CSRF tokens must be rotated or scoped.
    pub csrf_token_rotation_required: bool,
    /// Whether secure response headers must be required.
    pub secure_headers_required: bool,
    /// Whether clickjacking protection must be required.
    pub clickjacking_protection_required: bool,
    /// Whether request rate limiting must be required.
    pub rate_limit_required: bool,
    /// Maximum hosted requests per minute allowed by the local review policy.
    pub max_requests_per_minute: u32,
    /// Whether future hosting must remain loopback-only unless separately approved.
    pub loopback_only_required: bool,
    /// Whether future hosted startup requires audit/state preflight.
    pub audit_state_preflight_required: bool,
    /// Whether future hosted sessions require revocation/logout controls.
    pub session_revocation_required: bool,
    /// Whether future hosted access requires operator role review.
    pub operator_role_review_required: bool,
    /// Whether future hosted controls must remain read-only until separately approved.
    pub read_only_controls_required: bool,
    /// Whether the reviewed policy requested public exposure. Must be false here.
    pub public_exposure_requested: bool,
    /// Whether the reviewed policy requested server startup. Must be false here.
    pub server_start_requested: bool,
    /// Whether the reviewed policy requested live controls. Must be false here.
    pub live_controls_requested: bool,
}

/// Local hosted-dashboard security review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardHostedSecurityReviewStatus {
    /// Required controls are represented locally, but hosting is still not approved.
    ReadyForLocalReview,
    /// Required controls are missing or unsafe hosting flags were requested.
    BlockedMissingControls,
}

/// Local hosted-dashboard security review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedSecurityReviewReport {
    /// Boundary version that produced this report.
    pub dashboard_boundary_version: String,
    /// Stable local review id.
    pub review_id: String,
    /// Review status.
    pub status: DashboardHostedSecurityReviewStatus,
    /// Whether hosted authentication is required by the policy.
    pub authentication_required: bool,
    /// Whether hosted authorization is required by the policy.
    pub authorization_required: bool,
    /// Whether CSRF protection is required by the policy.
    pub csrf_protection_required: bool,
    /// Whether CSRF token rotation/scoping is required by the policy.
    pub csrf_token_rotation_required: bool,
    /// Whether secure response headers are required by the policy.
    pub secure_headers_required: bool,
    /// Whether clickjacking protection is required by the policy.
    pub clickjacking_protection_required: bool,
    /// Whether rate limiting is required by the policy.
    pub rate_limit_required: bool,
    /// Maximum hosted requests per minute represented by this local review.
    pub max_requests_per_minute: u32,
    /// Whether loopback-only hosting is required by the policy.
    pub loopback_only_required: bool,
    /// Whether future hosted startup requires audit/state preflight.
    pub audit_state_preflight_required: bool,
    /// Whether future hosted sessions require revocation/logout controls.
    pub session_revocation_required: bool,
    /// Whether future hosted access requires operator role review.
    pub operator_role_review_required: bool,
    /// Whether future hosted controls must remain read-only until separately approved.
    pub read_only_controls_required: bool,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a server was started. Always false for this review.
    pub server_started: bool,
    /// Whether public network exposure occurred. Always false for this review.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Always false for this review.
    pub live_controls_enabled: bool,
    /// Whether this report approves hosted dashboard production readiness. Always false here.
    pub production_ready: bool,
}

/// Local hosted-dashboard request method model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardHostedRequestMethod {
    /// Read-only request.
    Get,
    /// State-changing request.
    Post,
    /// State-changing request.
    Put,
    /// State-changing request.
    Delete,
}

impl DashboardHostedRequestMethod {
    const fn is_state_changing(self) -> bool {
        matches!(self, Self::Post | Self::Put | Self::Delete)
    }

    const fn as_http_method(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

/// Local hosted-dashboard request preflight input.
///
/// This models future hosted request controls without starting a server, binding
/// sockets, authenticating a browser, serving tokens, or exposing a dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedRequestPreflight {
    /// Stable local preflight id.
    pub preflight_id: String,
    /// Intended future bind host for the reviewed request path.
    pub bind_host: String,
    /// Source of the reviewed access request.
    pub access_source: DashboardAccessSource,
    /// Request method represented by this local preflight.
    pub method: DashboardHostedRequestMethod,
    /// Whether the caller was represented as authenticated.
    pub authenticated: bool,
    /// Whether the caller was represented as authorized for dashboard access.
    pub authorized: bool,
    /// Whether a CSRF token was represented on the request.
    pub csrf_token_present: bool,
    /// Whether the represented CSRF token passed validation.
    pub csrf_token_valid: bool,
    /// Whether a content security policy header would be served.
    pub content_security_policy_present: bool,
    /// Whether frame-ancestor or X-Frame-Options protection would be served.
    pub frame_protection_present: bool,
    /// Whether X-Content-Type-Options protection would be served.
    pub content_type_options_present: bool,
    /// Whether Referrer-Policy protection would be served.
    pub referrer_policy_present: bool,
    /// Number of represented requests in the current local rate window.
    pub requests_in_current_window: u32,
    /// Maximum represented requests allowed in the current local rate window.
    pub max_requests_per_minute: u32,
    /// Whether public exposure was requested. Must remain false here.
    pub public_exposure_requested: bool,
    /// Whether server startup was requested. Must remain false here.
    pub server_start_requested: bool,
    /// Whether live controls were requested. Must remain false here.
    pub live_controls_requested: bool,
}

/// Local hosted-dashboard request preflight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardHostedRequestPreflightStatus {
    /// Request controls are locally coherent, but hosting remains unapproved.
    ReadyForLocalReview,
    /// Required controls are missing or unsafe hosting flags were requested.
    BlockedMissingControls,
}

/// Local hosted-dashboard request preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedRequestPreflightReport {
    /// Boundary version that produced this report.
    pub dashboard_boundary_version: String,
    /// Stable local preflight id.
    pub preflight_id: String,
    /// Preflight status.
    pub status: DashboardHostedRequestPreflightStatus,
    /// Whether the represented bind host is loopback-only.
    pub loopback_bind_validated: bool,
    /// Source of the reviewed access request.
    pub access_source: DashboardAccessSource,
    /// Request method represented by this local preflight.
    pub method: DashboardHostedRequestMethod,
    /// Whether this request method is state-changing.
    pub state_changing_request: bool,
    /// Whether hosted authentication was represented as present.
    pub authenticated: bool,
    /// Whether hosted authorization was represented as present.
    pub authorized: bool,
    /// Whether CSRF protection passed for this request shape.
    pub csrf_validated: bool,
    /// Whether secure response headers were represented as complete.
    pub secure_headers_validated: bool,
    /// Whether the represented request remains within local rate limits.
    pub rate_limit_validated: bool,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a server was started. Always false for this preflight.
    pub server_started: bool,
    /// Whether public network exposure occurred. Always false for this preflight.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Always false for this preflight.
    pub live_controls_enabled: bool,
    /// Whether this report approves hosted dashboard production readiness. Always false here.
    pub production_ready: bool,
}

/// Local one-shot hosted dashboard request validation input.
///
/// This briefly hosts a loopback-only listener, serves one authenticated local
/// dashboard response, then closes the listener. It does not expose a public
/// interface, enable live controls, authenticate real users, persist sessions,
/// or approve production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedRequestValidation {
    /// Stable local validation id.
    pub validation_id: String,
    /// Local render record whose sanitized content is represented by the response.
    pub render_record: DashboardRenderRecord,
    /// Loopback host to bind.
    pub bind_host: String,
    /// Requested port. Use 0 for an ephemeral local port.
    pub requested_port: u16,
    /// Request method sent to the local one-shot handler.
    pub method: DashboardHostedRequestMethod,
    /// Request path sent to the local one-shot handler.
    pub request_path: String,
    /// Whether the caller is represented as authenticated.
    pub authenticated: bool,
    /// Whether the caller is represented as authorized.
    pub authorized: bool,
    /// Whether a CSRF token was present.
    pub csrf_token_present: bool,
    /// Whether the CSRF token was valid.
    pub csrf_token_valid: bool,
    /// Whether secure response headers are required and served.
    pub secure_headers_required: bool,
    /// Number of represented requests in the current local rate window.
    pub requests_in_current_window: u32,
    /// Maximum represented requests allowed in the current local rate window.
    pub max_requests_per_minute: u32,
    /// Whether public exposure was requested. Must remain false.
    pub public_exposure_requested: bool,
    /// Whether live controls were requested. Must remain false.
    pub live_controls_requested: bool,
}

/// Local one-shot hosted dashboard request validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardHostedRequestValidationStatus {
    /// One local authenticated loopback dashboard request was served and closed.
    ReadyForLocalReview,
    /// The request was blocked before serving or the one-shot exchange failed.
    BlockedMissingControls,
}

/// Local one-shot hosted dashboard request validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedRequestValidationReport {
    /// Boundary version that produced this report.
    pub dashboard_boundary_version: String,
    /// Stable local validation id.
    pub validation_id: String,
    /// Validation status.
    pub status: DashboardHostedRequestValidationStatus,
    /// Source dashboard render id.
    pub render_id: String,
    /// Loopback host that was bound.
    pub bind_host: String,
    /// Requested port.
    pub requested_port: u16,
    /// Actual local port assigned by the operating system.
    pub bound_port: Option<u16>,
    /// Request method sent to the local one-shot handler.
    pub method: DashboardHostedRequestMethod,
    /// Request path sent to the local one-shot handler.
    pub request_path: String,
    /// Whether the bind host was loopback-only.
    pub loopback_bind_validated: bool,
    /// Whether hosted authentication was represented as present.
    pub authenticated: bool,
    /// Whether hosted authorization was represented as present.
    pub authorized: bool,
    /// Whether CSRF protection passed for this request shape.
    pub csrf_validated: bool,
    /// Whether secure response headers were served.
    pub secure_headers_validated: bool,
    /// Whether the represented request remains within local rate limits.
    pub rate_limit_validated: bool,
    /// Local HTTP status served by the one-shot handler.
    pub local_http_status_code: u16,
    /// Number of sanitized dashboard panels represented in the response.
    pub response_panel_count: u64,
    /// Bytes in the sanitized dashboard body served by the one-shot handler.
    pub response_body_bytes: u64,
    /// SHA-256 digest of the sanitized dashboard body served by the one-shot handler.
    pub response_body_sha256: String,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether a local one-shot dashboard server was opened.
    pub local_server_started: bool,
    /// Whether exactly one local socket request was served.
    pub network_request_served: bool,
    /// Whether public network exposure occurred. Always false for ready reports.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Always false.
    pub live_controls_enabled: bool,
    /// Whether this report approves dashboard production readiness. Always false.
    pub production_ready: bool,
    /// Sanitized local validation warnings.
    pub warnings: Vec<String>,
}

/// Local hosted dashboard session validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardHostedSessionValidationStatus {
    /// Local session controls accepted one loopback request and rejected unsafe shapes.
    ReadyForLocalReview,
    /// Required local session controls were not proven.
    BlockedMissingControls,
}

/// Local hosted dashboard session validation summary.
///
/// This summarizes multiple local request validation reports so auth, CSRF,
/// and rate-limit denial paths can be checked without creating a public or
/// persistent hosted dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedSessionValidationReport {
    /// Boundary version that produced this report.
    pub dashboard_boundary_version: String,
    /// Stable local session validation id.
    pub session_id: String,
    /// Validation status.
    pub status: DashboardHostedSessionValidationStatus,
    /// Request validation ids included in this session summary.
    pub validation_ids: Vec<String>,
    /// Number of request validation reports summarized.
    pub total_request_count: u32,
    /// Number of accepted authenticated loopback requests.
    pub accepted_request_count: u32,
    /// Number of unauthenticated requests rejected locally.
    pub rejected_unauthenticated_count: u32,
    /// Number of CSRF-invalid requests rejected locally.
    pub rejected_csrf_count: u32,
    /// Number of rate-limited requests rejected locally.
    pub rejected_rate_limited_count: u32,
    /// Whether all summarized request reports validated loopback binding.
    pub loopback_bind_validated: bool,
    /// Whether all summarized request reports required secure headers.
    pub secure_headers_validated: bool,
    /// Whether a local one-shot server was started for at least one accepted request.
    pub local_server_started: bool,
    /// Whether a local loopback network request was served for at least one accepted request.
    pub network_request_served: bool,
    /// Number of missing-control findings across all request reports.
    pub missing_control_count: u32,
    /// Whether public network exposure occurred. Always false here.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Always false here.
    pub live_controls_enabled: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

/// Local hosted dashboard session lifecycle validation input.
///
/// This records non-secret session and CSRF reference lifecycle facts for future
/// hosted dashboard work. It does not create browser sessions, store cookies,
/// retain CSRF token material, start servers, expose networks, or enable live controls.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedSessionLifecycleValidation {
    /// Stable local lifecycle validation id.
    pub lifecycle_id: String,
    /// Non-secret session reference id, never a cookie or bearer token.
    pub session_reference: String,
    /// Non-secret CSRF reference id, never token material.
    pub csrf_reference: String,
    /// Sanitized operator role label.
    pub operator_role: String,
    /// Whether the session reference was authenticated by local metadata.
    pub authenticated: bool,
    /// Whether the operator role was authorized by local policy metadata.
    pub authorized: bool,
    /// Whether a CSRF reference was issued.
    pub csrf_reference_issued: bool,
    /// Whether the CSRF reference was scoped to the session/request class.
    pub csrf_reference_scoped: bool,
    /// Whether CSRF rotation is represented.
    pub csrf_reference_rotated: bool,
    /// Whether session revocation/logout is represented.
    pub session_revocation_supported: bool,
    /// Whether the session was revoked.
    pub session_revoked: bool,
    /// Whether the role is read-only for current dashboard controls.
    pub read_only_role: bool,
    /// Remaining requests in the local rate-limit bucket.
    pub rate_limit_remaining: u32,
    /// Maximum requests per minute for this local lifecycle record.
    pub max_requests_per_minute: u32,
    /// Whether the lifecycle remains loopback-only.
    pub loopback_only: bool,
    /// Whether public network exposure occurred. Must remain false.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Must remain false.
    pub live_controls_enabled: bool,
    /// Whether secret/session/token material was present. Must remain false.
    pub secret_material_present: bool,
    /// Whether a persistent server was started. Must remain false.
    pub persistent_server_started: bool,
    /// Whether this lifecycle claims production readiness. Must remain false.
    pub production_ready_claimed: bool,
    /// Local validation timestamp.
    pub validated_at_unix_ms: u64,
}

/// Local hosted dashboard session lifecycle validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardHostedSessionLifecycleValidationStatus {
    /// Local session lifecycle references are coherent for local review only.
    ReadyForLocalReview,
    /// Required lifecycle controls are missing or unsafe side effects occurred.
    BlockedMissingControls,
}

/// Local hosted dashboard session lifecycle validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedSessionLifecycleValidationReport {
    /// Boundary version that produced this report.
    pub dashboard_boundary_version: String,
    /// Stable local lifecycle validation id.
    pub lifecycle_id: String,
    /// Validation status.
    pub status: DashboardHostedSessionLifecycleValidationStatus,
    /// Whether the session reference is non-secret and present.
    pub session_reference_recorded: bool,
    /// Whether the CSRF reference is non-secret and present.
    pub csrf_reference_recorded: bool,
    /// Sanitized operator role label.
    pub operator_role: String,
    /// Whether the session reference was authenticated by local metadata.
    pub authenticated: bool,
    /// Whether the operator role was authorized by local policy metadata.
    pub authorized: bool,
    /// Whether the CSRF lifecycle is represented.
    pub csrf_lifecycle_validated: bool,
    /// Whether revocation/logout controls are represented.
    pub session_revocation_supported: bool,
    /// Whether the session was revoked.
    pub session_revoked: bool,
    /// Whether the role is read-only for current dashboard controls.
    pub read_only_role: bool,
    /// Whether rate-limit metadata is usable.
    pub rate_limit_validated: bool,
    /// Whether the lifecycle remains loopback-only.
    pub loopback_only: bool,
    /// Number of missing or unsafe control findings.
    pub missing_control_count: u32,
    /// Whether public network exposure occurred. Always false for ready reports.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Always false.
    pub live_controls_enabled: bool,
    /// Whether secret/session/token material was present. Always false.
    pub secret_material_present: bool,
    /// Whether a persistent server was started. Always false.
    pub persistent_server_started: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Stable non-secret violation codes.
    pub violation_codes: Vec<String>,
}

/// Bounded loopback dashboard runtime probe request.
///
/// This exercises a single local loopback listener for multiple read-only
/// requests, then shuts it down. It is not public hosting and does not enable
/// live controls.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardLoopbackRuntimeProbe {
    /// Stable local runtime probe id.
    pub probe_id: String,
    /// Rendered dashboard record to serve.
    pub render_record: DashboardRenderRecord,
    /// Numeric loopback bind host.
    pub bind_host: String,
    /// Requested local port, usually 0 for an ephemeral port.
    pub requested_port: u16,
    /// Number of read-only loopback requests to serve before shutdown.
    pub request_count: u32,
    /// Whether public exposure was requested. Must remain false here.
    pub public_exposure_requested: bool,
    /// Whether live controls were requested. Must remain false here.
    pub live_controls_requested: bool,
}

/// Local bounded loopback dashboard runtime probe status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardLoopbackRuntimeProbeStatus {
    /// Local bounded loopback runtime probe succeeded.
    ReadyForLocalReview,
    /// Runtime probe was blocked by missing controls or unsafe flags.
    Blocked,
}

/// Local bounded loopback dashboard runtime probe report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardLoopbackRuntimeProbeReport {
    /// Boundary version that produced this report.
    pub dashboard_boundary_version: String,
    /// Stable local runtime probe id.
    pub probe_id: String,
    /// Probe status.
    pub status: DashboardLoopbackRuntimeProbeStatus,
    /// Bind host used for the local listener.
    pub bind_host: String,
    /// Requested local port.
    pub requested_port: u16,
    /// Bound ephemeral/local port.
    pub bound_port: Option<u16>,
    /// Whether the bind host was numeric loopback.
    pub loopback_bind_validated: bool,
    /// Number of requests expected.
    pub expected_request_count: u32,
    /// Number of requests successfully served.
    pub served_request_count: u32,
    /// Whether all served requests returned HTTP 200.
    pub all_requests_returned_ok: bool,
    /// Whether all served responses matched the rendered body digest.
    pub response_digest_consistent: bool,
    /// Number of missing-control findings.
    pub missing_control_count: u32,
    /// Whether the bounded local listener started.
    pub bounded_runtime_started: bool,
    /// Whether the bounded local listener shut down after the expected requests.
    pub bounded_runtime_shutdown: bool,
    /// Whether public network exposure occurred. Always false here.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled. Always false here.
    pub live_controls_enabled: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
    /// Stable non-secret warnings.
    pub warnings: Vec<String>,
}

/// Local hosted-dashboard runtime readiness review request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedRuntimeReadinessReviewRequest {
    /// Stable non-secret review identifier.
    pub review_id: String,
    /// Future-hosting security control review.
    pub security_review: DashboardHostedSecurityReviewReport,
    /// Future hosted-request preflight report.
    pub request_preflight: DashboardHostedRequestPreflightReport,
    /// Local one-shot hosted-session validation report.
    pub session_validation: DashboardHostedSessionValidationReport,
    /// Remaining non-secret external/deployment evidence references.
    pub remaining_external_evidence: Vec<String>,
    /// Whether this review requested persistent server startup.
    pub persistent_server_start_requested: bool,
    /// Whether this review requested public network exposure.
    pub public_network_exposure_requested: bool,
    /// Whether this review requested live dashboard controls.
    pub live_controls_requested: bool,
    /// Whether this review claims production readiness.
    pub production_ready_claimed: bool,
}

/// Local hosted-dashboard runtime readiness review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DashboardHostedRuntimeReadinessReviewStatus {
    /// Existing local controls are coherent for local review only.
    ReadyForLocalReview,
    /// Local evidence is incomplete or unsafe side effects were requested.
    Blocked,
}

/// Local hosted-dashboard runtime readiness review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardHostedRuntimeReadinessReviewReport {
    /// Stable review schema version.
    pub review_version: String,
    /// Stable non-secret review identifier.
    pub review_id: String,
    /// Review status.
    pub status: DashboardHostedRuntimeReadinessReviewStatus,
    /// Whether security controls are ready for local review.
    pub security_review_ready: bool,
    /// Whether hosted-request preflight is ready for local review.
    pub request_preflight_ready: bool,
    /// Whether local hosted-session validation is ready for local review.
    pub session_validation_ready: bool,
    /// Whether at least one accepted local request was validated.
    pub accepted_request_validated: bool,
    /// Whether rejected unauthenticated request accounting was validated.
    pub unauthenticated_rejection_validated: bool,
    /// Whether rejected CSRF request accounting was validated.
    pub csrf_rejection_validated: bool,
    /// Whether rejected rate-limit request accounting was validated.
    pub rate_limit_rejection_validated: bool,
    /// Whether local one-shot loopback serving was validated.
    pub loopback_serving_validated: bool,
    /// Whether secure-header checks were validated.
    pub secure_headers_validated: bool,
    /// Whether remaining external/deployment evidence is recorded.
    pub remaining_external_evidence_recorded: bool,
    /// Count of remaining external/deployment evidence references.
    pub remaining_external_evidence_count: usize,
    /// Count of local missing controls across component reports.
    pub missing_control_count: u32,
    /// Whether a persistent dashboard server was started.
    pub persistent_server_started: bool,
    /// Whether public network exposure occurred.
    pub public_network_exposed: bool,
    /// Whether live controls were enabled.
    pub live_controls_enabled: bool,
    /// Whether production readiness was claimed.
    pub production_ready: bool,
    /// Stable non-secret violation codes.
    pub violation_codes: Vec<String>,
}

impl DashboardHostedRequestValidation {
    /// Validate local one-shot hosted dashboard request validation input.
    pub fn validate(&self) -> Result<(), DashboardError> {
        self.render_record.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted request validation",
            &self.validation_id,
            &mut violations,
        );
        validate_id(
            "dashboard hosted request path",
            &self.request_path,
            &mut violations,
        );
        if contains_secret_like_text(&self.bind_host) {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_REQUEST_BIND_SECRET_LIKE",
                "hosted dashboard request bind host looks like secret material",
            ));
        }
        if !is_loopback_host(&self.bind_host) {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_REQUEST_BIND_NOT_LOOPBACK",
                "hosted dashboard one-shot validation requires loopback binding",
            ));
        }
        if self.max_requests_per_minute == 0 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_REQUEST_RATE_LIMIT_ZERO",
                "hosted dashboard one-shot validation requires a positive rate limit",
            ));
        }
        if self.public_exposure_requested || self.live_controls_requested {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_REQUEST_SIDE_EFFECT_REQUESTED",
                "hosted dashboard one-shot validation must not request public exposure or live controls",
            ));
        }
        finish_validation(violations)
    }
}

impl DashboardHostedRequestValidationReport {
    /// Validate local one-shot hosted dashboard request validation report.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted request validation",
            &self.validation_id,
            &mut violations,
        );
        validate_id("dashboard render", &self.render_id, &mut violations);
        validate_id(
            "dashboard hosted request path",
            &self.request_path,
            &mut violations,
        );
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        if self.public_network_exposed || self.live_controls_enabled || self.production_ready {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_REQUEST_FORBIDDEN_SIDE_EFFECT",
                "hosted dashboard one-shot validation must not expose public networks, enable live controls, or approve production readiness",
            ));
        }
        match self.status {
            DashboardHostedRequestValidationStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || self.local_http_status_code != 200
                    || self.request_path != "/"
                    || self.bound_port.is_none()
                    || !self.loopback_bind_validated
                    || !self.authenticated
                    || !self.authorized
                    || !self.csrf_validated
                    || !self.secure_headers_validated
                    || !self.rate_limit_validated
                    || !self.local_server_started
                    || !self.network_request_served
                    || self.response_panel_count == 0
                    || self.response_body_bytes == 0
                    || !is_sha256_hex(&self.response_body_sha256)
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_REQUEST_READY_MISMATCH",
                        "ready hosted dashboard one-shot validations require one authenticated loopback request, CSRF/header/rate controls, served sanitized body metadata, and zero missing controls",
                    ));
                }
            }
            DashboardHostedRequestValidationStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 || self.local_http_status_code == 200 {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_REQUEST_BLOCKED_MISMATCH",
                        "blocked hosted dashboard one-shot validations require missing controls and a non-200 local status",
                    ));
                }
            }
        }
        for warning in &self.warnings {
            validate_id("dashboard hosted request warning", warning, &mut violations);
        }
        finish_validation(violations)
    }
}

impl DashboardHostedSessionValidationReport {
    /// Validate local hosted dashboard session-control summary.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted session validation",
            &self.session_id,
            &mut violations,
        );
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        if self.validation_ids.is_empty() {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SESSION_VALIDATIONS_REQUIRED",
                "hosted dashboard session validation requires summarized request validation ids",
            ));
        }
        let mut unique_ids = BTreeSet::new();
        for validation_id in &self.validation_ids {
            validate_id(
                "dashboard hosted session request validation id",
                validation_id,
                &mut violations,
            );
            if !unique_ids.insert(validation_id) {
                violations.push(DashboardViolation::new(
                    "DASHBOARD_HOSTED_SESSION_DUPLICATE_VALIDATION_ID",
                    "hosted dashboard session validation ids must be unique",
                ));
            }
        }
        if self.public_network_exposed || self.live_controls_enabled || self.production_ready {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SESSION_FORBIDDEN_SIDE_EFFECT",
                "hosted dashboard session validation must not expose public networks, enable live controls, or approve production readiness",
            ));
        }
        match self.status {
            DashboardHostedSessionValidationStatus::ReadyForLocalReview => {
                if self.total_request_count < 4
                    || self.accepted_request_count == 0
                    || self.rejected_unauthenticated_count == 0
                    || self.rejected_csrf_count == 0
                    || self.rejected_rate_limited_count == 0
                    || !self.loopback_bind_validated
                    || !self.secure_headers_validated
                    || !self.local_server_started
                    || !self.network_request_served
                    || self.validation_ids.len()
                        != usize::try_from(self.total_request_count).unwrap_or(usize::MAX)
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_SESSION_READY_MISMATCH",
                        "ready hosted dashboard session validation requires accepted loopback traffic plus unauthenticated, CSRF, and rate-limit rejections",
                    ));
                }
            }
            DashboardHostedSessionValidationStatus::BlockedMissingControls => {
                if self.accepted_request_count > 0
                    && self.rejected_unauthenticated_count > 0
                    && self.rejected_csrf_count > 0
                    && self.rejected_rate_limited_count > 0
                    && self.loopback_bind_validated
                    && self.secure_headers_validated
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_SESSION_BLOCKED_MISMATCH",
                        "blocked hosted dashboard session validation must be missing at least one required control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl DashboardHostedSessionLifecycleValidation {
    /// Validate local hosted dashboard session lifecycle input.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted session lifecycle validation",
            &self.lifecycle_id,
            &mut violations,
        );
        validate_reference_id(
            "dashboard hosted session reference",
            &self.session_reference,
            &mut violations,
        );
        validate_reference_id(
            "dashboard hosted CSRF reference",
            &self.csrf_reference,
            &mut violations,
        );
        validate_id(
            "dashboard hosted operator role",
            &self.operator_role,
            &mut violations,
        );
        if self.max_requests_per_minute == 0 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SESSION_LIFECYCLE_RATE_LIMIT_ZERO",
                "hosted dashboard session lifecycle requires a positive rate limit",
            ));
        }
        finish_validation(violations)
    }
}

impl DashboardHostedSessionLifecycleValidationReport {
    /// Validate local hosted dashboard session lifecycle report invariants.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted session lifecycle validation",
            &self.lifecycle_id,
            &mut violations,
        );
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        let forbidden_side_effects = self.public_network_exposed
            || self.live_controls_enabled
            || self.secret_material_present
            || self.persistent_server_started
            || self.production_ready;
        if forbidden_side_effects
            && self.status == DashboardHostedSessionLifecycleValidationStatus::ReadyForLocalReview
        {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SESSION_LIFECYCLE_FORBIDDEN_SIDE_EFFECT",
                "hosted dashboard session lifecycle must not expose networks, enable controls, retain secrets, start servers, or approve production readiness",
            ));
        }
        let ready = self.session_reference_recorded
            && self.csrf_reference_recorded
            && self.authenticated
            && self.authorized
            && self.csrf_lifecycle_validated
            && self.session_revocation_supported
            && !self.session_revoked
            && self.read_only_role
            && self.rate_limit_validated
            && self.loopback_only
            && self.missing_control_count == 0
            && self.violation_codes.is_empty()
            && !self.public_network_exposed
            && !self.live_controls_enabled
            && !self.secret_material_present
            && !self.persistent_server_started
            && !self.production_ready;
        match self.status {
            DashboardHostedSessionLifecycleValidationStatus::ReadyForLocalReview => {
                if !ready {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_READY_MISMATCH",
                        "ready hosted dashboard session lifecycle requires non-secret references, auth, authorization, CSRF lifecycle, revocation, read-only role, rate limit, loopback-only scope, and zero side effects",
                    ));
                }
            }
            DashboardHostedSessionLifecycleValidationStatus::BlockedMissingControls => {
                if ready {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_BLOCKED_MISMATCH",
                        "blocked hosted dashboard session lifecycle must be missing at least one required control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl DashboardLoopbackRuntimeProbe {
    /// Validate bounded local loopback runtime probe input.
    pub fn validate(&self) -> Result<(), DashboardError> {
        self.render_record.validate()?;
        let mut violations = Vec::new();
        validate_id(
            "dashboard loopback runtime probe",
            &self.probe_id,
            &mut violations,
        );
        if !is_loopback_host(&self.bind_host) {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOOPBACK_RUNTIME_BIND_NOT_LOOPBACK",
                "dashboard loopback runtime probe requires numeric loopback binding",
            ));
        }
        if self.request_count == 0 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOOPBACK_RUNTIME_REQUEST_COUNT_ZERO",
                "dashboard loopback runtime probe requires at least one request",
            ));
        }
        if self.public_exposure_requested
            || self.live_controls_requested
            || self.render_record.public_network_exposed
            || self.render_record.live_controls_enabled
            || self.render_record.server_started
        {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOOPBACK_RUNTIME_FORBIDDEN_SIDE_EFFECT_REQUEST",
                "dashboard loopback runtime probe must not request public exposure, live controls, or reuse a side-effectful render record",
            ));
        }
        finish_validation(violations)
    }
}

impl DashboardLoopbackRuntimeProbeReport {
    /// Validate bounded local loopback runtime probe output.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard loopback runtime probe",
            &self.probe_id,
            &mut violations,
        );
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        if !is_loopback_host(&self.bind_host) || !self.loopback_bind_validated {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOOPBACK_RUNTIME_BIND_NOT_VALIDATED",
                "dashboard loopback runtime report requires validated numeric loopback binding",
            ));
        }
        if self.public_network_exposed || self.live_controls_enabled || self.production_ready {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LOOPBACK_RUNTIME_FORBIDDEN_SIDE_EFFECT",
                "dashboard loopback runtime report must not expose public networks, enable live controls, or approve production readiness",
            ));
        }
        for warning in &self.warnings {
            validate_id(
                "dashboard loopback runtime warning",
                warning,
                &mut violations,
            );
        }
        match self.status {
            DashboardLoopbackRuntimeProbeStatus::ReadyForLocalReview => {
                if self.expected_request_count == 0
                    || self.served_request_count != self.expected_request_count
                    || !self.all_requests_returned_ok
                    || !self.response_digest_consistent
                    || self.missing_control_count != 0
                    || !self.bounded_runtime_started
                    || !self.bounded_runtime_shutdown
                    || self.bound_port.is_none()
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_LOOPBACK_RUNTIME_READY_MISMATCH",
                        "ready dashboard loopback runtime report requires all expected local requests to return 200 with consistent response digest and clean shutdown",
                    ));
                }
            }
            DashboardLoopbackRuntimeProbeStatus::Blocked => {
                if self.expected_request_count > 0
                    && self.served_request_count == self.expected_request_count
                    && self.all_requests_returned_ok
                    && self.response_digest_consistent
                    && self.missing_control_count == 0
                    && self.bounded_runtime_started
                    && self.bounded_runtime_shutdown
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_LOOPBACK_RUNTIME_BLOCKED_MISMATCH",
                        "blocked dashboard loopback runtime report must have at least one missing control or failed request",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl DashboardHostedRuntimeReadinessReviewRequest {
    /// Validate the local readiness review request before composing component evidence.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        if self.review_id.trim().is_empty() {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_RUNTIME_REVIEW_ID_EMPTY",
                "hosted dashboard runtime readiness review id must not be empty",
            ));
        }
        self.security_review.validate()?;
        self.request_preflight.validate()?;
        self.session_validation.validate()?;
        finish_validation(violations)
    }
}

impl DashboardHostedRuntimeReadinessReviewReport {
    /// Validate derived readiness review invariants.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        if self.review_version != DASHBOARD_HOSTED_RUNTIME_READINESS_REVIEW_VERSION {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_RUNTIME_REVIEW_VERSION_MISMATCH",
                "hosted dashboard runtime readiness review version mismatch",
            ));
        }
        if self.review_id.trim().is_empty() {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_RUNTIME_REVIEW_REPORT_ID_EMPTY",
                "hosted dashboard runtime readiness review report id must not be empty",
            ));
        }
        if self.status == DashboardHostedRuntimeReadinessReviewStatus::ReadyForLocalReview {
            for (missing, code, message) in [
                (
                    !self.security_review_ready,
                    "DASHBOARD_HOSTED_RUNTIME_SECURITY_NOT_READY",
                    "hosted dashboard security review is not ready",
                ),
                (
                    !self.request_preflight_ready,
                    "DASHBOARD_HOSTED_RUNTIME_PREFLIGHT_NOT_READY",
                    "hosted dashboard request preflight is not ready",
                ),
                (
                    !self.session_validation_ready,
                    "DASHBOARD_HOSTED_RUNTIME_SESSION_NOT_READY",
                    "hosted dashboard session validation is not ready",
                ),
                (
                    !self.accepted_request_validated,
                    "DASHBOARD_HOSTED_RUNTIME_ACCEPTED_REQUEST_MISSING",
                    "hosted dashboard accepted request validation is missing",
                ),
                (
                    !self.unauthenticated_rejection_validated,
                    "DASHBOARD_HOSTED_RUNTIME_UNAUTH_REJECTION_MISSING",
                    "hosted dashboard unauthenticated rejection validation is missing",
                ),
                (
                    !self.csrf_rejection_validated,
                    "DASHBOARD_HOSTED_RUNTIME_CSRF_REJECTION_MISSING",
                    "hosted dashboard CSRF rejection validation is missing",
                ),
                (
                    !self.rate_limit_rejection_validated,
                    "DASHBOARD_HOSTED_RUNTIME_RATE_LIMIT_REJECTION_MISSING",
                    "hosted dashboard rate-limit rejection validation is missing",
                ),
                (
                    !self.loopback_serving_validated,
                    "DASHBOARD_HOSTED_RUNTIME_LOOPBACK_SERVING_MISSING",
                    "hosted dashboard loopback serving validation is missing",
                ),
                (
                    !self.secure_headers_validated,
                    "DASHBOARD_HOSTED_RUNTIME_SECURE_HEADERS_MISSING",
                    "hosted dashboard secure-header validation is missing",
                ),
                (
                    !self.remaining_external_evidence_recorded,
                    "DASHBOARD_HOSTED_RUNTIME_REMAINING_EVIDENCE_MISSING",
                    "remaining hosted dashboard deployment evidence must be recorded",
                ),
            ] {
                if missing {
                    violations.push(DashboardViolation::new(code, message));
                }
            }
            for (unsafe_flag, code, message) in [
                (
                    self.persistent_server_started,
                    "DASHBOARD_HOSTED_RUNTIME_PERSISTENT_SERVER_STARTED",
                    "local hosted dashboard readiness review must not start a persistent server",
                ),
                (
                    self.public_network_exposed,
                    "DASHBOARD_HOSTED_RUNTIME_PUBLIC_NETWORK_EXPOSED",
                    "local hosted dashboard readiness review must not expose public network bindings",
                ),
                (
                    self.live_controls_enabled,
                    "DASHBOARD_HOSTED_RUNTIME_LIVE_CONTROLS_ENABLED",
                    "local hosted dashboard readiness review must not enable live controls",
                ),
                (
                    self.production_ready,
                    "DASHBOARD_HOSTED_RUNTIME_PRODUCTION_READY_CLAIMED",
                    "local hosted dashboard readiness review must not claim production readiness",
                ),
            ] {
                if unsafe_flag {
                    violations.push(DashboardViolation::new(code, message));
                }
            }
        }
        finish_validation(violations)
    }
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

impl DashboardHostedSecurityPolicy {
    /// Validate the local hosted-dashboard security review input.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted security review",
            &self.review_id,
            &mut violations,
        );
        if self.rate_limit_required && self.max_requests_per_minute == 0 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_RATE_LIMIT_ZERO",
                "hosted dashboard rate limit must be positive when required",
            ));
        }
        for (enabled, code, message) in [
            (
                self.audit_state_preflight_required,
                "DASHBOARD_HOSTED_AUDIT_STATE_PREFLIGHT_REQUIRED",
                "future hosted dashboard startup requires audit/state preflight",
            ),
            (
                self.session_revocation_required,
                "DASHBOARD_HOSTED_SESSION_REVOCATION_REQUIRED",
                "future hosted dashboard sessions require revocation/logout controls",
            ),
            (
                self.operator_role_review_required,
                "DASHBOARD_HOSTED_OPERATOR_ROLE_REVIEW_REQUIRED",
                "future hosted dashboard access requires operator role review",
            ),
            (
                self.read_only_controls_required,
                "DASHBOARD_HOSTED_READ_ONLY_CONTROLS_REQUIRED",
                "future hosted dashboard controls must remain read-only until separately approved",
            ),
        ] {
            if !enabled {
                violations.push(DashboardViolation::new(code, message));
            }
        }
        finish_validation(violations)
    }
}

impl DashboardHostedSecurityReviewReport {
    /// Validate hosted-dashboard security review report invariants.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted security review",
            &self.review_id,
            &mut violations,
        );
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        if self.server_started || self.public_network_exposed || self.live_controls_enabled {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SECURITY_REVIEW_SIDE_EFFECT",
                "hosted dashboard security reviews must not start servers, expose public networks, or enable live controls",
            ));
        }
        if self.production_ready {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_SECURITY_REVIEW_PRODUCTION_READY",
                "hosted dashboard security reviews must not approve production readiness",
            ));
        }
        match self.status {
            DashboardHostedSecurityReviewStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.authentication_required
                    || !self.authorization_required
                    || !self.csrf_protection_required
                    || !self.csrf_token_rotation_required
                    || !self.secure_headers_required
                    || !self.clickjacking_protection_required
                    || !self.rate_limit_required
                    || self.max_requests_per_minute == 0
                    || !self.loopback_only_required
                    || !self.audit_state_preflight_required
                    || !self.session_revocation_required
                    || !self.operator_role_review_required
                    || !self.read_only_controls_required
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_SECURITY_REVIEW_READY_MISMATCH",
                        "ready hosted dashboard security reviews require all local controls and zero missing controls",
                    ));
                }
            }
            DashboardHostedSecurityReviewStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_SECURITY_REVIEW_BLOCKED_MISMATCH",
                        "blocked hosted dashboard security reviews require at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl DashboardHostedRequestPreflight {
    /// Validate the local hosted-dashboard request preflight input.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted request preflight",
            &self.preflight_id,
            &mut violations,
        );
        if contains_secret_like_text(&self.bind_host) {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_PREFLIGHT_BIND_SECRET_LIKE",
                "dashboard hosted preflight bind host looks like secret material",
            ));
        }
        if self.max_requests_per_minute == 0 {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_PREFLIGHT_RATE_LIMIT_ZERO",
                "hosted dashboard request preflight rate limit must be positive",
            ));
        }
        finish_validation(violations)
    }
}

impl DashboardHostedRequestPreflightReport {
    /// Validate hosted-dashboard request preflight report invariants.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id(
            "dashboard hosted request preflight",
            &self.preflight_id,
            &mut violations,
        );
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        if self.state_changing_request != self.method.is_state_changing() {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_PREFLIGHT_METHOD_MISMATCH",
                "hosted request preflight state-changing flag must match method",
            ));
        }
        if self.server_started || self.public_network_exposed || self.live_controls_enabled {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_PREFLIGHT_SIDE_EFFECT",
                "hosted dashboard request preflights must not start servers, expose public networks, or enable live controls",
            ));
        }
        if self.production_ready {
            violations.push(DashboardViolation::new(
                "DASHBOARD_HOSTED_PREFLIGHT_PRODUCTION_READY",
                "hosted dashboard request preflights must not approve production readiness",
            ));
        }
        match self.status {
            DashboardHostedRequestPreflightStatus::ReadyForLocalReview => {
                if self.missing_control_count != 0
                    || !self.loopback_bind_validated
                    || self.access_source != DashboardAccessSource::BrowserSession
                    || !self.authenticated
                    || !self.authorized
                    || !self.csrf_validated
                    || !self.secure_headers_validated
                    || !self.rate_limit_validated
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_PREFLIGHT_READY_MISMATCH",
                        "ready hosted dashboard request preflights require loopback binding, browser-session auth, CSRF, secure headers, rate limits, and zero missing controls",
                    ));
                }
            }
            DashboardHostedRequestPreflightStatus::BlockedMissingControls => {
                if self.missing_control_count == 0 {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_HOSTED_PREFLIGHT_BLOCKED_MISMATCH",
                        "blocked hosted dashboard request preflights require at least one missing or unsafe control",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

/// Review future hosted-dashboard security controls without starting a server.
pub fn review_dashboard_hosted_security(
    policy: &DashboardHostedSecurityPolicy,
) -> Result<DashboardHostedSecurityReviewReport, DashboardError> {
    policy.validate()?;
    let mut missing_control_count = 0_u32;
    for missing in [
        !policy.authentication_required,
        !policy.authorization_required,
        !policy.csrf_protection_required,
        !policy.csrf_token_rotation_required,
        !policy.secure_headers_required,
        !policy.clickjacking_protection_required,
        !policy.rate_limit_required,
        policy.max_requests_per_minute == 0,
        !policy.loopback_only_required,
        !policy.audit_state_preflight_required,
        !policy.session_revocation_required,
        !policy.operator_role_review_required,
        !policy.read_only_controls_required,
        policy.public_exposure_requested,
        policy.server_start_requested,
        policy.live_controls_requested,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let status = if missing_control_count == 0 {
        DashboardHostedSecurityReviewStatus::ReadyForLocalReview
    } else {
        DashboardHostedSecurityReviewStatus::BlockedMissingControls
    };
    let report = DashboardHostedSecurityReviewReport {
        dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
        review_id: policy.review_id.clone(),
        status,
        authentication_required: policy.authentication_required,
        authorization_required: policy.authorization_required,
        csrf_protection_required: policy.csrf_protection_required,
        csrf_token_rotation_required: policy.csrf_token_rotation_required,
        secure_headers_required: policy.secure_headers_required,
        clickjacking_protection_required: policy.clickjacking_protection_required,
        rate_limit_required: policy.rate_limit_required,
        max_requests_per_minute: policy.max_requests_per_minute,
        loopback_only_required: policy.loopback_only_required,
        audit_state_preflight_required: policy.audit_state_preflight_required,
        session_revocation_required: policy.session_revocation_required,
        operator_role_review_required: policy.operator_role_review_required,
        read_only_controls_required: policy.read_only_controls_required,
        missing_control_count,
        server_started: false,
        public_network_exposed: false,
        live_controls_enabled: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Preflight a future hosted-dashboard request without starting a server.
pub fn preflight_dashboard_hosted_request(
    request: &DashboardHostedRequestPreflight,
) -> Result<DashboardHostedRequestPreflightReport, DashboardError> {
    request.validate()?;
    let loopback_bind_validated = is_loopback_host(&request.bind_host);
    let browser_session_requested = request.access_source == DashboardAccessSource::BrowserSession;
    let csrf_validated = if request.method.is_state_changing() {
        request.csrf_token_present && request.csrf_token_valid
    } else {
        !request.csrf_token_present || request.csrf_token_valid
    };
    let secure_headers_validated = request.content_security_policy_present
        && request.frame_protection_present
        && request.content_type_options_present
        && request.referrer_policy_present;
    let rate_limit_validated =
        request.requests_in_current_window <= request.max_requests_per_minute;

    let mut missing_control_count = 0_u32;
    for missing in [
        !loopback_bind_validated,
        !browser_session_requested,
        !request.authenticated,
        !request.authorized,
        !csrf_validated,
        !secure_headers_validated,
        !rate_limit_validated,
        request.public_exposure_requested,
        request.server_start_requested,
        request.live_controls_requested,
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
        }
    }
    let status = if missing_control_count == 0 {
        DashboardHostedRequestPreflightStatus::ReadyForLocalReview
    } else {
        DashboardHostedRequestPreflightStatus::BlockedMissingControls
    };
    let report = DashboardHostedRequestPreflightReport {
        dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
        preflight_id: request.preflight_id.clone(),
        status,
        loopback_bind_validated,
        access_source: request.access_source,
        method: request.method,
        state_changing_request: request.method.is_state_changing(),
        authenticated: request.authenticated,
        authorized: request.authorized,
        csrf_validated,
        secure_headers_validated,
        rate_limit_validated,
        missing_control_count,
        server_started: false,
        public_network_exposed: false,
        live_controls_enabled: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Validate one local hosted-dashboard request over a loopback-only one-shot listener.
pub fn validate_dashboard_hosted_request(
    request: DashboardHostedRequestValidation,
) -> Result<DashboardHostedRequestValidationReport, DashboardError> {
    request.validate()?;
    let mut missing_control_count = 0_u32;
    let mut warnings = Vec::new();
    let bind_host = request.bind_host.trim().to_owned();
    let request_path = request.request_path.trim().to_owned();
    let loopback_ip = parse_loopback_ip(&bind_host);
    let loopback_bind_validated = loopback_ip.is_some();
    let csrf_validated = if request.method.is_state_changing() {
        request.csrf_token_present && request.csrf_token_valid
    } else {
        !request.csrf_token_present || request.csrf_token_valid
    };
    let secure_headers_validated = request.secure_headers_required;
    let rate_limit_validated =
        request.requests_in_current_window <= request.max_requests_per_minute;

    for (missing, warning) in [
        (
            !loopback_bind_validated,
            "dashboard bind host is not numeric loopback",
        ),
        (request_path != "/", "dashboard request path is not /"),
        (
            !request.authenticated,
            "dashboard request is not authenticated",
        ),
        (!request.authorized, "dashboard request is not authorized"),
        (!csrf_validated, "dashboard request CSRF validation failed"),
        (
            !secure_headers_validated,
            "dashboard secure headers are not required",
        ),
        (
            !rate_limit_validated,
            "dashboard request exceeds rate limit",
        ),
    ] {
        if missing {
            missing_control_count = missing_control_count.saturating_add(1);
            warnings.push(warning.to_owned());
        }
    }

    let mut bound_port = None;
    let mut local_server_started = false;
    let mut network_request_served = false;
    let mut local_http_status_code = 403_u16;
    let mut response_body_bytes = 0_u64;
    let mut response_body_sha256 = String::new();
    if missing_control_count == 0 {
        if let Some(ip) = loopback_ip {
            let body = render_dashboard_response_body(&request.render_record)?;
            match serve_one_dashboard_request(
                ip,
                request.requested_port,
                request.method,
                &request_path,
                body,
            ) {
                Ok(exchange) => {
                    bound_port = Some(exchange.bound_port);
                    local_server_started = true;
                    network_request_served = exchange.network_request_served;
                    local_http_status_code = exchange.local_http_status_code;
                    response_body_bytes = exchange.response_body_bytes;
                    response_body_sha256 = exchange.response_body_sha256;
                    if local_http_status_code != 200 || !network_request_served {
                        missing_control_count = missing_control_count.saturating_add(1);
                        warnings.push("dashboard one-shot request did not return 200".to_owned());
                    }
                }
                Err(error) => {
                    missing_control_count = missing_control_count.saturating_add(1);
                    warnings.push(format!("dashboard one-shot request failed: {error}"));
                }
            }
        }
    }

    let response_panel_count = u64::try_from(request.render_record.panels.len()).map_err(|_| {
        DashboardError::StateStoreFailed {
            reason: "dashboard panel count overflowed".to_owned(),
        }
    })?;
    let report = DashboardHostedRequestValidationReport {
        dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
        validation_id: request.validation_id,
        status: if missing_control_count == 0 {
            DashboardHostedRequestValidationStatus::ReadyForLocalReview
        } else {
            DashboardHostedRequestValidationStatus::BlockedMissingControls
        },
        render_id: request.render_record.snapshot_id,
        bind_host,
        requested_port: request.requested_port,
        bound_port,
        method: request.method,
        request_path,
        loopback_bind_validated,
        authenticated: request.authenticated,
        authorized: request.authorized,
        csrf_validated,
        secure_headers_validated,
        rate_limit_validated,
        local_http_status_code,
        response_panel_count,
        response_body_bytes,
        response_body_sha256,
        missing_control_count,
        local_server_started,
        network_request_served,
        public_network_exposed: false,
        live_controls_enabled: false,
        production_ready: false,
        warnings,
    };
    report.validate()?;
    Ok(report)
}

/// Summarize multiple local hosted-dashboard request validations into a
/// session-control report without starting a persistent server.
pub fn validate_dashboard_hosted_session(
    session_id: impl Into<String>,
    reports: &[DashboardHostedRequestValidationReport],
) -> Result<DashboardHostedSessionValidationReport, DashboardError> {
    let session_id = session_id.into();
    let mut validation_ids = Vec::with_capacity(reports.len());
    let mut accepted_request_count = 0_u32;
    let mut rejected_unauthenticated_count = 0_u32;
    let mut rejected_csrf_count = 0_u32;
    let mut rejected_rate_limited_count = 0_u32;
    let mut missing_control_count = 0_u32;
    let mut loopback_bind_validated = !reports.is_empty();
    let mut secure_headers_validated = !reports.is_empty();
    let mut local_server_started = false;
    let mut network_request_served = false;
    let mut public_network_exposed = false;
    let mut live_controls_enabled = false;
    let mut production_ready = false;

    for report in reports {
        report.validate()?;
        validation_ids.push(report.validation_id.clone());
        missing_control_count = missing_control_count.saturating_add(report.missing_control_count);
        loopback_bind_validated &= report.loopback_bind_validated;
        secure_headers_validated &= report.secure_headers_validated;
        local_server_started |= report.local_server_started;
        network_request_served |= report.network_request_served;
        public_network_exposed |= report.public_network_exposed;
        live_controls_enabled |= report.live_controls_enabled;
        production_ready |= report.production_ready;
        if report.status == DashboardHostedRequestValidationStatus::ReadyForLocalReview
            && report.authenticated
            && report.authorized
            && report.csrf_validated
            && report.rate_limit_validated
        {
            accepted_request_count = accepted_request_count.saturating_add(1);
        }
        if report.status == DashboardHostedRequestValidationStatus::BlockedMissingControls
            && !report.authenticated
        {
            rejected_unauthenticated_count = rejected_unauthenticated_count.saturating_add(1);
        }
        if report.status == DashboardHostedRequestValidationStatus::BlockedMissingControls
            && !report.csrf_validated
        {
            rejected_csrf_count = rejected_csrf_count.saturating_add(1);
        }
        if report.status == DashboardHostedRequestValidationStatus::BlockedMissingControls
            && !report.rate_limit_validated
        {
            rejected_rate_limited_count = rejected_rate_limited_count.saturating_add(1);
        }
    }

    let status = if accepted_request_count > 0
        && rejected_unauthenticated_count > 0
        && rejected_csrf_count > 0
        && rejected_rate_limited_count > 0
        && loopback_bind_validated
        && secure_headers_validated
        && local_server_started
        && network_request_served
        && !public_network_exposed
        && !live_controls_enabled
        && !production_ready
    {
        DashboardHostedSessionValidationStatus::ReadyForLocalReview
    } else {
        DashboardHostedSessionValidationStatus::BlockedMissingControls
    };
    let report = DashboardHostedSessionValidationReport {
        dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
        session_id,
        status,
        validation_ids,
        total_request_count: u32::try_from(reports.len()).map_err(|_| {
            DashboardError::StateStoreFailed {
                reason: "hosted dashboard session request count overflowed".to_owned(),
            }
        })?,
        accepted_request_count,
        rejected_unauthenticated_count,
        rejected_csrf_count,
        rejected_rate_limited_count,
        loopback_bind_validated,
        secure_headers_validated,
        local_server_started,
        network_request_served,
        missing_control_count,
        public_network_exposed,
        live_controls_enabled,
        production_ready,
    };
    report.validate()?;
    Ok(report)
}

/// Validate local hosted dashboard session lifecycle references and controls.
pub fn validate_dashboard_hosted_session_lifecycle(
    request: &DashboardHostedSessionLifecycleValidation,
) -> Result<DashboardHostedSessionLifecycleValidationReport, DashboardError> {
    request.validate()?;
    let session_reference_recorded = !request.session_reference.trim().is_empty()
        && !contains_secret_like_text(&request.session_reference);
    let csrf_reference_recorded = !request.csrf_reference.trim().is_empty()
        && !contains_secret_like_text(&request.csrf_reference);
    let csrf_lifecycle_validated = request.csrf_reference_issued
        && request.csrf_reference_scoped
        && request.csrf_reference_rotated;
    let rate_limit_validated = request.max_requests_per_minute > 0
        && request.rate_limit_remaining <= request.max_requests_per_minute;
    let mut violation_codes = Vec::new();
    push_dashboard_code(
        &mut violation_codes,
        !session_reference_recorded,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SESSION_REFERENCE_MISSING",
    );
    push_dashboard_code(
        &mut violation_codes,
        !csrf_reference_recorded,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_CSRF_REFERENCE_MISSING",
    );
    push_dashboard_code(
        &mut violation_codes,
        !request.authenticated,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_NOT_AUTHENTICATED",
    );
    push_dashboard_code(
        &mut violation_codes,
        !request.authorized,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_NOT_AUTHORIZED",
    );
    push_dashboard_code(
        &mut violation_codes,
        !csrf_lifecycle_validated,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_CSRF_INCOMPLETE",
    );
    push_dashboard_code(
        &mut violation_codes,
        !request.session_revocation_supported,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_REVOCATION_MISSING",
    );
    push_dashboard_code(
        &mut violation_codes,
        request.session_revoked,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SESSION_REVOKED",
    );
    push_dashboard_code(
        &mut violation_codes,
        !request.read_only_role,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_ROLE_NOT_READ_ONLY",
    );
    push_dashboard_code(
        &mut violation_codes,
        !rate_limit_validated,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_RATE_LIMIT_INVALID",
    );
    push_dashboard_code(
        &mut violation_codes,
        !request.loopback_only,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_NOT_LOOPBACK_ONLY",
    );
    push_dashboard_code(
        &mut violation_codes,
        request.public_network_exposed,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_PUBLIC_NETWORK_EXPOSED",
    );
    push_dashboard_code(
        &mut violation_codes,
        request.live_controls_enabled,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_LIVE_CONTROLS_ENABLED",
    );
    push_dashboard_code(
        &mut violation_codes,
        request.secret_material_present,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SECRET_MATERIAL_PRESENT",
    );
    push_dashboard_code(
        &mut violation_codes,
        request.persistent_server_started,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SERVER_STARTED",
    );
    push_dashboard_code(
        &mut violation_codes,
        request.production_ready_claimed,
        "DASHBOARD_HOSTED_SESSION_LIFECYCLE_PRODUCTION_READY_CLAIMED",
    );
    let missing_control_count = u32::try_from(violation_codes.len()).unwrap_or(u32::MAX);
    let report = DashboardHostedSessionLifecycleValidationReport {
        dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
        lifecycle_id: request.lifecycle_id.clone(),
        status: if missing_control_count == 0 {
            DashboardHostedSessionLifecycleValidationStatus::ReadyForLocalReview
        } else {
            DashboardHostedSessionLifecycleValidationStatus::BlockedMissingControls
        },
        session_reference_recorded,
        csrf_reference_recorded,
        operator_role: request.operator_role.clone(),
        authenticated: request.authenticated,
        authorized: request.authorized,
        csrf_lifecycle_validated,
        session_revocation_supported: request.session_revocation_supported,
        session_revoked: request.session_revoked,
        read_only_role: request.read_only_role,
        rate_limit_validated,
        loopback_only: request.loopback_only,
        missing_control_count,
        public_network_exposed: request.public_network_exposed,
        live_controls_enabled: request.live_controls_enabled,
        secret_material_present: request.secret_material_present,
        persistent_server_started: request.persistent_server_started,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a bounded loopback-only dashboard runtime that serves multiple
/// read-only local requests on one listener before shutting down.
///
/// This exercises local socket lifecycle behavior only. It does not expose a
/// public dashboard, authenticate real browsers, enable live controls, or claim
/// production readiness.
pub fn validate_dashboard_loopback_runtime_probe(
    probe: DashboardLoopbackRuntimeProbe,
) -> Result<DashboardLoopbackRuntimeProbeReport, DashboardError> {
    probe.validate()?;
    let mut missing_control_count = 0_u32;
    let mut warnings = Vec::new();
    let bind_host = probe.bind_host.trim().to_owned();
    let loopback_ip = parse_loopback_ip(&bind_host);
    let loopback_bind_validated = loopback_ip.is_some();
    if !loopback_bind_validated {
        missing_control_count = missing_control_count.saturating_add(1);
        warnings.push("dashboard loopback runtime bind host is not loopback".to_owned());
    }

    let mut bound_port = None;
    let mut served_request_count = 0_u32;
    let mut all_requests_returned_ok = false;
    let mut response_digest_consistent = false;
    let mut bounded_runtime_started = false;
    let mut bounded_runtime_shutdown = false;
    if let Some(ip) = loopback_ip {
        let body = render_dashboard_response_body(&probe.render_record)?;
        match serve_bounded_dashboard_loopback_runtime(
            ip,
            probe.requested_port,
            DashboardHostedRequestMethod::Get,
            "/",
            body,
            probe.request_count,
        ) {
            Ok(exchange) => {
                bound_port = Some(exchange.bound_port);
                served_request_count = exchange.served_request_count;
                all_requests_returned_ok = exchange.all_requests_returned_ok;
                response_digest_consistent = exchange.response_digest_consistent;
                bounded_runtime_started = true;
                bounded_runtime_shutdown = exchange.bounded_runtime_shutdown;
                if served_request_count != probe.request_count
                    || !all_requests_returned_ok
                    || !response_digest_consistent
                    || !bounded_runtime_shutdown
                {
                    missing_control_count = missing_control_count.saturating_add(1);
                    warnings.push(
                        "dashboard loopback runtime did not serve all expected requests cleanly"
                            .to_owned(),
                    );
                }
            }
            Err(error) => {
                missing_control_count = missing_control_count.saturating_add(1);
                warnings.push(format!("dashboard loopback runtime failed: {error}"));
            }
        }
    }
    let report = DashboardLoopbackRuntimeProbeReport {
        dashboard_boundary_version: DASHBOARD_BOUNDARY_VERSION.to_owned(),
        probe_id: probe.probe_id,
        status: if missing_control_count == 0 {
            DashboardLoopbackRuntimeProbeStatus::ReadyForLocalReview
        } else {
            DashboardLoopbackRuntimeProbeStatus::Blocked
        },
        bind_host,
        requested_port: probe.requested_port,
        bound_port,
        loopback_bind_validated,
        expected_request_count: probe.request_count,
        served_request_count,
        all_requests_returned_ok,
        response_digest_consistent,
        missing_control_count,
        bounded_runtime_started,
        bounded_runtime_shutdown,
        public_network_exposed: false,
        live_controls_enabled: false,
        production_ready: false,
        warnings,
    };
    report.validate()?;
    Ok(report)
}

/// Compose local hosted-dashboard runtime readiness evidence without starting a
/// persistent server, exposing public bindings, enabling live controls, or
/// claiming production readiness.
pub fn review_dashboard_hosted_runtime_readiness(
    request: DashboardHostedRuntimeReadinessReviewRequest,
) -> Result<DashboardHostedRuntimeReadinessReviewReport, DashboardError> {
    request.validate()?;

    let security_review_ready =
        request.security_review.status == DashboardHostedSecurityReviewStatus::ReadyForLocalReview;
    let request_preflight_ready = request.request_preflight.status
        == DashboardHostedRequestPreflightStatus::ReadyForLocalReview;
    let session_validation_ready = request.session_validation.status
        == DashboardHostedSessionValidationStatus::ReadyForLocalReview;
    let accepted_request_validated = request.session_validation.accepted_request_count > 0;
    let unauthenticated_rejection_validated =
        request.session_validation.rejected_unauthenticated_count > 0;
    let csrf_rejection_validated = request.session_validation.rejected_csrf_count > 0;
    let rate_limit_rejection_validated = request.session_validation.rejected_rate_limited_count > 0;
    let loopback_serving_validated = request.request_preflight.loopback_bind_validated
        && request.session_validation.loopback_bind_validated
        && request.session_validation.local_server_started
        && request.session_validation.network_request_served;
    let secure_headers_validated = request.request_preflight.secure_headers_validated
        && request.session_validation.secure_headers_validated;
    let remaining_external_evidence_count = request.remaining_external_evidence.len();
    let remaining_external_evidence_recorded = remaining_external_evidence_count > 0;
    let missing_control_count = request
        .security_review
        .missing_control_count
        .saturating_add(request.request_preflight.missing_control_count)
        .saturating_add(request.session_validation.missing_control_count);
    let persistent_server_started = request.persistent_server_start_requested;
    let public_network_exposed = request.public_network_exposure_requested
        || request.security_review.public_network_exposed
        || request.request_preflight.public_network_exposed
        || request.session_validation.public_network_exposed;
    let live_controls_enabled = request.live_controls_requested
        || request.security_review.live_controls_enabled
        || request.request_preflight.live_controls_enabled
        || request.session_validation.live_controls_enabled;
    let production_ready = request.production_ready_claimed
        || request.security_review.production_ready
        || request.request_preflight.production_ready
        || request.session_validation.production_ready;

    let mut violation_codes = Vec::new();
    push_dashboard_code_if(
        &mut violation_codes,
        !security_review_ready,
        "DASHBOARD_HOSTED_RUNTIME_SECURITY_NOT_READY",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !request_preflight_ready,
        "DASHBOARD_HOSTED_RUNTIME_PREFLIGHT_NOT_READY",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !session_validation_ready,
        "DASHBOARD_HOSTED_RUNTIME_SESSION_NOT_READY",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !accepted_request_validated,
        "DASHBOARD_HOSTED_RUNTIME_ACCEPTED_REQUEST_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !unauthenticated_rejection_validated,
        "DASHBOARD_HOSTED_RUNTIME_UNAUTH_REJECTION_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !csrf_rejection_validated,
        "DASHBOARD_HOSTED_RUNTIME_CSRF_REJECTION_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !rate_limit_rejection_validated,
        "DASHBOARD_HOSTED_RUNTIME_RATE_LIMIT_REJECTION_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !loopback_serving_validated,
        "DASHBOARD_HOSTED_RUNTIME_LOOPBACK_SERVING_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !secure_headers_validated,
        "DASHBOARD_HOSTED_RUNTIME_SECURE_HEADERS_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        !remaining_external_evidence_recorded,
        "DASHBOARD_HOSTED_RUNTIME_REMAINING_EVIDENCE_MISSING",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        persistent_server_started,
        "DASHBOARD_HOSTED_RUNTIME_PERSISTENT_SERVER_STARTED",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        public_network_exposed,
        "DASHBOARD_HOSTED_RUNTIME_PUBLIC_NETWORK_EXPOSED",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        live_controls_enabled,
        "DASHBOARD_HOSTED_RUNTIME_LIVE_CONTROLS_ENABLED",
    );
    push_dashboard_code_if(
        &mut violation_codes,
        production_ready,
        "DASHBOARD_HOSTED_RUNTIME_PRODUCTION_READY_CLAIMED",
    );

    let report = DashboardHostedRuntimeReadinessReviewReport {
        review_version: DASHBOARD_HOSTED_RUNTIME_READINESS_REVIEW_VERSION.to_owned(),
        review_id: request.review_id,
        status: if violation_codes.is_empty() {
            DashboardHostedRuntimeReadinessReviewStatus::ReadyForLocalReview
        } else {
            DashboardHostedRuntimeReadinessReviewStatus::Blocked
        },
        security_review_ready,
        request_preflight_ready,
        session_validation_ready,
        accepted_request_validated,
        unauthenticated_rejection_validated,
        csrf_rejection_validated,
        rate_limit_rejection_validated,
        loopback_serving_validated,
        secure_headers_validated,
        remaining_external_evidence_recorded,
        remaining_external_evidence_count,
        missing_control_count,
        persistent_server_started,
        public_network_exposed,
        live_controls_enabled,
        production_ready,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

struct DashboardHostedRequestExchange {
    bound_port: u16,
    local_http_status_code: u16,
    network_request_served: bool,
    response_body_bytes: u64,
    response_body_sha256: String,
}

struct DashboardLoopbackRuntimeExchange {
    bound_port: u16,
    served_request_count: u32,
    all_requests_returned_ok: bool,
    response_digest_consistent: bool,
    bounded_runtime_shutdown: bool,
}

fn serve_one_dashboard_request(
    ip: IpAddr,
    requested_port: u16,
    method: DashboardHostedRequestMethod,
    path: &str,
    body: String,
) -> Result<DashboardHostedRequestExchange, String> {
    let listener = TcpListener::bind((ip, requested_port))
        .map_err(|error| format!("bind failed: {}", error.kind()))?;
    let bound_port = listener
        .local_addr()
        .map_err(|error| format!("local address failed: {}", error.kind()))?
        .port();
    let response_body_bytes = u64::try_from(body.len())
        .map_err(|_| "dashboard response body length overflowed".to_owned())?;
    let response_body_sha256 = sha256_hex(body.as_bytes());
    let server_handle = thread::spawn(move || serve_dashboard_connection(listener, body));
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
        "{} {path} HTTP/1.1\r\nHost: {ip}:{bound_port}\r\nAuthorization: Bearer local-dashboard-reference\r\nX-CSRF-Token: local\r\nConnection: close\r\n\r\n",
        method.as_http_method()
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
    Ok(DashboardHostedRequestExchange {
        bound_port,
        local_http_status_code: parse_http_status_code(&response).unwrap_or(0),
        network_request_served: request_served,
        response_body_bytes,
        response_body_sha256,
    })
}

fn serve_bounded_dashboard_loopback_runtime(
    ip: IpAddr,
    requested_port: u16,
    method: DashboardHostedRequestMethod,
    path: &str,
    body: String,
    request_count: u32,
) -> Result<DashboardLoopbackRuntimeExchange, String> {
    if request_count == 0 {
        return Err("request count must be positive".to_owned());
    }
    let listener = TcpListener::bind((ip, requested_port))
        .map_err(|error| format!("bind failed: {}", error.kind()))?;
    let bound_port = listener
        .local_addr()
        .map_err(|error| format!("local address failed: {}", error.kind()))?
        .port();
    let expected_body_sha256 = sha256_hex(body.as_bytes());
    let server_handle =
        thread::spawn(move || serve_dashboard_connections(listener, body, request_count));
    let mut served_request_count = 0_u32;
    let mut all_requests_returned_ok = true;
    let mut response_digest_consistent = true;
    for _ in 0..request_count {
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
            "{} {path} HTTP/1.1\r\nHost: {ip}:{bound_port}\r\nAuthorization: Bearer local-dashboard-reference\r\nX-CSRF-Token: local\r\nConnection: close\r\n\r\n",
            method.as_http_method()
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| format!("client write failed: {}", error.kind()))?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|error| format!("client read failed: {}", error.kind()))?;
        served_request_count = served_request_count.saturating_add(1);
        all_requests_returned_ok &= parse_http_status_code(&response) == Some(200);
        let body_start = response
            .find("\r\n\r\n")
            .map_or(response.as_str(), |index| {
                &response[index.saturating_add(4)..]
            });
        response_digest_consistent &= sha256_hex(body_start.as_bytes()) == expected_body_sha256;
    }
    let server_served_count = server_handle
        .join()
        .map_err(|_| "server thread panicked".to_owned())?
        .map_err(|error| format!("server failed: {error}"))?;
    Ok(DashboardLoopbackRuntimeExchange {
        bound_port,
        served_request_count,
        all_requests_returned_ok,
        response_digest_consistent,
        bounded_runtime_shutdown: server_served_count == request_count
            && served_request_count == request_count,
    })
}

fn render_dashboard_response_body(
    record: &DashboardRenderRecord,
) -> Result<String, DashboardError> {
    record.validate()?;
    let mut body = String::new();
    writeln!(body, "snapshot_id={}", record.snapshot_id)
        .expect("writing dashboard response body to String cannot fail");
    writeln!(body, "runtime_mode={:?}", record.runtime_mode)
        .expect("writing dashboard response body to String cannot fail");
    writeln!(
        body,
        "production_readiness_percent={}",
        record.production_readiness_percent
    )
    .expect("writing dashboard response body to String cannot fail");
    writeln!(body, "panel_count={}", record.panels.len())
        .expect("writing dashboard response body to String cannot fail");
    for panel in &record.panels {
        writeln!(body, "panel={:?}:{}", panel.kind, panel.title)
            .expect("writing dashboard response body to String cannot fail");
        writeln!(body, "summary={}", panel.summary)
            .expect("writing dashboard response body to String cannot fail");
        for item in &panel.items {
            writeln!(
                body,
                "item={}:{}:{:?}",
                item.label, item.value, item.severity
            )
            .expect("writing dashboard response body to String cannot fail");
        }
    }
    if contains_secret_like_text(&body) {
        return Err(DashboardError::ValidationFailed {
            violations: vec![DashboardViolation::new(
                "DASHBOARD_HOSTED_RESPONSE_SECRET_LIKE",
                "hosted dashboard response body still looks like secret material",
            )],
        });
    }
    Ok(body)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn serve_dashboard_connection(listener: TcpListener, body: String) -> Result<bool, String> {
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
    let authorized = request.starts_with("GET / ")
        && request.contains("\r\nAuthorization: Bearer local-dashboard-reference\r\n")
        && request.contains("\r\nX-CSRF-Token: local\r\n");
    let (status, response_body) = if authorized {
        ("200 OK", body)
    } else {
        ("403 Forbidden", String::new())
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Security-Policy: default-src 'self'\r\nX-Frame-Options: DENY\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
        response_body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("server write failed: {}", error.kind()))?;
    Ok(true)
}

fn serve_dashboard_connections(
    listener: TcpListener,
    body: String,
    request_count: u32,
) -> Result<u32, String> {
    let mut served = 0_u32;
    for _ in 0..request_count {
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
        let authorized = request.starts_with("GET / ")
            && request.contains("\r\nAuthorization: Bearer local-dashboard-reference\r\n")
            && request.contains("\r\nX-CSRF-Token: local\r\n");
        let (status, response_body) = if authorized {
            ("200 OK", body.as_str())
        } else {
            ("403 Forbidden", "")
        };
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Security-Policy: default-src 'self'\r\nX-Frame-Options: DENY\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
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
    /// Local dashboard access context for authorization.
    pub access: DashboardAccessContext,
    /// Optional panel allowlist. Empty means all panels.
    pub requested_panels: Vec<DashboardPanelKind>,
    /// Optional operator-facing label. Must not contain secret material.
    pub operator_label: Option<String>,
    /// Render timestamp in Unix epoch milliseconds.
    pub rendered_at_ms: u64,
}

/// Dashboard access source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardAccessSource {
    /// In-process local render path.
    LocalRender,
    /// Future local browser session.
    BrowserSession,
    /// Future remote dashboard session.
    RemoteSession,
}

/// Dashboard access authorization status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardAccessAuthorizationStatus {
    /// Local in-process render is authorized.
    AuthorizedLocalRender,
    /// Local rendering is disabled by config.
    RejectedLocalRenderingDisabled,
    /// Hosted/browser session source is not enabled in this phase.
    RejectedHostedSession,
}

/// Non-secret dashboard access context.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardAccessContext {
    /// Access source.
    pub source: DashboardAccessSource,
    /// Stable non-secret operator label or local process label.
    pub operator_label: Option<String>,
}

impl DashboardAccessContext {
    /// Local in-process render access context.
    #[must_use]
    pub fn local_render(operator_label: Option<String>) -> Self {
        Self {
            source: DashboardAccessSource::LocalRender,
            operator_label,
        }
    }

    fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        if let Some(label) = &self.operator_label {
            if contains_secret_like_text(label) {
                violations.push(DashboardViolation::new(
                    "DASHBOARD_ACCESS_LABEL_SECRET_LIKE",
                    "dashboard access operator label looks like secret material",
                ));
            }
        }
        finish_validation(violations)
    }
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
    /// Whether dashboard access was locally authorized.
    pub access_authorized: bool,
    /// Dashboard access authorization status.
    pub access_authorization_status: DashboardAccessAuthorizationStatus,
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
        request.access.validate()?;

        if !request.config.local_rendering_enabled {
            return Err(DashboardError::ValidationFailed {
                violations: vec![DashboardViolation::new(
                    "DASHBOARD_LOCAL_RENDERING_DISABLED",
                    "local dashboard rendering is disabled",
                )],
            });
        }

        let access_decision = authorize_dashboard_access(&request.access, &request.config);
        if !access_decision.access_authorized {
            return Err(DashboardError::ValidationFailed {
                violations: vec![DashboardViolation::new_owned(
                    "DASHBOARD_ACCESS_DENIED",
                    access_decision.reason,
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
            access_authorized: access_decision.access_authorized,
            access_authorization_status: access_decision.status,
            panels,
            warnings,
            server_started: false,
            public_network_exposed: false,
            live_controls_enabled: false,
            secret_redaction_applied: redaction_applied,
        })
    }
}

impl DashboardRenderRecord {
    /// Validate local dashboard render record invariants.
    pub fn validate(&self) -> Result<(), DashboardError> {
        let mut violations = Vec::new();
        validate_id("dashboard snapshot", &self.snapshot_id, &mut violations);
        if self.dashboard_boundary_version != DASHBOARD_BOUNDARY_VERSION {
            violations.push(DashboardViolation::new_owned(
                "DASHBOARD_VERSION_MISMATCH",
                format!(
                    "dashboard_boundary_version must be {DASHBOARD_BOUNDARY_VERSION}, got {}",
                    self.dashboard_boundary_version
                ),
            ));
        }
        if self.server_started {
            violations.push(DashboardViolation::new(
                "DASHBOARD_SERVER_STARTED",
                "dashboard render records must not start a server",
            ));
        }
        if self.public_network_exposed {
            violations.push(DashboardViolation::new(
                "DASHBOARD_PUBLIC_NETWORK_EXPOSED",
                "dashboard render records must not expose public network bindings",
            ));
        }
        if self.live_controls_enabled {
            violations.push(DashboardViolation::new(
                "DASHBOARD_LIVE_CONTROLS_ENABLED",
                "dashboard render records must not enable live controls",
            ));
        }
        if !self.access_authorized {
            violations.push(DashboardViolation::new(
                "DASHBOARD_RENDER_ACCESS_NOT_AUTHORIZED",
                "dashboard render records must be locally access-authorized",
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
            if contains_secret_like_text(&panel.title) || contains_secret_like_text(&panel.summary)
            {
                violations.push(DashboardViolation::new(
                    "DASHBOARD_PANEL_SECRET_LIKE",
                    "dashboard panel text still looks like it may contain secret material",
                ));
            }
            for item in &panel.items {
                if item.label.trim().is_empty() {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_PANEL_ITEM_LABEL_EMPTY",
                        "dashboard panel item label must be non-empty",
                    ));
                }
                if contains_secret_like_text(&item.label) || contains_secret_like_text(&item.value)
                {
                    violations.push(DashboardViolation::new(
                        "DASHBOARD_PANEL_ITEM_SECRET_LIKE",
                        "dashboard panel item text still looks like it may contain secret material",
                    ));
                }
            }
        }
        for warning in &self.warnings {
            if contains_secret_like_text(warning) {
                violations.push(DashboardViolation::new(
                    "DASHBOARD_WARNING_SECRET_LIKE",
                    "dashboard warning still looks like it may contain secret material",
                ));
            }
        }
        finish_validation(violations)
    }
}

/// Persist the latest local dashboard render through the typed state boundary.
///
/// This stores redacted dashboard render metadata only. It does not start a web
/// server, expose a browser UI, authenticate operators, or enable live controls.
pub fn persist_dashboard_render_checkpoint(
    store: &mut impl StateStore,
    record: &DashboardRenderRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_RENDER_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!("failed to serialize dashboard render checkpoint: {error}"),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append one local dashboard render record to the append-only audit journal.
///
/// This records sanitized render outcomes only. It does not start a server,
/// expose a dashboard, or enable operator controls.
pub fn append_dashboard_render_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &DashboardRenderRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("dashboard-render-{}", record.snapshot_id),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-renderer",
        "dashboard render recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
        )
        .with_metadata("snapshot_id", AuditValue::Text(record.snapshot_id.clone()))
        .with_metadata(
            "runtime_mode",
            AuditValue::Text(format!("{:?}", record.runtime_mode)),
        )
        .with_metadata(
            "production_readiness_percent",
            AuditValue::Text(record.production_readiness_percent.to_string()),
        )
        .with_metadata(
            "panel_count",
            AuditValue::Text(record.panels.len().to_string()),
        )
        .with_metadata(
            "access_authorized",
            AuditValue::Bool(record.access_authorized),
        )
        .with_metadata(
            "access_authorization_status",
            AuditValue::Text(format!("{:?}", record.access_authorization_status)),
        )
        .with_metadata("server_started", AuditValue::Bool(record.server_started))
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(record.public_network_exposed),
        )
        .with_metadata(
            "live_controls_enabled",
            AuditValue::Bool(record.live_controls_enabled),
        )
        .with_metadata(
            "render_text_redaction_applied",
            AuditValue::Bool(record.secret_redaction_applied),
        );
    journal.append_event(event).map_err(DashboardError::from)
}

/// Persist the latest hosted-dashboard security review through the typed state boundary.
///
/// This stores local review metadata only. It does not start a web server,
/// expose a browser UI, authenticate operators, issue CSRF tokens, or enable live controls.
pub fn persist_dashboard_hosted_security_review_checkpoint(
    store: &mut impl StateStore,
    report: &DashboardHostedSecurityReviewReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!(
                "failed to serialize dashboard hosted security review checkpoint: {error}"
            ),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append one local hosted-dashboard security review to the append-only audit journal.
///
/// This records local review outcomes only. It does not start a server,
/// expose a dashboard, authenticate browser sessions, issue CSRF tokens, or enable controls.
pub fn append_dashboard_hosted_security_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DashboardHostedSecurityReviewReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("dashboard-hosted-security-review-{}", report.review_id),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-hosted-security-review",
        "dashboard hosted security review recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
        )
        .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "csrf_protection_required",
            AuditValue::Bool(report.csrf_protection_required),
        )
        .with_metadata(
            "secure_headers_required",
            AuditValue::Bool(report.secure_headers_required),
        )
        .with_metadata(
            "rate_limit_required",
            AuditValue::Bool(report.rate_limit_required),
        )
        .with_metadata(
            "audit_state_preflight_required",
            AuditValue::Bool(report.audit_state_preflight_required),
        )
        .with_metadata(
            "session_revocation_required",
            AuditValue::Bool(report.session_revocation_required),
        )
        .with_metadata(
            "operator_role_review_required",
            AuditValue::Bool(report.operator_role_review_required),
        )
        .with_metadata(
            "read_only_controls_required",
            AuditValue::Bool(report.read_only_controls_required),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata("server_started", AuditValue::Bool(report.server_started))
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "live_controls_enabled",
            AuditValue::Bool(report.live_controls_enabled),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal.append_event(event).map_err(DashboardError::from)
}

/// Persist the latest hosted-dashboard request preflight through the typed state boundary.
///
/// This stores local preflight metadata only. It does not start a web server,
/// bind sockets, authenticate browser sessions, issue CSRF tokens, or enable controls.
pub fn persist_dashboard_hosted_request_preflight_checkpoint(
    store: &mut impl StateStore,
    report: &DashboardHostedRequestPreflightReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!(
                "failed to serialize dashboard hosted request preflight checkpoint: {error}"
            ),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append one local hosted-dashboard request preflight to the append-only audit journal.
///
/// This records local preflight outcomes only. It does not start a server,
/// expose a dashboard, authenticate browser sessions, issue CSRF tokens, or enable controls.
pub fn append_dashboard_hosted_request_preflight_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DashboardHostedRequestPreflightReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("dashboard-hosted-request-preflight-{}", report.preflight_id),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-hosted-request-preflight",
        "dashboard hosted request preflight recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
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
            "access_source",
            AuditValue::Text(format!("{:?}", report.access_source)),
        )
        .with_metadata("method", AuditValue::Text(format!("{:?}", report.method)))
        .with_metadata("authenticated", AuditValue::Bool(report.authenticated))
        .with_metadata("authorized", AuditValue::Bool(report.authorized))
        .with_metadata("csrf_validated", AuditValue::Bool(report.csrf_validated))
        .with_metadata(
            "secure_headers_validated",
            AuditValue::Bool(report.secure_headers_validated),
        )
        .with_metadata(
            "rate_limit_validated",
            AuditValue::Bool(report.rate_limit_validated),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata("server_started", AuditValue::Bool(report.server_started))
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "live_controls_enabled",
            AuditValue::Bool(report.live_controls_enabled),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal.append_event(event).map_err(DashboardError::from)
}

/// Persist the latest local one-shot hosted-dashboard request validation.
///
/// This stores sanitized local loopback validation metadata only. It does not
/// publish a dashboard, authenticate real browser sessions, retain CSRF
/// material, enable live controls, or approve production readiness.
pub fn persist_dashboard_hosted_request_validation_checkpoint(
    store: &mut impl StateStore,
    report: &DashboardHostedRequestValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!(
                "failed to serialize dashboard hosted request validation checkpoint: {error}"
            ),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append one local one-shot hosted-dashboard request validation to audit.
///
/// This records local loopback validation outcomes only. It does not expose
/// public networks, preserve request contents, store secret material, enable
/// live controls, or claim production readiness.
pub fn append_dashboard_hosted_request_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DashboardHostedRequestValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!(
            "dashboard-hosted-request-validation-{}",
            report.validation_id
        ),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-hosted-request-validation",
        "dashboard hosted request validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
        )
        .with_metadata(
            "validation_id",
            AuditValue::Text(report.validation_id.clone()),
        )
        .with_metadata("render_id", AuditValue::Text(report.render_id.clone()))
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
        .with_metadata("method", AuditValue::Text(format!("{:?}", report.method)))
        .with_metadata(
            "request_path",
            AuditValue::Text(report.request_path.clone()),
        )
        .with_metadata(
            "loopback_bind_validated",
            AuditValue::Bool(report.loopback_bind_validated),
        )
        .with_metadata("authenticated", AuditValue::Bool(report.authenticated))
        .with_metadata("authorized", AuditValue::Bool(report.authorized))
        .with_metadata("csrf_validated", AuditValue::Bool(report.csrf_validated))
        .with_metadata(
            "secure_headers_validated",
            AuditValue::Bool(report.secure_headers_validated),
        )
        .with_metadata(
            "rate_limit_validated",
            AuditValue::Bool(report.rate_limit_validated),
        )
        .with_metadata(
            "local_http_status_code",
            AuditValue::Text(report.local_http_status_code.to_string()),
        )
        .with_metadata(
            "response_panel_count",
            AuditValue::Text(report.response_panel_count.to_string()),
        )
        .with_metadata(
            "response_body_bytes",
            AuditValue::Text(report.response_body_bytes.to_string()),
        )
        .with_metadata(
            "response_body_sha256",
            AuditValue::Text(report.response_body_sha256.clone()),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "local_server_started",
            AuditValue::Bool(report.local_server_started),
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
            "live_controls_enabled",
            AuditValue::Bool(report.live_controls_enabled),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal.append_event(event).map_err(DashboardError::from)
}

/// Persist the latest local hosted-dashboard session validation summary.
///
/// This stores sanitized local session-control metadata only. It does not
/// publish a dashboard, retain session credentials, enable live controls, or
/// approve production readiness.
pub fn persist_dashboard_hosted_session_validation_checkpoint(
    store: &mut impl StateStore,
    report: &DashboardHostedSessionValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!(
                "failed to serialize dashboard hosted session validation checkpoint: {error}"
            ),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append one local hosted-dashboard session validation summary to audit.
///
/// This records local session-control outcomes only. It does not expose public
/// networks, store secret material, enable live controls, or claim production
/// readiness.
pub fn append_dashboard_hosted_session_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DashboardHostedSessionValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("dashboard-hosted-session-validation-{}", report.session_id),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-hosted-session-validation",
        "dashboard hosted session validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
        )
        .with_metadata("session_id", AuditValue::Text(report.session_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "total_request_count",
            AuditValue::Text(report.total_request_count.to_string()),
        )
        .with_metadata(
            "accepted_request_count",
            AuditValue::Text(report.accepted_request_count.to_string()),
        )
        .with_metadata(
            "rejected_unauthenticated_count",
            AuditValue::Text(report.rejected_unauthenticated_count.to_string()),
        )
        .with_metadata(
            "rejected_csrf_count",
            AuditValue::Text(report.rejected_csrf_count.to_string()),
        )
        .with_metadata(
            "rejected_rate_limited_count",
            AuditValue::Text(report.rejected_rate_limited_count.to_string()),
        )
        .with_metadata(
            "loopback_bind_validated",
            AuditValue::Bool(report.loopback_bind_validated),
        )
        .with_metadata(
            "secure_headers_validated",
            AuditValue::Bool(report.secure_headers_validated),
        )
        .with_metadata(
            "local_server_started",
            AuditValue::Bool(report.local_server_started),
        )
        .with_metadata(
            "network_request_served",
            AuditValue::Bool(report.network_request_served),
        )
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "live_controls_enabled",
            AuditValue::Bool(report.live_controls_enabled),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal.append_event(event).map_err(DashboardError::from)
}

/// Persist the latest local hosted-dashboard session lifecycle validation.
///
/// This stores sanitized local lifecycle metadata only. It does not store
/// cookies, CSRF token material, browser credentials, live controls, or readiness claims.
pub fn persist_dashboard_hosted_session_lifecycle_checkpoint(
    store: &mut impl StateStore,
    report: &DashboardHostedSessionLifecycleValidationReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_HOSTED_SESSION_LIFECYCLE_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!(
                "failed to serialize dashboard hosted session lifecycle checkpoint: {error}"
            ),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append one local hosted-dashboard session lifecycle validation to audit.
///
/// This records non-secret lifecycle outcomes only. It does not expose public
/// networks, store token material, start persistent servers, enable live controls,
/// or claim production readiness.
pub fn append_dashboard_hosted_session_lifecycle_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DashboardHostedSessionLifecycleValidationReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("dashboard-hosted-session-lifecycle-{}", report.lifecycle_id),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-hosted-session-lifecycle",
        "dashboard hosted session lifecycle validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
        )
        .with_metadata(
            "lifecycle_id",
            AuditValue::Text(report.lifecycle_id.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "session_reference_recorded",
            AuditValue::Bool(report.session_reference_recorded),
        )
        .with_metadata(
            "csrf_reference_recorded",
            AuditValue::Bool(report.csrf_reference_recorded),
        )
        .with_metadata(
            "operator_role",
            AuditValue::Text(report.operator_role.clone()),
        )
        .with_metadata("authenticated", AuditValue::Bool(report.authenticated))
        .with_metadata("authorized", AuditValue::Bool(report.authorized))
        .with_metadata(
            "csrf_lifecycle_validated",
            AuditValue::Bool(report.csrf_lifecycle_validated),
        )
        .with_metadata(
            "session_revocation_supported",
            AuditValue::Bool(report.session_revocation_supported),
        )
        .with_metadata("session_revoked", AuditValue::Bool(report.session_revoked))
        .with_metadata("read_only_role", AuditValue::Bool(report.read_only_role))
        .with_metadata(
            "rate_limit_validated",
            AuditValue::Bool(report.rate_limit_validated),
        )
        .with_metadata("loopback_only", AuditValue::Bool(report.loopback_only))
        .with_metadata(
            "missing_control_count",
            AuditValue::Text(report.missing_control_count.to_string()),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "live_controls_enabled",
            AuditValue::Bool(report.live_controls_enabled),
        )
        .with_metadata(
            "sensitive_material_present",
            AuditValue::Bool(report.secret_material_present),
        )
        .with_metadata(
            "persistent_server_started",
            AuditValue::Bool(report.persistent_server_started),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal.append_event(event).map_err(DashboardError::from)
}

/// Persist the latest bounded local loopback dashboard runtime probe checkpoint.
///
/// This stores sanitized local runtime probe metadata only. It does not expose
/// public networks, retain browser credentials, enable live controls, or claim
/// production readiness.
pub fn persist_dashboard_loopback_runtime_probe_checkpoint(
    store: &mut impl StateStore,
    report: &DashboardLoopbackRuntimeProbeReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DashboardError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DASHBOARD_LAST_LOOPBACK_RUNTIME_PROBE_CHECKPOINT_KEY.to_owned(),
        subsystem: DASHBOARD_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| DashboardError::StateStoreFailed {
            reason: format!("failed to serialize dashboard loopback runtime checkpoint: {error}"),
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DashboardError::from)?;
    Ok(checkpoint)
}

/// Append a bounded local loopback dashboard runtime probe audit record.
///
/// This records local loopback lifecycle outcomes only. It does not expose
/// public networks, store secret material, enable live controls, or claim
/// production readiness.
pub fn append_dashboard_loopback_runtime_probe_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DashboardLoopbackRuntimeProbeReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DashboardError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("dashboard-loopback-runtime-probe-{}", report.probe_id),
        AuditEventKind::RuntimeLifecycle,
        DASHBOARD_STATE_SUBSYSTEM,
        "dashboard-loopback-runtime-probe",
        "dashboard loopback runtime probe recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dashboard_boundary_version",
            AuditValue::Text(DASHBOARD_BOUNDARY_VERSION.to_owned()),
        )
        .with_metadata("probe_id", AuditValue::Text(report.probe_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("bind_host", AuditValue::Text(report.bind_host.clone()))
        .with_metadata(
            "expected_request_count",
            AuditValue::Text(report.expected_request_count.to_string()),
        )
        .with_metadata(
            "served_request_count",
            AuditValue::Text(report.served_request_count.to_string()),
        )
        .with_metadata(
            "all_requests_returned_ok",
            AuditValue::Bool(report.all_requests_returned_ok),
        )
        .with_metadata(
            "response_digest_consistent",
            AuditValue::Bool(report.response_digest_consistent),
        )
        .with_metadata(
            "bounded_runtime_started",
            AuditValue::Bool(report.bounded_runtime_started),
        )
        .with_metadata(
            "bounded_runtime_shutdown",
            AuditValue::Bool(report.bounded_runtime_shutdown),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(report.public_network_exposed),
        )
        .with_metadata(
            "live_controls_enabled",
            AuditValue::Bool(report.live_controls_enabled),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal.append_event(event).map_err(DashboardError::from)
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

impl From<crate::AuditError> for DashboardError {
    fn from(error: crate::AuditError) -> Self {
        Self::AuditJournalFailed {
            reason: error.to_string(),
        }
    }
}

impl From<StateStoreError> for DashboardError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
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
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "dashboard audit journal failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(formatter, "dashboard state store failed: {reason}")
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

fn push_dashboard_code_if(codes: &mut Vec<String>, condition: bool, code: &str) {
    if condition {
        codes.push(code.to_owned());
    }
}

fn push_dashboard_code(codes: &mut Vec<String>, condition: bool, code: &'static str) {
    push_dashboard_code_if(codes, condition, code);
}

fn validate_reference_id(kind: &'static str, id: &str, violations: &mut Vec<DashboardViolation>) {
    validate_id(kind, id, violations);
    if contains_secret_like_text(id) {
        violations.push(DashboardViolation::new_owned(
            "DASHBOARD_REFERENCE_SECRET_LIKE",
            format!("{kind} must be a non-secret reference"),
        ));
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

struct DashboardAccessAuthorizationDecision {
    access_authorized: bool,
    status: DashboardAccessAuthorizationStatus,
    reason: String,
}

fn authorize_dashboard_access(
    access: &DashboardAccessContext,
    config: &DashboardBoundaryConfig,
) -> DashboardAccessAuthorizationDecision {
    match access.source {
        DashboardAccessSource::LocalRender => {
            if !config.local_rendering_enabled {
                return DashboardAccessAuthorizationDecision {
                    access_authorized: false,
                    status: DashboardAccessAuthorizationStatus::RejectedLocalRenderingDisabled,
                    reason: "local dashboard rendering is disabled by configuration".to_owned(),
                };
            }
            DashboardAccessAuthorizationDecision {
                access_authorized: true,
                status: DashboardAccessAuthorizationStatus::AuthorizedLocalRender,
                reason: "local in-process dashboard render authorized".to_owned(),
            }
        }
        DashboardAccessSource::BrowserSession | DashboardAccessSource::RemoteSession => {
            DashboardAccessAuthorizationDecision {
                access_authorized: false,
                status: DashboardAccessAuthorizationStatus::RejectedHostedSession,
                reason:
                    "hosted dashboard sessions require external authentication and remain disabled"
                        .to_owned(),
            }
        }
    }
}

fn is_loopback_host(host: &str) -> bool {
    matches!(
        host.trim().to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1"
    )
}

fn parse_loopback_ip(host: &str) -> Option<IpAddr> {
    match host.trim().to_ascii_lowercase().as_str() {
        "localhost" => Some(IpAddr::from([127, 0, 0, 1])),
        value => value.parse::<IpAddr>().ok().filter(IpAddr::is_loopback),
    }
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
        append_dashboard_hosted_request_preflight_audit,
        append_dashboard_hosted_request_validation_audit,
        append_dashboard_hosted_security_review_audit,
        append_dashboard_hosted_session_lifecycle_audit,
        append_dashboard_hosted_session_validation_audit,
        append_dashboard_loopback_runtime_probe_audit, append_dashboard_render_audit,
        is_sha256_hex, persist_dashboard_hosted_request_preflight_checkpoint,
        persist_dashboard_hosted_request_validation_checkpoint,
        persist_dashboard_hosted_security_review_checkpoint,
        persist_dashboard_hosted_session_lifecycle_checkpoint,
        persist_dashboard_hosted_session_validation_checkpoint,
        persist_dashboard_loopback_runtime_probe_checkpoint, persist_dashboard_render_checkpoint,
        preflight_dashboard_hosted_request, review_dashboard_hosted_runtime_readiness,
        review_dashboard_hosted_security, sha256_hex, validate_dashboard_hosted_request,
        validate_dashboard_hosted_session, validate_dashboard_hosted_session_lifecycle,
        validate_dashboard_loopback_runtime_probe, DashboardAccessAuthorizationStatus,
        DashboardAccessContext, DashboardAccessSource, DashboardBoundaryConfig, DashboardError,
        DashboardHostedRequestMethod, DashboardHostedRequestPreflight,
        DashboardHostedRequestPreflightStatus, DashboardHostedRequestValidation,
        DashboardHostedRequestValidationReport, DashboardHostedRequestValidationStatus,
        DashboardHostedRuntimeReadinessReviewRequest, DashboardHostedRuntimeReadinessReviewStatus,
        DashboardHostedSecurityPolicy, DashboardHostedSecurityReviewStatus,
        DashboardHostedSessionLifecycleValidation, DashboardHostedSessionLifecycleValidationReport,
        DashboardHostedSessionLifecycleValidationStatus, DashboardHostedSessionValidationReport,
        DashboardHostedSessionValidationStatus, DashboardLoopbackRuntimeProbe,
        DashboardLoopbackRuntimeProbeReport, DashboardLoopbackRuntimeProbeStatus, DashboardPanel,
        DashboardPanelItem, DashboardPanelKind, DashboardRenderRecord, DashboardRenderRequest,
        DashboardRenderer, DashboardServerBinding, DashboardSeverity, DashboardSnapshot,
        DeterministicDashboardRenderer, DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY,
        DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY,
        DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY,
        DASHBOARD_LAST_HOSTED_SESSION_LIFECYCLE_CHECKPOINT_KEY,
        DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY,
        DASHBOARD_LAST_LOOPBACK_RUNTIME_PROBE_CHECKPOINT_KEY, DASHBOARD_LAST_RENDER_CHECKPOINT_KEY,
    };
    use crate::{AppendOnlyAuditJournal, RuntimeMode, SqliteWalStateStore, StateStore};
    use std::{env, fs, path::PathBuf, process};

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

    fn render_record() -> DashboardRenderRecord {
        DeterministicDashboardRenderer
            .render(DashboardRenderRequest {
                config: DashboardBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: DashboardAccessContext::local_render(Some(
                    "local-dashboard-validation".to_owned(),
                )),
                requested_panels: Vec::new(),
                operator_label: Some("local-dashboard-validation".to_owned()),
                rendered_at_ms: 1_700_000_000_600,
            })
            .expect("local dashboard render should succeed")
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
        let DashboardError::ValidationFailed { violations } = error else {
            panic!("expected dashboard validation error");
        };
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
                access: DashboardAccessContext::local_render(Some("local-operator".to_owned())),
                requested_panels: Vec::new(),
                operator_label: Some("local-operator".to_owned()),
                rendered_at_ms: 1_700_000_000_100,
            })
            .expect("local render should succeed");

        assert!(!record.server_started);
        assert!(!record.public_network_exposed);
        assert!(!record.live_controls_enabled);
        assert!(record.access_authorized);
        assert_eq!(
            record.access_authorization_status,
            DashboardAccessAuthorizationStatus::AuthorizedLocalRender
        );
        assert_eq!(record.panels.len(), 1);
    }

    #[test]
    fn renderer_rejects_hosted_sessions_without_external_auth() {
        let renderer = DeterministicDashboardRenderer;
        let error = renderer
            .render(DashboardRenderRequest {
                config: DashboardBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: DashboardAccessContext {
                    source: DashboardAccessSource::BrowserSession,
                    operator_label: Some("browser-operator".to_owned()),
                },
                requested_panels: Vec::new(),
                operator_label: Some("browser-operator".to_owned()),
                rendered_at_ms: 1_700_000_000_150,
            })
            .expect_err("hosted sessions must fail closed without auth");

        let DashboardError::ValidationFailed { violations } = error else {
            panic!("expected dashboard validation error");
        };
        assert!(violations
            .iter()
            .any(|violation| violation.code() == "DASHBOARD_ACCESS_DENIED"));
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
                access: DashboardAccessContext::local_render(None),
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

    #[test]
    fn dashboard_render_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-render");
        let state_path = temp_state_path("dashboard-render");
        let renderer = DeterministicDashboardRenderer;
        let record = renderer
            .render(DashboardRenderRequest {
                config: DashboardBoundaryConfig::default(),
                snapshot: minimal_snapshot(),
                access: DashboardAccessContext::local_render(Some(
                    "local-dashboard-review".to_owned(),
                )),
                requested_panels: Vec::new(),
                operator_label: Some("local-dashboard-review".to_owned()),
                rendered_at_ms: 1_700_000_000_300,
            })
            .expect("local render should succeed");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_dashboard_render_audit(&mut journal, &record, 1_700_000_000_301)
            .expect("dashboard render audit writes");
        let checkpoint =
            persist_dashboard_render_checkpoint(&mut store, &record, 1_700_000_000_302)
                .expect("dashboard render checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, DASHBOARD_LAST_RENDER_CHECKPOINT_KEY);
        assert!(!record.server_started);
        assert!(!record.public_network_exposed);
        assert!(!record.live_controls_enabled);
        assert!(record.access_authorized);
        assert_eq!(
            record.access_authorization_status,
            DashboardAccessAuthorizationStatus::AuthorizedLocalRender
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_RENDER_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("dashboard checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"server_started\":false"));
        assert!(recovered.value.contains("\"public_network_exposed\":false"));
        assert!(recovered.value.contains("\"live_controls_enabled\":false"));
        assert!(recovered.value.contains("\"access_authorized\":true"));
        assert!(recovered
            .value
            .contains("\"access_authorization_status\":\"authorized-local-render\""));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn hosted_dashboard_security_review_requires_csrf_headers_and_rate_limits() {
        let report = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
            review_id: "dashboard-hosted-security-ready".to_owned(),
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
        .expect("complete local hosted-dashboard security policy should review");

        assert_eq!(
            report.status,
            DashboardHostedSecurityReviewStatus::ReadyForLocalReview
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(report.csrf_protection_required);
        assert!(report.csrf_token_rotation_required);
        assert!(report.secure_headers_required);
        assert!(report.clickjacking_protection_required);
        assert!(report.rate_limit_required);
        assert!(report.audit_state_preflight_required);
        assert!(report.session_revocation_required);
        assert!(report.operator_role_review_required);
        assert!(report.read_only_controls_required);
        assert!(!report.server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_security_review_blocks_missing_controls() {
        let report = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
            review_id: "dashboard-hosted-security-blocked".to_owned(),
            authentication_required: true,
            authorization_required: false,
            csrf_protection_required: false,
            csrf_token_rotation_required: false,
            secure_headers_required: false,
            clickjacking_protection_required: false,
            rate_limit_required: false,
            max_requests_per_minute: 0,
            loopback_only_required: false,
            audit_state_preflight_required: true,
            session_revocation_required: true,
            operator_role_review_required: true,
            read_only_controls_required: true,
            public_exposure_requested: true,
            server_start_requested: true,
            live_controls_requested: true,
        })
        .expect("incomplete local hosted-dashboard security policy should produce blocked report");

        assert_eq!(
            report.status,
            DashboardHostedSecurityReviewStatus::BlockedMissingControls
        );
        assert!(report.missing_control_count > 0);
        assert!(!report.csrf_protection_required);
        assert!(!report.secure_headers_required);
        assert!(!report.rate_limit_required);
        assert!(report.audit_state_preflight_required);
        assert!(report.session_revocation_required);
        assert!(report.operator_role_review_required);
        assert!(report.read_only_controls_required);
        assert!(!report.server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_security_review_requires_future_hosting_preconditions() {
        let error = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
            review_id: "dashboard-hosted-security-missing-preconditions".to_owned(),
            authentication_required: true,
            authorization_required: true,
            csrf_protection_required: true,
            csrf_token_rotation_required: true,
            secure_headers_required: true,
            clickjacking_protection_required: true,
            rate_limit_required: true,
            max_requests_per_minute: 60,
            loopback_only_required: true,
            audit_state_preflight_required: false,
            session_revocation_required: false,
            operator_role_review_required: false,
            read_only_controls_required: false,
            public_exposure_requested: false,
            server_start_requested: false,
            live_controls_requested: false,
        })
        .expect_err("hosted dashboard security must require future hosting preconditions");

        let DashboardError::ValidationFailed { violations } = error else {
            panic!("expected hosted dashboard validation failure");
        };
        for expected in [
            "DASHBOARD_HOSTED_AUDIT_STATE_PREFLIGHT_REQUIRED",
            "DASHBOARD_HOSTED_SESSION_REVOCATION_REQUIRED",
            "DASHBOARD_HOSTED_OPERATOR_ROLE_REVIEW_REQUIRED",
            "DASHBOARD_HOSTED_READ_ONLY_CONTROLS_REQUIRED",
        ] {
            assert!(
                violations
                    .iter()
                    .any(|violation| violation.code() == expected),
                "missing expected violation {expected}"
            );
        }
    }

    #[test]
    fn hosted_dashboard_security_review_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-hosted-security-review");
        let state_path = temp_state_path("dashboard-hosted-security-review");
        let report = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
            review_id: "dashboard-hosted-security-audit-state".to_owned(),
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
        .expect("hosted security review should produce local report");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_dashboard_hosted_security_review_audit(&mut journal, &report, 1_700_000_000_401)
                .expect("hosted security review audit writes");
        let checkpoint = persist_dashboard_hosted_security_review_checkpoint(
            &mut store,
            &report,
            1_700_000_000_402,
        )
        .expect("hosted security review checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY
        );
        assert!(!report.server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_HOSTED_SECURITY_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("hosted security review checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"csrf_protection_required\":true"));
        assert!(recovered.value.contains("\"secure_headers_required\":true"));
        assert!(recovered.value.contains("\"rate_limit_required\":true"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn hosted_dashboard_request_preflight_accepts_loopback_auth_csrf_headers_and_rate_limit() {
        let report = preflight_dashboard_hosted_request(&DashboardHostedRequestPreflight {
            preflight_id: "dashboard-hosted-preflight-ready".to_owned(),
            bind_host: "127.0.0.1".to_owned(),
            access_source: DashboardAccessSource::BrowserSession,
            method: DashboardHostedRequestMethod::Post,
            authenticated: true,
            authorized: true,
            csrf_token_present: true,
            csrf_token_valid: true,
            content_security_policy_present: true,
            frame_protection_present: true,
            content_type_options_present: true,
            referrer_policy_present: true,
            requests_in_current_window: 12,
            max_requests_per_minute: 60,
            public_exposure_requested: false,
            server_start_requested: false,
            live_controls_requested: false,
        })
        .expect("complete hosted request preflight should review locally");

        assert_eq!(
            report.status,
            DashboardHostedRequestPreflightStatus::ReadyForLocalReview
        );
        assert!(report.loopback_bind_validated);
        assert!(report.state_changing_request);
        assert!(report.authenticated);
        assert!(report.authorized);
        assert!(report.csrf_validated);
        assert!(report.secure_headers_validated);
        assert!(report.rate_limit_validated);
        assert_eq!(report.missing_control_count, 0);
        assert!(!report.server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_request_preflight_blocks_public_missing_auth_csrf_headers_and_rate_limit() {
        let report = preflight_dashboard_hosted_request(&DashboardHostedRequestPreflight {
            preflight_id: "dashboard-hosted-preflight-blocked".to_owned(),
            bind_host: "0.0.0.0".to_owned(),
            access_source: DashboardAccessSource::RemoteSession,
            method: DashboardHostedRequestMethod::Delete,
            authenticated: false,
            authorized: false,
            csrf_token_present: true,
            csrf_token_valid: false,
            content_security_policy_present: true,
            frame_protection_present: false,
            content_type_options_present: true,
            referrer_policy_present: false,
            requests_in_current_window: 61,
            max_requests_per_minute: 60,
            public_exposure_requested: true,
            server_start_requested: true,
            live_controls_requested: true,
        })
        .expect("unsafe hosted request preflight should produce blocked report");

        assert_eq!(
            report.status,
            DashboardHostedRequestPreflightStatus::BlockedMissingControls
        );
        assert!(!report.loopback_bind_validated);
        assert!(!report.authenticated);
        assert!(!report.authorized);
        assert!(!report.csrf_validated);
        assert!(!report.secure_headers_validated);
        assert!(!report.rate_limit_validated);
        assert!(report.missing_control_count >= 8);
        assert!(!report.server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_request_preflight_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-hosted-request-preflight");
        let state_path = temp_state_path("dashboard-hosted-request-preflight");
        let report = preflight_dashboard_hosted_request(&DashboardHostedRequestPreflight {
            preflight_id: "dashboard-hosted-preflight-audit-state".to_owned(),
            bind_host: "localhost".to_owned(),
            access_source: DashboardAccessSource::BrowserSession,
            method: DashboardHostedRequestMethod::Post,
            authenticated: true,
            authorized: true,
            csrf_token_present: true,
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
        .expect("hosted request preflight should produce local report");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_dashboard_hosted_request_preflight_audit(
            &mut journal,
            &report,
            1_700_000_000_501,
        )
        .expect("hosted request preflight audit writes");
        let checkpoint = persist_dashboard_hosted_request_preflight_checkpoint(
            &mut store,
            &report,
            1_700_000_000_502,
        )
        .expect("hosted request preflight checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY
        );
        assert!(!report.server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_HOSTED_REQUEST_PREFLIGHT_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("hosted request preflight checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"loopback_bind_validated\":true"));
        assert!(recovered.value.contains("\"csrf_validated\":true"));
        assert!(recovered
            .value
            .contains("\"secure_headers_validated\":true"));
        assert!(recovered.value.contains("\"rate_limit_validated\":true"));
        assert!(recovered.value.contains("\"production_ready\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn hosted_dashboard_request_validation_serves_authenticated_loopback_request() {
        let report = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-hosted-request-ready".to_owned(),
            render_record: render_record(),
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
        .expect("authenticated local hosted dashboard request should validate");

        assert_eq!(
            report.status,
            DashboardHostedRequestValidationStatus::ReadyForLocalReview
        );
        assert!(report.loopback_bind_validated);
        assert!(report.bound_port.is_some());
        assert_eq!(report.local_http_status_code, 200);
        assert!(report.authenticated);
        assert!(report.authorized);
        assert!(report.csrf_validated);
        assert!(report.secure_headers_validated);
        assert!(report.rate_limit_validated);
        assert!(report.local_server_started);
        assert!(report.network_request_served);
        assert_eq!(report.response_panel_count, 1);
        let expected_body =
            "snapshot_id=snapshot-001\nruntime_mode=Paper\nproduction_readiness_percent=73\npanel_count=1\npanel=Safety:Safety\nsummary=Live controls are disabled\nitem=Live controls:disabled:Ok\n";
        assert_eq!(
            report.response_body_bytes,
            u64::try_from(expected_body.len()).expect("expected body length fits")
        );
        assert_eq!(
            report.response_body_sha256,
            sha256_hex(expected_body.as_bytes())
        );
        assert_eq!(report.missing_control_count, 0);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_request_validation_blocks_missing_controls_without_serving() {
        let report = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-hosted-request-blocked".to_owned(),
            render_record: render_record(),
            bind_host: "localhost".to_owned(),
            requested_port: 0,
            method: DashboardHostedRequestMethod::Post,
            request_path: "/admin".to_owned(),
            authenticated: false,
            authorized: false,
            csrf_token_present: true,
            csrf_token_valid: false,
            secure_headers_required: false,
            requests_in_current_window: 61,
            max_requests_per_minute: 60,
            public_exposure_requested: false,
            live_controls_requested: false,
        })
        .expect("missing local hosted dashboard request controls should report blocked");

        assert_eq!(
            report.status,
            DashboardHostedRequestValidationStatus::BlockedMissingControls
        );
        assert!(report.loopback_bind_validated);
        assert!(report.bound_port.is_none());
        assert_eq!(report.local_http_status_code, 403);
        assert!(!report.authenticated);
        assert!(!report.authorized);
        assert!(!report.csrf_validated);
        assert!(!report.secure_headers_validated);
        assert!(!report.rate_limit_validated);
        assert!(!report.local_server_started);
        assert!(!report.network_request_served);
        assert_eq!(report.response_body_bytes, 0);
        assert!(report.response_body_sha256.is_empty());
        assert!(report.missing_control_count >= 6);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_request_validation_rejects_side_effect_requests() {
        let error = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-hosted-request-side-effect".to_owned(),
            render_record: render_record(),
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
            public_exposure_requested: true,
            live_controls_requested: true,
        })
        .expect_err("hosted dashboard side-effect requests must fail closed");

        let DashboardError::ValidationFailed { violations } = error else {
            panic!("expected dashboard validation error");
        };
        assert!(violations.iter().any(|violation| {
            violation.code() == "DASHBOARD_HOSTED_REQUEST_SIDE_EFFECT_REQUESTED"
        }));
    }

    #[test]
    fn hosted_dashboard_request_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-hosted-request-validation");
        let state_path = temp_state_path("dashboard-hosted-request-validation");
        let report = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-hosted-request-audit-state".to_owned(),
            render_record: render_record(),
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
        .expect("hosted dashboard one-shot request should validate locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_dashboard_hosted_request_validation_audit(
            &mut journal,
            &report,
            1_700_000_000_601,
        )
        .expect("hosted request validation audit writes");
        let checkpoint = persist_dashboard_hosted_request_validation_checkpoint(
            &mut store,
            &report,
            1_700_000_000_602,
        )
        .expect("hosted request validation checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY
        );
        assert!(report.local_server_started);
        assert!(report.network_request_served);
        assert!(report.response_body_bytes > 0);
        assert!(is_sha256_hex(&report.response_body_sha256));
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_HOSTED_REQUEST_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("hosted request validation checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: DashboardHostedRequestValidationReport =
            serde_json::from_str(&recovered.value).expect("checkpoint report parses");
        assert_eq!(
            recovered_report.status,
            DashboardHostedRequestValidationStatus::ReadyForLocalReview
        );
        assert!(recovered_report.local_server_started);
        assert!(recovered_report.network_request_served);
        assert!(recovered_report.loopback_bind_validated);
        assert_eq!(recovered_report.local_http_status_code, 200);
        assert_eq!(
            recovered_report.response_body_bytes,
            report.response_body_bytes
        );
        assert_eq!(
            recovered_report.response_body_sha256,
            report.response_body_sha256
        );
        assert!(!recovered_report.public_network_exposed);
        assert!(!recovered_report.live_controls_enabled);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn hosted_dashboard_session_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-hosted-session-validation");
        let state_path = temp_state_path("dashboard-hosted-session-validation");
        let render_record = render_record();
        let accepted = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-session-accepted".to_owned(),
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
        .expect("accepted local dashboard request should validate");
        let unauthenticated = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-session-unauthenticated".to_owned(),
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
        .expect("unauthenticated request should be blocked locally");
        let csrf_rejected = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-session-csrf-rejected".to_owned(),
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
        .expect("csrf rejected request should be blocked locally");
        let rate_limited = validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: "dashboard-session-rate-limited".to_owned(),
            render_record,
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
        .expect("rate-limited request should be blocked locally");
        let report = validate_dashboard_hosted_session(
            "dashboard-session-audit-state",
            &[accepted, unauthenticated, csrf_rejected, rate_limited],
        )
        .expect("dashboard session should summarize local controls");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_dashboard_hosted_session_validation_audit(
            &mut journal,
            &report,
            1_700_000_000_701,
        )
        .expect("hosted session validation audit writes");
        let checkpoint = persist_dashboard_hosted_session_validation_checkpoint(
            &mut store,
            &report,
            1_700_000_000_702,
        )
        .expect("hosted session validation checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY
        );
        assert_eq!(
            report.status,
            DashboardHostedSessionValidationStatus::ReadyForLocalReview
        );
        assert_eq!(report.total_request_count, 4);
        assert_eq!(report.accepted_request_count, 1);
        assert_eq!(report.rejected_unauthenticated_count, 1);
        assert_eq!(report.rejected_csrf_count, 1);
        assert_eq!(report.rejected_rate_limited_count, 1);
        assert!(report.local_server_started);
        assert!(report.network_request_served);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_HOSTED_SESSION_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("hosted session validation checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: DashboardHostedSessionValidationReport =
            serde_json::from_str(&recovered.value).expect("checkpoint report parses");
        assert_eq!(
            recovered_report.status,
            DashboardHostedSessionValidationStatus::ReadyForLocalReview
        );
        assert_eq!(recovered_report.validation_ids.len(), 4);
        assert_eq!(recovered_report.rejected_unauthenticated_count, 1);
        assert_eq!(recovered_report.rejected_csrf_count, 1);
        assert_eq!(recovered_report.rejected_rate_limited_count, 1);
        assert!(!recovered_report.public_network_exposed);
        assert!(!recovered_report.live_controls_enabled);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    fn ready_hosted_session_lifecycle_request() -> DashboardHostedSessionLifecycleValidation {
        DashboardHostedSessionLifecycleValidation {
            lifecycle_id: "dashboard-session-lifecycle-ready".to_owned(),
            session_reference: "session-ref-local-001".to_owned(),
            csrf_reference: "csrf-ref-local-001".to_owned(),
            operator_role: "operator-read-only".to_owned(),
            authenticated: true,
            authorized: true,
            csrf_reference_issued: true,
            csrf_reference_scoped: true,
            csrf_reference_rotated: true,
            session_revocation_supported: true,
            session_revoked: false,
            read_only_role: true,
            rate_limit_remaining: 59,
            max_requests_per_minute: 60,
            loopback_only: true,
            public_network_exposed: false,
            live_controls_enabled: false,
            secret_material_present: false,
            persistent_server_started: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 1_700_000_000_731,
        }
    }

    #[test]
    fn hosted_dashboard_session_lifecycle_accepts_non_secret_local_references() {
        let report =
            validate_dashboard_hosted_session_lifecycle(&ready_hosted_session_lifecycle_request())
                .expect("ready hosted dashboard session lifecycle should validate locally");

        assert_eq!(
            report.status,
            DashboardHostedSessionLifecycleValidationStatus::ReadyForLocalReview
        );
        assert!(report.session_reference_recorded);
        assert!(report.csrf_reference_recorded);
        assert!(report.authenticated);
        assert!(report.authorized);
        assert!(report.csrf_lifecycle_validated);
        assert!(report.session_revocation_supported);
        assert!(!report.session_revoked);
        assert!(report.read_only_role);
        assert!(report.rate_limit_validated);
        assert!(report.loopback_only);
        assert_eq!(report.missing_control_count, 0);
        assert!(report.violation_codes.is_empty());
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.secret_material_present);
        assert!(!report.persistent_server_started);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_session_lifecycle_blocks_side_effects_and_missing_controls() {
        let mut request = ready_hosted_session_lifecycle_request();
        request.lifecycle_id = "dashboard-session-lifecycle-blocked".to_owned();
        request.authenticated = false;
        request.csrf_reference_rotated = false;
        request.session_revoked = true;
        request.read_only_role = false;
        request.rate_limit_remaining = 61;
        request.public_network_exposed = true;
        request.live_controls_enabled = true;
        request.secret_material_present = true;
        request.persistent_server_started = true;
        request.production_ready_claimed = true;

        let report = validate_dashboard_hosted_session_lifecycle(&request)
            .expect("blocked hosted dashboard session lifecycle should report locally");

        assert_eq!(
            report.status,
            DashboardHostedSessionLifecycleValidationStatus::BlockedMissingControls
        );
        assert!(!report.authenticated);
        assert!(!report.csrf_lifecycle_validated);
        assert!(report.session_revoked);
        assert!(!report.read_only_role);
        assert!(!report.rate_limit_validated);
        assert!(report.public_network_exposed);
        assert!(report.live_controls_enabled);
        assert!(report.secret_material_present);
        assert!(report.persistent_server_started);
        assert!(!report.production_ready);
        assert!(report.missing_control_count >= 9);
        for expected in [
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_NOT_AUTHENTICATED",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_CSRF_INCOMPLETE",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SESSION_REVOKED",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_ROLE_NOT_READ_ONLY",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_RATE_LIMIT_INVALID",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_PUBLIC_NETWORK_EXPOSED",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_LIVE_CONTROLS_ENABLED",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SECRET_MATERIAL_PRESENT",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_SERVER_STARTED",
            "DASHBOARD_HOSTED_SESSION_LIFECYCLE_PRODUCTION_READY_CLAIMED",
        ] {
            assert!(
                report.violation_codes.iter().any(|code| code == expected),
                "missing expected violation code {expected}"
            );
        }
    }

    #[test]
    fn hosted_dashboard_session_lifecycle_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-hosted-session-lifecycle");
        let state_path = temp_state_path("dashboard-hosted-session-lifecycle");
        let report =
            validate_dashboard_hosted_session_lifecycle(&ready_hosted_session_lifecycle_request())
                .expect("ready hosted dashboard session lifecycle should validate locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_dashboard_hosted_session_lifecycle_audit(
            &mut journal,
            &report,
            1_700_000_000_732,
        )
        .expect("hosted session lifecycle audit writes");
        let checkpoint = persist_dashboard_hosted_session_lifecycle_checkpoint(
            &mut store,
            &report,
            1_700_000_000_733,
        )
        .expect("hosted session lifecycle checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DASHBOARD_LAST_HOSTED_SESSION_LIFECYCLE_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_HOSTED_SESSION_LIFECYCLE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("hosted session lifecycle checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: DashboardHostedSessionLifecycleValidationReport =
            serde_json::from_str(&recovered.value).expect("checkpoint report parses");
        assert_eq!(
            recovered_report.status,
            DashboardHostedSessionLifecycleValidationStatus::ReadyForLocalReview
        );
        assert!(recovered_report.csrf_lifecycle_validated);
        assert!(recovered_report.session_revocation_supported);
        assert!(recovered_report.read_only_role);
        assert!(!recovered_report.secret_material_present);
        assert!(!recovered_report.persistent_server_started);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn dashboard_loopback_runtime_probe_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("dashboard-loopback-runtime-probe");
        let state_path = temp_state_path("dashboard-loopback-runtime-probe");
        let report = validate_dashboard_loopback_runtime_probe(DashboardLoopbackRuntimeProbe {
            probe_id: "dashboard-loopback-runtime-probe".to_owned(),
            render_record: render_record(),
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            request_count: 3,
            public_exposure_requested: false,
            live_controls_requested: false,
        })
        .expect("loopback runtime probe should validate locally");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_dashboard_loopback_runtime_probe_audit(&mut journal, &report, 1_700_000_000_801)
                .expect("loopback runtime probe audit writes");
        let checkpoint = persist_dashboard_loopback_runtime_probe_checkpoint(
            &mut store,
            &report,
            1_700_000_000_802,
        )
        .expect("loopback runtime probe checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DASHBOARD_LAST_LOOPBACK_RUNTIME_PROBE_CHECKPOINT_KEY
        );
        assert_eq!(
            report.status,
            DashboardLoopbackRuntimeProbeStatus::ReadyForLocalReview
        );
        assert!(report.loopback_bind_validated);
        assert_eq!(report.expected_request_count, 3);
        assert_eq!(report.served_request_count, 3);
        assert!(report.all_requests_returned_ok);
        assert!(report.response_digest_consistent);
        assert_eq!(report.missing_control_count, 0);
        assert!(report.bounded_runtime_started);
        assert!(report.bounded_runtime_shutdown);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DASHBOARD_LAST_LOOPBACK_RUNTIME_PROBE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("loopback runtime probe checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_report: DashboardLoopbackRuntimeProbeReport =
            serde_json::from_str(&recovered.value).expect("checkpoint report parses");
        assert_eq!(
            recovered_report.status,
            DashboardLoopbackRuntimeProbeStatus::ReadyForLocalReview
        );
        assert_eq!(recovered_report.served_request_count, 3);
        assert!(recovered_report.bounded_runtime_shutdown);
        assert!(!recovered_report.public_network_exposed);
        assert!(!recovered_report.live_controls_enabled);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn hosted_dashboard_runtime_readiness_review_accepts_local_evidence() {
        let report = review_dashboard_hosted_runtime_readiness(hosted_runtime_readiness_request(
            false, true,
        ))
        .expect("complete local hosted dashboard evidence should review");

        assert_eq!(
            report.status,
            DashboardHostedRuntimeReadinessReviewStatus::ReadyForLocalReview
        );
        assert!(report.security_review_ready);
        assert!(report.request_preflight_ready);
        assert!(report.session_validation_ready);
        assert!(report.accepted_request_validated);
        assert!(report.unauthenticated_rejection_validated);
        assert!(report.csrf_rejection_validated);
        assert!(report.rate_limit_rejection_validated);
        assert!(report.loopback_serving_validated);
        assert!(report.secure_headers_validated);
        assert!(report.remaining_external_evidence_recorded);
        assert_eq!(report.remaining_external_evidence_count, 4);
        assert!(!report.persistent_server_started);
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn hosted_dashboard_runtime_readiness_review_blocks_missing_external_evidence() {
        let report = review_dashboard_hosted_runtime_readiness(hosted_runtime_readiness_request(
            false, false,
        ))
        .expect("missing external evidence should produce blocked review");

        assert_eq!(
            report.status,
            DashboardHostedRuntimeReadinessReviewStatus::Blocked
        );
        assert!(!report.remaining_external_evidence_recorded);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| { code == "DASHBOARD_HOSTED_RUNTIME_REMAINING_EVIDENCE_MISSING" }));
        assert!(!report.public_network_exposed);
        assert!(!report.live_controls_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn hosted_dashboard_runtime_readiness_review_fails_closed_on_side_effect_claims() {
        let report =
            review_dashboard_hosted_runtime_readiness(hosted_runtime_readiness_request(true, true))
                .expect("side-effect claims should produce blocked review");

        assert_eq!(
            report.status,
            DashboardHostedRuntimeReadinessReviewStatus::Blocked
        );
        assert!(report.persistent_server_started);
        assert!(report.public_network_exposed);
        assert!(report.live_controls_enabled);
        assert!(report.production_ready);
        for expected in [
            "DASHBOARD_HOSTED_RUNTIME_PERSISTENT_SERVER_STARTED",
            "DASHBOARD_HOSTED_RUNTIME_PUBLIC_NETWORK_EXPOSED",
            "DASHBOARD_HOSTED_RUNTIME_LIVE_CONTROLS_ENABLED",
            "DASHBOARD_HOSTED_RUNTIME_PRODUCTION_READY_CLAIMED",
        ] {
            assert!(
                report.violation_codes.iter().any(|code| code == expected),
                "missing expected violation code {expected}"
            );
        }
    }

    fn hosted_runtime_readiness_request(
        side_effect_claimed: bool,
        include_remaining_external_evidence: bool,
    ) -> DashboardHostedRuntimeReadinessReviewRequest {
        let security_review = review_dashboard_hosted_security(&DashboardHostedSecurityPolicy {
            review_id: "dashboard-hosted-runtime-security".to_owned(),
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
        .expect("security review builds");
        let request_preflight =
            preflight_dashboard_hosted_request(&DashboardHostedRequestPreflight {
                preflight_id: "dashboard-hosted-runtime-preflight".to_owned(),
                bind_host: "127.0.0.1".to_owned(),
                access_source: DashboardAccessSource::BrowserSession,
                method: DashboardHostedRequestMethod::Post,
                authenticated: true,
                authorized: true,
                csrf_token_present: true,
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
            .expect("request preflight builds");
        let render_record = render_record();
        let accepted = hosted_request_report(
            "dashboard-hosted-runtime-accepted",
            render_record.clone(),
            DashboardHostedRequestMethod::Get,
            true,
            true,
            true,
            1,
        );
        let unauthenticated = hosted_request_report(
            "dashboard-hosted-runtime-unauthenticated",
            render_record.clone(),
            DashboardHostedRequestMethod::Get,
            false,
            false,
            true,
            1,
        );
        let csrf_rejected = hosted_request_report(
            "dashboard-hosted-runtime-csrf",
            render_record.clone(),
            DashboardHostedRequestMethod::Post,
            true,
            true,
            false,
            1,
        );
        let rate_limited = hosted_request_report(
            "dashboard-hosted-runtime-rate-limited",
            render_record,
            DashboardHostedRequestMethod::Get,
            true,
            true,
            true,
            61,
        );
        let session_validation = validate_dashboard_hosted_session(
            "dashboard-hosted-runtime-session",
            &[accepted, unauthenticated, csrf_rejected, rate_limited],
        )
        .expect("hosted session validation builds");
        DashboardHostedRuntimeReadinessReviewRequest {
            review_id: "dashboard-hosted-runtime-readiness".to_owned(),
            security_review,
            request_preflight,
            session_validation,
            remaining_external_evidence: if include_remaining_external_evidence {
                vec![
                    "persistent daemon hosting validation".to_owned(),
                    "browser authentication/session validation".to_owned(),
                    "CSRF and secure-header serving validation".to_owned(),
                    "external dashboard penetration testing".to_owned(),
                ]
            } else {
                Vec::new()
            },
            persistent_server_start_requested: side_effect_claimed,
            public_network_exposure_requested: side_effect_claimed,
            live_controls_requested: side_effect_claimed,
            production_ready_claimed: side_effect_claimed,
        }
    }

    fn hosted_request_report(
        validation_id: &str,
        render_record: DashboardRenderRecord,
        method: DashboardHostedRequestMethod,
        authenticated: bool,
        authorized: bool,
        csrf_token_valid: bool,
        requests_in_current_window: u32,
    ) -> DashboardHostedRequestValidationReport {
        validate_dashboard_hosted_request(DashboardHostedRequestValidation {
            validation_id: validation_id.to_owned(),
            render_record,
            bind_host: "127.0.0.1".to_owned(),
            requested_port: 0,
            method,
            request_path: "/".to_owned(),
            authenticated,
            authorized,
            csrf_token_present: method.is_state_changing(),
            csrf_token_valid,
            secure_headers_required: true,
            requests_in_current_window,
            max_requests_per_minute: 60,
            public_exposure_requested: false,
            live_controls_requested: false,
        })
        .expect("hosted dashboard request report builds")
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
