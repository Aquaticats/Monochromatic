//! Controls for the record's four encodings:
//!  a path as hexadecimal,
//!  a folder identity without its device,
//!  a request as a form and an argument,
//!  and the pointer's bytes.

use crate::worktree_clone_binding_record::binding_encoding::*;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::PathBuf;

/// A path round-trips through the hexadecimal encoding,
/// a newline included,
/// which is why the encoding exists.
#[test]
fn a_path_with_a_newline_round_trips_through_hexadecimal() {
    let path: PathBuf = PathBuf::from("/home/user/a\nb=c");
    let digits: String = path_as_hexadecimal(path.as_path());
    assert!(digits.bytes().all(|byte| return is_lowercase_hexadecimal(byte)));
    assert_eq!(path_from_hexadecimal(digits.as_str()), Some(path));
}

/// Hexadecimal refuses an odd length,
/// an uppercase digit and an empty value.
#[test]
fn hexadecimal_refuses_what_is_not_two_digits_a_byte() {
    assert_eq!(hexadecimal_as_bytes("abc"), None);
    assert_eq!(hexadecimal_as_bytes("AB"), None);
    assert_eq!(hexadecimal_as_bytes(""), None);
    assert_eq!(hexadecimal_as_bytes("00ff"), Some(vec![0, 255]));
}

/// An identity encodes without its device,
/// and the `-:-` form reads as an inode with no birth time.
#[test]
fn an_identity_drops_the_device_and_keeps_the_birth_time() {
    let identity: FolderIdentity = FolderIdentity { inode: 131_074, birth: Some((1_763_750_400, 123_456_789)) };
    assert_eq!(identity.encode(), "131074:1763750400:123456789");
    assert_eq!(FolderIdentity::decode("131074:1763750400:123456789"), Some(identity));
    let no_birth: FolderIdentity = FolderIdentity::decode("131074:-:-").expect("the no-birth form reads");
    assert_eq!(no_birth.inode(), 131_074);
    assert_eq!(no_birth.birth(), None);
    assert_eq!(FolderIdentity::decode("131074:-:1"), None);
    assert_eq!(FolderIdentity::decode("131074:1"), None);
    assert_eq!(FolderIdentity::decode("131074:1:2:3"), None);
}

/// A request's argument round-trips through hexadecimal,
/// and `-` belongs to a detached request alone.
#[test]
fn a_request_round_trips_and_the_empty_argument_is_detached_only() {
    let branch: Request = Request::with_argument(RequestForm::Branch, b"main");
    assert_eq!(branch.encode(), "branch 6d61696e");
    assert_eq!(Request::decode("branch 6d61696e"), Some(branch));
    let no_argument: Request = Request::detached();
    assert_eq!(no_argument.encode(), "detach -");
    assert_eq!(Request::decode("detach -"), Some(no_argument));
    assert_eq!(Request::decode("branch -"), None);
    assert_eq!(Request::decode("tag 6d61696e"), None);
    assert_eq!(Request::decode("branch"), None);
}

/// The identity of a real folder names its own inode and no device.
#[test]
fn a_folders_identity_names_its_inode_and_no_device() {
    let root: PathBuf = crate::test_support::fixture("identity");
    std::fs::create_dir(root.join("slot")).expect("the slot folder is made");
    let identity: FolderIdentity = FolderIdentity::of_folder(root.join("slot").as_path()).expect("the folder's identity reads");
    assert!(identity.inode() > 0);
    let encoded: String = identity.encode();
    assert_eq!(encoded.matches(':').count(), 2, "the encoding is `inode:seconds:nanoseconds`");
    crate::test_support::remove(root.as_path());
}

/// A guid's canonical spelling is its decimal form,
/// which the record writes verbatim.
#[test]
fn a_guid_is_written_as_its_decimal() {
    let guid: DatasetGuid = DatasetGuid::parse("17293822569102704641").expect("the guid reads");
    assert_eq!(guid.file_name(), "17293822569102704641");
    assert!(DatasetGuid::parse("0100").is_err());
    assert!(DatasetGuid::parse("0").is_ok());
}
