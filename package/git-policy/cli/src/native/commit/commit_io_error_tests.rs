//! What:
//!  Controls of the departure's and the seal's forcing when the disk answers with an error (EIO, ENOSPC, EROFS), through the test seam that
//!  makes a file or directory sync fail once.
//! Why:
//!  After an `fsync` error the kernel may have dropped the dirty pages, so a second `fsync` of the same file can succeed without the data
//!  being durable. A departure therefore never retries a forcing error and never reports success after one: it returns the error, and the
//!  caller takes the outside-fault exit (owner decision 14: a returned error is an outside I/O failure). The seal that fails to force its
//!  closure leaves the older root in place. Only ENOENT (the segment was unlinked by a reclaimer) is retried, and it is retried by
//!  selecting again, never by forcing again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // injectSyncError(afterSuccesses, errno); expect(() => departureAcknowledge(state, 3)).toThrow(errno); expect(forcingAttempts()).toBe(1);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::lane_with_envelope;
use git_policy_cli::commit_identity::{inject_sync_error, start_events, take_events};
use git_policy_cli::commit_lane_lifecycle::{capture_lane_closure, departure_acknowledge, seal_for_shutdown};
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, select};
use std::path::PathBuf;

/// The errors a disk answers a forcing with.
const DISK_ERRORS: [i32; 3] = [libc::EIO, libc::ENOSPC, libc::EROFS];

/// A lane whose state log holds one checked record.
fn lane_with_record(label: &str) -> (PathBuf, PathBuf) {
    let (lane, state, _envelope) = lane_with_envelope(label);
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the record to acknowledge");
    return (lane, state);
}

/// The number of completed file syncs in `events`.
fn file_syncs(events: &[String]) -> usize {
    return events.iter().filter(|event| { return event.starts_with("file-sync:"); }).count();
}

/// A file sync that fails is the departure's answer: the error, with the disk's number, and no second forcing (a retry would record a
/// completed sync here, and could report a success the data does not have).
#[test]
fn a_failed_file_sync_is_returned_and_never_retried() {
    for errno in DISK_ERRORS {
        let (_lane, state) = lane_with_record("io-file-sync");
        start_events();
        inject_sync_error(Some((0, errno)));
        let failed = departure_acknowledge(&state, 3);
        inject_sync_error(None);
        let events: Vec<String> = take_events();
        let error = failed.expect_err("the forcing error was answered with a success");
        assert_eq!(error.raw_os_error(), Some(errno), "{}", error);
        assert_eq!(file_syncs(&events), 0, "the failed sync was attempted again: {:?}", events);
    }
}

/// A directory sync that fails after the file sync succeeded is the same: the error, and no second pass over the file.
#[test]
fn a_failed_directory_sync_is_returned_and_the_file_is_not_forced_again() {
    for errno in DISK_ERRORS {
        let (_lane, state) = lane_with_record("io-directory-sync");
        start_events();
        inject_sync_error(Some((1, errno)));
        let failed = departure_acknowledge(&state, 3);
        inject_sync_error(None);
        let events: Vec<String> = take_events();
        assert_eq!(failed.expect_err("a success after a failed directory sync").raw_os_error(), Some(errno));
        assert_eq!(file_syncs(&events), 1, "the file was forced once and not again: {:?}", events);
    }
}

/// After the disk recovers, the same call acknowledges: the error was not remembered as a state, and nothing was left half done.
#[test]
fn the_next_call_after_the_disk_recovers_acknowledges() {
    let (_lane, state) = lane_with_record("io-recovers");
    inject_sync_error(Some((0, libc::EIO)));
    assert!(departure_acknowledge(&state, 3).is_err());
    inject_sync_error(None);
    assert!(departure_acknowledge(&state, 3).is_ok());
}

/// The seal that cannot force its closure fails and leaves the older root in place: the predecessor segment stays, recovery selects it,
/// and a seal after the disk recovers succeeds.
#[test]
fn a_seal_whose_forcing_fails_leaves_the_older_root_and_can_be_repeated() {
    for errno in DISK_ERRORS {
        let (lane, state) = lane_with_record("io-seal");
        inject_sync_error(Some((0, errno)));
        let failed = seal_for_shutdown(&lane, &state);
        inject_sync_error(None);
        assert!(failed.is_err(), "a seal reported success after a forcing error ({})", errno);
        assert!(state.join("state.log.1").exists(), "the older root was removed although the seal did not complete");
        assert!(select(&state).expect("readable").expect("a root").generation >= 1);
        assert!(seal_for_shutdown(&lane, &state).is_ok(), "the repeated seal after the disk recovered");
    }
}
