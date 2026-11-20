//! What:
//!  The identity of one folder as the ZFS worktree route records it:
//! its device,
//! inode,
//! owning user,
//! permission bits,
//! and its access and default POSIX ACLs,
//! read from an open descriptor and compared field by field.
//! Why:
//!  A path or a name proves nothing about which folder it names later,
//! and the design's measurements found that changing only an ACL passes a mode-only check
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Descriptor mount and unmount",
//! item 2 of "VM measurements preceding acceptance").
//! The route therefore records these six fields for a folder it may act on
//! and refuses a folder whose value differs,
//! naming the field:
//! a changed identity is not permission to adopt the replacement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorded = capture(handle); // { device, inode, owner, mode, accessAcl, defaultAcl }
//! const differs = recorded.differences(capture(handleLater)); // ['mode'] or []
//! ```

use super::clone_folder_identity_acl::{ACCESS_ACL_ATTRIBUTE, DEFAULT_ACL_ATTRIBUTE, acl_from_token, acl_token, acl_words, read_acl};
use crate::diagnostic_log::debug;
use std::os::unix::fs::MetadataExt;

/// Module tag for diagnostics.
const TAG: &str = "cloneFolderIdentity";

/// The permission bits of a mode:
/// `S_IRWXU`,
/// `S_IRWXG` and `S_IRWXO`,
/// without the file type bits.
pub const PERMISSION_BITS: u32 = 0o7777;

/// What:
///  The part of a folder's identity two records of it can differ in.
/// Why:
///   A refusal names the field that moved,
/// so a person can tell an ACL drift from a replaced inode,
///  and a control can prove each field on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneFolderField = 'device' | 'inode' | 'owner' | 'mode' | 'access-acl' | 'default-acl';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloneFolderField {
    /// The device the folder is on.
    Device,
    /// The inode number of the folder.
    Inode,
    /// The user owning the folder.
    Owner,
    /// The permission bits of the folder.
    Mode,
    /// The access ACL of the folder.
    AccessAcl,
    /// The default ACL of the folder.
    DefaultAcl,
}

impl CloneFolderField {
    /// The field's name in a sentence,
    /// such as `the access ACL differs`.
    pub fn name(self) -> &'static str {
        return match self {
            CloneFolderField::Device => "device",
            CloneFolderField::Inode => "inode",
            CloneFolderField::Owner => "owner",
            CloneFolderField::Mode => "mode",
            CloneFolderField::AccessAcl => "access ACL",
            CloneFolderField::DefaultAcl => "default ACL",
        };
    }
}

/// What:
///  The fields that differ,
///  as words one sentence can hold.
/// Why:
///   The walk's refusal,
///  the post-attach check and the adoption table each report the fields that moved,
///  so the join lives beside the field names it joins and all three read the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = fieldNames(['mode', 'access-acl']); // 'mode, access ACL'
/// ```
pub fn field_names(fields: &[CloneFolderField]) -> String {
    let mut names: Vec<&str> = Vec::new();
    for field in fields {
        names.push(field.name());
    }
    return names.join(", ");
}

/// What:
///  Every field the route records for one folder.
/// Why:
///   Comparing whole records is what the route does,
/// so one value type holds all six fields and one function compares them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneFolderIdentity = { device: bigint; inode: bigint; owner: number; mode: number; accessAcl: Uint8Array | null; defaultAcl: Uint8Array | null };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneFolderIdentity {
    /// The device number,
    /// from `fstat`.
    pub device: u64,
    /// The inode number,
    /// from `fstat`.
    pub inode: u64,
    /// The user owning the folder,
    /// from `fstat`.
    pub owner: u32,
    /// The permission bits,
    /// without the file type.
    pub mode: u32,
    /// The access ACL's raw value,
    /// or nothing when the folder has none.
    pub access_acl: Option<Vec<u8>>,
    /// The default ACL's raw value,
    /// or nothing when the folder has none.
    pub default_acl: Option<Vec<u8>>,
}

impl CloneFolderIdentity {
    /// What:
    ///  Read the identity of an open folder.
    /// Why:
    ///   Reading through the descriptor the caller already holds keeps every field on the object that was judged,
    ///  never on whatever its name holds now.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const identity = capture(openFolder);
    /// ```
    ///
    /// # Errors
    /// The stat or extended-attribute failure,
    /// or a refusal naming the ACL whose value is larger than `MAXIMUM_ACL_BYTES`.
    pub fn capture(folder: &std::fs::File) -> std::io::Result<CloneFolderIdentity> {
        let metadata: std::fs::Metadata = folder.metadata()?;
        let identity: CloneFolderIdentity = CloneFolderIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            owner: metadata.uid(),
            mode: metadata.mode() & PERMISSION_BITS,
            access_acl: read_acl(folder, ACCESS_ACL_ATTRIBUTE, "access ACL")?,
            default_acl: read_acl(folder, DEFAULT_ACL_ATTRIBUTE, "default ACL")?,
        };
        debug(TAG, "read the identity of one folder");
        return Ok(identity);
    }

    /// What:
    ///  The fields in which `self` and `other` differ,
    ///  in a fixed order.
    /// Why:
    ///   A refusal reports every field that moved,
    /// and the order is the route's own so a message reads the same way every time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const moved = recorded.differences(current);
    /// ```
    pub fn differences(&self, other: &CloneFolderIdentity) -> Vec<CloneFolderField> {
        let mut moved: Vec<CloneFolderField> = Vec::new();
        if self.device != other.device {
            moved.push(CloneFolderField::Device);
        }
        if self.inode != other.inode {
            moved.push(CloneFolderField::Inode);
        }
        if self.owner != other.owner {
            moved.push(CloneFolderField::Owner);
        }
        if self.mode != other.mode {
            moved.push(CloneFolderField::Mode);
        }
        if self.access_acl != other.access_acl {
            moved.push(CloneFolderField::AccessAcl);
        }
        if self.default_acl != other.default_acl {
            moved.push(CloneFolderField::DefaultAcl);
        }
        return moved;
    }

    /// The identity in words,
    /// naming every field,
    /// for a report that must be readable without the store's file.
    pub fn describe(&self) -> String {
        return format!(
            "device {}, inode {}, owner {}, mode {:04o}, access ACL {}, default ACL {}",
            self.device,
            self.inode,
            self.owner,
            self.mode,
            acl_words(self.access_acl.as_deref()),
            acl_words(self.default_acl.as_deref()),
        );
    }

    /// The identity as one store token,
    /// `device:inode:owner:mode:access:default`,
    /// where an ACL is its bytes in lowercase hexadecimal or `-` for none.
    pub fn encode(&self) -> String {
        return format!(
            "{}:{}:{}:{:04o}:{}:{}",
            self.device,
            self.inode,
            self.owner,
            self.mode,
            acl_token(self.access_acl.as_deref()),
            acl_token(self.default_acl.as_deref()),
        );
    }

    /// The identity from its store token,
    /// or nothing when the token is not one this module wrote.
    pub fn decode(token: &str) -> Option<CloneFolderIdentity> {
        let mut parts = token.split(':');
        let device: u64 = parts.next()?.parse().ok()?;
        let inode: u64 = parts.next()?.parse().ok()?;
        let owner: u32 = parts.next()?.parse().ok()?;
        let mode: u32 = u32::from_str_radix(parts.next()?, 8).ok()?;
        let access_acl: Option<Vec<u8>> = acl_from_token(parts.next()?)?;
        let default_acl: Option<Vec<u8>> = acl_from_token(parts.next()?)?;
        return if parts.next().is_none() { Some(CloneFolderIdentity { device, inode, owner, mode, access_acl, default_acl }) } else { None };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_folder_identity_tests.rs"]
mod tests;
