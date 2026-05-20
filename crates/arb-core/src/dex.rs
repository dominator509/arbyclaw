#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]

use crate::{
    DestinationPolicy, ExecutionIntent, ExecutionIntentKind, ExecutionScope, FeeProvider,
    MarketPair, PolicyApproval, PolicyDecision, PolicyEngine, PolicyViolation, VenueKind, VenueRef,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error, fmt};

/// Stable DEX/Web3 framework version for audit and future replay surfaces.
pub const DEX_CONNECTOR_FRAMEWORK_VERSION: &str = "phase-8-dex-web3-framework-v1";

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
            | Self::PolicyDenied { .. } => &[],
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
        }
    }
}

impl Error for DexConnectorError {}

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
        DexConnectorError, DexConnectorRegistry, DexPolicyGate, DexRouteKind,
        DexRouterCapabilities, DexRouterProfile, DexSimulationStatus, DexSwapMode,
        DexSwapQuoteRequest, DexTokenProfile, Web3ChainProfile, Web3TransactionSimulationRequest,
    };
    use crate::{AgentConfig, ExecutionScope, MarketPair, PolicyEngine, VenueKind, VenueRef};

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

    fn policy_gate() -> DexPolicyGate {
        let config = AgentConfig::from_toml_str(PAPER_CONFIG).expect("config should validate");
        DexPolicyGate::new(PolicyEngine::from_config(config))
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
}
