//! What:
//!  The record a removal writes before its pass empties a worktree around a volume it did not make,
//!  so a command killed inside that pass leaves a state the next command finishes instead of one only
//!  `--force` clears.
//! Why:
//!  A removal without `--force` proves the worktree clean, and only then empties it.
//! The pass deletes the worktree's own entries first
//!  and forgets the worktree in Git after it,
//! so a command killed in between leaves tracked files deleted,
//! the volume standing and the
//!  registration kept.
//! The next unforced removal then meets a worktree Git reports as changed,
//! and without a record nothing can tell those
//!  deletions from a person's:
//! the refused state would be a dead end that only `--force` opens,
//! and a state the wrapper itself
//!  made would be reported as one a person made.
//!  The record names the registration and the worktree it was written for,
//! so a worktree a
//!  person removed and added again at the same path never matches it.
//!  It lies in a folder of its own and in a file of this build's shape,
//! so a release that does not know it reads
//!  nothing of it and removes nothing because of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordAroundBegun(commonDir, { id, admin, worktree, target });
//! const begun = aroundBegunBy(commonDir, { id, admin, worktree, target });
//! forgetAroundBegun(commonDir, id);
//! ```

use crate::fs_beneath::ObjectId;
use crate::path_bytes_in_records::{hex_of_path, path_of_hex};
use crate::plain_file::read_plain_file;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::process_identity::current_birth_identity;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The folder of the common Git directory that holds one record per removal whose pass began,
/// named by the registration's id.
pub const BEGUN_FOLDER: &str = "monochromatic-git-policy-cli/around-removals-begun";

/// The most bytes of a record that is read:
/// five short lines and the hexadecimal form of a path, which is twice its length.
const RECORD_LIMIT: u64 = 16384;

/// The first line of every record,
/// which tells it from a file of anything else this program keeps.
const MAGIC: &str = "around-removal-begun=1";

/// The line that names the process which wrote the record,
/// and the identity that tells it from a later process of the same number.
const OWNER_PREFIX: &str = "owner=";

/// The line that names the registration the pass was proven against.
const ADMIN_PREFIX: &str = "admin=";

/// The line that names the worktree the pass was emptied.
const WORKTREE_PREFIX: &str = "worktree=";

/// The line that names the path the removal was given,
/// in the hexadecimal form a path of any bytes takes in a line of a record.
const TARGET_PREFIX: &str = "targetHex=";

/// What:
///  The identity a record must match for one removal to be taken as a continuation of another.
/// Why:
///   The registration's identity says the pass was proven against the registration standing there now,
/// and the worktree's says
///  the tree is the one that pass emptied:
/// a person who removed the worktree and added another at the same path is
///  two other objects, and an earlier record removes nothing of theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhichRemoval = { id: OsStr; admin: ObjectId; worktree: ObjectId; target: Path };
/// ```
#[derive(Clone, Copy)]
pub struct WhichRemoval<'a> {
    /// The name of the registration below `worktrees`.
    pub id: &'a OsStr,
    /// The identity of the registration when the pass was proven against it.
    pub admin: ObjectId,
    /// The identity of the worktree root the pass emptied.
    pub worktree: ObjectId,
    /// The path the removal was given.
    pub target: &'a Path,
}

/// The path of the record of one registration.
fn record_path(common_dir: &Path, id: &OsStr) -> PathBuf {
    return common_dir.join(BEGUN_FOLDER).join(id);
}

/// What:
///  Record that this command is about to empty the worktree `which` names around the volumes it did not make.
/// Why:
///   The record is written before the first entry of the worktree goes,
/// so a command killed in the pass leaves
///  the proof that the entries Git then reports missing are this program's own.
/// A record that cannot be written costs that
///  proof and nothing else:
/// the removal goes on,
/// and a command killed in it leaves a state the words of the
///  refusal name and `--force` finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordAroundBegun(commonDir, which);
/// ```
///
/// # Errors
///
/// The words for a folder or a record that cannot be written,
/// which a caller logs and never fails a removal for.
pub fn record_around_begun(common_dir: &Path, which: WhichRemoval<'_>) -> Result<(), String> {
    let path: PathBuf = record_path(common_dir, which.id);
    if let Some(parent) = path.parent()
        && let Err(error) = create_private_directory_all(parent)
    {
        return Err(format!("{} cannot be made ({error})", parent.display()));
    }
    let owner: String = match current_birth_identity() {
        Ok(identity) => format!("{OWNER_PREFIX}{}:{identity}\n", std::process::id()),
        // A record without an owner is one an earlier build wrote, which is read as abandoned.
        Err(_unknown) => String::new(),
    };
    let text: String = format!(
        "{MAGIC}\n{owner}{ADMIN_PREFIX}{admin}\n{WORKTREE_PREFIX}{worktree}\n{TARGET_PREFIX}{target}\n",
        admin = which.admin.encode(),
        worktree = which.worktree.encode(),
        target = hex_of_path(which.target)
    );
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("the record {} of a removal that goes around a volume cannot be written ({error})", path.display())),
    };
}

/// What:
///  Whether an earlier command's pass emptied this worktree and was cut short before it finished the removal.
/// Why:
///   The answer is what lets the same unforced command finish the removal it began,
/// and what keeps it from
///  reading a person's own state as this program's work:
/// both identities and the path must match,
/// and the record
///  must be readable in the shape this build writes.
/// A record that cannot be read proves nothing and is said in the
///  diagnostic log,
///  so the refusal a person then reads is one they can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const begun = aroundBegunBy(commonDir, which); // boolean
/// ```
pub fn around_begun_by(common_dir: &Path, which: WhichRemoval<'_>) -> bool {
    let path: PathBuf = record_path(common_dir, which.id);
    let bytes: Vec<u8> = match read_plain_file(path.as_path(), RECORD_LIMIT) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return false,
        Err(error) => {
            crate::diagnostic_log::debug("worktreeRemovalAroundBegun", format!("{} cannot be read ({error}), so nothing proves an earlier pass emptied this worktree", path.display()).as_str());
            return false;
        }
    };
    let Ok(text) = std::str::from_utf8(bytes.as_slice()) else {
        crate::diagnostic_log::debug("worktreeRemovalAroundBegun", format!("{} is not text this build wrote, so nothing proves an earlier pass emptied this worktree", path.display()).as_str());
        return false;
    };
    return read_record(text).is_some_and(|read| {
        return read.admin == which.admin && read.worktree == which.worktree && read.target == which.target;
    });
}

/// What:
///  Drop the record of one registration,
///  whose removal finished or whose pass never began.
/// Why:
///   A record that outlived its removal would let the next one take a state of its own for a continuation,
/// so it goes
///  with the registration it names,
///  and the folder goes with the last record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetAroundBegun(commonDir, id);
/// ```
pub fn forget_around_begun(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            crate::diagnostic_log::warn("worktreeRemovalAroundBegun", format!("the record {} cannot be removed ({error}); a later removal of another worktree tries the same thing again", path.display()).as_str());
            return;
        }
    }
    // The folder goes when the last record does; one that still holds another record stays, which its own failure says.
    let folder: PathBuf = common_dir.join(BEGUN_FOLDER);
    let _kept = std::fs::remove_dir(folder.as_path());
}

/// What:
///  Drop every record whose registration is no longer in this repository.
/// Why:
///   A worktree a person removed with `--force` through Git,
/// or one `git worktree prune` forgot,
/// leaves a record
///  nothing will ever finish,
/// and a folder that grows with every such removal costs a reader of it every command.
/// A
///  registration that is still there keeps its record,
/// because that is what a later removal reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pruneAroundBegun(commonDir);
/// ```
pub fn prune_around_begun(common_dir: &Path) {
    let folder: PathBuf = common_dir.join(BEGUN_FOLDER);
    let Ok(listed) = std::fs::read_dir(folder.as_path()) else { return };
    let registrations: PathBuf = common_dir.join("worktrees");
    for found in listed.flatten() {
        let name: std::ffi::OsString = found.file_name();
        if registrations.join(name.as_os_str()).is_dir() {
            continue;
        }
        let _removed = std::fs::remove_file(found.path());
    }
    let _kept = std::fs::remove_dir(folder.as_path());
}

/// What a record holds:
///  the registration and the worktree the pass was proven against,
///  and the path the removal was given.
/// Why:
///   The identities say whether this is the removal a later command may finish,
/// and the path is what a message
///  names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AroundRecord = { admin: ObjectId; worktree: ObjectId; target: Path };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct AroundRecord {
    /// The identity of the registration when the pass was proven.
    admin: ObjectId,
    /// The identity of the worktree root the pass emptied.
    worktree: ObjectId,
    /// The path the removal was given.
    target: PathBuf,
}

/// What:
///  The record a text holds,
///  or nothing when it is not one this build wrote.
/// Why:
///   The records folder is read on every command,
///  so a file
///  this build cannot read is left for the build that wrote it rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = readRecord(text); // AroundRecord | undefined
/// ```
fn read_record(text: &str) -> Option<AroundRecord> {
    let mut lines = text.lines();
    if lines.next()? != MAGIC {
        return None;
    }
    let mut admin: Option<ObjectId> = None;
    let mut worktree: Option<ObjectId> = None;
    let mut target: Option<PathBuf> = None;
    for line in lines {
        if line.starts_with(OWNER_PREFIX) {
            continue;
        }
        if let Some(spelled) = line.strip_prefix(ADMIN_PREFIX) {
            admin = ObjectId::decode(spelled);
            continue;
        }
        if let Some(spelled) = line.strip_prefix(WORKTREE_PREFIX) {
            worktree = ObjectId::decode(spelled);
            continue;
        }
        if let Some(spelled) = line.strip_prefix(TARGET_PREFIX) {
            target = path_of_hex(spelled);
            continue;
        }
    }
    return Some(AroundRecord { admin: admin?, worktree: worktree?, target: target? });
}

/// The records of a removal that goes around a volume stay out of the release executable's tests.
#[cfg(test)]
#[path = "worktree_removal_around_begun_tests.rs"]
mod tests;
