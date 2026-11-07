//! Controls for the typed primitive failure:
//! its words,
//! the feature-missing classification,
//! and the NUL refusal.

use super::*;

/// A path with a NUL byte fails with `EINVAL` before any system call and says so.
#[test]
fn a_path_with_a_nul_byte_is_refused_before_the_call() {
    let bytes: &[u8] = b"a\0b";
    let path: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
    let error: PrimitiveError = c_path("statfs", path.as_path()).expect_err("a NUL byte names no file");
    assert_eq!(error.errno, libc::EINVAL);
    assert!(error.means_feature_missing());
    assert!(error.to_string().contains("statfs"), "{error}");
}

/// The message names the call,
/// the path and the system's words for the error number.
#[test]
fn the_message_names_the_call_the_path_and_the_error() {
    let error: PrimitiveError = PrimitiveError { operation: "FICLONE", path: PathBuf::from("/x/y"), errno: libc::EXDEV };
    let text: String = error.to_string();
    let system_words: String = std::io::Error::from_raw_os_error(libc::EXDEV).to_string();
    for part in ["FICLONE", "/x/y", system_words.as_str()] {
        assert!(text.contains(part), "{text} lacks {part}");
    }
}

/// Only the error numbers that mean "this filesystem cannot do that" send a ladder to its next rung.
#[test]
fn only_missing_feature_errors_fall_through() {
    for errno in [libc::ENOTTY, libc::EOPNOTSUPP, libc::EINVAL, libc::EXDEV, libc::ENOSYS] {
        let error: PrimitiveError = PrimitiveError { operation: "x", path: PathBuf::new(), errno };
        assert!(error.means_feature_missing(), "errno {errno}");
    }
    for errno in [libc::EPERM, libc::ENOENT, libc::EEXIST, libc::ENOSPC, libc::EIO] {
        let error: PrimitiveError = PrimitiveError { operation: "x", path: PathBuf::new(), errno };
        assert!(!error.means_feature_missing(), "errno {errno}");
    }
}

/// The last error of the thread is the one read.
#[test]
fn the_last_os_error_is_read_from_the_thread() {
    let missing: PathBuf = PathBuf::from("/definitely/not/here");
    let _ignored = std::fs::metadata(&missing);
    let error: PrimitiveError = PrimitiveError::last_os_error("stat", missing.as_path());
    assert_eq!(error.errno, libc::ENOENT);
}
