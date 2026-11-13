//! What:
//!  The judging of stage claims and their temporaries,
//! and the removal of the stages that interrupted copies left behind.
//! Why:
//!  A claim is the record that a stage may exist.
//! A claim whose owner is dead and whose stage no journal adopted is reaped;
//! a
//!  stage a copy left is a private directory this wrapper made,
//! while a stage an add left is removed only when the record of
//!  the object its creation made proves that object still stands there and holds nothing the record cannot account for
//!  (`worktree_copy_stage_made_object.rs`,
//! `worktree_copy_stage_entries_the_record_accounts_for.rs`);
//! a stage that record does
//!  not prove is reported and never removed,
//! and the claim stays so that the next command reports it again.
//! What an interrupted `git worktree add` settles to lives beside this file,
//! and is named here so one import path reaches both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = await judgeStageOwners(commonDir);
//! const notices = await reapJudgedStages(commonDir, verdicts);
//! ```

use crate::diagnostic_log::debug;
use crate::json_record::quote;
use crate::path_bytes_in_records::path_of_the_two_fields;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_recovery::RecoveryDestinations;
use crate::worktree_copy_journal::{Role, assert_private_path, journal_root, read_pending_journals, temporary_writer_token};
use crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX;
use crate::worktree_copy_journal_removal::{remove_file_if_present, remove_stage};
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_selection::{STAGE_PREFIX, js_order};
use crate::worktree_copy_stage::PAYLOAD_NAME;
use crate::worktree_copy_stage_claim::{COMPLETE_SUFFIX, CONTAINER_HEX_PREFIX, DESTINATION_SUFFIX, OWNER_LINE_PREFIX, claim_names};
use crate::worktree_copy_stage_claim_owed_copy_elsewhere::copy_owed_to_another_worktree;
use crate::worktree_copy_stage_made_object::remove_made;
use crate::worktree_copy_stage_reaping_proof_and_words::{Provenance, kept_copy_stage_words, kept_stage_words, provenance_of, removed_copy_stage_words, removed_stage_words, still_there};
// The sweeps that remove files rather than stages live beside this file, so this one stays under the line limit.
pub use crate::worktree_copy_stage_claim_swept_files::remove_dead_temporaries;
pub(crate) use crate::worktree_copy_stage_claim_swept_files::remove_records_nothing_claims;
/// The tag every line this module logs carries.
const TAG: &str = "stageReaping";

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What an interrupted `git worktree add` settles to,
/// the proof its checkout is whole or gone,
/// and the names of the files that proof reads:
/// re-exported so a reader of the claims and a control of them reach them here.
/// The settlement lives in a file of its own (`worktree_copy_interrupted_add_settlement.rs`),
/// which is where this
///  branch's proof of the destination's pointer now is too.
pub use crate::worktree_copy_interrupted_add_settlement::*;

/// What:
///  The container and the owner a claim names.
/// Why:
///   `read_claim` acts on these values,
/// so the shape of a record is decided in one place.
struct ClaimFields<'a> {
    /// The container as text,
    /// when the record spells it on one line.
    text: Option<&'a str>,
    /// The container's bytes,
    /// when the record holds them.
    bytes: Option<&'a str>,
    /// The owner token,
    /// when the record names one.
    owner: Option<String>,
}

/// What:
///  Whether this account owns the folder,
/// it is a directory,
/// and its canonical path is the path given.
/// Why:
///   A removal is recursive,
/// and only a folder this account made may be examined and removed at all;
/// another account's folder,
/// a link to one,
/// and a file at the name are every one of them somebody else's and are named instead.
/// The mode is not asked:
/// a stage name holds the directory Git made after an interrupted exchange,
/// at Git's own mode,
/// and what may be removed from it is decided by what it holds (`an_entry_the_wrappers_stage_does_not_explain`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours = (path) => isDirectory(path) && owner(path) === euid() && canonicalize(path) === path;
/// ```
fn is_this_accounts_directory(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else { return false };
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    let effective_user: u32 = unsafe { libc::geteuid() };
    if !metadata.is_dir() || metadata.uid() != effective_user {
        return false;
    }
    return std::fs::canonicalize(path).is_ok_and(|canonical: PathBuf| { return canonical.as_path() == path; });
}

/// What:
///  Whether any of these files stands,
/// answering yes when one cannot be looked up for any reason but absence.
/// Why:
///   Every one of them is a proof that a stage folder is an add's,
/// and a proof that could not be read is not an absence of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standsAnyOf = (paths) => paths.some((path) => existsOrUnknown(path));
/// ```
fn stands_any_of(paths: &[&Path]) -> bool {
    for path in paths {
        if !matches!(std::fs::symlink_metadata(path), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the wrapper's own writing explains this top-level name of a stage container.
/// Why:
///   These are the only names a copy leaves at the top of its stage:
/// the registration of the directory Git made and the temporaries its write is published through,
/// the payload,
/// and the record of the entries the payload will hold with its own temporary.
/// A name that begins like one of those records and ends with a temporary's suffix is that write itself,
/// interrupted before its rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = (name) => name === PAYLOAD || name === INTENDED || isARecordWrite(name, REGISTRATION) || isARecordWrite(name, INTENDED);
/// ```
fn names_a_top_level_entry_the_wrapper_writes(name: &str) -> bool {
    if name == PAYLOAD_NAME || name == crate::worktree_copy_stage_intended_entries::INTENDED_NAME {
        return true;
    }
    return names_a_record_write(name, crate::worktree_copy_stage_binding::STAGE_BINDING_NAME)
        || names_a_record_write(name, crate::worktree_copy_stage_intended_entries::INTENDED_NAME);
}

/// What:
///  Whether one name is the record of that name, or a write of it.
/// Why:
///   A record is published by one rename of a hidden file it was written in,
/// so a kill leaves either the record or one of those two names,
/// and a name of another shape is somebody else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = (name, record) => name === record || (name.startsWith(`${record}.`) && (name.endsWith('.tmp') || name.endsWith('.partial')));
/// ```
fn names_a_record_write(name: &str, record: &str) -> bool {
    let bare: &str = name.strip_prefix('.').unwrap_or(name);
    let Some(rest): Option<&str> = bare.strip_prefix(record) else {
        return false;
    };
    if rest.is_empty() {
        return true;
    }
    return rest.starts_with('.') && (rest.ends_with(TEMPORARY_SUFFIX) || rest.ends_with(SCRATCH_SUFFIX));
}

/// What:
///  The first entry of a stage folder that the wrapper's own writing does not explain,
/// at any depth,
/// or nothing when every entry is one this wrapper makes there.
/// Why:
///   A dead claim with no record proves nothing about the folder it names,
/// so the folder is removed only when nothing inside it can be a person's:
/// the entries of the payload must be the paths the copy wrote down before it wrote them,
/// with the directories above those paths,
/// and the top of the stage holds only the records this wrapper writes.
/// Anything else at any depth keeps the folder and is named,
/// because a person's file there exists nowhere else and the removal is recursive.
/// A payload without that record proves nothing:
/// no copy of this release leaves one,
/// and a folder an earlier release left is kept and named rather than guessed at.
/// Only a payload that holds nothing at all is removed without one,
/// because there is nothing in it to lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unexplained = firstEntryTheWrappersWritingDoesNotExplain(container);
/// ```
fn an_entry_the_wrappers_stage_does_not_explain(container: &Path) -> Result<Option<String>, WorktreeCopyError> {
    let listing = match std::fs::read_dir(container) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        names.push(listed.map_err(raw_failure_of)?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    for name in names {
        if !names_a_top_level_entry_the_wrapper_writes(name.as_str()) {
            return Ok(Some(format!("it holds {name}, which the wrapper's writing does not hold there")));
        }
    }
    let payload: PathBuf = container.join(PAYLOAD_NAME);
    match std::fs::symlink_metadata(payload.as_path()) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    let holds_nothing: bool = std::fs::read_dir(payload.as_path()).is_ok_and(|mut found: std::fs::ReadDir| { return found.next().is_none(); });
    if holds_nothing {
        return Ok(None);
    }
    let Some(allowed) = crate::worktree_copy_stage_intended_entries::entries_the_payload_may_hold(container) else {
        return Ok(Some(format!("it holds {PAYLOAD_NAME}/, and nothing says what the copy was writing into it")));
    };
    return first_entry_the_payload_may_not_hold(payload.as_path(), allowed);
}

/// What:
///  The first entry below a payload that is not one of the paths the copy recorded,
/// or a directory above one of them,
/// as the words the person reads,
/// or nothing when every entry is one of those.
/// Why:
///   The walk reads and never writes,
/// opens no file and follows no link,
/// and stops at the first entry that is somebody else's,
/// so a payload the wrapper filled costs one lookup per entry and a payload holding a person's file usually
/// costs fewer.
/// A directory that cannot be listed is answered with its reason,
/// so a look that failed is never an answer of absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const foreign = firstEntryThePayloadMayNotHold(payload, allowed); // undefined when every entry is the copy's
/// ```
pub(crate) fn first_entry_the_payload_may_not_hold(payload: &Path, allowed: std::collections::HashSet<Vec<u8>>) -> Result<Option<String>, WorktreeCopyError> {
    use std::os::unix::ffi::OsStrExt as _;
    let mut pending: Vec<(PathBuf, Vec<u8>)> = vec![(payload.to_path_buf(), Vec::new())];
    while let Some((directory, relative)) = pending.pop() {
        let listing: std::fs::ReadDir = match std::fs::read_dir(directory.as_path()) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Ok(Some(format!("nothing shows what stands below {} ({error})", crate::worktree_copy_repository_path::shown_repository_path(relative.as_slice())))),
        };
        for found in listing {
            let entry: std::fs::DirEntry = match found {
                Ok(entry) => entry,
                Err(error) => return Ok(Some(format!("nothing shows what stands below {} ({error})", crate::worktree_copy_repository_path::shown_repository_path(relative.as_slice())))),
            };
            let name: std::ffi::OsString = entry.file_name();
            let mut below: Vec<u8> = relative.clone();
            if !below.is_empty() {
                below.push(b'/');
            }
            below.extend_from_slice(name.as_os_str().as_bytes());
            if !allowed.contains(&below) {
                return Ok(Some(format!(
                    "it holds {PAYLOAD_NAME}/{}, and nothing in the copy's own record names that path",
                    crate::worktree_copy_repository_path::shown_repository_path(below.as_slice())
                )));
            }
            let is_directory: bool = match std::fs::symlink_metadata(entry.path().as_path()) {
                Ok(metadata) => metadata.is_dir(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Ok(Some(format!("nothing shows what stands below {} ({error})", crate::worktree_copy_repository_path::shown_repository_path(below.as_slice())))),
            };
            if is_directory {
                pending.push((entry.path(), below));
            }
        }
    }
    return Ok(None);
}

/// What:
///  Whether a line of a claim holds the container's bytes.
/// Why:
///   A reader must place the bytes line and count it,
/// so both questions are asked in one way of one name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsBytes = (line: string) => line.startsWith('containerHex=')
/// ```
fn holds_bytes(line: &str) -> bool {
    return line.starts_with(CONTAINER_HEX_PREFIX);
}

/// What:
///  The lines of a claim as this release and the one before it write them:
/// the container as text,
/// its bytes on the line after it,
/// then the owner.
/// Why:
///   The container is removed recursively,
/// so a line of bytes is believed only in the two places a writer puts one,
/// and a claim with any other line is refused:
/// the earlier release wrote a container path verbatim,
/// so a path holding a line break put lines of its own in the record,
/// and one of them could name any private directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimFields = (text: string) => { const lines = text.split('\n'); const textLine = lines[0]?.startsWith('containerHex=') ? undefined : lines[0]; ... }
/// ```
fn claim_fields(text: &str) -> Result<ClaimFields<'_>, String> {
    let lines: Vec<&str> = text.lines().collect();
    // The first line is the container's bytes or the container's text; its bytes then sit where this release puts them.
    let named_text: Option<&str> = match lines.first() {
        Some(line) if holds_bytes(line) => None,
        first => first.copied(),
    };
    let named_bytes: Option<&str> = match lines.get(usize::from(named_text.is_some())) {
        Some(line) => line.strip_prefix(CONTAINER_HEX_PREFIX),
        None => None,
    };
    let read: usize = usize::from(named_text.is_some()) + usize::from(named_bytes.is_some());
    // A line of container bytes past those is a path that ended early, not a line a writer put there.
    if lines.iter().skip(read).copied().any(holds_bytes) {
        return Err(String::from("its line of container bytes is not where this release writes one"));
    }
    // The owner, when the record names one, is the last line.
    let mut following = lines.iter().skip(read);
    let Some(first) = following.next() else {
        return Ok(ClaimFields { text: named_text, bytes: named_bytes, owner: None });
    };
    let Some(token) = first.strip_prefix(OWNER_LINE_PREFIX) else {
        return Err(String::from("a line this release does not write follows its container"));
    };
    if following.next().is_some() {
        return Err(String::from("more lines follow its owner"));
    }
    return Ok(ClaimFields { text: named_text, bytes: named_bytes, owner: Some(token.to_string()) });
}

/// What:
///  One stage claim as read:
/// the container it names and its owner token,
/// or the reason its bytes are not a record this release writes.
/// Why:
///   The sweep that settles an unreadable claim needs that reason for its one line,
/// and every other reader needs only the claims it may act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimFile = { kind: 'named'; container: string; owner?: string } | { kind: 'unreadable'; reason: string };
/// ```
pub enum ClaimFile {
    /// A claim this release reads.
    Named {
        /// The stage container it names,
        /// absolute and under a name this wrapper reserves.
        container: PathBuf,
        /// The owner token,
        /// when the record names one.
        owner: Option<String>,
    },
    /// A file at a claim's name whose bytes are not the record this release writes.
    Unreadable {
        /// What one line says about why it does not read.
        reason: String,
    },
}

/// What:
///  One claim file as read,
/// or nothing when it is gone,
/// with the damage a crash can leave named instead of refused.
/// Why:
///   A claim is published whole by one rename,
/// so a file at its name that does not read is the mark of a write a crash cut short;
/// the command that found it moves it aside and settles what it named
/// (`worktree_copy_unreadable_records`).
/// A claim another account owns,
/// or a link or a pipe at its name,
/// stays a refusal:
/// no crash writes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = await claimFileReading(claim); // undefined when the claim is gone
/// ```
pub fn claim_file_reading(claim: &Path) -> Result<Option<ClaimFile>, WorktreeCopyError> {
    // `match` on the lookup: a claim already gone is nobody's fault, and nothing is left to reap.
    match std::fs::symlink_metadata(claim) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    // A claim with a second name is the half-done keep of a release before this one: its bytes were linked into the
    // folder beside the records before the claim was written again, and every command then refused the claim for ever
    // after. It is answered as unreadable here, so the pass that settles a claim it cannot read finishes that keep
    // (issue 6297 covers the link; the release that wrote it is 0.6.1-alpha.15's).
    if std::fs::symlink_metadata(claim).is_ok_and(|found: std::fs::Metadata| { return std::os::unix::fs::MetadataExt::nlink(&found) > 1; }) {
        return Ok(Some(ClaimFile::Unreadable { reason: String::from("it has a second name, and a claim of this release has one") }));
    }
    assert_private_path(claim, Role::JournalFile)?;
    // `match` on the read: the claim can vanish between the lookup and the read, and bytes that are not text are damage.
    let text: String = match std::fs::read_to_string(claim) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Ok(Some(ClaimFile::Unreadable { reason: String::from("its bytes are not text") }));
        }
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read worktree-copy stage claim {}.",
                quote(claim.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    let fields: ClaimFields<'_> = match claim_fields(text.as_str()) {
        Ok(fields) => fields,
        Err(reason) => return Ok(Some(ClaimFile::Unreadable { reason: reason.to_string() })),
    };
    let container: PathBuf = match path_of_the_two_fields(fields.bytes, fields.text) {
        Ok(Some(path)) => path,
        Ok(None) => PathBuf::new(),
        Err(reason) => {
            return Ok(Some(ClaimFile::Unreadable { reason: format!("it holds a container in bytes that is not a path ({reason})") }));
        }
    };
    // Both lines of a claim this release writes name one path; a line of text that says otherwise is a shorter path
    // that ended where the line break was, and the bytes it stood beside are not the record's own.
    if let (Some(named), Some(_)) = (fields.text, fields.bytes)
        && container.as_path() != Path::new(named)
    {
        return Ok(Some(ClaimFile::Unreadable { reason: String::from("its line of text and its line of container bytes name different paths") }));
    }
    let stage_named: bool = match container.file_name() {
        Some(name) => name.to_string_lossy().starts_with(STAGE_PREFIX),
        None => false,
    };
    let named: bool = container.is_absolute() && stage_named;
    if !named {
        return Ok(Some(ClaimFile::Unreadable { reason: String::from("it does not name a stage container") }));
    }
    return Ok(Some(ClaimFile::Named { container, owner: fields.owner }));
}

/// What:
///  One claim file as read:
/// the stage container it names,
/// refused unless it is a monochromatic-git-policy-cli stage container name,
/// and the owner token,
/// with a claim this release cannot read read as none.
/// Why:
///   The container is deleted recursively,
/// so a claim that names anything else is never acted on;
/// the owner decides whether
///  the claim may be reaped at all.
/// A claim that vanished while it was read was removed by its owner or by another command.
/// A claim whose lines are not a record this release writes is settled by the command that found it,
/// so a sweep that may only act on claims this release reads has nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = await readClaim(claimPath); // undefined when gone, unreadable, or of another shape
/// ```
pub fn read_claim(claim: &Path) -> Result<Option<(PathBuf, Option<String>)>, WorktreeCopyError> {
    return match claim_file_reading(claim)? {
        Some(ClaimFile::Named { container, owner }) => Ok(Some((container, owner))),
        Some(ClaimFile::Unreadable { .. }) | None => Ok(None),
    };
}


/// Which owners were found dead,
/// decided before anything is removed.
#[derive(Debug, Default)]
pub struct StageVerdicts {
    /// The names of the claims whose owner is dead.
    dead_claims: BTreeSet<String>,
    /// The names of the temporaries whose writer is dead.
    dead_temporaries: BTreeSet<String>,
}

impl StageVerdicts {
    /// What:
    ///  Whether the claim of this name has a dead owner.
    /// Why:
    ///   The pass that finishes an owed copy and the sweep that settles the claim act on one judgement,
    /// taken before
    ///  anything is changed,
    /// so neither acts on a claim whose owner still runs or cannot be judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.namesDeadClaim(name)
    /// ```
    pub fn names_dead_claim(&self, name: &str) -> bool {
        return self.dead_claims.contains(name);
    }

    /// What:
    ///  Judges one claim that was written again during this pass dead,
    /// because it names no owner.
    /// Why:
    ///   A claim this release could not read is settled by the command that found it,
    /// which writes it again from the records beside it;
    /// that claim names no owner,
    /// so the sweeps of the same pass must act on it as they would on any dead owner's claim.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.recordDeadClaim(name)
    /// ```
    pub fn record_dead_claim(&mut self, name: &str) {
        let _inserted: bool = self.dead_claims.insert(name.to_string());
    }

    ///  The names of the temporaries whose writer is dead.
    /// Why:
    ///   The sweep that removes them lives beside this file,
    /// and a dead writer's names are the one thing it needs from here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.deadTemporaryNames() // the names of a dead writer's temporaries
    /// ```
    pub(crate) fn dead_temporary_names(&self) -> &BTreeSet<String> {
        return &self.dead_temporaries;
    }
}

/// The suffix of the file one write is made in before its own temporary takes its name:
/// a write cut off inside it leaves this rather than a temporary the sweep already knows.
pub(crate) const SCRATCH_SUFFIX: &str = ".partial";

/// What:
///  The writer's owner token in the name of the file one write was making,
/// or nothing for a name of another shape.
/// Why:
///   A write makes `<temporary>.<random>.partial` first,
/// so the temporary's own name,
/// with the writer in it,
/// is what the sweep judges before removing it;
/// a name of another shape names no writer and counts as a dead one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const token = scratchWriterToken(name); // stem before the last dot, parsed as a temporary name
/// ```
fn scratch_writer_token(name: &str) -> Option<String> {
    let stem: &str = name.strip_suffix(SCRATCH_SUFFIX)?;
    let (temporary, _unique) = stem.rsplit_once('.')?;
    return temporary_writer_token(temporary).map(str::to_string);
}

/// What:
///  Order two listed temporaries by name the way JavaScript's default sort does.
/// Why:
///   Which temporary is judged first,
/// and so which refusal is reported first,
/// must not depend on the directory's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// temporaries.sort(([left], [right]) => compareJs(left, right))
/// ```
fn by_temporary_name(left: &(String, PathBuf), right: &(String, PathBuf)) -> std::cmp::Ordering {
    return js_order(&left.0, &right.0);
}

/// What:
///  Judges the owner of every stage claim and every temporary in the journal root,
/// and changes nothing.
/// Why:
///   A command that refuses an item of another namespace older than the patience must refuse before it changes anything,
/// so every
///  owner is judged first,
/// once (sol-1 round 3,
/// P1 C),
/// and the sweep acts on the verdicts without judging again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdicts = await judgeStageOwners(commonDir); // throws on an expired foreign item, before any removal
/// ```
pub fn judge_stage_owners(common_dir: &Path) -> Result<StageVerdicts, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let mut verdicts: StageVerdicts = StageVerdicts::default();
    for name in claim_names(root.as_path())? {
        let claim: PathBuf = root.join(name.as_str());
        let Some((_container, owner)) = read_claim(claim.as_path())? else {
            continue;
        };
        // A live owner's claim is its running add or copy; a claim whose owner cannot be judged is kept, too.
        if token_owner_is_dead(owner.as_deref(), claim.as_path())? {
            verdicts.dead_claims.insert(name);
        }
    }
    // `match` treats an absent root as having nothing to remove.
    let listing = match std::fs::read_dir(root.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(verdicts),
        Err(error) => return Err(raw_failure(&error)),
    };
    // In name order, so which temporary is judged first (and which refusal is reported first) does not depend on the directory's order.
    let mut temporaries: Vec<(String, PathBuf)> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(TEMPORARY_SUFFIX) || name.ends_with(SCRATCH_SUFFIX) {
            temporaries.push((name, entry.path()));
        }
    }
    temporaries.sort_by(by_temporary_name);
    for (name, path) in temporaries {
        // The name is `<item>.<writer>.<unique>.tmp`; a name of another shape has no writer and counts as a dead one's.
        let writer: Option<String> = match scratch_writer_token(name.as_str()) {
            Some(token) => Some(token),
            None => temporary_writer_token(name.as_str()).map(str::to_string),
        };
        if token_owner_is_dead(writer.as_deref(), path.as_path())? {
            verdicts.dead_temporaries.insert(name);
        }
    }
    return Ok(verdicts);
}

/// What:
///  Accepts a finished action,
/// or one that found its directory absent,
/// and reports every other failure.
/// Why:
///   A root that was never created holds nothing to make durable,
/// while any other refusal means the durable state is
///  unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await action(); } catch (e) { if (e.code !== 'ENOENT') throw e; }
/// ```
pub fn absent_is_fine(outcome: std::io::Result<()>) -> Result<(), WorktreeCopyError> {
    // `match` on the failure kind: only absence is accepted.
    match outcome {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes every stage container that a dead owner's claim names and no journal does,
/// then those claims,
/// and says what it
///  removed.
/// Why:
///   A claim whose owner still runs belongs to an add or a copy between claiming and journaling,
/// so it is left whole;
///  a claimed container no journal names,
/// whose owner is dead,
/// was abandoned by a process that died.
/// A stage a copy left is removed whenever it is a private directory;
/// a stage an add left is removed only when the
///  record of the object its creation made proves that object stands there and holds nothing the record cannot account
///  for (`worktree_copy_stage_made_object.rs`),
/// and is otherwise reported and kept,
/// so nothing a person added there
///  after the kill is lost and the claim goes on naming the folder.
/// The removals are published and not flushed:
/// what they settle is the wrapper's own bookkeeping of a command that died,
/// its absence costs a repeated notice and never a byte of a person's,
/// and native Git flushes nothing for the worktree add they serve,
/// while a flush of the journal root is a commit of the whole btrfs filesystem on the
///  path of every command of the repository
/// (`SPEC.md`,
/// "Local durability follows what the remote already holds").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await reapAbandonedStages(commonDir);
/// ```
pub fn reap_abandoned_stages(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    return reap_judged_stages(common_dir, &judge_stage_owners(common_dir)?, &RecoveryDestinations::EveryDestination);
}

/// `reap_abandoned_stages` on verdicts that were made before the command changed anything;
/// a claim or a temporary that the verdicts
///  do not name as dead is left whole.
pub fn reap_judged_stages(
    common_dir: &Path,
    verdicts: &StageVerdicts,
    destinations: &RecoveryDestinations<'_>,
) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let names: Vec<String> = claim_names(root.as_path())?;
    let mut notices: Vec<String> = Vec::new();
    if !names.is_empty() {
        let mut journaled: BTreeSet<PathBuf> = BTreeSet::new();
        for pending in read_pending_journals(common_dir)? {
            journaled.insert(pending.journal.stage_container);
        }
        for name in names {
            let claim: PathBuf = root.join(name.as_str());
            let Some((container, _owner)) = read_claim(claim.as_path())? else {
                continue;
            };
            // A live owner's claim is its running add or copy; a claim whose owner cannot be judged is kept, too.
            if !verdicts.dead_claims.contains(name.as_str()) {
                continue;
            }
            let present: bool = still_there(container.as_path())?;
            let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
            // A record that cannot be looked up counts as one: when in doubt the stage is kept. The record of the object a
            // stage's creation made and the completion marker say the same thing on their own, and a sweep that removed
            // the destination record of an interrupted add must not leave its stage to be taken for a copy's own: that
            // branch removes a folder without asking whether a person put anything in it. The marker is written by an
            // add and by nothing else, and a release of an earlier release removed both records before it, so a kill
            // inside that release leaves this claim with no record at all (issue 6296).
            let of_an_add: bool = stands_any_of(
                [
                    record.as_path(),
                    crate::worktree_copy_stage_made_object::made_record_beside_claim(claim.as_path()).as_path(),
                    claim.with_extension(&COMPLETE_SUFFIX[1..]).as_path(),
                ]
                .as_slice(),
            );
            let mut stage_stays: bool = false;
            // The exchange a kill interrupted left the directory Git made at the stage name, which the proof below
            // cannot account for; settling it is one empty folder and a few identity checks, never a copy, so every
            // command may do it wherever it runs. A stage name that is already clear is settled here too.
            if of_an_add
                && !journaled.contains(&container)
                && crate::worktree_copy_swapped_add_settlement::settle_before_the_stage_proof(claim.as_path(), container.as_path(), &mut notices)?
            {
                continue;
            }
            if present && !journaled.contains(&container) {
                if of_an_add {
                    // Both proofs, or nothing is removed: the record of the object the creation made names the object
                    // standing there, and that object holds nothing the record cannot account for.
                    match provenance_of(claim.as_path(), container.as_path())? {
                        Provenance::Proved => {
                            remove_stage(container.as_path())?;
                            notices.push(removed_stage_words(container.as_path()));
                        }
                        Provenance::Gone => {}
                        Provenance::Unproved(reason) => {
                            notices.push(kept_stage_words(container.as_path(), reason.as_str()));
                            stage_stays = true;
                        }
                    }
                } else {
                    // A copy's stage is a folder this account made; one another command removed in between is nobody's
                    // failure. What may be removed from it is decided by what it holds and never by its mode: the stage
                    // name holds the directory Git made after an interrupted exchange, at Git's own mode, and that
                    // folder is the wrapper's own leftover. Anything else keeps the folder and is named, because the
                    // removal is recursive and a person's file there exists nowhere else.
                    if !is_this_accounts_directory(container.as_path()) {
                        if !still_there(container.as_path())? {
                            continue;
                        }
                        notices.push(kept_copy_stage_words(container.as_path(), "it is not a folder this account made, so nothing in it was touched"));
                        stage_stays = true;
                    } else {
                        // A folder that was examined and kept says why, and the walk behind that answer is made once
                        // for a folder that does not change: the person reads the same line and no command pays it again.
                        if let Some(reason) = crate::worktree_copy_stage_intended_entries::reason_the_stage_was_kept(claim.as_path(), container.as_path()) {
                            notices.push(kept_copy_stage_words(container.as_path(), reason.as_str()));
                            continue;
                        }
                        match an_entry_the_wrappers_stage_does_not_explain(container.as_path())? {
                            Some(reason) => {
                                if let Err(error) = crate::worktree_copy_stage_intended_entries::record_the_stage_was_kept(claim.as_path(), container.as_path(), reason.as_str()) {
                                    debug(TAG, format!("the reason {} was kept could not be written ({}); the next command examines it again", container.display(), error.0).as_str());
                                }
                                notices.push(kept_copy_stage_words(container.as_path(), reason.as_str()));
                                stage_stays = true;
                            }
                            None => {
                                remove_stage(container.as_path())?;
                                notices.push(removed_copy_stage_words(container.as_path()));
                            }
                        }
                    }
                }
            }
            // A claim that still owes a copy into another worktree is that worktree's own work: the first command run
            // there makes the copy, and the pending check has already named the worktree. Nothing here settles it, so the
            // obligation outlives this command. The stage above is a partial copy nothing may resume, so its removal stays
            // open to every command: a worktree that is gone has no command left to clean up its own stage.
            if copy_owed_to_another_worktree(common_dir, claim.as_path(), record.as_path(), destinations) {
                continue;
            }
            let settled: Settlement = settle_interrupted_add(record.as_path());
            if let Some(notice) = settled.notice {
                notices.push(notice);
            }
            // While the stage stands its claim is what names it, so the record, the marker and the claim stay and the next
            //  command reports them again; a settled worktree keeps its records for the same reason.
            if settled.keep || stage_stays {
                continue;
            }
            remove_made(claim.as_path())?;
            remove_file_if_present(record.as_path())?;
            remove_file_if_present(claim.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
            remove_file_if_present(claim.as_path())?;
        }
    }
    remove_dead_temporaries(root.as_path(), verdicts)?;
    remove_records_nothing_claims(root.as_path())?;
    // A claim this command moved aside leaves the record of its destination unread by every sweep, because a record of
    // an add is read through the claim beside it; settling that record is what still names the worktree.
    notices.extend(crate::worktree_copy_unreadable_records::settle_records_without_a_claim(root.as_path())?);
    return Ok(notices);
}
