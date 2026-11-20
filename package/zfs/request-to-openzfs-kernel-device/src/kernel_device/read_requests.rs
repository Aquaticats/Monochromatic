//! What:
//!  The five reads as methods on the open handle:
//! a dataset's identity and property
//!  values and sources,
//! a step of the child or snapshot listing with its cursor,
//! the
//!  delegations on a dataset,
//! and its bookmarks.
//! Why:
//!  Every read may be asked again with a bigger reply,
//! so all five go through
//!  the one retry rule;
//! what differs is only how the answer is read back,
//! and the
//!  listing steps also carry the kernel's cursor and statistics.

/// The check that a finished read wrote a list.
use super::call_outcome::require_reply;
/// The handle they are methods of.
use super::device::ZfsDevice;
/// The failure type these calls return.
use super::device_error::DeviceError;
/// One step of a listing,
/// and the identity read's statistics with its properties.
use super::listed_object::{DatasetIdentity, ListedObject};
/// The decoded list a read returns.
use crate::nvlist::NvList;
/// The request constructors and the request value.
use crate::requests::{
    Request, dataset_list_next, get_bookmarks, get_fsacl, objset_stats, snapshot_list_next,
};

/// The methods of `ZfsDevice` that read.
impl ZfsDevice {
    /// `ZFS_IOC_OBJSET_STATS` on `target`:
    /// the identity,
    /// the origin and the
    /// property values with their sources.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::read`] throws,
    /// and [`DeviceError::MissingReply`] when the
    /// kernel finished without a property list.
    pub fn objset_stats(&self, target: &str) -> Result<DatasetIdentity, DeviceError> {
        let mut request: Request = objset_stats(target)?;
        let reply: Option<NvList> = self.read(&mut request)?;
        let properties: NvList = require_reply(request.operation(), request.target(), reply)?;
        return Ok(DatasetIdentity {
            stats: request.command().zc_objset_stats,
            properties,
        });
    }

    /// `ZFS_IOC_DATASET_LIST_NEXT`:
    /// the child of `target` after `cursor`,
    /// or
    /// `None` when the listing reached its end.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::read`] throws,
    /// and [`DeviceError::MissingReply`] when a
    /// step arrived without its property list.
    pub fn dataset_list_next(
        &self,
        target: &str,
        cursor: u64,
    ) -> Result<Option<ListedObject>, DeviceError> {
        let mut request: Request = dataset_list_next(target, cursor)?;
        return self.read_listing_step(&mut request);
    }

    /// `ZFS_IOC_SNAPSHOT_LIST_NEXT`:
    /// the snapshot of `target` after `cursor`,
    /// or `None` when the listing reached its end.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::read`] throws,
    /// and [`DeviceError::MissingReply`] when a
    /// step arrived without its property list.
    pub fn snapshot_list_next(
        &self,
        target: &str,
        cursor: u64,
    ) -> Result<Option<ListedObject>, DeviceError> {
        let mut request: Request = snapshot_list_next(target, cursor)?;
        return self.read_listing_step(&mut request);
    }

    /// `ZFS_IOC_GET_FSACL` on `target`:
    /// the delegations granted on it and its
    /// ancestors.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::read`] throws,
    /// and [`DeviceError::MissingReply`] when the
    /// kernel finished without a delegation list.
    pub fn get_fsacl(&self, target: &str) -> Result<NvList, DeviceError> {
        let mut request: Request = get_fsacl(target)?;
        let reply: Option<NvList> = self.read(&mut request)?;
        return require_reply(request.operation(), request.target(), reply);
    }

    /// `ZFS_IOC_GET_BOOKMARKS` on `target`,
    /// asking for `properties` when any are
    /// named.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::read`] throws,
    /// and [`DeviceError::MissingReply`] when the
    /// kernel finished without a bookmark list.
    pub fn get_bookmarks(&self, target: &str, properties: &[&str]) -> Result<NvList, DeviceError> {
        let mut request: Request = get_bookmarks(target, properties)?;
        let reply: Option<NvList> = self.read(&mut request)?;
        return require_reply(request.operation(), request.target(), reply);
    }

    /// Read one listing step and rebuild the name,
    /// cursor,
    /// statistics and
    /// properties the kernel wrote.
    ///
    /// # Throws
    ///
    /// What [`ZfsDevice::read`] throws,
    /// and [`DeviceError::MissingReply`] when a
    /// step arrived without its property list.
    fn read_listing_step(
        &self,
        request: &mut Request,
    ) -> Result<Option<ListedObject>, DeviceError> {
        let Some(reply) = self.read(request)? else {
            return Ok(None);
        };
        let command = request.command();
        return Ok(Some(ListedObject {
            name: command.name(),
            cursor: command.zc_cookie,
            stats: command.zc_objset_stats,
            properties: reply,
        }));
    }
}
