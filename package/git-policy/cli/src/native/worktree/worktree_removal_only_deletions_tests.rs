//! What:
//!  Controls for the question a removal asks about a tree its own earlier attempt half emptied.
//! Why:
//!   The answer decides whether a removal may finish a tree with `--force`,
//! and `--force` deletes every file Git
//!  would keep:
//! a file a person changed or added must make the answer false,
//! or their work goes with the tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(onlyDeletions(treeWithOneFileDeleted)).toBe(true);
//! expect(onlyDeletions(treeWithOneFileChanged)).toBe(false);
//! ```

use super::*;
use crate::test_support::{age, fixture, git, git_context, remove, repository};
use std::path::PathBuf;

/// A repository of one tracked file and one ignored name,
/// with a linked worktree `topic` in it.
fn a_repository_with_a_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("a tracked file");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("an ignore rule");
    git(repo.as_path(), &["add", "--", "tracked.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let worktree: PathBuf = root.join("topic");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", worktree.to_str().expect("a UTF-8 fixture path")]);
    age(worktree.join("tracked.txt").as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (root, repo, worktree);
}

/// The administrative directory of the worktree in a repository,
/// which Git's status of that worktree is asked with.
fn admin_of(worktree: &Path) -> PathBuf {
    let pointer: String = std::fs::read_to_string(worktree.join(".git")).expect("the worktree's pointer");
    let named: &str = pointer.trim_end().strip_prefix("gitdir: ").expect("a pointer names a Git directory");
    return std::fs::canonicalize(worktree.join(named)).expect("the administrative directory");
}

/// What:
///  A worktree whose only trouble is a tracked entry missing from it answers yes,
///  which is the state a removal that was killed in its pass leaves.
/// Why:
///   The pass deletes the worktree's own entries and then finishes the removal,
/// so a command killed between the two
///  leaves exactly this,
/// and the same unforced command has to read it as its own work.
#[test]
fn a_tree_with_only_a_missing_tracked_entry_answers_yes() {
    let (root, _repo, worktree) = a_repository_with_a_worktree("only-deletions-missing");
    std::fs::remove_file(worktree.join("tracked.txt")).expect("remove the tracked file");
    assert!(only_deletions(&git_context(), worktree.as_path(), admin_of(worktree.as_path()).as_path()), "a missing tracked entry is a deletion");
    remove(root.as_path());
}

/// What:
///  A worktree whose tracked entry is changed answers no,
///  and one with a staged deletion answers yes.
/// Why:
///   `--force` deletes a changed file,
/// so a containment that read a change as a deletion would take a person's work;
/// a separate test asks the staged case because that is the same deletion Git names with another letter.
#[test]
fn a_changed_file_answers_no_and_a_staged_deletion_answers_yes() {
    let (root, _repo, worktree) = a_repository_with_a_worktree("only-deletions-changed");
    std::fs::write(worktree.join("tracked.txt"), b"mine\n").expect("change the tracked file");
    assert!(!only_deletions(&git_context(), worktree.as_path(), admin_of(worktree.as_path()).as_path()), "a changed file is not a deletion");
    git(worktree.as_path(), &["add", "--", "tracked.txt"]);
    assert!(!only_deletions(&git_context(), worktree.as_path(), admin_of(worktree.as_path()).as_path()), "a staged change is not a deletion");
    git(worktree.as_path(), &["rm", "--quiet", "--force", "tracked.txt"]);
    assert!(only_deletions(&git_context(), worktree.as_path(), admin_of(worktree.as_path()).as_path()), "a staged deletion is a deletion");
    remove(root.as_path());
}

/// What:
///  A worktree that holds an untracked file answers no,
///  and one Git reports nothing about answers no too,
///  saying that it names nothing rather than that it names only deletions.
/// Why:
///   An untracked file is the one thing a removal spares,
/// so a tree holding one is not a tree to finish with `--force`;
/// and a tree Git reports nothing about was never half emptied by anything,
/// so reading its empty report as
///  "only deletions" told a person their fresh worktree was half gone.
#[test]
fn an_untracked_file_answers_no_and_a_clean_tree_names_nothing() {
    let (root, _repo, worktree) = a_repository_with_a_worktree("only-deletions-untracked");
    let admin: PathBuf = admin_of(worktree.as_path());
    assert!(!only_deletions(&git_context(), worktree.as_path(), admin.as_path()), "a clean tree is not a report of deletions");
    assert_eq!(reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[]), Reported::Nothing, "it names nothing at all");
    std::fs::write(worktree.join("notes.txt"), b"mine\n").expect("an untracked file");
    assert!(!only_deletions(&git_context(), worktree.as_path(), admin.as_path()), "an untracked file is not a deletion");
    // An ignored file is not named by Git at all, so only its own entry's absence is reported.
    std::fs::create_dir_all(worktree.join("ignored")).expect("an ignored directory");
    std::fs::write(worktree.join("ignored/state.txt"), b"build state\n").expect("an ignored file");
    std::fs::remove_file(worktree.join("notes.txt")).expect("remove the untracked file");
    assert_eq!(reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[]), Reported::Nothing, "an ignored file is no change Git reports");
    remove(root.as_path());
}

/// What:
///  An untracked entry inside a volume the removal leaves does not make the answer false,
///  and one outside it does.
/// Why:
///   The pass deletes the ignore rule that hid the volume,
/// so Git reports the volume itself as untracked in the state
///  the pass leaves;
/// an entry of the person's elsewhere still keeps the worktree,
/// because nothing of theirs may go.
#[test]
fn an_untracked_entry_inside_a_volume_is_left_out_of_the_question() {
    let (root, _repo, worktree) = a_repository_with_a_worktree("only-deletions-volume");
    let volume: PathBuf = worktree.join("snap");
    std::fs::create_dir_all(volume.as_path()).expect("the directory the volume stands in");
    std::fs::write(volume.join("precious.txt"), b"mine\n").expect("a file inside the volume");
    std::fs::remove_file(worktree.join("tracked.txt")).expect("the entry the pass removed");
    std::fs::remove_file(worktree.join(".gitignore")).expect("the ignore rule the pass removed");
    let admin: PathBuf = admin_of(worktree.as_path());
    assert_eq!(reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[volume.as_path()]), Reported::OnlyDeletions, "the volume's own entries are not the worktree's");
    assert_eq!(reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[]), Reported::Something, "and they are what a strict reading refuses");
    std::fs::write(worktree.join("notes.txt"), b"mine\n").expect("an untracked file of the person's");
    assert_eq!(reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[volume.as_path()]), Reported::Something, "an entry outside the volume still keeps the tree");
    remove(root.as_path());
}

/// What:
///  A worktree Git reports nothing about is told apart from one whose entries are all missing,
///  and from one holding a change.
/// Why:
///   An empty `git status` read as "only deletions" made a fresh, clean worktree look like one a pass of
///   this program had half emptied,
/// and the words built on that answer told a person their entries were
///  missing when none was;
/// the state is the one every person meets a moment after a worktree is added.
#[test]
fn an_empty_report_is_told_apart_from_a_report_of_deletions() {
    let (root, _repo, worktree) = a_repository_with_a_worktree("only-deletions-empty");
    let admin: PathBuf = admin_of(worktree.as_path());
    assert_eq!(
        reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[]),
        Reported::Nothing,
        "a tree Git names nothing in is no half-emptied tree"
    );
    assert!(!only_deletions(&git_context(), worktree.as_path(), admin.as_path()), "and nothing may read it as a report of deletions");
    std::fs::remove_file(worktree.join("tracked.txt")).expect("remove the tracked file");
    assert_eq!(
        reported_outside(&git_context(), worktree.as_path(), admin.as_path(), &[]),
        Reported::OnlyDeletions,
        "an entry missing from the tree is the shape a killed pass leaves"
    );
    assert!(only_deletions(&git_context(), worktree.as_path(), admin.as_path()), "and that one is a report of deletions");
    remove(root.as_path());
}
