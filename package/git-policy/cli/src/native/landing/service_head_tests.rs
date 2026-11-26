//! Controls for reading a worktree's head:
//! what a `HEAD` file can and cannot say.

use super::*;

/// A symbolic head names its branch;
/// the text after the prefix,
/// up to the line end,
/// is the reference.
#[test]
fn a_symbolic_head_names_its_branch() {
    assert_eq!(
        interpret(b"ref: refs/heads/main\n"),
        HeadFile::Branch(RefName::from_text("refs/heads/main"))
    );
    assert_eq!(
        interpret(b"ref: refs/heads/a b\r\n"),
        HeadFile::Branch(RefName::from_text("refs/heads/a b"))
    );
}

/// An object name is a detached head.
#[test]
fn an_object_name_is_a_detached_head() {
    assert_eq!(
        interpret(b"0123456789abcdef0123456789abcdef01234567\n"),
        HeadFile::Detached(String::from("0123456789abcdef0123456789abcdef01234567"))
    );
}

/// The reftable placeholder and an empty file say nothing,
/// so Git is asked.
#[test]
fn the_reftable_placeholder_and_an_empty_file_cannot_answer() {
    assert_eq!(
        interpret(format!("ref: {REFTABLE_PLACEHOLDER}\n").as_bytes()),
        HeadFile::Unreadable
    );
    assert_eq!(interpret(b""), HeadFile::Unreadable);
}

/// A directory that is not a Git directory of the repository is not trusted,
/// so a `HEAD` file there is never read.
#[test]
fn a_directory_of_another_repository_is_not_trusted() {
    let root = crate::test_support::fixture("head-foreign");
    let ours = root.join("ours");
    let theirs = root.join("theirs");
    std::fs::create_dir_all(&ours).expect("ours");
    std::fs::create_dir_all(&theirs).expect("theirs");
    assert!(belongs_to(ours.as_path(), ours.as_path()));
    assert!(!belongs_to(theirs.as_path(), ours.as_path()));
    std::fs::write(theirs.join("commondir"), "../ours\n").expect("pointer");
    assert!(
        belongs_to(theirs.as_path(), ours.as_path()),
        "a linked worktree points back to its common directory"
    );
    crate::test_support::remove(root.as_path());
}

/// A `commondir` that is a named pipe names no worktree of the repository,
/// and it answers at once:
/// opening a named pipe for reading waits for a writer that may never come,
/// which held the head question before the file was read as a plain one.
#[test]
fn a_commondir_that_is_a_named_pipe_is_not_a_worktree_of_the_repository() {
    let root: std::path::PathBuf = crate::test_support::fixture("head-commondir-named-pipe");
    let ours: std::path::PathBuf = root.join("ours");
    let linked: std::path::PathBuf = root.join("linked");
    std::fs::create_dir_all(ours.as_path()).expect("the common Git directory");
    std::fs::create_dir_all(linked.as_path()).expect("the linked worktree's Git directory");
    crate::subvolume_sync_attrs::make_fifo(linked.join("commondir").as_path(), 0o600)
        .expect("a named pipe where a commondir goes");
    let inside: std::path::PathBuf = linked.clone();
    let asked = crate::test_support::finished_within(move || {
        return belongs_to(inside.as_path(), ours.as_path());
    });
    let Some(belongs) = asked else {
        panic!("a `commondir` that is a named pipe held the head question")
    };
    assert!(
        !belongs,
        "a directory whose `commondir` is a pipe is no worktree of this repository"
    );
    crate::test_support::remove(root.as_path());
}

/// A `HEAD` that is a named pipe is a head this command cannot read,
/// and it answers at once,
/// so the caller asks Git instead of holding a worker of the runtime:
/// opening a named pipe for reading waits for a writer that may never come.
#[test]
fn a_head_that_is_a_named_pipe_cannot_answer_at_once() {
    let root: std::path::PathBuf = crate::test_support::fixture("head-named-pipe");
    let git_dir: std::path::PathBuf = root.join("git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    crate::subvolume_sync_attrs::make_fifo(git_dir.join("HEAD").as_path(), 0o600)
        .expect("a named pipe where HEAD goes");
    let inside: std::path::PathBuf = git_dir.clone();
    let asked = crate::test_support::finished_within(move || {
        return head_file(inside.as_path(), inside.as_path());
    });
    let Some(answer) = asked else {
        panic!("a `HEAD` that is a named pipe held the head question")
    };
    assert_eq!(
        answer,
        HeadFile::Unreadable,
        "a pipe is no head, so the caller asks Git"
    );
    // The read this module made before, on the same fixture under the same bound, so the bound above is shown to see a wait.
    let head: std::path::PathBuf = git_dir.join("HEAD");
    let waited = crate::test_support::finished_within(move || return std::fs::read(head).ok());
    assert!(
        waited.is_none(),
        "a plain read of the same name waits, so the bound above can see a wait"
    );
    crate::test_support::remove(root.as_path());
}

/// A detached head is what a commit on `HEAD` needs and a branch is what a commit on that branch needs;
/// each is read from the
/// `HEAD` file without starting Git,
/// and the other answer is the opposite.
#[test]
fn the_checked_out_state_is_read_for_a_branch_and_for_a_detached_head() {
    use crate::test_support::{
        fixture,
        git,
        git_context,
        git_text,
        remove,
        repository,
    };
    let root = fixture("head-still-checked-out");
    let repo = repository(root.as_path(), "repo");
    let common = repo.join(".git");
    let context = git_context();
    let ask = |reference: &str| -> Result<bool, FailureCode> {
        return still_checked_out(
            &context,
            repo.as_path(),
            common.as_path(),
            common.as_path(),
            &RefName::from_text(reference),
        );
    };
    assert_eq!(ask("refs/heads/main"), Ok(true));
    assert_eq!(ask("refs/heads/other"), Ok(false));
    assert_eq!(
        ask("HEAD"),
        Ok(false),
        "a branch is checked out, so a commit on HEAD no longer applies"
    );
    let head = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(
        repo.as_path(),
        &["checkout", "--quiet", "--detach", head.as_str()],
    );
    assert_eq!(ask("HEAD"), Ok(true));
    assert_eq!(ask("refs/heads/main"), Ok(false));
    remove(root.as_path());
}

/// A linked worktree whose index file sits in the main worktree's Git directory (an alternate index) still has its own head:
/// the
/// head is read from the Git directory of the worktree that asked,
/// not from the directory the index is in,
/// so the main worktree's
/// branch is never taken for the linked worktree's.
#[test]
fn a_linked_worktrees_head_is_read_from_its_own_git_directory() {
    use crate::test_support::{
        fixture,
        git,
        git_context,
        remove,
        repository,
    };
    let root = fixture("head-linked-alternate");
    let repo = repository(root.as_path(), "repo");
    git(repo.as_path(), &["branch", "linked-branch"]);
    let linked = root.join("linked");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            linked.to_str().expect("path"),
            "linked-branch",
        ],
    );
    let common = repo.join(".git");
    // The linked worktree uses an index file that lives in the main worktree's Git directory.
    let own_git_dir = common.join("worktrees").join("linked");
    let context = git_context();
    assert_eq!(
        still_checked_out(
            &context,
            linked.as_path(),
            own_git_dir.as_path(),
            common.as_path(),
            &crate::ref_name::RefName::from_text("refs/heads/linked-branch")
        ),
        Ok(true)
    );
    assert_eq!(
        still_checked_out(
            &context,
            linked.as_path(),
            own_git_dir.as_path(),
            common.as_path(),
            &crate::ref_name::RefName::from_text("refs/heads/main")
        ),
        Ok(false)
    );
    // The same answer whatever index the request names: the head comes from the worktree's recorded Git directory.
    let _alternate = common.join("alternate-index");
    remove(root.as_path());
}
