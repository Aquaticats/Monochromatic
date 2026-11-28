//! What:
//!  The calls the shutdown pass makes,
//!  each one a trait a control scripts and a live implementation that makes
//!  the kernel's or the pool's own call:
//!  a fresh capture of one slot,
//!  the `.claude/worktrees` identity the route recorded,
//!  one clone's live GUID,
//!  the all-namespace holder scan,
//!  the mount table,
//!  the staging route's own calls,
//!  the descriptor walk to a slot,
//!  and the clock a line's time comes from.
//! Why:
//!  A control on a person's machine cannot mount,
//!  cannot unmount and must not read another account's processes,
//!  so every call that would is a seam the release path fills with the kernel
//!  and a value test fills with an answer it states
//! (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "Nothing a test changes may be process-wide";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//! The traits are the unmount route's already where the unmount route has one:
//!  `SlotWalk` opens the slot across its mount,
//! and `StagingCalls` unmounts from the held descriptor without force or lazy
//!  detach,
//!  so the shutdown pass makes the same calls the removal makes and a reader
//!  compares one implementation of each.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sources = { mounts: new LiveMountedDatasets(), walk: new LiveSlotWalk(), calls: new LiveStagingCalls(), lookups: new LiveSlotLookups(), guids, worktrees, holders, time };
//! ```

use crate::clone_holder_scan::holder_scan::{
    HolderScan,
    HolderScanRequest,
    scan_holders,
};
use crate::foundation::iso_time::now_iso;
use crate::mount_namespace_scan::mount_namespace_tasks::RootIdentity;
use crate::mount_namespace_scan::namespace_target;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount::slot_identity_store::SlotIdentityStore;
use crate::worktree_clone_mount::{
    FreshSlotLookup,
    LookAgainAtSlotRequest,
    look_again_at_the_slot,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_unmount::slot_walk_calls::DatasetGuids;
use std::path::Path;

/// What:
///  One slot's fresh capture:
///  the mount a whole walk reaches,
///  the mount its parent is on,
///  the folder's identity and the path the kernel reports.
/// Why:
///  The pass compares what it captures here with the descriptor it then opens
///  and with the row the mount table holds,
///  so a mount that moved between the three refuses rather than becoming the
///  new truth
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface SlotLookups { freshLookup(checkout, slot): FreshSlotLookup }
/// ```
pub trait SlotLookups {
    /// The slot's facts as a whole fresh walk of it finds them.
    ///
    /// # Errors
    /// A walk component refused,
    /// or the folder's facts could not be read.
    fn fresh_lookup(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<FreshSlotLookup, String>;
}

/// What:
///  The release capture:
///  the walk the attach and the removal already make.
/// Why:
///  The pass has to see the slot the way the attach saw it,
///  and a second walk written for the shutdown would be a second answer to
///  which mount stands at a slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fresh = new LiveSlotLookups().freshLookup('/home/user/Mono', 'topic');
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveSlotLookups;

impl SlotLookups for LiveSlotLookups {
    fn fresh_lookup(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<FreshSlotLookup, String> {
        return look_again_at_the_slot(&LookAgainAtSlotRequest {
            checkout,
            slot,
        })
        .map_err(|refusal| return refusal.message());
    }
}

/// What:
///  The `.claude/worktrees` identity the route recorded for this checkout.
/// Why:
///  The store's `.parent` record is the route's own proof of which folder the
///  slots hang in,
///  and an account that replaced that folder makes every mount below it a
///  mount this pass may not unmount
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WorktreesFolder { recorded(): CloneFolderIdentity }
/// ```
pub trait WorktreesFolder {
    /// The identity the store recorded for the folder the slots hang in.
    ///
    /// # Errors
    /// The store cannot be opened,
    /// holds no `.parent` record,
    /// or holds a record this route refuses.
    fn recorded(&self) -> Result<CloneFolderIdentity, String>;
}

/// What:
///  The release reading:
///  the store opened under root control once,
///  and its `.parent` record read from that descriptor.
/// Why:
///  The records are root's,
///  so the folder is opened under root control and nothing below a name the
///  account could change is followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = LiveWorktreesFolder.open('/var/lib/…/git-worktree-clone-slot-identities').recorded();
/// ```
pub struct LiveWorktreesFolder {
    /// The store folder,
    ///  held open.
    store: SlotIdentityStore,
}

impl LiveWorktreesFolder {
    /// The store at one path,
    /// opened under root control.
    ///
    /// # Errors
    /// The folder is not root's,
    ///  or cannot be opened.
    pub fn open(path: &Path) -> Result<Self, String> {
        return SlotIdentityStore::open_under_root_control(path)
            .map(|store| {
                return LiveWorktreesFolder {
                    store,
                };
            })
            .map_err(|refusal| return refusal.message());
    }
}

impl WorktreesFolder for LiveWorktreesFolder {
    fn recorded(&self) -> Result<CloneFolderIdentity, String> {
        return self
            .store
            .parent()
            .map_err(|refusal| return refusal.message());
    }
}

/// What:
///  The one call that takes a mount down:
///  the mount standing at one slot,
///  unmounted from a descriptor on the folder the slot hangs in.
/// Why:
///  A descriptor on the clone itself holds it busy,
///  so the kernel answers `EBUSY` for the pass's own mount
///  (the removal's unmount writes the same down and drops its descriptor for
///  it:
///  `worktree_clone_unmount/unmount_pair.rs`,
///  "A descriptor on the clone would hold it busy").
///  The folder the slot hangs in is on the checkout dataset,
///  so a descriptor on it names the slot without keeping the clone busy,
///  and the call is a `/proc/self/fd` string rather than a path of the
///  account's tree,
///  which the staging design allows and a bare path does not
///  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
///  "The unmount call from staging").
///  The errno is answered with the kernel's words so a busy mount is told from
///  any other refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface SlotUnmounts { unmountTheSlot(checkout, slot): void } // throws [errno, detail]
/// ```
pub trait SlotUnmounts {
    /// Unmount what stands at one slot,
    /// naming it through a descriptor on the folder it hangs in.
    ///
    /// # Errors
    /// The folder could not be held,
    /// or the kernel refused the unmount,
    /// with its errno and its words.
    fn unmount_the_slot(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<(), (i32, String)>;
}

/// What:
///  The release unmount:
///  the folder the slot hangs in,
///  held without crossing a mount,
///  and `umount2` of the slot's own name inside it.
/// Why:
///  No force and no lazy detach,
///  because the design forbids both:
///  flags zero is the whole of the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new LiveSlotUnmounts().unmountTheSlot('/home/user/Mono', 'one'); // umount2('/proc/self/fd/N/one', 0)
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveSlotUnmounts;

impl SlotUnmounts for LiveSlotUnmounts {
    fn unmount_the_slot(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<(), (i32, String)> {
        let held: crate::worktree_clone_mount::HeldWorktreesParent =
            crate::worktree_clone_mount::hold_the_worktrees_parent(checkout)
                .map_err(|refusal| return (0, refusal.message()))?;
        let parent: std::fs::File = held.folder;
        let named: std::ffi::CString = std::ffi::CString::new(format!(
            "/proc/self/fd/{}/{}",
            std::os::fd::AsRawFd::as_raw_fd(&parent),
            slot
        ))
        .map_err(|_nul| return (libc::EINVAL, String::from("the slot's name holds a NUL")))?;
        // SAFETY: `named` is NUL-terminated and outlives the call, and no flag asks
        //  for force or lazy detach.
        let taken: libc::c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
        return if taken == 0 {
            Ok(())
        } else {
            let error: std::io::Error = std::io::Error::last_os_error();
            Err((error.raw_os_error().unwrap_or(0), error.to_string()))
        };
    }
}

/// What:
/// One clone's live GUID,
///  as the pool answers it now.
/// Why:
///  A row whose source names the clone is not yet proof that the dataset is
///  the one the route made:
/// a dataset destroyed and made again under the same name answers a different
///  GUID,
///  and the pass reports that rather than unmounting what it cannot name
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface DatasetGuids { liveGuid(dataset): DatasetGuid }
/// ```
pub struct LiveDatasetGuids<'a> {
    /// The pool's reader.
    pub reader: &'a dyn DatasetReader,
}

impl DatasetGuids for LiveDatasetGuids<'_> {
    fn live_guid(
        &self,
        dataset: &str,
    ) -> Result<DatasetGuid, String> {
        let identity = self
            .reader
            .identity(dataset)
            .map_err(|refusal| return refusal.message())?;
        return DatasetGuid::parse(identity.stats.dds_guid.to_string().as_str())
            .map_err(|refusal| return refusal.message());
    }
}

/// What:
///  Who holds one clone,
///  asked the way the route asks it:
///  every mount namespace that holds a mount of the dataset.
/// Why:
///  A busy unmount is answered by naming the clone and its holders and by
///  tearing nothing down,
///  and the holder scan is the project's own answer to "who holds this clone"
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Mounts held by other namespaces";
///  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
///  step 9).
///  A control hands in a `/proc` tree of its own,
///  so no test reads another account's processes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface HolderScanning { holdersOf(clone, dataset): HolderScan }
/// ```
pub trait HolderScanning {
    /// Every namespace holding a mount of one clone's dataset,
    /// under a nonce this scan made.
    fn holders_of(
        &self,
        clone: &str,
        dataset: DatasetGuid,
    ) -> Result<HolderScan, String>;
}

/// What:
///  The release scan:
///  this machine's `/proc`,
///  this process's own namespace and root,
///  and the clock.
/// Why:
///  The result has to say which namespaces are not the reader's,
///  which it can only answer from the reader's own place,
///  and its nonce has to be fresh because a result names one scan
///  (`crate::clone_holder_scan::holder_scan::HolderScanRequest`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scan = new LiveHolderScans({ procRoot: '/proc', selfTask: '/proc/self' }).holdersOf('tank/…/clones/one', guid);
/// ```
pub struct LiveHolderScans {
    /// The `/proc` root the scan reads.
    pub proc_root: std::path::PathBuf,
    /// The task whose namespace and root are this process's own.
    pub self_task: std::path::PathBuf,
}

impl HolderScanning for LiveHolderScans {
    fn holders_of(
        &self,
        clone: &str,
        dataset: DatasetGuid,
    ) -> Result<HolderScan, String> {
        // A scan of one dataset answers under a nonce of its own, and a random
        // source that failed leaves no nonce to answer under: the scan is
        // refused rather than reported as a clear machine.
        let nonce: RequestNonce =
            RequestNonce::fresh().map_err(|refusal| return refusal.message())?;
        let namespace: Option<String> =
            namespace_target(self.self_task.join("ns").join("mnt").as_path());
        let root: Option<RootIdentity> = RootIdentity::of(self.self_task.join("root").as_path());
        return Ok(scan_holders(&HolderScanRequest {
            proc_root: self.proc_root.as_path(),
            source: clone,
            own_namespace: namespace.as_deref(),
            readers_root: root,
            dataset,
            nonce: &nonce,
            time: now_iso(),
        }));
    }
}
