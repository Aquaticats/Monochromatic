//! What:
//!  The lease one filling holds on the names of one build directory:
//!  a lock the operating system keeps for as long as the process that took it lives,
//!  and gives back when it dies.
//! Why:
//!  The marker says a filling wrote the names beside a build directory,
//!  and a marker alone cannot say whether that filling is still at work:
//!  a killed filling and a live one leave the same file.
//!  A lease is proof instead of evidence:
//!  the kernel holds it for the lifetime of the descriptor,
//!  so a filling that was killed holds nothing and the next one may take the names,
//!  while a filling that is still cloning, cleaning or publishing holds the lease and is
//!  never taken for a dead one.
//!  This is the smallest unit a lock can cover, one build directory's three names,
//!  because nothing else is shared between two fillings (`AGENTS.md`,
//!  "Concurrency is lock-free first").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lease = openLease(`${stagingOf(destination)}.claim`); // flock(LOCK_EX | LOCK_NB)
//! // the holder releases it by closing or dying; the file left behind is reused by the next holder
//! ```

use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

/// What the lease file's name is suffixed with,
/// beside the staging directory it is about.
const LEASE_SUFFIX: &str = ".claim";

/// What:
///  The lease file one build directory's names are locked under.
/// Why:
///   It sits beside the staging name,
///  inside the worktree,
///  so the lock is on the same filesystem as the names it protects and no other command can be
///  holding it;
///  the repository's exclude file names the pattern so the person never sees it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaseOf = (staging) => `${staging}${LEASE_SUFFIX}`;
/// ```
pub(crate) fn lease_of(staging: &Path) -> PathBuf {
    let mut named: std::ffi::OsString = staging.as_os_str().to_os_string();
    named.push(LEASE_SUFFIX);
    return PathBuf::from(named);
}

/// What:
///  A lease this run holds,
///  released when it is dropped or when the process dies.
/// Why:
///   The descriptor is the proof;
///  keeping it alive for as long as the filling works on the names is what stops a second filling
///  from concluding the first one is gone.
#[derive(Debug)]
pub(crate) struct TheLease {
    /// The name the lock was taken on,
    /// removed when the names are given back.
    path: PathBuf,
    /// The descriptor the kernel holds the lock for.
    /// Its name begins with an underscore because the descriptor is the lock: nothing reads it, and dropping it releases it.
    _held: std::fs::File,
}

impl TheLease {
    /// The name this lease was taken on.
    pub(crate) fn path(&self) -> &Path {
        return self.path.as_path();
    }
}

/// What:
///  What taking the lease of one build directory's names came to.
/// Why:
///   A caller that holds the lease may treat leftovers as a dead filling's and remove them;
/// a caller that does not must touch nothing,
///  and the two cannot be confused because the kernel decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheClaim = { kind: 'held'; lease: TheLease } | { kind: 'taken' } | { kind: 'refused'; reason: string };
/// ```
#[derive(Debug)]
pub(crate) enum TheClaim {
    /// This run holds the names,
    /// and no other filling does.
    Held(TheLease),
    /// Another filling of this build directory holds them and is still at work.
    Taken,
    /// The lease file could not be opened or locked,
    /// so nothing proves who holds the names.
    Refused(String),
}

/// What:
///  Take the lease of one build directory's names without waiting.
/// Why:
///   The lock is taken without waiting,
///  so a filling that meets a live one answers at once and leaves its names alone;
///  the answer is this run's only proof about another filling,
///  so a refusal that is not somebody else's lock is kept apart from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimTheNames = (staging) => { const file = open(leaseOf(staging), 'a+'); return flock(file, LOCK_EX | LOCK_NB) ? held(file) : errno === EWOULDBLOCK ? taken() : refused(errno); };
/// ```
pub(crate) fn claim_the_names(staging: &Path) -> TheClaim {
    let path: PathBuf = lease_of(staging);
    let opened: std::io::Result<std::fs::File> = {
        use std::os::unix::fs::OpenOptionsExt as _;
        std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(path.as_path())
    };
    let held: std::fs::File = match opened {
        Ok(file) => file,
        Err(error) => return TheClaim::Refused(format!("{} could not be opened ({error})", path.display())),
    };
    // SAFETY: the descriptor belongs to `held`, which outlives the call when the lock is taken.
    let taken: i32 = unsafe { libc::flock(held.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if taken == 0 {
        return TheClaim::Held(TheLease { path, _held: held });
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    return match error.kind() {
        // Another descriptor of this account holds the lease: a filling that is still at work.
        std::io::ErrorKind::WouldBlock => TheClaim::Taken,
        _ => TheClaim::Refused(format!("{} could not be locked ({error})", path.display())),
    };
}

#[cfg(test)]
#[path = "package_repair_prewarm_claim_tests.rs"]
mod tests;
