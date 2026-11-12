//! Controls for the line about one path a carry kept whole:
//! what stands there of the person's own work,
//! where the merge's version is,
//! and the command that reaches it.

use super::*;
use crate::merge_isolation_carry_parts::CapturedFile;

/// A captured worktree entry of one path,
/// as the capture records it.
fn captured(path: &str, mode: &str) -> CapturedPath {
    let file: CapturedFile = CapturedFile { mode: String::from(mode), raw: String::from("raw"), blob: String::from("blob"), permissions: None };
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
    let kept: Kept = kept_path(&deleted("a.txt"), &change("a.txt", true), "you deleted it, and the merge changes it", Staged::Nothing);
    let line: String = kept_line("git merge feature", &kept);
    assert!(line.contains("a.txt holds nothing of yours"), "{line}");
    assert!(line.contains("the merge's version of a.txt is in the merge commit"), "{line}");
    assert!(line.contains("git restore --no-enforce-worktree --source=HEAD --staged --worktree -- a.txt"), "{line}");
    assert!(!line.contains("holds your own bytes"), "{line}");
    assert!(!line.contains("holds what you wrote"), "{line}");
}

/// What:
///  A path whose own bytes the carry kept whole names those bytes and the merge's version beside them.
#[test]
fn a_kept_file_names_the_bytes_that_stand_and_the_merges_version() {
    let kept: Kept = kept_path(&captured("a.txt", "100644"), &change("a.txt", true), "its changes cannot be merged as text", Staged::Nothing);
    let line: String = kept_line("git merge feature", &kept);
    assert!(line.contains("a.txt holds your own bytes"), "{line}");
    assert!(line.contains("git restore --no-enforce-worktree --source=HEAD -- a.txt"), "{line}");
    assert!(!line.contains("holds nothing of yours"), "{line}");
}

/// What:
///  A path the person made a symbolic link of is named as their link,
/// not as bytes of a file.
#[test]
fn a_kept_link_is_named_as_a_link() {
    let kept: Kept = kept_path(&captured("a.txt", "120000"), &change("a.txt", true), "its changes cannot be merged as text", Staged::Nothing);
    let line: String = kept_line("git merge feature", &kept);
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
    let kept: Kept = kept_path(&captured("gone.txt", "100644"), &change("gone.txt", false), "the merge deletes it", Staged::Nothing);
    let line: String = kept_line("git merge feature", &kept);
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
    let kept: Kept = kept_path(&captured("gone.txt", "100644"), &change("gone.txt", false), "the merge deletes it", Staged::ThePathsBytes);
    let line: String = kept_line("git merge feature", &kept);
    assert!(line.contains("`git rm --no-enforce-worktree --force -- gone.txt` takes the merge's deletion"), "{line}");
    assert!(!line.contains("`rm -- gone.txt`"), "{line}");
}

/// What:
///  A path whose name holds a space is one shell word in every command the line prints.
/// Why:
///  The person copies the command out of the line,
/// and a name a shell splits into three arguments reaches
///  `rm` as three paths:
/// `rm -- gone with space.txt` deleted unrelated files named `gone`,
/// `with` and
///  `space.txt` and left the intended one in place
/// (review `merge-notice-for-a-deleted-file` round 133,
/// finding 1).
#[test]
fn a_path_with_a_space_is_quoted_inside_every_command_the_line_prints() {
    let kept: Kept = kept_path(&captured("gone with space.txt", "100644"), &change("gone with space.txt", false), "the merge deletes it", Staged::Nothing);
    let line: String = kept_line("git merge feature", &kept);
    assert!(line.contains("`rm -- 'gone with space.txt'` takes the merge's deletion"), "{line}");
    assert!(line.contains("`git add -- 'gone with space.txt'`"), "{line}");
    let standing: Kept = kept_path(&captured("gone with space.txt", "100644"), &change("gone with space.txt", true), "its changes cannot be merged as text", Staged::Nothing);
    let standing_line: String = kept_line("git merge feature", &standing);
    assert!(standing_line.contains("`git restore --no-enforce-worktree --source=HEAD -- 'gone with space.txt'`"), "{standing_line}");
    let gone: Kept = kept_path(&deleted("gone with space.txt"), &change("gone with space.txt", true), "you deleted it, and the merge changes it", Staged::Nothing);
    let gone_line: String = kept_line("git merge feature", &gone);
    assert!(gone_line.contains("`git restore --no-enforce-worktree --source=HEAD --staged --worktree -- 'gone with space.txt'`"), "{gone_line}");
}

/// What:
///  A path neither side holds is named as nothing on both sides,
/// because a line this command prints is
///  true even where no layer could decide it.
#[test]
fn a_path_neither_side_holds_is_named_as_nothing() {
    let kept: Kept = kept_path(&deleted("a.txt"), &change("a.txt", false), "the merge deletes it", Staged::Nothing);
    let line: String = kept_line("git merge feature", &kept);
    assert!(line.contains("a.txt holds nothing"), "{line}");
    assert!(line.contains("The merge commit holds no a.txt"), "{line}");
    assert!(!line.contains("restore"), "{line}");
}

/// What:
///  What the index holds at a kept path is told from the bytes each command in the line would write.
/// Why:
///  The line may name `git add` only where it replaces nothing of the person's,
/// and a restore that writes the index
///  only where the index holds no version of theirs.
#[test]
fn the_index_layer_is_told_by_the_bytes_a_command_would_write() {
    assert_eq!(staged_of(None, Some("the-path"), Some("the-merge")), Staged::Nothing);
    assert_eq!(staged_of(Some("the-path"), Some("the-path"), Some("the-merge")), Staged::ThePathsBytes);
    assert_eq!(staged_of(Some("the-merge"), None, Some("the-merge")), Staged::TheMergesVersion);
    assert_eq!(staged_of(Some("staged"), Some("saved"), Some("the-merge")), Staged::OwnVersion);
}

/// What:
///  A path the merge deletes,
/// whose index holds a version of the person's that differs from the bytes at the path,
/// names no `git add`:
///  that command would replace the staged version with the bytes at the path.
/// Why:
///  The capture's ref goes when the merge lands,
/// so the staged version stands only in the index,
/// and
///  following the old line left the index holding the later save
/// (review `merge-notice-for-a-deleted-file` round 133,
/// finding 2).
#[test]
fn a_staged_version_of_the_persons_is_never_told_to_run_git_add() {
    let kept: Kept = kept_path(&captured("gone.txt", "100644"), &change("gone.txt", false), "the merge deletes it", Staged::OwnVersion);
    let line: String = kept_line("git merge feature", &kept);
    assert!(!line.contains("keeps your bytes in the index instead"), "no command writes over the staged version: {line}");
    assert!(line.contains("would replace the version the index holds"), "the line says why `git add` is not the command to run: {line}");
    assert!(line.contains("leave the path as it is and both stand"), "the line says both layers stand: {line}");
    assert!(line.contains("`git rm --no-enforce-worktree --force -- gone.txt` takes the merge's deletion"), "{line}");
}

/// What:
///  A path the merge writes,
/// whose index holds a version of the person's,
/// names a restore that writes the checkout alone.
/// Why:
///  `--staged` writes the merge's version into the index too,
/// which replaced the version the person staged there
///  (the deletion half of review `merge-notice-for-a-deleted-file` round 133,
/// finding 2).
#[test]
fn a_staged_version_of_the_persons_stays_through_a_restore_of_the_merges_version() {
    let kept: Kept = kept_path(&deleted("a.txt"), &change("a.txt", true), "you deleted it, and the merge changes it", Staged::OwnVersion);
    let line: String = kept_line("git merge feature", &kept);
    assert!(line.contains("`git restore --no-enforce-worktree --source=HEAD -- a.txt` puts it in the checkout"), "{line}");
    assert!(!line.contains("--staged"), "the command writes no index entry over theirs: {line}");
    assert!(line.contains("the version of yours that the index holds stays there"), "{line}");
}
