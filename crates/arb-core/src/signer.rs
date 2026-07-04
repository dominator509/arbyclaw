#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    AppendOnlyAuditJournal, AuditError, AuditEvent, AuditEventKind, AuditRecord, AuditValue,
    DestinationPolicy, ExecutionScope, PolicyDecisionRecord, SecretRef, StateCheckpoint,
    StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};

/// Stable local signer-boundary version.
pub const SIGNER_BOUNDARY_VERSION: &str = "phase-8-local-fail-closed-signer-boundary-v1";

/// Owning subsystem label for signer request checkpoints.
pub const SIGNER_STATE_SUBSYSTEM: &str = "signer";

/// Stable checkpoint key for the latest local signer request record.
pub const SIGNER_LAST_REQUEST_CHECKPOINT_KEY: &str = "signer:last-request";
/// Stable checkpoint key for the latest local signer secret-scope review.
pub const SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY: &str = "signer:last-secret-scope-review";
/// Stable checkpoint key for the latest local signer authorization envelope.
pub const SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY: &str =
    "signer:last-authorization-envelope";

/// Local signer request status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignerRequestStatus {
    /// Request was rejected because policy did not approve the intent.
    RejectedPolicyDenied,
    /// Request destination is not eligible for signer handling.
    RejectedUnauthorizedDestination,
    /// Request was policy-approved, but local signing is unavailable.
    RejectedSignerUnavailable,
}

/// Local signer secret-scope review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignerSecretScopeReviewStatus {
    /// The keystore reference and scope metadata are locally coherent.
    ReadyForLocalReview,
    /// Signing is disabled by reference.
    RejectedDisabledReference,
    /// Signer references must use the local keystore boundary.
    RejectedNonKeystoreReference,
    /// The request is outside the configured signer secret scope.
    RejectedScopeMismatch,
}

/// Local signer runtime isolation review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignerRuntimeIsolationReviewStatus {
    /// Local runtime isolation metadata passed deterministic review.
    ReadyForLocalReview,
    /// Local runtime isolation metadata failed deterministic review.
    Blocked,
}

/// Local signer authorization envelope status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignerAuthorizationEnvelopeStatus {
    /// Local metadata is ready to be reviewed by a future constrained signer.
    ReadyForLocalAuthorization,
    /// Local metadata is incomplete or unsafe for signer handoff.
    Blocked,
}

/// Non-secret signer request made by a future adapter boundary.
///
/// This record intentionally carries references only. It never includes raw
/// calldata, private keys, mnemonics, seed phrases, wallet credentials, or
/// signed payload bytes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerRequest {
    /// Stable request id.
    pub request_id: String,
    /// Intent id requesting a signature.
    pub intent_id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Requested execution scope.
    pub requested_scope: ExecutionScope,
    /// Optional chain reference.
    pub chain: Option<String>,
    /// Destination classification supplied by policy/planner.
    pub destination: DestinationPolicy,
    /// Non-secret payload reference, hash, or local fixture id.
    pub payload_reference: String,
    /// Durable local policy decision summary for the intent.
    pub policy_decision: PolicyDecisionRecord,
    /// Operator-supplied non-secret timestamp.
    pub requested_at_unix_ms: u64,
}

/// Non-secret local signer boundary result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerRequestRecord {
    /// Stable boundary version.
    pub boundary_version: String,
    /// Stable request id.
    pub request_id: String,
    /// Intent id requesting a signature.
    pub intent_id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Requested execution scope.
    pub requested_scope: ExecutionScope,
    /// Optional chain reference.
    pub chain: Option<String>,
    /// Destination classification supplied by policy/planner.
    pub destination: DestinationPolicy,
    /// Non-secret payload reference, hash, or local fixture id.
    pub payload_reference: String,
    /// Local signer request status.
    pub status: SignerRequestStatus,
    /// Whether the supplied policy decision approved the intent.
    pub policy_approved: bool,
    /// Whether the supplied policy decision matched request intent/strategy ids.
    pub policy_decision_matches_request: bool,
    /// Whether the signer request destination is locally authorized.
    pub destination_authorized: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local signer boundary never loads signer material.
    pub signer_material_loaded: bool,
    /// Local signer boundary never signs payloads.
    pub signing_performed: bool,
    /// Local signer boundary never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local signer boundary never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local signer boundary never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub recorded_at_unix_ms: u64,
}

/// Non-secret local signer secret-scope review request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerSecretScopeReviewRequest {
    /// Stable review id.
    pub review_id: String,
    /// Stable signer request id.
    pub request_id: String,
    /// Intent id requesting signer access.
    pub intent_id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Optional chain reference.
    pub chain: Option<String>,
    /// Reference-only signer material location.
    pub signer_reference: SecretRef,
    /// Strategy ids authorized to use this signer reference.
    pub allowed_strategy_ids: Vec<String>,
    /// Optional chain allowlist for this signer reference.
    pub allowed_chains: Vec<String>,
    /// Keystore aliases authorized for signer scope.
    pub allowed_keystore_aliases: Vec<String>,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Non-secret local signer secret-scope review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerSecretScopeReviewReport {
    /// Stable boundary version.
    pub boundary_version: String,
    /// Stable review id.
    pub review_id: String,
    /// Stable signer request id.
    pub request_id: String,
    /// Intent id requesting signer access.
    pub intent_id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Optional chain reference.
    pub chain: Option<String>,
    /// Local signer secret-scope status.
    pub status: SignerSecretScopeReviewStatus,
    /// Reference source label only.
    pub signer_reference_source: String,
    /// Reference alias/name only, never material.
    pub signer_reference_label: String,
    /// Whether the reference is a keystore alias.
    pub signer_reference_is_keystore: bool,
    /// Whether the strategy id is authorized for this signer reference.
    pub strategy_scope_authorized: bool,
    /// Whether the chain is authorized for this signer reference.
    pub chain_scope_authorized: bool,
    /// Whether the keystore alias is authorized for this signer reference.
    pub keystore_alias_authorized: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local scope review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local scope review never decrypts plaintext.
    pub plaintext_decrypted: bool,
    /// Local scope review never signs payloads.
    pub signing_performed: bool,
    /// Local scope review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local scope review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local scope review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Non-secret local signer runtime isolation review request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerRuntimeIsolationReviewRequest {
    /// Stable review id.
    pub review_id: String,
    /// Runtime boundary label.
    pub runtime_boundary_label: String,
    /// Strategy ids allowed to request signer handling.
    pub allowed_strategy_ids: Vec<String>,
    /// Whether the LLM/orchestration layer can directly access signer material.
    pub llm_direct_signer_access: bool,
    /// Whether the LLM/orchestration layer can directly call the signing operation.
    pub llm_direct_signing_call: bool,
    /// Whether plaintext key material is exposed to the runtime boundary.
    pub plaintext_key_material_exposed: bool,
    /// Whether signer material is loaded during this review. Always false here.
    pub signer_material_loaded: bool,
    /// Whether decrypted plaintext is produced during this review. Always false here.
    pub plaintext_decrypted: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether RPC was called. Always false here.
    pub rpc_called: bool,
    /// Whether requests must pass policy before any future signer boundary.
    pub policy_gate_required: bool,
    /// Whether destination authorization is required before any future signer boundary.
    pub destination_allowlist_required: bool,
    /// Whether signer secret scope review is required before any future signer boundary.
    pub secret_scope_review_required: bool,
    /// Whether audit append is required before any future signer boundary.
    pub audit_before_signing_required: bool,
    /// Whether state checkpointing is required before any future signer boundary.
    pub state_checkpoint_required: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Non-secret local signer runtime isolation review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerRuntimeIsolationReviewReport {
    /// Stable boundary version.
    pub boundary_version: String,
    /// Stable review id.
    pub review_id: String,
    /// Runtime boundary label.
    pub runtime_boundary_label: String,
    /// Local review status.
    pub status: SignerRuntimeIsolationReviewStatus,
    /// Whether LLM direct signer access is denied.
    pub llm_signer_access_denied: bool,
    /// Whether plaintext key exposure is denied.
    pub plaintext_key_exposure_denied: bool,
    /// Whether policy/destination/scope gates are required.
    pub policy_destination_scope_required: bool,
    /// Whether audit/state ordering is required before future signing.
    pub audit_state_before_signing_required: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local isolation review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local isolation review never decrypts plaintext.
    pub plaintext_decrypted: bool,
    /// Local isolation review never signs payloads.
    pub signing_performed: bool,
    /// Local isolation review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local isolation review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local isolation review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Non-secret local signer authorization envelope request.
///
/// This is a reference-only pre-signing handoff gate. It never carries raw
/// transaction bytes, signer material, plaintext, signatures, or broadcast
/// handles.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerAuthorizationEnvelopeRequest {
    /// Stable envelope id.
    pub envelope_id: String,
    /// Local signer request record that already failed closed before signing.
    pub signer_request_record: SignerRequestRecord,
    /// Local signer secret-scope review for the same request.
    pub secret_scope_review: SignerSecretScopeReviewReport,
    /// Local signer runtime isolation review.
    pub runtime_isolation_review: SignerRuntimeIsolationReviewReport,
    /// Non-secret transaction simulation, dry-run, or fixture reference.
    pub transaction_simulation_reference: String,
    /// Non-secret nonce plan or sequence reference.
    pub nonce_plan_reference: String,
    /// Non-secret audit record/hash reference that must exist before signing.
    pub pre_sign_audit_reference: String,
    /// Non-secret state checkpoint key/reference that must exist before signing.
    pub pre_sign_state_checkpoint_key: String,
    /// Whether signer material was loaded while creating this envelope. Always false here.
    pub signer_material_loaded: bool,
    /// Whether plaintext was decrypted while creating this envelope. Always false here.
    pub plaintext_decrypted: bool,
    /// Whether signing occurred while creating this envelope. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred while creating this envelope. Always false here.
    pub broadcast_performed: bool,
    /// Whether RPC was called while creating this envelope. Always false here.
    pub rpc_called: bool,
    /// Whether this envelope claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub created_at_unix_ms: u64,
}

/// Non-secret local signer authorization envelope report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignerAuthorizationEnvelopeReport {
    /// Stable boundary version.
    pub boundary_version: String,
    /// Stable envelope id.
    pub envelope_id: String,
    /// Stable signer request id.
    pub request_id: String,
    /// Intent id requesting signer handling.
    pub intent_id: String,
    /// Strategy profile id that produced the intent.
    pub strategy_id: String,
    /// Local envelope status.
    pub status: SignerAuthorizationEnvelopeStatus,
    /// Whether the signer request had policy, matching ids, destination, and fail-closed signer state.
    pub policy_destination_ready: bool,
    /// Whether the signer secret-scope review was ready for local review.
    pub secret_scope_ready: bool,
    /// Whether runtime isolation was ready for local review.
    pub runtime_isolation_ready: bool,
    /// Whether simulation and nonce references were present and sanitized.
    pub transaction_safety_references_ready: bool,
    /// Whether audit and state pre-signing references were present and sanitized.
    pub audit_state_references_ready: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local authorization envelope never loads signer material.
    pub signer_material_loaded: bool,
    /// Local authorization envelope never decrypts plaintext.
    pub plaintext_decrypted: bool,
    /// Local authorization envelope never signs payloads.
    pub signing_performed: bool,
    /// Local authorization envelope never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local authorization envelope never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local authorization envelope never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub created_at_unix_ms: u64,
}

impl SignerRequestRecord {
    /// Validate the record is fail-closed and side-effect free.
    pub fn validate(&self) -> Result<(), StateStoreError> {
        if self.boundary_version != SIGNER_BOUNDARY_VERSION {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer boundary version mismatch".to_owned(),
            });
        }
        if self.request_id.trim().is_empty() || self.intent_id.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer request and intent ids are required".to_owned(),
            });
        }
        if self.strategy_id.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer request strategy id is required".to_owned(),
            });
        }
        if self.payload_reference.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer request payload reference is required".to_owned(),
            });
        }
        if contains_secret_like_text(&self.payload_reference) {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer request payload reference contains secret-like text".to_owned(),
            });
        }
        if self.recorded_at_unix_ms == 0 {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer request timestamp is required".to_owned(),
            });
        }
        if self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.rpc_called
            || self.production_ready
        {
            return Err(StateStoreError::ValidationFailed {
                reason:
                    "local signer boundary must not load signer material, sign, broadcast, call RPC, or claim production readiness"
                        .to_owned(),
            });
        }
        if self.violation_count == 0
            || self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX)
        {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer request records require coherent fail-closed violation codes"
                    .to_owned(),
            });
        }
        match self.status {
            SignerRequestStatus::RejectedPolicyDenied => {
                if self.policy_approved || self.policy_decision_matches_request {
                    return Err(StateStoreError::ValidationFailed {
                        reason: "policy-denied signer records must not report matching approval"
                            .to_owned(),
                    });
                }
            }
            SignerRequestStatus::RejectedUnauthorizedDestination => {
                if self.destination_authorized {
                    return Err(StateStoreError::ValidationFailed {
                        reason:
                            "unauthorized-destination signer records must not report an authorized destination"
                                .to_owned(),
                    });
                }
            }
            SignerRequestStatus::RejectedSignerUnavailable => {
                if !self.policy_approved
                    || !self.policy_decision_matches_request
                    || !self.destination_authorized
                {
                    return Err(StateStoreError::ValidationFailed {
                        reason:
                            "signer-unavailable records require a matching policy-approved decision and authorized destination"
                                .to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl SignerSecretScopeReviewReport {
    /// Validate the report is reference-only, fail-closed, and side-effect free.
    pub fn validate(&self) -> Result<(), StateStoreError> {
        if self.boundary_version != SIGNER_BOUNDARY_VERSION {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer secret-scope boundary version mismatch".to_owned(),
            });
        }
        if self.review_id.trim().is_empty()
            || self.request_id.trim().is_empty()
            || self.intent_id.trim().is_empty()
            || self.strategy_id.trim().is_empty()
        {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer secret-scope review ids are required".to_owned(),
            });
        }
        if self.signer_reference_label.trim().is_empty()
            || contains_secret_like_text(&self.signer_reference_label)
        {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer secret-scope reference label must be non-secret".to_owned(),
            });
        }
        if self.reviewed_at_unix_ms == 0 {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer secret-scope timestamp is required".to_owned(),
            });
        }
        if self.signer_material_loaded
            || self.plaintext_decrypted
            || self.signing_performed
            || self.broadcast_performed
            || self.rpc_called
            || self.production_ready
        {
            return Err(StateStoreError::ValidationFailed {
                reason:
                    "signer secret-scope review must not load material, decrypt plaintext, sign, broadcast, call RPC, or claim production readiness"
                        .to_owned(),
            });
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer secret-scope review violation count mismatch".to_owned(),
            });
        }
        match self.status {
            SignerSecretScopeReviewStatus::ReadyForLocalReview => {
                if self.violation_count != 0
                    || !self.signer_reference_is_keystore
                    || !self.strategy_scope_authorized
                    || !self.chain_scope_authorized
                    || !self.keystore_alias_authorized
                {
                    return Err(StateStoreError::ValidationFailed {
                        reason:
                            "ready signer secret-scope reviews require coherent authorized keystore scope"
                                .to_owned(),
                    });
                }
            }
            SignerSecretScopeReviewStatus::RejectedDisabledReference => {
                if self.signer_reference_is_keystore {
                    return Err(StateStoreError::ValidationFailed {
                        reason: "disabled signer references must not report keystore scope"
                            .to_owned(),
                    });
                }
            }
            SignerSecretScopeReviewStatus::RejectedNonKeystoreReference
            | SignerSecretScopeReviewStatus::RejectedScopeMismatch => {
                if self.violation_count == 0 {
                    return Err(StateStoreError::ValidationFailed {
                        reason: "rejected signer secret-scope reviews require violation codes"
                            .to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl SignerRuntimeIsolationReviewReport {
    /// Validate the runtime isolation report is non-secret and side-effect free.
    pub fn validate(&self) -> Result<(), StateStoreError> {
        if self.boundary_version != SIGNER_BOUNDARY_VERSION {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer runtime isolation boundary version mismatch".to_owned(),
            });
        }
        if self.review_id.trim().is_empty() || self.runtime_boundary_label.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer runtime isolation ids are required".to_owned(),
            });
        }
        if contains_secret_like_text(&self.runtime_boundary_label) {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer runtime isolation label must be non-secret".to_owned(),
            });
        }
        if self.reviewed_at_unix_ms == 0 {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer runtime isolation timestamp is required".to_owned(),
            });
        }
        if self.signer_material_loaded
            || self.plaintext_decrypted
            || self.signing_performed
            || self.broadcast_performed
            || self.rpc_called
            || self.production_ready
        {
            return Err(StateStoreError::ValidationFailed {
                reason:
                    "signer runtime isolation review must not load material, decrypt plaintext, sign, broadcast, call RPC, or claim production readiness"
                        .to_owned(),
            });
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer runtime isolation violation count mismatch".to_owned(),
            });
        }
        match self.status {
            SignerRuntimeIsolationReviewStatus::ReadyForLocalReview => {
                if self.violation_count != 0
                    || !self.llm_signer_access_denied
                    || !self.plaintext_key_exposure_denied
                    || !self.policy_destination_scope_required
                    || !self.audit_state_before_signing_required
                {
                    return Err(StateStoreError::ValidationFailed {
                        reason:
                            "ready signer runtime isolation reports require all local isolation controls"
                                .to_owned(),
                    });
                }
            }
            SignerRuntimeIsolationReviewStatus::Blocked => {
                if self.violation_count == 0 {
                    return Err(StateStoreError::ValidationFailed {
                        reason: "blocked signer runtime isolation reports require violation codes"
                            .to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl SignerAuthorizationEnvelopeReport {
    /// Validate the authorization envelope is reference-only and side-effect free.
    pub fn validate(&self) -> Result<(), StateStoreError> {
        if self.boundary_version != SIGNER_BOUNDARY_VERSION {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer authorization envelope boundary version mismatch".to_owned(),
            });
        }
        if self.envelope_id.trim().is_empty()
            || self.request_id.trim().is_empty()
            || self.intent_id.trim().is_empty()
            || self.strategy_id.trim().is_empty()
        {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer authorization envelope ids are required".to_owned(),
            });
        }
        if self.created_at_unix_ms == 0 {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer authorization envelope timestamp is required".to_owned(),
            });
        }
        if self.signer_material_loaded
            || self.plaintext_decrypted
            || self.signing_performed
            || self.broadcast_performed
            || self.rpc_called
            || self.production_ready
        {
            return Err(StateStoreError::ValidationFailed {
                reason:
                    "signer authorization envelope must not load material, decrypt plaintext, sign, broadcast, call RPC, or claim production readiness"
                        .to_owned(),
            });
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            return Err(StateStoreError::ValidationFailed {
                reason: "signer authorization envelope violation count mismatch".to_owned(),
            });
        }
        match self.status {
            SignerAuthorizationEnvelopeStatus::ReadyForLocalAuthorization => {
                if self.violation_count != 0
                    || !self.policy_destination_ready
                    || !self.secret_scope_ready
                    || !self.runtime_isolation_ready
                    || !self.transaction_safety_references_ready
                    || !self.audit_state_references_ready
                {
                    return Err(StateStoreError::ValidationFailed {
                        reason:
                            "ready signer authorization envelopes require all local pre-signing controls"
                                .to_owned(),
                    });
                }
            }
            SignerAuthorizationEnvelopeStatus::Blocked => {
                if self.violation_count == 0 {
                    return Err(StateStoreError::ValidationFailed {
                        reason: "blocked signer authorization envelopes require violation codes"
                            .to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Evaluate a local signer request without loading keys, signing, broadcasting,
/// or calling RPC.
#[must_use]
pub fn evaluate_local_signer_request(request: &SignerRequest) -> SignerRequestRecord {
    let policy_decision_matches_request = request.policy_decision.intent_id == request.intent_id
        && request.policy_decision.strategy_id == request.strategy_id;
    let policy_approved = request.policy_decision.approved;
    let destination_authorized = signer_destination_authorized(&request.destination);
    let mut violation_codes = Vec::new();

    if request.request_id.trim().is_empty() {
        violation_codes.push("SIGNER_REQUEST_ID_REQUIRED".to_owned());
    }
    if request.intent_id.trim().is_empty() {
        violation_codes.push("SIGNER_INTENT_ID_REQUIRED".to_owned());
    }
    if request.strategy_id.trim().is_empty() {
        violation_codes.push("SIGNER_STRATEGY_ID_REQUIRED".to_owned());
    }
    if request.payload_reference.trim().is_empty() {
        violation_codes.push("SIGNER_PAYLOAD_REFERENCE_REQUIRED".to_owned());
    }
    if contains_secret_like_text(&request.payload_reference) {
        violation_codes.push("SIGNER_PAYLOAD_REFERENCE_SECRET_LIKE".to_owned());
    }
    if request.requested_at_unix_ms == 0 {
        violation_codes.push("SIGNER_REQUEST_TIMESTAMP_REQUIRED".to_owned());
    }
    if !policy_decision_matches_request {
        violation_codes.push("SIGNER_POLICY_DECISION_MISMATCH".to_owned());
    }
    if !policy_approved {
        violation_codes.push("SIGNER_POLICY_APPROVAL_REQUIRED".to_owned());
    }
    if !destination_authorized {
        violation_codes.push("SIGNER_DESTINATION_AUTHORIZATION_REQUIRED".to_owned());
    }

    let status = if !destination_authorized {
        SignerRequestStatus::RejectedUnauthorizedDestination
    } else if policy_approved && policy_decision_matches_request {
        violation_codes.push("SIGNER_IMPLEMENTATION_UNAVAILABLE".to_owned());
        SignerRequestStatus::RejectedSignerUnavailable
    } else {
        SignerRequestStatus::RejectedPolicyDenied
    };

    SignerRequestRecord {
        boundary_version: SIGNER_BOUNDARY_VERSION.to_owned(),
        request_id: request.request_id.clone(),
        intent_id: request.intent_id.clone(),
        strategy_id: request.strategy_id.clone(),
        requested_scope: request.requested_scope,
        chain: request.chain.clone(),
        destination: request.destination.clone(),
        payload_reference: request.payload_reference.clone(),
        status,
        policy_approved,
        policy_decision_matches_request,
        destination_authorized,
        violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
        violation_codes,
        signer_material_loaded: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        production_ready: false,
        recorded_at_unix_ms: request.requested_at_unix_ms,
    }
}

/// Review signer secret scope without loading signer material or decrypting plaintext.
#[must_use]
pub fn review_signer_secret_scope(
    request: &SignerSecretScopeReviewRequest,
) -> SignerSecretScopeReviewReport {
    let (signer_reference_source, signer_reference_label, signer_reference_is_keystore) =
        signer_reference_summary(&request.signer_reference);
    let mut violation_codes = Vec::new();

    if request.review_id.trim().is_empty() {
        violation_codes.push("SIGNER_SECRET_SCOPE_REVIEW_ID_REQUIRED".to_owned());
    }
    if request.request_id.trim().is_empty() {
        violation_codes.push("SIGNER_SECRET_SCOPE_REQUEST_ID_REQUIRED".to_owned());
    }
    if request.intent_id.trim().is_empty() {
        violation_codes.push("SIGNER_SECRET_SCOPE_INTENT_ID_REQUIRED".to_owned());
    }
    if request.strategy_id.trim().is_empty() {
        violation_codes.push("SIGNER_SECRET_SCOPE_STRATEGY_ID_REQUIRED".to_owned());
    }
    if request.reviewed_at_unix_ms == 0 {
        violation_codes.push("SIGNER_SECRET_SCOPE_TIMESTAMP_REQUIRED".to_owned());
    }
    if request.signer_reference.validate_reference().is_err() {
        violation_codes.push("SIGNER_SECRET_SCOPE_REFERENCE_INVALID".to_owned());
    }

    let strategy_scope_authorized = request
        .allowed_strategy_ids
        .iter()
        .any(|strategy_id| strategy_id == &request.strategy_id);
    if !strategy_scope_authorized {
        violation_codes.push("SIGNER_SECRET_SCOPE_STRATEGY_NOT_AUTHORIZED".to_owned());
    }

    let chain_scope_authorized = match &request.chain {
        Some(chain) => request
            .allowed_chains
            .iter()
            .any(|allowed| allowed == chain),
        None => request.allowed_chains.is_empty(),
    };
    if !chain_scope_authorized {
        violation_codes.push("SIGNER_SECRET_SCOPE_CHAIN_NOT_AUTHORIZED".to_owned());
    }

    let keystore_alias_authorized = match &request.signer_reference {
        SecretRef::Keystore { alias } => request
            .allowed_keystore_aliases
            .iter()
            .any(|allowed| allowed == alias),
        _ => false,
    };
    if request.signer_reference.is_disabled() {
        violation_codes.push("SIGNER_SECRET_SCOPE_REFERENCE_DISABLED".to_owned());
    } else if !signer_reference_is_keystore {
        violation_codes.push("SIGNER_SECRET_SCOPE_KEYSTORE_REQUIRED".to_owned());
    } else if !keystore_alias_authorized {
        violation_codes.push("SIGNER_SECRET_SCOPE_KEYSTORE_ALIAS_NOT_AUTHORIZED".to_owned());
    }

    let status = if request.signer_reference.is_disabled() {
        SignerSecretScopeReviewStatus::RejectedDisabledReference
    } else if !signer_reference_is_keystore {
        SignerSecretScopeReviewStatus::RejectedNonKeystoreReference
    } else if strategy_scope_authorized && chain_scope_authorized && keystore_alias_authorized {
        SignerSecretScopeReviewStatus::ReadyForLocalReview
    } else {
        SignerSecretScopeReviewStatus::RejectedScopeMismatch
    };

    SignerSecretScopeReviewReport {
        boundary_version: SIGNER_BOUNDARY_VERSION.to_owned(),
        review_id: request.review_id.clone(),
        request_id: request.request_id.clone(),
        intent_id: request.intent_id.clone(),
        strategy_id: request.strategy_id.clone(),
        chain: request.chain.clone(),
        status,
        signer_reference_source,
        signer_reference_label,
        signer_reference_is_keystore,
        strategy_scope_authorized,
        chain_scope_authorized,
        keystore_alias_authorized,
        violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
        violation_codes,
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        production_ready: false,
        reviewed_at_unix_ms: request.reviewed_at_unix_ms,
    }
}

/// Review local signer runtime isolation without loading keys, signing,
/// broadcasting, or calling RPC.
#[must_use]
pub fn review_signer_runtime_isolation(
    request: &SignerRuntimeIsolationReviewRequest,
) -> SignerRuntimeIsolationReviewReport {
    let mut violation_codes = Vec::new();

    if request.review_id.trim().is_empty() {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_REVIEW_ID_REQUIRED".to_owned());
    }
    if request.runtime_boundary_label.trim().is_empty()
        || contains_secret_like_text(&request.runtime_boundary_label)
    {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_LABEL_INVALID".to_owned());
    }
    if request.allowed_strategy_ids.is_empty()
        || request
            .allowed_strategy_ids
            .iter()
            .any(|strategy_id| strategy_id.trim().is_empty())
    {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_STRATEGY_SCOPE_REQUIRED".to_owned());
    }
    if request.reviewed_at_unix_ms == 0 {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_TIMESTAMP_REQUIRED".to_owned());
    }
    if request.llm_direct_signer_access {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_LLM_SIGNER_ACCESS".to_owned());
    }
    if request.llm_direct_signing_call {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_LLM_SIGNING_CALL".to_owned());
    }
    if request.plaintext_key_material_exposed {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_PLAINTEXT_EXPOSED".to_owned());
    }
    if !request.policy_gate_required {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_POLICY_GATE_REQUIRED".to_owned());
    }
    if !request.destination_allowlist_required {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_DESTINATION_REQUIRED".to_owned());
    }
    if !request.secret_scope_review_required {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_SECRET_SCOPE_REQUIRED".to_owned());
    }
    if !request.audit_before_signing_required {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_AUDIT_BEFORE_SIGNING_REQUIRED".to_owned());
    }
    if !request.state_checkpoint_required {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_STATE_CHECKPOINT_REQUIRED".to_owned());
    }
    if request.signer_material_loaded
        || request.plaintext_decrypted
        || request.signing_performed
        || request.broadcast_performed
        || request.rpc_called
        || request.production_ready
    {
        violation_codes.push("SIGNER_RUNTIME_ISOLATION_SIDE_EFFECT_FLAG".to_owned());
    }

    let llm_signer_access_denied =
        !request.llm_direct_signer_access && !request.llm_direct_signing_call;
    let plaintext_key_exposure_denied = !request.plaintext_key_material_exposed;
    let policy_destination_scope_required = request.policy_gate_required
        && request.destination_allowlist_required
        && request.secret_scope_review_required;
    let audit_state_before_signing_required =
        request.audit_before_signing_required && request.state_checkpoint_required;
    let status = if violation_codes.is_empty() {
        SignerRuntimeIsolationReviewStatus::ReadyForLocalReview
    } else {
        SignerRuntimeIsolationReviewStatus::Blocked
    };

    SignerRuntimeIsolationReviewReport {
        boundary_version: SIGNER_BOUNDARY_VERSION.to_owned(),
        review_id: request.review_id.clone(),
        runtime_boundary_label: request.runtime_boundary_label.clone(),
        status,
        llm_signer_access_denied,
        plaintext_key_exposure_denied,
        policy_destination_scope_required,
        audit_state_before_signing_required,
        violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
        violation_codes,
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        production_ready: false,
        reviewed_at_unix_ms: request.reviewed_at_unix_ms,
    }
}

/// Build a local signer authorization envelope without loading keys, signing,
/// broadcasting, or calling RPC.
#[must_use]
pub fn build_local_signer_authorization_envelope(
    request: &SignerAuthorizationEnvelopeRequest,
) -> SignerAuthorizationEnvelopeReport {
    let mut violation_codes = Vec::new();

    if request.envelope_id.trim().is_empty() {
        violation_codes.push("SIGNER_AUTHORIZATION_ENVELOPE_ID_REQUIRED".to_owned());
    }
    if request.created_at_unix_ms == 0 {
        violation_codes.push("SIGNER_AUTHORIZATION_ENVELOPE_TIMESTAMP_REQUIRED".to_owned());
    }
    if request.signer_request_record.validate().is_err() {
        violation_codes.push("SIGNER_AUTHORIZATION_REQUEST_RECORD_INVALID".to_owned());
    }
    if request.secret_scope_review.validate().is_err() {
        violation_codes.push("SIGNER_AUTHORIZATION_SECRET_SCOPE_INVALID".to_owned());
    }
    if request.runtime_isolation_review.validate().is_err() {
        violation_codes.push("SIGNER_AUTHORIZATION_RUNTIME_ISOLATION_INVALID".to_owned());
    }
    if !signer_authorization_ids_match(request) {
        violation_codes.push("SIGNER_AUTHORIZATION_ID_MISMATCH".to_owned());
    }

    let policy_destination_ready = request.signer_request_record.status
        == SignerRequestStatus::RejectedSignerUnavailable
        && request.signer_request_record.policy_approved
        && request
            .signer_request_record
            .policy_decision_matches_request
        && request.signer_request_record.destination_authorized;
    if !policy_destination_ready {
        violation_codes.push("SIGNER_AUTHORIZATION_POLICY_DESTINATION_REQUIRED".to_owned());
    }

    let secret_scope_ready =
        request.secret_scope_review.status == SignerSecretScopeReviewStatus::ReadyForLocalReview;
    if !secret_scope_ready {
        violation_codes.push("SIGNER_AUTHORIZATION_SECRET_SCOPE_REQUIRED".to_owned());
    }

    let runtime_isolation_ready = request.runtime_isolation_review.status
        == SignerRuntimeIsolationReviewStatus::ReadyForLocalReview;
    if !runtime_isolation_ready {
        violation_codes.push("SIGNER_AUTHORIZATION_RUNTIME_ISOLATION_REQUIRED".to_owned());
    }

    let transaction_safety_references_ready =
        reference_is_non_secret(&request.transaction_simulation_reference)
            && reference_is_non_secret(&request.nonce_plan_reference);
    if !transaction_safety_references_ready {
        violation_codes.push("SIGNER_AUTHORIZATION_TRANSACTION_REFERENCES_REQUIRED".to_owned());
    }

    let audit_state_references_ready = reference_is_non_secret(&request.pre_sign_audit_reference)
        && reference_is_non_secret(&request.pre_sign_state_checkpoint_key);
    if !audit_state_references_ready {
        violation_codes.push("SIGNER_AUTHORIZATION_AUDIT_STATE_REFERENCES_REQUIRED".to_owned());
    }

    if request.signer_material_loaded
        || request.plaintext_decrypted
        || request.signing_performed
        || request.broadcast_performed
        || request.rpc_called
        || request.production_ready
    {
        violation_codes.push("SIGNER_AUTHORIZATION_SIDE_EFFECT_FLAG".to_owned());
    }

    let status = if violation_codes.is_empty() {
        SignerAuthorizationEnvelopeStatus::ReadyForLocalAuthorization
    } else {
        SignerAuthorizationEnvelopeStatus::Blocked
    };

    SignerAuthorizationEnvelopeReport {
        boundary_version: SIGNER_BOUNDARY_VERSION.to_owned(),
        envelope_id: request.envelope_id.clone(),
        request_id: request.signer_request_record.request_id.clone(),
        intent_id: request.signer_request_record.intent_id.clone(),
        strategy_id: request.signer_request_record.strategy_id.clone(),
        status,
        policy_destination_ready,
        secret_scope_ready,
        runtime_isolation_ready,
        transaction_safety_references_ready,
        audit_state_references_ready,
        violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
        violation_codes,
        signer_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        broadcast_performed: false,
        rpc_called: false,
        production_ready: false,
        created_at_unix_ms: request.created_at_unix_ms,
    }
}

fn signer_reference_summary(reference: &SecretRef) -> (String, String, bool) {
    match reference {
        SecretRef::Env { name } => ("env".to_owned(), name.clone(), false),
        SecretRef::Keystore { alias } => ("keystore".to_owned(), alias.clone(), true),
        SecretRef::Disabled => ("disabled".to_owned(), "disabled".to_owned(), false),
    }
}

fn signer_destination_authorized(destination: &DestinationPolicy) -> bool {
    match destination {
        DestinationPolicy::None | DestinationPolicy::InternalAccount => true,
        DestinationPolicy::ApprovedAddress { chain, label } => {
            !chain.trim().is_empty() && !label.trim().is_empty()
        }
        DestinationPolicy::UnknownAddress { .. } | DestinationPolicy::LlmGenerated => false,
    }
}

fn signer_authorization_ids_match(request: &SignerAuthorizationEnvelopeRequest) -> bool {
    request.signer_request_record.request_id == request.secret_scope_review.request_id
        && request.signer_request_record.intent_id == request.secret_scope_review.intent_id
        && request.signer_request_record.strategy_id == request.secret_scope_review.strategy_id
}

fn reference_is_non_secret(reference: &str) -> bool {
    !reference.trim().is_empty() && !contains_secret_like_text(reference)
}

/// Persist the latest local signer request through the typed state boundary.
pub fn persist_signer_request_checkpoint(
    store: &mut impl StateStore,
    record: &SignerRequestRecord,
) -> Result<StateCheckpoint, StateStoreError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: SIGNER_LAST_REQUEST_CHECKPOINT_KEY.to_owned(),
        subsystem: SIGNER_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize signer request record: {error}"),
        })?,
        updated_at_unix_ms: record.recorded_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Persist the latest local signer secret-scope review through the typed state boundary.
pub fn persist_signer_secret_scope_review_checkpoint(
    store: &mut impl StateStore,
    report: &SignerSecretScopeReviewReport,
) -> Result<StateCheckpoint, StateStoreError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: SIGNER_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize signer secret-scope review: {error}"),
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Persist the latest local signer authorization envelope through the typed state boundary.
pub fn persist_signer_authorization_envelope_checkpoint(
    store: &mut impl StateStore,
    report: &SignerAuthorizationEnvelopeReport,
) -> Result<StateCheckpoint, StateStoreError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY.to_owned(),
        subsystem: SIGNER_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize signer authorization envelope: {error}"),
        })?,
        updated_at_unix_ms: report.created_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Append one local signer request summary to the append-only audit journal.
pub fn append_signer_request_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &SignerRequestRecord,
) -> Result<AuditRecord, AuditError> {
    record
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "SIGNER_REQUEST_RECORD_INVALID",
                error.to_string(),
            )],
        })?;

    let event = AuditEvent::new(
        format!("signer-request-{}", record.request_id),
        AuditEventKind::SecurityAlert,
        SIGNER_STATE_SUBSYSTEM,
        "local-signer-boundary",
        "local signer request rejected without key access, signing, broadcast, or RPC",
    )
    .with_metadata(
        "boundary_version",
        AuditValue::Text(record.boundary_version.clone()),
    )
    .with_metadata("request_id", AuditValue::Text(record.request_id.clone()))
    .with_metadata("intent_id", AuditValue::Text(record.intent_id.clone()))
    .with_metadata("strategy_id", AuditValue::Text(record.strategy_id.clone()))
    .with_metadata(
        "requested_scope",
        AuditValue::Text(format!("{:?}", record.requested_scope)),
    )
    .with_metadata("status", AuditValue::Text(format!("{:?}", record.status)))
    .with_metadata("policy_approved", AuditValue::Bool(record.policy_approved))
    .with_metadata(
        "policy_decision_matches_request",
        AuditValue::Bool(record.policy_decision_matches_request),
    )
    .with_metadata(
        "destination_authorized",
        AuditValue::Bool(record.destination_authorized),
    )
    .with_metadata(
        "violation_count",
        AuditValue::Unsigned(record.violation_count),
    )
    .with_metadata(
        "signer_material_loaded",
        AuditValue::Bool(record.signer_material_loaded),
    )
    .with_metadata(
        "signing_performed",
        AuditValue::Bool(record.signing_performed),
    )
    .with_metadata(
        "broadcast_performed",
        AuditValue::Bool(record.broadcast_performed),
    )
    .with_metadata("rpc_called", AuditValue::Bool(record.rpc_called))
    .with_metadata(
        "production_ready",
        AuditValue::Bool(record.production_ready),
    );

    journal.append_event(event)
}

/// Append one local signer authorization envelope to the append-only audit journal.
pub fn append_signer_authorization_envelope_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &SignerAuthorizationEnvelopeReport,
) -> Result<AuditRecord, AuditError> {
    report
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "SIGNER_AUTHORIZATION_ENVELOPE_INVALID",
                error.to_string(),
            )],
        })?;

    let event = AuditEvent::new(
        format!("signer-authorization-envelope-{}", report.envelope_id),
        AuditEventKind::SecurityAlert,
        SIGNER_STATE_SUBSYSTEM,
        "local-signer-authorization-envelope",
        "local signer authorization envelope reviewed without key access, plaintext, signing, broadcast, or RPC",
    )
    .with_metadata(
        "boundary_version",
        AuditValue::Text(report.boundary_version.clone()),
    )
    .with_metadata("envelope_id", AuditValue::Text(report.envelope_id.clone()))
    .with_metadata("request_id", AuditValue::Text(report.request_id.clone()))
    .with_metadata("intent_id", AuditValue::Text(report.intent_id.clone()))
    .with_metadata("strategy_id", AuditValue::Text(report.strategy_id.clone()))
    .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
    .with_metadata(
        "policy_destination_ready",
        AuditValue::Bool(report.policy_destination_ready),
    )
    .with_metadata(
        "scope_review_ready",
        AuditValue::Bool(report.secret_scope_ready),
    )
    .with_metadata(
        "runtime_isolation_ready",
        AuditValue::Bool(report.runtime_isolation_ready),
    )
    .with_metadata(
        "transaction_safety_references_ready",
        AuditValue::Bool(report.transaction_safety_references_ready),
    )
    .with_metadata(
        "audit_state_references_ready",
        AuditValue::Bool(report.audit_state_references_ready),
    )
    .with_metadata(
        "violation_count",
        AuditValue::Unsigned(report.violation_count),
    )
    .with_metadata(
        "signer_material_loaded",
        AuditValue::Bool(report.signer_material_loaded),
    )
    .with_metadata(
        "plaintext_decrypted",
        AuditValue::Bool(report.plaintext_decrypted),
    )
    .with_metadata(
        "signing_performed",
        AuditValue::Bool(report.signing_performed),
    )
    .with_metadata(
        "broadcast_performed",
        AuditValue::Bool(report.broadcast_performed),
    )
    .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
    .with_metadata(
        "production_ready",
        AuditValue::Bool(report.production_ready),
    );

    journal.append_event(event)
}

/// Append one local signer secret-scope review to the append-only audit journal.
pub fn append_signer_secret_scope_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &SignerSecretScopeReviewReport,
) -> Result<AuditRecord, AuditError> {
    report
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "SIGNER_SECRET_SCOPE_REVIEW_INVALID",
                error.to_string(),
            )],
        })?;

    let event = AuditEvent::new(
        format!("signer-secret-scope-{}", report.review_id),
        AuditEventKind::SecurityAlert,
        SIGNER_STATE_SUBSYSTEM,
        "local-signer-secret-scope",
        "local signer secret scope reviewed without key access, plaintext, signing, broadcast, or RPC",
    )
    .with_metadata(
        "boundary_version",
        AuditValue::Text(report.boundary_version.clone()),
    )
    .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
    .with_metadata("request_id", AuditValue::Text(report.request_id.clone()))
    .with_metadata("intent_id", AuditValue::Text(report.intent_id.clone()))
    .with_metadata("strategy_id", AuditValue::Text(report.strategy_id.clone()))
    .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
    .with_metadata(
        "signer_reference_source",
        AuditValue::Text(report.signer_reference_source.clone()),
    )
    .with_metadata(
        "signer_reference_is_keystore",
        AuditValue::Bool(report.signer_reference_is_keystore),
    )
    .with_metadata(
        "strategy_scope_authorized",
        AuditValue::Bool(report.strategy_scope_authorized),
    )
    .with_metadata(
        "chain_scope_authorized",
        AuditValue::Bool(report.chain_scope_authorized),
    )
    .with_metadata(
        "keystore_alias_authorized",
        AuditValue::Bool(report.keystore_alias_authorized),
    )
    .with_metadata(
        "violation_count",
        AuditValue::Unsigned(report.violation_count),
    )
    .with_metadata(
        "signer_material_loaded",
        AuditValue::Bool(report.signer_material_loaded),
    )
    .with_metadata(
        "plaintext_decrypted",
        AuditValue::Bool(report.plaintext_decrypted),
    )
    .with_metadata(
        "signing_performed",
        AuditValue::Bool(report.signing_performed),
    )
    .with_metadata(
        "broadcast_performed",
        AuditValue::Bool(report.broadcast_performed),
    )
    .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
    .with_metadata(
        "production_ready",
        AuditValue::Bool(report.production_ready),
    );

    journal.append_event(event)
}

fn contains_secret_like_text(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    [
        "api_key=",
        "api-key=",
        "secret=",
        "private_key=",
        "private-key=",
        "seed_phrase=",
        "seed phrase=",
        "mnemonic=",
        "bearer ",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{
        append_signer_authorization_envelope_audit, append_signer_request_audit,
        append_signer_secret_scope_review_audit, build_local_signer_authorization_envelope,
        evaluate_local_signer_request, persist_signer_authorization_envelope_checkpoint,
        persist_signer_request_checkpoint, persist_signer_secret_scope_review_checkpoint,
        review_signer_runtime_isolation, review_signer_secret_scope,
        SignerAuthorizationEnvelopeReport, SignerAuthorizationEnvelopeRequest,
        SignerAuthorizationEnvelopeStatus, SignerRequest, SignerRequestRecord, SignerRequestStatus,
        SignerRuntimeIsolationReviewReport, SignerRuntimeIsolationReviewRequest,
        SignerRuntimeIsolationReviewStatus, SignerSecretScopeReviewReport,
        SignerSecretScopeReviewRequest, SignerSecretScopeReviewStatus,
        SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY, SIGNER_LAST_REQUEST_CHECKPOINT_KEY,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, AuditValue, DestinationPolicy, ExecutionIntent,
        ExecutionIntentKind, ExecutionScope, PolicyDecisionRecord, PolicyEngine, SecretRef,
        SqliteWalStateStore, StateStore, VenueKind, VenueRef,
        SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY,
    };
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    const BASE_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 10.0
max_daily_loss_quote = 2.0
max_open_exposure_quote = 20.0
slippage_bps = 50
gas_fee_cap_quote = 1.0

[venues]
cex_allowlist = ["coinbase"]
dex_allowlist = ["uniswap"]
chain_allowlist = ["ethereum"]
asset_allowlist = ["ETH", "USDC"]

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
    fn signer_boundary_rejects_policy_approved_request_without_signing() {
        let intent = intent();
        let engine = PolicyEngine::from_config(config());
        let decision = engine.evaluate(&intent);
        assert!(decision.is_approved());
        let policy_decision =
            PolicyDecisionRecord::from_decision(&intent, &decision, 1_719_000_000_001);
        let request = SignerRequest {
            request_id: "signer-request-001".to_owned(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            requested_scope: ExecutionScope::Paper,
            chain: Some("ethereum".to_owned()),
            destination: DestinationPolicy::InternalAccount,
            payload_reference: "payload-sha256:abc123".to_owned(),
            policy_decision,
            requested_at_unix_ms: 1_719_000_000_002,
        };

        let record = evaluate_local_signer_request(&request);

        assert_eq!(
            record.status,
            SignerRequestStatus::RejectedSignerUnavailable
        );
        assert!(record.policy_approved);
        assert!(record.policy_decision_matches_request);
        assert!(record.destination_authorized);
        assert!(record
            .violation_codes
            .iter()
            .any(|code| code == "SIGNER_IMPLEMENTATION_UNAVAILABLE"));
        assert!(!record.signer_material_loaded);
        assert!(!record.signing_performed);
        assert!(!record.broadcast_performed);
        assert!(!record.rpc_called);
        assert!(!record.production_ready);
        record.validate().expect("record validates");
    }

    #[test]
    fn signer_boundary_rejects_policy_denied_request() {
        let mut intent = intent();
        intent.scope = ExecutionScope::Live;
        let engine = PolicyEngine::from_config(config());
        let decision = engine.evaluate(&intent);
        assert!(!decision.is_approved());
        let policy_decision =
            PolicyDecisionRecord::from_decision(&intent, &decision, 1_719_000_000_003);
        let request = SignerRequest {
            request_id: "signer-request-002".to_owned(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            requested_scope: ExecutionScope::Live,
            chain: Some("ethereum".to_owned()),
            destination: DestinationPolicy::InternalAccount,
            payload_reference: "payload-sha256:def456".to_owned(),
            policy_decision,
            requested_at_unix_ms: 1_719_000_000_004,
        };

        let record = evaluate_local_signer_request(&request);

        assert_eq!(record.status, SignerRequestStatus::RejectedPolicyDenied);
        assert!(!record.policy_approved);
        assert!(record.destination_authorized);
        assert!(record
            .violation_codes
            .iter()
            .any(|code| code == "SIGNER_POLICY_APPROVAL_REQUIRED"));
        assert!(!record.signing_performed);
        assert!(!record.broadcast_performed);
        assert!(!record.rpc_called);
    }

    #[test]
    fn signer_boundary_rejects_unknown_destination_even_with_matching_policy_record() {
        let intent = intent();
        let engine = PolicyEngine::from_config(config());
        let decision = engine.evaluate(&intent);
        assert!(decision.is_approved());
        let policy_decision =
            PolicyDecisionRecord::from_decision(&intent, &decision, 1_719_000_000_007);
        let request = SignerRequest {
            request_id: "signer-request-unknown-destination".to_owned(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            requested_scope: ExecutionScope::Paper,
            chain: Some("ethereum".to_owned()),
            destination: DestinationPolicy::UnknownAddress {
                chain: "ethereum".to_owned(),
            },
            payload_reference: "payload-sha256:unknown-destination".to_owned(),
            policy_decision,
            requested_at_unix_ms: 1_719_000_000_008,
        };

        let record = evaluate_local_signer_request(&request);

        assert_eq!(
            record.status,
            SignerRequestStatus::RejectedUnauthorizedDestination
        );
        assert!(record.policy_approved);
        assert!(record.policy_decision_matches_request);
        assert!(!record.destination_authorized);
        assert!(record
            .violation_codes
            .iter()
            .any(|code| { code == "SIGNER_DESTINATION_AUTHORIZATION_REQUIRED" }));
        assert!(!record.signer_material_loaded);
        assert!(!record.signing_performed);
        assert!(!record.broadcast_performed);
        assert!(!record.rpc_called);
        record.validate().expect("record validates");
    }

    #[test]
    fn signer_request_audit_and_state_reopen_locally() {
        let record = signer_unavailable_record();
        let audit_path = unique_temp_path("signer-request-audit", "jsonl");
        let state_path = unique_temp_path("signer-request-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_signer_request_audit(&mut journal, &record).expect("audit append succeeds");
        let checkpoint =
            persist_signer_request_checkpoint(&mut store, &record).expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, SIGNER_LAST_REQUEST_CHECKPOINT_KEY);
        assert!(matches!(
            audit_record.event.metadata.get("signing_performed"),
            Some(AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("destination_authorized"),
            Some(AuditValue::Bool(true))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(SIGNER_LAST_REQUEST_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: SignerRequestRecord =
            serde_json::from_str(&checkpoint.value).expect("record deserializes");
        assert_eq!(recovered, record);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn signer_secret_scope_accepts_authorized_keystore_reference_without_loading_material() {
        let report = signer_secret_scope_ready_report();

        assert_eq!(
            report.status,
            SignerSecretScopeReviewStatus::ReadyForLocalReview
        );
        assert!(report.signer_reference_is_keystore);
        assert!(report.strategy_scope_authorized);
        assert!(report.chain_scope_authorized);
        assert!(report.keystore_alias_authorized);
        assert_eq!(report.violation_count, 0);
        assert!(!report.signer_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.rpc_called);
        assert!(!report.production_ready);
        report.validate().expect("ready scope report validates");
    }

    #[test]
    fn signer_secret_scope_rejects_env_reference_without_material_access() {
        let report = review_signer_secret_scope(&SignerSecretScopeReviewRequest {
            review_id: "signer-secret-scope-review-env".to_owned(),
            request_id: "signer-request-env".to_owned(),
            intent_id: "intent-signer-env".to_owned(),
            strategy_id: "strategy-signer".to_owned(),
            chain: Some("ethereum".to_owned()),
            signer_reference: SecretRef::Env {
                name: "ARBYCLAW_SIGNER_REFERENCE".to_owned(),
            },
            allowed_strategy_ids: vec!["strategy-signer".to_owned()],
            allowed_chains: vec!["ethereum".to_owned()],
            allowed_keystore_aliases: vec!["ops-eth-signer".to_owned()],
            reviewed_at_unix_ms: 1_700_000_000_202,
        });

        assert_eq!(
            report.status,
            SignerSecretScopeReviewStatus::RejectedNonKeystoreReference
        );
        assert!(!report.signer_reference_is_keystore);
        assert!(!report.keystore_alias_authorized);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "SIGNER_SECRET_SCOPE_KEYSTORE_REQUIRED"));
        assert!(!report.signer_material_loaded);
        assert!(!report.plaintext_decrypted);
        report.validate().expect("env rejection report validates");
    }

    #[test]
    fn signer_secret_scope_rejects_strategy_chain_or_alias_mismatch() {
        let report = review_signer_secret_scope(&SignerSecretScopeReviewRequest {
            review_id: "signer-secret-scope-review-mismatch".to_owned(),
            request_id: "signer-request-mismatch".to_owned(),
            intent_id: "intent-signer-mismatch".to_owned(),
            strategy_id: "strategy-other".to_owned(),
            chain: Some("polygon".to_owned()),
            signer_reference: SecretRef::Keystore {
                alias: "ops-eth-signer".to_owned(),
            },
            allowed_strategy_ids: vec!["strategy-signer".to_owned()],
            allowed_chains: vec!["ethereum".to_owned()],
            allowed_keystore_aliases: vec!["ops-different-signer".to_owned()],
            reviewed_at_unix_ms: 1_700_000_000_203,
        });

        assert_eq!(
            report.status,
            SignerSecretScopeReviewStatus::RejectedScopeMismatch
        );
        assert!(!report.strategy_scope_authorized);
        assert!(!report.chain_scope_authorized);
        assert!(!report.keystore_alias_authorized);
        assert!(report.violation_count >= 3);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        report.validate().expect("scope mismatch report validates");
    }

    #[test]
    fn signer_runtime_isolation_accepts_no_llm_access_metadata() {
        let report = review_signer_runtime_isolation(&SignerRuntimeIsolationReviewRequest {
            review_id: "signer-runtime-isolation-ready".to_owned(),
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
            reviewed_at_unix_ms: 1_700_000_000_204,
        });

        assert_eq!(
            report.status,
            SignerRuntimeIsolationReviewStatus::ReadyForLocalReview
        );
        assert!(report.llm_signer_access_denied);
        assert!(report.plaintext_key_exposure_denied);
        assert!(report.policy_destination_scope_required);
        assert!(report.audit_state_before_signing_required);
        assert_eq!(report.violation_count, 0);
        assert!(!report.signer_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.rpc_called);
        assert!(!report.production_ready);
        report.validate().expect("isolation report validates");
    }

    #[test]
    fn signer_runtime_isolation_blocks_llm_plaintext_and_missing_gates() {
        let report = review_signer_runtime_isolation(&SignerRuntimeIsolationReviewRequest {
            review_id: "signer-runtime-isolation-blocked".to_owned(),
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
            reviewed_at_unix_ms: 1_700_000_000_205,
        });

        assert_eq!(report.status, SignerRuntimeIsolationReviewStatus::Blocked);
        for expected in [
            "SIGNER_RUNTIME_ISOLATION_STRATEGY_SCOPE_REQUIRED",
            "SIGNER_RUNTIME_ISOLATION_LLM_SIGNER_ACCESS",
            "SIGNER_RUNTIME_ISOLATION_LLM_SIGNING_CALL",
            "SIGNER_RUNTIME_ISOLATION_PLAINTEXT_EXPOSED",
            "SIGNER_RUNTIME_ISOLATION_POLICY_GATE_REQUIRED",
            "SIGNER_RUNTIME_ISOLATION_DESTINATION_REQUIRED",
            "SIGNER_RUNTIME_ISOLATION_SECRET_SCOPE_REQUIRED",
            "SIGNER_RUNTIME_ISOLATION_AUDIT_BEFORE_SIGNING_REQUIRED",
            "SIGNER_RUNTIME_ISOLATION_STATE_CHECKPOINT_REQUIRED",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        assert!(!report.llm_signer_access_denied);
        assert!(!report.plaintext_key_exposure_denied);
        assert!(!report.policy_destination_scope_required);
        assert!(!report.audit_state_before_signing_required);
        report
            .validate()
            .expect("blocked isolation report validates");
    }

    #[test]
    fn signer_runtime_isolation_blocks_side_effect_flags_without_preserving_them() {
        let report = review_signer_runtime_isolation(&SignerRuntimeIsolationReviewRequest {
            review_id: "signer-runtime-isolation-side-effect".to_owned(),
            runtime_boundary_label: "local-signer-boundary".to_owned(),
            allowed_strategy_ids: vec!["strategy-signer".to_owned()],
            llm_direct_signer_access: false,
            llm_direct_signing_call: false,
            plaintext_key_material_exposed: false,
            signer_material_loaded: true,
            plaintext_decrypted: true,
            signing_performed: true,
            broadcast_performed: true,
            rpc_called: true,
            policy_gate_required: true,
            destination_allowlist_required: true,
            secret_scope_review_required: true,
            audit_before_signing_required: true,
            state_checkpoint_required: true,
            production_ready: true,
            reviewed_at_unix_ms: 1_700_000_000_206,
        });

        assert_eq!(report.status, SignerRuntimeIsolationReviewStatus::Blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "SIGNER_RUNTIME_ISOLATION_SIDE_EFFECT_FLAG"));
        assert!(!report.signer_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.rpc_called);
        assert!(!report.production_ready);
        report.validate().expect("side-effect report validates");
    }

    #[test]
    fn signer_authorization_envelope_accepts_complete_local_references_without_signing() {
        let report = signer_authorization_ready_report();

        assert_eq!(
            report.status,
            SignerAuthorizationEnvelopeStatus::ReadyForLocalAuthorization
        );
        assert!(report.policy_destination_ready);
        assert!(report.secret_scope_ready);
        assert!(report.runtime_isolation_ready);
        assert!(report.transaction_safety_references_ready);
        assert!(report.audit_state_references_ready);
        assert_eq!(report.violation_count, 0);
        assert!(!report.signer_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.rpc_called);
        assert!(!report.production_ready);
        report.validate().expect("ready envelope validates");
    }

    #[test]
    fn signer_authorization_envelope_blocks_missing_preconditions() {
        let request = SignerAuthorizationEnvelopeRequest {
            envelope_id: "signer-auth-envelope-blocked".to_owned(),
            signer_request_record: signer_unavailable_record(),
            secret_scope_review: review_signer_secret_scope(&SignerSecretScopeReviewRequest {
                strategy_id: "strategy-other".to_owned(),
                ..signer_secret_scope_request()
            }),
            runtime_isolation_review: review_signer_runtime_isolation(
                &SignerRuntimeIsolationReviewRequest {
                    allowed_strategy_ids: Vec::new(),
                    ..signer_runtime_isolation_request()
                },
            ),
            transaction_simulation_reference: "secret=bad".to_owned(),
            nonce_plan_reference: String::new(),
            pre_sign_audit_reference: String::new(),
            pre_sign_state_checkpoint_key: "invalid-sensitive-locator".to_owned(),
            signer_material_loaded: false,
            plaintext_decrypted: false,
            signing_performed: false,
            broadcast_performed: false,
            rpc_called: false,
            production_ready: false,
            created_at_unix_ms: 1_700_000_000_302,
        };

        let report = build_local_signer_authorization_envelope(&request);

        assert_eq!(report.status, SignerAuthorizationEnvelopeStatus::Blocked);
        for expected in [
            "SIGNER_AUTHORIZATION_SECRET_SCOPE_REQUIRED",
            "SIGNER_AUTHORIZATION_RUNTIME_ISOLATION_REQUIRED",
            "SIGNER_AUTHORIZATION_TRANSACTION_REFERENCES_REQUIRED",
            "SIGNER_AUTHORIZATION_AUDIT_STATE_REFERENCES_REQUIRED",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        assert!(report.policy_destination_ready);
        assert!(!report.secret_scope_ready);
        assert!(!report.runtime_isolation_ready);
        assert!(!report.transaction_safety_references_ready);
        assert!(!report.audit_state_references_ready);
        report.validate().expect("blocked envelope validates");
    }

    #[test]
    fn signer_authorization_envelope_blocks_side_effect_flags_without_preserving_them() {
        let mut request = signer_authorization_ready_request();
        request.envelope_id = "signer-auth-envelope-side-effect".to_owned();
        request.signer_material_loaded = true;
        request.plaintext_decrypted = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.rpc_called = true;
        request.production_ready = true;

        let report = build_local_signer_authorization_envelope(&request);

        assert_eq!(report.status, SignerAuthorizationEnvelopeStatus::Blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "SIGNER_AUTHORIZATION_SIDE_EFFECT_FLAG"));
        assert!(!report.signer_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.rpc_called);
        assert!(!report.production_ready);
        report.validate().expect("side-effect envelope validates");
    }

    #[test]
    fn signer_authorization_envelope_audit_and_state_reopen_locally() {
        let report = signer_authorization_ready_report();
        let audit_path = unique_temp_path("signer-authorization-envelope-audit", "jsonl");
        let state_path = unique_temp_path("signer-authorization-envelope-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_signer_authorization_envelope_audit(&mut journal, &report)
            .expect("audit append succeeds");
        let checkpoint = persist_signer_authorization_envelope_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("signing_performed"),
            Some(AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("audit_state_references_ready"),
            Some(AuditValue::Bool(true))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(SIGNER_LAST_AUTHORIZATION_ENVELOPE_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: SignerAuthorizationEnvelopeReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn signer_secret_scope_audit_and_state_reopen_locally() {
        let report = signer_secret_scope_ready_report();
        let audit_path = unique_temp_path("signer-secret-scope-audit", "jsonl");
        let state_path = unique_temp_path("signer-secret-scope-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_signer_secret_scope_review_audit(&mut journal, &report)
            .expect("audit append succeeds");
        let checkpoint = persist_signer_secret_scope_review_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("signer_reference_is_keystore"),
            Some(AuditValue::Bool(true))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("signer_material_loaded"),
            Some(AuditValue::Bool(false))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(SIGNER_LAST_SECRET_SCOPE_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: SignerSecretScopeReviewReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);
        assert!(!recovered.signer_material_loaded);
        assert!(!recovered.plaintext_decrypted);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    fn signer_unavailable_record() -> SignerRequestRecord {
        let intent = intent();
        let engine = PolicyEngine::from_config(config());
        let decision = engine.evaluate(&intent);
        let policy_decision =
            PolicyDecisionRecord::from_decision(&intent, &decision, 1_719_000_000_005);
        evaluate_local_signer_request(&SignerRequest {
            request_id: "signer-request-003".to_owned(),
            intent_id: intent.id.clone(),
            strategy_id: intent.strategy_id.clone(),
            requested_scope: ExecutionScope::Paper,
            chain: Some("ethereum".to_owned()),
            destination: DestinationPolicy::InternalAccount,
            payload_reference: "payload-sha256:789abc".to_owned(),
            policy_decision,
            requested_at_unix_ms: 1_719_000_000_006,
        })
    }

    fn signer_secret_scope_ready_report() -> SignerSecretScopeReviewReport {
        review_signer_secret_scope(&signer_secret_scope_request())
    }

    fn signer_secret_scope_request() -> SignerSecretScopeReviewRequest {
        SignerSecretScopeReviewRequest {
            review_id: "signer-secret-scope-review-003".to_owned(),
            request_id: "signer-request-003".to_owned(),
            intent_id: "intent-signer-001".to_owned(),
            strategy_id: "strategy-signer".to_owned(),
            chain: Some("ethereum".to_owned()),
            signer_reference: SecretRef::Keystore {
                alias: "ops-eth-signer".to_owned(),
            },
            allowed_strategy_ids: vec!["strategy-signer".to_owned()],
            allowed_chains: vec!["ethereum".to_owned()],
            allowed_keystore_aliases: vec!["ops-eth-signer".to_owned()],
            reviewed_at_unix_ms: 1_700_000_000_211,
        }
    }

    fn signer_runtime_isolation_ready_report() -> SignerRuntimeIsolationReviewReport {
        review_signer_runtime_isolation(&signer_runtime_isolation_request())
    }

    fn signer_runtime_isolation_request() -> SignerRuntimeIsolationReviewRequest {
        SignerRuntimeIsolationReviewRequest {
            review_id: "signer-runtime-isolation-ready".to_owned(),
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
            reviewed_at_unix_ms: 1_700_000_000_301,
        }
    }

    fn signer_authorization_ready_request() -> SignerAuthorizationEnvelopeRequest {
        SignerAuthorizationEnvelopeRequest {
            envelope_id: "signer-auth-envelope-ready".to_owned(),
            signer_request_record: signer_unavailable_record(),
            secret_scope_review: signer_secret_scope_ready_report(),
            runtime_isolation_review: signer_runtime_isolation_ready_report(),
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
            created_at_unix_ms: 1_700_000_000_303,
        }
    }

    fn signer_authorization_ready_report() -> SignerAuthorizationEnvelopeReport {
        build_local_signer_authorization_envelope(&signer_authorization_ready_request())
    }

    fn intent() -> ExecutionIntent {
        ExecutionIntent {
            id: "intent-signer-001".to_owned(),
            strategy_id: "strategy-signer".to_owned(),
            kind: ExecutionIntentKind::CexOrder,
            scope: ExecutionScope::Paper,
            venue: VenueRef {
                name: "coinbase".to_owned(),
                kind: VenueKind::Cex,
            },
            chain: None,
            base_asset: "ETH".to_owned(),
            quote_asset: "USDC".to_owned(),
            notional_quote: 5.0,
            expected_profit_quote: 0.20,
            max_loss_quote: 0.5,
            slippage_bps: 10,
            estimated_fee_quote: 0.03,
            gas_fee_quote: 0.0,
            market_data_age_ms: 1_000,
            destination: DestinationPolicy::InternalAccount,
            requires_signing: false,
        }
    }

    fn config() -> AgentConfig {
        AgentConfig::from_toml_str(BASE_CONFIG).expect("test config validates")
    }

    fn unique_temp_path(label: &str, extension: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("arbyclaw-signer-{label}-{nanos}-{n}.{extension}"))
    }
}
