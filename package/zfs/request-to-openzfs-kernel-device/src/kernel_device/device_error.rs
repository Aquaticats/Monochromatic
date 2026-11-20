//! What:
//!  The failures a live call adds to the request failures:
//! a device that cannot
//!  be opened,
//! the request failure the kernel answered,
//! and the two answers that
//!  show the kernel and this crate disagree about the reply.
//! Why:
//!  A caller decides once whether it can use `/dev/zfs` at all,
//! and after that
//!  every failure is the request's own,
//! carrying the operation,
//! the
//!  dataset and the kernel's error number.
//! Keeping the device's failures in
//!  one type means a caller never matches on an untyped `io::Error` to find
//!  out whether the device was merely absent.

/// The failure type a packed request returns.
use crate::requests::Error;
/// The request identifiers the two disagreement variants name.
use crate::requests::Operation;
/// The formatting traits the diagnostics implement.
use std::fmt;
/// The path type the open failure names.
use std::path::PathBuf;

/// Why a live call could not be made or answered.
#[derive(Debug)]
pub enum DeviceError {
    /// The device file could not be opened.
    Open {
        /// The path that was opened.
        path: PathBuf,
        /// Why the open failed.
        source: std::io::Error,
    },
    /// The request itself failed:
    /// a name,
    /// an encoding,
    /// a reply,
    /// a grown reply,
    /// or the kernel's error number.
    Request {
        /// The request failure.
        source: Error,
    },
    /// The kernel asked for a bigger reply after the buffer had already grown
    /// to the size it named,
    /// or kept asking past the number of attempts this crate
    /// makes.
    UnstableReply {
        /// The request that was answered.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
        /// The size the kernel last asked for.
        requested: u64,
        /// The bytes the buffer holds.
        held: usize,
    },
    /// A read finished without putting a list in the reply buffer.
    MissingReply {
        /// The request that was answered.
        operation: Operation,
        /// The dataset or pool the request named.
        target: String,
    },
}

/// The methods of `DeviceError`.
impl DeviceError {
    /// The kernel's error number,
    /// when the kernel supplied one.
    pub fn errno(&self) -> Option<i32> {
        return match self {
            DeviceError::Open { source, .. } => source.raw_os_error(),
            DeviceError::Request { source } => source.errno(),
            DeviceError::UnstableReply { .. } | DeviceError::MissingReply { .. } => None,
        };
    }

    /// The request failure,
    /// when the kernel answered a request.
    pub fn request(&self) -> Option<&Error> {
        return match self {
            DeviceError::Request { source } => Some(source),
            DeviceError::Open { .. }
            | DeviceError::UnstableReply { .. }
            | DeviceError::MissingReply { .. } => None,
        };
    }
}

/// How the value prints to a person.
impl fmt::Display for DeviceError {
    /// Names the path or the request,
    /// never a bare error number.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            DeviceError::Open { path, source } => write!(
                formatter,
                "opening the OpenZFS device at '{}': {source}",
                path.display()
            ),
            DeviceError::Request { source } => write!(formatter, "{source}"),
            DeviceError::UnstableReply {
                operation,
                target,
                requested,
                held,
            } => write!(
                formatter,
                "{operation} on '{target}': the kernel asked for a {requested} byte reply and the \
                 buffer already holds {held} bytes, so the answer is not trusted"
            ),
            DeviceError::MissingReply { operation, target } => write!(
                formatter,
                "{operation} on '{target}': the kernel finished without a reply list"
            ),
        };
    }
}

/// The standard-error view of the failure.
impl std::error::Error for DeviceError {
    /// The lower failure this error wraps,
    /// if any.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        return match self {
            DeviceError::Open { source, .. } => Some(source),
            DeviceError::Request { source } => Some(source),
            DeviceError::UnstableReply { .. } | DeviceError::MissingReply { .. } => None,
        };
    }
}

/// A request failure is a device failure once a call carries it.
impl From<Error> for DeviceError {
    /// Wraps the request failure without changing what it says.
    fn from(source: Error) -> DeviceError {
        return DeviceError::Request { source };
    }
}
