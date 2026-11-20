//! What:
//!  How a read that found its reply too small grows the buffer and asks again,
//! and when it stops asking.
//! Why:
//!  The kernel names the exact size it needs,
//! so one growth is normally enough;
//!  state can change between two calls,
//! so a second growth is allowed,
//! and a
//! third answer that still does not fit is a disagreement this crate refuses
//!  rather than loops on.
//! A read may be asked again because it changed nothing;
//! the
//!  write path never comes through here.

/// The failure type a live call returns.
use super::device_error::DeviceError;
/// The decoded list a read returns.
use crate::nvlist::NvList;
/// The request whose reply grows.
use crate::requests::Request;

/// What one attempt at a read returned.
pub(crate) enum ReadAttempt {
    /// The listing reached its end.
    End,
    /// The kernel filled the reply buffer.
    Reply {
        /// The list it wrote.
        list: NvList,
    },
    /// The kernel named the size the reply needs.
    NeedsBiggerReply {
        /// The size it named.
        requested: u64,
    },
}

/// Most attempts one read makes:
/// the first,
/// and one per growth the kernel asks for.
/// A
/// third answer that still does not fit means the kernel and this crate
///  disagree about the reply,
/// which is reported rather than asked again.
pub(crate) const MAXIMUM_READ_ATTEMPTS: usize = 3;

/// Read `request`,
/// growing its reply and asking again while the kernel names a size
/// the buffer does not hold.
///
/// Yields `None` when the listing reached its end.
///
/// # Throws
///
/// [`DeviceError::UnstableReply`] when the kernel keeps naming a size the
/// buffer already holds,
/// or names one after [`MAXIMUM_READ_ATTEMPTS`] attempts,
/// and
/// [`DeviceError::Request`] when the kernel refuses the read or the grown reply
/// is past the cap.
pub(crate) fn retry_read<F>(
    request: &mut Request,
    mut attempt: F,
) -> Result<Option<NvList>, DeviceError>
where
    F: FnMut(&mut Request) -> Result<ReadAttempt, DeviceError>,
{
    let mut made: usize = 0;
    loop {
        made += 1;
        match attempt(request)? {
            ReadAttempt::End => return Ok(None),
            ReadAttempt::Reply { list } => return Ok(Some(list)),
            ReadAttempt::NeedsBiggerReply { requested } => {
                let held: usize = request.reply().len();
                if requested <= held as u64 || made >= MAXIMUM_READ_ATTEMPTS {
                    return Err(DeviceError::UnstableReply {
                        operation: request.operation(),
                        target: request.target().to_string(),
                        requested,
                        held,
                    });
                }
                request.grow_reply(requested)?;
            }
        }
    }
}
