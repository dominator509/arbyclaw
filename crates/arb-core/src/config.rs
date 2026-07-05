#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{RuntimeMode, SecretRef};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::Path};

/// Stable config schema version for local compatibility checks.
pub const CONFIG_SCHEMA_VERSION: &str = "phase-2-config-v1";

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

/// Local config migration status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMigrationStatus {
    /// Input already matched the current schema and validated.
    AlreadyCurrent,
    /// Input was migrated from a known local legacy shape and validated.
    Migrated,
}

/// Local runtime config reload validation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeConfigReloadStatus {
    /// Reload was validated for local review.
    ReadyForLocalReview,
    /// Reload is unsafe or incomplete.
    Blocked,
}

/// Local, non-secret runtime config reload validation request.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigReloadValidationRequest {
    /// Stable non-secret reload validation id.
    pub reload_id: String,
    /// Initially loaded config.
    pub initial_config: AgentConfig,
    /// Reloaded config.
    pub reloaded_config: AgentConfig,
    /// Whether the validator performed service-manager actions. Must remain false.
    pub service_manager_action_performed: bool,
    /// Whether the validator loaded secret material. Must remain false.
    pub secret_material_loaded: bool,
    /// Whether the validator submitted to an external adapter. Must remain false.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Must remain false.
    pub live_execution_performed: bool,
    /// Whether this validation claims production readiness. Must remain false.
    pub production_ready_claimed: bool,
}

/// Local, non-secret runtime config reload validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigReloadValidationReport {
    /// Stable non-secret reload validation id.
    pub reload_id: String,
    /// Reload validation status.
    pub status: RuntimeConfigReloadStatus,
    /// Runtime mode before reload.
    pub initial_mode: RuntimeMode,
    /// Runtime mode after reload.
    pub reloaded_mode: RuntimeMode,
    /// Whether the initial config is non-live.
    pub initial_mode_safe: bool,
    /// Whether the reloaded config is non-live.
    pub reloaded_mode_safe: bool,
    /// Whether at least one local config field changed.
    pub reload_change_detected: bool,
    /// Whether the CEX allowlist changed.
    pub cex_allowlist_changed: bool,
    /// Whether the asset allowlist changed.
    pub asset_allowlist_changed: bool,
    /// Initial CEX allowlist count.
    pub initial_cex_venue_count: usize,
    /// Reloaded CEX allowlist count.
    pub reloaded_cex_venue_count: usize,
    /// Initial asset allowlist count.
    pub initial_asset_count: usize,
    /// Reloaded asset allowlist count.
    pub reloaded_asset_count: usize,
    /// Whether the validator performed service-manager actions. Always false here.
    pub service_manager_action_performed: bool,
    /// Whether the validator loaded secret material. Always false here.
    pub secret_material_loaded: bool,
    /// Whether the validator submitted to an external adapter. Always false here.
    pub external_submission_performed: bool,
    /// Whether live execution was performed. Always false here.
    pub live_execution_performed: bool,
    /// Whether this validation approves production readiness. Always false here.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Local config migration report.
///
/// This records schema compatibility actions only. It does not read secret
/// material, call providers, perform live execution, or claim readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMigrationReport {
    /// Schema version after migration.
    pub target_schema_version: String,
    /// Migration status.
    pub status: ConfigMigrationStatus,
    /// Sanitized action codes applied while migrating.
    pub action_codes: Vec<String>,
    /// Migrated config text in the current schema.
    pub migrated_toml: String,
    /// Validated migrated/current config.
    pub config: AgentConfig,
    /// Whether live execution was enabled by the resulting config.
    pub live_execution_enabled: bool,
    /// Whether secret material was loaded. Always false here.
    pub secret_material_loaded: bool,
    /// Production readiness is never claimed by config migration.
    pub production_ready: bool,
}

/// Migrate a local non-secret config TOML string to the current schema.
///
/// Known legacy aliases are upgraded:
/// - `[markets]` becomes `[venues]`
/// - `[notifications]` becomes `[communication]`
/// - legacy venue allowlist field names under `[venues]` are renamed
/// - missing `risk.gas_fee_cap_quote` is filled with `0.0`
/// - missing `[secrets]` is filled with disabled secret references
pub fn migrate_config_toml_to_current(text: &str) -> Result<ConfigMigrationReport, ConfigError> {
    if let Ok(config) = AgentConfig::from_toml_str(text) {
        let migrated_toml =
            toml::to_string(&config).map_err(|source| ConfigError::ParseFailed {
                reason: format!("failed to serialize current config: {source}"),
            })?;
        return Ok(ConfigMigrationReport {
            target_schema_version: CONFIG_SCHEMA_VERSION.to_owned(),
            status: ConfigMigrationStatus::AlreadyCurrent,
            action_codes: Vec::new(),
            migrated_toml,
            live_execution_enabled: config.runtime.mode.permits_live_execution(),
            secret_material_loaded: false,
            production_ready: false,
            config,
        });
    }

    let mut value = text
        .parse::<toml::Value>()
        .map_err(|source| ConfigError::ParseFailed {
            reason: source.to_string(),
        })?;
    let table = value
        .as_table_mut()
        .ok_or_else(|| ConfigError::ParseFailed {
            reason: "config root must be a TOML table".to_owned(),
        })?;
    let mut action_codes = Vec::new();

    if let Some(markets) = table.remove("markets") {
        let venues = migrate_markets_to_venues(markets)?;
        table.insert("venues".to_owned(), venues);
        action_codes.push("CONFIG_MIGRATED_MARKETS_TO_VENUES".to_owned());
    }

    if let Some(notifications) = table.remove("notifications") {
        let communication = migrate_notifications_to_communication(notifications)?;
        table.insert("communication".to_owned(), communication);
        action_codes.push("CONFIG_MIGRATED_NOTIFICATIONS_TO_COMMUNICATION".to_owned());
    }

    if migrate_venue_field_aliases(table)? {
        action_codes.push("CONFIG_MIGRATED_VENUE_FIELD_ALIASES".to_owned());
    }

    if ensure_risk_gas_cap(table)? {
        action_codes.push("CONFIG_DEFAULTED_RISK_GAS_FEE_CAP".to_owned());
    }

    if ensure_disabled_secrets(table) {
        action_codes.push("CONFIG_DEFAULTED_DISABLED_SECRET_REFERENCES".to_owned());
    }

    if ensure_communication(table) {
        action_codes.push("CONFIG_DEFAULTED_COMMUNICATION".to_owned());
    }

    if ensure_audit(table) {
        action_codes.push("CONFIG_DEFAULTED_AUDIT".to_owned());
    }

    let migrated_toml = toml::to_string(&value).map_err(|source| ConfigError::ParseFailed {
        reason: format!("failed to serialize migrated config: {source}"),
    })?;
    let config = AgentConfig::from_toml_str(&migrated_toml)?;

    Ok(ConfigMigrationReport {
        target_schema_version: CONFIG_SCHEMA_VERSION.to_owned(),
        status: ConfigMigrationStatus::Migrated,
        action_codes,
        migrated_toml,
        live_execution_enabled: config.runtime.mode.permits_live_execution(),
        secret_material_loaded: false,
        production_ready: false,
        config,
    })
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

/// Validate a local runtime config reload without supervising services.
pub fn validate_runtime_config_reload(
    request: RuntimeConfigReloadValidationRequest,
) -> Result<RuntimeConfigReloadValidationReport, ConfigError> {
    request.initial_config.validate()?;
    request.reloaded_config.validate()?;

    let mut violation_codes = Vec::new();
    if request.reload_id.trim().is_empty() {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_ID_REQUIRED".to_owned());
    }
    let initial_mode_safe = !request.initial_config.runtime.mode.permits_live_execution();
    let reloaded_mode_safe = !request
        .reloaded_config
        .runtime
        .mode
        .permits_live_execution();
    let cex_allowlist_changed =
        request.initial_config.venues.cex_allowlist != request.reloaded_config.venues.cex_allowlist;
    let asset_allowlist_changed = request.initial_config.venues.asset_allowlist
        != request.reloaded_config.venues.asset_allowlist;
    let reload_change_detected = request.initial_config.runtime.mode
        != request.reloaded_config.runtime.mode
        || cex_allowlist_changed
        || asset_allowlist_changed
        || request.initial_config.risk != request.reloaded_config.risk;

    if !initial_mode_safe {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_INITIAL_MODE_LIVE".to_owned());
    }
    if !reloaded_mode_safe {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_RELOADED_MODE_LIVE".to_owned());
    }
    if !reload_change_detected {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_NO_CHANGE_DETECTED".to_owned());
    }
    if request.service_manager_action_performed {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_SERVICE_MANAGER_ACTION".to_owned());
    }
    if request.secret_material_loaded {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_SECRET_MATERIAL_LOADED".to_owned());
    }
    if request.external_submission_performed {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_EXTERNAL_SUBMISSION".to_owned());
    }
    if request.live_execution_performed {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_LIVE_EXECUTION".to_owned());
    }
    if request.production_ready_claimed {
        violation_codes.push("RUNTIME_CONFIG_RELOAD_PRODUCTION_READY_CLAIMED".to_owned());
    }

    Ok(RuntimeConfigReloadValidationReport {
        reload_id: request.reload_id,
        status: if violation_codes.is_empty() {
            RuntimeConfigReloadStatus::ReadyForLocalReview
        } else {
            RuntimeConfigReloadStatus::Blocked
        },
        initial_mode: request.initial_config.runtime.mode,
        reloaded_mode: request.reloaded_config.runtime.mode,
        initial_mode_safe,
        reloaded_mode_safe,
        reload_change_detected,
        cex_allowlist_changed,
        asset_allowlist_changed,
        initial_cex_venue_count: request.initial_config.venues.cex_allowlist.len(),
        reloaded_cex_venue_count: request.reloaded_config.venues.cex_allowlist.len(),
        initial_asset_count: request.initial_config.venues.asset_allowlist.len(),
        reloaded_asset_count: request.reloaded_config.venues.asset_allowlist.len(),
        service_manager_action_performed: request.service_manager_action_performed,
        secret_material_loaded: request.secret_material_loaded,
        external_submission_performed: request.external_submission_performed,
        live_execution_performed: request.live_execution_performed,
        production_ready: false,
        violation_codes,
    })
}

fn migrate_markets_to_venues(markets: toml::Value) -> Result<toml::Value, ConfigError> {
    let markets = markets.as_table().ok_or_else(|| ConfigError::ParseFailed {
        reason: "legacy markets section must be a table".to_owned(),
    })?;
    let mut venues = toml::map::Map::new();
    venues.insert(
        "cex_allowlist".to_owned(),
        markets
            .get("allowed_exchanges")
            .cloned()
            .unwrap_or_else(|| toml::Value::Array(Vec::new())),
    );
    venues.insert(
        "dex_allowlist".to_owned(),
        markets
            .get("allowed_dexes")
            .cloned()
            .unwrap_or_else(|| toml::Value::Array(Vec::new())),
    );
    venues.insert(
        "chain_allowlist".to_owned(),
        markets
            .get("allowed_chains")
            .cloned()
            .unwrap_or_else(|| toml::Value::Array(Vec::new())),
    );
    venues.insert(
        "asset_allowlist".to_owned(),
        markets
            .get("allowed_assets")
            .cloned()
            .unwrap_or_else(|| toml::Value::Array(Vec::new())),
    );
    Ok(toml::Value::Table(venues))
}

fn migrate_notifications_to_communication(
    notifications: toml::Value,
) -> Result<toml::Value, ConfigError> {
    let notifications = notifications
        .as_table()
        .ok_or_else(|| ConfigError::ParseFailed {
            reason: "legacy notifications section must be a table".to_owned(),
        })?;
    let mut communication = toml::map::Map::new();
    communication.insert("cli_enabled".to_owned(), toml::Value::Boolean(true));
    communication.insert(
        "notify_channels".to_owned(),
        notifications
            .get("notify_channels")
            .cloned()
            .unwrap_or_else(|| toml::Value::Array(Vec::new())),
    );
    Ok(toml::Value::Table(communication))
}

fn migrate_venue_field_aliases(
    table: &mut toml::map::Map<String, toml::Value>,
) -> Result<bool, ConfigError> {
    let Some(venues) = table.get_mut("venues") else {
        return Ok(false);
    };
    let venues = venues
        .as_table_mut()
        .ok_or_else(|| ConfigError::ParseFailed {
            reason: "venues section must be a table".to_owned(),
        })?;
    let mut migrated = false;
    for (legacy, current) in [
        ("allowed_exchanges", "cex_allowlist"),
        ("allowed_dexes", "dex_allowlist"),
        ("allowed_chains", "chain_allowlist"),
        ("allowed_assets", "asset_allowlist"),
    ] {
        if let Some(value) = venues.remove(legacy) {
            if venues.contains_key(current) {
                return Err(ConfigError::ParseFailed {
                    reason: format!(
                        "venues section cannot contain both legacy field {legacy} and current field {current}"
                    ),
                });
            }
            venues.insert(current.to_owned(), value);
            migrated = true;
        }
    }
    Ok(migrated)
}

fn ensure_risk_gas_cap(
    table: &mut toml::map::Map<String, toml::Value>,
) -> Result<bool, ConfigError> {
    let risk = table
        .get_mut("risk")
        .and_then(toml::Value::as_table_mut)
        .ok_or_else(|| ConfigError::ParseFailed {
            reason: "config risk section is required for migration".to_owned(),
        })?;
    if risk.contains_key("gas_fee_cap_quote") {
        return Ok(false);
    }
    risk.insert("gas_fee_cap_quote".to_owned(), toml::Value::Float(0.0));
    Ok(true)
}

fn ensure_disabled_secrets(table: &mut toml::map::Map<String, toml::Value>) -> bool {
    if table.contains_key("secrets") {
        return false;
    }
    let mut secrets = toml::map::Map::new();
    secrets.insert(
        "backend".to_owned(),
        toml::Value::String("disabled".to_owned()),
    );
    secrets.insert(
        "exchange_credentials".to_owned(),
        disabled_secret_reference(),
    );
    secrets.insert("wallet_signer".to_owned(), disabled_secret_reference());
    table.insert("secrets".to_owned(), toml::Value::Table(secrets));
    true
}

fn disabled_secret_reference() -> toml::Value {
    let mut reference = toml::map::Map::new();
    reference.insert(
        "source".to_owned(),
        toml::Value::String("disabled".to_owned()),
    );
    toml::Value::Table(reference)
}

fn ensure_communication(table: &mut toml::map::Map<String, toml::Value>) -> bool {
    if table.contains_key("communication") {
        return false;
    }
    let mut communication = toml::map::Map::new();
    communication.insert("cli_enabled".to_owned(), toml::Value::Boolean(true));
    communication.insert("notify_channels".to_owned(), toml::Value::Array(Vec::new()));
    table.insert(
        "communication".to_owned(),
        toml::Value::Table(communication),
    );
    true
}

fn ensure_audit(table: &mut toml::map::Map<String, toml::Value>) -> bool {
    if table.contains_key("audit") {
        return false;
    }
    let mut audit = toml::map::Map::new();
    audit.insert("enabled".to_owned(), toml::Value::Boolean(true));
    audit.insert("redact_secrets".to_owned(), toml::Value::Boolean(true));
    table.insert("audit".to_owned(), toml::Value::Table(audit));
    true
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
    use super::{
        migrate_config_toml_to_current, validate_runtime_config_reload, AgentConfig, ConfigError,
        ConfigMigrationStatus, RuntimeConfigReloadStatus, RuntimeConfigReloadValidationRequest,
        LIVE_ACKNOWLEDGEMENT,
    };

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
    fn config_migration_reports_already_current_schema_without_side_effects() {
        let report =
            migrate_config_toml_to_current(OBSERVE_CONFIG).expect("current config should migrate");

        assert_eq!(report.status, ConfigMigrationStatus::AlreadyCurrent);
        assert!(report.action_codes.is_empty());
        assert!(!report.live_execution_enabled);
        assert!(!report.secret_material_loaded);
        assert!(!report.production_ready);
        assert!(report.migrated_toml.contains("[runtime]"));
    }

    #[test]
    fn runtime_config_reload_accepts_local_safe_config_change() {
        let report = validate_runtime_config_reload(runtime_reload_request(false, true))
            .expect("local config reload should validate");

        assert_eq!(
            report.status,
            RuntimeConfigReloadStatus::ReadyForLocalReview
        );
        assert!(report.initial_mode_safe);
        assert!(report.reloaded_mode_safe);
        assert!(report.reload_change_detected);
        assert!(report.cex_allowlist_changed);
        assert!(report.asset_allowlist_changed);
        assert_eq!(report.initial_cex_venue_count, 2);
        assert_eq!(report.reloaded_cex_venue_count, 3);
        assert_eq!(report.initial_asset_count, 3);
        assert_eq!(report.reloaded_asset_count, 4);
        assert!(!report.service_manager_action_performed);
        assert!(!report.secret_material_loaded);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn runtime_config_reload_blocks_unchanged_config() {
        let report = validate_runtime_config_reload(runtime_reload_request(false, false))
            .expect("unchanged local config reload should produce blocked report");

        assert_eq!(report.status, RuntimeConfigReloadStatus::Blocked);
        assert!(!report.reload_change_detected);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "RUNTIME_CONFIG_RELOAD_NO_CHANGE_DETECTED"));
        assert!(!report.production_ready);
    }

    #[test]
    fn runtime_config_reload_fails_closed_on_side_effect_claims() {
        let report = validate_runtime_config_reload(runtime_reload_request(true, true))
            .expect("side-effect local config reload should produce blocked report");

        assert_eq!(report.status, RuntimeConfigReloadStatus::Blocked);
        assert!(report.service_manager_action_performed);
        assert!(report.secret_material_loaded);
        assert!(report.external_submission_performed);
        assert!(report.live_execution_performed);
        assert!(!report.production_ready);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "RUNTIME_CONFIG_RELOAD_SERVICE_MANAGER_ACTION"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "RUNTIME_CONFIG_RELOAD_SECRET_MATERIAL_LOADED"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "RUNTIME_CONFIG_RELOAD_LIVE_EXECUTION"));
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "RUNTIME_CONFIG_RELOAD_PRODUCTION_READY_CLAIMED"));
    }

    #[test]
    fn config_migration_upgrades_legacy_local_markets_and_notifications() {
        let legacy = r#"
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

[markets]
allowed_exchanges = ["coinbase", "kraken"]
allowed_dexes = []
allowed_chains = []
allowed_assets = ["BTC", "ETH", "USDC"]

[notifications]
notify_channels = []
"#;

        let report = migrate_config_toml_to_current(legacy)
            .expect("legacy local config should migrate and validate");

        assert_eq!(report.status, ConfigMigrationStatus::Migrated);
        assert!(report
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_MIGRATED_MARKETS_TO_VENUES"));
        assert!(report
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_MIGRATED_NOTIFICATIONS_TO_COMMUNICATION"));
        assert!(report
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_DEFAULTED_RISK_GAS_FEE_CAP"));
        assert!(report
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_DEFAULTED_DISABLED_SECRET_REFERENCES"));
        assert_eq!(report.config.venues.cex_allowlist.len(), 2);
        assert!(report.config.risk.gas_fee_cap_quote.abs() < f64::EPSILON);
        assert!(!report.live_execution_enabled);
        assert!(!report.secret_material_loaded);
        assert!(!report.production_ready);
        assert!(!report.migrated_toml.contains("[markets]"));
        assert!(report.migrated_toml.contains("[venues]"));
    }

    #[test]
    fn config_migration_upgrades_legacy_venue_field_aliases_in_current_section() {
        let legacy = r#"
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
gas_fee_cap_quote = 0.0

[venues]
allowed_exchanges = ["coinbase", "kraken"]
allowed_dexes = ["paper-uniswap"]
allowed_chains = ["ethereum"]
allowed_assets = ["BTC", "ETH", "USDC"]

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

        let report = migrate_config_toml_to_current(legacy)
            .expect("legacy venue field aliases should migrate and validate");

        assert_eq!(report.status, ConfigMigrationStatus::Migrated);
        assert!(report
            .action_codes
            .iter()
            .any(|code| code == "CONFIG_MIGRATED_VENUE_FIELD_ALIASES"));
        assert_eq!(report.config.venues.cex_allowlist, ["coinbase", "kraken"]);
        assert_eq!(report.config.venues.dex_allowlist, ["paper-uniswap"]);
        assert_eq!(report.config.venues.chain_allowlist, ["ethereum"]);
        assert_eq!(report.config.venues.asset_allowlist, ["BTC", "ETH", "USDC"]);
        assert!(!report.migrated_toml.contains("allowed_exchanges"));
        assert!(report.migrated_toml.contains("cex_allowlist"));
        assert!(!report.secret_material_loaded);
        assert!(!report.live_execution_enabled);
        assert!(!report.production_ready);
    }

    #[test]
    fn config_migration_rejects_ambiguous_legacy_and_current_venue_fields() {
        let legacy = r#"
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
gas_fee_cap_quote = 0.0

[venues]
cex_allowlist = ["coinbase"]
allowed_exchanges = ["kraken"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC"]

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

        let error = migrate_config_toml_to_current(legacy)
            .expect_err("ambiguous legacy and current venue fields should fail closed");

        match error {
            ConfigError::ParseFailed { reason } => {
                assert!(reason.contains("allowed_exchanges"));
                assert!(reason.contains("cex_allowlist"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
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

    fn runtime_reload_request(
        side_effect_claimed: bool,
        include_reload_change: bool,
    ) -> RuntimeConfigReloadValidationRequest {
        let initial_config =
            AgentConfig::from_toml_str(OBSERVE_CONFIG).expect("initial config should parse");
        let reloaded_text = if include_reload_change {
            OBSERVE_CONFIG
                .replace(
                    "cex_allowlist = [\"coinbase\", \"kraken\"]",
                    "cex_allowlist = [\"coinbase\", \"kraken\", \"binance\"]",
                )
                .replace(
                    "asset_allowlist = [\"BTC\", \"ETH\", \"USDC\"]",
                    "asset_allowlist = [\"BTC\", \"ETH\", \"USDC\", \"SOL\"]",
                )
        } else {
            OBSERVE_CONFIG.to_owned()
        };
        let reloaded_config =
            AgentConfig::from_toml_str(&reloaded_text).expect("reloaded config should parse");

        RuntimeConfigReloadValidationRequest {
            reload_id: "local-runtime-config-reload".to_owned(),
            initial_config,
            reloaded_config,
            service_manager_action_performed: side_effect_claimed,
            secret_material_loaded: side_effect_claimed,
            external_submission_performed: side_effect_claimed,
            live_execution_performed: side_effect_claimed,
            production_ready_claimed: side_effect_claimed,
        }
    }
}
