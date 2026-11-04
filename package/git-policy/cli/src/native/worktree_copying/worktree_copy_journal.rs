//! What:
//!  The durable journal of one destination's linked-worktree copy:
//!  its exact JSON encoding, strict validation of every field recovery will act on,
//!  private storage under `<common>/cli-git-worktree-copy/v1`,
//!  and removal of a finished transaction's stage and journal.
//! Why:
//!  A crash can happen after any step, so the journal must say which phase the copy reached
//!  and which paths it may have touched, and recovery must never act on a journal it cannot prove
//!  is its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeJournal({ path, record });
//! const pending = await readPendingWorktreeCopyJournals(commonDir);
//! ```

use crate::scoped_start::start_or_run;
use crate::json_record::quote;
use crate::parallel_budget::{HostResources, sliced_workers};
use crate::private_storage::{sync_directory, write_exclusive_scratch, write_exclusive_synced};
use crate::random_id::random_uuid;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner, token_owner_is_dead};
use crate::worktree_copy_selection::{STAGE_PREFIX, assert_safe_repository_path, js_order};
use crate::worktree_copy_stage::{PARALLEL_COPY_MAX_WORKERS, StagedSnapshot};
use std::path::{Path, PathBuf};

/// What:
///  The parent directory name of the journals, beneath the common Git directory.
/// Why:
///   Every linked worktree of one repository shares the same journal location.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_PARENT_NAME = 'cli-git-worktree-copy';
/// ```
pub const JOURNAL_PARENT_NAME: &str = "cli-git-worktree-copy";

/// What:
///  The schema directory of the journals.
/// Why:
///   A later schema gets its own directory, so an older wrapper never reads a newer record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_VERSION_NAME = 'v1';
/// ```
pub const JOURNAL_VERSION_NAME: &str = "v1";

/// What:
///  The suffix of a final journal file.
/// Why:
///   Only files with this suffix are journals; anything else in the directory is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_SUFFIX = '.json';
/// ```
const JOURNAL_SUFFIX: &str = ".json";

/// What:
///  The suffix of an unpublished temporary journal write.
/// Why:
///   An interrupted write leaves only files with this suffix, which recovery removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_SUFFIX = '.tmp';
/// ```
pub const TEMPORARY_SUFFIX: &str = ".tmp";

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
///  The durable phase a copy has reached.
/// Why:
///   Recovery resumes an installation and finishes a completed cleanup differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyJournalPhase = 'complete' | 'installing' | 'staged';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// The stage is complete and nothing has been installed.
    Staged,
    /// Installation has begun; the intents and creations in the install log say what it touched.
    Installing,
    /// Installation finished; only cleanup remains.
    Complete,
}

impl Phase {
    /// The phase's name as the incumbent writes it.
    fn as_str(self) -> &'static str {
        // `match` keeps each phase's spelling next to its variant.
        match self {
            Phase::Staged => return "staged",
            Phase::Installing => return "installing",
            Phase::Complete => return "complete",
        }
    }
}

/// What:
///  One durable journal record.
/// Why:
///   Every field is read back by recovery before any filesystem path is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyJournal = { createdEntries; intendedEntries; destinationRoot; phase; selectedRoots; sourceRoot; stageContainer; stageRoot; version: 1 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Journal {
    /// Paths proven created, with their post-creation identities.
    pub created_entries: Vec<InstalledPath>,
    /// Selected paths claimed before installation.
    pub intended_entries: Vec<String>,
    /// The worktree receiving the copy.
    pub destination_root: String,
    /// The durable phase.
    pub phase: Phase,
    /// The ignored roots the staged payload represents.
    pub selected_roots: Vec<String>,
    /// The source worktree.
    pub source_root: String,
    /// The private container beside the destination.
    pub stage_container: String,
    /// The payload root inside the container.
    pub stage_root: String,
}

/// What:
///  A journal and the path it was read from or written to.
/// Why:
///   Removal and recovery need both the record and the file that must be deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingWorktreeCopyJournal = { path: string; record: WorktreeCopyJournal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingJournal {
    /// Absolute journal file path.
    pub path: PathBuf,
    /// The validated record.
    pub journal: Journal,
}

impl PendingJournal {
    /// The owner token the journal's file name carries, or nothing for a name no owner is written into.
    pub fn owner_token(&self) -> Option<String> {
        return owner_token_of(self.path.as_path());
    }
}

/// What:
///  The owner token in a journal file name `<id>.<owner>.json`, if the name has that shape.
/// Why:
///   The owner is part of the name so that taking a journal over is one atomic rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerTokenOf('0a1b.2c3d.json') === '2c3d';
/// ```
pub fn owner_token_of(path: &Path) -> Option<String> {
    let name: String = path.file_name()?.to_string_lossy().into_owned();
    let stem: &str = name.strip_suffix(JOURNAL_SUFFIX)?;
    let (_identifier, token) = stem.rsplit_once('.')?;
    return Some(token.to_string());
}

/// What:
///  The JSON text of one journal, with the incumbent's key order and a final line feed.
/// Why:
///   The incumbent reads these bytes, so the order and spelling must match its `JSON.stringify` output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify(serializedRecord)}\n`
/// ```
pub fn encode_journal(journal: &Journal) -> String {
    let created: Vec<String> = journal
        .created_entries
        .iter()
        .map(encode_installed_entry)
        .collect();
    return format!(
        "{{\"createdEntries\":[{}],\"intendedEntries\":{},\"destinationRoot\":{},\"phase\":{},\"selectedRoots\":{},\"sourceRoot\":{},\"stageContainer\":{},\"stageRoot\":{},\"version\":1}}\n",
        created.join(","),
        string_array(journal.intended_entries.as_slice()),
        quote(journal.destination_root.as_str()),
        quote(journal.phase.as_str()),
        string_array(journal.selected_roots.as_slice()),
        quote(journal.source_root.as_str()),
        quote(journal.stage_container.as_str()),
        quote(journal.stage_root.as_str()),
    );
}

/// What:
///  One created entry as its JSON object, with the incumbent's key order.
/// Why:
///   The install log and the journal share this spelling, so both wrappers read either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ device, inode, relativePath, selected })
/// ```
pub fn encode_installed_entry(entry: &InstalledPath) -> String {
    return format!(
        "{{\"device\":\"{}\",\"inode\":\"{}\",\"relativePath\":{},\"selected\":{}}}",
        entry.device,
        entry.inode,
        quote(entry.relative_path.as_str()),
        entry.selected
    );
}

/// What:
///  A JSON array of strings, each quoted as `JSON.stringify` quotes it.
/// Why:
///   Repository paths are the only arrays a journal holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(items)
/// ```
pub fn string_array(items: &[String]) -> String {
    let quoted: Vec<String> = items
        .iter()
        .map(|item| return quote(item.as_str()))
        .collect();
    return format!("[{}]", quoted.join(","));
}

/// What:
///  Whether a path is absolute and already in lexical canonical form.
/// Why:
///   A journal path that is not canonical could name a different file than the one validated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAbsolute(value) && resolve(value) === value
/// ```
fn is_canonical_absolute(value: &str) -> bool {
    if !value.starts_with('/') {
        return false;
    }
    if value == "/" {
        return true;
    }
    return value[1..]
        .split('/')
        .all(|component| return !component.is_empty() && component != "." && component != "..");
}

/// What:
///  Whether a text is a canonical decimal identity: digits only, no leading zero.
/// Why:
///   Device and inode numbers are compared as text, so only one spelling of each may exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isDecimalIdentity(value: unknown): value is string;
/// ```
fn decimal_identity(value: &serde_json::Value) -> Option<u64> {
    let text: &str = value.as_str()?;
    if text.is_empty() || (text.len() > 1 && text.starts_with('0')) {
        return None;
    }
    if !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return None;
    }
    return text.parse::<u64>().ok();
}

/// What:
///  One created-entry object, read strictly.
/// Why:
///   An identity with any wrong type is not an identity cli-git wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isInstalledEntry(value: unknown): value is InstalledWorktreePath;
/// ```
pub fn installed_entry(value: &serde_json::Value) -> Option<InstalledPath> {
    let object = value.as_object()?;
    return Some(InstalledPath {
        device: decimal_identity(object.get("device")?)?,
        inode: decimal_identity(object.get("inode")?)?,
        relative_path: object.get("relativePath")?.as_str()?.to_string(),
        selected: object.get("selected")?.as_bool()?,
    });
}

/// What:
///  An array of strings, read strictly.
/// Why:
///   A non-string path would reach the filesystem without the validation a string gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isStringArray(value)
/// ```
fn string_list(value: Option<&serde_json::Value>) -> Option<Vec<String>> {
    let array = value?.as_array()?;
    return array
        .iter()
        .map(|item| return item.as_str().map(str::to_string))
        .collect();
}

/// What:
///  The record's fields read strictly, before any path is checked.
/// Why:
///   The shape check refuses every record that a later check would have to guess about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(isRecord(value) && value.version === 1 && ...)) throw new WorktreeCopyError('... is corrupt ...');
/// ```
fn read_shape(value: &serde_json::Value) -> Option<Journal> {
    let object = value.as_object()?;
    if object.get("version")?.as_f64()? != 1.0 {
        return None;
    }
    let phase: Phase = match object.get("phase")?.as_str()? {
        "staged" => Phase::Staged,
        "installing" => Phase::Installing,
        "complete" => Phase::Complete,
        _ => return None,
    };
    let created_entries: Vec<InstalledPath> = object
        .get("createdEntries")?
        .as_array()?
        .iter()
        .map(installed_entry)
        .collect::<Option<Vec<InstalledPath>>>()?;
    return Some(Journal {
        created_entries,
        intended_entries: string_list(object.get("intendedEntries"))?,
        destination_root: object.get("destinationRoot")?.as_str()?.to_string(),
        phase,
        selected_roots: string_list(object.get("selectedRoots"))?,
        source_root: object.get("sourceRoot")?.as_str()?.to_string(),
        stage_container: object.get("stageContainer")?.as_str()?.to_string(),
        stage_root: object.get("stageRoot")?.as_str()?.to_string(),
    });
}

/// What:
///  Parses and validates one journal's text, refusing every shape recovery cannot trust.
/// Why:
///   Recovery turns journal paths into deletions, so every relation between the paths is checked first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// validateJournalValue({ value: JSON.parse(text), path, journalRoot });
/// ```
pub fn parse_journal(
    text: &str,
    path: &Path,
    journal_root: &Path,
) -> Result<Journal, WorktreeCopyError> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|error| {
        let message: String = format!(
            "cli-git: could not read worktree-copy journal {}.",
            quote(path.to_string_lossy().as_ref())
        );
        return failure_with_cause(message.as_str(), Some(&error));
    })?;
    let Some(journal) = read_shape(&value) else {
        return Err(corrupt(path));
    };
    if path.parent() != Some(journal_root) {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy journal escaped private root: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    for (field, field_value) in [
        ("destinationRoot", journal.destination_root.as_str()),
        ("sourceRoot", journal.source_root.as_str()),
        ("stageContainer", journal.stage_container.as_str()),
        ("stageRoot", journal.stage_root.as_str()),
    ] {
        if !is_canonical_absolute(field_value) {
            return Err(failure_with_cause(
                format!(
                    "cli-git: worktree-copy journal has unsafe {field}: {}.",
                    quote(field_value)
                )
                .as_str(),
                None,
            ));
        }
    }
    let stage_relation: bool = Path::new(journal.stage_container.as_str()).parent()
        == Path::new(journal.destination_root.as_str()).parent()
        && Path::new(journal.stage_container.as_str())
            .file_name()
            .is_some_and(|name| return name.to_string_lossy().starts_with(STAGE_PREFIX))
        && journal.stage_root == format!("{}/payload", journal.stage_container);
    if !stage_relation {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy journal has unsafe private stage relation: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    for selected in &journal.selected_roots {
        assert_safe_repository_path(selected)?;
    }
    for intended in &journal.intended_entries {
        assert_safe_repository_path(intended)?;
    }
    for created in &journal.created_entries {
        assert_safe_repository_path(created.relative_path.as_str())?;
    }
    let created_paths: Vec<&str> = journal
        .created_entries
        .iter()
        .map(|entry| return entry.relative_path.as_str())
        .collect();
    let duplicated: bool = has_duplicates(journal.selected_roots.as_slice())
        || has_duplicates(journal.intended_entries.as_slice())
        || has_duplicates(created_paths.as_slice());
    if duplicated {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy journal contains duplicate repository paths: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return Ok(journal);
}

/// What:
///  Whether a list holds the same entry twice.
/// Why:
///   Duplicate claims would make rollback ownership ambiguous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Set(items).size !== items.length
/// ```
fn has_duplicates<T: AsRef<str>>(items: &[T]) -> bool {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    return items.iter().any(|item| return !seen.insert(item.as_ref()));
}

/// What:
///  The corruption failure naming one journal.
/// Why:
///   The user learns which file to inspect; the file itself is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`cli-git: worktree-copy journal is corrupt: ${JSON.stringify(path)}.`)
/// ```
fn corrupt(path: &Path) -> WorktreeCopyError {
    return failure_with_cause(
        format!(
            "cli-git: worktree-copy journal is corrupt: {}.",
            quote(path.to_string_lossy().as_ref())
        )
        .as_str(),
        None,
    );
}

/// What:
///  The role a private path plays, which selects its expected entry kind.
/// Why:
///   A journal file must be a single-link file; a directory must be a directory.
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
///  Checks that a path is canonical, owned by this account, private, and of the role's kind.
/// Why:
///   A journal or stage that another account can write, or that a link redirects, is never acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await assertPrivateWorktreeCopyPath({ path, role });
/// ```
pub fn assert_private_path(path: &Path, role: Role) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(path).map_err(|error| return raw_failure(&error))?;
    let shape: bool = match role {
        Role::JournalFile => metadata.is_file() && metadata.nlink() == 1,
        Role::JournalDirectory | Role::PrivateStage => metadata.is_dir(),
    };
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    let effective_user: u32 = unsafe { libc::geteuid() };
    let canonical: bool = std::fs::canonicalize(path)
        .map_err(|error| return raw_failure(&error))?
        .as_path()
        == path;
    if !shape
        || (metadata.mode() & NON_PRIVATE_MODE_BITS) != 0
        || metadata.uid() != effective_user
        || !canonical
    {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy {} is unsafe: {}.",
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
///  The journal root beneath a common Git directory, without creating anything.
/// Why:
///   Readers check for pending state before they decide whether to create it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'cli-git-worktree-copy', 'v1')
/// ```
pub fn journal_root(common_dir: &Path) -> PathBuf {
    return common_dir
        .join(JOURNAL_PARENT_NAME)
        .join(JOURNAL_VERSION_NAME);
}

/// What:
///  Creates one private directory, or accepts an existing one, and checks it.
/// Why:
///   An interrupted creation or a second writer must not fail the other writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { mode: 0o700 }) // EEXIST accepted; then assertPrivateWorktreeCopyPath
/// ```
fn ensure_private_directory(path: &Path) -> Result<(), WorktreeCopyError> {
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
///  Creates and checks the journal parent and root, and returns the root.
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
///  Writes one complete journal atomically: a private exclusive temporary, synced, renamed, and its directory synced.
/// Why:
///   A reader sees either the previous complete journal or the new complete one, never a fragment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournal({ path, record });
/// ```
pub fn write_journal(path: &Path, journal: &Journal) -> Result<(), WorktreeCopyError> {
    return write_atomic(path, encode_journal(journal).as_bytes(), "journal");
}

/// What:
///  The name of the temporary file a write of `path` by `owner` uses: `<path>.<owner>.<unique>.tmp`.
/// Why:
///   The writer's owner is part of the temporary's name, so a sweep removes only a dead writer's temporaries and can never
///  delete one a live writer is about to rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// temporaryPath('/j/abc.stage', owner, 'u1') === `/j/abc.stage.${owner.token()}.u1.tmp`;
/// ```
pub fn temporary_path(path: &Path, owner: &CopyOwner, unique: &str) -> PathBuf {
    return PathBuf::from(format!("{}.{}.{unique}{TEMPORARY_SUFFIX}", path.to_string_lossy(), owner.token()));
}

/// What:
///  The writer's owner token in a temporary file's name, or nothing for a name of another shape.
/// Why:
///   It is the inverse of [`temporary_path`] that the sweep of dead writers' temporaries reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writerTokenOf('abc.stage.<owner>.u1.tmp') === '<owner>';
/// ```
pub fn temporary_writer_token(file_name: &str) -> Option<&str> {
    let stem: &str = file_name.strip_suffix(TEMPORARY_SUFFIX)?;
    let (rest, _unique) = stem.rsplit_once('.')?;
    let (_item, token) = rest.rsplit_once('.')?;
    return Some(token);
}

/// What:
///  Writes one private file atomically beside its final name; a failure names `subject` and leaves no temporary.
/// Why:
///   Journals and stage claims share one rule: a reader sees a complete file or none.
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
    /// The file and its directory are flushed: a crash finds the record.
    ToDisk,
    /// Neither is flushed: the file is published whole by the rename, and the filesystem makes it durable with its next commit.
    Never,
}

/// What:
///  Writes one private file atomically beside its final name without flushing the file or its directory to disk; a failure
///  names `subject` and leaves no temporary.
/// Why:
///   A flush after a subvolume was swapped into place in the same btrfs transaction is a commit of the whole filesystem (the
///  kernel sets `btrfs_set_log_full_commit` for the swap), which waits for every other writer's data. A record that only
///  silences a notice is published whole by the rename and made durable by the commit the filesystem makes anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomicallyUnflushed({ path, bytes, subject });
/// ```
pub fn write_atomic_unsynced(path: &Path, bytes: &[u8], subject: &str) -> Result<(), WorktreeCopyError> {
    return write_atomic_flushing(path, bytes, subject, Flush::Never);
}

/// `write_atomic` and `write_atomic_unsynced`: one rename publishes the whole file; `flush` says whether the file and its
///  directory are flushed.
fn write_atomic_flushing(path: &Path, bytes: &[u8], subject: &str, flush: Flush) -> Result<(), WorktreeCopyError> {
    let unique: String = random_uuid().map_err(|error| {
        return failure_with_cause(
            format!(
                "cli-git: could not persist worktree-copy {subject} {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            Some(&error.0),
        );
    })?;
    let temporary: PathBuf = temporary_path(path, &current_copy_owner()?, unique.as_str());
    let written: std::io::Result<()> = match flush {
        Flush::ToDisk => write_exclusive_synced(temporary.as_path(), bytes),
        Flush::Never => write_exclusive_scratch(temporary.as_path(), bytes),
    };
    let persisted: std::io::Result<()> =
        written
            .and_then(|()| return std::fs::rename(temporary.as_path(), path))
            .and_then(|()| match (flush, path.parent()) {
                (Flush::ToDisk, Some(parent)) => return sync_directory(parent),
                _ => return Ok(()),
            });
    // A failed write removes its own temporary, so the directory keeps no partial journal.
    if let Err(error) = persisted {
        let _ignored_removal: std::io::Result<()> = std::fs::remove_file(temporary.as_path());
        return Err(failure_with_cause(
            format!(
                "cli-git: could not persist worktree-copy {subject} {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            Some(&error),
        ));
    }
    return Ok(());
}

/// What:
///  Creates the staged journal of one destination, durable before any destination is touched.
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
///  Creates the staged journal of one destination for a named owner, durable before any destination is touched.
/// Why:
///   The journal's name carries its owner; a control or a hand-over can name an owner other than the running process.
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
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    let path: PathBuf = root.join(format!("{unique}.{}{JOURNAL_SUFFIX}", owner.token()));
    let journal: Journal = Journal {
        created_entries: Vec::new(),
        intended_entries: Vec::new(),
        destination_root: destination_root.to_string_lossy().into_owned(),
        phase: Phase::Staged,
        selected_roots: snapshot.selected_roots.clone(),
        source_root: snapshot.source_root.to_string_lossy().into_owned(),
        stage_container: snapshot.stage_container.to_string_lossy().into_owned(),
        stage_root: snapshot.stage_root.to_string_lossy().into_owned(),
    };
    write_journal(path.as_path(), &journal)?;
    return Ok(PendingJournal { path, journal });
}

/// What:
///  Takes over a journal whose owner is dead, by renaming it to a name that carries the new owner.
/// Why:
///   One rename is the whole claim: a command that loses the race finds the old name gone and leaves the journal to the
///  winner, so two commands never install one journal. This is the lock-free single winner for recovery; no file of a lock
///  exists, none is waited for and none can go stale (if the winner dies, the journal's name shows a dead owner again).
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
    match std::fs::rename(pending.path.as_path(), adopted.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    sync_directory(parent).map_err(|error| return raw_failure(&error))?;
    return Ok(Some(PendingJournal { path: adopted, journal: pending.journal.clone() }));
}

/// What:
///  Whether the journal's owner is dead, so any command may take it over.
/// Why:
///   A journal with a live owner belongs to a copy that is still running; recovering it would install it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalOwnerIsDead(pending);
/// ```
pub fn journal_owner_is_dead(pending: &PendingJournal) -> Result<bool, WorktreeCopyError> {
    return token_owner_is_dead(pending.owner_token().as_deref(), pending.path.as_path());
}

/// What:
///  Reads one journal a listing named, or nothing when the file is gone by the time it is read.
/// Why:
///   Another command finishes or takes over a journal at any moment, which is no fault of this reader; every other failure,
///  a corrupt or unsafe journal included, is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending = await readListedJournal(root, name); // undefined when it vanished
/// ```
fn read_listed_journal(root: &Path, name: &str) -> Result<Option<PendingJournal>, WorktreeCopyError> {
    let path: PathBuf = root.join(name);
    let loaded: Result<Journal, WorktreeCopyError> = (|| {
        assert_private_path(path.as_path(), Role::JournalFile)?;
        let text: String = read_journal_text(path.as_path())?;
        return parse_journal(text.as_str(), path.as_path(), root);
    })();
    // `match` on the load: a vanished journal is skipped; any other failure is reported.
    match loaded {
        Ok(journal) => return Ok(Some(PendingJournal { path, journal })),
        Err(error) => {
            let vanished: bool = matches!(std::fs::symlink_metadata(path.as_path()), Err(ref lookup) if lookup.kind() == std::io::ErrorKind::NotFound);
            if vanished {
                return Ok(None);
            }
            return Err(error);
        }
    }
}

/// What:
///  Lists the journal files of a common directory, validated, in journal-name order.
/// Why:
///   Recovery and the pending check read the same validated set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readPendingWorktreeCopyJournals(commonDir);
/// ```
pub fn read_pending_journals(common_dir: &Path) -> Result<Vec<PendingJournal>, WorktreeCopyError> {
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
    let listing = std::fs::read_dir(root.as_path()).map_err(|error| {
        let message: String = format!(
            "cli-git: could not inspect worktree-copy journals under {}.",
            quote(root.to_string_lossy().as_ref())
        );
        return failure_with_cause(message.as_str(), Some(&error));
    })?;
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(JOURNAL_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| return js_order(left, right));
    let mut pending: Vec<PendingJournal> = Vec::new();
    for name in names {
        if let Some(found) = read_listed_journal(root.as_path(), name.as_str())? {
            pending.push(found);
        }
    }
    return Ok(pending);
}

/// What:
///  The text of one journal file, read as UTF-8 with replacement, as the incumbent reads it.
/// Why:
///   An unreadable journal is named, not skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(path, 'utf8')
/// ```
fn read_journal_text(path: &Path) -> Result<String, WorktreeCopyError> {
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| {
        let message: String = format!(
            "cli-git: could not read worktree-copy journal {}.",
            quote(path.to_string_lossy().as_ref())
        );
        return failure_with_cause(message.as_str(), Some(&error));
    })?;
    return Ok(String::from_utf8_lossy(bytes.as_slice()).into_owned());
}

/// What:
///  Makes every directory of a stage owner-accessible before its entries are deleted.
/// Why:
///   An installed directory can carry a mode that forbids its own removal; only the stage's own directories are changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await prepareStageRemoval(stageContainer);
/// ```
pub fn prepare_stage_removal(stage_container: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::PermissionsExt;
    let mut pending: Vec<PathBuf> = vec![stage_container.to_path_buf()];
    while let Some(directory) = pending.pop() {
        // `match` skips a directory already removed and refuses anything else that cannot be inspected.
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(directory.as_path()) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        if !metadata.is_dir() {
            continue;
        }
        std::fs::set_permissions(directory.as_path(), std::fs::Permissions::from_mode(0o700))
            .map_err(|error| return raw_failure(&error))?;
        for listed in
            std::fs::read_dir(directory.as_path()).map_err(|error| return raw_failure(&error))?
        {
            let child: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
            let child_type: std::fs::FileType = child
                .file_type()
                .map_err(|error| return raw_failure(&error))?;
            if child_type.is_dir() {
                pending.push(child.path());
            }
        }
    }
    return Ok(());
}

/// What:
///  Removes a stage container with everything in it, whatever modes the installed directories carry.
/// Why:
///   Completed transactions and abandoned stages are removed the same way, and an absent stage is already removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await prepareStageRemoval(stageContainer); await rm(stageContainer, { recursive: true, force: true });
/// ```
pub fn remove_stage(stage_container: &Path) -> Result<(), WorktreeCopyError> {
    // A stage made by a snapshot is a subvolume; where the mount allows it one call removes the whole tree.
    #[cfg(target_os = "linux")]
    if crate::fs_kind::is_subvolume_root(stage_container).unwrap_or(false) && crate::fs_btrfs_ioctl::subvolume_destroy_at(stage_container).is_ok() {
        return Ok(());
    }
    prepare_stage_removal(stage_container)?;
    return remove_tree_if_present(stage_container);
}

/// What:
///  Removes one completed transaction: its stage, its unfinished temporaries, and its journal.
/// Why:
///   A complete phase is written first, so an interruption during cleanup resumes as cleanup, not as installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeWorktreeCopyJournal(pending);
/// ```
pub fn remove_pending_journal(pending: &PendingJournal) -> Result<(), WorktreeCopyError> {
    if pending.journal.phase != Phase::Complete {
        let completed: Journal = Journal {
            phase: Phase::Complete,
            ..pending.journal.clone()
        };
        write_journal(pending.path.as_path(), &completed)?;
    }
    remove_stage(Path::new(pending.journal.stage_container.as_str()))?;
    remove_file_if_present(pending.path.as_path())?;
    remove_unfinished_writes(pending.path.as_path())?;
    if let Some(parent) = pending.path.parent() {
        sync_directory(parent).map_err(|error| return raw_failure(&error))?;
    }
    return Ok(());
}

/// What:
///  How many directory levels below a removed stage are emptied by one thread before the rest is split among workers.
/// Why:
///   A stage holds a few very large trees (`node_modules`, `target`); three levels below the payload give enough
///  independent subtrees to share without walking the whole stage on one thread first.
///  Measured with `remove_tree_split` itself at depths 1 to 6 on reflinked copies of a tree of 22,717 entries of this repository
///  (2026-11-03, five runs each in alternating orders, other sessions running, a debug build): medians 365, 281, 304, 325, 437 and
///  512 ms; depths 2 and 3 overlap (251 to 305 ms and 253 to 309 ms), and depths 1, 4, 5 and 6 were slower. Kept at 3 as the coordinator's pick under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups", option A). The tree has no `node_modules` or `target`, so a stage with a few very large subtrees was not measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMOVAL_SPLIT_DEPTH = 3;
/// ```
const REMOVAL_SPLIT_DEPTH: usize = 3;

/// What:
///  Removes a directory tree, sharing the subtrees below a split depth among a bounded set of threads.
/// Why:
///   Unlinking a staged copy entry by entry took longer than staging it on a tree of hundreds of thousands of entries;
///  the unlinks are independent, and the thread count follows the host's budget (the CPUs the process may use less a margin, and the free
///  memory), capped at eight.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeTreeSplit(path); // rm -r, with the subtrees removed concurrently
/// ```
fn remove_tree_split(path: &Path) -> std::io::Result<()> {
    let mut shallow: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut level: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut subtrees: Vec<PathBuf> = Vec::new();
    for depth in 0..REMOVAL_SPLIT_DEPTH {
        let mut next: Vec<PathBuf> = Vec::new();
        for directory in &level {
            for listed in std::fs::read_dir(directory)? {
                let entry: std::fs::DirEntry = listed?;
                let child: PathBuf = entry.path();
                if !entry.file_type()?.is_dir() {
                    std::fs::remove_file(child.as_path())?;
                } else if depth + 1 == REMOVAL_SPLIT_DEPTH {
                    subtrees.push(child);
                } else {
                    shallow.push(child.clone());
                    next.push(child);
                }
            }
        }
        level = next;
    }
    let workers: usize = sliced_workers(usize::MAX, PARALLEL_COPY_MAX_WORKERS, &HostResources::read());
    let cursor: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let outcomes: Vec<std::io::Result<()>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_worker| {
                return start_or_run(scope, || -> std::io::Result<()> {
                    loop {
                        let index: usize = cursor.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(subtree) = subtrees.get(index) else {
                            return Ok(());
                        };
                        std::fs::remove_dir_all(subtree.as_path())?;
                    }
                });
            })
            .collect();
        return handles
            .into_iter()
            .map(|handle| return handle.join().unwrap_or_else(|_panic| return Err(std::io::Error::other("a removal worker panicked"))))
            .collect();
    });
    for outcome in outcomes {
        outcome?;
    }
    for directory in shallow.iter().rev() {
        std::fs::remove_dir(directory.as_path())?;
    }
    return Ok(());
}

/// What:
///  Removes a tree if it exists, sharing its subtrees among threads.
/// Why:
///   An absent tree is the normal answer for a retried removal; a failed split removal falls back to the plain recursive
///  removal, which reports what remains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { recursive: true, force: true });
/// ```
fn remove_tree_if_present(path: &Path) -> Result<(), WorktreeCopyError> {
    // `match` accepts absence and reports every other removal failure.
    match remove_tree_split(path).or_else(|_split_failure| return std::fs::remove_dir_all(path)) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes one file, treating an absent file as removed.
/// Why:
///   The same retry reasoning as for trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true });
/// ```
pub fn remove_file_if_present(path: &Path) -> Result<(), WorktreeCopyError> {
    // `match` accepts absence and reports every other removal failure.
    match std::fs::remove_file(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes the temporary writes an interrupted write of one journal left beside it.
/// Why:
///   Only temporaries named after this journal are removed, so another journal's write is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeUnfinishedJournalWrites(path)
/// ```
fn remove_unfinished_writes(path: &Path) -> Result<(), WorktreeCopyError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let prefix: String = format!(
        "{}.",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    for listed in std::fs::read_dir(parent).map_err(|error| return raw_failure(&error))? {
        let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix.as_str()) && name.ends_with(TEMPORARY_SUFFIX) {
            remove_file_if_present(entry.path().as_path())?;
        }
    }
    return Ok(());
}

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_journal_tests.rs"]
mod tests;
