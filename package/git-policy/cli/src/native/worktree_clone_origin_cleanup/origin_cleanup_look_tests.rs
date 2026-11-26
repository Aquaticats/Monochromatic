//! What:
//!  The controls of the look:
//!  the healthy empty folder,
//!  one record that finishes,
//!  one that blocks and speaks once,
//!  a foreign entry,
//!  and a record this reader refuses.
//! Why:
//!  The look destroys a snapshot and the project fixed when it prints,
//!  so each control reads both the folder's state and the lines a person sees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theOriginCleanupLook({ commonDir, reads, device, now });
//! expect(answer.lines).toHaveLength(1);
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::VisiblePool;
use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::{
    OWN,
    an_owed_record,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_look::{
    LookAnswer,
    OriginCleanupLook,
    the_origin_cleanup_look,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    NvList,
};
use std::cell::{
    Cell,
    RefCell,
};
use std::path::PathBuf;
use std::rc::Rc;

/// The moment every look records.
const NOW: &str = "2026-11-25T09:15:01-05:00";

/// One throwaway common directory with the cleanup folder made.
fn a_folder(name: &str) -> (PathBuf, CleanupFolder) {
    let common: PathBuf = fixture(&format!("origin-cleanup-look-{name}"));
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    return (common, folder);
}

#[test]
fn a_repository_with_no_cleanup_folder_prints_nothing_and_opens_nothing() {
    let common: PathBuf = fixture("origin-cleanup-look-absent");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    let pool: VisiblePool = VisiblePool::holding();
    let device: OneDestroy = OneDestroy {
        present: Rc::clone(&pool.present),
        sent: RefCell::new(Vec::new()),
    };
    let answer: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert!(answer.lines.is_empty(), "the healthy case says nothing");
    assert_eq!(answer.finished + answer.blocked + answer.refused, 0);
    assert!(device.sent.borrow().is_empty());
}

#[test]
fn a_free_record_is_destroyed_and_its_record_and_lines_end() {
    let (common, folder) = a_folder("finished");
    folder
        .publish(&an_owed_record())
        .expect("the record is written");
    let pool: VisiblePool = VisiblePool::free();
    let device: OneDestroy = OneDestroy {
        present: Rc::clone(&pool.present),
        sent: RefCell::new(Vec::new()),
    };
    let answer: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert_eq!(answer.finished, 1);
    assert_eq!(answer.lines.len(), 1, "one line for the destruction");
    assert!(answer.lines[0].contains("destroyed the snapshot"));
    assert!(
        std::fs::read_dir(folder.path())
            .expect("the folder lists")
            .next()
            .is_none(),
        "the record is gone"
    );
}

#[test]
fn a_blocked_record_is_kept_and_speaks_once() {
    let (common, folder) = a_folder("blocked");
    folder
        .publish(&an_owed_record())
        .expect("the record is written");
    let pool: VisiblePool = VisiblePool::pinned();
    let device: OneDestroy = OneDestroy {
        present: Rc::clone(&pool.present),
        sent: RefCell::new(Vec::new()),
    };
    let first: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert_eq!(first.blocked, 1);
    assert_eq!(
        first.lines.len(),
        1,
        "an owed record that becomes blocked speaks"
    );
    assert!(first.lines[0].contains("it is still pinned"));
    assert!(
        device.sent.borrow().is_empty(),
        "no request is sent to a pinned snapshot"
    );
    let second: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert_eq!(second.blocked, 1);
    assert!(
        second.lines.is_empty(),
        "the same reason read again says nothing"
    );
    let kept: OriginCleanupRecord =
        crate::worktree_clone_origin_cleanup::origin_cleanup_reading::read_the_cleanup_record(
            common.as_path(),
            OWN,
        )
        .expect("the record stands");
    assert_eq!(
        kept.block.map(|held| return held.reason),
        Some(BlockReason::Dependency)
    );
}

#[test]
fn a_foreign_entry_is_reported_and_never_read() {
    let (common, folder) = a_folder("foreign");
    std::fs::write(folder.path().join("notes.txt"), b"x").expect("a foreign entry");
    let pool: VisiblePool = VisiblePool::free();
    let device: OneDestroy = OneDestroy {
        present: Rc::clone(&pool.present),
        sent: RefCell::new(Vec::new()),
    };
    let answer: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert_eq!(answer.refused, 1);
    assert!(answer.lines[0].contains("notes.txt"));
    assert!(answer.lines[0].contains("cannot be used"));
}

#[test]
fn a_record_the_reader_refuses_is_named_and_left() {
    let (common, folder) = a_folder("unreadable");
    std::fs::write(folder.path().join(OWN), b"not a record\n").expect("the bytes are written");
    let pool: VisiblePool = VisiblePool::free();
    let device: OneDestroy = OneDestroy {
        present: Rc::clone(&pool.present),
        sent: RefCell::new(Vec::new()),
    };
    let answer: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert_eq!(answer.refused, 1);
    assert!(answer.lines[0].contains("cannot be used"));
    assert!(
        std::fs::symlink_metadata(folder.path().join(OWN).as_path()).is_ok(),
        "the record is left as it was"
    );
    assert!(device.sent.borrow().is_empty());
}

#[test]
fn a_block_whose_counts_changed_speaks_again() {
    let (common, folder) = a_folder("changed");
    let blocked: OriginCleanupRecord = an_owed_record().blocked(
        CleanupBlock {
            reason: BlockReason::Dependency,
            errno: 0,
            clones: 1,
            holds: 0,
        },
        NOW,
    );
    folder
        .publish(&blocked)
        .expect("the blocked record is written");
    let pool: VisiblePool = VisiblePool::pinned_by_two();
    let device: OneDestroy = OneDestroy {
        present: Rc::clone(&pool.present),
        sent: RefCell::new(Vec::new()),
    };
    let answer: LookAnswer = the_origin_cleanup_look(&OriginCleanupLook {
        common_dir: common.as_path(),
        reads: &pool,
        device: &device,
        now: NOW,
    });
    assert_eq!(answer.blocked, 1);
    assert_eq!(answer.lines.len(), 1, "one clone becoming two speaks");
    assert!(answer.lines[0].contains("2 clone(s)"));
}

/// A device that answers success and records what it was asked for.
struct OneDestroy {
    /// The pool's listed flag,
    /// which a successful destroy clears.
    present: Rc<Cell<bool>>,
    /// Every name the device was asked to destroy.
    sent: RefCell<Vec<String>>,
}

impl SnapshotDestroyDevice for OneDestroy {
    fn destroy_snaps(
        &self,
        _pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        self.sent
            .borrow_mut()
            .extend(names.iter().map(|name| return String::from(*name)));
        self.present.set(false);
        return Ok(None);
    }
}
