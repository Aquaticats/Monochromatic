//! Controls for publishing a landed commit that a moved remote branch rejected, against real repositories and bare
//! remotes. Issue 1769: both histories reach the remote and the active worktree stays byte-identical.

use super::*;
use crate::auto_push::{AutoPushOutcome, AutoPushResult, PUSH_FAILED_NOTE, auto_push};
use crate::auto_push_single_flight::PushGit;
use crate::test_support::{fixture, git, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A bare remote, the repository under test, and a second clone that stands for another worktree's work.
struct Scenario {
    /// Fixture root, removed when the test ends.
    root: PathBuf,
    /// The bare remote named `origin`.
    remote: PathBuf,
    /// The repository whose commit is published.
    local: PathBuf,
    /// A clone that advances the remote.
    other: PathBuf,
}

/// Environment of every Git child of the coordinator in these fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    return [
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

/// The text Git printed for a read-only query, trimmed.
fn text_of(directory: &Path, arguments: &[&str]) -> String {
    return String::from_utf8_lossy(&git(directory, arguments).stdout)
        .trim()
        .to_owned();
}

/// Create the remote with `a.txt` and `b.txt`, the repository under test and the other clone.
fn scenario(name: &str) -> Scenario {
    let root = fixture(name);
    let remote = root.join("remote.git");
    let seed = root.join("seed");
    git(
        &root,
        &[
            "init",
            "--bare",
            "--quiet",
            "--initial-branch=main",
            "remote.git",
        ],
    );
    git(&root, &["init", "--quiet", "--initial-branch=main", "seed"]);
    std::fs::write(seed.join("a.txt"), "base\n").expect("a.txt");
    std::fs::write(seed.join("b.txt"), "base\n").expect("b.txt");
    git(&seed, &["add", "a.txt", "b.txt"]);
    git(&seed, &["commit", "--quiet", "-m", "seed"]);
    git(
        &seed,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    git(
        &seed,
        &["push", "--quiet", "--set-upstream", "origin", "main"],
    );
    git(
        &root,
        &["clone", "--quiet", remote.to_str().expect("utf-8"), "local"],
    );
    git(
        &root,
        &["clone", "--quiet", remote.to_str().expect("utf-8"), "other"],
    );
    return Scenario {
        local: root.join("local"),
        other: root.join("other"),
        remote,
        root,
    };
}

/// Change one file in the other clone, commit it and publish it.
fn advance_remote(scenario: &Scenario, file: &str, text: &str) {
    git(&scenario.other, &["fetch", "--quiet", "origin"]);
    git(
        &scenario.other,
        &["merge", "--quiet", "--ff-only", "origin/main"],
    );
    std::fs::write(scenario.other.join(file), text).expect("remote change");
    git(&scenario.other, &["add", file]);
    git(
        &scenario.other,
        &["commit", "--quiet", "-m", &format!("remote {file}")],
    );
    git(&scenario.other, &["push", "--quiet", "origin", "main"]);
}

/// Change one file in the repository under test and commit only that file.
fn commit_local(scenario: &Scenario, file: &str, text: &str) -> String {
    std::fs::write(scenario.local.join(file), text).expect("local change");
    git(
        &scenario.local,
        &[
            "commit",
            "--quiet",
            "-m",
            &format!("local {file}"),
            "--",
            file,
        ],
    );
    return text_of(&scenario.local, &["rev-parse", "HEAD"]);
}

/// Run auto-push in the repository under test and capture the text it prints.
fn run(scenario: &Scenario, landed: &str) -> (AutoPushResult, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = auto_push(&push_git(&scenario.local), landed, &mut err).expect("auto-push");
    return (result, String::from_utf8_lossy(&err).into_owned());
}

/// What a consumer of the worktree can observe: the status, the bytes of two files and the index file.
fn worktree_snapshot(local: &Path) -> (String, Vec<u8>, Vec<u8>, Vec<u8>) {
    return (
        text_of(local, &["status", "--porcelain=v1"]),
        std::fs::read(local.join("a.txt")).expect("a.txt"),
        std::fs::read(local.join("scratch.txt")).expect("scratch.txt"),
        std::fs::read(local.join(".git").join("index")).expect("index"),
    );
}

/// Install a hook in the repository under test that advances the remote each time the reconcile creates its private
/// fetch ref, while the count is at most `limit`: the push that follows then finds a remote that moved again.
fn advancing_hook(scenario: &Scenario, limit: u32) -> PathBuf {
    let hooks = scenario.root.join("hooks");
    let counter = scenario.root.join("counter");
    std::fs::create_dir_all(&hooks).expect("hooks");
    let hook = hooks.join("reference-transaction");
    let script = format!(
        "#!/bin/sh\n[ \"$1\" = committed ] || exit 0\ngrep -E '^0{{40}} [0-9a-f]{{40}} refs/cli-git/auto-push/' | grep -Evq '^0{{40}} 0{{40}} ' || exit 0\ncount=$(cat '{counter}' 2>/dev/null || echo 0)\ncount=$((count + 1))\necho $count > '{counter}'\n\
         if [ $count -le {limit} ]; then\n\
         unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE\n\
         git -C '{other}' fetch --quiet origin\n\
         git -C '{other}' merge --quiet --ff-only origin/main\n\
         echo $count > \"{other}/advance-$count.txt\"\n\
         git -C '{other}' add .\n\
         git -C '{other}' commit --quiet -m advance-$count\n\
         git -C '{other}' push --quiet origin main\n\
         fi\nexit 0\n",
        counter = counter.display(),
        other = scenario.other.display(),
    );
    std::fs::write(&hook, script).expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    git(
        &scenario.local,
        &["config", "core.hooksPath", hooks.to_str().expect("utf-8")],
    );
    return counter;
}

/// How many times the pre-push hook ran.
fn hook_runs(counter: &Path) -> u32 {
    return std::fs::read_to_string(counter)
        .expect("counter")
        .trim()
        .parse()
        .expect("number");
}

/// Both histories reach the remote as one merge above the landed commit, and the worktree is not touched.
#[test]
fn a_divergent_branch_is_published_as_a_merge_and_the_worktree_is_not_touched() {
    let s = scenario("ar-merge");
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    // The path the remote changed is dirty in this checkout, and an untracked file sits beside it.
    std::fs::write(s.local.join("a.txt"), "dirty-local\n").expect("dirty file");
    std::fs::write(s.local.join("scratch.txt"), "scratch\n").expect("untracked file");
    let landed = commit_local(&s, "b.txt", "local\n");
    let before = worktree_snapshot(&s.local);
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(
        text_of(
            &s.remote,
            &["rev-list", "--parents", "--max-count=1", "main"]
        ),
        format!("{published} {landed} {remote_before}")
    );
    assert_eq!(text_of(&s.remote, &["show", "main:a.txt"]), "remote");
    assert_eq!(text_of(&s.remote, &["show", "main:b.txt"]), "local");
    assert_eq!(
        text_of(&s.local, &["rev-parse", "HEAD"]),
        landed,
        "the local branch moved"
    );
    assert_eq!(
        worktree_snapshot(&s.local),
        before,
        "the worktree, the index or an untracked file changed"
    );
    assert_eq!(
        text_of(&s.local, &["for-each-ref", "refs/cli-git"]),
        "",
        "the private fetch ref remains"
    );
    assert!(err.contains(RECONCILE_LINE_PREFIX), "{err}");
    assert!(err.contains(&published), "{err}");
    assert!(!err.contains(PUSH_FAILED_NOTE), "{err}");
    remove(&s.root);
}

/// A conflicting merge publishes nothing, names the paths and does not recommend a push that cannot work.
#[test]
fn a_conflicting_merge_is_kept_unpublished_with_the_conflicting_paths() {
    let s = scenario("ar-conflict");
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let landed = commit_local(&s, "a.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_ne!(result.exit_code, 0);
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert!(err.contains(RECONCILE_FAILED_PREFIX), "{err}");
    assert!(err.contains("conflicts in: a.txt"), "{err}");
    assert!(!err.contains(PUSH_FAILED_NOTE), "{err}");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// An amended published commit is merged like any other divergence: nothing published is lost, the replaced commit
/// stays in the remote history beside the amended one, and the push is not refused.
#[test]
fn an_amended_published_commit_is_merged_when_nothing_conflicts() {
    let s = scenario("ar-amend");
    let first = commit_local(&s, "b.txt", "first\n");
    let (published, _) = run(&s, &first);
    assert_eq!(published.outcome, AutoPushOutcome::Pushed);
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    git(
        &s.local,
        &["commit", "--quiet", "--amend", "-m", "amended", "b.txt"],
    );
    let amended = text_of(&s.local, &["rev-parse", "HEAD"]);
    let (result, err) = run(&s, &amended);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let merged = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(
        text_of(&s.remote, &["rev-list", "--parents", "--max-count=1", "main"]),
        format!("{merged} {amended} {remote_before}")
    );
    assert_eq!(text_of(&s.remote, &["show", "main:a.txt"]), "remote");
    assert_eq!(text_of(&s.remote, &["show", "main:b.txt"]), "first");
    remove(&s.root);
}

/// An amend that changes the same lines as the published commit is a conflict, and the conflict is reported.
#[test]
fn an_amend_that_conflicts_with_the_published_commit_is_reported() {
    let s = scenario("ar-amend-conflict");
    let first = commit_local(&s, "b.txt", "first\n");
    let (published, _) = run(&s, &first);
    assert_eq!(published.outcome, AutoPushOutcome::Pushed);
    advance_remote(&s, "c.txt", "remote\n");
    std::fs::write(s.local.join("b.txt"), "second\n").expect("amend change");
    git(
        &s.local,
        &["commit", "--quiet", "--amend", "-m", "amended", "b.txt"],
    );
    // The remote's copy of the published commit changes the same line differently.
    git(&s.other, &["fetch", "--quiet", "origin"]);
    git(&s.other, &["merge", "--quiet", "--ff-only", "origin/main"]);
    std::fs::write(s.other.join("b.txt"), "third\n").expect("remote change");
    git(&s.other, &["commit", "--quiet", "-m", "remote b", "--", "b.txt"]);
    git(&s.other, &["push", "--quiet", "origin", "main"]);
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let amended = text_of(&s.local, &["rev-parse", "HEAD"]);
    let (result, err) = run(&s, &amended);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert!(err.contains("conflicts in: b.txt"), "{err}");
    remove(&s.root);
}

/// A remote that moves again during the push is fetched and merged again, within the bound.
#[test]
fn a_remote_that_moves_again_is_merged_again() {
    let s = scenario("ar-retry");
    advance_remote(&s, "a.txt", "remote\n");
    let counter = advancing_hook(&s, 2);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    // The remote moved after each of the first two fetches; the merge made after the third fetch was accepted.
    assert_eq!(hook_runs(&counter), 3);
    assert_eq!(text_of(&s.remote, &["show", "main:advance-1.txt"]), "1");
    assert_eq!(text_of(&s.remote, &["show", "main:advance-2.txt"]), "2");
    assert_eq!(text_of(&s.remote, &["show", "main:b.txt"]), "local");
    assert_eq!(
        text_of(&s.remote, &["merge-base", "--is-ancestor", &landed, "main"]).len(),
        0
    );
    remove(&s.root);
}

/// Busy sibling worktrees are not a conflict: a remote that moves five times in a row is still merged into and
/// published, which needs more attempts than a short bound allows.
#[test]
fn a_busy_remote_is_merged_into_until_it_settles() {
    let s = scenario("ar-busy");
    advance_remote(&s, "a.txt", "remote\n");
    let counter = advancing_hook(&s, 5);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    assert_eq!(hook_runs(&counter), 6);
    assert_eq!(text_of(&s.remote, &["show", "main:advance-5.txt"]), "5");
    assert_eq!(text_of(&s.remote, &["show", "main:b.txt"]), "local");
    remove(&s.root);
}

/// A remote that never settles ends after the bound with the reason, and nothing newer than the rejection is pushed.
#[test]
fn a_remote_that_never_settles_is_reported_after_the_bound() {
    let s = scenario("ar-exhausted");
    advance_remote(&s, "a.txt", "remote\n");
    let counter = advancing_hook(&s, 100);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_ne!(result.exit_code, 0);
    // One fetch per attempt, and the remote moved after each.
    assert_eq!(hook_runs(&counter), MAX_RECONCILE_ATTEMPTS);
    assert!(err.contains(RECONCILE_FAILED_PREFIX), "{err}");
    assert!(
        err.contains(&format!("during each of {MAX_RECONCILE_ATTEMPTS} attempts")),
        "{err}"
    );
    assert!(!err.contains(PUSH_FAILED_NOTE), "{err}");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// A commit that the remote already contains is reported as delivered, without a push.
#[test]
fn a_commit_the_remote_already_contains_is_reported_as_delivered() {
    let s = scenario("ar-contained");
    let landed = commit_local(&s, "b.txt", "local\n");
    git(&s.local, &["push", "--quiet", "origin", "main"]);
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert!(err.contains("already contains"), "{err}");
    remove(&s.root);
}

/// A push remote other than the tracked one is reconciled against the push remote: the rejection came from there.
#[test]
fn a_push_remote_other_than_the_upstream_is_reconciled_against_the_push_remote() {
    let s = scenario("ar-push-remote");
    git(
        &s.local,
        &["remote", "add", "mirror", s.remote.to_str().expect("utf-8")],
    );
    git(&s.local, &["config", "branch.main.pushRemote", "mirror"]);
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let merged = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(
        text_of(&s.remote, &["rev-list", "--parents", "--max-count=1", "main"]),
        format!("{merged} {landed} {remote_before}")
    );
    assert!(err.contains("mirror/main had new commits"), "{err}");
    remove(&s.root);
}

/// A remote that cannot be fetched leaves the rejection explained and nothing pushed.
#[test]
fn a_remote_that_cannot_be_fetched_is_reported_without_a_push() {
    let s = scenario("ar-fetch-fails");
    git(
        &s.local,
        &[
            "config",
            "remote.origin.pushurl",
            s.remote.to_str().expect("utf-8"),
        ],
    );
    git(
        &s.local,
        &[
            "config",
            "remote.origin.url",
            s.root.join("missing.git").to_str().expect("utf-8"),
        ],
    );
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_ne!(result.exit_code, 0);
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert!(err.contains("fetching them failed"), "{err}");
    assert!(!err.contains(PUSH_FAILED_NOTE), "{err}");
    remove(&s.root);
}

/// Only a stale-update refusal is reconciled; any other failed push is returned as it came.
#[test]
fn a_failure_that_is_not_a_stale_update_is_returned_unchanged() {
    let s = scenario("ar-other-failure");
    let refused = PushAttempt {
        exit_code: 1,
        output: "remote: error: hook declined".to_owned(),
    };
    assert!(!was_rejected_as_stale(&refused.output));
    assert!(was_rejected_as_stale(
        " ! [rejected]        main -> main (non-fast-forward)"
    ));
    assert_eq!(
        reconcile_divergent_push(&push_git(&s.local), "refs/heads/main", refused.clone())
            .expect("reconcile"),
        refused
    );
    let stale = PushAttempt {
        exit_code: 1,
        output: " ! [rejected] v1 -> v1".to_owned(),
    };
    assert_eq!(
        reconcile_divergent_push(&push_git(&s.local), "refs/tags/v1", stale.clone())
            .expect("reconcile"),
        stale,
        "a ref that is not a branch has no upstream to merge"
    );
    remove(&s.root);
}

#[test]
fn an_unpublished_attempt_never_reports_success_and_keeps_a_real_exit_code() {
    let from_zero = unpublished(RECONCILE_FAILED_PREFIX, "before", 0, "reason");
    assert_eq!(from_zero.exit_code, 1, "a failure must not carry exit code zero");
    assert_eq!(
        from_zero.output,
        format!("before\n{RECONCILE_FAILED_PREFIX}reason")
    );
    assert_eq!(
        unpublished(RECONCILE_FAILED_PREFIX, "before", 128, "reason").exit_code,
        128
    );
}

/// The destination the reconcile would fetch from, as `remote merge_ref`, or nothing.
fn destination_of(scenario: &Scenario, branch_ref: &str) -> Option<String> {
    return read_destination(&push_git(&scenario.local), branch_ref)
        .expect("destination")
        .map(|destination| {
            return format!("{} {}", destination.remote, destination.merge_ref);
        });
}

/// The branch that tracks its own remote branch is the one case that is reconciled.
#[test]
fn a_branch_that_pushes_to_its_upstream_has_that_destination() {
    let s = scenario("ar-dest-upstream");
    assert_eq!(
        destination_of(&s, "refs/heads/main").as_deref(),
        Some("origin refs/heads/main")
    );
    remove(&s.root);
}

/// A branch without an upstream, or named outside `refs/heads`, has no destination to fetch from.
#[test]
fn a_branch_without_an_upstream_or_outside_the_heads_has_no_destination() {
    let s = scenario("ar-dest-none");
    git(&s.local, &["branch", "--quiet", "loose"]);
    assert_eq!(destination_of(&s, "refs/heads/loose"), None);
    assert_eq!(destination_of(&s, "refs/tags/main"), None);
    remove(&s.root);
}

/// A push remote other than the tracked one is where a triangular push goes, so it is the destination.
#[test]
fn a_push_remote_that_differs_from_the_upstream_is_the_destination() {
    let s = scenario("ar-dest-pushremote");
    git(
        &s.local,
        &["remote", "add", "mirror", s.remote.to_str().expect("utf-8")],
    );
    git(&s.local, &["config", "branch.main.pushRemote", "mirror"]);
    assert_eq!(
        destination_of(&s, "refs/heads/main").as_deref(),
        Some("mirror refs/heads/main")
    );
    git(&s.local, &["config", "--unset", "branch.main.pushRemote"]);
    git(&s.local, &["config", "remote.pushDefault", "mirror"]);
    assert_eq!(
        destination_of(&s, "refs/heads/main").as_deref(),
        Some("mirror refs/heads/main"),
        "remote.pushDefault is the next in Git's order"
    );
    remove(&s.root);
}

/// `push.default` picks the ref: `current` pushes to the branch's own name, `upstream` to the tracked ref, and
/// `matching` and `nothing` name no single destination.
#[test]
fn push_default_decides_which_ref_is_the_destination() {
    let s = scenario("ar-dest-modes");
    git(&s.local, &["branch", "--quiet", "--track", "feature", "origin/main"]);
    let cases = [
        ("current", Some("origin refs/heads/feature")),
        ("upstream", Some("origin refs/heads/main")),
        ("tracking", Some("origin refs/heads/main")),
        ("matching", None),
        ("nothing", None),
    ];
    for (mode, expected) in cases {
        git(&s.local, &["config", "push.default", mode]);
        assert_eq!(
            destination_of(&s, "refs/heads/feature").as_deref(),
            expected,
            "{mode}"
        );
    }
    remove(&s.root);
}

/// With the default `simple`, a push remote other than the tracked one gets the branch's own name, and the tracked
/// remote gets the tracked ref; a branch whose name differs from what it tracks tells the two apart.
#[test]
fn simple_pushes_by_name_to_a_push_remote_and_to_the_tracked_ref_otherwise() {
    let s = scenario("ar-dest-simple");
    git(&s.local, &["branch", "--quiet", "--track", "feature", "origin/main"]);
    assert_eq!(
        destination_of(&s, "refs/heads/feature").as_deref(),
        Some("origin refs/heads/main")
    );
    git(
        &s.local,
        &["remote", "add", "mirror", s.remote.to_str().expect("utf-8")],
    );
    git(&s.local, &["config", "branch.feature.pushRemote", "mirror"]);
    assert_eq!(
        destination_of(&s, "refs/heads/feature").as_deref(),
        Some("mirror refs/heads/feature")
    );
    remove(&s.root);
}

/// An explicit `remote.<name>.push` refspec overrides every default, so the destination is not the one derived.
#[test]
fn a_configured_push_refspec_has_no_destination() {
    let s = scenario("ar-dest-refspec");
    git(
        &s.local,
        &["config", "remote.origin.push", "refs/heads/main:refs/heads/elsewhere"],
    );
    assert_eq!(destination_of(&s, "refs/heads/main"), None);
    remove(&s.root);
}

/// A branch that tracks a local branch has the remote `.`, which is never fetched from.
#[test]
fn a_branch_that_tracks_a_local_branch_has_no_destination() {
    let s = scenario("ar-dest-dot");
    git(&s.local, &["branch", "--quiet", "--track", "topic", "main"]);
    git(&s.local, &["config", "push.default", "upstream"]);
    assert_eq!(
        text_of(&s.local, &["rev-parse", "--symbolic-full-name", "topic@{push}"]),
        "refs/heads/main",
        "the push query must agree with the upstream, or the remote check is not what decides"
    );
    assert_eq!(destination_of(&s, "refs/heads/topic"), None);
    remove(&s.root);
}

/// An upstream outside `refs/heads` cannot be fetched into a private ref by branch name.
#[test]
fn an_upstream_outside_the_heads_has_no_destination() {
    let s = scenario("ar-dest-namespace");
    git(
        &s.local,
        &[
            "config",
            "--add",
            "remote.origin.fetch",
            "+refs/notes/*:refs/remotes/origin/notes/*",
        ],
    );
    git(&s.local, &["update-ref", "refs/remotes/origin/notes/x", "HEAD"]);
    git(&s.local, &["config", "branch.main.merge", "refs/notes/x"]);
    assert_eq!(destination_of(&s, "refs/heads/main"), None);
    remove(&s.root);
}

/// A rejection while the remote is behind the branch was not a divergence, so it is returned as it came.
#[test]
fn a_rejection_while_the_remote_is_behind_is_returned_unchanged() {
    let s = scenario("ar-behind");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    commit_local(&s, "b.txt", "local\n");
    let rejected = PushAttempt {
        exit_code: 1,
        output: " ! [rejected]        main -> main (fetch first)".to_owned(),
    };
    let returned = reconcile_divergent_push(&push_git(&s.local), "refs/heads/main", rejected.clone())
        .expect("reconcile");
    assert_eq!(returned, rejected);
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// Histories that share no commit are merged like any other: that is not a conflict, and the merge has both parents.
#[test]
fn unrelated_histories_are_merged() {
    let s = scenario("ar-unrelated");
    git(&s.local, &["checkout", "--quiet", "--orphan", "island"]);
    git(&s.local, &["rm", "--quiet", "--force", "-r", "."]);
    git(&s.local, &["commit", "--quiet", "--allow-empty", "-m", "island"]);
    let island = text_of(&s.local, &["rev-parse", "HEAD"]);
    let main = text_of(&s.local, &["rev-parse", "main"]);
    let merged = commit_merge(&push_git(&s.local), &island, &main, "message")
        .expect("git")
        .expect("no conflict");
    assert_eq!(
        text_of(&s.local, &["rev-list", "--parents", "--max-count=1", &merged]),
        format!("{merged} {island} {main}")
    );
    remove(&s.root);
}

/// Make the repository under test sign commits with a stand-in `gpg` that exits `status`: it reads the payload,
/// answers the status line Git waits for and prints a placeholder signature, so no key or keyring is involved.
fn sign_commits_with_stand_in(scenario: &Scenario, status: u32) {
    let program = scenario.root.join("stand-in-gpg");
    let script = format!(
        "#!/bin/sh\ncat > /dev/null\n[ {status} -eq 0 ] || exit {status}\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nplaceholder\\n-----END PGP SIGNATURE-----\\n'\n"
    );
    std::fs::write(&program, script).expect("stand-in gpg");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("mode");
    git(
        &scenario.local,
        &["config", "gpg.program", program.to_str().expect("utf-8")],
    );
    git(&scenario.local, &["config", "commit.gpgsign", "true"]);
}

/// Whether the commit `oid` of the remote carries a signature header.
fn is_signed(scenario: &Scenario, oid: &str) -> bool {
    return text_of(&scenario.remote, &["cat-file", "commit", oid]).contains("\ngpgsig ");
}

/// A repository that signs its commits publishes a signed merge, so a remote that requires verified signatures
/// accepts it; `git commit-tree` does not read `commit.gpgsign` by itself.
#[test]
fn the_merge_is_signed_when_the_repository_signs_commits() {
    let s = scenario("ar-signed");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    sign_commits_with_stand_in(&s, 0);
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert!(is_signed(&s, &published), "the merge is unsigned");
    remove(&s.root);
}

/// A repository that does not sign its commits gets an unsigned merge, and no signer is run.
#[test]
fn the_merge_is_unsigned_when_the_repository_does_not_sign_commits() {
    let s = scenario("ar-unsigned");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    sign_commits_with_stand_in(&s, 1);
    git(&s.local, &["config", "commit.gpgsign", "false"]);
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert!(!is_signed(&s, &published), "the merge is signed");
    remove(&s.root);
}

/// A signer that fails leaves the merge uncreated and the failure visible, never an unsigned merge in its place.
#[test]
fn a_failing_signer_publishes_nothing() {
    let s = scenario("ar-signer-fails");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    sign_commits_with_stand_in(&s, 1);
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let (result, err) = run(&s, &landed);
    assert_ne!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_ne!(result.exit_code, 0, "{err}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// A `commit.gpgsign` value that `git config --type=bool` rejects cannot be read as a yes or a no, so nothing is
/// published rather than guessing between a signed and an unsigned merge.
#[test]
fn an_unreadable_signing_setting_publishes_nothing() {
    let s = scenario("ar-gpgsign-unreadable");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    git(&s.local, &["config", "commit.gpgsign", "sometimes"]);
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let (result, err) = run(&s, &landed);
    assert_ne!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_ne!(result.exit_code, 0, "{err}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// Fetch the remote branch into the tracking ref and return its tip, which a clean checkout of the old tip may fast-forward to.
fn fetched_remote_tip(scenario: &Scenario) -> String {
    git(&scenario.local, &["fetch", "--quiet", "origin"]);
    return text_of(&scenario.local, &["rev-parse", "origin/main"]);
}

/// Run the fast-forward of the repository under test from its current tip to `target`.
fn advance(scenario: &Scenario, target: &str) -> Result<bool, String> {
    let local = text_of(&scenario.local, &["rev-parse", "HEAD"]);
    return advance_local_branch(&push_git(&scenario.local), "refs/heads/main", &local, target)
        .expect("git");
}

/// A clean worktree on the pushed branch follows the published merge, so the next commit starts from it.
#[test]
fn a_clean_worktree_is_fast_forwarded_to_the_published_merge() {
    let s = scenario("ar-follow");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), published);
    assert_eq!(text_of(&s.local, &["rev-parse", "main"]), published);
    assert_eq!(
        std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"),
        "remote\n"
    );
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "");
    assert!(err.contains("main and the worktree were fast-forwarded"), "{err}");
    assert!(!err.contains("unchanged"), "{err}");
    remove(&s.root);
}

/// A tracked change in a file the move writes keeps the branch and the worktree where they are, and the report names
///  the file and the command.
#[test]
fn a_worktree_with_a_tracked_change_is_left_behind_with_the_reason_reported() {
    let s = scenario("ar-dirty-report");
    advance_remote(&s, "a.txt", "remote\n");
    std::fs::write(s.local.join("a.txt"), "dirty-local\n").expect("dirty file");
    let landed = commit_local(&s, "b.txt", "local\n");
    let (_result, err) = run(&s, &landed);
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), landed);
    assert!(
        err.contains(
            "unchanged (the worktree has uncommitted changes to files the merge changes: a.txt)"
        ),
        "{err}"
    );
    assert!(err.contains("git merge --ff-only origin/main"), "{err}");
    remove(&s.root);
}

/// The fast-forward moves the checked-out branch to the target and nothing else.
#[test]
fn the_fast_forward_moves_a_clean_checkout_to_the_target() {
    let s = scenario("ar-ff");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    assert_eq!(advance(&s, &target), Ok(false));
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), target);
    remove(&s.root);
}

/// A detached `HEAD` is not a branch to move.
#[test]
fn a_detached_head_is_not_fast_forwarded() {
    let s = scenario("ar-ff-detached");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    git(&s.local, &["checkout", "--quiet", "--detach"]);
    assert_eq!(advance(&s, &target), Err(String::from("HEAD is detached")));
    assert_eq!(text_of(&s.local, &["rev-parse", "main"]), before);
    remove(&s.root);
}

/// Another branch checked out in this worktree leaves the pushed branch alone.
#[test]
fn another_checked_out_branch_is_not_fast_forwarded() {
    let s = scenario("ar-ff-other");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    git(&s.local, &["switch", "--quiet", "--create", "topic"]);
    assert_eq!(
        advance(&s, &target),
        Err(String::from("this worktree has another branch checked out"))
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "main"]), before);
    assert_eq!(text_of(&s.local, &["rev-parse", "topic"]), before);
    remove(&s.root);
}

/// A branch whose tip is no longer the pushed tip moved on its own, and is not moved again.
#[test]
fn a_branch_that_moved_after_the_landing_is_not_fast_forwarded() {
    let s = scenario("ar-ff-moved");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    let result = advance_local_branch(
        &push_git(&s.local),
        "refs/heads/main",
        &target,
        &target,
    )
    .expect("git");
    assert_eq!(
        result,
        Err(String::from("the branch moved after the commit landed"))
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), before);
    remove(&s.root);
}

/// A modified tracked file the move writes blocks the move and is named.
#[test]
fn a_modified_file_the_move_writes_blocks_the_fast_forward() {
    let s = scenario("ar-ff-modified");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    std::fs::write(s.local.join("a.txt"), "edited\n").expect("edit");
    assert_eq!(
        advance(&s, &target),
        Err(String::from(
            "the worktree has uncommitted changes to files the merge changes: a.txt"
        ))
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), before);
    remove(&s.root);
}

/// A modified tracked file the move does not write stays as it is while the branch follows.
#[test]
fn a_modified_file_the_move_leaves_alone_is_kept_while_the_branch_follows() {
    let s = scenario("ar-ff-disjoint");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    std::fs::write(s.local.join("b.txt"), "edited\n").expect("edit");
    let status = text_of(&s.local, &["status", "--porcelain=v1"]);
    assert_eq!(advance(&s, &target), Ok(true));
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), target);
    assert_eq!(
        std::fs::read_to_string(s.local.join("b.txt")).expect("b.txt"),
        "edited\n"
    );
    assert_eq!(
        std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"),
        "remote\n"
    );
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), status);
    remove(&s.root);
}

/// A staged new file the move does not write stays staged while the branch follows.
#[test]
fn a_staged_file_the_move_leaves_alone_stays_staged() {
    let s = scenario("ar-ff-staged");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    std::fs::write(s.local.join("c.txt"), "staged\n").expect("c.txt");
    git(&s.local, &["add", "c.txt"]);
    assert_eq!(advance(&s, &target), Ok(true));
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), target);
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "A  c.txt");
    remove(&s.root);
}

/// The incident: a staged rename the move does not write survives the fast-forward untouched.
#[test]
fn a_staged_rename_the_move_leaves_alone_survives() {
    let s = scenario("ar-ff-rename");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    git(&s.local, &["mv", "b.txt", "moved.txt"]);
    let status = text_of(&s.local, &["status", "--porcelain=v1"]);
    assert_eq!(status, "R  b.txt -> moved.txt");
    assert_eq!(advance(&s, &target), Ok(true));
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), target);
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), status);
    remove(&s.root);
}

/// A staged rename blocks the move through either of its names.
#[test]
fn a_staged_rename_blocks_the_move_through_either_name() {
    let s = scenario("ar-ff-rename-blocked");
    advance_remote(&s, "b.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    git(&s.local, &["mv", "b.txt", "moved.txt"]);
    assert_eq!(
        advance(&s, &target),
        Err(String::from(
            "the worktree has uncommitted changes to files the merge changes: b.txt"
        ))
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), before);
    assert_eq!(
        text_of(&s.local, &["status", "--porcelain=v1"]),
        "R  b.txt -> moved.txt"
    );
    remove(&s.root);
}

/// A staged change to a file the move writes blocks the move.
#[test]
fn a_staged_change_to_a_file_the_move_writes_blocks_it() {
    let s = scenario("ar-ff-staged-blocked");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    std::fs::write(s.local.join("a.txt"), "staged\n").expect("a.txt");
    git(&s.local, &["add", "a.txt"]);
    assert_eq!(
        advance(&s, &target),
        Err(String::from(
            "the worktree has uncommitted changes to files the merge changes: a.txt"
        ))
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), before);
    remove(&s.root);
}

/// The whole auto-push reports that the branch followed and the work was kept.
#[test]
fn the_report_says_the_uncommitted_work_was_left_as_it_is() {
    let s = scenario("ar-ff-report-kept");
    advance_remote(&s, "a.txt", "remote\n");
    std::fs::write(s.local.join("c.txt"), "new\n").expect("c.txt");
    git(&s.local, &["add", "c.txt"]);
    let landed = commit_local(&s, "c.txt", "local\n");
    std::fs::write(s.local.join("b.txt"), "edited\n").expect("edit");
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), published);
    assert!(
        err.contains("fast-forwarded to it; your uncommitted changes were left as they are."),
        "{err}"
    );
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "M b.txt");
    remove(&s.root);
}

/// An untracked file does not block the move unless the target would overwrite it; Git then refuses and nothing changes.
#[test]
fn an_untracked_file_the_target_would_overwrite_is_kept() {
    let s = scenario("ar-ff-untracked");
    advance_remote(&s, "new.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    std::fs::write(s.local.join("new.txt"), "mine\n").expect("untracked");
    let result = advance(&s, &target);
    assert!(
        result
            .as_ref()
            .is_err_and(|reason| {
                return reason.starts_with("Git refused the fast-forward: ");
            }),
        "{result:?}"
    );
    assert_eq!(
        std::fs::read_to_string(s.local.join("new.txt")).expect("new.txt"),
        "mine\n"
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), before);
    remove(&s.root);
}

/// An untracked file the target does not touch is not a reason to stay behind.
#[test]
fn an_unrelated_untracked_file_does_not_block_the_fast_forward() {
    let s = scenario("ar-ff-scratch");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    std::fs::write(s.local.join("scratch.txt"), "scratch\n").expect("untracked");
    assert_eq!(advance(&s, &target), Ok(false));
    assert_eq!(
        std::fs::read_to_string(s.local.join("scratch.txt")).expect("scratch.txt"),
        "scratch\n"
    );
    remove(&s.root);
}

/// A target that is not above the branch tip is not a fast-forward, and Git's refusal is reported.
#[test]
fn a_target_that_is_not_a_fast_forward_is_refused() {
    let s = scenario("ar-ff-diverged");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let landed = commit_local(&s, "b.txt", "local\n");
    let result = advance(&s, &target);
    assert!(
        result
            .as_ref()
            .is_err_and(|reason| {
                return reason.starts_with("Git refused the fast-forward: ");
            }),
        "{result:?}"
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), landed);
    remove(&s.root);
}

/// A status that cannot be read is a reason to stay behind, never a failure of the published push.
#[test]
fn an_unreadable_status_blocks_the_fast_forward() {
    let s = scenario("ar-ff-status");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let before = text_of(&s.local, &["rev-parse", "HEAD"]);
    std::fs::write(s.local.join(".git").join("index"), b"not an index").expect("index");
    assert_eq!(
        advance(&s, &target),
        Err(String::from("the worktree status could not be read"))
    );
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), before);
    remove(&s.root);
}

/// The local branch stays behind after a reconcile that could not fast-forward it, so the next commit does not
/// contain the published merge; that is a branch that moved on, not one that was rewritten, and its auto-push is
/// reconciled again.
#[test]
fn a_second_commit_on_a_branch_left_behind_by_a_reconcile_is_reconciled_again() {
    let s = scenario("ar-twice");
    advance_remote(&s, "a.txt", "remote\n");
    // A tracked change keeps the worktree from following the merge.
    std::fs::write(s.local.join("a.txt"), "dirty-local\n").expect("dirty file");
    let first = commit_local(&s, "b.txt", "local\n");
    let (first_result, first_err) = run(&s, &first);
    assert_eq!(
        (first_result.outcome, first_result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{first_err}"
    );
    let first_merge = text_of(&s.remote, &["rev-parse", "main"]);
    // The local branch is not advanced; a new commit lands on top of the first local tip.
    let second = commit_local(&s, "b.txt", "local second\n");
    let (second_result, second_err) = run(&s, &second);
    assert_eq!(
        (second_result.outcome, second_result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{second_err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(
        text_of(
            &s.remote,
            &["rev-list", "--parents", "--max-count=1", "main"]
        ),
        format!("{published} {second} {first_merge}")
    );
    remove(&s.root);
}

/// A failed attempt must not make the branch look rewritten: the record of a failure keeps the last tip known to be
/// published, so once the remote accepts pushes again the next commit is reconciled instead of staying rejected until
/// someone merges by hand.
#[test]
fn a_failed_attempt_between_two_reconciles_does_not_stop_the_next_one() {
    let s = scenario("ar-failure-between");
    advance_remote(&s, "a.txt", "remote\n");
    // A tracked change keeps the worktree from following the merge, so the next commit starts from the old tip.
    std::fs::write(s.local.join("a.txt"), "dirty-local\n").expect("dirty file");
    let first = commit_local(&s, "b.txt", "local\n");
    let (first_result, first_err) = run(&s, &first);
    assert_eq!(
        (first_result.outcome, first_result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{first_err}"
    );
    let first_merge = text_of(&s.remote, &["rev-parse", "main"]);
    // The remote refuses every update for one attempt, which records a failure.
    let refusal = s.remote.join("hooks/pre-receive");
    std::fs::create_dir_all(s.remote.join("hooks")).expect("hooks");
    std::fs::write(&refusal, "#!/bin/sh\nexit 1\n").expect("pre-receive");
    std::fs::set_permissions(&refusal, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let second = commit_local(&s, "b.txt", "local second\n");
    let (second_result, second_err) = run(&s, &second);
    assert_ne!(second_result.exit_code, 0, "{second_err}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), first_merge);
    // The remote accepts updates again, and the next commit is published.
    std::fs::remove_file(&refusal).expect("remove pre-receive");
    let third = commit_local(&s, "b.txt", "local third\n");
    let (third_result, third_err) = run(&s, &third);
    assert_eq!(
        (third_result.outcome, third_result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{third_err}"
    );
    let published = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(
        text_of(
            &s.remote,
            &["rev-list", "--parents", "--max-count=1", "main"]
        ),
        format!("{published} {third} {first_merge}")
    );
    remove(&s.root);
}

/// Install a `pre-receive` hook on the bare remote that declines the first `declines` pushes the way GitHub's protected
/// branch does, and, when `advance` is set, first moves the remote branch from the other clone: the race behind GH006.
/// Returns the file that counts the hook's runs.
fn declining_remote_hook(scenario: &Scenario, declines: u32, advance: bool) -> PathBuf {
    let counter = scenario.root.join("receive-counter");
    let git_without_hook_environment = "env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_QUARANTINE_PATH \
                                        -u GIT_OBJECT_DIRECTORY -u GIT_ALTERNATE_OBJECT_DIRECTORIES git";
    let advance_block = if advance {
        format!(
            "if [ $count -eq 1 ]; then\n\
             {git} -C '{other}' fetch --quiet origin\n\
             {git} -C '{other}' merge --quiet --ff-only origin/main\n\
             echo moved > '{other}/moved.txt'\n\
             {git} -C '{other}' add moved.txt\n\
             {git} -C '{other}' commit --quiet -m moved\n\
             {git} -C '{other}' push --quiet origin main\n\
             fi\n",
            git = git_without_hook_environment,
            other = scenario.other.display(),
        )
    } else {
        String::new()
    };
    let script = format!(
        "#!/bin/sh\ncount=$(cat '{counter}' 2>/dev/null || echo 0)\ncount=$((count + 1))\necho $count > '{counter}'\n\
         if [ $count -le {declines} ]; then\n{advance_block}\
         echo 'GH006: Protected branch update failed for refs/heads/main.' >&2\nexit 1\nfi\nexit 0\n",
        counter = counter.display(),
    );
    let hook = scenario.remote.join("hooks").join("pre-receive");
    std::fs::write(&hook, script).expect("pre-receive hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    return counter;
}

/// GitHub prints a protected-branch refusal as `[remote rejected]`; when the remote branch moved meanwhile, it is the
/// same race as a stale update and the merge is published (issue 1032).
#[test]
fn a_protected_branch_refusal_while_the_remote_moved_is_reconciled() {
    let s = scenario("ar-gh006-moved");
    declining_remote_hook(&s, 1, true);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(
        (result.outcome, result.exit_code),
        (AutoPushOutcome::Pushed, 0),
        "{err}"
    );
    assert_eq!(text_of(&s.remote, &["show", "main:b.txt"]), "local");
    assert_eq!(text_of(&s.remote, &["show", "main:moved.txt"]), "moved");
    remove(&s.root);
}

/// A refusal from a remote that did not move is a real refusal: nothing is merged and the push output is unchanged.
#[test]
fn a_protected_branch_refusal_while_the_remote_is_behind_is_reported_as_it_came() {
    let s = scenario("ar-gh006-behind");
    let counter = declining_remote_hook(&s, 100, false);
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert!(result.shown.contains("[remote rejected]"), "{}", result.shown);
    assert!(err.contains(PUSH_FAILED_NOTE), "{err}");
    assert!(!err.contains(RECONCILE_LINE_PREFIX), "{err}");
    assert_eq!(hook_runs(&counter), 1);
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    remove(&s.root);
}

/// A refusal of the merge itself, by a remote that has not moved since the merge was made, is final: the merge is not
/// pushed again and the refusal is not reported as a remote that kept moving.
#[test]
fn a_refusal_of_the_merge_by_an_unmoved_remote_ends_the_attempts() {
    let s = scenario("ar-gh006-merge-refused");
    advance_remote(&s, "a.txt", "remote\n");
    let counter = declining_remote_hook(&s, 100, false);
    let landed = commit_local(&s, "b.txt", "local\n");
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert!(result.shown.contains("[remote rejected]"), "{}", result.shown);
    assert!(!err.contains("during each of"), "{err}");
    assert_eq!(hook_runs(&counter), 1);
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// Equal status text is no change, and any difference, even one byte, is a change.
#[test]
fn a_status_differs_only_when_its_bytes_do() {
    assert!(!status_differs(b"", b""));
    assert!(!status_differs(b" M a.txt\0", b" M a.txt\0"));
    assert!(status_differs(b" M a.txt\0", b""));
    assert!(status_differs(b"", b" M a.txt\0"));
    assert!(status_differs(b" M a.txt\0", b" M b.txt\0"));
}
