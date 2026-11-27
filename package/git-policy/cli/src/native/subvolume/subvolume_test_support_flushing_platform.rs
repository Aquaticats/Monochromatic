//! What:
//!  A stand-in platform that answers that it flushes its writes,
//!  and calls `on_question` before its first answer about the filesystem.
//! Why:
//!  The switch that skips the flushes is the calling thread's,
//!  so a control shows that a call on a platform that flushes puts the flushes back inside a call on a stand-in that
//!  skips them,
//!  and the question is where a control reads which policy is in force inside that call.
//!  Every other operation answers as the plain stand-in does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Flushing implements Platform { constructor(readonly inner: PlainPlatform, readonly onQuestion: () => void) {} }
//! ```

use crate::fs_kind::FilesystemKind;
use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_platform::Platform;
use crate::subvolume_test_support_plain_platform::PlainPlatform;
use std::fs::File;
use std::path::Path;

/// What:
///  A stand-in platform that answers that it flushes its writes,
/// and calls `on_question` before its first answer about the filesystem.
/// Why:
///   The switch that skips the flushes is the calling thread's,
/// so a control has to show that a call on a platform that
///  flushes puts the flushes back inside a call on a stand-in that skips them;
/// the question is where a control reads which policy
///  is in force inside that call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Flushing implements Platform {
///   constructor(readonly inner: PlainPlatform, readonly onQuestion: () => void) {}
///   flushesToDisk() { return true }
/// }
/// ```
pub(crate) struct Flushing {
    /// The stand-in whose answers every operation is delegated to.
    pub inner: PlainPlatform,
    /// Called before the first answer about the filesystem.
    pub on_question: Box<dyn Fn()>,
}

/// Every operation answers as the stand-in,
/// and the platform says it flushes.
impl Platform for Flushing {
    fn filesystem_facts(
        &self,
        path: &Path,
    ) -> Result<(FilesystemKind, u64), PrimitiveError> {
        (self.on_question)();
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
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(
        &self,
        first: &Path,
        second: &Path,
    ) -> Result<(), PrimitiveError> {
        return self.inner.exchange(first, second);
    }
    fn flushes_to_disk(&self) -> bool {
        return true;
    }
}
