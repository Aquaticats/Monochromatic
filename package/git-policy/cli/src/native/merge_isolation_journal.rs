//! What:
//!  The durable intent record of an isolated merge: the commit it created, the branch value it expects, and the
//!  process that runs it.
//! Why:
//!  Between the moment Git starts updating the worktree and the index for the merge commit and the moment the branch
//!  moves, a killed process leaves a worktree and an index that agree with the merge while the branch does not.
//!  The record, written and synced before Git starts, lets the next command under the landing lock decide whether
//!  the merge never started (discard it) or must be completed.
//!  Records are created once and never rewritten, like the commit transaction's own state files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeFile(join(gitDir, 'cli-git-merges', `${id}.json`), encodeMergeIntent(intent), { flag: 'wx' });
//! ```

/// Record writers and readers.
use super::json_record::{ObjectWriter, decode_fatal, number_equals, parse_object, safe_integer_field, string_field};
/// The owner of a record.
use super::transaction_owner::TransactionOwner;
/// Exclusive, synced, private files and directories.
use super::private_storage::{create_private_directory, sync_directory, write_exclusive_synced};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// Directory, inside the worktree's Git directory, that holds the intent records.
pub const MERGE_INTENT_DIRECTORY: &str = "cli-git-merges";

/// Schema version of an intent record.
pub const MERGE_INTENT_SCHEMA_VERSION: i64 = 1;

/// Suffix of an intent record's file name.
const RECORD_SUFFIX: &str = ".json";

/// What:
///  One isolated merge that has created its commit and is about to move the branch to it.
/// Why:
///   Recovery needs the two commits, the ref that moves, the worktree to work in, and the owner whose death makes
///  the record recoverable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeIntent = { owner: TransactionOwner; repositoryRoot: string; targetRef?: string; oldOid: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeIntent {
    /// Transaction ID, also the record's file stem.
    pub owner: TransactionOwner,
    /// Canonical worktree top level, where recovery runs Git.
    pub repository_root: String,
    /// The branch ref that moves; absent for a detached `HEAD`.
    pub target_ref: Option<String>,
    /// The commit the target holds before the merge.
    pub old_oid: String,
    /// The merge commit the target moves to.
    pub new_oid: String,
}

/// What:
///  The directory that holds a worktree's intent records.
/// Why:
///   Every reader and writer names it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(gitDir, 'cli-git-merges')
/// ```
pub fn intent_directory(git_dir: &Path) -> PathBuf {
    return git_dir.join(MERGE_INTENT_DIRECTORY);
}

/// What:
///  The record's file name.
/// Why:
///   The transaction ID is unique per merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${id}.json`
/// ```
pub fn intent_path(git_dir: &Path, transaction_id: &str) -> PathBuf {
    return intent_directory(git_dir).join(format!("{transaction_id}{RECORD_SUFFIX}"));
}

/// What:
///  Encode an intent as compact JSON with a trailing line feed.
/// Why:
///   Every text is quoted at this final step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify(record)}\n`
/// ```
pub fn encode_merge_intent(intent: &MergeIntent) -> String {
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", MERGE_INTENT_SCHEMA_VERSION)
        .string("state", "merge-intent")
        .string("transactionId", intent.owner.transaction_id.as_str())
        .integer("ownerPid", intent.owner.owner_pid)
        .string("ownerIdentity", intent.owner.owner_identity.as_str())
        .string("createdAt", intent.owner.created_at.as_str());
    if let Some(namespace) = &intent.owner.owner_pid_namespace {
        writer.string("ownerPidNamespace", namespace.as_str());
    }
    writer
        .string("repositoryRoot", intent.repository_root.as_str())
        .string("targetRef", intent.target_ref.as_deref().unwrap_or_default())
        .string("oldOid", intent.old_oid.as_str())
        .string("newOid", intent.new_oid.as_str());
    return format!("{}\n", writer.finish());
}

/// What:
///  Parse record bytes, or nothing when they are not a valid intent.
/// Why:
///   A malformed record fails recovery closed instead of being guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMergeIntent(bytes: Uint8Array): MergeIntent | undefined;
/// ```
pub fn parse_merge_intent(bytes: &[u8]) -> Option<MergeIntent> {
    let map: serde_json::Map<String, serde_json::Value> = parse_object(decode_fatal(bytes)?)?;
    if !number_equals(&map, "schemaVersion", MERGE_INTENT_SCHEMA_VERSION)
        || string_field(&map, "state")? != "merge-intent"
    {
        return None;
    }
    let owner_pid: i64 = safe_integer_field(&map, "ownerPid")?;
    let owner_identity: &str = string_field(&map, "ownerIdentity")?;
    let old_oid: &str = string_field(&map, "oldOid")?;
    let new_oid: &str = string_field(&map, "newOid")?;
    let repository_root: &str = string_field(&map, "repositoryRoot")?;
    if owner_pid < 1 || owner_identity.is_empty() || old_oid.is_empty() || new_oid.is_empty()
        || repository_root.is_empty()
    {
        return None;
    }
    let target_ref: &str = string_field(&map, "targetRef")?;
    return Some(MergeIntent {
        owner: TransactionOwner {
            transaction_id: String::from(string_field(&map, "transactionId")?),
            owner_pid,
            owner_identity: String::from(owner_identity),
            created_at: String::from(string_field(&map, "createdAt")?),
            owner_pid_namespace: string_field(&map, "ownerPidNamespace")
                .filter(|namespace| { return !namespace.is_empty(); })
                .map(String::from),
        },
        repository_root: String::from(repository_root),
        target_ref: if target_ref.is_empty() { None } else { Some(String::from(target_ref)) },
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
    });
}

/// What:
///  Write an intent record durably: the private directory, the exclusive synced file, the synced directory.
/// Why:
///   The merge may change the worktree only after the record would survive a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }); await writeExclusiveSynced(path, bytes); await syncDirectory(directory);
/// ```
pub fn write_merge_intent(git_dir: &Path, intent: &MergeIntent) -> std::io::Result<PathBuf> {
    let directory: PathBuf = intent_directory(git_dir);
    match create_private_directory(directory.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let path: PathBuf = intent_path(git_dir, intent.owner.transaction_id.as_str());
    write_exclusive_synced(path.as_path(), encode_merge_intent(intent).as_bytes())?;
    sync_directory(directory.as_path())?;
    return Ok(path);
}

/// What:
///  Remove an intent record once the merge ended, and sync the directory.
/// Why:
///   A record that outlives its merge would be recovered again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true }); await syncDirectory(dirname(path));
/// ```
pub fn remove_merge_intent(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
    if let Some(directory) = path.parent() {
        sync_directory(directory)?;
    }
    return Ok(());
}

/// What:
///  The intent records of a worktree, oldest name first, or none when the directory does not exist.
/// Why:
///   The common case, no merge ever interrupted, costs one failed directory read and no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(directory).catch(() => [])).filter((name) => name.endsWith('.json')).sort()
/// ```
pub fn list_merge_intents(git_dir: &Path) -> Result<Vec<PathBuf>, RecoveryError> {
    let directory: PathBuf = intent_directory(git_dir);
    let listing: std::fs::ReadDir = match std::fs::read_dir(directory.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(io_failure("listing", directory.as_path(), &error)),
    };
    let mut records: Vec<PathBuf> = Vec::new();
    for item in listing {
        match item {
            Ok(entry) => {
                if entry.file_name().to_string_lossy().ends_with(RECORD_SUFFIX) {
                    records.push(entry.path());
                }
            }
            Err(error) => return Err(io_failure("listing", directory.as_path(), &error)),
        }
    }
    records.sort();
    return Ok(records);
}

/// Encoding, parsing, writing and listing of intent records.
#[cfg(test)]
#[path = "merge_isolation_journal_tests.rs"]
mod tests;
