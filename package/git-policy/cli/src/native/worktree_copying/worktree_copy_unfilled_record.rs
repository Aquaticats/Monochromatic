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
use crate::path_bytes_in_records::{hex_of_bytes, hex_of_path, path_of_hex};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_journal::{Role, assert_private_path, journal_root, write_atomic};
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_selection::js_order;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  The suffix of a record that names a worktree left without part of its ignored state.
/// Why:
///   Journals end in `.json` and claims in `.stage`,
/// so a record of this kind is never read as one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNFILLED_SUFFIX = '.unfilled';
/// ```
pub(crate) const UNFILLED_SUFFIX: &str = ".unfilled";

/// What:
///  The field of the record that names its owner.
/// Why:
///   A record written by a running installation is that installation's,
/// and its owner is what tells a later command whether the
///  instrument may be settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWNER_FIELD = 'owner';
/// ```
const OWNER_FIELD: &str = "owner";

/// What:
///  The field of the record that names the worktree.
/// Why:
///   The bytes of the path are the one form that loses nothing,
/// and the record is read by commands that must act on exactly that worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DESTINATION_FIELD = 'destination';
/// ```
const DESTINATION_FIELD: &str = "destination";

/// What:
///  The field of the record that holds the worktree's identity.
/// Why:
///   A worktree removed and made again at the same path settles the record,
/// and only the object's identity tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDENTITY_FIELD = 'destinationIdentity';
/// ```
const IDENTITY_FIELD: &str = "destinationIdentity";

/// What:
///  The field of the record that names the worktree the ignored state came from.
/// Why:
///   The files the copy did not install are still in that source,
/// and the person needs its name to know what is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SOURCE_FIELD = 'source';
/// ```
const SOURCE_FIELD: &str = "source";

/// What:
///  The field of the record that holds what the installation ended with.
/// Why:
///   The command that ended the copy says why once;
/// the record keeps the reason for every later command that names the worktree again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REASON_FIELD = 'reason';
/// ```
const REASON_FIELD: &str = "reason";

/// What:
///  One record as written and as read back.
/// Why:
///   The writer and the reader must agree on every field the reader acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnfilledCopy = { destination: string; destinationIdentity: ObjectId; source: string; reason: string; owner?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnfilledCopy {
    /// The worktree that was left without part of its source's ignored state.
    pub destination: PathBuf,
    /// The identity of that worktree when the copy gave up.
    pub destination_identity: crate::fs_beneath::ObjectId,
    /// The worktree whose ignored state was being copied.
    pub source: PathBuf,
    /// What the installation ended with.
    pub reason: String,
    /// The owner that wrote the record,
    /// when the record names one.
    pub owner: Option<String>,
}

/// What:
///  The value of one `key=value` line of a record.
/// Why:
///   Every field of a record is one line,
/// and a field is looked up by its key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldOf = (text, key) => text.split('\n').find((line) => line.startsWith(`${key}=`))?.slice(key.length + 1);
/// ```
fn field_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(key) else { continue };
        if let Some(value) = rest.strip_prefix('=') {
            return Some(value);
        }
    }
    return None;
}

/// What:
///  The bytes a hexadecimal field spells.
/// Why:
///   The reason holds whatever the operating system said,
/// which is not text that one line can hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytesOfHex = (text) => Uint8Array.from(text.match(/../g) ?? [], (pair) => parseInt(pair, 16));
/// ```
fn bytes_of_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || !text.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.as_bytes().chunks(2) {
        bytes.push(u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
    }
    return Some(bytes);
}

/// What:
///  The reason a record's bytes are not the text a reader names.
/// Why:
///   The reason holds whatever the operating system said,
/// and a reader must be able to tell it from a record it cannot parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notText = () => 'what it says the copy ended with is not text';
/// ```
fn not_text(_undecodable: std::string::FromUtf8Error) -> &'static str {
    return "what it says the copy ended with is not text";
}

/// What:
///  The record of one worktree a copy gave up on,
/// as the lines its reader parses.
/// Why:
///   A record the reader cannot parse proves nothing about the worktree,
/// so the reader keeps it and says so instead of acting on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${DESTINATION_FIELD}=${hexOfPath(destination)}\n${IDENTITY_FIELD}=${id}\n${SOURCE_FIELD}=${hexOfPath(source)}\n${REASON_FIELD}=${hexOfBytes(reason)}\nowner=${owner}\n`
/// ```
pub fn encode_unfilled(copy: &UnfilledCopy) -> String {
    let owner: String = copy.owner.clone().unwrap_or_default();
    return format!(
        "{DESTINATION_FIELD}={}\n{IDENTITY_FIELD}={}\n{SOURCE_FIELD}={}\n{REASON_FIELD}={}\n{OWNER_FIELD}={owner}\n",
        hex_of_path(copy.destination.as_path()),
        copy.destination_identity.encode(),
        hex_of_path(copy.source.as_path()),
        hex_of_bytes(copy.reason.as_bytes()),
    );
}

/// What:
///  The record a command wrote,
/// or the reason its lines are not one this release writes.
/// Why:
///   Recovery acts on a worktree this record names,
/// so a line that is not of this release's shape authorizes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decodeUnfilled = (text) => ({ destination: pathOfHex(fieldOf(text, 'destination')), ... });
/// ```
pub fn decode_unfilled(text: &str) -> Result<UnfilledCopy, String> {
    let destination: PathBuf = path_of_hex(field_of(text, DESTINATION_FIELD).ok_or("it names no worktree")?).ok_or("its worktree is not a path")?;
    let identity: crate::fs_beneath::ObjectId = field_of(text, IDENTITY_FIELD)
        .and_then(crate::fs_beneath::ObjectId::decode)
        .ok_or("it holds no identity of the worktree")?;
    let source: PathBuf = path_of_hex(field_of(text, SOURCE_FIELD).ok_or("it names no source worktree")?).ok_or("its source worktree is not a path")?;
    let said: Vec<u8> = bytes_of_hex(field_of(text, REASON_FIELD).ok_or("it does not say what the copy ended with")?).ok_or("what it says the copy ended with is not bytes")?;
    let reason: String = String::from_utf8(said).map_err(not_text)?;
    let owner: Option<String> = match field_of(text, OWNER_FIELD) {
        Some(token) if !token.is_empty() => Some(token.to_string()),
        _ => None,
    };
    return Ok(UnfilledCopy {
        destination,
        destination_identity: identity,
        source,
        reason,
        owner,
    });
}

/// What:
///  The record's path for one journal:
/// the journal's identifier and this suffix.
/// Why:
///   A resumed copy renames its journal to its own owner token,
/// so a record named after the journal's whole name would be left behind
///  by every resumed attempt and reported as an unfilled worktree that is being filled;
/// the identifier alone stays one name for one copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unfilledPathOf = (journalPath) => join(dirname(journalPath), `${basename(journalPath, '.json').split('.')[0]}${UNFILLED_SUFFIX}`);
/// ```
pub fn unfilled_path_of(journal: &Path) -> PathBuf {
    let name: String = match journal.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let stem: &str = name.split('.').next().unwrap_or_default();
    return journal.with_file_name(format!("{stem}{UNFILLED_SUFFIX}"));
}

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
    return remember_unfilled_with(journal, destination, identity, source, owner, "the copy did not finish");
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
    return write_atomic(path.as_path(), encode_unfilled(&copy).as_bytes(), "unfilled-copy record");
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
fn unfilled_names(root: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    // `match` treats an absent root as having no records, and every other failure as fatal.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed.map_err(raw_failure_of)?.file_name().to_string_lossy().into_owned();
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
fn by_record_name<Name: AsRef<str>>(left: &Name, right: &Name) -> std::cmp::Ordering {
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
///  The record one file in the journal root holds,
/// nothing when the file is gone,
/// and the failure that keeps it when its lines are not this release's.
/// Why:
///   Recovery never acts on a record it cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readUnfilled = async (path) => { const text = await readFile(path, 'utf8').catch(absent => undefined); return text === undefined ? undefined : decodeUnfilled(text); };
/// ```
fn read_unfilled(path: &Path) -> Result<Option<UnfilledCopy>, WorktreeCopyError> {
    assert_private_path(path, Role::JournalFile)?;
    // `match` on the read: a record another command settled while this one looked is nobody's fault.
    let text: String = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read the record {} of a worktree whose ignored-state copy did not finish.",
                quote(path.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    // `match` on the decode keeps a record this release does not write, and names the file the person must check.
    return match decode_unfilled(text.as_str()) {
        Ok(copy) => Ok(Some(copy)),
        Err(reason) => Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: the record {} of a worktree whose ignored-state copy did not finish is not one this release writes ({reason}); nothing it names was changed, so check it and remove the file.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        )),
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
fn unfilled_notice(copy: &UnfilledCopy, worktree: &Path) -> String {
    let shown: String = quote(worktree.to_string_lossy().as_ref());
    let source: String = quote(copy.source.to_string_lossy().as_ref());
    let commands_destination: String = crate::shell_quote::shell_quote(worktree.as_os_str().as_bytes());
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
fn standing_at(common_dir: &Path, copy: &UnfilledCopy) -> Standing {
    // The recorded path answers for every worktree that was never moved, so no registration is read then.
    if crate::worktree_add_binding::path_identity(copy.destination.as_path()) == Some(copy.destination_identity) {
        return Standing::At(copy.destination.clone());
    }
    return match crate::worktree_copy_registration_naming::registered_object(common_dir, copy.destination_identity) {
        crate::worktree_copy_registration_naming::RegisteredObject::Root(root) => Standing::At(root),
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
pub(crate) fn standing_record_of(common_dir: &Path, destination: &Path) -> Result<Option<StandingRecord>, WorktreeCopyError> {
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
        let Some(copy) = read_unfilled(path.as_path())? else { continue };
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
        let Some(copy) = read_unfilled(path.as_path())? else { continue };
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
            Standing::At(destination) => notices.push(unfilled_notice(&copy, destination.as_path())),
            Standing::Unproven => {}
        }
    }
    return Ok(notices);
}

/// Unfilled-copy controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_unfilled_record_tests.rs"]
mod tests;
