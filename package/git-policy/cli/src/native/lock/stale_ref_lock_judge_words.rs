//! What:
//!  The two facts a judgement puts into words:
//!  what a process number the pid file names is now,
//!  and whether the lock looks older than the boot by the wall clock.
//! Why:
//!  A person reads these words and acts on them,
//!  so each says what is known and never more than that:
//!  a process number nothing answers for is said as such rather than guessed at,
//!  and a time before the boot is said to explain nothing,
//!  because a correction of the clock moves every time a lock and the boot are compared by
//!  (finding S-2 of the cross-vendor review of 2026-11-03T11:33:50-05:00).
//!  They take the facts they word rather than a judge,
//!  so the reader of one sentence needs no judge to follow it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function processStateWords(procRoot: string, pid: number): string;
//! function looksOlderThanBoot(boot: Date | undefined, born: Date | undefined, identity: Identity): boolean;
//! ```

/// `ProcessStart` and the two readers that answer for one process number.
use crate::process_start::{
    ProcessStart,
    linux_start,
    now_ms,
};
/// `Identity` is the file's identity at the moment it was judged.
use crate::stale_ref_lock_identity::Identity;
/// `nanoseconds_to_time` turns a file's stamp into a wall-clock instant.
use crate::stale_ref_lock_judge_environment::nanoseconds_to_time;
/// `Path` is a borrowed filesystem path,
/// and not the owned `PathBuf`.
use std::path::Path;
/// `SystemTime` is a wall-clock instant,
/// and not the monotonic `Instant`.
use std::time::SystemTime;

/// What:
///  What the pid file's number is now,
///  in words:
///  the notice says it and nothing decides on it.
/// Why:
///  A reader who sees "process 42" needs to know whether that process is still there,
///  and a number the process table cannot answer for is said as such rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processStateWords(procRoot: string, pid: number): string;
/// ```
pub(crate) fn process_state_words(
    proc_root: &Path,
    pid: i64,
) -> String {
    return match linux_start(pid, proc_root, now_ms()) {
        ProcessStart::Missing => String::from("has ended"),
        ProcessStart::Unknown(reason) => format!("could not be looked at ({reason})"),
        ProcessStart::Running {
            ..
        } => String::from("is running"),
    };
}

/// What:
///  Whether the lock looks older than the boot by the wall clock.
/// Why:
///  Only words the notice:
///  a forward correction of the clock
///   during the boot makes a lock that a live Git holds look older than the boot,
///  so no lock is removed on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looksOlderThanBoot(boot: Date | undefined, born: Date | undefined, identity: Identity): boolean;
/// ```
pub(crate) fn looks_older_than_boot(
    boot: Option<SystemTime>,
    born: Option<SystemTime>,
    identity: &Identity,
) -> bool {
    let Some(booted_at) = boot else {
        return false;
    };
    let modified: SystemTime = nanoseconds_to_time(identity.modified_ns);
    return match born {
        Some(at) => at < booted_at && modified < booted_at,
        None => false,
    };
}
