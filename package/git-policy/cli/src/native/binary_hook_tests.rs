//! The hook dispatcher through the built executable:
//!  hooks run once, run in the right place, propagate status, serialize on the hook lock
//!  unless `hooks.concurrentCommits` allows overlap, and start names outside a transaction behave as `git`.
//! Why:
//!  Git starts the executable by the entry's path, so these controls prove the start name and the
//!  transaction shape end to end with real Git 2.55.0 or 2.56.0 underneath.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git commit with a hooks path of <tx>/hooks; each entry runs the dispatcher once
//! ```

/// Fixture and process helpers shared by every binary control.
use super::support::{
    Fixture, REAL_GIT, TRANSACTION_ID, bounded, executable, fixture, observe, registered_transaction,
    remove, repository, wrapper_first_path,
};
use git_policy_cli::hook_dispatch_entries::write_hook_dispatch;
use git_policy_cli::hook_dispatch_plan::{
    HookDispatchPlan, HookPlanInputs, compute_hook_dispatch_plan, new_preparation_lease,
};
use git_policy_cli::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Poll interval for barrier files.
const POLL: Duration = Duration::from_millis(20);

/// How long a barrier waits before the control fails.
const BARRIER_LIMIT: Duration = Duration::from_secs(20);

/// Environment every Git and dispatched hook sees: no host configuration.
fn hermetic_environment(fixture: &Fixture) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("PATH"), wrapper_first_path(fixture)),
        (OsString::from("HOME"), fixture.root.join("home").into_os_string()),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
        (OsString::from("GIT_AUTHOR_NAME"), OsString::from("Fixture")),
        (OsString::from("GIT_AUTHOR_EMAIL"), OsString::from("fixture@example.invalid")),
        (OsString::from("GIT_COMMITTER_NAME"), OsString::from("Fixture")),
        (OsString::from("GIT_COMMITTER_EMAIL"), OsString::from("fixture@example.invalid")),
        (OsString::from("LC_ALL"), OsString::from("C")),
    ];
}

/// Real Git's context with the hermetic environment, as the incumbent's config reads use.
fn context(fixture: &Fixture) -> GitContext {
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: hermetic_environment(fixture),
        global_prefix: Vec::new(),
    };
}

/// A command that starts `program` with the hermetic environment inside `directory`.
fn hermetic_command(fixture: &Fixture, program: &Path, directory: &Path) -> Command {
    let mut command: Command = Command::new(program);
    command
        .env_clear()
        .envs(hermetic_environment(fixture))
        .current_dir(directory);
    return command;
}

/// A repository whose own hooks path is `shared`, with a published transaction whose entries
/// start the native executable and whose plan comes from the real repository.
struct Prepared {
    repository: PathBuf,
    hooks: PathBuf,
    plan: HookDispatchPlan,
}

/// Build the transaction for one repository.
fn prepared(fixture: &Fixture, name: &str, shared: &Path, concurrent_commits: bool) -> Prepared {
    let created: PathBuf = repository(fixture, std::ffi::OsStr::new(name));
    let repository_root: PathBuf = std::fs::canonicalize(created).expect("canonical");
    std::fs::create_dir_all(repository_root.join(".git").join("cli-git")).expect("cli-git");
    let mut config: Command = hermetic_command(fixture, Path::new(REAL_GIT), repository_root.as_path());
    config.args(["config", "core.hooksPath", shared.to_str().expect("UTF-8")]);
    assert!(config.status().expect("config").success());
    let registry: PathBuf = registered_transaction(repository_root.as_path(), true);
    let directory: PathBuf = registry.join(TRANSACTION_ID);
    let lease: String = new_preparation_lease(directory.as_path()).expect("lease");
    let common: PathBuf = repository_root.join(".git");
    let plan: HookDispatchPlan = compute_hook_dispatch_plan(&HookPlanInputs {
        context: &context(fixture),
        cwd: repository_root.as_path(),
        global_arguments: &[],
        common_directory: common.as_path(),
        worktree_root: repository_root.as_path(),
        lease: lease.as_str(),
        concurrent_commits,
        inherited_lease_valid: false,
        environment: &[],
        snapshot: None,
    })
    .expect("plan");
    let hooks: PathBuf = directory.join("hooks");
    write_hook_dispatch(hooks.as_path(), &plan, fixture.wrapper.as_path()).expect("entries");
    return Prepared {
        repository: repository_root,
        hooks,
        plan,
    };
}

/// A shared hooks directory with the given `pre-commit` script.
fn shared_hooks(fixture: &Fixture, script: &[u8]) -> PathBuf {
    let shared: PathBuf = fixture.root.join("shared-hooks");
    std::fs::create_dir_all(&shared).expect("shared hooks");
    executable(shared.join("pre-commit").as_path(), script);
    return shared;
}

/// Run one entry by its path, as Git does, and collect its output.
fn run_entry(fixture: &Fixture, prepared: &Prepared, event: &str, directory: &Path, tag: &str) -> Output {
    let mut command: Command = hermetic_command(
        fixture,
        prepared.hooks.join(event).as_path(),
        directory,
    );
    command.env("TAG", tag);
    return command.output().expect("entry");
}

/// Start one entry in the background.
fn spawn_entry(fixture: &Fixture, prepared: &Prepared, tag: &str) -> Child {
    let mut command: Command = hermetic_command(
        fixture,
        prepared.hooks.join("pre-commit").as_path(),
        prepared.repository.as_path(),
    );
    command.env("TAG", tag).stdin(Stdio::null());
    return command.spawn().expect("spawn entry");
}

/// The hook script's shared log, one line per event.
fn log_lines(path: &Path) -> Vec<String> {
    return std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect();
}

/// A hookdir hook and a `hook.<name>.command` hook each run once, through the dispatcher.
#[test]
fn a_hookdir_hook_and_a_command_hook_each_run_once() {
    let fixture: Fixture = fixture("hook-once");
    let log: PathBuf = fixture.root.join("log");
    let shared: PathBuf = shared_hooks(
        &fixture,
        format!("#!/bin/sh\necho hookdir >> {}\n", log.display()).as_bytes(),
    );
    let prepared_state: Prepared = prepared(&fixture, "repo", shared.as_path(), false);
    let mut command: Command = hermetic_command(
        &fixture,
        Path::new(REAL_GIT),
        prepared_state.repository.as_path(),
    );
    // A `hook.<name>.command` hook names its event with `hook.<name>.event`, per git-hook(1).
    command
        .args(["config", "hook.logcmd.command"])
        .arg(format!("echo command >> {}", log.display()));
    assert!(command.status().expect("config").success());
    let mut event: Command = hermetic_command(
        &fixture,
        Path::new(REAL_GIT),
        prepared_state.repository.as_path(),
    );
    event.args(["config", "hook.logcmd.event", "pre-commit"]);
    assert!(event.status().expect("event config").success());
    let outcome: Output = run_entry(&fixture, &prepared_state, "pre-commit", prepared_state.repository.as_path(), "a");
    assert_eq!(outcome.status.code(), Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
    let lines: Vec<String> = log_lines(log.as_path());
    assert_eq!(lines.iter().filter(|line| { return line.as_str() == "hookdir"; }).count(), 1, "{lines:?}");
    assert_eq!(lines.iter().filter(|line| { return line.as_str() == "command"; }).count(), 1, "{lines:?}");
    assert!(!prepared_state.repository.join(".git/cli-git/hook.lock").exists());
    remove_fixture(fixture);
}

/// A hook in a subdirectory sees the worktree top level and an absolute `GIT_WORK_TREE`.
#[test]
fn a_hook_in_a_subdirectory_sees_the_top_level_and_an_absolute_work_tree() {
    let fixture: Fixture = fixture("hook-subdirectory");
    let log: PathBuf = fixture.root.join("log");
    let shared: PathBuf = shared_hooks(
        &fixture,
        format!(
            "#!/bin/sh\necho \"top=$(git rev-parse --show-toplevel)\" >> {log}\necho \"work=$GIT_WORK_TREE\" >> {log}\n",
            log = log.display()
        )
        .as_bytes(),
    );
    let prepared_state: Prepared = prepared(&fixture, "repo", shared.as_path(), false);
    let subdirectory: PathBuf = prepared_state.repository.join("sub");
    std::fs::create_dir(&subdirectory).expect("subdirectory");
    let outcome: Output = run_entry(&fixture, &prepared_state, "pre-commit", subdirectory.as_path(), "a");
    assert_eq!(outcome.status.code(), Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
    let lines: Vec<String> = log_lines(log.as_path());
    let top: String = format!("top={}", prepared_state.repository.display());
    assert!(lines.contains(&top), "{lines:?} does not contain {top}");
    let work: String = format!("work={}", prepared_state.repository.display());
    assert!(lines.contains(&work), "{lines:?} does not contain {work}");
    remove_fixture(fixture);
}

/// A failing hook's status is the entry's status.
#[test]
fn a_failing_hook_propagates_its_status() {
    let fixture: Fixture = fixture("hook-status");
    let shared: PathBuf = shared_hooks(&fixture, b"#!/bin/sh\nexit 3\n");
    let prepared_state: Prepared = prepared(&fixture, "repo", shared.as_path(), false);
    let outcome: Output = run_entry(&fixture, &prepared_state, "pre-commit", prepared_state.repository.as_path(), "a");
    assert_eq!(outcome.status.code(), Some(3));
    assert!(!prepared_state.repository.join(".git/cli-git/hook.lock").exists());
    remove_fixture(fixture);
}

/// Two dispatches serialize on the hook lock: the second hook starts only after the first one ends.
#[test]
fn two_dispatches_serialize_by_default() {
    let fixture: Fixture = fixture("hook-serialize");
    let log: PathBuf = fixture.root.join("log");
    let release: PathBuf = fixture.root.join("release");
    let shared: PathBuf = shared_hooks(&fixture, barrier_script(&log, &release).as_bytes());
    let prepared_state: Prepared = prepared(&fixture, "repo", shared.as_path(), false);
    assert!(!prepared_state.plan.skip_hook_lock);
    let mut first: Child = spawn_entry(&fixture, &prepared_state, "a");
    wait_for_line(&log, "start a");
    let mut second: Child = spawn_entry(&fixture, &prepared_state, "b");
    std::thread::sleep(Duration::from_millis(500));
    assert!(!log_lines(log.as_path()).contains(&String::from("start b")), "second hook started early");
    std::fs::write(&release, b"").expect("release");
    assert!(first.wait().expect("first").success());
    assert!(second.wait().expect("second").success());
    assert_eq!(log_lines(log.as_path()), vec!["start a", "end a", "start b", "end b"]);
    remove_fixture(fixture);
}

/// With `hooks.concurrentCommits` both dispatches run their hooks at once.
#[test]
fn two_dispatches_overlap_with_concurrent_commits() {
    let fixture: Fixture = fixture("hook-overlap");
    let log: PathBuf = fixture.root.join("log");
    let release: PathBuf = fixture.root.join("release");
    let shared: PathBuf = shared_hooks(&fixture, barrier_script(&log, &release).as_bytes());
    let prepared_state: Prepared = prepared(&fixture, "repo", shared.as_path(), true);
    assert!(prepared_state.plan.skip_hook_lock);
    let mut first: Child = spawn_entry(&fixture, &prepared_state, "a");
    wait_for_line(&log, "start a");
    let mut second: Child = spawn_entry(&fixture, &prepared_state, "b");
    wait_for_line(&log, "start b");
    std::fs::write(&release, b"").expect("release");
    assert!(first.wait().expect("first").success());
    assert!(second.wait().expect("second").success());
    remove_fixture(fixture);
}

/// Start names that are not a preparation entry of a transaction behave as `git`.
#[test]
fn other_start_names_behave_as_git() {
    let fixture: Fixture = fixture("hook-other-names");
    let shared: PathBuf = shared_hooks(&fixture, b"#!/bin/sh\nexit 9\n");
    let prepared_state: Prepared = prepared(&fixture, "repo", shared.as_path(), false);
    let outside: PathBuf = fixture.root.join("outside");
    std::fs::create_dir(&outside).expect("outside");
    let plain: PathBuf = outside.join("pre-commit");
    std::os::unix::fs::symlink(&fixture.wrapper, &plain).expect("plain link");
    std::fs::copy(
        prepared_state.hooks.join("plan.json"),
        outside.join("plan.json"),
    )
    .expect("copy plan");
    let outside_hooks: PathBuf = fixture.root.join("elsewhere").join("hooks");
    std::fs::create_dir_all(&outside_hooks).expect("elsewhere");
    std::fs::copy(prepared_state.hooks.join("plan.json"), outside_hooks.join("plan.json")).expect("copy");
    std::os::unix::fs::symlink(&fixture.wrapper, outside_hooks.join("pre-commit")).expect("link");
    std::os::unix::fs::symlink(&fixture.wrapper, prepared_state.hooks.join("post-commit")).expect("post");
    std::os::unix::fs::symlink(&fixture.wrapper, prepared_state.hooks.join("pre-push")).expect("push");
    for start in [
        plain,
        outside_hooks.join("pre-commit"),
        prepared_state.hooks.join("post-commit"),
        prepared_state.hooks.join("pre-push"),
    ] {
        let mut command: Command = bounded(&fixture, start.as_path(), wrapper_first_path(&fixture).as_os_str());
        command.arg("--version").current_dir(&prepared_state.repository);
        let outcome = observe(&mut command, b"");
        assert_eq!(outcome.code, Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
        assert!(String::from_utf8_lossy(&outcome.stdout).starts_with("git version "), "{start:?}");
    }
    remove_fixture(fixture);
}

/// A hook script that records its start, waits for the release marker, and records its end.
fn barrier_script(log: &Path, release: &Path) -> String {
    return format!(
        "#!/bin/sh\necho \"start $TAG\" >> {log}\ni=0\nwhile [ ! -e {release} ] && [ $i -lt 1000 ]; do sleep 0.02; i=$((i+1)); done\necho \"end $TAG\" >> {log}\n",
        log = log.display(),
        release = release.display()
    );
}

/// Wait until the log names the line.
fn wait_for_line(log: &Path, line: &str) {
    let started: Instant = Instant::now();
    while !log_lines(log).iter().any(|entry| { return entry == line; }) {
        assert!(started.elapsed() < BARRIER_LIMIT, "timed out waiting for {line}");
        std::thread::sleep(POLL);
    }
}

/// Remove the fixture after the control.
fn remove_fixture(fixture: Fixture) {
    remove(&fixture);
}
