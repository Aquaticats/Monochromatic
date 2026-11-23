//! Controls for the clone rung's provisional limits:
//! the changed-file cut is the fence's own share of the tracked paths,
//! the whole-tree sum judges a source with changes,
//! the mean file size judges a clean source,
//! and each reason appears exactly at the edge it was pinned at.

use super::*;
use crate::fs_kind::{FilesystemKind, filesystem_kind};
use crate::worktree_slow_route::{DECLINED_LEAD, explain_decline};
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
///  A source of the given tracked files and sizes,
/// read through the source check,
/// so a control asks the limits what the ladder asks them.
/// Why:
///   Both reasons read the index the source check copied and the changes it listed,
/// and a control that built
///  those by hand would not show what an add sees.
/// The files are sparse:
/// the limit reads the sizes the index records,
/// never the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = checkSource(repo); // the index, the tree of HEAD and the changes
/// ```
fn source_of(root: &std::path::Path, files: &[(String, u64)]) -> SnapshotSource {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    for (name, bytes) in files {
        let path: PathBuf = repo.join(name.as_str());
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        let file: std::fs::File = std::fs::File::create(path.as_path()).expect("the file");
        file.set_len(*bytes).expect("the file's length");
        // Older than the index the commit writes, so the clean check proves the file instead of calling it racy.
        crate::test_support::age(path.as_path());
    }
    commit_all(repo.as_path(), "fixture");
    return source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
}

/// The index entries of a source,
/// as the ladder reads them.
fn entries_of(source: &SnapshotSource) -> Vec<StatEntry> {
    return tracked_entries(source).expect("the source's index entries");
}

/// What:
///  A source with changes of exactly the pinned bytes is cloned,
/// and one byte fewer is declined,
/// with the sum in the sentence.
/// Why:
///   The whole-tree limit's edge is what the measurement was taken at,
/// so the control is the arithmetic beside it:
/// a
///  limit that said "at or below" or that summed something else would show here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reason(tree(THE_FEWEST))).toBeUndefined(); expect(reason(dirtyTree(THE_FEWEST - 1))).toContain('sum to');
/// ```
#[test]
fn the_size_limit_declines_one_byte_below_the_pinned_bytes_for_a_source_with_changes() {
    let at: PathBuf = fixture("limit-at-the-size");
    let at_source: SnapshotSource = source_of(at.as_path(), &[(String::from("one.bin"), THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG)]);
    assert_eq!(tracked_bytes(entries_of(&at_source).as_slice()), THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG);
    assert!(reason_the_tree_is_too_small(&at_source, entries_of(&at_source).as_slice()).is_none(), "a tree of the pinned bytes is cloned");
    crate::test_support::remove(at.as_path());
    let below: PathBuf = fixture("limit-below-the-size");
    let below_source: SnapshotSource = source_of(below.as_path(), &[(String::from("one.bin"), THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG - 1)]);
    let reason: String = reason_the_tree_is_too_small(&below_source, entries_of(&below_source).as_slice()).expect("one byte fewer is declined");
    assert!(reason.contains("sum to"), "{reason}");
    assert!(reason.contains((THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG - 1).to_string().as_str()), "{reason}");
    crate::test_support::remove(below.as_path());
}

/// What:
///  A clean source whose mean tracked file is exactly the pinned edge is cloned,
/// and one byte off the total declines it,
/// with the mean in the sentence;
/// on a filesystem with no edge, a clean tree of the same tiny files is cloned.
/// Why:
///   A clean source is judged by its mean file size,
/// so the control is that arithmetic:
/// the total divided by the paths,
/// floored,
/// and compared with the edge.
/// btrfs has no edge (the project's "No size rule on btrfs"),
/// so there is no value there to subtract one from,
/// and the control says so instead of underflowing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(mean(tree(4, EDGE))).toBe(EDGE); expect(reason(tree(4, EDGE) with one byte fewer)).toContain('average');
/// if (EDGE === undefined) expect(reason(tree(4, 1))).toBeUndefined();
/// ```
#[test]
fn the_mean_limit_declines_one_byte_below_the_pinned_mean_and_clones_it() {
    let files: usize = 4;
    let at: PathBuf = fixture("limit-at-the-mean");
    let at_kind: FilesystemKind = filesystem_kind(at.as_path()).expect("the fixture's filesystem");
    let Some(edge): Option<u64> = the_smallest_mean_tracked_file(at_kind) else {
        // A filesystem with no mean edge clones every clean tree, the tiniest included.
        let tiny: Vec<(String, u64)> = (0..files).map(|index: usize| { return (format!("file-{index}.txt"), 1); }).collect();
        let tiny_source: SnapshotSource = source_of(at.as_path(), tiny.as_slice());
        assert!(reason_the_mean_file_is_too_small(&tiny_source, entries_of(&tiny_source).as_slice()).is_none(), "a filesystem with no edge declines no clean tree by its mean");
        assert!(decline_before_cloning(&tiny_source, entries_of(&tiny_source).as_slice()).is_none(), "and the ladder clones it");
        crate::test_support::remove(at.as_path());
        return;
    };
    let at_files: Vec<(String, u64)> = (0..files).map(|index: usize| { return (format!("file-{index}.txt"), edge); }).collect();
    let at_source: SnapshotSource = source_of(at.as_path(), at_files.as_slice());
    assert_eq!(mean_tracked_file(entries_of(&at_source).as_slice()), Some(edge), "a tree of the edge's mean");
    assert!(reason_the_mean_file_is_too_small(&at_source, entries_of(&at_source).as_slice()).is_none(), "a tree of the pinned mean is cloned");
    crate::test_support::remove(at.as_path());
    let below: PathBuf = fixture("limit-below-the-mean");
    let below_files: Vec<(String, u64)> = (0..files).map(|index: usize| { return (format!("file-{index}.txt"), if index == 0 { edge - 1 } else { edge }); }).collect();
    let below_source: SnapshotSource = source_of(below.as_path(), below_files.as_slice());
    let mean: u64 = mean_tracked_file(entries_of(&below_source).as_slice()).expect("a mean");
    assert!(mean < edge, "one byte off the total is under the edge: {mean}");
    let reason: String = reason_the_mean_file_is_too_small(&below_source, entries_of(&below_source).as_slice()).expect("one byte fewer of the mean is declined");
    assert!(reason.contains("average"), "{reason}");
    assert!(reason.contains(mean.to_string().as_str()), "{reason}");
    crate::test_support::remove(below.as_path());
}

/// What:
///  The limits judge a clean source by its mean file size and a source with changes by its whole-tree sum.
/// Why:
///   This is the decision of 2026-11-22 built:
/// a clean tree of a few large files clones whatever its total,
/// and a small clean tree takes Git's own checkout even though its total is far above nothing,
/// while a source with changes at the same small sizes is judged by the sum.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(decline(clean(fewLarge))).toBeUndefined(); expect(decline(clean(small))).toContain('average'); expect(decline(dirty(small))).toContain('sum to');
/// ```
#[test]
fn a_clean_source_is_judged_by_its_mean_file_and_a_source_with_changes_by_its_sum() {
    let few_large: PathBuf = fixture("limit-clean-few-large");
    // A mean well above any edge (or any file size at all on a filesystem with no edge), and a total far below the
    // whole-tree pin: the shape that must clone and that the whole-tree rule declined.
    let per_file: u64 = the_safest_mean_tracked_file() * 8;
    let large_files: Vec<(String, u64)> = vec![(String::from("first.bin"), per_file), (String::from("second.bin"), per_file)];
    let large_source: SnapshotSource = source_of(few_large.as_path(), large_files.as_slice());
    assert!(tracked_bytes(entries_of(&large_source).as_slice()) < THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG, "the total is far below the whole-tree pin");
    assert!(large_source.dirty.is_none(), "the few large files are a clean source");
    assert!(decline_before_cloning(&large_source, entries_of(&large_source).as_slice()).is_none(), "a clean tree of few large files clones");
    crate::test_support::remove(few_large.as_path());
    let small: PathBuf = fixture("limit-clean-small");
    let small_source: SnapshotSource = source_of(small.as_path(), &[(String::from("one.txt"), 1), (String::from("two.txt"), 1)]);
    let small_kind: FilesystemKind = filesystem_kind(small.as_path()).expect("the fixture's filesystem");
    match the_smallest_mean_tracked_file(small_kind) {
        Some(_edge) => {
            let reason: String = decline_before_cloning(&small_source, entries_of(&small_source).as_slice()).expect("a clean tree of tiny files is declined");
            assert!(reason.contains("average"), "the clean source's decline is about its mean: {reason}");
            assert!(!reason.contains("sum to"), "the whole-tree sum says nothing about a clean source: {reason}");
        }
        None => {
            assert!(decline_before_cloning(&small_source, entries_of(&small_source).as_slice()).is_none(), "a filesystem with no mean edge clones a clean tree of tiny files");
        }
    }
    // The same two files with one of them changed: now the whole-tree sum decides.
    std::fs::write(small_source.root.join("one.txt"), b"a change\n").expect("the change");
    let dirty_source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(small_source.root.as_path())).expect("a source with changes");
    let sum_reason: String = decline_before_cloning(&dirty_source, entries_of(&dirty_source).as_slice()).expect("a small tree with changes is declined");
    assert!(sum_reason.contains("sum to"), "the source with changes is declined by its sum: {sum_reason}");
    assert!(!sum_reason.contains("average"), "the mean says nothing about a source with changes: {sum_reason}");
    crate::test_support::remove(small.as_path());
}

/// What:
///  ZFS takes its measured mean edge, btrfs none, and a filesystem nobody measured takes the ZFS edge.
/// Why:
///   A mapping that gave btrfs an edge would decline a clean btrfs tree `main` clones,
/// which the recheck measured as 1.36 to 1.62 times `main` (its finding RH1),
/// and a mapping that gave an unmeasured filesystem no edge would clone small files where no round says the copy wins.
/// This control is what pins the mapping, and it runs on a btrfs fixture root as well as on the host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(edge('btrfs')).toBeUndefined(); expect(edge('zfs')).toBe(2600); expect(edge('other')).toBe(2600);
/// ```
#[test]
fn the_mean_edge_is_the_subject_filesystems_own_measurement() {
    assert_eq!(the_smallest_mean_tracked_file(FilesystemKind::Btrfs), None, "the project answered no size rule on btrfs");
    assert_eq!(the_smallest_mean_tracked_file(FilesystemKind::Zfs), Some(THE_SMALLEST_MEAN_TRACKED_FILE_ON_ZFS));
    assert_eq!(the_smallest_mean_tracked_file(FilesystemKind::Overlayfs), Some(the_safest_mean_tracked_file()));
    assert_eq!(the_smallest_mean_tracked_file(FilesystemKind::Other), Some(the_safest_mean_tracked_file()));
    assert_eq!(the_safest_mean_tracked_file(), THE_SMALLEST_MEAN_TRACKED_FILE_ON_ZFS, "the only measured edge a clean source still takes is ZFS's");
}

/// What:
///  Every limit sentence a person can see starts with the decline's own lead and never says the add took longer.
/// Why:
///   The lead was changed because nothing of a declined add had started when a limit declined it,
/// and the recheck's plant set the old lead back and every control stayed green (its finding RL1):
/// a sentence can regress to the snapshot's lead and no test would see it.
/// The three limits are asked here through their own reasons, and the route wraps each as the add tells it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const reason of reasons) expect(describeDecline(reason).line().startsWith(DECLINED_LEAD)).toBe(true);
/// ```
#[test]
fn every_limit_sentence_starts_with_the_decline_lead_and_never_says_it_took_longer() {
    let root: PathBuf = fixture("limit-decline-lead");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    tracked(repo.as_path(), "one.txt", b"the commit's bytes\n");
    tracked(repo.as_path(), "two.txt", b"the commit's bytes\n");
    commit_all(repo.as_path(), "fixture");
    std::fs::write(repo.join("one.txt"), b"a change\n").expect("the change");
    let dirty: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source with changes");
    let entries: Vec<StatEntry> = entries_of(&dirty);
    let mut reasons: Vec<String> = Vec::new();
    reasons.push(reason_the_tree_is_too_small(&dirty, entries.as_slice()).expect("a tree this small is declined by its sum"));
    reasons.push(reason_too_many_changed_paths(&dirty, entries.as_slice()).expect("one changed path of two is over the cut"));
    crate::test_support::remove(root.as_path());
    let clean: PathBuf = fixture("limit-decline-lead-mean");
    let clean_source: SnapshotSource = source_of(clean.as_path(), &[(String::from("one.txt"), 1), (String::from("two.txt"), 1)]);
    let clean_entries: Vec<StatEntry> = entries_of(&clean_source);
    if the_smallest_mean_tracked_file(filesystem_kind(clean.as_path()).expect("the fixture's filesystem")).is_some() {
        reasons.push(reason_the_mean_file_is_too_small(&clean_source, clean_entries.as_slice()).expect("a clean tree of tiny files is declined by its mean"));
    }
    crate::test_support::remove(clean.as_path());
    assert!(!reasons.is_empty(), "the controls built the limits' sentences");
    for reason in &reasons {
        let line: String = explain_decline(reason.as_str()).line();
        assert!(line.starts_with(DECLINED_LEAD), "the sentence starts with the decline's lead: {line}");
        assert!(!line.contains("took longer"), "a declined add started nothing, so nothing took longer: {line}");
    }
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
    let clean: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
    assert!(reason_too_many_changed_paths(&clean, entries_of(&clean).as_slice()).is_none(), "a clean source is cloned");
    for index in 0..cut {
        std::fs::write(repo.join(format!("file-{index}.txt").as_str()), b"a change\n").expect("the change");
    }
    let at_cut: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
    assert_eq!(at_cut.dirty.as_ref().expect("the changes").tracked, cut);
    assert!(reason_too_many_changed_paths(&at_cut, entries_of(&at_cut).as_slice()).is_none(), "exactly the cut is cloned: the comparison is strict");
    std::fs::write(repo.join(format!("file-{cut}.txt").as_str()), b"a change\n").expect("one more change");
    // Three files nobody tracks: they are changes Git's status lists and no worktree takes.
    for index in 0..3 {
        std::fs::write(repo.join(format!("untracked-{index}.txt").as_str()), b"nobody tracks this\n").expect("the untracked file");
    }
    let beyond: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let changes = beyond.dirty.as_ref().expect("the changes");
    assert_eq!(changes.tracked, cut + 1, "the untracked files are no tracked change");
    assert_eq!(changes.count, cut + 4, "Git's status lists them all");
    let reason: String = reason_too_many_changed_paths(&beyond, entries_of(&beyond).as_slice()).expect("one changed path more than the cut is declined");
    assert!(reason.contains("have changes"), "{reason}");
    assert!(reason.contains(format!("{} of the {} tracked files", cut + 1, files).as_str()), "{reason}");
    crate::test_support::remove(root.as_path());
}
