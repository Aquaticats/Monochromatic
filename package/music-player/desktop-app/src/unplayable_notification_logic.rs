//! The pure rules of the "cannot be played" report (D122): what a report of undecodable files becomes, and which files
//! have already been told.
//!
//! D83 used to show this report in a floating message inside the window.
//! D122 sends it to the desktop as a system notification instead, and the window keeps only the row mark.
//! Nothing here touches the bus, a thread or a clock, so every rule is tested without them.
//! The thread and the bus calls are in `unplayable_notification` and `unplayable_notification_wire`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// What: `SUMMARY` is the title of every "cannot be played" notification.
/// Why: a fixed, plain title carries no file name or folder, so nothing private sits in the part that a desktop may
/// show on a lock screen or keep in its history; the body names the file.
pub const SUMMARY: &str = "Cannot play a file";

/// What: `Notice` is what one report becomes: a title and a body.
/// Why: the bus layer sends exactly these two texts and holds no wording of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The fixed title, `SUMMARY`.
    pub summary: &'static str,
    /// The message: one file named, or a count with the first file named.
    pub body: String,
}

/// What: `file_name_text` is the last component of a path as text, or the whole path when it has none.
/// Why: a message names the file, not its long folder path.
pub fn file_name_text(path: &Path) -> String {
    return path
        .file_name()
        .map_or_else(|| return path.display().to_string(), |name| return name.to_string_lossy().into_owned());
}

/// What: `unplayable_message` words the report of files that could not be opened for playback.
/// Why: one file is named; several collapse to a count with the first named (D9: a count, not one message per file).
/// An empty report gives an empty message, which the caller does not show.
pub fn unplayable_message(paths: &[PathBuf]) -> String {
    return match paths {
        [] => String::new(),
        [only] => format!("{} can't be played.", file_name_text(only)),
        [first, rest @ ..] => format!("{} files can't be played, including {}.", rest.len() + 1, file_name_text(first)),
    };
}

/// What: `Ledger` remembers which files have been announced in this run.
/// Why: a repeated report of the same files must not post a second notification; the engine already reports each file
/// once, and this keeps that true when the engine forgets (a folder opened again) or the same file fails after it
/// decoded once. The row mark still shows the file every time.
#[derive(Debug, Default)]
pub struct Ledger {
    announced: HashSet<PathBuf>,
}

/// Methods of `Ledger`.
impl Ledger {
    /// What: `new` builds a ledger that has announced nothing.
    /// Why: every run of the app starts without any file told.
    pub fn new() -> Ledger {
        return Ledger::default();
    }

    /// What: `admit` returns the notification for the files of `paths` not announced before, or `None` when there is
    /// nothing new to say.
    /// Why: files in one report give one notification (D9), an empty or fully repeated report gives none, and a file
    /// named twice in one report is counted once.
    pub fn admit(&mut self, paths: &[PathBuf]) -> Option<Notice> {
        let fresh: Vec<PathBuf> = paths.iter().filter(|path| return self.announced.insert((*path).clone())).cloned().collect();
        if fresh.is_empty() {
            tracing::debug!(files = paths.len(), "no unplayable file is new; no notification is made");
            return None;
        }
        tracing::info!(files = fresh.len(), "unplayable files are announced as one notification");
        return Some(Notice { summary: SUMMARY, body: unplayable_message(&fresh) });
    }
}

#[cfg(test)]
#[path = "unplayable_notification_logic_tests.rs"]
mod tests;
