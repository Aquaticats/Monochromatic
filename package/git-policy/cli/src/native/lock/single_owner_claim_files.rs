//! What:
//!  The numbered claim files one item's claims are kept in:
//!  `<prefix>.<generation>` beside it,
//!  each made whole in a private temporary file and published with one exclusive `link`,
//!  read back through the same bound a record of this command has,
//!  and a released claim rewritten in place by a rename.
//! Why:
//!  A claim is a file the kernel publishes for exactly one command,
//!  and the words that make that true are a job of their own:
//!  the generation a name holds,
//!  the private file every write goes through,
//!  and the directory sync that keeps a published claim after a crash.
//!  Every write of a claim goes through here,
//!  so the one place a claim's text is written is the one place its durability is.
//!
//! Files beside this one,
//!  and why the claim is not one file:
//!  `single_owner_claim_record.rs` holds the text of one claim file,
//!  `single_owner_claim_liveness.rs` decides whether the owner a record names still runs,
//!  and `single_owner_claim.rs` takes the claim and gives it back.
//!  `rust/max-lines` splits the claim by role,
//!  and the files a claim lives in are the role a reader can hold alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const generations = (directory: string, prefix: string): number[] => { /* ... */ };
//! const publishNew = (directory: string, target: string, record: ClaimRecord): void => { /* ... */ };
//! ```

/// The record a claim file's text holds,
///  and the text an owner's record is written as.
use crate::single_owner_claim_record::{
    ClaimRecord,
    encode,
};
/// The refusal a claim fails with,
///  which names the item and what was being done to it.
use crate::subvolume_error::MigrateError;
/// The shared write steps:
///  the flush-and-sync of a written file,
///  and the sync of the directory that holds it.
use crate::subvolume_file_write_steps::{
    sync_directory_to_disk,
    write_all_then_sync,
};
/// Borrowed and owned file names,
///  which a claim name is built from.
use std::ffi::{
    OsStr,
    OsString,
};
/// The bytes of a file name,
///  which the generation a name holds is read from.
use std::os::unix::ffi::OsStrExt;
/// The mode a private claim file is made with (`0o600`),
///  as an open option.
use std::os::unix::fs::OpenOptionsExt;
/// The paths a claim is laid out as.
use std::path::{
    Path,
    PathBuf,
};
/// The counter that keeps two temporary files of one process apart,
///  and its ordering.
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// Distinguishes the temporary files of claims taken by one process at the same instant.
static TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The generation a directory entry names under `prefix`,
/// or nothing for any other name.
fn generation_of(
    name: &OsStr,
    prefix: &OsStr,
) -> Option<u64> {
    let digits: &[u8] = name
        .as_bytes()
        .strip_prefix(prefix.as_bytes())?
        .strip_prefix(b".")?;
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    return std::str::from_utf8(digits).ok()?.parse().ok();
}

/// Every generation of `prefix` in `directory`,
/// ascending.
pub(crate) fn generations(
    directory: &Path,
    prefix: &OsStr,
) -> Result<Vec<u64>, MigrateError> {
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) => return Err(MigrateError::io("list the claims in", directory, &error)),
    };
    let mut found: Vec<u64> = Vec::new();
    for entry in entries.flatten() {
        if let Some(generation) = generation_of(entry.file_name().as_os_str(), prefix) {
            found.push(generation);
        }
    }
    found.sort_unstable();
    return Ok(found);
}

/// The path of one generation.
pub(crate) fn path_of(
    directory: &Path,
    prefix: &OsStr,
    generation: u64,
) -> PathBuf {
    let mut name: OsString = prefix.to_os_string();
    name.push(format!(".{generation}"));
    return directory.join(name);
}

/// Write `record` into a new private file beside the claims,
/// synchronized,
/// and return its path.
fn write_private(
    directory: &Path,
    record: &ClaimRecord,
) -> std::io::Result<PathBuf> {
    let unique: u64 = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };
    let path: PathBuf = directory.join(format!(
        ".cli-git-claim.tmp.{}.{nanos}.{unique}",
        std::process::id()
    ));
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    // The flush is the shared step's, so a call that keeps its writes in memory (
    // `subvolume_file_write_steps::WritesKeptInMemory`,
    // which a stand-in platform installs for a control that simulates a kill in the process) keeps this one too,
    // and a release build always flushes.
    let written: std::io::Result<()> = write_all_then_sync(&mut file, encode(record).as_bytes());
    if let Err(error) = written {
        let _removed: std::io::Result<()> = std::fs::remove_file(&path);
        return Err(error);
    }
    return Ok(path);
}

/// Make the directory's new entry durable,
/// as the shared step does.
fn sync_directory(directory: &Path) -> std::io::Result<()> {
    return sync_directory_to_disk(directory);
}

/// Publish `record` as `target` without replacing anything:
/// `AlreadyExists` when the generation exists.
pub(crate) fn publish_new(
    directory: &Path,
    target: &Path,
    record: &ClaimRecord,
) -> std::io::Result<()> {
    let private: PathBuf = write_private(directory, record)?;
    let linked: std::io::Result<()> = std::fs::hard_link(&private, target);
    let _removed: std::io::Result<()> = std::fs::remove_file(&private);
    linked?;
    return sync_directory(directory);
}

/// The claim file's record,
/// or nothing when the file is gone,
/// or an error when it is not a claim of this command.
pub(crate) fn read_record(path: &Path) -> Result<Option<ClaimRecord>, MigrateError> {
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file(path, SMALL_RECORD_LIMIT) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(MigrateError::io("read the claim", path, &error)),
    };
    return match crate::single_owner_claim_record::decode(
        String::from_utf8_lossy(bytes.as_slice()).as_ref(),
    ) {
        Some(record) => Ok(Some(record)),
        None => Err(MigrateError::Journal(format!(
            "{} is not a claim file this command wrote, so it cannot tell whether another command is working here; nothing was changed. Look at the file: if no monochromatic-git-policy-cli command is running on this item, remove it and run the command again",
            path.display()
        ))),
    };
}

/// The most bytes of a claim file that are read.
const SMALL_RECORD_LIMIT: u64 = 4096;

/// Replace the claim file at `path` by a released record through a new private file and a rename,
/// then make it durable.
pub(crate) fn rewrite_as_released(
    directory: &Path,
    path: &Path,
) -> std::io::Result<()> {
    let private: PathBuf = write_private(directory, &ClaimRecord::Released)?;
    if let Err(error) = std::fs::rename(&private, path) {
        let _removed: std::io::Result<()> = std::fs::remove_file(&private);
        return Err(error);
    }
    return sync_directory(directory);
}
