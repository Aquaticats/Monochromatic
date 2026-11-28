//! What:
//!  The mark a trigger writes when it asked for a pass that no pass has covered:
//!  one root-owned file whose presence is its whole meaning.
//! Why:
//!  Two triggers that arrive together need one pass,
//!  and the pass that would answer the second is either the first pass's own
//!  second half or the next trigger's
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does" and "What the marks guarantee,
//! and what they do not";
//!  veto entries V4 and V6).
//!  The mark is one exclusive creation and one unlink,
//!  so no process waits on it and no lock is taken for it
//!  (rule `NLK`;
//!  the design's "Why this needs no lock").
//!  A mark that cannot be written or removed is reported and never refuses the
//!  pass,
//!  because the device is cost and the per-slot claim is safety (V7's first
//!  option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeTheMark(folder); // creates 'another-pass-is-due'
//! removeTheMark(folder); // unlinks it, and a mark that is not there is fine
//! ```

use crate::worktree_clone_mount_sweep_merge::sweeps_folder::{
    PASS_MARK_MODE,
    PASS_MARK_TEXT,
    SweepsFolder,
};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

/// What:
///  What looking for the mark answered.
/// Why:
///   A mark that is there asks for a pass,
///  a mark that is not there asks for nothing,
///  and a mark that cannot be read is neither:
///  the device reports it and treats it as a request,
///  because a mark a person could not read still says a trigger asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkReading = { kind: 'absent' } | { kind: 'present' } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarkReading {
    /// No trigger asked for a pass.
    Absent,
    /// A trigger asked for a pass that no pass has covered.
    Present,
    /// The mark could not be looked at,
    /// so what it says is unknown.
    Unreadable {
        /// What the call answered,
        /// with the folder and the file it named.
        reason: String,
    },
}

impl MarkReading {
    /// What:
    ///  Whether a trigger asked for a pass.
    /// Why:
    ///   A mark that cannot be read counts as a request,
    ///  so an unreadable device costs one pass rather than a lost trigger,
    ///  which is the same balance V7 strikes for every other record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const asked = reading.kind !== 'absent';
    /// ```
    pub fn asks_for_a_pass(&self) -> bool {
        return !matches!(self, MarkReading::Absent);
    }
}

/// What:
///  Ask for a pass no pass has covered.
/// Why:
///   Presence is the meaning,
///  so the file is created exclusively and a mark that is already there is the
///  answer rather than a failure;
///  the write happens after the creation because a reader that found an empty
///  file must still see that a trigger asked.
///
/// # Errors
/// The folder or the file could not be created,
/// in the words a pass reports before it runs anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeTheMark(folder);
/// ```
pub fn write_the_mark(folder: &SweepsFolder) -> Result<(), String> {
    folder.prepare()?;
    let path: PathBuf = folder.pass_mark_path();
    let opened: Result<std::fs::File, std::io::Error> = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PASS_MARK_MODE)
        .open(path.as_path());
    let mut file: std::fs::File = match opened {
        Ok(file) => file,
        // A mark another trigger wrote is the request this one would have made.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) => {
            return Err(format!(
                "the mark {} could not be written ({error})",
                path.display()
            ));
        }
    };
    // Presence is the meaning, so a body that did not reach the file is reported and the mark stands.
    file.write_all(format!("{PASS_MARK_TEXT}\n").as_bytes())
        .map_err(|error| {
            return format!("the mark {} could not be written ({error})", path.display());
        })?;
    return Ok(());
}

/// What:
///  Look whether a trigger asked for a pass.
/// Why:
///   The look is one `lstat`,
///  so it follows no link and answers about the entry itself;
///  a file,
/// a link or a folder of that name all say a request stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = theMarkIsThere(folder);
/// ```
pub fn the_mark_is_there(folder: &SweepsFolder) -> MarkReading {
    let path: PathBuf = folder.pass_mark_path();
    return match std::fs::symlink_metadata(path.as_path()) {
        Ok(_found) => MarkReading::Present,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => MarkReading::Absent,
        Err(error) => MarkReading::Unreadable {
            reason: format!(
                "the mark {} could not be looked at ({error})",
                path.display()
            ),
        },
    };
}

/// What:
///  Drop the mark,
///  because the pass that is about to run covers what it asked for.
/// Why:
///   A mark that is not there is exactly the state the caller wants,
///  so a missing file is no failure;
///  anything else is reported and the pass runs anyway,
///  and a name that is no file at all cannot be unlinked,
///  which is a report rather than a refusal.
///
/// # Errors
/// The mark is there and could not be removed,
/// in the words a pass reports before it runs anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeTheMark(folder);
/// ```
pub fn remove_the_mark(folder: &SweepsFolder) -> Result<(), String> {
    let path: PathBuf = folder.pass_mark_path();
    return match std::fs::remove_file(path.as_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "the mark {} could not be removed ({error})",
            path.display()
        )),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pass_mark_tests.rs"]
mod tests;
