//! What:
//!  The finish pass's guest arm:
//!  a real clone,
//!  a real batch and head destroy,
//!  a settle that refuses so the plan keeps `head=gone` beside a standing registration,
//!  and then the pass finishing that tail on the live pool.
//! Why:
//!  The design's guest arm is the pass after a real head destroy:
//!  the registration settles,
//!  the two inventories and then the binding are removed and proved gone,
//!  the plan is removed,
//!  and the claim is released last
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Guest arms").
//!  The gate is the arm's own answer,
//!  because a refused gate is the host arm's subject and this arm judges the settle,
//!  the records and the claim;
//!  every pool read,
//! every destroy and every Git write here is the release's own.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = finishTheRemovalPlans({ commonDir, folder, gate, probe, settle });
//! ```

use crate::worktree_clone_binding_record::registration_home::{binding_name, clone_inventory_name, claim_prefix, home_folder, origin_inventory_name};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_finish_pass::the_pass_seams::FinishPass;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::TheLivePlansFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_completion::{RemovalCompletion, complete_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{RemovalEntry, TheLiveClock};
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{LivePoolFixture, PoolVariables, SLOT};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::LiveRemovalReads;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{RegistrationSettle, SettleAnswer, SettleRequest, TheLiveSettle};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{RemovalUnmount, UnmountAnswer};
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::TheLiveHolderScan;
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::zfs_compatibility_gate::{BeforeRequests, Facts, Verdict};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DeviceError, NvList, ZfsDevice};
use std::path::PathBuf;

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_FINISH_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_FINISH_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_FINISH_PREFIX";
/// The variable naming the folder the pool's image and the repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_FINISH_ROOT";

/// The guest directory the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-16-finish-fixture";
/// The pool the arm builds when the driver names none.
const DEFAULT_POOL: &str = "zfsstep16finish";
/// The clone's own first snapshot.
const MANUAL_SNAPSHOT: &str = "manual-2026-11-21";
/// The clone's own second snapshot.
const SANOID_SNAPSHOT: &str = "sanoid-hourly-2026-11-22";
/// The add's own origin snapshot name,
/// which begins with the prefix and the nonce the cleanup record's grammar requires.
const ADD_ORIGIN: &str = "git-worktree-clone-origin-2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f";

/// A gate that passed,
/// so this arm judges the settle and not the kernel's declaration.
struct APassedGate;

impl BeforeRequests for APassedGate {
    fn verdict(&self) -> Verdict {
        return Verdict::Compatible(Facts { target: "x86_64-unknown-linux-gnu", version: None, absent: Vec::new(), base_btf_bytes: 0, module_btf_bytes: 0, header_bytes: (0, 0) });
    }
}

/// The unmount answer the guest gives,
/// so the arm judges the destruction and never the mount work.
struct ADoneUnmount {
    clone_guid: DatasetGuid,
}

impl RemovalUnmount for ADoneUnmount {
    fn request_the_unmount(&self, intent: &MountIntent) -> Result<UnmountAnswer, RemovalStop> {
        return Ok(UnmountAnswer { dataset: self.clone_guid, operation: intent.operation, nonce: intent.nonce.clone(), outcome: ReceiptOutcome::Done, mount_id: None, failed_step: None, errno: None });
    }
}

/// A settle that refuses once,
/// so this arm reaches the state a kill between the head destroy and the settle leaves
/// without a signal.
struct ARefusingSettle;

impl RegistrationSettle for ARefusingSettle {
    fn settle_the_registration(&self, _request: &SettleRequest<'_>) -> Result<SettleAnswer, RemovalStop> {
        return Err(RemovalStop::TheSettleRefused { step: String::from("a control's own refusal"), said: String::from("this arm leaves the tail for the finish pass") });
    }
}

/// The device the guest's own snapshot requests go to,
/// recording every name it sent.
struct RecordingDevice<'a> {
    live: &'a ZfsDevice,
    sent: std::cell::RefCell<Vec<Vec<String>>>,
}

impl SnapshotDestroyDevice for RecordingDevice<'_> {
    fn destroy_snaps(&self, pool: &str, names: &[&str]) -> Result<Option<NvList>, DeviceError> {
        self.sent.borrow_mut().push(names.iter().map(|name| return String::from(*name)).collect());
        return self.live.destroy_snaps(pool, names);
    }
}

/// The four variables and the two defaults this arm's fixture takes.
fn variables() -> PoolVariables {
    return PoolVariables { guest: GUEST_VARIABLE, pool: POOL_VARIABLE, prefix: PREFIX_VARIABLE, root: ROOT_VARIABLE, default_root: DEFAULT_ROOT, default_pool: DEFAULT_POOL };
}

/// The budget both of the clone's own snapshots fit in,
/// measured from the exact request they become.
fn the_budget(fixture: &LivePoolFixture) -> RequestBudget {
    let names: Vec<String> = vec![format!("{}@{MANUAL_SNAPSHOT}", fixture.clone()), format!("{}@{SANOID_SNAPSHOT}", fixture.clone())];
    let bytes: usize = crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes(fixture.pool.as_str(), names.as_slice()).expect("both names pack into one request");
    return RequestBudget::of_input_bytes(bytes);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_removal_finish_pass_guest_arms() {
    let Some(fixture) = LivePoolFixture::from_environment("the_removal_finish_pass_guest_arms", &variables()).map(|fixture| return fixture.with_origin_name(ADD_ORIGIN)) else {
        return;
    };
    println!("FINISH guest={} pool={}", fixture.guest, fixture.pool);
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let live: ZfsDevice = ZfsDevice::open().expect("the guest has /dev/zfs");

    // Arm 1: the removal destroys the head and the origin snapshot and stops at a settle that
    // refuses, which leaves the plan with `head=gone` beside a standing registration and binding.
    println!("FINISH step=arm-1-build");
    fixture.teardown();
    fixture.build(&[MANUAL_SNAPSHOT, SANOID_SNAPSHOT]);
    let recorded = fixture.recorded();
    fixture.write_the_adds_records(&recorded);
    let device: RecordingDevice<'_> = RecordingDevice { live: &live, sent: std::cell::RefCell::new(Vec::new()) };
    let unmount: ADoneUnmount = ADoneUnmount { clone_guid: recorded.clone_guid };
    let common: PathBuf = fixture.common();
    let at: RemovalEntry<'_> = RemovalEntry { common_dir: common.as_path(), slot: SLOT };
    let completion: RemovalCompletion<'_> = RemovalCompletion {
        entry: &at,
        reads: &reads,
        unmount: &unmount,
        clock: &TheLiveClock,
        check: &reader,
        probe: &reader,
        snapshots: &device,
        head: &live,
        holders: &TheLiveHolderScan,
        settle: &ARefusingSettle,
        budget: the_budget(&fixture),
        now: "2026-11-25T09:12:44-05:00",
    };
    println!("FINISH step=arm-1-enter");
    let stopped: Result<_, RemovalStop> = complete_the_removal(&completion);
    assert!(matches!(stopped, Err(RemovalStop::TheSettleRefused { .. })), "the arm's settle refuses after the head destroy");
    assert!(!fixture.exists(fixture.clone().as_str()), "the head destroy proved the clone gone");
    let kept = crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan(common.as_path(), recorded.clone_guid).expect("the plan reads").expect("the plan stands");
    assert_eq!(kept.head, crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord::Gone, "the plan keeps the head's own proof");
    assert!(fixture.registration().exists(), "the registration stands for the finish pass");
    let home: PathBuf = home_folder(common.as_path());
    for name in [origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] {
        assert!(home.join(name.as_str()).exists(), "`{name}` stands when the pass starts");
    }
    println!("FINISH left=head-gone registration=standing binding=standing");

    // Arm 2: the pass settles the registration, removes the two inventories and then the binding,
    // removes the plan last and gives the claim back.
    let settle: TheLiveSettle = TheLiveSettle;
    let pass: FinishPass<'_> = FinishPass { common_dir: common.as_path(), folder: &TheLivePlansFolder, gate: &APassedGate, probe: &reader, settle: &settle };
    println!("FINISH step=arm-2-enter");
    let lines: Vec<String> = crate::worktree_clone_removal_finish_pass::pass_over_the_plans::finish_the_removal_plans(&pass);
    assert_eq!(lines.len(), 1, "the pass prints one line for the removal it finished, and this run printed {lines:?}");
    assert!(lines[0].contains("Finished the interrupted removal"), "the line says what it finished: {}", lines[0]);
    assert!(!fixture.registration().exists(), "the registration was settled");
    assert!(!fixture.slot_path().exists(), "the slot's underlay folder went with the settle");
    for name in [origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] {
        assert!(!home.join(name.as_str()).exists(), "`{name}` was removed");
    }
    let plan_folder: PathBuf = crate::worktree_clone_binding_record::registration_home::plans_folder(common.as_path());
    assert!(!plan_folder.exists(), "the plan and its empty folder went");
    let claims: Vec<String> = std::fs::read_dir(home.as_path()).expect("the wrapper's home lists").flatten().map(|entry| return entry.file_name().to_string_lossy().into_owned()).filter(|name| return name.starts_with(claim_prefix(SLOT).to_string_lossy().as_ref())).collect();
    assert_eq!(claims.len(), 1, "the pass left one claim file, and this run left {claims:?}");
    let text: String = std::fs::read_to_string(home.join(claims[0].as_str()).as_path()).expect("the claim reads");
    assert!(text.contains("released"), "the claim was given back after the plan's removal: {text}");
    println!("FINISH settled=registration records=removed plan=removed claim=released lines={}", lines.len());

    fixture.teardown();
    println!("FINISH done");
}
