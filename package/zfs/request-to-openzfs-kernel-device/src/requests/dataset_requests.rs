//! What:
//!  The requests that change one dataset:
//! set properties,
//! destroy it with or
//!  without the defer mark,
//! promote a clone,
//! and clone from an origin.
//! Why:
//!  These are the writes the wrapper's snapshot,
//! clone and removal steps make,
//!  and each carries its parameters in a different place:
//! properties in the
//!  input list,
//! the defer mark in the command union,
//! the origin inside the
//!  input list.
//! Keeping them together shows the three shapes side by side,
//! and
//!  none of them takes a general-purpose property setter.

// The failure type these requests return.
use super::error::Error;
// The typed request numbers.
use super::operation::Operation;
// The request value and the name check.
use super::request::{Request, checked_name};
// The field a request name is bound for.
use crate::names::NameField;
// The codec list type.
use crate::nvlist::NvList;

/// `ZFS_IOC_SET_PROP` on `target`,
/// setting the properties in `properties`.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`,
/// and
/// [`Error::Encode`] when the property list cannot be packed.
pub fn set_prop(target: &str, properties: &NvList) -> Result<Request, Error> {
    let name = checked_name(Operation::SetProp, target, NameField::CommandName, target)?;
    let mut request: Request = Request::new(Operation::SetProp, name);
    request.attach_input(properties)?;
    return Ok(request);
}

/// `ZFS_IOC_DESTROY` of the dataset or snapshot `name`;
/// `defer` only marks it
/// for deferred destruction when the caller asks.
///
/// # Throws
///
/// [`Error::Name`] when the name does not fit `zc_name`.
pub fn destroy(name: &str, defer: bool) -> Result<Request, Error> {
    let checked = checked_name(Operation::Destroy, name, NameField::CommandName, name)?;
    let mut request: Request = Request::new(Operation::Destroy, checked);
    request.command_mut().set_defer_destroy(defer);
    return Ok(request);
}

/// `ZFS_IOC_PROMOTE` of the clone `name`;
/// the kernel answers with any
/// conflicting snapshot name in `zc_string`.
///
/// # Throws
///
/// [`Error::Name`] when the name does not fit `zc_name`.
pub fn promote(name: &str) -> Result<Request, Error> {
    let checked = checked_name(Operation::Promote, name, NameField::CommandName, name)?;
    return Ok(Request::new(Operation::Promote, checked));
}

/// `ZFS_IOC_CLONE` of `origin` as the dataset `target`,
/// with `properties`
/// applied at creation when they are given.
///
/// # Throws
///
/// [`Error::Name`] when the target does not fit `zc_name`,
/// and
/// [`Error::Encode`] when the origin or the property list cannot be packed.
pub fn clone(target: &str, origin: &str, properties: Option<&NvList>) -> Result<Request, Error> {
    let name = checked_name(Operation::Clone, target, NameField::CommandName, target)?;
    let mut input: NvList = NvList::new();
    input.add_str("origin", origin);
    if let Some(list) = properties {
        input.add_nvlist("props", list.clone());
    }
    let mut request: Request = Request::new(Operation::Clone, name);
    request.attach_input(&input)?;
    return Ok(request);
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "dataset_requests_tests.rs"]
mod tests;
