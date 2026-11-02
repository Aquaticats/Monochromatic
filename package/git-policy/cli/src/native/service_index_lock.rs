//! What:
//!  The service's hold on Git's own `index.lock` across a landing: taken first, by exclusive creation, waited for without
//!  polling when another program holds it, and released by moving the new index over the real one.
//! Why:
//!  Git's own `commit` takes the index lock before it moves the branch and gives it up after the index is written. A landing
//!  that moved the branch first and took the lock afterwards left the index showing content a landed commit had replaced for
//!  as long as a foreign `git` held the lock, and a commit prepared from that index could land a revert. The wait for a
//!  foreign holder is event-driven: an inotify watch on the Git directory wakes the service when the lock itself is removed or
//!  moved away, and the only timer is the limit (`CLI_GIT_LOCK_WAIT_SECONDS`) and the repetition of the holder's notice. On
//!  platforms without inotify a held lock fails at once with the plain message. The lock carries a marker with the batch
//!  number and the holding process from the moment it exists until it is removed after the new index is in place, so a
//!  restart can tell its own stale lock from a foreign one wherever the service was killed, and never takes over a lock
//!  whose holder still runs (`doc/planning/cli-git-resident-service.md`, "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lock = await acquireIndexLock(indexPath, deadline, announce); ... await lock.releaseWith(newIndex);
//! ```

use super::index_lock_evidence::{Verdict, classify_index_lock, gather_index_lock_evidence, proven_holder_line, proven_holder_progress_line};
use super::process_identity::{current_birth_identity, process_birth_identity};
use super::service_protocol::FailureCode;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The first bytes of a lock the service holds, followed by the batch number, the holding process and its birth identity.
pub const LOCK_MARKER: &str = "cli-git-service ";

/// The longest marker this reader looks at.
const MARKER_LIMIT: u64 = 512;

/// What follows the lock's file name in the name of a staging file: the marker is written there before it becomes the lock.
pub const STAGED_INFIX: &str = ".cli-git-marker";

/// How often the holder's notice is repeated while the wait lasts.
pub const NOTICE_EVERY: Duration = Duration::from_secs(10);

/// The environment variable that sets how often the holder's line is repeated, in whole seconds.
pub const NOTICE_VARIABLE: &str = "CLI_GIT_WAIT_NOTICE_SECONDS";

/// How many times this process tried to place a lock; tests read it to show a wait does not wake itself.
#[cfg(test)]
pub static PLACE_ATTEMPTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// How often the holder's line is repeated: the variable when it holds a positive number of seconds, ten seconds otherwise.
fn notice_every() -> Duration {
    return std::env::var(NOTICE_VARIABLE).ok().and_then(|text| { return text.trim().parse::<u64>().ok(); }).filter(|seconds| { return *seconds > 0; }).map_or(NOTICE_EVERY, Duration::from_secs);
}

/// What the wait tells the clients of the batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Announce {
    /// A line for the person's terminal: who holds the lock and how long the wait lasts.
    Waiting(String),
    /// Who held the lock when the wait ran out, in words that fit "... is using this repository's index".
    Holder(String),
}

/// Why the lock was not taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockFailure {
    /// The code the clients are answered with.
    pub code: FailureCode,
}

/// The service's hold on one index lock; removed unless released.
#[derive(Debug)]
pub struct IndexLock {
    lock: PathBuf,
    index: PathBuf,
    armed: bool,
}

/// What a service marker names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Marked {
    /// The batch the lock was taken for.
    pub seq: u64,
    /// The holding process and its birth identity; none in a marker a release before this one wrote.
    pub holder: Option<(i64, String)>,
}

/// What:
///  The marker this process writes for batch `seq`: the batch, the process and its birth identity, one line.
/// Why:
///   A restart reclaims a marked lock only when that process is gone, so a slow but live holder keeps its lock; without a
///  birth identity the marker names no holder and reads as a dead instance's, as markers did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git-service ${seq} ${process.pid} ${birth}\n`
/// ```
pub fn marker_text(seq: u64) -> String {
    return match current_birth_identity() {
        Ok(birth) => format!("{LOCK_MARKER}{seq} {} {birth}\n", std::process::id()),
        Err(_) => format!("{LOCK_MARKER}{seq}\n"),
    };
}

/// What:
///  The batch and holder a lock's text names, or nothing when it is not a service marker.
/// Why:
///   A marker without a holder is one a release before this one wrote; one with a malformed holder is not the service's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseMarker(text) // { seq, holder?: [pid, birth] } | undefined
/// ```
pub fn parse_marker(text: &str) -> Option<Marked> {
    let rest: &str = text.strip_prefix(LOCK_MARKER)?;
    let line: &str = rest.strip_suffix('\n').unwrap_or(rest);
    let mut parts = line.splitn(3, ' ');
    let seq: u64 = parts.next()?.parse::<u64>().ok()?;
    let holder: Option<(i64, String)> = match (parts.next(), parts.next()) {
        (None, None) => None,
        (Some(pid), Some(birth)) if !birth.is_empty() => Some((pid.parse::<i64>().ok()?, birth.to_string())),
        _ => return None,
    };
    return Some(Marked { seq, holder });
}

/// Whether the process a marker names is gone: no process under that number, or another one born since. A question that
///  cannot be answered keeps the lock, which is the safe side.
fn holder_is_gone(holder: Option<&(i64, String)>) -> bool {
    let Some((pid, birth)) = holder else { return true };
    return match process_birth_identity(*pid) {
        Ok(None) => true,
        Ok(Some(now)) => now != *birth,
        Err(_) => false,
    };
}

/// The marker of the file at `path`, when it holds one.
fn marker_of(path: &Path) -> Option<Marked> {
    let mut text: String = String::new();
    std::fs::File::open(path).ok()?.take(MARKER_LIMIT).read_to_string(&mut text).ok()?;
    return parse_marker(text.as_str());
}

/// Whether the lock at `lock` is the service's and its holder is gone, so it may be removed.
fn is_reclaimable(lock: &Path) -> bool {
    return marker_of(lock).is_some_and(|marked| { return holder_is_gone(marked.holder.as_ref()); });
}

/// An inotify watch on the directory of the lock; nothing on platforms without inotify.
pub struct Watch {
    #[cfg(target_os = "linux")]
    descriptor: i32,
}

/// What:
///  Whether a buffer of inotify events names `wanted`, or reports an overflow that may have hidden it.
/// Why:
///   The watch covers the whole Git directory, where Git and the service create and remove other files all the time; only
///  the lock's own removal is a reason to try again, so a wait is not woken by its own staging file or a neighbour's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// events.some((event) => event.overflow || event.name === wanted)
/// ```
#[cfg(target_os = "linux")]
pub fn events_name(buffer: &[u8], wanted: &[u8]) -> bool {
    let header: usize = std::mem::size_of::<libc::inotify_event>();
    let mut at: usize = 0;
    while at + header <= buffer.len() {
        let field = |offset: usize| -> u32 {
            let mut bytes: [u8; 4] = [0; 4];
            bytes.copy_from_slice(&buffer[at + offset..at + offset + 4]);
            return u32::from_ne_bytes(bytes);
        };
        if field(4) & libc::IN_Q_OVERFLOW != 0 {
            return true;
        }
        let length: usize = usize::try_from(field(12)).unwrap_or(usize::MAX);
        let start: usize = at + header;
        let end: usize = start.saturating_add(length).min(buffer.len());
        let name: &[u8] = buffer[start..end].split(|byte| { return *byte == 0; }).next().unwrap_or(&[]);
        if name == wanted {
            return true;
        }
        at = start.saturating_add(length);
    }
    return false;
}

#[cfg(target_os = "linux")]
impl Watch {
    pub fn on(directory: &Path) -> Option<Watch> {
        use std::os::unix::ffi::OsStrExt;
        // SAFETY: plain system calls with valid arguments; the descriptor is closed in `Drop`.
        let descriptor: i32 = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
        if descriptor < 0 {
            return None;
        }
        let path = std::ffi::CString::new(directory.as_os_str().as_bytes()).ok()?;
        // SAFETY: `path` is a valid NUL-terminated string for the call.
        let added: i32 = unsafe { libc::inotify_add_watch(descriptor, path.as_ptr(), libc::IN_DELETE | libc::IN_MOVED_FROM) };
        if added < 0 {
            // SAFETY: the descriptor was just opened and nothing else owns it.
            unsafe { libc::close(descriptor) };
            return None;
        }
        return Some(Watch { descriptor });
    }

    /// Blocks until the file `name` in the directory was removed or moved away, or `timeout` passed; either way the caller
    ///  looks again. Events for other files are drained and the wait goes on.
    pub fn wait(&self, timeout: Duration, name: &std::ffi::OsStr) {
        use std::os::unix::ffi::OsStrExt;
        let deadline: Instant = Instant::now() + timeout;
        loop {
            let left: Duration = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return;
            }
            let mut entry = libc::pollfd { fd: self.descriptor, events: libc::POLLIN, revents: 0 };
            let milliseconds: i32 = i32::try_from(left.as_millis()).unwrap_or(i32::MAX).max(1);
            // SAFETY: `entry` is valid for the one descriptor named by the count.
            let ready: i32 = unsafe { libc::poll(&raw mut entry, 1, milliseconds) };
            if ready <= 0 {
                return;
            }
            let mut buffer: [u8; 4096] = [0; 4096];
            // SAFETY: the buffer is valid for its length.
            let read: isize = unsafe { libc::read(self.descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
            let Ok(count) = usize::try_from(read) else { return };
            if count == 0 || events_name(&buffer[..count], name.as_bytes()) {
                return;
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for Watch {
    fn drop(&mut self) {
        // SAFETY: the descriptor belongs to this watch and is closed once.
        unsafe { libc::close(self.descriptor) };
    }
}

#[cfg(not(target_os = "linux"))]
impl Watch {
    pub fn on(_directory: &Path) -> Option<Watch> {
        return None;
    }

    pub fn wait(&self, _timeout: Duration, _name: &std::ffi::OsStr) {}
}

/// The staging file this process writes for batch `seq` of the lock at `lock`: named by process and batch, so no attempt
///  ever opens a name another attempt left, which may be the lock's own file.
pub fn staged_path_for(lock: &Path, seq: u64) -> PathBuf {
    let mut name = lock.as_os_str().to_os_string();
    name.push(format!("{STAGED_INFIX}-{}-{seq}", std::process::id()));
    return PathBuf::from(name);
}

/// Renames `from` to `to` unless `to` exists: one step that is as exclusive as `create_new` and leaves no second name.
#[cfg(target_os = "linux")]
fn rename_noreplace(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target = std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe { libc::renameat2(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, target.as_ptr(), libc::RENAME_NOREPLACE) };
    return if done == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

#[cfg(not(target_os = "linux"))]
fn rename_noreplace(_from: &Path, _to: &Path) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

/// Whether a failed rename without replacing means the filesystem cannot do one (renameat2(2): `EINVAL`), not that the lock
///  exists or storage failed.
fn rename_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EINVAL || code == libc::ENOSYS || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// Whether a failed hard link means the filesystem makes none (link(2): `EPERM`), not that the lock exists or storage failed.
fn links_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EPERM || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// What:
///  Makes the marked `staged` file the lock at `lock`; `AlreadyExists` when any process holds it, and `staged` then stays for
///  the next attempt.
/// Why:
///   Git's lock is taken by exclusive creation, and a restart tells the service's own stale lock from a foreign one only by
///  its marker, so the lock must never exist without it. A rename that does not replace is exclusive and atomic and leaves
///  no second name; where the filesystem has none, a hard link is exclusive too and its staging name is removed after; where
///  it has neither, exclusive creation and one write remain, a failed write removes the lock again, and only a kill between
///  the creation and the write can leave an unmarked lock, which no restart takes over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameNoReplace(staged, lock) ?? await link(staged, lock) ?? await createAndWrite(lock, marker)
/// ```
fn place_marked(staged: &Path, lock: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    PLACE_ATTEMPTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    match rename_noreplace(staged, lock) {
        Ok(()) => return Ok(()),
        Err(error) if !rename_unsupported(&error) => return Err(error),
        Err(_) => {}
    }
    match std::fs::hard_link(staged, lock) {
        Ok(()) => {
            let _removed = std::fs::remove_file(staged);
            return Ok(());
        }
        Err(error) if !links_unsupported(&error) => return Err(error),
        Err(_) => {}
    }
    let marker: Vec<u8> = std::fs::read(staged)?;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(lock)?;
    if let Err(error) = file.write_all(marker.as_slice()) {
        drop(file);
        let _removed = std::fs::remove_file(lock);
        return Err(error);
    }
    let _removed = std::fs::remove_file(staged);
    return Ok(());
}

/// What:
///  Removes the staging files of the lock at `lock` that a killed attempt left: the fixed name an earlier release used, and
///  any whose writer is gone.
/// Why:
///   A kill before the rename leaves the staging file behind; a live writer's file is its own and stays.
fn remove_dead_staging(lock: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let (Some(directory), Some(file)) = (lock.parent(), lock.file_name()) else { return };
    let mut prefix: Vec<u8> = file.as_bytes().to_vec();
    prefix.extend_from_slice(STAGED_INFIX.as_bytes());
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let bytes: &[u8] = name.as_bytes();
        if !bytes.starts_with(prefix.as_slice()) {
            continue;
        }
        let path: PathBuf = entry.path();
        let gone: bool = bytes.len() == prefix.len() || marker_of(path.as_path()).is_none_or(|marked| { return holder_is_gone(marked.holder.as_ref()); });
        if gone {
            let _removed = std::fs::remove_file(path);
        }
    }
}

/// What:
///  One attempt to take the lock, with no wait.
/// Why:
///   Finishing a batch a killed service left open must not hold up the next batch for the whole limit; a foreign holder, or
///  a service holder that still runs, leaves the batch open for the next round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryAcquireIndexLock(index, seq)
/// ```
pub fn try_acquire(index: &Path, seq: u64) -> Option<IndexLock> {
    let lock: PathBuf = lock_path_of(index);
    let staged: PathBuf = staged_path_for(&lock, seq);
    std::fs::write(&staged, marker_text(seq).as_bytes()).ok()?;
    let placed: std::io::Result<()> = match place_marked(&staged, &lock) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && is_reclaimable(&lock) => {
            let _removed = std::fs::remove_file(&lock);
            place_marked(&staged, &lock)
        }
        other => other,
    };
    let _removed = std::fs::remove_file(&staged);
    placed.ok()?;
    remove_dead_staging(&lock);
    return Some(IndexLock { lock, index: index.to_path_buf(), armed: true });
}

/// The path of the lock of an index.
pub fn lock_path_of(index: &Path) -> PathBuf {
    let mut name = index.as_os_str().to_os_string();
    name.push(".lock");
    return PathBuf::from(name);
}

/// The words for the process holding the lock, from the evidence Git's lock leaves.
fn holder_words(pid: i64, command: Option<&str>) -> String {
    return match command {
        Some(name) => format!("Process {pid} ({name})"),
        None => format!("Process {pid}"),
    };
}

/// What the evidence says about the holder: its notice line for the person and its words, when a live holder is proven.
fn holder_notice(index: &Path, waited: Duration, limit: Duration, first: bool) -> (Option<String>, Option<String>) {
    let Ok(Some(evidence)) = gather_index_lock_evidence(index) else { return (None, None) };
    let Verdict::ProvenAlive { pid, by_descriptor, command } = classify_index_lock(&evidence) else {
        let line: String = format!(
            "cli-git: waiting for {}, whose owner cannot be proven alive, after {} seconds; giving up after {} seconds.\n",
            evidence.lock_path.display(),
            waited.as_secs(),
            limit.as_secs()
        );
        return (Some(line), None);
    };
    let line: String = if first {
        proven_holder_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), limit.as_secs())
    } else {
        proven_holder_progress_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), waited.as_secs(), limit.as_secs())
    };
    return (Some(line), Some(holder_words(pid, command.as_deref())));
}

/// What:
///  Takes `index.lock` by exclusive creation, waiting for a foreign holder until the limit.
/// Why:
///   Git's own protocol: creation fails while any process holds the lock. The wait blocks on an inotify watch set before the
///  first attempt, so a release between the attempt and the wait is not missed, and it wakes only for the lock itself. The
///  marker is written once, before the first attempt, so the attempts themselves create and remove nothing in the watched
///  directory. When the limit passes, nothing was changed; the clients learn who held the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireIndexLock(index, marker, limit, announce)
/// ```
///
/// # Errors
///
/// `LockFailure` with `IndexInUse` when the lock stayed held to the limit (or cannot be waited for), and with the storage
///  code when the lock cannot be created.
pub fn acquire(index: &Path, seq: u64, limit: Duration, announce: &dyn Fn(Announce)) -> Result<IndexLock, LockFailure> {
    return acquire_noting_holder(index, seq, limit, announce, &mut None);
}

/// What:
///  `acquire`, leaving the words for the holder the evidence named in `holder` while the lock was waited for.
/// Why:
///   The landing's log line names who held the index when a commit waited for it, also when the wait ended in a success,
///  which `acquire` alone reports to no one (the clients are told only when the wait fails).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireIndexLock(index, marker, limit, announce, (who) => { holder = who })
/// ```
///
/// # Errors
///
/// As `acquire`.
pub fn acquire_noting_holder(
    index: &Path,
    seq: u64,
    limit: Duration,
    announce: &dyn Fn(Announce),
    holder: &mut Option<String>,
) -> Result<IndexLock, LockFailure> {
    let lock: PathBuf = lock_path_of(index);
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let watch: Option<Watch> = Watch::on(directory);
    let staged: PathBuf = staged_path_for(&lock, seq);
    std::fs::write(&staged, marker_text(seq).as_bytes()).map_err(|error| { return LockFailure { code: super::service_land_follow::storage_failure(&error) }; })?;
    let mut waiting: Waiting<'_> = Waiting { limit, announce, holder };
    let outcome: Result<(), LockFailure> = wait_and_place(&staged, &lock, lock_name.as_os_str(), index, watch.as_ref(), &mut waiting);
    let _removed = std::fs::remove_file(&staged);
    outcome?;
    remove_dead_staging(&lock);
    return Ok(IndexLock { lock, index: index.to_path_buf(), armed: true });
}

/// How long the wait lasts, whom it tells, and where it leaves the holder's words.
struct Waiting<'a> {
    /// The longest wait.
    limit: Duration,
    /// Tells the clients what is waited for.
    announce: &'a dyn Fn(Announce),
    /// The words for the holder, when the evidence names one.
    holder: &'a mut Option<String>,
}

/// The attempts and the wait of `acquire`, with the staging file already written.
fn wait_and_place(
    staged: &Path,
    lock: &Path,
    lock_name: &std::ffi::OsStr,
    index: &Path,
    watch: Option<&Watch>,
    waiting: &mut Waiting<'_>,
) -> Result<(), LockFailure> {
    let limit: Duration = waiting.limit;
    let announce: &dyn Fn(Announce) = waiting.announce;
    let began: Instant = Instant::now();
    let mut first: bool = true;
    let mut last_notice: Instant = began;
    loop {
        match place_marked(staged, lock) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // A lock with the service's marker whose holder is gone is a dead instance's: it is removed, and the
                //  attempt is made again at once. A live holder of any release is waited for like a foreign one.
                if is_reclaimable(lock) {
                    let _removed = std::fs::remove_file(lock);
                    continue;
                }
            }
            Err(error) => return Err(LockFailure { code: super::service_land_follow::storage_failure(&error) }),
        }
        let waited: Duration = began.elapsed();
        let exhausted: bool = waited >= limit || watch.is_none();
        // A wake for something else in the directory repeats nothing: the line comes at the first look and then every interval.
        let due: bool = first || last_notice.elapsed() >= notice_every();
        let (line, who) = if due || exhausted { holder_notice(index, waited, limit, first) } else { (None, None) };
        if who.is_some() {
            waiting.holder.clone_from(&who);
        }
        if due {
            if let Some(text) = line {
                announce(Announce::Waiting(text));
            }
            first = false;
            last_notice = Instant::now();
        }
        if exhausted {
            announce(Announce::Holder(who.unwrap_or_else(|| { return String::from("Another program"); })));
            return Err(LockFailure { code: FailureCode::IndexInUse });
        }
        if let Some(active) = watch {
            active.wait(notice_every().saturating_sub(last_notice.elapsed()).max(Duration::from_millis(1)).min(limit.saturating_sub(waited)), lock_name);
        }
    }
}

impl IndexLock {
    /// What:
    ///  Installs `built`, which sits in the index's own directory, as the index by one rename, then gives the lock up.
    /// Why:
    ///   A reader sees the old index or the new one and never a part of either, as with Git's rename of its lock. The lock,
    ///  still marked, is removed only after the rename, so a kill between the two leaves a lock the next instance knows as the
    ///  service's own. Copying the new index over the lock first left a lock without the marker for as long as the copy ran,
    ///  and a kill then left a lock no restart would take over.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.releaseWith(built) // rename(built, index); unlink(lock)
    /// ```
    ///
    /// # Errors
    ///
    /// The storage failure when the new index cannot be put in place; the lock is then removed and the old index stays.
    pub fn release_with(mut self, built: &Path) -> Result<(), std::io::Error> {
        // No index existed and nothing was staged when `built` is absent: there is nothing to install.
        if built.exists() {
            std::fs::rename(built, &self.index)?;
        }
        std::fs::remove_file(&self.lock)?;
        self.armed = false;
        return Ok(());
    }
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        if self.armed {
            let _removed = std::fs::remove_file(&self.lock);
        }
    }
}

/// What:
///  Removes an index lock left by a service that was killed, when it still carries the service's marker for `seq` and its
///  holder is gone.
/// Why:
///   Only the marker proves the lock is ours; a foreign lock, one a Git holds, or one whose holder still runs is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStaleLock(index, seq)
/// ```
pub fn remove_stale(index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    if marker_of(lock.as_path()).is_some_and(|marked| { return marked.seq == seq && holder_is_gone(marked.holder.as_ref()); }) {
        let _removed = std::fs::remove_file(&lock);
    }
}

/// Index lock controls stay out of the release executable.
#[cfg(test)]
#[path = "service_index_lock_tests.rs"]
mod tests;
