//! Controls for the journal read at a service's start:
//! a plan's name that holds a pipe is answered at once and left alone,
//! and a journal that could not be listed is kept and read again,
//! so the service starts rather than never starting,
//! keeps both directories,
//! and tries the pushes an earlier service promised.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! putAPipeAtAPlansName(); const [retries, watch] = restoredWithin(100, plan, watchDir, undefined); expect(retries.needsRestore()).toBe(false);
//! ```

use super::*;
use std::os::unix::fs::FileTypeExt as _;
use std::time::Instant;

/// The ceiling the controls give the read,
/// short so that a read which waited in the kernel is stopped inside the control and named by it,
/// rather than by the five minutes nextest allows one test.
const SHORT_CEILING: Duration = Duration::from_millis(100);

/// What:
///  A plan's name that holds a pipe is answered at once and left alone,
/// the service keeps both directories,
/// and it owes no second read.
/// Why:
///  "nothing should ever wait forever or indefinitely":
/// opening a pipe for reading waits in the kernel until a writer appears,
/// and nothing promises one,
/// so a record is read the way every record of this wrapper is read
/// (`private_storage::read_regular_file`,
/// `SPEC.md`,
/// "a pipe at a plan's name is answered at once and left alone rather than waited on and discarded").
/// Nothing ever opens this pipe for writing,
///  so a read that answered while it stood there is a read that did not wait inside `open(2)`.
/// A name that holds a pipe,
/// a link,
/// a directory or a file no plan fits in is no plan this store wrote:
/// it is said,
/// it stays,
/// and the start does not read it again after a pause,
/// because a second read of it answers the same.
/// Both directories are kept either way,
/// so the plans this service writes and the notes it makes are where the next start reads them.
#[test]
fn a_pipe_at_a_plans_name_is_answered_at_once_and_left_alone() {
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
    let (retries, notes): (PushRetries, ClaimWatch) = restored_within(SHORT_CEILING, plan.clone(), watch.clone(), Some(String::from("instance")));
    assert!(retries.directories().is_empty(), "a pipe holds no plan, so the service starts with no push to retry yet");
    assert_eq!(retries.plans(), Some(plan.as_path()), "the plan's directory is kept although a pipe stood in it, so what this service writes is read by the next one");
    assert!(!retries.needs_restore(), "a name that holds no plan is not read again after a pause, because a second read of it answers the same");
    let pipe: std::fs::FileType = std::fs::symlink_metadata(stalled.as_path()).expect("the pipe is still there").file_type();
    assert!(pipe.is_fifo(), "a pipe at a plan's name is left alone: it holds no plan, and nothing of this read takes it away");
    assert_eq!(notes.key(), Some(watch), "the notes' directory is kept too, so the notes this service makes are read by the next one");
    assert!(!notes.needs_retry(), "the notes were listed, so nothing of them is owed again");
    std::fs::remove_file(stalled.as_path()).expect("removing the pipe");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A journal a service could not list at its start is read again,
/// and the push it holds is tried.
/// Why:
///   The listing of the journal answers twice:
/// the entries it names,
///  and an error that ended it before it named them,
/// which is what a path that a file or a permission took answers,
/// and what a directory read on a mount that stopped answering answers
/// (`SPEC.md`,
/// "an error that ended it before it named them all ... leaves the journal unread,
/// and it is listed again after a pause").
/// A plan a service could not list is a promise the person's failure record already made:
/// "the commit service tries this push again by itself",
/// so giving up on the read until the next restart breaks that promise for the life of this service.
/// The service therefore keeps the journal's directory and makes the read again,
/// and every push it finds is due.
#[test]
fn a_journal_a_service_could_not_list_at_its_start_is_read_again_and_tried() {
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
    // A file stands where the journal's directory belongs, so the next start's listing of it answers an error.
    let moved: std::path::PathBuf = root.join("push-retries-moved-aside");
    std::fs::rename(plan.as_path(), moved.as_path()).expect("the earlier service's journal is moved aside");
    std::fs::write(plan.as_path(), b"").expect("a file where the journal's directory belongs");
    let (mut retries, _notes): (PushRetries, ClaimWatch) = restored_within(SHORT_CEILING, plan.clone(), watch.clone(), None);
    assert!(retries.directories().is_empty(), "a journal that could not be listed leaves this service with no push to retry yet");
    assert!(retries.needs_restore(), "the plan the earlier service kept is still to be read");
    // The journal stands again with its plan in it, and the read that was owed finds the push.
    std::fs::remove_file(plan.as_path()).expect("removing the file that stood where the journal's directory belongs");
    std::fs::rename(moved.as_path(), plan.as_path()).expect("the earlier service's journal stands again");
    let found: Vec<std::path::PathBuf> = retries.restore();
    assert_eq!(found, vec![here.clone()], "the read names the repository whose push waits");
    assert!(!retries.needs_restore(), "the journal was read");
    assert!(retries.next_wait(here.as_path(), Instant::now()).is_some(), "the push the earlier service promised a retry for is due again");
    assert_eq!(retries.directories(), vec![here], "the loop has the repository to drain for it");
    crate::test_support::remove(root.as_path());
}
