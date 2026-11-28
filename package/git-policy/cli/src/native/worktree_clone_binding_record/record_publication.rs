//! What:
//!  Publishing the binding and the two inventories,
//!  and reading them back with the refusals a reader owes a record it did not write.
//! Why:
//!  The binding is replaced whole by one rename,
//!  so a reader sees either the record that was there or the complete new one,
//!  and it is flushed at every write,
//!  because its identities are already durable in the pool
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "Publication,
//!  and what a kill leaves" and "Durability").
//!  Each inventory is published once,
//!  with an exclusive creation and one link,
//!  so a second publication refuses instead of merging two histories.
//!  A reader refuses a record another account owns,
//!  one another account could rewrite,
//!  one longer than its bound,
//!  and one the parser cannot read,
//!  by name,
//!  and changes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await publishReplacing(home, `${slot}.binding`, record.encode());
//! await publishOnce(home, `${slot}.origin-inventory`, inventory.encode());
//! const binding = await readBinding(home, slot);
//! ```

use super::binding_bound::BINDING_MAXIMUM_BYTES;
use super::binding_inventory::{
    InventoryKind,
    InventoryRecord,
};
use super::binding_inventory_parse::parse_inventory;
use super::binding_parse::parse_binding;
use super::binding_record::BindingRecord;
use super::record_refusal::RecordRefusal;
use super::registration_home::{
    binding_name,
    clone_inventory_name,
    origin_inventory_name,
    record_staging_name,
};
use crate::earlier_service_process_look::effective_user;
use crate::private_storage::{
    PRIVATE_FILE_MODE,
    ReadRefusal,
    open_regular_file,
};
use crate::publishing_entries_in_one_open_folder::{
    create_new_entry,
    fill_entry_and_sync,
    link_entry,
    remove_entry,
    rename_entry_replacing,
    sync_folder,
};
use std::ffi::OsStr;
use std::io::Read as _;
use std::path::Path;

/// The bytes a staging name's random part holds.
const STAGING_RANDOM_BYTES: usize = 16;

/// What:
///  Why a record could not be read,
///  or written.
/// Why:
///  Every reader prints one of these and acts on none of them,
///  and the writing arm exists because a full disk or a replaced folder is a normal failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'missing' | 'not-regular' | 'too-large' | 'another-account' | 'writable-by-others' | 'unreadable'; message: string };
/// ```
#[derive(Debug)]
pub enum RecordRefusalAtTheFile {
    /// Nothing exists at the record's name.
    Missing,
    /// The name holds a link,
    /// a directory or another non-regular file.
    NotRegular,
    /// The file is longer than its grammar allows.
    TooLarge {
        /// The bytes found.
        found: u64,
        /// The bound they passed.
        bound: u64,
    },
    /// The file belongs to another account.
    AnotherAccount {
        /// The owner the file records.
        owner: u32,
        /// The account reading it.
        reader: u32,
    },
    /// Another account could write the file.
    WritableByOthers {
        /// The mode the file records.
        mode: u32,
    },
    /// The operating system refused the read.
    Io(String),
    /// The bytes are not a record of this grammar.
    Record(RecordRefusal),
}

impl RecordRefusalAtTheFile {
    /// The sentence a person reads,
    /// naming the file and the way forward.
    pub fn message(
        &self,
        path: &Path,
    ) -> String {
        let shown: String = path.display().to_string();
        return match self {
            RecordRefusalAtTheFile::Missing => format!(
                "{shown} does not exist, so the record it should hold was not read. Nothing was changed. Ways forward: run the add again at the slot, which writes a record when none stands."
            ),
            RecordRefusalAtTheFile::NotRegular => format!(
                "{shown} is a link, a directory or another non-regular file, and a record must be a private file. Nothing was changed. Ways forward: inspect the name, remove what is not the add's record, and run the command again."
            ),
            RecordRefusalAtTheFile::TooLarge {
                found,
                bound,
            } => format!(
                "{shown} holds {found} bytes and the widest record its grammar allows is {bound} bytes, so it was not read. Nothing was changed. Ways forward: read the file, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusalAtTheFile::AnotherAccount {
                owner,
                reader,
            } => format!(
                "{shown} belongs to account {owner} and this command runs as account {reader}, so the record is not this account's to act on. Nothing was changed. Ways forward: a person reads the file and decides, because this route never changes another account's record."
            ),
            RecordRefusalAtTheFile::WritableByOthers {
                mode,
            } => format!(
                "{shown} has mode {mode:04o}, which lets another account write it, so nothing in it can be trusted as this add's record. Nothing was changed. Ways forward: a person compares the file with the record the add wrote, restores it from a copy, or deletes it and runs the command again."
            ),
            RecordRefusalAtTheFile::Io(reason) => format!(
                "{shown} could not be read ({reason}), so the record was not read. Nothing was changed. Ways forward: inspect the file and the disk, then run the command again."
            ),
            RecordRefusalAtTheFile::Record(refusal) => format!("{shown}: {}", refusal.message()),
        };
    }
}

/// The recorded file and its handle's metadata,
/// after the account and mode checks a record owes.
fn open_record(path: &Path) -> Result<std::fs::File, RecordRefusalAtTheFile> {
    let file: std::fs::File = match open_regular_file(path) {
        Ok(opened) => opened,
        Err(ReadRefusal::Missing) => return Err(RecordRefusalAtTheFile::Missing),
        Err(ReadRefusal::NotRegular) => return Err(RecordRefusalAtTheFile::NotRegular),
        Err(ReadRefusal::TooLarge) => {
            return Err(RecordRefusalAtTheFile::Io(String::from(
                "the file is larger than the read allows",
            )));
        }
        Err(ReadRefusal::Io(_reason)) if !path.exists() => {
            return Err(RecordRefusalAtTheFile::Missing);
        }
        Err(ReadRefusal::Io(reason)) => return Err(RecordRefusalAtTheFile::Io(reason.to_string())),
    };
    let metadata: std::fs::Metadata = file
        .metadata()
        .map_err(|error| return RecordRefusalAtTheFile::Io(error.to_string()))?;
    /// The traits that read an owner and a mode from Unix metadata.
    use std::os::unix::fs::MetadataExt as _;
    let reader: u32 = effective_user();
    if metadata.uid() != reader {
        return Err(RecordRefusalAtTheFile::AnotherAccount {
            owner: metadata.uid(),
            reader,
        });
    }
    let mode: u32 = metadata.mode();
    if mode & 0o022 != 0 {
        return Err(RecordRefusalAtTheFile::WritableByOthers {
            mode,
        });
    }
    return Ok(file);
}

/// Read one record's bytes,
/// bounded by the grammar that will read them.
fn read_bytes(
    path: &Path,
    bound: u64,
) -> Result<Vec<u8>, RecordRefusalAtTheFile> {
    let mut file: std::fs::File = open_record(path)?;
    let found: u64 = file
        .metadata()
        .map_err(|error| return RecordRefusalAtTheFile::Io(error.to_string()))?
        .len();
    if found > bound {
        return Err(RecordRefusalAtTheFile::TooLarge {
            found,
            bound,
        });
    }
    // One byte past the bound reads too, so a file that grew after its size was read is still refused.
    let mut bytes: Vec<u8> = Vec::new();
    file.by_ref()
        .take(bound + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| return RecordRefusalAtTheFile::Io(error.to_string()))?;
    if bytes.len() as u64 > bound {
        return Err(RecordRefusalAtTheFile::TooLarge {
            found: bytes.len() as u64,
            bound,
        });
    }
    return Ok(bytes);
}

/// What:
///  Read one slot's binding.
/// Why:
///  The add's resume,
///  the removal and `doctor` read one record through this call,
///  so the account,
///  mode and size checks are made once.
///
/// # Errors
/// The file's own refusal,
/// or the grammar's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binding = await readBinding(home, slot);
/// ```
pub fn read_binding(
    home: &Path,
    slot: &str,
) -> Result<BindingRecord, RecordRefusalAtTheFile> {
    let path: std::path::PathBuf = home.join(binding_name(slot));
    let bytes: Vec<u8> = read_bytes(path.as_path(), BINDING_MAXIMUM_BYTES)?;
    return parse_binding(&bytes).map_err(RecordRefusalAtTheFile::Record);
}

/// Read one slot's inventory,
/// as the kind its name says it is.
///
/// # Errors
/// The file's own refusal,
/// the grammar's,
/// or a kind that disagrees with the name.
pub fn read_inventory(
    home: &Path,
    slot: &str,
    kind: InventoryKind,
) -> Result<InventoryRecord, RecordRefusalAtTheFile> {
    let name: String = match kind {
        InventoryKind::Origin => origin_inventory_name(slot),
        InventoryKind::Clone => clone_inventory_name(slot),
    };
    let path: std::path::PathBuf = home.join(name);
    // No cap is shipped on an inventory's count, so its own size is what a read costs (the design's `L3`).
    let mut file: std::fs::File = open_record(path.as_path())?;
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| return RecordRefusalAtTheFile::Io(error.to_string()))?;
    return parse_inventory(&bytes, kind).map_err(RecordRefusalAtTheFile::Record);
}

/// The random sixteen bytes a staging name carries.
fn staging_random() -> std::io::Result<[u8; STAGING_RANDOM_BYTES]> {
    let mut random: [u8; STAGING_RANDOM_BYTES] = [0; STAGING_RANDOM_BYTES];
    getrandom::fill(&mut random).map_err(|error| {
        return std::io::Error::other(format!("the random source failed ({error})"));
    })?;
    return Ok(random);
}

/// What:
///  Publish one binding over whatever name it holds,
///  flushed and whole.
/// Why:
///  A record whose facts nothing can rediscover must not be the weaker half of a pool that already committed them,
///  so the write is flushed and published by one rename.
///
/// # Errors
/// A staging file cannot be made,
/// filled or renamed,
/// or the folder cannot be flushed.
pub fn write_binding(
    home: &std::fs::File,
    slot: &str,
    record: &BindingRecord,
) -> std::io::Result<()> {
    let name: String = binding_name(slot);
    let staging: String = record_staging_name(name.as_str(), &staging_random()?);
    let mut file: std::fs::File =
        create_new_entry(home, OsStr::new(staging.as_str()), PRIVATE_FILE_MODE)?;
    fill_entry_and_sync(&mut file, record.encode().as_slice(), PRIVATE_FILE_MODE)?;
    drop(file);
    rename_entry_replacing(
        home,
        OsStr::new(staging.as_str()),
        OsStr::new(name.as_str()),
    )?;
    return sync_folder(home);
}

/// What:
///  Publish one inventory under its own name,
///  once.
/// Why:
///  The two inventories are immutable,
///  and a second publication must refuse rather than merge two histories,
///  which the exclusive creation and one link give.
///
/// # Errors
/// A staging file cannot be made,
/// filled or linked,
/// the name is already taken (`EEXIST`),
/// or the folder cannot be flushed.
pub fn publish_inventory(
    home: &std::fs::File,
    slot: &str,
    kind: InventoryKind,
    record: &InventoryRecord,
) -> std::io::Result<()> {
    let name: String = match kind {
        InventoryKind::Origin => origin_inventory_name(slot),
        InventoryKind::Clone => clone_inventory_name(slot),
    };
    let staging: String = record_staging_name(name.as_str(), &staging_random()?);
    let mut file: std::fs::File =
        create_new_entry(home, OsStr::new(staging.as_str()), PRIVATE_FILE_MODE)?;
    fill_entry_and_sync(&mut file, record.encode().as_slice(), PRIVATE_FILE_MODE)?;
    drop(file);
    let linked: std::io::Result<()> = link_entry(
        home,
        OsStr::new(staging.as_str()),
        OsStr::new(name.as_str()),
    );
    remove_entry(home, OsStr::new(staging.as_str()))?;
    linked?;
    return sync_folder(home);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "record_publication_tests.rs"]
mod tests;
