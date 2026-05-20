#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{RuntimeMode, SecretRef};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::Path};

/// Exact acknowledgement required before a config file may select live mode.
pub const LIVE_ACKNOWLEDGEMENT: &str = "I UNDERSTAND LIVE CRYPTO TRADING RISK";

/// Load and validate an agent config from a TOML file.
pub fn load_config_file(path: impl AsRef<Path>) -> Result<AgentConfig, ConfigError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| ConfigError::ReadFailed {
        path: path.display().to_string(),
        reason: source.to_string(),
    })?;
    AgentConfig::from_toml_str(&text)
}

/// Top-level non-secret runtime configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    /// Runtime lifecycle and mode-gate configuration.
    pub runtime: RuntimeConfig,
    /// Risk limits required before any execution phase can use this config.
    pub risk: RiskLimitsConfig,
    /// Venue and asset allowlists.
    pub venues: VenueAllowlistsConfig,
    /// Secret references and backend selection.
    pub secrets: SecretsConfig,
    /// Communication channel configuration.
    pub communication: CommunicationConfig,
    /// Audit configuration.
    pub audit: AuditConfig,
}

impl AgentConfig {
    /// Parse and validate config from TOML text.
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        let config = toml::from_str::<Self>(text).map_err(|source| ConfigError::ParseFailed {
            reason: source.to_string(),
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Validate deterministic mode gates and non-secret configuration rules.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut violations = Vec::new();

        if self.risk.max_single_trade_quote <= 0.0 {
            violations.push(ConfigViolation::new(
                "RISK_MAX_SINGLE_TRADE_NON_POSITIVE",
                "risk.max_single_trade_quote must be greater than zero",
            ));
        }

        if self.risk.max_daily_loss_quote < 0.0 {
            violations.push(ConfigViolation::new(
                "RISK_MAX_DAILY_LOSS_NEGATIVE",
                "risk.max_daily_loss_quote cannot be negative",
            ));
        }

        if self.risk.max_open_exposure_quote <= 0.0 {
            violations.push(ConfigViolation::new(
                "RISK_MAX_OPEN_EXPOSURE_NON_POSITIVE",
                "risk.max_open_exposure_quote must be greater than zero",
            ));
        }

        if self.risk.slippage_bps > 500 {
            violations.push(ConfigViolation::new(
                "RISK_SLIPPAGE_TOO_HIGH",
                "risk.slippage_bps cannot exceed 500 basis points in Phase 2",
            ));
        }

        if self.risk.gas_fee_cap_quote < 0.0 {
            violations.push(ConfigViolation::new(
                "RISK_GAS_CAP_NEGATIVE",
                "risk.gas_fee_cap_quote cannot be negative",
            ));
        }

        if self.venues.cex_allowlist.is_empty() && self.venues.dex_allowlist.is_empty() {
            violations.push(ConfigViolation::new(
                "VENUE_ALLOWLIST_EMPTY",
                "at least one CEX or DEX venue reference must be configured",
            ));
        }

        if self.venues.asset_allowlist.is_empty() {
            violations.push(ConfigViolation::new(
                "ASSET_ALLOWLIST_EMPTY",
                "at least one asset symbol must be configured",
            ));
        }

        if let Err(error) = self.secrets.exchange_credentials.validate_reference() {
            violations.push(ConfigViolation::new_owned(
                "SECRET_EXCHANGE_REFERENCE_INVALID",
                error.to_string(),
            ));
        }

        if let Err(error) = self.secrets.wallet_signer.validate_reference() {
            violations.push(ConfigViolation::new_owned(
                "SECRET_WALLET_REFERENCE_INVALID",
                error.to_string(),
            ));
        }

        if self.runtime.mode.permits_live_execution() {
            self.validate_live_mode(&mut violations);
        }

        if self.audit.enabled && self.audit.redact_secrets != Some(true) {
            violations.push(ConfigViolation::new(
                "AUDIT_REDACTION_REQUIRED",
                "audit.redact_secrets must be true when audit logging is enabled",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::ValidationFailed { violations })
        }
    }

    fn validate_live_mode(&self, violations: &mut Vec<ConfigViolation>) {
        if !self.runtime.live_execution_enabled {
            violations.push(ConfigViolation::new(
                "LIVE_MODE_NOT_ENABLED",
                "runtime.live_execution_enabled must be true for live-armed mode",
            ));
        }

        if self.runtime.operator_acknowledgement.as_deref() != Some(LIVE_ACKNOWLEDGEMENT) {
            violations.push(ConfigViolation::new(
                "LIVE_ACKNOWLEDGEMENT_MISSING",
                "runtime.operator_acknowledgement must match the required live-risk acknowledgement",
            ));
        }

        if self.runtime.allow_withdrawals {
            violations.push(ConfigViolation::new(
                "WITHDRAWALS_BLOCKED_IN_PHASE_2",
                "runtime.allow_withdrawals must remain false until custody and policy phases are externally validated",
            ));
        }

        if self.runtime.kill_switch_enabled != Some(true) {
            violations.push(ConfigViolation::new(
                "KILL_SWITCH_REQUIRED",
                "runtime.kill_switch_enabled must be true for live-armed mode",
            ));
        }

        if self.secrets.backend == SecretBackend::Disabled {
            violations.push(ConfigViolation::new(
                "SECRET_BACKEND_DISABLED_FOR_LIVE",
                "secrets.backend cannot be disabled for live-armed mode",
            ));
        }

        if self.secrets.exchange_credentials.is_disabled() {
            violations.push(ConfigViolation::new(
                "EXCHANGE_SECRET_REFERENCE_REQUIRED",
                "secrets.exchange_credentials must reference an approved provider for live-armed mode",
            ));
        }

        if self.secrets.wallet_signer.is_disabled() {
            violations.push(ConfigViolation::new(
                "WALLET_SECRET_REFERENCE_REQUIRED",
                "secrets.wallet_signer must reference an approved provider for live-armed mode",
            ));
        }
    }
}

/// Runtime lifecycle and mode gates.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    /// Selected runtime mode.
    pub mode: RuntimeMode,
    /// Explicit live execution switch. Mode alone is not sufficient.
    pub live_execution_enabled: bool,
    /// Operator acknowledgement string required for live mode.
    pub operator_acknowledgement: Option<String>,
    /// Emergency kill-switch posture.
    pub kill_switch_enabled: Option<bool>,
    /// Withdrawal behavior. Must remain false in Phase 2.
    pub allow_withdrawals: bool,
}

/// Configured risk caps.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RiskLimitsConfig {
    /// Maximum quote-currency amount per single trade.
    pub max_single_trade_quote: f64,
    /// Maximum quote-currency loss allowed per day.
    pub max_daily_loss_quote: f64,
    /// Maximum quote-currency open exposure.
    pub max_open_exposure_quote: f64,
    /// Maximum allowed slippage in basis points.
    pub slippage_bps: u16,
    /// Maximum gas/network-fee budget in quote-currency units.
    pub gas_fee_cap_quote: f64,
}

/// Venue and asset allowlists.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VenueAllowlistsConfig {
    /// Centralized exchange connector names.
    pub cex_allowlist: Vec<String>,
    /// Decentralized exchange or aggregator connector names.
    pub dex_allowlist: Vec<String>,
    /// Chain names allowed for future Web3 routes.
    pub chain_allowlist: Vec<String>,
    /// Asset symbols allowed by strategy.
    pub asset_allowlist: Vec<String>,
}

/// Secret backend selector and non-secret references.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecretsConfig {
    /// Selected secret backend.
    pub backend: SecretBackend,
    /// Exchange credential reference only.
    pub exchange_credentials: SecretRef,
    /// Wallet signer reference only.
    pub wallet_signer: SecretRef,
}

/// Approved secret backend kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecretBackend {
    /// No secret provider available.
    Disabled,
    /// Environment variables for local development only.
    Env,
    /// Future encrypted file or OS-keyring backed keystore.
    EncryptedKeystore,
}

/// Communication channel toggles and references.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommunicationConfig {
    /// Enable local CLI interaction.
    pub cli_enabled: bool,
    /// Notification channel identifiers, not credentials.
    pub notify_channels: Vec<String>,
}

/// Audit logging posture.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditConfig {
    /// Whether audit logging is configured as enabled.
    pub enabled: bool,
    /// Whether audit output must redact secrets.
    pub redact_secrets: Option<bool>,
}

/// One deterministic validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigViolation {
    code: &'static str,
    message: String,
}

impl ConfigViolation {
    /// Create a validation violation.
    #[must_use]
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_owned(),
        }
    }

    /// Create a validation violation with owned message text.
    #[must_use]
    pub fn new_owned(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// Stable violation code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Human-readable violation detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Config loading and validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Config file could not be read.
    ReadFailed { path: String, reason: String },
    /// Config TOML could not be parsed.
    ParseFailed { reason: String },
    /// Config parsed but failed validation.
    ValidationFailed { violations: Vec<ConfigViolation> },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed { path, reason } => {
                write!(formatter, "failed to read config file {path}: {reason}")
            }
            Self::ParseFailed { reason } => {
                write!(formatter, "failed to parse config file: {reason}")
            }
            Self::ValidationFailed { violations } => {
                writeln!(
                    formatter,
                    "config validation failed with {} violation(s):",
                    violations.len()
                )?;
                for violation in violations {
                    writeln!(formatter, "- {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::{AgentConfig, ConfigError, LIVE_ACKNOWLEDGEMENT};

    const OBSERVE_CONFIG: &str = r#"
[runtime]
mode = "observe"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 10.0
max_daily_loss_quote = 2.0
max_open_exposure_quote = 20.0
slippage_bps = 50
gas_fee_cap_quote = 1.0

[venues]
cex_allowlist = ["coinbase", "kraken"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC", "ETH", "USDC"]

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
    fn observe_config_validates_without_secrets() {
        let config =
            AgentConfig::from_toml_str(OBSERVE_CONFIG).expect("observe config should parse");
        assert!(!config.runtime.mode.permits_live_execution());
    }

    #[test]
    fn live_mode_requires_acknowledgement() {
        let text = OBSERVE_CONFIG
            .replace("mode = \"observe\"", "mode = \"live-armed\"")
            .replace(
                "live_execution_enabled = false",
                "live_execution_enabled = true",
            )
            .replace("backend = \"disabled\"", "backend = \"env\"")
            .replace(
                "exchange_credentials = { source = \"disabled\" }",
                "exchange_credentials = { source = \"env\", name = \"ARB_EXCHANGE_REFERENCE\" }",
            )
            .replace(
                "wallet_signer = { source = \"disabled\" }",
                "wallet_signer = { source = \"env\", name = \"ARB_WALLET_REFERENCE\" }",
            );

        let error = AgentConfig::from_toml_str(&text).expect_err("missing live ack should fail");
        match error {
            ConfigError::ValidationFailed { violations } => {
                assert!(violations
                    .iter()
                    .any(|violation| violation.code() == "LIVE_ACKNOWLEDGEMENT_MISSING"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn live_mode_can_pass_phase_two_mode_gate_when_references_exist() {
        let text = OBSERVE_CONFIG
            .replace("mode = \"observe\"", "mode = \"live-armed\"")
            .replace("live_execution_enabled = false", "live_execution_enabled = true")
            .replace("backend = \"disabled\"", "backend = \"env\"")
            .replace(
                "kill_switch_enabled = true",
                &format!(
                    "operator_acknowledgement = \"{LIVE_ACKNOWLEDGEMENT}\"\nkill_switch_enabled = true"
                ),
            )
            .replace(
                "exchange_credentials = { source = \"disabled\" }",
                "exchange_credentials = { source = \"env\", name = \"ARB_EXCHANGE_REFERENCE\" }",
            )
            .replace(
                "wallet_signer = { source = \"disabled\" }",
                "wallet_signer = { source = \"env\", name = \"ARB_WALLET_REFERENCE\" }",
            );

        let config =
            AgentConfig::from_toml_str(&text).expect("live gate should pass with references only");
        assert!(config.runtime.mode.permits_live_execution());
    }
}
