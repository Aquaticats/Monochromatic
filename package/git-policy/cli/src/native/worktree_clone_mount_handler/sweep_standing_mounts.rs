//! What:
//!  How one sweep answers whether a clone's own mount already stands at its
//!  slot:
//!  one read of the kernel's mount table for the mount's shape,
//!  refreshed whenever the namespace's poll says the table changed,
//!  and the mount's source asked of the kernel by that one mount's unique
//!  number.
//! Why:
//!  Reading the whole table for every standing child made a steady sweep cost
//!  grow faster than the child count,
//!  and answering from a table that was never refreshed let a mount whose flags
//!  changed or whose number was reused stand as the clone's own.
//!  The kernel raises its namespace event after a mount,
//!  an unmount,
//!  a move and a remount or `mount_setattr` that changes flags,
//!  so a poll before every answer keeps the copy right about those;
//!  a poll that cannot be made counts as a change,
//!  so a sweep without a descriptor pays the read `main` makes rather than
//!  trusting a copy nothing watches.
//!  One change to a field the answer reads raises no event at all:
//!  renaming a dataset whose mountpoint is `legacy` moves the source the
//!  kernel prints for its mount,
//!  so the copy's source is never trusted for the answer and the source is
//!  asked of the kernel for the one mount at the slot
//!  (`sweep_standing_mount_source.rs`).
//!  Where the kernel cannot answer that,
//!  the live table is read for that mount,
//!  which is the call `main` makes.
//!  A mount id the table does not hold is read live as well.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = LiveStandingMounts.read().standingMountOf(request, clone);
//! ```

use super::sweep_standing_mount_source::{KernelMountSource, MountSourceAnswer, MountSourceByNumber};
use crate::diagnostic_log::debug;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;
use crate::worktree_clone_mount::{FreshSlotLookup, LookAgainAtSlotRequest, MountIdentity, look_again_at_the_slot, mount_facts_of};
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
    return the_facts_are_a_datasets_own_mount(facts) && facts.source == clone;
}

/// What:
///  Whether these facts describe a dataset's own mount at all,
///  whatever dataset it is of.
/// Why:
///  A bind shows another root,
///  another filesystem type is another kind of mount,
///  and the two attributes are the ones the attach itself enforces;
///  the dataset a mount is of is the one field a rename can move without a
///  mount event,
///  so it is asked of the kernel rather than read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const datasetMount = theFactsAreADatasetsOwnMount(facts); // type, root and both flags
/// ```
pub(crate) fn the_facts_are_a_datasets_own_mount(facts: &MountFacts) -> bool {
    return facts.filesystem_type == ZFS_FILE_SYSTEM_TYPE && facts.root == Path::new("/") && facts.has_option("nosuid") && facts.has_option("nodev");
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
///  so a poll before every answer keeps the copy right about a mount,
///  an unmount,
///  a move and a flag change.
///  A file that cannot be opened,
///  and a poll that refuses or that names no descriptor the kernel knows,
///  each count as a change,
///  so the sweep reads the table again for the next answer instead of trusting
///  a copy nothing watches
///  (the table's own read still tries by path).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = LiveMountTableSource.open();
/// ```
pub(crate) struct LiveMountTableSource {
    /// The descriptor the kernel's poll watches.
    descriptor: Option<std::fs::File>,
    /// A control's own poll answer,
    /// which wins over the descriptor so the two fail-open paths are provable.
    #[cfg(test)]
    answered_for_the_control: Option<Option<libc::c_short>>,
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
        return Self {
            descriptor,
            #[cfg(test)]
            answered_for_the_control: None,
        };
    }
}

impl MountTableSource for LiveMountTableSource {
    fn changed(&self) -> bool {
        return match self.poll_answer() {
            // A poll this build cannot make is not evidence that the table stood still: the answer may have changed, so
            // the caller reads the table again rather than trusting a copy nothing watches.
            None => true,
            Some(revents) => revents & (libc::POLLPRI | libc::POLLERR | libc::POLLNVAL) != 0,
        };
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        let path: &Path = Path::new(crate::fs_mount_options::MOUNTINFO_PATH);
        let bytes: Vec<u8> = crate::plain_file::read_plain_file(path, crate::plain_file::MOUNT_TABLE_LIMIT)
            .map_err(|error| return format!("the mount table {path:?} could not be read: {error}"))?;
        return Ok(crate::fs_mount_options::parse_mount_table(bytes.as_slice()));
    }
}

impl LiveMountTableSource {
    /// What one poll of the table's descriptor answered,
    /// or nothing when no poll could be made.
    /// Why:
    ///  A missing descriptor,
    ///  a poll that refused,
    ///  a descriptor the kernel no longer knows
    ///  and a control's own answer all have to reach the same rule,
    ///  so the rule lives here and the caller treats "no answer" as a change.
    fn poll_answer(&self) -> Option<libc::c_short> {
        #[cfg(test)]
        if let Some(answer) = self.answered_for_the_control {
            return answer;
        }
        let Some(file) = &self.descriptor else {
            return None;
        };
        let mut watched: libc::pollfd = libc::pollfd { fd: file.as_raw_fd(), events: libc::POLLPRI | libc::POLLERR, revents: 0 };
        // SAFETY: `watched` is one initialised `pollfd`, the call writes only its `revents` field, and the timeout is
        // zero so it never waits.
        let ready: libc::c_int = unsafe { libc::poll(&mut watched, 1, 0) };
        if ready < 0 {
            debug(TAG, format!("the mount table's descriptor could not be polled: {}", std::io::Error::last_os_error()).as_str());
            return None;
        }
        if ready == 0 {
            return Some(0);
        }
        return Some(watched.revents);
    }

    /// A source with no descriptor,
    /// for the control that the answer is then "changed".
    #[cfg(test)]
    pub(crate) fn without_a_descriptor() -> Self {
        return Self { descriptor: None, answered_for_the_control: None };
    }

    /// A source whose poll answers the control's value,
    /// `None` being a poll that refused.
    #[cfg(test)]
    pub(crate) fn whose_poll_answers(answer: Option<libc::c_short>) -> Self {
        return Self { descriptor: None, answered_for_the_control: Some(answer) };
    }
}

/// What:
///  The release reader:
///  the mount table one sweep read,
///  refreshed when the kernel says the namespace's table changed,
///  and the canonical slot's mount named as this clone's own only when the
///  kernel says the dataset under it is this clone's.
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
///  a poll that cannot be made counts as a change,
///  and a firing re-reads the whole table.
///  The one field a change can move without any event is the source a
///  `legacy` dataset's mount prints,
///  so the source of the mount at the slot is asked of the kernel by that
///  mount's unique number,
///  and where the kernel cannot answer it the live table is read for that
///  mount,
///  which is `main`'s call.
///  A column of the table is rebuilt as a map only when a second answer needs
///  it,
///  the same read,
///  so a table that changes between every answer costs a scan per answer
///  where `main` pays a whole parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = LiveStandingMounts.read().standingMountOf(request, clone);
/// ```
pub struct LiveStandingMounts {
    /// The kernel table and the signal that it changed.
    source: Box<dyn MountTableSource>,
    /// The kernel's answer about one mount's source.
    sources: Box<dyn MountSourceByNumber>,
    /// The kernel's mount table as this sweep last read it.
    table: RefCell<CopiedTable>,
    /// How many times this sweep read the table.
    reads: Cell<usize>,
}

/// What:
///  One read of the kernel's mount table,
///  and the map over it a second answer builds.
/// Why:
///  A table that changes before every answer answers one child each,
///  so building a map for it would be work `main` never does;
///  a table that stands still answers many,
///  so the map is what keeps the answer independent of the child count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopiedTable = { mounts: MountFacts[]; answered: boolean; byNumber?: Map<number, MountFacts> };
/// ```
#[derive(Default)]
struct CopiedTable {
    /// The mounts this read listed,
    /// in the kernel's order.
    mounts: Vec<MountFacts>,
    /// Whether this read has already answered one child.
    answered: bool,
    /// The same mounts by the number the kernel reuses,
    /// where a second answer needed them.
    by_number: Option<HashMap<u64, MountFacts>>,
}

impl CopiedTable {
    /// What this read says about one mount,
    /// or nothing when it did not list it.
    /// Why:
    ///  The first answer to a read scans it,
    ///  the second builds the map,
    ///  and every later answer is a lookup.
    fn look_up(&mut self, mount_id: u64) -> Option<MountFacts> {
        if let Some(by_number) = &self.by_number {
            return by_number.get(&mount_id).cloned();
        }
        if !self.answered {
            self.answered = true;
            return self.mounts.iter().find(|facts| return facts.mount_id == mount_id).cloned();
        }
        let by_number: HashMap<u64, MountFacts> = self.mounts.iter().map(|facts| return (facts.mount_id, facts.clone())).collect();
        let found: Option<MountFacts> = by_number.get(&mount_id).cloned();
        self.by_number = Some(by_number);
        return found;
    }
}

impl LiveStandingMounts {
    /// Read the sweep's first copy of the kernel's mount table.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const standing = LiveStandingMounts.read();
    /// ```
    pub fn read() -> Self {
        return Self::from_sources(Box::new(LiveMountTableSource::open()), Box::new(KernelMountSource));
    }

    /// Build one reader over a table source a control owns,
    /// which asks the kernel for a mount's source.
    #[cfg(test)]
    pub(crate) fn from_source(source: Box<dyn MountTableSource>) -> Self {
        return Self::from_sources(source, Box::new(KernelMountSource));
    }

    /// Build one reader over the sources a control owns.
    pub(crate) fn from_sources(source: Box<dyn MountTableSource>, sources: Box<dyn MountSourceByNumber>) -> Self {
        let standing: Self = Self { source, sources, table: RefCell::new(CopiedTable::default()), reads: Cell::new(0) };
        standing.refresh();
        return standing;
    }

    /// Read the whole table again,
    /// clearing it when the read refuses so every child falls back to its own
    /// live read.
    fn refresh(&self) {
        self.reads.set(self.reads.get() + 1);
        let mut table = self.table.borrow_mut();
        match self.source.read() {
            Ok(read) => {
                *table = CopiedTable { mounts: read, answered: false, by_number: None };
            }
            Err(error) => {
                // A table the run cannot read leaves the copy empty, and each standing answer falls back to the child's
                // own live read, which is `main`'s answer.
                debug(TAG, format!("the mount table could not be read for this sweep: {error}").as_str());
                *table = CopiedTable::default();
            }
        }
    }

    /// Re-read the table when the kernel says this namespace's table changed,
    /// or when no poll could say it did not.
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
        if let Some(facts) = self.table.borrow_mut().look_up(mount_id) {
            return Ok(facts);
        }
        return mount_facts_of(mount_id).map_err(|refusal| return refusal.message());
    }

    /// What:
    ///  Whether the dataset mounted at the slot is this clone's,
    ///  and the source the answer names.
    /// Why:
    ///  The copy's source is the one field a rename moves without an event,
    ///  so the kernel is asked about the one mount at the slot by that mount's
    ///  unique number,
    ///  which the kernel never reuses;
    ///  an answer it cannot give,
    ///  a kernel without the call and an answer that disagrees with the walk all
    ///  fall back to the live table for that mount,
    ///  which is `main`'s call and its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [own, source] = standing.theDatasetUnder(fresh, clone);
    /// ```
    fn the_dataset_under(&self, fresh: &FreshSlotLookup, clone: &str) -> (bool, String) {
        return match self.sources.source_of(fresh.unique_mount_number) {
            MountSourceAnswer::Source(source) => (source == clone, source),
            MountSourceAnswer::NoAnswer(why) => {
                debug(TAG, format!("the kernel did not name the source of mount {}: {why}", fresh.mount_id).as_str());
                self.the_live_tables_answer(fresh.mount_id, clone)
            }
        };
    }

    /// The live table's whole answer about one mount,
    /// which is the call and the rule this route used before it kept a copy:
    /// the table is read again now,
    /// the copy is not consulted,
    /// and every clause of the predicate is applied to what the read says.
    fn the_live_tables_answer(&self, mount_id: u64, clone: &str) -> (bool, String) {
        return match self.source.read() {
            Ok(mounts) => match mounts.into_iter().find(|facts| return facts.mount_id == mount_id) {
                Some(facts) => (is_this_clones_own_mount(&facts, clone), facts.source),
                None => {
                    debug(TAG, format!("the live mount table does not list mount {mount_id}").as_str());
                    (false, String::new())
                }
            },
            Err(said) => {
                debug(TAG, format!("the live mount table could not be read for mount {mount_id}: {said}").as_str());
                (false, String::new())
            }
        };
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
        // A bind, another filesystem type or a mount without the two enforced attributes is not a dataset's own mount,
        // and no answer about its source can make it one.
        if !the_facts_are_a_datasets_own_mount(&facts) {
            return None;
        }
        let (own, source): (bool, String) = self.the_dataset_under(&fresh, clone);
        if !own {
            return None;
        }
        return Some(MountIdentity {
            mount_id: fresh.mount_id,
            parent_mount_id: facts.parent_id,
            mount_point: facts.mount_point,
            source,
            file_system_type: facts.filesystem_type,
            device: fresh.slot_identity.device,
            inode: fresh.slot_identity.inode,
            observed_path: fresh.observed_path,
        });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_standing_mounts_tests.rs"]
mod tests;
