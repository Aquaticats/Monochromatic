//! Controls for post-commit auto-push against real repositories, bare remotes and hooks.
//! The TypeScript corpus is `src/auto-push.unit.test.ts`; each case below reproduces one of its cases.

use super::*;
use crate::test_support::{fixture, git, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Marker the bare remote's post-receive hook prints, which Git relays as `remote: <marker>`.
const REMOTE_HOOK_MARKER: &str = "marker-from-receive-hook";

/// Environment of every Git child in these fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
    // The per-user scratch directory of the child runners is made under a home beside the fixtures.
    environment.extend(crate::test_support::scratch_environment());
    return environment;
}

/// Coordinator Git for a repository at `root`.
fn push_git(root: &Path) -> PushGit {
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment: fixture_environment(),
        lfs_pre_push: None,
    };
}

/// The OID `HEAD` names in a fixture.
fn head_oid(root: &Path) -> String {
    return String::from_utf8_lossy(&git(root, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}

/// Create a repository with one commit that changes `tracked.txt`.
fn init_repo_with_commit(path: &Path) {
    std::fs::create_dir_all(path).expect("repo directory");
    git(path, &["init", "--quiet"]);
    git(path, &["config", "user.email", "cli-git@example.invalid"]);
    git(path, &["config", "user.name", "cli-git auto-push test"]);
    std::fs::write(path.join("tracked.txt"), "initial\n").expect("tracked file");
    git(path, &["add", "tracked.txt"]);
    git(path, &["commit", "--quiet", "-m", "initial", "tracked.txt"]);
}

/// Change `tracked.txt` and commit the change.
fn commit_change(path: &Path, text: &str) {
    std::fs::write(path.join("tracked.txt"), text).expect("tracked file");
    git(
        path,
        &["commit", "--quiet", "-m", text.trim(), "tracked.txt"],
    );
}

/// Create a bare remote whose post-receive hook prints the marker.
fn bare_remote_with_hook(remote: &Path) {
    git(
        remote.parent().expect("parent"),
        &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")],
    );
    let hook = remote.join("hooks").join("post-receive");
    std::fs::write(&hook, format!("#!/bin/sh\necho \"{REMOTE_HOOK_MARKER}\"\n")).expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
}

/// Create a repository whose origin is a fresh bare remote and whose pre-push hook logs and exits with `code`.
fn repo_with_remote_and_pre_push(root: &Path, code: i32) -> (PathBuf, PathBuf) {
    let repo = root.join("repo");
    let remote = root.join("remote.git");
    let hooks = root.join("hooks");
    let log = root.join("pre-push.log");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    std::fs::create_dir_all(&hooks).expect("hooks");
    let hook = hooks.join("pre-push");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\necho pre-push >> '{}'\nexit {code}\n",
            log.display()
        ),
    )
    .expect("pre-push hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    // A configuration hook: traditional hooks never run through the wrapper's real-Git starts.
    git(&repo, &["config", "hook.control.event", "pre-push"]);
    git(&repo, &["config", "hook.control.command", hook.to_str().expect("utf-8")]);
    return (repo, log);
}

/// The pre-push log text, empty when no hook ran.
fn read_log(log: &Path) -> String {
    return std::fs::read_to_string(log).unwrap_or_default();
}

/// Run auto-push and capture the terminal text it prints to `err`.
fn run(repo: &Path, landed: &str) -> (Result<AutoPushResult, SingleFlightError>, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = auto_push(&push_git(repo), landed, &mut err);
    return (result, String::from_utf8_lossy(&err).into_owned());
}

/// Filtering a clean push keeps only the `remote:` lines.
#[test]
fn keeps_only_remote_lines_on_a_clean_push() {
    assert_eq!(
        filter_push_output("remote: hello\nTo origin\nremote: world\nbranch set up", 0),
        "remote: hello\nremote: world"
    );
}

/// A clean push without `remote:` lines surfaces nothing.
#[test]
fn a_clean_push_without_remote_lines_surfaces_nothing() {
    assert_eq!(
        filter_push_output("To origin\nbranch main set up to track origin/main", 0),
        ""
    );
}

/// The lines a reconciled push adds stay visible beside the `remote:` lines of a clean push.
#[test]
fn a_clean_push_keeps_the_reconcile_lines() {
    assert_eq!(
        filter_push_output(
            "To origin\nremote: hello\ncli-git: auto-push: published a merge\nbranch set up",
            0
        ),
        "remote: hello\ncli-git: auto-push: published a merge"
    );
}

/// A failed push surfaces its complete output.
#[test]
fn a_failed_push_surfaces_the_full_output() {
    let output = "remote: rejected\nTo origin\n! [remote rejected] HEAD -> main";
    assert_eq!(filter_push_output(output, 1), output);
}

/// A repository with neither upstream nor origin is skipped silently.
#[test]
fn skips_when_no_origin_remote_exists() {
    let root = fixture("ap-no-origin");
    init_repo_with_commit(&root);
    let (attempted, err) = run(&root, &head_oid(&root));
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Skipped);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.shown, "");
    assert_eq!(err, "");
    remove(&root);
}

/// A first push creates the branch on origin, prints only the `remote:` lines and records the upstream.
#[test]
fn pushes_and_surfaces_only_remote_lines_on_success() {
    let root = fixture("ap-success");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    let landed = head_oid(&repo);
    let (attempted, err) = run(&repo, &landed);
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed);
    assert_eq!(result.exit_code, 0);
    assert!(
        result.shown.contains(REMOTE_HOOK_MARKER),
        "{}",
        result.shown
    );
    assert!(
        result
            .shown
            .split('\n')
            .filter(|line| {
                return !line.is_empty();
            })
            .all(|line| {
                return line.starts_with("remote: ");
            }),
        "{}",
        result.shown
    );
    assert_eq!(err, format!("{}\n", result.shown));
    remove(&root);
}

/// A detached HEAD is skipped with the note on stderr and in the result.
#[test]
fn skips_with_a_note_when_head_is_detached() {
    let root = fixture("ap-detached");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    git(&repo, &["checkout", "--quiet", "--detach", "HEAD"]);
    let (attempted, err) = run(&repo, &head_oid(&repo));
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Skipped);
    assert_eq!(result.exit_code, 0);
    assert!(
        result.shown.contains("HEAD is detached"),
        "{}",
        result.shown
    );
    assert!(err.contains("HEAD is detached"), "{err}");
    remove(&root);
}

/// An existing upstream on another remote is pushed to, and origin is never created or used.
#[test]
fn pushes_to_an_existing_upstream_without_re_pointing_it_to_origin() {
    let root = fixture("ap-upstream");
    let repo = root.join("repo");
    let alt = root.join("alt.git");
    bare_remote_with_hook(&alt);
    init_repo_with_commit(&repo);
    git(
        &repo,
        &["remote", "add", "alt", alt.to_str().expect("utf-8")],
    );
    git(&repo, &["push", "--quiet", "--set-upstream", "alt", "HEAD"]);
    commit_change(&repo, "second\n");
    let landed = head_oid(&repo);
    let (attempted, _err) = run(&repo, &landed);
    assert_eq!(
        attempted.expect("auto-push").outcome,
        AutoPushOutcome::Pushed
    );
    let branch = String::from_utf8_lossy(&git(&repo, &["branch", "--show-current"]).stdout)
        .trim()
        .to_owned();
    let tracked = String::from_utf8_lossy(
        &git(&repo, &["config", &format!("branch.{branch}.remote")]).stdout,
    )
    .trim()
    .to_owned();
    assert_eq!(tracked, "alt");
    assert_eq!(
        String::from_utf8_lossy(&git(&alt, &["rev-parse", &branch]).stdout).trim(),
        landed
    );
    remove(&root);
}

/// A rejected push reports failure with its output, exits non-zero and keeps the commit.
#[test]
fn reports_failure_and_surfaces_full_output_when_the_push_is_rejected() {
    let root = fixture("ap-rejected");
    let repo = root.join("repo");
    init_repo_with_commit(&repo);
    git(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            root.join("missing.git").to_str().expect("utf-8"),
        ],
    );
    let (attempted, err) = run(&repo, &head_oid(&repo));
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Failed);
    assert_ne!(result.exit_code, 0);
    assert!(!result.shown.is_empty());
    assert!(err.contains(PUSH_FAILED_NOTE), "{err}");
    remove(&root);
}

/// Backing up the same commit again joins the recorded push and does not run pre-push.
#[test]
fn joins_the_recorded_push_when_the_same_commit_is_backed_up_again() {
    let root = fixture("ap-join");
    let (repo, log) = repo_with_remote_and_pre_push(&root, 0);
    let landed = head_oid(&repo);
    let first = run(&repo, &landed).0.expect("first");
    let second = run(&repo, &landed).0.expect("second");
    assert_eq!(
        (first.outcome, first.joined),
        (AutoPushOutcome::Pushed, false)
    );
    assert_eq!(
        second,
        AutoPushResult {
            outcome: AutoPushOutcome::Pushed,
            exit_code: 0,
            shown: String::new(),
            joined: true,
        }
    );
    assert_eq!(read_log(&log), "pre-push\n");
    remove(&root);
}

/// A failed pre-push hook is a failed push whose full output is surfaced.
#[test]
fn surfaces_a_failed_pre_push_hook_as_a_failed_push_with_its_full_output() {
    let root = fixture("ap-prepush-fails");
    let (repo, log) = repo_with_remote_and_pre_push(&root, 1);
    let (attempted, _err) = run(&repo, &head_oid(&repo));
    let result = attempted.expect("auto-push");
    assert_eq!(
        (result.outcome, result.joined),
        (AutoPushOutcome::Failed, false)
    );
    assert_ne!(result.exit_code, 0);
    assert!(result.shown.contains("failed to push"), "{}", result.shown);
    assert_eq!(read_log(&log), "pre-push\n");
    remove(&root);
}

/// A coordination failure is a failed push that returns instead of erroring, and no push runs.
#[test]
fn surfaces_a_push_coordination_failure_as_a_failed_push_instead_of_erroring() {
    let root = fixture("ap-coordination");
    let (repo, log) = repo_with_remote_and_pre_push(&root, 0);
    std::fs::create_dir_all(repo.join(".git").join("cli-git")).expect("cli-git");
    std::fs::write(repo.join(".git").join("cli-git").join("push"), "").expect("blocking file");
    let (attempted, err) = run(&repo, &head_oid(&repo));
    let result = attempted.expect("returns the failure");
    assert_eq!(result.outcome, AutoPushOutcome::Failed);
    assert_eq!(result.exit_code, 1);
    assert!(!result.shown.is_empty());
    assert!(err.contains("could not run"), "{err}");
    assert_eq!(read_log(&log), "");
    remove(&root);
}

/// A push that stalls past its deadline is stopped and reported as a failed push,
///  and the push lock is free for the next commit.
#[test]
fn a_stalled_push_is_stopped_and_reported_as_a_failed_push() {
    let root = fixture("ap-stalled");
    let (repo, log) = repo_with_remote_and_pre_push(&root, 0);
    let hooks = root.join("hooks");
    let hook = hooks.join("pre-push");
    std::fs::write(&hook, "#!/bin/sh\necho stalling\nexec sleep 8\n").expect("stalling hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let mut git = push_git(&repo);
    git.environment.push((
        OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE),
        OsString::from("1"),
    ));
    let mut err: Vec<u8> = Vec::new();
    let started = std::time::Instant::now();
    let result = auto_push(&git, &head_oid(&repo), &mut err).expect("auto-push");
    let elapsed = started.elapsed();
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert!(elapsed >= Duration::from_secs(1), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(30), "{elapsed:?}");
    assert_eq!(result.outcome, AutoPushOutcome::Failed);
    assert_eq!(result.exit_code, crate::bounded_push::STOPPED_EXIT_CODE);
    assert!(
        result.shown.contains("did not finish within 1 seconds"),
        "{}",
        result.shown
    );
    assert!(shown.contains(PUSH_FAILED_NOTE), "{shown}");
    assert_eq!(read_log(&log), "", "the replaced hook logs nothing");
    // The failure is recorded and the lock released, so a later commit coordinates normally.
    let lock_directory = std::fs::read_dir(repo.join(".git").join("cli-git").join("push"))
        .expect("coordination directory")
        .map(|entry| return entry.expect("entry").file_name())
        .collect::<Vec<OsString>>();
    assert!(
        !lock_directory
            .iter()
            .any(|name| return name.to_string_lossy().ends_with(".lock")),
        "{lock_directory:?}"
    );
    remove(&root);
}

/// The default environment of an auto-push gives Git's low-speed abort to the push it runs.
#[test]
fn a_push_runs_with_git_s_low_speed_abort_by_default() {
    let root = fixture("ap-low-speed");
    let (repo, _log) = repo_with_remote_and_pre_push(&root, 0);
    let hook = root.join("hooks").join("pre-push");
    let seen = root.join("seen.txt");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\necho \"$GIT_HTTP_LOW_SPEED_LIMIT $GIT_HTTP_LOW_SPEED_TIME\" > '{}'\n",
            seen.display()
        ),
    )
    .expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let (attempted, _err) = run(&repo, &head_oid(&repo));
    assert_eq!(attempted.expect("auto-push").outcome, AutoPushOutcome::Pushed);
    assert_eq!(read_log(&seen), "1000 60\n");
    remove(&root);
}

/// A person's own low-speed variables reach the push unchanged,
///  and the other default still applies.
#[test]
fn a_push_keeps_the_low_speed_variables_the_person_set() {
    let root = fixture("ap-low-speed-set");
    let (repo, _log) = repo_with_remote_and_pre_push(&root, 0);
    let hook = root.join("hooks").join("pre-push");
    let seen = root.join("seen.txt");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\necho \"$GIT_HTTP_LOW_SPEED_LIMIT $GIT_HTTP_LOW_SPEED_TIME\" > '{}'\n",
            seen.display()
        ),
    )
    .expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let mut git = push_git(&repo);
    git.environment.push((
        OsString::from("GIT_HTTP_LOW_SPEED_LIMIT"),
        OsString::from("5"),
    ));
    let mut err: Vec<u8> = Vec::new();
    let result = auto_push(&git, &head_oid(&repo), &mut err).expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed);
    assert_eq!(read_log(&seen), "5 60\n");
    remove(&root);
}

/// A commit that waits behind a live push says so on the terminal,
///  naming the pushing process, the branch and the limit.
#[test]
fn a_waiting_commit_names_the_process_it_waits_for() {
    let root = fixture("ap-waiting-notice");
    let (repo, _log) = repo_with_remote_and_pre_push(&root, 0);
    let branch_output = git(&repo, &["symbolic-ref", "HEAD"]);
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_owned();
    let mut git = push_git(&repo);
    git.environment.push((
        OsString::from(crate::bounded_push::LOCK_WAIT_VARIABLE),
        OsString::from("1"),
    ));
    let paths = crate::auto_push_single_flight::resolve_push_coordination_paths(&git, &branch)
        .expect("paths");
    let held = crate::owner_lock::acquire_owner_lock(
        &paths.lock_directory,
        crate::owner_lock::DEFAULT_POLL_DELAY,
        &mut crate::owner_lock::SilentWait,
    )
    .expect("held");
    let mut err: Vec<u8> = Vec::new();
    let result = auto_push(&git, &head_oid(&repo), &mut err).expect("auto-push");
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert!(
        shown.contains(&format!(
            "cli-git: waiting for process {} to finish pushing {branch}; giving up after 1 seconds \
             (CLI_GIT_PUSH_LOCK_WAIT_SECONDS changes this). The commit is already saved.",
            std::process::id()
        )),
        "{shown}"
    );
    assert_eq!(result.outcome, AutoPushOutcome::Failed);
    assert!(shown.contains("gave up after waiting 1 seconds"), "{shown}");
    assert!(shown.contains(PUSH_FAILED_NOTE), "{shown}");
    drop(held);
    remove(&root);
}

/// A commit that keeps waiting behind a live push repeats its notice at the interval,
///  each repeat naming the process, the branch, how long the wait has lasted and the limit.
///  The third notice makes the pusher let go, so the wait ends on that event and exactly three notices are out,
///  however slowly the host runs the waiting commit.
#[test]
fn a_commit_waiting_behind_a_long_push_repeats_its_notice() {
    let root = fixture("ap-waiting-repeat");
    let (repo, _log) = repo_with_remote_and_pre_push(&root, 0);
    let branch_output = git(&repo, &["symbolic-ref", "HEAD"]);
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_owned();
    let mut git = push_git(&repo);
    git.environment.push((
        OsString::from(crate::bounded_push::LOCK_WAIT_VARIABLE),
        OsString::from("120"),
    ));
    git.environment.push((
        OsString::from(crate::wait_progress::NOTICE_INTERVAL_VARIABLE),
        OsString::from("1"),
    ));
    let paths = crate::auto_push_single_flight::resolve_push_coordination_paths(&git, &branch)
        .expect("paths");
    let held = crate::owner_lock::acquire_owner_lock(
        &paths.lock_directory,
        crate::owner_lock::DEFAULT_POLL_DELAY,
        &mut crate::owner_lock::SilentWait,
    )
    .expect("held");
    let shared = crate::test_support::SharedSink::default();
    let mut err = crate::test_support::ActionSink::new(&shared, 3, move || { drop(held); });
    let _result = auto_push(&git, &head_oid(&repo), &mut err).expect("auto-push");
    let shown = shared.text();
    let holder = format!("waiting for process {} to finish pushing {branch}", std::process::id());
    let lines: Vec<&str> = shown.lines().filter(|line| { return line.contains(holder.as_str()); }).collect();
    // The wait ends on the third notice, but the lock is taken one attempt later, so a late tick may add a fourth.
    assert!(lines.len() >= 3, "the start and two repeats: {shown}");
    assert!(!lines[0].contains("seconds so far"), "{shown}");
    // The repeats name the time waited, which only grows; its exact value depends on the host's speed.
    let waited = |line: &str| -> u64 {
        let tail = line.split('(').nth(1).expect("the time waited");
        return tail.split(' ').next().expect("a number").parse().expect("whole seconds");
    };
    assert!(lines[1].contains(" seconds so far); giving up after 120 seconds"), "{shown}");
    assert!(lines[2].contains(" seconds so far); giving up after 120 seconds"), "{shown}");
    assert!(waited(lines[1]) >= 1 && waited(lines[2]) >= waited(lines[1]), "{shown}");
    remove(&root);
}

/// A first push, one with no upstream yet, is never reconciled with a merge, even when the push destination resolves.
#[test]
fn a_first_push_rejected_for_a_moved_remote_branch_publishes_no_merge() {
    let root = fixture("ap-first-push-stale");
    let repo = root.join("repo");
    let remote = root.join("remote.git");
    let other = root.join("other");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    let remote_text = remote.to_str().expect("utf-8");
    git(&repo, &["remote", "add", "origin", remote_text]);
    git(&repo, &["push", "--quiet", "origin", "HEAD"]);
    git(&root, &["clone", "--quiet", remote_text, "other"]);
    git(&other, &["config", "user.email", "cli-git@example.invalid"]);
    git(&other, &["config", "user.name", "cli-git auto-push test"]);
    commit_change(&other, "remote side\n");
    git(&other, &["push", "--quiet", "origin", "HEAD"]);
    let branch = String::from_utf8_lossy(&git(&repo, &["branch", "--show-current"]).stdout)
        .trim()
        .to_owned();
    let remote_tip = head_oid(&other);
    std::fs::write(repo.join("local.txt"), "local side\n").expect("local file");
    git(&repo, &["add", "local.txt"]);
    git(&repo, &["commit", "--quiet", "-m", "local side", "local.txt"]);
    git(&repo, &["config", "remote.pushDefault", "origin"]);
    git(&repo, &["config", "push.default", "current"]);
    let (attempted, err) = run(&repo, &head_oid(&repo));
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_eq!(
        String::from_utf8_lossy(&git(&remote, &["rev-parse", &branch]).stdout).trim(),
        remote_tip
    );
    remove(&root);
}

/// The name of the branch a fixture repository has checked out.
fn current_branch(repo: &Path) -> String {
    return String::from_utf8_lossy(&git(repo, &["branch", "--show-current"]).stdout)
        .trim()
        .to_owned();
}

/// A repository whose `feature` branch tracks the remote's default branch under another name, as a linked worktree
/// created from `origin/<default>` does; the default branch is returned with the repository and the remote.
fn repo_with_differently_named_upstream(root: &Path) -> (PathBuf, PathBuf, String) {
    let repo = root.join("repo");
    let remote = root.join("remote.git");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    let default = current_branch(&repo);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    git(
        &repo,
        &["checkout", "--quiet", "-b", "feature", "--track", &format!("origin/{default}")],
    );
    return (repo, remote, default);
}

/// A branch tracking a differently named remote branch is pushed to the tracked branch, not refused by Git.
#[test]
fn a_branch_with_a_differently_named_upstream_is_pushed_to_the_tracked_branch() {
    let root = fixture("ap-renamed-upstream");
    let (repo, remote, default) = repo_with_differently_named_upstream(&root);
    commit_change(&repo, "feature work\n");
    let landed = head_oid(&repo);
    let (attempted, err) = run(&repo, &landed);
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(
        String::from_utf8_lossy(&git(&remote, &["rev-parse", &default]).stdout).trim(),
        landed
    );
    let remote_refs = String::from_utf8_lossy(
        &git(&remote, &["for-each-ref", "--format=%(refname)"]).stdout,
    )
    .into_owned();
    assert!(
        !remote_refs.contains("refs/heads/feature"),
        "no branch named after the local one is created on the remote: {remote_refs}"
    );
    remove(&root);
}

/// A differently named upstream whose remote branch moved is reconciled with a merge on the tracked branch.
#[test]
fn a_differently_named_upstream_that_moved_is_reconciled_on_the_tracked_branch() {
    let root = fixture("ap-renamed-upstream-moved");
    let (repo, remote, default) = repo_with_differently_named_upstream(&root);
    let other = root.join("other");
    git(&root, &["clone", "--quiet", remote.to_str().expect("utf-8"), "other"]);
    git(&other, &["config", "user.email", "cli-git@example.invalid"]);
    git(&other, &["config", "user.name", "cli-git auto-push test"]);
    std::fs::write(other.join("remote.txt"), "remote side\n").expect("remote file");
    git(&other, &["add", "remote.txt"]);
    git(&other, &["commit", "--quiet", "-m", "remote side", "remote.txt"]);
    git(&other, &["push", "--quiet", "origin", "HEAD"]);
    let remote_tip = head_oid(&other);
    commit_change(&repo, "feature work\n");
    let landed = head_oid(&repo);
    let (attempted, err) = run(&repo, &landed);
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    let published = String::from_utf8_lossy(&git(&remote, &["rev-parse", &default]).stdout)
        .trim()
        .to_owned();
    // `merge-base --is-ancestor` exits non-zero for an unreachable commit, and `git` requires success.
    for ancestor in [&landed, &remote_tip] {
        git(&remote, &["merge-base", "--is-ancestor", ancestor, &published]);
    }
    remove(&root);
}
