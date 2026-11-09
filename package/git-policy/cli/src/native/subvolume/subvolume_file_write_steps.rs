//! What:
//!  Two file steps that the migration's writers share:
//! write bytes and flush them to disk,
//! and flush a directory,
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
    /// Only a control whose platform stands in for a filesystem that loses nothing sets it,
    /// and only for the
    ///  thread that runs the command,
    /// so no other test of the same process inherits it.
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
///  The switch turned on for the calling thread,
/// put back when this value goes.
/// Why:
///   A guard puts the switch back however the scope ends,
/// so a run that panics inside it does not leave the rest of
///  its thread flushing into the page cache.
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
    /// Turn the switch on for the calling thread until the value goes.
    pub(crate) fn for_this_thread() -> WritesKeptInMemory {
        #[cfg(test)]
        return WritesKeptInMemory { before: KEEPING_WRITES_IN_MEMORY.with(|flag| { return flag.replace(true); }) };
        // A build that is not a test keeps every write on its way to the disk and has no switch to set.
        #[cfg(not(test))]
        return WritesKeptInMemory { before: false };
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

/// Controls for the steps and the switch.
#[cfg(test)]
#[path = "subvolume_file_write_steps_tests.rs"]
mod tests;
