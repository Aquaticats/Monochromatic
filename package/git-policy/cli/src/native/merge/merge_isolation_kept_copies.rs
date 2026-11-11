//! What:
//!  The records of the copies of the person's own bytes a merge keeps beside a path:
//!  one file per copy inside the worktree's Git directory,
//!  naming the copy and the path it belongs to.
//! Why:
//!   A take moves a file aside before a merge writes its own content there,
//! and a command that stops between the two leaves the file beside its path,
//!  so the copy is the only thing that holds those bytes.
//! Nothing of the person's may be removed,
//!  but a copy nothing looks at again is a file they cannot account for:
//! it sits in `git status` as an untracked name with a hash in it,
//! a merge says nothing about it unless a later merge touches that same path,
//! and no command ever ends it
//!  (`merge-carry-review-round-2-findings-1`,
//! its first two findings).
//!  So a copy that stays is recorded here,
//! and every command that runs the merge recovery decides it again
//!  (`merge_isolation_kept_copies_decisions.rs`).
//!  The record of one copy is a small file of its own,
//! written by exclusive creation,
//! so two merges in one worktree never write over each other,
//! and a record that cannot be read names a copy that is still beside its path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keepTheCopy(gitDir, location, aside); const copies = keptCopies(gitDir); forgetTheCopy(gitDir, aside);
//! ```

/// A short,
/// stable key for one text.
use crate::merge_isolation_take_aside_names::name_key;
/// The line a command writes when a record cannot be written or removed.
use crate::diagnostic_log::warn;
/// Exclusive,
/// synced,
/// private files and directories.
use crate::private_storage::{create_private_directory, sync_directory, write_exclusive_synced};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// Directory,
/// inside the worktree's Git directory,
/// that names every copy a merge kept beside its path.
pub(crate) const KEPT_COPY_DIRECTORY: &str = "monochromatic-git-policy-cli-kept-copies";

/// Suffix of one kept copy's record.
const RECORD_SUFFIX: &str = ".copy";

/// The tag of this module in the diagnostic log.
const TAG: &str = "mergeKeptCopies";

/// What:
///  One copy a merge kept,
///  with the worktree path it belongs to.
/// Why:
///   The name of a copy carries a hash of its path's name and not the name itself,
/// so a record that held only the copy's name could not say where the copy belongs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptCopy = { location: string; aside: string };
/// ```
pub(crate) struct KeptCopy {
    /// The worktree path the copy belongs to.
    pub(crate) location: PathBuf,
    /// The private name the copy waits under.
    pub(crate) aside: PathBuf,
}

/// The directory that holds the records of one worktree's kept copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(gitDir, 'monochromatic-git-policy-cli-kept-copies')
/// ```
fn copy_directory(git_dir: &Path) -> PathBuf {
    return git_dir.join(KEPT_COPY_DIRECTORY);
}

/// The hex text of one path,
/// which a record keeps so a name that is not valid text survives the round trip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hex = (path: string) => Buffer.from(path, 'utf8').toString('hex');
/// ```
fn hex_of(path: &Path) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return hex_of_bytes(path.as_os_str().as_bytes());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return hex_of_bytes(path.to_string_lossy().as_bytes());
    }
}

/// The hex text of a run of bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexOfBytes = (bytes: Uint8Array) => Buffer.from(bytes).toString('hex');
/// ```
fn hex_of_bytes(bytes: &[u8]) -> String {
    let mut found: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        found.push_str(format!("{byte:02x}").as_str());
    }
    return found;
}

/// Whether one byte is one of the hex digits a key or a ran number is written in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAHexDigit = (byte: number) => /[0-9a-f]/i.test(String.fromCharCode(byte));
/// ```
fn is_a_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// The path one hex text names,
/// or nothing when the text is not an even run of hex digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathOfHex = (hex: string) => Buffer.from(hex, 'hex');
/// ```
fn path_of_hex(text: &str) -> Option<PathBuf> {
    if text.len() % 2 != 0 || !text.bytes().all(is_a_hex_digit) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.as_bytes().as_chunks::<2>().0 {
        let high: u32 = u32::from((pair[0] as char).to_digit(16)?);
        let low: u32 = u32::from((pair[1] as char).to_digit(16)?);
        bytes.push(u8::try_from((high << 4) | low).ok()?);
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        return Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return Some(PathBuf::from(String::from_utf8(bytes).ok()?));
    }
}

/// What:
///  The record of one kept copy:
///  its path,
///  then the private name it waits under,
///  one hex text a line.
/// Why:
///   Both are paths a person can make out of any byte,
/// so hex keeps a name that is not valid text intact,
/// and the
///  record's own name is a key,
/// which stays short enough for any path the file system takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${hex(location)}\n${hex(aside)}\n`
/// ```
fn encode_copy(copy: &KeptCopy) -> String {
    return format!("{}\n{}\n", hex_of(copy.location.as_path()), hex_of(copy.aside.as_path()));
}

/// What:
///  The copy one record names,
///  or nothing when the record is not one.
/// Why:
///   A record that cannot be read names a copy that is still beside its path,
/// so leaving it out loses nothing and stops nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeCopy(bytes: Uint8Array): KeptCopy | undefined;
/// ```
fn decode_copy(bytes: &[u8]) -> Option<KeptCopy> {
    let text: &str = std::str::from_utf8(bytes).ok()?;
    let mut lines = text.lines();
    let location: PathBuf = path_of_hex(lines.next()?)?;
    let aside: PathBuf = path_of_hex(lines.next()?)?;
    return Some(KeptCopy { location, aside });
}

/// The record's file name for one copy,
/// which is a key of the private name it waits under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordName = (aside: string) => `${nameKey(hex(aside))}.copy`;
/// ```
fn record_path(git_dir: &Path, aside: &Path) -> PathBuf {
    return copy_directory(git_dir).join(format!("{}{RECORD_SUFFIX}", name_key(hex_of(aside).as_str())));
}

/// What:
///  Write the record of a copy a merge kept,
///  and flush the directory it went into.
/// Why:
///   The bytes of the copy are the person's,
/// and the record is what makes a later command find them;
///  a record a power loss dropped leaves the copy beside its path,
///  where the person still meets it in `git status`.
///  A record that cannot be written stops nothing:
/// the copy stays and the merge that kept it names it either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepTheCopy(gitDir, location, aside); // mkdir, writeExclusiveSynced, syncDirectory; log a failure
/// ```
pub(crate) fn keep_the_copy(git_dir: &Path, location: &Path, aside: &Path) {
    let directory: PathBuf = copy_directory(git_dir);
    match create_private_directory(directory.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            warn(TAG, format!("the record of {} was not written: {error}", aside.display()).as_str());
            return;
        }
    }
    let copy: KeptCopy = KeptCopy { location: location.to_path_buf(), aside: aside.to_path_buf() };
    // A record that is already there names the same copy, which a later command keeps again, and that is no failure.
    match write_exclusive_synced(record_path(git_dir, aside).as_path(), encode_copy(&copy).as_bytes()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            warn(TAG, format!("the record of {} was not written: {error}", aside.display()).as_str());
            return;
        }
    }
    if let Err(error) = sync_directory(directory.as_path()) {
        warn(TAG, format!("the record of {} was not flushed: {error}", aside.display()).as_str());
    }
}

/// What:
///  Forget a copy whose end has come,
///  so no later command looks for it.
/// Why:
///   A record that outlives its copy is one more name a person has to clean up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheCopy(gitDir, aside); // rm the record, ignoring a missing one
/// ```
pub(crate) fn forget_the_copy(git_dir: &Path, aside: &Path) {
    match std::fs::remove_file(record_path(git_dir, aside)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn(TAG, format!("the record of a kept copy was not removed: {error}").as_str()),
    }
}

/// What:
///  Every copy a merge kept in this worktree,
///  oldest record name first.
/// Why:
///   Each command decides each copy the same way,
/// and the order is only so two runs say the same thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptCopies(gitDir): KeptCopy[]; // readdir, read, decode; skip what cannot be read
/// ```
pub(crate) fn kept_copies(git_dir: &Path) -> Vec<KeptCopy> {
    let mut paths: Vec<PathBuf> = record_files(git_dir);
    paths.sort();
    let mut copies: Vec<KeptCopy> = Vec::with_capacity(paths.len());
    for path in paths {
        let Ok(bytes) = std::fs::read(path.as_path()) else { continue };
        if let Some(copy) = decode_copy(bytes.as_slice()) {
            copies.push(copy);
        }
    }
    return copies;
}

/// The record files of one worktree's kept copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(copyDirectory(gitDir), { withFileTypes: true })).filter((one) => one.name.endsWith('.copy'))
/// ```
fn record_files(git_dir: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(copy_directory(git_dir).as_path()) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = Vec::new();
    for item in listing.flatten() {
        if is_a_record(item.file_name().as_os_str()) {
            paths.push(item.path());
        }
    }
    return paths;
}

/// Whether one name in the copies' directory is a record of a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isARecord = (name: string) => name.endsWith('.copy');
/// ```
fn is_a_record(name: &std::ffi::OsStr) -> bool {
    return name.to_string_lossy().ends_with(RECORD_SUFFIX);
}

/// What:
///  Whether this worktree holds a record of a copy a merge kept.
/// Why:
///   The recovery of a worktree with no merge record is skipped for speed,
/// and a kept copy is the one thing it still has to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anyKeptCopy = (gitDir) => existsSync(copyDirectory(gitDir));
/// ```
pub(crate) fn any_kept_copy(git_dir: &Path) -> bool {
    return !record_files(git_dir).is_empty();
}


/// The record of one copy,
/// for a control that makes the state a killed command leaves.
#[cfg(test)]
pub(crate) fn record_of_a_control(git_dir: &Path, aside: &Path) -> PathBuf {
    return record_path(git_dir, aside);
}

/// The records of the copies a merge kept.
#[cfg(test)]
#[path = "merge_isolation_kept_copies_tests.rs"]
mod tests;
