//! What:
//!  The file steps that the migration's writers share:
//! write bytes and flush them to disk,
//! flush a directory,
//! replace a small file beside the tree without following a link at its name,
//!  and the switch a control uses to keep those writes in the page cache.
//! Why:
//!  Each writer ran "write,
//! and only if that worked,
//! flush" as a chain of `and_then` closures,
//! which
//!  `rust/avoidable-closure` reports;
//! one named function per step reads the same everywhere.
//!  The switch exists because a control that simulates a kill in the process does not lose the page cache,
//! so the
//!  flush proves nothing there and costs the disk it shares with every other session:
//! measured 2026-11-08,
//! the same
//!  control took 35.4 s and 32.2 s with the flushes and 2.1 s and 2.4 s without them
//!  (`doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`).
//! A stand-in platform says so itself
//!  (`Platform::flushes_to_disk`),
//! and the tests that run against the real platform keep every flush.
//! The switch is the answer of the platform whose call is running,
//! installed for that call's
//!  whole extent and put back when it ends,
//! so a call on a platform that flushes keeps its flushes inside a call on one that does not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await file.write(bytes); await file.sync();   // the second line runs only if the first did not throw
//! await (await open(directory)).sync();
//! ```

use std::io::Write as _;
use std::path::Path;

#[cfg(test)]
thread_local! {
    /// Whether this thread's writes are kept in the page cache.
    /// Only a call on a platform that says it does not flush sets it,
    /// and the call puts the switch back when it ends,
    /// so no other test of the same process inherits it
    ///  and a call on a platform that flushes inside such a call turns the flushes back on.
    static KEEPING_WRITES_IN_MEMORY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// What:
///  Whether the calling thread's writes may skip the flush.
/// Why:
///   The steps below ask once per call,
/// and a release build always flushes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keepInMemory = () => false;
/// ```
fn keeping_writes_in_memory() -> bool {
    #[cfg(test)]
    return KEEPING_WRITES_IN_MEMORY.with(std::cell::Cell::get);
    // A build that is not a test has no switch to read.
    #[cfg(not(test))]
    return false;
}

/// What:
///  The flush question answered as a platform answers it,
/// put back when this value goes.
/// Why:
///   A guard puts the answer back however the scope ends,
/// so a run that panics inside it does not leave the rest of
///  its thread flushing into the page cache;
/// every call installs its own platform's answer,
/// so a call on a platform that flushes
///  inside a call on one that does not keeps its flushes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = keepingWritesInMemory(); try { run() } finally { kept.restore() }
/// ```
pub(crate) struct WritesKeptInMemory {
    /// What the switch was before.
    before: bool,
}

impl WritesKeptInMemory {
    /// Use the answer of a platform that says `flushes_to_disk` until the value goes.
    pub(crate) fn for_flush_policy(flushes_to_disk: bool) -> WritesKeptInMemory {
        #[cfg(test)]
        return WritesKeptInMemory { before: KEEPING_WRITES_IN_MEMORY.with(|flag| { return flag.replace(!flushes_to_disk); }) };
        // A build that is not a test keeps every write on its way to the disk and has no switch to set.
        #[cfg(not(test))]
        let _ = flushes_to_disk;
        #[cfg(not(test))]
        return WritesKeptInMemory { before: false };
    }

    /// Turn the switch on for the calling thread until the value goes.
    #[cfg(test)]
    pub(crate) fn for_this_thread() -> WritesKeptInMemory {
        return WritesKeptInMemory::for_flush_policy(false);
    }
}

impl Drop for WritesKeptInMemory {
    /// Put the switch back,
    /// which is the only thing this value does.
    fn drop(&mut self) {
        #[cfg(test)]
        KEEPING_WRITES_IN_MEMORY.with(|flag| { flag.set(self.before); });
        // A build that is not a test has no switch, so there is nothing to put back.
        #[cfg(not(test))]
        let _ = self.before;
    }
}

/// Write all of `bytes` into `file`,
/// then flush the file to disk;
/// the flush runs only if the write worked,
/// and no flush runs
///  where the calling thread keeps its writes in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeAllThenSync(file: FileHandle, bytes: Uint8Array): Promise<void>; // rejects with the first failure
/// ```
pub fn write_all_then_sync(file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<()> {
    file.write_all(bytes)?;
    if keeping_writes_in_memory() {
        return Ok(());
    }
    return file.sync_all();
}

/// Flush a directory to disk,
/// so a rename or a new name inside it survives a crash,
/// unless the calling thread keeps its
///  writes in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function syncDirectory(directory: string): Promise<void>; // (await open(directory)).sync()
/// ```
pub fn sync_directory_to_disk(directory: &Path) -> std::io::Result<()> {
    if keeping_writes_in_memory() {
        return Ok(());
    }
    let handle: std::fs::File = std::fs::File::open(directory)?;
    return handle.sync_all();
}

/// What:
///  Write `bytes` to `target`,
/// by writing them into a sibling created exclusively under a name of its own and renaming it over
///  `target`.
/// Why:
///   A plain write follows a symbolic link and truncates whatever it points at,
/// so a link planted at a
///  list's name would let the command overwrite a file of the person's own.
/// A rename replaces the link itself,
/// never its
///  target,
/// and the exclusive create refuses a name that already exists.
/// A failure removes what this call made,
/// so a
///  failed write leaves neither a half file nor a temporary one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeBesideThenRename(listPath, bytes); // O_EXCL sibling, fsync, rename, fsync the directory
/// ```
pub fn write_beside_then_rename(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_before_epoch) => 0,
    };
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".tmp.{}.{nanos}", std::process::id()));
    let sibling: std::path::PathBuf = std::path::PathBuf::from(name);
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(sibling.as_path())?;
    if let Err(error) = write_all_then_sync(&mut file, bytes) {
        drop(file);
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling.as_path());
        return Err(error);
    }
    drop(file);
    if let Err(error) = std::fs::rename(sibling.as_path(), target) {
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling.as_path());
        return Err(error);
    }
    return match target.parent() {
        Some(directory) => sync_directory_to_disk(directory),
        None => Ok(()),
    };
}

/// Controls for the steps,
/// the writer and the switch stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_file_write_steps_tests.rs"]
mod tests;
