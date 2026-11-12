//! Controls for the awaited Git directory of a worktree:
//! it reads the directory the synchronous read reads,
//! from the files and from Git,
//! and it reads
//!  it inside the runtime,
//! where the synchronous read cannot start Git at all.

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use crate::test_support::{fixture, git, isolated_overlay, remove};

const GIT: &str = "/usr/bin/git";

/// What:
///  The answers one worktree's Git directory gives through both faces,
///  so a control compares them without asking
///  which is which.
/// Why:
///  The awaited face must name what the synchronous one names;
/// a difference here is a commit that reaches
///  another service,
/// or none at all,
/// once the commit path is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [sync, awaited] = bothAnswers(context, directory);
/// ```
fn both_answers(context: &GitContext, directory: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let synchronous: Option<PathBuf> = worktree_git_dir(context, directory);
    let awaited: Option<PathBuf> = crate::lazy_runtime::run_on_runtime(worktree_git_dir_awaiting(context, directory)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  A repository and the context that asks about it through Git,
///  because a global option names the repository and the in-process
///  reader leaves the answer to Git.
/// Why:
///  Both answers of the reader have to be exercised:
/// the files for a plain layout,
/// and Git for a selection the
///  reader cannot honour.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); const context = { ...gitContext, globalPrefix: ['--git-dir', join(repo, '.git')], commonDir: undefined }; return { repo, context };
/// ```
fn a_repository_named_by_a_global_option(name: &str) -> (PathBuf, PathBuf, GitContext) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let context: GitContext = GitContext {
        real_git: PathBuf::from(GIT),
        overlay: isolated_overlay(),
        global_prefix: vec![OsString::from("--git-dir"), repo.join(".git").into_os_string()],
        common_dir: None,
    };
    return (root, repo, context);
}

/// What:
///  Both faces name the same Git directory,
///  from the files of a plain worktree and from Git where the reader
///  leaves the answer to Git,
///  and the awaited face names it inside the runtime,
/// where the synchronous face cannot start Git.
/// Why:
///  The directory names the service instance,
/// so a face that named another directory would hand the batch to another
///  worktree's service,
/// and one that named none would refuse a commit the wrapper could serve.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(...bothAnswers(context, cwd)); assert.equal(bothAnswers(context, cwd)[1], join(repo, '.git'));
/// ```
#[test]
fn the_awaited_git_directory_is_the_one_the_synchronous_face_reads() {
    let (root, repo, context) = a_repository_named_by_a_global_option("route-awaiting-agrees");
    let (synchronous, awaited) = both_answers(&context, repo.as_path());
    assert_eq!(synchronous, Some(repo.join(".git")), "the synchronous face reads the Git directory Git names");
    assert_eq!(awaited, synchronous, "the two faces name one Git directory for the worktree they are asked about");
    // The files answer for a plain layout, where no global option names the repository.
    let plain: GitContext = crate::test_support::git_context();
    let (synchronous_plain, awaited_plain) = both_answers(&plain, repo.as_path());
    assert_eq!(synchronous_plain, Some(repo.join(".git")), "the in-process reader names the plain worktree's Git directory");
    assert_eq!(awaited_plain, synchronous_plain, "the two faces name one Git directory where the files answer");
    remove(root.as_path());
}

/// What:
///  A reading of the layout that gives no answer asks Git,
///  and the directory is Git's rather than none.
/// Why:
///  A task of the runtime may not walk the file system,
/// so the reading is a job of the pool with a ceiling;
/// a job that
///  outlasted its ceiling answered nothing,
/// and nothing is what the files answer when the layout is not one the reader
///  models,
/// so the caller asks Git.
/// A reading nobody answered must not be read as "outside a repository":
/// that
///  refuses a commit the wrapper could serve,
/// with the words of a repository that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using _stated = stateTheAnswer('noAnswer'); assert.equal(await worktreeGitDirAwaiting(plainContext, repo), join(repo, '.git'));
/// ```
#[test]
fn a_layout_reading_that_gives_no_answer_asks_git() {
    let (root, repo, _named) = a_repository_named_by_a_global_option("route-awaiting-no-answer");
    let plain: GitContext = crate::test_support::git_context();
    let _stated = state_the_answer(AStatesAnswer::NoAnswer);
    let awaited: Option<PathBuf> = crate::lazy_runtime::run_on_runtime(worktree_git_dir_awaiting(&plain, repo.as_path())).expect("the runtime");
    assert_eq!(
        awaited,
        Some(repo.join(".git")),
        "a layout reading that gave no answer is answered by Git's own Git directory, not by an absence"
    );
    remove(root.as_path());
}
