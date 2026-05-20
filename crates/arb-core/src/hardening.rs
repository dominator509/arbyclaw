#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable external hardening boundary version for audit and handoff surfaces.
pub const EXTERNAL_HARDENING_VERSION: &str = "phase-17-external-hardening-v1";

/// Conservative Phase 17 external-hardening settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalHardeningBoundaryConfig {
    /// Whether local evidence planning/review records may be produced.
    pub evidence_recording_enabled: bool,
    /// Whether this boundary may perform external network or deployment actions. Phase 17 requires false.
    pub external_actions_enabled: bool,
    /// Whether this boundary may claim production readiness. Phase 17 requires false.
    pub production_readiness_claims_enabled: bool,
    /// Whether this boundary may approve live funds. Phase 17 requires false.
    pub live_funds_approval_enabled: bool,
    /// Whether evidence fields may contain secret material. Phase 17 requires false.
    pub secret_material_in_evidence_allowed: bool,
    /// Whether public exposure may be approved. Phase 17 requires false.
    pub public_exposure_approval_enabled: bool,
}

impl Default for ExternalHardeningBoundaryConfig {
    fn default() -> Self {
        Self {
            evidence_recording_enabled: true,
            external_actions_enabled: false,
            production_readiness_claims_enabled: false,
            live_funds_approval_enabled: false,
            secret_material_in_evidence_allowed: false,
            public_exposure_approval_enabled: false,
        }
    }
}

impl ExternalHardeningBoundaryConfig {
    /// Validate fail-closed Phase 17 external hardening settings.
    pub fn validate(&self) -> Result<(), ExternalHardeningError> {
        let mut violations = Vec::new();

        if !self.evidence_recording_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_RECORDING_DISABLED",
                "Phase 17 requires evidence planning records to remain enabled",
            ));
        }
        if self.external_actions_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EXTERNAL_ACTIONS_DENIED",
                "ChatGPT Project Mode must not perform external hardening actions",
            ));
        }
        if self.production_readiness_claims_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PRODUCTION_CLAIMS_DENIED",
                "Phase 17 records must not claim production readiness",
            ));
        }
        if self.live_funds_approval_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_LIVE_FUNDS_APPROVAL_DENIED",
                "Phase 17 records must not approve live funds",
            ));
        }
        if self.secret_material_in_evidence_allowed {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_SECRET_EVIDENCE_DENIED",
                "external hardening evidence must not contain secret material",
            ));
        }
        if self.public_exposure_approval_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PUBLIC_EXPOSURE_APPROVAL_DENIED",
                "Phase 17 records must not approve public exposure",
            ));
        }

        finish_validation(violations)
    }
}

/// Environment-limited hardening activity category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalHardeningActivityKind {
    /// Rust formatting/check/test/clippy validation.
    RustWorkspaceValidation,
    /// Release binary build validation.
    ReleaseBuildValidation,
    /// Dependency audit or vulnerability scan.
    DependencyAudit,
    /// SBOM generation and review.
    SbomReview,
    /// Container build and image scanning.
    ContainerBuildAndScan,
    /// systemd or service hardening validation.
    ServiceHardeningValidation,
    /// ARM or edge target build/runtime validation.
    ArmTargetValidation,
    /// Staging or dry-run deployment validation.
    StagingDeploymentValidation,
    /// Load and soak test validation.
    LoadAndSoakTest,
    /// Penetration test or AppSec review.
    PenetrationTest,
    /// Exchange sandbox or paper integration validation.
    ExchangeSandboxValidation,
    /// DEX/RPC sandbox validation without signing or broadcasting.
    DexRpcSandboxValidation,
    /// Audit replay and state recovery drill.
    AuditReplayDrill,
    /// Rollback drill.
    RollbackDrill,
    /// Incident-response tabletop or drill.
    IncidentResponseDrill,
    /// Key custody and signer design review.
    KeyCustodyReview,
    /// Legal, compliance, and terms review.
    ComplianceReview,
}

/// Evidence status for a hardening activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HardeningEvidenceStatus {
    /// Evidence is planned but not executed.
    Planned,
    /// Evidence was observed externally and recorded by reference only.
    ExternallyObserved,
    /// External activity failed.
    Failed,
    /// External activity is blocked.
    Blocked,
}

/// One deterministic hardening evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HardeningEvidenceRecord {
    /// Stable evidence identifier.
    pub evidence_id: String,
    /// Hardening activity category.
    pub activity_kind: ExternalHardeningActivityKind,
    /// Human-readable description.
    pub description: String,
    /// Evidence status.
    pub status: HardeningEvidenceStatus,
    /// Non-secret evidence reference, such as a local file path, ticket id, or CI run label.
    pub evidence_reference: Option<String>,
    /// Operator, system, or agent that performed the external activity.
    pub performed_by: Option<String>,
    /// External environment label.
    pub external_environment: Option<String>,
    /// Whether this model boundary performed the external action. Phase 17 requires false.
    pub performed_by_this_boundary: bool,
    /// Whether this record claims production readiness. Phase 17 requires false.
    pub claims_production_ready: bool,
    /// Whether this record used live funds. Phase 17 requires false.
    pub used_live_funds: bool,
    /// Whether this record contains secret material. Phase 17 requires false.
    pub contains_secret_material: bool,
    /// Whether this record approves public exposure. Phase 17 requires false.
    pub approves_public_exposure: bool,
}

impl HardeningEvidenceRecord {
    /// Construct a planned external hardening evidence record.
    #[must_use]
    pub fn planned(
        evidence_id: impl Into<String>,
        activity_kind: ExternalHardeningActivityKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            evidence_id: evidence_id.into(),
            activity_kind,
            description: description.into(),
            status: HardeningEvidenceStatus::Planned,
            evidence_reference: None,
            performed_by: None,
            external_environment: None,
            performed_by_this_boundary: false,
            claims_production_ready: false,
            used_live_funds: false,
            contains_secret_material: false,
            approves_public_exposure: false,
        }
    }

    fn validate(
        &self,
        config: &ExternalHardeningBoundaryConfig,
        violations: &mut Vec<ExternalHardeningViolation>,
    ) {
        let context = format!("evidence {}", self.evidence_id);
        if self.evidence_id.trim().is_empty() {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_ID_EMPTY",
                "evidence_id must be non-empty",
            ));
        }
        if self.description.trim().is_empty() {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_DESCRIPTION_EMPTY",
                format!("{context} description must be non-empty"),
            ));
        }
        if contains_secret_like_text(&self.description) {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_DESCRIPTION_SECRET_LIKE",
                format!("{context} description contains secret-like text"),
            ));
        }
        if self.performed_by_this_boundary || config.external_actions_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_SELF_PERFORMED_ACTION_DENIED",
                format!("{context} must not claim this boundary performed external hardening"),
            ));
        }
        if self.claims_production_ready || config.production_readiness_claims_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_PRODUCTION_CLAIM_DENIED",
                format!("{context} must not claim production readiness"),
            ));
        }
        if self.used_live_funds || config.live_funds_approval_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_LIVE_FUNDS_DENIED",
                format!("{context} must not use or approve live funds"),
            ));
        }
        if self.contains_secret_material || config.secret_material_in_evidence_allowed {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_SECRET_MATERIAL_DENIED",
                format!("{context} must not contain secret material"),
            ));
        }
        if self.approves_public_exposure || config.public_exposure_approval_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_PUBLIC_EXPOSURE_DENIED",
                format!("{context} must not approve public exposure"),
            ));
        }
        if matches!(self.status, HardeningEvidenceStatus::ExternallyObserved)
            && self
                .evidence_reference
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EXTERNAL_EVIDENCE_REFERENCE_REQUIRED",
                format!("{context} needs a non-secret reference when externally observed"),
            ));
        }
        for (field, value) in [
            ("evidence_reference", self.evidence_reference.as_deref()),
            ("performed_by", self.performed_by.as_deref()),
            ("external_environment", self.external_environment.as_deref()),
        ] {
            if let Some(value) = value {
                if value.trim().is_empty() {
                    violations.push(ExternalHardeningViolation::new(
                        "HARDENING_OPTIONAL_FIELD_EMPTY",
                        format!("{context} {field} must be omitted rather than empty"),
                    ));
                }
                if contains_secret_like_text(value) {
                    violations.push(ExternalHardeningViolation::new(
                        "HARDENING_OPTIONAL_FIELD_SECRET_LIKE",
                        format!("{context} {field} contains secret-like text"),
                    ));
                }
            }
        }
    }
}

/// Production hardening plan that records required external evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionHardeningPlan {
    /// Stable plan identifier.
    pub plan_id: String,
    /// Human-readable label.
    pub label: String,
    /// Evidence records to review.
    pub evidence_records: Vec<HardeningEvidenceRecord>,
    /// Explicit release blockers or open risks.
    pub release_blockers: Vec<String>,
    /// Whether the plan requests production readiness approval. Phase 17 requires false.
    pub production_readiness_requested: bool,
    /// Whether the plan requests live-funds approval. Phase 17 requires false.
    pub live_funds_requested: bool,
    /// Whether public exposure is requested. Phase 17 requires false.
    pub public_exposure_requested: bool,
}

impl ProductionHardeningPlan {
    /// Construct a conservative external hardening plan with all evidence pending.
    #[must_use]
    pub fn conservative(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            label: "phase-17-external-hardening-evidence-boundary".to_owned(),
            evidence_records: vec![
                HardeningEvidenceRecord::planned(
                    "rust-workspace-validation",
                    ExternalHardeningActivityKind::RustWorkspaceValidation,
                    "Run cargo fmt, check, test, and clippy in a capable external environment",
                ),
                HardeningEvidenceRecord::planned(
                    "dependency-audit",
                    ExternalHardeningActivityKind::DependencyAudit,
                    "Run dependency audit and vulnerability review externally",
                ),
                HardeningEvidenceRecord::planned(
                    "release-build",
                    ExternalHardeningActivityKind::ReleaseBuildValidation,
                    "Build release artifacts externally without embedded secrets",
                ),
                HardeningEvidenceRecord::planned(
                    "container-image-scan",
                    ExternalHardeningActivityKind::ContainerBuildAndScan,
                    "Build and scan container image externally",
                ),
                HardeningEvidenceRecord::planned(
                    "staging-deployment",
                    ExternalHardeningActivityKind::StagingDeploymentValidation,
                    "Validate hardened staging deployment without live funds",
                ),
                HardeningEvidenceRecord::planned(
                    "load-and-soak",
                    ExternalHardeningActivityKind::LoadAndSoakTest,
                    "Run load and soak testing in a non-production environment",
                ),
                HardeningEvidenceRecord::planned(
                    "penetration-test",
                    ExternalHardeningActivityKind::PenetrationTest,
                    "Run AppSec review and penetration test externally",
                ),
                HardeningEvidenceRecord::planned(
                    "rollback-drill",
                    ExternalHardeningActivityKind::RollbackDrill,
                    "Execute rollback drill with no live funds or secrets",
                ),
                HardeningEvidenceRecord::planned(
                    "incident-response-drill",
                    ExternalHardeningActivityKind::IncidentResponseDrill,
                    "Execute incident-response drill and document lessons learned",
                ),
                HardeningEvidenceRecord::planned(
                    "key-custody-review",
                    ExternalHardeningActivityKind::KeyCustodyReview,
                    "Review signer and custody design before any live-funds work",
                ),
            ],
            release_blockers: vec![
                "Rust/Cargo validation has not been executed in this environment".to_owned(),
                "No live exchange, DEX/RPC, signer, custody, or broadcast validation exists".to_owned(),
                "No production deployment, penetration test, load test, or rollback drill has been executed".to_owned(),
            ],
            production_readiness_requested: false,
            live_funds_requested: false,
            public_exposure_requested: false,
        }
    }

    fn validate(
        &self,
        config: &ExternalHardeningBoundaryConfig,
    ) -> Result<(), ExternalHardeningError> {
        let mut violations = Vec::new();

        if self.plan_id.trim().is_empty() {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PLAN_ID_EMPTY",
                "plan_id must be non-empty",
            ));
        }
        if self.label.trim().is_empty() {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PLAN_LABEL_EMPTY",
                "plan label must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.label) {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PLAN_LABEL_SECRET_LIKE",
                "plan label contains secret-like text",
            ));
        }
        if self.evidence_records.is_empty() {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_EVIDENCE_EMPTY",
                "hardening plan must include evidence records",
            ));
        }
        if self.release_blockers.is_empty() {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_RELEASE_BLOCKERS_EMPTY",
                "hardening plan must include explicit release blockers",
            ));
        }
        if self.production_readiness_requested || config.production_readiness_claims_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PLAN_PRODUCTION_READINESS_DENIED",
                "hardening plan must not request production readiness approval",
            ));
        }
        if self.live_funds_requested || config.live_funds_approval_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PLAN_LIVE_FUNDS_DENIED",
                "hardening plan must not request live-funds approval",
            ));
        }
        if self.public_exposure_requested || config.public_exposure_approval_enabled {
            violations.push(ExternalHardeningViolation::new(
                "HARDENING_PLAN_PUBLIC_EXPOSURE_DENIED",
                "hardening plan must not request public exposure approval",
            ));
        }

        let mut evidence_ids = BTreeSet::new();
        for evidence in &self.evidence_records {
            if !evidence_ids.insert(evidence.evidence_id.clone()) {
                violations.push(ExternalHardeningViolation::new(
                    "HARDENING_DUPLICATE_EVIDENCE_ID",
                    format!("duplicate evidence id {}", evidence.evidence_id),
                ));
            }
            evidence.validate(config, &mut violations);
        }

        let mut blockers = BTreeSet::new();
        for blocker in &self.release_blockers {
            if blocker.trim().is_empty() {
                violations.push(ExternalHardeningViolation::new(
                    "HARDENING_RELEASE_BLOCKER_EMPTY",
                    "release blockers must be non-empty",
                ));
            }
            if contains_secret_like_text(blocker) {
                violations.push(ExternalHardeningViolation::new(
                    "HARDENING_RELEASE_BLOCKER_SECRET_LIKE",
                    "release blocker contains secret-like text",
                ));
            }
            if !blockers.insert(blocker.clone()) {
                violations.push(ExternalHardeningViolation::new(
                    "HARDENING_DUPLICATE_RELEASE_BLOCKER",
                    format!("duplicate release blocker {blocker}"),
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Request to review a Phase 17 hardening plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalHardeningReviewRequest {
    /// Boundary configuration.
    pub config: ExternalHardeningBoundaryConfig,
    /// Plan to validate and record.
    pub plan: ProductionHardeningPlan,
    /// Operator or automation label creating the review.
    pub requested_by: String,
}

impl ExternalHardeningReviewRequest {
    /// Construct a conservative request.
    #[must_use]
    pub fn conservative(plan_id: impl Into<String>, requested_by: impl Into<String>) -> Self {
        Self {
            config: ExternalHardeningBoundaryConfig::default(),
            plan: ProductionHardeningPlan::conservative(plan_id),
            requested_by: requested_by.into(),
        }
    }
}

/// Hardening review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalHardeningReviewStatus {
    /// Plan was accepted as a pending hardening evidence checklist.
    PendingExternalEvidence,
    /// Plan was rejected due to boundary violations.
    Rejected,
}

/// Deterministic external hardening review record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalHardeningReviewRecord {
    /// Reviewed plan id.
    pub plan_id: String,
    /// Review status.
    pub status: ExternalHardeningReviewStatus,
    /// Total evidence record count.
    pub evidence_count: usize,
    /// Count of externally observed evidence records.
    pub externally_observed_count: usize,
    /// Count of pending or blocked external records.
    pub unresolved_evidence_count: usize,
    /// Count of explicit release blockers.
    pub release_blocker_count: usize,
    /// Whether external hardening was performed by this boundary. Always false in Phase 17.
    pub external_hardening_performed_by_boundary: bool,
    /// Whether production readiness is approved. Always false in Phase 17.
    pub production_ready: bool,
    /// Whether live funds are approved. Always false in Phase 17.
    pub live_funds_approved: bool,
    /// Whether public exposure is approved. Always false in Phase 17.
    pub public_exposure_approved: bool,
    /// Whether secret material was observed in evidence. Always false for accepted records.
    pub secret_material_recorded: bool,
    /// Violations when rejected.
    pub violations: Vec<ExternalHardeningViolation>,
}

impl ExternalHardeningReviewRecord {
    fn pending(plan: &ProductionHardeningPlan) -> Self {
        let externally_observed_count = plan
            .evidence_records
            .iter()
            .filter(|record| record.status == HardeningEvidenceStatus::ExternallyObserved)
            .count();
        let unresolved_evidence_count = plan
            .evidence_records
            .iter()
            .filter(|record| record.status != HardeningEvidenceStatus::ExternallyObserved)
            .count();

        Self {
            plan_id: plan.plan_id.clone(),
            status: ExternalHardeningReviewStatus::PendingExternalEvidence,
            evidence_count: plan.evidence_records.len(),
            externally_observed_count,
            unresolved_evidence_count,
            release_blocker_count: plan.release_blockers.len(),
            external_hardening_performed_by_boundary: false,
            production_ready: false,
            live_funds_approved: false,
            public_exposure_approved: false,
            secret_material_recorded: false,
            violations: Vec::new(),
        }
    }

    fn rejected(plan_id: impl Into<String>, violations: Vec<ExternalHardeningViolation>) -> Self {
        Self {
            plan_id: plan_id.into(),
            status: ExternalHardeningReviewStatus::Rejected,
            evidence_count: 0,
            externally_observed_count: 0,
            unresolved_evidence_count: 0,
            release_blocker_count: 0,
            external_hardening_performed_by_boundary: false,
            production_ready: false,
            live_funds_approved: false,
            public_exposure_approved: false,
            secret_material_recorded: false,
            violations,
        }
    }
}

/// Phase 17 hardening review boundary.
pub trait ExternalHardeningReviewer {
    /// Validate a production hardening plan and emit a local review record.
    fn review_hardening(
        &self,
        request: ExternalHardeningReviewRequest,
    ) -> Result<ExternalHardeningReviewRecord, ExternalHardeningError>;
}

/// Deterministic model-only implementation of the Phase 17 hardening boundary.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicExternalHardeningReviewer;

impl ExternalHardeningReviewer for DeterministicExternalHardeningReviewer {
    fn review_hardening(
        &self,
        request: ExternalHardeningReviewRequest,
    ) -> Result<ExternalHardeningReviewRecord, ExternalHardeningError> {
        request.config.validate()?;

        let mut request_violations = Vec::new();
        if request.requested_by.trim().is_empty() {
            request_violations.push(ExternalHardeningViolation::new(
                "HARDENING_REQUESTED_BY_EMPTY",
                "requested_by must be non-empty",
            ));
        }
        if contains_secret_like_text(&request.requested_by) {
            request_violations.push(ExternalHardeningViolation::new(
                "HARDENING_REQUESTED_BY_SECRET_LIKE",
                "requested_by contains secret-like text",
            ));
        }
        if !request_violations.is_empty() {
            return Err(ExternalHardeningError::ValidationFailed {
                violations: request_violations,
            });
        }

        match request.plan.validate(&request.config) {
            Ok(()) => Ok(ExternalHardeningReviewRecord::pending(&request.plan)),
            Err(ExternalHardeningError::ValidationFailed { violations }) => Ok(
                ExternalHardeningReviewRecord::rejected(request.plan.plan_id, violations),
            ),
        }
    }
}

/// External hardening violation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalHardeningViolation {
    /// Stable violation code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl ExternalHardeningViolation {
    /// Construct a hardening violation.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// External hardening boundary error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalHardeningError {
    /// Validation failed with one or more violations.
    ValidationFailed {
        /// Collected validation violations.
        violations: Vec<ExternalHardeningViolation>,
    },
}

impl fmt::Display for ExternalHardeningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(f, "external hardening validation failed")?;
                for violation in violations {
                    write!(f, "; {}: {}", violation.code, violation.message)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ExternalHardeningError {}

fn finish_validation(
    violations: Vec<ExternalHardeningViolation>,
) -> Result<(), ExternalHardeningError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ExternalHardeningError::ValidationFailed { violations })
    }
}

fn contains_secret_like_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "private_key",
        "seed phrase",
        "mnemonic",
        "bearer ",
        "wallet key",
        "provider token",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{
        DeterministicExternalHardeningReviewer, ExternalHardeningBoundaryConfig,
        ExternalHardeningError, ExternalHardeningReviewRequest, ExternalHardeningReviewStatus,
        ExternalHardeningReviewer, HardeningEvidenceStatus, ProductionHardeningPlan,
    };

    #[test]
    fn conservative_hardening_plan_records_pending_evidence_only() {
        let reviewer = DeterministicExternalHardeningReviewer;
        let record = reviewer
            .review_hardening(ExternalHardeningReviewRequest::conservative(
                "phase-17-hardening",
                "local-operator",
            ))
            .expect("conservative hardening plan should be valid");

        assert_eq!(
            record.status,
            ExternalHardeningReviewStatus::PendingExternalEvidence
        );
        assert!(record.evidence_count > 0);
        assert_eq!(record.externally_observed_count, 0);
        assert!(record.unresolved_evidence_count > 0);
        assert!(!record.external_hardening_performed_by_boundary);
        assert!(!record.production_ready);
        assert!(!record.live_funds_approved);
        assert!(!record.public_exposure_approved);
        assert!(!record.secret_material_recorded);
    }

    #[test]
    fn externally_observed_evidence_requires_reference() {
        let reviewer = DeterministicExternalHardeningReviewer;
        let mut plan = ProductionHardeningPlan::conservative("phase-17-missing-reference");
        plan.evidence_records[0].status = HardeningEvidenceStatus::ExternallyObserved;

        let record = reviewer
            .review_hardening(ExternalHardeningReviewRequest {
                config: ExternalHardeningBoundaryConfig::default(),
                plan,
                requested_by: "local-operator".to_owned(),
            })
            .expect("plan-level denials should produce rejected record");

        assert_eq!(record.status, ExternalHardeningReviewStatus::Rejected);
        assert!(record.violations.iter().any(|violation| {
            violation.code == "HARDENING_EXTERNAL_EVIDENCE_REFERENCE_REQUIRED"
        }));
    }

    #[test]
    fn production_readiness_requests_are_rejected() {
        let reviewer = DeterministicExternalHardeningReviewer;
        let mut plan = ProductionHardeningPlan::conservative("phase-17-prod-denial");
        plan.production_readiness_requested = true;

        let record = reviewer
            .review_hardening(ExternalHardeningReviewRequest {
                config: ExternalHardeningBoundaryConfig::default(),
                plan,
                requested_by: "local-operator".to_owned(),
            })
            .expect("plan-level denials should produce rejected record");

        assert_eq!(record.status, ExternalHardeningReviewStatus::Rejected);
        assert!(record
            .violations
            .iter()
            .any(|violation| { violation.code == "HARDENING_PLAN_PRODUCTION_READINESS_DENIED" }));
    }

    #[test]
    fn config_rejects_external_action_claims() {
        let config = ExternalHardeningBoundaryConfig {
            external_actions_enabled: true,
            ..ExternalHardeningBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("external action claims must fail closed");

        match error {
            ExternalHardeningError::ValidationFailed { violations } => assert!(violations
                .iter()
                .any(|violation| violation.code == "HARDENING_EXTERNAL_ACTIONS_DENIED")),
        }
    }
}
