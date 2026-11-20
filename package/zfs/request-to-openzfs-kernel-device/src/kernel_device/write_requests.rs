//! What:
//!  The six writes as methods on the open handle:
//! set properties,
//! destroy one
//!  dataset,
//! create snapshots,
//! destroy an explicit snapshot set,
//! clone,
//! and the
//!  promote primitive no production caller may reach.
//! Why:
//!  A write's outcome is unknown the moment the kernel refuses it,
//! so none of
//!  these is ever sent twice;
//! they differ only in what they name and pack,
//! and
//!  the per-name failures a batch returns travel back with the kernel's error
//!  number.
//! Promote is compiled for tests only:
//! the design allows the primitive in
//!  the binding and no production caller,
//! so the live call is behind the test build
//!  rather than behind a comment.

/// The handle they are methods of.
use super::device::ZfsDevice;
/// The failure type these calls return.
use super::device_error::DeviceError;
/// The decoded list a write may answer with.
use crate::nvlist::NvList;
/// The request constructors and the request value.
use crate::requests::{Request, clone, destroy, destroy_snaps, set_prop, snapshot};

/// The methods of `ZfsDevice` that write.
impl ZfsDevice {
    /// `ZFS_IOC_SET_PROP` on `target`,
    /// setting the properties in `properties`.
    ///
    /// # Throws
    ///
    /// What `ZfsDevice::write` throws.
    pub fn set_prop(
        &self,
        target: &str,
        properties: &NvList,
    ) -> Result<Option<NvList>, DeviceError> {
        let mut request: Request = set_prop(target, properties)?;
        return self.write(&mut request);
    }

    /// `ZFS_IOC_DESTROY` of the dataset or snapshot `name`.
    ///
    /// `defer` marks it for deferred destruction and is never set by this
    /// crate's callers;
    /// the parameter exists because the request shape has the
    /// field.
    ///
    /// # Throws
    ///
    /// What `ZfsDevice::write` throws.
    pub fn destroy(&self, name: &str, defer: bool) -> Result<Option<NvList>, DeviceError> {
        let mut request: Request = destroy(name, defer)?;
        return self.write(&mut request);
    }

    /// `ZFS_IOC_SNAPSHOT` of `snapshots` in `pool`,
    /// with `properties` applied to
    /// the snapshots when they are given.
    ///
    /// # Throws
    ///
    /// What `ZfsDevice::write` throws.
    pub fn snapshot(
        &self,
        pool: &str,
        snapshots: &[&str],
        properties: Option<&NvList>,
    ) -> Result<Option<NvList>, DeviceError> {
        let mut request: Request = snapshot(pool, snapshots, properties)?;
        return self.write(&mut request);
    }

    /// `ZFS_IOC_DESTROY_SNAPS` of the explicit set `snapshots` in `pool`,
    /// with no
    /// `defer` member,
    /// so every name is destroyed now or reported as failed.
    ///
    /// # Throws
    ///
    /// What `ZfsDevice::write` throws,
    /// including the per-name failures a
    /// refused set returns.
    pub fn destroy_snaps(
        &self,
        pool: &str,
        snapshots: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        let mut request: Request = destroy_snaps(pool, snapshots)?;
        return self.write(&mut request);
    }

    /// `ZFS_IOC_CLONE` of `origin` as the dataset `target`,
    /// with `properties`
    /// applied at creation when they are given.
    ///
    /// # Throws
    ///
    /// What `ZfsDevice::write` throws.
    pub fn clone(
        &self,
        target: &str,
        origin: &str,
        properties: Option<&NvList>,
    ) -> Result<Option<NvList>, DeviceError> {
        let mut request: Request = clone(target, origin, properties)?;
        return self.write(&mut request);
    }

    /// `ZFS_IOC_PROMOTE` of the clone `name`.
    ///
    /// Compiled for tests only:
    /// the design exposes the primitive and no product
    /// path calls it,
    /// because promoting a worktree clone moves checkout history into
    /// a dataset ordinary removal destroys.
    /// A refusal is a value,
    /// not an error,
    /// because the
    /// kernel answers a name conflict with `EEXIST` and the conflicting
    /// snapshot's name in `zc_string` (`dsl_dataset_promote`,
    /// `module/zfs/dsl_dataset.c` of `zfs-2.4.4`).
    #[cfg(test)]
    pub fn promote(&self, name: &str) -> Promotion {
        let mut request: Request = match crate::requests::promote(name) {
            Ok(request) => request,
            Err(source) => {
                return Promotion::Refused {
                    conflicting_snapshot: String::new(),
                    error: DeviceError::Request { source },
                };
            }
        };
        let outcome = self.write(&mut request);
        let conflicting_snapshot: String = request.command().string();
        return match outcome {
            Ok(_) => Promotion::Promoted,
            Err(error) => Promotion::Refused {
                conflicting_snapshot,
                error,
            },
        };
    }
}

/// What a promotion did.
#[cfg(test)]
pub enum Promotion {
    /// The kernel promoted the clone,
    /// which moved its origin into it.
    Promoted,
    /// The kernel refused the promotion,
    /// naming the snapshot that conflicts when
    /// it is a name conflict.
    Refused {
        /// The conflicting snapshot's name,
        /// empty when the kernel named none.
        conflicting_snapshot: String,
        /// Why the kernel refused.
        error: DeviceError,
    },
}
