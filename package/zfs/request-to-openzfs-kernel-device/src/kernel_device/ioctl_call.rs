//! What:
//!  The one unsafe call in this crate:
//! `libc::ioctl` on the opened device with the
//!  request's number and command address.
//! Why:
//!  The design keeps the unsafe call in one place.
//! Everything the kernel needs
//!  is already in the request's owned buffers,
//! so this file only carries two
//!  numbers and reports the kernel's error number,
//! and every rule about what that
//!  answer means lives in `call_outcome.rs` where a test can read it.

/// The packed request the call carries.
use crate::requests::Request;
/// The error number a failed call leaves.
use std::os::fd::RawFd;

/// Send one request to the device.
///
/// Yields the error number the call failed with,
/// or `None` when the kernel
/// accepted the request.
pub(crate) fn perform(descriptor: RawFd, request: &mut Request) -> Option<i32> {
    let code: libc::c_ulong = request.operation().code() as libc::c_ulong;
    let address: *mut libc::c_void = request.command_abi_address() as *mut libc::c_void;
    // SAFETY: the descriptor is the open `/dev/zfs` handle the caller owns and
    // keeps open for this call. The address is the command buffer's first byte,
    // which the request owns, is 8-byte aligned, and is at least the size of the
    // mirrored `zfs_cmd_t`; the kernel copies exactly its own `sizeof
    // (zfs_cmd_t)` in and out of it. The two list addresses inside that struct
    // point into the request's own input and reply buffers, which the request
    // also keeps alive for the call. The request's number is one of the fixed
    // `ZFS_IOC_*` values, so the kernel reads the argument as a `zfs_cmd_t *`.
    let answer: libc::c_int = unsafe { libc::ioctl(descriptor, code, address) };
    if answer == -1 {
        return Some(
            std::io::Error::last_os_error()
                .raw_os_error()
                .expect("a failed ioctl sets the thread's error number"),
        );
    }
    return None;
}
