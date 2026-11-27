//! What:
//!  The reading of one attributes file:
//! an absolute path Git configured,
//! the `.gitattributes` of a held directory,
//! and the parse of an open file under the room the caller's bound leaves.
//! Why:
//!   Git rewrites a file's bytes on the way into the index when an attribute converts it,
//! so the rules must be read the way Git reads them and proven to be the bytes that were named,
//! and every byte read is charged to the caller's bound when it set one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function readOutside(path, label) { const file = open(path); /* compare the name before and after the read */ }
//! ```

use crate::fs_beneath::open_file_at;
use crate::worktree_attributes::parse_attribute_lines;
use crate::worktree_clean_content::ContentJudge;
use crate::worktree_clean_content::{
    ATTRIBUTES_LIMIT,
    CONTENT_LIMIT,
};
use crate::worktree_clean_content_attribute_file_identity::{
    identity_at_path,
    identity_beneath,
    identity_of,
};
use crate::worktree_clean_content_attribute_sources::AttributeFile;
use crate::worktree_path_failure_message::subject_failure;
use std::ffi::OsStr;
use std::os::unix::fs::{
    MetadataExt,
    OpenOptionsExt,
};
use std::path::Path;

impl<'a> ContentJudge<'a> {
    /// Read an attributes file at an absolute path the way Git does,
    /// following a link;
    /// absent is `None`.
    /// Why the name is looked at twice:
    ///   a configured path is usually a link (`core.attributesFile` or the global default),
    /// and retargeting it while the file is read leaves the descriptor's own metadata unchanged,
    /// so the name
    ///  must still hold the file that was read or the rules are stale
    /// (the fifth review's second finding).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readOutside(path) { const file = open(path); const described = fstat(file); const named = stat(path); if (!sameInode(named, described)) return refuse;
    ///   const parsed = parseOpen(file); const after = stat(path); if (!sameInode(after, described)) return refuse; return parsed; }
    /// ```
    pub(super) fn read_outside(
        &self,
        path: &Path,
        label: String,
    ) -> Result<Option<AttributeFile>, String> {
        // Without blocking, so a named pipe is found out by its type instead of waiting for a writer.
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path)
        {
            Ok(opened) => opened,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(None);
            }
            Err(error) => {
                return Err(format!(
                    "the attributes file {} cannot be read ({error})",
                    path.display()
                ));
            }
        };
        let described: (u64, u64) = subject_failure(
            identity_of(file.metadata(), &label),
            &label,
            "cannot be examined",
        )?;
        let named: (u64, u64) =
            subject_failure(identity_at_path(path), &label, "cannot be looked at")?;
        if named != described {
            return Err(format!(
                "{label} names another file than the one opened, so the attributes it sets cannot be ruled out"
            ));
        }
        let parsed: Option<AttributeFile> = self.parse_open(file, label.clone(), Vec::new())?;
        let after: (u64, u64) =
            subject_failure(identity_at_path(path), &label, "cannot be looked at again")?;
        if after != described {
            return Err(format!(
                "{label} was replaced while it was read, so the attributes it sets cannot be ruled out"
            ));
        }
        return Ok(parsed);
    }

    /// Read the `.gitattributes` of the held directory `directory`;
    /// absent is `None`,
    /// a link or anything but a file is a refusal.
    pub(super) fn read_in(
        &self,
        directory: &std::fs::File,
        label: String,
        base: Vec<u8>,
    ) -> Result<Option<AttributeFile>, String> {
        let file = match open_file_at(directory, OsStr::new(".gitattributes")) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "{label} cannot be read without following a link ({error}), so the attributes it sets cannot be ruled out"
                ));
            }
        };
        let described: (u64, u64) = subject_failure(
            identity_of(file.metadata(), &label),
            &label,
            "cannot be examined",
        )?;
        let named: (u64, u64) = subject_failure(
            identity_beneath(directory, &label),
            &label,
            "cannot be looked at",
        )?;
        if named != described {
            return Err(format!(
                "{label} names another file than the one opened, so the attributes it sets cannot be ruled out"
            ));
        }
        let parsed: Option<AttributeFile> = self.parse_open(file, label.clone(), base)?;
        let after: (u64, u64) = subject_failure(
            identity_beneath(directory, &label),
            &label,
            "cannot be looked at again",
        )?;
        if after != described {
            return Err(format!(
                "{label} was replaced while it was read, so the attributes it sets cannot be ruled out"
            ));
        }
        return Ok(parsed);
    }

    /// Parse an attributes file that is open,
    /// refusing one that is not a plain file or is larger than the limit.
    /// What:
    ///  The bytes of an attributes file this reader still has room for,
    /// with what was read charged to the caller's bound.
    /// Why:
    ///   An attributes file is read under the same lock as the files it describes,
    /// so its bytes are part of what the caller bounds;
    /// a file that does not fit is refused,
    /// which costs a refresh and never a failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function attributeRoom(metadata) { const room = budget ? CONTENT_LIMIT - budget.bytes : ATTRIBUTES_LIMIT; return Math.min(room, ATTRIBUTES_LIMIT); }
    /// ```
    pub(super) fn attribute_room(&self) -> u64 {
        return match self.budget {
            Some(spent) => CONTENT_LIMIT
                .saturating_sub(spent.get())
                .min(ATTRIBUTES_LIMIT),
            None => ATTRIBUTES_LIMIT,
        };
    }

    /// Parse an attributes file that is open,
    /// refusing one that is not a plain file or is larger than the room it has,
    /// and charging the bytes it took.
    pub(super) fn parse_open(
        &self,
        file: std::fs::File,
        label: String,
        base: Vec<u8>,
    ) -> Result<Option<AttributeFile>, String> {
        let metadata: std::fs::Metadata =
            subject_failure(file.metadata(), &label, "cannot be examined")?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "{label} is not a plain file (a named pipe, a device or a directory), so the attributes it sets cannot be ruled out"
            ));
        }
        // What:
        //  A caller that bounds its reads is recording stat data for a file it proves unchanged,
        //  so the attributes must be proven current.
        // Why:
        //   Two writes inside one timestamp tick carry the same change time,
        // so a file changed within the granularity of the clock may have
        //  been rewritten under this read without the state showing it;
        // `stamp_is_settled` names that hazard for the whole crate
        // (`worktree_copy_stage_leaf_copying.rs`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (bounded && !stampIsSettled([metadata.ctime, metadata.ctimeNsec])) return undefined;
        // ```
        if self.budget.is_some()
            && !crate::worktree_copy_stage_leaf_copying::stamp_is_settled((
                metadata.ctime(),
                metadata.ctime_nsec(),
            ))
        {
            return Err(format!(
                "{label} changed too recently for its attributes to be proven current, so the conversion it sets cannot be ruled out"
            ));
        }
        let room: u64 = self.attribute_room();
        if metadata.len() > room {
            return Err(format!(
                "{label} is larger than the {room} bytes this caller may still read, so the attributes it sets cannot be ruled out"
            ));
        }
        let Some(bytes) = self.attribute_bytes(&file, &metadata, room, &label)? else {
            return Err(format!(
                "{label} changed while it was read, so the attributes it sets cannot be ruled out"
            ));
        };
        return Ok(Some(AttributeFile {
            label,
            base,
            lines: parse_attribute_lines(bytes.as_slice()),
        }));
    }
}
