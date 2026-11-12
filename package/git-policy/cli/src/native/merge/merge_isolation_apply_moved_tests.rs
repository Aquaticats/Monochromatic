//! Controls for what a landed merge says and does when its branch moved while its own carry ran:
//! the line that names the moved commit,
//! the per-path lines of the carry onto it,
//! and the endings that keep the person's bytes for the next command.

use super::*;

/// The lines a carry onto the moved head writes,
/// handed to the builder the way the carry hands them.
const CARRIED_LINES: &str = "monochromatic-git-policy-cli: git merge feature: your changes to a.txt were merged.\n";

/// What:
///  A carry onto the moved head names the branch,
/// the commit it moved to,
/// and then the lines the carry itself wrote.
/// Why:
///   The person reads why the lines that follow name the branch rather than their edit alone,
/// and the per-path lines must come
///  after the sentence that explains them.
#[test]
fn the_carry_onto_the_branch_names_the_commit_it_moved_to() {
    let line: String = the_carry_onto_the_branch_as_it_is("feature", "refs/heads/main", "abc123", CARRIED_LINES.as_bytes());
    assert!(line.contains("git merge feature"), "{line}");
    assert!(line.contains("refs/heads/main moved to abc123"), "{line}");
    assert!(line.contains("merged with the branch as it is now"), "{line}");
    assert!(line.ends_with(CARRIED_LINES), "the carry's own lines follow the sentence: {line}");
}

/// What:
///  A branch that moved away from the merge is not written over,
/// and the line says where the person's work is until the next command.
/// Why:
///   This command may not put a merge back into a branch that no longer holds it,
/// so the record and the ref are the
///  only promise it can make.
#[test]
fn the_branch_that_moved_away_is_named_and_the_record_kept() {
    let line: String = the_branch_is_not_built_on_it("feature", "refs/heads/main", "abc123");
    assert!(line.contains("refs/heads/main moved to abc123"), "{line}");
    assert!(line.contains("not built on this merge"), "{line}");
    assert!(line.contains("kept in the merge's record"), "{line}");
    assert!(line.contains("nothing of yours was removed"), "{line}");
}

/// What:
///  A branch that moved onto the merge while another program held the index says so,
/// and points at the same record.
/// Why:
///   The carry onto the moved head waits for the lock,
/// so the next command writes the same bytes and this one must
///  not claim they are in the files.
#[test]
fn the_held_index_is_named_with_the_record() {
    let line: String = the_index_kept_it("feature", "refs/heads/main", "abc123");
    assert!(line.contains("refs/heads/main moved to abc123"), "{line}");
    assert!(line.contains("holds this repository's index"), "{line}");
    assert!(line.contains("kept in the merge's record"), "{line}");
    assert!(line.contains("nothing of yours was removed"), "{line}");
}

/// What:
///  A branch this command could not read leaves the record too,
/// and the line names the branch rather than a commit.
/// Why:
///   A reading that failed proves nothing about the checkout,
/// so nothing may be dropped on it.
#[test]
fn the_branch_that_could_not_be_read_keeps_the_record() {
    let line: String = the_branch_could_not_be_read("feature", "refs/heads/main");
    assert!(line.contains("refs/heads/main could not be read"), "{line}");
    assert!(line.contains("kept in the merge's record"), "{line}");
    assert!(line.contains("nothing of yours was removed"), "{line}");
}
