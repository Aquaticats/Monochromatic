//! What:
//!  What one call's answer means before anything is read again:
//! the error number,
//!  the size the kernel left in the command struct,
//! the bytes the reply buffer
//!  holds,
//! and the per-name failures the reply carried.
//! Why:
//!  The kernel answers a listing's end with `ESRCH`,
//! asks a read for a bigger
//!  buffer with `ENOMEM` and the size it needs,
//! and answers a failed batch with
//!  `EEXIST` plus an entry per name it refused.
//! Telling those apart is the whole
//!  retry rule,
//! so it is one pure function per path and the device-free tests
//!  pin every branch without a pool.

/// The failure type a live call returns.
use super::device_error::DeviceError;
/// The per-name failures a write reply carries.
use crate::requests::ElementError;
/// The request failure type and its reply refusal.
use crate::requests::{Error, Operation, ReplyRefusal, ReplyRefusalReason};

/// The facts one answer supplies.
pub(crate) struct AnswerFacts<'a> {
    /// The request that was answered.
    pub operation: Operation,
    /// The dataset or pool the request named.
    pub target: &'a str,
    /// The error number the call failed with.
    pub errno: i32,
    /// The size the kernel left in the reply size field.
    pub reported_reply_size: u64,
    /// The bytes the reply buffer holds.
    pub held: usize,
}

/// What a failed read means.
#[derive(Debug)]
pub(crate) enum ReadFailure {
    /// The listing reached its end,
    /// which is not a failure.
    End,
    /// The reply buffer was too small,
    /// and the kernel named the size it needs.
    NeedsBiggerReply {
        /// The size the kernel named.
        requested: u64,
    },
    /// The kernel refused the request,
    /// with the operation,
    /// target and error number.
    Failed {
        /// The refusal.
        source: DeviceError,
    },
}

/// Read a failed call as the end of a listing,
/// a request for a bigger reply,
/// or a failure.
pub(crate) fn interpret_read_failure(facts: AnswerFacts<'_>) -> ReadFailure {
    if facts.operation.lists() && facts.errno == libc::ESRCH {
        return ReadFailure::End;
    }
    if facts.errno == libc::ENOMEM && facts.reported_reply_size > facts.held as u64 {
        return ReadFailure::NeedsBiggerReply {
            requested: facts.reported_reply_size,
        };
    }
    return ReadFailure::Failed {
        source: DeviceError::Request {
            source: Error::Call {
                operation: facts.operation,
                target: facts.target.to_string(),
                errno: facts.errno,
            },
        },
    };
}

/// What a failed write means.
///
/// A write whose reply was too small is never replayed:
/// by the time the kernel asks,
/// it may already have changed state.
/// The per-name failures it did supply come with the error number,
/// so a
/// caller sees which names the kernel refused and why.
pub(crate) fn interpret_write_failure(
    facts: AnswerFacts<'_>,
    elements: Vec<ElementError>,
) -> DeviceError {
    if facts.errno == libc::ENOMEM && facts.reported_reply_size > facts.held as u64 {
        return DeviceError::Request {
            source: Error::Refuse {
                operation: facts.operation,
                target: facts.target.to_string(),
                source: ReplyRefusal {
                    requested: facts.reported_reply_size,
                    held: facts.held,
                    limit: crate::requests::MAXIMUM_REPLY_SIZE,
                    reason: ReplyRefusalReason::NeverReplayAWrite,
                },
            },
        };
    }
    if !elements.is_empty() {
        return DeviceError::Request {
            source: Error::Elements {
                operation: facts.operation,
                target: facts.target.to_string(),
                errno: facts.errno,
                elements,
            },
        };
    }
    return DeviceError::Request {
        source: Error::Call {
            operation: facts.operation,
            target: facts.target.to_string(),
            errno: facts.errno,
        },
    };
}

/// A read that finished without a list is a disagreement,
/// not an empty answer.
pub(crate) fn require_reply(
    operation: Operation,
    target: &str,
    reply: Option<crate::nvlist::NvList>,
) -> Result<crate::nvlist::NvList, DeviceError> {
    return match reply {
        Some(list) => Ok(list),
        None => Err(DeviceError::MissingReply {
            operation,
            target: target.to_string(),
        }),
    };
}
