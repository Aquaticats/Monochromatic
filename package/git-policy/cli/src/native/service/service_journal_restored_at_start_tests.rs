//! Controls for the journal read at a service's start:
//! a read the control holds gives no answer within the ceiling,
//! so the service starts with the fallback that keeps both directories and owes both reads again,
//! and the read it owes is made again and tries the push it finds;
//! a journal that could not be listed is read again the same way;
//! and a plan's name that holds a pipe is answered at once and left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [reached, letGo] = holdThePoint(THE_READ_AT_A_START); const [retries, watch] = restoredWithin(100, plan, watchDir, undefined);
//! ```

use super::*;
use std::os::unix::fs::FileTypeExt as _;
use std::time::Instant;

/// How long the controls wait for anything:
/// the start of the service to answer,
/// the held read to say it was reached,
/// and the pipe's read to answer.
/// Why:
///   A read the control holds answers nothing,
/// so a wait for the start is not a margin over what a read of two directories costs on this machine,
/// and a wait that reaches this bound has found a break rather than a slow disk.
const PATIENCE: Duration = Duration::from_secs(60);

/// The ceiling the journal's controls give its read.
/// Why:
///   The held control's read never answers inside a ceiling,
/// so this value only decides how long that control takes and nothing in it depends on the machine's speed;
/// the other control's journal answers its listing with an error at once,
/// so no ceiling decides what the start makes of it either.
const SHORT_CEILING: Duration = Duration::from_millis(100);

/// The ceiling the pipe's control gives its read.
/// Why:
///   The ceiling is not what that control judges,
/// and a read it gave a short one would fail it on a slow disk:
/// it judges that a pipe at a plan's name is answered rather than waited on,
/// and a read that waited inside `open(2)` answers at this ceiling instead.
const A_CEILING_NO_READ_REACHES: Duration = Duration::from_secs(3_600);

/// What:
///  Start the read of the journal on a thread of its own and give its answer,
/// or nothing when it gave none within `PATIENCE`.
/// Why:
///   The start waits for its read no longer than its ceiling,
/// so the control waits for the start here rather than where it stands,
/// and a start that never answers is a failure this bound names in its own words
/// instead of holding the suite to nextest's bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [retries, watch] = startedWithin(PATIENCE, () => restoredWithin(ceiling, plan, watchDir, instance));
/// ```
fn started_within(ceiling: Duration, plan: &std::path::Path, watch: &std::path::Path, instance: Option<String>) -> Option<(PushRetries, ClaimWatch)> {
    let (plan_of_the_read, watch_of_the_read): (std::path::PathBuf, std::path::PathBuf) = (plan.to_path_buf(), watch.to_path_buf());
    return crate::test_support::finished_within_at(PATIENCE, move || { return restored_within(ceiling, plan_of_the_read, watch_of_the_read, instance); });
}

/// What:
///  Whether the held read said it reached the point the control holds,
/// and nothing when it never did.
/// Why:
///   A hold is taken by the thread that reaches it,
/// so this is the proof that the read whose answer the control judges is the read it holds and not a read that finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reached = await reachedWithin(PATIENCE, reachedWait);
/// ```
fn the_read_was_reached(reached: tokio::sync::oneshot::Receiver<()>) -> Option<bool> {
    return crate::test_support::finished_within_at(PATIENCE, move || { return reached.blocking_recv().is_ok(); });
}

/// What:
///  A read of the journal that gives no answer within its ceiling leaves the service starting without it:
/// both directories are kept,
/// both reads are owed again,
/// and no push is planned yet;
/// the reads it owes are then made again and the push an earlier service left is tried.
/// Why:
///  "nothing should ever wait forever or indefinitely":
/// a journal on a mount that stopped answering must not hold the start of a service,
/// and what the service promises afterwards must not be lost with that read.
/// A plan this service makes is written where the next start reads it,
/// and a repository an earlier service was woken for is looked at only by the read made again,
/// so neither directory may go with the answer that never came
/// (`SPEC.md`,
/// "the plan's directory and the watch's directory are kept either way",
/// and "a read that gave no answer is made again rather than given up on").
/// The read is held at its own fault point,
/// so "gives no answer" is a state of the code and not a duration.
#[test]
fn a_read_that_gives_no_answer_within_its_ceiling_leaves_both_directories_and_reads_them_again() {
    let root: std::path::PathBuf = crate::test_support::fixture("journal-restore-held");
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
    let (reached, let_go) = crate::service_fault::hooks::hold_the_point(THE_READ_AT_A_START);
    let started: Option<(PushRetries, ClaimWatch)> = started_within(SHORT_CEILING, plan.as_path(), watch.as_path(), Some(String::from("instance")));
    assert_eq!(the_read_was_reached(reached), Some(true), "the journal's read reached the point this control holds, so what the start answered is its ceiling and not a read that finished");
    let (mut retries, mut notes): (PushRetries, ClaimWatch) = started.expect("the start of a service whose journal gives no answer answers at its ceiling, so the service starts without its journal");
    assert!(retries.directories().is_empty(), "the held read found no push, so this service has no push to retry yet");
    assert_eq!(retries.plans(), Some(plan.as_path()), "the plan's directory is kept, so a push this service promises is written where the next start reads it");
    assert!(retries.needs_restore(), "the plan the start could not read is read again after a pause");
    assert_eq!(notes.key(), Some(watch.clone()), "the notes' directory is kept too, so the repositories this service looks at are kept for the next start");
    assert!(notes.needs_retry(), "the notes the start could not read are read again after a pause");
    // The read is let go, so the thread the ceiling left behind ends instead of standing in the kernel for the life of the test
    //  process, and the reads this service owes are made again: the push the earlier service left is due, and a notes directory
    //  with no note owes nothing more.
    let _let_go = let_go.send(());
    let found: Vec<std::path::PathBuf> = retries.restore();
    assert_eq!(found, vec![here.clone()], "the read names the repository whose push waits");
    assert!(!retries.needs_restore(), "the journal was read");
    assert!(retries.next_wait(here.as_path(), Instant::now()).is_some(), "the push the earlier service promised a retry for is due again");
    assert_eq!(retries.directories(), vec![here], "the loop has the repository to drain for it");
    let _nothing_found: Vec<std::path::PathBuf> = notes.retry();
    assert!(!notes.needs_retry(), "the notes were read again and hold no note, so nothing of them is owed");
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
    let started: Option<(PushRetries, ClaimWatch)> = started_within(SHORT_CEILING, plan.as_path(), watch.as_path(), None);
    let (mut retries, _notes): (PushRetries, ClaimWatch) = started.expect("a journal that could not be listed answers the start, which keeps what it read");
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
///  so a read that answered while it stood there is a read that did not wait inside `open(2)`,
/// and the ceiling this control gives the read is past any read a disk could make.
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
    let started: Option<(PushRetries, ClaimWatch)> = started_within(A_CEILING_NO_READ_REACHES, plan.as_path(), watch.as_path(), Some(String::from("instance")));
    let (retries, notes): (PushRetries, ClaimWatch) = started.expect("a pipe at a plan's name is answered rather than waited on, so the start answers");
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
