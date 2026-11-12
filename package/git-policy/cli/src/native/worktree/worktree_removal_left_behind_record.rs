//! What:
//!  The durable record of a folder a removal handed to Git:
//!  the entries the folder held before Git ran,
//!  the process that wrote the record,
//!  and whether Git had returned.
//! Why:
//!  Git deletes a worktree's registration whether or not its walk emptied the folder
//! (`builtin/worktree.c`,
//! `remove_worktree`:
//!  "continue on even if ret is non-zero, there's no going back from here"),
//!  and something writing into the folder while the walk runs makes it fail
//!  with `Directory not empty`.
//!  What the walk left is the folder the person asked to be gone,
//!  and nothing points at it any more,
//!  so the entries the folder held before Git ran are written down before it runs:
//!  those are the removal's own to delete,
//!  and every entry that is not in this list appeared while the removal was running.
//!  A process killed between the record and the end of the walk leaves the record,
//!  which is what the next command finishes the removal from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordBeforeGit(commonDir, id, target);  // owner, entries, path
//! recordReturned(commonDir, id);                  // Git came back
//! const record = readRecord(bytes);               // { owner, returned, seen, target }
//! forget(commonDir, id);
//! ```

use crate::fs_beneath::{entry_metadata_at, open_root};
use crate::plain_file::read_plain_file;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::process_identity::current_birth_identity;
use crate::subvolume_file_write_steps::write_beside_then_rename;
use crate::worktree_removal_interrupted_destroy::record_owner_may_be_running;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The folder of the common Git directory that holds one record per removal handed to Git,
/// named by the registration's id.
pub const RECORD_FOLDER: &str = "monochromatic-git-policy-cli/left-behind-removals";

/// The most bytes of a record that is read:
/// the two short lines,
/// then the entries and the path.
pub const RECORD_LIMIT: u64 = 4 * 1024 * 1024;

/// The prefix of the record's first line,
/// which names the process that wrote it.
const OWNER_PREFIX: &str = "owner=";

/// The line that says Git returned,
/// without its terminator.
const RETURNED_LINE: &[u8] = b"returned=1";

/// The path of the record of one registration.
fn record_path(common_dir: &Path, id: &OsStr) -> PathBuf {
    return common_dir.join(RECORD_FOLDER).join(id);
}

/// What:
///  What one record says.
/// Why:
///   The entries the folder held decide what a finish may delete,
///  the path decides which folder is finished,
///  and the owner line keeps a second command from taking a removal that is still walking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LeftBehindRecord = { owner: string | undefined; returned: boolean; seen: string[]; target: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeftBehindRecord {
    /// The process that wrote the record,
    /// and the identity that tells it from a later process of the same number;
    /// nothing for a record whose birth identity could not be read.
    pub owner: Option<String>,
    /// Whether Git had returned when the record was last written.
    pub returned: bool,
    /// The entries the folder held before Git ran,
    /// in the order they were listed.
    pub seen: Vec<OsString>,
    /// The folder the removal named.
    pub target: PathBuf,
}

/// What:
///  Record the folder a removal is about to hand to Git,
///  with the entries it holds now and the process writing the record.
/// Why:
///   The list separates the removal's own entries from what appeared while it ran,
///  so it is written before Git starts;
///  the owner line keeps a second command from finishing a removal that is still walking.
///  A record that cannot be written costs the automatic finish and nothing else,
///  so the caller warns and goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordBeforeGit(commonDir, id, target);
/// ```
pub fn record_before_git(common_dir: &Path, id: &OsStr, target: &Path) -> Result<(), String> {
    return record_before_git_with_owner(common_dir, id, target, None);
}

/// What:
///  `record_before_git` with the process the record names instead of this one,
/// so a control can write the record of a removal that ended.
///  `None` writes this process.
/// Why:
///   Recovery leaves a record alone while its owner may still be running,
///  and a control that simulates a killed removal has to write one whose owner is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordBeforeGitWithOwner(commonDir, id, target, '1:ended');
/// ```
pub fn record_before_git_with_owner(common_dir: &Path, id: &OsStr, target: &Path, owner: Option<&str>) -> Result<(), String> {
    let seen: Vec<OsString> = entries_of(target)?;
    let mut bytes: Vec<u8> = Vec::new();
    match owner {
        Some(spelled) => bytes.extend_from_slice(format!("{OWNER_PREFIX}{spelled}\n").as_bytes()),
        None => match current_birth_identity() {
            Ok(identity) => bytes.extend_from_slice(format!("{OWNER_PREFIX}{}:{identity}\n", std::process::id()).as_bytes()),
            // A record without an owner is read as one whose process proves nothing, which is what the recovery then assumes.
            Err(_unknown) => {}
        },
    }
    for name in &seen {
        bytes.extend_from_slice(name.as_os_str().as_bytes());
        bytes.push(0);
    }
    bytes.extend_from_slice(target.as_os_str().as_bytes());
    return write_record(common_dir, id, bytes.as_slice());
}

/// What:
///  The entries of one directory,
///  as they are now.
/// Why:
///   The record holds names and nothing else,
///  and a directory that cannot be listed is a record this program cannot write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entriesOf = (path) => readdir(path);
/// ```
pub fn entries_of(target: &Path) -> Result<Vec<OsString>, String> {
    let listing = std::fs::read_dir(target).map_err(|error| format!("{} cannot be listed ({error})", target.display()))?;
    let mut names: Vec<OsString> = Vec::new();
    for listed in listing {
        let found = listed.map_err(|error| format!("{} cannot be read ({error})", target.display()))?;
        names.push(found.file_name());
    }
    return Ok(names);
}

/// What:
///  Write the record's bytes as one whole file under the record's name.
/// Why:
///   The record is written whole and never edited in place,
///  so a command reads a record or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeRecord(commonDir, id, bytes);
/// ```
fn write_record(common_dir: &Path, id: &OsStr, bytes: &[u8]) -> Result<(), String> {
    let path: PathBuf = record_path(common_dir, id);
    if bytes.len() as u64 > RECORD_LIMIT {
        return Err(format!("the record {} would hold {} bytes, more than the {RECORD_LIMIT} a reader takes", path.display(), bytes.len()));
    }
    if let Some(parent) = path.parent()
        && let Err(error) = create_private_directory_all(parent)
    {
        return Err(format!("{} cannot be made ({error})", parent.display()));
    }
    return match write_private_file_unsynced(path.as_path(), bytes) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} cannot be written ({error})", path.display())),
    };
}

/// What:
///  Say that Git returned,
///  which is what proves its walk is over.
/// Why:
///   A record without this mark and with a living owner is a removal still running,
///  which no other command may finish;
///  with the mark the walk is over whatever the command that started it does next.
///  A record that cannot be marked stays a record the next command finishes only once the owner is proven gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordReturned(commonDir, id);
/// ```
pub fn record_returned(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    let Ok(bytes) = read_plain_file(path.as_path(), RECORD_LIMIT) else { return };
    let Ok(record) = read_record(bytes.as_slice()) else { return };
    let body: &[u8] = &bytes[header_length(bytes.as_slice())..];
    let mut marked: Vec<u8> = RETURNED_LINE.to_vec();
    marked.push(b'\n');
    if let Some(owner) = record.owner.as_ref() {
        marked.extend_from_slice(format!("{OWNER_PREFIX}{owner}\n").as_bytes());
    }
    marked.extend_from_slice(body);
    let _written = write_beside_then_rename(path.as_path(), marked.as_slice());
}

/// What:
///  Drop one record,
///  which had nothing left to finish.
/// Why:
///   The caller drops it only once the folder is gone,
///  or the removal is known not to have taken the registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forget(commonDir, id);
/// ```
pub fn forget(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_unremovable) => return,
    }
    // The folder goes when the last record does; one that still holds another record stays.
    let _kept = std::fs::remove_dir(common_dir.join(RECORD_FOLDER).as_path());
}

/// What:
///  How many bytes the record's two header lines take,
///  and nothing when the whole file is header.
/// Why:
///   The body is written back unchanged when the mark is added,
///  and both readers of the record must agree on where it starts.
///  A name cannot hold a zero byte,
///  so the entries need no escaping and the body is one run of bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const headerLength = (bytes) => headerLines(bytes).reduce((end, line) => end + line.length + 1, 0);
/// ```
fn header_length(bytes: &[u8]) -> usize {
    let mut rest: &[u8] = bytes;
    loop {
        // The body starts where this piece starts when the piece is no header line, which is also the answer for a file
        // whose first line never ends.
        let Some(split) = rest.iter().position(|byte: &u8| { return *byte == b'\n'; }) else { return bytes.len() - rest.len() };
        let (line, after): (&[u8], &[u8]) = rest.split_at(split);
        let is_returned: bool = line == RETURNED_LINE;
        let is_owner: bool = std::str::from_utf8(line).is_ok_and(|text: &str| { return text.starts_with(OWNER_PREFIX); });
        if !is_returned && !is_owner {
            return bytes.len() - rest.len();
        }
        rest = after.strip_prefix(b"\n").unwrap_or(after);
    }
}

/// What:
///  The record of one registration,
///  or the reason it cannot be used.
/// Why:
///   A reader that cannot parse a record cannot finish its removal,
///  so the reason is what the person reads instead of a silent nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readRecord(bytes); // throws the reason
/// ```
pub fn read_record(bytes: &[u8]) -> Result<LeftBehindRecord, String> {
    let mut rest: &[u8] = bytes;
    let mut owner: Option<String> = None;
    let mut returned: bool = false;
    // The header comes first, one line each, so no path and no entry can be taken for one of them.
    loop {
        let Some(split) = rest.iter().position(|byte: &u8| { return *byte == b'\n'; }) else { break };
        let (line, after): (&[u8], &[u8]) = rest.split_at(split);
        if line == RETURNED_LINE {
            returned = true;
            rest = after.strip_prefix(b"\n").unwrap_or(after);
            continue;
        }
        let Ok(text) = std::str::from_utf8(line) else { break };
        let Some(spelled) = text.strip_prefix(OWNER_PREFIX) else { break };
        owner = Some(spelled.to_string());
        rest = after.strip_prefix(b"\n").unwrap_or(after);
        break;
    }
    let mut pieces: Vec<&[u8]> = rest.split(|byte: &u8| { return *byte == 0; }).collect();
    let Some(target) = pieces.pop() else { return Err(String::from("it names no folder")) };
    if target.is_empty() {
        return Err(String::from("it names no folder"));
    }
    let seen: Vec<OsString> = pieces.iter().map(|name: &&[u8]| { return OsStr::from_bytes(name).to_os_string(); }).collect();
    return Ok(LeftBehindRecord { owner, returned, seen, target: PathBuf::from(OsStr::from_bytes(target)) });
}

/// What:
///  Whether the process that wrote a record may still be running,
/// which a record without an owner counts as.
/// Why:
///   A command that finishes a removal another command is still walking would delete what that command is still
///   proving,
///  and a removal whose record is dropped cannot be finished at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const running = ownerMayBeRunning(record.owner); // true when nothing proves the owner left
/// ```
pub fn owner_may_be_running(owner: Option<&String>) -> bool {
    return record_owner_may_be_running(owner);
}

/// What:
///  What the name `target` holds now,
///  read without following a link,
///  or nothing when it holds nothing.
/// Why:
///   The finish is about one directory,
///  so it asks the name the record carries,
///  and a folder replaced by a link or a file is not one this program removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = entryMetadataAt(openRoot(parent), name); // undefined when absent
/// ```
pub fn entry_kind(target: &Path) -> Result<Option<std::fs::Metadata>, String> {
    let Some(parent) = target.parent() else { return Err(format!("{} has no parent directory", target.display())) };
    let Some(name) = target.file_name() else { return Err(format!("{} has no last component", target.display())) };
    let held: std::fs::File = match open_root(parent) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("{} cannot be opened without following a link ({error})", parent.display())),
    };
    return match entry_metadata_at(&held, name) {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{} cannot be examined ({error})", target.display())),
    };
}

/// Controls of this module stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_left_behind_record_tests.rs"]
mod tests;
