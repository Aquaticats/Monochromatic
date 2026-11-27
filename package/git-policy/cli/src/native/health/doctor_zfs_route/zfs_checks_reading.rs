//! What:
//!  What a read of one dataset answered:
//!  the facts,
//!  that the dataset is not there,
//!  or that this caller could not read it.
//! Why:
//!  The three answers decide different checks:
//!  a dataset that is gone is a finding,
//!  a dataset this account may not read is unknown,
//!  and only the facts let a property be judged,
//!  so the rule that reads the kernel's error number lives here rather than
//!  at each call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Reading<T> = { kind: 'read'; value: T } | { kind: 'missing'; why } | { kind: 'unreadable'; why };
//! const reading = read.errno === 2 ? { kind: 'missing', why: read.said } : { kind: 'unreadable', why: read.said };
//! ```

/// The one error number that proves a dataset is gone.
use super::zfs_checks_limits::NOT_FOUND_ERRNO;
/// What a failed read of one dataset answered.
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
//region The reading of one dataset

/// What:
///  What a read of one dataset answered:
///  the facts,
///  that the dataset is not there,
///  or that this caller could not read it.
/// Why:
///  The three answers decide different checks:
///  a dataset that is gone is a finding,
///  a dataset this account may not read is unknown,
///  and only the facts let a property be judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading<T> = { kind: 'read'; value: T } | { kind: 'missing'; why: string } | { kind: 'unreadable'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reading<T> {
    /// The dataset answered with its facts.
    Read(T),
    /// The dataset is not in the pool.
    Missing {
        /// What the read answered.
        why: String,
    },
    /// The caller could not read the dataset.
    Unreadable {
        /// What the read answered.
        why: String,
    },
}

/// What:
///  Whether a failed dataset read means the dataset is gone or that it could
///  not be read.
/// Why:
///  The kernel's `ENOENT` is the one answer that proves absence,
///  and a read with no error number proves nothing about the dataset either
///  way,
///  so the rule lives here rather than at each call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = read.errno === 2 ? { kind: 'missing', why: read.said } : { kind: 'unreadable', why: read.said };
/// ```
pub fn reading_of<T>(read: Result<T, ReadFailure>) -> Reading<T> {
    return match read {
        Ok(value) => Reading::Read(value),
        Err(failure) if failure.errno == Some(NOT_FOUND_ERRNO) => Reading::Missing {
            why: failure.said,
        },
        Err(failure) => Reading::Unreadable {
            why: failure.said,
        },
    };
}

//endregion
