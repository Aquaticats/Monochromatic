//! What:
//!  Reading `Cargo.lock` before a `cargo clean` and putting it back as it was afterwards.
//! Why:
//!  A clean may rewrite the lock file,
//! and a new worktree must stay as checked out;
//! the two steps need nothing from
//!  either clean,
//! so they live in a file of their own that both import:
//! `package_repair_cargo.rs`,
//! which cleans what the worktree already has,
//! and `package_repair_prewarm_job.rs`,
//! which cleans the copy the commit service fills a build directory from after the add returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = readLock(root); await clean(); restoreLock(root, before);
//! ```

/// The debug log of the repair.
use crate::diagnostic_log::debug;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairCargoLock";

/// The lock file beside a workspace's root manifest.
const LOCK_FILE: &str = "Cargo.lock";

/// What:
///  The bytes of the workspace's `Cargo.lock`,
/// or none when it has none.
/// Why:
///   `cargo clean --package` resolves the workspace,
/// and Cargo rewrites a lock that no longer matches its manifest;
/// the
///  new worktree must stay as checked out,
/// so the file is put back afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = readLock(root); // Uint8Array | undefined
/// ```
pub fn read_lock(root: &Path) -> Option<Vec<u8>> {
    return std::fs::read(root.join(LOCK_FILE)).ok();
}

/// What:
///  Put `Cargo.lock` back as `before` read it,
/// removing a lock that did not exist before.
/// Why:
///   A write is made only when the content changed,
/// so an untouched lock keeps its file time.
/// A lock that cannot be put back
///  is logged,
/// and left for the person:
/// it is the worktree's own file and the checkout shows the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restoreLock(root, before);
/// ```
pub fn restore_lock(root: &Path, before: Option<&[u8]>) {
    let path: PathBuf = root.join(LOCK_FILE);
    let after: Option<Vec<u8>> = std::fs::read(&path).ok();
    if after.as_deref() == before {
        return;
    }
    let outcome: std::io::Result<()> = match before {
        Some(bytes) => std::fs::write(&path, bytes),
        None => std::fs::remove_file(&path),
    };
    match outcome {
        Ok(()) => debug(TAG, format!("{} was put back as checked out after the clean", path.display()).as_str()),
        Err(error) => debug(TAG, format!("{} changed during the clean and could not be put back ({error})", path.display()).as_str()),
    }
}
