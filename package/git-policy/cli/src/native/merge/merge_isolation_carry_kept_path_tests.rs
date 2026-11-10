//! Controls for the line about one path a carry kept whole:
//! what stands there of the person's own work,
//! where the merge's version is,
//! and the command that reaches it.

use super::*;
use crate::merge_isolation_carry_parts::CapturedFile;

/// A captured worktree entry of one path,
/// as the capture records it.
fn captured(path: &str, mode: &str) -> CapturedPath {
    let file: CapturedFile = CapturedFile { mode: String::from(mode), raw: String::from("raw"), blob: String::from("blob") };
    return CapturedPath { path: path.as_bytes().to_vec(), worktree: Some(file), index: None };
}

/// A captured path the person deleted,
/// which the index does not hold either.
fn deleted(path: &str) -> CapturedPath {
    return CapturedPath { path: path.as_bytes().to_vec(), worktree: None, index: None };
}

/// The merge's change at one path,
/// either a version of it or a deletion.
fn change(path: &str, exists_after: bool) -> Change {
    let after: (&str, &str) = if exists_after { ("100644", "after") } else { ("000000", "0000000") };
    return Change {
        path: path.as_bytes().to_vec(),
        old_mode: String::from("100644"),
        new_mode: String::from(after.0),
        old_oid: String::from("before"),
        new_oid: String::from(after.1),
    };
}

/// What:
///  A path the person deleted and the merge changes is named as their kept deletion,
/// with the merge's version in the merge commit,
///  and never as a file that holds bytes the path does not have.
/// Why:
///  The line is what sends a person to their work,
/// and the file is not there.
#[test]
fn a_deleted_path_names_the_deletion_and_the_way_to_the_merges_version() {
    let kept: Kept = kept_path(&deleted("a.txt"), &change("a.txt", true), "you deleted it, and the merge changes it", false);
    let line: String = kept_line("feature", &kept);
    assert!(line.contains("a.txt keeps your own deletion"), "{line}");
    assert!(line.contains("the merge's version of a.txt is in the merge commit"), "{line}");
    assert!(line.contains("git restore --source=HEAD --staged --worktree -- a.txt"), "{line}");
    assert!(!line.contains("holds your own bytes"), "{line}");
    assert!(!line.contains("holds what you wrote"), "{line}");
}

/// What:
///  A path whose own bytes the carry kept whole names those bytes and the merge's version beside them.
#[test]
fn a_kept_file_names_the_bytes_that_stand_and_the_merges_version() {
    let kept: Kept = kept_path(&captured("a.txt", "100644"), &change("a.txt", true), "its changes cannot be merged as text", false);
    let line: String = kept_line("feature", &kept);
    assert!(line.contains("a.txt holds your own bytes"), "{line}");
    assert!(line.contains("git restore --source=HEAD -- a.txt"), "{line}");
    assert!(!line.contains("keeps your own deletion"), "{line}");
}

/// What:
///  A path the person made a symbolic link of is named as their link,
/// not as bytes of a file.
#[test]
fn a_kept_link_is_named_as_a_link() {
    let kept: Kept = kept_path(&captured("a.txt", "120000"), &change("a.txt", true), "its changes cannot be merged as text", false);
    let line: String = kept_line("feature", &kept);
    assert!(line.contains("a.txt is your own symbolic link"), "{line}");
    assert!(!line.contains("holds your own bytes"), "{line}");
}

/// What:
///  A path the merge deletes names the merge's absence,
/// and a command that takes that deletion,
/// which is `rm`
///  for a file the index does not hold.
#[test]
fn a_path_the_merge_deletes_names_its_absence_and_the_command_for_it() {
    let kept: Kept = kept_path(&captured("gone.txt", "100644"), &change("gone.txt", false), "the merge deletes it", false);
    let line: String = kept_line("feature", &kept);
    assert!(line.contains("gone.txt holds your own bytes"), "{line}");
    assert!(line.contains("The merge commit holds no gone.txt"), "{line}");
    assert!(line.contains("`rm -- gone.txt` takes the merge's deletion"), "{line}");
    assert!(!line.contains("git restore --"), "{line}");
}

/// What:
///  The same absence for a path the index still holds names `git rm`,
/// which is the command that reaches an
///  entry `rm` alone leaves behind.
#[test]
fn a_path_the_merge_deletes_that_the_index_holds_names_git_rm() {
    let kept: Kept = kept_path(&captured("gone.txt", "100644"), &change("gone.txt", false), "the merge deletes it", true);
    let line: String = kept_line("feature", &kept);
    assert!(line.contains("`git rm --force -- gone.txt` takes the merge's deletion"), "{line}");
    assert!(!line.contains("`rm -- gone.txt`"), "{line}");
}

/// What:
///  A path neither side holds is named as nothing on both sides,
/// because a line this command prints is
///  true even where no layer could decide it.
#[test]
fn a_path_neither_side_holds_is_named_as_nothing() {
    let kept: Kept = kept_path(&deleted("a.txt"), &change("a.txt", false), "the merge deletes it", false);
    let line: String = kept_line("feature", &kept);
    assert!(line.contains("a.txt holds nothing"), "{line}");
    assert!(line.contains("The merge commit holds no a.txt"), "{line}");
    assert!(!line.contains("restore"), "{line}");
}
