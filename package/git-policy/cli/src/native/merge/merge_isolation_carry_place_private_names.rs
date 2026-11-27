//! What:
//!  The private names one planned worktree write makes beside its path:
//!  the name the new bytes are written under before they take the path,
//!  the writing of them,
//!  and the removal of a name the write turned out not to need.
//! Why:
//!  A rename in the same directory is one step on whatever file system the worktree is on,
//!  so a kill leaves either what was there or the whole new entry;
//!  the name carries the key of the merge that wrote it,
//!  so the next command can tell whose name it is before it removes one
//!  (`merge_isolation_take_aside_names.rs`).
//!  Writing the new bytes under that name and placing them is
//!  `merge_isolation_carry_place.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const private = writePrivate(location, bytes, mode, bits, transactionId); // then place it, or remove it
//! ```

/// One file call of this step handed to the blocking pool,
/// so a disk that makes it wait holds no worker of the runtime
/// (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The entry such a name is written with.
use crate::merge_isolation_carry_place_write::write_entry;
/// The prefix a private name carries,
/// which tells the next command whose name it is.
use crate::merge_isolation_take_aside_names::CARRY_PREFIX;
/// A place in the worktree,
/// and the private name beside it.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The private name the new entry is written under before it takes its path,
///  which carries the key of the merge that wrote it.
/// Why:
///  A kill between writing those bytes and renaming them into place leaves the name behind,
/// and nothing outside this
///  command knows it,
/// so the next command must be able to tell whose name it is before it removes it:
/// a name with no
///  merge key could be one a merge that still runs is about to rename into place
/// (`merge_isolation_take_aside_names.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname(location)}/${CARRY_PREFIX}-${nameKey(basename)}-${nameKey(transactionId)}-${pid}-${count}-${nanos}`
/// ```
fn private_name(
    location: &Path,
    transaction_id: &str,
) -> PathBuf {
    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let count: u64 = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let process: u32 = std::process::id();
    return location.with_file_name(format!(
        "{CARRY_PREFIX}-{}-{}-{process}-{count}-{nanos:x}",
        crate::merge_isolation_take_aside_names::name_key(
            crate::merge_isolation_take_aside_names::name_text(location).as_str()
        ),
        crate::merge_isolation_take_aside_names::name_key(transaction_id)
    ));
}

/// What:
///  Write the new entry under a private name beside its path,
///  so placing it is one rename.
/// Why:
///  A rename in the same directory is one step on whatever file system the worktree is on,
/// so a kill leaves
///  either what was there or the whole new entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writePrivate(location, bytes, mode, bits): string; // throws on a write that failed
/// ```
fn write_private(
    location: &Path,
    bytes: &[u8],
    mode: &str,
    the_persons_bits: Option<u32>,
    transaction_id: &str,
) -> Result<PathBuf, String> {
    if let Some(parent) = location.parent() {
        let _made = std::fs::create_dir_all(parent);
    }
    let private: PathBuf = private_name(location, transaction_id);
    write_entry(private.as_path(), bytes, mode, the_persons_bits)?;
    return Ok(private);
}

/// What:
///  `write_private`,
///  made as a job of the blocking pool.
/// Why:
///   Making the directory and writing the entry are calls that can wait on a disk,
/// so they never hold the worker of the runtime that
///  polls this step,
///  and the name and the error are the ones `write_private` gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writePrivateOnThePool(location, bytes, mode, bits, transactionId): Promise<string>; // throws on a write that failed
/// ```
pub(super) async fn write_private_awaiting(
    location: &Path,
    bytes: &[u8],
    mode: &str,
    the_persons_bits: Option<u32>,
    transaction_id: &str,
) -> Result<PathBuf, String> {
    let (named, written, entry_mode): (PathBuf, Vec<u8>, String) =
        (location.to_path_buf(), bytes.to_vec(), String::from(mode));
    let record: String = String::from(transaction_id);
    return match run_blocking_job_within_the_budget(move || {
        return write_private(
            named.as_path(),
            written.as_slice(),
            entry_mode.as_str(),
            the_persons_bits,
            record.as_str(),
        );
    })
    .await
    {
        Ok(made) => made,
        // The system gave the pool no thread, so the entry may not have been written at all.
        Err(refused) => Err(refused.to_string()),
    };
}

/// What:
///  Remove one private name this step made,
///  as a job of the blocking pool,
///  with nothing said about a failure.
/// Why:
///   A removal is a file call that can wait on a disk;
/// a name that does not go is not a failure of the write,
///  because the next command's sweep of the merge's own names collects what is left
///  (`merge_isolation_take_aside_names.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeAPrivateName(private: string): Promise<void>; // rm(private), a failure ignored
/// ```
pub(super) async fn remove_a_private_name_awaiting(private: &Path) {
    let named: PathBuf = private.to_path_buf();
    let _removal = run_blocking_job_within_the_budget(move || {
        return std::fs::remove_file(named.as_path());
    })
    .await;
}
