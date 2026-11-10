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
//!  The one entry of that directory a finished commit does not owe a removal of is the pool of finished shadows the
//!  service fills ahead (`shadow_pool.rs`,
//!  `kept_on_purpose`):
//!  the fill is the service's work,
//!  it stands beside the trees for as long as the worktree has a service,
//!  and a commit that finds no member makes its own shadow.

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, kept_on_purpose, repository};
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
///  Whether the repository scratch directory of `repo` is gone or holds nothing a finished commit left.
/// Why:
///   The names `left_behind` waits for are the trees the wrapper removes,
///  and an entry its two prefixes do not name would end that wait while it is still there:
///  the directory has to be counted,
///  every entry of it,
///  not only the ones a leak would be named by.
///  The pool of shadows the service fills ahead is kept there on purpose (`kept_on_purpose`),
///  so it is the one name this count leaves out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const empty = (repo) => (await readdir(scratch(repo))).filter(keptOnPurpose).length === 0;
/// ```
fn the_scratch_directory_holds_nothing(repo: &Path) -> bool {
    let Some(directory) = scratch_directory(repo) else { return true };
    let Ok(entries) = std::fs::read_dir(directory) else { return true };
    return entries
        .flatten()
        .all(|entry| { return kept_on_purpose(entry.file_name().as_os_str()); });
}

/// The emptiness check the control above ends with is a check:
/// an entry neither prefix names and the service does not keep on purpose still means the directory holds something,
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
