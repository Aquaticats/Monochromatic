//! A cheap fingerprint of one rule file,
//!  so a changed `.gitignore` is noticed without reading it.

/// Rule files are only inspected,
///  never opened for writing.
use std::{fs, io::ErrorKind, os::unix::fs::MetadataExt, path::Path};

/// What:
///  Inode,
///  length,
///  and modification time (seconds and nanoseconds) of a file;
///  `None` inside means the file does not exist or cannot be inspected.
///  `Option<(u64, u64, i64, i64)>` is a value that is either absent or a tuple of four numbers.
/// Why:
///  A replaced file changes the inode,
///  an edited one its length or time,
///  so two equal stamps mean the rules cannot have changed in a way worth reading again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stamp = { inode: number; length: number; seconds: number; nanoseconds: number } | undefined;
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stamp(
    /// The inode,
    ///  length,
    ///  seconds,
    ///  and nanoseconds of the file,
    ///  or nothing when it is absent or cannot be inspected.
    Option<(u64, u64, i64, i64)>,
);

/// Stamps come only from metadata calls.
impl Stamp {
    /// Stamp a path,
    ///  following symbolic links as git does when it reads a rule file.
    /// A missing file and a file that cannot be inspected both stamp as absent;
    ///  only the second is logged,
    ///  because a rule file that does not exist is the ordinary case.
    pub fn of(path: &Path) -> Self {
        return match fs::metadata(path) {
            Ok(metadata) => Self(Some((
                metadata.ino(),
                metadata.len(),
                metadata.mtime(),
                metadata.mtime_nsec(),
            ))),
            Err(error) if error.kind() == ErrorKind::NotFound => Self(None),
            Err(error) => {
                tracing::debug!(path = %path.display(), %error, "cannot inspect an ignore rule file; treating it as absent");
                Self(None)
            }
        };
    }

    /// Whether the file existed when it was stamped.
    pub fn exists(&self) -> bool {
        return self.0.is_some();
    }
}
