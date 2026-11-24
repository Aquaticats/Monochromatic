//! What:
//!  The staging claim one repository keeps for each folder the wrapper made,
//!  its own place beside the copy journal,
//!  and its record of the folder's identity and the intent's nonce.
//! Why:
//!  The claim is what proves the wrapper made a folder,
//!  and its own suffix keeps every reader of the copy journal's `.stage` and
//!  `.json` names away from it
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging claim").
//!  It lives beside `v1` rather than inside it,
//!  because the copy recovery counts every entry inside `v1` as pending work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const path = stagingClaimPath(common, guid); // <common>/monochromatic-git-policy-cli-worktree-copy/staging/<guid>.staged
//! writeStagingClaim(directory, claim);
//! ```

use super::staged_folder_identity::StagedFolderIdentity;
use super::staging_folder_name::staging_claim_name;
use crate::json_record::{parse_object, string_field};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use serde_json::Map;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The claim folder's path below a repository's Git common directory.
pub const CLAIM_FOLDER: &str = "monochromatic-git-policy-cli-worktree-copy/staging";

/// The claim's format version.
pub const CLAIM_VERSION: &str = "v1";

/// The claim's kind.
pub const CLAIM_KIND: &str = "staging-claim";

/// The largest claim this reader accepts.
pub const MAXIMUM_CLAIM_BYTES: u64 = 4096;

/// What:
///  One staging claim.
/// Why:
///  It carries the nonce of the intent the folder waits for,
///  the owner that made it,
///  and the folder's identity,
///  which is every proof the wrapper's removal needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingClaim = { guid; nonce; owner; identity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingClaim {
    /// The clone's GUID.
    pub guid: DatasetGuid,
    /// The intent's nonce.
    pub nonce: String,
    /// The owner's token.
    pub owner: String,
    /// The folder's recorded identity.
    pub identity: StagedFolderIdentity,
}

/// Why a claim could not be read or written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimRefusal {
    /// The file could not be read or written.
    Unreadable {
        /// What was read.
        path: String,
        /// What the read answered.
        reason: String,
    },
    /// The file is larger than its bound.
    TooLarge {
        /// The bytes it holds.
        bytes: u64,
        /// The bound.
        bound: u64,
    },
    /// The file is not one claim of this shape.
    Malformed {
        /// What about it is wrong.
        said: String,
    },
    /// The claim names another clone's folder.
    WrongGuid {
        /// The GUID the name derives.
        wanted: DatasetGuid,
        /// The GUID the claim carries.
        recorded: String,
    },
}

impl ClaimRefusal {
    /// The refusal in words.
    pub fn message(&self) -> String {
        return match self {
            ClaimRefusal::Unreadable { path, reason } => format!("the staging claim at {path} could not be read: {reason}. Nothing was removed on a claim that could not be read."),
            ClaimRefusal::TooLarge { bytes, bound } => format!("the staging claim is {bytes} bytes long, and the longest claim this route reads is {bound}. It is kept and reported rather than acted on."),
            ClaimRefusal::Malformed { said } => format!("the staging claim is not one this route reads: {said}. It is kept and reported rather than moved aside, because an unreadable claim may name a folder whose state is unproven."),
            ClaimRefusal::WrongGuid { wanted, recorded } => format!("the staging claim beside GUID {} carries GUID `{recorded}`. It is kept and reported.", wanted.file_name()),
        };
    }
}

/// The claim's path for one clone below a Git common directory.
pub fn staging_claim_path(common: &Path, guid: DatasetGuid) -> PathBuf {
    return common.join(CLAIM_FOLDER).join(staging_claim_name(guid));
}

/// The claim as JSON text.
impl StagingClaim {
    /// The text the file holds.
    #[allow(clippy::disallowed_methods, reason = "the JSON macro builds one object of every field; no fallible read is made here")]
    pub fn encode(&self) -> String {
        let object: serde_json::Value = serde_json::json!({
            "version": CLAIM_VERSION,
            "kind": CLAIM_KIND,
            "guid": self.guid.file_name(),
            "nonce": self.nonce,
            "owner": self.owner,
            "device": self.identity.device.to_string(),
            "inode": self.identity.inode.to_string(),
            "birth": match self.identity.birth {
                Some((seconds, nanos)) => format!("{seconds}:{nanos}"),
                None => String::from("none"),
            },
            "owner-uid": self.identity.owner.to_string(),
            "mode": format!("{:04o}", self.identity.mode),
        });
        return format!("{object}\n");
    }
}

/// One text field,
/// refused with its own name.
fn field(map: &Map<String, serde_json::Value>, key: &'static str) -> Result<String, ClaimRefusal> {
    return string_field(map, key).map(String::from).ok_or_else(|| return ClaimRefusal::Malformed { said: format!("`{key}` is absent or is not text") });
}

/// One decimal field.
fn number(map: &Map<String, serde_json::Value>, key: &'static str) -> Result<u64, ClaimRefusal> {
    let text: String = field(map, key)?;
    return text.parse::<u64>().map_err(|_error| return ClaimRefusal::Malformed { said: format!("`{key}` is not a decimal number") });
}

/// Read one claim text as the claim of `wanted`.
pub fn claim_from_text(text: &str, wanted: DatasetGuid) -> Result<StagingClaim, ClaimRefusal> {
    let map: Map<String, serde_json::Value> = parse_object(text).ok_or_else(|| return ClaimRefusal::Malformed { said: String::from("the text is not one JSON object") })?;
    let version: String = field(&map, "version")?;
    let kind: String = field(&map, "kind")?;
    if version != CLAIM_VERSION || kind != CLAIM_KIND {
        return Err(ClaimRefusal::Malformed { said: format!("its version is `{version}` and its kind `{kind}`") });
    }
    let guid_text: String = field(&map, "guid")?;
    let guid: DatasetGuid = DatasetGuid::parse(guid_text.as_str()).map_err(|refusal| return ClaimRefusal::Malformed { said: refusal.message() })?;
    if guid != wanted {
        return Err(ClaimRefusal::WrongGuid { wanted, recorded: guid_text });
    }
    let birth: Option<(i64, u32)> = match field(&map, "birth")?.as_str() {
        "none" => None,
        birth_text => match birth_text.split_once(':') {
            Some((seconds, nanos)) => Some((seconds.parse().map_err(|_error| return ClaimRefusal::Malformed { said: String::from("`birth` seconds are not a number") })?, nanos.parse().map_err(|_error| return ClaimRefusal::Malformed { said: String::from("`birth` nanoseconds are not a number") })?)),
            None => return Err(ClaimRefusal::Malformed { said: String::from("`birth` is neither `none` nor `seconds:nanoseconds`") }),
        },
    };
    let mode: u32 = u32::from_str_radix(field(&map, "mode")?.as_str(), 8).map_err(|_error| return ClaimRefusal::Malformed { said: String::from("`mode` is not octal") })?;
    return Ok(StagingClaim {
        guid,
        nonce: field(&map, "nonce")?,
        owner: field(&map, "owner")?,
        identity: StagedFolderIdentity {
            device: number(&map, "device")?,
            inode: number(&map, "inode")?,
            birth,
            owner: u32::try_from(number(&map, "owner-uid")?).map_err(|_error| return ClaimRefusal::Malformed { said: String::from("`owner-uid` is larger than 32 bits") })?,
            mode,
        },
    });
}

/// Write one claim atomically in its folder.
pub fn write_claim(directory: &std::fs::File, claim: &StagingClaim) -> Result<(), ClaimRefusal> {
    let temporary: OsString = OsString::from(format!("{}.tmp", staging_claim_name(claim.guid)));
    let final_name: OsString = OsString::from(staging_claim_name(claim.guid));
    let _stale = crate::fs_beneath::remove_at(directory, temporary.as_os_str(), false);
    let mut file: std::fs::File = crate::fs_beneath::create_file_at(directory, temporary.as_os_str(), 0o644).map_err(|error| return ClaimRefusal::Unreadable { path: temporary.to_string_lossy().to_string(), reason: error.to_string() })?;
    file.write_all(claim.encode().as_bytes()).map_err(|error| return ClaimRefusal::Unreadable { path: temporary.to_string_lossy().to_string(), reason: error.to_string() })?;
    file.sync_all().map_err(|error| return ClaimRefusal::Unreadable { path: temporary.to_string_lossy().to_string(), reason: error.to_string() })?;
    drop(file);
    crate::fs_beneath::rename_at(directory, temporary.as_os_str(), directory, final_name.as_os_str(), true).map_err(|error| return ClaimRefusal::Unreadable { path: final_name.to_string_lossy().to_string(), reason: error.to_string() })?;
    return directory.sync_all().map_err(|error| return ClaimRefusal::Unreadable { path: String::from("the claim's folder"), reason: error.to_string() });
}

/// Read one claim file,
/// or nothing when it is absent.
pub fn read_claim(directory: &std::fs::File, guid: DatasetGuid) -> Result<Option<StagingClaim>, ClaimRefusal> {
    let name: OsString = OsString::from(staging_claim_name(guid));
    let file: std::fs::File = match crate::fs_beneath::open_file_at(directory, name.as_os_str()) {
        Ok(file) => file,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(None),
        Err(error) => return Err(ClaimRefusal::Unreadable { path: name.to_string_lossy().to_string(), reason: error.to_string() }),
    };
    let facts: std::fs::Metadata = file.metadata().map_err(|error| return ClaimRefusal::Unreadable { path: name.to_string_lossy().to_string(), reason: error.to_string() })?;
    if facts.len() > MAXIMUM_CLAIM_BYTES {
        return Err(ClaimRefusal::TooLarge { bytes: facts.len(), bound: MAXIMUM_CLAIM_BYTES });
    }
    use std::io::Read;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::take(&file, MAXIMUM_CLAIM_BYTES + 1).read_to_end(&mut bytes).map_err(|error| return ClaimRefusal::Unreadable { path: name.to_string_lossy().to_string(), reason: error.to_string() })?;
    if bytes.len() as u64 > MAXIMUM_CLAIM_BYTES {
        return Err(ClaimRefusal::TooLarge { bytes: bytes.len() as u64, bound: MAXIMUM_CLAIM_BYTES });
    }
    let text: &str = crate::json_record::decode_fatal(bytes.as_slice()).ok_or_else(|| return ClaimRefusal::Malformed { said: String::from("the file is not UTF-8 text") })?;
    return claim_from_text(text, guid).map(Some);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staging_claim_tests.rs"]
mod tests;
