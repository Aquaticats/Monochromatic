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
use git_policy_cli::commit_state_log::{Candidate, Prefix, StateError, StateWriter, read, rotation_number, sync_prefix};
use std::path::PathBuf;

/// Appends as the state's writer (the only way to mutate the log).
fn append(dir: &std::path::Path, generation: u64, payload: &[u8]) -> std::io::Result<()> {
    return StateWriter::acquire(dir)?.append(generation, payload);
}

/// Starts a checkpoint as the writer.
fn rotate(dir: &std::path::Path, generation: u64, document: &[u8]) -> std::io::Result<Candidate> {
    return StateWriter::acquire(dir)?.begin_checkpoint(generation, document);
}

/// Completes `candidate` as the writer, with no external dependency.
fn finish(dir: &std::path::Path, candidate: &Candidate) -> std::io::Result<()> {
    return StateWriter::acquire(dir)?.complete_checkpoint(candidate, &[]);
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
    assert_eq!(prefix, Prefix { number: 1, length: captured, generation: 1 });
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

/// Review 7, 2.3: completion forces the live external dependency its document names before it removes the older root, and refuses (older
/// files kept) when the dependency is missing.
#[test]
fn a_checkpoint_forces_its_live_dependencies_first_and_refuses_a_missing_one() {
    let dir: PathBuf = scratch("log-dependency");
    let inbox: PathBuf = scratch("log-dependency-inbox");
    let request: PathBuf = inbox.join("request-file");
    std::fs::write(&request, b"the request the document names").expect("write");
    append(&dir, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&dir).expect("writer");
    let missing: Candidate = writer.begin_checkpoint(2, b"document naming a request").expect("begin");
    let absent: PathBuf = inbox.join("not-there");
    assert!(writer.complete_checkpoint(&missing, &[absent.as_path()]).is_err());
    assert!(dir.join("state.log.1").exists(), "the older root stays when a live dependency is missing");
    start_events();
    writer.complete_checkpoint(&missing, &[request.as_path()]).expect("complete");
    let events: Vec<String> = take_events();
    let dependency_sync: usize = events.iter().position(|event| { return event == &format!("file-sync:{}", request.display()); }).expect("the dependency was forced");
    let removal: usize = events.iter().position(|event| { return event.starts_with("remove:"); }).expect("the older file was removed");
    assert!(dependency_sync < removal, "{:?}", events);
}

/// Components 8, 2.1 (P1): a path that exists and is forced is not a satisfied dependency. The checkpoint adopts a request whose envelope is
/// torn; the caller supplies that exact path; opening and syncing it succeed. Completion must refuse (the envelope does not decode) and keep
/// the older root.
#[test]
fn p1_a_torn_envelope_named_as_a_dependency_is_not_a_satisfied_dependency() {
    let dir: PathBuf = scratch("p1-torn-dependency");
    let torn: PathBuf = scratch("p1-torn-dependency-inbox").join("request");
    std::fs::write(&torn, b"cli-git-inbox 1\nrecord 40\ntorn").expect("write a torn envelope");
    append(&dir, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&dir).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint(2, b"document adopting the request").expect("begin");
    let outcome = writer.complete_checkpoint(&candidate, &[torn.as_path()]);
    assert!(outcome.is_err(), "completion forced a torn envelope and removed the fallback");
    assert!(dir.join("state.log.1").exists());
}

/// Components 8, 2.2 (P1): a dependency under a name chain that is only visible. `root/saved` exists and its name in `root` was never synced;
/// completion syncs the dependency and `saved` but must also sync `root` (the chain through the durable anchor), after `saved`.
#[test]
fn p1_a_dependency_under_an_unsynced_name_chain_is_forced_through_the_anchor() {
    let root: PathBuf = scratch("p1-ancestry");
    let dir: PathBuf = root.join("state");
    std::fs::create_dir(&dir).expect("state directory");
    let saved: PathBuf = root.join("saved");
    std::fs::create_dir(&saved).expect("saved directory whose name in the root was never synced");
    let input: PathBuf = saved.join("input");
    std::fs::write(&input, b"a valid dependency").expect("write");
    append(&dir, 1, b"first").expect("append");
    let mut writer: StateWriter = StateWriter::acquire(&dir).expect("writer");
    let candidate: Candidate = writer.begin_checkpoint(2, b"document").expect("begin");
    start_events();
    writer.complete_checkpoint(&candidate, &[input.as_path()]).expect("complete");
    let events: Vec<String> = take_events();
    let saved_sync: usize = events.iter().position(|event| { return *event == format!("dir:{}", saved.display()); }).expect("saved synced");
    let root_sync: Option<usize> = events.iter().position(|event| { return *event == format!("dir:{}", root.display()); });
    assert!(root_sync.is_some_and(|at| { return at > saved_sync; }), "the root was not synced after saved: {:?}", events);
}
