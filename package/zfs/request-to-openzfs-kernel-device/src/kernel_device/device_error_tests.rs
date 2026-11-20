//! What:
//!  The device-free proof of the live failure type:
//! an unopenable device names its
//!  path,
//! a request failure keeps its operation,
//! target and error number,
//! and the two
//!  disagreements say what the kernel asked for.
//! Why:
//!  A caller tells a machine without OpenZFS from a refused request by this
//!  type alone,
//! so each variant's text and error number must be its own.

use super::device_error::DeviceError;
use crate::requests::{Error, Operation};

#[test]
fn an_unopenable_device_names_the_path_and_the_number() {
    let error = DeviceError::Open {
        path: std::path::PathBuf::from("/dev/zfs-absent"),
        source: std::io::Error::from_raw_os_error(libc::ENOENT),
    };
    assert_eq!(error.errno(), Some(libc::ENOENT));
    assert!(error.request().is_none());
    let printed: String = error.to_string();
    assert!(printed.contains("/dev/zfs-absent"));
    assert!(printed.contains("opening"));
}

#[test]
fn a_request_failure_keeps_what_the_kernel_said() {
    let error: DeviceError = Error::Call {
        operation: Operation::GetBookmarks,
        target: String::from("pool/dataset"),
        errno: libc::ENOENT,
    }
    .into();
    assert_eq!(error.errno(), Some(libc::ENOENT));
    assert!(error.request().is_some());
    assert!(error.to_string().contains("GET_BOOKMARKS"));
}

#[test]
fn an_unstable_reply_says_what_was_asked_for_and_what_is_held() {
    let error = DeviceError::UnstableReply {
        operation: Operation::ObjsetStats,
        target: String::from("pool/dataset"),
        requested: 300_000,
        held: 262_144,
    };
    assert_eq!(error.errno(), None);
    let printed: String = error.to_string();
    assert!(printed.contains("300000"));
    assert!(printed.contains("262144"));
}

#[test]
fn a_missing_reply_names_the_operation_and_the_target() {
    let error = DeviceError::MissingReply {
        operation: Operation::GetFsacl,
        target: String::from("pool/dataset"),
    };
    assert!(error.to_string().contains("GET_FSACL"));
    assert!(error.to_string().contains("pool/dataset"));
}

#[test]
fn the_lower_failure_is_reachable_for_both_wrapped_variants() {
    use std::error::Error as _;
    let open = DeviceError::Open {
        path: std::path::PathBuf::from("/dev/zfs-absent"),
        source: std::io::Error::from_raw_os_error(libc::EACCES),
    };
    assert!(open.source().is_some());
    let request: DeviceError = Error::Call {
        operation: Operation::Destroy,
        target: String::from("pool/dataset"),
        errno: libc::EBUSY,
    }
    .into();
    assert!(request.source().is_some());
    let unstable = DeviceError::UnstableReply {
        operation: Operation::Destroy,
        target: String::from("pool/dataset"),
        requested: 1,
        held: 2,
    };
    assert!(unstable.source().is_none());
}
