//! What:
//!  Every read one managed clone's `doctor` lines take from the running
//!  machine,
//!  as one seam with one method per read,
//!  and the request those reads answer for.
//! Why:
//!  The checks plan step 18 adds judge values and never a live machine
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds"),
//!  and the fact set they judge is gathered once per clone,
//!  so the reads that fill it are one value too:
//!  a control answers any of them without a pool,
//!  a mount table or a `/proc`,
//!  and the release answers the same methods from the machine
//!  ([`super::clone_facts_live_reads`]).
//!  One method per read rather than one per fact,
//!  because a fact built from two reads would hide which of them failed,
//!  and the report's three answers
//!  (pass,
//!  finding,
//!  unknown with its reason)
//!  can only name a reason the read itself gave.
//!  A read is asked only where its fact needs it:
//!  a clone whose record names no registration,
//!  origin or plan GUID has no such read made for it,
//!  so a healthy clone costs no request this build cannot answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reads: CloneFactsReads = new LiveCloneFactsReads();
//! const facts = gatherCloneFacts({ commonDir, slot, slotPath, cache }, reads);
//! ```

/// The pool read one clone's facts are about.
use super::clone_retention_checks::AccountingFacts;
/// What stands at one path,
/// as every handler check reads it.
use super::handler_checks::PathState;
/// What stands at the destruction plan's own file name.
use super::removal_plan_checks::PlanFileReading;
/// What one check could read,
/// and the three answers a read of one object gives.
use super::zfs_checks::Reading;
/// Every ZFS dataset mount one process's table listed.
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_managed_lock::Classification;
/// The pool's own GUID for one dataset.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
/// The holders one all-namespace scan found,
/// as the removal folder that owns the scan reports them.
use crate::worktree_clone_removal_proof_and_plan::holder_report::TheHoldersOfOneClone;
/// The snapshots and bookmarks one clone holds.
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;
/// One slot's readings,
/// which step 12 already decides from the record,
/// the plan folder,
/// the intent and the mount table.
use crate::worktree_clone_slot_state::slot_state_readings::SlotReadings;
/// The state of one origin snapshot's deferred-destroy mark,
/// as step 15's own read answers it.
use crate::worktree_removal_deferred_origin_check::origin_mark::OriginMark;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  One managed clone and the places its reads are about:
///  the repository whose records name it,
///  the slot it stands at and the cache its staging folder lives under.
/// Why:
///  The record's home,
///  the plan's folder and the staging folder are all under the repository's
///  common directory or the account's cache,
///  so the two folders and the slot's name travel with every read,
///  and a control writes them rather than this machine's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { commonDir: '/repo/.git', slot: 'one', slotPath: '/repo/.claude/worktrees/one', cache: '/home/user/.cache/cli-git', callerTask: 4242 };
/// ```
pub struct CloneFactsRequest {
    /// The repository's Git common directory,
    /// under which the record,
    /// the plan folder and the origin inventory live.
    pub common_dir: PathBuf,
    /// The slot's name,
    /// which is the record's own key.
    pub slot: String,
    /// The slot's whole path,
    /// which the record names and the registration links to.
    pub slot_path: PathBuf,
    /// The account's cache folder,
    /// under which a removal stages a clone.
    pub cache: PathBuf,
    /// The task that asked for the report,
    /// which is the representative task of the caller's own namespace's
    /// mounts.
    pub caller_task: u32,
}

/// What:
///  One clone dataset as the pool names it now:
///  its GUID and the transaction group that created it.
/// Why:
///  The GUID is what a record's stale name is compared against,
///  and the creation group is the one fact that splits a clone's own
///  snapshots from the history it inherited,
///  so both come from one read of one object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = { guid: '1837462836473829', createtxg: 121294 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneFactsIdentity {
    /// The dataset's own GUID.
    pub guid: DatasetGuid,
    /// The transaction group that created it.
    pub createtxg: u64,
}

/// What:
///  Every read one clone's facts are gathered from.
/// Why:
///  A control scripts any one of them and the release answers them from the
///  machine,
///  so the composition below is the same code in both,
///  and a read that fails says why in the fact's own unknown-carrying shape
///  rather than in a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CloneFactsReads {
///   slotReadings(request): SlotReadings;
///   mountRows(): MountFacts[];
///   pathState(path): PathState;
///   registrationGitdir(registration): Reading<PathBuf>;
///   lockClassification(request): Classification;
///   planReading(request, cloneGuid): PlanFileReading;
///   cloneIdentity(clone): Reading<CloneFactsIdentity>;
///   inventory(clone, cloneGuid): Reading<LiveInventory>;
///   originMark(originSnapshot): Reading<OriginMark>;
///   accounting(object): Reading<AccountingFacts>;
///   holderScan(clone, cloneGuid): Result<HolderScan, string>;
///   inTheCallersSession(tasks): number[];
///   theForcedArmIsMeasured(): boolean;
/// }
/// ```
pub trait CloneFactsReads {
    /// One slot's record,
    /// plan,
    /// intent,
    /// mount table and registration,
    /// as step 12's own live read answers them.
    fn slot_readings(
        &self,
        request: &CloneFactsRequest,
    ) -> SlotReadings;

    /// Every mount row of this process's table,
    /// which a removal's staging folder is found among.
    fn mount_rows(&self) -> Vec<MountFacts>;

    /// What stands at one path.
    fn path_state(
        &self,
        path: &Path,
    ) -> PathState;

    /// The path one registration's `gitdir` file names,
    /// as the registration's own folder spells it.
    ///
    /// A `Missing` answer is a registration with no `gitdir` file,
    /// which is a state Git's own killed add leaves,
    /// and an `Unreadable` one is a link nobody read.
    fn registration_gitdir(
        &self,
        registration: &Path,
    ) -> Reading<PathBuf>;

    /// The managed Git lock's state,
    /// as step 13's proof over this machine answers it.
    fn lock_classification(
        &self,
        request: &CloneFactsRequest,
    ) -> Classification;

    /// What stands at the destruction plan's own file name for one clone.
    fn plan_reading(
        &self,
        request: &CloneFactsRequest,
        clone_guid: DatasetGuid,
    ) -> PlanFileReading;

    /// The clone dataset's own identity,
    /// read from the pool.
    fn clone_identity(
        &self,
        clone: &str,
    ) -> Reading<CloneFactsIdentity>;

    /// Every snapshot and bookmark the clone holds.
    fn inventory(
        &self,
        clone: &str,
        clone_guid: DatasetGuid,
    ) -> Reading<LiveInventory>;

    /// The state of one origin snapshot's deferred-destroy mark,
    /// through step 15's own read.
    fn origin_mark(
        &self,
        origin_snapshot: &str,
    ) -> Reading<OriginMark>;

    /// One dataset's or snapshot's accounting fields.
    fn accounting(
        &self,
        object: &str,
    ) -> Reading<AccountingFacts>;

    /// One fresh all-namespace scan for the clone,
    /// or the reason the scan could not be made.
    ///
    /// # Errors
    ///
    /// The scan's own run nonce could not be made,
    /// so its answer would be bound to nothing.
    fn holder_scan(
        &self,
        clone: &str,
        clone_guid: DatasetGuid,
    ) -> Result<TheHoldersOfOneClone, String>;

    /// Which of these tasks the caller's own session holds.
    fn in_the_callers_session(
        &self,
        tasks: &[u32],
    ) -> Vec<u32>;

    /// Whether this build offers a forced removal whose waits are measured.
    fn the_forced_arm_is_measured(&self) -> bool;
}
