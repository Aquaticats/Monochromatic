//! What:
//!  The dispatch of one source child by what it is:
//! a directory (created or replaced,
//! then visited),
//! a symbolic link,
//! a named pipe,
//! a regular file,
//! or an entry of another kind,
//! which is listed and left alone.
//! Why:
//!  The walk of one directory reads a source listing once,
//! and this step turns each entry of it into the copy or the visit that entry needs;
//! the copying itself is `subvolume_sync_copy_one_entry.rs`
//! and the walk that calls this is `subvolume_sync_directory_visit.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pass.handleChild(directory, relative, metadata, stack);
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_sync_pass_state_and_checks::*;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::Metadata;
use std::os::unix::fs::{
    DirBuilderExt,
    FileTypeExt,
    MetadataExt,
};
use std::path::PathBuf;

impl Pass<'_> {
    pub(crate) fn handle_child(
        &mut self,
        directory: &Directory,
        relative: PathBuf,
        metadata: Metadata,
        stack: &mut Vec<Work>,
    ) -> Result<(), MigrateError> {
        let name: OsString = relative
            .file_name()
            .map(OsStr::to_os_string)
            .unwrap_or_default();
        let destination: PathBuf = directory.child(name.as_os_str());
        let existing: Option<Metadata> = std::fs::symlink_metadata(&destination).ok();
        let older_than_since: bool = match self.options.created_since {
            Some(since) => metadata.mtime() < since,
            None => false,
        };
        if existing.is_none() && self.options.update_only && older_than_since {
            self.stats.left_out.push(relative);
            return Ok(());
        }
        let file_type: std::fs::FileType = metadata.file_type();
        if file_type.is_dir() {
            if let Some(found) = &existing
                && !found.is_dir()
            {
                if !self.replace_allowed(relative.as_path(), found, &metadata) {
                    return Ok(());
                }
                remove_entry(
                    destination.as_path(),
                    self.shown(relative.as_path()).as_path(),
                )?;
            }
            let is_directory_now: bool = match existing.as_ref() {
                Some(found) => found.is_dir(),
                None => false,
            };
            if !is_directory_now {
                MigrateError::from_io_result(
                    std::fs::DirBuilder::new().mode(0o700).create(&destination),
                    "create the directory",
                    self.shown(relative.as_path()).as_path(),
                )?;
            }
            self.stats.directories += 1;
            stack.push(Work::Finish {
                relative: relative.clone(),
                metadata,
                created: !is_directory_now,
            });
            stack.push(Work::Visit(relative));
            return Ok(());
        }
        if file_type.is_symlink() {
            return self.copy_symlink(directory, &relative, &metadata, existing);
        }
        if file_type.is_fifo() {
            return self.copy_fifo(directory, &relative, &metadata, existing);
        }
        if file_type.is_file() {
            return self.copy_file(directory, &relative, &metadata, existing);
        }
        self.stats.special_skipped.push(relative);
        return Ok(());
    }
}
