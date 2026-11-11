//! Controls for the journal read at a service's start:
//! a plan or a watch that gives no answer is left behind at its ceiling,
//! and the service starts without it instead of never starting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stallThePlan(); expect(restoredWithin(100, plan, watch, undefined)).toEqual([noRetries, noWatch]);
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
/// and the service starts with no plan and no watch.
/// Why:
///  Finding 3 of the review of `b3d00b3be4` is a service whose read of the journal held it,
/// so the read is bounded and its
///  answer is allowed to be nothing:
/// a journal nobody may read costs a push its retry until the next service starts,
/// never a commit.
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
    let (retries, notes): (PushRetries, ClaimWatch) = restored_within(SHORT_CEILING, plan, watch, None);
    let waited: Duration = began.elapsed();
    release_the_pipe(stalled.as_path());
    assert!(retries.directories().is_empty(), "a plan that gave no answer leaves the service with no pushes to retry");
    assert!(notes.key().is_none(), "a watch that gave no answer is kept in memory only, so nothing is scheduled for it");
    assert!(waited >= SHORT_CEILING, "the read was given its ceiling: {waited:?}");
    crate::test_support::remove(root.as_path());
}
