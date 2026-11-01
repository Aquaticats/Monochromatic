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
//!  The copy is published without a lock:
//!  each caller that finds no stored copy writes its own uniquely named partial file and links it to the final name,
//!  which fails if another caller published first, so the final name is only ever a complete file and the first
//!  publisher wins.
//!  Callers that start together may each write a partial copy once per build; every later caller finds the final file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!exists(stored)) { await copy(built, partial); try { await link(partial, stored); } catch (AlreadyExists) {} await unlink(partial); }
//! ```

/// Native path and file types.
use std::path::{Path, PathBuf};
/// Distinguishes the partial files of callers in one process.
use std::sync::atomic::{AtomicUsize, Ordering};
/// Age of a partial file.
use std::time::Duration;

/// The directory of a fixture root that holds the stored copies.
pub const STORE_DIRECTORY: &str = "wrapper-store";

/// Prefix of a copy in progress; a finished copy is linked to its final name, so a name with this prefix is never used.
pub const PARTIAL_PREFIX: &str = ".partial-";

/// How old a partial file must be before a caller removes it: a copy of the executable takes seconds, so a partial this
///  old belongs to a caller that died.
const STALE_PARTIAL: Duration = Duration::from_secs(3600);

/// The number of partial files this process has started, which keeps their names unique.
static PARTIALS_STARTED: AtomicUsize = AtomicUsize::new(0);

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
///  Fixtures on `root`'s filesystem link this file. The partial copy is linked to the stored name, never renamed over
///  it, so a caller that loses the race leaves the winner's file alone and uses it, and a reader that opened the stored
///  name keeps a complete file whatever happens to the other names. Copies of older builds are removed once the new
///  one stands; a fixture that linked one keeps its own link, and a caller that finds its stored name gone links
///  nothing and falls back to a copy of its own.
///
/// # Errors
///  When the store cannot be created or written, or the copy fails.
pub fn shared_copy(built: &Path, root: &Path) -> std::io::Result<PathBuf> {
    let store: PathBuf = root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store)?;
    let stored: PathBuf = store.join(stored_name(built)?);
    if stored.exists() {
        return Ok(stored);
    }
    let partial: PathBuf = store.join(format!(
        "{PARTIAL_PREFIX}{}-{}",
        std::process::id(),
        PARTIALS_STARTED.fetch_add(1, Ordering::SeqCst)
    ));
    super::support::copy_executable(built, partial.as_path());
    publish(partial.as_path(), stored.as_path())?;
    remove_other_builds(store.as_path(), stored.as_path());
    return Ok(stored);
}

/// What:
///  Make the finished `partial` file the stored copy unless a stored copy already exists, and remove `partial` either way.
/// Why:
///  A hard link fails with `AlreadyExists` when another caller published first, and then that caller's file stays the
///  stored copy: a rename would replace it under the links the other caller's fixtures already hold. Taking the
///  winner's file is correct because both are copies of the same build.
///
/// # Errors
///  When the link fails for a reason other than the stored copy existing.
pub fn publish(partial: &Path, stored: &Path) -> std::io::Result<()> {
    let published: std::io::Result<()> = match std::fs::hard_link(partial, stored) {
        Err(error) if error.kind() != std::io::ErrorKind::AlreadyExists => Err(error),
        _ => Ok(()),
    };
    if let Err(error) = std::fs::remove_file(partial) {
        eprintln!("wrapper store: could not remove {} ({error})", partial.display());
    }
    return published;
}

/// Remove every finished copy in the store except `keep`, and partial files old enough to belong to a dead caller;
///  a name that cannot be removed stays. A partial file of a live caller is never touched.
fn remove_other_builds(store: &Path, keep: &Path) {
    let Ok(entries) = std::fs::read_dir(store) else {
        return;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let removable: bool = if name.starts_with(PARTIAL_PREFIX) {
            entry
                .metadata()
                .and_then(|metadata| { return metadata.modified(); })
                .ok()
                .and_then(|modified| { return modified.elapsed().ok(); })
                .is_some_and(|age| { return age > STALE_PARTIAL; })
        } else {
            path != keep
        };
        if removable
            && let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("wrapper store: could not remove {} ({error})", path.display());
        }
    }
}
