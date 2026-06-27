#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    AppendOnlyAuditJournal, AuditError, AuditEvent, AuditEventKind, AuditRecord, AuditValue,
    StateCheckpoint, StateStore, StateStoreError,
};
use chacha20poly1305::{
    aead::{Aead, Payload},
    KeyInit, XChaCha20Poly1305, XNonce,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
    sync::atomic::{compiler_fence, Ordering},
};

/// Master key environment variable for local encrypted-keystore access.
pub const KEYRING_MASTER_KEY_ENV: &str = "ARBYCLAW_KEYRING_MASTER_KEY";
/// Optional keystore directory override for encrypted alias loading.
pub const KEYRING_DIR_ENV: &str = "ARBYCLAW_KEYRING_DIR";
/// State-store subsystem name for local secret lifecycle records.
pub const SECRET_LIFECYCLE_STATE_SUBSYSTEM: &str = "secret-lifecycle";
/// State-store key for the latest local secret rotation plan.
pub const SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY: &str = "secret-lifecycle:last-rotation-plan";
/// State-store key for the latest local secret backup/restore review.
pub const SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY: &str =
    "secret-lifecycle:last-backup-restore-review";
const KEYRING_FILE_SUFFIX: &str = "secret";
const KEYRING_DEFAULT_DIR: &str = ".arbyclaw/keyring";
const KEYSTORE_PAYLOAD_VERSION_V1: &str = "v1";
const KEYSTORE_V1_SALT_LEN: usize = 32;
const KEYSTORE_V1_NONCE_LEN: usize = 24;

/// Non-secret reference to sensitive material stored outside repository files.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum SecretRef {
    /// Load the secret from an environment variable name.
    Env { name: String },
    /// Load the secret from a local encrypted keystore alias.
    Keystore { alias: String },
    /// Explicitly disabled secret reference for observe-only configuration.
    Disabled,
}

impl SecretRef {
    /// Validate that a secret reference is a reference only, not raw material.
    pub fn validate_reference(&self) -> Result<(), SecretStoreError> {
        match self {
            Self::Env { name } => validate_reference_name(name, "environment variable"),
            Self::Keystore { alias } => validate_reference_name(alias, "keystore alias"),
            Self::Disabled => Ok(()),
        }
    }

    /// Returns true when no secret source is configured.
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    /// Returns true when the reference is from the keystore backend.
    #[must_use]
    pub const fn is_keystore(&self) -> bool {
        matches!(self, Self::Keystore { .. })
    }
}

fn validate_reference_name(value: &str, label: &'static str) -> Result<(), SecretStoreError> {
    if value.trim().is_empty() {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference cannot be empty"),
        });
    }

    if value.len() > 128 {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference is too long"),
        });
    }

    if value.chars().any(char::is_whitespace) {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference cannot contain whitespace"),
        });
    }

    if value.contains('/') || value.contains('\\') {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference cannot contain path separators"),
        });
    }

    Ok(())
}

/// Sensitive bytes loaded from an approved provider.
///
/// The debug representation is always redacted. Local boundary code clears
/// secret bytes on drop and when requested, and intentionally does not
/// implement `Clone`.
#[derive(PartialEq, Eq)]
pub struct SecretMaterial {
    bytes: Vec<u8>,
}

impl SecretMaterial {
    /// Create secret material from a string value loaded outside config files.
    #[must_use]
    pub fn from_string(value: String) -> Self {
        Self {
            bytes: value.into_bytes(),
        }
    }

    /// Create secret material from bytes loaded outside config files.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Returns true when the loaded secret is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Borrow the raw bytes inside a constrained caller.
    #[must_use]
    pub fn expose_for_constrained_use(&self) -> &[u8] {
        &self.bytes
    }

    /// Overwrite secret bytes in place.
    pub fn clear(&mut self) {
        zeroize_bytes(&mut self.bytes);
    }
}

impl Drop for SecretMaterial {
    fn drop(&mut self) {
        self.clear();
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretMaterial")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

/// Boundary implemented by approved secret providers.
pub trait SecretProvider {
    /// Load secret material by reference only.
    fn load(&self, reference: &SecretRef) -> Result<SecretMaterial, SecretStoreError>;
}

/// Environment-variable provider for local development and CI test contexts.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvSecretProvider;

impl SecretProvider for EnvSecretProvider {
    fn load(&self, reference: &SecretRef) -> Result<SecretMaterial, SecretStoreError> {
        match reference {
            SecretRef::Env { name } => {
                reference.validate_reference()?;
                let value = env::var(name).map_err(|_| SecretStoreError::Unavailable {
                    reference: name.clone(),
                })?;
                let material = SecretMaterial::from_string(value);
                if material.is_empty() {
                    return Err(SecretStoreError::EmptySecret {
                        reference: name.clone(),
                    });
                }
                Ok(material)
            }
            SecretRef::Keystore { .. } => Err(SecretStoreError::ProviderUnavailable {
                reference: reference_label(reference),
                provider: "encrypted keystore",
            }),
            SecretRef::Disabled => Err(SecretStoreError::Disabled),
        }
    }
}

/// Local keystore provider for alias-based secret lookup.
///
/// This reads `<keyring-dir>/<alias>.secret` where file contents are
/// `v1:salt_hex:nonce_hex:ciphertext_hex`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EncryptedKeystoreSecretProvider;

impl SecretProvider for EncryptedKeystoreSecretProvider {
    fn load(&self, reference: &SecretRef) -> Result<SecretMaterial, SecretStoreError> {
        let SecretRef::Keystore { alias } = reference else {
            return Err(SecretStoreError::ProviderUnavailable {
                reference: reference_label(reference),
                provider: "encrypted keystore",
            });
        };
        reference.validate_reference()?;

        let keystore_dir = env::var(KEYRING_DIR_ENV).map_or_else(
            |_| Path::new(KEYRING_DEFAULT_DIR).to_path_buf(),
            PathBuf::from,
        );
        let path = keystore_dir.join(format!("{alias}.{KEYRING_FILE_SUFFIX}"));
        let payload = fs::read_to_string(&path).map_err(|_| SecretStoreError::Unavailable {
            reference: alias.clone(),
        })?;

        let parsed = parse_keystore_payload(payload.trim())?;

        let master_key =
            env::var(KEYRING_MASTER_KEY_ENV).map_err(|_| SecretStoreError::Unavailable {
                reference: KEYRING_MASTER_KEY_ENV.to_owned(),
            })?;
        if master_key.trim().is_empty() {
            return Err(SecretStoreError::InvalidReference {
                reason: "keystore master key cannot be empty".to_owned(),
            });
        }

        let mut master_key_bytes = master_key.into_bytes();
        let key = derive_keystore_key(&master_key_bytes, alias.as_bytes(), &parsed.salt);
        zeroize_bytes(&mut master_key_bytes);
        let cipher = XChaCha20Poly1305::new((&key).into());
        let mut key = key.to_vec();
        let mut plaintext = cipher
            .decrypt(
                XNonce::from_slice(&parsed.nonce),
                Payload {
                    msg: &parsed.ciphertext,
                    aad: keystore_aad(alias).as_bytes(),
                },
            )
            .map_err(|_| SecretStoreError::InvalidReference {
                reason: "keystore ciphertext authentication failed".to_owned(),
            })?;
        zeroize_bytes(&mut key);
        let material = SecretMaterial::from_bytes(std::mem::take(&mut plaintext));
        zeroize_bytes(&mut plaintext);
        if material.is_empty() {
            return Err(SecretStoreError::EmptySecret {
                reference: alias.clone(),
            });
        }
        Ok(material)
    }
}

/// Non-secret local keystore entry preflight request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalKeystoreEntryPreflightRequest {
    /// Keystore alias to inspect by reference only.
    pub alias: String,
    /// Optional local keystore directory override.
    pub keystore_dir: Option<PathBuf>,
    /// Operator-supplied non-secret timestamp.
    pub checked_at_unix_ms: u64,
}

/// Non-secret local keystore entry preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalKeystoreEntryPreflightReport {
    /// Keystore alias inspected by reference only.
    pub alias: String,
    /// Whether the referenced local keystore entry exists.
    pub entry_exists: bool,
    /// Whether the entry payload has `v1:salt:nonce:ciphertext` shape.
    pub payload_shape_valid: bool,
    /// Whether salt and ciphertext are valid hex.
    pub hex_payload_valid: bool,
    /// Salt byte count, not salt contents.
    pub salt_len_bytes: u64,
    /// Nonce byte count, not nonce contents.
    pub nonce_len_bytes: u64,
    /// Ciphertext byte count, not ciphertext contents.
    pub ciphertext_len_bytes: u64,
    /// Whether the entry file is marked read-only by local metadata.
    pub file_marked_readonly: bool,
    /// Whether secret material was loaded. Always false for this preflight.
    pub secret_material_loaded: bool,
    /// Whether plaintext was decrypted. Always false for this preflight.
    pub plaintext_decrypted: bool,
    /// Whether signing was performed. Always false for this preflight.
    pub signing_performed: bool,
    /// Whether this local preflight approves production readiness. Always false.
    pub production_ready: bool,
    /// Stable non-secret validation/denial codes.
    pub validation_codes: Vec<String>,
    /// Operator-supplied non-secret timestamp.
    pub checked_at_unix_ms: u64,
}

/// Local non-secret secret rotation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecretRotationPlanStatus {
    /// Rotation references are locally coherent and ready for operator review.
    ReadyForLocalReview,
    /// Rotation is rejected because current and replacement references are not both keystore aliases.
    RejectedNonKeystoreReference,
    /// Rotation is rejected because current and replacement aliases are the same.
    RejectedSameAlias,
    /// Rotation is rejected because the planned cutover window is invalid.
    RejectedInvalidWindow,
}

/// Local non-secret secret backup/restore review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecretBackupRestoreReviewStatus {
    /// Backup and restore references are coherent and ready for operator review.
    ReadyForLocalReview,
    /// Review is blocked because the backup locator is missing.
    BlockedMissingBackupReference,
    /// Review is blocked because restore verification did not pass.
    BlockedRestoreVerification,
    /// Review is blocked because the review window is invalid.
    BlockedInvalidReviewWindow,
}

/// Non-secret local secret rotation planning request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecretRotationPlanRequest {
    /// Stable plan id.
    pub plan_id: String,
    /// Non-secret operator label for the secret purpose.
    pub secret_purpose: String,
    /// Current reference-only secret location.
    pub current_reference: SecretRef,
    /// Replacement reference-only secret location.
    pub replacement_reference: SecretRef,
    /// Operator who prepared the local plan.
    pub requested_by: String,
    /// Non-secret reason for rotation.
    pub rotation_reason: String,
    /// Operator-supplied non-secret timestamp.
    pub planned_at_unix_ms: u64,
    /// Start of the operator review/cutover window.
    pub not_before_unix_ms: u64,
    /// End of the operator review/cutover window.
    pub expires_at_unix_ms: u64,
}

/// Non-secret local secret rotation planning report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecretRotationPlanReport {
    /// Stable plan id.
    pub plan_id: String,
    /// Non-secret operator label for the secret purpose.
    pub secret_purpose: String,
    /// Local rotation plan status.
    pub status: SecretRotationPlanStatus,
    /// Current reference source label only.
    pub current_reference_source: String,
    /// Current reference label only, never material.
    pub current_reference_label: String,
    /// Replacement reference source label only.
    pub replacement_reference_source: String,
    /// Replacement reference label only, never material.
    pub replacement_reference_label: String,
    /// Whether both references are keystore aliases.
    pub both_references_keystore_aliases: bool,
    /// Whether aliases differ.
    pub aliases_distinct: bool,
    /// Whether the operator review/cutover window is coherent.
    pub rotation_window_valid: bool,
    /// Stable validation/denial codes.
    pub validation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub validation_count: u64,
    /// Local rotation planning never loads secret material.
    pub secret_material_loaded: bool,
    /// Local rotation planning never decrypts plaintext.
    pub plaintext_decrypted: bool,
    /// Local rotation planning never writes keystore entries.
    pub keystore_entry_written: bool,
    /// Local rotation planning never revokes external credentials.
    pub external_secret_revoked: bool,
    /// Local rotation planning never claims production readiness.
    pub production_ready: bool,
    /// Operator who prepared the local plan.
    pub requested_by: String,
    /// Non-secret reason for rotation.
    pub rotation_reason: String,
    /// Operator-supplied non-secret timestamp.
    pub planned_at_unix_ms: u64,
    /// Start of the operator review/cutover window.
    pub not_before_unix_ms: u64,
    /// End of the operator review/cutover window.
    pub expires_at_unix_ms: u64,
}

/// Non-secret local secret backup/restore review request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecretBackupRestoreReviewRequest {
    /// Stable review id.
    pub review_id: String,
    /// Non-secret operator label for the secret purpose.
    pub secret_purpose: String,
    /// Reference-only source secret location.
    pub source_reference: SecretRef,
    /// Sanitized backup locator, such as an artifact name or approved store reference.
    pub backup_reference: String,
    /// Sanitized restore target label, never plaintext or material.
    pub restore_target_label: String,
    /// Operator/reviewer who prepared the local review.
    pub reviewed_by: String,
    /// Non-secret review note.
    pub review_note: String,
    /// Whether the copied backup payload shape was checked without decrypting.
    pub backup_payload_shape_verified: bool,
    /// Whether restore verification passed without exposing plaintext.
    pub restore_verification_passed: bool,
    /// Whether backup and restore locators were recorded as non-secret references only.
    pub references_sanitized: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
    /// Start of the review window.
    pub review_window_start_unix_ms: u64,
    /// End of the review window.
    pub review_window_expires_unix_ms: u64,
}

/// Non-secret local secret backup/restore review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecretBackupRestoreReviewReport {
    /// Stable review id.
    pub review_id: String,
    /// Non-secret operator label for the secret purpose.
    pub secret_purpose: String,
    /// Local backup/restore review status.
    pub status: SecretBackupRestoreReviewStatus,
    /// Source reference label only.
    pub source_reference_source: String,
    /// Source reference label only, never material.
    pub source_reference_label: String,
    /// Sanitized backup locator.
    pub backup_reference: String,
    /// Sanitized restore target label.
    pub restore_target_label: String,
    /// Whether the backup locator is present.
    pub backup_reference_present: bool,
    /// Whether the copied backup payload shape was checked without decrypting.
    pub backup_payload_shape_verified: bool,
    /// Whether restore verification passed without exposing plaintext.
    pub restore_verification_passed: bool,
    /// Whether backup and restore locators were recorded as non-secret references only.
    pub references_sanitized: bool,
    /// Whether the operator review window is coherent.
    pub review_window_valid: bool,
    /// Stable validation/denial codes.
    pub validation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub validation_count: u64,
    /// Local review never loads secret material.
    pub secret_material_loaded: bool,
    /// Local review never decrypts plaintext.
    pub plaintext_decrypted: bool,
    /// Local review never writes keystore entries.
    pub keystore_entry_written: bool,
    /// Local review never restores external credentials.
    pub external_secret_restored: bool,
    /// Local review never signs or broadcasts.
    pub signing_or_broadcast_performed: bool,
    /// Local review never claims production readiness.
    pub production_ready: bool,
    /// Operator/reviewer who prepared the local review.
    pub reviewed_by: String,
    /// Non-secret review note.
    pub review_note: String,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
    /// Start of the review window.
    pub review_window_start_unix_ms: u64,
    /// End of the review window.
    pub review_window_expires_unix_ms: u64,
}

impl LocalKeystoreEntryPreflightRequest {
    /// Validate local preflight request shape.
    pub fn validate(&self) -> Result<(), SecretStoreError> {
        SecretRef::Keystore {
            alias: self.alias.clone(),
        }
        .validate_reference()?;
        if self.checked_at_unix_ms == 0 {
            return Err(SecretStoreError::InvalidReference {
                reason: "keystore preflight timestamp is required".to_owned(),
            });
        }
        Ok(())
    }
}

impl LocalKeystoreEntryPreflightReport {
    /// Validate local preflight report invariants.
    pub fn validate(&self) -> Result<(), SecretStoreError> {
        SecretRef::Keystore {
            alias: self.alias.clone(),
        }
        .validate_reference()?;
        if self.checked_at_unix_ms == 0 {
            return Err(SecretStoreError::InvalidReference {
                reason: "keystore preflight timestamp is required".to_owned(),
            });
        }
        if self.secret_material_loaded
            || self.plaintext_decrypted
            || self.signing_performed
            || self.production_ready
        {
            return Err(SecretStoreError::InvalidReference {
                reason: "keystore preflight must not load material, decrypt plaintext, sign, or claim production readiness".to_owned(),
            });
        }
        if self.entry_exists && self.validation_codes.is_empty() {
            return Err(SecretStoreError::InvalidReference {
                reason: "keystore preflight reports require at least one validation code"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl SecretRotationPlanRequest {
    /// Validate local secret rotation plan request shape.
    pub fn validate(&self) -> Result<(), SecretStoreError> {
        validate_reference_name(&self.plan_id, "secret rotation plan id")?;
        validate_reference_name(&self.secret_purpose, "secret rotation purpose")?;
        validate_reference_name(&self.requested_by, "secret rotation requester")?;
        if self.rotation_reason.trim().is_empty() {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret rotation reason is required".to_owned(),
            });
        }
        self.current_reference.validate_reference()?;
        self.replacement_reference.validate_reference()?;
        if self.planned_at_unix_ms == 0 {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret rotation planned timestamp is required".to_owned(),
            });
        }
        Ok(())
    }
}

impl SecretRotationPlanReport {
    /// Validate local rotation report invariants.
    pub fn validate(&self) -> Result<(), SecretStoreError> {
        validate_reference_name(&self.plan_id, "secret rotation plan id")?;
        validate_reference_name(&self.secret_purpose, "secret rotation purpose")?;
        validate_reference_name(&self.requested_by, "secret rotation requester")?;
        if self.current_reference_label.trim().is_empty()
            || self.replacement_reference_label.trim().is_empty()
        {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret rotation reference labels are required".to_owned(),
            });
        }
        if self.planned_at_unix_ms == 0 {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret rotation planned timestamp is required".to_owned(),
            });
        }
        if self.secret_material_loaded
            || self.plaintext_decrypted
            || self.keystore_entry_written
            || self.external_secret_revoked
            || self.production_ready
        {
            return Err(SecretStoreError::InvalidReference {
                reason:
                    "secret rotation planning must not load material, decrypt plaintext, write keystore entries, revoke external credentials, or claim production readiness"
                        .to_owned(),
            });
        }
        if self.validation_count != u64::try_from(self.validation_codes.len()).unwrap_or(u64::MAX) {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret rotation validation count mismatch".to_owned(),
            });
        }
        match self.status {
            SecretRotationPlanStatus::ReadyForLocalReview => {
                if self.validation_count != 0
                    || !self.both_references_keystore_aliases
                    || !self.aliases_distinct
                    || !self.rotation_window_valid
                {
                    return Err(SecretStoreError::InvalidReference {
                        reason: "ready rotation plans require distinct keystore aliases and a valid cutover window".to_owned(),
                    });
                }
            }
            SecretRotationPlanStatus::RejectedNonKeystoreReference
            | SecretRotationPlanStatus::RejectedSameAlias
            | SecretRotationPlanStatus::RejectedInvalidWindow => {
                if self.validation_count == 0 {
                    return Err(SecretStoreError::InvalidReference {
                        reason: "rejected rotation plans require validation codes".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl SecretBackupRestoreReviewRequest {
    /// Validate local secret backup/restore review request shape.
    pub fn validate(&self) -> Result<(), SecretStoreError> {
        validate_reference_name(&self.review_id, "secret backup restore review id")?;
        validate_reference_name(&self.secret_purpose, "secret backup restore purpose")?;
        validate_reference_name(&self.reviewed_by, "secret backup restore reviewer")?;
        validate_reference_name(
            &self.restore_target_label,
            "secret backup restore target label",
        )?;
        if self.review_note.trim().is_empty() {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret backup restore review note is required".to_owned(),
            });
        }
        self.source_reference.validate_reference()?;
        if self.reviewed_at_unix_ms == 0 {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret backup restore review timestamp is required".to_owned(),
            });
        }
        Ok(())
    }
}

impl SecretBackupRestoreReviewReport {
    /// Validate local backup/restore review invariants.
    pub fn validate(&self) -> Result<(), SecretStoreError> {
        validate_reference_name(&self.review_id, "secret backup restore review id")?;
        validate_reference_name(&self.secret_purpose, "secret backup restore purpose")?;
        validate_reference_name(&self.reviewed_by, "secret backup restore reviewer")?;
        validate_reference_name(
            &self.restore_target_label,
            "secret backup restore target label",
        )?;
        if self.source_reference_label.trim().is_empty() {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret backup restore source reference label is required".to_owned(),
            });
        }
        if self.reviewed_at_unix_ms == 0 {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret backup restore review timestamp is required".to_owned(),
            });
        }
        if self.secret_material_loaded
            || self.plaintext_decrypted
            || self.keystore_entry_written
            || self.external_secret_restored
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            return Err(SecretStoreError::InvalidReference {
                reason:
                    "secret backup restore review must not load material, decrypt plaintext, write keystore entries, restore external credentials, sign, broadcast, or claim production readiness"
                        .to_owned(),
            });
        }
        if self.validation_count != u64::try_from(self.validation_codes.len()).unwrap_or(u64::MAX) {
            return Err(SecretStoreError::InvalidReference {
                reason: "secret backup restore validation count mismatch".to_owned(),
            });
        }
        match self.status {
            SecretBackupRestoreReviewStatus::ReadyForLocalReview => {
                if self.validation_count != 0
                    || !self.backup_reference_present
                    || !self.backup_payload_shape_verified
                    || !self.restore_verification_passed
                    || !self.references_sanitized
                    || !self.review_window_valid
                {
                    return Err(SecretStoreError::InvalidReference {
                        reason: "ready secret backup restore reviews require a sanitized backup reference, verified payload shape, restore verification, and a valid review window".to_owned(),
                    });
                }
            }
            SecretBackupRestoreReviewStatus::BlockedMissingBackupReference
            | SecretBackupRestoreReviewStatus::BlockedRestoreVerification
            | SecretBackupRestoreReviewStatus::BlockedInvalidReviewWindow => {
                if self.validation_count == 0 {
                    return Err(SecretStoreError::InvalidReference {
                        reason: "blocked secret backup restore reviews require validation codes"
                            .to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Plan a local secret rotation using references only.
///
/// This does not load material, decrypt plaintext, write keystore entries,
/// revoke external credentials, call providers, sign, broadcast, or approve
/// production readiness.
pub fn plan_local_secret_rotation(
    request: SecretRotationPlanRequest,
) -> Result<SecretRotationPlanReport, SecretStoreError> {
    request.validate()?;
    let (current_reference_source, current_reference_label) =
        secret_reference_summary(&request.current_reference);
    let (replacement_reference_source, replacement_reference_label) =
        secret_reference_summary(&request.replacement_reference);
    let both_references_keystore_aliases =
        request.current_reference.is_keystore() && request.replacement_reference.is_keystore();
    let aliases_distinct = current_reference_label != replacement_reference_label;
    let rotation_window_valid = request.not_before_unix_ms >= request.planned_at_unix_ms
        && request.expires_at_unix_ms > request.not_before_unix_ms;
    let mut validation_codes = Vec::new();

    if !both_references_keystore_aliases {
        validation_codes.push("SECRET_ROTATION_KEYSTORE_REFERENCES_REQUIRED".to_owned());
    }
    if !aliases_distinct {
        validation_codes.push("SECRET_ROTATION_REPLACEMENT_ALIAS_MUST_DIFFER".to_owned());
    }
    if !rotation_window_valid {
        validation_codes.push("SECRET_ROTATION_WINDOW_INVALID".to_owned());
    }

    let status = if !both_references_keystore_aliases {
        SecretRotationPlanStatus::RejectedNonKeystoreReference
    } else if !aliases_distinct {
        SecretRotationPlanStatus::RejectedSameAlias
    } else if !rotation_window_valid {
        SecretRotationPlanStatus::RejectedInvalidWindow
    } else {
        SecretRotationPlanStatus::ReadyForLocalReview
    };

    let report = SecretRotationPlanReport {
        plan_id: request.plan_id,
        secret_purpose: request.secret_purpose,
        status,
        current_reference_source,
        current_reference_label,
        replacement_reference_source,
        replacement_reference_label,
        both_references_keystore_aliases,
        aliases_distinct,
        rotation_window_valid,
        validation_count: u64::try_from(validation_codes.len()).unwrap_or(u64::MAX),
        validation_codes,
        secret_material_loaded: false,
        plaintext_decrypted: false,
        keystore_entry_written: false,
        external_secret_revoked: false,
        production_ready: false,
        requested_by: request.requested_by,
        rotation_reason: request.rotation_reason,
        planned_at_unix_ms: request.planned_at_unix_ms,
        not_before_unix_ms: request.not_before_unix_ms,
        expires_at_unix_ms: request.expires_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Review a local secret backup/restore operation using non-secret references only.
///
/// This does not load material, decrypt plaintext, write keystore entries,
/// restore external credentials, call providers, sign, broadcast, or approve
/// production readiness.
pub fn review_local_secret_backup_restore(
    request: SecretBackupRestoreReviewRequest,
) -> Result<SecretBackupRestoreReviewReport, SecretStoreError> {
    request.validate()?;
    let (source_reference_source, source_reference_label) =
        secret_reference_summary(&request.source_reference);
    let backup_reference_present = !request.backup_reference.trim().is_empty();
    let review_window_valid = request.review_window_start_unix_ms >= request.reviewed_at_unix_ms
        && request.review_window_expires_unix_ms > request.review_window_start_unix_ms;
    let mut validation_codes = Vec::new();

    if !backup_reference_present {
        validation_codes.push("SECRET_BACKUP_REFERENCE_REQUIRED".to_owned());
    }
    if !request.backup_payload_shape_verified {
        validation_codes.push("SECRET_BACKUP_PAYLOAD_SHAPE_UNVERIFIED".to_owned());
    }
    if !request.restore_verification_passed {
        validation_codes.push("SECRET_RESTORE_VERIFICATION_MISSING".to_owned());
    }
    if !request.references_sanitized {
        validation_codes.push("SECRET_BACKUP_RESTORE_REFERENCES_UNSANITIZED".to_owned());
    }
    if !review_window_valid {
        validation_codes.push("SECRET_BACKUP_RESTORE_REVIEW_WINDOW_INVALID".to_owned());
    }

    let status = if !backup_reference_present {
        SecretBackupRestoreReviewStatus::BlockedMissingBackupReference
    } else if !request.restore_verification_passed
        || !request.backup_payload_shape_verified
        || !request.references_sanitized
    {
        SecretBackupRestoreReviewStatus::BlockedRestoreVerification
    } else if !review_window_valid {
        SecretBackupRestoreReviewStatus::BlockedInvalidReviewWindow
    } else {
        SecretBackupRestoreReviewStatus::ReadyForLocalReview
    };

    let report = SecretBackupRestoreReviewReport {
        review_id: request.review_id,
        secret_purpose: request.secret_purpose,
        status,
        source_reference_source,
        source_reference_label,
        backup_reference: request.backup_reference,
        restore_target_label: request.restore_target_label,
        backup_reference_present,
        backup_payload_shape_verified: request.backup_payload_shape_verified,
        restore_verification_passed: request.restore_verification_passed,
        references_sanitized: request.references_sanitized,
        review_window_valid,
        validation_count: u64::try_from(validation_codes.len()).unwrap_or(u64::MAX),
        validation_codes,
        secret_material_loaded: false,
        plaintext_decrypted: false,
        keystore_entry_written: false,
        external_secret_restored: false,
        signing_or_broadcast_performed: false,
        production_ready: false,
        reviewed_by: request.reviewed_by,
        review_note: request.review_note,
        reviewed_at_unix_ms: request.reviewed_at_unix_ms,
        review_window_start_unix_ms: request.review_window_start_unix_ms,
        review_window_expires_unix_ms: request.review_window_expires_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Inspect a local keystore entry by alias without decrypting or loading material.
///
/// The report contains only metadata and validation codes. It does not read the
/// master key, decrypt ciphertext, expose plaintext, sign, broadcast, call RPC,
/// or approve production readiness.
pub fn preflight_local_keystore_entry(
    request: LocalKeystoreEntryPreflightRequest,
) -> Result<LocalKeystoreEntryPreflightReport, SecretStoreError> {
    request.validate()?;
    let keystore_dir = request.keystore_dir.clone().unwrap_or_else(|| {
        env::var(KEYRING_DIR_ENV).map_or_else(
            |_| Path::new(KEYRING_DEFAULT_DIR).to_path_buf(),
            PathBuf::from,
        )
    });
    let path = keystore_dir.join(format!("{}.{}", request.alias, KEYRING_FILE_SUFFIX));
    let mut validation_codes = Vec::new();
    let metadata = fs::metadata(&path).ok();
    let entry_exists = metadata.is_some();
    if entry_exists {
        validation_codes.push("KEYSTORE_ENTRY_EXISTS".to_owned());
    } else {
        validation_codes.push("KEYSTORE_ENTRY_MISSING".to_owned());
    }

    let mut payload_shape_valid = false;
    let mut hex_payload_valid = false;
    let mut salt_len_bytes = 0_u64;
    let mut ciphertext_len_bytes = 0_u64;

    if let Ok(payload) = fs::read_to_string(&path) {
        if let Ok(parsed) = parse_keystore_payload(payload.trim()) {
            payload_shape_valid = true;
            validation_codes.push("KEYSTORE_PAYLOAD_SHAPE_VALID".to_owned());
            salt_len_bytes = u64::try_from(parsed.salt.len()).unwrap_or(u64::MAX);
            let nonce_len_bytes = u64::try_from(parsed.nonce.len()).unwrap_or(u64::MAX);
            ciphertext_len_bytes = u64::try_from(parsed.ciphertext.len()).unwrap_or(u64::MAX);
            if !parsed.ciphertext.is_empty() {
                hex_payload_valid = true;
                validation_codes.push("KEYSTORE_HEX_PAYLOAD_VALID".to_owned());
            } else {
                validation_codes.push("KEYSTORE_HEX_PAYLOAD_EMPTY".to_owned());
            }
            let report = LocalKeystoreEntryPreflightReport {
                alias: request.alias,
                entry_exists,
                payload_shape_valid,
                hex_payload_valid,
                salt_len_bytes,
                nonce_len_bytes,
                ciphertext_len_bytes,
                file_marked_readonly: metadata
                    .is_some_and(|metadata| metadata.permissions().readonly()),
                secret_material_loaded: false,
                plaintext_decrypted: false,
                signing_performed: false,
                production_ready: false,
                validation_codes,
                checked_at_unix_ms: request.checked_at_unix_ms,
            };
            report.validate()?;
            return Ok(report);
        } else if payload.contains(':') {
            validation_codes.push("KEYSTORE_PAYLOAD_SHAPE_INVALID".to_owned());
            let parts = payload.trim().split(':').collect::<Vec<_>>();
            if parts.len() == 2 {
                let salt = decode_hex_bytes("legacy keystore salt", parts[0]);
                let ciphertext = decode_hex_bytes("legacy keystore ciphertext", parts[1]);
                if let (Ok(salt), Ok(ciphertext)) = (salt, ciphertext) {
                    salt_len_bytes = u64::try_from(salt.len()).unwrap_or(u64::MAX);
                    ciphertext_len_bytes = u64::try_from(ciphertext.len()).unwrap_or(u64::MAX);
                    if !salt.is_empty() && !ciphertext.is_empty() {
                        validation_codes.push("KEYSTORE_LEGACY_UNAUTHENTICATED_FORMAT".to_owned());
                    }
                } else {
                    validation_codes.push("KEYSTORE_HEX_PAYLOAD_INVALID".to_owned());
                }
            } else {
                validation_codes.push("KEYSTORE_HEX_PAYLOAD_INVALID".to_owned());
            }
        } else {
            validation_codes.push("KEYSTORE_PAYLOAD_SHAPE_INVALID".to_owned());
        }
    }

    let report = LocalKeystoreEntryPreflightReport {
        alias: request.alias,
        entry_exists,
        payload_shape_valid,
        hex_payload_valid,
        salt_len_bytes,
        nonce_len_bytes: 0,
        ciphertext_len_bytes,
        file_marked_readonly: metadata.is_some_and(|metadata| metadata.permissions().readonly()),
        secret_material_loaded: false,
        plaintext_decrypted: false,
        signing_performed: false,
        production_ready: false,
        validation_codes,
        checked_at_unix_ms: request.checked_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Persist the latest local secret rotation plan through the typed state boundary.
pub fn persist_secret_rotation_plan_checkpoint(
    store: &mut impl StateStore,
    report: &SecretRotationPlanReport,
) -> Result<StateCheckpoint, StateStoreError> {
    report
        .validate()
        .map_err(|error| StateStoreError::ValidationFailed {
            reason: error.to_string(),
        })?;
    let checkpoint = StateCheckpoint {
        key: SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY.to_owned(),
        subsystem: SECRET_LIFECYCLE_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize secret rotation plan: {error}"),
        })?,
        updated_at_unix_ms: report.planned_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Persist the latest local secret backup/restore review through the typed state boundary.
pub fn persist_secret_backup_restore_review_checkpoint(
    store: &mut impl StateStore,
    report: &SecretBackupRestoreReviewReport,
) -> Result<StateCheckpoint, StateStoreError> {
    report
        .validate()
        .map_err(|error| StateStoreError::ValidationFailed {
            reason: error.to_string(),
        })?;
    let checkpoint = StateCheckpoint {
        key: SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: SECRET_LIFECYCLE_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| StateStoreError::BackendFailed {
            reason: format!("failed to serialize secret backup restore review: {error}"),
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    Ok(checkpoint)
}

/// Append a local secret rotation plan to the audit journal.
pub fn append_secret_rotation_plan_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &SecretRotationPlanReport,
) -> Result<AuditRecord, AuditError> {
    report
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "SECRET_ROTATION_PLAN_INVALID",
                error.to_string(),
            )],
        })?;

    let event = AuditEvent::new(
        format!("secret-rotation-{}", report.plan_id),
        AuditEventKind::SecurityAlert,
        SECRET_LIFECYCLE_STATE_SUBSYSTEM,
        "local-secret-rotation-plan",
        "local secret rotation plan recorded without material access or keystore mutation",
    )
    .with_metadata("plan_id", AuditValue::Text(report.plan_id.clone()))
    .with_metadata(
        "rotation_purpose",
        AuditValue::Text(report.secret_purpose.clone()),
    )
    .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
    .with_metadata(
        "current_reference_source",
        AuditValue::Text(report.current_reference_source.clone()),
    )
    .with_metadata(
        "replacement_reference_source",
        AuditValue::Text(report.replacement_reference_source.clone()),
    )
    .with_metadata(
        "both_references_keystore_aliases",
        AuditValue::Bool(report.both_references_keystore_aliases),
    )
    .with_metadata(
        "aliases_distinct",
        AuditValue::Bool(report.aliases_distinct),
    )
    .with_metadata(
        "rotation_window_valid",
        AuditValue::Bool(report.rotation_window_valid),
    )
    .with_metadata(
        "validation_count",
        AuditValue::Unsigned(report.validation_count),
    )
    .with_metadata(
        "material_loaded",
        AuditValue::Bool(report.secret_material_loaded),
    )
    .with_metadata(
        "plaintext_decrypted",
        AuditValue::Bool(report.plaintext_decrypted),
    )
    .with_metadata(
        "keystore_entry_written",
        AuditValue::Bool(report.keystore_entry_written),
    )
    .with_metadata(
        "external_reference_revoked",
        AuditValue::Bool(report.external_secret_revoked),
    )
    .with_metadata(
        "production_ready",
        AuditValue::Bool(report.production_ready),
    );

    journal.append_event(event)
}

/// Append a local secret backup/restore review to the audit journal.
pub fn append_secret_backup_restore_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &SecretBackupRestoreReviewReport,
) -> Result<AuditRecord, AuditError> {
    report
        .validate()
        .map_err(|error| AuditError::ValidationFailed {
            violations: vec![crate::AuditViolation::new_owned(
                "SECRET_BACKUP_RESTORE_REVIEW_INVALID",
                error.to_string(),
            )],
        })?;

    let event = AuditEvent::new(
        format!("secret-backup-restore-{}", report.review_id),
        AuditEventKind::SecurityAlert,
        SECRET_LIFECYCLE_STATE_SUBSYSTEM,
        "local-secret-backup-restore-review",
        "local secret backup restore review recorded without material access or keystore mutation",
    )
    .with_metadata("review_id", AuditValue::Text(report.review_id.clone()))
    .with_metadata(
        "purpose_label",
        AuditValue::Text(report.secret_purpose.clone()),
    )
    .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
    .with_metadata(
        "source_reference_source",
        AuditValue::Text(report.source_reference_source.clone()),
    )
    .with_metadata(
        "backup_reference_present",
        AuditValue::Bool(report.backup_reference_present),
    )
    .with_metadata(
        "backup_payload_shape_verified",
        AuditValue::Bool(report.backup_payload_shape_verified),
    )
    .with_metadata(
        "restore_verification_passed",
        AuditValue::Bool(report.restore_verification_passed),
    )
    .with_metadata(
        "references_sanitized",
        AuditValue::Bool(report.references_sanitized),
    )
    .with_metadata(
        "review_window_valid",
        AuditValue::Bool(report.review_window_valid),
    )
    .with_metadata(
        "validation_count",
        AuditValue::Unsigned(report.validation_count),
    )
    .with_metadata(
        "material_loaded",
        AuditValue::Bool(report.secret_material_loaded),
    )
    .with_metadata(
        "plaintext_decrypted",
        AuditValue::Bool(report.plaintext_decrypted),
    )
    .with_metadata(
        "keystore_entry_written",
        AuditValue::Bool(report.keystore_entry_written),
    )
    .with_metadata(
        "external_reference_restored",
        AuditValue::Bool(report.external_secret_restored),
    )
    .with_metadata(
        "signing_or_broadcast_performed",
        AuditValue::Bool(report.signing_or_broadcast_performed),
    )
    .with_metadata(
        "production_ready",
        AuditValue::Bool(report.production_ready),
    );

    journal.append_event(event)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedKeystorePayload {
    salt: Vec<u8>,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

fn parse_keystore_payload(payload: &str) -> Result<ParsedKeystorePayload, SecretStoreError> {
    let parts = payload.split(':').collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != KEYSTORE_PAYLOAD_VERSION_V1 {
        return Err(SecretStoreError::InvalidReference {
            reason: "keystore entry must be v1:salt:nonce:ciphertext hex".to_owned(),
        });
    }
    let salt = decode_hex_bytes("keystore salt", parts[1])?;
    let nonce = decode_hex_bytes("keystore nonce", parts[2])?;
    let ciphertext = decode_hex_bytes("keystore ciphertext", parts[3])?;
    if salt.len() != KEYSTORE_V1_SALT_LEN {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("keystore salt must be {KEYSTORE_V1_SALT_LEN} bytes"),
        });
    }
    if nonce.len() != KEYSTORE_V1_NONCE_LEN {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("keystore nonce must be {KEYSTORE_V1_NONCE_LEN} bytes"),
        });
    }
    if ciphertext.is_empty() {
        return Err(SecretStoreError::InvalidReference {
            reason: "keystore ciphertext cannot be empty".to_owned(),
        });
    }
    Ok(ParsedKeystorePayload {
        salt,
        nonce,
        ciphertext,
    })
}

fn secret_reference_summary(reference: &SecretRef) -> (String, String) {
    match reference {
        SecretRef::Env { name } => ("env".to_owned(), name.clone()),
        SecretRef::Keystore { alias } => ("keystore".to_owned(), alias.clone()),
        SecretRef::Disabled => ("disabled".to_owned(), "disabled".to_owned()),
    }
}

#[cfg(test)]
fn build_keystore_payload_v1(
    master_key: &[u8],
    alias: &str,
    salt: &[u8],
    nonce: &[u8],
    plaintext: &[u8],
) -> Result<String, SecretStoreError> {
    if salt.len() != KEYSTORE_V1_SALT_LEN {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("keystore salt must be {KEYSTORE_V1_SALT_LEN} bytes"),
        });
    }
    if nonce.len() != KEYSTORE_V1_NONCE_LEN {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("keystore nonce must be {KEYSTORE_V1_NONCE_LEN} bytes"),
        });
    }
    if plaintext.is_empty() {
        return Err(SecretStoreError::EmptySecret {
            reference: alias.to_owned(),
        });
    }
    let key = derive_keystore_key(master_key, alias.as_bytes(), salt);
    let cipher = XChaCha20Poly1305::new((&key).into());
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: plaintext,
                aad: keystore_aad(alias).as_bytes(),
            },
        )
        .map_err(|_| SecretStoreError::InvalidReference {
            reason: "keystore encryption failed".to_owned(),
        })?;
    Ok(format!(
        "{KEYSTORE_PAYLOAD_VERSION_V1}:{}:{}:{}",
        encode_hex_bytes(salt),
        encode_hex_bytes(nonce),
        encode_hex_bytes(&ciphertext)
    ))
}

fn derive_keystore_key(master_key: &[u8], alias: &[u8], salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"arbyclaw-local-keystore-v1");
    hasher.update(master_key);
    hasher.update(alias);
    hasher.update(salt);
    let digest = hasher.finalize();
    let mut key = [0_u8; 32];
    key.copy_from_slice(&digest);
    key
}

fn keystore_aad(alias: &str) -> String {
    format!("arbyclaw-keystore:{alias}:{KEYSTORE_PAYLOAD_VERSION_V1}")
}

#[cfg(test)]
fn encode_hex_bytes(value: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn zeroize_bytes(bytes: &mut [u8]) {
    compiler_fence(Ordering::SeqCst);
    bytes.fill(0);
    compiler_fence(Ordering::SeqCst);
}

fn reference_label(reference: &SecretRef) -> String {
    match reference {
        SecretRef::Env { name } => name.to_owned(),
        SecretRef::Keystore { alias } => alias.to_owned(),
        SecretRef::Disabled => "disabled".to_owned(),
    }
}

fn decode_hex_bytes(label: &str, value: &str) -> Result<Vec<u8>, SecretStoreError> {
    if value.len() % 2 != 0 {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} must have even length"),
        });
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let mut cursor = 0usize;
    while cursor < value.len() {
        let pair = &value[cursor..cursor + 2];
        let byte =
            u8::from_str_radix(pair, 16).map_err(|_| SecretStoreError::InvalidReference {
                reason: format!("{label} contains non-hex characters"),
            })?;
        bytes.push(byte);
        cursor += 2;
    }
    Ok(bytes)
}

/// Errors produced by secret-reference validation and loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretStoreError {
    /// Reference name is invalid.
    InvalidReference { reason: String },
    /// Referenced secret is not available in the provider.
    Unavailable { reference: String },
    /// Referenced secret loaded successfully but was empty.
    EmptySecret { reference: String },
    /// Secret provider has not been implemented or enabled.
    ProviderUnavailable {
        reference: String,
        provider: &'static str,
    },
    /// Secret access is intentionally disabled.
    Disabled,
}

impl fmt::Display for SecretStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReference { reason } => {
                write!(formatter, "invalid secret reference: {reason}")
            }
            Self::Unavailable { reference } => {
                write!(formatter, "secret reference is unavailable: {reference}")
            }
            Self::EmptySecret { reference } => write!(
                formatter,
                "secret reference resolved to an empty value: {reference}"
            ),
            Self::ProviderUnavailable {
                reference,
                provider,
            } => write!(
                formatter,
                "secret provider {provider} is unavailable for reference: {reference}"
            ),
            Self::Disabled => formatter.write_str("secret access is disabled"),
        }
    }
}

impl std::error::Error for SecretStoreError {}

#[cfg(test)]
mod tests {
    use super::{
        append_secret_backup_restore_review_audit, append_secret_rotation_plan_audit,
        decode_hex_bytes, persist_secret_backup_restore_review_checkpoint,
        persist_secret_rotation_plan_checkpoint, plan_local_secret_rotation,
        preflight_local_keystore_entry, review_local_secret_backup_restore,
        EncryptedKeystoreSecretProvider, LocalKeystoreEntryPreflightRequest,
        SecretBackupRestoreReviewReport, SecretBackupRestoreReviewRequest,
        SecretBackupRestoreReviewStatus, SecretMaterial, SecretProvider, SecretRef,
        SecretRotationPlanReport, SecretRotationPlanRequest, SecretRotationPlanStatus,
        SecretStoreError, KEYRING_DIR_ENV, KEYRING_MASTER_KEY_ENV,
    };
    use crate::{
        AppendOnlyAuditJournal, AuditValue, SqliteWalStateStore, StateCheckpoint, StateStore,
        StateStoreError, SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY,
        SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY,
    };
    use std::{
        env,
        fmt::Write,
        fs,
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            Mutex,
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    static KEYSTORE_ENV_LOCK: Mutex<()> = Mutex::new(());
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn encode_hex_bytes(value: &[u8]) -> String {
        let mut encoded = String::with_capacity(value.len() * 2);
        for byte in value {
            let _ = write!(encoded, "{byte:02x}");
        }
        encoded
    }

    fn build_keystore_entry(master_key: &str, alias: &str, plaintext: &[u8]) -> String {
        let salt = [0x42_u8; super::KEYSTORE_V1_SALT_LEN];
        let nonce = [0x24_u8; super::KEYSTORE_V1_NONCE_LEN];
        super::build_keystore_payload_v1(master_key.as_bytes(), alias, &salt, &nonce, plaintext)
            .expect("test keystore payload should encrypt")
    }

    #[test]
    fn secret_material_debug_is_redacted() {
        let material = SecretMaterial::from_string("example-sensitive-value".to_owned());
        let debug = format!("{material:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("example-sensitive-value"));
    }

    #[test]
    fn environment_secret_reference_validates_name_only() {
        let reference = SecretRef::Env {
            name: "ARB_EXAMPLE_REFERENCE".to_owned(),
        };
        assert!(reference.validate_reference().is_ok());
    }

    #[test]
    fn empty_reference_is_rejected() {
        let reference = SecretRef::Keystore {
            alias: String::new(),
        };
        assert!(reference.validate_reference().is_err());
    }

    #[test]
    fn secret_material_can_be_cleared() {
        let mut material = SecretMaterial::from_string("erase-me".to_owned());
        material.clear();
        assert!(material
            .expose_for_constrained_use()
            .iter()
            .all(|byte| *byte == 0));
    }

    #[test]
    fn encrypted_keystore_roundtrips_reference_data() {
        let _guard = KEYSTORE_ENV_LOCK
            .lock()
            .expect("keystore env lock poisoned");
        let root = env::temp_dir().join("arbyclaw-keystore-test");
        let alias = "ops-local-keystore";
        let master_key = "stable-master-key-value";
        let plaintext = b"phase-26-keystore-value";

        let dir = root.join("providers");
        let _ = fs::create_dir_all(&dir);
        let original_dir = env::var(KEYRING_DIR_ENV).ok();
        let original_key = env::var(KEYRING_MASTER_KEY_ENV).ok();

        let payload = build_keystore_entry(master_key, alias, plaintext);
        let secret_path = dir.join(format!("{alias}.secret"));
        fs::write(&secret_path, payload).expect("keystore fixture write");
        let provider = EncryptedKeystoreSecretProvider;

        env::set_var(KEYRING_DIR_ENV, &dir);
        env::set_var(KEYRING_MASTER_KEY_ENV, master_key);
        let loaded = provider
            .load(&SecretRef::Keystore {
                alias: alias.to_owned(),
            })
            .expect("keystore payload should load");
        assert_eq!(loaded.expose_for_constrained_use(), plaintext);

        if let Some(previous) = original_dir {
            env::set_var(KEYRING_DIR_ENV, previous);
        } else {
            env::remove_var(KEYRING_DIR_ENV);
        }
        if let Some(previous) = original_key {
            env::set_var(KEYRING_MASTER_KEY_ENV, previous);
        } else {
            env::remove_var(KEYRING_MASTER_KEY_ENV);
        }
    }

    #[test]
    fn encrypted_keystore_rejects_tampered_authenticated_payload() {
        let _guard = KEYSTORE_ENV_LOCK
            .lock()
            .expect("keystore env lock poisoned");
        let root = env::temp_dir().join("arbyclaw-keystore-tamper-test");
        let alias = "ops-local-keystore-tamper";
        let master_key = "stable-master-key-value";
        let plaintext = b"phase-26-keystore-value";

        let dir = root.join("providers");
        let _ = fs::create_dir_all(&dir);
        let original_dir = env::var(KEYRING_DIR_ENV).ok();
        let original_key = env::var(KEYRING_MASTER_KEY_ENV).ok();

        let mut payload = build_keystore_entry(master_key, alias, plaintext);
        let last = payload.pop().expect("payload should not be empty");
        payload.push(if last == '0' { '1' } else { '0' });
        let secret_path = dir.join(format!("{alias}.secret"));
        fs::write(&secret_path, payload).expect("keystore fixture write");
        let provider = EncryptedKeystoreSecretProvider;

        env::set_var(KEYRING_DIR_ENV, &dir);
        env::set_var(KEYRING_MASTER_KEY_ENV, master_key);
        let result = provider.load(&SecretRef::Keystore {
            alias: alias.to_owned(),
        });
        assert!(matches!(
            result,
            Err(SecretStoreError::InvalidReference { reason })
                if reason.contains("authentication failed")
        ));

        if let Some(previous) = original_dir {
            env::set_var(KEYRING_DIR_ENV, previous);
        } else {
            env::remove_var(KEYRING_DIR_ENV);
        }
        if let Some(previous) = original_key {
            env::set_var(KEYRING_MASTER_KEY_ENV, previous);
        } else {
            env::remove_var(KEYRING_MASTER_KEY_ENV);
        }
    }

    #[test]
    fn local_keystore_preflight_reports_valid_entry_without_loading_material() {
        let root = env::temp_dir().join("arbyclaw-keystore-preflight-valid");
        let alias = "ops-local-keystore-preflight";
        let master_key = "stable-master-key-value";
        let plaintext = b"phase-26-keystore-value";
        let _ = fs::create_dir_all(&root);
        let payload = build_keystore_entry(master_key, alias, plaintext);
        let secret_path = root.join(format!("{alias}.secret"));
        fs::write(&secret_path, payload).expect("keystore fixture write");

        let report = preflight_local_keystore_entry(LocalKeystoreEntryPreflightRequest {
            alias: alias.to_owned(),
            keystore_dir: Some(root.clone()),
            checked_at_unix_ms: 1_700_000_000_001,
        })
        .expect("local preflight should validate entry metadata");

        assert!(report.entry_exists);
        assert!(report.payload_shape_valid);
        assert!(report.hex_payload_valid);
        assert!(report.salt_len_bytes > 0);
        assert_eq!(
            report.nonce_len_bytes,
            u64::try_from(super::KEYSTORE_V1_NONCE_LEN).expect("nonce length fits")
        );
        assert!(
            report.ciphertext_len_bytes
                > u64::try_from(plaintext.len()).expect("plaintext length fits")
        );
        assert!(!report.secret_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.production_ready);
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "KEYSTORE_HEX_PAYLOAD_VALID"));

        let _ = fs::remove_file(secret_path);
        let _ = fs::remove_dir(root);
    }

    #[test]
    fn local_keystore_preflight_reports_missing_entry_without_secret_access() {
        let root = env::temp_dir().join("arbyclaw-keystore-preflight-missing");
        let _ = fs::create_dir_all(&root);

        let report = preflight_local_keystore_entry(LocalKeystoreEntryPreflightRequest {
            alias: "missing-local-keystore-entry".to_owned(),
            keystore_dir: Some(root.clone()),
            checked_at_unix_ms: 1_700_000_000_002,
        })
        .expect("missing local preflight should still return metadata");

        assert!(!report.entry_exists);
        assert!(!report.payload_shape_valid);
        assert!(!report.hex_payload_valid);
        assert_eq!(report.salt_len_bytes, 0);
        assert_eq!(report.nonce_len_bytes, 0);
        assert_eq!(report.ciphertext_len_bytes, 0);
        assert!(!report.secret_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.production_ready);
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "KEYSTORE_ENTRY_MISSING"));

        let _ = fs::remove_dir(root);
    }

    #[test]
    fn local_keystore_preflight_rejects_invalid_payload_shape() {
        let root = env::temp_dir().join("arbyclaw-keystore-preflight-invalid");
        let alias = "invalid-local-keystore-entry";
        let _ = fs::create_dir_all(&root);
        let secret_path = root.join(format!("{alias}.secret"));
        fs::write(&secret_path, "not-a-valid-local-entry").expect("keystore fixture write");

        let report = preflight_local_keystore_entry(LocalKeystoreEntryPreflightRequest {
            alias: alias.to_owned(),
            keystore_dir: Some(root.clone()),
            checked_at_unix_ms: 1_700_000_000_003,
        })
        .expect("invalid shape should be reported without loading material");

        assert!(report.entry_exists);
        assert!(!report.payload_shape_valid);
        assert!(!report.hex_payload_valid);
        assert!(!report.secret_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_performed);
        assert!(!report.production_ready);
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "KEYSTORE_PAYLOAD_SHAPE_INVALID"));

        let _ = fs::remove_file(secret_path);
        let _ = fs::remove_dir(root);
    }

    #[test]
    fn secret_rotation_plan_accepts_distinct_keystore_references_without_material_access() {
        let report = ready_rotation_plan();

        assert_eq!(report.status, SecretRotationPlanStatus::ReadyForLocalReview);
        assert!(report.both_references_keystore_aliases);
        assert!(report.aliases_distinct);
        assert!(report.rotation_window_valid);
        assert_eq!(report.validation_count, 0);
        assert!(!report.secret_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.keystore_entry_written);
        assert!(!report.external_secret_revoked);
        assert!(!report.production_ready);
        report.validate().expect("ready rotation plan validates");
    }

    #[test]
    fn secret_rotation_plan_rejects_same_alias_without_mutation() {
        let report = plan_local_secret_rotation(SecretRotationPlanRequest {
            plan_id: "rotation-same-alias".to_owned(),
            secret_purpose: "wallet-signer".to_owned(),
            current_reference: SecretRef::Keystore {
                alias: "signer-active".to_owned(),
            },
            replacement_reference: SecretRef::Keystore {
                alias: "signer-active".to_owned(),
            },
            requested_by: "operator-a".to_owned(),
            rotation_reason: "scheduled rotation".to_owned(),
            planned_at_unix_ms: 1_700_000_001_000,
            not_before_unix_ms: 1_700_000_002_000,
            expires_at_unix_ms: 1_700_000_003_000,
        })
        .expect("same-alias rotation should return rejected report");

        assert_eq!(report.status, SecretRotationPlanStatus::RejectedSameAlias);
        assert!(!report.aliases_distinct);
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "SECRET_ROTATION_REPLACEMENT_ALIAS_MUST_DIFFER"));
        assert!(!report.keystore_entry_written);
        assert!(!report.external_secret_revoked);
        report.validate().expect("same-alias report validates");
    }

    #[test]
    fn secret_rotation_plan_rejects_non_keystore_reference_without_loading_secret() {
        let report = plan_local_secret_rotation(SecretRotationPlanRequest {
            plan_id: "rotation-env-reference".to_owned(),
            secret_purpose: "wallet-signer".to_owned(),
            current_reference: SecretRef::Env {
                name: "ARBYCLAW_SIGNER_ACTIVE".to_owned(),
            },
            replacement_reference: SecretRef::Keystore {
                alias: "signer-next".to_owned(),
            },
            requested_by: "operator-a".to_owned(),
            rotation_reason: "scheduled rotation".to_owned(),
            planned_at_unix_ms: 1_700_000_001_000,
            not_before_unix_ms: 1_700_000_002_000,
            expires_at_unix_ms: 1_700_000_003_000,
        })
        .expect("non-keystore rotation should return rejected report");

        assert_eq!(
            report.status,
            SecretRotationPlanStatus::RejectedNonKeystoreReference
        );
        assert!(!report.both_references_keystore_aliases);
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "SECRET_ROTATION_KEYSTORE_REFERENCES_REQUIRED"));
        assert!(!report.secret_material_loaded);
        assert!(!report.plaintext_decrypted);
        report.validate().expect("non-keystore report validates");
    }

    #[test]
    fn secret_rotation_plan_audit_and_state_reopen_locally() {
        let report = ready_rotation_plan();
        let audit_path = unique_temp_path("secret-rotation-audit", "jsonl");
        let state_path = unique_temp_path("secret-rotation-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_secret_rotation_plan_audit(&mut journal, &report)
            .expect("rotation audit appends");
        let checkpoint = persist_secret_rotation_plan_checkpoint(&mut store, &report)
            .expect("rotation checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY);
        assert!(matches!(
            audit_record.event.metadata.get("keystore_entry_written"),
            Some(AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("material_loaded"),
            Some(AuditValue::Bool(false))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 2);
        let reopened_store = SqliteWalStateStore::open(&state_path).expect("state reopens");
        let checkpoint = reopened_store
            .get_checkpoint(SECRET_LAST_ROTATION_PLAN_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: SecretRotationPlanReport =
            serde_json::from_str(&checkpoint.value).expect("rotation report json");
        assert_eq!(recovered, report);
        assert!(!recovered.secret_material_loaded);
        assert!(!recovered.keystore_entry_written);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn secret_backup_restore_review_accepts_sanitized_reference_only_recovery() {
        let report = ready_backup_restore_review();

        assert_eq!(
            report.status,
            SecretBackupRestoreReviewStatus::ReadyForLocalReview
        );
        assert!(report.backup_reference_present);
        assert!(report.backup_payload_shape_verified);
        assert!(report.restore_verification_passed);
        assert!(report.references_sanitized);
        assert!(report.review_window_valid);
        assert_eq!(report.validation_count, 0);
        assert!(!report.secret_material_loaded);
        assert!(!report.plaintext_decrypted);
        assert!(!report.keystore_entry_written);
        assert!(!report.external_secret_restored);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
        report.validate().expect("ready backup restore validates");
    }

    #[test]
    fn secret_backup_restore_review_blocks_missing_backup_reference() {
        let report = review_local_secret_backup_restore(SecretBackupRestoreReviewRequest {
            backup_reference: String::new(),
            ..ready_backup_restore_request()
        })
        .expect("blocked backup restore review still reports");

        assert_eq!(
            report.status,
            SecretBackupRestoreReviewStatus::BlockedMissingBackupReference
        );
        assert!(!report.backup_reference_present);
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "SECRET_BACKUP_REFERENCE_REQUIRED"));
        assert!(!report.secret_material_loaded);
        assert!(!report.external_secret_restored);
        report
            .validate()
            .expect("missing-reference backup restore report validates");
    }

    #[test]
    fn secret_backup_restore_review_blocks_unverified_restore_without_side_effects() {
        let report = review_local_secret_backup_restore(SecretBackupRestoreReviewRequest {
            restore_verification_passed: false,
            references_sanitized: false,
            ..ready_backup_restore_request()
        })
        .expect("blocked backup restore review still reports");

        assert_eq!(
            report.status,
            SecretBackupRestoreReviewStatus::BlockedRestoreVerification
        );
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "SECRET_RESTORE_VERIFICATION_MISSING"));
        assert!(report
            .validation_codes
            .iter()
            .any(|code| code == "SECRET_BACKUP_RESTORE_REFERENCES_UNSANITIZED"));
        assert!(!report.plaintext_decrypted);
        assert!(!report.signing_or_broadcast_performed);
        report
            .validate()
            .expect("unverified backup restore report validates");
    }

    #[test]
    fn secret_backup_restore_review_audit_and_state_reopen_locally() {
        let report = ready_backup_restore_review();
        let audit_path = unique_temp_path("secret-backup-restore-audit", "jsonl");
        let state_path = unique_temp_path("secret-backup-restore-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_secret_backup_restore_review_audit(&mut journal, &report)
            .expect("backup restore audit appends");
        let checkpoint = persist_secret_backup_restore_review_checkpoint(&mut store, &report)
            .expect("backup restore checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("external_reference_restored"),
            Some(AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("material_loaded"),
            Some(AuditValue::Bool(false))
        ));

        let next_sequence = journal.next_sequence();
        let mut invalid = report.clone();
        invalid.secret_material_loaded = true;
        assert!(append_secret_backup_restore_review_audit(&mut journal, &invalid).is_err());
        assert_eq!(journal.next_sequence(), next_sequence);

        let mut denied_store = DeniedSecretStateStore::default();
        assert!(
            persist_secret_backup_restore_review_checkpoint(&mut denied_store, &report).is_err()
        );

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 2);
        let reopened_store = SqliteWalStateStore::open(&state_path).expect("state reopens");
        let checkpoint = reopened_store
            .get_checkpoint(SECRET_LAST_BACKUP_RESTORE_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: SecretBackupRestoreReviewReport =
            serde_json::from_str(&checkpoint.value).expect("backup restore report json");
        assert_eq!(recovered, report);
        assert!(!recovered.secret_material_loaded);
        assert!(!recovered.external_secret_restored);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn decode_hex_bytes_rejects_invalid_length_and_characters() {
        assert!(decode_hex_bytes("label", "abc").is_err());
        assert!(decode_hex_bytes("label", "zz").is_err());
        assert_eq!(encode_hex_bytes(&[0x0a, 0xff]), "0aff");
    }

    fn ready_rotation_plan() -> SecretRotationPlanReport {
        plan_local_secret_rotation(SecretRotationPlanRequest {
            plan_id: "rotation-ready".to_owned(),
            secret_purpose: "wallet-signer".to_owned(),
            current_reference: SecretRef::Keystore {
                alias: "signer-active".to_owned(),
            },
            replacement_reference: SecretRef::Keystore {
                alias: "signer-next".to_owned(),
            },
            requested_by: "operator-a".to_owned(),
            rotation_reason: "scheduled rotation".to_owned(),
            planned_at_unix_ms: 1_700_000_001_000,
            not_before_unix_ms: 1_700_000_002_000,
            expires_at_unix_ms: 1_700_000_003_000,
        })
        .expect("ready rotation plan succeeds")
    }

    fn ready_backup_restore_request() -> SecretBackupRestoreReviewRequest {
        SecretBackupRestoreReviewRequest {
            review_id: "backup-restore-ready".to_owned(),
            secret_purpose: "wallet-signer".to_owned(),
            source_reference: SecretRef::Keystore {
                alias: "signer-active".to_owned(),
            },
            backup_reference: "actions-artifact:secret-backup-shape-v1".to_owned(),
            restore_target_label: "local-restore-shape-check".to_owned(),
            reviewed_by: "operator-a".to_owned(),
            review_note: "sanitized backup and restore shape review".to_owned(),
            backup_payload_shape_verified: true,
            restore_verification_passed: true,
            references_sanitized: true,
            reviewed_at_unix_ms: 1_700_000_010_000,
            review_window_start_unix_ms: 1_700_000_011_000,
            review_window_expires_unix_ms: 1_700_000_012_000,
        }
    }

    fn ready_backup_restore_review() -> SecretBackupRestoreReviewReport {
        review_local_secret_backup_restore(ready_backup_restore_request())
            .expect("ready backup restore review succeeds")
    }

    #[derive(Default)]
    struct DeniedSecretStateStore {
        put_attempts: u64,
    }

    impl StateStore for DeniedSecretStateStore {
        fn put_checkpoint(&mut self, _checkpoint: StateCheckpoint) -> Result<(), StateStoreError> {
            self.put_attempts = self.put_attempts.saturating_add(1);
            Err(StateStoreError::BackendFailed {
                reason: "permission denied".to_owned(),
            })
        }

        fn get_checkpoint(&self, _key: &str) -> Result<Option<StateCheckpoint>, StateStoreError> {
            Ok(None)
        }
    }

    fn unique_temp_path(label: &str, extension: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!(
            "arbyclaw-secrets-{label}-{nanos}-{counter}.{extension}"
        ))
    }
}
