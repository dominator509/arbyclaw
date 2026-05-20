#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Genesis hash marker for the first append-only audit record.
pub const AUDIT_GENESIS_HASH: &str = "GENESIS";

/// Current audit journal format version.
pub const AUDIT_JOURNAL_FORMAT_VERSION: &str = "audit-jsonl-hash-chain-v1";

/// Append-only local audit journal with hash-chained JSONL records.
///
/// The journal is intentionally small and local-first for the single-binary
/// runtime. It is not a replacement for later SQLite WAL storage, external log
/// shipping, crash testing, filesystem permission hardening, or SIEM ingestion.
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
        event.validate()?;

        let sequence = self.next_sequence;
        let previous_hash = self.previous_hash.clone();
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

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| AuditError::Io {
                path: self.path.display().to_string(),
                reason: source.to_string(),
            })?;
        writeln!(file, "{line}").map_err(|source| AuditError::Io {
            path: self.path.display().to_string(),
            reason: source.to_string(),
        })?;
        file.flush().map_err(|source| AuditError::Io {
            path: self.path.display().to_string(),
            reason: source.to_string(),
        })?;

        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(AuditError::SequenceOverflow)?;
        self.previous_hash = record.record_hash.clone();

        Ok(record)
    }
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
    /// JSON serialization failure.
    Serialize { reason: String },
    /// JSON parsing failure.
    Deserialize { line: u64, reason: String },
    /// Record failed redaction or field validation.
    ValidationFailed { violations: Vec<AuditViolation> },
    /// Hash chain failed during replay.
    IntegrityViolation { line: u64, reason: String },
    /// Sequence would overflow.
    SequenceOverflow,
}

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, reason } => write!(formatter, "audit I/O failed for {path}: {reason}"),
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

#[cfg(test)]
mod tests {
    use super::{
        AppendOnlyAuditJournal, AuditError, AuditEvent, AuditEventKind, AuditValue,
        AUDIT_GENESIS_HASH,
    };
    use std::{env, fs, process};

    fn temp_path(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "arb-agent-{name}-{}-{}.jsonl",
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
}
