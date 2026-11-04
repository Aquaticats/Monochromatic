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
//!  A caller that finds a partial copy still being written waits for the stored copy instead of writing a copy of its
//!  own (`wait_for_publisher`): a wave of tests that start together on a fresh store otherwise writes one 190 MB copy
//!  each, and the write-back of that wave stalled the bounded commands of the tests around it (measured in
//!  `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`, "The first wave on a fresh
//!  store"). The wait is a poll for a file, not a lock: it ends when the stored name exists, when no partial copy has
//!  been written to for `PROGRESS_WINDOW`, or at `WAIT_CAP`, and then the caller copies as before.
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

/// How long a partial copy may go without being written before a caller stops waiting for it and copies for itself:
///  the time a live publisher needs to read its copy back and link it, plus a stall of the volume. A partial copy that a
///  killed run left behind is older than this the next time anyone looks, so it is never waited for.
pub const PROGRESS_WINDOW: Duration = Duration::from_secs(10);

/// The longest a caller waits for another caller's copy in all, however recently that copy was written.
pub const WAIT_CAP: Duration = Duration::from_secs(120);

/// How often a waiting caller looks for the stored copy.
pub const WAIT_POLL: Duration = Duration::from_millis(25);

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
    return shared_copy_waiting(built, name.as_str(), root, &|| {}, &|store, stored| { return wait_for_publisher(store, stored); });
}

/// What:
///  `shared_copy_named` after the caller's `wait` for a copy another caller is writing, which says how it ended.
/// Why:
///  A control passes a wait that decides without a clock, and sees from `before_publish` (which only a caller that
///  copied runs) whether the caller wrote a copy of its own.
///
/// # Errors
///  The errors of `shared_copy`.
pub fn shared_copy_waiting(built: &Path, name: &str, root: &Path, before_publish: &dyn Fn(), wait: &dyn Fn(&Path, &Path) -> WaitEnd) -> std::io::Result<PathBuf> {
    let store: PathBuf = root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store)?;
    wait(store.as_path(), store.join(name).as_path());
    return shared_copy_named(built, name, root, before_publish);
}

/// How a wait for another caller's copy ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitEnd {
    /// The stored copy exists.
    Published,
    /// No partial copy was written to within the progress window: nobody is copying, so the caller copies.
    NobodyCopying,
    /// A copy kept being written to until the cap: the caller stops waiting and copies.
    CapReached,
}

/// What the wait asks of time: what it is, and a pause of one poll. The production clock is the system's; a control
/// passes one that moves only when the wait pauses, so the wait's decisions are tested without a wall clock.
pub trait WaitClock {
    /// The time now.
    fn now(&self) -> std::time::SystemTime;
    /// Wait for `poll`.
    fn pause(&self, poll: Duration);
}

/// The system's clock and sleep.
struct SystemClock;

/// The wait on the system's time.
impl WaitClock for SystemClock {
    /// The system time.
    fn now(&self) -> std::time::SystemTime {
        return std::time::SystemTime::now();
    }

    /// A thread sleep.
    fn pause(&self, poll: Duration) {
        std::thread::sleep(poll);
    }
}

/// What:
///  Whether any partial copy in `store` was written to within the `window` before `now`: a publisher is at work.
/// Why:
///  A partial file's modification time moves with every write, so a live copy always has a recent one, and a copy that a
///  killed run left behind does not. A live copy has no second name until it is published, which keeps a leftover that
///  aliases a published file (`reserve_partial`) from counting. A time in the future (a clock that moved back) counts as
///  recent, and the wait it causes ends at the cap.
fn copy_in_progress(store: &Path, window: Duration, now: std::time::SystemTime) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir(store) else {
        return false;
    };
    return entries.flatten().any(|entry| {
        let Ok(metadata) = entry.metadata() else {
            return false;
        };
        if !entry.file_name().to_string_lossy().starts_with(PARTIAL_PREFIX) || metadata.nlink() != 1 {
            return false;
        }
        return match metadata.modified().map(|modified| { return now.duration_since(modified); }) {
            Ok(Ok(age)) => age < window,
            Ok(Err(_)) => true,
            Err(_) => false,
        };
    });
}

/// What:
///  Wait for another caller's copy to be published, on the system's clock with the production limits.
/// Why:
///  See `wait_for_publisher_on`.
pub fn wait_for_publisher(store: &Path, stored: &Path) -> WaitEnd {
    return wait_for_publisher_on(store, stored, [PROGRESS_WINDOW, WAIT_POLL, WAIT_CAP], &SystemClock);
}

/// What:
///  Wait for another caller's copy to be published: `Published` once `stored` exists, `NobodyCopying` when no partial copy
///  was written to within the window, `CapReached` after the cap. `limits` is the window, the poll and the cap, in that order.
/// Why:
///  Copying the build is the only expensive step of a fixture's creation, and callers that start together on a fresh
///  store all did it, each writing the whole executable to the volume the bounded commands of the same wave sync to.
///  The wait asks only for the stored name and for recent writes, so it holds nothing another caller needs; a caller
///  that gives up simply copies, as it always did, and the link of the final name still decides the winner. Time comes
///  from `clock`, so the end of the wait is a decision a control reads from the result.
pub fn wait_for_publisher_on(store: &Path, stored: &Path, limits: [Duration; 3], clock: &dyn WaitClock) -> WaitEnd {
    let [window, poll, cap] = limits;
    let began: std::time::SystemTime = clock.now();
    loop {
        if stored.exists() {
            return WaitEnd::Published;
        }
        let now: std::time::SystemTime = clock.now();
        if !copy_in_progress(store, window, now) {
            return WaitEnd::NobodyCopying;
        }
        if now.duration_since(began).is_ok_and(|waited| { return waited >= cap; }) {
            return WaitEnd::CapReached;
        }
        clock.pause(poll);
    }
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
    // The copy is on disk before any fixture can run it: the fixtures that wait for it, and the ones that link it later,
    // start bounded commands next, and a command that syncs would otherwise wait for this copy's write-back.
    std::fs::File::open(&partial)?.sync_all()?;
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
