//! What:
//!  The guest arm of the destruction step:
//!  a healthy removal that destroys the clone's own snapshots and stops before the head,
//!  the look's refusal when the mark is already there,
//!  the first check's refusal when the mark appears after the look,
//!  the owed measurement's second half,
//!  where the mark appears after the batch and the head check refuses,
//!  and a replacement clone whose different origin is marked.
//! Why:
//!  The mark is the kernel's own field and only a real module answers it,
//!  so the arm runs inside the throwaway `mvm` guest the package's e2e recipe drives
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Measurements owed",
//! M3a to M3d,
//!  and the proposal's "The refusal of a removal whose origin is marked for deferred destroy",
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Forced removal and deferred origins").
//!  The reap itself is the deferred-origin folder's own guest arm,
//! which ran before this one:
//!  a harness that cannot see the loss cannot show that the refusal avoids it (rule `QPC`).
//!  This arm drives the destruction half the library offers,
//!  because the whole order's own arm is `removal_completion_guest_arm_tests`.
//!  The windows the design reaches with its pause route are reached here from the arm's own
//!  seams:
//!  the check's reader answers the first check from an unmarked pool and every later read from
//!  a marked one,
//!  and the device marks the origin after the batch's request,
//!  which is the same moment the pause route would hold the removal at.
//!  The unmount is a scripted answer here,
//!  because step 9's live path needs the root handler the guest fixture does not start,
//!  so the arm judges the plans and the checks and never the mount work.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the removal destruction guest arm', () => { /* zpool, checkout, clone, mark, replacement */ });
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
    ZfsDatasetReader,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::TheLiveClock;
use crate::worktree_clone_removal_proof_and_plan::removal_destruction::{
    DestructionUnderway,
    RemovalDestruction,
    destroy_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{
    HookedDevice,
    LivePoolFixture,
    PoolVariables,
    SLOT,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::LiveRemovalReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes;
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    MARK_OFF,
    MARK_ON,
    OriginMark,
    origin_mark,
};
use crate::worktree_removal_deferred_origin_check::origin_refusal::OriginRefusal;
use crate::worktree_removal_deferred_origin_check::origin_refusals::{
    RefusalForm,
    WhatARefusalLeaves,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DeviceError,
    NvList,
    ZfsDevice,
};
use std::cell::{
    Cell,
    RefCell,
};
use std::path::PathBuf;

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_DESTRUCTION_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_DESTRUCTION_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_DESTRUCTION_PREFIX";
/// The variable naming the folder the pool's image and the repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_DESTRUCTION_ROOT";

/// The guest directory the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-15-destruction-fixture";
/// The pool the arm builds when the driver names none.
const DEFAULT_POOL: &str = "zfsstep15destruction";
/// The clone's own first snapshot.
const MANUAL_SNAPSHOT: &str = "manual-2026-11-21";
/// The clone's own second snapshot.
const SANOID_SNAPSHOT: &str = "sanoid-hourly-2026-11-22";
/// The origin snapshot the replacement clone stands on,
/// which the arm makes marked.
const OTHER_ORIGIN: &str = "other";

/// The four variables and the two defaults this arm's fixture takes.
fn variables() -> PoolVariables {
    return PoolVariables {
        guest: GUEST_VARIABLE,
        pool: POOL_VARIABLE,
        prefix: PREFIX_VARIABLE,
        root: ROOT_VARIABLE,
        default_root: DEFAULT_ROOT,
        default_pool: DEFAULT_POOL,
    };
}

/// The unmount answer the guest gives:
/// the intent's own nonce and the live clone's GUID,
/// so the arm judges the plan and the checks and never the mount work.
struct ADoneUnmount {
    clone_guid: DatasetGuid,
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
///  The guest's reader,
///  with the mark set at the moment the checks first read.
/// Why:
///  The project's first check refuses the mark that appeared after the look,
///  and the look ran inside the entry,
///  so the arm sets the mark at the first read the check makes,
///  which is after the entry returned and before any of the check's own three reads of that
///  pass.
struct MarkingReader<'a> {
    live: &'a ZfsDatasetReader,
    fixture: &'a LivePoolFixture,
    origin: &'a str,
    reads: Cell<usize>,
}

impl DatasetReader for MarkingReader<'_> {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        let seen: usize = self.reads.get();
        self.reads.set(seen + 1);
        if seen == 0 {
            self.fixture.run("zfs", &["destroy", "-d", self.origin]);
        }
        return self.live.identity(dataset);
    }

    fn next_child(
        &self,
        parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return self.live.next_child(parent, cursor);
    }
}

/// What the arm's every run needs:
/// the repository's common directory,
/// the pool reads,
/// the unmount answer and the budget one request of the whole set fits in.
struct Run<'a> {
    common: PathBuf,
    reads: &'a LiveRemovalReads,
    unmount: ADoneUnmount,
    budget: RequestBudget,
}

impl<'a> Run<'a> {
    fn over(
        fixture: &LivePoolFixture,
        reads: &'a LiveRemovalReads,
    ) -> Run<'a> {
        let recorded: RecordedBinding = fixture.recorded();
        let names: Vec<String> = vec![
            format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()),
            format!("{}@{SANOID_SNAPSHOT}", fixture.clone()),
        ];
        let bytes: usize = encoded_request_bytes(fixture.pool.as_str(), names.as_slice())
            .expect("both names pack into one request");
        return Run {
            common: fixture.common(),
            reads,
            unmount: ADoneUnmount {
                clone_guid: recorded.clone_guid,
            },
            budget: RequestBudget::of_input_bytes(bytes),
        };
    }

    /// One whole removal's destruction over the checks' own reader.
    fn destroy(
        &self,
        check: &dyn DatasetReader,
        device: &dyn SnapshotDestroyDevice,
    ) -> Result<DestructionUnderway, RemovalStop> {
        let at: RemovalEntry<'_> = RemovalEntry {
            common_dir: self.common.as_path(),
            slot: SLOT,
        };
        return destroy_the_removal(&RemovalDestruction {
            entry: &at,
            reads: self.reads,
            unmount: &self.unmount,
            clock: &TheLiveClock,
            check,
            device,
            budget: self.budget,
        });
    }
}

/// The guest's own device,
/// as the arm's requests reach it.
fn the_live_device() -> ZfsDevice {
    return ZfsDevice::open().expect("the guest has /dev/zfs");
}

/// One refusal's cause,
/// or a panic naming the state the arm expected instead.
fn cause_of(stop: &RemovalStop) -> OriginRefusal {
    return match stop {
        RemovalStop::OriginCheckRefused {
            refusal,
            ..
        } => (**refusal).clone(),
        other => panic!(
            "the arm expected one of step 15's refusals, and the removal stopped with {other:?}"
        ),
    };
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_removal_destruction_guest_arms() {
    let Some(fixture) =
        LivePoolFixture::from_environment("the_removal_destruction_guest_arms", &variables())
    else {
        return;
    };
    println!("DESTRUCTION guest={} pool={}", fixture.guest, fixture.pool);
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");
    let live: ZfsDevice = the_live_device();
    println!("DESTRUCTION step=fixture-open");

    // Arm 1: the healthy removal destroys both own snapshots, records each answer, and stops before the head.
    println!("DESTRUCTION step=arm-1-build");
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let healthy_recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&healthy_recorded);
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let healthy_run: Run<'_> = Run::over(&fixture, &reads);
    println!("DESTRUCTION step=arm-1-enter");
    let underway: DestructionUnderway = healthy_run
        .destroy(&reader, &live)
        .expect("the healthy removal destroys its own snapshots");
    assert_eq!(underway.requests.len(), 1, "the whole set fits one request");
    for snapshot in underway.plan.named_snapshots() {
        let state = underway
            .plan
            .current_state(snapshot.name.as_str())
            .expect("the plan holds the name the request carried");
        assert_eq!(
            state.outcome,
            SnapshotOutcome::Destroyed,
            "every submitted name's answer is the state its history ends on"
        );
    }
    assert!(
        underway.remaining_snapshots.is_empty(),
        "the fresh enumeration finds no snapshot left"
    );
    assert!(
        !fixture.exists(format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()).as_str()),
        "the batch destroyed the clone's own snapshot"
    );
    assert!(
        !fixture.exists(format!("{}@{SANOID_SNAPSHOT}", fixture.clone()).as_str()),
        "the batch destroyed the clone's other own snapshot"
    );
    assert!(
        fixture.exists(fixture.clone().as_str()),
        "the head stands, because this release sends no head destroy"
    );
    assert!(
        fixture.exists(fixture.origin().as_str()),
        "an unmarked origin stands after the batch"
    );
    let healthy_filed: DestructionPlan =
        read_the_plan(fixture.common().as_path(), healthy_recorded.clone_guid)
            .expect("the plan reads back");
    assert_eq!(
        healthy_filed, underway.plan,
        "the file holds every answer this run received"
    );
    println!("DESTRUCTION healthy=destroyed snapshots=2 remaining=0 head=present");

    // Arm 2: the mark is already there, so the look inside the entry refuses before anything is taken down.
    println!("DESTRUCTION step=arm-2-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let look_recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&look_recorded);
    fixture.run("zfs", &["destroy", "-d", fixture.origin().as_str()]);
    assert_eq!(
        origin_mark(&reader, fixture.origin().as_str()),
        OriginMark::Marked {
            clones: 1,
            holds: crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds::Counted(
                0
            )
        },
        "the mark reads {MARK_ON} with one clone and no hold"
    );
    let look_run: Run<'_> = Run::over(&fixture, &reads);
    println!("DESTRUCTION step=arm-2-enter");
    let look_stop: RemovalStop = look_run
        .destroy(&reader, &live)
        .expect_err("a marked origin refuses at the look");
    let look_cause: OriginRefusal = cause_of(&look_stop);
    assert_eq!(
        look_cause.cause_name(),
        "origin-marked-before-the-unmount",
        "the look is what refused, and the mark was there before the run"
    );
    assert_eq!(
        look_cause.form(),
        RefusalForm::NothingTakenDown,
        "the look's message claims nothing was taken down"
    );
    assert_eq!(
        look_cause.what_it_leaves(),
        WhatARefusalLeaves::NothingPending
    );
    assert!(
        look_stop
            .message()
            .contains("nothing is pending and there is nothing to cancel"),
        "the look's sentence says nothing is pending"
    );
    assert!(
        fixture.exists(format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()).as_str()),
        "no snapshot was destroyed"
    );
    assert_eq!(
        fixture.plan_entries(),
        0,
        "the look's refusal wrote no plan"
    );
    println!(
        "DESTRUCTION look=refused cause={} pending=none plan=none",
        look_cause.cause_name()
    );

    // Arm 3: the mark appears after the look and before the first check, which refuses with nothing sent.
    println!("DESTRUCTION step=arm-3-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let first_recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&first_recorded);
    let first_run: Run<'_> = Run::over(&fixture, &reads);
    let origin: String = fixture.origin();
    let marking: MarkingReader<'_> = MarkingReader {
        live: &reader,
        fixture: &fixture,
        origin: origin.as_str(),
        reads: Cell::new(0),
    };
    let quiet_device: RecordingDevice = RecordingDevice::new();
    println!("DESTRUCTION step=arm-3-enter");
    let first_stop: RemovalStop = first_run
        .destroy(&marking, &quiet_device)
        .expect_err("the first check refuses the mark that appeared after the look");
    let first_cause: OriginRefusal = cause_of(&first_stop);
    assert_eq!(
        first_cause.cause_name(),
        "origin-marked-for-deferred-destroy",
        "the first check is what refused"
    );
    assert_eq!(first_cause.form(), RefusalForm::PendingAfterTheUnmount);
    assert_eq!(
        first_cause.what_it_leaves(),
        WhatARefusalLeaves::PendingWithItsRecord,
        "the intent and the plan are kept"
    );
    assert!(
        first_stop
            .message()
            .contains("This removal has not destroyed anything yet."),
        "nothing was sent, so the sentence says nothing was destroyed"
    );
    assert!(quiet_device.sent.borrow().is_empty(), "no request was sent");
    assert!(
        fixture.exists(format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()).as_str()),
        "the clone keeps both its own snapshots"
    );
    assert!(
        fixture.exists(fixture.origin().as_str()),
        "the origin snapshot stands"
    );
    let first_filed: DestructionPlan =
        read_the_plan(fixture.common().as_path(), first_recorded.clone_guid)
            .expect("the plan stands");
    assert!(
        first_filed
            .attempts
            .iter()
            .all(|line| return line.outcome == SnapshotOutcome::Planned),
        "the plan records no answer"
    );
    println!(
        "DESTRUCTION first-check=refused cause={} sent=0 lost=nothing",
        first_cause.cause_name()
    );

    // Arm 4: the owed measurement's second half: the mark appears after the batch and before the head check.
    println!("DESTRUCTION step=arm-4-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let head_recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&head_recorded);
    let head_run: Run<'_> = Run::over(&fixture, &reads);
    let marking_device: HookedDevice<'_> = HookedDevice {
        live: &live,
        after_the_first: Box::new(|| {
            fixture.run("zfs", &["destroy", "-d", fixture.origin().as_str()]);
        }),
        calls: Cell::new(0),
    };
    println!("DESTRUCTION step=arm-4-enter");
    let head_stop: RemovalStop = head_run
        .destroy(&reader, &marking_device)
        .expect_err("the head check refuses the mark the batch's own request made");
    let head_cause: OriginRefusal = cause_of(&head_stop);
    assert_eq!(
        head_cause.cause_name(),
        "origin-marked-for-deferred-destroy",
        "the head check is what refused"
    );
    assert_eq!(head_cause.form(), RefusalForm::PendingAfterTheUnmount);
    let head_message: String = head_stop.message();
    assert!(
        head_message.contains("has already lost"),
        "the head check's sentence names what the batch destroyed: {head_message}"
    );
    assert!(
        !head_message.contains("has not destroyed anything yet"),
        "the sentence never claims nothing was destroyed: {head_message}"
    );
    assert!(
        fixture.exists(fixture.origin().as_str()),
        "the origin snapshot stands"
    );
    assert!(
        fixture.exists(fixture.clone().as_str()),
        "the clone head stands, so no head request was sent"
    );
    assert!(
        !fixture.exists(format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()).as_str()),
        "the batch destroyed the first own snapshot"
    );
    assert!(
        !fixture.exists(format!("{}@{SANOID_SNAPSHOT}", fixture.clone()).as_str()),
        "the batch destroyed the second own snapshot"
    );
    assert!(
        fixture.registration().exists(),
        "the Git registration survives the refusal"
    );
    let head_filed: DestructionPlan =
        read_the_plan(fixture.common().as_path(), head_recorded.clone_guid)
            .expect("the plan stands");
    for snapshot in head_filed.named_snapshots() {
        let state = head_filed
            .current_state(snapshot.name.as_str())
            .expect("the plan holds every name the batch submitted");
        assert_eq!(
            state.outcome,
            SnapshotOutcome::Destroyed,
            "the plan keeps every name the batch destroyed"
        );
    }
    assert_eq!(
        fixture.plan_entries(),
        1,
        "the refusal kept the plan, which is the record of what the batch destroyed"
    );
    println!(
        "DESTRUCTION head-check=refused cause={} plan=kept registration=kept",
        head_cause.cause_name()
    );

    // Arm 5: a replacement clone whose different origin is marked, in place before the run.
    println!("DESTRUCTION step=arm-5-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let replacement_recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&replacement_recorded);
    let replacement_run: Run<'_> = Run::over(&fixture, &reads);
    fixture.run(
        "zfs",
        &[
            "destroy",
            format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()).as_str(),
        ],
    );
    fixture.run(
        "zfs",
        &[
            "destroy",
            format!("{}@{SANOID_SNAPSHOT}", fixture.clone()).as_str(),
        ],
    );
    fixture.run("zfs", &["destroy", fixture.clone().as_str()]);
    let checkout: String = fixture.checkout();
    fixture.run(
        "zfs",
        &["snapshot", format!("{checkout}@{OTHER_ORIGIN}").as_str()],
    );
    fixture.run(
        "zfs",
        &[
            "clone",
            format!("{checkout}@{OTHER_ORIGIN}").as_str(),
            fixture.clone().as_str(),
        ],
    );
    fixture.run(
        "zfs",
        &[
            "destroy",
            "-d",
            format!("{checkout}@{OTHER_ORIGIN}").as_str(),
        ],
    );
    let replacement_device: RecordingDevice = RecordingDevice::new();
    println!("DESTRUCTION step=arm-5-enter");
    let replacement_stop: RemovalStop = replacement_run
        .destroy(&reader, &replacement_device)
        .expect_err("a replacement clone is not the planned one");
    let replacement_message: String = replacement_stop.message();
    match &replacement_stop {
        RemovalStop::IdentityDiffers {
            field,
            ..
        } => assert_eq!(
            *field, "cloneGuid",
            "the entry's own proof refuses the replacement's GUID before any mark is judged: {replacement_message}"
        ),
        other => panic!(
            "the entry's proof refuses a replacement clone, and this run stopped with {other:?}"
        ),
    }
    assert!(
        replacement_device.sent.borrow().is_empty(),
        "no request was sent"
    );
    assert!(
        fixture.exists(fixture.clone().as_str()),
        "the replacement clone stands"
    );
    assert!(
        fixture.exists(format!("{checkout}@{OTHER_ORIGIN}").as_str()),
        "the marked origin the replacement stands on stands"
    );
    assert_eq!(fixture.plan_entries(), 0, "the refusal wrote no plan");
    println!("DESTRUCTION replacement=refused field=cloneGuid sent=0");

    fixture.teardown();
    println!("DESTRUCTION done marks={MARK_OFF}/{MARK_ON}");
}

/// What:
///  A device that records every request and answers success,
///  so an arm that expects a refusal can show that none was sent.
/// Why:
///  The arm's own claims about "no request" are claims about this seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const device = { sent: [], destroySnaps: () => ok(none) };
/// ```
struct RecordingDevice {
    sent: RefCell<Vec<Vec<String>>>,
}

impl RecordingDevice {
    fn new() -> RecordingDevice {
        return RecordingDevice {
            sent: RefCell::new(Vec::new()),
        };
    }
}

impl SnapshotDestroyDevice for RecordingDevice {
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
        return Ok(None);
    }
}
