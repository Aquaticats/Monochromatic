//! What:
//!  The staged-folder record:
//!  one JSON object per clone GUID beside its staging folder,
//!  holding the folder's identity,
//!  its file handle,
//!  the intent's nonce,
//!  and the handler's mount-namespace pair and boot id;
//!  its bounded,
//!  no-follow read;
//!  and the order its void tests run in.
//! Why:
//!  A later sweep needs three things a receipt cannot give it:
//!  the folder's handle,
//!  its identity,
//!  and the boot and namespace pair that say whether a recorded number still
//!  means anything
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staged-folder record").
//!  The handler writes it before the first bind,
//!  through the `~/.cache` descriptor,
//!  as one temporary file renamed over the final name,
//!  so a reader sees one whole record or the one before it.
//!  The boot id is tested first and on its own,
//!  the handle second,
//!  and the folder's identity only after both,
//!  because a record from another boot proves nothing about this boot's mounts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeStagedFolderRecord(cache, record);           // one atomic rename
//! const reading = readStagedFolderRecord(cache, guid, boot, handles, mounts);
//! ```

use super::staged_folder_identity::StagedFolderIdentity;
use super::staging_folder_name::{staging_folder_name, staging_record_name, staging_temporary_name};
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::foundation::json_record::{parse_object, string_field};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::io::Write;

/// The record's format version.
pub const RECORD_VERSION: &str = "v1";

/// The record's kind,
/// which tells it from another JSON file under `~/.cache`.
pub const RECORD_KIND: &str = "staged-folder-record";

/// The largest record this reader accepts.
pub const MAXIMUM_RECORD_BYTES: u64 = 8192;

/// The largest file handle `name_to_handle_at` may report,
/// which is the kernel's own `MAX_HANDLE_SZ`.
pub const MAXIMUM_HANDLE_BYTES: usize = 128;

/// The digits a 64-bit number is written with.
const MAXIMUM_INTEGER_DIGITS: usize = 20;

/// What:
///  The one record a staging folder has.
/// Why:
///  The fields travel together because every reader compares them together:
///  the component names the folder,
///  the handle reopens it after its name moved,
///  and the boot and namespace pair bound what the handle may mean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedFolderRecord = { component, dataset, guid, identity, handleType, handle, nonce, namespaceDevice, namespaceInode, boot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedFolderRecord {
    /// The folder's component,
    /// which the GUID derives.
    pub component: String,
    /// The clone's dataset name.
    pub dataset: String,
    /// The clone's dataset GUID.
    pub guid: DatasetGuid,
    /// The folder's recorded identity.
    pub identity: StagedFolderIdentity,
    /// The handle's type,
    /// as `name_to_handle_at` reports it.
    pub handle_type: i32,
    /// The handle's bytes,
    /// as `name_to_handle_at` reports them.
    pub handle: Vec<u8>,
    /// The nonce of the intent the record last served.
    pub nonce: String,
    /// The mount namespace's device the handler ran in.
    pub namespace_device: u64,
    /// The mount namespace's inode the handler ran in.
    pub namespace_inode: u64,
    /// The boot the handler ran on.
    pub boot: String,
}

pub use super::staged_folder_error::StagedFolderRefusal;

/// The birth time as the record spells it.
fn birth_text(birth: Option<(i64, u32)>) -> String {
    return match birth {
        Some((seconds, nanos)) => format!("{seconds}:{nanos}"),
        None => String::from("none"),
    };
}

/// The birth time the record spells.
fn birth_of(text: &str) -> Option<Option<(i64, u32)>> {
    if text == "none" {
        return Some(None);
    }
    let (seconds, nanos) = text.split_once(':')?;
    return Some(Some((seconds.parse().ok()?, nanos.parse().ok()?)));
}

/// One field's text,
/// refused with its own name.
fn field(map: &Map<String, Value>, key: &'static str) -> Result<String, StagedFolderRefusal> {
    return match string_field(map, key) {
        Some(text) => Ok(String::from(text)),
        None => Err(StagedFolderRefusal::Field { field: key, said: String::from("the field is absent or is not text") }),
    };
}

/// One field's number.
fn number(map: &Map<String, Value>, key: &'static str) -> Result<u64, StagedFolderRefusal> {
    let text: String = field(map, key)?;
    if text.is_empty() || text.len() > MAXIMUM_INTEGER_DIGITS || !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return Err(StagedFolderRefusal::Field { field: key, said: String::from("it is not a decimal number") });
    }
    return text.parse::<u64>().map_err(|_error| return StagedFolderRefusal::Field { field: key, said: String::from("it is larger than 64 bits") });
}

impl StagedFolderRecord {
    /// The record as the one JSON object the file holds.
    pub fn encode(&self) -> String {
        #[allow(clippy::disallowed_methods, reason = "the JSON macro builds one object of every field; no fallible read is made here")]
        let object: Value = serde_json::json!({
            "version": RECORD_VERSION,
            "kind": RECORD_KIND,
            "component": self.component,
            "dataset": self.dataset,
            "guid": self.guid.file_name(),
            "device": self.identity.device.to_string(),
            "inode": self.identity.inode.to_string(),
            "birth": birth_text(self.identity.birth),
            "owner": self.identity.owner.to_string(),
            "mode": format!("{:04o}", self.identity.mode),
            "handle-type": self.handle_type.to_string(),
            "handle": hexadecimal_of_bytes(self.handle.as_slice()),
            "nonce": self.nonce,
            "namespace-device": self.namespace_device.to_string(),
            "namespace-inode": self.namespace_inode.to_string(),
            "boot": self.boot,
        });
        return format!("{object}\n");
    }
}

/// What:
///  Read one record text as the record of `wanted`.
/// Why:
///  Every field is checked,
///  and the component is compared with the one the GUID derives,
///  so a record renamed beside another folder describes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = stagedFolderRecordFromText(text, guid);
/// ```
///
/// # Errors
/// The text is not an object,
/// a field is absent or wrong,
/// the version or kind is not this shape,
/// or the record names another GUID.
pub fn record_from_text(text: &str, wanted: DatasetGuid) -> Result<StagedFolderRecord, StagedFolderRefusal> {
    let map: Map<String, Value> = parse_object(text).ok_or_else(|| return StagedFolderRefusal::Malformed { said: String::from("the text is not one JSON object") })?;
    let version: String = field(&map, "version")?;
    if version != RECORD_VERSION {
        return Err(StagedFolderRefusal::Malformed { said: format!("its version is `{version}`, and this route reads `{RECORD_VERSION}`") });
    }
    let kind: String = field(&map, "kind")?;
    if kind != RECORD_KIND {
        return Err(StagedFolderRefusal::Malformed { said: format!("its kind is `{kind}`, and this route reads `{RECORD_KIND}`") });
    }
    let guid_text: String = field(&map, "guid")?;
    let guid: DatasetGuid = DatasetGuid::parse(guid_text.as_str()).map_err(|refusal| return StagedFolderRefusal::Field { field: "guid", said: refusal.message() })?;
    if guid != wanted {
        return Err(StagedFolderRefusal::WrongGuid { wanted, recorded: guid_text });
    }
    let component: String = field(&map, "component")?;
    let derived: String = staging_folder_name(wanted);
    if component != derived {
        return Err(StagedFolderRefusal::Field { field: "component", said: format!("it is `{component}`, and the GUID derives `{derived}`") });
    }
    let birth: Option<(i64, u32)> = birth_of(field(&map, "birth")?.as_str()).ok_or_else(|| return StagedFolderRefusal::Field { field: "birth", said: String::from("it is neither `none` nor `seconds:nanoseconds`") })?;
    let mode_text: String = field(&map, "mode")?;
    let mode: u32 = u32::from_str_radix(mode_text.as_str(), 8).map_err(|_error| return StagedFolderRefusal::Field { field: "mode", said: String::from("it is not octal") })?;
    let handle_text: String = field(&map, "handle")?;
    if handle_text.is_empty() || handle_text.len() > MAXIMUM_HANDLE_BYTES * 2 || handle_text.len() % 2 != 0 {
        return Err(StagedFolderRefusal::Handle { said: format!("its text is {} bytes, and a handle is one to {MAXIMUM_HANDLE_BYTES} whole bytes", handle_text.len()) });
    }
    let handle: Vec<u8> = decode_hex(handle_text.as_str()).ok_or_else(|| return StagedFolderRefusal::Handle { said: String::from("its text is not lowercase hexadecimal") })?;
    let handle_type: i32 = i32::try_from(number(&map, "handle-type")?).map_err(|_error| return StagedFolderRefusal::Field { field: "handle-type", said: String::from("it is larger than 32 bits") })?;
    let nonce: String = field(&map, "nonce")?;
    if nonce.len() != 32 || !nonce.bytes().all(|byte| return byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(StagedFolderRefusal::Field { field: "nonce", said: String::from("it is not 32 lowercase hexadecimal digits") });
    }
    return Ok(StagedFolderRecord {
        component,
        dataset: field(&map, "dataset")?,
        guid,
        identity: StagedFolderIdentity {
            device: number(&map, "device")?,
            inode: number(&map, "inode")?,
            birth,
            owner: u32::try_from(number(&map, "owner")?).map_err(|_error| return StagedFolderRefusal::Field { field: "owner", said: String::from("it is larger than 32 bits") })?,
            mode,
        },
        handle_type,
        handle,
        nonce,
        namespace_device: number(&map, "namespace-device")?,
        namespace_inode: number(&map, "namespace-inode")?,
        boot: field(&map, "boot")?,
    });
}

/// The bytes a hexadecimal text spells.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    let mut digits = text.bytes();
    while let (Some(high), Some(low)) = (digits.next(), digits.next()) {
        if !high.is_ascii_hexdigit() || !low.is_ascii_hexdigit() || high.is_ascii_uppercase() || low.is_ascii_uppercase() {
            return None;
        }
        bytes.push(u8::from_str_radix(std::str::from_utf8(&[high, low]).ok()?, 16).ok()?);
    }
    return Some(bytes);
}

/// The names a record is written under.
fn names(guid: DatasetGuid) -> (OsString, OsString) {
    return (OsString::from(staging_temporary_name(guid)), OsString::from(staging_record_name(guid)));
}

/// What:
///  Write one record atomically through the `~/.cache` descriptor:
///  a temporary name created exclusively,
///  written and synced,
///  renamed over the final name,
///  and the folder synced.
/// Why:
///  A reader sees the whole record of one write or the one before it,
///  and a killed writer leaves only the temporary name the next write replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeStagedFolderRecord(cache, record);
/// ```
///
/// # Errors
/// The temporary name could not be created or written,
/// the rename refused,
/// or the folder could not be synced.
pub fn write_record(cache: &std::fs::File, record: &StagedFolderRecord) -> Result<(), StagedFolderRefusal> {
    let (temporary, final_name): (OsString, OsString) = names(record.guid);
    let _stale = crate::fs_beneath::remove_at(cache, temporary.as_os_str(), false);
    let mut file: std::fs::File = crate::fs_beneath::create_file_at(cache, temporary.as_os_str(), 0o644)
        .map_err(|error| return StagedFolderRefusal::Unreadable { path: temporary.to_string_lossy().to_string(), reason: error.to_string() })?;
    file.write_all(record.encode().as_bytes())
        .map_err(|error| return StagedFolderRefusal::Unreadable { path: temporary.to_string_lossy().to_string(), reason: error.to_string() })?;
    file.sync_all().map_err(|error| return StagedFolderRefusal::Unreadable { path: temporary.to_string_lossy().to_string(), reason: error.to_string() })?;
    drop(file);
    crate::fs_beneath::rename_at(cache, temporary.as_os_str(), cache, final_name.as_os_str(), true)
        .map_err(|error| return StagedFolderRefusal::Unreadable { path: final_name.to_string_lossy().to_string(), reason: error.to_string() })?;
    return cache.sync_all().map_err(|error| return StagedFolderRefusal::Unreadable { path: String::from("~/.cache"), reason: error.to_string() });
}

/// What:
///  Read the record beside one GUID's folder,
///  or nothing when the name is absent or is not a plain file.
/// Why:
///  The reader takes the bytes from an open handle that followed no link and
///  refuses a file past the record's bound,
///  so a name swapped for a pipe or a link cannot redirect the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readRecord(cache, guid); // { kind: 'absent' | 'text' | 'unreadable' }
/// ```
pub fn read_record_text(cache: &std::fs::File, guid: DatasetGuid) -> Result<Option<String>, StagedFolderRefusal> {
    let name: OsString = OsString::from(staging_record_name(guid));
    let file: std::fs::File = match crate::fs_beneath::open_file_at(cache, name.as_os_str()) {
        Ok(file) => file,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(None),
        Err(error) => return Err(StagedFolderRefusal::Unreadable { path: name.to_string_lossy().to_string(), reason: error.to_string() }),
    };
    let facts: std::fs::Metadata = file.metadata().map_err(|error| return StagedFolderRefusal::Unreadable { path: name.to_string_lossy().to_string(), reason: error.to_string() })?;
    if facts.len() > MAXIMUM_RECORD_BYTES {
        return Err(StagedFolderRefusal::TooLarge { bytes: facts.len(), bound: MAXIMUM_RECORD_BYTES });
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(usize::try_from(facts.len()).unwrap_or(0));
    use std::io::Read;
    std::io::Read::take(&file, MAXIMUM_RECORD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| return StagedFolderRefusal::Unreadable { path: name.to_string_lossy().to_string(), reason: error.to_string() })?;
    if bytes.len() as u64 > MAXIMUM_RECORD_BYTES {
        return Err(StagedFolderRefusal::TooLarge { bytes: bytes.len() as u64, bound: MAXIMUM_RECORD_BYTES });
    }
    return match std::str::from_utf8(bytes.as_slice()) {
        Ok(text) => Ok(Some(String::from(text))),
        Err(_) => Err(StagedFolderRefusal::Malformed { said: String::from("the file is not UTF-8 text") }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staged_folder_record_tests.rs"]
mod tests;
