//! What:
//!  The service's hold on Git's own `index.lock` across a landing: taken first, by exclusive creation, waited for without
//!  polling when another program holds it, and released by moving the new index over the real one.
//! Why:
//!  Git's own `commit` takes the index lock before it moves the branch and gives it up after the index is written. A landing
//!  that moved the branch first and took the lock afterwards left the index showing content a landed commit had replaced for
//!  as long as a foreign `git` held the lock, and a commit prepared from that index could land a revert. The wait for a
//!  foreign holder is event-driven: an inotify watch on the Git directory wakes the service when the lock is removed or moved
//!  away, and the only timer is the limit (`CLI_GIT_LOCK_WAIT_SECONDS`) and the repetition of the holder's notice. On
//!  platforms without inotify a held lock fails at once with the plain message. The lock carries a marker with the batch
//!  number while the service holds it, so a restart can tell its own stale lock from a foreign one
//!  (`doc/planning/cli-git-resident-service.md`, "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lock = await acquireIndexLock(indexPath, deadline, announce); ... await lock.releaseWith(newIndex);
//! ```

use super::index_lock_evidence::{Verdict, classify_index_lock, gather_index_lock_evidence, proven_holder_line, proven_holder_progress_line};
use super::service_protocol::FailureCode;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The first bytes of a lock the service holds, followed by the batch number.
pub const LOCK_MARKER: &str = "cli-git-service ";

/// How often the holder's notice is repeated while the wait lasts.
pub const NOTICE_EVERY: Duration = Duration::from_secs(10);

/// The environment variable that sets how often the holder's line is repeated, in whole seconds.
pub const NOTICE_VARIABLE: &str = "CLI_GIT_WAIT_NOTICE_SECONDS";

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

/// An inotify watch on the directory of the lock; nothing on platforms without inotify.
struct Watch {
    #[cfg(target_os = "linux")]
    descriptor: i32,
}

#[cfg(target_os = "linux")]
impl Watch {
    fn on(directory: &Path) -> Option<Watch> {
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

    /// Blocks until something was removed or moved away in the directory, or `timeout` passed; either way the caller looks again.
    fn wait(&self, timeout: Duration) {
        let mut entry = libc::pollfd { fd: self.descriptor, events: libc::POLLIN, revents: 0 };
        let milliseconds: i32 = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
        // SAFETY: `entry` is valid for the one descriptor named by the count.
        let ready: i32 = unsafe { libc::poll(&raw mut entry, 1, milliseconds) };
        if ready > 0 {
            let mut buffer: [u8; 4096] = [0; 4096];
            // SAFETY: the buffer is valid for its length; the events are only drained.
            let _read = unsafe { libc::read(self.descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
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
    fn on(_directory: &Path) -> Option<Watch> {
        return None;
    }

    fn wait(&self, _timeout: Duration) {}
}

/// What:
///  One attempt to take the lock, with no wait.
/// Why:
///   Finishing a batch a killed service left open must not hold up the next batch for the whole limit; a foreign holder leaves
///  the batch open for the next round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryAcquireIndexLock(index, seq)
/// ```
pub fn try_acquire(index: &Path, seq: u64) -> Option<IndexLock> {
    let lock: PathBuf = lock_path_of(index);
    let mut file = match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && is_services_own(&lock) => {
            let _removed = std::fs::remove_file(&lock);
            std::fs::OpenOptions::new().write(true).create_new(true).open(&lock).ok()?
        }
        Err(_) => return None,
    };
    let _marked = writeln!(file, "{LOCK_MARKER}{seq}");
    return Some(IndexLock { lock, index: index.to_path_buf(), armed: true });
}

/// Whether the lock at `lock` carries the marker of the service (of any batch).
fn is_services_own(lock: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(lock) else { return false };
    let mut head: [u8; LOCK_MARKER.len()] = [0; LOCK_MARKER.len()];
    return std::io::Read::read_exact(&mut file, &mut head).is_ok() && head == *LOCK_MARKER.as_bytes();
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
///  first attempt, so a release between the attempt and the wait is not missed. When the limit passes, nothing was changed;
///  the clients learn who held the lock.
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
    let lock: PathBuf = lock_path_of(index);
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let watch: Option<Watch> = Watch::on(directory);
    let began: Instant = Instant::now();
    let mut first: bool = true;
    let mut last_notice: Instant = began;
    loop {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock) {
            Ok(mut file) => {
                let _marked = writeln!(file, "{LOCK_MARKER}{seq}");
                return Ok(IndexLock { lock, index: index.to_path_buf(), armed: true });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Only one service runs, and it holds no lock between two landings, so a lock with its marker is a dead
                //  instance's: it is removed, and the attempt is made again at once.
                if is_services_own(&lock) {
                    let _removed = std::fs::remove_file(&lock);
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
        if let Some(active) = watch.as_ref() {
            active.wait(notice_every().saturating_sub(last_notice.elapsed()).max(Duration::from_millis(1)).min(limit.saturating_sub(waited)));
        }
    }
}

impl IndexLock {
    /// What:
    ///  Installs `built` as the index and gives the lock up, by moving the new index over the lock and the lock over the index.
    /// Why:
    ///   Git installs an index the same way, so a reader sees the old index or the new one and never a part of either.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.releaseWith(built)
    /// ```
    ///
    /// # Errors
    ///
    /// The storage failure when the new index cannot be put in place; the lock is then removed and the old index stays.
    pub fn release_with(mut self, built: &Path) -> Result<(), std::io::Error> {
        if built.exists() {
            std::fs::copy(built, &self.lock)?;
            std::fs::rename(&self.lock, &self.index)?;
            let _removed = std::fs::remove_file(built);
        } else {
            // No index existed and nothing was staged: there is nothing to install.
            std::fs::remove_file(&self.lock)?;
        }
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
///  Removes an index lock left by a service that was killed, when it still carries the service's marker for `seq`.
/// Why:
///   Only the marker proves the lock is ours; a foreign lock, or one a Git holds, is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStaleLock(index, seq)
/// ```
pub fn remove_stale(index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    if std::fs::read_to_string(&lock).is_ok_and(|text| { return text == format!("{LOCK_MARKER}{seq}\n"); }) {
        let _removed = std::fs::remove_file(&lock);
    }
}

/// Index lock controls stay out of the release executable.
#[cfg(test)]
#[path = "service_index_lock_tests.rs"]
mod tests;
