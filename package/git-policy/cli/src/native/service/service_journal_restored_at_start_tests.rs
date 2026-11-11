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
/// Findings 1 and 2 of the review of `bc3d54f9cd` are what a start that then forgot the journal cost:
/// a service that wrote its plan into memory only,
/// and one that never read the journal again,
/// so a push an earlier service had promised was not tried for the life of that service
/// and a retry it reported as kept was lost when the process ended.
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
    let (retries, notes): (PushRetries, ClaimWatch) = restored_within(SHORT_CEILING, plan.clone(), watch.clone(), None);
    let waited: Duration = began.elapsed();
    release_the_pipe(stalled.as_path());
    assert!(retries.directories().is_empty(), "a plan that gave no answer leaves the service with no pushes to retry yet");
    assert!(waited >= SHORT_CEILING, "the read was given its ceiling: {waited:?}");
    // What this service makes is written where the next service reads it, and both reads are made again: a service that started
    //  without the journal still keeps every retry it promises and still finds the pushes an earlier service left.
    assert_eq!(retries.plans(), Some(plan.as_path()), "the plan of a start that could not read the journal leaves its files in the journal");
    assert!(retries.needs_restore(), "the plan that gave no answer is read again, so the pushes an earlier service left are tried");
    assert_eq!(notes.key(), Some(watch.clone()), "the notes of a start that could not read them are written in the journal, not lost with the process");
    assert!(notes.needs_retry(), "the notes that gave no answer are read again");
    crate::test_support::remove(root.as_path());
}
