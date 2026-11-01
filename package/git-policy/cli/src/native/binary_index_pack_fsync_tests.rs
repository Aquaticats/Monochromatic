//! The migration's `index-pack` child runs with `-c core.fsync=-pack,-pack-metadata` and no other Git child does.
//! Why:
//!  The override removes the three flushes of the migrated pack (`doc/handover/wrapper-sync-attribution.md`, item 4)
//!  and replaces a `core.fsync` the user configured, for that one child only.
//!  Two controls pin it on the built wrapper: Git's own trace2 `def_param` events list the `core.fsync` values each child
//!  process saw, in the order Git applied them, so the last one is the effective setting; and a `strace` count shows
//!  that a user's `core.fsync=all` still leaves the child with no flush.
//!  The count controls of `binary_budget_tests.rs` pin the same saving for a default configuration.

use super::budget_trace::{TRACED_CALLS, count};
use super::commit::{assert_nothing_left, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::fork_failure::{strace_available, strace_program};
use super::support::{Fixture, REAL_GIT, fixture, git, repository};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The explicit-path commit every control runs.
const COMMIT: [&str; 5] = ["commit", "--quiet", "--message=x", "--", "a.txt"];

/// A repository whose own configuration asks Git to flush everything, with `a.txt` staged for the commit.
fn repository_asking_for_every_flush(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    git(fixture, repo.as_path(), &["config", "core.fsync", "all"]);
    return repo;
}

/// Require the commit to have landed and to have left nothing behind.
fn require_landed(label: &str, fixture: &Fixture, repo: &Path, run: &Output) {
    assert_eq!(run.status.code(), Some(0), "{label}: the commit failed: {}", String::from_utf8_lossy(&run.stderr));
    let head_count = git(fixture, repo, &["rev-list", "--count", "HEAD"]);
    assert_eq!(String::from_utf8_lossy(&head_count.stdout).trim(), "2", "{label}: commits");
    assert_nothing_left(fixture, repo);
}

/// The `core.fsync` settings each Git process of the trace saw, as `scope:value` in application order, by command name.
///  A process appears once per `cmd_name` event; processes of the same command are kept apart.
fn fsync_settings_by_process(trace: &str) -> Vec<(String, Vec<String>)> {
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    let mut settings: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for line in trace.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(process) = event["sid"].as_str() else {
            continue;
        };
        match event["event"].as_str() {
            Some("cmd_name") => {
                names.insert(process.to_string(), event["name"].as_str().unwrap_or("").to_string());
                order.push(process.to_string());
            }
            Some("def_param") if event["param"] == "core.fsync" => {
                let scope: &str = event["scope"].as_str().unwrap_or("");
                let value: &str = event["value"].as_str().unwrap_or("");
                settings.entry(process.to_string()).or_default().push(format!("{scope}:{value}"));
            }
            _ => {}
        }
    }
    return order
        .into_iter()
        .map(|process| {
            let seen: Vec<String> = settings.remove(&process).unwrap_or_default();
            return (names.remove(&process).unwrap_or_default(), seen);
        })
        .collect();
}

/// The migration's `index-pack` ends with the override as its last `core.fsync`, after the user's own value, and no
///  other child of the commit gets a `core.fsync` from the command line.
#[test]
fn only_the_migration_index_pack_gets_the_fsync_override() {
    let fixture: Fixture = fixture("index-pack-fsync-config");
    let repo: PathBuf = repository_asking_for_every_flush(&fixture, "config");
    let trace: PathBuf = fixture.root.join("trace2.jsonl");
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &COMMIT);
    command.env("GIT_TRACE2_EVENT", &trace).env("GIT_TRACE2_CONFIG_PARAMS", "core.fsync");
    let run: Output = command.output().expect("the commit starts");
    require_landed("config", &fixture, repo.as_path(), &run);
    let text: String = std::fs::read_to_string(&trace).expect("the trace2 log");
    let processes: Vec<(String, Vec<String>)> = fsync_settings_by_process(text.as_str());
    let index_packs: Vec<&Vec<String>> =
        processes.iter().filter(|(name, _)| { return name == "index-pack"; }).map(|(_, seen)| { return seen; }).collect();
    assert_eq!(index_packs.len(), 1, "one migration index-pack starts in a commit: {processes:?}");
    assert_eq!(
        index_packs[0].as_slice(),
        [String::from("local:all"), String::from("command:-pack,-pack-metadata")],
        "the user's setting reaches the child and the override is applied after it, so the override is effective"
    );
    for (name, seen) in processes.iter().filter(|(name, _)| { return name != "index-pack"; }) {
        assert!(
            seen.iter().all(|setting| { return !setting.starts_with("command:"); }),
            "git {name} must not get a core.fsync from the command line: {seen:?}"
        );
    }
    remove_settled(&fixture);
}

/// With a user's `core.fsync=all` in the repository, the migration `index-pack` still makes no flush.
#[test]
fn the_fsync_override_beats_a_users_core_fsync() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no sync call was counted");
        return;
    }
    let fixture: Fixture = fixture("index-pack-fsync-count");
    let repo: PathBuf = repository_asking_for_every_flush(&fixture, "count");
    let trace: PathBuf = fixture.root.join("trace.txt");
    let inner: Command = wrapped_commit(&fixture, repo.as_path(), &COMMIT);
    let mut outer: Command = Command::new(strace_program().expect("strace is installed"));
    outer
        .args(["--follow-forks", "--quiet", "--quiet", "--string-limit=4096", "--no-abbrev"])
        .arg(format!("--output={}", trace.display()))
        .arg(format!("--trace={TRACED_CALLS}"))
        .arg(inner.get_program())
        .args(inner.get_args())
        .env_clear();
    for (name, setting) in inner.get_envs() {
        if let Some(text) = setting {
            outer.env(name, text);
        }
    }
    outer.current_dir(repo.as_path());
    let run: Output = outer.output().expect("strace starts");
    require_landed("count", &fixture, repo.as_path(), &run);
    let text: String = std::fs::read_to_string(&trace).expect("trace");
    let counts = count(text.as_str(), fixture.root.to_str().expect("UTF-8 root"), REAL_GIT);
    assert_eq!(counts.git_starts.get("index-pack"), Some(&1), "one migration index-pack starts: {:?}", counts.commands);
    assert_eq!(
        counts.git_syncs.get("index-pack").copied().unwrap_or(0),
        0,
        "the migration index-pack flushed although the override removes the pack components: {:?}",
        counts.commands
    );
    remove_settled(&fixture);
}
