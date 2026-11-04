//! What:
//!  Controls of rule V (revision 12) for the durable-record components: a request is one checksummed file in `inbox/`, the state is a
//!  hash-chained log, the waiting path of a plain commit makes no sync of the wrapper's, and a departure forces to disk
//!  only what the leaving command promises, in the order of the dependency chain.
//! Why:
//!  The owner's rules "commits vs native git should not take 5s longer." and "Disk pressure being high is not an excuse." need a
//!  count: the earlier components made 22 sync calls to acceptance for a one-path commit. The count is measured here through the event
//!  log of the test seams, so the file is compiled only with the feature `test-seams`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the waiting path makes zero syncs', () => { expect(syncEvents(acceptAndWriteState())).toEqual([]); });
//! ```
#![cfg(feature = "test-seams")]

use crate::model_fixture::scratch;
use git_policy_cli::commit_identity::{bind, start_events, take_events};
use git_policy_cli::commit_inbox::{Accepted, accept, decode_inbox_file, depart, depart_closure, encode_inbox_file, init_lane_directories, with_identity};
use git_policy_cli::commit_integrity::sha256_hex;
use git_policy_cli::commit_request::{Change, Operation, Request, publish};
use git_policy_cli::commit_spool::write_spool;
use git_policy_cli::commit_state_log;
use std::path::PathBuf;

/// The lane instance of the tests.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// A one-path request (an added file) with its identity derived from its content and the instance.
fn one_path_request() -> (Request, Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let request: Request = with_identity(
        INSTANCE,
        Request {
            id: "0".repeat(32),
            branch: b"main".to_vec(),
            base: "b".repeat(40),
            message: b"m".to_vec(),
            operation: Operation::Commit,
            nonce: vec![1; 16],
            changes: vec![Change {
                path: b"f.txt".to_vec(),
                mode: "100644".to_string(),
                sha256: sha256_hex(b"new bytes"),
                old_mode: "000000".to_string(),
                old_sha256: "-".to_string(),
            }],
        },
    )
    .expect("identity");
    return (request, vec![b"new bytes".to_vec()], vec![Vec::new()]);
}

/// The number of syncs (file syncs and directory syncs) in `events`.
fn syncs(events: &[String]) -> usize {
    return events.iter().filter(|event| { return event.starts_with("file-sync:") || event.starts_with("dir:"); }).count();
}

/// The baseline the fourth review counted: the revision 11 path (binding, spool, record) syncs many times before acceptance.
#[test]
fn the_revision_eleven_acceptance_path_makes_many_syncs() {
    let root: PathBuf = scratch("v-baseline");
    let (request, new_files, old_files) = one_path_request();
    start_events();
    assert!(bind(&root, &request.id, &sha256_hex(b"hash")).is_ok());
    write_spool(&root, &request.id, &new_files, &old_files).expect("spool");
    publish(&root, &root.join("inbox"), &request).expect("publish");
    let count: usize = syncs(&take_events());
    println!("revision 11 acceptance path: {} syncs", count);
    assert!(count >= 15, "{}", count);
}

/// Rule V: accepting a request and writing the state makes no sync at all: the files are visible (they survive the death of any
/// process) and the syncer or a departure makes them durable later.
#[test]
fn the_waiting_path_of_a_plain_commit_makes_no_sync() {
    let root: PathBuf = scratch("v-waiting");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    start_events();
    let accepted: Accepted = accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    commit_state_log::append(&lane.join("state"), 1, b"adopted").expect("state");
    commit_state_log::append(&lane.join("state"), 2, b"ready").expect("state");
    let events: Vec<String> = take_events();
    assert_eq!(syncs(&events), 0, "{:?}", events);
    assert_eq!(accepted, Accepted::Created);
    assert_eq!(commit_state_log::read(&lane.join("state")).expect("readable").expect("state"), (2, b"ready".to_vec()));
}

/// A resend of the same request is answered from the file in place: no sync, no temporary file left, and the same bytes.
#[test]
fn a_resend_makes_no_sync_and_leaves_no_temporary_file() {
    let root: PathBuf = scratch("v-resend");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    start_events();
    assert_eq!(accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("resend"), Accepted::AlreadyPresent);
    assert_eq!(syncs(&take_events()), 0);
    assert_eq!(std::fs::read_dir(lane.join("inbox").join("tmp")).expect("tmp").count(), 0);
}

/// The departure barrier forces what the leaving command promises, in the order of the dependency chain: the request file, then the
/// inbox directory (its entry), then the newest state file.
#[test]
fn a_departure_syncs_the_promised_records_in_dependency_order_before_it_returns() {
    let root: PathBuf = scratch("v-departure");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    commit_state_log::append(&lane.join("state"), 1, b"adopted").expect("state");
    start_events();
    depart(&lane.join("inbox"), std::slice::from_ref(&request.id), &lane.join("state")).expect("depart");
    let events: Vec<String> = take_events();
    assert_eq!(
        events,
        vec![
            format!("file-sync:{}", lane.join("inbox").join(&request.id).display()),
            format!("dir:{}", lane.join("inbox").display()),
            format!("file-sync:{}", lane.join("state").join("state.log.1").display()),
            format!("dir:{}", lane.join("state").display()),
        ]
    );
}

/// What power loss may leave is recovered, not trusted: every truncation of the inbox file fails its checksum (it is treated as never
/// written), a changed byte fails it, and an intact file decodes to the request and both sides.
#[test]
fn a_torn_or_damaged_inbox_file_is_treated_as_never_written() {
    let (request, new_files, old_files) = one_path_request();
    let bytes: Vec<u8> = encode_inbox_file(&request, &new_files, &old_files).expect("encode");
    let (decoded, news, olds) = decode_inbox_file(&bytes).expect("decode");
    assert_eq!((decoded, news, olds), (request, new_files, old_files));
    for cut in 0..bytes.len() {
        assert!(decode_inbox_file(&bytes[..cut]).is_err(), "cut {}", cut);
    }
    for at in 0..bytes.len() {
        let mut flipped: Vec<u8> = bytes.clone();
        flipped[at] ^= 1;
        assert!(decode_inbox_file(&flipped).is_err(), "flip {}", at);
    }
}

/// An identity names one content: a request whose identity is not the hash of its content and the instance is refused, and so is a
/// side that does not match what the record promises.
#[test]
fn an_identity_names_exactly_one_content() {
    let root: PathBuf = scratch("v-identity");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    let mut forged: Request = request.clone();
    forged.message = b"another message".to_vec();
    assert!(accept(&lane.join("inbox"), INSTANCE, &forged, &new_files, &old_files).is_err());
    assert!(accept(&lane.join("inbox"), INSTANCE, &request, &[b"other bytes".to_vec()], &old_files).is_err());
}

/// Review 5 P1 3: two independent invocations with equal payloads carry different nonces and get different identities; a resend of one
/// invocation carries its retained nonce and gets the same identity.
#[test]
fn independent_identical_requests_get_different_identities_and_a_resend_the_same() {
    let (first, _, _) = one_path_request();
    let mut other: Request = first.clone();
    other.nonce = vec![2; 16];
    let second: Request = with_identity(INSTANCE, other).expect("identity");
    assert_ne!(first.id, second.id);
    let resend: Request = with_identity(INSTANCE, first.clone()).expect("identity");
    assert_eq!(resend.id, first.id);
    let mut short: Request = first.clone();
    short.nonce = vec![1; 3];
    assert!(with_identity(INSTANCE, short).is_err());
}

/// Review 5 P1 1: successive unsynced transitions then a power cut. The state log keeps its durable checkpoint (the alternating pair that
/// this replaced lost every valid generation in the same schedule and is removed).
#[test]
fn successive_unsynced_state_transitions_then_a_power_cut() {
    let root: PathBuf = scratch("v-powercut");
    let log: PathBuf = root.join("log");
    std::fs::create_dir_all(&log).expect("dir");
    commit_state_log::append(&log, 1, b"one").expect("append");
    commit_state_log::rotate(&log, 2, b"the checkpoint document").expect("rotate");
    commit_state_log::complete_rotation(&log).expect("durable checkpoint");
    let durable: u64 = std::fs::metadata(log.join("state.log.2")).expect("size").len();
    for generation in 3..6 {
        commit_state_log::append(&log, generation, b"unsynced transition").expect("append");
    }
    let bytes: Vec<u8> = std::fs::read(log.join("state.log.2")).expect("read");
    std::fs::write(log.join("state.log.2"), &bytes[..durable as usize + 7]).expect("the power cut tears the first unsynced record");
    assert_eq!(commit_state_log::read(&log).expect("readable").expect("durable checkpoint"), (2, b"the checkpoint document".to_vec()));
}

/// Review 5 P1 2: the departure forces the closure its exit promises, in dependency order: request files, the inbox directory, the order
/// file and its directory, the newest state log file and its directory (the state's name), then the ledger file and its directory.
#[test]
fn a_departure_forces_the_whole_closure_including_state_names_order_and_ledger() {
    let root: PathBuf = scratch("v-closure");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    let state: PathBuf = lane.join("state");
    commit_state_log::append(&state, 1, b"adopted").expect("append");
    let order: PathBuf = lane.join("queue").join("order");
    std::fs::write(&order, b"adopt line
").expect("order");
    let ledger: PathBuf = lane.join("queue").join("ledger");
    std::fs::write(&ledger, b"ledger line
").expect("ledger");
    start_events();
    depart_closure(&lane.join("inbox"), std::slice::from_ref(&request.id), Some(&order), &state, Some(&ledger)).expect("depart");
    let events: Vec<String> = take_events();
    let expect = |kind: &str, path: &std::path::Path| -> String { return format!("{}:{}", kind, path.display()); };
    assert_eq!(
        events,
        vec![
            expect("file-sync", &lane.join("inbox").join(&request.id)),
            expect("dir", &lane.join("inbox")),
            expect("file-sync", &order),
            expect("dir", &lane.join("queue")),
            expect("file-sync", &state.join("state.log.1")),
            expect("dir", &state),
            expect("file-sync", &ledger),
            expect("dir", &lane.join("queue")),
        ]
    );
}

/// Review 5 P2: the state log refuses a generation that does not increase.
#[test]
fn the_state_log_requires_increasing_generations() {
    let dir: PathBuf = scratch("v-generations");
    commit_state_log::append(&dir, 5, b"five").expect("append");
    assert!(commit_state_log::append(&dir, 5, b"again").is_err());
    assert!(commit_state_log::append(&dir, 4, b"older").is_err());
    commit_state_log::append(&dir, 6, b"six").expect("append");
}
