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
///  within seconds of finding it, far inside this. A copy that a fixture still links is never removed, and a link that
///  arrives during the removal keeps the copy (`remove_idle_builds_at`).
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
///  Copies of other builds that no fixture has linked for `IDLE_BUILD_AGE` are removed once a new copy stands, unless a
///  fixture links one while it is being removed (`remove_idle_builds_at`).
///
/// # Errors
///  When the store cannot be created or written, the copy fails, or the copy does not have the build's name.
pub fn shared_copy(built: &Path, root: &Path) -> std::io::Result<PathBuf> {
    let name: String = build_name(built)?;
    return shared_copy_named(built, name.as_str(), root, &|| {});
}

/// What:
///  `shared_copy` for a build whose `name` is known, with `before_publish` run after the copy is complete and checked
///  and before it is linked to the stored name.
/// Why:
///  A control that needs several callers to hold finished partial copies at the same moment, so that all of them
///  publish against each other, blocks them in `before_publish` until all have arrived; a small file can then stand
///  for the build, and the name need not come from the process-wide cell.
///
/// # Errors
///  The errors of `shared_copy`.
pub fn shared_copy_named(built: &Path, name: &str, root: &Path, before_publish: &dyn Fn()) -> std::io::Result<PathBuf> {
    let store: PathBuf = root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store)?;
    let stored: PathBuf = store.join(name);
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
    before_publish();
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

/// The seconds since the Unix epoch now, or zero when the clock is before it (then nothing is old enough to remove).
fn now_seconds() -> i64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| { return i64::try_from(since.as_secs()).unwrap_or(i64::MAX); });
}

/// What:
///  Remove every finished copy in the store except `keep` that no fixture holds a link to and that has gone unlinked
///  for `idle`, and partial files that old (a dead caller's).
/// Why:
///  A younger partial file belongs to a live caller and is never touched, and a name that cannot be removed stays.
///  Each link of a fixture changes the copy's status-change time, so the age is the time since a fixture last linked
///  or unlinked it. `remove_idle_builds_at` says how a link that arrives while the removal is under way is kept.
pub fn remove_idle_builds(store: &Path, keep: &Path, idle: Duration) {
    remove_idle_builds_at(store, keep, idle, now_seconds(), &mut |_| {});
}

/// What:
///  `remove_idle_builds` judging ages at `now` (seconds since the Unix epoch), with `after_decision` run with the path
///  of each finished copy between the decision to remove it and the removal.
/// Why:
///  A copy is removed only if its status-change time is `idle` or more before `now` and its only name is the store's
///  (no fixture holds a link, and removing the name is what frees the disk). The decision is made from a status read
///  that a fixture's link can follow, and a lock-free removal cannot make the read and the unlink one step, so the
///  name is first renamed to a name nobody asks for and the link count is read again: a count that grew means a
///  fixture linked in between, and the name is linked back. A caller that asks for the copy while its name is away
///  finds nothing and publishes another copy (`link_stored` asks again), so the worst outcome of the narrow window
///  that remains is one extra copy, never a fixture without its wrapper. `after_decision` lets a control link a
///  fixture at exactly that moment.
pub fn remove_idle_builds_at(store: &Path, keep: &Path, idle: Duration, now: i64, after_decision: &mut dyn FnMut(&Path)) {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir(store) else {
        return;
    };
    let idle_seconds: i64 = i64::try_from(idle.as_secs()).unwrap_or(i64::MAX);
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if now.saturating_sub(metadata.ctime()) < idle_seconds {
            continue;
        }
        if entry.file_name().to_string_lossy().starts_with(PARTIAL_PREFIX) {
            remove_name(&path);
        } else if path != keep && metadata.nlink() == 1 {
            after_decision(path.as_path());
            remove_copy_nobody_linked(store, path.as_path());
        }
    }
}

/// Remove the name `path`, saying so when that fails for another reason than its being gone already.
fn remove_name(path: &Path) {
    if let Err(error) = std::fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!("wrapper store: could not remove {} ({error})", path.display());
    }
}

/// Remove the finished copy `path` unless a fixture linked it since its link count was read as one; then its name
///  stays. See `remove_idle_builds_at`.
fn remove_copy_nobody_linked(store: &Path, path: &Path) {
    use std::os::unix::fs::MetadataExt;
    let away: PathBuf = match reserve_partial(store, &PARTIALS_STARTED) {
        Ok(reserved) => reserved,
        Err(error) => {
            eprintln!("wrapper store: could not set {} aside ({error})", path.display());
            return;
        }
    };
    if let Err(error) = std::fs::rename(path, &away) {
        if error.kind() != std::io::ErrorKind::NotFound {
            eprintln!("wrapper store: could not set {} aside ({error})", path.display());
        }
        remove_name(&away);
        return;
    }
    if std::fs::metadata(&away).is_ok_and(|metadata| { return metadata.nlink() != 1; })
        && let Err(error) = std::fs::hard_link(&away, path)
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        eprintln!("wrapper store: could not restore {} ({error})", path.display());
    }
    remove_name(&away);
}
