//! What:
//! Why an item was not removed,
//! and the words that show it to a person.
//! Why:
//! Every refusal names the path and what was found there,
//! so a person can see which entry changed and what stayed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { changed: { path: string; judged: Identity; found: Identity } } | { gone: string } | { io: { path: string; error: Error } } | { partlyRemoved: { path: string; deleted: Deleted } };
//! function modesNamed(modes: string[]): string
//! ```
/// Imports what a partly finished deletion did.
use super::item_and_deletion_record::Deleted;
/// Imports the identity a refusal reports on both sides of a change.
use crate::lean_identity::Identity;
/// Imports the error type a failed system call is reported with.
use std::io;
/// Imports the owned path type a refusal and a changed directory are named by.
use std::path::PathBuf;
/// How many changed directories a proof line names before it counts the rest.
const NAMED_MODES: usize = 5;

/// Why an item was not removed.
#[derive(Debug)]
pub enum Refusal {
    /// The path now holds another entry,
    /// or the moved entry is not the judged one.
    Changed {
        /// The path.
        path: PathBuf,
        /// What was judged.
        judged: Identity,
        /// What was found.
        found: Identity,
    },
    /// The entry vanished before it was claimed:
    /// another owner removed it.
    Gone(
        /// The path that was already removed.
        PathBuf,
    ),
    /// A system call failed.
    Io {
        /// The path.
        path: PathBuf,
        /// The error.
        error: io::Error,
    },
    /// Part of the entry is gone and part of it stayed:
    /// a directory the walk reached could not be emptied,
    /// so what lies under it is still there.
    PartlyRemoved {
        /// The claimed entry.
        path: PathBuf,
        /// What went and what stayed.
        deleted: Deleted,
    },
}

/// What:
/// `pub fn modes_named(modes: &[PathBuf]) -> String` is the clause a proof line carries when a removal changed the mode of
/// a directory,
/// naming a few of them and counting the rest.
/// Why:
/// Changing a mode is a mutation of the tree before it goes,
/// so the line says it happened,
/// as it says which locks were taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modesNamed = (modes: string[]): string => modes.length === 0 ? '' : `; mode changed on ${modes.length} directories this user could not write: ${modes.slice(0, 5).join(', ')}${modes.length > 5 ? ` and ${modes.length - 5} more` : ''}`;
/// ```
pub fn modes_named(modes: &[PathBuf]) -> String {
    if modes.is_empty() {
        return String::new();
    }
    let mut named: Vec<String> = Vec::with_capacity(NAMED_MODES);
    for mode in modes.iter().take(NAMED_MODES) {
        named.push(mode.display().to_string());
    }
    let rest: String = match modes.len().checked_sub(NAMED_MODES) {
        Some(more) if more > 0 => format!(" and {more} more"),
        _ => String::new(),
    };
    return format!(
        "; mode changed on {} directories this user could not write: {}{rest}",
        modes.len(),
        named.join(", ")
    );
}

/// The text a refusal shows.
impl std::fmt::Display for Refusal {
    /// Writes the refusal as text.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            Refusal::Changed {
                path,
                judged,
                found,
            } => write!(
                formatter,
                "{} is not the entry that was judged (device {} inode {} then, device {} inode {} now); nothing was removed",
                path.display(),
                judged.device,
                judged.inode,
                found.device,
                found.inode
            ),
            Refusal::Gone(path) => write!(
                formatter,
                "{} was already removed by another owner",
                path.display()
            ),
            Refusal::Io {
                path,
                error,
            } => write!(formatter, "{}: {error}", path.display()),
            Refusal::PartlyRemoved {
                path,
                deleted,
            } => write!(
                formatter,
                "{} was only partly removed: {} disk bytes are gone, {} part(s) of it stay{}, and each stayed because: {}",
                path.display(),
                deleted.bytes,
                deleted.remaining.len(),
                modes_named(&deleted.modes_added),
                deleted.remaining.join("; ")
            ),
        };
    }
}
