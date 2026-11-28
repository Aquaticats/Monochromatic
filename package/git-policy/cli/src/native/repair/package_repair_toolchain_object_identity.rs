//! What:
//!  The object a name holds,
//!  as device,
//!  inode,
//!  size and write time,
//!  and the two comparisons the repair makes with it:
//!  whether a launcher record names another checkout's lock,
//!  however that lock is spelled,
//!  and whether the object at a name is still the one a pass judged.
//! Why:
//!  `package_repair_toolchain.rs` would otherwise pass the file-size limit,
//!  and one file has many spellings:
//!  a link to the worktree,
//!  a path that walks into a directory and back out of it,
//!  and a bind mount of the same folder all resolve to one object,
//!  so the comparison is the object's and it lives in a file of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identityOf = (path: string) => [statSync(path).dev, statSync(path).ino, statSync(path).size, statSync(path).mtime];
//! ```

/// Import the slot a control stands a concurrent writer in,
/// which only a control build has.
#[cfg(test)]
use super::names_and_bounds::WHEN_THE_RECORD_IS_OPENED;
/// Import the lock file's name and the record's key.
use super::names_and_bounds::{
    LOCK_FILE,
    LOCK_KEY,
};
/// Import the byte bound a record is read at and the log tag.
use super::names_and_bounds::{
    LONGEST_RECORD,
    TAG,
};
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the bounded,
/// non-blocking read of a plain file.
use crate::plain_file::read_plain_text_at;
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  The object a name held:
///  its device and inode,
///  its size,
///  and the moment it was written.
/// Why:
///   A launcher a concurrent install writes is a new object under the same name
///   (`mpm-shadow` places it with a rename),
///   so an object equal in all four is the one the pass classified and no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Identity = { device: bigint; inode: bigint; size: bigint; modified: [bigint, bigint] };
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct Identity {
    /// The device the object is on.
    device: u64,
    /// Its inode.
    inode: u64,
    /// Its size in bytes.
    size: u64,
    /// When it was last written,
    /// in whole seconds since the epoch.
    modified_seconds: i64,
    /// And the nanoseconds of that moment.
    modified_nanoseconds: i64,
}

/// What:
///  The object `path` names now,
///  or nothing when it is not there.
/// Why:
///   A launcher is compared with the object the pass classified,
///  and an absent one was removed by someone else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = (path) => { const s = statSync(path); return [s.dev, s.ino, s.size, s.mtime]; };
/// ```
pub(super) fn identity_of(path: &Path) -> Option<Identity> {
    return identity_from(std::fs::symlink_metadata(path).ok()?);
}

/// What:
///  The object `path` names once every link and relative segment on the way is resolved.
/// Why:
///   A record's `lockFile` may name one file through any spelling,
///  so what is compared is the object and not the text:
///  a link to the worktree,
///  a path through `..` and a bind mount of the same folder all resolve to one object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityFollowingLinks = (path) => { const s = statSync(path); return [s.dev, s.ino, s.size, s.mtime]; };
/// ```
fn identity_following_links_of(path: &Path) -> Option<Identity> {
    return identity_from(std::fs::metadata(path).ok()?);
}

/// What:
///  The identity of one looked-up object.
/// Why:
///   Both looks above hand the same four numbers on,
///  so the comparison between them cannot differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityFrom = (found) => ({ device: found.dev, inode: found.ino, size: found.size, modified: found.mtime });
/// ```
fn identity_from(found: std::fs::Metadata) -> Option<Identity> {
    use std::os::unix::fs::MetadataExt as _;
    return Some(Identity {
        device: found.dev(),
        inode: found.ino(),
        size: found.size(),
        modified_seconds: found.mtime(),
        modified_nanoseconds: found.mtime_nsec(),
    });
}

/// What:
///  This worktree's own `toolchain.lock.json`,
///  as its path is written and as the filesystem resolves it,
///  so a record that names either spelling of it is not a copy of another checkout's.
/// Why:
///  One read of that pair serves every record of a pass,
///  and the removal asks the same question again of the bytes it holds,
///  so the two answers cannot be given by different comparisons.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = (destination) => ({ own: join(destination, 'toolchain.lock.json'), real: realpath(destination) + '/toolchain.lock.json' });
/// ```
pub(super) struct ThisWorktreesLock {
    /// The lock as a record spells it.
    own: PathBuf,
    /// The same lock as the filesystem resolves it,
    /// for a name whose object cannot be read.
    real: Option<PathBuf>,
    /// The object the worktree's own lock is,
    /// when it is there.
    identity: Option<Identity>,
}

impl ThisWorktreesLock {
    /// The lock of `destination`.
    pub(super) fn of(destination: &Path) -> ThisWorktreesLock {
        let own: PathBuf = destination.join(LOCK_FILE);
        let real: Option<PathBuf> = match std::fs::canonicalize(destination) {
            Ok(resolved) => Some(resolved.join(LOCK_FILE)),
            Err(_) => None,
        };
        let identity: Option<Identity> = identity_following_links_of(own.as_path());
        return ThisWorktreesLock {
            own,
            real,
            identity,
        };
    }

    /// Whether `record` names a lock other than this worktree's:
    /// `Some(true)` when it does,
    /// `Some(false)` when it names this one,
    /// and nothing when it cannot be read as a record holding a lock.
    /// Why:
    ///   One file has many spellings,
    /// and a record may name this worktree's lock through a link to the worktree,
    ///  a path that walks into a directory and back out of it,
    /// or a bind mount of the same folder:
    ///  every one of them resolves to the same object,
    /// and only the object's identity tells that from a copy of another
    ///  checkout's lock.
    ///  Comparing spellings deleted a record an install had just written,
    /// which is why the comparison is the object's.
    ///  A name whose object cannot be read (a lock that is gone) falls back to the two spellings this release writes.
    pub(super) fn names_another_lock(
        &self,
        record: &Path,
    ) -> Option<bool> {
        let named: PathBuf = lock_named_by_record(record)?;
        if let (Some(mine), Some(theirs)) =
            (self.identity, identity_following_links_of(named.as_path()))
        {
            return Some(mine != theirs);
        }
        let mine: bool = named == self.own || Some(&named) == self.real.as_ref();
        return Some(!mine);
    }
}

/// What:
///  Run the replacement a control queued,
///  once,
///  at the one instruction before this release opens a launcher record.
/// Why:
///   A record is a file another command may replace at any moment,
///  and the window between a look at its name and its open cannot be timed from a fixture;
///  a control that shows what the read does in that window stands in it here.
///  The slot is per thread and absent from a release build,
///  so controls running beside each other never see one another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aControlMayReplaceTheRecordNow() { const replace = take(); if (replace) replace(); }
/// ```
#[cfg(test)]
fn a_control_may_replace_the_record_now() {
    let Some(replace) =
        WHEN_THE_RECORD_IS_OPENED.with(|cell: &std::cell::RefCell<Option<Box<dyn FnOnce()>>>| {
            return cell.borrow_mut().take();
        })
    else {
        return;
    };
    replace();
}

/// What:
///  The lock file a launcher record names,
/// when the record can be read and holds the key.
///  `Option<PathBuf>` is "the path,
/// or no answer".
/// Why:
///   A record that cannot be read,
/// is not JSON or names no lock is left alone,
/// and each step of reading it may fail the same way.
///  The record is read the way the lock beside it and a registration's `gitdir` are read
///  (`read_plain_text_at`):
///  opened without blocking and without following a link,
///  refused unless the open handle is a plain file,
///  and never read past `LONGEST_RECORD`.
///  A named pipe at that name therefore holds neither the add's thread nor a job of the service's pool,
///  a name swapped for one after any look is refused at the open rather than waited on,
///  and no file at that name decides how much memory a command reads.
///  A record refused there is left alone,
///  which is what a record this release cannot read already gets:
///  the add answers and goes on,
///  the repair asks the record again on its next pass,
///  the diagnostic log names the file and what refused it,
///  and the person sees no line of this repair unless another record asks for work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockNamedByRecord(path: string): string | undefined;
/// ```
pub(super) fn lock_named_by_record(path: &Path) -> Option<PathBuf> {
    #[cfg(test)]
    a_control_may_replace_the_record_now();
    let text: String = match read_plain_text_at(path, LONGEST_RECORD) {
        Ok(text) => text,
        Err(error) => {
            debug(
                TAG,
                format!(
                    "{} is no launcher record this release reads, so it is left ({error})",
                    path.display()
                )
                .as_str(),
            );
            return None;
        }
    };
    let record: serde_json::Value =
        serde_json::from_str::<serde_json::Value>(text.as_str()).ok()?;
    let lock: &str = record.get(LOCK_KEY).and_then(serde_json::Value::as_str)?;
    return Some(PathBuf::from(lock));
}
