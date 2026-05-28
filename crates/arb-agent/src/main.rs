#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

use arb_core::{
    load_config_file, BuildIdentity, ConfigError, PolicyEngine, AGENTIC_HANDOFF_VERSION,
    CEX_CONNECTOR_FRAMEWORK_VERSION, COMMUNICATIONS_CLI_VERSION, DASHBOARD_BOUNDARY_VERSION,
    DEFAULT_MARKET_DATA_FRESHNESS_MS, DEX_CONNECTOR_FRAMEWORK_VERSION,
    EXECUTION_ADAPTER_FRAMEWORK_VERSION, EXECUTION_PLANNER_VERSION, EXTERNAL_HARDENING_VERSION,
    OBSERVABILITY_RUNBOOK_VERSION, OPPORTUNITY_ENGINE_VERSION, PACKAGING_DEPLOYMENT_VERSION,
    PAPER_BALANCE_LEDGER_VERSION, PAPER_CONNECTOR_VERSION, PAPER_REALISM_VALIDATION_VERSION,
    PAPER_REALISTIC_FILL_MODEL_VERSION, RUNTIME_LIFECYCLE_VERSION, SQLITE_WAL_DURABILITY_VERSION,
    TESTING_BACKTESTING_VERSION,
};
use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ConfigError> {
    let identity = BuildIdentity::current();
    println!("{} {}", identity.name(), identity.version());

    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("--config") => {
            let Some(path) = args.next() else {
                return Err(ConfigError::ReadFailed {
                    path: "<missing>".to_owned(),
                    reason: "--config requires a path".to_owned(),
                });
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
            println!("audit: append-only boundary available; runtime journal writing is not auto-started yet");
            println!("state: trait boundary and local SQLite WAL checkpoints available; {SQLITE_WAL_DURABILITY_VERSION} validates local integrity, WAL checkpoint, reopen, backup/restore, and multi-handle durability; external production-host validation pending");
            println!("market-data: normalized quote/order-book/fee boundaries available; live providers pending");
            println!("paper-connectors: {PAPER_CONNECTOR_VERSION} available for deterministic in-memory simulation only");
            println!("paper-balance-ledger: {PAPER_BALANCE_LEDGER_VERSION} available for local simulated balances, reservations, fills, and SQLite checkpoints only");
            println!("paper-fill-model: {PAPER_REALISTIC_FILL_MODEL_VERSION} available for supplied-depth local paper fills only; exchange-specific calibration pending");
            println!("paper-replay-calibration-backtest: {PAPER_REALISM_VALIDATION_VERSION} available for local matching profiles, adverse selection, calibration records, replay validation, and fixture backtests only; production-host validation still pending");
            println!("cex-framework: {CEX_CONNECTOR_FRAMEWORK_VERSION} available as typed interface only; live exchange adapters pending");
            println!("dex-web3-framework: {DEX_CONNECTOR_FRAMEWORK_VERSION} available as typed interface only; live RPC, signing, and broadcasts pending");
            println!("opportunity-engine: {OPPORTUNITY_ENGINE_VERSION} available for deterministic discovery/ranking only; live execution pending");
            println!("execution-planner: {EXECUTION_PLANNER_VERSION} available for draft-only policy-evaluated planning; adapter submission disabled");
            println!("execution-adapter-framework: {EXECUTION_ADAPTER_FRAMEWORK_VERSION} available for deterministic boundary records only; external submission disabled");
            println!("runtime-lifecycle: {RUNTIME_LIFECYCLE_VERSION} available for local fail-closed audit/state/adapter wiring only; live execution disabled");
            println!("communications-cli: {COMMUNICATIONS_CLI_VERSION} available for typed command/notification boundaries only; outbound integrations disabled");
            println!("embedded-dashboard: {DASHBOARD_BOUNDARY_VERSION} available for local render records only; web server exposure and live controls disabled");
            println!("observability-runbooks: {OBSERVABILITY_RUNBOOK_VERSION} available for local health/metric/log/runbook records only; metrics endpoints and outbound alerts disabled");
            println!("testing-backtesting: {TESTING_BACKTESTING_VERSION} available for deterministic validation plans only; external fuzzers, live networks, and live execution disabled");
            println!("packaging-deployment: {PACKAGING_DEPLOYMENT_VERSION} available for deterministic package/deployment plans only; builds, installs, public exposure, and production claims disabled");
            println!("external-hardening: {EXTERNAL_HARDENING_VERSION} available for deterministic evidence/checklist records only; external actions, production claims, and live-funds approval disabled");
            println!("agentic-handoff: {AGENTIC_HANDOFF_VERSION} available for deterministic prompts/checklists/package records only; external agent execution, production claims, and live-funds approval disabled");
            println!("market-data-default-freshness-ms: {DEFAULT_MARKET_DATA_FRESHNESS_MS}");
            println!("status: config/policy/audit/market-data/paper/paper-ledger/paper-fill-model/paper-replay-calibration-backtest/cex-framework/dex-web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-cli/dashboard/observability/testing/packaging/external-hardening/agentic-handoff-ready; live trading still requires custody, exchange-specific live connectors, DEX RPC adapters, signing boundaries, live adapter submission, outbound communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, external validation harness execution, packaging/deployment hardening, executed external hardening evidence, and production execution hardening phases");
            Ok(())
        }
        Some("--help" | "-h") => {
            println!("usage: arb-agent [--config <path>]");
            println!("default mode reports scaffold status without loading secrets or trading");
            println!("communication, dashboard, validation, packaging, hardening, and handoff commands are typed/local boundaries only; live execute, withdraw, bridge, sign, broadcast, external fuzzing, live network tests, public web exposure, service installation, external agent execution, external hardening execution, production claims, and production deployment remain unavailable");
            Ok(())
        }
        Some(other) => Err(ConfigError::ReadFailed {
            path: other.to_owned(),
            reason: "unknown argument; use --help".to_owned(),
        }),
        None => {
            println!("status: scaffold/config/policy/audit/market-data/paper/paper-ledger/paper-fill-model/paper-replay-calibration-backtest/cex-framework/dex-web3-framework/opportunity-engine/execution-planner/execution-adapter-framework/runtime-lifecycle/communications-cli/dashboard/observability/testing/packaging/external-hardening/agentic-handoff-ready; live trading disabled until secrets, custody, exchange-specific live connectors, DEX RPC adapters, signing boundaries, live adapter submission, outbound communications adapters, dashboard hosting hardening, observability runtime hardening, production-host runtime validation, external validation harness execution, packaging/deployment hardening, executed external hardening evidence, and production execution hardening phases are implemented");
            Ok(())
        }
    }
}
