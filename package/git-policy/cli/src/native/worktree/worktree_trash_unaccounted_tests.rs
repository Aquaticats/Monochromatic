//! What:
//!  What the scan that names an unaccounted trash reads,
//!  and what it passes over without reading.
//! Why:
//!   The scan runs on every command in a worktree,
//!  and the queue it reads holds every kind of work:
//! a repository with hundreds of failed pushes and reviews beside one trash record made the scan the largest single
//!  cost of an ordinary command
//!  (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`).

use super::*;
use crate::held_push_release_wakes_the_service::WakeTheService;
use crate::plain_file::take_reads;
use crate::service_work::work_directory;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// How many records of other kinds stand beside the one trash record in the control's queue.
const RECORDS_OF_OTHER_KINDS: usize = 12;

/// A service that is never woken in these controls:
/// nothing the scan did writes a record,
/// so nothing is runnable because of it.
struct NeverWoken;

impl WakeTheService for NeverWoken {
    fn wake(&self, _git_dir: &Path) {
        panic!("the scan writes no record here, so it wakes nothing");
    }
}

/// The name of one object these controls name a record after,
/// and of one trash directory too:
/// forty lowercase hexadecimal digits,
/// which is what a trash is named.
fn object_name(index: usize) -> String {
    return format!("{:040x}", index);
}

/// What:
///  A repository whose queue holds one trash record and `RECORDS_OF_OTHER_KINDS` records of other work,
///  and a worktree whose trash parent is there.
/// Why:
///   That is the shape every command in this repository meets:
///  the scan is asked whether a trash is unaccounted for,
/// and the queue it lists is almost entirely other work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, gitDir, worktree, work } = aQueueOfEveryKind();
/// ```
fn a_queue_of_every_kind(name: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("repository").join(".git");
    let worktree: PathBuf = root.join("repository");
    let work: PathBuf = work_directory(git_dir.as_path());
    let failed: PathBuf = state_directory(work.as_path(), FAILED);
    std::fs::create_dir_all(failed.as_path()).expect("the failed directory");
    std::fs::create_dir_all(worktree.join(TRASH_PARENT)).expect("the worktree's trash parent");
    for index in 0..RECORDS_OF_OTHER_KINDS {
        let object: String = object_name(index);
        for kind in [WorkKind::Push, WorkKind::Maintenance, WorkKind::Review] {
            let record_name: String = format!("{}-{object}", kind.name());
            std::fs::write(failed.join(record_name), "a record this scan cannot use").expect("a record of another kind");
        }
    }
    std::fs::write(failed.join(format!("{}-{}", WorkKind::Trash.name(), object_name(RECORDS_OF_OTHER_KINDS))), "a record this scan cannot read").expect("the trash record");
    return (root, git_dir, worktree, work);
}

/// What:
///  The scan opens the trash records alone.
/// Why:
///   It is the only kind it can use,
///  and a name carries the kind before any bytes are read;
///  opening the rest was fifty times the reads for nothing on a machine where one open can take a second.
#[test]
fn the_scan_reads_the_trash_records_alone() {
    let (root, git_dir, worktree, _work) = a_queue_of_every_kind("trash-unaccounted-reads");
    let mut notices: Vec<u8> = Vec::new();
    let _taken: Vec<PathBuf> = take_reads();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut notices, &NeverWoken);
    let read: Vec<PathBuf> = take_reads();
    assert_eq!(read.len(), 1, "one record of the queue can be this scan's work, so one is read: {read:?}");
    assert_eq!(
        read.first().map(|path| { return path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }); }).flatten(),
        Some(format!("{}-{}", WorkKind::Trash.name(), object_name(RECORDS_OF_OTHER_KINDS))),
        "the record it reads is the trash one: {read:?}"
    );
    assert!(notices.is_empty(), "an empty trash parent leaves nothing to name: {}", String::from_utf8_lossy(notices.as_slice()));
    remove(root.as_path());
}

/// What:
///  A trash record the queue holds for a trash that is there is read and accounted for,
///  so the person is told nothing and no record is written again.
/// Why:
///   Reading the right record is only useful if what it says still settles the trash it names.
#[test]
fn a_trash_record_read_accounts_for_the_trash_it_names() {
    let (root, git_dir, worktree, work) = a_queue_of_every_kind("trash-unaccounted-accounted");
    let oid: String = object_name(RECORDS_OF_OTHER_KINDS);
    assert!(crate::worktree_trash::is_trash_name(oid.as_str()), "the control's trash name is one the scan looks for");
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(oid.as_str());
    std::fs::create_dir_all(trash.as_path()).expect("a trash of the worktree");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Trash,
        repository: worktree.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: oid.clone(),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
    let claimed: PathBuf = state_directory(work.as_path(), CLAIMED).join(item.id());
    std::fs::create_dir_all(claimed.parent().expect("the claimed directory")).expect("the claimed directory");
    std::fs::write(claimed.as_path(), item.encode()).expect("a trash record the service owns");
    let mut notices: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut notices, &NeverWoken);
    assert!(notices.is_empty(), "the record the queue holds is the trash's, so nothing is named: {}", String::from_utf8_lossy(notices.as_slice()));
    assert!(!state_directory(work.as_path(), PENDING).join(item.id()).exists(), "no second record is made");
    remove(root.as_path());
}
