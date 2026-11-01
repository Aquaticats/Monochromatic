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
//!  each caller that finds no stored copy writes its own exclusively created partial file and links it to the final name,
//!  which fails if another caller published first, so the final name is only ever a complete file and the first
//!  publisher wins.
//!  Callers that start together may each write a partial copy once per build; every later caller finds the final file.
//!  The final name is the content of the build, not its size or time, and a copy is checked against that name before
//!  it is published.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!exists(stored)) { await copy(built, partial); try { await link(partial, stored); } catch (AlreadyExists) {} await unlink(partial); }
//! ```

/// Native path and file types.
use std::path::{Path, PathBuf};
/// Distinguishes the partial files of callers in one process.
use std::sync::atomic::{AtomicUsize, Ordering};
/// The name of the build, computed once per process.
use std::sync::OnceLock;
/// Ages.
use std::time::Duration;

/// The directory of a fixture root that holds the stored copies.
pub const STORE_DIRECTORY: &str = "wrapper-store";

/// Prefix of a copy in progress; a finished copy is linked to its final name, so a name with this prefix is never used.
pub const PARTIAL_PREFIX: &str = ".partial-";

/// How long a copy of another build must go unlinked before a publisher removes it. Each fixture's link changes the
///  copy's status-change time, so the age is the time since a fixture last linked it; a run that links a copy does so
///  within seconds of finding it, far inside this.
pub const IDLE_BUILD_AGE: Duration = Duration::from_secs(24 * 3600);

/// How many times a caller asks for the stored copy when the link finds it gone.
const LINK_ATTEMPTS: usize = 3;

/// How many partial names a caller tries before it gives up; each failed name belonged to a file that already existed.
pub const RESERVE_ATTEMPTS: usize = 64;

/// The number of partial names this process has tried, which keeps its own names apart from each other.
static PARTIALS_STARTED: AtomicUsize = AtomicUsize::new(0);

/// What reading a build's name came to: the name, or the kind and text of the error that stopped the read.
pub type BuildName = Result<String, (std::io::ErrorKind, String)>;

/// The stored name of the built executable, read once per process, an error included.
static BUILD_NAME: OnceLock<BuildName> = OnceLock::new();

/// How many bytes one read takes, so a build of any size is hashed in bounded memory.
const CHUNK_BYTES: usize = 1 << 20;

/// What:
///  The name that identifies the bytes of a file:
///  a 64-bit hash of them and their length.
/// Why:
///  Length and modification time do not identify a build: two different builds of one length can share a timestamp.
///  The hash is `DefaultHasher` (SipHash), which runs at the speed of the optimized standard library even in a debug
///  test build, and it guards against accidental equality, not against someone who crafts a file; its value is stable
///  only within one toolchain, which is enough, since a different toolchain just stores another copy.
///
/// # Errors
///  When the file cannot be read.
pub fn content_name(path: &Path) -> std::io::Result<String> {
    use std::hash::Hasher;
    use std::io::Read;
    let mut file: std::fs::File = std::fs::File::open(path)?;
    let mut hasher: std::hash::DefaultHasher = std::hash::DefaultHasher::new();
    let mut chunk: Vec<u8> = vec![0; CHUNK_BYTES];
    let mut length: u64 = 0;
    loop {
        let read: usize = file.read(chunk.as_mut_slice())?;
        if read == 0 {
            break;
        }
        hasher.write(&chunk[..read]);
        length += u64::try_from(read).map_err(std::io::Error::other)?;
    }
    return Ok(format!("{:016x}-{length}-cli-git-native", hasher.finish()));
}

/// What:
///  The outcome of `compute` for `cell`, which runs `compute` once however many callers arrive together.
/// Why:
///  Every caller that found the cell empty used to hash the whole 22 MB executable before the first result was
///  stored, so a wave of fixtures that start together read it once each. `OnceLock::get_or_init` holds the other
///  callers until the first one's `compute` returns, so exactly one read happens. A read that failed is the outcome
///  for the rest of the process (an unreadable build rarely becomes readable within a test run), and the callers of
///  a failed read fall back to a private copy of their own, which is slower but never stuck.
///
/// # Errors
///  When `compute` failed, in this caller or an earlier one.
pub fn name_once(
    cell: &OnceLock<BuildName>,
    compute: impl FnOnce() -> std::io::Result<String>,
) -> std::io::Result<String> {
    return cell
        .get_or_init(|| { return compute().map_err(|error| { return (error.kind(), error.to_string()); }); })
        .clone()
        .map_err(|(kind, message)| { return std::io::Error::new(kind, message); });
}

/// The name of the build `built`, read once per process.
fn build_name(built: &Path) -> std::io::Result<String> {
    return name_once(&BUILD_NAME, || { return content_name(built); });
}

/// What:
///  The path of the stored copy of `built` under `root`, made if it does not exist yet.
/// Why:
///  Fixtures on `root`'s filesystem link this file. The partial copy is read back and must have the build's name
///  before it is linked to the stored name, never renamed over it, so a caller that loses the race leaves the
///  winner's file alone and uses it, and a copy made from a build that changed under the caller is never published.
///  Copies of other builds that no fixture has linked for `IDLE_BUILD_AGE` are removed once a new copy stands.
///
/// # Errors
///  When the store cannot be created or written, the copy fails, or the copy does not have the build's name.
pub fn shared_copy(built: &Path, root: &Path) -> std::io::Result<PathBuf> {
    let store: PathBuf = root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store)?;
    let name: String = build_name(built)?;
    let stored: PathBuf = store.join(&name);
    if stored.exists() {
        return Ok(stored);
    }
    let partial: PathBuf = reserve_partial(store.as_path(), &PARTIALS_STARTED)?;
    super::support::copy_executable(built, partial.as_path());
    // `cp` into an existing file keeps that file's mode, so the build's mode is applied after the copy.
    std::fs::set_permissions(&partial, std::fs::metadata(built)?.permissions())?;
    if content_name(partial.as_path())? != name {
        let _ = std::fs::remove_file(&partial);
        return Err(std::io::Error::other(format!(
            "{} changed while it was copied: the copy does not have the name {name} read before",
            built.display()
        )));
    }
    publish(partial.as_path(), stored.as_path())?;
    remove_idle_builds(store.as_path(), stored.as_path(), IDLE_BUILD_AGE);
    return Ok(stored);
}

/// What:
///  Create an empty partial file under a name no file in `store` has, and return its path.
/// Why:
///  A name made of the process identifier and a counter is unique in one process but not across processes: a
///  process identifier is reused after a crash and repeats in another PID namespace that shares the store, and a
///  partial file left by a caller that died after linking it to the final name but before removing it is the
///  published file under a second name. Opening such a name for writing, as the copy does, would truncate the
///  published executable under the fixtures that run it. Creating the name exclusively (`create_new`, `O_EXCL`)
///  fails on every existing name, so the caller takes the next counter value; no lock is involved.
///  The copy then writes into the empty file this created.
///
/// # Errors
///  When the file cannot be created for another reason than its name existing, or every name tried exists.
pub fn reserve_partial(store: &Path, counter: &AtomicUsize) -> std::io::Result<PathBuf> {
    for _ in 0..RESERVE_ATTEMPTS {
        let candidate: PathBuf = store.join(format!(
            "{PARTIAL_PREFIX}{}-{}",
            std::process::id(),
            counter.fetch_add(1, Ordering::SeqCst)
        ));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    return Err(std::io::Error::other(format!(
        "every one of {RESERVE_ATTEMPTS} partial names in {} exists",
        store.display()
    )));
}

/// What:
///  Make the finished `partial` file the stored copy unless a stored copy already exists, and remove `partial` either way.
/// Why:
///  A hard link fails with `AlreadyExists` when another caller published first, and then that caller's file stays the
///  stored copy: a rename would replace it under the links the other caller's fixtures already hold. Taking the
///  winner's file is correct because both have the build's name.
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

/// What:
///  Hard-link the stored copy of `built` at `to`, asking for the copy again if it is gone before the link.
/// Why:
///  A copy of another build can be removed by a publisher between a caller's getting its path and linking it (it
///  must have gone unlinked for `IDLE_BUILD_AGE`, so this is rare, but not impossible); the caller then makes or finds
///  the copy again instead of falling back to a copy of its own. `after_found` runs between the two steps so a control
///  can remove the copy at exactly that moment.
///
/// # Errors
///  When the stored copy cannot be made, the link fails for another reason than the copy being gone (a link across
///  filesystems included), or the copy is gone on every attempt.
pub fn link_stored(
    built: &Path,
    root: &Path,
    to: &Path,
    after_found: &mut dyn FnMut(&Path),
) -> std::io::Result<()> {
    for _ in 0..LINK_ATTEMPTS {
        let stored: PathBuf = shared_copy(built, root)?;
        after_found(stored.as_path());
        match std::fs::hard_link(&stored, to) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !stored.exists() => {}
            Err(error) => return Err(error),
        }
    }
    return Err(std::io::Error::other(format!(
        "the stored copy of {} was gone {LINK_ATTEMPTS} times before it could be linked",
        built.display()
    )));
}

/// Remove every finished copy in the store except `keep` that no fixture has linked for `idle`, and partial files
///  that old (a dead caller's); a name that cannot be removed stays. A younger partial file, a live caller's, is
///  never touched.
pub fn remove_idle_builds(store: &Path, keep: &Path, idle: Duration) {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir(store) else {
        return;
    };
    let now: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| { return i64::try_from(since.as_secs()).unwrap_or(i64::MAX); });
    let idle_seconds: i64 = i64::try_from(idle.as_secs()).unwrap_or(i64::MAX);
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let is_partial: bool = entry.file_name().to_string_lossy().starts_with(PARTIAL_PREFIX);
        let idle_long_enough: bool = entry
            .metadata()
            .is_ok_and(|metadata| { return now.saturating_sub(metadata.ctime()) >= idle_seconds; });
        if (is_partial || path != keep) && idle_long_enough
            && let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("wrapper store: could not remove {} ({error})", path.display());
        }
    }
}
