//! What:
//!  Controls for the fourth round of review of plan step 2 and step 3 (component tests, code against whole files, models and map): the
//!  disposal revalidation against the whole retained set, the bounded doorbell, and the false-green windows of the earlier tests, each with
//!  its guard removed in the proof of the round.
//! Why:
//!  A retained set may be larger than the document's references (the public append admits an over-approximation), and recovery's eligibility
//!  depends on every retained entry; the doorbell is on the client's path and may not wait for ever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a newer record that retains the item only in its closure refuses the stale unlink', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::{lane_with_envelope, raw_record};
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content, retire_saved_content, unlink_retired};
use git_policy_cli::commit_flusher::{FlushError, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane, open_error_record};
use git_policy_cli::commit_flusher_unit::{inject_pending_child, inject_unreapable_child, pending_reaps, reap_pending, run_within, validate_lane};
use git_policy_cli::commit_identity::{inject_sync_error, set_fault_point, start_events, take_events};
use git_policy_cli::commit_lane_lifecycle::{capture_lane_closure, seal_for_shutdown, startup_sweep, syncer_pass, syncer_pass_with};
use git_policy_cli::commit_lane_service::{bind_ring, ring_running_service, start_lane_service};
use git_policy_cli::commit_state_log::{ClosureEntry, EntryKind, LaneDocument, StateError, StateWriter, create_saved_content, read, select};
use std::cell::{Cell, RefCell};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The exit status of a child that died at its crash point.
const CRASHED: i32 = 77;

thread_local! {
    /// The fault point at which the process of this thread dies.
    static CRASH_AT: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// Where a hook runs, with the lane and the state directory it works on.
    static HOOK: RefCell<Option<(&'static str, PathBuf, PathBuf, String)>> = const { RefCell::new(None) };
    /// What the hooks saw.
    static SEEN: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Dies at once when the named fault point is reached.
fn die_at_the_point(point: &str) {
    if CRASH_AT.with(|cell| { return cell.get(); }) == Some(point) {
        // SAFETY: `_exit` takes an integer and never returns.
        unsafe { libc::_exit(CRASHED) };
    }
}

/// Runs, once, the hook armed for the fault point reached; it records what it saw in `SEEN`.
fn hook(point: &str) {
    let armed = HOOK.with(|cell| { return cell.borrow().as_ref().filter(|(wanted, _, _, _)| { return *wanted == point; }).cloned(); });
    let Some((_, lane, state, argument)) = armed else { return };
    HOOK.with(|cell| { *cell.borrow_mut() = None; });
    let note = |text: String| { SEEN.with(|seen| { seen.borrow_mut().push(text); }); };
    match argument.as_str() {
        "retire-during-seal" => {
            let name: String = fs::read_dir(lane.join("saved")).expect("saved").flatten().next().expect("an item").file_name().to_string_lossy().to_string();
            let outcome = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name));
            note(format!("retire:{}", outcome.err().map_or("ok".to_string(), |error| { return format!("{:?}", error.kind()); })));
            note(format!("ledger-exists:{}", lane.join("ledger").exists()));
        }
        "writer-and-seal-during-unlink" => {
            let writer = StateWriter::acquire(&state);
            note(format!("writer:{}", writer.err().map_or("acquired".to_string(), |error| { return format!("{:?}", error.kind()); })));
            let seal = seal_for_shutdown(&lane, &state);
            note(format!("seal:{}", if seal.is_ok() { "ok" } else { "refused" }));
        }
        other => note(format!("unknown hook {}", other)),
    }
}

/// What the hooks saw, and clears it.
fn seen() -> Vec<String> {
    return SEEN.with(|seen| { return std::mem::take(&mut *seen.borrow_mut()); });
}

/// The longest a test waits for something to happen.
const WAIT: Duration = Duration::from_secs(10);


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

/// Review of step 3 (code, 1): a newer record whose document does not list the retired item but whose retained set does is legal (the
/// retained set may over-approximate the document). The stale token must be refused against the retained set, not the document alone,
/// or the unlink makes that newer record ineligible and recovery falls back, discarding its state.
#[test]
fn a_newer_record_that_retains_the_item_only_in_its_closure_refuses_the_stale_unlink() {
    let (lane, state, name) = lane_with_saved("rev4-closure-only");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    let saved: ClosureEntry = ClosureEntry::capture_sealed_saved(&lane, &lane.join("saved").join(&name)).expect("the item is still on disk");
    let mut items: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&items).expect("a document that does not list the item");
    items.push(saved);
    let generation: u64 = select(&state).expect("readable").expect("root").generation + 1;
    StateWriter::acquire(&state).expect("writer").append_with(generation, &document.encode(), &items).expect("an over-approximating record is admitted");
    let refused = unlink_retired(retired);
    assert!(refused.is_err(), "the stale token deleted content that the newest record retains in its closure");
    assert!(lane.join("saved").join(&name).exists());
    assert_eq!(select(&state).expect("readable").expect("a root").generation, generation, "the newest record is still the root");
}

/// The queue length of a datagram socket on this host.
fn datagram_queue_length() -> usize {
    return fs::read_to_string("/proc/sys/net/unix/max_dgram_qlen").ok().and_then(|text| { return text.trim().parse::<usize>().ok(); }).unwrap_or(512);
}

/// Review of step 3, all three reviews: a ring to a receiver that is bound and never reads. Its queue fills, and a blocking send would wait
/// for ever; every ring returns, and a full queue is the answer "rings are waiting".
#[test]
fn a_ring_to_a_receiver_that_does_not_read_never_blocks() {
    let lane: PathBuf = scratch("rev4-saturated");
    let _receiver: std::os::unix::net::UnixDatagram = bind_ring(&lane).expect("a bound receiver that never reads");
    let rings: usize = datagram_queue_length() * 4;
    let child_lane: PathBuf = lane.clone();
    let outcome: Option<i32> = in_single_threaded_process(move || {
        for _ in 0..rings {
            match ring_running_service(&child_lane) {
                Ok(true) => {}
                _ => return 1,
            }
        }
        return 0;
    });
    assert_eq!(outcome, Some(0), "a ring blocked (None) or failed (1) once the receiver's queue was full");
}

/// The doorbell is a socket file in the lane directory: a stale one (nobody answers) is replaced, a live one is refused, anything that is not
/// a socket is refused, and a lane that is a link is not followed.
#[test]
fn the_doorbell_replaces_a_stale_socket_and_refuses_a_live_one_and_a_foreign_file() {
    let lane: PathBuf = scratch("rev4-doorbell-names");
    let first = bind_ring(&lane).expect("the first service's doorbell");
    assert!(matches!(bind_ring(&lane), Err(error) if error.kind() == io::ErrorKind::AddrInUse), "a second doorbell was bound over a live one");
    assert!(ring_running_service(&lane).expect("ring"));
    drop(first);
    assert!(!ring_running_service(&lane).expect("ring"), "nobody is bound, so nobody answers");
    drop(bind_ring(&lane).expect("a stale socket file is replaced"));
    fs::remove_file(lane.join("service.ring")).expect("remove the socket");
    fs::write(lane.join("service.ring"), b"not a socket").expect("a foreign file at the name");
    assert!(matches!(bind_ring(&lane), Err(error) if error.kind() == io::ErrorKind::InvalidInput));
    let link: PathBuf = scratch("rev4-doorbell-link").join("link");
    std::os::unix::fs::symlink(&lane, &link).expect("a link to a lane");
    assert!(bind_ring(&link).is_err(), "a lane that is a symbolic link was followed");
}

/// A lane with a request and saved content retained by a first checked record, and a crash child helper.
fn lane_with_saved_content(label: &str, contents: &[&[u8]]) -> (PathBuf, PathBuf, Vec<String>) {
    let (lane, state, _envelope) = lane_with_envelope(label);
    let names: Vec<String> = contents.iter().map(|bytes| {
        let saved: ClosureEntry = create_saved_content(&lane, bytes).expect("saved content");
        return saved.path().file_name().expect("name").to_string_lossy().to_string();
    }).collect();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    return (lane, state, names);
}

/// Runs `body` in a single-threaded copy that dies at `point`; returns its status.
fn crash_at(point: &'static str, body: impl FnOnce() + Send + 'static) -> Option<i32> {
    return in_single_threaded_process(move || {
        CRASH_AT.with(|cell| { cell.set(Some(point)); });
        set_fault_point(Some(die_at_the_point));
        body();
        return 0;
    });
}

/// Review of step 3 (tests, 1): forcing failures while recovery forces the selected root or its closure leave the saved content in place, no
/// unlink is attempted, and a success forces the segment that is the selected root by its name. Each injection point is a guard: without the
/// forcing of the selected root the first is not reached, without the forcing of the closure the second is not.
#[test]
fn recovery_does_not_unlink_when_forcing_the_selected_root_or_its_closure_fails() {
    for (after, what) in [(0_u32, "the selected segment"), (3, "what the selected root retains")] {
        let (lane, state, names) = lane_with_saved_content("rev4-force-fails", &[b"item"]);
        let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), names[0].clone());
        let status = crash_at("disposal:after-candidate", move || { let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name)); });
        assert_eq!(status, Some(CRASHED));
        start_events();
        inject_sync_error(Some((after, libc::EIO)));
        let swept = startup_sweep(&lane, &state);
        inject_sync_error(None);
        let events: Vec<String> = take_events();
        assert!(swept.is_err(), "the recovery went on after a failure of forcing {}", what);
        assert!(lane.join("saved").join(&names[0]).exists(), "the content was unlinked after a failure of forcing {}", what);
        assert!(!events.iter().any(|event| { return event.starts_with("unlink:"); }), "{:?}", events);
    }
    let (lane, state, names) = lane_with_saved_content("rev4-force-ok", &[b"item"]);
    let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), names[0].clone());
    assert_eq!(crash_at("disposal:after-candidate", move || { let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name)); }), Some(CRASHED));
    let selected = select(&state).expect("readable").expect("a root").number;
    start_events();
    startup_sweep(&lane, &state).expect("the sweep");
    let events: Vec<String> = take_events();
    let wanted: String = format!("file-sync:{}", state.join(format!("state.log.{}", selected)).display());
    let synced: usize = events.iter().position(|event| { return *event == wanted; }).expect("the selected segment was forced by its name");
    let unlinked: usize = events.iter().position(|event| { return event.starts_with("unlink:"); }).expect("the unlink");
    assert!(synced < unlinked, "{:?}", events);
}

/// Review of step 3 (tests, 2): a retirement is refused at the moment a seal sits between its capture and its publication, and an unlink
/// holds the writer between its revalidation and its removal. Each hook proves that its window was reached.
#[test]
fn the_overlapping_schedules_of_the_seal_the_retirement_and_the_unlink_are_refused() {
    let (lane, state, names) = lane_with_saved_content("rev4-overlap", &[b"item"]);
    HOOK.with(|cell| { *cell.borrow_mut() = Some(("lifecycle:seal-after-capture", lane.clone(), state.clone(), "retire-during-seal".to_string())); });
    set_fault_point(Some(hook));
    seal_for_shutdown(&lane, &state).expect("the seal");
    set_fault_point(None);
    assert_eq!(seen(), ["retire:WouldBlock".to_string(), "ledger-exists:false".to_string()], "the hook did not run in the window, or the retirement was not refused");
    let root = select(&state).expect("readable").expect("a root");
    assert!(LaneDocument::parse(&root.payload).expect("checked").saved().contains(&names[0]), "the seal published its closure, which names the item");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&names[0])).expect("the retirement afterwards");
    HOOK.with(|cell| { *cell.borrow_mut() = Some(("disposal:before-unlink", lane.clone(), state.clone(), "writer-and-seal-during-unlink".to_string())); });
    set_fault_point(Some(hook));
    unlink_retired(retired).expect("the unlink");
    set_fault_point(None);
    assert_eq!(seen(), ["writer:WouldBlock".to_string(), "seal:refused".to_string()], "the writer was not held between the revalidation and the removal");
    assert!(!lane.join("saved").join(&names[0]).exists());
}

/// Review of step 3 (tests, 3): a FIFO and a hard link at the error record are refused by their kind and name count, not by chance, at the
/// open and at the clearing; a link or a FIFO put in the record's place between the pass's read and its clearing is not followed or
/// truncated.
#[test]
fn the_error_record_is_refused_as_a_fifo_a_link_and_a_hard_link_at_the_open_and_at_the_clearing() {
    let lane: PathBuf = scratch("rev4-record-kinds");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let fifo: std::ffi::CString = std::ffi::CString::new(lane.join(RECEIPT_ERRORS).as_os_str().as_encoded_bytes()).expect("a path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no other memory.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    for write in [false, true] {
        assert!(matches!(open_error_record(&lane, write), Err(error) if error.kind() == io::ErrorKind::InvalidInput), "a FIFO was opened as the record (write: {})", write);
    }
    assert!(matches!(syncer_pass(&lane), Err(FlushError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput));
    fs::remove_file(lane.join(RECEIPT_ERRORS)).expect("remove the FIFO");
    let outside: PathBuf = lane.join("outside-the-record");
    fs::write(&outside, b"r1\told\n").expect("a file with the record's bytes");
    fs::hard_link(&outside, lane.join(RECEIPT_ERRORS)).expect("a hard link");
    assert!(matches!(open_error_record(&lane, true), Err(error) if error.kind() == io::ErrorKind::InvalidInput), "a hard-linked record was opened");
    fs::remove_file(lane.join(RECEIPT_ERRORS)).expect("remove the hard link");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    // A pass reads a regular record; before it clears, the record is replaced by a link to a file holding the same bytes.
    fs::write(lane.join(RECEIPT_ERRORS), b"r1\told\n").expect("the record");
    let replace_lane: PathBuf = lane.clone();
    let target: PathBuf = outside.clone();
    let pass = syncer_pass_with(&lane, &mut || {
        fs::remove_file(replace_lane.join(RECEIPT_ERRORS)).expect("remove the record");
        std::os::unix::fs::symlink(&target, replace_lane.join(RECEIPT_ERRORS)).expect("a link in its place");
    });
    assert!(pass.is_err(), "the clearing followed a link put in the record's place");
    assert_eq!(fs::read(&outside).expect("the target"), b"r1\told\n", "the target of the link was truncated");
    // The same with a FIFO in its place.
    fs::remove_file(lane.join(RECEIPT_ERRORS)).expect("remove the link");
    fs::write(lane.join(RECEIPT_ERRORS), b"r1\told\n").expect("the record again");
    let fifo_lane: PathBuf = lane.clone();
    let fifo_pass = syncer_pass_with(&lane, &mut || {
        fs::remove_file(fifo_lane.join(RECEIPT_ERRORS)).expect("remove the record");
        let path: std::ffi::CString = std::ffi::CString::new(fifo_lane.join(RECEIPT_ERRORS).as_os_str().as_encoded_bytes()).expect("a path");
        // SAFETY: as above.
        assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    });
    assert!(matches!(fifo_pass, Err(FlushError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput), "a FIFO in the record's place was not refused by its kind");
}

/// Review of step 3 (tests, 5): two real disposals crashed after their lines; a third saved content is a sentinel with no line; the batch
/// recovery itself crashes after it wrote its replacement and before it completed it; the restart finishes both and leaves the sentinel.
#[test]
fn a_crash_during_the_batch_recovery_of_two_pending_disposals_is_recovered_and_the_sentinel_stays() {
    let (lane, state, names) = lane_with_saved_content("rev4-batch", &[b"first", b"second", b"sentinel"]);
    for name in &names[..2] {
        let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), name.clone());
        assert_eq!(crash_at("disposal:after-line", move || { let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name)); }), Some(CRASHED));
    }
    let ledger: String = fs::read_to_string(lane.join("ledger")).expect("the ledger");
    assert_eq!(ledger.lines().count(), 2, "both lines are durable and neither replacement was written");
    let (batch_lane, batch_state) = (lane.clone(), state.clone());
    assert_eq!(crash_at("disposal:after-candidate", move || { let _ = startup_sweep(&batch_lane, &batch_state); }), Some(CRASHED), "the batch recovery did not reach the window");
    assert!(lane.join("saved").join(&names[0]).exists() && lane.join("saved").join(&names[1]).exists(), "nothing was unlinked yet");
    startup_sweep(&lane, &state).expect("the restart");
    assert!(!lane.join("saved").join(&names[0]).exists() && !lane.join("saved").join(&names[1]).exists());
    assert!(lane.join("saved").join(&names[2]).exists(), "the sentinel was removed");
    let root = select(&state).expect("readable").expect("a root");
    let document: LaneDocument = LaneDocument::parse(&root.payload).expect("checked");
    assert_eq!(document.saved(), std::slice::from_ref(&names[2]), "the root names the sentinel and nothing else");
    // Two authorizations and the two lines that ended them: no authorization line was written twice.
    let ledger_text: String = fs::read_to_string(lane.join("ledger")).expect("the ledger");
    assert_eq!(ledger_text.lines().filter(|line| { return line.contains("disposed"); }).count(), 2, "no authorization line was written twice");
    assert_eq!(ledger_text.lines().filter(|line| { return line.contains("disposal-done"); }).count(), 2, "each authorization ended once");
}

/// Review of step 3 (tests, 6): a record on disk whose retained set names one path twice is refused by the parsers (the file is ineligible),
/// and a refused append leaves the previous root as it was.
#[test]
fn the_disk_parsers_refuse_a_duplicate_path_and_a_refused_append_leaves_the_previous_root() {
    let (lane, state, envelope) = lane_with_envelope("rev4-duplicates");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let document: Vec<u8> = LaneDocument::from_closure(std::slice::from_ref(&entry)).expect("document").encode();
    let item: String = format!("item request {} {} {}\n", entry.length(), entry.sha256(), git_policy_cli::commit_integrity::hex(entry.path().as_os_str().as_encoded_bytes()));
    let payload: String = format!("cli-git-body 2\n{}{}document {}\n{}", item, item, document.len(), String::from_utf8(document.clone()).expect("text"));
    let (record, _hash) = raw_record(1, &"0".repeat(64), payload.as_bytes());
    fs::write(state.join("state.log.1"), record).expect("a record a writer would not have produced");
    assert!(!matches!(read(&state), Ok(Some(_))), "a record whose retained set names a path twice was selected");
    fs::remove_file(state.join("state.log.1")).expect("start again");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document, std::slice::from_ref(&entry)).expect("a good first record");
    let doubled: Vec<ClosureEntry> = vec![entry.clone(), entry];
    assert!(StateWriter::acquire(&state).expect("writer").append_with(2, &document, &doubled).is_err());
    assert_eq!(read(&state).expect("readable").expect("a root"), (1, document), "the refusal changed the root");
}

/// Review of step 3 (tests, 9) and the fifth review: a child that a command left unreaped waits in the pending list, no thread is started
/// for it (a fork may follow), and a later poll reaps it once it has ended.
#[test]
fn a_killed_command_that_is_not_reaped_in_its_window_waits_in_the_pending_list_and_is_reaped_when_polled() {
    let threads = || -> usize { return fs::read_dir("/proc/self/task").expect("tasks").count(); };
    let before: usize = threads();
    inject_unreapable_child(true);
    let mut command = std::process::Command::new("sleep");
    command.arg("30");
    let result = run_within(command, Duration::from_millis(20));
    inject_unreapable_child(false);
    assert!(result.is_err());
    assert_eq!(threads(), before, "a reaper thread was started");
    assert_eq!(pending_reaps(), 1, "the killed child that was not reaped in its window never entered the pending list");
    let deadline: Instant = Instant::now() + WAIT;
    while pending_reaps() != 0 {
        assert!(Instant::now() < deadline, "the killed child was never reaped by a poll");
        reap_pending();
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The fifth review: polling reaps a child that has ended. A child that ends by itself after it was handed over leaves no zombie once a
/// poll has seen it: its process number is gone (a zombie still answers signal zero).
#[test]
fn polling_the_pending_list_leaves_no_zombie() {
    let mut command = std::process::Command::new("sleep");
    command.arg("0.3");
    let child = command.spawn().expect("a child that ends by itself");
    let pid: i32 = i32::try_from(child.id()).expect("a pid");
    inject_pending_child(child);
    let deadline: Instant = Instant::now() + WAIT;
    // SAFETY: `kill` with signal zero takes two integers and sends nothing.
    while unsafe { libc::kill(pid, 0) } == 0 {
        assert!(Instant::now() < deadline, "the child {} is still there (a zombie nobody polled)", pid);
        reap_pending();
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The helper's report end is closed once. A helper whose entry returns ends with its code: the process that launched it adopts it (as a
/// subreaper) and reads its status, which was a `SIGABRT` with a core dump in a debug build while the descriptor was closed twice.
#[test]
fn a_helper_that_returns_from_its_entry_ends_with_its_code_and_does_not_abort() {
    let status: Option<i32> = in_single_threaded_process(|| {
        // SAFETY: `prctl` with `PR_SET_CHILD_SUBREAPER` takes integers and changes only this process's attribute.
        unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) };
        if git_policy_cli::commit_helper_launch::launch_detached(|| { return 7; }).is_err() {
            return 90;
        }
        let deadline: Instant = Instant::now() + Duration::from_secs(5);
        loop {
            let mut raw: libc::c_int = 0;
            // SAFETY: `waitpid` takes a pid, a pointer to a live local and flags.
            let reaped: libc::pid_t = unsafe { libc::waitpid(-1, &raw mut raw, libc::WNOHANG) };
            if reaped > 0 {
                return if libc::WIFEXITED(raw) && libc::WEXITSTATUS(raw) == 7 { 0 } else if libc::WIFSIGNALED(raw) { 100 + libc::WTERMSIG(raw) } else { 91 };
            }
            if Instant::now() >= deadline {
                return 92;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    assert_eq!(status, Some(0), "the helper did not end with its code (106 is SIGABRT)");
}

/// Review of step 3 (code, 6): a seal completes while a departure's `select` sits between its scan and its check of the establishment
/// record. The record is read before the scan, so the healthy departure sees the world as it was and is not told `Damaged`.
#[test]
fn a_healthy_concurrent_seal_does_not_make_a_selection_report_damage() {
    let (lane, state, _names) = lane_with_saved_content("rev4-select-race", &[b"item"]);
    seal_for_shutdown(&lane, &state).expect("the first seal establishes a checkpoint");
    HOOK.with(|cell| { *cell.borrow_mut() = Some(("state:select-after-scan", lane.clone(), state.clone(), "seal-during-select".to_string())); });
    set_fault_point(Some(seal_at_select));
    let selected = select(&state);
    set_fault_point(None);
    assert!(SEEN.with(|seen| { return seen.borrow().iter().any(|note| { return note == "sealed"; }); }), "the hook never ran inside the selection");
    seen();
    assert!(!matches!(selected, Err(StateError::Damaged)), "a healthy concurrent seal made the selection report damage");
    // Not only "not damaged": a healthy selection succeeds, and it selects the record the departure saw or the checkpoint that replaced it.
    let root = selected.expect("a healthy concurrent seal made the selection fail").expect("a root");
    assert!(root.generation >= 1, "{:?}", root.generation);
}

/// The hook of the selection race: a seal, once.
fn seal_at_select(point: &str) {
    if point != "state:select-after-scan" {
        return;
    }
    let armed = HOOK.with(|cell| { return cell.borrow_mut().take(); });
    if let Some((_, lane, state, _)) = armed {
        set_fault_point(None);
        seal_for_shutdown(&lane, &state).expect("the seal inside the selection");
        SEEN.with(|seen| { seen.borrow_mut().push("sealed".to_string()); });
    }
}

/// Review of step 3 (tests, 11): old and new error recording at the same time. A legacy writer appends plain lines (no lock, as the old
/// implementation did) while the new recorder writes its own; every line of both is whole and present.
#[test]
fn legacy_and_new_error_recording_at_the_same_time_lose_no_line() {
    let lane: PathBuf = scratch("rev4-concurrent-record");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let count: usize = 60;
    for number in 0..count {
        std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join(format!("new{:03}", number))).expect("a receipt that cannot be forced");
    }
    let legacy_lane: PathBuf = lane.clone();
    let legacy = std::thread::spawn(move || {
        let mut record = fs::OpenOptions::new().create(true).append(true).open(legacy_lane.join(RECEIPT_ERRORS)).expect("the record, opened as the old implementation did");
        for number in 0..count {
            io::Write::write_all(&mut record, format!("old{:03}\tlegacy\n", number).as_bytes()).expect("one write per line");
            std::thread::yield_now();
        }
    });
    let outcome = flush_lane(&lane);
    legacy.join().expect("the legacy writer");
    assert!(outcome.is_err(), "the unforceable receipts are an error");
    let text: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the record");
    assert!(text.ends_with('\n') && text.lines().all(|line| { return line.split('\t').count() == 2; }), "a torn line");
    for number in 0..count {
        assert!(text.contains(&format!("old{:03}\t", number)) && text.contains(&format!("new{:03}\t", number)), "line {} is missing", number);
    }
}

/// Review of step 3 (tests, 11): an old flusher and a new one at the same time over receipts that keep arriving: each of the two sweeps runs
/// on its own lock, neither fails, and every receipt is there at the end.
#[test]
fn an_old_flusher_and_a_new_one_run_at_the_same_time_over_arriving_receipts() {
    let lane: PathBuf = scratch("rev4-concurrent-flushers");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(3));
    let old_lane: PathBuf = lane.clone();
    let old_barrier = barrier.clone();
    let old = std::thread::spawn(move || {
        let _held = git_policy_cli::commit_flusher::hold_legacy_lock(&old_lane).expect("the old flusher's lock");
        old_barrier.wait();
        let mut sweeps: usize = 0;
        for _ in 0..40 {
            for listed in fs::read_dir(old_lane.join(RECEIPT_DIR)).expect("receipts").flatten() {
                git_policy_cli::commit_flusher::sync_path(&listed.path()).expect("the old flusher forces a receipt");
                sweeps += 1;
            }
            std::thread::yield_now();
        }
        return sweeps;
    });
    let new_lane: PathBuf = lane.clone();
    let new_barrier = barrier.clone();
    let new = std::thread::spawn(move || {
        new_barrier.wait();
        let mut held: usize = 0;
        let mut forced: usize = 0;
        for _ in 0..40 {
            match flush_lane(&new_lane).expect("the new flusher") {
                git_policy_cli::commit_flusher::Flushed::Held => held += 1,
                git_policy_cli::commit_flusher::Flushed::Finished { forced: count } | git_policy_cli::commit_flusher::Flushed::Handed { forced: count } | git_policy_cli::commit_flusher::Flushed::Yielded { forced: count } => forced += count,
            }
            std::thread::yield_now();
        }
        return (held, forced);
    });
    barrier.wait();
    for number in 0..80 {
        fs::write(lane.join(RECEIPT_DIR).join(format!("{:032x}", number)), b"receipt").expect("a receipt arrives");
        std::thread::yield_now();
    }
    assert!(old.join().expect("the old flusher") > 0);
    // The new flusher is never `Held` (there is no other new flusher; an old holder's lock is another name), and it forced receipts itself
    // while the old holder held its lock: a new flusher that took the old lock name would be `Held` for as long as the old holder lives.
    let (held, forced) = new.join().expect("the new flusher");
    assert_eq!(held, 0, "the new flusher was held by the old flusher's lock");
    assert!(forced > 0, "the new flusher forced nothing");
    assert_eq!(fs::read_dir(lane.join(RECEIPT_DIR)).expect("receipts").count(), 80, "a receipt is missing");
    flush_lane(&lane).expect("a last sweep forces whatever arrived after the others ended");
}

/// Review of step 3 (code, 4 and 8): an interval of zero is refused before the doorbell exists, a lane another account could write is not a
/// lane, and a lane service refuses a second service on the same lane.
#[test]
fn a_lane_service_refuses_a_zero_interval_a_writable_lane_and_a_second_service() {
    let lane: PathBuf = scratch("rev4-start");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    assert!(start_lane_service(&lane, Duration::ZERO).is_err());
    assert!(!lane.join("service.ring").exists(), "the doorbell was published before the interval was refused");
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&lane, fs::Permissions::from_mode(0o777)).expect("a lane writable by everyone");
    assert!(validate_lane(&lane).is_err(), "a lane writable by others was accepted");
    fs::set_permissions(&lane, fs::Permissions::from_mode(0o770)).expect("a lane writable by its group");
    assert!(validate_lane(&lane).is_err(), "a lane writable by its group was accepted");
    fs::set_permissions(&lane, fs::Permissions::from_mode(0o755)).expect("an ordinary lane");
    assert!(validate_lane(&lane).is_ok());
    let child_lane: PathBuf = lane.clone();
    let second: Option<i32> = in_single_threaded_process(move || {
        if start_lane_service(&child_lane, Duration::from_secs(60)).is_err() {
            return 1;
        }
        return if start_lane_service(&child_lane, Duration::from_secs(60)).is_err() { 0 } else { 2 };
    });
    assert_eq!(second, Some(0), "the first start failed (1) or a second service was allowed (2)");
}
