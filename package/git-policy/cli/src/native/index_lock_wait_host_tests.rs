//! Controls for the host wait effects against real paths:
//!  an unreadable lock path reports the I/O text.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// A lock path inside a directory without search permission cannot be read,
///  and the wait reports the operating system's text for the failure.
#[test]
#[cfg(unix)]
fn an_unreadable_lock_path_reports_the_io_text() {
    let root: PathBuf = fixture("wait-host-io-text");
    let sealed: PathBuf = root.join("sealed");
    std::fs::create_dir(&sealed).expect("the sealed directory is created");
    let mut effects: HostWaitEffects = HostWaitEffects {
        real_index: sealed.join("index"),
        started: std::time::Instant::now(),
    };
    std::fs::set_permissions(&sealed, std::fs::Permissions::from_mode(0o000)).expect("the directory is sealed");
    let outcome: Result<bool, String> = effects.lock_present();
    std::fs::set_permissions(&sealed, std::fs::Permissions::from_mode(0o700)).expect("the directory is unsealed");
    let failure: String = outcome.expect_err("a path inside a sealed directory is unreadable");
    assert!(failure.contains("Permission denied"), "{failure}");
    remove(root.as_path());
}
