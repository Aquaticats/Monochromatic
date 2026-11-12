//! What:
//!  The record one removal writes before it destroys a worktree's files,
//!  the mark that says the kernel call returned,
//!  and the owner a removal that gave up leaves.
//! Why:
//!   A command killed between its kernel call and the removal of the worktree's registration leaves a state only
//! `git worktree prune` could clear,
//!  so the removal writes what it is about to do,
//!  marks it once the files are gone,
//!  and the next command finishes what it could not
//!  (`worktree_removal_interrupted_destroy.rs`).
//!  The record is a file of its own because what a record says is read with the code that writes it,
//!  and the finish that reads it and this writer would pass the crate's line ceiling together.
//!  A record names the process that wrote it,
//!  so a command can tell a removal still running from one a kill left,
//!  and a removal that gave up writes an owner no process can be proven to be
//!  (`ABANDONED_OWNER`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordDestroy(commonDir, id, identity, target);
//! await recordDestroyed(commonDir, id);
//! await abandonDestroy(commonDir, id);
//! forgetDestroy(commonDir, id);
//! ```

use crate::diagnostic_log::warn;
use crate::fs_beneath::ObjectId;
use crate::plain_file::read_plain_file;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::process_identity::current_birth_identity;
use crate::subvolume_file_write_steps::write_beside_then_rename;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalDestroyRecord";

/// The folder of the common Git directory that holds one record per destroy the wrapper started,
/// named by the registration's id.
pub const RECORD_FOLDER: &str = "monochromatic-git-policy-cli/recorded-destroys";

/// The most bytes of a record that is read:
/// three short lines,
/// and a worktree path far under any limit.
pub(super) const RECORD_LIMIT: u64 = 4096;

/// The prefix of the record's first line:
/// the process that wrote it,
/// and the identity that tells it from a later process of the same number.
const OWNER_PREFIX: &str = "owner=";

/// The prefix of the record's identity line.
const IDENTITY_PREFIX: &str = "identity=";

/// The first line of a record whose kernel call returned,
/// which is the only proof that the files of the worktree are gone.
/// It comes first because the rest of the record is a pathname,
/// whose bytes are the person's and may be anything,
/// including these very bytes.
const DESTROYED_MARK: &[u8] = b"destroyed=1\n";

/// The path of the record of one registration.
pub(super) fn record_path(common_dir: &Path, id: &OsStr) -> PathBuf {
    return common_dir.join(RECORD_FOLDER).join(id);
}

/// What:
///  Record that this command is about to destroy the files of the worktree whose registration is `id`,
///  and later that it did,
///  so that a later command finishes what this one does not.
/// Why:
///   The record is written before the kernel call,
/// so a kill before the call leaves the worktree where it was,
/// and it is
///  marked once the call returned,
/// so only that mark proves the files are gone.
/// Without the distinction an absent
///  pathname would be read as destruction,
/// which it is not:
/// a person can move an intact worktree,
/// and emptying its
///  registration leaves its `.git` file pointing at nothing.
/// A record that cannot be written costs the automatic finish and
///  nothing else,
/// so the removal goes on and says so in the diagnostic log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordDestroy(commonDir, id, identity, target);
/// ```
pub fn record_destroy(common_dir: &Path, id: &OsStr, admin: &ObjectId, target: &Path) -> Result<(), String> {
    return record_destroy_with_owner(common_dir, id, admin, target, None);
}

/// What:
///  `record_destroy`,
/// with the owner the record names instead of this process,
/// so a control can write the record of a removal that
///  has ended.
///  `None` writes this process.
/// Why:
///   Recovery leaves a record alone while its owner may still be running,
/// and a control that simulates a killed removal has to
///  write one whose owner is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordDestroyWithOwner(commonDir, id, identity, target, '1:ended');
/// ```
pub fn record_destroy_with_owner(common_dir: &Path, id: &OsStr, admin: &ObjectId, target: &Path, owner: Option<&str>) -> Result<(), String> {
    let path: PathBuf = record_path(common_dir, id);
    if let Some(parent) = path.parent()
        && let Err(error) = create_private_directory_all(parent)
    {
        return Err(format!("{} cannot be made ({error})", parent.display()));
    }
    // The owner first, so another command can tell a removal that is still running from an abandoned one, then the identity
    // the registration was proven with, then the target's own bytes, which need not be text.
    let written_owner: String = match owner {
        Some(spelled) => format!("{OWNER_PREFIX}{spelled}\n"),
        None => match current_birth_identity() {
            Ok(identity) => format!("{OWNER_PREFIX}{}:{identity}\n", std::process::id()),
            // A record without an owner is read as one an earlier build wrote, which is what the recovery then assumes.
            Err(_unknown) => String::new(),
        },
    };
    let mut bytes: Vec<u8> = written_owner.into_bytes();
    bytes.extend_from_slice(format!("{IDENTITY_PREFIX}{}\n", admin.encode()).as_bytes());
    bytes.extend_from_slice(target.as_os_str().as_bytes());
    return match write_private_file_unsynced(path.as_path(), bytes.as_slice()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("{} cannot be written ({error})", path.display())),
    };
}

/// What:
///  Mark the record of `id` as one whose kernel call returned,
///  which is the proof that its files are gone.
/// Why:
///   A record written before the call says only that a destroy was intended.
/// The mark is written after the call,
///  over
/// the same name,
/// always as a whole file,
/// so a later command has either the intent or the proof and never a torn
///  record.
/// A mark that cannot be written costs the automatic finish and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordDestroyed(commonDir, id);
/// ```
pub fn record_destroyed(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    let bytes: Vec<u8> = match read_plain_file(path.as_path(), RECORD_LIMIT) {
        Ok(read) => read,
        Err(error) => {
            warn(TAG, format!("{} cannot be read, so the finished destroy of {id:?} is not recorded ({error})", path.display()).as_str());
            return;
        }
    };
    let read: DestroyRecord = match read_record(bytes.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("{} cannot be read, so the finished destroy of {id:?} is not recorded ({reason})", path.display()).as_str());
            return;
        }
    };
    // The record is rewritten whole with the mark first, through the replacing write: a sibling created exclusively under a
    // name of its own, then one rename, so a kill leaves the intent or the proof and nothing is ever written through a link.
    let mut marked: Vec<u8> = DESTROYED_MARK.to_vec();
    if let Some(owner) = read.owner.as_ref() {
        marked.extend_from_slice(format!("{OWNER_PREFIX}{owner}\n").as_bytes());
    }
    marked.extend_from_slice(format!("{IDENTITY_PREFIX}{}\n", read.admin.encode()).as_bytes());
    marked.extend_from_slice(read.target.as_os_str().as_bytes());
    if let Err(error) = write_beside_then_rename(path.as_path(), marked.as_slice()) {
        warn(TAG, format!("{} cannot be marked ({error}), so a later command cannot finish the removal of {id:?}", path.display()).as_str());
    }
}

/// The owner a removal that gave up on its own registration writes,
/// which no process can be proven to be.
/// Why:
///   A command that meets a record still owned by a live process leaves it to that process
///  (a coordinating note of 2026-11-12:
///  a removal still emptying its registration was reported as one that "cannot be finished").
///  A removal that tried and could not empty its registration is not working on it any more,
///  so it writes this owner before it returns and the next command may finish the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ABANDONED_OWNER = '0:abandoned';
/// ```
pub const ABANDONED_OWNER: &str = "0:abandoned";

/// What:
///  Mark the record of `id` as one whose owner has given up,
///  so the next command may finish it.
/// Why:
///  See `ABANDONED_OWNER`.
///  The record keeps its mark,
///  its identity and its target:
///  only the owner changes,
///  and it changes in one whole write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await abandonDestroy(commonDir, id);
/// ```
pub fn abandon_destroy(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    let bytes: Vec<u8> = match read_plain_file(path.as_path(), RECORD_LIMIT) {
        Ok(read) => read,
        Err(_gone) => return,
    };
    let read: DestroyRecord = match read_record(bytes.as_slice()) {
        Ok(found) => found,
        Err(_unreadable) => return,
    };
    let mut written: Vec<u8> = Vec::new();
    if bytes.starts_with(DESTROYED_MARK) {
        written.extend_from_slice(DESTROYED_MARK);
    }
    written.extend_from_slice(format!("{OWNER_PREFIX}{ABANDONED_OWNER}\n").as_bytes());
    written.extend_from_slice(format!("{IDENTITY_PREFIX}{}\n", read.admin.encode()).as_bytes());
    written.extend_from_slice(read.target.as_os_str().as_bytes());
    if let Err(error) = write_beside_then_rename(path.as_path(), written.as_slice()) {
        warn(TAG, format!("{} cannot be marked as given up ({error}), so a later command leaves it to this one", path.display()).as_str());
    }
}

/// What:
///  Drop the record of one registration,
/// which had nothing left to finish.
/// Why:
///   The caller drops it only after the registration is gone or the destroy is known not to have happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forgetDestroy(commonDir, id);
/// ```
pub fn forget_destroy(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            warn(TAG, format!("the record {} cannot be removed ({error}); a later command tries the same thing again", path.display()).as_str());
            return;
        }
    }
    // The folder goes when the last record does; one that still holds another record stays, which its own failure says.
    let folder: PathBuf = common_dir.join(RECORD_FOLDER);
    let _kept = std::fs::remove_dir(folder.as_path());
}

/// What a record holds:
/// the identity of the registration it was written for and the worktree it names.
/// Why:
///   The identity says whether the registration is still the one the destroy was proven against,
/// and the target says
///  whether the destroy happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestroyRecord = { admin: ObjectId; target: PathBuf };
/// ```
pub(super) struct DestroyRecord {
    /// The process that wrote the record,
    /// and the identity that tells it from a later process with the same number;
    /// nothing for a record an earlier build wrote.
    pub(super) owner: Option<String>,
    /// The identity of the registration when the destroy was proven.
    pub(super) admin: ObjectId,
    /// The worktree directory the destroy was aimed at.
    pub(super) target: PathBuf,
    /// Whether the kernel call returned,
    /// which is the only proof that the files are gone.
    pub(super) destroyed: bool,
}

/// Whether one byte of a record is the newline that ends its identity line.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// The fields of a record,
/// or the reason it cannot be used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestroyRecord = { admin: ObjectId; target: string; destroyed: boolean };
/// ```
pub(super) fn read_record(bytes: &[u8]) -> Result<DestroyRecord, String> {
    // The owner and the mark come first, one line each, so nothing a pathname holds can be taken for them; a record of an
    // earlier build has neither, or the mark alone at the front, and is read the same way. The mark that build wrote last is
    // not read: such a record counts as one whose kernel call is not proven, and recovery then leaves the registration alone
    // and says so.
    let mut rest: &[u8] = bytes;
    let mut owner: Option<String> = None;
    let mut destroyed: bool = false;
    loop {
        let Some(split_at) = rest.iter().position(is_newline) else { break };
        let (line, after): (&[u8], &[u8]) = rest.split_at(split_at);
        if line == DESTROYED_MARK.strip_suffix(b"\n").unwrap_or(DESTROYED_MARK) {
            destroyed = true;
            rest = after.strip_prefix(b"\n").unwrap_or(after);
            continue;
        }
        let Ok(text) = std::str::from_utf8(line) else { break };
        match text.strip_prefix(OWNER_PREFIX) {
            Some(owner_text) => {
                owner = Some(owner_text.to_string());
                rest = after.strip_prefix(b"\n").unwrap_or(after);
            }
            None => break,
        }
    }
    let Some(split_at) = rest.iter().position(is_newline) else {
        return Err(String::from("it has no identity line"));
    };
    let (identity_bytes, after_identity): (&[u8], &[u8]) = rest.split_at(split_at);
    let Ok(identity_text) = std::str::from_utf8(identity_bytes) else {
        return Err(String::from("its identity line is not text"));
    };
    // A record of this build spells the field; an earlier one wrote the identity alone.
    let spelled_identity: &str = identity_text.strip_prefix(IDENTITY_PREFIX).unwrap_or(identity_text);
    let Some(admin) = ObjectId::decode(spelled_identity) else {
        return Err(format!("its identity line {spelled_identity} cannot be read"));
    };
    // Everything after the identity's newline is the target, byte for byte.
    let target: &[u8] = after_identity.strip_prefix(b"\n").unwrap_or(after_identity);
    return Ok(DestroyRecord { owner, admin, target: PathBuf::from(OsStr::from_bytes(target)), destroyed });
}
