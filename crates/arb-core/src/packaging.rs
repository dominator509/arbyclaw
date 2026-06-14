#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, StateCheckpoint,
    StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable packaging/deployment boundary version for audit and handoff surfaces.
pub const PACKAGING_DEPLOYMENT_VERSION: &str = "phase-16-packaging-deployment-v1";

/// Stable local rollback execution transcript validation version.
pub const PACKAGING_ROLLBACK_EXECUTION_TRANSCRIPT_VERSION: &str =
    "phase54-rollback-execution-transcript-local-v1";
/// Stable local incident-response execution transcript validation version.
pub const PACKAGING_INCIDENT_RESPONSE_EXECUTION_TRANSCRIPT_VERSION: &str =
    "phase55-incident-response-execution-transcript-local-v1";

/// State-store subsystem name for local packaging/deployment checkpoints.
pub const PACKAGING_STATE_SUBSYSTEM: &str = "packaging";

/// State-store key for the latest local deployment package record.
pub const PACKAGING_LAST_PACKAGE_RECORD_CHECKPOINT_KEY: &str = "packaging:last-package-record";

/// State-store key for the latest local rollback validation record.
pub const PACKAGING_LAST_ROLLBACK_VALIDATION_CHECKPOINT_KEY: &str =
    "packaging:last-rollback-validation";

/// Conservative packaging boundary settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackagingBoundaryConfig {
    /// Whether deterministic local packaging plan records may be produced.
    pub local_packaging_plan_enabled: bool,
    /// Whether container-image plan metadata may be recorded. This does not build images.
    pub container_image_plan_enabled: bool,
    /// Whether systemd unit plan metadata may be recorded. This does not install services.
    pub systemd_unit_plan_enabled: bool,
    /// Whether ARM target plan metadata may be recorded. This does not cross-compile.
    pub arm_build_profile_plan_enabled: bool,
    /// Whether public network exposure is allowed in generated deployment records. Phase 16 requires false.
    pub public_network_exposure_enabled: bool,
    /// Whether live trading may be enabled by a deployment plan. Phase 16 requires false.
    pub live_trading_deployment_enabled: bool,
    /// Whether deployment artifacts may contain embedded secret material. Phase 16 requires false.
    pub embedded_secret_material_allowed: bool,
    /// Whether this boundary may claim production deployment. Phase 16 requires false.
    pub production_deployment_claims_enabled: bool,
}

impl Default for PackagingBoundaryConfig {
    fn default() -> Self {
        Self {
            local_packaging_plan_enabled: true,
            container_image_plan_enabled: true,
            systemd_unit_plan_enabled: true,
            arm_build_profile_plan_enabled: true,
            public_network_exposure_enabled: false,
            live_trading_deployment_enabled: false,
            embedded_secret_material_allowed: false,
            production_deployment_claims_enabled: false,
        }
    }
}

impl PackagingBoundaryConfig {
    /// Validate fail-closed Phase 16 packaging settings.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();

        if !self.local_packaging_plan_enabled {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_LOCAL_PLAN_DISABLED",
                "local packaging plan records must remain enabled for Phase 16",
            ));
        }

        if self.public_network_exposure_enabled {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PUBLIC_NETWORK_EXPOSURE_DENIED",
                "Phase 16 deployment records must not permit public network exposure",
            ));
        }

        if self.live_trading_deployment_enabled {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_LIVE_TRADING_DENIED",
                "Phase 16 deployment records must not enable live trading",
            ));
        }

        if self.embedded_secret_material_allowed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_EMBEDDED_SECRET_DENIED",
                "deployment artifacts must not embed secret material",
            ));
        }

        if self.production_deployment_claims_enabled {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PRODUCTION_CLAIM_DENIED",
                "Phase 16 model records must not claim production deployment",
            ));
        }

        finish_validation(violations)
    }
}

/// Deployment target environment category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentEnvironmentKind {
    /// Local operator workstation.
    LocalWorkstation,
    /// Virtual private server target.
    Vps,
    /// ARM or edge host target.
    ArmEdge,
    /// Continuous integration packaging runner.
    CiRunner,
    /// Future production host. Phase 16 records may model this but not claim deployment.
    ProductionCandidate,
}

/// Packaging artifact category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageArtifactKind {
    /// Release-mode Rust binary artifact.
    RustBinary,
    /// Container image artifact.
    ContainerImage,
    /// systemd unit file artifact.
    SystemdUnit,
    /// ARM build profile artifact.
    ArmBuildProfile,
    /// Deployment documentation artifact.
    DeploymentDocument,
    /// CI release gate artifact.
    CiReleaseGate,
}

/// Network exposure declared by a target plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentNetworkExposure {
    /// No listening network service.
    None,
    /// Localhost-only binding.
    LocalhostOnly,
    /// Private network exposure. Phase 16 rejects this for default plans.
    PrivateNetwork,
    /// Public internet exposure. Phase 16 rejects this.
    PublicInternet,
}

impl DeploymentNetworkExposure {
    /// Return whether this exposure is acceptable in Phase 16 plan records.
    #[must_use]
    pub const fn is_phase16_allowed(self) -> bool {
        matches!(self, Self::None | Self::LocalhostOnly)
    }
}

/// How a deployment plan expects runtime configuration to be supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeConfigurationStrategy {
    /// File path loaded at runtime.
    RuntimeConfigFile,
    /// Environment references only, without embedding values into artifacts.
    EnvironmentReferences,
    /// Future external secret manager references.
    ExternalSecretReferences,
    /// Embedded material in artifacts. Phase 16 rejects this strategy.
    EmbeddedMaterial,
}

impl RuntimeConfigurationStrategy {
    /// Return whether this configuration strategy is acceptable in Phase 16 plan records.
    #[must_use]
    pub const fn is_phase16_allowed(self) -> bool {
        matches!(
            self,
            Self::RuntimeConfigFile | Self::EnvironmentReferences | Self::ExternalSecretReferences
        )
    }
}

/// Service hardening metadata for future deployment targets.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceHardeningProfile {
    /// Whether the service must run as a non-root account.
    pub run_as_non_root: bool,
    /// Whether the service should request no-new-privileges.
    pub no_new_privileges: bool,
    /// Whether the service should use a private temporary directory.
    pub private_tmp: bool,
    /// Whether the filesystem should be read-only by default.
    pub read_only_filesystem: bool,
    /// Whether home directories should be protected.
    pub protect_home: bool,
    /// Whether the plan allows write paths beyond explicit runtime state paths.
    pub unrestricted_write_paths_allowed: bool,
}

impl Default for ServiceHardeningProfile {
    fn default() -> Self {
        Self {
            run_as_non_root: true,
            no_new_privileges: true,
            private_tmp: true,
            read_only_filesystem: true,
            protect_home: true,
            unrestricted_write_paths_allowed: false,
        }
    }
}

impl ServiceHardeningProfile {
    fn validate(&self, context: &str, violations: &mut Vec<PackagingBoundaryViolation>) {
        if !self.run_as_non_root {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_NON_ROOT_REQUIRED",
                format!("{context} must require a non-root service identity"),
            ));
        }
        if !self.no_new_privileges {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_NO_NEW_PRIVILEGES_REQUIRED",
                format!("{context} must request no-new-privileges"),
            ));
        }
        if !self.read_only_filesystem {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_READ_ONLY_FS_REQUIRED",
                format!("{context} must default to read-only filesystem assumptions"),
            ));
        }
        if self.unrestricted_write_paths_allowed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_UNRESTRICTED_WRITE_PATHS_DENIED",
                format!("{context} must not allow unrestricted write paths"),
            ));
        }
    }
}

/// Target artifact plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageTargetPlan {
    /// Stable target identifier.
    pub target_id: String,
    /// Artifact category.
    pub artifact_kind: PackageArtifactKind,
    /// Intended deployment environment.
    pub environment: DeploymentEnvironmentKind,
    /// Rust target triple or platform label.
    pub platform: String,
    /// CPU architecture label.
    pub architecture: String,
    /// Runtime entrypoint.
    pub entrypoint: String,
    /// Configuration supply strategy.
    pub configuration_strategy: RuntimeConfigurationStrategy,
    /// Declared network exposure.
    pub network_exposure: DeploymentNetworkExposure,
    /// Service hardening metadata.
    pub hardening: ServiceHardeningProfile,
    /// Whether this target claims an actual build happened. Phase 16 requires false.
    pub build_performed: bool,
    /// Whether this target claims an actual deployment happened. Phase 16 requires false.
    pub deployment_performed: bool,
}

impl PackageTargetPlan {
    /// Construct a conservative local binary target plan.
    #[must_use]
    pub fn local_binary(target_id: impl Into<String>) -> Self {
        Self {
            target_id: target_id.into(),
            artifact_kind: PackageArtifactKind::RustBinary,
            environment: DeploymentEnvironmentKind::LocalWorkstation,
            platform: "x86_64-unknown-linux-gnu".to_owned(),
            architecture: "x86_64".to_owned(),
            entrypoint: "arb-agent --config /etc/arb-agent/config.toml".to_owned(),
            configuration_strategy: RuntimeConfigurationStrategy::RuntimeConfigFile,
            network_exposure: DeploymentNetworkExposure::None,
            hardening: ServiceHardeningProfile::default(),
            build_performed: false,
            deployment_performed: false,
        }
    }

    fn validate(
        &self,
        config: &PackagingBoundaryConfig,
        violations: &mut Vec<PackagingBoundaryViolation>,
    ) {
        let context = format!("target {}", self.target_id);
        if self.target_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_TARGET_ID_EMPTY",
                "target_id must be non-empty",
            ));
        }
        if self.platform.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLATFORM_EMPTY",
                format!("{context} platform must be non-empty"),
            ));
        }
        if self.architecture.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ARCHITECTURE_EMPTY",
                format!("{context} architecture must be non-empty"),
            ));
        }
        if self.entrypoint.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ENTRYPOINT_EMPTY",
                format!("{context} entrypoint must be non-empty"),
            ));
        }
        if contains_secret_like_text(&self.entrypoint) {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ENTRYPOINT_SECRET_LIKE",
                format!("{context} entrypoint contains secret-like text"),
            ));
        }
        if !self.network_exposure.is_phase16_allowed()
            || self.network_exposure == DeploymentNetworkExposure::PublicInternet
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_TARGET_NETWORK_EXPOSURE_DENIED",
                format!("{context} declares unsupported network exposure"),
            ));
        }
        if self.network_exposure != DeploymentNetworkExposure::None
            && config.public_network_exposure_enabled
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_CONFIG_NETWORK_CONFLICT",
                format!("{context} conflicts with public exposure denial"),
            ));
        }
        if !self.configuration_strategy.is_phase16_allowed() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_CONFIGURATION_STRATEGY_DENIED",
                format!("{context} uses an embedded configuration strategy"),
            ));
        }
        if self.build_performed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_BUILD_CLAIM_DENIED",
                format!("{context} must not claim a build was performed by model code"),
            ));
        }
        if self.deployment_performed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_DEPLOYMENT_CLAIM_DENIED",
                format!("{context} must not claim deployment was performed by model code"),
            ));
        }
        if matches!(self.artifact_kind, PackageArtifactKind::ContainerImage)
            && !config.container_image_plan_enabled
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_CONTAINER_PLAN_DISABLED",
                format!("{context} requested a container-image plan while disabled"),
            ));
        }
        if matches!(self.artifact_kind, PackageArtifactKind::SystemdUnit)
            && !config.systemd_unit_plan_enabled
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_SYSTEMD_PLAN_DISABLED",
                format!("{context} requested a systemd-unit plan while disabled"),
            ));
        }
        if matches!(self.artifact_kind, PackageArtifactKind::ArmBuildProfile)
            && !config.arm_build_profile_plan_enabled
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ARM_PLAN_DISABLED",
                format!("{context} requested an ARM build profile while disabled"),
            ));
        }
        self.hardening.validate(&context, violations);
    }
}

/// Release gate entry for future CI/deployment workflows.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseGate {
    /// Stable gate identifier.
    pub gate_id: String,
    /// Human-readable gate description.
    pub description: String,
    /// Whether this gate was executed. Phase 16 records should leave this false unless performed externally.
    pub executed: bool,
    /// Whether this gate passed. Meaningful only when executed externally.
    pub passed: bool,
}

impl ReleaseGate {
    /// Construct a pending release gate.
    #[must_use]
    pub fn pending(gate_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            gate_id: gate_id.into(),
            description: description.into(),
            executed: false,
            passed: false,
        }
    }

    fn validate(&self, violations: &mut Vec<PackagingBoundaryViolation>) {
        if self.gate_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RELEASE_GATE_ID_EMPTY",
                "release gate id must be non-empty",
            ));
        }
        if self.description.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RELEASE_GATE_DESCRIPTION_EMPTY",
                format!("release gate {} must have a description", self.gate_id),
            ));
        }
        if contains_secret_like_text(&self.description) {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RELEASE_GATE_SECRET_LIKE",
                format!("release gate {} contains secret-like text", self.gate_id),
            ));
        }
        if self.passed && !self.executed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RELEASE_GATE_PASS_WITHOUT_EXECUTION",
                format!(
                    "release gate {} cannot pass without execution",
                    self.gate_id
                ),
            ));
        }
    }
}

/// Rollback instruction metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RollbackStep {
    /// Deterministic step order.
    pub sequence: u32,
    /// Step description.
    pub action: String,
    /// Whether the step requires manual operator confirmation.
    pub requires_manual_confirmation: bool,
}

impl RollbackStep {
    /// Construct a rollback step.
    #[must_use]
    pub fn new(sequence: u32, action: impl Into<String>) -> Self {
        Self {
            sequence,
            action: action.into(),
            requires_manual_confirmation: true,
        }
    }

    fn validate(&self, violations: &mut Vec<PackagingBoundaryViolation>) {
        if self.sequence == 0 {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_SEQUENCE_ZERO",
                "rollback step sequence must be positive",
            ));
        }
        if self.action.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_ACTION_EMPTY",
                "rollback step action must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.action) {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_ACTION_SECRET_LIKE",
                "rollback step action contains secret-like text",
            ));
        }
    }
}

/// Deterministic deployment package plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentPackagePlan {
    /// Stable plan identifier.
    pub plan_id: String,
    /// Human-readable plan label.
    pub label: String,
    /// Target artifacts covered by this plan.
    pub targets: Vec<PackageTargetPlan>,
    /// Release gates that must be executed externally before production claims.
    pub release_gates: Vec<ReleaseGate>,
    /// Rollback steps that must remain operator-controlled.
    pub rollback_steps: Vec<RollbackStep>,
    /// Whether the plan attempts to enable live trading. Phase 16 requires false.
    pub live_trading_requested: bool,
    /// Whether the plan embeds secret material. Phase 16 requires false.
    pub embeds_secret_material: bool,
    /// Whether the plan claims production deployment. Phase 16 requires false.
    pub claims_production_deployment: bool,
}

impl DeploymentPackagePlan {
    /// Construct a conservative default Phase 16 plan.
    #[must_use]
    pub fn conservative(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            label: "phase-16-local-packaging-boundary".to_owned(),
            targets: vec![PackageTargetPlan::local_binary("local-release-binary")],
            release_gates: vec![
                ReleaseGate::pending("cargo-check", "Run cargo check for the full workspace"),
                ReleaseGate::pending("cargo-test", "Run cargo test for the full workspace"),
                ReleaseGate::pending(
                    "container-build",
                    "Build container artifact in an approved external environment",
                ),
                ReleaseGate::pending(
                    "systemd-lint",
                    "Validate service unit hardening in an approved Linux environment",
                ),
                ReleaseGate::pending(
                    "rollback-drill",
                    "Execute rollback drill with no live funds or secrets",
                ),
            ],
            rollback_steps: vec![
                RollbackStep::new(1, "Stop the candidate service if it was started externally"),
                RollbackStep::new(2, "Restore the previous validated artifact"),
                RollbackStep::new(3, "Restore the previous validated configuration reference"),
                RollbackStep::new(
                    4,
                    "Re-run local health checks and audit replay before resuming observe mode",
                ),
            ],
            live_trading_requested: false,
            embeds_secret_material: false,
            claims_production_deployment: false,
        }
    }

    fn validate(&self, config: &PackagingBoundaryConfig) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();

        if self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLAN_ID_EMPTY",
                "plan_id must be non-empty",
            ));
        }
        if self.label.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLAN_LABEL_EMPTY",
                "plan label must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.label) {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLAN_LABEL_SECRET_LIKE",
                "plan label contains secret-like text",
            ));
        }
        if self.targets.is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_TARGETS_EMPTY",
                "deployment package plan must include at least one target",
            ));
        }
        if self.release_gates.is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RELEASE_GATES_EMPTY",
                "deployment package plan must include release gates",
            ));
        }
        if self.rollback_steps.is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_STEPS_EMPTY",
                "deployment package plan must include rollback steps",
            ));
        }
        if self.live_trading_requested || config.live_trading_deployment_enabled {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLAN_LIVE_TRADING_DENIED",
                "deployment package plan must not request live trading",
            ));
        }
        if self.embeds_secret_material || config.embedded_secret_material_allowed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLAN_SECRET_EMBEDDING_DENIED",
                "deployment package plan must not embed secret material",
            ));
        }
        if self.claims_production_deployment || config.production_deployment_claims_enabled {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLAN_PRODUCTION_CLAIM_DENIED",
                "deployment package plan must not claim production deployment",
            ));
        }

        let mut target_ids = BTreeSet::new();
        for target in &self.targets {
            if !target_ids.insert(target.target_id.clone()) {
                violations.push(PackagingBoundaryViolation::new(
                    "PACKAGING_DUPLICATE_TARGET_ID",
                    format!("duplicate target id {}", target.target_id),
                ));
            }
            target.validate(config, &mut violations);
        }

        let mut gate_ids = BTreeSet::new();
        for gate in &self.release_gates {
            if !gate_ids.insert(gate.gate_id.clone()) {
                violations.push(PackagingBoundaryViolation::new(
                    "PACKAGING_DUPLICATE_RELEASE_GATE_ID",
                    format!("duplicate release gate id {}", gate.gate_id),
                ));
            }
            gate.validate(&mut violations);
        }

        let mut rollback_sequences = BTreeSet::new();
        for step in &self.rollback_steps {
            if !rollback_sequences.insert(step.sequence) {
                violations.push(PackagingBoundaryViolation::new(
                    "PACKAGING_DUPLICATE_ROLLBACK_SEQUENCE",
                    format!("duplicate rollback sequence {}", step.sequence),
                ));
            }
            step.validate(&mut violations);
        }

        finish_validation(violations)
    }
}

/// Request to generate a deterministic deployment package record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentPackageRequest {
    /// Boundary configuration.
    pub config: PackagingBoundaryConfig,
    /// Plan to validate and record.
    pub plan: DeploymentPackagePlan,
    /// Operator or automation label creating the record.
    pub requested_by: String,
}

impl DeploymentPackageRequest {
    /// Construct a conservative request.
    #[must_use]
    pub fn conservative(plan_id: impl Into<String>, requested_by: impl Into<String>) -> Self {
        Self {
            config: PackagingBoundaryConfig::default(),
            plan: DeploymentPackagePlan::conservative(plan_id),
            requested_by: requested_by.into(),
        }
    }
}

/// Deployment package record status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentPackageStatus {
    /// Record was accepted as a local model-only package plan.
    Planned,
    /// Record failed closed due to one or more violations.
    Rejected,
}

/// Local rollback validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RollbackValidationStatus {
    /// Rollback plan is coherent enough for local operator review.
    ReadyForLocalReview,
    /// Rollback plan failed local validation.
    Rejected,
}

/// Local validation status for sanitized rollback execution transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RollbackExecutionTranscriptStatus {
    /// Transcript contains all required rollback execution evidence references.
    ReadyForExternalReview,
    /// Transcript is missing rollback execution evidence or contains unsafe flags.
    Blocked,
}

/// Local validation status for sanitized incident-response execution transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IncidentResponseExecutionTranscriptStatus {
    /// Transcript contains all required incident-response execution evidence references.
    ReadyForExternalReview,
    /// Transcript is missing incident-response execution evidence or contains unsafe flags.
    Blocked,
}

/// Deterministic package/deployment record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentPackageRecord {
    /// Plan id recorded.
    pub plan_id: String,
    /// Record status.
    pub status: DeploymentPackageStatus,
    /// Number of package targets recorded.
    pub target_count: usize,
    /// Number of release gates recorded.
    pub release_gate_count: usize,
    /// Number of rollback steps recorded.
    pub rollback_step_count: usize,
    /// Whether a build was performed by this boundary. Always false in Phase 16.
    pub build_performed: bool,
    /// Whether deployment was performed by this boundary. Always false in Phase 16.
    pub deployment_performed: bool,
    /// Whether a public network was exposed. Always false in Phase 16.
    pub public_network_exposed: bool,
    /// Whether live trading was enabled. Always false in Phase 16.
    pub live_trading_enabled: bool,
    /// Whether secret material was embedded. Always false in Phase 16.
    pub secret_material_embedded: bool,
    /// Whether production deployment was claimed. Always false in Phase 16.
    pub production_deployment_claimed: bool,
    /// Violations when rejected.
    pub violations: Vec<PackagingBoundaryViolation>,
}

/// Local non-secret rollback validation record.
///
/// This record validates rollback metadata only. It never executes rollback
/// steps, mutates files, touches a service manager, calls external systems, or
/// claims production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RollbackValidationRecord {
    /// Source deployment package plan id.
    pub plan_id: String,
    /// Local rollback validation status.
    pub status: RollbackValidationStatus,
    /// Number of rollback steps reviewed.
    pub rollback_step_count: usize,
    /// Number of rollback steps requiring manual confirmation.
    pub manual_confirmation_required_count: usize,
    /// Whether rollback steps are numbered 1..N without gaps.
    pub steps_sequential: bool,
    /// Whether rollback was executed. Always false in this local boundary.
    pub rollback_executed: bool,
    /// Whether any service-manager action was performed. Always false here.
    pub service_manager_action_performed: bool,
    /// Whether files were mutated. Always false here.
    pub files_mutated: bool,
    /// Whether external calls were performed. Always false here.
    pub external_calls_performed: bool,
    /// Whether live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local validation approves production readiness. Always false.
    pub production_ready: bool,
    /// Validation violations when rejected.
    pub violations: Vec<PackagingBoundaryViolation>,
}

/// Sanitized rollback execution evidence transcript.
///
/// This records only operator-supplied reference presence and outcome flags. It
/// must not embed service logs, host paths, artifact contents, command output,
/// secrets, audit payloads, checkpoint values, or evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RollbackExecutionTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Source deployment package plan id.
    pub plan_id: String,
    /// Whether candidate deployment/run reference is present.
    pub candidate_reference_present: bool,
    /// Whether rollback target/artifact reference is present.
    pub rollback_reference_present: bool,
    /// Whether service quiesce/stop reference is present.
    pub service_quiesced_reference_present: bool,
    /// Whether previous artifact restoration evidence is present.
    pub previous_artifact_restored: bool,
    /// Whether previous configuration restoration evidence is present.
    pub previous_config_restored: bool,
    /// Whether post-rollback runtime smoke evidence is present.
    pub post_rollback_runtime_smoke_passed: bool,
    /// Whether audit replay after rollback evidence is present.
    pub audit_replay_after_rollback_validated: bool,
    /// Whether SQLite recovery after rollback evidence is present.
    pub sqlite_recovery_after_rollback_validated: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether this validator executed rollback. Must be false.
    pub rollback_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated files. Must be false.
    pub files_mutated_by_validator: bool,
    /// Whether this validator performed external calls. Must be false.
    pub external_calls_performed: bool,
    /// Whether this validator performed live execution. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for rollback execution evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RollbackExecutionTranscriptReport {
    /// Rollback execution transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Source deployment package plan id.
    pub plan_id: String,
    /// Whether candidate deployment/run reference is present.
    pub candidate_reference_present: bool,
    /// Whether rollback target/artifact reference is present.
    pub rollback_reference_present: bool,
    /// Whether service quiesce/stop reference is present.
    pub service_quiesced_reference_present: bool,
    /// Whether previous artifact restoration evidence is present.
    pub previous_artifact_restored: bool,
    /// Whether previous configuration restoration evidence is present.
    pub previous_config_restored: bool,
    /// Whether post-rollback runtime smoke evidence is present.
    pub post_rollback_runtime_smoke_passed: bool,
    /// Whether audit replay after rollback evidence is present.
    pub audit_replay_after_rollback_validated: bool,
    /// Whether SQLite recovery after rollback evidence is present.
    pub sqlite_recovery_after_rollback_validated: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Validation status.
    pub status: RollbackExecutionTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator executed rollback. Always false.
    pub rollback_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated files. Always false.
    pub files_mutated_by_validator: bool,
    /// Whether this validator performed external calls. Always false.
    pub external_calls_performed: bool,
    /// Whether this validator performed live execution. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Sanitized incident-response execution evidence transcript.
///
/// This records only operator-supplied reference presence and outcome flags. It
/// must not embed message bodies, host paths, artifact contents, command
/// output, secrets, audit payloads, checkpoint values, or evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncidentResponseExecutionTranscript {
    /// Stable transcript id.
    pub transcript_id: String,
    /// Source deployment package plan id or incident package id.
    pub plan_id: String,
    /// Whether incident scenario reference is present.
    pub incident_scenario_reference_present: bool,
    /// Whether incident severity reference is present.
    pub severity_reference_present: bool,
    /// Whether responder reference is present.
    pub responder_reference_present: bool,
    /// Whether reviewer reference is present.
    pub reviewer_reference_present: bool,
    /// Whether detection/triage evidence reference is present.
    pub detection_triage_reference_present: bool,
    /// Whether containment/recovery evidence reference is present.
    pub containment_recovery_reference_present: bool,
    /// Whether post-incident runtime smoke evidence is present.
    pub post_incident_runtime_smoke_passed: bool,
    /// Whether audit replay after recovery evidence is present.
    pub audit_replay_after_recovery_validated: bool,
    /// Whether SQLite recovery after recovery evidence is present.
    pub sqlite_recovery_after_recovery_validated: bool,
    /// Whether communications/escalation evidence reference is present.
    pub communications_reference_present: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Whether this validator executed incident-response actions. Must be false.
    pub incident_response_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Must be false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated files. Must be false.
    pub files_mutated_by_validator: bool,
    /// Whether this validator sent alerts or escalation messages. Must be false.
    pub alerts_sent_by_validator: bool,
    /// Whether this validator performed external calls. Must be false.
    pub external_calls_performed: bool,
    /// Whether this validator performed live execution. Must be false.
    pub live_execution_performed: bool,
    /// Whether this transcript attempts to claim production readiness. Must be false.
    pub production_ready_claimed: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local validation report for incident-response execution evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncidentResponseExecutionTranscriptReport {
    /// Incident-response execution transcript validation version.
    pub validation_version: String,
    /// Stable transcript id.
    pub transcript_id: String,
    /// Source deployment package plan id or incident package id.
    pub plan_id: String,
    /// Whether incident scenario reference is present.
    pub incident_scenario_reference_present: bool,
    /// Whether incident severity reference is present.
    pub severity_reference_present: bool,
    /// Whether responder reference is present.
    pub responder_reference_present: bool,
    /// Whether reviewer reference is present.
    pub reviewer_reference_present: bool,
    /// Whether detection/triage evidence reference is present.
    pub detection_triage_reference_present: bool,
    /// Whether containment/recovery evidence reference is present.
    pub containment_recovery_reference_present: bool,
    /// Whether post-incident runtime smoke evidence is present.
    pub post_incident_runtime_smoke_passed: bool,
    /// Whether audit replay after recovery evidence is present.
    pub audit_replay_after_recovery_validated: bool,
    /// Whether SQLite recovery after recovery evidence is present.
    pub sqlite_recovery_after_recovery_validated: bool,
    /// Whether communications/escalation evidence reference is present.
    pub communications_reference_present: bool,
    /// Whether operator approval/reference is present.
    pub operator_approved: bool,
    /// Whether reviewer approval/reference is present.
    pub reviewer_approved: bool,
    /// Count of non-secret evidence references.
    pub non_secret_reference_count: u64,
    /// Validation status.
    pub status: IncidentResponseExecutionTranscriptStatus,
    /// Non-secret blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether this validator executed incident-response actions. Always false.
    pub incident_response_executed_by_validator: bool,
    /// Whether this validator performed service-manager actions. Always false.
    pub service_manager_action_performed_by_validator: bool,
    /// Whether this validator mutated files. Always false.
    pub files_mutated_by_validator: bool,
    /// Whether this validator sent alerts or escalation messages. Always false.
    pub alerts_sent_by_validator: bool,
    /// Whether this validator performed external calls. Always false.
    pub external_calls_performed: bool,
    /// Whether this validator performed live execution. Always false.
    pub live_execution_performed: bool,
    /// Whether this report approves production readiness. Always false.
    pub production_ready: bool,
    /// Transcript validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

impl DeploymentPackageRecord {
    fn planned(plan: &DeploymentPackagePlan) -> Self {
        Self {
            plan_id: plan.plan_id.clone(),
            status: DeploymentPackageStatus::Planned,
            target_count: plan.targets.len(),
            release_gate_count: plan.release_gates.len(),
            rollback_step_count: plan.rollback_steps.len(),
            build_performed: false,
            deployment_performed: false,
            public_network_exposed: false,
            live_trading_enabled: false,
            secret_material_embedded: false,
            production_deployment_claimed: false,
            violations: Vec::new(),
        }
    }

    fn rejected(plan_id: impl Into<String>, violations: Vec<PackagingBoundaryViolation>) -> Self {
        Self {
            plan_id: plan_id.into(),
            status: DeploymentPackageStatus::Rejected,
            target_count: 0,
            release_gate_count: 0,
            rollback_step_count: 0,
            build_performed: false,
            deployment_performed: false,
            public_network_exposed: false,
            live_trading_enabled: false,
            secret_material_embedded: false,
            production_deployment_claimed: false,
            violations,
        }
    }

    /// Validate a local package record before audit/state persistence.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();

        if self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RECORD_PLAN_ID_EMPTY",
                "deployment package record plan id must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.plan_id) {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RECORD_PLAN_ID_SECRET_LIKE",
                "deployment package record plan id contains secret-like text",
            ));
        }
        if self.status == DeploymentPackageStatus::Planned
            && (self.target_count == 0
                || self.release_gate_count == 0
                || self.rollback_step_count == 0)
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RECORD_COUNTS_EMPTY",
                "planned deployment package record must include targets, gates, and rollback steps",
            ));
        }
        if self.status == DeploymentPackageStatus::Rejected && self.violations.is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_REJECTED_RECORD_WITHOUT_VIOLATIONS",
                "rejected deployment package records must include violations",
            ));
        }
        if self.status == DeploymentPackageStatus::Planned && !self.violations.is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_PLANNED_RECORD_WITH_VIOLATIONS",
                "planned deployment package records must not include violations",
            ));
        }
        if self.build_performed
            || self.deployment_performed
            || self.public_network_exposed
            || self.live_trading_enabled
            || self.secret_material_embedded
            || self.production_deployment_claimed
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_RECORD_SIDE_EFFECT_CLAIM_DENIED",
                "deployment package records must not claim builds, deployments, public exposure, live trading, embedded secrets, or production deployment",
            ));
        }

        finish_validation(violations)
    }
}

impl RollbackValidationRecord {
    /// Build a local rollback validation record for one deployment package plan.
    #[must_use]
    pub fn from_plan(plan: &DeploymentPackagePlan, config: &PackagingBoundaryConfig) -> Self {
        let mut violations = Vec::new();
        if let Err(PackagingBoundaryError::ValidationFailed {
            violations: plan_violations,
        }) = plan.validate(config)
        {
            violations.extend(plan_violations);
        }

        let manual_confirmation_required_count = plan
            .rollback_steps
            .iter()
            .filter(|step| step.requires_manual_confirmation)
            .count();
        let steps_sequential = rollback_steps_are_sequential(&plan.rollback_steps);
        if !steps_sequential {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_SEQUENCE_GAP",
                "rollback steps must be sequential from 1 without gaps",
            ));
        }
        if manual_confirmation_required_count != plan.rollback_steps.len() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_MANUAL_CONFIRMATION_REQUIRED",
                "every rollback step must require manual operator confirmation",
            ));
        }

        let status = if violations.is_empty() {
            RollbackValidationStatus::ReadyForLocalReview
        } else {
            RollbackValidationStatus::Rejected
        };

        Self {
            plan_id: plan.plan_id.clone(),
            status,
            rollback_step_count: plan.rollback_steps.len(),
            manual_confirmation_required_count,
            steps_sequential,
            rollback_executed: false,
            service_manager_action_performed: false,
            files_mutated: false,
            external_calls_performed: false,
            live_execution_performed: false,
            production_ready: false,
            violations,
        }
    }

    /// Validate a local rollback record before audit/state persistence.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();

        if self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_RECORD_PLAN_ID_EMPTY",
                "rollback validation record plan id must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.plan_id) {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_RECORD_PLAN_ID_SECRET_LIKE",
                "rollback validation record plan id contains secret-like text",
            ));
        }
        if self.status == RollbackValidationStatus::ReadyForLocalReview
            && (self.rollback_step_count == 0
                || !self.steps_sequential
                || self.manual_confirmation_required_count != self.rollback_step_count
                || !self.violations.is_empty())
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_READY_RECORD_INVALID",
                "ready rollback validation records require sequential manually-confirmed steps and no violations",
            ));
        }
        if self.status == RollbackValidationStatus::Rejected && self.violations.is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_REJECTED_RECORD_WITHOUT_VIOLATIONS",
                "rejected rollback validation records must include violations",
            ));
        }
        if self.rollback_executed
            || self.service_manager_action_performed
            || self.files_mutated
            || self.external_calls_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_SIDE_EFFECT_CLAIM_DENIED",
                "local rollback validation records must not claim rollback execution, service actions, file mutation, external calls, live execution, or production readiness",
            ));
        }

        finish_validation(violations)
    }
}

impl RollbackExecutionTranscript {
    /// Validate sanitized rollback execution transcript input.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();
        if self.transcript_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_TRANSCRIPT_ID_EMPTY",
                "rollback execution transcript id must be non-empty",
            ));
        }
        if self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_PLAN_ID_EMPTY",
                "rollback execution transcript plan id must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.transcript_id)
            || contains_secret_like_text(&self.plan_id)
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_SECRET_LIKE",
                "rollback execution transcript ids must not contain secret-like text",
            ));
        }
        if self.validated_at_unix_ms == 0 {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_TIMESTAMP_EMPTY",
                "rollback execution transcript timestamp must be non-zero",
            ));
        }
        if self.rollback_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.files_mutated_by_validator
            || self.external_calls_performed
            || self.live_execution_performed
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_VALIDATOR_SIDE_EFFECT_DENIED",
                "rollback execution transcript validator must not execute rollback, perform service actions, mutate files, call externally, or perform live execution",
            ));
        }
        if self.production_ready_claimed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_PRODUCTION_CLAIM_DENIED",
                "rollback execution transcript must not claim production readiness",
            ));
        }
        finish_validation(violations)
    }
}

impl RollbackExecutionTranscriptReport {
    /// Validate rollback execution transcript report invariants.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();
        if self.validation_version != PACKAGING_ROLLBACK_EXECUTION_TRANSCRIPT_VERSION {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_VERSION_MISMATCH",
                format!(
                    "validation_version must be {PACKAGING_ROLLBACK_EXECUTION_TRANSCRIPT_VERSION}"
                ),
            ));
        }
        if self.transcript_id.trim().is_empty() || self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_REPORT_ID_EMPTY",
                "rollback execution report requires transcript id and plan id",
            ));
        }
        if self.rollback_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.files_mutated_by_validator
            || self.external_calls_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_REPORT_SIDE_EFFECT_DENIED",
                "rollback execution report must not contain validator side effects or production readiness",
            ));
        }
        if self.status == RollbackExecutionTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_READY_WITH_BLOCKERS",
                "ready rollback execution report must not contain blockers",
            ));
        }
        if self.status == RollbackExecutionTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_ROLLBACK_EXECUTION_BLOCKED_WITHOUT_BLOCKERS",
                "blocked rollback execution report requires blocker codes",
            ));
        }
        finish_validation(violations)
    }
}

impl IncidentResponseExecutionTranscript {
    /// Validate sanitized incident-response execution transcript input.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();
        if self.transcript_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_TRANSCRIPT_ID_EMPTY",
                "incident-response execution transcript id must be non-empty",
            ));
        }
        if self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_PLAN_ID_EMPTY",
                "incident-response execution transcript plan id must be non-empty",
            ));
        }
        if contains_secret_like_text(&self.transcript_id)
            || contains_secret_like_text(&self.plan_id)
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_SECRET_LIKE",
                "incident-response execution transcript ids must not contain secret-like text",
            ));
        }
        if self.validated_at_unix_ms == 0 {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_TIMESTAMP_EMPTY",
                "incident-response execution transcript timestamp must be non-zero",
            ));
        }
        if self.incident_response_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.files_mutated_by_validator
            || self.alerts_sent_by_validator
            || self.external_calls_performed
            || self.live_execution_performed
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_VALIDATOR_SIDE_EFFECT_DENIED",
                "incident-response execution transcript validator must not execute incident actions, perform service actions, mutate files, send alerts, call externally, or perform live execution",
            ));
        }
        if self.production_ready_claimed {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_PRODUCTION_CLAIM_DENIED",
                "incident-response execution transcript must not claim production readiness",
            ));
        }
        finish_validation(violations)
    }
}

impl IncidentResponseExecutionTranscriptReport {
    /// Validate incident-response execution transcript report invariants.
    pub fn validate(&self) -> Result<(), PackagingBoundaryError> {
        let mut violations = Vec::new();
        if self.validation_version != PACKAGING_INCIDENT_RESPONSE_EXECUTION_TRANSCRIPT_VERSION {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_VERSION_MISMATCH",
                format!(
                    "validation_version must be {PACKAGING_INCIDENT_RESPONSE_EXECUTION_TRANSCRIPT_VERSION}"
                ),
            ));
        }
        if self.transcript_id.trim().is_empty() || self.plan_id.trim().is_empty() {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_REPORT_ID_EMPTY",
                "incident-response execution report requires transcript id and plan id",
            ));
        }
        if self.incident_response_executed_by_validator
            || self.service_manager_action_performed_by_validator
            || self.files_mutated_by_validator
            || self.alerts_sent_by_validator
            || self.external_calls_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_REPORT_SIDE_EFFECT_DENIED",
                "incident-response execution report must not contain validator side effects or production readiness",
            ));
        }
        if self.status == IncidentResponseExecutionTranscriptStatus::ReadyForExternalReview
            && !self.blocker_codes.is_empty()
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_READY_WITH_BLOCKERS",
                "ready incident-response execution report must not contain blockers",
            ));
        }
        if self.status == IncidentResponseExecutionTranscriptStatus::Blocked
            && self.blocker_codes.is_empty()
        {
            violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_INCIDENT_RESPONSE_EXECUTION_BLOCKED_WITHOUT_BLOCKERS",
                "blocked incident-response execution report requires blocker codes",
            ));
        }
        finish_validation(violations)
    }
}

/// Validate local rollback metadata without executing rollback steps.
#[must_use]
pub fn validate_local_deployment_rollback_plan(
    plan: &DeploymentPackagePlan,
    config: &PackagingBoundaryConfig,
) -> RollbackValidationRecord {
    RollbackValidationRecord::from_plan(plan, config)
}

/// Validate sanitized rollback execution evidence metadata.
///
/// This consumes operator-owned reference metadata only. It does not stop
/// services, restore artifacts, mutate files, call external systems, perform
/// live execution, or claim production readiness.
pub fn validate_rollback_execution_transcript(
    transcript: RollbackExecutionTranscript,
) -> Result<RollbackExecutionTranscriptReport, PackagingBoundaryError> {
    transcript.validate()?;
    let blocker_codes = rollback_execution_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        RollbackExecutionTranscriptStatus::ReadyForExternalReview
    } else {
        RollbackExecutionTranscriptStatus::Blocked
    };
    let report = RollbackExecutionTranscriptReport {
        validation_version: PACKAGING_ROLLBACK_EXECUTION_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        plan_id: transcript.plan_id,
        candidate_reference_present: transcript.candidate_reference_present,
        rollback_reference_present: transcript.rollback_reference_present,
        service_quiesced_reference_present: transcript.service_quiesced_reference_present,
        previous_artifact_restored: transcript.previous_artifact_restored,
        previous_config_restored: transcript.previous_config_restored,
        post_rollback_runtime_smoke_passed: transcript.post_rollback_runtime_smoke_passed,
        audit_replay_after_rollback_validated: transcript.audit_replay_after_rollback_validated,
        sqlite_recovery_after_rollback_validated: transcript
            .sqlite_recovery_after_rollback_validated,
        operator_approved: transcript.operator_approved,
        reviewer_approved: transcript.reviewer_approved,
        non_secret_reference_count: transcript.non_secret_reference_count,
        status,
        blocker_codes,
        rollback_executed_by_validator: false,
        service_manager_action_performed_by_validator: false,
        files_mutated_by_validator: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Validate sanitized incident-response execution evidence metadata.
///
/// This consumes operator-owned reference metadata only. It does not stop
/// services, modify deployments, send alerts, mutate files, call external
/// systems, perform live execution, or claim production readiness.
pub fn validate_incident_response_execution_transcript(
    transcript: IncidentResponseExecutionTranscript,
) -> Result<IncidentResponseExecutionTranscriptReport, PackagingBoundaryError> {
    transcript.validate()?;
    let blocker_codes = incident_response_execution_blockers(&transcript);
    let status = if blocker_codes.is_empty() {
        IncidentResponseExecutionTranscriptStatus::ReadyForExternalReview
    } else {
        IncidentResponseExecutionTranscriptStatus::Blocked
    };
    let report = IncidentResponseExecutionTranscriptReport {
        validation_version: PACKAGING_INCIDENT_RESPONSE_EXECUTION_TRANSCRIPT_VERSION.to_owned(),
        transcript_id: transcript.transcript_id,
        plan_id: transcript.plan_id,
        incident_scenario_reference_present: transcript.incident_scenario_reference_present,
        severity_reference_present: transcript.severity_reference_present,
        responder_reference_present: transcript.responder_reference_present,
        reviewer_reference_present: transcript.reviewer_reference_present,
        detection_triage_reference_present: transcript.detection_triage_reference_present,
        containment_recovery_reference_present: transcript.containment_recovery_reference_present,
        post_incident_runtime_smoke_passed: transcript.post_incident_runtime_smoke_passed,
        audit_replay_after_recovery_validated: transcript.audit_replay_after_recovery_validated,
        sqlite_recovery_after_recovery_validated: transcript
            .sqlite_recovery_after_recovery_validated,
        communications_reference_present: transcript.communications_reference_present,
        operator_approved: transcript.operator_approved,
        reviewer_approved: transcript.reviewer_approved,
        non_secret_reference_count: transcript.non_secret_reference_count,
        status,
        blocker_codes,
        incident_response_executed_by_validator: false,
        service_manager_action_performed_by_validator: false,
        files_mutated_by_validator: false,
        alerts_sent_by_validator: false,
        external_calls_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: transcript.validated_at_unix_ms,
    };
    report.validate()?;
    Ok(report)
}

/// Persist the latest local package/deployment record through the typed state boundary.
pub fn persist_deployment_package_record_checkpoint(
    store: &mut impl StateStore,
    record: &DeploymentPackageRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, PackagingBoundaryError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: PACKAGING_LAST_PACKAGE_RECORD_CHECKPOINT_KEY.to_owned(),
        subsystem: PACKAGING_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            PackagingBoundaryError::StateStoreFailed {
                reason: format!("failed to serialize deployment package checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(PackagingBoundaryError::from)?;
    Ok(checkpoint)
}

/// Persist the latest local rollback validation record through the typed state boundary.
pub fn persist_rollback_validation_checkpoint(
    store: &mut impl StateStore,
    record: &RollbackValidationRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, PackagingBoundaryError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: PACKAGING_LAST_ROLLBACK_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: PACKAGING_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            PackagingBoundaryError::StateStoreFailed {
                reason: format!("failed to serialize rollback validation checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(PackagingBoundaryError::from)?;
    Ok(checkpoint)
}

/// Append one local package/deployment record to the append-only audit journal.
pub fn append_deployment_package_record_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &DeploymentPackageRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, PackagingBoundaryError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("deployment-package-{}", record.plan_id),
        AuditEventKind::RuntimeLifecycle,
        PACKAGING_STATE_SUBSYSTEM,
        "deployment-package-planner",
        "local deployment package record",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "packaging_deployment_version",
            AuditValue::Text(PACKAGING_DEPLOYMENT_VERSION.to_owned()),
        )
        .with_metadata("plan_id", AuditValue::Text(record.plan_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", record.status)))
        .with_metadata(
            "target_count",
            AuditValue::Text(record.target_count.to_string()),
        )
        .with_metadata(
            "release_gate_count",
            AuditValue::Text(record.release_gate_count.to_string()),
        )
        .with_metadata(
            "rollback_step_count",
            AuditValue::Text(record.rollback_step_count.to_string()),
        )
        .with_metadata("build_performed", AuditValue::Bool(record.build_performed))
        .with_metadata(
            "deployment_performed",
            AuditValue::Bool(record.deployment_performed),
        )
        .with_metadata(
            "public_network_exposed",
            AuditValue::Bool(record.public_network_exposed),
        )
        .with_metadata(
            "live_trading_enabled",
            AuditValue::Bool(record.live_trading_enabled),
        )
        .with_metadata(
            "embedded_material_present",
            AuditValue::Bool(record.secret_material_embedded),
        )
        .with_metadata(
            "production_deployment_claimed",
            AuditValue::Bool(record.production_deployment_claimed),
        );
    journal
        .append_event(event)
        .map_err(PackagingBoundaryError::from)
}

/// Append one local rollback validation record to the append-only audit journal.
pub fn append_rollback_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &RollbackValidationRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, PackagingBoundaryError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("deployment-rollback-validation-{}", record.plan_id),
        AuditEventKind::RuntimeLifecycle,
        PACKAGING_STATE_SUBSYSTEM,
        "deployment-rollback-validator",
        "local deployment rollback validation record",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "packaging_deployment_version",
            AuditValue::Text(PACKAGING_DEPLOYMENT_VERSION.to_owned()),
        )
        .with_metadata("plan_id", AuditValue::Text(record.plan_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", record.status)))
        .with_metadata(
            "rollback_step_count",
            AuditValue::Text(record.rollback_step_count.to_string()),
        )
        .with_metadata(
            "manual_confirmation_required_count",
            AuditValue::Text(record.manual_confirmation_required_count.to_string()),
        )
        .with_metadata(
            "steps_sequential",
            AuditValue::Bool(record.steps_sequential),
        )
        .with_metadata(
            "rollback_executed",
            AuditValue::Bool(record.rollback_executed),
        )
        .with_metadata(
            "service_manager_action_performed",
            AuditValue::Bool(record.service_manager_action_performed),
        )
        .with_metadata("files_mutated", AuditValue::Bool(record.files_mutated))
        .with_metadata(
            "external_calls_performed",
            AuditValue::Bool(record.external_calls_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(record.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(record.production_ready),
        );
    journal
        .append_event(event)
        .map_err(PackagingBoundaryError::from)
}

/// Packaging/deployment planner boundary.
pub trait PackagingDeploymentPlanner {
    /// Validate a package/deployment plan and emit a local record.
    fn plan_package(
        &self,
        request: DeploymentPackageRequest,
    ) -> Result<DeploymentPackageRecord, PackagingBoundaryError>;
}

/// Deterministic model-only implementation of the Phase 16 packaging boundary.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicPackagingDeploymentPlanner;

impl PackagingDeploymentPlanner for DeterministicPackagingDeploymentPlanner {
    fn plan_package(
        &self,
        request: DeploymentPackageRequest,
    ) -> Result<DeploymentPackageRecord, PackagingBoundaryError> {
        request.config.validate()?;

        let mut request_violations = Vec::new();
        if request.requested_by.trim().is_empty() {
            request_violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_REQUESTED_BY_EMPTY",
                "requested_by must be non-empty",
            ));
        }
        if contains_secret_like_text(&request.requested_by) {
            request_violations.push(PackagingBoundaryViolation::new(
                "PACKAGING_REQUESTED_BY_SECRET_LIKE",
                "requested_by contains secret-like text",
            ));
        }
        if !request_violations.is_empty() {
            return Err(PackagingBoundaryError::ValidationFailed {
                violations: request_violations,
            });
        }

        match request.plan.validate(&request.config) {
            Ok(()) => Ok(DeploymentPackageRecord::planned(&request.plan)),
            Err(PackagingBoundaryError::ValidationFailed { violations }) => Ok(
                DeploymentPackageRecord::rejected(request.plan.plan_id, violations),
            ),
            Err(
                error @ (PackagingBoundaryError::AuditJournalFailed { .. }
                | PackagingBoundaryError::StateStoreFailed { .. }),
            ) => Err(error),
        }
    }
}

/// Packaging boundary violation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackagingBoundaryViolation {
    /// Stable violation code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl PackagingBoundaryViolation {
    /// Construct a packaging violation.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Packaging boundary error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackagingBoundaryError {
    /// Validation failed with one or more violations.
    ValidationFailed {
        /// Collected validation violations.
        violations: Vec<PackagingBoundaryViolation>,
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

impl From<crate::AuditError> for PackagingBoundaryError {
    fn from(error: crate::AuditError) -> Self {
        Self::AuditJournalFailed {
            reason: error.to_string(),
        }
    }
}

impl From<StateStoreError> for PackagingBoundaryError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for PackagingBoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(f, "packaging boundary validation failed")?;
                for violation in violations {
                    write!(f, "; {}: {}", violation.code, violation.message)?;
                }
                Ok(())
            }
            Self::AuditJournalFailed { reason } => {
                write!(f, "packaging audit journal failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(f, "packaging state store failed: {reason}")
            }
        }
    }
}

impl Error for PackagingBoundaryError {}

fn finish_validation(
    violations: Vec<PackagingBoundaryViolation>,
) -> Result<(), PackagingBoundaryError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(PackagingBoundaryError::ValidationFailed { violations })
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

fn rollback_steps_are_sequential(steps: &[RollbackStep]) -> bool {
    steps
        .iter()
        .enumerate()
        .all(|(index, step)| step.sequence == u32::try_from(index + 1).unwrap_or(u32::MAX))
}

fn rollback_execution_blockers(transcript: &RollbackExecutionTranscript) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.candidate_reference_present {
        blockers.push("missing-candidate-reference".to_owned());
    }
    if !transcript.rollback_reference_present {
        blockers.push("missing-rollback-reference".to_owned());
    }
    if !transcript.service_quiesced_reference_present {
        blockers.push("missing-service-quiesce-reference".to_owned());
    }
    if !transcript.previous_artifact_restored {
        blockers.push("missing-previous-artifact-restore-evidence".to_owned());
    }
    if !transcript.previous_config_restored {
        blockers.push("missing-previous-config-restore-evidence".to_owned());
    }
    if !transcript.post_rollback_runtime_smoke_passed {
        blockers.push("missing-post-rollback-runtime-smoke-evidence".to_owned());
    }
    if !transcript.audit_replay_after_rollback_validated {
        blockers.push("missing-audit-replay-after-rollback-evidence".to_owned());
    }
    if !transcript.sqlite_recovery_after_rollback_validated {
        blockers.push("missing-sqlite-recovery-after-rollback-evidence".to_owned());
    }
    if transcript.non_secret_reference_count < 5 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

fn incident_response_execution_blockers(
    transcript: &IncidentResponseExecutionTranscript,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !transcript.incident_scenario_reference_present {
        blockers.push("missing-incident-scenario-reference".to_owned());
    }
    if !transcript.severity_reference_present {
        blockers.push("missing-severity-reference".to_owned());
    }
    if !transcript.responder_reference_present {
        blockers.push("missing-responder-reference".to_owned());
    }
    if !transcript.reviewer_reference_present {
        blockers.push("missing-reviewer-reference".to_owned());
    }
    if !transcript.detection_triage_reference_present {
        blockers.push("missing-detection-triage-reference".to_owned());
    }
    if !transcript.containment_recovery_reference_present {
        blockers.push("missing-containment-recovery-reference".to_owned());
    }
    if !transcript.post_incident_runtime_smoke_passed {
        blockers.push("missing-post-incident-runtime-smoke-evidence".to_owned());
    }
    if !transcript.audit_replay_after_recovery_validated {
        blockers.push("missing-audit-replay-after-recovery-evidence".to_owned());
    }
    if !transcript.sqlite_recovery_after_recovery_validated {
        blockers.push("missing-sqlite-recovery-after-recovery-evidence".to_owned());
    }
    if !transcript.communications_reference_present {
        blockers.push("missing-communications-reference".to_owned());
    }
    if transcript.non_secret_reference_count < 6 {
        blockers.push("insufficient-non-secret-references".to_owned());
    }
    if !transcript.operator_approved {
        blockers.push("missing-operator-approval".to_owned());
    }
    if !transcript.reviewer_approved {
        blockers.push("missing-reviewer-approval".to_owned());
    }
    blockers
}

#[cfg(test)]
mod tests {
    use super::{
        append_deployment_package_record_audit, append_rollback_validation_audit,
        persist_deployment_package_record_checkpoint, persist_rollback_validation_checkpoint,
        validate_incident_response_execution_transcript, validate_local_deployment_rollback_plan,
        validate_rollback_execution_transcript, DeploymentNetworkExposure, DeploymentPackagePlan,
        DeploymentPackageRequest, DeploymentPackageStatus, DeterministicPackagingDeploymentPlanner,
        IncidentResponseExecutionTranscript, IncidentResponseExecutionTranscriptStatus,
        PackageTargetPlan, PackagingBoundaryConfig, PackagingBoundaryError,
        PackagingDeploymentPlanner, RollbackExecutionTranscript, RollbackExecutionTranscriptStatus,
        RollbackValidationStatus, RuntimeConfigurationStrategy,
        PACKAGING_LAST_PACKAGE_RECORD_CHECKPOINT_KEY,
        PACKAGING_LAST_ROLLBACK_VALIDATION_CHECKPOINT_KEY,
    };
    use crate::{AppendOnlyAuditJournal, SqliteWalStateStore, StateStore};
    use std::{env, fs, path::PathBuf, process};

    #[test]
    fn conservative_package_plan_records_no_side_effects() {
        let planner = DeterministicPackagingDeploymentPlanner;
        let record = planner
            .plan_package(DeploymentPackageRequest::conservative(
                "phase-16-plan",
                "local-operator",
            ))
            .expect("conservative plan should be valid");

        assert_eq!(record.status, DeploymentPackageStatus::Planned);
        assert_eq!(record.target_count, 1);
        assert!(!record.build_performed);
        assert!(!record.deployment_performed);
        assert!(!record.public_network_exposed);
        assert!(!record.live_trading_enabled);
        assert!(!record.secret_material_embedded);
        assert!(!record.production_deployment_claimed);
    }

    #[test]
    fn public_network_targets_are_rejected() {
        let planner = DeterministicPackagingDeploymentPlanner;
        let mut plan = DeploymentPackagePlan::conservative("phase-16-public-denial");
        plan.targets[0].network_exposure = DeploymentNetworkExposure::PublicInternet;

        let record = planner
            .plan_package(DeploymentPackageRequest {
                config: PackagingBoundaryConfig::default(),
                plan,
                requested_by: "local-operator".to_owned(),
            })
            .expect("plan-level denials should produce rejected record");

        assert_eq!(record.status, DeploymentPackageStatus::Rejected);
        assert!(record
            .violations
            .iter()
            .any(|violation| violation.code == "PACKAGING_TARGET_NETWORK_EXPOSURE_DENIED"));
    }

    #[test]
    fn embedded_material_strategy_is_rejected() {
        let planner = DeterministicPackagingDeploymentPlanner;
        let mut target = PackageTargetPlan::local_binary("embedded-material-denial");
        target.configuration_strategy = RuntimeConfigurationStrategy::EmbeddedMaterial;
        let mut plan = DeploymentPackagePlan::conservative("phase-16-secret-denial");
        plan.targets = vec![target];

        let record = planner
            .plan_package(DeploymentPackageRequest {
                config: PackagingBoundaryConfig::default(),
                plan,
                requested_by: "local-operator".to_owned(),
            })
            .expect("plan-level denials should produce rejected record");

        assert_eq!(record.status, DeploymentPackageStatus::Rejected);
        assert!(record
            .violations
            .iter()
            .any(|violation| violation.code == "PACKAGING_CONFIGURATION_STRATEGY_DENIED"));
    }

    #[test]
    fn config_rejects_live_deployment_flags() {
        let config = PackagingBoundaryConfig {
            live_trading_deployment_enabled: true,
            ..PackagingBoundaryConfig::default()
        };

        let error = config
            .validate()
            .expect_err("live deployment flags must fail closed");

        match error {
            PackagingBoundaryError::ValidationFailed { violations } => assert!(violations
                .iter()
                .any(|violation| violation.code == "PACKAGING_LIVE_TRADING_DENIED")),
            PackagingBoundaryError::AuditJournalFailed { .. }
            | PackagingBoundaryError::StateStoreFailed { .. } => {
                panic!("expected validation failure")
            }
        }
    }

    #[test]
    fn deployment_package_record_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("deployment-package");
        let state_path = temp_state_path("deployment-package");
        let planner = DeterministicPackagingDeploymentPlanner;
        let record = planner
            .plan_package(DeploymentPackageRequest::conservative(
                "phase-16-package-record",
                "local-release-operator",
            ))
            .expect("conservative plan should be valid");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_deployment_package_record_audit(&mut journal, &record, 1_700_000_000_801)
                .expect("deployment package audit writes");
        let checkpoint =
            persist_deployment_package_record_checkpoint(&mut store, &record, 1_700_000_000_802)
                .expect("deployment package checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, PACKAGING_LAST_PACKAGE_RECORD_CHECKPOINT_KEY);
        assert!(!record.build_performed);
        assert!(!record.deployment_performed);
        assert!(!record.public_network_exposed);
        assert!(!record.live_trading_enabled);
        assert!(!record.secret_material_embedded);
        assert!(!record.production_deployment_claimed);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(PACKAGING_LAST_PACKAGE_RECORD_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("deployment package checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_record: super::DeploymentPackageRecord =
            serde_json::from_str(&recovered.value).expect("checkpoint decodes");
        assert_eq!(recovered_record.status, DeploymentPackageStatus::Planned);
        assert!(!recovered_record.build_performed);
        assert!(!recovered_record.deployment_performed);
        assert!(!recovered_record.public_network_exposed);
        assert!(!recovered_record.live_trading_enabled);
        assert!(!recovered_record.secret_material_embedded);
        assert!(!recovered_record.production_deployment_claimed);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn rollback_validation_audit_and_state_reopen_locally_without_execution() {
        let audit_path = temp_audit_path("rollback-validation");
        let state_path = temp_state_path("rollback-validation");
        let plan = DeploymentPackagePlan::conservative("phase-16-rollback-validation");
        let record =
            validate_local_deployment_rollback_plan(&plan, &PackagingBoundaryConfig::default());
        assert_eq!(record.status, RollbackValidationStatus::ReadyForLocalReview);
        assert_eq!(record.rollback_step_count, plan.rollback_steps.len());
        assert_eq!(
            record.manual_confirmation_required_count,
            record.rollback_step_count
        );
        assert!(record.steps_sequential);
        assert!(!record.rollback_executed);
        assert!(!record.service_manager_action_performed);
        assert!(!record.files_mutated);
        assert!(!record.external_calls_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        let audit_record =
            append_rollback_validation_audit(&mut journal, &record, 1_700_000_000_901)
                .expect("rollback validation audit writes");
        let checkpoint =
            persist_rollback_validation_checkpoint(&mut store, &record, 1_700_000_000_902)
                .expect("rollback validation checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            PACKAGING_LAST_ROLLBACK_VALIDATION_CHECKPOINT_KEY
        );
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(PACKAGING_LAST_ROLLBACK_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("rollback validation checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        let recovered_record: super::RollbackValidationRecord =
            serde_json::from_str(&recovered.value).expect("checkpoint decodes");
        assert_eq!(
            recovered_record.status,
            RollbackValidationStatus::ReadyForLocalReview
        );
        assert!(!recovered_record.rollback_executed);
        assert!(!recovered_record.service_manager_action_performed);
        assert!(!recovered_record.files_mutated);
        assert!(!recovered_record.external_calls_performed);
        assert!(!recovered_record.live_execution_performed);
        assert!(!recovered_record.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn rollback_validation_rejects_non_manual_or_gapped_steps() {
        let mut plan = DeploymentPackagePlan::conservative("phase-16-rollback-rejected");
        plan.rollback_steps[1].requires_manual_confirmation = false;
        plan.rollback_steps[2].sequence = 7;

        let record =
            validate_local_deployment_rollback_plan(&plan, &PackagingBoundaryConfig::default());

        assert_eq!(record.status, RollbackValidationStatus::Rejected);
        assert!(!record.steps_sequential);
        assert!(record.manual_confirmation_required_count < record.rollback_step_count);
        assert!(record.violations.iter().any(|violation| {
            violation.code == "PACKAGING_ROLLBACK_MANUAL_CONFIRMATION_REQUIRED"
        }));
        assert!(record
            .violations
            .iter()
            .any(|violation| violation.code == "PACKAGING_ROLLBACK_SEQUENCE_GAP"));
        assert!(!record.rollback_executed);
        assert!(!record.service_manager_action_performed);
        assert!(!record.files_mutated);
        assert!(!record.external_calls_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);
    }

    #[test]
    fn rollback_execution_transcript_validates_operator_evidence_shape() {
        let report = validate_rollback_execution_transcript(rollback_execution_transcript(true))
            .expect("complete rollback execution transcript should validate");

        assert_eq!(
            report.status,
            RollbackExecutionTranscriptStatus::ReadyForExternalReview
        );
        assert!(report.candidate_reference_present);
        assert!(report.rollback_reference_present);
        assert!(report.service_quiesced_reference_present);
        assert!(report.previous_artifact_restored);
        assert!(report.previous_config_restored);
        assert!(report.post_rollback_runtime_smoke_passed);
        assert!(report.audit_replay_after_rollback_validated);
        assert!(report.sqlite_recovery_after_rollback_validated);
        assert!(report.operator_approved);
        assert!(report.reviewer_approved);
        assert_eq!(report.non_secret_reference_count, 7);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.rollback_executed_by_validator);
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.files_mutated_by_validator);
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn rollback_execution_transcript_blocks_missing_execution_evidence() {
        let report = validate_rollback_execution_transcript(rollback_execution_transcript(false))
            .expect("incomplete rollback execution transcript should produce blocked report");

        assert_eq!(report.status, RollbackExecutionTranscriptStatus::Blocked);
        assert!(!report.rollback_reference_present);
        assert!(!report.previous_artifact_restored);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-rollback-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-previous-artifact-restore-evidence"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn rollback_execution_transcript_rejects_validator_execution() {
        let mut transcript = rollback_execution_transcript(true);
        transcript.rollback_executed_by_validator = true;

        let error = validate_rollback_execution_transcript(transcript)
            .expect_err("validator rollback execution must fail closed");

        assert!(error.to_string().contains("must not execute rollback"));
    }

    #[test]
    fn incident_response_execution_transcript_validates_operator_evidence_shape() {
        let report = validate_incident_response_execution_transcript(
            incident_response_execution_transcript(true),
        )
        .expect("complete incident-response execution transcript should validate");

        assert_eq!(
            report.status,
            IncidentResponseExecutionTranscriptStatus::ReadyForExternalReview
        );
        assert!(report.incident_scenario_reference_present);
        assert!(report.severity_reference_present);
        assert!(report.responder_reference_present);
        assert!(report.reviewer_reference_present);
        assert!(report.detection_triage_reference_present);
        assert!(report.containment_recovery_reference_present);
        assert!(report.post_incident_runtime_smoke_passed);
        assert!(report.audit_replay_after_recovery_validated);
        assert!(report.sqlite_recovery_after_recovery_validated);
        assert!(report.communications_reference_present);
        assert!(report.operator_approved);
        assert!(report.reviewer_approved);
        assert_eq!(report.non_secret_reference_count, 8);
        assert!(report.blocker_codes.is_empty());
        assert!(!report.incident_response_executed_by_validator);
        assert!(!report.service_manager_action_performed_by_validator);
        assert!(!report.files_mutated_by_validator);
        assert!(!report.alerts_sent_by_validator);
        assert!(!report.external_calls_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn incident_response_execution_transcript_blocks_missing_execution_evidence() {
        let report = validate_incident_response_execution_transcript(
            incident_response_execution_transcript(false),
        )
        .expect("incomplete incident-response execution transcript should produce blocked report");

        assert_eq!(
            report.status,
            IncidentResponseExecutionTranscriptStatus::Blocked
        );
        assert!(!report.incident_scenario_reference_present);
        assert!(!report.containment_recovery_reference_present);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-incident-scenario-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "missing-containment-recovery-reference"));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| code == "insufficient-non-secret-references"));
        assert!(!report.production_ready);
    }

    #[test]
    fn incident_response_execution_transcript_rejects_validator_execution() {
        let mut transcript = incident_response_execution_transcript(true);
        transcript.alerts_sent_by_validator = true;

        let error = validate_incident_response_execution_transcript(transcript)
            .expect_err("validator alert sending must fail closed");

        assert!(error
            .to_string()
            .contains("must not execute incident actions"));
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

    fn rollback_execution_transcript(complete: bool) -> RollbackExecutionTranscript {
        RollbackExecutionTranscript {
            transcript_id: if complete {
                "rollback-execution-ready".to_owned()
            } else {
                "rollback-execution-blocked".to_owned()
            },
            plan_id: "phase-54-rollback-execution".to_owned(),
            candidate_reference_present: complete,
            rollback_reference_present: complete,
            service_quiesced_reference_present: complete,
            previous_artifact_restored: complete,
            previous_config_restored: complete,
            post_rollback_runtime_smoke_passed: complete,
            audit_replay_after_rollback_validated: complete,
            sqlite_recovery_after_rollback_validated: complete,
            operator_approved: complete,
            reviewer_approved: complete,
            non_secret_reference_count: if complete { 7 } else { 1 },
            rollback_executed_by_validator: false,
            service_manager_action_performed_by_validator: false,
            files_mutated_by_validator: false,
            external_calls_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 96_000,
        }
    }

    fn incident_response_execution_transcript(
        complete: bool,
    ) -> IncidentResponseExecutionTranscript {
        IncidentResponseExecutionTranscript {
            transcript_id: if complete {
                "incident-response-execution-ready".to_owned()
            } else {
                "incident-response-execution-blocked".to_owned()
            },
            plan_id: "phase-55-incident-response-execution".to_owned(),
            incident_scenario_reference_present: complete,
            severity_reference_present: complete,
            responder_reference_present: complete,
            reviewer_reference_present: complete,
            detection_triage_reference_present: complete,
            containment_recovery_reference_present: complete,
            post_incident_runtime_smoke_passed: complete,
            audit_replay_after_recovery_validated: complete,
            sqlite_recovery_after_recovery_validated: complete,
            communications_reference_present: complete,
            operator_approved: complete,
            reviewer_approved: complete,
            non_secret_reference_count: if complete { 8 } else { 1 },
            incident_response_executed_by_validator: false,
            service_manager_action_performed_by_validator: false,
            files_mutated_by_validator: false,
            alerts_sent_by_validator: false,
            external_calls_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 97_000,
        }
    }
}
