#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    AppendOnlyAuditJournal, AuditEvent, AuditEventKind, AuditRecord, AuditValue, StateCheckpoint,
    StateStore, StateStoreError,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

/// Stable testing, fuzzing, and backtesting boundary version for audit and handoff surfaces.
pub const TESTING_BACKTESTING_VERSION: &str = "phase-15-testing-fuzzing-backtesting-v1";

/// State-store subsystem name for local validation run checkpoints.
pub const TESTING_STATE_SUBSYSTEM: &str = "testing";

/// State-store key for the latest local validation run record.
pub const TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY: &str = "testing:last-validation-run";

/// State-store key for the latest local property-check report.
pub const TESTING_LAST_PROPERTY_CHECK_REPORT_KEY: &str = "testing:last-property-check-report";

/// State-store key for the latest local validation corpus report.
pub const TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY: &str = "testing:last-validation-corpus-report";

/// State-store key for the latest local fuzz-corpus replay report.
pub const TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY: &str =
    "testing:last-fuzz-corpus-replay-report";

/// Conservative validation harness settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationHarnessConfig {
    /// Whether deterministic local validation planning is enabled.
    pub local_validation_enabled: bool,
    /// Maximum test cases accepted in a plan.
    pub max_test_cases: usize,
    /// Maximum fixture records accepted in a plan.
    pub max_fixtures: usize,
    /// Maximum fuzz corpora accepted in a plan.
    pub max_fuzz_corpora: usize,
    /// Maximum backtest scenarios accepted in a plan.
    pub max_backtest_scenarios: usize,
    /// Whether an external fuzzer process may be launched. Phase 15 requires false.
    pub external_fuzzer_invocation_enabled: bool,
    /// Whether live network tests may run. Phase 15 requires false.
    pub live_network_tests_enabled: bool,
    /// Whether live execution tests may submit orders/swaps. Phase 15 requires false.
    pub live_execution_tests_enabled: bool,
    /// Whether fixture metadata may include secret-like text. Phase 15 requires false.
    pub allow_secret_fixtures: bool,
}

impl Default for ValidationHarnessConfig {
    fn default() -> Self {
        Self {
            local_validation_enabled: true,
            max_test_cases: 256,
            max_fixtures: 256,
            max_fuzz_corpora: 64,
            max_backtest_scenarios: 64,
            external_fuzzer_invocation_enabled: false,
            live_network_tests_enabled: false,
            live_execution_tests_enabled: false,
            allow_secret_fixtures: false,
        }
    }
}

impl ValidationHarnessConfig {
    /// Validate fail-closed Phase 15 harness settings.
    pub fn validate(&self) -> Result<(), ValidationHarnessError> {
        let mut violations = Vec::new();

        if self.max_test_cases == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_MAX_TEST_CASES_ZERO",
                "max_test_cases must be positive",
            ));
        }

        if self.max_fixtures == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_MAX_FIXTURES_ZERO",
                "max_fixtures must be positive",
            ));
        }

        if self.max_fuzz_corpora == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_MAX_FUZZ_CORPORA_ZERO",
                "max_fuzz_corpora must be positive",
            ));
        }

        if self.max_backtest_scenarios == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_MAX_BACKTEST_SCENARIOS_ZERO",
                "max_backtest_scenarios must be positive",
            ));
        }

        if self.external_fuzzer_invocation_enabled {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_EXTERNAL_FUZZER_DENIED_IN_PHASE_15",
                "Phase 15 validation boundaries must not launch external fuzzers",
            ));
        }

        if self.live_network_tests_enabled {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_LIVE_NETWORK_TESTS_DENIED_IN_PHASE_15",
                "Phase 15 validation boundaries must not use live networks",
            ));
        }

        if self.live_execution_tests_enabled {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_LIVE_EXECUTION_TESTS_DENIED_IN_PHASE_15",
                "Phase 15 validation boundaries must not submit live execution",
            ));
        }

        if self.allow_secret_fixtures {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_SECRET_FIXTURES_DENIED",
                "validation fixtures must not contain secret-like text",
            ));
        }

        finish_validation(violations)
    }
}

/// Validation suite category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationSuiteKind {
    /// Unit tests for isolated functions and models.
    Unit,
    /// Integration tests across internal subsystems.
    Integration,
    /// Policy and trust-contract denial/approval tests.
    Policy,
    /// Audit and state replay tests.
    Replay,
    /// Property and fuzzing corpus tests.
    Fuzz,
    /// Deterministic fixture-based backtests.
    Backtest,
    /// Security, redaction, and abuse-case tests.
    Security,
    /// Regression tests for fixed defects.
    Regression,
}

/// Expected validation case outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExpectedValidationOutcome {
    /// The validation case should pass.
    Pass,
    /// The validation case should fail closed.
    FailClosed,
    /// The validation case should produce a deterministic warning.
    Warn,
}

/// Validation execution mode. Phase 15 supports model-only local modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationExecutionMode {
    /// Static plan inspection only.
    PlanOnly,
    /// Replay local fixtures only.
    FixtureReplayOnly,
    /// Simulate paper/backtest behavior without live submission.
    PaperSimulationOnly,
    /// Future external fuzzer runner mode. Phase 15 rejects this mode.
    ExternalFuzzer,
    /// Future live network validation mode. Phase 15 rejects this mode.
    LiveNetwork,
    /// Future live execution validation mode. Phase 15 rejects this mode.
    LiveExecution,
}

impl ValidationExecutionMode {
    /// Return whether this mode is allowed in Phase 15.
    #[must_use]
    pub const fn is_phase15_allowed(self) -> bool {
        matches!(
            self,
            Self::PlanOnly | Self::FixtureReplayOnly | Self::PaperSimulationOnly
        )
    }
}

/// Fixture category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureKind {
    /// Normalized quote fixture.
    MarketQuote,
    /// Order book fixture.
    OrderBook,
    /// Fee schedule fixture.
    FeeSchedule,
    /// Policy input/output fixture.
    PolicyDecision,
    /// Opportunity candidate fixture.
    OpportunityCandidate,
    /// Planner draft fixture.
    PlannerDraft,
    /// Execution adapter boundary fixture.
    ExecutionAdapterRecord,
    /// Observability or dashboard fixture.
    OperatorSurface,
}

/// Fuzz target category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FuzzTargetKind {
    /// Configuration parser fuzzing boundary.
    ConfigParser,
    /// Policy intent and destination fuzzing boundary.
    PolicyIntent,
    /// Market data normalization fuzzing boundary.
    MarketDataNormalizer,
    /// Opportunity ranking fuzzing boundary.
    OpportunityRanking,
    /// Execution planner fuzzing boundary.
    ExecutionPlanner,
    /// Command parser fuzzing boundary.
    CommandParser,
    /// Redaction and display safety fuzzing boundary.
    Redaction,
}

/// Deterministic test case metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationTestCase {
    /// Stable test case identifier.
    pub case_id: String,
    /// Human-readable test name.
    pub name: String,
    /// Suite category.
    pub suite: ValidationSuiteKind,
    /// Subsystem under validation.
    pub subsystem: String,
    /// Expected outcome.
    pub expected_outcome: ExpectedValidationOutcome,
    /// Referenced fixture identifiers.
    pub fixture_ids: Vec<String>,
    /// Whether this case requires live network access. Phase 15 requires false.
    pub requires_live_network: bool,
    /// Whether this case would submit live execution. Phase 15 requires false.
    pub submits_live_execution: bool,
}

impl ValidationTestCase {
    /// Create one validation test case.
    #[must_use]
    pub fn new(
        case_id: impl Into<String>,
        name: impl Into<String>,
        suite: ValidationSuiteKind,
        subsystem: impl Into<String>,
        expected_outcome: ExpectedValidationOutcome,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            name: name.into(),
            suite,
            subsystem: subsystem.into(),
            expected_outcome,
            fixture_ids: Vec::new(),
            requires_live_network: false,
            submits_live_execution: false,
        }
    }
}

/// Fixture metadata record. Payloads remain outside this model and are referenced by digest/path only.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationFixtureRecord {
    /// Stable fixture identifier.
    pub fixture_id: String,
    /// Fixture kind.
    pub kind: FixtureKind,
    /// Relative fixture path or logical name.
    pub source: String,
    /// Payload digest, if available.
    pub payload_sha256: Option<String>,
    /// Whether the fixture is synthetic/local. Phase 15 expects true for created fixtures.
    pub synthetic: bool,
    /// Whether fixture material contains credentials. Phase 15 requires false.
    pub contains_credentials: bool,
    /// Fixture generation timestamp in Unix epoch milliseconds.
    pub generated_at_ms: u64,
}

impl ValidationFixtureRecord {
    /// Create one local synthetic fixture metadata record.
    #[must_use]
    pub fn synthetic(
        fixture_id: impl Into<String>,
        kind: FixtureKind,
        source: impl Into<String>,
        payload_sha256: Option<String>,
        generated_at_ms: u64,
    ) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            kind,
            source: source.into(),
            payload_sha256,
            synthetic: true,
            contains_credentials: false,
            generated_at_ms,
        }
    }
}

/// Fuzz seed metadata. It does not invoke any external fuzzing tool.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FuzzSeedRecord {
    /// Stable seed identifier.
    pub seed_id: String,
    /// Seed digest.
    pub seed_sha256: String,
    /// Short purpose description.
    pub purpose: String,
}

impl FuzzSeedRecord {
    /// Create one fuzz seed record.
    #[must_use]
    pub fn new(
        seed_id: impl Into<String>,
        seed_sha256: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            seed_id: seed_id.into(),
            seed_sha256: seed_sha256.into(),
            purpose: purpose.into(),
        }
    }
}

/// Fuzz corpus metadata boundary. It is not an external fuzzer runner.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FuzzCorpusDefinition {
    /// Stable corpus identifier.
    pub corpus_id: String,
    /// Fuzz target kind.
    pub target: FuzzTargetKind,
    /// Deterministic seeds.
    pub seeds: Vec<FuzzSeedRecord>,
    /// Whether launching an external fuzzer is required. Phase 15 requires false.
    pub external_fuzzer_required: bool,
}

impl FuzzCorpusDefinition {
    /// Create one local-only fuzz corpus definition.
    #[must_use]
    pub fn local_only(
        corpus_id: impl Into<String>,
        target: FuzzTargetKind,
        seeds: Vec<FuzzSeedRecord>,
    ) -> Self {
        Self {
            corpus_id: corpus_id.into(),
            target,
            seeds,
            external_fuzzer_required: false,
        }
    }
}

/// Backtest dataset metadata boundary. It does not download data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BacktestDatasetDefinition {
    /// Stable dataset identifier.
    pub dataset_id: String,
    /// Base asset symbol.
    pub base_asset: String,
    /// Quote asset symbol.
    pub quote_asset: String,
    /// Venue identifiers included in the dataset.
    pub venue_ids: Vec<String>,
    /// Inclusive start timestamp in Unix epoch milliseconds.
    pub start_ms: u64,
    /// Exclusive end timestamp in Unix epoch milliseconds.
    pub end_ms: u64,
    /// Number of normalized quote records.
    pub quote_count: usize,
    /// Whether dataset is synthetic or recorded local fixture data.
    pub local_fixture_only: bool,
    /// Whether loading the dataset requires live network access. Phase 15 requires false.
    pub requires_live_network: bool,
}

impl BacktestDatasetDefinition {
    /// Return normalized pair text.
    #[must_use]
    pub fn pair(&self) -> String {
        format!("{}/{}", self.base_asset, self.quote_asset)
    }
}

/// Deterministic backtest scenario metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BacktestScenarioDefinition {
    /// Stable scenario identifier.
    pub scenario_id: String,
    /// Dataset metadata.
    pub dataset: BacktestDatasetDefinition,
    /// Initial account balance in quote-asset micro-units.
    pub initial_balance_microunits: u128,
    /// Maximum allowed drawdown in basis points for scenario acceptance.
    pub max_drawdown_basis_points: u32,
    /// Whether this scenario submits live orders. Phase 15 requires false.
    pub submits_live_orders: bool,
    /// Whether this scenario signs or broadcasts transactions. Phase 15 requires false.
    pub signs_or_broadcasts_transactions: bool,
}

/// Deterministic validation plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationPlan {
    /// Stable plan identifier.
    pub plan_id: String,
    /// Plan generation timestamp in Unix epoch milliseconds.
    pub generated_at_ms: u64,
    /// Validation execution mode.
    pub execution_mode: ValidationExecutionMode,
    /// Planned test cases.
    pub test_cases: Vec<ValidationTestCase>,
    /// Fixture metadata.
    pub fixtures: Vec<ValidationFixtureRecord>,
    /// Fuzz corpus definitions.
    pub fuzz_corpora: Vec<FuzzCorpusDefinition>,
    /// Backtest scenario definitions.
    pub backtest_scenarios: Vec<BacktestScenarioDefinition>,
}

impl ValidationPlan {
    /// Validate plan shape and fail-closed Phase 15 boundaries.
    pub fn validate(&self, config: &ValidationHarnessConfig) -> Result<(), ValidationHarnessError> {
        let mut violations = Vec::new();

        validate_id("plan", &self.plan_id, &mut violations);

        if !self.execution_mode.is_phase15_allowed() {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_EXECUTION_MODE_DENIED",
                "validation execution mode is not allowed in Phase 15",
            ));
        }

        validate_count(
            "test cases",
            self.test_cases.len(),
            config.max_test_cases,
            "VALIDATION_TEST_CASE_LIMIT_EXCEEDED",
            &mut violations,
        );
        validate_count(
            "fixtures",
            self.fixtures.len(),
            config.max_fixtures,
            "VALIDATION_FIXTURE_LIMIT_EXCEEDED",
            &mut violations,
        );
        validate_count(
            "fuzz corpora",
            self.fuzz_corpora.len(),
            config.max_fuzz_corpora,
            "VALIDATION_FUZZ_CORPUS_LIMIT_EXCEEDED",
            &mut violations,
        );
        validate_count(
            "backtest scenarios",
            self.backtest_scenarios.len(),
            config.max_backtest_scenarios,
            "VALIDATION_BACKTEST_SCENARIO_LIMIT_EXCEEDED",
            &mut violations,
        );

        validate_test_cases(&self.test_cases, &mut violations);
        validate_fixtures(&self.fixtures, &mut violations);
        validate_fuzz_corpora(&self.fuzz_corpora, &mut violations);
        validate_backtest_scenarios(&self.backtest_scenarios, &mut violations);

        finish_validation(violations)
    }
}

/// Validation harness request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRunRequest {
    /// Harness config.
    pub config: ValidationHarnessConfig,
    /// Validation plan.
    pub plan: ValidationPlan,
    /// Request timestamp in Unix epoch milliseconds.
    pub requested_at_ms: u64,
    /// Operator-facing label. Must not contain secrets.
    pub operator_label: Option<String>,
}

/// Validation run status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationRunStatus {
    /// Plan was validated but no external tools were launched.
    PlannedOnly,
    /// Plan failed validation.
    Rejected,
}

/// Deterministic validation run record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRunRecord {
    /// Boundary version that produced this record.
    pub testing_backtesting_version: String,
    /// Plan identifier.
    pub plan_id: String,
    /// Record status.
    pub status: ValidationRunStatus,
    /// Request timestamp in Unix epoch milliseconds.
    pub requested_at_ms: u64,
    /// Sanitized operator label.
    pub operator_label: Option<String>,
    /// Planned test case count.
    pub planned_test_cases: usize,
    /// Planned fixture count.
    pub planned_fixtures: usize,
    /// Planned fuzz corpus count.
    pub planned_fuzz_corpora: usize,
    /// Planned backtest scenario count.
    pub planned_backtest_scenarios: usize,
    /// Whether external fuzzer process invocation occurred. Phase 15 always returns false.
    pub external_fuzzer_invoked: bool,
    /// Whether live network access occurred. Phase 15 always returns false.
    pub live_network_used: bool,
    /// Whether live orders/swaps were submitted. Phase 15 always returns false.
    pub live_execution_submitted: bool,
    /// Whether signing or broadcast occurred. Phase 15 always returns false.
    pub signing_or_broadcast_performed: bool,
    /// Whether secret-like text was redacted from local record fields.
    pub secret_redaction_applied: bool,
}

/// Deterministic local property-check report over a validation plan.
///
/// This is an in-process invariant runner over already-local metadata. It does
/// not invoke proptest, cargo-fuzz, AFL/libFuzzer, live networks, downloads,
/// signing, broadcasts, or execution adapters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalPropertyCheckReport {
    /// Boundary version that produced this report.
    pub testing_backtesting_version: String,
    /// Plan identifier that was checked.
    pub plan_id: String,
    /// Number of deterministic local property checks executed.
    pub checks_executed: usize,
    /// Number of deterministic local property checks that passed.
    pub checks_passed: usize,
    /// Number of deterministic local property checks that failed.
    pub checks_failed: usize,
    /// Fixture references from test cases that were not declared in fixtures.
    pub missing_fixture_references: Vec<String>,
    /// Local fuzz corpus ids that had no seeds.
    pub empty_fuzz_corpora: Vec<String>,
    /// Backtest dataset ids that were not local-only.
    pub nonlocal_backtest_datasets: Vec<String>,
    /// Whether forbidden side-effect flags were found in the plan.
    pub forbidden_side_effect_flags_detected: bool,
    /// Whether an external fuzzer process was invoked. Always false here.
    pub external_fuzzer_invoked: bool,
    /// Whether live network access occurred. Always false here.
    pub live_network_used: bool,
    /// Whether live execution was submitted. Always false here.
    pub live_execution_submitted: bool,
    /// Whether signing or broadcast occurred. Always false here.
    pub signing_or_broadcast_performed: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

/// Local-only validation corpus execution request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalValidationCorpusRequest {
    /// Stable corpus identifier.
    pub corpus_id: String,
    /// Harness config used for every local validation plan.
    pub config: ValidationHarnessConfig,
    /// Caller-supplied local validation plans.
    pub plans: Vec<ValidationPlan>,
    /// Minimum number of local plans required for this corpus.
    pub min_plan_count: usize,
    /// Minimum aggregate test cases required for this corpus.
    pub min_test_case_count: usize,
    /// Minimum aggregate local fixtures required for this corpus.
    pub min_fixture_count: usize,
    /// Minimum aggregate local fuzz corpora required for this corpus.
    pub min_fuzz_corpus_count: usize,
    /// Minimum aggregate local backtest scenarios required for this corpus.
    pub min_backtest_scenario_count: usize,
    /// Request timestamp in Unix epoch milliseconds.
    pub requested_at_ms: u64,
    /// Operator-facing label. Must not contain secrets.
    pub operator_label: Option<String>,
}

/// Local validation corpus execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalValidationCorpusStatus {
    /// Every local plan and local property check passed for review.
    ReadyForLocalReview,
}

/// Local fuzz corpus replay request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalFuzzCorpusReplayRequest {
    /// Stable replay identifier.
    pub replay_id: String,
    /// Harness config used to validate local-only fuzz metadata.
    pub config: ValidationHarnessConfig,
    /// Caller-supplied local fuzz corpus definitions.
    pub fuzz_corpora: Vec<FuzzCorpusDefinition>,
    /// Request timestamp in Unix epoch milliseconds.
    pub requested_at_ms: u64,
    /// Operator-facing label. Must not contain secrets.
    pub operator_label: Option<String>,
}

/// Local fuzz corpus replay status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalFuzzCorpusReplayStatus {
    /// Local seed metadata replay is ready for operator review.
    ReadyForLocalReview,
}

/// Local fuzz replay target summary.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalFuzzTargetReplaySummary {
    /// Fuzz target kind summarized.
    pub target: FuzzTargetKind,
    /// Number of corpora covering this target.
    pub corpus_count: usize,
    /// Number of seeds covering this target.
    pub seed_count: usize,
}

/// Local-only fuzz corpus replay report.
///
/// This replays seed metadata and digest/accounting invariants only. It does
/// not invoke fuzz engines, mutate corpora, download inputs, use live networks,
/// submit execution, sign, broadcast, or approve production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalFuzzCorpusReplayReport {
    /// Boundary version that produced this report.
    pub testing_backtesting_version: String,
    /// Stable replay identifier.
    pub replay_id: String,
    /// Replay status.
    pub status: LocalFuzzCorpusReplayStatus,
    /// Request timestamp in Unix epoch milliseconds.
    pub requested_at_ms: u64,
    /// Sanitized operator label.
    pub operator_label: Option<String>,
    /// Number of local fuzz corpora replayed.
    pub corpus_count: usize,
    /// Number of deterministic local seeds replayed.
    pub seed_count: usize,
    /// Number of unique deterministic local seed identifiers replayed.
    pub unique_seed_count: usize,
    /// Per-target local replay summaries.
    pub target_summaries: Vec<LocalFuzzTargetReplaySummary>,
    /// Whether secret-like text was redacted from local record fields.
    pub secret_redaction_applied: bool,
    /// Whether external fuzzer process invocation occurred. Always false here.
    pub external_fuzzer_invoked: bool,
    /// Whether live network access occurred. Always false here.
    pub live_network_used: bool,
    /// Whether live execution was submitted. Always false here.
    pub live_execution_submitted: bool,
    /// Whether signing or broadcast occurred. Always false here.
    pub signing_or_broadcast_performed: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

/// Local validation corpus execution report.
///
/// This aggregates deterministic in-process validation plans only. It does not
/// invoke external fuzzers, download corpora, use live networks, submit orders,
/// sign, broadcast, or approve production readiness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalValidationCorpusReport {
    /// Boundary version that produced this report.
    pub testing_backtesting_version: String,
    /// Stable corpus identifier.
    pub corpus_id: String,
    /// Corpus status.
    pub status: LocalValidationCorpusStatus,
    /// Request timestamp in Unix epoch milliseconds.
    pub requested_at_ms: u64,
    /// Sanitized operator label.
    pub operator_label: Option<String>,
    /// Number of local validation plans executed.
    pub plan_count: usize,
    /// Number of accepted local validation plans.
    pub accepted_plan_count: usize,
    /// Aggregate planned test case count.
    pub planned_test_cases: usize,
    /// Aggregate planned fixture count.
    pub planned_fixtures: usize,
    /// Aggregate planned fuzz corpus count.
    pub planned_fuzz_corpora: usize,
    /// Aggregate planned backtest scenario count.
    pub planned_backtest_scenarios: usize,
    /// Aggregate deterministic local property checks executed.
    pub property_checks_executed: usize,
    /// Aggregate deterministic local property checks passed.
    pub property_checks_passed: usize,
    /// Aggregate deterministic local property checks failed.
    pub property_checks_failed: usize,
    /// Minimum number of local plans required by the request.
    pub min_plan_count: usize,
    /// Minimum aggregate test cases required by the request.
    pub min_test_case_count: usize,
    /// Minimum aggregate local fixtures required by the request.
    pub min_fixture_count: usize,
    /// Minimum aggregate local fuzz corpora required by the request.
    pub min_fuzz_corpus_count: usize,
    /// Minimum aggregate local backtest scenarios required by the request.
    pub min_backtest_scenario_count: usize,
    /// Whether aggregate corpus breadth satisfied the request.
    pub corpus_breadth_requirements_met: bool,
    /// Whether secret-like text was redacted from local record fields.
    pub secret_redaction_applied: bool,
    /// Whether external fuzzer process invocation occurred. Always false here.
    pub external_fuzzer_invoked: bool,
    /// Whether live network access occurred. Always false here.
    pub live_network_used: bool,
    /// Whether live execution was submitted. Always false here.
    pub live_execution_submitted: bool,
    /// Whether signing or broadcast occurred. Always false here.
    pub signing_or_broadcast_performed: bool,
    /// Whether this report approves production readiness. Always false here.
    pub production_ready: bool,
}

impl LocalPropertyCheckReport {
    /// Validate report invariants before audit/state persistence.
    pub fn validate(&self) -> Result<(), ValidationHarnessError> {
        let mut violations = Vec::new();
        validate_id("property check plan", &self.plan_id, &mut violations);
        if self.testing_backtesting_version != TESTING_BACKTESTING_VERSION {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_PROPERTY_REPORT_VERSION_MISMATCH",
                format!(
                    "property check version {} does not match {}",
                    self.testing_backtesting_version, TESTING_BACKTESTING_VERSION
                ),
            ));
        }
        if self.checks_executed == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_PROPERTY_CHECKS_EMPTY",
                "local property check report must execute at least one check",
            ));
        }
        if self.checks_passed.saturating_add(self.checks_failed) != self.checks_executed {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_PROPERTY_CHECK_COUNTS_INVALID",
                "local property check pass/fail counts must match executed count",
            ));
        }
        for reference in &self.missing_fixture_references {
            validate_id("missing fixture reference", reference, &mut violations);
        }
        for corpus_id in &self.empty_fuzz_corpora {
            validate_id("empty fuzz corpus", corpus_id, &mut violations);
        }
        for dataset_id in &self.nonlocal_backtest_datasets {
            validate_id("nonlocal backtest dataset", dataset_id, &mut violations);
        }
        if self.external_fuzzer_invoked
            || self.live_network_used
            || self.live_execution_submitted
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_PROPERTY_REPORT_SIDE_EFFECT_DENIED",
                "local property check report must not record external fuzzers, live networks, live execution, signing, broadcasts, or production readiness",
            ));
        }
        finish_validation(violations)
    }
}

impl LocalFuzzCorpusReplayReport {
    /// Validate report invariants before audit/state persistence.
    pub fn validate(&self) -> Result<(), ValidationHarnessError> {
        let mut violations = Vec::new();
        validate_id("fuzz replay", &self.replay_id, &mut violations);

        if self.testing_backtesting_version != TESTING_BACKTESTING_VERSION {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_FUZZ_REPLAY_VERSION_MISMATCH",
                format!(
                    "fuzz replay version {} does not match {}",
                    self.testing_backtesting_version, TESTING_BACKTESTING_VERSION
                ),
            ));
        }

        if let Some(operator_label) = &self.operator_label {
            validate_name(
                "fuzz replay operator label",
                operator_label,
                &mut violations,
            );
        }

        if self.corpus_count == 0 || self.seed_count == 0 || self.unique_seed_count == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_FUZZ_REPLAY_EMPTY",
                "local fuzz corpus replay requires at least one local corpus and seed",
            ));
        }

        if self.unique_seed_count > self.seed_count {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_FUZZ_REPLAY_UNIQUE_SEED_COUNT_INVALID",
                "unique fuzz seed count cannot exceed replayed seed count",
            ));
        }

        let target_seed_total: usize = self
            .target_summaries
            .iter()
            .map(|summary| summary.seed_count)
            .sum();
        let target_corpus_total: usize = self
            .target_summaries
            .iter()
            .map(|summary| summary.corpus_count)
            .sum();

        if self.target_summaries.is_empty()
            || target_seed_total != self.seed_count
            || target_corpus_total != self.corpus_count
        {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_FUZZ_REPLAY_TARGET_COUNTS_INVALID",
                "local fuzz replay target summaries must account for all corpora and seeds",
            ));
        }

        for summary in &self.target_summaries {
            if summary.corpus_count == 0 || summary.seed_count == 0 {
                violations.push(ValidationHarnessViolation::new(
                    "VALIDATION_FUZZ_REPLAY_TARGET_EMPTY",
                    "local fuzz replay target summaries require non-empty corpus and seed counts",
                ));
            }
        }

        if self.external_fuzzer_invoked
            || self.live_network_used
            || self.live_execution_submitted
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_FUZZ_REPLAY_SIDE_EFFECT_DENIED",
                "local fuzz corpus replay must not record external fuzzers, live networks, live execution, signing, broadcasts, or production readiness",
            ));
        }

        finish_validation(violations)
    }
}

impl LocalValidationCorpusReport {
    /// Validate report invariants before audit/state persistence.
    pub fn validate(&self) -> Result<(), ValidationHarnessError> {
        let mut violations = Vec::new();
        validate_id("validation corpus", &self.corpus_id, &mut violations);

        if self.testing_backtesting_version != TESTING_BACKTESTING_VERSION {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_CORPUS_REPORT_VERSION_MISMATCH",
                format!(
                    "validation corpus version {} does not match {}",
                    self.testing_backtesting_version, TESTING_BACKTESTING_VERSION
                ),
            ));
        }

        if let Some(operator_label) = &self.operator_label {
            validate_name(
                "validation corpus operator label",
                operator_label,
                &mut violations,
            );
        }

        if self.plan_count == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_EMPTY",
                "local validation corpus report must include at least one plan",
            ));
        }

        if self.accepted_plan_count != self.plan_count {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_PLAN_COUNT_MISMATCH",
                "local validation corpus accepted plan count must match plan count",
            ));
        }

        if self.property_checks_executed == 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_PROPERTY_CHECKS_EMPTY",
                "local validation corpus must execute property checks",
            ));
        }

        if self
            .property_checks_passed
            .saturating_add(self.property_checks_failed)
            != self.property_checks_executed
        {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_PROPERTY_CHECK_COUNTS_INVALID",
                "local validation corpus property check counts must match executed count",
            ));
        }

        if self.property_checks_failed > 0 {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_PROPERTY_CHECKS_FAILED",
                "local validation corpus property checks must all pass",
            ));
        }

        if !self.corpus_breadth_requirements_met
            || self.plan_count < self.min_plan_count
            || self.planned_test_cases < self.min_test_case_count
            || self.planned_fixtures < self.min_fixture_count
            || self.planned_fuzz_corpora < self.min_fuzz_corpus_count
            || self.planned_backtest_scenarios < self.min_backtest_scenario_count
        {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_BREADTH_REQUIREMENTS_NOT_MET",
                "local validation corpus must satisfy requested plan, test, fixture, fuzz, and backtest breadth requirements",
            ));
        }

        if self.external_fuzzer_invoked
            || self.live_network_used
            || self.live_execution_submitted
            || self.signing_or_broadcast_performed
            || self.production_ready
        {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_CORPUS_SIDE_EFFECT_DENIED",
                "local validation corpus report must not record external fuzzers, live networks, live execution, signing, broadcasts, or production readiness",
            ));
        }

        finish_validation(violations)
    }
}

impl ValidationRunRecord {
    /// Validate a local validation run record before audit or state persistence.
    pub fn validate(&self) -> Result<(), ValidationHarnessError> {
        let mut violations = Vec::new();

        validate_id("validation plan", &self.plan_id, &mut violations);

        if self.testing_backtesting_version != TESTING_BACKTESTING_VERSION {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_RECORD_VERSION_MISMATCH",
                format!(
                    "validation run version {} does not match {}",
                    self.testing_backtesting_version, TESTING_BACKTESTING_VERSION
                ),
            ));
        }

        if let Some(operator_label) = &self.operator_label {
            validate_name("validation operator label", operator_label, &mut violations);
        }

        if self.external_fuzzer_invoked {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_RECORD_EXTERNAL_FUZZER_DENIED",
                "validation run records must not indicate external fuzzer invocation",
            ));
        }

        if self.live_network_used {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_RECORD_LIVE_NETWORK_DENIED",
                "validation run records must not indicate live network usage",
            ));
        }

        if self.live_execution_submitted {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_RECORD_LIVE_EXECUTION_DENIED",
                "validation run records must not indicate live execution submission",
            ));
        }

        if self.signing_or_broadcast_performed {
            violations.push(ValidationHarnessViolation::new(
                "VALIDATION_RECORD_SIGNING_BROADCAST_DENIED",
                "validation run records must not indicate signing or broadcast",
            ));
        }

        finish_validation(violations)
    }
}

/// Execute deterministic local property checks over a validation plan.
///
/// The checks are intentionally metadata-level and local-only. They prove basic
/// plan consistency without launching external property/fuzz engines.
pub fn run_local_validation_property_checks(
    plan: &ValidationPlan,
    config: &ValidationHarnessConfig,
) -> Result<LocalPropertyCheckReport, ValidationHarnessError> {
    config.validate()?;
    plan.validate(config)?;

    let fixture_ids = plan
        .fixtures
        .iter()
        .map(|fixture| fixture.fixture_id.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut missing_fixture_references = BTreeSet::new();
    for test_case in &plan.test_cases {
        for fixture_id in &test_case.fixture_ids {
            if !fixture_ids.contains(&fixture_id.to_ascii_lowercase()) {
                missing_fixture_references.insert(fixture_id.clone());
            }
        }
    }

    let empty_fuzz_corpora = plan
        .fuzz_corpora
        .iter()
        .filter(|corpus| corpus.seeds.is_empty())
        .map(|corpus| corpus.corpus_id.clone())
        .collect::<Vec<_>>();

    let nonlocal_backtest_datasets = plan
        .backtest_scenarios
        .iter()
        .filter(|scenario| {
            !scenario.dataset.local_fixture_only || scenario.dataset.requires_live_network
        })
        .map(|scenario| scenario.dataset.dataset_id.clone())
        .collect::<Vec<_>>();

    let forbidden_side_effect_flags_detected = plan
        .test_cases
        .iter()
        .any(|test_case| test_case.requires_live_network || test_case.submits_live_execution)
        || plan
            .fuzz_corpora
            .iter()
            .any(|corpus| corpus.external_fuzzer_required)
        || plan.backtest_scenarios.iter().any(|scenario| {
            scenario.submits_live_orders
                || scenario.signs_or_broadcasts_transactions
                || scenario.dataset.requires_live_network
        });

    let checks_executed = 4;
    let checks_failed = usize::from(!missing_fixture_references.is_empty())
        + usize::from(!empty_fuzz_corpora.is_empty())
        + usize::from(!nonlocal_backtest_datasets.is_empty())
        + usize::from(forbidden_side_effect_flags_detected);
    let report = LocalPropertyCheckReport {
        testing_backtesting_version: TESTING_BACKTESTING_VERSION.to_owned(),
        plan_id: plan.plan_id.clone(),
        checks_executed,
        checks_passed: checks_executed - checks_failed,
        checks_failed,
        missing_fixture_references: missing_fixture_references.into_iter().collect(),
        empty_fuzz_corpora,
        nonlocal_backtest_datasets,
        forbidden_side_effect_flags_detected,
        external_fuzzer_invoked: false,
        live_network_used: false,
        live_execution_submitted: false,
        signing_or_broadcast_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Replay local fuzz corpus seed metadata without invoking an external fuzzer.
///
/// This validates the corpus/seed definitions, deduplicates seed ids for
/// accounting, and summarizes target coverage. It deliberately does not execute
/// cargo-fuzz, libFuzzer, AFL, property frameworks, downloads, live networks,
/// signing, broadcasts, or execution adapters.
pub fn run_local_fuzz_corpus_replay(
    request: LocalFuzzCorpusReplayRequest,
) -> Result<LocalFuzzCorpusReplayReport, ValidationHarnessError> {
    let mut violations = Vec::new();
    validate_id("fuzz replay", &request.replay_id, &mut violations);
    if request.fuzz_corpora.is_empty() {
        violations.push(ValidationHarnessViolation::new(
            "VALIDATION_FUZZ_REPLAY_REQUEST_EMPTY",
            "local fuzz corpus replay request must include at least one corpus",
        ));
    }
    finish_validation(violations)?;

    request.config.validate()?;
    let mut corpus_violations = Vec::new();
    validate_count(
        "fuzz corpora",
        request.fuzz_corpora.len(),
        request.config.max_fuzz_corpora,
        "VALIDATION_FUZZ_CORPUS_LIMIT_EXCEEDED",
        &mut corpus_violations,
    );
    validate_fuzz_corpora(&request.fuzz_corpora, &mut corpus_violations);
    finish_validation(corpus_violations)?;

    let mut seen_seed_ids = BTreeSet::new();
    let mut target_counts = BTreeMap::<FuzzTargetKind, (usize, usize)>::new();
    let mut seed_count = 0usize;

    for corpus in &request.fuzz_corpora {
        let target_entry = target_counts.entry(corpus.target).or_insert((0, 0));
        target_entry.0 += 1;
        target_entry.1 += corpus.seeds.len();
        seed_count += corpus.seeds.len();
        for seed in &corpus.seeds {
            seen_seed_ids.insert(seed.seed_id.to_ascii_lowercase());
        }
    }

    let target_summaries = target_counts
        .into_iter()
        .map(
            |(target, (corpus_count, seed_count))| LocalFuzzTargetReplaySummary {
                target,
                corpus_count,
                seed_count,
            },
        )
        .collect::<Vec<_>>();

    let (operator_label, secret_redaction_applied) = match request.operator_label {
        Some(label) => {
            let (sanitized, redacted) = sanitize_validation_text(&label, 128);
            (Some(sanitized), redacted)
        }
        None => (None, false),
    };

    let report = LocalFuzzCorpusReplayReport {
        testing_backtesting_version: TESTING_BACKTESTING_VERSION.to_owned(),
        replay_id: request.replay_id,
        status: LocalFuzzCorpusReplayStatus::ReadyForLocalReview,
        requested_at_ms: request.requested_at_ms,
        operator_label,
        corpus_count: request.fuzz_corpora.len(),
        seed_count,
        unique_seed_count: seen_seed_ids.len(),
        target_summaries,
        secret_redaction_applied,
        external_fuzzer_invoked: false,
        live_network_used: false,
        live_execution_submitted: false,
        signing_or_broadcast_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Execute a local deterministic validation corpus over caller-supplied plans.
///
/// This composes the existing local validation harness and local property-check
/// runner. It is intentionally side-effect-free with respect to external
/// fuzzers, live networks, execution adapters, signing, and broadcasts.
pub fn run_local_validation_corpus(
    request: LocalValidationCorpusRequest,
) -> Result<LocalValidationCorpusReport, ValidationHarnessError> {
    let mut violations = Vec::new();
    validate_id("validation corpus", &request.corpus_id, &mut violations);
    if request.plans.is_empty() {
        violations.push(ValidationHarnessViolation::new(
            "VALIDATION_CORPUS_REQUEST_EMPTY",
            "local validation corpus request must include at least one plan",
        ));
    }
    finish_validation(violations)?;

    request.config.validate()?;
    let min_plan_count = request.min_plan_count;
    let min_test_case_count = request.min_test_case_count;
    let min_fixture_count = request.min_fixture_count;
    let min_fuzz_corpus_count = request.min_fuzz_corpus_count;
    let min_backtest_scenario_count = request.min_backtest_scenario_count;

    let harness = DeterministicValidationHarness;
    let plan_count = request.plans.len();
    let mut accepted_plan_count = 0usize;
    let mut planned_test_cases = 0usize;
    let mut planned_fixtures = 0usize;
    let mut planned_fuzz_corpora = 0usize;
    let mut planned_backtest_scenarios = 0usize;
    let mut property_checks_executed = 0usize;
    let mut property_checks_passed = 0usize;
    let mut property_checks_failed = 0usize;
    let mut secret_redaction_applied = false;

    for plan in request.plans {
        let property_report = run_local_validation_property_checks(&plan, &request.config)?;
        let run_record = harness.validate_plan(ValidationRunRequest {
            config: request.config.clone(),
            plan,
            requested_at_ms: request.requested_at_ms,
            operator_label: request.operator_label.clone(),
        })?;

        accepted_plan_count += 1;
        planned_test_cases += run_record.planned_test_cases;
        planned_fixtures += run_record.planned_fixtures;
        planned_fuzz_corpora += run_record.planned_fuzz_corpora;
        planned_backtest_scenarios += run_record.planned_backtest_scenarios;
        property_checks_executed += property_report.checks_executed;
        property_checks_passed += property_report.checks_passed;
        property_checks_failed += property_report.checks_failed;
        secret_redaction_applied |= run_record.secret_redaction_applied;
    }

    let corpus_breadth_requirements_met = plan_count >= min_plan_count
        && planned_test_cases >= min_test_case_count
        && planned_fixtures >= min_fixture_count
        && planned_fuzz_corpora >= min_fuzz_corpus_count
        && planned_backtest_scenarios >= min_backtest_scenario_count;

    let operator_label = request.operator_label.as_ref().map(|label| {
        let (sanitized, redacted) = sanitize_validation_text(label, 256);
        secret_redaction_applied |= redacted;
        sanitized
    });

    let report = LocalValidationCorpusReport {
        testing_backtesting_version: TESTING_BACKTESTING_VERSION.to_owned(),
        corpus_id: request.corpus_id,
        status: LocalValidationCorpusStatus::ReadyForLocalReview,
        requested_at_ms: request.requested_at_ms,
        operator_label,
        plan_count,
        accepted_plan_count,
        planned_test_cases,
        planned_fixtures,
        planned_fuzz_corpora,
        planned_backtest_scenarios,
        property_checks_executed,
        property_checks_passed,
        property_checks_failed,
        min_plan_count,
        min_test_case_count,
        min_fixture_count,
        min_fuzz_corpus_count,
        min_backtest_scenario_count,
        corpus_breadth_requirements_met,
        secret_redaction_applied,
        external_fuzzer_invoked: false,
        live_network_used: false,
        live_execution_submitted: false,
        signing_or_broadcast_performed: false,
        production_ready: false,
    };
    report.validate()?;
    Ok(report)
}

/// Persist the latest local validation corpus report through the typed state boundary.
pub fn persist_validation_corpus_report_checkpoint(
    store: &mut impl StateStore,
    report: &LocalValidationCorpusReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ValidationHarnessError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY.to_owned(),
        subsystem: TESTING_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ValidationHarnessError::StateStoreFailed {
                reason: format!("failed to serialize validation corpus report checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ValidationHarnessError::from)?;
    Ok(checkpoint)
}

/// Append one local validation corpus report to the append-only audit journal.
pub fn append_validation_corpus_report_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &LocalValidationCorpusReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ValidationHarnessError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("validation-corpus-{}", report.corpus_id),
        AuditEventKind::RuntimeLifecycle,
        TESTING_STATE_SUBSYSTEM,
        "validation-corpus-runner",
        "local validation corpus recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "testing_backtesting_version",
            AuditValue::Text(TESTING_BACKTESTING_VERSION.to_owned()),
        )
        .with_metadata("corpus_id", AuditValue::Text(report.corpus_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "plan_count",
            AuditValue::Text(report.plan_count.to_string()),
        )
        .with_metadata(
            "accepted_plan_count",
            AuditValue::Text(report.accepted_plan_count.to_string()),
        )
        .with_metadata(
            "property_checks_executed",
            AuditValue::Text(report.property_checks_executed.to_string()),
        )
        .with_metadata(
            "property_checks_failed",
            AuditValue::Text(report.property_checks_failed.to_string()),
        )
        .with_metadata(
            "min_plan_count",
            AuditValue::Text(report.min_plan_count.to_string()),
        )
        .with_metadata(
            "min_test_case_count",
            AuditValue::Text(report.min_test_case_count.to_string()),
        )
        .with_metadata(
            "min_fixture_count",
            AuditValue::Text(report.min_fixture_count.to_string()),
        )
        .with_metadata(
            "min_fuzz_corpus_count",
            AuditValue::Text(report.min_fuzz_corpus_count.to_string()),
        )
        .with_metadata(
            "min_backtest_scenario_count",
            AuditValue::Text(report.min_backtest_scenario_count.to_string()),
        )
        .with_metadata(
            "corpus_breadth_requirements_met",
            AuditValue::Bool(report.corpus_breadth_requirements_met),
        )
        .with_metadata(
            "external_fuzzer_invoked",
            AuditValue::Bool(report.external_fuzzer_invoked),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "live_execution_submitted",
            AuditValue::Bool(report.live_execution_submitted),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ValidationHarnessError::from)
}

/// Persist the latest local fuzz-corpus replay report through the typed state boundary.
pub fn persist_fuzz_corpus_replay_report_checkpoint(
    store: &mut impl StateStore,
    report: &LocalFuzzCorpusReplayReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ValidationHarnessError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY.to_owned(),
        subsystem: TESTING_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ValidationHarnessError::StateStoreFailed {
                reason: format!("failed to serialize fuzz corpus replay checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ValidationHarnessError::from)?;
    Ok(checkpoint)
}

/// Append one local fuzz-corpus replay report to the append-only audit journal.
pub fn append_fuzz_corpus_replay_report_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &LocalFuzzCorpusReplayReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ValidationHarnessError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("validation-fuzz-replay-{}", report.replay_id),
        AuditEventKind::RuntimeLifecycle,
        TESTING_STATE_SUBSYSTEM,
        "validation-fuzz-corpus-replay",
        "local fuzz corpus replay recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "testing_backtesting_version",
            AuditValue::Text(TESTING_BACKTESTING_VERSION.to_owned()),
        )
        .with_metadata("replay_id", AuditValue::Text(report.replay_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", report.status)))
        .with_metadata(
            "corpus_count",
            AuditValue::Text(report.corpus_count.to_string()),
        )
        .with_metadata(
            "seed_count",
            AuditValue::Text(report.seed_count.to_string()),
        )
        .with_metadata(
            "unique_seed_count",
            AuditValue::Text(report.unique_seed_count.to_string()),
        )
        .with_metadata(
            "target_summary_count",
            AuditValue::Text(report.target_summaries.len().to_string()),
        )
        .with_metadata(
            "external_fuzzer_invoked",
            AuditValue::Bool(report.external_fuzzer_invoked),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "live_execution_submitted",
            AuditValue::Bool(report.live_execution_submitted),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ValidationHarnessError::from)
}

/// Persist the latest local property-check report through the typed state boundary.
pub fn persist_property_check_report_checkpoint(
    store: &mut impl StateStore,
    report: &LocalPropertyCheckReport,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ValidationHarnessError> {
    report.validate()?;
    let checkpoint = StateCheckpoint {
        key: TESTING_LAST_PROPERTY_CHECK_REPORT_KEY.to_owned(),
        subsystem: TESTING_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(report).map_err(|error| {
            ValidationHarnessError::StateStoreFailed {
                reason: format!("failed to serialize property check report checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ValidationHarnessError::from)?;
    Ok(checkpoint)
}

/// Append one local property-check report to the append-only audit journal.
pub fn append_property_check_report_audit(
    journal: &mut AppendOnlyAuditJournal,
    report: &LocalPropertyCheckReport,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ValidationHarnessError> {
    report.validate()?;
    let mut event = AuditEvent::new(
        format!("validation-property-check-{}", report.plan_id),
        AuditEventKind::RuntimeLifecycle,
        TESTING_STATE_SUBSYSTEM,
        "validation-property-check-runner",
        "local validation property checks recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "testing_backtesting_version",
            AuditValue::Text(TESTING_BACKTESTING_VERSION.to_owned()),
        )
        .with_metadata("plan_id", AuditValue::Text(report.plan_id.clone()))
        .with_metadata(
            "checks_executed",
            AuditValue::Text(report.checks_executed.to_string()),
        )
        .with_metadata(
            "checks_passed",
            AuditValue::Text(report.checks_passed.to_string()),
        )
        .with_metadata(
            "checks_failed",
            AuditValue::Text(report.checks_failed.to_string()),
        )
        .with_metadata(
            "forbidden_side_effect_flags_detected",
            AuditValue::Bool(report.forbidden_side_effect_flags_detected),
        )
        .with_metadata(
            "external_fuzzer_invoked",
            AuditValue::Bool(report.external_fuzzer_invoked),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(report.live_network_used),
        )
        .with_metadata(
            "live_execution_submitted",
            AuditValue::Bool(report.live_execution_submitted),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(report.signing_or_broadcast_performed),
        )
        .with_metadata(
            "production_ready",
            AuditValue::Bool(report.production_ready),
        );
    journal
        .append_event(event)
        .map_err(ValidationHarnessError::from)
}

/// Persist the latest local validation run through the typed state boundary.
///
/// This stores sanitized local validation-run metadata only. It does not launch
/// property tests, fuzzers, backtest runners, live-network checks, or execution.
pub fn persist_validation_run_checkpoint(
    store: &mut impl StateStore,
    record: &ValidationRunRecord,
    updated_at_unix_ms: u64,
) -> Result<StateCheckpoint, ValidationHarnessError> {
    record.validate()?;
    let checkpoint = StateCheckpoint {
        key: TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY.to_owned(),
        subsystem: TESTING_STATE_SUBSYSTEM.to_owned(),
        value: serde_json::to_string(record).map_err(|error| {
            ValidationHarnessError::StateStoreFailed {
                reason: format!("failed to serialize validation run checkpoint: {error}"),
            }
        })?,
        updated_at_unix_ms,
    };
    store
        .put_checkpoint(checkpoint.clone())
        .map_err(ValidationHarnessError::from)?;
    Ok(checkpoint)
}

/// Append one local validation run record to the append-only audit journal.
///
/// This records sanitized validation planning outcomes only. It does not execute
/// external fuzzers, live networks, live orders, signing, or broadcasts.
pub fn append_validation_run_audit(
    journal: &mut AppendOnlyAuditJournal,
    record: &ValidationRunRecord,
    occurred_at_unix_ms: u64,
) -> Result<AuditRecord, ValidationHarnessError> {
    record.validate()?;
    let mut event = AuditEvent::new(
        format!("validation-run-{}", record.plan_id),
        AuditEventKind::RuntimeLifecycle,
        TESTING_STATE_SUBSYSTEM,
        "validation-harness",
        "validation run recorded",
    );
    event.occurred_at_unix_ms = occurred_at_unix_ms;
    event = event
        .with_metadata(
            "testing_backtesting_version",
            AuditValue::Text(TESTING_BACKTESTING_VERSION.to_owned()),
        )
        .with_metadata("plan_id", AuditValue::Text(record.plan_id.clone()))
        .with_metadata("status", AuditValue::Text(format!("{:?}", record.status)))
        .with_metadata(
            "planned_test_cases",
            AuditValue::Text(record.planned_test_cases.to_string()),
        )
        .with_metadata(
            "planned_fixtures",
            AuditValue::Text(record.planned_fixtures.to_string()),
        )
        .with_metadata(
            "planned_fuzz_corpora",
            AuditValue::Text(record.planned_fuzz_corpora.to_string()),
        )
        .with_metadata(
            "planned_backtest_scenarios",
            AuditValue::Text(record.planned_backtest_scenarios.to_string()),
        )
        .with_metadata(
            "external_fuzzer_invoked",
            AuditValue::Bool(record.external_fuzzer_invoked),
        )
        .with_metadata(
            "live_network_used",
            AuditValue::Bool(record.live_network_used),
        )
        .with_metadata(
            "live_execution_submitted",
            AuditValue::Bool(record.live_execution_submitted),
        )
        .with_metadata(
            "signing_or_broadcast_performed",
            AuditValue::Bool(record.signing_or_broadcast_performed),
        )
        .with_metadata(
            "validation_text_redaction_applied",
            AuditValue::Bool(record.secret_redaction_applied),
        );
    journal
        .append_event(event)
        .map_err(ValidationHarnessError::from)
}

/// Deterministic validation harness boundary.
pub trait ValidationHarness {
    /// Validate a plan and return a local record without invoking external tooling.
    fn validate_plan(
        &self,
        request: ValidationRunRequest,
    ) -> Result<ValidationRunRecord, ValidationHarnessError>;
}

/// Local-only deterministic validation harness.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicValidationHarness;

impl ValidationHarness for DeterministicValidationHarness {
    fn validate_plan(
        &self,
        request: ValidationRunRequest,
    ) -> Result<ValidationRunRecord, ValidationHarnessError> {
        request.config.validate()?;

        if !request.config.local_validation_enabled {
            return Err(ValidationHarnessError::ValidationFailed {
                violations: vec![ValidationHarnessViolation::new(
                    "VALIDATION_LOCAL_HARNESS_DISABLED",
                    "local validation harness is disabled",
                )],
            });
        }

        request.plan.validate(&request.config)?;

        let mut redaction_applied = false;
        let operator_label = request.operator_label.as_ref().map(|label| {
            let (sanitized, redacted) = sanitize_validation_text(label, 256);
            redaction_applied |= redacted;
            sanitized
        });

        Ok(ValidationRunRecord {
            testing_backtesting_version: TESTING_BACKTESTING_VERSION.to_owned(),
            plan_id: request.plan.plan_id,
            status: ValidationRunStatus::PlannedOnly,
            requested_at_ms: request.requested_at_ms,
            operator_label,
            planned_test_cases: request.plan.test_cases.len(),
            planned_fixtures: request.plan.fixtures.len(),
            planned_fuzz_corpora: request.plan.fuzz_corpora.len(),
            planned_backtest_scenarios: request.plan.backtest_scenarios.len(),
            external_fuzzer_invoked: false,
            live_network_used: false,
            live_execution_submitted: false,
            signing_or_broadcast_performed: false,
            secret_redaction_applied: redaction_applied,
        })
    }
}

/// One deterministic validation harness violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationHarnessViolation {
    code: &'static str,
    message: String,
}

impl ValidationHarnessViolation {
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

/// Validation harness error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationHarnessError {
    /// Validation failed.
    ValidationFailed {
        /// Validation violations.
        violations: Vec<ValidationHarnessViolation>,
    },
    /// Local audit journal write failed.
    AuditJournalFailed {
        /// Non-secret failure reason.
        reason: String,
    },
    /// Local state-store write failed.
    StateStoreFailed {
        /// Non-secret failure reason.
        reason: String,
    },
}

impl ValidationHarnessError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[ValidationHarnessViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
            Self::AuditJournalFailed { .. } | Self::StateStoreFailed { .. } => &[],
        }
    }
}

impl From<crate::AuditError> for ValidationHarnessError {
    fn from(error: crate::AuditError) -> Self {
        Self::AuditJournalFailed {
            reason: error.to_string(),
        }
    }
}

impl From<StateStoreError> for ValidationHarnessError {
    fn from(error: StateStoreError) -> Self {
        Self::StateStoreFailed {
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for ValidationHarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed { violations } => {
                writeln!(
                    formatter,
                    "validation harness failed with {} violation(s):",
                    violations.len()
                )?;
                for violation in violations {
                    writeln!(formatter, "- {}: {}", violation.code(), violation.message())?;
                }
                Ok(())
            }
            Self::AuditJournalFailed { reason } => {
                write!(formatter, "validation audit journal failed: {reason}")
            }
            Self::StateStoreFailed { reason } => {
                write!(formatter, "validation state store failed: {reason}")
            }
        }
    }
}

impl Error for ValidationHarnessError {}

fn finish_validation(
    violations: Vec<ValidationHarnessViolation>,
) -> Result<(), ValidationHarnessError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ValidationHarnessError::ValidationFailed { violations })
    }
}

fn validate_count(
    label: &'static str,
    actual: usize,
    max_allowed: usize,
    code: &'static str,
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    if actual > max_allowed {
        violations.push(ValidationHarnessViolation::new_owned(
            code,
            format!("{label} count {actual} exceeds limit {max_allowed}"),
        ));
    }
}

fn validate_test_cases(
    cases: &[ValidationTestCase],
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    let mut ids = BTreeSet::new();

    for test_case in cases {
        validate_id("test case", &test_case.case_id, violations);
        validate_name("test case name", &test_case.name, violations);
        validate_name("test case subsystem", &test_case.subsystem, violations);

        if !ids.insert(test_case.case_id.to_ascii_lowercase()) {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_TEST_CASE_DUPLICATE",
                format!("test case {} is duplicated", test_case.case_id),
            ));
        }

        if test_case.requires_live_network {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_TEST_CASE_LIVE_NETWORK_DENIED",
                format!("test case {} requires live network", test_case.case_id),
            ));
        }

        if test_case.submits_live_execution {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_TEST_CASE_LIVE_EXECUTION_DENIED",
                format!("test case {} submits live execution", test_case.case_id),
            ));
        }

        for fixture_id in &test_case.fixture_ids {
            validate_id("test case fixture reference", fixture_id, violations);
        }
    }
}

fn validate_fixtures(
    fixtures: &[ValidationFixtureRecord],
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    let mut ids = BTreeSet::new();

    for fixture in fixtures {
        validate_id("fixture", &fixture.fixture_id, violations);
        validate_name("fixture source", &fixture.source, violations);

        if !ids.insert(fixture.fixture_id.to_ascii_lowercase()) {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_FIXTURE_DUPLICATE",
                format!("fixture {} is duplicated", fixture.fixture_id),
            ));
        }

        if fixture.contains_credentials {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_FIXTURE_CREDENTIALS_DENIED",
                format!(
                    "fixture {} is marked as credential-bearing",
                    fixture.fixture_id
                ),
            ));
        }

        if let Some(digest) = &fixture.payload_sha256 {
            validate_sha256("fixture payload", digest, violations);
        }
    }
}

fn validate_fuzz_corpora(
    corpora: &[FuzzCorpusDefinition],
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    let mut ids = BTreeSet::new();

    for corpus in corpora {
        validate_id("fuzz corpus", &corpus.corpus_id, violations);

        if !ids.insert(corpus.corpus_id.to_ascii_lowercase()) {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_FUZZ_CORPUS_DUPLICATE",
                format!("fuzz corpus {} is duplicated", corpus.corpus_id),
            ));
        }

        if corpus.external_fuzzer_required {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_FUZZ_CORPUS_EXTERNAL_FUZZER_DENIED",
                format!("fuzz corpus {} requires external fuzzer", corpus.corpus_id),
            ));
        }

        let mut seed_ids = BTreeSet::new();
        for seed in &corpus.seeds {
            validate_id("fuzz seed", &seed.seed_id, violations);
            validate_sha256("fuzz seed", &seed.seed_sha256, violations);
            validate_name("fuzz seed purpose", &seed.purpose, violations);
            if !seed_ids.insert(seed.seed_id.to_ascii_lowercase()) {
                violations.push(ValidationHarnessViolation::new_owned(
                    "VALIDATION_FUZZ_SEED_DUPLICATE",
                    format!(
                        "fuzz seed {} is duplicated in corpus {}",
                        seed.seed_id, corpus.corpus_id
                    ),
                ));
            }
        }
    }
}

fn validate_backtest_scenarios(
    scenarios: &[BacktestScenarioDefinition],
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    let mut ids = BTreeSet::new();

    for scenario in scenarios {
        validate_id("backtest scenario", &scenario.scenario_id, violations);
        if !ids.insert(scenario.scenario_id.to_ascii_lowercase()) {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_BACKTEST_SCENARIO_DUPLICATE",
                format!("backtest scenario {} is duplicated", scenario.scenario_id),
            ));
        }

        if scenario.initial_balance_microunits == 0 {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_BACKTEST_INITIAL_BALANCE_ZERO",
                format!(
                    "backtest scenario {} initial balance must be positive",
                    scenario.scenario_id
                ),
            ));
        }

        if scenario.max_drawdown_basis_points > 10_000 {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_BACKTEST_DRAWDOWN_INVALID",
                format!(
                    "backtest scenario {} max drawdown exceeds 10000 basis points",
                    scenario.scenario_id
                ),
            ));
        }

        if scenario.submits_live_orders {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_BACKTEST_LIVE_ORDERS_DENIED",
                format!(
                    "backtest scenario {} submits live orders",
                    scenario.scenario_id
                ),
            ));
        }

        if scenario.signs_or_broadcasts_transactions {
            violations.push(ValidationHarnessViolation::new_owned(
                "VALIDATION_BACKTEST_SIGN_OR_BROADCAST_DENIED",
                format!(
                    "backtest scenario {} signs or broadcasts transactions",
                    scenario.scenario_id
                ),
            ));
        }

        validate_dataset(&scenario.dataset, violations);
    }
}

fn validate_dataset(
    dataset: &BacktestDatasetDefinition,
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    validate_id("backtest dataset", &dataset.dataset_id, violations);
    validate_asset("base asset", &dataset.base_asset, violations);
    validate_asset("quote asset", &dataset.quote_asset, violations);

    if dataset
        .base_asset
        .eq_ignore_ascii_case(&dataset.quote_asset)
    {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_BACKTEST_PAIR_INVALID",
            format!(
                "backtest dataset {} uses identical assets",
                dataset.dataset_id
            ),
        ));
    }

    if dataset.start_ms >= dataset.end_ms {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_BACKTEST_TIME_RANGE_INVALID",
            format!(
                "backtest dataset {} has invalid time range",
                dataset.dataset_id
            ),
        ));
    }

    if dataset.quote_count == 0 {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_BACKTEST_QUOTE_COUNT_ZERO",
            format!("backtest dataset {} has no quotes", dataset.dataset_id),
        ));
    }

    if dataset.venue_ids.is_empty() {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_BACKTEST_VENUES_EMPTY",
            format!("backtest dataset {} has no venues", dataset.dataset_id),
        ));
    }

    if dataset.requires_live_network {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_BACKTEST_LIVE_NETWORK_DENIED",
            format!(
                "backtest dataset {} requires live network",
                dataset.dataset_id
            ),
        ));
    }

    for venue_id in &dataset.venue_ids {
        validate_id("backtest venue", venue_id, violations);
    }
}

fn validate_id(kind: &'static str, id: &str, violations: &mut Vec<ValidationHarnessViolation>) {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_ID_EMPTY",
            format!("{kind} id must be non-empty"),
        ));
    }
    if trimmed.len() > 128 {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_ID_TOO_LONG",
            format!("{kind} id is too long"),
        ));
    }
    if contains_secret_like_text(trimmed) {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_ID_SECRET_LIKE",
            format!("{kind} id looks like secret material"),
        ));
    }
}

fn validate_name(kind: &'static str, name: &str, violations: &mut Vec<ValidationHarnessViolation>) {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_NAME_EMPTY",
            format!("{kind} must be non-empty"),
        ));
    }
    if trimmed.len() > 160 {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_NAME_TOO_LONG",
            format!("{kind} is too long"),
        ));
    }
    if contains_secret_like_text(trimmed) {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_NAME_SECRET_LIKE",
            format!("{kind} looks like secret material"),
        ));
    }
}

fn validate_asset(
    kind: &'static str,
    asset: &str,
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    validate_name(kind, asset, violations);
    let invalid = asset.chars().any(|character| {
        !(character.is_ascii_alphanumeric() || character == '-' || character == '_')
    });
    if invalid {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_ASSET_INVALID",
            format!("{kind} contains unsupported characters"),
        ));
    }
}

fn validate_sha256(
    kind: &'static str,
    digest: &str,
    violations: &mut Vec<ValidationHarnessViolation>,
) {
    let valid = digest.len() == 64
        && digest
            .chars()
            .all(|character| character.is_ascii_hexdigit());
    if !valid {
        violations.push(ValidationHarnessViolation::new_owned(
            "VALIDATION_SHA256_INVALID",
            format!("{kind} digest must be a 64-character hexadecimal SHA-256 value"),
        ));
    }
}

fn sanitize_validation_text(text: &str, max_chars: usize) -> (String, bool) {
    let mut sanitized = text.trim().to_owned();
    let mut changed = sanitized.len() != text.len();

    if contains_secret_like_text(&sanitized) {
        sanitized = "[REDACTED SECRET-LIKE VALIDATION TEXT]".to_owned();
        changed = true;
    }

    if sanitized.chars().count() > max_chars {
        sanitized = sanitized.chars().take(max_chars).collect::<String>();
        sanitized.push('…');
        changed = true;
    }

    (sanitized, changed)
}

fn contains_secret_like_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let secret_markers = [
        "api_key=",
        "api-key=",
        "apikey=",
        "secret=",
        "private_key=",
        "private-key=",
        "seed_phrase=",
        "seed-phrase=",
        "mnemonic=",
        "bearer ",
        "authorization:",
        "provider token",
        "wallet key",
    ];

    secret_markers.iter().any(|marker| lower.contains(marker)) || looks_like_long_hex(&lower)
}

fn looks_like_long_hex(text: &str) -> bool {
    let longest = text
        .split(|character: char| !character.is_ascii_hexdigit())
        .map(str::len)
        .max()
        .unwrap_or_default();
    longest >= 48
}

#[cfg(test)]
mod tests {
    use super::{
        append_fuzz_corpus_replay_report_audit, append_property_check_report_audit,
        append_validation_corpus_report_audit, append_validation_run_audit,
        persist_fuzz_corpus_replay_report_checkpoint, persist_property_check_report_checkpoint,
        persist_validation_corpus_report_checkpoint, persist_validation_run_checkpoint,
        run_local_fuzz_corpus_replay, run_local_validation_corpus,
        run_local_validation_property_checks, BacktestDatasetDefinition,
        BacktestScenarioDefinition, DeterministicValidationHarness, ExpectedValidationOutcome,
        FixtureKind, FuzzCorpusDefinition, FuzzSeedRecord, FuzzTargetKind,
        LocalFuzzCorpusReplayReport, LocalFuzzCorpusReplayRequest, LocalFuzzCorpusReplayStatus,
        LocalPropertyCheckReport, LocalValidationCorpusReport, LocalValidationCorpusRequest,
        LocalValidationCorpusStatus, ValidationExecutionMode, ValidationFixtureRecord,
        ValidationHarness, ValidationHarnessConfig, ValidationHarnessError, ValidationPlan,
        ValidationRunRequest, ValidationRunStatus, ValidationSuiteKind, ValidationTestCase,
        TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY, TESTING_LAST_PROPERTY_CHECK_REPORT_KEY,
        TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY, TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY,
    };
    use crate::{AppendOnlyAuditJournal, SqliteWalStateStore, StateStore};
    use std::{env, fs, path::PathBuf, process};

    const VALID_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn minimal_plan() -> ValidationPlan {
        let mut policy_case = ValidationTestCase::new(
            "test-policy-deny-live",
            "Policy denies live execution in unsafe modes",
            ValidationSuiteKind::Policy,
            "policy",
            ExpectedValidationOutcome::FailClosed,
        );
        policy_case
            .fixture_ids
            .push("fixture-policy-intent-denied".to_owned());

        ValidationPlan {
            plan_id: "phase15-validation-plan".to_owned(),
            generated_at_ms: 1_700_000_000_000,
            execution_mode: ValidationExecutionMode::PlanOnly,
            test_cases: vec![policy_case],
            fixtures: vec![ValidationFixtureRecord::synthetic(
                "fixture-policy-intent-denied",
                FixtureKind::PolicyDecision,
                "fixtures/policy/intent_denied.json",
                Some(VALID_DIGEST.to_owned()),
                1_700_000_000_001,
            )],
            fuzz_corpora: vec![FuzzCorpusDefinition::local_only(
                "fuzz-corpus-command-parser",
                FuzzTargetKind::CommandParser,
                vec![FuzzSeedRecord::new(
                    "seed-help-command",
                    VALID_DIGEST,
                    "local command parser baseline",
                )],
            )],
            backtest_scenarios: vec![BacktestScenarioDefinition {
                scenario_id: "backtest-synthetic-btc-usd".to_owned(),
                dataset: BacktestDatasetDefinition {
                    dataset_id: "dataset-synthetic-btc-usd".to_owned(),
                    base_asset: "BTC".to_owned(),
                    quote_asset: "USD".to_owned(),
                    venue_ids: vec!["paper-a".to_owned(), "paper-b".to_owned()],
                    start_ms: 1_700_000_000_000,
                    end_ms: 1_700_003_600_000,
                    quote_count: 120,
                    local_fixture_only: true,
                    requires_live_network: false,
                },
                initial_balance_microunits: 1_000_000_000,
                max_drawdown_basis_points: 500,
                submits_live_orders: false,
                signs_or_broadcasts_transactions: false,
            }],
        }
    }

    fn minimal_plan_with_id(plan_id: &str) -> ValidationPlan {
        let mut plan = minimal_plan();
        plan.plan_id = plan_id.to_owned();
        plan
    }

    #[test]
    fn deterministic_harness_records_plan_without_external_side_effects() {
        let harness = DeterministicValidationHarness;
        let record = harness
            .validate_plan(ValidationRunRequest {
                config: ValidationHarnessConfig::default(),
                plan: minimal_plan(),
                requested_at_ms: 1_700_000_000_100,
                operator_label: Some("local validation".to_owned()),
            })
            .expect("local validation plan should be accepted");

        assert_eq!(record.status, ValidationRunStatus::PlannedOnly);
        assert_eq!(record.planned_test_cases, 1);
        assert_eq!(record.planned_fixtures, 1);
        assert_eq!(record.planned_fuzz_corpora, 1);
        assert_eq!(record.planned_backtest_scenarios, 1);
        assert!(!record.external_fuzzer_invoked);
        assert!(!record.live_network_used);
        assert!(!record.live_execution_submitted);
        assert!(!record.signing_or_broadcast_performed);
    }

    #[test]
    fn harness_config_rejects_live_and_external_toggles() {
        let config = ValidationHarnessConfig {
            external_fuzzer_invocation_enabled: true,
            live_network_tests_enabled: true,
            live_execution_tests_enabled: true,
            allow_secret_fixtures: true,
            ..ValidationHarnessConfig::default()
        };

        let error = config
            .validate()
            .expect_err("unsafe validation harness settings must fail closed");
        let ValidationHarnessError::ValidationFailed { violations } = error else {
            panic!("expected validation failure");
        };
        let codes = violations
            .iter()
            .map(super::ValidationHarnessViolation::code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"VALIDATION_EXTERNAL_FUZZER_DENIED_IN_PHASE_15"));
        assert!(codes.contains(&"VALIDATION_LIVE_NETWORK_TESTS_DENIED_IN_PHASE_15"));
        assert!(codes.contains(&"VALIDATION_LIVE_EXECUTION_TESTS_DENIED_IN_PHASE_15"));
        assert!(codes.contains(&"VALIDATION_SECRET_FIXTURES_DENIED"));
    }

    #[test]
    fn validation_plan_rejects_live_backtest_scope() {
        let mut plan = minimal_plan();
        plan.backtest_scenarios[0].dataset.requires_live_network = true;
        plan.backtest_scenarios[0].submits_live_orders = true;
        plan.backtest_scenarios[0].signs_or_broadcasts_transactions = true;

        let error = plan
            .validate(&ValidationHarnessConfig::default())
            .expect_err("live backtest scope must fail closed");
        let ValidationHarnessError::ValidationFailed { violations } = error else {
            panic!("expected validation failure");
        };
        let codes = violations
            .iter()
            .map(super::ValidationHarnessViolation::code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"VALIDATION_BACKTEST_LIVE_NETWORK_DENIED"));
        assert!(codes.contains(&"VALIDATION_BACKTEST_LIVE_ORDERS_DENIED"));
        assert!(codes.contains(&"VALIDATION_BACKTEST_SIGN_OR_BROADCAST_DENIED"));
    }

    #[test]
    fn operator_label_is_redacted_in_run_record() {
        let harness = DeterministicValidationHarness;
        let record = harness
            .validate_plan(ValidationRunRequest {
                config: ValidationHarnessConfig::default(),
                plan: minimal_plan(),
                requested_at_ms: 1_700_000_000_200,
                operator_label: Some(
                    concat!("api", "_key=not-a-real-value-for-test-only").to_owned(),
                ),
            })
            .expect("secret-like operator label should be redacted");

        assert!(record.secret_redaction_applied);
        assert_eq!(
            record.operator_label.as_deref(),
            Some("[REDACTED SECRET-LIKE VALIDATION TEXT]")
        );
    }

    #[test]
    fn validation_run_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("validation-run");
        let state_path = temp_state_path("validation-run");
        let harness = DeterministicValidationHarness;
        let record = harness
            .validate_plan(ValidationRunRequest {
                config: ValidationHarnessConfig::default(),
                plan: minimal_plan(),
                requested_at_ms: 1_700_000_000_300,
                operator_label: Some("local-validation-review".to_owned()),
            })
            .expect("local validation plan should be accepted");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record = append_validation_run_audit(&mut journal, &record, 1_700_000_000_301)
            .expect("validation run audit writes");
        let checkpoint = persist_validation_run_checkpoint(&mut store, &record, 1_700_000_000_302)
            .expect("validation run checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY);
        assert!(!record.external_fuzzer_invoked);
        assert!(!record.live_network_used);
        assert!(!record.live_execution_submitted);
        assert!(!record.signing_or_broadcast_performed);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(TESTING_LAST_VALIDATION_RUN_CHECKPOINT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("validation run checkpoint exists");
        assert_eq!(recovered.value, checkpoint.value);
        assert!(recovered
            .value
            .contains("\"external_fuzzer_invoked\":false"));
        assert!(recovered.value.contains("\"live_network_used\":false"));
        assert!(recovered
            .value
            .contains("\"live_execution_submitted\":false"));
        assert!(recovered
            .value
            .contains("\"signing_or_broadcast_performed\":false"));

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn local_property_check_runner_validates_plan_invariants() {
        let report = run_local_validation_property_checks(
            &minimal_plan(),
            &ValidationHarnessConfig::default(),
        )
        .expect("local property checks should pass");

        assert_eq!(report.checks_executed, 4);
        assert_eq!(report.checks_passed, 4);
        assert_eq!(report.checks_failed, 0);
        assert!(report.missing_fixture_references.is_empty());
        assert!(report.empty_fuzz_corpora.is_empty());
        assert!(report.nonlocal_backtest_datasets.is_empty());
        assert!(!report.forbidden_side_effect_flags_detected);
        assert!(!report.external_fuzzer_invoked);
        assert!(!report.live_network_used);
        assert!(!report.live_execution_submitted);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn local_property_check_runner_reports_metadata_failures() {
        let mut plan = minimal_plan();
        plan.test_cases[0]
            .fixture_ids
            .push("missing-fixture-reference".to_owned());
        plan.fuzz_corpora.push(FuzzCorpusDefinition::local_only(
            "empty-local-fuzz-corpus",
            FuzzTargetKind::Redaction,
            Vec::new(),
        ));
        plan.backtest_scenarios[0].dataset.local_fixture_only = false;

        let report =
            run_local_validation_property_checks(&plan, &ValidationHarnessConfig::default())
                .expect("metadata failures should be reported without external side effects");

        assert_eq!(report.checks_executed, 4);
        assert_eq!(report.checks_failed, 3);
        assert_eq!(
            report.missing_fixture_references,
            vec!["missing-fixture-reference".to_owned()]
        );
        assert_eq!(
            report.empty_fuzz_corpora,
            vec!["empty-local-fuzz-corpus".to_owned()]
        );
        assert_eq!(
            report.nonlocal_backtest_datasets,
            vec!["dataset-synthetic-btc-usd".to_owned()]
        );
        assert!(!report.forbidden_side_effect_flags_detected);
        assert!(!report.external_fuzzer_invoked);
        assert!(!report.live_network_used);
        assert!(!report.live_execution_submitted);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn local_property_check_report_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("property-check");
        let state_path = temp_state_path("property-check");
        let report = run_local_validation_property_checks(
            &minimal_plan(),
            &ValidationHarnessConfig::default(),
        )
        .expect("local property checks should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_property_check_report_audit(&mut journal, &report, 1_700_000_000_401)
                .expect("property check audit writes");
        let checkpoint =
            persist_property_check_report_checkpoint(&mut store, &report, 1_700_000_000_402)
                .expect("property check checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, TESTING_LAST_PROPERTY_CHECK_REPORT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(TESTING_LAST_PROPERTY_CHECK_REPORT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("property check checkpoint exists");
        let recovered_report: LocalPropertyCheckReport =
            serde_json::from_str(&recovered.value).expect("checkpoint decodes");
        assert_eq!(recovered_report.plan_id, "phase15-validation-plan");
        assert_eq!(recovered_report.checks_failed, 0);
        assert!(!recovered_report.external_fuzzer_invoked);
        assert!(!recovered_report.live_network_used);
        assert!(!recovered_report.live_execution_submitted);
        assert!(!recovered_report.signing_or_broadcast_performed);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn local_fuzz_corpus_replay_summarizes_seed_metadata_without_external_fuzzer() {
        let report = run_local_fuzz_corpus_replay(LocalFuzzCorpusReplayRequest {
            replay_id: "phase15-local-fuzz-replay".to_owned(),
            config: ValidationHarnessConfig::default(),
            fuzz_corpora: vec![
                FuzzCorpusDefinition::local_only(
                    "fuzz-command-parser",
                    FuzzTargetKind::CommandParser,
                    vec![
                        FuzzSeedRecord::new(
                            "seed-help-command",
                            VALID_DIGEST,
                            "local help command seed",
                        ),
                        FuzzSeedRecord::new(
                            "seed-status-command",
                            VALID_DIGEST,
                            "local status command seed",
                        ),
                    ],
                ),
                FuzzCorpusDefinition::local_only(
                    "fuzz-redaction",
                    FuzzTargetKind::Redaction,
                    vec![FuzzSeedRecord::new(
                        "seed-redaction-placeholder",
                        VALID_DIGEST,
                        "local redaction placeholder seed",
                    )],
                ),
            ],
            requested_at_ms: 1_700_000_000_450,
            operator_label: Some("local fuzz replay".to_owned()),
        })
        .expect("local fuzz corpus replay should pass");

        assert_eq!(
            report.status,
            LocalFuzzCorpusReplayStatus::ReadyForLocalReview
        );
        assert_eq!(report.corpus_count, 2);
        assert_eq!(report.seed_count, 3);
        assert_eq!(report.unique_seed_count, 3);
        assert_eq!(report.target_summaries.len(), 2);
        assert!(report
            .target_summaries
            .iter()
            .any(|summary| summary.target == FuzzTargetKind::CommandParser
                && summary.corpus_count == 1
                && summary.seed_count == 2));
        assert!(report
            .target_summaries
            .iter()
            .any(|summary| summary.target == FuzzTargetKind::Redaction
                && summary.corpus_count == 1
                && summary.seed_count == 1));
        assert!(!report.external_fuzzer_invoked);
        assert!(!report.live_network_used);
        assert!(!report.live_execution_submitted);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn local_fuzz_corpus_replay_rejects_external_fuzzer_requests() {
        let mut corpus = FuzzCorpusDefinition::local_only(
            "fuzz-external-denied",
            FuzzTargetKind::ExecutionPlanner,
            vec![FuzzSeedRecord::new(
                "seed-execution-planner",
                VALID_DIGEST,
                "local execution planner seed",
            )],
        );
        corpus.external_fuzzer_required = true;

        let error = run_local_fuzz_corpus_replay(LocalFuzzCorpusReplayRequest {
            replay_id: "phase15-external-fuzzer-denied".to_owned(),
            config: ValidationHarnessConfig::default(),
            fuzz_corpora: vec![corpus],
            requested_at_ms: 1_700_000_000_451,
            operator_label: None,
        })
        .expect_err("external fuzzer request must fail closed");
        let ValidationHarnessError::ValidationFailed { violations } = error else {
            panic!("expected validation failure");
        };
        assert!(violations
            .iter()
            .map(super::ValidationHarnessViolation::code)
            .any(|code| code == "VALIDATION_FUZZ_CORPUS_EXTERNAL_FUZZER_DENIED"));
    }

    #[test]
    fn local_fuzz_corpus_replay_report_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("fuzz-replay");
        let state_path = temp_state_path("fuzz-replay");
        let report = run_local_fuzz_corpus_replay(LocalFuzzCorpusReplayRequest {
            replay_id: "phase15-local-fuzz-replay".to_owned(),
            config: ValidationHarnessConfig::default(),
            fuzz_corpora: minimal_plan().fuzz_corpora,
            requested_at_ms: 1_700_000_000_452,
            operator_label: Some("local-fuzz-replay".to_owned()),
        })
        .expect("local fuzz corpus replay should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_fuzz_corpus_replay_report_audit(&mut journal, &report, 1_700_000_000_453)
                .expect("fuzz replay audit writes");
        let checkpoint =
            persist_fuzz_corpus_replay_report_checkpoint(&mut store, &report, 1_700_000_000_454)
                .expect("fuzz replay checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(TESTING_LAST_FUZZ_CORPUS_REPLAY_REPORT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("fuzz replay checkpoint exists");
        let recovered_report: LocalFuzzCorpusReplayReport =
            serde_json::from_str(&recovered.value).expect("checkpoint decodes");
        assert_eq!(recovered_report.replay_id, "phase15-local-fuzz-replay");
        assert_eq!(recovered_report.seed_count, 1);
        assert!(!recovered_report.external_fuzzer_invoked);
        assert!(!recovered_report.live_network_used);
        assert!(!recovered_report.live_execution_submitted);
        assert!(!recovered_report.signing_or_broadcast_performed);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    #[test]
    fn local_validation_corpus_aggregates_local_plans_without_side_effects() {
        let report = run_local_validation_corpus(LocalValidationCorpusRequest {
            corpus_id: "phase15-local-validation-corpus".to_owned(),
            config: ValidationHarnessConfig::default(),
            plans: vec![
                minimal_plan_with_id("phase15-validation-plan-a"),
                minimal_plan_with_id("phase15-validation-plan-b"),
            ],
            min_plan_count: 2,
            min_test_case_count: 2,
            min_fixture_count: 2,
            min_fuzz_corpus_count: 2,
            min_backtest_scenario_count: 2,
            requested_at_ms: 1_700_000_000_500,
            operator_label: Some("local corpus review".to_owned()),
        })
        .expect("local validation corpus should pass");

        assert_eq!(
            report.status,
            LocalValidationCorpusStatus::ReadyForLocalReview
        );
        assert_eq!(report.plan_count, 2);
        assert_eq!(report.accepted_plan_count, 2);
        assert_eq!(report.planned_test_cases, 2);
        assert_eq!(report.planned_fixtures, 2);
        assert_eq!(report.planned_fuzz_corpora, 2);
        assert_eq!(report.planned_backtest_scenarios, 2);
        assert_eq!(report.property_checks_executed, 8);
        assert_eq!(report.property_checks_passed, 8);
        assert_eq!(report.property_checks_failed, 0);
        assert_eq!(report.min_plan_count, 2);
        assert_eq!(report.min_test_case_count, 2);
        assert_eq!(report.min_fixture_count, 2);
        assert_eq!(report.min_fuzz_corpus_count, 2);
        assert_eq!(report.min_backtest_scenario_count, 2);
        assert!(report.corpus_breadth_requirements_met);
        assert!(!report.external_fuzzer_invoked);
        assert!(!report.live_network_used);
        assert!(!report.live_execution_submitted);
        assert!(!report.signing_or_broadcast_performed);
        assert!(!report.production_ready);
    }

    #[test]
    fn local_validation_corpus_rejects_empty_requests() {
        let error = run_local_validation_corpus(LocalValidationCorpusRequest {
            corpus_id: "phase15-empty-validation-corpus".to_owned(),
            config: ValidationHarnessConfig::default(),
            plans: Vec::new(),
            min_plan_count: 1,
            min_test_case_count: 1,
            min_fixture_count: 1,
            min_fuzz_corpus_count: 1,
            min_backtest_scenario_count: 1,
            requested_at_ms: 1_700_000_000_600,
            operator_label: None,
        })
        .expect_err("empty validation corpus must fail closed");
        let ValidationHarnessError::ValidationFailed { violations } = error else {
            panic!("expected validation failure");
        };
        assert!(violations
            .iter()
            .map(super::ValidationHarnessViolation::code)
            .any(|code| code == "VALIDATION_CORPUS_REQUEST_EMPTY"));
    }

    #[test]
    fn local_validation_corpus_rejects_insufficient_breadth() {
        let error = run_local_validation_corpus(LocalValidationCorpusRequest {
            corpus_id: "phase15-narrow-validation-corpus".to_owned(),
            config: ValidationHarnessConfig::default(),
            plans: vec![minimal_plan_with_id("phase15-validation-plan-a")],
            min_plan_count: 2,
            min_test_case_count: 2,
            min_fixture_count: 2,
            min_fuzz_corpus_count: 2,
            min_backtest_scenario_count: 2,
            requested_at_ms: 1_700_000_000_650,
            operator_label: None,
        })
        .expect_err("narrow validation corpus must fail closed");
        let ValidationHarnessError::ValidationFailed { violations } = error else {
            panic!("expected validation failure");
        };
        assert!(violations
            .iter()
            .map(super::ValidationHarnessViolation::code)
            .any(|code| code == "VALIDATION_CORPUS_BREADTH_REQUIREMENTS_NOT_MET"));
    }

    #[test]
    fn local_validation_corpus_report_audit_and_state_reopen_locally() {
        let audit_path = temp_audit_path("validation-corpus");
        let state_path = temp_state_path("validation-corpus");
        let report = run_local_validation_corpus(LocalValidationCorpusRequest {
            corpus_id: "phase15-local-validation-corpus".to_owned(),
            config: ValidationHarnessConfig::default(),
            plans: vec![
                minimal_plan_with_id("phase15-validation-plan-a"),
                minimal_plan_with_id("phase15-validation-plan-b"),
            ],
            min_plan_count: 2,
            min_test_case_count: 2,
            min_fixture_count: 2,
            min_fuzz_corpus_count: 2,
            min_backtest_scenario_count: 2,
            requested_at_ms: 1_700_000_000_700,
            operator_label: Some("local-corpus-review".to_owned()),
        })
        .expect("local validation corpus should pass");
        let mut journal = AppendOnlyAuditJournal::open(&audit_path).expect("journal opens");
        let mut store = SqliteWalStateStore::open(&state_path).expect("sqlite opens");

        let audit_record =
            append_validation_corpus_report_audit(&mut journal, &report, 1_700_000_000_701)
                .expect("validation corpus audit writes");
        let checkpoint =
            persist_validation_corpus_report_checkpoint(&mut store, &report, 1_700_000_000_702)
                .expect("validation corpus checkpoint writes");
        assert_eq!(audit_record.sequence, 1);
        assert_eq!(checkpoint.key, TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY);
        drop(store);
        drop(journal);

        let replayed = AppendOnlyAuditJournal::open(&audit_path).expect("journal replays");
        assert_eq!(replayed.next_sequence(), 2);
        let reopened = SqliteWalStateStore::open(&state_path).expect("sqlite reopens");
        let recovered = reopened
            .get_checkpoint(TESTING_LAST_VALIDATION_CORPUS_REPORT_KEY)
            .expect("checkpoint lookup succeeds")
            .expect("validation corpus checkpoint exists");
        let recovered_report: LocalValidationCorpusReport =
            serde_json::from_str(&recovered.value).expect("checkpoint decodes");
        assert_eq!(recovered_report.plan_count, 2);
        assert_eq!(recovered_report.property_checks_failed, 0);
        assert!(recovered_report.corpus_breadth_requirements_met);
        assert!(recovered
            .value
            .contains("\"corpus_breadth_requirements_met\":true"));
        assert!(!recovered_report.external_fuzzer_invoked);
        assert!(!recovered_report.live_network_used);
        assert!(!recovered_report.live_execution_submitted);
        assert!(!recovered_report.signing_or_broadcast_performed);
        assert!(!recovered_report.production_ready);

        let _ = fs::remove_file(audit_path);
        cleanup_state_files(&state_path);
    }

    fn temp_audit_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!("arbyclaw-{label}-{}-{nanos}.jsonl", process::id()));
        path
    }

    fn temp_state_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!(
            "arbyclaw-{label}-{}-{nanos}.sqlite3",
            process::id()
        ));
        path
    }

    fn cleanup_state_files(path: &PathBuf) {
        let _ = fs::remove_file(path);
        for suffix in ["-wal", "-shm"] {
            let related = format!("{}{}", path.display(), suffix);
            let _ = fs::remove_file(related);
        }
    }
}
