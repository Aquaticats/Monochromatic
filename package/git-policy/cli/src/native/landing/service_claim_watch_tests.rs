//! Controls for the Git directories a restarted service looks at by itself.

use super::*;
use crate::service_timing::capture::Captured;
use std::ffi::OsString;
use std::os::unix::fs::FileTypeExt;

/// The first pause of the controls.
const FIRST: Duration = Duration::from_secs(10);

/// How long an answer that must not wait may take in a control.
///
/// Why:
///  A read of a name that holds a named pipe waits inside `open(2)` for a writer that may never come,
///  so a control that made that read on its own thread would never end and would report nothing;
///  every control that reads a note waits for the answer on a thread of its own instead.
const PATIENCE: Duration = Duration::from_secs(20);

/// The answer of `work`,
/// run on a thread of its own,
/// or nothing when it gave no answer within `patience`.
///
/// Why:
///  The bound is the control's own,
/// so a control that caught a read waiting on a pipe ends and says so.
///  The thread is left behind at the patience:
/// a read that the kernel never returns is ended only by the
///  process ending,
/// and waiting for that thread here would hang the control it belongs to.
fn answered_within<Value: Send + 'static>(patience: Duration, work: impl FnOnce() -> Value + Send + 'static) -> Option<Value> {
    let (sender, receiver): (std::sync::mpsc::Sender<Value>, std::sync::mpsc::Receiver<Value>) = std::sync::mpsc::channel::<Value>();
    let started = std::thread::Builder::new().name(String::from("claim-watch-control")).spawn(move || {
        // A control that stopped waiting is gone with its receiver, and the answer goes with the send.
        let _sent = sender.send(work());
    });
    if started.is_err() {
        return None;
    }
    return receiver.recv_timeout(patience).ok();
}

/// The watch a read of `store` made,
/// awaited no longer than `PATIENCE`,
/// and the lines that read wrote on the thread it ran on.
///
/// Why:
///  The lines are captured on the thread that wrote them,
///  so the read and its capture are one piece of work handed to `answered_within`.
fn restored_within(store: &Path) -> Option<(ClaimWatch, Vec<Captured>)> {
    let asked: PathBuf = store.to_path_buf();
    return answered_within(PATIENCE, move || {
        let mut held: Option<ClaimWatch> = None;
        let entries: Vec<Captured> = crate::service_timing::capture::entries_of(|| {
            held = Some(ClaimWatch::restored(asked.as_path(), None));
        });
        return (held.expect("the read answered"), entries);
    });
}

/// The warning lines of `entries` that name `note`.
fn warnings_about(entries: &[Captured], note: &Path) -> Vec<String> {
    let named: String = note.display().to_string();
    return entries
        .iter()
        .filter(|entry| {
            return entry.priority == crate::journald_native::Priority::Warning && entry.message.contains(named.as_str());
        })
        .map(|entry| {
            return entry.message.clone();
        })
        .collect();
}

/// A scratch directory under the package's own build directory,
/// removed by the caller.
fn scratch(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("claim-watch-tests").join(format!("{name}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    return directory;
}

/// A Git directory with the work directory the wrapper makes.
fn git_dir_with_work(root: &Path, name: &str) -> PathBuf {
    let git_dir: PathBuf = root.join(name);
    std::fs::create_dir_all(git_dir.join(WORK_DIRECTORY)).expect("the work directory");
    return git_dir;
}

/// The files of a store,
/// by name.
fn files_of(store: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(store).map_or_else(|_| { return Vec::new(); }, |listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); });
    names.sort();
    return names;
}

/// A directory written down by one watch is read back by the watch of the service that starts next.
#[test]
fn a_remembered_directory_is_restored_by_the_next_watch() {
    let root: PathBuf = scratch("restored");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut first: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    assert!(first.directories().is_empty(), "a new journal watches nothing");
    first.remember(git_dir.as_path());
    first.remember(git_dir.as_path());
    assert_eq!(first.directories(), vec![git_dir.clone()], "remembered once");
    assert_eq!(files_of(store.as_path()).len(), 1, "one file for one directory");
    assert!(!first.needs_retry());
    assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// Several directories are each written down and each restored.
#[test]
fn every_directory_has_its_own_entry() {
    let root: PathBuf = scratch("several");
    let one: PathBuf = git_dir_with_work(root.as_path(), "one.git");
    let two: PathBuf = git_dir_with_work(root.as_path(), "two.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(one.as_path());
    watch.remember(two.as_path());
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path(), None).directories();
    restored.sort();
    assert_eq!(restored, vec![one, two]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A directory that lost its work directory,
/// a file that holds no path,
/// and a work directory that is a link are certainly of no
/// use:
/// their files are removed,
/// so a removed worktree is not looked at at every start.
/// A file of another kind stays.
#[test]
fn files_that_are_certainly_of_no_use_are_removed() {
    let root: PathBuf = scratch("dropped");
    let gone: PathBuf = git_dir_with_work(root.as_path(), "gone.git");
    let linked: PathBuf = root.join("linked.git");
    std::fs::create_dir_all(linked.join("monochromatic-git-policy-cli")).expect("the parent of the link");
    std::os::unix::fs::symlink(gone.join(WORK_DIRECTORY), linked.join(WORK_DIRECTORY)).expect("the link");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(gone.as_path());
    watch.remember(linked.as_path());
    std::fs::write(store.join("junk.watch"), "relative/path\n").expect("a file that names no absolute path");
    std::fs::write(store.join("other.txt"), "left alone\n").expect("a file of another kind");
    std::fs::remove_dir_all(&gone).expect("the worktree is removed");
    let restored: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    assert!(restored.directories().is_empty());
    assert!(!restored.needs_retry(), "nothing failed to be read");
    assert_eq!(files_of(store.as_path()), vec![String::from("other.txt")], "only the file of another kind stays");
    let _removed = std::fs::remove_dir_all(&root);
}

/// A watch kept in memory only writes nothing and still knows what it was told.
#[test]
fn an_in_memory_watch_writes_nothing() {
    let root: PathBuf = scratch("memory");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let mut watch: ClaimWatch = ClaimWatch::in_memory();
    watch.remember(git_dir.as_path());
    assert_eq!(watch.directories(), vec![git_dir]);
    assert!(!watch.needs_retry());
    assert!(!root.join(WATCH_DIRECTORY).exists());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A file that cannot be written is not lost:
/// the watch keeps the directory,
/// asks to be tried again,
/// and a try after the fault is
/// mended writes it and hands the directory back,
/// so that a service killed afterwards finds it.
#[test]
fn a_file_that_could_not_be_written_is_written_by_a_later_try() {
    let root: PathBuf = scratch("unwritable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    // A file where the store's directory should be makes the write fail.
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(&blocker, "not a directory\n").expect("the blocker");
    let store: PathBuf = blocker.join("store");
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(git_dir.as_path());
    assert!(watch.directories().is_empty(), "not written, so not yet known to the next service");
    assert!(watch.needs_retry());
    assert!(watch.retry().is_empty(), "the fault is still there");
    assert!(watch.needs_retry());
    assert!(watch.pause(FIRST) > FIRST, "the pause grows with the failures in a row: {:?}", watch.pause(FIRST));
    std::fs::remove_file(&blocker).expect("the fault is mended");
    assert!(watch.retry().is_empty(), "the directory was this process's own, so it is not new to the caller");
    assert!(!watch.needs_retry());
    assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir], "the next service finds it");
    let _removed = std::fs::remove_dir_all(&root);
}

/// A note that cannot be read is kept,
/// not removed,
/// and read again once the fault is over;
/// the watch asks to be tried again
/// meanwhile,
/// so that the service neither forgets the repository nor exits idle.
#[test]
fn a_note_that_could_not_be_read_is_kept_and_found_by_a_later_try() {
    let root: PathBuf = scratch("unreadable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    ClaimWatch::restored(store.as_path(), None).remember(git_dir.as_path());
    let refuse = |_: &Path| -> Result<Vec<u8>, ReadRefusal> { return Err(ReadRefusal::Io(std::io::Error::from(std::io::ErrorKind::PermissionDenied))); };
    let mut watch: ClaimWatch = ClaimWatch::restored_reading(store.as_path(), None, &refuse);
    assert!(watch.directories().is_empty());
    assert!(watch.needs_retry(), "a refused read is tried again");
    assert_eq!(files_of(store.as_path()).len(), 1, "the note stays");
    assert!(watch.retry_reading(&refuse).is_empty());
    assert!(watch.needs_retry());
    assert_eq!(watch.retry(), vec![git_dir.clone()], "the read that works hands the directory over");
    assert!(!watch.needs_retry());
    assert_eq!(watch.directories(), vec![git_dir]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A store that cannot be listed is tried again;
/// one that does not exist is a journal that never held a note.
#[test]
fn a_store_that_cannot_be_listed_is_tried_again() {
    let root: PathBuf = scratch("unlistable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(&blocker, "not a directory\n").expect("the blocker");
    let mut watch: ClaimWatch = ClaimWatch::restored(blocker.join("store").as_path(), None);
    assert!(watch.needs_retry(), "a store under a file cannot be listed");
    assert!(!ClaimWatch::restored(root.join("never-written").as_path(), None).needs_retry(), "a store that is not there holds nothing");
    std::fs::remove_file(&blocker).expect("the fault is mended");
    std::fs::create_dir_all(blocker.join("store")).expect("the store");
    std::fs::write(blocker.join("store").join(watch_name(git_dir.as_path(), 0)), encode(git_dir.as_path())).expect("a note");
    assert_eq!(watch.retry(), vec![git_dir], "listed again, the note is found");
    assert!(!watch.needs_retry());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A note that names a directory of another instance is not this instance's to look at:
/// it is removed,
/// and a directory of another
/// instance is not written down.
#[test]
fn a_directory_of_another_instance_is_neither_kept_nor_written() {
    let root: PathBuf = scratch("foreign");
    let mine: PathBuf = git_dir_with_work(root.as_path(), "mine.git");
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let instance: String = instance_of(mine.as_path());
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), Some(instance.as_str()));
    watch.remember(other.as_path());
    watch.remember(mine.as_path());
    assert_eq!(watch.directories(), vec![mine.clone()]);
    assert_eq!(files_of(store.as_path()).len(), 1, "only this instance's directory is written");
    // A note another instance's directory was put into (a copied journal) is removed at the start.
    std::fs::write(store.join("copied.watch"), encode(other.as_path())).expect("a copied note");
    let restored: ClaimWatch = ClaimWatch::restored(store.as_path(), Some(instance.as_str()));
    assert_eq!(restored.directories(), vec![mine]);
    assert_eq!(files_of(store.as_path()).len(), 1, "the copied note is gone");
    let _removed = std::fs::remove_dir_all(&root);
}

/// A path that was redirected through a link to another repository names another instance,
/// so the note that holds it is removed
/// and the other repository is not looked at.
#[test]
fn a_path_redirected_to_another_repository_is_not_followed() {
    let root: PathBuf = scratch("redirected");
    let mine: PathBuf = git_dir_with_work(root.as_path(), "mine.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let instance: String = instance_of(mine.as_path());
    ClaimWatch::restored(store.as_path(), Some(instance.as_str())).remember(mine.as_path());
    // The path is replaced by a link to another repository that has a work directory of its own.
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    std::fs::remove_dir_all(&mine).expect("the repository is removed");
    std::os::unix::fs::symlink(&other, &mine).expect("the path now leads to another repository");
    let restored: ClaimWatch = ClaimWatch::restored(store.as_path(), Some(instance.as_str()));
    assert!(restored.directories().is_empty(), "the other repository is not this instance's");
    assert!(files_of(store.as_path()).is_empty());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A path that is not text,
/// or that ends in a newline,
/// is kept exactly and found again.
#[test]
fn a_path_is_kept_byte_for_byte() {
    let root: PathBuf = scratch("bytes");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let odd: PathBuf = root.join(OsString::from_vec(vec![b'o', 0xff, b'd', b'\n']));
    std::fs::create_dir_all(odd.join(WORK_DIRECTORY)).expect("a directory with a name that is not text and ends in a newline");
    let plain: PathBuf = root.join("od");
    std::fs::create_dir_all(plain.join(WORK_DIRECTORY)).expect("a directory whose name a lossy copy of the odd one would spell");
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(odd.as_path());
    watch.remember(plain.as_path());
    assert_eq!(files_of(store.as_path()).len(), 2, "two different paths are two notes");
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path(), None).directories();
    restored.sort();
    let mut expected: Vec<PathBuf> = vec![odd, plain];
    expected.sort();
    assert_eq!(restored, expected);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A note that holds another directory's path under the name this directory would take is left alone and the next name is used;
/// a damaged note under the name is replaced.
#[test]
fn a_taken_name_is_checked_not_trusted() {
    let root: PathBuf = scratch("taken");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    std::fs::create_dir_all(&store).expect("the store");
    // Another directory's whole note sits under the first name of this one.
    std::fs::write(store.join(watch_name(git_dir.as_path(), 0)), encode(other.as_path())).expect("a note of another directory");
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(git_dir.as_path());
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path(), None).directories();
    restored.sort();
    let mut expected: Vec<PathBuf> = vec![git_dir.clone(), other.clone()];
    expected.sort();
    assert_eq!(restored, expected, "both are kept");
    // A torn note under the first name of a third directory is replaced.
    let third: PathBuf = git_dir_with_work(root.as_path(), "third.git");
    let mut again: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    std::fs::write(store.join(watch_name(third.as_path(), 0)), "claim-watch-1 2f7468").expect("a note that was cut short, left after the scan");
    again.remember(third.as_path());
    assert!(ClaimWatch::restored(store.as_path(), None).directories().contains(&third), "the cut note was replaced");
    let _removed = std::fs::remove_dir_all(&root);
}

/// What:
///  A repository the landing wrote down is looked at by the service that starts after it,
/// and one of another instance is not.
/// Why:
///   This is the whole of the promise that a kill is followed by the work being run with no commit needed:
/// the
///  wrapper's wake is what would have written the note,
/// and a wrapper killed between promoting a push and that wake
///  leaves the note the landing wrote instead.
#[test]
fn a_note_written_by_the_landing_is_looked_at_by_the_next_service() {
    let root: PathBuf = scratch("noted-by-the-landing");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let instance: String = instance_of(git_dir.as_path());
    // The landing knows the repository and its instance; no wake has happened for it.
    note_for_the_next_service(store.as_path(), Some(instance.as_str()), git_dir.as_path());
    assert_eq!(ClaimWatch::restored(store.as_path(), Some(instance.as_str())).directories(), vec![git_dir.clone()], "the next service looks at it");
    // A note of another instance's repository is neither written nor read, so the file stays one.
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    note_for_the_next_service(store.as_path(), Some(instance.as_str()), other.as_path());
    assert_eq!(files_of(store.as_path()).len(), 1, "another instance's repository is not written down");
    let _removed = std::fs::remove_dir_all(&root);
}

/// What:
///  A note whose name holds a named pipe with no writer is answered at once,
///  said once,
///  and left where it is.
/// Why:
///  A plain read of a named pipe waits inside `open(2)` for a writer that may never come,
///  and all three callers of this read answer a person:
///  the start of the service,
///  a retry of the watch,
///  and the note a landing writes.
///  So a note is read the way every record of this wrapper is read
///  (`private_storage::read_regular_file`):
///  a name holding no regular file is no note this store wrote,
///  and a second read of it would answer the same,
///  so the watch neither removes it nor asks to be tried again.
#[test]
fn a_note_that_is_a_named_pipe_is_answered_at_once_and_left_where_it_is() {
    let root: PathBuf = scratch("pipe");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    std::fs::create_dir_all(store.as_path()).expect("the store");
    let stalled: PathBuf = store.join(format!("stalled.{WATCH_EXTENSION}"));
    crate::subvolume_sync_attrs::make_fifo(stalled.as_path(), 0o600).expect("a named pipe at a note's name");
    let Some((watch, entries)): Option<(ClaimWatch, Vec<Captured>)> = restored_within(store.as_path()) else {
        let _removed = std::fs::remove_dir_all(&root);
        panic!("the read of a note whose name is a named pipe gave no answer within {PATIENCE:?}: it waited for a writer that may never come");
    };
    let left_where_it_was: bool = std::fs::symlink_metadata(stalled.as_path()).is_ok_and(|found| {
        return found.file_type().is_fifo();
    });
    let _removed = std::fs::remove_dir_all(&root);
    assert_eq!(warnings_about(entries.as_slice(), stalled.as_path()).len(), 1, "the note is said once and named: {entries:?}");
    assert!(left_where_it_was, "the pipe is left where it is");
    assert!(!watch.needs_retry(), "a name that holds no note is not read again after a pause");
    assert!(watch.directories().is_empty(), "a pipe names no repository to look at");
}

/// What:
///  A note whose name is a link is not read through:
///  the name is said once and left where it is,
///  and the file it points at is neither adopted nor removed.
/// Why:
///  Records are files this store wrote itself,
///  a link at one of their names is not this store's,
///  and reading through it makes a directory the wrapper never watched a repository the service looks at.
#[test]
fn a_note_that_is_a_link_is_not_read_through() {
    let root: PathBuf = scratch("link");
    let linked: PathBuf = git_dir_with_work(root.as_path(), "linked.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    std::fs::create_dir_all(store.as_path()).expect("the store");
    let elsewhere: PathBuf = root.join("elsewhere.note");
    std::fs::write(&elsewhere, encode(linked.as_path())).expect("a note under another name");
    let redirected: PathBuf = store.join(format!("redirected.{WATCH_EXTENSION}"));
    std::os::unix::fs::symlink(&elsewhere, &redirected).expect("a link where a note belongs");
    let Some((watch, entries)): Option<(ClaimWatch, Vec<Captured>)> = restored_within(store.as_path()) else {
        let _removed = std::fs::remove_dir_all(&root);
        panic!("the read of a note whose name is a link gave no answer within {PATIENCE:?}");
    };
    let left_where_it_was: bool = std::fs::symlink_metadata(redirected.as_path()).is_ok_and(|found| {
        return found.file_type().is_symlink();
    });
    let target_stays: bool = elsewhere.exists();
    let _removed = std::fs::remove_dir_all(&root);
    assert!(watch.directories().is_empty(), "a link names no repository any more than a pipe does: {:?}", watch.directories());
    assert_eq!(warnings_about(entries.as_slice(), redirected.as_path()).len(), 1, "the note is said once and named: {entries:?}");
    assert!(!watch.needs_retry());
    assert!(left_where_it_was, "the link is left where it is");
    assert!(target_stays, "the file it points at is left where it is");
}

/// What:
///  A note whose name is a directory is answered at once,
///  said once and left where it is,
///  and the watch does not ask to be tried again.
/// Why:
///  A directory is no note this store wrote,
///  and a read of it fails for as long as it stands there,
///  so reading it again after a pause would say the same at every pause and keep the service from exiting idle.
#[test]
fn a_note_that_is_a_directory_is_answered_once_and_left_where_it_is() {
    let root: PathBuf = scratch("directory");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let stalled: PathBuf = store.join(format!("stalled.{WATCH_EXTENSION}"));
    std::fs::create_dir_all(stalled.as_path()).expect("a directory at a note's name");
    let Some((watch, entries)): Option<(ClaimWatch, Vec<Captured>)> = restored_within(store.as_path()) else {
        let _removed = std::fs::remove_dir_all(&root);
        panic!("the read of a note whose name is a directory gave no answer within {PATIENCE:?}");
    };
    let left_where_it_was: bool = stalled.is_dir();
    let _removed = std::fs::remove_dir_all(&root);
    assert_eq!(warnings_about(entries.as_slice(), stalled.as_path()).len(), 1, "the note is said once and named: {entries:?}");
    assert!(!watch.needs_retry(), "a name that holds no note is not read again after a pause");
    assert!(left_where_it_was, "the directory is left where it is");
}

/// What:
///  A note whose name holds more bytes than a note can is answered at once,
///  said once and left where it is.
/// Why:
///  A note is the format words and one path's bytes in hexadecimal,
///  so a longer file is no note this store wrote:
///  reading it whole would cost the service memory for nothing,
///  and removing it would delete a file this store did not write.
#[test]
fn a_note_that_holds_more_than_a_note_can_is_left_where_it_is() {
    let root: PathBuf = scratch("oversized");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    std::fs::create_dir_all(store.as_path()).expect("the store");
    let oversized: PathBuf = store.join(format!("oversized.{WATCH_EXTENSION}"));
    let longer_than_a_note: usize = usize::try_from(NOTE_FILE_LIMIT + 1).expect("the limit fits this system's memory");
    std::fs::write(&oversized, vec![b'x'; longer_than_a_note]).expect("a file longer than a note can be");
    let Some((watch, entries)): Option<(ClaimWatch, Vec<Captured>)> = restored_within(store.as_path()) else {
        let _removed = std::fs::remove_dir_all(&root);
        panic!("the read of a file longer than a note can be gave no answer within {PATIENCE:?}");
    };
    let left_where_it_was: bool = oversized.is_file();
    let _removed = std::fs::remove_dir_all(&root);
    assert_eq!(warnings_about(entries.as_slice(), oversized.as_path()).len(), 1, "the note is said once and named: {entries:?}");
    assert!(!watch.needs_retry(), "a name that holds no note is not read again after a pause");
    assert!(left_where_it_was, "a file longer than a note can be is not removed");
}

/// What:
///  The note a landing writes for a repository answers at once when a named pipe holds the first name it would use,
///  the pipe is left where it is,
///  and the note is written under the next name,
///  so the next service finds the repository.
/// Why:
///  `note_for_the_next_service` runs on the landing thread,
///  which is the only thread that lands and answers,
///  so a read of a pipe there is a commit that is never answered and every later commit behind it,
///  not a delay the after-commit work can wait out.
#[test]
fn a_note_written_where_a_named_pipe_holds_the_first_name_answers_at_once() {
    let root: PathBuf = scratch("write-pipe");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    std::fs::create_dir_all(store.as_path()).expect("the store");
    let taken: PathBuf = store.join(watch_name(git_dir.as_path(), 0));
    crate::subvolume_sync_attrs::make_fifo(taken.as_path(), 0o600).expect("a named pipe at the note's first name");
    let (asked_store, asked_git_dir): (PathBuf, PathBuf) = (store.clone(), git_dir.clone());
    let answered: Option<()> = answered_within(PATIENCE, move || {
        note_for_the_next_service(asked_store.as_path(), None, asked_git_dir.as_path());
    });
    let left_where_it_was: bool = std::fs::symlink_metadata(taken.as_path()).is_ok_and(|found| {
        return found.file_type().is_fifo();
    });
    let names: Vec<String> = files_of(store.as_path());
    let found_again: Option<(ClaimWatch, Vec<Captured>)> = restored_within(store.as_path());
    let _removed = std::fs::remove_dir_all(&root);
    assert!(answered.is_some(), "the write of a note waited on a named pipe at the name it would use, so a commit on the landing thread was never answered");
    assert!(left_where_it_was, "the pipe is left where it is");
    assert!(names.contains(&watch_name(git_dir.as_path(), 1)), "the note is written under the next name: {names:?}");
    let Some((watch, _entries)): Option<(ClaimWatch, Vec<Captured>)> = found_again else {
        panic!("the next service's read of the store gave no answer within {PATIENCE:?}");
    };
    assert_eq!(watch.directories(), vec![git_dir], "the next service looks at the repository the landing wrote down");
}
