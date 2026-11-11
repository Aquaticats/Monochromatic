//! Controls for post-commit auto-push against real repositories,
//! bare remotes and hooks.
//! The TypeScript corpus is `src/auto-push.unit.test.ts`;
//! each case below reproduces one of its cases.

use super::*;
use crate::test_support::{fixture, git, git_text, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Marker the bare remote's post-receive hook prints,
/// which Git relays as `remote: <marker>`.
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
        lfs: false,
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
    git(path, &["config", "user.name", "monochromatic-git-policy-cli auto-push test"]);
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

/// The pre-push log text,
/// empty when no hook ran.
fn read_log(log: &Path) -> String {
    return std::fs::read_to_string(log).unwrap_or_default();
}

/// The full ref name of the branch a fixture repository has checked out.
fn head_ref(repo: &Path) -> String {
    return String::from_utf8_lossy(&git(repo, &["symbolic-ref", "HEAD"]).stdout).trim().to_owned();
}

/// Push the checked-out branch the way the service does and capture the terminal text it prints to `err`.
fn run(repo: &Path, landed: &str) -> (Result<AutoPushResult, SingleFlightError>, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(repo), &head_ref(repo), landed, &[], &[], &mut err);
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
            "To origin\nremote: hello\nmonochromatic-git-policy-cli: auto-push: published a merge\nbranch set up",
            0
        ),
        "remote: hello\nmonochromatic-git-policy-cli: auto-push: published a merge"
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

/// A first push creates the branch on origin,
/// prints only the `remote:` lines and records the upstream.
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

/// An existing upstream on another remote is pushed to,
/// and origin is never created or used.
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

/// A rejected push reports failure with its output,
/// exits non-zero and keeps the commit.
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

/// A failed pre-push hook is a failed push whose full output is surfaced.
#[test]
fn surfaces_a_failed_pre_push_hook_as_a_failed_push_with_its_full_output() {
    let root = fixture("ap-prepush-fails");
    let (repo, log) = repo_with_remote_and_pre_push(&root, 1);
    let (attempted, _err) = run(&repo, &head_oid(&repo));
    let result = attempted.expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Failed);
    assert_ne!(result.exit_code, 0);
    assert!(result.shown.contains("failed to push"), "{}", result.shown);
    assert_eq!(read_log(&log), "pre-push\n");
    remove(&root);
}

/// A push that stalls past its deadline is stopped and reported as a failed push.
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
    let result = push_once(&git, &head_ref(&repo), &head_oid(&repo), &[], &[], &mut err).expect("auto-push");
    let elapsed = started.elapsed();
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert!(elapsed >= Duration::from_secs(1), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(30), "{elapsed:?}");
    assert_eq!(result.outcome, AutoPushOutcome::Failed);
    assert_eq!(result.exit_code, crate::bounded_push::STOPPED_EXIT_CODE);
    assert!(
        result.shown.contains("made no progress for 1 seconds"),
        "{}",
        result.shown
    );
    assert!(shown.contains(PUSH_FAILED_NOTE), "{shown}");
    assert!(shown.contains(crate::bounded_push::PUSH_QUEUED_LINE), "the command's output says the push is queued: {shown}");
    assert_eq!(read_log(&log), "", "the replaced hook logs nothing");
    remove(&root);
}

/// The default environment of an auto-push sets no low-speed abort for the push it runs:
/// a push waits while its connection is
///  open (owner,
/// 2026-11-03:
/// "Wait while connected (Recommended)"),
/// and Git sets neither variable itself (`http.c` reads them
///  only when the configuration or the environment gives them).
#[test]
fn a_push_runs_without_a_low_speed_abort_of_the_wrappers_own() {
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
    assert_eq!(read_log(&seen), " \n");
    remove(&root);
}

/// A person's own low-speed variables reach the push unchanged,
///  and the other one stays unset.
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
    let result = push_once(&git, &head_ref(&repo), &head_oid(&repo), &[], &[], &mut err).expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed);
    assert_eq!(read_log(&seen), "5 \n");
    remove(&root);
}

/// A first push,
/// one with no upstream yet,
/// is never reconciled with a merge,
/// even when the push destination resolves.
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
    git(&other, &["config", "user.name", "monochromatic-git-policy-cli auto-push test"]);
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

/// A repository whose `feature` branch tracks the remote's default branch under another name,
/// as a linked worktree
/// created from `origin/<default>` does;
/// the default branch is returned with the repository and the remote.
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

/// A branch tracking a differently named remote branch is pushed to the tracked branch,
/// not refused by Git.
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

/// The service pushes the branch its record names to the branch that branch tracks,
/// when the two names differ,
/// and does not
/// re-point the upstream to a branch of the same name:
/// the recorded branch is read as the branch it is,
/// not as an unknown name.
#[test]
fn a_named_branch_with_a_differently_named_upstream_is_pushed_to_the_tracked_branch() {
    let root = fixture("ap-named-renamed-upstream");
    let (repo, remote, default) = repo_with_differently_named_upstream(&root);
    commit_change(&repo, "feature work\n");
    let landed = head_oid(&repo);
    let (attempted, err) = run_for_branch(&repo, "refs/heads/feature", &landed);
    assert_eq!(attempted.expect("push").outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&remote, &default).as_deref(), Some(landed.as_str()), "the tracked branch received the commit: {err}");
    assert_eq!(remote_tip(&remote, "feature"), None, "no branch named after the local one is created on the remote");
    let tracked = String::from_utf8_lossy(&git(&repo, &["rev-parse", "--abbrev-ref", "feature@{upstream}"]).stdout).trim().to_owned();
    assert_eq!(tracked, format!("origin/{default}"), "the upstream is unchanged");
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
    git(&other, &["config", "user.name", "monochromatic-git-policy-cli auto-push test"]);
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

/// The service's push of a named branch whose tracked remote branch moved is reconciled with a merge,
/// as the wrapper's own is:
/// the recorded branch has an upstream,
/// so a rejection for a moved remote branch can be cured.
#[test]
fn a_named_branch_whose_tracked_remote_branch_moved_is_reconciled() {
    let root = fixture("ap-named-moved");
    let (repo, remote, default) = repo_with_differently_named_upstream(&root);
    let other = root.join("other");
    git(&root, &["clone", "--quiet", remote.to_str().expect("utf-8"), "other"]);
    git(&other, &["config", "user.email", "cli-git@example.invalid"]);
    git(&other, &["config", "user.name", "monochromatic-git-policy-cli auto-push test"]);
    std::fs::write(other.join("remote.txt"), "remote side\n").expect("remote file");
    git(&other, &["add", "remote.txt"]);
    git(&other, &["commit", "--quiet", "-m", "remote side", "remote.txt"]);
    git(&other, &["push", "--quiet", "origin", "HEAD"]);
    let remote_tip_before = head_oid(&other);
    commit_change(&repo, "feature work\n");
    let landed = head_oid(&repo);
    let (attempted, err) = run_for_branch(&repo, "refs/heads/feature", &landed);
    assert_eq!(attempted.expect("push").outcome, AutoPushOutcome::Pushed, "{err}");
    let published = remote_tip(&remote, &default).expect("the tracked branch exists");
    for ancestor in [&landed, &remote_tip_before] {
        git(&remote, &["merge-base", "--is-ancestor", ancestor, &published]);
    }
    remove(&root);
}

/// Run the service's push of a named branch and capture the text it prints.
fn run_for_branch(repo: &Path, branch_ref: &str, landed: &str) -> (Result<AutoPushResult, SingleFlightError>, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(repo), branch_ref, landed, &[], &[], &mut err);
    return (result, String::from_utf8_lossy(&err).into_owned());
}

/// What a bare remote holds under a branch name,
/// nothing when the branch is absent.
fn remote_tip(remote: &Path, branch: &str) -> Option<String> {
    let output = crate::test_support::git_output(remote, &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{branch}")]);
    return output.status.success().then(|| { return String::from_utf8_lossy(&output.stdout).trim().to_owned(); });
}

/// The service pushes the branch its record names,
/// not the branch the worktree has checked out by the time it runs:
/// a branch
/// that tracks its remote branch is pushed there,
/// and the checked-out branch's unpublished commit stays unpublished.
#[test]
fn a_push_of_a_named_branch_does_not_follow_the_checked_out_branch() {
    let root = fixture("ap-named-tracked");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let default = current_branch(&repo);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    git(&repo, &["checkout", "--quiet", "-b", "topic"]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "topic"]);
    commit_change(&repo, "topic work\n");
    let landed = head_oid(&repo);
    git(&repo, &["checkout", "--quiet", &default]);
    commit_change(&repo, "default work\n");
    let default_tip = head_oid(&repo);
    let (attempted, err) = run_for_branch(&repo, "refs/heads/topic", &landed);
    let result = attempted.expect("push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&remote, "topic").as_deref(), Some(landed.as_str()), "the named branch reached the remote");
    assert_ne!(remote_tip(&remote, &default).as_deref(), Some(default_tip.as_str()), "the checked-out branch was not pushed");
    assert_eq!(current_branch(&repo), default, "the worktree stayed where it was");
    remove(&root);
}

/// A named branch with no upstream is created on origin and tracks it,
/// whatever the worktree has checked out.
#[test]
fn a_first_push_of_a_named_branch_creates_it_on_origin() {
    let root = fixture("ap-named-first");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let default = current_branch(&repo);
    git(&repo, &["checkout", "--quiet", "-b", "topic"]);
    commit_change(&repo, "topic work\n");
    let landed = head_oid(&repo);
    git(&repo, &["checkout", "--quiet", &default]);
    let (attempted, err) = run_for_branch(&repo, "refs/heads/topic", &landed);
    assert_eq!(attempted.expect("push").outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&remote, "topic").as_deref(), Some(landed.as_str()));
    assert_eq!(remote_tip(&remote, &default), None, "the checked-out branch was not published");
    let tracked = String::from_utf8_lossy(&git(&repo, &["rev-parse", "--abbrev-ref", "topic@{upstream}"]).stdout).trim().to_owned();
    assert_eq!(tracked, "origin/topic");
    remove(&root);
}

/// A detached head does not stop the push of a branch the record names.
#[test]
fn a_named_branch_is_pushed_from_a_detached_worktree() {
    let root = fixture("ap-named-detached");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let default = current_branch(&repo);
    commit_change(&repo, "default work\n");
    let landed = head_oid(&repo);
    git(&repo, &["checkout", "--quiet", "--detach"]);
    let (attempted, err) = run_for_branch(&repo, &format!("refs/heads/{default}"), &landed);
    assert_eq!(attempted.expect("push").outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&remote, &default).as_deref(), Some(landed.as_str()));
    remove(&root);
}

/// Two services of two worktrees share the repository's references:
/// while one holds the lock of a remote-tracking reference
/// for a moment,
/// Git still delivers the push,
/// prints the refusal to update the reference and exits 0,
/// and leaves the reference
/// stale.
/// The push is tried again until the reference agrees with what was delivered.
#[test]
fn a_tracking_reference_locked_for_a_moment_ends_up_to_date() {
    let root = fixture("ap-tracking-locked");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let default = current_branch(&repo);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    commit_change(&repo, "work\n");
    let landed = head_oid(&repo);
    let lock = repo.join(format!(".git/refs/remotes/origin/{default}.lock"));
    std::fs::write(&lock, b"").expect("another worktree's service holds the lock");
    let releasing = lock.clone();
    let holder = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        std::fs::remove_file(releasing).expect("the other service lets go");
    });
    let (attempted, err) = run_for_branch(&repo, &format!("refs/heads/{default}"), &landed);
    holder.join().expect("holder");
    assert_eq!(attempted.expect("push").outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&remote, &default).as_deref(), Some(landed.as_str()));
    let tracking = String::from_utf8_lossy(&git(&repo, &["rev-parse", &format!("origin/{default}")]).stdout).trim().to_owned();
    assert_eq!(tracking, landed, "the remote-tracking reference agrees with the remote");
    remove(&root);
}

/// A Git that switches the worktree to `other` right after the first time the service starts it,
/// as a person
///  switching branches while the service is between its decision and its push.
fn switching_git(root: &Path, other: &str) -> PathBuf {
    let program = root.join("switching-git");
    let mark = root.join("switched");
    let text = format!(
        "#!/bin/sh\n/usr/bin/git \"$@\"\nstatus=$?\nif [ ! -e '{mark}' ]; then\n  : > '{mark}'\n  /usr/bin/git checkout --quiet {other}\nfi\nexit $status\n",
        mark = mark.display(),
    );
    std::fs::write(&program, text).expect("switching git");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("mode");
    return program;
}

/// The branch the record names is pushed even when the worktree switches to another branch between the service's check of
/// `HEAD` and its push:
/// the other branch is never published in its place.
#[test]
fn a_branch_switch_between_the_check_and_the_push_does_not_change_what_is_pushed() {
    let root = fixture("ap-switch-window");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let default = current_branch(&repo);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    git(&repo, &["checkout", "--quiet", "-b", "other"]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "other"]);
    commit_change(&repo, "other work\n");
    let other_tip = head_oid(&repo);
    git(&repo, &["checkout", "--quiet", &default]);
    commit_change(&repo, "default work\n");
    let landed = head_oid(&repo);
    let mut pushing = push_git(&repo);
    pushing.real_git = switching_git(&root, "other");
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&pushing, &format!("refs/heads/{default}"), &landed, &[], &[], &mut err).expect("push");
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert!(root.join("switched").exists(), "the switch happened inside the window");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_tip(&remote, &default).as_deref(), Some(landed.as_str()), "the recorded branch reached the remote: {shown}");
    assert_ne!(remote_tip(&remote, "other").as_deref(), Some(other_tip.as_str()), "the branch switched to was not published");
    remove(&root);
}

/// A recorded branch whose destination the settings leave unnamed (`push.default=matching`) is still pushed to the branch
/// it tracks,
/// by name,
/// so the plan needs no `HEAD` for it.
#[test]
fn a_recorded_branch_with_an_unnamed_destination_is_pushed_to_its_tracked_branch() {
    let root = fixture("ap-unnamed-destination");
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&remote);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let default = current_branch(&repo);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    git(&repo, &["config", "push.default", "matching"]);
    commit_change(&repo, "default work\n");
    let landed = head_oid(&repo);
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(&repo), &format!("refs/heads/{default}"), &landed, &[], &[], &mut err).expect("push");
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_tip(&remote, &default).as_deref(), Some(landed.as_str()), "the tracked branch reached the remote: {shown}");
    remove(&root);
}

/// The branch the triangular fixture records and the branch it tracks on the upstream remote.
const TRIANGULAR_BRANCH: &str = "feature";
const TRIANGULAR_TRACKED: &str = "main";

/// A triangular setup:
/// `feature` tracks `upstream/main` and pushes to `origin`;
/// one commit of `feature` is not yet published.
/// Returns the repository,
/// the push remote,
/// the upstream remote and the unpublished commit.
fn triangular_repo(root: &Path) -> (PathBuf, PathBuf, PathBuf, String) {
    let origin = root.join("origin.git");
    let upstream = root.join("upstream.git");
    let repo = root.join("repo");
    bare_remote_with_hook(&origin);
    bare_remote_with_hook(&upstream);
    init_repo_with_commit(&repo);
    git(&repo, &["remote", "add", "origin", origin.to_str().expect("utf-8")]);
    git(&repo, &["remote", "add", "upstream", upstream.to_str().expect("utf-8")]);
    git(&repo, &["push", "--quiet", "upstream", &format!("HEAD:refs/heads/{TRIANGULAR_TRACKED}")]);
    git(&repo, &["fetch", "--quiet", "upstream"]);
    git(&repo, &["checkout", "--quiet", "-b", TRIANGULAR_BRANCH, "--track", &format!("upstream/{TRIANGULAR_TRACKED}")]);
    git(&repo, &["config", &format!("branch.{TRIANGULAR_BRANCH}.pushRemote"), "origin"]);
    commit_change(&repo, "feature work\n");
    let landed = head_oid(&repo);
    return (repo, origin, upstream, landed);
}

/// Every branch a bare remote holds,
/// as `name=oid` lines,
/// so a test can show that nothing was published there.
fn remote_branches(remote: &Path) -> String {
    return String::from_utf8_lossy(&git(remote, &["for-each-ref", "--format=%(refname)=%(objectname)", "refs/heads"]).stdout).into_owned();
}

/// Push the triangular fixture's `feature` branch the way the service does.
fn push_triangular(repo: &Path, landed: &str) -> (AutoPushResult, String) {
    let (attempted, err) = run_for_branch(repo, &format!("refs/heads/{TRIANGULAR_BRANCH}"), landed);
    return (attempted.expect("push"), err);
}

/// A `remote.<name>.push` refspec decides where the branch goes:
/// the review branch gets the commit,
/// and neither the
/// branch's own name on the push remote nor the tracked branch is touched.
#[test]
fn a_configured_push_refspec_decides_where_a_named_branch_goes() {
    let root = fixture("ap-refspec-destination");
    let (repo, origin, upstream, landed) = triangular_repo(&root);
    let upstream_before = remote_branches(&upstream);
    git(&repo, &["config", "remote.origin.push", &format!("refs/heads/{TRIANGULAR_BRANCH}:refs/heads/review/{TRIANGULAR_BRANCH}")]);
    let (result, err) = push_triangular(&repo, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&origin, &format!("review/{TRIANGULAR_BRANCH}")).as_deref(), Some(landed.as_str()), "{err}");
    assert_eq!(remote_branches(&origin), format!("refs/heads/review/{TRIANGULAR_BRANCH}={landed}\n"), "nothing else was published");
    assert_eq!(remote_branches(&upstream), upstream_before, "the tracked remote is untouched");
    remove(&root);
}

/// A glob refspec maps the branch the way Git maps it.
#[test]
fn a_glob_push_refspec_maps_a_named_branch_the_way_git_does() {
    let root = fixture("ap-refspec-glob");
    let (repo, origin, _upstream, landed) = triangular_repo(&root);
    git(&repo, &["config", "remote.origin.push", "refs/heads/*:refs/heads/mirror/*"]);
    let (result, err) = push_triangular(&repo, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_tip(&origin, &format!("mirror/{TRIANGULAR_BRANCH}")).as_deref(), Some(landed.as_str()), "{err}");
    remove(&root);
}

/// A refspec that does not cover the branch (it names the checked-out branch,
/// `HEAD`) leaves Git with nothing to push for it,
/// so
/// the service pushes nothing and says so.
#[test]
fn a_push_refspec_that_does_not_cover_the_branch_pushes_nothing_and_says_so() {
    let root = fixture("ap-refspec-uncovered");
    let (repo, origin, upstream, landed) = triangular_repo(&root);
    let upstream_before = remote_branches(&upstream);
    git(&repo, &["config", "remote.origin.push", "HEAD"]);
    let (result, err) = push_triangular(&repo, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{err}");
    assert!(err.contains("`remote.origin.push` has `HEAD`"), "the note names the refspec it does not interpret: {err}");
    assert_eq!(remote_branches(&origin), "", "nothing was published on the push remote");
    assert_eq!(remote_branches(&upstream), upstream_before, "the tracked remote is untouched");
    remove(&root);
}

/// With `push.default=matching` a triangular branch the push remote does not have is not pushed,
/// as Git does not push it:
/// the
/// plain push finds no branch in common and fails with Git's words,
/// and nothing is published under the tracked branch's name.
#[test]
fn a_matching_push_of_a_branch_the_push_remote_lacks_publishes_nothing() {
    let root = fixture("ap-matching-absent");
    let (repo, origin, upstream, landed) = triangular_repo(&root);
    let upstream_before = remote_branches(&upstream);
    git(&repo, &["config", "push.default", "matching"]);
    let (result, err) = push_triangular(&repo, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert!(err.contains("No refs in common"), "{err}");
    assert_eq!(remote_branches(&origin), "", "the push remote gained no branch, least of all one named after the tracked branch");
    assert_eq!(remote_branches(&upstream), upstream_before, "the tracked remote is untouched");
    remove(&root);
}

/// With `push.default=matching` a triangular branch the push remote has goes to the same name there,
/// not to the tracked name.
#[test]
fn a_matching_push_of_a_branch_the_push_remote_has_goes_to_the_same_name() {
    let root = fixture("ap-matching-present");
    let (repo, origin, upstream, landed) = triangular_repo(&root);
    let upstream_before = remote_branches(&upstream);
    git(&repo, &["push", "--quiet", "origin", &format!("HEAD~1:refs/heads/{TRIANGULAR_BRANCH}")]);
    git(&repo, &["fetch", "--quiet", "origin"]);
    git(&repo, &["config", "push.default", "matching"]);
    let (result, err) = push_triangular(&repo, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_branches(&origin), format!("refs/heads/{TRIANGULAR_BRANCH}={landed}\n"), "{err}");
    assert_eq!(remote_branches(&upstream), upstream_before, "the tracked remote is untouched");
    remove(&root);
}

/// `push.default=nothing` and a triangular `push.default=upstream` are settings under which Git itself refuses to push the
/// branch,
/// so the service publishes nothing for either.
#[test]
fn a_setting_under_which_git_refuses_to_push_the_branch_publishes_nothing() {
    for mode in ["nothing", "upstream"] {
        let root = fixture(&format!("ap-refused-{mode}"));
        let (repo, origin, upstream, landed) = triangular_repo(&root);
        let upstream_before = remote_branches(&upstream);
        git(&repo, &["config", "push.default", mode]);
        let (result, err) = push_triangular(&repo, &landed);
        assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{mode}: {err}");
        assert!(err.contains(&format!("`push.default` is `{mode}`")), "{mode}: {err}");
        assert_eq!(remote_branches(&origin), "", "{mode}: the push remote gained no branch");
        assert_eq!(remote_branches(&upstream), upstream_before, "{mode}: the tracked remote is untouched");
        remove(&root);
    }
}

/// A push that delivered its commits but whose remote-tracking reference another process's lock refused exits `0`,
/// and the refusal is shown anyway:
/// it is the only sign that `git status` will report the branch ahead of a remote that already has the commits.
#[test]
fn a_clean_push_whose_reference_lock_was_refused_surfaces_the_refusal() {
    let output = "To origin\nremote: hello\nerror: cannot lock ref 'refs/remotes/origin/main': Unable to create '/repo/.git/refs/remotes/origin/main.lock': File exists.\n";
    let shown: String = filter_push_output(output, 0);
    assert!(shown.contains("cannot lock ref"), "{shown}");
    assert!(shown.contains("remote: hello"), "the remote's own lines stay: {shown}");
    assert!(!shown.contains("To origin"), "the other lines of a clean push stay out: {shown}");
}

/// The awaited push answers what the synchronous push answers:
/// a clean push of the checked-out branch,
/// its output and its exit status,
/// and the same for a push the remote refuses,
/// and the synchronous face of the same push is refused from inside the runtime.
#[test]
fn the_awaited_push_answers_what_the_synchronous_push_answers() {
    let root: PathBuf = fixture("auto-push-awaiting-same-answer");
    std::fs::create_dir_all(root.join("sync")).expect("the synchronous fixture's folder");
    std::fs::create_dir_all(root.join("awaited")).expect("the awaited fixture's folder");
    let (sync_repo, sync_log) = repo_with_remote_and_pre_push(root.join("sync").as_path(), 0);
    let (awaited_repo, awaited_log) = repo_with_remote_and_pre_push(root.join("awaited").as_path(), 0);
    let sync_landed: String = head_oid(sync_repo.as_path());
    let awaited_landed: String = head_oid(awaited_repo.as_path());
    let (sync_result, sync_shown): (Result<AutoPushResult, SingleFlightError>, String) = run(sync_repo.as_path(), sync_landed.as_str());
    let mut awaited_err: Vec<u8> = Vec::new();
    let awaited_result: Result<AutoPushResult, SingleFlightError> = crate::lazy_runtime::run_on_runtime(crate::auto_push::push_once_awaiting(
        &push_git(awaited_repo.as_path()),
        &head_ref(awaited_repo.as_path()),
        awaited_landed.as_str(),
        &[],
        &[],
        &mut awaited_err,
    ))
    .expect("the runtime runs the work");
    assert_eq!(
        format!("{awaited_result:?}"),
        format!("{sync_result:?}"),
        "the two faces answer the same push the same way"
    );
    assert_eq!(String::from_utf8_lossy(awaited_err.as_slice()).into_owned(), sync_shown);
    // Each face delivered the tip of its own repository, and its push created that repository's tracking ref.
    // The two repositories commit one after the other,
    // so their commit names differ whenever the clock passes a second between them:
    // comparing the two names compared a clock and not a push.
    for repo in [sync_repo.as_path(), awaited_repo.as_path()] {
        let delivered: String = String::from_utf8_lossy(&git(repo, &["rev-parse", "@{upstream}"]).stdout).trim().to_owned();
        let tip: String = String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
        assert_eq!(delivered, tip, "the push delivered {}'s own tip", repo.display());
        assert!(!tip.is_empty(), "{} has a tip", repo.display());
    }
    assert_eq!(read_log(sync_log.as_path()), read_log(awaited_log.as_path()), "both ran the same hook");
    // The positive control: the synchronous face inside the runtime is refused, not silently run.
    let mut refused_err: Vec<u8> = Vec::new();
    let refused = crate::lazy_runtime::run_on_runtime(async {
        return push_once(
            &push_git(awaited_repo.as_path()),
            &head_ref(awaited_repo.as_path()),
            awaited_landed.as_str(),
            &[],
            &[],
            &mut refused_err,
        );
    })
    .expect("the runtime runs the work");
    let refusal: String = match refused {
        Err(error) => format!("{error}"),
        Ok(result) => panic!("the synchronous push cannot start Git inside the runtime, and it answered {result:?}"),
    };
    assert!(refusal.contains("was asked for from inside itself"), "{refusal}");
    remove(root.as_path());
}

/// What:
///  A push that delivered the branch while another Git held the lock of its remote-tracking reference leaves that reference
///  naming the pushed commit.
/// Why:
///   Git delivers such a push, prints `cannot lock ref`, and exits `0`,
///  so the service records the push as made while the reference stays behind what the remote holds.
/// Every later question about the branch's remote is answered from that reference, so a stale one makes every command say a
///  published commit is held.
/// The fixture is a real push killed at the moment it holds that lock (`stale_ref_lock_killed_git`), which is that state:
///  the remote took the commit and the reference still names the commit before it.
#[test]
fn a_push_whose_tracking_reference_was_left_behind_writes_it() {
    let stage: crate::stale_ref_lock_killed_git::Stage = crate::stale_ref_lock_killed_git::stage("auto-push-tracking-ref-repaired", true);
    let mut held: crate::stale_ref_lock_killed_git::Held = crate::stale_ref_lock_killed_git::hold_push(&stage);
    assert!(stage.lock_of("refs/remotes/origin/main").exists(), "the killed push holds the tracking reference");
    held.kill();
    // The person clears the lock the notice names, and the next push of the branch runs with nothing left to deliver.
    std::fs::remove_file(stage.lock_of("refs/remotes/origin/main")).expect("the lock goes");
    let tip: String = head_oid(stage.repo.as_path());
    let tracked: String = git_text(stage.repo.as_path(), &["rev-parse", "refs/remotes/origin/main"]);
    assert_ne!(tracked, tip, "the fixture starts with the reference behind the branch, as the refused update left it");
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(stage.repo.as_path()), "refs/heads/main", tip.as_str(), &[], &[], &mut err).expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{}", String::from_utf8_lossy(err.as_slice()));
    let tracked_after: String = git_text(stage.repo.as_path(), &["rev-parse", "refs/remotes/origin/main"]);
    assert_eq!(tracked_after, tip, "the successful push leaves the tracking reference naming its own commit");
    stage.finish();
}
