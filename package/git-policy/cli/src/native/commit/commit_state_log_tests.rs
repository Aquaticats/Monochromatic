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
use git_policy_cli::commit_state_log::{Candidate, ClosureEntry, EntryKind, Prefix, StateError, StateWriter, read, rotation_number, sync_prefix};
use std::path::PathBuf;

/// Appends as the state's writer (the only way to mutate the log).
fn append(dir: &std::path::Path, generation: u64, payload: &[u8]) -> std::io::Result<()> {
    return StateWriter::acquire(dir)?.append(generation, payload);
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
    append(&dir, 1, b"first").expect("append");
    append(&dir, 2, b"second").expect("append");
    let candidate: Candidate = rotate(&dir, 3, b"the whole document").expect("rotate");
    assert_eq!(candidate.number, 2);
    let new_path: PathBuf = dir.join("state.log.2");
    let complete: Vec<u8> = std::fs::read(&new_path).expect("read");
    for cut in 0..complete.len() {
        std::fs::write(&new_path, &complete[..cut]).expect("interrupted");
        assert_eq!(read(&dir).expect("readable").expect("old log"), (2, b"second".to_vec()), "cut {}", cut);
    }
    std::fs::write(&new_path, &complete).expect("complete");
    assert_eq!(read(&dir).expect("readable").expect("new log"), (3, b"the whole document".to_vec()));
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
    assert_eq!(read(&dir).expect("readable").expect("after"), (3, b"the whole document".to_vec()));
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
    writer.append(1, b"one").expect("append");
    writer.append(2, b"two").expect("append");
    assert_eq!(writer.checkpoint(2, b"two", &[]).expect("checkpoint"), 2);
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
    std::fs::create_dir(&saved).expect("saved directory whose name in the lane was never synced");
    let input: PathBuf = saved.join(git_policy_cli::commit_integrity::sha256_hex(b"a valid dependency"));
    std::fs::write(&input, b"a valid dependency").expect("write");
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &input, EntryKind::Opaque).expect("capture");
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
    append(&state, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint(2, b"second", &[]).expect("begin");
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
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &ledger, EntryKind::AppendPrefix).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"document", std::slice::from_ref(&entry)).expect("checkpoint");
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
    append(&state, 1, b"first").expect("append");
    let entry: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    StateWriter::acquire(&state).expect("writer").checkpoint(2, b"document adopting the request", std::slice::from_ref(&entry)).expect("checkpoint");
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

/// Writes saved content under `lane/saved/<sha256>` and returns its path.
fn saved_content(lane: &std::path::Path, bytes: &[u8]) -> PathBuf {
    std::fs::create_dir_all(lane.join("saved")).expect("saved directory");
    let path: PathBuf = lane.join("saved").join(git_policy_cli::commit_integrity::sha256_hex(bytes));
    std::fs::write(&path, bytes).expect("saved content");
    return path;
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
    let saved: PathBuf = saved_content(&lane, b"objects of b");
    let content: ClosureEntry = ClosureEntry::capture(&lane, &saved, EntryKind::Opaque).expect("capture s");
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
    let saved: PathBuf = saved_content(&lane, b"objects of b");
    let content: ClosureEntry = ClosureEntry::capture(&lane, &saved, EntryKind::Opaque).expect("capture s");
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
    assert!(git_policy_cli::commit_state_log::state_covers(&state, &forged_hash).expect("readable"), "a successor that carries every item covers the same promise");
    // A substitute root that dropped the request: a second lane directory standing in for a verified-substitution failure.
    let other_lane: PathBuf = scratch("r14-ack-substitute");
    let other_state: PathBuf = other_lane.join("state");
    std::fs::create_dir(&other_state).expect("state");
    append(&other_state, 5, b"a substitute at a newer generation with nothing retained").expect("append");
    assert!(!git_policy_cli::commit_state_log::state_covers(&other_state, &promised).expect("readable"), "a substitute that drops the support is not coverage");
}
