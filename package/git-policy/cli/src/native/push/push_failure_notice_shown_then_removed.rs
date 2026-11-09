//! What:
//!  One record of a failed backup push as a command holds it before it shows it:
//! the words for the person,
//! the text
//!  they were worded from,
//! and the step that removes the record once those words are out.
//! Why:
//!  Reading a record and removing it are two moments,
//! and only the first is safe while the commit is being named;
//! a record
//!  removed before its words reach the person is gone with nothing shown,
//! and a record the service replaced meanwhile is
//!  another statement about the same commit.
//! The reading stays in `push_failure_notice.rs`;
//! this file holds what the caller does with
//!  one record,
//! so neither file outgrows the line limit alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Notice = { path: string; seen: string; text: string };
//! async function show(notice, out) { out.write(notice.text + '\n'); out.flush(); if (readFile(notice.path) === notice.seen) unlink(notice.path) }
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  One record read from the disk:
/// the words the person is to be shown,
/// and the text they were worded from.
/// Why:
///   The record goes only after its words have been written out,
/// and only while the file still holds the text that was
///  worded,
/// so a command killed before it prints loses nothing,
/// and a record the service replaced meanwhile is not taken for
///  the one this command read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Notice = { path: string; seen: string; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice {
    /// The record's file.
    pub(crate) path: PathBuf,
    /// The record's text as it was read,
    /// which is what may be removed.
    pub(crate) seen: String,
    /// The words for the person.
    pub(crate) text: String,
}

impl Notice {
    /// The words the person is shown.
    pub fn text(&self) -> &str {
        return self.text.as_str();
    }

    /// The record's file,
    /// for a caller that shows the words somewhere else and then removes them.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Shows the words on `out`,
    /// and removes the record once they are out.
    /// Why:
    ///   A record removed before its words reach the person is gone with nothing shown,
    /// and the next command cannot
    ///  find it;
    /// when the write fails the record stays for that command.
    /// The removal names what was read,
    /// so a record written
    ///  meanwhile at the same name is kept for the person.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// out.write(text + '\n'); out.flush(); if (readFile(path) === seen) unlink(path);
    /// ```
    pub fn show(&self, out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out, "{}", self.text)?;
        out.flush()?;
        self.remove_what_was_read();
        return Ok(());
    }

    /// What:
    ///  Removes the record once its words are out,
    /// if the file still holds the text this notice was worded from.
    /// Why:
    ///   A command that pauses while it words a notice can outlive the service's replacement of that same record,
    /// a terminal
    ///  failure after a retryable one,
    /// and removing the file then would delete words nobody has read.
    /// A removal that fails
    ///  leaves the record to be shown again,
    /// which costs a repeat and loses nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (readFile(path) === seen) unlink(path);
    /// ```
    pub fn remove_what_was_read(&self) {
        let Ok(now) = std::fs::read_to_string(self.path.as_path()) else {
            return;
        };
        if now != self.seen {
            return;
        }
        let _removed = std::fs::remove_file(self.path.as_path());
    }
}
