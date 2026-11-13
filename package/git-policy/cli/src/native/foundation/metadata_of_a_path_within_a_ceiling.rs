//! What:
//!  One path's own metadata, read for no longer than a ceiling.
//! Why:
//!  A metadata lookup on a mount whose server stopped answering waits in the kernel with no end,
//!  and the guard reads metadata to tell a file from a directory and to compare a marked edit with
//!  its index entry;
//!  the owner's rule is "nothing should ever wait forever or indefinitely",
//!  so a lookup that gives no answer inside the ceiling is "no answer",
//!  and the caller refuses the command instead of holding it for as long as the mount takes.
//!  The lookup runs on a thread of its own, because a timer around a call that blocks inside the
//!  kernel cannot end it.
//!  The answer is the lookup's own result, so a path that is not there (`NotFound`) stays
//!  distinguishable from a lookup that failed:
//!  the first means nothing of the person's stands there, and the second says nothing at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = await workWithin(ceiling, () => lstatSync(path, { throwIfNoEntry: false }));
//! // found === { ok: true, metadata } | { ok: false, error } | undefined
//! ```

use std::path::Path;
use std::time::Duration;

/// What:
///  The metadata of the entry at `path`, without following a symbolic link,
///  or nothing when the lookup gave no answer within `ceiling`.
///  `Option<std::io::Result<std::fs::Metadata>>` is
///  "the lookup's own answer, or none".
/// Why:
///  A caller must tell three cases apart:
///  an entry with its metadata,
///  a path that is not there or cannot be read (the lookup's error), and
///  a lookup still waiting at the ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = symlinkMetadataWithin(READ_CEILING, 'a.txt');
/// ```
pub fn symlink_metadata_within(ceiling: Duration, path: &Path) -> Option<std::io::Result<std::fs::Metadata>> {
    let named: std::path::PathBuf = path.to_path_buf();
    return crate::work_awaited_within_a_ceiling::work_within(ceiling, move || {
        // `symlink_metadata` reports the link itself, which is the entry the command writes over.
        return std::fs::symlink_metadata(named.as_path());
    });
}

/// The bounded lookup is controlled beside it.
#[cfg(test)]
#[path = "metadata_of_a_path_within_a_ceiling_tests.rs"]
mod tests;
