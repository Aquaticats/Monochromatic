//! What:
//!  The read of one attempt's entry:
//!  the file opened as a plain file with no link followed,
//!  proved regular,
//!  the account's own and private,
//!  bounded to one page,
//!  and parsed by the entry's own grammar.
//! Why:
//!  A command reads this folder before it runs,
//!  so a link,
//!  a pipe or a device at an entry's name could lead it elsewhere or stop it
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!  The bound is the start-time gate's per-file bound,
//!  which is one page,
//!  and every field of an entry is bounded,
//!  so a longer file is corrupt rather than a long request.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entry = readOne(folder, name); // no link, one page, the account's own
//! ```

use crate::worktree_clone_mount_protocol::unanswered_request_entry::{
    MAXIMUM_ENTRY_BYTES,
    RequestEntry,
    parse_entry,
};
use crate::worktree_clone_mount_protocol::unanswered_request_names::EntryName;
use crate::worktree_clone_mount_protocol::unanswered_requests_refusals::RequestsRefusal;
use std::fs::Metadata;
use std::io::Read;
use std::os::unix::fs::{
    MetadataExt,
    OpenOptionsExt,
};
use std::path::{
    Path,
    PathBuf,
};

/// The bits a file of the account's own may not carry.
const OTHER_WRITERS: u32 = 0o077;

/// Read one attempt's entry of one folder.
///
/// # Errors
/// The file is not a regular file of the account's own,
/// it is larger than one page,
/// or its text is not one entry.
pub(crate) fn read_one(
    directory: &Path,
    account: u32,
    name: EntryName,
) -> Result<RequestEntry, RequestsRefusal> {
    let path: PathBuf = directory.join(name.file_name());
    let refusal = |reason: String| {
        return RequestsRefusal::File {
            path: path.clone(),
            reason,
        };
    };
    let file: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        // The entry is read as a plain file: a link, a pipe or a device would let a reader wait or be led
        // somewhere else, which the pending check's own read forbids as well.
        .open(path.as_path())
        .map_err(|error| return refusal(error.to_string()))?;
    let facts: Metadata = file
        .metadata()
        .map_err(|error| return refusal(error.to_string()))?;
    if !facts.is_file() {
        return Err(refusal(String::from("it is not a regular file")));
    }
    if facts.uid() != account {
        return Err(refusal(format!(
            "it belongs to account {}, and this account is {account}",
            facts.uid()
        )));
    }
    if facts.mode() & OTHER_WRITERS != 0 {
        return Err(refusal(format!(
            "its mode is {:o}, and only the account may read or write it",
            facts.mode() & 0o777
        )));
    }
    if facts.len() > MAXIMUM_ENTRY_BYTES as u64 {
        return Err(refusal(format!(
            "it holds {} bytes, and one entry may take at most {MAXIMUM_ENTRY_BYTES}",
            facts.len()
        )));
    }
    let mut bytes: Vec<u8> = Vec::<u8>::with_capacity(MAXIMUM_ENTRY_BYTES);
    file.take(MAXIMUM_ENTRY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| return refusal(error.to_string()))?;
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    return parse_entry(text.as_ref()).map_err(|entry_refusal| {
        return RequestsRefusal::Text {
            path: path.clone(),
            refusal: entry_refusal,
        };
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_requests_one_entry_read_tests.rs"]
mod tests;
