//! What:
//!  Controls of when the pass removes a repository's records:
//!  a lock another command's Git makes between the reading of the records and the walk of the lock files still has the
//!  record that explains it,
//!  and the pass clears it in that same run.
//! Why:
//!  A record removed on a walk made before its reading loses the proof of a lock made in between,
//!  and the next command then meets a lock it cannot prove abandoned
//!  (a review of `2af2ecb879`, finding 2).
//!  The second command's lock is placed by the pass's own seam,
//!  between the reading and the walk,
//!  and nothing waits on a clock to put it there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! placeAfterTheRecordsAreRead(() => makeTheLock()); const outcome = clearAbandonedLocks([common]); expect(outcome.cleared).toHaveLength(1);
//! ```

use crate::abandoned_lock_pass_fault_hooks_test_support::place_after_the_records_are_read;
use crate::abandoned_locks::clear_abandoned_locks;
use crate::abandoned_lock_words::clearance_line;
use crate::lock_holder_records::{CHILD_KIND, SELF_KIND, folder_of, record_text};
use crate::record_proof_test_support::{a_dead_process, make_pid_file_after_the_lock};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  A repository's Git directory with the record a killed command leaves for a holder that is gone,
///  and a promise to make that holder's lock and PID file at the pass's seam.
///  `(PathBuf, ...)` is the Git directory and the record folder.
/// Why:
///   The record must be there before the pass reads the records,
///  and the lock must be there only after that reading:
///  that order is the state finding 2 is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const common = gitDirectoryWithARecordOfADeadHolder(); placeAfterTheRecordsAreRead(() => makeLockAndPidFile());
/// ```
fn git_directory_with_a_record(control: &str) -> (PathBuf, PathBuf, (i64, String)) {
    let root: PathBuf = fixture(control);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the Git directory");
    let folder: PathBuf = folder_of(common.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    let (gone, identity): (i64, String) = a_dead_process();
    std::fs::write(
        folder.join("record-of-the-gone-holder"),
        record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity.clone())]).as_bytes(),
    )
    .expect("the record a killed command leaves");
    return (common, folder, (gone, identity));
}

/// What:
///  Plant the lock a recorded Git leaves when it is killed:
///  `index.lock` with a half-written index and its PID file naming the recorded process.
/// Why:
///   This is the state the next command must be able to clear,
///  and its proof is the record the pass has just read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(common, 'index.lock'), 'a half-written index\n'); makePidFileAfterTheLock(indexLock, pid);
/// ```
fn make_the_lock(common: &Path, gone: i64) {
    let lock: PathBuf = common.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock a killed Git left");
    make_pid_file_after_the_lock(lock.as_path(), gone);
}

/// What:
///  A lock that a Git made after the pass read the records is seen by the pass's walk:
///  the pass clears it on the record it read,
///  and the record then goes with the lock it explained.
/// Why:
///  On the other order the walk found no lock,
///  nothing was cleared,
///  and the record was removed against that stale sight of the locks:
///  the next command then met the lock with no proof it was abandoned,
///  and a write through it failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(outcome.cleared.map(clearanceLine)).toHaveLength(1); expect(existsSync(indexLock)).toBe(false); expect(existsSync(record)).toBe(false);
/// ```
#[test]
fn a_lock_made_after_the_records_are_read_is_cleared_and_its_record_is_used() {
    let (common, folder, (gone, _identity)): (PathBuf, PathBuf, (i64, String)) = git_directory_with_a_record("record-removal-order");
    let lock: PathBuf = common.join("index.lock");
    let record: PathBuf = folder.join("record-of-the-gone-holder");
    let at_the_seam: PathBuf = common.clone();
    place_after_the_records_are_read(Box::new(move || { return make_the_lock(at_the_seam.as_path(), gone); }));
    let outcome = clear_abandoned_locks(&[common.clone()]);
    assert_eq!(outcome.cleared.len(), 1, "the pass cleared the lock the record it read explains: {:?}", outcome.cleared.iter().map(clearance_line).collect::<Vec<String>>());
    assert!(!lock.exists(), "the lock that was made after the reading is gone");
    assert!(!record.exists(), "and the record goes with the lock it explained, because nothing needs it any more");
    assert!(!folder.exists(), "and the folder with the last record it held");
    remove(common.parent().expect("the fixture root"));
}
