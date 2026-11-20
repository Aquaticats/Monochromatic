//! What:
//!  The open `/dev/zfs` handle:
//! where it was opened,
//! the descriptor it owns,
//!  one attempt at a request,
//! and the two finished paths a caller uses,
//! a read that
//!  may be asked again and a write that never is.
//! Why:
//!  Opening the device is the one step a program cannot skip when OpenZFS is
//!  absent,
//! so it is separate from sending,
//! and the descriptor closes with the
//!  handle.
//! A read's answer is either a list,
//! the end of a listing,
//! or a
//!  request for a bigger buffer;
//! a write's answer is a list or a failure,
//! because a
//!  replayed write could create a second snapshot or destroy a second time.

/// The facts one answer supplies and what they mean.
use super::call_outcome::{
    AnswerFacts, ReadFailure, interpret_read_failure, interpret_write_failure,
};
/// The failure type these calls return.
use super::device_error::DeviceError;
/// The one unsafe call.
use super::ioctl_call;
/// The read retry rule and what one attempt returned.
use super::read_reply::{ReadAttempt, retry_read};
/// The decoded list a call returns.
use crate::nvlist::NvList;
/// The request a call carries and the list it answers with.
use crate::requests::Request;
/// The per-name failures a write reply carries.
use crate::requests::element_errors;
/// The owned descriptor,
/// the file it came from and the raw handle the call needs.
use std::fs::{File, OpenOptions};
/// The descriptor the call needs and the ownership that closes it.
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
/// The path the handle remembers.
use std::path::{Path, PathBuf};

/// The device file a live call is carried over.
use super::DEVICE_PATH;

/// An open `/dev/zfs` handle.
pub struct ZfsDevice {
    /// The path that was opened,
    /// for diagnostics.
    path: PathBuf,
    /// The descriptor every request travels on.
    descriptor: OwnedFd,
}

/// The methods of `ZfsDevice`.
impl ZfsDevice {
    /// Open the OpenZFS device at [`DEVICE_PATH`].
    ///
    /// # Throws
    ///
    /// [`DeviceError::Open`] when the device is absent or refuses the open,
    /// which is what a machine without OpenZFS answers.
    pub fn open() -> Result<ZfsDevice, DeviceError> {
        return ZfsDevice::open_at(Path::new(DEVICE_PATH));
    }

    /// Open an OpenZFS device at `path`,
    /// which a test uses when it must name
    /// the file itself.
    ///
    /// # Throws
    ///
    /// [`DeviceError::Open`] when the device is absent or refuses the open.
    pub fn open_at(path: &Path) -> Result<ZfsDevice, DeviceError> {
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|source| {
                return DeviceError::Open {
                    path: path.to_path_buf(),
                    source,
                };
            })?;
        return Ok(ZfsDevice {
            path: path.to_path_buf(),
            descriptor: OwnedFd::from(file),
        });
    }

    /// The path this handle was opened on.
    pub fn path(&self) -> &Path {
        return &self.path;
    }

    /// The raw descriptor the call travels on.
    fn descriptor(&self) -> RawFd {
        return self.descriptor.as_raw_fd();
    }

    /// Send `request` once and report what one answer means,
    /// without growing the
    /// reply.
    ///
    /// This is the seam the reply-growth test reads:
    /// it sees the kernel ask
    /// for a bigger buffer,
    /// grows it,
    /// and asks once more,
    /// where a caller uses
    /// [`ZfsDevice::read`] and never sees the growth.
    ///
    /// # Throws
    ///
    /// [`DeviceError::Request`] when the kernel refuses the read,
    /// and
    /// [`DeviceError::MissingReply`] when it finished without a list.
    pub(crate) fn attempt_read(&self, request: &mut Request) -> Result<ReadAttempt, DeviceError> {
        let Some(errno) = ioctl_call::perform(self.descriptor(), request) else {
            let reply: Option<NvList> = request.take_reply()?;
            let list: NvList =
                super::call_outcome::require_reply(request.operation(), request.target(), reply)?;
            return Ok(ReadAttempt::Reply { list });
        };
        let facts: AnswerFacts<'_> = AnswerFacts {
            operation: request.operation(),
            target: request.target(),
            errno,
            reported_reply_size: request.command().zc_nvlist_dst_size,
            held: request.reply().len(),
        };
        return match interpret_read_failure(facts) {
            ReadFailure::End => Ok(ReadAttempt::End),
            ReadFailure::NeedsBiggerReply { requested } => {
                Ok(ReadAttempt::NeedsBiggerReply { requested })
            }
            ReadFailure::Failed { source } => Err(source),
        };
    }

    /// Read `request`,
    /// growing its reply and asking again while the kernel names
    /// a size the buffer does not hold.
    ///
    /// Yields `None` when a listing reached its end.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::attempt_read`] and [`retry_read`] throw.
    pub(crate) fn read(&self, request: &mut Request) -> Result<Option<NvList>, DeviceError> {
        return retry_read(request, |asking| {
            return self.attempt_read(asking);
        });
    }

    /// Send `request` once and decode the list the kernel wrote,
    /// if any.
    ///
    /// A write is never sent twice,
    /// so a reply the kernel says was too small is
    /// reported rather than asked for again.
    ///
    /// # Throws
    ///
    /// [`DeviceError::Request`] when the kernel refuses the write,
    /// with the
    /// per-name failures it supplied,
    /// and when the reply it wrote does not decode.
    pub(crate) fn write(&self, request: &mut Request) -> Result<Option<NvList>, DeviceError> {
        let failure: Option<i32> = ioctl_call::perform(self.descriptor(), request);
        let reported: u64 = request.command().zc_nvlist_dst_size;
        let held: usize = request.reply().len();
        let Some(errno) = failure else {
            return Ok(request.take_reply()?);
        };
        // A reply the kernel says is bigger than the buffer is not read: the
        // size field names bytes the buffer does not hold.
        let elements = if reported <= held as u64 {
            let reply: Option<NvList> = request.take_reply()?;
            reply.map_or_else(Vec::new, |list| {
                return element_errors(&list);
            })
        } else {
            Vec::new()
        };
        let facts: AnswerFacts<'_> = AnswerFacts {
            operation: request.operation(),
            target: request.target(),
            errno,
            reported_reply_size: reported,
            held,
        };
        return Err(interpret_write_failure(facts, elements));
    }
}
