//! What:
//!  Which file a name holds,
//! as its device and inode,
//! for the attributes files a content decision reads.
//! Why:
//!   A descriptor's metadata cannot show that the name it was opened through now holds another file,
//! so every attributes read asks the name again after the read and compares the two,
//! and this file is where both sides of that comparison come from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identityOf = (metadata) => [metadata.dev, metadata.ino];
//! ```

use crate::worktree_path_failure_message::subject_failure;
use std::ffi::OsStr;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  Which file a path holds,
/// as its device and inode.
/// Why:
///   A descriptor's metadata cannot show that the name it was opened through now holds another file,
/// so the name is asked
///  again after the read and compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityOf = (metadata) => [metadata.dev, metadata.ino];
/// ```
pub(super) fn identity_of(
    metadata: std::io::Result<std::fs::Metadata>,
    label: &str,
) -> Result<(u64, u64), String> {
    let examined: std::fs::Metadata =
        subject_failure(metadata, &String::from(label), "cannot be examined")?;
    return Ok((examined.dev(), examined.ino()));
}

/// Which file the path `path` holds,
/// following a link as the open does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityAtPath = (path) => { const found = statSync(path); return [found.dev, found.ino]; };
/// ```
pub(super) fn identity_at_path(path: &Path) -> Result<(u64, u64), String> {
    let examined: std::fs::Metadata = std::fs::metadata(path).map_err(error_text)?;
    return Ok((examined.dev(), examined.ino()));
}

/// What:
///  A failed look-up as the text a refusal carries.
/// Why:
///   A name that cannot be looked at again is a refusal,
/// and its reason is the operating system's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errorText = (error: Error) => error.message;
/// ```
pub(super) fn error_text(error: std::io::Error) -> String {
    return error.to_string();
}

/// Which file the name `.gitattributes` of `directory` holds,
/// without following a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityBeneath = (directory) => { const found = lstatAt(directory, '.gitattributes'); return [found.dev, found.ino]; };
/// ```
pub(super) fn identity_beneath(
    directory: &std::fs::File,
    _label: &str,
) -> Result<(u64, u64), String> {
    let examined: std::fs::Metadata =
        crate::fs_beneath::entry_metadata_at(directory, OsStr::new(".gitattributes"))
            .map_err(error_text)?;
    return Ok((examined.dev(), examined.ino()));
}
