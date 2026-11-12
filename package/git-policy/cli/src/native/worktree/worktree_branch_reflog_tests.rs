//! What:
//!  Controls for what a branch's own reflog says about the commits it moved between.
//! Why:
//!   The landed sweep decides with these answers whether a branch that a merge brought in holds work of its own,
//!  so each shape of a real reflog needs a control that pins what it means:
//!  a branch created at a commit,
//!  a branch moved onto a commit that already existed,
//!  and a repository that keeps no reflog at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commonDir = aCommonDirWithReflog('refs/heads/main', lines); expect(branchMovedOnto(commonDir, 'refs/heads/main', tip)).toBe(true);
//! ```

use super::{branch_made_at, branch_moved_onto};
use crate::test_support::fixture;
use std::path::PathBuf;

/// The commit a fixture's reflog is written about,
/// as Git writes it.
const FIRST: &str = "1111111111111111111111111111111111111111";

/// The commit a branch is moved onto,
/// as Git writes it.
const SECOND: &str = "2222222222222222222222222222222222222222";

/// The commit a reflog line names before a reference existed,
/// as Git writes it.
const NO_COMMIT: &str = "0000000000000000000000000000000000000000";

/// The words of every line after the commit fields,
/// which nothing here reads.
const WHO: &str = "A Person <a@example.com> 1794436000 -0500";

/// What:
///  A common Git directory holding one branch's reflog with the lines a control writes.
/// Why:
///  The reader takes the bytes Git writes and nothing else,
///  so a fixture writes the same fields Git writes rather than starting Git for each shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonDir = aCommonDirWithReflog('branch-reflog-moved', ['0000... T1 ...', 'T1 T2 ...']);
/// ```
fn a_common_dir_with_reflog(name: &str, reference: &str, lines: &[String]) -> PathBuf {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join(".git");
    let reflog: PathBuf = common.join("logs").join(reference);
    std::fs::create_dir_all(reflog.parent().expect("the directory of the reflog")).expect("the reflog's directory");
    let text: String = lines.join("\n") + "\n";
    std::fs::write(reflog.as_path(), text.as_bytes()).expect("the reflog");
    return common;
}

/// What:
///  A branch made at a commit records no commit before its first line,
///  so it was created there and nothing says who made that commit.
/// Why:
///  `git worktree add -b task-z` writes exactly this line,
///  and the landed sweep must not read a branch a person created at a tip as the maker of that commit,
///  or every worktree a person just branched from would stay.
#[test]
fn a_branch_created_at_a_commit_made_no_commit_of_its_own() {
    let lines: Vec<String> = vec![format!("{NO_COMMIT} {FIRST} {WHO}\tbranch: Created from HEAD")];
    let common: PathBuf = a_common_dir_with_reflog("branch-reflog-created", "refs/heads/task-z", lines.as_slice());
    assert_eq!(branch_made_at(common.as_path(), "refs/heads/task-z").as_deref(), Some(FIRST), "the oldest line names the commit it was made at");
    assert!(
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", FIRST),
        "a branch created at a commit moved onto nothing"
    );
}

/// What:
///  A branch a person moved onto a commit another branch holds made nothing of its own there.
/// Why:
///  `git reset --hard main` writes exactly this line,
///  and the landed sweep removed the worktree of a branch moved that way
///  because the tip differed from the commit the reflog says the branch was made at
///  (a cross-vendor review of the landed sweep,
///  round 208).
#[test]
fn a_branch_moved_onto_a_commit_made_no_commit_there() {
    let lines: Vec<String> = vec![
        format!("{NO_COMMIT} {FIRST} {WHO}\tbranch: Created from HEAD"),
        format!("{FIRST} {SECOND} {WHO}\treset: moving to main"),
    ];
    let common: PathBuf = a_common_dir_with_reflog("branch-reflog-moved", "refs/heads/task-z", lines.as_slice());
    assert_eq!(branch_made_at(common.as_path(), "refs/heads/task-z").as_deref(), Some(FIRST), "the branch was made at the first commit");
    assert!(
        branch_moved_onto(common.as_path(), "refs/heads/task-z", SECOND),
        "the branch moved onto the second commit from another one"
    );
    assert!(
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", FIRST),
        "no line moves onto the commit the branch was made at"
    );
}

/// What:
///  A line whose two commits are the same says nothing about where the commit came from.
/// Why:
///  Git writes such a line for a reference that is set to the value it already held,
///  and reading it as a move would call a commit the branch made another branch's work.
#[test]
fn a_line_that_moves_nowhere_says_nothing() {
    let lines: Vec<String> = vec![
        format!("{NO_COMMIT} {FIRST} {WHO}\tbranch: Created from HEAD"),
        format!("{SECOND} {SECOND} {WHO}\treset: moving to HEAD"),
    ];
    let common: PathBuf = a_common_dir_with_reflog("branch-reflog-in-place", "refs/heads/task-z", lines.as_slice());
    assert!(
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", SECOND),
        "a line that stayed on its commit proves nothing"
    );
}

/// What:
///  A repository that keeps no reflog for a branch answers nothing about it.
/// Why:
///  Nothing then proves the branch was made anywhere,
///  so the landed sweep examines it as it did before the rule that reads a reflog:
///  `README.md` says so under
///  "The wrapper removes a worktree by itself when the branch it holds lands".
#[test]
fn a_branch_with_no_reflog_answers_nothing() {
    let root: PathBuf = fixture("branch-reflog-absent");
    let common: PathBuf = root.join(".git");
    assert_eq!(branch_made_at(common.as_path(), "refs/heads/task-z"), None, "no reflog names no commit");
    assert!(!branch_moved_onto(common.as_path(), "refs/heads/task-z", FIRST), "no reflog records a move onto any commit");
}
