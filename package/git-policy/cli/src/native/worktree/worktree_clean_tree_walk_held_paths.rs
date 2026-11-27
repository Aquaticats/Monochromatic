//! What:
//!  The directory chain a walk keeps,
//!  the cursor that reads one tracked file's stat fields below a held root,
//!  and the single read of one path's fields.
//! Why:
//!  The paths of an index come in order,
//!  so most of them share their parent with the one before,
//!  and one `openat2` chain per directory instead of per file is what a walk costs.
//!  Every file is still reached through handles that follow no link,
//!  so a directory a link replaced is refused rather than read through.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_clean_tree_walk_by_content.rs` holds the walks that judge a file by content,
//!  and `worktree_clean_tree_walk.rs` holds the walks that judge by metadata alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cursor = new StatCursor(root); cursor.fields(path); // StatFields | undefined
//! ```

use crate::fs_beneath::{
    entry_metadata_at,
    plain_components,
};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::fields_of;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  The directory `parents` lead to below the walk's root,
/// from the chain that keeps the directories it has already opened.
/// Why:
///   The index's paths come in order,
/// so a chain that keeps the part it shares opens each directory once
///  instead of opening every directory above it again for each entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = chain.directory(parents); // the handle, or the refusal
/// ```
pub(crate) fn directory_of<'chain>(
    chain: &'chain mut DirectoryChain<'_>,
    parents: &[std::ffi::OsString],
) -> Option<&'chain std::fs::File> {
    return chain.directory(parents).ok();
}

/// What:
///  A walk below a held directory that keeps the last directory it opened.
/// Why:
///   The paths of an index come in order,
/// so most of them share their parent with the one before;
/// one `openat2` chain per
///  directory instead of per file is what the walk costs,
/// and each file is still reached through handles that follow no link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cursor = new StatCursor(root); cursor.fields(path); // StatFields | undefined
/// ```
pub struct StatCursor<'a> {
    /// The chain of directories opened below the root,
    /// or the reason the root cannot be examined.
    chain: Result<DirectoryChain<'a>, std::io::Error>,
}

impl<'a> StatCursor<'a> {
    /// A cursor below `root` that has opened nothing.
    pub fn new(root: &'a std::fs::File) -> Self {
        return Self {
            chain: DirectoryChain::new(root),
        };
    }

    /// The metadata of the file `path` names below the root,
    /// or the reason it cannot be found through plain directories.
    fn metadata(
        &mut self,
        path: &[u8],
    ) -> Result<std::fs::Metadata, String> {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(path))) else {
            return Err(format!("{shown} is not a plain path"));
        };
        let Some((name, parents)) = names.split_last() else {
            return Err(format!("{shown} is not a plain path"));
        };
        let Ok(chain) = &mut self.chain else {
            return Err(format!(
                "{shown} is missing from the worktree: the worktree root cannot be examined"
            ));
        };
        let Some(directory) = directory_of(chain, parents) else {
            return Err(format!(
                "{shown} is missing from the worktree or reached through a link"
            ));
        };
        return match entry_metadata_at(directory, name.as_os_str()) {
            Ok(found) => Ok(found),
            Err(_missing) => Err(format!("{shown} is missing from the worktree")),
        };
    }

    /// The stat fields of the tracked file `path` below the root,
    /// or nothing when it cannot be found through plain directories.
    pub fn fields(
        &mut self,
        path: &[u8],
    ) -> Option<StatFields> {
        return match self.metadata(path) {
            Ok(found) => Some(fields_of(&found)),
            Err(_refused) => None,
        };
    }
}

/// What:
///  The stat fields of the tracked file `path` below the held directory `root`,
/// found through handles,
/// or nothing.
/// Why:
///   The index of a snapshot takes the stat fields of the snapshot's own files;
/// reading them by a path with several
///  components would follow a link substituted below the root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFieldsBeneath(root: Dir, path: Uint8Array): StatFields | undefined;
/// ```
pub fn stat_fields_beneath(
    root: &std::fs::File,
    path: &[u8],
) -> Option<StatFields> {
    return StatCursor::new(root).fields(path);
}
