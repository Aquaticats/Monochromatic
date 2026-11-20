//! What:
//!  The read requests:
//! the two listing requests,
//! their cursor,
//! and the three
//!  one-shot reads for identity,
//! delegations and bookmarks.
//! Why:
//!  A read never changes state,
//! so its reply may be read again with a bigger
//!  buffer.
//! The listing requests are the only ones whose end is the kernel's
//!  `ESRCH` rather than a failure,
//! which is why the cursor travels in the
//!  request and not on the handle.

/// The failure type these requests return.
use super::error::Error;
/// The typed request numbers.
use super::operation::Operation;
/// The request value,
/// the name check and the flag list builder.
use super::request::{Request, checked_name, flag_list};
/// The field a request name is bound for.
use crate::names::NameField;

/// `ZFS_IOC_OBJSET_STATS` for `target`:
/// identity,
/// origin and property values.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`.
pub fn objset_stats(target: &str) -> Result<Request, Error> {
    let name = checked_name(
        Operation::ObjsetStats,
        target,
        NameField::CommandName,
        target,
    )?;
    return Ok(Request::new(Operation::ObjsetStats, name));
}

/// `ZFS_IOC_DATASET_LIST_NEXT` for `target`,
/// asking for the entry after
/// `cursor`.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`.
pub fn dataset_list_next(target: &str, cursor: u64) -> Result<Request, Error> {
    let name = checked_name(
        Operation::DatasetListNext,
        target,
        NameField::CommandName,
        target,
    )?;
    let mut request: Request = Request::new(Operation::DatasetListNext, name);
    request.command_mut().zc_cookie = cursor;
    return Ok(request);
}

/// `ZFS_IOC_SNAPSHOT_LIST_NEXT` for `target`,
/// asking for the snapshot after
/// `cursor`.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`.
pub fn snapshot_list_next(target: &str, cursor: u64) -> Result<Request, Error> {
    let name = checked_name(
        Operation::SnapshotListNext,
        target,
        NameField::CommandName,
        target,
    )?;
    let mut request: Request = Request::new(Operation::SnapshotListNext, name);
    request.command_mut().zc_cookie = cursor;
    return Ok(request);
}

/// `ZFS_IOC_GET_BOOKMARKS` for `target`,
/// optionally naming the properties to
/// read;
/// each name travels as a boolean flag in the input list.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`,
/// and
/// [`Error::Encode`] when a property name cannot be packed.
pub fn get_bookmarks(target: &str, properties: &[&str]) -> Result<Request, Error> {
    let name = checked_name(
        Operation::GetBookmarks,
        target,
        NameField::CommandName,
        target,
    )?;
    let mut request: Request = Request::new(Operation::GetBookmarks, name);
    if !properties.is_empty() {
        request.attach_input(&flag_list(properties))?;
    }
    return Ok(request);
}

/// `ZFS_IOC_GET_FSACL` for `target`:
/// the delegations granted on it and its
/// ancestors.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`.
pub fn get_fsacl(target: &str) -> Result<Request, Error> {
    let name = checked_name(Operation::GetFsacl, target, NameField::CommandName, target)?;
    return Ok(Request::new(Operation::GetFsacl, name));
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "read_requests_tests.rs"]
mod tests;
