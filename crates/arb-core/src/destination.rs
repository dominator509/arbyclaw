#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, StateCheckpoint,
    StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable destination allowlist boundary version.
pub const DESTINATION_ALLOWLIST_VERSION: &str = "phase-3-destination-allowlist-v1";

/// State-store subsystem name for local destination allowlist checkpoints.
pub const DESTINATION_ALLOWLIST_STATE_SUBSYSTEM: &str = "destination-allowlist";

/// State-store key for the latest local destination allowlist snapshot.
pub const DESTINATION_ALLOWLIST_CHECKPOINT_KEY: &str = "destination-allowlist:snapshot";

/// State-store key for the latest local ownership-evidence review report.
pub const DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY: &str =
    "destination-allowlist:ownership-review";

/// Operator approval source for a destination entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DestinationApprovalSource {
    /// Explicit local operator approval.
    LocalOperator,
    /// Future multi-party approval boundary.
    MultiPartyReview,
    /// LLM-generated or inferred destination. Always rejected.
    LlmGenerated,
}

/// One approved destination label.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovedDestinationEntry {
    /// Stable operator-facing label used by policy records.
    pub label: String,
    /// Chain identifier, such as ethereum or solana-mainnet.
    pub chain: String,
    /// Non-secret address fingerprint or approved external address-book reference.
    pub address_fingerprint: String,
    /// Non-secret approval identifier.
    pub approval_id: String,
    /// Non-secret operator or reviewer label.
    pub approved_by: String,
    /// Approval source.
    pub approval_source: DestinationApprovalSource,
    /// Whether ownership/control evidence is referenced. This boundary does not verify ownership.
    pub ownership_evidence_referenced: bool,
    /// Whether this destination may be considered by policy.
    pub enabled: bool,
}

impl ApprovedDestinationEntry {
    /// Validate one destination allowlist entry.
    pub fn validate(&self) -> Result<(), DestinationAllowlistError> {
        let mut violations = Vec::new();
        validate_id("destination label", &self.label, &mut violations);
        validate_id("destination chain", &self.chain, &mut violations);
        validate_id(
            "destination address fingerprint",
            &self.address_fingerprint,
            &mut violations,
        );
        validate_id(
            "destination approval id",
            &self.approval_id,
            &mut violations,
        );
        validate_id("destination approver", &self.approved_by, &mut violations);

        if self.approval_source == DestinationApprovalSource::LlmGenerated {
            violations.push(DestinationAllowlistViolation::new(
                "DESTINATION_LLM_APPROVAL_DENIED",
                "LLM-generated destinations cannot be approved",
            ));
        }

        finish_validation(violations)
    }

    /// Return whether this entry matches a policy destination.
    #[must_use]
    pub fn matches_policy_destination(&self, chain: &str, label: &str) -> bool {
        self.enabled
            && self.chain.eq_ignore_ascii_case(chain)
            && self.label.eq_ignore_ascii_case(label)
    }
}

/// Local approved-destination allowlist snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DestinationAllowlist {
    /// Boundary version that produced this snapshot.
    pub destination_allowlist_version: String,
    /// Snapshot identifier.
    pub snapshot_id: String,
    /// Snapshot creation/update timestamp in Unix epoch milliseconds.
    pub updated_at_unix_ms: u64,
    /// Approved destination entries.
    pub entries: Vec<ApprovedDestinationEntry>,
}

/// Local non-secret ownership evidence review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DestinationOwnershipReviewStatus {
    /// Local evidence references are present for enabled entries.
    Referenced,
    /// One or more enabled entries lack an evidence reference.
    MissingReference,
}

/// Local non-secret destination ownership evidence review report.
///
/// This report checks references only. It does not prove wallet ownership, call
/// RPC endpoints, load signer material, sign challenge messages, or validate
/// addresses against a chain.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DestinationOwnershipReviewReport {
    /// Boundary version that produced this report.
    pub destination_allowlist_version: String,
    /// Snapshot identifier reviewed.
    pub snapshot_id: String,
    /// Review timestamp in Unix epoch milliseconds.
    pub reviewed_at_unix_ms: u64,
    /// Number of enabled destinations reviewed.
    pub enabled_entry_count: usize,
    /// Number of enabled destinations with ownership evidence references.
    pub referenced_evidence_count: usize,
    /// Enabled chain/label pairs missing ownership evidence references.
    pub missing_evidence_labels: Vec<String>,
    /// Local review status.
    pub status: DestinationOwnershipReviewStatus,
    /// Whether a chain/RPC ownership verification was performed. Always false here.
    pub chain_ownership_verified: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether a challenge was signed. Always false here.
    pub challenge_signed: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

impl DestinationOwnershipReviewReport {
    /// Validate report invariants before audit/state persistence.
    pub fn validate(&self) -> Result<(), DestinationAllowlistError> {
        let mut violations = Vec::new();
        validate_id(
            "destination ownership review snapshot",
            &self.snapshot_id,
            &mut violations,
        );
        if self.destination_allowlist_version != DESTINATION_ALLOWLIST_VERSION {
            violations.push(DestinationAllowlistViolation::new_owned(
                "DESTINATION_OWNERSHIP_REVIEW_VERSION_MISMATCH",
                format!(
                    "destination ownership review version {} does not match {}",
                    self.destination_allowlist_version, DESTINATION_ALLOWLIST_VERSION
                ),
            ));
        }
        if self.referenced_evidence_count > self.enabled_entry_count {
            violations.push(DestinationAllowlistViolation::new(
                "DESTINATION_OWNERSHIP_REVIEW_COUNT_INVALID",
                "referenced evidence count cannot exceed enabled entry count",
            ));
        }
        for label in &self.missing_evidence_labels {
            validate_id("missing destination evidence label", label, &mut violations);
        }
        match self.status {
            DestinationOwnershipReviewStatus::Referenced => {
                if !self.missing_evidence_labels.is_empty()
                    || self.referenced_evidence_count != self.enabled_entry_count
                {
                    violations.push(DestinationAllowlistViolation::new(
                        "DESTINATION_OWNERSHIP_REVIEW_STATUS_INVALID",
                        "referenced status requires all enabled entries to reference evidence",
                    ));
                }
            }
            DestinationOwnershipReviewStatus::MissingReference => {
                if self.missing_evidence_labels.is_empty() {
                    violations.push(DestinationAllowlistViolation::new(
                        "DESTINATION_OWNERSHIP_REVIEW_MISSING_LABELS_REQUIRED",
                        "missing-reference status requires missing evidence labels",
                    ));
                }
            }
        }
        if self.chain_ownership_verified
            || self.signer_material_loaded
            || self.challenge_signed
            || self.production_ready
        {
            violations.push(DestinationAllowlistViolation::new(
                "DESTINATION_OWNERSHIP_REVIEW_SIDE_EFFECT_DENIED",
                "local ownership review must not verify chains, load signers, sign challenges, or approve production readiness",
            ));
        }
        finish_validation(violations)
    }
}

impl DestinationAllowlist {
    /// Create an empty local destination allowlist snapshot.
    #[must_use]
    pub fn empty(snapshot_id: impl Into<String>, updated_at_unix_ms: u64) -> Self {
        Self {
            destination_allowlist_version: DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: snapshot_id.into(),
            updated_at_unix_ms,
            entries: Vec::new(),
        }
    }

    /// Validate this destination allowlist snapshot.
    pub fn validate(&self) -> Result<(), DestinationAllowlistError> {
        let mut violations = Vec::new();
        validate_id(
            "destination allowlist snapshot",
            &self.snapshot_id,
            &mut violations,
        );
        if self.destination_allowlist_version != DESTINATION_ALLOWLIST_VERSION {
            violations.push(DestinationAllowlistViolation::new_owned(
                "DESTINATION_ALLOWLIST_VERSION_MISMATCH",
                format!(
                    "destination allowlist version {} does not match {}",
                    self.destination_allowlist_version, DESTINATION_ALLOWLIST_VERSION
                ),
            ));
        }

        let mut labels = BTreeSet::new();
        for entry in &self.entries {
            if let Err(error) = entry.validate() {
                violations.extend(error.violations().iter().cloned());
            }
            if entry.enabled && !entry.ownership_evidence_referenced {
                violations.push(DestinationAllowlistViolation::new_owned(
                    "DESTINATION_ENABLED_OWNERSHIP_EVIDENCE_REQUIRED",
                    format!(
                        "enabled destination {} on {} must reference ownership evidence",
                        entry.label, entry.chain
                    ),
                ));
            }
            let key = format!(
                "{}:{}",
                entry.chain.to_ascii_lowercase(),
                entry.label.to_ascii_lowercase()
            );
            if !labels.insert(key) {
                violations.push(DestinationAllowlistViolation::new(
                    "DESTINATION_ALLOWLIST_DUPLICATE",
                    "destination allowlist contains duplicate chain/label entries",
                ));
            }
        }

        finish_validation(violations)
    }

    /// Return true when the policy destination is approved by this snapshot.
    #[must_use]
    pub fn approves(&self, chain: &str, label: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.matches_policy_destination(chain, label))
    }

    /// Review local ownership evidence references for enabled destinations.
    pub fn review_ownership_evidence(
        &self,
        reviewed_at_unix_ms: u64,
    ) -> Result<DestinationOwnershipReviewReport, DestinationAllowlistError> {
        self.validate()?;
        let enabled_entry_count = self.entries.iter().filter(|entry| entry.enabled).count();
        let referenced_evidence_count = self
            .entries
            .iter()
            .filter(|entry| entry.enabled && entry.ownership_evidence_referenced)
            .count();
        let missing_evidence_labels = self
            .entries
            .iter()
            .filter(|entry| entry.enabled && !entry.ownership_evidence_referenced)
            .map(|entry| format!("{}:{}", entry.chain, entry.label))
            .collect::<Vec<_>>();
        let status = if missing_evidence_labels.is_empty() {
            DestinationOwnershipReviewStatus::Referenced
        } else {
            DestinationOwnershipReviewStatus::MissingReference
        };
        let report = DestinationOwnershipReviewReport {
            destination_allowlist_version: DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: self.snapshot_id.clone(),
            reviewed_at_unix_ms,
            enabled_entry_count,
            referenced_evidence_count,
            missing_evidence_labels,
            status,
            chain_ownership_verified: false,
            signer_material_loaded: false,
            challenge_signed: false,
            production_ready: false,
        };
        report.validate()?;
        Ok(report)
    }
}

/// Persist a local destination allowlist snapshot through the typed state boundary.
pub fn persist_destination_allowlist_checkpoint(
    store: &mut impl StateStore,
    allowlist: &DestinationAllowlist,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DestinationAllowlistError> {
    allowlist.validate()?;
    let checkpoint = StateCheckpoint {
        key: DESTINATION_ALLOWLIST_CHECKPOINT_KEY.to_owned(),
        subsystem: DESTINATION_ALLOWLIST_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(allowlist).map_err(|error| {
            DestinationAllowlistError::StateStoreFailed {
                reason: format!("failed to serialize destination allowlist checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DestinationAllowlistError::from)?;
    Ok(checkpoint)
}

/// Append a local destination allowlist snapshot to the audit journal.
pub fn append_destination_allowlist_audit(
    journal: &mut AppendOnlyAuditJournal,
    allowlist: &DestinationAllowlist,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DestinationAllowlistError> {
    allowlist.validate()?;
    let enabled_count = allowlist
        .entries
        .iter()
        .filter(|entry| entry.enabled)
        .count();
    let mut event = AuditEvent::new(
        format!("destination-allowlist-{}", allowlist.snapshot_id),
        AuditEventKind::PolicyDecision,
        DESTINATION_ALLOWLIST_STATE_SUBSYSTEM,
        "destination-allowlist",
        "destination allowlist recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "destination_allowlist_version",
            AuditValue::Text(DESTINATION_ALLOWLIST_VERSION.to_owned()),
        )
        .with_metadata(
            "snapshot_id",
            AuditValue::Text(allowlist.snapshot_id.clone()),
        )
        .with_metadata(
            "entry_count",
            AuditValue::Text(allowlist.entries.len().to_string()),
        )
        .with_metadata(
            "enabled_entry_count",
            AuditValue::Text(enabled_count.to_string()),
        )
        .with_metadata("external_action_performed", AuditValue::Bool(false))
        .with_metadata("llm_destination_approved", AuditValue::Bool(false));
    journal
        .append_event(event)
        .map_err(DestinationAllowlistError::from)
}

/// Persist a local destination ownership-evidence review through the typed state boundary.
pub fn persist_destination_ownership_review_checkpoint(
    store: &mut impl StateStore,
    report: &DestinationOwnershipReviewReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DestinationAllowlistError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: DESTINATION_ALLOWLIST_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DestinationAllowlistError::StateStoreFailed {
                reason: format!("failed to serialize destination ownership review: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DestinationAllowlistError::from)?;
    Ok(checkpoint)
}

/// Append a local destination ownership-evidence review to the audit journal.
pub fn append_destination_ownership_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &DestinationOwnershipReviewReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DestinationAllowlistError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("destination-ownership-review-{}", report.snapshot_id),
        AuditEventKind::PolicyDecision,
        DESTINATION_ALLOWLIST_STATE_SUBSYSTEM,
        "destination-ownership-review",
        "destination ownership evidence references reviewed",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "destination_allowlist_version",
            AuditValue::Text(DESTINATION_ALLOWLIST_VERSION.to_owned()),
        )
        .with_metadata("snapshot_id", AuditValue::Text(report.snapshot_id.clone()))
        .with_metadata(
            "enabled_entry_count",
            AuditValue::Text(report.enabled_entry_count.to_string()),
        )
        .with_metadata(
            "referenced_evidence_count",
            AuditValue::Text(report.referenced_evidence_count.to_string()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "chain_ownership_verified",
            AuditValue::Bool(report.chain_ownership_verified),
        )
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "challenge_signed",
            AuditValue::Bool(report.challenge_signed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(DestinationAllowlistError::from)
}

/// Destination allowlist validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationAllowlistError {
    /// Validation failed.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<DestinationAllowlistViolation>,
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

impl DestinationAllowlistError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[DestinationAllowlistViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::AuditJournalFailed { .. } | Self::StateStoreFailed { .. } => &[],
        }
    }
}

impl From<crate::AuditError> for DestinationAllowlistError {
    fn from(error: crate::AuditError) -> Self {
        Self::AuditJournalFailed {
            reason: error.to_string(),
        }
    }
}

impl From<StateStoreError> for DestinationAllowlistError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for DestinationAllowlistError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                writeln!(
                    formatter,
                    "destination allowlist validation failed with {} violation(s):",
                    violations.len()
                )?;
                for violation in violations {
                    writeln!(formatter, "- {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "destination allowlist audit failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(
                    formatter,
                    "destination allowlist state store failed: {reason}"
                )
            }
        }
    }
}

impl Error for DestinationAllowlistError {}

/// One destination allowlist violation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DestinationAllowlistViolation {
    code: String,
    message: String,
}

impl DestinationAllowlistViolation {
    /// Create a validation violation.
    #[must_use]
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
        }
    }

    /// Create a validation violation with owned message text.
    #[must_use]
    pub fn new_owned(code: &'static str, message: String) -> Self {
        Self {
            code: code.to_owned(),
            message,
        }
    }

    /// Stable violation code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Human-readable violation detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

fn finish_validation(
    violations: Vec<DestinationAllowlistViolation>,
) -> Result<(), DestinationAllowlistError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(DestinationAllowlistError::ValidationFailed { violations })
    }
}

fn validate_id(
    label: &'static str,
    value: &str,
    violations: &mut Vec<DestinationAllowlistViolation>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        violations.push(DestinationAllowlistViolation::new_owned(
            "DESTINATION_ID_EMPTY",
            format!("{label} must be non-empty"),
        ));
    }
    if trimmed.len() > 128 {
        violations.push(DestinationAllowlistViolation::new_owned(
            "DESTINATION_ID_TOO_LONG",
            format!("{label} must be 128 characters or fewer"),
        ));
    }
    if trimmed.chars().any(char::is_whitespace) {
        violations.push(DestinationAllowlistViolation::new_owned(
            "DESTINATION_ID_WHITESPACE",
            format!("{label} cannot contain whitespace"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_destination_allowlist_audit, append_destination_ownership_review_audit,
        persist_destination_allowlist_checkpoint, persist_destination_ownership_review_checkpoint,
        ApprovedDestinationEntry, DestinationAllowlist, DestinationApprovalSource,
        DestinationOwnershipReviewReport, DestinationOwnershipReviewStatus,
        DESTINATION_ALLOWLIST_CHECKPOINT_KEY, DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY,
    };
    use crate::{AppendOnlyAuditJournal, SqliteWalStateStore, StateStore};
    use std::{env, fs, path::PathBuf, process};

    fn entry() -> ApprovedDestinationEntry {
        ApprovedDestinationEntry {
            label: "ops-vault".to_owned(),
            chain: "ethereum".to_owned(),
            address_fingerprint: "sha256:ops-vault-reference".to_owned(),
            approval_id: "approval-001".to_owned(),
            approved_by: "operator-a".to_owned(),
            approval_source: DestinationApprovalSource::LocalOperator,
            ownership_evidence_referenced: true,
            enabled: true,
        }
    }

    #[test]
    fn destination_allowlist_approves_only_matching_enabled_entries() {
        let allowlist = DestinationAllowlist {
            destination_allowlist_version: super::DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: "destinations-local".to_owned(),
            updated_at_unix_ms: 1_700_000_000_000,
            entries: vec![entry()],
        };

        allowlist.validate().expect("allowlist validates");
        assert!(allowlist.approves("ethereum", "ops-vault"));
        assert!(!allowlist.approves("ethereum", "unknown"));
        assert!(!allowlist.approves("solana", "ops-vault"));
    }

    #[test]
    fn destination_allowlist_rejects_llm_generated_approval() {
        let mut entry = entry();
        entry.approval_source = DestinationApprovalSource::LlmGenerated;
        let error = entry
            .validate()
            .expect_err("LLM generated destination approval must fail closed");
        assert!(error
            .violations()
            .iter()
            .map(super::DestinationAllowlistViolation::code)
            .any(|code| code == "DESTINATION_LLM_APPROVAL_DENIED"));
    }

    #[test]
    fn enabled_destination_requires_ownership_evidence_reference() {
        let mut entry = entry();
        entry.ownership_evidence_referenced = false;
        let allowlist = DestinationAllowlist {
            destination_allowlist_version: super::DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: "destinations-local".to_owned(),
            updated_at_unix_ms: 1_700_000_000_000,
            entries: vec![entry],
        };

        let error = allowlist
            .validate()
            .expect_err("enabled destination without evidence reference must fail closed");
        assert!(error.violations().iter().any(|violation| {
            violation.code() == "DESTINATION_ENABLED_OWNERSHIP_EVIDENCE_REQUIRED"
        }));
    }

    #[test]
    fn destination_ownership_review_checks_references_without_chain_or_signer_side_effects() {
        let allowlist = DestinationAllowlist {
            destination_allowlist_version: super::DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: "destinations-local".to_owned(),
            updated_at_unix_ms: 1_700_000_000_000,
            entries: vec![entry()],
        };

        let report = allowlist
            .review_ownership_evidence(1_700_000_000_010)
            .expect("ownership evidence reference review should pass");
        assert_eq!(report.status, DestinationOwnershipReviewStatus::Referenced);
        assert_eq!(report.enabled_entry_count, 1);
        assert_eq!(report.referenced_evidence_count, 1);
        assert!(report.missing_evidence_labels.is_empty());
        assert!(!report.chain_ownership_verified);
        assert!(!report.signer_material_loaded);
        assert!(!report.challenge_signed);
        assert!(!report.production_ready);
    }

    #[test]
    fn destination_allowlist_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("destination-allowlist");
        let state_path = temp_state_path("destination-allowlist");
        let allowlist = DestinationAllowlist {
            destination_allowlist_version: super::DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: "destinations-local".to_owned(),
            updated_at_unix_ms: 1_700_000_000_000,
            entries: vec![entry()],
        };
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_destination_allowlist_audit(&mut journal, &allowlist, 1_700_000_000_001)
                .expect("destination audit writes");
        let checkpoint =
            persist_destination_allowlist_checkpoint(&mut store, &allowlist, 1_700_000_000_002)
                .expect("destination checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, DESTINATION_ALLOWLIST_CHECKPOINT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DESTINATION_ALLOWLIST_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("destination checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered.value.contains("\"label\":\"ops-vault\""));
        assert!(recovered.value.contains("\"enabled\":true"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn destination_ownership_review_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("destination-ownership-review");
        let state_path = temp_state_path("destination-ownership-review");
        let allowlist = DestinationAllowlist {
            destination_allowlist_version: super::DESTINATION_ALLOWLIST_VERSION.to_owned(),
            snapshot_id: "destinations-local".to_owned(),
            updated_at_unix_ms: 1_700_000_000_000,
            entries: vec![entry()],
        };
        let report = allowlist
            .review_ownership_evidence(1_700_000_000_010)
            .expect("ownership evidence reference review should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_destination_ownership_review_audit(&mut journal, &report, 1_700_000_000_011)
                .expect("ownership review audit writes");
        let checkpoint =
            persist_destination_ownership_review_checkpoint(&mut store, &report, 1_700_000_000_012)
                .expect("ownership review checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(DESTINATION_OWNERSHIP_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("ownership review checkpoint exists");
        let recovered_report: DestinationOwnershipReviewReport =
            serde_json::from_str(&recovered.value).expect("checkpoint decodes");
        assert_eq!(
            recovered_report.status,
            DestinationOwnershipReviewStatus::Referenced
        );
        assert!(!recovered_report.chain_ownership_verified);
        assert!(!recovered_report.signer_material_loaded);
        assert!(!recovered_report.challenge_signed);
        assert!(!recovered_report.production_ready);

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
