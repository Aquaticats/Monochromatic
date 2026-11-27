//! What:
//! The directories this process knows it left unsynchronized:
//! the set itself,
//! the thread-safe calls that add to it,
//! remove from it and read it,
//! and the two entry points the busy-registry test drives.
//! Why:
//! A directory whose creation could not be made durable must not be treated as durable later,
//! so the knowledge outlives the call that failed and every walk of a later preservation asks it.
//! Split out of `journal_replace.rs`;
//! the items are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal_replace_unsynced.ts: a Set of paths plus a flag saying the set may be missing an entry
//! ```

/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// A directory is recorded under its canonical path,
/// so two spellings of one directory are one entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { realpathSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The registry owns the paths it holds and the calls take a borrowed one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};

/// The directories this process knows it left unsynchronized because a sync after their creation failed.
/// Contract assumption b:
/// such a directory is not treated as durable merely because it exists now.
/// The registry lives as long as the process;
/// across processes a directory left unsynced is a pre-existing directory and assumption b applies to it.
static UNSYNCED: std::sync::Mutex<std::collections::BTreeSet<PathBuf>> =
    std::sync::Mutex::new(std::collections::BTreeSet::new());

/// Set when the registry could not be updated because another thread held it:
/// knowledge may be missing,
/// so every directory counts as unsynced from then on (the safe answer continues the walk).
static UNSYNCED_UNKNOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
/// `fn remember_unsynced(directory: &Path)` records a directory whose creation was not made durable.
/// Why:
/// The walk of a later preservation continues through it.
/// The registry is only touched with `try_lock` and no file-system call is made while it is held (contract V6:
/// an internal lock
/// returns at once);
/// when it is busy the knowledge is not lost but widened to "unknown".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rememberUnsynced(directory: string): void
/// ```
pub(crate) fn remember_unsynced(directory: &Path) {
    let resolved: PathBuf = match fs::canonicalize(directory) {
        Ok(found) => found,
        Err(_) => directory.to_path_buf(),
    };
    match UNSYNCED.try_lock() {
        Ok(mut known) => {
            known.insert(resolved);
        }
        Err(_) => UNSYNCED_UNKNOWN.store(true, std::sync::atomic::Ordering::Relaxed),
    }
}

/// What:
/// `fn forget_unsynced(directory: &Path)` removes a directory from the registry once its entry has been synchronized.
/// Why:
/// A later preservation may stop at it again.
/// A busy registry keeps the entry,
/// which only makes later walks longer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetUnsynced(directory: string): void
/// ```
pub(crate) fn forget_unsynced(directory: &Path) {
    if let Ok(mut known) = UNSYNCED.try_lock() {
        known.remove(directory);
    }
}

/// What:
/// `fn is_unsynced(directory: &Path) -> bool` says whether this process knows it left the directory unsynchronized.
/// Why:
/// A busy or unreadable registry,
/// and a registry that missed an update,
/// count as "unsynced":
/// the safe answer continues the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isUnsynced(directory: string): boolean
/// ```
pub(crate) fn is_unsynced(directory: &Path) -> bool {
    if UNSYNCED_UNKNOWN.load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }
    return match UNSYNCED.try_lock() {
        Ok(known) => known.contains(directory),
        Err(_) => true,
    };
}

/// What:
/// `pub(crate) fn hold_unsynced_registry_for_test(work: impl FnOnce())` runs `work` while this thread holds the registry,
/// and clears the "unknown" mark afterwards.
/// Why:
/// The test of the busy-registry rule makes the lock busy on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function holdUnsyncedRegistryForTest(work: () => void): void
/// ```
#[cfg(test)]
pub(crate) fn hold_unsynced_registry_for_test(work: impl FnOnce()) {
    let guard = UNSYNCED.lock();
    work();
    drop(guard);
    UNSYNCED_UNKNOWN.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// What:
/// `pub(crate) fn registry_calls_for_test(directory: &Path) -> bool` makes the three registry calls on a directory and returns what
/// `is_unsynced` said last.
/// Why:
/// The busy-registry test calls them from a thread while another holds the registry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function registryCallsForTest(directory: string): boolean
/// ```
#[cfg(test)]
pub(crate) fn registry_calls_for_test(directory: &Path) -> bool {
    remember_unsynced(directory);
    forget_unsynced(directory);
    return is_unsynced(directory);
}
