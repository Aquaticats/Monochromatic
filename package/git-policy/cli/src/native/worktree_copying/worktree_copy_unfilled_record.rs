//! What:
//!  The durable record that one worktree was left without part of its source's ignored state:
//!  written before the first entry of an installation,
//!  removed when that installation completes,
//!  and reported by every command that runs the recovery until the worktree is gone or replaced.
//! Why:
//!  A copy that truly cannot finish leaves a worktree that looks ordinary and holds less than its source ignores,
//!  and a line printed once by the command that saw it is soon out of sight;
//! the record names the worktree and
//!  the two commands that settle it on every later command,
//! so nothing about the worktree is forgotten while it stands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await rememberUnfilled({ journal, source, reason });
//! await forgetUnfilled(journal.path);
//! const notices = await reportUnfilledRecords(commonDir);
//! ```

use crate::json_record::quote;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure,
    raw_failure_of,
};
use crate::worktree_copy_journal::{
    Role,
    assert_private_path,
    journal_root,
    write_atomic_unsynced,
};
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_selection::js_order;
pub(crate) use crate::worktree_copy_unfilled_record_words::UNFILLED_SUFFIX;
pub use crate::worktree_copy_unfilled_record_words::{
    UnfilledCopy,
    decode_unfilled,
    encode_unfilled,
    unfilled_path_of,
};
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Name the worktree a copy is about to fill,
/// before its first entry is created.
/// Why:
///   A kill at any instruction after this leaves the record,
/// so a worktree that never received its source's ignored state is
///  named by every later command;
/// the installation removes the record when it completes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rememberUnfilled({ journal, source, reason: 'in progress' });
/// ```
pub fn remember_unfilled(
    journal: &Path,
    destination: &Path,
    identity: crate::fs_beneath::ObjectId,
    source: &Path,
    owner: Option<&str>,
) -> Result<(), WorktreeCopyError> {
    return remember_unfilled_with(
        journal,
        destination,
        identity,
        source,
        owner,
        "the copy did not finish",
    );
}

/// What:
///  Name the worktree a copy gave up on,
/// with what ended the copy in the record itself.
/// Why:
///   A reason that lived only in the output of the command that saw the failure is gone with it;
/// the record repeats the words
///  on every later command,
/// so the person reads which entry differs or which worktree cannot be reached,
/// not only that something
///  is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rememberUnfilledWith({ journal, destination, identity, source, owner, reason });
/// ```
pub fn remember_unfilled_with(
    journal: &Path,
    destination: &Path,
    identity: crate::fs_beneath::ObjectId,
    source: &Path,
    owner: Option<&str>,
    reason: &str,
) -> Result<(), WorktreeCopyError> {
    let copy: UnfilledCopy = UnfilledCopy {
        destination: destination.to_path_buf(),
        destination_identity: identity,
        source: source.to_path_buf(),
        reason: reason.to_string(),
        owner: owner.map(str::to_string),
    };
    let path: PathBuf = unfilled_path_of(journal);
    // Published whole by its rename and flushed by nobody: it names a worktree whose copy did not finish, and a power
    // cut that loses it loses a notice, never a byte (`SPEC.md`, "The records an add writes for itself").
    return write_atomic_unsynced(
        path.as_path(),
        encode_unfilled(&copy).as_bytes(),
        "unfilled-copy record",
    );
}

/// What:
///  Forget the record of a worktree whose copy completed.
/// Why:
///   The record promises that ignored files are missing,
/// and an installation that finished keeps nothing of that promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forgetUnfilled(journal);
/// ```
pub fn forget_unfilled(journal: &Path) -> Result<(), WorktreeCopyError> {
    return remove_file_if_present(unfilled_path_of(journal).as_path());
}

/// What:
///  The names of the unfilled-copy records under a repository's journal root,
/// in record-name order.
/// Why:
///   The pending check and the sweep read the same set,
/// in an order that does not depend on the directory's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unfilledNames = (root) => readdir(root).filter((name) => name.endsWith(UNFILLED_SUFFIX)).sort(jsOrder);
/// ```
pub(crate) fn unfilled_names(root: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    // `match` treats an absent root as having no records, and every other failure as fatal.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed
            .map_err(raw_failure_of)?
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.ends_with(UNFILLED_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(by_record_name);
    return Ok(names);
}

/// What:
///  Order two records by name the way JavaScript's default sort does.
/// Why:
///   Which record is settled or named first must not depend on the directory's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.sort((left, right) => compareJs(left, right))
/// ```
fn by_record_name<Name: AsRef<str>>(
    left: &Name,
    right: &Name,
) -> std::cmp::Ordering {
    return js_order(left.as_ref(), right.as_ref());
}

/// What:
///  Whether any worktree is named as one a copy gave up on.
/// Why:
///   The pending check runs before every forwarded command and must not skip the line that names such a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unfilledRecordsExist = (commonDir) => unfilledNames(journalRoot(commonDir)).length > 0;
/// ```
pub fn unfilled_records_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    return Ok(!unfilled_names(journal_root(common_dir).as_path())?.is_empty());
}

/// What:
///  One record of a worktree a copy gave up on,
/// or the reason its lines are not one this release writes.
/// Why:
///   The sweep that settles an unreadable record needs that reason for its one line,
/// and every other reader needs only the records it may act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnfilledFile = { kind: 'record'; copy: UnfilledCopy } | { kind: 'unreadable'; reason: string };
/// ```
pub enum UnfilledFile {
    /// A record this release reads.
    Record(UnfilledCopy),
    /// A file at a record's name whose lines are not the record this release writes.
    Unreadable {
        /// What one line says about why it does not read.
        reason: String,
    },
}

/// What:
///  The record one file in the journal root holds,
/// nothing when the file is gone,
/// with the damage a crash leaves named instead of refused.
/// Why:
///   A record is published whole by one rename,
/// so a file at its name that does not decode is the mark of a write a crash cut short;
/// the command that found it moves it aside and goes on
/// (`worktree_copy_unreadable_records`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = await unfilledFileReading(path); // undefined when the file is gone
/// ```
pub(crate) fn unfilled_file_reading(
    path: &Path
) -> Result<Option<UnfilledFile>, WorktreeCopyError> {
    assert_private_path(path, Role::JournalFile)?;
    // `match` on the read: a record another command settled while this one looked is nobody's fault.
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read the record {} of a worktree whose ignored-state copy did not finish.",
                quote(path.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(Some(UnfilledFile::Unreadable {
            reason: String::from("its bytes are not text"),
        }));
    };
    return match decode_unfilled(text.as_str()) {
        Ok(copy) => Ok(Some(UnfilledFile::Record(copy))),
        Err(reason) => Ok(Some(UnfilledFile::Unreadable {
            reason: reason.to_string(),
        })),
    };
}

/// What:
///  The record one file in the journal root holds,
/// nothing when the file is gone or its lines are not this release's,
/// and the failure that keeps a file it may not read.
/// Why:
///   A record that does not decode names no worktree,
/// so a sweep has nothing to act on there;
/// the command that found it settles it first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = await readUnfilled(path); // undefined when gone or unreadable
/// ```
fn read_unfilled(path: &Path) -> Result<Option<UnfilledCopy>, WorktreeCopyError> {
    return match unfilled_file_reading(path)? {
        Some(UnfilledFile::Record(copy)) => Ok(Some(copy)),
        Some(UnfilledFile::Unreadable {
            ..
        })
        | None => Ok(None),
    };
}

/// What:
///  The line that names one worktree as one a copy gave up on,
/// with the commands that settle it.
/// Why:
///   The person needs the worktree,
/// the source that still holds what is missing,
/// what ended the copy,
/// and the two commands
///  that remove the worktree or fill it;
/// a worktree Git moved stands at a path the record never wrote,
/// and the commands must name the path it stands at now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${shown} was left without part of the ignored state of ${source}: ${reason} ...`
/// ```
fn unfilled_notice(
    copy: &UnfilledCopy,
    worktree: &Path,
) -> String {
    let shown: String = quote(worktree.to_string_lossy().as_ref());
    let source: String = quote(copy.source.to_string_lossy().as_ref());
    let commands_destination: String =
        crate::shell_quote::shell_quote(worktree.as_os_str().as_bytes());
    return format!(
        "monochromatic-git-policy-cli: {shown} holds less than {source} ignores: the ignored-state copy ended before it finished ({}). Nothing in that worktree was changed by this command, and the files it did not receive are still in {source}. `git worktree remove --force {commands_destination}` removes that worktree, and running the same `git worktree add` line again copies the ignored state; this line comes again from every command in this repository until that worktree is removed or replaced.",
        copy.reason
    );
}

/// What:
///  One record still standing,
/// with the file it was read from.
/// Why:
///   A caller that judges the owner's liveness needs the record's own file,
/// and the worktree it names is what a caller asks about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StandingRecord = { path: string; destination: string; identity: ObjectId; owner?: string };
/// ```
pub struct StandingRecord {
    /// The file the record was read from.
    pub path: PathBuf,
    /// The worktree the copy did not fill,
    /// at the path it stands at now.
    pub destination: PathBuf,
    /// The object the record captured,
    /// which is the worktree's identity wherever Git has moved it.
    pub identity: crate::fs_beneath::ObjectId,
    /// The owner token the record names,
    /// when it names one.
    pub owner: Option<String>,
}

/// What:
///  Where the worktree a record names stands,
///  or why that cannot be told.
/// Why:
///   `git worktree move` renames the worktree's directory and keeps its object,
/// so the recorded path names nothing from then on while the worktree still stands;
/// the identity the record captured is the proof that names it either way,
/// and the path it stands at now is the one every line and command about it must name.
///   A record whose object stands nowhere describes a worktree that is gone or was replaced,
/// and one whose registration cannot be read describes a worktree nobody has proven gone,
/// so the answer that proves nothing is kept apart from the answer that proves absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Standing = { path: string } | 'nowhere' | 'unproven';
/// ```
enum Standing {
    /// The worktree stands at this path.
    At(PathBuf),
    /// The worktree is gone or was replaced.
    Nowhere,
    /// A registration that could name the worktree cannot be read,
    /// so where it stands is unknown.
    Unproven,
}

/// What:
///  Where the worktree a record names stands now:
///  the path the record wrote while the object it recorded is there,
///  else the root of the registration of this repository that is that object.
/// Why:
///   `git worktree move` renames the worktree's directory and keeps its object,
/// so the recorded path names nothing from then on while the worktree still stands;
/// the identity the record captured is the proof that names it either way,
/// and the path it stands at now is the one every line and command about it must name.
///   A record whose object stands nowhere describes a worktree that is gone or was replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standingAt = (commonDir, copy) => identityAt(copy.destination) === copy.destinationIdentity ? { path: copy.destination } : registeredObject(commonDir, copy.destinationIdentity);
/// ```
fn standing_at(
    common_dir: &Path,
    copy: &UnfilledCopy,
) -> Standing {
    // The recorded path answers for every worktree that was never moved, so no registration is read then.
    if crate::worktree_add_binding::path_identity(copy.destination.as_path())
        == Some(copy.destination_identity)
    {
        return Standing::At(copy.destination.clone());
    }
    return match crate::worktree_copy_registration_naming::registered_object(
        common_dir,
        copy.destination_identity,
    ) {
        crate::worktree_copy_registration_naming::RegisteredObject::Root(root) => {
            Standing::At(root)
        }
        crate::worktree_copy_registration_naming::RegisteredObject::None => Standing::Nowhere,
        crate::worktree_copy_registration_naming::RegisteredObject::Unknown => Standing::Unproven,
    };
}

/// What:
///  The record of a worktree a copy ended in,
///  while the worktree it names still stands,
///  whichever path it stands at now.
/// Why:
///   A record whose worktree is gone or was replaced describes nothing that stands,
/// so it holds no copy back and its
///  own command settles it;
/// the object the record captured is what the caller's worktree is compared with,
/// so the worktree Git moved is the one the record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = await standingRecords(commonDir).find((record) => record.identity === identityAt(sourceRoot));
/// ```
pub(crate) fn standing_record_of(
    common_dir: &Path,
    destination: &Path,
) -> Result<Option<StandingRecord>, WorktreeCopyError> {
    let Some(asked) = crate::worktree_add_binding::path_identity(destination) else {
        return Ok(None);
    };
    for record in standing_records(common_dir)? {
        if record.identity == asked {
            return Ok(Some(record));
        }
    }
    return Ok(None);
}

/// What:
///  Every record whose worktree is still the object it names.
/// Why:
///   The proof is the one every reader of these records takes:
/// the object the record captured is the object of the worktree that stands,
/// at the path the record wrote or at the path a registration of this repository gives it,
/// so a record of a worktree that was removed or replaced describes nothing that stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of unfilledNames(root)) { const copy = await readUnfilled(root/name); if (copy && standingAt(commonDir, copy)) yield copy }
/// ```
fn standing_records(common_dir: &Path) -> Result<Vec<StandingRecord>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let mut standing: Vec<StandingRecord> = Vec::new();
    for name in unfilled_names(root.as_path())? {
        let path: PathBuf = root.join(name.as_str());
        let Some(copy) = read_unfilled(path.as_path())? else {
            continue;
        };
        // Only a worktree that is still the object the record captured is one it describes: one that was removed or
        // replaced settles the record instead, and one Git moved is still described, at the path it stands at now.
        // A record nobody has proven either way is not one this caller can hold a copy back with, and it is not settled.
        let Some(destination) = (match standing_at(common_dir, &copy) {
            Standing::At(destination) => Some(destination),
            Standing::Nowhere | Standing::Unproven => None,
        }) else {
            continue;
        };
        standing.push(StandingRecord {
            path,
            destination,
            identity: copy.destination_identity,
            owner: copy.owner,
        });
    }
    return Ok(standing);
}

/// What:
///  Settle every record whose worktree is gone or replaced,
/// and name every worktree that still holds less than its source ignores.
/// Why:
///   A record settles only against a proof about the worktree,
/// and a worktree that still stands is named again on
///  every command that runs the recovery,
/// at the path it stands at now.
/// A record whose owner still runs belongs to a copy in progress,
/// so it is left without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await reportUnfilledRecords(commonDir);
/// ```
pub fn report_unfilled_records(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let mut notices: Vec<String> = Vec::new();
    for name in unfilled_names(root.as_path())? {
        let path: PathBuf = root.join(name.as_str());
        let Some(copy) = read_unfilled(path.as_path())? else {
            continue;
        };
        // A copy that still runs wrote the record before its first entry: it is filling this worktree now.
        if !token_owner_is_dead(copy.owner.as_deref(), path.as_path())? {
            continue;
        }
        // `match` on where the worktree stands: one that is gone or was replaced settles the record, one that still
        // stands is named where it is now, which a `git worktree move` changed from the path the record wrote, and one
        // whose registration cannot be read is kept for a later pass rather than settled on a guess.
        match standing_at(common_dir, &copy) {
            Standing::Nowhere => {
                remove_file_if_present(path.as_path())?;
            }
            Standing::At(destination) => {
                notices.push(unfilled_notice(&copy, destination.as_path()))
            }
            Standing::Unproven => {}
        }
    }
    return Ok(notices);
}

/// Unfilled-copy controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_unfilled_record_tests.rs"]
mod tests;
