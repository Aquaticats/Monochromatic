//! What:
//!  The gatherer's controls:
//!  one scripted seam built from the folder's healthy clone,
//!  the whole fact set one gather answers,
//!  and one control per read that shows which fact a read's answer lands in
//!  and which answer the check prints when the read fails.
//! Why:
//!  The gatherer is the one place a live answer becomes a fact,
//!  so a field swapped between two facts,
//! or an unknown answer swallowed into
//!  a plausible zero,
//! would reach a person's report unseen.
//!  Every control here changes one scripted answer and reads the one field
//!  that moved,
//!  and the healthy one asserts the whole set against
//!  `route_test_facts::healthy_clone`,
//!  which the checks' own controls already judge.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scripted = readsOfTheHealthyClone(healthyClone());
//! scripted.plan = { kind: 'unreadable', path, reason };
//! expect(gatherCloneFacts(request, scripted).blocked.planStands).toBe(true);
//! ```

/// The checks whose answers a read's failure moves.
use super::blocked_removal_checks::check_blocked_removal;
use super::check_outcome::CheckOutcome;
/// The gatherer,
/// and the seam it reads through.
use super::clone_facts_gathering::gather_clone_facts;
use super::clone_facts_reads::{
    CloneFactsIdentity,
    CloneFactsReads,
    CloneFactsRequest,
};
use super::clone_origin_checks::check_clone_origin;
/// One object's accounting fields.
use super::clone_retention_checks::{
    AccountingFacts,
    check_clone_retention,
};
use super::clone_snapshot_checks::check_clone_snapshots;
/// What stands at one path,
/// and the registration line.
use super::handler_checks::PathState;
/// The namespaces that hold a clone.
use super::holder_checks::check_holders;
use super::registration_checks::check_registration;
use super::removal_plan_checks::{
    PlanFileReading,
    check_removal_plan,
};
/// The healthy clone every control starts from,
/// and the facts one gather answers.
use super::route_checks::CloneFacts;
use super::route_test_facts::healthy_clone;
use super::zfs_checks::Reading;
/// This process's mount rows.
use crate::fs_mount_options::MountFacts;
/// The binding words a record without a clone is written at,
/// and what one inventory entry is.
use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_binding_record::binding_words::Phase;
/// The managed Git lock's state.
use crate::worktree_clone_managed_lock::Classification;
/// The pool's own GUID for one dataset.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
/// The holders one all-namespace scan found,
/// as the removal folder reports them.
use crate::worktree_clone_removal_proof_and_plan::holder_report::{
    AProvenHolder,
    TheHoldersOfOneClone,
};
/// The snapshots and bookmarks one clone holds.
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{
    LiveInventory,
    LiveObject,
};
/// The record,
/// plan,
/// intent and mounts step 12 read.
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    SlotReadings,
};
use crate::worktree_clone_slot_state::slot_state_test_records::record_at;
/// The state of one origin snapshot's mark,
/// and the count of user holds it carries.
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    OriginMark,
    UserHolds,
};
use std::cell::Cell;
use std::path::{
    Path,
    PathBuf,
};

/// The task the request says asked for the report.
const CALLER_TASK: u32 = 4242;

/// The holder's namespace the scan names.
const A_HOLDERS_NAMESPACE: &str = "mnt:[4026532001]";

/// The task the scan saw in that namespace.
const A_HOLDERS_TASK: u32 = 777;

/// What:
///  Every answer a control scripts,
///  one field per read of the seam.
/// Why:
///  A control changes one field and reads the one fact that moved,
///  so the seam's shape is the whole of what a control can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scripted = { readings, mountRows, pathState, gitdir, lock, plan, identity, inventory, mark, accounting, scan, session, forced };
/// ```
struct ScriptedReads {
    /// One slot's readings.
    readings: SlotReadings,
    /// This process's mount table.
    mount_rows: Vec<MountFacts>,
    /// What stands at the registration's path.
    path_state: PathState,
    /// The registration's `gitdir` answer.
    gitdir: Reading<PathBuf>,
    /// The managed Git lock's state.
    lock: Classification,
    /// What stands at the destruction plan's own file name.
    plan: PlanFileReading,
    /// The clone dataset's identity.
    identity: Reading<CloneFactsIdentity>,
    /// The clone's snapshots and bookmarks.
    inventory: Reading<LiveInventory>,
    /// The origin snapshot's mark,
    /// read only where the record names an origin.
    mark: Reading<OriginMark>,
    /// One accounting answer per object the control states.
    accounting: Vec<(String, Reading<AccountingFacts>)>,
    /// The scan's answer,
    /// or the reason no scan could be made.
    scan: Result<TheHoldersOfOneClone, String>,
    /// Which holder tasks the caller's session keeps.
    session: Vec<u32>,
    /// Whether this build is said to offer a measured forced arm.
    forced: bool,
    /// How many pool reads were asked for.
    pool_reads: Cell<usize>,
    /// How many plan reads were asked for.
    plan_reads: Cell<usize>,
    /// How many scans were asked for.
    scan_reads: Cell<usize>,
}

impl CloneFactsReads for ScriptedReads {
    fn slot_readings(
        &self,
        _request: &CloneFactsRequest,
    ) -> SlotReadings {
        return self.readings.clone();
    }

    fn mount_rows(&self) -> Vec<MountFacts> {
        return self.mount_rows.clone();
    }

    fn path_state(
        &self,
        _path: &Path,
    ) -> PathState {
        return self.path_state.clone();
    }

    fn registration_gitdir(
        &self,
        _registration: &Path,
    ) -> Reading<PathBuf> {
        return self.gitdir.clone();
    }

    fn lock_classification(
        &self,
        _request: &CloneFactsRequest,
    ) -> Classification {
        return self.lock.clone();
    }

    fn plan_reading(
        &self,
        _request: &CloneFactsRequest,
        _clone_guid: DatasetGuid,
    ) -> PlanFileReading {
        self.plan_reads.set(self.plan_reads.get() + 1);
        return self.plan.clone();
    }

    fn clone_identity(
        &self,
        _clone: &str,
    ) -> Reading<CloneFactsIdentity> {
        self.pool_reads.set(self.pool_reads.get() + 1);
        return self.identity.clone();
    }

    fn inventory(
        &self,
        _clone: &str,
        _clone_guid: DatasetGuid,
    ) -> Reading<LiveInventory> {
        self.pool_reads.set(self.pool_reads.get() + 1);
        return self.inventory.clone();
    }

    fn origin_mark(
        &self,
        _origin_snapshot: &str,
    ) -> Reading<OriginMark> {
        self.pool_reads.set(self.pool_reads.get() + 1);
        return self.mark.clone();
    }

    fn accounting(
        &self,
        object: &str,
    ) -> Reading<AccountingFacts> {
        self.pool_reads.set(self.pool_reads.get() + 1);
        return match self
            .accounting
            .iter()
            .find(|(named, _answer)| return named == object)
        {
            Some((_named, answer)) => answer.clone(),
            None => Reading::Unreadable {
                why: format!("the control scripted no accounting for `{object}`"),
            },
        };
    }

    fn holder_scan(
        &self,
        _clone: &str,
        _clone_guid: DatasetGuid,
    ) -> Result<TheHoldersOfOneClone, String> {
        self.scan_reads.set(self.scan_reads.get() + 1);
        return self.scan.clone();
    }

    fn in_the_callers_session(
        &self,
        _tasks: &[u32],
    ) -> Vec<u32> {
        return self.session.clone();
    }

    fn the_forced_arm_is_measured(&self) -> bool {
        return self.forced;
    }
}

/// The request one clone's facts are gathered for.
fn request(facts: &CloneFacts) -> CloneFactsRequest {
    return CloneFactsRequest {
        common_dir: PathBuf::from("/home/user/Monochromatic/.git"),
        slot: String::from(facts.lock.slot.as_str()),
        slot_path: facts.state.readings.slot_path.clone(),
        cache: PathBuf::from("/home/user/.cache/monochromatic-git-policy-cli"),
        caller_task: CALLER_TASK,
    };
}

/// An empty scan that found nothing and read everything.
fn a_clear_scan() -> TheHoldersOfOneClone {
    return TheHoldersOfOneClone {
        holders: Vec::new(),
        own_mounts: Vec::new(),
        own_namespace: Some(String::from("mnt:[4026531840]")),
        incomplete: Vec::new(),
    };
}

/// What:
///  The seam scripted from the folder's healthy clone,
///  so the gatherer's own composition is what every control judges.
fn reads_of_the_healthy_clone(facts: &CloneFacts) -> ScriptedReads {
    return ScriptedReads {
        readings: facts.state.readings.clone(),
        mount_rows: Vec::new(),
        path_state: facts.registration.at_the_path.clone(),
        gitdir: Reading::Read(
            facts
                .registration
                .gitdir
                .clone()
                .expect("the healthy clone's registration names its slot's .git"),
        ),
        lock: facts.lock.classification.clone(),
        plan: facts.plan.reading.clone(),
        identity: Reading::Read(CloneFactsIdentity {
            guid: DatasetGuid::parse("4821193048571903102").expect("a canonical GUID"),
            createtxg: facts.snapshots.clone_createtxg,
        }),
        inventory: facts.snapshots.live.clone(),
        mark: facts
            .origin
            .live
            .clone()
            .expect("the healthy clone's origin is read"),
        accounting: Vec::from([(
            facts.retention.clone.clone(),
            facts.retention.dataset.clone(),
        )]),
        scan: Ok(a_clear_scan()),
        session: Vec::new(),
        forced: false,
        pool_reads: Cell::new(0),
        plan_reads: Cell::new(0),
        scan_reads: Cell::new(0),
    };
}

/// The line one check printed.
fn line(outcome: &CheckOutcome) -> String {
    return format!("{outcome:?}");
}

/// Whether one check answered unknown.
fn is_unknown(outcome: &CheckOutcome) -> bool {
    return matches!(outcome, CheckOutcome::Unknown { .. });
}

/// Whether one check answered a finding.
fn is_finding(outcome: &CheckOutcome) -> bool {
    return matches!(outcome, CheckOutcome::Finding { .. });
}

//region The whole set

/// What:
///  The healthy clone's reads gather the healthy clone's facts.
/// Why:
///  Every other control changes one answer of this set,
///  so this one proves the composition is right where nothing is broken.
#[test]
fn the_healthy_reads_gather_the_healthy_clone() {
    let healthy: CloneFacts = healthy_clone();
    let gathered: CloneFacts =
        gather_clone_facts(&request(&healthy), &reads_of_the_healthy_clone(&healthy));
    assert_eq!(gathered.state, healthy.state);
    assert_eq!(gathered.registration, healthy.registration);
    assert_eq!(gathered.lock, healthy.lock);
    assert_eq!(gathered.plan, healthy.plan);
    assert_eq!(gathered.snapshots, healthy.snapshots);
    assert_eq!(gathered.origin, healthy.origin);
    assert_eq!(gathered.retention, healthy.retention);
    assert_eq!(gathered.blocked, healthy.blocked);
    assert_eq!(gathered.holders, healthy.holders);
}

//endregion

//region Each read's answer

/// What:
///  A plan nobody read stands,
///  so the blocked line judges a removal in flight.
/// Why:
///  A file at the plan's own name is what a removal in flight leaves,
///  and reading "no plan" there would tell a person nothing is pending.
#[test]
fn a_plan_nobody_read_stands_and_is_unknown() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.plan = PlanFileReading::Unreadable {
        path: PathBuf::from("/repo/.git/cli-git-clone-plans/4821193048571903102"),
        reason: String::from("the plan's header is not one this build reads"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert!(gathered.snapshots.plan_stands, "a plan stands at its name");
    assert!(gathered.blocked.plan_stands, "a plan stands at its name");
    assert!(
        gathered.blocked.recorded.is_empty(),
        "a plan nobody read states no outcome"
    );
    let plan: CheckOutcome = check_removal_plan(&gathered.plan);
    assert!(is_unknown(&plan), "{}", line(&plan));
}

/// What:
///  A registration whose `gitdir` nobody read answers unknown at the
///  registration rather than as the half-made state a missing file means.
#[test]
fn a_gitdir_nobody_read_is_unknown_at_the_registration() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.gitdir = Reading::Unreadable {
        why: String::from("Permission denied (os error 13)"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(gathered.registration.gitdir, None);
    let outcome: CheckOutcome = check_registration(&gathered.registration);
    assert!(is_unknown(&outcome), "{}", line(&outcome));
}

/// What:
///  A registration with no `gitdir` file is the state a killed add leaves,
///  and it is a finding rather than an unknown.
#[test]
fn a_registration_with_no_gitdir_file_is_a_finding() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.gitdir = Reading::Missing {
        why: String::from("no such file or directory"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(gathered.registration.gitdir, None);
    let outcome: CheckOutcome = check_registration(&gathered.registration);
    assert!(is_finding(&outcome), "{}", line(&outcome));
}

/// What:
///  A pool nobody could read leaves the inventory and the retention unknown,
///  with the pool's own reason.
#[test]
fn a_pool_nobody_read_leaves_the_inventory_unknown() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.identity = Reading::Unreadable {
        why: String::from("the ZFS compatibility gate refused"),
    };
    reads.inventory = Reading::Unreadable {
        why: String::from("the ZFS compatibility gate refused"),
    };
    reads.accounting = Vec::new();
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert!(gathered.snapshots.live == reads.inventory);
    let snapshots: CheckOutcome = check_clone_snapshots(&gathered.snapshots);
    assert!(is_unknown(&snapshots), "{}", line(&snapshots));
    let retention: CheckOutcome = check_clone_retention(&gathered.retention);
    assert!(is_unknown(&retention), "{}", line(&retention));
}

/// What:
///  A clone the pool lost is the snapshot line's finding,
///  because the kernel proved the absence.
#[test]
fn a_clone_the_pool_lost_is_a_finding() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.inventory = Reading::Missing {
        why: String::from("dataset does not exist"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    let outcome: CheckOutcome = check_clone_snapshots(&gathered.snapshots);
    assert!(is_finding(&outcome), "{}", line(&outcome));
}

/// What:
///  An own snapshot whose accounting nobody read is unknown,
///  naming that snapshot.
#[test]
fn an_own_snapshots_accounting_nobody_read_is_unknown() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    let own: String = format!("{}@worktree-own", healthy.retention.clone);
    reads.inventory = Reading::Read(LiveInventory {
        dataset: healthy.retention.clone.clone(),
        dataset_guid: DatasetGuid::parse("4821193048571903102").expect("a canonical GUID"),
        objects: Vec::from([LiveObject {
            kind: EntryKind::Snapshot,
            name: String::from("worktree-own"),
            guid: DatasetGuid::parse("991").expect("a canonical GUID"),
            createtxg: healthy.snapshots.clone_createtxg + 1,
            owning_dataset: healthy.retention.clone.clone(),
            owning_dataset_guid: DatasetGuid::parse("4821193048571903102")
                .expect("a canonical GUID"),
        }]),
    });
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(gathered.retention.snapshots.len(), 1);
    assert_eq!(gathered.retention.snapshots[0].object, own);
    let outcome: CheckOutcome = check_clone_retention(&gathered.retention);
    assert!(is_unknown(&outcome), "{}", line(&outcome));
}

/// What:
///  A holder the scan named is named by its namespace and its task,
///  and the blocked line says a removal is stopped.
#[test]
fn a_holder_is_named_by_its_namespace_and_its_task() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    let mount_point: PathBuf =
        PathBuf::from("/home/user/.cache/cli-git/staged-4821193048571903102");
    reads.scan = Ok(TheHoldersOfOneClone {
        holders: Vec::from([AProvenHolder {
            namespace: String::from(A_HOLDERS_NAMESPACE),
            task: A_HOLDERS_TASK,
            mount_point: mount_point.clone(),
        }]),
        own_mounts: Vec::new(),
        own_namespace: Some(String::from("mnt:[4026531840]")),
        incomplete: Vec::new(),
    });
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(gathered.holders.holders.len(), 1);
    assert_eq!(gathered.holders.holders[0].namespace, A_HOLDERS_NAMESPACE);
    assert_eq!(
        gathered.holders.holders[0].representative_pid,
        A_HOLDERS_TASK
    );
    assert_eq!(gathered.holders.holders[0].mount_point, mount_point);
    let holders: CheckOutcome = check_holders(&gathered.holders);
    let CheckOutcome::Finding {
        detail,
        ..
    } = &holders
    else {
        panic!("a holder is a finding: {holders:?}");
    };
    assert!(detail.contains(A_HOLDERS_NAMESPACE), "{detail}");
    assert!(
        detail.contains(A_HOLDERS_TASK.to_string().as_str()),
        "{detail}"
    );
    assert!(
        gathered.blocked.holders.holders == gathered.holders.holders,
        "the blocked line reads the same scan"
    );
}

/// What:
///  A scan nobody could make is unknown and never an absence.
#[test]
fn a_scan_nobody_made_is_unknown() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.scan = Err(String::from(
        "the scan's role for a whole command by name of its own was not raised",
    ));
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert!(gathered.holders.holders.is_empty());
    assert_eq!(gathered.holders.incomplete.len(), 1);
    let outcome: CheckOutcome = check_holders(&gathered.holders);
    assert!(is_unknown(&outcome), "{}", line(&outcome));
}

/// What:
///  The caller's own mount is a holder whose task is the caller's.
#[test]
fn the_callers_own_mount_is_a_holder_with_the_callers_task() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    let mut scan: TheHoldersOfOneClone = a_clear_scan();
    scan.own_mounts = Vec::from([PathBuf::from("/repo/.claude/worktrees/one")]);
    scan.own_namespace = None;
    reads.scan = Ok(scan);
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(gathered.holders.holders.len(), 1);
    assert_eq!(gathered.holders.holders[0].representative_pid, CALLER_TASK);
    assert_eq!(
        gathered.holders.holders[0].namespace,
        super::clone_facts_gathering_holders::THE_READERS_OWN_NAMESPACE
    );
}

/// What:
///  The session read names the holder the caller's session keeps,
///  and the blocked line says the caller's own session holds it.
#[test]
fn the_callers_session_is_read_from_the_holder_tasks() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    let mut scan: TheHoldersOfOneClone = a_clear_scan();
    scan.holders = Vec::from([AProvenHolder {
        namespace: String::from(A_HOLDERS_NAMESPACE),
        task: A_HOLDERS_TASK,
        mount_point: PathBuf::from("/home/user/.cache/cli-git/staged-4821193048571903102"),
    }]);
    reads.scan = Ok(scan);
    reads.session = Vec::from([A_HOLDERS_TASK]);
    // A holder alone is the plan's own line, so a plan stands here for the
    // blocked line to judge the session it spares.
    reads.plan = PlanFileReading::Unreadable {
        path: PathBuf::from("/repo/.git/cli-git-clone-plans/4821193048571903102"),
        reason: String::from("the control wrote a file where a plan is filed"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(
        gathered.blocked.in_the_callers_session,
        Vec::from([A_HOLDERS_TASK])
    );
    let blocked: CheckOutcome = check_blocked_removal(&gathered.blocked);
    let CheckOutcome::Finding {
        detail,
        ..
    } = &blocked
    else {
        panic!("a holder blocks the removal: {blocked:?}");
    };
    assert!(detail.contains("own session"), "{detail}");
}

/// What:
///  A marked origin is a finding that names the mark.
#[test]
fn a_marked_origin_is_a_finding() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.mark = Reading::Read(OriginMark::Marked {
        clones: 1,
        holds: UserHolds::Counted(0),
    });
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    let outcome: CheckOutcome = check_clone_origin(&gathered.origin);
    let CheckOutcome::Finding {
        field,
        ..
    } = &outcome
    else {
        panic!("a marked origin is a finding: {outcome:?}");
    };
    assert_eq!(field, "defer_destroy");
}

/// What:
///  An origin mark nobody read is unknown,
///  and it is read for a record that names an origin.
#[test]
fn an_origin_mark_nobody_read_is_unknown() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.mark = Reading::Unreadable {
        why: String::from("the snapshot's object-stats reply carried no such property"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert!(reads.pool_reads.get() > 0, "the mark was asked for");
    let outcome: CheckOutcome = check_clone_origin(&gathered.origin);
    assert!(is_unknown(&outcome), "{}", line(&outcome));
}

//endregion

//region A record that names less

/// What:
///  A slot whose record reads nothing asks no pool read and no scan,
///  and every removal-side fact says why instead of naming a clone.
#[test]
fn a_slot_with_no_record_asks_no_pool_read() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    reads.readings.binding = BindingReading::Absent;
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(reads.plan_reads.get(), 0);
    assert_eq!(reads.scan_reads.get(), 0);
    assert!(
        gathered.holders.incomplete[0].contains("names no clone GUID"),
        "{}",
        gathered.holders.incomplete[0]
    );
    let snapshots: CheckOutcome = check_clone_snapshots(&gathered.snapshots);
    assert!(is_unknown(&snapshots), "{}", line(&snapshots));
}

/// What:
///  A record that names no clone GUID files no plan,
///  because a plan is filed under that GUID.
#[test]
fn a_record_with_no_clone_guid_files_no_plan() {
    let healthy: CloneFacts = healthy_clone();
    let mut reads: ScriptedReads = reads_of_the_healthy_clone(&healthy);
    let slot_path: PathBuf = healthy.state.readings.slot_path.clone();
    reads.readings.binding =
        BindingReading::Record(Box::new(record_at(Phase::Snapshotted, slot_path.as_path())));
    reads.inventory = Reading::Unreadable {
        why: String::from("the pool answered no GUID for the clone"),
    };
    let gathered: CloneFacts = gather_clone_facts(&request(&healthy), &reads);
    assert_eq!(reads.plan_reads.get(), 0);
    assert_eq!(gathered.plan.reading, PlanFileReading::NoPlan);
    let plan: CheckOutcome = check_removal_plan(&gathered.plan);
    assert!(!is_finding(&plan) && !is_unknown(&plan), "{}", line(&plan));
}

//endregion
