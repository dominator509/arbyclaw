#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Stable testing, fuzzing, and backtesting boundary version for audit and handoff surfaces.
pub const TESTING_BACKTESTING_VERSION: &str = "phase-15-testing-fuzzing-backtesting-v1";

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
}

impl ValidationHarnessError {
    /// Return validation violations.
    #[must_use]
    pub fn violations(&self) -> &[ValidationHarnessViolation] {
        match self {
            Self::ValidationFailed { violations } => violations,
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
        BacktestDatasetDefinition, BacktestScenarioDefinition, DeterministicValidationHarness,
        ExpectedValidationOutcome, FixtureKind, FuzzCorpusDefinition, FuzzSeedRecord,
        FuzzTargetKind, ValidationExecutionMode, ValidationFixtureRecord, ValidationHarness,
        ValidationHarnessConfig, ValidationHarnessError, ValidationPlan, ValidationRunRequest,
        ValidationRunStatus, ValidationSuiteKind, ValidationTestCase,
    };

    const VALID_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn minimal_plan() -> ValidationPlan {
        ValidationPlan {
            plan_id: "phase15-validation-plan".to_owned(),
            generated_at_ms: 1_700_000_000_000,
            execution_mode: ValidationExecutionMode::PlanOnly,
            test_cases: vec![ValidationTestCase::new(
                "test-policy-deny-live",
                "Policy denies live execution in unsafe modes",
                ValidationSuiteKind::Policy,
                "policy",
                ExpectedValidationOutcome::FailClosed,
            )],
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
        let ValidationHarnessError::ValidationFailed { violations } = error;
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
        let ValidationHarnessError::ValidationFailed { violations } = error;
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
}
