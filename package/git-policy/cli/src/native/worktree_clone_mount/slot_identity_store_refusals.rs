//! What:
//!  Why the slot identity store could not be read,
//!  written,
//!  or listed,
//!  and the sentence each cause prints.
//! Why:
//!  An absent store folder asks for the setup run,
//!  a file this repository did not write asks for the administrator,
//!  a slot already recorded asks for no refresh at all,
//!  and a record filed under another name asks for the record put back;
//!  one sentence per cause is what keeps those ways forward apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! print(refusal.message()); // names the file, the failing component and every way forward
//! ```

use super::clone_folder_identity::{CloneFolderIdentity, CloneFolderField, field_names};
use crate::private_storage::ReadRefusal;
use std::path::{Path, PathBuf};

/// What:
///  Why the store could not be read,
///  written,
///  or listed.
/// Why:
///   Each cause needs its own sentence:
///  an absent folder asks for the setup run,
///  a file this module did not write asks for the administrator,
///  and a slot already recorded asks for no refresh at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoreRefusal = { cause: 'missing' | 'unreadable' | 'line' | 'too-large' | 'another-name' | 'foreign-entry' | 'no-parent' | 'no-slot' | 'already-recorded' | 'parent-identity' | 'parent-already-recorded' | 'slot-name' | 'not-under-root-control' | 'write'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreRefusal {
    /// Nothing is at the store folder's path.
    Missing {
        /// The path that was read.
        path: PathBuf,
    },
    /// An entry or the folder could not be read as the store.
    Unreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// One line is not a line this module wrote.
    Line {
        /// The file that was read.
        path: PathBuf,
        /// The one-based line number.
        number: usize,
        /// What is wrong with it.
        reason: String,
    },
    /// One record file is larger than one read accepts.
    RecordTooLarge {
        /// The file that was read.
        path: PathBuf,
        /// The size it held.
        bytes: u64,
        /// The largest one record may hold.
        bound: u64,
    },
    /// One record file holds a key the file's own name does not belong to.
    RecordUnderAnotherName {
        /// The file that was read.
        path: PathBuf,
        /// The name the file is filed under.
        filed_under: String,
        /// The key the record carries.
        recorded: String,
    },
    /// One entry of the store folder is neither a record this layout names nor a staging entry.
    ForeignEntry {
        /// The entry that is no record.
        path: PathBuf,
        /// Its name as the folder lists it.
        entry: String,
    },
    /// The store folder holds no `.parent` record.
    NoParentRecord {
        /// The store folder that was read.
        path: PathBuf,
    },
    /// The store folder holds no record for the slot named.
    NoSlotRecord {
        /// The store folder that was read.
        path: PathBuf,
        /// The slot that has no record.
        slot: String,
    },
    /// The slot named already has a record,
    /// which a second write never replaces;
    /// the identity carried is the record that won.
    SlotAlreadyRecorded {
        /// The record file that holds the winner.
        path: PathBuf,
        /// The slot that has a record.
        slot: String,
        /// The identity the winner recorded.
        identity: CloneFolderIdentity,
    },
    /// A slot name is not one plain folder name.
    SlotName {
        /// The name as it was given.
        slot: String,
        /// What makes it no plain name.
        reason: String,
    },
    /// The folder, or one record in it, is not under root control,
    /// so this account could rewrite the identities the route reads.
    NotUnderRootControl {
        /// The path that is not under root control.
        path: PathBuf,
        /// What that path is,
        /// such as `the store folder`.
        what: String,
        /// What lets this account write it.
        reason: String,
    },
    /// The store could not be written,
    /// filled or flushed.
    Write {
        /// The path that was written.
        path: PathBuf,
        /// What the write answered.
        reason: String,
    },
    /// The walked `.claude/worktrees` folder is not the one `.parent` records.
    ParentIdentity {
        /// The store folder that was read.
        path: PathBuf,
        /// The identity `.parent` holds.
        recorded: Box<CloneFolderIdentity>,
        /// The identity the walk read for the live folder.
        found: Box<CloneFolderIdentity>,
        /// Every field that moved.
        fields: Vec<CloneFolderField>,
    },
    /// The setup run found `.parent` holding another identity than the live folder's.
    ParentAlreadyRecorded {
        /// The store folder that was read.
        path: PathBuf,
        /// The identity `.parent` holds.
        recorded: Box<CloneFolderIdentity>,
        /// The identity the walk read for the live folder.
        live: Box<CloneFolderIdentity>,
        /// Every field that moved.
        fields: Vec<CloneFolderField>,
    },
}

impl StoreRefusal {
    /// The refusal in words,
    /// naming the file,
    /// the failing component and every way forward this repository offers.
    pub fn message(&self) -> String {
        return match self {
            StoreRefusal::Missing { path } => format!(
                "the recorded worktree clone slot identities are missing: the store folder {} is not there. Nothing is mounted onto a clone slot until the setup run has made that folder and recorded the `.claude/worktrees` parent identity in it. Ways forward: run the repository's setup as root, which makes the folder mode 0755 and publishes `.parent`, and run the command again; or, when the folder was moved, restore the administrator's copy of it.",
                path.display()
            ),
            StoreRefusal::Unreadable { path, reason } => format!(
                "the recorded worktree clone slot identities at {} could not be read: {reason}. Ways forward: restore that file from the administrator's copy, or run the repository's setup again, and run the command again.",
                path.display()
            ),
            StoreRefusal::Line { path, number, reason } => format!(
                "{} line {number} is not a line this repository wrote: {reason}. Nothing is attached to a slot until the file reads whole. Ways forward: restore that file from the administrator's copy; nobody edits this store by hand.",
                path.display()
            ),
            StoreRefusal::RecordTooLarge { path, bytes, bound } => format!(
                "{} is {bytes} bytes, and one record may hold at most {bound} bytes, so it was refused rather than read into memory. That bound is derived from the kernel's own limits on one extended attribute value and one file name, so no folder this route's capture accepts can make a wider record, and nothing was written. Ways forward: restore that file from the administrator's copy, and report a file wider than the bound together with its size; no command in this repository raises the bound or removes a record, and nobody edits this store by hand.",
                path.display()
            ),
            StoreRefusal::RecordUnderAnotherName { path, filed_under, recorded } => format!(
                "{} is filed under `{filed_under}` and holds the record of `{recorded}`, so nothing says which folder it describes. A record's key is the name it is filed under, and a slot record is only ever filed under its own slot's name. Ways forward: restore that file from the administrator's copy, or remove the wrong file and record the slot again by letting the sweep that first verifies the clone publish it; nobody edits this store by hand.",
                path.display()
            ),
            StoreRefusal::ForeignEntry { path, entry } => format!(
                "{} holds the entry `{}`, which is neither `.parent`, nor a clone slot's name, nor a staging entry this repository makes. A foreign name could be a slot whose record a later read would miss. Ways forward: stop whatever wrote that entry, restore the store folder from the administrator's copy, and report the command that made it; nobody edits this store by hand.",
                path.display(),
                entry.escape_debug()
            ),
            StoreRefusal::NoParentRecord { path } => format!(
                "the store folder {} holds no identity for the `.claude/worktrees` parent folder, so there is nothing to compare that folder against. Ways forward: run the repository's setup, which publishes the parent record, and run the command again.",
                path.display()
            ),
            StoreRefusal::NoSlotRecord { path, slot } => format!(
                "the store folder {} holds no identity for the clone slot `{slot}`, so this slot was never verified. Ways forward: let the sweep that first verifies a clone publish its record, and run the command again; a slot whose clone is gone takes ordinary reflink creation or Git's own checkout instead, without this route.",
                path.display()
            ),
            StoreRefusal::SlotAlreadyRecorded { path, slot, identity } => format!(
                "{} already records an identity for the clone slot `{slot}` (device {}, inode {}, owner {}, mode {:04o}), and a slot identity is never refreshed because a new inode or mode appeared. Ways forward: this is the record that won, so a sweep carries on with it and compares the folder against it; to change the binding, restore the folder whose identity is recorded, or settle it with the removal workflow. Only an administrator's `adopt-worktree-parent` may replace the parent folder's record, and never a slot's.",
                path.display(),
                identity.device,
                identity.inode,
                identity.owner,
                identity.mode & 0o7777
            ),
            StoreRefusal::SlotName { slot, reason } => format!(
                "`{}` is no slot name: {reason}. A slot name is one plain folder name below `.claude/worktrees`, with no slash, no space and no leading dot. Ways forward: name the slot exactly as `git worktree list` shows it below that folder.",
                slot.escape_debug()
            ),
            StoreRefusal::NotUnderRootControl { path, what, reason } => format!(
                "{what} at {} is not under root control: {reason}. Nothing is attached or adopted on an identity this account could rewrite. Ways forward: run the repository's setup as root, which makes the store folder and its records root's own with no group or other write bit; or restore the administrator's copy of the folder and of its permissions.",
                path.display()
            ),
            StoreRefusal::Write { path, reason } => format!(
                "{} could not be written: {reason}. Nothing is attached on a record the store did not keep. Ways forward: check the store folder's free space and permissions, restore the administrator's copy of any file this write reported, and run the command again; nobody edits this store by hand.",
                path.display()
            ),
            StoreRefusal::ParentIdentity { path, recorded, found, fields } => format!(
                "the `.claude/worktrees` folder this walk reached is not the folder {} records: the {} differs, and the record holds device {} and inode {} while the walk reached device {} and inode {}. Nothing was recorded, because a record is never taken for a folder reached through a parent no check compared. Ways forward: let the administrator's `adopt-worktree-parent` decide whether the recreated parent may be recorded, then run this command again.",
                path.display(),
                field_names(fields),
                recorded.device,
                recorded.inode,
                found.device,
                found.inode
            ),
            StoreRefusal::ParentAlreadyRecorded { path, recorded, live, fields } => format!(
                "the store folder {} already records the `.claude/worktrees` parent folder, and the live folder's {} differs from that record (recorded device {} inode {}, live device {} inode {}). Nothing was written, because the setup run publishes the parent record once and only the administrator's `adopt-worktree-parent` replaces it. Ways forward: when the recorded folder is the one this checkout has, run the setup again and it compares equal; when the folder really was recreated, run `adopt-worktree-parent` so an administrator's decision replaces the record.",
                path.display(),
                field_names(fields),
                recorded.device,
                recorded.inode,
                live.device,
                live.inode
            ),
        };
    }
}

/// The refusal for a record or folder the read would not open,
/// naming which of the causes the open answered.
pub(super) fn unreadable(path: &Path, refusal: ReadRefusal, limit: u64) -> StoreRefusal {
    return match refusal {
        ReadRefusal::Missing => StoreRefusal::Missing { path: path.to_path_buf() },
        ReadRefusal::NotRegular => StoreRefusal::Unreadable { path: path.to_path_buf(), reason: String::from("the name holds a link, a folder or another non-regular file") },
        ReadRefusal::TooLarge => StoreRefusal::RecordTooLarge { path: path.to_path_buf(), bytes: limit + 1, bound: limit },
        ReadRefusal::Io(error) => StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() },
    };
}

/// The refusal for a write or a flush the system answered with `error`.
///
/// Why:
///  A refused write is not a read problem:
///  a person who is told the store could not be read looks in the wrong place,
///  so every failure to write,
///  fill or flush the store has its own cause and its own repair.
pub(super) fn write_refusal(path: &Path, error: std::io::Error) -> StoreRefusal {
    return match error.kind() {
        std::io::ErrorKind::NotFound => StoreRefusal::Missing { path: path.to_path_buf() },
        _ => StoreRefusal::Write { path: path.to_path_buf(), reason: error.to_string() },
    };
}
