//! What:
//!  Where the capture and the proof's phase marker live, how they are published
//!  so a kill leaves either the old record or the new one, and how an add that
//!  took the claim clears another add's records at its slot.
//! Why:
//!  The capture stands beside the binding in the wrapper's folder, keyed by the
//!  slot and owned by the add's nonce
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "What 'the captured source' is").
//!  The marker is a small file an add flips once the proof has passed, so a
//!  resume does not prove a tree that has already been written.
//!  Every publication is a private staging name, one exclusive link that
//!  replaces nothing, and the staging name's removal, which is the store
//!  precedent this repository uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! publishCapture(home, slot, capture); // staging, link, unlink staging
//! const found = readCapture(home, slot); // null: absent
//! ```

use super::add_capture::Capture;
use super::add_capture_record::{
    capture_bound,
    encode_capture,
    parse_capture,
};
use crate::private_storage::read_regular_file;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::ffi::OsStr;
use std::io::Read as _;
use std::os::unix::fs::{
    MetadataExt as _,
    OpenOptionsExt,
};
use std::path::{
    Path,
    PathBuf,
};

/// The suffix of the capture record.
pub const CAPTURE_SUFFIX: &str = ".capture";

/// The suffix of the private staging name one publication fills.
pub const CAPTURE_STAGING_SUFFIX: &str = ".capture.pending";

/// The suffix of the proof's phase marker.
pub const MARKER_SUFFIX: &str = ".capture.passed";

/// The suffix of the settle marker, which step 11 writes and this step clears
/// when it names another add.
pub const SETTLE_SUFFIX: &str = ".settle";

/// The capture's file name for one slot.
pub fn capture_name(slot: &str) -> String {
    return format!("{slot}{CAPTURE_SUFFIX}");
}

/// The staging name one capture publication fills.
pub fn capture_staging_name(slot: &str) -> String {
    return format!("{slot}{CAPTURE_STAGING_SUFFIX}");
}

/// The marker's file name for one slot.
pub fn marker_name(slot: &str) -> String {
    return format!("{slot}{MARKER_SUFFIX}");
}

/// The settle marker's file name for one slot.
pub fn settle_name(slot: &str) -> String {
    return format!("{slot}{SETTLE_SUFFIX}");
}

/// The mode of every record this store writes.
const RECORD_MODE: u32 = 0o600;

/// What:
///  Publish one capture under its real name, replacing nothing.
/// Why:
///  A capture belongs to one add's nonce and is written once per add, so a
///  second publication at the same name is a refusal and not a merge.
///
/// # Errors
/// The staging file cannot be made or written,
/// the link is refused,
/// or the staging name cannot be removed.
pub fn publish_capture(
    home: &std::fs::File,
    slot: &str,
    capture: &Capture,
) -> std::io::Result<()> {
    let bytes: Vec<u8> = encode_capture(capture);
    return publish_once(
        home,
        capture_staging_name(slot).as_str(),
        capture_name(slot).as_str(),
        bytes.as_slice(),
    );
}

/// What:
///  Read one slot's capture, or nothing when no capture stands.
/// Why:
///  A resume needs the facts, and a capture whose bytes do not fit the grammar
///  is refused by name rather than half read.
///
/// # Errors
/// The file is unreadable,
/// too large for its own count,
/// or not the grammar's.
pub fn read_capture(
    home: &Path,
    slot: &str,
) -> Result<Option<Capture>, String> {
    let path: PathBuf = home.join(capture_name(slot));
    let Some(prefix) = read_prefix(home, capture_name(slot).as_str())? else {
        return Ok(None);
    };
    let count: usize = count_from_prefix(prefix.as_slice())?;
    let limit: u64 = capture_bound(count);
    let bytes: Vec<u8> = read_regular_file(path.as_path(), limit).map_err(|refusal| {
        return format!(
            "the capture {} cannot be read ({refusal:?})",
            path.display()
        );
    })?;
    check_the_records_owner(path.as_path())?;
    let text: String = String::from_utf8(bytes)
        .map_err(|_not_text| return format!("the capture {} is not text", path.display()))?;
    return parse_capture(text.as_str()).map(Some).map_err(|refusal| return format!("the capture {} is refused: {refusal}. Ways forward: read it and delete it, then run the add again", path.display()));
}

/// What:
///  Publish the proof's phase marker for one nonce.
/// Why:
///  The marker says the clone's tree was proven before the first write, and its
///  nonce says whose proof it is.
///
/// # Errors
/// The staging file or the link was refused.
pub fn publish_marker(
    home: &std::fs::File,
    slot: &str,
    nonce: &RequestNonce,
) -> std::io::Result<()> {
    let bytes: Vec<u8> = marker_bytes(nonce);
    return publish_once(
        home,
        marker_staging_name(slot).as_str(),
        marker_name(slot).as_str(),
        bytes.as_slice(),
    );
}

/// The marker's whole content.
pub fn marker_bytes(nonce: &RequestNonce) -> Vec<u8> {
    return format!("nonce={}\n", nonce.as_str()).into_bytes();
}

/// The staging name one marker publication fills.
fn marker_staging_name(slot: &str) -> String {
    return format!("{slot}{MARKER_SUFFIX}.pending");
}

/// What:
///  Whether the marker at one slot names this add's nonce.
/// Why:
///  A resume reads the marker rather than the capture, and another add's marker
///  is no proof of this one's.
///
/// # Errors
/// The marker is unreadable or its content is not the one line.
pub fn marker_is_ours(
    home: &Path,
    slot: &str,
    nonce: &RequestNonce,
) -> Result<bool, String> {
    let path: PathBuf = home.join(marker_name(slot));
    let bytes: Vec<u8> =
        match read_regular_file(path.as_path(), marker_bytes(nonce).len() as u64 + 1) {
            Ok(found) => found,
            Err(crate::private_storage::ReadRefusal::Missing) => return Ok(false),
            Err(refusal) => {
                return Err(format!(
                    "the proof marker {} cannot be read ({refusal:?})",
                    path.display()
                ));
            }
        };
    if bytes != marker_bytes(nonce) {
        let found: String = String::from_utf8_lossy(bytes.as_slice()).trim().to_string();
        return Err(format!(
            "the proof marker {} says `{found}`, and this add's nonce is {}",
            path.display(),
            nonce.as_str()
        ));
    }
    return Ok(true);
}

/// What:
///  Remove one capture and its marker, in the design's order: the marker, then
///  the record, and the settle marker last when this step wrote one.
/// Why:
///  A kill at any removal leaves a state the resume can read, and a leftover
///  marker whose capture is gone is refused by the reader rather than trusted.
pub fn remove_capture(
    home: &Path,
    slot: &str,
) {
    let _marker: std::io::Result<()> = std::fs::remove_file(home.join(marker_name(slot)));
    let _capture: std::io::Result<()> = std::fs::remove_file(home.join(capture_name(slot)));
}

/// What:
///  Clear another add's capture, marker, settle marker and staging name at one
///  slot, and leave this add's own alone.
/// Why:
///  An add that has just taken the claim is the only owner of the slot, so a
///  record with another nonce was left by an add that is not running. The next
///  add clears it, which is what makes "remove it and run the same add again"
///  work after a pending worktree is removed.
///
/// # Errors
/// The folder cannot be listed,
/// or a record of another add cannot be removed.
pub fn clear_other_adds_records(
    home: &Path,
    slot: &str,
    nonce: &RequestNonce,
) -> Result<(), String> {
    let names: [String; 4] = [
        capture_name(slot),
        marker_name(slot),
        settle_name(slot),
        capture_staging_name(slot),
    ];
    for name in names {
        let path: PathBuf = home.join(name.as_str());
        let prefix: Vec<u8> = match prefix_of(path.as_path(), 4096) {
            Ok(found) => found,
            Err(refusal) if refusal.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "another add's record {} cannot be read ({error})",
                    path.display()
                ));
            }
        };
        let ours: bool = prefix
            .windows(nonce.as_str().len())
            .any(|window| return window == nonce.as_str().as_bytes());
        if ours {
            continue;
        }
        std::fs::remove_file(path.as_path()).map_err(|error| {
            return format!(
                "another add's record {} cannot be removed ({error})",
                path.display()
            );
        })?;
    }
    return Ok(());
}

/// Publish `bytes` under `staging` and then under `name`, replacing nothing.
fn publish_once(
    home: &std::fs::File,
    staging: &str,
    name: &str,
    bytes: &[u8],
) -> std::io::Result<()> {
    use std::io::Write as _;
    let staging_path: PathBuf = home_path(home, staging)?;
    let _stale: std::io::Result<()> = std::fs::remove_file(staging_path.as_path());
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(RECORD_MODE)
        .open(staging_path.as_path())?;
    file.write_all(bytes)?;
    file.sync_all()?;
    crate::fs_beneath::link_open_file_at(&file, home, OsStr::new(name))?;
    drop(file);
    std::fs::remove_file(staging_path.as_path())?;
    home.sync_all()?;
    return Ok(());
}

/// The path one name below the home has, read back from the held folder.
fn home_path(
    home: &std::fs::File,
    name: &str,
) -> std::io::Result<PathBuf> {
    return Ok(crate::fs_beneath::handle_path(home).join(name));
}

/// The first bytes of a capture file, enough for the header and the count.
fn read_prefix(
    home: &Path,
    name: &str,
) -> Result<Option<Vec<u8>>, String> {
    let path: PathBuf = home.join(name);
    let file: std::fs::File = match crate::private_storage::open_regular_file(path.as_path()) {
        Ok(opened) => opened,
        Err(crate::private_storage::ReadRefusal::Missing) => return Ok(None),
        Err(refusal) => {
            return Err(format!(
                "the capture {} cannot be read ({refusal:?})",
                path.display()
            ));
        }
    };
    // The prefix is the first bytes of a record whose whole length is not yet known.
    // It is taken from the open file rather than read as a whole.
    // `read_regular_file` refuses anything longer than its limit.
    // A capture of a few hundred tracked paths is longer than the prefix.
    let mut bytes: Vec<u8> = Vec::new();
    file.take(CAPTURE_PREFIX_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return format!("the capture {} cannot be read ({error})", path.display());
        })?;
    return Ok(Some(bytes));
}

/// The prefix bytes the reader takes before it knows the entry count.
/// Why:
///  The count sets the whole record's bound,
///  so it is read from a bounded prefix of a record whose length is unbounded
///  until that count is known.
const CAPTURE_PREFIX_BYTES: u64 = 8 * 1024;

/// The first `most` bytes of one file, however large the file is.
fn prefix_of(
    path: &Path,
    most: u64,
) -> std::io::Result<Vec<u8>> {
    let file: std::fs::File = std::fs::File::open(path)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(most).read_to_end(&mut bytes)?;
    return Ok(bytes);
}

/// Refuse a record another account owns or one this account's group or others
/// may write.
fn check_the_records_owner(path: &Path) -> Result<(), String> {
    let found: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(|error| {
        return format!(
            "the capture {} cannot be examined ({error})",
            path.display()
        );
    })?;
    // SAFETY: `geteuid` has no preconditions and reads this process's own identity.
    let ours: u32 = unsafe { libc::geteuid() };
    if found.uid() != ours {
        return Err(format!(
            "the capture {} belongs to account {}, and this is account {ours}; Ways forward: read it and delete it as its owner, then run the add again",
            path.display(),
            found.uid()
        ));
    }
    if found.mode() & 0o077 != 0 {
        return Err(format!(
            "the capture {} may be written by this account's group or others (mode {:o}); Ways forward: stop the account that could write it and delete the record, then run the add again",
            path.display(),
            found.mode() & 0o777
        ));
    }
    return Ok(());
}

/// The `entries=` count of a capture's prefix.
fn count_from_prefix(prefix: &[u8]) -> Result<usize, String> {
    let text: &str = std::str::from_utf8(prefix)
        .map_err(|_not_text| return String::from("the capture is not text"))?;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("entries=")
            && let Ok(count) = value.parse::<usize>()
        {
            return Ok(count);
        }
    }
    return Err(String::from(
        "the capture's prefix holds no `entries=` line, so its bound cannot be computed",
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_capture_store_tests.rs"]
mod tests;
