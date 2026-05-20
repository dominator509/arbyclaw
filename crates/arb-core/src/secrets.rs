#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};
use std::{env, fmt};

/// Non-secret reference to sensitive material stored outside repository files.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum SecretRef {
    /// Load the secret from an environment variable name.
    Env { name: String },
    /// Load the secret from a future encrypted local keystore alias.
    Keystore { alias: String },
    /// Explicitly disabled secret reference for observe-only configuration.
    Disabled,
}

impl SecretRef {
    /// Validate that a secret reference is a reference only, not raw material.
    pub fn validate_reference(&self) -> Result<(), SecretStoreError> {
        match self {
            Self::Env { name } => validate_reference_name(name, "environment variable"),
            Self::Keystore { alias } => validate_reference_name(alias, "keystore alias"),
            Self::Disabled => Ok(()),
        }
    }

    /// Returns true when no secret source is configured.
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

fn validate_reference_name(value: &str, label: &'static str) -> Result<(), SecretStoreError> {
    if value.trim().is_empty() {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference cannot be empty"),
        });
    }

    if value.len() > 128 {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference is too long"),
        });
    }

    if value.chars().any(char::is_whitespace) {
        return Err(SecretStoreError::InvalidReference {
            reason: format!("{label} reference cannot contain whitespace"),
        });
    }

    Ok(())
}

/// Sensitive bytes loaded from an approved provider.
///
/// The debug representation is always redacted. Future phases should add
/// zeroization once the dependency policy is finalized.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretMaterial {
    bytes: Vec<u8>,
}

impl SecretMaterial {
    /// Create secret material from a string value loaded outside config files.
    #[must_use]
    pub fn from_string(value: String) -> Self {
        Self {
            bytes: value.into_bytes(),
        }
    }

    /// Returns true when the loaded secret is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Borrow the raw bytes inside a constrained caller.
    #[must_use]
    pub fn expose_for_constrained_use(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretMaterial")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

/// Boundary implemented by approved secret providers.
pub trait SecretProvider {
    /// Load secret material by reference only.
    fn load(&self, reference: &SecretRef) -> Result<SecretMaterial, SecretStoreError>;
}

/// Environment-variable provider for local development and CI test contexts.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvSecretProvider;

impl SecretProvider for EnvSecretProvider {
    fn load(&self, reference: &SecretRef) -> Result<SecretMaterial, SecretStoreError> {
        match reference {
            SecretRef::Env { name } => {
                reference.validate_reference()?;
                let value = env::var(name).map_err(|_| SecretStoreError::Unavailable {
                    reference: name.clone(),
                })?;
                let material = SecretMaterial::from_string(value);
                if material.is_empty() {
                    return Err(SecretStoreError::EmptySecret {
                        reference: name.clone(),
                    });
                }
                Ok(material)
            }
            SecretRef::Keystore { alias } => Err(SecretStoreError::ProviderUnavailable {
                reference: alias.clone(),
                provider: "encrypted keystore",
            }),
            SecretRef::Disabled => Err(SecretStoreError::Disabled),
        }
    }
}

/// Errors produced by secret-reference validation and loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretStoreError {
    /// Reference name is invalid.
    InvalidReference { reason: String },
    /// Referenced secret is not available in the provider.
    Unavailable { reference: String },
    /// Referenced secret loaded successfully but was empty.
    EmptySecret { reference: String },
    /// Secret provider has not been implemented or enabled.
    ProviderUnavailable {
        reference: String,
        provider: &'static str,
    },
    /// Secret access is intentionally disabled.
    Disabled,
}

impl fmt::Display for SecretStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReference { reason } => {
                write!(formatter, "invalid secret reference: {reason}")
            }
            Self::Unavailable { reference } => {
                write!(formatter, "secret reference is unavailable: {reference}")
            }
            Self::EmptySecret { reference } => write!(
                formatter,
                "secret reference resolved to an empty value: {reference}"
            ),
            Self::ProviderUnavailable {
                reference,
                provider,
            } => write!(
                formatter,
                "secret provider {provider} is unavailable for reference: {reference}"
            ),
            Self::Disabled => formatter.write_str("secret access is disabled"),
        }
    }
}

impl std::error::Error for SecretStoreError {}

#[cfg(test)]
mod tests {
    use super::{SecretMaterial, SecretRef};

    #[test]
    fn secret_material_debug_is_redacted() {
        let material = SecretMaterial::from_string("example-sensitive-value".to_owned());
        let debug = format!("{material:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("example-sensitive-value"));
    }

    #[test]
    fn environment_secret_reference_validates_name_only() {
        let reference = SecretRef::Env {
            name: "ARB_EXAMPLE_REFERENCE".to_owned(),
        };
        assert!(reference.validate_reference().is_ok());
    }

    #[test]
    fn empty_reference_is_rejected() {
        let reference = SecretRef::Keystore {
            alias: String::new(),
        };
        assert!(reference.validate_reference().is_err());
    }
}
