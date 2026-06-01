#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

use arb_core::{
    load_config_file, AgentConfig, BuildIdentity, ConfigError, DeterministicExecutionPlanner,
    ExecutionAdapterConfig, ExecutionPlanner, ExecutionPlannerConfig, ExecutionPlannerRequest,
    ExecutionScope, FeeAdjustedEdge, FeeEstimate, LiquidityRole, MarketPair, OpportunityCandidate,
    OpportunityLeg, OpportunityLegSide, OpportunityRouteKind, OpportunityScore, PolicyEngine,
    RuntimeDeploymentSmokeValidationRequest, RuntimeGracefulShutdownRequest,
    RuntimeRestartRecoveryDisposition, VenueKind, VenueRef, AGENTIC_HANDOFF_VERSION,
    AUDIT_DURABILITY_VALIDATION_VERSION, CEX_CONNECTOR_FRAMEWORK_VERSION,
    COMMUNICATIONS_CLI_VERSION, DASHBOARD_BOUNDARY_VERSION, DEFAULT_MARKET_DATA_FRESHNESS_MS,
    DEX_CONNECTOR_FRAMEWORK_VERSION, EXECUTION_ADAPTER_FRAMEWORK_VERSION,
    EXECUTION_PLANNER_VERSION, EXTERNAL_HARDENING_VERSION, OBSERVABILITY_RUNBOOK_VERSION,
    OPPORTUNITY_ENGINE_VERSION, PACKAGING_DEPLOYMENT_VERSION, PAPER_AUDIT_INTEGRATION_VERSION,
    PAPER_BALANCE_LEDGER_VERSION, PAPER_CONNECTOR_VERSION, PAPER_REALISM_VALIDATION_VERSION,
    PAPER_REALISTIC_FILL_MODEL_VERSION, RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION,
    RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION, RUNTIME_GRACEFUL_SHUTDOWN_VERSION,
    RUNTIME_LIFECYCLE_VERSION, RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION,
    SQLITE_WAL_DURABILITY_VERSION, TESTING_BACKTESTING_VERSION,
};
use std::{
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), AgentCliError> {
    run_with_args(env::args().skip(1))
}

fn run_with_args(args: impl IntoIterator<Item = String>) -> Result<(), AgentCliError> {
    let identity = BuildIdentity::current();
    println!("{} {}", identity.name(), identity.version());

    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("--config") => {
            let Some(path) = args.next() else {
                return Err(AgentCliError::Usage("--config requires a path".to_owned()));
            };
            let path = PathBuf::from(path);
            let config = load_config_file(&path)?;
            let policy = PolicyEngine::from_config(config.clone());
            println!("config: loaded and validated from {}", path.display());
            println!("mode: {:?}", config.runtime.mode);
            println!(
                "live-intent: {}",
                config.runtime.mode.permits_live_execution()
            );
            println!("policy: {} initialized", policy.trust_contract_version());
            println!("audit: append-only boundary available; {AUDIT_DURABILITY_VALIDATION_VERSION} validates local replay rejection, sync, concurrency, filesystem failure, simulated disk-full fail-closed probes, side-effect-free retention planning, and stale-lock restart recheck planning; runtime journal writing is not auto-started yet");
            println!("state: trait boundary and local SQLite WAL checkpoints available; {SQLITE_WAL_DURABILITY_VERSION} validates local integrity, WAL checkpoint, reopen, backup/restore, and multi-handle durability; external production-host validation pending");
            println!("market-data: normalized quote/order-book/fee boundaries available; live providers pending");
            println!("paper-connectors: {PAPER_CONNECTOR_VERSION} available for deterministic in-memory simulation only");
            println!("paper-balance-ledger: {PAPER_BALANCE_LEDGER_VERSION} available for local simulated balances, reservations, fills, and SQLite checkpoints only");
            println!("paper-fill-model: {PAPER_REALISTIC_FILL_MODEL_VERSION} available for supplied-depth local paper fills only; exchange-specific calibration pending");
            println!("paper-replay-calibration-backtest: {PAPER_REALISM_VALIDATION_VERSION} available for local matching profiles, adverse selection, calibration records, replay validation, and fixture backtests only; production-host validation still pending");
            println!("paper-audit-integration: {PAPER_AUDIT_INTEGRATION_VERSION} available for local paper report and ledger mutation audit records only; production audit durability validation still pending");
            println!("cex-framework: {CEX_CONNECTOR_FRAMEWORK_VERSION} available as typed interface only; live exchange adapters pending");
            println!("dex-web3-framework: {DEX_CONNECTOR_FRAMEWORK_VERSION} available as typed interface only; live RPC, signing, and broadcasts pending");
            println!("opportunity-engine: {OPPORTUNITY_ENGINE_VERSION} available for deterministic discovery/ranking with local depth, paper inventory, and transfer-risk modeling only; live execution pending");
            println!("execution-planner: {EXECUTION_PLANNER_VERSION} available for draft-only policy-evaluated planning; adapter submission disabled");
            println!("execution-adapter-framework: {EXECUTION_ADAPTER_FRAMEWORK_VERSION} available for deterministic boundary records only; external submission disabled");
            println!("runtime-lifecycle: {RUNTIME_LIFECYCLE_VERSION} available for local fail-closed audit/state/adapter wiring only; {RUNTIME_GRACEFUL_SHUTDOWN_VERSION} records local graceful-shutdown audit/state checkpoints without stopping services; {RUNTIME_BACKUP_RESTORE_VALIDATION_VERSION} validates local audit/state backup restore without deployment actions; {RUNTIME_RESTART_RECOVERY_VALIDATION_VERSION} validates local restart recovery summaries without service resume; {RUNTIME_DEPLOYMENT_SMOKE_VALIDATION_VERSION} validates local deployment-like smoke sequencing without service-manager actions; {}; live execution disabled", runtime_recovery_disposition_status());
            println!("communications-cli: {COMMUNICATIONS_CLI_VERSION} available for typed command/notification boundaries only; outbound integrations disabled");
            println!("embedded-dashboard: {DASHBOARD_BOUNDARY_VERSION} available for local render records only; web server exposure and live controls disabled");
            println!("observability-runbooks: {OBSERVABILITY_RUNBOOK_VERSION} available for local health/metric/log/runbook records only; metrics endpoints and outbound alerts disabled");
            println!("testing-backtesting: {TESTING_BACKTESTING_VERSION} available for deterministic validation plans only; external fuzzers, live networks, and live execution disabled");
            println!("packaging-deployment: {PACKAGING_DEPLOYMENT_VERSION} available for deterministic package/deployment plans only; builds, installs, public exposure, and production claims disabled");
            println!("external-hardening: {EXTERNAL_HARDENING_VERSION} available for deterministic evidence/checklist records only; external actions, production claims, and live-funds approval disabled");
            println!("agentic-handoff: {AGENTIC_HANDOFF_VERSION} available for deterministic prompts/checklists/package records only; external agent execution, production claims, and live-funds approval disabled");
            println!("market-data-default-freshness-ms: {DEFAULT_MARKET_DATA_FRESHNESS_MS}");
            println!("status: config/policy/audit/market-data/paper/paper-ledger/paper-fill-model/paper-replay-calibration-backtest/paper-audit-integration/cex-framework/dex-web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-cli/dashboard/observability/testing/packaging/external-hardening/agentic-handoff-ready; live trading still requires custody, exchange-specific live connectors, DEX RPC adapters, signing boundaries, live adapter submission, outbound communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, external validation harness execution, packaging/deployment hardening, executed external hardening evidence, and production execution hardening phases");
            Ok(())
        }
        Some("--help" | "-h") => {
            println!("usage: arb-agent [--config <path>]");
            println!(
                "       arb-agent validate-runtime-smoke --config <path> --workspace <fresh-dir>"
            );
            println!("default mode reports scaffold status without loading secrets or trading");
            println!("communication, dashboard, validation, packaging, hardening, and handoff commands are typed/local boundaries only; live execute, withdraw, bridge, sign, broadcast, external fuzzing, live network tests, public web exposure, service installation, external agent execution, external hardening execution, production claims, and production deployment remain unavailable");
            Ok(())
        }
        Some("validate-runtime-smoke") => {
            let options = parse_runtime_smoke_options(args)?;
            run_runtime_smoke_validation(&options)
        }
        Some(other) => Err(ConfigError::ReadFailed {
            path: other.to_owned(),
            reason: "unknown argument; use --help".to_owned(),
        }
        .into()),
        None => {
            println!("status: scaffold/config/policy/audit/market-data/paper/paper-ledger/paper-fill-model/paper-replay-calibration-backtest/paper-audit-integration/cex-framework/dex-web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-cli/dashboard/observability/testing/packaging/external-hardening/agentic-handoff-ready; live trading disabled until secrets, custody, exchange-specific live connectors, DEX RPC adapters, signing boundaries, live adapter submission, outbound communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, external validation harness execution, packaging/deployment hardening, executed external hardening evidence, and production execution hardening phases are implemented");
            println!(
                "runtime-recovery-dispositions: {}",
                runtime_recovery_disposition_status()
            );
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSmokeOptions {
    config_path: PathBuf,
    workspace_dir: PathBuf,
}

fn parse_runtime_smoke_options(
    args: impl Iterator<Item = String>,
) -> Result<RuntimeSmokeOptions, AgentCliError> {
    let mut config_path = None;
    let mut workspace_dir = None;
    let mut pending = args;
    while let Some(arg) = pending.next() {
        match arg.as_str() {
            "--config" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-runtime-smoke --config requires a path".to_owned(),
                    ));
                };
                config_path = Some(PathBuf::from(value));
            }
            "--workspace" => {
                let Some(value) = pending.next() else {
                    return Err(AgentCliError::Usage(
                        "validate-runtime-smoke --workspace requires a fresh directory".to_owned(),
                    ));
                };
                workspace_dir = Some(PathBuf::from(value));
            }
            other => {
                return Err(AgentCliError::Usage(format!(
                    "unknown validate-runtime-smoke argument: {other}"
                )));
            }
        }
    }

    Ok(RuntimeSmokeOptions {
        config_path: config_path.ok_or_else(|| {
            AgentCliError::Usage("validate-runtime-smoke requires --config <path>".to_owned())
        })?,
        workspace_dir: workspace_dir.ok_or_else(|| {
            AgentCliError::Usage(
                "validate-runtime-smoke requires --workspace <fresh-dir>".to_owned(),
            )
        })?,
    })
}

fn run_runtime_smoke_validation(options: &RuntimeSmokeOptions) -> Result<(), AgentCliError> {
    let config = load_config_file(&options.config_path)?;
    if config.runtime.mode.permits_live_execution() {
        return Err(AgentCliError::Usage(
            "validate-runtime-smoke only accepts observe or paper configs".to_owned(),
        ));
    }
    prepare_fresh_workspace(&options.workspace_dir)?;

    let policy = PolicyEngine::from_config(config.clone());
    let now_unix_ms = current_unix_ms()?;
    let lifecycle_request = build_runtime_smoke_lifecycle_request(&config, &policy, now_unix_ms)?;
    let shutdown_request = RuntimeGracefulShutdownRequest {
        id: "cli-runtime-smoke-shutdown".to_owned(),
        reason: "local-cli-runtime-smoke-validation".to_owned(),
        now_unix_ms: now_unix_ms.saturating_add(1),
    };
    let report = arb_core::validate_local_runtime_deployment_smoke(
        options.workspace_dir.join("runtime-audit.jsonl"),
        options.workspace_dir.join("runtime-state.sqlite3"),
        options.workspace_dir.join("runtime-audit.backup.jsonl"),
        options.workspace_dir.join("runtime-state.backup.sqlite3"),
        options.workspace_dir.join("audit-durability-workspace"),
        &policy,
        RuntimeDeploymentSmokeValidationRequest {
            lifecycle_request,
            shutdown_request,
            validated_at_unix_ms: now_unix_ms.saturating_add(2),
        },
    )
    .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    println!("runtime-smoke: validation passed");
    println!("runtime-smoke-version: {}", report.validation_version);
    println!("lifecycle-completed: {}", report.lifecycle_completed);
    println!(
        "graceful-shutdown-checkpointed: {}",
        report.graceful_shutdown_checkpointed
    );
    println!(
        "backup-restore-validated: {}",
        report.backup_restore_validated
    );
    println!(
        "restart-recovery-validated: {}",
        report.restart_recovery_validated
    );
    println!(
        "audit-durability-validated: {}",
        report.audit_durability_validated
    );
    println!(
        "restart-audit-records-replayed: {}",
        report.restart_audit_records_replayed
    );
    println!("service-manager-action-performed: false");
    println!("external-submission-performed: false");
    println!("live-execution-performed: false");
    println!("production-ready: false");
    Ok(())
}

fn prepare_fresh_workspace(path: &Path) -> Result<(), AgentCliError> {
    if path.as_os_str().is_empty() {
        return Err(AgentCliError::Usage(
            "runtime smoke workspace path is required".to_owned(),
        ));
    }
    if path.exists() {
        return Err(AgentCliError::Usage(format!(
            "runtime smoke workspace must not already exist: {}",
            path.display()
        )));
    }
    fs::create_dir_all(path).map_err(|error| {
        AgentCliError::Validation(format!(
            "failed to create runtime smoke workspace {}: {error}",
            path.display()
        ))
    })
}

fn build_runtime_smoke_lifecycle_request(
    config: &AgentConfig,
    policy: &PolicyEngine,
    now_unix_ms: u64,
) -> Result<arb_core::RuntimeLifecycleRequest, AgentCliError> {
    let plan_scope = if config.runtime.mode.permits_live_execution() {
        return Err(AgentCliError::Usage(
            "runtime smoke refuses live-armed configs".to_owned(),
        ));
    } else if matches!(config.runtime.mode, arb_core::RuntimeMode::Paper) {
        ExecutionScope::Paper
    } else {
        ExecutionScope::Observe
    };
    let candidate = build_runtime_smoke_candidate(config, now_unix_ms)?;
    let planner_request = ExecutionPlannerRequest {
        id: "cli-runtime-smoke-planner-request".to_owned(),
        strategy_id: "cli-runtime-smoke-strategy".to_owned(),
        candidate,
        config: ExecutionPlannerConfig {
            requested_scope: plan_scope,
            max_plan_legs: 2,
            max_total_notional_quote: config.risk.max_single_trade_quote.max(10.0) * 2.0,
            default_slippage_bps: config.risk.slippage_bps,
            max_market_data_age_ms: DEFAULT_MARKET_DATA_FRESHNESS_MS,
            require_policy_preflight: true,
        },
        default_chain: None,
        now_unix_ms,
    };
    let plan = DeterministicExecutionPlanner::new()
        .plan(&planner_request, policy)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    Ok(arb_core::RuntimeLifecycleRequest {
        id: "cli-runtime-smoke-lifecycle".to_owned(),
        adapter_request_id: "cli-runtime-smoke-adapter-request".to_owned(),
        plan,
        adapter_config: ExecutionAdapterConfig::default(),
        now_unix_ms,
    })
}

fn build_runtime_smoke_candidate(
    config: &AgentConfig,
    now_unix_ms: u64,
) -> Result<OpportunityCandidate, AgentCliError> {
    if config.venues.cex_allowlist.len() < 2 {
        return Err(AgentCliError::Usage(
            "validate-runtime-smoke requires at least two configured CEX venues".to_owned(),
        ));
    }
    if config.venues.asset_allowlist.len() < 2 {
        return Err(AgentCliError::Usage(
            "validate-runtime-smoke requires at least two configured assets".to_owned(),
        ));
    }

    let base = config.venues.asset_allowlist[0].clone();
    let quote = config
        .venues
        .asset_allowlist
        .iter()
        .find(|asset| !asset.eq_ignore_ascii_case(&base) && stable_quote_asset(asset))
        .cloned()
        .unwrap_or_else(|| config.venues.asset_allowlist[1].clone());
    let pair = MarketPair::new(base, quote)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;
    let buy_price = 4.0_f64.min(config.risk.max_single_trade_quote.max(1.0));
    let sell_price = buy_price + 1.0;
    let total_fees = 0.2;
    let edge = FeeAdjustedEdge::calculate(1.0, total_fees, sell_price)
        .map_err(|error| AgentCliError::Validation(error.to_string()))?;

    Ok(OpportunityCandidate {
        id: "cli-runtime-smoke-candidate".to_owned(),
        route_kind: OpportunityRouteKind::CexCex,
        pair: pair.clone(),
        legs: vec![
            runtime_smoke_leg(
                &config.venues.cex_allowlist[0],
                pair.clone(),
                OpportunityLegSide::Buy,
                buy_price,
            ),
            runtime_smoke_leg(
                &config.venues.cex_allowlist[1],
                pair,
                OpportunityLegSide::Sell,
                sell_price,
            ),
        ],
        edge,
        score: OpportunityScore {
            roi_bps: edge.roi_bps,
            freshness_penalty_bps: 0.0,
            risk_penalty_bps: 0.0,
            score_bps: edge.roi_bps,
        },
        liquidity_model: None,
        transfer_risk: None,
        discovered_at_unix_ms: now_unix_ms,
        source_quote_ids: vec![
            "cli-runtime-smoke-quote-a".to_owned(),
            "cli-runtime-smoke-quote-b".to_owned(),
        ],
        warnings: vec![
            "local CLI runtime smoke candidate; no market data, network, or execution occurred"
                .to_owned(),
        ],
    })
}

fn runtime_smoke_leg(
    venue_name: &str,
    pair: MarketPair,
    side: OpportunityLegSide,
    price_quote: f64,
) -> OpportunityLeg {
    let quantity_base = 1.0;
    let notional_quote = price_quote * quantity_base;
    let venue = VenueRef {
        name: venue_name.to_owned(),
        kind: VenueKind::Cex,
    };
    OpportunityLeg {
        venue: venue.clone(),
        pair: pair.clone(),
        side,
        price_quote,
        quantity_base,
        notional_quote,
        fee_estimate: FeeEstimate {
            venue,
            pair: Some(pair),
            notional_quote,
            liquidity_role: LiquidityRole::Taker,
            fee_bps: 10.0,
            venue_fee_quote: 0.1,
            network_fee_quote: 0.0,
            total_fee_quote: 0.1,
            externally_verified: false,
        },
        source_quote_id: format!("cli-runtime-smoke-quote-{venue_name}"),
        market_data_age_ms: 100,
    }
}

fn stable_quote_asset(asset: &str) -> bool {
    matches!(
        asset.to_ascii_uppercase().as_str(),
        "USD" | "USDC" | "USDT" | "DAI"
    )
}

fn current_unix_ms() -> Result<u64, AgentCliError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AgentCliError::Validation(format!("system clock error: {error}")))?;
    u64::try_from(duration.as_millis())
        .map_err(|_| AgentCliError::Validation("system clock value is too large".to_owned()))
}

fn runtime_recovery_disposition_status() -> String {
    format!(
        "restart recovery dispositions are {} and {} for local operator review only",
        recovery_disposition_label(RuntimeRestartRecoveryDisposition::ReadyForLocalReview),
        recovery_disposition_label(RuntimeRestartRecoveryDisposition::NeedsOperatorReview)
    )
}

#[derive(Debug)]
enum AgentCliError {
    Config(ConfigError),
    Usage(String),
    Validation(String),
}

impl fmt::Display for AgentCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "{error}"),
            Self::Usage(message) | Self::Validation(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for AgentCliError {}

impl From<ConfigError> for AgentCliError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

const fn recovery_disposition_label(
    disposition: RuntimeRestartRecoveryDisposition,
) -> &'static str {
    match disposition {
        RuntimeRestartRecoveryDisposition::ReadyForLocalReview => "ready-for-local-review",
        RuntimeRestartRecoveryDisposition::NeedsOperatorReview => "needs-operator-review",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_runtime_smoke_options, recovery_disposition_label,
        runtime_recovery_disposition_status, RuntimeRestartRecoveryDisposition,
    };

    #[test]
    fn recovery_disposition_labels_are_operator_facing() {
        assert_eq!(
            recovery_disposition_label(RuntimeRestartRecoveryDisposition::ReadyForLocalReview),
            "ready-for-local-review"
        );
        assert_eq!(
            recovery_disposition_label(RuntimeRestartRecoveryDisposition::NeedsOperatorReview),
            "needs-operator-review"
        );
    }

    #[test]
    fn runtime_recovery_disposition_status_is_local_only() {
        let status = runtime_recovery_disposition_status();

        assert!(status.contains("ready-for-local-review"));
        assert!(status.contains("needs-operator-review"));
        assert!(status.contains("local operator review only"));
    }

    #[test]
    fn runtime_smoke_options_require_config_and_workspace() {
        let options = parse_runtime_smoke_options(
            [
                "--config".to_owned(),
                "config.example.toml".to_owned(),
                "--workspace".to_owned(),
                "target/runtime-smoke".to_owned(),
            ]
            .into_iter(),
        )
        .expect("options should parse");

        assert_eq!(
            options.config_path,
            std::path::PathBuf::from("config.example.toml")
        );
        assert_eq!(
            options.workspace_dir,
            std::path::PathBuf::from("target/runtime-smoke")
        );
    }
}
