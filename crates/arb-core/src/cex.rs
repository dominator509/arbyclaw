#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, DestinationPolicy,
    ExecutionIntent, ExecutionIntentKind, ExecutionScope, FeeModelError, FeeProvider, FeeSchedule,
    LiquidityRole, MarketDataCapabilities, MarketDataError, MarketDataProvider, MarketDataRequest,
    MarketPair, NormalizedQuote, OrderBookSnapshot, PolicyApproval, PolicyDecision, PolicyEngine,
    PolicyViolation, PriceLevel, SecretRef, StateCheckpoint, StateStore, StateStoreError,
    VenueKind, VenueRef,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

/// Stable CEX framework version for audit and future replay surfaces.
pub const CEX_CONNECTOR_FRAMEWORK_VERSION: &str = "phase-7-cex-framework-v1";

/// State-store subsystem name for local CEX framework checkpoints.
pub const CEX_STATE_SUBSYSTEM: &str = "cex";

/// Checkpoint key for the latest locally validated CEX framework order.
pub const CEX_LAST_ORDER_VALIDATION_CHECKPOINT_KEY: &str = "cex.last_order_validation";

/// Checkpoint key for the latest locally reconciled CEX order lifecycle.
pub const CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY: &str = "cex.last_order_lifecycle";

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

/// Local deterministic matching rules for one exchange/pair fixture.
///
/// These rules are non-secret fixture metadata only. They model exchange-shaped
/// validation constraints without connecting to any exchange API.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexExchangeMatchingRules {
    /// Venue these matching rules apply to.
    pub venue: VenueRef,
    /// Market pair these matching rules apply to.
    pub pair: MarketPair,
    /// Minimum base quantity accepted by the fixture.
    pub min_quantity_base: f64,
    /// Quantity increment in base units.
    pub quantity_step_base: f64,
    /// Minimum quote notional accepted by the fixture.
    pub min_notional_quote: f64,
    /// Optional maximum quote notional accepted by the fixture.
    pub max_notional_quote: Option<f64>,
    /// Price tick size in quote units for limit/post-only requests.
    pub price_tick_quote: f64,
    /// Whether post-only requests are represented as GTC maker-only orders.
    pub post_only_requires_gtc: bool,
    /// Whether IOC market orders are accepted by the fixture.
    pub ioc_market_orders_supported: bool,
}

/// Sanitized result from local exchange-specific CEX fixture validation.
///
/// This is intentionally a report, not an execution receipt. It carries side
/// effect booleans so future runtime/replay callers can assert the boundary
/// stayed local-only.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexExchangeFixtureValidation {
    /// Framework version that produced this local validation.
    pub framework_version: String,
    /// Connector name used by the local fixture adapter.
    pub connector_name: String,
    /// Request id that was validated.
    pub request_id: String,
    /// Client order id used for idempotency checks.
    pub client_order_id: String,
    /// Venue validated by the fixture.
    pub venue: VenueRef,
    /// Pair validated by the fixture.
    pub pair: MarketPair,
    /// Whether profile/policy validation passed.
    pub profile_policy_validated: bool,
    /// Whether exchange-specific matching rules passed.
    pub matching_rules_validated: bool,
    /// Whether a REST call occurred. Always false for this fixture path.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection occurred. Always false for this fixture path.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Always false for this fixture path.
    pub credentials_loaded: bool,
    /// Whether an external order was submitted. Always false for this fixture path.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Always false for this fixture path.
    pub live_execution_performed: bool,
    /// Whether this report claims production readiness. Always false.
    pub production_ready: bool,
    /// Remaining non-secret blockers for this exchange-specific fixture path.
    pub unresolved_blockers: Vec<String>,
}

/// Local CEX live-adapter implementation boundary review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexLiveAdapterBoundaryReviewStatus {
    /// Local prerequisites exist, but live adapter implementation and external validation are missing.
    BlockedPendingLiveAdapterImplementation,
}

/// Non-secret local review request for the CEX live-adapter boundary.
///
/// This consumes local validation booleans only. It must not perform exchange
/// calls, open WebSockets, load credentials, submit orders, cancel orders, or
/// claim production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexLiveAdapterBoundaryReviewRequest {
    /// Stable review id.
    pub review_id: String,
    /// Connector or adapter label under review.
    pub connector_name: String,
    /// Venue under review.
    pub venue: VenueRef,
    /// Whether local REST request-plan validation exists.
    pub rest_request_plan_validated: bool,
    /// Whether local WebSocket request-plan validation exists.
    pub websocket_request_plan_validated: bool,
    /// Whether local exchange-shaped lifecycle transcript parsing exists.
    pub lifecycle_transcript_parsing_validated: bool,
    /// Whether local exchange-shaped balance snapshot parsing exists.
    pub balance_snapshot_parsing_validated: bool,
    /// Whether local credential/API-scope review exists.
    pub credential_scope_reviewed: bool,
    /// Whether local rate-limit review exists.
    pub rate_limit_reviewed: bool,
    /// Whether local exchange-specific matching rules are validated.
    pub exchange_matching_rules_validated: bool,
    /// Whether external sandbox order lifecycle evidence is available.
    pub sandbox_order_lifecycle_evidence_available: bool,
    /// Whether external sandbox balance snapshot evidence is available.
    pub sandbox_balance_evidence_available: bool,
    /// Whether external sandbox cancel/reconciliation evidence is available.
    pub sandbox_cancel_evidence_available: bool,
    /// Whether production idempotency/replay evidence is available.
    pub production_idempotency_evidence_available: bool,
    /// Whether credential material was loaded by this review. Must remain false.
    pub credential_material_loaded: bool,
    /// Whether a REST call was performed by this review. Must remain false.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection was opened by this review. Must remain false.
    pub websocket_connection_opened: bool,
    /// Whether an external order/cancel submission occurred. Must remain false.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether the request attempts to claim production readiness. Must remain false.
    pub production_ready_claimed: bool,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
}

/// Non-secret local CEX live-adapter boundary review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexLiveAdapterBoundaryReviewReport {
    /// Framework version that produced this report.
    pub framework_version: String,
    /// Stable review id.
    pub review_id: String,
    /// Connector or adapter label under review.
    pub connector_name: String,
    /// Venue under review.
    pub venue: VenueRef,
    /// Local review status.
    pub status: CexLiveAdapterBoundaryReviewStatus,
    /// Whether local REST request-plan validation exists.
    pub rest_request_plan_validated: bool,
    /// Whether local WebSocket request-plan validation exists.
    pub websocket_request_plan_validated: bool,
    /// Whether local lifecycle transcript parsing exists.
    pub lifecycle_transcript_parsing_validated: bool,
    /// Whether local balance snapshot parsing exists.
    pub balance_snapshot_parsing_validated: bool,
    /// Whether local credential/API-scope review exists.
    pub credential_scope_reviewed: bool,
    /// Whether local rate-limit review exists.
    pub rate_limit_reviewed: bool,
    /// Whether local exchange-specific matching rules are validated.
    pub exchange_matching_rules_validated: bool,
    /// Whether external sandbox order lifecycle evidence is available.
    pub sandbox_order_lifecycle_evidence_available: bool,
    /// Whether external sandbox balance snapshot evidence is available.
    pub sandbox_balance_evidence_available: bool,
    /// Whether external sandbox cancel/reconciliation evidence is available.
    pub sandbox_cancel_evidence_available: bool,
    /// Whether production idempotency/replay evidence is available.
    pub production_idempotency_evidence_available: bool,
    /// Whether credential material was loaded. Always false here.
    pub credential_material_loaded: bool,
    /// Whether a REST call was performed. Always false here.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection was opened. Always false here.
    pub websocket_connection_opened: bool,
    /// Whether an external order/cancel submission occurred. Always false here.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this local review claims production readiness. Always false.
    pub production_ready: bool,
    /// Validation timestamp in Unix milliseconds.
    pub validated_at_unix_ms: u64,
    /// Remaining blocker codes for live adapter implementation and validation.
    pub blocker_codes: Vec<String>,
}

/// Supported local exchange-specific market-data transcript formats.
///
/// These are parser fixtures for captured/mock payloads only. They do not
/// perform REST calls, open WebSockets, or authenticate to exchanges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexExchangeMarketDataFormat {
    /// Binance-style `/api/v3/depth` payload with `bids` and `asks` arrays.
    BinanceDepth,
    /// Coinbase-style product book payload with `bids`, `asks`, and `sequence`.
    CoinbaseProductBook,
    /// Kraken-style depth payload with `result`, `b`, and `a` arrays.
    KrakenDepth,
}

/// Local exchange-specific market-data request kind.
///
/// These request plans are inert metadata for future adapter implementation.
/// They do not perform REST calls, open WebSockets, load credentials, or fetch
/// market data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexMarketDataRequestKind {
    /// Public REST order-book/depth request.
    RestOrderBook,
    /// Public WebSocket order-book/depth subscription request.
    WebSocketOrderBook,
}

/// Local exchange-specific market-data request plan.
///
/// A plan describes the endpoint or subscription shape a future adapter would
/// use, but it remains local metadata until a separately validated adapter is
/// implemented. The side-effect flags must stay false.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexMarketDataRequestPlan {
    /// Stable local plan id.
    pub plan_id: String,
    /// Exchange-specific transcript/parser shape expected for the response.
    pub format: CexExchangeMarketDataFormat,
    /// REST or WebSocket market-data request kind.
    pub request_kind: CexMarketDataRequestKind,
    /// Venue represented by the plan.
    pub venue: VenueRef,
    /// Market pair represented by the plan.
    pub pair: MarketPair,
    /// HTTP method for REST plans.
    pub rest_method: Option<String>,
    /// REST path for REST plans.
    pub rest_path: Option<String>,
    /// Sanitized REST query string for REST plans.
    pub rest_query: Option<String>,
    /// WebSocket channel for subscription plans.
    pub websocket_channel: Option<String>,
    /// Sanitized WebSocket subscription payload for subscription plans.
    pub websocket_subscription_json: Option<String>,
    /// Whether a REST call occurred. Must remain false.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection occurred. Must remain false.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Must remain false.
    pub credentials_loaded: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether this plan claims production readiness. Must remain false.
    pub production_ready: bool,
}

/// Local mocked/captured CEX market-data transcript.
///
/// This is the read-only exchange-specific parsing boundary. It accepts a
/// caller-supplied JSON payload and normalizes it into ArbyClaw market-data
/// models without using network, credentials, accounts, or live exchange APIs.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexMockMarketDataTranscript {
    /// Stable local transcript id.
    pub transcript_id: String,
    /// Exchange-specific payload shape.
    pub format: CexExchangeMarketDataFormat,
    /// Venue represented by the transcript.
    pub venue: VenueRef,
    /// Market pair represented by the transcript.
    pub pair: MarketPair,
    /// Raw non-secret JSON fixture payload.
    pub payload_json: String,
    /// Provider capture timestamp in Unix milliseconds.
    pub captured_at_unix_ms: u64,
    /// Local receive timestamp in Unix milliseconds.
    pub received_at_unix_ms: u64,
    /// Whether a REST call occurred. Must remain false.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection occurred. Must remain false.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Must remain false.
    pub credentials_loaded: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
}

/// Local CEX rate-limit scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexRateLimitScope {
    /// Public REST market-data request budget.
    RestMarketData,
    /// Public WebSocket connection/subscription budget.
    WebSocketMarketData,
    /// Authenticated order submission request budget.
    OrderSubmission,
    /// Authenticated order cancel request budget.
    OrderCancel,
}

/// Caller-supplied local CEX rate-limit observation.
///
/// This models fail-closed connector rate-limit behavior without calling an
/// exchange, loading credentials, opening sockets, submitting orders, or
/// querying account state.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexRateLimitObservation {
    /// Stable local observation id.
    pub observation_id: String,
    /// Venue represented by this observation.
    pub venue: VenueRef,
    /// Rate-limit scope represented by this observation.
    pub scope: CexRateLimitScope,
    /// Maximum requests allowed in the local window.
    pub max_requests_per_window: u32,
    /// Window length in milliseconds.
    pub window_ms: u64,
    /// Requests already observed in the current local window.
    pub observed_requests_in_window: u32,
    /// Optional exchange/provider retry-after hint in milliseconds.
    pub retry_after_ms: Option<u64>,
    /// Whether the provider already signaled rate limiting.
    pub provider_rate_limited: bool,
    /// Whether this observation came from a live provider call. Must remain false.
    pub live_provider_call_performed: bool,
    /// Whether a WebSocket was opened. Must remain false.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Must remain false.
    pub credential_loaded: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
}

/// Local CEX rate-limit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexRateLimitStatus {
    /// Observation is within the local configured budget.
    ReadyForLocalReview,
    /// Observation must fail closed before adapter activity.
    Blocked,
}

/// Non-secret CEX credential/API permission category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexCredentialPermission {
    /// Public market-data access only.
    ReadOnlyMarketData,
    /// Authenticated account balance read access.
    ReadBalances,
    /// Spot order placement permission.
    TradeOrders,
    /// Spot order cancellation permission.
    CancelOrders,
    /// Withdrawal permission. Forbidden for ArbyClaw local CEX review.
    Withdrawals,
    /// Internal transfer permission. Forbidden for ArbyClaw local CEX review.
    Transfers,
    /// Margin, derivatives, borrowing, or leverage permission. Forbidden here.
    MarginOrDerivatives,
    /// Account administration or API-key management permission. Forbidden here.
    AccountAdmin,
}

impl CexCredentialPermission {
    #[must_use]
    const fn is_forbidden_live_funds_permission(self) -> bool {
        matches!(
            self,
            Self::Withdrawals | Self::Transfers | Self::MarginOrDerivatives | Self::AccountAdmin
        )
    }
}

/// Local CEX credential/API-scope review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexCredentialScopeReviewStatus {
    /// Metadata-only review is coherent and ready for local human review.
    ReadyForLocalReview,
    /// Review must fail closed before adapter use.
    Blocked,
}

/// Caller-supplied local CEX credential/API-scope review request.
///
/// This carries non-secret reference metadata and permission labels only. It
/// must never contain API keys, tokens, plaintext secret material, provider
/// responses, account balances, or exchange-side credential dumps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexCredentialScopeReviewInput {
    /// Stable local review id.
    pub review_id: String,
    /// Venue represented by this review.
    pub venue: VenueRef,
    /// Non-secret credential reference.
    pub credential_reference: SecretRef,
    /// Permissions expected by the future adapter path.
    pub required_permissions: Vec<CexCredentialPermission>,
    /// Permissions declared or observed by a sanitized operator record.
    pub observed_permissions: Vec<CexCredentialPermission>,
    /// Permissions that must not be present.
    pub forbidden_permissions: Vec<CexCredentialPermission>,
    /// Last human/operator review timestamp in Unix milliseconds.
    pub reviewed_at_unix_ms: u64,
    /// Current local validation timestamp in Unix milliseconds.
    pub now_unix_ms: u64,
    /// Maximum allowed review age in milliseconds.
    pub max_review_age_ms: u64,
    /// Whether local fee schedule verification metadata is present.
    pub fee_schedule_reviewed: bool,
    /// Whether local rate-limit documentation review metadata is present.
    pub rate_limit_documentation_reviewed: bool,
    /// Whether local terms-of-service review metadata is present.
    pub terms_of_service_reviewed: bool,
    /// Whether local jurisdiction review metadata is present.
    pub jurisdiction_reviewed: bool,
    /// Whether local API capability review metadata is present.
    pub api_capabilities_reviewed: bool,
    /// Whether local incident/reputation review metadata is present.
    pub incident_reputation_reviewed: bool,
    /// Whether secret material was loaded. Must remain false.
    pub secret_material_loaded: bool,
    /// Whether plaintext credential material was seen. Must remain false.
    pub credential_plaintext_seen: bool,
    /// Whether a live provider call occurred. Must remain false.
    pub live_provider_call_performed: bool,
    /// Whether an account/balance query occurred. Must remain false.
    pub account_state_queried: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether this input claims production readiness. Must remain false.
    pub production_ready_claimed: bool,
}

/// Sanitized local CEX credential/API-scope review report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexCredentialScopeReviewReport {
    /// Stable local review id.
    pub review_id: String,
    /// Venue represented by this report.
    pub venue: VenueRef,
    /// Validation status.
    pub status: CexCredentialScopeReviewStatus,
    /// Permissions expected by the future adapter path.
    pub required_permissions: Vec<CexCredentialPermission>,
    /// Permissions declared or observed by a sanitized operator record.
    pub observed_permissions: Vec<CexCredentialPermission>,
    /// Permissions that must not be present.
    pub forbidden_permissions: Vec<CexCredentialPermission>,
    /// Required permissions missing from observed metadata.
    pub missing_required_permissions: Vec<CexCredentialPermission>,
    /// Forbidden permissions present in observed metadata.
    pub forbidden_permissions_present: Vec<CexCredentialPermission>,
    /// Whether the credential reference was validated as metadata only.
    pub credential_reference_validated: bool,
    /// Whether the review age exceeded `max_review_age_ms`.
    pub stale_review: bool,
    /// Whether local fee schedule verification metadata is present.
    pub fee_schedule_reviewed: bool,
    /// Whether local rate-limit documentation review metadata is present.
    pub rate_limit_documentation_reviewed: bool,
    /// Whether local terms-of-service review metadata is present.
    pub terms_of_service_reviewed: bool,
    /// Whether local jurisdiction review metadata is present.
    pub jurisdiction_reviewed: bool,
    /// Whether local API capability review metadata is present.
    pub api_capabilities_reviewed: bool,
    /// Whether local incident/reputation review metadata is present.
    pub incident_reputation_reviewed: bool,
    /// Whether local CEX governance metadata is coherent for local review.
    pub governance_review_passed: bool,
    /// Whether secret material was loaded. Always false for ready reports.
    pub secret_material_loaded: bool,
    /// Whether plaintext credential material was seen. Always false for ready reports.
    pub credential_plaintext_seen: bool,
    /// Whether a live provider call occurred. Always false for ready reports.
    pub live_provider_call_performed: bool,
    /// Whether account state was queried. Always false for ready reports.
    pub account_state_queried: bool,
    /// Whether live execution occurred. Always false for ready reports.
    pub live_execution_performed: bool,
    /// Whether this report claims production readiness. Always false.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

/// Sanitized local CEX rate-limit validation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexRateLimitReport {
    /// Stable local observation id.
    pub observation_id: String,
    /// Venue represented by this report.
    pub venue: VenueRef,
    /// Rate-limit scope represented by this report.
    pub scope: CexRateLimitScope,
    /// Validation status.
    pub status: CexRateLimitStatus,
    /// Maximum requests allowed in the local window.
    pub max_requests_per_window: u32,
    /// Window length in milliseconds.
    pub window_ms: u64,
    /// Requests already observed in the current local window.
    pub observed_requests_in_window: u32,
    /// Remaining request budget in the local window.
    pub remaining_requests_in_window: u32,
    /// Optional exchange/provider retry-after hint in milliseconds.
    pub retry_after_ms: Option<u64>,
    /// Whether the configured budget was exhausted or exceeded.
    pub local_budget_exhausted: bool,
    /// Whether the provider already signaled rate limiting.
    pub provider_rate_limited: bool,
    /// Whether this report is based on a live provider call. Always false here.
    pub live_provider_call_performed: bool,
    /// Whether a WebSocket was opened. Always false here.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Always false here.
    pub credential_loaded: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this report claims production readiness. Always false.
    pub production_ready: bool,
    /// Sanitized local violation codes.
    pub violation_codes: Vec<String>,
}

struct ParsedCexOrderBook {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
    source_sequence: Option<String>,
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

impl CexExchangeMatchingRules {
    /// Construct validated local exchange-specific matching rules.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        venue: VenueRef,
        pair: MarketPair,
        min_quantity_base: f64,
        quantity_step_base: f64,
        min_notional_quote: f64,
        max_notional_quote: Option<f64>,
        price_tick_quote: f64,
        post_only_requires_gtc: bool,
        ioc_market_orders_supported: bool,
    ) -> Result<Self, CexConnectorError> {
        let rules = Self {
            venue,
            pair,
            min_quantity_base,
            quantity_step_base,
            min_notional_quote,
            max_notional_quote,
            price_tick_quote,
            post_only_requires_gtc,
            ioc_market_orders_supported,
        };
        rules.validate()?;
        Ok(rules)
    }

    /// Binance-like local fixture profile for BTC/USDC spot matching rules.
    pub fn binance_btc_usdc_fixture() -> Result<Self, CexConnectorError> {
        Self::new(
            VenueRef {
                name: "binance".to_owned(),
                kind: VenueKind::Cex,
            },
            MarketPair::new("BTC", "USDC").map_err(|error| {
                CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new_owned(
                        "CEX_MATCHING_PAIR_INVALID",
                        error.to_string(),
                    )],
                }
            })?,
            0.000_01,
            0.000_01,
            10.0,
            None,
            0.01,
            true,
            true,
        )
    }

    /// Coinbase-like local fixture profile for BTC/USDC spot matching rules.
    pub fn coinbase_btc_usdc_fixture() -> Result<Self, CexConnectorError> {
        Self::new(
            VenueRef {
                name: "coinbase".to_owned(),
                kind: VenueKind::Cex,
            },
            MarketPair::new("BTC", "USDC").map_err(|error| {
                CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new_owned(
                        "CEX_MATCHING_PAIR_INVALID",
                        error.to_string(),
                    )],
                }
            })?,
            0.000_000_01,
            0.000_000_01,
            1.0,
            None,
            0.01,
            true,
            true,
        )
    }

    /// Kraken-like local fixture profile for BTC/USDC spot matching rules.
    pub fn kraken_btc_usdc_fixture() -> Result<Self, CexConnectorError> {
        Self::new(
            VenueRef {
                name: "kraken".to_owned(),
                kind: VenueKind::Cex,
            },
            MarketPair::new("BTC", "USDC").map_err(|error| {
                CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new_owned(
                        "CEX_MATCHING_PAIR_INVALID",
                        error.to_string(),
                    )],
                }
            })?,
            0.000_1,
            0.000_1,
            5.0,
            None,
            0.1,
            true,
            false,
        )
    }

    /// Validate fixture rules without network, credentials, balances, or orders.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_MATCHING_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if !is_positive_finite(self.min_quantity_base) {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_MIN_QUANTITY_INVALID",
                "CEX matching minimum quantity must be positive and finite",
            ));
        }
        if !is_positive_finite(self.quantity_step_base) {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_QUANTITY_STEP_INVALID",
                "CEX matching quantity step must be positive and finite",
            ));
        }
        if !is_positive_finite(self.min_notional_quote) {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_MIN_NOTIONAL_INVALID",
                "CEX matching minimum notional must be positive and finite",
            ));
        }
        if self
            .max_notional_quote
            .is_some_and(|max_notional| max_notional < self.min_notional_quote)
        {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_MAX_NOTIONAL_INVALID",
                "CEX matching maximum notional must be greater than or equal to minimum notional",
            ));
        }
        if !is_positive_finite(self.price_tick_quote) {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_PRICE_TICK_INVALID",
                "CEX matching price tick must be positive and finite",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Validate one request against these local matching rules.
    pub fn validate_order(&self, request: &CexOrderRequest) -> Result<(), CexConnectorError> {
        request.validate()?;
        self.validate()?;
        let mut violations = Vec::new();

        if !same_venue(&self.venue, &request.venue) {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_MATCHING_VENUE_MISMATCH",
                format!(
                    "CEX matching rules for {} cannot validate request venue {}",
                    self.venue.name, request.venue.name
                ),
            ));
        }
        if self.pair != request.pair {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_PAIR_MISMATCH",
                "CEX matching rules pair must match request pair",
            ));
        }
        if request.quantity_base + f64::EPSILON < self.min_quantity_base {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_MIN_QUANTITY_NOT_MET",
                "CEX order quantity is below exchange fixture minimum",
            ));
        }
        if !is_multiple_of_step(request.quantity_base, self.quantity_step_base) {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_QUANTITY_STEP_NOT_MET",
                "CEX order quantity does not align with exchange fixture step size",
            ));
        }
        if request.notional_quote + f64::EPSILON < self.min_notional_quote {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_MIN_NOTIONAL_NOT_MET",
                "CEX order notional is below exchange fixture minimum",
            ));
        }
        if self
            .max_notional_quote
            .is_some_and(|max_notional| request.notional_quote - max_notional > f64::EPSILON)
        {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_MAX_NOTIONAL_EXCEEDED",
                "CEX order notional exceeds exchange fixture maximum",
            ));
        }
        if let Some(limit_price) = request.limit_price_quote {
            if !is_multiple_of_step(limit_price, self.price_tick_quote) {
                violations.push(CexConnectorViolation::new(
                    "CEX_MATCHING_PRICE_TICK_NOT_MET",
                    "CEX limit price does not align with exchange fixture tick size",
                ));
            }
        }
        if request.order_type == CexOrderType::PostOnly
            && self.post_only_requires_gtc
            && request.time_in_force != CexTimeInForce::Gtc
        {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_POST_ONLY_TIF_INVALID",
                "CEX post-only fixture orders must use GTC",
            ));
        }
        if request.order_type == CexOrderType::Market
            && request.time_in_force == CexTimeInForce::Ioc
            && !self.ioc_market_orders_supported
        {
            violations.push(CexConnectorViolation::new(
                "CEX_MATCHING_IOC_MARKET_UNSUPPORTED",
                "CEX fixture does not support IOC market orders",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexExchangeFixtureValidation {
    fn from_validated_request(
        connector_name: &str,
        request: &CexOrderRequest,
    ) -> Result<Self, CexConnectorError> {
        let record = Self {
            framework_version: CEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            connector_name: connector_name.to_owned(),
            request_id: request.id.clone(),
            client_order_id: request.client_order_id.clone(),
            venue: request.venue.clone(),
            pair: request.pair.clone(),
            profile_policy_validated: true,
            matching_rules_validated: true,
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready: false,
            unresolved_blockers: vec![
                "real exchange REST/WebSocket adapter not implemented".to_owned(),
                "exchange sandbox/live response calibration missing".to_owned(),
                "credential scope and production rate-limit validation missing".to_owned(),
            ],
        };
        record.validate()?;
        Ok(record)
    }

    /// Validate report invariants before callers persist or display it.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != CEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(CexConnectorViolation::new(
                "CEX_FRAMEWORK_VERSION_MISMATCH",
                "CEX exchange fixture validation has an unexpected framework version",
            ));
        }
        validate_id("connector", &self.connector_name, &mut violations);
        validate_id(
            "fixture validation request",
            &self.request_id,
            &mut violations,
        );
        validate_id(
            "fixture validation client order",
            &self.client_order_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_FIXTURE_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if !self.profile_policy_validated || !self.matching_rules_validated {
            violations.push(CexConnectorViolation::new(
                "CEX_FIXTURE_VALIDATION_INCOMPLETE",
                "CEX exchange fixture validation requires profile, policy, and matching checks",
            ));
        }
        if self.rest_call_performed
            || self.websocket_connection_opened
            || self.credentials_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_FIXTURE_EXTERNAL_SIDE_EFFECT",
                "CEX exchange fixture validation must not perform network, credential, submission, live execution, or production-ready side effects",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexLiveAdapterBoundaryReviewRequest {
    /// Validate request shape and side-effect flags before building a report.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id("live adapter review", &self.review_id, &mut violations);
        validate_id(
            "live adapter connector",
            &self.connector_name,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if self.validated_at_unix_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_LIVE_ADAPTER_REVIEW_TIMESTAMP_REQUIRED",
                "CEX live adapter boundary review timestamp is required",
            ));
        }
        if self.credential_material_loaded
            || self.rest_call_performed
            || self.websocket_connection_opened
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready_claimed
        {
            violations.push(CexConnectorViolation::new(
                "CEX_LIVE_ADAPTER_REVIEW_SIDE_EFFECT",
                "CEX live adapter boundary review must not load credentials, call exchanges, submit orders, execute live, or claim readiness",
            ));
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexLiveAdapterBoundaryReviewReport {
    /// Validate report invariants before callers persist or aggregate it.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != CEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(CexConnectorViolation::new(
                "CEX_FRAMEWORK_VERSION_MISMATCH",
                "CEX live adapter boundary review has an unexpected framework version",
            ));
        }
        validate_id("live adapter review", &self.review_id, &mut violations);
        validate_id(
            "live adapter connector",
            &self.connector_name,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if self.credential_material_loaded
            || self.rest_call_performed
            || self.websocket_connection_opened
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_LIVE_ADAPTER_REVIEW_SIDE_EFFECT",
                "CEX live adapter boundary report must remain side-effect free and not claim readiness",
            ));
        }
        if self.blocker_codes.is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_LIVE_ADAPTER_REVIEW_BLOCKERS_REQUIRED",
                "CEX live adapter boundary report must retain live-adapter blockers",
            ));
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

/// Review local CEX live-adapter implementation prerequisites without
/// performing any live adapter work.
///
/// This converts the previous loose "not implemented" boundary into a typed,
/// auditable report. It is still blocked until real sandbox/live exchange
/// evidence exists outside this local-only validator.
pub fn review_cex_live_adapter_boundary(
    request: CexLiveAdapterBoundaryReviewRequest,
) -> Result<CexLiveAdapterBoundaryReviewReport, CexConnectorError> {
    request.validate()?;
    let blocker_codes = cex_live_adapter_boundary_blockers(&request);
    let report = CexLiveAdapterBoundaryReviewReport {
        framework_version: CEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
        review_id: request.review_id,
        connector_name: request.connector_name,
        venue: request.venue,
        status: CexLiveAdapterBoundaryReviewStatus::BlockedPendingLiveAdapterImplementation,
        rest_request_plan_validated: request.rest_request_plan_validated,
        websocket_request_plan_validated: request.websocket_request_plan_validated,
        lifecycle_transcript_parsing_validated: request.lifecycle_transcript_parsing_validated,
        balance_snapshot_parsing_validated: request.balance_snapshot_parsing_validated,
        credential_scope_reviewed: request.credential_scope_reviewed,
        rate_limit_reviewed: request.rate_limit_reviewed,
        exchange_matching_rules_validated: request.exchange_matching_rules_validated,
        sandbox_order_lifecycle_evidence_available: request
            .sandbox_order_lifecycle_evidence_available,
        sandbox_balance_evidence_available: request.sandbox_balance_evidence_available,
        sandbox_cancel_evidence_available: request.sandbox_cancel_evidence_available,
        production_idempotency_evidence_available: request
            .production_idempotency_evidence_available,
        credential_material_loaded: false,
        rest_call_performed: false,
        websocket_connection_opened: false,
        external_submission_performed: false,
        live_execution_performed: false,
        production_ready: false,
        validated_at_unix_ms: request.validated_at_unix_ms,
        blocker_codes,
    };
    report.validate()?;
    Ok(report)
}

impl CexMarketDataRequestPlan {
    /// Build a Binance-style public depth REST request plan.
    pub fn binance_depth_rest(
        plan_id: impl Into<String>,
        venue: VenueRef,
        pair: MarketPair,
        limit: u16,
    ) -> Result<Self, CexConnectorError> {
        let symbol = binance_symbol(&pair);
        Self::new_rest(
            plan_id,
            CexExchangeMarketDataFormat::BinanceDepth,
            venue,
            pair,
            "GET",
            "/api/v3/depth",
            format!("symbol={symbol}&limit={limit}"),
        )
    }

    /// Build a Binance-style public depth WebSocket subscription plan.
    pub fn binance_depth_websocket(
        plan_id: impl Into<String>,
        venue: VenueRef,
        pair: MarketPair,
    ) -> Result<Self, CexConnectorError> {
        let stream = format!("{}@depth", binance_symbol(&pair).to_ascii_lowercase());
        Self::new_websocket(
            plan_id,
            CexExchangeMarketDataFormat::BinanceDepth,
            venue,
            pair,
            "depth",
            format!(r#"{{"method":"SUBSCRIBE","params":["{stream}"],"id":1}}"#),
        )
    }

    /// Build a Coinbase-style public product-book REST request plan.
    pub fn coinbase_product_book_rest(
        plan_id: impl Into<String>,
        venue: VenueRef,
        pair: MarketPair,
        level: u8,
    ) -> Result<Self, CexConnectorError> {
        let product = dash_symbol(&pair);
        Self::new_rest(
            plan_id,
            CexExchangeMarketDataFormat::CoinbaseProductBook,
            venue,
            pair,
            "GET",
            format!("/products/{product}/book"),
            format!("level={level}"),
        )
    }

    /// Build a Coinbase-style public level2 WebSocket subscription plan.
    pub fn coinbase_product_book_websocket(
        plan_id: impl Into<String>,
        venue: VenueRef,
        pair: MarketPair,
    ) -> Result<Self, CexConnectorError> {
        let product = dash_symbol(&pair);
        Self::new_websocket(
            plan_id,
            CexExchangeMarketDataFormat::CoinbaseProductBook,
            venue,
            pair,
            "level2",
            format!(r#"{{"type":"subscribe","product_ids":["{product}"],"channels":["level2"]}}"#),
        )
    }

    /// Build a Kraken-style public depth REST request plan.
    pub fn kraken_depth_rest(
        plan_id: impl Into<String>,
        venue: VenueRef,
        pair: MarketPair,
        count: u16,
    ) -> Result<Self, CexConnectorError> {
        let kraken_pair = kraken_symbol(&pair);
        Self::new_rest(
            plan_id,
            CexExchangeMarketDataFormat::KrakenDepth,
            venue,
            pair,
            "GET",
            "/0/public/Depth",
            format!("pair={kraken_pair}&count={count}"),
        )
    }

    /// Build a Kraken-style public book WebSocket subscription plan.
    pub fn kraken_depth_websocket(
        plan_id: impl Into<String>,
        venue: VenueRef,
        pair: MarketPair,
    ) -> Result<Self, CexConnectorError> {
        let kraken_pair = kraken_symbol(&pair);
        Self::new_websocket(
            plan_id,
            CexExchangeMarketDataFormat::KrakenDepth,
            venue,
            pair,
            "book",
            format!(
                r#"{{"event":"subscribe","pair":["{kraken_pair}"],"subscription":{{"name":"book"}}}}"#
            ),
        )
    }

    fn new_rest(
        plan_id: impl Into<String>,
        format: CexExchangeMarketDataFormat,
        venue: VenueRef,
        pair: MarketPair,
        method: impl Into<String>,
        path: impl Into<String>,
        query: impl Into<String>,
    ) -> Result<Self, CexConnectorError> {
        let plan = Self {
            plan_id: plan_id.into(),
            format,
            request_kind: CexMarketDataRequestKind::RestOrderBook,
            venue,
            pair,
            rest_method: Some(method.into()),
            rest_path: Some(path.into()),
            rest_query: Some(query.into()),
            websocket_channel: None,
            websocket_subscription_json: None,
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            live_execution_performed: false,
            production_ready: false,
        };
        plan.validate()?;
        Ok(plan)
    }

    fn new_websocket(
        plan_id: impl Into<String>,
        format: CexExchangeMarketDataFormat,
        venue: VenueRef,
        pair: MarketPair,
        channel: impl Into<String>,
        subscription_json: impl Into<String>,
    ) -> Result<Self, CexConnectorError> {
        let plan = Self {
            plan_id: plan_id.into(),
            format,
            request_kind: CexMarketDataRequestKind::WebSocketOrderBook,
            venue,
            pair,
            rest_method: None,
            rest_path: None,
            rest_query: None,
            websocket_channel: Some(channel.into()),
            websocket_subscription_json: Some(subscription_json.into()),
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            live_execution_performed: false,
            production_ready: false,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Validate local request-plan invariants and side-effect denial flags.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX market-data request plan",
            &self.plan_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_REQUEST_PLAN_PAIR_INVALID",
                source.to_string(),
            ));
        }

        match self.request_kind {
            CexMarketDataRequestKind::RestOrderBook => {
                if self.rest_method.as_deref() != Some("GET")
                    || empty_optional(self.rest_path.as_ref())
                    || empty_optional(self.rest_query.as_ref())
                    || self.websocket_channel.is_some()
                    || self.websocket_subscription_json.is_some()
                {
                    violations.push(CexConnectorViolation::new(
                        "CEX_REQUEST_PLAN_REST_SHAPE_INVALID",
                        "CEX REST market-data request plans require GET path/query and no WebSocket subscription fields",
                    ));
                }
            }
            CexMarketDataRequestKind::WebSocketOrderBook => {
                if empty_optional(self.websocket_channel.as_ref())
                    || empty_optional(self.websocket_subscription_json.as_ref())
                    || self.rest_method.is_some()
                    || self.rest_path.is_some()
                    || self.rest_query.is_some()
                {
                    violations.push(CexConnectorViolation::new(
                        "CEX_REQUEST_PLAN_WEBSOCKET_SHAPE_INVALID",
                        "CEX WebSocket market-data request plans require channel/subscription fields and no REST fields",
                    ));
                }
            }
        }

        if self.rest_call_performed
            || self.websocket_connection_opened
            || self.credentials_loaded
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_REQUEST_PLAN_EXTERNAL_SIDE_EFFECT",
                "CEX market-data request plans must not perform REST, WebSocket, credential, live-execution, or production-ready side effects",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Parse a caller-supplied local transcript through this request plan.
    pub fn parse_transcript(
        &self,
        transcript: &CexMockMarketDataTranscript,
    ) -> Result<OrderBookSnapshot, CexConnectorError> {
        self.validate()?;
        transcript.validate()?;
        let mut violations = Vec::new();
        if self.format != transcript.format {
            violations.push(CexConnectorViolation::new(
                "CEX_REQUEST_PLAN_TRANSCRIPT_FORMAT_MISMATCH",
                "CEX request plan and transcript formats must match",
            ));
        }
        if self.venue != transcript.venue {
            violations.push(CexConnectorViolation::new(
                "CEX_REQUEST_PLAN_TRANSCRIPT_VENUE_MISMATCH",
                "CEX request plan and transcript venues must match",
            ));
        }
        if self.pair != transcript.pair {
            violations.push(CexConnectorViolation::new(
                "CEX_REQUEST_PLAN_TRANSCRIPT_PAIR_MISMATCH",
                "CEX request plan and transcript pairs must match",
            ));
        }
        if !violations.is_empty() {
            return Err(CexConnectorError::ValidationFailed { violations });
        }
        transcript.parse_order_book_snapshot()
    }
}

impl CexMockMarketDataTranscript {
    /// Construct a validated local exchange-specific market-data transcript.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transcript_id: impl Into<String>,
        format: CexExchangeMarketDataFormat,
        venue: VenueRef,
        pair: MarketPair,
        payload_json: impl Into<String>,
        captured_at_unix_ms: u64,
        received_at_unix_ms: u64,
    ) -> Result<Self, CexConnectorError> {
        let transcript = Self {
            transcript_id: transcript_id.into(),
            format,
            venue,
            pair,
            payload_json: payload_json.into(),
            captured_at_unix_ms,
            received_at_unix_ms,
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            live_execution_performed: false,
        };
        transcript.validate()?;
        Ok(transcript)
    }

    /// Validate local transcript invariants and side-effect denial flags.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX market-data transcript",
            &self.transcript_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_TRANSCRIPT_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if self.payload_json.trim().is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_TRANSCRIPT_PAYLOAD_REQUIRED",
                "CEX market-data transcript payload must be non-empty",
            ));
        }
        if self.captured_at_unix_ms == 0 || self.received_at_unix_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_TRANSCRIPT_TIMESTAMP_REQUIRED",
                "CEX market-data transcript timestamps are required",
            ));
        }
        if self.captured_at_unix_ms > self.received_at_unix_ms {
            violations.push(CexConnectorViolation::new(
                "CEX_TRANSCRIPT_TIMESTAMP_ORDER_INVALID",
                "CEX market-data transcript capture time must not be after receive time",
            ));
        }
        if self.rest_call_performed
            || self.websocket_connection_opened
            || self.credentials_loaded
            || self.live_execution_performed
        {
            violations.push(CexConnectorViolation::new(
                "CEX_TRANSCRIPT_EXTERNAL_SIDE_EFFECT",
                "CEX market-data transcript parsing must not perform REST, WebSocket, credential, or live-execution side effects",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Parse this transcript into a normalized order-book snapshot.
    pub fn parse_order_book_snapshot(&self) -> Result<OrderBookSnapshot, CexConnectorError> {
        self.validate()?;
        let value: serde_json::Value =
            serde_json::from_str(&self.payload_json).map_err(|error| {
                CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new_owned(
                        "CEX_TRANSCRIPT_JSON_INVALID",
                        format!("CEX market-data transcript JSON is invalid: {error}"),
                    )],
                }
            })?;

        let parsed = match self.format {
            CexExchangeMarketDataFormat::BinanceDepth => parse_binance_depth_payload(&value)?,
            CexExchangeMarketDataFormat::CoinbaseProductBook => {
                parse_coinbase_product_book_payload(&value)?
            }
            CexExchangeMarketDataFormat::KrakenDepth => parse_kraken_depth_payload(&value)?,
        };

        let snapshot = OrderBookSnapshot {
            id: format!("{}-normalized-book", self.transcript_id),
            venue: self.venue.clone(),
            pair: self.pair.clone(),
            captured_at_unix_ms: self.captured_at_unix_ms,
            received_at_unix_ms: self.received_at_unix_ms,
            bids: parsed.bids,
            asks: parsed.asks,
            source_sequence: parsed.source_sequence,
        };
        snapshot
            .validate()
            .map_err(|error| CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new_owned(
                    "CEX_TRANSCRIPT_ORDER_BOOK_INVALID",
                    error.to_string(),
                )],
            })?;
        Ok(snapshot)
    }

    /// Parse this transcript into a normalized top-of-book quote.
    pub fn parse_top_of_book(&self) -> Result<NormalizedQuote, CexConnectorError> {
        self.parse_order_book_snapshot()?
            .to_quote()
            .map_err(|error| CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new_owned(
                    "CEX_TRANSCRIPT_QUOTE_INVALID",
                    error.to_string(),
                )],
            })
    }
}

impl CexRateLimitObservation {
    /// Construct a local rate-limit observation after validation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        observation_id: impl Into<String>,
        venue: VenueRef,
        scope: CexRateLimitScope,
        max_requests_per_window: u32,
        window_ms: u64,
        observed_requests_in_window: u32,
        retry_after_ms: Option<u64>,
        provider_rate_limited: bool,
    ) -> Result<Self, CexConnectorError> {
        let observation = Self {
            observation_id: observation_id.into(),
            venue,
            scope,
            max_requests_per_window,
            window_ms,
            observed_requests_in_window,
            retry_after_ms,
            provider_rate_limited,
            live_provider_call_performed: false,
            websocket_connection_opened: false,
            credential_loaded: false,
            live_execution_performed: false,
        };
        observation.validate()?;
        Ok(observation)
    }

    /// Validate local observation invariants and side-effect denial flags.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX rate-limit observation",
            &self.observation_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if self.max_requests_per_window == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_MAX_REQUESTS_ZERO",
                "CEX rate-limit max requests must be positive",
            ));
        }
        if self.window_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_WINDOW_ZERO",
                "CEX rate-limit window must be positive",
            ));
        }
        if self.retry_after_ms == Some(0) {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_RETRY_AFTER_ZERO",
                "CEX rate-limit retry-after must be positive when present",
            ));
        }
        if self.live_provider_call_performed
            || self.websocket_connection_opened
            || self.credential_loaded
            || self.live_execution_performed
        {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_EXTERNAL_SIDE_EFFECT",
                "CEX rate-limit validation must not perform provider calls, WebSocket opens, credential loading, or live execution",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexRateLimitReport {
    /// Validate report invariants before callers persist or display it.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX rate-limit report",
            &self.observation_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if self.max_requests_per_window == 0 || self.window_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_REPORT_LIMIT_INVALID",
                "CEX rate-limit report requires positive request and window limits",
            ));
        }
        let expected_remaining = self
            .max_requests_per_window
            .saturating_sub(self.observed_requests_in_window);
        if self.remaining_requests_in_window != expected_remaining {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_REMAINING_INCOHERENT",
                "CEX rate-limit remaining budget must match max minus observed requests",
            ));
        }
        let should_block = self.local_budget_exhausted
            || self.provider_rate_limited
            || self.live_provider_call_performed
            || self.websocket_connection_opened
            || self.credential_loaded
            || self.live_execution_performed;
        if should_block && self.status != CexRateLimitStatus::Blocked {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_STATUS_SHOULD_BLOCK",
                "CEX rate-limit report with exhausted budget or side effects must block",
            ));
        }
        if !should_block && self.status != CexRateLimitStatus::ReadyForLocalReview {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_STATUS_SHOULD_BE_READY",
                "CEX rate-limit report within local budget must be ready for local review",
            ));
        }
        if self.production_ready {
            violations.push(CexConnectorViolation::new(
                "CEX_RATE_LIMIT_PRODUCTION_READY_FORBIDDEN",
                "CEX rate-limit validation must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexCredentialScopeReviewInput {
    /// Construct a local credential/API-scope review input after validation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        review_id: impl Into<String>,
        venue: VenueRef,
        credential_reference: SecretRef,
        required_permissions: Vec<CexCredentialPermission>,
        observed_permissions: Vec<CexCredentialPermission>,
        forbidden_permissions: Vec<CexCredentialPermission>,
        reviewed_at_unix_ms: u64,
        now_unix_ms: u64,
        max_review_age_ms: u64,
    ) -> Result<Self, CexConnectorError> {
        let input = Self {
            review_id: review_id.into(),
            venue,
            credential_reference,
            required_permissions,
            observed_permissions,
            forbidden_permissions,
            reviewed_at_unix_ms,
            now_unix_ms,
            max_review_age_ms,
            fee_schedule_reviewed: true,
            rate_limit_documentation_reviewed: true,
            terms_of_service_reviewed: true,
            jurisdiction_reviewed: true,
            api_capabilities_reviewed: true,
            incident_reputation_reviewed: true,
            secret_material_loaded: false,
            credential_plaintext_seen: false,
            live_provider_call_performed: false,
            account_state_queried: false,
            live_execution_performed: false,
            production_ready_claimed: false,
        };
        input.validate()?;
        Ok(input)
    }

    /// Validate input invariants without loading or resolving the secret reference.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX credential-scope review",
            &self.review_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(error) = self.credential_reference.validate_reference() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_CREDENTIAL_REFERENCE_INVALID",
                error.to_string(),
            ));
        }
        if self.credential_reference.is_disabled() {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_REFERENCE_DISABLED",
                "CEX credential-scope review requires a non-disabled secret reference",
            ));
        }
        if self.required_permissions.is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_REQUIRED_PERMISSIONS_EMPTY",
                "CEX credential-scope review requires at least one required permission",
            ));
        }
        if self.max_review_age_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_MAX_REVIEW_AGE_ZERO",
                "CEX credential-scope review max age must be positive",
            ));
        }
        if self.now_unix_ms < self.reviewed_at_unix_ms {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_REVIEW_TIME_IN_FUTURE",
                "CEX credential-scope review timestamp cannot be in the future",
            ));
        }
        if self.secret_material_loaded
            || self.credential_plaintext_seen
            || self.live_provider_call_performed
            || self.account_state_queried
            || self.live_execution_performed
        {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_SCOPE_EXTERNAL_SIDE_EFFECT",
                "CEX credential-scope review must not load secrets, see plaintext, call providers, query accounts, or execute live",
            ));
        }
        if self.production_ready_claimed {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_SCOPE_PRODUCTION_READY_FORBIDDEN",
                "CEX credential-scope review must not claim production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexCredentialScopeReviewReport {
    /// Validate report invariants before callers persist or display it.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX credential-scope review report",
            &self.review_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        let should_block = !self.missing_required_permissions.is_empty()
            || !self.forbidden_permissions_present.is_empty()
            || !self.credential_reference_validated
            || self.stale_review
            || !self.governance_review_passed
            || self.secret_material_loaded
            || self.credential_plaintext_seen
            || self.live_provider_call_performed
            || self.account_state_queried
            || self.live_execution_performed;
        let expected_governance_review_passed = self.fee_schedule_reviewed
            && self.rate_limit_documentation_reviewed
            && self.terms_of_service_reviewed
            && self.jurisdiction_reviewed
            && self.api_capabilities_reviewed
            && self.incident_reputation_reviewed;
        if self.governance_review_passed != expected_governance_review_passed {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_SCOPE_GOVERNANCE_STATUS_INCOHERENT",
                "CEX credential-scope governance summary must match the detailed governance review flags",
            ));
        }
        if should_block && self.status != CexCredentialScopeReviewStatus::Blocked {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_SCOPE_STATUS_SHOULD_BLOCK",
                "CEX credential-scope report with missing permissions, forbidden permissions, stale review, invalid reference, or side effects must block",
            ));
        }
        if !should_block && self.status != CexCredentialScopeReviewStatus::ReadyForLocalReview {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_SCOPE_STATUS_SHOULD_BE_READY",
                "CEX credential-scope report without local blockers must be ready for local review",
            ));
        }
        if self.production_ready {
            violations.push(CexConnectorViolation::new(
                "CEX_CREDENTIAL_SCOPE_REPORT_PRODUCTION_READY_FORBIDDEN",
                "CEX credential-scope report must not approve production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

/// Validate local CEX credential/API-scope metadata without loading secrets.
pub fn validate_cex_credential_scope_review(
    input: CexCredentialScopeReviewInput,
) -> Result<CexCredentialScopeReviewReport, CexConnectorError> {
    input.validate()?;
    let observed: HashSet<_> = input.observed_permissions.iter().copied().collect();
    let forbidden: HashSet<_> = input
        .forbidden_permissions
        .iter()
        .copied()
        .chain(
            input
                .observed_permissions
                .iter()
                .copied()
                .filter(|permission| permission.is_forbidden_live_funds_permission()),
        )
        .collect();
    let missing_required_permissions = input
        .required_permissions
        .iter()
        .copied()
        .filter(|permission| !observed.contains(permission))
        .collect::<Vec<_>>();
    let forbidden_permissions_present = input
        .observed_permissions
        .iter()
        .copied()
        .filter(|permission| forbidden.contains(permission))
        .collect::<Vec<_>>();
    let stale_review =
        input.now_unix_ms.saturating_sub(input.reviewed_at_unix_ms) > input.max_review_age_ms;
    let governance_review_passed = input.fee_schedule_reviewed
        && input.rate_limit_documentation_reviewed
        && input.terms_of_service_reviewed
        && input.jurisdiction_reviewed
        && input.api_capabilities_reviewed
        && input.incident_reputation_reviewed;
    let blocked = !missing_required_permissions.is_empty()
        || !forbidden_permissions_present.is_empty()
        || stale_review
        || !governance_review_passed
        || input.secret_material_loaded
        || input.credential_plaintext_seen
        || input.live_provider_call_performed
        || input.account_state_queried
        || input.live_execution_performed
        || input.production_ready_claimed;
    let mut violation_codes = Vec::new();
    push_code_if(
        &mut violation_codes,
        !missing_required_permissions.is_empty(),
        "CEX_CREDENTIAL_SCOPE_MISSING_REQUIRED_PERMISSION",
    );
    push_code_if(
        &mut violation_codes,
        !forbidden_permissions_present.is_empty(),
        "CEX_CREDENTIAL_SCOPE_FORBIDDEN_PERMISSION_PRESENT",
    );
    push_code_if(
        &mut violation_codes,
        stale_review,
        "CEX_CREDENTIAL_SCOPE_REVIEW_STALE",
    );
    push_code_if(
        &mut violation_codes,
        !input.fee_schedule_reviewed,
        "CEX_CREDENTIAL_SCOPE_FEE_REVIEW_MISSING",
    );
    push_code_if(
        &mut violation_codes,
        !input.rate_limit_documentation_reviewed,
        "CEX_CREDENTIAL_SCOPE_RATE_LIMIT_DOCUMENTATION_MISSING",
    );
    push_code_if(
        &mut violation_codes,
        !input.terms_of_service_reviewed,
        "CEX_CREDENTIAL_SCOPE_TERMS_REVIEW_MISSING",
    );
    push_code_if(
        &mut violation_codes,
        !input.jurisdiction_reviewed,
        "CEX_CREDENTIAL_SCOPE_JURISDICTION_REVIEW_MISSING",
    );
    push_code_if(
        &mut violation_codes,
        !input.api_capabilities_reviewed,
        "CEX_CREDENTIAL_SCOPE_API_CAPABILITIES_REVIEW_MISSING",
    );
    push_code_if(
        &mut violation_codes,
        !input.incident_reputation_reviewed,
        "CEX_CREDENTIAL_SCOPE_INCIDENT_REPUTATION_REVIEW_MISSING",
    );
    push_code_if(
        &mut violation_codes,
        input.secret_material_loaded,
        "CEX_CREDENTIAL_SCOPE_SECRET_MATERIAL_LOADED",
    );
    push_code_if(
        &mut violation_codes,
        input.credential_plaintext_seen,
        "CEX_CREDENTIAL_SCOPE_PLAINTEXT_SEEN",
    );
    push_code_if(
        &mut violation_codes,
        input.live_provider_call_performed,
        "CEX_CREDENTIAL_SCOPE_LIVE_PROVIDER_CALL",
    );
    push_code_if(
        &mut violation_codes,
        input.account_state_queried,
        "CEX_CREDENTIAL_SCOPE_ACCOUNT_QUERY",
    );
    push_code_if(
        &mut violation_codes,
        input.live_execution_performed,
        "CEX_CREDENTIAL_SCOPE_LIVE_EXECUTION",
    );
    push_code_if(
        &mut violation_codes,
        input.production_ready_claimed,
        "CEX_CREDENTIAL_SCOPE_PRODUCTION_READY_CLAIMED",
    );

    let report = CexCredentialScopeReviewReport {
        review_id: input.review_id,
        venue: input.venue,
        status: if blocked {
            CexCredentialScopeReviewStatus::Blocked
        } else {
            CexCredentialScopeReviewStatus::ReadyForLocalReview
        },
        required_permissions: input.required_permissions,
        observed_permissions: input.observed_permissions,
        forbidden_permissions: input.forbidden_permissions,
        missing_required_permissions,
        forbidden_permissions_present,
        credential_reference_validated: true,
        stale_review,
        fee_schedule_reviewed: input.fee_schedule_reviewed,
        rate_limit_documentation_reviewed: input.rate_limit_documentation_reviewed,
        terms_of_service_reviewed: input.terms_of_service_reviewed,
        jurisdiction_reviewed: input.jurisdiction_reviewed,
        api_capabilities_reviewed: input.api_capabilities_reviewed,
        incident_reputation_reviewed: input.incident_reputation_reviewed,
        governance_review_passed,
        secret_material_loaded: input.secret_material_loaded,
        credential_plaintext_seen: input.credential_plaintext_seen,
        live_provider_call_performed: input.live_provider_call_performed,
        account_state_queried: input.account_state_queried,
        live_execution_performed: input.live_execution_performed,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
}

/// Validate a local CEX rate-limit observation without provider calls.
pub fn validate_cex_rate_limit(
    observation: CexRateLimitObservation,
) -> Result<CexRateLimitReport, CexConnectorError> {
    observation.validate()?;
    let local_budget_exhausted =
        observation.observed_requests_in_window >= observation.max_requests_per_window;
    let blocked = local_budget_exhausted
        || observation.provider_rate_limited
        || observation.live_provider_call_performed
        || observation.websocket_connection_opened
        || observation.credential_loaded
        || observation.live_execution_performed;
    let mut violation_codes = Vec::new();
    push_code_if(
        &mut violation_codes,
        local_budget_exhausted,
        "CEX_RATE_LIMIT_LOCAL_BUDGET_EXHAUSTED",
    );
    push_code_if(
        &mut violation_codes,
        observation.provider_rate_limited,
        "CEX_RATE_LIMIT_PROVIDER_SIGNALED",
    );
    push_code_if(
        &mut violation_codes,
        observation.live_provider_call_performed,
        "CEX_RATE_LIMIT_LIVE_PROVIDER_CALL",
    );
    push_code_if(
        &mut violation_codes,
        observation.websocket_connection_opened,
        "CEX_RATE_LIMIT_WEBSOCKET_OPENED",
    );
    push_code_if(
        &mut violation_codes,
        observation.credential_loaded,
        "CEX_RATE_LIMIT_CREDENTIAL_LOADED",
    );
    push_code_if(
        &mut violation_codes,
        observation.live_execution_performed,
        "CEX_RATE_LIMIT_LIVE_EXECUTION",
    );

    let report = CexRateLimitReport {
        observation_id: observation.observation_id,
        venue: observation.venue,
        scope: observation.scope,
        status: if blocked {
            CexRateLimitStatus::Blocked
        } else {
            CexRateLimitStatus::ReadyForLocalReview
        },
        max_requests_per_window: observation.max_requests_per_window,
        window_ms: observation.window_ms,
        observed_requests_in_window: observation.observed_requests_in_window,
        remaining_requests_in_window: observation
            .max_requests_per_window
            .saturating_sub(observation.observed_requests_in_window),
        retry_after_ms: observation.retry_after_ms,
        local_budget_exhausted,
        provider_rate_limited: observation.provider_rate_limited,
        live_provider_call_performed: observation.live_provider_call_performed,
        websocket_connection_opened: observation.websocket_connection_opened,
        credential_loaded: observation.credential_loaded,
        live_execution_performed: observation.live_execution_performed,
        production_ready: false,
        violation_codes,
    };
    report.validate()?;
    Ok(report)
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

/// Durable local CEX framework order-validation record.
///
/// This records policy-gated local validation only. It never submits orders,
/// reads balances, calls an exchange, or claims sandbox/live execution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexOrderValidationRecord {
    /// CEX framework version that produced the record.
    pub framework_version: String,
    /// Original request id.
    pub request_id: String,
    /// Client order id for future idempotency checks.
    pub client_order_id: String,
    /// Strategy profile id that produced the request.
    pub strategy_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target venue.
    pub venue: VenueRef,
    /// Market pair.
    pub pair: MarketPair,
    /// Requested base quantity.
    pub quantity_base: f64,
    /// Order status at the local framework boundary.
    pub status: CexOrderStatus,
    /// Whether policy approved the converted intent.
    pub policy_approved: bool,
    /// Trust-contract version for the approval.
    pub trust_contract_version: String,
    /// Whether an external order submission occurred. Always false here.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Non-secret unresolved blockers that keep this local-only.
    pub unresolved_blockers: Vec<String>,
}

/// Local/mock CEX order response consumed by lifecycle reconciliation tests.
///
/// This is a deterministic fixture record only. It models future exchange
/// adapter responses without submitting orders, opening sockets, loading
/// credentials, or calling an exchange.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexOrderLifecycleResponse {
    /// Stable local response id.
    pub id: String,
    /// Request id this response belongs to.
    pub request_id: String,
    /// Client-side id used for idempotency checks.
    pub client_order_id: String,
    /// Optional exchange order id from a mocked adapter response.
    pub exchange_order_id: Option<String>,
    /// Response status.
    pub status: CexOrderStatus,
    /// Fill quantity delta in base units.
    pub fill_quantity_base_delta: f64,
    /// Optional fill price in quote units.
    pub fill_price_quote: Option<f64>,
    /// Fee delta in quote units.
    pub fee_quote_delta: f64,
    /// Whether this response was produced by a local/mock fixture.
    pub locally_simulated: bool,
    /// Whether an external submission occurred. Must remain false.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Local response timestamp.
    pub occurred_at_unix_ms: u64,
}

/// Local exchange-shaped CEX order lifecycle transcript format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexOrderLifecycleTranscriptFormat {
    /// Binance-style execution report payload.
    BinanceExecutionReport,
    /// Coinbase-style order event payload.
    CoinbaseOrderEvent,
    /// Kraken-style order status payload.
    KrakenOrderStatus,
}

/// Caller-supplied local CEX order lifecycle transcript.
///
/// This parser is intentionally local-only. It normalizes exchange-shaped JSON
/// into `CexOrderLifecycleResponse` records for reconciliation tests without
/// opening sockets, calling REST APIs, loading credentials, or submitting orders.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexOrderLifecycleTranscript {
    /// Stable local transcript id.
    pub transcript_id: String,
    /// Exchange-shaped transcript format.
    pub format: CexOrderLifecycleTranscriptFormat,
    /// Venue represented by the transcript.
    pub venue: VenueRef,
    /// Market pair represented by the transcript.
    pub pair: MarketPair,
    /// Raw local JSON payload.
    pub payload_json: String,
    /// Local capture timestamp.
    pub captured_at_unix_ms: u64,
    /// Local receive timestamp.
    pub received_at_unix_ms: u64,
    /// Whether a REST call occurred. Must remain false.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection was opened. Must remain false.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Must remain false.
    pub credentials_loaded: bool,
    /// Whether an external order submission occurred. Must remain false.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether this transcript claims production readiness. Must remain false.
    pub production_ready: bool,
}

/// Local exchange-shaped CEX balance snapshot transcript format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CexBalanceSnapshotTranscriptFormat {
    /// Binance-style account balances payload.
    BinanceAccountBalances,
    /// Coinbase-style account list payload.
    CoinbaseAccounts,
    /// Kraken-style balance result payload.
    KrakenBalance,
}

/// Normalized local CEX asset balance from a caller-supplied transcript.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexAssetBalanceSnapshot {
    /// Asset symbol.
    pub asset: String,
    /// Available amount.
    pub available: f64,
    /// Total amount, including unavailable/held/locked units.
    pub total: f64,
}

/// Caller-supplied local CEX balance snapshot transcript.
///
/// This does not query accounts, load credentials, or call an exchange. It only
/// normalizes local JSON that an operator or fixture supplied.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexBalanceSnapshotTranscript {
    /// Stable local transcript id.
    pub transcript_id: String,
    /// Exchange-shaped balance transcript format.
    pub format: CexBalanceSnapshotTranscriptFormat,
    /// Venue represented by the transcript.
    pub venue: VenueRef,
    /// Raw local JSON payload.
    pub payload_json: String,
    /// Local capture timestamp.
    pub captured_at_unix_ms: u64,
    /// Local receive timestamp.
    pub received_at_unix_ms: u64,
    /// Whether a REST call occurred. Must remain false.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection was opened. Must remain false.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Must remain false.
    pub credentials_loaded: bool,
    /// Whether account state was queried live. Must remain false.
    pub account_state_queried: bool,
    /// Whether live execution occurred. Must remain false.
    pub live_execution_performed: bool,
    /// Whether this transcript claims production readiness. Must remain false.
    pub production_ready: bool,
}

/// Normalized local CEX balance snapshot record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexBalanceSnapshotRecord {
    /// CEX framework version that produced the record.
    pub framework_version: String,
    /// Source transcript id.
    pub transcript_id: String,
    /// Venue represented by the snapshot.
    pub venue: VenueRef,
    /// Normalized local balances.
    pub balances: Vec<CexAssetBalanceSnapshot>,
    /// Whether a REST call occurred. Always false here.
    pub rest_call_performed: bool,
    /// Whether a WebSocket connection was opened. Always false here.
    pub websocket_connection_opened: bool,
    /// Whether credentials were loaded. Always false here.
    pub credentials_loaded: bool,
    /// Whether account state was queried live. Always false here.
    pub account_state_queried: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this record proves production readiness. Always false here.
    pub production_ready: bool,
}

/// Local CEX order lifecycle reconciliation summary.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CexOrderLifecycleRecord {
    /// CEX framework version that produced the record.
    pub framework_version: String,
    /// Original request id.
    pub request_id: String,
    /// Client order id for idempotency checks.
    pub client_order_id: String,
    /// Strategy profile id that produced the request.
    pub strategy_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target venue.
    pub venue: VenueRef,
    /// Market pair.
    pub pair: MarketPair,
    /// Final reconciled local/mock order status.
    pub final_status: CexOrderStatus,
    /// Number of local/mock response transitions reconciled.
    pub transition_count: usize,
    /// Number of fill-bearing response records.
    pub fill_count: usize,
    /// Total reconciled filled quantity in base units.
    pub filled_quantity_base: f64,
    /// Remaining quantity in base units.
    pub remaining_quantity_base: f64,
    /// Average fill price in quote units, if any quantity filled.
    pub average_fill_price_quote: Option<f64>,
    /// Total reconciled fee in quote units.
    pub total_fee_quote: f64,
    /// Whether duplicate client-order-id records were rejected separately.
    pub duplicate_client_order_id_rejected: bool,
    /// Whether every mocked response was audit/state eligible.
    pub responses_reconciled: bool,
    /// Whether an external order submission occurred. Always false here.
    pub external_submission_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this record proves production readiness. Always false here.
    pub production_ready: bool,
    /// Non-secret unresolved blockers that keep this local-only.
    pub unresolved_blockers: Vec<String>,
}

impl CexOrderValidationRecord {
    /// Build a durable local record from an approved framework request.
    pub fn from_approved_request(
        request: &CexOrderRequest,
        approval: &PolicyApproval,
    ) -> Result<Self, CexConnectorError> {
        request.validate()?;
        if approval.intent_id != request.id {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new(
                    "CEX_APPROVAL_INTENT_MISMATCH",
                    "policy approval intent id must match CEX request id",
                )],
            });
        }
        if approval.approved_scope != request.scope {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new(
                    "CEX_APPROVAL_SCOPE_MISMATCH",
                    "policy approval scope must match CEX request scope",
                )],
            });
        }

        let record = Self {
            framework_version: CEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            request_id: request.id.clone(),
            client_order_id: request.client_order_id.clone(),
            strategy_id: request.strategy_id.clone(),
            scope: request.scope,
            venue: request.venue.clone(),
            pair: request.pair.clone(),
            quantity_base: request.quantity_base,
            status: CexOrderStatus::LocallyValidated,
            policy_approved: true,
            trust_contract_version: approval.trust_contract_version.to_owned(),
            external_submission_performed: false,
            live_execution_performed: false,
            unresolved_blockers: vec![
                "exchange-specific adapter validation missing".to_owned(),
                "sandbox/live exchange response validation missing".to_owned(),
                "production restart recovery and fill reconciliation missing".to_owned(),
            ],
        };
        record.validate()?;
        Ok(record)
    }

    /// Validate record invariants before audit/state persistence.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != CEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(CexConnectorViolation::new(
                "CEX_FRAMEWORK_VERSION_MISMATCH",
                "CEX validation record has an unexpected framework version",
            ));
        }
        validate_id("order validation record", &self.request_id, &mut violations);
        validate_id("client order", &self.client_order_id, &mut violations);
        validate_id("strategy", &self.strategy_id, &mut violations);
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_RECORD_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if !is_positive_finite(self.quantity_base) {
            violations.push(CexConnectorViolation::new(
                "CEX_RECORD_QUANTITY_INVALID",
                "CEX validation record quantity must be positive and finite",
            ));
        }
        if self.status != CexOrderStatus::LocallyValidated {
            violations.push(CexConnectorViolation::new(
                "CEX_RECORD_STATUS_NOT_LOCAL",
                "CEX validation record must remain locally validated",
            ));
        }
        if !self.policy_approved || self.trust_contract_version.trim().is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_POLICY_APPROVAL_REQUIRED",
                "CEX validation record requires a local policy approval",
            ));
        }
        if self.external_submission_performed || self.live_execution_performed {
            violations.push(CexConnectorViolation::new(
                "CEX_RECORD_EXTERNAL_SIDE_EFFECT",
                "CEX validation record must not include external submission or live execution",
            ));
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexOrderLifecycleResponse {
    /// Validate a local/mock CEX response before reconciliation.
    pub fn validate(&self, validation: &CexOrderValidationRecord) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id("CEX lifecycle response", &self.id, &mut violations);
        if self.request_id != validation.request_id {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_REQUEST_MISMATCH",
                "CEX lifecycle response request id must match validation record",
            ));
        }
        if self.client_order_id != validation.client_order_id {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_CLIENT_ORDER_MISMATCH",
                "CEX lifecycle response client order id must match validation record",
            ));
        }
        if matches!(
            self.status,
            CexOrderStatus::Accepted
                | CexOrderStatus::Filled
                | CexOrderStatus::PartiallyFilled
                | CexOrderStatus::Cancelled
        ) && self
            .exchange_order_id
            .as_ref()
            .map_or(true, |id| id.trim().is_empty())
        {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_EXCHANGE_ORDER_ID_REQUIRED",
                "accepted, filled, partially-filled, or cancelled responses require an exchange order id",
            ));
        }
        if !self.fill_quantity_base_delta.is_finite() || self.fill_quantity_base_delta < 0.0 {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_FILL_QUANTITY_INVALID",
                "CEX lifecycle response fill quantity must be finite and non-negative",
            ));
        }
        if !self.fee_quote_delta.is_finite() || self.fee_quote_delta < 0.0 {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_FEE_INVALID",
                "CEX lifecycle response fee must be finite and non-negative",
            ));
        }
        if self.fill_quantity_base_delta > 0.0 {
            match self.fill_price_quote {
                Some(price) if is_positive_finite(price) => {}
                Some(_) | None => violations.push(CexConnectorViolation::new(
                    "CEX_RESPONSE_FILL_PRICE_REQUIRED",
                    "fill-bearing CEX lifecycle responses require a positive finite fill price",
                )),
            }
        }
        if !self.locally_simulated
            || self.external_submission_performed
            || self.live_execution_performed
        {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_EXTERNAL_SIDE_EFFECT",
                "CEX lifecycle responses must be local/mock records without external submission or live execution",
            ));
        }
        if self.occurred_at_unix_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_RESPONSE_TIMESTAMP_REQUIRED",
                "CEX lifecycle response timestamp is required",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexOrderLifecycleTranscript {
    /// Construct a validated local exchange-shaped order lifecycle transcript.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transcript_id: impl Into<String>,
        format: CexOrderLifecycleTranscriptFormat,
        venue: VenueRef,
        pair: MarketPair,
        payload_json: impl Into<String>,
        captured_at_unix_ms: u64,
        received_at_unix_ms: u64,
    ) -> Result<Self, CexConnectorError> {
        let transcript = Self {
            transcript_id: transcript_id.into(),
            format,
            venue,
            pair,
            payload_json: payload_json.into(),
            captured_at_unix_ms,
            received_at_unix_ms,
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        transcript.validate()?;
        Ok(transcript)
    }

    /// Validate local transcript invariants and side-effect denial flags.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX lifecycle transcript",
            &self.transcript_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_LIFECYCLE_TRANSCRIPT_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if self.payload_json.trim().is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_TRANSCRIPT_PAYLOAD_REQUIRED",
                "CEX lifecycle transcript payload must be non-empty",
            ));
        }
        if self.captured_at_unix_ms == 0 || self.received_at_unix_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_TRANSCRIPT_TIMESTAMP_REQUIRED",
                "CEX lifecycle transcript timestamps are required",
            ));
        }
        if self.captured_at_unix_ms > self.received_at_unix_ms {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_TRANSCRIPT_TIME_ORDER_INVALID",
                "CEX lifecycle transcript capture time must not be after receive time",
            ));
        }
        if self.rest_call_performed
            || self.websocket_connection_opened
            || self.credentials_loaded
            || self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_TRANSCRIPT_EXTERNAL_SIDE_EFFECT",
                "CEX lifecycle transcript parsing must not perform REST, WebSocket, credential, submission, or live-execution side effects",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Parse this local transcript into a lifecycle response.
    pub fn parse_lifecycle_response(
        &self,
        validation: &CexOrderValidationRecord,
    ) -> Result<CexOrderLifecycleResponse, CexConnectorError> {
        self.validate()?;
        validation.validate()?;
        if !same_venue(&self.venue, &validation.venue) {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new(
                    "CEX_LIFECYCLE_TRANSCRIPT_VENUE_MISMATCH",
                    "CEX lifecycle transcript venue must match validation record",
                )],
            });
        }
        if self.pair != validation.pair {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new(
                    "CEX_LIFECYCLE_TRANSCRIPT_PAIR_MISMATCH",
                    "CEX lifecycle transcript pair must match validation record",
                )],
            });
        }

        let payload: Value = serde_json::from_str(&self.payload_json).map_err(|error| {
            CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new_owned(
                    "CEX_LIFECYCLE_TRANSCRIPT_JSON_INVALID",
                    format!("CEX lifecycle transcript JSON is invalid: {error}"),
                )],
            }
        })?;

        let parsed = match self.format {
            CexOrderLifecycleTranscriptFormat::BinanceExecutionReport => {
                parse_binance_lifecycle_payload(&payload)?
            }
            CexOrderLifecycleTranscriptFormat::CoinbaseOrderEvent => {
                parse_coinbase_lifecycle_payload(&payload)?
            }
            CexOrderLifecycleTranscriptFormat::KrakenOrderStatus => {
                parse_kraken_lifecycle_payload(&payload)?
            }
        };

        let response = CexOrderLifecycleResponse {
            id: self.transcript_id.clone(),
            request_id: validation.request_id.clone(),
            client_order_id: parsed.client_order_id,
            exchange_order_id: parsed.exchange_order_id,
            status: parsed.status,
            fill_quantity_base_delta: parsed.fill_quantity_base_delta,
            fill_price_quote: parsed.fill_price_quote,
            fee_quote_delta: parsed.fee_quote_delta,
            locally_simulated: true,
            external_submission_performed: false,
            live_execution_performed: false,
            occurred_at_unix_ms: self.received_at_unix_ms,
        };
        response.validate(validation)?;
        Ok(response)
    }
}

impl CexBalanceSnapshotTranscript {
    /// Construct a validated local exchange-shaped balance snapshot transcript.
    pub fn new(
        transcript_id: impl Into<String>,
        format: CexBalanceSnapshotTranscriptFormat,
        venue: VenueRef,
        payload_json: impl Into<String>,
        captured_at_unix_ms: u64,
        received_at_unix_ms: u64,
    ) -> Result<Self, CexConnectorError> {
        let transcript = Self {
            transcript_id: transcript_id.into(),
            format,
            venue,
            payload_json: payload_json.into(),
            captured_at_unix_ms,
            received_at_unix_ms,
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            account_state_queried: false,
            live_execution_performed: false,
            production_ready: false,
        };
        transcript.validate()?;
        Ok(transcript)
    }

    /// Validate local transcript invariants and side-effect denial flags.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "CEX balance transcript",
            &self.transcript_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if self.payload_json.trim().is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_BALANCE_TRANSCRIPT_PAYLOAD_REQUIRED",
                "CEX balance transcript payload must be non-empty",
            ));
        }
        if self.captured_at_unix_ms == 0 || self.received_at_unix_ms == 0 {
            violations.push(CexConnectorViolation::new(
                "CEX_BALANCE_TRANSCRIPT_TIMESTAMP_REQUIRED",
                "CEX balance transcript timestamps are required",
            ));
        }
        if self.captured_at_unix_ms > self.received_at_unix_ms {
            violations.push(CexConnectorViolation::new(
                "CEX_BALANCE_TRANSCRIPT_TIME_ORDER_INVALID",
                "CEX balance transcript capture time must not be after receive time",
            ));
        }
        if self.rest_call_performed
            || self.websocket_connection_opened
            || self.credentials_loaded
            || self.account_state_queried
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_BALANCE_TRANSCRIPT_EXTERNAL_SIDE_EFFECT",
                "CEX balance transcript parsing must not perform REST, WebSocket, credential, account-query, or live-execution side effects",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Parse this local transcript into a normalized balance snapshot record.
    pub fn parse_snapshot(&self) -> Result<CexBalanceSnapshotRecord, CexConnectorError> {
        self.validate()?;
        let payload: Value = serde_json::from_str(&self.payload_json).map_err(|error| {
            CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new_owned(
                    "CEX_BALANCE_TRANSCRIPT_JSON_INVALID",
                    format!("CEX balance transcript JSON is invalid: {error}"),
                )],
            }
        })?;
        let balances = match self.format {
            CexBalanceSnapshotTranscriptFormat::BinanceAccountBalances => {
                parse_binance_balance_snapshot(&payload)?
            }
            CexBalanceSnapshotTranscriptFormat::CoinbaseAccounts => {
                parse_coinbase_balance_snapshot(&payload)?
            }
            CexBalanceSnapshotTranscriptFormat::KrakenBalance => {
                parse_kraken_balance_snapshot(&payload)?
            }
        };
        let record = CexBalanceSnapshotRecord {
            framework_version: CEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            transcript_id: self.transcript_id.clone(),
            venue: self.venue.clone(),
            balances,
            rest_call_performed: false,
            websocket_connection_opened: false,
            credentials_loaded: false,
            account_state_queried: false,
            live_execution_performed: false,
            production_ready: false,
        };
        record.validate()?;
        Ok(record)
    }
}

impl CexBalanceSnapshotRecord {
    /// Validate normalized balance snapshot invariants.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != CEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(CexConnectorViolation::new(
                "CEX_FRAMEWORK_VERSION_MISMATCH",
                "CEX balance snapshot record has an unexpected framework version",
            ));
        }
        validate_id(
            "CEX balance transcript",
            &self.transcript_id,
            &mut violations,
        );
        validate_venue_ref(&self.venue, &mut violations);
        if self.balances.is_empty() {
            violations.push(CexConnectorViolation::new(
                "CEX_BALANCE_SNAPSHOT_EMPTY",
                "CEX balance snapshot requires at least one asset balance",
            ));
        }
        let mut assets = HashSet::new();
        for balance in &self.balances {
            validate_asset_balance(balance, &mut violations);
            if !assets.insert(balance.asset.to_ascii_uppercase()) {
                violations.push(CexConnectorViolation::new(
                    "CEX_BALANCE_ASSET_DUPLICATE",
                    "CEX balance snapshot assets must be unique",
                ));
            }
        }
        if self.rest_call_performed
            || self.websocket_connection_opened
            || self.credentials_loaded
            || self.account_state_queried
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_BALANCE_SNAPSHOT_EXTERNAL_SIDE_EFFECT",
                "CEX balance snapshot record must not include REST, WebSocket, credential, account-query, live-execution, or production-readiness side effects",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
}

impl CexOrderLifecycleRecord {
    /// Reconcile local/mock CEX responses into a lifecycle summary.
    pub fn from_local_responses(
        validation: &CexOrderValidationRecord,
        responses: &[CexOrderLifecycleResponse],
        duplicate_client_order_id_rejected: bool,
    ) -> Result<Self, CexConnectorError> {
        validation.validate()?;
        if responses.is_empty() {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new(
                    "CEX_LIFECYCLE_RESPONSES_REQUIRED",
                    "CEX lifecycle reconciliation requires at least one local/mock response",
                )],
            });
        }

        let mut current_status = CexOrderStatus::LocallyValidated;
        let mut filled_quantity_base = 0.0;
        let mut fill_notional_quote = 0.0;
        let mut total_fee_quote = 0.0;
        let mut fill_count = 0;
        let mut response_ids = HashSet::new();

        for response in responses {
            response.validate(validation)?;
            if !response_ids.insert(response.id.to_ascii_lowercase()) {
                return Err(CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new(
                        "CEX_RESPONSE_ID_DUPLICATE",
                        "CEX lifecycle response ids must be unique",
                    )],
                });
            }
            validate_cex_lifecycle_transition(current_status, response.status)?;
            current_status = response.status;

            if response.fill_quantity_base_delta > 0.0 {
                let Some(fill_price_quote) = response.fill_price_quote else {
                    return Err(CexConnectorError::ValidationFailed {
                        violations: vec![CexConnectorViolation::new(
                            "CEX_RESPONSE_FILL_PRICE_REQUIRED",
                            "fill-bearing CEX lifecycle responses require a positive finite fill price",
                        )],
                    });
                };
                fill_count += 1;
                filled_quantity_base += response.fill_quantity_base_delta;
                fill_notional_quote = response
                    .fill_quantity_base_delta
                    .mul_add(fill_price_quote, fill_notional_quote);
            }
            total_fee_quote += response.fee_quote_delta;
        }

        if filled_quantity_base - validation_quantity(validation) > f64::EPSILON {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new(
                    "CEX_FILL_QUANTITY_EXCEEDS_ORDER",
                    "CEX lifecycle fill reconciliation exceeds requested order quantity",
                )],
            });
        }

        let average_fill_price_quote = if filled_quantity_base > 0.0 {
            Some(fill_notional_quote / filled_quantity_base)
        } else {
            None
        };
        let remaining_quantity_base =
            (validation_quantity(validation) - filled_quantity_base).max(0.0);

        let record = Self {
            framework_version: CEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            request_id: validation.request_id.clone(),
            client_order_id: validation.client_order_id.clone(),
            strategy_id: validation.strategy_id.clone(),
            scope: validation.scope,
            venue: validation.venue.clone(),
            pair: validation.pair.clone(),
            final_status: current_status,
            transition_count: responses.len(),
            fill_count,
            filled_quantity_base,
            remaining_quantity_base,
            average_fill_price_quote,
            total_fee_quote,
            duplicate_client_order_id_rejected,
            responses_reconciled: true,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready: false,
            unresolved_blockers: vec![
                "exchange-specific live adapter responses missing".to_owned(),
                "sandbox/live fill reconciliation evidence missing".to_owned(),
                "deployment restart idempotency validation missing".to_owned(),
            ],
        };
        record.validate()?;
        Ok(record)
    }

    /// Validate lifecycle reconciliation invariants before persistence.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != CEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(CexConnectorViolation::new(
                "CEX_FRAMEWORK_VERSION_MISMATCH",
                "CEX lifecycle record has an unexpected framework version",
            ));
        }
        validate_id("CEX lifecycle request", &self.request_id, &mut violations);
        validate_id(
            "CEX lifecycle client order",
            &self.client_order_id,
            &mut violations,
        );
        validate_id("CEX lifecycle strategy", &self.strategy_id, &mut violations);
        validate_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "CEX_LIFECYCLE_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if self.transition_count == 0 || !self.responses_reconciled {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_RESPONSES_REQUIRED",
                "CEX lifecycle record requires reconciled responses",
            ));
        }
        if !self.filled_quantity_base.is_finite() || self.filled_quantity_base < 0.0 {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_FILLED_QUANTITY_INVALID",
                "CEX lifecycle filled quantity must be finite and non-negative",
            ));
        }
        if !self.remaining_quantity_base.is_finite() || self.remaining_quantity_base < 0.0 {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_REMAINING_QUANTITY_INVALID",
                "CEX lifecycle remaining quantity must be finite and non-negative",
            ));
        }
        if self
            .average_fill_price_quote
            .is_some_and(|price| !is_positive_finite(price))
        {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_AVERAGE_PRICE_INVALID",
                "CEX lifecycle average fill price must be positive and finite when present",
            ));
        }
        if !self.total_fee_quote.is_finite() || self.total_fee_quote < 0.0 {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_FEE_INVALID",
                "CEX lifecycle total fee must be finite and non-negative",
            ));
        }
        if self.external_submission_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(CexConnectorViolation::new(
                "CEX_LIFECYCLE_EXTERNAL_SIDE_EFFECT",
                "CEX lifecycle record must not include external submission, live execution, or production readiness",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }
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

/// Deterministic local CEX adapter for framework, policy, and replay tests.
///
/// This adapter is exchange-shaped but not exchange-connected. It serves
/// caller-supplied local quotes and fee schedules, validates paper order
/// requests through policy, and never opens sockets, loads credentials, reads
/// balances, submits live orders, cancels orders, or mutates exchange state.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalDeterministicCexAdapter {
    connector_name: String,
    profile: CexVenueProfile,
    quote: NormalizedQuote,
    fee_schedule: FeeSchedule,
    policy_gate: CexPolicyGate,
}

impl LocalDeterministicCexAdapter {
    /// Create a local deterministic CEX adapter from non-secret fixtures.
    pub fn new(
        connector_name: impl Into<String>,
        profile: CexVenueProfile,
        quote: NormalizedQuote,
        fee_schedule: FeeSchedule,
        policy: PolicyEngine,
    ) -> Result<Self, CexConnectorError> {
        let adapter = Self {
            connector_name: connector_name.into(),
            profile,
            quote,
            fee_schedule,
            policy_gate: CexPolicyGate::new(policy),
        };
        adapter.validate()?;
        Ok(adapter)
    }

    /// Validate local fixture invariants.
    pub fn validate(&self) -> Result<(), CexConnectorError> {
        let mut violations = Vec::new();
        validate_id("connector", &self.connector_name, &mut violations);
        if let Err(CexConnectorError::ValidationFailed {
            violations: profile_violations,
        }) = self.profile.validate()
        {
            violations.extend(profile_violations);
        }
        if let Err(error) = self.quote.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "LOCAL_CEX_QUOTE_INVALID",
                error.to_string(),
            ));
        }
        if let Err(error) = self.fee_schedule.validate() {
            violations.push(CexConnectorViolation::new_owned(
                "LOCAL_CEX_FEE_SCHEDULE_INVALID",
                error.to_string(),
            ));
        }
        if !self
            .quote
            .venue
            .name
            .eq_ignore_ascii_case(&self.profile.venue.name)
        {
            violations.push(CexConnectorViolation::new(
                "LOCAL_CEX_QUOTE_VENUE_MISMATCH",
                "local CEX quote venue must match adapter profile venue",
            ));
        }
        if self.quote.pair
            != self
                .fee_schedule
                .pair
                .clone()
                .unwrap_or_else(|| self.quote.pair.clone())
        {
            violations.push(CexConnectorViolation::new(
                "LOCAL_CEX_FEE_PAIR_MISMATCH",
                "local CEX fee schedule pair must match quote pair when scoped",
            ));
        }
        if !self
            .fee_schedule
            .venue
            .name
            .eq_ignore_ascii_case(&self.profile.venue.name)
        {
            violations.push(CexConnectorViolation::new(
                "LOCAL_CEX_FEE_VENUE_MISMATCH",
                "local CEX fee schedule venue must match adapter profile venue",
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(CexConnectorError::ValidationFailed { violations })
        }
    }

    /// Whether this adapter performed external network I/O. Always false.
    #[must_use]
    pub const fn external_network_used(&self) -> bool {
        false
    }

    /// Whether this adapter loaded credentials. Always false.
    #[must_use]
    pub const fn credentials_loaded(&self) -> bool {
        false
    }

    /// Whether this adapter submitted a live order. Always false.
    #[must_use]
    pub const fn live_order_submitted(&self) -> bool {
        false
    }

    /// Validate a request against local exchange-specific matching fixtures.
    ///
    /// This does not call REST/WebSocket APIs, load credentials, read balances,
    /// submit orders, or claim sandbox/live exchange behavior.
    pub fn validate_exchange_fixture_order(
        &self,
        rules: &CexExchangeMatchingRules,
        request: &CexOrderRequest,
    ) -> Result<CexExchangeFixtureValidation, CexConnectorError> {
        self.validate_order_request(request)?;
        rules.validate_order(request)?;
        if !same_venue(&self.profile.venue, &rules.venue) {
            return Err(CexConnectorError::VenueMismatch {
                profile: self.profile.venue.name.clone(),
                request: rules.venue.name.clone(),
            });
        }
        CexExchangeFixtureValidation::from_validated_request(&self.connector_name, request)
    }
}

impl CexConnectorIdentity for LocalDeterministicCexAdapter {
    fn connector_name(&self) -> &str {
        &self.connector_name
    }

    fn venue_profile(&self) -> &CexVenueProfile {
        &self.profile
    }
}

impl MarketDataProvider for LocalDeterministicCexAdapter {
    fn provider_name(&self) -> &str {
        &self.connector_name
    }

    fn capabilities(&self) -> MarketDataCapabilities {
        MarketDataCapabilities {
            order_book: true,
            top_of_book: true,
            fees: true,
            websocket: false,
            rest: false,
        }
    }

    fn order_book(
        &self,
        request: &MarketDataRequest,
    ) -> Result<OrderBookSnapshot, MarketDataError> {
        request.validate()?;
        self.validate_market_data_request(request)?;
        Ok(OrderBookSnapshot {
            id: format!("{}-local-book", self.quote.id),
            venue: self.quote.venue.clone(),
            pair: self.quote.pair.clone(),
            captured_at_unix_ms: self.quote.captured_at_unix_ms,
            received_at_unix_ms: self.quote.received_at_unix_ms,
            bids: vec![PriceLevel {
                price_quote: self.quote.bid.price_quote,
                quantity_base: self.quote.bid.quantity_base,
            }],
            asks: vec![PriceLevel {
                price_quote: self.quote.ask.price_quote,
                quantity_base: self.quote.ask.quantity_base,
            }],
            source_sequence: Some("local-deterministic-cex-fixture".to_owned()),
        })
    }

    fn top_of_book(&self, request: &MarketDataRequest) -> Result<NormalizedQuote, MarketDataError> {
        request.validate()?;
        self.validate_market_data_request(request)?;
        Ok(self.quote.clone())
    }
}

impl LocalDeterministicCexAdapter {
    fn validate_market_data_request(
        &self,
        request: &MarketDataRequest,
    ) -> Result<(), MarketDataError> {
        if !request
            .venue
            .name
            .eq_ignore_ascii_case(&self.profile.venue.name)
            || request.pair != self.quote.pair
        {
            return Err(MarketDataError::NoData {
                provider: self.connector_name.clone(),
                reason: "local CEX fixture does not contain the requested venue/pair".to_owned(),
            });
        }
        Ok(())
    }
}

impl FeeProvider for LocalDeterministicCexAdapter {
    fn provider_name(&self) -> &str {
        &self.connector_name
    }

    fn fee_schedule(
        &self,
        venue: &VenueRef,
        pair: Option<&MarketPair>,
    ) -> Result<FeeSchedule, FeeModelError> {
        if !venue.name.eq_ignore_ascii_case(&self.profile.venue.name) {
            return Err(FeeModelError::ScheduleUnavailable {
                provider: self.connector_name.clone(),
                reason: "local CEX fixture does not contain the requested venue".to_owned(),
            });
        }
        if let Some(pair) = pair {
            if self.fee_schedule.pair.as_ref() != Some(pair) {
                return Err(FeeModelError::ScheduleUnavailable {
                    provider: self.connector_name.clone(),
                    reason: "local CEX fixture does not contain the requested pair".to_owned(),
                });
            }
        }
        Ok(self.fee_schedule.clone())
    }
}

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

impl CexTradingConnector for LocalDeterministicCexAdapter {
    fn validate_order_request(
        &self,
        request: &CexOrderRequest,
    ) -> Result<PolicyApproval, CexConnectorError> {
        self.policy_gate.validate_order(&self.profile, request)
    }

    fn submit_order(&self, request: &CexOrderRequest) -> Result<CexOrderStatus, CexConnectorError> {
        self.validate_order_request(request)?;
        Ok(CexOrderStatus::LocallyValidated)
    }
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
    /// Append-only audit journal persistence failed.
    AuditJournalFailed { reason: String },
    /// State-store checkpoint persistence failed.
    StateStoreFailed { reason: String },
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
            | Self::PolicyDenied { .. }
            | Self::AuditJournalFailed { .. }
            | Self::StateStoreFailed { .. } => &[],
        }
    }

    fn audit_failed(reason: impl Into<String>) -> Self {
        Self::AuditJournalFailed {
            reason: reason.into(),
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
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "CEX audit journal persistence failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(formatter, "CEX state-store persistence failed: {reason}")
            }
        }
    }
}

impl Error for CexConnectorError {}

impl From<StateStoreError> for CexConnectorError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
}

/// Persist the latest local CEX framework validation through the state boundary.
pub fn persist_cex_order_validation_checkpoint(
    store: &mut impl StateStore,
    record: &CexOrderValidationRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CexConnectorError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: CEX_LAST_ORDER_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: CEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            CexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize CEX validation checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local CEX framework validation record to the audit journal.
pub fn append_cex_order_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &CexOrderValidationRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CexConnectorError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("cex-order-validation-{}", record.request_id),
        AuditEventKind::ExecutionSubmission,
        CEX_STATE_SUBSYSTEM,
        "cex-framework",
        "CEX order framework validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "cex_framework_version",
            AuditValue::Text(record.framework_version.clone()),
        )
        .with_metadata("request_id", AuditValue::Text(record.request_id.clone()))
        .with_metadata(
            "client_order_id",
            AuditValue::Text(record.client_order_id.clone()),
        )
        .with_metadata("venue", AuditValue::Text(record.venue.name.clone()))
        .with_metadata("scope", AuditValue::Text(format!("{:?}", record.scope)))
        .with_metadata("status", AuditValue::Text(format!("{:?}", record.status)))
        .with_metadata("policy_approved", AuditValue::Bool(record.policy_approved))
        .with_metadata(
            "external_submission_performed",
            AuditValue::Bool(record.external_submission_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(record.live_execution_performed),
        );
    journal
        .append_event(event)
        .map_err(|error| CexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local/mock CEX lifecycle reconciliation through state.
pub fn persist_cex_order_lifecycle_checkpoint(
    store: &mut impl StateStore,
    record: &CexOrderLifecycleRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, CexConnectorError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY.to_owned(),
        subsystem: CEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            CexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize CEX lifecycle checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(CexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local/mock CEX lifecycle reconciliation record to the audit journal.
pub fn append_cex_order_lifecycle_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &CexOrderLifecycleRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, CexConnectorError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("cex-order-lifecycle-{}", record.request_id),
        AuditEventKind::ExecutionResult,
        CEX_STATE_SUBSYSTEM,
        "cex-framework",
        "CEX order lifecycle reconciliation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "cex_framework_version",
            AuditValue::Text(record.framework_version.clone()),
        )
        .with_metadata("request_id", AuditValue::Text(record.request_id.clone()))
        .with_metadata(
            "client_order_id",
            AuditValue::Text(record.client_order_id.clone()),
        )
        .with_metadata("venue", AuditValue::Text(record.venue.name.clone()))
        .with_metadata(
            "final_status",
            AuditValue::Text(format!("{:?}", record.final_status)),
        )
        .with_metadata(
            "transition_count",
            AuditValue::Integer(i64::try_from(record.transition_count).unwrap_or(i64::MAX)),
        )
        .with_metadata(
            "fill_count",
            AuditValue::Integer(i64::try_from(record.fill_count).unwrap_or(i64::MAX)),
        )
        .with_metadata(
            "duplicate_client_order_id_rejected",
            AuditValue::Bool(record.duplicate_client_order_id_rejected),
        )
        .with_metadata(
            "external_submission_performed",
            AuditValue::Bool(record.external_submission_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(record.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(record.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| CexConnectorError::audit_failed(error.to_string()))
}

/// Validate local CEX client-order-id uniqueness for idempotency tests.
pub fn validate_cex_client_order_id_uniqueness(
    records: &[CexOrderValidationRecord],
) -> Result<(), CexConnectorError> {
    let mut seen = HashMap::new();
    for record in records {
        record.validate()?;
        let normalized = record.client_order_id.to_ascii_lowercase();
        if let Some(first_request_id) = seen.insert(normalized, record.request_id.clone()) {
            return Err(CexConnectorError::ValidationFailed {
                violations: vec![CexConnectorViolation::new_owned(
                    "CEX_CLIENT_ORDER_ID_DUPLICATE",
                    format!(
                        "duplicate CEX client order id across requests {first_request_id} and {}",
                        record.request_id
                    ),
                )],
            });
        }
    }
    Ok(())
}

fn cex_live_adapter_boundary_blockers(
    request: &CexLiveAdapterBoundaryReviewRequest,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !request.rest_request_plan_validated {
        blockers.push("local-rest-request-plan-validation-missing".to_owned());
    }
    if !request.websocket_request_plan_validated {
        blockers.push("local-websocket-request-plan-validation-missing".to_owned());
    }
    if !request.lifecycle_transcript_parsing_validated {
        blockers.push("local-lifecycle-transcript-parsing-missing".to_owned());
    }
    if !request.balance_snapshot_parsing_validated {
        blockers.push("local-balance-snapshot-parsing-missing".to_owned());
    }
    if !request.credential_scope_reviewed {
        blockers.push("local-credential-scope-review-missing".to_owned());
    }
    if !request.rate_limit_reviewed {
        blockers.push("local-rate-limit-review-missing".to_owned());
    }
    if !request.exchange_matching_rules_validated {
        blockers.push("local-exchange-matching-rules-validation-missing".to_owned());
    }
    if !request.sandbox_order_lifecycle_evidence_available {
        blockers.push("sandbox-order-lifecycle-evidence-missing".to_owned());
    }
    if !request.sandbox_balance_evidence_available {
        blockers.push("sandbox-balance-evidence-missing".to_owned());
    }
    if !request.sandbox_cancel_evidence_available {
        blockers.push("sandbox-cancel-reconciliation-evidence-missing".to_owned());
    }
    if !request.production_idempotency_evidence_available {
        blockers.push("production-idempotency-replay-evidence-missing".to_owned());
    }
    if blockers.is_empty() {
        blockers.push("live-exchange-adapter-implementation-review-required".to_owned());
    }
    blockers
}

fn parse_binance_depth_payload(
    value: &serde_json::Value,
) -> Result<ParsedCexOrderBook, CexConnectorError> {
    let bids = parse_price_levels(
        value.get("bids").ok_or_else(|| {
            transcript_violation(
                "CEX_BINANCE_BIDS_REQUIRED",
                "Binance depth payload requires bids",
            )
        })?,
        "Binance bid",
    )?;
    let asks = parse_price_levels(
        value.get("asks").ok_or_else(|| {
            transcript_violation(
                "CEX_BINANCE_ASKS_REQUIRED",
                "Binance depth payload requires asks",
            )
        })?,
        "Binance ask",
    )?;
    let source_sequence = value
        .get("lastUpdateId")
        .and_then(serde_json::Value::as_i64)
        .map(|sequence| sequence.to_string());
    Ok(ParsedCexOrderBook {
        bids,
        asks,
        source_sequence,
    })
}

fn parse_coinbase_product_book_payload(
    value: &serde_json::Value,
) -> Result<ParsedCexOrderBook, CexConnectorError> {
    let bids = parse_price_levels(
        value.get("bids").ok_or_else(|| {
            transcript_violation(
                "CEX_COINBASE_BIDS_REQUIRED",
                "Coinbase product book payload requires bids",
            )
        })?,
        "Coinbase bid",
    )?;
    let asks = parse_price_levels(
        value.get("asks").ok_or_else(|| {
            transcript_violation(
                "CEX_COINBASE_ASKS_REQUIRED",
                "Coinbase product book payload requires asks",
            )
        })?,
        "Coinbase ask",
    )?;
    let source_sequence = value
        .get("sequence")
        .and_then(serde_json::Value::as_i64)
        .map(|sequence| sequence.to_string());
    Ok(ParsedCexOrderBook {
        bids,
        asks,
        source_sequence,
    })
}

fn parse_kraken_depth_payload(
    value: &serde_json::Value,
) -> Result<ParsedCexOrderBook, CexConnectorError> {
    let errors = value
        .get("error")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            transcript_violation(
                "CEX_KRAKEN_ERROR_ARRAY_REQUIRED",
                "Kraken depth payload requires an error array",
            )
        })?;
    if !errors.is_empty() {
        return Err(transcript_violation(
            "CEX_KRAKEN_ERROR_RESPONSE",
            "Kraken depth payload contains exchange error entries",
        ));
    }
    let result = value
        .get("result")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            transcript_violation(
                "CEX_KRAKEN_RESULT_REQUIRED",
                "Kraken depth payload requires a result object",
            )
        })?;
    let (symbol, book) = result.iter().next().ok_or_else(|| {
        transcript_violation(
            "CEX_KRAKEN_RESULT_EMPTY",
            "Kraken depth payload result object must contain one book",
        )
    })?;
    let bids = parse_price_levels(
        book.get("b").ok_or_else(|| {
            transcript_violation(
                "CEX_KRAKEN_BIDS_REQUIRED",
                "Kraken depth payload requires b",
            )
        })?,
        "Kraken bid",
    )?;
    let asks = parse_price_levels(
        book.get("a").ok_or_else(|| {
            transcript_violation(
                "CEX_KRAKEN_ASKS_REQUIRED",
                "Kraken depth payload requires a",
            )
        })?,
        "Kraken ask",
    )?;
    Ok(ParsedCexOrderBook {
        bids,
        asks,
        source_sequence: Some(symbol.clone()),
    })
}

fn parse_price_levels(
    value: &serde_json::Value,
    label: &'static str,
) -> Result<Vec<PriceLevel>, CexConnectorError> {
    let levels = value.as_array().ok_or_else(|| {
        transcript_violation(
            "CEX_TRANSCRIPT_LEVELS_ARRAY_REQUIRED",
            "CEX market-data transcript levels must be an array",
        )
    })?;
    if levels.is_empty() {
        return Err(transcript_violation(
            "CEX_TRANSCRIPT_LEVELS_EMPTY",
            "CEX market-data transcript levels must be non-empty",
        ));
    }

    levels
        .iter()
        .map(|level| {
            let tuple = level.as_array().ok_or_else(|| {
                transcript_violation(
                    "CEX_TRANSCRIPT_LEVEL_ARRAY_REQUIRED",
                    "CEX market-data transcript level must be an array",
                )
            })?;
            if tuple.len() < 2 {
                return Err(transcript_violation(
                    "CEX_TRANSCRIPT_LEVEL_FIELDS_REQUIRED",
                    "CEX market-data transcript level requires price and quantity",
                ));
            }
            let price_quote = parse_json_number(&tuple[0], "price")?;
            let quantity_base = parse_json_number(&tuple[1], "quantity")?;
            PriceLevel::new(price_quote, quantity_base).map_err(|error| {
                CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new_owned(
                        "CEX_TRANSCRIPT_LEVEL_INVALID",
                        format!("{label} level is invalid: {error}"),
                    )],
                }
            })
        })
        .collect()
}

fn parse_json_number(
    value: &serde_json::Value,
    label: &'static str,
) -> Result<f64, CexConnectorError> {
    let parsed = match value {
        serde_json::Value::String(raw) => raw.parse::<f64>().ok(),
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Array(_)
        | serde_json::Value::Object(_) => None,
    };
    parsed.filter(|number| number.is_finite()).ok_or_else(|| {
        transcript_violation_owned(
            "CEX_TRANSCRIPT_NUMBER_INVALID",
            format!("CEX market-data transcript {label} must be a finite number"),
        )
    })
}

fn transcript_violation(code: &'static str, message: &'static str) -> CexConnectorError {
    CexConnectorError::ValidationFailed {
        violations: vec![CexConnectorViolation::new(code, message)],
    }
}

fn transcript_violation_owned(code: &'static str, message: String) -> CexConnectorError {
    CexConnectorError::ValidationFailed {
        violations: vec![CexConnectorViolation::new_owned(code, message)],
    }
}

fn push_code_if(codes: &mut Vec<String>, condition: bool, code: &'static str) {
    if condition {
        codes.push(code.to_owned());
    }
}

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

struct ParsedCexLifecycleResponse {
    client_order_id: String,
    exchange_order_id: Option<String>,
    status: CexOrderStatus,
    fill_quantity_base_delta: f64,
    fill_price_quote: Option<f64>,
    fee_quote_delta: f64,
}

fn parse_binance_lifecycle_payload(
    payload: &Value,
) -> Result<ParsedCexLifecycleResponse, CexConnectorError> {
    let status = parse_cex_order_status(&json_string(payload, &["X", "orderStatus", "status"])?)?;
    let fill_quantity_base_delta = json_number_or_default(payload, &["l", "lastExecutedQty"], 0.0)?;
    Ok(ParsedCexLifecycleResponse {
        client_order_id: json_string(payload, &["c", "clientOrderId", "client_order_id"])?,
        exchange_order_id: optional_json_string(payload, &["i", "orderId", "order_id"]),
        status,
        fill_quantity_base_delta,
        fill_price_quote: optional_json_number(payload, &["L", "lastExecutedPrice", "price"])?,
        fee_quote_delta: json_number_or_default(payload, &["n", "commission", "fee"], 0.0)?,
    })
}

fn parse_coinbase_lifecycle_payload(
    payload: &Value,
) -> Result<ParsedCexLifecycleResponse, CexConnectorError> {
    let status = parse_cex_order_status(&json_string(payload, &["status", "type"])?)?;
    let fill_quantity_base_delta = json_number_or_default(
        payload,
        &["last_fill_size", "size", "filled_size_delta"],
        0.0,
    )?;
    Ok(ParsedCexLifecycleResponse {
        client_order_id: json_string(payload, &["client_order_id", "clientOrderId"])?,
        exchange_order_id: optional_json_string(payload, &["order_id", "orderId"]),
        status,
        fill_quantity_base_delta,
        fill_price_quote: optional_json_number(payload, &["price", "last_fill_price"])?,
        fee_quote_delta: json_number_or_default(payload, &["fee", "commission"], 0.0)?,
    })
}

fn parse_kraken_lifecycle_payload(
    payload: &Value,
) -> Result<ParsedCexLifecycleResponse, CexConnectorError> {
    let status = parse_cex_order_status(&json_string(payload, &["status"])?)?;
    let fill_quantity_base_delta = json_number_or_default(
        payload,
        &["vol_exec_delta", "last_fill_size", "fill_qty"],
        0.0,
    )?;
    Ok(ParsedCexLifecycleResponse {
        client_order_id: json_string(payload, &["userref", "client_order_id", "clientOrderId"])?,
        exchange_order_id: optional_json_string(payload, &["txid", "order_id", "orderId"]),
        status,
        fill_quantity_base_delta,
        fill_price_quote: optional_json_number(payload, &["price", "avg_price"])?,
        fee_quote_delta: json_number_or_default(payload, &["fee", "fee_quote"], 0.0)?,
    })
}

fn parse_binance_balance_snapshot(
    payload: &Value,
) -> Result<Vec<CexAssetBalanceSnapshot>, CexConnectorError> {
    let balances = payload
        .get("balances")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            balance_violation(
                "CEX_BALANCE_TRANSCRIPT_FIELD_REQUIRED",
                "Binance balance payload requires balances array",
            )
        })?;
    balances
        .iter()
        .map(|entry| {
            let asset = json_string(entry, &["asset"])?;
            let available = json_number_or_default(entry, &["free"], 0.0)?;
            let locked = json_number_or_default(entry, &["locked"], 0.0)?;
            Ok(CexAssetBalanceSnapshot {
                asset: normalize_balance_asset(&asset),
                available,
                total: available + locked,
            })
        })
        .collect()
}

fn parse_coinbase_balance_snapshot(
    payload: &Value,
) -> Result<Vec<CexAssetBalanceSnapshot>, CexConnectorError> {
    let accounts = payload
        .get("accounts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            balance_violation(
                "CEX_BALANCE_TRANSCRIPT_FIELD_REQUIRED",
                "Coinbase balance payload requires accounts array",
            )
        })?;
    accounts
        .iter()
        .map(|entry| {
            let asset = json_string(entry, &["currency", "asset"])?;
            let available =
                nested_json_number(entry, &["available_balance", "available"], "value")?
                    .unwrap_or(0.0);
            let hold = nested_json_number(entry, &["hold", "held"], "value")?.unwrap_or(0.0);
            let total = nested_json_number(entry, &["balance", "total"], "value")?
                .unwrap_or(available + hold);
            Ok(CexAssetBalanceSnapshot {
                asset: normalize_balance_asset(&asset),
                available,
                total,
            })
        })
        .collect()
}

fn parse_kraken_balance_snapshot(
    payload: &Value,
) -> Result<Vec<CexAssetBalanceSnapshot>, CexConnectorError> {
    let result = payload
        .get("result")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            balance_violation(
                "CEX_BALANCE_TRANSCRIPT_FIELD_REQUIRED",
                "Kraken balance payload requires result object",
            )
        })?;
    result
        .iter()
        .map(|(asset, value)| {
            let total = json_value_number(value, asset)?;
            Ok(CexAssetBalanceSnapshot {
                asset: normalize_kraken_balance_asset(asset),
                available: total,
                total,
            })
        })
        .collect()
}

fn validate_asset_balance(
    balance: &CexAssetBalanceSnapshot,
    violations: &mut Vec<CexConnectorViolation>,
) {
    if balance.asset.trim().is_empty() {
        violations.push(CexConnectorViolation::new(
            "CEX_BALANCE_ASSET_REQUIRED",
            "CEX balance asset must be non-empty",
        ));
    }
    if !balance.available.is_finite() || balance.available < 0.0 {
        violations.push(CexConnectorViolation::new(
            "CEX_BALANCE_AVAILABLE_INVALID",
            "CEX balance available amount must be finite and non-negative",
        ));
    }
    if !balance.total.is_finite() || balance.total < 0.0 {
        violations.push(CexConnectorViolation::new(
            "CEX_BALANCE_TOTAL_INVALID",
            "CEX balance total amount must be finite and non-negative",
        ));
    }
    if balance.available - balance.total > f64::EPSILON {
        violations.push(CexConnectorViolation::new(
            "CEX_BALANCE_AVAILABLE_EXCEEDS_TOTAL",
            "CEX balance available amount must not exceed total amount",
        ));
    }
}

fn nested_json_number(
    payload: &Value,
    object_keys: &[&str],
    nested_key: &str,
) -> Result<Option<f64>, CexConnectorError> {
    for object_key in object_keys {
        let Some(value) = payload.get(*object_key) else {
            continue;
        };
        if value.is_object() {
            return optional_json_number(value, &[nested_key]);
        }
        return optional_json_number(payload, &[*object_key]);
    }
    Ok(None)
}

fn json_value_number(value: &Value, key: &str) -> Result<f64, CexConnectorError> {
    let parsed = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    };
    parsed.ok_or_else(|| {
        balance_violation_owned(
            "CEX_BALANCE_TRANSCRIPT_NUMBER_INVALID",
            format!("CEX balance transcript number field is invalid: {key}"),
        )
    })
}

fn normalize_balance_asset(asset: &str) -> String {
    asset.trim().to_ascii_uppercase()
}

fn normalize_kraken_balance_asset(asset: &str) -> String {
    match asset {
        "XXBT" | "XBT" => "BTC".to_owned(),
        "ZUSD" => "USD".to_owned(),
        "ZUSDC" => "USDC".to_owned(),
        other => normalize_balance_asset(other),
    }
}

fn parse_cex_order_status(raw: &str) -> Result<CexOrderStatus, CexConnectorError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "new" | "open" | "accepted" | "pending" => Ok(CexOrderStatus::Accepted),
        "rejected" | "reject" | "expired" => Ok(CexOrderStatus::Rejected),
        "filled" | "done" | "closed" => Ok(CexOrderStatus::Filled),
        "partially_filled" | "partial_fill" | "partial" | "match" => {
            Ok(CexOrderStatus::PartiallyFilled)
        }
        "canceled" | "cancelled" => Ok(CexOrderStatus::Cancelled),
        _ => Err(CexConnectorError::ValidationFailed {
            violations: vec![CexConnectorViolation::new_owned(
                "CEX_LIFECYCLE_TRANSCRIPT_STATUS_UNKNOWN",
                format!("unsupported CEX lifecycle transcript status: {raw}"),
            )],
        }),
    }
}

fn balance_violation(code: &'static str, message: &'static str) -> CexConnectorError {
    CexConnectorError::ValidationFailed {
        violations: vec![CexConnectorViolation::new(code, message)],
    }
}

fn balance_violation_owned(code: &'static str, message: String) -> CexConnectorError {
    CexConnectorError::ValidationFailed {
        violations: vec![CexConnectorViolation::new_owned(code, message)],
    }
}

fn json_string(payload: &Value, keys: &[&str]) -> Result<String, CexConnectorError> {
    optional_json_string(payload, keys).ok_or_else(|| CexConnectorError::ValidationFailed {
        violations: vec![CexConnectorViolation::new_owned(
            "CEX_LIFECYCLE_TRANSCRIPT_FIELD_REQUIRED",
            format!("required CEX lifecycle transcript string field missing: {keys:?}"),
        )],
    })
}

fn optional_json_string(payload: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| payload.get(*key))
        .find_map(|value| match value {
            Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        })
}

fn json_number_or_default(
    payload: &Value,
    keys: &[&str],
    default: f64,
) -> Result<f64, CexConnectorError> {
    match optional_json_number(payload, keys)? {
        Some(value) => {
            if value.is_finite() && value >= 0.0 {
                Ok(value)
            } else {
                Err(CexConnectorError::ValidationFailed {
                    violations: vec![CexConnectorViolation::new_owned(
                        "CEX_LIFECYCLE_TRANSCRIPT_NUMBER_INVALID",
                        format!(
                        "CEX lifecycle transcript number must be finite and non-negative: {keys:?}"
                    ),
                    )],
                })
            }
        }
        None => Ok(default),
    }
}

fn optional_json_number(payload: &Value, keys: &[&str]) -> Result<Option<f64>, CexConnectorError> {
    for key in keys {
        let Some(value) = payload.get(*key) else {
            continue;
        };
        let parsed = match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse::<f64>().ok(),
            _ => None,
        };
        if let Some(number) = parsed {
            return Ok(Some(number));
        }
        return Err(CexConnectorError::ValidationFailed {
            violations: vec![CexConnectorViolation::new_owned(
                "CEX_LIFECYCLE_TRANSCRIPT_NUMBER_INVALID",
                format!("CEX lifecycle transcript number field is invalid: {key}"),
            )],
        });
    }
    Ok(None)
}

fn validate_cex_lifecycle_transition(
    from: CexOrderStatus,
    to: CexOrderStatus,
) -> Result<(), CexConnectorError> {
    let allowed = matches!(
        (from, to),
        (
            CexOrderStatus::LocallyValidated,
            CexOrderStatus::Accepted | CexOrderStatus::Rejected
        ) | (
            CexOrderStatus::Accepted | CexOrderStatus::PartiallyFilled,
            CexOrderStatus::PartiallyFilled | CexOrderStatus::Filled | CexOrderStatus::Cancelled
        )
    );
    if allowed {
        Ok(())
    } else {
        Err(CexConnectorError::ValidationFailed {
            violations: vec![CexConnectorViolation::new_owned(
                "CEX_ORDER_LIFECYCLE_TRANSITION_INVALID",
                format!("invalid local CEX lifecycle transition from {from:?} to {to:?}"),
            )],
        })
    }
}

fn validation_quantity(validation: &CexOrderValidationRecord) -> f64 {
    validation.quantity_base
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn is_non_negative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn is_multiple_of_step(value: f64, step: f64) -> bool {
    if !is_positive_finite(value) || !is_positive_finite(step) {
        return false;
    }
    let ratio = value / step;
    (ratio - ratio.round()).abs() <= 1e-8
}

fn empty_optional(value: Option<&String>) -> bool {
    value.map_or(true, String::is_empty)
}

fn binance_symbol(pair: &MarketPair) -> String {
    format!("{}{}", pair.base, pair.quote)
}

fn dash_symbol(pair: &MarketPair) -> String {
    format!("{}-{}", pair.base, pair.quote)
}

fn kraken_symbol(pair: &MarketPair) -> String {
    let base = if pair.base == "BTC" {
        "XBT"
    } else {
        pair.base.as_str()
    };
    format!("{base}{}", pair.quote)
}

#[cfg(test)]
mod tests {
    use super::{
        append_cex_order_lifecycle_audit, append_cex_order_validation_audit,
        persist_cex_order_lifecycle_checkpoint, persist_cex_order_validation_checkpoint,
        validate_cex_client_order_id_uniqueness, validate_cex_credential_scope_review,
        validate_cex_rate_limit, CexBalanceSnapshotTranscript, CexBalanceSnapshotTranscriptFormat,
        CexConnectorCapabilities, CexConnectorError, CexConnectorRegistry, CexCredentialPermission,
        CexCredentialScopeReviewInput, CexCredentialScopeReviewStatus, CexExchangeMarketDataFormat,
        CexExchangeMatchingRules, CexLiveAdapterBoundaryReviewRequest,
        CexLiveAdapterBoundaryReviewStatus, CexMarketDataRequestPlan, CexMockMarketDataTranscript,
        CexOrderLifecycleRecord, CexOrderLifecycleResponse, CexOrderLifecycleTranscript,
        CexOrderLifecycleTranscriptFormat, CexOrderRequest, CexOrderSide, CexOrderStatus,
        CexOrderType, CexOrderValidationRecord, CexPolicyGate, CexRateLimitObservation,
        CexRateLimitScope, CexRateLimitStatus, CexTimeInForce, CexTradingConnector,
        CexVenueProfile, LocalDeterministicCexAdapter, CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY,
        CEX_LAST_ORDER_VALIDATION_CHECKPOINT_KEY, CEX_STATE_SUBSYSTEM,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, ExecutionScope, FeeProvider, FeeSchedule,
        LiquidityRole, MarketDataProvider, MarketDataRequest, MarketPair, NormalizedQuote,
        PolicyEngine, PriceLevel, SecretRef, SqliteWalStateStore, StateStore, VenueKind, VenueRef,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
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

    fn local_adapter() -> LocalDeterministicCexAdapter {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        LocalDeterministicCexAdapter::new(
            "local-paper-coinbase",
            profile(),
            quote(),
            fee_schedule(),
            PolicyEngine::from_config(config),
        )
        .expect("local adapter should validate")
    }

    fn quote() -> NormalizedQuote {
        NormalizedQuote {
            id: "local-cex-quote-1".to_owned(),
            venue: venue(),
            pair: MarketPair::new("BTC", "USDC").expect("pair should validate"),
            bid: PriceLevel {
                price_quote: 99.5,
                quantity_base: 1.0,
            },
            ask: PriceLevel {
                price_quote: 100.5,
                quantity_base: 1.0,
            },
            captured_at_unix_ms: 1_700_000_000,
            received_at_unix_ms: 1_700_000_001,
        }
    }

    fn fee_schedule() -> FeeSchedule {
        FeeSchedule {
            venue: venue(),
            pair: Some(MarketPair::new("BTC", "USDC").expect("pair should validate")),
            maker_bps: 10.0,
            taker_bps: 20.0,
            network_fee_quote: 0.0,
            externally_verified: false,
        }
    }

    fn exchange_venue(name: &str) -> VenueRef {
        VenueRef {
            name: name.to_owned(),
            kind: VenueKind::Cex,
        }
    }

    fn exchange_config(venue_name: &str) -> AgentConfig {
        AgentConfig::from_toml_str(&format!(
            r#"
[runtime]
mode = "paper"
live_execution_enabled = false
allow_withdrawals = false
kill_switch_enabled = true

[risk]
max_single_trade_quote = 100000.0
max_daily_loss_quote = 1000.0
max_open_exposure_quote = 100000.0
slippage_bps = 100
gas_fee_cap_quote = 2.0

[venues]
cex_allowlist = ["{venue_name}"]
dex_allowlist = []
chain_allowlist = []
asset_allowlist = ["BTC", "USDC"]

[secrets]
backend = "disabled"
exchange_credentials = {{ source = "disabled" }}
wallet_signer = {{ source = "disabled" }}

[communication]
cli_enabled = true
notify_channels = []

[audit]
enabled = true
redact_secrets = true
"#
        ))
        .expect("exchange fixture config should validate")
    }

    fn exchange_order_request(venue_name: &str) -> CexOrderRequest {
        CexOrderRequest {
            venue: exchange_venue(venue_name),
            quantity_base: 0.001,
            limit_price_quote: Some(50_000.0),
            notional_quote: 50.0,
            ..order_request()
        }
    }

    fn exchange_adapter(venue_name: &str) -> LocalDeterministicCexAdapter {
        let venue = exchange_venue(venue_name);
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let profile = CexVenueProfile::new(
            venue.clone(),
            format!("{venue_name} local fixture"),
            CexConnectorCapabilities::paper_sandbox(),
        )
        .expect("exchange profile should validate");
        let quote = NormalizedQuote {
            id: format!("{venue_name}-local-cex-quote-1"),
            venue: venue.clone(),
            pair: pair.clone(),
            bid: PriceLevel {
                price_quote: 49_995.0,
                quantity_base: 2.0,
            },
            ask: PriceLevel {
                price_quote: 50_005.0,
                quantity_base: 2.0,
            },
            captured_at_unix_ms: 1_700_000_000,
            received_at_unix_ms: 1_700_000_001,
        };
        let fee_schedule = FeeSchedule {
            venue,
            pair: Some(pair),
            maker_bps: 8.0,
            taker_bps: 12.0,
            network_fee_quote: 0.0,
            externally_verified: false,
        };

        LocalDeterministicCexAdapter::new(
            format!("local-{venue_name}-fixture"),
            profile,
            quote,
            fee_schedule,
            PolicyEngine::from_config(exchange_config(venue_name)),
        )
        .expect("exchange adapter should validate")
    }

    fn temp_path(label: &str, extension: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-{label}-{}-{nonce}.{extension}",
            std::process::id(),
        ));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension(format!("{extension}-wal")));
        let _ = fs::remove_file(path.with_extension(format!("{extension}-shm")));
        path
    }

    fn cleanup_sqlite(path: &PathBuf) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(format!("{}-wal", path.display()));
        let _ = fs::remove_file(format!("{}-shm", path.display()));
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

    #[test]
    fn local_cex_adapter_serves_fixture_market_data_without_network() {
        let adapter = local_adapter();
        let request = MarketDataRequest {
            venue: venue(),
            pair: MarketPair::new("BTC", "USDC").expect("pair should validate"),
            max_age_ms: 5_000,
        };

        let top = adapter
            .top_of_book(&request)
            .expect("top of book should come from fixture");
        let book = adapter
            .order_book(&request)
            .expect("order book should be synthesized from fixture");
        let fees = adapter
            .fee_schedule(&venue(), Some(&request.pair))
            .expect("fee fixture should be returned");

        assert_eq!(top.id, "local-cex-quote-1");
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 1);
        assert!((fees.maker_bps - 10.0).abs() < f64::EPSILON);
        assert!(!adapter.external_network_used());
        assert!(!adapter.credentials_loaded());
        assert!(!adapter.live_order_submitted());
    }

    #[test]
    fn local_cex_adapter_submits_paper_order_as_local_validation_only() {
        let adapter = local_adapter();
        let approval = adapter
            .validate_order_request(&order_request())
            .expect("paper order validates through policy");
        let status = adapter
            .submit_order(&order_request())
            .expect("paper submit returns local status only");

        assert_eq!(approval.intent_id, "cex-order-1");
        assert_eq!(status, CexOrderStatus::LocallyValidated);
        assert!(!adapter.external_network_used());
        assert!(!adapter.credentials_loaded());
        assert!(!adapter.live_order_submitted());
    }

    #[test]
    fn local_cex_adapter_rejects_live_order_without_submission() {
        let adapter = local_adapter();
        let mut request = order_request();
        request.scope = ExecutionScope::Live;

        let error = adapter
            .submit_order(&request)
            .expect_err("live order should fail before local submission status");

        assert!(matches!(error, CexConnectorError::LiveOrdersUnavailable));
        assert!(!adapter.external_network_used());
        assert!(!adapter.credentials_loaded());
        assert!(!adapter.live_order_submitted());
    }

    #[test]
    fn local_exchange_fixture_profiles_validate_named_cex_matching_rules() {
        for (venue_name, rules) in [
            (
                "binance",
                CexExchangeMatchingRules::binance_btc_usdc_fixture()
                    .expect("binance rules should validate"),
            ),
            (
                "coinbase",
                CexExchangeMatchingRules::coinbase_btc_usdc_fixture()
                    .expect("coinbase rules should validate"),
            ),
            (
                "kraken",
                CexExchangeMatchingRules::kraken_btc_usdc_fixture()
                    .expect("kraken rules should validate"),
            ),
        ] {
            let adapter = exchange_adapter(venue_name);
            let request = exchange_order_request(venue_name);
            let report = adapter
                .validate_exchange_fixture_order(&rules, &request)
                .expect("exchange-specific local fixture order should validate");

            assert!(report.profile_policy_validated);
            assert!(report.matching_rules_validated);
            assert!(!report.rest_call_performed);
            assert!(!report.websocket_connection_opened);
            assert!(!report.credentials_loaded);
            assert!(!report.external_submission_performed);
            assert!(!report.live_execution_performed);
            assert!(!report.production_ready);
        }
    }

    #[test]
    fn cex_live_adapter_boundary_blocks_until_sandbox_and_live_evidence_exists() {
        let report = super::review_cex_live_adapter_boundary(CexLiveAdapterBoundaryReviewRequest {
            review_id: "local-cex-live-adapter-boundary".to_owned(),
            connector_name: "binance-local-boundary".to_owned(),
            venue: exchange_venue("binance"),
            rest_request_plan_validated: true,
            websocket_request_plan_validated: true,
            lifecycle_transcript_parsing_validated: true,
            balance_snapshot_parsing_validated: true,
            credential_scope_reviewed: true,
            rate_limit_reviewed: true,
            exchange_matching_rules_validated: true,
            sandbox_order_lifecycle_evidence_available: false,
            sandbox_balance_evidence_available: false,
            sandbox_cancel_evidence_available: false,
            production_idempotency_evidence_available: false,
            credential_material_loaded: false,
            rest_call_performed: false,
            websocket_connection_opened: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: false,
            validated_at_unix_ms: 100_000,
        })
        .expect("local CEX live adapter boundary should produce a blocked report");

        assert_eq!(
            report.status,
            CexLiveAdapterBoundaryReviewStatus::BlockedPendingLiveAdapterImplementation
        );
        assert!(report.rest_request_plan_validated);
        assert!(report.websocket_request_plan_validated);
        assert!(report.lifecycle_transcript_parsing_validated);
        assert!(report.balance_snapshot_parsing_validated);
        assert!(report.credential_scope_reviewed);
        assert!(report.rate_limit_reviewed);
        assert!(report.exchange_matching_rules_validated);
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| { code == "sandbox-order-lifecycle-evidence-missing" }));
        assert!(report
            .blocker_codes
            .iter()
            .any(|code| { code == "production-idempotency-replay-evidence-missing" }));
        assert!(!report.credential_material_loaded);
        assert!(!report.rest_call_performed);
        assert!(!report.websocket_connection_opened);
        assert!(!report.external_submission_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn cex_live_adapter_boundary_rejects_side_effect_claims() {
        let error = super::review_cex_live_adapter_boundary(CexLiveAdapterBoundaryReviewRequest {
            review_id: "local-cex-live-adapter-boundary".to_owned(),
            connector_name: "binance-local-boundary".to_owned(),
            venue: exchange_venue("binance"),
            rest_request_plan_validated: true,
            websocket_request_plan_validated: true,
            lifecycle_transcript_parsing_validated: true,
            balance_snapshot_parsing_validated: true,
            credential_scope_reviewed: true,
            rate_limit_reviewed: true,
            exchange_matching_rules_validated: true,
            sandbox_order_lifecycle_evidence_available: true,
            sandbox_balance_evidence_available: true,
            sandbox_cancel_evidence_available: true,
            production_idempotency_evidence_available: true,
            credential_material_loaded: true,
            rest_call_performed: false,
            websocket_connection_opened: false,
            external_submission_performed: false,
            live_execution_performed: false,
            production_ready_claimed: true,
            validated_at_unix_ms: 100_000,
        })
        .expect_err("side-effect claims must be rejected");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_LIVE_ADAPTER_REVIEW_SIDE_EFFECT"));
    }

    #[test]
    fn local_exchange_fixture_rejects_venue_specific_rule_mismatches() {
        let adapter = exchange_adapter("kraken");
        let mut request = exchange_order_request("kraken");
        request.quantity_base = 0.000_01;
        request.limit_price_quote = Some(50_000.05);
        request.notional_quote = 0.50;

        let error = adapter
            .validate_exchange_fixture_order(
                &CexExchangeMatchingRules::kraken_btc_usdc_fixture()
                    .expect("kraken rules should validate"),
                &request,
            )
            .expect_err("kraken fixture should reject quantity, tick, and notional mismatches");
        let violation_codes: Vec<_> = error
            .violations()
            .iter()
            .map(|violation| violation.code())
            .collect();

        assert!(violation_codes.contains(&"CEX_MATCHING_MIN_QUANTITY_NOT_MET"));
        assert!(violation_codes.contains(&"CEX_MATCHING_QUANTITY_STEP_NOT_MET"));
        assert!(violation_codes.contains(&"CEX_MATCHING_MIN_NOTIONAL_NOT_MET"));
        assert!(violation_codes.contains(&"CEX_MATCHING_PRICE_TICK_NOT_MET"));
    }

    #[test]
    fn local_exchange_fixture_rejects_unsupported_ioc_market_order() {
        let adapter = exchange_adapter("kraken");
        let mut request = exchange_order_request("kraken");
        request.order_type = CexOrderType::Market;
        request.time_in_force = CexTimeInForce::Ioc;
        request.limit_price_quote = None;

        let error = adapter
            .validate_exchange_fixture_order(
                &CexExchangeMatchingRules::kraken_btc_usdc_fixture()
                    .expect("kraken rules should validate"),
                &request,
            )
            .expect_err("kraken fixture should reject unsupported IOC market order");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_MATCHING_IOC_MARKET_UNSUPPORTED"));
    }

    #[test]
    fn local_exchange_market_data_transcripts_parse_named_order_books_without_network() {
        let cases = [
            (
                CexExchangeMarketDataFormat::BinanceDepth,
                "binance",
                r#"{"lastUpdateId":42,"bids":[["49990.00","1.25"],["49980.00","0.50"]],"asks":[["50010.00","0.75"],["50020.00","0.40"]]}"#,
                Some("42"),
                49_990.0,
                50_010.0,
            ),
            (
                CexExchangeMarketDataFormat::CoinbaseProductBook,
                "coinbase",
                r#"{"sequence":84,"bids":[["49991.00","1.10",1]],"asks":[["50011.00","0.90",1]]}"#,
                Some("84"),
                49_991.0,
                50_011.0,
            ),
            (
                CexExchangeMarketDataFormat::KrakenDepth,
                "kraken",
                r#"{"error":[],"result":{"XBTUSDC":{"b":[["49992.0","1.05","1700000000"]],"a":[["50012.0","0.95","1700000000"]]}}}"#,
                Some("XBTUSDC"),
                49_992.0,
                50_012.0,
            ),
        ];

        for (format, venue_name, payload, sequence, expected_bid, expected_ask) in cases {
            let transcript = CexMockMarketDataTranscript::new(
                format!("{venue_name}-depth-transcript"),
                format,
                exchange_venue(venue_name),
                MarketPair::new("BTC", "USDC").expect("pair should validate"),
                payload,
                1_700_000_000,
                1_700_000_001,
            )
            .expect("transcript should validate");
            let book = transcript
                .parse_order_book_snapshot()
                .expect("exchange transcript should parse");
            let quote = transcript
                .parse_top_of_book()
                .expect("top-of-book should parse from exchange transcript");

            assert_eq!(book.venue.name, venue_name);
            assert_eq!(book.source_sequence.as_deref(), sequence);
            assert!((quote.bid.price_quote - expected_bid).abs() < f64::EPSILON);
            assert!((quote.ask.price_quote - expected_ask).abs() < f64::EPSILON);
            assert!(!transcript.rest_call_performed);
            assert!(!transcript.websocket_connection_opened);
            assert!(!transcript.credentials_loaded);
            assert!(!transcript.live_execution_performed);
        }
    }

    #[test]
    fn local_exchange_market_data_transcripts_fail_closed_on_side_effects_or_bad_payloads() {
        let mut side_effect_transcript = CexMockMarketDataTranscript::new(
            "binance-side-effect-transcript",
            CexExchangeMarketDataFormat::BinanceDepth,
            exchange_venue("binance"),
            MarketPair::new("BTC", "USDC").expect("pair should validate"),
            r#"{"lastUpdateId":42,"bids":[["49990.00","1.25"]],"asks":[["50010.00","0.75"]]}"#,
            1_700_000_000,
            1_700_000_001,
        )
        .expect("baseline transcript should validate");
        side_effect_transcript.rest_call_performed = true;
        let side_effect_error = side_effect_transcript
            .parse_order_book_snapshot()
            .expect_err("side-effect transcript must fail closed");
        assert!(side_effect_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_TRANSCRIPT_EXTERNAL_SIDE_EFFECT"));

        let malformed_transcript = CexMockMarketDataTranscript::new(
            "kraken-error-transcript",
            CexExchangeMarketDataFormat::KrakenDepth,
            exchange_venue("kraken"),
            MarketPair::new("BTC", "USDC").expect("pair should validate"),
            r#"{"error":["EGeneral:Invalid arguments"],"result":{}}"#,
            1_700_000_000,
            1_700_000_001,
        )
        .expect("malformed payload shape is checked during parse");
        let malformed_error = malformed_transcript
            .parse_order_book_snapshot()
            .expect_err("Kraken error response must fail closed");
        assert!(malformed_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_KRAKEN_ERROR_RESPONSE"));
    }

    #[test]
    fn local_cex_market_data_request_plans_build_exchange_specific_shapes_without_execution() {
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let plans = [
            CexMarketDataRequestPlan::binance_depth_rest(
                "binance-rest-depth-plan",
                exchange_venue("binance"),
                pair.clone(),
                100,
            )
            .expect("binance REST plan should validate"),
            CexMarketDataRequestPlan::binance_depth_websocket(
                "binance-ws-depth-plan",
                exchange_venue("binance"),
                pair.clone(),
            )
            .expect("binance WebSocket plan should validate"),
            CexMarketDataRequestPlan::coinbase_product_book_rest(
                "coinbase-rest-book-plan",
                exchange_venue("coinbase"),
                pair.clone(),
                2,
            )
            .expect("coinbase REST plan should validate"),
            CexMarketDataRequestPlan::coinbase_product_book_websocket(
                "coinbase-ws-book-plan",
                exchange_venue("coinbase"),
                pair.clone(),
            )
            .expect("coinbase WebSocket plan should validate"),
            CexMarketDataRequestPlan::kraken_depth_rest(
                "kraken-rest-depth-plan",
                exchange_venue("kraken"),
                pair.clone(),
                100,
            )
            .expect("kraken REST plan should validate"),
            CexMarketDataRequestPlan::kraken_depth_websocket(
                "kraken-ws-depth-plan",
                exchange_venue("kraken"),
                pair,
            )
            .expect("kraken WebSocket plan should validate"),
        ];

        assert_eq!(plans[0].rest_path.as_deref(), Some("/api/v3/depth"));
        assert_eq!(
            plans[0].rest_query.as_deref(),
            Some("symbol=BTCUSDC&limit=100")
        );
        assert_eq!(plans[1].websocket_channel.as_deref(), Some("depth"));
        assert!(plans[1]
            .websocket_subscription_json
            .as_deref()
            .expect("subscription json")
            .contains("btcusdc@depth"));
        assert_eq!(
            plans[2].rest_path.as_deref(),
            Some("/products/BTC-USDC/book")
        );
        assert_eq!(plans[2].rest_query.as_deref(), Some("level=2"));
        assert_eq!(plans[3].websocket_channel.as_deref(), Some("level2"));
        assert_eq!(plans[4].rest_path.as_deref(), Some("/0/public/Depth"));
        assert_eq!(
            plans[4].rest_query.as_deref(),
            Some("pair=XBTUSDC&count=100")
        );
        assert_eq!(plans[5].websocket_channel.as_deref(), Some("book"));

        for plan in plans {
            plan.validate().expect("plan should remain valid");
            assert!(!plan.rest_call_performed);
            assert!(!plan.websocket_connection_opened);
            assert!(!plan.credentials_loaded);
            assert!(!plan.live_execution_performed);
            assert!(!plan.production_ready);
        }
    }

    #[test]
    fn local_cex_market_data_request_plan_parses_matching_transcript() {
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let plan = CexMarketDataRequestPlan::binance_depth_rest(
            "binance-rest-depth-plan",
            exchange_venue("binance"),
            pair.clone(),
            100,
        )
        .expect("plan should validate");
        let transcript = CexMockMarketDataTranscript::new(
            "binance-depth-transcript",
            CexExchangeMarketDataFormat::BinanceDepth,
            exchange_venue("binance"),
            pair,
            r#"{"lastUpdateId":42,"bids":[["49990.00","1.25"]],"asks":[["50010.00","0.75"]]}"#,
            1_700_000_000,
            1_700_000_001,
        )
        .expect("transcript should validate");

        let book = plan
            .parse_transcript(&transcript)
            .expect("matching local transcript should parse");

        assert_eq!(book.source_sequence.as_deref(), Some("42"));
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 1);
    }

    #[test]
    fn local_cex_market_data_request_plan_fails_closed_on_side_effects_or_mismatch() {
        let pair = MarketPair::new("BTC", "USDC").expect("pair should validate");
        let mut side_effect_plan = CexMarketDataRequestPlan::coinbase_product_book_rest(
            "coinbase-rest-book-plan",
            exchange_venue("coinbase"),
            pair.clone(),
            2,
        )
        .expect("baseline plan should validate");
        side_effect_plan.rest_call_performed = true;
        let side_effect_error = side_effect_plan
            .validate()
            .expect_err("side-effect plan must fail closed");
        assert!(side_effect_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_REQUEST_PLAN_EXTERNAL_SIDE_EFFECT"));

        let plan = CexMarketDataRequestPlan::coinbase_product_book_rest(
            "coinbase-rest-book-plan",
            exchange_venue("coinbase"),
            pair,
            2,
        )
        .expect("plan should validate");
        let transcript = CexMockMarketDataTranscript::new(
            "binance-depth-transcript",
            CexExchangeMarketDataFormat::BinanceDepth,
            exchange_venue("binance"),
            MarketPair::new("BTC", "USDC").expect("pair should validate"),
            r#"{"lastUpdateId":42,"bids":[["49990.00","1.25"]],"asks":[["50010.00","0.75"]]}"#,
            1_700_000_000,
            1_700_000_001,
        )
        .expect("transcript should validate");
        let mismatch_error = plan
            .parse_transcript(&transcript)
            .expect_err("mismatched plan/transcript must fail closed");
        let codes: Vec<_> = mismatch_error
            .violations()
            .iter()
            .map(|violation| violation.code())
            .collect();
        assert!(codes.contains(&"CEX_REQUEST_PLAN_TRANSCRIPT_FORMAT_MISMATCH"));
        assert!(codes.contains(&"CEX_REQUEST_PLAN_TRANSCRIPT_VENUE_MISMATCH"));
    }

    #[test]
    fn local_cex_rate_limit_validation_reports_ready_budget_without_provider_calls() {
        let report = validate_cex_rate_limit(
            CexRateLimitObservation::new(
                "binance-rest-budget-1",
                exchange_venue("binance"),
                CexRateLimitScope::RestMarketData,
                1_200,
                60_000,
                10,
                None,
                false,
            )
            .expect("rate-limit observation should validate"),
        )
        .expect("rate-limit report should validate");

        assert_eq!(report.status, CexRateLimitStatus::ReadyForLocalReview);
        assert_eq!(report.remaining_requests_in_window, 1_190);
        assert!(!report.local_budget_exhausted);
        assert!(!report.provider_rate_limited);
        assert!(!report.live_provider_call_performed);
        assert!(!report.websocket_connection_opened);
        assert!(!report.credential_loaded);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn local_cex_rate_limit_validation_blocks_exhausted_or_provider_limited_budget() {
        let exhausted_report = validate_cex_rate_limit(
            CexRateLimitObservation::new(
                "coinbase-rest-budget-exhausted",
                exchange_venue("coinbase"),
                CexRateLimitScope::RestMarketData,
                10,
                1_000,
                10,
                Some(500),
                false,
            )
            .expect("rate-limit observation should validate"),
        )
        .expect("exhausted rate-limit report should validate");
        assert_eq!(exhausted_report.status, CexRateLimitStatus::Blocked);
        assert!(exhausted_report.local_budget_exhausted);
        assert!(exhausted_report
            .violation_codes
            .contains(&"CEX_RATE_LIMIT_LOCAL_BUDGET_EXHAUSTED".to_owned()));

        let provider_limited_report = validate_cex_rate_limit(
            CexRateLimitObservation::new(
                "kraken-provider-rate-limited",
                exchange_venue("kraken"),
                CexRateLimitScope::WebSocketMarketData,
                20,
                60_000,
                2,
                Some(1_000),
                true,
            )
            .expect("rate-limit observation should validate"),
        )
        .expect("provider-limited report should validate");
        assert_eq!(provider_limited_report.status, CexRateLimitStatus::Blocked);
        assert!(provider_limited_report.provider_rate_limited);
        assert!(provider_limited_report
            .violation_codes
            .contains(&"CEX_RATE_LIMIT_PROVIDER_SIGNALED".to_owned()));
    }

    #[test]
    fn local_cex_rate_limit_validation_fails_closed_on_side_effect_observation() {
        let mut observation = CexRateLimitObservation::new(
            "binance-rate-limit-side-effect",
            exchange_venue("binance"),
            CexRateLimitScope::OrderSubmission,
            5,
            1_000,
            1,
            None,
            false,
        )
        .expect("baseline rate-limit observation should validate");
        observation.credential_loaded = true;

        let error = validate_cex_rate_limit(observation)
            .expect_err("side-effect rate-limit observation must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_RATE_LIMIT_EXTERNAL_SIDE_EFFECT"));
    }

    #[test]
    fn local_cex_credential_scope_review_accepts_sanitized_required_permissions() {
        let report = validate_cex_credential_scope_review(
            CexCredentialScopeReviewInput::new(
                "binance-scope-review-1",
                exchange_venue("binance"),
                SecretRef::Keystore {
                    alias: "binance-paper-api-key".to_owned(),
                },
                vec![
                    CexCredentialPermission::ReadOnlyMarketData,
                    CexCredentialPermission::ReadBalances,
                    CexCredentialPermission::TradeOrders,
                    CexCredentialPermission::CancelOrders,
                ],
                vec![
                    CexCredentialPermission::ReadOnlyMarketData,
                    CexCredentialPermission::ReadBalances,
                    CexCredentialPermission::TradeOrders,
                    CexCredentialPermission::CancelOrders,
                ],
                vec![
                    CexCredentialPermission::Withdrawals,
                    CexCredentialPermission::Transfers,
                    CexCredentialPermission::MarginOrDerivatives,
                    CexCredentialPermission::AccountAdmin,
                ],
                1_700_000_000_000,
                1_700_001_000_000,
                86_400_000,
            )
            .expect("credential-scope input should validate"),
        )
        .expect("credential-scope report should validate");

        assert_eq!(
            report.status,
            CexCredentialScopeReviewStatus::ReadyForLocalReview
        );
        assert!(report.missing_required_permissions.is_empty());
        assert!(report.forbidden_permissions_present.is_empty());
        assert!(report.credential_reference_validated);
        assert!(report.fee_schedule_reviewed);
        assert!(report.rate_limit_documentation_reviewed);
        assert!(report.terms_of_service_reviewed);
        assert!(report.jurisdiction_reviewed);
        assert!(report.api_capabilities_reviewed);
        assert!(report.incident_reputation_reviewed);
        assert!(report.governance_review_passed);
        assert!(!report.secret_material_loaded);
        assert!(!report.credential_plaintext_seen);
        assert!(!report.live_provider_call_performed);
        assert!(!report.account_state_queried);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        assert!(report.violation_codes.is_empty());
    }

    #[test]
    fn local_cex_credential_scope_review_blocks_forbidden_permissions() {
        let mut input = CexCredentialScopeReviewInput::new(
            "coinbase-withdrawal-scope",
            exchange_venue("coinbase"),
            SecretRef::Keystore {
                alias: "coinbase-paper-api-key".to_owned(),
            },
            vec![CexCredentialPermission::ReadOnlyMarketData],
            vec![
                CexCredentialPermission::ReadOnlyMarketData,
                CexCredentialPermission::Withdrawals,
                CexCredentialPermission::AccountAdmin,
            ],
            vec![CexCredentialPermission::Withdrawals],
            1_700_000_000_000,
            1_700_000_500_000,
            86_400_000,
        )
        .expect("credential-scope input should validate");
        input.fee_schedule_reviewed = false;
        input.rate_limit_documentation_reviewed = false;
        input.terms_of_service_reviewed = false;
        input.jurisdiction_reviewed = false;
        input.api_capabilities_reviewed = false;
        input.incident_reputation_reviewed = false;

        let report = validate_cex_credential_scope_review(input)
            .expect("forbidden-permission report should validate");

        assert_eq!(report.status, CexCredentialScopeReviewStatus::Blocked);
        assert!(report
            .forbidden_permissions_present
            .contains(&CexCredentialPermission::Withdrawals));
        assert!(report
            .forbidden_permissions_present
            .contains(&CexCredentialPermission::AccountAdmin));
        assert!(!report.fee_schedule_reviewed);
        assert!(!report.rate_limit_documentation_reviewed);
        assert!(!report.terms_of_service_reviewed);
        assert!(!report.jurisdiction_reviewed);
        assert!(!report.api_capabilities_reviewed);
        assert!(!report.incident_reputation_reviewed);
        assert!(!report.governance_review_passed);
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_FORBIDDEN_PERMISSION_PRESENT".to_owned()));
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_FEE_REVIEW_MISSING".to_owned()));
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_RATE_LIMIT_DOCUMENTATION_MISSING".to_owned()));
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_TERMS_REVIEW_MISSING".to_owned()));
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_JURISDICTION_REVIEW_MISSING".to_owned()));
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_API_CAPABILITIES_REVIEW_MISSING".to_owned()));
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_INCIDENT_REPUTATION_REVIEW_MISSING".to_owned()));
    }

    #[test]
    fn local_cex_credential_scope_review_blocks_stale_review() {
        let report = validate_cex_credential_scope_review(
            CexCredentialScopeReviewInput::new(
                "kraken-stale-scope-review",
                exchange_venue("kraken"),
                SecretRef::Keystore {
                    alias: "kraken-paper-api-key".to_owned(),
                },
                vec![CexCredentialPermission::ReadOnlyMarketData],
                vec![CexCredentialPermission::ReadOnlyMarketData],
                vec![CexCredentialPermission::Withdrawals],
                1_700_000_000_000,
                1_700_200_000_000,
                86_400_000,
            )
            .expect("credential-scope input should validate"),
        )
        .expect("stale report should validate");

        assert_eq!(report.status, CexCredentialScopeReviewStatus::Blocked);
        assert!(report.stale_review);
        assert!(report
            .violation_codes
            .contains(&"CEX_CREDENTIAL_SCOPE_REVIEW_STALE".to_owned()));
    }

    #[test]
    fn local_cex_credential_scope_review_fails_closed_on_secret_or_live_side_effects() {
        let mut input = CexCredentialScopeReviewInput::new(
            "binance-secret-loaded-scope-review",
            exchange_venue("binance"),
            SecretRef::Keystore {
                alias: "binance-paper-api-key".to_owned(),
            },
            vec![CexCredentialPermission::ReadOnlyMarketData],
            vec![CexCredentialPermission::ReadOnlyMarketData],
            vec![CexCredentialPermission::Withdrawals],
            1_700_000_000_000,
            1_700_000_500_000,
            86_400_000,
        )
        .expect("baseline credential-scope input should validate");
        input.secret_material_loaded = true;
        input.live_provider_call_performed = true;

        let error = validate_cex_credential_scope_review(input)
            .expect_err("side-effect credential-scope review must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| { violation.code() == "CEX_CREDENTIAL_SCOPE_EXTERNAL_SIDE_EFFECT" }));
    }

    #[test]
    fn cex_order_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_path("cex-validation-audit", "jsonl");
        let state_path = temp_path("cex-validation-state", "sqlite");
        let approval = policy_gate()
            .validate_order(&profile(), &order_request())
            .expect("paper order should be policy approved");
        let record = CexOrderValidationRecord::from_approved_request(&order_request(), &approval)
            .expect("validation record should build");

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        let audit_record = append_cex_order_validation_audit(&mut journal, &record, 1_700_000_001)
            .expect("audit append should pass");
        let checkpoint =
            persist_cex_order_validation_checkpoint(&mut store, &record, 1_700_000_002)
                .expect("checkpoint should persist");
        assert_eq!(audit_record.event.subsystem, CEX_STATE_SUBSYSTEM);
        assert_eq!(checkpoint.key, CEX_LAST_ORDER_VALIDATION_CHECKPOINT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let stored = reopened
            .get_checkpoint(CEX_LAST_ORDER_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        let recovered: CexOrderValidationRecord =
            serde_json::from_str(&stored.value).expect("checkpoint decodes");
        assert_eq!(recovered.request_id, "cex-order-1");
        assert!(!recovered.external_submission_performed);
        assert!(!recovered.live_execution_performed);

        let _ = fs::remove_file(audit_path);
        cleanup_sqlite(&state_path);
    }

    #[test]
    fn cex_order_lifecycle_reconciles_fills_and_reopens_locally() {
        let audit_path = temp_path("cex-lifecycle-audit", "jsonl");
        let state_path = temp_path("cex-lifecycle-state", "sqlite");
        let validation = approved_validation_record();
        let responses = vec![
            lifecycle_response(
                "cex-response-accepted",
                CexOrderStatus::Accepted,
                0.0,
                None,
                0.0,
                1_700_000_010,
            ),
            lifecycle_response(
                "cex-response-partial",
                CexOrderStatus::PartiallyFilled,
                0.004,
                Some(100.0),
                0.0008,
                1_700_000_011,
            ),
            lifecycle_response(
                "cex-response-filled",
                CexOrderStatus::Filled,
                0.006,
                Some(101.0),
                0.0012,
                1_700_000_012,
            ),
        ];
        let record = CexOrderLifecycleRecord::from_local_responses(&validation, &responses, false)
            .expect("local lifecycle reconciliation should pass");

        assert_eq!(record.final_status, CexOrderStatus::Filled);
        assert_eq!(record.transition_count, 3);
        assert_eq!(record.fill_count, 2);
        assert!((record.filled_quantity_base - 0.01).abs() < f64::EPSILON);
        assert!(record.remaining_quantity_base.abs() < f64::EPSILON);
        assert!((record.average_fill_price_quote.expect("average price") - 100.6).abs() < 1e-9);
        assert!((record.total_fee_quote - 0.002).abs() < f64::EPSILON);
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        append_cex_order_lifecycle_audit(&mut journal, &record, 1_700_000_013)
            .expect("lifecycle audit append should pass");
        persist_cex_order_lifecycle_checkpoint(&mut store, &record, 1_700_000_014)
            .expect("lifecycle checkpoint should persist");
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let stored = reopened
            .get_checkpoint(CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY)
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        let recovered: CexOrderLifecycleRecord =
            serde_json::from_str(&stored.value).expect("checkpoint decodes");
        assert_eq!(recovered.final_status, CexOrderStatus::Filled);
        assert!(
            (recovered.filled_quantity_base - record.filled_quantity_base).abs() < f64::EPSILON
        );

        let _ = fs::remove_file(audit_path);
        cleanup_sqlite(&state_path);
    }

    #[test]
    fn cex_order_lifecycle_rejects_invalid_status_transition() {
        let validation = approved_validation_record();
        let responses = vec![
            lifecycle_response(
                "cex-response-filled",
                CexOrderStatus::Filled,
                1.0,
                Some(100.0),
                0.2,
                1_700_000_020,
            ),
            lifecycle_response(
                "cex-response-late-accepted",
                CexOrderStatus::Accepted,
                0.0,
                None,
                0.0,
                1_700_000_021,
            ),
        ];

        let error = CexOrderLifecycleRecord::from_local_responses(&validation, &responses, false)
            .expect_err("filled to accepted transition must fail closed");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_ORDER_LIFECYCLE_TRANSITION_INVALID"));
    }

    #[test]
    fn cex_client_order_id_uniqueness_rejects_duplicates() {
        let first = approved_validation_record();
        let mut duplicate = first.clone();
        duplicate.request_id = "cex-order-duplicate".to_owned();

        let error = validate_cex_client_order_id_uniqueness(&[first, duplicate])
            .expect_err("duplicate client order ids must fail closed");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_CLIENT_ORDER_ID_DUPLICATE"));
    }

    #[test]
    fn cex_order_lifecycle_persistence_rejects_side_effect_records() {
        let audit_path = temp_path("cex-lifecycle-invalid-audit", "jsonl");
        let state_path = temp_path("cex-lifecycle-invalid-state", "sqlite");
        let validation = approved_validation_record();
        let responses = vec![lifecycle_response(
            "cex-response-accepted",
            CexOrderStatus::Accepted,
            0.0,
            None,
            0.0,
            1_700_000_030,
        )];
        let mut record =
            CexOrderLifecycleRecord::from_local_responses(&validation, &responses, false)
                .expect("baseline lifecycle should validate");
        record.external_submission_performed = true;

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        let audit_error = append_cex_order_lifecycle_audit(&mut journal, &record, 1_700_000_031)
            .expect_err("side-effect lifecycle audit must fail closed");
        let checkpoint_error =
            persist_cex_order_lifecycle_checkpoint(&mut store, &record, 1_700_000_032)
                .expect_err("side-effect lifecycle checkpoint must fail closed");

        assert!(audit_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_LIFECYCLE_EXTERNAL_SIDE_EFFECT"));
        assert!(checkpoint_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_LIFECYCLE_EXTERNAL_SIDE_EFFECT"));
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 1);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        assert!(reopened
            .get_checkpoint(CEX_LAST_ORDER_LIFECYCLE_CHECKPOINT_KEY)
            .expect("checkpoint lookup should succeed")
            .is_none());

        let _ = fs::remove_file(audit_path);
        cleanup_sqlite(&state_path);
    }

    #[test]
    fn cex_order_lifecycle_transcripts_parse_exchange_shapes_locally() {
        let validation = approved_validation_record();
        let transcripts = [
            lifecycle_transcript(
                "binance-accepted-transcript",
                CexOrderLifecycleTranscriptFormat::BinanceExecutionReport,
                r#"{"c":"client-cex-order-1","i":"binance-order-1","X":"NEW","l":"0","L":"0","n":"0"}"#,
                1_700_000_041,
            ),
            lifecycle_transcript(
                "coinbase-partial-transcript",
                CexOrderLifecycleTranscriptFormat::CoinbaseOrderEvent,
                r#"{"client_order_id":"client-cex-order-1","order_id":"coinbase-order-1","status":"match","last_fill_size":"0.004","price":"100.0","fee":"0.0008"}"#,
                1_700_000_042,
            ),
            lifecycle_transcript(
                "kraken-filled-transcript",
                CexOrderLifecycleTranscriptFormat::KrakenOrderStatus,
                r#"{"userref":"client-cex-order-1","txid":"kraken-order-1","status":"closed","vol_exec_delta":"0.006","price":"101.0","fee":"0.0012"}"#,
                1_700_000_043,
            ),
        ];
        let responses = transcripts
            .iter()
            .map(|transcript| {
                transcript
                    .parse_lifecycle_response(&validation)
                    .expect("local lifecycle transcript should parse")
            })
            .collect::<Vec<_>>();

        let record = CexOrderLifecycleRecord::from_local_responses(&validation, &responses, true)
            .expect("parsed lifecycle responses should reconcile");

        assert_eq!(record.final_status, CexOrderStatus::Filled);
        assert_eq!(record.transition_count, 3);
        assert_eq!(record.fill_count, 2);
        assert!((record.filled_quantity_base - 0.01).abs() < f64::EPSILON);
        assert!((record.total_fee_quote - 0.002).abs() < 1e-12);
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);
    }

    #[test]
    fn cex_order_lifecycle_transcripts_reconcile_cancelled_remainder_locally() {
        let validation = approved_validation_record();
        let transcripts = [
            lifecycle_transcript(
                "binance-cancel-accepted-transcript",
                CexOrderLifecycleTranscriptFormat::BinanceExecutionReport,
                r#"{"c":"client-cex-order-1","i":"binance-cancel-order-1","X":"NEW","l":"0","L":"0","n":"0"}"#,
                1_700_000_061,
            ),
            lifecycle_transcript(
                "coinbase-cancel-partial-transcript",
                CexOrderLifecycleTranscriptFormat::CoinbaseOrderEvent,
                r#"{"client_order_id":"client-cex-order-1","order_id":"coinbase-cancel-order-1","status":"match","last_fill_size":"0.004","price":"100.0","fee":"0.0008"}"#,
                1_700_000_062,
            ),
            lifecycle_transcript(
                "kraken-cancelled-transcript",
                CexOrderLifecycleTranscriptFormat::KrakenOrderStatus,
                r#"{"userref":"client-cex-order-1","txid":"kraken-cancel-order-1","status":"canceled","vol_exec_delta":"0","price":"0","fee":"0"}"#,
                1_700_000_063,
            ),
        ];
        let responses = transcripts
            .iter()
            .map(|transcript| {
                transcript
                    .parse_lifecycle_response(&validation)
                    .expect("local cancel lifecycle transcript should parse")
            })
            .collect::<Vec<_>>();

        let record = CexOrderLifecycleRecord::from_local_responses(&validation, &responses, true)
            .expect("parsed cancel lifecycle responses should reconcile");

        assert_eq!(record.final_status, CexOrderStatus::Cancelled);
        assert_eq!(record.transition_count, 3);
        assert_eq!(record.fill_count, 1);
        assert!((record.filled_quantity_base - 0.004).abs() < f64::EPSILON);
        assert!((record.remaining_quantity_base - 0.006).abs() < 1e-12);
        assert!((record.total_fee_quote - 0.0008).abs() < 1e-12);
        assert!(!record.external_submission_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);
    }

    #[test]
    fn cex_order_lifecycle_transcripts_fail_closed_on_side_effects_or_mismatch() {
        let validation = approved_validation_record();
        let mut side_effect_transcript = lifecycle_transcript(
            "binance-side-effect-lifecycle-transcript",
            CexOrderLifecycleTranscriptFormat::BinanceExecutionReport,
            r#"{"c":"client-cex-order-1","i":"binance-order-1","X":"NEW"}"#,
            1_700_000_051,
        );
        side_effect_transcript.rest_call_performed = true;
        let side_effect_error = side_effect_transcript
            .parse_lifecycle_response(&validation)
            .expect_err("side-effect transcript must fail closed");
        assert!(side_effect_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_LIFECYCLE_TRANSCRIPT_EXTERNAL_SIDE_EFFECT"));

        let mismatched_transcript = CexOrderLifecycleTranscript::new(
            "coinbase-mismatch-lifecycle-transcript",
            CexOrderLifecycleTranscriptFormat::CoinbaseOrderEvent,
            VenueRef {
                name: "other-cex".to_owned(),
                kind: VenueKind::Cex,
            },
            order_request().pair,
            r#"{"client_order_id":"client-cex-order-1","order_id":"coinbase-order-1","status":"open"}"#,
            1_700_000_052,
            1_700_000_053,
        )
        .expect("baseline mismatch transcript validates");
        let mismatch_error = mismatched_transcript
            .parse_lifecycle_response(&validation)
            .expect_err("venue mismatch must fail closed");
        assert!(mismatch_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_LIFECYCLE_TRANSCRIPT_VENUE_MISMATCH"));
    }

    #[test]
    fn cex_balance_snapshots_parse_exchange_shapes_locally() {
        let venue = VenueRef {
            name: "paper-cex".to_owned(),
            kind: VenueKind::Cex,
        };
        let transcripts = [
            CexBalanceSnapshotTranscript::new(
                "binance-balance-transcript",
                CexBalanceSnapshotTranscriptFormat::BinanceAccountBalances,
                venue.clone(),
                r#"{"balances":[{"asset":"BTC","free":"1.25","locked":"0.10"},{"asset":"USDC","free":"5000","locked":"25"}]}"#,
                1_700_000_071,
                1_700_000_072,
            )
            .expect("Binance balance transcript validates"),
            CexBalanceSnapshotTranscript::new(
                "coinbase-balance-transcript",
                CexBalanceSnapshotTranscriptFormat::CoinbaseAccounts,
                venue.clone(),
                r#"{"accounts":[{"currency":"BTC","available_balance":{"value":"0.75"},"hold":{"value":"0.05"}},{"currency":"USD","available_balance":{"value":"1000"},"hold":{"value":"0"}}]}"#,
                1_700_000_073,
                1_700_000_074,
            )
            .expect("Coinbase balance transcript validates"),
            CexBalanceSnapshotTranscript::new(
                "kraken-balance-transcript",
                CexBalanceSnapshotTranscriptFormat::KrakenBalance,
                venue,
                r#"{"result":{"XXBT":"0.5","ZUSD":"1250"}}"#,
                1_700_000_075,
                1_700_000_076,
            )
            .expect("Kraken balance transcript validates"),
        ];

        let snapshots = transcripts
            .iter()
            .map(|transcript| {
                transcript
                    .parse_snapshot()
                    .expect("local balance transcript should parse")
            })
            .collect::<Vec<_>>();

        assert_eq!(snapshots.len(), 3);
        assert_eq!(snapshots[0].balances.len(), 2);
        assert_eq!(snapshots[0].balances[0].asset, "BTC");
        assert!((snapshots[0].balances[0].available - 1.25).abs() < f64::EPSILON);
        assert!((snapshots[0].balances[0].total - 1.35).abs() < 1e-12);
        assert_eq!(snapshots[2].balances[0].asset, "BTC");
        assert!(!snapshots.iter().any(|snapshot| snapshot.credentials_loaded));
        assert!(!snapshots
            .iter()
            .any(|snapshot| snapshot.account_state_queried));
        assert!(!snapshots
            .iter()
            .any(|snapshot| snapshot.live_execution_performed));
        assert!(!snapshots.iter().any(|snapshot| snapshot.production_ready));
    }

    #[test]
    fn cex_balance_snapshots_fail_closed_on_side_effects_or_duplicate_assets() {
        let venue = VenueRef {
            name: "paper-cex".to_owned(),
            kind: VenueKind::Cex,
        };
        let mut side_effect_transcript = CexBalanceSnapshotTranscript::new(
            "binance-balance-side-effect",
            CexBalanceSnapshotTranscriptFormat::BinanceAccountBalances,
            venue.clone(),
            r#"{"balances":[{"asset":"BTC","free":"1","locked":"0"}]}"#,
            1_700_000_081,
            1_700_000_082,
        )
        .expect("baseline balance transcript validates");
        side_effect_transcript.account_state_queried = true;
        let side_effect_error = side_effect_transcript
            .parse_snapshot()
            .expect_err("account-query transcript must fail closed");
        assert!(side_effect_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_BALANCE_TRANSCRIPT_EXTERNAL_SIDE_EFFECT"));

        let duplicate_transcript = CexBalanceSnapshotTranscript::new(
            "binance-balance-duplicate",
            CexBalanceSnapshotTranscriptFormat::BinanceAccountBalances,
            venue,
            r#"{"balances":[{"asset":"BTC","free":"1","locked":"0"},{"asset":"btc","free":"2","locked":"0"}]}"#,
            1_700_000_083,
            1_700_000_084,
        )
        .expect("duplicate asset transcript validates before parse");
        let duplicate_error = duplicate_transcript
            .parse_snapshot()
            .expect_err("duplicate assets must fail closed");
        assert!(duplicate_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "CEX_BALANCE_ASSET_DUPLICATE"));
    }

    fn approved_validation_record() -> CexOrderValidationRecord {
        let approval = policy_gate()
            .validate_order(&profile(), &order_request())
            .expect("paper order should be policy approved");
        CexOrderValidationRecord::from_approved_request(&order_request(), &approval)
            .expect("validation record should build")
    }

    fn lifecycle_response(
        id: &str,
        status: CexOrderStatus,
        fill_quantity_base_delta: f64,
        fill_price_quote: Option<f64>,
        fee_quote_delta: f64,
        occurred_at_unix_ms: u64,
    ) -> CexOrderLifecycleResponse {
        CexOrderLifecycleResponse {
            id: id.to_owned(),
            request_id: "cex-order-1".to_owned(),
            client_order_id: "client-cex-order-1".to_owned(),
            exchange_order_id: if status == CexOrderStatus::Rejected {
                None
            } else {
                Some("mock-exchange-order-1".to_owned())
            },
            status,
            fill_quantity_base_delta,
            fill_price_quote,
            fee_quote_delta,
            locally_simulated: true,
            external_submission_performed: false,
            live_execution_performed: false,
            occurred_at_unix_ms,
        }
    }

    fn lifecycle_transcript(
        transcript_id: &str,
        format: CexOrderLifecycleTranscriptFormat,
        payload_json: &str,
        received_at_unix_ms: u64,
    ) -> CexOrderLifecycleTranscript {
        let request = order_request();
        CexOrderLifecycleTranscript::new(
            transcript_id,
            format,
            request.venue,
            request.pair,
            payload_json,
            received_at_unix_ms - 1,
            received_at_unix_ms,
        )
        .expect("static lifecycle transcript should validate")
    }
}
