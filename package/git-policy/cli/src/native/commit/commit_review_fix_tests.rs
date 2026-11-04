//! What:
//!  Controls for the findings of the second cross-vendor review of plan step 2 (parts A and B): the departure forces the closure it
//!  acknowledges, a lineage survives successive seals, a disposal cannot be undone by a seal, the receipt directory's own entry is forced,
//!  the manager's executor is confirmed, relative lanes are made absolute, the internal flush mode validates its lane and refuses special
//!  receipts, the startup sweep does not depend on the state log, unit names are unique, and the waits are strict bounds.
//! Why:
//!  Each is a schedule the review found in the code as built; each test is red against the code before the fix.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a seal between the retirement and the unlink does not re-adopt the item', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::lane_with_envelope;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content, retire_saved_content, unlink_retired};
use git_policy_cli::commit_flusher::{FLUSHER_FAILED_EXIT, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane, flush_lane_with, flusher_entry};
use git_policy_cli::commit_flusher_unit::{TransientUnitRoute, fresh_nonce, transient_unit_arguments, validate_lane};
use git_policy_cli::commit_identity::{inject_sync_error, start_events, take_events};
use git_policy_cli::commit_lane_lifecycle::{capture_lane_closure, client_after_accept, departure_acknowledge, seal_for_shutdown, startup_sweep};
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, create_saved_content, select, state_covers, sync_prefix};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A lane with an envelope, a saved content, a mark, an order file and a ledger, all retained by a first checked record. Returns the lane,
/// the state directory and the saved content's name.
fn lane_with_everything(label: &str) -> (PathBuf, PathBuf, String) {
    let (lane, state, envelope) = lane_with_envelope(label);
    let id: String = envelope.file_name().expect("name").to_string_lossy().to_string();
    fs::create_dir_all(lane.join("decision")).expect("decision directory");
    fs::write(lane.join("decision").join(format!("{}.withdraw", id)), b"").expect("a mark");
    fs::write(lane.join("order"), format!("adopt {} 0badc0de\n", id)).expect("an order file");
    fs::write(lane.join("ledger"), b"").expect("an empty ledger");
    let saved: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("saved content");
    let name: String = saved.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    return (lane, state, name);
}

/// P1: the departure forces what the acknowledged record retains (the order file, the ledger, the request, the saved content, the mark),
/// not only the log and its directory; a retained file that is lost in a power loss would make the durable record ineligible.
#[test]
fn a_departure_forces_the_closure_of_the_record_it_acknowledges() {
    let (lane, state, _name) = lane_with_everything("fix-closure");
    start_events();
    departure_acknowledge(&state, 3).expect("the acknowledgment");
    let events: Vec<String> = take_events();
    for retained in ["order", "ledger"] {
        let wanted: String = format!("file-sync:{}", lane.join(retained).display());
        assert!(events.contains(&wanted), "{} was not forced: {:?}", retained, events);
    }
    assert!(events.iter().any(|event| { return event.starts_with("file-sync:") && event.contains("/inbox/"); }), "the request was not forced: {:?}", events);
    assert!(events.iter().any(|event| { return event.starts_with("file-sync:") && event.contains("/saved/"); }), "the saved content was not forced: {:?}", events);
    assert!(events.iter().any(|event| { return event.starts_with("file-sync:") && event.contains("/decision/"); }), "the mark was not forced: {:?}", events);
}

/// P1: a forcing error in the closure is the departure's error too: no success is returned for a record whose dependency could not be forced.
#[test]
fn a_departure_whose_closure_cannot_be_forced_does_not_succeed() {
    let (_lane, state, _name) = lane_with_everything("fix-closure-error");
    inject_sync_error(Some((2, libc::EIO)));
    let failed = departure_acknowledge(&state, 3);
    inject_sync_error(None);
    assert_eq!(failed.expect_err("a success after a failed forcing of a retained file").raw_os_error(), Some(libc::EIO));
}

/// P1 (lineage): after two successive seals the acknowledgment of the first record is still covered: a checkpoint carries the lineage of
/// the authorities it replaces.
#[test]
fn an_acknowledgment_survives_two_successive_seals() {
    let (lane, state, _name) = lane_with_everything("fix-lineage");
    let promised = sync_prefix(&state).expect("the acknowledgment of generation 1");
    seal_for_shutdown(&lane, &state).expect("the first seal");
    seal_for_shutdown(&lane, &state).expect("the second seal");
    assert!(state_covers(&state, &promised).expect("readable"), "the first record's acknowledgment lost its coverage through two seals");
}

/// P1/part B 4: retirement, then a seal, then the unlink. The seal must not re-adopt the item whose `disposed` line is in the ledger; the
/// unlink then leaves an eligible root that does not name it.
#[test]
fn a_seal_between_the_retirement_and_the_unlink_does_not_re_adopt_the_item() {
    let (lane, state, name) = lane_with_everything("fix-readopt");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    seal_for_shutdown(&lane, &state).expect("the seal in the window");
    let root = select(&state).expect("readable").expect("a root");
    let document: LaneDocument = LaneDocument::parse(&root.payload).expect("checked");
    assert!(!document.saved().contains(&name), "the seal re-adopted the retired item");
    unlink_retired(retired).expect("the unlink");
    let after = select(&state).expect("readable").expect("an eligible root after the unlink");
    assert!(!LaneDocument::parse(&after.payload).expect("checked").saved().contains(&name));
}

/// P1/part B 4: a token is checked against the current root before the unlink, so a stale token cannot delete content a root names again,
/// and it is bound to its lane and consumed by use.
#[test]
fn a_stale_retirement_token_does_not_delete_content_the_root_names_again() {
    let (lane, state, name) = lane_with_everything("fix-stale");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    // Something outside the procedure makes a root name the item again.
    let saved: ClosureEntry = ClosureEntry::capture_sealed_saved(&lane, &lane.join("saved").join(&name)).expect("the item is still on disk");
    let mut closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    closure.push(saved);
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    let generation: u64 = select(&state).expect("readable").expect("root").generation + 1;
    StateWriter::acquire(&state).expect("writer").checkpoint(generation, &document.encode(), &closure).expect("a root that names the item again");
    assert!(unlink_retired(retired).is_err(), "the stale token deleted content the current root names");
    assert!(lane.join("saved").join(&name).exists());
}

/// P1: `dispose_saved_content` leaves an eligible root whether or not a seal runs in its window; the seal after it keeps the root eligible.
#[test]
fn a_seal_after_the_disposal_keeps_an_eligible_root() {
    let (lane, state, name) = lane_with_everything("fix-after");
    dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the disposal");
    seal_for_shutdown(&lane, &state).expect("the seal");
    assert!(select(&state).expect("readable").is_some());
    assert!(!lane.join("saved").join(&name).exists());
}

/// P1: a new receipt directory's own entry in the lane is forced with the receipts: the flusher forces the lane directory as well.
#[test]
fn the_flusher_forces_the_lane_directory_entry_of_the_receipt_directory() {
    let lane: PathBuf = scratch("fix-receipt-ancestry");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    let forced: std::cell::RefCell<Vec<PathBuf>> = std::cell::RefCell::new(Vec::new());
    flush_lane_with(&lane, &mut |path: &Path| { forced.borrow_mut().push(path.to_path_buf()); return Ok(()); }, &mut || {}).expect("flush");
    assert!(forced.borrow().contains(&lane), "the lane directory was not forced: {:?}", forced.borrow());
}

/// P1: the transient unit is confirmed at execution (`Type=exec`), so a missing executable is a refusal and the caller falls back.
#[test]
fn the_unit_waits_for_the_execution_and_a_missing_program_is_a_refusal() {
    let arguments: Vec<String> = transient_unit_arguments("unit", Path::new("/x/service"), Path::new("/lane")).iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
    assert!(arguments.contains(&"--property=Type=exec".to_string()), "{:?}", arguments);
    let probe = git_policy_cli::commit_flusher_unit::run_within(
        {
            let mut command = std::process::Command::new("systemd-run");
            command.args(["--user", "--collect", "--quiet", "--wait", "--", "/bin/true"]);
            command
        },
        Duration::from_secs(10),
    );
    if probe.is_err() {
        eprintln!("SKIPPED: no user manager answers systemd-run --user here");
        return;
    }
    let lane: PathBuf = scratch("fix-missing-program");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let mut route: TransientUnitRoute = TransientUnitRoute::new(PathBuf::from("systemd-run"), PathBuf::from("/nonexistent/cli-git-service"));
    assert!(git_policy_cli::commit_flusher_unit::ManagerRoute::start(&mut route, &lane).is_err(), "a unit whose executable is missing was reported as started");
}

/// P1: the flusher entry that cannot take the lock records every receipt as not forced, as the fallback helper does, so the next sweep is told.
#[test]
fn a_flusher_entry_that_cannot_take_the_lock_records_the_receipts() {
    let lane: PathBuf = scratch("fix-entry-lock");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    git_policy_cli::commit_flusher::inject_lock_failure_after(Some(0));
    let code: i32 = flusher_entry(&lane);
    git_policy_cli::commit_flusher::inject_lock_failure_after(None);
    assert_eq!(code, FLUSHER_FAILED_EXIT);
    assert!(fs::read_to_string(lane.join(RECEIPT_ERRORS)).unwrap_or_default().contains("r1"), "the receipt was not recorded");
}

/// P2: a relative lane is made absolute before the receipt is created and the unit launched.
#[test]
fn a_relative_lane_is_made_absolute_before_the_obligation_is_created() {
    struct Spy(Vec<PathBuf>);
    impl git_policy_cli::commit_flusher_unit::ManagerRoute for Spy {
        fn start(&mut self, lane: &Path) -> Result<String, git_policy_cli::commit_flusher_unit::ManagerRefused> {
            self.0.push(lane.to_path_buf());
            return Ok("unit".to_string());
        }
    }
    let base: PathBuf = scratch("fix-relative");
    let child_base: PathBuf = base.clone();
    let code: Option<i32> = in_single_threaded_process(move || {
        std::env::set_current_dir(&child_base).expect("the working directory of the client");
        let mut spy: Spy = Spy(Vec::new());
        let step = client_after_accept(Path::new("lane-relative"), "0123456789abcdef0123456789abcdef", &mut |_lane: &Path| { return Ok(false); }, &mut spy);
        let absolute: bool = spy.0.iter().all(|lane| { return lane.is_absolute(); }) && !spy.0.is_empty();
        let receipt_where_the_unit_will_look: bool = spy.0.first().is_some_and(|lane| { return lane.join(RECEIPT_DIR).join("0123456789abcdef0123456789abcdef").exists(); });
        return if step.is_ok() && absolute && receipt_where_the_unit_will_look { 0 } else { 1 };
    });
    assert_eq!(code, Some(0));
}

/// P2: the startup sweep serves the receipts even when the state log is damaged, and reports the state error.
#[test]
fn a_damaged_state_log_does_not_stop_the_startup_sweep_from_serving_receipts() {
    let (lane, state, _envelope) = lane_with_envelope("fix-startup-damaged");
    fs::write(state.join("state.log.1"), b"rec 1 garbage that is not a record\n").expect("a damaged log");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("dangling")).expect("a receipt that cannot be forced");
    fs::write(lane.join(RECEIPT_DIR).join("good"), b"receipt").expect("a receipt");
    let recovery = startup_sweep(&lane, &state);
    // The dangling receipt is a forcing error; the sweep still attempted the lane and recorded it, and the state error is reported.
    let record: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).unwrap_or_default();
    assert!(record.contains("dangling"), "the sweep did not run: {}", record);
    assert!(recovery.is_err() || recovery.as_ref().is_ok_and(|found| { return found.state_error.is_some(); }), "the damaged state log was not reported");
}

/// P2: unit names do not collide: a thousand fresh nonces are distinct.
#[test]
fn fresh_nonces_are_distinct() {
    let nonces: BTreeSet<u64> = (0..1000).map(|_| { return fresh_nonce(); }).collect();
    assert_eq!(nonces.len(), 1000);
}

/// P2: the internal flush mode refuses a directory that is not a lane, and a symlink for a lane.
#[test]
fn the_internal_flush_mode_validates_its_lane() {
    let base: PathBuf = scratch("fix-validate-lane");
    assert!(validate_lane(Path::new("relative-lane")).is_err(), "a relative path");
    assert!(validate_lane(&base.join("missing")).is_err(), "a missing directory");
    fs::write(base.join("file"), b"x").expect("a file");
    assert!(validate_lane(&base.join("file")).is_err(), "a file");
    let plain: PathBuf = base.join("plain");
    fs::create_dir_all(&plain).expect("a directory that is not a lane");
    assert!(validate_lane(&plain).is_err(), "a directory without a receipt directory");
    let lane: PathBuf = base.join("lane");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("a lane");
    assert!(validate_lane(&lane).is_ok());
    std::os::unix::fs::symlink(&lane, base.join("link")).expect("a symlink to a lane");
    assert!(validate_lane(&base.join("link")).is_err(), "a symlink for the lane");
    assert!(!plain.join("flusher.lock").exists(), "validation created nothing");
}

/// P2: a FIFO named like a receipt does not block the flusher: it is refused as a forcing error and recorded.
#[test]
fn a_fifo_receipt_does_not_block_the_flusher() {
    let lane: PathBuf = scratch("fix-fifo");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let fifo: std::ffi::CString = std::ffi::CString::new(lane.join(RECEIPT_DIR).join("fifo").as_os_str().as_encoded_bytes()).expect("a path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no other memory.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let child_lane: PathBuf = lane.clone();
    let code: Option<i32> = in_single_threaded_process(move || { return if flush_lane(&child_lane).is_err() { 0 } else { 1 }; });
    assert_eq!(code, Some(0), "the flusher hung on a FIFO (None) or accepted it (1)");
    assert!(fs::read_to_string(lane.join(RECEIPT_ERRORS)).unwrap_or_default().contains("fifo"));
}

/// The pause of the drain test, in seconds: long enough to show a missing clamp within the test's own limit, short enough to end it.
const PAUSE_PAST_THE_BOUND: u64 = 5;

/// P2: the drain wait and the timed command return at their bounds: a pause of an hour does not hold a bound of milliseconds, and the
/// upper limit is a small multiple of the bound.
#[test]
fn a_pause_longer_than_the_bound_does_not_extend_the_wait() {
    use git_policy_cli::commit_drain::{DrainAnswer, wait_for_drain};
    let bound: Duration = Duration::from_millis(30);
    let started: Instant = Instant::now();
    let answer: DrainAnswer = wait_for_drain(bound, Duration::from_secs(PAUSE_PAST_THE_BOUND), &mut || { return false; });
    assert_eq!(answer, DrainAnswer::InProgress);
    assert!(started.elapsed() < bound * 20, "the caller slept past its bound: {:?}", started.elapsed());
}
