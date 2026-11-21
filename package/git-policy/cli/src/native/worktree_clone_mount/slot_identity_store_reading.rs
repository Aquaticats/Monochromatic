//! What:
//!  Reading one record file below a held store folder,
//!  listing the folder's slot names,
//!  and removing the staging names a killed writer left.
//! Why:
//!  One read is one file:
//!  an `openat` on the held folder,
//!  a check of that descriptor's owner and mode,
//!  a bounded read,
//!  a UTF-8 check and the two lines' parse.
//!  Nothing here lists the folder for a single read,
//!  so a reader pays nothing for the number of slots,
//!  and the listing exists only for the adoption and `doctor`,
//!  which need the whole set.
//!  A staging name is never read as a record,
//!  and only the two forms this layout's writers make are removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = readRecord(folder, 'add-cache-layer'); // undefined when the record is absent
//! const names = listedSlotNames(folder); const removed = removeKnownStagingEntries(folder);
//! ```

use crate::entries_of_one_open_folder::{entry_names, open_entry_for_reading, read_entry_bytes};
use crate::publishing_entries_in_one_open_folder::{remove_entry, sync_folder};
use super::slot_identity_store_control::check_under_root_control;
use super::slot_identity_store_records::{PARENT_RECORD_NAME, SLOT_NAME_MAXIMUM_BYTES, StoredIdentity, is_a_dot_name, is_known_staging_name, parse_one_record, slot_name_refusal};
use super::slot_identity_store_refusals::{StoreRefusal, unreadable, write_refusal};
use crate::private_storage::ReadRefusal;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The largest one record file may hold,
/// derived from the two kernel limits rather than chosen.
///
/// Why:
///  The widest identity token is two ACLs of `MAXIMUM_ACL_BYTES` at two hexadecimal characters a byte,
///  plus the device,
///  inode,
///  owner and mode fields and five colons;
///  the widest record line is `slot `,
///  a name of at most `SLOT_NAME_MAXIMUM_BYTES` bytes,
///  a space,
///  that token and its newline;
///  and the widest file is that line and the header with its newline.
///  A folder whose ACL the kernel would not hold never reaches a record,
///  because the capture refuses it (`read_acl`),
///  so no record a writer of this repository makes is refused by this bound.
///  What the bound is for is a runaway or replaced file,
/// which must be named rather than read into memory.
pub const RECORD_MAXIMUM_BYTES: u64 = header_line_bytes() + slot_line_prefix_bytes() + SLOT_NAME_MAXIMUM_BYTES + 1 + widest_identity_token_bytes() + 1;

/// The header line and its newline.
const fn header_line_bytes() -> u64 {
    return super::slot_identity_store_records::STORE_HEADER.len() as u64 + 1;
}

/// `slot ` and the space after the name.
const fn slot_line_prefix_bytes() -> u64 {
    return "slot ".len() as u64;
}

/// The widest identity token the capture can produce.
const fn widest_identity_token_bytes() -> u64 {
    let hexadecimal_digits_per_byte: u64 = 2;
    let acls_per_record: u64 = 2;
    let five_colons: u64 = 5;
    let device_digits: u64 = 20;
    let inode_digits: u64 = 20;
    let owner_digits: u64 = 10;
    let mode_digits: u64 = 4;
    return acls_per_record * (super::clone_folder_identity_acl::MAXIMUM_ACL_BYTES as u64) * hexadecimal_digits_per_byte + five_colons + device_digits + inode_digits + owner_digits + mode_digits;
}

/// The path a refusal names for the record filed under `name`.
pub(super) fn record_path(folder_path: &Path, name: &str) -> PathBuf {
    return folder_path.join(name);
}

/// What:
///  The record `name` holds below `folder`,
///  or nothing when no record is filed under that name.
/// Why:
///  Every reader of one record goes through this call,
///  so a partial file,
///  a file filed under another name and a file this account could rewrite are refused in one place.
///  The control terms apply to the record's own descriptor when the caller holds the folder under root control,
///  so a swap between the name and the read cannot change which object was judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = readRecord(folder, name, { underRootControl: true }); // undefined when absent
/// ```
///
/// # Errors
/// A record larger than `RECORD_MAXIMUM_BYTES`,
/// a link,
/// a folder or another non-regular file at the name,
/// text that is not UTF-8,
/// a line this repository did not write,
/// a key the file's name does not belong to,
/// or a record this account could rewrite.
pub(super) fn read_record(folder: &std::fs::File, folder_path: &Path, name: &str, under_root_control: bool) -> Result<Option<StoredIdentity>, StoreRefusal> {
    let path: PathBuf = record_path(folder_path, name);
    let mut file: std::fs::File = match open_entry_for_reading(folder, OsStr::new(name)) {
        Ok(opened) => opened,
        Err(ReadRefusal::Missing) => return Ok(None),
        Err(refusal) => return Err(unreadable(path.as_path(), refusal, RECORD_MAXIMUM_BYTES)),
    };
    if under_root_control {
        check_under_root_control(path.as_path(), "the record file", &file)?;
    }
    let held: u64 = file.metadata().map_err(|error| return StoreRefusal::Unreadable { path: path.clone(), reason: error.to_string() })?.len();
    if held > RECORD_MAXIMUM_BYTES {
        return Err(StoreRefusal::RecordTooLarge { path, bytes: held, bound: RECORD_MAXIMUM_BYTES });
    }
    let bytes: Vec<u8> = read_entry_bytes(&mut file, RECORD_MAXIMUM_BYTES).map_err(|refusal| return unreadable(path.as_path(), refusal, RECORD_MAXIMUM_BYTES))?;
    let text: String = String::from_utf8(bytes).map_err(|_error| return StoreRefusal::Unreadable { path: path.clone(), reason: String::from("the file is not UTF-8 text") })?;
    return parse_one_record(path.as_path(), text.as_str(), name).map(|record| return Some(record));
}

/// What:
///  Every slot name the folder lists,
///  in one order two listings can be compared in.
/// Why:
///  The adoption needs the whole set and refuses a name that is neither `.parent`,
///  nor a slot name,
///  nor a staging entry;
///  a foreign name could be a slot whose record a later read would miss.
///  Staging entries and `.parent` are not slots and are left out,
///  and an unknown dot name is left out too,
///  because `doctor` is what reports it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = listedSlotNames(folder); // sorted, foreign names refused
/// ```
///
/// # Errors
/// A folder that could not be listed,
/// a name that is not UTF-8,
/// or an entry that is none of the three kinds this layout names.
pub(super) fn listed_slot_names(folder: &std::fs::File, folder_path: &Path) -> Result<Vec<String>, StoreRefusal> {
    let entries: Vec<OsString> = entry_names(folder).map_err(|error| return StoreRefusal::Unreadable { path: folder_path.to_path_buf(), reason: error.to_string() })?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let Some(name) = entry.to_str() else {
            return Err(StoreRefusal::ForeignEntry { path: folder_path.join(entry.as_os_str()), entry: entry.to_string_lossy().into_owned() });
        };
        if name == PARENT_RECORD_NAME || is_a_dot_name(name) {
            continue;
        }
        if slot_name_refusal(name).is_some() {
            return Err(StoreRefusal::ForeignEntry { path: folder_path.join(name), entry: name.to_string() });
        }
        names.push(name.to_string());
    }
    names.sort();
    names.dedup();
    return Ok(names);
}

/// What:
///  Remove every staging name of the two forms this layout's writers make,
///  and answer their names.
/// Why:
///  A writer killed between its bytes and its link leaves one,
///  and a writer killed between the link and the removal of its own staging name leaves one naming a published record too;
///  a run that lists the folder anyway removes them,
///  and a dot entry matching neither form is left in place for an administrator,
///  because a third form means a writer this layout has not accounted for.
///  The caller's own precondition is what makes the removal safe:
///  an adoption runs with managed operations quiesced,
///  and a sweep run removed a live writer's staging name only at the cost of that writer's own refusal to publish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = removeKnownStagingEntries(folder); // ['..parent.3f.partial', '.a1b2...']
/// ```
///
/// # Errors
/// A folder that could not be listed,
/// and a removal the system refused.
pub(super) fn remove_known_staging_entries(folder: &std::fs::File, folder_path: &Path) -> Result<Vec<String>, StoreRefusal> {
    let entries: Vec<OsString> = entry_names(folder).map_err(|error| return StoreRefusal::Unreadable { path: folder_path.to_path_buf(), reason: error.to_string() })?;
    let mut removed: Vec<String> = Vec::new();
    for entry in entries {
        let Some(name) = entry.to_str() else {
            continue;
        };
        if !is_known_staging_name(name) {
            continue;
        }
        match remove_entry(folder, entry.as_os_str()) {
            Ok(()) => removed.push(name.to_string()),
            // A name that went between the listing and the removal is the end state this call asks for.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(write_refusal(folder_path.join(name).as_path(), error)),
        }
    }
    if removed.is_empty() {
        return Ok(removed);
    }
    sync_folder(folder).map_err(|error| return write_refusal(folder_path, error))?;
    removed.sort();
    return Ok(removed);
}
