#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope, FeeProvider,
    LiquidityRole, MarketDataProvider, MarketPair, PolicyApproval, PolicyDecision, PolicyEngine,
    PolicyViolation, VenueKind, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error, fmt};

/// Stable CEX framework version for audit and future replay surfaces.
pub const CEX_CONNECTOR_FRAMEWORK_VERSION: &str = "phase-7-cex-framework-v1";

/// Centralized-exchange connector capabilities.
///
/// These are declarations only. A true production adapter must still prove each
/// capability with integration tests, rate-limit tests, and exchange-specific
/// review before use with live funds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexConnectorCapabilities {
    /// Public REST market-data endpoint support.
    pub rest_market_data: bool,
    /// Public WebSocket market-data endpoint support.
    pub websocket_market_data: bool,
    /// Authenticated balance-read support.
    pub authenticated_balances: bool,
    /// Order submission support.
    pub order_submission: bool,
    /// Order cancellation support.
    pub order_cancel: bool,
    /// Exchange-provided sandbox or test environment support.
    pub sandbox: bool,
    /// Limit orders are supported.
    pub limit_orders: bool,
    /// Market orders are supported.
    pub market_orders: bool,
    /// Post-only maker orders are supported.
    pub post_only_orders: bool,
    /// Immediate-or-cancel orders are supported.
    pub time_in_force_ioc: bool,
    /// Fill-or-kill orders are supported.
    pub time_in_force_fok: bool,
    /// Connector has explicit rate-limit metadata.
    pub rate_limit_metadata: bool,
}

impl CexConnectorCapabilities {
    /// Safest default for framework-only declarations.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            rest_market_data: false,
            websocket_market_data: false,
            authenticated_balances: false,
            order_submission: false,
            order_cancel: false,
            sandbox: false,
            limit_orders: false,
            market_orders: false,
            post_only_orders: false,
            time_in_force_ioc: false,
            time_in_force_fok: false,
            rate_limit_metadata: false,
        }
    }

    /// Conservative paper/sandbox test capability set.
    #[must_use]
    pub const fn paper_sandbox() -> Self {
        Self {
            rest_market_data: false,
            websocket_market_data: false,
            authenticated_balances: false,
            order_submission: true,
            order_cancel: false,
            sandbox: true,
            limit_orders: true,
            market_orders: true,
            post_only_orders: true,
            time_in_force_ioc: true,
            time_in_force_fok: false,
            rate_limit_metadata: false,
        }
    }
}

/// Non-secret centralized-exchange venue profile.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexVenueProfile {
    /// Normalized venue reference. Kind must be `VenueKind::Cex`.
    pub venue: VenueRef,
    /// Human-readable display name.
    pub display_name: String,
    /// Declared connector capabilities.
    pub capabilities: CexConnectorCapabilities,
    /// Whether terms/jurisdiction review is complete for this venue.
    pub terms_and_jurisdiction_reviewed: bool,
    /// Whether fee schedule validation is complete for this venue.
    pub fees_verified: bool,
    /// Whether rate-limit validation is complete for this venue.
    pub rate_limits_verified: bool,
}

impl CexVenueProfile {
    /// Create a CEX venue profile after deterministic validation.
    pub fn new(
        venue: VenueRef,
        display_name: impl Into<String>,
        capabilities: CexConnectorCapabilities,
    ) -> Result<Self, CexConnectorError> {
        let profile = Self {
            venue,
            display_name: display_name.into(),
            capabilities,
            terms_and_jurisdiction_reviewed: false,
            fees_verified: false,
            rate_limits_verified: false,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Validate this profile without performing network or credential checks.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_venue_ref(&self.venue, &mut violations);

        if self.display_name.trim().is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_DISPLAY_NAME_REQUIRED",
                "CEX display name must be non-empty",
            ));
        }

        if self.capabilities.post_only_orders && !self.capabilities.limit_orders {
            violations.push(CexConnectorViolation::new(
                "POST_ONLY_REQUIRES_LIMIT_SUPPORT",
                "post-only support requires limit-order support",
            ));
        }

        if self.capabilities.time_in_force_fok
            && !self.capabilities.limit_orders
            && !self.capabilities.market_orders
        {
            violations.push(CexConnectorViolation::new(
                "FOK_REQUIRES_ORDER_SUPPORT",
                "fill-or-kill support requires at least one order type",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

/// Registry for known centralized-exchange venue profiles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CexConnectorRegistry {
    profiles: Vec<CexVenueProfile>,
}

impl CexConnectorRegistry {
    /// Build a registry from validated profiles.
    pub fn new(profiles: Vec<CexVenueProfile>) -> Result<Self, CexConnectorError> {
        let registry = Self { profiles };
        registry.validate()?;
        Ok(registry)
    }

    /// Return the number of profiles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Return true when no profiles are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// Return all registered profiles.
    #[must_use]
    pub fn profiles(&self) -> &[CexVenueProfile] {
        &self.profiles
    }

    /// Find a profile by configured venue name.
    #[must_use]
    pub fn find(&self, venue_name: &str) -> Option<&CexVenueProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.venue.name.eq_ignore_ascii_case(venue_name))
    }

    /// Require a profile by configured venue name.
    pub fn require(&self, venue_name: &str) -> Result<&CexVenueProfile, CexConnectorError> {
        self.find(venue_name)
            .ok_or_else(|| CexConnectorError::VenueNotRegistered {
                venue: venue_name.to_owned(),
            })
    }

    fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        let mut names = HashSet::new();

        for profile in &self.profiles {
            if let Err(CexConnectorError::ValidationFailed {
                violations: profile_violations,
            }) = profile.validate()
            {
                violations.extend(profile_violations);
            }

            let normalized = profile.venue.name.trim().to_ascii_lowercase();
            if !normalized.is_empty() && !names.insert(normalized.clone()) {
                violations.push(CexConnectorViolation::new_owned(
                    "CEX_DUPLICATE_VENUE",
                    format!("duplicate CEX venue profile: {normalized}"),
                ));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

/// CEX order side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexOrderSide {
    /// Buy base asset with quote asset.
    Buy,
    /// Sell base asset for quote asset.
    Sell,
}

/// CEX order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexOrderType {
    /// Limit order with explicit limit price.
    Limit,
    /// Market order.
    Market,
    /// Post-only limit order.
    PostOnly,
}

/// CEX time-in-force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexTimeInForce {
    /// Good-till-canceled.
    Gtc,
    /// Immediate-or-cancel.
    Ioc,
    /// Fill-or-kill.
    Fok,
}

/// Deterministic CEX order status boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexOrderStatus {
    /// Request was validated by local framework but not submitted live.
    LocallyValidated,
    /// Future live adapter accepted the order.
    Accepted,
    /// Future live adapter rejected the order.
    Rejected,
    /// Future live adapter reported a full fill.
    Filled,
    /// Future live adapter reported a partial fill.
    PartiallyFilled,
    /// Future live adapter reported cancellation.
    Cancelled,
}

/// CEX order request model consumed by future CEX execution adapters.
///
/// Phase 7 validation can approve only non-live framework/paper checks. This
/// request does not place an order by itself.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexOrderRequest {
    /// Stable request id.
    pub id: String,
    /// Strategy profile id that produced the request.
    pub strategy_id: String,
    /// Client-side id suitable for future idempotency checks.
    pub client_order_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target venue.
    pub venue: VenueRef,
    /// Market pair.
    pub pair: MarketPair,
    /// Buy or sell side.
    pub side: CexOrderSide,
    /// Order type.
    pub order_type: CexOrderType,
    /// Time in force.
    pub time_in_force: CexTimeInForce,
    /// Requested base quantity.
    pub quantity_base: f64,
    /// Optional limit price in quote units.
    pub limit_price_quote: Option<f64>,
    /// Proposed notional in quote units.
    pub notional_quote: f64,
    /// Expected gross profit in quote units before fees.
    pub expected_profit_quote: f64,
    /// Worst accepted loss in quote units.
    pub max_loss_quote: f64,
    /// Requested slippage ceiling.
    pub slippage_bps: u16,
    /// Estimated venue fee in quote units.
    pub estimated_fee_quote: f64,
    /// Age of source market data in milliseconds.
    pub market_data_age_ms: u64,
    /// Expected liquidity role for fee and behavior checks.
    pub liquidity_role: LiquidityRole,
    /// Whether this request is reduce-only in a future margin/derivatives connector.
    pub reduce_only: bool,
}

impl CexOrderRequest {
    /// Validate the request shape and order-type rules.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id("order request", &self.id, &mut violations);
        validate_id("strategy", &self.strategy_id, &mut violations);
        validate_id("client order", &self.client_order_id, &mut violations);
        validate_venue_ref(&self.venue, &mut violations);

        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_PAIR_INVALID",
                source.to_string(),
            ));
        }

        if !is_positive_finite(self.quantity_base) {
            violations.push(CexConnectorViolation::new(
                "CEX_ORDER_QUANTITY_INVALID",
                "CEX order quantity must be positive and finite",
            ));
        }

        if !is_positive_finite(self.notional_quote) {
            violations.push(CexConnectorViolation::new(
                "CEX_ORDER_NOTIONAL_INVALID",
                "CEX order notional must be positive and finite",
            ));
        }

        if !is_positive_finite(self.expected_profit_quote) {
            violations.push(CexConnectorViolation::new(
                "CEX_EXPECTED_PROFIT_INVALID",
                "CEX order expected profit must be positive and finite before fees",
            ));
        }

        if !is_non_negative_finite(self.max_loss_quote) {
            violations.push(CexConnectorViolation::new(
                "CEX_MAX_LOSS_INVALID",
                "CEX order max loss must be non-negative and finite",
            ));
        }

        if !is_non_negative_finite(self.estimated_fee_quote) {
            violations.push(CexConnectorViolation::new(
                "CEX_ESTIMATED_FEE_INVALID",
                "CEX estimated fee must be non-negative and finite",
            ));
        }

        match self.order_type {
            CexOrderType::Limit | CexOrderType::PostOnly => {
                if !self.limit_price_quote.is_some_and(is_positive_finite) {
                    violations.push(CexConnectorViolation::new(
                        "CEX_LIMIT_PRICE_REQUIRED",
                        "limit and post-only orders require a positive finite limit price",
                    ));
                }
            }
            CexOrderType::Market => {
                if self.limit_price_quote.is_some() {
                    violations.push(CexConnectorViolation::new(
                        "CEX_MARKET_PRICE_FORBIDDEN",
                        "market orders must not carry a limit price",
                    ));
                }
            }
        }

        if self.order_type == CexOrderType::PostOnly && self.time_in_force != CexTimeInForce::Gtc {
            violations.push(CexConnectorViolation::new(
                "CEX_POST_ONLY_REQUIRES_GTC",
                "post-only orders must use good-till-canceled time in force in Phase 7",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Convert the order request into a policy intent. This does not execute.
    pub fn to_execution_intent(&self) -> Result<ExecutionIntent, CexConnectorError> {
        self.validate()?;
        Ok(ExecutionIntent {
            id: self.id.clone(),
            strategy_id: self.strategy_id.clone(),
            kind: ExecutionIntentKind::CexOrder,
            scope: self.scope,
            venue: self.venue.clone(),
            chain: None,
            base_asset: self.pair.base.clone(),
            quote_asset: self.pair.quote.clone(),
            notional_quote: self.notional_quote,
            expected_profit_quote: self.expected_profit_quote,
            max_loss_quote: self.max_loss_quote,
            slippage_bps: self.slippage_bps,
            estimated_fee_quote: self.estimated_fee_quote,
            gas_fee_quote: 0.0,
            market_data_age_ms: self.market_data_age_ms,
            destination: DestinationPolicy::InternalAccount,
            requires_signing: false,
        })
    }
}

/// Framework-level policy gate for CEX order requests.
///
/// Phase 7 validates paper/sandbox order requests only. Live CEX orders remain
/// unavailable until later phases implement audited execution adapters,
/// credential custody, rate-limit handling, and external validation.
#[derive(Debug, Clone, PartialEq)]
pub struct CexPolicyGate {
    policy: PolicyEngine,
}

impl CexPolicyGate {
    /// Build a CEX policy gate from the current policy engine.
    #[must_use]
    pub fn new(policy: PolicyEngine) -> Self {
        Self { policy }
    }

    /// Validate a CEX order request against profile capabilities and policy.
    pub fn validate_order(
        &self,
        profile: &CexVenueProfile,
        request: &CexOrderRequest,
    ) -> Result<PolicyApproval, CexConnectorError> {
        request.validate()?;
        profile.validate()?;
        validate_profile_matches_request(profile, request)?;

        if request.scope == ExecutionScope::Live {
            return Err(CexConnectorError::LiveOrdersUnavailable);
        }

        if request.scope == ExecutionScope::Observe {
            return Err(CexConnectorError::ObserveOrdersUnavailable);
        }

        validate_capabilities(profile, request)?;

        match self.policy.evaluate(&request.to_execution_intent()?) {
            PolicyDecision::Approved { approval } => Ok(approval),
            PolicyDecision::Denied { violations } => {
                Err(CexConnectorError::PolicyDenied { violations })
            }
        }
    }
}

/// CEX connector identity boundary.
pub trait CexConnectorIdentity {
    /// Stable connector name for diagnostics and audit records.
    fn connector_name(&self) -> &str;

    /// Venue profile for this connector.
    fn venue_profile(&self) -> &CexVenueProfile;
}

/// Read-only CEX connector boundary.
///
/// Implementors may provide public market data and fee lookups, but this trait
/// does not permit order submission or balance mutation.
pub trait CexReadOnlyConnector: CexConnectorIdentity + MarketDataProvider + FeeProvider {}

impl<T> CexReadOnlyConnector for T where T: CexConnectorIdentity + MarketDataProvider + FeeProvider {}

/// Future authenticated CEX trading connector boundary.
///
/// Phase 7 defines the interface only. Implementors in later phases must fail
/// closed, call policy, journal audit records, enforce rate limits, and never
/// submit live orders without external validation.
pub trait CexTradingConnector: CexConnectorIdentity {
    /// Validate a CEX order request before submission.
    fn validate_order_request(
        &self,
        request: &CexOrderRequest,
    ) -> Result<PolicyApproval, CexConnectorError>;

    /// Submit a validated CEX order request.
    fn submit_order(&self, request: &CexOrderRequest) -> Result<CexOrderStatus, CexConnectorError>;
}

/// One CEX framework validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CexConnectorViolation {
    code: &'static str,
    message: String,
}

impl CexConnectorViolation {
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

    /// Human-readable violation message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// CEX framework errors.
#[derive(Debug, Clone, PartialEq)]
pub enum CexConnectorError {
    /// Validation failed with deterministic violations.
    ValidationFailed {
        violations: Vec<CexConnectorViolation>,
    },
    /// Venue profile is not registered.
    VenueNotRegistered { venue: String },
    /// Profile and request reference different venues.
    VenueMismatch { profile: String, request: String },
    /// Requested feature is unsupported by the venue profile.
    CapabilityUnavailable {
        venue: String,
        capability: &'static str,
    },
    /// Phase 7 live order submission is unavailable.
    LiveOrdersUnavailable,
    /// Observe scope cannot submit CEX orders.
    ObserveOrdersUnavailable,
    /// Policy denied the converted execution intent.
    PolicyDenied { violations: Vec<PolicyViolation> },
}

impl CexConnectorError {
    /// Return validation violations, if available.
    #[must_use]
    pub fn violations(&self) -> &[CexConnectorViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::VenueNotRegistered { .. }
            | Self::VenueMismatch { .. }
            | Self::CapabilityUnavailable { .. }
            | Self::LiveOrdersUnavailable
            | Self::ObserveOrdersUnavailable
            | Self::PolicyDenied { .. } => &[],
        }
    }
}

impl fmt::Display for CexConnectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "CEX connector validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::VenueNotRegistered { venue } => {
                write!(formatter, "CEX venue is not registered: {venue}")
            }
            Self::VenueMismatch { profile, request } => {
                write!(
                    formatter,
                    "CEX venue mismatch between profile {profile} and request {request}"
                )
            }
            Self::CapabilityUnavailable { venue, capability } => {
                write!(
                    formatter,
                    "CEX venue {venue} does not support capability {capability}"
                )
            }
            Self::LiveOrdersUnavailable => {
                formatter.write_str("live CEX orders are unavailable in Phase 7")
            }
            Self::ObserveOrdersUnavailable => {
                formatter.write_str("observe scope cannot submit CEX orders")
            }
            Self::PolicyDenied { violations } => {
                write!(
                    formatter,
                    "CEX order denied by policy with {} violation(s)",
                    violations.len()
                )?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for CexConnectorError {}

fn validate_profile_matches_request(
    profile: &CexVenueProfile,
    request: &CexOrderRequest,
) -> Result<(), CexConnectorError> {
    if same_venue(&profile.venue, &request.venue) {
        return Ok(());
    }

    Err(CexConnectorError::VenueMismatch {
        profile: profile.venue.name.clone(),
        request: request.venue.name.clone(),
    })
}

fn validate_capabilities(
    profile: &CexVenueProfile,
    request: &CexOrderRequest,
) -> Result<(), CexConnectorError> {
    let venue = profile.venue.name.clone();

    if !profile.capabilities.order_submission {
        return Err(CexConnectorError::CapabilityUnavailable {
            venue,
            capability: "order-submission",
        });
    }

    match request.order_type {
        CexOrderType::Limit if !profile.capabilities.limit_orders => {
            return Err(CexConnectorError::CapabilityUnavailable {
                venue,
                capability: "limit-orders",
            });
        }
        CexOrderType::Market if !profile.capabilities.market_orders => {
            return Err(CexConnectorError::CapabilityUnavailable {
                venue,
                capability: "market-orders",
            });
        }
        CexOrderType::PostOnly if !profile.capabilities.post_only_orders => {
            return Err(CexConnectorError::CapabilityUnavailable {
                venue,
                capability: "post-only-orders",
            });
        }
        CexOrderType::Limit | CexOrderType::Market | CexOrderType::PostOnly => {}
    }

    match request.time_in_force {
        CexTimeInForce::Ioc if !profile.capabilities.time_in_force_ioc => {
            return Err(CexConnectorError::CapabilityUnavailable {
                venue,
                capability: "time-in-force-ioc",
            });
        }
        CexTimeInForce::Fok if !profile.capabilities.time_in_force_fok => {
            return Err(CexConnectorError::CapabilityUnavailable {
                venue,
                capability: "time-in-force-fok",
            });
        }
        CexTimeInForce::Gtc | CexTimeInForce::Ioc | CexTimeInForce::Fok => {}
    }

    if request.reduce_only {
        return Err(CexConnectorError::CapabilityUnavailable {
            venue,
            capability: "reduce-only-derivatives",
        });
    }

    Ok(())
}

fn validate_venue_ref(venue: &VenueRef, violations: &mut Vec<CexConnectorViolation>) {
    if venue.name.trim().is_empty() {
        violations.push(CexConnectorViolation::new(
            "CEX_VENUE_NAME_REQUIRED",
            "CEX venue name must be non-empty",
        ));
    }

    if venue.kind != VenueKind::Cex {
        violations.push(CexConnectorViolation::new(
            "CEX_VENUE_KIND_REQUIRED",
            "CEX framework only accepts VenueKind::Cex venues",
        ));
    }
}

fn validate_id(label: &'static str, value: &str, violations: &mut Vec<CexConnectorViolation>) {
    if value.trim().is_empty() {
        violations.push(CexConnectorViolation::new_owned(
            "CEX_ID_REQUIRED",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn same_venue(left: &VenueRef, right: &VenueRef) -> bool {
    left.kind == right.kind && left.name.eq_ignore_ascii_case(&right.name)
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn is_non_negative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::{
        CexConnectorCapabilities, CexConnectorError, CexConnectorRegistry, CexOrderRequest,
        CexOrderSide, CexOrderType, CexPolicyGate, CexTimeInForce, CexVenueProfile,
    };
    use crate::{
        AgentConfig, ExecutionScope, LiquidityRole, MarketPair, PolicyEngine, VenueKind, VenueRef,
    };

    const PAPER_CONFIG: &str = r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 50.0
max_daily_loss_quote = 10.0
max_open_exposure_quote = 100.0
slippage_bps = 50
gas_fee_cap_quote = 2.0

[venues]
cex_allowlist = ["paper-coinbase"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC", "USDC"]

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

    fn venue() -> VenueRef {
        VenueRef {
            name: "paper-coinbase".to_owned(),
            kind: VenueKind::Cex,
        }
    }

    fn profile() -> CexVenueProfile {
        CexVenueProfile::new(
            venue(),
            "Paper Coinbase",
            CexConnectorCapabilities::paper_sandbox(),
        )
        .expect("profile should validate")
    }

    fn order_request() -> CexOrderRequest {
        CexOrderRequest {
            id: "cex-order-1".to_owned(),
            strategy_id: "strategy-cex".to_owned(),
            client_order_id: "client-cex-order-1".to_owned(),
            scope: ExecutionScope::Paper,
            venue: venue(),
            pair: MarketPair::new("BTC", "USDC").expect("pair should validate"),
            side: CexOrderSide::Buy,
            order_type: CexOrderType::Limit,
            time_in_force: CexTimeInForce::Gtc,
            quantity_base: 0.01,
            limit_price_quote: Some(100.0),
            notional_quote: 25.0,
            expected_profit_quote: 1.0,
            max_loss_quote: 1.0,
            slippage_bps: 20,
            estimated_fee_quote: 0.10,
            market_data_age_ms: 1_000,
            liquidity_role: LiquidityRole::Maker,
            reduce_only: false,
        }
    }

    fn policy_gate() -> CexPolicyGate {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        CexPolicyGate::new(PolicyEngine::from_config(config))
    }

    #[test]
    fn registry_rejects_duplicate_venues() {
        let error = CexConnectorRegistry::new(vec![profile(), profile()])
            .expect_err("duplicates must be rejected");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_DUPLICATE_VENUE"));
    }

    #[test]
    fn policy_gate_approves_paper_cex_order() {
        let approval = policy_gate()
            .validate_order(&profile(), &order_request())
            .expect("paper order should be policy approved");
        assert_eq!(approval.intent_id, "cex-order-1");
        assert_eq!(approval.approved_scope, ExecutionScope::Paper);
    }

    #[test]
    fn policy_gate_rejects_live_scope_in_phase_7() {
        let mut request = order_request();
        request.scope = ExecutionScope::Live;
        let error = policy_gate()
            .validate_order(&profile(), &request)
            .expect_err("live CEX order must be blocked in Phase 7");
        assert!(matches!(error, CexConnectorError::LiveOrdersUnavailable));
    }

    #[test]
    fn capability_validation_rejects_unsupported_market_order() {
        let mut capabilities = CexConnectorCapabilities::paper_sandbox();
        capabilities.market_orders = false;
        let profile = CexVenueProfile::new(venue(), "Paper Coinbase", capabilities)
            .expect("profile should validate");
        let mut request = order_request();
        request.order_type = CexOrderType::Market;
        request.limit_price_quote = None;
        let error = policy_gate()
            .validate_order(&profile, &request)
            .expect_err("unsupported market order must be rejected");
        assert!(matches!(
            error,
            CexConnectorError::CapabilityUnavailable {
                capability: "market-orders",
                ..
            }
        ));
    }
}
