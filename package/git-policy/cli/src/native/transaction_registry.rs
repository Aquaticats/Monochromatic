//! What:
//!  The per-worktree registry of commit-transaction directories,
//!       `<git-dir>/cli-git-transactions/`:
//!  entry names,
//!  publication by rename,
//!  removal by
//!       rename,
//!  and the strict listing recovery reads.
//! Why:
//!  Each transaction owns `<registry>/<uuid>/`,
//!  published only once `owner.json` is
//!      complete,
//!  so every published directory names its owner;
//!  removal renames it to a
//!      retired name first,
//!  so a reader never sees a published directory lose its owner.
//!  Both
//!      wrappers create and read the same registry (`src/policy-engine/commit-transaction-registry.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const directory = await publishTransactionDirectory({ root, transactionId, ownerBytes });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Private creation,
///  writing,
///  syncing and removal.
use super::private_storage::{
    PRIVATE_DIRECTORY_MODE, create_private_directory, protect_path, remove_tree, sync_directory,
    write_private_file,
};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Git administrative name of the per-worktree transaction registry.
pub const TRANSACTION_ROOT_NAME: &str = "cli-git-transactions";

/// The single per-index journal directory written by builds before per-transaction journals.
pub const LEGACY_TRANSACTION_DIRECTORY_NAME: &str = "cli-git-transaction";

/// Owner record filename in every published transaction directory.
pub const OWNER_FILENAME: &str = "owner.json";

/// Suffix of an unpublished candidate whose owner record may be incomplete.
pub const STAGING_SUFFIX: &str = ".pending";

/// Suffix of a completed transaction directory whose removal is in progress.
pub const RETIRED_SUFFIX: &str = ".retired";

/// Landing lock directory name inside the registry.
pub const LANDING_LOCK_NAME: &str = "landing.lock";

/// Landing reservation lock directory name inside the registry.
pub const RESERVATION_LOCK_NAME: &str = "reservation.lock";

/// What:
///  Whether a registry entry is one of the registry's own locks or a candidate or stale
///       copy of one,
///  rather than a transaction.
/// Why:
///   `landing.lock` and `reservation.lock` and their `.<uuid>.pending` / `.<uuid>.stale`
///       siblings live beside transactions and are never classified as transactions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isRegistryLockName(name: string): boolean;
/// ```
pub fn is_registry_lock_name(name: &str) -> bool {
    for lock in [LANDING_LOCK_NAME, RESERVATION_LOCK_NAME] {
        if name == lock || name.strip_prefix(lock).is_some_and(starts_with_dot) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether text starts with a dot.
/// Why:
///   A named function keeps the check out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (rest: string) => rest.startsWith('.')
/// ```
fn starts_with_dot(rest: &str) -> bool {
    return rest.starts_with('.');
}

/// What:
///  Whether a name is exactly one canonical lowercase UUID,
///  as `randomUUID` prints it.
/// Why:
///   Only such a name is a published transaction;
///  any other name is staging,
///  retired,
///  or
///       unexpected state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTransactionId(name: string): boolean;
/// ```
pub fn is_transaction_id(name: &str) -> bool {
    let bytes: &[u8] = name.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let separator: bool = index == 8 || index == 13 || index == 18 || index == 23;
        let valid: bool = if separator {
            *byte == b'-'
        } else {
            byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)
        };
        if !valid {
            return false;
        }
    }
    return true;
}

/// What:
///  How recovery treats one registry entry,
///  by its name.
/// Why:
///   A staging candidate never started capture,
///  a retired directory finished,
///  and a
///       published one must be recovered or skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionRegistryEntryKind = 'transaction' | 'staging' | 'retired';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A published transaction directory.
    Transaction,
    /// An unpublished candidate.
    Staging,
    /// A completed directory whose removal was interrupted.
    Retired,
}

/// What:
///  One classified registry entry.
/// Why:
///   Recovery needs the kind,
///  the transaction ID shared by every name form,
///  and the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionRegistryEntry = { kind; transactionId: string; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryEntry {
    /// Publication state encoded in the name.
    pub kind: EntryKind,
    /// Transaction ID.
    pub transaction_id: String,
    /// Absolute entry path.
    pub path: PathBuf,
}

/// What:
///  The transaction ID of a `<uuid><suffix>` name,
///  or nothing.
/// Why:
///   Staging and retired names carry the ID of the transaction they belong to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function suffixedTransactionId({ name, suffix }): string; // '' when not suffixed
/// ```
fn suffixed_transaction_id<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let prefix: &str = name.strip_suffix(suffix)?;
    if is_transaction_id(prefix) {
        return Some(prefix);
    }
    return None;
}

/// What:
///  Classify one entry by its name,
///  given that it is a real directory.
/// Why:
///   A name that is neither a transaction,
///  a staging candidate nor a retired directory is
///       unexpected state,
///  which recovery refuses to guess about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyRegistryEntry({ root, entry }): TransactionRegistryEntry; // throws when unexpected
/// ```
pub fn classify_entry_name(root: &Path, name: &str) -> Result<RegistryEntry, RecoveryError> {
    let path: PathBuf = root.join(name);
    if is_transaction_id(name) {
        return Ok(RegistryEntry {
            kind: EntryKind::Transaction,
            transaction_id: String::from(name),
            path,
        });
    }
    if let Some(id) = suffixed_transaction_id(name, STAGING_SUFFIX) {
        return Ok(RegistryEntry {
            kind: EntryKind::Staging,
            transaction_id: String::from(id),
            path,
        });
    }
    if let Some(id) = suffixed_transaction_id(name, RETIRED_SUFFIX) {
        return Ok(RegistryEntry {
            kind: EntryKind::Retired,
            transaction_id: String::from(id),
            path,
        });
    }
    return Err(RecoveryError(format!(
        "Unexpected transaction registry entry: {}",
        path.display()
    )));
}

/// What:
///  Whether a path exists as a real directory,
///  not a link.
///       `Ok(false)` means nothing is there.
/// Why:
///   The registry and its entries are trusted only as real directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = await lstat(path); metadata.isDirectory() && !metadata.isSymbolicLink()
/// ```
fn real_directory(path: &Path) -> Result<Option<bool>, std::io::Error> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => return Ok(Some(metadata.is_dir())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// What:
///  Every registry entry except the registry's own locks,
///  sorted by name,
///  without
///       following links;
///  an absent registry has none.
/// Why:
///   Recovery must see every transaction;
///  a link,
///  a file or an unexpected name in the
///       registry fails closed with its path named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function listTransactionEntries(root: string): Promise<readonly TransactionRegistryEntry[]>;
/// ```
pub fn list_transaction_entries(root: &Path) -> Result<Vec<RegistryEntry>, RecoveryError> {
    match real_directory(root) {
        Ok(None) => return Ok(Vec::new()),
        Ok(Some(true)) => {}
        Ok(Some(false)) => {
            return Err(RecoveryError(format!(
                "Unsafe transaction registry: {}",
                root.display()
            )));
        }
        Err(error) => {
            return Err(io_failure(
                "inspecting the transaction registry",
                root,
                &error,
            ));
        }
    }
    let listing: std::fs::ReadDir = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) => return Err(io_failure("listing the transaction registry", root, &error)),
    };
    // `mut` allows collecting one entry at a time.
    let mut entries: Vec<RegistryEntry> = Vec::new();
    for item in listing {
        let entry: std::fs::DirEntry = match item {
            Ok(found) => found,
            Err(error) => return Err(io_failure("listing the transaction registry", root, &error)),
        };
        let name_os: std::ffi::OsString = entry.file_name();
        // A name that is not UTF-8 cannot be a transaction or a lock.
        let Some(name) = name_os.to_str() else {
            return Err(RecoveryError(format!(
                "Unexpected transaction registry entry: {}",
                root.join(&name_os).display()
            )));
        };
        if is_registry_lock_name(name) {
            continue;
        }
        // `file_type` reads the entry type without following a link.
        let is_directory: bool = match entry.file_type() {
            Ok(kind) => kind.is_dir(),
            Err(error) => {
                return Err(io_failure(
                    "inspecting a registry entry",
                    entry.path().as_path(),
                    &error,
                ));
            }
        };
        if !is_directory {
            return Err(RecoveryError(format!(
                "Unsafe transaction recovery directory: {}",
                entry.path().display()
            )));
        }
        entries.push(classify_entry_name(root, name)?);
    }
    entries.sort_by(compare_entry_paths);
    return Ok(entries);
}

/// What:
///  Order two entries by path bytes.
/// Why:
///   A named comparison keeps the sort free of closures;
///  the order only needs to be
///       stable,
///  because recovery orders transactions by their owners' creation times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0
/// ```
fn compare_entry_paths(left: &RegistryEntry, right: &RegistryEntry) -> std::cmp::Ordering {
    return left.path.cmp(&right.path);
}

/// What:
///  Create the registry when absent and prove it is a real directory under a canonical
///       parent.
/// Why:
///   A link anywhere on the way could redirect transaction state outside the Git
///       directory;
///  the incumbent refuses a non-canonical parent and a linked registry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ensureTransactionRoot(root: string): Promise<void>;
/// ```
pub fn ensure_transaction_root(root: &Path) -> Result<(), RecoveryError> {
    let (Some(parent), Some(name)) = (root.parent(), root.file_name()) else {
        return Err(RecoveryError(format!(
            "Git transaction registry path {} has no administrative parent.",
            root.display()
        )));
    };
    let canonical: PathBuf = match std::fs::canonicalize(parent) {
        Ok(found) => found,
        Err(error) => return Err(io_failure("resolving the registry parent", parent, &error)),
    };
    if canonical.join(name) != root {
        return Err(RecoveryError(format!(
            "Git transaction registry path {} has a noncanonical administrative parent.",
            root.display()
        )));
    }
    match create_private_directory(root) {
        Ok(()) => {
            if let Err(error) = protect_path(root, PRIVATE_DIRECTORY_MODE) {
                return Err(io_failure(
                    "protecting the transaction registry",
                    root,
                    &error,
                ));
            }
            if let Err(error) = sync_directory(canonical.as_path()) {
                return Err(io_failure(
                    "syncing the registry parent",
                    canonical.as_path(),
                    &error,
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(io_failure(
                "creating the transaction registry",
                root,
                &error,
            ));
        }
    }
    match real_directory(root) {
        Ok(Some(true)) => return Ok(()),
        Ok(_) => {
            return Err(RecoveryError(format!(
                "Unsafe transaction registry: {}",
                root.display()
            )));
        }
        Err(error) => {
            return Err(io_failure(
                "inspecting the transaction registry",
                root,
                &error,
            ));
        }
    }
}

/// What:
///  Publish one transaction directory holding its complete owner record.
///       Returns the published directory.
/// Why:
///   The directory is built under its staging name,
///  made durable,
///  then renamed;
///  a
///       failure before the rename removes the candidate,
///  so nothing half-built is published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function publishTransactionDirectory({ root, transactionId, ownerBytes }): Promise<string>;
/// ```
pub fn publish_transaction_directory(
    root: &Path,
    transaction_id: &str,
    owner_bytes: &[u8],
) -> Result<PathBuf, RecoveryError> {
    if !is_transaction_id(transaction_id) {
        return Err(RecoveryError(format!(
            "Malformed commit transaction ID: {transaction_id}"
        )));
    }
    let staging: PathBuf = root.join(format!("{transaction_id}{STAGING_SUFFIX}"));
    let directory: PathBuf = root.join(transaction_id);
    if let Err(error) = create_private_directory(staging.as_path()) {
        return Err(io_failure(
            "creating the transaction candidate",
            staging.as_path(),
            &error,
        ));
    }
    if let Err(failure) = fill_candidate(staging.as_path(), owner_bytes) {
        // The candidate was never published; removing it is best effort.
        let _ = remove_tree(staging.as_path());
        return Err(failure);
    }
    if let Err(error) = std::fs::rename(staging.as_path(), directory.as_path()) {
        let _ = remove_tree(staging.as_path());
        return Err(io_failure(
            "publishing the transaction directory",
            directory.as_path(),
            &error,
        ));
    }
    if let Err(error) = sync_directory(root) {
        return Err(io_failure("syncing the transaction registry", root, &error));
    }
    return Ok(directory);
}

/// What:
///  Protect a fresh candidate and write its owner record durably.
/// Why:
///   Everything before the publishing rename,
///  so a failure can remove the candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await protectPath(staging); await writePrivateFile(owner); await syncDirectory(staging);
/// ```
fn fill_candidate(staging: &Path, owner_bytes: &[u8]) -> Result<(), RecoveryError> {
    if let Err(error) = protect_path(staging, PRIVATE_DIRECTORY_MODE) {
        return Err(io_failure(
            "protecting the transaction candidate",
            staging,
            &error,
        ));
    }
    let owner: PathBuf = staging.join(OWNER_FILENAME);
    if let Err(error) = write_private_file(owner.as_path(), owner_bytes) {
        return Err(io_failure(
            "writing the transaction owner record",
            owner.as_path(),
            &error,
        ));
    }
    if let Err(error) = sync_directory(staging) {
        return Err(io_failure(
            "syncing the transaction candidate",
            staging,
            &error,
        ));
    }
    return Ok(());
}

/// What:
///  Remove a transaction directory:
///  a registry directory is renamed to its retired name
///       first,
///  any other directory (the legacy journal) is removed in place.
/// Why:
///   A reader listing the registry must never see a published directory without its
///       owner record;
///  the retired name is ignored by every reader.
///  A registry removal syncs nothing, because recovery removes a retired directory that outlives a power loss and
///       completes a landed transaction directory that is still published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeTransactionDirectory(directory: string): Promise<void>;
/// ```
pub fn remove_transaction_directory(directory: &Path) -> Result<(), RecoveryError> {
    let Some(parent) = directory.parent() else {
        return Err(RecoveryError(format!(
            "Transaction directory {} has no parent.",
            directory.display()
        )));
    };
    let in_registry: bool = parent.file_name().is_some_and(is_registry_name)
        && directory
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(is_transaction_id);
    if !in_registry {
        if let Err(error) = remove_tree(directory) {
            return Err(io_failure(
                "removing the transaction directory",
                directory,
                &error,
            ));
        }
        return sync_parent(parent);
    }
    // `as_os_str().to_owned()` copies the raw path so the suffix can be appended.
    let mut retired_name: std::ffi::OsString = directory.as_os_str().to_owned();
    retired_name.push(RETIRED_SUFFIX);
    let retired: PathBuf = PathBuf::from(retired_name);
    if let Err(error) = std::fs::rename(directory, retired.as_path()) {
        // Another recoverer of the same dead transaction retired it first: the end state is the same.
        if error.kind() == std::io::ErrorKind::NotFound {
            debug(
                "removeTransactionDirectory",
                format!("transaction already retired by another recoverer: {}", directory.display()).as_str(),
            );
            return Ok(());
        }
        return Err(io_failure(
            "retiring the transaction directory",
            directory,
            &error,
        ));
    }
    // No sync around the rename or the deletions. Journaling filesystems apply metadata operations in order, so a power
    // loss leaves the published name, the retired name with some contents, or nothing, and recovery handles each.
    if let Err(error) = remove_tree(retired.as_path()) {
        return Err(io_failure(
            "removing the retired transaction",
            retired.as_path(),
            &error,
        ));
    }
    return Ok(());
}

/// What:
///  Whether a directory name is the registry's name.
/// Why:
///   A named function keeps the check out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (name: string) => name === TRANSACTION_ROOT_NAME
/// ```
fn is_registry_name(name: &std::ffi::OsStr) -> bool {
    return name == TRANSACTION_ROOT_NAME;
}

/// What:
///  Sync a parent directory,
///  naming it on failure.
/// Why:
///   Only the in-place removal of a legacy directory, which has no retired name for recovery to find, syncs its parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await syncDirectory(parent);
/// ```
fn sync_parent(parent: &Path) -> Result<(), RecoveryError> {
    if let Err(error) = sync_directory(parent) {
        return Err(io_failure(
            "syncing the transaction registry",
            parent,
            &error,
        ));
    }
    return Ok(());
}

/// Registry controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_registry_tests.rs"]
mod tests;
