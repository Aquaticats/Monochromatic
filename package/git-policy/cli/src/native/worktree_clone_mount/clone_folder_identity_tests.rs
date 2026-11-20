//! Controls for the recorded identity of one folder:
//! every field is read from the descriptor that was opened,
//! a difference in any one field is told apart from the others,
//! and the store token round-trips.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

/// The version of the kernel's ACL extended-attribute header
/// (`POSIX_ACL_XATTR_VERSION`).
/// Its path in the kernel tree is `include/uapi/linux/posix_acl_xattr.h`.
const ACL_VERSION: u32 = 2;

/// The ACL entry tag of a named user
/// (`ACL_USER`).
const ACL_USER: u16 = 0x02;

/// The ACL entry tag of the owning user
/// (`ACL_USER_OBJ`).
const ACL_USER_OBJ: u16 = 0x01;

/// The ACL entry tag of the owning group
/// (`ACL_GROUP_OBJ`).
const ACL_GROUP_OBJ: u16 = 0x04;

/// The ACL entry tag of the mask
/// (`ACL_MASK`).
const ACL_MASK: u16 = 0x10;

/// The ACL entry tag of other
/// (`ACL_OTHER`).
const ACL_OTHER: u16 = 0x20;

/// The user ID an unnamed ACL entry carries
/// (`ACL_UNDEFINED_ID`).
const ACL_UNDEFINED_ID: u32 = u32::MAX;

/// One ACL entry:
/// its tag,
/// its permission bits and its user or group ID.
type AclEntry = (u16, u16, u32);

/// An ACL blob in the kernel's extended-attribute encoding,
/// with the version header the kernel expects.
fn acl_blob(entries: &[AclEntry]) -> Vec<u8> {
    let mut blob: Vec<u8> = Vec::new();
    blob.extend_from_slice(&ACL_VERSION.to_le_bytes());
    for (tag, permissions, id) in entries {
        blob.extend_from_slice(&tag.to_le_bytes());
        blob.extend_from_slice(&permissions.to_le_bytes());
        blob.extend_from_slice(&id.to_le_bytes());
    }
    return blob;
}

/// The three-entry ACL a folder of mode `0750` has once one is written.
fn basic_access_acl() -> Vec<u8> {
    return acl_blob(&[(ACL_USER_OBJ, 0o7, ACL_UNDEFINED_ID), (ACL_GROUP_OBJ, 0o5, ACL_UNDEFINED_ID), (ACL_OTHER, 0o0, ACL_UNDEFINED_ID)]);
}

/// What:
///  Write the access ACL of an open folder,
///  the way `setfacl` does,
///  so a control can prove the identity read the value the kernel holds.
/// Why:
///   The route reads ACLs byte for byte and no library here writes them,
/// so the control composes the kernel's own encoding.
fn set_access_acl(folder: &std::fs::File, blob: &[u8]) {
    use std::ffi::CString;
    let name: CString = CString::new("system.posix_acl_access").expect("no NUL in an attribute name");
    // SAFETY: `name` is NUL-terminated and outlives the call, `blob` holds the bytes the call reads, and `folder` is an
    // open descriptor.
    let result: libc::c_int = unsafe { libc::fsetxattr(folder.as_raw_fd(), name.as_ptr(), blob.as_ptr().cast::<libc::c_void>(), blob.len(), 0) };
    assert_eq!(result, 0, "the fixture's access ACL is written: {}", std::io::Error::last_os_error());
}

/// What:
///  Every field of an identity is read from the folder the descriptor holds.
/// Why:
///  A field taken from the path instead could describe another folder,
///  so the control compares the captured values with the folder's own metadata.
#[test]
fn capture_reads_the_folder_the_descriptor_holds() {
    let root: PathBuf = fixture("clone-identity-capture");
    let folder: PathBuf = root.join("folder");
    std::fs::create_dir(&folder).expect("folder");
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let opened: std::fs::File = std::fs::File::open(&folder).expect("open");
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    let named: std::fs::Metadata = std::fs::metadata(&folder).expect("metadata");
    assert_eq!(identity.device, named.dev());
    assert_eq!(identity.inode, named.ino());
    assert_eq!(identity.owner, named.uid());
    assert_eq!(identity.mode, 0o755);
    assert_eq!(identity.access_acl, None, "a folder without an ACL has none");
    assert_eq!(identity.default_acl, None);
    assert_eq!(CloneFolderIdentity::capture(&opened).expect("again"), identity, "the same folder reads the same way twice");
    remove(root.as_path());
}

/// What:
///  A difference in exactly one field is reported as that field,
///  and equal identities report nothing.
/// Why:
///  This is the positive control of the compare,
///  field by field:
///  a compare that reported the wrong field would send a person to the wrong cause.
#[test]
fn each_field_alone_is_told_apart() {
    let base: CloneFolderIdentity = CloneFolderIdentity {
        device: 66304,
        inode: 42,
        owner: 1000,
        mode: 0o755,
        access_acl: Some(vec![2, 0, 0, 0]),
        default_acl: Some(vec![3, 0, 0, 0]),
    };
    assert_eq!(base.differences(&base.clone()), Vec::<CloneFolderField>::new(), "the same folder differs in nothing");
    let changed: [CloneFolderIdentity; 6] = [
        CloneFolderIdentity { device: base.device + 1, ..base.clone() },
        CloneFolderIdentity { inode: base.inode + 1, ..base.clone() },
        CloneFolderIdentity { owner: base.owner + 1, ..base.clone() },
        CloneFolderIdentity { mode: 0o750, ..base.clone() },
        CloneFolderIdentity { access_acl: Some(vec![2, 0, 0, 1]), ..base.clone() },
        CloneFolderIdentity { default_acl: None, ..base.clone() },
    ];
    let expected: [CloneFolderField; 6] = [
        CloneFolderField::Device,
        CloneFolderField::Inode,
        CloneFolderField::Owner,
        CloneFolderField::Mode,
        CloneFolderField::AccessAcl,
        CloneFolderField::DefaultAcl,
    ];
    for (other, field) in changed.iter().zip(expected.iter()) {
        assert_eq!(base.differences(other), vec![*field], "only {} moved", field.name());
        assert_eq!(other.differences(&base), vec![*field], "the report is the same in either direction");
    }
    let two: CloneFolderIdentity = CloneFolderIdentity { mode: 0o700, default_acl: None, ..base.clone() };
    assert_eq!(base.differences(&two), vec![CloneFolderField::Mode, CloneFolderField::DefaultAcl], "every field that moved is reported");
}

/// What:
///  A permission change and each of the two ACLs are seen on a real folder,
///  and an access ACL whose entries changed while its mode stayed is seen too.
/// Why:
///   The design's measurements found an ACL drift passing a mode-only check,
///  so the control changes only an ACL entry,
///  with the same mask and owner,
///  and requires the access ACL to be the one reported.
#[test]
fn a_permission_change_and_each_acl_are_seen_apart() {
    let root: PathBuf = fixture("clone-identity-acl");
    let folder: PathBuf = root.join("folder");
    std::fs::create_dir(&folder).expect("folder");
    let opened: std::fs::File = std::fs::File::open(&folder).expect("open");
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o750)).expect("mode");
    let plain: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700)).expect("mode");
    let tightened: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    assert_eq!(plain.differences(&tightened), vec![CloneFolderField::Mode]);
    // A folder whose access ACL is exactly its mode keeps no ACL: the kernel stores the mode alone, so the control
    // writes one named user's entry, which no mode can express, and the mask the mode shows stays as it was.
    let first: Vec<u8> = acl_blob(&[(ACL_USER_OBJ, 0o7, ACL_UNDEFINED_ID), (ACL_USER, 0o4, 1000), (ACL_GROUP_OBJ, 0o5, ACL_UNDEFINED_ID), (ACL_MASK, 0o5, ACL_UNDEFINED_ID), (ACL_OTHER, 0o0, ACL_UNDEFINED_ID)]);
    let second: Vec<u8> = acl_blob(&[(ACL_USER_OBJ, 0o7, ACL_UNDEFINED_ID), (ACL_USER, 0o5, 1000), (ACL_GROUP_OBJ, 0o5, ACL_UNDEFINED_ID), (ACL_MASK, 0o5, ACL_UNDEFINED_ID), (ACL_OTHER, 0o0, ACL_UNDEFINED_ID)]);
    set_access_acl(&opened, &first);
    let one_user: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    assert_eq!(one_user.mode, plain.mode, "the fixture keeps one mode as its ACL appears");
    assert_eq!(plain.differences(&one_user), vec![CloneFolderField::AccessAcl], "the access ACL alone moved");
    set_access_acl(&opened, &second);
    let another_user: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    assert_eq!(one_user.mode, another_user.mode, "the mode is unchanged by the entry that changed");
    assert_eq!(one_user.differences(&another_user), vec![CloneFolderField::AccessAcl], "an ACL drift passes a mode-only check, and this compare is not mode-only");
    // A default ACL appears on the same folder, its access ACL untouched.
    let default_blob: Vec<u8> = basic_access_acl();
    let without_default: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    let name: std::ffi::CString = std::ffi::CString::new("system.posix_acl_default").expect("name");
    // SAFETY: `name` is NUL-terminated and outlives the call, the blob holds what the call reads, and `opened` is live.
    let written: libc::c_int = unsafe { libc::fsetxattr(opened.as_raw_fd(), name.as_ptr(), default_blob.as_ptr().cast::<libc::c_void>(), default_blob.len(), 0) };
    assert_eq!(written, 0, "the default ACL is written: {}", std::io::Error::last_os_error());
    let with_default: CloneFolderIdentity = CloneFolderIdentity::capture(&opened).expect("identity");
    assert_eq!(without_default.differences(&with_default), vec![CloneFolderField::DefaultAcl]);
}

/// What:
///  An identity is written as one store token and read back,
///  and a token this module did not write is refused.
/// Why:
///  The store is the only copy of a recorded identity,
///  so a token that lost a field would silently widen what a later run accepts.
#[test]
fn an_identity_round_trips_through_its_store_token() {
    let recorded: CloneFolderIdentity = CloneFolderIdentity { device: 66304, inode: 306, owner: 1000, mode: 0o750, access_acl: Some(basic_access_acl()), default_acl: None };
    assert_eq!(CloneFolderIdentity::decode(recorded.encode().as_str()), Some(recorded.clone()));
    assert!(recorded.encode().ends_with(":-"), "no default ACL is the `-` token: {}", recorded.encode());
    assert_eq!(CloneFolderIdentity::decode(&format!("66304:306:1000:0750:{}:-", acl_token(Some(&basic_access_acl())))), Some(recorded.clone()));
    // An empty ACL field and an uppercase digit are not tokens `acl_token` writes, so a line holding one is a
    // line this repository did not write and the identity is refused rather than read.
    let broken: [&str; 10] = [
        "",
        "66304:306:1000:0750",
        "66304:306:1000:0750:-:-:-",
        "a:306:1000:0750:-:-",
        "66304:b:1000:0750:-:-",
        "66304:306:c:0750:-:-",
        "66304:306:1000:zzz:-:-",
        "66304:306:1000:0750:abc:-",
        "66304:306:1000:0750::-",
        "66304:306:1000:0750:2F010000:-",
    ];
    for token in broken {
        assert_eq!(CloneFolderIdentity::decode(token), None, "{token}");
    }
    let described: String = recorded.describe();
    for field in [CloneFolderField::Device, CloneFolderField::Inode, CloneFolderField::Owner, CloneFolderField::Mode, CloneFolderField::AccessAcl, CloneFolderField::DefaultAcl] {
        assert!(described.contains(field.name()), "{} names {}", described, field.name());
    }
    assert!(described.contains("0750"));
    assert!(described.contains("none"), "a missing ACL is said in words");
}
