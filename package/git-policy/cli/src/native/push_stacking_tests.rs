//! Controls for integrating an explicit push that lost a race, against real repositories and bare remotes.
//! The stacked push is published by plain Git here: the gates that the executable adds are exercised in the binary tests.

use super::*;
use crate::test_support::{fixture, git, remove};
use std::path::{Path, PathBuf};

/// Environment of every Git child in these fixtures.
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

/// A fixture: a bare remote, the pushing clone, and the clone another session pushes from.
struct Race {
    /// Directory that holds the fixture, removed at the end of each test.
    root: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The clone whose push loses the race.
    local: PathBuf,
    /// The clone that wins it.
    rival: PathBuf,
    /// The remote's default branch.
    default: String,
}

/// Write `text` to `name` in `repo` and commit it.
fn commit_file(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("fixture file");
    git(repo, &["add", name]);
    git(repo, &["commit", "--quiet", "-m", name, name]);
}

/// Clone the remote into `path` with an identity.
fn clone_of(root: &Path, remote: &Path, path: &Path) {
    git(
        root,
        &[
            "clone",
            "--quiet",
            remote.to_str().expect("utf-8"),
            path.to_str().expect("utf-8"),
        ],
    );
    git(path, &["config", "user.email", "cli-git@example.invalid"]);
    git(path, &["config", "user.name", "cli-git push-stacking test"]);
}

/// Build the fixture with one commit on the remote and no local change yet.
fn race(name: &str) -> Race {
    let root = fixture(name);
    let remote = root.join("remote.git");
    git(&root, &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    let seed = root.join("seed");
    clone_of(&root, &remote, &seed);
    commit_file(&seed, "base.txt", "base\n");
    let default = String::from_utf8_lossy(&git(&seed, &["branch", "--show-current"]).stdout)
        .trim()
        .to_owned();
    git(&seed, &["push", "--quiet", "origin", "HEAD"]);
    let local = root.join("local");
    let rival = root.join("rival");
    clone_of(&root, &remote, &local);
    clone_of(&root, &remote, &rival);
    return Race {
        root,
        remote,
        local,
        rival,
        default,
    };
}

/// The commit a ref names in a repository.
fn oid(repo: &Path, name: &str) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", name]).stdout)
        .trim()
        .to_owned();
}

/// The arguments after `push` for `git push origin HEAD:<default>`.
fn region(race: &Race) -> Vec<OsString> {
    return ["origin".to_owned(), format!("HEAD:{}", race.default)]
        .iter()
        .map(OsString::from)
        .collect();
}

/// Integrate a failed push of `HEAD` from the local clone; plain Git publishes the merge.
fn stack(race: &Race, region: &[OsString]) -> (Option<i32>, String) {
    let git_for_local = push_git(&race.local);
    let request = StackRequest {
        git: &git_for_local,
        own_executable: Path::new("/usr/bin/git"),
        global_prefix: &[],
        region,
    };
    let mut err: Vec<u8> = Vec::new();
    let code = stack_failed_push(&request, 1, &mut err);
    return (code, String::from_utf8_lossy(&err).into_owned());
}

/// Whether `ancestor` is reachable from `descendant` in the remote.
fn reachable(race: &Race, ancestor: &str, descendant: &str) -> bool {
    return std::process::Command::new("/usr/bin/git")
        .current_dir(&race.remote)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .status()
        .expect("git")
        .success();
}

/// A push that lost a race is merged with what landed and published, and the branch follows the merge.
#[test]
fn a_push_that_lost_a_race_publishes_the_merge_and_the_branch_follows() {
    let race = race("ps-lost-race");
    commit_file(&race.rival, "rival.txt", "rival\n");
    git(&race.rival, &["push", "--quiet", "origin", "HEAD"]);
    commit_file(&race.local, "local.txt", "local\n");
    let landed = oid(&race.local, "HEAD");
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, Some(0), "{err}");
    let published = oid(&race.remote, &race.default);
    assert!(reachable(&race, &landed, &published), "the local commit is published");
    assert!(
        reachable(&race, &oid(&race.rival, "HEAD"), &published),
        "the rival commit is kept"
    );
    assert!(err.contains("cli-git: push: origin/"), "{err}");
    assert_eq!(oid(&race.local, "HEAD"), published, "the branch follows the merge");
    remove(&race.root);
}

/// A detached HEAD stays where it is and the report says so.
#[test]
fn a_detached_head_stays_where_it_is_and_the_report_says_so() {
    let race = race("ps-detached");
    commit_file(&race.rival, "rival.txt", "rival\n");
    git(&race.rival, &["push", "--quiet", "origin", "HEAD"]);
    git(&race.local, &["checkout", "--quiet", "--detach"]);
    commit_file(&race.local, "local.txt", "local\n");
    let landed = oid(&race.local, "HEAD");
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(oid(&race.local, "HEAD"), landed, "HEAD is not moved");
    assert!(err.contains("HEAD is detached"), "{err}");
    assert!(reachable(&race, &landed, &oid(&race.remote, &race.default)));
    remove(&race.root);
}

/// A conflict publishes nothing, names the paths, and exits non-zero.
#[test]
fn a_conflict_publishes_nothing_and_names_the_paths() {
    let race = race("ps-conflict");
    commit_file(&race.rival, "base.txt", "rival edit\n");
    git(&race.rival, &["push", "--quiet", "origin", "HEAD"]);
    let before = oid(&race.remote, &race.default);
    commit_file(&race.local, "base.txt", "local edit\n");
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, Some(1), "{err}");
    assert!(err.contains("conflicts in: base.txt"), "{err}");
    assert_eq!(oid(&race.remote, &race.default), before);
    remove(&race.root);
}

/// A failure that is not a divergence of the destination is left exactly as the push reported it.
#[test]
fn a_failure_that_is_not_a_divergence_prints_and_changes_nothing() {
    let race = race("ps-not-divergent");
    commit_file(&race.local, "local.txt", "local\n");
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, None);
    assert_eq!(err, "");
    remove(&race.root);
}

/// A remote that cannot be reached leaves the push's own report as the only one.
#[test]
fn an_unreachable_remote_prints_and_changes_nothing() {
    let race = race("ps-unreachable");
    commit_file(&race.local, "local.txt", "local\n");
    git(
        &race.local,
        &["remote", "set-url", "origin", race.root.join("missing.git").to_str().expect("utf-8")],
    );
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, None);
    assert_eq!(err, "");
    remove(&race.root);
}

/// Forced, deleting and multi-ref forms are never integrated.
#[test]
fn forced_deleting_and_multi_ref_pushes_are_never_integrated() {
    let race = race("ps-ineligible");
    commit_file(&race.rival, "rival.txt", "rival\n");
    git(&race.rival, &["push", "--quiet", "origin", "HEAD"]);
    commit_file(&race.local, "local.txt", "local\n");
    let before = oid(&race.remote, &race.default);
    for arguments in [
        vec!["--force", "origin", "HEAD:{default}"],
        vec!["origin", "+HEAD:{default}"],
        vec!["--delete", "origin", "{default}"],
        vec!["--all", "origin"],
        vec!["--dry-run", "origin", "HEAD:{default}"],
    ] {
        let region: Vec<OsString> = arguments
            .iter()
            .map(|token| {
                return OsString::from(token.replace("{default}", &race.default));
            })
            .collect();
        let (code, err) = stack(&race, &region);
        assert_eq!(code, None, "{arguments:?}: {err}");
        assert_eq!(err, "", "{arguments:?}");
    }
    assert_eq!(oid(&race.remote, &race.default), before);
    remove(&race.root);
}
