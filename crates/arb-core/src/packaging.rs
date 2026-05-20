#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable packaging/deployment boundary version for audit and handoff surfaces.
pub const PACKAGING_DEPLOYMENT_VERSION: &str = "phase-16-packaging-deployment-v1";

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

#[cfg(test)]
mod tests {
    use super::{
        DeploymentNetworkExposure, DeploymentPackagePlan, DeploymentPackageRequest,
        DeploymentPackageStatus, DeterministicPackagingDeploymentPlanner, PackageTargetPlan,
        PackagingBoundaryConfig, PackagingBoundaryError, PackagingDeploymentPlanner,
        RuntimeConfigurationStrategy,
    };

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
        }
    }
}
