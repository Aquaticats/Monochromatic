//! What:
//!  Controls of the nudge:
//!  the gap that doubles and stops,
//!  the link that leaves one owner of one attempt,
//!  the outcome a helper records,
//!  and what a helper killed before it records leaves behind.
//! Why:
//!  The link is what makes a burst one nudge,
//!  and a helper that dies must leave the clone's trigger standing in the
//!  attempt under its claim,
//!  or a killed helper would cost the clone its recovery
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  test 16,
//!  and the record's choice 5 on what a killed helper leaves).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(claimTheNextAttempt(folder, entry)).toBeDefined();
//! expect(claimTheNextAttempt(folder, entry)).toBeUndefined(); // one link winner
//! ```

use super::{
    TheGaps,
    ask_for_a_nudge,
    claim_the_next_attempt,
    is_due,
    the_gap_before,
    the_gaps,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::{
    RequestEntry,
    RequestOutcome,
};
use crate::worktree_clone_mount_protocol::unanswered_request_names::EntryName;
use crate::worktree_clone_mount_protocol::unanswered_requests_folder::{
    UnansweredRequests,
    this_account,
};
use std::path::PathBuf;
use std::time::{
    Duration,
    SystemTime,
};

/// The clone every control writes.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("1234567890123456789").expect("the control GUID is canonical");
}

/// One waiting entry of this clone.
fn waiting() -> RequestEntry {
    return RequestEntry {
        guid: guid(),
        nonce: RequestNonce::parse("0123456789abcdef".repeat(2).as_str())
            .expect("the control nonce is 32 hexadecimal digits"),
        operation: MountOperation::Mount,
        written_at: String::from("2026-11-27T21:14:14.000Z"),
        outcome: RequestOutcome::Waiting,
        dataset: String::from("zpcachyos/nosnap/clones/one"),
    };
}

/// A folder of unanswered requests in a fixture of its own.
fn folder(name: &str) -> UnansweredRequests {
    let root: PathBuf = fixture(name);
    return UnansweredRequests::at(root.join("zfs-unanswered-requests"), this_account());
}

/// The gaps a control states,
/// since no number ships from this build.
const STATED: TheGaps = TheGaps {
    first: Duration::from_secs(1),
    longest: Duration::from_secs(8),
    wait_delay: Duration::from_secs(1),
};

/// The gap doubles from the first attempt and stops at the longest.
#[test]
fn the_gap_doubles_from_the_first_and_stops_at_the_longest() {
    let expected: Vec<u64> = vec![1, 2, 4, 8, 8, 8];
    let read: Vec<u64> = (1..=expected.len() as u32)
        .map(|attempt| return the_gap_before(attempt, &STATED).as_secs())
        .collect();
    assert_eq!(read, expected);
    assert_eq!(
        the_gap_before(u32::MAX, &STATED).as_secs(),
        8,
        "an attempt number no name reaches still stops at the longest gap"
    );
}

/// An attempt is due once its own gap has passed and not before.
#[test]
fn an_attempt_is_due_once_its_gap_has_passed() {
    let written: SystemTime = SystemTime::UNIX_EPOCH;
    let inside: SystemTime = written + Duration::from_millis(999);
    let at_the_gap: SystemTime = written + Duration::from_secs(1);
    assert!(!is_due(1, written, inside, &STATED));
    assert!(is_due(1, written, at_the_gap, &STATED));
    assert!(
        !is_due(2, written, at_the_gap, &STATED),
        "the second attempt waits twice the first gap"
    );
    assert!(
        is_due(9, written, written + Duration::from_secs(8), &STATED),
        "from the attempt that reaches the longest gap on, one nudge per longest gap"
    );
}

/// This build ships no gaps,
/// so the switch's numbers are still owed and no command nudges anything.
#[test]
fn this_build_ships_no_gaps() {
    assert_eq!(
        the_gaps(),
        None,
        "step 22 answers the gaps from M8, with the measurement beside each number"
    );
    let requests: UnansweredRequests = folder("nudge-no-gaps");
    let name: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants an entry");
    assert_eq!(
        ask_for_a_nudge(&requests, name, &waiting()),
        None,
        "a build whose gaps are not measured claims nothing"
    );
    assert_eq!(
        std::fs::read_dir(requests.path())
            .expect("the folder lists")
            .flatten()
            .count(),
        1,
        "and it writes no attempt"
    );
}

/// The first claim takes the number past the waiting entry,
/// and a later claim takes the number past the claim,
/// which is what spaces a clone's attempts by its gaps.
#[test]
fn the_first_claim_takes_the_number_past_the_waiting_entry() {
    let requests: UnansweredRequests = folder("nudge-numbers");
    let _waiting: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants the waiting entry");
    let first: Option<EntryName> = claim_the_next_attempt(&requests, &waiting());
    let second: Option<EntryName> = claim_the_next_attempt(&requests, &waiting());
    assert_eq!(
        first,
        Some(EntryName {
            guid: guid(),
            attempt: 2
        })
    );
    assert_eq!(
        second,
        Some(EntryName {
            guid: guid(),
            attempt: 3
        }),
        "a claim after a claim takes the number past the highest name present"
    );
}

/// One name has one link winner,
/// which is what makes a burst one nudge:
/// the writer that lost the link writes nothing more and prints nothing.
#[test]
fn one_name_has_one_link_winner() {
    let requests: UnansweredRequests = folder("nudge-one-name");
    let _waiting: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants the waiting entry");
    let contested: EntryName = EntryName {
        guid: guid(),
        attempt: 2,
    };
    let claimed: RequestEntry = RequestEntry {
        outcome: RequestOutcome::Claimed,
        ..waiting()
    };
    assert!(
        matches!(
            requests.publish_a_new_attempt(contested, &claimed),
            Ok(true)
        ),
        "the first writer of a burst takes the name"
    );
    assert!(
        matches!(
            requests.publish_a_new_attempt(contested, &claimed),
            Ok(false)
        ),
        "the second writer of the same name lost the link, and a lost link is not a failure"
    );
    assert_eq!(
        requests.attempts_of(guid()),
        vec![1, 2],
        "the burst left one claim beside the attempt under it"
    );
}

/// What:
///  A helper killed before it records its outcome leaves the attempt it claimed
///  and the attempt under it,
///  the reader still takes the readable lower one,
///  and the next claim takes the number past both.
/// Why:
///  This is the record's choice 5 stated as a fact:
///  a killed helper leaves one extra file per gap until the request is
///  answered,
///  and the clone's trigger survives it.
///  The killed process is a `sleep` the control owns,
///  which stands in the helper until step 22 brings the helper's own mode and
///  spawn:
///  what the folder must show is the claim and the attempt under it,
///  whatever process failed to record.
#[test]
fn a_helper_killed_before_it_records_leaves_the_claim_and_the_attempt_under_it() {
    let requests: UnansweredRequests = folder("nudge-killed-helper");
    let under: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants the waiting entry");
    let claim: EntryName = claim_the_next_attempt(&requests, &waiting()).expect("the claim wins");
    let mut helper: std::process::Command = std::process::Command::new("/usr/bin/sleep");
    helper.arg("60");
    let mut child: std::process::Child =
        crate::testing::test_support::spawn_owned(&mut helper).expect("the stand-in helper starts");
    child.kill().expect("the control kills the stand-in helper");
    let ended: std::process::ExitStatus = child.wait().expect("the stand-in helper ends");
    assert!(!ended.success(), "the stand-in died by a signal: {ended:?}");
    assert_eq!(
        requests.attempts_of(guid()),
        vec![under.attempt, claim.attempt],
        "the claim and the attempt under it stand, which is one extra file per gap"
    );
    let held: Vec<(EntryName, RequestEntry)> = requests.the_entries();
    assert_eq!(held.len(), 1, "one entry per clone is read");
    assert_eq!(
        held[0].0, claim,
        "the claim is readable, so it is the attempt the reader takes, and the clone's trigger survives the kill"
    );
    assert_eq!(held[0].1.outcome, RequestOutcome::Claimed);
    assert_eq!(
        held[0].1.nonce,
        waiting().nonce,
        "the attempt the reader takes names the request that still waits"
    );
    assert_ne!(held[0].0, under);
    let next: EntryName =
        claim_the_next_attempt(&requests, &waiting()).expect("the next claim takes a free name");
    assert_eq!(
        next.attempt, 3,
        "the next claim takes the number one past the highest name present"
    );
}

/// A common directory the wrapper cannot write in stops the command there:
/// no staging write,
/// no link,
/// no claim,
/// and no line.
#[test]
fn a_common_directory_the_command_cannot_write_in_claims_nothing() {
    let requests: UnansweredRequests = folder("nudge-read-only");
    std::fs::create_dir_all(requests.path()).expect("the control makes the folder");
    let _closed = std::fs::set_permissions(
        requests.path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o500),
    );
    assert_eq!(
        claim_the_next_attempt(&requests, &waiting()),
        None,
        "a claim that cannot be written is no claim and no line"
    );
    std::fs::set_permissions(
        requests.path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("the control opens the folder again for the fixture's removal");
}
