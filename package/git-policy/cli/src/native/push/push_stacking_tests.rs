//! Controls for integrating an explicit push that lost a race,
//! against real repositories and bare remotes.
//! The stacked push is published by plain Git here:
//! the gates that the executable adds are exercised in the binary tests.

use super::*;
use crate::test_support::{fixture, git, remove};
use std::os::unix::process::ExitStatusExt;
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
    // The probe's per-user scratch directory is made under a home beside the repository.
    let mut environment = fixture_environment();
    environment.push((OsString::from("HOME"), root.parent().unwrap_or(root).join("home").into_os_string()));
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment,
        lfs: false,
    };
}

/// A fixture:
/// a bare remote,
/// the pushing clone,
/// and the clone another session pushes from.
struct Race {
    /// Directory that holds the fixture,
    /// removed at the end of each test.
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
    git(path, &["config", "user.name", "monochromatic-git-policy-cli push-stacking test"]);
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

/// Publish the merge with plain Git in the clone,
/// as the planner's forwarded push does once the gates pass.
fn plain_git_merge_push(
    git: &PushGit,
    global_prefix: &[OsString],
    tokens: &[OsString],
    destination: &[String],
) -> Result<i32, SingleFlightError> {
    let status = std::process::Command::new("/usr/bin/git")
        .current_dir(&git.cwd)
        .args(global_prefix)
        .arg("push")
        .args(tokens)
        .args(destination)
        .status()
        .map_err(SingleFlightError::Io)?;
    return Ok(status.code().unwrap_or(1));
}

/// Integrate a failed push of `HEAD` from the local clone;
/// plain Git publishes the merge.
fn stack(race: &Race, region: &[OsString]) -> (Option<i32>, String) {
    let git_for_local = push_git(&race.local);
    let request = StackRequest {
        git: &git_for_local,
        global_prefix: &[],
        merge_push: &plain_git_merge_push,
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

/// A push that lost a race is merged with what landed and published,
/// and the branch follows the merge.
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
    assert!(err.contains("monochromatic-git-policy-cli: push: origin/"), "{err}");
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

/// What:
///  An integration that stops on a failed command tells the person what stopped it.
/// Why:
///  The push above has already printed Git's own rejection,
/// so a stop that says nothing leaves the rejection as the whole story
/// and the caller cannot tell why the merge did not happen or what to repair.
/// A signer that fails is the real case:
/// the merge is committed with `-S` when the repository signs commits,
/// and its words are the cause the person acts on.
#[test]
fn an_integration_that_stops_on_a_failed_command_names_it_for_the_person() {
    const SIGNER_WORDS: &str = "SIGNER-FAILURE-THAT-MUST-REACH-THE-PERSON";
    let race = race("ps-integration-stops");
    commit_file(&race.rival, "rival.txt", "rival\n");
    git(&race.rival, &["push", "--quiet", "origin", "HEAD"]);
    commit_file(&race.local, "local.txt", "local\n");
    let before = oid(&race.remote, &race.default);
    let signer: PathBuf = race.root.join("failing-signer");
    crate::test_support::executable(signer.as_path(), format!("#!/bin/sh\necho '{SIGNER_WORDS}' >&2\nexit 1\n").as_bytes());
    git(&race.local, &["config", "commit.gpgsign", "true"]);
    git(&race.local, &["config", "gpg.program", signer.to_str().expect("utf-8")]);
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, None, "the rejection's own status stands: {err}");
    assert!(err.contains("monochromatic-git-policy-cli: push: "), "the stop is not worded as the loop words its lines: {err}");
    assert!(err.contains("did not finish"), "the stop says nothing about what became of the push: {err}");
    assert!(err.contains(SIGNER_WORDS), "the signer's own words are missing: {err}");
    assert_eq!(oid(&race.remote, &race.default), before, "nothing was published");
    remove(&race.root);
}

/// A conflict publishes nothing,
/// names the paths,
/// and exits non-zero.
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

/// A push that failed only because another push of the same commit landed first ends with success,
/// because the remote holds what it sends:
/// the commit's auto-push publishes the branch while the person's own push is on its way.
#[test]
fn a_push_the_remote_already_holds_ends_with_success() {
    let race = race("ps-remote-already-holds");
    // The value the person's remote-tracking ref holds before their push: their push's own update of the
    //  tracking ref is the one that failed, so the file still says this.
    let before = oid(&race.local, "HEAD");
    commit_file(&race.local, "local.txt", "local\n");
    let landed = oid(&race.local, "HEAD");
    // What the commit's auto-push does before the person's push reaches the remote.
    git(&race.local, &["push", "--quiet", "origin", "HEAD"]);
    git(&race.local, &["update-ref", "refs/remotes/origin/main", before.as_str()]);
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, Some(0), "{err}");
    assert!(err.contains("already holds everything this push sends"), "{err}");
    assert_eq!(oid(&race.remote, &race.default), landed, "the remote holds the pushed commit");
    remove(&race.root);
}

/// A push whose branch the remote moved past keeps Git's failure,
/// because the remote does not hold the object it sends.
#[test]
fn a_push_the_remote_moved_past_keeps_its_failure() {
    let race = race("ps-remote-moved-past");
    commit_file(&race.local, "local.txt", "local\n");
    let pushed = oid(&race.local, "HEAD");
    git(&race.local, &["push", "--quiet", "origin", "HEAD"]);
    // Another session commits above the pushed commit, so the remote holds it but stands above it.
    git(&race.rival, &["pull", "--quiet", "--ff-only"]);
    commit_file(&race.rival, "rival.txt", "rival\n");
    git(&race.rival, &["push", "--quiet", "origin", "HEAD"]);
    let (code, err) = stack(&race, &region(&race));
    assert_eq!(code, None, "{err}");
    assert_eq!(err, "");
    assert!(reachable(&race, &pushed, &oid(&race.remote, &race.default)), "the remote holds the pushed commit");
    assert_ne!(oid(&race.remote, &race.default), pushed, "but not as the value this push sends");
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

/// Forced,
/// deleting and multi-ref forms are never integrated.
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

/// The variable that marks the child of the signalled-push control.
const PUSH_SIGNAL_CHILD_VARIABLE: &str = "CLI_GIT_PUSH_SIGNAL_CHILD";

/// What:
///  A push whose Git a signal ended ends the wrapper by that signal.
/// Why:
///  The route keeps a process around Git,
/// reports its death as a value,
/// and that value was turned into the status a
///  shell prints (128 + the signal):
/// with stacking off,
/// and with stacking on and no divergence,
/// that status was returned as
///  the command's,
/// so the caller saw a normal exit with 143 where native Git's caller sees `WIFSIGNALED`
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4 of the round-4
///  review).
/// The body runs in a child,
/// because the death is the assertion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // in a child: runStackedPush(pushThroughAStandInThatKillsItself); expect(parent).toSeeSignal('SIGTERM');
/// ```
#[test]
fn a_push_whose_git_a_signal_ended_keeps_the_signal() {
    if std::env::var_os(PUSH_SIGNAL_CHILD_VARIABLE).is_some() {
        let root: PathBuf = fixture("push-signal");
        let stand_in: PathBuf = root.join("git");
        crate::test_support::executable(stand_in.as_path(), b"#!/bin/sh\nkill -TERM $$\n");
        let forward = PushForward {
            real_git: stand_in,
            arguments: vec![OsString::from("push"), OsString::from("origin"), OsString::from("main")],
            overlay: Vec::new(),
            copy: None,
            subcommand_index: 0,
            environment: fixture_environment(),
            lfs: false,
            lfs_settings: None,
            stack: false,
        };
        let code: i32 = run_stacked_push(&forward);
        // Only reached when the death could not end this process: the parent names that status.
        std::process::exit(code);
    }
    let output: std::process::Output = crate::test_support::child_test_process::run(
        PUSH_SIGNAL_CHILD_VARIABLE,
        "push::push_stacking::tests::a_push_whose_git_a_signal_ended_keeps_the_signal",
    );
    assert_eq!(
        output.status.signal(),
        Some(libc::SIGTERM),
        "the push ended with {:?} instead of the death by signal {}: {}",
        output.status.code(),
        libc::SIGTERM,
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}
