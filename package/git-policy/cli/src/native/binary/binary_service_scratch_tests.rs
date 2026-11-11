//! What:
//!  The scratch directory of a commit's preparation,
//!  removed by the service once the landing that read it is over.
//! Why:
//!  The tree holds the private index and the conclusion copies a landing reads,
//!  so nothing may remove it before that landing,
//!  and a command killed while its request is queued must still land on the tree it made.
//!  A tree no landing ever takes is left where it is
//!  and ages out under the rule the sweep and the preparation share
//!  (`scratch_location.rs`,
//! `service_scratch_abandonment.rs`).

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::wrapper_names_old_and_new::SHADOW_POOL_FOLDER;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The longest a control waits for the removal the service starts beside its loop.
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with one tracked file and an edit of it ready to commit.
fn edited_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// A commit's scratch directory is gone once the command that made it has its answer:
/// the service removes it beside its landing loop,
/// bounded,
/// and the command does not wait for that removal to be over.
#[test]
fn a_commit_leaves_no_scratch_directory_once_its_landing_is_over() {
    let fixture: Fixture = fixture("service-scratch-landed");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let output: std::process::Output = wrapped_commit(&fixture, &repo, &["commit", "--quiet", "--message=edit", "--", "a.txt"]).output().expect("the commit runs");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    // The service starts the removal after its landing, so the control waits for it within its own bound.
    let began: Instant = Instant::now();
    loop {
        let left: Vec<String> = super::support::left_behind(repo.as_path());
        if left.is_empty() {
            break;
        }
        assert!(began.elapsed() < PATIENCE, "the tree of a finished commit stayed: {left:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(the_scratch_directory_holds_nothing(repo.as_path()), "the repository scratch directory holds nothing");
    remove_settled(&fixture);
}

/// What:
///  Whether the repository scratch directory of `repo` is gone or holds no entry at all.
/// Why:
///   The names `left_behind` waits for are the trees the wrapper removes,
///  and an entry its two prefixes do not name would end that wait while it is still there:
///  the directory has to be counted,
///  every entry of it,
///  not only the ones a leak would be named by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const empty = (repo) => (await readdir(scratch(repo))).length === 0;
/// ```
fn the_scratch_directory_holds_nothing(repo: &Path) -> bool {
    let Some(directory) = scratch_directory(repo) else { return true };
    return std::fs::read_dir(directory).map(|entries| { return entries.count(); }).unwrap_or(0) == 0;
}

/// The emptiness check the control above ends with is a check:
/// an entry neither prefix names still means the directory holds something,
/// so a directory holding one is not reported empty.
#[test]
fn a_scratch_directory_that_holds_an_entry_the_prefixes_do_not_name_is_not_empty() {
    let fixture: Fixture = fixture("service-scratch-not-empty");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let directory: PathBuf = repo.join(".git/monochromatic-git-policy-cli-scratch");
    std::fs::create_dir_all(directory.as_path()).expect("the repository scratch directory");
    std::fs::write(directory.join("commit.index"), b"a private index").expect("an entry left behind");
    assert!(!the_scratch_directory_holds_nothing(repo.as_path()), "an entry `left_behind` does not name still means the directory holds something");
    remove_settled(&fixture);
}

/// The repository scratch directory of a worktree,
/// when it exists.
fn scratch_directory(repo: &Path) -> Option<PathBuf> {
    let directory: PathBuf = repo.join(".git/monochromatic-git-policy-cli-scratch");
    return directory.is_dir().then_some(directory);
}

/// What:
///  A landed commit leaves the pool of finished shadows beside the scratch directory exactly as it found it.
/// Why:
///  A member is standing state the owner asked for,
///  not scratch
///  ("Pool made ahead (Recommended),
///  We don't need to build a shadow for every commit.
///  Be smarter.",
///  `doc/handover/questions-for-the-owner.md`),
///  so the release of a commit's scratch tree,
///  the sweep the next command runs and the emptiness check above must all leave one alone.
///  This is that check at the user boundary:
///  the scratch directory holds nothing,
///  and the pool's own folder still holds what it held.
#[test]
fn a_landed_commit_leaves_the_pool_of_finished_shadows_alone() {
    let fixture: Fixture = fixture("service-scratch-pool-beside");
    let repo: PathBuf = edited_repository(&fixture);
    let pool: PathBuf = repo.join(".git").join(SHADOW_POOL_FOLDER).join("main");
    std::fs::create_dir_all(pool.as_path()).expect("the pool's folder");
    let standing: PathBuf = pool.join("a-mark-of-standing-state");
    std::fs::write(standing.as_path(), b"a member nobody has claimed yet\n").expect("standing state");
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let output: std::process::Output = wrapped_commit(&fixture, &repo, &["commit", "--quiet", "--message=edit", "--", "a.txt"]).output().expect("the commit runs");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let began: Instant = Instant::now();
    while !the_scratch_directory_holds_nothing(repo.as_path()) {
        assert!(began.elapsed() < PATIENCE, "the scratch directory of a landed commit still holds something");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        std::fs::read(standing.as_path()).expect("the pool's standing state"),
        b"a member nobody has claimed yet\n",
        "the pool beside the scratch directory is untouched"
    );
    remove_settled(&fixture);
}
