//! What:
//!  A stand-in platform that runs a hook before every clone and can make every hard link fail with `EXDEV`,
//!  as links between two subvolumes do.
//! Why:
//!  The migration must survive a clone refused at any point and a hard link it cannot make,
//!  so a control asks for that failure at the count it wants and reads which call it reached.
//!  Every other operation answers as the plain stand-in does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Hooked implements Platform { constructor(readonly inner: PlainPlatform, readonly onClone: (n: number) => void) {} }
//! ```

use crate::fs_kind::FilesystemKind;
use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_platform::Platform;
use crate::subvolume_test_support_plain_platform::PlainPlatform;
use std::fs::File;
use std::path::Path;

/// A platform that runs a hook before every clone and can make every hard link fail with `EXDEV`,
/// as links between two
///  subvolumes do.
pub(crate) struct Hooked {
    pub inner: PlainPlatform,
    /// Called with the number of earlier clone calls,
    /// before the clone is refused.
    pub on_clone: Box<dyn Fn(u32)>,
    /// How many clone calls were made.
    pub calls: std::cell::Cell<u32>,
    /// Whether every hard link fails with `EXDEV`.
    pub links_fail: bool,
}

/// Only the clone and the hard link are different.
impl Platform for Hooked {
    fn filesystem_facts(
        &self,
        path: &Path,
    ) -> Result<(FilesystemKind, u64), PrimitiveError> {
        return self.inner.filesystem_facts(path);
    }
    fn is_subvolume_root(
        &self,
        path: &Path,
    ) -> Result<bool, PrimitiveError> {
        return self.inner.is_subvolume_root(path);
    }
    fn create_subvolume(
        &self,
        path: &Path,
    ) -> Result<(), PrimitiveError> {
        return self.inner.create_subvolume(path);
    }
    fn destroy_subvolume(
        &self,
        path: &Path,
    ) -> Result<(), PrimitiveError> {
        return self.inner.destroy_subvolume(path);
    }
    fn removal_allowed(
        &self,
        path: &Path,
    ) -> bool {
        return self.inner.removal_allowed(path);
    }
    fn is_mount_point(
        &self,
        path: &Path,
    ) -> bool {
        return self.inner.is_mount_point(path);
    }
    fn clone_file(
        &self,
        source: &File,
        destination: &File,
        path: &Path,
    ) -> Result<(), PrimitiveError> {
        let calls: u32 = self.calls.get();
        self.calls.set(calls + 1);
        (self.on_clone)(calls);
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(
        &self,
        first: &Path,
        second: &Path,
    ) -> Result<(), PrimitiveError> {
        return self.inner.exchange(first, second);
    }
    fn hard_link(
        &self,
        existing: &Path,
        new: &Path,
    ) -> std::io::Result<()> {
        if self.links_fail {
            return Err(std::io::Error::from_raw_os_error(libc::EXDEV));
        }
        return std::fs::hard_link(existing, new);
    }
    fn flushes_to_disk(&self) -> bool {
        return self.inner.flushes_to_disk();
    }
}

impl Hooked {
    /// A hooked platform with no hook and working links.
    pub(crate) fn new(hook: Box<dyn Fn(u32)>) -> Hooked {
        return Hooked {
            inner: PlainPlatform::new(),
            on_clone: hook,
            calls: std::cell::Cell::new(0),
            links_fail: false,
        };
    }
}
