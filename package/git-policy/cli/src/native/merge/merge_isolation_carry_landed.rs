//! What:
//!  What Git's own two-way update left at each path a merge changes,
//!  as the file system named it at the instant the checkout moved:
//!  the entry the path holds,
//!  and that entry's change time;
//!  and whether a path still holds exactly that entry.
//! Why:
//!  A carry decides from a path's bytes whether the file standing there is the merge's own,
//!  and a person's save whose bytes
//!  accept every line the merge brought answers yes,
//! so their newer decision was replaced with the older layer the merge captured
//!  (a cross-vendor review of the second round,
//! its second finding).
//!  The entry Git wrote is the only thing that tells the two apart:
//! the device and the inode name the file,
//!  and its size and modification time say whether it was written again.
//!  The change time is asked too,
//! because a write that keeps the length and puts the modification time back still changes it,
//! and a program that saves that way is not this merge's own step.
//!  A change time is not part of the entry's identity,
//! because a rename changes it:
//! every take compares the identity it read
//!  before the rename with the file afterwards,
//! so a change time there would make every take refuse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landed = landedOf(root, paths); const still = theFileGitLanded(landed, landedFileAt(location, path));
//! ```

use crate::merge_isolation_changes::{FileIdentity, WorktreeState, file_identity, worktree_path};

/// The version of the record of what Git's update left.
pub const LANDED_RECORD_SCHEMA_VERSION: i64 = 1;
use std::path::{Path, PathBuf};

/// One worktree path as the file system named it when Git's update finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedFile = { path: Uint8Array; identity?: FileIdentity; changedSeconds: bigint; changedNanos: bigint };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedFile {
    /// The path from the top of the worktree,
    /// exact bytes.
    pub path: Vec<u8>,
    /// The entry that stood there,
    /// or nothing when nothing did.
    pub identity: Option<FileIdentity>,
    /// The change time of that entry,
    /// whole seconds since the epoch.
    pub changed_seconds: i64,
    /// The change time's nanoseconds part.
    pub changed_nanos: i64,
}

/// The change time of one entry,
/// as the platform keeps it.
#[cfg(unix)]
fn changed_time_of(found: &std::fs::Metadata) -> (i64, i64) {
    use std::os::unix::fs::MetadataExt;
    return (found.ctime(), found.ctime_nsec());
}

/// The change time of one entry on a platform that keeps none.
#[cfg(not(unix))]
fn changed_time_of(found: &std::fs::Metadata) -> (i64, i64) {
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    let _ = found;
    return (0, 0);
}

/// What:
///  The entry at `location`,
///  named as the repository path `path`,
///  with the one thing a rename changes and a write cannot put back.
/// Why:
///   The identity and the change time are one read of the file system,
/// so the two comparisons that decide ownership cannot see two different files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function landedFileAt(location: string, path: Uint8Array): LandedFile; // lstat, no link followed
/// ```
pub fn landed_file_at(location: &Path, path: &[u8]) -> LandedFile {
    let found: Option<std::fs::Metadata> = std::fs::symlink_metadata(location).ok();
    let changed: (i64, i64) = found.as_ref().map(changed_time_of).unwrap_or_default();
    return LandedFile { path: path.to_vec(), identity: file_identity(location), changed_seconds: changed.0, changed_nanos: changed.1 };
}

/// What:
///  Read what Git's update left at each of these paths.
/// Why:
///   The merge's apply reads this once the checkout has moved and before the carry after it,
/// so a save that arrives in between is a file the
///  carry did not write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function landedOf(root: string, paths: Uint8Array[]): LandedFile[];
/// ```
pub fn landed_of(root: &Path, paths: &[Vec<u8>]) -> Vec<LandedFile> {
    let mut landed: Vec<LandedFile> = Vec::with_capacity(paths.len());
    for path in paths {
        let location: PathBuf = worktree_path(root, path.as_slice());
        landed.push(landed_file_at(location.as_path(), path.as_slice()));
    }
    return landed;
}

/// What:
///  The file that records what Git's own update left,
///  beside the merge's intent record.
/// Why:
///   The record is written once and never rewritten,
/// so what the checkout held after Git's update travels in a file of its
///  own,
/// and it is removed with the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(intentDirectory(gitDir), `${transactionId}.landed`)
/// ```
pub fn landed_record_path(git_dir: &Path, transaction_id: &str) -> PathBuf {
    return crate::merge_isolation_journal::intent_directory(git_dir).join(format!("{transaction_id}.landed"));
}

/// The hex text of one path,
/// which the record keeps so a path that is not text survives the round trip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hex = (bytes: Uint8Array) => Buffer.from(bytes).toString('hex');
/// ```
fn hex_of(bytes: &[u8]) -> String {
    let mut found: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        found.push_str(format!("{byte:02x}").as_str());
    }
    return found;
}

/// Whether one byte is one of the hex digits a record is written in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAHexDigit = (byte: number) => /[0-9a-f]/i.test(String.fromCharCode(byte));
/// ```
fn is_a_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// The bytes one hex text names,
/// or nothing when the text is not an even run of hex digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytesOfHex = (hex: string) => Buffer.from(hex, 'hex');
/// ```
fn bytes_of_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || !text.bytes().all(is_a_hex_digit) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.as_bytes().as_chunks::<2>().0 {
        let high: u32 = u32::from((pair[0] as char).to_digit(16)?);
        let low: u32 = u32::from((pair[1] as char).to_digit(16)?);
        bytes.push(u8::try_from((high << 4) | low).ok()?);
    }
    return Some(bytes);
}

/// The record's name for a number,
/// which is written as text so a device number beyond a double's exact range survives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// String(value)
/// ```
fn number_text(value: i64) -> String {
    return value.to_string();
}

/// Whole-number field of the landed record,
/// or nothing when it is not one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numberField = (map, key) => Number(map[key]);
/// ```
fn number_field(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<i64> {
    return crate::json_record::string_field(map, key)?.parse::<i64>().ok();
}

/// One landed path as the record holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify(entry)}`
/// ```
fn encode_one(landed: &LandedFile) -> String {
    let mut writer: crate::json_record::ObjectWriter = crate::json_record::ObjectWriter::new();
    writer
        .string("pathHex", hex_of(landed.path.as_slice()).as_str())
        .string("path", crate::path_bytes::path_message_text(landed.path.as_slice()).as_str())
        .string("holds", if landed.identity.is_some() { "file" } else { "nothing" })
        .string("changedSeconds", number_text(landed.changed_seconds).as_str())
        .string("changedNanos", number_text(landed.changed_nanos).as_str());
    if let Some(identity) = landed.identity {
        writer
            .string("device", number_text(i64::try_from(identity.device).unwrap_or(i64::MAX)).as_str())
            .string("inode", number_text(i64::try_from(identity.inode).unwrap_or(i64::MAX)).as_str())
            .string("size", number_text(i64::try_from(identity.size).unwrap_or(i64::MAX)).as_str())
            .string("mode", number_text(i64::from(identity.mode)).as_str())
            .string("modifiedSeconds", number_text(identity.modified_seconds).as_str())
            .string("modifiedNanos", number_text(identity.modified_nanos).as_str());
    }
    return format!("{{{}}}", writer.finish().trim_start_matches('{').trim_end_matches('}'));
}

/// What:
///  Encode what Git's update left at each path,
///  so a recovery reads the same entries the command that moved the checkout read.
/// Why:
///   A killed command's recovery has no live reading of its own,
/// and a person's save whose bytes accept
///  every line the merge brought looks exactly like the merge's own file without one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify({ schemaVersion: 1, state: 'merge-landed', transactionId, paths: landed })}\n`
/// ```
pub fn encode_landed_record(transaction_id: &str, landed: &[LandedFile]) -> String {
    let mut writer: crate::json_record::ObjectWriter = crate::json_record::ObjectWriter::new();
    writer
        .integer("schemaVersion", LANDED_RECORD_SCHEMA_VERSION)
        .string("state", "merge-landed")
        .string("transactionId", transaction_id);
    let mut paths: Vec<String> = Vec::with_capacity(landed.len());
    for entry in landed {
        paths.push(encode_one(entry));
    }
    writer.raw("paths", format!("[{}]", paths.join(",")).as_str());
    return format!("{}\n", writer.finish());
}

/// What:
///  Parse one landed record,
///  or nothing when it is not one.
/// Why:
///   A record that cannot be read proves nothing,
/// so the recovery reads the checkout by bytes alone,
/// which is what a
///  command without this record has always done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLandedRecord(bytes: Uint8Array): LandedFile[] | undefined;
/// ```
pub fn parse_landed_record(bytes: &[u8]) -> Option<Vec<LandedFile>> {
    let text: &str = crate::json_record::decode_fatal(bytes)?;
    let map: serde_json::Map<String, serde_json::Value> = crate::json_record::parse_object(text)?;
    if !crate::json_record::number_equals(&map, "schemaVersion", LANDED_RECORD_SCHEMA_VERSION)
        || crate::json_record::string_field(&map, "state")? != "merge-landed"
    {
        return None;
    }
    let serde_json::Value::Array(listed) = map.get("paths")? else { return None };
    let mut landed: Vec<LandedFile> = Vec::with_capacity(listed.len());
    for value in listed {
        let serde_json::Value::Object(entry) = value else { return None };
        let path: Vec<u8> = bytes_of_hex(crate::json_record::string_field(entry, "pathHex")?)?;
        let changed_seconds: i64 = number_field(entry, "changedSeconds")?;
        let changed_nanos: i64 = number_field(entry, "changedNanos")?;
        let identity: Option<FileIdentity> = match crate::json_record::string_field(entry, "holds")? {
            "nothing" => None,
            "file" => Some(FileIdentity {
                device: u64::try_from(number_field(entry, "device")?).ok()?,
                inode: u64::try_from(number_field(entry, "inode")?).ok()?,
                size: u64::try_from(number_field(entry, "size")?).ok()?,
                mode: u32::try_from(number_field(entry, "mode")?).ok()?,
                modified_seconds: number_field(entry, "modifiedSeconds")?,
                modified_nanos: number_field(entry, "modifiedNanos")?,
            }),
            _ => return None,
        };
        landed.push(LandedFile { path, identity, changed_seconds, changed_nanos });
    }
    return Some(landed);
}

/// What:
///  Write what Git's update left,
///  in the place the recovery of a killed merge reads it.
/// Why:
///   The write is the only thing between a kill and a save being read as the merge's own,
/// so it happens
///  after the last path Git wrote and before the carry;
/// it is not flushed,
///  because a kill leaves the
///  page cache intact and a power loss is the one case this record is only a
///  help in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileExclusiveSynced(partial, encodeLandedRecord(id, landed)); rename(partial, landedRecordPath(gitDir, id));
/// ```
///  The record is made under a private name beside its own and renamed onto it,
/// so a kill inside the write leaves either no record
///  at all,
///  which is the state a power loss leaves and the recovery reads by bytes,
/// or the whole record;
///  a record
///  a kill cut in half names paths whose entries it never finished writing,
/// and a reading that cannot be read is the same
///  as none.
/// The private name carries the record's own name,
/// so the removal of the record removes it too.
pub fn write_landed_record(git_dir: &Path, transaction_id: &str, landed: &[LandedFile]) {
    let path: PathBuf = landed_record_path(git_dir, transaction_id);
    if let Some(parent) = path.parent() {
        let _made = std::fs::create_dir_all(parent);
    }
    let bytes: String = encode_landed_record(transaction_id, landed);
    let partial: PathBuf = partial_landed_record_path(git_dir, transaction_id);
    // A kill inside an earlier write of this same record leaves its private name; it holds no reading this command wants.
    let _stale = std::fs::remove_file(partial.as_path());
    if let Err(error) = crate::private_storage::write_exclusive_synced(partial.as_path(), bytes.as_bytes()) {
        crate::diagnostic_log::warn("mergeCarry", format!("the landed reading was not written: {error}").as_str());
        return;
    }
    if let Err(error) = std::fs::rename(partial.as_path(), path.as_path()) {
        let _unwritten = std::fs::remove_file(partial.as_path());
        crate::diagnostic_log::warn("mergeCarry", format!("the landed reading was not written: {error}").as_str());
    }
}

/// What:
///  The private name the record is made under before it takes its own,
///  beside it.
/// Why:
///  A rename inside one directory is one step on whatever file system the Git directory is on,
/// so a kill
///  leaves either the old record or the whole new one;
/// the name it is made under is the record's own name with a suffix,
///  so the removal of the record removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(intentDirectory(gitDir), `${transactionId}.landed.writing`)
/// ```
pub fn partial_landed_record_path(git_dir: &Path, transaction_id: &str) -> PathBuf {
    return crate::merge_isolation_journal::intent_directory(git_dir).join(format!("{transaction_id}.landed.writing"));
}

/// What:
///  Read what the command that moved the checkout left at each path,
///  or nothing when no command wrote it.
/// Why:
///   A merge killed after Git's update has no live reading,
/// and a reading that cannot be read is the same
///  as none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landed = readFileSync(landedRecordPath(gitDir, id)).then(parseLandedRecord) ?? [];
/// ```
pub fn read_landed_record(git_dir: &Path, transaction_id: &str) -> Vec<LandedFile> {
    let Ok(bytes) = std::fs::read(landed_record_path(git_dir, transaction_id).as_path()) else {
        return Vec::new();
    };
    return parse_landed_record(bytes.as_slice()).unwrap_or_default();
}

/// What:
///  Whether the reading names this path,
///  which is what makes the entry standing there provable.
/// Why:
///   An entry the reading does not name was read by nothing,
/// so its bytes are all the evidence there is;
///  a byte
///  comparison cannot tell Git's own write from a save whose bytes are the same,
/// and the caller that must keep both
///  needs the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theReadingNamesThePath = (landed, path) => landed.some((one) => sameBytes(one.path, path));
/// ```
pub fn the_reading_names_the_path(landed: &[LandedFile], path: &[u8]) -> bool {
    for left in landed {
        if left.path == path {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the file standing at a path is still the one Git's own update wrote there.
/// Why:
///   The identity is the device,
/// the inode,
/// the size and the modification time of the entry the file system hands out,
/// and the
///  change time says whether it was written again at all,
/// so a save from an editor is a different entry even when it accepts every
///  line the merge brought.
/// A path the landed reading does not name was read by nothing,
/// so its bytes are all the evidence
///  there is,
/// which is what a recovery of a killed merge has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theFileGitLanded = (landed: LandedFile[], standing: LandedFile): boolean => landed.find((left) => sameBytes(left.path, standing.path)) === undefined ? true : deepEqual(left, standing);
/// ```
pub fn the_file_git_landed(landed: &[LandedFile], standing: &LandedFile) -> bool {
    for left in landed {
        if left.path == standing.path {
            return left == standing;
        }
    }
    return true;
}

/// The record of what Git's update left.
#[cfg(test)]
#[path = "merge_isolation_carry_landed_tests.rs"]
mod tests;

/// What:
///  Whether the bytes a carry would write at a path stay beside it instead,
///  because nothing proves the file standing there is the one Git's update wrote.
/// Why:
///   A file whose bytes are exactly the merge's own is what Git's update wrote,
/// and also what a save made after a
///  killed merge holds,
/// so the reading is the only thing that tells them apart.
/// Where the reading does not name the path,
/// the
///  file is left as it is and the bytes the carry planned for it are kept beside it and named,
/// so the person's
///  newest bytes are never replaced by the older layer the merge captured
/// (`SPEC.md`,
/// the copy whose bytes the file
///  at its path holds).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theWriteIsUnproven = (landed, path, state, merged) => !landed.some((one) => sameBytes(one.path, path)) && state.kind === 'content' && state.object === merged;
/// ```
pub fn the_write_is_unproven(landed: &[LandedFile], path: &[u8], state: &WorktreeState, merged: Option<&str>) -> bool {
    if the_reading_names_the_path(landed, path) {
        return false;
    }
    return match (state, merged) {
        (WorktreeState::Content { object, .. }, Some(found)) => object == found,
        _ => false,
    };
}
