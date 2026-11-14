//! What:
//!  The durable journal of one destination's linked-worktree copy:
//!  private storage under `<common>/monochromatic-git-policy-cli-worktree-copy/v1`,
//! the atomic writes into it,
//! and the reading of what it holds.
//!  The exact JSON encoding is in `worktree_copy_journal_encoding_and_validation.rs`,
//! and the removal of a finished
//!  transaction's stage and journal is in `worktree_copy_journal_removal.rs`.
//! Why:
//!  A crash can happen after any step,
//! so the journal must say which phase the copy reached
//!  and which paths it may have touched,
//! and recovery must never act on a journal it cannot prove
//!  is its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeJournal({ path, record });
//! const pending = await readPendingWorktreeCopyJournals(commonDir);
//! ```

use crate::json_record::quote;
use crate::private_storage::{sync_directory, write_exclusive_scratch, write_exclusive_synced};
use crate::random_id::random_uuid;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_journal_encoding_and_validation::{JOURNAL_PARENT_NAME, JOURNAL_SUFFIX, JOURNAL_VERSION_NAME, Journal, JournalRefusal, PendingJournal, Phase, TEMPORARY_SUFFIX, encode_journal, parse_journal_reading};
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner, token_owner_is_dead};
use crate::worktree_copy_selection::by_js_order;
use crate::worktree_copy_stage::StagedSnapshot;
use std::path::{Path, PathBuf};

/// What:
///  The group and other permission bits a private path must not have.
/// Why:
///   A journal or stage readable or writable by others could be planted or read by another account.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NON_PRIVATE_MODE_BITS = 0o077;
/// ```
const NON_PRIVATE_MODE_BITS: u32 = 0o077;

/// What:
///  The role a private path plays,
/// which selects its expected entry kind.
/// Why:
///   A journal file must be a single-link file;
/// a directory must be a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrivateWorktreeCopyPathRole = 'journal directory' | 'journal file' | 'private stage';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    /// A journal parent or root directory.
    JournalDirectory,
    /// A journal file.
    JournalFile,
    /// A private stage container or payload.
    PrivateStage,
}

impl Role {
    /// The role's name in diagnostics.
    fn name(self) -> &'static str {
        // `match` keeps each role's diagnostic spelling next to its variant.
        match self {
            Role::JournalDirectory => return "journal directory",
            Role::JournalFile => return "journal file",
            Role::PrivateStage => return "private stage",
        }
    }
}

/// What:
///  Checks that a path is canonical,
/// owned by this account,
/// private,
/// and of the role's kind.
/// Why:
///   A journal or stage that another account can write,
/// or that a link redirects,
/// is never acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await assertPrivateWorktreeCopyPath({ path, role });
/// ```
pub fn assert_private_path(path: &Path, role: Role) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(path).map_err(raw_failure_of)?;
    let shape: bool = match role {
        Role::JournalFile => metadata.is_file() && metadata.nlink() == 1,
        Role::JournalDirectory | Role::PrivateStage => metadata.is_dir(),
    };
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    let effective_user: u32 = unsafe { libc::geteuid() };
    let canonical: bool = std::fs::canonicalize(path)
        .map_err(raw_failure_of)?
        .as_path()
        == path;
    if !shape
        || (metadata.mode() & NON_PRIVATE_MODE_BITS) != 0
        || metadata.uid() != effective_user
        || !canonical
    {
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: worktree-copy {} is unsafe: {}.",
                role.name(),
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return Ok(());
}

/// What:
///  The journal root beneath a common Git directory,
/// without creating anything.
/// Why:
///   Readers check for pending state before they decide whether to create it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'monochromatic-git-policy-cli-worktree-copy', 'v1')
/// ```
pub fn journal_root(common_dir: &Path) -> PathBuf {
    return common_dir
        .join(JOURNAL_PARENT_NAME)
        .join(JOURNAL_VERSION_NAME);
}

/// What:
///  Creates one private directory,
/// or accepts an existing one,
/// and checks it.
/// Why:
///   An interrupted creation or a second writer must not fail the other writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { mode: 0o700 }) // EEXIST accepted; then assertPrivateWorktreeCopyPath
/// ```
pub(crate) fn ensure_private_directory(path: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::DirBuilderExt;
    // `match` accepts an existing directory and reports every other creation failure.
    match std::fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(raw_failure(&error)),
    }
    return assert_private_path(path, Role::JournalDirectory);
}

/// What:
///  Creates and checks the journal parent and root,
/// and returns the root.
/// Why:
///   Every writer needs the same private location before it writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureWorktreeCopyJournalRoot(commonDir);
/// ```
pub fn ensure_journal_root(common_dir: &Path) -> Result<PathBuf, WorktreeCopyError> {
    ensure_private_directory(common_dir.join(JOURNAL_PARENT_NAME).as_path())?;
    let root: PathBuf = journal_root(common_dir);
    ensure_private_directory(root.as_path())?;
    return Ok(root);
}

/// What:
///  Writes one complete journal atomically:
/// a private exclusive temporary,
/// renamed,
/// and flushed by nobody.
/// Why:
///   A reader sees either the previous complete journal or the new complete one,
/// never a fragment.
/// What the journal guards is a copy into a worktree Git has just made,
/// whose every byte the source still holds,
/// so a power cut costs the rollback of a partial installation and not a byte of a person's;
/// native Git flushes nothing for the worktree add it serves,
/// and a flush here is a commit of the whole btrfs filesystem on the add's waiting path
/// (`SPEC.md`,
/// "The records an add writes for itself").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournal({ path, record });
/// ```
pub fn write_journal(path: &Path, journal: &Journal) -> Result<(), WorktreeCopyError> {
    return write_atomic_unsynced(path, encode_journal(journal).as_bytes(), "journal");
}

/// What:
///  The name of the temporary file a write of `path` by `owner` uses:
/// `<path>.<owner>.<unique>.tmp`.
/// Why:
///   The writer's owner is part of the temporary's name,
/// so a sweep removes only a dead writer's temporaries and can never
///  delete one a live writer is about to rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// temporaryPath('/j/abc.stage', owner, 'u1') === `/j/abc.stage.${owner.token()}.u1.tmp`;
/// ```
pub fn temporary_path(path: &Path, owner: &CopyOwner, unique: &str) -> PathBuf {
    // The suffix is appended to the path's own bytes, so a name that is not text keeps every byte of itself.
    let mut temporary: std::ffi::OsString = path.as_os_str().to_owned();
    temporary.push(format!(".{}.{unique}{TEMPORARY_SUFFIX}", owner.token()));
    return PathBuf::from(temporary);
}

/// What:
///  The writer's owner token in a temporary file's name,
/// or nothing for a name of another shape.
/// Why:
///   It is the inverse of [`temporary_path`] that the sweep of dead writers' temporaries reads.
///  A name of that shape has a token field,
///  so a stranger's name whose token field is empty names no writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writerTokenOf('abc.stage.<owner>.u1.tmp') === '<owner>';
/// ```
pub fn temporary_writer_token(file_name: &str) -> Option<&str> {
    let stem: &str = file_name.strip_suffix(TEMPORARY_SUFFIX)?;
    let (rest, _unique) = stem.rsplit_once('.')?;
    let (_item, token) = rest.rsplit_once('.')?;
    // The writer fills each field, so an empty token field is a stranger's file that must name no writer.
    if token.is_empty() {
        return None;
    }
    return Some(token);
}

/// What:
///  Writes one private file atomically beside its final name;
/// a failure names `subject` and leaves no temporary.
/// Why:
///   Journals and stage claims share one rule:
/// a reader sees a complete file or none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomically({ path, bytes, subject });
/// ```
pub fn write_atomic(path: &Path, bytes: &[u8], subject: &str) -> Result<(), WorktreeCopyError> {
    return write_atomic_flushing(path, bytes, subject, Flush::ToDisk);
}

/// Whether a record written by [`write_atomic_flushing`] is flushed to disk before the call returns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flush {
    /// The file and its directory are flushed:
    /// a crash finds the record.
    ToDisk,
    /// Neither is flushed:
    /// the file is published whole by the rename,
    /// and the filesystem makes it durable with its next commit.
    Never,
}

/// What:
///  Writes one private file atomically beside its final name without flushing the file or its directory to disk;
/// a failure
///  names `subject` and leaves no temporary.
/// Why:
///   A flush after a subvolume was swapped into place in the same btrfs transaction is a commit of the whole filesystem (the
///  kernel sets `btrfs_set_log_full_commit` for the swap),
/// which waits for every other writer's data.
/// A record that only
///  silences a notice is published whole by the rename and made durable by the commit the filesystem makes anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomicallyUnflushed({ path, bytes, subject });
/// ```
pub fn write_atomic_unsynced(path: &Path, bytes: &[u8], subject: &str) -> Result<(), WorktreeCopyError> {
    return write_atomic_flushing(path, bytes, subject, Flush::Never);
}

/// `write_atomic` and `write_atomic_unsynced`:
/// one rename publishes the whole file;
/// `flush` says whether the file and its
///  directory are flushed.
fn write_atomic_flushing(path: &Path, bytes: &[u8], subject: &str, flush: Flush) -> Result<(), WorktreeCopyError> {
    let unique: String = match random_uuid() {
        Ok(made) => made,
        Err(error) => {
            return Err(failure_with_cause(
                format!(
                    "monochromatic-git-policy-cli: could not persist worktree-copy {subject} {}.",
                    quote(path.to_string_lossy().as_ref())
                )
                .as_str(),
                Some(&error.0),
            ));
        }
    };
    let temporary: PathBuf = temporary_path(path, &current_copy_owner()?, unique.as_str());
    let written: std::io::Result<()> = match flush {
        Flush::ToDisk => write_exclusive_synced(temporary.as_path(), bytes),
        Flush::Never => write_exclusive_scratch(temporary.as_path(), bytes),
    };
    let persisted: std::io::Result<()> = match written {
        Ok(()) => publish_written(temporary.as_path(), path, flush),
        Err(error) => Err(error),
    };
    // A failed write removes its own temporary, so the directory keeps no partial journal.
    if let Err(error) = persisted {
        let _ignored_removal: std::io::Result<()> = std::fs::remove_file(temporary.as_path());
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: could not persist worktree-copy {subject} {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            Some(&error),
        ));
    }
    return Ok(());
}

/// What:
///  Creates the staged journal of one destination,
/// durable before any destination is touched.
/// Why:
///   Recovery needs the stage and destination named before installation starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createWorktreeCopyJournal({ commonDir, destinationRoot, snapshot });
/// ```
pub fn create_journal(
    common_dir: &Path,
    destination_root: &Path,
    snapshot: &StagedSnapshot,
) -> Result<PendingJournal, WorktreeCopyError> {
    return create_journal_for(common_dir, destination_root, snapshot, &current_copy_owner()?);
}

/// What:
///  Creates the staged journal of one destination for a named owner,
/// durable before any destination is touched.
/// Why:
///   The journal's name carries its owner;
/// a control or a hand-over can name an owner other than the running process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createWorktreeCopyJournalFor({ commonDir, destinationRoot, snapshot, owner });
/// ```
pub fn create_journal_for(
    common_dir: &Path,
    destination_root: &Path,
    snapshot: &StagedSnapshot,
    owner: &CopyOwner,
) -> Result<PendingJournal, WorktreeCopyError> {
    let root: PathBuf = ensure_journal_root(common_dir)?;
    let unique: String = match random_uuid() {
        Ok(made) => made,
        Err(error) => return Err(raw_failure(&error.0)),
    };
    let path: PathBuf = root.join(format!("{unique}.{}{JOURNAL_SUFFIX}", owner.token()));
    let journal: Journal = Journal {
        created_entries: Vec::new(),
        intended_entries: Vec::new(),
        destination_root: destination_root.to_path_buf(),
        phase: Phase::Staged,
        selected_roots: snapshot.selected_roots.clone(),
        source_root: snapshot.source_root.clone(),
        stage_container: snapshot.stage_container.clone(),
        stage_root: snapshot.stage_root.clone(),
    };
    write_journal(path.as_path(), &journal)?;
    return Ok(PendingJournal { path, journal });
}

/// What:
///  Whether the journal root holds any item at all.
/// Why:
///   Every forwarded command asks whether there is recovery work before it does any,
/// and a temporary of a write a
///  killed command never renamed is work too:
/// a predicate that looked only for journals,
/// claims and unfilled records let those
///  temporaries stay for as long as the repository lived.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending = (await readdir(root)).length > 0;
/// ```
pub fn journal_items_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    // `match` treats an absent root as holding nothing, and every other failure as fatal.
    return match std::fs::read_dir(journal_root(common_dir).as_path()) {
        Ok(mut listing) => Ok(listing.any(|listed| {
            // A record the command that found it moved aside is not work any later command does: the folder that keeps
            // those records stands until the repository is gone, and a look at it would make every command pay for a
            // sweep they have nothing to do.
            return match listed {
                Ok(entry) => entry.file_name() != std::ffi::OsStr::new(crate::worktree_copy_unreadable_records::UNREADABLE_FOLDER),
                Err(_unreadable) => true,
            };
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Takes over a journal whose owner is dead,
/// by renaming it to a name that carries the new owner.
/// Why:
///   One rename is the whole claim:
/// a command that loses the race finds the old name gone and leaves the journal to the
///  winner,
/// so two commands never install one journal.
/// This is the lock-free single winner for recovery;
/// no file of a lock
///  exists,
/// none is waited for and none can go stale (if the winner dies,
/// the journal's name shows a dead owner again).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = await adoptJournal({ pending, owner }); // undefined when another command won
/// ```
pub fn adopt_journal(pending: &PendingJournal, owner: &CopyOwner) -> Result<Option<PendingJournal>, WorktreeCopyError> {
    let Some(parent) = pending.path.parent() else {
        return Ok(None);
    };
    let name: String = pending.path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let identifier: &str = name.split('.').next().unwrap_or_default();
    let adopted: PathBuf = parent.join(format!("{identifier}.{}{JOURNAL_SUFFIX}", owner.token()));
    // `match` on the rename: a vanished source means another command took the journal first; any other failure is real.
    // The rename is the whole claim and nothing waits to make it durable: a journal that a power cut returns to its
    // dead owner's name is taken over again by the next command, and nothing of a person's depends on it.
    match std::fs::rename(pending.path.as_path(), adopted.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    return Ok(Some(PendingJournal { path: adopted, journal: pending.journal.clone() }));
}

/// What:
///  Whether the journal's owner is dead,
/// so any command may take it over.
/// Why:
///   A journal with a live owner belongs to a copy that is still running;
/// recovering it would install it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalOwnerIsDead(pending);
/// ```
pub fn journal_owner_is_dead(pending: &PendingJournal) -> Result<bool, WorktreeCopyError> {
    return token_owner_is_dead(pending.owner_token().as_deref(), pending.path.as_path());
}

/// What:
///  Rename a written temporary file over `path`,
/// and make the directory durable when the flush mode asks.
/// Why:
///   The rename publishes the journal in one step,
/// and the sync makes the publication survive a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renameSync(temporary, path); if (flush === 'toDisk') syncDirectory(dirname(path));
/// ```
fn publish_written(temporary: &Path, path: &Path, flush: Flush) -> std::io::Result<()> {
    std::fs::rename(temporary, path)?;
    return match (flush, path.parent()) {
        (Flush::ToDisk, Some(parent)) => sync_directory(parent),
        _ => Ok(()),
    };
}

/// What:
///  One journal file as a listing found it:
/// a record this release reads,
/// or one it cannot read.
/// Why:
///   The record a copy writes is published whole,
/// so a file at a journal's name that does not parse is the mark of a write a crash cut short;
/// the command that found it settles it
/// (`worktree_copy_unreadable_records`)
/// and the commands that only need the readable journals never see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JournalFile = { kind: 'pending'; pending: PendingJournal } | { kind: 'unreadable'; path: string; reason: string };
/// ```
pub enum JournalFile {
    /// A journal this release reads.
    Pending(PendingJournal),
    /// A file at a journal's name whose bytes are not the record this release writes.
    Unreadable {
        /// The file.
        path: PathBuf,
        /// What one line says about why it does not read.
        reason: String,
    },
}

/// What:
///  Reads one journal a listing named,
/// or nothing when the file is gone by the time it is read.
/// Why:
///   Another command finishes or takes over a journal at any moment,
/// which is no fault of this reader.
/// A file whose bytes are not the record this release writes is reported as unreadable rather than
///  refused,
/// because the record a copy writes is published whole and a file that does not read is what a crash leaves;
/// a file another account owns,
/// or a link or a pipe at the name,
/// stays a refusal,
/// because no crash writes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = await readListedJournal(root, name); // undefined when it vanished
/// ```
fn read_listed_journal(root: &Path, name: &str) -> Result<Option<JournalFile>, WorktreeCopyError> {
    let path: PathBuf = root.join(name);
    // `match` on the lookup: a journal another command settled is nobody's fault, and every other refusal is a fault.
    match std::fs::symlink_metadata(path.as_path()) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(path.as_path(), Role::JournalFile)?;
    let bytes: Vec<u8> = match std::fs::read(path.as_path()) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read worktree-copy journal {}.",
                quote(path.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    // A journal is text this wrapper wrote; bytes that are not UTF-8 are damage, and no path is read from them.
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(Some(JournalFile::Unreadable { path, reason: String::from("its bytes are not text") }));
    };
    // `match` on the parse: damage is settled by its finder, and a record whose paths no writer writes stays a refusal.
    return match parse_journal_reading(text.as_str(), path.as_path(), root) {
        Ok(journal) => Ok(Some(JournalFile::Pending(PendingJournal { path, journal }))),
        Err(JournalRefusal::Unsafe(error)) => Err(error),
        Err(refusal) => Ok(Some(JournalFile::Unreadable { path, reason: refusal.reason() })),
    };
}

/// What:
///  Lists the journal files of a common directory,
/// validated,
/// in journal-name order,
/// with the ones this release cannot read named.
/// Why:
///   The settle pass moves an unreadable journal aside;
/// every other reader asks only for the journals it can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readListedWorktreeCopyJournals(commonDir);
/// ```
pub fn read_listed_journals(common_dir: &Path) -> Result<Vec<JournalFile>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    // `match` treats an absent root as having no journals, and every other failure as fatal.
    match std::fs::symlink_metadata(root.as_path()) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(
        root.parent().unwrap_or(root.as_path()),
        Role::JournalDirectory,
    )?;
    assert_private_path(root.as_path(), Role::JournalDirectory)?;
    let mut names: Vec<String> = Vec::new();
    let listing = match std::fs::read_dir(root.as_path()) {
        Ok(found) => found,
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not inspect worktree-copy journals under {}.",
                quote(root.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(JOURNAL_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(by_js_order);
    let mut journals: Vec<JournalFile> = Vec::new();
    for name in names {
        if let Some(found) = read_listed_journal(root.as_path(), name.as_str())? {
            journals.push(found);
        }
    }
    return Ok(journals);
}

/// What:
///  Lists the journal files of a common directory,
/// validated,
/// in journal-name order.
/// Why:
///   Recovery and the pending check read the same validated set.
/// An unreadable journal is not among them:
///  the command that found it settles it first,
/// and a reader that meets one anyway has nothing it may act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readPendingWorktreeCopyJournals(commonDir);
/// ```
pub fn read_pending_journals(common_dir: &Path) -> Result<Vec<PendingJournal>, WorktreeCopyError> {
    let mut pending: Vec<PendingJournal> = Vec::new();
    for journal in read_listed_journals(common_dir)? {
        if let JournalFile::Pending(found) = journal {
            pending.push(found);
        }
    }
    return Ok(pending);
}

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_journal_tests.rs"]
mod tests;
