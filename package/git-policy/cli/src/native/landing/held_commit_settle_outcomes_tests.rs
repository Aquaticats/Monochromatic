//! What:
//!  Controls for the line the service writes about a settled hold:
//!  one per state a settlement decides between.
//! Why:
//!  The line is the only thing a person reading the journal has about a held commit,
//!  and the host run of the 0.6.1-alpha.10 release showed what a line without a repository costs:
//!  a held fixture commit was read as a commit on this repository's own `main` that failed.
//!  Every state therefore names the repository,
//!  and none of them words a held commit as one a lock refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(settledWords(7, held, { waitsForLocks: ['/repo/.git/refs/heads/main.lock'] })).toContain('in the repository at /repo');
//! ```

use super::{Settled, settled_words};
use crate::held_commit_record::Held;
use std::path::{Path, PathBuf};

/// The batch every control here writes a line about.
const SEQ: u64 = 7;

/// The worktree root every control here acts in.
const REPOSITORY: &str = "/home/a person/repo";

/// The branch the batch moves.
const BRANCH: &str = "refs/heads/main";

/// The commit the batch makes.
const COMMIT: &str = "3ad2a9ab2e32eaf6f484b23fafe0cba5c5172a98";

/// A held record for a batch of `REPOSITORY`,
/// with the branch and the commit every control names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = { seq: SEQ, repository: REPOSITORY, reference: BRANCH, new: COMMIT };
/// ```
fn held() -> Held {
    return Held {
        seq: SEQ,
        journal: PathBuf::from("/state/cli-git/service/instance"),
        repository: PathBuf::from(REPOSITORY),
        common_dir: PathBuf::from("/home/a person/repo/.git"),
        git_dir: PathBuf::from("/home/a person/repo/.git"),
        reference: String::from(BRANCH),
        old: String::from("28eee868fd46048086fa119276660a6900226510"),
        new: String::from(COMMIT),
        commits: vec![String::from(COMMIT)],
        index: PathBuf::from("/home/a person/repo/.git/index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![PathBuf::from("/home/a person/repo/.git/refs/heads/main.lock")],
        created: String::from("2026-11-12T00:00:00+00:00"),
    };
}

/// The words a line names the repository with,
/// which every state carries.
fn names_the_repository(line: &str) -> bool {
    return line.contains(format!("in the repository at {REPOSITORY}").as_str());
}

/// What:
///  A line about a held commit names the repository,
///  says the commit is kept,
///  and says the branch has not moved yet.
/// Why:
///  A commit a lock keeps off its branch is not one that failed:
///  it is kept and waits,
///  which is what the owner's "Commits should never fail" asks of it.
#[test]
fn a_lock_that_still_stands_is_said_as_a_commit_that_is_kept() {
    let lock: PathBuf = Path::new("/home/a person/repo/.git/refs/heads/main.lock").to_path_buf();
    let line: String = settled_words(SEQ, &held(), &Settled::WaitsForLocks(vec![lock]));
    assert!(names_the_repository(line.as_str()), "the repository is named: {line}");
    assert!(line.contains(&format!("the commit {COMMIT} is kept")), "the commit is kept: {line}");
    assert!(line.contains("has not moved yet"), "the branch has not moved yet: {line}");
    assert!(line.contains("refs/heads/main.lock"), "the lock it waits for is named: {line}");
    assert!(line.contains("removing the lock with `rm -- "), "what the person may do is said, and only when no Git process uses the repository: {line}");
    assert!(!line.contains("refus"), "nothing was refused: {line}");
}

/// What:
///  A line about a commit that landed says the lock is gone and the branch names it.
/// Why:
///  The settlement moved the branch,
///  so the person has nothing to wait for and needs to know which commit it is.
#[test]
fn a_commit_that_landed_is_said_as_landed() {
    let line: String = settled_words(SEQ, &held(), &Settled::Landed { commit: String::from(COMMIT) });
    assert!(names_the_repository(line.as_str()), "the repository is named: {line}");
    assert!(line.contains("was held: the lock is gone"), "the lock is gone: {line}");
    assert!(line.contains(&format!("the commit {COMMIT} is on {BRANCH} now")), "the branch names the commit: {line}");
}

/// What:
///  A commit that landed with its index update still owed says so,
///  so the person knows the next command sets those entries.
/// Why:
///  A branch that shows the commit while the index does not is a state a person will meet in `git status`,
///  and the line is where they learn why.
#[test]
fn a_commit_that_landed_with_the_index_owed_is_said_with_the_next_command() {
    let line: String = settled_words(SEQ, &held(), &Settled::LandedTheIndexWaits { commit: String::from(COMMIT) });
    assert!(names_the_repository(line.as_str()), "the repository is named: {line}");
    assert!(line.contains("the index catches up on the next command"), "the index is said to catch up: {line}");
}

/// What:
///  A change that does not combine with the branch says the commit is kept and names every path.
/// Why:
///  Nothing decides a conflict for the person,
///  and the commit is never dropped over one.
#[test]
fn a_commit_that_cannot_be_carried_over_names_the_paths_and_is_kept() {
    let settled: Settled = Settled::Conflicts { commit: String::from(COMMIT), paths: vec![b"a.txt".to_vec(), b"b.txt".to_vec()] };
    let line: String = settled_words(SEQ, &held(), &settled);
    assert!(names_the_repository(line.as_str()), "the repository is named: {line}");
    assert!(line.contains(&format!("the commit {COMMIT} is kept")), "the commit is kept: {line}");
    assert!(line.contains("a.txt, b.txt"), "every path that conflicts is named: {line}");
}

/// What:
///  A state this command cannot move says the commit is kept and why nothing moved.
/// Why:
///  A merge,
/// a branch another session created or a signature is no reason to lose a commit.
#[test]
fn a_state_this_command_cannot_move_says_so_and_keeps_the_commit() {
    let settled: Settled = Settled::Cannot { commit: String::from(COMMIT), because: String::from("a signed commit is not signed again by a command") };
    let line: String = settled_words(SEQ, &held(), &settled);
    assert!(names_the_repository(line.as_str()), "the repository is named: {line}");
    assert!(line.contains("this command cannot move refs/heads/main"), "the branch it cannot move is named: {line}");
    assert!(line.contains("a signed commit is not signed again"), "the reason is given: {line}");
    assert!(!line.contains("refus"), "nothing was refused: {line}");
}

/// What:
///  A record whose commit the branch already shows says nothing waits any more.
/// Why:
///  A record can outlive its batch,
///  and saying so keeps a person from looking for the commit twice.
#[test]
fn a_record_whose_commit_is_there_says_nothing_waits() {
    let line: String = settled_words(SEQ, &held(), &Settled::AlreadyThere { commit: String::from(COMMIT) });
    assert!(names_the_repository(line.as_str()), "the repository is named: {line}");
    assert!(line.contains("is not held any more"), "nothing waits: {line}");
    assert!(line.contains(&format!("{BRANCH} already names {COMMIT}")), "the branch names the commit: {line}");
}
