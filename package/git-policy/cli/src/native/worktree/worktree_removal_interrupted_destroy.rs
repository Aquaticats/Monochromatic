//! What:
//!  Finish the removal of a worktree whose files a destroy already took and whose registration it did not,
//!  so that no removal is left half done by a kill or a failed step.
//! Why:
//!  The served removal destroys the subvolume first and removes the registration after it,
//! and the registration is emptied and
//!  removed through its own held descriptor and proof,
//! which can fail where Git's unlinks would not.
//! A command killed in that window,
//!  or one whose step failed,
//! leaves a worktree Git reports as missing but registered,
//! and `git worktree add` at that
//!  path then refuses until someone runs `git worktree prune`.
//! The destroy is therefore recorded before the kernel call,
//! in a
//!  folder of the common Git directory,
//! and every wrapped command finishes what such a record names:
//! the registration is removed
//!  once its records are copied into the removal snapshot the rescue route settles,
//! so an unshown push failure in it is
//!  shown rather than destroyed with it.
//! A record whose worktree is still there is dropped:
//! the destroy did not happen,
//!  so nothing is removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordDestroy(commonDir, id, identity, target);       // before the ioctl
//! await forgetDestroy(commonDir, id);                          // once the registration is gone
//! const problems = await finishInterruptedDestroys(commonDir); // on the next command
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::ObjectId;
use crate::path_bytes_in_records::{hex_of_path, path_of_hex};
use crate::plain_file::read_plain_file;
use crate::private_storage::create_private_directory_all;
use crate::process_identity::current_birth_identity;
use crate::subvolume_file_write_steps::write_beside_then_rename;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  The function that finishes an interrupted removal,
///  where every command of the repository reaches it.
/// Why:
///   What decides an unfinished removal lives in a file of its own
/// (`worktree_removal_finishes_an_interrupted_destroy.rs`),
/// and every caller names it by this module,
/// so the subject moved and the name did not.
pub use crate::worktree_removal_finishes_an_interrupted_destroy::finish_interrupted_destroys;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalInterruptedDestroy";

/// The folder of the common Git directory that holds one record per destroy the wrapper started,
/// named by the registration's id.
pub(crate) const RECORD_FOLDER: &str = "monochromatic-git-policy-cli/recorded-destroys";

/// The most bytes of a record that is read:
/// four short lines,
/// the hexadecimal form of the place a renamed subvolume stands (twice the length of a path),
///  and the worktree's path.
pub(crate) const RECORD_LIMIT: u64 = 16384;

/// The prefix of the record's first line:
/// the process that wrote it,
/// and the identity that tells it from a later process of the same number.
const OWNER_PREFIX: &str = "owner=";

/// The prefix of the record's identity line.
const IDENTITY_PREFIX: &str = "identity=";

/// The prefix of the record's line that names where this removal's subvolume stands once it is renamed,
/// in the hexadecimal form a path of any bytes takes in a line of a record.
const LEAVING_PREFIX: &str = "leavingHex=";

/// The first line of a record whose kernel call returned,
/// which is the only proof that the files of the worktree are gone.
/// It comes first because the rest of the record is a pathname,
/// whose bytes are the person's and may be anything,
/// including these very bytes.
const DESTROYED_MARK: &[u8] = b"destroyed=1\n";

/// The path of the record of one registration.
pub(crate) fn record_path(common_dir: &Path, id: &OsStr) -> PathBuf {
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
    // The record is written whole and replaces whatever stands at its name. A worktree name is used again and again, so a
    // record of an earlier removal of the same registration can be there, marked as destroyed; keeping it would tell the
    // next command that this worktree's files are gone before this removal has destroyed anything, and a sweep could drop
    // the record of a removal that is still running (reported twice on 2026-11-11, 22:30:57 and 22:37:25 -05:00).
    return match write_beside_then_rename(path.as_path(), bytes.as_slice()) {
        Ok(()) => Ok(()),
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
        // A record that is gone needs no mark: another command dropped it, and this removal is about to finish the
        // registration itself, so nothing is left for a later command to finish.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            debug(TAG, format!("the record {} is gone, so there is nothing to mark and nothing a later command must finish", path.display()).as_str());
            return;
        }
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
    if let Some(leaving) = read.leaving.as_ref() {
        marked.extend_from_slice(format!("{LEAVING_PREFIX}{}\n", hex_of_path(leaving.as_path())).as_bytes());
    }
    marked.extend_from_slice(format!("{IDENTITY_PREFIX}{}\n", read.admin.encode()).as_bytes());
    marked.extend_from_slice(read.target.as_os_str().as_bytes());
    if let Err(error) = write_beside_then_rename(path.as_path(), marked.as_slice()) {
        // The folder goes with the last record another command dropped, which is nothing this removal needs: the
        // registration it is emptying is finished by this command, and only a kill before it ends would need the record.
        if error.kind() == std::io::ErrorKind::NotFound {
            debug(TAG, format!("the record {} is gone, so there is nothing to mark and nothing a later command must finish", path.display()).as_str());
            return;
        }
        warn(TAG, format!("{} cannot be marked ({error}), so a command killed before this removal ends would leave its registration to `git worktree prune`", path.display()).as_str());
    }
}

/// What:
///  Record where the subvolume of the removal whose registration is `id` stands once it is renamed,
///  before the rename is made.
/// Why:
///   A removal hands its subvolume to the service by renaming it out of the worktree,
/// and the record's proof
///  that the files left the worktree's path was written only after that rename:
/// a command killed in between
///  left a record that said only that a destroy was intended,
/// with nothing at the worktree's path,
/// and the
///  next command could neither finish the registration nor prove the files had gone,
/// so it reported the state
///  once and left a registration the person had to prune (a review finding,
/// F4).
/// With the place written before
///  the rename,
/// the same kill leaves a record whose target is gone and whose leaving place is named,
/// and the
///  next command finishes the registration the delete of that place belongs to.
/// A record that cannot be read or written
///  costs that finish and nothing else,
/// and the rename goes on as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordLeaving(commonDir, id, join(parentPath, deletingName(id)));
/// ```
pub fn record_leaving(common_dir: &Path, id: &OsStr, leaving: &Path) {
    let path: PathBuf = record_path(common_dir, id);
    let bytes: Vec<u8> = match read_plain_file(path.as_path(), RECORD_LIMIT) {
        Ok(read) => read,
        Err(error) => {
            debug(TAG, format!("{} cannot be read, so the place the subvolume of {id:?} stands is not recorded ({error})", path.display()).as_str());
            return;
        }
    };
    let read: DestroyRecord = match read_record(bytes.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("{} cannot be read, so the place the subvolume of {id:?} stands is not recorded ({reason})", path.display()).as_str());
            return;
        }
    };
    let mut named: Vec<u8> = Vec::new();
    if read.destroyed {
        named.extend_from_slice(DESTROYED_MARK);
    }
    if let Some(owner) = read.owner.as_ref() {
        named.extend_from_slice(format!("{OWNER_PREFIX}{owner}\n").as_bytes());
    }
    named.extend_from_slice(format!("{LEAVING_PREFIX}{}\n", hex_of_path(leaving)).as_bytes());
    named.extend_from_slice(format!("{IDENTITY_PREFIX}{}\n", read.admin.encode()).as_bytes());
    named.extend_from_slice(read.target.as_os_str().as_bytes());
    if let Err(error) = write_beside_then_rename(path.as_path(), named.as_slice()) {
        debug(TAG, format!("{} cannot be written with the place the subvolume of {id:?} stands ({error})", path.display()).as_str());
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
pub(crate) struct DestroyRecord {
    /// The process that wrote the record,
    /// and the identity that tells it from a later process with the same number;
    /// nothing for a record an earlier build wrote.
    pub(crate) owner: Option<String>,
    /// The identity of the registration when the destroy was proven.
    pub(crate) admin: ObjectId,
    /// Where this removal renamed the subvolume,
    /// when it handed it to the service:
    /// the place the worktree's files stand,
    /// written before the rename.
    pub(crate) leaving: Option<PathBuf>,
    /// The worktree directory the destroy was aimed at.
    pub(crate) target: PathBuf,
    /// Whether the kernel call returned,
    /// which is the only proof that the files are gone.
    pub(crate) destroyed: bool,
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
pub(crate) fn read_record(bytes: &[u8]) -> Result<DestroyRecord, String> {
    // The owner and the mark come first, one line each, so nothing a pathname holds can be taken for them; a record of an
    // earlier build has neither, or the mark alone at the front, and is read the same way. The mark that build wrote last is
    // not read: such a record counts as one whose kernel call is not proven, and recovery then leaves the registration alone
    // and says so.
    let mut rest: &[u8] = bytes;
    let mut owner: Option<String> = None;
    let mut leaving: Option<PathBuf> = None;
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
        if let Some(owner_text) = text.strip_prefix(OWNER_PREFIX) {
            owner = Some(owner_text.to_string());
            rest = after.strip_prefix(b"\n").unwrap_or(after);
            continue;
        }
        match text.strip_prefix(LEAVING_PREFIX) {
            Some(spelled) => {
                leaving = path_of_hex(spelled);
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
    return Ok(DestroyRecord { owner, admin, leaving, target: PathBuf::from(OsStr::from_bytes(target)), destroyed });
}

/// Finish controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_interrupted_destroy_tests.rs"]
mod tests;
