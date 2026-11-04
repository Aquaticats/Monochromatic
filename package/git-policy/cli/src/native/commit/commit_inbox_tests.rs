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
use git_policy_cli::commit_identity::{set_fault_point, start_events, take_events};
use git_policy_cli::commit_inbox::{Accepted, accept, decode_inbox_file, depart, depart_closure, encode_inbox_file, init_lane_directories, with_identity};
use git_policy_cli::commit_integrity::sha256_hex;
use git_policy_cli::commit_request::{Change, Operation, Request};
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

/// Appends to a state directory as its writer.
fn state_append(dir: &std::path::Path, generation: u64, payload: &[u8]) -> std::io::Result<()> {
    return commit_state_log::StateWriter::acquire(dir)?.append(generation, payload);
}

/// The number of syncs (file syncs and directory syncs) in `events`.
fn syncs(events: &[String]) -> usize {
    return events.iter().filter(|event| { return event.starts_with("file-sync:") || event.starts_with("dir:"); }).count();
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
    state_append(&lane.join("state"), 1, b"adopted").expect("state");
    state_append(&lane.join("state"), 2, b"ready").expect("state");
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
    state_append(&lane.join("state"), 1, b"adopted").expect("state");
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
    state_append(&log, 1, b"one").expect("append");
    commit_state_log::StateWriter::acquire(&log).expect("writer").checkpoint(2, b"the checkpoint document", &[]).expect("durable checkpoint");
    let durable: u64 = std::fs::metadata(log.join("state.log.2")).expect("size").len();
    for generation in 3..6 {
        state_append(&log, generation, b"unsynced transition").expect("append");
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
    state_append(&state, 1, b"adopted").expect("append");
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
    state_append(&dir, 5, b"five").expect("append");
    assert!(state_append(&dir, 5, b"again").is_err());
    assert!(state_append(&dir, 4, b"older").is_err());
    state_append(&dir, 6, b"six").expect("append");
}

/// The inbox directory whose temporary name the fault point below turns into a directory.
static CLEANUP_TARGET: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// At `accept:after-link`, replaces the temporary file by a directory, so removing it as a file fails for every user.
fn break_the_cleanup(point: &str) {
    if point != "accept:after-link" {
        return;
    }
    let Some(tmp) = CLEANUP_TARGET.lock().expect("lock").clone() else { return };
    for entry in std::fs::read_dir(&tmp).expect("tmp").flatten() {
        std::fs::remove_file(entry.path()).expect("remove the temporary");
        std::fs::create_dir(entry.path()).expect("put a directory in its place");
    }
}

/// Review 5 item 3.6: the link succeeded and the temporary name could not be removed. The request is published, so `accept` must say so
/// (a distinct value), not return an error that a caller would read as "not accepted" and answer with a native Git commit beside a
/// queued request.
#[test]
fn a_failed_cleanup_after_a_successful_link_is_not_an_error() {
    let root: PathBuf = scratch("v-cleanup");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    *CLEANUP_TARGET.lock().expect("lock") = Some(lane.join("inbox").join("tmp"));
    set_fault_point(Some(break_the_cleanup));
    let outcome = accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files);
    set_fault_point(None);
    assert!(matches!(outcome, Ok(Accepted::CreatedTemporaryLeft(_))), "{:?}", outcome);
    assert!(lane.join("inbox").join(&request.id).exists(), "the request is in place");
    assert_eq!(accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("resend"), Accepted::AlreadyPresent);
}

/// F7 reopened against the inbox file: two clients send the same bytes, the first is stopped after it wrote its temporary file and before
/// it links, the second finishes; the first then finds `EEXIST` and answers `AlreadyPresent` (a resend), and exactly one file and no
/// temporary name remain. Whoever answers `AlreadyPresent` has not forced anything, which is why a departure forces the file whoever linked it.
#[test]
fn two_clients_sending_the_same_bytes_end_with_one_file_and_a_resend() {
    static ARRIVED: std::sync::Barrier = std::sync::Barrier::new(2);
    static RELEASE: std::sync::Barrier = std::sync::Barrier::new(2);
    fn pause(point: &str) {
        if point == "accept:before-link" {
            ARRIVED.wait();
            RELEASE.wait();
        }
    }
    let root: PathBuf = scratch("v-two-clients");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let inbox: PathBuf = lane.join("inbox");
    let (request, new_files, old_files) = one_path_request();
    let (paused_inbox, paused_request, paused_new, paused_old) = (inbox.clone(), request.clone(), new_files.clone(), old_files.clone());
    let paused = std::thread::spawn(move || {
        set_fault_point(Some(pause));
        return accept(&paused_inbox, INSTANCE, &paused_request, &paused_new, &paused_old);
    });
    ARRIVED.wait();
    // The second client is a different thread without the pause: it links first.
    assert_eq!(accept(&inbox, INSTANCE, &request, &new_files, &old_files).expect("second client"), Accepted::Created);
    RELEASE.wait();
    assert_eq!(paused.join().expect("join").expect("first client"), Accepted::AlreadyPresent);
    assert_eq!(std::fs::read_dir(&inbox).expect("inbox").flatten().filter(|entry| { return entry.file_name() != "tmp"; }).count(), 1);
    assert_eq!(std::fs::read_dir(inbox.join("tmp")).expect("tmp").count(), 0);
}

/// Review 6, 4.3: a read failure at the target is not absence. A directory in the place of `inbox/<id>` makes the first read fail with an
/// operational error; `accept` returns it without writing a temporary file, so a caller cannot read it as "not accepted" by mistake.
#[test]
fn a_read_failure_at_the_target_is_not_taken_for_absence() {
    let root: PathBuf = scratch("v-read-failure");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    std::fs::create_dir(lane.join("inbox").join(&request.id)).expect("a directory where the request file would be");
    let outcome = accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files);
    assert!(matches!(outcome, Err(git_policy_cli::commit_inbox::InboxError::Io(_))), "{:?}", outcome);
    assert_eq!(std::fs::read_dir(lane.join("inbox").join("tmp")).expect("tmp").count(), 0);
}

/// Review 6, 3.3: the barrier accepts bare relative names for the order and the ledger (their parent is the current directory, not the
/// empty path). Each test runs in its own process under nextest, so changing the directory here touches no other test.
#[test]
fn a_departure_accepts_bare_relative_order_and_ledger_names() {
    let root: PathBuf = scratch("v-relative");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    state_append(&lane.join("state"), 1, b"adopted").expect("append");
    std::fs::write(lane.join("order"), b"adopt line\n").expect("order");
    std::fs::write(lane.join("ledger"), b"ledger line\n").expect("ledger");
    std::env::set_current_dir(&lane).expect("enter the lane");
    depart_closure(&lane.join("inbox"), &[], Some(std::path::Path::new("order")), &lane.join("state"), Some(std::path::Path::new("ledger")))
        .expect("bare relative names");
}

/// Review 7, 4.3 and 3.4: when the link finds the name and the read of the final then fails, the request may be in place and its status is
/// unknown: `StatusUnknown`, never a plain error that reads as "not accepted". The pause at `accept:before-link` makes the directory
/// appear at the target after the early read and before the link.
#[test]
fn a_link_that_finds_the_name_and_cannot_read_it_reports_an_unknown_status() {
    static TARGET: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
    fn occupy(point: &str) {
        if point == "accept:before-link" {
            let target: PathBuf = TARGET.lock().expect("lock").clone().expect("target");
            std::fs::create_dir(&target).expect("occupy the name with a directory");
        }
    }
    let root: PathBuf = scratch("v-status-unknown");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    *TARGET.lock().expect("lock") = Some(lane.join("inbox").join(&request.id));
    set_fault_point(Some(occupy));
    let outcome = accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files);
    set_fault_point(None);
    assert!(matches!(outcome, Err(git_policy_cli::commit_inbox::InboxError::StatusUnknown(_))), "{:?}", outcome);
}

/// Review 7, 3.4: the early-error path writes no temporary file. The regression is observed by whether the pre-link point is reached
/// (the old path reached it), with a positive control that a fresh acceptance does reach it.
#[test]
fn an_early_read_failure_never_reaches_the_link_and_a_fresh_acceptance_does() {
    static REACHED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    fn count(point: &str) {
        if point == "accept:before-link" {
            REACHED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let root: PathBuf = scratch("v-early-failure");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    set_fault_point(Some(count));
    std::fs::create_dir(lane.join("inbox").join(&request.id)).expect("a directory at the target");
    assert!(accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).is_err());
    assert_eq!(REACHED.load(std::sync::atomic::Ordering::SeqCst), 0, "the early failure reached the link");
    std::fs::remove_dir(lane.join("inbox").join(&request.id)).expect("clear");
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("fresh");
    set_fault_point(None);
    assert_eq!(REACHED.load(std::sync::atomic::Ordering::SeqCst), 1, "the positive control: a fresh acceptance reaches the link");
}

/// Review 7, 4.4: a torn final is `DamagedFinal` (not `Conflict`), `quarantine_damaged` keeps its bytes aside, and the resend is then
/// accepted. A file that decodes is never moved.
#[test]
fn a_torn_final_is_quarantined_and_the_resend_is_accepted() {
    let root: PathBuf = scratch("v-torn-final");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let inbox: PathBuf = lane.join("inbox");
    let (request, new_files, old_files) = one_path_request();
    accept(&inbox, INSTANCE, &request, &new_files, &old_files).expect("accept");
    let whole: Vec<u8> = std::fs::read(inbox.join(&request.id)).expect("read");
    assert!(git_policy_cli::commit_inbox::quarantine_damaged(&inbox, &request.id).is_err(), "a decoding file is not moved");
    std::fs::write(inbox.join(&request.id), &whole[..whole.len() / 2]).expect("tear");
    assert!(matches!(accept(&inbox, INSTANCE, &request, &new_files, &old_files), Err(git_policy_cli::commit_inbox::InboxError::DamagedFinal)));
    let moved: PathBuf = git_policy_cli::commit_inbox::quarantine_damaged(&inbox, &request.id).expect("quarantine");
    assert_eq!(std::fs::read(&moved).expect("kept"), whole[..whole.len() / 2].to_vec());
    assert_eq!(accept(&inbox, INSTANCE, &request, &new_files, &old_files).expect("resend"), Accepted::Created);
}

/// Review 7, 2.5 and 3.2: a departure bound to a disposition forces the stop receipt first and alone, then the request, the saved content
/// the exit names, and (with an anchor) the chain above each parent.
#[test]
fn a_departure_forces_the_stop_first_and_alone_then_the_closure_and_the_ancestry() {
    let root: PathBuf = scratch("v-departure");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    state_append(&lane.join("state"), 1, b"adopted").expect("append");
    let decision: PathBuf = lane.join("decision");
    std::fs::create_dir(&decision).expect("decision directory");
    let mark: PathBuf = decision.join("mark");
    std::fs::write(&mark, b"").expect("the empty mark file");
    let saved: PathBuf = lane.join("saved-commit");
    std::fs::write(&saved, b"objects").expect("saved");
    start_events();
    git_policy_cli::commit_inbox::Departure {
        inbox: &lane.join("inbox"),
        ids: std::slice::from_ref(&request.id),
        order: None,
        state_dir: &lane.join("state"),
        ledger: None,
        stop: Some(&mark),
        closure: &[saved.as_path()],
        anchor: Some(&root),
    }
    .run()
    .expect("departure");
    let events: Vec<String> = take_events();
    assert_eq!(events[0], format!("file-sync:{}", mark.display()), "the stop receipt is forced first");
    assert_eq!(events[1], format!("dir:{}", decision.display()), "and alone with its own directory entry");
    let request_sync: usize = events.iter().position(|event| { return event.starts_with("file-sync:") && event.ends_with(&request.id); }).expect("request");
    let saved_sync: usize = events.iter().position(|event| { return event == &format!("file-sync:{}", saved.display()); }).expect("saved content");
    assert!(2 <= request_sync && request_sync < saved_sync, "{:?}", events);
    assert!(events.iter().any(|event| { return *event == format!("dir:{}", root.display()); }), "the ancestry reaches the anchor");
}

/// Components 8, 2.4 (P1): the helper `depart_closure` is a caller-assembled flush list. A withdrawal's mark sits in `decision/`; the
/// departure returns success and never forced it, so a power loss can remove the prohibition on later attempts.
#[test]
fn p1_a_departure_for_a_withdrawal_forces_the_mark() {
    let root: PathBuf = scratch("p1-departure-mark");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    state_append(&lane.join("state"), 1, b"adopted").expect("append");
    let mark: PathBuf = lane.join("decision").join(&request.id);
    std::fs::create_dir(lane.join("decision")).expect("decision");
    std::fs::write(&mark, b"").expect("the mark");
    start_events();
    git_policy_cli::commit_inbox::depart_closure(&lane.join("inbox"), std::slice::from_ref(&request.id), None, &lane.join("state"), None).expect("depart");
    let events: Vec<String> = take_events();
    assert!(events.contains(&format!("file-sync:{}", mark.display())), "the mark was not forced: {:?}", events);
}

/// Components 8, 2.5 (P1): quarantine is an unowned destructive mutation. Q1 inspects the torn final and pauses; the file is replaced by a
/// complete accepted request (here the pause writes it); Q1 resumes and renames the file at the name, which is not the object it inspected.
#[test]
fn p1_a_quarantine_never_moves_a_file_it_did_not_inspect() {
    static REPLACEMENT: std::sync::Mutex<Option<(PathBuf, Vec<u8>)>> = std::sync::Mutex::new(None);
    fn replace_the_final(point: &str) {
        if point == "inbox:after-inspect" {
            let (path, bytes) = REPLACEMENT.lock().expect("lock").clone().expect("replacement");
            std::fs::remove_file(&path).expect("the inspected object leaves");
            std::fs::write(&path, bytes).expect("a complete request takes the name");
        }
    }
    let root: PathBuf = scratch("p1-quarantine-race");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let inbox: PathBuf = lane.join("inbox");
    let (request, new_files, old_files) = one_path_request();
    accept(&inbox, INSTANCE, &request, &new_files, &old_files).expect("accept");
    let whole: Vec<u8> = std::fs::read(inbox.join(&request.id)).expect("read");
    std::fs::write(inbox.join(&request.id), &whole[..whole.len() / 2]).expect("tear");
    *REPLACEMENT.lock().expect("lock") = Some((inbox.join(&request.id), whole.clone()));
    set_fault_point(Some(replace_the_final));
    let outcome = git_policy_cli::commit_inbox::quarantine_damaged(&inbox, &request.id);
    set_fault_point(None);
    assert!(outcome.is_err(), "the accepted request was moved into damaged/");
    assert_eq!(std::fs::read(inbox.join(&request.id)).expect("the accepted request is still in place"), whole);
}
