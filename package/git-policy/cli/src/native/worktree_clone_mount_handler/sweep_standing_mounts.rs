//! What:
//!  How one sweep answers whether a clone's own mount already stands at its
//!  slot:
//!  one read of the kernel's mount table,
//!  refreshed when the namespace's poll says the table changed.
//! Why:
//!  Reading the whole table for every standing child made a steady sweep cost
//!  grow faster than the child count,
//!  and answering from a table that was never refreshed let a mount whose flags
//!  changed or whose number was reused stand as the clone's own.
//!  The kernel raises its namespace event after a mount,
//!  an unmount,
//!  a move and a remount or `mount_setattr` that changes flags,
//!  so a poll before every answer closes every change to what the predicate
//!  reads.
//!  A change between the poll and the answer is the same window a fresh read
//!  has between its read and its answer;
//!  a mount id the table lacks is still read live.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = LiveStandingMounts.read().standingMountOf(request, clone);
//! ```

use crate::diagnostic_log::debug;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;
use crate::worktree_clone_mount::{LookAgainAtSlotRequest, MountIdentity, look_again_at_the_slot, mount_facts_of};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "sweepStandingMounts";

/// What:
///  Whether one mount is this clone's own rather than a bind or another
///  dataset's mount.
/// Why:
///  A bind of the same dataset prints the same source,
///  and the route's identity rule tells a mount from another mount of the same
///  dataset by its mount ID and a bind by its parent mount and its source
///  (`src/native/worktree_clone_mount/attached_mount_identity.rs`).
///  The table here has no parent mount of the attach to compare,
///  so the bind is told by the root the mount shows and by the two attributes
///  the attach itself enforces:
///  a dataset's own mount shows root `/`,
///  and a mount this route made carries `nosuid` and `nodev`.
///  Anything else is left to the attach step,
/// whose walk refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = isThisClonesOwnMount(facts, clone); // source, root, type and both flags
/// ```
pub(crate) fn is_this_clones_own_mount(facts: &MountFacts, clone: &str) -> bool {
    return facts.source == clone
        && facts.filesystem_type == ZFS_FILE_SYSTEM_TYPE
        && facts.root == Path::new("/")
        && facts.has_option("nosuid")
        && facts.has_option("nodev");
}

/// What:
///  What the canonical slot holds before this clone's attach.
/// Why:
///  The ordinary sweep runs over a clone whose own mount still stands at the
///  slot,
///  and recognising that mount needs the kernel's table,
///  which a control on a person's machine cannot mount into;
///  the seam is what makes the recognition provable here and the guest proves
///  the table read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface StandingMounts { standingMountOf(request, clone): MountIdentity | undefined }
/// ```
pub trait StandingMounts {
    /// The mount the canonical slot holds when it is this clone's own,
    /// or nothing when the slot holds no mount or another dataset's.
    fn standing_mount_of(&self, request: &LookAgainAtSlotRequest<'_>, clone: &str) -> Option<MountIdentity>;
}

/// What:
///  One source of the kernel's mount table and the signal that the table
///  changed.
/// Why:
///  A control needs to answer with a table it built,
///  and the release source is the kernel's own file:
///  opening it makes the kernel remember the namespace's event number,
///  and `poll` reports a later event as `POLLPRI` or `POLLERR`.
///  The descriptor is opened before the first table read,
///  so a change between the open and the read fires the next poll rather than
///  being missed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MountTableSource { changed(): boolean; read(): MountFacts[] }
/// ```
pub(crate) trait MountTableSource {
    /// Whether this namespace's mount table changed since the last poll.
    fn changed(&self) -> bool;

    /// The whole table,
    /// bounded as the route reads it.
    fn read(&self) -> Result<Vec<MountFacts>, String>;
}

/// What:
///  The release source:
///  `/proc/self/mountinfo`,
///  held open for its poll and read by path when it fires.
/// Why:
///  The kernel's `mounts_poll` sets `POLLPRI|POLLERR` when the namespace's
///  event counter differs from the one the open remembered,
///  and `fs/namespace.c` raises that counter after a mount,
///  an unmount,
///  a move and a `mount_setattr` or remount that changes flags,
///  so every change to what the own-mount predicate reads raises it.
///  A file that cannot be opened leaves the poll quiet,
///  and the read still tries by path,
///  which is the answer the route had before the snapshot existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = LiveMountTableSource.open();
/// ```
pub(crate) struct LiveMountTableSource {
    /// The descriptor the kernel's poll watches.
    descriptor: Option<std::fs::File>,
}

impl LiveMountTableSource {
    /// Open the descriptor before the first read,
    /// so a change after this call fires the next poll.
    pub(crate) fn open() -> Self {
        let path: &Path = Path::new(crate::fs_mount_options::MOUNTINFO_PATH);
        let descriptor: Option<std::fs::File> = match std::fs::File::open(path) {
            Ok(file) => Some(file),
            Err(error) => {
                debug(TAG, format!("the mount table {path:?} could not be opened for its poll: {error}").as_str());
                None
            }
        };
        return Self { descriptor };
    }
}

impl MountTableSource for LiveMountTableSource {
    fn changed(&self) -> bool {
        let Some(file) = &self.descriptor else {
            return false;
        };
        let mut watched: libc::pollfd = libc::pollfd { fd: file.as_raw_fd(), events: libc::POLLPRI | libc::POLLERR, revents: 0 };
        // SAFETY: `watched` is one initialised `pollfd`, the call writes only its `revents` field, and the timeout is
        // zero so it never waits.
        let ready: libc::c_int = unsafe { libc::poll(&mut watched, 1, 0) };
        return ready > 0 && (watched.revents & (libc::POLLPRI | libc::POLLERR)) != 0;
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        let path: &Path = Path::new(crate::fs_mount_options::MOUNTINFO_PATH);
        let bytes: Vec<u8> = crate::plain_file::read_plain_file(path, crate::plain_file::MOUNT_TABLE_LIMIT)
            .map_err(|error| return format!("the mount table {path:?} could not be read: {error}"))?;
        return Ok(crate::fs_mount_options::parse_mount_table(bytes.as_slice()));
    }
}

/// What:
///  The release reader:
///  the mount table one sweep read,
///  refreshed when the kernel says the namespace's table changed,
///  and the canonical slot's mount named as this clone's own only when its
///  source is the clone's dataset.
/// Why:
///  A slot holding another clone's mount or a bind answers nothing here,
///  and the attach step's own walk refuses it as before.
///  Reading the whole table for every standing child made a steady sweep cost
///  grow faster than the child count,
///  because each child scanned the same table again;
///  one table for the run makes that work independent of how many children
///  stand.
///  The table is not trusted while the namespace changes:
///  the poll is checked before every answer,
///  and a firing re-reads the whole table.
///  A change between the poll and the answer is the same window `main` has
///  between its read and its answer,
///  and a mount id the table does not hold is read live,
///  so a mount another sweep made during the run is judged on the live table
///  rather than on the snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = LiveStandingMounts.read().standingMountOf(request, clone);
/// ```
pub struct LiveStandingMounts {
    /// The kernel table and the signal that it changed.
    source: Box<dyn MountTableSource>,
    /// The kernel's mount table as this sweep last read it,
    /// by mount id.
    table: RefCell<HashMap<u64, MountFacts>>,
    /// How many times this sweep read the table.
    reads: Cell<usize>,
}

impl LiveStandingMounts {
    /// Read the sweep's first copy of the kernel's mount table.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const standing = LiveStandingMounts.read();
    /// ```
    pub fn read() -> Self {
        return Self::from_source(Box::new(LiveMountTableSource::open()));
    }

    /// Build one reader over a source a control owns.
    pub(crate) fn from_source(source: Box<dyn MountTableSource>) -> Self {
        let standing: Self = Self { source, table: RefCell::new(HashMap::new()), reads: Cell::new(0) };
        standing.refresh();
        return standing;
    }

    /// Read the whole table again,
    /// clearing it when the read refuses so every child falls back to its own
    /// live read.
    fn refresh(&self) {
        self.reads.set(self.reads.get() + 1);
        match self.source.read() {
            Ok(read) => {
                *self.table.borrow_mut() = read.into_iter().map(|mount| return (mount.mount_id, mount)).collect();
            }
            Err(error) => {
                // A table the run cannot read leaves the snapshot empty, and each standing answer falls back to the
                // child's own read, which is the answer the route gave before the snapshot existed.
                debug(TAG, format!("the mount table could not be read for this sweep: {error}").as_str());
                self.table.borrow_mut().clear();
            }
        }
    }

    /// Re-read the table when the kernel says this namespace's table changed.
    fn refresh_if_changed(&self) {
        if self.source.changed() {
            debug(TAG, "the mount table of this namespace changed, so this sweep reads it again");
            self.refresh();
        }
    }

    /// How many times this sweep has read the table,
    /// which a control uses to prove one read for many standing children.
    #[cfg(test)]
    pub(crate) fn table_reads(&self) -> usize {
        return self.reads.get();
    }

    /// One mount's facts from this sweep's table,
    /// or the live table when the run did not see the mount.
    pub(crate) fn facts_of(&self, mount_id: u64) -> Result<MountFacts, String> {
        self.refresh_if_changed();
        let table = self.table.borrow();
        if let Some(facts) = table.get(&mount_id) {
            return Ok(facts.clone());
        }
        drop(table);
        return mount_facts_of(mount_id).map_err(|refusal| return refusal.message());
    }
}

impl StandingMounts for LiveStandingMounts {
    fn standing_mount_of(&self, request: &LookAgainAtSlotRequest<'_>, clone: &str) -> Option<MountIdentity> {
        let fresh = match look_again_at_the_slot(request) {
            Ok(fresh) => fresh,
            Err(refusal) => {
                debug(TAG, format!("the canonical slot could not be read for `{clone}`: {}", refusal.message()).as_str());
                return None;
            }
        };
        let facts = match self.facts_of(fresh.mount_id) {
            Ok(facts) => facts,
            Err(said) => {
                debug(TAG, format!("the mount {} the slot is on could not be read: {said}", fresh.mount_id).as_str());
                return None;
            }
        };
        if !is_this_clones_own_mount(&facts, clone) {
            return None;
        }
        return Some(MountIdentity {
            mount_id: fresh.mount_id,
            parent_mount_id: facts.parent_id,
            mount_point: facts.mount_point,
            source: facts.source,
            file_system_type: facts.filesystem_type,
            device: fresh.slot_identity.device,
            inode: fresh.slot_identity.inode,
            observed_path: fresh.observed_path,
        });
    }
}
