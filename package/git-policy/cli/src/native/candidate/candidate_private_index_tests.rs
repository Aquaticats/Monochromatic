//! What:
//!  Controls for the private index copy:
//!  where it is made,
//!  what it holds,
//!  its times,
//!       its permissions,
//!  and that it is removed on every path.
//! Why:
//!  A copy with a fresh time would let a prediction miss a same-second edit,
//!  a copy in
//!      a shared directory would expose staged entries,
//!  and a leftover directory would
//!      accumulate in the Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // { using index = createPrivateIndex(real); expect(read(index.path)).toEqual(read(real)); } expect(exists(dir)).toBe(false);
//! ```
#![cfg(unix)]

/// Import the type under test and its name prefix.
use super::{PRIVATE_INDEX_PREFIX, PrivateIndex, STAGING_SUFFIX};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The entries of a directory,
///  sorted by name.
fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("readable directory") {
        names.push(
            entry
                .expect("directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    return names;
}

/// How long the creating thread is given to answer,
///  so that a wait which never answers fails the control instead of hanging it.
/// Why:
///   The wait is a bet on the host,
///  and the only figure this repository has measured for one is how long a test process
///  of this crate went without showing any work: 8.0 s,
///  on this host,
///  which is why the watchdog's window is fifteen times it
///  (`QUIET_WINDOW` in `testing/test_support.rs`).
///  The ten seconds this control bet before was 1.25 times that figure,
///  inside the spread it describes,
///  so this is the owner's own "3x worst case"
///  (`ANNOUNCEMENT_HOLD` uses the same multiple for a holder that is only slow)
///  and still a fifth of the watchdog's window,
///  so a product that waits on the pipe is still reported by the control rather than by the watchdog.
const COPY_ANSWER_MEASURED_QUIET_STRETCH: Duration = Duration::from_secs(8);

/// The multiple of the measured quiet stretch the control allows,
///  the owner's "3x worst case".
const COPY_ANSWER_SLACK: u32 = 3;

/// The longest the creating thread may take, in all.
const THE_COPY_ANSWERS_WITHIN: Duration = COPY_ANSWER_MEASURED_QUIET_STRETCH.saturating_mul(COPY_ANSWER_SLACK);

/// The failure `create` returned,
///  which the control requires.
fn creation_failure(real_index: &Path) -> CandidateError {
    match PrivateIndex::create(real_index) {
        Ok(created) => panic!("expected a failure, got {created:?}"),
        Err(error) => return error,
    }
}

/// The copy sits in a fresh private directory beside the real index,
///  holds its bytes and
/// times,
///  and disappears with everything in it when dropped.
#[test]
fn the_copy_keeps_bytes_and_times_and_is_removed_on_drop() {
    let root: PathBuf = fixture("private-index-copy");
    let real: PathBuf = root.join("index");
    std::fs::write(&real, b"DIRC index bytes\0\xff").expect("real index");
    let accessed: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    let modified: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_100_000_000);
    std::fs::File::options()
        .write(true)
        .open(&real)
        .expect("open real index")
        .set_times(
            std::fs::FileTimes::new()
                .set_accessed(accessed)
                .set_modified(modified),
        )
        .expect("old times");
    let private: PrivateIndex = PrivateIndex::create(real.as_path()).expect("private index");
    let directory: PathBuf = private.path().parent().expect("directory").to_path_buf();
    assert_eq!(
        directory.parent(),
        Some(root.as_path()),
        "beside the real index"
    );
    let name: String = directory
        .file_name()
        .expect("name")
        .to_string_lossy()
        .into_owned();
    assert!(name.starts_with(PRIVATE_INDEX_PREFIX), "{name}");
    assert_eq!(private.path().file_name(), Some("index".as_ref()));
    // The times are read before the content, because reading may update the access time.
    let copied: std::fs::Metadata = std::fs::metadata(private.path()).expect("copy metadata");
    assert_eq!(copied.modified().expect("modified"), modified);
    assert_eq!(copied.accessed().expect("accessed"), accessed);
    assert_eq!(
        std::fs::read(private.path()).expect("copy"),
        b"DIRC index bytes\0\xff".to_vec()
    );
    let mode: u32 = std::fs::metadata(&directory)
        .expect("directory metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o700, "only the owner may enter");
    // A second copy made right after gets its own directory.
    let second: PrivateIndex = PrivateIndex::create(real.as_path()).expect("second copy");
    assert_ne!(second.path(), private.path());
    // Whatever Git writes into the directory goes with it.
    std::fs::write(directory.join("index.lock"), b"lock").expect("extra file");
    drop(private);
    drop(second);
    assert_eq!(entries(root.as_path()), vec!["index"]);
    remove(root.as_path());
}

/// A repository without an index file gets an empty directory and a private path Git reads as empty.
#[test]
fn a_missing_real_index_needs_no_copy() {
    let root: PathBuf = fixture("private-index-missing");
    let private: PrivateIndex =
        PrivateIndex::create(root.join("index").as_path()).expect("private index");
    assert!(!private.path().exists(), "nothing is copied");
    assert!(private.path().parent().expect("directory").is_dir());
    drop(private);
    assert_eq!(entries(root.as_path()), Vec::<String>::new());
    remove(root.as_path());
}

/// Every step that cannot be done is a private-index failure,
///  and no directory is left behind.
#[test]
fn failures_leave_nothing_behind() {
    let root: PathBuf = fixture("private-index-failures");
    let rootless: CandidateError = creation_failure(Path::new("/"));
    assert_eq!(rootless.failure, CandidateFailure::PrivateIndexUnavailable);
    assert!(
        rootless.message.contains("without a directory"),
        "{}",
        rootless.message
    );
    let no_parent: CandidateError = creation_failure(root.join("missing/index").as_path());
    assert_eq!(no_parent.failure, CandidateFailure::PrivateIndexUnavailable);
    assert!(
        no_parent.message.contains("creating its directory"),
        "{}",
        no_parent.message
    );
    // A real index that is a directory cannot be copied; the fresh directory is removed again.
    std::fs::create_dir(root.join("index")).expect("directory in place of the index");
    let not_a_file: CandidateError = creation_failure(root.join("index").as_path());
    assert_eq!(
        not_a_file.failure,
        CandidateFailure::PrivateIndexUnavailable
    );
    assert!(
        not_a_file.message.contains("copying the index"),
        "{}",
        not_a_file.message
    );
    assert_eq!(entries(root.as_path()), vec!["index"]);
    // An index path that cannot be inspected is a failure, not a missing index.
    std::os::unix::fs::symlink("loop", root.join("loop")).expect("link to itself");
    let looping: CandidateError = creation_failure(root.join("loop").as_path());
    assert_eq!(looping.failure, CandidateFailure::PrivateIndexUnavailable);
    assert!(
        looping.message.contains("copying the index"),
        "{}",
        looping.message
    );
    assert_eq!(entries(root.as_path()), vec!["index", "loop"]);
    remove(root.as_path());
}

/// What:
///  A copy whose owner is proven gone is removed by the next copy,
///  a copy this process still
///       holds is kept,
///  and a copy nothing proves gone is kept.
/// Why:
///  A killed check leaves its copy in the Git directory for ever (39 of them were counted
///       from July to October in the native cutover's measurement),
///  and the copy holds the repository's
///       index,
///  so they fill the disk until commits fail.
///  Nothing may be removed on anything less than a
///       proof that the owner is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // writeAbandonedCopy(deadPid); createPrivateIndex(); expect(exists(abandoned)).toBe(false)
/// ```
#[test]
fn a_copy_whose_owner_is_gone_is_removed_and_a_live_one_is_kept() {
    let root: PathBuf = fixture("private-index-reap");
    let real: PathBuf = root.join("index");
    std::fs::write(&real, b"DIRC").expect("real index");
    // A process id no process can have (above this host's `pid_max`), so no reuse can make it look alive.
    let dead: i64 = 4_194_304;
    let abandoned: PathBuf = root.join(format!("{PRIVATE_INDEX_PREFIX}{dead}-1-0"));
    std::fs::create_dir(&abandoned).expect("abandoned directory");
    std::fs::write(abandoned.join("index"), b"stale copy").expect("stale copy");
    let owner: crate::worktree_copy_owner::CopyOwner = crate::worktree_copy_owner::CopyOwner {
        pid: dead,
        birth_identity: String::from("0"),
        pid_namespace: crate::pid_namespace::current_pid_namespace(),
    };
    std::fs::write(abandoned.join("owner"), owner.token()).expect("owner record");
    // A copy from before owner records existed names its owner only in its own name, and a name proves nothing
    // across PID namespaces, so it stays for the person to remove.
    let old: PathBuf = root.join(format!("{PRIVATE_INDEX_PREFIX}{dead}-2-0"));
    std::fs::create_dir(&old).expect("old directory");
    std::fs::write(old.join("index"), b"stale copy").expect("stale copy");
    // A copy still being built under its staging name keeps its bytes until it is published.
    let staging: PathBuf = root.join(format!("{PRIVATE_INDEX_PREFIX}{dead}-4-0{STAGING_SUFFIX}"));
    std::fs::create_dir(&staging).expect("staging directory");
    std::fs::write(staging.join("index"), b"half a copy").expect("staging copy");
    // A copy whose owner record is unreadable is not proven gone.
    let unreadable: PathBuf = root.join(format!("{PRIVATE_INDEX_PREFIX}{dead}-3-0"));
    std::fs::create_dir(&unreadable).expect("unreadable directory");
    std::fs::create_dir(unreadable.join("owner")).expect("owner record that is a directory");
    // The copy this process holds right now must never be taken away from it.
    let live: PrivateIndex = PrivateIndex::create(real.as_path()).expect("own copy");
    assert!(!abandoned.exists(), "the abandoned copy is removed");
    assert!(old.exists(), "a copy with no record is kept");
    assert!(staging.exists(), "a copy being built is kept");
    assert!(unreadable.exists(), "an unproven owner keeps its copy");
    // A later copy leaves the one this process still holds alone.
    let later: PrivateIndex = PrivateIndex::create(real.as_path()).expect("second copy");
    assert!(live.path().exists(), "a live copy is kept");
    drop(later);
    drop(live);
    std::fs::remove_dir_all(&unreadable).expect("unproven copy");
    std::fs::remove_dir_all(&old).expect("record-less copy");
    std::fs::remove_dir_all(&staging).expect("staging copy");
    assert_eq!(entries(root.as_path()), vec!["index"]);
    remove(root.as_path());
}

/// What:
///  A copy with no owner record is never removed,
///  and every copy this release publishes holds its record.
/// Why:
///  Before the record existed the sweep judged a copy by the process id in its name,
///  and a
///       process id means something only inside its PID namespace:
///  a copy a command in another namespace
///       was still reading looked dead here,
///  so the sweep could delete a live command's copy,
///  and a second
///       command then failed while copying the index or while using it.
///  The record is therefore written before the name that a
///       sweep can see exists at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // writeRecordlessCopy(deadLookingPid); createPrivateIndex(); expect(recordless).toExist(); expect(copiesWithoutRecord()).toEqual([])
/// ```
#[test]
fn a_copy_without_an_owner_record_is_kept_and_every_published_copy_has_one() {
    let root: PathBuf = fixture("private-index-recordless");
    let real: PathBuf = root.join("index");
    std::fs::write(&real, b"DIRC").expect("real index");
    // A copy whose record a kill prevented, named with a process id no process here has: another PID
    // namespace's live command looks exactly like this from here.
    let unseen: i64 = 4_194_304;
    let recordless: PathBuf = root.join(format!("{PRIVATE_INDEX_PREFIX}{unseen}-1-0"));
    std::fs::create_dir(&recordless).expect("record-less directory");
    std::fs::write(recordless.join("index"), b"stale copy").expect("stale copy");
    let live: PrivateIndex = PrivateIndex::create(real.as_path()).expect("own copy");
    assert!(
        recordless.exists(),
        "a copy nothing proves dead is kept, namespaces aside"
    );
    // Every copy this release publishes names its owner, so the interval the old fallback judged cannot exist.
    let mut unnamed: Vec<String> = Vec::new();
    for name in entries(root.as_path()) {
        if !name.starts_with(PRIVATE_INDEX_PREFIX) {
            continue;
        }
        let directory: PathBuf = root.join(&name);
        if !directory.is_dir() {
            continue;
        }
        if !directory.join("owner").is_file() {
            unnamed.push(name);
        }
    }
    assert_eq!(
        unnamed,
        vec![recordless.file_name().expect("name").to_string_lossy().into_owned()],
        "only the copy this release did not make may lack a record"
    );
    // The record names this process, so a later sweep keeps the copy it is reading.
    let token: String =
        std::fs::read_to_string(live.path().parent().expect("dir").join("owner")).expect("record");
    let owner: crate::worktree_copy_owner::CopyOwner =
        crate::worktree_copy_owner::CopyOwner::from_token(token.trim()).expect("a token");
    assert_eq!(owner.pid, i64::from(std::process::id()));
    drop(live);
    std::fs::remove_dir_all(&recordless).expect("record-less copy");
    assert_eq!(entries(root.as_path()), vec!["index"]);
    remove(root.as_path());
}

/// What:
///  A copy whose `owner` record is a FIFO nobody writes to does not hold up a new copy.
/// Why:
///   Finding of the fifth review:
/// a directory named with this release's prefix can hold any file at `owner`,
/// and reading a FIFO with no writer waits for one that never comes,
/// so the sweep before every private index waited for ever and the command with it.
///  The record is opened without blocking and judged by its own type,
/// so a pipe proves nothing and its copy is kept,
/// and the new copy is made.
#[cfg(target_os = "linux")]
#[test]
fn an_owner_record_that_is_a_fifo_does_not_hold_up_a_new_copy() {
    use std::os::unix::ffi::OsStrExt as _;
    let root: PathBuf = fixture("private-index-fifo");
    let real: PathBuf = root.join("index");
    std::fs::write(&real, b"DIRC").expect("real index");
    let blocked: PathBuf = root.join(format!("{PRIVATE_INDEX_PREFIX}4194304-9-0"));
    std::fs::create_dir(&blocked).expect("a copy directory");
    std::fs::write(blocked.join("index"), b"stale copy").expect("stale copy");
    let record: PathBuf = blocked.join("owner");
    let path: std::ffi::CString = std::ffi::CString::new(record.as_os_str().as_bytes()).expect("a path without a NUL");
    // SAFETY: `mkfifo` takes a valid C string and a mode; it touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(path.as_ptr(), 0o600) };
    assert_eq!(made, 0, "mkfifo: {}", std::io::Error::last_os_error());
    // The creation runs on a thread of its own, so a wait that never answers fails this control instead of hanging it.
    let (told, heard): (std::sync::mpsc::Sender<bool>, std::sync::mpsc::Receiver<bool>) = std::sync::mpsc::channel();
    let for_the_copy: PathBuf = real.clone();
    let made_on_a_thread: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _told = told.send(PrivateIndex::create(for_the_copy.as_path()).is_ok());
    });
    let answered: Option<bool> = heard.recv_timeout(THE_COPY_ANSWERS_WITHIN).ok();
    assert_eq!(
        answered,
        Some(true),
        "a private index was not created beside an owner record that is a FIFO with no writer"
    );
    assert!(blocked.exists(), "a record that proves nothing keeps its copy");
    std::fs::remove_dir_all(&blocked).expect("the unproven copy");
    // The thread answers before it drops its copy, so the listing waits for that thread.
    made_on_a_thread.join().expect("the thread that made the copy");
    assert_eq!(entries(root.as_path()), vec!["index"]);
    remove(root.as_path());
}
