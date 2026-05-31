#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Error as IoError, ErrorKind, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};

/// Genesis hash marker for the first append-only audit record.
pub const AUDIT_GENESIS_HASH: &str = "GENESIS";

/// Current audit journal format version.
pub const AUDIT_JOURNAL_FORMAT_VERSION: &str = "audit-jsonl-hash-chain-v1";

/// Current local audit durability validation version.
pub const AUDIT_DURABILITY_VALIDATION_VERSION: &str =
    "phase-26-audit-crash-concurrency-filesystem-disk-full-stale-lock-v1";

const AUDIT_LOCK_RETRY_COUNT: usize = 200;
const AUDIT_LOCK_RETRY_DELAY: Duration = Duration::from_millis(5);

/// Append-only local audit journal with hash-chained JSONL records.
///
/// The journal is intentionally small and local-first for the single-binary
/// runtime. It is not a replacement for later SQLite WAL storage, external log
/// shipping, deployment-host durability validation, disk-full validation,
/// retention/rotation validation, or SIEM ingestion.
#[derive(Debug)]
pub struct AppendOnlyAuditJournal {
    path: PathBuf,
    next_sequence: u64,
    previous_hash: String,
}

impl AppendOnlyAuditJournal {
    /// Open or create an append-only audit journal and replay existing records.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AuditError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|source| AuditError::Io {
                    path: parent.display().to_string(),
                    reason: source.to_string(),
                })?;
            }
        }

        if !path.exists() {
            File::create(&path).map_err(|source| AuditError::Io {
                path: path.display().to_string(),
                reason: source.to_string(),
            })?;
        }

        let replay = replay_journal(&path)?;
        Ok(Self {
            path,
            next_sequence: replay.next_sequence,
            previous_hash: replay.previous_hash,
        })
    }

    /// Return the backing path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the next sequence number that will be assigned.
    #[must_use]
    pub const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Return the latest hash in the chain.
    #[must_use]
    pub fn previous_hash(&self) -> &str {
        &self.previous_hash
    }

    /// Append one validated event and return the immutable audit record.
    pub fn append_event(&mut self, event: AuditEvent) -> Result<AuditRecord, AuditError> {
        self.append_event_with_fault(event, AuditAppendFault::None)
    }

    fn append_event_with_fault(
        &mut self,
        event: AuditEvent,
        fault: AuditAppendFault,
    ) -> Result<AuditRecord, AuditError> {
        event.validate()?;
        let _lock = AuditJournalFileLock::acquire(&self.path)?;
        let replay = replay_journal(&self.path)?;

        let sequence = replay.next_sequence;
        let previous_hash = replay.previous_hash;
        let record_hash = compute_record_hash(sequence, &previous_hash, &event)?;
        let record = AuditRecord {
            format_version: AUDIT_JOURNAL_FORMAT_VERSION.to_owned(),
            sequence,
            previous_hash,
            event,
            record_hash,
        };

        let line = serde_json::to_string(&record).map_err(|source| AuditError::Serialize {
            reason: source.to_string(),
        })?;

        match fault {
            AuditAppendFault::None => {}
            AuditAppendFault::DiskFullBeforeWrite => {
                return Err(AuditError::DiskFull {
                    path: self.path.display().to_string(),
                    reason: "simulated disk-full before audit append write".to_owned(),
                });
            }
            #[cfg(test)]
            AuditAppendFault::PermissionDeniedBeforeWrite => {
                return Err(AuditError::Io {
                    path: self.path.display().to_string(),
                    reason: "simulated permission-denied before audit append write".to_owned(),
                });
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| audit_io_error(&self.path, source))?;
        writeln!(file, "{line}").map_err(|source| audit_io_error(&self.path, source))?;
        file.flush()
            .map_err(|source| audit_io_error(&self.path, source))?;
        file.sync_all()
            .map_err(|source| audit_io_error(&self.path, source))?;

        self.next_sequence = sequence
            .checked_add(1)
            .ok_or(AuditError::SequenceOverflow)?;
        self.previous_hash = record.record_hash.clone();

        Ok(record)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuditAppendFault {
    None,
    DiskFullBeforeWrite,
    #[cfg(test)]
    PermissionDeniedBeforeWrite,
}

/// Local audit journal durability validation report.
///
/// This is a local filesystem validation harness. It does not prove behavior on
/// every production filesystem, container runtime, service manager, or remote
/// storage layer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditDurabilityValidationReport {
    /// Validation version.
    pub validation_version: String,
    /// Workspace directory used for non-secret test journals.
    pub workspace_dir: String,
    /// Whether basic append/reopen replay passed.
    pub append_replay_validated: bool,
    /// Whether a crash-like truncated JSONL line was rejected on replay.
    pub truncated_replay_rejected: bool,
    /// Whether a tampered record was rejected on replay.
    pub tamper_replay_rejected: bool,
    /// Whether concurrent local appenders produced a replayable hash chain.
    pub concurrent_append_validated: bool,
    /// Whether an invalid filesystem shape failed closed.
    pub filesystem_failure_validated: bool,
    /// Whether simulated disk-full append failure failed closed.
    pub disk_full_failure_validated: bool,
    /// Number of records appended by the append/replay probe.
    pub append_records: usize,
    /// Number of records appended by the concurrent append probe.
    pub concurrent_records: usize,
    /// Whether live network access was used.
    pub live_network_used: bool,
    /// Whether external execution was performed.
    pub external_execution_performed: bool,
    /// Whether this report approves production use.
    pub production_ready: bool,
    /// Remaining blockers in non-secret wording.
    pub unresolved_blockers: Vec<String>,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Local audit retention and rotation policy.
///
/// This model plans retention decisions only. It never deletes files, renames
/// files, compresses archives, or starts a background rotation task.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRetentionPolicy {
    /// Maximum active journal size before a rotation should be requested.
    pub max_active_bytes: u64,
    /// Maximum archived journals to retain after age filtering.
    pub max_archived_files: usize,
    /// Maximum archive age in milliseconds.
    pub retention_window_ms: u64,
}

/// Metadata for one local audit journal file supplied to the retention model.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditJournalFileMetadata {
    /// Local path or sanitized file label.
    pub path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last modified timestamp in Unix milliseconds.
    pub modified_at_unix_ms: u64,
    /// Whether this is the active append target.
    pub active: bool,
}

/// Local retention/rotation plan for audit journals.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRetentionPlan {
    /// Whether the active journal should be rotated by a future operator/runtime.
    pub rotate_active: bool,
    /// Active journal path or label.
    pub active_path: String,
    /// Archive paths retained by the model.
    pub retained_archives: Vec<String>,
    /// Archive paths marked expired by the model.
    pub expired_archives: Vec<String>,
    /// Whether this model deleted files.
    pub deletion_performed: bool,
    /// Whether this model changed the filesystem.
    pub filesystem_mutated: bool,
    /// Whether this plan approves production readiness.
    pub production_ready: bool,
}

/// Local stale-lock recheck policy for audit journal restart planning.
///
/// This model never removes lock files or starts services. It only records
/// whether a caller-supplied lock observation should be rechecked by a future
/// operator or service manager after restart.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditStaleLockPolicy {
    /// Age threshold after which a lock should be treated as stale.
    pub stale_after_ms: u64,
}

/// Caller-supplied metadata for one observed audit journal lock file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditLockFileMetadata {
    /// Local path or sanitized file label for the lock.
    pub path: String,
    /// Process id recorded in the lock metadata.
    pub owner_pid: u32,
    /// Timestamp when the lock was acquired in Unix milliseconds.
    pub acquired_at_unix_ms: u64,
}

/// Side-effect-free stale-lock recheck plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditStaleLockPlan {
    /// Lock path or sanitized file label.
    pub lock_path: String,
    /// Lock owner process id from caller-supplied metadata.
    pub owner_pid: u32,
    /// Observed lock age in milliseconds.
    pub lock_age_ms: u64,
    /// Whether the lock age exceeds the configured stale threshold.
    pub stale: bool,
    /// Whether a future operator/service-manager restart recheck is required.
    pub service_manager_recheck_required: bool,
    /// Whether this model removed the lock file.
    pub lock_removal_performed: bool,
    /// Whether this model changed the filesystem.
    pub filesystem_mutated: bool,
    /// Whether this plan approves production readiness.
    pub production_ready: bool,
}

/// Build a local audit retention/rotation plan from caller-supplied metadata.
///
/// The plan is intentionally side-effect free. It records which archives would
/// be retained or marked expired by a future runtime, but it does not mutate or
/// delete local files.
pub fn plan_audit_journal_retention(
    policy: &AuditRetentionPolicy,
    files: &[AuditJournalFileMetadata],
    now_unix_ms: u64,
) -> Result<AuditRetentionPlan, AuditError> {
    if policy.max_active_bytes == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit retention max_active_bytes must be non-zero".to_owned(),
        });
    }
    if policy.retention_window_ms == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit retention window must be non-zero".to_owned(),
        });
    }
    if now_unix_ms == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit retention timestamp must be non-zero".to_owned(),
        });
    }

    let active_files = files.iter().filter(|file| file.active).collect::<Vec<_>>();
    if active_files.len() != 1 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit retention plan requires exactly one active journal".to_owned(),
        });
    }
    if files.iter().any(|file| file.path.trim().is_empty()) {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit retention file paths must be non-empty".to_owned(),
        });
    }

    let active = active_files[0];
    let rotate_active = active.size_bytes > policy.max_active_bytes;
    let cutoff = now_unix_ms.saturating_sub(policy.retention_window_ms);
    let mut archives = files.iter().filter(|file| !file.active).collect::<Vec<_>>();
    archives.sort_by(|left, right| {
        right
            .modified_at_unix_ms
            .cmp(&left.modified_at_unix_ms)
            .then_with(|| left.path.cmp(&right.path))
    });

    let mut retained_archives = Vec::new();
    let mut expired_archives = Vec::new();
    for archive in archives {
        if archive.modified_at_unix_ms < cutoff
            || retained_archives.len() >= policy.max_archived_files
        {
            expired_archives.push(archive.path.clone());
        } else {
            retained_archives.push(archive.path.clone());
        }
    }

    Ok(AuditRetentionPlan {
        rotate_active,
        active_path: active.path.clone(),
        retained_archives,
        expired_archives,
        deletion_performed: false,
        filesystem_mutated: false,
        production_ready: false,
    })
}

/// Build a side-effect-free stale-lock recheck plan from caller-supplied lock metadata.
///
/// The plan intentionally does not inspect processes, remove lock files, start
/// services, or mutate deployment state. A stale result is only a request for a
/// future operator/runtime to recheck the lock under deployment-specific rules.
pub fn plan_audit_stale_lock_recheck(
    policy: &AuditStaleLockPolicy,
    lock: &AuditLockFileMetadata,
    now_unix_ms: u64,
) -> Result<AuditStaleLockPlan, AuditError> {
    if policy.stale_after_ms == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit stale-lock threshold must be non-zero".to_owned(),
        });
    }
    if lock.path.trim().is_empty() {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit stale-lock path must be non-empty".to_owned(),
        });
    }
    if lock.owner_pid == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit stale-lock owner pid must be non-zero".to_owned(),
        });
    }
    if lock.acquired_at_unix_ms == 0 || now_unix_ms == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit stale-lock timestamps must be non-zero".to_owned(),
        });
    }
    if lock.acquired_at_unix_ms > now_unix_ms {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit stale-lock acquired timestamp cannot be in the future".to_owned(),
        });
    }

    let lock_age_ms = now_unix_ms - lock.acquired_at_unix_ms;
    let stale = lock_age_ms >= policy.stale_after_ms;

    Ok(AuditStaleLockPlan {
        lock_path: lock.path.clone(),
        owner_pid: lock.owner_pid,
        lock_age_ms,
        stale,
        service_manager_recheck_required: stale,
        lock_removal_performed: false,
        filesystem_mutated: false,
        production_ready: false,
    })
}

/// Run local audit crash/concurrency/filesystem validation probes.
///
/// The supplied workspace directory must be empty or absent. The harness creates
/// local JSONL files only, uses no network, and stores no secrets.
pub fn validate_audit_journal_durability(
    workspace_dir: impl AsRef<Path>,
    validated_at_unix_ms: u64,
) -> Result<AuditDurabilityValidationReport, AuditError> {
    if validated_at_unix_ms == 0 {
        return Err(AuditError::ValidationHarnessFailed {
            reason: "audit validation timestamp must be non-zero".to_owned(),
        });
    }

    let workspace_dir = workspace_dir.as_ref().to_path_buf();
    if workspace_dir.exists() {
        return Err(AuditError::ValidationHarnessFailed {
            reason: format!(
                "audit validation workspace already exists: {}",
                workspace_dir.display()
            ),
        });
    }
    fs::create_dir_all(&workspace_dir).map_err(|source| AuditError::Io {
        path: workspace_dir.display().to_string(),
        reason: source.to_string(),
    })?;

    let main_path = workspace_dir.join("audit-validation.jsonl");
    let append_replay_validated = validate_append_replay_probe(&main_path)?;
    let truncated_replay_rejected = validate_truncated_replay_probe(&main_path, &workspace_dir)?;
    let tamper_replay_rejected = validate_tamper_replay_probe(&main_path, &workspace_dir)?;
    let concurrent_records = 32;
    let concurrent_append_validated =
        validate_concurrent_append_probe(&workspace_dir, concurrent_records)?;
    let filesystem_failure_validated = validate_filesystem_failure_probe(&workspace_dir)?;
    let disk_full_failure_validated = validate_disk_full_failure_probe(&workspace_dir)?;

    let mut unresolved_blockers = Vec::new();
    if !append_replay_validated {
        unresolved_blockers.push("audit append/reopen replay probe failed".to_owned());
    }
    if !truncated_replay_rejected {
        unresolved_blockers.push("audit truncated-record replay was not rejected".to_owned());
    }
    if !tamper_replay_rejected {
        unresolved_blockers.push("audit tamper replay was not rejected".to_owned());
    }
    if !concurrent_append_validated {
        unresolved_blockers.push("audit concurrent append replay probe failed".to_owned());
    }
    if !filesystem_failure_validated {
        unresolved_blockers.push("audit filesystem failure probe did not fail closed".to_owned());
    }
    if !disk_full_failure_validated {
        unresolved_blockers.push("audit disk-full failure probe did not fail closed".to_owned());
    }
    unresolved_blockers.push(
        "deployment-host audit validation, physical disk-full evidence, retention/rotation validation, and service-manager restart validation remain external".to_owned(),
    );

    Ok(AuditDurabilityValidationReport {
        validation_version: AUDIT_DURABILITY_VALIDATION_VERSION.to_owned(),
        workspace_dir: workspace_dir.display().to_string(),
        append_replay_validated,
        truncated_replay_rejected,
        tamper_replay_rejected,
        concurrent_append_validated,
        filesystem_failure_validated,
        disk_full_failure_validated,
        append_records: 2,
        concurrent_records,
        live_network_used: false,
        external_execution_performed: false,
        production_ready: false,
        unresolved_blockers,
        validated_at_unix_ms,
    })
}

fn validate_append_replay_probe(path: &Path) -> Result<bool, AuditError> {
    let mut journal = AppendOnlyAuditJournal::open(path)?;
    journal.append_event(validation_event("append-1"))?;
    journal.append_event(validation_event("append-2"))?;
    let reopened = AppendOnlyAuditJournal::open(path)?;
    Ok(reopened.next_sequence() == 3 && reopened.previous_hash() == journal.previous_hash())
}

fn validate_truncated_replay_probe(
    main_path: &Path,
    workspace_dir: &Path,
) -> Result<bool, AuditError> {
    let truncated_path = workspace_dir.join("audit-validation-truncated.jsonl");
    fs::copy(main_path, &truncated_path).map_err(|source| AuditError::Io {
        path: truncated_path.display().to_string(),
        reason: source.to_string(),
    })?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(&truncated_path)
        .map_err(|source| AuditError::Io {
            path: truncated_path.display().to_string(),
            reason: source.to_string(),
        })?;
    write!(file, "{{\"partial-crash-record\"").map_err(|source| AuditError::Io {
        path: truncated_path.display().to_string(),
        reason: source.to_string(),
    })?;
    file.flush().map_err(|source| AuditError::Io {
        path: truncated_path.display().to_string(),
        reason: source.to_string(),
    })?;
    file.sync_all().map_err(|source| AuditError::Io {
        path: truncated_path.display().to_string(),
        reason: source.to_string(),
    })?;
    Ok(matches!(
        AppendOnlyAuditJournal::open(&truncated_path),
        Err(AuditError::Deserialize { .. })
    ))
}

fn validate_tamper_replay_probe(
    main_path: &Path,
    workspace_dir: &Path,
) -> Result<bool, AuditError> {
    let tampered_path = workspace_dir.join("audit-validation-tampered.jsonl");
    let tampered = fs::read_to_string(main_path)
        .map_err(|source| AuditError::Io {
            path: main_path.display().to_string(),
            reason: source.to_string(),
        })?
        .replace(
            "audit durability validation event",
            "audit durability tampered event",
        );
    fs::write(&tampered_path, tampered).map_err(|source| AuditError::Io {
        path: tampered_path.display().to_string(),
        reason: source.to_string(),
    })?;
    Ok(matches!(
        AppendOnlyAuditJournal::open(&tampered_path),
        Err(AuditError::IntegrityViolation { .. })
    ))
}

fn validate_concurrent_append_probe(
    workspace_dir: &Path,
    concurrent_records: usize,
) -> Result<bool, AuditError> {
    let concurrent_path = workspace_dir.join("audit-validation-concurrent.jsonl");
    let concurrent_writers = 4_usize;
    let events_per_writer = concurrent_records / concurrent_writers;
    let mut handles = Vec::with_capacity(concurrent_writers);
    for writer_index in 0..concurrent_writers {
        let path = concurrent_path.clone();
        handles.push(thread::spawn(move || -> Result<(), AuditError> {
            for event_index in 0..events_per_writer {
                let mut journal = AppendOnlyAuditJournal::open(&path)?;
                journal.append_event(validation_event(format!(
                    "concurrent-{writer_index}-{event_index}"
                )))?;
            }
            Ok(())
        }));
    }
    for handle in handles {
        handle
            .join()
            .map_err(|_| AuditError::ValidationHarnessFailed {
                reason: "audit concurrent validation thread panicked".to_owned(),
            })??;
    }
    let concurrent_replay = AppendOnlyAuditJournal::open(&concurrent_path)?;
    Ok(concurrent_replay.next_sequence()
        == u64::try_from(concurrent_records)
            .unwrap_or(u64::MAX)
            .saturating_add(1))
}

fn validate_filesystem_failure_probe(workspace_dir: &Path) -> Result<bool, AuditError> {
    let file_parent = workspace_dir.join("not-a-directory");
    fs::write(&file_parent, "not a directory").map_err(|source| AuditError::Io {
        path: file_parent.display().to_string(),
        reason: source.to_string(),
    })?;
    let invalid_path = file_parent.join("audit.jsonl");
    Ok(matches!(
        AppendOnlyAuditJournal::open(&invalid_path),
        Err(AuditError::Io { .. })
    ))
}

fn validate_disk_full_failure_probe(workspace_dir: &Path) -> Result<bool, AuditError> {
    let path = workspace_dir.join("audit-validation-disk-full.jsonl");
    let mut journal = AppendOnlyAuditJournal::open(&path)?;
    let first = journal.append_event(validation_event("disk-full-before"))?;
    let next_sequence_before = journal.next_sequence();
    let previous_hash_before = journal.previous_hash().to_owned();
    let error = journal
        .append_event_with_fault(
            validation_event("disk-full-denied"),
            AuditAppendFault::DiskFullBeforeWrite,
        )
        .expect_err("simulated disk-full append should fail closed");

    let in_memory_unchanged = journal.next_sequence() == next_sequence_before
        && journal.previous_hash() == previous_hash_before;
    let replay = AppendOnlyAuditJournal::open(&path)?;
    let replay_unchanged = replay.next_sequence() == next_sequence_before
        && replay.previous_hash() == first.record_hash;

    Ok(matches!(error, AuditError::DiskFull { .. }) && in_memory_unchanged && replay_unchanged)
}

/// One immutable audit record stored as a JSONL line.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRecord {
    /// Format version for deterministic future migrations.
    pub format_version: String,
    /// Monotonic local journal sequence.
    pub sequence: u64,
    /// Previous record hash, or genesis marker for sequence 1.
    pub previous_hash: String,
    /// Redacted audit event payload.
    pub event: AuditEvent,
    /// Hash over sequence, previous hash, and event payload.
    pub record_hash: String,
}

/// Typed event categories used by policy, runtime, and future execution paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditEventKind {
    /// Runtime process started or stopped.
    RuntimeLifecycle,
    /// Configuration file or runtime mode event.
    Configuration,
    /// Policy decision or policy subsystem event.
    PolicyDecision,
    /// Strategy/opportunity generated an intent.
    IntentLifecycle,
    /// Execution planner event.
    ExecutionPlanning,
    /// Future connector submission event.
    ExecutionSubmission,
    /// Future connector result event.
    ExecutionResult,
    /// Balance, position, or state reconciliation event.
    Reconciliation,
    /// Security alert or guardrail denial.
    SecurityAlert,
}

/// Redacted audit event payload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEvent {
    /// Stable event id produced by caller.
    pub id: String,
    /// Event category.
    pub kind: AuditEventKind,
    /// Subsystem that produced the event.
    pub subsystem: String,
    /// Actor class, such as runtime, policy, operator, or connector.
    pub actor: String,
    /// Unix timestamp in milliseconds.
    pub occurred_at_unix_ms: u64,
    /// Human-readable summary. Must not contain secrets.
    pub message: String,
    /// Structured redacted metadata.
    pub metadata: BTreeMap<String, AuditValue>,
}

impl AuditEvent {
    /// Create a new event with current wall-clock timestamp.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        kind: AuditEventKind,
        subsystem: impl Into<String>,
        actor: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            subsystem: subsystem.into(),
            actor: actor.into(),
            occurred_at_unix_ms: unix_timestamp_ms(),
            message: message.into(),
            metadata: BTreeMap::new(),
        }
    }

    /// Attach one metadata key/value pair.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: AuditValue) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Validate deterministic event fields and secret redaction requirements.
    pub fn validate(&self) -> Result<(), AuditError> {
        let mut violations = Vec::new();

        if self.id.trim().is_empty() {
            violations.push(AuditViolation::new("AUDIT_EVENT_ID_REQUIRED"));
        }
        if self.subsystem.trim().is_empty() {
            violations.push(AuditViolation::new("AUDIT_SUBSYSTEM_REQUIRED"));
        }
        if self.actor.trim().is_empty() {
            violations.push(AuditViolation::new("AUDIT_ACTOR_REQUIRED"));
        }
        if self.message.trim().is_empty() {
            violations.push(AuditViolation::new("AUDIT_MESSAGE_REQUIRED"));
        }
        if contains_obvious_secret_assignment(&self.message) {
            violations.push(AuditViolation::new("AUDIT_MESSAGE_SECRET_LIKE"));
        }

        for (key, value) in &self.metadata {
            if key.trim().is_empty() {
                violations.push(AuditViolation::new("AUDIT_METADATA_KEY_REQUIRED"));
            }
            if sensitive_key_name(key) && !value.is_redacted() {
                violations.push(AuditViolation::new_owned(
                    "AUDIT_METADATA_SECRET_NOT_REDACTED",
                    format!("metadata key {key} must use AuditValue::Redacted"),
                ));
            }
            value.validate(&mut violations);
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(AuditError::ValidationFailed { violations })
        }
    }
}

/// Metadata value that can be safely serialized to the audit journal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum AuditValue {
    /// Non-secret text.
    Text(String),
    /// Signed integer.
    Integer(i64),
    /// Unsigned integer.
    Unsigned(u64),
    /// Basis points or fixed-point minor units should prefer integer values.
    Bool(bool),
    /// Explicit redaction marker for sensitive values.
    Redacted,
}

impl AuditValue {
    /// Return true for explicit redaction markers.
    #[must_use]
    pub const fn is_redacted(&self) -> bool {
        matches!(self, Self::Redacted)
    }

    fn validate(&self, violations: &mut Vec<AuditViolation>) {
        if let Self::Text(value) = self {
            if contains_obvious_secret_assignment(value) {
                violations.push(AuditViolation::new("AUDIT_METADATA_SECRET_LIKE"));
            }
        }
    }
}

/// Audit validation or integrity violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditViolation {
    code: &'static str,
    message: String,
}

impl AuditViolation {
    /// Create a violation with code as message.
    #[must_use]
    pub fn new(code: &'static str) -> Self {
        Self {
            code,
            message: code.to_owned(),
        }
    }

    /// Create a violation with an owned human-readable message.
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

/// Audit journal errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    /// Filesystem failure.
    Io { path: String, reason: String },
    /// Disk-full or no-space-left failure.
    DiskFull { path: String, reason: String },
    /// JSON serialization failure.
    Serialize { reason: String },
    /// JSON parsing failure.
    Deserialize { line: u64, reason: String },
    /// Record failed redaction or field validation.
    ValidationFailed { violations: Vec<AuditViolation> },
    /// Hash chain failed during replay.
    IntegrityViolation { line: u64, reason: String },
    /// Local validation harness failed before it could produce a report.
    ValidationHarnessFailed { reason: String },
    /// Sequence would overflow.
    SequenceOverflow,
}

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, reason } => write!(formatter, "audit I/O failed for {path}: {reason}"),
            Self::DiskFull { path, reason } => {
                write!(formatter, "audit disk-full failure for {path}: {reason}")
            }
            Self::Serialize { reason } => write!(formatter, "audit serialization failed: {reason}"),
            Self::Deserialize { line, reason } => {
                write!(
                    formatter,
                    "audit deserialize failed on line {line}: {reason}"
                )
            }
            Self::ValidationFailed { violations } => {
                write!(
                    formatter,
                    "audit validation failed with {} violation(s)",
                    violations.len()
                )
            }
            Self::IntegrityViolation { line, reason } => {
                write!(
                    formatter,
                    "audit integrity violation on line {line}: {reason}"
                )
            }
            Self::ValidationHarnessFailed { reason } => {
                write!(formatter, "audit validation harness failed: {reason}")
            }
            Self::SequenceOverflow => write!(formatter, "audit sequence overflow"),
        }
    }
}

impl std::error::Error for AuditError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayState {
    next_sequence: u64,
    previous_hash: String,
}

struct AuditJournalFileLock {
    path: PathBuf,
}

impl AuditJournalFileLock {
    fn acquire(journal_path: &Path) -> Result<Self, AuditError> {
        let path = journal_lock_path(journal_path);
        for _ in 0..AUDIT_LOCK_RETRY_COUNT {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    write!(
                        file,
                        "pid={} acquired_at_unix_ms={}",
                        std::process::id(),
                        unix_timestamp_ms()
                    )
                    .map_err(|source| AuditError::Io {
                        path: path.display().to_string(),
                        reason: source.to_string(),
                    })?;
                    file.sync_all().map_err(|source| AuditError::Io {
                        path: path.display().to_string(),
                        reason: source.to_string(),
                    })?;
                    return Ok(Self { path });
                }
                Err(source)
                    if matches!(
                        source.kind(),
                        ErrorKind::AlreadyExists | ErrorKind::PermissionDenied
                    ) =>
                {
                    thread::sleep(AUDIT_LOCK_RETRY_DELAY);
                }
                Err(source) => {
                    return Err(AuditError::Io {
                        path: path.display().to_string(),
                        reason: source.to_string(),
                    });
                }
            }
        }
        Err(AuditError::Io {
            path: path.display().to_string(),
            reason: "timed out waiting for audit journal lock".to_owned(),
        })
    }
}

impl Drop for AuditJournalFileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn audit_io_error(path: &Path, source: IoError) -> AuditError {
    let reason = source.to_string();
    if is_disk_full_error(&source, &reason) {
        AuditError::DiskFull {
            path: path.display().to_string(),
            reason,
        }
    } else {
        AuditError::Io {
            path: path.display().to_string(),
            reason,
        }
    }
}

fn is_disk_full_error(source: &IoError, reason: &str) -> bool {
    matches!(source.raw_os_error(), Some(28 | 39 | 112))
        || reason.contains("No space left")
        || reason.contains("There is not enough space")
        || reason.contains("disk full")
}

fn journal_lock_path(journal_path: &Path) -> PathBuf {
    let mut lock_path = journal_path.to_path_buf();
    let extension = journal_path
        .extension()
        .and_then(|value| value.to_str())
        .map_or_else(|| "lock".to_owned(), |value| format!("{value}.lock"));
    lock_path.set_extension(extension);
    lock_path
}

fn replay_journal(path: &Path) -> Result<ReplayState, AuditError> {
    let file = File::open(path).map_err(|source| AuditError::Io {
        path: path.display().to_string(),
        reason: source.to_string(),
    })?;
    let reader = BufReader::new(file);
    let mut expected_sequence = 1_u64;
    let mut previous_hash = AUDIT_GENESIS_HASH.to_owned();

    for (index, line) in reader.lines().enumerate() {
        let line_number = u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1);
        let line = line.map_err(|source| AuditError::Io {
            path: path.display().to_string(),
            reason: source.to_string(),
        })?;
        if line.trim().is_empty() {
            continue;
        }

        let record = serde_json::from_str::<AuditRecord>(&line).map_err(|source| {
            AuditError::Deserialize {
                line: line_number,
                reason: source.to_string(),
            }
        })?;
        validate_record(&record, expected_sequence, &previous_hash, line_number)?;
        previous_hash = record.record_hash;
        expected_sequence = expected_sequence
            .checked_add(1)
            .ok_or(AuditError::SequenceOverflow)?;
    }

    Ok(ReplayState {
        next_sequence: expected_sequence,
        previous_hash,
    })
}

fn validate_record(
    record: &AuditRecord,
    expected_sequence: u64,
    expected_previous_hash: &str,
    line_number: u64,
) -> Result<(), AuditError> {
    if record.format_version != AUDIT_JOURNAL_FORMAT_VERSION {
        return Err(AuditError::IntegrityViolation {
            line: line_number,
            reason: format!("unexpected format version {}", record.format_version),
        });
    }

    if record.sequence != expected_sequence {
        return Err(AuditError::IntegrityViolation {
            line: line_number,
            reason: format!(
                "sequence {} did not match expected {}",
                record.sequence, expected_sequence
            ),
        });
    }

    if record.previous_hash != expected_previous_hash {
        return Err(AuditError::IntegrityViolation {
            line: line_number,
            reason: "previous hash does not match replay state".to_owned(),
        });
    }

    record.event.validate()?;

    let expected_hash = compute_record_hash(record.sequence, &record.previous_hash, &record.event)?;
    if record.record_hash != expected_hash {
        return Err(AuditError::IntegrityViolation {
            line: line_number,
            reason: "record hash mismatch".to_owned(),
        });
    }

    Ok(())
}

#[derive(Serialize)]
struct HashableRecord<'a> {
    sequence: u64,
    previous_hash: &'a str,
    event: &'a AuditEvent,
}

fn compute_record_hash(
    sequence: u64,
    previous_hash: &str,
    event: &AuditEvent,
) -> Result<String, AuditError> {
    let hashable = HashableRecord {
        sequence,
        previous_hash,
        event,
    };
    let bytes = serde_json::to_vec(&hashable).map_err(|source| AuditError::Serialize {
        reason: source.to_string(),
    })?;
    let digest = Sha256::digest(bytes);
    Ok(to_lower_hex(&digest))
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

fn unix_timestamp_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        Err(_) => 0,
    }
}

fn sensitive_key_name(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    normalized.contains("secret")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("private_key")
        || normalized.contains("privatekey")
        || normalized.contains("seed_phrase")
        || normalized.contains("seedphrase")
        || normalized.contains("mnemonic")
        || normalized.ends_with("token")
        || normalized.contains("auth_token")
}

fn contains_obvious_secret_assignment(value: &str) -> bool {
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

fn validation_event(id: impl Into<String>) -> AuditEvent {
    AuditEvent::new(
        id,
        AuditEventKind::SecurityAlert,
        "audit",
        "validation-harness",
        "audit durability validation event",
    )
    .with_metadata("validation", AuditValue::Text("local-only".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{
        plan_audit_journal_retention, plan_audit_stale_lock_recheck,
        validate_audit_journal_durability, validate_disk_full_failure_probe,
        AppendOnlyAuditJournal, AuditAppendFault, AuditError, AuditEvent, AuditEventKind,
        AuditJournalFileMetadata, AuditLockFileMetadata, AuditRetentionPolicy,
        AuditStaleLockPolicy, AuditValue, AUDIT_DURABILITY_VALIDATION_VERSION, AUDIT_GENESIS_HASH,
    };
    use std::{env, fs, fs::OpenOptions, io::Write, process};

    fn temp_path(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "arb-agent-{name}-{}-{}.jsonl",
            process::id(),
            super::unix_timestamp_ms()
        ))
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "arb-agent-{name}-{}-{}",
            process::id(),
            super::unix_timestamp_ms()
        ))
    }

    fn event(id: &str) -> AuditEvent {
        AuditEvent::new(
            id,
            AuditEventKind::PolicyDecision,
            "policy",
            "test",
            "policy decision recorded",
        )
        .with_metadata("intent_id", AuditValue::Text("intent-1".to_owned()))
    }

    #[test]
    fn append_and_reopen_preserves_hash_chain() {
        let path = temp_path("append-reopen");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        assert_eq!(journal.next_sequence(), 1);
        assert_eq!(journal.previous_hash(), AUDIT_GENESIS_HASH);

        let first = journal
            .append_event(event("event-1"))
            .expect("append succeeds");
        assert_eq!(first.sequence, 1);
        assert_ne!(first.record_hash, AUDIT_GENESIS_HASH);

        let reopened = AppendOnlyAuditJournal::open(&path).expect("reopen succeeds");
        assert_eq!(reopened.next_sequence(), 2);
        assert_eq!(reopened.previous_hash(), first.record_hash);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn secret_named_metadata_must_be_redacted() {
        let path = temp_path("redaction");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let bad = event("event-secret")
            .with_metadata("api_key", AuditValue::Text("not-for-logs".to_owned()));
        let error = journal.append_event(bad).expect_err("secret key must fail");
        match error {
            AuditError::ValidationFailed { violations } => assert!(violations
                .iter()
                .any(|violation| violation.code() == "AUDIT_METADATA_SECRET_NOT_REDACTED")),
            other => panic!("unexpected error: {other:?}"),
        }
        let _ = fs::remove_file(path);
    }

    #[test]
    fn durability_validation_covers_crash_concurrency_and_filesystem_failures() {
        let workspace = temp_dir("audit-durability");
        let report = validate_audit_journal_durability(&workspace, 1_700_000_001_000)
            .expect("durability validation should complete");

        assert_eq!(
            report.validation_version,
            AUDIT_DURABILITY_VALIDATION_VERSION
        );
        assert!(report.append_replay_validated);
        assert!(report.truncated_replay_rejected);
        assert!(report.tamper_replay_rejected);
        assert!(report.concurrent_append_validated);
        assert!(report.filesystem_failure_validated);
        assert!(report.disk_full_failure_validated);
        assert_eq!(report.append_records, 2);
        assert_eq!(report.concurrent_records, 32);
        assert!(!report.live_network_used);
        assert!(!report.external_execution_performed);
        assert!(!report.production_ready);
        assert!(report
            .unresolved_blockers
            .iter()
            .any(|blocker| blocker.contains("deployment-host audit validation")));

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn durability_validation_refuses_existing_workspace() {
        let workspace = temp_dir("audit-durability-existing");
        fs::create_dir_all(&workspace).expect("workspace should be created");
        let error = validate_audit_journal_durability(&workspace, 1_700_000_001_000)
            .expect_err("existing workspace must fail closed");
        assert!(matches!(error, AuditError::ValidationHarnessFailed { .. }));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn disk_full_validation_keeps_journal_state_unchanged() {
        let workspace = temp_dir("audit-disk-full");
        fs::create_dir_all(&workspace).expect("workspace should be created");
        assert!(
            validate_disk_full_failure_probe(&workspace).expect("disk-full probe should complete")
        );
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn partial_jsonl_tail_is_rejected_on_replay() {
        let path = temp_path("partial-jsonl-tail");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        journal
            .append_event(event("before-partial-tail"))
            .expect("first event appends");

        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("journal opens for partial tail append");
        write!(file, "{{\"format_version\":\"audit-jsonl-hash-chain-v1\"")
            .expect("partial tail writes");
        file.flush().expect("partial tail flushes");
        file.sync_all().expect("partial tail syncs");

        assert!(matches!(
            AppendOnlyAuditJournal::open(&path),
            Err(AuditError::Deserialize { .. })
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn permission_failure_keeps_journal_state_unchanged() {
        let path = temp_path("permission-failure");
        let mut journal = AppendOnlyAuditJournal::open(&path).expect("journal opens");
        let first = journal
            .append_event(event("before-permission-failure"))
            .expect("first event appends");
        let next_sequence_before = journal.next_sequence();
        let previous_hash_before = journal.previous_hash().to_owned();

        let error = journal
            .append_event_with_fault(
                event("permission-denied"),
                AuditAppendFault::PermissionDeniedBeforeWrite,
            )
            .expect_err("permission failure must fail closed");

        assert!(matches!(error, AuditError::Io { .. }));
        assert_eq!(journal.next_sequence(), next_sequence_before);
        assert_eq!(journal.previous_hash(), previous_hash_before);

        let reopened = AppendOnlyAuditJournal::open(&path).expect("journal reopens");
        assert_eq!(reopened.next_sequence(), next_sequence_before);
        assert_eq!(reopened.previous_hash(), first.record_hash);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn retention_plan_marks_rotation_without_deleting_logs() {
        let policy = AuditRetentionPolicy {
            max_active_bytes: 100,
            max_archived_files: 2,
            retention_window_ms: 1_000,
        };
        let files = vec![
            AuditJournalFileMetadata {
                path: "audit.jsonl".to_owned(),
                size_bytes: 101,
                modified_at_unix_ms: 10_000,
                active: true,
            },
            AuditJournalFileMetadata {
                path: "audit.3.jsonl".to_owned(),
                size_bytes: 40,
                modified_at_unix_ms: 9_900,
                active: false,
            },
            AuditJournalFileMetadata {
                path: "audit.2.jsonl".to_owned(),
                size_bytes: 40,
                modified_at_unix_ms: 9_800,
                active: false,
            },
            AuditJournalFileMetadata {
                path: "audit.1.jsonl".to_owned(),
                size_bytes: 40,
                modified_at_unix_ms: 8_000,
                active: false,
            },
        ];

        let plan =
            plan_audit_journal_retention(&policy, &files, 10_000).expect("plan should build");
        assert!(plan.rotate_active);
        assert_eq!(plan.active_path, "audit.jsonl");
        assert_eq!(
            plan.retained_archives,
            vec!["audit.3.jsonl", "audit.2.jsonl"]
        );
        assert_eq!(plan.expired_archives, vec!["audit.1.jsonl"]);
        assert!(!plan.deletion_performed);
        assert!(!plan.filesystem_mutated);
        assert!(!plan.production_ready);
    }

    #[test]
    fn retention_plan_expires_over_count_without_filesystem_mutation() {
        let policy = AuditRetentionPolicy {
            max_active_bytes: 1_000,
            max_archived_files: 1,
            retention_window_ms: 10_000,
        };
        let files = vec![
            AuditJournalFileMetadata {
                path: "active.jsonl".to_owned(),
                size_bytes: 12,
                modified_at_unix_ms: 20_000,
                active: true,
            },
            AuditJournalFileMetadata {
                path: "newer.jsonl".to_owned(),
                size_bytes: 1,
                modified_at_unix_ms: 19_000,
                active: false,
            },
            AuditJournalFileMetadata {
                path: "older.jsonl".to_owned(),
                size_bytes: 1,
                modified_at_unix_ms: 18_000,
                active: false,
            },
        ];

        let plan =
            plan_audit_journal_retention(&policy, &files, 20_000).expect("plan should build");
        assert!(!plan.rotate_active);
        assert_eq!(plan.retained_archives, vec!["newer.jsonl"]);
        assert_eq!(plan.expired_archives, vec!["older.jsonl"]);
        assert!(!plan.deletion_performed);
        assert!(!plan.filesystem_mutated);
    }

    #[test]
    fn retention_plan_rejects_ambiguous_active_journals() {
        let policy = AuditRetentionPolicy {
            max_active_bytes: 100,
            max_archived_files: 1,
            retention_window_ms: 1_000,
        };
        let files = vec![
            AuditJournalFileMetadata {
                path: "active-a.jsonl".to_owned(),
                size_bytes: 1,
                modified_at_unix_ms: 1,
                active: true,
            },
            AuditJournalFileMetadata {
                path: "active-b.jsonl".to_owned(),
                size_bytes: 1,
                modified_at_unix_ms: 1,
                active: true,
            },
        ];

        let error = plan_audit_journal_retention(&policy, &files, 2)
            .expect_err("ambiguous active journals must fail closed");
        assert!(matches!(error, AuditError::ValidationHarnessFailed { .. }));
    }

    #[test]
    fn stale_lock_plan_marks_restart_recheck_without_removing_lock() {
        let policy = AuditStaleLockPolicy {
            stale_after_ms: 5_000,
        };
        let lock = AuditLockFileMetadata {
            path: "audit.jsonl.lock".to_owned(),
            owner_pid: 42,
            acquired_at_unix_ms: 10_000,
        };

        let plan =
            plan_audit_stale_lock_recheck(&policy, &lock, 16_000).expect("plan should build");

        assert_eq!(plan.lock_path, "audit.jsonl.lock");
        assert_eq!(plan.owner_pid, 42);
        assert_eq!(plan.lock_age_ms, 6_000);
        assert!(plan.stale);
        assert!(plan.service_manager_recheck_required);
        assert!(!plan.lock_removal_performed);
        assert!(!plan.filesystem_mutated);
        assert!(!plan.production_ready);
    }

    #[test]
    fn fresh_lock_plan_does_not_request_restart_recheck() {
        let policy = AuditStaleLockPolicy {
            stale_after_ms: 5_000,
        };
        let lock = AuditLockFileMetadata {
            path: "audit.jsonl.lock".to_owned(),
            owner_pid: 42,
            acquired_at_unix_ms: 10_000,
        };

        let plan =
            plan_audit_stale_lock_recheck(&policy, &lock, 14_999).expect("plan should build");

        assert_eq!(plan.lock_age_ms, 4_999);
        assert!(!plan.stale);
        assert!(!plan.service_manager_recheck_required);
        assert!(!plan.lock_removal_performed);
        assert!(!plan.filesystem_mutated);
    }

    #[test]
    fn stale_lock_plan_rejects_future_timestamp() {
        let policy = AuditStaleLockPolicy {
            stale_after_ms: 5_000,
        };
        let lock = AuditLockFileMetadata {
            path: "audit.jsonl.lock".to_owned(),
            owner_pid: 42,
            acquired_at_unix_ms: 20_000,
        };

        let error = plan_audit_stale_lock_recheck(&policy, &lock, 19_999)
            .expect_err("future timestamp must fail closed");
        assert!(matches!(error, AuditError::ValidationHarnessFailed { .. }));
    }
}
