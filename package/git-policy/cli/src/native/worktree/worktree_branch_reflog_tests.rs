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

use super::{MergeMove, branch_made_at, branch_moved_onto};
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

/// The commit a SHA-256 repository records before a branch's first reflog line,
/// which is sixty-four zeros rather than the forty of SHA-1.
const SHA256_NO_COMMIT: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The commit a SHA-256 repository names,
/// as Git writes it.
const SHA256_FIRST: &str = "1111111111111111111111111111111111111111111111111111111111111111";

/// The words of every line after the commit fields,
/// which nothing here reads.
const WHO: &str = "A Person <a@example.com> 1794436000 -0500";

/// How long an answer that must not wait may take in a control.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &std::path::Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

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
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", FIRST, None),
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
        branch_moved_onto(common.as_path(), "refs/heads/task-z", SECOND, None),
        "the branch moved onto the second commit from another one"
    );
    assert!(
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", FIRST, None),
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
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", SECOND, None),
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
    assert!(!branch_moved_onto(common.as_path(), "refs/heads/task-z", FIRST, None), "no reflog records a move onto any commit");
}

/// What:
///  A branch Git created in a SHA-256 repository records the null object name of that repository,
///  which is sixty-four zeros,
///  and moved onto nothing.
/// Why:
///  Git writes the null id of the repository's own hash width before a branch's first reflog line,
///  and this reader compared it against the forty zeros of SHA-1.
///  In a SHA-256 repository every branch merely created at a commit then read as one that moved onto it,
///  so the landed sweep kept every worktree whose tip another branch was created at,
///  which is the state the wrapper's own `git worktree add -b` makes on every task
///  (a cross-vendor review of the landed sweep,
///  round 208).
#[test]
fn a_branch_created_in_a_sha256_repository_moved_onto_nothing() {
    let lines: Vec<String> = vec![format!("{SHA256_NO_COMMIT} {SHA256_FIRST} {WHO}\tbranch: Created from HEAD")];
    let common: PathBuf = a_common_dir_with_reflog("branch-reflog-sha256-created", "refs/heads/task-z", lines.as_slice());
    assert_eq!(
        branch_made_at(common.as_path(), "refs/heads/task-z").as_deref(),
        Some(SHA256_FIRST),
        "the oldest line names the commit it was made at"
    );
    assert!(
        !branch_moved_onto(common.as_path(), "refs/heads/task-z", SHA256_FIRST, None),
        "a branch created at a commit in a SHA-256 repository moved onto nothing"
    );
}

/// What:
///  A reflog whose name holds a named pipe answers at once.
/// Why:
///  A reflog was read with an ordinary blocking open,
///  which waits for a writer that never comes when the name holds a pipe.
///  The sweep reads a reflog inside the person's `git merge`,
///  after the merge commit exists,
///  so the whole command hung with nothing said and no end
///  (a cross-vendor review of the landed sweep,
///  round 208).
#[test]
fn a_reflog_that_is_a_named_pipe_answers_at_once() {
    let root: PathBuf = fixture("branch-reflog-fifo");
    let common: PathBuf = root.join(".git");
    let reflog: PathBuf = common.join("logs").join("refs/heads/task-z");
    std::fs::create_dir_all(reflog.parent().expect("the directory of the reflog")).expect("the reflog's directory");
    make_pipe(reflog.as_path());
    let (told, answers) = std::sync::mpsc::channel();
    let asked: PathBuf = common.clone();
    std::thread::spawn(move || {
        let made: Option<String> = branch_made_at(asked.as_path(), "refs/heads/task-z");
        let moved: bool = branch_moved_onto(asked.as_path(), "refs/heads/task-z", FIRST, None);
        let _sent = told.send((made, moved));
    });
    let (made, moved) = answers.recv_timeout(PATIENCE).expect("a reflog that is a named pipe answers without waiting for a writer");
    assert_eq!(made, None, "a named pipe names no commit");
    assert!(!moved, "a named pipe records no move onto any commit");
}

/// What:
///  A reflog whose name is a link answers nothing,
///  and answers at once.
/// Why:
///  A link planted at a reflog's name can point at a device that never ends (`/dev/zero`),
///  and a read that followed it would never finish while holding the person's merge:
///  the reader takes a plain file at the name itself,
///  as every other read of a file beside a checkout does
///  (`plain_file.rs`).
#[test]
fn a_reflog_reached_through_a_link_answers_nothing() {
    let root: PathBuf = fixture("branch-reflog-link");
    let common: PathBuf = root.join(".git");
    let reflog: PathBuf = common.join("logs").join("refs/heads/task-z");
    std::fs::create_dir_all(reflog.parent().expect("the directory of the reflog")).expect("the reflog's directory");
    std::os::unix::fs::symlink("/dev/zero", reflog.as_path()).expect("a link where the reflog goes");
    let (told, answers) = std::sync::mpsc::channel();
    let asked: PathBuf = common.clone();
    std::thread::spawn(move || {
        let made: Option<String> = branch_made_at(asked.as_path(), "refs/heads/task-z");
        let moved: bool = branch_moved_onto(asked.as_path(), "refs/heads/task-z", FIRST, None);
        let _sent = told.send((made, moved));
    });
    let (made, moved) = answers.recv_timeout(PATIENCE).expect("a reflog a link names answers without reading through it");
    assert_eq!(made, None, "a link names no commit");
    assert!(!moved, "a link records no move onto any commit");
}

/// What:
///  The move a merge makes in the branch it ran in is no witness,
///  while any other move of that branch still is.
/// Why:
///  A fast-forward leaves the branch the merge ran in on the tip the landed branch has,
///  and Git records that move in its reflog like any other.
///  Reading it as a witness kept every landed worktree in a fast-forward,
///  and a rule that ignored the branch's reflog altogether would lose the witness a person's own reset leaves in it.
///  Measured with Git 2.56 on a fast-forward fixture:
///  `main`'s reflog line is `<before> <now> ... merge feature: Fast-forward`.
#[test]
fn the_move_a_merge_made_in_its_own_branch_is_no_witness_and_an_earlier_one_is() {
    let lines: Vec<String> = vec![
        format!("{NO_COMMIT} {FIRST} {WHO}\tbranch: Created from HEAD"),
        format!("{FIRST} {SECOND} {WHO}\tcommit: the branch's own work"),
    ];
    let common: PathBuf = a_common_dir_with_reflog("branch-reflog-merge-move", "refs/heads/main", lines.as_slice());
    let merge: MergeMove<'_> = MergeMove { from: FIRST, to: SECOND };
    assert!(
        !branch_moved_onto(common.as_path(), "refs/heads/main", SECOND, Some(merge)),
        "the move the merge itself made is no witness"
    );
    assert!(
        branch_moved_onto(common.as_path(), "refs/heads/main", SECOND, Some(MergeMove { from: SECOND, to: FIRST })),
        "a move the merge did not make is still a witness"
    );
}
