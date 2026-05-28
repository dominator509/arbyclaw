#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

/// Version marker for SQLite WAL durability validation records.
pub const SQLITE_WAL_DURABILITY_VERSION: &str = "phase20-sqlite-wal-durability-v1";

/// State checkpoint persisted by future durable stores.
///
/// This model intentionally stores only non-secret operational state. Secrets,
/// wallet keys, exchange credentials, seed phrases, and bearer tokens must never
/// be stored through this boundary.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateCheckpoint {
    /// Stable checkpoint key.
    pub key: String,
    /// Owning subsystem.
    pub subsystem: String,
    /// Non-secret serialized value or opaque non-secret pointer.
    pub value: String,
    /// Last update timestamp in Unix milliseconds.
    pub updated_at_unix_ms: u64,
}

impl StateCheckpoint {
    /// Validate checkpoint fields and secret-like values.
    pub fn validate(&self) -> Result<(), StateStoreError> {
        if self.key.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "checkpoint key is required".to_owned(),
            });
        }
        if self.subsystem.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "checkpoint subsystem is required".to_owned(),
            });
        }
        if contains_secret_like_content(&self.key) || contains_secret_like_content(&self.value) {
            return Err(StateStoreError::ValidationFailed {
                reason: "checkpoint contains secret-like content".to_owned(),
            });
        }
        Ok(())
    }
}

/// Storage abstraction for operational state.
pub trait StateStore {
    /// Persist or replace a checkpoint.
    fn put_checkpoint(&mut self, checkpoint: StateCheckpoint) -> Result<(), StateStoreError>;

    /// Retrieve a checkpoint by key.
    fn get_checkpoint(&self, key: &str) -> Result<Option<StateCheckpoint>, StateStoreError>;
}

/// SQLite WAL-backed state store for non-secret operational checkpoints.
///
/// This store is local and typed only. It does not encrypt data, store secrets,
/// execute trades, call networks, or replace the future custody/signer boundary.
#[derive(Debug)]
pub struct SqliteWalStateStore {
    connection: Connection,
    path: PathBuf,
}

/// Non-secret result of a SQLite WAL durability validation pass.
///
/// This report intentionally records outcomes only. It does not include local
/// filesystem paths, checkpoint values, database contents, secrets, dependency
/// graphs, or embedded artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteWalDurabilityReport {
    /// SQLite journal mode observed by the validation pass.
    pub journal_mode: String,
    /// SQLite synchronous mode observed by the validation pass.
    pub synchronous_mode: String,
    /// True when `PRAGMA integrity_check` returned `ok`.
    pub integrity_check_passed: bool,
    /// True when `PRAGMA wal_checkpoint(TRUNCATE)` completed without busy pages.
    pub wal_checkpoint_truncate_passed: bool,
    /// True when a fresh handle reopened the primary database and read the probe.
    pub reopen_read_check_passed: bool,
    /// True when a copied, checkpointed database reopened and read the probe.
    pub backup_restore_check_passed: bool,
    /// True when two local handles observed each other's non-secret checkpoints.
    pub multi_handle_check_passed: bool,
    /// Live execution is never performed by this validation boundary.
    pub live_execution_performed: bool,
    /// External network access is never performed by this validation boundary.
    pub external_network_used: bool,
    /// Secret material is never recorded by this validation boundary.
    pub secret_material_recorded: bool,
}

impl SqliteWalStateStore {
    /// Open or create a SQLite WAL-backed state store at `path`.
    ///
    /// The path is retained for operator diagnostics only and must not contain
    /// credentials or secret-bearing material.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StateStoreError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "state database path is required".to_owned(),
            });
        }
        let connection =
            Connection::open(path).map_err(|error| StateStoreError::BackendFailed {
                reason: format!("failed to open sqlite state store: {error}"),
            })?;
        let store = Self {
            connection,
            path: path.to_path_buf(),
        };
        store.initialize()?;
        Ok(store)
    }

    /// Filesystem path backing this store.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the current SQLite journal mode for validation evidence.
    pub fn journal_mode(&self) -> Result<String, StateStoreError> {
        self.connection
            .query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .map(|mode| mode.to_ascii_lowercase())
            .map_err(sqlite_backend_error("failed to read sqlite journal mode"))
    }

    /// Return the current SQLite synchronous mode for validation evidence.
    pub fn synchronous_mode(&self) -> Result<String, StateStoreError> {
        let mode = self
            .connection
            .query_row("PRAGMA synchronous", [], |row| row.get::<_, i64>(0))
            .map_err(sqlite_backend_error(
                "failed to read sqlite synchronous mode",
            ))?;
        Ok(match mode {
            0 => "OFF",
            1 => "NORMAL",
            2 => "FULL",
            3 => "EXTRA",
            _ => "UNKNOWN",
        }
        .to_owned())
    }

    /// Run SQLite's integrity check and fail closed unless it returns `ok`.
    pub fn integrity_check(&self) -> Result<(), StateStoreError> {
        let result = self
            .connection
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .map_err(sqlite_backend_error("failed to run sqlite integrity check"))?;
        if result == "ok" {
            Ok(())
        } else {
            Err(StateStoreError::BackendFailed {
                reason: format!("sqlite integrity check failed: {result}"),
            })
        }
    }

    /// Flush WAL pages into the main database and truncate the WAL file.
    pub fn wal_checkpoint_truncate(&self) -> Result<(), StateStoreError> {
        let (busy, _log_pages, _checkpointed_pages): (i64, i64, i64) = self
            .connection
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(sqlite_backend_error(
                "failed to run sqlite WAL checkpoint truncate",
            ))?;
        if busy == 0 {
            Ok(())
        } else {
            Err(StateStoreError::BackendFailed {
                reason: format!("sqlite WAL checkpoint was busy: {busy}"),
            })
        }
    }

    /// Validate local SQLite WAL durability without live trading or network use.
    ///
    /// The validation writes deterministic non-secret probe checkpoints, verifies
    /// two local handles can observe each other's writes, runs integrity and WAL
    /// checkpoint checks, reopens the primary database, copies the checkpointed
    /// main database to `backup_path`, and verifies the copied database can be
    /// reopened and read. `backup_path` must not already exist.
    pub fn validate_durability(
        &mut self,
        probe_key: &str,
        backup_path: impl AsRef<Path>,
        now_unix_ms: u64,
    ) -> Result<SqliteWalDurabilityReport, StateStoreError> {
        validate_checkpoint_key(probe_key)?;
        let backup_path = backup_path.as_ref();
        validate_backup_path(&self.path, backup_path)?;

        let journal_mode = self.journal_mode()?;
        if journal_mode != "wal" {
            return Err(StateStoreError::BackendFailed {
                reason: format!("sqlite journal mode is not WAL: {journal_mode}"),
            });
        }

        let synchronous_mode = self.synchronous_mode()?;
        if synchronous_mode != "FULL" {
            return Err(StateStoreError::BackendFailed {
                reason: format!("sqlite synchronous mode is not FULL: {synchronous_mode}"),
            });
        }

        let primary_checkpoint = StateCheckpoint {
            key: probe_key.to_owned(),
            subsystem: "state-durability".to_owned(),
            value: "sqlite-wal-durability-validation".to_owned(),
            updated_at_unix_ms: now_unix_ms,
        };
        self.put_checkpoint(primary_checkpoint.clone())?;

        let peer_key = format!("{probe_key}:peer");
        {
            let mut peer_store = Self::open(&self.path)?;
            peer_store.put_checkpoint(StateCheckpoint {
                key: peer_key.clone(),
                subsystem: "state-durability".to_owned(),
                value: "sqlite-wal-multi-handle-validation".to_owned(),
                updated_at_unix_ms: now_unix_ms.saturating_add(1),
            })?;
            let peer_read = peer_store.get_checkpoint(probe_key)?;
            if peer_read != Some(primary_checkpoint.clone()) {
                return Err(StateStoreError::BackendFailed {
                    reason: "sqlite peer handle could not read primary checkpoint".to_owned(),
                });
            }
        }
        let primary_read = self.get_checkpoint(&peer_key)?;
        if primary_read.is_none() {
            return Err(StateStoreError::BackendFailed {
                reason: "sqlite primary handle could not read peer checkpoint".to_owned(),
            });
        }

        self.integrity_check()?;
        self.wal_checkpoint_truncate()?;

        {
            let reopened = Self::open(&self.path)?;
            if reopened.get_checkpoint(probe_key)? != Some(primary_checkpoint.clone()) {
                return Err(StateStoreError::BackendFailed {
                    reason: "sqlite reopened primary could not read durability probe".to_owned(),
                });
            }
        }

        fs::copy(&self.path, backup_path).map_err(filesystem_backend_error(
            "failed to copy sqlite durability backup",
        ))?;
        {
            let restored = Self::open(backup_path)?;
            if restored.get_checkpoint(probe_key)? != Some(primary_checkpoint) {
                return Err(StateStoreError::BackendFailed {
                    reason: "sqlite restored backup could not read durability probe".to_owned(),
                });
            }
            restored.integrity_check()?;
        }

        Ok(SqliteWalDurabilityReport {
            journal_mode,
            synchronous_mode,
            integrity_check_passed: true,
            wal_checkpoint_truncate_passed: true,
            reopen_read_check_passed: true,
            backup_restore_check_passed: true,
            multi_handle_check_passed: true,
            live_execution_performed: false,
            external_network_used: false,
            secret_material_recorded: false,
        })
    }

    fn initialize(&self) -> Result<(), StateStoreError> {
        self.connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(sqlite_backend_error("failed to enable sqlite WAL mode"))?;
        self.connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(sqlite_backend_error(
                "failed to set sqlite synchronous mode",
            ))?;
        self.connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(sqlite_backend_error("failed to enable sqlite foreign keys"))?;
        self.connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(sqlite_backend_error("failed to set sqlite busy timeout"))?;
        self.connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS state_checkpoints (
                    key TEXT PRIMARY KEY NOT NULL,
                    subsystem TEXT NOT NULL,
                    value TEXT NOT NULL,
                    updated_at_unix_ms INTEGER NOT NULL
                );",
            )
            .map_err(sqlite_backend_error("failed to initialize state schema"))?;
        Ok(())
    }
}

impl StateStore for SqliteWalStateStore {
    fn put_checkpoint(&mut self, checkpoint: StateCheckpoint) -> Result<(), StateStoreError> {
        checkpoint.validate()?;
        let updated_at_unix_ms = i64::try_from(checkpoint.updated_at_unix_ms).map_err(|_| {
            StateStoreError::ValidationFailed {
                reason: "checkpoint timestamp exceeds sqlite integer range".to_owned(),
            }
        })?;
        self.connection
            .execute(
                "INSERT INTO state_checkpoints (key, subsystem, value, updated_at_unix_ms)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(key) DO UPDATE SET
                    subsystem = excluded.subsystem,
                    value = excluded.value,
                    updated_at_unix_ms = excluded.updated_at_unix_ms",
                params![
                    checkpoint.key,
                    checkpoint.subsystem,
                    checkpoint.value,
                    updated_at_unix_ms
                ],
            )
            .map_err(sqlite_backend_error("failed to write state checkpoint"))?;
        Ok(())
    }

    fn get_checkpoint(&self, key: &str) -> Result<Option<StateCheckpoint>, StateStoreError> {
        validate_checkpoint_key(key)?;
        self.connection
            .query_row(
                "SELECT key, subsystem, value, updated_at_unix_ms
                 FROM state_checkpoints
                 WHERE key = ?1",
                params![key],
                |row| {
                    let updated_at_unix_ms: i64 = row.get(3)?;
                    Ok(StateCheckpoint {
                        key: row.get(0)?,
                        subsystem: row.get(1)?,
                        value: row.get(2)?,
                        updated_at_unix_ms: u64::try_from(updated_at_unix_ms).map_err(|_| {
                            rusqlite::Error::IntegralValueOutOfRange(3, updated_at_unix_ms)
                        })?,
                    })
                },
            )
            .optional()
            .map_err(sqlite_backend_error("failed to read state checkpoint"))
    }
}

/// Non-production in-memory state store.
///
/// This exists for unit tests and early subsystem wiring only. It provides no
/// crash persistence, process isolation, encryption, or durability guarantee.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InMemoryStateStore {
    checkpoints: BTreeMap<String, StateCheckpoint>,
}

impl InMemoryStateStore {
    /// Create an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored checkpoints.
    #[must_use]
    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    /// Returns true if the store has no checkpoints.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }
}

impl StateStore for InMemoryStateStore {
    fn put_checkpoint(&mut self, checkpoint: StateCheckpoint) -> Result<(), StateStoreError> {
        checkpoint.validate()?;
        self.checkpoints.insert(checkpoint.key.clone(), checkpoint);
        Ok(())
    }

    fn get_checkpoint(&self, key: &str) -> Result<Option<StateCheckpoint>, StateStoreError> {
        validate_checkpoint_key(key)?;
        Ok(self.checkpoints.get(key).cloned())
    }
}

/// State-store errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateStoreError {
    /// Validation failure.
    ValidationFailed { reason: String },
    /// Durable store backend failure for future implementations.
    BackendFailed { reason: String },
}

impl fmt::Display for StateStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { reason } => {
                write!(formatter, "state validation failed: {reason}")
            }
            Self::BackendFailed { reason } => write!(formatter, "state backend failed: {reason}"),
        }
    }
}

impl std::error::Error for StateStoreError {}

fn contains_secret_like_content(value: &str) -> bool {
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

fn validate_checkpoint_key(key: &str) -> Result<(), StateStoreError> {
    if key.trim().is_empty() {
        return Err(StateStoreError::ValidationFailed {
            reason: "checkpoint key is required".to_owned(),
        });
    }
    if contains_secret_like_content(key) {
        return Err(StateStoreError::ValidationFailed {
            reason: "checkpoint key contains secret-like content".to_owned(),
        });
    }
    Ok(())
}

fn validate_backup_path(primary_path: &Path, backup_path: &Path) -> Result<(), StateStoreError> {
    if backup_path.as_os_str().is_empty() {
        return Err(StateStoreError::ValidationFailed {
            reason: "sqlite durability backup path is required".to_owned(),
        });
    }
    if backup_path == primary_path {
        return Err(StateStoreError::ValidationFailed {
            reason: "sqlite durability backup path must differ from primary path".to_owned(),
        });
    }
    if backup_path.exists() {
        return Err(StateStoreError::ValidationFailed {
            reason: "sqlite durability backup path already exists".to_owned(),
        });
    }
    if contains_secret_like_content(&backup_path.display().to_string()) {
        return Err(StateStoreError::ValidationFailed {
            reason: "sqlite durability backup path contains secret-like content".to_owned(),
        });
    }
    Ok(())
}

fn sqlite_backend_error(context: &'static str) -> impl FnOnce(rusqlite::Error) -> StateStoreError {
    move |error| StateStoreError::BackendFailed {
        reason: format!("{context}: {error}"),
    }
}

fn filesystem_backend_error(
    context: &'static str,
) -> impl FnOnce(std::io::Error) -> StateStoreError {
    move |error| StateStoreError::BackendFailed {
        reason: format!("{context}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InMemoryStateStore, SqliteWalStateStore, StateCheckpoint, StateStore, StateStoreError,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn in_memory_store_round_trips_checkpoint() {
        let mut store = InMemoryStateStore::new();
        let checkpoint = StateCheckpoint {
            key: "policy:last-intent".to_owned(),
            subsystem: "policy".to_owned(),
            value: "intent-001".to_owned(),
            updated_at_unix_ms: 1,
        };
        store
            .put_checkpoint(checkpoint.clone())
            .expect("put succeeds");
        assert_eq!(store.len(), 1);
        assert_eq!(
            store
                .get_checkpoint("policy:last-intent")
                .expect("get succeeds"),
            Some(checkpoint)
        );
    }

    #[test]
    fn secret_like_checkpoint_is_rejected() {
        let mut store = InMemoryStateStore::new();
        let checkpoint = StateCheckpoint {
            key: "runtime:last".to_owned(),
            subsystem: "runtime".to_owned(),
            value: "api_key=x".to_owned(),
            updated_at_unix_ms: 1,
        };
        let error = store
            .put_checkpoint(checkpoint)
            .expect_err("secret-like state should fail");
        assert!(matches!(error, StateStoreError::ValidationFailed { .. }));
    }

    #[test]
    fn sqlite_wal_store_round_trips_checkpoint_across_reopen() {
        let path = unique_state_path("round-trip");
        let checkpoint = StateCheckpoint {
            key: "runtime:last-safe-mode".to_owned(),
            subsystem: "runtime".to_owned(),
            value: "paper".to_owned(),
            updated_at_unix_ms: 42,
        };
        {
            let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");
            assert_eq!(store.path(), path.as_path());
            store
                .put_checkpoint(checkpoint.clone())
                .expect("checkpoint persists");
        }
        {
            let store = SqliteWalStateStore::open(&path).expect("sqlite store reopens");
            assert_eq!(
                store
                    .get_checkpoint("runtime:last-safe-mode")
                    .expect("checkpoint reads"),
                Some(checkpoint)
            );
        }
        cleanup_state_files(&path);
    }

    #[test]
    fn sqlite_wal_store_replaces_checkpoint() {
        let path = unique_state_path("replace");
        let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");
        store
            .put_checkpoint(StateCheckpoint {
                key: "planner:last-plan".to_owned(),
                subsystem: "planner".to_owned(),
                value: "plan-001".to_owned(),
                updated_at_unix_ms: 1,
            })
            .expect("initial checkpoint persists");
        store
            .put_checkpoint(StateCheckpoint {
                key: "planner:last-plan".to_owned(),
                subsystem: "planner".to_owned(),
                value: "plan-002".to_owned(),
                updated_at_unix_ms: 2,
            })
            .expect("replacement checkpoint persists");

        let checkpoint = store
            .get_checkpoint("planner:last-plan")
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        assert_eq!(checkpoint.value, "plan-002");
        assert_eq!(checkpoint.updated_at_unix_ms, 2);
        drop(store);
        cleanup_state_files(&path);
    }

    #[test]
    fn sqlite_wal_store_rejects_secret_like_checkpoint() {
        let path = unique_state_path("secret-reject");
        let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");
        let error = store
            .put_checkpoint(StateCheckpoint {
                key: "runtime:last".to_owned(),
                subsystem: "runtime".to_owned(),
                value: "bearer token".to_owned(),
                updated_at_unix_ms: 1,
            })
            .expect_err("secret-like value is rejected");
        assert!(matches!(error, StateStoreError::ValidationFailed { .. }));
        drop(store);
        cleanup_state_files(&path);
    }

    #[test]
    fn sqlite_wal_durability_validation_covers_integrity_checkpoint_reopen_and_backup() {
        let path = unique_state_path("durability");
        let backup_path = unique_state_path("durability-backup");
        let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");

        let report = store
            .validate_durability("state:durability-probe", &backup_path, 100)
            .expect("durability validation passes");

        assert_eq!(report.journal_mode, "wal");
        assert_eq!(report.synchronous_mode, "FULL");
        assert!(report.integrity_check_passed);
        assert!(report.wal_checkpoint_truncate_passed);
        assert!(report.reopen_read_check_passed);
        assert!(report.backup_restore_check_passed);
        assert!(report.multi_handle_check_passed);
        assert!(!report.live_execution_performed);
        assert!(!report.external_network_used);
        assert!(!report.secret_material_recorded);

        let restored = SqliteWalStateStore::open(&backup_path).expect("backup reopens");
        let checkpoint = restored
            .get_checkpoint("state:durability-probe")
            .expect("backup checkpoint reads")
            .expect("backup checkpoint exists");
        assert_eq!(checkpoint.value, "sqlite-wal-durability-validation");

        drop(store);
        cleanup_state_files(&path);
        cleanup_state_files(&backup_path);
    }

    #[test]
    fn sqlite_wal_durability_validation_rejects_existing_backup_path() {
        let path = unique_state_path("durability-existing-backup");
        let backup_path = unique_state_path("durability-existing-backup-copy");
        fs::write(&backup_path, b"existing").expect("existing backup fixture writes");
        let mut store = SqliteWalStateStore::open(&path).expect("sqlite store opens");

        let error = store
            .validate_durability("state:durability-probe", &backup_path, 100)
            .expect_err("existing backup path should fail closed");

        assert!(matches!(error, StateStoreError::ValidationFailed { .. }));
        drop(store);
        cleanup_state_files(&path);
        cleanup_state_files(&backup_path);
    }

    fn unique_state_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "arbyclaw-{label}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup_state_files(path: &PathBuf) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
