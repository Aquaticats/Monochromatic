//! What:
//!  The one failure type a request returns:
//! the operation,
//! the dataset or pool
//!  it named,
//! the kernel's error number,
//! and any per-name errors the kernel
//!  supplied.
//! Why:
//!  A caller of a snapshot batch needs to know which snapshot failed and why,
//!  and an operator reading a refusal needs the dataset's name and the error in
//!  `zfs`'s own words.
//! One type carrying all of it means no caller has to
//!  reconstruct the context,
//! and the end-of-listing case can be told apart from
//!  a lost read by the operation it belongs to.

// The per-name failures and their diagnostic suffix.
use super::element_error::{ElementError, elements_suffix};
// The typed request numbers.
use super::operation::Operation;
// The reply decode error and the growth refusal.
use super::reply::{ReplyError, ReplyRefusal};
// The name refusal a request reports.
use crate::names::NameError;
// The codec refusal type.
use crate::nvlist::NvError;
// The formatting traits the diagnostics implement.
use std::fmt;

/// A request that could not be built or answered.
#[derive(Debug)]
pub enum Error {
    /// The caller's name does not fit the fixed field it was bound for.
    Name {
        /// The request the name was being built for.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// What was wrong with the text.
        source: NameError,
    },
    /// A name-value list could not be encoded.
    Encode {
        /// The request whose input list failed.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// What was wrong with the list.
        source: NvError,
    },
    /// The kernel's reply could not be decoded.
    Reply {
        /// The request that was answered.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// What was wrong with the reply.
        source: ReplyError,
    },
    /// A reply would have to be read again with a bigger buffer,
    /// which this
    /// request may not do.
    Refuse {
        /// The request that was answered.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// Why the bigger reply was refused.
        source: ReplyRefusal,
    },
    /// The kernel answered with an error number.
    Call {
        /// The request that failed.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// The kernel's error number.
        errno: i32,
    },
    /// The kernel answered with an error number and named the elements that
    /// failed.
    Elements {
        /// The request that failed.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// The kernel's error number for the request as a whole.
        errno: i32,
        /// The per-name errors it supplied.
        elements: Vec<ElementError>,
    },
}

/// The methods of `Error`.
impl Error {
    /// The operation that failed.
    pub fn operation(&self) -> Operation {
        return match self {
            Error::Name { operation, .. }
            | Error::Encode { operation, .. }
            | Error::Reply { operation, .. }
            | Error::Refuse { operation, .. }
            | Error::Call { operation, .. }
            | Error::Elements { operation, .. } => *operation,
        };
    }

    /// The dataset or pool the request named.
    pub fn target(&self) -> &str {
        return match self {
            Error::Name { target, .. }
            | Error::Encode { target, .. }
            | Error::Reply { target, .. }
            | Error::Refuse { target, .. }
            | Error::Call { target, .. }
            | Error::Elements { target, .. } => target,
        };
    }

    /// The kernel's error number,
    /// when the kernel supplied one.
    pub fn errno(&self) -> Option<i32> {
        return match self {
            Error::Call { errno, .. } | Error::Elements { errno, .. } => Some(*errno),
            Error::Name { source, .. } => Some(source.errno()),
            _ => None,
        };
    }

    /// The per-name errors the kernel supplied,
    /// empty when it supplied none.
    pub fn elements(&self) -> &[ElementError] {
        return match self {
            Error::Elements { elements, .. } => elements,
            _ => &[],
        };
    }

    /// Whether this is a listing that reached its end,
    /// which is not a failure.
    ///
    /// The kernel answers an exhausted cursor with `ESRCH`;
    /// the same number on
    /// an operation that does not list is a real failure.
    pub fn is_end_of_list(&self) -> bool {
        return self.operation().lists() && self.errno() == Some(libc::ESRCH);
    }
}

/// How the value prints to a person.
impl fmt::Display for Error {
    /// Prints the operation,
    /// target,
    /// error number and elements.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operation: Operation = self.operation();
        let target: &str = self.target();
        return match self {
            Error::Name { source, .. } => {
                write!(formatter, "{operation} on '{target}': {source}")
            }
            Error::Encode { source, .. } => {
                write!(
                    formatter,
                    "{operation} on '{target}': encoding the request: {source}"
                )
            }
            Error::Reply { source, .. } => {
                write!(formatter, "{operation} on '{target}': {source}")
            }
            Error::Refuse { source, .. } => {
                write!(formatter, "{operation} on '{target}': {source}")
            }
            Error::Call { errno, .. } => write!(
                formatter,
                "{operation} on '{target}' failed: {}{}",
                errno_text(*errno),
                errno_hint(operation, *errno)
            ),
            Error::Elements {
                errno, elements, ..
            } => write!(
                formatter,
                "{operation} on '{target}' failed: {}{}{}",
                errno_text(*errno),
                errno_hint(operation, *errno),
                elements_suffix(elements)
            ),
        };
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for Error {
    /// The lower failure this error wraps,
    /// if any.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        return match self {
            Error::Name { source, .. } => Some(source),
            Error::Encode { source, .. } => Some(source),
            Error::Reply { source, .. } => Some(source),
            Error::Refuse { source, .. } => Some(source),
            Error::Call { .. } | Error::Elements { .. } => None,
        };
    }
}

/// The text `strerror` gives an error number.
fn errno_text(errno: i32) -> String {
    return std::io::Error::from_raw_os_error(errno).to_string();
}

/// A short hint for the error numbers these requests commonly return,
/// empty for
/// the rest.
///
/// `ENOMEM` is also how the kernel asks for a bigger reply buffer:
/// `put_nvlist` answers it when the list it has is larger than
/// `zc_nvlist_dst_size`,
/// and leaves the size it needs there
/// (`module/zfs/zfs_ioctl.c:1437-1447`),
/// so the hint names both causes.
fn errno_hint(operation: Operation, errno: i32) -> &'static str {
    if operation.lists() && errno == libc::ESRCH {
        return " (end of the listing)";
    }
    return match errno {
        libc::EPERM | libc::EACCES => {
            " (need root, or a `zfs allow` delegation for this operation)"
        }
        libc::ENOENT => " (no such pool or dataset)",
        libc::EEXIST => " (already exists)",
        libc::EBUSY => " (busy: mounted, held, or has children)",
        libc::ENAMETOOLONG => " (name too long)",
        libc::EINVAL => " (invalid argument: a bad name, property or snapshot set?)",
        libc::ENOMEM => " (the kernel ran out of memory, or the reply buffer was too small)",
        libc::ENOSPC => " (no space left in the pool)",
        _ => "",
    };
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
