#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable agentic handoff boundary version for audit and continuation surfaces.
pub const AGENTIC_HANDOFF_VERSION: &str = "phase-18-agentic-handoff-v1";

/// Conservative Phase 18 handoff settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgenticHandoffBoundaryConfig {
    /// Whether local handoff package records may be generated.
    pub handoff_generation_enabled: bool,
    /// Whether this boundary may execute external agents. Phase 18 requires false.
    pub external_agent_execution_enabled: bool,
    /// Whether this boundary may claim external validation passed. Phase 18 requires false.
    pub external_validation_claims_enabled: bool,
    /// Whether this boundary may claim production readiness. Phase 18 requires false.
    pub production_readiness_claims_enabled: bool,
    /// Whether this boundary may approve live funds. Phase 18 requires false.
    pub live_funds_approval_enabled: bool,
    /// Whether handoff artifacts may contain secret material. Phase 18 requires false.
    pub secret_material_in_handoff_allowed: bool,
    /// Whether handoff artifacts may approve public exposure. Phase 18 requires false.
    pub public_exposure_approval_enabled: bool,
    /// Whether unresolved gaps must be preserved in generated packages. Phase 18 requires true.
    pub gap_preservation_required: bool,
}

impl Default for AgenticHandoffBoundaryConfig {
    fn default() -> Self {
        Self {
            handoff_generation_enabled: true,
            external_agent_execution_enabled: false,
            external_validation_claims_enabled: false,
            production_readiness_claims_enabled: false,
            live_funds_approval_enabled: false,
            secret_material_in_handoff_allowed: false,
            public_exposure_approval_enabled: false,
            gap_preservation_required: true,
        }
    }
}

impl AgenticHandoffBoundaryConfig {
    /// Validate fail-closed Phase 18 handoff settings.
    pub fn validate(&self) -> Result<(), AgenticHandoffError> {
        let mut violations = Vec::new();

        if !self.handoff_generation_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_GENERATION_DISABLED",
                "Phase 18 requires local handoff package generation to remain enabled",
            ));
        }
        if self.external_agent_execution_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_EXTERNAL_AGENT_EXECUTION_DENIED",
                "Phase 18 must not execute external coding agents",
            ));
        }
        if self.external_validation_claims_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_EXTERNAL_VALIDATION_CLAIMS_DENIED",
                "Phase 18 must not claim external validation passed",
            ));
        }
        if self.production_readiness_claims_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_PRODUCTION_CLAIMS_DENIED",
                "Phase 18 must not claim production readiness",
            ));
        }
        if self.live_funds_approval_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_LIVE_FUNDS_APPROVAL_DENIED",
                "Phase 18 must not approve live funds",
            ));
        }
        if self.secret_material_in_handoff_allowed {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_SECRET_MATERIAL_DENIED",
                "handoff artifacts must not contain secret material",
            ));
        }
        if self.public_exposure_approval_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_PUBLIC_EXPOSURE_APPROVAL_DENIED",
                "Phase 18 must not approve public service exposure",
            ));
        }
        if !self.gap_preservation_required {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_GAP_PRESERVATION_REQUIRED",
                "Phase 18 handoffs must preserve unresolved gaps",
            ));
        }

        finish_validation(violations)
    }
}

/// Target recipient category for handoff guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HandoffAgentKind {
    /// OpenAI Codex or equivalent repository coding agent.
    Codex,
    /// Cursor or local IDE agent.
    Cursor,
    /// Jules or asynchronous coding agent.
    Jules,
    /// Claude or equivalent code-review/coding assistant.
    Claude,
    /// Human maintainer or project owner.
    HumanMaintainer,
    /// CI or build-validation automation.
    CiAutomation,
    /// DevSecOps/release-engineering operator.
    DevSecOps,
    /// AppSec/security-review operator.
    AppSec,
    /// Rust implementation specialist.
    RustImplementation,
    /// Compliance, legal, or terms reviewer.
    ComplianceReviewer,
}

/// Handoff artifact category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HandoffArtifactKind {
    /// Continuation prompt for a future agent.
    ContinuationPrompt,
    /// Governance reconciliation checklist.
    GovernanceChecklist,
    /// External validation checklist.
    ExternalValidationChecklist,
    /// Security warning or hard blocker summary.
    SecurityWarning,
    /// Gap tracker summary.
    GapSummary,
    /// Rollback and recovery guidance.
    RollbackGuide,
    /// Agent-specific role instructions.
    AgentRoleInstructions,
}

/// Deterministic handoff artifact for future agents or maintainers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffInstructionArtifact {
    /// Stable artifact identifier.
    pub artifact_id: String,
    /// Artifact category.
    pub artifact_kind: HandoffArtifactKind,
    /// Intended target agent or maintainer.
    pub target_agent_kind: HandoffAgentKind,
    /// Human-readable artifact title.
    pub title: String,
    /// Non-secret artifact body.
    pub body: String,
    /// Governance files that must be read before using this artifact.
    pub required_files: Vec<String>,
    /// Gap IDs explicitly preserved by this artifact.
    pub preserved_gap_ids: Vec<String>,
    /// Whether the artifact contains secret material. Phase 18 requires false.
    pub contains_secret_material: bool,
    /// Whether the artifact claims external validation passed. Phase 18 requires false.
    pub claims_external_validation: bool,
    /// Whether the artifact claims production readiness. Phase 18 requires false.
    pub claims_production_ready: bool,
    /// Whether the artifact approves live funds. Phase 18 requires false.
    pub approves_live_funds: bool,
    /// Whether the artifact enables live execution. Phase 18 requires false.
    pub enables_live_execution: bool,
    /// Whether the artifact approves public service exposure. Phase 18 requires false.
    pub approves_public_exposure: bool,
}

impl HandoffInstructionArtifact {
    /// Construct a conservative non-executing handoff artifact.
    #[must_use]
    pub fn conservative(
        artifact_id: impl Into<String>,
        artifact_kind: HandoffArtifactKind,
        target_agent_kind: HandoffAgentKind,
        title: impl Into<String>,
        body: impl Into<String>,
        required_files: Vec<String>,
        preserved_gap_ids: Vec<String>,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            artifact_kind,
            target_agent_kind,
            title: title.into(),
            body: body.into(),
            required_files,
            preserved_gap_ids,
            contains_secret_material: false,
            claims_external_validation: false,
            claims_production_ready: false,
            approves_live_funds: false,
            enables_live_execution: false,
            approves_public_exposure: false,
        }
    }

    fn validate(
        &self,
        config: &AgenticHandoffBoundaryConfig,
        violations: &mut Vec<AgenticHandoffViolation>,
    ) {
        let context = format!("artifact {}", self.artifact_id);
        if self.artifact_id.trim().is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_ID_EMPTY",
                "artifact_id must be non-empty",
            ));
        }
        if self.title.trim().is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_TITLE_EMPTY",
                format!("{context} title must be non-empty"),
            ));
        }
        if self.body.trim().is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_BODY_EMPTY",
                format!("{context} body must be non-empty"),
            ));
        }
        if self.required_files.is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_REQUIRED_FILES_EMPTY",
                format!("{context} must name required governance files"),
            ));
        }
        if config.gap_preservation_required && self.preserved_gap_ids.is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_GAPS_EMPTY",
                format!("{context} must preserve at least one unresolved gap id"),
            ));
        }
        if looks_like_secret_assignment(&self.title) || looks_like_secret_assignment(&self.body) {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_SECRET_LIKE_TEXT",
                format!("{context} contains secret-like assignment text"),
            ));
        }
        if self.contains_secret_material && !config.secret_material_in_handoff_allowed {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_SECRET_MATERIAL_DENIED",
                format!("{context} declares secret material"),
            ));
        }
        if self.claims_external_validation && !config.external_validation_claims_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_EXTERNAL_VALIDATION_CLAIM_DENIED",
                format!("{context} claims external validation"),
            ));
        }
        if self.claims_production_ready && !config.production_readiness_claims_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_PRODUCTION_CLAIM_DENIED",
                format!("{context} claims production readiness"),
            ));
        }
        if self.approves_live_funds && !config.live_funds_approval_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_LIVE_FUNDS_APPROVAL_DENIED",
                format!("{context} approves live funds"),
            ));
        }
        if self.enables_live_execution {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_LIVE_EXECUTION_DENIED",
                format!("{context} enables live execution"),
            ));
        }
        if self.approves_public_exposure && !config.public_exposure_approval_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACT_PUBLIC_EXPOSURE_DENIED",
                format!("{context} approves public exposure"),
            ));
        }
    }
}

/// Deterministic package of handoff records for future agents and maintainers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgenticHandoffPackage {
    /// Stable package identifier.
    pub package_id: String,
    /// Source phase used as the authority checkpoint.
    pub source_phase: String,
    /// Next target phase for downstream work.
    pub target_phase: String,
    /// Required authoritative files.
    pub authoritative_files: Vec<String>,
    /// Completed phase labels preserved for continuity.
    pub completed_phases: Vec<String>,
    /// Generated handoff artifacts.
    pub artifacts: Vec<HandoffInstructionArtifact>,
    /// Unresolved gap IDs carried forward.
    pub unresolved_gap_ids: Vec<String>,
    /// Live-funds blockers carried forward.
    pub live_funds_blockers: Vec<String>,
    /// Next required action after this package.
    pub next_required_action: String,
    /// Governance-readiness approximation copied from roadmap, not a production claim.
    pub production_readiness_percent: u8,
    /// Whether this package executed external agents. Phase 18 requires false.
    pub external_agents_executed: bool,
    /// Whether this package claims external validation passed. Phase 18 requires false.
    pub external_validation_claimed: bool,
    /// Whether this package claims production readiness. Phase 18 requires false.
    pub claims_production_ready: bool,
    /// Whether this package contains secret material. Phase 18 requires false.
    pub contains_secret_material: bool,
    /// Whether this package approves public service exposure. Phase 18 requires false.
    pub public_exposure_approved: bool,
    /// Whether this package approves live funds. Phase 18 requires false.
    pub live_funds_approved: bool,
}

impl AgenticHandoffPackage {
    /// Construct a conservative Phase 18 handoff package.
    #[must_use]
    pub fn conservative(package_id: impl Into<String>) -> Self {
        let authoritative_files = authoritative_governance_files();
        let unresolved_gap_ids = unresolved_gap_ids();
        let live_funds_blockers = vec![
            "real funds are not approved".to_owned(),
            "live exchange credentials are not approved".to_owned(),
            "wallet signing and broadcasts are not implemented".to_owned(),
            "exchange-specific live adapters are not implemented".to_owned(),
            "external hardening evidence is not executed".to_owned(),
            "production readiness is not approved".to_owned(),
        ];

        let artifact_required_files = authoritative_files.clone();
        let artifact_gap_ids = unresolved_gap_ids.clone();

        Self {
            package_id: package_id.into(),
            source_phase: "Phase 18 — Agentic Handoff Package".to_owned(),
            target_phase: "External validation and future implementation under governance".to_owned(),
            authoritative_files,
            completed_phases: (0..=18).map(|phase| format!("Phase {phase}")).collect(),
            artifacts: vec![
                HandoffInstructionArtifact::conservative(
                    "handoff-continuation-prompt",
                    HandoffArtifactKind::ContinuationPrompt,
                    HandoffAgentKind::Codex,
                    "Repository continuation prompt",
                    "Inspect the latest repository checkout or approved archive, read governance files before changes, run the structure validator, preserve all unresolved gaps, and do not enable live trading, signing, broadcasts, withdrawals, bridges, public exposure, or production claims.",
                    artifact_required_files.clone(),
                    artifact_gap_ids.clone(),
                ),
                HandoffInstructionArtifact::conservative(
                    "handoff-governance-checklist",
                    HandoffArtifactKind::GovernanceChecklist,
                    HandoffAgentKind::HumanMaintainer,
                    "Governance reconciliation checklist",
                    "Before work, reconcile ARCHITECTURE, ROADMAP, active sub-roadmap, AGENTS, gap tracker, handoff context, and manifest. Stop on conflicts and record assumptions, blockers, and safest next step.",
                    artifact_required_files.clone(),
                    artifact_gap_ids.clone(),
                ),
                HandoffInstructionArtifact::conservative(
                    "handoff-external-validation-checklist",
                    HandoffArtifactKind::ExternalValidationChecklist,
                    HandoffAgentKind::DevSecOps,
                    "External validation checklist",
                    "Run Rust formatting, check, tests, clippy, release build, dependency audit, SBOM review, container scan, service hardening, ARM validation, staging deployment, load test, penetration test, rollback drill, and incident drill outside ChatGPT Project Mode before any production claim.",
                    artifact_required_files.clone(),
                    artifact_gap_ids.clone(),
                ),
                HandoffInstructionArtifact::conservative(
                    "handoff-security-warning",
                    HandoffArtifactKind::SecurityWarning,
                    HandoffAgentKind::AppSec,
                    "Security blockers",
                    "Do not introduce credentials, signing material, live orders, live swaps, wallet broadcasts, public dashboard exposure, public metrics exposure, or outbound messaging integrations without explicit later design, tests, external validation, and accountable approval.",
                    artifact_required_files,
                    artifact_gap_ids,
                ),
            ],
            unresolved_gap_ids,
            live_funds_blockers,
            next_required_action: "Refresh local/CI Rust validation after changes and run production-hardening evidence generation before any production or live-funds claim".to_owned(),
            production_readiness_percent: 87,
            external_agents_executed: false,
            external_validation_claimed: false,
            claims_production_ready: false,
            contains_secret_material: false,
            public_exposure_approved: false,
            live_funds_approved: false,
        }
    }

    /// Validate the handoff package under Phase 18 fail-closed rules.
    pub fn validate(
        &self,
        config: &AgenticHandoffBoundaryConfig,
    ) -> Result<(), AgenticHandoffError> {
        config.validate()?;
        let mut violations = Vec::new();

        if self.package_id.trim().is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_PACKAGE_ID_EMPTY",
                "package_id must be non-empty",
            ));
        }
        if self.source_phase.trim().is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_SOURCE_PHASE_EMPTY",
                "source_phase must be non-empty",
            ));
        }
        if self.target_phase.trim().is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_TARGET_PHASE_EMPTY",
                "target_phase must be non-empty",
            ));
        }
        if self.authoritative_files.is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_AUTHORITATIVE_FILES_EMPTY",
                "authoritative files must be preserved",
            ));
        }
        if self.completed_phases.len() < 19 {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_COMPLETED_PHASES_INCOMPLETE",
                "completed phases 0 through 18 must be preserved",
            ));
        }
        if self.artifacts.is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_ARTIFACTS_EMPTY",
                "at least one handoff artifact is required",
            ));
        }
        if config.gap_preservation_required && self.unresolved_gap_ids.is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_UNRESOLVED_GAPS_EMPTY",
                "unresolved gaps must be preserved",
            ));
        }
        if self.live_funds_blockers.is_empty() {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_LIVE_FUNDS_BLOCKERS_EMPTY",
                "live-funds blockers must be preserved",
            ));
        }
        if self.production_readiness_percent > 87 {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_READINESS_INCREASE_DENIED",
                "Phase 18 handoff packaging must not increase production readiness",
            ));
        }
        if self.external_agents_executed || config.external_agent_execution_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_EXTERNAL_AGENTS_EXECUTED_DENIED",
                "Phase 18 must not execute external agents",
            ));
        }
        if self.external_validation_claimed || config.external_validation_claims_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_EXTERNAL_VALIDATION_CLAIM_DENIED",
                "Phase 18 must not claim external validation passed",
            ));
        }
        if self.claims_production_ready || config.production_readiness_claims_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_PRODUCTION_CLAIM_DENIED",
                "Phase 18 must not claim production readiness",
            ));
        }
        if self.contains_secret_material || config.secret_material_in_handoff_allowed {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_SECRET_MATERIAL_DENIED",
                "handoff package must not contain secret material",
            ));
        }
        if self.public_exposure_approved || config.public_exposure_approval_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_PUBLIC_EXPOSURE_DENIED",
                "Phase 18 must not approve public exposure",
            ));
        }
        if self.live_funds_approved || config.live_funds_approval_enabled {
            violations.push(AgenticHandoffViolation::new(
                "HANDOFF_LIVE_FUNDS_APPROVAL_DENIED",
                "Phase 18 must not approve live funds",
            ));
        }

        let mut artifact_ids = BTreeSet::new();
        for artifact in &self.artifacts {
            if !artifact_ids.insert(artifact.artifact_id.clone()) {
                violations.push(AgenticHandoffViolation::new(
                    "HANDOFF_DUPLICATE_ARTIFACT_ID",
                    format!("duplicate artifact id {}", artifact.artifact_id),
                ));
            }
            artifact.validate(config, &mut violations);
        }

        finish_validation(violations)
    }
}

/// Review status for a handoff package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgenticHandoffReviewStatus {
    /// Handoff package is valid and ready for external review.
    ReadyForExternalReview,
    /// Handoff package was rejected by local boundary rules.
    Rejected,
}

/// Request to review a handoff package.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgenticHandoffReviewRequest {
    /// Phase 18 boundary config.
    pub config: AgenticHandoffBoundaryConfig,
    /// Handoff package under review.
    pub package: AgenticHandoffPackage,
    /// Non-secret operator or agent label.
    pub requested_by: String,
}

impl AgenticHandoffReviewRequest {
    /// Construct a conservative package review request.
    #[must_use]
    pub fn conservative(package_id: impl Into<String>, requested_by: impl Into<String>) -> Self {
        Self {
            config: AgenticHandoffBoundaryConfig::default(),
            package: AgenticHandoffPackage::conservative(package_id),
            requested_by: requested_by.into(),
        }
    }
}

/// Deterministic record produced by Phase 18 package review.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgenticHandoffReviewRecord {
    /// Package identifier reviewed.
    pub package_id: String,
    /// Review status.
    pub status: AgenticHandoffReviewStatus,
    /// Number of generated handoff artifacts.
    pub artifact_count: usize,
    /// Number of unresolved gaps preserved.
    pub unresolved_gap_count: usize,
    /// Number of live-funds blockers preserved.
    pub live_funds_blocker_count: usize,
    /// Whether this boundary executed external agents. Phase 18 requires false.
    pub external_agents_executed: bool,
    /// Whether this record claims external validation passed. Phase 18 requires false.
    pub external_validation_claimed: bool,
    /// Whether this record claims production readiness. Phase 18 requires false.
    pub production_ready: bool,
    /// Whether this record approves live funds. Phase 18 requires false.
    pub live_funds_approved: bool,
    /// Whether this record approves public exposure. Phase 18 requires false.
    pub public_exposure_approved: bool,
    /// Whether this record contains secret material. Phase 18 requires false.
    pub secret_material_recorded: bool,
    /// Validation violations, if rejected.
    pub violations: Vec<AgenticHandoffViolation>,
}

impl AgenticHandoffReviewRecord {
    fn ready(package: &AgenticHandoffPackage) -> Self {
        Self {
            package_id: package.package_id.clone(),
            status: AgenticHandoffReviewStatus::ReadyForExternalReview,
            artifact_count: package.artifacts.len(),
            unresolved_gap_count: package.unresolved_gap_ids.len(),
            live_funds_blocker_count: package.live_funds_blockers.len(),
            external_agents_executed: false,
            external_validation_claimed: false,
            production_ready: false,
            live_funds_approved: false,
            public_exposure_approved: false,
            secret_material_recorded: false,
            violations: Vec::new(),
        }
    }

    fn rejected(package_id: String, violations: Vec<AgenticHandoffViolation>) -> Self {
        Self {
            package_id,
            status: AgenticHandoffReviewStatus::Rejected,
            artifact_count: 0,
            unresolved_gap_count: 0,
            live_funds_blocker_count: 0,
            external_agents_executed: false,
            external_validation_claimed: false,
            production_ready: false,
            live_funds_approved: false,
            public_exposure_approved: false,
            secret_material_recorded: false,
            violations,
        }
    }
}

/// Phase 18 handoff package boundary.
pub trait AgenticHandoffPackager {
    /// Validate a handoff package and emit a local review record.
    fn review_package(
        &self,
        request: AgenticHandoffReviewRequest,
    ) -> Result<AgenticHandoffReviewRecord, AgenticHandoffError>;
}

/// Deterministic model-only implementation of the Phase 18 handoff boundary.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicAgenticHandoffPackager;

impl AgenticHandoffPackager for DeterministicAgenticHandoffPackager {
    fn review_package(
        &self,
        request: AgenticHandoffReviewRequest,
    ) -> Result<AgenticHandoffReviewRecord, AgenticHandoffError> {
        request.config.validate()?;

        let mut request_violations = Vec::new();
        if request.requested_by.trim().is_empty() {
            request_violations.push(AgenticHandoffViolation::new(
                "HANDOFF_REQUESTED_BY_EMPTY",
                "requested_by must be non-empty",
            ));
        }
        if looks_like_secret_assignment(&request.requested_by) {
            request_violations.push(AgenticHandoffViolation::new(
                "HANDOFF_REQUESTED_BY_SECRET_LIKE",
                "requested_by contains secret-like assignment text",
            ));
        }
        if !request_violations.is_empty() {
            return Err(AgenticHandoffError::ValidationFailed {
                violations: request_violations,
            });
        }

        match request.package.validate(&request.config) {
            Ok(()) => Ok(AgenticHandoffReviewRecord::ready(&request.package)),
            Err(AgenticHandoffError::ValidationFailed { violations }) => Ok(
                AgenticHandoffReviewRecord::rejected(request.package.package_id, violations),
            ),
        }
    }
}

/// Agentic handoff violation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgenticHandoffViolation {
    /// Stable violation code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl AgenticHandoffViolation {
    /// Construct a handoff violation.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Agentic handoff boundary error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgenticHandoffError {
    /// Validation failed with one or more violations.
    ValidationFailed {
        /// Collected validation violations.
        violations: Vec<AgenticHandoffViolation>,
    },
}

impl fmt::Display for AgenticHandoffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(f, "agentic handoff validation failed")?;
                for violation in violations {
                    write!(f, "; {}: {}", violation.code, violation.message)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for AgenticHandoffError {}

fn authoritative_governance_files() -> Vec<String> {
    vec![
        "HANDOFF_CONTEXT.md".to_owned(),
        "STRUCTURE_MANIFEST.md".to_owned(),
        "ARCHITECTURE.md".to_owned(),
        "ROADMAP.md".to_owned(),
        "AGENTS.md".to_owned(),
        "PHASE_18_SUBROADMAP.md".to_owned(),
        "PRODUCTION_GAP_TRACKER.md".to_owned(),
    ]
}

fn unresolved_gap_ids() -> Vec<String> {
    vec![
        "GAP-0057".to_owned(),
        "GAP-0059".to_owned(),
        "GAP-0061".to_owned(),
        "GAP-0063".to_owned(),
        "GAP-0065".to_owned(),
        "GAP-0066".to_owned(),
        "GAP-0067".to_owned(),
        "GAP-0068".to_owned(),
        "GAP-0069".to_owned(),
        "GAP-0070".to_owned(),
    ]
}

fn finish_validation(violations: Vec<AgenticHandoffViolation>) -> Result<(), AgenticHandoffError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(AgenticHandoffError::ValidationFailed { violations })
    }
}

fn looks_like_secret_assignment(value: &str) -> bool {
    value.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        let mentions_secret_name = [
            "api_key",
            "apikey",
            "private_key",
            "seed_phrase",
            "mnemonic",
            "bearer_token",
            "provider_token",
        ]
        .iter()
        .any(|needle| lower.contains(needle));

        if !mentions_secret_name || !(line.contains('=') || line.contains(':')) {
            return false;
        }

        line.split(['=', ':'])
            .nth(1)
            .map(str::trim)
            .is_some_and(|candidate| {
                candidate.len() >= 12 && candidate.chars().any(char::is_alphanumeric)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AgenticHandoffBoundaryConfig, AgenticHandoffError, AgenticHandoffPackage,
        AgenticHandoffPackager, AgenticHandoffReviewRequest, AgenticHandoffReviewStatus,
        DeterministicAgenticHandoffPackager,
    };

    #[test]
    fn conservative_handoff_package_is_ready_for_external_review() {
        let packager = DeterministicAgenticHandoffPackager;
        let record = packager
            .review_package(AgenticHandoffReviewRequest::conservative(
                "phase-18-handoff",
                "local-operator",
            ))
            .expect("conservative handoff package should be valid");

        assert_eq!(
            record.status,
            AgenticHandoffReviewStatus::ReadyForExternalReview
        );
        assert!(record.artifact_count >= 4);
        assert!(record.unresolved_gap_count >= 1);
        assert!(record.live_funds_blocker_count >= 1);
        assert!(!record.external_agents_executed);
        assert!(!record.external_validation_claimed);
        assert!(!record.production_ready);
        assert!(!record.live_funds_approved);
        assert!(!record.public_exposure_approved);
        assert!(!record.secret_material_recorded);
    }

    #[test]
    fn production_readiness_claims_are_rejected() {
        let packager = DeterministicAgenticHandoffPackager;
        let mut package = AgenticHandoffPackage::conservative("phase-18-prod-denial");
        package.claims_production_ready = true;

        let record = packager
            .review_package(AgenticHandoffReviewRequest {
                config: AgenticHandoffBoundaryConfig::default(),
                package,
                requested_by: "local-operator".to_owned(),
            })
            .expect("package-level denials should produce rejected record");

        assert_eq!(record.status, AgenticHandoffReviewStatus::Rejected);
        assert!(record
            .violations
            .iter()
            .any(|violation| violation.code == "HANDOFF_PRODUCTION_CLAIM_DENIED"));
    }

    #[test]
    fn gap_preservation_is_required() {
        let packager = DeterministicAgenticHandoffPackager;
        let mut package = AgenticHandoffPackage::conservative("phase-18-gap-denial");
        package.unresolved_gap_ids.clear();

        let record = packager
            .review_package(AgenticHandoffReviewRequest {
                config: AgenticHandoffBoundaryConfig::default(),
                package,
                requested_by: "local-operator".to_owned(),
            })
            .expect("package-level denials should produce rejected record");

        assert_eq!(record.status, AgenticHandoffReviewStatus::Rejected);
        assert!(record
            .violations
            .iter()
            .any(|violation| violation.code == "HANDOFF_UNRESOLVED_GAPS_EMPTY"));
    }

    #[test]
    fn config_rejects_external_agent_execution() {
        let config = AgenticHandoffBoundaryConfig {
            external_agent_execution_enabled: true,
            ..AgenticHandoffBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("external agent execution must fail closed");

        match error {
            AgenticHandoffError::ValidationFailed { violations } => assert!(violations
                .iter()
                .any(|violation| violation.code == "HANDOFF_EXTERNAL_AGENT_EXECUTION_DENIED")),
        }
    }
}
