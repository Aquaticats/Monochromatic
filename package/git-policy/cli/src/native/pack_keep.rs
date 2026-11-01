//! What:
//!  The `.keep` files that protect a landing's migrated pack until the compare-and-swap,
//!       named by the message `cli-git <transaction-id>`.
//! Why:
//!  `git index-pack --keep=<message>` writes the `.keep` before the pack becomes visible,
//!  so
//!      no prune or repack drops the objects mid-landing;
//!  recovery finds a dead transaction's
//!      `.keep` by its message even before a landing record names the pack
//!      (`src/policy-engine/commit-landing-objects.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeTransactionKeeps({ objectDirectory, transactionId });
//! ```

/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Single-file removal.
use super::recovery_files::remove_file_if_present;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The keep message naming one transaction.
/// Why:
///   Recovery matches `.keep` contents against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// transactionKeepMessage(transactionId) // `cli-git ${transactionId}`
/// ```
pub fn transaction_keep_message(transaction_id: &str) -> String {
    return format!("cli-git {transaction_id}");
}

/// What:
///  Remove the `.keep` of one migrated pack.
/// Why:
///   `removePackKeep`:
///  after `ref-updated.json`,
///  and after a failed compare-and-swap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removePackKeep({ objectDirectory, packName });
/// ```
pub fn remove_pack_keep(object_directory: &Path, pack_name: &str) -> Result<(), RecoveryError> {
    return remove_file_if_present(
        object_directory
            .join("pack")
            .join(format!("pack-{pack_name}.keep"))
            .as_path(),
    );
}

/// What:
///  Remove every `.keep` whose message names the transaction;
///  returns how many.
/// Why:
///   `removeTransactionKeeps`:
///  a `.keep` holds the message and a newline,
///  exactly;
///  a file
///       that vanished meanwhile is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeTransactionKeeps({ objectDirectory, transactionId }): Promise<number>;
/// ```
pub fn remove_transaction_keeps(
    object_directory: &Path,
    transaction_id: &str,
) -> Result<usize, RecoveryError> {
    let pack_directory: PathBuf = object_directory.join("pack");
    let listing: std::fs::ReadDir = match std::fs::read_dir(&pack_directory) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(io_failure("listing", pack_directory.as_path(), &error)),
    };
    let expected: String = format!("{}\n", transaction_keep_message(transaction_id));
    // `mut` allows collecting owned keeps before removing them.
    let mut owned: Vec<PathBuf> = Vec::new();
    for item in listing {
        let entry: std::fs::DirEntry = match item {
            Ok(found) => found,
            Err(error) => return Err(io_failure("listing", pack_directory.as_path(), &error)),
        };
        if !entry.file_name().to_string_lossy().ends_with(".keep") {
            continue;
        }
        let path: PathBuf = entry.path();
        let contents: Vec<u8> = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_failure("reading", path.as_path(), &error)),
        };
        if contents == expected.as_bytes() {
            owned.push(path);
        }
    }
    for path in &owned {
        remove_file_if_present(path.as_path())?;
    }
    return Ok(owned.len());
}

/// Keep-file controls stay out of the release executable.
#[cfg(test)]
#[path = "pack_keep_tests.rs"]
mod tests;
