//! What:
//!  Controls of the edits a mark keeps Git from noticing.
//! Why:
//!  The two marks hide an edit from every command of Git,
//!  so a removal that reads only Git's own answer deletes it,
//!  and a proof that named every marked file would keep worktrees nothing stands in the way of.
//!  Each control therefore names exactly the file the comparison proved different.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! hiddenEditsBeneath(context, worktree, admin, common) // ['the tracked file base.txt is marked ...']
//! ```

use super::hidden_edits_beneath;
use crate::test_support::{fixture, git, git_context, repository};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// A repository and one linked worktree of it,
/// with one tracked file in the first commit.
struct Fixture {
    /// The common Git directory.
    common: PathBuf,
    /// The linked worktree.
    worktree: PathBuf,
    /// That worktree's own Git directory.
    admin: PathBuf,
}

impl Fixture {
    /// What a removal of the linked worktree would leave behind a mark.
    fn hidden(&self) -> Result<Vec<String>, String> {
        let context: GitContext = git_context();
        return hidden_edits_beneath(&context, self.worktree.as_path(), self.admin.as_path(), self.common.as_path());
    }

    /// Write other bytes into the tracked file.
    fn edit(&self) {
        std::fs::write(self.worktree.join("base.txt"), b"my edit, only here\n").expect("an edit to a tracked file");
    }
}

/// What:
///  The repository every control here works on.
/// Why:
///  The comparison reads the worktree's index,
///  the file it names and the attributes Git would apply,
///  so the fixture holds a committed tracked file and its own worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithAWorktree('hidden-edits');
/// ```
fn a_repository_with_a_worktree(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    std::fs::write(main.join("base.txt"), b"base\n").expect("a tracked file");
    git(main.as_path(), &["add", "--", "base.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let added: PathBuf = root.join("topic");
    let named: &str = added.to_str().expect("a UTF-8 fixture path");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", named]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    let worktree: PathBuf = std::fs::canonicalize(added.as_path()).expect("the worktree's real path");
    let admin: PathBuf = common.join("worktrees").join("topic");
    return Fixture { common, worktree, admin };
}

/// What:
///  A worktree with no marked entry holds no hidden edit,
///  which is what lets a removal go.
/// Why:
///  Every other control here keeps a worktree,
///  and one that always answered "keep" would pass all of them.
#[test]
fn a_clean_worktree_holds_no_hidden_edit() {
    let fixture: Fixture = a_repository_with_a_worktree("hidden-edits-clean");
    fixture.edit();
    // An edit Git can see is Git's own refusal, not this question's.
    assert_eq!(fixture.hidden(), Ok(Vec::new()), "no entry carries a mark, so nothing is judged here");
}

/// What:
///  An edit behind `assume-unchanged` is named,
///  with the file and the two ways forward.
/// Why:
///  This is the mark the wrapper's own proof of cleanliness declines on,
///  and the removal then handed the worktree to Git,
/// which deleted the edit (a Sonnet audit,
/// finding F3).
#[test]
fn an_edit_behind_assume_unchanged_is_named() {
    let fixture: Fixture = a_repository_with_a_worktree("hidden-edits-assume");
    git(fixture.worktree.as_path(), &["update-index", "--assume-unchanged", "base.txt"]);
    fixture.edit();
    let said: Vec<String> = fixture.hidden().expect("the index and the file can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("the tracked file base.txt"), "{}", said[0]);
    assert!(said[0].contains("assume-unchanged"), "{}", said[0]);
}

/// What:
///  An edit behind `skip-worktree` is named the same way.
/// Why:
///  The second mark hides the edit from Git the same way,
///  and it is stored in a different place in the index,
///  so a proof that read one of the two would let the other through.
#[test]
fn an_edit_behind_skip_worktree_is_named() {
    let fixture: Fixture = a_repository_with_a_worktree("hidden-edits-skip");
    git(fixture.worktree.as_path(), &["update-index", "--skip-worktree", "base.txt"]);
    fixture.edit();
    let said: Vec<String> = fixture.hidden().expect("the index and the file can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("the tracked file base.txt"), "{}", said[0]);
    assert!(said[0].contains("skip-worktree"), "{}", said[0]);
}

/// What:
///  A marked file whose bytes are still the ones its entry names is no hidden edit.
/// Why:
///  Marking a file is a thing people do to keep Git from looking,
///  and a proof that kept every marked worktree would keep worktrees for the mark alone.
#[test]
fn a_marked_file_that_did_not_change_is_not_named() {
    let fixture: Fixture = a_repository_with_a_worktree("hidden-edits-unchanged");
    git(fixture.worktree.as_path(), &["update-index", "--assume-unchanged", "base.txt"]);
    assert_eq!(fixture.hidden(), Ok(Vec::new()), "the bytes are the ones the entry names");
}

/// What:
///  A marked file the worktree does not hold is no hidden edit.
/// Why:
///  A sparse checkout marks the files it leaves out,
///  and none of them is an edit the removal could destroy.
#[test]
fn a_marked_file_the_worktree_does_not_hold_is_not_named() {
    let fixture: Fixture = a_repository_with_a_worktree("hidden-edits-absent");
    git(fixture.worktree.as_path(), &["update-index", "--skip-worktree", "base.txt"]);
    std::fs::remove_file(fixture.worktree.join("base.txt")).expect("the file goes");
    assert_eq!(fixture.hidden(), Ok(Vec::new()), "an absent file is nothing a removal deletes");
}
