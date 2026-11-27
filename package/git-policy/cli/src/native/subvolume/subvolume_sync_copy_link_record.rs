//! What:
//!  The bookkeeping of a file with several names in a tree pass:
//! whether a destination file is one of the copies already recorded for a source file,
//! recording a copy,
//! and trying to make the next copy a hard link to an earlier one.
//! Why:
//!  A source file with several names is copied once per name,
//! and a destination that cannot link across a boundary (two subvolumes) holds separate copies;
//! the pass keeps the copies it made for each source (device,
//! inode) so the next name is a hard link where the platform allows it,
//! and so an equal file is only taken for unchanged when it is one of them.
//! The copying of one entry is `subvolume_sync_copy_one_entry.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pass.copyFile(..); // records the copy; the next name links to it when it can
//! ```

use crate::subvolume_anchor::Anchor;
use crate::subvolume_error::MigrateError;
use crate::subvolume_sync_pass_state_and_checks::*;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

impl Pass<'_> {
    /// Whether the destination file is one of the recorded copies of this source file,
    /// or none is recorded yet.
    pub(crate) fn is_linked_as_recorded(
        &self,
        key: (u64, u64),
        found: &Metadata,
    ) -> bool {
        let Some(copies) = self.links.get(&key) else {
            return true;
        };
        for copy in copies {
            if let Ok(copy_meta) = std::fs::symlink_metadata(self.destination.join(copy))
                && copy_meta.ino() == found.ino()
            {
                return true;
            }
        }
        return false;
    }

    /// Record a destination file as a copy of a source file with several names.
    pub(crate) fn record_link(
        &mut self,
        key: (u64, u64),
        relative: &Path,
    ) {
        let copies: &mut Vec<PathBuf> = self.links.entry(key).or_default();
        let mut recorded: bool = false;
        for copy in copies.iter() {
            if copy == relative {
                recorded = true;
                break;
            }
        }
        if !recorded {
            copies.push(relative.to_path_buf());
        }
    }

    /// Try to make the destination another name of an earlier copy.
    /// True when it was done.
    pub(crate) fn link_to_earlier_copy(
        &mut self,
        directory: &Directory,
        relative: &Path,
        key: (u64, u64),
        existing: Option<&Metadata>,
    ) -> Result<bool, MigrateError> {
        for copy in self.links.get(&key).cloned().unwrap_or_default() {
            let Ok(earlier) =
                Anchor::open_below(self.destination, copy.parent().unwrap_or(Path::new("")))
            else {
                continue;
            };
            let first: PathBuf = earlier.child(copy.file_name().unwrap_or_default());
            remove_if_present(
                directory.partial().as_path(),
                self.shown(relative).as_path(),
            )?;
            if self
                .platform
                .hard_link(first.as_path(), directory.partial().as_path())
                .is_err()
            {
                continue;
            }
            self.publish(directory, relative, existing)?;
            self.stats.hard_linked += 1;
            return Ok(true);
        }
        return Ok(false);
    }
}
