//! Controls for the clone rung's two provisional limits:
//! the changed-file cut is the fence's own share of the tracked paths,
//! the byte sum is the index's own record,
//! and each reason appears exactly at the edge it was pinned at.

use super::*;
use crate::test_support::{fixture, git_context, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use crate::worktree_snapshot_source::{SnapshotSource, source_or_dirty};
use std::path::PathBuf;

/// What:
///  The cut of a tracked path count,
/// and the count the fence declines at,
/// which is the same share of the same tree.
/// Why:
///   The cut is pinned at the fence's own decline share,
/// so a change to that share moves both and a control that stales here says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theChangedFileCut(n * DECLINE_SHARE_DIVISOR)).toBe(n);
/// ```
#[test]
fn the_cut_is_the_fence_own_share_of_the_tracked_paths() {
    let divisor: usize = crate::worktree_clone_conversion_fence::DECLINE_SHARE_DIVISOR;
    let tracked: usize = 24_620;
    assert_eq!(the_changed_file_cut(tracked), tracked / divisor, "the cut is one share of the tracked paths");
    assert_eq!(the_changed_file_cut(divisor * 3), 3, "the divisor splits the count and floors the rest");
    assert_eq!(the_changed_file_cut(divisor - 1), 0, "a tree smaller than the divisor takes no change at all");
}

/// What:
///  A source of the given tracked files and changes,
/// read through the source check,
/// so a control asks the limits what the ladder asks them.
/// Why:
///   Both reasons read the index the source check copied and the changes it listed,
/// and a control that built
///  those by hand would not show what an add sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = checkSource(repo); // the index, the tree of HEAD and the changes
/// ```
fn source_of(root: &std::path::Path, files: &[(&str, u64)]) -> SnapshotSource {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    for (name, bytes) in files {
        let path: PathBuf = repo.join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        let file: std::fs::File = std::fs::File::create(path.as_path()).expect("the file");
        file.set_len(*bytes).expect("the file's length");
    }
    commit_all(repo.as_path(), "fixture");
    return source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
}

/// What:
///  A tree of exactly the pinned bytes is cloned,
/// and one byte fewer is declined,
/// with the sum in the sentence.
/// Why:
///   The limit's edge is what the measurement was taken at,
/// so the control is the arithmetic beside it:
/// a
///  limit that said "at or below" or that summed something else would show here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reason(tree(THE_FEWEST))).toBeUndefined(); expect(reason(tree(THE_FEWEST - 1))).toContain('sum to');
/// ```
#[test]
fn the_size_limit_declines_one_byte_below_the_pinned_bytes_and_clones_it() {
    let at: PathBuf = fixture("limit-at-the-size");
    let at_source: SnapshotSource = source_of(at.as_path(), &[("one.bin", THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG)]);
    assert_eq!(tracked_bytes(&at_source), Some(THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG));
    assert!(reason_the_tree_is_too_small(&at_source).is_none(), "a tree of the pinned bytes is cloned");
    crate::test_support::remove(at.as_path());
    let below: PathBuf = fixture("limit-below-the-size");
    let below_source: SnapshotSource = source_of(below.as_path(), &[("one.bin", THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG - 1)]);
    let reason: String = reason_the_tree_is_too_small(&below_source).expect("one byte fewer is declined");
    assert!(reason.contains("sum to"), "{reason}");
    assert!(reason.contains((THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG - 1).to_string().as_str()), "{reason}");
    crate::test_support::remove(below.as_path());
}

/// What:
///  A source at the cut is cloned,
/// one changed path more is declined,
/// and a source whose dirt is only untracked is cloned.
/// Why:
///   The cut counts the paths a checkout has to write,
/// and an untracked file is not one of them:
/// a
///  limit that counted Git's whole status would decline the machine's main checkout on most days,
///  where untracked build output outnumbers the changed files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reason(source(tracked, cut + 1))).toContain('have changes'); expect(reason(source(tracked, 0, untracked + 3))).toBeUndefined();
/// ```
#[test]
fn the_cut_declines_one_changed_path_more_than_its_share_and_ignores_untracked_ones() {
    let root: PathBuf = fixture("limit-changed-files");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    let files: usize = 8;
    for index in 0..files {
        tracked(repo.as_path(), format!("file-{index}.txt").as_str(), b"the commit's bytes\n");
    }
    commit_all(repo.as_path(), "fixture");
    let cut: usize = the_changed_file_cut(files);
    assert!(reason_too_many_changed_paths(&source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source")).is_none(), "a clean source is cloned");
    for index in 0..cut {
        std::fs::write(repo.join(format!("file-{index}.txt").as_str()), b"a change\n").expect("the change");
    }
    let at_cut: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
    assert_eq!(at_cut.dirty.as_ref().expect("the changes").tracked, cut);
    assert!(reason_too_many_changed_paths(&at_cut).is_none(), "exactly the cut is cloned: the comparison is strict");
    std::fs::write(repo.join(format!("file-{cut}.txt").as_str()), b"a change\n").expect("one more change");
    // Three files nobody tracks: they are changes Git's status lists and no worktree takes.
    for index in 0..3 {
        std::fs::write(repo.join(format!("untracked-{index}.txt").as_str()), b"nobody tracks this\n").expect("the untracked file");
    }
    let beyond: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let changes = beyond.dirty.as_ref().expect("the changes");
    assert_eq!(changes.tracked, cut + 1, "the untracked files are no tracked change");
    assert_eq!(changes.count, cut + 4, "Git's status lists them all");
    let reason: String = reason_too_many_changed_paths(&beyond).expect("one changed path more than the cut is declined");
    assert!(reason.contains("have changes"), "{reason}");
    assert!(reason.contains(format!("{} of the {} tracked files", cut + 1, files).as_str()), "{reason}");
    crate::test_support::remove(root.as_path());
}
