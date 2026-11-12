//! What:
//!  Controls for the words of a removal that went around what it did not make,
//!  and for the record a later command shows.
//! Why:
//!   The removal changed files and forgot a worktree,
//!  so the report is the only thing that says the path is still taken and what
//!  frees it;
//! a report that named the worktree alone would read as a finished job,
//! and one that named
//!  `btrfs subvolume delete` would send a person to destroy what a volume holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(aroundRemovalReport('/repo/wt', left)).toContain('Git no longer lists it');
//! ```

use super::*;
use crate::subvolume_remove_walk::{A_MOUNT_POINT, ANOTHER_FILESYSTEM, Boundary};
use crate::test_support::{fixture, remove, shown_notices};

/// The two volumes a control works with:
///  a nested subvolume and a mount point,
///  at paths a report reads.
fn two_volumes(target: &Path) -> Vec<Boundary> {
    return vec![
        Boundary { path: target.join("snap"), what: ANOTHER_FILESYSTEM },
        Boundary { path: target.join("mnt/data"), what: A_MOUNT_POINT },
    ];
}

/// What:
///  The report of a removal that went around names each volume,
///  what it is,
///  where it stands,
///  that the worktree is otherwise gone and forgotten,
///  and what frees the path.
/// Why:
///   The path is still taken and the person has to be able to act on that at once.
#[test]
fn the_report_of_a_removal_that_went_around_names_every_one_and_what_frees_the_path() {
    let root: PathBuf = fixture("around-report");
    let target: PathBuf = root.join("wt");
    let left: Vec<Boundary> = two_volumes(target.as_path());
    let report: String = around_removal_report(target.as_path(), left.as_slice());
    for found in &left {
        assert!(report.contains(found.path.to_str().expect("UTF-8")), "the report names {}: {report}", found.path.display());
        assert!(report.contains(found.what), "and what it is: {report}");
    }
    assert!(report.contains("Git no longer lists it"), "it says Git forgot the worktree: {report}");
    assert!(report.contains("left where it stands"), "and that what was not the removal's stayed: {report}");
    assert!(report.contains("Nothing of what is left was moved, unmounted or deleted"), "and that none of it was touched: {report}");
    assert!(report.contains("free for `git worktree add`"), "and what frees the path: {report}");
    assert!(report.contains("umount"), "a mount point's own way on is named: {report}");
    assert!(report.contains("git worktree remove"), "and another repository's worktree's: {report}");
    assert!(!report.contains("btrfs subvolume delete"), "and nothing that destroys what a volume holds: {report}");
    remove(root.as_path());
}

/// What:
///  The refusal of a removal beside a volume says nothing was removed,
///  names each volume,
///  gives the reason Git refuses,
///  and gives both ways on,
///  with `--force` promising what it does.
/// Why:
///   A person who reads "refused" needs the two commands that end the state,
///  and `--force` through this wrapper keeps the volume while real Git's
///  would delete what is mounted there.
#[test]
fn the_refusal_beside_a_volume_names_both_ways_on() {
    let root: PathBuf = fixture("around-refusal");
    let target: PathBuf = root.join("wt");
    let left: Vec<Boundary> = two_volumes(target.as_path());
    let report: String = refused_beside_a_volume_report(target.as_path(), "../wt", left.as_slice(), "notes.txt is untracked and not ignored");
    assert!(report.contains("nothing was removed"), "the report says so: {report}");
    for found in &left {
        assert!(report.contains(found.path.to_str().expect("UTF-8")), "and names {}: {report}", found.path.display());
    }
    assert!(report.contains("notes.txt is untracked and not ignored"), "and the reason Git refuses: {report}");
    assert!(report.contains("git worktree remove --force ../wt"), "and the forced command as the person typed the worktree: {report}");
    assert!(report.contains("leaves what is named above exactly as it is"), "saying what that command keeps: {report}");
    assert!(report.contains("free for `git worktree add`"), "and what frees the path: {report}");
    remove(root.as_path());
}

/// What:
///  A removal that stopped halfway says which of the two things happened,
///  which is what a half-emptied worktree needs,
///  and names the reason.
/// Why:
///   "Removed" and "nothing was removed" are both wrong there,
///  and a person would act on either.
#[test]
fn the_report_of_a_removal_that_stopped_halfway_says_so() {
    let root: PathBuf = fixture("around-stopped");
    let target: PathBuf = root.join("wt");
    let report: String = stopped_around_report(target.as_path(), "wt/a cannot be removed (Permission denied)");
    assert!(report.contains("stopped before it finished"), "the report says so: {report}");
    assert!(report.contains("still registered"), "and that the worktree is still in Git's records: {report}");
    assert!(report.contains("Permission denied"), "and what stopped it: {report}");
    assert!(report.contains("run the same removal again"), "and what finishes it: {report}");
    remove(root.as_path());
}

/// What:
///  The line the landed sweep prints names the landed branch,
///  the worktree,
///  its branch,
///  and what stayed,
///  and never reads as a plain removal.
/// Why:
///  The sweep removed a worktree nobody asked it to remove,
///  and the path it leaves taken is
///  the person's next step.
#[test]
fn the_landed_line_names_the_branch_and_what_was_left() {
    let root: PathBuf = fixture("around-landed-line");
    let worktree: PathBuf = root.join("wt");
    let left: Vec<Boundary> = two_volumes(worktree.as_path());
    let line: String = landed_around_line("main", worktree.as_path(), "refs/heads/feature", left.as_slice());
    assert!(line.contains("is now on main"), "the landed branch is named: {line}");
    assert!(line.contains(worktree.to_str().expect("UTF-8")), "and the worktree: {line}");
    assert!(line.contains("refs/heads/feature"), "and its own branch: {line}");
    assert!(line.contains("Git no longer lists it"), "and that Git forgot it: {line}");
    for found in &left {
        assert!(line.contains(found.path.to_str().expect("UTF-8")), "and each volume: {line}");
    }
    assert!(line.contains("The branch stays"), "and that the branch stays: {line}");
    remove(root.as_path());
}

/// What:
///  The line a later add prints connects Git's refusal to the volume a removal left,
///  and says what frees the path.
/// Why:
///  Git's own sentence for an occupied directory says only that the directory exists.
#[test]
fn the_add_line_names_what_a_removal_left() {
    let root: PathBuf = fixture("around-add-line");
    let destination: PathBuf = root.join("wt");
    let left: Vec<Boundary> = two_volumes(destination.as_path());
    let line: String = add_beside_a_volume_line(destination.as_path(), left.as_slice());
    assert!(line.contains("is not empty because a removal of the worktree there had to leave"), "the line says why: {line}");
    for found in &left {
        assert!(line.contains(found.path.to_str().expect("UTF-8")), "and names each volume: {line}");
    }
    assert!(line.contains("free for `git worktree add`"), "and what frees the path: {line}");
    remove(root.as_path());
}

/// What:
///  A record of a removal that went around is shown once,
///  in the words the sweep used,
///  and is gone after that.
/// Why:
///   The record is the only thing left that speaks for a worktree Git no longer knows,
///  and a notice that stayed would be shown on every command for ever.
#[test]
fn a_record_of_a_removal_that_went_around_is_shown_once() {
    let root: PathBuf = fixture("around-record");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("a common Git directory");
    let worktree: PathBuf = root.join("wt");
    let left: Vec<Boundary> = two_volumes(worktree.as_path());
    record_left_in_place(common.as_path(), worktree.as_path(), "refs/heads/feature", left.as_slice()).expect("the record is written");
    assert!(left_in_place_directory(common.as_path()).join(record_name(worktree.as_path())).is_file(), "the record is where the reader looks");

    let shown: Vec<String> = shown_notices(left_in_place_notices(common.as_path()));
    assert_eq!(shown.len(), 1, "one record, one line: {shown:?}");
    assert!(shown[0].contains(worktree.to_str().expect("UTF-8")), "the words name the worktree: {}", shown[0]);
    assert!(shown[0].contains("of branch refs/heads/feature"), "and the branch it held: {}", shown[0]);
    for found in &left {
        assert!(shown[0].contains(found.path.to_str().expect("UTF-8")), "and each volume: {}", shown[0]);
    }
    assert!(shown[0].contains("free for `git worktree add`"), "and what frees the path: {}", shown[0]);

    assert!(left_in_place_notices(common.as_path()).is_empty(), "a record whose words are out is taken");
    remove(root.as_path());
}

/// What:
///  A file in the records' directory that is not one of them is neither shown nor moved.
/// Why:
///   The directory is read by every command,
///  and guessing at a person's file would print their text as this program's sentence
///  and take it out of its own name.
#[test]
fn a_file_that_is_no_record_is_left_alone() {
    let root: PathBuf = fixture("around-record-stranger");
    let common: PathBuf = root.join("common");
    let directory: PathBuf = left_in_place_directory(common.as_path());
    std::fs::create_dir_all(directory.as_path()).expect("the records' directory");
    let stranger: PathBuf = directory.join("left-0000.txt");
    std::fs::write(stranger.as_path(), b"a person's own file\n").expect("a file nobody can read as a record");
    assert!(left_in_place_notices(common.as_path()).is_empty(), "nothing is shown");
    assert_eq!(std::fs::read(stranger.as_path()).expect("the file"), b"a person's own file\n", "and the file is where it was");
    remove(root.as_path());
}
