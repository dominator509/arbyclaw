#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use crate::{
    persist_execution_adapter_run_checkpoint, persist_execution_plan_draft_checkpoint,
    validate_audit_journal_durability, AppendOnlyAuditJournal, AuditError, AuditEvent,
    AuditEventKind, AuditValue, DeterministicExecutionAdapterBoundary, ExecutionAdapter,
    ExecutionAdapterConfig, ExecutionAdapterError, ExecutionAdapterRequest,
    ExecutionAdapterRunRecord, ExecutionPlanDraft, ExecutionScope, PolicyEngine,
    SqliteWalStateStore, StateCheckpoint, StateStore, StateStoreError,
    EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, fs, path::Path};

/// Stable runtime lifecycle version for audit, state, and handoff surfaces.
pub const RUNTIME_LIFECYCLE_VERSION: &str = "phase-runtime-local-lifecycle-v1";

/// Stable local graceful-shutdown validation version.
pub const RUNTIME_GRACEFUL_SHUTDOWN_VERSION: &str = "phase26-runtime-graceful-shutdown-local-v1";

/// Stable local runtime backup/restore validation version.
pub const RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION: &str =
    "phase26-runtime-backup-restore-local-v1";

/// Stable local runtime restart recovery validation version.
pub const RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION: &str =
    "phase26-runtime-restart-recovery-local-v1";

/// Stable local runtime deployment-smoke validation version.
pub const RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION: &str =
    "phase26-runtime-deployment-smoke-local-v1";

/// State checkpoint key for the last local graceful-shutdown record.
pub const RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY: &str = "runtime:last-graceful-shutdown";

/// Runtime lifecycle status for local-only adapter evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeLifecycleStatus {
    /// Plan state was persisted before adapter evaluation.
    PlanCheckpointed,
    /// Deterministic adapter run was evaluated and persisted.
    AdapterRunCheckpointed,
}

/// One local runtime lifecycle request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLifecycleRequest {
    /// Stable lifecycle id for audit and replay.
    pub id: String,
    /// Stable adapter request id.
    pub adapter_request_id: String,
    /// Draft plan to checkpoint and evaluate.
    pub plan: ExecutionPlanDraft,
    /// Adapter-boundary configuration.
    pub adapter_config: ExecutionAdapterConfig,
    /// Runtime clock in Unix milliseconds.
    pub now_unix_ms: u64,
}

impl RuntimeLifecycleRequest {
    /// Validate local runtime lifecycle boundaries before side effects.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle id is required".to_owned(),
            });
        }
        if self.adapter_request_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "adapter request id is required".to_owned(),
            });
        }
        if self.now_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "now_unix_ms must be non-zero".to_owned(),
            });
        }
        if self.plan.scope == ExecutionScope::Live {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle rejects live-scope plans".to_owned(),
            });
        }
        self.plan
            .validate()
            .map_err(RuntimeLifecycleError::Planner)?;
        self.adapter_config
            .validate()
            .map_err(RuntimeLifecycleError::Adapter)?;
        Ok(())
    }
}

/// Completed local runtime lifecycle record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLifecycleRecord {
    /// Stable lifecycle id.
    pub id: String,
    /// Runtime lifecycle model version.
    pub runtime_lifecycle_version: String,
    /// Source plan id.
    pub plan_id: String,
    /// Source adapter request id.
    pub adapter_request_id: String,
    /// Plan scope.
    pub scope: ExecutionScope,
    /// Completed lifecycle status.
    pub status: RuntimeLifecycleStatus,
    /// Plan checkpoint key.
    pub plan_checkpoint_key: String,
    /// Adapter run checkpoint key.
    pub adapter_run_checkpoint_key: String,
    /// Audit sequence for lifecycle start.
    pub start_audit_sequence: u64,
    /// Audit sequence for plan checkpoint.
    pub plan_checkpoint_audit_sequence: u64,
    /// Audit sequence for adapter completion.
    pub adapter_complete_audit_sequence: u64,
    /// Deterministic adapter run.
    pub adapter_run: ExecutionAdapterRunRecord,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Record creation time.
    pub created_at_unix_ms: u64,
    /// Non-secret warnings.
    pub warnings: Vec<String>,
}

/// Local graceful-shutdown checkpoint request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeGracefulShutdownRequest {
    /// Stable shutdown id for audit and replay.
    pub id: String,
    /// Non-secret shutdown reason or operator note.
    pub reason: String,
    /// Runtime clock in Unix milliseconds.
    pub now_unix_ms: u64,
}

impl RuntimeGracefulShutdownRequest {
    /// Validate local graceful-shutdown checkpoint input.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown id is required".to_owned(),
            });
        }
        if self.reason.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown reason is required".to_owned(),
            });
        }
        if self.now_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "now_unix_ms must be non-zero".to_owned(),
            });
        }
        Ok(())
    }
}

/// Completed local graceful-shutdown checkpoint record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeGracefulShutdownRecord {
    /// Stable shutdown id.
    pub id: String,
    /// Runtime graceful-shutdown model version.
    pub runtime_graceful_shutdown_version: String,
    /// Shutdown checkpoint key.
    pub shutdown_checkpoint_key: String,
    /// Non-secret checkpoint value persisted to the state store.
    pub shutdown_checkpoint_value: String,
    /// Audit sequence for shutdown start.
    pub shutdown_start_audit_sequence: u64,
    /// Audit sequence for shutdown checkpoint persistence.
    pub shutdown_checkpoint_audit_sequence: u64,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Whether this local record approves production readiness.
    pub production_ready: bool,
    /// Record creation time.
    pub created_at_unix_ms: u64,
    /// Non-secret warnings.
    pub warnings: Vec<String>,
}

/// Non-secret result of a local runtime audit/state backup-restore validation pass.
///
/// The report intentionally stores outcomes only. It does not include local
/// filesystem paths, audit payloads, checkpoint values, database contents,
/// secrets, dependency graphs, or embedded evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeBackupRestoreValidationReport {
    /// Runtime backup/restore validation model version.
    pub validation_version: String,
    /// Number of audit records replayed from the copied journal.
    pub audit_records_replayed: u64,
    /// True when the copied audit journal reopened with the same next sequence.
    pub audit_restore_check_passed: bool,
    /// True when the copied SQLite database reopened and passed integrity check.
    pub sqlite_restore_check_passed: bool,
    /// True when the restored planner checkpoint was present.
    pub plan_checkpoint_restored: bool,
    /// True when the restored adapter-run checkpoint was present.
    pub adapter_checkpoint_restored: bool,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Whether this local validation approves production readiness.
    pub production_ready: bool,
}

/// Local restart recovery disposition for operator review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeRestartRecoveryDisposition {
    /// Required runtime checkpoints and graceful-shutdown checkpoint are present.
    ReadyForLocalReview,
    /// Required runtime checkpoints are present, but operator review is needed.
    NeedsOperatorReview,
}

/// Non-secret result of a local runtime restart recovery validation pass.
///
/// This report records only restart/replay outcomes and deliberately omits
/// filesystem paths, audit payloads, checkpoint values, secrets, database
/// contents, deployment metadata, and embedded evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRestartRecoveryValidationReport {
    /// Runtime restart recovery validation model version.
    pub validation_version: String,
    /// Number of audit records replayed from the local journal.
    pub audit_records_replayed: u64,
    /// True when audit replay reopened a non-empty journal.
    pub audit_replay_check_passed: bool,
    /// True when the SQLite store reopened and passed integrity check.
    pub sqlite_reopen_check_passed: bool,
    /// True when the planner checkpoint was present after reopen.
    pub plan_checkpoint_recovered: bool,
    /// True when the adapter-run checkpoint was present after reopen.
    pub adapter_checkpoint_recovered: bool,
    /// True when a graceful-shutdown checkpoint was present after reopen.
    pub graceful_shutdown_checkpoint_recovered: bool,
    /// Local recovery disposition for operator review.
    pub recovery_disposition: RuntimeRestartRecoveryDisposition,
    /// True when local lifecycle state is coherent enough for operator review.
    pub local_review_ready: bool,
    /// Whether any external adapter was submitted to. Always false in this boundary.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false in this boundary.
    pub live_execution_performed: bool,
    /// Whether this local validation approves production readiness.
    pub production_ready: bool,
}

/// Non-secret result of a local deployment-like runtime smoke validation pass.
///
/// This report intentionally records outcomes only. It does not include paths,
/// audit payloads, checkpoint values, database contents, secrets, deployment
/// metadata, service-manager data, or embedded evidence artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSmokeValidationReport {
    /// Runtime deployment-smoke validation model version.
    pub validation_version: String,
    /// Whether one local runtime lifecycle completed.
    pub lifecycle_completed: bool,
    /// Whether graceful-shutdown audit/state checkpointing completed.
    pub graceful_shutdown_checkpointed: bool,
    /// Whether local backup/restore validation completed.
    pub backup_restore_validated: bool,
    /// Whether restart recovery validation completed.
    pub restart_recovery_validated: bool,
    /// Whether local audit durability probes completed.
    pub audit_durability_validated: bool,
    /// Number of audit records replayed by restart recovery.
    pub restart_audit_records_replayed: u64,
    /// Number of audit records replayed from backup/restore validation.
    pub backup_audit_records_replayed: u64,
    /// Local recovery disposition for operator review.
    pub recovery_disposition: RuntimeRestartRecoveryDisposition,
    /// Whether any service manager action was performed. Always false here.
    pub service_manager_action_performed: bool,
    /// Whether any external adapter was submitted to. Always false here.
    pub external_submission_performed: bool,
    /// Whether any live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local validation approves production readiness.
    pub production_ready: bool,
    /// Remaining blockers in non-secret wording.
    pub unresolved_blockers: Vec<String>,
}

/// Inputs for one local deployment-like runtime smoke validation pass.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDeploymentSmokeValidationRequest {
    /// Runtime lifecycle request to execute locally.
    pub lifecycle_request: RuntimeLifecycleRequest,
    /// Graceful-shutdown checkpoint request to execute locally.
    pub shutdown_request: RuntimeGracefulShutdownRequest,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

impl RuntimeDeploymentSmokeValidationRequest {
    /// Validate local deployment-smoke request invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validated_at_unix_ms == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke timestamp must be non-zero".to_owned(),
            });
        }
        self.lifecycle_request.validate()?;
        self.shutdown_request.validate()?;
        Ok(())
    }
}

impl RuntimeRestartRecoveryValidationReport {
    /// Validate local restart recovery report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION}"
                ),
            });
        }
        if self.audit_records_replayed == 0 || !self.audit_replay_check_passed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery requires replayed audit records".to_owned(),
            });
        }
        if !self.sqlite_reopen_check_passed
            || !self.plan_checkpoint_recovered
            || !self.adapter_checkpoint_recovered
            || !self.local_review_ready
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery requires coherent local audit/state checkpoints"
                    .to_owned(),
            });
        }
        match self.recovery_disposition {
            RuntimeRestartRecoveryDisposition::ReadyForLocalReview => {
                if !self.graceful_shutdown_checkpoint_recovered {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "ready recovery disposition requires graceful shutdown checkpoint"
                            .to_owned(),
                    });
                }
            }
            RuntimeRestartRecoveryDisposition::NeedsOperatorReview => {
                if self.graceful_shutdown_checkpoint_recovered {
                    return Err(RuntimeLifecycleError::ValidationFailed {
                        reason: "operator-review recovery disposition requires missing graceful shutdown checkpoint".to_owned(),
                    });
                }
            }
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery validation must not perform external submission or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime restart recovery validation must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeDeploymentSmokeValidationReport {
    /// Validate local deployment-smoke report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION}"
                ),
            });
        }
        if !self.lifecycle_completed
            || !self.graceful_shutdown_checkpointed
            || !self.backup_restore_validated
            || !self.restart_recovery_validated
            || !self.audit_durability_validated
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke checks must pass".to_owned(),
            });
        }
        if self.restart_audit_records_replayed == 0 || self.backup_audit_records_replayed == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires replayed audit records".to_owned(),
            });
        }
        if self.recovery_disposition != RuntimeRestartRecoveryDisposition::ReadyForLocalReview {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke requires clean local recovery disposition"
                    .to_owned(),
            });
        }
        if self.service_manager_action_performed
            || self.external_submission_performed
            || self.live_execution_performed
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke must not perform service-manager action, external submission, or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime deployment smoke must not approve production readiness".to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeBackupRestoreValidationReport {
    /// Validate local backup/restore report invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.validation_version != RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "validation_version must be {RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION}"
                ),
            });
        }
        if self.audit_records_replayed == 0 {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore validation requires audit records".to_owned(),
            });
        }
        if !self.audit_restore_check_passed
            || !self.sqlite_restore_check_passed
            || !self.plan_checkpoint_restored
            || !self.adapter_checkpoint_restored
        {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore checks must pass".to_owned(),
            });
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore validation must not perform external submission or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime backup/restore validation must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeGracefulShutdownRecord {
    /// Validate local graceful-shutdown invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown id is required".to_owned(),
            });
        }
        if self.runtime_graceful_shutdown_version != RUNTIME_GRACEFUL_SHUTDOWN_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!(
                    "runtime_graceful_shutdown_version must be {RUNTIME_GRACEFUL_SHUTDOWN_VERSION}"
                ),
            });
        }
        if self.shutdown_checkpoint_key != RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected graceful shutdown checkpoint key".to_owned(),
            });
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown checkpoint must not perform external submission or live execution".to_owned(),
            });
        }
        if self.production_ready {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "graceful shutdown checkpoint must not approve production readiness"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

impl RuntimeLifecycleRecord {
    /// Validate local runtime lifecycle invariants.
    pub fn validate(&self) -> Result<(), RuntimeLifecycleError> {
        if self.id.trim().is_empty() {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle id is required".to_owned(),
            });
        }
        if self.runtime_lifecycle_version != RUNTIME_LIFECYCLE_VERSION {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: format!("runtime_lifecycle_version must be {RUNTIME_LIFECYCLE_VERSION}"),
            });
        }
        if self.scope == ExecutionScope::Live {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle records must not use live scope".to_owned(),
            });
        }
        if self.external_submission_performed || self.live_execution_performed {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "runtime lifecycle must not perform external submission or live execution"
                    .to_owned(),
            });
        }
        if self.plan_checkpoint_key != EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected plan checkpoint key".to_owned(),
            });
        }
        if self.adapter_run_checkpoint_key != EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY {
            return Err(RuntimeLifecycleError::ValidationFailed {
                reason: "unexpected adapter run checkpoint key".to_owned(),
            });
        }
        self.adapter_run
            .validate()
            .map_err(RuntimeLifecycleError::Adapter)?;
        Ok(())
    }
}

/// Execute one local fail-closed runtime lifecycle.
///
/// The lifecycle appends audit events and writes the plan checkpoint before the
/// adapter boundary is evaluated. Any audit/state failure returns an error and
/// prevents later lifecycle steps. This function does not submit orders, call
/// exchanges/RPCs, sign payloads, broadcast transactions, withdraw funds, or
/// bridge assets.
pub fn run_local_runtime_lifecycle(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    policy: &PolicyEngine,
    request: RuntimeLifecycleRequest,
) -> Result<RuntimeLifecycleRecord, RuntimeLifecycleError> {
    request.validate()?;

    let start_record = journal.append_event(lifecycle_event(
        &request,
        AuditEventKind::RuntimeLifecycle,
        "runtime lifecycle started",
    ))?;

    let plan_checkpoint = persist_execution_plan_draft_checkpoint(store, &request.plan)?;
    let plan_checkpoint_record = journal.append_event(checkpoint_event(
        &request,
        AuditEventKind::ExecutionPlanning,
        "execution plan checkpoint persisted before adapter evaluation",
        &plan_checkpoint,
    ))?;

    let adapter = DeterministicExecutionAdapterBoundary::new();
    let adapter_request = ExecutionAdapterRequest {
        id: request.adapter_request_id.clone(),
        plan: request.plan.clone(),
        config: request.adapter_config.clone(),
        now_unix_ms: request.now_unix_ms,
    };
    let adapter_run = adapter.evaluate_plan(&adapter_request, policy)?;
    let adapter_checkpoint = persist_execution_adapter_run_checkpoint(store, &adapter_run)?;
    let adapter_record = journal.append_event(checkpoint_event(
        &request,
        AuditEventKind::ExecutionResult,
        "execution adapter run checkpoint persisted",
        &adapter_checkpoint,
    ))?;

    let record = RuntimeLifecycleRecord {
        id: request.id,
        runtime_lifecycle_version: RUNTIME_LIFECYCLE_VERSION.to_owned(),
        plan_id: request.plan.id,
        adapter_request_id: adapter_request.id,
        scope: adapter_run.scope,
        status: RuntimeLifecycleStatus::AdapterRunCheckpointed,
        plan_checkpoint_key: plan_checkpoint.key,
        adapter_run_checkpoint_key: adapter_checkpoint.key,
        start_audit_sequence: start_record.sequence,
        plan_checkpoint_audit_sequence: plan_checkpoint_record.sequence,
        adapter_complete_audit_sequence: adapter_record.sequence,
        adapter_run,
        external_submission_performed: false,
        live_execution_performed: false,
        created_at_unix_ms: request.now_unix_ms,
        warnings: vec![
            "local runtime lifecycle only; no external submission, signing, broadcast, withdrawal, bridge, or live execution occurred".to_owned(),
        ],
    };
    record.validate()?;
    Ok(record)
}

/// Persist one local graceful-shutdown audit/state checkpoint.
///
/// This boundary models the local audit/state writes expected before a clean
/// runtime stop. It does not stop a process, interact with a service manager,
/// submit orders, call networks, sign payloads, broadcast transactions,
/// withdraw funds, or bridge assets.
pub fn run_local_graceful_shutdown_checkpoint(
    journal: &mut AppendOnlyAuditJournal,
    store: &mut impl StateStore,
    request: RuntimeGracefulShutdownRequest,
) -> Result<RuntimeGracefulShutdownRecord, RuntimeLifecycleError> {
    request.validate()?;

    let start_record = journal.append_event(graceful_shutdown_event(
        &request,
        "runtime graceful shutdown started",
    ))?;
    let checkpoint = StateCheckpoint {
        key: RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY.to_owned(),
        subsystem: "runtime-lifecycle".to_owned(),
        value: format!("graceful-shutdown:{}:{}", request.id, request.now_unix_ms),
        updated_at_unix_ms: request.now_unix_ms,
    };
    store.put_checkpoint(checkpoint.clone())?;
    let checkpoint_record = journal.append_event(
        graceful_shutdown_event(&request, "runtime graceful shutdown checkpoint persisted")
            .with_metadata("checkpoint_key", AuditValue::Text(checkpoint.key.clone()))
            .with_metadata(
                "checkpoint_subsystem",
                AuditValue::Text(checkpoint.subsystem.clone()),
            )
            .with_metadata(
                "checkpoint_updated_at_unix_ms",
                AuditValue::Unsigned(checkpoint.updated_at_unix_ms),
            ),
    )?;

    let record = RuntimeGracefulShutdownRecord {
        id: request.id,
        runtime_graceful_shutdown_version: RUNTIME_GRACEFUL_SHUTDOWN_VERSION.to_owned(),
        shutdown_checkpoint_key: checkpoint.key,
        shutdown_checkpoint_value: checkpoint.value,
        shutdown_start_audit_sequence: start_record.sequence,
        shutdown_checkpoint_audit_sequence: checkpoint_record.sequence,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        created_at_unix_ms: request.now_unix_ms,
        warnings: vec![
            "local graceful-shutdown checkpoint only; no service manager action, external submission, signing, broadcast, withdrawal, bridge, or live execution occurred".to_owned(),
        ],
    };
    record.validate()?;
    Ok(record)
}

/// Validate local backup/restore of runtime audit and SQLite state artifacts.
///
/// This boundary copies an existing non-secret local audit journal and
/// checkpointed SQLite database to caller-supplied backup paths, then reopens
/// the copies and verifies the runtime planner and adapter checkpoints can be
/// read. It does not start services, inspect deployment state, submit orders,
/// call networks, sign payloads, broadcast transactions, withdraw funds, or
/// bridge assets.
pub fn validate_local_runtime_backup_restore(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    backup_audit_path: impl AsRef<Path>,
    backup_state_path: impl AsRef<Path>,
) -> Result<RuntimeBackupRestoreValidationReport, RuntimeLifecycleError> {
    let audit_path = audit_path.as_ref();
    let state_path = state_path.as_ref();
    let backup_audit_path = backup_audit_path.as_ref();
    let backup_state_path = backup_state_path.as_ref();

    validate_runtime_backup_target(audit_path, backup_audit_path, "audit")?;
    validate_runtime_backup_target(state_path, backup_state_path, "state")?;

    let primary_journal = AppendOnlyAuditJournal::open(audit_path)?;
    let primary_next_sequence = primary_journal.next_sequence();
    let primary_audit_records = primary_next_sequence.saturating_sub(1);
    if primary_audit_records == 0 {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime backup/restore validation requires a non-empty audit journal"
                .to_owned(),
        });
    }

    let primary_state = SqliteWalStateStore::open(state_path)?;
    primary_state.integrity_check()?;
    primary_state.wal_checkpoint_truncate()?;
    let primary_plan_checkpoint = primary_state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let primary_adapter_checkpoint = primary_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();
    if !primary_plan_checkpoint || !primary_adapter_checkpoint {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime backup/restore validation requires planner and adapter checkpoints"
                .to_owned(),
        });
    }
    drop(primary_state);

    copy_runtime_backup_file(audit_path, backup_audit_path, "audit")?;
    copy_runtime_backup_file(state_path, backup_state_path, "state")?;

    let restored_journal = AppendOnlyAuditJournal::open(backup_audit_path)?;
    if restored_journal.next_sequence() != primary_next_sequence {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "restored audit journal sequence did not match primary".to_owned(),
        });
    }

    let restored_state = SqliteWalStateStore::open(backup_state_path)?;
    restored_state.integrity_check()?;
    let plan_checkpoint_restored = restored_state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let adapter_checkpoint_restored = restored_state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();

    let report = RuntimeBackupRestoreValidationReport {
        validation_version: RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION.to_owned(),
        audit_records_replayed: restored_journal.next_sequence().saturating_sub(1),
        audit_restore_check_passed: true,
        sqlite_restore_check_passed: true,
        plan_checkpoint_restored,
        adapter_checkpoint_restored,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Validate local restart recovery from existing audit and SQLite state files.
///
/// This boundary reopens the local audit journal and SQLite state store, checks
/// audit replay, SQLite integrity, and required runtime lifecycle checkpoints,
/// then returns a non-secret recovery summary for operator review. It does not
/// start services, resume work, inspect deployment state, submit orders, call
/// networks, sign payloads, broadcast transactions, withdraw funds, or bridge
/// assets.
pub fn validate_local_runtime_restart_recovery(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
) -> Result<RuntimeRestartRecoveryValidationReport, RuntimeLifecycleError> {
    let audit_path = audit_path.as_ref();
    let state_path = state_path.as_ref();
    if audit_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery audit path is required".to_owned(),
        });
    }
    if state_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery state path is required".to_owned(),
        });
    }

    let journal = AppendOnlyAuditJournal::open(audit_path)?;
    let audit_records_replayed = journal.next_sequence().saturating_sub(1);
    if audit_records_replayed == 0 {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime restart recovery requires a non-empty audit journal".to_owned(),
        });
    }

    let state = SqliteWalStateStore::open(state_path)?;
    state.integrity_check()?;
    let plan_checkpoint_recovered = state
        .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)?
        .is_some();
    let adapter_checkpoint_recovered = state
        .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)?
        .is_some();
    let graceful_shutdown_checkpoint_recovered = state
        .get_checkpoint(RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY)?
        .is_some();
    let recovery_disposition = if graceful_shutdown_checkpoint_recovered {
        RuntimeRestartRecoveryDisposition::ReadyForLocalReview
    } else {
        RuntimeRestartRecoveryDisposition::NeedsOperatorReview
    };

    let report = RuntimeRestartRecoveryValidationReport {
        validation_version: RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION.to_owned(),
        audit_records_replayed,
        audit_replay_check_passed: true,
        sqlite_reopen_check_passed: true,
        plan_checkpoint_recovered,
        adapter_checkpoint_recovered,
        graceful_shutdown_checkpoint_recovered,
        recovery_disposition,
        local_review_ready: plan_checkpoint_recovered && adapter_checkpoint_recovered,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Run a local deployment-like runtime smoke validation sequence.
///
/// The harness uses caller-supplied local paths, runs one lifecycle, records a
/// graceful-shutdown checkpoint, validates backup/restore and restart recovery,
/// and runs the audit durability probes in a separate workspace. It does not
/// start services, interact with a service manager, inspect deployment state,
/// submit orders, call networks, sign payloads, broadcast transactions,
/// withdraw funds, or bridge assets.
pub fn validate_local_runtime_deployment_smoke(
    audit_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    backup_audit_path: impl AsRef<Path>,
    backup_state_path: impl AsRef<Path>,
    audit_validation_workspace: impl AsRef<Path>,
    policy: &PolicyEngine,
    request: RuntimeDeploymentSmokeValidationRequest,
) -> Result<RuntimeDeploymentSmokeValidationReport, RuntimeLifecycleError> {
    request.validate()?;

    let audit_path = audit_path.as_ref();
    let state_path = state_path.as_ref();
    let backup_audit_path = backup_audit_path.as_ref();
    let backup_state_path = backup_state_path.as_ref();
    let audit_validation_workspace = audit_validation_workspace.as_ref();

    validate_runtime_smoke_target(audit_path, "audit")?;
    validate_runtime_smoke_target(state_path, "state")?;
    validate_runtime_backup_target(audit_path, backup_audit_path, "audit")?;
    validate_runtime_backup_target(state_path, backup_state_path, "state")?;
    if audit_validation_workspace.exists() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: "runtime deployment smoke audit validation workspace must not already exist"
                .to_owned(),
        });
    }

    let mut journal = AppendOnlyAuditJournal::open(audit_path)?;
    let mut store = SqliteWalStateStore::open(state_path)?;
    let lifecycle_record =
        run_local_runtime_lifecycle(&mut journal, &mut store, policy, request.lifecycle_request)?;
    let graceful_shutdown_record =
        run_local_graceful_shutdown_checkpoint(&mut journal, &mut store, request.shutdown_request)?;
    drop(store);
    drop(journal);

    let backup_report = validate_local_runtime_backup_restore(
        audit_path,
        state_path,
        backup_audit_path,
        backup_state_path,
    )?;
    let restart_report = validate_local_runtime_restart_recovery(audit_path, state_path)?;
    let audit_report = validate_audit_journal_durability(
        audit_validation_workspace,
        request.validated_at_unix_ms,
    )?;

    let report = RuntimeDeploymentSmokeValidationReport {
        validation_version: RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION.to_owned(),
        lifecycle_completed: lifecycle_record.status == RuntimeLifecycleStatus::AdapterRunCheckpointed,
        graceful_shutdown_checkpointed: graceful_shutdown_record.shutdown_checkpoint_key
            == RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY,
        backup_restore_validated: backup_report.audit_restore_check_passed
            && backup_report.sqlite_restore_check_passed
            && backup_report.plan_checkpoint_restored
            && backup_report.adapter_checkpoint_restored,
        restart_recovery_validated: restart_report.audit_replay_check_passed
            && restart_report.sqlite_reopen_check_passed
            && restart_report.plan_checkpoint_recovered
            && restart_report.adapter_checkpoint_recovered
            && restart_report.graceful_shutdown_checkpoint_recovered,
        audit_durability_validated: audit_report.append_replay_validated
            && audit_report.truncated_replay_rejected
            && audit_report.tamper_replay_rejected
            && audit_report.concurrent_append_validated
            && audit_report.filesystem_failure_validated
            && audit_report.disk_full_failure_validated,
        restart_audit_records_replayed: restart_report.audit_records_replayed,
        backup_audit_records_replayed: backup_report.audit_records_replayed,
        recovery_disposition: restart_report.recovery_disposition,
        service_manager_action_performed: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        unresolved_blockers: vec![
            "deployment-host service-manager restart execution remains external".to_owned(),
            "physical disk-full and retention/rotation execution evidence remain external"
                .to_owned(),
            "production deployment, live exchange/RPC validation, custody, signing, broadcasts, withdrawals, and bridges remain blocked".to_owned(),
        ],
    };
    report.validate()?;
    Ok(report)
}

/// Runtime lifecycle errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeLifecycleError {
    /// Request or record validation failed.
    ValidationFailed { reason: String },
    /// Planner or plan validation failed.
    Planner(crate::ExecutionPlannerError),
    /// Adapter validation or evaluation failed.
    Adapter(ExecutionAdapterError),
    /// Audit append/replay failed.
    Audit(AuditError),
    /// State checkpoint persistence failed.
    State(StateStoreError),
}

impl fmt::Display for RuntimeLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { reason } => {
                write!(formatter, "runtime lifecycle validation failed: {reason}")
            }
            Self::Planner(error) => write!(formatter, "runtime lifecycle planner failed: {error}"),
            Self::Adapter(error) => write!(formatter, "runtime lifecycle adapter failed: {error}"),
            Self::Audit(error) => write!(formatter, "runtime lifecycle audit failed: {error}"),
            Self::State(error) => write!(formatter, "runtime lifecycle state failed: {error}"),
        }
    }
}

impl Error for RuntimeLifecycleError {}

impl From<AuditError> for RuntimeLifecycleError {
    fn from(error: AuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<StateStoreError> for RuntimeLifecycleError {
    fn from(error: StateStoreError) -> Self {
        Self::State(error)
    }
}

impl From<ExecutionAdapterError> for RuntimeLifecycleError {
    fn from(error: ExecutionAdapterError) -> Self {
        Self::Adapter(error)
    }
}

fn lifecycle_event(
    request: &RuntimeLifecycleRequest,
    kind: AuditEventKind,
    message: &str,
) -> AuditEvent {
    AuditEvent::new(
        format!("runtime:{}:start", request.id),
        kind,
        "runtime-lifecycle",
        "runtime",
        message,
    )
    .with_metadata("lifecycle_id", AuditValue::Text(request.id.clone()))
    .with_metadata("plan_id", AuditValue::Text(request.plan.id.clone()))
    .with_metadata(
        "adapter_request_id",
        AuditValue::Text(request.adapter_request_id.clone()),
    )
    .with_metadata("live_execution", AuditValue::Bool(false))
    .with_metadata("external_submission", AuditValue::Bool(false))
}

fn checkpoint_event(
    request: &RuntimeLifecycleRequest,
    kind: AuditEventKind,
    message: &str,
    checkpoint: &StateCheckpoint,
) -> AuditEvent {
    AuditEvent::new(
        format!("runtime:{}:{}", request.id, checkpoint.key),
        kind,
        "runtime-lifecycle",
        "runtime",
        message,
    )
    .with_metadata("lifecycle_id", AuditValue::Text(request.id.clone()))
    .with_metadata("plan_id", AuditValue::Text(request.plan.id.clone()))
    .with_metadata("checkpoint_key", AuditValue::Text(checkpoint.key.clone()))
    .with_metadata(
        "checkpoint_subsystem",
        AuditValue::Text(checkpoint.subsystem.clone()),
    )
    .with_metadata(
        "checkpoint_updated_at_unix_ms",
        AuditValue::Unsigned(checkpoint.updated_at_unix_ms),
    )
    .with_metadata("live_execution", AuditValue::Bool(false))
    .with_metadata("external_submission", AuditValue::Bool(false))
}

fn graceful_shutdown_event(request: &RuntimeGracefulShutdownRequest, message: &str) -> AuditEvent {
    AuditEvent::new(
        format!("runtime:{}:graceful-shutdown", request.id),
        AuditEventKind::RuntimeLifecycle,
        "runtime-lifecycle",
        "runtime",
        message,
    )
    .with_metadata("shutdown_id", AuditValue::Text(request.id.clone()))
    .with_metadata("shutdown_reason", AuditValue::Text(request.reason.clone()))
    .with_metadata("live_execution", AuditValue::Bool(false))
    .with_metadata("external_submission", AuditValue::Bool(false))
}

fn validate_runtime_backup_target(
    primary_path: &Path,
    backup_path: &Path,
    artifact_label: &str,
) -> Result<(), RuntimeLifecycleError> {
    if primary_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} primary path is required"),
        });
    }
    if backup_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} backup path is required"),
        });
    }
    if primary_path == backup_path {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} backup path must differ from primary"),
        });
    }
    if backup_path.exists() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} backup path must not already exist"),
        });
    }
    Ok(())
}

fn validate_runtime_smoke_target(
    target_path: &Path,
    artifact_label: &str,
) -> Result<(), RuntimeLifecycleError> {
    if target_path.as_os_str().is_empty() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} smoke path is required"),
        });
    }
    if target_path.exists() {
        return Err(RuntimeLifecycleError::ValidationFailed {
            reason: format!("runtime {artifact_label} smoke path must not already exist"),
        });
    }
    if artifact_label == "state" {
        for suffix in ["-wal", "-shm"] {
            let related = std::path::PathBuf::from(format!("{}{suffix}", target_path.display()));
            if related.exists() {
                return Err(RuntimeLifecycleError::ValidationFailed {
                    reason: format!(
                        "runtime state smoke related path must not already exist: {suffix}"
                    ),
                });
            }
        }
    }
    Ok(())
}

fn copy_runtime_backup_file(
    source_path: &Path,
    destination_path: &Path,
    artifact_label: &str,
) -> Result<(), RuntimeLifecycleError> {
    if let Some(parent) = destination_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| StateStoreError::BackendFailed {
                reason: format!("failed to create runtime {artifact_label} backup parent: {error}"),
            })?;
        }
    }
    fs::copy(source_path, destination_path).map_err(|error| StateStoreError::BackendFailed {
        reason: format!("failed to copy runtime {artifact_label} backup: {error}"),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        run_local_graceful_shutdown_checkpoint, run_local_runtime_lifecycle,
        validate_local_runtime_backup_restore, validate_local_runtime_deployment_smoke,
        validate_local_runtime_restart_recovery, RuntimeDeploymentSmokeValidationRequest,
        RuntimeGracefulShutdownRequest, RuntimeLifecycleError, RuntimeLifecycleRequest,
        RuntimeLifecycleStatus, RuntimeRestartRecoveryDisposition,
        EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
        RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION, RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, AuditEvent, AuditEventKind,
        DeterministicExecutionPlanner, ExecutionAdapterConfig, ExecutionPlanner,
        ExecutionPlannerConfig, ExecutionPlannerRequest, FeeAdjustedEdge, FeeEstimate,
        InMemoryStateStore, LiquidityRole, MarketPair, OpportunityCandidate, OpportunityLeg,
        OpportunityLegSide, OpportunityRouteKind, OpportunityScore, PolicyEngine,
        SqliteWalStateStore, StateCheckpoint, StateStore, StateStoreError, VenueKind, VenueRef,
    };
    use std::{
        env, fs,
        path::PathBuf,
        process,
        sync::{Arc, Barrier, Mutex},
        thread,
    };

    const PAPER_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 1_000.0
max_daily_loss_quote = 100.0
max_open_exposure_quote = 2_000.0
slippage_bps = 100
gas_fee_cap_quote = 10.0

[venues]
cex_allowlist = ["paper-a", "paper-b"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC", "USD"]

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
    fn runtime_lifecycle_audits_and_persists_before_adapter_completion() {
        let path = temp_audit_path("runtime-lifecycle");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let mut store = InMemoryStateStore::new();
        let policy = policy();
        let request = request(&policy);

        let record = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");

        assert_eq!(
            record.status,
            RuntimeLifecycleStatus::AdapterRunCheckpointed
        );
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert_eq!(record.start_audit_sequence, 1);
        assert_eq!(record.plan_checkpoint_audit_sequence, 2);
        assert_eq!(record.adapter_complete_audit_sequence, 3);
        assert_eq!(journal.next_sequence(), 4);
        assert!(store
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_some());
        assert!(store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_some());

        let reopened = AppendOnlyAuditJournal::open(&path).expect("journal reopens");
        assert_eq!(reopened.next_sequence(), 4);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_lifecycle_rejects_live_scope_before_audit_or_state() {
        let path = temp_audit_path("runtime-live-denied");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let mut store = InMemoryStateStore::new();
        let policy = policy();
        let mut request = request(&policy);
        request.plan.scope = crate::ExecutionScope::Live;

        let error = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect_err("live lifecycle must be rejected");

        assert!(error.to_string().contains("live-scope"));
        assert_eq!(journal.next_sequence(), 1);
        assert!(store.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_lifecycle_state_permission_failure_stops_before_adapter() {
        let path = temp_audit_path("runtime-state-permission-denied");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let mut store = PermissionDeniedStateStore::default();
        let policy = policy();
        let request = request(&policy);

        let error = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect_err("state permission failure must fail closed");

        match error {
            RuntimeLifecycleError::State(StateStoreError::BackendFailed { reason }) => {
                assert!(reason.contains("simulated permission-denied state path"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(store.put_attempts, 1);
        assert_eq!(journal.next_sequence(), 2);

        let reopened = AppendOnlyAuditJournal::open(&path).expect("journal reopens");
        assert_eq!(reopened.next_sequence(), 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_lifecycle_persists_through_sqlite_wal_store() {
        let audit_path = temp_audit_path("runtime-sqlite");
        let state_path = temp_state_path("runtime-sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        let record = run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");

        drop(store);

        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let plan_checkpoint = reopened
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .expect("plan checkpoint exists");
        let adapter_checkpoint = reopened
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .expect("adapter checkpoint exists");

        assert_eq!(plan_checkpoint.key, record.plan_checkpoint_key);
        assert_eq!(adapter_checkpoint.key, record.adapter_run_checkpoint_key);
        assert_eq!(
            adapter_checkpoint.updated_at_unix_ms,
            record.created_at_unix_ms
        );

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn graceful_shutdown_checkpoint_reopens_audit_and_sqlite_state() {
        let audit_path = temp_audit_path("runtime-graceful-shutdown");
        let state_path = temp_state_path("runtime-graceful-shutdown");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");

        let record = run_local_graceful_shutdown_checkpoint(
            &mut journal,
            &mut store,
            RuntimeGracefulShutdownRequest {
                id: "shutdown-1".to_owned(),
                reason: "operator-requested-local-stop".to_owned(),
                now_unix_ms: 30_000,
            },
        )
        .expect("graceful shutdown checkpoint should persist");

        assert_eq!(record.shutdown_start_audit_sequence, 1);
        assert_eq!(record.shutdown_checkpoint_audit_sequence, 2);
        assert_eq!(
            record.shutdown_checkpoint_key,
            RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY
        );
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);

        drop(store);
        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 3);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        let checkpoint = reopened_store
            .get_checkpoint(RUNTIME_GRACEFUL_SHUTDOWN_CHECKPOINT_KEY)
            .expect("shutdown checkpoint reads")
            .expect("shutdown checkpoint exists");
        assert_eq!(checkpoint.value, record.shutdown_checkpoint_value);
        assert_eq!(checkpoint.updated_at_unix_ms, record.created_at_unix_ms);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_backup_restore_replays_audit_and_sqlite_checkpoints() {
        let audit_path = temp_audit_path("runtime-backup-primary");
        let state_path = temp_state_path("runtime-backup-primary");
        let backup_audit_path = temp_audit_path("runtime-backup-copy");
        let backup_state_path = temp_state_path("runtime-backup-copy");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        let lifecycle_record =
            run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
                .expect("runtime lifecycle should complete");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_backup_restore(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
        )
        .expect("local runtime backup/restore validation should pass");

        assert_eq!(report.audit_records_replayed, 3);
        assert!(report.audit_restore_check_passed);
        assert!(report.sqlite_restore_check_passed);
        assert!(report.plan_checkpoint_restored);
        assert!(report.adapter_checkpoint_restored);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let restored_journal =
            AppendOnlyAuditJournal::open(&backup_audit_path).expect("backup journal reopens");
        assert_eq!(restored_journal.next_sequence(), 4);

        let restored_state =
            SqliteWalStateStore::open(&backup_state_path).expect("backup sqlite reopens");
        let plan_checkpoint = restored_state
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .expect("plan checkpoint exists");
        let adapter_checkpoint = restored_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .expect("adapter checkpoint exists");
        assert_eq!(plan_checkpoint.key, lifecycle_record.plan_checkpoint_key);
        assert_eq!(
            adapter_checkpoint.key,
            lifecycle_record.adapter_run_checkpoint_key
        );

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
    }

    #[test]
    fn runtime_restart_recovery_replays_audit_and_reopens_sqlite_checkpoints() {
        let audit_path = temp_audit_path("runtime-restart-recovery");
        let state_path = temp_state_path("runtime-restart-recovery");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");
        run_local_graceful_shutdown_checkpoint(
            &mut journal,
            &mut store,
            RuntimeGracefulShutdownRequest {
                id: "shutdown-before-restart-recovery".to_owned(),
                reason: "local-restart-recovery-test".to_owned(),
                now_unix_ms: 50_000,
            },
        )
        .expect("graceful shutdown checkpoint should persist");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect("restart recovery validation should pass");

        assert_eq!(report.audit_records_replayed, 5);
        assert!(report.audit_replay_check_passed);
        assert!(report.sqlite_reopen_check_passed);
        assert!(report.plan_checkpoint_recovered);
        assert!(report.adapter_checkpoint_recovered);
        assert!(report.graceful_shutdown_checkpoint_recovered);
        assert_eq!(
            report.recovery_disposition,
            RuntimeRestartRecoveryDisposition::ReadyForLocalReview
        );
        assert!(report.local_review_ready);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_restart_recovery_needs_operator_review_without_shutdown_checkpoint() {
        let audit_path = temp_audit_path("runtime-restart-recovery-review-needed");
        let state_path = temp_state_path("runtime-restart-recovery-review-needed");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");
        let policy = policy();
        let request = request(&policy);

        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
            .expect("runtime lifecycle should complete");
        drop(store);
        drop(journal);

        let report = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect("restart recovery validation should pass with operator review");

        assert_eq!(report.audit_records_replayed, 3);
        assert!(report.audit_replay_check_passed);
        assert!(report.sqlite_reopen_check_passed);
        assert!(report.plan_checkpoint_recovered);
        assert!(report.adapter_checkpoint_recovered);
        assert!(!report.graceful_shutdown_checkpoint_recovered);
        assert_eq!(
            report.recovery_disposition,
            RuntimeRestartRecoveryDisposition::NeedsOperatorReview
        );
        assert!(report.local_review_ready);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_restart_recovery_fails_closed_when_sqlite_checkpoints_missing() {
        let audit_path = temp_audit_path("runtime-restart-recovery-missing-state");
        let state_path = temp_state_path("runtime-restart-recovery-missing-state");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let store = SqliteWalStateStore::open(&state_path).expect("sqlite store opens");

        journal
            .append_event(AuditEvent::new(
                "runtime:incomplete-recovery:start",
                AuditEventKind::RuntimeLifecycle,
                "runtime-lifecycle",
                "runtime",
                "runtime lifecycle started without durable checkpoints",
            ))
            .expect("audit event should append");
        drop(store);
        drop(journal);

        let error = validate_local_runtime_restart_recovery(&audit_path, &state_path)
            .expect_err("restart recovery must fail closed when checkpoints are missing");

        assert!(error
            .to_string()
            .contains("coherent local audit/state checkpoints"));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(reopened_journal.next_sequence(), 2);
        let reopened_state = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        assert!(reopened_state
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_none());
        assert!(reopened_state
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_none());

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn concurrent_runtime_lifecycles_share_audit_and_sqlite_state() {
        let audit_path = temp_audit_path("runtime-concurrent");
        let state_path = temp_state_path("runtime-concurrent");
        let workers = 4_usize;
        let barrier = Arc::new(Barrier::new(workers));
        let open_lock = Arc::new(Mutex::new(()));
        let handles = (0..workers)
            .map(|worker| {
                let audit_path = audit_path.clone();
                let state_path = state_path.clone();
                let barrier = Arc::clone(&barrier);
                let open_lock = Arc::clone(&open_lock);
                thread::spawn(move || {
                    let policy = policy();
                    let mut request = request(&policy);
                    request.id = format!("runtime-concurrent-{worker}");
                    request.adapter_request_id = format!("adapter-concurrent-{worker}");
                    request.plan.id = format!("plan-concurrent-{worker}");
                    request.now_unix_ms = 40_000 + u64::try_from(worker).unwrap_or(u64::MAX);

                    let (mut journal, mut store) = {
                        let _guard = open_lock.lock().expect("open lock should not be poisoned");
                        (
                            AppendOnlyAuditJournal::open(&audit_path).expect("journal opens"),
                            SqliteWalStateStore::open(&state_path).expect("sqlite store opens"),
                        )
                    };
                    barrier.wait();

                    let record =
                        run_local_runtime_lifecycle(&mut journal, &mut store, &policy, request)
                            .expect("runtime lifecycle should complete");

                    assert!(!record.external_submission_performed);
                    assert!(!record.live_execution_performed);
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().expect("worker should not panic");
        }

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal reopens");
        assert_eq!(
            reopened_journal.next_sequence(),
            1 + u64::try_from(workers * 3).unwrap_or(u64::MAX)
        );

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite store reopens");
        assert!(reopened_store
            .get_checkpoint(EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY)
            .expect("plan checkpoint reads")
            .is_some());
        assert!(reopened_store
            .get_checkpoint(EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY)
            .expect("adapter checkpoint reads")
            .is_some());
        reopened_store
            .integrity_check()
            .expect("sqlite integrity check should pass");

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn runtime_deployment_smoke_validates_local_artifact_sequence() {
        let audit_path = temp_audit_path("runtime-deployment-smoke");
        let state_path = temp_state_path("runtime-deployment-smoke");
        let backup_audit_path = temp_audit_path("runtime-deployment-smoke-backup");
        let backup_state_path = temp_state_path("runtime-deployment-smoke-backup");
        let audit_validation_workspace = temp_workspace_path("runtime-deployment-smoke-audit");
        let policy = policy();
        let lifecycle_request = request(&policy);
        let shutdown_request = RuntimeGracefulShutdownRequest {
            id: "shutdown-before-deployment-smoke".to_owned(),
            reason: "local-deployment-smoke-test".to_owned(),
            now_unix_ms: 60_000,
        };

        let report = validate_local_runtime_deployment_smoke(
            &audit_path,
            &state_path,
            &backup_audit_path,
            &backup_state_path,
            &audit_validation_workspace,
            &policy,
            RuntimeDeploymentSmokeValidationRequest {
                lifecycle_request,
                shutdown_request,
                validated_at_unix_ms: 70_000,
            },
        )
        .expect("local deployment-like smoke validation should pass");

        assert_eq!(
            report.validation_version,
            RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION
        );
        assert!(report.lifecycle_completed);
        assert!(report.graceful_shutdown_checkpointed);
        assert!(report.backup_restore_validated);
        assert!(report.restart_recovery_validated);
        assert!(report.audit_durability_validated);
        assert_eq!(report.restart_audit_records_replayed, 5);
        assert_eq!(report.backup_audit_records_replayed, 5);
        assert_eq!(
            report.recovery_disposition,
            RuntimeRestartRecoveryDisposition::ReadyForLocalReview
        );
        assert!(!report.service_manager_action_performed);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        assert!(report
            .unresolved_blockers
            .iter()
            .any(|blocker| blocker.contains("service-manager")));

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(backup_audit_path);
        cleanup_state_files(&state_path);
        cleanup_state_files(&backup_state_path);
        let _ = fs::remove_dir_all(audit_validation_workspace);
    }

    fn request(policy: &PolicyEngine) -> RuntimeLifecycleRequest {
        RuntimeLifecycleRequest {
            id: "runtime-lifecycle-1".to_owned(),
            adapter_request_id: "adapter-request-1".to_owned(),
            plan: planner_plan(policy),
            adapter_config: ExecutionAdapterConfig::default(),
            now_unix_ms: 20_000,
        }
    }

    fn policy() -> PolicyEngine {
        PolicyEngine::from_config(
            AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate"),
        )
    }

    fn planner_plan(policy: &PolicyEngine) -> crate::ExecutionPlanDraft {
        let request = ExecutionPlannerRequest {
            id: "planner-request-1".to_owned(),
            strategy_id: "strategy-basic-arb".to_owned(),
            candidate: candidate(),
            config: ExecutionPlannerConfig::default(),
            default_chain: None,
            now_unix_ms: 10_000,
        };
        DeterministicExecutionPlanner::new()
            .plan(&request, policy)
            .expect("planner should create a draft")
    }

    fn candidate() -> OpportunityCandidate {
        let pair = MarketPair::new("BTC", "USD").expect("pair should validate");
        let edge = FeeAdjustedEdge::calculate(15.0, 2.0, 100.0).expect("edge should validate");
        OpportunityCandidate {
            id: "opp-cex-cex-btc-usd".to_owned(),
            route_kind: OpportunityRouteKind::CexCex,
            pair: pair.clone(),
            legs: vec![
                leg("paper-a", pair.clone(), OpportunityLegSide::Buy, 100.0, 1.0),
                leg("paper-b", pair, OpportunityLegSide::Sell, 115.0, 1.0),
            ],
            edge,
            score: OpportunityScore {
                roi_bps: edge.roi_bps,
                freshness_penalty_bps: 0.0,
                risk_penalty_bps: 0.0,
                score_bps: edge.roi_bps,
            },
            liquidity_model: None,
            transfer_risk: None,
            discovered_at_unix_ms: 9_900,
            source_quote_ids: vec!["quote-a".to_owned(), "quote-b".to_owned()],
            warnings: Vec::new(),
        }
    }

    fn leg(
        venue_name: &str,
        pair: MarketPair,
        side: OpportunityLegSide,
        price_quote: f64,
        quantity_base: f64,
    ) -> OpportunityLeg {
        let notional_quote = price_quote * quantity_base;
        OpportunityLeg {
            venue: VenueRef {
                name: venue_name.to_owned(),
                kind: VenueKind::Cex,
            },
            pair: pair.clone(),
            side,
            price_quote,
            quantity_base,
            notional_quote,
            fee_estimate: FeeEstimate {
                venue: VenueRef {
                    name: venue_name.to_owned(),
                    kind: VenueKind::Cex,
                },
                pair: Some(pair),
                notional_quote,
                liquidity_role: LiquidityRole::Taker,
                fee_bps: 10.0,
                venue_fee_quote: 1.0,
                network_fee_quote: 0.0,
                total_fee_quote: 1.0,
                externally_verified: true,
            },
            source_quote_id: format!("quote-{venue_name}"),
            market_data_age_ms: 100,
        }
    }

    fn temp_audit_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-runtime-{label}-{}-{nanos}.jsonl",
            process::id()
        ));
        path
    }

    fn temp_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-runtime-{label}-{}-{nanos}.sqlite3",
            process::id()
        ));
        path
    }

    fn temp_workspace_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-runtime-{label}-{}-{nanos}",
            process::id()
        ));
        path
    }

    fn cleanup_state_files(path: &std::path::Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", path.display())));
        }
    }

    #[derive(Default)]
    struct PermissionDeniedStateStore {
        put_attempts: usize,
    }

    impl StateStore for PermissionDeniedStateStore {
        fn put_checkpoint(&mut self, _checkpoint: StateCheckpoint) -> Result<(), StateStoreError> {
            self.put_attempts += 1;
            Err(StateStoreError::BackendFailed {
                reason: "simulated permission-denied state path".to_owned(),
            })
        }

        fn get_checkpoint(&self, _key: &str) -> Result<Option<StateCheckpoint>, StateStoreError> {
            Ok(None)
        }
    }
}
