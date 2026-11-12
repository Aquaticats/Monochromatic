//! Controls for the journal read at a service's start:
//! a plan or a watch that gives no answer is left behind at its ceiling,
//! the service starts with what it read rather than never starting,
//! and it keeps both directories and reads them again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stallThePlan(); const [retries, watch] = restoredWithin(100, plan, watchDir, undefined); expect(retries.needsRestore()).toBe(true);
//! ```

use super::*;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::time::Instant;

/// How long the control's own read is given,
/// long enough that a pipe with no writer never ends inside it and short enough to keep the control quick.
const SHORT_CEILING: Duration = Duration::from_millis(100);

/// The most the control waits for the read it left behind to end.
const PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends the read the ceiling left behind.
/// Why:
///  The read is abandoned,
/// never cancelled (`work_awaited_within_a_ceiling.rs`),
/// so the thread that holds it ends only when a writer appears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { close(openSync(path, 'w')); return } catch { await sleep(10) } }
/// ```
fn release_the_pipe(path: &std::path::Path) {
    let began: Instant = Instant::now();
    while began.elapsed() < PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            Err(_no_reader) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// What:
///  A plan file that gives no answer is left behind at the ceiling,
/// the service starts with no plan and no watch read yet,
/// and both keep their directories and read them again.
/// Why:
///  Finding 3 of the review of `b3d00b3be4` is a service whose read of the journal held it,
/// so the read is bounded and its
///  answer is allowed to be nothing:
/// a journal nobody may read costs a push its retry until the next read,
/// never a commit.
/// What the answer may not be is a list with nowhere to write and a watch with no directory:
/// every plan made afterwards would live and die with the service,
/// and every note would stand nowhere,
/// which is what findings 1 and 2 of the review of `bc3d54f9cd` were
/// (`service_journal_restored_at_start.rs`).
#[test]
fn a_journal_that_gives_no_answer_is_left_behind_at_its_ceiling() {
    let root: std::path::PathBuf = crate::test_support::fixture("journal-restored-at-start");
    let plan: std::path::PathBuf = root.join("push-retries");
    let watch: std::path::PathBuf = root.join("watched");
    std::fs::create_dir_all(plan.as_path()).expect("the plan's directory");
    std::fs::create_dir_all(watch.as_path()).expect("the watch's directory");
    let stalled: std::path::PathBuf = plan.join("a-push-waiting.plan");
    let text: std::ffi::CString = std::ffi::CString::new(stalled.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where a plan file would be: {}", std::io::Error::last_os_error());
    let began: Instant = Instant::now();
    let (retries, notes): (PushRetries, ClaimWatch) = restored_within(SHORT_CEILING, plan.clone(), watch.clone(), Some(String::from("instance")));
    let waited: Duration = began.elapsed();
    release_the_pipe(stalled.as_path());
    match std::fs::remove_file(stalled.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("removing the released pipe failed: {error}"),
    }
    assert!(waited >= SHORT_CEILING, "the read was given its ceiling: {waited:?}");
    assert!(retries.directories().is_empty(), "a plan that gave no answer leaves the service with no pushes to retry yet");
    // What this service makes is written where the next service reads it, and both reads are made again: a service that started
    //  without the journal still keeps every retry it promises and still finds the pushes an earlier service left.
    assert_eq!(retries.plans(), Some(plan.as_path()), "the plan of a start that could not read the journal leaves its files in the journal");
    assert!(retries.needs_restore(), "the plan that gave no answer is read again, so the pushes an earlier service left are tried");
    assert_eq!(notes.key(), Some(watch.clone()), "the notes of a start that could not read them are written in the journal, not lost with the process");
    assert!(notes.needs_retry(), "the notes that gave no answer are read again");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A plan a service could not read at its start is read again,
/// and the push it holds is tried.
/// Why:
///   A read that outlasts its ceiling leaves the service with the journal's directory and no plans,
/// and a plan on disk is a promise the person's failure record already made:
/// "the commit service tries this push again by itself".
/// Giving up on the read until the next restart breaks that promise for the life of this service,
/// so the service makes the read again and every push it finds is due.
#[test]
fn a_plan_a_service_could_not_read_at_its_start_is_read_again_and_tried() {
    let root: std::path::PathBuf = crate::test_support::fixture("journal-restore-read-again");
    let plan: std::path::PathBuf = root.join("push-retries");
    let watch: std::path::PathBuf = root.join("watched");
    std::fs::create_dir_all(plan.as_path()).expect("the plan's directory");
    std::fs::create_dir_all(watch.as_path()).expect("the watch's directory");
    let here: std::path::PathBuf = root.join("repo/.git");
    let item: crate::work_item::WorkItem = crate::work_item::WorkItem {
        kind: crate::work_item::WorkKind::Push,
        repository: root.join("repo"),
        git_dir: here.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    // A service that stopped left the plan of a push it was still trying.
    let mut previous: PushRetries = PushRetries::restored(Duration::from_secs(1), plan.as_path());
    let _pause: Duration = previous.schedule(here.as_path(), &item, 1, Instant::now());
    crate::push_retry_store::save(plan.as_path(), &crate::push_retry_store::StoredPlan { git_dir: here.clone(), item: item.clone(), attempt: 1 })
        .expect("the earlier service's plan file");
    drop(previous);
    assert_eq!(crate::push_retry_store::load(plan.as_path()).len(), 1, "the earlier service's plan is on disk");
    // A file that gives no answer stands where another plan would be, so the next start's read outlasts its ceiling.
    let stalled: std::path::PathBuf = plan.join("z-a-pipe.plan");
    let text: std::ffi::CString = std::ffi::CString::new(stalled.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where another plan file would be: {}", std::io::Error::last_os_error());
    let (mut retries, _notes): (PushRetries, ClaimWatch) = restored_within(SHORT_CEILING, plan.clone(), watch.clone(), None);
    release_the_pipe(stalled.as_path());
    match std::fs::remove_file(stalled.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("removing the released pipe failed: {error}"),
    }
    assert!(retries.directories().is_empty(), "a read that gave no answer leaves this service with no push to retry yet");
    assert!(retries.needs_restore(), "the plan the earlier service kept is still to be read");
    // The read is made again, and the push it finds is due.
    let found: Vec<std::path::PathBuf> = retries.restore();
    assert_eq!(found, vec![here.clone()], "the read names the repository whose push waits");
    assert!(!retries.needs_restore(), "the journal was read");
    assert!(retries.next_wait(here.as_path(), Instant::now()).is_some(), "the push the earlier service promised a retry for is due again");
    assert_eq!(retries.directories(), vec![here], "the loop has the repository to drain for it");
    crate::test_support::remove(root.as_path());
}
