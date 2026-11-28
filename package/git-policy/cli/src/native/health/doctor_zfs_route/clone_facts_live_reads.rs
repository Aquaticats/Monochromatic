//! What:
//!  The release answers to [`super::clone_facts_reads::CloneFactsReads`]:
//!  this machine's slot readings,
//!  mount table,
//!  paths,
//!  managed Git lock,
//!  destruction plan,
//!  pool reads,
//!  origin mark,
//!  all-namespace scan and the caller's own session.
//! Why:
//!  `doctor`'s report is the one reader of these facts a person meets,
//!  so the requests it makes are the route's own typed ones and never a
//!  `zfs` or a `zpool` program:
//!  every raw request passes the compatibility gate the moment the device is
//!  opened,
//!  and one open serves the whole clone's group
//!  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "A missing device ... disables raw requests,
//!  reads included").
//!  The gate's refusal is a read nobody made,
//!  so it becomes the fact's unreadable answer with the gate's own sentence
//!  and never a pass on an empty pool.
//!  The reads themselves are the modules that own them:
//!  step 12's slot read,
//!  step 13's lock proof,
//!  the removal's pool and plan readers,
//!  and step 15's mark read,
//!  so a rule is not restated here.
//!  Two facts are not reads of this machine at all:
//!  which holders the caller's own session keeps,
//!  answered from the kernel's own session numbers,
//!  and whether this build offers a forced removal whose waits are measured,
//!  which no build does yet,
//!  so the answer stays `false` until the round that measures them changes
//!  it beside its constants.
//!  The holder scan is made once per clone and names the namespaces and tasks
//!  a person can look at,
//!  as the decision record requires.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reads = new LiveCloneFactsReads();
//! const facts = gatherCloneFacts({ commonDir, slot, slotPath, cache, callerTask: process.pid }, reads);
//! ```

/// The two answers this machine gives a clone's facts without a pool.
use super::clone_facts_live_answers::{
    the_state_of_a_path,
    the_tasks_in_the_callers_session,
};
/// The two shapes the gatherer's pool reads take of one gated device.
use super::clone_facts_live_pool::{
    ThePoolBehindTheGate,
    ThePoolsGuids,
};
/// Every read one clone's facts come from,
/// and the request they answer.
use super::clone_facts_reads::{
    CloneFactsIdentity,
    CloneFactsReads,
    CloneFactsRequest,
};
/// One object's accounting fields.
use super::clone_retention_checks::{
    AccountingFacts,
    LOGICAL_REFERENCED_PROPERTY,
    REFERENCED_PROPERTY,
    USED_PROPERTY,
    WRITTEN_PROPERTY,
};
/// What stands at one path.
use super::handler_checks::PathState;
/// What stands at the destruction plan's own file name.
use super::removal_plan_checks::PlanFileReading;
/// What a read of one object answered.
use super::zfs_checks::{
    Reading,
    reading_of,
};
/// This process's mount table,
/// read by the repository's own reader.
use crate::fs_mount_options::{
    MountFacts,
    parse_mount_table,
};
/// The path one clone's destruction plan is filed at.
use crate::worktree_clone_binding_record::registration_home::plans_folder;
/// The managed Git lock's state.
use crate::worktree_clone_managed_lock::{
    Classification,
    classify_the_managed_lock,
};
/// One dataset's or snapshot's accounting fields,
/// as the pool's reply spells them.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::value_of;
/// One dataset's identity and property reply,
/// read behind the compatibility gate,
/// and one step of a parent's listing,
/// which step 15's mark read takes.
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
/// The pool's own GUID for one dataset.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
/// The holders one all-namespace scan found,
/// as the folder that owns that read reports them.
use crate::worktree_clone_removal_proof_and_plan::holder_report::{
    TheHoldersOfOneClone,
    the_holders_of_one_clone,
};
/// The reader of an existing destruction plan.
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
/// The snapshots and bookmarks one clone holds.
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;
/// The pool behind the gate,
/// and the reads one removal's lineage proof makes.
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
    whole_inventory,
};
/// One slot's readings,
/// and the live pair its own reader takes.
use crate::worktree_clone_slot_state::slot_state_live_read::{
    read_the_slot_on_this_machine,
    the_mounts_on_this_machine,
};
use crate::worktree_clone_slot_state::slot_state_readings::SlotReadings;
/// The state of one origin snapshot's deferred-destroy mark.
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    OriginMark,
    origin_mark,
};
use std::path::{
    Path,
    PathBuf,
};

/// Where this machine's description of every task's mounts is.
const PROC_MOUNT_INFO: &str = "/proc/self/mountinfo";

/// What:
///  Why one pool read could not be made:
///  the gate refused it,
///  or the device could not be opened.
/// Why:
///  A database of the machine's own devices answers through several
///  structures,
///  so the pool is opened once for one clone's group and the reason it failed
///  travels with the reader rather than being asked again for each fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reads = new LiveCloneFactsReads();  // the pool opens here, behind the gate, and never again
/// ```
pub struct LiveCloneFactsReads {
    /// This machine's pool,
    /// read behind the gate,
    /// or the sentence that says why it could not be read.
    pool: Result<LiveRemovalReads, String>,
}

impl LiveCloneFactsReads {
    /// The reader of this machine,
    /// whose pool is opened and gated once.
    pub fn open() -> LiveCloneFactsReads {
        return LiveCloneFactsReads {
            pool: LiveRemovalReads::open(),
        };
    }

    /// The pool,
    /// or the gate's own sentence as the reason a fact is unproved.
    fn the_pool(&self) -> Result<&LiveRemovalReads, String> {
        return match &self.pool {
            Ok(pool) => Ok(pool),
            Err(why) => Err(why.clone()),
        };
    }
}

impl CloneFactsReads for LiveCloneFactsReads {
    fn slot_readings(
        &self,
        request: &CloneFactsRequest,
    ) -> SlotReadings {
        return read_the_slot_on_this_machine(
            request.common_dir.as_path(),
            request.slot.as_str(),
            request.slot_path.as_path(),
        );
    }

    fn mount_rows(&self) -> Vec<MountFacts> {
        let read: std::io::Result<Vec<u8>> = std::fs::read(PROC_MOUNT_INFO);
        // A table nobody read lists no mount, so no staging bind is named from
        // it; the state line's own reading of the same table answers unknown
        // where the table could not be read at all.
        return match read {
            Ok(bytes) => parse_mount_table(bytes.as_slice()),
            Err(_why) => Vec::new(),
        };
    }

    fn path_state(
        &self,
        path: &Path,
    ) -> PathState {
        return the_state_of_a_path(path);
    }

    fn registration_gitdir(
        &self,
        registration: &Path,
    ) -> Reading<PathBuf> {
        let file: PathBuf = registration.join("gitdir");
        let text: String = match std::fs::read_to_string(file.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Reading::Missing {
                    why: format!("{} holds no file of that name ({error})", file.display()),
                };
            }
            Err(error) => {
                return Reading::Unreadable {
                    why: format!("{} could not be read ({error})", file.display()),
                };
            }
        };
        let named: PathBuf = PathBuf::from(text.trim());
        // Git writes an absolute back-link and a hand-written one may be
        // relative to the registration, so the relative spelling is resolved
        // here rather than read as damage by the check.
        return Reading::Read(if named.is_absolute() {
            named
        } else {
            registration.join(named)
        });
    }

    fn lock_classification(
        &self,
        request: &CloneFactsRequest,
    ) -> Classification {
        let mounts = the_mounts_on_this_machine();
        return classify_the_managed_lock(
            request.common_dir.as_path(),
            request.slot.as_str(),
            request.slot_path.as_path(),
            &mounts,
            &ThePoolsGuids {
                pool: &self.pool,
            },
        );
    }

    fn plan_reading(
        &self,
        request: &CloneFactsRequest,
        clone_guid: DatasetGuid,
    ) -> PlanFileReading {
        let path: PathBuf =
            plans_folder(request.common_dir.as_path()).join(clone_guid.file_name().as_str());
        return match read_an_existing_plan(request.common_dir.as_path(), clone_guid) {
            Ok(None) => PlanFileReading::NoPlan,
            Ok(Some(plan)) => PlanFileReading::Standing {
                path,
                plan: Box::new(plan),
            },
            Err(refusal) => PlanFileReading::Unreadable {
                reason: refusal.message(path.as_path()),
                path,
            },
        };
    }

    fn clone_identity(
        &self,
        clone: &str,
    ) -> Reading<CloneFactsIdentity> {
        let pool: &LiveRemovalReads = match self.the_pool() {
            Ok(read) => read,
            Err(why) => {
                return Reading::Unreadable {
                    why,
                };
            }
        };
        return match pool.identity(clone) {
            Ok(held) => match DatasetGuid::parse(held.stats.dds_guid.to_string().as_str()) {
                Ok(guid) => Reading::Read(CloneFactsIdentity {
                    guid,
                    createtxg: held.stats.dds_creation_txg,
                }),
                Err(refusal) => Reading::Unreadable {
                    why: format!(
                        "the pool reported no GUID this route can name for `{clone}` ({refusal:?}), so no plan or snapshot inventory can be named for it"
                    ),
                },
            },
            Err(why) => Reading::Unreadable {
                why,
            },
        };
    }

    fn inventory(
        &self,
        clone: &str,
        clone_guid: DatasetGuid,
    ) -> Reading<LiveInventory> {
        let pool: &LiveRemovalReads = match self.the_pool() {
            Ok(read) => read,
            Err(why) => {
                return Reading::Unreadable {
                    why,
                };
            }
        };
        return reading_of(whole_inventory(pool, clone, clone_guid).map_err(|said| {
            return ReadFailure {
                said,
                errno: None,
            };
        }));
    }

    fn origin_mark(
        &self,
        origin_snapshot: &str,
    ) -> Reading<OriginMark> {
        let pool: &LiveRemovalReads = match self.the_pool() {
            Ok(read) => read,
            Err(why) => {
                return Reading::Unreadable {
                    why,
                };
            }
        };
        let reader: ThePoolBehindTheGate<'_> = ThePoolBehindTheGate {
            pool,
        };
        // Step 15's own read keeps a snapshot it cannot read as the mark's
        // unreadable answer, and the sentence it carries says which read
        // refused, so no absence and no mark is restated here.
        return Reading::Read(origin_mark(&reader, origin_snapshot));
    }

    fn accounting(
        &self,
        object: &str,
    ) -> Reading<AccountingFacts> {
        let pool: &LiveRemovalReads = match self.the_pool() {
            Ok(read) => read,
            Err(why) => {
                return Reading::Unreadable {
                    why,
                };
            }
        };
        return match pool.identity(object) {
            Ok(held) => Reading::Read(AccountingFacts {
                object: String::from(object),
                used: value_of(&held.properties, USED_PROPERTY),
                referenced: value_of(&held.properties, REFERENCED_PROPERTY),
                logical_referenced: value_of(&held.properties, LOGICAL_REFERENCED_PROPERTY),
                written: value_of(&held.properties, WRITTEN_PROPERTY),
            }),
            Err(why) => Reading::Unreadable {
                why,
            },
        };
    }

    fn holder_scan(
        &self,
        clone: &str,
        clone_guid: DatasetGuid,
    ) -> Result<TheHoldersOfOneClone, String> {
        return the_holders_of_one_clone(clone, clone_guid);
    }

    fn in_the_callers_session(
        &self,
        tasks: &[u32],
    ) -> Vec<u32> {
        return the_tasks_in_the_callers_session(tasks);
    }

    fn the_forced_arm_is_measured(&self) -> bool {
        // The forced arm's waits carry no value in this build (`T_wait`, `T_h`
        // and `T_stop` are unset in `worktree_clone_forced_removal`), so this
        // build offers no forced removal for a report to promise and `--force`
        // ends no holder. The round that measures those waits changes this
        // answer beside them.
        return false;
    }
}
