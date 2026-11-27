//! What:
//! The one kernel call that makes two names change places:
//! `renameat2` with `RENAME_EXCHANGE`,
//! and the errors that say a file system cannot do it at all.
//! Why:
//! Replacing the copy at a version's name in two renames leaves an instant in which that name is absent,
//! and a process killed in that instant leaves a directory nothing reads
//! (issue 6600).
//! One exchange removes the instant:
//! both names exist throughout,
//! and after the call the version's name holds the finished copy and the hidden name holds the one it replaced.
//! The errors are taken from the kernel's own page for the call
//! (`man 2 renameat2`,
//! read 2026-11-27):
//! "The filesystem does not support one of the flags in flags" is `EINVAL`,
//! "the filesystem containing oldpath does not support renaming of the type requested" is listed with
//! `EPERM or EACCES`,
//! and a kernel without the call answers `ENOSYS`.
//! `EOPNOTSUPP` is answered by a few file systems for the same reason.
//! A file system that refuses the exchange therefore publishes in two renames,
//! and the instant it leaves is repaired by a later install
//! (`leftovers_of_a_killed_install` in the module above this one).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // exchanging-the-two-names.ts
//! export function exchangeTheTwoNames(left: string, right: string): void;  // throws
//! export function cannotExchange(error: NodeJS.ErrnoException): boolean;
//! ```

/// Imports the input and output types a refused call answers with.
use std::io;
/// Imports the borrowed path type the two names arrive as.
use std::path::Path;

/// Imports the call and its flags from `rustix`,
/// which owns the safe wrapper around `renameat2`.
#[cfg(unix)]
use rustix::fs::{
    CWD,
    RenameFlags,
    renameat_with,
};
/// Imports the operating system's error code type,
/// which is how "this file system cannot exchange" is told from "the call failed".
use rustix::io::Errno;

/// What:
/// Makes what `left` and `right` name change places,
/// in one call that either happened or did not.
/// Why:
/// The caller holds two directories it owns,
/// and this call is the whole replacement of the one by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exchangeTheTwoNames(left: string, right: string): void  // throws
/// ```
#[cfg(unix)]
pub(super) fn exchange_the_two_names(
    left: &Path,
    right: &Path,
) -> io::Result<()> {
    return renameat_with(CWD, left, CWD, right, RenameFlags::EXCHANGE).map_err(io::Error::from);
}

/// What:
/// The same step on a platform whose file system calls this crate does not carry:
/// nothing is exchanged.
/// Why:
/// The caller's answer is the same as a file system's that cannot exchange the two names,
/// so it publishes in two renames and the instant they leave is repaired by a later install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exchangeTheTwoNames(): never { throw Object.assign(new Error('unsupported'), { code: 'ENOTSUP' }); }
/// ```
#[cfg(not(unix))]
pub(super) fn exchange_the_two_names(
    _left: &Path,
    _right: &Path,
) -> io::Result<()> {
    return Err(io::Error::from(io::ErrorKind::Unsupported));
}

/// What:
/// Whether an error says the file system cannot exchange two names at all,
/// rather than that this exchange failed.
/// Why:
/// The first is answered by publishing in two renames;
/// the second is reported as it is,
/// because both names are as they were and nothing was published.
/// A denial of permission counts as the first:
/// the two renames then meet the same denial and report it,
/// so falling back cannot hide one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cannotExchange(error: NodeJS.ErrnoException): boolean { return ['EINVAL', 'ENOSYS', 'ENOTSUP', 'EPERM', 'EACCES'].includes(error.code); }
/// ```
pub(super) fn cannot_exchange(error: &io::Error) -> bool {
    if error.kind() == io::ErrorKind::Unsupported {
        return true;
    }
    let Some(code) = Errno::from_io_error(error) else {
        return false;
    };
    return matches!(
        code,
        Errno::INVAL | Errno::NOSYS | Errno::NOTSUP | Errno::PERM | Errno::ACCESS
    );
}
