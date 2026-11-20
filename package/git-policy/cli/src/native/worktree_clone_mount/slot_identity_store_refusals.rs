//! What:
//!  Why the slot identity store could not be read,
//!  added to,
//!  or rewritten,
//!  and the sentence each cause prints.
//! Why:
//!  An absent store asks for the setup run,
//!  a line this repository did not write asks for the administrator,
//!  and a slot already recorded asks for no refresh at all;
//!  one sentence per cause is what keeps those ways forward apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! print(refusal.message()); // names the file, the failing component and every way forward
//! ```

use crate::private_storage::ReadRefusal;
use std::path::{Path, PathBuf};

/// What:
///  Why the store could not be read,
///  added to,
///  or rewritten.
/// Why:
///   Each cause needs its own sentence:
///  an absent store asks for the setup run,
///  a line this module did not write asks for the administrator,
///  and a slot already recorded asks for no refresh at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoreRefusal = { cause: 'missing' | 'unreadable' | 'line' | 'duplicate' | 'no-parent' | 'no-slot' | 'already-recorded' | 'slot-name' | 'not-under-root-control' | 'write'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreRefusal {
    /// Nothing is at the store's path.
    Missing {
        /// The path that was read.
        path: PathBuf,
    },
    /// The file could not be read as the store.
    Unreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// One line is not a record this module wrote.
    Line {
        /// The path that was read.
        path: PathBuf,
        /// The one-based line number.
        number: usize,
        /// What is wrong with it.
        reason: String,
    },
    /// One key is recorded twice.
    DuplicateKey {
        /// The path that was read.
        path: PathBuf,
        /// The one-based line number of the second record.
        number: usize,
        /// The key that appears twice.
        key: String,
    },
    /// The store holds no record for `.claude/worktrees`.
    NoParentRecord {
        /// The path that was read.
        path: PathBuf,
    },
    /// The store holds no record for the slot named.
    NoSlotRecord {
        /// The path that was read.
        path: PathBuf,
        /// The slot that has no record.
        slot: String,
    },
    /// The slot named already has a record,
    /// which a second write never replaces.
    SlotAlreadyRecorded {
        /// The path that was read.
        path: PathBuf,
        /// The slot that has a record.
        slot: String,
    },
    /// A slot name is not one plain folder name.
    SlotName {
        /// The name as it was given.
        slot: String,
        /// What makes it no plain name.
        reason: String,
    },
    /// The file or the folder that holds it is not under root control,
    /// so this account could rewrite the identities it reads.
    NotUnderRootControl {
        /// The path that is not under root control.
        path: PathBuf,
        /// What that path is,
        /// such as `the store file`.
        what: String,
        /// What lets this account write it.
        reason: String,
    },
    /// The store could not be written,
    /// added to or flushed.
    Write {
        /// The path that was written.
        path: PathBuf,
        /// What the write answered.
        reason: String,
    },
}

impl StoreRefusal {
    /// The refusal in words,
    /// naming the file,
    /// the failing component and every way forward this repository offers.
    pub fn message(&self) -> String {
        return match self {
            StoreRefusal::Missing { path } => format!(
                "the recorded worktree clone slot identities are missing: {} is not there. Nothing is mounted onto a clone slot until the setup run has written that file. Ways forward: run the repository's setup, which writes this file and the `.claude/worktrees` parent identity, and run the command again; or, when the file was moved, restore the administrator's copy of it.",
                path.display()
            ),
            StoreRefusal::Unreadable { path, reason } => format!(
                "the recorded worktree clone slot identities at {} could not be read: {reason}. Ways forward: restore that file from the administrator's copy, or run the repository's setup again, and run the command again.",
                path.display()
            ),
            StoreRefusal::Line { path, number, reason } => format!(
                "{} line {number} is not a record this repository wrote: {reason}. Nothing is attached to a slot until the file reads whole. Ways forward: restore that file from the administrator's copy, or run the repository's setup again; nobody edits this file by hand.",
                path.display()
            ),
            StoreRefusal::DuplicateKey { path, number, key } => format!(
                "{} line {number} records `{key}` a second time. Two identities for one folder decide nothing. Ways forward: restore that file from the administrator's copy, and report which command wrote the second record.",
                path.display()
            ),
            StoreRefusal::NoParentRecord { path } => format!(
                "{} holds no identity for the `.claude/worktrees` parent folder, so there is nothing to compare that folder against. Ways forward: run the repository's setup, whose first sweep records it, and run the command again.",
                path.display()
            ),
            StoreRefusal::NoSlotRecord { path, slot } => format!(
                "{} holds no identity for the clone slot `{slot}`, so this slot was never verified. Ways forward: let the sweep that first verifies a clone record it, and run the command again; a slot whose clone is gone takes ordinary reflink creation or Git's own checkout instead, without this route.",
                path.display()
            ),
            StoreRefusal::SlotAlreadyRecorded { path, slot } => format!(
                "{} already records an identity for the clone slot `{slot}`, and a slot identity is never refreshed because a new inode or mode appeared. Ways forward: restore the folder whose identity is recorded, or settle the binding with the removal workflow; only an administrator's `adopt-worktree-parent` may replace the parent folder's record, and never a slot's.",
                path.display()
            ),
            StoreRefusal::SlotName { slot, reason } => format!(
                "`{}` is no slot name: {reason}. A slot name is one plain folder name below `.claude/worktrees`, with no slash, no space and no leading dot. Ways forward: name the slot exactly as `git worktree list` shows it below that folder.",
                slot.escape_debug()
            ),
            StoreRefusal::NotUnderRootControl { path, what, reason } => format!(
                "{what} at {} is not under root control: {reason}. Nothing is attached or adopted on an identity this account could rewrite. Ways forward: run the repository's setup as root, which writes the file and its folder with root as their owner and no group or other write bit; or restore the administrator's copy of the file and of its folder's permissions.",
                path.display()
            ),
            StoreRefusal::Write { path, reason } => format!(
                "{} could not be written: {reason}. Nothing is attached on a record the store did not keep. Ways forward: when the line was written short, restore that file from the administrator's copy, because it may hold a torn record; then check the folder's free space and permissions and run the command again; nobody edits this file by hand.",
                path.display()
            ),
        };
    }
}

/// The refusal for a store file the read would not open,
/// naming the largest store one read accepts.
pub(super) fn unreadable(path: &Path, refusal: ReadRefusal, limit: u64) -> StoreRefusal {
    return match refusal {
        ReadRefusal::Missing => StoreRefusal::Missing { path: path.to_path_buf() },
        ReadRefusal::NotRegular => StoreRefusal::Unreadable { path: path.to_path_buf(), reason: String::from("the name holds a link, a directory or another non-regular file") },
        ReadRefusal::TooLarge => StoreRefusal::Unreadable { path: path.to_path_buf(), reason: format!("the file is larger than the {limit} bytes one store read accepts") },
        ReadRefusal::Io(error) => StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() },
    };
}

/// The refusal for a write or sync that the system answered with `error`.
///
/// Why:
///  A refused append is not a read problem:
///  a person who is told the file could not be read looks in the wrong place,
///  so every failure to write,
///  add to or flush the store has its own cause and its own repair.
pub(super) fn write_refusal(store: &Path, error: std::io::Error) -> StoreRefusal {
    return match error.kind() {
        std::io::ErrorKind::NotFound => StoreRefusal::Missing { path: store.to_path_buf() },
        _ => StoreRefusal::Write { path: store.to_path_buf(), reason: error.to_string() },
    };
}
