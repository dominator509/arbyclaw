#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, DestinationPolicy,
    ExecutionIntent, ExecutionIntentKind, ExecutionScope, FeeModelError, FeeProvider, FeeSchedule,
    MarketPair, PolicyApproval, PolicyDecision, PolicyEngine, PolicyViolation, StateCheckpoint,
    StateStore, StateStoreError, VenueKind, VenueRef,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, error::Error, fmt};

/// Stable DEX/Web3 framework version for audit and future replay surfaces.
pub const DEX_CONNECTOR_FRAMEWORK_VERSION: &str = "phase-8-dex-web3-framework-v1";

/// State-store subsystem name for local DEX/Web3 framework checkpoints.
pub const DEX_STATE_SUBSYSTEM: &str = "dex";

/// Checkpoint key for the latest locally validated DEX/Web3 swap quote.
pub const DEX_LAST_SWAP_VALIDATION_CHECKPOINT_KEY: &str = "dex.last_swap_validation";

/// Checkpoint key for the latest locally reconciled DEX/Web3 lifecycle record.
pub const DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY: &str = "dex.last_swap_lifecycle";
/// Checkpoint key for the latest local Web3 pre-sign safety review.
pub const DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY: &str = "dex.last_web3_pre_sign_safety";
/// Checkpoint key for the latest local Web3 nonce reservation plan.
pub const DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY: &str = "dex.last_web3_nonce_reservation";
/// Checkpoint key for the latest local Web3 unsigned payload review.
pub const DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY: &str =
    "dex.last_web3_unsigned_payload_review";
/// Checkpoint key for the latest local Web3 broadcast-readiness review.
pub const DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY: &str =
    "dex.last_web3_broadcast_readiness";
/// Checkpoint key for the latest local Web3 unsigned transaction construction.
pub const DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY: &str =
    "dex.last_web3_unsigned_transaction_construction";
/// Checkpoint key for the latest local Web3 provider nonce reconciliation.
pub const DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY: &str =
    "dex.last_web3_provider_nonce_reconciliation";
/// Checkpoint key for the latest local Web3 raw transaction serialization review.
pub const DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY: &str =
    "dex.last_web3_raw_transaction_serialization_review";
/// Checkpoint key for the latest local Web3 broadcast adapter control review.
pub const DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY: &str =
    "dex.last_web3_broadcast_adapter_control_review";
/// Checkpoint key for the latest local Web3 sandbox/live discrepancy calibration.
pub const DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY: &str =
    "dex.last_web3_sandbox_live_discrepancy_calibration";

/// Web3 chain metadata used for allowlist and policy alignment.
///
/// This is non-secret metadata only. It does not contain RPC URLs, provider
/// tokens, private keys, wallet addresses, or chain-specific live clients.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3ChainProfile {
    /// Stable chain identifier used by config allowlists, such as ethereum or solana-mainnet.
    pub chain: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Native fee asset symbol, such as ETH or SOL.
    pub native_asset: String,
    /// Whether the future connector claims transaction simulation support.
    pub transaction_simulation_supported: bool,
    /// Whether the future connector has EIP-1559-style gas metadata.
    pub eip1559_fee_market: bool,
}

impl Web3ChainProfile {
    /// Create a chain profile after deterministic validation.
    pub fn new(
        chain: impl Into<String>,
        display_name: impl Into<String>,
        native_asset: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let profile = Self {
            chain: chain.into(),
            display_name: display_name.into(),
            native_asset: native_asset.into(),
            transaction_simulation_supported: false,
            eip1559_fee_market: false,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Validate this chain profile without network calls.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_non_empty("DEX_CHAIN_REQUIRED", "chain", &self.chain, &mut violations);
        validate_non_empty(
            "DEX_CHAIN_DISPLAY_NAME_REQUIRED",
            "chain display name",
            &self.display_name,
            &mut violations,
        );
        validate_non_empty(
            "DEX_CHAIN_NATIVE_ASSET_REQUIRED",
            "native fee asset",
            &self.native_asset,
            &mut violations,
        );
        finish_validation(violations)
    }
}

/// Public token metadata for a chain allowlist.
///
/// `contract_label` is a stable reviewed label, not a private key and not an
/// executable instruction. Future phases may map labels to verified addresses
/// outside arbitrary LLM-generated calldata paths.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexTokenProfile {
    /// Chain on which this token profile is valid.
    pub chain: String,
    /// Canonical token symbol used by strategy/policy allowlists.
    pub symbol: String,
    /// Reviewed contract or native-token label.
    pub contract_label: String,
    /// Token decimal precision.
    pub decimals: u8,
    /// Whether this is the native gas asset for the chain.
    pub native_asset: bool,
    /// Whether token metadata has been externally verified.
    pub externally_verified: bool,
}

impl DexTokenProfile {
    /// Create a token profile after deterministic validation.
    pub fn new(
        chain: impl Into<String>,
        symbol: impl Into<String>,
        contract_label: impl Into<String>,
        decimals: u8,
    ) -> Result<Self, DexConnectorError> {
        let profile = Self {
            chain: chain.into(),
            symbol: normalize_symbol(symbol.into()),
            contract_label: contract_label.into(),
            decimals,
            native_asset: false,
            externally_verified: false,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Validate this token profile without RPC or contract calls.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_non_empty(
            "DEX_TOKEN_CHAIN_REQUIRED",
            "token chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_TOKEN_SYMBOL_REQUIRED",
            "token symbol",
            &self.symbol,
            &mut violations,
        );
        validate_non_empty(
            "DEX_TOKEN_CONTRACT_LABEL_REQUIRED",
            "token contract label",
            &self.contract_label,
            &mut violations,
        );

        if self.decimals > 36 {
            violations.push(DexConnectorViolation::new(
                "DEX_TOKEN_DECIMALS_UNSUPPORTED",
                "token decimals must be 36 or lower for deterministic arithmetic boundaries",
            ));
        }

        finish_validation(violations)
    }
}

/// DEX/router connector capability declaration.
///
/// These are framework declarations only. A future adapter must prove each
/// capability with integration tests, simulation results, protocol review,
/// and audit/state wiring before use with live funds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexRouterCapabilities {
    /// Public quote generation support.
    pub quote: bool,
    /// Multi-hop or aggregator routing support.
    pub route_aggregation: bool,
    /// Exact-input swap modeling support.
    pub exact_input_swaps: bool,
    /// Exact-output swap modeling support.
    pub exact_output_swaps: bool,
    /// Transaction simulation support.
    pub transaction_simulation: bool,
    /// Gas estimation support.
    pub gas_estimation: bool,
    /// Allowance/spender metadata support.
    pub allowance_checks: bool,
    /// MEV/slippage risk metadata support.
    pub mev_risk_metadata: bool,
}

impl DexRouterCapabilities {
    /// Safest default for framework-only declarations.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            quote: false,
            route_aggregation: false,
            exact_input_swaps: false,
            exact_output_swaps: false,
            transaction_simulation: false,
            gas_estimation: false,
            allowance_checks: false,
            mev_risk_metadata: false,
        }
    }

    /// Conservative paper/simulation capability set.
    #[must_use]
    pub const fn paper_simulation() -> Self {
        Self {
            quote: true,
            route_aggregation: true,
            exact_input_swaps: true,
            exact_output_swaps: false,
            transaction_simulation: true,
            gas_estimation: true,
            allowance_checks: true,
            mev_risk_metadata: false,
        }
    }
}

/// Non-secret DEX/router venue profile.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexRouterProfile {
    /// Normalized venue reference. Kind must be `VenueKind::Dex` or `VenueKind::Aggregator`.
    pub venue: VenueRef,
    /// Chain on which this router profile applies.
    pub chain: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Stable reviewed router contract label.
    pub router_label: String,
    /// Stable reviewed spender label for approval hygiene.
    pub spender_label: String,
    /// Declared connector capabilities.
    pub capabilities: DexRouterCapabilities,
    /// Whether router/contract review is complete for this profile.
    pub contract_reviewed: bool,
    /// Whether terms/jurisdiction/protocol-risk review is complete for this profile.
    pub terms_and_jurisdiction_reviewed: bool,
}

impl DexRouterProfile {
    /// Create a DEX/router profile after deterministic validation.
    pub fn new(
        venue: VenueRef,
        chain: impl Into<String>,
        display_name: impl Into<String>,
        router_label: impl Into<String>,
        spender_label: impl Into<String>,
        capabilities: DexRouterCapabilities,
    ) -> Result<Self, DexConnectorError> {
        let profile = Self {
            venue,
            chain: chain.into(),
            display_name: display_name.into(),
            router_label: router_label.into(),
            spender_label: spender_label.into(),
            capabilities,
            contract_reviewed: false,
            terms_and_jurisdiction_reviewed: false,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Validate this profile without RPC, signing, or contract calls.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "DEX_ROUTER_CHAIN_REQUIRED",
            "router chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_ROUTER_DISPLAY_NAME_REQUIRED",
            "router display name",
            &self.display_name,
            &mut violations,
        );
        validate_non_empty(
            "DEX_ROUTER_LABEL_REQUIRED",
            "router label",
            &self.router_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SPENDER_LABEL_REQUIRED",
            "spender label",
            &self.spender_label,
            &mut violations,
        );

        if self.capabilities.exact_output_swaps && !self.capabilities.exact_input_swaps {
            violations.push(DexConnectorViolation::new(
                "DEX_EXACT_OUTPUT_REQUIRES_EXACT_INPUT_BASELINE",
                "exact-output capability requires exact-input modeling support in Phase 8",
            ));
        }

        if self.capabilities.gas_estimation && !self.capabilities.transaction_simulation {
            violations.push(DexConnectorViolation::new(
                "DEX_GAS_ESTIMATION_REQUIRES_SIMULATION",
                "gas-estimation support requires transaction-simulation support in Phase 8",
            ));
        }

        finish_validation(violations)
    }
}

/// Registry for framework-only DEX/Web3 chain, router, and token profiles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DexConnectorRegistry {
    chains: Vec<Web3ChainProfile>,
    routers: Vec<DexRouterProfile>,
    tokens: Vec<DexTokenProfile>,
}

impl DexConnectorRegistry {
    /// Build a registry from validated profiles.
    pub fn new(
        chains: Vec<Web3ChainProfile>,
        routers: Vec<DexRouterProfile>,
        tokens: Vec<DexTokenProfile>,
    ) -> Result<Self, DexConnectorError> {
        let registry = Self {
            chains,
            routers,
            tokens,
        };
        registry.validate()?;
        Ok(registry)
    }

    /// Return registered chain profiles.
    #[must_use]
    pub fn chains(&self) -> &[Web3ChainProfile] {
        &self.chains
    }

    /// Return registered router profiles.
    #[must_use]
    pub fn routers(&self) -> &[DexRouterProfile] {
        &self.routers
    }

    /// Return registered token profiles.
    #[must_use]
    pub fn tokens(&self) -> &[DexTokenProfile] {
        &self.tokens
    }

    /// Find a router by venue name and chain.
    #[must_use]
    pub fn find_router(&self, venue_name: &str, chain: &str) -> Option<&DexRouterProfile> {
        self.routers.iter().find(|profile| {
            profile.venue.name.eq_ignore_ascii_case(venue_name)
                && profile.chain.eq_ignore_ascii_case(chain)
        })
    }

    /// Require a router by venue name and chain.
    pub fn require_router(
        &self,
        venue_name: &str,
        chain: &str,
    ) -> Result<&DexRouterProfile, DexConnectorError> {
        self.find_router(venue_name, chain)
            .ok_or_else(|| DexConnectorError::RouterNotRegistered {
                venue: venue_name.to_owned(),
                chain: chain.to_owned(),
            })
    }

    /// Find a token by symbol and chain.
    #[must_use]
    pub fn find_token(&self, chain: &str, symbol: &str) -> Option<&DexTokenProfile> {
        self.tokens.iter().find(|profile| {
            profile.chain.eq_ignore_ascii_case(chain) && profile.symbol.eq_ignore_ascii_case(symbol)
        })
    }

    fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        let mut chains = HashSet::new();
        let mut routers = HashSet::new();
        let mut tokens = HashSet::new();

        for chain in &self.chains {
            if let Err(DexConnectorError::ValidationFailed {
                violations: profile_violations,
            }) = chain.validate()
            {
                violations.extend(profile_violations);
            }

            let normalized = chain.chain.trim().to_ascii_lowercase();
            if !normalized.is_empty() && !chains.insert(normalized.clone()) {
                violations.push(DexConnectorViolation::new_owned(
                    "DEX_DUPLICATE_CHAIN",
                    format!("duplicate DEX/Web3 chain profile: {normalized}"),
                ));
            }
        }

        for router in &self.routers {
            if let Err(DexConnectorError::ValidationFailed {
                violations: profile_violations,
            }) = router.validate()
            {
                violations.extend(profile_violations);
            }

            let normalized = format!(
                "{}|{}",
                router.chain.trim().to_ascii_lowercase(),
                router.venue.name.trim().to_ascii_lowercase()
            );
            if !normalized.starts_with('|')
                && !normalized.ends_with('|')
                && !routers.insert(normalized.clone())
            {
                violations.push(DexConnectorViolation::new_owned(
                    "DEX_DUPLICATE_ROUTER",
                    format!("duplicate DEX router profile: {normalized}"),
                ));
            }
        }

        for token in &self.tokens {
            if let Err(DexConnectorError::ValidationFailed {
                violations: profile_violations,
            }) = token.validate()
            {
                violations.extend(profile_violations);
            }

            let normalized = format!(
                "{}|{}",
                token.chain.trim().to_ascii_lowercase(),
                token.symbol.trim().to_ascii_uppercase()
            );
            if !normalized.starts_with('|')
                && !normalized.ends_with('|')
                && !tokens.insert(normalized.clone())
            {
                violations.push(DexConnectorViolation::new_owned(
                    "DEX_DUPLICATE_TOKEN",
                    format!("duplicate DEX token profile: {normalized}"),
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Swap quote mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DexSwapMode {
    /// Exact input amount with estimated output.
    ExactInput,
    /// Exact output amount with estimated maximum input.
    ExactOutput,
}

/// Route style for future quote and simulation adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DexRouteKind {
    /// Single pool/router hop.
    SinglePool,
    /// Multiple pool/router hops inside one chain.
    MultiHop,
    /// Aggregator-selected route.
    Aggregator,
}

/// Local transaction simulation status boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DexSimulationStatus {
    /// Simulation has not run.
    NotRun,
    /// Request was locally validated only; no RPC call occurred.
    LocallyValidated,
    /// Future simulation adapter reported likely success.
    WouldSucceed,
    /// Future simulation adapter reported revert/failure.
    WouldRevert,
    /// Future simulation adapter could not determine outcome.
    Indeterminate,
}

/// Local-only DEX/Web3 request-plan shape for future adapter implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DexRequestPlanKind {
    /// HTTP quote request against a reviewed router or aggregator endpoint.
    HttpQuote,
    /// JSON-RPC `eth_call`-style quote request.
    RpcQuoteCall,
    /// JSON-RPC `eth_call`-style transaction simulation request.
    RpcSimulationCall,
    /// Solana/Jupiter-style HTTP quote request.
    SolanaHttpQuote,
}

/// Typed local request plan for future DEX/router/RPC adapters.
///
/// This is plan metadata only. It validates reviewed request shapes and can
/// produce existing local quote/simulation request records, but it never opens
/// sockets, calls HTTP/RPC providers, signs, broadcasts, bridges, or handles
/// secret material.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexRequestPlan {
    /// Stable request-plan id.
    pub id: String,
    /// Future adapter request family.
    pub request_kind: DexRequestPlanKind,
    /// Reviewed protocol/router label.
    pub protocol_label: String,
    /// Target DEX/router venue.
    pub venue: VenueRef,
    /// Target chain label.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Reviewed router label.
    pub router_label: String,
    /// Reviewed spender label.
    pub spender_label: String,
    /// HTTP method for future quote endpoints.
    pub http_method: Option<String>,
    /// HTTP path for future quote endpoints.
    pub http_path: Option<String>,
    /// Query keys that a future adapter may populate.
    pub query_keys: Vec<String>,
    /// RPC method for future JSON-RPC calls.
    pub rpc_method: Option<String>,
    /// Function selector or method label for local review.
    pub call_selector: Option<String>,
    /// Non-secret payload field names expected by the future adapter.
    pub payload_fields: Vec<String>,
    /// Whether an HTTP request occurred. Always false here.
    pub http_call_performed: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_call_performed: bool,
    /// Whether credentials were loaded. Always false here.
    pub credentials_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether bridge execution occurred. Always false here.
    pub bridge_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this plan claims production readiness. Always false here.
    pub production_ready: bool,
}

/// Caller-supplied local DEX/Web3 response transcript.
///
/// The payload is parsed only from local JSON provided by tests or operators.
/// It is not fetched from HTTP/RPC, is not signed, and is never broadcastable.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexResponseTranscript {
    /// Stable transcript id.
    pub id: String,
    /// Request id this transcript belongs to.
    pub request_id: String,
    /// Future adapter request family.
    pub request_kind: DexRequestPlanKind,
    /// Reviewed protocol/router label.
    pub protocol_label: String,
    /// Venue that produced the local transcript.
    pub venue: VenueRef,
    /// Chain on which the transcript applies.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Raw local JSON transcript payload.
    pub payload_json: String,
    /// Whether an HTTP response was received live. Always false here.
    pub http_response_received: bool,
    /// Whether an RPC response was received live. Always false here.
    pub rpc_response_received: bool,
    /// Whether credentials were loaded. Always false here.
    pub credentials_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether bridge execution occurred. Always false here.
    pub bridge_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this transcript claims production readiness. Always false here.
    pub production_ready: bool,
}

/// Local-only future Web3 transaction status transcript family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3TransactionLifecycleTranscriptFormat {
    /// EVM transaction receipt/status fixture.
    EvmTransactionReceipt,
    /// Solana signature-status fixture.
    SolanaSignatureStatus,
}

/// Normalized local transaction lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3TransactionLifecycleStatus {
    /// Fixture indicates the transaction is accepted but not yet confirmed.
    Pending,
    /// Fixture indicates the transaction has local confirmations.
    Confirmed,
    /// Fixture indicates an execution failure without a successful receipt.
    Failed,
    /// Fixture indicates a reverted EVM receipt.
    Reverted,
    /// Fixture indicates the transaction is not found or dropped from the local status corpus.
    Dropped,
}

/// Caller-supplied local transaction lifecycle transcript.
///
/// The payload is parsed only from local JSON supplied by tests or operators.
/// It never calls RPC, loads credentials, signs, broadcasts, bridges, or submits
/// transactions.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3TransactionLifecycleTranscript {
    /// Stable transcript id.
    pub id: String,
    /// Local transaction request id this status belongs to.
    pub request_id: String,
    /// Local chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Transcript fixture family.
    pub format: Web3TransactionLifecycleTranscriptFormat,
    /// Raw local JSON transcript payload.
    pub payload_json: String,
    /// Whether an RPC response was received live. Always false here.
    pub rpc_response_received: bool,
    /// Whether credentials were loaded. Always false here.
    pub credentials_loaded: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether bridge execution occurred. Always false here.
    pub bridge_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this transcript claims production readiness. Always false here.
    pub production_ready: bool,
}

/// Normalized local transaction lifecycle record parsed from a local transcript.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3TransactionLifecycleRecord {
    /// DEX/Web3 framework version that produced the record.
    pub framework_version: String,
    /// Transcript id that produced this record.
    pub transcript_id: String,
    /// Local transaction request id.
    pub request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Transaction hash or signature from the local fixture.
    pub transaction_id: String,
    /// Optional nonce when present in local EVM fixtures.
    pub nonce: Option<u64>,
    /// Optional EVM block number when present.
    pub block_number: Option<u64>,
    /// Optional Solana slot when present.
    pub slot: Option<u64>,
    /// Local confirmation count.
    pub confirmations: u64,
    /// Normalized lifecycle status.
    pub status: Web3TransactionLifecycleStatus,
    /// Optional local diagnostic such as revert reason or Solana error.
    pub diagnostic: Option<String>,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_call_performed: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this record claims production readiness. Always false here.
    pub production_ready: bool,
}

/// Local DEX/Web3 protocol risk review outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DexProtocolRiskReviewStatus {
    /// Local metadata passed the deterministic review checks.
    ReadyForLocalReview,
    /// Local metadata failed one or more deterministic review checks.
    Blocked,
}

/// Local pre-sign Web3 safety review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3PreSignSafetyReviewStatus {
    /// Local simulation and nonce metadata are ready for signer authorization review.
    ReadyForLocalReview,
    /// Local simulation or nonce metadata is incomplete or unsafe.
    Blocked,
}

/// Local Web3 nonce reservation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3NonceReservationStatus {
    /// A local nonce can be reserved for a future pre-sign review.
    ReservedForLocalReview,
    /// Local nonce metadata is stale, duplicated, missing, or unsafe.
    Blocked,
}

/// Local Web3 unsigned payload review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3UnsignedPayloadReviewStatus {
    /// Local unsigned payload metadata is ready for pre-sign safety review.
    ReadyForLocalReview,
    /// Local unsigned payload metadata is incomplete or unsafe.
    Blocked,
}

/// Local Web3 broadcast-readiness review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3BroadcastReadinessStatus {
    /// Local prerequisite metadata is ready for external operator review, not broadcast.
    ReadyForExternalReview,
    /// Local prerequisite metadata is incomplete, unsafe, or side-effectful.
    Blocked,
}

/// Local Web3 unsigned transaction construction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3UnsignedTransactionConstructionStatus {
    /// Local unsigned transaction metadata is constructed for review only.
    ConstructedForLocalReview,
    /// Local unsigned transaction metadata is incomplete, unsafe, or side-effectful.
    Blocked,
}

/// Local Web3 provider nonce reconciliation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3ProviderNonceReconciliationStatus {
    /// Local provider nonce metadata reconciles with the unsigned transaction metadata.
    ReconciledForLocalReview,
    /// Local provider nonce metadata is stale, conflicting, unsafe, or side-effectful.
    Blocked,
}

/// Local Web3 raw transaction serialization review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3RawTransactionSerializationReviewStatus {
    /// Local serialization metadata is coherent and ready for external serializer review.
    ReadyForExternalReview,
    /// Local serialization metadata is incomplete, unsafe, or side-effectful.
    Blocked,
}

/// Local Web3 broadcast adapter control review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3BroadcastAdapterControlReviewStatus {
    /// Local broadcast adapter controls are ready for external adapter review, not broadcast.
    ReadyForExternalReview,
    /// Local broadcast adapter controls are incomplete, unsafe, or side-effectful.
    Blocked,
}

/// Local Web3 sandbox/live discrepancy calibration status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web3SandboxLiveDiscrepancyCalibrationStatus {
    /// Caller-supplied sandbox/live discrepancy metadata is coherent for local review.
    CalibratedForLocalReview,
    /// Caller-supplied sandbox/live discrepancy metadata is incomplete, unsafe, or side-effectful.
    Blocked,
}

/// Local-only DEX/Web3 protocol risk review request.
///
/// This is deterministic metadata review only. It does not inspect live
/// contracts, call RPC, request wallet approvals, sign, broadcast, or bridge.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexProtocolRiskReviewRequest {
    /// Stable review id.
    pub id: String,
    /// Human-reviewed protocol/router label.
    pub protocol_label: String,
    /// Venue/router metadata.
    pub venue: VenueRef,
    /// Chain label.
    pub chain: String,
    /// Whether the chain label is locally allowlisted.
    pub chain_allowlisted: bool,
    /// Router label.
    pub router_label: String,
    /// Spender label.
    pub spender_label: String,
    /// Market pair under review.
    pub pair: MarketPair,
    /// Whether the market pair is locally allowlisted.
    pub pair_allowlisted: bool,
    /// Maximum allowed slippage in basis points.
    pub max_slippage_bps: u16,
    /// Local quoted slippage in basis points.
    pub quoted_slippage_bps: f64,
    /// Maximum allowed gas/network fee in quote units.
    pub max_gas_fee_quote: f64,
    /// Local estimated gas/network fee in quote units.
    pub estimated_gas_fee_quote: f64,
    /// Local MEV/sandwich-risk estimate in basis points.
    pub mev_risk_bps: f64,
    /// Maximum allowed MEV/sandwich risk in basis points.
    pub mev_risk_limit_bps: f64,
    /// Whether the router label is locally allowlisted.
    pub router_allowlisted: bool,
    /// Whether the spender label is locally allowlisted.
    pub spender_allowlisted: bool,
    /// Whether unlimited allowance is requested. Must be false.
    pub unlimited_allowance_requested: bool,
    /// Whether an approval revocation/expiry strategy is documented.
    pub approval_revocation_planned: bool,
    /// Whether token metadata has been locally reviewed.
    pub token_metadata_reviewed: bool,
    /// Whether token contract metadata is locally reviewed.
    pub token_contract_reviewed: bool,
    /// Whether token decimals have been locally verified.
    pub token_decimals_verified: bool,
    /// Whether protocol/router terms have been locally reviewed.
    pub protocol_terms_reviewed: bool,
    /// Whether jurisdiction constraints for this protocol/path were locally reviewed.
    pub jurisdiction_reviewed: bool,
    /// Whether protocol incident/reputation history was locally reviewed.
    pub incident_reputation_reviewed: bool,
    /// Whether public mempool exposure is required by the reviewed path.
    pub public_mempool_required: bool,
    /// Whether local MEV mitigation metadata is present for public-mempool paths.
    pub mev_mitigation_reviewed: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_call_performed: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether bridge execution occurred. Always false here.
    pub bridge_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
}

/// Deterministic local protocol risk review report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexProtocolRiskReviewReport {
    /// DEX/Web3 framework version that produced the report.
    pub framework_version: String,
    /// Original review request id.
    pub request_id: String,
    /// Protocol label.
    pub protocol_label: String,
    /// Venue/router metadata.
    pub venue: VenueRef,
    /// Chain label.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Deterministic local review status.
    pub status: DexProtocolRiskReviewStatus,
    /// Non-secret local blocker codes.
    pub blocker_codes: Vec<String>,
    /// Whether chain and token-pair scope controls passed.
    pub asset_scope_passed: bool,
    /// Whether router and spender contract hygiene passed.
    pub contract_hygiene_passed: bool,
    /// Whether token metadata/contract/decimals checks passed.
    pub token_hygiene_passed: bool,
    /// Whether jurisdiction, protocol terms, and incident/reputation checks passed.
    pub governance_review_passed: bool,
    /// Whether spender hygiene passed.
    pub spender_hygiene_passed: bool,
    /// Whether gas/slippage checks passed.
    pub gas_slippage_passed: bool,
    /// Whether MEV controls passed.
    pub mev_controls_passed: bool,
    /// Whether terms and token metadata checks passed.
    pub terms_metadata_passed: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_call_performed: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether bridge execution occurred. Always false here.
    pub bridge_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this report claims production readiness. Always false here.
    pub production_ready: bool,
}

impl DexResponseTranscript {
    /// Create a local transcript with side-effect flags disabled.
    #[allow(clippy::too_many_arguments)]
    pub fn local(
        id: impl Into<String>,
        request_id: impl Into<String>,
        request_kind: DexRequestPlanKind,
        protocol_label: impl Into<String>,
        venue: VenueRef,
        chain: impl Into<String>,
        pair: MarketPair,
        payload_json: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let transcript = Self {
            id: id.into(),
            request_id: request_id.into(),
            request_kind,
            protocol_label: protocol_label.into(),
            venue,
            chain: chain.into(),
            pair,
            payload_json: payload_json.into(),
            http_response_received: false,
            rpc_response_received: false,
            credentials_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        transcript.validate()?;
        Ok(transcript)
    }

    /// Validate transcript metadata without trusting the payload.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("DEX response transcript", &self.id, &mut violations);
        validate_id(
            "DEX response transcript request",
            &self.request_id,
            &mut violations,
        );
        validate_non_empty(
            "DEX_RESPONSE_TRANSCRIPT_PROTOCOL_REQUIRED",
            "DEX response transcript protocol",
            &self.protocol_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_RESPONSE_TRANSCRIPT_CHAIN_REQUIRED",
            "DEX response transcript chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_RESPONSE_TRANSCRIPT_PAYLOAD_REQUIRED",
            "DEX response transcript payload",
            &self.payload_json,
            &mut violations,
        );
        validate_non_empty(
            "DEX_RESPONSE_TRANSCRIPT_VENUE_REQUIRED",
            "DEX response transcript venue",
            &self.venue.name,
            &mut violations,
        );
        if self.venue.kind != VenueKind::Dex {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_VENUE_KIND_INVALID",
                "DEX response transcript venue kind must be Dex",
            ));
        }
        if let Err(source) = self.pair.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "DEX_RESPONSE_TRANSCRIPT_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if self.http_response_received
            || self.rpc_response_received
            || self.credentials_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.bridge_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_SIDE_EFFECT_FLAG",
                "DEX response transcripts must remain local-only and side-effect free",
            ));
        }
        finish_validation(violations)
    }

    fn parsed_payload(&self) -> Result<Value, DexConnectorError> {
        serde_json::from_str(&self.payload_json).map_err(|error| {
            DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new_owned(
                    "DEX_RESPONSE_TRANSCRIPT_JSON_INVALID",
                    format!("DEX response transcript JSON must parse: {error}"),
                )],
            }
        })
    }
}

impl Web3TransactionLifecycleTranscript {
    /// Create a local transaction lifecycle transcript with side-effect flags disabled.
    pub fn local(
        id: impl Into<String>,
        request_id: impl Into<String>,
        chain: impl Into<String>,
        venue: VenueRef,
        format: Web3TransactionLifecycleTranscriptFormat,
        payload_json: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let transcript = Self {
            id: id.into(),
            request_id: request_id.into(),
            chain: chain.into(),
            venue,
            format,
            payload_json: payload_json.into(),
            rpc_response_received: false,
            credentials_loaded: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        transcript.validate()?;
        Ok(transcript)
    }

    /// Validate local transcript metadata without trusting payload contents.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "Web3 transaction lifecycle transcript",
            &self.id,
            &mut violations,
        );
        validate_id(
            "Web3 transaction lifecycle request",
            &self.request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_TRANSACTION_LIFECYCLE_CHAIN_REQUIRED",
            "Web3 transaction lifecycle chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_TRANSACTION_LIFECYCLE_PAYLOAD_REQUIRED",
            "Web3 transaction lifecycle payload",
            &self.payload_json,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_TRANSACTION_LIFECYCLE_VENUE_REQUIRED",
            "Web3 transaction lifecycle venue",
            &self.venue.name,
            &mut violations,
        );
        if self.venue.kind != VenueKind::Dex {
            violations.push(DexConnectorViolation::new(
                "WEB3_TRANSACTION_LIFECYCLE_VENUE_KIND_INVALID",
                "Web3 transaction lifecycle venue kind must be Dex",
            ));
        }
        if self.rpc_response_received
            || self.credentials_loaded
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.bridge_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_TRANSACTION_LIFECYCLE_SIDE_EFFECT_FLAG",
                "Web3 transaction lifecycle transcripts must remain local-only and side-effect free",
            ));
        }
        finish_validation(violations)
    }

    /// Parse the local transcript into a normalized lifecycle record.
    pub fn parse_record(&self) -> Result<Web3TransactionLifecycleRecord, DexConnectorError> {
        self.validate()?;
        let payload = self.parsed_payload()?;
        let record = match self.format {
            Web3TransactionLifecycleTranscriptFormat::EvmTransactionReceipt => {
                self.parse_evm_receipt(&payload)?
            }
            Web3TransactionLifecycleTranscriptFormat::SolanaSignatureStatus => {
                self.parse_solana_signature_status(&payload)?
            }
        };
        record.validate()?;
        Ok(record)
    }

    fn parsed_payload(&self) -> Result<Value, DexConnectorError> {
        serde_json::from_str(&self.payload_json).map_err(|error| {
            DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new_owned(
                    "WEB3_TRANSACTION_LIFECYCLE_JSON_INVALID",
                    format!("Web3 transaction lifecycle JSON must parse: {error}"),
                )],
            }
        })
    }

    fn parse_evm_receipt(
        &self,
        payload: &Value,
    ) -> Result<Web3TransactionLifecycleRecord, DexConnectorError> {
        let transaction_id = first_json_string(payload, &["transactionHash", "hash", "txHash"])?;
        let receipt_status = optional_json_string(payload, &["status"])?;
        let confirmations = first_json_u64_or_default(payload, &["confirmations"], 0)?;
        let block_number = optional_json_u64(payload, &["blockNumber"])?;
        let nonce = optional_json_u64(payload, &["nonce"])?;
        let status = match receipt_status.as_deref().map(str::to_ascii_lowercase) {
            Some(value) if matches!(value.as_str(), "0x1" | "1" | "success" | "confirmed") => {
                if confirmations > 0 || block_number.is_some() {
                    Web3TransactionLifecycleStatus::Confirmed
                } else {
                    Web3TransactionLifecycleStatus::Pending
                }
            }
            Some(value) if matches!(value.as_str(), "0x0" | "0" | "reverted") => {
                Web3TransactionLifecycleStatus::Reverted
            }
            Some(value) if value == "pending" => Web3TransactionLifecycleStatus::Pending,
            Some(value) if value == "dropped" => Web3TransactionLifecycleStatus::Dropped,
            Some(_) => Web3TransactionLifecycleStatus::Failed,
            None if confirmations > 0 || block_number.is_some() => {
                Web3TransactionLifecycleStatus::Confirmed
            }
            None => Web3TransactionLifecycleStatus::Pending,
        };
        Ok(Web3TransactionLifecycleRecord {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            transcript_id: self.id.clone(),
            request_id: self.request_id.clone(),
            chain: self.chain.clone(),
            venue: self.venue.clone(),
            transaction_id,
            nonce,
            block_number,
            slot: None,
            confirmations,
            status,
            diagnostic: optional_json_string(payload, &["revertReason", "diagnostic"])?,
            rpc_call_performed: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
        })
    }

    fn parse_solana_signature_status(
        &self,
        payload: &Value,
    ) -> Result<Web3TransactionLifecycleRecord, DexConnectorError> {
        let transaction_id = first_json_string(payload, &["signature", "transactionHash", "hash"])?;
        let confirmations = first_json_u64_or_default(payload, &["confirmations"], 0)?;
        let slot = optional_json_u64(payload, &["slot"])?;
        let confirmation_status = optional_json_string(payload, &["confirmationStatus", "status"])?;
        let diagnostic = optional_solana_error(payload)?;
        let status = if diagnostic.is_some() {
            Web3TransactionLifecycleStatus::Failed
        } else {
            match confirmation_status.as_deref().map(str::to_ascii_lowercase) {
                Some(value) if matches!(value.as_str(), "confirmed" | "finalized") => {
                    Web3TransactionLifecycleStatus::Confirmed
                }
                Some(value) if value == "dropped" => Web3TransactionLifecycleStatus::Dropped,
                Some(value) if value == "failed" => Web3TransactionLifecycleStatus::Failed,
                _ if confirmations > 0 || slot.is_some() => {
                    Web3TransactionLifecycleStatus::Confirmed
                }
                _ => Web3TransactionLifecycleStatus::Pending,
            }
        };
        Ok(Web3TransactionLifecycleRecord {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            transcript_id: self.id.clone(),
            request_id: self.request_id.clone(),
            chain: self.chain.clone(),
            venue: self.venue.clone(),
            transaction_id,
            nonce: None,
            block_number: None,
            slot,
            confirmations,
            status,
            diagnostic,
            rpc_call_performed: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
        })
    }
}

impl Web3TransactionLifecycleRecord {
    /// Validate normalized local transaction lifecycle metadata.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "Web3 transaction lifecycle transcript",
            &self.transcript_id,
            &mut violations,
        );
        validate_id(
            "Web3 transaction lifecycle request",
            &self.request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_TRANSACTION_LIFECYCLE_CHAIN_REQUIRED",
            "Web3 transaction lifecycle chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_TRANSACTION_LIFECYCLE_TRANSACTION_ID_REQUIRED",
            "Web3 transaction id",
            &self.transaction_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_TRANSACTION_LIFECYCLE_VENUE_REQUIRED",
            "Web3 transaction lifecycle venue",
            &self.venue.name,
            &mut violations,
        );
        if self.venue.kind != VenueKind::Dex {
            violations.push(DexConnectorViolation::new(
                "WEB3_TRANSACTION_LIFECYCLE_VENUE_KIND_INVALID",
                "Web3 transaction lifecycle venue kind must be Dex",
            ));
        }
        if matches!(self.status, Web3TransactionLifecycleStatus::Confirmed)
            && self.confirmations == 0
            && self.block_number.is_none()
            && self.slot.is_none()
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_TRANSACTION_LIFECYCLE_CONFIRMATION_MISSING",
                "confirmed lifecycle records must include confirmations, block number, or slot",
            ));
        }
        if self.rpc_call_performed
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_TRANSACTION_LIFECYCLE_RECORD_SIDE_EFFECT_FLAG",
                "Web3 transaction lifecycle records must remain local-only and side-effect free",
            ));
        }
        finish_validation(violations)
    }
}

impl DexProtocolRiskReviewRequest {
    /// Create a local protocol risk review request with side-effect flags disabled.
    #[allow(clippy::too_many_arguments)]
    pub fn local(
        id: impl Into<String>,
        protocol_label: impl Into<String>,
        venue: VenueRef,
        chain: impl Into<String>,
        router_label: impl Into<String>,
        spender_label: impl Into<String>,
        pair: MarketPair,
        max_slippage_bps: u16,
        quoted_slippage_bps: f64,
        max_gas_fee_quote: f64,
        estimated_gas_fee_quote: f64,
        mev_risk_bps: f64,
        mev_risk_limit_bps: f64,
    ) -> Result<Self, DexConnectorError> {
        let request = Self {
            id: id.into(),
            protocol_label: protocol_label.into(),
            venue,
            chain: chain.into(),
            chain_allowlisted: true,
            router_label: router_label.into(),
            spender_label: spender_label.into(),
            pair,
            pair_allowlisted: true,
            max_slippage_bps,
            quoted_slippage_bps,
            max_gas_fee_quote,
            estimated_gas_fee_quote,
            mev_risk_bps,
            mev_risk_limit_bps,
            router_allowlisted: true,
            spender_allowlisted: true,
            unlimited_allowance_requested: false,
            approval_revocation_planned: true,
            token_metadata_reviewed: true,
            token_contract_reviewed: true,
            token_decimals_verified: true,
            protocol_terms_reviewed: true,
            jurisdiction_reviewed: true,
            incident_reputation_reviewed: true,
            public_mempool_required: false,
            mev_mitigation_reviewed: true,
            rpc_call_performed: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        request.validate()?;
        Ok(request)
    }

    /// Validate review metadata and side-effect flags.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("DEX protocol risk review", &self.id, &mut violations);
        validate_non_empty(
            "DEX_PROTOCOL_REVIEW_PROTOCOL_REQUIRED",
            "DEX protocol risk review protocol",
            &self.protocol_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_PROTOCOL_REVIEW_CHAIN_REQUIRED",
            "DEX protocol risk review chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_PROTOCOL_REVIEW_ROUTER_REQUIRED",
            "DEX protocol risk review router",
            &self.router_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_PROTOCOL_REVIEW_SPENDER_REQUIRED",
            "DEX protocol risk review spender",
            &self.spender_label,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if let Err(source) = self.pair.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "DEX_PROTOCOL_REVIEW_PAIR_INVALID",
                source.to_string(),
            ));
        }
        for (code, label, value) in [
            (
                "DEX_PROTOCOL_REVIEW_SLIPPAGE_INVALID",
                "quoted slippage",
                self.quoted_slippage_bps,
            ),
            (
                "DEX_PROTOCOL_REVIEW_MAX_GAS_INVALID",
                "max gas fee",
                self.max_gas_fee_quote,
            ),
            (
                "DEX_PROTOCOL_REVIEW_ESTIMATED_GAS_INVALID",
                "estimated gas fee",
                self.estimated_gas_fee_quote,
            ),
            (
                "DEX_PROTOCOL_REVIEW_MEV_RISK_INVALID",
                "MEV risk",
                self.mev_risk_bps,
            ),
            (
                "DEX_PROTOCOL_REVIEW_MEV_LIMIT_INVALID",
                "MEV risk limit",
                self.mev_risk_limit_bps,
            ),
        ] {
            if !is_non_negative_finite(value) {
                violations.push(DexConnectorViolation::new_owned(
                    code,
                    format!("{label} must be non-negative and finite"),
                ));
            }
        }
        if self.rpc_call_performed
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.bridge_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "DEX_PROTOCOL_REVIEW_SIDE_EFFECT_FLAG",
                "DEX protocol risk reviews must remain local-only and side-effect free",
            ));
        }
        finish_validation(violations)
    }

    /// Evaluate the local protocol risk controls.
    pub fn review(&self) -> Result<DexProtocolRiskReviewReport, DexConnectorError> {
        self.validate()?;
        let mut blockers = Vec::new();
        let asset_scope_passed = self.chain_allowlisted && self.pair_allowlisted;
        if !self.chain_allowlisted {
            blockers.push("chain-not-allowlisted".to_owned());
        }
        if !self.pair_allowlisted {
            blockers.push("pair-not-allowlisted".to_owned());
        }
        let router_hygiene_passed = self.router_allowlisted;
        if !self.router_allowlisted {
            blockers.push("router-not-allowlisted".to_owned());
        }
        let spender_hygiene_passed = self.spender_allowlisted
            && !self.unlimited_allowance_requested
            && self.approval_revocation_planned;
        if !self.spender_allowlisted {
            blockers.push("spender-not-allowlisted".to_owned());
        }
        if self.unlimited_allowance_requested {
            blockers.push("unlimited-allowance-requested".to_owned());
        }
        if !self.approval_revocation_planned {
            blockers.push("approval-revocation-not-planned".to_owned());
        }
        let contract_hygiene_passed = router_hygiene_passed && spender_hygiene_passed;
        let token_hygiene_passed = self.token_metadata_reviewed
            && self.token_contract_reviewed
            && self.token_decimals_verified;
        if !self.token_contract_reviewed {
            blockers.push("token-contract-not-reviewed".to_owned());
        }
        if !self.token_decimals_verified {
            blockers.push("token-decimals-not-verified".to_owned());
        }

        let gas_slippage_passed = self.quoted_slippage_bps <= f64::from(self.max_slippage_bps)
            && self.estimated_gas_fee_quote <= self.max_gas_fee_quote;
        if self.quoted_slippage_bps > f64::from(self.max_slippage_bps) {
            blockers.push("slippage-limit-exceeded".to_owned());
        }
        if self.estimated_gas_fee_quote > self.max_gas_fee_quote {
            blockers.push("gas-fee-limit-exceeded".to_owned());
        }

        let mev_controls_passed = self.mev_risk_bps <= self.mev_risk_limit_bps
            && (!self.public_mempool_required || self.mev_mitigation_reviewed);
        if self.mev_risk_bps > self.mev_risk_limit_bps {
            blockers.push("mev-risk-limit-exceeded".to_owned());
        }
        if self.public_mempool_required && !self.mev_mitigation_reviewed {
            blockers.push("public-mempool-mev-mitigation-missing".to_owned());
        }

        let governance_review_passed = self.protocol_terms_reviewed
            && self.jurisdiction_reviewed
            && self.incident_reputation_reviewed;
        let terms_metadata_passed = self.token_metadata_reviewed && self.protocol_terms_reviewed;
        if !self.token_metadata_reviewed {
            blockers.push("token-metadata-not-reviewed".to_owned());
        }
        if !self.protocol_terms_reviewed {
            blockers.push("protocol-terms-not-reviewed".to_owned());
        }
        if !self.jurisdiction_reviewed {
            blockers.push("jurisdiction-not-reviewed".to_owned());
        }
        if !self.incident_reputation_reviewed {
            blockers.push("incident-reputation-not-reviewed".to_owned());
        }

        let status = if blockers.is_empty() {
            DexProtocolRiskReviewStatus::ReadyForLocalReview
        } else {
            DexProtocolRiskReviewStatus::Blocked
        };
        let report = DexProtocolRiskReviewReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            request_id: self.id.clone(),
            protocol_label: self.protocol_label.clone(),
            venue: self.venue.clone(),
            chain: self.chain.clone(),
            pair: self.pair.clone(),
            status,
            blocker_codes: blockers,
            asset_scope_passed,
            contract_hygiene_passed,
            token_hygiene_passed,
            governance_review_passed,
            spender_hygiene_passed,
            gas_slippage_passed,
            mev_controls_passed,
            terms_metadata_passed,
            rpc_call_performed: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        report.validate()?;
        Ok(report)
    }
}

impl DexProtocolRiskReviewReport {
    /// Validate deterministic local review output.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id(
            "DEX protocol risk review report",
            &self.request_id,
            &mut violations,
        );
        validate_non_empty(
            "DEX_PROTOCOL_REVIEW_PROTOCOL_REQUIRED",
            "DEX protocol risk review protocol",
            &self.protocol_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_PROTOCOL_REVIEW_CHAIN_REQUIRED",
            "DEX protocol risk review chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if matches!(
            self.status,
            DexProtocolRiskReviewStatus::ReadyForLocalReview
        ) && !self.blocker_codes.is_empty()
        {
            violations.push(DexConnectorViolation::new(
                "DEX_PROTOCOL_REVIEW_READY_WITH_BLOCKERS",
                "ready protocol risk reports must not contain blocker codes",
            ));
        }
        if matches!(self.status, DexProtocolRiskReviewStatus::Blocked)
            && self.blocker_codes.is_empty()
        {
            violations.push(DexConnectorViolation::new(
                "DEX_PROTOCOL_REVIEW_BLOCKED_WITHOUT_BLOCKERS",
                "blocked protocol risk reports must contain at least one blocker code",
            ));
        }
        if self.rpc_call_performed
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.bridge_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "DEX_PROTOCOL_REVIEW_REPORT_SIDE_EFFECT_FLAG",
                "DEX protocol risk review reports must remain local-only and side-effect free",
            ));
        }
        finish_validation(violations)
    }
}

impl DexRequestPlan {
    /// Create a local Uniswap V3 quoter `eth_call` request plan.
    pub fn uniswap_v3_quoter_eth_call(
        venue: VenueRef,
        pair: MarketPair,
        chain: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let plan = Self {
            id: "dex-plan-uniswap-v3-quoter-eth-call".to_owned(),
            request_kind: DexRequestPlanKind::RpcQuoteCall,
            protocol_label: "uniswap-v3-quoter".to_owned(),
            venue,
            chain: chain.into(),
            pair,
            router_label: "uniswap-v3-quoter-reviewed".to_owned(),
            spender_label: "uniswap-v3-router-reviewed".to_owned(),
            http_method: None,
            http_path: None,
            query_keys: Vec::new(),
            rpc_method: Some("eth_call".to_owned()),
            call_selector: Some("quoteExactInputSingle".to_owned()),
            payload_fields: vec![
                "tokenIn".to_owned(),
                "tokenOut".to_owned(),
                "fee".to_owned(),
                "amountIn".to_owned(),
                "sqrtPriceLimitX96".to_owned(),
            ],
            http_call_performed: false,
            rpc_call_performed: false,
            credentials_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Create a local 0x swap quote HTTP request plan.
    pub fn zero_ex_swap_quote_http(
        venue: VenueRef,
        pair: MarketPair,
        chain: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let plan = Self {
            id: "dex-plan-0x-swap-quote-http".to_owned(),
            request_kind: DexRequestPlanKind::HttpQuote,
            protocol_label: "zero-ex-swap-api".to_owned(),
            venue,
            chain: chain.into(),
            pair,
            router_label: "zero-ex-exchange-proxy-reviewed".to_owned(),
            spender_label: "zero-ex-allowance-target-reviewed".to_owned(),
            http_method: Some("GET".to_owned()),
            http_path: Some("/swap/v1/quote".to_owned()),
            query_keys: vec![
                "sellToken".to_owned(),
                "buyToken".to_owned(),
                "sellAmount".to_owned(),
                "slippagePercentage".to_owned(),
            ],
            rpc_method: None,
            call_selector: None,
            payload_fields: Vec::new(),
            http_call_performed: false,
            rpc_call_performed: false,
            credentials_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Create a local Jupiter quote HTTP request plan.
    pub fn jupiter_quote_http(
        venue: VenueRef,
        pair: MarketPair,
        chain: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let plan = Self {
            id: "dex-plan-jupiter-quote-http".to_owned(),
            request_kind: DexRequestPlanKind::SolanaHttpQuote,
            protocol_label: "jupiter-quote-api".to_owned(),
            venue,
            chain: chain.into(),
            pair,
            router_label: "jupiter-v6-router-reviewed".to_owned(),
            spender_label: "not-applicable-read-only-quote".to_owned(),
            http_method: Some("GET".to_owned()),
            http_path: Some("/v6/quote".to_owned()),
            query_keys: vec![
                "inputMint".to_owned(),
                "outputMint".to_owned(),
                "amount".to_owned(),
                "slippageBps".to_owned(),
            ],
            rpc_method: None,
            call_selector: None,
            payload_fields: Vec::new(),
            http_call_performed: false,
            rpc_call_performed: false,
            credentials_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Create a local EVM transaction simulation `eth_call` request plan.
    pub fn evm_transaction_simulation_eth_call(
        venue: VenueRef,
        pair: MarketPair,
        chain: impl Into<String>,
    ) -> Result<Self, DexConnectorError> {
        let plan = Self {
            id: "dex-plan-evm-simulation-eth-call".to_owned(),
            request_kind: DexRequestPlanKind::RpcSimulationCall,
            protocol_label: "evm-eth-call-simulation".to_owned(),
            venue,
            chain: chain.into(),
            pair,
            router_label: "reviewed-router-call-target".to_owned(),
            spender_label: "reviewed-spender-label".to_owned(),
            http_method: None,
            http_path: None,
            query_keys: Vec::new(),
            rpc_method: Some("eth_call".to_owned()),
            call_selector: Some("reviewed-router-swap-calldata-hash-only".to_owned()),
            payload_fields: vec![
                "from".to_owned(),
                "to".to_owned(),
                "data_hash".to_owned(),
                "value".to_owned(),
                "blockTag".to_owned(),
            ],
            http_call_performed: false,
            rpc_call_performed: false,
            credentials_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            bridge_performed: false,
            live_execution_performed: false,
            production_ready: false,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Validate this request plan without performing the planned request.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("DEX request plan", &self.id, &mut violations);
        validate_non_empty(
            "DEX_REQUEST_PLAN_PROTOCOL_REQUIRED",
            "DEX request plan protocol",
            &self.protocol_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_REQUEST_PLAN_CHAIN_REQUIRED",
            "DEX request plan chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_REQUEST_PLAN_ROUTER_REQUIRED",
            "DEX request plan router label",
            &self.router_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_REQUEST_PLAN_SPENDER_REQUIRED",
            "DEX request plan spender label",
            &self.spender_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_REQUEST_PLAN_VENUE_REQUIRED",
            "DEX request plan venue",
            &self.venue.name,
            &mut violations,
        );
        if self.venue.kind != VenueKind::Dex {
            violations.push(DexConnectorViolation::new(
                "DEX_REQUEST_PLAN_VENUE_KIND_INVALID",
                "DEX request plan venue kind must be Dex",
            ));
        }
        if let Err(DexConnectorError::ValidationFailed {
            violations: pair_violations,
        }) = self.validate_pair()
        {
            violations.extend(pair_violations);
        }

        match self.request_kind {
            DexRequestPlanKind::HttpQuote | DexRequestPlanKind::SolanaHttpQuote => {
                validate_expected_value(
                    "DEX_REQUEST_PLAN_HTTP_METHOD_INVALID",
                    "HTTP method",
                    self.http_method.as_deref(),
                    "GET",
                    &mut violations,
                );
                validate_http_path(self.http_path.as_deref(), &mut violations);
                validate_non_empty_fields(
                    "DEX_REQUEST_PLAN_QUERY_KEYS_REQUIRED",
                    "query key",
                    &self.query_keys,
                    &mut violations,
                );
                if self.rpc_method.is_some() || self.call_selector.is_some() {
                    violations.push(DexConnectorViolation::new(
                        "DEX_REQUEST_PLAN_RPC_FIELDS_FOR_HTTP",
                        "HTTP quote plans must not include RPC method or call selector",
                    ));
                }
            }
            DexRequestPlanKind::RpcQuoteCall | DexRequestPlanKind::RpcSimulationCall => {
                validate_expected_value(
                    "DEX_REQUEST_PLAN_RPC_METHOD_INVALID",
                    "RPC method",
                    self.rpc_method.as_deref(),
                    "eth_call",
                    &mut violations,
                );
                validate_non_empty(
                    "DEX_REQUEST_PLAN_CALL_SELECTOR_REQUIRED",
                    "call selector",
                    self.call_selector.as_deref().unwrap_or_default(),
                    &mut violations,
                );
                validate_non_empty_fields(
                    "DEX_REQUEST_PLAN_PAYLOAD_FIELDS_REQUIRED",
                    "payload field",
                    &self.payload_fields,
                    &mut violations,
                );
                if self.http_method.is_some()
                    || self.http_path.is_some()
                    || !self.query_keys.is_empty()
                {
                    violations.push(DexConnectorViolation::new(
                        "DEX_REQUEST_PLAN_HTTP_FIELDS_FOR_RPC",
                        "RPC plans must not include HTTP method, path, or query keys",
                    ));
                }
            }
        }

        if self.http_call_performed
            || self.rpc_call_performed
            || self.credentials_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.bridge_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "DEX_REQUEST_PLAN_SIDE_EFFECT_FLAG",
                "DEX request plans must remain local-only and side-effect free",
            ));
        }

        finish_validation(violations)
    }

    fn validate_pair(&self) -> Result<(), DexConnectorError> {
        MarketPair::new(self.pair.base.clone(), self.pair.quote.clone())
            .map(|_| ())
            .map_err(|error| DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new_owned(
                    "DEX_REQUEST_PLAN_PAIR_INVALID",
                    error.to_string(),
                )],
            })
    }

    /// Convert a quote-capable plan into an existing local swap quote request.
    pub fn to_local_quote_request(
        &self,
        id: impl Into<String>,
        strategy_id: impl Into<String>,
        amount_in: f64,
        expected_amount_out: f64,
        notional_quote: f64,
    ) -> Result<DexSwapQuoteRequest, DexConnectorError> {
        self.validate()?;
        if self.request_kind == DexRequestPlanKind::RpcSimulationCall {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "DEX_REQUEST_PLAN_NOT_QUOTE_CAPABLE",
                    "simulation-only request plans cannot create quote requests",
                )],
            });
        }
        let request = DexSwapQuoteRequest {
            id: id.into(),
            strategy_id: strategy_id.into(),
            scope: ExecutionScope::Paper,
            venue: self.venue.clone(),
            chain: self.chain.clone(),
            pair: self.pair.clone(),
            mode: DexSwapMode::ExactInput,
            route_kind: match self.request_kind {
                DexRequestPlanKind::HttpQuote | DexRequestPlanKind::SolanaHttpQuote => {
                    DexRouteKind::Aggregator
                }
                DexRequestPlanKind::RpcQuoteCall | DexRequestPlanKind::RpcSimulationCall => {
                    DexRouteKind::SinglePool
                }
            },
            input_token_symbol: self.pair.base.clone(),
            output_token_symbol: self.pair.quote.clone(),
            amount_in,
            expected_amount_out,
            notional_quote,
            expected_profit_quote: 1.0,
            max_loss_quote: 1.0,
            slippage_bps: 30,
            estimated_fee_quote: 0.10,
            gas_fee_quote: 0.25,
            market_data_age_ms: 1_000,
        };
        request.validate()?;
        Ok(request)
    }

    /// Convert a simulation-capable plan into an existing local simulation request.
    pub fn to_local_simulation_request(
        &self,
        id: impl Into<String>,
        swap_request_id: impl Into<String>,
        amount_in: f64,
        minimum_amount_out: f64,
    ) -> Result<Web3TransactionSimulationRequest, DexConnectorError> {
        self.validate()?;
        if self.request_kind != DexRequestPlanKind::RpcSimulationCall {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "DEX_REQUEST_PLAN_NOT_SIMULATION_CAPABLE",
                    "quote request plans cannot create simulation requests",
                )],
            });
        }
        let request = Web3TransactionSimulationRequest {
            id: id.into(),
            swap_request_id: swap_request_id.into(),
            scope: ExecutionScope::Paper,
            venue: self.venue.clone(),
            chain: self.chain.clone(),
            router_label: self.router_label.clone(),
            spender_label: self.spender_label.clone(),
            account_label: "local-paper-account".to_owned(),
            input_token_symbol: self.pair.base.clone(),
            output_token_symbol: self.pair.quote.clone(),
            amount_in,
            minimum_amount_out,
            gas_limit: 150_000,
            max_gas_fee_quote: 0.25,
            payload_hash: "reviewed-local-payload-hash-only".to_owned(),
        };
        request.validate()?;
        Ok(request)
    }

    /// Parse a matching local quote transcript into a quote response.
    pub fn parse_quote_transcript(
        &self,
        transcript: &DexResponseTranscript,
    ) -> Result<DexSwapQuoteResponse, DexConnectorError> {
        self.validate()?;
        self.validate_transcript_match(transcript)?;
        if self.request_kind == DexRequestPlanKind::RpcSimulationCall {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "DEX_RESPONSE_TRANSCRIPT_NOT_QUOTE",
                    "simulation transcripts cannot be parsed as quote responses",
                )],
            });
        }
        let payload = transcript.parsed_payload()?;
        let amount_in = first_json_number(
            &payload,
            &["sellAmount", "inAmount", "amountIn", "inputAmount"],
        )?;
        let amount_out = first_json_number(
            &payload,
            &["buyAmount", "outAmount", "amountOut", "outputAmount"],
        )?;
        let price_impact_bps =
            first_json_number_or_default(&payload, &["priceImpactBps", "priceImpact"], 0.0)?;
        let estimated_fee_quote =
            first_json_number_or_default(&payload, &["estimatedFeeQuote", "feeQuote"], 0.0)?;
        let gas_fee_quote =
            first_json_number_or_default(&payload, &["gasFeeQuote", "estimatedGasQuote"], 0.0)?;
        let market_data_age_ms =
            first_json_u64_or_default(&payload, &["marketDataAgeMs", "contextSlotAgeMs"], 1_000)?;
        let response = DexSwapQuoteResponse {
            id: transcript.id.clone(),
            request_id: transcript.request_id.clone(),
            venue: transcript.venue.clone(),
            chain: transcript.chain.clone(),
            pair: transcript.pair.clone(),
            route_kind: match self.request_kind {
                DexRequestPlanKind::HttpQuote | DexRequestPlanKind::SolanaHttpQuote => {
                    DexRouteKind::Aggregator
                }
                DexRequestPlanKind::RpcQuoteCall | DexRequestPlanKind::RpcSimulationCall => {
                    DexRouteKind::SinglePool
                }
            },
            amount_in,
            amount_out,
            price_impact_bps,
            estimated_fee_quote,
            gas_fee_quote,
            market_data_age_ms,
            simulation_status: DexSimulationStatus::LocallyValidated,
        };
        response.validate()?;
        Ok(response)
    }

    /// Parse a matching local simulation transcript into a simulation response.
    pub fn parse_simulation_transcript(
        &self,
        transcript: &DexResponseTranscript,
    ) -> Result<Web3TransactionSimulationResponse, DexConnectorError> {
        self.validate()?;
        self.validate_transcript_match(transcript)?;
        if self.request_kind != DexRequestPlanKind::RpcSimulationCall {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "DEX_RESPONSE_TRANSCRIPT_NOT_SIMULATION",
                    "quote transcripts cannot be parsed as simulation responses",
                )],
            });
        }
        let payload = transcript.parsed_payload()?;
        let status = match first_json_string(&payload, &["status", "simulationStatus"])?
            .to_ascii_lowercase()
            .as_str()
        {
            "success" | "would-succeed" | "would_succeed" => DexSimulationStatus::WouldSucceed,
            "revert" | "would-revert" | "would_revert" => DexSimulationStatus::WouldRevert,
            "indeterminate" => DexSimulationStatus::Indeterminate,
            _ => DexSimulationStatus::LocallyValidated,
        };
        let response = Web3TransactionSimulationResponse {
            id: transcript.id.clone(),
            request_id: transcript.request_id.clone(),
            status,
            gas_used: first_json_u64(&payload, &["gasUsed", "gas_used"])?,
            gas_fee_quote: first_json_number_or_default(
                &payload,
                &["gasFeeQuote", "estimatedGasQuote"],
                0.0,
            )?,
            amount_out: first_json_number_or_default(
                &payload,
                &["amountOut", "outputAmount"],
                0.0,
            )?,
            diagnostic: optional_json_string(&payload, &["diagnostic", "revertReason"])?,
            broadcastable: false,
        };
        response.validate()?;
        Ok(response)
    }

    fn validate_transcript_match(
        &self,
        transcript: &DexResponseTranscript,
    ) -> Result<(), DexConnectorError> {
        transcript.validate()?;
        let mut violations = Vec::new();
        if self.request_kind != transcript.request_kind {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_KIND_MISMATCH",
                "DEX response transcript kind must match request plan kind",
            ));
        }
        if !self
            .protocol_label
            .eq_ignore_ascii_case(&transcript.protocol_label)
        {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_PROTOCOL_MISMATCH",
                "DEX response transcript protocol must match request plan protocol",
            ));
        }
        if !same_venue(&self.venue, &transcript.venue) {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_VENUE_MISMATCH",
                "DEX response transcript venue must match request plan venue",
            ));
        }
        if !self.chain.eq_ignore_ascii_case(&transcript.chain) {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_CHAIN_MISMATCH",
                "DEX response transcript chain must match request plan chain",
            ));
        }
        if self.pair != transcript.pair {
            violations.push(DexConnectorViolation::new(
                "DEX_RESPONSE_TRANSCRIPT_PAIR_MISMATCH",
                "DEX response transcript pair must match request plan pair",
            ));
        }
        finish_validation(violations)
    }
}

/// Framework-only DEX swap quote request.
///
/// This request does not build, sign, or broadcast a transaction. Phase 8 policy
/// gating approves only paper/simulation-scoped intent validation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexSwapQuoteRequest {
    /// Stable request id.
    pub id: String,
    /// Strategy profile id that produced the request.
    pub strategy_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target DEX/router venue.
    pub venue: VenueRef,
    /// Target chain.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Swap quote mode.
    pub mode: DexSwapMode,
    /// Route style.
    pub route_kind: DexRouteKind,
    /// Token being spent by symbol.
    pub input_token_symbol: String,
    /// Token being received by symbol.
    pub output_token_symbol: String,
    /// Input amount in token units for exact-input mode, or estimated input for exact-output mode.
    pub amount_in: f64,
    /// Expected output amount in token units.
    pub expected_amount_out: f64,
    /// Proposed notional in quote units.
    pub notional_quote: f64,
    /// Expected gross profit in quote units before fees.
    pub expected_profit_quote: f64,
    /// Worst accepted loss in quote units.
    pub max_loss_quote: f64,
    /// Requested slippage ceiling.
    pub slippage_bps: u16,
    /// Estimated venue/protocol fee in quote units.
    pub estimated_fee_quote: f64,
    /// Estimated gas/network fee in quote units.
    pub gas_fee_quote: f64,
    /// Age of source market data in milliseconds.
    pub market_data_age_ms: u64,
}

/// Durable local DEX/Web3 swap-validation record.
///
/// This records policy-gated local quote validation only. It never calls RPC,
/// builds raw calldata, signs transactions, broadcasts transactions, or claims
/// sandbox/live execution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexSwapValidationRecord {
    /// DEX/Web3 framework version that produced the record.
    pub framework_version: String,
    /// Original swap quote request id.
    pub request_id: String,
    /// Strategy profile id that produced the request.
    pub strategy_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target venue.
    pub venue: VenueRef,
    /// Target chain label.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Input amount in token units.
    pub amount_in: f64,
    /// Expected output amount in token units.
    pub expected_amount_out: f64,
    /// Simulation status at the local framework boundary.
    pub simulation_status: DexSimulationStatus,
    /// Whether policy approved the converted intent.
    pub policy_approved: bool,
    /// Trust-contract version for the approval.
    pub trust_contract_version: String,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_call_performed: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Non-secret unresolved blockers that keep this local-only.
    pub unresolved_blockers: Vec<String>,
}

impl DexSwapValidationRecord {
    /// Build a durable local record from an approved framework request.
    pub fn from_approved_request(
        request: &DexSwapQuoteRequest,
        approval: &PolicyApproval,
    ) -> Result<Self, DexConnectorError> {
        request.validate()?;
        if approval.intent_id != request.id {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "DEX_APPROVAL_INTENT_MISMATCH",
                    "policy approval intent id must match DEX request id",
                )],
            });
        }
        if approval.approved_scope != request.scope {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "DEX_APPROVAL_SCOPE_MISMATCH",
                    "policy approval scope must match DEX request scope",
                )],
            });
        }

        let record = Self {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            request_id: request.id.clone(),
            strategy_id: request.strategy_id.clone(),
            scope: request.scope,
            venue: request.venue.clone(),
            chain: request.chain.clone(),
            pair: request.pair.clone(),
            amount_in: request.amount_in,
            expected_amount_out: request.expected_amount_out,
            simulation_status: DexSimulationStatus::LocallyValidated,
            policy_approved: true,
            trust_contract_version: approval.trust_contract_version.to_owned(),
            rpc_call_performed: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            unresolved_blockers: vec![
                "live RPC adapter validation missing".to_owned(),
                "signer/custody validation missing".to_owned(),
                "confirmation tracking and production restart recovery missing".to_owned(),
            ],
        };
        record.validate()?;
        Ok(record)
    }

    /// Validate record invariants before audit/state persistence.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "DEX_FRAMEWORK_VERSION_MISMATCH",
                "DEX validation record has an unexpected framework version",
            ));
        }
        validate_id("swap validation record", &self.request_id, &mut violations);
        validate_id("strategy", &self.strategy_id, &mut violations);
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "DEX_RECORD_CHAIN_REQUIRED",
            "validation record chain",
            &self.chain,
            &mut violations,
        );
        if let Err(source) = self.pair.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "DEX_RECORD_PAIR_INVALID",
                source.to_string(),
            ));
        }
        if !is_positive_finite(self.amount_in) {
            violations.push(DexConnectorViolation::new(
                "DEX_RECORD_AMOUNT_IN_INVALID",
                "DEX validation record amount in must be positive and finite",
            ));
        }
        if !is_positive_finite(self.expected_amount_out) {
            violations.push(DexConnectorViolation::new(
                "DEX_RECORD_EXPECTED_OUTPUT_INVALID",
                "DEX validation record expected output must be positive and finite",
            ));
        }
        if self.simulation_status != DexSimulationStatus::LocallyValidated {
            violations.push(DexConnectorViolation::new(
                "DEX_RECORD_STATUS_NOT_LOCAL",
                "DEX validation record must remain locally validated",
            ));
        }
        if !self.policy_approved || self.trust_contract_version.trim().is_empty() {
            violations.push(DexConnectorViolation::new(
                "DEX_POLICY_APPROVAL_REQUIRED",
                "DEX validation record requires a local policy approval",
            ));
        }
        if self.rpc_call_performed
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
        {
            violations.push(DexConnectorViolation::new(
                "DEX_RECORD_EXTERNAL_SIDE_EFFECT",
                "DEX validation record must not include RPC, signing, broadcast, or live execution",
            ));
        }
        finish_validation(violations)
    }
}

/// Local/mock DEX/Web3 lifecycle reconciliation summary.
///
/// This ties a policy-approved local swap validation record to deterministic
/// local quote and simulation responses. It never calls RPC, loads signer
/// material, signs payloads, broadcasts transactions, bridges assets, or claims
/// production readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexSwapLifecycleRecord {
    /// DEX/Web3 framework version that produced the record.
    pub framework_version: String,
    /// Original swap quote request id.
    pub request_id: String,
    /// Strategy profile id that produced the request.
    pub strategy_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target venue.
    pub venue: VenueRef,
    /// Target chain label.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Quote response id included in the local lifecycle.
    pub quote_response_id: String,
    /// Simulation response id included in the local lifecycle.
    pub simulation_response_id: String,
    /// Route style.
    pub route_kind: DexRouteKind,
    /// Validated input amount.
    pub amount_in: f64,
    /// Quote output amount.
    pub quoted_amount_out: f64,
    /// Local simulation output amount.
    pub simulated_amount_out: f64,
    /// Simulated output shortfall versus quote in basis points.
    pub output_shortfall_bps: f64,
    /// Local simulation gas used.
    pub gas_used: u64,
    /// Local simulation gas fee in quote units.
    pub gas_fee_quote: f64,
    /// Final local simulation status.
    pub simulation_status: DexSimulationStatus,
    /// Whether quote replay was locally reconciled.
    pub quote_replayed: bool,
    /// Whether simulation replay was locally reconciled.
    pub simulation_replayed: bool,
    /// Whether duplicate intent ids were rejected separately.
    pub duplicate_intent_id_rejected: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_call_performed: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this record proves production readiness. Always false here.
    pub production_ready: bool,
    /// Non-secret unresolved blockers that keep this local-only.
    pub unresolved_blockers: Vec<String>,
}

impl DexSwapLifecycleRecord {
    /// Reconcile local quote and simulation responses into a lifecycle summary.
    pub fn from_local_quote_and_simulation(
        validation: &DexSwapValidationRecord,
        quote: &DexSwapQuoteResponse,
        simulation: &Web3TransactionSimulationResponse,
        duplicate_intent_id_rejected: bool,
    ) -> Result<Self, DexConnectorError> {
        validation.validate()?;
        validate_dex_lifecycle_inputs(validation, quote, simulation)?;

        let output_shortfall_bps = if quote.amount_out > simulation.amount_out {
            ((quote.amount_out - simulation.amount_out) / quote.amount_out) * 10_000.0
        } else {
            0.0
        };
        let record = Self {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            request_id: validation.request_id.clone(),
            strategy_id: validation.strategy_id.clone(),
            scope: validation.scope,
            venue: validation.venue.clone(),
            chain: validation.chain.clone(),
            pair: validation.pair.clone(),
            quote_response_id: quote.id.clone(),
            simulation_response_id: simulation.id.clone(),
            route_kind: quote.route_kind,
            amount_in: validation.amount_in,
            quoted_amount_out: quote.amount_out,
            simulated_amount_out: simulation.amount_out,
            output_shortfall_bps,
            gas_used: simulation.gas_used,
            gas_fee_quote: simulation.gas_fee_quote,
            simulation_status: simulation.status,
            quote_replayed: true,
            simulation_replayed: true,
            duplicate_intent_id_rejected,
            rpc_call_performed: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            unresolved_blockers: vec![
                "live RPC adapter validation missing".to_owned(),
                "signer/custody validation missing".to_owned(),
                "testnet confirmation tracking and production restart recovery missing".to_owned(),
            ],
        };
        record.validate()?;
        Ok(record)
    }

    /// Validate lifecycle reconciliation invariants before persistence.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "DEX_FRAMEWORK_VERSION_MISMATCH",
                "DEX lifecycle record has an unexpected framework version",
            ));
        }
        validate_id("DEX lifecycle request", &self.request_id, &mut violations);
        validate_id("DEX lifecycle strategy", &self.strategy_id, &mut violations);
        validate_id(
            "DEX lifecycle quote response",
            &self.quote_response_id,
            &mut violations,
        );
        validate_id(
            "DEX lifecycle simulation response",
            &self.simulation_response_id,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "DEX_LIFECYCLE_CHAIN_REQUIRED",
            "lifecycle chain",
            &self.chain,
            &mut violations,
        );
        if let Err(source) = self.pair.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "DEX_LIFECYCLE_PAIR_INVALID",
                source.to_string(),
            ));
        }
        for (code, label, value) in [
            (
                "DEX_LIFECYCLE_AMOUNT_IN_INVALID",
                "amount in",
                self.amount_in,
            ),
            (
                "DEX_LIFECYCLE_QUOTED_OUTPUT_INVALID",
                "quoted output",
                self.quoted_amount_out,
            ),
            (
                "DEX_LIFECYCLE_SIMULATED_OUTPUT_INVALID",
                "simulated output",
                self.simulated_amount_out,
            ),
        ] {
            if !is_positive_finite(value) {
                violations.push(DexConnectorViolation::new_owned(
                    code,
                    format!("DEX lifecycle {label} must be positive and finite"),
                ));
            }
        }
        if !is_non_negative_finite(self.output_shortfall_bps) {
            violations.push(DexConnectorViolation::new(
                "DEX_LIFECYCLE_SHORTFALL_INVALID",
                "DEX lifecycle output shortfall must be non-negative and finite",
            ));
        }
        if self.gas_used == 0 {
            violations.push(DexConnectorViolation::new(
                "DEX_LIFECYCLE_GAS_USED_REQUIRED",
                "DEX lifecycle gas used must be non-zero",
            ));
        }
        if !is_non_negative_finite(self.gas_fee_quote) {
            violations.push(DexConnectorViolation::new(
                "DEX_LIFECYCLE_GAS_FEE_INVALID",
                "DEX lifecycle gas fee must be non-negative and finite",
            ));
        }
        if self.simulation_status != DexSimulationStatus::LocallyValidated {
            violations.push(DexConnectorViolation::new(
                "DEX_LIFECYCLE_STATUS_NOT_LOCAL",
                "DEX lifecycle record must remain locally validated",
            ));
        }
        if !self.quote_replayed || !self.simulation_replayed {
            violations.push(DexConnectorViolation::new(
                "DEX_LIFECYCLE_REPLAY_REQUIRED",
                "DEX lifecycle record requires local quote and simulation replay",
            ));
        }
        if self.rpc_call_performed
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "DEX_LIFECYCLE_EXTERNAL_SIDE_EFFECT",
                "DEX lifecycle record must not include RPC, signing, broadcast, live execution, or production readiness",
            ));
        }
        finish_validation(violations)
    }
}

impl DexSwapQuoteRequest {
    /// Validate the request shape and framework-only DEX constraints.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("swap quote request", &self.id, &mut violations);
        validate_id("strategy", &self.strategy_id, &mut violations);
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "DEX_SWAP_CHAIN_REQUIRED",
            "swap chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_INPUT_TOKEN_REQUIRED",
            "input token symbol",
            &self.input_token_symbol,
            &mut violations,
        );
        validate_non_empty(
            "DEX_OUTPUT_TOKEN_REQUIRED",
            "output token symbol",
            &self.output_token_symbol,
            &mut violations,
        );

        if self
            .input_token_symbol
            .eq_ignore_ascii_case(&self.output_token_symbol)
        {
            violations.push(DexConnectorViolation::new(
                "DEX_INPUT_EQUALS_OUTPUT_TOKEN",
                "input and output token symbols must differ",
            ));
        }

        if let Err(source) = self.pair.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "DEX_PAIR_INVALID",
                source.to_string(),
            ));
        }

        for (code, label, value) in [
            ("DEX_AMOUNT_IN_INVALID", "amount in", self.amount_in),
            (
                "DEX_EXPECTED_AMOUNT_OUT_INVALID",
                "expected amount out",
                self.expected_amount_out,
            ),
            ("DEX_NOTIONAL_INVALID", "notional", self.notional_quote),
            (
                "DEX_EXPECTED_PROFIT_INVALID",
                "expected profit",
                self.expected_profit_quote,
            ),
        ] {
            if !is_positive_finite(value) {
                violations.push(DexConnectorViolation::new_owned(
                    code,
                    format!("DEX swap {label} must be positive and finite"),
                ));
            }
        }

        if !is_non_negative_finite(self.max_loss_quote) {
            violations.push(DexConnectorViolation::new(
                "DEX_MAX_LOSS_INVALID",
                "DEX swap max loss must be non-negative and finite",
            ));
        }

        if !is_non_negative_finite(self.estimated_fee_quote) {
            violations.push(DexConnectorViolation::new(
                "DEX_ESTIMATED_FEE_INVALID",
                "DEX estimated fee must be non-negative and finite",
            ));
        }

        if !is_non_negative_finite(self.gas_fee_quote) {
            violations.push(DexConnectorViolation::new(
                "DEX_GAS_FEE_INVALID",
                "DEX gas/network fee estimate must be non-negative and finite",
            ));
        }

        finish_validation(violations)
    }

    /// Convert the quote request into a policy intent. This does not execute.
    pub fn to_execution_intent(&self) -> Result<ExecutionIntent, DexConnectorError> {
        self.validate()?;
        Ok(ExecutionIntent {
            id: self.id.clone(),
            strategy_id: self.strategy_id.clone(),
            kind: ExecutionIntentKind::DexSwap,
            scope: self.scope,
            venue: self.venue.clone(),
            chain: Some(self.chain.clone()),
            base_asset: self.pair.base.clone(),
            quote_asset: self.pair.quote.clone(),
            notional_quote: self.notional_quote,
            expected_profit_quote: self.expected_profit_quote,
            max_loss_quote: self.max_loss_quote,
            slippage_bps: self.slippage_bps,
            estimated_fee_quote: self.estimated_fee_quote,
            gas_fee_quote: self.gas_fee_quote,
            market_data_age_ms: self.market_data_age_ms,
            destination: DestinationPolicy::ApprovedAddress {
                chain: self.chain.clone(),
                label: format!("router:{}", self.venue.name),
            },
            requires_signing: false,
        })
    }
}

/// Framework-only swap quote response.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DexSwapQuoteResponse {
    /// Stable quote response id.
    pub id: String,
    /// Request id this response belongs to.
    pub request_id: String,
    /// Venue that produced the quote.
    pub venue: VenueRef,
    /// Chain on which the route applies.
    pub chain: String,
    /// Market pair.
    pub pair: MarketPair,
    /// Route style.
    pub route_kind: DexRouteKind,
    /// Input amount in token units.
    pub amount_in: f64,
    /// Output amount in token units.
    pub amount_out: f64,
    /// Estimated price impact in basis points.
    pub price_impact_bps: f64,
    /// Estimated venue/protocol fee in quote units.
    pub estimated_fee_quote: f64,
    /// Estimated gas/network fee in quote units.
    pub gas_fee_quote: f64,
    /// Age of source market data in milliseconds.
    pub market_data_age_ms: u64,
    /// Simulation status associated with this quote.
    pub simulation_status: DexSimulationStatus,
}

impl DexSwapQuoteResponse {
    /// Validate a quote response without trusting it as executable output.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("swap quote response", &self.id, &mut violations);
        validate_id("swap quote request", &self.request_id, &mut violations);
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "DEX_QUOTE_CHAIN_REQUIRED",
            "quote chain",
            &self.chain,
            &mut violations,
        );

        if let Err(source) = self.pair.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "DEX_QUOTE_PAIR_INVALID",
                source.to_string(),
            ));
        }

        for (code, label, value) in [
            (
                "DEX_QUOTE_AMOUNT_IN_INVALID",
                "quote amount in",
                self.amount_in,
            ),
            (
                "DEX_QUOTE_AMOUNT_OUT_INVALID",
                "quote amount out",
                self.amount_out,
            ),
        ] {
            if !is_positive_finite(value) {
                violations.push(DexConnectorViolation::new_owned(
                    code,
                    format!("{label} must be positive and finite"),
                ));
            }
        }

        for (code, label, value) in [
            (
                "DEX_QUOTE_PRICE_IMPACT_INVALID",
                "quote price impact",
                self.price_impact_bps,
            ),
            (
                "DEX_QUOTE_ESTIMATED_FEE_INVALID",
                "quote estimated fee",
                self.estimated_fee_quote,
            ),
            (
                "DEX_QUOTE_GAS_FEE_INVALID",
                "quote gas fee",
                self.gas_fee_quote,
            ),
        ] {
            if !is_non_negative_finite(value) {
                violations.push(DexConnectorViolation::new_owned(
                    code,
                    format!("{label} must be non-negative and finite"),
                ));
            }
        }

        finish_validation(violations)
    }
}

/// Local Web3 transaction simulation request boundary.
///
/// The request deliberately carries a payload hash/label rather than raw calldata
/// or a signed transaction. Phase 8 cannot use this model to call RPC, sign, or
/// broadcast.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3TransactionSimulationRequest {
    /// Stable simulation request id.
    pub id: String,
    /// Related swap quote request id.
    pub swap_request_id: String,
    /// Requested execution scope.
    pub scope: ExecutionScope,
    /// Target DEX/router venue.
    pub venue: VenueRef,
    /// Target chain.
    pub chain: String,
    /// Reviewed router label.
    pub router_label: String,
    /// Reviewed spender label.
    pub spender_label: String,
    /// Controlled account label only; never a private key or address generated by the LLM.
    pub account_label: String,
    /// Input token symbol.
    pub input_token_symbol: String,
    /// Output token symbol.
    pub output_token_symbol: String,
    /// Input amount in token units.
    pub amount_in: f64,
    /// Minimum acceptable output amount in token units.
    pub minimum_amount_out: f64,
    /// Estimated gas units for a future simulation adapter.
    pub gas_limit: u64,
    /// Maximum accepted gas fee in quote units.
    pub max_gas_fee_quote: f64,
    /// Hash or stable label of a reviewed unsigned payload; never raw calldata.
    pub payload_hash: String,
}

impl Web3TransactionSimulationRequest {
    /// Validate the request shape without RPC, signing, or broadcast.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("transaction simulation", &self.id, &mut violations);
        validate_id("swap quote request", &self.swap_request_id, &mut violations);
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "DEX_SIM_CHAIN_REQUIRED",
            "simulation chain",
            &self.chain,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SIM_ROUTER_LABEL_REQUIRED",
            "simulation router label",
            &self.router_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SIM_SPENDER_LABEL_REQUIRED",
            "simulation spender label",
            &self.spender_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SIM_ACCOUNT_LABEL_REQUIRED",
            "simulation account label",
            &self.account_label,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SIM_INPUT_TOKEN_REQUIRED",
            "simulation input token",
            &self.input_token_symbol,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SIM_OUTPUT_TOKEN_REQUIRED",
            "simulation output token",
            &self.output_token_symbol,
            &mut violations,
        );
        validate_non_empty(
            "DEX_SIM_PAYLOAD_HASH_REQUIRED",
            "simulation payload hash",
            &self.payload_hash,
            &mut violations,
        );

        if self
            .input_token_symbol
            .eq_ignore_ascii_case(&self.output_token_symbol)
        {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_INPUT_EQUALS_OUTPUT_TOKEN",
                "simulation input and output token symbols must differ",
            ));
        }

        if !is_positive_finite(self.amount_in) {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_AMOUNT_IN_INVALID",
                "simulation amount in must be positive and finite",
            ));
        }

        if !is_positive_finite(self.minimum_amount_out) {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_MIN_OUTPUT_INVALID",
                "simulation minimum output must be positive and finite",
            ));
        }

        if self.gas_limit == 0 {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_GAS_LIMIT_REQUIRED",
                "simulation gas limit must be non-zero",
            ));
        }

        if !is_non_negative_finite(self.max_gas_fee_quote) {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_MAX_GAS_FEE_INVALID",
                "simulation max gas fee must be non-negative and finite",
            ));
        }

        finish_validation(violations)
    }
}

/// Local Web3 transaction simulation response boundary.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3TransactionSimulationResponse {
    /// Stable response id.
    pub id: String,
    /// Request id this response belongs to.
    pub request_id: String,
    /// Simulation outcome status.
    pub status: DexSimulationStatus,
    /// Gas units used or estimated.
    pub gas_used: u64,
    /// Estimated gas fee in quote units.
    pub gas_fee_quote: f64,
    /// Estimated output amount in token units.
    pub amount_out: f64,
    /// Optional non-secret revert or diagnostic label.
    pub diagnostic: Option<String>,
    /// Must remain false in Phase 8; no response is broadcastable.
    pub broadcastable: bool,
}

/// Local Web3 nonce reservation request.
///
/// This boundary uses caller-supplied local nonce metadata only. It does not
/// call RPC, query account state, load signer material, sign, or broadcast.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3NonceReservationRequest {
    /// Stable reservation id.
    pub id: String,
    /// Chain label.
    pub chain: String,
    /// Target DEX/router venue.
    pub venue: VenueRef,
    /// Controlled account label only; never a private key.
    pub account_label: String,
    /// Last confirmed nonce from a caller-supplied local fixture or checkpoint.
    pub last_confirmed_nonce: Option<u64>,
    /// Requested nonce to reserve for the next local pre-sign review.
    pub requested_nonce: Option<u64>,
    /// Caller-supplied list of already reserved local nonces.
    pub in_flight_nonces: Vec<u64>,
    /// Local reservation time-to-live in milliseconds.
    pub ttl_ms: u64,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub planned_at_unix_ms: u64,
}

/// Local Web3 nonce reservation report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3NonceReservationReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable reservation id.
    pub id: String,
    /// Chain label.
    pub chain: String,
    /// Target DEX/router venue.
    pub venue: VenueRef,
    /// Controlled account label only; never a private key.
    pub account_label: String,
    /// Last confirmed nonce from local metadata.
    pub last_confirmed_nonce: Option<u64>,
    /// Reserved local nonce when ready.
    pub reserved_nonce: Option<u64>,
    /// Number of caller-supplied in-flight local nonces.
    pub in_flight_nonce_count: u64,
    /// Local reservation time-to-live in milliseconds.
    pub ttl_ms: u64,
    /// Local reservation status.
    pub status: Web3NonceReservationStatus,
    /// Whether nonce metadata is locally present and not stale or duplicated.
    pub nonce_ready: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local reservation never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local reservation never loads signer material.
    pub signer_material_loaded: bool,
    /// Local reservation never signs payloads.
    pub signing_performed: bool,
    /// Local reservation never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local reservation never performs live execution.
    pub live_execution_performed: bool,
    /// Local reservation never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub planned_at_unix_ms: u64,
}

/// Local Web3 unsigned payload review request.
///
/// This boundary reviews a non-secret payload hash/label and local nonce
/// reservation before pre-sign safety review. It never builds raw calldata,
/// requests signatures, calls RPC, or broadcasts.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3UnsignedPayloadReviewRequest {
    /// Stable review id.
    pub id: String,
    /// Simulation request whose payload metadata is being reviewed.
    pub simulation_request: Web3TransactionSimulationRequest,
    /// Local nonce reservation report.
    pub nonce_reservation: Web3NonceReservationReport,
    /// Reviewed unsigned payload hash or stable local label; never raw calldata.
    pub payload_hash: String,
    /// Reviewed router label.
    pub router_label: String,
    /// Reviewed spender label.
    pub spender_label: String,
    /// Maximum accepted gas fee in quote units.
    pub max_gas_fee_quote: f64,
    /// Whether raw calldata was embedded. Always false here.
    pub raw_calldata_embedded: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 unsigned payload review report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3UnsignedPayloadReviewReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable review id.
    pub id: String,
    /// Simulation request id.
    pub simulation_request_id: String,
    /// Nonce reservation id.
    pub nonce_reservation_id: String,
    /// Chain label.
    pub chain: String,
    /// Target DEX/router venue.
    pub venue: VenueRef,
    /// Controlled account label only; never a private key.
    pub account_label: String,
    /// Reserved nonce when ready.
    pub reserved_nonce: Option<u64>,
    /// Reviewed unsigned payload hash or stable label.
    pub payload_hash: String,
    /// Local review status.
    pub status: Web3UnsignedPayloadReviewStatus,
    /// Whether nonce reservation is locally ready and coherent.
    pub nonce_ready: bool,
    /// Whether payload reference is present and hash-like/label-only.
    pub payload_reference_ready: bool,
    /// Whether router and spender labels match the simulation request.
    pub router_spender_ready: bool,
    /// Whether gas cap metadata is locally acceptable.
    pub gas_cap_ready: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local review never embeds raw calldata.
    pub raw_calldata_embedded: bool,
    /// Local review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local review never signs payloads.
    pub signing_performed: bool,
    /// Local review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local review never performs live execution.
    pub live_execution_performed: bool,
    /// Local review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local pre-sign Web3 safety review request.
///
/// This request combines already-local simulation and lifecycle metadata. It
/// never performs RPC calls, never signs, and never broadcasts.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3PreSignSafetyReviewRequest {
    /// Stable review id.
    pub id: String,
    /// Simulation request under review.
    pub simulation_request: Web3TransactionSimulationRequest,
    /// Simulation response under review.
    pub simulation_response: Web3TransactionSimulationResponse,
    /// Optional parsed lifecycle record used for nonce/confirmation consistency.
    pub lifecycle_record: Option<Web3TransactionLifecycleRecord>,
    /// Maximum accepted gas fee in quote units.
    pub max_gas_fee_quote: f64,
    /// Whether a nonce is required before signer authorization.
    pub nonce_required: bool,
    /// Optional local nonce plan value.
    pub planned_nonce: Option<u64>,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local pre-sign Web3 safety review report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3PreSignSafetyReviewReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable review id.
    pub id: String,
    /// Simulation request id.
    pub simulation_request_id: String,
    /// Simulation response id.
    pub simulation_response_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local review status.
    pub status: Web3PreSignSafetyReviewStatus,
    /// Whether simulation reported a locally acceptable success path.
    pub simulation_success_ready: bool,
    /// Whether gas fee stayed under the supplied local cap.
    pub gas_fee_within_cap: bool,
    /// Whether the output amount met the request minimum.
    pub output_amount_sufficient: bool,
    /// Whether nonce metadata is present and coherent.
    pub nonce_ready: bool,
    /// Whether lifecycle metadata is local and coherent when supplied.
    pub lifecycle_coherent: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local review never signs payloads.
    pub signing_performed: bool,
    /// Local review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local review never performs live execution.
    pub live_execution_performed: bool,
    /// Local review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 broadcast-readiness review request.
///
/// This boundary composes reviewed unsigned-payload and pre-sign safety
/// metadata into an operator-review record. It never calls RPC, never loads
/// signer material, never signs, never broadcasts, and never authorizes a
/// transaction for broadcast.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3BroadcastReadinessRequest {
    /// Stable review id.
    pub id: String,
    /// Local unsigned payload review under consideration.
    pub unsigned_payload_review: Web3UnsignedPayloadReviewReport,
    /// Local pre-sign safety review under consideration.
    pub pre_sign_safety_review: Web3PreSignSafetyReviewReport,
    /// Non-secret signer authorization reference.
    pub signer_authorization_reference: String,
    /// Non-secret live adapter readiness reference.
    pub live_adapter_reference: String,
    /// Non-secret operator approval reference.
    pub operator_approval_reference: String,
    /// Whether this local review permits broadcast. Always false here.
    pub broadcast_allowed: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 broadcast-readiness review report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3BroadcastReadinessReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable review id.
    pub id: String,
    /// Local unsigned payload review id.
    pub unsigned_payload_review_id: String,
    /// Local pre-sign safety review id.
    pub pre_sign_safety_review_id: String,
    /// Simulation request id shared by both prerequisite reviews.
    pub simulation_request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local review status.
    pub status: Web3BroadcastReadinessStatus,
    /// Whether the unsigned-payload review is ready and coherent.
    pub unsigned_payload_ready: bool,
    /// Whether the pre-sign safety review is ready and coherent.
    pub pre_sign_safety_ready: bool,
    /// Whether the non-secret signer authorization reference is present.
    pub signer_authorization_reference_ready: bool,
    /// Whether the non-secret live adapter reference is present.
    pub live_adapter_reference_ready: bool,
    /// Whether the non-secret operator approval reference is present.
    pub operator_approval_reference_ready: bool,
    /// Whether this local review permits broadcast. Always false here.
    pub broadcast_allowed: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local review never signs payloads.
    pub signing_performed: bool,
    /// Local review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local review never performs live execution.
    pub live_execution_performed: bool,
    /// Local review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 unsigned transaction construction request.
///
/// This boundary records deterministic unsigned transaction metadata for
/// operator review. It does not embed raw calldata, serialize a raw
/// transaction, request signatures, call RPC, broadcast, or authorize live
/// execution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3UnsignedTransactionConstructionRequest {
    /// Stable construction id.
    pub id: String,
    /// Local broadcast-readiness review under consideration.
    pub broadcast_readiness_review: Web3BroadcastReadinessReport,
    /// Reviewed payload hash or stable local label; never raw calldata.
    pub payload_hash: String,
    /// Function selector or stable selector label.
    pub function_selector: String,
    /// Stable digest/label for encoded arguments; never raw encoded calldata.
    pub encoded_argument_digest: String,
    /// Target contract/router label.
    pub target_contract_label: String,
    /// Local nonce selected for the unsigned transaction metadata.
    pub nonce: Option<u64>,
    /// Local gas limit estimate supplied by fixture/review.
    pub gas_limit: u64,
    /// Maximum fee metadata in quote units supplied by fixture/review.
    pub max_fee_quote: f64,
    /// Whether raw calldata was embedded. Always false here.
    pub raw_calldata_embedded: bool,
    /// Whether a raw transaction was serialized. Always false here.
    pub raw_transaction_serialized: bool,
    /// Whether this local construction permits broadcast. Always false here.
    pub broadcast_allowed: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub constructed_at_unix_ms: u64,
}

/// Local Web3 unsigned transaction construction report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3UnsignedTransactionConstructionReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable construction id.
    pub id: String,
    /// Broadcast-readiness review id.
    pub broadcast_readiness_review_id: String,
    /// Simulation request id carried through prerequisite reviews.
    pub simulation_request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local construction status.
    pub status: Web3UnsignedTransactionConstructionStatus,
    /// Whether broadcast-readiness prerequisites are locally ready and non-broadcasting.
    pub broadcast_readiness_ready: bool,
    /// Whether payload reference is hash-like/label-only and coherent.
    pub payload_reference_ready: bool,
    /// Whether target contract and selector labels are present and sanitized.
    pub target_selector_ready: bool,
    /// Whether nonce metadata is present.
    pub nonce_ready: bool,
    /// Local construction nonce under review.
    pub construction_nonce: Option<u64>,
    /// Whether gas/fee metadata is locally acceptable.
    pub gas_metadata_ready: bool,
    /// Stable local unsigned transaction reference, never a raw transaction.
    pub unsigned_transaction_reference: String,
    /// Stable digest/label for encoded arguments, never raw calldata.
    pub encoded_argument_digest: String,
    /// Whether raw calldata was embedded. Always false here.
    pub raw_calldata_embedded: bool,
    /// Whether a raw transaction was serialized. Always false here.
    pub raw_transaction_serialized: bool,
    /// Whether this local construction permits broadcast. Always false here.
    pub broadcast_allowed: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local construction never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local construction never loads signer material.
    pub signer_material_loaded: bool,
    /// Local construction never signs payloads.
    pub signing_performed: bool,
    /// Local construction never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local construction never performs live execution.
    pub live_execution_performed: bool,
    /// Local construction never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub constructed_at_unix_ms: u64,
}

/// Local Web3 provider nonce reconciliation request.
///
/// This boundary reconciles caller-supplied provider nonce metadata against
/// local unsigned transaction construction metadata. It never calls RPC,
/// queries chain state, loads signer material, signs, broadcasts, or claims
/// production readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3ProviderNonceReconciliationRequest {
    /// Stable reconciliation id.
    pub id: String,
    /// Local unsigned transaction construction under review.
    pub unsigned_transaction_construction: Web3UnsignedTransactionConstructionReport,
    /// Non-secret provider snapshot/reference label; never an RPC URL or token.
    pub provider_snapshot_reference: String,
    /// Provider-observed next nonce from caller-supplied local evidence.
    pub provider_next_nonce: Option<u64>,
    /// Caller-supplied pending nonces from local evidence.
    pub provider_pending_nonces: Vec<u64>,
    /// Maximum accepted age for the provider snapshot.
    pub max_snapshot_age_ms: u64,
    /// Actual age of the caller-supplied provider snapshot.
    pub snapshot_age_ms: u64,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this reconciliation claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reconciled_at_unix_ms: u64,
}

/// Local Web3 provider nonce reconciliation report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3ProviderNonceReconciliationReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable reconciliation id.
    pub id: String,
    /// Unsigned transaction construction id.
    pub unsigned_transaction_construction_id: String,
    /// Simulation request id carried through prerequisite reviews.
    pub simulation_request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local reconciliation status.
    pub status: Web3ProviderNonceReconciliationStatus,
    /// Whether unsigned transaction construction is locally ready and non-side-effectful.
    pub unsigned_transaction_ready: bool,
    /// Whether provider snapshot reference is present and sanitized.
    pub provider_snapshot_reference_ready: bool,
    /// Whether provider next nonce is present.
    pub provider_next_nonce_ready: bool,
    /// Whether the construction nonce matches the provider next nonce.
    pub construction_nonce_matches_provider: bool,
    /// Whether construction nonce is absent from pending provider nonces.
    pub construction_nonce_not_pending: bool,
    /// Whether provider pending nonce metadata contains no duplicates.
    pub pending_nonce_set_unique: bool,
    /// Whether snapshot age is within the supplied cap.
    pub snapshot_fresh: bool,
    /// Construction nonce under review.
    pub construction_nonce: Option<u64>,
    /// Provider-observed next nonce from caller-supplied local evidence.
    pub provider_next_nonce: Option<u64>,
    /// Number of caller-supplied pending nonces.
    pub provider_pending_nonce_count: u64,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local reconciliation never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local reconciliation never loads signer material.
    pub signer_material_loaded: bool,
    /// Local reconciliation never signs payloads.
    pub signing_performed: bool,
    /// Local reconciliation never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local reconciliation never performs live execution.
    pub live_execution_performed: bool,
    /// Local reconciliation never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reconciled_at_unix_ms: u64,
}

/// Local Web3 raw transaction serialization review request.
///
/// This boundary reviews metadata needed before any future serializer can be
/// considered. It never serializes raw transaction bytes, embeds raw calldata,
/// loads signer material, signs, broadcasts, calls RPC, or claims production
/// readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3RawTransactionSerializationReviewRequest {
    /// Stable review id.
    pub id: String,
    /// Local provider nonce reconciliation under review.
    pub provider_nonce_reconciliation: Web3ProviderNonceReconciliationReport,
    /// Stable transaction type label, such as eip1559-local-review.
    pub transaction_type_label: String,
    /// Chain id reference label or decimal id supplied by local review.
    pub chain_id_reference: String,
    /// Fee field reference/digest; never raw signed transaction bytes.
    pub fee_field_reference: String,
    /// Access-list or account-meta reference label; never raw bytes.
    pub access_list_reference: String,
    /// Whether raw transaction bytes were embedded. Always false here.
    pub raw_transaction_bytes_embedded: bool,
    /// Whether raw calldata was embedded. Always false here.
    pub raw_calldata_embedded: bool,
    /// Whether a raw transaction was serialized. Always false here.
    pub raw_transaction_serialized: bool,
    /// Whether this local review permits broadcast. Always false here.
    pub broadcast_allowed: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 raw transaction serialization review report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3RawTransactionSerializationReviewReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable review id.
    pub id: String,
    /// Provider nonce reconciliation id.
    pub provider_nonce_reconciliation_id: String,
    /// Unsigned transaction construction id.
    pub unsigned_transaction_construction_id: String,
    /// Simulation request id carried through prerequisite reviews.
    pub simulation_request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local serialization review status.
    pub status: Web3RawTransactionSerializationReviewStatus,
    /// Whether provider nonce reconciliation is locally ready and non-side-effectful.
    pub provider_nonce_reconciliation_ready: bool,
    /// Whether transaction type metadata is present and sanitized.
    pub transaction_type_ready: bool,
    /// Whether chain id metadata is present and sanitized.
    pub chain_id_ready: bool,
    /// Whether fee field metadata is present and sanitized.
    pub fee_fields_ready: bool,
    /// Whether access-list/account-meta metadata is present and sanitized.
    pub access_list_reference_ready: bool,
    /// Stable local unsigned transaction reference, never raw transaction bytes.
    pub unsigned_transaction_reference: String,
    /// Whether raw transaction bytes were embedded. Always false here.
    pub raw_transaction_bytes_embedded: bool,
    /// Whether raw calldata was embedded. Always false here.
    pub raw_calldata_embedded: bool,
    /// Whether a raw transaction was serialized. Always false here.
    pub raw_transaction_serialized: bool,
    /// Whether this local review permits broadcast. Always false here.
    pub broadcast_allowed: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local review never signs payloads.
    pub signing_performed: bool,
    /// Local review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local review never performs live execution.
    pub live_execution_performed: bool,
    /// Local review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 broadcast adapter control review request.
///
/// This boundary reviews future broadcast adapter controls without submitting
/// transactions. It never broadcasts, calls RPC, loads signer material, signs,
/// embeds raw transaction bytes, or claims production readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3BroadcastAdapterControlReviewRequest {
    /// Stable review id.
    pub id: String,
    /// Local raw transaction serialization review under consideration.
    pub raw_transaction_serialization_review: Web3RawTransactionSerializationReviewReport,
    /// Non-secret broadcast adapter reference label.
    pub adapter_reference: String,
    /// Non-secret operator approval reference label.
    pub operator_approval_reference: String,
    /// Non-secret audit/state preflight reference label.
    pub audit_state_preflight_reference: String,
    /// Whether the kill switch is confirmed active for future broadcast paths.
    pub kill_switch_confirmed: bool,
    /// Whether rate-limit control metadata exists.
    pub rate_limit_control_ready: bool,
    /// Whether replay/idempotency control metadata exists.
    pub replay_protection_ready: bool,
    /// Whether broadcast permission was granted. Always false here.
    pub broadcast_permission_granted: bool,
    /// Whether raw transaction bytes were embedded. Always false here.
    pub raw_transaction_bytes_embedded: bool,
    /// Whether a raw transaction was serialized. Always false here.
    pub raw_transaction_serialized: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this review claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 broadcast adapter control review report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3BroadcastAdapterControlReviewReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable review id.
    pub id: String,
    /// Raw transaction serialization review id.
    pub raw_transaction_serialization_review_id: String,
    /// Provider nonce reconciliation id.
    pub provider_nonce_reconciliation_id: String,
    /// Unsigned transaction construction id.
    pub unsigned_transaction_construction_id: String,
    /// Simulation request id carried through prerequisite reviews.
    pub simulation_request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local broadcast adapter control status.
    pub status: Web3BroadcastAdapterControlReviewStatus,
    /// Whether raw transaction serialization review is locally ready and non-side-effectful.
    pub raw_transaction_serialization_review_ready: bool,
    /// Whether broadcast adapter reference is present and sanitized.
    pub adapter_reference_ready: bool,
    /// Whether operator approval reference is present and sanitized.
    pub operator_approval_reference_ready: bool,
    /// Whether audit/state preflight reference is present and sanitized.
    pub audit_state_preflight_reference_ready: bool,
    /// Whether kill-switch metadata is confirmed.
    pub kill_switch_confirmed: bool,
    /// Whether rate-limit control metadata exists.
    pub rate_limit_control_ready: bool,
    /// Whether replay/idempotency control metadata exists.
    pub replay_protection_ready: bool,
    /// Whether broadcast permission was granted. Always false here.
    pub broadcast_permission_granted: bool,
    /// Whether raw transaction bytes were embedded. Always false here.
    pub raw_transaction_bytes_embedded: bool,
    /// Whether a raw transaction was serialized. Always false here.
    pub raw_transaction_serialized: bool,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local review never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local review never loads signer material.
    pub signer_material_loaded: bool,
    /// Local review never signs payloads.
    pub signing_performed: bool,
    /// Local review never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local review never performs live execution.
    pub live_execution_performed: bool,
    /// Local review never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub reviewed_at_unix_ms: u64,
}

/// Local Web3 sandbox/live discrepancy calibration request.
///
/// This boundary compares caller-supplied, non-secret sandbox/live evidence
/// references and numeric tolerances. It does not call exchanges, call RPC,
/// load credentials, sign, broadcast, execute live, or claim production
/// readiness.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3SandboxLiveDiscrepancyCalibrationRequest {
    /// Stable calibration id.
    pub id: String,
    /// Local broadcast adapter control review under consideration.
    pub broadcast_adapter_control_review: Web3BroadcastAdapterControlReviewReport,
    /// Non-secret sandbox observation reference label.
    pub sandbox_observation_reference: String,
    /// Non-secret live observation reference label.
    pub live_observation_reference: String,
    /// Maximum accepted price discrepancy in basis points.
    pub max_price_deviation_bps: f64,
    /// Observed caller-supplied price discrepancy in basis points.
    pub observed_price_deviation_bps: f64,
    /// Maximum accepted latency discrepancy in milliseconds.
    pub max_latency_deviation_ms: u64,
    /// Observed caller-supplied latency discrepancy in milliseconds.
    pub observed_latency_deviation_ms: u64,
    /// Maximum accepted fee discrepancy in quote units.
    pub max_fee_deviation_quote: f64,
    /// Observed caller-supplied fee discrepancy in quote units.
    pub observed_fee_deviation_quote: f64,
    /// Minimum sandbox/live sample count required for local review.
    pub minimum_sample_count: u64,
    /// Caller-supplied sandbox sample count.
    pub sandbox_sample_count: u64,
    /// Caller-supplied live sample count.
    pub live_sample_count: u64,
    /// Whether an external call occurred. Always false here.
    pub external_call_performed: bool,
    /// Whether credentials were loaded. Always false here.
    pub credential_loaded: bool,
    /// Whether an RPC call occurred. Always false here.
    pub rpc_called: bool,
    /// Whether signer material was loaded. Always false here.
    pub signer_material_loaded: bool,
    /// Whether signing occurred. Always false here.
    pub signing_performed: bool,
    /// Whether broadcast occurred. Always false here.
    pub broadcast_performed: bool,
    /// Whether live execution occurred. Always false here.
    pub live_execution_performed: bool,
    /// Whether this calibration claims production readiness. Always false here.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub calibrated_at_unix_ms: u64,
}

/// Local Web3 sandbox/live discrepancy calibration report.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Web3SandboxLiveDiscrepancyCalibrationReport {
    /// Stable DEX/Web3 framework version.
    pub framework_version: String,
    /// Stable calibration id.
    pub id: String,
    /// Broadcast adapter control review id.
    pub broadcast_adapter_control_review_id: String,
    /// Raw transaction serialization review id.
    pub raw_transaction_serialization_review_id: String,
    /// Provider nonce reconciliation id.
    pub provider_nonce_reconciliation_id: String,
    /// Simulation request id carried through prerequisite reviews.
    pub simulation_request_id: String,
    /// Chain label.
    pub chain: String,
    /// Venue/router label.
    pub venue: VenueRef,
    /// Local discrepancy calibration status.
    pub status: Web3SandboxLiveDiscrepancyCalibrationStatus,
    /// Whether broadcast adapter controls are locally ready and non-side-effectful.
    pub broadcast_adapter_control_ready: bool,
    /// Whether sandbox observation reference is present and sanitized.
    pub sandbox_observation_reference_ready: bool,
    /// Whether live observation reference is present and sanitized.
    pub live_observation_reference_ready: bool,
    /// Whether sandbox and live sample counts meet the local minimum.
    pub sample_size_ready: bool,
    /// Whether price discrepancy is within the supplied cap.
    pub price_deviation_within_limit: bool,
    /// Whether latency discrepancy is within the supplied cap.
    pub latency_deviation_within_limit: bool,
    /// Whether fee discrepancy is within the supplied cap.
    pub fee_deviation_within_limit: bool,
    /// Caller-supplied sandbox sample count.
    pub sandbox_sample_count: u64,
    /// Caller-supplied live sample count.
    pub live_sample_count: u64,
    /// Stable validation/denial codes.
    pub violation_codes: Vec<String>,
    /// Number of validation/denial codes.
    pub violation_count: u64,
    /// Local calibration never performs external calls.
    pub external_call_performed: bool,
    /// Local calibration never loads credentials.
    pub credential_loaded: bool,
    /// Local calibration never calls RPC endpoints.
    pub rpc_called: bool,
    /// Local calibration never loads signer material.
    pub signer_material_loaded: bool,
    /// Local calibration never signs payloads.
    pub signing_performed: bool,
    /// Local calibration never broadcasts transactions.
    pub broadcast_performed: bool,
    /// Local calibration never performs live execution.
    pub live_execution_performed: bool,
    /// Local calibration never records production readiness.
    pub production_ready: bool,
    /// Operator-supplied non-secret timestamp.
    pub calibrated_at_unix_ms: u64,
}

impl Web3TransactionSimulationResponse {
    /// Validate the response and enforce non-broadcastable Phase 8 behavior.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("transaction simulation response", &self.id, &mut violations);
        validate_id(
            "transaction simulation request",
            &self.request_id,
            &mut violations,
        );

        if self.gas_used == 0 {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_RESPONSE_GAS_USED_REQUIRED",
                "simulation response gas used must be non-zero",
            ));
        }

        if !is_non_negative_finite(self.gas_fee_quote) {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_RESPONSE_GAS_FEE_INVALID",
                "simulation response gas fee must be non-negative and finite",
            ));
        }

        if !is_non_negative_finite(self.amount_out) {
            violations.push(DexConnectorViolation::new(
                "DEX_SIM_RESPONSE_AMOUNT_OUT_INVALID",
                "simulation response amount out must be non-negative and finite",
            ));
        }

        if self.broadcastable {
            violations.push(DexConnectorViolation::new(
                "DEX_PHASE8_RESPONSE_NOT_BROADCASTABLE",
                "Phase 8 simulation responses must never be marked broadcastable",
            ));
        }

        finish_validation(violations)
    }
}

impl Web3NonceReservationReport {
    /// Validate this reservation remains local, coherent, and side-effect free.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_NONCE_FRAMEWORK_VERSION_MISMATCH",
                "nonce reservation framework version mismatch",
            ));
        }
        validate_id("nonce reservation", &self.id, &mut violations);
        validate_non_empty(
            "WEB3_NONCE_CHAIN_REQUIRED",
            "nonce reservation chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "WEB3_NONCE_ACCOUNT_LABEL_REQUIRED",
            "nonce reservation account label",
            &self.account_label,
            &mut violations,
        );
        if self.ttl_ms == 0 {
            violations.push(DexConnectorViolation::new(
                "WEB3_NONCE_TTL_REQUIRED",
                "nonce reservation ttl must be non-zero",
            ));
        }
        if self.planned_at_unix_ms == 0 {
            violations.push(DexConnectorViolation::new(
                "WEB3_NONCE_TIMESTAMP_REQUIRED",
                "nonce reservation timestamp is required",
            ));
        }
        if self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_NONCE_SIDE_EFFECT_FLAG",
                "nonce reservation must not call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_NONCE_VIOLATION_COUNT_MISMATCH",
                "nonce reservation violation count mismatch",
            ));
        }
        match self.status {
            Web3NonceReservationStatus::ReservedForLocalReview => {
                if !self.nonce_ready
                    || self.reserved_nonce.is_none()
                    || !self.violation_codes.is_empty()
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_NONCE_READY_CONTROLS_INCOMPLETE",
                        "ready nonce reservations require a nonce and no violation codes",
                    ));
                }
            }
            Web3NonceReservationStatus::Blocked => {
                if self.violation_codes.is_empty() {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_NONCE_BLOCKED_WITHOUT_CODES",
                        "blocked nonce reservations require violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3NonceReservationRequest {
    /// Review local nonce metadata without RPC, signing, or broadcast.
    #[must_use]
    pub fn reserve(&self) -> Web3NonceReservationReport {
        let mut violation_codes = Vec::new();
        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_NONCE_RESERVATION_ID_REQUIRED".to_owned());
        }
        if self.chain.trim().is_empty() {
            violation_codes.push("WEB3_NONCE_CHAIN_REQUIRED".to_owned());
        }
        if validate_dex_venue_ref_result(&self.venue).is_err() {
            violation_codes.push("WEB3_NONCE_VENUE_INVALID".to_owned());
        }
        if self.account_label.trim().is_empty() {
            violation_codes.push("WEB3_NONCE_ACCOUNT_LABEL_REQUIRED".to_owned());
        }
        if self.ttl_ms == 0 {
            violation_codes.push("WEB3_NONCE_TTL_REQUIRED".to_owned());
        }
        if self.planned_at_unix_ms == 0 {
            violation_codes.push("WEB3_NONCE_TIMESTAMP_REQUIRED".to_owned());
        }

        let duplicate_in_flight = has_duplicate_nonce(&self.in_flight_nonces);
        if duplicate_in_flight {
            violation_codes.push("WEB3_NONCE_IN_FLIGHT_DUPLICATE".to_owned());
        }
        let nonce_present = self.requested_nonce.is_some();
        if !nonce_present {
            violation_codes.push("WEB3_NONCE_REQUESTED_REQUIRED".to_owned());
        }
        let stale_nonce = self
            .requested_nonce
            .zip(self.last_confirmed_nonce)
            .is_some_and(|(requested, confirmed)| requested <= confirmed);
        if stale_nonce {
            violation_codes.push("WEB3_NONCE_STALE".to_owned());
        }
        let already_reserved = self
            .requested_nonce
            .is_some_and(|nonce| self.in_flight_nonces.contains(&nonce));
        if already_reserved {
            violation_codes.push("WEB3_NONCE_ALREADY_RESERVED".to_owned());
        }
        if self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_NONCE_SIDE_EFFECT_FLAG".to_owned());
        }

        let nonce_ready =
            nonce_present && !stale_nonce && !already_reserved && !duplicate_in_flight;
        let status = if violation_codes.is_empty() {
            Web3NonceReservationStatus::ReservedForLocalReview
        } else {
            Web3NonceReservationStatus::Blocked
        };

        Web3NonceReservationReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            chain: self.chain.clone(),
            venue: self.venue.clone(),
            account_label: self.account_label.clone(),
            last_confirmed_nonce: self.last_confirmed_nonce,
            reserved_nonce: self
                .requested_nonce
                .filter(|_| status == Web3NonceReservationStatus::ReservedForLocalReview),
            in_flight_nonce_count: self.in_flight_nonces.len() as u64,
            ttl_ms: self.ttl_ms,
            status,
            nonce_ready,
            violation_count: violation_codes.len() as u64,
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            planned_at_unix_ms: self.planned_at_unix_ms,
        }
    }
}

impl Web3UnsignedPayloadReviewReport {
    /// Validate this review remains local, coherent, and side-effect free.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_PAYLOAD_FRAMEWORK_VERSION_MISMATCH",
                "unsigned payload review framework version mismatch",
            ));
        }
        validate_id("unsigned payload review", &self.id, &mut violations);
        validate_id(
            "unsigned payload simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_id(
            "unsigned payload nonce reservation",
            &self.nonce_reservation_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_UNSIGNED_PAYLOAD_CHAIN_REQUIRED",
            "unsigned payload chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        validate_non_empty(
            "WEB3_UNSIGNED_PAYLOAD_ACCOUNT_LABEL_REQUIRED",
            "unsigned payload account label",
            &self.account_label,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_UNSIGNED_PAYLOAD_REFERENCE_REQUIRED",
            "unsigned payload reference",
            &self.payload_hash,
            &mut violations,
        );
        if self.raw_calldata_embedded
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_PAYLOAD_SIDE_EFFECT_FLAG",
                "unsigned payload review must not embed raw calldata, call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_PAYLOAD_VIOLATION_COUNT_MISMATCH",
                "unsigned payload review violation count mismatch",
            ));
        }
        match self.status {
            Web3UnsignedPayloadReviewStatus::ReadyForLocalReview => {
                if !self.nonce_ready
                    || !self.payload_reference_ready
                    || !self.router_spender_ready
                    || !self.gas_cap_ready
                    || !self.violation_codes.is_empty()
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_UNSIGNED_PAYLOAD_READY_CONTROLS_INCOMPLETE",
                        "ready unsigned payload reviews require nonce, payload, router, spender, and gas controls",
                    ));
                }
            }
            Web3UnsignedPayloadReviewStatus::Blocked => {
                if self.violation_codes.is_empty() {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_UNSIGNED_PAYLOAD_BLOCKED_WITHOUT_CODES",
                        "blocked unsigned payload reviews require violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3UnsignedPayloadReviewRequest {
    /// Review local unsigned payload metadata without constructing raw calldata.
    #[must_use]
    pub fn review(&self) -> Web3UnsignedPayloadReviewReport {
        let mut violation_codes = Vec::new();
        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_REVIEW_ID_REQUIRED".to_owned());
        }
        if self.reviewed_at_unix_ms == 0 {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.simulation_request.validate().is_err() {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_SIMULATION_REQUEST_INVALID".to_owned());
        }
        if self.nonce_reservation.validate().is_err() {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_NONCE_RESERVATION_INVALID".to_owned());
        }

        let nonce_ready = self.nonce_reservation.status
            == Web3NonceReservationStatus::ReservedForLocalReview
            && self.nonce_reservation.nonce_ready
            && self.nonce_reservation.reserved_nonce.is_some()
            && self
                .nonce_reservation
                .chain
                .eq_ignore_ascii_case(&self.simulation_request.chain)
            && same_venue(
                &self.nonce_reservation.venue,
                &self.simulation_request.venue,
            )
            && self
                .nonce_reservation
                .account_label
                .eq_ignore_ascii_case(&self.simulation_request.account_label);
        if !nonce_ready {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_NONCE_NOT_READY".to_owned());
        }

        let payload_reference_ready = payload_reference_is_hash_or_label(&self.payload_hash)
            && self.payload_hash == self.simulation_request.payload_hash;
        if !payload_reference_ready {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_REFERENCE_INVALID".to_owned());
        }

        let router_spender_ready = self
            .router_label
            .eq_ignore_ascii_case(&self.simulation_request.router_label)
            && self
                .spender_label
                .eq_ignore_ascii_case(&self.simulation_request.spender_label);
        if !router_spender_ready {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_ROUTER_SPENDER_MISMATCH".to_owned());
        }

        let gas_cap_ready = is_non_negative_finite(self.max_gas_fee_quote)
            && self.max_gas_fee_quote <= self.simulation_request.max_gas_fee_quote;
        if !gas_cap_ready {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_GAS_CAP_INVALID".to_owned());
        }

        if self.raw_calldata_embedded
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_UNSIGNED_PAYLOAD_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3UnsignedPayloadReviewStatus::ReadyForLocalReview
        } else {
            Web3UnsignedPayloadReviewStatus::Blocked
        };

        Web3UnsignedPayloadReviewReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            simulation_request_id: self.simulation_request.id.clone(),
            nonce_reservation_id: self.nonce_reservation.id.clone(),
            chain: self.simulation_request.chain.clone(),
            venue: self.simulation_request.venue.clone(),
            account_label: self.simulation_request.account_label.clone(),
            reserved_nonce: self
                .nonce_reservation
                .reserved_nonce
                .filter(|_| status == Web3UnsignedPayloadReviewStatus::ReadyForLocalReview),
            payload_hash: self.payload_hash.clone(),
            status,
            nonce_ready,
            payload_reference_ready,
            router_spender_ready,
            gas_cap_ready,
            violation_count: violation_codes.len() as u64,
            violation_codes,
            raw_calldata_embedded: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: self.reviewed_at_unix_ms,
        }
    }
}

impl Web3PreSignSafetyReviewReport {
    /// Validate this review remains local, coherent, and side-effect free.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_PRE_SIGN_FRAMEWORK_VERSION_MISMATCH",
                "pre-sign safety review framework version mismatch",
            ));
        }
        validate_id("pre-sign safety review", &self.id, &mut violations);
        validate_id(
            "pre-sign simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_id(
            "pre-sign simulation response",
            &self.simulation_response_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_PRE_SIGN_CHAIN_REQUIRED",
            "pre-sign chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_PRE_SIGN_SIDE_EFFECT_FLAG",
                "pre-sign safety review must not call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_PRE_SIGN_VIOLATION_COUNT_MISMATCH",
                "pre-sign safety review violation count mismatch",
            ));
        }
        match self.status {
            Web3PreSignSafetyReviewStatus::ReadyForLocalReview => {
                if self.violation_count != 0
                    || !self.simulation_success_ready
                    || !self.gas_fee_within_cap
                    || !self.output_amount_sufficient
                    || !self.nonce_ready
                    || !self.lifecycle_coherent
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_PRE_SIGN_READY_REQUIRES_ALL_CONTROLS",
                        "ready pre-sign safety reviews require simulation, gas, output, nonce, and lifecycle controls",
                    ));
                }
            }
            Web3PreSignSafetyReviewStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_PRE_SIGN_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked pre-sign safety reviews require violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3PreSignSafetyReviewRequest {
    /// Review local pre-sign Web3 metadata without RPC, signing, or broadcast.
    #[must_use]
    pub fn review(&self) -> Web3PreSignSafetyReviewReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_PRE_SIGN_REVIEW_ID_REQUIRED".to_owned());
        }
        if self.reviewed_at_unix_ms == 0 {
            violation_codes.push("WEB3_PRE_SIGN_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.simulation_request.validate().is_err() {
            violation_codes.push("WEB3_PRE_SIGN_SIMULATION_REQUEST_INVALID".to_owned());
        }
        if self.simulation_response.validate().is_err() {
            violation_codes.push("WEB3_PRE_SIGN_SIMULATION_RESPONSE_INVALID".to_owned());
        }
        if self.simulation_response.request_id != self.simulation_request.id {
            violation_codes.push("WEB3_PRE_SIGN_SIMULATION_ID_MISMATCH".to_owned());
        }
        if !is_non_negative_finite(self.max_gas_fee_quote) {
            violation_codes.push("WEB3_PRE_SIGN_GAS_CAP_INVALID".to_owned());
        }

        let simulation_success_ready = matches!(
            self.simulation_response.status,
            DexSimulationStatus::LocallyValidated | DexSimulationStatus::WouldSucceed
        ) && !self.simulation_response.broadcastable;
        if !simulation_success_ready {
            violation_codes.push("WEB3_PRE_SIGN_SIMULATION_NOT_READY".to_owned());
        }

        let gas_fee_within_cap = self.simulation_response.gas_fee_quote <= self.max_gas_fee_quote;
        if !gas_fee_within_cap {
            violation_codes.push("WEB3_PRE_SIGN_GAS_FEE_EXCEEDS_CAP".to_owned());
        }

        let output_amount_sufficient =
            self.simulation_response.amount_out >= self.simulation_request.minimum_amount_out;
        if !output_amount_sufficient {
            violation_codes.push("WEB3_PRE_SIGN_OUTPUT_BELOW_MINIMUM".to_owned());
        }

        let nonce_ready = if self.nonce_required {
            self.planned_nonce.is_some()
        } else {
            true
        };
        if !nonce_ready {
            violation_codes.push("WEB3_PRE_SIGN_NONCE_REQUIRED".to_owned());
        }

        let lifecycle_coherent = self.lifecycle_record.as_ref().map_or(true, |record| {
            record.request_id == self.simulation_request.id
                && record
                    .chain
                    .eq_ignore_ascii_case(&self.simulation_request.chain)
                && same_venue(&record.venue, &self.simulation_request.venue)
                && !record.rpc_call_performed
                && !record.signing_performed
                && !record.broadcast_performed
                && !record.live_execution_performed
                && !record.production_ready
                && (!self.nonce_required || record.nonce == self.planned_nonce)
        });
        if !lifecycle_coherent {
            violation_codes.push("WEB3_PRE_SIGN_LIFECYCLE_INCOHERENT".to_owned());
        }

        if self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_PRE_SIGN_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3PreSignSafetyReviewStatus::ReadyForLocalReview
        } else {
            Web3PreSignSafetyReviewStatus::Blocked
        };

        Web3PreSignSafetyReviewReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            simulation_request_id: self.simulation_request.id.clone(),
            simulation_response_id: self.simulation_response.id.clone(),
            chain: self.simulation_request.chain.clone(),
            venue: self.simulation_request.venue.clone(),
            status,
            simulation_success_ready,
            gas_fee_within_cap,
            output_amount_sufficient,
            nonce_ready,
            lifecycle_coherent,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: self.reviewed_at_unix_ms,
        }
    }
}

impl Web3BroadcastReadinessReport {
    /// Validate this review remains local, coherent, and non-broadcasting.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_BROADCAST_READINESS_FRAMEWORK_VERSION_MISMATCH",
                "broadcast readiness framework version mismatch",
            ));
        }
        validate_id("broadcast readiness review", &self.id, &mut violations);
        validate_id(
            "broadcast unsigned payload review",
            &self.unsigned_payload_review_id,
            &mut violations,
        );
        validate_id(
            "broadcast pre-sign safety review",
            &self.pre_sign_safety_review_id,
            &mut violations,
        );
        validate_id(
            "broadcast simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_BROADCAST_READINESS_CHAIN_REQUIRED",
            "broadcast readiness chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if self.broadcast_allowed
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_BROADCAST_READINESS_SIDE_EFFECT_FLAG",
                "broadcast readiness review must not allow broadcast, call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_BROADCAST_READINESS_VIOLATION_COUNT_MISMATCH",
                "broadcast readiness violation count mismatch",
            ));
        }
        match self.status {
            Web3BroadcastReadinessStatus::ReadyForExternalReview => {
                if self.violation_count != 0
                    || !self.unsigned_payload_ready
                    || !self.pre_sign_safety_ready
                    || !self.signer_authorization_reference_ready
                    || !self.live_adapter_reference_ready
                    || !self.operator_approval_reference_ready
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_BROADCAST_READINESS_READY_REQUIRES_REFERENCES",
                        "ready broadcast-readiness reviews require local payload/pre-sign readiness and all external-review references",
                    ));
                }
            }
            Web3BroadcastReadinessStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_BROADCAST_READINESS_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked broadcast-readiness reviews require violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3BroadcastReadinessRequest {
    /// Review local prerequisites without enabling signing or broadcast.
    #[must_use]
    pub fn review(&self) -> Web3BroadcastReadinessReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_BROADCAST_READINESS_ID_REQUIRED".to_owned());
        }
        if self.reviewed_at_unix_ms == 0 {
            violation_codes.push("WEB3_BROADCAST_READINESS_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.unsigned_payload_review.validate().is_err() {
            violation_codes.push("WEB3_BROADCAST_READINESS_UNSIGNED_PAYLOAD_INVALID".to_owned());
        }
        if self.pre_sign_safety_review.validate().is_err() {
            violation_codes.push("WEB3_BROADCAST_READINESS_PRE_SIGN_INVALID".to_owned());
        }

        let unsigned_payload_ready = self.unsigned_payload_review.status
            == Web3UnsignedPayloadReviewStatus::ReadyForLocalReview
            && self.unsigned_payload_review.nonce_ready
            && self.unsigned_payload_review.payload_reference_ready
            && self.unsigned_payload_review.router_spender_ready
            && self.unsigned_payload_review.gas_cap_ready
            && !self.unsigned_payload_review.raw_calldata_embedded
            && !self.unsigned_payload_review.rpc_called
            && !self.unsigned_payload_review.signer_material_loaded
            && !self.unsigned_payload_review.signing_performed
            && !self.unsigned_payload_review.broadcast_performed
            && !self.unsigned_payload_review.live_execution_performed
            && !self.unsigned_payload_review.production_ready;
        if !unsigned_payload_ready {
            violation_codes.push("WEB3_BROADCAST_READINESS_UNSIGNED_PAYLOAD_NOT_READY".to_owned());
        }

        let pre_sign_safety_ready = self.pre_sign_safety_review.status
            == Web3PreSignSafetyReviewStatus::ReadyForLocalReview
            && self.pre_sign_safety_review.simulation_success_ready
            && self.pre_sign_safety_review.gas_fee_within_cap
            && self.pre_sign_safety_review.output_amount_sufficient
            && self.pre_sign_safety_review.nonce_ready
            && self.pre_sign_safety_review.lifecycle_coherent
            && !self.pre_sign_safety_review.rpc_called
            && !self.pre_sign_safety_review.signer_material_loaded
            && !self.pre_sign_safety_review.signing_performed
            && !self.pre_sign_safety_review.broadcast_performed
            && !self.pre_sign_safety_review.live_execution_performed
            && !self.pre_sign_safety_review.production_ready;
        if !pre_sign_safety_ready {
            violation_codes.push("WEB3_BROADCAST_READINESS_PRE_SIGN_NOT_READY".to_owned());
        }

        let prerequisites_match = self
            .unsigned_payload_review
            .chain
            .eq_ignore_ascii_case(&self.pre_sign_safety_review.chain)
            && same_venue(
                &self.unsigned_payload_review.venue,
                &self.pre_sign_safety_review.venue,
            )
            && self.unsigned_payload_review.simulation_request_id
                == self.pre_sign_safety_review.simulation_request_id;
        if !prerequisites_match {
            violation_codes.push("WEB3_BROADCAST_READINESS_PREREQUISITE_MISMATCH".to_owned());
        }

        let signer_authorization_reference_ready =
            payload_reference_is_hash_or_label(&self.signer_authorization_reference);
        if !signer_authorization_reference_ready {
            violation_codes.push(
                "WEB3_BROADCAST_READINESS_SIGNER_AUTHORIZATION_REFERENCE_REQUIRED".to_owned(),
            );
        }

        let live_adapter_reference_ready =
            payload_reference_is_hash_or_label(&self.live_adapter_reference);
        if !live_adapter_reference_ready {
            violation_codes
                .push("WEB3_BROADCAST_READINESS_LIVE_ADAPTER_REFERENCE_REQUIRED".to_owned());
        }

        let operator_approval_reference_ready =
            payload_reference_is_hash_or_label(&self.operator_approval_reference);
        if !operator_approval_reference_ready {
            violation_codes
                .push("WEB3_BROADCAST_READINESS_OPERATOR_APPROVAL_REFERENCE_REQUIRED".to_owned());
        }

        if self.broadcast_allowed
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_BROADCAST_READINESS_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3BroadcastReadinessStatus::ReadyForExternalReview
        } else {
            Web3BroadcastReadinessStatus::Blocked
        };

        Web3BroadcastReadinessReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            unsigned_payload_review_id: self.unsigned_payload_review.id.clone(),
            pre_sign_safety_review_id: self.pre_sign_safety_review.id.clone(),
            simulation_request_id: self.unsigned_payload_review.simulation_request_id.clone(),
            chain: self.unsigned_payload_review.chain.clone(),
            venue: self.unsigned_payload_review.venue.clone(),
            status,
            unsigned_payload_ready,
            pre_sign_safety_ready,
            signer_authorization_reference_ready,
            live_adapter_reference_ready,
            operator_approval_reference_ready,
            broadcast_allowed: false,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: self.reviewed_at_unix_ms,
        }
    }
}

impl Web3UnsignedTransactionConstructionReport {
    /// Validate this construction remains local, unsigned, and non-broadcastable.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_TX_FRAMEWORK_VERSION_MISMATCH",
                "unsigned transaction construction framework version mismatch",
            ));
        }
        validate_id(
            "unsigned transaction construction",
            &self.id,
            &mut violations,
        );
        validate_id(
            "unsigned transaction broadcast readiness",
            &self.broadcast_readiness_review_id,
            &mut violations,
        );
        validate_id(
            "unsigned transaction simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_UNSIGNED_TX_CHAIN_REQUIRED",
            "unsigned transaction chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if !payload_reference_is_hash_or_label(&self.unsigned_transaction_reference) {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_TX_REFERENCE_INVALID",
                "unsigned transaction reference must be a sanitized hash or label",
            ));
        }
        if !payload_reference_is_hash_or_label(&self.encoded_argument_digest) {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_TX_ARGUMENT_DIGEST_INVALID",
                "encoded argument digest must be a sanitized hash or label",
            ));
        }
        if self.raw_calldata_embedded
            || self.raw_transaction_serialized
            || self.broadcast_allowed
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_TX_SIDE_EFFECT_FLAG",
                "unsigned transaction construction must not embed raw calldata, serialize raw transactions, allow broadcast, call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.nonce_ready != self.construction_nonce.is_some() {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_TX_NONCE_READINESS_MISMATCH",
                "unsigned transaction nonce readiness must match construction nonce presence",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_UNSIGNED_TX_VIOLATION_COUNT_MISMATCH",
                "unsigned transaction construction violation count mismatch",
            ));
        }
        match self.status {
            Web3UnsignedTransactionConstructionStatus::ConstructedForLocalReview => {
                if self.violation_count != 0
                    || !self.broadcast_readiness_ready
                    || !self.payload_reference_ready
                    || !self.target_selector_ready
                    || !self.nonce_ready
                    || !self.gas_metadata_ready
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_UNSIGNED_TX_READY_REQUIRES_ALL_CONTROLS",
                        "constructed unsigned transaction metadata requires broadcast readiness, payload, target, nonce, and gas controls",
                    ));
                }
            }
            Web3UnsignedTransactionConstructionStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_UNSIGNED_TX_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked unsigned transaction construction requires violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3UnsignedTransactionConstructionRequest {
    /// Construct local unsigned transaction metadata without calldata, signing, or broadcast.
    #[must_use]
    pub fn construct(&self) -> Web3UnsignedTransactionConstructionReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_UNSIGNED_TX_ID_REQUIRED".to_owned());
        }
        if self.constructed_at_unix_ms == 0 {
            violation_codes.push("WEB3_UNSIGNED_TX_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.broadcast_readiness_review.validate().is_err() {
            violation_codes.push("WEB3_UNSIGNED_TX_BROADCAST_READINESS_INVALID".to_owned());
        }

        let broadcast_readiness_ready = self.broadcast_readiness_review.status
            == Web3BroadcastReadinessStatus::ReadyForExternalReview
            && self.broadcast_readiness_review.unsigned_payload_ready
            && self.broadcast_readiness_review.pre_sign_safety_ready
            && self
                .broadcast_readiness_review
                .signer_authorization_reference_ready
            && self.broadcast_readiness_review.live_adapter_reference_ready
            && self
                .broadcast_readiness_review
                .operator_approval_reference_ready
            && !self.broadcast_readiness_review.broadcast_allowed
            && !self.broadcast_readiness_review.rpc_called
            && !self.broadcast_readiness_review.signer_material_loaded
            && !self.broadcast_readiness_review.signing_performed
            && !self.broadcast_readiness_review.broadcast_performed
            && !self.broadcast_readiness_review.live_execution_performed
            && !self.broadcast_readiness_review.production_ready;
        if !broadcast_readiness_ready {
            violation_codes.push("WEB3_UNSIGNED_TX_BROADCAST_READINESS_NOT_READY".to_owned());
        }

        let payload_reference_ready = payload_reference_is_hash_or_label(&self.payload_hash);
        if !payload_reference_ready {
            violation_codes.push("WEB3_UNSIGNED_TX_PAYLOAD_REFERENCE_INVALID".to_owned());
        }

        let target_selector_ready = payload_reference_is_hash_or_label(&self.target_contract_label)
            && payload_reference_is_hash_or_label(&self.function_selector)
            && payload_reference_is_hash_or_label(&self.encoded_argument_digest);
        if !target_selector_ready {
            violation_codes.push("WEB3_UNSIGNED_TX_TARGET_SELECTOR_INVALID".to_owned());
        }

        let nonce_ready = self.nonce.is_some();
        if !nonce_ready {
            violation_codes.push("WEB3_UNSIGNED_TX_NONCE_REQUIRED".to_owned());
        }

        let gas_metadata_ready = self.gas_limit > 0 && is_non_negative_finite(self.max_fee_quote);
        if !gas_metadata_ready {
            violation_codes.push("WEB3_UNSIGNED_TX_GAS_METADATA_INVALID".to_owned());
        }

        if self.raw_calldata_embedded
            || self.raw_transaction_serialized
            || self.broadcast_allowed
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_UNSIGNED_TX_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3UnsignedTransactionConstructionStatus::ConstructedForLocalReview
        } else {
            Web3UnsignedTransactionConstructionStatus::Blocked
        };

        let encoded_argument_digest =
            if payload_reference_is_hash_or_label(&self.encoded_argument_digest) {
                self.encoded_argument_digest.clone()
            } else {
                "invalid-encoded-argument-digest-redacted".to_owned()
            };

        Web3UnsignedTransactionConstructionReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            broadcast_readiness_review_id: self.broadcast_readiness_review.id.clone(),
            simulation_request_id: self
                .broadcast_readiness_review
                .simulation_request_id
                .clone(),
            chain: self.broadcast_readiness_review.chain.clone(),
            venue: self.broadcast_readiness_review.venue.clone(),
            status,
            broadcast_readiness_ready,
            payload_reference_ready,
            target_selector_ready,
            nonce_ready,
            construction_nonce: self.nonce,
            gas_metadata_ready,
            unsigned_transaction_reference: format!("unsigned-tx-local-ref-{}", self.id),
            encoded_argument_digest,
            raw_calldata_embedded: false,
            raw_transaction_serialized: false,
            broadcast_allowed: false,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            constructed_at_unix_ms: self.constructed_at_unix_ms,
        }
    }
}

impl Web3ProviderNonceReconciliationReport {
    /// Validate this reconciliation remains local, coherent, and side-effect free.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_PROVIDER_NONCE_FRAMEWORK_VERSION_MISMATCH",
                "provider nonce reconciliation framework version mismatch",
            ));
        }
        validate_id("provider nonce reconciliation", &self.id, &mut violations);
        validate_id(
            "provider nonce unsigned transaction construction",
            &self.unsigned_transaction_construction_id,
            &mut violations,
        );
        validate_id(
            "provider nonce simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_PROVIDER_NONCE_CHAIN_REQUIRED",
            "provider nonce chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_PROVIDER_NONCE_SIDE_EFFECT_FLAG",
                "provider nonce reconciliation must not call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_PROVIDER_NONCE_VIOLATION_COUNT_MISMATCH",
                "provider nonce reconciliation violation count mismatch",
            ));
        }
        match self.status {
            Web3ProviderNonceReconciliationStatus::ReconciledForLocalReview => {
                if self.violation_count != 0
                    || !self.unsigned_transaction_ready
                    || !self.provider_snapshot_reference_ready
                    || !self.provider_next_nonce_ready
                    || !self.construction_nonce_matches_provider
                    || !self.construction_nonce_not_pending
                    || !self.pending_nonce_set_unique
                    || !self.snapshot_fresh
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_PROVIDER_NONCE_READY_REQUIRES_ALL_CONTROLS",
                        "reconciled provider nonce reports require construction, provider next nonce, pending nonce, and freshness controls",
                    ));
                }
            }
            Web3ProviderNonceReconciliationStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_PROVIDER_NONCE_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked provider nonce reconciliation requires violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3ProviderNonceReconciliationRequest {
    /// Reconcile caller-supplied provider nonce metadata without querying a provider.
    #[must_use]
    pub fn reconcile(&self) -> Web3ProviderNonceReconciliationReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_PROVIDER_NONCE_ID_REQUIRED".to_owned());
        }
        if self.reconciled_at_unix_ms == 0 {
            violation_codes.push("WEB3_PROVIDER_NONCE_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.unsigned_transaction_construction.validate().is_err() {
            violation_codes.push("WEB3_PROVIDER_NONCE_UNSIGNED_TX_INVALID".to_owned());
        }

        let unsigned_transaction_ready = self.unsigned_transaction_construction.status
            == Web3UnsignedTransactionConstructionStatus::ConstructedForLocalReview
            && self
                .unsigned_transaction_construction
                .broadcast_readiness_ready
            && self.unsigned_transaction_construction.nonce_ready
            && !self.unsigned_transaction_construction.raw_calldata_embedded
            && !self
                .unsigned_transaction_construction
                .raw_transaction_serialized
            && !self.unsigned_transaction_construction.broadcast_allowed
            && !self.unsigned_transaction_construction.rpc_called
            && !self
                .unsigned_transaction_construction
                .signer_material_loaded
            && !self.unsigned_transaction_construction.signing_performed
            && !self.unsigned_transaction_construction.broadcast_performed
            && !self
                .unsigned_transaction_construction
                .live_execution_performed
            && !self.unsigned_transaction_construction.production_ready;
        if !unsigned_transaction_ready {
            violation_codes.push("WEB3_PROVIDER_NONCE_UNSIGNED_TX_NOT_READY".to_owned());
        }

        let provider_snapshot_reference_ready =
            payload_reference_is_hash_or_label(&self.provider_snapshot_reference);
        if !provider_snapshot_reference_ready {
            violation_codes.push("WEB3_PROVIDER_NONCE_SNAPSHOT_REFERENCE_INVALID".to_owned());
        }

        let provider_next_nonce_ready = self.provider_next_nonce.is_some();
        if !provider_next_nonce_ready {
            violation_codes.push("WEB3_PROVIDER_NONCE_NEXT_NONCE_REQUIRED".to_owned());
        }

        let construction_nonce = self.unsigned_transaction_construction.construction_nonce;
        let construction_nonce_matches_provider =
            construction_nonce.is_some() && construction_nonce == self.provider_next_nonce;
        if !construction_nonce_matches_provider {
            violation_codes.push("WEB3_PROVIDER_NONCE_CONSTRUCTION_NONCE_MISMATCH".to_owned());
        }

        let construction_nonce_not_pending =
            construction_nonce.is_some_and(|nonce| !self.provider_pending_nonces.contains(&nonce));
        if !construction_nonce_not_pending {
            violation_codes.push("WEB3_PROVIDER_NONCE_ALREADY_PENDING".to_owned());
        }

        let pending_nonce_set_unique = !has_duplicate_nonce(&self.provider_pending_nonces);
        if !pending_nonce_set_unique {
            violation_codes.push("WEB3_PROVIDER_NONCE_PENDING_DUPLICATE".to_owned());
        }

        let snapshot_fresh =
            self.max_snapshot_age_ms > 0 && self.snapshot_age_ms <= self.max_snapshot_age_ms;
        if !snapshot_fresh {
            violation_codes.push("WEB3_PROVIDER_NONCE_SNAPSHOT_STALE".to_owned());
        }

        if self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_PROVIDER_NONCE_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3ProviderNonceReconciliationStatus::ReconciledForLocalReview
        } else {
            Web3ProviderNonceReconciliationStatus::Blocked
        };

        Web3ProviderNonceReconciliationReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            unsigned_transaction_construction_id: self.unsigned_transaction_construction.id.clone(),
            simulation_request_id: self
                .unsigned_transaction_construction
                .simulation_request_id
                .clone(),
            chain: self.unsigned_transaction_construction.chain.clone(),
            venue: self.unsigned_transaction_construction.venue.clone(),
            status,
            unsigned_transaction_ready,
            provider_snapshot_reference_ready,
            provider_next_nonce_ready,
            construction_nonce_matches_provider,
            construction_nonce_not_pending,
            pending_nonce_set_unique,
            snapshot_fresh,
            construction_nonce,
            provider_next_nonce: self.provider_next_nonce,
            provider_pending_nonce_count: u64::try_from(self.provider_pending_nonces.len())
                .unwrap_or(u64::MAX),
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reconciled_at_unix_ms: self.reconciled_at_unix_ms,
        }
    }
}

impl Web3RawTransactionSerializationReviewReport {
    /// Validate this serialization review remains metadata-only and side-effect free.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_RAW_TX_SERIALIZATION_FRAMEWORK_VERSION_MISMATCH",
                "raw transaction serialization review framework version mismatch",
            ));
        }
        validate_id(
            "raw transaction serialization review",
            &self.id,
            &mut violations,
        );
        validate_id(
            "raw transaction provider nonce reconciliation",
            &self.provider_nonce_reconciliation_id,
            &mut violations,
        );
        validate_id(
            "raw transaction unsigned transaction construction",
            &self.unsigned_transaction_construction_id,
            &mut violations,
        );
        validate_id(
            "raw transaction simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_RAW_TX_SERIALIZATION_CHAIN_REQUIRED",
            "raw transaction serialization chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if self.raw_transaction_bytes_embedded
            || self.raw_calldata_embedded
            || self.raw_transaction_serialized
            || self.broadcast_allowed
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_RAW_TX_SERIALIZATION_SIDE_EFFECT_FLAG",
                "raw transaction serialization review must not embed raw bytes, serialize, permit broadcast, call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_RAW_TX_SERIALIZATION_VIOLATION_COUNT_MISMATCH",
                "raw transaction serialization review violation count mismatch",
            ));
        }
        match self.status {
            Web3RawTransactionSerializationReviewStatus::ReadyForExternalReview => {
                if self.violation_count != 0
                    || !self.provider_nonce_reconciliation_ready
                    || !self.transaction_type_ready
                    || !self.chain_id_ready
                    || !self.fee_fields_ready
                    || !self.access_list_reference_ready
                    || self.unsigned_transaction_reference.trim().is_empty()
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_RAW_TX_SERIALIZATION_READY_REQUIRES_ALL_CONTROLS",
                        "ready raw transaction serialization reviews require prerequisite, chain, type, fee, access-list, and reference controls",
                    ));
                }
            }
            Web3RawTransactionSerializationReviewStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_RAW_TX_SERIALIZATION_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked raw transaction serialization review requires violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3RawTransactionSerializationReviewRequest {
    /// Review serialization readiness metadata without serializing raw transactions.
    #[must_use]
    pub fn review(&self) -> Web3RawTransactionSerializationReviewReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_ID_REQUIRED".to_owned());
        }
        if self.reviewed_at_unix_ms == 0 {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.provider_nonce_reconciliation.validate().is_err() {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_PROVIDER_NONCE_INVALID".to_owned());
        }

        let provider_nonce_reconciliation_ready = self.provider_nonce_reconciliation.status
            == Web3ProviderNonceReconciliationStatus::ReconciledForLocalReview
            && self
                .provider_nonce_reconciliation
                .unsigned_transaction_ready
            && self
                .provider_nonce_reconciliation
                .provider_snapshot_reference_ready
            && self.provider_nonce_reconciliation.provider_next_nonce_ready
            && self
                .provider_nonce_reconciliation
                .construction_nonce_matches_provider
            && self
                .provider_nonce_reconciliation
                .construction_nonce_not_pending
            && self.provider_nonce_reconciliation.pending_nonce_set_unique
            && self.provider_nonce_reconciliation.snapshot_fresh
            && !self.provider_nonce_reconciliation.rpc_called
            && !self.provider_nonce_reconciliation.signer_material_loaded
            && !self.provider_nonce_reconciliation.signing_performed
            && !self.provider_nonce_reconciliation.broadcast_performed
            && !self.provider_nonce_reconciliation.live_execution_performed
            && !self.provider_nonce_reconciliation.production_ready;
        if !provider_nonce_reconciliation_ready {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_PROVIDER_NONCE_NOT_READY".to_owned());
        }

        let transaction_type_ready =
            payload_reference_is_hash_or_label(&self.transaction_type_label);
        if !transaction_type_ready {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_TYPE_INVALID".to_owned());
        }

        let chain_id_ready = payload_reference_is_hash_or_label(&self.chain_id_reference);
        if !chain_id_ready {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_CHAIN_ID_INVALID".to_owned());
        }

        let fee_fields_ready = payload_reference_is_hash_or_label(&self.fee_field_reference);
        if !fee_fields_ready {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_FEE_FIELDS_INVALID".to_owned());
        }

        let access_list_reference_ready =
            payload_reference_is_hash_or_label(&self.access_list_reference);
        if !access_list_reference_ready {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_ACCESS_LIST_INVALID".to_owned());
        }

        if self.raw_transaction_bytes_embedded
            || self.raw_calldata_embedded
            || self.raw_transaction_serialized
            || self.broadcast_allowed
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_RAW_TX_SERIALIZATION_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3RawTransactionSerializationReviewStatus::ReadyForExternalReview
        } else {
            Web3RawTransactionSerializationReviewStatus::Blocked
        };

        Web3RawTransactionSerializationReviewReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            provider_nonce_reconciliation_id: self.provider_nonce_reconciliation.id.clone(),
            unsigned_transaction_construction_id: self
                .provider_nonce_reconciliation
                .unsigned_transaction_construction_id
                .clone(),
            simulation_request_id: self
                .provider_nonce_reconciliation
                .simulation_request_id
                .clone(),
            chain: self.provider_nonce_reconciliation.chain.clone(),
            venue: self.provider_nonce_reconciliation.venue.clone(),
            status,
            provider_nonce_reconciliation_ready,
            transaction_type_ready,
            chain_id_ready,
            fee_fields_ready,
            access_list_reference_ready,
            unsigned_transaction_reference: format!(
                "unsigned-tx-reference:{}",
                self.provider_nonce_reconciliation
                    .unsigned_transaction_construction_id
            ),
            raw_transaction_bytes_embedded: false,
            raw_calldata_embedded: false,
            raw_transaction_serialized: false,
            broadcast_allowed: false,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: self.reviewed_at_unix_ms,
        }
    }
}

impl Web3BroadcastAdapterControlReviewReport {
    /// Validate this broadcast adapter control review remains local and non-broadcasting.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_BROADCAST_ADAPTER_FRAMEWORK_VERSION_MISMATCH",
                "broadcast adapter control review framework version mismatch",
            ));
        }
        validate_id(
            "broadcast adapter control review",
            &self.id,
            &mut violations,
        );
        validate_id(
            "broadcast adapter raw transaction serialization review",
            &self.raw_transaction_serialization_review_id,
            &mut violations,
        );
        validate_id(
            "broadcast adapter provider nonce reconciliation",
            &self.provider_nonce_reconciliation_id,
            &mut violations,
        );
        validate_id(
            "broadcast adapter unsigned transaction construction",
            &self.unsigned_transaction_construction_id,
            &mut violations,
        );
        validate_id(
            "broadcast adapter simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_BROADCAST_ADAPTER_CHAIN_REQUIRED",
            "broadcast adapter chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if self.broadcast_permission_granted
            || self.raw_transaction_bytes_embedded
            || self.raw_transaction_serialized
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_BROADCAST_ADAPTER_SIDE_EFFECT_FLAG",
                "broadcast adapter control review must not grant broadcast, embed raw bytes, serialize, call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_BROADCAST_ADAPTER_VIOLATION_COUNT_MISMATCH",
                "broadcast adapter control review violation count mismatch",
            ));
        }
        match self.status {
            Web3BroadcastAdapterControlReviewStatus::ReadyForExternalReview => {
                if self.violation_count != 0
                    || !self.raw_transaction_serialization_review_ready
                    || !self.adapter_reference_ready
                    || !self.operator_approval_reference_ready
                    || !self.audit_state_preflight_reference_ready
                    || !self.kill_switch_confirmed
                    || !self.rate_limit_control_ready
                    || !self.replay_protection_ready
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_BROADCAST_ADAPTER_READY_REQUIRES_ALL_CONTROLS",
                        "ready broadcast adapter control reviews require prerequisite, adapter, approval, audit/state, kill-switch, rate-limit, and replay controls",
                    ));
                }
            }
            Web3BroadcastAdapterControlReviewStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_BROADCAST_ADAPTER_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked broadcast adapter control review requires violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3BroadcastAdapterControlReviewRequest {
    /// Review future broadcast adapter controls without broadcasting.
    #[must_use]
    pub fn review(&self) -> Web3BroadcastAdapterControlReviewReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_ID_REQUIRED".to_owned());
        }
        if self.reviewed_at_unix_ms == 0 {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_TIMESTAMP_REQUIRED".to_owned());
        }
        if self
            .raw_transaction_serialization_review
            .validate()
            .is_err()
        {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_RAW_TX_REVIEW_INVALID".to_owned());
        }

        let raw_transaction_serialization_review_ready =
            self.raw_transaction_serialization_review.status
                == Web3RawTransactionSerializationReviewStatus::ReadyForExternalReview
                && self
                    .raw_transaction_serialization_review
                    .provider_nonce_reconciliation_ready
                && self
                    .raw_transaction_serialization_review
                    .transaction_type_ready
                && self.raw_transaction_serialization_review.chain_id_ready
                && self.raw_transaction_serialization_review.fee_fields_ready
                && self
                    .raw_transaction_serialization_review
                    .access_list_reference_ready
                && !self
                    .raw_transaction_serialization_review
                    .raw_transaction_bytes_embedded
                && !self
                    .raw_transaction_serialization_review
                    .raw_calldata_embedded
                && !self
                    .raw_transaction_serialization_review
                    .raw_transaction_serialized
                && !self.raw_transaction_serialization_review.broadcast_allowed
                && !self.raw_transaction_serialization_review.rpc_called
                && !self
                    .raw_transaction_serialization_review
                    .signer_material_loaded
                && !self.raw_transaction_serialization_review.signing_performed
                && !self
                    .raw_transaction_serialization_review
                    .broadcast_performed
                && !self
                    .raw_transaction_serialization_review
                    .live_execution_performed
                && !self.raw_transaction_serialization_review.production_ready;
        if !raw_transaction_serialization_review_ready {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_RAW_TX_REVIEW_NOT_READY".to_owned());
        }

        let adapter_reference_ready = payload_reference_is_hash_or_label(&self.adapter_reference);
        if !adapter_reference_ready {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_REFERENCE_INVALID".to_owned());
        }

        let operator_approval_reference_ready =
            payload_reference_is_hash_or_label(&self.operator_approval_reference);
        if !operator_approval_reference_ready {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_OPERATOR_APPROVAL_INVALID".to_owned());
        }

        let audit_state_preflight_reference_ready =
            payload_reference_is_hash_or_label(&self.audit_state_preflight_reference);
        if !audit_state_preflight_reference_ready {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_AUDIT_STATE_PREFLIGHT_INVALID".to_owned());
        }

        if !self.kill_switch_confirmed {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_KILL_SWITCH_REQUIRED".to_owned());
        }
        if !self.rate_limit_control_ready {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_RATE_LIMIT_REQUIRED".to_owned());
        }
        if !self.replay_protection_ready {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_REPLAY_PROTECTION_REQUIRED".to_owned());
        }

        if self.broadcast_permission_granted
            || self.raw_transaction_bytes_embedded
            || self.raw_transaction_serialized
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_BROADCAST_ADAPTER_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3BroadcastAdapterControlReviewStatus::ReadyForExternalReview
        } else {
            Web3BroadcastAdapterControlReviewStatus::Blocked
        };

        Web3BroadcastAdapterControlReviewReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            raw_transaction_serialization_review_id: self
                .raw_transaction_serialization_review
                .id
                .clone(),
            provider_nonce_reconciliation_id: self
                .raw_transaction_serialization_review
                .provider_nonce_reconciliation_id
                .clone(),
            unsigned_transaction_construction_id: self
                .raw_transaction_serialization_review
                .unsigned_transaction_construction_id
                .clone(),
            simulation_request_id: self
                .raw_transaction_serialization_review
                .simulation_request_id
                .clone(),
            chain: self.raw_transaction_serialization_review.chain.clone(),
            venue: self.raw_transaction_serialization_review.venue.clone(),
            status,
            raw_transaction_serialization_review_ready,
            adapter_reference_ready,
            operator_approval_reference_ready,
            audit_state_preflight_reference_ready,
            kill_switch_confirmed: self.kill_switch_confirmed,
            rate_limit_control_ready: self.rate_limit_control_ready,
            replay_protection_ready: self.replay_protection_ready,
            broadcast_permission_granted: false,
            raw_transaction_bytes_embedded: false,
            raw_transaction_serialized: false,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: self.reviewed_at_unix_ms,
        }
    }
}

impl Web3SandboxLiveDiscrepancyCalibrationReport {
    /// Validate this calibration remains local, reference-only, and side-effect free.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        if self.framework_version != DEX_CONNECTOR_FRAMEWORK_VERSION {
            violations.push(DexConnectorViolation::new(
                "WEB3_SANDBOX_LIVE_CALIBRATION_FRAMEWORK_VERSION_MISMATCH",
                "sandbox/live discrepancy calibration framework version mismatch",
            ));
        }
        validate_id(
            "sandbox/live discrepancy calibration",
            &self.id,
            &mut violations,
        );
        validate_id(
            "sandbox/live broadcast adapter control review",
            &self.broadcast_adapter_control_review_id,
            &mut violations,
        );
        validate_id(
            "sandbox/live raw transaction serialization review",
            &self.raw_transaction_serialization_review_id,
            &mut violations,
        );
        validate_id(
            "sandbox/live provider nonce reconciliation",
            &self.provider_nonce_reconciliation_id,
            &mut violations,
        );
        validate_id(
            "sandbox/live simulation request",
            &self.simulation_request_id,
            &mut violations,
        );
        validate_non_empty(
            "WEB3_SANDBOX_LIVE_CALIBRATION_CHAIN_REQUIRED",
            "sandbox/live calibration chain",
            &self.chain,
            &mut violations,
        );
        validate_dex_venue_ref(&self.venue, &mut violations);
        if self.external_call_performed
            || self.credential_loaded
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violations.push(DexConnectorViolation::new(
                "WEB3_SANDBOX_LIVE_CALIBRATION_SIDE_EFFECT_FLAG",
                "sandbox/live discrepancy calibration must not call external systems, load credentials, call RPC, load signer material, sign, broadcast, execute live, or claim readiness",
            ));
        }
        if self.violation_count != u64::try_from(self.violation_codes.len()).unwrap_or(u64::MAX) {
            violations.push(DexConnectorViolation::new(
                "WEB3_SANDBOX_LIVE_CALIBRATION_VIOLATION_COUNT_MISMATCH",
                "sandbox/live discrepancy calibration violation count mismatch",
            ));
        }
        match self.status {
            Web3SandboxLiveDiscrepancyCalibrationStatus::CalibratedForLocalReview => {
                if self.violation_count != 0
                    || !self.broadcast_adapter_control_ready
                    || !self.sandbox_observation_reference_ready
                    || !self.live_observation_reference_ready
                    || !self.sample_size_ready
                    || !self.price_deviation_within_limit
                    || !self.latency_deviation_within_limit
                    || !self.fee_deviation_within_limit
                {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_SANDBOX_LIVE_CALIBRATION_READY_REQUIRES_ALL_CONTROLS",
                        "calibrated sandbox/live reports require prerequisite, reference, sample, and discrepancy controls",
                    ));
                }
            }
            Web3SandboxLiveDiscrepancyCalibrationStatus::Blocked => {
                if self.violation_count == 0 {
                    violations.push(DexConnectorViolation::new(
                        "WEB3_SANDBOX_LIVE_CALIBRATION_BLOCKED_REQUIRES_VIOLATIONS",
                        "blocked sandbox/live discrepancy calibration requires violation codes",
                    ));
                }
            }
        }
        finish_validation(violations)
    }
}

impl Web3SandboxLiveDiscrepancyCalibrationRequest {
    /// Calibrate caller-supplied sandbox/live metadata without external calls.
    #[must_use]
    pub fn calibrate(&self) -> Web3SandboxLiveDiscrepancyCalibrationReport {
        let mut violation_codes = Vec::new();

        if self.id.trim().is_empty() {
            violation_codes.push("WEB3_SANDBOX_LIVE_CALIBRATION_ID_REQUIRED".to_owned());
        }
        if self.calibrated_at_unix_ms == 0 {
            violation_codes.push("WEB3_SANDBOX_LIVE_CALIBRATION_TIMESTAMP_REQUIRED".to_owned());
        }
        if self.broadcast_adapter_control_review.validate().is_err() {
            violation_codes
                .push("WEB3_SANDBOX_LIVE_CALIBRATION_BROADCAST_REVIEW_INVALID".to_owned());
        }

        let broadcast_adapter_control_ready = self.broadcast_adapter_control_review.status
            == Web3BroadcastAdapterControlReviewStatus::ReadyForExternalReview
            && self
                .broadcast_adapter_control_review
                .raw_transaction_serialization_review_ready
            && self
                .broadcast_adapter_control_review
                .adapter_reference_ready
            && self
                .broadcast_adapter_control_review
                .operator_approval_reference_ready
            && self
                .broadcast_adapter_control_review
                .audit_state_preflight_reference_ready
            && self.broadcast_adapter_control_review.kill_switch_confirmed
            && self
                .broadcast_adapter_control_review
                .rate_limit_control_ready
            && self
                .broadcast_adapter_control_review
                .replay_protection_ready
            && !self
                .broadcast_adapter_control_review
                .broadcast_permission_granted
            && !self
                .broadcast_adapter_control_review
                .raw_transaction_bytes_embedded
            && !self
                .broadcast_adapter_control_review
                .raw_transaction_serialized
            && !self.broadcast_adapter_control_review.rpc_called
            && !self.broadcast_adapter_control_review.signer_material_loaded
            && !self.broadcast_adapter_control_review.signing_performed
            && !self.broadcast_adapter_control_review.broadcast_performed
            && !self
                .broadcast_adapter_control_review
                .live_execution_performed
            && !self.broadcast_adapter_control_review.production_ready;
        if !broadcast_adapter_control_ready {
            violation_codes
                .push("WEB3_SANDBOX_LIVE_CALIBRATION_BROADCAST_REVIEW_NOT_READY".to_owned());
        }

        let sandbox_observation_reference_ready =
            payload_reference_is_hash_or_label(&self.sandbox_observation_reference);
        if !sandbox_observation_reference_ready {
            violation_codes
                .push("WEB3_SANDBOX_LIVE_CALIBRATION_SANDBOX_REFERENCE_INVALID".to_owned());
        }

        let live_observation_reference_ready =
            payload_reference_is_hash_or_label(&self.live_observation_reference);
        if !live_observation_reference_ready {
            violation_codes.push("WEB3_SANDBOX_LIVE_CALIBRATION_LIVE_REFERENCE_INVALID".to_owned());
        }

        let sample_size_ready = self.minimum_sample_count > 0
            && self.sandbox_sample_count >= self.minimum_sample_count
            && self.live_sample_count >= self.minimum_sample_count;
        if !sample_size_ready {
            violation_codes
                .push("WEB3_SANDBOX_LIVE_CALIBRATION_SAMPLE_COUNT_INSUFFICIENT".to_owned());
        }

        let price_deviation_within_limit = is_non_negative_finite(self.max_price_deviation_bps)
            && is_non_negative_finite(self.observed_price_deviation_bps)
            && self.observed_price_deviation_bps <= self.max_price_deviation_bps;
        if !price_deviation_within_limit {
            violation_codes
                .push("WEB3_SANDBOX_LIVE_CALIBRATION_PRICE_DEVIATION_EXCEEDED".to_owned());
        }

        let latency_deviation_within_limit =
            self.observed_latency_deviation_ms <= self.max_latency_deviation_ms;
        if !latency_deviation_within_limit {
            violation_codes
                .push("WEB3_SANDBOX_LIVE_CALIBRATION_LATENCY_DEVIATION_EXCEEDED".to_owned());
        }

        let fee_deviation_within_limit = is_non_negative_finite(self.max_fee_deviation_quote)
            && is_non_negative_finite(self.observed_fee_deviation_quote)
            && self.observed_fee_deviation_quote <= self.max_fee_deviation_quote;
        if !fee_deviation_within_limit {
            violation_codes.push("WEB3_SANDBOX_LIVE_CALIBRATION_FEE_DEVIATION_EXCEEDED".to_owned());
        }

        if self.external_call_performed
            || self.credential_loaded
            || self.rpc_called
            || self.signer_material_loaded
            || self.signing_performed
            || self.broadcast_performed
            || self.live_execution_performed
            || self.production_ready
        {
            violation_codes.push("WEB3_SANDBOX_LIVE_CALIBRATION_SIDE_EFFECT_FLAG".to_owned());
        }

        let status = if violation_codes.is_empty() {
            Web3SandboxLiveDiscrepancyCalibrationStatus::CalibratedForLocalReview
        } else {
            Web3SandboxLiveDiscrepancyCalibrationStatus::Blocked
        };

        Web3SandboxLiveDiscrepancyCalibrationReport {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            id: self.id.clone(),
            broadcast_adapter_control_review_id: self.broadcast_adapter_control_review.id.clone(),
            raw_transaction_serialization_review_id: self
                .broadcast_adapter_control_review
                .raw_transaction_serialization_review_id
                .clone(),
            provider_nonce_reconciliation_id: self
                .broadcast_adapter_control_review
                .provider_nonce_reconciliation_id
                .clone(),
            simulation_request_id: self
                .broadcast_adapter_control_review
                .simulation_request_id
                .clone(),
            chain: self.broadcast_adapter_control_review.chain.clone(),
            venue: self.broadcast_adapter_control_review.venue.clone(),
            status,
            broadcast_adapter_control_ready,
            sandbox_observation_reference_ready,
            live_observation_reference_ready,
            sample_size_ready,
            price_deviation_within_limit,
            latency_deviation_within_limit,
            fee_deviation_within_limit,
            sandbox_sample_count: self.sandbox_sample_count,
            live_sample_count: self.live_sample_count,
            violation_count: u64::try_from(violation_codes.len()).unwrap_or(u64::MAX),
            violation_codes,
            external_call_performed: false,
            credential_loaded: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            calibrated_at_unix_ms: self.calibrated_at_unix_ms,
        }
    }
}

/// Framework-level policy gate for DEX/Web3 swap quote requests.
///
/// Phase 8 validates paper/simulation DEX intent boundaries only. Live swaps,
/// wallet signing, transaction broadcast, bridge routing, and real RPC calls
/// remain unavailable until later phases implement custody, audit/state,
/// runtime execution, external validation, and explicit operator controls.
#[derive(Debug, Clone, PartialEq)]
pub struct DexPolicyGate {
    policy: PolicyEngine,
}

impl DexPolicyGate {
    /// Build a DEX policy gate from the current policy engine.
    #[must_use]
    pub fn new(policy: PolicyEngine) -> Self {
        Self { policy }
    }

    /// Validate a DEX swap quote request against router capabilities and policy.
    pub fn validate_swap_quote(
        &self,
        profile: &DexRouterProfile,
        request: &DexSwapQuoteRequest,
    ) -> Result<PolicyApproval, DexConnectorError> {
        request.validate()?;
        profile.validate()?;
        validate_profile_matches_request(profile, request)?;

        if request.scope == ExecutionScope::Live {
            return Err(DexConnectorError::LiveSwapsUnavailable);
        }

        if request.scope == ExecutionScope::Observe {
            return Err(DexConnectorError::ObserveSwapsUnavailable);
        }

        validate_quote_capabilities(profile, request)?;

        match self.policy.evaluate(&request.to_execution_intent()?) {
            PolicyDecision::Approved { approval } => Ok(approval),
            PolicyDecision::Denied { violations } => {
                Err(DexConnectorError::PolicyDenied { violations })
            }
        }
    }

    /// Locally validate a transaction simulation request without RPC or signing.
    pub fn validate_simulation_request(
        &self,
        profile: &DexRouterProfile,
        request: &Web3TransactionSimulationRequest,
    ) -> Result<Web3TransactionSimulationResponse, DexConnectorError> {
        request.validate()?;
        profile.validate()?;
        validate_profile_matches_simulation(profile, request)?;

        if request.scope == ExecutionScope::Live {
            return Err(DexConnectorError::LiveSimulationUnavailable);
        }

        if !profile.capabilities.transaction_simulation {
            return Err(DexConnectorError::CapabilityUnavailable {
                venue: profile.venue.name.clone(),
                capability: "transaction-simulation",
            });
        }

        let response = Web3TransactionSimulationResponse {
            id: format!("local-simulation:{}", request.id),
            request_id: request.id.clone(),
            status: DexSimulationStatus::LocallyValidated,
            gas_used: request.gas_limit,
            gas_fee_quote: request.max_gas_fee_quote,
            amount_out: request.minimum_amount_out,
            diagnostic: Some(
                "locally validated only; no RPC, signing, or broadcast performed".to_owned(),
            ),
            broadcastable: false,
        };
        response.validate()?;
        Ok(response)
    }
}

/// DEX/Web3 connector identity boundary.
pub trait DexConnectorIdentity {
    /// Stable connector name for diagnostics and audit records.
    fn connector_name(&self) -> &str;

    /// Router profile for this connector.
    fn router_profile(&self) -> &DexRouterProfile;
}

/// Read-only DEX quote connector boundary.
///
/// Implementors may provide quotes and fee lookups in later phases, but this
/// trait does not permit signing, transaction submission, or balance mutation.
pub trait DexQuoteConnector: DexConnectorIdentity + FeeProvider {
    /// Produce a quote response for a validated request.
    fn quote_swap(
        &self,
        request: &DexSwapQuoteRequest,
    ) -> Result<DexSwapQuoteResponse, DexConnectorError>;
}

/// Deterministic local DEX/Web3 adapter for framework and replay tests.
///
/// This adapter is router-shaped but not chain-connected. It serves
/// caller-supplied local quote, fee, and simulation fixtures, validates paper
/// swap requests through policy, and never calls RPC, builds raw calldata,
/// loads signer material, signs payloads, broadcasts transactions, bridges, or
/// mutates wallet/chain state.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalDeterministicDexAdapter {
    connector_name: String,
    profile: DexRouterProfile,
    quote_response: DexSwapQuoteResponse,
    fee_schedule: FeeSchedule,
    simulation_response: Web3TransactionSimulationResponse,
    policy_gate: DexPolicyGate,
}

impl LocalDeterministicDexAdapter {
    /// Create a local deterministic DEX adapter from non-secret fixtures.
    pub fn new(
        connector_name: impl Into<String>,
        profile: DexRouterProfile,
        quote_response: DexSwapQuoteResponse,
        fee_schedule: FeeSchedule,
        simulation_response: Web3TransactionSimulationResponse,
        policy: PolicyEngine,
    ) -> Result<Self, DexConnectorError> {
        let adapter = Self {
            connector_name: connector_name.into(),
            profile,
            quote_response,
            fee_schedule,
            simulation_response,
            policy_gate: DexPolicyGate::new(policy),
        };
        adapter.validate()?;
        Ok(adapter)
    }

    /// Validate local fixture invariants.
    pub fn validate(&self) -> Result<(), DexConnectorError> {
        let mut violations = Vec::new();
        validate_id("connector", &self.connector_name, &mut violations);
        if let Err(DexConnectorError::ValidationFailed {
            violations: profile_violations,
        }) = self.profile.validate()
        {
            violations.extend(profile_violations);
        }
        if let Err(DexConnectorError::ValidationFailed {
            violations: quote_violations,
        }) = self.quote_response.validate()
        {
            violations.extend(quote_violations);
        }
        if let Err(error) = self.fee_schedule.validate() {
            violations.push(DexConnectorViolation::new_owned(
                "LOCAL_DEX_FEE_SCHEDULE_INVALID",
                error.to_string(),
            ));
        }
        if let Err(DexConnectorError::ValidationFailed {
            violations: simulation_violations,
        }) = self.simulation_response.validate()
        {
            violations.extend(simulation_violations);
        }
        validate_local_dex_fixture_matches_profile(
            &self.profile,
            &self.quote_response,
            &self.fee_schedule,
            &mut violations,
        );

        finish_validation(violations)
    }

    /// Whether this adapter performed RPC I/O. Always false.
    #[must_use]
    pub const fn rpc_called(&self) -> bool {
        false
    }

    /// Whether this adapter loaded signer material. Always false.
    #[must_use]
    pub const fn signer_material_loaded(&self) -> bool {
        false
    }

    /// Whether this adapter signed a payload. Always false.
    #[must_use]
    pub const fn signing_performed(&self) -> bool {
        false
    }

    /// Whether this adapter broadcast a transaction. Always false.
    #[must_use]
    pub const fn broadcast_performed(&self) -> bool {
        false
    }

    /// Whether this adapter executed a bridge. Always false.
    #[must_use]
    pub const fn bridge_performed(&self) -> bool {
        false
    }
}

impl DexConnectorIdentity for LocalDeterministicDexAdapter {
    fn connector_name(&self) -> &str {
        &self.connector_name
    }

    fn router_profile(&self) -> &DexRouterProfile {
        &self.profile
    }
}

impl FeeProvider for LocalDeterministicDexAdapter {
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
                reason: "local DEX fixture does not contain the requested venue".to_owned(),
            });
        }
        if let Some(pair) = pair {
            if self.fee_schedule.pair.as_ref() != Some(pair) {
                return Err(FeeModelError::ScheduleUnavailable {
                    provider: self.connector_name.clone(),
                    reason: "local DEX fixture does not contain the requested pair".to_owned(),
                });
            }
        }
        Ok(self.fee_schedule.clone())
    }
}

impl DexQuoteConnector for LocalDeterministicDexAdapter {
    fn quote_swap(
        &self,
        request: &DexSwapQuoteRequest,
    ) -> Result<DexSwapQuoteResponse, DexConnectorError> {
        self.policy_gate
            .validate_swap_quote(&self.profile, request)?;
        if self.quote_response.request_id != request.id
            || self.quote_response.pair != request.pair
            || !self
                .quote_response
                .chain
                .eq_ignore_ascii_case(&request.chain)
            || !self
                .quote_response
                .venue
                .name
                .eq_ignore_ascii_case(&request.venue.name)
        {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "LOCAL_DEX_QUOTE_FIXTURE_MISMATCH",
                    "local DEX quote fixture does not match the requested swap",
                )],
            });
        }
        Ok(self.quote_response.clone())
    }
}

/// Future transaction simulation connector boundary.
///
/// Phase 8 defines the interface only. Implementors in later phases must fail
/// closed, avoid signing, avoid broadcasting, call policy where applicable, and
/// write durable audit records before any external action.
pub trait Web3SimulationConnector: DexConnectorIdentity {
    /// Simulate an unsigned, reviewed transaction payload.
    fn simulate_transaction(
        &self,
        request: &Web3TransactionSimulationRequest,
    ) -> Result<Web3TransactionSimulationResponse, DexConnectorError>;
}

impl Web3SimulationConnector for LocalDeterministicDexAdapter {
    fn simulate_transaction(
        &self,
        request: &Web3TransactionSimulationRequest,
    ) -> Result<Web3TransactionSimulationResponse, DexConnectorError> {
        self.policy_gate
            .validate_simulation_request(&self.profile, request)?;
        if self.simulation_response.request_id != request.id {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new(
                    "LOCAL_DEX_SIMULATION_FIXTURE_MISMATCH",
                    "local DEX simulation fixture does not match the requested simulation",
                )],
            });
        }
        Ok(self.simulation_response.clone())
    }
}

fn validate_local_dex_fixture_matches_profile(
    profile: &DexRouterProfile,
    quote: &DexSwapQuoteResponse,
    fee_schedule: &FeeSchedule,
    violations: &mut Vec<DexConnectorViolation>,
) {
    if !quote.venue.name.eq_ignore_ascii_case(&profile.venue.name) {
        violations.push(DexConnectorViolation::new(
            "LOCAL_DEX_QUOTE_VENUE_MISMATCH",
            "local DEX quote venue must match adapter router profile",
        ));
    }
    if !quote.chain.eq_ignore_ascii_case(&profile.chain) {
        violations.push(DexConnectorViolation::new(
            "LOCAL_DEX_QUOTE_CHAIN_MISMATCH",
            "local DEX quote chain must match adapter router profile",
        ));
    }
    if !fee_schedule
        .venue
        .name
        .eq_ignore_ascii_case(&profile.venue.name)
    {
        violations.push(DexConnectorViolation::new(
            "LOCAL_DEX_FEE_VENUE_MISMATCH",
            "local DEX fee schedule venue must match adapter router profile",
        ));
    }
    if fee_schedule
        .pair
        .as_ref()
        .is_some_and(|pair| pair != &quote.pair)
    {
        violations.push(DexConnectorViolation::new(
            "LOCAL_DEX_FEE_PAIR_MISMATCH",
            "local DEX fee schedule pair must match quote pair when scoped",
        ));
    }
}

fn validate_dex_lifecycle_inputs(
    validation: &DexSwapValidationRecord,
    quote: &DexSwapQuoteResponse,
    simulation: &Web3TransactionSimulationResponse,
) -> Result<(), DexConnectorError> {
    quote.validate()?;
    simulation.validate()?;
    let mut violations = Vec::new();
    if quote.request_id != validation.request_id {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_QUOTE_REQUEST_MISMATCH",
            "DEX lifecycle quote response request id must match validation record",
        ));
    }
    if quote.venue != validation.venue {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_QUOTE_VENUE_MISMATCH",
            "DEX lifecycle quote venue must match validation record",
        ));
    }
    if !quote.chain.eq_ignore_ascii_case(&validation.chain) {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_QUOTE_CHAIN_MISMATCH",
            "DEX lifecycle quote chain must match validation record",
        ));
    }
    if quote.pair != validation.pair {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_QUOTE_PAIR_MISMATCH",
            "DEX lifecycle quote pair must match validation record",
        ));
    }
    if (quote.amount_in - validation.amount_in).abs() > f64::EPSILON {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_QUOTE_AMOUNT_MISMATCH",
            "DEX lifecycle quote amount must match validation record",
        ));
    }
    if simulation.request_id.trim().is_empty() {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_SIMULATION_REQUIRED",
            "DEX lifecycle simulation response id is required",
        ));
    }
    if simulation.status != DexSimulationStatus::LocallyValidated {
        violations.push(DexConnectorViolation::new(
            "DEX_LIFECYCLE_SIMULATION_NOT_LOCAL",
            "DEX lifecycle simulation must remain locally validated",
        ));
    }
    finish_validation(violations)
}

/// One DEX/Web3 framework validation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexConnectorViolation {
    code: &'static str,
    message: String,
}

impl DexConnectorViolation {
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

/// DEX/Web3 framework errors.
#[derive(Debug, Clone, PartialEq)]
pub enum DexConnectorError {
    /// Validation failed with deterministic violations.
    ValidationFailed {
        violations: Vec<DexConnectorViolation>,
    },
    /// Router profile is not registered.
    RouterNotRegistered { venue: String, chain: String },
    /// Profile and request reference different venues.
    VenueMismatch { profile: String, request: String },
    /// Profile and request reference different chains.
    ChainMismatch { profile: String, request: String },
    /// Requested feature is unsupported by the router profile.
    CapabilityUnavailable {
        venue: String,
        capability: &'static str,
    },
    /// Phase 8 live DEX swaps are unavailable.
    LiveSwapsUnavailable,
    /// Observe scope cannot validate executable DEX swaps.
    ObserveSwapsUnavailable,
    /// Phase 8 live RPC simulation is unavailable.
    LiveSimulationUnavailable,
    /// Phase 8 transaction broadcast is unavailable.
    BroadcastUnavailable,
    /// Policy denied the converted execution intent.
    PolicyDenied { violations: Vec<PolicyViolation> },
    /// Append-only audit journal persistence failed.
    AuditJournalFailed { reason: String },
    /// State-store checkpoint persistence failed.
    StateStoreFailed { reason: String },
}

impl DexConnectorError {
    /// Return validation violations, if available.
    #[must_use]
    pub fn violations(&self) -> &[DexConnectorViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::RouterNotRegistered { .. }
            | Self::VenueMismatch { .. }
            | Self::ChainMismatch { .. }
            | Self::CapabilityUnavailable { .. }
            | Self::LiveSwapsUnavailable
            | Self::ObserveSwapsUnavailable
            | Self::LiveSimulationUnavailable
            | Self::BroadcastUnavailable
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

impl fmt::Display for DexConnectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                write!(formatter, "DEX/Web3 connector validation failed")?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::RouterNotRegistered { venue, chain } => {
                write!(
                    formatter,
                    "DEX router is not registered: {venue} on {chain}"
                )
            }
            Self::VenueMismatch { profile, request } => {
                write!(
                    formatter,
                    "DEX venue mismatch between profile {profile} and request {request}"
                )
            }
            Self::ChainMismatch { profile, request } => {
                write!(
                    formatter,
                    "DEX chain mismatch between profile {profile} and request {request}"
                )
            }
            Self::CapabilityUnavailable { venue, capability } => {
                write!(
                    formatter,
                    "DEX venue {venue} does not support capability {capability}"
                )
            }
            Self::LiveSwapsUnavailable => {
                formatter.write_str("live DEX swaps are unavailable in Phase 8")
            }
            Self::ObserveSwapsUnavailable => {
                formatter.write_str("observe scope cannot validate executable DEX swaps")
            }
            Self::LiveSimulationUnavailable => {
                formatter.write_str("live RPC transaction simulation is unavailable in Phase 8")
            }
            Self::BroadcastUnavailable => {
                formatter.write_str("transaction broadcast is unavailable in Phase 8")
            }
            Self::PolicyDenied { violations } => {
                write!(
                    formatter,
                    "DEX swap denied by policy with {} violation(s)",
                    violations.len()
                )?;
                for violation in violations {
                    write!(formatter, "; {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::AuditJournalFailed { reason } => {
                write!(
                    formatter,
                    "DEX/Web3 audit journal persistence failed: {reason}"
                )
            }
            Self::StateStoreFailed { reason } => {
                write!(
                    formatter,
                    "DEX/Web3 state-store persistence failed: {reason}"
                )
            }
        }
    }
}

impl Error for DexConnectorError {}

impl From<StateStoreError> for DexConnectorError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
}

/// Persist the latest local DEX/Web3 framework validation through the state boundary.
pub fn persist_dex_swap_validation_checkpoint(
    store: &mut impl StateStore,
    record: &DexSwapValidationRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DexConnectorError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_SWAP_VALIDATION_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize DEX validation checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local DEX/Web3 framework validation record to the audit journal.
pub fn append_dex_swap_validation_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &DexSwapValidationRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DexConnectorError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("dex-swap-validation-{}", record.request_id),
        AuditEventKind::ExecutionSubmission,
        DEX_STATE_SUBSYSTEM,
        "dex-framework",
        "DEX/Web3 swap framework validation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(record.framework_version.clone()),
        )
        .with_metadata("request_id", AuditValue::Text(record.request_id.clone()))
        .with_metadata("venue", AuditValue::Text(record.venue.name.clone()))
        .with_metadata("chain", AuditValue::Text(record.chain.clone()))
        .with_metadata("scope", AuditValue::Text(format!("{:?}", record.scope)))
        .with_metadata(
            "simulation_status",
            AuditValue::Text(format!("{:?}", record.simulation_status)),
        )
        .with_metadata("policy_approved", AuditValue::Bool(record.policy_approved))
        .with_metadata(
            "rpc_call_performed",
            AuditValue::Bool(record.rpc_call_performed),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(record.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(record.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(record.live_execution_performed),
        );
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local DEX/Web3 lifecycle reconciliation through state.
pub fn persist_dex_swap_lifecycle_checkpoint(
    store: &mut impl StateStore,
    record: &DexSwapLifecycleRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, DexConnectorError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize DEX lifecycle checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local DEX/Web3 lifecycle reconciliation record to the audit journal.
pub fn append_dex_swap_lifecycle_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &DexSwapLifecycleRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, DexConnectorError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("dex-swap-lifecycle-{}", record.request_id),
        AuditEventKind::ExecutionResult,
        DEX_STATE_SUBSYSTEM,
        "dex-framework",
        "DEX/Web3 swap lifecycle reconciliation recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(record.framework_version.clone()),
        )
        .with_metadata("request_id", AuditValue::Text(record.request_id.clone()))
        .with_metadata("venue", AuditValue::Text(record.venue.name.clone()))
        .with_metadata("chain", AuditValue::Text(record.chain.clone()))
        .with_metadata(
            "quote_response_id",
            AuditValue::Text(record.quote_response_id.clone()),
        )
        .with_metadata(
            "simulation_response_id",
            AuditValue::Text(record.simulation_response_id.clone()),
        )
        .with_metadata(
            "simulation_status",
            AuditValue::Text(format!("{:?}", record.simulation_status)),
        )
        .with_metadata(
            "duplicate_intent_id_rejected",
            AuditValue::Bool(record.duplicate_intent_id_rejected),
        )
        .with_metadata(
            "rpc_call_performed",
            AuditValue::Bool(record.rpc_call_performed),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(record.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(record.broadcast_performed),
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
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 nonce reservation through state.
pub fn persist_web3_nonce_reservation_checkpoint(
    store: &mut impl StateStore,
    report: &Web3NonceReservationReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize Web3 nonce reservation checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms: report.planned_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 nonce reservation to the audit journal.
pub fn append_web3_nonce_reservation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3NonceReservationReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-nonce-reservation-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-nonce-reservation",
        "local Web3 nonce reservation reviewed without RPC, signing, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.planned_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("reservation_id", AuditValue::Text(report.id.clone()))
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata(
            "account_label",
            AuditValue::Text(report.account_label.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("nonce_ready", AuditValue::Bool(report.nonce_ready))
        .with_metadata(
            "in_flight_nonce_count",
            AuditValue::Unsigned(report.in_flight_nonce_count),
        )
        .with_metadata("ttl_ms", AuditValue::Unsigned(report.ttl_ms))
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    if let Some(nonce) = report.reserved_nonce {
        event = event.with_metadata("reserved_nonce", AuditValue::Unsigned(nonce));
    }
    if let Some(nonce) = report.last_confirmed_nonce {
        event = event.with_metadata("last_confirmed_nonce", AuditValue::Unsigned(nonce));
    }
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 unsigned payload review through state.
pub fn persist_web3_unsigned_payload_review_checkpoint(
    store: &mut impl StateStore,
    report: &Web3UnsignedPayloadReviewReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize Web3 unsigned payload checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 unsigned payload review to the audit journal.
pub fn append_web3_unsigned_payload_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3UnsignedPayloadReviewReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-unsigned-payload-review-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-unsigned-payload-review",
        "local Web3 unsigned payload reviewed without raw calldata, RPC, signing, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.reviewed_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata(
            "nonce_reservation_id",
            AuditValue::Text(report.nonce_reservation_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata(
            "account_label",
            AuditValue::Text(report.account_label.clone()),
        )
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata("nonce_ready", AuditValue::Bool(report.nonce_ready))
        .with_metadata(
            "payload_reference_ready",
            AuditValue::Bool(report.payload_reference_ready),
        )
        .with_metadata(
            "router_spender_ready",
            AuditValue::Bool(report.router_spender_ready),
        )
        .with_metadata("gas_cap_ready", AuditValue::Bool(report.gas_cap_ready))
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata(
            "raw_calldata_embedded",
            AuditValue::Bool(report.raw_calldata_embedded),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    if let Some(nonce) = report.reserved_nonce {
        event = event.with_metadata("reserved_nonce", AuditValue::Unsigned(nonce));
    }
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 pre-sign safety review through state.
pub fn persist_web3_pre_sign_safety_checkpoint(
    store: &mut impl StateStore,
    report: &Web3PreSignSafetyReviewReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize Web3 pre-sign safety checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 pre-sign safety review to the audit journal.
pub fn append_web3_pre_sign_safety_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3PreSignSafetyReviewReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-pre-sign-safety-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-pre-sign-safety",
        "local Web3 pre-sign safety reviewed without RPC, signing, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.reviewed_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata(
            "simulation_response_id",
            AuditValue::Text(report.simulation_response_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "simulation_success_ready",
            AuditValue::Bool(report.simulation_success_ready),
        )
        .with_metadata(
            "gas_fee_within_cap",
            AuditValue::Bool(report.gas_fee_within_cap),
        )
        .with_metadata(
            "output_amount_sufficient",
            AuditValue::Bool(report.output_amount_sufficient),
        )
        .with_metadata("nonce_ready", AuditValue::Bool(report.nonce_ready))
        .with_metadata(
            "lifecycle_coherent",
            AuditValue::Bool(report.lifecycle_coherent),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 broadcast-readiness review through state.
pub fn persist_web3_broadcast_readiness_checkpoint(
    store: &mut impl StateStore,
    report: &Web3BroadcastReadinessReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!("failed to serialize Web3 broadcast readiness checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 broadcast-readiness review to the audit journal.
pub fn append_web3_broadcast_readiness_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3BroadcastReadinessReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-broadcast-readiness-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-broadcast-readiness",
        "local Web3 broadcast readiness reviewed without RPC, signer material, signing, broadcast permission, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.reviewed_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "unsigned_payload_review_id",
            AuditValue::Text(report.unsigned_payload_review_id.clone()),
        )
        .with_metadata(
            "pre_sign_safety_review_id",
            AuditValue::Text(report.pre_sign_safety_review_id.clone()),
        )
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "unsigned_payload_ready",
            AuditValue::Bool(report.unsigned_payload_ready),
        )
        .with_metadata(
            "pre_sign_safety_ready",
            AuditValue::Bool(report.pre_sign_safety_ready),
        )
        .with_metadata(
            "signer_authorization_reference_ready",
            AuditValue::Bool(report.signer_authorization_reference_ready),
        )
        .with_metadata(
            "live_adapter_reference_ready",
            AuditValue::Bool(report.live_adapter_reference_ready),
        )
        .with_metadata(
            "operator_approval_reference_ready",
            AuditValue::Bool(report.operator_approval_reference_ready),
        )
        .with_metadata(
            "broadcast_allowed",
            AuditValue::Bool(report.broadcast_allowed),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 unsigned transaction construction through state.
pub fn persist_web3_unsigned_transaction_construction_checkpoint(
    store: &mut impl StateStore,
    report: &Web3UnsignedTransactionConstructionReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!(
                    "failed to serialize Web3 unsigned transaction construction checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms: report.constructed_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 unsigned transaction construction to the audit journal.
pub fn append_web3_unsigned_transaction_construction_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3UnsignedTransactionConstructionReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-unsigned-transaction-construction-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-unsigned-transaction-construction",
        "local Web3 unsigned transaction metadata constructed without raw calldata, raw transaction serialization, RPC, signer material, signing, broadcast permission, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.constructed_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("construction_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "broadcast_readiness_review_id",
            AuditValue::Text(report.broadcast_readiness_review_id.clone()),
        )
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "broadcast_readiness_ready",
            AuditValue::Bool(report.broadcast_readiness_ready),
        )
        .with_metadata(
            "payload_reference_ready",
            AuditValue::Bool(report.payload_reference_ready),
        )
        .with_metadata(
            "target_selector_ready",
            AuditValue::Bool(report.target_selector_ready),
        )
        .with_metadata("nonce_ready", AuditValue::Bool(report.nonce_ready))
        .with_metadata(
            "gas_metadata_ready",
            AuditValue::Bool(report.gas_metadata_ready),
        )
        .with_metadata(
            "raw_calldata_embedded",
            AuditValue::Bool(report.raw_calldata_embedded),
        )
        .with_metadata(
            "raw_transaction_serialized",
            AuditValue::Bool(report.raw_transaction_serialized),
        )
        .with_metadata(
            "broadcast_allowed",
            AuditValue::Bool(report.broadcast_allowed),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    if let Some(nonce) = report.construction_nonce {
        event = event.with_metadata("construction_nonce", AuditValue::Unsigned(nonce));
    }
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 provider nonce reconciliation through state.
pub fn persist_web3_provider_nonce_reconciliation_checkpoint(
    store: &mut impl StateStore,
    report: &Web3ProviderNonceReconciliationReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!(
                    "failed to serialize Web3 provider nonce reconciliation checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms: report.reconciled_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 provider nonce reconciliation to the audit journal.
pub fn append_web3_provider_nonce_reconciliation_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3ProviderNonceReconciliationReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-provider-nonce-reconciliation-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-provider-nonce-reconciliation",
        "local Web3 provider nonce metadata reconciled without RPC, signer material, signing, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.reconciled_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("reconciliation_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "unsigned_transaction_construction_id",
            AuditValue::Text(report.unsigned_transaction_construction_id.clone()),
        )
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "unsigned_transaction_ready",
            AuditValue::Bool(report.unsigned_transaction_ready),
        )
        .with_metadata(
            "provider_snapshot_reference_ready",
            AuditValue::Bool(report.provider_snapshot_reference_ready),
        )
        .with_metadata(
            "provider_next_nonce_ready",
            AuditValue::Bool(report.provider_next_nonce_ready),
        )
        .with_metadata(
            "construction_nonce_matches_provider",
            AuditValue::Bool(report.construction_nonce_matches_provider),
        )
        .with_metadata(
            "construction_nonce_not_pending",
            AuditValue::Bool(report.construction_nonce_not_pending),
        )
        .with_metadata(
            "pending_nonce_set_unique",
            AuditValue::Bool(report.pending_nonce_set_unique),
        )
        .with_metadata("snapshot_fresh", AuditValue::Bool(report.snapshot_fresh))
        .with_metadata(
            "provider_pending_nonce_count",
            AuditValue::Unsigned(report.provider_pending_nonce_count),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    if let Some(nonce) = report.construction_nonce {
        event = event.with_metadata("construction_nonce", AuditValue::Unsigned(nonce));
    }
    if let Some(nonce) = report.provider_next_nonce {
        event = event.with_metadata("provider_next_nonce", AuditValue::Unsigned(nonce));
    }
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 raw transaction serialization review through state.
pub fn persist_web3_raw_transaction_serialization_review_checkpoint(
    store: &mut impl StateStore,
    report: &Web3RawTransactionSerializationReviewReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!(
                    "failed to serialize Web3 raw transaction serialization review checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 raw transaction serialization review to the audit journal.
pub fn append_web3_raw_transaction_serialization_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3RawTransactionSerializationReviewReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-raw-transaction-serialization-review-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-raw-transaction-serialization-review",
        "local Web3 raw transaction serialization metadata reviewed without raw bytes, serialization, RPC, signer material, signing, broadcast permission, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.reviewed_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "provider_nonce_reconciliation_id",
            AuditValue::Text(report.provider_nonce_reconciliation_id.clone()),
        )
        .with_metadata(
            "unsigned_transaction_construction_id",
            AuditValue::Text(report.unsigned_transaction_construction_id.clone()),
        )
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "provider_nonce_reconciliation_ready",
            AuditValue::Bool(report.provider_nonce_reconciliation_ready),
        )
        .with_metadata(
            "transaction_type_ready",
            AuditValue::Bool(report.transaction_type_ready),
        )
        .with_metadata("chain_id_ready", AuditValue::Bool(report.chain_id_ready))
        .with_metadata(
            "fee_fields_ready",
            AuditValue::Bool(report.fee_fields_ready),
        )
        .with_metadata(
            "access_list_reference_ready",
            AuditValue::Bool(report.access_list_reference_ready),
        )
        .with_metadata(
            "raw_transaction_bytes_embedded",
            AuditValue::Bool(report.raw_transaction_bytes_embedded),
        )
        .with_metadata(
            "raw_calldata_embedded",
            AuditValue::Bool(report.raw_calldata_embedded),
        )
        .with_metadata(
            "raw_transaction_serialized",
            AuditValue::Bool(report.raw_transaction_serialized),
        )
        .with_metadata(
            "broadcast_allowed",
            AuditValue::Bool(report.broadcast_allowed),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 broadcast adapter control review through state.
pub fn persist_web3_broadcast_adapter_control_review_checkpoint(
    store: &mut impl StateStore,
    report: &Web3BroadcastAdapterControlReviewReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!(
                    "failed to serialize Web3 broadcast adapter control review checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms: report.reviewed_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 broadcast adapter control review to the audit journal.
pub fn append_web3_broadcast_adapter_control_review_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3BroadcastAdapterControlReviewReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-broadcast-adapter-control-review-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-broadcast-adapter-control-review",
        "local Web3 broadcast adapter controls reviewed without broadcast permission, raw bytes, serialization, RPC, signer material, signing, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.reviewed_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("review_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "raw_transaction_serialization_review_id",
            AuditValue::Text(report.raw_transaction_serialization_review_id.clone()),
        )
        .with_metadata(
            "provider_nonce_reconciliation_id",
            AuditValue::Text(report.provider_nonce_reconciliation_id.clone()),
        )
        .with_metadata(
            "unsigned_transaction_construction_id",
            AuditValue::Text(report.unsigned_transaction_construction_id.clone()),
        )
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "raw_transaction_serialization_review_ready",
            AuditValue::Bool(report.raw_transaction_serialization_review_ready),
        )
        .with_metadata(
            "adapter_reference_ready",
            AuditValue::Bool(report.adapter_reference_ready),
        )
        .with_metadata(
            "operator_approval_reference_ready",
            AuditValue::Bool(report.operator_approval_reference_ready),
        )
        .with_metadata(
            "audit_state_preflight_reference_ready",
            AuditValue::Bool(report.audit_state_preflight_reference_ready),
        )
        .with_metadata(
            "kill_switch_confirmed",
            AuditValue::Bool(report.kill_switch_confirmed),
        )
        .with_metadata(
            "rate_limit_control_ready",
            AuditValue::Bool(report.rate_limit_control_ready),
        )
        .with_metadata(
            "replay_protection_ready",
            AuditValue::Bool(report.replay_protection_ready),
        )
        .with_metadata(
            "broadcast_permission_granted",
            AuditValue::Bool(report.broadcast_permission_granted),
        )
        .with_metadata(
            "raw_transaction_bytes_embedded",
            AuditValue::Bool(report.raw_transaction_bytes_embedded),
        )
        .with_metadata(
            "raw_transaction_serialized",
            AuditValue::Bool(report.raw_transaction_serialized),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Persist the latest local Web3 sandbox/live discrepancy calibration through state.
pub fn persist_web3_sandbox_live_discrepancy_calibration_checkpoint(
    store: &mut impl StateStore,
    report: &Web3SandboxLiveDiscrepancyCalibrationReport,
) -> Result<StateCheckpoint, DexConnectorError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY.to_owned(),
        subsystem: DEX_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            DexConnectorError::StateStoreFailed {
                reason: format!(
                    "failed to serialize Web3 sandbox/live discrepancy calibration checkpoint: {error}"
                ),
            }
        })?,
        updated_at_unix_ms: report.calibrated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(DexConnectorError::from)?;
    Ok(checkpoint)
}

/// Append one local Web3 sandbox/live discrepancy calibration to the audit journal.
pub fn append_web3_sandbox_live_discrepancy_calibration_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &Web3SandboxLiveDiscrepancyCalibrationReport,
) -> Result<AuditRecord, DexConnectorError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("web3-sandbox-live-discrepancy-calibration-{}", report.id),
        AuditEventKind::SecurityAlert,
        DEX_STATE_SUBSYSTEM,
        "web3-sandbox-live-discrepancy-calibration",
        "local Web3 sandbox/live discrepancy metadata calibrated from references without external calls, credentials, RPC, signer material, signing, broadcast, or live execution",
    );
    event.occurred_at_unix_ms = report.calibrated_at_unix_ms;
    event = event
        .with_metadata(
            "dex_framework_version",
            AuditValue::Text(report.framework_version.clone()),
        )
        .with_metadata("calibration_id", AuditValue::Text(report.id.clone()))
        .with_metadata(
            "broadcast_adapter_control_review_id",
            AuditValue::Text(report.broadcast_adapter_control_review_id.clone()),
        )
        .with_metadata(
            "raw_transaction_serialization_review_id",
            AuditValue::Text(report.raw_transaction_serialization_review_id.clone()),
        )
        .with_metadata(
            "provider_nonce_reconciliation_id",
            AuditValue::Text(report.provider_nonce_reconciliation_id.clone()),
        )
        .with_metadata(
            "simulation_request_id",
            AuditValue::Text(report.simulation_request_id.clone()),
        )
        .with_metadata("chain", AuditValue::Text(report.chain.clone()))
        .with_metadata("venue", AuditValue::Text(report.venue.name.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "broadcast_adapter_control_ready",
            AuditValue::Bool(report.broadcast_adapter_control_ready),
        )
        .with_metadata(
            "sandbox_observation_reference_ready",
            AuditValue::Bool(report.sandbox_observation_reference_ready),
        )
        .with_metadata(
            "live_observation_reference_ready",
            AuditValue::Bool(report.live_observation_reference_ready),
        )
        .with_metadata(
            "sample_size_ready",
            AuditValue::Bool(report.sample_size_ready),
        )
        .with_metadata(
            "price_deviation_within_limit",
            AuditValue::Bool(report.price_deviation_within_limit),
        )
        .with_metadata(
            "latency_deviation_within_limit",
            AuditValue::Bool(report.latency_deviation_within_limit),
        )
        .with_metadata(
            "fee_deviation_within_limit",
            AuditValue::Bool(report.fee_deviation_within_limit),
        )
        .with_metadata(
            "sandbox_sample_count",
            AuditValue::Unsigned(report.sandbox_sample_count),
        )
        .with_metadata(
            "live_sample_count",
            AuditValue::Unsigned(report.live_sample_count),
        )
        .with_metadata(
            "violation_count",
            AuditValue::Unsigned(report.violation_count),
        )
        .with_metadata(
            "external_call_performed",
            AuditValue::Bool(report.external_call_performed),
        )
        .with_metadata(
            "credential_loaded",
            AuditValue::Bool(report.credential_loaded),
        )
        .with_metadata("rpc_called", AuditValue::Bool(report.rpc_called))
        .with_metadata(
            "signer_material_loaded",
            AuditValue::Bool(report.signer_material_loaded),
        )
        .with_metadata(
            "signing_performed",
            AuditValue::Bool(report.signing_performed),
        )
        .with_metadata(
            "broadcast_performed",
            AuditValue::Bool(report.broadcast_performed),
        )
        .with_metadata(
            "live_execution_performed",
            AuditValue::Bool(report.live_execution_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(|error| DexConnectorError::audit_failed(error.to_string()))
}

/// Validate local DEX/Web3 intent id uniqueness for idempotency tests.
pub fn validate_dex_intent_id_uniqueness(
    records: &[DexSwapValidationRecord],
) -> Result<(), DexConnectorError> {
    let mut seen = HashSet::new();
    for record in records {
        record.validate()?;
        let normalized = record.request_id.to_ascii_lowercase();
        if !seen.insert(normalized) {
            return Err(DexConnectorError::ValidationFailed {
                violations: vec![DexConnectorViolation::new_owned(
                    "DEX_INTENT_ID_DUPLICATE",
                    format!("duplicate DEX/Web3 intent id: {}", record.request_id),
                )],
            });
        }
    }
    Ok(())
}

fn validate_profile_matches_request(
    profile: &DexRouterProfile,
    request: &DexSwapQuoteRequest,
) -> Result<(), DexConnectorError> {
    if !same_venue(&profile.venue, &request.venue) {
        return Err(DexConnectorError::VenueMismatch {
            profile: profile.venue.name.clone(),
            request: request.venue.name.clone(),
        });
    }

    if !profile.chain.eq_ignore_ascii_case(&request.chain) {
        return Err(DexConnectorError::ChainMismatch {
            profile: profile.chain.clone(),
            request: request.chain.clone(),
        });
    }

    Ok(())
}

fn validate_profile_matches_simulation(
    profile: &DexRouterProfile,
    request: &Web3TransactionSimulationRequest,
) -> Result<(), DexConnectorError> {
    if !same_venue(&profile.venue, &request.venue) {
        return Err(DexConnectorError::VenueMismatch {
            profile: profile.venue.name.clone(),
            request: request.venue.name.clone(),
        });
    }

    if !profile.chain.eq_ignore_ascii_case(&request.chain) {
        return Err(DexConnectorError::ChainMismatch {
            profile: profile.chain.clone(),
            request: request.chain.clone(),
        });
    }

    if !profile
        .router_label
        .eq_ignore_ascii_case(&request.router_label)
        || !profile
            .spender_label
            .eq_ignore_ascii_case(&request.spender_label)
    {
        return Err(DexConnectorError::CapabilityUnavailable {
            venue: profile.venue.name.clone(),
            capability: "reviewed-router-and-spender-labels",
        });
    }

    Ok(())
}

fn validate_quote_capabilities(
    profile: &DexRouterProfile,
    request: &DexSwapQuoteRequest,
) -> Result<(), DexConnectorError> {
    let venue = profile.venue.name.clone();

    if !profile.capabilities.quote {
        return Err(DexConnectorError::CapabilityUnavailable {
            venue,
            capability: "quote",
        });
    }

    match request.mode {
        DexSwapMode::ExactInput if !profile.capabilities.exact_input_swaps => {
            return Err(DexConnectorError::CapabilityUnavailable {
                venue: venue.clone(),
                capability: "exact-input-swaps",
            });
        }
        DexSwapMode::ExactOutput if !profile.capabilities.exact_output_swaps => {
            return Err(DexConnectorError::CapabilityUnavailable {
                venue: venue.clone(),
                capability: "exact-output-swaps",
            });
        }
        DexSwapMode::ExactInput | DexSwapMode::ExactOutput => {}
    }

    match request.route_kind {
        DexRouteKind::MultiHop | DexRouteKind::Aggregator
            if !profile.capabilities.route_aggregation =>
        {
            return Err(DexConnectorError::CapabilityUnavailable {
                venue: venue.clone(),
                capability: "route-aggregation",
            });
        }
        DexRouteKind::SinglePool | DexRouteKind::MultiHop | DexRouteKind::Aggregator => {}
    }

    if request.gas_fee_quote > 0.0 && !profile.capabilities.gas_estimation {
        return Err(DexConnectorError::CapabilityUnavailable {
            venue,
            capability: "gas-estimation",
        });
    }

    Ok(())
}

fn validate_dex_venue_ref(venue: &VenueRef, violations: &mut Vec<DexConnectorViolation>) {
    if venue.name.trim().is_empty() {
        violations.push(DexConnectorViolation::new(
            "DEX_VENUE_NAME_REQUIRED",
            "DEX venue name must be non-empty",
        ));
    }

    if !matches!(venue.kind, VenueKind::Dex | VenueKind::Aggregator) {
        violations.push(DexConnectorViolation::new(
            "DEX_VENUE_KIND_REQUIRED",
            "DEX framework only accepts VenueKind::Dex or VenueKind::Aggregator venues",
        ));
    }
}

fn validate_dex_venue_ref_result(venue: &VenueRef) -> Result<(), DexConnectorError> {
    let mut violations = Vec::new();
    validate_dex_venue_ref(venue, &mut violations);
    finish_validation(violations)
}

fn has_duplicate_nonce(nonces: &[u64]) -> bool {
    let mut seen = HashSet::new();
    nonces.iter().any(|nonce| !seen.insert(*nonce))
}

fn payload_reference_is_hash_or_label(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 128
        && !trimmed.contains("0x")
        && !trimmed.contains('{')
        && !trimmed.contains('}')
        && !trimmed.contains('(')
        && !trimmed.contains(')')
        && trimmed.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
        })
}

fn validate_id(label: &'static str, value: &str, violations: &mut Vec<DexConnectorViolation>) {
    if value.trim().is_empty() {
        violations.push(DexConnectorViolation::new_owned(
            "DEX_ID_REQUIRED",
            format!("{label} id must be non-empty"),
        ));
    }
}

fn validate_non_empty(
    code: &'static str,
    label: &'static str,
    value: &str,
    violations: &mut Vec<DexConnectorViolation>,
) {
    if value.trim().is_empty() {
        violations.push(DexConnectorViolation::new_owned(
            code,
            format!("{label} must be non-empty"),
        ));
    }
}

fn validate_expected_value(
    code: &'static str,
    label: &'static str,
    actual: Option<&str>,
    expected: &str,
    violations: &mut Vec<DexConnectorViolation>,
) {
    if !actual.is_some_and(|value| value.eq_ignore_ascii_case(expected)) {
        violations.push(DexConnectorViolation::new_owned(
            code,
            format!("{label} must be {expected}"),
        ));
    }
}

fn validate_http_path(path: Option<&str>, violations: &mut Vec<DexConnectorViolation>) {
    let Some(path) = path else {
        violations.push(DexConnectorViolation::new(
            "DEX_REQUEST_PLAN_HTTP_PATH_REQUIRED",
            "HTTP request plan path must be present",
        ));
        return;
    };
    if !path.starts_with('/') || path.contains("://") || path.trim().len() < 2 {
        violations.push(DexConnectorViolation::new(
            "DEX_REQUEST_PLAN_HTTP_PATH_INVALID",
            "HTTP request plan path must be a relative API path",
        ));
    }
}

fn validate_non_empty_fields(
    code: &'static str,
    label: &'static str,
    fields: &[String],
    violations: &mut Vec<DexConnectorViolation>,
) {
    if fields.is_empty() || fields.iter().any(|field| field.trim().is_empty()) {
        violations.push(DexConnectorViolation::new_owned(
            code,
            format!("at least one non-empty {label} is required"),
        ));
    }
}

fn first_json_number(payload: &Value, keys: &[&str]) -> Result<f64, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_number(value, key);
        }
    }
    Err(DexConnectorError::ValidationFailed {
        violations: vec![DexConnectorViolation::new_owned(
            "DEX_RESPONSE_TRANSCRIPT_NUMBER_MISSING",
            format!("missing numeric field from any of: {}", keys.join(", ")),
        )],
    })
}

fn first_json_number_or_default(
    payload: &Value,
    keys: &[&str],
    default: f64,
) -> Result<f64, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_number(value, key);
        }
    }
    Ok(default)
}

fn first_json_u64(payload: &Value, keys: &[&str]) -> Result<u64, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_u64(value, key);
        }
    }
    Err(DexConnectorError::ValidationFailed {
        violations: vec![DexConnectorViolation::new_owned(
            "DEX_RESPONSE_TRANSCRIPT_U64_MISSING",
            format!(
                "missing unsigned integer field from any of: {}",
                keys.join(", ")
            ),
        )],
    })
}

fn first_json_u64_or_default(
    payload: &Value,
    keys: &[&str],
    default: u64,
) -> Result<u64, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_u64(value, key);
        }
    }
    Ok(default)
}

fn optional_json_u64(payload: &Value, keys: &[&str]) -> Result<Option<u64>, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_u64(value, key).map(Some);
        }
    }
    Ok(None)
}

fn first_json_string(payload: &Value, keys: &[&str]) -> Result<String, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_string(value, key);
        }
    }
    Err(DexConnectorError::ValidationFailed {
        violations: vec![DexConnectorViolation::new_owned(
            "DEX_RESPONSE_TRANSCRIPT_STRING_MISSING",
            format!("missing string field from any of: {}", keys.join(", ")),
        )],
    })
}

fn optional_json_string(
    payload: &Value,
    keys: &[&str],
) -> Result<Option<String>, DexConnectorError> {
    for key in keys {
        if let Some(value) = payload.get(*key) {
            return json_string(value, key).map(Some);
        }
    }
    Ok(None)
}

fn optional_solana_error(payload: &Value) -> Result<Option<String>, DexConnectorError> {
    let Some(value) = payload.get("err").or_else(|| payload.get("error")) else {
        return optional_json_string(payload, &["diagnostic"]);
    };
    match value {
        Value::Null => Ok(None),
        Value::String(text) if text.trim().is_empty() => Ok(None),
        Value::String(text) => Ok(Some(text.clone())),
        other => Ok(Some(other.to_string())),
    }
}

fn json_number(value: &Value, label: &str) -> Result<f64, DexConnectorError> {
    let parsed = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    };
    parsed
        .filter(|number| number.is_finite())
        .ok_or_else(|| DexConnectorError::ValidationFailed {
            violations: vec![DexConnectorViolation::new_owned(
                "DEX_RESPONSE_TRANSCRIPT_NUMBER_INVALID",
                format!("{label} must be a finite number or numeric string"),
            )],
        })
}

fn json_u64(value: &Value, label: &str) -> Result<u64, DexConnectorError> {
    let parsed = match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse::<u64>().ok(),
        _ => None,
    };
    parsed.ok_or_else(|| DexConnectorError::ValidationFailed {
        violations: vec![DexConnectorViolation::new_owned(
            "DEX_RESPONSE_TRANSCRIPT_U64_INVALID",
            format!("{label} must be an unsigned integer or numeric string"),
        )],
    })
}

fn json_string(value: &Value, label: &str) -> Result<String, DexConnectorError> {
    let Value::String(text) = value else {
        return Err(DexConnectorError::ValidationFailed {
            violations: vec![DexConnectorViolation::new_owned(
                "DEX_RESPONSE_TRANSCRIPT_STRING_INVALID",
                format!("{label} must be a string"),
            )],
        });
    };
    if text.trim().is_empty() {
        return Err(DexConnectorError::ValidationFailed {
            violations: vec![DexConnectorViolation::new_owned(
                "DEX_RESPONSE_TRANSCRIPT_STRING_EMPTY",
                format!("{label} must be non-empty"),
            )],
        });
    }
    Ok(text.trim().to_owned())
}

fn finish_validation(violations: Vec<DexConnectorViolation>) -> Result<(), DexConnectorError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(DexConnectorError::ValidationFailed { violations })
    }
}

fn same_venue(left: &VenueRef, right: &VenueRef) -> bool {
    left.kind == right.kind && left.name.eq_ignore_ascii_case(&right.name)
}

fn normalize_symbol(value: String) -> String {
    value.trim().to_ascii_uppercase()
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
        append_dex_swap_lifecycle_audit, append_dex_swap_validation_audit,
        append_web3_broadcast_adapter_control_review_audit, append_web3_broadcast_readiness_audit,
        append_web3_nonce_reservation_audit, append_web3_pre_sign_safety_audit,
        append_web3_provider_nonce_reconciliation_audit,
        append_web3_raw_transaction_serialization_review_audit,
        append_web3_sandbox_live_discrepancy_calibration_audit,
        append_web3_unsigned_payload_review_audit,
        append_web3_unsigned_transaction_construction_audit, persist_dex_swap_lifecycle_checkpoint,
        persist_dex_swap_validation_checkpoint,
        persist_web3_broadcast_adapter_control_review_checkpoint,
        persist_web3_broadcast_readiness_checkpoint, persist_web3_nonce_reservation_checkpoint,
        persist_web3_pre_sign_safety_checkpoint,
        persist_web3_provider_nonce_reconciliation_checkpoint,
        persist_web3_raw_transaction_serialization_review_checkpoint,
        persist_web3_sandbox_live_discrepancy_calibration_checkpoint,
        persist_web3_unsigned_payload_review_checkpoint,
        persist_web3_unsigned_transaction_construction_checkpoint,
        validate_dex_intent_id_uniqueness, DexConnectorError, DexConnectorRegistry, DexPolicyGate,
        DexProtocolRiskReviewRequest, DexProtocolRiskReviewStatus, DexQuoteConnector,
        DexRequestPlan, DexRequestPlanKind, DexResponseTranscript, DexRouteKind,
        DexRouterCapabilities, DexRouterProfile, DexSimulationStatus, DexSwapLifecycleRecord,
        DexSwapMode, DexSwapQuoteRequest, DexSwapQuoteResponse, DexSwapValidationRecord,
        DexTokenProfile, LocalDeterministicDexAdapter, Web3BroadcastAdapterControlReviewReport,
        Web3BroadcastAdapterControlReviewRequest, Web3BroadcastAdapterControlReviewStatus,
        Web3BroadcastReadinessReport, Web3BroadcastReadinessRequest, Web3BroadcastReadinessStatus,
        Web3ChainProfile, Web3NonceReservationReport, Web3NonceReservationRequest,
        Web3NonceReservationStatus, Web3PreSignSafetyReviewReport, Web3PreSignSafetyReviewRequest,
        Web3PreSignSafetyReviewStatus, Web3ProviderNonceReconciliationReport,
        Web3ProviderNonceReconciliationRequest, Web3ProviderNonceReconciliationStatus,
        Web3RawTransactionSerializationReviewReport, Web3RawTransactionSerializationReviewRequest,
        Web3RawTransactionSerializationReviewStatus, Web3SandboxLiveDiscrepancyCalibrationReport,
        Web3SandboxLiveDiscrepancyCalibrationRequest, Web3SandboxLiveDiscrepancyCalibrationStatus,
        Web3SimulationConnector, Web3TransactionLifecycleRecord, Web3TransactionLifecycleStatus,
        Web3TransactionLifecycleTranscript, Web3TransactionLifecycleTranscriptFormat,
        Web3TransactionSimulationRequest, Web3TransactionSimulationResponse,
        Web3UnsignedPayloadReviewReport, Web3UnsignedPayloadReviewRequest,
        Web3UnsignedPayloadReviewStatus, Web3UnsignedTransactionConstructionReport,
        Web3UnsignedTransactionConstructionRequest, Web3UnsignedTransactionConstructionStatus,
        DEX_CONNECTOR_FRAMEWORK_VERSION, DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY,
        DEX_LAST_SWAP_VALIDATION_CHECKPOINT_KEY,
        DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY,
        DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY,
        DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY,
        DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY,
        DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY,
        DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY,
        DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY,
        DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY,
        DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY, DEX_STATE_SUBSYSTEM,
    };
    use crate::{
        AgentConfig, AppendOnlyAuditJournal, ApprovedDestinationEntry, DestinationApprovalSource,
        ExecutionScope, FeeProvider, FeeSchedule, MarketPair, PolicyContext, PolicyEngine,
        SqliteWalStateStore, StateStore, VenueKind, VenueRef,
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
slippage_bps = 75
gas_fee_cap_quote = 2.0

[venues]
cex_allowlist = []
dex_allowlist = ["paper-uniswap"]
chain_allowlist = ["ethereum"]
asset_allowlist = ["ETH", "USDC"]

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
            name: "paper-uniswap".to_owned(),
            kind: VenueKind::Dex,
        }
    }

    fn router_profile() -> DexRouterProfile {
        DexRouterProfile::new(
            venue(),
            "ethereum",
            "Paper Uniswap",
            "uniswap-v3-router-reviewed",
            "uniswap-v3-spender-reviewed",
            DexRouterCapabilities::paper_simulation(),
        )
        .expect("router profile should validate")
    }

    fn swap_request() -> DexSwapQuoteRequest {
        DexSwapQuoteRequest {
            id: "dex-swap-1".to_owned(),
            strategy_id: "strategy-dex".to_owned(),
            scope: ExecutionScope::Paper,
            venue: venue(),
            chain: "ethereum".to_owned(),
            pair: MarketPair::new("ETH", "USDC").expect("pair should validate"),
            mode: DexSwapMode::ExactInput,
            route_kind: DexRouteKind::SinglePool,
            input_token_symbol: "ETH".to_owned(),
            output_token_symbol: "USDC".to_owned(),
            amount_in: 0.1,
            expected_amount_out: 25.0,
            notional_quote: 25.0,
            expected_profit_quote: 1.0,
            max_loss_quote: 1.0,
            slippage_bps: 20,
            estimated_fee_quote: 0.10,
            gas_fee_quote: 0.25,
            market_data_age_ms: 1_000,
        }
    }

    fn simulation_request() -> Web3TransactionSimulationRequest {
        Web3TransactionSimulationRequest {
            id: "dex-sim-1".to_owned(),
            swap_request_id: "dex-swap-1".to_owned(),
            scope: ExecutionScope::Paper,
            venue: venue(),
            chain: "ethereum".to_owned(),
            router_label: "uniswap-v3-router-reviewed".to_owned(),
            spender_label: "uniswap-v3-spender-reviewed".to_owned(),
            account_label: "paper-wallet".to_owned(),
            input_token_symbol: "ETH".to_owned(),
            output_token_symbol: "USDC".to_owned(),
            amount_in: 0.1,
            minimum_amount_out: 24.5,
            gas_limit: 150_000,
            max_gas_fee_quote: 0.25,
            payload_hash: "reviewed-payload-hash-only".to_owned(),
        }
    }

    fn request_plans() -> Vec<DexRequestPlan> {
        vec![
            DexRequestPlan::uniswap_v3_quoter_eth_call(
                venue(),
                MarketPair::new("ETH", "USDC").expect("pair should validate"),
                "ethereum",
            )
            .expect("Uniswap request plan should validate"),
            DexRequestPlan::zero_ex_swap_quote_http(
                venue(),
                MarketPair::new("ETH", "USDC").expect("pair should validate"),
                "ethereum",
            )
            .expect("0x request plan should validate"),
            DexRequestPlan::jupiter_quote_http(
                VenueRef {
                    name: "paper-jupiter".to_owned(),
                    kind: VenueKind::Dex,
                },
                MarketPair::new("SOL", "USDC").expect("pair should validate"),
                "solana",
            )
            .expect("Jupiter request plan should validate"),
            DexRequestPlan::evm_transaction_simulation_eth_call(
                venue(),
                MarketPair::new("ETH", "USDC").expect("pair should validate"),
                "ethereum",
            )
            .expect("EVM simulation request plan should validate"),
        ]
    }

    fn policy_gate() -> DexPolicyGate {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        DexPolicyGate::new(PolicyEngine::new(
            config,
            PolicyContext {
                destination_allowlist: vec![ApprovedDestinationEntry {
                    label: "router:paper-uniswap".to_owned(),
                    chain: "ethereum".to_owned(),
                    address_fingerprint: "sha256:paper-uniswap-router".to_owned(),
                    approval_id: "dex-test-approval".to_owned(),
                    approved_by: "local-test-operator".to_owned(),
                    approval_source: DestinationApprovalSource::LocalOperator,
                    ownership_evidence_referenced: true,
                    enabled: true,
                }],
                ..PolicyContext::default()
            },
        ))
    }

    fn local_adapter() -> LocalDeterministicDexAdapter {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        LocalDeterministicDexAdapter::new(
            "local-paper-uniswap",
            router_profile(),
            quote_response(),
            fee_schedule(),
            simulation_response(),
            PolicyEngine::new(
                config,
                PolicyContext {
                    destination_allowlist: vec![ApprovedDestinationEntry {
                        label: "router:paper-uniswap".to_owned(),
                        chain: "ethereum".to_owned(),
                        address_fingerprint: "sha256:paper-uniswap-router".to_owned(),
                        approval_id: "dex-test-approval".to_owned(),
                        approved_by: "local-test-operator".to_owned(),
                        approval_source: DestinationApprovalSource::LocalOperator,
                        ownership_evidence_referenced: true,
                        enabled: true,
                    }],
                    ..PolicyContext::default()
                },
            ),
        )
        .expect("local DEX adapter should validate")
    }

    fn quote_response() -> DexSwapQuoteResponse {
        DexSwapQuoteResponse {
            id: "dex-quote-response-1".to_owned(),
            request_id: "dex-swap-1".to_owned(),
            venue: venue(),
            chain: "ethereum".to_owned(),
            pair: MarketPair::new("ETH", "USDC").expect("pair should validate"),
            route_kind: DexRouteKind::SinglePool,
            amount_in: 0.1,
            amount_out: 25.0,
            price_impact_bps: 12.0,
            estimated_fee_quote: 0.10,
            gas_fee_quote: 0.25,
            market_data_age_ms: 1_000,
            simulation_status: DexSimulationStatus::LocallyValidated,
        }
    }

    fn fee_schedule() -> FeeSchedule {
        FeeSchedule {
            venue: venue(),
            pair: Some(MarketPair::new("ETH", "USDC").expect("pair should validate")),
            maker_bps: 0.0,
            taker_bps: 30.0,
            network_fee_quote: 0.25,
            externally_verified: false,
        }
    }

    fn simulation_response() -> Web3TransactionSimulationResponse {
        Web3TransactionSimulationResponse {
            id: "dex-sim-response-1".to_owned(),
            request_id: "dex-sim-1".to_owned(),
            status: DexSimulationStatus::LocallyValidated,
            gas_used: 150_000,
            gas_fee_quote: 0.25,
            amount_out: 24.5,
            diagnostic: Some("local fixture only; no RPC, signing, or broadcast".to_owned()),
            broadcastable: false,
        }
    }

    fn web3_lifecycle_record() -> Web3TransactionLifecycleRecord {
        Web3TransactionLifecycleRecord {
            framework_version: DEX_CONNECTOR_FRAMEWORK_VERSION.to_owned(),
            transcript_id: "web3-pre-sign-lifecycle".to_owned(),
            request_id: "dex-sim-1".to_owned(),
            chain: "ethereum".to_owned(),
            venue: venue(),
            transaction_id: "0xlocalpresign".to_owned(),
            nonce: Some(7),
            block_number: Some(19_000_001),
            slot: None,
            confirmations: 12,
            status: Web3TransactionLifecycleStatus::Confirmed,
            diagnostic: Some("local lifecycle fixture only".to_owned()),
            rpc_call_performed: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
        }
    }

    fn web3_pre_sign_ready_request() -> Web3PreSignSafetyReviewRequest {
        Web3PreSignSafetyReviewRequest {
            id: "web3-pre-sign-ready".to_owned(),
            simulation_request: simulation_request(),
            simulation_response: simulation_response(),
            lifecycle_record: Some(web3_lifecycle_record()),
            max_gas_fee_quote: 0.25,
            nonce_required: true,
            planned_nonce: Some(7),
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: 1_700_000_300,
        }
    }

    fn web3_pre_sign_ready_report() -> Web3PreSignSafetyReviewReport {
        web3_pre_sign_ready_request().review()
    }

    fn web3_nonce_ready_request() -> Web3NonceReservationRequest {
        Web3NonceReservationRequest {
            id: "web3-nonce-ready".to_owned(),
            chain: "ethereum".to_owned(),
            venue: venue(),
            account_label: "paper-wallet".to_owned(),
            last_confirmed_nonce: Some(6),
            requested_nonce: Some(7),
            in_flight_nonces: vec![8, 9],
            ttl_ms: 30_000,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            planned_at_unix_ms: 1_700_000_350,
        }
    }

    fn web3_nonce_ready_report() -> Web3NonceReservationReport {
        web3_nonce_ready_request().reserve()
    }

    fn web3_unsigned_payload_ready_request() -> Web3UnsignedPayloadReviewRequest {
        Web3UnsignedPayloadReviewRequest {
            id: "web3-unsigned-payload-ready".to_owned(),
            simulation_request: simulation_request(),
            nonce_reservation: web3_nonce_ready_report(),
            payload_hash: "reviewed-payload-hash-only".to_owned(),
            router_label: "uniswap-v3-router-reviewed".to_owned(),
            spender_label: "uniswap-v3-spender-reviewed".to_owned(),
            max_gas_fee_quote: 0.25,
            raw_calldata_embedded: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: 1_700_000_375,
        }
    }

    fn web3_unsigned_payload_ready_report() -> Web3UnsignedPayloadReviewReport {
        web3_unsigned_payload_ready_request().review()
    }

    fn web3_broadcast_readiness_ready_request() -> Web3BroadcastReadinessRequest {
        Web3BroadcastReadinessRequest {
            id: "web3-broadcast-readiness-ready".to_owned(),
            unsigned_payload_review: web3_unsigned_payload_ready_report(),
            pre_sign_safety_review: web3_pre_sign_ready_report(),
            signer_authorization_reference: "signer-authorization-ref-local".to_owned(),
            live_adapter_reference: "live-adapter-ref-deferred".to_owned(),
            operator_approval_reference: "operator-approval-ref-local".to_owned(),
            broadcast_allowed: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: 1_700_000_425,
        }
    }

    fn web3_broadcast_readiness_ready_report() -> Web3BroadcastReadinessReport {
        web3_broadcast_readiness_ready_request().review()
    }

    fn web3_unsigned_transaction_construction_ready_request(
    ) -> Web3UnsignedTransactionConstructionRequest {
        Web3UnsignedTransactionConstructionRequest {
            id: "web3-unsigned-transaction-ready".to_owned(),
            broadcast_readiness_review: web3_broadcast_readiness_ready_report(),
            payload_hash: "reviewed-payload-hash-only".to_owned(),
            function_selector: "swap-exact-input-single".to_owned(),
            encoded_argument_digest: "encoded-argument-digest-only".to_owned(),
            target_contract_label: "uniswap-v3-router-reviewed".to_owned(),
            nonce: Some(7),
            gas_limit: 150_000,
            max_fee_quote: 0.25,
            raw_calldata_embedded: false,
            raw_transaction_serialized: false,
            broadcast_allowed: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            constructed_at_unix_ms: 1_700_000_475,
        }
    }

    fn web3_unsigned_transaction_construction_ready_report(
    ) -> Web3UnsignedTransactionConstructionReport {
        web3_unsigned_transaction_construction_ready_request().construct()
    }

    fn web3_provider_nonce_reconciliation_ready_request() -> Web3ProviderNonceReconciliationRequest
    {
        Web3ProviderNonceReconciliationRequest {
            id: "web3-provider-nonce-ready".to_owned(),
            unsigned_transaction_construction: web3_unsigned_transaction_construction_ready_report(
            ),
            provider_snapshot_reference: "provider-nonce-snapshot-local".to_owned(),
            provider_next_nonce: Some(7),
            provider_pending_nonces: vec![8, 9],
            max_snapshot_age_ms: 30_000,
            snapshot_age_ms: 250,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reconciled_at_unix_ms: 1_700_000_525,
        }
    }

    fn web3_provider_nonce_reconciliation_ready_report() -> Web3ProviderNonceReconciliationReport {
        web3_provider_nonce_reconciliation_ready_request().reconcile()
    }

    fn web3_raw_transaction_serialization_review_ready_request(
    ) -> Web3RawTransactionSerializationReviewRequest {
        Web3RawTransactionSerializationReviewRequest {
            id: "web3-raw-transaction-serialization-ready".to_owned(),
            provider_nonce_reconciliation: web3_provider_nonce_reconciliation_ready_report(),
            transaction_type_label: "eip1559-local-review".to_owned(),
            chain_id_reference: "chain-id-1".to_owned(),
            fee_field_reference: "fee-fields-reviewed".to_owned(),
            access_list_reference: "access-list-empty-reviewed".to_owned(),
            raw_transaction_bytes_embedded: false,
            raw_calldata_embedded: false,
            raw_transaction_serialized: false,
            broadcast_allowed: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: 1_700_000_526,
        }
    }

    fn web3_raw_transaction_serialization_review_ready_report(
    ) -> Web3RawTransactionSerializationReviewReport {
        web3_raw_transaction_serialization_review_ready_request().review()
    }

    fn web3_broadcast_adapter_control_review_ready_request(
    ) -> Web3BroadcastAdapterControlReviewRequest {
        Web3BroadcastAdapterControlReviewRequest {
            id: "web3-broadcast-adapter-control-ready".to_owned(),
            raw_transaction_serialization_review:
                web3_raw_transaction_serialization_review_ready_report(),
            adapter_reference: "local-broadcast-adapter-reviewed".to_owned(),
            operator_approval_reference: "operator-approval-reference-reviewed".to_owned(),
            audit_state_preflight_reference: "audit-state-preflight-reviewed".to_owned(),
            kill_switch_confirmed: true,
            rate_limit_control_ready: true,
            replay_protection_ready: true,
            broadcast_permission_granted: false,
            raw_transaction_bytes_embedded: false,
            raw_transaction_serialized: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            reviewed_at_unix_ms: 1_700_000_527,
        }
    }

    fn web3_broadcast_adapter_control_review_ready_report(
    ) -> Web3BroadcastAdapterControlReviewReport {
        web3_broadcast_adapter_control_review_ready_request().review()
    }

    fn web3_sandbox_live_discrepancy_calibration_ready_request(
    ) -> Web3SandboxLiveDiscrepancyCalibrationRequest {
        Web3SandboxLiveDiscrepancyCalibrationRequest {
            id: "web3-sandbox-live-calibration-ready".to_owned(),
            broadcast_adapter_control_review: web3_broadcast_adapter_control_review_ready_report(),
            sandbox_observation_reference: "sandbox-observation-reviewed".to_owned(),
            live_observation_reference: "live-observation-reviewed".to_owned(),
            max_price_deviation_bps: 25.0,
            observed_price_deviation_bps: 12.5,
            max_latency_deviation_ms: 250,
            observed_latency_deviation_ms: 125,
            max_fee_deviation_quote: 0.05,
            observed_fee_deviation_quote: 0.02,
            minimum_sample_count: 3,
            sandbox_sample_count: 5,
            live_sample_count: 4,
            external_call_performed: false,
            credential_loaded: false,
            rpc_called: false,
            signer_material_loaded: false,
            signing_performed: false,
            broadcast_performed: false,
            live_execution_performed: false,
            production_ready: false,
            calibrated_at_unix_ms: 1_700_000_528,
        }
    }

    fn web3_sandbox_live_discrepancy_calibration_ready_report(
    ) -> Web3SandboxLiveDiscrepancyCalibrationReport {
        web3_sandbox_live_discrepancy_calibration_ready_request().calibrate()
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
    fn registry_rejects_duplicate_tokens() {
        let chain =
            Web3ChainProfile::new("ethereum", "Ethereum", "ETH").expect("chain should validate");
        let token_profile = DexTokenProfile::new("ethereum", "ETH", "native-eth", 18)
            .expect("token should validate");
        let error = DexConnectorRegistry::new(
            vec![chain],
            vec![router_profile()],
            vec![token_profile.clone(), token_profile],
        )
        .expect_err("duplicate tokens must be rejected");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_DUPLICATE_TOKEN"));
    }

    #[test]
    fn policy_gate_approves_paper_dex_swap_quote() {
        let approval = policy_gate()
            .validate_swap_quote(&router_profile(), &swap_request())
            .expect("paper DEX swap quote should be policy approved");
        assert_eq!(approval.intent_id, "dex-swap-1");
        assert_eq!(approval.approved_scope, ExecutionScope::Paper);
    }

    #[test]
    fn policy_gate_rejects_live_scope_in_phase_8() {
        let mut request = swap_request();
        request.scope = ExecutionScope::Live;
        let error = policy_gate()
            .validate_swap_quote(&router_profile(), &request)
            .expect_err("live DEX swap must be blocked in Phase 8");
        assert!(matches!(error, DexConnectorError::LiveSwapsUnavailable));
    }

    #[test]
    fn capability_validation_rejects_unsupported_exact_output() {
        let mut request = swap_request();
        request.mode = DexSwapMode::ExactOutput;
        let error = policy_gate()
            .validate_swap_quote(&router_profile(), &request)
            .expect_err("unsupported exact-output swap must be rejected");
        assert!(matches!(
            error,
            DexConnectorError::CapabilityUnavailable {
                capability: "exact-output-swaps",
                ..
            }
        ));
    }

    #[test]
    fn local_simulation_validation_does_not_broadcast() {
        let response = policy_gate()
            .validate_simulation_request(&router_profile(), &simulation_request())
            .expect("local simulation validation should succeed");
        assert_eq!(response.status, DexSimulationStatus::LocallyValidated);
        assert!(!response.broadcastable);
    }

    #[test]
    fn local_dex_adapter_quotes_and_simulates_without_rpc_or_broadcast() {
        let adapter = local_adapter();

        let quote = adapter
            .quote_swap(&swap_request())
            .expect("local quote should validate through policy");
        let simulation = adapter
            .simulate_transaction(&simulation_request())
            .expect("local simulation fixture should validate");
        let fees = adapter
            .fee_schedule(
                &venue(),
                Some(&MarketPair::new("ETH", "USDC").expect("pair should validate")),
            )
            .expect("fee fixture should be returned");

        assert_eq!(quote.id, "dex-quote-response-1");
        assert_eq!(simulation.status, DexSimulationStatus::LocallyValidated);
        assert!(!simulation.broadcastable);
        assert!((fees.taker_bps - 30.0).abs() < f64::EPSILON);
        assert!(!adapter.rpc_called());
        assert!(!adapter.signer_material_loaded());
        assert!(!adapter.signing_performed());
        assert!(!adapter.broadcast_performed());
        assert!(!adapter.bridge_performed());
    }

    #[test]
    fn local_dex_adapter_rejects_live_swap_before_quote() {
        let adapter = local_adapter();
        let mut request = swap_request();
        request.scope = ExecutionScope::Live;

        let error = adapter
            .quote_swap(&request)
            .expect_err("live swap quote must be rejected");

        assert!(matches!(error, DexConnectorError::LiveSwapsUnavailable));
        assert!(!adapter.rpc_called());
        assert!(!adapter.signing_performed());
        assert!(!adapter.broadcast_performed());
    }

    #[test]
    fn local_dex_adapter_rejects_live_simulation_before_rpc() {
        let adapter = local_adapter();
        let mut request = simulation_request();
        request.scope = ExecutionScope::Live;

        let error = adapter
            .simulate_transaction(&request)
            .expect_err("live simulation must be rejected");

        assert!(matches!(
            error,
            DexConnectorError::LiveSimulationUnavailable
        ));
        assert!(!adapter.rpc_called());
        assert!(!adapter.signing_performed());
        assert!(!adapter.broadcast_performed());
    }

    #[test]
    fn dex_request_plans_validate_without_network_or_signing() {
        let plans = request_plans();
        assert_eq!(plans.len(), 4);
        assert_eq!(
            plans
                .iter()
                .filter(|plan| plan.request_kind == DexRequestPlanKind::HttpQuote)
                .count(),
            1
        );
        assert_eq!(
            plans
                .iter()
                .filter(|plan| plan.request_kind == DexRequestPlanKind::SolanaHttpQuote)
                .count(),
            1
        );
        assert_eq!(
            plans
                .iter()
                .filter(|plan| plan.request_kind == DexRequestPlanKind::RpcQuoteCall)
                .count(),
            1
        );
        assert_eq!(
            plans
                .iter()
                .filter(|plan| plan.request_kind == DexRequestPlanKind::RpcSimulationCall)
                .count(),
            1
        );
        for plan in plans {
            plan.validate().expect("request plan should validate");
            assert!(!plan.http_call_performed);
            assert!(!plan.rpc_call_performed);
            assert!(!plan.credentials_loaded);
            assert!(!plan.signing_performed);
            assert!(!plan.broadcast_performed);
            assert!(!plan.bridge_performed);
            assert!(!plan.live_execution_performed);
            assert!(!plan.production_ready);
        }
    }

    #[test]
    fn dex_request_plans_convert_to_existing_local_requests() {
        let plans = request_plans();
        let quote_requests = plans
            .iter()
            .filter(|plan| plan.request_kind != DexRequestPlanKind::RpcSimulationCall)
            .map(|plan| {
                plan.to_local_quote_request(
                    format!("{}-quote", plan.id),
                    "strategy-dex-plan",
                    1.0,
                    100.0,
                    100.0,
                )
                .expect("quote-capable plan should create local quote request")
            })
            .collect::<Vec<_>>();
        assert_eq!(quote_requests.len(), 3);
        assert!(quote_requests
            .iter()
            .all(|request| request.scope == ExecutionScope::Paper));

        let simulation_request = plans
            .iter()
            .find(|plan| plan.request_kind == DexRequestPlanKind::RpcSimulationCall)
            .expect("simulation plan should exist")
            .to_local_simulation_request("sim-plan-1", "swap-plan-1", 1.0, 99.0)
            .expect("simulation plan should create local simulation request");
        assert_eq!(simulation_request.scope, ExecutionScope::Paper);
        assert_eq!(
            simulation_request.payload_hash,
            "reviewed-local-payload-hash-only"
        );
    }

    #[test]
    fn dex_request_plan_rejects_side_effect_flags_and_wrong_conversion() {
        let mut plan = DexRequestPlan::zero_ex_swap_quote_http(
            venue(),
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            "ethereum",
        )
        .expect("plan should validate");
        plan.http_call_performed = true;
        let error = plan
            .validate()
            .expect_err("side-effect flags must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_REQUEST_PLAN_SIDE_EFFECT_FLAG"));

        let plan = DexRequestPlan::zero_ex_swap_quote_http(
            venue(),
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            "ethereum",
        )
        .expect("plan should validate");
        let error = plan
            .to_local_simulation_request("sim-plan-1", "swap-plan-1", 1.0, 99.0)
            .expect_err("quote plans must not produce simulation requests");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_REQUEST_PLAN_NOT_SIMULATION_CAPABLE"));
    }

    #[test]
    fn dex_response_transcripts_parse_quote_shapes_locally() {
        let plans = request_plans();
        let quote_plans = plans
            .iter()
            .filter(|plan| plan.request_kind != DexRequestPlanKind::RpcSimulationCall)
            .collect::<Vec<_>>();
        let payloads = [
            r#"{"amountIn":"1.0","amountOut":"1900.0","priceImpactBps":4.0,"estimatedFeeQuote":0.1,"gasFeeQuote":0.25}"#,
            r#"{"sellAmount":"1.0","buyAmount":"1899.5","priceImpactBps":"5.0","feeQuote":"0.2","estimatedGasQuote":"0.3"}"#,
            r#"{"inAmount":"1","outAmount":"100","priceImpactBps":6,"marketDataAgeMs":500}"#,
        ];
        for (index, plan) in quote_plans.iter().enumerate() {
            let transcript = DexResponseTranscript::local(
                format!("dex-quote-transcript-{index}"),
                format!("dex-quote-request-{index}"),
                plan.request_kind,
                plan.protocol_label.clone(),
                plan.venue.clone(),
                plan.chain.clone(),
                plan.pair.clone(),
                payloads[index],
            )
            .expect("local quote transcript should validate");
            let response = plan
                .parse_quote_transcript(&transcript)
                .expect("local quote transcript should parse");
            assert_eq!(response.request_id, format!("dex-quote-request-{index}"));
            assert!(response.amount_in > 0.0);
            assert!(response.amount_out > 0.0);
            assert_eq!(
                response.simulation_status,
                DexSimulationStatus::LocallyValidated
            );
        }
    }

    #[test]
    fn dex_response_transcript_parses_simulation_locally() {
        let plan = DexRequestPlan::evm_transaction_simulation_eth_call(
            venue(),
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            "ethereum",
        )
        .expect("simulation request plan should validate");
        let transcript = DexResponseTranscript::local(
            "dex-simulation-transcript",
            "dex-simulation-request",
            plan.request_kind,
            plan.protocol_label.clone(),
            plan.venue.clone(),
            plan.chain.clone(),
            plan.pair.clone(),
            r#"{"status":"success","gasUsed":"142000","gasFeeQuote":"0.24","amountOut":"1898.0","diagnostic":"local simulation fixture"}"#,
        )
        .expect("local simulation transcript should validate");
        let response = plan
            .parse_simulation_transcript(&transcript)
            .expect("local simulation transcript should parse");
        assert_eq!(response.status, DexSimulationStatus::WouldSucceed);
        assert_eq!(response.gas_used, 142_000);
        assert!(!response.broadcastable);
    }

    #[test]
    fn dex_response_transcript_rejects_side_effects_and_mismatch() {
        let plan = DexRequestPlan::zero_ex_swap_quote_http(
            venue(),
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            "ethereum",
        )
        .expect("quote request plan should validate");
        let mut transcript = DexResponseTranscript::local(
            "dex-bad-transcript",
            "dex-bad-request",
            plan.request_kind,
            plan.protocol_label.clone(),
            plan.venue.clone(),
            plan.chain.clone(),
            plan.pair.clone(),
            r#"{"sellAmount":"1.0","buyAmount":"1900.0"}"#,
        )
        .expect("local transcript should validate");
        transcript.rpc_response_received = true;
        let error = plan
            .parse_quote_transcript(&transcript)
            .expect_err("side-effect transcript must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_RESPONSE_TRANSCRIPT_SIDE_EFFECT_FLAG"));

        let transcript = DexResponseTranscript::local(
            "dex-mismatch-transcript",
            "dex-mismatch-request",
            DexRequestPlanKind::SolanaHttpQuote,
            plan.protocol_label.clone(),
            plan.venue.clone(),
            plan.chain.clone(),
            plan.pair.clone(),
            r#"{"sellAmount":"1.0","buyAmount":"1900.0"}"#,
        )
        .expect("local transcript should validate");
        let error = plan
            .parse_quote_transcript(&transcript)
            .expect_err("kind mismatch must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_RESPONSE_TRANSCRIPT_KIND_MISMATCH"));
    }

    #[test]
    fn web3_transaction_lifecycle_transcripts_parse_local_statuses() {
        let evm_transcript = Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-evm",
            "web3-request-evm",
            "ethereum",
            venue(),
            Web3TransactionLifecycleTranscriptFormat::EvmTransactionReceipt,
            r#"{"transactionHash":"0xabc123","status":"0x1","blockNumber":"19000001","confirmations":"12","nonce":"7","gasUsed":"142000"}"#,
        )
        .expect("EVM lifecycle transcript should validate");
        let evm_record = evm_transcript
            .parse_record()
            .expect("EVM lifecycle transcript should parse");
        assert_eq!(evm_record.status, Web3TransactionLifecycleStatus::Confirmed);
        assert_eq!(evm_record.nonce, Some(7));
        assert_eq!(evm_record.block_number, Some(19_000_001));
        assert_eq!(evm_record.confirmations, 12);
        assert!(!evm_record.rpc_call_performed);
        assert!(!evm_record.signing_performed);
        assert!(!evm_record.broadcast_performed);

        let solana_transcript = Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-solana",
            "web3-request-solana",
            "solana",
            VenueRef {
                name: "paper-jupiter".to_owned(),
                kind: VenueKind::Dex,
            },
            Web3TransactionLifecycleTranscriptFormat::SolanaSignatureStatus,
            r#"{"signature":"5abc","slot":"250000000","confirmations":32,"confirmationStatus":"finalized","err":null}"#,
        )
        .expect("Solana lifecycle transcript should validate");
        let solana_record = solana_transcript
            .parse_record()
            .expect("Solana lifecycle transcript should parse");
        assert_eq!(
            solana_record.status,
            Web3TransactionLifecycleStatus::Confirmed
        );
        assert_eq!(solana_record.slot, Some(250_000_000));
        assert_eq!(solana_record.confirmations, 32);
    }

    #[test]
    fn web3_transaction_lifecycle_transcripts_fail_closed_on_side_effects_and_bad_confirmation() {
        let mut transcript = Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-side-effect",
            "web3-request-side-effect",
            "ethereum",
            venue(),
            Web3TransactionLifecycleTranscriptFormat::EvmTransactionReceipt,
            r#"{"transactionHash":"0xabc123","status":"0x1","confirmations":1}"#,
        )
        .expect("local transcript should validate");
        transcript.rpc_response_received = true;
        let error = transcript
            .parse_record()
            .expect_err("side-effect transcript must fail closed");
        assert!(error.violations().iter().any(|violation| {
            violation.code() == "WEB3_TRANSACTION_LIFECYCLE_SIDE_EFFECT_FLAG"
        }));

        let transcript = Web3TransactionLifecycleTranscript::local(
            "web3-lifecycle-bad-confirmation",
            "web3-request-bad-confirmation",
            "solana",
            VenueRef {
                name: "paper-jupiter".to_owned(),
                kind: VenueKind::Dex,
            },
            Web3TransactionLifecycleTranscriptFormat::SolanaSignatureStatus,
            r#"{"signature":"5abc","confirmationStatus":"confirmed","err":null}"#,
        )
        .expect("local transcript should validate");
        let error = transcript
            .parse_record()
            .expect_err("confirmed status without local confirmation evidence must fail closed");
        assert!(error.violations().iter().any(|violation| {
            violation.code() == "WEB3_TRANSACTION_LIFECYCLE_CONFIRMATION_MISSING"
        }));
    }

    #[test]
    fn web3_nonce_reservation_accepts_next_local_nonce() {
        let report = web3_nonce_ready_report();

        assert_eq!(
            report.status,
            Web3NonceReservationStatus::ReservedForLocalReview
        );
        assert!(report.nonce_ready);
        assert_eq!(report.reserved_nonce, Some(7));
        assert_eq!(report.last_confirmed_nonce, Some(6));
        assert_eq!(report.in_flight_nonce_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report.validate().expect("nonce reservation validates");
    }

    #[test]
    fn web3_nonce_reservation_blocks_stale_and_duplicate_nonce() {
        let mut request = web3_nonce_ready_request();
        request.id = "web3-nonce-blocked".to_owned();
        request.last_confirmed_nonce = Some(7);
        request.requested_nonce = Some(7);
        request.in_flight_nonces = vec![7, 8, 8];

        let report = request.reserve();

        assert_eq!(report.status, Web3NonceReservationStatus::Blocked);
        assert!(!report.nonce_ready);
        assert_eq!(report.reserved_nonce, None);
        for expected in [
            "WEB3_NONCE_IN_FLIGHT_DUPLICATE",
            "WEB3_NONCE_STALE",
            "WEB3_NONCE_ALREADY_RESERVED",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report.validate().expect("blocked nonce report validates");
    }

    #[test]
    fn web3_nonce_reservation_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_nonce_ready_request();
        request.id = "web3-nonce-side-effect".to_owned();
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.reserve();

        assert_eq!(report.status, Web3NonceReservationStatus::Blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_NONCE_SIDE_EFFECT_FLAG"));
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect nonce report validates");
    }

    #[test]
    fn web3_nonce_reservation_audit_and_state_reopen_locally() {
        let report = web3_nonce_ready_report();
        let audit_path = temp_path("web3-nonce-reservation-audit", "jsonl");
        let state_path = temp_path("web3-nonce-reservation-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_nonce_reservation_audit(&mut journal, &report).expect("audit appends");
        let checkpoint = persist_web3_nonce_reservation_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("signing_performed"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("reserved_nonce"),
            Some(crate::AuditValue::Unsigned(7))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_NONCE_RESERVATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3NonceReservationReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_unsigned_payload_review_accepts_local_payload_hash_and_nonce() {
        let report = web3_unsigned_payload_ready_report();

        assert_eq!(
            report.status,
            Web3UnsignedPayloadReviewStatus::ReadyForLocalReview
        );
        assert!(report.nonce_ready);
        assert!(report.payload_reference_ready);
        assert!(report.router_spender_ready);
        assert!(report.gas_cap_ready);
        assert_eq!(report.reserved_nonce, Some(7));
        assert_eq!(report.payload_hash, "reviewed-payload-hash-only");
        assert_eq!(report.violation_count, 0);
        assert!(!report.raw_calldata_embedded);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report.validate().expect("unsigned payload validates");
    }

    #[test]
    fn web3_unsigned_payload_review_blocks_nonce_payload_and_router_mismatch() {
        let mut request = web3_unsigned_payload_ready_request();
        request.id = "web3-unsigned-payload-blocked".to_owned();
        request.nonce_reservation = {
            let mut nonce = web3_nonce_ready_request();
            nonce.requested_nonce = Some(6);
            nonce.reserve()
        };
        request.payload_hash = "0xrawcalldata".to_owned();
        request.router_label = "wrong-router".to_owned();
        request.max_gas_fee_quote = 9.0;

        let report = request.review();

        assert_eq!(report.status, Web3UnsignedPayloadReviewStatus::Blocked);
        assert!(!report.nonce_ready);
        assert!(!report.payload_reference_ready);
        assert!(!report.router_spender_ready);
        assert!(!report.gas_cap_ready);
        for expected in [
            "WEB3_UNSIGNED_PAYLOAD_NONCE_NOT_READY",
            "WEB3_UNSIGNED_PAYLOAD_REFERENCE_INVALID",
            "WEB3_UNSIGNED_PAYLOAD_ROUTER_SPENDER_MISMATCH",
            "WEB3_UNSIGNED_PAYLOAD_GAS_CAP_INVALID",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report.validate().expect("blocked payload report validates");
    }

    #[test]
    fn web3_unsigned_payload_review_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_unsigned_payload_ready_request();
        request.id = "web3-unsigned-payload-side-effect".to_owned();
        request.raw_calldata_embedded = true;
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.review();

        assert_eq!(report.status, Web3UnsignedPayloadReviewStatus::Blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_UNSIGNED_PAYLOAD_SIDE_EFFECT_FLAG"));
        assert!(!report.raw_calldata_embedded);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect payload report validates");
    }

    #[test]
    fn web3_unsigned_payload_review_audit_and_state_reopen_locally() {
        let report = web3_unsigned_payload_ready_report();
        let audit_path = temp_path("web3-unsigned-payload-audit", "jsonl");
        let state_path = temp_path("web3-unsigned-payload-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_web3_unsigned_payload_review_audit(&mut journal, &report)
            .expect("audit appends");
        let checkpoint = persist_web3_unsigned_payload_review_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("raw_calldata_embedded"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("reserved_nonce"),
            Some(crate::AuditValue::Unsigned(7))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_UNSIGNED_PAYLOAD_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3UnsignedPayloadReviewReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_pre_sign_safety_accepts_local_simulation_and_nonce_metadata() {
        let report = web3_pre_sign_ready_report();

        assert_eq!(
            report.status,
            Web3PreSignSafetyReviewStatus::ReadyForLocalReview
        );
        assert!(report.simulation_success_ready);
        assert!(report.gas_fee_within_cap);
        assert!(report.output_amount_sufficient);
        assert!(report.nonce_ready);
        assert!(report.lifecycle_coherent);
        assert_eq!(report.violation_count, 0);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report.validate().expect("pre-sign report validates");
    }

    #[test]
    fn web3_pre_sign_safety_blocks_revert_missing_nonce_and_expensive_gas() {
        let mut request = web3_pre_sign_ready_request();
        request.id = "web3-pre-sign-blocked".to_owned();
        request.simulation_response.status = DexSimulationStatus::WouldRevert;
        request.simulation_response.gas_fee_quote = 9.0;
        request.simulation_response.amount_out = 10.0;
        request.planned_nonce = None;

        let report = request.review();

        assert_eq!(report.status, Web3PreSignSafetyReviewStatus::Blocked);
        for expected in [
            "WEB3_PRE_SIGN_SIMULATION_NOT_READY",
            "WEB3_PRE_SIGN_GAS_FEE_EXCEEDS_CAP",
            "WEB3_PRE_SIGN_OUTPUT_BELOW_MINIMUM",
            "WEB3_PRE_SIGN_NONCE_REQUIRED",
            "WEB3_PRE_SIGN_LIFECYCLE_INCOHERENT",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        assert!(!report.simulation_success_ready);
        assert!(!report.gas_fee_within_cap);
        assert!(!report.output_amount_sufficient);
        assert!(!report.nonce_ready);
        assert!(!report.lifecycle_coherent);
        report.validate().expect("blocked report validates");
    }

    #[test]
    fn web3_pre_sign_safety_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_pre_sign_ready_request();
        request.id = "web3-pre-sign-side-effect".to_owned();
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.review();

        assert_eq!(report.status, Web3PreSignSafetyReviewStatus::Blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_PRE_SIGN_SIDE_EFFECT_FLAG"));
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report.validate().expect("side-effect report validates");
    }

    #[test]
    fn web3_pre_sign_safety_audit_and_state_reopen_locally() {
        let report = web3_pre_sign_ready_report();
        let audit_path = temp_path("web3-pre-sign-safety-audit", "jsonl");
        let state_path = temp_path("web3-pre-sign-safety-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_pre_sign_safety_audit(&mut journal, &report).expect("audit appends");
        let checkpoint = persist_web3_pre_sign_safety_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY);
        assert!(matches!(
            audit_record.event.metadata.get("signing_performed"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("nonce_ready"),
            Some(crate::AuditValue::Bool(true))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_PRE_SIGN_SAFETY_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3PreSignSafetyReviewReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_broadcast_readiness_accepts_local_prerequisites_without_allowing_broadcast() {
        let report = web3_broadcast_readiness_ready_report();

        assert_eq!(
            report.status,
            Web3BroadcastReadinessStatus::ReadyForExternalReview
        );
        assert!(report.unsigned_payload_ready);
        assert!(report.pre_sign_safety_ready);
        assert!(report.signer_authorization_reference_ready);
        assert!(report.live_adapter_reference_ready);
        assert!(report.operator_approval_reference_ready);
        assert_eq!(report.violation_count, 0);
        assert!(!report.broadcast_allowed);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("broadcast readiness report validates");
    }

    #[test]
    fn web3_broadcast_readiness_blocks_incomplete_references_and_mismatched_prerequisites() {
        let mut request = web3_broadcast_readiness_ready_request();
        request.id = "web3-broadcast-readiness-blocked".to_owned();
        request.pre_sign_safety_review.simulation_request_id = "different-request".to_owned();
        request.signer_authorization_reference.clear();
        request.live_adapter_reference = "0x-live-adapter".to_owned();
        request.operator_approval_reference.clear();

        let report = request.review();

        assert_eq!(report.status, Web3BroadcastReadinessStatus::Blocked);
        assert!(!report.signer_authorization_reference_ready);
        assert!(!report.live_adapter_reference_ready);
        assert!(!report.operator_approval_reference_ready);
        for expected in [
            "WEB3_BROADCAST_READINESS_PREREQUISITE_MISMATCH",
            "WEB3_BROADCAST_READINESS_SIGNER_AUTHORIZATION_REFERENCE_REQUIRED",
            "WEB3_BROADCAST_READINESS_LIVE_ADAPTER_REFERENCE_REQUIRED",
            "WEB3_BROADCAST_READINESS_OPERATOR_APPROVAL_REFERENCE_REQUIRED",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        assert!(!report.broadcast_allowed);
        report
            .validate()
            .expect("blocked broadcast readiness report validates");
    }

    #[test]
    fn web3_broadcast_readiness_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_broadcast_readiness_ready_request();
        request.id = "web3-broadcast-readiness-side-effect".to_owned();
        request.broadcast_allowed = true;
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.review();

        assert_eq!(report.status, Web3BroadcastReadinessStatus::Blocked);
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_BROADCAST_READINESS_SIDE_EFFECT_FLAG"));
        assert!(!report.broadcast_allowed);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect broadcast readiness report validates");
    }

    #[test]
    fn web3_broadcast_readiness_audit_and_state_reopen_locally() {
        let report = web3_broadcast_readiness_ready_report();
        let audit_path = temp_path("web3-broadcast-readiness-audit", "jsonl");
        let state_path = temp_path("web3-broadcast-readiness-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_broadcast_readiness_audit(&mut journal, &report).expect("audit appends");
        let checkpoint = persist_web3_broadcast_readiness_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("broadcast_allowed"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("unsigned_payload_ready"),
            Some(crate::AuditValue::Bool(true))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_BROADCAST_READINESS_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3BroadcastReadinessReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_unsigned_transaction_construction_accepts_local_metadata_without_broadcastability() {
        let report = web3_unsigned_transaction_construction_ready_report();

        assert_eq!(
            report.status,
            Web3UnsignedTransactionConstructionStatus::ConstructedForLocalReview
        );
        assert!(report.broadcast_readiness_ready);
        assert!(report.payload_reference_ready);
        assert!(report.target_selector_ready);
        assert!(report.nonce_ready);
        assert!(report.gas_metadata_ready);
        assert_eq!(report.violation_count, 0);
        assert!(!report.raw_calldata_embedded);
        assert!(!report.raw_transaction_serialized);
        assert!(!report.broadcast_allowed);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("unsigned transaction construction validates");
    }

    #[test]
    fn web3_unsigned_transaction_construction_blocks_bad_metadata() {
        let mut request = web3_unsigned_transaction_construction_ready_request();
        request.id = "web3-unsigned-transaction-blocked".to_owned();
        request.broadcast_readiness_review = {
            let mut blocked = web3_broadcast_readiness_ready_request();
            blocked.operator_approval_reference.clear();
            blocked.review()
        };
        request.payload_hash = "0xrawcalldata".to_owned();
        request.function_selector.clear();
        request.nonce = None;
        request.gas_limit = 0;

        let report = request.construct();

        assert_eq!(
            report.status,
            Web3UnsignedTransactionConstructionStatus::Blocked
        );
        assert!(!report.broadcast_readiness_ready);
        assert!(!report.payload_reference_ready);
        assert!(!report.target_selector_ready);
        assert!(!report.nonce_ready);
        assert!(!report.gas_metadata_ready);
        for expected in [
            "WEB3_UNSIGNED_TX_BROADCAST_READINESS_NOT_READY",
            "WEB3_UNSIGNED_TX_PAYLOAD_REFERENCE_INVALID",
            "WEB3_UNSIGNED_TX_TARGET_SELECTOR_INVALID",
            "WEB3_UNSIGNED_TX_NONCE_REQUIRED",
            "WEB3_UNSIGNED_TX_GAS_METADATA_INVALID",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report
            .validate()
            .expect("blocked unsigned transaction construction validates");
    }

    #[test]
    fn web3_unsigned_transaction_construction_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_unsigned_transaction_construction_ready_request();
        request.id = "web3-unsigned-transaction-side-effect".to_owned();
        request.raw_calldata_embedded = true;
        request.raw_transaction_serialized = true;
        request.broadcast_allowed = true;
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.construct();

        assert_eq!(
            report.status,
            Web3UnsignedTransactionConstructionStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_UNSIGNED_TX_SIDE_EFFECT_FLAG"));
        assert!(!report.raw_calldata_embedded);
        assert!(!report.raw_transaction_serialized);
        assert!(!report.broadcast_allowed);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect unsigned transaction construction validates");
    }

    #[test]
    fn web3_unsigned_transaction_construction_audit_and_state_reopen_locally() {
        let report = web3_unsigned_transaction_construction_ready_report();
        let audit_path = temp_path("web3-unsigned-transaction-audit", "jsonl");
        let state_path = temp_path("web3-unsigned-transaction-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_unsigned_transaction_construction_audit(&mut journal, &report)
                .expect("audit appends");
        let checkpoint =
            persist_web3_unsigned_transaction_construction_checkpoint(&mut store, &report)
                .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("raw_calldata_embedded"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("broadcast_allowed"),
            Some(crate::AuditValue::Bool(false))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_UNSIGNED_TRANSACTION_CONSTRUCTION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3UnsignedTransactionConstructionReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_provider_nonce_reconciliation_accepts_matching_local_snapshot() {
        let report = web3_provider_nonce_reconciliation_ready_report();

        assert_eq!(
            report.status,
            Web3ProviderNonceReconciliationStatus::ReconciledForLocalReview
        );
        assert!(report.unsigned_transaction_ready);
        assert!(report.provider_snapshot_reference_ready);
        assert!(report.provider_next_nonce_ready);
        assert!(report.construction_nonce_matches_provider);
        assert!(report.construction_nonce_not_pending);
        assert!(report.pending_nonce_set_unique);
        assert!(report.snapshot_fresh);
        assert_eq!(report.construction_nonce, Some(7));
        assert_eq!(report.provider_next_nonce, Some(7));
        assert_eq!(report.provider_pending_nonce_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("provider nonce reconciliation validates");
    }

    #[test]
    fn web3_provider_nonce_reconciliation_blocks_nonce_conflicts() {
        let mut request = web3_provider_nonce_reconciliation_ready_request();
        request.id = "web3-provider-nonce-blocked".to_owned();
        request.provider_snapshot_reference = "0xprovider".to_owned();
        request.provider_next_nonce = Some(8);
        request.provider_pending_nonces = vec![7, 8, 8];
        request.snapshot_age_ms = 60_000;

        let report = request.reconcile();

        assert_eq!(
            report.status,
            Web3ProviderNonceReconciliationStatus::Blocked
        );
        assert!(!report.provider_snapshot_reference_ready);
        assert!(!report.construction_nonce_matches_provider);
        assert!(!report.construction_nonce_not_pending);
        assert!(!report.pending_nonce_set_unique);
        assert!(!report.snapshot_fresh);
        for expected in [
            "WEB3_PROVIDER_NONCE_SNAPSHOT_REFERENCE_INVALID",
            "WEB3_PROVIDER_NONCE_CONSTRUCTION_NONCE_MISMATCH",
            "WEB3_PROVIDER_NONCE_ALREADY_PENDING",
            "WEB3_PROVIDER_NONCE_PENDING_DUPLICATE",
            "WEB3_PROVIDER_NONCE_SNAPSHOT_STALE",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report
            .validate()
            .expect("blocked provider nonce reconciliation validates");
    }

    #[test]
    fn web3_provider_nonce_reconciliation_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_provider_nonce_reconciliation_ready_request();
        request.id = "web3-provider-nonce-side-effect".to_owned();
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.reconcile();

        assert_eq!(
            report.status,
            Web3ProviderNonceReconciliationStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_PROVIDER_NONCE_SIDE_EFFECT_FLAG"));
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect provider nonce reconciliation validates");
    }

    #[test]
    fn web3_provider_nonce_reconciliation_audit_and_state_reopen_locally() {
        let report = web3_provider_nonce_reconciliation_ready_report();
        let audit_path = temp_path("web3-provider-nonce-audit", "jsonl");
        let state_path = temp_path("web3-provider-nonce-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_web3_provider_nonce_reconciliation_audit(&mut journal, &report)
            .expect("audit appends");
        let checkpoint = persist_web3_provider_nonce_reconciliation_checkpoint(&mut store, &report)
            .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("construction_nonce"),
            Some(crate::AuditValue::Unsigned(7))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("provider_next_nonce"),
            Some(crate::AuditValue::Unsigned(7))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_PROVIDER_NONCE_RECONCILIATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3ProviderNonceReconciliationReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_raw_transaction_serialization_review_accepts_metadata_only_ready_path() {
        let report = web3_raw_transaction_serialization_review_ready_report();

        assert_eq!(
            report.status,
            Web3RawTransactionSerializationReviewStatus::ReadyForExternalReview
        );
        assert!(report.provider_nonce_reconciliation_ready);
        assert!(report.transaction_type_ready);
        assert!(report.chain_id_ready);
        assert!(report.fee_fields_ready);
        assert!(report.access_list_reference_ready);
        assert!(!report.unsigned_transaction_reference.trim().is_empty());
        assert_eq!(report.violation_count, 0);
        assert!(!report.raw_transaction_bytes_embedded);
        assert!(!report.raw_calldata_embedded);
        assert!(!report.raw_transaction_serialized);
        assert!(!report.broadcast_allowed);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("raw transaction serialization review validates");
    }

    #[test]
    fn web3_raw_transaction_serialization_review_blocks_bad_metadata() {
        let mut request = web3_raw_transaction_serialization_review_ready_request();
        request.id = "web3-raw-transaction-serialization-blocked".to_owned();
        request.provider_nonce_reconciliation =
            web3_provider_nonce_reconciliation_ready_request().reconcile();
        request
            .provider_nonce_reconciliation
            .provider_next_nonce_ready = false;
        request.transaction_type_label.clear();
        request.chain_id_reference = "0xrawchain".to_owned();
        request.fee_field_reference = "0xrawfees".to_owned();
        request.access_list_reference = "0xrawaccess".to_owned();

        let report = request.review();

        assert_eq!(
            report.status,
            Web3RawTransactionSerializationReviewStatus::Blocked
        );
        assert!(!report.provider_nonce_reconciliation_ready);
        assert!(!report.transaction_type_ready);
        assert!(!report.chain_id_ready);
        assert!(!report.fee_fields_ready);
        assert!(!report.access_list_reference_ready);
        for expected in [
            "WEB3_RAW_TX_SERIALIZATION_PROVIDER_NONCE_NOT_READY",
            "WEB3_RAW_TX_SERIALIZATION_TYPE_INVALID",
            "WEB3_RAW_TX_SERIALIZATION_CHAIN_ID_INVALID",
            "WEB3_RAW_TX_SERIALIZATION_FEE_FIELDS_INVALID",
            "WEB3_RAW_TX_SERIALIZATION_ACCESS_LIST_INVALID",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report
            .validate()
            .expect("blocked raw transaction serialization review validates");
    }

    #[test]
    fn web3_raw_transaction_serialization_review_blocks_side_effect_flags_without_preserving_them()
    {
        let mut request = web3_raw_transaction_serialization_review_ready_request();
        request.id = "web3-raw-transaction-serialization-side-effect".to_owned();
        request.raw_transaction_bytes_embedded = true;
        request.raw_calldata_embedded = true;
        request.raw_transaction_serialized = true;
        request.broadcast_allowed = true;
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.review();

        assert_eq!(
            report.status,
            Web3RawTransactionSerializationReviewStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_RAW_TX_SERIALIZATION_SIDE_EFFECT_FLAG"));
        assert!(!report.raw_transaction_bytes_embedded);
        assert!(!report.raw_calldata_embedded);
        assert!(!report.raw_transaction_serialized);
        assert!(!report.broadcast_allowed);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect raw transaction serialization review validates");
    }

    #[test]
    fn web3_raw_transaction_serialization_review_audit_and_state_reopen_locally() {
        let report = web3_raw_transaction_serialization_review_ready_report();
        let audit_path = temp_path("web3-raw-transaction-serialization-audit", "jsonl");
        let state_path = temp_path("web3-raw-transaction-serialization-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_raw_transaction_serialization_review_audit(&mut journal, &report)
                .expect("audit appends");
        let checkpoint =
            persist_web3_raw_transaction_serialization_review_checkpoint(&mut store, &report)
                .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("raw_transaction_bytes_embedded"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("raw_transaction_serialized"),
            Some(crate::AuditValue::Bool(false))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_RAW_TRANSACTION_SERIALIZATION_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3RawTransactionSerializationReviewReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_broadcast_adapter_control_review_accepts_non_broadcasting_ready_path() {
        let report = web3_broadcast_adapter_control_review_ready_report();

        assert_eq!(
            report.status,
            Web3BroadcastAdapterControlReviewStatus::ReadyForExternalReview
        );
        assert!(report.raw_transaction_serialization_review_ready);
        assert!(report.adapter_reference_ready);
        assert!(report.operator_approval_reference_ready);
        assert!(report.audit_state_preflight_reference_ready);
        assert!(report.kill_switch_confirmed);
        assert!(report.rate_limit_control_ready);
        assert!(report.replay_protection_ready);
        assert_eq!(report.violation_count, 0);
        assert!(!report.broadcast_permission_granted);
        assert!(!report.raw_transaction_bytes_embedded);
        assert!(!report.raw_transaction_serialized);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("broadcast adapter control review validates");
    }

    #[test]
    fn web3_broadcast_adapter_control_review_blocks_missing_controls() {
        let mut request = web3_broadcast_adapter_control_review_ready_request();
        request.id = "web3-broadcast-adapter-control-blocked".to_owned();
        request.raw_transaction_serialization_review =
            web3_raw_transaction_serialization_review_ready_request().review();
        request
            .raw_transaction_serialization_review
            .fee_fields_ready = false;
        request.adapter_reference = "0xadapter".to_owned();
        request.operator_approval_reference.clear();
        request.audit_state_preflight_reference = "0xaudit".to_owned();
        request.kill_switch_confirmed = false;
        request.rate_limit_control_ready = false;
        request.replay_protection_ready = false;

        let report = request.review();

        assert_eq!(
            report.status,
            Web3BroadcastAdapterControlReviewStatus::Blocked
        );
        assert!(!report.raw_transaction_serialization_review_ready);
        assert!(!report.adapter_reference_ready);
        assert!(!report.operator_approval_reference_ready);
        assert!(!report.audit_state_preflight_reference_ready);
        assert!(!report.kill_switch_confirmed);
        assert!(!report.rate_limit_control_ready);
        assert!(!report.replay_protection_ready);
        for expected in [
            "WEB3_BROADCAST_ADAPTER_RAW_TX_REVIEW_NOT_READY",
            "WEB3_BROADCAST_ADAPTER_REFERENCE_INVALID",
            "WEB3_BROADCAST_ADAPTER_OPERATOR_APPROVAL_INVALID",
            "WEB3_BROADCAST_ADAPTER_AUDIT_STATE_PREFLIGHT_INVALID",
            "WEB3_BROADCAST_ADAPTER_KILL_SWITCH_REQUIRED",
            "WEB3_BROADCAST_ADAPTER_RATE_LIMIT_REQUIRED",
            "WEB3_BROADCAST_ADAPTER_REPLAY_PROTECTION_REQUIRED",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report
            .validate()
            .expect("blocked broadcast adapter control review validates");
    }

    #[test]
    fn web3_broadcast_adapter_control_review_blocks_side_effect_flags_without_preserving_them() {
        let mut request = web3_broadcast_adapter_control_review_ready_request();
        request.id = "web3-broadcast-adapter-control-side-effect".to_owned();
        request.broadcast_permission_granted = true;
        request.raw_transaction_bytes_embedded = true;
        request.raw_transaction_serialized = true;
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.review();

        assert_eq!(
            report.status,
            Web3BroadcastAdapterControlReviewStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_BROADCAST_ADAPTER_SIDE_EFFECT_FLAG"));
        assert!(!report.broadcast_permission_granted);
        assert!(!report.raw_transaction_bytes_embedded);
        assert!(!report.raw_transaction_serialized);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect broadcast adapter control review validates");
    }

    #[test]
    fn web3_broadcast_adapter_control_review_audit_and_state_reopen_locally() {
        let report = web3_broadcast_adapter_control_review_ready_report();
        let audit_path = temp_path("web3-broadcast-adapter-control-audit", "jsonl");
        let state_path = temp_path("web3-broadcast-adapter-control-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_broadcast_adapter_control_review_audit(&mut journal, &report)
                .expect("audit appends");
        let checkpoint =
            persist_web3_broadcast_adapter_control_review_checkpoint(&mut store, &report)
                .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record
                .event
                .metadata
                .get("broadcast_permission_granted"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("broadcast_performed"),
            Some(crate::AuditValue::Bool(false))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_BROADCAST_ADAPTER_CONTROL_REVIEW_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3BroadcastAdapterControlReviewReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn web3_sandbox_live_discrepancy_calibration_accepts_reference_only_ready_path() {
        let report = web3_sandbox_live_discrepancy_calibration_ready_report();

        assert_eq!(
            report.status,
            Web3SandboxLiveDiscrepancyCalibrationStatus::CalibratedForLocalReview
        );
        assert!(report.broadcast_adapter_control_ready);
        assert!(report.sandbox_observation_reference_ready);
        assert!(report.live_observation_reference_ready);
        assert!(report.sample_size_ready);
        assert!(report.price_deviation_within_limit);
        assert!(report.latency_deviation_within_limit);
        assert!(report.fee_deviation_within_limit);
        assert_eq!(report.violation_count, 0);
        assert!(!report.external_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("sandbox/live discrepancy calibration validates");
    }

    #[test]
    fn web3_sandbox_live_discrepancy_calibration_blocks_bad_metadata() {
        let mut request = web3_sandbox_live_discrepancy_calibration_ready_request();
        request.id = "web3-sandbox-live-calibration-blocked".to_owned();
        request.broadcast_adapter_control_review =
            web3_broadcast_adapter_control_review_ready_request().review();
        request
            .broadcast_adapter_control_review
            .rate_limit_control_ready = false;
        request.sandbox_observation_reference = "0xsandbox".to_owned();
        request.live_observation_reference.clear();
        request.minimum_sample_count = 5;
        request.sandbox_sample_count = 2;
        request.live_sample_count = 1;
        request.observed_price_deviation_bps = 30.0;
        request.observed_latency_deviation_ms = 300;
        request.observed_fee_deviation_quote = 0.07;

        let report = request.calibrate();

        assert_eq!(
            report.status,
            Web3SandboxLiveDiscrepancyCalibrationStatus::Blocked
        );
        assert!(!report.broadcast_adapter_control_ready);
        assert!(!report.sandbox_observation_reference_ready);
        assert!(!report.live_observation_reference_ready);
        assert!(!report.sample_size_ready);
        assert!(!report.price_deviation_within_limit);
        assert!(!report.latency_deviation_within_limit);
        assert!(!report.fee_deviation_within_limit);
        for expected in [
            "WEB3_SANDBOX_LIVE_CALIBRATION_BROADCAST_REVIEW_NOT_READY",
            "WEB3_SANDBOX_LIVE_CALIBRATION_SANDBOX_REFERENCE_INVALID",
            "WEB3_SANDBOX_LIVE_CALIBRATION_LIVE_REFERENCE_INVALID",
            "WEB3_SANDBOX_LIVE_CALIBRATION_SAMPLE_COUNT_INSUFFICIENT",
            "WEB3_SANDBOX_LIVE_CALIBRATION_PRICE_DEVIATION_EXCEEDED",
            "WEB3_SANDBOX_LIVE_CALIBRATION_LATENCY_DEVIATION_EXCEEDED",
            "WEB3_SANDBOX_LIVE_CALIBRATION_FEE_DEVIATION_EXCEEDED",
        ] {
            assert!(report
                .violation_codes
                .iter()
                .any(|actual| actual == expected));
        }
        report
            .validate()
            .expect("blocked sandbox/live discrepancy calibration validates");
    }

    #[test]
    fn web3_sandbox_live_discrepancy_calibration_blocks_side_effect_flags_without_preserving_them()
    {
        let mut request = web3_sandbox_live_discrepancy_calibration_ready_request();
        request.id = "web3-sandbox-live-calibration-side-effect".to_owned();
        request.external_call_performed = true;
        request.credential_loaded = true;
        request.rpc_called = true;
        request.signer_material_loaded = true;
        request.signing_performed = true;
        request.broadcast_performed = true;
        request.live_execution_performed = true;
        request.production_ready = true;

        let report = request.calibrate();

        assert_eq!(
            report.status,
            Web3SandboxLiveDiscrepancyCalibrationStatus::Blocked
        );
        assert!(report
            .violation_codes
            .iter()
            .any(|code| code == "WEB3_SANDBOX_LIVE_CALIBRATION_SIDE_EFFECT_FLAG"));
        assert!(!report.external_call_performed);
        assert!(!report.credential_loaded);
        assert!(!report.rpc_called);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
        assert!(!report.live_execution_performed);
        assert!(!report.production_ready);
        report
            .validate()
            .expect("side-effect sandbox/live discrepancy calibration validates");
    }

    #[test]
    fn web3_sandbox_live_discrepancy_calibration_audit_and_state_reopen_locally() {
        let report = web3_sandbox_live_discrepancy_calibration_ready_report();
        let audit_path = temp_path("web3-sandbox-live-calibration-audit", "jsonl");
        let state_path = temp_path("web3-sandbox-live-calibration-state", "sqlite");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_web3_sandbox_live_discrepancy_calibration_audit(&mut journal, &report)
                .expect("audit appends");
        let checkpoint =
            persist_web3_sandbox_live_discrepancy_calibration_checkpoint(&mut store, &report)
                .expect("checkpoint persists");

        assert_eq!(audit_record.sequence, 1);
        assert_eq!(
            checkpoint.key,
            DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY
        );
        assert!(matches!(
            audit_record.event.metadata.get("external_call_performed"),
            Some(crate::AuditValue::Bool(false))
        ));
        assert!(matches!(
            audit_record.event.metadata.get("live_execution_performed"),
            Some(crate::AuditValue::Bool(false))
        ));

        let reopened_journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(reopened_journal.next_sequence(), 2);

        let reopened_store = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let checkpoint = reopened_store
            .get_checkpoint(DEX_LAST_WEB3_SANDBOX_LIVE_DISCREPANCY_CALIBRATION_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("checkpoint exists");
        let recovered: Web3SandboxLiveDiscrepancyCalibrationReport =
            serde_json::from_str(&checkpoint.value).expect("report deserializes");
        assert_eq!(recovered, report);

        let _ = fs::remove_file(audit_path);
        let _ = fs::remove_file(state_path);
    }

    #[test]
    fn dex_protocol_risk_review_passes_local_metadata() {
        let request = DexProtocolRiskReviewRequest::local(
            "dex-protocol-review-ready",
            "uniswap-v3-reviewed",
            venue(),
            "ethereum",
            "uniswap-v3-router-reviewed",
            "uniswap-v3-spender-reviewed",
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            75,
            25.0,
            2.0,
            0.25,
            10.0,
            50.0,
        )
        .expect("local protocol review request should validate");
        let report = request.review().expect("local review should pass");
        assert_eq!(
            report.status,
            DexProtocolRiskReviewStatus::ReadyForLocalReview
        );
        assert!(report.blocker_codes.is_empty());
        assert!(report.asset_scope_passed);
        assert!(report.contract_hygiene_passed);
        assert!(report.token_hygiene_passed);
        assert!(report.governance_review_passed);
        assert!(report.spender_hygiene_passed);
        assert!(report.gas_slippage_passed);
        assert!(report.mev_controls_passed);
        assert!(report.terms_metadata_passed);
        assert!(!report.rpc_call_performed);
        assert!(!report.signer_material_loaded);
        assert!(!report.signing_performed);
        assert!(!report.broadcast_performed);
    }

    #[test]
    fn dex_protocol_risk_review_blocks_unsafe_local_metadata() {
        let mut request = DexProtocolRiskReviewRequest::local(
            "dex-protocol-review-blocked",
            "uniswap-v3-reviewed",
            venue(),
            "ethereum",
            "uniswap-v3-router-reviewed",
            "unknown-spender",
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            50,
            75.0,
            0.25,
            0.50,
            80.0,
            40.0,
        )
        .expect("local protocol review request should validate");
        request.chain_allowlisted = false;
        request.pair_allowlisted = false;
        request.router_allowlisted = false;
        request.spender_allowlisted = false;
        request.unlimited_allowance_requested = true;
        request.approval_revocation_planned = false;
        request.token_contract_reviewed = false;
        request.token_decimals_verified = false;
        request.public_mempool_required = true;
        request.mev_mitigation_reviewed = false;
        request.token_metadata_reviewed = false;
        request.protocol_terms_reviewed = false;
        request.jurisdiction_reviewed = false;
        request.incident_reputation_reviewed = false;

        let report = request
            .review()
            .expect("blocked review should still report");
        assert_eq!(report.status, DexProtocolRiskReviewStatus::Blocked);
        for expected in [
            "chain-not-allowlisted",
            "pair-not-allowlisted",
            "router-not-allowlisted",
            "spender-not-allowlisted",
            "unlimited-allowance-requested",
            "approval-revocation-not-planned",
            "token-contract-not-reviewed",
            "token-decimals-not-verified",
            "slippage-limit-exceeded",
            "gas-fee-limit-exceeded",
            "mev-risk-limit-exceeded",
            "public-mempool-mev-mitigation-missing",
            "token-metadata-not-reviewed",
            "protocol-terms-not-reviewed",
            "jurisdiction-not-reviewed",
            "incident-reputation-not-reviewed",
        ] {
            assert!(report.blocker_codes.iter().any(|actual| actual == expected));
        }
        assert!(!report.asset_scope_passed);
        assert!(!report.contract_hygiene_passed);
        assert!(!report.token_hygiene_passed);
        assert!(!report.governance_review_passed);
        assert!(!report.spender_hygiene_passed);
        assert!(!report.gas_slippage_passed);
        assert!(!report.mev_controls_passed);
        assert!(!report.terms_metadata_passed);
    }

    #[test]
    fn dex_protocol_risk_review_fails_closed_on_side_effect_flags() {
        let mut request = DexProtocolRiskReviewRequest::local(
            "dex-protocol-review-side-effect",
            "uniswap-v3-reviewed",
            venue(),
            "ethereum",
            "uniswap-v3-router-reviewed",
            "uniswap-v3-spender-reviewed",
            MarketPair::new("ETH", "USDC").expect("pair should validate"),
            75,
            25.0,
            2.0,
            0.25,
            10.0,
            50.0,
        )
        .expect("local protocol review request should validate");
        request.signer_material_loaded = true;

        let error = request
            .review()
            .expect_err("side-effect review request must fail closed");
        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_PROTOCOL_REVIEW_SIDE_EFFECT_FLAG"));
    }

    #[test]
    fn dex_swap_validation_audit_and_state_reopen_locally() {
        let audit_path = temp_path("dex-validation-audit", "jsonl");
        let state_path = temp_path("dex-validation-state", "sqlite");
        let approval = policy_gate()
            .validate_swap_quote(&router_profile(), &swap_request())
            .expect("paper DEX swap quote should be policy approved");
        let record = DexSwapValidationRecord::from_approved_request(&swap_request(), &approval)
            .expect("validation record should build");

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        let audit_record = append_dex_swap_validation_audit(&mut journal, &record, 1_700_000_101)
            .expect("audit append should pass");
        let checkpoint = persist_dex_swap_validation_checkpoint(&mut store, &record, 1_700_000_102)
            .expect("checkpoint should persist");
        assert_eq!(audit_record.event.subsystem, DEX_STATE_SUBSYSTEM);
        assert_eq!(checkpoint.key, DEX_LAST_SWAP_VALIDATION_CHECKPOINT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let stored = reopened
            .get_checkpoint(DEX_LAST_SWAP_VALIDATION_CHECKPOINT_KEY)
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        let recovered: DexSwapValidationRecord =
            serde_json::from_str(&stored.value).expect("checkpoint decodes");
        assert_eq!(recovered.request_id, "dex-swap-1");
        assert!(!recovered.rpc_call_performed);
        assert!(!recovered.signing_performed);
        assert!(!recovered.broadcast_performed);
        assert!(!recovered.live_execution_performed);

        let _ = fs::remove_file(audit_path);
        cleanup_sqlite(&state_path);
    }

    #[test]
    fn dex_swap_lifecycle_reconciles_quote_and_simulation_locally() {
        let audit_path = temp_path("dex-lifecycle-audit", "jsonl");
        let state_path = temp_path("dex-lifecycle-state", "sqlite");
        let validation = approved_validation_record();
        let quote = quote_response();
        let simulation = simulation_response();
        let record = DexSwapLifecycleRecord::from_local_quote_and_simulation(
            &validation,
            &quote,
            &simulation,
            false,
        )
        .expect("local DEX lifecycle should reconcile");

        assert_eq!(
            record.simulation_status,
            DexSimulationStatus::LocallyValidated
        );
        assert_eq!(record.quote_response_id, "dex-quote-response-1");
        assert_eq!(record.simulation_response_id, "dex-sim-response-1");
        assert!((record.amount_in - 0.1).abs() < f64::EPSILON);
        assert!((record.quoted_amount_out - 25.0).abs() < f64::EPSILON);
        assert!((record.simulated_amount_out - 24.5).abs() < f64::EPSILON);
        assert!((record.output_shortfall_bps - 200.0).abs() < f64::EPSILON);
        assert!(!record.rpc_call_performed);
        assert!(!record.signing_performed);
        assert!(!record.broadcast_performed);
        assert!(!record.live_execution_performed);
        assert!(!record.production_ready);

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        append_dex_swap_lifecycle_audit(&mut journal, &record, 1_700_000_201)
            .expect("lifecycle audit append should pass");
        persist_dex_swap_lifecycle_checkpoint(&mut store, &record, 1_700_000_202)
            .expect("lifecycle checkpoint should persist");
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let stored = reopened
            .get_checkpoint(DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY)
            .expect("checkpoint reads")
            .expect("checkpoint exists");
        let recovered: DexSwapLifecycleRecord =
            serde_json::from_str(&stored.value).expect("checkpoint decodes");
        assert_eq!(recovered.request_id, "dex-swap-1");
        assert!(!recovered.rpc_call_performed);
        assert!(!recovered.signing_performed);
        assert!(!recovered.broadcast_performed);

        let _ = fs::remove_file(audit_path);
        cleanup_sqlite(&state_path);
    }

    #[test]
    fn dex_swap_lifecycle_rejects_mismatched_quote_replay() {
        let validation = approved_validation_record();
        let mut quote = quote_response();
        quote.request_id = "different-swap".to_owned();

        let error = DexSwapLifecycleRecord::from_local_quote_and_simulation(
            &validation,
            &quote,
            &simulation_response(),
            false,
        )
        .expect_err("mismatched quote replay must fail closed");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_LIFECYCLE_QUOTE_REQUEST_MISMATCH"));
    }

    #[test]
    fn dex_intent_id_uniqueness_rejects_duplicates() {
        let first = approved_validation_record();
        let duplicate = first.clone();

        let error = validate_dex_intent_id_uniqueness(&[first, duplicate])
            .expect_err("duplicate DEX intent ids must fail closed");

        assert!(error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_INTENT_ID_DUPLICATE"));
    }

    #[test]
    fn dex_swap_lifecycle_persistence_rejects_side_effect_records() {
        let audit_path = temp_path("dex-lifecycle-invalid-audit", "jsonl");
        let state_path = temp_path("dex-lifecycle-invalid-state", "sqlite");
        let validation = approved_validation_record();
        let mut record = DexSwapLifecycleRecord::from_local_quote_and_simulation(
            &validation,
            &quote_response(),
            &simulation_response(),
            false,
        )
        .expect("baseline DEX lifecycle should validate");
        record.broadcast_performed = true;

        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");
        let audit_error = append_dex_swap_lifecycle_audit(&mut journal, &record, 1_700_000_211)
            .expect_err("side-effect lifecycle audit must fail closed");
        let checkpoint_error =
            persist_dex_swap_lifecycle_checkpoint(&mut store, &record, 1_700_000_212)
                .expect_err("side-effect lifecycle checkpoint must fail closed");

        assert!(audit_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_LIFECYCLE_EXTERNAL_SIDE_EFFECT"));
        assert!(checkpoint_error
            .violations()
            .iter()
            .any(|violation| violation.code() == "DEX_LIFECYCLE_EXTERNAL_SIDE_EFFECT"));
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 1);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        assert!(reopened
            .get_checkpoint(DEX_LAST_SWAP_LIFECYCLE_CHECKPOINT_KEY)
            .expect("checkpoint lookup should succeed")
            .is_none());

        let _ = fs::remove_file(audit_path);
        cleanup_sqlite(&state_path);
    }

    fn approved_validation_record() -> DexSwapValidationRecord {
        let approval = policy_gate()
            .validate_swap_quote(&router_profile(), &swap_request())
            .expect("paper DEX swap quote should be policy approved");
        DexSwapValidationRecord::from_approved_request(&swap_request(), &approval)
            .expect("validation record should build")
    }
}
