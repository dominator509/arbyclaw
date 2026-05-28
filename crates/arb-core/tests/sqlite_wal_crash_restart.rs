#![allow(clippy::missing_panics_doc)]

use arb_core::{
    SqliteWalStateStore, StateCheckpoint, StateStore, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY,
    EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

const CHILD_SCENARIO_ENV: &str = "ARBYCLAW_CRASH_RESTART_SCENARIO";
const CHILD_DB_ENV: &str = "ARBYCLAW_CRASH_RESTART_DB";
const CHILD_EXIT_CODE: i32 = 42;
const START_CHECKPOINT_KEY: &str = "runtime-crash-restart:start";

#[test]
fn sqlite_wal_crash_restart_recovers_committed_checkpoints() {
    if env::var_os(CHILD_SCENARIO_ENV).is_some() {
        return;
    }

    let scenarios = [
        CrashScenario::Start,
        CrashScenario::Plan,
        CrashScenario::Adapter,
    ];
    for scenario in scenarios {
        let path = unique_state_path(scenario.as_str());
        let status = Command::new(env::current_exe().expect("test binary path should resolve"))
            .arg("--exact")
            .arg("crash_restart_child_entrypoint")
            .arg("--nocapture")
            .env(CHILD_SCENARIO_ENV, scenario.as_str())
            .env(CHILD_DB_ENV, &path)
            .status()
            .expect("child crash-restart test process should launch");

        assert_eq!(status.code(), Some(CHILD_EXIT_CODE));
        assert_recovered_state(&path, scenario);
        cleanup_state_files(&path);
    }
}

#[test]
fn crash_restart_child_entrypoint() {
    let Some(scenario) = env::var_os(CHILD_SCENARIO_ENV) else {
        return;
    };
    let scenario = CrashScenario::parse(&scenario.to_string_lossy());
    let path = env::var_os(CHILD_DB_ENV).expect("child database path should be set");
    write_scenario_checkpoints(&PathBuf::from(path), scenario);
    process::exit(CHILD_EXIT_CODE);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrashScenario {
    Start,
    Plan,
    Adapter,
}

impl CrashScenario {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "after-start",
            Self::Plan => "after-plan",
            Self::Adapter => "after-adapter",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "after-start" => Self::Start,
            "after-plan" => Self::Plan,
            "after-adapter" => Self::Adapter,
            other => panic!("unknown crash scenario: {other}"),
        }
    }
}

fn write_scenario_checkpoints(path: &Path, scenario: CrashScenario) {
    let mut store = SqliteWalStateStore::open(path).expect("child sqlite store should open");
    put_checkpoint(
        &mut store,
        START_CHECKPOINT_KEY,
        "runtime-crash-restart",
        scenario.as_str(),
        1,
    );

    if matches!(scenario, CrashScenario::Plan | CrashScenario::Adapter) {
        put_checkpoint(
            &mut store,
            EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
            "execution-planner",
            "plan-checkpoint-survived-crash",
            2,
        );
    }

    if scenario == CrashScenario::Adapter {
        put_checkpoint(
            &mut store,
            EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY,
            "execution-adapter",
            "adapter-checkpoint-survived-crash",
            3,
        );
    }
}

fn put_checkpoint(
    store: &mut SqliteWalStateStore,
    key: &str,
    subsystem: &str,
    value: &str,
    updated_at_unix_ms: u64,
) {
    store
        .put_checkpoint(StateCheckpoint {
            key: key.to_owned(),
            subsystem: subsystem.to_owned(),
            value: value.to_owned(),
            updated_at_unix_ms,
        })
        .expect("checkpoint should persist before child exits");
}

fn assert_recovered_state(path: &Path, scenario: CrashScenario) {
    let store = SqliteWalStateStore::open(path).expect("parent sqlite store should reopen");
    store
        .integrity_check()
        .expect("database should pass integrity check after child exit");

    assert_checkpoint(&store, START_CHECKPOINT_KEY, Some(scenario.as_str()));
    match scenario {
        CrashScenario::Start => {
            assert_checkpoint(&store, EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY, None);
            assert_checkpoint(&store, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, None);
        }
        CrashScenario::Plan => {
            assert_checkpoint(
                &store,
                EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
                Some("plan-checkpoint-survived-crash"),
            );
            assert_checkpoint(&store, EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY, None);
        }
        CrashScenario::Adapter => {
            assert_checkpoint(
                &store,
                EXECUTION_PLANNER_LAST_DRAFT_CHECKPOINT_KEY,
                Some("plan-checkpoint-survived-crash"),
            );
            assert_checkpoint(
                &store,
                EXECUTION_ADAPTER_LAST_RUN_CHECKPOINT_KEY,
                Some("adapter-checkpoint-survived-crash"),
            );
        }
    }
}

fn assert_checkpoint(store: &SqliteWalStateStore, key: &str, expected_value: Option<&str>) {
    let checkpoint = store
        .get_checkpoint(key)
        .expect("checkpoint read should complete");
    match expected_value {
        Some(value) => {
            let checkpoint = checkpoint.expect("checkpoint should exist");
            assert_eq!(checkpoint.value, value);
        }
        None => assert!(checkpoint.is_none()),
    }
}

fn unique_state_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "arbyclaw-crash-restart-{label}-{}-{nanos}.sqlite3",
        process::id()
    ))
}

fn cleanup_state_files(path: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", path.display())));
    }
}
