//! What:
//!  Controls for the work of a worktree registered inside one being removed that no other copy holds.
//! Why:
//!   The refusal it feeds keeps a person's uncommitted edit,
//!  so each answer must be shown:
//! a changed tracked file is found,
//! a staged change is found,
//! a detached commit no reference names is found,
//! a branch's commit and a detached commit a branch contains are not,
//! and a worktree holding none of these is not a reason to keep the outer one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(nestedWorktreesHoldingGitWork(git, common, outer)).toEqual([{ worktree: '/x/nested', changed: { shown: ['a.txt'], total: 1 } }]);
//! ```

use super::{NestedHoldingGitWork, nested_worktrees_holding_git_work};
use crate::test_support::{age, fixture, git, git_context, repository};
use std::path::PathBuf;

/// A repository,
/// an outer worktree registered at `outer`,
/// and one worktree registered inside it at the path its ignore rules hide.
struct Nested {
    /// The common Git directory.
    common: PathBuf,
    /// The worktree the removal would take.
    outer: PathBuf,
    /// The worktree inside it.
    nested: PathBuf,
}

/// What:
///  The repository every control here works on.
/// Why:
///  The question is asked about a registered worktree inside another one,
///  and only a registration makes a directory a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithAWorktreeInsideAnother('nested-git-work');
/// ```
fn a_repository_with_a_worktree_inside_another(name: &str) -> Nested {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    std::fs::write(main.join("README.md").as_path(), b"# fixture\n").expect("a tracked file");
    std::fs::write(main.join(".gitignore").as_path(), b"nested/\n").expect("the ignore rules");
    age(main.join("README.md").as_path());
    age(main.join(".gitignore").as_path());
    git(main.as_path(), &["add", "--", "README.md", ".gitignore"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let outer: PathBuf = root.join("outer");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "outer", outer.to_str().expect("a UTF-8 fixture path")]);
    let nested: PathBuf = outer.join("nested");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "nested", nested.to_str().expect("a UTF-8 fixture path")]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    return Nested {
        common,
        outer: std::fs::canonicalize(outer.as_path()).expect("the outer worktree's real path"),
        nested: std::fs::canonicalize(nested.as_path()).expect("the nested worktree's real path"),
    };
}

#[test]
fn a_nested_worktree_holding_nothing_of_the_persons_is_no_reason_to_keep_the_outer_one() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-clean");
    assert_eq!(
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()),
        Ok(Vec::new()),
        "a nested worktree with nothing changed holds no work a removal would lose"
    );
}

#[test]
fn an_edit_to_a_tracked_file_of_a_nested_worktree_is_found_and_named() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-edited");
    std::fs::write(place.nested.join("README.md").as_path(), b"# what the person changed\n").expect("an uncommitted edit");
    let found: Vec<NestedHoldingGitWork> =
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()).expect("the question is answered");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].worktree, place.nested, "the nested worktree is named");
    assert_eq!(found[0].changed.shown, vec![String::from("README.md")], "the edited file is named");
    assert!(!found[0].holds_a_commit_nothing_else_names, "its HEAD is a branch, so no commit of it is lost");
    assert_eq!(found[0].why(), "changed tracked files (`README.md`)");
}

#[test]
fn a_staged_change_of_a_nested_worktree_is_found() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-staged");
    std::fs::write(place.nested.join("staged.txt").as_path(), b"what the person staged\n").expect("a file");
    git(place.nested.as_path(), &["add", "--", "staged.txt"]);
    let found: Vec<NestedHoldingGitWork> =
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()).expect("the question is answered");
    assert_eq!(found.len(), 1, "a staged change is the person's work too: {found:?}");
    assert_eq!(found[0].changed.shown, vec![String::from("staged.txt")], "the staged file is named");
}

#[test]
fn an_untracked_file_alone_is_not_a_reason_to_keep_the_outer_one() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-untracked");
    std::fs::write(place.nested.join("untracked.txt").as_path(), b"untracked\n").expect("an untracked file");
    assert_eq!(
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()),
        Ok(Vec::new()),
        "an untracked file goes with the removal and is recorded, which is the decision of 2026-11-12T13:04:14"
    );
}

/// What:
///  A commit made in the nested worktree and nothing else,
///  reached through a detached `HEAD`.
/// Why:
///  A branch would name the commit and the branch survives the removal;
///  a detached `HEAD` names it only while the worktree's own Git directory exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commitOnADetachedHead(nested, 'a commit nothing else names')
/// ```
fn commit_on_a_detached_head(nested: &std::path::Path) {
    // The detach comes first: a commit made on the branch and then detached from is still named by the branch,
    //  which is the other control below.
    git(nested, &["checkout", "--quiet", "--detach"]);
    std::fs::write(nested.join("only-here.txt").as_path(), b"a commit nothing else names\n").expect("a file");
    git(nested, &["add", "--", "only-here.txt"]);
    git(nested, &["commit", "--quiet", "--message=a commit nothing else names"]);
}

#[test]
fn a_detached_commit_that_no_reference_names_is_found() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-detached");
    commit_on_a_detached_head(place.nested.as_path());
    let found: Vec<NestedHoldingGitWork> =
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()).expect("the question is answered");
    assert_eq!(found.len(), 1, "a commit only the nested worktree's own HEAD names is the person's work: {found:?}");
    assert!(found[0].holds_a_commit_nothing_else_names, "the commit is named as one nothing else holds");
    assert_eq!(found[0].why(), "a commit that no reference names, which only its own HEAD holds");
}

#[test]
fn a_detached_commit_a_branch_names_is_not_held_against_the_removal() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-detached-behind-a-branch");
    commit_on_a_detached_head(place.nested.as_path());
    // A branch naming the commit is what keeps it: the branch lives in the common Git directory and survives the removal.
    git(place.nested.as_path(), &["branch", "--force", "kept-by-a-branch", "HEAD"]);
    assert_eq!(
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()),
        Ok(Vec::new()),
        "a branch names the commit and the branch survives the removal"
    );
}

#[test]
fn a_worktree_on_a_branch_of_its_own_holds_no_commit_that_goes() {
    let place: Nested = a_repository_with_a_worktree_inside_another("nested-git-work-branch");
    std::fs::write(place.nested.join("committed.txt").as_path(), b"committed\n").expect("a file");
    git(place.nested.as_path(), &["add", "--", "committed.txt"]);
    git(place.nested.as_path(), &["commit", "--quiet", "--message=committed on the branch"]);
    let found: Vec<NestedHoldingGitWork> =
        nested_worktrees_holding_git_work(&git_context(), place.common.as_path(), place.outer.as_path()).expect("the question is answered");
    assert_eq!(found, Vec::new(), "the branch refs/heads/nested names the commit and survives");
}
