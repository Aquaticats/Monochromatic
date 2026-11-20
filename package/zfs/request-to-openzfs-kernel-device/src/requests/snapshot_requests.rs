//! What:
//!  The two snapshot requests:
//! create the named snapshots of one pool,
//! and
//!  destroy an explicit set of snapshot names.
//! Why:
//!  Both carry their names as boolean flags in an embedded `snaps` list,
//! and the
//!  destroy request deliberately has no `defer` member:
//! a deferred destroy marks
//!  a snapshot that later removal accounting would still meet,
//! and the design
//!  chose an immediate destroy for the names it verified.
//!  The encoder is where that absence is guaranteed,
//! so the assertion belongs
//!  here,
//! on the bytes the kernel would receive.

/// The failure type these requests return.
use super::error::Error;
/// The typed request numbers.
use super::operation::Operation;
/// The request value,
/// the name check and the flag list builder.
use super::request::{Request, checked_name, flag_list};
/// The field a request name is bound for.
use crate::names::NameField;
/// The codec list type.
use crate::nvlist::NvList;

/// `ZFS_IOC_SNAPSHOT` of `snapshots` in `pool`,
/// with `properties` applied to
/// the snapshots when they are given.
///
/// # Throws
///
/// [`Error::Name`] when the pool does not fit `zc_name`,
/// and
/// [`Error::Encode`] when a snapshot name or property list cannot be packed.
pub fn snapshot(
    pool: &str,
    snapshots: &[&str],
    properties: Option<&NvList>,
) -> Result<Request, Error> {
    let name = checked_name(Operation::Snapshot, pool, NameField::CommandName, pool)?;
    let mut input: NvList = NvList::new();
    input.add_nvlist("snaps", flag_list(snapshots));
    if let Some(list) = properties {
        input.add_nvlist("props", list.clone());
    }
    let mut request: Request = Request::new(Operation::Snapshot, name);
    request.attach_input(&input)?;
    return Ok(request);
}

/// `ZFS_IOC_DESTROY_SNAPS` of the explicit set `snapshots` in `pool`.
///
/// The request carries no `defer` member,
/// so no snapshot is marked for
/// deferred destroy and every name in the set is destroyed now or reported as
/// failed.
///
/// # Throws
///
/// [`Error::Name`] when the pool does not fit `zc_name`,
/// and
/// [`Error::Encode`] when a snapshot name cannot be packed.
pub fn destroy_snaps(pool: &str, snapshots: &[&str]) -> Result<Request, Error> {
    let name = checked_name(Operation::DestroySnaps, pool, NameField::CommandName, pool)?;
    let mut input: NvList = NvList::new();
    input.add_nvlist("snaps", flag_list(snapshots));
    let mut request: Request = Request::new(Operation::DestroySnaps, name);
    request.attach_input(&input)?;
    return Ok(request);
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "snapshot_requests_tests.rs"]
mod tests;
