//! What:
//!  The durable intent record of an isolated merge:
//! the commit it created,
//! the branch value it expects,
//! and the
//!  process that runs it.
//! Why:
//!  Between the moment Git starts updating the worktree and the index for the merge commit and the moment the branch
//!  moves,
//! a killed process leaves a worktree and an index that agree with the merge while the branch does not.
//!  The record,
//! written and synced before Git starts,
//! lets the next command decide whether
//!  the merge never started (discard it) or must be completed.
//!  Records are created once,
//! like the commit transaction's own state files,
//! and the one field a later step adds
//!  is published by a single rename,
//! so a reader sees either the record as it was or the whole new one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeFile(join(gitDir, 'monochromatic-git-policy-cli-merges', `${id}.json`), encodeMergeIntent(intent), { flag: 'wx' });
//! ```

/// Record writers and readers.
use crate::wrapper_names_old_and_new::MERGE_INTENT_FOLDER;
use crate::json_record::{ObjectWriter, decode_fatal, number_equals, parse_object, safe_integer_field, string_field};
/// The person's own work in the paths a merge changes,
/// as the record holds it.
use crate::merge_isolation_carry_parts::{Capture, decode, encode};
/// What Git's own update left at each path a merge changes,
/// as the record holds it.
use crate::merge_isolation_carry_landed::{LandedFile, decode_landed, encode_landed};
/// The owner of a record.
use crate::path_bytes_in_records as path_bytes;
use crate::transaction_owner::TransactionOwner;
/// Exclusive,
/// synced,
/// private files and directories,
/// and the one writer that replaces a record whole.
use crate::private_storage::{create_private_directory, replace_private_file_unsynced, sync_directory, write_exclusive_synced};
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// Directory,
/// inside the worktree's Git directory,
/// that holds the intent records.
pub const MERGE_INTENT_DIRECTORY: &str = MERGE_INTENT_FOLDER.current;

/// Schema version of an intent record.
///  Version 2 adds the capture of the person's own work in the paths the merge changes;
/// a record of version 1 holds none,
/// and both are read.
/// What Git's own update left is an optional field of version 2 and not a version of its own:
/// a release that does not read it ignores it and judges a path by its bytes,
/// so a downgrade after a kill still recovers the merge,
/// while a version it does not know stops every command in the worktree.
pub const MERGE_INTENT_SCHEMA_VERSION: i64 = 2;

/// The versions an earlier release wrote,
/// which this one still recovers.
pub const EARLIER_MERGE_INTENT_SCHEMA_VERSIONS: [i64; 1] = [1];

/// Suffix of an intent record's file name.
const RECORD_SUFFIX: &str = ".json";

/// What:
///  One isolated merge that has created its commit and is about to move the branch to it.
/// Why:
///   Recovery needs the two commits,
/// the ref that moves,
/// the worktree to work in,
/// and the owner whose death makes
///  the record recoverable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeIntent = { owner: TransactionOwner; repositoryRoot: string; targetRef?: string; oldOid: string; newOid: string; sideRef?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeIntent {
    /// Transaction ID,
    /// also the record's file stem.
    pub owner: TransactionOwner,
    /// Canonical worktree top level,
    /// where recovery runs Git.
    pub repository_root: PathBuf,
    /// The branch ref that moves;
    /// absent for a detached `HEAD`.
    pub target_ref: Option<String>,
    /// The commit the target holds before the merge.
    pub old_oid: String,
    /// The merge commit the target moves to.
    pub new_oid: String,
    /// The side ref that keeps the merge commit from being collected while the checkout waits for a holder of
    /// `index.lock` to let go,
    /// or nothing for a merge that is applied as soon as it is computed.
    /// A record with a side ref is finished even though nothing of the merge is in the checkout yet
    /// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
    /// finding 3).
    pub side_ref: Option<String>,
    /// The person's own work in the paths the merge changes,
    /// kept outside the checkout before the first of them is written,
    /// or nothing for a merge whose paths held none of it
    /// (`merge_isolation_carry_parts.rs`).
    pub carried: Option<Capture>,
    /// What Git's own update left at each path the merge changes,
    /// read as soon as the branch moved,
    /// or empty for a record written before that reading
    /// and for a record of a release that read none.
    /// A recovery has no reading of its own,
    ///  so a save that accepts the whole merge is told apart from Git's own
    ///  write by this one
    /// (`merge_isolation_carry_landed.rs`,
    /// `doc/handover/cli-git-rust-implementation.md`,
    /// "The carry loses nothing to an old descriptor, a save of the merge's own file, or a stranded private name").
    pub landed: Vec<LandedFile>,
}

/// What:
///  The directory that holds a worktree's intent records.
/// Why:
///   Every reader and writer names it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(gitDir, 'monochromatic-git-policy-cli-merges')
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
    // The root goes in as plain text and as its bytes, so recovery runs Git in the worktree the merge was made for.
    let plain_root: String = intent.repository_root.display().to_string();
    writer
        .string("repositoryRoot", plain_root.as_str())
        .string("repositoryRootHex", path_bytes::hex_of_path(intent.repository_root.as_path()).as_str())
        .string("targetRef", intent.target_ref.as_deref().unwrap_or_default())
        .string("oldOid", intent.old_oid.as_str())
        .string("newOid", intent.new_oid.as_str());
    if let Some(side_ref) = &intent.side_ref {
        writer.string("sideRef", side_ref.as_str());
    }
    if let Some(carried) = &intent.carried {
        writer.raw("carried", encode(carried).as_str());
    }
    // The reading goes in only where the apply made one, so the record a crash before the branch moved leaves is the record
    //  this release wrote before.
    if !intent.landed.is_empty() {
        writer.raw("landed", encode_landed(intent.landed.as_slice()).as_str());
    }
    return format!("{}\n", writer.finish());
}

/// What:
///  Whether a record's `schemaVersion` is one of the earlier versions this build still reads.
/// Why:
///   An earlier record carries the same fields,
///  so the version it names is what decides it,
///  never the shape of its bytes.
fn an_earlier_schema_version(map: &serde_json::Map<String, serde_json::Value>, earlier: &[i64]) -> bool {
    for version in earlier {
        if number_equals(map, "schemaVersion", *version) {
            return true;
        }
    }
    return false;
}

/// What:
///  Parse record bytes,
/// or nothing when they are not a valid intent.
/// Why:
///   A malformed record fails recovery closed instead of being guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMergeIntent(bytes: Uint8Array): MergeIntent | undefined;
/// ```
pub fn parse_merge_intent(bytes: &[u8]) -> Option<MergeIntent> {
    let map: serde_json::Map<String, serde_json::Value> = parse_object(decode_fatal(bytes)?)?;
    let version_readable: bool = number_equals(&map, "schemaVersion", MERGE_INTENT_SCHEMA_VERSION)
        || an_earlier_schema_version(&map, &EARLIER_MERGE_INTENT_SCHEMA_VERSIONS);
    if !version_readable || string_field(&map, "state")? != "merge-intent" {
        return None;
    }
    let owner_pid: i64 = safe_integer_field(&map, "ownerPid")?;
    let owner_identity: &str = string_field(&map, "ownerIdentity")?;
    let old_oid: &str = string_field(&map, "oldOid")?;
    let new_oid: &str = string_field(&map, "newOid")?;
    let repository_root: PathBuf = match map.get("repositoryRootHex") {
        Some(serde_json::Value::String(hexadecimal)) => path_bytes::path_of_hex(hexadecimal.as_str())?,
        Some(_not_text) => return None,
        None => PathBuf::from(string_field(&map, "repositoryRoot")?),
    };
    if owner_pid < 1 || owner_identity.is_empty() || old_oid.is_empty() || new_oid.is_empty()
        || repository_root.as_os_str().is_empty()
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
            owner_pid_namespace: match string_field(&map, "ownerPidNamespace") {
                Some(namespace) if !namespace.is_empty() => Some(String::from(namespace)),
                _ => None,
            },
        },
        repository_root,
        target_ref: if target_ref.is_empty() { None } else { Some(String::from(target_ref)) },
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
        // A record of this release's first form holds none, and a merge it names is applied in its own command.
        side_ref: match string_field(&map, "sideRef") {
            Some(name) if !name.is_empty() => Some(String::from(name)),
            _ => None,
        },
        // A record of version 1 holds none: every path it wrote was the merge's own.
        carried: match map.get("carried") {
            Some(value) => Some(decode(value)?),
            None => None,
        },
        // A record written before the branch moved holds none, and a recovery there judges a path by its bytes.
        landed: match map.get("landed") {
            Some(value) => decode_landed(value)?,
            None => Vec::new(),
        },
    });
}

/// What:
///  Write an intent record durably:
/// the private directory,
/// the exclusive synced file,
/// the synced directory.
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
///  Record what Git's own update left,
///  once the branch moved and before the carry,
///  so a command that kills the merge there leaves the next one the reading it cannot make itself.
/// Why:
///   A recovery that judged a path by its bytes alone replaced a save whose bytes accepted the whole merge
///  with the older captured layer
/// (a cross-vendor review of round two,
/// its first finding).
///   The bytes are staged beside the record and published by one rename that replaces it,
/// and nothing is flushed:
///  a machine crash then leaves the record as it was before this step,
/// which every reader treats as the state before the merge
///  wrote anything,
/// and the recovery of a killed command judges the path by its bytes as it always did.
/// The merge's own answer waits on no disk flush here,
///  as native Git's does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rewriteMergeIntent({ path, intent }): Promise<void>; // stage beside, rename over
/// ```
pub fn rewrite_merge_intent(path: &Path, intent: &MergeIntent) -> std::io::Result<()> {
    return replace_private_file_unsynced(path, encode_merge_intent(intent).as_bytes());
}

/// What:
///  Remove an intent record once the merge ended,
/// without flushing the directory.
/// Why:
///   A record that outlives its merge would be recovered again,
/// and that is what the next command does with one a power loss
///  brings back:
/// it names a merge whose commit the branch already holds,
/// and `settle_failed_merge` drops it as already
///  landed,
/// or a merge whose branch moved,
/// which it settles as it would settle any such record.
///  The flush was the third of the merge's fsyncs and it sat on the path the person waits for:
/// a merge that needs no
///  resolution took 22 to 41 seconds in a measured disk stall where real Git took 9 to 16 milliseconds
///  (finding 1 of the merge-and-pull audit of 2026-11-09;
/// `AGENTS.md`,
/// `SLW`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true }); // no syncDirectory: the next command settles a record this brought back
/// ```
pub fn remove_merge_intent(path: &Path) -> std::io::Result<()> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    };
}

/// What:
///  The intent records of a worktree,
/// oldest name first,
/// or none when the directory does not exist.
/// Why:
///   The common case,
/// no merge ever interrupted,
/// costs one failed directory read and no Git process.
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

/// Encoding,
/// parsing,
/// writing and listing of intent records.
#[cfg(test)]
#[path = "merge_isolation_journal_tests.rs"]
mod tests;
