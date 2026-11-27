//! What:
//!  Finding the worktree file a conflict-side command is about to replace:
//!  the held directory that contains it,
//!  the entry's own name,
//!  and why a name in front of it can never be saved.
//! Why:
//!  Nothing can be saved that was not found,
//! and a path string is not a finding:
//! every call that took the path again would follow whatever stands there now,
//! so the walk here never follows a link,
//! a link or a file in front of the name is a failure that names it,
//! and the entry is held open on the descriptor that was checked.
//! `conflict_side_source.rs` copies what this file finds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const located = locate(root, 'dir/a.rs'); // undefined when nothing is there to save; throws below a link
//! ```

/// The debug log that says what a save refused.
use crate::diagnostic_log::debug;
/// The reads below an open directory,
/// each addressed through the descriptor it was given.
use crate::fs_beneath::{
    entry_metadata_at,
    open_directory_at,
};
/// `OsStr` is a borrowed file name in the system's own bytes.
use std::ffi::{
    OsStr,
    OsString,
};
/// `File` is the held directory and the held entry.
use std::fs::File;
/// `OsStrExt` reads a name's raw bytes.
use std::os::unix::ffi::OsStrExt;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// Module tag for diagnostics.
pub(crate) const TAG: &str = "conflictSideSource";
/// Which advice goes with a failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureKind {
    /// The system refused a read,
    /// a write or a place:
    /// free the space or fix the permissions.
    Io,
    /// A link or a file stands where a directory in front of the name should be,
    /// and Git would replace it unsaved.
    Ancestor,
    /// The file changed while it was copied,
    /// so the copy could be stale or cut short.
    Changed,
}

/// Why a copy could not be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveFailure {
    /// The repository-relative path being copied,
    /// the directory in front of it that is in the way,
    /// or the place the copies go
    ///  when none was reached.
    pub subject: String,
    /// The operating system's words,
    /// or what the subject is.
    pub reason: String,
    /// Which advice fits.
    pub kind: FailureKind,
}

impl SaveFailure {
    /// A failure of the system on `subject`,
    /// logged.
    pub fn io(
        subject: String,
        reason: String,
    ) -> SaveFailure {
        debug(TAG, &format!("fail closed: {subject}: {reason}"));
        return SaveFailure {
            subject,
            reason,
            kind: FailureKind::Io,
        };
    }

    /// A failure of kind `kind` on `subject`,
    /// logged.
    fn of_kind(
        kind: FailureKind,
        subject: String,
        reason: String,
    ) -> SaveFailure {
        debug(TAG, &format!("fail closed ({kind:?}): {subject}: {reason}"));
        return SaveFailure {
            subject,
            reason,
            kind,
        };
    }
}

/// A failure of the system on `subject`,
/// logged.
pub(crate) fn io_failure(
    subject: &str,
    error: &std::io::Error,
) -> SaveFailure {
    return SaveFailure::io(String::from(subject), error.to_string());
}

/// The names joined with `/`,
/// as the path they make is written.
fn joined(names: &[OsString]) -> String {
    let mut text: String = String::new();
    for (position, name) in names.iter().enumerate() {
        if position > 0 {
            text.push('/');
        }
        text.push_str(&name.to_string_lossy());
    }
    return text;
}

/// A worktree entry that exists and is to be copied.
/// Its fields are `pub(crate)` because `conflict_side_source.rs` copies what this file found.
#[derive(Debug)]
pub struct Located {
    /// The top of the worktree,
    /// which is trusted:
    /// the reader returns its real path.
    pub(crate) root: PathBuf,
    /// The path below the root,
    /// as typed.
    pub(crate) relative: Vec<u8>,
    /// The held directory that contains the entry.
    pub(crate) parent: File,
    /// The entry's own name.
    pub(crate) name: OsString,
}

/// What:
///  The held directory that contains `relative` below `root`,
/// reached without following a link,
/// or nothing when a directory
///  in front of the name is missing.
/// Why:
///   A path string is resolved again by every call that takes it,
/// and a link in front of the name sends the read somewhere
///  Git does not write.
/// A link or a file in front of the name is a failure that names it,
/// because Git replaces it with a
///  directory and nothing of it would be saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openParent(root: string, relative: Uint8Array): { parent: Dir; name: string } | undefined;
/// ```
pub(crate) fn open_parent(
    root: &Path,
    relative: &[u8],
) -> Result<Option<(File, OsString)>, SaveFailure> {
    let shown: String = String::from_utf8_lossy(relative).into_owned();
    let names: Vec<OsString> =
        match crate::fs_beneath::plain_components(Path::new(OsStr::from_bytes(relative))) {
            Ok(found) => found,
            Err(error) => return Err(io_failure(&shown, &error)),
        };
    let Some((last, ancestors)) = names.split_last() else {
        return Err(SaveFailure::io(shown, String::from("it names no file")));
    };
    // The root is the worktree's real path as the layout reader found it; links among its own ancestors are the person's
    // arrangement and Git's own, not something a name below it can redirect.
    let mut current: File = match File::open(root) {
        Ok(opened) => opened,
        Err(error) => return Err(io_failure(&root.display().to_string(), &error)),
    };
    for (depth, name) in ancestors.iter().enumerate() {
        match open_directory_at(&current, name.as_os_str()) {
            Ok(next) => current = next,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                let subject: String = joined(&names[..=depth]);
                let reason: String = match error.raw_os_error() {
                    Some(libc::ELOOP) => String::from("is a symbolic link"),
                    Some(libc::ENOTDIR) => String::from("is a file, not a directory"),
                    Some(libc::EXDEV) => {
                        // Git writes through a mount; only this wrapper does not read across one.
                        return Err(SaveFailure::io(
                            subject,
                            String::from(
                                "is on another mount, which the wrapper does not read across",
                            ),
                        ));
                    }
                    _ => format!("cannot be opened without following a link ({error})"),
                };
                return Err(SaveFailure::of_kind(FailureKind::Ancestor, subject, reason));
            }
        }
    }
    return Ok(Some((current, last.clone())));
}

/// What:
///  Find the entry at `relative` below `root`,
/// or nothing when no entry is there.
/// Why:
///   A path with no file in the worktree has nothing to copy,
/// and Git then creates it;
/// every other lookup failure
///  stops the save,
/// because Git would still write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(root: string, relative: Uint8Array): Located | undefined;
/// ```
pub fn locate(
    root: &Path,
    relative: &[u8],
) -> Result<Option<Located>, SaveFailure> {
    let Some((parent, name)) = open_parent(root, relative)? else {
        return Ok(None);
    };
    return match entry_metadata_at(&parent, name.as_os_str()) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_failure(&String::from_utf8_lossy(relative), &error)),
        Ok(_) => Ok(Some(Located {
            root: root.to_path_buf(),
            relative: relative.to_vec(),
            parent,
            name,
        })),
    };
}

/// The failure for a source that changed while it was read;
/// `what` is a verb phrase that says how.
pub(crate) fn changed(
    located: &Located,
    what: &str,
) -> SaveFailure {
    return SaveFailure::of_kind(
        FailureKind::Changed,
        String::from_utf8_lossy(located.relative.as_slice()).into_owned(),
        what.to_string(),
    );
}
