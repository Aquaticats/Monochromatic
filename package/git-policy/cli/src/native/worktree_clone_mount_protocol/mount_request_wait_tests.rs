//! What:
//!  Controls of the wait and the stale result:
//!  a matching receipt ends the wait,
//!  another request's receipt is the last observation and not an answer,
//!  and the deadline reports the request and how to inspect the handler.
//! Why:
//!  The plan's acceptance test asks that stale results cause no unauthorized
//!  filesystem action,
//!  and the design says a matching receipt is confirmed by the caller's own
//!  mount view
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Waiting and refusal").
//!  The frozen clock reaches the deadline without waiting it out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(runtime.blockOn(waitForAnswer(folder, request, 0n)).kind).toBe('timed-out');
//! ```

use super::{
    LastObservation,
    MOUNT_REQUEST_POLL_INTERVAL,
    MountRequest,
    PROVISIONAL_MOUNT_REQUEST_DEADLINE,
    ReceiptStanding,
    StaleReason,
    WaitRefusal,
    reads_taken,
    standing_of,
    wait_for_answer,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    CloneMountReceipt,
    ReceiptMountIdentity,
    ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

/// The dataset every control files under.
const GUID: u64 = 7;

/// The boot this test runs in.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The request's nonce.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

/// Another request's nonce.
const OTHER_NONCE: &str = "fedcba9876543210fedcba9876543210";

/// This account's number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// The request the controls wait on.
fn request() -> MountRequest {
    return MountRequest {
        dataset: DatasetGuid::parse(GUID.to_string().as_str())
            .expect("the control GUID is canonical"),
        operation: MountOperation::Mount,
        nonce: RequestNonce::parse(NONCE).expect("the control nonce is valid"),
        boot: String::from(BOOT),
    };
}

/// A receipt for the request,
/// with the nonce and operation the caller chooses.
fn receipt(
    nonce: &str,
    operation: MountOperation,
) -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: DatasetGuid::parse(GUID.to_string().as_str())
            .expect("the control GUID is canonical"),
        operation,
        nonce: RequestNonce::parse(nonce).expect("the control nonce is valid"),
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 9,
            device: 2049,
            inode: 12,
            source: String::from("zpcachyos/nosnap/clones/one"),
            canonical_path: String::from("/home/user/repo/.claude/worktrees/one"),
        }),
        failed_step: None,
        errno: None,
        boot: String::from(BOOT),
        time: String::from("2026-11-21T15:00:00Z"),
    };
}

/// A fixture folder with the designed mode,
/// owned by this account.
fn folder(name: &str) -> ReceiptsFolder {
    let root: PathBuf = fixture(name);
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the fixture takes the directory mode");
    return ReceiptsFolder::at(root, this_uid());
}

/// The runtime the async wait runs on,
/// with time enabled so the deadline check has a monotonic clock.
fn runtime() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("the control runtime starts");
}

/// A matching receipt ends the wait and is the caller's answer.
#[test]
fn a_matching_receipt_ends_the_wait() {
    let folder: ReceiptsFolder = folder("wait-answer");
    let answer: CloneMountReceipt = receipt(NONCE, MountOperation::Mount);
    folder.publish(&answer).expect("the receipt is filed");
    let found: CloneMountReceipt = runtime()
        .block_on(wait_for_answer(&folder, &request(), Duration::from_secs(1)))
        .expect("the matching receipt is the answer");
    assert_eq!(found, answer);
}

/// A receipt under another nonce is stale:
/// the wait does not accept it and the deadline names it as the last
/// observation,
/// so nothing acted on it.
#[test]
fn a_stale_receipt_is_not_an_answer_and_is_reported() {
    let folder: ReceiptsFolder = folder("wait-stale");
    folder
        .publish(&receipt(OTHER_NONCE, MountOperation::Mount))
        .expect("the stale receipt is filed");
    let refusal: WaitRefusal = runtime()
        .block_on(wait_for_answer(&folder, &request(), Duration::ZERO))
        .expect_err("a stale receipt is not this request's answer");
    let WaitRefusal::TimedOut {
        request: waited,
        last,
        ..
    } = &refusal
    else {
        panic!("the wait times out: {refusal:?}");
    };
    assert_eq!(waited.as_ref(), &request());
    assert!(
        matches!(
            last,
            Some(LastObservation::Stale(StaleReason::Nonce { .. }))
        ),
        "{last:?}"
    );
    let message: String = refusal.message();
    assert!(message.contains(NONCE), "{message}");
    assert!(message.contains("not this request's answer"), "{message}");
    assert!(message.contains("doctor"), "{message}");
    assert!(
        message.contains("A deadline is not cancellation"),
        "{message}"
    );
}

/// A receipt for the same nonce but the other operation is stale,
/// and so is one from another boot.
#[test]
fn the_other_fields_are_compared_too() {
    let answer: CloneMountReceipt = receipt(NONCE, MountOperation::Mount);
    assert_eq!(
        standing_of(&request(), &answer),
        ReceiptStanding::Answer(answer.clone())
    );
    let other_operation: CloneMountReceipt = receipt(NONCE, MountOperation::Unmount);
    assert!(matches!(
        standing_of(&request(), &other_operation),
        ReceiptStanding::Stale(StaleReason::Operation { .. })
    ));
    let mut other_boot: CloneMountReceipt = answer;
    other_boot.boot = String::from("11111111-2222-4333-8444-555555555555");
    assert!(matches!(
        standing_of(&request(), &other_boot),
        ReceiptStanding::Stale(StaleReason::Boot { .. })
    ));
    let mut other_nonce: CloneMountReceipt = other_operation;
    other_nonce.nonce = RequestNonce::parse(OTHER_NONCE).expect("the other nonce is valid");
    assert!(matches!(
        standing_of(&request(), &other_nonce),
        ReceiptStanding::Stale(StaleReason::Nonce { .. })
    ));
    let mut other_dataset: CloneMountReceipt = receipt(NONCE, MountOperation::Mount);
    other_dataset.dataset = DatasetGuid::parse("8").expect("8 is canonical");
    assert!(matches!(
        standing_of(&request(), &other_dataset),
        ReceiptStanding::Stale(StaleReason::Dataset { .. })
    ));
}

/// What:
///  The deadline the plan's step 7 states and leaves to step 22.
/// Why:
///   The value is provisional,
///  and this constant is the only place the control names it,
///  so the check reads the plan's number rather than a number of its own.
const PLAN_PROVISIONAL_DEADLINE: Duration = Duration::from_secs(30);

/// What:
///  The slowest answer the measurement document keeps,
///  in microseconds
///  (`package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`).
/// Why:
///   The deadline must leave far more room than any measured round,
///  and the check reads that round rather than asserting a history of its own.
const SLOWEST_KEPT_ANSWER: Duration = Duration::from_micros(12_149);

/// The provisional deadline is the plan's open value,
/// it leaves far more room than the slowest kept round,
/// and one poll is shorter than the deadline it sleeps inside.
#[test]
fn the_deadline_is_provisional_and_generous() {
    assert_eq!(
        PROVISIONAL_MOUNT_REQUEST_DEADLINE,
        PLAN_PROVISIONAL_DEADLINE
    );
    assert!(
        PROVISIONAL_MOUNT_REQUEST_DEADLINE > SLOWEST_KEPT_ANSWER,
        "the deadline leaves room over the slowest round"
    );
    assert!(
        MOUNT_REQUEST_POLL_INTERVAL < PROVISIONAL_MOUNT_REQUEST_DEADLINE,
        "the wait sleeps whole polls inside its deadline"
    );
    assert!(
        PROVISIONAL_MOUNT_REQUEST_DEADLINE.as_micros() % MOUNT_REQUEST_POLL_INTERVAL.as_micros()
            == 0,
        "the deadline is a whole number of polls"
    );
}

/// What:
///  A wait for a request whose results directory does not exist yet keeps
///  looking until its deadline and names the absent directory in the report.
/// Why:
///   The first request after a boot meets no directory at all,
///  and ending at once would refuse in no time instead of waiting for the
///  handler to file its first answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = waitForAnswer(absentFolder, request, 300n); expect(elapsed).toBeAtLeast(300n);
/// ```
#[test]
fn an_absent_results_directory_is_a_last_observation_and_never_the_end() {
    let absent: ReceiptsFolder =
        ReceiptsFolder::at(fixture("wait-absent").join("never-made"), this_uid());
    let deadline: Duration = Duration::from_millis(300);
    let started: std::time::Instant = std::time::Instant::now();
    let refusal: WaitRefusal = runtime()
        .block_on(wait_for_answer(&absent, &request(), deadline))
        .expect_err("no directory files no answer");
    let elapsed: Duration = started.elapsed();
    assert!(
        elapsed >= deadline,
        "the wait looked until its deadline: {elapsed:?}"
    );
    let WaitRefusal::TimedOut {
        last: Some(LastObservation::AbsentDirectory {
            path,
        }),
        ..
    } = &refusal
    else {
        panic!("the deadline reports the absent directory: {refusal:?}");
    };
    assert_eq!(path, &absent.directory().to_path_buf());
    let message: String = refusal.message();
    assert!(
        message.contains(
            absent
                .directory()
                .to_str()
                .expect("the fixture path is text")
        ),
        "{message}"
    );
}

/// What:
///  A results directory that appears and stays empty before the deadline is
///  reported as no receipt,
///  never as a directory that was not there.
/// Why:
///   The report sends a person to the handler's journal,
///  and "the directory was not there at the deadline" is wrong once the wait read
///  it successfully afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await join(waitForAnswer(folder, request, 400n), makeDirectoryAfter(20n)); expect(last?.kind).toBeUndefined();
/// ```
#[test]
fn a_directory_that_appears_empty_is_not_reported_as_absent() {
    let parent: PathBuf = fixture("wait-directory-appears");
    let absent: ReceiptsFolder = ReceiptsFolder::at(parent.join("late"), this_uid());
    let deadline: Duration = Duration::from_millis(400);
    runtime().block_on(async {
        let make = {
            let path: PathBuf = absent.directory().to_path_buf();
            async move {
                tokio::time::sleep(MOUNT_REQUEST_POLL_INTERVAL / 2).await;
                std::fs::create_dir_all(path.as_path()).expect("the results directory appears");
                std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o755))
                    .expect("the directory takes its mode");
            }
        };
        let request: MountRequest = request();
        let waited = wait_for_answer(&absent, &request, deadline);
        let (found, ()) = tokio::join!(waited, make);
        let refusal: WaitRefusal = found.expect_err("an empty directory files no answer");
        let WaitRefusal::TimedOut {
            last,
            ..
        } = &refusal
        else {
            panic!("the wait times out: {refusal:?}");
        };
        assert_eq!(
            last, &None,
            "the deadline reports no receipt rather than a directory that is there: {refusal:?}"
        );
    });
}

/// What:
///  How many reads the deadline of the sleep control holds,
///  which is how many sleeps a wait with that deadline takes between them.
/// Why:
///   The count is the control's own and not the route's,
///  so it is stated once here and the deadline and its bound both read it.
const READS_IN_THE_CONTROL_DEADLINE: u32 = 4;

/// What:
///  The reads a sleeping wait makes at the edges of its deadline:
///  the first read,
///  and the last look the loop takes once its final sleep returned.
/// Why:
///   The bound and the proof that the count is live both read this number,
///  so a wait that reads fewer times than this is a wait that never looked
///  again.
const READS_AT_THE_DEADLINES_EDGES: u64 = 2;

/// What:
///  A wait that no receipt answers reads a bounded number of times in a fixed
///  span,
///  which is the sleep between its reads.
/// Why:
///   Every other control hands the wait its receipt at once or a deadline that
///  already passed,
///  so none of them holds the sleep:
///  a wait without it still answers,
///  and reads in a tight loop until its deadline.
///  The control counts the reads of one wait of its own on the thread it runs
///  that wait on,
///  so no other control's waits are in the count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reads = waitAndCountReads(staleFolder, request, 4n * pollInterval); expect(reads).toBeLessThanOrEqual(4 + 2);
/// ```
#[test]
fn a_wait_with_no_answer_sleeps_between_its_reads() {
    let folder: ReceiptsFolder = folder("wait-poll-sleep");
    folder
        .publish(&receipt(OTHER_NONCE, MountOperation::Mount))
        .expect("a stale receipt keeps the wait looking");
    let deadline: Duration = MOUNT_REQUEST_POLL_INTERVAL * READS_IN_THE_CONTROL_DEADLINE;
    let allowed: u64 = u64::from(READS_IN_THE_CONTROL_DEADLINE) + READS_AT_THE_DEADLINES_EDGES;
    let before: u64 = reads_taken();
    let refusal: WaitRefusal = runtime()
        .block_on(wait_for_answer(&folder, &request(), deadline))
        .expect_err("a stale receipt never answers");
    let reads: u64 = reads_taken() - before;
    assert!(
        matches!(refusal, WaitRefusal::TimedOut { .. }),
        "the wait looked until its deadline: {refusal:?}"
    );
    assert!(
        reads >= READS_AT_THE_DEADLINES_EDGES,
        "the wait read more than once: {reads} reads"
    );
    assert!(
        reads <= allowed,
        "the wait slept between its reads: {reads} reads in {deadline:?} at {MOUNT_REQUEST_POLL_INTERVAL:?} a read"
    );
}

/// What:
///  An answer filed after the wait started ends it,
///  which is the second poll and the sleep between the two.
/// Why:
///   Every other control hands the wait a receipt that is already on the file or
///  a zero deadline,
///  so nothing holds the poll loop:
///  removing the sleep or the second look would leave them all green.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [found] = await join(waitForAnswer(folder, request, 5s), sleepThenPublish());
/// ```
#[test]
fn an_answer_filed_after_the_wait_started_ends_it() {
    let folder: ReceiptsFolder = folder("wait-second-poll");
    let answer: CloneMountReceipt = receipt(NONCE, MountOperation::Mount);
    runtime().block_on(async {
        let publish = {
            let late_folder: ReceiptsFolder = folder.clone();
            let late_answer: CloneMountReceipt = answer.clone();
            async move {
                tokio::time::sleep(MOUNT_REQUEST_POLL_INTERVAL / 2).await;
                late_folder
                    .publish(&late_answer)
                    .expect("the late answer is filed");
            }
        };
        let request: MountRequest = request();
        let waited = wait_for_answer(&folder, &request, Duration::from_secs(5));
        let (found, ()) = tokio::join!(waited, publish);
        assert_eq!(
            found.expect("the late answer ends the wait"),
            answer,
            "the second poll read the answer"
        );
    });
}
