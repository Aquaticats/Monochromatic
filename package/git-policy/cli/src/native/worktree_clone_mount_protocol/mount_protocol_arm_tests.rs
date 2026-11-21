//! What:
//!  Step 7's guest arms,
//! run as root in the throwaway `mvm` guest the ZFS
//!  binding's recipe prepares:
//!  malformed,
//!  oversized and inherited intent and a stale receipt with no mount,
//!  the nudge delivered under the proposed grants and refused on the parent,
//!  and the measurement from one property event to a test zedlet and to the
//!  product's receipt.
//! Why:
//!  These arms need a real pool,
//!  a real ZED and a real delegated account,
//!  and the plan's acceptance tests name exactly these cases
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 7:
//! The intent property,
//! the receipts and the readiness record";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding,
//! Git and lifecycle").
//!  A run on any other machine prints `SKIP <test>` and returns,
//!  and a run inside the guest panics when its fixture is missing,
//!  because a returning arm would be a passing arm whose assertions never ran.
//!  The measurement is kept in
//!  `package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const arm = guestArm(); arm.setMalformed(); arm.assertNoMount(); arm.measureEventToReceipt();
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::{IntentReading, MountOperation, read_intent};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::{DESIRED_STATE_PROPERTY, SWEEP_NONCE_PROPERTY};
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{CloneMountReceipt, ReceiptMountIdentity, ReceiptOutcome};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::RESULTS_DIRECTORY;
use crate::worktree_clone_mount_protocol::clone_mount_sweep_nonce::{NudgeReading, read_sweep_nonce};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::mount_request_wait::{MountRequest, WaitRefusal, wait_for_answer};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, ZfsDevice};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// The guest hostname variable.
const GUEST_VARIABLE: &str = "MONOCHROMATIC_ZFS_STEP7_GUEST";
/// The OpenZFS userspace `sbin` the arms call.
const SBIN_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_SBIN";
/// The pool the recipe prepared.
const POOL_VARIABLE: &str = "MONOCHROMATIC_ZFS_STEP7_POOL";
/// The clone parent.
const PARENT_VARIABLE: &str = "MONOCHROMATIC_ZFS_STEP7_PARENT";
/// The clone the arms read.
const CLONE_VARIABLE: &str = "MONOCHROMATIC_ZFS_STEP7_CLONE";
/// The delegated account.
const ACCOUNT_VARIABLE: &str = "MONOCHROMATIC_ZFS_STEP7_ACCOUNT";
/// The test zedlet's log.
const ZEDLET_LOG_VARIABLE: &str = "MONOCHROMATIC_ZFS_STEP7_ZEDLET_LOG";
/// The empty folder an attach would land in,
/// so the arms can show nothing attached.
const SLOT: &str = "/root/step7-slot";

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP worktree_clone_mount_protocol::mount_protocol_arm_tests: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One required fixture variable,
/// or a panic naming it.
fn required(variable: &str) -> String {
    return std::env::var(variable).unwrap_or_else(|_| panic!("{variable} is unset inside the guest, so this arm has no fixture to run against"));
}

/// Run one `zfs` or `zpool` command as root,
/// returning its status and both streams.
fn zfs(sbin: &str, program: &str, arguments: &[&str]) -> (bool, String, String) {
    let path: String = format!("{sbin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    let output: std::process::Output = Command::new(program)
        .args(arguments)
        .env("PATH", path)
        .env("LD_LIBRARY_PATH", format!("{}/lib", sbin.trim_end_matches("/sbin")))
        .output()
        .unwrap_or_else(|error| panic!("{program} {arguments:?} could not start: {error}"));
    return (
        output.status.success(),
        String::from_utf8_lossy(output.stdout.as_slice()).to_string(),
        String::from_utf8_lossy(output.stderr.as_slice()).to_string(),
    );
}

/// Set one property on one dataset as root,
/// and require the kernel took it.
fn set_property(sbin: &str, dataset: &str, property: &str, value: &str) {
    let assignment: String = format!("{property}={value}");
    let (ok, _out, error): (bool, String, String) = zfs(sbin, "zfs", &["set", assignment.as_str(), dataset]);
    assert!(ok, "zfs set {assignment} {dataset} was refused: {error}");
}

/// Read one dataset's property reply through the binding.
fn read_properties(device: &ZfsDevice, dataset: &str) -> DatasetIdentity {
    return device.objset_stats(dataset).unwrap_or_else(|error| panic!("{dataset} could not be read: {error}"));
}

/// Whether any mount claims the dataset as its source,
/// which is what an attach would have left.
fn mounted(dataset: &str) -> bool {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the arm can read this process's mount table");
    return table.lines().any(|line| return line.contains(&format!(" - zfs {dataset} ")));
}

/// Whether the slot folder is still empty,
/// which no attach would leave otherwise.
fn slot_empty() -> bool {
    let entries = std::fs::read_dir(Path::new(SLOT)).expect("the arm can list its own empty slot folder");
    return entries.count() == 0;
}

/// A magnitude for one duration in microseconds,
/// without a decimal place,
/// as the measurement lines print it.
fn microseconds(duration: Duration) -> u128 {
    return duration.as_micros();
}

/// What:
///  One property event's arrival at the test zedlet,
///  read from the zedlet's own log.
/// Why:
///   ZED delivers an event to the zedlets itself,
///  so the only honest reading of "delivered" is a line the zedlet wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arrived = waitForZedletLine(log, since);
/// ```
fn wait_for_zedlet_line(log: &Path, clone: &str, within: Duration) -> Duration {
    let started: Instant = Instant::now();
    let deadline: Instant = started + within;
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(log) {
            let seen: bool = text.lines().any(|line| {
                let mut words = line.split_whitespace();
                return words.next() == Some("zedlet") && words.any(|word| return word == clone);
            });
            if seen {
                return started.elapsed();
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the test zedlet wrote no line for {clone} within {within:?}; is zfs-zed running and the zedlet installed?");
}

/// Run one sweep-like answer for the current intent:
/// read the clone's own state through the binding,
/// publish a product receipt,
/// and answer through the product wait.
fn answer_and_measure(sbin: &str, device: &ZfsDevice, clone: &str, nonce: &RequestNonce, started: Instant) -> (Duration, Duration) {
    let identity: DatasetIdentity = read_properties(device, clone);
    let intent: IntentReading = read_intent(Some(&identity.properties), clone);
    let IntentReading::Request(request) = intent else {
        panic!("the clone's own valid intent was not a request: {intent:?}");
    };
    let guid: DatasetGuid = DatasetGuid::parse(identity.stats.dds_guid.to_string().as_str()).expect("the kernel's GUID is canonical");
    let receipt: CloneMountReceipt = CloneMountReceipt {
        dataset: guid,
        operation: request.operation,
        nonce: request.nonce.clone(),
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 0,
            device: identity.stats.dds_guid,
            inode: identity.stats.dds_creation_txg,
            source: String::from(clone),
            canonical_path: String::from(SLOT),
        }),
        failed_step: None,
        errno: None,
        boot: std::fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("the boot identifier is readable").trim().to_string(),
        time: crate::foundation::iso_time::now_iso(),
    };
    let folder: ReceiptsFolder = ReceiptsFolder::live();
    std::fs::create_dir_all(RESULTS_DIRECTORY).expect("the results directory is made for the arm");
    std::fs::set_permissions(RESULTS_DIRECTORY, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("the results directory takes its mode");
    folder.publish(&receipt).expect("the product files its receipt");
    let filed: Duration = started.elapsed();
    let answer_request: MountRequest = MountRequest { dataset: guid, operation: MountOperation::Mount, nonce: nonce.clone(), boot: receipt.boot.clone() };
    let runtime = tokio::runtime::Builder::new_current_thread().enable_time().build().expect("the arm's runtime starts");
    let answer: Result<CloneMountReceipt, WaitRefusal> = runtime.block_on(wait_for_answer(&folder, &answer_request, Duration::from_secs(5)));
    assert_eq!(answer.expect("the product wait finds its own receipt"), receipt);
    let _ = sbin;
    return (filed, started.elapsed());
}

/// The guest arm.
#[test]
#[ignore = "needs a real pool, ZED and the delegated account: the throwaway mvm guest the ZFS binding's recipe prepares runs it, and any other machine answers SKIP"]
fn mount_protocol_arm_intent_receipt_and_nudge() {
    if !guest() {
        return;
    }
    let sbin: String = required(SBIN_VARIABLE);
    let pool: String = required(POOL_VARIABLE);
    let parent: String = required(PARENT_VARIABLE);
    let clone: String = required(CLONE_VARIABLE);
    let account: String = required(ACCOUNT_VARIABLE);
    let zedlet_log: String = required(ZEDLET_LOG_VARIABLE);
    std::fs::create_dir_all(SLOT).expect("the empty slot folder is made");
    assert!(slot_empty(), "the slot must be empty before the arm starts");
    assert!(!mounted(clone.as_str()), "the clone must be unmounted before the arm starts");
    let device: ZfsDevice = ZfsDevice::open().expect("the guest permits the device");

    // A malformed local intent is refused before anything acts on it.
    set_property(sbin.as_str(), clone.as_str(), DESIRED_STATE_PROPERTY, "v1:nothex:mount");
    let malformed_identity: DatasetIdentity = read_properties(&device, clone.as_str());
    let malformed: IntentReading = read_intent(Some(&malformed_identity.properties), clone.as_str());
    assert!(matches!(malformed, IntentReading::Refused(_)), "{malformed:?}");
    assert!(!mounted(clone.as_str()), "a malformed intent mounted nothing");
    assert!(slot_empty(), "a malformed intent wrote nothing into the slot");

    // An oversized value is refused at the bound, not parsed.
    let oversized: String = format!("v1:{}:unmount!", "0".repeat(32));
    set_property(sbin.as_str(), clone.as_str(), DESIRED_STATE_PROPERTY, oversized.as_str());
    let oversized_identity: DatasetIdentity = read_properties(&device, clone.as_str());
    let too_long: IntentReading = read_intent(Some(&oversized_identity.properties), clone.as_str());
    assert!(matches!(too_long, IntentReading::Refused(crate::worktree_clone_mount_protocol::clone_mount_intent::IntentRefusal::TooLong { .. })), "{too_long:?}");
    assert!(!mounted(clone.as_str()), "an oversized intent mounted nothing");

    // An inherited value is ignored and authorizes nothing.
    let (inherited_removed, _inherited_out, inherited_error): (bool, String, String) = zfs(sbin.as_str(), "zfs", &["inherit", DESIRED_STATE_PROPERTY, clone.as_str()]);
    assert!(inherited_removed, "the clone's local intent could not be removed: {inherited_error}");
    let inherited: String = format!("v1:{}:mount", "1".repeat(32));
    set_property(sbin.as_str(), parent.as_str(), DESIRED_STATE_PROPERTY, inherited.as_str());
    let inherited_identity: DatasetIdentity = read_properties(&device, clone.as_str());
    let inherited_reading: IntentReading = read_intent(Some(&inherited_identity.properties), clone.as_str());
    assert!(matches!(inherited_reading, IntentReading::Inherited { .. }), "{inherited_reading:?}");
    assert!(!inherited_reading.asks_for_work(), "an inherited value asks for nothing");
    assert!(!mounted(clone.as_str()), "an inherited intent mounted nothing");
    let _ = zfs(sbin.as_str(), "zfs", &["inherit", DESIRED_STATE_PROPERTY, parent.as_str()]);

    // A stale receipt is another request's answer and authorizes nothing.
    let wanted: RequestNonce = RequestNonce::fresh().expect("the random source answers");
    set_property(sbin.as_str(), clone.as_str(), DESIRED_STATE_PROPERTY, format!("v1:{}:mount", wanted.as_str()).as_str());
    let stale_identity: DatasetIdentity = read_properties(&device, clone.as_str());
    let guid: DatasetGuid = DatasetGuid::parse(stale_identity.stats.dds_guid.to_string().as_str()).expect("the kernel's GUID is canonical");
    let stale_nonce: RequestNonce = RequestNonce::fresh().expect("the random source answers");
    let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("the boot identifier is readable").trim().to_string();
    let stale: CloneMountReceipt = CloneMountReceipt {
        dataset: guid,
        operation: MountOperation::Mount,
        nonce: stale_nonce,
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 1,
            device: 1,
            inode: 1,
            source: String::from(clone.as_str()),
            canonical_path: String::from(SLOT),
        }),
        failed_step: None,
        errno: None,
        boot: boot.clone(),
        time: crate::foundation::iso_time::now_iso(),
    };
    let folder: ReceiptsFolder = ReceiptsFolder::live();
    std::fs::create_dir_all(RESULTS_DIRECTORY).expect("the results directory is made for the arm");
    std::fs::set_permissions(RESULTS_DIRECTORY, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("the results directory takes its mode");
    folder.publish(&stale).expect("the stale receipt is filed");
    let stale_request: MountRequest = MountRequest { dataset: guid, operation: MountOperation::Mount, nonce: wanted, boot };
    let runtime = tokio::runtime::Builder::new_current_thread().enable_time().build().expect("the arm's runtime starts");
    let refusal: WaitRefusal = runtime
        .block_on(wait_for_answer(&folder, &stale_request, Duration::ZERO))
        .expect_err("a stale receipt is not this request's answer");
    assert!(matches!(refusal, WaitRefusal::TimedOut { last: Some(_), .. }), "{refusal:?}");
    assert!(!mounted(clone.as_str()), "a stale receipt mounted nothing");
    assert!(slot_empty(), "a stale receipt wrote nothing into the slot");

    // The nudge, under the design's proposed descendant grants only.
    let grant: Vec<&str> = vec!["-d", "-u", account.as_str(), "mount,userprop", parent.as_str()];
    let (granted, _grant_out, grant_error): (bool, String, String) = zfs(sbin.as_str(), "zfs", &[vec!["allow"], grant].concat());
    assert!(granted, "the proposed descendant grant was refused: {grant_error}");
    let nudge: RequestNonce = RequestNonce::fresh().expect("the random source answers");
    let assignment: String = format!("{SWEEP_NONCE_PROPERTY}={}", nudge.as_str());
    let path: String = format!("{sbin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    let delivered: std::process::Output = Command::new("su")
        .args(["--shell", "/bin/sh", account.as_str(), "--command", format!("PATH={path} zfs set {assignment} {clone}").as_str()])
        .output()
        .expect("the delegated set runs");
    assert!(delivered.status.success(), "the account could not set its clone's nudge: {}", String::from_utf8_lossy(delivered.stderr.as_slice()));
    let nudge_identity: DatasetIdentity = read_properties(&device, clone.as_str());
    let read_back: NudgeReading = read_sweep_nonce(Some(&nudge_identity.properties), clone.as_str());
    assert_eq!(read_back, NudgeReading::Nudge(nudge.clone()));
    let on_parent: String = format!("{SWEEP_NONCE_PROPERTY}={}", RequestNonce::fresh().expect("the random source answers").as_str());
    let refused: std::process::Output = Command::new("su")
        .args(["--shell", "/bin/sh", account.as_str(), "--command", format!("PATH={path} zfs set {on_parent} {parent}").as_str()])
        .output()
        .expect("the delegated parent set runs");
    assert!(!refused.status.success(), "the account changed the parent's nudge, which the proposal forbids");

    // One property event to the test zedlet and to the product's receipt.
    let _ = std::fs::remove_file(zedlet_log.as_str());
    let event_nonce: RequestNonce = RequestNonce::fresh().expect("the random source answers");
    let value: String = format!("v1:{}:mount", event_nonce.as_str());
    let started: Instant = Instant::now();
    set_property(sbin.as_str(), clone.as_str(), DESIRED_STATE_PROPERTY, value.as_str());
    let event_to_zedlet: Duration = wait_for_zedlet_line(Path::new(zedlet_log.as_str()), clone.as_str(), Duration::from_secs(10));
    let (event_to_receipt, event_to_answer): (Duration, Duration) = answer_and_measure(sbin.as_str(), &device, clone.as_str(), &event_nonce, started);
    println!("MEASUREMENT mount-protocol event-to-zedlet-us={}", microseconds(event_to_zedlet));
    println!("MEASUREMENT mount-protocol event-to-receipt-us={}", microseconds(event_to_receipt));
    println!("MEASUREMENT mount-protocol event-to-answer-us={}", microseconds(event_to_answer));
    assert!(!mounted(clone.as_str()), "no arm of this step attaches a mount");
    assert!(slot_empty(), "no arm of this step writes into the slot");
    println!("MEASUREMENT mount-protocol pool={pool} arms=intent-receipt-nudge");
}
