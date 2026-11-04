//! What:
//!  Controls for the third review of plan step 3 (a review of the diffs and a review of the whole files): an old disposal authorization
//!  that outlives the content it was given for, the confinement of the lane's mutable children, the flusher lock that must identify
//!  this lane, and a hint file that must not stop the receipts from being forced.
//! Why:
//!  A `disposed` line is the person's authorization for one incarnation of some saved content; content with the same bytes that is saved
//!  again later is a new incarnation, and nothing the person authorized earlier may remove it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('content saved again after its disposal is not disposed of by the old line', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::lane_with_envelope;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content, resume_disposals, retire_saved_content, unlink_retired};
use git_policy_cli::commit_flusher::{FlusherLock, RECEIPT_DIR};
use git_policy_cli::commit_flusher_unit::{inject_nonce, open_lane};
use git_policy_cli::commit_identity::set_fault_point;
use git_policy_cli::commit_lane_lifecycle::{capture_lane_closure, seal_for_shutdown, syncer_pass};
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, create_saved_content, select};
use std::cell::RefCell;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

/// A lane with a request and saved content retained by a first checked record.
fn lane_with_saved(label: &str, bytes: &[u8]) -> (PathBuf, PathBuf, String) {
    let (lane, state, _envelope) = lane_with_envelope(label);
    let saved: ClosureEntry = create_saved_content(&lane, bytes).expect("saved content");
    let name: String = saved.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    return (lane, state, name);
}

/// Publishes a root that retains the saved content `name` (the public append admits it).
fn publish_retaining(lane: &Path, state: &Path, name: &str) {
    let saved: ClosureEntry = ClosureEntry::capture_sealed_saved(lane, &lane.join("saved").join(name)).expect("the item is on disk");
    let mut items: Vec<ClosureEntry> = capture_lane_closure(lane).expect("capture");
    items.retain(|entry| { return entry.path() != saved.path(); });
    items.push(saved);
    let document: LaneDocument = LaneDocument::from_closure(&items).expect("expressible");
    let generation: u64 = select(state).expect("readable").expect("root").generation + 1;
    StateWriter::acquire(state).expect("writer").append_with(generation, &document.encode(), &items).expect("a root that retains the item");
}

fn make_fifo(path: &Path) {
    let name: CString = CString::new(path.as_os_str().as_encoded_bytes()).expect("a path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no other memory.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "mkfifo");
}

//region disposal authorization and incarnations

/// Review of step 3 (whole files, 1): content saved again after its disposal has the name of the old content (the name is its checksum),
/// and a root that retains it is accepted. The old `disposed` line is not an authorization for it: recovery leaves it where it is, the next
/// seal captures it, and a new authorization disposes of it.
#[test]
fn content_saved_again_after_its_disposal_is_not_removed_by_the_old_line() {
    let (lane, state, name) = lane_with_saved("rev7-resurrect", b"the same bytes");
    dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the first disposal");
    assert!(!lane.join("saved").join(&name).exists());
    create_saved_content(&lane, b"the same bytes").expect("the same bytes are saved again");
    publish_retaining(&lane, &state, &name);
    assert_eq!(resume_disposals(&lane, &state).expect("recovery"), 0, "recovery acted on the old line");
    assert!(lane.join("saved").join(&name).exists(), "the old authorization removed content saved again");
    let root = select(&state).expect("readable").expect("a root");
    assert!(LaneDocument::parse(&root.payload).expect("checked").saved().contains(&name), "the root that retains the content was replaced");
    assert!(
        capture_lane_closure(&lane).expect("capture").iter().any(|entry| { return entry.path().file_name().is_some_and(|leaf| { return leaf.to_string_lossy() == name.as_str(); }); }),
        "the next seal would not capture content saved again"
    );
    seal_for_shutdown(&lane, &state).expect("a seal keeps it");
    assert!(lane.join("saved").join(&name).exists());
    dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("a new authorization disposes of the new content");
    assert!(!lane.join("saved").join(&name).exists());
}

/// Review of step 3 (whole files, 1): a root that retains the content again between the retirement and the unlink makes the token stale
/// (the unlink refuses), and the authorization ends with it: a later recovery does not drop the renewed retention.
#[test]
fn a_renewed_retention_between_the_retirement_and_the_unlink_ends_the_authorization() {
    let (lane, state, name) = lane_with_saved("rev7-renewed", b"retained again");
    let token = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    publish_retaining(&lane, &state, &name);
    assert!(unlink_retired(token).is_err(), "the stale token unlinked content a newer root retains");
    assert!(fs::read_to_string(lane.join("ledger")).expect("the ledger").contains("disposal-done"), "the stale token left the authorization pending");
    assert_eq!(resume_disposals(&lane, &state).expect("recovery"), 0, "recovery dropped the renewed retention");
    assert!(lane.join("saved").join(&name).exists());
    let root = select(&state).expect("readable").expect("a root");
    assert!(LaneDocument::parse(&root.payload).expect("checked").saved().contains(&name), "the renewed retention was replaced");
}

/// A crash after the authorization, then a root written after it that retains the content: recovery alone (no unlink was attempted, so
/// nothing else ended the authorization) lapses it and leaves the content.
#[test]
fn recovery_lapses_an_authorization_that_a_later_root_retains_the_content_against() {
    let (lane, state, name) = lane_with_saved("rev7-lapse", b"retained after the line");
    let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), name.clone());
    let status: Option<i32> = in_single_threaded_process(move || {
        set_fault_point(Some(die_after_the_line));
        let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name));
        return 0;
    });
    assert_eq!(status, Some(77), "the disposal did not reach its crash point");
    publish_retaining(&lane, &state, &name);
    assert_eq!(resume_disposals(&lane, &state).expect("recovery"), 0, "recovery finished a disposal that a later root retains the content against");
    assert!(lane.join("saved").join(&name).exists());
}

/// Dies at once after the disposal's line.
fn die_after_the_line(point: &str) {
    if point == "disposal:after-line" {
        // SAFETY: `_exit` takes an integer and never returns.
        unsafe { libc::_exit(77) };
    }
}

//endregion

//region confinement below the pinned lane

/// Review of step 3 (whole files, 2): `saved` is a link to an unrelated directory that holds a file named like the content. Saving content
/// followed the link and renamed the staged inode over that file.
#[test]
fn saving_content_never_goes_through_a_linked_saved_directory() {
    let lane: PathBuf = scratch("rev7-create-link");
    let outside: PathBuf = scratch("rev7-create-outside");
    let bytes: &[u8] = b"content to save";
    let name: String = git_policy_cli::commit_integrity::sha256_hex(bytes);
    fs::write(outside.join(&name), b"important, not the content").expect("a file outside named like the content");
    std::os::unix::fs::symlink(&outside, lane.join("saved")).expect("saved is a link");
    assert!(create_saved_content(&lane, bytes).is_err(), "content was saved through a linked directory");
    assert_eq!(fs::read(outside.join(&name)).expect("the file"), b"important, not the content", "a file outside the lane was replaced");
}

/// Review of step 3 (whole files, 2): after a record is selected, a retained item whose directory became a link to a copy is not the
/// lane's item. The selection validates by the lane's own directories.
#[test]
fn a_retained_item_reached_through_a_directory_that_became_a_link_is_not_the_lanes() {
    let (lane, state, _name) = lane_with_saved("rev7-validate-link", b"retained item");
    let copy: PathBuf = scratch("rev7-validate-copy").join("saved");
    fs::create_dir_all(&copy).expect("a copy directory");
    for listed in fs::read_dir(lane.join("saved")).expect("saved").flatten() {
        fs::copy(listed.path(), copy.join(listed.file_name())).expect("a copy");
    }
    fs::remove_dir_all(lane.join("saved")).expect("remove the directory");
    std::os::unix::fs::symlink(&copy, lane.join("saved")).expect("a link in its place");
    let selected = select(&state);
    assert!(!matches!(selected, Ok(Some(_))), "a record was selected through a linked directory");
}

thread_local! {
    /// The place the segment hook puts a valuable file, and the segment it replaces.
    static APPEND_SWAP: RefCell<Option<(PathBuf, PathBuf)>> = const { RefCell::new(None) };
    /// How many receipts the flusher was about to force.
    static FORCED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Counts the receipts the flusher is about to force.
fn count_forcing(point: &str) {
    if point == "flusher:before-force" {
        FORCED.with(|cell| { cell.set(cell.get() + 1); });
    }
}

/// Replaces the state segment by a link to the valuable file, once, between the append's selection and its writable open.
fn swap_segment_for_a_link(point: &str) {
    if point != "state:before-append-open" {
        return;
    }
    if let Some((segment, valuable)) = APPEND_SWAP.with(|cell| { return cell.borrow_mut().take(); }) {
        fs::remove_file(&segment).expect("remove the segment");
        std::os::unix::fs::symlink(&valuable, &segment).expect("a link at the segment's name");
    }
}

/// Review of step 3 (whole files, 2): the segment of an append is opened for writing without following a link; a link put at its name after
/// the selection does not make the append truncate the file it leads to.
#[test]
fn an_append_never_truncates_a_file_that_a_link_at_the_segment_leads_to() {
    let (lane, state, _name) = lane_with_saved("rev7-append-link", b"item");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    let valuable: PathBuf = scratch("rev7-append-valuable").join("valuable");
    fs::write(&valuable, b"keep these bytes, they are longer than nothing").expect("a valuable file");
    APPEND_SWAP.with(|cell| { *cell.borrow_mut() = Some((state.join("state.log.1"), valuable.clone())); });
    set_fault_point(Some(swap_segment_for_a_link));
    let appended = StateWriter::acquire(&state).expect("writer").append_with(2, &document.encode(), &closure);
    set_fault_point(None);
    assert!(APPEND_SWAP.with(|cell| { return cell.borrow().is_none(); }), "the swap never ran");
    assert!(appended.is_err(), "the append went through a link");
    assert_eq!(fs::read(&valuable).expect("the file"), b"keep these bytes, they are longer than nothing", "the file the link leads to was changed");
}

/// A segment with a second name is not appended to (truncating it would cut the other name's file too).
#[test]
fn an_append_refuses_a_segment_that_has_a_second_name() {
    let (lane, state, _name) = lane_with_saved("rev7-append-hard", b"item");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    let other: PathBuf = scratch("rev7-append-hard-other").join("second-name");
    fs::hard_link(state.join("state.log.1"), &other).expect("a second name");
    assert!(StateWriter::acquire(&state).expect("writer").append_with(2, &document.encode(), &closure).is_err(), "an append went on with a segment that has a second name");
}

/// Review of step 3 (whole files, 2): a child directory that another account could write is not accepted as a lane.
#[test]
fn a_lane_whose_child_directory_others_can_write_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    for child in ["state", "saved", "inbox", "decision"] {
        let lane: PathBuf = scratch(&format!("rev7-child-{}", child));
        fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
        fs::create_dir_all(lane.join(child)).expect("the child");
        assert!(open_lane(&lane).is_ok(), "an ordinary {} was refused", child);
        fs::set_permissions(lane.join(child), fs::Permissions::from_mode(0o777)).expect("open to everyone");
        assert!(open_lane(&lane).is_err(), "a {} directory everyone can write was accepted", child);
    }
}

/// The third review (A, 4): the establishment record's name is derived from a nonce; a name already taken (here by a link to a valuable
/// file) is not opened, whatever its shape.
#[test]
fn the_establishment_record_never_opens_the_name_a_nonce_gives_if_it_is_taken() {
    let (lane, state, _name) = lane_with_saved("rev7-nonce", b"item");
    let valuable: PathBuf = scratch("rev7-nonce-valuable").join("valuable");
    fs::write(&valuable, b"keep these bytes").expect("a valuable file");
    inject_nonce(Some(0x1234_5678_9abc_def0));
    std::os::unix::fs::symlink(&valuable, state.join("state.established.123456789abcdef0.tmp")).expect("the name the nonce gives is taken");
    let sealed = seal_for_shutdown(&lane, &state);
    inject_nonce(None);
    assert!(sealed.is_err(), "the checkpoint went on although its temporary name was taken");
    assert_eq!(fs::read(&valuable).expect("the file"), b"keep these bytes", "the taken name was opened and truncated");
}

//endregion

//region the flusher lock and the hint file

/// Review of step 3 (whole files, 3): the flusher lock names the holder that sweeps this lane. A lock file that others can open (they could
/// take its lock and never sweep) or that has a second name (another lane's flusher holds the same inode) is refused, and a new one is
/// private.
#[test]
fn the_flusher_lock_is_private_to_the_account_and_has_one_name() {
    use std::os::unix::fs::PermissionsExt;
    let lane: PathBuf = scratch("rev7-flusher-lock");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let held = FlusherLock::try_acquire(&lane).expect("lock").expect("the lock");
    let mode: u32 = fs::metadata(lane.join(git_policy_cli::commit_flusher::FLUSHER_LOCK)).expect("the lock file").permissions().mode();
    assert_eq!(mode & 0o077, 0, "the lock file was created open to the group or to others ({:o})", mode);
    drop(held);
    fs::set_permissions(lane.join(git_policy_cli::commit_flusher::FLUSHER_LOCK), fs::Permissions::from_mode(0o644)).expect("open to others");
    assert!(FlusherLock::try_acquire(&lane).is_err(), "a lock others can open was taken");
    fs::set_permissions(lane.join(git_policy_cli::commit_flusher::FLUSHER_LOCK), fs::Permissions::from_mode(0o600)).expect("private again");
    let other: PathBuf = scratch("rev7-flusher-lock-other");
    fs::hard_link(lane.join(git_policy_cli::commit_flusher::FLUSHER_LOCK), other.join(git_policy_cli::commit_flusher::FLUSHER_LOCK)).expect("a second name");
    assert!(FlusherLock::try_acquire(&lane).is_err(), "a lock with a second name was taken");
}

/// Review of step 3 (whole files, 4): an error record that cannot be read safely (a link, a FIFO) is reported, and the receipts are still
/// forced: the record is a hint, and the receipt directory is the authority.
#[test]
fn an_unreadable_error_record_does_not_stop_the_receipts_from_being_forced() {
    let lane: PathBuf = scratch("rev7-hint");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    let outside: PathBuf = scratch("rev7-hint-outside").join("record");
    fs::write(&outside, b"r1\told\n").expect("a record elsewhere");
    std::os::unix::fs::symlink(&outside, lane.join("receipt-errors")).expect("a link at the record's name");
    set_fault_point(Some(count_forcing));
    let pass = syncer_pass(&lane);
    set_fault_point(None);
    assert!(pass.is_err(), "the unsafe record was not reported");
    assert_eq!(FORCED.with(std::cell::Cell::get), 1, "the receipt was not forced because the record was unsafe");
    assert_eq!(fs::read(&outside).expect("the target"), b"r1\told\n", "the record's target was changed");
    fs::remove_file(lane.join("receipt-errors")).expect("remove the link");
    make_fifo(&lane.join("receipt-errors"));
    let fifo_lane: PathBuf = lane.clone();
    let outcome: Option<i32> = in_single_threaded_process(move || {
        FORCED.with(|cell| { cell.set(0); });
        set_fault_point(Some(count_forcing));
        let fifo_pass = syncer_pass(&fifo_lane);
        return if fifo_pass.is_err() && FORCED.with(std::cell::Cell::get) == 1 { 0 } else { 1 };
    });
    assert_eq!(outcome, Some(0), "a FIFO at the record's name blocked the pass (None) or stopped the forcing (1)");
}

//endregion
