//! What:
//!  Controls of revision 13's state log: a hash-chained append-only log with checkpoint rotation, recovered as the longest valid prefix.
//! Why:
//!  Two unsynced overwrites of an alternating pair can tear both generations (R12-4); a log is never overwritten, a record that follows
//!  a damaged one is ignored however valid it looks, and a rotation interrupted at any step leaves a valid log. The waiting path of a
//!  plain commit still makes no sync: appending is visible, the syncer or a departure makes it durable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a torn newer record keeps the longest valid prefix', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::model_fixture::scratch;
use git_policy_cli::commit_identity::{start_events, take_events};
use git_policy_cli::commit_identity::set_fault_point;
use git_policy_cli::commit_state_log::{Candidate, ClosureEntry, EntryKind, Prefix, StateError, StateWriter, create_saved_content, inject_lease_errno, inject_no_tmpfile, read, rotation_number, sync_prefix};
use std::path::PathBuf;

/// Appends as the state's writer (the only way to mutate the log).
fn append(dir: &std::path::Path, generation: u64, payload: &[u8]) -> std::io::Result<()> {
    return StateWriter::acquire(dir)?.append(generation, payload);
}

/// The checked document that names nothing: what a lane with no request, no saved content and no ledger holds.
fn empty_document() -> Vec<u8> {
    return git_policy_cli::commit_state_log::LaneDocument::default().encode();
}

/// The checked document that names exactly what `closure` holds.
fn checked(closure: &[ClosureEntry]) -> git_policy_cli::commit_state_log::LaneDocument {
    return git_policy_cli::commit_state_log::LaneDocument::from_closure(closure).expect("a closure the representation can express");
}

/// Starts a checkpoint as the writer.
fn rotate(dir: &std::path::Path, generation: u64, document: &[u8]) -> std::io::Result<Candidate> {
    return StateWriter::acquire(dir)?.begin_checkpoint(generation, document, &[]);
}

/// Completes `candidate` as the writer, with no external dependency.
fn finish(dir: &std::path::Path, candidate: &Candidate) -> std::io::Result<()> {
    return StateWriter::acquire(dir)?.complete_checkpoint(candidate);
}

/// The number of syncs in `events`.
fn syncs(events: &[String]) -> usize {
    return events.iter().filter(|event| { return event.starts_with("file-sync:") || event.starts_with("dir:"); }).count();
}

/// Appends are visible, read back as the newest generation, and make no sync (the waiting path of a plain commit).
#[test]
fn appending_to_the_state_log_makes_no_sync_and_reads_back_the_newest() {
    let dir: PathBuf = scratch("log-append");
    start_events();
    append(&dir, 1, b"adopted").expect("append");
    append(&dir, 2, b"ready").expect("append");
    append(&dir, 3, b"published").expect("append");
    assert_eq!(syncs(&take_events()), 0);
    assert_eq!(read(&dir).expect("readable").expect("state"), (3, b"published".to_vec()));
}

/// A torn newer record, cut at every byte, leaves the longest valid prefix: the previous generation.
#[test]
fn a_torn_newer_record_keeps_the_longest_valid_prefix() {
    let dir: PathBuf = scratch("log-torn");
    append(&dir, 1, b"first").expect("append");
    let after_first: u64 = std::fs::metadata(dir.join("state.log.1")).expect("size").len();
    append(&dir, 2, b"second").expect("append");
    let full: Vec<u8> = std::fs::read(dir.join("state.log.1")).expect("read");
    for cut in after_first as usize..full.len() {
        std::fs::write(dir.join("state.log.1"), &full[..cut]).expect("torn");
        assert_eq!(read(&dir).expect("readable").expect("prefix"), (1, b"first".to_vec()), "cut {}", cut);
    }
}

/// A record that follows a damaged one is ignored however valid it looks: the chain, not the checksum, decides.
#[test]
fn a_valid_looking_record_after_a_damaged_one_is_ignored() {
    let dir: PathBuf = scratch("log-chain");
    append(&dir, 1, b"first").expect("append");
    let boundary: usize = std::fs::metadata(dir.join("state.log.1")).expect("size").len() as usize;
    append(&dir, 2, b"second").expect("append");
    append(&dir, 3, b"third").expect("append");
    let mut bytes: Vec<u8> = std::fs::read(dir.join("state.log.1")).expect("read");
    bytes[boundary + 6] ^= 1;
    std::fs::write(dir.join("state.log.1"), &bytes).expect("damage the second record");
    assert_eq!(read(&dir).expect("readable").expect("prefix"), (1, b"first".to_vec()));
}

/// A rotation interrupted at each step: the new file empty or cut at every byte of its first record keeps the old log as the state; the
/// complete new file is the state; after `complete_rotation` the old file is gone, removed only after the new file and its directory
/// entry were synced.
#[test]
fn a_rotation_interrupted_at_each_step_keeps_a_valid_log() {
    let dir: PathBuf = scratch("log-rotate");
    append(&dir, 1, &empty_document()).expect("append");
    append(&dir, 2, &empty_document()).expect("append");
    let candidate: Candidate = StateWriter::acquire(&dir).expect("writer").begin_checkpoint_reclaiming(3, &git_policy_cli::commit_state_log::LaneDocument::default(), &[]).expect("rotate");
    assert_eq!(candidate.number, 2);
    let new_path: PathBuf = dir.join("state.log.2");
    let complete: Vec<u8> = std::fs::read(&new_path).expect("read");
    for cut in 0..complete.len() {
        std::fs::write(&new_path, &complete[..cut]).expect("interrupted");
        assert_eq!(read(&dir).expect("readable").expect("old log"), (2, empty_document()), "cut {}", cut);
    }
    std::fs::write(&new_path, &complete).expect("complete");
    assert_eq!(read(&dir).expect("readable").expect("new log"), (3, empty_document()));
    assert!(dir.join("state.log.1").exists(), "the old file stays until the new one is durable");
    start_events();
    finish(&dir, &candidate).expect("complete");
    let events: Vec<String> = take_events();
    assert_eq!(
        events,
        vec![
            format!("file-sync:{}", new_path.display()),
            format!("dir:{}", dir.display()),
            format!("remove:{}", dir.join("state.log.1").display()),
        ]
    );
    assert!(!dir.join("state.log.1").exists());
    assert_eq!(read(&dir).expect("readable").expect("after"), (3, empty_document()));
    assert_eq!(rotation_number(&dir).expect("listing"), Some(2));
    append(&dir, 4, b"later").expect("append");
    assert_eq!(read(&dir).expect("readable").expect("later"), (4, b"later".to_vec()));
}

/// Review 5, item 3.5: absence, damage and an I/O error are three answers. A directory with no log is absent, not an error.
#[test]
fn a_directory_with_no_log_is_absent() {
    let dir: PathBuf = scratch("log-absent");
    assert!(read(&dir).expect("an empty directory is readable").is_none());
}

/// A log whose bytes validate nowhere is damaged, not empty: recovery must not read it as "no state", and an append must not truncate
/// it (the bytes are the only evidence of what was there).
#[test]
fn a_log_with_no_valid_record_is_damaged_and_an_append_leaves_it_alone() {
    let dir: PathBuf = scratch("log-damaged");
    std::fs::write(dir.join("state.log.1"), b"not a record at all\n").expect("write garbage");
    assert!(matches!(read(&dir), Err(StateError::Damaged)));
    let refused = append(&dir, 1, b"first");
    assert!(refused.is_err(), "an append over a damaged log must refuse");
    assert_eq!(std::fs::read(dir.join("state.log.1")).expect("read"), b"not a record at all\n".to_vec(), "the evidence is untouched");
}

/// An unreadable newest log (here a directory in its place, which fails for every user) is an I/O error: recovery must not fall back
/// silently to the older generation, and an append must not write a newer generation beside it.
#[test]
fn an_unreadable_newest_log_is_an_io_error_not_an_older_generation() {
    let dir: PathBuf = scratch("log-unreadable");
    append(&dir, 1, b"first").expect("append");
    std::fs::create_dir(dir.join("state.log.2")).expect("a directory where the newest log should be");
    assert!(matches!(read(&dir), Err(StateError::Io(_))), "the newest log cannot be read");
    assert!(append(&dir, 2, b"second").is_err());
    assert_eq!(std::fs::read_dir(dir.join("state.log.2")).expect("still a directory").count(), 0);
}

/// A newest file whose first record is torn falls back to the previous file (it exists until the new one is durable); that fallback is
/// not an error.
#[test]
fn a_torn_newest_file_still_falls_back_to_the_previous_one() {
    let dir: PathBuf = scratch("log-fallback");
    append(&dir, 1, b"first").expect("append");
    std::fs::write(dir.join("state.log.2"), b"rec 2 ").expect("a torn new file");
    assert_eq!(read(&dir).expect("readable").expect("fallback"), (1, b"first".to_vec()));
}

/// Review 6, 2.1: a checkpoint candidate that does not validate (a failed rotation left it empty or torn) must not authorize deleting the
/// older valid root: completion refuses and every older file stays.
#[test]
fn completion_refuses_a_torn_candidate_and_keeps_the_older_root() {
    let dir: PathBuf = scratch("log-torn-candidate");
    append(&dir, 1, b"first").expect("append");
    let candidate: Candidate = rotate(&dir, 2, b"the checkpoint").expect("rotate");
    let full: Vec<u8> = std::fs::read(dir.join("state.log.2")).expect("read");
    for cut in 0..full.len() {
        std::fs::write(dir.join("state.log.2"), &full[..cut]).expect("a failed rotation left a partial candidate");
        assert!(finish(&dir, &candidate).is_err(), "cut {}", cut);
        assert!(dir.join("state.log.1").exists(), "cut {}", cut);
        assert_eq!(read(&dir).expect("readable").expect("older root"), (1, b"first".to_vec()), "cut {}", cut);
    }
}

/// Review 6, 2.1: a checkpoint never introduces an older generation than the log holds, and the refusal leaves no file behind.
#[test]
fn a_checkpoint_of_an_older_generation_is_refused() {
    let dir: PathBuf = scratch("log-older-checkpoint");
    append(&dir, 10, b"ten").expect("append");
    assert!(rotate(&dir, 9, b"nine").is_err());
    assert_eq!(rotation_number(&dir).expect("listing"), Some(1));
    assert_eq!(rotate(&dir, 10, b"ten").expect("the same document may be preserved at its generation").number, 2);
}

/// Review 6, 2.1: an older file holding a newer generation than the candidate is kept (completion refuses): the candidate would be a
/// rollback of the state.
#[test]
fn completion_keeps_an_older_file_that_holds_a_newer_generation() {
    let dir: PathBuf = scratch("log-newer-older");
    append(&dir, 10, b"ten").expect("append");
    append(&dir, 11, b"eleven").expect("append");
    let other: PathBuf = scratch("log-older-candidate");
    append(&other, 10, b"ten").expect("append");
    std::fs::copy(other.join("state.log.1"), dir.join("state.log.2")).expect("a candidate that holds only generation ten");
    assert_eq!(read(&dir).expect("readable").expect("the newer generation is the root").0, 11, "recovery does not roll back to the candidate");
    assert!(finish(&dir, &Candidate { number: 2, generation: 10, hash: String::new() }).is_err());
    assert!(dir.join("state.log.1").exists() && dir.join("state.log.2").exists());
}

/// Review 6, 2.3: the sync acts on the file recovery selects. A torn highest filename is not the state; recovery reads file 1, and file 1
/// is the one that is forced.
#[test]
fn a_sync_forces_the_recovered_file_not_the_highest_filename() {
    let dir: PathBuf = scratch("log-sync-selected");
    append(&dir, 1, b"first").expect("append");
    std::fs::write(dir.join("state.log.2"), b"rec 2 ").expect("a torn candidate");
    start_events();
    let prefix: Prefix = sync_prefix(&dir).expect("sync");
    let events: Vec<String> = take_events();
    assert_eq!(prefix.number, 1);
    assert!(events.contains(&format!("file-sync:{}", dir.join("state.log.1").display())), "{:?}", events);
    assert!(!events.iter().any(|event| { return event.contains("state.log.2"); }), "{:?}", events);
}

/// Review 6, 2.3: a sync claims only the prefix captured before it ran. A record appended while the sync is in flight is visible, is not
/// in the returned prefix, and so is not acknowledged as durable.
#[test]
fn a_sync_acknowledges_only_the_captured_prefix() {
    static DIRECTORY: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
    fn append_during_the_sync(point: &str) {
        if point == "state:after-capture" {
            let dir: PathBuf = DIRECTORY.lock().expect("lock").clone().expect("directory");
            append(&dir, 2, b"appended while the sync runs").expect("append");
        }
    }
    let dir: PathBuf = scratch("log-watermark");
    append(&dir, 1, b"first").expect("append");
    let captured: u64 = std::fs::metadata(dir.join("state.log.1")).expect("size").len();
    *DIRECTORY.lock().expect("lock") = Some(dir.clone());
    set_fault_point(Some(append_during_the_sync));
    let prefix: Prefix = sync_prefix(&dir).expect("sync");
    set_fault_point(None);
    assert_eq!((prefix.number, prefix.length, prefix.generation, prefix.items.len()), (1, captured, 1, 0));
    assert!(std::fs::metadata(dir.join("state.log.1")).expect("size").len() > captured, "the later record is visible");
    assert_eq!(read(&dir).expect("readable").expect("state").0, 2);
}

/// Review 6, 2.2: one writer owns the state; a second is refused at once. A checkpoint seals the segment and later appends go to the new
/// one and survive, so no transition appended after the snapshot is lost with the old segment.
#[test]
fn one_writer_owns_the_state_and_a_checkpoint_does_not_lose_later_appends() {
    let dir: PathBuf = scratch("log-writer");
    let mut writer: StateWriter = StateWriter::acquire(&dir).expect("acquire");
    assert_eq!(StateWriter::acquire(&dir).expect_err("a second writer").kind(), std::io::ErrorKind::WouldBlock);
    writer.append(1, &empty_document()).expect("append");
    writer.append(2, &empty_document()).expect("append");
    assert_eq!(writer.checkpoint_reclaiming(2, &git_policy_cli::commit_state_log::LaneDocument::default(), &[]).expect("checkpoint"), 2);
    writer.append(3, b"three").expect("append after the cutover");
    assert!(!dir.join("state.log.1").exists());
    assert_eq!(read(&dir).expect("readable").expect("state"), (3, b"three".to_vec()));
    drop(writer);
    StateWriter::acquire(&dir).expect("the lock is released with the writer");
}

/// Review 6, 2.2: appending beside a candidate that does not validate is refused; the transition would be lost when the candidate is
/// completed or abandoned.
#[test]
fn an_append_beside_an_unfinished_candidate_is_refused() {
    let dir: PathBuf = scratch("log-unfinished");
    append(&dir, 1, b"first").expect("append");
    std::fs::write(dir.join("state.log.2"), b"rec 2 ").expect("a torn candidate");
    assert!(append(&dir, 2, b"second").is_err());
}

/// Review 6, P2: a header that names a length near the maximum ends the valid prefix; it neither overflows nor panics.
#[test]
fn a_length_field_near_the_maximum_is_damage_not_a_panic() {
    let dir: PathBuf = scratch("log-overflow");
    let header: String = format!("rec 1 {} {}\n", "0".repeat(64), usize::MAX);
    std::fs::write(dir.join("state.log.1"), header.as_bytes()).expect("write");
    assert!(matches!(read(&dir), Err(StateError::Damaged)));
}

/// Review 7, 2.2: a checkpoint at the log's own generation must hold the log's document; another document at one generation would make two
/// roots, and recovery refuses to pick one (`Conflict`) when two valid files disagree.
#[test]
fn two_documents_at_one_generation_are_a_conflict_not_a_root() {
    let dir: PathBuf = scratch("log-same-generation");
    append(&dir, 5, b"five").expect("append");
    assert!(rotate(&dir, 5, b"a different document").is_err());
    let other: PathBuf = scratch("log-same-generation-other");
    append(&other, 5, b"another five").expect("append");
    std::fs::copy(other.join("state.log.1"), dir.join("state.log.2")).expect("a second valid file at the same generation");
    assert!(matches!(read(&dir), Err(StateError::Conflict)));
    assert!(append(&dir, 6, b"six").is_err(), "nothing is appended beside a conflict");
}

/// Review 7, 3.2: the highest possible segment number neither panics nor wraps; the checkpoint is refused and nothing is removed.
#[test]
fn an_exhausted_segment_number_is_an_error_not_a_wrap() {
    let dir: PathBuf = scratch("log-number-overflow");
    append(&dir, 1, b"first").expect("append");
    std::fs::rename(dir.join("state.log.1"), dir.join(format!("state.log.{}", u64::MAX))).expect("rename to the largest number");
    assert!(rotate(&dir, 2, b"second").is_err());
    assert_eq!(read(&dir).expect("readable").expect("state"), (1, b"first".to_vec()));
}

/// Review 7, 3.1: the writer stores its directory absolute, so a later change of the working directory leaves it writing under the lock.
#[test]
fn a_writer_keeps_its_directory_when_the_working_directory_changes() {
    let dir: PathBuf = scratch("log-absolute");
    let relative: PathBuf = dir.strip_prefix(std::env::current_dir().expect("cwd")).map(PathBuf::from).unwrap_or_else(|_| { return dir.clone(); });
    let mut writer: StateWriter = StateWriter::acquire(&relative).expect("acquire");
    std::env::set_current_dir(&dir).expect("move away");
    writer.append(1, b"one").expect("append");
    assert_eq!(read(&dir).expect("readable").expect("state"), (1, b"one".to_vec()));
}

/// Review 7, 4.6: a FIFO at the writer's lock path is refused at once; the open that would block before `try_lock` is never made. The
/// call runs on a thread and the test waits for it with a deadline, so a regression fails instead of hanging.
#[test]
fn a_fifo_at_the_lock_path_is_refused_without_blocking() {
    let dir: PathBuf = scratch("log-fifo-lock");
    let path = std::ffi::CString::new(dir.join("state.writer").to_string_lossy().as_bytes()).expect("path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode; the path outlives the call.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker_dir: PathBuf = dir.clone();
    std::thread::spawn(move || { let _ = sender.send(StateWriter::acquire(&worker_dir).map(|_| ())); });
    let outcome = receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("acquire returned within the deadline");
    assert!(outcome.is_err());
}

/// A lane with a state directory and a request envelope that decodes, for the closure tests.
fn lane_with_envelope(label: &str) -> (PathBuf, PathBuf, PathBuf) {
    let lane: PathBuf = scratch(label);
    let state: PathBuf = lane.join("state");
    std::fs::create_dir(&state).expect("state directory");
    std::fs::create_dir(lane.join("inbox")).expect("inbox directory");
    let (id, bytes): (String, Vec<u8>) = {
        use git_policy_cli::commit_inbox::{encode_inbox_file, with_identity};
        use git_policy_cli::commit_request::{Change, Operation, Request};
        let request: Request = with_identity(
            "0123456789abcdef0123456789abcdef",
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
                    sha256: git_policy_cli::commit_integrity::sha256_hex(b"new bytes"),
                    old_mode: "000000".to_string(),
                    old_sha256: "-".to_string(),
                }],
            },
        )
        .expect("identity");
        (request.id.clone(), encode_inbox_file(&request, &[b"new bytes".to_vec()], &[Vec::new()]).expect("encode"))
    };
    let envelope: PathBuf = lane.join("inbox").join(id);
    std::fs::write(&envelope, bytes).expect("write the envelope");
    return (lane, state, envelope);
}

/// Review 7, 2.3 and review 8, 2.1: completion forces the closure entry (with its ancestry) before it removes the older root, and a closure
/// entry that is not the captured content, or is gone, is refused: the candidate is not a root and the older files stay.
#[test]
fn a_checkpoint_validates_and_forces_its_closure_first_and_refuses_a_missing_entry() {
    let (lane, state, envelope) = lane_with_envelope("log-dependency");
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint(2, b"document naming a request", std::slice::from_ref(&entry)).expect("begin");
    std::fs::remove_file(&envelope).expect("the V envelope is lost");
    assert!(writer.complete_checkpoint(&candidate).is_err());
    assert!(state.join("state.log.1").exists(), "the older root stays when a closure entry is missing");
}

/// Review 8, 2.1 (P1, was red against the path-list API): a path that exists and is forced is not a satisfied dependency. A torn envelope
/// cannot be captured as an envelope entry, and an entry captured whole whose file is torn afterwards does not validate at begin: the
/// checkpoint is refused. Entries have no public fields, so none can be forged from bytes.
#[test]
fn p1_a_torn_envelope_named_as_a_dependency_is_not_a_satisfied_dependency() {
    let (lane, state, envelope) = lane_with_envelope("p1-torn-dependency");
    let whole: Vec<u8> = std::fs::read(&envelope).expect("read");
    let captured: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("the whole envelope is captured");
    std::fs::write(&envelope, &whole[..whole.len() / 2]).expect("tear the envelope");
    assert!(ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).is_err(), "a torn envelope is not captured as an envelope");
    append(&state, 1, b"first").expect("append");
    let outcome = StateWriter::acquire(&state).expect("writer").begin_checkpoint(2, b"document adopting the request", &[captured]);
    assert!(outcome.is_err(), "a checkpoint whose closure has a torn envelope was written");
    assert!(state.join("state.log.1").exists());
}

/// Review 8, 2.2 (P1, was red): a closure entry under a name chain that is only visible. `lane/saved` was created and its name in `lane`
/// never synced; completion syncs the entry, `saved`, and then `lane` (the chain through the durable anchor).
#[test]
fn p1_a_dependency_under_an_unsynced_name_chain_is_forced_through_the_anchor() {
    let lane: PathBuf = scratch("p1-ancestry");
    let state: PathBuf = lane.join("state");
    std::fs::create_dir(&state).expect("state directory");
    let saved: PathBuf = lane.join("saved");
    let entry: ClosureEntry = create_saved_content(&lane, b"a valid dependency").expect("create");
    append(&state, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint(2, b"document", std::slice::from_ref(&entry)).expect("begin");
    start_events();
    writer.complete_checkpoint(&candidate).expect("complete");
    let events: Vec<String> = take_events();
    let saved_sync: usize = events.iter().position(|event| { return *event == format!("dir:{}", saved.display()); }).expect("saved synced");
    let lane_sync: Option<usize> = events.iter().position(|event| { return *event == format!("dir:{}", lane.display()); });
    assert!(lane_sync.is_some_and(|at| { return at > saved_sync; }), "the lane was not synced after saved: {:?}", events);
}

/// Review 8, 2.3 (P1) and the sixteenth review's control 13: a candidate is not a root because it was written or because a background sync
/// made it durable. Here the sync selects and flushes it, a power loss removes the envelope, and recovery's validation of the closure makes it
/// lose to its predecessor.
#[test]
fn p1_a_background_synced_candidate_with_a_lost_dependency_loses_to_its_predecessor() {
    let (lane, state, envelope) = lane_with_envelope("p1-activation");
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let bytes: Vec<u8> = std::fs::read(&envelope).expect("read");
    StateWriter::acquire(&state).expect("writer").begin_checkpoint(2, b"document adopting the request", std::slice::from_ref(&entry)).expect("begin");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2, "with its closure present the candidate is the root");
    sync_prefix(&state).expect("the background sync flushes whatever recovery selects");
    std::fs::remove_file(&envelope).expect("power loss removes the envelope");
    assert_eq!(read(&state).expect("readable").expect("state"), (1, b"first".to_vec()), "the candidate must lose to its predecessor");
    std::fs::write(&envelope, bytes).expect("the envelope comes back");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2);
}

/// Review 8, 2.1: completion removes only the segments the authority names. A segment created after the candidate was begun is not named.
#[test]
fn a_completion_removes_only_the_segments_the_authority_names() {
    let lane: PathBuf = scratch("log-reclaims");
    let state: PathBuf = lane.join("state");
    std::fs::create_dir(&state).expect("state directory");
    append(&state, 1, &empty_document()).expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint_reclaiming(2, &git_policy_cli::commit_state_log::LaneDocument::default(), &[]).expect("begin");
    std::fs::copy(state.join("state.log.1"), state.join("state.log.0")).expect("an unrelated older-numbered file the authority does not name");
    writer.complete_checkpoint(&candidate).expect("complete");
    assert!(!state.join("state.log.1").exists(), "the named predecessor is removed");
    assert!(state.join("state.log.0").exists(), "a segment the authority does not name is kept");
}

/// Components 9, P1 (caller-selected authority): the previous torn-envelope counterexample through the public API. The torn envelope is
/// captured with the kind `Opaque`; its length and checksum match, so capture and the checkpoint succeed, completion forces the torn bytes and
/// removes the predecessor. The kind must not be the caller's choice: content that is an inbox envelope is validated as one.
#[test]
fn p9_a_torn_envelope_captured_as_opaque_is_not_replacement_authority() {
    let (lane, state, envelope) = lane_with_envelope("p9-opaque-torn");
    let whole: Vec<u8> = std::fs::read(&envelope).expect("read");
    std::fs::write(&envelope, &whole[..whole.len() / 2]).expect("tear the envelope");
    append(&state, 1, b"first").expect("append");
    let outcome = ClosureEntry::capture(&lane, &envelope, EntryKind::Opaque).and_then(|entry| {
        let mut writer: StateWriter = StateWriter::acquire(&state)?;
        let candidate: Candidate = writer.begin_checkpoint(2, b"document adopting the request", std::slice::from_ref(&entry))?;
        return writer.complete_checkpoint(&candidate);
    });
    assert!(outcome.is_err(), "the torn envelope was accepted as an opaque dependency and the fallback removed");
    assert!(state.join("state.log.1").exists());
}

/// Components 9, P1 (later appends): after a checkpoint whose closure holds request A, an append whose document no longer lists A is a
/// drop of a live request. It is refused; selection would otherwise return a document that A no longer supports.
#[test]
fn p9_a_later_append_cannot_drop_a_live_request_of_the_checkpoint() {
    let (lane, state, envelope) = lane_with_envelope("p9-append-drops");
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"document adopting A", std::slice::from_ref(&entry)).expect("checkpoint");
    assert!(append(&state, 3, b"a document that does not mention A").is_err(), "an append dropped the checkpoint's live request");
}

/// Components 9, P1 (completion deletes its dependency): a closure entry that is the predecessor segment itself. The authority names the
/// same file as a live dependency and as a deletion, completion validates, forces and deletes it, and the new root can no longer validate.
#[test]
fn p9_completion_cannot_delete_a_dependency_it_validated_and_forced() {
    let lane: PathBuf = scratch("p9-self-dependency");
    let state: PathBuf = lane.join("state");
    std::fs::create_dir(&state).expect("state directory");
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &state.join("state.log.1"), EntryKind::AppendPrefix).expect("capture the predecessor segment");
    let outcome = StateWriter::acquire(&state).expect("writer").checkpoint(2, b"document", std::slice::from_ref(&entry));
    assert!(outcome.is_err(), "completion succeeded and removed the file its own closure requires");
    assert!(state.join("state.log.1").exists());
    assert!(read(&state).expect("readable").is_some());
}

/// Components 9, P1 (mutable dependency): a ledger is captured, the checkpoint completes, and the ledger's writer legitimately appends a
/// line. A whole-file identity then fails and recovery, with the predecessor gone, reports the lane damaged.
#[test]
fn p9_an_append_to_a_captured_ledger_does_not_invalidate_the_root() {
    let lane: PathBuf = scratch("p9-mutable");
    let state: PathBuf = lane.join("state");
    std::fs::create_dir(&state).expect("state directory");
    let ledger: PathBuf = lane.join("ledger");
    std::fs::write(&ledger, b"line one\n").expect("ledger");
    append(&state, 1, &empty_document()).expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint_reclaiming(2, &checked(std::slice::from_ref(&entry)), std::slice::from_ref(&entry)).expect("checkpoint");
    std::fs::OpenOptions::new().append(true).open(&ledger).and_then(|mut file| { return std::io::Write::write_all(&mut file, b"line two\n"); }).expect("the legitimate append succeeds");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2, "the root stays valid after a legitimate append");
    // The prefix is what is identified: a rewrite of the captured bytes is not an append and invalidates the root.
    std::fs::write(&ledger, b"line ONE\nline two\n").expect("a rewrite");
    assert!(matches!(read(&state), Err(StateError::Damaged)));
}

/// Revision 17 control 10 (R16a-6): an established root (the checkpoint completed and its predecessor was deleted) whose listed request file is
/// then lost. Recovery fails closed: the state is reported damaged and its bytes are kept, never an older or an empty state, and never an
/// automatic loss.
#[test]
fn p9_an_established_root_with_a_lost_dependency_fails_closed() {
    let (lane, state, envelope) = lane_with_envelope("p9-established-lost");
    append(&state, 1, &empty_document()).expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint_reclaiming(2, &checked(std::slice::from_ref(&entry)), std::slice::from_ref(&entry)).expect("checkpoint");
    assert!(!state.join("state.log.1").exists(), "the predecessor is gone: the checkpoint is the only root");
    std::fs::remove_file(&envelope).expect("the listed request file is lost");
    assert!(matches!(read(&state), Err(StateError::Damaged)), "recovery must stop and report");
    assert!(state.join("state.log.2").exists(), "the bytes of the root are kept");
}

/// Models review 8 (mandatory kinds at the validation boundary): a closure entry built with the kind `AppendPrefix` under `inbox/<id>` used to
/// validate a torn envelope as a prefix and escape the live set. The kind follows from the path now and entries have no public fields: a torn
/// envelope is refused whatever kind the caller names.
#[test]
fn r14_a_prefix_entry_under_the_inbox_cannot_validate_a_torn_envelope() {
    let (lane, state, envelope) = lane_with_envelope("r14-prefix-torn");
    let whole: Vec<u8> = std::fs::read(&envelope).expect("read");
    std::fs::write(&envelope, &whole[..whole.len() / 2]).expect("tear the envelope");
    append(&state, 1, b"first").expect("append");
    for declared in [EntryKind::AppendPrefix, EntryKind::Opaque, EntryKind::InboxEnvelope] {
        assert!(ClosureEntry::capture(&lane, &envelope, declared).is_err(), "a torn envelope was captured as {:?}", declared);
    }
    assert!(state.join("state.log.1").exists());
}

/// Models review 8 (mutable dependency lifetime): a ledger writer already holds an append descriptor when the ledger is captured and the
/// checkpoint completes (permission bits do not revoke a descriptor). A mutable file cannot be captured as immutable content; captured as the
/// append-only prefix it is, the root survives the writer's next line through the descriptor it already held.
#[test]
fn r14_an_existing_ledger_writer_cannot_invalidate_a_root_that_captured_the_ledger() {
    let lane: PathBuf = scratch("r14-open-descriptor");
    let state: PathBuf = lane.join("state");
    std::fs::create_dir(&state).expect("state directory");
    let ledger: PathBuf = lane.join("ledger");
    std::fs::write(&ledger, b"line one\n").expect("ledger");
    let mut writer_descriptor = std::fs::OpenOptions::new().append(true).open(&ledger).expect("the ledger writer's descriptor");
    append(&state, 1, b"first").expect("append");
    assert!(ClosureEntry::capture(&lane, &ledger, EntryKind::Opaque).is_err(), "a mutable file was captured as immutable content");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("the ledger is an append-only prefix");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"document", std::slice::from_ref(&entry)).expect("checkpoint");
    std::io::Write::write_all(&mut writer_descriptor, b"line two\n").expect("the open descriptor still writes");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2, "the only root was invalidated by its own ledger's writer");
}

/// Models review 8 (retirement discharge): a request is retired because its ledger line exists, and the proof must be inside the replacement
/// authority: the ledger prefix is a retained item, so completion forces it, and a retirement whose ledger prefix does not hold the request's
/// line is refused.
#[test]
fn r14_a_retirement_forces_the_ledger_prefix_that_discharges_it() {
    let (lane, state, envelope) = lane_with_envelope("r14-retire");
    append(&state, 1, b"first").expect("append");
    let request_id: String = envelope.file_name().expect("name").to_string_lossy().into_owned();
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"adopts the request", std::slice::from_ref(&entry)).expect("checkpoint");
    let ledger: PathBuf = lane.join("ledger");
    let line = git_policy_cli::commit_ledger::encode_line(&git_policy_cli::commit_ledger::LedgerEntry {
        id: request_id.clone(),
        outcome: "conflicted".to_string(),
        commit: "-".to_string(),
        time: "2026-01-01T00:00:00+00:00".to_string(),
    })
    .expect("line");
    std::fs::write(&ledger, line).expect("a visible terminal line");
    let proof: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture the ledger prefix");
    let mut writer = StateWriter::acquire(&state).expect("writer");
    assert!(writer.begin_checkpoint_retiring(3, b"retired without proof", &[], std::slice::from_ref(&request_id)).is_err(), "a retirement with no ledger prefix in the authority was accepted");
    start_events();
    let candidate = writer.begin_checkpoint_retiring(3, b"the request is retired", std::slice::from_ref(&proof), std::slice::from_ref(&request_id)).expect("begin");
    writer.complete_checkpoint(&candidate).expect("complete");
    let events: Vec<String> = take_events();
    assert!(events.contains(&format!("file-sync:{}", ledger.display())), "the retirement was not backed by a forced ledger prefix: {:?}", events);
}

/// Models review 8 (later records and retained-prefix obligations): a later record retains the complete set, not only new request
/// identities. The record that adds saved content S is selected while S is there, and when a power loss takes S it is no longer eligible and
/// recovery returns the previous record, whose set still holds.
#[test]
fn r14_a_later_record_that_retains_new_saved_content_is_not_selected_without_it() {
    let (lane, state, envelope) = lane_with_envelope("r14-later-saved");
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.checkpoint(2, b"adopts a", std::slice::from_ref(&request)).expect("checkpoint");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create s");
    let saved: PathBuf = lane.join(content.path());
    writer.append_with(3, b"requires a and s", &[request.clone(), content.clone()]).expect("append with the complete set");
    assert_eq!(read(&state).expect("readable").expect("state").0, 3);
    std::fs::remove_file(&saved).expect("a power loss takes the saved content");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2, "the record whose set is incomplete must not be selected");
}

/// Models review 8 (complete document-bound obligations): an append that leaves out an item the previous record retained is refused, the
/// omitted request and the omitted saved content alike; saved content is dropped only by a `disposed` ledger line inside the new set.
#[test]
fn r14_a_record_cannot_omit_what_the_previous_record_retained() {
    let (lane, state, envelope) = lane_with_envelope("r14-omit");
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create s");
    let saved: PathBuf = lane.join(content.path());
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.checkpoint(2, b"adopts a and s", &[request.clone(), content.clone()]).expect("checkpoint");
    assert!(writer.append_with(3, b"forgets s", std::slice::from_ref(&request)).is_err(), "saved content was dropped without disposal");
    assert!(writer.append_with(3, b"forgets a", std::slice::from_ref(&content)).is_err(), "a request was dropped without a ledger line");
    assert!(writer.append_with(3, b"forgets both", &[]).is_err());
    let name: String = saved.file_name().expect("name").to_string_lossy().chars().take(32).collect();
    let ledger: PathBuf = lane.join("ledger");
    let line = |outcome: &str| -> Vec<u8> {
        return git_policy_cli::commit_ledger::encode_line(&git_policy_cli::commit_ledger::LedgerEntry {
            id: name.clone(),
            outcome: outcome.to_string(),
            commit: "-".to_string(),
            time: "2026-01-01T00:00:00+00:00".to_string(),
        })
        .expect("line");
    };
    std::fs::write(&ledger, line("kept")).expect("a ledger line that is not a disposal");
    let wrong: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture ledger");
    assert!(writer.append_retiring(3, b"disposes s", &[request.clone(), wrong], std::slice::from_ref(&name)).is_err(), "a line that is not `disposed` discharged saved content");
    std::fs::write(&ledger, line("disposed")).expect("the disposal line");
    let right: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture ledger");
    writer.append_retiring(3, b"disposes s", &[request, right], std::slice::from_ref(&name)).expect("a disposed line inside the set discharges it");
}

/// Models review 8 (content-bound covering acknowledgment): the acknowledgment names the record it covered. It still covers after the
/// checkpoint completes and after a later append that keeps its items, and it does not cover a substitute that dropped what it promised.
#[test]
fn r14_an_acknowledgment_survives_completion_and_not_a_substitute_that_drops_its_support() {
    let (lane, state, envelope) = lane_with_envelope("r14-ack");
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint(2, b"adopts a", std::slice::from_ref(&request)).expect("begin");
    let promised = sync_prefix(&state).expect("sync the candidate, which recovery selects");
    assert!(git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "the record itself");
    writer.complete_checkpoint(&candidate).expect("complete");
    assert!(git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "after completion the support survives");
    writer.append_with(3, b"later", std::slice::from_ref(&request)).expect("append that keeps the item");
    assert!(git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "after a later append that keeps the items");
    let forged_hash: Prefix = Prefix { hash: "0".repeat(64), ..promised.clone() };
    assert!(!git_policy_cli::commit_state_log::state_covers(&state, &forged_hash).expect("readable"), "a record the successor does not descend from is not coverage, whatever items it carries");
    // A substitute root that dropped the request: a second lane directory standing in for a verified-substitution failure.
    let other_lane: PathBuf = scratch("r14-ack-substitute");
    let other_state: PathBuf = other_lane.join("state");
    std::fs::create_dir(&other_state).expect("state");
    append(&other_state, 5, b"a substitute at a newer generation with nothing retained").expect("append");
    assert!(!git_policy_cli::commit_state_log::state_covers(&other_state, &promised).expect("readable"), "a substitute that drops the support is not coverage");
}

/// The ledger line `outcome` for `id`, as the ledger encodes it.
fn ledger_line_of(id: &str, outcome: &str) -> Vec<u8> {
    return git_policy_cli::commit_ledger::encode_line(&git_policy_cli::commit_ledger::LedgerEntry {
        id: id.to_string(),
        outcome: outcome.to_string(),
        commit: "-".to_string(),
        time: "2026-01-01T00:00:00+00:00".to_string(),
    })
    .expect("line");
}

/// The name under which saved content is discharged: the first characters of its checksum name.
fn discharge_name(saved: &std::path::Path) -> String {
    return saved.file_name().expect("name").to_string_lossy().chars().take(32).collect();
}

/// Review 9a section 2C (second half): a syntactically valid `disposed` line in an unrelated append-only file is not the lane's discharge
/// evidence. Only the lane's own ledger can discharge a retained item.
#[test]
fn r15_a_disposal_line_in_an_unrelated_prefix_file_does_not_discharge_saved_content() {
    let (lane, state, envelope) = lane_with_envelope("r15-unrelated-ledger");
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create s");
    let saved: PathBuf = lane.join(content.path());
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.checkpoint(2, b"adopts a and s", &[request.clone(), content]).expect("checkpoint");
    let name: String = discharge_name(&saved);
    let notes: PathBuf = lane.join("notes");
    std::fs::write(&notes, ledger_line_of(&name, "disposed")).expect("a valid-looking line in a file that is not the ledger");
    let unrelated: ClosureEntry = ClosureEntry::capture(&lane, &notes, EntryKind::AppendPrefix).expect("capture notes");
    assert!(writer.append_retiring(3, b"disposes s", &[request.clone(), unrelated], std::slice::from_ref(&name)).is_err(), "an unrelated file discharged saved content");
    let ledger: PathBuf = lane.join("ledger");
    std::fs::write(&ledger, ledger_line_of(&name, "disposed")).expect("the lane's ledger");
    let proof: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture the ledger");
    writer.append_retiring(3, b"disposes s", &[request, proof], std::slice::from_ref(&name)).expect("the lane's ledger discharges it");
}

/// Review 9a section 2C (first half): after a checkpoint retained S, a retirement record replaced S with its `disposed` ledger prefix, the
/// proof and the state were forced and S was disposed of. The predecessor segment is gone, and recovery reaches the independently supported
/// retirement record instead of stopping at the checkpoint whose item S is gone and reporting damage.
#[test]
fn r15_recovery_reaches_a_retirement_record_after_the_disposal_removed_the_content() {
    let (lane, state, envelope) = lane_with_envelope("r15-retirement-recovery");
    append(&state, 1, b"first").expect("append");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create s");
    let saved: PathBuf = lane.join(content.path());
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.checkpoint(2, b"retains a and s", &[request.clone(), content]).expect("checkpoint");
    let name: String = discharge_name(&saved);
    let ledger: PathBuf = lane.join("ledger");
    std::fs::write(&ledger, ledger_line_of(&name, "disposed")).expect("the disposal line");
    let proof: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture the ledger");
    writer.append_retiring(3, b"s is disposed", &[request.clone(), proof.clone()], std::slice::from_ref(&name)).expect("retire s");
    git_policy_cli::commit_state_log::force_items(&lane, std::slice::from_ref(&proof)).expect("force the proof");
    sync_prefix(&state).expect("force the state");
    let mut mode = std::fs::metadata(&saved).expect("metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o600);
    std::fs::set_permissions(&saved, mode).expect("the disposal may remove it");
    std::fs::remove_file(&saved).expect("the disposal removes the content");
    assert_eq!(read(&state).expect("recovery reaches the retirement record").expect("state"), (3, b"s is disposed".to_vec()));
}

/// A document that names `paths` as its dependencies and says `text`.
fn document_needing(text: &str, paths: &[&std::path::Path]) -> Vec<u8> {
    let mut document: String = String::from(text);
    document.push('\n');
    for path in paths {
        document.push_str(&git_policy_cli::commit_state_log::needs_line(path));
    }
    return document.into_bytes();
}

/// Writes one raw record the way the log encodes it, bypassing the writer: what a crash, an older version or a damaged disk could leave.
fn raw_record(generation: u64, previous: &str, payload: &[u8]) -> (Vec<u8>, String) {
    let header: String = format!("rec {} {} {}\n", generation, previous, payload.len());
    let mut material: Vec<u8> = previous.as_bytes().to_vec();
    material.extend_from_slice(header.as_bytes());
    material.extend_from_slice(payload);
    let hash: String = git_policy_cli::commit_integrity::sha256_hex(&material);
    let mut bytes: Vec<u8> = header.into_bytes();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(format!("\nhash {}\n", hash).as_bytes());
    return (bytes, hash);
}

/// Review 9a section 2A: a checkpoint (or an append) took a document and an independently supplied list. A durable record retaining nothing,
/// then a request R that is visible and not durable, then a checkpoint whose document requires R and passes no items: no dependency was
/// forced, the predecessor was removed, and after R was lost recovery accepted the document. A document names what it depends on, and a set
/// that does not cover it is refused at the writer.
#[test]
fn r15_a_document_that_names_a_dependency_cannot_be_checkpointed_or_appended_without_it() {
    let (lane, state, envelope) = lane_with_envelope("r15-document-bound");
    append(&state, 1, b"first").expect("a durable record that retains nothing");
    let relative: PathBuf = PathBuf::from("inbox").join(envelope.file_name().expect("name"));
    let document: Vec<u8> = document_needing("requires the request", &[&relative]);
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    assert!(writer.begin_checkpoint(2, &document, &[]).is_err(), "a checkpoint whose document needs a request was accepted with no items");
    assert!(writer.append_with(2, &document, &[]).is_err(), "an append whose document needs a request was accepted with no items");
    assert_eq!(rotation_number(&state).expect("listing"), Some(1), "no candidate segment was started");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    writer.checkpoint(2, &document, std::slice::from_ref(&request)).expect("the covered document is accepted");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2);
}

/// Recovery enforces the same rule on bytes the writer did not produce: a record whose document names a dependency that its own set does
/// not cover is not selected, and the previous record is.
#[test]
fn r15_recovery_does_not_select_a_document_whose_needs_its_set_does_not_cover() {
    let (_lane, state, envelope) = lane_with_envelope("r15-document-bound-recovery");
    let relative: PathBuf = PathBuf::from("inbox").join(envelope.file_name().expect("name"));
    let (first, hash) = raw_record(1, &"0".repeat(64), b"first");
    let (second, _) = raw_record(2, &hash, &document_needing("requires the request", &[&relative]));
    let mut bytes: Vec<u8> = first;
    bytes.extend_from_slice(&second);
    std::fs::write(state.join("state.log.1"), bytes).expect("a log the writer did not produce");
    assert_eq!(read(&state).expect("readable").expect("state"), (1, b"first".to_vec()));
}

/// Review 9a section 2B, coordinator pick of 2026-11-04: immutability is enforced by the creator. Saved content enters the store only
/// through the component, from bytes; an existing file, which may have been written by a process that still holds a descriptor, is not
/// accepted as saved content however it was made, so the reviewer's schedule (a writer keeps a writable descriptor) is impossible through
/// the API and a raw file is refused at capture.
#[test]
fn r15_a_raw_file_in_the_saved_directory_is_not_saved_content_and_the_component_made_one_cannot_be_reopened_for_writing() {
    let (lane, state, envelope) = lane_with_envelope("r15-creator-enforced");
    append(&state, 1, b"first").expect("append");
    std::fs::create_dir_all(lane.join("saved")).expect("saved directory");
    let raw: PathBuf = lane.join("saved").join(git_policy_cli::commit_integrity::sha256_hex(b"raw bytes"));
    std::fs::write(&raw, b"raw bytes").expect("a raw creator, no descriptor kept");
    assert!(ClosureEntry::capture(&lane, &raw, EntryKind::Opaque).is_err(), "an existing writable file was accepted as saved content");
    let kept = std::fs::OpenOptions::new().write(true).open(&raw).expect("the raw creator's writable descriptor");
    assert!(ClosureEntry::capture(&lane, &raw, EntryKind::Opaque).is_err(), "and with a writable descriptor open too");
    drop(kept);
    std::fs::remove_file(&raw).expect("the stray file is removed: unknown content in saved/ would hold up a reclaiming checkpoint");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create");
    let made: PathBuf = lane.join(content.path());
    assert!(std::fs::OpenOptions::new().write(true).open(&made).is_err(), "the component's saved content was opened for writing");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"retains s", &[request, content]).expect("checkpoint");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2);
}

/// The coordinator pick: the read lease is an extra check where the kernel grants it, never a refusal. A file system that refuses leases
/// (NFS, CIFS, 9p, FUSE without lock support) still creates, captures, checkpoints and completes.
#[test]
fn r15_a_file_system_that_refuses_read_leases_still_creates_and_checkpoints_saved_content() {
    let (lane, state, envelope) = lane_with_envelope("r15-no-leases");
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    inject_lease_errno(Some(libc::ENOTSUP));
    let made = create_saved_content(&lane, b"objects of b");
    let checkpoint = made.as_ref().map_err(|error| { return std::io::Error::new(error.kind(), error.to_string()); }).and_then(|entry| {
        return StateWriter::acquire(&state)?.checkpoint(2, b"retains s", &[request.clone(), entry.clone()]).map(|_| ());
    });
    inject_lease_errno(None);
    made.expect("creation without leases");
    checkpoint.expect("a checkpoint without leases");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2);
}

/// Without `O_TMPFILE` (a file system that lacks it) the content is staged in the lane's private directory and ends the same: the same
/// name, read-only, the right bytes, no staging file left, and the staging directory private to the owner.
#[test]
fn r15_saved_content_created_without_tmpfile_ends_sealed_and_leaves_no_staging_file() {
    let (lane, _state, _envelope) = lane_with_envelope("r15-no-tmpfile");
    inject_no_tmpfile(true);
    let made = create_saved_content(&lane, b"objects of b");
    inject_no_tmpfile(false);
    let entry: ClosureEntry = made.expect("creation by the staging route");
    let path: PathBuf = lane.join(entry.path());
    assert_eq!(std::fs::read(&path).expect("read"), b"objects of b");
    let mode = std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(&path).expect("metadata").permissions());
    assert_eq!(mode & 0o222, 0, "read-only");
    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err());
    let staging: PathBuf = lane.join("saved-staging");
    assert_eq!(std::fs::read_dir(&staging).expect("staging directory").count(), 0, "no staged file is left");
    let staging_mode = std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(&staging).expect("metadata").permissions());
    assert_eq!(staging_mode & 0o077, 0, "the staging directory is private to its owner");
}

/// Review 10a section 2B, the adoption boundary: an existing read-only inode at the checksum name is not adopted. A raw creator wrote the
/// bytes under their name, kept its writable descriptor and changed the file to 0444; the old creation found the name, accepted the
/// matching bytes and mode, a checkpoint retained it, and a write through the kept descriptor made the only root unrecoverable.
/// Creation now always installs its own fresh inode over the name, so the raw creator's descriptor writes into an inode that nothing
/// refers to; capture of saved content is refused for any existing file and entries come from creation only.
#[test]
fn r16_an_existing_read_only_inode_at_the_name_is_replaced_and_never_adopted() {
    let (lane, state, envelope) = lane_with_envelope("r16-adoption");
    // The reviewer's condition: a file system or account where the read lease is unavailable, so that nothing but provenance can tell.
    inject_lease_errno(Some(libc::ENOTSUP));
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    std::fs::create_dir_all(lane.join("saved")).expect("saved directory");
    let raw: PathBuf = lane.join("saved").join(git_policy_cli::commit_integrity::sha256_hex(b"objects of b"));
    std::fs::write(&raw, b"objects of b").expect("the raw creator writes the bytes under their name");
    let mut kept = std::fs::OpenOptions::new().write(true).open(&raw).expect("and keeps a writable descriptor");
    std::fs::set_permissions(&raw, std::os::unix::fs::PermissionsExt::from_mode(0o444)).expect("then seals the file");
    assert!(ClosureEntry::capture(&lane, &raw, EntryKind::Opaque).is_err(), "an existing read-only file was captured as saved content");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("creation");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"retains a and s", &[request, content]).expect("checkpoint");
    std::io::Write::write_all(&mut kept, b"tampered").expect("the kept descriptor still writes, into an inode nothing refers to");
    inject_lease_errno(None);
    assert_eq!(read(&state).expect("readable").expect("state").0, 2, "the root was invalidated through an adopted inode");
}

/// Creating the same bytes twice returns the same entry, and a writable occupant of the name is replaced by the component's own sealed
/// file and not refused or adopted.
#[test]
fn r15_creating_saved_content_twice_returns_the_same_entry_and_a_writable_occupant_is_replaced() {
    let (lane, _state, _envelope) = lane_with_envelope("r15-saved-twice");
    let first: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("first");
    let second: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("second");
    assert_eq!(first, second);
    let raw: PathBuf = lane.join("saved").join(git_policy_cli::commit_integrity::sha256_hex(b"other bytes"));
    std::fs::write(&raw, b"other bytes").expect("a writable occupant written by a raw creator");
    let made: ClosureEntry = create_saved_content(&lane, b"other bytes").expect("the occupant is replaced");
    let mode = std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(lane.join(made.path())).expect("metadata").permissions());
    assert_eq!(mode & 0o222, 0, "the file at the name is the component's sealed one");
}

/// Review 10a section 2A: a document that omits its `needs` line passed the declared-dependency check, and the reclaiming checkpoint
/// removed the predecessor. A reclaiming checkpoint derives its obligations from the lane itself: every request envelope in `inbox/`,
/// every saved content in `saved/` and the lane ledger up to its length now must be in the closure (or discharged by a ledger line inside
/// it), whatever the document says.
#[test]
fn r16_a_reclaiming_checkpoint_must_retain_what_the_lane_holds_whatever_its_document_declares() {
    let (lane, state, envelope) = lane_with_envelope("r16-derived-closure");
    append(&state, 1, b"first").expect("a durable record that retains nothing");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    assert!(writer.begin_checkpoint_reclaiming(2, &git_policy_cli::commit_state_log::LaneDocument::default(), &[]).is_err(), "a visible request R was left out of a reclaiming checkpoint");
    assert!(state.join("state.log.1").exists());
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create s");
    assert!(writer.begin_checkpoint_reclaiming(2, &checked(std::slice::from_ref(&request)), std::slice::from_ref(&request)).is_err(), "saved content S was left out");
    let ledger: PathBuf = lane.join("ledger");
    std::fs::write(&ledger, ledger_line_of(&"1".repeat(32), "conflicted")).expect("a ledger line");
    let short: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture the ledger now");
    let mut grown = std::fs::OpenOptions::new().append(true).open(&ledger).expect("the ledger writer");
    std::io::Write::write_all(&mut grown, &ledger_line_of(&"2".repeat(32), "conflicted")).expect("a later line");
    let stale: Vec<ClosureEntry> = vec![request.clone(), content.clone(), short];
    assert!(writer.begin_checkpoint_reclaiming(2, &checked(&stale), &stale).is_err(), "a ledger prefix shorter than the ledger now was accepted");
    let whole: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture the whole ledger");
    let complete: Vec<ClosureEntry> = vec![request, content, whole];
    writer.checkpoint_reclaiming(2, &checked(&complete), &complete).expect("the closure that holds everything the lane holds is accepted");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2);
}

/// Review 9a section 2E: the acknowledged state D retained S. A different document E at a higher generation retained the same S, and
/// `still_covers` accepted it because the items were included. Keeping the same dependency bytes does not show that E descends from D:
/// the replacement authority names the hashes of the records it replaces, and coverage across segments needs the promised record among them.
/// (Restored in round 16: an edit of round 15 had cut this test out of the file while the commit that added it was green.)
#[test]
fn r15_an_unrelated_document_that_retains_the_same_items_does_not_cover_the_acknowledgment() {
    let (lane, state, envelope) = lane_with_envelope("r15-succession");
    append(&state, 1, &empty_document()).expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let document_d: Vec<u8> = checked(std::slice::from_ref(&request)).encode();
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(2, &document_d, std::slice::from_ref(&request)).expect("append d");
    let promised: Prefix = sync_prefix(&state).expect("d is acknowledged");
    let unrelated: PathBuf = lane.join("state-of-another-history");
    std::fs::create_dir(&unrelated).expect("another state directory in the same lane");
    StateWriter::acquire(&unrelated).expect("writer").checkpoint(9, b"e is another document", std::slice::from_ref(&request)).expect("checkpoint e");
    assert!(!git_policy_cli::commit_state_log::state_covers(&unrelated, &promised).expect("readable"), "an unrelated document with the same items was taken for the acknowledged state");
    writer.checkpoint_reclaiming(2, &checked(std::slice::from_ref(&request)), std::slice::from_ref(&request)).expect("the checked compaction of the head replaces the segment");
    assert!(!state.join("state.log.1").exists(), "the segment whose records the checked document contains was reclaimed");
    assert!(git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "the checkpoint that replaced the acknowledged record covers it");
}

/// Review 10a section 2E: the recorded ancestry was automatic, so any checkpoint that the writer began after D named D in its lineage. D was
/// acknowledged retaining S; the writer then checkpointed an unrelated document E at a higher generation that retains S and omits what D
/// promised, and `state_covers` said yes. A checked covering relation: across segments only a compaction of the head covers (the
/// checkpoint holds the head's document at the head's generation, so it replays the chain the acknowledgment is part of); a checkpoint
/// that changes the document records no lineage and covers nothing.
#[test]
fn r16_a_checkpoint_that_changes_the_document_does_not_cover_an_earlier_acknowledgment() {
    let (lane, state, envelope) = lane_with_envelope("r16-checked-covering");
    append(&state, 1, b"first").expect("append");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(2, b"d adopts the request", std::slice::from_ref(&request)).expect("append d");
    let promised: Prefix = sync_prefix(&state).expect("d is acknowledged");
    writer.checkpoint(3, b"e is another document that retains the same request", std::slice::from_ref(&request)).expect("checkpoint e");
    assert!(state.join("state.log.1").exists(), "an opaque checkpoint reclaimed the segment that holds the acknowledged record");
    assert!(!git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "a checkpoint that changed the document covered the acknowledgment");
}

/// An order file in the lane, outside the places the component scans, with the line the state depends on.
fn lane_with_order_file(label: &str) -> (PathBuf, PathBuf, ClosureEntry) {
    let (lane, state, envelope) = lane_with_envelope(label);
    std::fs::write(lane.join("order"), b"adopt a\n").expect("an unforced order file");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture a");
    return (lane, state, request);
}

/// Review 11 section 1, 2A: an opaque document whose state requires an order file O outside the places the component scans, with no
/// `needs` line, was checkpointed with a closure that covered everything the scan requires, and the predecessor was reclaimed; with O lost,
/// recovery accepted the document. The component cannot derive an opaque document's dependencies, so its checkpoints never reclaim.
#[test]
fn r17_an_opaque_document_that_needs_a_file_outside_the_derived_places_never_reclaims_its_predecessor() {
    let (_lane, state, request) = lane_with_order_file("r17-order-file");
    append(&state, 1, b"first").expect("a durable predecessor");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"the state that requires the order file", std::slice::from_ref(&request)).expect("checkpoint");
    assert!(state.join("state.log.1").exists(), "the predecessor of an opaque checkpoint was reclaimed");
    assert_eq!(read(&state).expect("readable").expect("state").0, 2);
}

/// Review 11 section 1, 2A: a plain append reclaims nothing, but a later checkpoint that compacts it would. The compaction of an append
/// whose dependencies the component cannot prove does not reclaim either.
#[test]
fn r17_a_checkpoint_cannot_compact_an_opaque_append_and_reclaim_its_segment() {
    let (_lane, state, request) = lane_with_order_file("r17-compact-append");
    append(&state, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(2, b"an append whose dependencies cannot be proven", std::slice::from_ref(&request)).expect("append");
    writer.checkpoint(2, b"an append whose dependencies cannot be proven", std::slice::from_ref(&request)).expect("the compaction");
    assert!(state.join("state.log.1").exists(), "the compaction of an opaque append removed the segment that holds it");
}

/// Review 11 section 1, 2E: D and E are opaque records retaining the same set; D is acknowledged; E is checkpointed as the head. Head equality
/// credited every record of the chain, so D's hash entered the lineage and D's segment was reclaimed, with nothing showing that E contains
/// what D promised. A checkpoint covers an earlier acknowledgment only when its document is shown to contain that acknowledgment's promised
/// state; otherwise the segment that holds it is not reclaimed.
#[test]
fn r17_a_compaction_of_a_later_opaque_record_does_not_reclaim_or_cover_an_earlier_acknowledgment() {
    let (_lane, state, request) = lane_with_order_file("r17-head-equality");
    append(&state, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(2, b"d adopts the request and promises more", std::slice::from_ref(&request)).expect("append d");
    let promised: Prefix = sync_prefix(&state).expect("d is acknowledged");
    writer.append_with(3, b"e adopts the request and omits d's promise", std::slice::from_ref(&request)).expect("append e");
    writer.checkpoint(3, b"e adopts the request and omits d's promise", std::slice::from_ref(&request)).expect("the compaction of e");
    assert!(state.join("state.log.1").exists(), "the segment that holds the acknowledged record was reclaimed");
    assert!(!git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "a record that was never shown to contain the promise covered it");
}

/// The checked representation: a document that names exactly what the lane holds is checkpointed with reclamation, and it covers an earlier
/// acknowledgment of a document in the same representation whose references it contains. Anything the representation cannot express (an
/// opaque document, a document that differs from its closure, an append-only file that is not the ledger) is refused for reclamation.
#[test]
fn r17_a_checked_document_reclaims_and_covers_and_anything_unchecked_is_refused_for_reclamation() {
    let (lane, state, request) = lane_with_order_file("r17-lane-document");
    let first: git_policy_cli::commit_state_log::LaneDocument = git_policy_cli::commit_state_log::LaneDocument::from_closure(std::slice::from_ref(&request)).expect("expressible");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(1, &first.encode(), std::slice::from_ref(&request)).expect("a checked record");
    let promised: Prefix = sync_prefix(&state).expect("acknowledged");
    let content: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("create s");
    let both: Vec<ClosureEntry> = vec![request.clone(), content.clone()];
    let document: git_policy_cli::commit_state_log::LaneDocument = git_policy_cli::commit_state_log::LaneDocument::from_closure(&both).expect("expressible");
    let order: ClosureEntry = ClosureEntry::capture(&lane, &lane.join("order"), EntryKind::AppendPrefix).expect("capture the order file");
    let with_order: Vec<ClosureEntry> = vec![request.clone(), content.clone(), order];
    assert!(git_policy_cli::commit_state_log::LaneDocument::from_closure(&with_order).is_none(), "an order file has no place in the representation");
    assert!(writer.checkpoint_reclaiming(2, &document, &with_order).is_err(), "a closure with an inexpressible file was reclaimed");
    let other: git_policy_cli::commit_state_log::LaneDocument = git_policy_cli::commit_state_log::LaneDocument::from_closure(std::slice::from_ref(&request)).expect("expressible");
    assert!(writer.checkpoint_reclaiming(2, &other, &both).is_err(), "a document that differs from its closure was reclaimed");
    assert!(git_policy_cli::commit_state_log::LaneDocument::parse(b"an opaque document").is_none());
    writer.checkpoint_reclaiming(2, &document, &both).expect("the checked document is reclaimed");
    assert!(!state.join("state.log.1").exists(), "the contained predecessor was not reclaimed");
    assert!(git_policy_cli::commit_state_log::state_covers(&state, &promised).expect("readable"), "a document that contains the acknowledged references did not cover it");
}
