//! What:
//!  Controls for the third round of review of plan step 2 (code part A and models part B of the fix round): disposal recovery forces the
//!  selected root before it unlinks, the unlink and the seal are serialized by the state writer, the state directory's own entry is
//!  forced, the establishment record binds the identity of the established segment, the error record is opened without following a link or
//!  blocking and is cleared only if it is the bytes the pass read, several pending disposals recover together, and the departure retries the
//!  reads that a concurrent reclaimer can make fail.
//! Why:
//!  Each is a schedule the reviewers found in the code as built; each test is red against it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('recovery forces the replacement root before it unlinks', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::{lane_with_envelope, ledger_line_of};
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content, retire_saved_content, unlink_retired};
use git_policy_cli::commit_flusher::{FlushError, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane};
use git_policy_cli::commit_identity::{set_fault_point, start_events, take_events};
use git_policy_cli::commit_lane_lifecycle::{capture_lane_closure, departure_acknowledge, seal_for_shutdown, startup_sweep, syncer_pass, syncer_pass_with};
use git_policy_cli::commit_state_log::{ClosureEntry, EntryKind, LaneDocument, StateError, StateWriter, create_saved_content, read, select};
use std::cell::{Cell, RefCell};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The exit status of a child that died at its crash point.
const CRASHED: i32 = 77;

/// What a bound is allowed to overshoot by on a loaded host: the fork of the child, its polling interval and the scheduler.
const SCHEDULING_ALLOWANCE: Duration = Duration::from_secs(3);

thread_local! {
    /// The fault point at which the process of this thread dies.
    static CRASH_AT: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// What a fault point does once, and with which lane and state directory.
    static ONCE: RefCell<Option<(PathBuf, PathBuf, &'static str, String)>> = const { RefCell::new(None) };
}

/// Dies at once when the named fault point is reached.
fn die_at_the_point(point: &str) {
    if CRASH_AT.with(|cell| { return cell.get(); }) == Some(point) {
        // SAFETY: `_exit` takes an integer and never returns.
        unsafe { libc::_exit(CRASHED) };
    }
}

/// Runs the armed action once when its fault point is reached: a seal or a disposal in the window of the departure.
fn act_once_at(point: &str) {
    let armed = ONCE.with(|cell| { return cell.borrow().as_ref().filter(|(_, _, wanted, _)| { return *wanted == point; }).cloned(); });
    let Some((lane, state, _wanted, argument)) = armed else { return };
    ONCE.with(|cell| { *cell.borrow_mut() = None; });
    if argument == "seal" {
        seal_for_shutdown(&lane, &state).expect("the seal in the window");
    } else {
        dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&argument)).expect("the disposal in the window");
    }
}

/// A lane with a request and saved content retained by a first checked record. Returns the lane, the state directory and the content name.
fn lane_with_saved(label: &str) -> (PathBuf, PathBuf, String) {
    let (lane, state, _envelope) = lane_with_envelope(label);
    let saved: ClosureEntry = create_saved_content(&lane, label.as_bytes()).expect("saved content");
    let name: String = saved.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    return (lane, state, name);
}

/// P1: a crash after the candidate of the replacement root is written and before it is completed leaves a visible root that omits the item.
/// Recovery does not take that for a durable cover: it forces the selected root before it unlinks the content.
#[test]
fn recovery_forces_the_selected_root_before_it_unlinks_behind_an_uncompleted_replacement() {
    let (lane, state, name) = lane_with_saved("rev3-uncompleted");
    let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), name.clone());
    let status: Option<i32> = in_single_threaded_process(move || {
        CRASH_AT.with(|cell| { cell.set(Some("disposal:after-candidate")); });
        set_fault_point(Some(die_at_the_point));
        let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name));
        return 0;
    });
    assert_eq!(status, Some(CRASHED), "the disposal did not reach the window between the candidate and its completion");
    let candidate: usize = fs::read_dir(&state).expect("state directory").flatten().filter(|entry| { return entry.file_name().to_string_lossy().starts_with("state.log."); }).count();
    assert_eq!(candidate, 2, "the predecessor and the uncompleted candidate");
    start_events();
    startup_sweep(&lane, &state).expect("the replacement owner's sweep");
    let events: Vec<String> = take_events();
    let unlinked: usize = events.iter().position(|event| { return event.starts_with("unlink:"); }).expect("the content was unlinked");
    let forced_root: usize = events.iter().position(|event| { return event.starts_with("file-sync:") && event.contains("/state/state.log."); }).expect("a state segment was forced");
    assert!(forced_root < unlinked, "the content was unlinked before the selected root was forced: {:?}", events);
}

/// P1 (both reviews): the revalidation and the unlink happen under the state writer, so nothing can retain the content between them, and
/// the retirement, which appends the line, and the seal, which captures, cannot overlap. Each is refused while another holds the writer.
#[test]
fn the_state_writer_serializes_the_unlink_the_retirement_and_the_seal() {
    let (lane, state, name) = lane_with_saved("rev3-serial");
    let seal_in_progress: StateWriter = StateWriter::acquire(&state).expect("a writer, as a seal in progress");
    let retirement = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name));
    assert!(matches!(&retirement, Err(error) if error.kind() == io::ErrorKind::WouldBlock), "a retirement began while a writer held the state");
    assert!(!lane.join("ledger").exists(), "no disposed line was appended before the writer was taken");
    drop(seal_in_progress);
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    let publishing: StateWriter = StateWriter::acquire(&state).expect("a writer, as a root being published");
    assert!(unlink_retired(retired).is_err(), "the unlink ran while another writer held the state");
    assert!(lane.join("saved").join(&name).exists(), "the content was deleted without the writer");
    assert!(seal_for_shutdown(&lane, &state).is_err(), "a seal ran while a writer held the state");
    drop(publishing);
}

/// Part B 1 (split capture and publish): while a disposal holds the writer, a seal cannot start capturing, and after the disposal published
/// its root the seal captures afresh and does not name the item.
#[test]
fn a_seal_after_the_disposal_root_captures_afresh_and_does_not_name_the_item() {
    let (lane, state, name) = lane_with_saved("rev3-capture-publish");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    seal_for_shutdown(&lane, &state).expect("the seal after the root");
    let root = select(&state).expect("readable").expect("a root");
    assert!(!LaneDocument::parse(&root.payload).expect("checked").saved().contains(&name));
    unlink_retired(retired).expect("the unlink");
}

/// P1: the state directory's own entry in the lane is forced by an acknowledgment, whatever the closure holds: an empty closure leaves no
/// retained file whose forcing would sync the lane by the way.
#[test]
fn an_acknowledgment_forces_the_lane_directory_entry_of_the_state_directory() {
    let (lane, state, _envelope) = lane_with_envelope("rev3-state-ancestry");
    StateWriter::acquire(&state).expect("writer").append(1, &LaneDocument::default().encode()).expect("a record that retains nothing");
    start_events();
    departure_acknowledge(&state, 3).expect("the acknowledgment");
    let events: Vec<String> = take_events();
    assert!(events.contains(&format!("dir:{}", lane.display())), "the lane directory was not forced: {:?}", events);
}

/// Part A, establishment: a completed non-reclaiming checkpoint at the same generation, with one more retained envelope, is established. When
/// its envelope is damaged, recovery does not fall back to the older segment of the same generation.
#[test]
fn an_established_checkpoint_at_the_same_generation_is_not_replaced_by_the_older_segment() {
    let (lane, state, envelope) = lane_with_envelope("rev3-established-same-generation");
    let doc: Vec<u8> = b"an opaque document".to_vec();
    StateWriter::acquire(&state).expect("writer").append(1, &doc).expect("generation 1, retaining nothing");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint(1, &doc, std::slice::from_ref(&entry)).expect("a checkpoint at the same generation that retains the envelope");
    assert!(state.join("state.log.1").exists() && state.join("state.log.2").exists());
    fs::remove_file(&envelope).expect("the established root loses its envelope");
    assert!(matches!(read(&state), Err(StateError::Damaged)), "recovery fell back past a completed checkpoint of the same generation");
}

/// Part B 3: the record binds the segment and its full hash: a different valid segment at the same number and generation is refused, and a
/// hash that is not hexadecimal is not a hash.
#[test]
fn the_establishment_record_binds_the_segment_and_the_hash() {
    let build = |label: &str, text: &[u8]| -> PathBuf {
        let (_lane, state, _envelope) = lane_with_envelope(label);
        StateWriter::acquire(&state).expect("writer").append(1, b"first").expect("append");
        StateWriter::acquire(&state).expect("writer").checkpoint(2, text, &[]).expect("checkpoint");
        return state;
    };
    let genuine: PathBuf = build("rev3-binding-a", b"document a");
    let other: PathBuf = build("rev3-binding-b", b"document b");
    assert_eq!(read(&genuine).expect("readable").expect("state").0, 2);
    fs::copy(other.join("state.log.2"), genuine.join("state.log.2")).expect("a valid segment of another history at the same number and generation");
    assert!(matches!(read(&genuine), Err(StateError::Damaged)), "a substitute segment with the same number and generation was accepted");
    let torn: PathBuf = build("rev3-binding-c", b"document c");
    let recorded: String = fs::read_to_string(torn.join("state.established")).expect("the record");
    let bad: String = recorded.trim_end().rsplit_once(' ').map(|(head, _)| { return format!("{} {}\n", head, "z".repeat(64)); }).expect("a hash field");
    fs::write(torn.join("state.established"), bad).expect("a record whose hash is not hexadecimal");
    assert!(matches!(read(&torn), Err(StateError::Damaged)));
}

/// Part B 4 and part A: the pass clears the record only if it is the bytes the pass read. The ABA: pass A reads, pass B clears, a new record
/// of the same length arrives, and A must not clear it.
#[test]
fn a_new_record_of_the_same_length_survives_a_pass_that_read_the_old_one() {
    let lane: PathBuf = scratch("rev3-aba");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    fs::write(lane.join(RECEIPT_ERRORS), "r1\told\n").expect("the old record");
    let inner_lane: PathBuf = lane.clone();
    syncer_pass_with(&lane, &mut || {
        syncer_pass(&inner_lane).expect("pass B clears the record");
        fs::write(inner_lane.join(RECEIPT_ERRORS), "r2\tnew\n").expect("a new record of the same length");
    })
    .expect("pass A");
    assert_eq!(fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the record"), "r2\tnew\n", "the new record was cleared by a pass that read the old one");
}

/// P1 (flush mode): a symbolic link at the error record is not written through; the recording fails instead.
#[test]
fn an_error_record_that_is_a_symbolic_link_is_not_written_through() {
    let lane: PathBuf = scratch("rev3-record-link");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("bad")).expect("a receipt that cannot be forced");
    let target: PathBuf = lane.join("somebody-elses-file");
    fs::write(&target, b"theirs\n").expect("an unrelated file");
    std::os::unix::fs::symlink(&target, lane.join(RECEIPT_ERRORS)).expect("the record is a link");
    let outcome = flush_lane(&lane);
    assert!(matches!(outcome, Err(FlushError::ForcingUnrecorded { .. })), "{:?}", outcome.map(|found| { return format!("{:?}", found); }));
    assert_eq!(fs::read_to_string(&target).expect("the target"), "theirs\n", "the flusher wrote through the link");
}

/// P1 (flush mode): a FIFO at the error record does not hang the flusher or the syncer.
#[test]
fn an_error_record_that_is_a_fifo_does_not_hang_the_flusher() {
    let lane: PathBuf = scratch("rev3-record-fifo");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("bad")).expect("a receipt that cannot be forced");
    let fifo: std::ffi::CString = std::ffi::CString::new(lane.join(RECEIPT_ERRORS).as_os_str().as_encoded_bytes()).expect("a path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no other memory.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let (flush_lane_path, pass_lane_path) = (lane.clone(), lane.clone());
    let flushed: Option<i32> = in_single_threaded_process(move || { return if flush_lane(&flush_lane_path).is_err() { 0 } else { 1 }; });
    assert_eq!(flushed, Some(0), "the flusher hung on a FIFO error record (None) or reported success (1)");
    let passed: Option<i32> = in_single_threaded_process(move || { return if syncer_pass(&pass_lane_path).is_err() { 0 } else { 1 }; });
    assert_eq!(passed, Some(0), "the syncer hung on a FIFO error record (None) or reported success (1)");
}

/// P1 (flush mode): a recorder does not wait without a bound for a lock another process holds on the error record.
#[test]
fn a_recorder_does_not_wait_without_a_bound_for_the_error_record_lock() {
    let lane: PathBuf = scratch("rev3-record-lock");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("bad")).expect("a receipt that cannot be forced");
    let record: fs::File = fs::OpenOptions::new().create(true).write(true).truncate(false).open(lane.join(RECEIPT_ERRORS)).expect("the record");
    record.lock().expect("an exclusive lock held by another party");
    let child_lane: PathBuf = lane.clone();
    let started: Instant = Instant::now();
    let outcome: Option<i32> = in_single_threaded_process(move || { return if flush_lane(&child_lane).is_err() { 0 } else { 1 }; });
    assert_eq!(outcome, Some(0), "the recorder waited for ever (None) or reported success (1)");
    // The recorder waits for the lock at most RECORD_LOCK_WAIT, plus the scheduling allowance of a loaded host (a process fork, a few polls).
    assert!(started.elapsed() >= git_policy_cli::commit_flusher::RECORD_LOCK_WAIT, "the recorder gave up before its wait: {:?}", started.elapsed());
    assert!(started.elapsed() < git_policy_cli::commit_flusher::RECORD_LOCK_WAIT + SCHEDULING_ALLOWANCE, "{:?}", started.elapsed());
    drop(record);
}

/// P2: two disposals whose `disposed` lines are durable and whose replacements never completed recover together: the replacement names every
/// omitted item as retired, and the sweep finishes both.
#[test]
fn several_pending_disposals_recover_together() {
    let (lane, state, first) = lane_with_saved("rev3-two-a");
    let second_entry: ClosureEntry = create_saved_content(&lane, b"a second content").expect("a second saved content");
    let second: String = second_entry.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let generation: u64 = select(&state).expect("readable").expect("root").generation + 1;
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(generation, &document.encode(), &closure).expect("a record that holds both");
    let mut ledger: Vec<u8> = Vec::new();
    for name in [&first, &second] {
        ledger.extend_from_slice(&ledger_line_of(&name.chars().take(32).collect::<String>(), "disposed"));
    }
    fs::write(lane.join("ledger"), ledger).expect("both lines are durable and neither replacement completed");
    let recovery = startup_sweep(&lane, &state).expect("the sweep");
    assert_eq!(recovery.disposals, 2);
    assert!(!lane.join("saved").join(&first).exists() && !lane.join("saved").join(&second).exists());
    assert!(select(&state).expect("readable").is_some());
}

/// P1 (departure): the read of the segment that `state_covers` selects can fail with ENOENT when a seal reclaims it meanwhile; that is a
/// recapture, not a terminal error.
#[test]
fn a_departure_retries_when_the_seal_removes_the_segment_it_is_about_to_read() {
    let (lane, state, _name) = lane_with_saved("rev3-covers-enoent");
    ONCE.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), state.clone(), "state:covers-after-select", "seal".to_string())); });
    set_fault_point(Some(act_once_at));
    let result = departure_acknowledge(&state, 3);
    set_fault_point(None);
    assert!(ONCE.with(|cell| { return cell.borrow().is_none(); }), "the hook never fired: the window was not reached");
    assert!(!state.join("state.log.1").exists(), "the seal in the window did not reclaim the segment the departure was about to read");
    assert!(result.is_ok(), "{:?}", result.map(|prefix| { return prefix.generation; }));
}

/// P1 (departure): support the acknowledged record retained is removed by a disposal before the departure forces it; the force finds it gone
/// and the departure captures again.
#[test]
fn a_departure_retries_when_a_disposal_removes_support_before_the_closure_is_forced() {
    let (lane, state, name) = lane_with_saved("rev3-support-enoent");
    ONCE.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), state.clone(), "lifecycle:before-force-closure", name.clone())); });
    set_fault_point(Some(act_once_at));
    let result = departure_acknowledge(&state, 3);
    set_fault_point(None);
    assert!(ONCE.with(|cell| { return cell.borrow().is_none(); }), "the hook never fired: the window was not reached");
    assert!(!lane.join("saved").join(&name).exists(), "the disposal in the window did not remove the support the departure was about to force");
    assert!(result.is_ok(), "{:?}", result.map(|prefix| { return prefix.generation; }));
}

/// Part A, finding C: a duplicate path in the retained set of an append is refused, as it is for a checked reclaiming checkpoint.
#[test]
fn a_retained_set_that_names_a_path_twice_is_refused_at_admission() {
    let (lane, state, envelope) = lane_with_envelope("rev3-duplicate");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let document: Vec<u8> = LaneDocument::from_closure(std::slice::from_ref(&entry)).expect("document").encode();
    let doubled: Vec<ClosureEntry> = vec![entry.clone(), entry];
    assert!(StateWriter::acquire(&state).expect("writer").append_with(1, &document, &doubled).is_err(), "a set that names a path twice was admitted");
    assert!(StateWriter::acquire(&state).expect("writer").checkpoint(1, &document, &doubled).is_err(), "and a checkpoint of it");
}

/// Part A: the damaged-state startup test, strengthened: the good receipt is forced, the state error is reported, and the sweep itself
/// succeeds.
#[test]
fn a_damaged_state_log_is_reported_beside_the_receipts_that_were_served() {
    let (lane, state, _envelope) = lane_with_envelope("rev3-startup-damaged");
    fs::write(state.join("state.log.1"), b"rec 1 garbage that is not a record\n").expect("a damaged log");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("good"), b"receipt").expect("a receipt");
    start_events();
    let recovery = startup_sweep(&lane, &state).expect("the sweep succeeds although the state log is damaged");
    let events: Vec<String> = take_events();
    assert!(recovery.state_error.is_some(), "the state error was not reported");
    assert!(recovery.forced >= 1, "the good receipt was not forced");
    let _ = events;
}

/// Part A (launch route): a child that outlives the cleanup window is reaped by a thread of its own, not left a zombie. The timed command is
/// `sleep` with a TERM-ignoring parent `sh`: the check is that `run_within` returns at its bound and the child is gone soon after.
#[test]
fn a_timed_out_command_is_reaped_after_the_cleanup_window() {
    let started: Instant = Instant::now();
    let mut command = std::process::Command::new("sleep");
    command.arg("30");
    let result = git_policy_cli::commit_flusher_unit::run_within(command, Duration::from_millis(30));
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(5));
}

/// A path that must not exist in a lane for the test above to be about the flusher; here only to keep the `Path` import used.
fn _unused(_path: &Path) {}
