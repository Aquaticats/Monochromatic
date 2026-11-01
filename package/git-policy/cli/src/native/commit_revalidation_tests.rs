//! Controls for the `pre-commit` re-run of a replayed tree.

use super::*;
use crate::test_support::{REAL_GIT, executable, fixture, git, remove, repository};

/// A context that runs the host's Git.
fn context() -> GitContext {
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: vec![
            (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
            (OsString::from("GIT_AUTHOR_NAME"), OsString::from("Fixture")),
            (OsString::from("GIT_AUTHOR_EMAIL"), OsString::from("fixture@example.invalid")),
        ],
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// The hook plan the re-run reads: hooks possible, no event disabled, the lock under `root`.
fn plan(root: &Path) -> HookDispatchPlan {
    return HookDispatchPlan {
        disabled_events: Vec::new(),
        lease: String::from("a-lease"),
        hook_lock_path: root.join("hook.lock").display().to_string(),
        skip_hook_lock: false,
        named_hooks_possible: true,
    };
}

/// A repository whose configuration defines a `pre-commit` hook that runs this script.
fn with_hook(name: &str, script: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let path: PathBuf = root.join("hook-script");
    executable(path.as_path(), script.as_bytes());
    git(repo.as_path(), &["config", "hook.under-test.event", "pre-commit"]);
    git(repo.as_path(), &["config", "hook.under-test.command", path.to_str().expect("UTF-8")]);
    return (root, repo);
}

/// A hook that succeeds lets the replay go on, and sees the replayed index, no editor, the work tree, the lease and
///  the held hook lock.
#[test]
fn a_passing_hook_sees_the_replayed_index_no_editor_the_lease_and_the_lock() {
    let root_for_lock: PathBuf = fixture("revalidation-pass-lock");
    let lock: PathBuf = root_for_lock.join("hook.lock");
    let script: String = format!(
        "#!/bin/sh\nprintf '%s|%s|%s|%s|' \"$GIT_INDEX_FILE\" \"$GIT_EDITOR\" \"$(pwd -P)\" \"$CLI_GIT_PREPARATION_LEASE\" > hook-saw\n\
         if [ -d '{}' ]; then printf 'locked' >> hook-saw; else printf 'unlocked' >> hook-saw; fi\n",
        lock.display()
    );
    let (root, repo) = with_hook("revalidation-pass", script.as_str());
    let index: PathBuf = repo.join(".git/replayed.index");
    rerun_pre_commit(&context(), repo.as_path(), &[], repo.join(".git").as_path(), &plan(root_for_lock.as_path()), index.as_path())
        .expect("the hook passes");
    let saw: String = std::fs::read_to_string(repo.join("hook-saw")).expect("hook ran");
    assert_eq!(saw, format!("{}|:|{}|a-lease|locked", index.display(), repo.display()));
    assert!(!lock.exists(), "the lock is released when the re-run ends");
    remove(root.as_path());
    remove(root_for_lock.as_path());
}

/// A hook that fails rejects the replayed tree with Git's own status.
#[test]
fn a_failing_hook_rejects_the_replay_with_status_one() {
    let (root, repo) = with_hook("revalidation-fail", "#!/bin/sh\nexit 3\n");
    let outcome = rerun_pre_commit(
        &context(),
        repo.as_path(),
        &[],
        repo.join(".git").as_path(),
        &plan(root.as_path()),
        repo.join(".git/index").as_path(),
    );
    assert!(matches!(outcome, Err(TransactionStop::NativeExit(1))), "{outcome:?}");
    assert!(!root.join("hook.lock").exists(), "a rejected re-run still releases the lock");
    remove(root.as_path());
}

/// A replay directory that already exists is accepted; any other failure is reported.
#[test]
fn an_existing_replay_directory_is_accepted_and_another_failure_is_not() {
    let root: PathBuf = fixture("revalidation-directory");
    let directory: PathBuf = root.join("replay-1");
    ensure_replay_directory(directory.as_path()).expect("created");
    assert!(directory.is_dir());
    ensure_replay_directory(directory.as_path()).expect("already there");
    assert!(ensure_replay_directory(root.join("missing/replay-2").as_path()).is_err());
    remove(root.as_path());
}

/// Only a detached `HEAD` is detached.
#[test]
fn only_a_detached_head_is_detached() {
    assert!(is_detached(&SymbolicHead::Detached));
    assert!(!is_detached(&SymbolicHead::Branch(String::from("refs/heads/main"))));
}

/// A configuration that defines a hook for another event only finds no `pre-commit` hook, which `--ignore-missing`
///  accepts.
#[test]
fn a_missing_hook_passes() {
    let root: PathBuf = fixture("revalidation-missing");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.other.event", "commit-msg"]);
    git(repo.as_path(), &["config", "hook.other.command", "true"]);
    rerun_pre_commit(
        &context(),
        repo.as_path(),
        &[],
        repo.join(".git").as_path(),
        &plan(root.as_path()),
        repo.join(".git/index").as_path(),
    )
    .expect("no hook, no rejection");
    remove(root.as_path());
}

/// A configuration with no hook defined by name starts nothing, even where a hook would reject the replay.
#[test]
fn without_named_hooks_the_rerun_starts_nothing() {
    let (root, repo) = with_hook("revalidation-unnamed", "#!/bin/sh\ntouch hook-saw\nexit 3\n");
    let unnamed: HookDispatchPlan = HookDispatchPlan { named_hooks_possible: false, ..plan(root.as_path()) };
    rerun_pre_commit(
        &context(),
        repo.as_path(),
        &[],
        repo.join(".git").as_path(),
        &unnamed,
        repo.join(".git/index").as_path(),
    )
    .expect("nothing starts, so nothing rejects");
    assert!(!repo.join("hook-saw").exists(), "no process ran the hook");
    remove(root.as_path());
}

/// A `pre-commit` the user switched off with `hook.pre-commit.enabled=false` starts nothing and takes no lock.
#[test]
fn a_disabled_pre_commit_starts_nothing_and_takes_no_lock() {
    let (root, repo) = with_hook("revalidation-disabled", "#!/bin/sh\ntouch hook-saw\nexit 3\n");
    let disabled: HookDispatchPlan =
        HookDispatchPlan { disabled_events: vec![String::from("pre-commit")], ..plan(root.as_path()) };
    rerun_pre_commit(
        &context(),
        repo.as_path(),
        &[],
        repo.join(".git").as_path(),
        &disabled,
        repo.join(".git/index").as_path(),
    )
    .expect("a disabled event rejects nothing");
    assert!(!repo.join("hook-saw").exists());
    assert!(!root.join("hook.lock").exists());
    remove(root.as_path());
}

/// A plan that skips the lock (concurrent commits or a nested commit inside a hook) holds none during the re-run.
#[test]
fn a_plan_that_skips_the_lock_holds_none_during_the_rerun() {
    let root_for_lock: PathBuf = fixture("revalidation-skip-lock");
    let lock: PathBuf = root_for_lock.join("hook.lock");
    let script: String = format!(
        "#!/bin/sh\nif [ -d '{}' ]; then printf locked > hook-saw; else printf unlocked > hook-saw; fi\n",
        lock.display()
    );
    let (root, repo) = with_hook("revalidation-skip", script.as_str());
    let skipping: HookDispatchPlan = HookDispatchPlan { skip_hook_lock: true, ..plan(root_for_lock.as_path()) };
    rerun_pre_commit(&context(), repo.as_path(), &[], repo.join(".git").as_path(), &skipping, repo.join(".git/index").as_path())
        .expect("the hook passes");
    assert_eq!(std::fs::read_to_string(repo.join("hook-saw")).expect("hook ran"), "unlocked");
    remove(root.as_path());
    remove(root_for_lock.as_path());
}

/// The caller's global options reach Git ahead of the hook command.
#[test]
fn the_callers_global_options_reach_the_hook() {
    let (root, repo) = with_hook(
        "revalidation-config",
        "#!/bin/sh\ngit config --get fixture.marker > hook-config\n",
    );
    rerun_pre_commit(
        &context(),
        repo.as_path(),
        &[String::from("-c"), String::from("fixture.marker=present")],
        repo.join(".git").as_path(),
        &plan(root.as_path()),
        repo.join(".git/index").as_path(),
    )
    .expect("the hook passes");
    assert_eq!(std::fs::read_to_string(repo.join("hook-config")).expect("hook ran").trim(), "present");
    remove(root.as_path());
}
