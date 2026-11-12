//! What:
//!  Which account owns one process of this host's process table.
//! Why:
//!  The kernel closes a process's descriptor directory to another account,
//! and it closes one of this account's too when that process is not dumpable:
//!  the owner of the process's own `/proc` entry tells the two apart,
//!  and a removal may act through the first while the second may hold the file
//!  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its first finding).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const anotherAccountOwns = (process) => statSync(process).uid !== statSync('/proc/self').uid;
//! ```

/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;

/// The beginning of a scan's reason for a process another account owns,
/// whose descriptors this account may not read.
/// A caller that removes a file on a scan tolerates this reason alone,
/// because such a process cannot write into a file of this account's.
#[cfg(unix)]
pub(crate) const OTHER_ACCOUNTS_PROCESS_REASON: &str = "PID of another account ";

/// What:
///  Whether a process belongs to another account than this one,
///  as its `/proc` entry shows.
/// Why:
///  The kernel closes the descriptor directory of a process of this account that is not dumpable,
/// and it also closes the one of every process of another account:
///  the owner of the process's own `/proc` entry tells the two apart,
///  because a process of this account keeps it
///  (measured:
/// `PR_SET_DUMPABLE` leaves `/proc/<pid>` owned by the account and makes `/proc/<pid>/fd` unreadable).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anotherAccountOwns = (process) => statSync(process).uid !== statSync('/proc/self').uid;
/// ```
#[cfg(unix)]
pub(crate) fn another_account_owns(process: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(theirs) = std::fs::metadata(process) else { return false };
    let Ok(ours) = std::fs::metadata("/proc/self") else { return false };
    return theirs.uid() != ours.uid();
}
