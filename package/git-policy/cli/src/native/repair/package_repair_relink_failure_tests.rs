//! What:
//!  Controls for the reason a relink's last try failed,
//!  kept for the notice a given-up record leaves.
//! Why:
//!  The words a person reads about a worktree whose links still name another checkout must say what failed,
//!  and the answer the tool gave is the only thing that does;
//!  a reason that is lost, or read twice, or left behind after the record settled,
//!  is a line the person cannot act on.

use super::*;
use crate::test_support::{fixture, remove};

/// What:
///  A kept reason is read back exactly as it was kept,
///  and keeping another one replaces it.
/// Why:
///   One record has one last try,
///   so what the person reads is that try's answer and never an earlier one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember(gitDir, 'relink-1', 'first'); remember(gitDir, 'relink-1', 'second'); assert(recall(gitDir, 'relink-1') === 'second');
/// ```
#[test]
fn a_kept_reason_is_read_back_and_replaced_by_the_next_try() {
    let root: PathBuf = fixture("relink-failure-kept");
    remember(root.as_path(), "relink-1", "pnpm: exit status 1").expect("kept");
    assert_eq!(recall(root.as_path(), "relink-1").as_deref(), Some("pnpm: exit status 1"));
    remember(root.as_path(), "relink-1", "pnpm: exit status 1: the second answer").expect("kept");
    assert_eq!(recall(root.as_path(), "relink-1").as_deref(), Some("pnpm: exit status 1: the second answer"));
    remove(root.as_path());
}

/// What:
///  A record no try has failed on names no reason.
/// Why:
///   The notice words a record whose tries ran out with no answer as one no tool ever answered,
///   so a missing reason must read as nothing rather than as an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(recall(gitDir, 'relink-never-tried') === undefined);
/// ```
#[test]
fn a_record_with_no_failure_names_no_reason() {
    let root: PathBuf = fixture("relink-failure-none");
    assert_eq!(recall(root.as_path(), "relink-never-tried"), None);
    remove(root.as_path());
}

/// What:
///  A record that settled leaves no reason behind,
///  and forgetting one that is not there is nothing to report.
/// Why:
///   Nothing reads a settled record's reason again,
///  so it would be a file the person never sees and no step removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember(...); forget(gitDir, id); assert(recall(gitDir, id) === undefined); forget(gitDir, id);
/// ```
#[test]
fn a_settled_record_takes_its_reason_away() {
    let root: PathBuf = fixture("relink-failure-settled");
    remember(root.as_path(), "relink-1", "toolchain: exit status 1").expect("kept");
    forget(root.as_path(), "relink-1");
    assert_eq!(recall(root.as_path(), "relink-1"), None);
    forget(root.as_path(), "relink-1");
    remove(root.as_path());
}

/// How long an answer that must not wait may take in a control.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// What:
///  A reason file that is a named pipe with no writer is refused at once,
///  so a command that reads the reason answers instead of waiting for ever.
/// Why:
///   The reason is read on the path a command's notice is worded on,
///  and a name another process may make anything is the class of read this branch added:
///  the file beside it and the lock are read the same way, for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(reasonPath); const answered = await within(PATIENCE, () => recall(gitDir, 'relink-1')); assert(answered === undefined);
/// ```
#[test]
fn a_reason_that_is_a_named_pipe_is_refused_without_waiting() {
    let root: PathBuf = fixture("relink-failure-pipe");
    let directory: PathBuf = root.join(FAILURE_DIRECTORY);
    std::fs::create_dir_all(directory.as_path()).expect("the reason directory");
    make_pipe(directory.join("relink-1").as_path());
    let (answered, answer) = std::sync::mpsc::channel::<Option<String>>();
    let git_dir: PathBuf = root.clone();
    let thread = std::thread::spawn(move || {
        let reason: Option<String> = recall(git_dir.as_path(), "relink-1");
        let _sent = answered.send(reason);
    });
    let within_the_patience: Result<Option<String>, _> = answer.recv_timeout(PATIENCE);
    // A thread that did block on the pipe is never joined, so a control that caught the defect reports it rather than
    // waiting on that thread for ever.
    if within_the_patience.is_ok() {
        thread.join().expect("the reading thread answers");
    } else {
        std::mem::forget(thread);
    }
    remove(root.as_path());
    let Ok(answered_with) = within_the_patience else { panic!("recall waited for a reason file that is a named pipe, and nothing ended it") };
    assert_eq!(answered_with, None, "a pipe holds no reason, so the notice words the account with none to quote");
}
