//! What:
//!  The guest arms of the cancel command:
//!  a real pool whose clone lost one snapshot,
//!  reversed by the command's own proof,
//!  and a removal already cancelled whose plan leaves.
//! Why:
//!  The step's acceptance items are measurements over a real pool,
//!  and the command reads the clone's identity,
//!  its own request and its inventory there
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 17's "What it builds" and "Acceptance tests").
//!  The mount request and the staging settlement are scripted answers,
//!  because a live handler is not started in this fixture,
//!  and what the arm measures is the proof,
//!  the destroyed set the pool answers,
//!  and the plan's fate.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the cancel command guest arms', () => { /* pool, destroy, prove, plan */ });
//! ```

use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_intent::intent_text;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_cancel::cancel_arguments::CancelRequest;
use crate::worktree_clone_removal_cancel::the_cancel_command::{
    TheCancelCommand,
    TheCancellation,
    cancel_the_removal,
};
use crate::worktree_clone_removal_cancel::the_removal_claim::{
    HeldClaim,
    TheRemovalClaim,
};
use crate::worktree_clone_removal_cancel::the_reversing_request::{
    MountAnswer,
    MountIdentity,
    TheMountRequest,
};
use crate::worktree_clone_removal_cancel::the_slot_mount::TheSlotMount;
use crate::worktree_clone_removal_cancel::the_staged_state::{
    StagedSettlement,
    TheStagedFolder,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::TheLiveClock;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::AStagingFolderAControlAnswers;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    RemovalEntry,
    enter_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{
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
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use std::cell::RefCell;
use std::path::{
    Path,
    PathBuf,
};

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_CANCEL_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_CANCEL_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_CANCEL_PREFIX";
/// The variable naming the folder the pool's image and the repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_CANCEL_ROOT";

/// The guest directory the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-17-cancel-fixture";
/// The pool the arm builds when the driver names none.
const DEFAULT_POOL: &str = "zfsstep17cancel";
/// The clone's first own snapshot.
const FIRST_SNAPSHOT: &str = "sanoid-hourly-2026-11-22_03";
/// The clone's second own snapshot.
const SECOND_SNAPSHOT: &str = "sanoid-hourly-2026-11-22_04";
/// The nonce the clone's own request carries.
const THE_NONCE: &str = "3f2a0000000000000000000003f2a000";

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

/// A nonce of the shape the request needs.
fn a_nonce() -> RequestNonce {
    return RequestNonce::parse(THE_NONCE).expect("a nonce");
}

/// What:
///  The request writer a guest arm answers with,
///  which records the intent it was asked to file.
/// Why:
///  A live handler is not started in this fixture,
///  and the arm's own assertion is that the request the command wrote named the
///  clone and the `mount` operation.
struct RecordingRequest {
    /// The clone's dataset,
    /// which the answer names as its mount's source.
    clone: String,
    /// The clone's slot.
    slot_path: PathBuf,
    /// The clone's GUID.
    clone_guid: DatasetGuid,
    /// The intents it was asked to file.
    asked: RefCell<Vec<String>>,
}

impl TheMountRequest for RecordingRequest {
    fn request_the_mount(
        &self,
        intent: &MountIntent,
    ) -> Result<MountAnswer, crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal>
    {
        self.asked
            .borrow_mut()
            .push(intent_text(intent.operation, &intent.nonce));
        return Ok(MountAnswer {
            dataset: self.clone_guid,
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            outcome: ReceiptOutcome::Done,
            mount: Some(MountIdentity {
                mount_id: 1,
                source: self.clone.clone(),
                canonical_path: self.slot_path.display().to_string(),
            }),
            failed_step: None,
            errno: None,
        });
    }
}

/// The staging folder a guest arm answers with,
/// which nothing stands for in this fixture.
struct NoStagedFolder;

impl TheStagedFolder for NoStagedFolder {
    fn settle(
        &self,
        _clone_guid: DatasetGuid,
    ) -> StagedSettlement {
        return StagedSettlement::NoStagedFolder;
    }
}

/// The claim a guest arm holds,
/// which creates no file because the arm's own run is the only operation.
struct AClaimTheArmHolds;

impl TheRemovalClaim for AClaimTheArmHolds {
    fn take(
        &self,
        _slot: &str,
    ) -> Result<HeldClaim, crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal>
    {
        return Ok(HeldClaim::AControl);
    }
}

/// The mount table a guest arm answers with,
/// which is the state the scripted answer put the clone in.
struct ATable {
    /// Whether the clone is read as standing at its slot.
    at_the_slot: bool,
}

impl TheSlotMount for ATable {
    fn where_the_clone_stands(
        &self,
        _clone_dataset: &str,
        _slot_path: &Path,
    ) -> CloneMount {
        if self.at_the_slot {
            return CloneMount::AtItsSlot;
        }
        return CloneMount::Nowhere;
    }
}

/// What:
///  The unmount answer the arm gives the removal's entry,
///  so the arm judges the command and never the mount work.
/// Why:
///  The fixture starts no handler,
///  and an answer this seam gives is the state the removal records for itself.
struct ADoneUnmount {
    /// The clone's GUID.
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

/// A process id no process can have,
/// because `/proc/sys/kernel/pid_max` is at most 4,194,304 on this kernel.
const A_PID_NO_PROCESS_CAN_HAVE: i64 = 9_000_001;

/// What:
///  The plan a removal that was killed left:
///  the entry's own record,
///  with a writer that no longer runs.
/// Why:
///  The arm runs the removal's entry in the test's own process,
///  and a plan whose owner is alive is a running removal,
///  which the command refuses by design.
///  The killed state is the same record with a dead owner,
///  and a pid above `pid_max` cannot be alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlanADeadRemovalLeft(common, recorded);
/// ```
fn the_plan_a_dead_removal_left(
    common: &Path,
    recorded: &crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding,
) {
    let mut plan: DestructionPlan =
        crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan(
            common,
            recorded.clone_guid,
        )
        .expect("the entry's plan reads")
        .expect("the entry wrote a plan");
    plan.facts.owner = format!("{A_PID_NO_PROCESS_CAN_HAVE}:linux:0");
    let folder: PathBuf = plans_folder(common);
    std::fs::write(
        folder.join(recorded.clone_guid.file_name()),
        plan.encode().expect("the plan encodes"),
    )
    .expect("the plan file");
}

/// The slot's `.git` pointer,
/// which the proof reads and which the fixture's add would have written.
fn write_the_pointer(fixture: &LivePoolFixture) {
    let registration: PathBuf = fixture.common().join("worktrees").join(SLOT);
    std::fs::write(
        fixture.slot_path().join(".git"),
        format!("gitdir: {}\n", registration.display()),
    )
    .expect("the slot's pointer");
}

/// What:
///  One whole guest arm over the fixture.
/// Why:
///  The removal's own entry writes the records the command proves,
///  so the arm measures the command and never a record this arm built by hand.
fn the_arm(
    fixture: &LivePoolFixture,
    already_cancelled: bool,
) {
    fixture.build(&[FIRST_SNAPSHOT, SECOND_SNAPSHOT]);
    let recorded = fixture.recorded();
    fixture.write_the_adds_records(&recorded);
    write_the_pointer(fixture);
    let clone: String = fixture.clone();
    let first: String = format!("{clone}@{FIRST_SNAPSHOT}");
    let common: PathBuf = fixture.common();
    let entry: RemovalEntry<'_> = RemovalEntry {
        common_dir: common.as_path(),
        slot: SLOT,
    };
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    enter_the_removal(
        &entry,
        &reads,
        &unmount,
        &TheLiveClock,
        &AStagingFolderAControlAnswers::making_nothing(),
    )
    .expect("the removal's own entry writes its plan");
    the_plan_a_dead_removal_left(common.as_path(), &recorded);
    let plan_path: PathBuf = plans_folder(common.as_path()).join(recorded.clone_guid.file_name());
    assert!(
        plan_path.exists(),
        "positive control: the removal's entry wrote a plan the command can prove"
    );
    // The request the clone carries is the state a killed removal leaves:
    // an `unmount` request, or the `mount` one a cancellation's first act writes.
    let operation: MountOperation = if already_cancelled {
        MountOperation::Mount
    } else {
        MountOperation::Unmount
    };
    let intent: MountIntent = MountIntent {
        operation,
        nonce: a_nonce(),
    };
    fixture.run(
        "zfs",
        &[
            "set",
            format!(
                "{DESIRED_STATE_PROPERTY}={}",
                intent_text(intent.operation, &intent.nonce)
            )
            .as_str(),
            clone.as_str(),
        ],
    );
    if !already_cancelled {
        // The removal destroyed one snapshot before a holder outlasted it, which the plan still calls planned.
        fixture.run("zfs", &["destroy", first.as_str()]);
    }
    let absence: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");
    let checkout: PathBuf = common
        .parent()
        .expect("the common directory is inside the fixture's checkout")
        .to_path_buf();
    let request: RecordingRequest = RecordingRequest {
        clone: clone.clone(),
        slot_path: fixture.slot_path(),
        clone_guid: recorded.clone_guid,
        asked: RefCell::new(Vec::new()),
    };
    let table: ATable = ATable {
        at_the_slot: true,
    };
    let slot: CancelRequest = CancelRequest {
        slot: String::from(SLOT),
    };
    let input: TheCancelCommand<'_> = TheCancelCommand {
        checkout: checkout.as_path(),
        common_dir: common.as_path(),
        home: fixture.root.as_path(),
        request: &slot,
        reads: &reads,
        absence: &absence,
        mounts: &table,
        claim: &AClaimTheArmHolds,
        mount_request: &request,
        staged: &NoStagedFolder,
    };
    let outcome: TheCancellation =
        cancel_the_removal(&input).expect("the command proves and reverses the removal");
    let report = outcome.report();
    assert_eq!(report.exit_code(), 0);
    if already_cancelled {
        assert!(
            report.already_cancelled,
            "a `mount` request beside a clone at its slot is an already-cancelled removal"
        );
        assert!(
            request.asked.borrow().is_empty(),
            "no second request is written"
        );
        assert!(
            !plan_path.exists(),
            "the plan that witnessed nothing leaves"
        );
        println!("CANCEL already-cancelled=reported plan=removed requests=0");
    } else {
        assert_eq!(
            report.destroyed,
            vec![first.clone()],
            "the pool proves exactly the snapshot the arm destroyed"
        );
        assert!(
            !report.already_cancelled,
            "an `unmount` request is not an already-cancelled removal"
        );
        assert_eq!(
            request.asked.borrow().len(),
            1,
            "one mount request is written"
        );
        assert!(
            request.asked.borrow()[0].ends_with(":mount"),
            "{:?}",
            request.asked.borrow()
        );
        assert!(plan_path.exists(), "the plan records a loss and stays");
        let text: String = report.lines().join("\n");
        assert!(text.contains("do not come back"), "{text}");
        assert!(text.contains(first.as_str()), "{text}");
        println!("CANCEL destroyed={first} plan=kept mounted=at-its-slot");
    }
    fixture.teardown();
}

#[test]
fn the_cancel_command_guest_arms() {
    let Some(fixture): Option<LivePoolFixture> =
        LivePoolFixture::from_environment("cancel", &variables())
    else {
        println!(
            "SKIPPED: {GUEST_VARIABLE} is not set, so this arm runs only in the throwaway guest"
        );
        return;
    };
    the_arm(&fixture, false);
    the_arm(&fixture, true);
    println!("CANCEL done");
}
