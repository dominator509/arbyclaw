#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

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
///
/// Phase 4 only provides the trait and an in-memory implementation for tests and
/// local scaffolding. Production persistence must be added through a future
/// SQLite WAL-backed implementation and must be externally validated.
pub trait StateStore {
    /// Persist or replace a checkpoint.
    fn put_checkpoint(&mut self, checkpoint: StateCheckpoint) -> Result<(), StateStoreError>;

    /// Retrieve a checkpoint by key.
    fn get_checkpoint(&self, key: &str) -> Result<Option<StateCheckpoint>, StateStoreError>;
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
        if key.trim().is_empty() {
            return Err(StateStoreError::ValidationFailed {
                reason: "checkpoint key is required".to_owned(),
            });
        }
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

#[cfg(test)]
mod tests {
    use super::{InMemoryStateStore, StateCheckpoint, StateStore, StateStoreError};

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
}
