//! What:
//!  The guest arm of the whole removal:
//!  a mounted clone's refusal,
//!  the healthy removal that destroys the head and the origin snapshot and settles the Git
//!  registration,
//!  a settled registration whose origin cleanup is blocked,
//!  and a resume that finishes a settle the first run refused.
//! Why:
//!  The step's acceptance items are measurements over a real pool:
//!  the batch and the head destroy,
//!  the origin cleanup under the account's own authority,
//!  the loss report,
//!  the resume,
//!  and the Git settle
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16's "VM measurements";
//!  the proposal's "Binding,
//! Git and lifecycle" fourth and sixth bullets).
//!  The mounted arm is the wrapper's own refusal to destroy a clone a namespace still holds,
//!  and the scan is the release's live one,
//!  so the arm measures the refusal and not a scripted value.
//!  The unmount stays a scripted answer,
//!  because a live unmount needs the root handler the guest fixture does not start,
//!  and this arm judges the destruction,
//!  the settle and the resume.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the removal completion guest arms', () => { /* pool, mount, head, origin, settle, resume */ });
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::an_existing_cleanup_record;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::TheLiveHolderScan;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::TheLiveClock;
use crate::worktree_clone_removal_proof_and_plan::removal_completion::complete_the_removal;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{
    LivePoolFixture,
    PoolVariables,
    SLOT,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::LiveRemovalReads;
use crate::worktree_clone_removal_proof_and_plan::removal_report::TheRemovalDone;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{
    RegistrationSettle,
    SettleAnswer,
    SettleRequest,
    TheLiveSettle,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::worktree_removal_deferred_origin_check::origin_mark::MARK_OFF;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    NvList,
    ZfsDevice,
};
use std::cell::{
    Cell,
    RefCell,
};

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_COMPLETION_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_COMPLETION_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_COMPLETION_PREFIX";
/// The variable naming the folder the pool's image and the repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_COMPLETION_ROOT";

/// The guest directory the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-16-completion-fixture";
/// The pool the arm builds when the driver names none.
const DEFAULT_POOL: &str = "zfsstep16completion";
/// The clone's own first snapshot.
const MANUAL_SNAPSHOT: &str = "manual-2026-11-21";
/// The clone's own second snapshot.
const SANOID_SNAPSHOT: &str = "sanoid-hourly-2026-11-22";
/// Where the mounted arm mounts the clone.
const MOUNT_POINT: &str = "/home/zfs-step-16-mount-point";
/// The add's own origin snapshot name,
/// which begins with the prefix and the nonce the cleanup record's grammar requires.
const ADD_ORIGIN: &str = "git-worktree-clone-origin-2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f";

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

/// The unmount answer the guest gives,
/// so the arm judges the destruction,
/// the settle and the resume and never the mount work.
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
///  The guest's own snapshot device,
///  with every request recorded.
/// Why:
///  The resume's claim is that the second run sends no request,
///  which is only worth something from a seam that could have shown one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const device = { sent: [], destroySnaps: (pool, names) => live.destroySnaps(pool, names) };
/// ```
struct RecordingDevice<'a> {
    live: &'a ZfsDevice,
    sent: RefCell<Vec<Vec<String>>>,
}

impl SnapshotDestroyDevice for RecordingDevice<'_> {
    fn destroy_snaps(
        &self,
        pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        self.sent.borrow_mut().push(
            names
                .iter()
                .map(|name| return String::from(*name))
                .collect(),
        );
        return self.live.destroy_snaps(pool, names);
    }
}

/// What:
///  The settle a resume arm answers:
///  every call up to a named count refuses,
///  and the later ones are the release's own.
/// Why:
///  A kill between the head and the settle is the state a resume finishes,
///  and the arm reaches it without a signal by refusing the first call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = { refusals: 1, settle: (request) => refusals-- > 0 ? refuse() : live.settle(request) };
/// ```
struct RefusingOnce<'a> {
    refusals: Cell<usize>,
    live: &'a TheLiveSettle,
}

impl RegistrationSettle for RefusingOnce<'_> {
    fn settle_the_registration(
        &self,
        request: &SettleRequest<'_>,
    ) -> Result<SettleAnswer, RemovalStop> {
        let left: usize = self.refusals.get();
        if left > 0 {
            self.refusals.set(left - 1);
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("a control's own refusal"),
                said: String::from("this arm refuses the first settle"),
            });
        }
        return self.live.settle_the_registration(request);
    }
}

/// The budget every arm's whole first write fits in,
/// measured from the exact request it becomes.
fn the_budget(fixture: &LivePoolFixture) -> RequestBudget {
    let names: Vec<String> = vec![
        format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()),
        format!("{}@{SANOID_SNAPSHOT}", fixture.clone()),
    ];
    let bytes: usize =
        crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes(
            fixture.pool.as_str(),
            names.as_slice(),
        )
        .expect("both names pack into one request");
    return RequestBudget::of_input_bytes(bytes);
}

/// What:
///  One whole removal over the guest's own pool,
///  with the clone's recorded GUID and a settle the arm names.
/// Why:
///  A resumed run has no live clone to read the GUID from,
///  and the plan and the binding carry it,
///  so the arm passes the GUID it recorded before the clone went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = complete(fixture, cloneGuid, reads, reader, device, settle);
/// ```
fn complete(
    fixture: &LivePoolFixture,
    clone_guid: DatasetGuid,
    reads: &LiveRemovalReads,
    reader: &ZfsDatasetReader,
    device: &RecordingDevice<'_>,
    settle: &dyn RegistrationSettle,
) -> Result<TheRemovalDone, RemovalStop> {
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid,
    };
    let common: std::path::PathBuf = fixture.common();
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: common.as_path(),
        slot: SLOT,
    };
    let input: RemovalCompletion<'_> = RemovalCompletion {
        entry: &at,
        reads,
        unmount: &unmount,
        clock: &TheLiveClock,
        check: reader,
        probe: reader,
        snapshots: device,
        head: device.live,
        holders: &TheLiveHolderScan,
        settle,
        budget: the_budget(fixture),
        now: "2026-11-25T09:12:44-05:00",
    };
    return complete_the_removal(&input);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_removal_completion_guest_arms() {
    let Some(fixture) =
        LivePoolFixture::from_environment("the_removal_completion_guest_arms", &variables())
            .map(|fixture| return fixture.with_origin_name(ADD_ORIGIN))
    else {
        return;
    };
    println!("COMPLETION guest={} pool={}", fixture.guest, fixture.pool);
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let live: ZfsDevice = ZfsDevice::open().expect("the guest has /dev/zfs");
    let settle: TheLiveSettle = TheLiveSettle;

    // Arm 1: a clone a namespace holds refuses before any request, which is the wrapper's own refusal.
    println!("COMPLETION step=arm-1-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let recorded = fixture.recorded();
    fixture.write_the_adds_records(&recorded);
    std::fs::create_dir_all(MOUNT_POINT).expect("the mount point");
    fixture.run(
        "mount",
        &["-t", "zfs", fixture.clone().as_str(), MOUNT_POINT],
    );
    let mounted_device: RecordingDevice<'_> = RecordingDevice {
        live: &live,
        sent: RefCell::new(Vec::new()),
    };
    println!("COMPLETION step=arm-1-enter");
    let mounted: Result<TheRemovalDone, RemovalStop> = complete(
        &fixture,
        recorded.clone_guid,
        &reads,
        &reader,
        &mounted_device,
        &settle,
    );
    let stop: RemovalStop = match mounted {
        Ok(_) => panic!("a mounted clone must not be destroyed"),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(stop, RemovalStop::HoldersSeeTheClone { .. }),
        "the fresh scan sees the mount, and this run stopped with {stop:?}"
    );
    assert!(
        mounted_device.sent.borrow().is_empty(),
        "no snapshot request is sent while a namespace holds the clone"
    );
    assert!(
        fixture.exists(format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()).as_str()),
        "the clone keeps its own snapshot"
    );
    assert!(
        fixture.exists(fixture.clone().as_str()),
        "the clone head stands"
    );
    println!("COMPLETION mounted=refused sent=0 snapshot=kept");
    fixture.run("umount", &[MOUNT_POINT]);

    // Arm 2: the healthy removal destroys both own snapshots, the head and the origin snapshot, and settles the registration.
    println!("COMPLETION step=arm-2-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let healthy_recorded = fixture.recorded();
    fixture.write_the_adds_records(&healthy_recorded);
    let device: RecordingDevice<'_> = RecordingDevice {
        live: &live,
        sent: RefCell::new(Vec::new()),
    };
    println!("COMPLETION step=arm-2-enter");
    let done: TheRemovalDone = complete(
        &fixture,
        healthy_recorded.clone_guid,
        &reads,
        &reader,
        &device,
        &settle,
    )
    .expect("a healthy removal finishes");
    assert_eq!(
        device.sent.borrow().len(),
        2,
        "one request for the clone's snapshots and one for the origin snapshot"
    );
    for snapshot in [MANUAL_SNAPSHOT, SANOID_SNAPSHOT] {
        assert!(
            !fixture.exists(format!("{}@{snapshot}", fixture.clone()).as_str()),
            "the batch destroyed `{snapshot}`"
        );
    }
    assert!(
        !fixture.exists(fixture.clone().as_str()),
        "the head destroy proved the clone gone"
    );
    assert!(
        !fixture.exists(fixture.origin().as_str()),
        "the origin cleanup destroyed the checkout's origin snapshot"
    );
    assert!(
        !fixture.registration().exists(),
        "the Git registration went"
    );
    assert!(!fixture.slot_path().exists(), "the underlay folder went");
    assert!(
        !fixture
            .common()
            .join("monochromatic-git-policy-cli")
            .join("clone-destruction-plans")
            .exists(),
        "the plan and its folder went"
    );
    assert!(
        an_existing_cleanup_record(fixture.common().as_path(), ADD_ORIGIN)
            .expect("the folder answers")
            .is_none(),
        "no cleanup record stands after a finished cleanup"
    );
    assert_eq!(
        done.head,
        crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord::Gone
    );
    assert_eq!(
        done.destroyed.len(),
        2,
        "the report names both of the clone's own snapshots"
    );
    println!("COMPLETION healthy=destroyed snapshots=2 head=gone origin=gone registration=gone");

    // Arm 3: another clone pins the origin snapshot, so the registration is settled and the cleanup record stands blocked.
    println!("COMPLETION step=arm-3-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let pinned_recorded = fixture.recorded();
    fixture.write_the_adds_records(&pinned_recorded);
    let other: String = fixture.named("clones-other");
    fixture.run("zfs", &["clone", fixture.origin().as_str(), other.as_str()]);
    let pinned_device: RecordingDevice<'_> = RecordingDevice {
        live: &live,
        sent: RefCell::new(Vec::new()),
    };
    println!("COMPLETION step=arm-3-enter");
    let pinned: TheRemovalDone = complete(
        &fixture,
        pinned_recorded.clone_guid,
        &reads,
        &reader,
        &pinned_device,
        &settle,
    )
    .expect("the removal finishes whatever the origin does");
    assert!(
        !fixture.exists(fixture.clone().as_str()),
        "the clone is gone"
    );
    assert!(
        fixture.exists(fixture.origin().as_str()),
        "the origin snapshot stands, pinned by the other clone"
    );
    assert!(
        !fixture.registration().exists(),
        "the registration is settled even while the origin cleanup is blocked"
    );
    assert!(
        fixture.exists(fixture.origin().as_str()),
        "the pinned origin snapshot stands"
    );
    assert!(matches!(pinned.origin, crate::worktree_clone_removal_proof_and_plan::removal_report::OriginCleanupDone::Blocked { .. }), "the cleanup recorded its block");
    println!("COMPLETION blocked=origin registration=settled plan=removed record=blocked");
    fixture.run("zfs", &["destroy", other.as_str()]);

    // Arm 4: the first run's settle refuses, and the next run finishes it without sending a request.
    println!("COMPLETION step=arm-4-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let resume_recorded = fixture.recorded();
    fixture.write_the_adds_records(&resume_recorded);
    let first_device: RecordingDevice<'_> = RecordingDevice {
        live: &live,
        sent: RefCell::new(Vec::new()),
    };
    let refusing: RefusingOnce<'_> = RefusingOnce {
        refusals: Cell::new(1),
        live: &settle,
    };
    println!("COMPLETION step=arm-4-first-enter");
    let first: Result<TheRemovalDone, RemovalStop> = complete(
        &fixture,
        resume_recorded.clone_guid,
        &reads,
        &reader,
        &first_device,
        &refusing,
    );
    let first_stop: RemovalStop = match first {
        Ok(_) => panic!("the arm's settle refuses the first run"),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(first_stop, RemovalStop::TheSettleRefused { .. }),
        "the settle's refusal stops the run"
    );
    let kept = read_an_existing_plan(fixture.common().as_path(), resume_recorded.clone_guid)
        .expect("the plan reads")
        .expect("the plan stands after the refusal");
    assert_eq!(
        kept.head,
        crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord::Gone,
        "the head's own proof is kept"
    );
    assert!(
        fixture.registration().exists(),
        "the registration stands for the next run"
    );
    let scnd: RecordingDevice<'_> = RecordingDevice {
        live: &live,
        sent: RefCell::new(Vec::new()),
    };
    println!("COMPLETION step=arm-4-second-enter");
    let resumed: TheRemovalDone = complete(
        &fixture,
        resume_recorded.clone_guid,
        &reads,
        &reader,
        &scnd,
        &settle,
    )
    .expect("the second run finishes the settle");
    assert!(
        scnd.sent.borrow().is_empty(),
        "the head-gone path sends no snapshot request"
    );
    assert!(
        !fixture.registration().exists(),
        "the second run settled the registration"
    );
    assert!(
        resumed.records.plan_removed,
        "the second run removed the plan"
    );
    println!("COMPLETION resume=finished requests=0 registration=settled");

    fixture.teardown();
    println!("COMPLETION done marks={MARK_OFF}");
}
