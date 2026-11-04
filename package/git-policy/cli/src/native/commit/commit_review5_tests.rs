//! What:
//!  Controls for the fifth round of review of plan step 3 (a review of the diffs and a review of the whole files): the pathnames the
//!  component opens for writing or removal, the opens and reads that could wait for ever, the doorbell and the lock that make one service
//!  the owner of a lane, the lifetime of the reaper, and the pass that arriving receipts could postpone for ever.
//! Why:
//!  A lane directory is private to its account, and that is no proof that every object already inside it is what the component expects:
//!  a link, a hard link, a FIFO or a growing file put there by accident or by another program must be refused, and never truncated,
//!  followed out of the lane, or waited for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a checkpoint never opens an existing name for its establishment record', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::lane_with_envelope;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content, resume_disposals, retire_saved_content, unlink_retired};
use git_policy_cli::commit_flusher::{RECEIPT_DIR, flush_lane};
use git_policy_cli::commit_flusher_unit::{ManagerRefused, ManagerRoute, inject_pending_child, launch_routed, pending_reaps};
use git_policy_cli::commit_identity::set_fault_point;
use git_policy_cli::commit_lane_lifecycle::{ClientStep, capture_lane_closure, client_after_accept, create_receipt, departure_acknowledge, seal_for_shutdown, syncer_pass, syncer_pass_with};
use git_policy_cli::commit_lane_service::{bind_ring, lock_service, next_due, ring_running_service, start_lane_service};
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, create_saved_content, select};
use std::cell::{Cell, RefCell};
use std::ffi::CString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The exit status of a child that died at its crash point.
const CRASHED: i32 = 77;

/// A request identity for a receipt.
const RECEIPT_ID: &str = "fedcba9876543210fedcba9876543210";

thread_local! {
    /// The directories the swap hook uses: the lane, the place the real saved directory moves to, and the outside directory that takes its name.
    static SWAP: RefCell<Option<(PathBuf, PathBuf, PathBuf)>> = const { RefCell::new(None) };
    /// The fault point at which the process of this thread dies.
    static CRASH_AT: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// The lane in which the flood hook creates receipts, and how many it created.
    static FLOOD: RefCell<Option<(PathBuf, usize)>> = const { RefCell::new(None) };
}

/// Dies at once when the named fault point is reached.
fn die_at_the_point(point: &str) {
    if CRASH_AT.with(|cell| { return cell.get(); }) == Some(point) {
        // SAFETY: `_exit` takes an integer and never returns.
        unsafe { libc::_exit(CRASHED) };
    }
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

/// Makes a FIFO at `path`.
fn make_fifo(path: &Path, mode: u32) {
    let name: CString = CString::new(path.as_os_str().as_encoded_bytes()).expect("a path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no other memory.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), mode) }, 0, "mkfifo");
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

//region the data a checkpoint or a disposal could destroy

/// Review of step 3 (whole files, 1): the establishment record is written through a temporary name; a link, a hard link or a FIFO already
/// at a fixed temporary name was opened and truncated (or waited for). The temporary file is created exclusively under a fresh name, and a
/// file at the old fixed name is left as it is.
#[test]
fn a_checkpoint_never_opens_an_existing_name_for_its_establishment_record() {
    let valuable: PathBuf = scratch("rev5-valuable").join("valuable");
    fs::write(&valuable, b"keep these bytes").expect("a file the account cares about");
    for kind in ["symbolic link", "hard link"] {
        let (lane, state, _name) = lane_with_saved(&format!("rev5-establish-{}", kind.replace(' ', "-")));
        let temporary: PathBuf = state.join("state.established.tmp");
        match kind {
            "symbolic link" => std::os::unix::fs::symlink(&valuable, &temporary).expect("a link at the temporary name"),
            _ => fs::hard_link(&valuable, &temporary).expect("a hard link at the temporary name"),
        }
        seal_for_shutdown(&lane, &state).expect("the seal completes a checkpoint");
        assert_eq!(fs::read(&valuable).expect("the file"), b"keep these bytes", "a {} at the temporary name was truncated by a checkpoint", kind);
        assert!(state.join("state.established").exists(), "the checkpoint was not established ({})", kind);
    }
    let (lane, state, _name) = lane_with_saved("rev5-establish-fifo");
    make_fifo(&state.join("state.established.tmp"), 0o600);
    let sealed: Option<i32> = in_single_threaded_process(move || { return if seal_for_shutdown(&lane, &state).is_ok() { 0 } else { 1 }; });
    assert_eq!(sealed, Some(0), "a FIFO at the temporary name blocked or failed the checkpoint");
}

/// Review of step 3 (whole files, 1): `saved` is a link to a directory outside the lane. A disposal read the item through the link and then
/// removed the outside name. A saved directory that is not a real directory of the lane is refused before a line is written, and a pending
/// disposal found by recovery is not carried out through it either.
#[test]
fn a_saved_directory_that_is_a_link_is_never_followed_by_a_disposal() {
    let (lane, state, name) = lane_with_saved("rev5-saved-link");
    let outside: PathBuf = scratch("rev5-saved-outside").join("saved");
    fs::rename(lane.join("saved"), &outside).expect("move the directory out");
    std::os::unix::fs::symlink(&outside, lane.join("saved")).expect("a link in its place");
    let outcome = dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name));
    assert!(outcome.is_err(), "a disposal went through a linked saved directory");
    assert!(outside.join(&name).exists(), "the disposal removed a file outside the lane");
    assert!(!lane.join("ledger").exists(), "a disposed line was written for content in a linked directory");
    // A line that is already durable (the crash left it) is not carried out through a link put in the directory's place afterwards.
    let (second_lane, second_state, second_name) = lane_with_saved("rev5-saved-link-pending");
    let (child_lane, child_state, child_name) = (second_lane.clone(), second_state.clone(), second_name.clone());
    assert_eq!(crash_at("disposal:after-line", move || { let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name)); }), Some(CRASHED));
    let second_outside: PathBuf = scratch("rev5-saved-outside-pending").join("saved");
    fs::rename(second_lane.join("saved"), &second_outside).expect("move the directory out");
    std::os::unix::fs::symlink(&second_outside, second_lane.join("saved")).expect("a link in its place");
    assert!(resume_disposals(&second_lane, &second_state).is_err(), "recovery went on through a linked saved directory");
    assert!(second_outside.join(&second_name).exists(), "recovery removed a file outside the lane");
    // A seal does not read the lane's saved content through a link either.
    let (third_lane, third_state, _third_name) = lane_with_saved("rev5-saved-link-seal");
    let third_outside: PathBuf = scratch("rev5-saved-outside-seal").join("saved");
    fs::rename(third_lane.join("saved"), &third_outside).expect("move the directory out");
    std::os::unix::fs::symlink(&third_outside, third_lane.join("saved")).expect("a link in its place");
    assert!(seal_for_shutdown(&third_lane, &third_state).is_err(), "a seal captured the content of a linked saved directory");
    // The last step alone: a retirement that was made while the directory was the lane's, and an unlink after it became a link.
    let (fourth_lane, fourth_state, fourth_name) = lane_with_saved("rev5-saved-link-token");
    let token = retire_saved_content(&fourth_lane, &fourth_state, &DisposalAuthorization::explicit(&fourth_name)).expect("the retirement");
    let fourth_outside: PathBuf = scratch("rev5-saved-outside-token").join("saved");
    fs::rename(fourth_lane.join("saved"), &fourth_outside).expect("move the directory out");
    std::os::unix::fs::symlink(&fourth_outside, fourth_lane.join("saved")).expect("a link in its place");
    assert!(unlink_retired(token).is_err(), "the unlink went through a link that took the directory's place");
    assert!(fourth_outside.join(&fourth_name).exists(), "the unlink removed a file outside the lane");
    // The window after the open: the directory is swapped for a link between the unlink's check and its removal. The removal is relative to the
    // directory that was opened, so the outside file of the same name survives.
    let (fifth_lane, fifth_state, fifth_name) = lane_with_saved("rev5-saved-link-window");
    let fifth_token = retire_saved_content(&fifth_lane, &fifth_state, &DisposalAuthorization::explicit(&fifth_name)).expect("the retirement");
    let fifth_moved: PathBuf = scratch("rev5-saved-window-moved").join("saved");
    let fifth_outside: PathBuf = scratch("rev5-saved-window-outside");
    SWAP.with(|cell| { *cell.borrow_mut() = Some((fifth_lane.clone(), fifth_moved.clone(), fifth_outside.clone())); });
    set_fault_point(Some(swap_saved_for_a_link));
    let windowed = unlink_retired(fifth_token);
    set_fault_point(None);
    assert!(SWAP.with(|cell| { return cell.borrow().is_none(); }), "the swap never ran in the window");
    assert!(windowed.is_ok(), "{:?}", windowed);
    assert!(fifth_outside.join(&fifth_name).exists(), "the unlink removed the outside file that the swapped link led to");
    assert!(!fifth_moved.join(&fifth_name).exists(), "the unlink did not remove the lane's own file");
}

/// Replaces the saved directory by a link to an outside directory that holds a file of the same name, once, at the window between the
/// unlink's opening of the directory and its removal.
fn swap_saved_for_a_link(point: &str) {
    if point != "disposal:before-unlink" {
        return;
    }
    if let Some((lane, moved, outside)) = SWAP.with(|cell| { return cell.borrow_mut().take(); }) {
        fs::rename(lane.join("saved"), &moved).expect("move the real directory away");
        for listed in fs::read_dir(&moved).expect("the real directory").flatten() {
            fs::copy(listed.path(), outside.join(listed.file_name())).expect("a file of the same name outside");
        }
        std::os::unix::fs::symlink(&outside, lane.join("saved")).expect("a link in its place");
    }
}

/// Review of step 3 (whole files, 6): a seal over a root that is not in the checked representation published a newer root with an empty
/// state, so the state the opaque root carried was no longer what recovery selects. The seal refuses; the opaque root stays the root.
#[test]
fn the_seal_refuses_a_root_it_cannot_read_instead_of_replacing_its_state() {
    let (lane, state, _envelope) = lane_with_envelope("rev5-opaque");
    StateWriter::acquire(&state).expect("writer").append(1, b"an opaque document").expect("an opaque record");
    assert!(seal_for_shutdown(&lane, &state).is_err(), "the seal superseded a root it could not read");
    let root = select(&state).expect("readable").expect("a root");
    assert_eq!(root.generation, 1, "the opaque root is still the root");
    assert_eq!(root.payload, b"an opaque document");
}

//endregion

//region the receipt the doorbell could swallow

/// A manager route that starts nothing and counts.
struct Counting {
    calls: usize,
}

impl ManagerRoute for Counting {
    fn start(&mut self, _lane: &Path) -> Result<String, ManagerRefused> {
        self.calls += 1;
        return Ok("unit".to_string());
    }
}

/// Review of step 3, both reviews: a send succeeds against any socket the name leads to. A link to another lane's live doorbell, and a
/// socket that others can open, are not this lane's service; the ring is not an answer and the client starts the flusher.
#[test]
fn a_doorbell_that_is_a_link_or_open_to_others_is_not_an_answer_and_the_client_launches() {
    let other: PathBuf = scratch("rev5-ring-other");
    let _live_elsewhere = bind_ring(&other).expect("another lane's service");
    let linked: PathBuf = scratch("rev5-ring-link");
    std::os::unix::fs::symlink(other.join("service.ring"), linked.join("service.ring")).expect("a link to the other doorbell");
    assert!(!matches!(ring_running_service(&linked), Ok(true)), "a link to another lane's doorbell was taken for this lane's service");
    let mut route: Counting = Counting { calls: 0 };
    let step = client_after_accept(&linked, RECEIPT_ID, &mut |lane| { return ring_running_service(lane); }, &mut route).expect("the client");
    assert!(matches!(step, ClientStep::Launched(_)), "the client believed a link and launched nothing");
    assert_eq!(route.calls, 1);
    let open: PathBuf = scratch("rev5-ring-open");
    let socket: std::os::unix::net::UnixDatagram = std::os::unix::net::UnixDatagram::bind(open.join("service.ring")).expect("a socket the account made");
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(open.join("service.ring"), fs::Permissions::from_mode(0o666)).expect("open to others");
    assert!(!matches!(ring_running_service(&open), Ok(true)), "a doorbell that other accounts can write was taken for the service");
    assert!(matches!(bind_ring(&open), Err(error) if error.kind() == io::ErrorKind::InvalidInput), "a service bound beside a doorbell that others can write");
    drop(socket);
    let own: PathBuf = scratch("rev5-ring-own");
    let _service = bind_ring(&own).expect("the service's own doorbell");
    assert!(matches!(ring_running_service(&own), Ok(true)), "the lane's own doorbell is an answer");
}

//endregion

//region the opens and reads that could wait for ever

/// Review of step 3 (whole files, 4): a FIFO where a file is expected. Each of these waited for a peer that never comes; each now ends at
/// once with an error. Every case runs in a single-threaded copy with the harness' limit.
#[test]
fn a_fifo_where_the_lane_expects_a_file_never_blocks_recovery_a_seal_or_an_acceptance() {
    let (ledger_lane, ledger_state, ledger_name) = lane_with_saved("rev5-fifo-ledger");
    make_fifo(&ledger_lane.join("ledger"), 0o600);
    let ledger: Option<i32> = in_single_threaded_process(move || { return if resume_disposals(&ledger_lane, &ledger_state).is_err() { 0 } else { 1 }; });
    assert_eq!(ledger, Some(0), "a FIFO named ledger blocked recovery (None) or was read as a ledger (1) for {}", ledger_name);
    let (saved_lane, saved_state, _saved_name) = lane_with_saved("rev5-fifo-saved");
    let fifo_name: String = "ab".repeat(32);
    make_fifo(&saved_lane.join("saved").join(&fifo_name), 0o444);
    let saved: Option<i32> = in_single_threaded_process(move || { return if seal_for_shutdown(&saved_lane, &saved_state).is_err() { 0 } else { 1 }; });
    assert_eq!(saved, Some(0), "a read-only FIFO named like saved content blocked a seal (None) or was sealed (1)");
    let receipt_lane: PathBuf = scratch("rev5-fifo-receipt");
    fs::create_dir_all(receipt_lane.join(RECEIPT_DIR)).expect("receipt directory");
    make_fifo(&receipt_lane.join(RECEIPT_DIR).join(RECEIPT_ID), 0o600);
    let receipt: Option<i32> = in_single_threaded_process(move || { return if create_receipt(&receipt_lane, RECEIPT_ID).is_err() { 0 } else { 1 }; });
    assert_eq!(receipt, Some(0), "a FIFO named like the receipt blocked an acceptance (None) or was taken as the receipt (1)");
}

/// Review of step 3 (whole files, 4): a regular file that grows while it is read is read to the length it had when it was opened. The
/// hook appends after the open, so a read to the end of the file returns more than the file held.
#[test]
fn a_file_that_grows_during_its_read_is_read_only_to_its_length_at_the_open() {
    let (lane, state, _name) = lane_with_saved("rev5-growing");
    let segment: PathBuf = state.join("state.log.1");
    let length: u64 = fs::metadata(&segment).expect("the segment").len();
    GROWING.with(|cell| { *cell.borrow_mut() = Some(segment.clone()); });
    set_fault_point(Some(grow_after_open));
    let read = git_policy_cli::commit_state_log::read_regular_for_test(&segment);
    set_fault_point(None);
    assert!(GROWING.with(|cell| { return cell.borrow().is_none(); }), "the hook never ran after the open");
    assert_eq!(read.expect("the read").len() as u64, length, "the read followed the file as it grew");
    drop(lane);
}

thread_local! {
    /// The file the hook appends to.
    static GROWING: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Appends a block to the growing file, once, when the read has opened it.
fn grow_after_open(point: &str) {
    if point != "state:read-after-open" {
        return;
    }
    if let Some(path) = GROWING.with(|cell| { return cell.borrow_mut().take(); }) {
        let mut file = fs::OpenOptions::new().append(true).open(path).expect("the file");
        io::Write::write_all(&mut file, &[b'x'; 4096]).expect("growth");
    }
}

//endregion

//region the pass that arriving receipts could postpone

/// Creates one more receipt at every forcing, so that a sweep always finds something new.
fn flood(point: &str) {
    if point != "flusher:before-force" {
        return;
    }
    FLOOD.with(|cell| {
        if let Some((lane, count)) = cell.borrow_mut().as_mut() {
            *count += 1;
            fs::write(lane.join(RECEIPT_DIR).join(format!("f{:031x}", *count)), b"receipt").expect("a receipt arrives");
        }
    });
}

/// Review of step 3 (whole files, 3 and the code review): receipts that keep arriving kept the service's pass from ever ending, so the
/// disposals after it and every later pass were postponed for ever. A service pass sweeps the receipts that were there when it began,
/// says that more arrived, and ends; the one-shot flusher still drains the lane.
#[test]
fn receipts_that_keep_arriving_do_not_keep_a_service_pass_from_ending() {
    let lane: PathBuf = scratch("rev5-flood");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    for number in 0..3_u128 {
        fs::write(lane.join(RECEIPT_DIR).join(format!("a{:031x}", number)), b"receipt").expect("a receipt");
    }
    let pass_lane: PathBuf = lane.clone();
    let outcome: Option<i32> = in_single_threaded_process(move || {
        FLOOD.with(|cell| { *cell.borrow_mut() = Some((pass_lane.clone(), 0)); });
        set_fault_point(Some(flood));
        return match syncer_pass(&pass_lane) {
            Ok(pass) if pass.more => 0,
            Ok(_) => 2,
            Err(_) => 1,
        };
    });
    assert_eq!(outcome, Some(0), "the pass never ended (None), failed (1), or did not say that more arrived (2)");
    let quiet: PathBuf = scratch("rev5-quiet");
    fs::create_dir_all(quiet.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(quiet.join(RECEIPT_DIR).join(RECEIPT_ID), b"receipt").expect("a receipt");
    assert!(!syncer_pass(&quiet).expect("a pass").more, "a pass over a quiet lane said that more arrived");
    flush_lane(&quiet).expect("the one-shot flusher");
}

//endregion

//region the reaper that must leave the process able to fork

/// Review of step 3, both reviews: a child the manager command left that does not end must not make the next fallback impossible. Pending
/// children wait in a list that is polled without waiting, and no thread is started for them, so a long-lived caller can fork again.
#[test]
fn a_pending_child_that_never_ends_does_not_stop_the_next_two_fallback_launches() {
    let lane: PathBuf = scratch("rev5-reaper");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let outcome: Option<i32> = in_single_threaded_process(move || {
        let mut command = std::process::Command::new("sleep");
        command.arg("25");
        let Ok(child) = command.spawn() else { return 90 };
        let pid: libc::pid_t = i32::try_from(child.id()).unwrap_or(0);
        inject_pending_child(child);
        struct Refusing;
        impl ManagerRoute for Refusing {
            fn start(&mut self, _lane: &Path) -> Result<String, ManagerRefused> { return Err(ManagerRefused("refused".to_string())); }
        }
        let mut result: i32 = 0;
        for _ in 0..2 {
            if launch_routed(&lane, &mut Refusing).is_err() {
                result = 1;
            }
        }
        if pending_reaps() != 1 {
            result = 2;
        }
        let threads: usize = fs::read_dir("/proc/self/task").map_or(0, Iterator::count);
        if threads != 1 {
            result = 3;
        }
        // SAFETY: `kill` takes two integers.
        unsafe { libc::kill(pid, libc::SIGKILL) };
        return result;
    });
    assert_eq!(outcome, Some(0), "the second launch failed (1), the child left the list (2) or a thread was started (3)");
}

//endregion

//region the lock and the doorbell that make one service the owner of a lane

/// The fifth review of the diffs: the service lock is created private to the account, and a lock another account could open (it could take
/// the exclusive lock and keep every service out) or that has another name (a hard link couples two lanes) is refused.
#[test]
fn the_service_lock_is_private_and_a_lock_others_can_open_or_a_second_name_of_one_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let lane: PathBuf = scratch("rev5-lock");
    let held = lock_service(&lane).expect("the lock");
    let mode: u32 = fs::metadata(lane.join("service.lock")).expect("the lock file").permissions().mode();
    assert_eq!(mode & 0o077, 0, "the lock file was created open to the group or to others ({:o})", mode);
    drop(held);
    fs::set_permissions(lane.join("service.lock"), fs::Permissions::from_mode(0o644)).expect("open to others");
    assert!(matches!(lock_service(&lane), Err(error) if error.kind() == io::ErrorKind::InvalidInput), "a lock others can open was taken");
    fs::set_permissions(lane.join("service.lock"), fs::Permissions::from_mode(0o600)).expect("private again");
    let other: PathBuf = scratch("rev5-lock-other");
    fs::hard_link(lane.join("service.lock"), other.join("service.lock")).expect("a second name");
    assert!(matches!(lock_service(&lane), Err(error) if error.kind() == io::ErrorKind::InvalidInput), "a lock with a second name was taken");
}

/// The fifth review of the diffs: the public start function itself enforces the lane boundary (the earlier control checked `validate_lane`
/// only), and creates neither the lock nor the doorbell in a directory it refuses.
#[test]
fn the_public_start_function_refuses_a_writable_lane_a_linked_lane_directory_and_a_non_lane() {
    use std::os::unix::fs::PermissionsExt;
    let writable: PathBuf = scratch("rev5-start-writable");
    fs::create_dir_all(writable.join(RECEIPT_DIR)).expect("receipt directory");
    fs::set_permissions(&writable, fs::Permissions::from_mode(0o777)).expect("open to everyone");
    assert!(start_lane_service(&writable, Duration::from_secs(60)).is_err(), "a lane everyone can write was started");
    let linked: PathBuf = scratch("rev5-start-linked");
    fs::create_dir_all(linked.join(RECEIPT_DIR)).expect("receipt directory");
    let outside: PathBuf = scratch("rev5-start-outside");
    std::os::unix::fs::symlink(&outside, linked.join("saved")).expect("a saved directory that is a link");
    assert!(start_lane_service(&linked, Duration::from_secs(60)).is_err(), "a lane whose saved directory is a link was started");
    let bare: PathBuf = scratch("rev5-start-bare");
    assert!(start_lane_service(&bare, Duration::from_secs(60)).is_err(), "a directory with no receipt directory was started");
    for lane in [&writable, &linked, &bare] {
        assert!(!lane.join("service.lock").exists() && !lane.join("service.ring").exists(), "{} got a lock or a doorbell from a refused start", lane.display());
    }
}

thread_local! {
    /// The directory in which the paused service and the second service leave their markers.
    static PAUSE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Pauses the service at the point between its inspection of a stale doorbell and its removal of it, until the second service is done.
fn pause_at_the_stale_socket(point: &str) {
    if point != "service:before-stale-removal" {
        return;
    }
    let Some(markers) = PAUSE.with(|cell| { return cell.borrow().clone(); }) else { return };
    fs::write(markers.join("first-paused"), b"").expect("a marker");
    let deadline: std::time::Instant = std::time::Instant::now() + Duration::from_secs(10);
    while !markers.join("second-done").exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The fourth review of the diffs: two services start on one lane with a stale doorbell. The first has inspected the stale socket and not
/// yet removed it when the second starts. Exactly one of them may run: without the lock the second binds, and the first then removes the
/// second's live socket and binds over it, so both run and one is unreachable.
#[test]
fn two_services_that_meet_at_a_stale_doorbell_are_not_both_started() {
    let lane: PathBuf = scratch("rev5-stale-race");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    drop(bind_ring(&lane).expect("a service that died and left its doorbell"));
    let markers: PathBuf = scratch("rev5-stale-markers");
    let first_lane: PathBuf = lane.clone();
    let first_markers: PathBuf = markers.clone();
    let first = std::thread::spawn(move || {
        return in_single_threaded_process(move || {
            PAUSE.with(|cell| { *cell.borrow_mut() = Some(first_markers); });
            set_fault_point(Some(pause_at_the_stale_socket));
            return if start_lane_service(&first_lane, Duration::from_secs(60)).is_ok() { 0 } else { 1 };
        });
    });
    let deadline: std::time::Instant = std::time::Instant::now() + Duration::from_secs(10);
    while !markers.join("first-paused").exists() {
        assert!(std::time::Instant::now() < deadline, "the first service never reached the stale doorbell");
        std::thread::sleep(Duration::from_millis(5));
    }
    let second_lane: PathBuf = lane.clone();
    let second_markers: PathBuf = markers.clone();
    let second: Option<i32> = in_single_threaded_process(move || {
        let started: bool = start_lane_service(&second_lane, Duration::from_secs(60)).is_ok();
        fs::write(second_markers.join("second-done"), b"").expect("a marker");
        return if started { 0 } else { 1 };
    });
    let first: Option<i32> = first.join().expect("the first service's process");
    let started: usize = [first, second].iter().filter(|status| { return **status == Some(0); }).count();
    assert_eq!(started, 1, "the first service ended with {:?} and the second with {:?}: exactly one may run", first, second);
}

/// The fifth review of the diffs: the probe of a stale-looking doorbell is not blocking either. A receiver that is bound and never reads has
/// a full queue; the probe of a second service must report it as answering, at once, and not wait.
#[test]
fn the_probe_of_a_doorbell_whose_queue_is_full_does_not_block() {
    let lane: PathBuf = scratch("rev5-probe-full");
    let _receiver = bind_ring(&lane).expect("a bound receiver that never reads");
    let qlen: usize = fs::read_to_string("/proc/sys/net/unix/max_dgram_qlen").ok().and_then(|text| { return text.trim().parse::<usize>().ok(); }).unwrap_or(512);
    let child_lane: PathBuf = lane.clone();
    let outcome: Option<i32> = in_single_threaded_process(move || {
        for _ in 0..qlen * 4 {
            if !matches!(ring_running_service(&child_lane), Ok(true)) {
                return 1;
            }
        }
        return if matches!(bind_ring(&child_lane), Err(error) if error.kind() == io::ErrorKind::AddrInUse) { 0 } else { 2 };
    });
    assert_eq!(outcome, Some(0), "a ring failed (1), the probe blocked (None) or did not report the service (2)");
}

//endregion

//region the error record cleared in place

/// The fifth review of the diffs: a hard link put in the record's place between the pass's read and its clearing (to a file that holds the
/// same bytes) is refused, and the file it leads to is not truncated.
#[test]
fn a_hard_link_in_the_error_records_place_is_refused_at_the_clearing() {
    let lane: PathBuf = scratch("rev5-record-hardlink");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    fs::write(lane.join("receipt-errors"), b"r1\told\n").expect("the record");
    let outside: PathBuf = lane.join("outside-the-record");
    fs::write(&outside, b"r1\told\n").expect("a file with the record's bytes");
    let replace_lane: PathBuf = lane.clone();
    let target: PathBuf = outside.clone();
    let pass = syncer_pass_with(&lane, &mut || {
        fs::remove_file(replace_lane.join("receipt-errors")).expect("remove the record");
        fs::hard_link(&target, replace_lane.join("receipt-errors")).expect("a hard link in its place");
    });
    assert!(pass.is_err(), "the clearing accepted a hard link in the record's place");
    assert_eq!(fs::read(&outside).expect("the target"), b"r1\told\n", "the file the hard link leads to was truncated");
}

//endregion

//region recovery that finishes a disposal a newer record retains only in its closure

/// The fifth review of the diffs: a crash after the `disposed` line, and then a newer record that retains the item only in its closure
/// (admitted by the public append). Recovery must replace that root and finish the disposal; without the closure test it refuses the unlink
/// and fails on every start.
#[test]
fn recovery_finishes_a_disposal_that_a_newer_record_retains_only_in_its_closure() {
    let (lane, state, name) = lane_with_saved("rev5-closure-only-recovery");
    let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), name.clone());
    assert_eq!(crash_at("disposal:after-line", move || { let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name)); }), Some(CRASHED));
    let saved: ClosureEntry = ClosureEntry::capture_sealed_saved(&lane, &lane.join("saved").join(&name)).expect("the item is still on disk");
    let mut items: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&items).expect("a document that does not list the item");
    items.push(saved);
    let generation: u64 = select(&state).expect("readable").expect("root").generation + 1;
    StateWriter::acquire(&state).expect("writer").append_with(generation, &document.encode(), &items).expect("an over-approximating record is admitted");
    assert_eq!(resume_disposals(&lane, &state).expect("the recovery finishes the disposal"), 1);
    assert!(!lane.join("saved").join(&name).exists(), "the item is still there");
    assert_eq!(resume_disposals(&lane, &state).expect("a second recovery"), 0, "nothing is left to finish");
}

//endregion

//region the due time and the wiring of the reaper

/// The fifth review: the next pass is due an interval after the start of the one that ended, and at once when receipts arrived meanwhile.
#[test]
fn the_next_pass_is_due_an_interval_after_the_start_or_at_once_when_receipts_arrived() {
    let interval: Duration = Duration::from_secs(3_600);
    let started: std::time::Instant = std::time::Instant::now();
    assert_eq!(next_due(started, interval, false), started + interval);
    assert!(next_due(started, interval, true) < started + interval, "a pass that left arrivals waited for the interval");
}

/// A route that records how many children were pending when it was started, and optionally leaves an ended child in the list.
struct Recording {
    pending_at_start: Option<usize>,
    leaves_a_child: bool,
}

/// An ended child, as a killed command leaves one.
fn ended_child() -> std::process::Child {
    let mut child = std::process::Command::new("true").spawn().expect("a child");
    child.wait().expect("it ends");
    return child;
}

impl ManagerRoute for Recording {
    fn start(&mut self, _lane: &Path) -> Result<String, ManagerRefused> {
        self.pending_at_start = Some(pending_reaps());
        if self.leaves_a_child {
            inject_pending_child(ended_child());
        }
        return Ok("unit".to_string());
    }
}

/// The fifth review of the diffs: the launch itself polls the pending list, before the start (children of earlier launches) and after it
/// (the child this launch's command left); a test that drains the list by hand shows neither.
#[test]
fn a_launch_polls_the_pending_children_before_and_after_the_manager_start() {
    let lane: PathBuf = scratch("rev5-reaper-wiring");
    inject_pending_child(ended_child());
    let mut before: Recording = Recording { pending_at_start: None, leaves_a_child: false };
    launch_routed(&lane, &mut before).expect("a launch");
    assert_eq!(before.pending_at_start, Some(0), "the launch did not reap an ended child of an earlier launch before the start");
    let mut after: Recording = Recording { pending_at_start: None, leaves_a_child: true };
    launch_routed(&lane, &mut after).expect("a launch");
    assert_eq!(pending_reaps(), 0, "the launch did not reap the ended child its own start left");
}

//endregion

//region the force of a retained item that became a FIFO

/// The fifth review of the whole files: forcing the retained set opens each item without blocking. A retained request replaced by a FIFO
/// after the record was written makes the departure's acknowledgment fail, and does not wait for a writer.
#[test]
fn forcing_the_retained_set_does_not_wait_on_an_item_that_became_a_fifo() {
    let (lane, state, _name) = lane_with_saved("rev5-force-fifo");
    let envelope: PathBuf = fs::read_dir(lane.join("inbox")).expect("inbox").flatten().next().expect("an envelope").path();
    fs::remove_file(&envelope).expect("remove the envelope");
    make_fifo(&envelope, 0o600);
    let outcome: Option<i32> = in_single_threaded_process(move || { return if departure_acknowledge(&state, 1).is_err() { 0 } else { 1 }; });
    assert_eq!(outcome, Some(0), "the force of the retained set waited on a FIFO (None) or accepted it (1)");
}

//endregion
