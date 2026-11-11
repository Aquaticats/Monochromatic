//! Controls that the service never pushes where a plain `git push` of the same branch would not.
//! Each case compares the service's push with the plan real Git (`git push --dry-run --porcelain`) makes for the branch
//! checked out,
//! in disposable repositories with a local bare remote.

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
    /// Fixture root,
    /// removed when the test ends.
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

/// Create the remote with `main`,
/// `release` and `other`,
/// and the repository on `feature` with an unpublished commit.
/// `feature` tracks `origin/main`;
/// no `push.default` is set.
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

/// Every ref of the bare remote with its object,
/// so a test can show what a push changed.
fn remote_refs(remote: &Path) -> BTreeMap<String, String> {
    let listing = String::from_utf8_lossy(&git(remote, &["for-each-ref", "--format=%(refname) %(objectname)"]).stdout).into_owned();
    return listing
        .lines()
        .filter_map(|line| {
            return line.split_once(' ').map(|(name, object)| { return (name.to_owned(), object.to_owned()); });
        })
        .collect();
}

/// What a plain `git push` of the checked-out branch would do,
/// according to real Git.
struct GitPlan {
    /// Whether Git refused before updating anything.
    refused: bool,
    /// Remote refs that receive the branch (a `HEAD` source counts as the checked-out branch).
    destinations: BTreeSet<String>,
}

/// Ask real Git what `git push origin` would send for the branch checked out in `repo`,
/// without sending it.
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
    return service_push_of(rig, rig.landed.as_str());
}

/// `service_push` for the commit `landed`,
/// which a case that commits again names itself.
fn service_push_of(rig: &Rig, landed: &str) -> (AutoPushResult, String) {
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(&rig.repo), BRANCH_REF, landed, &[], &mut err).expect("the push runs");
    return (result, String::from_utf8_lossy(&err).into_owned());
}

/// The commit `HEAD` names in `repo`.
fn head_of(repo: &Path) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
}

/// Publish `feature` to the remote,
/// so that a matching push carries it,
/// and commit again on it.
/// Gives the commit that is now unpublished.
fn publish_feature_and_commit_again(rig: &Rig) -> String {
    git(&rig.repo, &["push", "--quiet", "origin", &format!("{BRANCH}:{BRANCH}")]);
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=later work"]);
    return head_of(rig.repo.as_path());
}

/// Move `main` on the remote only,
/// so that the local `main` a matching push also carries cannot fast-forward it.
fn move_main_on_the_remote_only(rig: &Rig) {
    git(&rig.repo, &["switch", "--quiet", "--create", "moved", "main"]);
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=moved"]);
    git(&rig.repo, &["push", "--quiet", "origin", "moved:main"]);
    git(&rig.repo, &["switch", "--quiet", BRANCH]);
}

/// A clone of the remote that publishes one commit of its own on `branch`,
/// so the repository's copy of that branch can no longer fast-forward it.
fn move_the_remote_branch_on(rig: &Rig, branch: &str) {
    let elsewhere: PathBuf = rig.root.join(format!("elsewhere-{branch}"));
    git(&rig.root, &["clone", "--quiet", rig.remote.to_str().expect("utf-8"), elsewhere.to_str().expect("utf-8")]);
    git(&elsewhere, &["config", "user.name", "Elsewhere"]);
    git(&elsewhere, &["config", "user.email", "elsewhere@example.invalid"]);
    git(&elsewhere, &["switch", "--quiet", "--create", branch, "--track", &format!("origin/{branch}")]);
    git(&elsewhere, &["commit", "--quiet", "--allow-empty", "--message=elsewhere"]);
    git(&elsewhere, &["push", "--quiet", "origin", branch]);
}

/// The remote refs a push changed,
/// with what they hold now.
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

/// Give `feature` the two upstreams `main` and `release`,
/// as `git config --add` does.
fn track_two_branches(rig: &Rig) {
    git(&rig.repo, &["config", "--add", "branch.feature.merge", "refs/heads/release"]);
}

/// Under `simple` and `upstream` Git refuses a branch with several upstream branches before it updates anything;
/// the first
/// upstream still resolves,
/// which is why the service's upstream query alone does not show the refusal.
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

/// `current` never asks for the upstream,
/// so several of them are no obstacle:
/// the branch goes to its own name.
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
    /// The values of `remote.origin.push`,
    /// in order.
    refspecs: &'static [&'static str],
    /// Whether the service must publish exactly Git's destinations;
    /// otherwise it may publish fewer,
    /// never more.
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

/// What:
///  The service publishes `feature` to the destinations real Git does for the configuration of `case`,
///  and to no other,
/// or says why not.
/// Why:
///  One control per configuration:
/// each one builds its own rig and starts real Git several times,
/// which took the whole table 83.0 s in one
///  control on 2026-11-09 while the host ran at a load average of 76 and the control timed out at the runner's
///  five-minute limit inside a suite.
/// The table itself stays one list,
/// so no case can be forgotten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const case of cases) test(case.name, () => oneRefspecConfiguration(case));
/// ```
fn a_refspec_configuration_is_sent_where_git_sends_it(case: &RefspecCase) {
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

/// The case of `REFSPEC_CASES` called `name`.
fn case(name: &str) -> &'static RefspecCase {
    return REFSPEC_CASES.iter().find(|case| { return case.name == name; }).unwrap_or_else(|| { panic!("no refspec case called {name}") });
}


/// Two sources for one destination:
/// Git sends the last one,
/// and the service sends that one and no other.
#[test]
fn a_refspec_two_sources_one_destination_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("two-sources-one-destination"));
}


/// A source that does not exist is refused by Git and sent by nobody,
/// while the source beside it is sent.
#[test]
fn a_refspec_a_source_that_does_not_exist_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-source-that-does-not-exist"));
}


/// A pattern loses to an explicit refspec that names the destination.
#[test]
fn a_refspec_a_pattern_loses_to_an_explicit_refspec_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-pattern-loses-to-an-explicit-refspec"));
}


/// An explicit refspec before a pattern decides the destination,
/// whatever the order suggests.
#[test]
fn a_refspec_an_explicit_refspec_before_a_pattern_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("an-explicit-refspec-before-a-pattern"));
}


/// Of two patterns the first decides,
/// as it does for Git.
#[test]
fn a_refspec_two_patterns_first_wins_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("two-patterns-first-wins"));
}


/// One source with two destinations goes to both.
#[test]
fn a_refspec_one_source_two_destinations_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("one-source-two-destinations"));
}


/// A negative refspec excludes the destination it names.
#[test]
fn a_refspec_a_negative_refspec_excludes_the_destination_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-negative-refspec-excludes-the-destination"));
}


/// A glob to another namespace sends the branch there and nowhere else.
#[test]
fn a_refspec_a_glob_to_another_namespace_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-glob-to-another-namespace"));
}


/// A refspec without a destination is a configuration Git refuses,
/// so nothing is sent and the person is told why.
#[test]
fn a_refspec_a_refspec_without_a_destination_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-refspec-without-a-destination"));
}


/// A pattern without a remote side sends nothing,
/// as Git sends nothing.
#[test]
fn a_refspec_a_pattern_without_a_remote_side_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-pattern-without-a-remote-side"));
}


/// A forced explicit refspec sends the branch where it names.
#[test]
fn a_refspec_a_forced_explicit_refspec_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-forced-explicit-refspec"));
}


/// The checked-out branch by the name HEAD is sent where Git sends it.
#[test]
fn a_refspec_the_checked_out_branch_by_name_head_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("the-checked-out-branch-by-name-head"));
}


/// A short source name is sent where Git sends it.
#[test]
fn a_refspec_a_short_source_name_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("a-short-source-name"));
}


/// The matching refspec sends the branch,
/// as Git's matching push does.
#[test]
fn a_refspec_matching_refspec_is_sent_where_git_sends_it() {
    a_refspec_configuration_is_sent_where_git_sends_it(case("matching-refspec"));
}

/// A remote that cannot be asked is a failed push with Git's own words,
/// not a skip:
/// the person must learn the branch is
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

/// A branch that tracks a local branch has the remote `.`;
/// there is nowhere to push,
/// and the note says so.
#[test]
fn a_branch_that_tracks_a_local_branch_is_skipped_with_a_note() {
    let rig = rig("aps-local-upstream");
    git(&rig.repo, &["branch", "--quiet", "--track", "topic", "main"]);
    let before = remote_refs(&rig.remote);
    let mut err: Vec<u8> = Vec::new();
    let result = push_once(&push_git(&rig.repo), "refs/heads/topic", &rig.landed, &[], &mut err).expect("the push runs");
    let shown = String::from_utf8_lossy(&err).into_owned();
    assert_eq!(result.outcome, AutoPushOutcome::Skipped, "{shown}");
    assert_eq!(result.exit_code, 0, "{shown}");
    assert!(result.shown.contains("topic") && result.shown.contains("no remote"), "{}", result.shown);
    assert!(shown.contains(&result.shown), "{shown}");
    assert_eq!(remote_refs(&rig.remote), before);
    remove(&rig.root);
}

/// Git refuses a branch pushed by name to a remote configured as a mirror ("--mirror can't be combined with refspecs"),
/// and a
/// plain `git push` there mirrors every ref and deletes the remote's others,
/// which the service never does:
/// it skips with a note.
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

/// Give the local `other` branch a commit that the remote lacks,
/// so a plain matching push has a second branch to send.
fn advance_other(rig: &Rig) -> String {
    git(&rig.repo, &["push", "--quiet", "origin", "other:other"]);
    git(&rig.repo, &["switch", "--quiet", "other"]);
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=other work"]);
    let tip = String::from_utf8_lossy(&git(&rig.repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    git(&rig.repo, &["switch", "--quiet", BRANCH]);
    return tip;
}

/// Under `push.default=matching` the service runs the plain `git push <remote>` and lets Git match:
/// every local branch the
/// remote also has is sent,
/// the recorded branch and its siblings alike,
/// exactly as Git's own dry run lists them.
#[test]
fn a_matching_push_sends_every_branch_the_remote_has() {
    let mut rig = rig("aps-matching-siblings");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=later work"]);
    rig.landed = String::from_utf8_lossy(&git(&rig.repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    let other_tip = advance_other(&rig);
    git(&rig.repo, &["config", "push.default", "matching"]);
    let planned = git_output(&rig.repo, &["push", "--dry-run", "--porcelain", "origin"]);
    let planned_text = String::from_utf8_lossy(&planned.stdout).into_owned();
    assert!(planned_text.contains("refs/heads/other:refs/heads/other") && planned_text.contains("refs/heads/feature:refs/heads/feature"), "{planned_text}");
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(
        changed_refs(&before, &remote_refs(&rig.remote)),
        BTreeMap::from([(BRANCH_REF.to_owned(), rig.landed.clone()), ("refs/heads/other".to_owned(), other_tip)])
    );
    remove(&rig.root);
}

/// Git's matching skips a branch the remote lacks,
/// so a branch deleted on the remote stays deleted;
/// the owner accepts what
/// Git does,
/// and the push of the branches the remote still has succeeds.
#[test]
fn a_matching_push_leaves_a_branch_the_remote_deleted_as_git_does() {
    let rig = rig("aps-matching-deleted");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    git(&rig.remote, &["update-ref", "-d", BRANCH_REF]);
    git(&rig.repo, &["config", "push.default", "matching"]);
    assert!(git_plan(&rig.repo).destinations.is_empty(), "Git itself does not send the branch");
    let before = remote_refs(&rig.remote);
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_refs(&rig.remote), before);
    remove(&rig.root);
}

/// What:
///  One matching push carries every branch the remote has,
/// so Git's exit status is the worst of them:
/// a sibling the remote refused leaves the recorded branch published and the status non-zero.
/// Why:
///  The repository is asked whether the recorded branch arrived,
/// and its answer decides the branch's fate,
///  so a published branch is not reported as a push that may not have arrived,
/// nor retried every ten seconds until it does.
///  Git's lines about the sibling stay in the report.
#[test]
fn a_matching_push_whose_sibling_the_remote_refused_still_publishes_the_branch() {
    let rig = rig("aps-matching-sibling-refused");
    let landed: String = publish_feature_and_commit_again(&rig);
    move_main_on_the_remote_only(&rig);
    git(&rig.repo, &["config", "push.default", "matching"]);
    let before: BTreeMap<String, String> = remote_refs(&rig.remote);
    let (result, err) = service_push_of(&rig, landed.as_str());
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(result.exit_code, 0, "{err}");
    assert_eq!(remote_refs(&rig.remote).get(BRANCH_REF), Some(&landed), "the recorded branch is published");
    assert!(result.shown.contains("rejected"), "Git's line about the sibling is kept: {}", result.shown);
    assert_eq!(remote_refs(&rig.remote).get("refs/heads/main"), before.get("refs/heads/main"), "the sibling is not moved by this push");
    remove(&rig.root);
}

/// A matching push the remote refused for the recorded branch itself is the branch's failure,
/// with Git's words and the note that says the remote may or may not have taken it.
///  This is the answer a push whose branch did not arrive keeps,
/// so the retry that publishes it is planned.
#[test]
fn a_matching_push_the_remote_refused_for_the_recorded_branch_is_a_failure() {
    let rig = rig("aps-matching-recorded-refused");
    let landed: String = publish_feature_and_commit_again(&rig);
    move_the_remote_branch_on(&rig, BRANCH);
    git(&rig.repo, &["config", "push.default", "matching"]);
    let (result, err) = service_push_of(&rig, landed.as_str());
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert_ne!(result.exit_code, 0, "{err}");
    assert!(err.contains(PUSH_FAILED_NOTE), "{err}");
    assert_ne!(remote_refs(&rig.remote).get(BRANCH_REF), Some(&landed), "the branch did not arrive");
    remove(&rig.root);
}

/// With no branch in common Git does nothing and says so with a failure,
/// which the service reports as it is.
#[test]
fn a_matching_push_with_no_branch_in_common_fails_with_gits_words() {
    let rig = rig("aps-matching-nothing");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    git(&rig.repo, &["config", "push.default", "matching"]);
    for name in ["main", "release", BRANCH] {
        git(&rig.remote, &["update-ref", "-d", &format!("refs/heads/{name}")]);
    }
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Failed, "{err}");
    assert!(result.shown.contains("No refs in common"), "{}", result.shown);
    remove(&rig.root);
}

/// A second bare repository beside the rig's remote,
/// with `main` and `feature` pushed to it,
/// to serve as a push URL.
fn second_remote(rig: &Rig, name: &str) -> PathBuf {
    let path = rig.root.join(name);
    git(&rig.root, &["init", "--bare", "--quiet", "--initial-branch=main", name]);
    git(&rig.repo, &["push", "--quiet", path.to_str().expect("utf-8"), "main:main", "feature:feature"]);
    return path;
}

/// Plain `git push` follows the remote's push URL,
/// so under `matching` the branches of the push URL are updated and the
/// fetch URL is not touched.
#[test]
fn a_matching_push_goes_to_the_push_url() {
    let mut rig = rig("aps-matching-pushurl");
    git(&rig.repo, &["push", "--quiet", "--set-upstream", "origin", BRANCH]);
    let published = second_remote(&rig, "published.git");
    git(&rig.repo, &["commit", "--quiet", "--allow-empty", "--message=later work"]);
    rig.landed = String::from_utf8_lossy(&git(&rig.repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    git(&rig.repo, &["config", "push.default", "matching"]);
    git(&rig.repo, &["config", "remote.origin.pushurl", published.to_str().expect("utf-8")]);
    let (before_fetch_side, before_push_side) = (remote_refs(&rig.remote), remote_refs(&published));
    let (result, err) = service_push(&rig);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{err}");
    assert_eq!(remote_refs(&rig.remote), before_fetch_side, "the fetch URL is untouched");
    assert_eq!(changed_refs(&before_push_side, &remote_refs(&published)), BTreeMap::from([(BRANCH_REF.to_owned(), rig.landed.clone())]));
    remove(&rig.root);
}
