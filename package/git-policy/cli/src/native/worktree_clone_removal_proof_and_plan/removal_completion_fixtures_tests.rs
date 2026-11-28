//! What:
//!  The seams a whole-removal control answers:
//!  a scripted head device,
//!  a scripted all-namespace scan,
//!  a scripted Git settle,
//!  a clock and an unmount,
//!  and one call that runs the completion over the removal fixture.
//! Why:
//!  No control on a person's machine may destroy a dataset,
//!  read every process's namespace or remove a real registration,
//!  so each is a value a control scripts and the release answers differently.
//!  The pool,
//! the check and the snapshot device are the ones the destruction step's controls
//!  already script,
//!  so a whole-removal control differs from them in exactly the seams this file adds.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const completion = theCompletion(fixture, pool, check, device, head, holders, settle);
//! const done = completeTheRemoval(completion);
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::FreshHolderScan;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::RemovalClock;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    ORIGIN_CREATETXG,
    ORIGIN_DATASET,
    ORIGIN_DATASET_GUID,
    ORIGIN_SNAPSHOT_GUID,
    OWN_CREATETXG,
    OWN_GUID,
    OWN_SNAPSHOT,
    POOL,
    POOL_GUID,
    RemovalFixture,
    SANOID_CREATETXG,
    SANOID_GUID,
    SANOID_SNAPSHOT,
    guid,
    the_stats,
};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::DatasetDestroyDevice;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{
    RegistrationSettle,
    SettleAnswer,
    SettleRequest,
};
use crate::worktree_clone_removal_proof_and_plan::removal_staging_folder::RemovalStagingFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes;
use crate::worktree_removal_deferred_origin_check::fixtures::{
    Answer,
    Pool,
    origin_properties,
};
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    MARK_OFF,
    MARK_ON,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    Error,
    NvList,
    Operation,
};
use std::cell::{
    Cell,
    RefCell,
};
use std::path::PathBuf;
use std::rc::Rc;

/// The run nonce every control's plan carries.
pub const RUN_NONCE: &str = "00112233445566778899aabbccddeeff";

/// The moment every control's records and attempts carry.
pub const NOW: &str = "2026-11-25T09:12:44-05:00";

/// What:
///  The clock a control answers,
///  so the plan's process facts are the control's and not the runner's.
/// Why:
///  The owner and the moment are facts about the writing process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = { run: () => RUN_NONCE, owner: () => '48213:linux:9481733', started: () => NOW };
/// ```
pub struct ScriptedClock;

impl RemovalClock for ScriptedClock {
    fn run(&self) -> Result<RequestNonce, RemovalStop> {
        return Ok(RequestNonce::parse(RUN_NONCE).expect("the control's nonce is canonical"));
    }

    fn owner(&self) -> Result<String, RemovalStop> {
        return Ok(String::from("48213:linux:9481733"));
    }

    fn started(&self) -> String {
        return String::from(NOW);
    }
}

/// The unmount answer a control gives:
/// the intent's own nonce and the clone's GUID.
pub struct ADoneUnmount {
    /// The clone's GUID,
    /// whose answer this receipt carries.
    pub clone_guid: DatasetGuid,
}

impl RemovalUnmount for ADoneUnmount {
    fn request_the_unmount(
        &self,
        intent: &MountIntent,
    ) -> Result<UnmountAnswer, RemovalStop> {
        return Ok(UnmountAnswer {
            dataset: self.clone_guid,
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            outcome: ReceiptOutcome::Done,
            mount_id: None,
            failed_step: None,
            errno: None,
        });
    }
}

/// What:
///  The head device a control answers,
///  keeping every request and the plan the moment before each one.
/// Why:
///  A control's claim that the plan said `requested` before the request is a claim about this
///  seam,
///  so the seam reads the plan's own file where the release wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = { sent: [], failNext: false };
/// ```
pub struct ScriptedHead {
    /// Every dataset the device was asked to destroy,
    /// in order.
    pub sent: RefCell<Vec<String>>,
    /// The plan's own header word when each request arrived.
    pub words: RefCell<Vec<String>>,
    /// The plan folder the request reads.
    pub plan_folder: PathBuf,
    /// The clone's GUID,
    /// whose file name the plan is filed under.
    pub clone_guid: DatasetGuid,
    /// Whether the one request is refused.
    pub refuses: bool,
    /// Whether the clone vanishes from the absence proof,
    /// which a successful request makes true.
    pub hidden: Option<Rc<Cell<bool>>>,
}

/// What:
///  The head device a control answers,
///  keeping every request and the plan the moment before each one.
/// Why:
///  A control's claim that the plan said `requested` before the request is a claim about this
///  seam,
///  so the seam reads the plan's own file where the release wrote it,
///  and a successful request hides the clone from the probe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = ScriptedHead.succeeding(planFolder, cloneGuid, hidden);
/// ```
impl ScriptedHead {
    /// A device that answers success,
    /// reads the plan before its request,
    /// and hides the clone from the absence proof afterwards.
    pub fn succeeding(
        plan_folder: PathBuf,
        clone_guid: DatasetGuid,
        hidden: Rc<Cell<bool>>,
    ) -> ScriptedHead {
        return ScriptedHead {
            sent: RefCell::new(Vec::new()),
            words: RefCell::new(Vec::new()),
            plan_folder,
            clone_guid,
            refuses: false,
            hidden: Some(hidden),
        };
    }

    /// The same device,
    /// refusing its one request and leaving the clone answering.
    pub fn refusing(
        plan_folder: PathBuf,
        clone_guid: DatasetGuid,
        hidden: Rc<Cell<bool>>,
    ) -> ScriptedHead {
        return ScriptedHead {
            sent: RefCell::new(Vec::new()),
            words: RefCell::new(Vec::new()),
            plan_folder,
            clone_guid,
            refuses: true,
            hidden: Some(hidden),
        };
    }
}

impl DatasetDestroyDevice for ScriptedHead {
    fn destroy_the_dataset(
        &self,
        name: &str,
    ) -> Result<Option<NvList>, DeviceError> {
        self.sent.borrow_mut().push(String::from(name));
        let path: PathBuf = self.plan_folder.join(self.clone_guid.file_name());
        let word: String = std::fs::read_to_string(path.as_path())
            .ok()
            .and_then(|text| {
                return text
                    .lines()
                    .find(|line| return line.starts_with("head="))
                    .map(|line| return String::from(line));
            })
            .unwrap_or_default();
        self.words.borrow_mut().push(word);
        if self.refuses {
            return Err(DeviceError::Request {
                source: Error::Call {
                    operation: Operation::Destroy,
                    target: String::from(CLONE),
                    errno: libc::EBUSY,
                },
            });
        }
        if let Some(hidden) = &self.hidden {
            hidden.set(true);
        }
        return Ok(None);
    }
}

/// What:
///  The all-namespace scan a control answers:
///  the scans it gave,
///  in order,
///  and the names the release asked about.
/// Why:
///  The holder check's place is what the design fixes,
///  so a control scripts which scan sees a holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holders = { scans: [clear, holding], asked: [] };
/// ```
pub struct ScriptedHolders {
    /// One answer per call,
    /// the last repeated when the calls run out.
    pub answers: Vec<HolderScan>,
    /// Every dataset the release asked about.
    pub asked: RefCell<Vec<String>>,
    /// How many calls were made.
    pub calls: Cell<usize>,
}

impl ScriptedHolders {
    /// A scan that answers the same value every time.
    pub fn always(answer: HolderScan) -> ScriptedHolders {
        return ScriptedHolders {
            answers: vec![answer],
            asked: RefCell::new(Vec::new()),
            calls: Cell::new(0),
        };
    }

    /// One answer per call,
    /// then the last one again.
    pub fn in_order(answers: Vec<HolderScan>) -> ScriptedHolders {
        return ScriptedHolders {
            answers,
            asked: RefCell::new(Vec::new()),
            calls: Cell::new(0),
        };
    }
}

impl FreshHolderScan for ScriptedHolders {
    fn holders_of(
        &self,
        dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<HolderScan, RemovalStop> {
        self.asked.borrow_mut().push(String::from(dataset));
        let seen: usize = self.calls.get();
        self.calls.set(seen + 1);
        let at: usize = seen.min(self.answers.len() - 1);
        return Ok(self.answers[at].clone());
    }
}

/// A scan that found nothing and read everything.
pub fn a_clear_scan() -> HolderScan {
    return a_scan(Vec::new(), Vec::new());
}

/// A scan that found one holder in another namespace.
pub fn a_scan_with_a_holder() -> HolderScan {
    let holder: NamespaceHolder = NamespaceHolder {
        namespace: String::from("mnt:[4026532001]"),
        task: PathBuf::from("/proc/1234"),
        mounts: vec![HeldMount {
            mount_id: 41,
            root: PathBuf::from("/"),
            mount_point: PathBuf::from("/srv/holder"),
        }],
    };
    return a_scan(vec![holder], Vec::new());
}

/// A scan that read everything it met except one namespace's link.
pub fn a_scan_with_an_incomplete_read() -> HolderScan {
    let unfinished: IncompleteRead = IncompleteRead::NamespaceLinkUnreadable {
        thread: PathBuf::from("/proc/999/task/999"),
        reason: String::from("permission denied"),
    };
    return a_scan(Vec::new(), vec![unfinished]);
}

/// One scan of `CLONE` with the holders and reads a control names.
pub fn a_scan(
    holders: Vec<NamespaceHolder>,
    incomplete: Vec<IncompleteRead>,
) -> HolderScan {
    return HolderScan {
        dataset: DatasetGuid::parse(CLONE_GUID.to_string().as_str())
            .expect("the fixture's GUID is canonical"),
        source: String::from(CLONE),
        nonce: RequestNonce::parse(RUN_NONCE).expect("the control's nonce is canonical"),
        own_namespace: Some(String::from("mnt:[4026532000]")),
        namespaces_scanned: 2,
        own_mounts: Vec::new(),
        holders,
        incomplete,
        time: String::from(NOW),
    };
}

/// What:
///  The Git settle a control answers,
///  keeping every request it was handed.
/// Why:
///  The settle runs only after a destruction proof,
///  so a control asserts the request's proof names the slot and the clone the plan does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = { asked: [], refuse: false };
/// ```
pub struct ScriptedSettle {
    /// The refusal every call answers,
    /// when a control plants one.
    pub refuses: Option<String>,
    /// How many calls were made.
    pub calls: Cell<usize>,
}

impl ScriptedSettle {
    /// A settle that answers what it did.
    pub fn succeeding() -> ScriptedSettle {
        return ScriptedSettle {
            refuses: None,
            calls: Cell::new(0),
        };
    }

    /// A settle that refuses with one sentence.
    pub fn refusing(said: String) -> ScriptedSettle {
        return ScriptedSettle {
            refuses: Some(said),
            calls: Cell::new(0),
        };
    }
}

impl RegistrationSettle for ScriptedSettle {
    fn settle_the_registration(
        &self,
        request: &SettleRequest<'_>,
    ) -> Result<SettleAnswer, RemovalStop> {
        self.calls.set(self.calls.get() + 1);
        if request.proof.slot() != request.slot {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the proof"),
                said: String::from("the proof names another slot"),
            });
        }
        if let Some(said) = &self.refuses {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the registration"),
                said: said.clone(),
            });
        }
        return Ok(SettleAnswer {
            wrote_the_lost_lock: false,
            released_the_lock: true,
            removed_the_registration: true,
            removed_the_underlay: true,
            lines: Vec::new(),
        });
    }
}

/// What:
///  The staging folder a control that is not about the folder answers:
///  it makes nothing and counts the calls it was asked.
/// Why:
///  The order makes that folder before it asks for an unmount,
///  and a control about another step says here that the folder is not what it
///  is about,
///  so nothing it does reads this machine's account or its pin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staging = { asked: 0, makeTheFolder() { this.asked += 1 } };
/// ```
pub struct AStagingFolderAControlAnswers {
    /// How many times the order asked for the folder.
    pub asked: Cell<usize>,
}

impl AStagingFolderAControlAnswers {
    /// An answer that makes nothing and counts its calls.
    pub fn making_nothing() -> AStagingFolderAControlAnswers {
        return AStagingFolderAControlAnswers {
            asked: Cell::new(0),
        };
    }
}

impl RemovalStagingFolder for AStagingFolderAControlAnswers {
    fn make_the_folder(
        &self,
        _clone_guid: DatasetGuid,
    ) -> Result<(), RemovalStop> {
        self.asked.set(self.asked.get() + 1);
        // What: `Ok(())` is the success variant of `Result`, saying this step owes nothing further.
        // Why: the control says the folder stands, and the order goes on to ask for the unmount.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(());
    }
}

/// What:
///  One whole-removal call over the fixture,
///  with every seam a control scripts.
/// Why:
///  Fourteen arguments at each control would hide which one the control changed,
///  so one value carries them and each control replaces the seam it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const completion = theCompletion({ fixture, pool, check, device, head, holders, settle, budget });
/// ```
pub struct TheCompletion<'a> {
    /// The throwaway repository and its records.
    pub fixture: &'a RemovalFixture,
    /// The pool the proofs read.
    pub pool: &'a dyn RemovalPoolReads,
    /// The reader the deferred-origin checks read.
    pub check: &'a dyn DatasetReader,
    /// The reader the absence proofs and the live intent read.
    pub probe: &'a dyn DatasetReader,
    /// The device the snapshot requests go to.
    pub device: &'a dyn SnapshotDestroyDevice,
    /// The device the head destroy goes to.
    pub head: &'a dyn DatasetDestroyDevice,
    /// The fresh all-namespace scans.
    pub holders: &'a dyn FreshHolderScan,
    /// The Git settle.
    pub settle: &'a dyn RegistrationSettle,
    /// The answer a stop point asks,
    /// and nothing for a control that is not about one.
    pub cancelling: Option<&'a dyn crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling>,
    /// The bytes one request may carry.
    pub budget: RequestBudget,
}

/// What:
///  Run the completion over one fixture.
/// Why:
///  Every control differs in one seam,
///  so one call keeps the rest of the arrangement out of each control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = completeTheRemoval(theCompletion({...}));
/// ```
pub fn complete(
    the: &TheCompletion<'_>
) -> Result<crate::worktree_clone_removal_proof_and_plan::removal_report::TheRemovalDone, RemovalStop>
{
    return complete_with_the_moment(the, NOW);
}

/// What:
///  The same call over the moment a control hands the records,
///  which is the removal's command-path clock in the release.
/// Why:
///  The origin cleanup record's own reader decides which spellings of a moment it
///  accepts,
///  so a control that is about that record's words hands in the moment it is about
///  rather than this file's healthy one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = completeTheRemoval(theCompletion({...}), '2026-11-25T09:12:44-05:00');
/// ```
pub fn complete_with_the_moment(
    the: &TheCompletion<'_>,
    moment: &str,
) -> Result<crate::worktree_clone_removal_proof_and_plan::removal_report::TheRemovalDone, RemovalStop>
{
    let recorded = the.fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let staging: AStagingFolderAControlAnswers = AStagingFolderAControlAnswers::making_nothing();
    return complete_with(the, moment, &staging, &unmount);
}

/// What:
///  The same call over the folder seam and the unmount seam a control states,
///  which is how a control about either one observes what the order did.
/// Why:
///  The order makes the folder before it asks for an unmount,
///  so a control that is about that order hands in both answers and reads them
///  after the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = completeTheRemoval(theCompletion({...}, staging, unmount));
/// ```
pub fn complete_with(
    the: &TheCompletion<'_>,
    moment: &str,
    staging: &dyn RemovalStagingFolder,
    unmount: &dyn crate::worktree_clone_removal_proof_and_plan::unmount_request::RemovalUnmount,
) -> Result<crate::worktree_clone_removal_proof_and_plan::removal_report::TheRemovalDone, RemovalStop>
{
    let recorded = the.fixture.recorded();
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: the.fixture.common(),
        slot: recorded.slot.as_str(),
    };
    let input: RemovalCompletion<'_> = RemovalCompletion {
        entry: &at,
        reads: the.pool,
        unmount,
        staging,
        clock: &ScriptedClock,
        check: the.check,
        probe: the.probe,
        snapshots: the.device,
        head: the.head,
        holders: the.holders,
        settle: the.settle,
        cancelling: the.cancelling,
        budget: the.budget,
        now: moment,
        force: crate::worktree_clone_removal_proof_and_plan::removal_force::RemovalForce::none(),
    };
    return crate::worktree_clone_removal_proof_and_plan::removal_completion::complete_the_removal(
        &input,
    );
}

/// What:
///  A pool whose lists change when a request destroys a name,
///  which is what a real batch leaves behind and a fixed list cannot.
/// Why:
///  The completion settles every name by a fresh read,
///  so a pool that kept listing a destroyed snapshot would make every removal stop as if the
///  kernel had done nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = LivePool.of(lists); const device = ClearingDevice.over(pool);
/// ```
pub struct LivePool {
    /// One identity reply per name,
    /// in the order the control wrote them.
    pub identities: Vec<(String, DatasetIdentity)>,
    /// One snapshot list per dataset,
    /// which a destroy removes from.
    pub lists: Rc<RefCell<Vec<(String, Vec<LiveObject>)>>>,
    /// The clone's bookmarks,
    /// which no destroy of this route removes.
    pub bookmarks: Rc<RefCell<Vec<LiveObject>>>,
}

impl LivePool {
    /// What:
    ///  One pool over the fixture's own tuple,
    ///  with the origin snapshot an origin cleanup record can name.
    /// Why:
    ///  The clone's two own snapshots and the checkout's one origin snapshot are the design's
    ///  healthy state,
    ///  and the origin snapshot's own name is the record's key.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pool = LivePool.healthy(originOwnName);
    /// ```
    pub fn healthy(origin_own_name: &str) -> LivePool {
        let clone_list: Vec<LiveObject> = vec![
            LiveObject {
                kind: EntryKind::Snapshot,
                name: String::from(OWN_SNAPSHOT),
                guid: guid(OWN_GUID),
                createtxg: OWN_CREATETXG,
                owning_dataset: String::from(CLONE),
                owning_dataset_guid: guid(CLONE_GUID),
            },
            LiveObject {
                kind: EntryKind::Snapshot,
                name: String::from(SANOID_SNAPSHOT),
                guid: guid(SANOID_GUID),
                createtxg: SANOID_CREATETXG,
                owning_dataset: String::from(CLONE),
                owning_dataset_guid: guid(CLONE_GUID),
            },
        ];
        let origin_list: Vec<LiveObject> = vec![LiveObject {
            kind: EntryKind::Snapshot,
            name: String::from(origin_own_name),
            guid: guid(ORIGIN_SNAPSHOT_GUID),
            createtxg: ORIGIN_CREATETXG,
            owning_dataset: String::from(ORIGIN_DATASET),
            owning_dataset_guid: guid(ORIGIN_DATASET_GUID),
        }];
        let identities: Vec<(String, DatasetIdentity)> = vec![
            (
                String::from(POOL),
                DatasetIdentity {
                    stats: the_stats(POOL_GUID, false, 1, 0, ""),
                    properties: NvList::new(),
                },
            ),
            (
                String::from(CLONE),
                DatasetIdentity {
                    stats: the_stats(
                        CLONE_GUID,
                        false,
                        CLONE_CREATETXG,
                        0,
                        format!("{ORIGIN_DATASET}@{origin_own_name}").as_str(),
                    ),
                    properties: NvList::new(),
                },
            ),
            (
                String::from(ORIGIN_DATASET),
                DatasetIdentity {
                    stats: the_stats(ORIGIN_DATASET_GUID, false, 2, 1, ""),
                    properties: NvList::new(),
                },
            ),
            (
                format!("{ORIGIN_DATASET}@{origin_own_name}"),
                DatasetIdentity {
                    stats: the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, 0, ""),
                    properties: origin_properties(Some(MARK_OFF), Some(0)),
                },
            ),
        ];
        return LivePool {
            identities,
            lists: Rc::new(RefCell::new(vec![
                (String::from(CLONE), clone_list),
                (String::from(ORIGIN_DATASET), origin_list),
            ])),
            bookmarks: Rc::new(RefCell::new(Vec::new())),
        };
    }
}

impl RemovalPoolReads for LivePool {
    fn identity(
        &self,
        name: &str,
    ) -> Result<DatasetIdentity, String> {
        for (scripted, identity) in &self.identities {
            if scripted == name {
                return Ok(DatasetIdentity {
                    stats: identity.stats,
                    properties: identity.properties.clone(),
                });
            }
        }
        return Err(format!("`{name}` does not answer this scripted pool"));
    }

    fn snapshots_of(
        &self,
        dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        for (scripted, objects) in self.lists.borrow().iter() {
            if scripted == dataset {
                return Ok(objects.clone());
            }
        }
        return Ok(Vec::new());
    }

    fn bookmarks_of(
        &self,
        _dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        return Ok(self.bookmarks.borrow().clone());
    }
}

/// What:
///  The snapshot device a whole-removal control answers:
///  every request's names,
///  and the pool list it empties,
///  so the next read sees what the kernel did.
/// Why:
///  A device that answered success and left the list whole would make the completion stop where
///  a real removal goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const device = ClearingDevice.over(pool);
/// ```
pub struct ClearingDevice {
    /// The pool's lists,
    /// which this device empties by name.
    pub lists: Rc<RefCell<Vec<(String, Vec<LiveObject>)>>>,
    /// Every request's names,
    /// in order.
    pub sent: RefCell<Vec<Vec<String>>>,
    /// The names the device refuses,
    /// when a control plants a refusal.
    pub refuses: Vec<String>,
}

impl ClearingDevice {
    /// A device that destroys what it is asked for,
    /// over one pool's own lists.
    pub fn over(pool: &LivePool) -> ClearingDevice {
        return ClearingDevice {
            lists: Rc::clone(&pool.lists),
            sent: RefCell::new(Vec::new()),
            refuses: Vec::new(),
        };
    }
}

/// What:
///  A pool reader that loses the clone once the head's request has run,
///  which is what a real destroy leaves and a fixed pool cannot.
/// Why:
///  The head's word is decided by the absence proof,
///  so a probe that kept answering the clone would make every successful head read `refused`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const probe = EpochProbe.over(pool, hidden);
/// ```
pub struct EpochProbe {
    /// The pool the reads answer from until the clone is hidden.
    pub pool: Pool,
    /// Whether the clone has been destroyed.
    pub hidden: Rc<Cell<bool>>,
}

/// What:
///  The checks reader of a removal whose origin is marked once the batch has gone:
///  the pool a check reads before the batch's request,
///  and the pool it reads after it.
/// Why:
///  The deferred-origin check runs before each snapshot request and again before the head,
///  so a mark set between the two is the fact the head's own check must refuse
///  (`SPEC.md`,
///  "The same check runs again before the head destroy").
///  A pool that stayed unmarked would let the head through,
///  and a control could not tell the head's check from the batch's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const check = { identity: (name) => device.sent.length === 0 ? before.identity(name) : after.identity(name) };
/// ```
pub struct MarkedAfterTheBatch<'a> {
    /// The pool the check reads while the batch has sent nothing.
    pub before: &'a Pool,
    /// The pool the check reads once the batch's request has gone.
    pub after: &'a Pool,
    /// The device whose own request is the moment the mark appears.
    pub device: &'a ClearingDevice,
}

impl DatasetReader for MarkedAfterTheBatch<'_> {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        if self.device.sent.borrow().is_empty() {
            return self.before.identity(dataset);
        }
        return self.after.identity(dataset);
    }

    fn next_child(
        &self,
        parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return self.before.next_child(parent, cursor);
    }
}

impl DatasetReader for EpochProbe {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        if self.hidden.get() && dataset == CLONE {
            return Err(ReadFailure {
                said: String::from("no such dataset"),
                errno: Some(libc::ENOENT),
            });
        }
        return self.pool.identity(dataset);
    }

    fn next_child(
        &self,
        parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        if self.hidden.get() {
            return Ok(None);
        }
        return self.pool.next_child(parent, cursor);
    }
}

impl SnapshotDestroyDevice for ClearingDevice {
    fn destroy_snaps(
        &self,
        _pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        self.sent.borrow_mut().push(
            names
                .iter()
                .map(|name| return String::from(*name))
                .collect(),
        );
        for refused in &self.refuses {
            if names.contains(&refused.as_str()) {
                return Err(DeviceError::Request {
                    source: Error::Elements {
                        operation: Operation::DestroySnaps,
                        target: String::from(POOL),
                        errno: libc::EBUSY,
                        elements: Vec::new(),
                    },
                });
            }
        }
        let mut lists = self.lists.borrow_mut();
        for pair in lists.iter_mut() {
            pair.1
                .retain(|object| return !names.contains(&object.full_name().as_str()));
        }
        return Ok(None);
    }
}

/// The origin snapshot's own name,
/// which begins with the add's prefix and its nonce.
pub const ORIGIN_OWN_NAME: &str = "git-worktree-clone-origin-445566778899aabbccddeeff00112233";

/// The origin snapshot's full name.
pub fn the_origin_full_name() -> String {
    return format!("{ORIGIN_DATASET}@{ORIGIN_OWN_NAME}");
}

/// What:
///  One healthy removal fixture whose origin snapshot a record can name.
/// Why:
///  The step 14 fixture names its origin snapshot `origin`,
///  which the cleanup record's grammar refuses,
///  so every control that reaches the origin record starts from the same tuple
///  with the add's own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aFixture('x');
/// ```
pub fn a_fixture(name: &str) -> RemovalFixture {
    return RemovalFixture::healthy(name).with_origin_snapshot(the_origin_full_name().as_str());
}

/// The check's pool,
/// keyed by the origin snapshot's full name.
pub fn the_checks_pool() -> Pool {
    return the_checks_pool_with(MARK_OFF);
}

/// The same pool with the origin snapshot's `defer_destroy` mark as a control
/// names it,
/// which is the one field a mark control changes.
pub fn the_checks_pool_with(mark: u64) -> Pool {
    return the_checks_pool_of(mark, NvList::new());
}

/// The same pool with the clone's own properties as a control names them,
/// which is where a live mount intent stands.
pub fn the_checks_pool_of(
    mark: u64,
    clone_properties: NvList,
) -> Pool {
    let full: String = the_origin_full_name();
    return Pool::new(vec![
        (
            CLONE,
            Answer::of(
                the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, full.as_str()),
                clone_properties,
            ),
        ),
        (
            ORIGIN_DATASET,
            Answer::of(
                the_stats(ORIGIN_DATASET_GUID, false, 2, 1, ""),
                NvList::new(),
            ),
        ),
        (
            full.as_str(),
            Answer::of(
                the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, 1, ""),
                origin_properties(Some(mark), Some(0)),
            ),
        ),
    ]);
}

/// The clone's own mount intent,
/// in the shape OpenZFS answers for a local property:
/// the pair of `value` and `source`,
/// whose source is the clone's own name.
/// Why:
///  A plan the head-present path adopts needs a live intent,
///  because nothing else proves the unmount it claims
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "The removal intent").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const properties = anIntentProperty('unmount');
/// ```
pub fn an_intent_property(operation: &str) -> NvList {
    let mut inner: NvList = NvList::new();
    inner.add_str("value", format!("v1:{RUN_NONCE}:{operation}").as_str());
    inner.add_str("source", CLONE);
    let mut properties: NvList = NvList::new();
    properties.add_nvlist(
        crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY,
        inner,
    );
    return properties;
}

/// The check's pool whose origin snapshot is marked for deferred destroy,
/// which is the state a second removal's act leaves.
pub fn a_marked_checks_pool() -> Pool {
    return the_checks_pool_with(MARK_ON);
}

/// The budget both of the clone's own snapshots need,
/// measured from the exact request they become.
pub fn the_budget() -> RequestBudget {
    let names: Vec<String> = vec![
        format!("{CLONE}@{OWN_SNAPSHOT}"),
        format!("{CLONE}@{SANOID_SNAPSHOT}"),
    ];
    let bytes: usize =
        encoded_request_bytes(POOL, names.as_slice()).expect("both names pack into a request");
    return RequestBudget::of_input_bytes(bytes);
}

/// Whether one record file stands in the wrapper's home,
/// by the name it is filed under.
pub fn a_record_stands(
    fixture: &RemovalFixture,
    name: &str,
) -> bool {
    return std::fs::symlink_metadata(home_folder(fixture.common()).join(name).as_path()).is_ok();
}
