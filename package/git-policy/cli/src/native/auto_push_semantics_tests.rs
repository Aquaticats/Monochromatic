//! Controls that the service never pushes where a plain `git push` of the same branch would not.
//! Each case compares the service's push with the plan real Git (`git push --dry-run --porcelain`) makes for the branch
//! checked out, in disposable repositories with a local bare remote.

use super::*;
use crate::test_support::{fixture, git, git_output, remove};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The branch every case publishes.
const BRANCH: &str = "feature";

/// The full ref name of `BRANCH`.
const BRANCH_REF: &str = "refs/heads/feature";

/// A bare remote and a repository with `feature` checked out and one commit of it not yet published.
struct Rig {
    /// Fixture root, removed when the test ends.
    root: PathBuf,
    /// The bare remote named `origin`.
    remote: PathBuf,
    /// The repository whose branch is published.
    repo: PathBuf,
    /// The commit of `feature` that is not yet published.
    landed: String,
}

/// Environment of every Git child of the service in these fixtures.
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
    environment.extend(crate::test_support::scratch_environment());
    return environment;
}

/// The service's Git for a repository at `root`.
fn push_git(root: &Path) -> PushGit {
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment: fixture_environment(),
        lfs: false,
    };
}

/// Create the remote with `main`, `release` and `other`, and the repository on `feature` with an unpublished commit.
/// `feature` tracks `origin/main`; no `push.default` is set.
fn rig(name: &str) -> Rig {
    let root = fixture(name);
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    git(&root, &["init", "--bare", "--quiet", "--initial-branch=main", "remote.git"]);
    git(&root, &["init", "--quiet", "--initial-branch=main", "repo"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=seed"]);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    git(&repo, &["push", "--quiet", "origin", "main:main", "main:release"]);
    git(&repo, &["fetch", "--quiet", "origin"]);
    git(&repo, &["branch", "--quiet", "other"]);
    git(&repo, &["switch", "--quiet", "--create", BRANCH, "--track", "origin/main"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=feature work"]);
    let landed = String::from_utf8_lossy(&git(&repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    return Rig { root, remote, repo, landed };
}

/// Every ref of the bare remote with its object, so a test can show what a push changed.
fn remote_refs(remote: &Path) -> BTreeMap<String, String> {
    let listing = String::from_utf8_lossy(&git(remote, &["for-each-ref", "--format=%(refname) %(objectname)"]).stdout).into_owned();
    return listing
        .lines()
        .filter_map(|line| {
            return line.split_once(' ').map(|(name, object)| { return (name.to_owned(), object.to_owned()); });
        })
        .collect();
}

/// What a plain `git push` of the checked-out branch would do, according to real Git.
struct GitPlan {
    /// Whether Git refused before updating anything.
    refused: bool,
    /// Remote refs that receive the branch (a `HEAD` source counts as the checked-out branch).
    destinations: BTreeSet<String>,
}

/// Ask real Git what `git push origin` would send for the branch checked out in `repo`, without sending it.
fn git_plan(repo: &Path) -> GitPlan {
    let output = git_output(repo, &["push", "--dry-run", "--porcelain", "origin"]);
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut destinations: BTreeSet<String> = BTreeSet::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let Some((source, destination)) = fields[1].split_once(':') else {
            continue;
        };
        if source == BRANCH_REF || source == "HEAD" {
            destinations.insert(destination.to_owned());
        }
    }
    return GitPlan { refused: !output.status.success(), destinations };
}

/// Run the service's push of `feature` and return the result with the terminal text it printed.
fn service_push(rig: &Rig) -> (AutoPushResult, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(&rig.repo), BRANCH_REF, &rig.landed, &mut err).expect("the push runs");
    return (result, String::from_utf8_lossy(&err).into_owned());
}

/// The remote refs a push changed, with what they hold now.
fn changed_refs(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    return after
        .iter()
        .filter(|(name, object)| {
            return before.get(*name) != Some(*object);
        })
        .map(|(name, object)| {
            return (name.clone(), object.clone());
        })
        .collect();
}

/// Plain `git push` under `push.default=matching` pushes only branches the remote advertises; the remote-tracking ref
/// `origin/feature` outlives the branch's deletion on the remote, so the service must not take it as proof the remote has it.
#[test]
fn a_matching_push_does_not_recreate_a_branch_the_remote_deleted() {
    let mut rig = rig("aps-matching-deleted");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=later work"]);
    rig.landed = String::from_utf8_lossy(&git(&rig.repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    git(&rig.repo, &["config", "push.default", "matching"]);
    git(&rig.remote, &["update-ref", "-d", BRANCH_REF]);
    git(&rig.repo, &["rev-parse", "--verify", "--quiet", "origin/feature"]);
    let plan = git_plan(&rig.repo);
    assert!(plan.destinations.is_empty(), "Git itself does not recreate the branch: {:?}", plan.destinations);
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{err}");
    assert_eq!(result.exit_code, 0, "{err}");
    assert_eq!(remote_refs(&rig.remote), before, "the remote branch stays deleted");
    for needle in [BRANCH, "matching"] {
        assert!(result.shown.contains(needle), "the note names {needle}: {}", result.shown);
    }
    assert!(err.contains(&result.shown), "the note reaches the terminal sink: {err}");
    remove(&rig.root);
}

/// The positive control of the matching case: a branch the remote still has is updated, in place, and nothing else moves.
#[test]
fn a_matching_push_updates_a_branch_the_remote_still_has() {
    let mut rig = rig("aps-matching-present");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=later work"]);
    rig.landed = String::from_utf8_lossy(&git(&rig.repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    git(&rig.repo, &["config", "push.default", "matching"]);
    assert_eq!(git_plan(&rig.repo).destinations, BTreeSet::from([BRANCH_REF.to_owned()]), "Git updates it");
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(changed_refs(&before, &remote_refs(&rig.remote)), BTreeMap::from([(BRANCH_REF.to_owned(), rig.landed.clone())]));
    remove(&rig.root);
}

/// Give `feature` the two upstreams `main` and `release`, as `git config --add` does.
fn track_two_branches(rig: &Rig) {
    git(&rig.repo, &["config", "--add", "branch.feature.merge", "refs/heads/release"]);
}

/// Under `simple` and `upstream` Git refuses a branch with several upstream branches before it updates anything; the first
/// upstream still resolves, which is why the service's upstream query alone does not show the refusal.
#[test]
fn several_upstream_branches_are_refused_where_git_refuses() {
    for mode in ["simple", "upstream", "tracking"] {
        let rig = rig(&format!("aps-two-merges-{mode}"));
        track_two_branches(&rig);
        git(&rig.repo, &["config", "push.default", mode]);
        let upstream = git_output(&rig.repo, &["rev-parse", "--abbrev-ref", "feature@{upstream}"]);
        assert!(upstream.status.success(), "{mode}: the first upstream resolves, so the query alone is not a refusal");
        let plan = git_plan(&rig.repo);
        assert!(plan.refused && plan.destinations.is_empty(), "{mode}: Git refuses");
        let before = remote_refs(&rig.remote);
        let (result, err) = service_push(&rig);
        assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{mode}: {err}");
        assert_eq!(result.exit_code, 0, "{mode}: {err}");
        assert_eq!(remote_refs(&rig.remote), before, "{mode}: neither upstream branch moved");
        for needle in [BRANCH, "branch.feature.merge", mode] {
            assert!(result.shown.contains(needle), "{mode}: the note names {needle}: {}", result.shown);
        }
        remove(&rig.root);
    }
}

/// `current` never asks for the upstream, so several of them are no obstacle: the branch goes to its own name.
#[test]
fn several_upstream_branches_do_not_stop_a_current_push() {
    let rig = rig("aps-two-merges-current");
    track_two_branches(&rig);
    git(&rig.repo, &["config", "push.default", "current"]);
    assert_eq!(git_plan(&rig.repo).destinations, BTreeSet::from([BRANCH_REF.to_owned()]));
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(changed_refs(&before, &remote_refs(&rig.remote)), BTreeMap::from([(BRANCH_REF.to_owned(), rig.landed.clone())]));
    remove(&rig.root);
}

/// One configuration of `remote.origin.push` and whether the service must send exactly what Git does.
struct RefspecCase {
    /// Test-name part.
    name: &'static str,
    /// The values of `remote.origin.push`, in order.
    refspecs: &'static [&'static str],
    /// Whether the service must publish exactly Git's destinations; otherwise it may publish fewer, never more.
    exact: bool,
}

/// The configurations compared with real Git.
const REFSPEC_CASES: &[RefspecCase] = &[
    RefspecCase { name: "two-sources-one-destination", refspecs: &["refs/heads/feature:refs/heads/review", "refs/heads/other:refs/heads/review"], exact: true },
    RefspecCase { name: "a-source-that-does-not-exist", refspecs: &["refs/heads/feature:refs/heads/review", "refs/heads/gone:refs/heads/gone"], exact: true },
    RefspecCase { name: "a-pattern-loses-to-an-explicit-refspec", refspecs: &["refs/heads/*:refs/heads/*", "refs/heads/other:refs/heads/feature"], exact: true },
    RefspecCase { name: "an-explicit-refspec-before-a-pattern", refspecs: &["refs/heads/other:refs/heads/feature", "refs/heads/*:refs/heads/*"], exact: true },
    RefspecCase { name: "two-patterns-first-wins", refspecs: &["refs/heads/f*:refs/heads/x/*", "refs/heads/*:refs/heads/*"], exact: true },
    RefspecCase { name: "one-source-two-destinations", refspecs: &["refs/heads/feature:refs/heads/a", "refs/heads/feature:refs/heads/b"], exact: true },
    RefspecCase { name: "a-negative-refspec-excludes-the-destination", refspecs: &["refs/heads/*:refs/heads/*", "^refs/heads/feature"], exact: true },
    RefspecCase { name: "a-glob-to-another-namespace", refspecs: &["refs/heads/*:refs/heads/mirror/*"], exact: true },
    RefspecCase { name: "a-refspec-without-a-destination", refspecs: &["refs/heads/feature"], exact: true },
    RefspecCase { name: "a-pattern-without-a-remote-side", refspecs: &["refs/heads/*"], exact: true },
    RefspecCase { name: "a-forced-explicit-refspec", refspecs: &["+refs/heads/feature:refs/heads/review"], exact: true },
    RefspecCase { name: "the-checked-out-branch-by-name-head", refspecs: &["HEAD"], exact: false },
    RefspecCase { name: "a-short-source-name", refspecs: &["feature:review"], exact: false },
    RefspecCase { name: "matching-refspec", refspecs: &[":"], exact: false },
];

/// For every configuration the service publishes `feature` to the destinations real Git does and to no other, or says why not.
#[test]
fn a_configured_push_refspec_sends_the_branch_where_git_does() {
    for case in REFSPEC_CASES {
        let rig = rig(&format!("aps-refspec-{}", case.name));
        for refspec in case.refspecs {
            git(&rig.repo, &["config", "--add", "remote.origin.push", refspec]);
        }
        let plan = git_plan(&rig.repo);
        let before = remote_refs(&rig.remote);
        let (result, err) = service_push(&rig);
        let published: BTreeSet<String> = changed_refs(&before, &remote_refs(&rig.remote)).keys().cloned().collect();
        assert!(published.is_subset(&plan.destinations), "{}: published {published:?}, Git sends {:?} (refused: {}): {err}", case.name, plan.destinations, plan.refused);
        for destination in &published {
            assert_eq!(remote_refs(&rig.remote).get(destination), Some(&rig.landed), "{}: {destination}", case.name);
        }
        if case.exact {
            assert_eq!(published, plan.destinations, "{}: {err}", case.name);
        }
        if published.is_empty() {
            assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{}: {err}", case.name);
            assert!(!result.shown.is_empty() && err.contains(&result.shown), "{}: a skip says why: {err}", case.name);
        } else {
            assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{}: {err}", case.name);
        }
        remove(&rig.root);
    }
}

/// A remote that cannot be asked is a failed push with Git's own words, not a skip: the person must learn the branch is
/// unpublished.
#[test]
fn a_matching_push_to_an_unreachable_remote_fails_loudly() {
    let rig = rig("aps-matching-unreachable");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    git(&rig.repo, &["config", "push.default", "matching"]);
    let missing = rig.root.join("missing.git");
    git(&rig.repo, &["config", "remote.origin.url", missing.to_str().expect("utf-8")]);
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_ne!(result.exit_code, 0, "{err}");
    assert!(err.contains(PUSH_FAILED_NOTE), "{err}");
    assert_eq!(remote_refs(&rig.remote), before);
    remove(&rig.root);
}

/// A branch that tracks a local branch has the remote `.`; there is nowhere to push, and the note says so.
#[test]
fn a_branch_that_tracks_a_local_branch_is_skipped_with_a_note() {
    let rig = rig("aps-local-upstream");
    git(&rig.repo, &["branch", "--quiet", "--track", "topic", "main"]);
    let before = remote_refs(&rig.remote);
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(&rig.repo), "refs/heads/topic", &rig.landed, &mut err).expect("the push runs");
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{shown}");
    assert_eq!(result.exit_code, 0, "{shown}");
    assert!(result.shown.contains("topic") && result.shown.contains("no remote"), "{}", result.shown);
    assert!(shown.contains(&result.shown), "{shown}");
    assert_eq!(remote_refs(&rig.remote), before);
    remove(&rig.root);
}

/// Git refuses a branch pushed by name to a remote configured as a mirror ("--mirror can't be combined with refspecs"), and a
/// plain `git push` there mirrors every ref and deletes the remote's others, which the service never does: it skips with a note.
#[test]
fn a_mirror_remote_is_skipped_with_a_note() {
    let rig = rig("aps-mirror");
    git(&rig.repo, &["config", "remote.origin.mirror", "true"]);
    let by_name = git_output(&rig.repo, &["push", "--dry-run", "origin", "refs/heads/feature:refs/heads/feature"]);
    assert!(!by_name.status.success(), "Git refuses a branch pushed by name to a mirror");
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{err}");
    assert_eq!(result.exit_code, 0, "{err}");
    assert!(result.shown.contains("remote.origin.mirror") && result.shown.contains(BRANCH), "{}", result.shown);
    assert_eq!(remote_refs(&rig.remote), before);
    remove(&rig.root);
}
