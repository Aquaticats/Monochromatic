//! What:
//!  One copy of the built executable per build on the fixtures' filesystem,
//!  for fixtures that cannot hard-link the build itself.
//! Why:
//!  Cargo's scratch directory moves with `build.build-dir`, which this repository's hosts relocate to another volume
//!      than the final artifacts,
//!  and a hard link cannot cross volumes.
//!  Every fixture then copied the 22 MB debug executable:
//!  a wave of 169 tests wrote about 3.7 GB (169 times 21.9 MB) within seconds, on the volume that every other command of
//!  those tests syncs to; the failing runs that carried the "no hard link" message hit the shared 5-second bound.
//!  The copy is made once per build under an advisory lock, so a wave of tests that all start together copies once,
//!  and fixtures link the copy as they would link the build.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await lock(join(store, '.lock')); if (!exists(stored)) { await copy(built, partial); await rename(partial, stored); }
//! ```

/// Native path and file types.
use std::path::{Path, PathBuf};

/// The directory of a fixture root that holds the stored copies.
pub const STORE_DIRECTORY: &str = "wrapper-store";

/// The advisory lock file inside the store, held while a copy is made.
const LOCK_FILE: &str = ".lock";

/// Prefix of a copy in progress; a finished copy is renamed to its final name, so a name with this prefix is never linked.
const PARTIAL_PREFIX: &str = ".partial-";

/// How many copies this process has made, so a test can tell one copy for many callers from one copy per caller.
pub static COPIES_MADE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// What:
///  The name that identifies one build of the executable:
///  its length and modification time in nanoseconds.
/// Why:
///  A rebuild changes the modification time, so a stored copy of an older build is never linked by mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${stat.size}-${stat.mtimeNs}-cli-git-native`
/// ```
fn stored_name(built: &Path) -> std::io::Result<String> {
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = std::fs::metadata(built)?;
    return Ok(format!("{}-{}{:09}-cli-git-native", metadata.len(), metadata.mtime(), metadata.mtime_nsec()));
}

/// What:
///  The path of the stored copy of `built` under `root`, made if it does not exist yet.
/// Why:
///  Fixtures on `root`'s filesystem link this file. The copy is written under a partial name and renamed, so a
///  stored name is complete whenever it exists, and the lock makes the processes that start together wait for the
///  first one's copy instead of each making their own; the kernel drops the lock if its holder dies.
///  Copies of older builds are removed once the new one stands, since fixtures that linked them keep their own link.
///
/// # Errors
///  When the store cannot be created, locked or written, or the copy fails.
pub fn shared_copy(built: &Path, root: &Path) -> std::io::Result<PathBuf> {
    let store: PathBuf = root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store)?;
    let stored: PathBuf = store.join(stored_name(built)?);
    if stored.exists() {
        return Ok(stored);
    }
    let lock: std::fs::File = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(store.join(LOCK_FILE))?;
    lock.lock()?;
    if !stored.exists() {
        let partial: PathBuf = store.join(format!("{PARTIAL_PREFIX}{}", std::process::id()));
        super::support::copy_executable(built, partial.as_path());
        COPIES_MADE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        std::fs::rename(&partial, &stored)?;
        remove_other_builds(store.as_path(), stored.as_path());
    }
    return Ok(stored);
}

/// Remove every copy in the store except `keep`, leaving the lock file; a name that cannot be removed stays.
fn remove_other_builds(store: &Path, keep: &Path) {
    let Ok(entries) = std::fs::read_dir(store) else {
        return;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        if path != keep && entry.file_name() != LOCK_FILE {
            // A copy of an older build, or a partial copy of a process that died.
            if let Err(error) = std::fs::remove_file(&path) {
                eprintln!("wrapper store: could not remove {} ({error})", path.display());
            }
        }
    }
}
