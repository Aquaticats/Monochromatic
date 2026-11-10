//! Controls for publishing a landed commit that a moved remote branch rejected,
//! against real repositories and bare
//! remotes.
//! Issue 1769:
//! both histories reach the remote and the active worktree stays byte-identical.

use super::*;
use crate::auto_push::{AutoPushOutcome, AutoPushResult, PUSH_FAILED_NOTE, push_once};
use crate::auto_push_single_flight::PushGit;
use crate::auto_push_reconcile_basics_and_limits::{fetch_remote_tip, fetch_remote_tip_awaiting, is_ancestor, is_ancestor_awaiting};
use crate::auto_push_reconcile_branch_advance::status_differs;
use crate::auto_push_reconcile_merge_commit::{common_dir_of, common_dir_of_awaiting, signs_commits, signs_commits_awaiting};
use crate::auto_push_reconcile_merge_commit::PushRunner;
use crate::merge_second_pass::GitRunner;
use crate::merge_second_pass::tests::await_it;
use crate::test_support::{fixture, git, remove};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A function-like value with the right shape is a merge push in these controls:
/// the shipped callers write a small struct that holds the state.
impl<Push: FnMut(&Destination, &str) -> Result<PushAttempt, SingleFlightError> + Send> MergePusher for Push {
    fn push_merge(&mut self, destination: &Destination, merge_oid: &str) -> Result<PushAttempt, SingleFlightError> {
        return self(destination, merge_oid);
    }
}

/// A control's move of the branch:
/// the person's `git switch topic` first,
/// then exactly the step that ships.
struct SwitchThenMove {
    /// The worktree the switch is made in.
    local: PathBuf,
}

impl crate::auto_push_reconcile_branch_advance::BranchMove for SwitchThenMove {
    fn move_branch<'a>(
        &'a self,
        git_ref: &'a PushGit,
        branch_ref: &'a str,
        local: &'a str,
        merge_oid: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Result<(), String>, SingleFlightError>> + Send + 'a>> {
        return Box::pin(async move {
            git(&self.local, &["switch", "--quiet", "topic"]);
            return crate::auto_push_reconcile_branch_advance::move_the_named_branch(git_ref, branch_ref, local, merge_oid).await;
        });
    }
}

/// A bare remote,
/// the repository under test,
/// and a second clone that stands for another worktree's work.
struct Scenario {
    /// Fixture root,
    /// removed when the test ends.
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

/// The text Git printed for a read-only query,
/// trimmed.
fn text_of(directory: &Path, arguments: &[&str]) -> String {
    return String::from_utf8_lossy(&git(directory, arguments).stdout)
        .trim()
        .to_owned();
}

/// Create the remote with `a.txt` and `b.txt`,
/// the repository under test and the other clone.
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

/// Change one file in the other clone,
/// commit it and publish it.
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

/// Push `main` of the repository under test the way the service does and capture the text it prints.
fn run(scenario: &Scenario, landed: &str) -> (AutoPushResult, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(&scenario.local), "refs/heads/main", landed, &mut err).expect("auto-push");
    return (result, String::from_utf8_lossy(&err).into_owned());
}

/// Push `main` of the repository under test and return the failure the service would report,
/// which a push that cannot
/// even compute its merge ends with.
fn run_error(scenario: &Scenario, landed: &str) -> SingleFlightError {
    let mut err: Vec<u8> = Vec::new();
    return push_once(&push_git(&scenario.local), "refs/heads/main", landed, &mut err).expect_err("the failure is returned");
}

/// What a consumer of the worktree can observe:
/// the status,
/// the bytes of two files and the index file.
fn worktree_snapshot(local: &Path) -> (String, Vec<u8>, Vec<u8>, Vec<u8>) {
    return (
        text_of(local, &["status", "--porcelain=v1"]),
        std::fs::read(local.join("a.txt")).expect("a.txt"),
        std::fs::read(local.join("scratch.txt")).expect("scratch.txt"),
        std::fs::read(local.join(".git").join("index")).expect("index"),
    );
}

/// Install a hook in the repository under test that advances the remote each time the reconcile creates its private
/// fetch ref,
/// while the count is at most `limit`:
/// the push that follows then finds a remote that moved again.
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
    // A configuration hook: traditional hooks never run through the wrapper's real-Git starts.
    git(&scenario.local, &["config", "hook.control.event", "reference-transaction"]);
    git(&scenario.local, &["config", "hook.control.command", hook.to_str().expect("utf-8")]);
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

/// Both histories reach the remote as one merge above the landed commit,
/// and the worktree is not touched.
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

/// The reconcile's fetch runs without a low-speed abort of the wrapper's own:
/// like the push,
/// it waits while its connection is
///  open (owner,
/// 2026-11-03:
/// "Wait while connected (Recommended)";
/// pick 5 of the shipped-limits audit,
/// the coordinator's under
///  the owner's delegation).
/// A stand-in for Git records the two variables the fetch is started with,
/// then runs the real Git.
#[test]
fn the_reconcile_fetch_sets_no_low_speed_abort() {
    let s = scenario("ar-fetch-no-low-speed");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let record = s.root.join("fetch-environment.txt");
    let stand_in = s.root.join("git-stand-in");
    std::fs::write(
        &stand_in,
        format!(
            "#!/bin/sh\nfor word in \"$@\"; do\n[ \"$word\" = fetch ] && echo \"$GIT_HTTP_LOW_SPEED_LIMIT|$GIT_HTTP_LOW_SPEED_TIME\" >> '{}'\ndone\nexec /usr/bin/git \"$@\"\n",
            record.display()
        ),
    )
    .expect("stand-in");
    std::fs::set_permissions(&stand_in, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let mut coordinator = push_git(&s.local);
    coordinator.real_git = stand_in;
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&coordinator, "refs/heads/main", &landed, &mut err).expect("auto-push");
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{}", String::from_utf8_lossy(&err));
    assert_eq!(
        std::fs::read_to_string(&record).expect("the fetch ran"),
        "|\n",
        "neither variable is set for the fetch"
    );
    remove(&s.root);
}

/// A conflicting merge publishes nothing,
/// names the paths and does not recommend a push that cannot work.
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

/// An amended published commit is merged like any other divergence:
/// nothing published is lost,
/// the replaced commit
/// stays in the remote history beside the amended one,
/// and the push is not refused.
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

/// An amend that changes the same lines as the published commit is a conflict,
/// and the conflict is reported.
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

/// A remote that moves again during the push is fetched and merged again,
/// within the bound.
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

/// Busy sibling worktrees are not a conflict:
/// a remote that moves five times in a row is still merged into and
/// published,
/// which needs more attempts than a short bound allows.
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

/// A remote that never settles ends after the bound with the reason,
/// and nothing newer than the rejection is pushed.
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

/// A commit that the remote already contains is reported as delivered,
/// without a push.
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

/// A push remote other than the tracked one is reconciled against the push remote:
/// the rejection came from there.
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

/// Only a stale-update refusal is reconciled;
/// any other failed push is returned as it came.
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

/// The destination the reconcile would fetch from,
/// as `remote merge_ref`,
/// or nothing.
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

/// A branch without an upstream,
/// or named outside `refs/heads`,
/// has no destination to fetch from.
#[test]
fn a_branch_without_an_upstream_or_outside_the_heads_has_no_destination() {
    let s = scenario("ar-dest-none");
    git(&s.local, &["branch", "--quiet", "loose"]);
    assert_eq!(destination_of(&s, "refs/heads/loose"), None);
    assert_eq!(destination_of(&s, "refs/tags/main"), None);
    remove(&s.root);
}

/// A push remote other than the tracked one is where a triangular push goes,
/// so it is the destination.
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

/// `push.default` picks the ref:
/// `current` pushes to the branch's own name,
/// `upstream` to the tracked ref,
/// and
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

/// With the default `simple`,
/// a push remote other than the tracked one gets the branch's own name,
/// and the tracked
/// remote gets the tracked ref;
/// a branch whose name differs from what it tracks tells the two apart.
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

/// An explicit `remote.<name>.push` refspec overrides every default,
/// so the destination is not the one derived.
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

/// A branch that tracks a local branch has the remote `.`,
/// which is never fetched from.
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

/// A rejection while the remote is behind the branch was not a divergence,
/// so it is returned as it came.
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

/// Histories that share no commit are merged like any other:
/// that is not a conflict,
/// and the merge has both parents.
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
        .expect("no conflict")
        .oid;
    assert_eq!(
        text_of(&s.local, &["rev-list", "--parents", "--max-count=1", &merged]),
        format!("{merged} {island} {main}")
    );
    remove(&s.root);
}

/// What:
///  A merge driver that never returns but keeps writing is stopped at the merge step's total ceiling.
/// Why:
///   The stall window cannot bound it:
/// the driver rewrites a scratch file for ever,
/// so every sample of the process tree shows bytes moved,
/// and the wait would never end
/// (cross-vendor review round 2,
/// finding 3).
/// A ceiling on the whole command ends it,
/// and the words say what was not changed.
#[test]
fn a_merge_driver_that_keeps_working_is_stopped_at_the_merge_ceiling() {
    let s = scenario("ar-driver-busy");
    let scratch: PathBuf = s.root.join("driver-scratch");
    std::fs::write(s.local.join(".gitattributes"), "driver.txt merge=stall\n").expect("the rule");
    std::fs::write(s.local.join("driver.txt"), "base\n").expect("the file");
    git(&s.local, &["add", "--", ".gitattributes", "driver.txt"]);
    git(&s.local, &["commit", "--quiet", "-m", "a rule that names a driver and the file it owns"]);
    git(&s.local, &["push", "--quiet", "origin", "main"]);
    // The driver writes for ever, which is progress the stall window can never call a stall.
    let driver: String = format!("sh -c 'while :; do printf x > {}; sleep 0.05; done'", scratch.display());
    git(&s.local, &["config", "merge.stall.driver", driver.as_str()]);
    advance_remote(&s, "driver.txt", "remote\n");
    let island: String = commit_local(&s, "driver.txt", "local\n");
    git(&s.local, &["fetch", "--quiet", "origin"]);
    let main: String = text_of(&s.local, &["rev-parse", "origin/main"]);
    let mut git_under_test: PushGit = push_git(&s.local);
    git_under_test.environment.push((OsString::from(crate::push_deadline_and_patience_settings::MERGE_CEILING_VARIABLE), OsString::from("2")));
    let started: std::time::Instant = std::time::Instant::now();
    let answer = commit_merge(&git_under_test, &island, &main, "message");
    let waited: std::time::Duration = started.elapsed();
    let said: String = match &answer {
        Err(error) => format!("{error}"),
        Ok(_) => String::from("the merge was made"),
    };
    assert!(said.contains("was still running at its 2-second ceiling"), "the ceiling that stopped it is named: {said}");
    assert!(said.contains("Nothing was changed"), "the words say what was not touched: {said}");
    assert!(waited < std::time::Duration::from_secs(60), "the wait ends at the ceiling and its grace, not never: {waited:?}");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "", "no merge ref is left");
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), island, "the commit is kept and nothing was committed");
    assert_eq!(std::fs::read(s.local.join("driver.txt")).expect("the file"), b"local\n".to_vec(), "the worktree was not touched");
    remove(&s.root);
}

/// What:
///  A merge driver that never returns ends the reconcile merge,
/// and the failure names the wait that stopped it.
/// Why:
///   The commands of the word-level second pass were started with no bound at all,
/// so a driver blocked on a read that never comes held the push
///  worker,
/// its claim and the drain of that repository for good
/// (cross-vendor review of this branch,
/// finding 4).
/// The window is the push's own,
/// so it is set to one second here and the grace is the one a stopped query gets.
#[test]
fn a_merge_driver_that_never_returns_stops_the_reconcile_merge() {
    let s = scenario("ar-driver-stall");
    std::fs::write(s.local.join(".gitattributes"), "driver.txt merge=stall\n").expect("the rule");
    std::fs::write(s.local.join("driver.txt"), "base\n").expect("the file");
    git(&s.local, &["add", "--", ".gitattributes", "driver.txt"]);
    git(&s.local, &["commit", "--quiet", "-m", "a rule that names a driver and the file it owns"]);
    git(&s.local, &["push", "--quiet", "origin", "main"]);
    // The driver waits for input that never arrives.
    git(&s.local, &["config", "merge.stall.driver", "sh -c 'sleep 300'"]);
    advance_remote(&s, "driver.txt", "remote\n");
    let island: String = commit_local(&s, "driver.txt", "local\n");
    git(&s.local, &["fetch", "--quiet", "origin"]);
    let main: String = text_of(&s.local, &["rev-parse", "origin/main"]);
    let mut git_under_test: PushGit = push_git(&s.local);
    git_under_test.environment.push((OsString::from(crate::push_deadline_and_patience_settings::PUSH_DEADLINE_VARIABLE), OsString::from("1")));
    let started: std::time::Instant = std::time::Instant::now();
    let answer = commit_merge(&git_under_test, &island, &main, "message");
    let waited: std::time::Duration = started.elapsed();
    let said: String = match &answer {
        Err(error) => format!("{error}"),
        Ok(_) => String::from("the merge was committed"),
    };
    assert!(said.contains("could not be run"), "the stopped command is named with its reason: {said}");
    assert!(said.contains("showed no sign of work"), "the wait that stopped it is the meter's: {said}");
    assert!(waited < std::time::Duration::from_secs(60), "the wait ends within the window and its grace, not never: {waited:?}");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "", "no merge ref is left");
    remove(&s.root);
}

/// What:
///  A `merge` rule the remote introduces governs the reconcile merge's second pass,
/// so the file it excludes keeps its conflict and nothing is committed.
/// Why:
///   The rule arrives with the side being merged,
/// so only the tree the merge produced holds it
/// (cross-vendor review of this branch,
/// finding 2;
/// the audit recorded this control as missing).
/// Reading the first side's tip
///  let the file be word-merged and published.
#[test]
fn a_rule_the_remote_introduces_keeps_the_reconcile_merge_from_merging_the_file() {
    use crate::merge_second_pass::tests::corpus;
    let s = scenario("ar-introduced-rule");
    std::fs::write(s.local.join("data.txt"), corpus("md1b", "base")).expect("the base file");
    git(&s.local, &["add", "--", "data.txt"]);
    git(&s.local, &["commit", "--quiet", "-m", "the base file"]);
    git(&s.local, &["push", "--quiet", "origin", "main"]);
    // The rule and the second side's bytes arrive from the remote in one commit.
    git(&s.other, &["fetch", "--quiet", "origin"]);
    git(&s.other, &["merge", "--quiet", "--ff-only", "origin/main"]);
    std::fs::write(s.other.join(".gitattributes"), "data.txt merge=binary\n").expect("the rule");
    std::fs::write(s.other.join("data.txt"), corpus("md1b", "theirs")).expect("their edit");
    git(&s.other, &["add", "--", ".gitattributes", "data.txt"]);
    git(&s.other, &["commit", "--quiet", "-m", "a rule and the edit it owns"]);
    git(&s.other, &["push", "--quiet", "origin", "main"]);
    let island: String = commit_local(&s, "data.txt", String::from_utf8_lossy(corpus("md1b", "ours").as_slice()).into_owned().as_str());
    git(&s.local, &["fetch", "--quiet", "origin"]);
    let main: String = text_of(&s.local, &["rev-parse", "origin/main"]);
    let answer = commit_merge(&push_git(&s.local), &island, &main, "message").expect("git");
    let Err(paths) = answer else { panic!("the rule the merge introduces excludes the file") };
    assert_eq!(paths, vec![String::from("data.txt")], "the excluded path is the one reported");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "", "no merge ref is left");
    remove(&s.root);
}

/// Make the repository under test sign commits with a stand-in `gpg` that exits `status`:
/// it reads the payload,
/// answers the status line Git waits for and prints a placeholder signature,
/// so no key or keyring is involved.
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

/// A repository that signs its commits publishes a signed merge,
/// so a remote that requires verified signatures
/// accepts it;
/// `git commit-tree` does not read `commit.gpgsign` by itself.
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

/// A repository that does not sign its commits gets an unsigned merge,
/// and no signer is run.
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

/// A signer that fails leaves the merge uncreated and the failure visible,
/// never an unsigned merge in its place.
#[test]
fn a_failing_signer_publishes_nothing() {
    let s = scenario("ar-signer-fails");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    sign_commits_with_stand_in(&s, 1);
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let error = run_error(&s, &landed);
    assert!(error.to_string().contains("failed to sign"), "{error}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// A `commit.gpgsign` value that `git config --type=bool` rejects cannot be read as a yes or a no,
/// so nothing is
/// published rather than guessing between a signed and an unsigned merge.
#[test]
fn an_unreadable_signing_setting_publishes_nothing() {
    let s = scenario("ar-gpgsign-unreadable");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    git(&s.local, &["config", "commit.gpgsign", "sometimes"]);
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let error = run_error(&s, &landed);
    assert!(error.to_string().contains("commit.gpgsign"), "{error}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "");
    remove(&s.root);
}

/// Fetch the remote branch into the tracking ref and return its tip,
/// which a clean checkout of the old tip may fast-forward to.
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

/// A clean worktree on the pushed branch follows the published merge,
/// so the next commit starts from it.
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

/// A tracked change in a file the move writes keeps the branch and the worktree where they are,
/// and the report names
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

/// What:
///  The record that guards a fast-forward is written in the worktree's own Git directory when that path is not text.
/// Why:
///   The directory comes from `rev-parse --absolute-git-dir`,
/// which prints its bytes;
/// read as text it names another directory,
/// or none,
/// so the record is written where recovery never looks
///  and a kill between Git's checkout and its ref update leaves a merge with no intent.
#[test]
fn the_fast_forward_record_is_written_in_the_git_directory_of_a_path_that_is_not_text() {
    use crate::merge_isolation_journal::{MERGE_INTENT_DIRECTORY, list_merge_intents};
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("ar-ff-bytes");
    let repo: PathBuf = root.join(OsStr::from_bytes(b"r\xffpo"));
    std::fs::create_dir_all(repo.as_path()).expect("repository directory");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a.txt");
    git(&repo, &["add", "a.txt"]);
    git(&repo, &["commit", "--quiet", "-m", "one"]);
    let old: String = text_of(&repo, &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a.txt");
    git(&repo, &["commit", "--quiet", "-m", "two", "--", "a.txt"]);
    let target: String = text_of(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["reset", "--quiet", "--hard", old.as_str()]);
    // A stand-in for Git looks for the record when the move writes the checkout's files, which is the instant a kill would
    //  leave it for; the move names its branch rather than running `git merge --ff-only`, so the step it watches is
    //  `read-tree`.
    let seen: PathBuf = root.join("record-seen");
    let shim: PathBuf = root.join("watching-git");
    let script = format!(
        "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = read-tree ]; then\n    if ls .git/{MERGE_INTENT_DIRECTORY}/*.json > /dev/null 2>&1; then echo present > '{seen}'; else echo absent > '{seen}'; fi\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        seen = seen.display()
    );
    std::fs::write(&shim, script).expect("stand-in");
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut watching: PushGit = push_git(repo.as_path());
    watching.real_git = shim;
    let _moved: () = await_it(crate::auto_push_reconcile_branch_advance::fast_forward_recorded(&watching, "refs/heads/main", old.as_str(), target.as_str(), &crate::auto_push_reconcile_branch_advance::SHIPPED_MOVE))
        .expect("git")
        .expect("the branch and the checkout moved");
    assert_eq!(std::fs::read_to_string(seen.as_path()).expect("the stand-in looked"), "present\n", "the record is in the worktree's own Git directory");
    assert_eq!(text_of(&repo, &["rev-parse", "HEAD"]), target);
    assert!(list_merge_intents(repo.join(".git").as_path()).expect("listing").is_empty(), "the record goes with the settled merge");
    remove(root.as_path());
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

/// A branch whose tip is no longer the pushed tip moved on its own,
/// and is not moved again.
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

/// The incident:
/// a staged rename the move does not write survives the fast-forward untouched.
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

/// An untracked file does not block the move unless the target would overwrite it;
/// Git then refuses and nothing changes.
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

/// A target that is not above the branch tip is not a fast-forward,
/// and Git's refusal is reported.
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

/// A status that cannot be read is a reason to stay behind,
/// never a failure of the published push.
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

/// The local branch stays behind after a reconcile that could not fast-forward it,
/// so the next commit does not
/// contain the published merge;
/// that is a branch that moved on,
/// not one that was rewritten,
/// and its auto-push is
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

/// A failed attempt must not make the branch look rewritten:
/// the record of a failure keeps the last tip known to be
/// published,
/// so once the remote accepts pushes again the next commit is reconciled instead of staying rejected until
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
/// branch does,
/// and,
/// when `advance` is set,
/// first moves the remote branch from the other clone:
/// the race behind GH006.
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

/// GitHub prints a protected-branch refusal as `[remote rejected]`;
/// when the remote branch moved meanwhile,
/// it is the
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

/// A refusal from a remote that did not move is a real refusal:
/// nothing is merged and the push output is unchanged.
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

/// A refusal of the merge itself,
/// by a remote that has not moved since the merge was made,
/// is final:
/// the merge is not
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

/// Equal status text is no change,
/// and any difference,
/// even one byte,
/// is a change.
#[test]
fn a_status_differs_only_when_its_bytes_do() {
    assert!(!status_differs(b"", b""));
    assert!(!status_differs(b" M a.txt\0", b" M a.txt\0"));
    assert!(status_differs(b" M a.txt\0", b""));
    assert!(status_differs(b"", b" M a.txt\0"));
    assert!(status_differs(b" M a.txt\0", b" M b.txt\0"));
}

/// A branch whose upstream has another name is a branch the merge is into,
/// and the message names the ref the merge is published to.
/// Native Git writes the same name for the same merge:
/// `git merge origin/main` while `feature` is checked out made
/// `Merge remote-tracking branch 'origin/main' into feature`
/// (measured on this host with Git 2.56.0,
/// 2026-11-09T09:33:10-05:00).
/// Issue 2646:
/// the message named the tracked ref,
/// a branch that was never checked out or merged into.
#[test]
fn a_branch_whose_upstream_has_another_name_is_the_one_the_message_names() {
    let s = scenario("ar-gh2646-other-upstream-name");
    advance_remote(&s, "a.txt", "remote\n");
    // The README's case: a branch created from the tracked ref, whose own name differs from it.
    git(&s.local, &["switch", "--quiet", "-c", "feature"]);
    git(&s.local, &["config", "branch.feature.remote", "origin"]);
    git(&s.local, &["config", "branch.feature.merge", "refs/heads/main"]);
    let landed = commit_local(&s, "b.txt", "local\n");
    let mut published: String = String::new();
    let mut push_merge = |destination: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        published = merge_oid.to_owned();
        git(&s.local, &["push", "--quiet", "origin", &format!("{merge_oid}:{}", destination.merge_ref)]);
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let attempt = finished(integrate_for(&s, &landed, Some("refs/heads/feature"), "refs/heads/main", false, &mut push_merge));
    assert_eq!(attempt.exit_code, 0, "{attempt:?}");
    let message: String = text_of(&s.local, &["log", "-1", "--format=%B", published.as_str()]);
    let mut lines = message.lines();
    assert_eq!(lines.next(), Some("Merge origin/main into feature"), "{message}");
    assert!(message.contains("Published to origin/main."), "{message}");
    assert!(message.contains("because origin/main advanced after the commit landed."), "{message}");
    remove(&s.root);
}

/// What:
///  An explicit `feature:main` push whose branch tracks `origin/feature` publishes a merge whose message names only what
///  is true of that push.
/// Why:
///  `Reconcile.destination` is the ref the rejected push updated,
///  which is not always the branch's upstream.
///  A commit message is permanent,
///  and the line said the branch tracked the destination,
///  which it did not.
#[test]
fn the_published_line_names_the_push_destination_and_no_tracking_relation() {
    let s = scenario("ar-gh2646-tracking-elsewhere");
    advance_remote(&s, "a.txt", "remote\n");
    // The review's sequence: the branch tracks its own remote branch, and the push goes to another ref.
    git(&s.local, &["switch", "--quiet", "-c", "feature"]);
    git(&s.local, &["config", "branch.feature.remote", "origin"]);
    git(&s.local, &["config", "branch.feature.merge", "refs/heads/feature"]);
    let landed = commit_local(&s, "b.txt", "local\n");
    let mut published: String = String::new();
    let mut push_merge = |destination: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        published = merge_oid.to_owned();
        git(&s.local, &["push", "--quiet", "origin", &format!("{merge_oid}:{}", destination.merge_ref)]);
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let attempt = finished(integrate_for(&s, &landed, Some("refs/heads/feature"), "refs/heads/main", true, &mut push_merge));
    assert_eq!(attempt.exit_code, 0, "{attempt:?}");
    let message: String = text_of(&s.local, &["log", "-1", "--format=%B", published.as_str()]);
    assert_eq!(message.lines().next(), Some("Merge origin/main into feature"), "{message}");
    assert!(message.contains("Published to origin/main."), "{message}");
    assert!(!message.contains("tracks"), "the message states no tracking relation: {message}");
    assert_eq!(text_of(&s.remote, &["log", "-1", "--format=%s", "main"]), "Merge origin/main into feature");
    remove(&s.root);
}

/// A branch named like its upstream is merged into the branch the message always named,
/// and no line names a publication the subject does not already spell.
#[test]
fn a_branch_named_like_its_upstream_keeps_the_message_it_had() {
    let s = scenario("ar-gh2646-same-upstream-name");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let mut published: String = String::new();
    let mut push_merge = |destination: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        published = merge_oid.to_owned();
        git(&s.local, &["push", "--quiet", "origin", &format!("{merge_oid}:{}", destination.merge_ref)]);
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let attempt = finished(integrate_for(&s, &landed, Some("refs/heads/main"), "refs/heads/main", false, &mut push_merge));
    assert_eq!(attempt.exit_code, 0, "{attempt:?}");
    let message: String = text_of(&s.local, &["log", "-1", "--format=%B", published.as_str()]);
    assert_eq!(message.lines().next(), Some("Merge origin/main into main"), "{message}");
    assert!(!message.contains("Published to"), "{message}");
    remove(&s.root);
}

/// What:
///  A conflict is reported against the branch the merge was into,
///  the name the merge's subject uses.
/// Why:
///  The refusal named the destination ref,
///  so an explicit `feature:main` push reported a merge into `main` that was into `feature`.
#[test]
fn a_conflict_is_reported_against_the_branch_the_merge_was_into() {
    let s = scenario("ar-gh-conflict-target-name");
    advance_remote(&s, "a.txt", "remote\n");
    git(&s.local, &["switch", "--quiet", "-c", "feature"]);
    let landed = commit_local(&s, "a.txt", "local\n");
    let mut push_merge = |_destination: &Destination, _merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        panic!("a conflicting merge publishes nothing");
    };
    let attempt = finished(integrate_for(&s, &landed, Some("refs/heads/feature"), "refs/heads/main", true, &mut push_merge));
    assert_ne!(attempt.exit_code, 0, "{attempt:?}");
    assert!(attempt.output.contains("merging origin/main into feature"), "{}", attempt.output);
    assert!(!attempt.output.contains("into main"), "{}", attempt.output);
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), text_of(&s.local, &["rev-parse", "origin/main"]), "nothing was published");
    remove(&s.root);
}

/// Run the shared reconcile loop for the repository under test with a caller-supplied merge push.
fn integrate(
    scenario: &Scenario,
    landed: &str,
    explicit: bool,
    push_merge: &mut dyn MergePusher,
) -> Integrated {
    return integrate_for(scenario, landed, Some("refs/heads/main"), "refs/heads/main", explicit, push_merge);
}

/// Run the shared reconcile loop for the branch `branch_ref`,
/// whose upstream is `merge_ref` on `origin`,
/// against the repository under test with a caller-supplied merge push.
fn integrate_for(
    scenario: &Scenario,
    landed: &str,
    branch_ref: Option<&str>,
    merge_ref: &str,
    explicit: bool,
    push_merge: &mut dyn MergePusher,
) -> Integrated {
    let plan = Reconcile {
        destination: Destination {
            remote: String::from("origin"),
            merge_ref: merge_ref.to_owned(),
        },
        local: landed.to_owned(),
        branch_ref,
        label: if explicit { "push" } else { "auto-push" },
        explicit,
    };
    let rejected = PushAttempt {
        exit_code: 1,
        output: String::from(" ! [rejected] main -> main (fetch first)"),
    };
    return integrate_and_push(&push_git(&scenario.local), &plan, rejected, push_merge)
        .expect("integrate");
}

/// What a run ended with,
/// which must be a finished one.
fn finished(integrated: Integrated) -> PushAttempt {
    return match integrated {
        Integrated::Finished(attempt) => attempt,
        Integrated::NotDivergent(attempt) => {
            panic!("the run was expected to finish, not stand down: {attempt:?}")
        }
    };
}

/// A diverged remote,
/// a first merge push that fails with `first_output` after the remote moved once more,
/// and a second
/// merge push that is delivered to the real remote;
/// returns how many merge pushes ran,
/// how the run ended,
/// and whether
/// the landed commit reached the remote.
fn failing_merge_push_while_the_remote_moves(
    name: &str,
    explicit: bool,
    first_output: &str,
) -> (u32, PushAttempt, bool) {
    let s = scenario(name);
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let mut pushes: u32 = 0;
    let mut push_merge = |_destination: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        pushes += 1;
        if pushes == 1 {
            // The remote moved after the first fetch, and the first merge push failed with `first_output`.
            advance_remote(&s, "c.txt", "moved\n");
            return Ok(PushAttempt {
                exit_code: 1,
                output: first_output.to_owned(),
            });
        }
        git(
            &s.local,
            &["push", "--quiet", "origin", &format!("{merge_oid}:refs/heads/main")],
        );
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let attempt = finished(integrate(&s, &landed, explicit, &mut push_merge));
    let delivered = text_of(&s.remote, &["show", "main:b.txt"]) == "local";
    remove(&s.root);
    return (pushes, attempt, delivered);
}

/// An explicit push that failed while the remote already holds the pushed commit failed for another reason:
/// the run
/// stands down and leaves the push's own report,
/// where an auto-push would report the commit as delivered.
#[test]
fn an_explicit_push_whose_commit_the_remote_already_contains_stands_down() {
    let s = scenario("ar-explicit-contained");
    let landed = commit_local(&s, "b.txt", "local\n");
    git(&s.local, &["push", "--quiet", "origin", "main"]);
    advance_remote(&s, "a.txt", "remote\n");
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let mut pushes: u32 = 0;
    let mut push_merge = |_destination: &Destination, _merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        pushes += 1;
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let integrated = integrate(&s, &landed, true, &mut push_merge);
    let Integrated::NotDivergent(attempt) = integrated else {
        panic!("an explicit push must stand down when the remote contains its commit");
    };
    assert_eq!(attempt.exit_code, 1);
    assert_eq!(pushes, 0, "nothing is pushed");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    remove(&s.root);
}

/// An auto-push whose merge push fails for a reason that is not a stale update reports that failure at once:
/// the remote
/// moving meanwhile does not make the failure a race.
#[test]
fn an_auto_push_whose_merge_push_fails_for_another_reason_is_not_retried() {
    let (pushes, attempt, delivered) = failing_merge_push_while_the_remote_moves(
        "ar-auto-other-failure",
        false,
        "error: failed to push some refs (a pre-push hook declined)",
    );
    assert_eq!(pushes, 1, "the failure is final");
    assert_eq!(attempt.exit_code, 1);
    assert!(attempt.output.contains("a pre-push hook declined"), "{}", attempt.output);
    assert!(!delivered, "nothing was published");
}

/// An auto-push whose merge push is refused as stale fetches again and merges again.
#[test]
fn an_auto_push_whose_merge_push_is_refused_as_stale_is_retried() {
    let (pushes, attempt, delivered) = failing_merge_push_while_the_remote_moves(
        "ar-auto-stale-failure",
        false,
        " ! [rejected] main -> main (fetch first)",
    );
    assert_eq!(pushes, 2, "the second merge sees the moved remote");
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(delivered, "the landed commit was published");
}

/// An explicit push reports no output of its own,
/// so a failed merge push is retried when the remote moved,
/// whatever
/// the refusal said.
#[test]
fn an_explicit_push_whose_merge_push_failed_is_retried_when_the_remote_moved() {
    let (pushes, attempt, delivered) =
        failing_merge_push_while_the_remote_moves("ar-explicit-failure", true, "");
    assert_eq!(pushes, 2, "the second merge sees the moved remote");
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(delivered, "the landed commit was published");
}

/// An explicit push whose merge push was refused with a stale marker is retried the same way.
#[test]
fn an_explicit_push_whose_merge_push_is_refused_as_stale_is_retried() {
    let (pushes, attempt, delivered) = failing_merge_push_while_the_remote_moves(
        "ar-explicit-stale-failure",
        true,
        " ! [rejected] main -> main (fetch first)",
    );
    assert_eq!(pushes, 2, "the second merge sees the moved remote");
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(delivered, "the landed commit was published");
}

//region word-level second pass and attribute source
// What: the reconcile merge falls back to the word-level pass for a file Git's line merge leaves in conflict, publishes the
//  merge only when every conflicted file resolves, lists each merged part with the merge commit, and honours the union line
//  of the local tip's `.gitattributes` although the worktree holds no such file.
// Why: the owner decided on 2026-11-03 that the wrapper's unattended merges use the same second pass and log every part.

/// A stored corpus file of the word-level merge.
fn corpus(name: &str, extension: &str) -> Vec<u8> {
    return std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/merge/word_merge_corpus").join(format!("{name}.{extension}")))
        .expect("a corpus file");
}

/// Publish the corpus input's base as `file`,
/// so the local clone and the remote start from it.
fn start_from_corpus(s: &Scenario, input: &str, file: &str) {
    if let Some(parent) = s.other.join(file).parent() {
        std::fs::create_dir_all(parent).expect("directories");
    }
    advance_remote(s, file, String::from_utf8(corpus(input, "base")).expect("UTF-8").as_str());
    git(&s.local, &["pull", "--quiet", "--ff-only"]);
}

/// An edit of a line by the remote and an insertion right after it by the local commit are merged by word and published,
/// and the merge commit's message lists the part.
#[test]
fn an_adjacent_edit_is_merged_by_word_published_and_listed() {
    let s = scenario("ar-word");
    start_from_corpus(&s, "md1b", "notes.md");
    advance_remote(&s, "notes.md", String::from_utf8(corpus("md1b", "ours")).expect("UTF-8").as_str());
    let landed = commit_local(&s, "notes.md", String::from_utf8(corpus("md1b", "theirs")).expect("UTF-8").as_str());
    let (result, err) = run(&s, &landed);
    assert_eq!((result.outcome, result.exit_code), (AutoPushOutcome::Pushed, 0), "{err}");
    let published = text_of(&s.remote, &["show", "main:notes.md"]);
    assert_eq!(published.as_bytes(), String::from_utf8(corpus("md1b", "word.out")).expect("UTF-8").trim_end().as_bytes());
    assert!(err.contains("word-level merge") && err.contains("notes.md"), "{err}");
    let message = text_of(&s.remote, &["log", "--max-count=1", "--format=%B", "main"]);
    assert!(message.contains("word-level merge") && message.contains("notes.md"), "the merge commit lists the part: {message}");
    remove(&s.root);
}

/// Sections appended at one point by both sides stay a conflict:
/// nothing is published and the paths are named.
#[test]
fn a_both_append_conflict_stays_unpublished_after_the_second_pass() {
    let s = scenario("ar-word-append");
    start_from_corpus(&s, "md2", "notes.md");
    advance_remote(&s, "notes.md", String::from_utf8(corpus("md2", "ours")).expect("UTF-8").as_str());
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    let landed = commit_local(&s, "notes.md", String::from_utf8(corpus("md2", "theirs")).expect("UTF-8").as_str());
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before);
    assert!(err.contains("conflicts in: notes.md"), "{err}");
    assert!(!err.contains("word-level merge"), "nothing was merged: {err}");
    remove(&s.root);
}

/// A file with a resolvable region and an unresolvable one blocks the publication of the whole merge,
/// even when another
/// file of the same merge resolves.
#[test]
fn one_unresolved_file_keeps_the_whole_merge_unpublished() {
    let s = scenario("ar-word-one-file");
    start_from_corpus(&s, "md1b", "notes.md");
    advance_remote(&s, "a.txt", "remote\n");
    advance_remote(&s, "notes.md", String::from_utf8(corpus("md1b", "ours")).expect("UTF-8").as_str());
    let remote_before = text_of(&s.remote, &["rev-parse", "main"]);
    std::fs::write(s.local.join("a.txt"), "local\n").expect("a.txt");
    std::fs::write(s.local.join("notes.md"), corpus("md1b", "theirs")).expect("notes.md");
    git(&s.local, &["commit", "--quiet", "--all", "-m", "local both"]);
    let landed = text_of(&s.local, &["rev-parse", "HEAD"]);
    let (result, err) = run(&s, &landed);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before, "nothing is published when any file conflicts");
    assert!(err.contains("conflicts in: a.txt"), "{err}");
    remove(&s.root);
}

/// The union line of the local tip's `.gitattributes` applies although the checkout holds no such file.
#[test]
fn the_union_attribute_of_the_local_tip_applies_without_a_worktree_file() {
    let s = scenario("ar-union");
    std::fs::create_dir_all(s.other.join("doc/handover")).expect("directories");
    std::fs::write(s.other.join(".gitattributes"), "doc/handover/*.md merge=union\n").expect("attributes");
    git(&s.other, &["add", ".gitattributes"]);
    git(&s.other, &["commit", "--quiet", "-m", "attributes"]);
    git(&s.other, &["push", "--quiet", "origin", "main"]);
    git(&s.local, &["pull", "--quiet", "--ff-only"]);
    start_from_corpus(&s, "md2", "doc/handover/log.md");
    advance_remote(&s, "doc/handover/log.md", String::from_utf8(corpus("md2", "ours")).expect("UTF-8").as_str());
    let landed = commit_local(&s, "doc/handover/log.md", String::from_utf8(corpus("md2", "theirs")).expect("UTF-8").as_str());
    // The checkout the merge is asked from does not hold the attributes file (another branch, or a deletion not committed).
    std::fs::remove_file(s.local.join(".gitattributes")).expect("remove the attributes file");
    git(&s.local, &["rm", "--quiet", "--cached", ".gitattributes"]);
    let (result, err) = run(&s, &landed);
    assert_eq!((result.outcome, result.exit_code), (AutoPushOutcome::Pushed, 0), "{err}");
    let published = text_of(&s.remote, &["show", "main:doc/handover/log.md"]);
    assert!(published.contains("Ours risk.") && published.contains("Theirs question."), "{published}");
    remove(&s.root);
}

/// The auto-push runs inside the service and journals under the service's identifier;
/// a push a person typed journals as the wrapper.
#[test]
fn the_journal_identifier_follows_who_runs_the_reconcile() {
    assert_eq!(journal_identifier(false), "monochromatic-git-policy-cli-service");
    assert_eq!(journal_identifier(true), "monochromatic-git-policy-cli");
}
//endregion

/// What:
///  The starts of a reconcile that fetches the moved remote,
/// publishes a merge and fast-forwards the clean worktree.
/// Why:
///   The owner's rule is that no Git start of the wrapper or the service gets `--quiet` or `-q`;
/// the fetch and the
///  fast-forward are the two that carried it (Git 2.56.0:
/// `builtin/merge.c` and the fetch verbosity gate only messages and
///  progress on it).
/// The control looks at both starts,
/// so an empty answer is not an absent run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pushOnce(recordingGit, 'refs/heads/main', landed); expect(startsOf('fetch')).toHaveLength(1); expect(startsOf('merge')).toHaveLength(1); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_fetch_and_the_fast_forward_of_a_reconcile_start_without_quiet() {
    let s = scenario("ar-no-quiet");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let (context, log) = crate::recording_git::recording_context(s.root.as_path());
    let mut git = push_git(&s.local);
    git.real_git = context.real_git.clone();
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&git, "refs/heads/main", &landed, &mut err).expect("auto-push");
    assert_eq!((result.outcome, result.exit_code), (AutoPushOutcome::Pushed, 0), "{}", String::from_utf8_lossy(&err));
    let starts: Vec<String> = crate::recording_git::recorded_starts(log.as_path());
    let named = |word: &str| -> usize { return starts.iter().filter(|start| { return start.split(' ').any(|argument| { return argument == word; }); }).count(); };
    assert_eq!(named("fetch"), 1, "one fetch: {starts:?}");
    // The move of the branch names it: an ancestry check, the two-way update of the checkout and a compare-and-swap ref write.
    assert_eq!(named("read-tree"), 1, "one update of the checkout: {starts:?}");
    let moves: Vec<&String> = starts.iter().filter(|start| { return start.contains("fast-forward refs/heads/main to the published reconcile merge"); }).collect();
    assert_eq!(moves.len(), 1, "one update of the branch, named: {starts:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(&s.root);
}

/// The fetch of a reconcile keeps Git's own words in the service's log,
/// and the move of the branch is in its reflog,
/// which `update-ref --message` writes:
/// `read-tree -u -m` and `update-ref` say nothing of their own.
#[test]
fn what_the_fetch_and_the_fast_forward_say_is_kept_in_the_journal() {
    let s = scenario("ar-words");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let entries = crate::service_timing::capture::entries_of(|| {
        let (result, err) = run(&s, &landed);
        assert_eq!((result.outcome, result.exit_code), (AutoPushOutcome::Pushed, 0), "{err}");
    });
    let said = |command: &str| -> Vec<&crate::service_timing::capture::Captured> {
        return entries.iter().filter(|entry| { return entry.message.contains(format!("git {command}").as_str()) && entry.message.contains(" said: "); }).collect();
    };
    assert_eq!(said("fetch --no-tags").len(), 1, "Git's report of the fetched branch: {entries:?}");
    let reflog: String = text_of(&s.local, &["reflog", "show", "--format=%gs", "refs/heads/main"]);
    assert!(reflog.contains("fast-forward refs/heads/main to the published reconcile merge"), "the move is in the branch's reflog: {reflog}");
    remove(&s.root);
}

/// The reconcile's fast-forward loses the race for the branch inside Git,
/// after Git installed the published merge's paths and
/// before its ref update (a stand-in for Git plays the person's commit,
/// which wins,
/// and then the loser's install,
/// and reports
/// Git's own failure):
/// the paths go back to the winner's content so the checkout matches the branch,
/// the report is the
/// reason and not a failure,
/// and no record is left.
/// Owner's rule:
/// "Auto merging should never fail."
#[test]
fn a_fast_forward_that_loses_the_race_inside_git_leaves_the_checkout_matching_the_branch() {
    let s = scenario("ar-ff-race");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let old = text_of(&s.local, &["rev-parse", "HEAD"]);
    let marker = s.root.join("raced");
    let shim = s.root.join("racing-git");
    // The stand-in plays the person's commit when the ref update of the move runs, so the checkout holds the merge's
    //  paths while the branch is at the winner's commit.
    let script = format!(
        "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = update-ref ] && [ ! -e '{marker}' ]; then\n    : > '{marker}'\n    cd '{local}' || exit 1\n    echo 'committed meanwhile' > z.txt\n    /usr/bin/git add -- z.txt\n    /usr/bin/git commit --quiet --message=meanwhile -- z.txt\n    winner=$(/usr/bin/git rev-parse HEAD)\n    echo \"fatal: update_ref failed for ref 'refs/heads/main': cannot lock ref 'refs/heads/main': is at $winner but expected {old}\" >&2\n    exit 128\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        marker = marker.display(),
        local = s.local.display(),
    );
    std::fs::write(&shim, script).expect("stand-in");
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut racing = push_git(&s.local);
    racing.real_git = shim;
    let result = advance_local_branch(&racing, "refs/heads/main", &old, &target).expect("git");
    assert!(marker.exists(), "the race was played");
    let Err(reason) = result else { panic!("the move lost the race, so it is not reported as done") };
    assert!(reason.contains("Git refused the fast-forward"), "{reason}");
    let winner = text_of(&s.local, &["rev-parse", "HEAD"]);
    assert_eq!(text_of(&s.local, &["log", "-1", "--format=%s", "HEAD"]), "meanwhile");
    assert_ne!(winner, old);
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "", "the checkout matches the branch");
    assert_ne!(std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"), "remote\n", "a.txt is not the published merge's");
    assert!(s.local.join("z.txt").exists());
    assert_eq!(std::fs::read_dir(s.local.join(".git/monochromatic-git-policy-cli-merges")).map_or(0, Iterator::count), 0, "no record is left");
    remove(&s.root);
}

/// A second-pass command whose input thread cannot start is stopped and reported,
/// not left waiting for input and not a panic
/// (`std::thread::spawn` panicked there).
/// The starts are refused to this thread by the seam of `scoped_start`.
#[test]
fn a_second_pass_command_whose_input_thread_cannot_start_is_stopped_and_reported() {
    let root: PathBuf = crate::test_support::fixture("reconcile-no-input-thread");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git: PushGit = push_git(repo.as_path());
    let runner: PushRunner<'_> = PushRunner { git: &git };
    let positive = await_it(runner.run(&["hash-object", "--stdin"], Some(b"x".as_slice()), None)).expect("positive control: the command runs with a thread");
    assert_eq!(positive.code, Some(0));
    let _refused = crate::scoped_start::RefusedStarts::here();
    let reported: String = await_it(runner.run(&["hash-object", "--stdin"], Some(b"x".as_slice()), None)).expect_err("no thread to give it its input");
    assert!(reported.contains("no thread could be started") && reported.contains("hash-object"), "{reported}");
    crate::test_support::remove(root.as_path());
}

/// How long the stand-in of the stalled second pass sleeps:
/// several times the window it is given.
const STALLED_STAND_IN_SLEEP: Duration = Duration::from_secs(8);

/// The stall window of the controls,
/// one whole second,
/// which is the smallest the variable reads.
const STALLED_WINDOW: &str = "1";

/// A second-pass command that shows no sign of work is stopped at the push's stall window and named,
/// and it is not waited out:
/// it is not a person's command,
/// and a `merge-tree` that never returns would hold the push worker,
/// its claim and the drain of that repository for good.
#[test]
fn a_second_pass_command_that_stalls_is_stopped_and_named() {
    let root: PathBuf = crate::test_support::fixture("reconcile-stalled-second-pass");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let shim: PathBuf = root.join("stalled-git");
    std::fs::write(&shim, format!("#!/bin/sh\nexec sleep {}\n", STALLED_STAND_IN_SLEEP.as_secs()).as_str()).expect("stand-in");
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut git: PushGit = push_git(repo.as_path());
    git.real_git = shim;
    git.environment.push((OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE), OsString::from(STALLED_WINDOW)));
    let runner: PushRunner<'_> = PushRunner { git: &git };
    let began: std::time::Instant = std::time::Instant::now();
    let reported: String = await_it(runner.run(&["merge-tree", "--write-tree"], None, None)).expect_err("a stalled command is stopped, not waited out");
    let waited: Duration = began.elapsed();
    assert!(reported.contains("merge-tree"), "{reported}");
    assert!(waited < STALLED_STAND_IN_SLEEP, "the command was stopped at its window: {reported}");
    crate::test_support::remove(root.as_path());
}

/// A `git switch` that lands in the window between the checks and the move moves only the branch this command checked:
/// the move names its branch,
/// so the branch the person switched to is where the person left it,
/// and the checkout that
///  switch made is left showing that branch's commit,
/// never the merge's files staged on a branch that does not hold them.
#[test]
fn a_switch_in_the_window_moves_only_the_branch_this_command_checked() {
    let s = scenario("ar-ff-window-switch");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let tip = text_of(&s.local, &["rev-parse", "refs/heads/main"]);
    git(&s.local, &["branch", "topic"]);
    // The person's `git switch topic` finishes in the window: after the checks, before the move.
    let interposed = SwitchThenMove { local: s.local.clone() };
    let moved: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_moving(&push_git(&s.local), "refs/heads/main", &tip, &target, &interposed)).expect("git");
    assert_eq!(moved, Ok(false), "the branch this command checked follows the merge");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), target);
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/topic"]), tip, "the branch the person switched to is not moved");
    assert_eq!(
        text_of(&s.local, &["status", "--porcelain=v1"]),
        "",
        "the checkout the person switched to holds no staged change of the merge"
    );
    assert_eq!(std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"), "base\n", "it shows the commit its branch names");
    remove(&s.root);
}

/// What:
///  A `git switch` that lands in the window on a branch already at the merge leaves that checkout alone.
/// Why:
///   The branch it switched to already holds the merge's files,
/// so there is nothing of this fast-forward to put back;
/// putting
///  back the branch this command checked would write that branch's tree over the person's branch,
/// staging a reversal of the
///  merge on a branch that still holds it.
#[test]
fn a_switch_in_the_window_to_a_branch_at_the_merge_is_not_put_back() {
    let s = scenario("ar-ff-window-switch-at-merge");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let tip = text_of(&s.local, &["rev-parse", "refs/heads/main"]);
    // The branch the person switches to already points at the merge.
    git(&s.local, &["branch", "topic", target.as_str()]);
    // The switch lands in the instant between the look at the checkout and the write of its files.
    let switching = std::path::PathBuf::from(&s.local);
    let _played = crate::auto_push_reconcile_branch_advance::between_look_and_write_in_a_control(move || {
        git(&switching, &["switch", "--quiet", "topic"]);
    });
    let moved: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_moving(&push_git(&s.local), "refs/heads/main", &tip, &target, &crate::auto_push_reconcile_branch_advance::SHIPPED_MOVE)).expect("git");
    drop(_played);
    assert_eq!(moved, Ok(false), "the branch this command checked follows the merge");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), target);
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/topic"]), target, "the branch the person switched to is not moved");
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "", "its checkout holds no staged reversal of the merge");
    assert_eq!(std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"), "remote\n", "and its files are the merge's");
    remove(&s.root);
}

/// What:
///  The put-back after a switch that landed between the look and the write leaves a staged edit of the person's alone,
///  when it lies on a path the merge does not change.
/// Why:
///   The checkout belongs to another branch,
/// and restoring its whole tree would replace an edit the forward step never
///  touched;
/// only the paths where the merge and the tree it started from differ are this command's to put back.
#[test]
fn a_switch_in_the_window_keeps_a_staged_edit_the_merge_does_not_touch() {
    let s = scenario("ar-ff-window-switch-staged-elsewhere");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let tip = text_of(&s.local, &["rev-parse", "refs/heads/main"]);
    // The branch the person switches to changes only b.txt, which the merge does not touch.
    git(&s.local, &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(s.local.join("b.txt"), "topic\n").expect("the branch's own b.txt");
    git(&s.local, &["commit", "--quiet", "-m", "topic changes b.txt", "--", "b.txt"]);
    git(&s.local, &["switch", "--quiet", "main"]);
    let switching = std::path::PathBuf::from(&s.local);
    // In the window: the switch, then an intentional reversal of b.txt that the person stages.
    let _played = crate::auto_push_reconcile_branch_advance::between_look_and_write_in_a_control(move || {
        git(&switching, &["switch", "--quiet", "topic"]);
        std::fs::write(switching.join("b.txt"), "base\n").expect("the reversal of b.txt");
        git(&switching, &["add", "--", "b.txt"]);
    });
    let moved: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_moving(&push_git(&s.local), "refs/heads/main", &tip, &target, &crate::auto_push_reconcile_branch_advance::SHIPPED_MOVE)).expect("git");
    drop(_played);
    assert_eq!(moved, Ok(false), "the branch this command checked follows the merge");
    assert_eq!(std::fs::read_to_string(s.local.join("b.txt")).expect("b.txt"), "base\n", "the person's staged reversal of b.txt is still there");
    assert_eq!(
        text_of(&s.local, &["diff", "--cached", "--name-only", "--", "b.txt"]),
        "b.txt",
        "and it is still staged"
    );
    assert_eq!(std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"), "base\n", "while the merge's own path is put back");
    remove(&s.root);
}

/// The companion control of the test above:
/// a switch that lands between the look and the write on a branch that does not hold the merge is put back to that
///  branch's own commit,
/// so the rollback still happens where it is needed.
#[test]
fn a_switch_in_the_window_to_another_commit_is_put_back_to_that_commit() {
    let s = scenario("ar-ff-window-switch-elsewhere");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let tip = text_of(&s.local, &["rev-parse", "refs/heads/main"]);
    // The branch the person switches to is this repository's own tip, whose files are not the merge's.
    git(&s.local, &["branch", "topic", tip.as_str()]);
    let switching = std::path::PathBuf::from(&s.local);
    let _played = crate::auto_push_reconcile_branch_advance::between_look_and_write_in_a_control(move || {
        git(&switching, &["switch", "--quiet", "topic"]);
    });
    let moved: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_moving(&push_git(&s.local), "refs/heads/main", &tip, &target, &crate::auto_push_reconcile_branch_advance::SHIPPED_MOVE)).expect("git");
    drop(_played);
    assert_eq!(moved, Ok(false), "the branch this command checked follows the merge");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), target);
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/topic"]), tip, "the branch the person switched to is not moved");
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "", "the merge's files were put back to that branch's own commit");
    assert_eq!(std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"), "base\n", "which is the tree that branch names");
    remove(&s.root);
}

/// The private fetch ref a killed instance left is taken by the next reconcile,
/// and one whose process still runs is left where it is:
/// another worktree of this repository may be reconciling at the same time.
#[test]
fn a_leaked_private_fetch_ref_is_swept_and_a_live_ones_kept() {
    let s = scenario("ar-private-ref-sweep");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    // No process can have this number, so the ref is one a killed instance left.
    let leaked: String = format!("{}4294967294", crate::auto_push_reconcile_basics_and_limits::PRIVATE_FETCH_PREFIX);
    git(&s.local, &["update-ref", leaked.as_str(), target.as_str()]);
    let mut living: std::process::Child = std::process::Command::new("sleep").arg("30").spawn().expect("a live process to name");
    let live_ref: String = format!("{}{}", crate::auto_push_reconcile_basics_and_limits::PRIVATE_FETCH_PREFIX, living.id());
    git(&s.local, &["update-ref", live_ref.as_str(), target.as_str()]);
    crate::auto_push_reconcile_basics_and_limits::sweep_private_fetch_refs(&push_git(&s.local)).expect("the sweep");
    let left: String = text_of(&s.local, &["for-each-ref", "refs/cli-git"]);
    assert!(!left.contains(leaked.as_str()), "the ref of a dead instance is gone: {left}");
    assert!(left.contains(live_ref.as_str()), "the ref of a live process is kept: {left}");
    let _killed = living.kill();
    let _reaped = living.wait();
    remove(&s.root);
}

/// What:
///  The awaited faces of the reconcile's own questions answer what the synchronous faces answer,
/// and a task beside them keeps running while their Git children do.
/// Why:
///   The landing round's reconcile asks these as tasks,
/// and a difference between the faces would relate the branch to
///  another remote tip,
/// sign a merge the configuration does not ask for,
/// or read another directory's scaffolding.
#[test]
fn the_awaited_questions_of_the_reconcile_answer_what_the_synchronous_ones_answer() {
    let scenario: Scenario = scenario("reconcile-awaiting-same-answer");
    advance_remote(&scenario, "a.txt", "remote a\n");
    let context: PushGit = push_git(&scenario.local);
    let behind: String = text_of(&scenario.local, &["rev-parse", "HEAD"]);
    let ahead: String = text_of(&scenario.other, &["rev-parse", "HEAD"]);
    let destination = Destination { remote: String::from("origin"), merge_ref: String::from("refs/heads/main") };
    for (ancestor, descendant) in [(behind.as_str(), ahead.as_str()), (ahead.as_str(), behind.as_str()), (behind.as_str(), behind.as_str())] {
        let synchronous = is_ancestor(&context, ancestor, descendant);
        let (awaited, _ticks): (Result<Option<bool>, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(is_ancestor_awaiting(&context, ancestor, descendant));
        assert_eq!(format!("{awaited:?}"), format!("{synchronous:?}"), "{ancestor} {descendant}");
        let common = common_dir_of(&context);
        let (awaited_common, _common_ticks): (Result<PathBuf, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(common_dir_of_awaiting(&context));
        assert_eq!(format!("{awaited_common:?}"), format!("{common:?}"));
    }
    // Both faces ask the same signing question, and the answer is the same whichever face asks it.
    for value in ["true", "false"] {
        git(&scenario.local, &["config", "commit.gpgsign", value]);
        let synchronous = signs_commits(&context);
        let (awaited, _ticks): (Result<bool, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(signs_commits_awaiting(&context));
        assert_eq!(format!("{awaited:?}"), format!("{synchronous:?}"), "{value}");
    }
    // The same remote tip is fetched by both faces, and each leaves no private ref behind.
    let synchronous = fetch_remote_tip(&context, &destination);
    let (awaited, _ticks): (Result<Result<String, PushAttempt>, SingleFlightError>, u32) =
        crate::ticker_beside_a_wait::with_a_ticker_beside(fetch_remote_tip_awaiting(&context, &destination));
    assert_eq!(format!("{awaited:?}"), format!("{synchronous:?}"));
    let left: String = text_of(&scenario.local, &["for-each-ref", "--format=%(refname)", "refs/cli-git/auto-push/"]);
    assert_eq!(left, "");
    remove(&scenario.root);
}

/// The fast-forward of a pushed branch is a task's work:
/// a caller that is itself a task of the runtime moves the branch and its checkout,
/// and the synchronous face of the same move is refused there,
/// which is why the awaited one exists.
#[test]
fn the_awaited_advance_of_the_branch_moves_it_from_a_task() {
    let s = scenario("ar-ff-awaiting-task");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let tip = text_of(&s.local, &["rev-parse", "refs/heads/main"]);
    // The positive control: the synchronous face inside the runtime is refused, not silently run.
    let refused = crate::lazy_runtime::run_on_runtime(async {
        return crate::auto_push_reconcile_branch_advance::advance_local_branch(
            &push_git(&s.local),
            "refs/heads/main",
            &tip,
            &target,
        );
    })
    .expect("the runtime runs the work");
    let refusal: String = match refused {
        Err(error) => format!("{error}"),
        Ok(moved) => panic!("the synchronous face cannot start Git inside the runtime, and it answered {moved:?}"),
    };
    assert!(refusal.contains("was asked for from inside itself"), "{refusal}");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), tip, "the refused face moved nothing");
    let moved: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_awaiting(
        &push_git(&s.local),
        "refs/heads/main",
        &tip,
        &target,
    ))
    .expect("git");
    assert_eq!(moved, Ok(false), "a clean worktree follows the merge");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), target);
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), target);
    assert_eq!(std::fs::read_to_string(s.local.join("a.txt")).expect("a.txt"), "remote\n");
    assert_eq!(std::fs::read_dir(s.local.join(".git/monochromatic-git-policy-cli-merges")).map_or(0, Iterator::count), 0, "no record is left");
    remove(&s.root);
}

/// A fast-forward that loses the race inside Git from a task leaves the checkout matching the branch,
/// as the same race does on a plain thread:
/// the settle of the record runs on a thread of its own,
/// because the recovery folder's Git starts are synchronous and refused from inside the runtime.
#[test]
fn a_fast_forward_that_loses_the_race_in_a_task_settles_its_record_from_a_thread_of_its_own() {
    let s = scenario("ar-ff-race-in-a-task");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let old = text_of(&s.local, &["rev-parse", "HEAD"]);
    let marker = s.root.join("raced-in-a-task");
    let shim = s.root.join("racing-git-in-a-task");
    let script = format!(
        "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = update-ref ] && [ ! -e '{marker}' ]; then\n    : > '{marker}'\n    cd '{local}' || exit 1\n    echo 'committed meanwhile' > z.txt\n    /usr/bin/git add -- z.txt\n    /usr/bin/git commit --quiet --message=meanwhile -- z.txt\n    winner=$(/usr/bin/git rev-parse HEAD)\n    echo \"fatal: update_ref failed for ref 'refs/heads/main': cannot lock ref 'refs/heads/main': is at $winner but expected {old}\" >&2\n    exit 128\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        marker = marker.display(),
        local = s.local.display(),
    );
    std::fs::write(&shim, script).expect("stand-in");
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut racing = push_git(&s.local);
    racing.real_git = shim;
    let result: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_awaiting(
        &racing,
        "refs/heads/main",
        &old,
        &target,
    ))
    .expect("git");
    assert!(marker.exists(), "the race was played");
    let Err(reason) = result else { panic!("the move lost the race, so it is not reported as done") };
    assert!(reason.contains("Git refused the fast-forward"), "{reason}");
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), text_of(&s.local, &["rev-parse", "refs/heads/main"]));
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "", "the checkout matches the branch");
    assert_eq!(std::fs::read_dir(s.local.join(".git/monochromatic-git-policy-cli-merges")).map_or(0, Iterator::count), 0, "the record was settled");
    remove(&s.root);
}

/// The reconcile of a rejected push is a task's work:
/// the awaited face publishes the merge and moves the branch and its checkout exactly as the synchronous face does,
/// and the synchronous face of the same reconcile is refused from inside the runtime,
/// which is why the awaited one exists.
#[test]
fn the_awaited_reconcile_publishes_and_moves_what_the_synchronous_one_does() {
    let s = scenario("ar-reconcile-awaiting");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let rejected = PushAttempt {
        exit_code: 1,
        output: String::from(" ! [rejected] main -> main (fetch first)"),
    };
    let remote_before: String = text_of(&s.remote, &["rev-parse", "main"]);
    let refused = crate::lazy_runtime::run_on_runtime(async {
        return reconcile_divergent_push(&push_git(&s.local), "refs/heads/main", rejected.clone());
    })
    .expect("the runtime runs the work");
    let refusal: String = match refused {
        Err(error) => format!("{error}"),
        Ok(attempt) => panic!("the synchronous reconcile cannot start Git inside the runtime, and it answered {attempt:?}"),
    };
    assert!(refusal.contains("was asked for from inside itself"), "{refusal}");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before, "the refused face published nothing");
    let attempt: PushAttempt = await_it(crate::auto_push_reconcile_awaiting::reconcile_divergent_push_awaiting(
        &push_git(&s.local),
        "refs/heads/main",
        rejected,
    ))
    .expect("git");
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(attempt.output.contains("was published"), "{}", attempt.output);
    let published: String = text_of(&s.remote, &["rev-parse", "main"]);
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), published, "the branch and its checkout follow the merge");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), published);
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "", "no private ref is left");
    assert_ne!(published, landed, "the published tip is the merge, not the landed commit");
    assert_eq!(
        text_of(&s.local, &["rev-list", "--parents", "-n", "1", published.as_str()]).split(' ').count(),
        3,
        "the published commit is a merge"
    );
    remove(&s.root);
}

/// A fast-forward the ref lock refused once is settled inside the task that asked for it:
/// the recovery finishes the merge,
/// the branch moves,
/// and the record goes,
/// so the task that pushes the person's backup is not left behind the remote by one refused `update-ref`.
/// Why this is the awaited face's work:
/// the settle reads Git,
/// and the synchronous face registers its step's pipes by entering the runtime,
/// which a task cannot do.
/// Planted,
/// by settling through the synchronous face instead,
/// it fails with
/// "the runtime that runs this command's waits was asked for from inside itself"
/// and the record stays.
#[test]
fn a_refused_fast_forward_is_settled_inside_the_task_that_asked_for_it() {
    let s = scenario("ar-ff-settle-in-the-task");
    advance_remote(&s, "a.txt", "remote\n");
    let target = fetched_remote_tip(&s);
    let old = text_of(&s.local, &["rev-parse", "HEAD"]);
    let marker = s.root.join("raced-refused");
    let shim = s.root.join("racing-git-refused");
    let script = format!(
        "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = update-ref ] && [ ! -e '{marker}' ]; then\n    : > '{marker}'\n    echo \"fatal: update_ref failed for ref 'refs/heads/main': cannot lock ref 'refs/heads/main'\" >&2\n    exit 128\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        marker = marker.display(),
    );
    std::fs::write(&shim, script).expect("stand-in");
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut racing = push_git(&s.local);
    racing.real_git = shim;
    let result: Result<bool, String> = await_it(crate::auto_push_reconcile_branch_advance::advance_local_branch_awaiting(
        &racing,
        "refs/heads/main",
        &old,
        &target,
    ))
    .expect("git");
    assert!(result.is_ok(), "the refused move was settled by the recovery, so the advance reports it done: {result:?}");
    assert_eq!(
        std::fs::read_dir(s.local.join(".git/monochromatic-git-policy-cli-merges")).map_or(0, Iterator::count),
        0,
        "the record is settled by the task itself, so the next command is not stopped for the merge"
    );
    assert_ne!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), old, "the branch moved to the settled merge");
    remove(&s.root);
}

/// A merge push a command outside the runtime supplied starts Git synchronously,
/// which is what `gated_merge_push` does for a person's explicit push,
/// so the blocking entry runs it where a start of Git is allowed:
/// on a thread of its own beside the loop,
/// never inside the runtime that runs the loop.
/// Why this is the whole reason for the bridge:
/// the loop is one body and a task's work,
/// and a start of Git from inside the runtime is refused.
/// Planted,
/// by calling the pusher from the loop's own task instead
/// (`Box::pin(std::future::ready(push_merge.push_merge(destination, merge_oid)))`),
/// it fails with "the runtime that runs this command's waits was asked for from inside itself",
/// and nothing is published.
#[test]
fn a_merge_push_that_starts_git_synchronously_runs_beside_the_loop_and_not_inside_it() {
    let s = scenario("ar-blocking-entry-merge-push-thread");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let mut pushed_on: Option<std::thread::ThreadId> = None;
    let mut push_merge = |destination: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        // The synchronous face of a read-only query, which the planner of the person's own push also asks.
        let read = crate::auto_push_single_flight::run_query(
            &push_git(&s.local),
            &argv(&["rev-parse", "--verify", "HEAD^{commit}"]),
        )
        .expect("the merge push of a command outside the runtime may start Git where it runs");
        assert_eq!(read.code, Some(0), "{}", String::from_utf8_lossy(&read.stderr));
        pushed_on = Some(std::thread::current().id());
        git(
            &s.local,
            &["push", "--quiet", "origin", &format!("{merge_oid}:{}", destination.merge_ref)],
        );
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let attempt = finished(integrate(&s, &landed, false, &mut push_merge));
    assert_eq!(attempt.exit_code, 0, "{attempt:?}");
    assert!(attempt.output.contains("was published"), "{}", attempt.output);
    assert_ne!(
        pushed_on,
        Some(std::thread::current().id()),
        "the merge push ran on a thread of its own, never on the thread that waits for the loop"
    );
    let published: String = text_of(&s.remote, &["rev-parse", "main"]);
    assert_ne!(published, landed, "the published tip is the merge, not the landed commit");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), published, "the branch follows the merge");
    assert_eq!(text_of(&s.local, &["rev-parse", "HEAD"]), published, "the checkout follows the merge");
    remove(&s.root);
}

/// A reconcile whose merge push finds no thread reports the rejected attempt and changes nothing:
/// the commit stays saved,
/// the branch and the worktree stay where the failed push left them,
/// and the push's own report is what the person reads.
#[test]
fn a_merge_push_whose_thread_the_system_refuses_leaves_the_rejected_attempt_as_it_was() {
    let s = scenario("ar-merge-push-thread-refused");
    advance_remote(&s, "a.txt", "remote\n");
    let landed = commit_local(&s, "b.txt", "local\n");
    let remote_before: String = text_of(&s.remote, &["rev-parse", "main"]);
    let mut pushes: u32 = 0;
    let mut push_merge = |_destination: &Destination, _merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        pushes += 1;
        return Ok(PushAttempt {
            exit_code: 0,
            output: String::new(),
        });
    };
    let refused = crate::scoped_start::RefusedStarts::here();
    let integrated = integrate(&s, &landed, false, &mut push_merge);
    drop(refused);
    let Integrated::NotDivergent(attempt) = integrated else {
        panic!("a reconcile that could not push reports the rejected attempt, not a finished one");
    };
    assert_eq!(attempt.exit_code, 1, "{attempt:?}");
    assert!(attempt.output.contains("[rejected]"), "{}", attempt.output);
    assert_eq!(pushes, 0, "no merge was pushed");
    assert_eq!(text_of(&s.remote, &["rev-parse", "main"]), remote_before, "nothing was published");
    assert_eq!(text_of(&s.local, &["rev-parse", "refs/heads/main"]), landed, "the branch did not move");
    assert_eq!(text_of(&s.local, &["status", "--porcelain=v1"]), "", "the worktree was not touched");
    assert_eq!(text_of(&s.local, &["for-each-ref", "refs/cli-git"]), "", "no private fetch ref was left");
    remove(&s.root);
}
