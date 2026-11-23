//! What:
//!  One clone's whole turn of the sweep:
//!  read what the dataset says,
//!  verify it,
//!  read its durable intent,
//!  take the slot's claim,
//!  read the clone again under the claim,
//!  recognise a mount of this clone that already stands at the slot,
//!  claim this boot's first attach,
//!  record the slot's first observation,
//!  attach,
//!  record the first attach's outcome,
//!  answer the request with its receipt (the attach's answer before the release,
//!  a standing mount's after it),
//!  release the claim,
//!  and read the intent once more.
//! Why:
//!  The order is the design's:
//!  the checks,
//!  then the intent,
//!  then the claim under which the desired state and identities are rechecked,
//!  then the attach,
//!  and one receipt per clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  which says "Recheck desired state and identities under a per-slot root
//!  claim";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The sweep plan step 8").
//!  A malformed intent is refused before any filesystem operation on its behalf,
//!  an inherited intent authorizes nothing,
//!  and a slot another sweep holds is left to that sweep rather than queued.
//!  A refusal files no receipt,
//!  except an unavailable key,
//!  which files a `refused` receipt naming that step so the waiting command
//!  learns of it at once
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals").
//!  The holder reads the intent again after it releases the claim,
//!  so a request that arrived while it held the slot is acted on in a second
//!  turn rather than lost
//!  (the same decision,
//!  "A slot whose claim is held").
//!  A clone whose durable intent is `mount` keeps that intent after it
//!  attached,
//!  so the ordinary sweep is one over a clone already mounted at its slot:
//!  that mount is recognised as this clone's own and the `done` receipt is
//!  kept,
//!  instead of the walk refusing the slot and a `refused` receipt overwriting
//!  it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = sweepOneClone(context, 'zpcachyos/…/clones/one');
//! ```

use super::sweep_attach_step::{AttachRequest, AttachStepOutcome, CloneAttachStep};
use super::sweep_clone_check::{VerifiedClone, verify};
use super::sweep_dataset_facts::{CloneObjectFacts, facts_of};
use super::sweep_dataset_reader::DatasetReader;
use super::sweep_first_attach::FirstAttachRecorder;
use super::sweep_one_clone_answers::{answer_the_attach, answer_the_standing_mount, answer_unmount, record_the_first_attach, refused_by_the_checks, the_first_attach_outcome, the_first_attach_refusal};
use crate::diagnostic_log::{debug, warn};
use crate::start_probe::first_attach_record::FirstAttachClaim;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;
use crate::worktree_clone_mount::{
    HoldCloneSlotRequest, LookAgainAtSlotRequest, MountIdentity, SlotIdentityStore, StoreRefusal, look_again_at_the_slot, mount_facts_of, observe_the_clone_slot_under_root_control,
    publish_slot_identity_under_root_control, read_the_slot_git_pointer,
};
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_protocol::{IntentReading, MountIntent, MountOperation, ReceiptsFolder, read_intent};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "sweepOneClone";

/// The step a receipt names when an unmount request reaches this build.
///
/// Why:
///  The unmount is plan step 9's,
///  and a caller that asked for one learns that from the reply and the
///  handler's journal,
///  while the durable intent stays on the clone unchanged.
pub const UNMOUNT_AWAITS_STEP_NINE: &str = "unmount-awaits-step-9";

/// The step a receipt and the first attach's record name when the slot's own
/// first observation could not be read or recorded.
pub const IDENTITY_RECORD_STEP: &str = "identity-record";

/// The step a receipt names when the receipt itself could not be filed.
pub const RECEIPT_STEP: &str = "receipt";

/// The step the sweep names when the durable intent changed while it waited for
/// the claim.
pub const INTENT_STEP: &str = "intent";

/// What:
///  Everything one clone's turn reads and writes.
/// Why:
///  The paths and the seams travel together so a caller cannot sweep one
///  checkout's children against another checkout's store,
///  and a control injects fixture folders and a scripted pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepContext = { parentDataset; checkout; checkoutDataset; store; claims; receipts; boot; time; reader; attach; firstAttach };
/// ```
pub struct SweepContext<'a> {
    /// The pinned clone parent dataset.
    pub parent_dataset: &'a str,
    /// The checkout the walk starts from.
    pub checkout: &'a Path,
    /// The checkout dataset a clone's origin must be a snapshot of.
    pub checkout_dataset: &'a str,
    /// The store of recorded identities.
    pub store: &'a Path,
    /// The per-slot claims.
    pub claims: &'a SlotClaimDirectory,
    /// The receipts folder.
    pub receipts: &'a ReceiptsFolder,
    /// This boot's kernel identifier.
    pub boot: &'a str,
    /// The clock a receipt's time is written from.
    pub time: &'a dyn Fn() -> String,
    /// The pool reads.
    pub reader: &'a dyn DatasetReader,
    /// The attach step.
    pub attach: &'a dyn CloneAttachStep,
    /// This boot's first attach,
    /// which the first attempt that passes the key check claims.
    pub first_attach: &'a dyn FirstAttachRecorder,
    /// What the canonical slot already holds.
    pub standing: &'a dyn StandingMounts,
}

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
    fn open() -> Self {
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
    fn from_source(source: Box<dyn MountTableSource>) -> Self {
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
    fn facts_of(&self, mount_id: u64) -> Result<MountFacts, String> {
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

/// What:
///  What one clone's turn answered.
/// Why:
///  A sweep reports every clone it reached,
///  and a clone it attached and a clone it refused must read differently in one
///  line an operator or a guest arm can find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneOutcome = { kind: 'nothing-asked' | 'attached' | 'refused' | 'failed' | 'held'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneOutcome {
    /// The clone asked for nothing,
    /// or the value it carries is inherited.
    NothingAsked {
        /// The dataset.
        name: String,
        /// The slot its name gave.
        slot: String,
        /// Why nothing was asked.
        why: String,
    },
    /// The clone's mount is at its slot.
    Attached {
        /// The dataset.
        name: String,
        /// The slot.
        slot: String,
        /// The mount the attach made,
        /// or the mount this clone already had at the slot.
        mount_id: u64,
        /// The canonical slot the mount was found at.
        canonical_slot: PathBuf,
    },
    /// Nothing was attached,
    /// and the line says why.
    Refused {
        /// The dataset.
        name: String,
        /// The slot,
        /// where the name gave one.
        slot: Option<String>,
        /// The step the answer names.
        step: String,
        /// The refusal in words.
        said: String,
    },
    /// A step failed after or during the attach,
    /// and the receipt says which,
    /// or nothing was filed when the receipt itself refused.
    Failed {
        /// The dataset.
        name: String,
        /// The slot,
        /// where the name gave one.
        slot: Option<String>,
        /// The step the receipt names.
        step: String,
        /// What the step answered.
        said: String,
    },
    /// Another sweep holds this slot.
    Held {
        /// The dataset.
        name: String,
        /// The slot another sweep holds.
        slot: String,
        /// What the claim answered.
        said: String,
    },
}

impl CloneOutcome {
    /// The dataset this outcome is about.
    pub fn name(&self) -> &str {
        return match self {
            CloneOutcome::NothingAsked { name, .. } | CloneOutcome::Attached { name, .. } | CloneOutcome::Refused { name, .. } | CloneOutcome::Failed { name, .. } | CloneOutcome::Held { name, .. } => {
                name.as_str()
            }
        };
    }

    /// The one line a sweep's report carries for this clone.
    /// It begins with the answer's own word,
    /// names the dataset and the slot,
    /// and says what a person does next when the answer was no.
    pub fn line(&self) -> String {
        return match self {
            CloneOutcome::NothingAsked { name, slot, why } => format!("nothing {name} slot={slot} why={why}"),
            CloneOutcome::Attached { name, slot, mount_id, canonical_slot } => format!("attached {name} slot={slot} mount={mount_id} at={}", canonical_slot.display()),
            CloneOutcome::Refused { name, slot, step, said } => format!("refused {name} slot={} step={step}: {said}", slot.as_deref().unwrap_or("none")),
            CloneOutcome::Failed { name, slot, step, said } => format!("failed {name} slot={} step={step}: {said}", slot.as_deref().unwrap_or("none")),
            CloneOutcome::Held { name, slot, said } => format!("held {name} slot={slot}: {said}"),
        };
    }
}

/// What:
///  Read one clone,
///  verify it,
///  and answer what its intent asked for.
/// Why:
///  One function is the whole turn so the order cannot differ between the
///  sweep a boot runs and the sweep a ZED event starts,
///  and the holder reads the intent again after it releases the claim at most
///  once,
///  so a change made during the turn is acted on and a change made during the
///  second turn waits for the next trigger rather than looping here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = sweepOneClone(context, name);
/// ```
pub fn sweep_one_clone(context: &SweepContext<'_>, name: &str) -> CloneOutcome {
    return sweep_one_clone_once(context, name, true);
}

/// The turn itself,
/// with the one re-read after the claim's release allowed or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = sweepOneCloneOnce(context, name, mayReadAgain);
/// ```
fn sweep_one_clone_once(context: &SweepContext<'_>, name: &str, may_read_again: bool) -> CloneOutcome {
    let facts: CloneObjectFacts = match context.reader.identity(name) {
        Ok(identity) => facts_of(name, &identity),
        Err(reason) => {
            warn(TAG, format!("`{name}` could not be read: {reason}").as_str());
            return CloneOutcome::Refused { name: String::from(name), slot: None, step: String::from("read"), said: reason.message() };
        }
    };
    let verified: VerifiedClone = match verify(&facts, context.parent_dataset, context.checkout_dataset) {
        Ok(verified) => verified,
        Err(refusal) => return refused_by_the_checks(context, name, &facts, &refusal),
    };
    let request: MountIntent = match read_intent(Some(&facts.properties), name) {
        IntentReading::Absent => return CloneOutcome::NothingAsked { name: String::from(name), slot: verified.slot, why: String::from("no local mount intent") },
        IntentReading::Inherited { from } => {
            return CloneOutcome::NothingAsked { name: String::from(name), slot: verified.slot, why: format!("the mount intent is inherited from {from}") };
        }
        IntentReading::Refused(refusal) => return CloneOutcome::Refused { name: String::from(name), slot: Some(verified.slot), step: String::from(INTENT_STEP), said: refusal.message() },
        IntentReading::Request(request) => request,
    };
    if request.operation == MountOperation::Unmount {
        return answer_unmount(&verified, &request);
    }
    return mount_the_clone(context, name, &verified, &request, may_read_again);
}

/// What:
///  Take the claim,
///  read the clone again under it,
///  recognise a mount of this clone that already stands at the slot,
///  claim this boot's first attach,
///  record the slot's first observation,
///  attach,
///  record the outcome,
///  and release.
/// Why:
///  The claim is what lets parallel sweeps work without acting on the same
///  mount twice,
///  and the desired state and the identities are rechecked under it so a sweep
///  that waited attaches nothing an intent changed in the meantime asked it not
///  to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = mountTheClone(context, name, verified, request, mayReadAgain);
/// ```
fn mount_the_clone(context: &SweepContext<'_>, name: &str, verified: &VerifiedClone, request: &MountIntent, may_read_again: bool) -> CloneOutcome {
    let claim = match context.claims.claim(verified.slot.as_str()) {
        Ok(claim) => claim,
        Err(refusal) => return CloneOutcome::Held { name: String::from(name), slot: verified.slot.clone(), said: refusal.message() },
    };
    let under_the_claim: (VerifiedClone, MountIntent) = match reread_under_the_claim(context, name) {
        Ok(reread) => reread,
        Err(outcome) => {
            claim.release();
            return turn_again_after_the_release(context, name, request, outcome, may_read_again);
        }
    };
    if under_the_claim.0.dataset != verified.dataset || under_the_claim.1.nonce != request.nonce || under_the_claim.1.operation != request.operation {
        claim.release();
        let said: String = format!(
            "the clone `{name}` was read as GUID {} asking `{}` under nonce {} before this sweep took slot `{}`, and under the claim it reads as GUID {} asking `{}` under nonce {}. Nothing was attached on the first reading, and the intent is read again after the claim is released",
            verified.dataset.file_name(),
            request.operation.word(),
            request.nonce.as_str(),
            verified.slot,
            under_the_claim.0.dataset.file_name(),
            under_the_claim.1.operation.word(),
            under_the_claim.1.nonce.as_str()
        );
        warn(TAG, said.as_str());
        let refused: CloneOutcome = CloneOutcome::Refused { name: String::from(name), slot: Some(verified.slot.clone()), step: String::from(INTENT_STEP), said };
        return turn_again_after_the_release(context, name, request, refused, may_read_again);
    }
    if let Some(mount) = context.standing.standing_mount_of(&LookAgainAtSlotRequest { checkout: context.checkout, slot: verified.slot.as_str() }, verified.dataset_name.as_str()) {
        claim.release();
        let answered: CloneOutcome = answer_the_standing_mount(context, verified, request, mount);
        return turn_again_after_the_release(context, name, request, answered, may_read_again);
    }
    // This boot's first attach is claimed after the key check that the re-read ran and before the walk that opens the
    // slot, which is the order the start-probe design states; a sweep that finds this clone already mounted above has
    // claimed nothing, because it attached nothing.
    let first_attach: Option<FirstAttachClaim> = context.first_attach.claim();
    if let Err(said) = ensure_the_slot_record(context, verified) {
        claim.release();
        record_the_first_attach(context, first_attach, &the_first_attach_refusal(verified, IDENTITY_RECORD_STEP, None));
        let refused: CloneOutcome = CloneOutcome::Refused { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step: String::from(IDENTITY_RECORD_STEP), said };
        return turn_again_after_the_release(context, name, request, refused, may_read_again);
    }
    let outcome: AttachStepOutcome = context.attach.attach(&AttachRequest { checkout: context.checkout, slot: verified.slot.as_str(), clone: name, store: context.store });
    record_the_first_attach(context, first_attach, &the_first_attach_outcome(verified, &outcome));
    // The answer is filed before the intent is read once more, so a mount this turn made is answered under the nonce it
    // was made for even when a request arrived meanwhile.
    let answered: CloneOutcome = answer_the_attach(context, verified, request, outcome);
    claim.release();
    return turn_again_after_the_release(context, name, request, answered, may_read_again);
}

/// What:
///  Turn once more after every release of a claim this turn took,
///  when the intent changed while the claim was held.
/// Why:
///  The project decided the holder reads the intent again after it releases
///  the claim,
///  so a request that arrived during a sweep is acted on rather than left to a
///  trigger that may not come.
///  The same rule holds wherever the turn releases a claim it took,
///  not only after an attach:
///  a standing mount,
///  a changed reading under the claim and a refused re-read all release it.
///  One turn may re-read,
///  so a request that keeps changing cannot make the sweep loop.
///  A turn whose own answer is a failure keeps that answer,
///  because a retained mount must reach the run's failure count rather than be
///  replaced by the second turn's refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = turnAgainAfterTheRelease(context, name, request, answered, mayReadAgain);
/// ```
fn turn_again_after_the_release(context: &SweepContext<'_>, name: &str, request: &MountIntent, answered: CloneOutcome, may_read_again: bool) -> CloneOutcome {
    if !may_read_again || !the_intent_changed(context, name, request) {
        return answered;
    }
    debug(TAG, format!("the intent of `{name}` changed while the slot was claimed, so the sweep turns once more").as_str());
    let second: CloneOutcome = sweep_one_clone_once(context, name, false);
    if matches!(answered, CloneOutcome::Failed { .. }) {
        return answered;
    }
    return second;
}

/// What:
///  Read the clone's identity and its intent again under the claim,
///  refusing where the first reading passed and the second does not.
/// Why:
///  A sweep that waited on a claim,
///  or ran slowly,
///  can act on facts the clone no longer says,
///  and the removal protocol relies on a standing `unmount` intent to exclude
///  new attaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [verified, request] = rereadUnderTheClaim(context, name);
/// ```
///
/// # Errors
/// The clone could not be read,
/// the checks refused it,
/// or its intent is absent,
/// inherited or malformed now.
fn reread_under_the_claim(context: &SweepContext<'_>, name: &str) -> Result<(VerifiedClone, MountIntent), CloneOutcome> {
    let facts: CloneObjectFacts = match context.reader.identity(name) {
        Ok(identity) => facts_of(name, &identity),
        Err(reason) => return Err(CloneOutcome::Refused { name: String::from(name), slot: None, step: String::from("read"), said: reason.message() }),
    };
    let verified: VerifiedClone = verify(&facts, context.parent_dataset, context.checkout_dataset).map_err(|refusal| {
        warn(TAG, refusal.message().as_str());
        return CloneOutcome::Refused { name: String::from(name), slot: None, step: String::from(refusal.step()), said: refusal.message() };
    })?;
    return match read_intent(Some(&facts.properties), name) {
        IntentReading::Request(request) => Ok((verified, request)),
        IntentReading::Absent => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: String::from("the clone's mount intent is gone under the claim, so nothing was attached on the reading that saw it"),
        }),
        IntentReading::Inherited { from } => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: format!("the clone's mount intent is inherited from {from} under the claim, so nothing was attached"),
        }),
        IntentReading::Refused(refusal) => Err(CloneOutcome::Refused { name: String::from(name), slot: Some(verified.slot), step: String::from(INTENT_STEP), said: refusal.message() }),
    };
}

/// Whether the clone's intent changed after the request this turn acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = theIntentChanged(context, name, request); // true when a read answers another request
/// ```
fn the_intent_changed(context: &SweepContext<'_>, name: &str, request: &MountIntent) -> bool {
    let Ok(identity) = context.reader.identity(name) else {
        return false;
    };
    let facts: CloneObjectFacts = facts_of(name, &identity);
    return match read_intent(Some(&facts.properties), name) {
        IntentReading::Request(now) => now.nonce != request.nonce || now.operation != request.operation,
        IntentReading::Absent | IntentReading::Inherited { .. } | IntentReading::Refused(_) => true,
    };
}

/// What:
///  Publish the slot's first-observation record when the store holds none,
///  and leave a record another sweep published standing.
/// Why:
///  The record is the folder's identity taken before the attach,
///  and the first observation also checks what the slot holds:
///  a bounded,
/// regular,
/// no-follow read of its Git pointer,
///  which must name this checkout's registration of the slot
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///  A slot another sweep recorded first is the store's own loser path:
///  this sweep reads the record that won and carries on with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ensureTheSlotRecord(context, verified);
/// ```
///
/// # Errors
/// The store's own refusal,
/// the first observation's walk refusal,
/// a slot whose Git pointer is not this checkout's registration,
/// or a publication the store did not keep.
fn ensure_the_slot_record(context: &SweepContext<'_>, verified: &VerifiedClone) -> Result<(), String> {
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(context.store).map_err(|refusal| return refusal.message())?;
    return match store.slot(verified.slot.as_str()) {
        Ok(_recorded) => Ok(()),
        Err(StoreRefusal::NoSlotRecord { .. }) => publish_the_first_observation(context, verified),
        Err(refusal) => Err(refusal.message()),
    };
}

/// The publication of one slot's first observation.
///
/// # Errors
/// The first observation's walk refusal,
/// a Git pointer this route does not accept,
/// or a publication the store did not keep.
fn publish_the_first_observation(context: &SweepContext<'_>, verified: &VerifiedClone) -> Result<(), String> {
    let request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest { checkout: context.checkout, slot: verified.slot.as_str(), store: context.store };
    let observed = observe_the_clone_slot_under_root_control(&request).map_err(|refusal| return refusal.message())?;
    let registration: PathBuf = read_the_slot_git_pointer(&observed.slot, context.checkout, verified.slot.as_str()).map_err(|refusal| return refusal.message())?;
    debug(TAG, format!("the slot `{}` holds the registration {}", verified.slot, registration.display()).as_str());
    let published = publish_slot_identity_under_root_control(context.store, verified.slot.as_str(), &observed.slot_identity, &observed.parent_identity);
    return match published {
        Ok(()) => {
            debug(TAG, format!("recorded the first observation of slot `{}`", verified.slot).as_str());
            Ok(())
        }
        // Another sweep recorded the slot between this sweep's read and its link, and its record is the one that stands:
        // the attach that follows compares the folder against it, so a folder that moved is refused there.
        Err(StoreRefusal::SlotAlreadyRecorded { .. }) => Ok(()),
        Err(refusal) => Err(refusal.message()),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_one_clone_tests.rs"]
mod tests;
